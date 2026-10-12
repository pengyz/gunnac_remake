//! game-core —— 纯逻辑核心（AGENTS.md §2 红线为架构合同）。
//!
//! 红线（由 `tests/contract.rs` 机器把关）：
//! - 无 I/O：禁止 `std::fs`/`std::net`/`std::process`；
//! - 确定性：禁止 `Instant::now()`/`SystemTime`/`thread_rng`，时间走 `Clock`、随机走 `GameRng`；
//! - 禁止平台依赖：唯一允许的依赖是 `game-platform`（纯 trait 缝）。
//!
//! Stage A：行为只增不减，原版 quirks 保真（D4）；本 crate 不发明规格未覆盖的行为。

#![forbid(unsafe_code)]

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
