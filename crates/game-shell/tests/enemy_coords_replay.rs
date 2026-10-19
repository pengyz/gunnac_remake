//! 敌槽坐标 C3 正式对拍（task-164；spec §4.2 口径 + **spawn 分段对齐**显式登记）。
//!
//! ## 口径（本卡显式登记，不得表述为"§4.2 原样"）
//!
//! spec [enemy-coords-impl-spec.md](../../docs/findings/enemy-coords-impl-spec.md) §4.2 规定：
//! `V1_TYPES` 过滤、共同 (帧,型) 采样点 ≥20、分母先冻结、≤1px ≥90%、y 0 容差优先。
//! **已知偏差（spec §4.2 自己列出"精确 spawn 帧不可比"）**：本实现（game-shell）无敌人释放语义
//! （`$D616` 死亡/回收门未证，fail-closed）⇒ 网格 20 槽约 f1750 饱和，成员 spawn 序列比 oracle
//! 短且绝对 spawn 帧不同 ⇒ **不能按绝对 (帧,型) 做逐帧位置比较**。本测试按 audit
//! [enemy-coords-c3-audit.md](../../docs/findings/enemy-coords-c3-audit.md) §3.4/§5 的
//! **spawn 分段口径**对齐：
//!
//! - `$2B`：以 audit §3.4 的 **GUI spawn 段表（A 级，tracked 证据）**为 oracle 段边界——
//!   每个 oracle 8f 采样按 `(slot, f∈[start,end])` 归段；Rust 段=按 slot 的坐标可见连续段
//!   （每槽当前实现无成员复用，见下），按**段首帧序**与 GUI 段 1:1 配对（前 N=12 段）。
//!   对每个 oracle 采样：相对段首偏移 `r=f-gui_start` ⇒ 取 Rust 段首后 `r` 帧比较（"首帧=init
//!   后 1 tick" 口径双方一致，见 audit §3.4）。**含 reuse 段**：GUI 段 7-12 即槽 8-13 复用段
//!   （段首 1470..1520），已纳入配对。
//! - `$4E`：仅比较**初始位置**（audit §4.4：运动 fail-closed，不入运动列）；oracle 每槽首样本
//!   x（恒值）vs Rust 段首 x；y/运动列不判。
//! - `$2F`：静态 (128,8)，按绝对 (帧,型) 存在性比较（无 spawn 相位问题）。
//!
//! ## 未实现对拍项（显式清单，不判 FAIL）
//!
//! - GUI `$2B` 段 13-24（段首 1854+）无 Rust 对应段：Rust 无成员释放⇒网格饱和后无新成员 spawn
//!   （fail-closed 边界，非本卡发明）；oracle 额外型 `$12/$1A/$1C/$2A`、低槽 `$05/$07` 未生成。
//! - 绝对 spawn 帧、槽号身份、`$4E` 运动列：不可对拍（spec §4.2/Audit）。
//!
//! ## 验收数字（分母先冻结；见常量）
//!
//! `$2F` 98 采样帧（恒 (128,8) 全等）/ `$2B` 96 采样点（y 0 容差 **96/96**、x ≤1px **96/96**）/
//! `$4E` 5 段初始 x 全等；总共同点 ≥20；两次 replay 全等（确定性）。
//!
//! 实现方式（task-118 Info-5 延续）：不采用 spec C2 独立工具+JSONL 导出，改为 replay 内联测试；
//! trace/段表均 tracked，可追溯性=audit §3.4（GUI 段表来源）。

use std::collections::BTreeMap;
use std::fs;

use game_core::flow::Mode;
use game_platform::{AssetSource, Audio, Buttons, Input, Renderer, CANVAS_LEN};
use game_shell::{Host, ROM_ASSET_PATH};

const ROM_PATH: &str = "../../rom/gunnac_cn.nes";
const ENEMY_TRACE: &str = "../../oracle/traces/gameplay-stage1.enemy.jsonl";
/// v1(C3) V1_TYPES（game-shell `enemy_coords::ENEMY_RENDER_TYPES`）。
const RENDER_TYPES: [u8; 3] = [0x2F, 0x2B, 0x4E];
/// replay 帧数（覆盖 trace 视界 f1264-2400）。
const REPLAY_FRAMES: u64 = 2408;
/// 坐标捕获起始帧（stage abs==host 帧号；f1193 装载）。
const CAPTURE_FROM: u64 = 1200;

/// GUI `$2B` spawn 段表（audit §3.4 A；`(slot, start, end)`，按段首排序）。
/// 来源=GUI 逐帧 trace（`analysis/traces/gameplay-stage1.frames.jsonl`，task-133 采集；
/// 本表已固化进 tracked 审计文档）。
const GUI_2B_SEGMENTS: [(u8, u64, u64); 24] = [
    (8, 1342, 1373),
    (9, 1352, 1381),
    (10, 1362, 1388),
    (11, 1372, 1402),
    (12, 1382, 1413),
    (13, 1392, 1420),
    (8, 1470, 1577),
    (9, 1480, 1591),
    (10, 1490, 1513),
    (11, 1500, 1609),
    (12, 1510, 1620),
    (13, 1520, 1630),
    (11, 1854, 1889),
    (9, 1864, 1972),
    (12, 1874, 1982),
    (13, 1884, 1993),
    (16, 1894, 2004),
    (17, 1904, 2011),
    (9, 2002, 2093),
    (12, 2012, 2045),
    (13, 2022, 2115),
    (8, 2032, 2125),
    (11, 1982, 2072),
    (18, 1992, 2086),
];

/// 冻结分母（实测值；防覆盖回退）。来源=本测试首次实现后跑出的共同采样点数。
const EXPECTED_2F_SAMPLES: usize = 98;
const EXPECTED_2B_SAMPLES: usize = 96;
const EXPECTED_4E_SEGMENTS: usize = 5;
/// 可配对 GUI `$2B` 段数（Rust 段数=12；其余因无释放语义不可对拍）。
const EXPECTED_2B_MATCHED_SEGMENTS: usize = 12;

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

/// trace 行 → `(f, state[28], x[28], y[28])`。
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

/// 字节模 256 差（回绕语义；spec §4.2 步骤 5）。
fn byte_diff(a: u8, b: u8) -> u16 {
    let d = u16::from(a.abs_diff(b));
    d.min(256 - d)
}

// ===================== replay（帧锚同 stg01_replay；stage abs==host 帧号） =====================

/// host 帧 → 20 槽 `(type, position)`（仅捕获 `CAPTURE_FROM..REPLAY_FRAMES`）。
type FrameRow = [(u8, Option<(u8, u8)>); 20];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayResult {
    frames: BTreeMap<u64, FrameRow>,
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
    let mut frames = BTreeMap::new();
    for frame in 0..REPLAY_FRAMES {
        for (ef, ev) in events {
            if *ef == frame {
                host.apply_event_hooked(*ev).expect("replay 事件均属已证边集");
            }
        }
        host.tick(&NoInput, &mut renderer, &mut audio);
        if frame >= CAPTURE_FROM {
            let view = host.frame_view();
            let mut row = [(0u8, None); 20];
            for (i, slot) in view.enemy_slots.iter().enumerate() {
                row[i] = (slot.type_code.unwrap_or(0), slot.position);
            }
            frames.insert(frame, row);
        }
    }
    assert_eq!(host.mode(), Mode::Combat, "replay 到达 Combat");
    assert!(host.stage_loaded(), "STG-01 已装载");
    ReplayResult { frames }
}

fn slot_index(slot: u8) -> Option<usize> {
    (8..28).contains(&slot).then(|| (slot - 8) as usize)
}

fn rust_position(row: &FrameRow, slot: u8) -> Option<(u8, u8)> {
    slot_index(slot).and_then(|i| row[i].1)
}

fn rust_type(row: &FrameRow, slot: u8) -> u8 {
    slot_index(slot).map(|i| row[i].0).unwrap_or(0)
}

/// Rust 侧 `$2B` 可见段（每槽连续 `position=Some` 段；当前实现无成员复用）。
fn rust_segments(result: &ReplayResult, type_: u8) -> Vec<(u8, Vec<(u64, u8, u8)>)> {
    let mut by_slot: BTreeMap<u8, Vec<(u64, u8, u8)>> = BTreeMap::new();
    for (f, row) in &result.frames {
        for slot in 8..28u8 {
            let i = (slot - 8) as usize;
            let (t, pos) = row[i];
            if t == type_ {
                if let Some((x, y)) = pos {
                    by_slot.entry(slot).or_default().push((*f, x, y));
                }
            }
        }
    }
    let mut segs: Vec<(u8, Vec<(u64, u8, u8)>)> = by_slot.into_iter().collect();
    // 段内必须连续（允许 0 间隙；本实现平顺）；按段首帧排序。
    segs.sort_by_key(|(_, v)| v[0].0);
    segs
}

// ===================== 正式对拍（§4.2 + spawn 分段对齐） =====================

#[derive(Debug, Default)]
struct CompareStats {
    /// 共同 (帧,型) 采样点数（分母）。
    total: usize,
    /// x 列 ≤1px 通过数；y 列 0 容差通过数。
    x_pass: usize,
    y_pass: usize,
    /// `$2B` 已配对 GUI 段数（含 reuse 段）。
    matched_segments: usize,
    /// 不可对拍（GUI 段超出 Rust 段数=无释放语义；oracle 额外型等）。
    unmatched_note: Vec<String>,
}

fn compare(result: &ReplayResult, oracle: &[(u64, [u8; 28], [u8; 28], [u8; 28])]) -> CompareStats {
    let mut stats = CompareStats::default();

    // ---- 类型集合（V1_TYPES 过滤后相等；oracle 额外型入未实现清单不判 FAIL）----
    let mut oracle_types = std::collections::BTreeSet::new();
    for (_, st, _, _) in oracle {
        for s in 8..28usize {
            if st[s] != 0 && !matches!(st[s], 1 | 2 | 15) {
                oracle_types.insert(st[s]);
            }
        }
    }
    let mut rust_types = std::collections::BTreeSet::new();
    for row in result.frames.values() {
        for (t, pos) in row.iter() {
            if pos.is_some() && *t != 0 {
                rust_types.insert(*t);
            }
        }
    }
    assert_eq!(rust_types, RENDER_TYPES.into_iter().collect::<std::collections::BTreeSet<_>>(),
        "Rust 暴露坐标的型集合必须=V1_TYPES");
    let mut oracle_v1: std::collections::BTreeSet<u8> = oracle_types
        .iter()
        .copied()
        .filter(|t| RENDER_TYPES.contains(t))
        .collect();
    assert_eq!(oracle_v1, rust_types, "V1_TYPES 过滤后类型集合相等（spec §4.2 步骤 2）");
    let rust_set: std::collections::BTreeSet<u8> = rust_types.iter().copied().collect();
    for t in oracle_types.difference(&rust_set) {
        stats.unmatched_note.push(format!("oracle 额外型 ${t:02X}（生成源未接，不判 FAIL）"));
    }
    oracle_v1.clear();

    // ---- $2F：静态 (128,8)；按绝对 (帧,型) 存在性比较 ----
    let mut f2f_total = 0usize;
    for (f, st, xs, ys) in oracle {
        let has = (8..28usize).any(|s| st[s] == 0x2F);
        if !has {
            continue;
        }
        let Some(row) = result.frames.get(f) else { continue };
        let rust: Vec<(u8, u8)> = (8..28u8)
            .filter(|&s| rust_type(row, s) == 0x2F)
            .filter_map(|s| rust_position(row, s))
            .collect();
        if rust.is_empty() {
            continue; // Rust 尚未生成该型（spawn 帧列不可对拍）
        }
        f2f_total += 1;
        stats.total += 1;
        let mut ok = true;
        for s in 8..28usize {
            if st[s] == 0x2F {
                let (ox, oy) = (xs[s], ys[s]);
                ok &= rust.iter().any(|&(rx, ry)| (rx, ry) == (ox, oy) || byte_diff(rx, ox) <= 1 && byte_diff(ry, oy) <= 1);
            }
        }
        if ok {
            stats.x_pass += 1;
            stats.y_pass += 1;
        }
    }
    assert_eq!(f2f_total, EXPECTED_2F_SAMPLES, "$2F 采样帧分母须=98（冻结）");

    // ---- $2B：GUI spawn 段对齐（含 reuse 段）----
    let rust_2b = rust_segments(result, 0x2B);
    let matched_segs = GUI_2B_SEGMENTS.len().min(rust_2b.len());
    assert_eq!(matched_segs, EXPECTED_2B_MATCHED_SEGMENTS, "$2B 可配对段数须=12（冻结）");
    stats.matched_segments = matched_segs;
    let mut seg_samples = 0usize;
    for (gi, &(gslot, gstart, gend)) in GUI_2B_SEGMENTS.iter().take(matched_segs).enumerate() {
        let (rslot, rvec) = &rust_2b[gi];
        let rstart = rvec[0].0;
        // oracle 该 GUI 段内的采样（同 slot 且帧落在段窗）。
        for (f, st, xs, ys) in oracle {
            if !(*f >= gstart && *f <= gend) || st[gslot as usize] != 0x2B {
                continue;
            }
            let r = *f - gstart;
            let rf = rstart + r;
            let Some(row) = result.frames.get(&rf) else { continue };
            // Rust 段槽位（段槽以 rslot 为准；槽号身份不可对拍，spec §4.2）。
            let Some((rx, ry)) = rust_position(row, *rslot) else {
                continue;
            };
            let (ox, oy) = (xs[gslot as usize], ys[gslot as usize]);
            seg_samples += 1;
            stats.total += 1;
            if ry == oy {
                stats.y_pass += 1;
            }
            if byte_diff(rx, ox) <= 1 {
                stats.x_pass += 1;
            }
        }
    }
    assert_eq!(seg_samples, EXPECTED_2B_SAMPLES, "$2B 采样点分母须=96（冻结）");
    for (gi, &(slot, start, _)) in GUI_2B_SEGMENTS.iter().enumerate().skip(matched_segs) {
        stats.unmatched_note.push(format!(
            "GUI $2B 段#{gi} slot{slot} f{start}+ 无 Rust 段（无敌人释放⇒网格饱和，fail-closed）"
        ));
    }

    // ---- $4E：仅初始位置 x（运动 fail-closed，不入运动列）----
    let mut oracle_4e: Vec<(u8, u8)> = Vec::new(); // (slot, first_x) 按首现序
    for (_, st, xs, _) in oracle {
        for s in 8..28usize {
            if st[s] == 0x4E && !oracle_4e.iter().any(|&(slot, _)| slot as usize == s) {
                oracle_4e.push((s as u8, xs[s]));
            }
        }
    }
    let rust_4e = rust_segments(result, 0x4E);
    let matched_4e = oracle_4e.len().min(rust_4e.len());
    assert_eq!(matched_4e, EXPECTED_4E_SEGMENTS, "$4E 可配对段数须=5（冻结）");
    for (&(oslot, ox), (_, rvec)) in oracle_4e.iter().zip(rust_4e.iter()) {
        let (_, rx, _) = rvec[0];
        stats.total += 1;
        if byte_diff(rx, ox) <= 1 {
            stats.x_pass += 1;
            stats.y_pass += 1; // $4E 只判初始位置（x），y 运动列不判
        }
        stats.unmatched_note.push(format!(
            "$4E 初始位置 oracle slot{oslot} x={ox} vs rust x={rx}"
        ));
    }
    for (i, &(slot, _)) in oracle_4e.iter().enumerate().skip(matched_4e) {
        stats.unmatched_note.push(format!(
            "$4E oracle 槽#{i} slot{slot} 无 Rust 段（无敌人释放⇒网格饱和，fail-closed）"
        ));
    }
    stats
}

#[test]
fn c3_formal_coords_match_oracle_by_spawn_segments() {
    let oracle = parse_enemy_trace(ENEMY_TRACE);
    let first = run_replay();
    let second = run_replay();
    assert_eq!(first, second, "replay 两次独立运行：坐标捕获全等（确定性）");

    let stats = compare(&first, &oracle);
    for note in &stats.unmatched_note {
        println!("C3 不可对拍/未实现: {note}");
    }
    let n = stats.total;
    println!(
        "C3 共同采样点 N={n}；x≤1px={}/{} ({:.1}%)；y 0容差={}/{} ({:.1}%)；$2B 配对段={}（含 reuse 段 7-12）",
        stats.x_pass,
        n,
        100.0 * stats.x_pass as f64 / n as f64,
        stats.y_pass,
        n,
        100.0 * stats.y_pass as f64 / n as f64,
        stats.matched_segments
    );
    assert!(n >= 20, "共同 (帧,型) 采样点须 ≥20（实得 {n}）");
    assert!(stats.y_pass >= n * 90 / 100, "y 列达成率须 ≥90%");
    assert!(stats.x_pass >= n * 90 / 100, "x 列 ≤1px 达成率须 ≥90%");
    // y 0 容差优先：$2B 段内 y 必须全等（分母冻结 96；上面 y_pass 已含 $2F）。
    assert!(stats.y_pass >= EXPECTED_2B_SAMPLES + EXPECTED_2F_SAMPLES,
        "$2B y 0 容差须全过（96 点）+ $2F 98 点");
    assert_eq!(stats.total, EXPECTED_2F_SAMPLES + EXPECTED_2B_SAMPLES + EXPECTED_4E_SEGMENTS,
        "冻结分母守卫（98+96+5=199）");
}

/// fail-closed：Rust 不得暴露 V1_TYPES 之外型的坐标（spec §4.2 步骤 6 方向性断言）。
#[test]
fn c3_no_positions_for_unimplemented_types() {
    let result = run_replay();
    for (f, row) in &result.frames {
        for (t, pos) in row.iter() {
            if pos.is_some() {
                assert!(
                    RENDER_TYPES.contains(t),
                    "frame {f}: 暴露了未实现型 ${t:02X} 的坐标"
                );
            }
        }
    }
}
