//! 平台无关游戏宿主（60Hz 固定步驱动 + 宿主状态机 + tick 序列 + 渲染/音频/资产/存档/生命周期接线）。
//!
//! 分层合同（platform-shell.md §1）：`game-core`（纯逻辑）← 调用/依赖 ← 本层 ← 注入 ← 平台实现
//! （`shell-pc`/`shell-android` 实现 `game-platform` trait 后注入）。本 crate **零平台依赖**
//! （机器扫描见 `tests/contract.rs`）。
//!
//! 纪律（未证不发明）：
//! - 状态迁移只走 `game_core::flow` 的**已证边集**；未证边由 core `Err` + 宿主关号守卫 fail-closed；
//! - tick 序列只在已证 gameplay 模式（`Combat`/`DeathRespawn`/`StageClear`）内跑引擎，
//!   玩家移动只在 `Combat` 跑（重生窗冻结=A 级，见 `respawn-window.md`）；
//! - 渲染=占位合成（见 `render.rs`）；音频=流消费契约（生产者 v1 未接）；未证域见方案 §6。

use game_core::engine::{self, StageEffects, StageState, TickInputs};
use game_core::flow::{FlowState, Mode, ModeEvent};
use game_core::player::{self, PlayerState};
use game_core::FrameClock;
use game_platform::{
    AppEvent, AssetSource, Audio, Buttons, Clock, Input, Lifecycle, Renderer, Storage,
};

use crate::audio::AudioPipe;
use crate::fixed_step::FixedStep;
use crate::input::{held_bitmap, pressed_bitmap, resolve_directions};
use crate::render::{compose_frame, FrameView};
use crate::snapshot::{decode_snapshot, encode_snapshot, mode_tag, HostSnapshot};

/// 玩家初始 X（`$04E0`）：已证重生复位锚点 (128,160)（`respawn-window.md` R-respawn-3，A 级）。
///
/// 开机/关首初值是否与重生复位同值未单独取证 ⇒ v1 **采用已证锚点值**（不发明新数值）。
pub const INITIAL_PLAYER_X: u8 = 128;
/// 玩家初始 Y（`$04FC`）：同 [`INITIAL_PLAYER_X`]。
pub const INITIAL_PLAYER_Y: u8 = 160;
/// 炸弹弹药初值（`$018F:=4`，bank00:9C5D，A 级）。
pub const INITIAL_BOMB_AMMO: u8 = 4;
/// `Combat` 段滚动增量（px/帧）：engine 侧 `tick=scroll/16` ⇒ 1 tick/帧（Wt-2 A 级）。
///
/// 逐帧滚动增量真值来自 stage-timeline；v1 未接关卡数据装载 ⇒ 固定 16 为**接线约定**
/// （与 `engine::ClearRun` 的 `Combat` 分支同值），不声称是逐帧真值。
pub const SCROLL_DELTA_PX_PER_FRAME: u16 = 16;

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
    /// `$16` 边沿位图（v1 只产出不消费）。
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
}

impl Host {
    /// 新建宿主（开机态：`Mode::Boot`、关号 1；玩家初值见 [`INITIAL_PLAYER_X`]）。
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

    // ================= 宿主状态机（接 flow 已证边集） =================

    /// 投递一个 flow 模式事件（只走 `game_core::flow` 已证边集 + 宿主关号守卫）。
    ///
    /// 守卫（A 级关末/通关链，`l5-all-clear.md` R-l5-4/5）：
    /// - `StageAdvance`（E37C `INC $180` 后关间）仅在 `stage_index<8` 合法；
    ///   `stage_index==8` 时 ROM 语义是 `$180:=9` → `AllClear`，故 fail-closed；
    /// - `AllClear`（E392 `CMP #9`）仅在 `stage_index==8` 合法。
    ///
    /// 成功时同步关号/通关标志；失败返回 core 的 fail-closed 原因（宿主状态不变）。
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

    // ================= 帧推进 =================

    /// 60Hz 固定步包装：把平台单调时钟推进到 `now_ns`，执行折算出的 0..=4 个逻辑帧。
    /// 返回执行的逻辑帧数。
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

    /// 一个逻辑帧（确定性核心）：输入采样→（按模式门）玩家/引擎 tick→音频流消费→
    /// 渲染合成+present→虚拟时钟推进。suspend/quit 时**不推进、不呈现**。
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

        let held = held_bitmap(&buttons);
        let mode_before = self.flow.mode;
        let mut gameplay_tick = false;
        let mut stage_clear_requested = false;
        let mut flow_event = None;

        if engine_tick_active(mode_before) {
            gameplay_tick = true;
            if player_tick_active(mode_before) {
                self.player = player::tick(&self.player, held);
            }
            let scroll_delta = if mode_before == Mode::Combat { SCROLL_DELTA_PX_PER_FRAME } else { 0 };
            let fx = engine::stage_tick(&mut self.stage_state, &TickInputs { scroll_delta });
            self.last_fx = StageFxSummary::from_effects(&fx);
            stage_clear_requested = fx.stage_clear_requested;
            // 已证自动接线：Combat→StageClear（flow 边集 + engine `$26 bit4` 门）。
            if mode_before == Mode::Combat && stage_clear_requested {
                if self.apply_event(ModeEvent::StageClear).is_ok() {
                    flow_event = Some(ModeEvent::StageClear);
                }
            }
        }

        // Audio 流消费（每帧恰一次 end_frame；v1 生产者空）。
        self.audio.flush(audio, frame as u32);

        // 渲染合成 + present（v1 占位：清屏 + 已证坐标面玩家块）。
        let view = FrameView {
            frame,
            mode: self.flow.mode,
            stage: self.stage_index,
            player: self.player,
            live_bullets: self.live_bullets(),
        };
        self.last_canvas_sig = compose_frame(renderer.canvas_mut(), &view);
        renderer.present();

        self.prev_buttons = buttons;
        self.clock.tick();

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

    fn live_bullets(&self) -> u16 {
        self.stage_state.bullets.state.iter().filter(|&&s| s != 0).count() as u16
    }

    /// 确定性状态签名（FNV-1a 64）：覆盖宿主可观测状态 + 上一帧画布/音频摘要。
    ///
    /// 用途：mock 契约测试「300 帧两次一致」；**不声称**等价于真机 trace 对拍。
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
        for slot in &self.stage_state.grid.slots {
            hash_bytes(&mut h, &[slot.type_]);
        }
        hash_u64(&mut h, u64::from(self.live_bullets()));
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

/// ROM 资产逻辑路径（PC=仓库工作目录相对路径；Android=APK assets 内同相对路径）。
pub const ROM_ASSET_PATH: &str = "rom/gunnac_cn.nes";

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}

/// 引擎 tick 活跃模式（已证 gameplay 段：R-ui-6/7 stage-start/combat、A2 respawn、A3 stage-clear）。
fn engine_tick_active(mode: Mode) -> bool {
    matches!(mode, Mode::Combat | Mode::DeathRespawn | Mode::StageClear)
}

/// 玩家移动活跃模式：仅 `Combat`。
///
/// `DeathRespawn` 窗内 x/y/亚像素冻结=A 级（`respawn-window.md` 逐帧向量）；
/// `StageClear` 飞离段由引擎状态向量驱动，输入不参与（未证不发明）。
fn player_tick_active(mode: Mode) -> bool {
    matches!(mode, Mode::Combat)
}

/// 宿主最小快照的持久化数据（re-export，便于平台侧做存档 UI）。
pub use game_core::flow::PersistData as HostPersistData;

#[cfg(test)]
mod tests {
    use super::{
        Host, INITIAL_BOMB_AMMO, INITIAL_PLAYER_X, INITIAL_PLAYER_Y, ROM_ASSET_PATH,
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
        assert_eq!((a.player().x, a.player().y), (INITIAL_PLAYER_X, INITIAL_PLAYER_Y), "已证复位锚点 (128,160)");
        assert_eq!(a.player().bomb_ammo, INITIAL_BOMB_AMMO, "bank00:9C5D $018F:=4");
        assert_eq!(a.frame(), 0);
    }

    #[test]
    fn walks_title_to_eight_stages_to_ending_via_proven_edges_only() {
        let mut host = Host::new();
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
        assert!(host.apply_event(ModeEvent::AllClear).is_err(), "非关 8 AllClear fail-closed");
        walk_to_combat(&mut host);
        host.apply_event(ModeEvent::StageClear).expect("关末门");
        assert!(host.apply_event(ModeEvent::AllClear).is_err(), "关 1 AllClear fail-closed");
        host.apply_event(ModeEvent::StageAdvance).expect("关 1 关间");
        // 走到关 8 关末
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
        // Boot + ModeEvent::MidCombat 不在已证边集（fail-closed）
        assert!(host.apply_event(ModeEvent::MidCombat).is_err());
        assert_eq!(host.mode(), Mode::Boot, "未证边不改状态");
    }

    // ---------------- tick 序列 ----------------

    #[test]
    fn tick_sequence_runs_engine_only_in_gameplay_modes() {
        let mut host = Host::new();
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let out = host.tick(&no_input(), &mut renderer, &mut audio);
        assert!(out.advanced && !out.gameplay_tick, "标题前不跑引擎");
        assert_eq!(host.stage_state().frame, 0);
        assert_eq!(renderer.presents, 1, "每逻辑帧一次 present");
        assert_eq!(audio.frames, vec![0], "每逻辑帧一次 end_frame");

        walk_to_combat(&mut host);
        let out = host.tick(&right_only(), &mut renderer, &mut audio);
        assert!(out.gameplay_tick, "Combat 跑 stage_tick");
        assert_eq!(host.stage_state().frame, 1);
        assert_eq!(host.player().x, INITIAL_PLAYER_X + 2, "player::tick 输入当帧生效（POS 整数 +2）");
        assert_eq!(host.player().x_subpix, 208, "亚像素 +208（R-move-6）");
        assert_eq!(audio.frames, vec![0, 1], "帧号随逻辑时钟");
    }

    #[test]
    fn opposite_buttons_are_resolved_before_core() {
        let mut host = Host::new();
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
        walk_to_combat(&mut host);
        host.apply_event(ModeEvent::DeathRespawn).expect("死亡窗已证边");
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        let before = *host.player();
        let out = host.tick(&right_only(), &mut renderer, &mut audio);
        assert!(out.gameplay_tick, "死亡窗仍跑引擎");
        assert_eq!(*host.player(), before, "重生窗 x/y/亚像素冻结（A 级 L3）");
        assert_eq!(host.stage_state().frame, 1);
    }

    #[test]
    fn stage_clear_effect_auto_advances_proven_edge() {
        let mut host = Host::new();
        walk_to_combat(&mut host);
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

    #[test]
    fn fixed_step_advance_drives_logical_frames() {
        let mut host = Host::new();
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        assert_eq!(host.advance(0, &no_input(), &mut renderer, &mut audio), 0);
        assert_eq!(host.frame(), 0);
        assert_eq!(host.advance(FRAME_NS, &no_input(), &mut renderer, &mut audio), 1);
        assert_eq!(host.frame(), 1);
        // 一大步⇒追帧上限 4
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

    // ---------------- 资产 / 存档 ----------------

    #[test]
    fn asset_source_loads_rom_bytes() {
        let mut host = Host::new();
        assert!(host.rom().is_none());
        assert!(host.load_asset(&mut FixtureAssets, ROM_ASSET_PATH));
        assert_eq!(host.rom().map(|r| r.len()), Some(6), "AssetSource 字节已取得");
        assert!(!host.load_asset(&mut FixtureAssets, "rom/missing.nes"), "缺失 fail-closed");
    }

    #[test]
    fn storage_snapshot_roundtrip_and_fail_closed() {
        let mut host = Host::new();
        walk_to_combat(&mut host);
        let mut renderer = NullRenderer::new();
        let mut audio = RecAudio::default();
        for _ in 0..3 {
            host.tick(&right_only(), &mut renderer, &mut audio);
        }
        let mut storage = SlotStorage::default();
        assert!(host.save_snapshot(0, &mut storage), "save 成功");
        assert!(storage.slots[0].is_some());

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
}
