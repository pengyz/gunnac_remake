//! 平台无关游戏宿主（60Hz 固定步驱动 + 宿主状态机 + 菜单派发 + STG-01 装载 +
//! tick 序列 + 渲染/音频/资产/存档/生命周期接线）。
//!
//! 分层合同（platform-shell.md §1）：`game-core`（纯逻辑）← 调用/依赖 ← 本层 ← 注入 ← 平台实现
//! （`shell-pc`/`shell-android` 实现 `game-platform` trait 后注入）。本 crate **零平台依赖**
//! （机器扫描见 `tests/contract.rs`）。
//!
//! 纪律（未证不发明）：
//! - 状态迁移只走 `game_core::flow` 的**已证边集**；未证边由 core `Err` + 宿主关号守卫 fail-closed；
//! - 菜单派发=输入驱动（`InputLatch` +1f/12f、`MenuInputGate` 两层门、`ShopState::on_b`），
//!   帧锚/响应时延逐条标级（见 `docs/findings/plan-playable-loop.md` §4）；
//! - STG-01 装载/滚动源/spawn 落地=现有 game-core 公开 API（`decode_sched`/`stage::scroll_delta`/
//!   `land_pair`/`land_leader`）；滚动源与 `.scroll.jsonl` 逐帧 1:1（A 级，`tests/stg01_replay.rs`）；
//! - 引擎 tick 活跃段：`Combat`=已证 gameplay 段；`Transition/StageStart`=B 级接线依据
//!   （载入后滚动/出敌在 stage-start 窗内仍在跑：`stage-appear` f1278 ∈ [f1260,f1500)）；
//!   `DeathRespawn/StageClear`=接线约定（细粒度未隔离，G-S5-5，见 platform-shell §8）；
//! - spawn 落地发生在 `stage_tick` 返回后同逻辑帧内；组长首员因 engine 无 wave_tick→behavior 间
//!   hook 而晚 1 帧滴灌（已知偏差，登记 platform-shell §8）。

use std::collections::VecDeque;

use game_core::engine::{
    self, InputLatch, MenuInputGate, ScriptCursor, StageEffects, StageState, TickInputs,
    RUN_STAGE_INTRO_FRAMES, TRANSITION_TAIL_FRAMES,
};
use game_core::flow::{FlowState, Mode, ModeEvent};
use game_core::player::{self, PlayerState};
use game_core::shot::{BulletGrid, PLAYER_ONLY_SLOTS, SHARED_SLOTS};
use game_core::ui::{render_digits, ShopState};
use game_core::wave::{decode_sched, SchedItem};
use game_core::FrameClock;
use game_platform::{
    AppEvent, AssetSource, Audio, Buttons, Clock, Input, Lifecycle, Renderer, Storage,
};

use crate::audio::AudioPipe;
use crate::fire::{self, EnemyHitEvent};
use crate::fixed_step::FixedStep;
use crate::input::{held_bitmap, menu_pressed_bitmap, pressed_bitmap, resolve_directions};
use crate::render::{compose_frame, EnemySlotView, FrameView, HudView};
use crate::shape;
use crate::snapshot::{decode_snapshot, encode_snapshot, mode_tag, HostSnapshot};

/// 玩家初始 X（`$04E0`）：已证重生复位锚点 (128,160)（`respawn-window.md` R-respawn-3，A 级）。
///
/// 开机/关首初值是否与重生复位同值未单独取证 ⇒ v1 **采用已证锚点值**（不发明新数值）。
pub const INITIAL_PLAYER_X: u8 = 128;
/// 玩家初始 Y（`$04FC`）：同 [`INITIAL_PLAYER_X`]。
pub const INITIAL_PLAYER_Y: u8 = 160;
/// 炸弹弹药初值（`$018F:=4`，bank00:9C5D，A 级）。
pub const INITIAL_BOMB_AMMO: u8 = 4;
/// `Combat` 段滚动增量（px/帧）——保留常量（v1 兼容）；task-9 起滚动源改由
/// [`Host`] 的 STG-01 stage clock + `stage::scroll_delta` 逐帧导出（见 `Host::stage_scroll_delta`）。
pub const SCROLL_DELTA_PX_PER_FRAME: u16 = 16;

/// 开机黑场 capture 帧锚（R-ui-7：boot-black f180，A 级 trace）。
pub const TITLE_BLACK_FRAME: u64 = 180;
/// 标题 capture 帧锚（R-ui-7：title f360，A 级 trace）。
pub const TITLE_FRAME: u64 = 360;
/// 标题段输入锁：九班 `(450,550]`（`flow-response-latency.md` §3，A 级引证）⇒
/// 首个解锁帧=551（不发明相位表；只用已证窗口边界）。
pub const MENU_INPUT_UNLOCK_FRAME: u64 = 551;
/// 出店转场响应时延：press@1160 → f1193 场景判定位（`flow-response-latency.md` §2，A 级 +33f）。
pub const SHOP_EXIT_RESPONSE_FRAMES: u64 = 33;
/// 转场尾常数（engine `TRANSITION_TAIL_FRAMES`，A 级常数；帧内分布未证）。
pub const TRANSITION_TAIL_FRAMES_HOST: u32 = TRANSITION_TAIL_FRAMES as u32;
/// 关始 intro 240f（R-ui-6 锚 f1260→f1500；engine `RUN_STAGE_INTRO_FRAMES`，A 级）。
pub const STAGE_INTRO_FRAMES_HOST: u32 = RUN_STAGE_INTRO_FRAMES;
/// STG-01 关卡数据装载帧锚（`gameplay-stage1.scroll.jsonl` 起点 f1193；A 级）。
pub const STG01_LOAD_FRAME: u32 = 1193;
/// STG-01 调度流头址（`bank8:$A26D`；`wave_timeline.py sched 1` + engine golden，A 级）。
pub const STG01_SCHED_STREAM_ADDR: u16 = 0xA26D;
/// STG-01 调度流记录数（A 级 golden：43 记录）。
pub const STG01_SCHED_RECORDS: usize = 43;
/// STG-01 调度流解码上限（43 记录 + 环检测余量）。
pub const STG01_SCHED_MAX_RECORDS: usize = 64;
/// STG-01 数据所在 PRG bank（MMC3 R7 数据 bank8 → CPU `$A000-$BFFF` 窗）。
pub const STG01_PRG_BANK: usize = 8;
/// 自机对象型值（`$0400` 槽0）：A 级 trace 快照 `gameplay-stage1.enemy.jsonl` f1264 槽0 `state=1`。
pub const PLAYER_OBJECT_TYPE: u8 = 0x01;

/// FNV-1a 64 偏移基。
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64 素数。
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *hash ^= u64::from(b);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

fn hash_u64(hash: &mut u64, v: u64) {
    hash_bytes(hash, &v.to_le_bytes());
}

/// 上一逻辑帧 `StageEffects` 摘要（确定性签名 + 宿主观测用；不含未证语义）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StageFxSummary {
    pub spawn_events: u32,
    pub drip_events: u32,
    pub far_call_requests: u32,
    pub phase_trace: u32,
    pub stage_clear_requested: bool,
}

impl StageFxSummary {
    fn from_effects(fx: &StageEffects) -> Self {
        Self {
            spawn_events: fx.spawn_events.len() as u32,
            drip_events: fx.drip_events.len() as u32,
            far_call_requests: fx.far_call_requests.len() as u32,
            phase_trace: fx.phase_trace.len() as u32,
            stage_clear_requested: fx.stage_clear_requested,
        }
    }
}

/// 敌槽激活事件（`$0400` 槽 0→非0 判定式；**型值不命名**）。帧号=宿主逻辑帧。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnEvent {
    pub frame: u64,
    pub slot: u8,
    pub type_code: u8,
}

/// 一逻辑帧的结果（平台观测面；不承载游戏语义）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameOutcome {
    /// 本逻辑帧号（虚拟时钟 `FrameClock`，tick 前取值）。
    pub frame: u64,
    /// 本帧结束后的 flow 模式。
    pub mode: Mode,
    /// 关号（`$180`，1..=8）。
    pub stage: u8,
    /// 消解后的按钮（对向同持已置 0）。
    pub buttons: Buttons,
    /// `$16` 边沿位图（v1 只产出不消费；方向+A/B 子集）。
    pub pressed: u8,
    /// 本帧是否推进了逻辑（suspend/quit 帧为 false）。
    pub advanced: bool,
    /// 本帧是否跑了 `engine::stage_tick`。
    pub gameplay_tick: bool,
    /// engine 是否上报关末门（`$26 bit4` 已臂）。
    pub stage_clear_requested: bool,
    /// 本帧宿主自动投递的 flow 事件（已证边；无 ⇒ None）。
    pub flow_event: Option<ModeEvent>,
    /// 退出请求已置位。
    pub quit: bool,
    /// 挂起中。
    pub suspended: bool,
}

/// 平台无关游戏宿主。
#[derive(Debug)]
pub struct Host {
    flow: FlowState,
    player: PlayerState,
    stage_state: StageState,
    stage_index: u8,
    all_clear: bool,
    clock: FrameClock,
    prev_buttons: Buttons,
    audio: AudioPipe,
    rom: Option<Vec<u8>>,
    suspended: bool,
    quit: bool,
    pacer: FixedStep,
    last_fx: StageFxSummary,
    last_canvas_sig: u64,
    // ===== task-9：菜单派发 / 模式时序 / STG-01 装载 =====
    menu_latch: InputLatch,
    menu_gate: MenuInputGate,
    shop_state: ShopState,
    pending_events: VecDeque<(u64, ModeEvent)>,
    stage_loaded: bool,
    stage_load_frame: u64,
    stage_sched_records: usize,
    spawn_log: Vec<SpawnEvent>,
    prev_slot_types: [u8; 28],
    last_mode: Mode,
    mode_frames: u32,
    auto_flow: bool,
    // ===== task-37：玩家弹丸生成/运动/命中接线 =====
    /// 玩家弹网格（**game-shell 自有**；不写 `StageState.bullets` ⇒ engine combat_frame 惰性，
    /// 避免双步进/双份命中；敌弹不参与）。
    player_bullets: BulletGrid,
    shot_timer: u8,
    hits_this_frame: [Option<EnemyHitEvent>; 4],
    hit_overflow: u16,
    enemy_hit_log: Vec<EnemyHitEvent>,
    shape_data_shapes: usize,
}

impl Host {
    /// 新建宿主（开机态：`Mode::Boot`、关号 1；玩家初值见 [`INITIAL_PLAYER_X`]）。
    ///
    /// **固定步时间基准**：`FixedStep` 以本调用时刻为 0；平台侧调用 [`Host::advance`] 时
    /// 必须传**相对本创建时刻**的纳秒数（首帧=0），不得传平台绝对 uptime ns（见 `advance` 文档）。
    pub fn new() -> Self {
        let player = PlayerState {
            x: INITIAL_PLAYER_X,
            y: INITIAL_PLAYER_Y,
            bomb_ammo: INITIAL_BOMB_AMMO,
            ..PlayerState::default()
        };
        Self {
            flow: FlowState::new(),
            player,
            stage_state: StageState::default(),
            stage_index: 1,
            all_clear: false,
            clock: FrameClock::new(),
            prev_buttons: Buttons::default(),
            audio: AudioPipe::new(),
            rom: None,
            suspended: false,
            quit: false,
            pacer: FixedStep::new(0),
            last_fx: StageFxSummary::default(),
            last_canvas_sig: 0,
            menu_latch: InputLatch::default(),
            menu_gate: MenuInputGate::default(),
            shop_state: ShopState::default(),
            pending_events: VecDeque::new(),
            stage_loaded: false,
            stage_load_frame: 0,
            stage_sched_records: 0,
            spawn_log: Vec::new(),
            prev_slot_types: [0; 28],
            last_mode: Mode::Boot,
            mode_frames: 0,
            auto_flow: true,
            player_bullets: BulletGrid::default(),
            shot_timer: 1,
            hits_this_frame: [None; 4],
            hit_overflow: 0,
            enemy_hit_log: Vec::new(),
            shape_data_shapes: 0,
        }
    }

    /// 逻辑帧号（虚拟时钟）。
    pub fn frame(&self) -> u64 {
        self.clock.frame()
    }

    /// 虚拟时钟毫秒（`frame*1000/60`，整数确定性）。
    pub fn now_ms(&self) -> u64 {
        self.clock.now_ms()
    }

    /// flow 状态（只读）。
    pub fn flow(&self) -> &FlowState {
        &self.flow
    }

    /// 当前模式。
    pub fn mode(&self) -> Mode {
        self.flow.mode
    }

    /// 关号（`$180`，1..=8）。
    pub fn stage_index(&self) -> u8 {
        self.stage_index
    }

    /// 是否已走 `AllClear` 边（结局到达）。
    pub fn all_clear(&self) -> bool {
        self.all_clear
    }

    /// 玩家状态（只读）。
    pub fn player(&self) -> &PlayerState {
        &self.player
    }

    /// 引擎状态（只读；排障/后续装载/对拍用）。
    pub fn stage_state(&self) -> &StageState {
        &self.stage_state
    }

    /// 引擎状态可变口：**仅供已证数据装载侧（`game-data` 解析产物）与门禁测试**。
    /// 平台实现不得写游戏状态（写游戏状态属 core 语义，会绕开差分）。
    pub fn stage_state_mut(&mut self) -> &mut StageState {
        &mut self.stage_state
    }

    /// 音频管道（已证音频写侧后续接线口；v1 生产者空）。
    pub fn audio(&self) -> &AudioPipe {
        &self.audio
    }

    /// 音频管道可变口（生产侧接线/契约测试注入）。
    pub fn audio_mut(&mut self) -> &mut AudioPipe {
        &mut self.audio
    }

    /// 已挂起（生命周期）。
    pub fn suspended(&self) -> bool {
        self.suspended
    }

    /// 退出请求已置位。
    pub fn quit_requested(&self) -> bool {
        self.quit
    }

    /// ROM 字节（`AssetSource` 已装载 ⇒ Some）。
    pub fn rom(&self) -> Option<&[u8]> {
        self.rom.as_deref()
    }

    /// 上一帧画布 hash（渲染合成确定性面）。
    pub fn last_canvas_sig(&self) -> u64 {
        self.last_canvas_sig
    }

    /// 上一帧引擎效果摘要。
    pub fn last_fx(&self) -> StageFxSummary {
        self.last_fx
    }

    /// 固定步折算器（只读，排障用）。
    pub fn pacer(&self) -> &FixedStep {
        &self.pacer
    }

    /// 菜单派发器状态（trait 契约测试/排障用；只读）。
    pub fn menu_dispatch_unlocked(&self) -> bool {
        self.menu_gate.phase_unlocked
    }

    /// 商店面板状态（`ui::ShopState`；只读）。
    pub fn shop_state(&self) -> ShopState {
        self.shop_state
    }

    /// 自动模式时序开/关（默认开）。关=replay/门禁测试按 capture 帧锚显式投递 `ModeEvent`
    /// （见 `docs/findings/plan-playable-loop.md` §7）；**不改变任何模式迁移合法性**。
    pub fn set_auto_flow(&mut self, enabled: bool) {
        self.auto_flow = enabled;
    }

    /// 待投递延迟事件数（响应时延队列；只读）。
    pub fn pending_events(&self) -> usize {
        self.pending_events.len()
    }

    // ================= 生命周期 =================

    /// 应用一个平台生命周期事件（宿主策略占位：suspend 停逻辑帧、quit 停一切）。
    pub fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Resume => self.suspended = false,
            AppEvent::Suspend => self.suspended = true,
            AppEvent::Quit => self.quit = true,
        }
    }

    /// 排空平台生命周期队列，返回处理事件数。
    pub fn poll_lifecycle(&mut self, lifecycle: &mut dyn Lifecycle) -> usize {
        let mut n = 0;
        while let Some(event) = lifecycle.poll_event() {
            self.handle_app_event(event);
            n += 1;
        }
        n
    }

    // ================= 资产 / 存档 =================

    /// 经 `AssetSource` 读取资产字节（v1 仅取得字节；解析/校验属 game-data，未接）。
    pub fn load_asset(&mut self, assets: &mut dyn AssetSource, path: &str) -> bool {
        match assets.read_all(path) {
            Some(bytes) => {
                if path == ROM_ASSET_PATH {
                    self.rom = Some(bytes);
                }
                true
            }
            None => false,
        }
    }

    /// 宿主最小快照（占位存档语义；见 `snapshot.rs`）。
    pub fn snapshot(&self) -> HostSnapshot {
        HostSnapshot {
            mode: self.flow.mode,
            stage: self.stage_index,
            all_clear: self.all_clear,
            player: self.player,
            persist: self.flow.data,
        }
    }

    /// 恢复宿主最小快照（引擎内部态不在快照内，见 `snapshot.rs` 注释）。
    pub fn restore(&mut self, snapshot: HostSnapshot) {
        self.flow = FlowState { mode: snapshot.mode, data: snapshot.persist };
        self.stage_index = snapshot.stage;
        self.all_clear = snapshot.all_clear;
        self.player = snapshot.player;
    }

    /// 经 `Storage` 保存快照（显式调用；**不自动保存**）。
    pub fn save_snapshot(&self, slot: u8, storage: &mut dyn Storage) -> bool {
        storage.save(slot, &encode_snapshot(&self.snapshot()))
    }

    /// 经 `Storage` 读取快照；损坏/缺失 ⇒ false（fail-closed，宿主状态不变）。
    pub fn load_snapshot(&mut self, slot: u8, storage: &mut dyn Storage) -> bool {
        let Some(bytes) = storage.load(slot) else {
            return false;
        };
        let Some(snapshot) = decode_snapshot(&bytes) else {
            return false;
        };
        self.restore(snapshot);
        true
    }

    // ================= STG-01 装载 =================

    /// 已装载 STG-01 关卡数据。
    pub fn stage_loaded(&self) -> bool {
        self.stage_loaded
    }

    /// 关卡数据装载所在宿主逻辑帧（滚动源偏移基准；见 [`STG01_LOAD_FRAME`]）。
    pub fn stage_load_frame(&self) -> u64 {
        self.stage_load_frame
    }

    /// 已装载调度流记录数（A 级 golden=43）。
    pub fn stage_sched_records(&self) -> usize {
        self.stage_sched_records
    }

    /// 敌槽激活事件日志（`$0400` 0→非0 判定式；对拍摘要用）。
    pub fn spawn_log(&self) -> &[SpawnEvent] {
        &self.spawn_log
    }

    /// 清空 spawn 日志（自检/对拍分段用）。
    pub fn clear_spawn_log(&mut self) {
        self.spawn_log.clear();
    }

    /// 从已装载 ROM 构造并装载 STG-01 `StageState`（`AssetSource` → bank8 → decode_sched）。
    ///
    /// 失败（ROM 未装载/bank 越界/调度流不是完整 43 记录）⇒ false，**不改宿主状态**（fail-closed）。
    pub fn load_stg01(&mut self) -> bool {
        let Some(rom) = self.rom.as_deref() else {
            return false;
        };
        let Some(bank8) = game_data::rom::prg_bank(rom, STG01_PRG_BANK) else {
            return false;
        };
        // 复制 bank8 以避免同时借用 self.rom 与 &mut self（8KB，装载期一次开销）。
        let bank8 = bank8.to_vec();
        if !self.load_stg01_from_bank8(&bank8) {
            return false;
        }
        // 框形数据（engine 命中管线所需；task-37）；失败=0 形 ⇒ 命中路径 fail-closed（不伪造）
        self.load_shape_data();
        true
    }

    /// 从 ROM bank2（`$8000` 窗）解析敌框形流填 `StageState.shape_data`（task-37；A 级表/格式）。
    ///
    /// 返回装载形数；ROM 未装载/bank 越界 ⇒ 0（fail-closed）。重复地址只保留首个。
    pub fn load_shape_data(&mut self) -> usize {
        let Some(rom) = self.rom.as_deref() else {
            return 0;
        };
        let Some(bank2) = game_data::rom::prg_bank(rom, shape::SHAPE_BANK) else {
            return 0;
        };
        let bank2 = bank2.to_vec();
        let n = shape::load_shape_data_from_bank2(&bank2, &mut self.stage_state);
        self.shape_data_shapes = n;
        n
    }

    /// 从 PRG bank8 窗口字节装载 STG-01（上游已装载 ROM 窗/门禁测试用；语义同 [`Host::load_stg01`]）。
    ///
    /// 构造（全部现有 game-core 公开 API）：`ScriptCursor::stg01` + `SchedCursor::load(decode_sched($A26D))`
    /// + 自机对象初态（A 级 trace 快照：槽0 `$0400=1/x=128/y=160`）。不写槽 1-7 型值
    /// （f1264 trace 快照其余槽语义未证，不发明）。
    pub fn load_stg01_from_bank8(&mut self, bank8: &[u8]) -> bool {
        let stream = decode_sched(bank8, STG01_SCHED_STREAM_ADDR, STG01_SCHED_MAX_RECORDS);
        if stream.records.len() != STG01_SCHED_RECORDS || stream.unknown_ctrl.is_some() {
            return false; // A 级 golden 不满足 ⇒ fail-closed（不是完整 STG-01 视界）
        }
        let mut stage = StageState {
            sched: engine::SchedCursor::load(stream),
            script: ScriptCursor::stg01(),
            scene: "stg01".into(),
            ..StageState::default()
        };
        stage.grid.slots[0] = engine::ObjectSlot {
            type_: PLAYER_OBJECT_TYPE,
            x: INITIAL_PLAYER_X,
            y: INITIAL_PLAYER_Y,
            ..engine::ObjectSlot::default()
        };
        self.stage_state = stage;
        self.stage_loaded = true;
        self.stage_load_frame = self.clock.frame();
        self.stage_sched_records = STG01_SCHED_RECORDS;
        self.prev_slot_types = [0; 28];
        self.spawn_log.clear();
        // task-37：弹/命中接线状态随关卡重置
        self.player_bullets = BulletGrid::default();
        self.shot_timer = 1;
        self.hits_this_frame = [None; 4];
        self.hit_overflow = 0;
        self.enemy_hit_log.clear();
        self.shape_data_shapes = 0;
        true
    }

    /// 当前帧对应的 STG-01 ROM 帧号（`STG01_LOAD_FRAME + (host_frame - stage_load_frame)`）。
    pub fn stage_abs_frame(&self, host_frame: u64) -> u32 {
        STG01_LOAD_FRAME.saturating_add(host_frame.saturating_sub(self.stage_load_frame) as u32)
    }

    /// 本帧滚动增量（px）：`-stage::scroll_delta(abs).1`（A 级逐帧公式；`Combat` 外 v1 亦用同一源）。
    ///
    /// 口径：stg01 stage clock 与 `.scroll.jsonl` 对齐（A 级，见 `tests/stg01_replay.rs` C1）。
    pub fn stage_scroll_delta(&self, host_frame: u64) -> u16 {
        let (_, dy) = game_core::stage::scroll_delta(self.stage_abs_frame(host_frame));
        if dy < 0 {
            (-(i32::from(dy))) as u16
        } else {
            0
        }
    }

    // ================= 玩家弹丸/命中（task-37） =================

    /// 已装载敌框形流数（0=命中路径 fail-closed，见 plan-bullets-hit §6）。
    pub fn shape_data_shapes(&self) -> usize {
        self.shape_data_shapes
    }

    /// `$39/$3A` 重复计时器当前值（`B` 长按门控；只读）。
    pub fn shot_timer(&self) -> u8 {
        self.shot_timer
    }

    /// 本帧敌命中事件（≤4；观测 engine 命中管线结果）。
    pub fn hits_this_frame(&self) -> &[Option<EnemyHitEvent>; 4] {
        &self.hits_this_frame
    }

    /// 本帧命中事件溢出计数（>4 条时；完整列表见 [`Host::enemy_hit_log`]）。
    pub fn hit_overflow(&self) -> u16 {
        self.hit_overflow
    }

    /// 全部敌命中事件日志（replay/单测；观测式，见 fire.rs `scan_enemy_hits`）。
    pub fn enemy_hit_log(&self) -> &[EnemyHitEvent] {
        &self.enemy_hit_log
    }

    /// 本帧玩家子弹视图（20 槽按玩家槽序；只读）。
    pub fn player_bullets(&self) -> [Option<crate::fire::BulletView>; 20] {
        crate::fire::player_bullet_views(&self.player_bullets)
    }

    // ================= 宿主状态机（接 flow 已证边集） =================

    /// 投递一个 flow 模式事件（只走 `game_core::flow` 已证边集 + 宿主关号守卫）。
    ///
    /// 守卫（A 级关末/通关链，`l5-all-clear.md` R-l5-4/5）：
    /// - `StageAdvance`（E37C `INC $180` 后关间）仅在 `stage_index<8` 合法；
    ///   `stage_index==8` 时 ROM 语义是 `$180:=9` → `AllClear`，故 fail-closed；
    /// - `AllClear`（E392 `CMP #9`）仅在 `stage_index==8` 合法。
    ///
    /// 成功时同步关号/通关标志；失败返回 core 的 fail-closed 原因（宿主状态不变）。
    /// **v1 驱动方=平台/上层**：除 `Combat→StageClear` 自动接线外，模式事件由平台显式投递
    /// 或本宿主菜单派发（输入驱动的已证序列）投递。
    pub fn apply_event(&mut self, event: ModeEvent) -> Result<Mode, String> {
        match event {
            ModeEvent::StageAdvance if self.stage_index >= 8 => {
                return Err(format!(
                    "宿主守卫 fail-closed：关 8 关末（$180==8）应走 AllClear（E392 CMP #9），不得 StageAdvance"
                ));
            }
            ModeEvent::AllClear if self.stage_index != 8 => {
                return Err(format!(
                    "宿主守卫 fail-closed：AllClear 仅当 $180==8（E392 CMP #9），实际关号 {}",
                    self.stage_index
                ));
            }
            _ => {}
        }
        let next = self.flow.advance(event)?;
        match event {
            ModeEvent::StageAdvance => self.stage_index = self.stage_index.saturating_add(1),
            ModeEvent::AllClear => self.all_clear = true,
            _ => {}
        }
        self.flow = next;
        Ok(self.flow.mode)
    }

    /// 平台/上层显式投递事件入口：同 [`Host::apply_event`] 的已证边校验 +
    /// **宿主副作用**（转场帧 `ExitTransition` 成功后装载 STG-01，A 级 f1193 锚）。
    ///
    /// 成功返回实际投递的事件；失败 `None`（状态不变，fail-closed）。
    pub fn apply_event_hooked(&mut self, event: ModeEvent) -> Option<ModeEvent> {
        self.apply_event_with_side_effects(event)
    }

    /// 投递事件并处理宿主副作用（转场帧装载 STG-01）；失败 ⇒ None（状态不变）。
    fn apply_event_with_side_effects(&mut self, event: ModeEvent) -> Option<ModeEvent> {
        match self.apply_event(event) {
            Ok(_) => {
                if event == ModeEvent::ExitTransition && self.flow.mode == Mode::Transition {
                    // 转场帧装载 STG-01（A 级：scroll trace 起点 f1193；装载失败=fail-closed 不伪造）
                    let _ = self.load_stg01();
                }
                Some(event)
            }
            Err(_) => None,
        }
    }

    // ================= 菜单/模式派发（输入驱动；证据见 plan-playable-loop §4） =================

    /// 标题/菜单/商店段输入派发（**触发键按模式分离**；证据/未证域见
    /// `docs/findings/plan-menuinput-fix.md` §4/§5）。
    ///
    /// - `Title | Menu`：`$18` 锁存 + `MenuInputGate` 两层门，**只喂 START 位**——严格对齐 A 级
    ///   START 专用早退门（bank00:92AA/9400 `LDA $18 / AND #$10`；`flow-response-latency.md` §8），
    ///   消除其它键 12f 保持导致的跨键吞按；
    /// - `Shop | ShopInfo`：`$16` **原始按下沿**（键级边沿）——Shop B 的保持/边沿门未证，
    ///   边沿模型=一次按下一次动作（fail-closed，不发明连发/保持重复）；两连 B 短间隔（2..12f）可识别；
    /// - 其它模式不派发（避免层 1 状态被 gameplay 输入污染）。
    fn dispatch_menu(&mut self, frame: u64, pressed: u8) {
        let trigger = match self.flow.mode {
            Mode::Title | Mode::Menu => {
                if self
                    .menu_gate
                    .dispatch(self.menu_latch.bitmap_18 & crate::input::MENU_START)
                    .is_some()
                {
                    crate::input::MENU_START
                } else {
                    0
                }
            }
            Mode::Shop | Mode::ShopInfo => pressed & crate::input::MENU_B,
            _ => 0,
        };
        match self.flow.mode {
            Mode::Title if trigger & crate::input::MENU_START != 0 => {
                let _ = self.apply_event_with_side_effects(ModeEvent::Menu);
            }
            Mode::Menu if trigger & crate::input::MENU_START != 0 => {
                let _ = self.apply_event_with_side_effects(ModeEvent::ShopArrival);
            }
            Mode::Shop if trigger & crate::input::MENU_B != 0 => {
                self.shop_state = self.shop_state.on_b(); // Main→Info（R-shop-6）
                let _ = self.apply_event_with_side_effects(ModeEvent::ShopPage);
            }
            Mode::ShopInfo if trigger & crate::input::MENU_B != 0 => {
                self.shop_state = self.shop_state.on_b(); // Info→Exited（R-shop-7 两连 B）
                // 出店转场响应 +33f（A 级：press@1160→f1193）⇒ 延迟投递 ExitTransition。
                let due = frame.saturating_add(SHOP_EXIT_RESPONSE_FRAMES.saturating_sub(1));
                self.pending_events.push_back((due, ModeEvent::ExitTransition));
            }
            _ => {}
        }
    }

    /// 帧锚自动推进 + 阶段计时（Transition 33f / StageStart 240f 常数，A 级；帧内分布未证）。
    fn auto_advance(&mut self, frame: u64) {
        if !self.auto_flow {
            return;
        }
        match self.flow.mode {
            Mode::Boot if frame >= TITLE_BLACK_FRAME => {
                let _ = self.apply_event_with_side_effects(ModeEvent::Black);
            }
            Mode::Black if frame >= TITLE_FRAME => {
                let _ = self.apply_event_with_side_effects(ModeEvent::Title);
            }
            Mode::Transition => {
                self.mode_frames = self.mode_frames.saturating_add(1);
                if self.mode_frames >= TRANSITION_TAIL_FRAMES_HOST {
                    let _ = self.apply_event_with_side_effects(ModeEvent::StageStart);
                }
            }
            Mode::StageStart => {
                self.mode_frames = self.mode_frames.saturating_add(1);
                if self.mode_frames >= STAGE_INTRO_FRAMES_HOST {
                    let _ = self.apply_event_with_side_effects(ModeEvent::MidCombat);
                }
            }
            _ => {}
        }
    }

    /// 到点的延迟模式事件（响应时延队列）。
    fn due_events(&mut self, frame: u64) {
        loop {
            let Some(&(due, event)) = self.pending_events.front() else {
                break;
            };
            if due > frame {
                break;
            }
            self.pending_events.pop_front();
            let _ = self.apply_event_with_side_effects(event);
        }
    }

    // ================= 引擎 tick / spawn 落地 =================

    /// 引擎 tick 活跃模式（`Combat`=已证 gameplay 段；`Transition/StageStart`=B 级接线依据；
    /// `DeathRespawn/StageClear`=接线约定、细粒度未隔离 G-S5-5——见 platform-shell §8）。
    fn engine_tick_active(mode: Mode) -> bool {
        matches!(
            mode,
            Mode::Transition | Mode::StageStart | Mode::Combat | Mode::DeathRespawn | Mode::StageClear
        )
    }

    /// 玩家移动活跃模式：`Combat` + `StageStart`。
    ///
    /// - `Combat`=已证 gameplay 段；
    /// - `StageStart`=**A 级 trace 证据**：`gameplay-stage1.player.jsonl` f1301 首移
    ///   （x=128→130、held=$81），该帧落在 flow capture「stage-start f1260→mid-combat f1500」窗内
    ///   ⇒ 关始段玩家可动（推翻"可动性未证"的旧保守假设；以 oracle 逐帧向量为准）；
    /// - `DeathRespawn` 窗内 x/y/亚像素冻结=A 级（`respawn-window.md` §3 R-respawn-2）⇒ 不跑；
    /// - `StageClear` 飞离段由引擎状态向量驱动，输入不参与（未证不发明）。
    fn player_tick_active(mode: Mode) -> bool {
        matches!(mode, Mode::Combat | Mode::StageStart)
    }

    /// spawn 效果落地：`Pair → land_pair`；`Spawn（组长）→ land_leader(...,None)`；
    /// 其余 item（Jump/SetType/Reg/Unknown）不落地=fail-closed（SetType 语义未接线）。
    ///
    /// 口径：落地发生在 `stage_tick` 返回后的同逻辑帧内（1 帧滴灌延迟已知偏差，见 crate 文档）。
    fn apply_stage_effects(&mut self, fx: &StageEffects) {
        for item in fx.spawn_events.iter().chain(fx.drip_events.iter()) {
            match *item {
                SchedItem::Pair { type_, param } => {
                    let _ = engine::land_pair(&mut self.stage_state.grid, type_, param);
                }
                SchedItem::Spawn { type_, args, .. } => {
                    let _ = engine::land_leader(&mut self.stage_state.grid, type_, args, None);
                }
                _ => {} // fail-closed：不发明未接线 item 的落地语义
            }
        }
    }

    /// 扫描槽型值 0→非0（trace 同口径判定式；槽复用不可见——与 oracle 抽取口径一致）。
    fn scan_spawns(&mut self, frame: u64) {
        for slot in 0..28usize {
            let type_code = self.stage_state.grid.slots[slot].type_;
            if self.prev_slot_types[slot] == 0 && type_code != 0 {
                self.spawn_log.push(SpawnEvent { frame, slot: slot as u8, type_code });
            }
            self.prev_slot_types[slot] = type_code;
        }
    }

    fn live_bullet_count(&self) -> u16 {
        self.player_bullets.state.iter().filter(|&&s| s != 0).count() as u16
    }

    fn player_bullet_count(&self) -> u16 {
        PLAYER_ONLY_SLOTS
            .iter()
            .chain(SHARED_SLOTS.iter())
            .filter(|&&i| self.player_bullets.state[i as usize] != 0)
            .count() as u16
    }

    // ================= 帧推进 =================

    /// 60Hz 固定步包装：把平台单调时钟推进到 `now_ns`，执行折算出的 0..=4 个逻辑帧。
    ///
    /// **时间基准**：`now_ns` 必须**相对 `Host` 创建时刻**（首帧=0；平台侧用
    /// `Instant::now() - instant_at_host_creation` 折算）。禁止直接传平台绝对 uptime ns——
    /// 否则首帧会把巨大差值当积压（仅受追帧上限 4 保护，造成启动偏移）。
    ///
    /// **返回值**：折算出的逻辑帧数（0..=4）。suspend/quit 期间这些帧由 [`Host::tick`]
    /// 门控为**不推进逻辑**（返回计数仍含折算帧，不等于实际推进帧数）；实际推进帧数=
    /// 前后 [`Host::frame`] 之差。
    pub fn advance(
        &mut self,
        now_ns: u64,
        input: &dyn Input,
        renderer: &mut dyn Renderer,
        audio: &mut dyn Audio,
    ) -> u32 {
        let frames = self.pacer.advance(now_ns);
        for _ in 0..frames {
            self.tick(input, &mut *renderer, &mut *audio);
        }
        frames
    }

    /// 一个逻辑帧（确定性核心）：输入锁存/派发→（按模式门）玩家/引擎 tick→效果消费→
    /// 音频流消费→渲染合成+present→虚拟时钟推进。suspend/quit 时**不推进、不呈现**。
    pub fn tick(
        &mut self,
        input: &dyn Input,
        renderer: &mut dyn Renderer,
        audio: &mut dyn Audio,
    ) -> FrameOutcome {
        let frame = self.clock.frame();
        let raw = input.buttons();
        let buttons = resolve_directions(raw);
        let pressed = pressed_bitmap(&self.prev_buttons, &buttons);

        if self.quit || self.suspended {
            return FrameOutcome {
                frame,
                mode: self.flow.mode,
                stage: self.stage_index,
                buttons,
                pressed,
                advanced: false,
                gameplay_tick: false,
                stage_clear_requested: false,
                flow_event: None,
                quit: self.quit,
                suspended: self.suspended,
            };
        }

        // ① 菜单 `$18` 锁存：先推进上一帧 pending（press@N→N+1f），再压入本帧按下沿。
        //    **只锁存 START 位**：A 级早退门=bank00:92AA/9400 `LDA $18/AND #$10`（START 专用），
        //    其它键（方向/A/B）不得参与层 1 保持判定（修跨键吞按，task-29 / plan-menuinput-fix §5）。
        self.menu_latch.tick_frame();
        self.menu_latch.press(menu_pressed_bitmap(&self.prev_buttons, &buttons) & crate::input::MENU_START);

        // ② 标题段输入锁（九班 (450,550]，A 级引证）：>550 解锁。
        if frame > MENU_INPUT_UNLOCK_FRAME {
            self.menu_gate.phase_unlocked = true;
        }

        // ③ 菜单/模式派发（输入驱动已证序列；触发键按模式分离，见 dispatch_menu 文档）
        self.dispatch_menu(frame, pressed);

        // ④ 帧锚自动推进 + 转场/关始阶段计时
        self.auto_advance(frame);

        // ⑤ 响应时延队列到点
        self.due_events(frame);

        let held = held_bitmap(&buttons);
        let mode_before = self.flow.mode;
        let mut gameplay_tick = false;
        let mut stage_clear_requested = false;
        let mut flow_event = None;

        // ⑥a 玩家移动（仅 Combat；重生窗冻结=A 级）。与关卡装载解耦：移动逻辑不依赖 StageState。
        if Self::player_tick_active(mode_before) {
            self.player = player::tick(&self.player, held);
        }

        // ⑥b 引擎 tick（已装载 + 已接线模式）
        if self.stage_loaded && Self::engine_tick_active(mode_before) {
            gameplay_tick = true;
            self.hits_this_frame = [None; 4];
            self.hit_overflow = 0;
            // ⑥b-1 玩家主武器（task-37）：B 长按（`$18` bit6）门控 + `$39/$3A` 计时器 → 六型生成器；
            //        然后**直接 `shot::tick`** 推进玩家弹（寿命/脚本机/LUT/死亡回收）。只在 Combat 段；
            //        其它 engine 模式玩家弹冻结（未证域，fail-closed；不生成不移动）。
            if mode_before == Mode::Combat {
                // R-shot-gen-4：`$34` weapon_slot=0 六型连发（本卡唯一已接分派）；
                // 非 0 分派（type3/4/6 齐射/退出）未接 ⇒ fail-closed 不生成（登记 plan §6）。
                if self.flow.data.weapon_slot == 0 {
                    let b_held = held & crate::input::MENU_B != 0;
                    if fire::fire_gate(b_held, &mut self.shot_timer) {
                        let _ = fire::generate_shots(
                            &mut self.player_bullets,
                            self.player.x,
                            self.player.y,
                            self.flow.data.weapon_level,
                        );
                    }
                } else {
                    self.shot_timer = 1;
                }
                let _released = fire::step_player_bullets(&mut self.player_bullets);
            } else {
                self.shot_timer = 1;
            }
            let scroll_delta = self.stage_scroll_delta(frame);
            let fx = engine::stage_tick(&mut self.stage_state, &TickInputs { scroll_delta });
            self.last_fx = StageFxSummary::from_effects(&fx);
            stage_clear_requested = fx.stage_clear_requested;
            self.apply_stage_effects(&fx);
            self.scan_spawns(frame);
            // ⑥b-2 shell 侧命中管线（task-38 §2.4）：注册/敌框/4 边属性/清框回收 + 受限额伤害。
            //        engine combat_frame 只见空 `StageState.bullets`（玩家弹在自有网格）⇒ 不重复判定。
            if mode_before == Mode::Combat {
                let events =
                    fire::resolve_player_hits(&mut self.player_bullets, &mut self.stage_state, frame);
                for event in events {
                    self.enemy_hit_log.push(event);
                    if let Some(free) = self.hits_this_frame.iter_mut().find(|e| e.is_none()) {
                        *free = Some(event);
                    } else {
                        self.hit_overflow = self.hit_overflow.saturating_add(1);
                    }
                }
            }
            // 已证自动接线：Combat→StageClear（flow 边集 + engine `$26 bit4` 门）。
            if mode_before == Mode::Combat && stage_clear_requested {
                if self.apply_event_with_side_effects(ModeEvent::StageClear).is_some() {
                    flow_event = Some(ModeEvent::StageClear);
                }
            }
        } else {
            self.shot_timer = 1;
        }

        // ⑦ Audio 流消费（每帧恰一次 end_frame；v1 生产者空）。
        self.audio.flush(audio, frame as u32);

        // ⑧ 渲染合成 + present（v1 占位：清屏 + 已证坐标面玩家块；view 供平台侧只读消费）。
        let view = self.frame_view();
        self.last_canvas_sig = compose_frame(renderer.canvas_mut(), &view);
        renderer.present();

        self.prev_buttons = buttons;
        self.clock.tick();

        // 模式变化检测（阶段计时基准）
        if self.flow.mode != self.last_mode {
            self.last_mode = self.flow.mode;
            self.mode_frames = 0;
        }

        FrameOutcome {
            frame,
            mode: self.flow.mode,
            stage: self.stage_index,
            buttons,
            pressed,
            advanced: true,
            gameplay_tick,
            stage_clear_requested,
            flow_event,
            quit: self.quit,
            suspended: self.suspended,
        }
    }

    /// 只读渲染视图（平台侧渲染合成消费；未证域显式 None/TODO——见 `render.rs`）。
    pub fn frame_view(&self) -> FrameView {
        let mut enemy_slots = [EnemySlotView::EMPTY; 20];
        for (i, view) in enemy_slots.iter_mut().enumerate() {
            let slot = 8 + i;
            let s = &self.stage_state.grid.slots[slot];
            *view = EnemySlotView {
                slot: slot as u8,
                type_code: if s.type_ == 0 { None } else { Some(s.type_) },
                // 坐标写者链（spawn 定位/行为 VM 位置写）未接线/未证 ⇒ 显式 None，
                // 不得把 None 当 (0,0)（platform-shell §8 / plan-playable-loop §6）。
                position: None,
            };
        }
        FrameView {
            frame: self.clock.frame(),
            mode: self.flow.mode,
            stage: self.stage_index,
            player: self.player,
            live_bullets: self.live_bullet_count(),
            player_bullets_live: self.player_bullet_count(),
            player_bullets: crate::fire::player_bullet_views(&self.player_bullets),
            hits_this_frame: self.hits_this_frame,
            enemy_slots,
            hud: HudView {
                money_digits: render_digits(&self.flow.data.money),
                bomb_ammo: self.player.bomb_ammo,
                stage: self.stage_index,
            },
        }
    }

    /// 确定性状态签名（FNV-1a 64）：覆盖宿主可观测状态 + 派发/装载/上一帧画布与音频摘要。
    ///
    /// 用途：mock/STG-01 replay 契约测试「两次一致」；**不声称**等价于真机 trace 对拍。
    pub fn signature(&self) -> u64 {
        let mut h = FNV_OFFSET;
        hash_u64(&mut h, self.clock.frame());
        hash_u64(&mut h, self.clock.now_ms());
        hash_bytes(
            &mut h,
            &[
                mode_tag(self.flow.mode),
                self.stage_index,
                u8::from(self.all_clear),
                u8::from(self.suspended),
                u8::from(self.quit),
                u8::from(self.stage_loaded),
            ],
        );
        hash_u64(&mut h, self.stage_load_frame);
        hash_u64(&mut h, self.stage_sched_records as u64);
        hash_u64(&mut h, self.mode_frames as u64);
        hash_u64(&mut h, self.pending_events.len() as u64);
        hash_u64(&mut h, u64::from(self.menu_latch.bitmap_18));
        hash_bytes(
            &mut h,
            &[
                u8::from(self.menu_gate.phase_unlocked),
                u8::from(self.menu_gate.prev_held),
                match self.shop_state {
                    ShopState::Main => 0,
                    ShopState::Info => 1,
                    ShopState::Exited => 2,
                },
                held_bitmap(&self.prev_buttons),
            ],
        );
        hash_bytes(
            &mut h,
            &[
                self.player.x,
                self.player.y,
                self.player.x_subpix,
                self.player.y_subpix,
                self.player.bomb_ammo,
                self.player.action_mode,
            ],
        );
        hash_bytes(
            &mut h,
            &[self.flow.data.weapon_slot, self.flow.data.weapon_level, self.flow.data.bomb_ammo],
        );
        hash_bytes(&mut h, &self.flow.data.money.digits);
        hash_u64(&mut h, u64::from(self.stage_state.frame));
        hash_u64(&mut h, u64::from(self.stage_state.wave.tick));
        hash_u64(&mut h, u64::from(self.stage_state.wave.scroll_progress));
        hash_bytes(&mut h, &[self.stage_state.wave.flags_26]);
        hash_u64(&mut h, self.stage_state.sched.next as u64);
        hash_u64(&mut h, self.stage_state.script.next as u64);
        for (slot, prev) in self.stage_state.grid.slots.iter().zip(self.prev_slot_types.iter()) {
            hash_bytes(&mut h, &[slot.type_, *prev]);
        }
        hash_u64(&mut h, u64::from(self.live_bullet_count()));
        hash_u64(&mut h, u64::from(self.player_bullet_count()));
        // task-37：玩家弹 20 槽六字段（坐标/型/寿命/动作指针/运动码）——逐槽哈希（确定性回归锚）
        for &slot in crate::fire::PLAYER_SLOT_ORDER.iter() {
            let i = slot as usize;
            hash_bytes(
                &mut h,
                &[
                    self.player_bullets.state[i],
                    self.player_bullets.x[i],
                    self.player_bullets.y[i],
                    self.player_bullets.type_code[i],
                    self.player_bullets.motion[i],
                    self.player_bullets.action_ptr[i],
                ],
            );
        }
        hash_u64(&mut h, u64::from(self.shot_timer));
        hash_u64(&mut h, self.shape_data_shapes as u64);
        hash_u64(&mut h, self.enemy_hit_log.len() as u64);
        for event in &self.enemy_hit_log {
            hash_u64(&mut h, event.frame);
            hash_bytes(&mut h, &[event.enemy_slot, event.bullet_slot, event.damage, u8::from(event.fail_closed)]);
        }
        hash_u64(&mut h, u64::from(self.hit_overflow));
        hash_u64(&mut h, self.spawn_log.len() as u64);
        for event in &self.spawn_log {
            hash_u64(&mut h, event.frame);
            hash_bytes(&mut h, &[event.slot, event.type_code]);
        }
        hash_bytes(
            &mut h,
            &[
                self.last_fx.spawn_events as u8,
                self.last_fx.drip_events as u8,
                self.last_fx.far_call_requests as u8,
                self.last_fx.phase_trace as u8,
                u8::from(self.last_fx.stage_clear_requested),
            ],
        );
        hash_u64(&mut h, self.last_canvas_sig);
        hash_u64(&mut h, self.audio.flushed_events());
        hash_u64(&mut h, self.audio.write_hash());
        h
    }
}

/// ROM 资产**逻辑路径**（相对资产根；跨平台语义一致）。
///
/// 资产根由平台实现决定：PC=`std::fs` 实现以**可执行文件所在目录**为基准
/// （`exe/rom/gunnac_cn.nes`；可回退 `exe/../rom/` 与 `CARGO_MANIFEST_DIR` 开发态定位）；
/// Android=APK `assets/` 根（`AAssetManager` 内同相对路径）。**不得依赖进程 cwd**。
pub const ROM_ASSET_PATH: &str = "rom/gunnac_cn.nes";

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}

/// 宿主最小快照的持久化数据（re-export，便于平台侧做存档 UI）。
pub use game_core::flow::PersistData as HostPersistData;

// 测试模块（task-9：菜单派发 / STG-01 装载 / FrameView / 确定性）
#[cfg(test)]
mod tests {
    use super::{
        Host, SpawnEvent, INITIAL_BOMB_AMMO, INITIAL_PLAYER_X, INITIAL_PLAYER_Y, ROM_ASSET_PATH,
        STG01_SCHED_RECORDS, STG01_SCHED_STREAM_ADDR,
    };
    use crate::fixed_step::FRAME_NS;
    use game_core::flow::{Mode, ModeEvent};
    use game_platform::{
        AppEvent, AssetSource, Audio, Buttons, Input, Lifecycle, Renderer, Storage, CANVAS_LEN,
    };

    // ---------------- 平台 mock（src 内最小面；零禁词） ----------------

    struct FixedInput(Buttons);

    impl Input for FixedInput {
        fn buttons(&self) -> Buttons {
            self.0
        }
    }

    fn no_input() -> FixedInput {
        FixedInput(Buttons::default())
    }

    fn right_only() -> FixedInput {
        FixedInput(Buttons { right: true, ..Buttons::default() })
    }

    struct NullRenderer {
        canvas: Vec<u8>,
        presents: u32,
    }

    impl NullRenderer {
        fn new() -> Self {
            Self { canvas: vec![0u8; CANVAS_LEN], presents: 0 }
        }
    }

    impl Renderer for NullRenderer {
        fn canvas_mut(&mut self) -> &mut [u8] {
            &mut self.canvas
        }
        fn present(&mut self) {
            self.presents += 1;
        }
    }

    #[derive(Default)]
    struct RecAudio {
        writes: Vec<(u32, u16, u8)>,
        frames: Vec<u32>,
    }

    impl Audio for RecAudio {
        fn write_apu(&mut self, frame: u32, reg: u16, value: u8) {
            self.writes.push((frame, reg, value));
        }
        fn end_frame(&mut self, frame: u32) {
            self.frames.push(frame);
        }
    }

    #[derive(Default)]
    struct SlotStorage {
        slots: [Option<Vec<u8>>; 8],
    }

    impl Storage for SlotStorage {
        fn load(&mut self, slot: u8) -> Option<Vec<u8>> {
            self.slots.get(slot as usize).cloned().flatten()
        }
        fn save(&mut self, slot: u8, data: &[u8]) -> bool {
            if let Some(s) = self.slots.get_mut(slot as usize) {
                *s = Some(data.to_vec());
                true
            } else {
                false
            }
        }
    }

    struct FixtureAssets;

    impl AssetSource for FixtureAssets {
        fn read_all(&mut self, path: &str) -> Option<Vec<u8>> {
            if path == ROM_ASSET_PATH {
                Some(vec![0x4E, 0x45, 0x53, 0x1A, 0x00, 0x01])
            } else {
                None
            }
        }
    }

    #[derive(Default)]
    struct QueueLifecycle {
        events: std::collections::VecDeque<AppEvent>,
    }

    impl Lifecycle for QueueLifecycle {
        fn poll_event(&mut self) -> Option<AppEvent> {
            self.events.pop_front()
        }
    }

    /// 合成 PRG bank8 窗口（门禁测试用）：43 条调度记录 + 环闭合（语义=计数/环检测，不对应真 ROM）。
    ///
    /// 布局：$A26D 起每记录 `[delay][ctrl0]`（ctrl0=记录分隔，下一记录=+2）；第 43 条以
    /// `[delay][ctrl3 lo hi]` 跳回 $A26D ⇒ decode_sched 见环终止、记录数=43、truncated=false。
    fn synthetic_stg01_bank8() -> Vec<u8> {
        let mut window = vec![0u8; 0x2000];
        let start = usize::from(STG01_SCHED_STREAM_ADDR - 0xA000);
        for i in 0..(STG01_SCHED_RECORDS - 1) {
            let pos = start + i * 2;
            window[pos] = if i == 0 { 0 } else { 1 };
            window[pos + 1] = 0; // ctrl0 记录分隔
        }
        let pos = start + (STG01_SCHED_RECORDS - 1) * 2;
        window[pos] = 1;
        window[pos + 1] = 3; // ctrl3 跳转
        let target = STG01_SCHED_STREAM_ADDR.to_le_bytes();
        window[pos + 2] = target[0];
        window[pos + 3] = target[1];
        window
    }

    /// 合成非法视界（非 43 记录）。
    fn incomplete_bank8() -> Vec<u8> {
        let mut window = vec![0u8; 0x2000];
        let start = usize::from(STG01_SCHED_STREAM_ADDR - 0xA000);
        window[start] = 0;
        window[start + 1] = 0;
        window
    }

    // ---------------- 宿主状态机 ----------------

    /// 开机 → 关 1 战斗（全部走 flow 已证边集）。
    fn walk_to_combat(host: &mut Host) {
        for ev in [
            ModeEvent::Black,
            ModeEvent::Title,
            ModeEvent::Menu,
            ModeEvent::ShopArrival,
            ModeEvent::ExitTransition,
            ModeEvent::StageStart,
            ModeEvent::MidCombat,
        ] {
            host.apply_event(ev).expect("已证边迁移");
        }
        assert_eq!(host.mode(), Mode::Combat);
    }

    /// 关末 →（关间/通关）回到下一关战斗 / 结局。
    fn clear_stage(host: &mut Host) {
        host.apply_event(ModeEvent::StageClear).expect("关末门");
        assert_eq!(host.mode(), Mode::StageClear);
        if host.stage_index() < 8 {
            host.apply_event(ModeEvent::StageAdvance).expect("关间");
            assert_eq!(host.mode(), Mode::Shop);
            for ev in [ModeEvent::ExitTransition, ModeEvent::StageStart, ModeEvent::MidCombat] {
                host.apply_event(ev).expect("下一关已证边");
            }
            assert_eq!(host.mode(), Mode::Combat);
        } else {
            host.apply_event(ModeEvent::AllClear).expect("通关");
            assert_eq!(host.mode(), Mode::Ending);
        }
    }

    #[test]
    fn boot_state_is_deterministic_and_uses_proven_anchors() {
        let a = Host::new();
        let b = Host::new();
        assert_eq!(a.signature(), b.signature(), "同构造⇒同签名");
        assert_eq!(a.mode(), Mode::Boot);
        assert_eq!(a.stage_index(), 1);
        assert!(!a.all_clear() && !a.suspended() && !a.quit_requested());
        assert!(!a.stage_loaded(), "未装载前 stage_loaded=false");
        assert_eq!((a.player().x, a.player().y), (INITIAL_PLAYER_X, INITIAL_PLAYER_Y), "已证复位锚点 (128,160)");
        assert_eq!(a.player().bomb_ammo, INITIAL_BOMB_AMMO, "bank00:9C5D $018F:=4");
        assert_eq!(a.frame(), 0);
    }

    #[test]
    fn walks_title_to_eight_stages_to_ending_via_proven_edges_only() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        for stage in 1..=8u8 {
            assert_eq!(host.stage_index(), stage, "关号同步（E37C INC $180）");
            clear_stage(&mut host);
        }
        assert_eq!(host.mode(), Mode::Ending, "8 关通关→结局（$95F9）");
        assert!(host.all_clear(), "AllClear 边已走");
        assert_eq!(host.stage_index(), 8);
    }

    #[test]
    fn host_guards_fail_closed_on_stage_consistency() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        assert!(host.apply_event(ModeEvent::AllClear).is_err(), "非关 8 AllClear fail-closed");
        walk_to_combat(&mut host);
        host.apply_event(ModeEvent::StageClear).expect("关末门");
        assert!(host.apply_event(ModeEvent::AllClear).is_err(), "关 1 AllClear fail-closed");
        host.apply_event(ModeEvent::StageAdvance).expect("关 1 关间");
        for _ in 2..=8u8 {
            for ev in [ModeEvent::ExitTransition, ModeEvent::StageStart, ModeEvent::MidCombat] {
                host.apply_event(ev).expect("下一关已证边");
            }
            host.apply_event(ModeEvent::StageClear).expect("关末门");
            if host.stage_index() < 8 {
                host.apply_event(ModeEvent::StageAdvance).expect("关间");
            }
        }
        assert_eq!(host.stage_index(), 8);
        assert_eq!(host.mode(), Mode::StageClear);
        assert!(host.apply_event(ModeEvent::StageAdvance).is_err(), "关 8 StageAdvance fail-closed");
        assert_eq!(host.mode(), Mode::StageClear, "失败不改状态");
        host.apply_event(ModeEvent::AllClear).expect("关 8 通关");
    }

    #[test]
    fn unproven_flow_edge_is_rejected() {
        let mut host = Host::new();
        assert!(host.apply_event(ModeEvent::MidCombat).is_err());
        assert_eq!(host.mode(), Mode::Boot, "未证边不改状态");
    }

    // ---------------- tick 序列 / 装载 ----------------

    #[test]
    fn tick_sequence_runs_engine_only_after_stage_load() {
        let mut host = Host::new();
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let out = host.tick(&no_input(), &mut renderer, &mut audio);
        assert!(out.advanced && !out.gameplay_tick, "标题前不跑引擎");
        assert_eq!(host.stage_state().frame, 0);
        assert_eq!(renderer.presents, 1, "每逻辑帧一次 present");
        assert_eq!(audio.frames, vec![0], "每逻辑帧一次 end_frame");

        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        let out = host.tick(&right_only(), &mut renderer, &mut audio);
        assert!(!out.gameplay_tick, "未装载关卡 ⇒ 不跑 engine tick（fail-closed）");
        assert_eq!(host.player().x, INITIAL_PLAYER_X + 2, "player::tick 输入当帧生效（POS 整数 +2）");
        assert_eq!(host.player().x_subpix, 208, "亚像素 +208（R-move-6）");

        assert!(host.load_stg01_from_bank8(&synthetic_stg01_bank8()), "合成 43 记录视界装载成功");
        assert_eq!(host.stage_sched_records(), STG01_SCHED_RECORDS);
        let out = host.tick(&no_input(), &mut renderer, &mut audio);
        assert!(out.gameplay_tick, "装载后 Combat 跑 engine tick");
        assert_eq!(host.stage_state().frame, 1);
    }

    #[test]
    fn load_stg01_fail_closed_on_incomplete_stream() {
        let mut host = Host::new();
        assert!(!host.load_stg01_from_bank8(&incomplete_bank8()), "非 43 记录 ⇒ false");
        assert!(!host.stage_loaded(), "失败不改装载状态");
        assert!(!host.load_stg01(), "ROM 未装载 ⇒ false");
        assert!(!host.load_stg01_from_bank8(&vec![0u8; 0x100]), "短窗 ⇒ false");
    }

    #[test]
    fn opposite_buttons_are_resolved_before_core() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let both = Buttons { left: true, right: true, up: true, down: true, ..Buttons::default() };
        let out = host.tick(&FixedInput(both), &mut renderer, &mut audio);
        assert_eq!(out.buttons, Buttons::default(), "对向同持消解为 0（不触发 core panic）");
        assert_eq!(host.player().x, INITIAL_PLAYER_X, "无移动");
        assert_eq!(host.player().y, INITIAL_PLAYER_Y);
    }

    #[test]
    fn respawn_window_freezes_player_movement() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        host.apply_event(ModeEvent::DeathRespawn).expect("死亡窗已证边");
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let before = *host.player();
        let out = host.tick(&right_only(), &mut renderer, &mut audio);
        assert!(!out.gameplay_tick, "未装载 ⇒ 引擎不 tick；玩家冻结断言仍成立");
        assert_eq!(*host.player(), before, "重生窗 x/y/亚像素冻结（A 级 L3）");
    }

    #[test]
    fn stage_clear_effect_auto_advances_proven_edge() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        assert!(host.load_stg01_from_bank8(&synthetic_stg01_bank8()));
        // 已证写点注入（门禁测试）：$26 bit4 已臂 + slot0 在场（$0400 门）
        host.stage_state_mut().wave.flags_26 |= 0x10;
        host.stage_state_mut().grid.slots[0].type_ = 0x03;
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let out = host.tick(&no_input(), &mut renderer, &mut audio);
        assert!(out.stage_clear_requested, "engine 上报 $26 bit4 关末门");
        assert_eq!(out.flow_event, Some(ModeEvent::StageClear), "宿主自动投递已证边");
        assert_eq!(host.mode(), Mode::StageClear);
    }

    // ---------------- 菜单派发（输入驱动） ----------------

    /// 逐帧驱动宿主，返回进入新模式的顺序序列。
    fn run_with_presses(host: &mut Host, frames: u64, presses: &[(u64, bool, bool)]) -> Vec<Mode> {
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let mut modes = vec![host.mode()];
        for f in 0..frames {
            let (mut want_start, mut want_b) = (false, false);
            for &(pf, s, b) in presses {
                if pf == f {
                    want_start = s;
                    want_b = b;
                }
            }
            let buttons = Buttons { start: want_start, b: want_b, ..Buttons::default() };
            host.tick(&FixedInput(buttons), &mut renderer, &mut audio);
            if modes.last() != Some(&host.mode()) {
                modes.push(host.mode());
            }
        }
        modes
    }

    #[test]
    fn menu_dispatch_walks_title_menu_shop_stage_by_proven_sequence() {
        let mut host = Host::new();
        // 帧锚自动（Boot@180→Black，Black@360→Title）+ 输入：Start@600→Menu、Start@700→Shop、
        // B@800→ShopInfo、B@900→ExitTransition(+33f)→Transition→(33f)→StageStart→(240f)→Combat。
        let presses = [(600u64, true, false), (700, true, false), (800, false, true), (900, false, true)];
        let modes = run_with_presses(&mut host, 1400, &presses);
        assert_eq!(
            modes,
            vec![
                Mode::Boot,
                Mode::Black,
                Mode::Title,
                Mode::Menu,
                Mode::Shop,
                Mode::ShopInfo,
                Mode::Transition,
                Mode::StageStart,
                Mode::Combat,
            ],
            "已证菜单/模式序列（R-ui-6/7 + R-shop-6/7 + 33f/240f 常数）"
        );
        assert_eq!(host.shop_state(), game_core::ui::ShopState::Exited, "两连 B 退出（R-shop-7）");
        assert!(!host.stage_loaded(), "ROM 未装载 ⇒ 转场帧装载 fail-closed（不伪造）");
    }

    #[test]
    fn title_input_lock_blocks_start_until_551() {
        let mut host = Host::new();
        // Start@500 落在九班输入锁 (450,550] 内 ⇒ 应被忽略；解锁后 Start@600 生效。
        let presses = [(500u64, true, false), (600, true, false)];
        let modes = run_with_presses(&mut host, 700, &presses);
        assert_eq!(modes, vec![Mode::Boot, Mode::Black, Mode::Title, Mode::Menu], "锁窗内 Start 无效、解锁后生效");
    }

    #[test]
    fn start_in_shop_has_no_effect() {
        let mut host = Host::new();
        // 先到 Shop（Start@600→Menu, Start@700→Shop），再在 Shop 内按 Start（R-shop-8 无效）。
        let presses = [(600u64, true, false), (700, true, false), (800, true, false), (900, true, false)];
        let modes = run_with_presses(&mut host, 1000, &presses);
        assert_eq!(modes, vec![Mode::Boot, Mode::Black, Mode::Title, Mode::Menu, Mode::Shop], "Shop 内 Start 无效果");
    }

    // ---------------- task-29 B1 回归：跨键/短间隔吞按 ----------------

    /// 逐帧按钮脚本驱动（未列帧=无按键；每帧单次采样）。
    fn run_with_button_script(
        host: &mut Host,
        frames: u64,
        script: &[(u64, Buttons)],
    ) -> Vec<Mode> {
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let mut modes = vec![host.mode()];
        for f in 0..frames {
            let buttons = script
                .iter()
                .find(|(pf, _)| *pf == f)
                .map(|(_, b)| *b)
                .unwrap_or_default();
            host.tick(&FixedInput(buttons), &mut renderer, &mut audio);
            if modes.last() != Some(&host.mode()) {
                modes.push(host.mode());
            }
        }
        modes
    }

    #[test]
    fn menu_start_not_swallowed_by_other_keys() {
        // B1 回归：right@590 / A@595 的 12f `$18` 保持不得吞掉 START@600 的按下沿
        // （A 级层 1 为 START 专用早退门 bank00:92AA/9400 `AND #$10`）。
        let mut host = Host::new();
        let dir = Buttons { right: true, ..Buttons::default() };
        let a = Buttons { a: true, ..Buttons::default() };
        let start = Buttons { start: true, ..Buttons::default() };
        let script = [(590u64, dir), (595, a), (600, start)];
        let modes = run_with_button_script(&mut host, 700, &script);
        assert_eq!(
            modes,
            vec![Mode::Boot, Mode::Black, Mode::Title, Mode::Menu],
            "其它键 12f 保持内 START 仍须派发（修复前被吞⇒停在 Title）"
        );
    }

    #[test]
    fn shop_two_b_presses_within_12_frames_both_register() {
        // B1 回归：两连 B 间隔 4f（<12f 锁存保持）也须各自派发（Shop B 走 `$16` 原始按下沿）。
        let mut host = Host::new();
        let start = Buttons { start: true, ..Buttons::default() };
        let b = Buttons { b: true, ..Buttons::default() };
        let script = [(600u64, start), (700, start), (800, b), (804, b)];
        let modes = run_with_button_script(&mut host, 900, &script);
        assert_eq!(
            modes,
            vec![
                Mode::Boot,
                Mode::Black,
                Mode::Title,
                Mode::Menu,
                Mode::Shop,
                Mode::ShopInfo,
                Mode::Transition,
                Mode::StageStart,
            ],
            "两连 B 间隔 4f：Shop→ShopInfo→(+33f)Transition→(33f)StageStart（修复前第二 B 被吞）"
        );
        assert_eq!(host.shop_state(), game_core::ui::ShopState::Exited, "第二 B 已注册（Info→Exited）");
    }

    // ---------------- FrameView / spawn 摘要 ----------------

    #[test]
    fn frame_view_exposes_only_proven_fields() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        assert!(host.load_stg01_from_bank8(&synthetic_stg01_bank8()));
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        host.tick(&no_input(), &mut renderer, &mut audio);

        let view = host.frame_view();
        assert_eq!(view.mode, Mode::Combat);
        assert_eq!(view.stage, 1);
        assert_eq!(view.frame, host.frame());
        assert_eq!(view.player, *host.player());
        assert_eq!(view.enemy_slots.len(), 20, "槽 8..=27");
        for (i, slot) in view.enemy_slots.iter().enumerate() {
            assert_eq!(slot.slot as usize, 8 + i, "槽号顺序");
            assert_eq!(slot.position, None, "坐标写者链未证 ⇒ 显式 None");
        }
        assert!(view.enemy_slots.iter().all(|s| s.type_code.is_none()), "合成视界无 spawn 项");
        assert_eq!(view.live_bullets, 0);
        assert_eq!(view.player_bullets_live, 0);
        assert_eq!(view.hud.bomb_ammo, INITIAL_BOMB_AMMO);
        assert_eq!(view.hud.stage, 1);
        assert_eq!(view.hud.money_digits.len(), 7);
        assert_eq!(host.frame_view(), view, "只读 view 幂等");
    }

    #[test]
    fn spawn_log_records_initial_object_activation() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        let load_frame = host.frame();
        assert!(host.load_stg01_from_bank8(&synthetic_stg01_bank8()));
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        host.tick(&no_input(), &mut renderer, &mut audio);
        let log: &[SpawnEvent] = host.spawn_log();
        assert_eq!(log.len(), 1, "仅槽0 自机对象 0→1（合成视界无 spawn 项）");
        assert_eq!(log[0].frame, load_frame, "激活帧=装载后首个 engine tick 帧");
        assert_eq!((log[0].slot, log[0].type_code), (0, super::PLAYER_OBJECT_TYPE));
        host.clear_spawn_log();
        host.tick(&no_input(), &mut renderer, &mut audio);
        assert!(host.spawn_log().is_empty(), "已激活槽不重复计数（0→非0 判定式口径）");
    }

    #[test]
    fn scroll_source_uses_stage_clock_and_scroll_delta() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        assert!(host.load_stg01_from_bank8(&synthetic_stg01_bank8()));
        let load_frame = host.stage_load_frame();
        assert_eq!(host.stage_abs_frame(load_frame), super::STG01_LOAD_FRAME);
        assert_eq!(host.stage_scroll_delta(load_frame), 48, "f1193 装载 −48");
        assert_eq!(host.stage_scroll_delta(load_frame + 10), 0, "f1194-1213 静止");
        assert_eq!(host.stage_scroll_delta(load_frame + 22), 2, "f1214+ 每帧 −2");
        assert_eq!(host.stage_scroll_delta(load_frame + 125), 18, "f1318 脉冲 −18");
    }

    // ---------------- 生命周期 / 资产 / 存档 / 音频 ----------------

    #[test]
    fn fixed_step_advance_drives_logical_frames() {
        let mut host = Host::new();
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        assert_eq!(host.advance(0, &no_input(), &mut renderer, &mut audio), 0);
        assert_eq!(host.frame(), 0);
        assert_eq!(host.advance(FRAME_NS, &no_input(), &mut renderer, &mut audio), 1);
        assert_eq!(host.frame(), 1);
        assert_eq!(host.advance(FRAME_NS * 100, &no_input(), &mut renderer, &mut audio), 4);
        assert_eq!(host.frame(), 5);
    }

    #[test]
    fn suspend_resume_quit_gate_logic_frames() {
        let mut host = Host::new();
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        host.tick(&no_input(), &mut renderer, &mut audio);
        assert_eq!(host.frame(), 1);
        host.handle_app_event(AppEvent::Suspend);
        let out = host.tick(&no_input(), &mut renderer, &mut audio);
        assert!(!out.advanced && !out.gameplay_tick, "挂起不推进");
        assert_eq!(host.frame(), 1);
        assert_eq!(renderer.presents, 1, "挂起不呈现");
        host.handle_app_event(AppEvent::Resume);
        let out = host.tick(&no_input(), &mut renderer, &mut audio);
        assert!(out.advanced);
        assert_eq!(host.frame(), 2);
        host.handle_app_event(AppEvent::Quit);
        assert!(!host.tick(&no_input(), &mut renderer, &mut audio).advanced, "退出后停止");
        assert!(host.quit_requested());
    }

    #[test]
    fn poll_lifecycle_drains_platform_queue() {
        let mut host = Host::new();
        let mut lc = QueueLifecycle::default();
        lc.events.push_back(AppEvent::Suspend);
        lc.events.push_back(AppEvent::Resume);
        assert_eq!(host.poll_lifecycle(&mut lc), 2);
        assert!(!host.suspended(), "Suspend→Resume 后恢复");
    }

    #[test]
    fn asset_source_loads_rom_bytes() {
        let mut host = Host::new();
        assert!(host.rom().is_none());
        assert!(host.load_asset(&mut FixtureAssets, ROM_ASSET_PATH));
        assert_eq!(host.rom().map(|r| r.len()), Some(6), "AssetSource 字节已取得");
        assert!(!host.load_asset(&mut FixtureAssets, "rom/missing.nes"), "缺失 fail-closed");
        assert!(!host.load_stg01(), "伪 ROM（非真实 bank8）⇒ 装载 fail-closed");
    }

    #[test]
    fn storage_snapshot_roundtrip_and_fail_closed() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        for _ in 0..3 {
            host.tick(&right_only(), &mut renderer, &mut audio);
        }
        let mut storage = SlotStorage::default();
        assert!(host.save_snapshot(0, &mut storage), "save 成功");
        let mut restored = Host::new();
        assert!(restored.load_snapshot(0, &mut storage), "load 成功");
        assert_eq!(restored.mode(), host.mode());
        assert_eq!(restored.stage_index(), host.stage_index());
        assert_eq!(restored.player(), host.player());
        assert_eq!(restored.flow().data, host.flow().data);
        assert!(!restored.load_snapshot(1, &mut storage), "空槽 fail-closed");
        storage.slots[1] = Some(vec![0u8; crate::snapshot::SNAPSHOT_LEN]);
        assert!(!restored.load_snapshot(1, &mut storage), "损坏快照 fail-closed");
        assert!(!restored.save_snapshot(9, &mut storage), "越界槽拒绝");
    }

    #[test]
    fn audio_pipe_consumes_injected_stream() {
        let mut host = Host::new();
        host.audio_mut().stream_mut().write(0, 0x00, 0x30);
        host.audio_mut().stream_mut().write(1, 0x15, 0x0F);
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        host.tick(&no_input(), &mut renderer, &mut audio);
        assert_eq!(audio.writes, vec![(0, 0x0000, 0x30), (1, 0x0015, 0x0F)], "写序保真");
        assert_eq!(audio.frames, vec![0], "每帧 end_frame");
        assert_eq!(host.audio().queued(), 0);
    }

    // ---------------- task-37：玩家弹丸生成 / 运动 / 命中 ----------------

    /// 注入合成敌框形（形 0：单框 x±16/y±16；A 格式：`[count][4B 符号偏移]`）。
    fn inject_synthetic_shape(host: &mut Host) {
        let addr = game_core::enemy::shape_stream_addr(0).expect("形 0 地址");
        host.stage_state_mut().shape_data.push((addr, vec![1, 0xF0, 16, 0xF0, 16]));
    }

    /// 注入玩家弹到 **shell 自有弹网格**（寿命 1/指针 0x0C：首帧 `shot::tick` 取指后运动
    /// `LUT_X[0x0C]=0 / LUT_Y[0x0C]=-9`）。
    fn inject_bullet(host: &mut Host, slot: u8, x: u8, y: u8, type_idx: u8) {
        let g = &mut host.player_bullets;
        g.state[slot as usize] = 1;
        g.action_ptr[slot as usize] = 0x0C;
        g.x[slot as usize] = x;
        g.y[slot as usize] = y;
        g.motion[slot as usize] = 0;
        g.type_code[slot as usize] = type_idx;
    }

    /// 注入敌槽（框形 0、属性 bit4 与弹 attr 0x30 相交、HP 字段=4）。
    fn inject_enemy(host: &mut Host, slot: usize, type_: u8, x: u8, y: u8, attr: u8) {
        host.stage_state_mut().grid.slots[slot] = game_core::engine::ObjectSlot {
            type_,
            x,
            y,
            f518: 0,
            f4c4: attr,
            f630: 4,
            ..game_core::engine::ObjectSlot::default()
        };
    }

    #[test]
    fn player_fire_gate_spawns_bullets_when_b_held() {
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        assert!(host.load_stg01_from_bank8(&synthetic_stg01_bank8()));
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let b_held = Buttons { b: true, ..Buttons::default() };
        host.tick(&FixedInput(b_held), &mut renderer, &mut audio);
        assert_eq!(host.player_bullets().iter().filter(|b| b.is_some()).count(), 1, "首按帧生成 1 弹");
        // 7f 周期（默认 STG-01 `$39`=7，bank02:$94DF[0]）：再 7 帧后第 2 发
        for _ in 0..super::fire::SHOT_TIMER_RELOAD {
            host.tick(&FixedInput(b_held), &mut renderer, &mut audio);
        }
        assert_eq!(
            host.player_bullets().iter().filter(|b| b.is_some()).count(),
            2,
            "$39/$3A 计时器 7f 周期生成"
        );
        assert_eq!(host.shot_timer(), super::fire::SHOT_TIMER_RELOAD);
        assert!(
            host.stage_state().bullets.state.iter().all(|&s| s == 0),
            "玩家弹在 shell 自有网格：engine StageState.bullets 保持空（避免双步进）"
        );
        // B 松开 ⇒ 计时器回 1（下按帧即发）
        host.tick(&no_input(), &mut renderer, &mut audio);
        assert_eq!(host.shot_timer(), 1);
    }

    #[test]
    fn player_bullet_hits_enemy_slot_positive_and_negative() {
        // 正例：弹 (91,100) 首帧 tick 后 (91,91)（y-9），弹框 y86..96 与敌框 y84..116 交叠 ⇒ 命中
        let mut host = Host::new();
        host.set_auto_flow(false);
        walk_to_combat(&mut host);
        assert!(host.load_stg01_from_bank8(&synthetic_stg01_bank8()));
        inject_synthetic_shape(&mut host);
        inject_enemy(&mut host, 8, 0x2B, 100, 100, 0x10);
        inject_bullet(&mut host, 0x80, 91, 100, 0);
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let out = host.tick(&no_input(), &mut renderer, &mut audio);
        assert!(out.gameplay_tick);
        let events = host.enemy_hit_log();
        assert_eq!(events.len(), 1, "命中事件=1（shell 侧已证命中管线）");
        assert_eq!(events[0].enemy_slot, 8);
        assert_eq!(events[0].damage, 0, "非 $4A ⇒ 0 伤");
        assert!(events[0].fail_closed, "未证 HP 域 ⇒ fail-closed 登记");
        assert_eq!(host.stage_state().grid.slots[8].f630, 4, "非白名单不写 HP 字段");
        assert_eq!(host.player_bullets.state[0x80], 0, "命中弹回收（$87FD）");
        assert_eq!(host.hits_this_frame()[0], Some(events[0]));
        assert!(host.frame_view().player_bullets.iter().all(|b| b.is_none()), "弹已释放");

        // 负例 1：敌框远离 ⇒ 无命中事件、弹仍活（FrameView 暴露坐标）
        let mut host2 = Host::new();
        host2.set_auto_flow(false);
        walk_to_combat(&mut host2);
        assert!(host2.load_stg01_from_bank8(&synthetic_stg01_bank8()));
        inject_synthetic_shape(&mut host2);
        inject_enemy(&mut host2, 8, 0x2B, 220, 220, 0x10);
        inject_bullet(&mut host2, 0x80, 91, 100, 0);
        let mut r2 = NullRenderer::new();
        let mut a2 = RecAudio::default();
        host2.tick(&no_input(), &mut r2, &mut a2);
        assert!(host2.enemy_hit_log().is_empty(), "框远离 ⇒ 无命中（负例）");
        assert_ne!(host2.player_bullets.state[0x80], 0, "未命中弹不回收");
        assert_eq!(
            host2.player_bullets().iter().filter(|b| b.is_some()).count(),
            1,
            "FrameView 暴露弹坐标（弹网格 A 地址面）"
        );

        // 负例 2：属性不相交（敌 `$04C4 & 0xF0 = 0`）⇒ 无命中、弹不回收
        let mut host3 = Host::new();
        host3.set_auto_flow(false);
        walk_to_combat(&mut host3);
        assert!(host3.load_stg01_from_bank8(&synthetic_stg01_bank8()));
        inject_synthetic_shape(&mut host3);
        inject_enemy(&mut host3, 8, 0x2B, 100, 100, 0x00);
        inject_bullet(&mut host3, 0x80, 91, 100, 0);
        let mut r3 = NullRenderer::new();
        let mut a3 = RecAudio::default();
        host3.tick(&no_input(), &mut r3, &mut a3);
        assert!(host3.enemy_hit_log().is_empty(), "属性不交 ⇒ 无命中（负例）");
        assert_ne!(host3.player_bullets.state[0x80], 0, "属性不交不回收弹");
    }
}
