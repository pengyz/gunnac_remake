//! game-core::engine —— 集成主干骨架：纯函数编排器 `stage_tick`。
//!
//! 边界总纲（lead 合流裁定）：确定性、无 I/O、虚拟时钟；**帧序沿 A2 已证序：
//! 行为→碰撞→主环**（game-over-continue.md 帧序锚）。
//! 四模块接口：tick 域 `wave_tick`（滚动进度域 `$84=+Nt−1`，Wt-2）/帧域
//! `behavior_frame`+`combat_frame`/flow 侧消费 tick 事件（`$26` bit4 等）。
//! **两时序域红线**：tick 原子（item 直投）vs 帧滴灌（组员 `$534`）——经
//! `wave::spawn_timing` 分流，混域必红。遇未证转移 fail-closed+挂账。

use crate::wave::{spawn_timing, SchedItem, SpawnTiming};

/// 帧序相位标签（A2 帧序锚：行为→碰撞→主环）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseTag {
    /// 行为 VM/运动机（帧域）。
    Behavior,
    /// 碰撞/命中/伤害（弹网格+命中框）。
    Collision,
    /// 主环收尾（scroll/UI/flow 步进）。
    MainLoop,
}

/// 帧序常量（A2 锚——对拍面锁此序）。
pub const FRAME_ORDER: [PhaseTag; 3] = [PhaseTag::Behavior, PhaseTag::Collision, PhaseTag::MainLoop];

/// tick 域状态（脚本解释器+波次调度器+记录机+item 环的共享时基）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WaveTickState {
    /// `$84` 滚动进度 tick（Wt-2：tick=16px 滚动进度单位；滚动停 ⇒ tick 冻结）。
    pub tick: u16,
    /// 累计滚动进度（px）。
    pub scroll_progress: u16,
    /// `$26` 标志（bit4=关末门臂——CA bit0 效果；flow 侧消费）。
    pub flags_26: u8,
}

/// 编排器状态（确定性虚拟时钟；子状态随子面扩充）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageState {
    /// 主环虚拟帧计数。
    pub frame: u32,
    /// tick 域。
    pub wave: WaveTickState,
    /// 调度流游标（STG-01 数据装载面——`SchedCursor::load` 装入）。
    pub sched: SchedCursor,
}

/// 帧输入（无 I/O 数据面）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TickInputs {
    /// 本帧滚动增量（px；0=停滚）。
    pub scroll_delta: u16,
}

/// 帧效果集（game-platform 执行侧消费；纯数据）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageEffects {
    /// tick 原子直投事件（同帧落地）。
    pub spawn_events: Vec<SchedItem>,
    /// 帧滴灌事件（组员 `$534` 节律展开——行为 VM 滴灌 handler 消费）。
    pub drip_events: Vec<SchedItem>,
    /// far-call 请求（`far_call_fires` 双门已过——bank11:$8012 速度重置流）。
    pub far_call_requests: Vec<u8>,
    /// 关末门请求（flow 侧：`$26` bit4 已臂 ⇒ 通关推进）。
    pub stage_clear_requested: bool,
    /// 帧序观测面（A2 锚对拍）。
    pub phase_trace: Vec<PhaseTag>,
}

/// 编排器主入口：一帧推进（帧序=行为→碰撞→主环；tick 域先于帧域）。
pub fn stage_tick(state: &mut StageState, inputs: &TickInputs) -> StageEffects {
    let mut fx = StageEffects::default();
    wave_tick(state, inputs.scroll_delta, &mut fx);
    behavior_frame(state, &mut fx);
    fx.phase_trace.push(PhaseTag::Behavior);
    combat_frame(state, &mut fx);
    fx.phase_trace.push(PhaseTag::Collision);
    main_loop_step(state, &mut fx);
    fx.phase_trace.push(PhaseTag::MainLoop);
    state.frame += 1;
    fx
}

/// tick 域推进（Wt-2：tick=16px 滚动进度——**滚动停 ⇒ tick 冻结**，
/// 关1 实测冻结 824/stg02 冻结 $006C=108 双证）。
pub fn wave_tick(state: &mut StageState, scroll_delta: u16, fx: &mut StageEffects) {
    state.wave.scroll_progress = state.wave.scroll_progress.saturating_add(scroll_delta);
    state.wave.tick = state.wave.scroll_progress / 16;
    // 调度游标按 tick 对点推进（到点记录项分流——两时序域红线）
    state.sched.step_to(u32::from(state.wave.tick), fx);
}

/// 帧域行为步（行为 VM 分发调用环+运动机+滴灌 handler——子面接续；未证 fail-closed）。
pub fn behavior_frame(_state: &mut StageState, _fx: &mut StageEffects) {}

/// 帧域战斗步（弹网格 SoA+命中框 7×10+伤害公式——批次3 面接入点）。
pub fn combat_frame(_state: &mut StageState, _fx: &mut StageEffects) {}

/// 主环收尾步（scroll 确认+flow 消费 `$26` bit4 关末门）。
pub fn main_loop_step(state: &StageState, fx: &mut StageEffects) {
    if state.wave.flags_26 & 0x10 != 0 {
        fx.stage_clear_requested = true;
    }
}

/// 两时序域分流（红线执行点）：tick 原子直投 ⇒ 同帧 `spawn_events`；
/// 组员帧滴灌 ⇒ `drip_events`（`$534` 帧间隔展开）；无投放项忽略。
pub fn route_spawn_item(item: &SchedItem, fx: &mut StageEffects) -> Option<SpawnTiming> {
    match spawn_timing(item)? {
        t @ SpawnTiming::TickAtomic => {
            fx.spawn_events.push(item.clone());
            Some(t)
        }
        t @ SpawnTiming::FrameDrip => {
            fx.drip_events.push(item.clone());
            Some(t)
        }
    }
}

// ===== STG-01 数据装载子面（合流裁定②：tick 域数据面接入）=====

use crate::wave::SchedStream;

/// 调度流游标（`sub_86AE` 记录推进器镜像）：按解码 tick 时点推进记录，
/// 项经 `route_spawn_item` 分流（两时序域红线）。首记录 tick=delay+1 quirk
/// 已在 `decode_sched` 内兑现——游标只按绝对 tick 对点。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedCursor {
    /// 已解码记录（`tick` 为绝对处理时点）。
    pub records: Vec<crate::wave::SchedRecord>,
    /// 下一待发记录下标。
    pub next: usize,
}

impl SchedCursor {
    /// 从解码流装载（`decode_sched` 产物透传）。
    pub fn load(stream: SchedStream) -> Self {
        SchedCursor {
            records: stream.records,
            next: 0,
        }
    }

    /// 推进到 `tick`（含）：到点记录的项逐一分流。返回本步发射的记录数。
    pub fn step_to(&mut self, tick: u32, fx: &mut StageEffects) -> usize {
        let mut fired = 0;
        while self.next < self.records.len() && self.records[self.next].tick <= tick {
            for item in &self.records[self.next].items {
                route_spawn_item(item, fx);
            }
            self.next += 1;
            fired += 1;
        }
        fired
    }
}

/// 关首快进环（`bank00:806A-80D9`，Wt-1：17 tick 压入 f1193——每迭代=1 全链 tick）：
/// 开场即推进 `ticks` 个 tick（STG-01=17）。**禁止跨关换算常数**（Wt-1 红线）。
pub fn fast_forward_ring(cursor: &mut SchedCursor, ticks: u16, fx: &mut StageEffects) {
    for t in 1..=u32::from(ticks) {
        cursor.step_to(t, fx);
    }
}
