//! STG-01 战斗段弹丸/命中集成自检（task-37）：
//! 真实 ROM 装载（含框形数据）→ capture 帧锚 replay 到 Combat → 逐帧推进；
//! 断言：框形数据装载、玩家弹生成/FrameView 暴露、两次运行确定性与签名全等；
//! 命中事件数如实打印（H-16/伤害路径残项见 `docs/findings/plan-bullets-hit.md` §6，
//! 不对 oracle shots golden 对拍——t50 未转正）。

use std::collections::BTreeMap;
use std::fs;

use game_core::flow::Mode;
use game_platform::{AssetSource, Audio, Buttons, Input, Renderer, CANVAS_LEN};
use game_shell::{Host, ROM_ASSET_PATH};

const ROM_PATH: &str = "../../rom/gunnac_cn.nes";
const PLAYER_TRACE: &str = "../../oracle/traces/gameplay-stage1.player.jsonl";
const COMBAT_START: u64 = 1500;
const RUN_FRAMES: u64 = 2500;

struct FsAssets {
    rom: Vec<u8>,
}

impl AssetSource for FsAssets {
    fn read_all(&mut self, path: &str) -> Option<Vec<u8>> {
        if path == ROM_ASSET_PATH {
            Some(self.rom.clone())
        } else {
            None
        }
    }
}

struct FixedInput {
    buttons: Buttons,
}

impl Input for FixedInput {
    fn buttons(&self) -> Buttons {
        self.buttons
    }
}

struct NullRenderer {
    canvas: Vec<u8>,
}

impl Renderer for NullRenderer {
    fn canvas_mut(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
    fn present(&mut self) {}
}

struct RecAudio;

impl Audio for RecAudio {
    fn write_apu(&mut self, _frame: u32, _reg: u16, _value: u8) {}
    fn end_frame(&mut self, _frame: u32) {}
}

fn json_num(line: &str, key: &str) -> Option<i64> {
    let pat = format!("\"{key}\":");
    let start = line.find(&pat)? + pat.len();
    let rest = line[start..].trim_start();
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '-')).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn buttons_from_held(held: u8) -> Buttons {
    Buttons {
        right: held & 0x01 != 0,
        left: held & 0x02 != 0,
        down: held & 0x04 != 0,
        up: held & 0x08 != 0,
        start: held & 0x10 != 0,
        select: held & 0x20 != 0,
        b: held & 0x40 != 0,
        a: held & 0x80 != 0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CombatRun {
    frames: u64,
    host_signature: u64,
    shapes: usize,
    bullets_seen: u64,
    bullets_at_combat_start: usize,
    hit_events: usize,
    hit_slots: Vec<(u64, u8, u8, u8, u8)>,
    enemy_killed: usize,
    stage_clear: bool,
    final_mode: Mode,
}

fn run_once() -> CombatRun {
    let rom = fs::read(ROM_PATH).expect("ROM 可读");
    let trace_text = fs::read_to_string(PLAYER_TRACE).expect("player trace 可读");
    let mut held_by_frame: BTreeMap<u64, u8> = BTreeMap::new();
    for line in trace_text.lines().filter(|l| !l.trim().is_empty()) {
        let f = json_num(line, "f").expect("f") as u64;
        let held = json_num(line, "held").expect("held") as u8;
        held_by_frame.insert(f, held);
    }

    let mut host = Host::new();
    host.set_auto_flow(false);
    let mut assets = FsAssets { rom };
    assert!(host.load_asset(&mut assets, ROM_ASSET_PATH), "AssetSource 装载 ROM");
    let mut renderer = NullRenderer { canvas: vec![0u8; CANVAS_LEN] };
    let mut audio = RecAudio;

    // capture 帧锚 replay（与 tests/stg01_replay.rs 同口径）：ExitTransition f1193 触发 STG-01+框形装载
    let events: [(u64, game_core::flow::ModeEvent); 7] = [
        (180, game_core::flow::ModeEvent::Black),
        (360, game_core::flow::ModeEvent::Title),
        (900, game_core::flow::ModeEvent::Menu),
        (1080, game_core::flow::ModeEvent::ShopArrival),
        (1193, game_core::flow::ModeEvent::ExitTransition),
        (1260, game_core::flow::ModeEvent::StageStart),
        (1500, game_core::flow::ModeEvent::MidCombat),
    ];

    let mut bullets_seen = 0u64;
    let mut bullets_at_combat_start = 0usize;
    let mut enemy_type_before: [u8; 28] = [0; 28];
    let mut enemy_killed = 0usize;
    let mut stage_clear = false;

    for frame in 0..RUN_FRAMES {
        for (ef, ev) in &events {
            if *ef == frame {
                host.apply_event_hooked(*ev).expect("replay 事件均属已证边集");
            }
        }
        let held = held_by_frame.get(&frame).copied().unwrap_or(0);
        let out = host.tick(
            &FixedInput { buttons: buttons_from_held(held) },
            &mut renderer,
            &mut audio,
        );
        let view = host.frame_view();
        let live = view.player_bullets.iter().filter(|b| b.is_some()).count();
        if live > 0 {
            bullets_seen += live as u64;
        }
        if frame == COMBAT_START {
            bullets_at_combat_start = live;
            for slot in 8..28 {
                enemy_type_before[slot] = host.stage_state().grid.slots[slot].type_;
            }
        }
        for slot in 8..28 {
            let now = host.stage_state().grid.slots[slot].type_;
            if enemy_type_before[slot] != 0 && now == 0 {
                enemy_killed += 1;
            }
            enemy_type_before[slot] = now;
        }
        if out.stage_clear_requested {
            stage_clear = true;
        }
    }

    let shapes = host.shape_data_shapes();
    CombatRun {
        frames: RUN_FRAMES,
        host_signature: host.signature(),
        shapes,
        bullets_seen,
        bullets_at_combat_start,
        hit_events: host.enemy_hit_log().len(),
        hit_slots: host
            .enemy_hit_log()
            .iter()
            .map(|e| (e.frame, e.enemy_slot, e.bullet_slot, e.damage, u8::from(e.fail_closed)))
            .collect(),
        enemy_killed,
        stage_clear,
        final_mode: host.mode(),
    }
}

#[test]
fn stg01_combat_bullets_and_hits_deterministic() {
    let a = run_once();
    let b = run_once();
    assert_eq!(a, b, "两次独立 run 全等（签名/弹/命中/击杀/终态）");
    println!(
        "stg01-combat: frames={} shapes={} bullets_seen={} hits={} killed={} stage_clear={} mode={:?}\n  signature={:#018x}",
        a.frames, a.shapes, a.bullets_seen, a.hit_events, a.enemy_killed, a.stage_clear, a.final_mode,
        a.host_signature
    );
    if a.hit_events > 0 {
        println!("  hit_slots(first 10)={:?}", &a.hit_slots[..a.hit_slots.len().min(10)]);
    }

    // A 级可断言面：框形数据装载（bank2 表/流 A 级）；玩家弹在战斗段被生成并通过 FrameView 暴露
    assert!(a.shapes > 0, "敌框形流已装载（shell 命中管线输入）");
    assert!(a.bullets_seen > 0, "Combat 段播放器 B 长按输入 ⇒ 玩家弹生成（FrameView 可见）");
    assert_eq!(a.final_mode, Mode::Combat, "replay 终态 Combat（无关末门自然触发）");
    // 命中事件数：本 replay 的弹/敌坐标对齐受 H-01/spawn 生成层未收口影响 ⇒ 不做阈值断言，
    // 只锁定确定性（两次全等）；命中正/负例由 host/fire 单测覆盖（见 plan-bullets-hit §5/§8）。
    println!(
        "  note: hits={} （replay 弹/敌对齐未收口，非 H-16 收口声明；正/负例见单测）",
        a.hit_events
    );
}
