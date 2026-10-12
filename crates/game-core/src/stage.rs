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
