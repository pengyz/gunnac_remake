//! 敌槽已证坐标列对拍（task-45；task-118 F1 收窄口径登记；**阈值不放宽**）。
//!
//! **本卡收窄口径（显式登记，task-118 F1）**：只对拍 **`$2F` 坐标列（x/y 双列）**。
//! spec `enemy-coords-impl-spec.md` §4.2 步骤 2/6 的 `V1_TYPES={$2F,$2B,$4E}` 过滤后
//! **类型集合相等/反例保护断言未实现**——`$2B/$4E` 坐标写者未逐值审计、生成源未接
//! （本卡只做坐标列可见化；差异见 `docs/findings/plan-enemy-render.md` §4/§8）。
//! **保留的 fail-closed 断言**：非 `RENDER_TYPES` 型（含 `$2B/$4E`）`position` 必须 `None`；
//! rust 不得暴露 `RENDER_TYPES` 外型的坐标（spec 步骤 6 的方向性部分覆盖）。
//!
//! - 共同 (帧,型) 采样点：按 spec §4.2 步骤 3 的**计数定义**（同帧同型只计 1 点）；分母先冻结；
//!   **N 已钉死=28（`EXPECTED_COMMON_SAMPLES`，防覆盖回退）**；≤1px 达成率 ≥90%
//!   （本测试对 $2F x/y 双列：trace 恒 (128,8)，实测 28/28=100%）。
//! - 不可对拍列（不做断言）：精确 spawn 帧（8f 采样+1 帧滴灌偏差）、槽号身份、帧内运动；
//!   oracle 额外型（`$12/$1A/$1C/$2A`、低槽 `$05/$07`）入未实现清单，不判 FAIL。
//!
//! **实现方式（task-118 Info-5 登记）**：spec C2 原设计的独立工具
//! `tools/enemy_coords_compare.py` + FrameView JSONL 导出**未采用**；改为本 replay 内联测试
//! （直接读 oracle trace、内存比较）。优点=零新增工具/依赖、可随 crate 测试跑；缺点=trace
//! 可追溯性弱于"独立工具+导出产物"（无独立 JSONL 中间物），跨实现复用需另卡。
//!
//! trace 仅 8 帧采样（143 行，f1264-2400）；replay 帧锚与 trace 同域（`ExitTransition`@f1193 装载，
//! 故 host 帧号==stage abs 帧号）。

use std::collections::BTreeMap;
use std::fs;

use game_core::flow::Mode;
use game_platform::{AssetSource, Audio, Buttons, Input, Renderer, CANVAS_LEN};
use game_shell::{Host, ROM_ASSET_PATH};

const ROM_PATH: &str = "../../rom/gunnac_cn.nes";
const ENEMY_TRACE: &str = "../../oracle/traces/gameplay-stage1.enemy.jsonl";
/// v1 已声明可对拍的敌型（= game-shell `enemy_coords::has_proven_coords`；**收窄：仅 $2F**）。
const RENDER_TYPES: &[u8] = &[0x2F];
/// 共同 (帧,型) 采样点期望分母（实测 28；钉死防覆盖回退，来源=trace $2F 首序槽帧 ∩ Rust 暴露帧）。
const EXPECTED_COMMON_SAMPLES: usize = 28;
/// replay 帧数（覆盖 trace 视界 f1264-2400）。
const REPLAY_FRAMES: u64 = 2408;

// ===================== 平台 mock =====================

struct NoInput;

impl Input for NoInput {
    fn buttons(&self) -> Buttons {
        Buttons::default()
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

struct NullAudio;

impl Audio for NullAudio {
    fn write_apu(&mut self, _frame: u32, _reg: u16, _value: u8) {}
    fn end_frame(&mut self, _frame: u32) {}
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

// ===================== 极简 trace 解析（零额外依赖；行格式固定） =====================

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

fn json_hex(line: &str, key: &str) -> Option<Vec<u8>> {
    let pat = format!("\"{key}\":");
    let start = line.find(&pat)? + pat.len();
    let rest = line[start..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    let hex = &rest[..end];
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = (bytes[i] as char).to_digit(16)?;
        let lo = (bytes[i + 1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
    }
    Some(out)
}

/// trace 行 → (f, state[28], x[28], y[28])。
fn parse_enemy_trace(path: &str) -> Vec<(u64, [u8; 28], [u8; 28], [u8; 28])> {
    let mut rows = Vec::new();
    for line in read_lines(path) {
        let f = json_num(&line, "f").expect("f") as u64;
        let st = json_hex(&line, "state").expect("state");
        let x = json_hex(&line, "x").expect("x");
        let y = json_hex(&line, "y").expect("y");
        assert_eq!((st.len(), x.len(), y.len()), (28, 28, 28), "trace 槽宽=28");
        let mut st_a = [0u8; 28];
        let mut x_a = [0u8; 28];
        let mut y_a = [0u8; 28];
        st_a.copy_from_slice(&st);
        x_a.copy_from_slice(&x);
        y_a.copy_from_slice(&y);
        rows.push((f, st_a, x_a, y_a));
    }
    rows
}

// ===================== replay（帧锚同 stg01_replay；stage abs==host 帧号） =====================

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayResult {
    /// host 帧 → 各槽位 (type_code, position)（仅记录 trace 采样帧）。
    samples: BTreeMap<u64, [(u8, Option<(u8, u8)>); 20]>,
}

fn run_replay() -> ReplayResult {
    use game_core::flow::ModeEvent;
    let rom = fs::read(ROM_PATH).expect("ROM 可读（cargo test cwd=crate 目录）");
    let mut host = Host::new();
    host.set_auto_flow(false); // 用 capture 帧锚（同 stg01_replay）
    let mut assets = FsAssets { rom };
    assert!(host.load_asset(&mut assets, ROM_ASSET_PATH), "AssetSource 装载 ROM");
    let mut renderer = NullRenderer { canvas: vec![0u8; CANVAS_LEN] };
    let mut audio = NullAudio;

    let events: &[(u64, ModeEvent)] = &[
        (180, ModeEvent::Black),
        (360, ModeEvent::Title),
        (900, ModeEvent::Menu),
        (1080, ModeEvent::ShopArrival),
        (1193, ModeEvent::ExitTransition), // A 级响应锚 + STG-01 装载
        (1260, ModeEvent::StageStart),
        (1500, ModeEvent::MidCombat),
    ];
    let sample_frames: std::collections::BTreeSet<u64> = parse_enemy_trace(ENEMY_TRACE)
        .into_iter()
        .map(|(f, _, _, _)| f)
        .collect();

    let mut samples = BTreeMap::new();
    for frame in 0..REPLAY_FRAMES {
        for (ef, ev) in events {
            if *ef == frame {
                host.apply_event_hooked(*ev).expect("replay 事件均属已证边集");
            }
        }
        host.tick(&NoInput, &mut renderer, &mut audio);
        if sample_frames.contains(&frame) {
            let view = host.frame_view();
            let mut row = [(0u8, None); 20];
            for (i, slot) in view.enemy_slots.iter().enumerate() {
                row[i] = (slot.type_code.unwrap_or(0), slot.position);
            }
            samples.insert(frame, row);
        }
    }
    assert_eq!(host.mode(), Mode::Combat, "replay 到达 Combat");
    assert!(host.stage_loaded(), "STG-01 已装载");
    ReplayResult { samples }
}

/// oracle 每型「首现序槽」的 (x,y)（spec §4.2 步骤 3：同帧同型只取首序槽）。
fn oracle_first_order_positions(
    rows: &[(u64, [u8; 28], [u8; 28], [u8; 28])],
    type_: u8,
) -> BTreeMap<u64, (u8, u8)> {
    // 先建立槽首现序（按 trace 帧序）。
    let mut first_frame: BTreeMap<u8, u64> = BTreeMap::new();
    for (f, st, _x, _y) in rows {
        for slot in 8..28usize {
            if st[slot] == type_ {
                first_frame.entry(slot as u8).or_insert(*f);
            }
        }
    }
    let mut order: Vec<(u8, u64)> = first_frame.into_iter().collect();
    order.sort_by_key(|(slot, f)| (*f, *slot));
    let Some((first_slot, _)) = order.first().copied() else {
        return BTreeMap::new();
    };
    // 每帧取首现序槽的坐标（同帧同型只计 1 点）。
    let mut out = BTreeMap::new();
    for (f, st, x, y) in rows {
        let s = first_slot as usize;
        if st[s] == type_ {
            out.insert(*f, (x[s], y[s]));
        }
    }
    out
}

fn replay_first_order_position(row: &[(u8, Option<(u8, u8)>); 20], type_: u8) -> Option<(u8, u8)> {
    row.iter()
        .find(|(t, pos)| *t == type_ && pos.is_some())
        .and_then(|(_, pos)| *pos)
}

/// 字节模 256 差（回绕语义；spec §4.2 步骤 5）。
fn byte_diff(a: u8, b: u8) -> u16 {
    let d = u16::from(a.abs_diff(b));
    d.min(256 - d)
}

#[test]
fn c2_enemy_proven_coords_match_oracle_columns() {
    let oracle_rows = parse_enemy_trace(ENEMY_TRACE);
    let first = run_replay();
    let second = run_replay();
    assert_eq!(first, second, "replay 两次独立运行：坐标可见列采样全等（确定性）");

    // fail-closed / 方向性断言（**不是** spec §4.2 步骤 2 的 V1_TYPES 过滤后集合相等；
    // 该断言未实现，收窄口径见文件头 task-118 F1）：rust 暴露坐标的型 ⊆ RENDER_TYPES。
    for (f, row) in &first.samples {
        for (t, pos) in row.iter() {
            if pos.is_some() {
                assert!(
                    RENDER_TYPES.contains(t),
                    "rust 暴露了未声明可渲染型的坐标：frame {f} type ${t:02X}"
                );
            }
        }
    }

    // 共同 (帧,型) 采样点 + ≤1px 达成率（分母先冻结）。
    let mut total = 0usize;
    let mut passed = 0usize;
    for &type_ in RENDER_TYPES {
        let oracle = oracle_first_order_positions(&oracle_rows, type_);
        for (f, (ox, oy)) in &oracle {
            let Some(row) = first.samples.get(f) else { continue };
            let Some((rx, ry)) = replay_first_order_position(row, type_) else {
                continue; // rust 该帧无此型 ⇒ 不构成共同采样点（不判 FAIL；spawn 帧列不可对拍）
            };
            total += 1;
            if byte_diff(*ox, rx) <= 1 && byte_diff(*oy, ry) <= 1 {
                passed += 1;
            } else {
                eprintln!(
                    "坐标偏差: frame {f} type ${type_:02X} oracle=({ox},{oy}) rust=({rx},{ry})"
                );
            }
        }
    }
    println!("C2 共同 (帧,型) 采样点 N={total} / ≤1px 通过={passed}");
    // N 钉死（task-118 Info-1）：期望分母来源=trace $2F 首序槽帧 ∩ Rust 暴露帧；防覆盖回退。
    assert_eq!(
        total, EXPECTED_COMMON_SAMPLES,
        "共同采样点分母须=28（实测值钉死；若 trace/replay 变更须显式说明并更新此常量）"
    );
    assert!(total >= 20, "共同采样点须 ≥20（spec 下限）");
    let rate = passed as f64 / total as f64;
    assert!(rate >= 0.90, "≤1px 达成率须 ≥90%（实得 {:.1}%）", rate * 100.0);

    // fail-closed 型不得暴露坐标（同一次 replay 采样断言；不重复跑）。
    for (f, row) in &first.samples {
        for (t, pos) in row.iter() {
            if RENDER_TYPES.contains(t) {
                continue;
            }
            assert!(
                pos.is_none(),
                "fail-closed 型不得暴露坐标：frame {f} type ${t:02X} pos={pos:?}"
            );
        }
    }
    // $2F 若出现，坐标必须=已证恒定 (128,8)（x/y 双列）。
    let mut leader_seen = 0usize;
    for (f, row) in &first.samples {
        for (t, pos) in row.iter() {
            if *t == 0x2F && pos.is_some() {
                leader_seen += 1;
                assert_eq!(*pos, Some((128, 8)), "frame {f}: $2F 已证坐标恒 (128,8)");
            }
        }
    }
    assert!(leader_seen > 0, "trace 视界内应有 $2F 已证坐标暴露（实得 {leader_seen} 槽帧）");
    println!("C2 $2F 已证坐标暴露槽帧数={leader_seen}");
}
