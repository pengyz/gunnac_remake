//! game-platform —— 平台抽象 trait 缝（AGENTS.md §2 架构合同）。
//!
//! 职责边界：
//! - `Clock`/`GameRng` 由 **game-core 的虚拟时钟/确定性 RNG** 实现（差分可复现的根基）；
//! - `Renderer`/`Audio`/`Storage`/`Input` 由**平台侧**实现（桌面/wasm…），game-core 只依赖 trait；
//! - 本 crate **零平台代码、零外部依赖**；平台 crate（winit/sdl/tokio/rand…）禁止引入。
//!
//! Stage A 约束：Renderer/Audio 的接口形状是缝的占位，具体绘制/混音语义待 S4-S6
//! 规格与 L4/L5 差分落地后锁定；接口不得提前发明行为（AGENTS.md §3）。

#![forbid(unsafe_code)]

/// 虚拟时钟：game-core 禁止 `Instant::now()`/`SystemTime`，时间只能从这里来。
///
/// Stage A 差分要求：同一场景输入下 `now_ms()` 序列必须可复现（60Hz 帧驱动）。
pub trait Clock {
    /// 自场景起点的毫秒数（整数，确定性）。
    fn now_ms(&self) -> u64;
}

/// 确定性 RNG：game-core 禁止 `thread_rng()`；序列由种子完全决定（种子进差分记录）。
///
/// 算法本身属于游戏行为（原版 RNG 语义），待 S4 spec 锁定；本 trait 只定契约。
pub trait GameRng {
    fn next_u32(&mut self) -> u32;

    /// [0, bound) 均匀取值（bound>0）；实现必须保持确定性。
    fn next_below(&mut self, bound: u32) -> u32 {
        assert!(bound > 0, "bound 必须 > 0");
        self.next_u32() % bound
    }
}

/// 8 个标准 NES 手柄按键（无组合语义；连发/宏是平台侧输入策略）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Buttons {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
}

/// 输入源：每逻辑帧采样一次。
pub trait Input {
    fn buttons(&self) -> Buttons;
}

/// 渲染缝（**本阶段锁定**——platform-shell.md §2）：256×240 逻辑帧 RGBA Canvas 呈现
/// （game-desktop Canvas 同语义）+整数倍缩放保比例（黑边）+present。已证语义最小面
/// （Canvas 像素）——绘制细节=实现侧（shell 自有），trait 不承载未证行为。
pub trait Renderer {
    /// 256×240 RGBA Canvas（像素级呈现——L4 同语义）。
    fn canvas_mut(&mut self) -> &mut [u8];
    /// 整数倍缩放保比例呈现（黑边——shell 实现侧缩放）。
    fn present(&mut self);
}

/// 音频缝（**本阶段锁定**——platform-shell.md §2）：ApuWrite 流消费（(frame,reg,value)
/// 三元组——L-Audio 写流 hash 面同源）→混音输出；无合成器期=流透传/静音占位显式。
pub trait Audio {
    /// ApuWrite 流消费（帧序三元组——L-Audio 同源）。
    fn write_apu(&mut self, frame: u32, reg: u16, value: u8);
    /// 帧推进（60Hz——shell 侧时钟驱动）。
    fn end_frame(&mut self, frame: u32);
}

/// 存档缝（占位形状）：A 阶段预留（D2 扩展缝）。
pub trait Storage {
    fn load(&mut self, slot: u8) -> Option<Vec<u8>>;
    fn save(&mut self, slot: u8, data: &[u8]) -> bool;
}
