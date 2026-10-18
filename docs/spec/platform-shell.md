# 平台抽象层 + Shell 架构规格（2026-10-18 v2）

> 用户裁定（2026-10-18）：多平台支持（Windows/安卓/其他）——game-platform trait 缝=唯一对接面；
> PC 是第一实现（现名 `shell/win`；**是否更名 `shell-pc` 由用户裁定，本规格不擅自改名**）。
> 本规格=**接口锁定依据**，接已证语义不发明；v2 由 task-5（平台抽象层 v1）升版。
>
> 实施状态（2026-10-18，task-1 回填）：`game-platform` v1 缝补全 ✅；`game-shell` 平台无关宿主 v1 ✅；
> `shell/win` v1 PC 宿主 ✅（winit/softbuffer 窗口+呈现、六 trait PC 实现、`--selftest-frames` 自检；
> 见 §4.1）；`shell/android` 仅方案（§6），未实现。

## 1. 分层合同（AGENTS.md §2 延伸）

```
                    game-core（纯逻辑：零 I/O、零平台依赖；确定性虚拟时钟 + GameRng）
                      │  依赖 trait 契约（Clock / GameRng / Input）
                      ▼
game-platform（trait 缝：零平台代码、零外部依赖 —— 机器门禁：crates/game-platform/tests/dependency_contract.rs）
      ▲ 实现 trait                                   ▲ 依赖
      │                                              │
      │                                    game-shell（平台无关宿主：60Hz 固定步/输入消解/
      │                                     宿主状态机/tick 序列/渲染合成/音频流消费；
      │                                     只认 game-platform trait；零平台依赖 —— 机器门禁：
      │                                     crates/game-shell/tests/contract.rs）
      │                                              ▲ 注入平台实现
      │                                              │
shell-pc（首实现，现名 shell/win；Win/Linux 同源）    shell-android（规划）
      └──── 实现 Renderer/Audio/Storage/AssetSource/Input/Lifecycle，驱动 game-shell ────┘
```

**层职责（单一方向依赖，禁止逆流）**

| 层 | 职责 | 红线 |
|---|---|---|
| `game-core` | 游戏逻辑/状态机/tick；时间走 `Clock`、随机走 `GameRng` | 无 I/O、无平台依赖、确定性（`tests/contract.rs`） |
| `game-platform` | trait 缝（平台能力契约） | 零外部依赖、零平台代码；新增项逐条标注已证/占位 |
| `game-shell` | 平台无关宿主：固定步、输入策略、状态机、tick 序列、渲染合成调用、音频流消费 | 零平台依赖（无 winit/softbuffer/cpal/std::fs）；读资产只走 `AssetSource` |
| `shell-pc` / `shell-android` | 平台实现：窗口/呈现/音频设备/资产/存储/生命周期 | 平台依赖**只允许**出现在这些 crate |

## 2. trait 锁定（v1 补全后；未证项显式占位，不发明）

| trait / 项 | 语义（已证面） | 状态 |
|---|---|---|
| `Clock` | 虚拟时钟 `now_ms`（确定性 60Hz，`FrameClock` 实现） | ✅ 已立（形状不动） |
| `GameRng` | 确定性 RNG 契约（`LcgRng` 占位；原版 RNG 待 S4） | ✅ 已立（形状不动） |
| `Input`/`Buttons` | 8 键手柄位（A/B/START/SELECT/↑↓←→）；连发/宏=平台侧策略 | ✅ 已立（形状不动） |
| `Renderer` | `canvas_mut`（256×240 RGBA）+ `present` + **`size`/`resize`（v1 新增）** | 🔒 锁定；`size`/`resize` 有占位默认（见下） |
| `Audio` | ApuWrite 流消费（(frame,reg,value) 三元组）→ 混音输出；**PCM 合成与设备输出=平台实现侧**（trait 仅承载写流消费；合成器未证期静音占位显式） | ✅ 已立（形状不动） |
| `Storage` | 字节块存取（save/load；存档语义未证=纯字节存取） | ✅ 已立（形状不动） |
| **`AssetSource`** | `read_all(path)->Option<Vec<u8>>`：资产字节读取（PC=std::fs/Android=APK assets 由平台实现） | 🆕 v1 占位契约 |
| **`Lifecycle`/`AppEvent`** | `poll_event()->Option<AppEvent>`；`AppEvent{Resume,Suspend,Quit}` | 🆕 v1 占位最小枚举 |
| `CANVAS_W/H/LEN` | 256/240/245760（256×240 RGBA） | ✅ 已证常量 |

**锁定原则**：trait 形状=已证语义最小面（Canvas 像素/ApuWrite 流/Buttons/字节块）；绘制细节/混音算法=
实现侧语义（shell 自有），trait 不承载未证行为。

**v1 新增项的占位语义（不得据此写游戏结论）**

- `Renderer::size` 默认 `(256,240)`＝「平台未独立报告输出面」回退，不声称缩放/黑边语义；
  `Renderer::resize` 默认空操作（CPU 帧缓冲实现可不响应）。平台实现应覆盖 `size`。
- `AssetSource` 只定**字节读取**；NES 头校验/CHR 解码/数据表解析属 `game-data`，缺失⇒`None`（fail-closed，
  不得返回默认内容）。
- `AppEvent`/`Lifecycle` 只表达宿主门控；Android 生命周期映射属平台实现侧（§5/§6）。

## 3. game-shell 架构（平台无关宿主；`crates/game-shell`）

| 模块 | 设计 |
|---|---|
| `fixed_step` | 60Hz 固定步：`FRAME_NS=1e9/60`；平台注入整数纳秒；追帧上限 4；时钟回拨⇒0 帧（fail-closed） |
| `input` | **对向同持消解**（L+R/U+D⇒两键 0；进入 core 前完成，core 对未证组合 panic=W6）；`$18` 位图（R-move-1 A 级 bit0/1/2/3/6/7）+ `$16` 边沿位图（v1 **仅方向+A/B 子集**，不含 START/SELECT；只产出不消费） |
| `host` | 宿主状态机（`FlowState`+关号 `stage_index`）只走 game-core flow **已证边集**；关号守卫（E37C/E392）；tick 序列（见下）；`signature()` 确定性状态签名；AssetSource/Storage/Lifecycle 端口 |
| `render` | 渲染合成：清屏 + 已证坐标面玩家 8×8 占位块（`$04E0/$04FC`）；BG/敌/弹/HUD/文本/结局卡**不画**（未证） |
| `audio` | `ApuWriteStream`→`Audio` 保序 `write_apu` + 每逻辑帧 `end_frame`；**v1 生产者未接**（音频驱动内部体未证） |
| `snapshot` | Storage 存档缝占位：`GNSH` v1 定长快照（不自动保存；损坏 fail-closed） |

**帧内 tick 序列（每逻辑帧）**

1. 输入采样 → `resolve_directions` → `held`/`pressed` 位图；
2. 模式门：`engine::stage_tick`：`Combat`=已证 gameplay 段（R-stage-tick；R-ui-6 是 UI capture 锚）；
   `DeathRespawn|StageClear`=**接线约定**（v1 行为等级降级：细粒度行为未隔离，G-S5-5 死亡窗/关末门
   并发未取证；A2 `death_chain_step`/A3 `stage_clear_step` 未接线），且**不含 `StageStart`**
   （intro 可动性未证）。`player::tick` 仅 `Combat`（重生窗 x/y/亚像素冻结=A 级，R-respawn-2）；
   `scroll_delta`：Combat=16（tick=scroll/16，Wt-2 A 级；未接关卡数据时=接线约定），其余 0；
3. 消费 `StageEffects`：`stage_clear_requested`（`$26 bit4` 门）在 `Combat` ⇒ 自动投递已证边
   `Combat→StageClear`；其余效果只登记摘要（不发明消费语义）；
4. Audio 流消费 → 渲染合成 + `present` → `FrameClock::tick`；
5. suspend/quit 帧不推进、不呈现（生命周期门控）。

**宿主状态机边集**：`Boot→Black→Title→Menu→Shop→Transition→StageStart→Combat→DeathRespawn→
GameOverPending→GameOverMenu→StageClear→Ending` + 关末 `StageAdvance(→Shop)` / `AllClear(→Ending)`；
未证边 fail-closed（core `Err` 上抛），宿主关号守卫：`StageAdvance` 仅关号<8、`AllClear` 仅关号==8。
边集证据指针见 [game-core flow.rs](../../crates/game-core/src/flow.rs) 注释（R-ui-6/7、A2 F6-F12、
A3 R-l5-3..6）。

**驱动方（v1）**：除 `Combat→StageClear` 一条自动接线（`$26 bit4` 门）外，**所有 `ModeEvent` 由平台/上层
显式投递**（宿主不做按键→模式自动转移，避免发明未证键序/时机）。**已证但未接线**的时序（S8）：
关始 intro 240f（`engine.rs RUN_STAGE_INTRO_FRAMES`，R-ui-6 锚 1260→1500）、A3 关末细段
`stage_clear_step`（50×8f 倒数+飞离+第 4 关特判）、A2 死亡链 `death_chain_step`、R-respawn 玩家侧复位
——属后续任务卡；shell 侧不得静默硬编码 240f/400f/265f 当作已收敛行为。

## 4. shell-pc 架构（首实现，现名 `shell/win`）

| 件 | 设计 |
|---|---|
| 窗口 | winit 0.30（跨平台同源）+ softbuffer CPU 帧缓冲（game-desktop 同族）；Win/Linux 同源编译 |
| 帧驱动 | 平台单调时钟→`game_shell::Host::advance(now_ns)`（60Hz 固定步）→ 渲染 → present。**时间基准（S1）**：`now_ns` 必须**相对 `Host` 创建时刻**（首帧=0，平台侧用 `Instant::now() - instant_at_host_creation` 折算）；禁止传平台绝对 uptime ns（否则首帧被当成积压、受追帧上限 4 截断而启动偏移）。`advance` 返回**折算帧数**；suspend/quit 帧不推进逻辑，实际推进帧数=前后 `Host::frame()` 之差 |
| 帧呈现 | 256×240 RGBA → blit 整数倍缩放（保比例黑边——game-desktop blit_scaled 语义） |
| 输入 | 键盘事件 → `Buttons`（Z/X/Enter/RShift/方向键）；对向同持由 game-shell 统一消解 |
| 音频 | 平台 `Audio` 实现（cpal 族，PCM=合成器未证期静音占位显式）；消费 ApuWrite 流 |
| 资产 | `AssetSource`=`std::fs`；**资产根=可执行文件所在目录**（`exe/rom/gunnac_cn.nes`；可回退 `exe/../rom/` 与 `CARGO_MANIFEST_DIR` 开发态），**不得依赖进程 cwd**（S3）；逻辑路径 `ROM_ASSET_PATH` 跨平台一致 |
| 存储 | `Storage`：Windows=`%APPDATA%/gunnac/`；Linux=`~/.local/share/gunnac/` |
| 生命周期 | 关窗→`AppEvent::Quit`；最小化/焦点可选发 Suspend/Resume（桌面可不发） |
| 实现状态 | **v1 已落地**（task-1，2026-10-18）：六 trait PC 实现 + winit/softbuffer 主循环 + 确定性自检；逐件状态见 §4.1 |

> 命名：本 crate 当前名 `shell-win`（=PC 第一实现，Win/Linux 同源）。**是否更名 `shell-pc` 由用户裁定**；
> 本规格与任务不改名、不移动 crate。

### 4.1 实现状态（2026-10-18，task-1 回填）

| 件 | 实现（文件） | 验证 |
|---|---|---|
| 窗口 | `WindowRenderer`（`crates/shell-win/src/main.rs`）：winit 0.30 `ApplicationHandler` + softbuffer `Surface`；标题 `gunnac-remake (Stage A)`；初始 512×480（2×） | 本机实跑窗口 `MainWindowTitle`=`gunnac-remake (Stage A)`；标题常量单测 |
| 帧驱动 | `about_to_wait`：`Instant`→整数 ns→`game_shell::Host::advance`（`FixedStep` 60Hz、追帧上限 4）；`ControlFlow::WaitUntil` 节流 | `--selftest-frames 300`：advanced=300（每采样 1 逻辑帧） |
| 呈现 | `FrameRenderer` 合成 + `blit_scaled` 整数倍缩放/黑边（softbuffer `0x00RRGGBB`） | `view_rect`/`blit_scaled` 单测（2×/3×/黑边/小窗裁切） |
| 输入 | `KeyboardInput`+`key_name`：winit 物理键码→`Buttons`（Z/X/Enter/RShift/方向）；对向同持交 game-shell `resolve_directions` 消解 | 键位表单测 + 按下/抬起状态单测 |
| 音频 | `PcAudio`（`Audio` impl）：真实消费 game-shell `ApuWrite` 流（写序保真登记，计数不丢）；PCM=静音占位显式（`silent()=true`） | 单测：写流保序/计数/`end_frame`/零 PCM；自检注入 2 条写流被消费（audio_writes=2） |
| 资产 | `FsAssets`（`AssetSource` impl）：`std::fs` 相对路径读 `rom/gunnac_cn.nes`；绝对路径/`..`/缺失⇒`None`（fail-closed）。**v1=相对 cwd 定位**（S3 复核项：task-1 后续加固为 exe 目录基准，见 §4） | 单测 + 自检 rom_len=262160 |
| 存储 | `FileStorage`（`Storage` impl）：`%APPDATA%/gunnac`（Win）/`~/.local/share/gunnac`（Linux），`slot{n}.sav` | 单测：往返/空槽 fail-closed；**不自动保存**（原版存档语义未证） |
| 生命周期 | `LifecycleQueue`：关窗→`Quit`；焦点丢失/获得→`Suspend`/`Resume`（最小平台策略）；宿主 suspend 停逻辑帧 | FIFO 单测；game-shell 门控测试 |
| 渲染内容 | CHR 背景（`ChrBackground`：`game-data::chr` 解码字模区 tile 平铺，灰阶 `GREY4` **C 级占位**）+ 玩家 8×8（game-shell 已证坐标面 `$04E0/$04FC`）；敌/弹/HUD/文本**显式不画** | 单测：短 ROM fail-closed、tile 解码、画布覆盖/清屏色保留；合成 hash 确定性 |
| 自检 | `cargo run -p shell-win -- --selftest-frames N`：无窗口、真实 `AssetSource`、固定已证 `ModeEvent`/按钮脚本 | 300 帧连跑两次 stdout 逐行一致、退出码 0（§4.2） |

**迁移**：旧骨架的 `FrameDriver`/`GameHost`（重复宿主逻辑）已删除，改接 `game_shell::FixedStep`/
`Host`；`AudioSink` 改为 `PcAudio`（平台 `Audio` 实现，不是第二套宿主）。实现方案与未证域声明见
[plan-shellwin-v1.md](../findings/plan-shellwin-v1.md)。
**节流口径（P3）**：旧 `FRAME_MS=16` 仅保留为历史占位常量（呈现/文档参考），主循环节流一律
`game_shell::FixedStep`（ns 固定步）；**不得同时保留两套节流**。

**§4.2 自检记录值（2026-10-18 本机 Windows；连跑两次逐行一致）**

```text
selftest: frames=300 advanced=300 presents=300
selftest: mode=Combat stage=1 frame=300
selftest: host_signature=0xcde9dc99514b5f4b
selftest: canvas_signature=0x963ac9a776a57625
selftest: display_signature=0xf68f9ffc0607cbad
selftest: audio_writes=2 audio_frames=300
selftest: rom_len=Some(262160)
selftest: OK
```

**口径**：`host_signature`/`canvas_signature`/`display_signature` 是**确定性回归锚**，
**不声称**等价真机 trace 对拍（platform-shell.md §3/§9 同口径）；`display_signature` 含平台侧
CHR 背景占位合成，非 L4 像素目标。GUI 视觉（缩放/黑边/CHR 背景）为手动验收项。

**未证/未接（与 §8 一致）**：菜单输入门控不自动派发（主循环不投递 `ModeEvent`，窗口初始停在
`Boot`，玩家块按已证坐标显示）、敌/弹/HUD/文本不画、音频 PCM 静音占位、关卡数据未装载、
`shell-win→shell-pc` 更名待用户裁定。

## 5. 平台矩阵（PC / Android 对照）

| 面 | PC（`shell/win`，首实现） | Android（`shell/android`，规划） |
|---|---|---|
| 窗口 | winit 0.30 + softbuffer CPU 表面 | winit 0.30 `android` 后端（`android-activity`；`android_main` 入口）＋ `ANativeWindow` |
| 呈现 | softbuffer 像素 blit（整数倍缩放/黑边） | EGL/GLES2 纹理上传（`glow` 族；**softbuffer 不支持 Android** ⇒ 平台侧实现 `Renderer`） |
| 输入 | 键盘→`Buttons` | 触摸虚拟手柄/外接手柄→`Buttons`（**布局与连发语义未证=占位**）；BACK→`Quit` |
| 音频 | cpal 族（PCM=静音占位显式）；消费 ApuWrite 流 | **AAudio**（API≥26）或 **oboe**（低版本兼容）；`Audio` 消费 ApuWrite 流→PCM |
| 资产 | `AssetSource`=`std::fs`；资产根=**可执行文件目录**（见 §4，不得依赖 cwd） | `AssetSource`=APK `assets/`（`AAssetManager`/JNI）；**资产根=APK `assets/` 根**，与 PC 共用同一相对路径语义（`ROM_ASSET_PATH`）；ROM 打包入 APK |
| 存储 | `%APPDATA%/gunnac`（Win）/`~/.local/share/gunnac`（Linux） | 应用内部目录（`Context.getFilesDir()`，JNI）；进程可被杀⇒快照时机待定 |
| 生命周期 | 关窗→Quit；挂起/恢复可选 | `winit` `Suspended`→`AppEvent::Suspend`、`Resumed`→`Resume`、destroy/back→`Quit`；宿主 suspend 停逻辑帧 |
| 编译验证 | 本机 `cargo test/build` 直接验证 | NDK + `cargo-apk`/gradle 交叉构建与真机验证（后续任务卡） |

## 6. Android 落地清单（后续任务卡逐项验收；本版只定契约/清单）

1. **winit 0.30 Android**：`android-activity` 依赖 + `#[cfg(target_os="android")] android_main`；
   `EventLoop` / 生命周期事件映射 `AppEvent`（§5 生命周期行）。
2. **呈现**：EGL/GLES2 上下文 + 256×240 RGBA 纹理上传；平台侧实现
   `Renderer::canvas_mut/present/size/resize`（softbuffer 不可用）。**surface 获取/Resume 重建路径待定
   （P1）**：候选=①winit Android 扩展取 native window + 自建 EGLSurface 重建；②ndk 原生 surface 管理。
   与 winit 事件循环的线程/时机约束需在 Android 任务卡先做 spike 定谳。
3. **音频**：AAudio（≥API 26）或 oboe 设备流；平台 `Audio` 消费 ApuWrite 流；合成器未证期静音占位显式。
4. **资产**：ROM/数据打包 APK `assets/`；`AssetSource` 用 `AAssetManager` 读整块字节（路径语义与 PC 同相对路径）。
5. **存储**：内部目录 `Storage` 实现；快照写入时机（生命周期/用户操作）由规格后续锁定（不自动保存）。
6. **生命周期**：suspend 时释放/暂停 GL 与音频设备、恢复时重建（真实资源策略=平台实现侧，占位）；
   宿主 `Suspend⇒停逻辑帧`。
7. **PCM/合成器**：APU 写流→PCM 合成属后续已证音频战役；Android 与 PC 共用同一 `Audio` 语义。
8. **命名**：`shell-win`→`shell-pc` 更名由用户裁定；Android crate 名 `shell-android` 规划。

## 7. 工程纪律

- 平台依赖（winit/softbuffer/cpal/AAudio…）**仅 shell crate**；`game-core`/`game-platform`/`game-shell`
  零平台依赖由机器测试把关（`game-core/tests/contract.rs`、`game-platform/tests/dependency_contract.rs`、
  `game-shell/tests/contract.rs`）。
- **契约门禁**：game-shell 零平台依赖扫描 + mock platform 驱动 300 帧两次确定性签名一致
  （`cargo test -p game-shell --test mock_platform -- --nocapture`）。
- TDD 双关照旧：实现→对拍（cargo test 全集 + `python gates/cli.py check`）→独立对抗复核（verifier 角色）。
- 编译验证：本机 cargo 直接过；真机运行验证=Windows/Linux 桌面用户侧 / Android 真机后续。

## 8. 未证域声明（fail-closed；实现侧不得"顺手"补齐）

H-01 弹丸/武器槽语义、H-16 命中判定、Boss/EN-9+ 段、商店消费侧、音效/音乐触发与 PCM 合成
（AUD-2/3 内部体）、原版调色板映射/BG nametable/文本流/结局卡渲染、玩家主武器连射与 A 边沿投放接线、
菜单输入门控派发、关卡数据装载（脚本/调度/shape/初始槽）、触摸虚拟手柄布局/连发、Android 音频后端
选型细节/EGL 资源策略/存档时机、原版存档语义、`StageStart` 段玩家可动性、死亡窗滚动行为。
逐条处理口径见 [plan-platform-abstraction.md](../findings/plan-platform-abstraction.md) §6。

v1 复核（task-6）补充登记：

- **死亡窗/关末并发窗的 engine tick 细粒度行为未隔离**（G-S5-5）：`DeathRespawn`/`StageClear` 内
  跑 `engine::stage_tick` 是**接线约定**，不是已证 gameplay 段（A2 `death_chain_step` / A3
  `stage_clear_step` 尚未接线）；差分对拍时视作未证域。
- **除 `Combat→StageClear` 外的模式时序已证但未接线**：关始 intro 240f（R-ui-6）、A3 关末细段、
  A2 死亡链、R-respawn 玩家侧复位——v1 由平台/上层显式投递 `ModeEvent`，不硬编码时序。
- **双状态源同步契约待定**：`Host.stage_index` vs `StageState.stage_counter`（`$180`）；
  `player.bomb_ammo` vs `FlowState.data.bomb_ammo`——接 game-data/死亡链任务卡的前置条款。
- **快照只做格式 fail-closed**（不做 stage 范围/mode×stage 组合语义校验；不恢复 `prev_buttons/pacer`）。
- **依赖门禁扫描面**只解析 `[dependencies]`（dev/build/target 段=后续加固）。
- **菜单域 `$16` START/SELECT 边沿**（bit4/bit5）不在 v1 `pressed_bitmap`（仅方向+A/B 子集）。
- **平台接入契约**：`Host::advance(now_ns)` 时间基准=相对 Host 创建（首帧=0）；PC `AssetSource`
  资产根=可执行文件目录（不得依赖 cwd），Android=APK `assets/` 根。

## 9. 验收与复现命令（task-5 v1）

```sh
# 1) trait 缝（形状锁 + 新 trait 契约 + 零外部依赖）
cargo test -p game-platform

# 2) 平台无关宿主（单元 + 零平台依赖扫描 + mock 300 帧确定性）
cargo test -p game-shell
cargo test -p game-shell --test mock_platform mock_platform_drives_300_frames_deterministically -- --nocapture
#   2026-10-18 记录值：host=0xefe39a9ba5e7c5b0 canvas=0x963ac9a776a57625 advanced_frames=290 storage_len=24
#   （连跑两次 AssertEq；签名覆盖宿主状态+画布/音频摘要，后续变更若签名变=需复核是否为预期）

# 3) 全量 + 总闸
cargo test --workspace       # 2026-10-18 task-5 tip：exit 0；326 个 #[test]：325 passed + 1 既有 ignored
python gates/cli.py check    # CHECK: PASS（PASS=11 FAIL=0 SKIP=1；rust-tests 326/326 OK）
```

**平台接入注意**：`Host::advance(now_ns)` 的 `now_ns` 必须**相对 Host 创建时刻**（首帧=0，S1）；
PC `AssetSource` 资产根=**可执行文件目录**、Android=`APK assets/` 根（S3）；旧 `FRAME_MS=16` 不得与
`FixedStep` 并存（P3）。

**v1 完成面**：game-platform 缝补全（AssetSource/Lifecycle·AppEvent/Renderer size·resize）+ game-shell
平台无关宿主（固定步/输入消解/状态机/tick/合成/音频/资产/存档/生命周期端口）+ 契约门禁。
**后续**：shell/win 窗口与设备实现（task-1）、game-data 装载接线、Android 落地（§6）、差分/视觉战役。
