//! game-core::stage —— 关卡卷轴推进器（M3 stage 卷轴，t73）。
//!
//! 语义来源：`docs/spec/stage-timeline.md` R-bg-scroll-1..4 + `docs/findings/bg-content-line.md`
//! §4（A 级，L3 已判）。**双卷轴通道建模**（R-bg-scroll-4）：关卡卷轴（$1C/$1D zp 对）与
//! UI 卷轴（bank1:$AB3B 直写 $2005）**分立**——类型级隔离，互不写入。
//!
//! quirk 保真（D4）：
//! - `$1C/$1D` 为**字节直差**（观测域内无亚像素配对——x 恒 0、y 整差；mod 256 回绕，
//!   f1317 y=0 → f1318 y=238 实测回绕）；
//! - UI 卷轴 `$AB3B`：`STA $2005` **同值双写**（X/Y 同值，字面指令流）；
//! - 脉冲相位基准 = **f1318**（1318+120k，10/10 命中）。

/// 关卡卷轴状态（$1C=x、$1D=y，zp 对）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StageScroll {
    pub x: u8,
    pub y: u8,
}

/// 逐帧增量（R-bg-scroll-1..3，有符号）：
/// - f1193：阶段边界**装载 −48**；
/// - f1194..=1213：**静止 20 帧**（0）；
/// - f1214+：基础 **−2/帧**；`f1318+120k`（k≥0）**单帧脉冲 −18**（R-bg-scroll-2）。
pub fn scroll_delta(frame: u32) -> (i8, i8) {
    if frame == 1193 {
        return (0, -48);
    }
    if (1194..=1213).contains(&frame) {
        return (0, 0);
    }
    if frame >= 1318 && (frame - 1318) % 120 == 0 {
        return (0, -18);
    }
    (0, -2)
}

/// 卷轴推进一步（8 位回绕，观测语义）。
pub fn scroll_tick(frame: u32, st: &StageScroll) -> StageScroll {
    let (dx, dy) = scroll_delta(frame);
    StageScroll {
        x: st.x.wrapping_add(dx as u8),
        y: st.y.wrapping_add(dy as u8),
    }
}

/// 脉冲相位判定（R-bg-scroll-2）：f≥1318 且 (f−1318)%120==0。
pub fn is_pulse_frame(frame: u32) -> bool {
    frame >= 1318 && (frame - 1318) % 120 == 0
}

/// UI 卷轴通道（R-bg-scroll-4：bank1:$AB3B 与关卡卷轴**分立**）。
/// `$2005` 同值双写（X/Y 同值，字面指令流）；写值语义由 UI 状态驱动——本通道只建写形状。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiScroll {
    pub value: u8,
}

impl UiScroll {
    /// `$AB3B` 写序列：返回 ($2005 写#1, $2005 写#2)——同值双写 quirk。
    pub fn ppu_scroll_writes(&self) -> (u8, u8) {
        (self.value, self.value)
    }
}

// ============ STG 内容集成（t84：关卡目录→生成记录→VM 脚本驱动） ============

/// 关卡目录行（R-stage-dir-1，bank8:$A000 每关 8B×8）：
/// `{root 流指针(2,LE), 图案表指针(2,LE), 参数对(2), 尾指针(2,LE)}`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageDirEntry {
    pub root_ptr: u16,
    pub pattern_ptr: u16,
    pub params: (u8, u8),
    pub tail_ptr: u16,
}

/// 关卡目录表解析（bank8:$A000-$A040，8 关全表）。`$0180`=关号×8 索引（R-stage-dir-2）。
pub fn parse_stage_dir(bank8: &[u8]) -> [StageDirEntry; 8] {
    let mut out = [StageDirEntry { root_ptr: 0, pattern_ptr: 0, params: (0, 0), tail_ptr: 0 }; 8];
    for (i, row) in out.iter_mut().enumerate() {
        let b = &bank8[i * 8..i * 8 + 8];
        *row = StageDirEntry {
            root_ptr: u16::from_le_bytes([b[0], b[1]]),
            pattern_ptr: u16::from_le_bytes([b[2], b[3]]),
            params: (b[4], b[5]),
            tail_ptr: u16::from_le_bytes([b[6], b[7]]),
        };
    }
    out
}

/// 目录行索引（R-stage-dir-2）：`$0180` = 关号×8。
pub fn stage_dir_index(stage_number: u8) -> u8 {
    stage_number.wrapping_mul(8)
}

/// STG-01 生成记录（t79 §2，bank8:$ADD0 起 8×5B）：`{p0,p1,p2,行为脚本指针(2,LE)}`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnRecord {
    /// X 坐标 = p0−$10（进位 quirk 同族，t79 表逐值 175/123/139/119/119/117/117/128）。
    pub x: u8,
    /// p1 原值（**W26 参数语义未证**——4B 装载机拆包为候选注记，不入结论）。
    pub p1: u8,
    /// p2 原值（W26 同）。
    pub p2: u8,
    /// 行为脚本指针（bank8 窗内：$B3E6/$B2B4/$B235/$B21D 四型）。
    pub script_lo: u8,
    pub script_hi: u8,
}

impl SpawnRecord {
    pub fn script_ptr(&self) -> u16 {
        u16::from_le_bytes([self.script_lo, self.script_hi])
    }
}

/// 生成记录流解析（8×5B）。
pub fn parse_spawn_records(region: &[u8]) -> Vec<SpawnRecord> {
    region
        .chunks_exact(5)
        .take(8)
        .map(|b| SpawnRecord {
            x: b[0].wrapping_sub(0x10), // p0−$10 quirk（t79 实证）
            p1: b[1],
            p2: b[2],
            script_lo: b[3],
            script_hi: b[4],
        })
        .collect()
}

/// tick 间隔三档（R-stage-tick-1，t83 A 级实测：8/10/15 帧）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickTier {
    S8 = 8,
    S10 = 10,
    S15 = 15,
}

/// 块级域表（t83 §1 六段实录）：每段 (起帧, 止帧, 间隔档)。
pub const TICK_DOMAINS: [(u32, u32, TickTier); 6] = [
    (1341, 1391, TickTier::S10), // 段A：每 tick 1 发
    (1460, 1519, TickTier::S10), // 段B：混突发（1460 同帧 5 发）
    (1597, 1657, TickTier::S15), // 段C
    (1853, 1903, TickTier::S10), // 段D
    (1981, 2011, TickTier::S10), // 段E
    (2232, 2360, TickTier::S8),  // 段F：长段
];

/// `$FD` 计数↔间隔**已实测档对**（R-stage-tick-2）：`10 FD`(16)↔15f、`08 FD`(8)↔8f。
pub const PROVEN_FD_PAIRS: [(u8, u8); 2] = [(16, 15), (8, 8)];

/// `$FD` 计数→间隔候选公式：`计数 ≈ 间隔+1`（R-stage-tick-2 候选注；逐块公式未全闭=W28′）。
pub fn fd_interval_candidate(count: u8) -> u8 {
    count.saturating_sub(1)
}

/// 波次调度（t79 §3：链 8 段 ↔ 记录 8 条 **1:1**；时刻层按 R-stage-tick-1..3 校准——
/// tick 间隔=块级三档（t83），替换 t84 的 8-10f 参数化默认）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WaveSchedule;

impl WaveSchedule {
    /// 第 `segment`（0..8）段的记录索引（1:1 驱动）。
    pub fn record_index(&self, segment: usize) -> Option<usize> {
        if segment < 8 {
            Some(segment)
        } else {
            None // EN-9+/Boss 段 fail-closed（未证不发明）
        }
    }

    /// 帧所在块域的间隔档（R-stage-tick-1）；不在任何域 ⇒ None。
    pub fn interval_at(&self, frame: u32) -> Option<u8> {
        for (start, end, tier) in TICK_DOMAINS {
            if (start..=end).contains(&frame) {
                return Some(tier as u8);
            }
        }
        None
    }

    /// 帧是否为块域网格上的 tick 帧（起帧+k×间隔，域内）。
    pub fn is_tick_frame(&self, frame: u32) -> bool {
        for (start, end, tier) in TICK_DOMAINS {
            if (start..=end).contains(&frame) {
                let step = tier as u32;
                return (frame - start) % step == 0;
            }
        }
        false
    }

    /// 段内第 `repeat` 次重复的预计帧偏移（按档间隔；W28′ 逐块公式未全闭）。
    pub fn frame_offset(&self, repeat: u32, tier: TickTier) -> u32 {
        repeat * tier as u32
    }
}

/// 行为脚本→VM 槽挂接（t71 归位注记：VM=**地形脚本机**，敌行为脚本是 VM 槽侧语义——
/// 分层如实：生成记录只决定初始 PC/X，更新由 terrain VM 走字节码）。
/// 返回初始化的 VM 槽（寿命/循环计数按 4B 装载机默认：计数 $10、寿命 0（待生成方写））。
pub fn spawn_to_vm_slot(rec: &SpawnRecord) -> crate::terrain::TerrainSlot {
    crate::terrain::TerrainSlot {
        x: rec.x,
        param: 0,
        sub_lo: 0,
        sub_hi: 0,
        loop_cnt: 0x10,
        opcode: 0,
        pc_lo: rec.script_lo,
        pc_hi: rec.script_hi,
    }
}

/// Boss / EN-9+ 段：**fail-closed**（未证不发明）。
pub fn boss_segment_unproven() -> Result<(), String> {
    Err("Boss/EN-9+ 段未证（t79 约束①：BOSS 子格不在本批）——不发明".to_string())
}
