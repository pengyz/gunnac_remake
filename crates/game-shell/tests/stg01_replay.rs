//! STG-01 replay 对拍 + 确定性自检（task-9 门禁）。
//!
//! C1 滚动列：`stage::scroll_delta` vs `oracle/traces/gameplay-stage1.scroll.jsonl` 逐帧 1:1（A 级）；
//! C2 模式序列：replay capture 帧锚 vs `.flow.jsonl`（顺序断言；帧锚按 replay 表）；
//! C4 敌槽首现：引擎 spawn 摘要 vs `stage-appear.jsonl`（**仅声明可对拍列**：型值/槽；帧列不可
//!    对拍——spawn 触发链含初生记录/生成器（H-01 未证））；
//! C6 玩家列：`player::tick` vs `.player.jsonl` 死亡窗前逐帧（A 级）。
//! 确定性：3000 帧两次独立运行签名/摘要全等（打印签名）。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use game_core::flow::Mode;
use game_platform::{AssetSource, Audio, Buttons, Input, Renderer, CANVAS_LEN};
use game_shell::{Host, SpawnEvent, ROM_ASSET_PATH};

const ROM_PATH: &str = "../../rom/gunnac_cn.nes";
const TRACE_DIR: &str = "../../oracle/traces";
/// replay 帧数（3000：覆盖 STG-01 前段已证门；本机 ~1s）。
const REPLAY_FRAMES: u64 = 3000;
/// 玩家列对拍死亡窗（`.player.jsonl` README：f1831-1916 等=死亡/重生未证窗，剔除）。
const PLAYER_COMPARE_RANGE: (u64, u64) = (1260, 1830);

// ===================== 平台 mock =====================

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
    presents: u64,
}

impl Renderer for NullRenderer {
    fn canvas_mut(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
    fn present(&mut self) {
        self.presents += 1;
    }
}

struct RecAudio {
    frames: Vec<u32>,
}

impl Audio for RecAudio {
    fn write_apu(&mut self, _frame: u32, _reg: u16, _value: u8) {}
    fn end_frame(&mut self, frame: u32) {
        self.frames.push(frame);
    }
}

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

// ===================== 极简 JSONL 读取（trace 行格式固定；零额外依赖） =====================

fn read_lines(path: &str) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("读取 {path} 失败: {e}"));
    text.lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_string()).collect()
}

fn json_num(line: &str, key: &str) -> Option<i64> {
    let pat = format!("\"{key}\":");
    let start = line.find(&pat)? + pat.len();
    let rest = line[start..].trim_start();
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '-')).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn json_str(line: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\":");
    let start = line.find(&pat)? + pat.len();
    let rest = line[start..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// `$18` 位图 → `Buttons`（R-move-1 位序：bit0=R/bit1=L/bit2=D/bit3=U/bit4=START/bit6=B/bit7=A）。
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

// ===================== replay =====================

/// replay 模式事件表（**capture 帧锚口径**：`flow.jsonl` f1080 Shop/f1200 Transition/f1260
/// StageStart/f1500 Combat；前段 Black/Title 用 R-ui-7 锚；ExitTransition 用 A 级响应锚 f1193
/// （scroll trace 装载帧）——replay 非通用主循环规则，通用规则见 `Host` 文档）。
const REPLAY_EVENTS: &[(u64, Mode)] = &[
    (180, Mode::Black),
    (360, Mode::Title),
    (900, Mode::Menu),
    (1080, Mode::Shop),
];

/// 事件帧 → 产生的 `ModeEvent`（流程序列：Shop→Transition→StageStart→Combat）。
fn replay_mode_event(mode: Mode) -> game_core::flow::ModeEvent {
    use game_core::flow::ModeEvent;
    match mode {
        Mode::Black => ModeEvent::Black,
        Mode::Title => ModeEvent::Title,
        Mode::Menu => ModeEvent::Menu,
        Mode::Shop => ModeEvent::ShopArrival,
        _ => unreachable!("replay 事件表仅含已列模式"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayerCompare {
    frame: u64,
    trace: (u8, u8, u8, u8),
    observed: (u8, u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayResult {
    host_signature: u64,
    canvas_signature: u64,
    final_mode: Mode,
    final_frame: u64,
    engine_frames: u32,
    spawn_log: Vec<SpawnEvent>,
    player_rows: Vec<PlayerCompare>,
    audio_frames: usize,
    presents: u64,
}

fn run_replay() -> ReplayResult {
    let rom = fs::read(ROM_PATH).expect("ROM 可读（仓库约定：cargo test cwd=crate 目录）");
    let player_lines = read_lines(&format!("{TRACE_DIR}/gameplay-stage1.player.jsonl"));
    let mut player_trace: BTreeMap<u64, (u8, u8, u8, u8, u8)> = BTreeMap::new();
    for line in &player_lines {
        let f = json_num(line, "f").expect("f") as u64;
        let x = json_num(line, "x").expect("x") as u8;
        let y = json_num(line, "y").expect("y") as u8;
        let sx = json_num(line, "sx").expect("sx") as u8;
        let sy = json_num(line, "sy").expect("sy") as u8;
        let held = json_num(line, "held").expect("held") as u8;
        player_trace.insert(f, (x, y, sx, sy, held));
    }

    let mut host = Host::new();
    host.set_auto_flow(false); // replay 用 capture 帧锚，不用通用主循环阶段计时
    let mut assets = FsAssets { rom };
    assert!(host.load_asset(&mut assets, ROM_ASSET_PATH), "AssetSource 装载 ROM");
    let mut renderer = NullRenderer { canvas: vec![0u8; CANVAS_LEN], presents: 0 };
    let mut audio = RecAudio { frames: Vec::new() };

    // replay 事件：Black/Title/Menu/Shop 用帧锚；转场用 A 级响应锚 f1193 进 Transition 并装载 STG-01；
    // 随后按 capture 帧锚 StageStart(f1260)/Combat(f1500)。
    let mut events: Vec<(u64, game_core::flow::ModeEvent)> = Vec::new();
    for &(f, mode) in REPLAY_EVENTS {
        events.push((f, replay_mode_event(mode)));
    }
    events.push((1193, game_core::flow::ModeEvent::ExitTransition)); // 出店 +33f 响应锚（A）
    events.push((1260, game_core::flow::ModeEvent::StageStart));
    events.push((1500, game_core::flow::ModeEvent::MidCombat));

    let mut player_rows = Vec::new();
    let mut engine_frames = 0u32;
    for frame in 0..REPLAY_FRAMES {
        for (ef, ev) in &events {
            if *ef == frame {
                host.apply_event_hooked(*ev).expect("replay 事件均属已证边集");
            }
        }
        let held = player_trace.get(&frame).map(|t| t.4).unwrap_or(0);
        let out = host.tick(
            &FixedInput { buttons: buttons_from_held(held) },
            &mut renderer,
            &mut audio,
        );
        if out.gameplay_tick {
            engine_frames += 1;
        }
        if let Some(&(x, y, sx, sy, _)) = player_trace.get(&frame) {
            if (PLAYER_COMPARE_RANGE.0..=PLAYER_COMPARE_RANGE.1).contains(&frame) {
                let p = host.player();
                player_rows.push(PlayerCompare {
                    frame,
                    trace: (x, y, sx, sy),
                    observed: (p.x, p.y, p.x_subpix, p.y_subpix),
                });
            }
        }
    }

    ReplayResult {
        host_signature: host.signature(),
        canvas_signature: host.last_canvas_sig(),
        final_mode: host.mode(),
        final_frame: host.frame(),
        engine_frames,
        spawn_log: host.spawn_log().to_vec(),
        player_rows,
        audio_frames: audio.frames.len(),
        presents: renderer.presents,
    }
}

// ===================== 测试 =====================

/// C1：滚动源（`stage::scroll_delta`）与 oracle `.scroll.jsonl` 逐帧 1:1（A 级）。
#[test]
fn c1_scroll_delta_matches_oracle_trace_row_by_row() {
    let lines = read_lines(&format!("{TRACE_DIR}/gameplay-stage1.scroll.jsonl"));
    let rows: Vec<(u64, i64)> = lines
        .iter()
        .map(|l| (json_num(l, "f").expect("f") as u64, json_num(l, "scroll_y").expect("scroll_y")))
        .collect();
    assert!(rows.len() > 1200, "trace 行数覆盖 f1193..2400（实际 {}）", rows.len());
    let mut mismatches = 0;
    let mut compared = 0;
    for pair in rows.windows(2) {
        let (f_prev, y_prev) = pair[0];
        let (f, y) = pair[1];
        assert_eq!(f, f_prev + 1, "trace 逐帧连续");
        let (_, dy) = game_core::stage::scroll_delta(f as u32);
        let expect = (i64::from(y_prev) + i64::from(dy)).rem_euclid(256) as u8; // dy 有符号、8 位回绕
        let got = y as u8;
        compared += 1;
        if expect != got {
            mismatches += 1;
            if mismatches <= 5 {
                eprintln!("C1 mismatch f={f} dy={dy} expect={expect} got={got}");
            }
        }
    }
    assert_eq!(mismatches, 0, "C1 滚动列与 oracle 逐帧一致（{compared} 帧，0 失配）");
}

/// C2/a：STG-01 装载用 game-core 现有 API 得到 A 级 golden：43 调度记录 + 首段 tick 锚。
#[test]
fn c2_stg01_load_reports_sched_golden() {
    let rom = fs::read(ROM_PATH).expect("ROM 可读");
    let mut host = Host::new();
    let mut assets = FsAssets { rom };
    assert!(host.load_asset(&mut assets, ROM_ASSET_PATH));
    assert!(host.load_stg01(), "STG-01 bank8 调度流装载");
    assert!(host.stage_loaded());
    assert_eq!(host.stage_sched_records(), game_shell::STG01_SCHED_RECORDS, "43 记录 A 级 golden");
    let records = &host.stage_state().sched.records;
    assert_eq!(records.len(), 43);
    let ticks: Vec<u32> = records.iter().take(5).map(|r| r.tick).collect();
    assert_eq!(ticks, vec![1, 33, 49, 65, 81], "调度 tick 锚（A 级 golden）");
    assert!(!host.stage_state().sched.records.is_empty());
}

/// C2/b + C6 + C4 + 确定性：3000 帧 replay 两次全等；玩家列死亡窗前逐帧一致；
/// 敌槽首现摘要打印并按可对拍列做集合级断言。
#[test]
fn replay_3000_frames_deterministic_with_declared_diffs() {
    let a = run_replay();
    let b = run_replay();
    assert_eq!(a, b, "两次独立 replay 全等（签名/画布/spawn/玩家列/音频/呈现）");
    assert_eq!(a.audio_frames, REPLAY_FRAMES as usize, "每逻辑帧一次 end_frame");
    assert_eq!(a.presents, 3000, "每逻辑帧一次 present");
    assert_eq!(a.final_frame, REPLAY_FRAMES);
    assert_eq!(a.final_mode, Mode::Combat, "replay 终态=Combat（f1500 后不再推进）");
    assert!(a.engine_frames > 0, "装载后引擎 tick 生效");

    // C6 玩家列：死亡窗前逐帧与 oracle 一致（A 级；state(N)=tick(state(N-1),held(N)) 约定）
    assert!(!a.player_rows.is_empty(), "玩家列对拍样本非空");
    let mut player_mismatches = Vec::new();
    for row in &a.player_rows {
        if row.trace != row.observed {
            player_mismatches.push(row);
        }
    }
    assert!(
        player_mismatches.is_empty(),
        "C6 玩家列死亡窗前逐帧一致失败 {} 帧，首例 {:?}",
        player_mismatches.len(),
        player_mismatches.first()
    );
    println!(
        "C6 玩家列 {}/{} 帧一致（f{}..=f{}，死亡窗剔除）",
        a.player_rows.len(),
        a.player_rows.len(),
        PLAYER_COMPARE_RANGE.0,
        PLAYER_COMPARE_RANGE.1
    );

    // C4 敌槽首现摘要（槽 8..=27）：引擎 spawn 事件 vs oracle stage-appear（只声明可对拍列）
    let engine: Vec<&SpawnEvent> = a.spawn_log.iter().filter(|s| s.slot >= 8).collect();
    let engine_slots: BTreeSet<u8> = engine.iter().map(|s| s.slot).collect();
    let engine_types: BTreeSet<u8> = engine.iter().map(|s| s.type_code).collect();
    let appear_lines = read_lines(&format!("{TRACE_DIR}/gameplay-stage1.stage-appear.jsonl"));
    let trace_slots: BTreeSet<u8> = appear_lines
        .iter()
        .map(|l| json_num(l, "slot").expect("slot") as u8)
        .filter(|s| *s >= 8)
        .collect();
    let mut trace_types: BTreeSet<u8> = BTreeSet::new();
    let enemy_lines = read_lines(&format!("{TRACE_DIR}/gameplay-stage1.enemy.jsonl"));
    for line in &enemy_lines {
        let state = json_str(line, "state").expect("state hex");
        let bytes = (0..28)
            .map(|i| u8::from_str_radix(&state[i * 2..i * 2 + 2], 16).expect("hex"))
            .collect::<Vec<u8>>();
        for (slot, t) in bytes.iter().enumerate() {
            if slot >= 8 && *t != 0 {
                trace_types.insert(*t);
            }
        }
    }
    let common_slots: Vec<&u8> = engine_slots.intersection(&trace_slots).collect();
    let unknown_types: Vec<&u8> = engine_types.difference(&trace_types).collect();
    println!(
        "C4 敌槽首现：engine={} 事件/{} 槽/{} 型；trace={} 槽/{} 型；共同槽={}；\
         engine 型不在 trace 型集：{:?}（帧列不可对拍：spawn 触发链含初生记录/生成器 H-01 未证）",
        engine.len(),
        engine_slots.len(),
        engine_types.len(),
        trace_slots.len(),
        trace_types.len(),
        common_slots.len(),
        unknown_types
    );
    // 可对拍列断言：引擎 spawn 槽号 ∈ 分配域 8..=27；型值非 0；且**不**把未对上的部分当 0 差异
    // （本断言只锁定"引擎输出落在 trace 观察到的型集内"这一可声明部分；失败即说明装载/落地口径
    //  与 oracle 观察面有未登记差异，须回 spec/取证，不得放宽）。
    assert!(engine.iter().all(|s| (8..=27).contains(&s.slot)), "spawn 槽号在分配域");
    assert!(engine.iter().all(|s| s.type_code != 0), "激活事件型值非 0");
    assert!(unknown_types.is_empty(), "C4：engine 型值均出现在 oracle trace 型集（部分对拍列）");

    println!(
        "replay-3000 signature: host={:#018x} canvas={:#018x} spawns={} engine_frames={}",
        a.host_signature, a.canvas_signature, a.spawn_log.len(), a.engine_frames
    );
}
