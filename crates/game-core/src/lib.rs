//! game-core —— 纯逻辑核心（AGENTS.md §2 红线为架构合同）。
//!
//! 红线（由 `tests/contract.rs` 机器把关）：
//! - 无 I/O：禁止 `std::fs`/`std::net`/`std::process`；
//! - 确定性：禁止 `Instant::now()`/`SystemTime`/`thread_rng`，时间走 `Clock`、随机走 `GameRng`；
//! - 禁止平台依赖：唯一允许的依赖是 `game-platform`（纯 trait 缝）。
//!
//! Stage A：行为只增不减，原版 quirks 保真（D4）；本 crate 不发明规格未覆盖的行为。

#![forbid(unsafe_code)]

pub mod engine;
pub mod enemy;
pub mod flow;
pub mod l2ports;
pub mod player;
pub mod shot;
pub mod stage;
pub mod terrain;
pub mod ui;
pub mod wave;
pub mod wave_record;
pub mod weapons;

pub use game_platform::{Buttons, Clock, GameRng, Input};

/// 目标帧率（NES = 60Hz）：一切 tick 逻辑以此锁帧。
pub const TARGET_FPS: u64 = 60;

/// 帧虚拟时钟：`tick()` 推进一帧，`now_ms()` 由帧数换算（整数运算，确定性）。
///
/// 换算：frame * 1000 / 60 = frame * 50 / 3（毫秒取整；误差随帧累积≤1ms，
/// 差分对比以帧号为准，不以毫秒为准）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FrameClock {
    frame: u64,
}

impl FrameClock {
    pub const fn new() -> Self {
        Self { frame: 0 }
    }

    /// 推进一帧（唯一时间推进入口）。
    pub fn tick(&mut self) {
        self.frame += 1;
    }

    pub const fn frame(&self) -> u64 {
        self.frame
    }
}

impl Clock for FrameClock {
    fn now_ms(&self) -> u64 {
        self.frame * 1000 / TARGET_FPS
    }
}

/// 确定性 RNG 骨架（占位）：算法属于游戏行为（原版 RNG 语义），待 S4 spec 锁定后替换。
///
/// 本实现只保证 trait 契约与“同种子 ⇒ 同序列”，**不得**据此写游戏数值逻辑。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LcgRng {
    state: u64,
}

impl LcgRng {
    pub const fn seed(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl GameRng for LcgRng {
    fn next_u32(&mut self) -> u32 {
        // 占位 LCG（数值无游戏语义，勿引用为规格）；S4 替换为考证后的原版 RNG。
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }
}

// ===== L-Audio APU 寄存器写流面（audio-golden.md——Stage A 音频战役）=====

/// APU 寄存器写事件（`event := (frame, reg, value)`——`reg ∈ $00-$17`（$4000+reg），
/// 写序保真；DMC 一体（$4010-$4015 入流）；样本内容不入 hash（Stage B 后补））。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApuWrite {
    /// 游戏逻辑帧（60Hz tick——非采样率，无需速率归一化）。
    pub frame: u32,
    /// 寄存器偏移（$00-$17）。
    pub reg: u8,
    /// 写值。
    pub value: u8,
}

/// APU 写流（有序事件序列——写序保真）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApuWriteStream {
    /// 事件序列。
    pub events: Vec<ApuWrite>,
}

impl ApuWriteStream {
    /// 追加写事件。
    pub fn write(&mut self, frame: u32, reg: u8, value: u8) {
        self.events.push(ApuWrite { frame, reg: reg & 0x1F, value });
    }

    /// 60f 帧窗分段（段 k=frame ∈ [60k, 60k+60) 子序列——有序含写序）。
    pub fn segments(&self) -> Vec<Vec<ApuWrite>> {
        if self.events.is_empty() {
            return Vec::new();
        }
        let max_frame = self.events.iter().map(|e| e.frame).max().unwrap_or(0);
        let n = (max_frame / 60) + 1;
        let mut segs = vec![Vec::new(); n as usize];
        for e in &self.events {
            segs[(e.frame / 60) as usize].push(*e);
        }
        segs
    }
}

/// varint 编码（LEB128——**格式冻结=open item ①**（audio-golden.md）；本实现=候选
/// 冻结面，golden 兼容性以最终冻结为准）。
pub fn encode_varint(mut v: u32, out: &mut Vec<u8>) {
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            break;
        }
        out.push(byte | 0x80);
    }
}

/// 段规范编码（`frame_delta(varint) | reg | value` 逐事件——帧内相对段首）。
/// 空段=空字节串（hash(空串)显式记录非缺失）。
pub fn canonical_encode_segment(seg: &[ApuWrite]) -> Vec<u8> {
    let mut out = Vec::new();
    let base = seg.first().map(|e| e.frame).unwrap_or(0);
    for e in seg {
        encode_varint(e.frame.saturating_sub(base), &mut out);
        out.push(e.reg);
        out.push(e.value);
    }
    out
}
