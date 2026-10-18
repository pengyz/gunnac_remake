# 平台抽象层 v1 方案（trait 缝补全 + 平台无关宿主 game-shell）

> 任务卡：task-5（2026-10-18 用户裁定：game-platform 是唯一 trait 缝；PC 是第一实现，Android 同缝接入）。
> 状态：**方案（实施前）**；实施完成后在 §9 回填状态（不改写本节历史主张）。
> 纪律依据：[AGENTS.md](../../AGENTS.md) §2/§3/§4/§9——未证不发明，证据 A/B 级才落结论，小步提交。

## 0. 目标与非目标

**目标（v1，一个逻辑步）**

1. 补全 `game-platform` trait 缝：`AssetSource`、`Lifecycle`/`AppEvent`、`Renderer::size`/`resize`；
   既有 `Clock`/`GameRng`/`Input`/`Audio`/`Storage` **形状不动**。
2. 新增 `crates/game-shell`：平台无关宿主（60Hz 固定步、输入消解、宿主状态机、tick 序列、
   渲染合成调用 `Renderer`、Audio 流消费），**零平台依赖**（无 winit/softbuffer/cpal/std::fs）。
3. 更新 [platform-shell.md](../spec/platform-shell.md)：分层图 + PC/Android 平台矩阵 + Android 落地清单 +
   `shell-win`＝PC 第一实现（**是否更名 shell-pc 由用户裁定，本任务不擅自改名**）。
4. 契约门禁：`game-shell` 零平台依赖机器扫描 + mock platform 驱动 300 帧、两次确定性签名一致 +
   `game-platform` 零外部依赖机器扫描。
5. 先本方案，后实现；实现只调用 `game-core` 现有公开 API，不改其语义。

**非目标（本任务不做）**

- 不做 PC shell（窗口/呈现/设备）= task-1（`shell-win`），本任务只提供它依赖的宿主层。
- 不做 Android 实现（只做落地方案与契约）。
- 不接未证行为：弹丸（H-01）、命中（H-16）、Boss/EN-9+、商店消费侧、音效/音乐合成、调色板映射、
  结局文本流等一律 fail-closed（§6 逐条声明）。
- 不改 `rom/` `reference/` `data/`；不改 `game-core`（若确需最小只读接口→先书面申请，本任务未申请）。

## 1. 层定位与动手前三问（AGENTS.md §4）

| 问 | 答 |
|---|---|
| 问题在哪一层？ | **宿主/平台层**：`game-platform` 是 trait 缝（缺 `AssetSource`/`Lifecycle`/输出面尺寸），宿主逻辑（60Hz 步进/输入策略/状态机）目前散落在 `shell-win` 库骨架里（[shell-win/src/lib.rs](../../crates/shell-win/src/lib.rs) 的 `FrameDriver`/`GameHost`/`AudioSink`），需抽成平台无关层。**不是**反汇编投影/数据表/core 语义问题。 |
| 消费者实际读的是哪个文件/数据？ | `game-shell` 消费 `game-core` 公开 API（`flow::FlowState`/`engine::stage_tick`/`player::tick`）与 `game-platform` trait；平台实现（`shell-win`/未来 `shell-android`）实现 trait 并注入。ROM 字节仍只经 `AssetSource` 取得（解析=game-data 侧，v1 不接）。 |
| 证据级别？ | 分层与 trait 形状＝**架构合同**（[AGENTS.md](../../AGENTS.md) §2 + platform-shell.md §2 已锁定面）；宿主状态机边集与 tick 顺序＝**A/B 级**（flow/stage/player 规格与 trace 锚，逐条列 §3.4/§3.5）；新增占位项显式标注"占位"，**不**据此写游戏结论。 |

## 2. trait 缝补全（`game-platform`）

原则：**已证语义最小面**；新增项逐条标注「已证 / 占位」；占位项只定契约，不承载游戏行为。
既有 6 trait 形状（方法签名）不改，仅新增独立 trait 与 `Renderer` 的输出面查询方法。

| trait / 项 | 形状（v1） | 级别 | 说明 |
|---|---|---|---|
| `AssetSource` | `fn read_all(&mut self, path: &str) -> Option<Vec<u8>>` | **占位契约** | 资产字节读取。PC=`std::fs`（shell-pc 侧实现）；Android=APK `assets/`（`AAssetManager`）。不存在→`None`（fail-closed，不提供默认内容）。解析/格式校验属 `game-data`，不在本 trait。 |
| `AppEvent` | `enum { Resume, Suspend, Quit }` | **占位最小枚举** | Android 生命周期必需；桌面实现可只发 `Quit`。不承载游戏行为。 |
| `Lifecycle` | `fn poll_event(&mut self) -> Option<AppEvent>` | **占位契约** | 平台事件队列 → 宿主（非阻塞）。宿主侧策略见 §3.8（suspend 停逻辑帧）。 |
| `Renderer::size` | `fn size(&self) -> (u32, u32)`，**默认**`(256,240)` | **占位**（默认=逻辑画布 1×） | 输出面像素尺寸（窗口/后缓冲）。默认值=「平台未独立报告」回退，不声称任何缩放语义。 |
| `Renderer::resize` | `fn resize(&mut self, w: u32, h: u32)`，**默认**空操作 | **占位** | 输出面尺寸变化通知（平台侧 allocate/resize）；默认无平台侧动作。 |
| `CANVAS_W/H/LEN` | `256/240/245760` | **已证**（platform-shell.md §2 锁定面） | 逻辑画布常量，供宿主/测试/mock 共用。 |
| `Clock`/`GameRng`/`Input`/`Audio`/`Storage` | **不动** | 已证/固定 | 形状照旧；`Clock`/`GameRng` 由 game-core 的 `FrameClock`/`LcgRng` 实现。 |

> 注：`Renderer` 用**默认方法**补两个查询，是为了不打断已存在的实现者（`game-desktop::Canvas` 在
> task-5 写范围之外）；默认值在 doc 注释中显式标"占位"，平台实现应覆盖 `size`。

## 3. `game-shell` 设计（平台无关宿主）

### 3.1 crate 与依赖

- `crates/game-shell`，`edition.workspace`，`#![forbid(unsafe_code)]`。
- 依赖仅 `game-core`、`game-platform`（**零平台依赖**；机器扫描见 §4）。
- 模块：`fixed_step`（固定步）/ `input`（按钮消解与位图）/ `host`（宿主状态机+tick 序列）/
  `render`（合成占位）/ `audio`（ApuWrite 流消费）/ `snapshot`（存档缝编解码）。

### 3.2 60Hz 固定步

- `FRAME_NS = 1_000_000_000 / 60`（16_666_666 ns；1s 漂移 40ns，可忽略）。
- `FixedStep { last_ns, accum_ns, max_frames }`：平台单调时钟推进到 `now_ns` ⇒ 返回应推进的逻辑帧数；
  追帧上限 `MAX_CATCHUP_FRAMES=4`（防 spiral of death，与 shell-win 现行 `FrameDriver` 同值）；
  时钟回拨⇒0 帧（fail-closed，不发明负帧）；超限积压丢弃（不雪崩）。
- `Host::tick(...)`＝**一个逻辑帧**（确定性核心）；`Host::advance(now_ns, ...)`＝固定步包装（0..=4 帧）。
  平台时间源（`Instant`）只在平台壳层，宿主只接收整数 ns。

### 3.3 输入策略（平台侧统一消解）

- `resolve_directions(Buttons) -> Buttons`：**L+R 同持⇒两键都置 0；U+D 同持⇒两键都置 0**。
  理由：game-core `player::velocity_from_input` 对未证组合（W6 未闭环）**panic**（fail-closed），
  平台/宿主必须在进入 core 前消解——与 [game-desktop `held_from_keys`](../../crates/game-desktop/src/lib.rs)
  同语义（该处已作为平台侧策略）。
- `held_bitmap(&Buttons) -> u8`：`$18` 位图（**A 级** R-move-1：bit0=R/bit1=L/bit2=D/bit3=U/bit6=B/bit7=A；
  START/SELECT 不入 `$18` 本图）。
- `pressed_bitmap(prev, now) -> u8`：`$16` 边沿脉冲（本帧按下且上帧未按；A/B/方向）。v1 **只产出不消费**
  ——射击/投放接线（A 边沿投放 R-fire-1..6、主武器连发 H-14/连射域）属后续步，不在此发明。
- 菜单输入门控（[engine `MenuInputGate`](../../crates/game-core/src/engine.rs) 两层模型）v1 **不接线**：
  菜单派发边集与帧相位未端到端定谳 ⇒ `ModeEvent` 由平台/测试显式投递（fail-closed）。

### 3.4 宿主状态机（title→八关→结局，接 `game-core::flow` 已证边集）

- 宿主持有 `flow::FlowState`（粗粒度 `Mode`）+ 关号 `stage_index`（`$180`，1..=8；通关后 `all_clear`）。
- 状态迁移入口 `Host::apply_event(ModeEvent) -> Result<Mode, String>`：**只**走
  [`FlowState::advance`](../../crates/game-core/src/flow.rs) 的已证边集；未证边由 core 返回 `Err`（fail-closed，
  宿主原样上抛，不发明）。
- 宿主侧附加一致性守卫（A 级关末/通关链，[l5-all-clear.md](../spec/l5-all-clear.md) R-l5-4/5）：
  - `StageAdvance`（E37C `INC $180` 后关间）仅当 `stage_index < 8`；`stage_index==8` 时该事件 → `Err`
    （ROM 语义：`$180:=9` 走 `AllClear`）；
  - `AllClear`（E392 `CMP #9`）仅当 `stage_index == 8`；否则 `Err`。
  - 成功迁移的副作用：`StageAdvance ⇒ stage_index += 1`；`AllClear ⇒ all_clear=true`。
- 已证边集（core 侧合同，逐条已在 `flow.rs` 注释带指针）：`Boot→Black→Title→Menu→Shop→Transition→
  StageStart→Combat→DeathRespawn→GameOverPending→GameOverMenu→StageClear→Ending`；关末
  `StageClear→Shop(StageAdvance)` / `StageClear→Ending(AllClear)`。v1 不增边、不改边。
- 引擎门→flow 边的自动接线（已证）：`main_loop_step` 置 `StageEffects::stage_clear_requested`
  （`$26 bit4` 门，A 级对账）且当前 `Mode::Combat` ⇒ 宿主自动 `apply_event(StageClear)`
  （`Combat→StageClear` 为已证边）。其余自动接线（死亡窗、GAME OVER 菜单输入）v1 不做。
- **驱动方（v1）**：除上述 `Combat→StageClear` 一条自动接线外，所有 `ModeEvent` 由**平台/上层显式投递**
  （宿主不做"按键→模式"自动转移，避免发明未证键序/时机）。**已证但未接线**的时序（S8）：
  关始 intro 240f（[engine.rs `RUN_STAGE_INTRO_FRAMES`](../../crates/game-core/src/engine.rs)，
  R-ui-6 锚 1260→1500）、A3 关末细段 `stage_clear_step`（50×8f 倒数+飞离+第 4 关特判）、
  A2 死亡链 `death_chain_step`、R-respawn 玩家侧复位——属后续任务卡，shell 侧不得静默硬编码
  240f/400f/265f 当作已收敛行为。

### 3.5 tick 序列（帧内次序）

每逻辑帧（`Host::tick`）：

1. 输入采样 → `resolve_directions` → `held`/`pressed` 位图；
2. 按模式门决定是否跑游戏 tick（**不发明未证段行为**）：
   - `engine::stage_tick`：`Combat`=已证 gameplay 段（R-stage-tick；R-ui-6 是 UI capture 锚、
     非 engine tick 证据）；`DeathRespawn | StageClear`=**接线约定**（v1 行为等级降级，复核 E1：
     细粒度行为未隔离——G-S5-5 死亡窗/关末门并发未取证，A2 `death_chain_step` /
     A3 `stage_clear_step` 尚未接线）；**不含 `StageStart`**（intro 段可动性未证）。
     `scroll_delta`：`Combat` 用 `16`（engine 内 `Combat` 先例：tick=scroll/16 ⇒ 1 tick/帧，A 级 Wt-2），
     其余 0（滚动活段未证 ⇒ fail-closed；W-SCLEAR 同族）。**关卡数据装载未接**（§6），v1 只推进空 StageState。
   - `player::tick`：仅 `Combat`。`DeathRespawn`/`StageClear` 不跑玩家移动——**A 级证据**：
     重生窗 x/y/亚像素全程冻结（[respawn-window.md](respawn-window.md) §3 **R-respawn-2**；
     R-respawn-3 仅为 (128,160) 复位锚），此时按输入积分会与已证冻结冲突 ⇒ 不跑。
   - `StageStart` 段是否可动**未隔离**（R-ui-6 只证画面锚）⇒ v1 不跑（fail-closed）。
3. 消费 `StageEffects`：摘要（spawn/drip/far-call 计数）进确定性签名；`stage_clear_requested` 自动接 flow（§3.4）。
4. Audio 流消费（§3.7）→ 渲染合成 + `present`（§3.6）→ 虚拟时钟 `FrameClock::tick`。
5. 输出 `FrameOutcome`（帧号/模式/关号/消解后按钮/是否跑游戏 tick/stage_clear/自动 flow 事件/quit/suspend）。

帧内次序依据：`engine::FRAME_ORDER`（[engine.rs:3-4/23-24](../../crates/game-core/src/engine.rs)；
注释记 A2 帧序=行为→碰撞→主环。**这是 A2 findings 归纳，spec 未单列该锚**），
宿主只在 core 输出后做呈现/音频，不插入游戏逻辑。

### 3.6 渲染合成（调用 `Renderer`；v1 显式占位）

- `compose_frame(renderer.canvas_mut(), &FrameView) -> canvas_sig`：清屏（RGBA 0,0,0,255）+
  **玩家坐标占位块**（8×8，坐标 `$04E0/$04FC` 为 A 级已证面；**图样/调色板=占位，非原版像素**）。
- **不画**：BG/nametable、敌精灵、弹丸、HUD、文本、结局卡、fade——全部未证或属 task-1 渲染战役；
  代码内 `TODO(platform-v1)` 登记。`renderer.present()` 每逻辑帧一次（suspend/quit 帧不呈现）。
- 实现侧（`shell-pc`/`shell-android`）负责整数倍缩放/黑边/上屏；`game-shell` 只写 256×240 RGBA 画布。

### 3.7 Audio 流消费（契约已接线，生产者 v1 未接）

- `AudioPipe { stream: game_core::ApuWriteStream, flushed_events, write_hash }`：每帧
  `drain` 流事件 → 逐事件 `Audio::write_apu(event.frame, event.reg as u16, event.value)`（写序保真）→
  `Audio::end_frame(本逻辑帧)`。
- **v1 无生产者**：APU 写侧（AUD-2/3 音频驱动体、音效/音乐触发）未证，`far_call_requests` 不映射为音频写
  （映射属猜测，禁止）；用 `Host::audio_mut()` 注入通道只作后续步接线口与契约测试。
- 合成器/PCM 输出属平台实现侧占位（shell-win 现为静音占位显式）。

### 3.8 资产 / 存档 / 生命周期端口

- **资产**：`Host::load_asset(&mut dyn AssetSource, path) -> bool`——只取得字节并存 `Option<Vec<u8>>`；
  不做 NES 头校验/解析（`game-data` 域，v1 未接）。**资产路径语义**（S3）：`ROM_ASSET_PATH` 是相对
  **资产根**的逻辑路径；PC 资产根=可执行文件所在目录（可回退 `exe/../rom/` 与 `CARGO_MANIFEST_DIR`
  开发态定位），Android=APK `assets/` 根；**不得依赖进程 cwd**。
- **存档（占位缝）**：`HostSnapshot`（flow mode/关号/all_clear/player/PersistData）+ `encode_snapshot`/
  `decode_snapshot`（magic `GNSH`/version 1/定长；损坏 fail-closed `None`）；`save_snapshot`/`load_snapshot`
  走 `Storage`。**不自动保存/不接原版存档语义**（原版无电池存档证据）；引擎 `StageState` 不落快照（见 §6）。
- **生命周期（占位策略）**：`Suspend ⇒ 暂停逻辑帧（tick 不推进、不呈现）；Resume ⇒ 恢复；Quit ⇒ 停止`
  ——Android 生命周期必需的最小策略；不产生游戏行为。

### 3.9 确定性状态签名

- `Host::signature() -> u64`（FNV-1a）：虚拟时钟帧/ms、模式 tag、关号/all_clear/suspend/quit、
  玩家全字段、PersistData、`StageState` 帧/wave tick/scroll/flags_26/游标/28 槽型码/活弹数、
  上帧 `StageEffects` 摘要、`AudioPipe` 冲流计数+写 hash、上帧画布 hash。
- 用途：mock 契约测试 300 帧两次 `signature()` 相等（门禁）；后续 L5 接真机时可作粗粒度回归锚
  （**不**声称等价于 trace 对拍）。

## 4. 契约门禁设计（机器判定）

| 门禁 | 位置 | 判据 |
|---|---|---|
| `game-platform` 零外部依赖 | `crates/game-platform/tests/dependency_contract.rs` | 解析 `Cargo.toml` 的 `[dependencies]` 段为空；源码零平台词面（照旧） |
| `game-shell` 零平台依赖 | `crates/game-shell/tests/contract.rs` | 扫 `src/`（去注释）禁词：`std::fs/net/process/time/env`、`Instant::now`、`SystemTime`、`thread_rng`、`rand::`、`getrandom`、`tokio`、`winit`、`softbuffer`、`cpal`、`sdl`、`oboe`、`aaudio`、`android`、`jni`；`Cargo.toml` 依赖仅 `game-core`/`game-platform` |
| mock 300 帧确定性 | `crates/game-shell/tests/mock_platform.rs` | mock 实现 **全部**平台 trait（Input/Renderer/Audio/Storage/AssetSource/Lifecycle），按固定输入脚本 + 固定 `ModeEvent` 调度驱动 300 帧；两次独立运行 `signature`/画布 hash/音频记录/存档字节全等；打印签名（复现命令见 §9） |
| trait 形状锁 | `crates/game-platform/tests/*` | 新增 trait mock 断言（AssetSource 往返/Lifecycle 队列/Renderer size 默认与覆盖）；既有形状锁不变 |

mock 300 帧脚本设计（确定性，不含时间/线程/哈希序）：
- 输入：8 帧循环（右/左/上/下/右+左（消解 0）/上+下（消解 0）/A/B）——同时覆盖对向消解；
- 事件调度：第 10 帧起每 4 帧投递一个 `ModeEvent`，走完
  `Boot→Black→Title→Menu→Shop→Transition→StageStart→Combat`，然后每关
  `StageClear→(StageAdvance→Shop→Transition→StageStart→Combat)`，关 8 `StageClear→AllClear→Ending`；
  共 44 事件，全部在已证边集内；断言终态 `Mode::Ending`/`stage_index==8`/`all_clear`；
- 期间调用 `load_asset`/`save_snapshot` 覆盖 AssetSource/Storage 真实消费；另设独立用例验证
  Suspend/Resume/Quit 与 Audio 流消费（注入写事件→保序+end_frame）。

## 5. 平台矩阵与 Android 落地方案

（正式版并入 [platform-shell.md](../spec/platform-shell.md) §5/§6；此节为方案草稿，实施后以规格为准。）

| 面 | PC（首实现＝`shell-win`，Win/Linux 同源） | Android（规划，同 trait 缝） |
|---|---|---|
| 窗口 | winit 0.30 + softbuffer CPU 表面 | winit 0.30 `android` 后端（`android-activity`，`android_main` 入口）＋ `ANativeWindow` |
| 呈现 | 256×240 RGBA→softbuffer 整数倍缩放 blit（黑边） | EGL/GLES2 纹理上传（`glow` 族；**softbuffer 不支持 Android** ⇒ 平台侧另实现 `Renderer`） |
| 输入 | 键盘→`Buttons`（Z/X/Enter/RShift/方向） | 触摸虚拟手柄／外接手柄→`Buttons`（**布局与连发语义未证=占位**，§6）；物理键 BACK→`Quit` |
| 音频 | 平台侧 `Audio` 实现（cpal 族，PCM=合成器未证期静音占位显式） | **AAudio**（API≥26）或 **oboe**（低版本兼容）；`Audio` 实现消费 ApuWrite 流→PCM |
| 资产 | `AssetSource`=`std::fs`（`rom/` 相对/绝对路径） | `AssetSource`=APK `assets/`（`AAssetManager`/JNI）；ROM 打包进 APK |
| 存储 | `Storage`=`%APPDATA%/gunnac`（Win）/`~/.local/share/gunnac`（Linux） | `Storage`=应用内部目录（`Context.getFilesDir()`，JNI）；进程可被杀 ⇒ 快照时机待定 |
| 生命周期 | 关窗→`Quit`；最小化可不发事件 | `winit` `Suspended`→`AppEvent::Suspend`、`Resumed`→`Resume`、destroy/back→`Quit`；宿主 suspend 停逻辑帧 |
| 编译 | 本机 `cargo test/build` 直接验证 | 交叉编译/真机 = 后续步骤（NDK + `cargo-apk`/gradle；本任务只定清单与契约） |

Android 落地清单（后续任务卡逐项验收）：
1. `winit 0.30` Android：`android-activity` 依赖 + `#[cfg(target_os="android")] android_main`；`EventLoop` 生命周期事件映射 §3.8。
2. 呈现：EGL/GLES2 上下文 + 纹理（256×240 RGBA 上传），平台侧 `Renderer::canvas_mut/present/size/resize` 实现。
3. 音频：AAudio/oboe 设备流；`Audio` trait 消费写流；PCM 合成器未证期显式静音占位。
4. 资产：ROM/数据打包 APK `assets/`；`AssetSource` 用 `AAssetManager` 读整块字节。
5. 存储：内部目录 `Storage` 实现；快照写入时机（生命周期/用户操作）待规格。
6. 生命周期：suspend 释放/恢复 GL 与音频设备（真实音频/EGL 资源策略=平台实现侧，占位）。
7. **命名**：`shell-win` 是 PC 第一实现；是否更名 `shell-pc` 需用户裁定（本任务不改名、不移动 crate）。

## 6. 未证域声明（fail-closed，v1 一律不实现/不声称）

| 域 | 状态 | v1 处理 |
|---|---|---|
| H-01 弹丸/武器槽语义（槽号↔武器名、弹幕行为） | 未证 | 不画、不推进、不建行为；TODO |
| H-16 命中判定（受击/伤害链） | 未证 | 不新增碰撞/命中逻辑；沿用 core 既有 `stage_tick` 管线（空 StageState 下 inert，不据此写命中结论） |
| Boss / EN-9+ 段（[stage.rs `boss_segment_unproven`](../../crates/game-core/src/stage.rs)） | 未证 | fail-closed，不 spawn |
| 商店消费侧 / 经济闭环 | 未证 | flow 只到 `Shop` 状态，不实现买卖 |
| 音效/音乐触发与 PCM 合成（AUD-2/3 内部体） | 未证 | 音频流消费契约接线，**生产者空**；平台侧静音占位 |
| 原版调色板映射 / BG nametable / 文本流 / 结局卡渲染 | 未证/后续战役 | 不画；只做占位合成 |
| 玩家主武器射击（H-14 连射域）与 A 边沿投放接线 | 部分已证但 t5 不做 | `$16` 边沿位图产出不消费；后续步接 |
| 菜单输入门控/派发（`MenuInputGate` 相位锁） | 机制已证、接线未定谳 | `ModeEvent` 显式投递，不自动从按键转移 |
| 除 `Combat→StageClear` 外的模式时序（关始 240f intro〔R-ui-6〕/ A3 关末细段 `stage_clear_step` / A2 死亡链 / R-respawn 玩家侧复位） | **已证未接线**（复核 S8） | v1 由平台/上层显式投递 `ModeEvent`；不得硬编码 240f/400f/265f 当已收敛行为；后续任务卡接线 |
| DeathRespawn/StageClear 内的 engine tick 细粒度行为（死亡窗滚动/主环 poll/关末门并发 G-S5-5） | **未隔离**（复核 E1） | 仅接线约定；差分对拍视未证域，不作已证断言 |
| 双状态源契约（`Host.stage_index` vs `StageState.stage_counter`〔`$180`〕；`player.bomb_ammo` vs `FlowState.data.bomb_ammo`） | 未定（复核 S2） | **接 game-data/死亡链任务卡前置条款**：须先定单一真相源与同步时机（`$180`、R-respawn-4 窗末回满 4），禁止下游自行发明 |
| 菜单域 `$16` START/SELECT 边沿（bit4/bit5） | v1 缺（复核 S5） | `pressed_bitmap` 仅方向+A/B 子集；菜单接线须独立菜单位图函数 |
| 关卡数据装载（`StageState` 的脚本/调度/shape/初始槽） | `game-data`/装载侧未接 | v1 空 `StageState`；`stage_state_mut()` 只作已证数据装载与门禁测试口 |
| 触摸虚拟手柄布局/连发/宏 | 未证 | Android 清单标注占位，不发明映射 |
| Android AAudio/oboe 选型细节、EGL 资源策略、存档时机 | 平台实现侧未证 | 方案清单只列"必做项"，不声称已实现 |
| 原版存档语义（电池存档证据） | 未证 | `Storage` 只作占位字节存取缝，不自动保存 |
| `StageStart` 段玩家可动性 | 未隔离 | 该模式不跑 `player::tick` / 0 滚动（§3.5），登记缺口 |
| 快照语义校验（stage 范围/mode×stage 组合）；`restore` 不恢复 `prev_buttons/pacer/FrameClock` | v1 仅格式 fail-closed（复核 S7） | 后续卡加固；不声称快照可任意恢复宿主全态 |
| 依赖门禁扫描面（`[dev-dependencies]`/`[build-dependencies]`/`[target.*.dependencies]`） | 仅解析 `[dependencies]`（复核 S6） | 后续加固（当前 manifest 干净） |

## 7. 风险与开放问题

1. **`Renderer` 默认方法**：为不破坏 task-5 写范围外的 `game-desktop::Canvas` 实现而用默认值；
   若裁定"必须强制实现"，需用户/Lead 批准改 `game-desktop`（本任务不改）。
2. **ratchet**：新增测试使 `rust_test_count` 高于基线（只升不降）；已由 Lead 处置（历史链
   `ff5aa7c` 280→326 / `16fb5b2` 326→338 / `fe8ce68` 338→339；task-9 再增测试后现测 355，Lead 统一上调）。
3. **`Cargo.lock`**：新增 workspace 成员必产生 lockfile 变更（生成物，非手工逻辑）。
4. **命名**：`shell-win`→`shell-pc` 由用户裁定；`game-shell` 名（平台无关宿主）本任务按卡内定名。
5. **引擎数据装载缺口**：v1 `StageState` 为空 ⇒ `stage_clear_requested` 不会自然发生；自动接线只由
   门禁测试以显式已证写点验证（`wave.flags_26|=0x10` + slot0 在场）。接 `game-data` 属后续任务卡。
6. **帧序/时序**：声音/呈现相对游戏逻辑的帧内次序未逐条取 A 级证据；v1 采用"core 输出→音频→呈现"
   并显式登记为宿主顺序（后续 L5 对拍时若冲突按 oracle 修宿主，不改 core）。

## 8. 提交计划（小步，每步跑门禁）

1. `docs(platform)`：本方案（含未证域声明与 mock 契约测试设计）。
2. `feat(platform)`：trait 缝补全（AssetSource/Lifecycle·AppEvent/Renderer size·resize/画布常量）+ 契约测试。
3. `feat(platform)`：game-shell crate 骨架（固定步/输入消解/宿主状态机 + flow 已证边集）。
4. `feat(platform)`：game-shell tick 序列/渲染合成/音频消费/资产·存档·生命周期端口。
5. `test(platform)`：Mock platform 300 帧确定性 + 零平台依赖契约扫描。
6. `docs(spec)`：platform-shell.md 分层图/平台矩阵/Android 落地清单/状态回填；本方案 §9 回填。

## 9. 实施状态（2026-10-18 回填）

**结论**：v1 交付完成（game-platform 缝补全 + game-shell 平台无关宿主 + 契约门禁 + 规格升版）。

**commit 列表（本任务写范围）**

| commit | 主题 |
|---|---|
| `0809317` | `docs(platform)`：本方案（未证域声明 + mock 契约测试设计） |
| `c43f08d` | `feat(platform)`：trait 缝补全（AssetSource/Lifecycle·AppEvent/Renderer size·resize/画布常量）+ 契约测试 |
| `98ad36c` | `feat(platform)`：新增 game-shell crate（固定步/输入消解/宿主状态机/tick 序列/渲染合成/音频/资产·存档·生命周期；含 contract/mock_platform 测试） |
| `docs(spec)`（本回填同 commit） | platform-shell.md v2 分层图/平台矩阵/Android 落地清单/状态回填 |
| `docs(platform)`（复核修正 commit） | 按 task-6 `needs_revision` 修正：E1/S8/S1/S3 + 便宜项（docs/注释/测试断言，零功能逻辑改动） |

> 偏差说明：实施把「crate 骨架」「tick/渲染/音频」「契约测试」合为 1 个 feat commit（同逻辑变更、
> 测试与实现同步落地）；其余按 §8 计划执行。

**门禁输出摘要（本机，复现见 platform-shell.md §9）**

- `cargo test -p game-platform`：trait_contract 11 + trait_shapes 2 + dependency_contract 2（共 15），全绿。
- `cargo test -p game-shell`：单元 34 + contract 2 + mock_platform 3，全绿。
- mock 300 帧两次运行全等；签名（2026-10-18 记录值）：
  `host=0xefe39a9ba5e7c5b0 canvas=0x963ac9a776a57625 advanced_frames=290 storage_len=24`
  （suspend 窗口 10 帧不推进⇒290；事件 44 条全在已证边集；终态 `Ending`/关号 8/all_clear）。
- `cargo test --workspace`：exit 0；**326 个 `#[test]`：325 passed + 1 既有 ignored**（game-core
  `engine_tick` 既有用例，非本任务跳过）。
- `python gates/cli.py check`：**CHECK: PASS**（PASS=11 FAIL=0 SKIP=1 TOLERATED=0；`rust-tests`
  326/326 OK；`docs-links` 306 md / 206 本地链接全有效）。
- 基线处置（Lead 执行，`data/status/` 非本任务写范围）：`ff5aa7c` `rust_test_count` 280→326、
  `16fb5b2` 326→338、`fe8ce68` 338→339（历史链，gates `rust-tests` 均 OK）；task-9 再增测试后
  现测 355（只升不降，由 Lead 统一上调）。

**遗留缺口（与 §6 一致，未实现/未声称）**：关卡数据装载（`StageState` 仍为空，`stage_clear_requested`
只能由门禁测试以显式已证写点触发）、音频生产侧（AUD-2/3）、玩家射击/投放接线、菜单输入派发、
死亡窗/GAME OVER 菜单自动接线、Android 实现（仅清单）、`shell-win` 主循环接线（task-1）；
**除 `Combat→StageClear` 外 ModeEvent 由平台/上层显式投递**（S8：关始 240f intro、A3 关末细段、
A2 死亡链、R-respawn 玩家侧复位=已证未接线）；**双状态源同步契约待定**（S2，接 game-data/死亡链
任务卡前置条款）；快照语义校验待加固（S7）；依赖扫描面加固（S6）。

**复核收敛（2026-10-18，task-6 `needs_revision` 修正）**：按
[adversarial-review-platform-v1.md](adversarial-review-platform-v1.md) 做 docs-only 修正——
E1（DeathRespawn/StageClear 行为等级降级为接线约定）、S8+E4（显式投递/已证未接线登记/数字口径）、
S1（`advance` 时间基准=相对 Host 创建）、S3（AssetSource 资产根定义）、E2/E3/S4/S5/S7/S9/S10、
S2/S6/P1/P2/P3 登记；另修既有陈旧 `trait_shapes.rs` 字节数注释与 `trait_contract.rs`
MockClock/MockInput 未构造断言。**实现逻辑零改动**（仅注释/文档/测试断言）。

> **R3 scope 留痕（task-23 复审残留收口）**：`b5e68b6` 为落实上述复核修正，除任务卡所列
> `crates/game-shell/src/host.rs` 外，另行修改了**同 crate** 的 `src/input.rs`/`src/snapshot.rs`
> **注释级**内容（spawn 修正在同一 crate 内、无功能逻辑改动），非写范围越界。

**风险**：`Renderer::size/resize` 采用默认方法是为不打断写范围外的 `game-desktop::Canvas`；
若后续裁定"必须强制实现"，需改 `game-desktop`（需用户/Lead 批准，非本任务写范围）。
