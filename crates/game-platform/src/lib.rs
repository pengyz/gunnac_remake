//! game-platform —— 平台抽象 trait 缝（AGENTS.md §2 架构合同）。
//!
//! 职责边界：
//! - `Clock`/`GameRng` 由 **game-core 的虚拟时钟/确定性 RNG** 实现（差分可复现的根基）；
//! - `Renderer`/`Audio`/`Storage`/`Input` 由**平台侧**实现（桌面/wasm…），game-core 只依赖 trait；
//! - 本 crate **零平台代码、零外部依赖**；平台 crate（winit/sdl/tokio/rand…）禁止引入。
//!
//! Stage A 约束：Renderer/Audio 的接口形状是缝的占位，具体绘制/混音语义待 S4-S6
//! 规格与 L4/L5 差分落地后锁定；接口不得提前发明行为（AGENTS.md §3）。
//!
//! v1 补全（2026-10-18 task-5）：`AssetSource`（资产字节读取）、`Lifecycle`/`AppEvent`
//! （Android 生命周期必需）、`Renderer::size`/`resize`（输出面尺寸查询）。新增项**逐条标注**
//! 「已证 / 占位」：占位项只定契约、不承载游戏行为；既有 Clock/GameRng/Input/Audio/Storage
//! 形状（方法签名）不动。

#![forbid(unsafe_code)]

/// 逻辑画布宽（256×240 RGBA——platform-shell.md §2 已锁定的呈现面）。
pub const CANVAS_W: usize = 256;

/// 逻辑画布高。
pub const CANVAS_H: usize = 240;

/// 逻辑画布字节数（RGBA=4B/像素）。
pub const CANVAS_LEN: usize = CANVAS_W * CANVAS_H * 4;

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

    /// 输出面像素尺寸（窗口/后缓冲），平台实现侧覆盖。
    ///
    /// **占位默认**：平台未独立报告输出面时回退逻辑画布尺寸 `(256, 240)`（1×）；
    /// 默认值不声称任何缩放/黑边语义（那是 shell 实现侧）。
    fn size(&self) -> (u32, u32) {
        (CANVAS_W as u32, CANVAS_H as u32)
    }

    /// 输出面尺寸变化通知（平台侧 allocate/resize 钩子）。
    ///
    /// **占位默认**：无平台侧动作（CPU 帧缓冲实现可不响应；GPU/交换链实现覆盖）。
    fn resize(&mut self, _width: u32, _height: u32) {}
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

/// 资产读取缝（**占位契约**——只定字节读取面，不承载解析/格式/校验语义）。
///
/// - PC（shell-pc）实现 = `std::fs`（路径由平台侧决定）；
/// - Android 实现 = APK `assets/`（`AAssetManager` 读整块字节）。
///
/// 缺失 ⇒ `None`：fail-closed，**不得**返回默认/合成内容（未证不发明）。
/// 解析/校验（NES 头、CHR 解码、数据表）属 `game-data` 域，不在本 trait。
pub trait AssetSource {
    /// 读取整个资产字节块；路径语义由平台侧映射（PC=文件路径；Android=APK assets 相对路径）。
    fn read_all(&mut self, path: &str) -> Option<Vec<u8>>;
}

/// 应用生命周期事件（**占位最小枚举**——Android 生命周期必需；桌面实现可只发 `Quit`）。
///
/// 只表达宿主级门控（暂停逻辑帧/退出），不承载游戏行为；事件语义=平台侧映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppEvent {
    /// 回到前台/恢复（Android `onResume`；桌面=窗口获得焦点/启动）。
    Resume,
    /// 进入后台/挂起（Android `onPause`；桌面最小化可不发）。
    Suspend,
    /// 退出请求（Android destroy/back；桌面=关窗）。
    Quit,
}

/// 生命周期事件源（平台事件队列 → 宿主；**占位契约**）。
///
/// `poll_event` 非阻塞：队列空 ⇒ `None`。平台实现负责把 OS 生命周期映射为 `AppEvent`。
pub trait Lifecycle {
    /// 取下一个待处理事件（无 ⇒ `None`）。
    fn poll_event(&mut self) -> Option<AppEvent>;
}
