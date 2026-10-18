//! game-shell —— 平台无关宿主层（platform-shell.md §1/§3）。
//!
//! 分层：`game-core`（纯逻辑）← 本层调用公开 API/trait ← 平台实现注入
//! （`shell-pc`/`shell-android` 实现 `game-platform` trait）。本 crate **零平台依赖**
//! （`tests/contract.rs` 机器把关闭词扫描），不含任何窗口/音频设备/文件系统代码。
//!
//! 模块职责：
//! - [`fixed_step`]：60Hz 固定步折算（平台只注入整数纳秒时间）；
//! - [`input`]：按钮消解（未证组合 L+R/U+D⇒0）、`$18`/`$16` 位图与菜单域按下沿位图；
//! - [`host`]：宿主状态机（接 `game-core::flow` 已证边集）+ 菜单派发 + STG-01 装载 +
//!   玩家弹丸生成/运动/命中接线 + 帧内 tick 序列 + 确定性签名；
//! - [`fire`]：玩家主武器（B 长按门控 + `$39/$3A` 计时器 + 六型生成器 + 弹预步 + 命中事件观测）；
//! - [`shape`]：敌框形流装载（bank2 `$8000` 窗 → `StageState.shape_data`，engine 命中管线输入）；
//! - [`render`]：渲染合成占位（调用 `Renderer` 256×240 RGBA 画布）+ 只读 `FrameView`；
//! - [`audio`]：`ApuWriteStream` → `Audio` 流消费（v1 生产者未接）；
//! - [`snapshot`]：`Storage` 存档缝占位编解码（不自动保存）。
//!
//! 未证域（H-01 弹丸行为/H-16 精确盒残项/伤害值简化/Boss/商店消费侧/音效合成/调色板/
//! 敌坐标写者链…）一律不实现或显式登记；详见 `docs/findings/plan-bullets-hit.md` §6。

#![forbid(unsafe_code)]

pub mod audio;
pub mod enemy_coords;
pub mod fire;
pub mod fixed_step;
pub mod host;
pub mod input;
pub mod render;
pub mod shape;
pub mod snapshot;

pub use audio::AudioPipe;
pub use enemy_coords::{
    init_proven_coords, has_proven_coords, ENEMY_FAIL_CLOSED_COORDS, ENEMY_LEADER_POSITION,
    ENEMY_LEADER_TYPE,
};
pub use fire::{
    BulletView, EnemyHitEvent, MAX_ACTIVE_TYPES, PLAYER_SLOT_ORDER, SHOT_TIMER_RELOAD,
};
pub use fixed_step::{FixedStep, FRAME_NS, MAX_CATCHUP_FRAMES};
pub use host::{
    FrameOutcome, Host, HostPersistData, SpawnEvent, StageFxSummary, INITIAL_BOMB_AMMO,
    INITIAL_PLAYER_X, INITIAL_PLAYER_Y, MENU_INPUT_UNLOCK_FRAME, PLAYER_OBJECT_TYPE,
    ROM_ASSET_PATH, SCROLL_DELTA_PX_PER_FRAME, SHOP_EXIT_RESPONSE_FRAMES, STAGE_INTRO_FRAMES_HOST,
    STG01_LOAD_FRAME, STG01_PRG_BANK, STG01_SCHED_RECORDS, STG01_SCHED_STREAM_ADDR,
    TITLE_BLACK_FRAME, TITLE_FRAME, TRANSITION_TAIL_FRAMES_HOST,
};
pub use input::{
    held_bitmap, menu_pressed_bitmap, pressed_bitmap, resolve_directions, MENU_A, MENU_B,
    MENU_DOWN, MENU_LEFT, MENU_RIGHT, MENU_SELECT, MENU_START, MENU_UP,
};
pub use render::{
    canvas_hash, compose_frame, EnemySlotView, FrameView, HudView, PLACEHOLDER_PLAYER_RGBA,
};
pub use shape::{load_shape_data_from_bank2, SHAPE_BANK, SHAPE_COUNT_MAX, SHAPE_WINDOW_BASE};
pub use snapshot::{
    decode_snapshot, encode_snapshot, HostSnapshot, SNAPSHOT_LEN, SNAPSHOT_MAGIC, SNAPSHOT_VERSION,
};
