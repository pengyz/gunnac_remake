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
    /// 对象网格（spawn 写链落地面）。
    pub grid: ObjectGrid,
    /// RNG 状态（$EC12 LFSR）。
    pub rng: RngState,
    /// zp `$54`（调用环分组判据输入——运行期来源 S4b 挂账，字段不发明）。
    pub zp_54: u8,
    /// flow 模式名（flow.rs Mode 镜像——golden L5 口径）。
    pub mode: String,
    /// 本帧 checkpoint（一次性事件）。
    pub checkpoint: Option<String>,
    /// 场景标签（checkpoint 命名前缀）。
    pub scene: String,
    /// W-PHASE 口径（真机裁点双模式可跑化）。
    pub wave_phase: WavePhaseTiming,
    /// ArmAtRecordComplete 的完成延迟（tick；真机拟合参数）。
    pub record_complete_delay: u32,
    /// 脚本游标（脚本解释器 thr 推进——子面⑦）。
    pub script: ScriptCursor,
    /// 弹网格 SoA（批次3 六数组面——自机/敌弹共用）。
    pub bullets: crate::shot::BulletGrid,
    /// 框形流数据（地址→流字节——game-core 纯态由装载侧喂入）。
    pub shape_data: Vec<(u16, Vec<u8>)>,
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
    let prev_tick = state.wave.tick;
    state.wave.scroll_progress = state.wave.scroll_progress.saturating_add(scroll_delta);
    state.wave.tick = state.wave.scroll_progress / 16;
    // **逐 tick 中间步进**（$84 逐拍 INC——脚本 thr 精确相等不跳拍；调度游标对点）
    for t in u32::from(prev_tick)..u32::from(state.wave.tick) {
        state.sched.step_to(t, fx);
        let mut cursor = core::mem::take(&mut state.script);
        cursor.step_to(t, state, fx);
        register_wave_flags(&mut cursor);
        cursor.process_waves(state, fx);
        state.script = cursor;
    }
    // 末 tick 同样步进一拍（含）
    let t = u32::from(state.wave.tick);
    state.sched.step_to(t, fx);
    let mut cursor = core::mem::take(&mut state.script);
    cursor.step_to(t, state, fx);
    register_wave_flags(&mut cursor);
    cursor.process_waves(state, fx);
    state.script = cursor;
}

/// 帧域行为步（子面④）：调用环双序逐槽分发（`$82F8` 双模式）+ `$89DF` 滴灌 handler。
/// 已证 handler 执行（`run_handler`）；未分析 ⇒ fail-closed 空转+挂账。
pub fn behavior_frame(state: &mut StageState, fx: &mut StageEffects) {
    let group = state.zp_54 & state.rng.z2b & 0x02 != 0;
    let order: Vec<u8> = call_loop_round1()
        .into_iter()
        .chain(call_loop_slots(group))
        .collect();
    for idx in order {
        let type_ = state.grid.slots[idx as usize].type_;
        if type_ == 0 {
            continue;
        }
        let beh_hi = state.grid.slots[idx as usize].beh_hi;
        match dispatch_mode(beh_hi) {
            DispatchMode::SelfVector => {
                let target = u16::from(beh_hi) << 8 | u16::from(state.grid.slots[idx as usize].beh_lo);
                let r = run_handler(&mut state.grid.slots[idx as usize], target, &mut state.rng);
                if r == HandlerResult::Recycle {
                    state.grid.slots[idx as usize] = ObjectSlot::default();
                }
            }
            DispatchMode::TableDispatch => {
                // 表分发（decomp f06a376 接线律）：hi=$80CE+type/lo=$7FFF+type → handler；
                // type≥$30 前置 MMC3 切 bank（调用环面 type_bank_switch 同律）
                let d = crate::enemy::TYPE_DISPATCH[type_ as usize];
                let r = run_handler(&mut state.grid.slots[idx as usize], d.target, &mut state.rng);
                if r == HandlerResult::Recycle {
                    state.grid.slots[idx as usize] = ObjectSlot::default();
                }
            }
        }
    }
    drip_step(state, fx);
}

/// `$89DF` 滴灌 handler（四参消费端）：组长槽（beh=$89DF/$89E4）按 `$534` 帧间隔
/// 逐员落地（首员=装载帧同帧）；`$630` 剩余数递减（首现=数−1 口径在 land_leader）。
pub fn drip_step(state: &mut StageState, _fx: &mut StageEffects) {
    for i in 8..28usize {
        let s = state.grid.slots[i];
        if s.type_ == 0 {
            continue;
        }
        let is_leader = (s.beh_hi, s.beh_lo) == (0x89, 0xDF) || (s.beh_hi, s.beh_lo) == (0x89, 0xE4);
        if !is_leader {
            continue;
        }
        let (member_type, member_param, interval) =
            (state.grid.slots[i].f518, state.grid.slots[i].f470, state.grid.slots[i].f534);
        // 首员（load 发射：同帧落地、**不递减 $630**——GroupRelease::load 语义）
        if state.grid.slots[i].first_pending {
            if land_pair(&mut state.grid, member_type, member_param).is_some() {
                state.grid.slots[i].first_pending = false;
                state.grid.slots[i].drip_cd = interval;
            }
            continue;
        }
        // 滴灌倒数=GroupRelease::tick_frame 语义（**递减归零帧发射**）
        if state.grid.slots[i].drip_cd > 0 {
            state.grid.slots[i].drip_cd -= 1;
            if state.grid.slots[i].drip_cd == 0
                && state.grid.slots[i].f630 > 0
                && land_pair(&mut state.grid, member_type, member_param).is_some()
            {
                state.grid.slots[i].f630 -= 1;
                state.grid.slots[i].drip_cd = interval;
            }
        }
    }
}

/// 帧域战斗步（子面⑥组装）：弹推进（LUT 运动机）→ 7×10 注册 → 敌框形取框 →
/// 命中判定（4 边+属性相交）→ 伤害（弹 −2）→ HP 归 0 回收（$D616 死亡门语义）。
pub fn combat_frame(state: &mut StageState, _fx: &mut StageEffects) {
    use crate::enemy::{fetch_shape_boxes, shape_stream_addr};
    use crate::shot::{
        damage_apply, hit_resolves, lut_accumulate_axis, shot_hitbox_register, HitBoxEntry, LUT_X,
        LUT_Y,
    };
    // 弹推进（motion_code 驱动 LUT——shot 运动机）
    for y in 0..0x100usize {
        if state.bullets.state[y] == 0 {
            continue;
        }
        let mc = state.bullets.motion[y];
        state.bullets.x[y] = lut_accumulate_axis(state.bullets.x[y], mc, &LUT_X)
            .unwrap_or(state.bullets.x[y]);
        state.bullets.y[y] = lut_accumulate_axis(state.bullets.y[y], mc, &LUT_Y)
            .unwrap_or(state.bullets.y[y]);
    }
    // 弹注册（7×10 + attr $30）
    let mut shot_boxes: Vec<(usize, HitBoxEntry)> = Vec::new();
    for y in 0..0x100usize {
        if state.bullets.state[y] == 0 {
            continue;
        }
        let e = shot_hitbox_register(state.bullets.x[y], state.bullets.y[y], y as u8);
        shot_boxes.push((y, e));
    }
    // 敌框形取框+命中判定+伤害
    for si in 8..28usize {
        let slot = state.grid.slots[si];
        if slot.type_ == 0 {
            continue;
        }
        let (flip_x, flip_y) = (slot.f534 & 0x40 != 0, slot.f534 & 0x80 != 0);
        let stream_addr = match shape_stream_addr(slot.f518) {
            Some(a) => a,
            None => continue,
        };
        let stream: &[u8] = state
            .shape_data
            .iter()
            .find(|(a, _)| *a == stream_addr)
            .map(|(_, v)| v.as_slice())
            .unwrap_or(&[]);
        let rects = fetch_shape_boxes(stream, slot.x, slot.y, flip_x, flip_y);
        let enemy_attr = slot.f4c4 & 0xF0;
        for (l, r, t, b) in rects {
            let enemy = HitBoxEntry {
                left: l,
                right: r,
                top: t,
                bottom: b,
                owner: si as u8,
                attr: enemy_attr,
            };
            for (bi, sb) in &mut shot_boxes {
                if state.bullets.state[*bi] == 0 {
                    continue; // 已回收
                }
                if hit_resolves(sb, &enemy) {
                    // 弹命中：敌 HP−2（$D688 弹路）；弹回收（$87FD）；一框一伤
                    let hp = state.grid.slots[si].f630;
                    let new_hp = damage_apply(hp, true, 0);
                    state.grid.slots[si].f630 = new_hp;
                    state.bullets.release(*bi as u8);
                    *sb = crate::shot::clear_hit_entry(sb);
                    if new_hp == 0 {
                        // $D616 死亡门：槽回收（爆炸/释放链未证 fail-closed）
                        state.grid.slots[si] = ObjectSlot::default();
                    }
                    break;
                }
            }
        }
    }
}

/// 主环收尾步（scroll 确认+flow 消费 `$26` bit4 关末门）。
pub fn main_loop_step(state: &mut StageState, fx: &mut StageEffects) {
    if state.wave.flags_26 & 0x10 != 0 {
        fx.stage_clear_requested = true;
        // flow 转场：关末门 ⇒ StageClear（checkpoint 事件一次性）
        if state.mode != "StageClear" {
            state.mode = "StageClear".to_string();
            state.checkpoint = Some(format!("{}-clear", state.scene));
        }
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

// ===== 子面③：spawn 写链落地（调度项→对象网格写集，合流裁定逐段跑通）=====

use crate::enemy::{type_signatures, BehaviorField};
use crate::wave::{alloc_c2c6, alloc_c304, AllocOutcome, SlotView};

/// 对象槽（28 槽网格字段镜像：`$400` 型/四参 `$518/$630/$534/$470`/
/// 行为字段 `$41C/$438`/码流参 `$48C`）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjectSlot {
    /// `$400+槽` 型/状态（0=空）。
    pub type_: u8,
    /// `$518+槽` 组员 type/速度参（两段生命周期——运动机复用面）。
    pub f518: u8,
    /// `$630+槽` 剩余数/HP（两义——组长滴灌/中 boss 血量）。
    pub f630: u8,
    /// `$534+槽` 间隔帧/速度参。
    pub f534: u8,
    /// `$470+槽` 参数。
    pub f470: u8,
    /// `$41C+槽` 行为字段 hi。
    pub beh_hi: u8,
    /// `$438+槽` 行为字段 lo。
    pub beh_lo: u8,
    /// `$48C+槽` 码流附加参（ctrl2/12/set10）。
    pub f48c: u8,
    /// `$04E0+槽` x 坐标。
    pub x: u8,
    /// `$04FC+槽` y 坐标。
    pub y: u8,
    /// 组员滴灌倒数（帧域——`$89DF` 滴灌 handler 状态面）。
    pub drip_cd: u8,
    /// 首员待发（`GroupRelease::load` 发射语义：装载即发、不递减 `$630`）。
    pub first_pending: bool,
    /// `$5DC+槽` 三角计数（Wt-6/$84D1 共享相位计数）。
    pub f5dc: u8,
    /// `$5A4+槽` 速度四元组一员（$90AB 清态面）。
    pub f5a4: u8,
    /// `$550+槽` 速度四元组一员。
    pub f550: u8,
    /// `$56C+槽` 速度四元组一员（x 速度 hi）。
    pub f56c: u8,
    /// `$588+槽` 速度四元组一员（y 速度 lo）。
    pub f588: u8,
    /// `$5C0+槽` 倒计时簇（$90AB 清态面）。
    pub f5c0: u8,
    /// `$04C4+槽` 属性掩码（命中框注册 `& $F0`——团队过滤面）。
    pub f4c4: u8,
}

/// 对象网格（28 槽，槽 0..7 与 8..27 分区——分配域 8..=27）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectGrid {
    pub slots: [ObjectSlot; 28],
}

impl Default for ObjectGrid {
    fn default() -> Self {
        ObjectGrid {
            slots: [ObjectSlot::default(); 28],
        }
    }
}

impl ObjectGrid {
    fn slot_views(&self) -> [SlotView; 28] {
        core::array::from_fn(|i| SlotView {
            type_: self.slots[i].type_,
            flags_668: 0,
            vec_41c: self.slots[i].beh_hi,
        })
    }
}

/// 落地结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandResult {
    /// 分配到的槽。
    pub slot: u8,
    /// 行为字段是否经签名登记写入（未登记 ⇒ false=fail-closed 空行为字段）。
    pub beh_registered: bool,
}

/// pair 直投落地（`$C304` 单遍 first-fit 零副作用）：`$400:=type / $470:=param`。
pub fn land_pair(grid: &mut ObjectGrid, type_: u8, param: u8) -> Option<u8> {
    let slot = alloc_c304(&grid.slot_views())?;
    let s = &mut grid.slots[slot as usize];
    s.type_ = type_;
    s.f470 = param;
    Some(slot)
}

/// 组长落地（ctrl1/2/11/12 项，`$C2C6` 三级回退）：`$400:=type` + 四参
/// `$518/$630/$534/$470:=args` + 行为字段按**签名登记**写（`TYPE_SIGNATURES`
/// BehPtr 对；未登记 ⇒ fail-closed 空字段+标志——写点钉取挂账 W-SIGN）。
/// `code_stream`（($3E),0）仅 ctrl2/12 写 `$48C`。
pub fn land_leader(
    grid: &mut ObjectGrid,
    type_: u8,
    args: [u8; 4],
    code_stream: Option<u8>,
) -> Option<LandResult> {
    let slot = match alloc_c2c6(&grid.slot_views()) {
        AllocOutcome::Empty { slot } | AllocOutcome::Reassign { slot } | AllocOutcome::Recycle { slot } => slot,
        AllocOutcome::Full => return None,
    };
    let [f518, f630_raw, f534, f470] = args;
    // `$630` 首现=数−1（**装载后先递减一次**——B1 §3 trace 三签名口径）
    let f630 = f630_raw.saturating_sub(1);
    let sig = type_signatures(type_)
        .into_iter()
        .find_map(|s| match s.beh {
            BehaviorField::BehPtr(p) => Some(p),
            BehaviorField::DispatchSeed(_) => None,
        });
    let s = &mut grid.slots[slot as usize];
    s.type_ = type_;
    s.f518 = f518;
    s.f630 = f630;
    s.f534 = f534;
    s.f470 = f470;
    s.f48c = code_stream.unwrap_or(0);
    s.first_pending = true; // 首员=装载帧同帧（load 发射，不递减 $630）
    let beh_registered = match sig {
        Some(p) => {
            s.beh_hi = (p >> 8) as u8;
            s.beh_lo = (p & 0xFF) as u8;
            true
        }
        None => false,
    };
    Some(LandResult { slot, beh_registered })
}

// ===== 子面④：behavior_frame 行为 VM 调用环（合流裁定逐段跑通）=====

use crate::enemy::{
    call_loop_round1, call_loop_slots, chain_rewrite, dispatch_mode, handler_const_descent,
    handler_duration_tick, rng_jitter_offsets, DispatchMode,
};

/// RNG 状态（`$2C/$2D/$2B` zp——`$EC12` LFSR 字面体）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RngState {
    /// `$2C`。
    pub z2c: u8,
    /// `$2D`（INC 自增）。
    pub z2d: u8,
    /// `$2B`（EOR 源/相位位）。
    pub z2b: u8,
}

/// `$EC12` RNG 体（字面逐指令：`LDA $2C/ADC $2D/ADC #$01/EOR $2B/STA $2C/INC $2D`）。
/// 返回 (A=新 RNG 字节, carry=两 ADC 进位链出口——**状态依赖**，jitter x 侧用）。
pub fn rng_ec12(st: &mut RngState) -> (u8, bool) {
    let (s1, c1) = st.z2c.overflowing_add(st.z2d);
    let (s2, c2) = s1.overflowing_add(1);
    let carry = c1 | c2;
    let a = s2 ^ st.z2b;
    st.z2c = a;
    st.z2d = st.z2d.wrapping_add(1);
    (a, carry)
}

/// 行为 handler 执行结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerResult {
    /// 续行（字段已更新）。
    Continue,
    /// `$D10C` 终止（槽回收）。
    Recycle,
    /// 未分析 handler（fail-closed：字段不动+挂账）。
    Unproven,
}

/// 已证 handler 执行（自向量目标→字面语义；其余 fail-closed）。
/// `target`=自向量地址；`rng` 供 jitter；`zp_2b` 供相位翻转。
pub fn run_handler(
    slot: &mut ObjectSlot,
    target: u16,
    rng: &mut RngState,
) -> HandlerResult {
    match target {
        0x836E => {
            // 恒速下沉（y+1/溢出终止）→尾 $83DE 参数化（$CD9D 内核未证 ⇒ 链停在此）
            match handler_const_descent(slot.y) {
                Some(y) => {
                    slot.y = y;
                    HandlerResult::Continue
                }
                None => HandlerResult::Recycle,
            }
        }
        0x8314 => {
            // duration 倒数（$438=倒数——表模式字段复用三义面）
            match handler_duration_tick(slot.beh_lo) {
                Some(d) => {
                    slot.beh_lo = d;
                    HandlerResult::Continue
                }
                None => HandlerResult::Recycle,
            }
        }
        0x8334 | 0x837E | 0x8403 => HandlerResult::Recycle, // 终止直通
        0x8337 => {
            // RNG jitter + 续链改写 → $836E
            let (a, carry) = rng_ec12(rng);
            let (dx, dy) = rng_jitter_offsets(a, carry);
            slot.x = slot.x.wrapping_add(dx as u8);
            slot.y = slot.y.wrapping_add(dy as u8);
            let (lo, hi) = chain_rewrite(0x836E);
            slot.beh_lo = lo;
            slot.beh_hi = hi;
            HandlerResult::Continue
        }
        0x90AB => {
            // $2B 成员链首（decomp f06a376 §2.3）：改写→$9120 + 清态
            //（$5DC 三角计数归零+$5A4/$550/$56C/$5C0）
            let (lo, hi) = chain_rewrite(0x9120);
            slot.beh_lo = lo;
            slot.beh_hi = hi;
            slot.f5dc = 0;
            slot.f5a4 = 0;
            slot.f550 = 0;
            slot.f56c = 0;
            slot.f5c0 = 0;
            HandlerResult::Continue
        }
        0x9120 => {
            // $CD63 登记 + $5DC BMI 相位支 + $CD9D(A=$30,Y=4)——内核 S4b-9b 未证 ⇒ fail-closed
            HandlerResult::Unproven
        }
        _ => HandlerResult::Unproven, // 未分析 handler fail-closed
    }
}

// ===== 子面⑦：脚本解释器+波次记录机效果——关末门判据（合流自然跑）=====

use crate::wave_record::{ca_judgment, far_call_fires, op7_action, Op7Action};

/// 脚本记录（脚本解释器 `bank00:8185-82FA` 消费；thr=**本记录触发 tick**=前一记录
/// 尾 thr 字段——dropper §2 口径）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRecord {
    /// 触发 tick（16 位精确相等）。
    pub thr: u16,
    /// op 码（0=setspeed/1=evt-gen/2=sched/4=set-tbl/5=wavestream/7=jump/9=set-9）。
    pub op: u8,
    /// op5 装载的波次流址；其他 op ⇒ None。
    pub wave: Option<u16>,
    /// op7 目标；其他 ⇒ None。
    pub target: Option<u16>,
}

/// 波次流激活态（op5 `$CC/$CD` 装载 → 记录机效果面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveStream {
    /// 流址。
    pub addr: u16,
    /// 记录 C9 标志。
    pub c9: u8,
    /// 记录 CA 标志。
    pub ca: u8,
    /// 已处理（效果已落地）。
    pub processed: bool,
    /// 记录完成·槽扫描口径的落地 tick（ArmAtLoad ⇒ =装载 tick；ArmAtRecordComplete
    /// ⇒ =装载 tick+record_complete_delay——真机拟合参数）。
    pub complete_at: u32,
}

/// 脚本游标（thr 精确相等推进；op5 装载波次流；op7 按 $C8 分支）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptCursor {
    pub records: Vec<ScriptRecord>,
    pub next: usize,
    /// 已装载波次流（op5）。
    pub waves: Vec<WaveStream>,
    /// `$C8` 波次机忙标志（op5 置 1；op7 分支判据）。
    pub c8: u8,
    /// `$86/$87` 当前阈值（尾 thr 传递：处理记录后 := 该记录尾 thr——精确相等匹配）。
    pub threshold: u16,
}

impl ScriptCursor {
    /// 通用装载（`(thr, op, wave, target)` 序列——thr=记录尾字段，阈值链自动传递）。
    pub fn from_records(rows: &[(u16, u8, Option<u16>, Option<u16>)]) -> Self {
        ScriptCursor {
            records: rows
                .iter()
                .map(|&(thr, op, wave, target)| ScriptRecord { thr, op, wave, target })
                .collect(),
            next: 0,
            waves: Vec::new(),
            c8: 0,
            threshold: 0,
        }
    }

    /// STG-01 脚本装载（`wave_timeline.py script 1` golden——14 记录）。
    pub fn stg01() -> Self {
        Self::from_records(&STG01_SCRIPT)
    }

    /// 逐 tick 推进（`$81A3` **精确相等**匹配 `threshold==tick`，同 tick 连锁（尾 thr
    /// 传递后仍相等则继续处理下一记录——82EF 回 81A3 环）。返回触发的记录数。
    pub fn step_to(&mut self, tick: u32, _state: &mut StageState, fx: &mut StageEffects) -> usize {
        let mut fired = 0;
        while self.next < self.records.len() && u32::from(self.threshold) == tick {
            let rec = &self.records[self.next].clone();
            self.threshold = rec.thr; // 尾 thr → $86/$87
            match rec.op {
                5 => {
                    // op5 set-wavestream：$C8:=1 + 装流（complete_at 由调用侧口径填）
                    self.c8 = 1;
                    let addr = rec.wave.unwrap();
                    self.waves.push(WaveStream {
                        addr,
                        c9: 0,
                        ca: 0,
                        processed: false,
                        complete_at: tick,
                    });
                }
                7 => {
                    // op7 jump-if-wave-busy（$C8≠0 ⇒ 跳转/重基；=0 ⇒ 跳过 2B）
                    match op7_action(self.c8, rec.target.unwrap_or(0)) {
                        Op7Action::JumpRebase { .. } | Op7Action::Skip2B => {}
                    }
                }
                _ => {} // 其余 op 效果面（速度/表/evt-gen）消费侧另面——fail-closed
            }
            self.next += 1;
            fired += 1;
        }
        let _ = fx;
        fired
    }

    /// 波次流记录机效果落地（CA/C9 判定链）——**W-PHASE 二择边界命名**（#94 靶⑤）：
    /// 现口径=`ArmAtLoad`（装载即处理）；二择=`ArmAtRecordComplete`（记录完成·槽扫描
    /// 口径）——真机验收裁点自带命名，分歧出现即可指名道姓挂账。
    /// arm 关末门/far-call 请求。返回落地流数。
    pub fn process_waves(&mut self, state: &mut StageState, fx: &mut StageEffects) -> usize {
        let timing = state.wave_phase;
        let delay = state.record_complete_delay;
        let now = u32::from(state.wave.tick);
        let mut n = 0;
        for w in &mut self.waves {
            if w.processed {
                continue;
            }
            // W-PHASE 双口径：ArmAtLoad=装载 tick 落地；ArmAtRecordComplete=完成 tick 落地
            let due = match timing {
                WavePhaseTiming::ArmAtLoad => w.complete_at,
                WavePhaseTiming::ArmAtRecordComplete => w.complete_at.saturating_add(delay),
            };
            if now < due {
                continue;
            }
            let j = ca_judgment(w.c9, w.ca, state.wave.flags_26, 0, 0);
            if j.arm_stage_end_gate {
                state.wave.flags_26 |= 0x10;
            }
            if far_call_fires(w.c9, state.wave.flags_26) {
                fx.far_call_requests.push(w.addr as u8);
            }
            w.processed = true;
            n += 1;
        }
        n
    }
}

/// STG-01 脚本 14 记录（thr=记录尾字段）。
pub const STG01_SCRIPT: [(u16, u8, Option<u16>, Option<u16>); 14] = [
    (0x0000, 1, None, None),
    (0x0000, 0, None, None),
    (0x0000, 4, None, None),
    (0x0000, 2, None, None), // sched=$A26D
    (0x00C0, 9, None, None),
    (0x00D8, 0, None, None),
    (0x0120, 9, None, None),
    (0x0330, 0, None, None),
    (0x04C8, 5, Some(0xA3CD), None),
    (0x04D0, 4, None, None),
    (0x04DA, 5, Some(0xA3E4), None),
    (0x04E8, 0, None, None),
    (0x0000, 4, None, None),
    (0x2718, 7, None, Some(0x080F)),
];

/// W-PHASE 二择边界（真机验收裁点命名面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WavePhaseTiming {
    #[default]
    ArmAtLoad,
    /// 记录完成·槽扫描（二择候选）。
    ArmAtRecordComplete,
}

/// 现行 W-PHASE 口径声明（真机裁决后改标）。
pub const WAVE_PHASE_TIMING: WavePhaseTiming = WavePhaseTiming::ArmAtLoad;

/// 波次流 C9/CA 全登记（**17 条 op5 流**八关全谱——`wave_stream_decode` golden）。
pub const WAVE_FLAGS: [(u16, u8, u8); 17] = [
    (0xA3CD, 0xF2, 0x00), (0xA3E4, 0xFB, 0x05), // STG-01
    (0xB1CE, 0xF2, 0x00), (0xB1E5, 0xFB, 0x31), // STG-02
    (0xA5BF, 0xF2, 0x70), (0xA5DC, 0xFB, 0x31), // STG-03
    (0xAF51, 0xE2, 0x00), (0xAF68, 0xFB, 0x01), // STG-04
    (0xB7D7, 0xF2, 0x70), (0xB801, 0xFB, 0x31), // STG-05
    (0xA5D9, 0xE2, 0x00), (0xA5F0, 0xFB, 0x01), // STG-06
    (0xAF7F, 0xFB, 0xB1),                       // STG-07（单流）
    (0xB830, 0xF2, 0x70), (0xB851, 0xF2, 0x70), (0xB86D, 0xF2, 0x70), (0xB889, 0xFB, 0x81), // STG-08
];

/// 装载波次流登记（WAVE_FLAGS 灌入游标 waves——CA 判定链数据面）。
pub fn register_wave_flags(cursor: &mut ScriptCursor) {
    for w in &mut cursor.waves {
        if let Some(&(_, c9, ca)) = WAVE_FLAGS.iter().find(|(a, _, _)| *a == w.addr) {
            w.c9 = c9;
            w.ca = ca;
        }
    }
}

/// STG-02 脚本 11 记录（`wave_timeline.py script 2` golden——thr=记录尾字段）。
pub const STG02_SCRIPT: [(u16, u8, Option<u16>, Option<u16>); 11] = [
    (0x0000, 1, None, None),
    (0x0000, 0, None, None),
    (0x0000, 4, None, None),
    (0x0000, 2, None, None),
    (0x004A, 9, None, None),
    (0x0218, 9, None, None),
    (0x0218, 0, None, None),
    (0x0350, 5, Some(0xB1CE), None),
    (0x035A, 5, Some(0xB1E5), None),
    (0xFFFF, 0, None, None),
    (0x3B2B, 7, None, Some(0x1B0F)),
];

/// STG-03 脚本 14 记录（`wave_timeline.py script 3` golden——thr=记录尾字段）。
pub const STG03_SCRIPT: [(u16, u8, Option<u16>, Option<u16>); 14] = [
    (0x0000, 1, None, None),
    (0x0000, 0, None, None),
    (0x0000, 4, None, None),
    (0x0000, 2, None, None),
    (0x0050, 9, None, None),
    (0x0050, 9, None, None),
    (0x0100, 0, None, None),
    (0x0108, 5, Some(0xA5BF), None),
    (0x0340, 0, None, None),
    (0x0390, 0, None, None),
    (0x0440, 0, None, None),
    (0x0449, 5, Some(0xA5DC), None),
    (0xFFFF, 0, None, None),
    (0x3121, 7, None, Some(0x020F)),
];

/// STG-04 脚本 14 记录（`wave_timeline.py script 4` golden——thr=记录尾字段）。
pub const STG04_SCRIPT: [(u16, u8, Option<u16>, Option<u16>); 14] = [
    (0x0000, 1, None, None),
    (0x0000, 0, None, None),
    (0x0000, 4, None, None),
    (0x0000, 2, None, None),
    (0x0018, 9, None, None),
    (0x0038, 0, None, None),
    (0x01A8, 9, None, None),
    (0x01E0, 5, Some(0xAF51), None),
    (0x0210, 9, None, None),
    (0x0300, 9, None, None),
    (0x0318, 9, None, None),
    (0x0320, 5, Some(0xAF68), None),
    (0xFFFF, 0, None, None),
    (0x2718, 7, None, Some(0x080F)),
];

/// STG-05 脚本 14 记录（`wave_timeline.py script 5` golden——thr=记录尾字段）。
pub const STG05_SCRIPT: [(u16, u8, Option<u16>, Option<u16>); 14] = [
    (0x0000, 1, None, None),
    (0x0000, 0, None, None),
    (0x0000, 4, None, None),
    (0x0000, 2, None, None),
    (0x0028, 9, None, None),
    (0x0170, 9, None, None),
    (0x018A, 9, None, None),
    (0x018A, 0, None, None),
    (0x01A0, 5, Some(0xB7D7), None),
    (0x0289, 9, None, None),
    (0x028F, 5, Some(0xB801), None),
    (0x0291, 9, None, None),
    (0xFFFF, 0, None, None),
    (0x3C2C, 7, None, Some(0x1C0F)),
];

/// STG-06 脚本 15 记录（`wave_timeline.py script 6` golden——thr=记录尾字段）。
pub const STG06_SCRIPT: [(u16, u8, Option<u16>, Option<u16>); 15] = [
    (0x0000, 1, None, None),
    (0x0000, 0, None, None),
    (0x0000, 4, None, None),
    (0x0000, 2, None, None),
    (0x0020, 9, None, None),
    (0x0030, 0, None, None),
    (0x0080, 0, None, None),
    (0x0110, 0, None, None),
    (0x0120, 9, None, None),
    (0x02D8, 0, None, None),
    (0x02D8, 9, None, None),
    (0x0412, 5, Some(0xA5D9), None),
    (0x0420, 5, Some(0xA5F0), None),
    (0xFFFF, 0, None, None),
    (0x2717, 7, None, Some(0x070F)),
];

/// STG-07 脚本 11 记录（`wave_timeline.py script 7` golden——thr=记录尾字段）。
pub const STG07_SCRIPT: [(u16, u8, Option<u16>, Option<u16>); 11] = [
    (0x0000, 1, None, None),
    (0x0000, 0, None, None),
    (0x0000, 4, None, None),
    (0x0000, 2, None, None),
    (0x0050, 9, None, None),
    (0x0060, 0, None, None),
    (0x0100, 9, None, None),
    (0x028C, 0, None, None),
    (0x0295, 5, Some(0xAF7F), None),
    (0xFFFF, 0, None, None),
    (0x3027, 7, None, Some(0x160F)),
];

/// STG-08 脚本 9 记录（`wave_timeline.py script 8` golden——thr=记录尾字段）。
pub const STG08_SCRIPT: [(u16, u8, Option<u16>, Option<u16>); 9] = [
    (0x0000, 1, None, None),
    (0x0000, 0, None, None),
    (0x0000, 4, None, None),
    (0x00F0, 2, None, None),
    (0x01F0, 5, Some(0xB830), None),
    (0x0270, 5, Some(0xB851), None),
    (0x0304, 5, Some(0xB86D), None),
    (0xFFFF, 5, Some(0xB889), None),
    (0x2919, 7, None, Some(0x090F)),
];

// ===== L5 真机验收前置：帧探针桥（引擎侧输出面）=====

/// 帧探针（L5 trace 对拍状态向量——named 模式对齐 oracle golden JSONL 口径）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageProbe {
    /// 帧号。
    pub f: u32,
    /// `$84` tick（滚动进度域）。
    pub tick: u16,
    /// 滚动进度（px）。
    pub scroll: u16,
    /// `$26` 标志（bit4=关末门）。
    pub flags_26: u8,
    /// 已装波次流数。
    pub waves: u8,
    /// 已发脚本记录数。
    pub script_recs: u8,
    /// 已发调度记录数。
    pub sched_recs: u8,
    /// 对象槽型码摘要（28 槽 $0400 镜像——逐槽向量）。
    pub slot_types: [u8; 28],
    /// 存活弹数（$040E state≠0 计数）。
    pub live_shots: u16,
    /// flow 模式（flow.rs Mode 命名——golden L5 口径）。
    pub mode: String,
    /// 本帧 checkpoint（转场帧一次性事件；无 ⇒ None）。
    pub checkpoint: Option<String>,
}

/// 帧探针抽取（纯函数——L5 逐帧对拍面）。
pub fn stage_probe(state: &StageState) -> StageProbe {
    let live_shots = state.bullets.state.iter().filter(|&&s| s != 0).count() as u16;
    StageProbe {
        f: state.frame,
        tick: state.wave.tick,
        scroll: state.wave.scroll_progress,
        flags_26: state.wave.flags_26,
        waves: state.script.waves.len() as u8,
        script_recs: state.script.next as u8,
        sched_recs: state.sched.next as u8,
        slot_types: core::array::from_fn(|i| state.grid.slots[i].type_),
        live_shots,
        mode: state.mode.clone(),
        checkpoint: state.checkpoint.clone(),
    }
}

/// 运动更新（bank14:$CCD3/$CD08 亚像素对，099c510 ①定谳）：
/// y：`$048C+=vy_lo($588)` → `$04FC+=vy_hi($5A4)+carry`；x 同构（`$0470/$04E0`
/// ← `$550/$56C`）；亚像素 mod256 回绕（trace 实证）。出屏（坐标溢出或 y≥`$F0`）
/// ⇒ `Offscreen`——`$534` bit4 门二择（SEC-only/`$D10C` 回收）**极性待证 W-GATE**，
/// 回收决定留给调用方。`$5C0` 亚像素溢出→`INC $5DC`（相位计数增量源——连接件
/// 接线在 `MotionOutcome` 进位位）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionOutcome {
    /// 续行（含亚像素进位位——$5C0/$5DC 连接件接线面）。
    Continue { subpixel_carry_x: bool, subpixel_carry_y: bool },
    /// 出屏（坐标溢出或 y≥$F0）——$534 bit4 门二择（W-GATE 极性待证）。
    Offscreen,
}

/// 运动步（$0470/$048C 亚像素、$04E0/$04FC 坐标、$550/$56C/$588/$5A4 速度对）。
pub fn motion_update(slot: &mut ObjectSlot) -> MotionOutcome {
    // x：sub += vx_lo → coord += vx_hi + carry
    let (xsub, cx) = slot.f470.overflowing_add(slot.f550);
    slot.f470 = xsub;
    let (x1, cxa) = slot.x.overflowing_add(slot.f56c);
    let (x2, cxb) = x1.overflowing_add(u8::from(cx));
    slot.x = x2;
    // y：sub += vy_lo → coord += vy_hi + carry
    let (ysub, cy) = slot.f48c.overflowing_add(slot.f588);
    slot.f48c = ysub;
    let (y1, cya) = slot.y.overflowing_add(slot.f5a4);
    let (y2, cyb) = y1.overflowing_add(u8::from(cy));
    slot.y = y2;
    if cxa || cxb || cya || cyb || slot.y >= 0xF0 {
        return MotionOutcome::Offscreen;
    }
    MotionOutcome::Continue {
        subpixel_carry_x: cx,
        subpixel_carry_y: cy,
    }
}

/// `$5C0` 亚像素溢出→`INC $5DC`（相位计数增量源，099c510 ②连接件）。
pub fn phase_counter_bump(slot: &mut ObjectSlot) {
    slot.f5dc = slot.f5dc.wrapping_add(1);
}

/// flow 时序面（缺口②）：s5 场景 mode/checkpoint 转场表——**trace A 级帧锚登记**
///（R-ui-6/7 实机采样帧：Black f180/Title f360/Menu f540+/Shop f1080/Transition
/// f1200/StageStart f1260/Combat f1500+）；转场逻辑=flow.rs 状态机+场景输入，
/// 本表仅登记实测时点（时序常量注册同迁移帧口径）。boot/menu/shop 段行为语义
/// 未移植部分=显式缺口（不发明响应时延）。
pub const FLOW_TIMELINE_S5: &[(u32, &str, &str)] = &[
    (180, "Black", "boot-black"),
    (360, "Title", "title-in"),
    (540, "Menu", "menu-in"),
    (1080, "Shop", "stg01-shop"),
    (1200, "Transition", "shop-exit"),
    (1260, "StageStart", "stg01-start"),
    (1500, "Combat", "stg01-combat"),
    (2401, "StageClear", "stg01-clear"),
    (3116, "Shop", "stg02-shop"),
    (3400, "Transition", "shop-exit-2"),
    (3450, "StageStart", "stg02-start"),
    (3900, "Combat", "stg02-combat"),
];

/// flow 转场施加（时序表驱动——调用侧按帧号触发；返回本帧事件）。
pub fn flow_timeline_step(state: &mut StageState, f: u32, timeline: &[(u32, &str, &str)]) -> bool {
    for &(tf, mode, cp) in timeline {
        if tf == f {
            state.mode = mode.to_string();
            state.checkpoint = if cp.is_empty() { None } else { Some(cp.to_string()) };
            return true;
        }
    }
    false
}

/// respawn-window 场景时序（trace 实帧锚——R-respawn-1 采样口径）。
pub const FLOW_TIMELINE_RESPAWN: &[(u32, &str, &str)] = &[
    (1260, "StageStart", ""),
    (1350, "Combat", ""),
    (1400, "DeathRespawn", "respawn-window"),
];
