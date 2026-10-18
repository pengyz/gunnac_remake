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

// ===== bank11 音频驱动面（AUD-1 七入口 A 级跳板）=====

/// bank11 七入口 API 跳板（`$8000-$8015` 3B/入口 `JMP`——ROM 直读逐字节吻合；
/// 尾 `$8015`=`RTS`）。far-call 内联参数面：`JSR $E649` 后 4B r6,r7,lo,hi——
/// 8 点 `r6:=$0B`（bank11 映射）。
pub const AUDIO_TRAMPOLINES: [(u16, u16, &str); 7] = [
    (0x8000, 0x88A3, "audio_init"),   // boot 域开机初始化
    (0x8003, 0x88A7, "api2"),         // 调用面未现
    (0x8006, 0x890B, "api3"),         // 同上
    (0x8009, 0x8016, "frame_body"),   // $0E 旗+JSR $81EF→APU 写簇
    (0x800C, 0x8894, "trigger_a"),    // sfx/音乐触发口 A
    (0x800F, 0x889B, "trigger_b"),    // 触发口 B
    (0x8012, 0x8E2C, "main_tick"),    // 音频主 tick（帧同步消费）
];

/// 音频驱动状态（内部体=AUD-2 字段切分/AUD-3 指针表挂——入口语义先接）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AudioDriver {
    /// init 已执行（$8000 入口）。
    pub inited: bool,
    /// 主 tick 计数（$8012 帧同步消费）。
    pub tick_count: u32,
    /// 触发计数（A/B 口）。
    pub triggers_a: u32,
    pub triggers_b: u32,
}

/// 音频入口派发（效果面=APU 写流产出——内部体参数化（AUD-2/3 挂）；
/// `trigger` 参数=触发口负载（sfx/音乐 id——语义挂 AUD-2 字段切分））。
pub fn audio_entry(
    entry: &str,
    driver: &mut AudioDriver,
    trigger: u8,
    writes: &mut ApuWriteStream,
) -> bool {
    match entry {
        "audio_init" => {
            driver.inited = true;
            // init 效果面=APU 写簇（§1 三簇——内部体 AUD-2 挂；此处=登记性空写注）
            true
        }
        "trigger_a" => {
            driver.triggers_a += 1;
            writes.write(driver.tick_count, 0x15, trigger); // 触发口写登记（值域 AUD-2 挂）
            true
        }
        "trigger_b" => {
            driver.triggers_b += 1;
            writes.write(driver.tick_count, 0x15, trigger);
            true
        }
        "main_tick" => {
            driver.tick_count += 1;
            true
        }
        "frame_body" | "api2" | "api3" => true, // 内部体挂（AUD-2/3）
        _ => false,
    }
}

// ===== AUD-3 音频通道参数表（四件套输入齐——3a294bb）=====

/// 音频指针表组（5 组 lo/hi 指针对=数据流基址——AUD-3 全集）。
pub const AUDIO_PTR_GROUPS: [(u16, u16); 5] = [
    (0x8AD7, 0x8B00), // 组 1（lo/hi）
    (0x8B30, 0x8B60), // 组 2
    (0x8B90, 0x8BC0), // 组 3
    (0x8BF0, 0x8C20), // 组 4
    (0x8C50, 0x8C80), // 组 5
];

/// 标量表 6 域（AUD-3——通道参数标量）。
pub const AUDIO_SCALAR_DOMAINS: u8 = 6;

/// 装载 8 点（STA $A/$B——间接指针装载）。
pub const AUDIO_LOAD_POINTS: u8 = 8;

/// ($A),Y 间接消费 13 点（AUD-3——数据流消费）。
pub const AUDIO_INDIRECT_CONSUMERS: u8 = 13;

/// AUD-2 字段切分（$8AD7 复合：低 5=音调/高 2=独立参——音频通道参数）。
pub fn audio_field_split(composite: u8) -> (u8, u8) {
    (composite & 0x1F, composite >> 5) // (音调, 独立参)
}

// ===== APU 合成模型（9d661b6 五线 A 级——Audio PCM 面输入）=====

/// 音调合成式（五线①：$03E3+transpose($0A)→AND #$0F（16 音阶）+$80 八度旗）。
/// 三参数音高模型：音阶（0-15）+八度旗（bit7）+transpose。
pub fn apu_pitch_model(scale_base: u8, transpose: u8) -> u8 {
    let combined = scale_base.wrapping_add(transpose) & 0x0F; // 16 音阶
    let octave = if scale_base.wrapping_add(transpose) & 0x80 != 0 { 0x80 } else { 0 };
    combined | octave
}

/// duty 4 形（五线③：$8AD7 高 2 位 ROL→pulse $4000 bits 7-6）。
/// 返回 duty 形（0-3）——音色。
pub fn apu_duty_form(composite: u8) -> u8 {
    (composite >> 5) & 0x03 // 高 2 位=duty 4 形
}

/// pulse 影子族映射（五线④：$0309+X→$4000+X 倒序 X=4→0——控制/扫频/周期低/高）。
pub fn apu_pulse_shadow(shadow: [u8; 4]) -> [(u16, u8); 4] {
    // $4000+X（X=0..3）：控制/扫频/周期低/周期高——倒序循环注（实装序=fail-closed）
    [(0x4000, shadow[0]), (0x4001, shadow[1]), (0x4002, shadow[2]), (0x4003, shadow[3])]
}

/// triangle 三字段（五线⑤：$0311→$4008 线性计数/$0313→$400A 周期低/$0314→$400B 高）。
pub fn apu_triangle_fields(linear: u8, period_lo: u8, period_hi: u8) -> [(u16, u8); 3] {
    [(0x4008, linear), (0x400A, period_lo), (0x400B, period_hi)]
}
