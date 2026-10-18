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
| `input` | **对向同持消解**（L+R/U+D⇒两键 0；进入 core 前完成，core 对未证组合 panic=W6）；`$18` 位图（R-move-1 A 级 bit0/1/2/3/6/7）+ `$16` 边沿位图（v1 **仅方向+A/B 子集**，不含 START/SELECT；只产出不消费）+ 菜单域 `$18` 按下沿位图（含 START/SELECT，喂 `InputLatch`） |
| `host` | 宿主状态机（`FlowState`+关号 `stage_index`）只走 game-core flow **已证边集**；关号守卫（E37C/E392）；**菜单/模式派发**（`InputLatch`+1f/12f、`MenuInputGate` 两层门、`ShopState::on_b`、帧锚 Boot@180/Black@360、33f/240f 常数、响应延迟队列）；**STG-01 装载**（`decode_sched($A26D,43)`+`ScriptCursor::stg01`+自机槽初态）；**stage clock 滚动源**（`stage::scroll_delta`）；spawn 效果落地（`land_pair`/`land_leader`）+ 激活日志；tick 序列（见下）；`signature()` 确定性状态签名；AssetSource/Storage/Lifecycle 端口 |
| `render` | 渲染合成：清屏 + 已证坐标面玩家 8×8 占位块（`$04E0/$04FC`）；BG/敌/弹/HUD/文本/结局卡**不画**（未证）；另有只读 `FrameView`（模式/关号/帧号/玩家/玩家子弹活弹数/敌槽原始型值+坐标 None 口径/HUD 已证数值域）供平台侧渲染合成（task-10 消费） |
| `audio` | `ApuWriteStream`→`Audio` 保序 `write_apu` + 每逻辑帧 `end_frame`；**v1 生产者未接**（音频驱动内部体未证） |
| `snapshot` | Storage 存档缝占位：`GNSH` v1 定长快照（不自动保存；损坏 fail-closed） |

**帧内 tick 序列（每逻辑帧）**

1. 输入采样 → `resolve_directions` → `held`/`pressed` 位图；
2. 模式门：`engine::stage_tick`：`Combat`=已证 gameplay 段（R-stage-tick）；`Transition|StageStart`=
   **B 级接线依据**（装载后滚动在转场窗推进=`.scroll.jsonl` A 级；出敌在 stage-start 窗内发生=
   `stage-appear` f1278∈[1260,1500)）；`DeathRespawn|StageClear`=**接线约定**（细粒度行为未隔离，
   G-S5-5 死亡窗/关末门并发未取证；A2/A3 细粒度状态机未接线）。`player::tick`=`Combat|StageStart`
   （**StageStart 可动=A 级 trace**：`.player.jsonl` f1301 首移；`DeathRespawn` 窗冻结=R-respawn-2 A 级，
   不跑）。滚动源=stage clock + `stage::scroll_delta`（A 级逐帧；见 §3.1）；未装载关卡⇒引擎不 tick（fail-closed）；
3. 消费 `StageEffects`：spawn 项落地（`Pair→land_pair` / `Spawn→land_leader(...,None)`；其余 item
   fail-closed 不落地）+ 槽 0→非0 激活日志；`stage_clear_requested`（`$26 bit4` 门）在 `Combat` ⇒
   自动投递已证边 `Combat→StageClear`；其余效果只登记摘要（不发明消费语义）；
4. Audio 流消费 → 渲染合成 + `present` → `FrameClock::tick`；
5. suspend/quit 帧不推进、不呈现（生命周期门控）。

**宿主状态机边集**：`Boot→Black→Title→Menu→Shop→Transition→StageStart→Combat→DeathRespawn→
GameOverPending→GameOverMenu→StageClear→Ending` + 关末 `StageAdvance(→Shop)` / `AllClear(→Ending)`；
未证边 fail-closed（core `Err` 上抛），宿主关号守卫：`StageAdvance` 仅关号<8、`AllClear` 仅关号==8。
边集证据指针见 [game-core flow.rs](../../crates/game-core/src/flow.rs) 注释（R-ui-6/7、A2 F6-F12、
A3 R-l5-3..6）。

**驱动方（v1）**：`Combat→StageClear`（`$26 bit4` 门）与**菜单/模式派发**为宿主自动接线；其余
`ModeEvent` 由平台/上层经 `Host::apply_event_hooked` 显式投递。菜单派发规则（证据级见
[plan-playable-loop.md](../findings/plan-playable-loop.md) §4）：`InputLatch` press@N→N+1f/保持 12f、
标题段锁 `(450,550]` 后解锁、`MenuInputGate` 按下沿派发、`ShopState::on_b` 双 B 退出、
Boot@180/Black@360 帧锚、转场/关始 33f/240f 常数、出店 +33f 响应延迟。**已证但未接线**的时序（S8）：
A3 关末细段 `stage_clear_step`、A2 死亡链 `death_chain_step`、R-respawn 玩家侧复位——属后续任务卡；
shell 侧不得静默硬编码 240f/400f/265f 当作已收敛行为。

**§3.1 可玩闭环 A 实施状态（2026-10-18，task-9 回填）**

| 件 | 实现 | 验证 |
|---|---|---|
| 菜单/模式派发 | `Host::dispatch_menu`/`auto_advance`/`due_events`（`InputLatch`+`MenuInputGate`+`ShopState::on_b`+帧锚/常数/延迟队列） | 单元测试：Boot→…→Combat 完整序列、标题锁窗内 Start 无效、Shop 内 Start 无效 |
| 菜单派发**触发键分离**（task-29 B1 修） | `Title/Menu`：层 1 **只喂 START 位**（A 级 START 专用早退门 bank00:92AA/9400 `AND #$10`）；`Shop/ShopInfo`：`$16` **原始按下沿**（键级；B 保持/边沿门未证 ⇒ 边沿模型 fail-closed，不发明连发） | 回归：`menu_start_not_swallowed_by_other_keys`、`shop_two_b_presses_within_12_frames_both_register`；方案见 [plan-menuinput-fix.md](../findings/plan-menuinput-fix.md) |
| STG-01 装载 | `Host::load_stg01`（`AssetSource`→`game_data::rom::prg_bank(8)`→`decode_sched($A26D,64)`=43 记录→`StageState`）；转场帧自动装载（A 级 f1193 锚） | `tests/stg01_replay.rs`：43 记录 + tick 锚 [1,33,49,65,81]（A 级 golden） |
| 滚动源 | `Host::stage_scroll_delta`=`-stage::scroll_delta(abs)`；`abs=1193+(frame-load_frame)` | **C1：vs `.scroll.jsonl` 1207/1207 帧 0 失配**（A 级） |
| spawn 落地/摘要 | `apply_stage_effects`（`land_pair`/`land_leader`）+ `scan_spawns`（`$0400` 0→非0 判定式） | C4：共同槽 16/16、engine 型集 ⊆ oracle 型集；帧列不可对拍（见 §8） |
| FrameView | `Host::frame_view()`：mode/stage/frame/player/live_bullets/player_bullets_live/敌槽型值（坐标 None）/HUD 已证数值域 | 单测：字段只读幂等、position=None 显式、槽 8..=27 |
| 玩家弹丸生成/运动（task-37） | `fire.rs`：B 长按（`$18` bit6）→ `$39/$3A` 计时器（默认 `$3C`=0 ⇒ **reload=7**，bank02:$94DF 表 `07 05 04 03 02`）→ 六型生成器（在途上限 `$074A`）→ `shot::tick` 运动/寿命/死亡回收；弹网格为 **game-shell 自有**（不写 `StageState.bullets`） | 单测：7f 周期/上限/槽序/出生字段/运动/死亡回收；集成：`bullets_seen>0` 且两次 run 全等 |
| 框形数据装载（task-37） | `shape.rs`：ROM **bank2 `$8000` 窗** → `shape_stream_addr($97BA/$9812)` 表 → 流 `[count][4B]` → `StageState.shape_data`（装载侧喂，不造数据） | 单测：合成流/截断 fail-closed；集成：装载 59 形 |
| 命中 + 受限额伤害（task-37） | shell 侧 `shot_hitbox_register`+`fetch_shape_boxes`+`hit_resolves`+`clear_hit_entry`；伤害**仅 `type==$4A`**（HP=14 A 级）`Const2`，其余 **0 伤 + `fail_closed` 事件**（不写 `f630`）；敌弹按 `type_code & 0x80` 过滤 | 单测：命中正例/负例 ≥4、`$4A` 7 发 14→0 不回收、敌弹不参与；**非 H-16 收口**（§8） |
| FrameView（task-37 扩展） | `player_bullets: [Option<BulletView>;20]`（`$04EE/$055E/$063E` A 地址面）/`hits_this_frame: [Option<EnemyHitEvent>;4]`/`FrameView::default()` | 单测：只读幂等、空槽 None、事件 fail-closed 显式 |
| 确定性 | 3000 帧 replay 两次全等 | `host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807` |
| 玩家列对拍（新增 C6） | `player::tick` vs `.player.jsonl`（死亡窗剔除 f1260..=1830） | **571/571 帧一致**（A 级） |

> `Host::signature()` 语义在 task-9 扩展（新增菜单锁存/门控/装载/spawn 日志/阶段计时覆盖）；
> task-29 修 B1 后触发键域/层 1 状态更新时机变化 ⇒ 签名值再次更新（§4.2；**非回归**，两次运行确定性不变）。
> task-37 再扩展玩家弹网格/计时器/shape 装载/命中事件覆盖 ⇒ 值再次更新（§4.2/§9；**非回归**）。
> 值链：task-1 `0xcde9dc99514b5f4b` → task-9 `0xf51d5a517e7458b7` → task-29 `0x5d7156266d6ac19d`
> → task-37 `0x68547665bf1ab396`（shell-win selftest 时点值）。

## 4. shell-pc 架构（首实现，现名 `shell/win`）

| 件 | 设计 |
|---|---|
| 窗口 | winit 0.30（跨平台同源）+ softbuffer CPU 帧缓冲（game-desktop 同族）；Win/Linux 同源编译 |
| 帧驱动 | 平台单调时钟→`game_shell::Host::advance(now_ns)`（60Hz 固定步）→ 渲染 → present。**时间基准（S1）**：`now_ns` 必须**相对 `Host` 创建时刻**（首帧=0，平台侧用 `Instant::now() - instant_at_host_creation` 折算）；禁止传平台绝对 uptime ns（否则首帧被当成积压、受追帧上限 4 截断而启动偏移）。`advance` 返回**折算帧数**；suspend/quit 帧不推进逻辑，实际推进帧数=前后 `Host::frame()` 之差 |
| 帧呈现 | 256×240 RGBA → blit 整数倍缩放（保比例黑边——game-desktop blit_scaled 语义） |
| 输入 | 键盘事件 → `Buttons`（Z/X/Enter/RShift/方向键）；对向同持由 game-shell 统一消解 |
| 音频 | 平台 `Audio` 实现：v1=**ApuWrite 写流消费 + PCM 静音占位显式**（`PcAudio`，**无 cpal 设备流**）；cpal 设备输出与 APU 合成器（AUD-2/3）=后续任务卡 |
| 资产 | `AssetSource`=`std::fs`；**资产根=可执行文件所在目录**（`exe/rom/gunnac_cn.nes`；可回退 `exe/../rom/` 与 `CARGO_MANIFEST_DIR` 开发态），**不得依赖进程 cwd**（S3）；逻辑路径 `ROM_ASSET_PATH` 跨平台一致 |
| 存储 | `Storage`：Windows=`%APPDATA%/gunnac/`；Linux=`~/.local/share/gunnac/` |
| 生命周期 | 关窗→`AppEvent::Quit`；最小化/焦点可选发 Suspend/Resume（桌面可不发） |
| 实现状态 | **v1 已落地**（task-1，2026-10-18）+ **v2 渲染覆盖层**（task-10，2026-10-18）：六 trait PC 实现 + winit/softbuffer 主循环 + FrameView 占位覆盖层 + 确定性自检；逐件状态见 §4.1 |

> 命名：本 crate 当前名 `shell-win`（=PC 第一实现，Win/Linux 同源）。**是否更名 `shell-pc` 由用户裁定**；
> 本规格与任务不改名、不移动 crate。

### 4.1 实现状态（2026-10-18，task-1 回填；task-10 v2 渲染/自检行回填）

| 件 | 实现（文件） | 验证 |
|---|---|---|
| 窗口 | `WindowRenderer`（`crates/shell-win/src/main.rs`）：winit 0.30 `ApplicationHandler` + softbuffer `Surface`；标题 `gunnac-remake (Stage A)`；初始 512×480（2×） | 本机实跑窗口 `MainWindowTitle`=`gunnac-remake (Stage A)`；标题常量单测 |
| 帧驱动 | `about_to_wait`：`Instant`→整数 ns→`game_shell::Host::advance`（`FixedStep` 60Hz、追帧上限 4）；`ControlFlow::WaitUntil` 节流 | `--selftest-frames 300`：advanced=300（每采样 1 逻辑帧） |
| 呈现 | `FrameRenderer` 合成 + `blit_scaled` 整数倍缩放/黑边（softbuffer `0x00RRGGBB`）；**每采样 1 次 softbuffer present**（task-36 E3 收口：`Renderer::present` 只合成+计数、统一由 `refresh_view` 上屏；收口前 2 次=采样临时口径；`presents` 计数语义不变） | `view_rect`/`blit_scaled` 单测（2×/3×/黑边/小窗裁切）；selftest ×2 四签名不变 |
| 输入 | `KeyboardInput`+`key_name`：winit 物理键码→`Buttons`（Z/X/Enter/RShift/方向）；对向同持交 game-shell `resolve_directions` 消解 | 键位表单测 + 按下/抬起状态单测 |
| 音频 | `PcAudio`（`Audio` impl）：真实消费 game-shell `ApuWrite` 流（写序保真登记，计数不丢）；PCM=静音占位显式（`silent()=true`） | 单测：写流保序/计数/`end_frame`/零 PCM；自检注入 2 条写流被消费（audio_writes=2） |
| 资产 | `FsAssets`（`AssetSource` impl）：`std::fs` 读 `rom/gunnac_cn.nes`；**资产根=exe 目录优先**（`exe/rom` → `exe/../rom`；debug 回退 `CARGO_MANIFEST_DIR`），**不依赖进程 cwd**；启动（窗口/自检）始终打印实际资产根。**拒绝绝对路径/Windows 前缀（`C:foo` drive-relative）/`..`/缺失** ⇒ `None`（fail-closed，B1/S1） | 单测：exe 两布局解析/前缀拒绝；smoke：异 cwd 子进程仍定位 ROM；自检 rom_len=262160 |

> **R4/S1 收口留痕（task-23）**：上表资产行为**现状**（exe 目录优先、不依赖 cwd）。
> S1 复核时点的旧现状「v1=相对 cwd」已由 task-18（commit `ddb1558`，B1/S1 收口）修正；
> 此处保留历史口径以免读者把旧注当现状。
| 存储 | `FileStorage`（`Storage` impl）：`%APPDATA%/gunnac`（Win）/`~/.local/share/gunnac`（Linux），`slot{n}.sav` | 单测：往返/空槽 fail-closed；**不自动保存**（原版存档语义未证） |
| 生命周期 | `LifecycleQueue`：关窗→`Quit`；焦点丢失/获得→`Suspend`/`Resume`（最小平台策略）；宿主 suspend 停逻辑帧 | FIFO 单测；game-shell 门控测试 |
| 渲染内容（v2，task-10；task-44 子弹） | `FrameView` 覆盖层：CHR 背景（v1 灰阶占位）+ 玩家 8×8（game-shell 已证坐标面 `$04E0/$04FC`）+ **模式占位角标**（`mode_tag`→`MODE_MARKER_COLORS`；Boot/Title/Menu/Shop/Stage 可见区分；**非原版美术/文字**）+ **HUD 数值域占位编码**（`money_digits(digit,show)` 逐桁 2×2 点阵、`bomb_ammo` 8bit 位行、`stage` 4bit 位行；固定占位槽；**非原版字形/布局/文本**）+ **敌槽占位块**（仅 `position+type_code` 均 `Some`；v1 运行态 `position` 恒 None⇒不画；`type_code` 不反推坐标）+ **玩家子弹占位块**（task-44：`player_bullets` 中 `Some(BulletView)` 按已证坐标面 `$04EE/$055E` 画 4×4 纯色占位块；图样/尺寸/颜色/弹道未证⇒`TODO(shell-v2)`；`hits_this_frame` 不画布、仅入 view 摘要） | 单测：合成保玩家块/模式占位色/HUD 编码/敌槽门/子弹坐标+数量变化⇒像素变化；smoke：1400 帧按键透传→Host 内派发→Combat→按 B 实弹渲染断言；GUI PostMessage+PrintWindow 抽样：子弹占位色像素出现（bbox/计数见 plan-shellwin-v2 §8.6） |
| 自检 | `cargo run -p shell-win -- --selftest-frames N`：无窗口、真实 `AssetSource`、固定已证 `ModeEvent`/按钮脚本；每帧用 `Host::frame_view()` 最新视图重合成覆盖层；新增**独立 view 摘要签名**（仅 FrameView 字段 FNV、平台无关；**不改** `Host::signature()` 语义） | 300 帧连跑两次 stdout 逐行一致、退出码 0（§4.2，含 `view_signature`） |

**迁移**：旧骨架的 `FrameDriver`/`GameHost`（重复宿主逻辑）已删除，改接 `game_shell::FixedStep`/
`Host`；`AudioSink` 改为 `PcAudio`（平台 `Audio` 实现，不是第二套宿主）。实现方案与未证域声明见
[plan-shellwin-v1.md](../findings/plan-shellwin-v1.md)。
**节流口径（P3）**：旧 `FRAME_MS=16` 仅保留为历史占位常量（呈现/文档参考），主循环节流一律
`game_shell::FixedStep`（ns 固定步）；**不得同时保留两套节流**。

**§4.2 自检记录值（2026-10-18 本机 Windows；连跑两次逐行一致）**

```text
selftest: frames=300 advanced=300 presents=300
selftest: mode=Combat stage=1 frame=300
selftest: host_signature=0x68547665bf1ab396
selftest: canvas_signature=0x963ac9a776a57625
selftest: display_signature=0xe99af5849b632fbd
selftest: view_signature=0x15dbc2c7b0ba0a5d
selftest: audio_writes=2 audio_frames=300
selftest: rom_len=Some(262160)
selftest: OK
```

> 【2026-10-18 task-9 更新】`host_signature` 记录值由 `0xcde9dc99514b5f4b` → `0xf51d5a517e7458b7`：
> 原因=`Host::signature()` 语义扩展（新增菜单锁存/门控/STG-01 装载/spawn 日志/阶段计时覆盖），
> **非回归**；`canvas_signature`/`display_signature` 不变（渲染面未改）。shell-win selftest 用
> `apply_event`（无转场装载挂钩）⇒ selftest 内 stage_loaded=false、引擎不 tick；GUI 主循环走
> game-shell 菜单派发自动接线（转场帧自动装载 STG-01）。
>
> 【2026-10-18 task-29 更新】`host_signature` 由 `0xf51d5a517e7458b7` → **`0x5d7156266d6ac19d`**：
> 原因=B1 修复（菜单触发键域分离：`menu_latch` 仅锁存 START、层 1 `prev_held` 只在 Title/Menu 更新；
> 见 [plan-menuinput-fix.md](../findings/plan-menuinput-fix.md)），签名覆盖 `menu_latch/gate` 状态；
> **非回归**，两次运行逐行一致、`canvas/display` 不变。shell-win `plan-shellwin-v1.md` 的时点值
> 需由 shell-porter 同步（不在本卡写范围）。
>
> 【2026-10-18 task-10 更新】`display_signature` 由 `0xf68f9ffc0607cbad` → **`0xe99af5849b632fbd`**，
> 并新增 `view_signature=0xf6603346bf818ffd`：原因=平台侧 v2 覆盖层（模式占位角标/HUD 数值域占位编码/
> 敌槽占位门）进入合成；`canvas_signature` 不变（game-shell 逻辑画布未改）、`host_signature` 不变
> （游戏逻辑未改）。**非回归**：300 帧连跑两次 stdout 逐行一致；覆盖层为**可见性占位**（非原版像素/
> 布局/文本，L4 不适用，不声称一致性）。
>
> 【2026-10-18 task-37 更新】`host_signature` 由 `0x5d7156266d6ac19d` → **`0x68547665bf1ab396`**：
> 原因=玩家弹丸/命中接线（签名新增自有弹网格 20 槽六字段/`$39` 计时器/shape 装载数/命中事件日志/
> 溢出计数）；**非回归**，300 帧连跑两次逐行一致。`canvas_signature`/`display_signature`/
> `view_signature` 均不变（渲染与 FrameView 视图签名字段语义未改；shell-win 仅 FrameView 字面量
> 适配 `..Default::default()`，见 [plan-bullets-hit.md](../findings/plan-bullets-hit.md) §8.1）。
>
> 【2026-10-18 task-44 更新】`view_signature` 由 `0xf6603346bf818ffd` → **`0x15dbc2c7b0ba0a5d`**：
> 原因=平台侧 view 摘要扩展纳入 `player_bullets`（20 槽：None/Some tag + `slot/x/y/type_code`）与
> `hits_this_frame`（事件面）；并开始按已证坐标面渲染玩家子弹 4×4 占位块（图样/尺寸/颜色未证）。
> `display_signature`/`canvas_signature`/`host_signature` 不变（selftest 无关卡装载⇒不产弹，
> 游戏逻辑未改）；**非回归**：300 帧连跑两次 stdout 逐行一致。GUI 实跑（PostMessage+PrintWindow）
> 抽样到子弹占位色像素（§4.1 验证列 / plan-shellwin-v2 §8.6）。

**口径**：`host_signature`/`canvas_signature`/`display_signature`/`view_signature` 是**确定性回归锚**，
**不声称**等价真机 trace 对拍（platform-shell.md §3/§9 同口径）；`display_signature` 含平台侧
CHR 背景+`FrameView` 覆盖层占位合成（非原版像素/布局，L4 不适用）；`view_signature` 仅覆盖 FrameView
字段（平台无关）。GUI 视觉（缩放/黑边/CHR 背景/模式角标/HUD 占位）为手动验收项。

**未证/未接（与 §8 一致）**：菜单/模式派发已由 game-shell 接线（窗口可从 Boot 推进、Start 派发、
转场帧装载 STG-01）；**shell-win 渲染（v2+）**=CHR 背景+玩家块+**占位覆盖层**（模式角标/HUD 数值域
占位编码/敌槽 Some 占位门/**玩家子弹已证坐标面 4×4 占位块**）；**敌坐标写者链/敌图样、玩家子弹
图样/尺寸/颜色/弹道（H-01 余项）、HUD 字形/布局/文本、原版 BG/nametable/美术一律占位/不画/不发明**；
GUI 键推进已可用 `PostMessage(WM_KEYDOWN)`+`PrintWindow` 自动化（task-44 实测；`SendKeys`/前台授权
在本会话失败属环境限制）；音频 PCM 静音占位；spawn 坐标写者链/初生记录未接；
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
**敌 spawn 帧级对齐与初生记录 loader**、**敌坐标写者链（spawn 定位/behavior VM 位置写）**、
触摸虚拟手柄布局/连发、Android 音频后端选型细节/EGL 资源策略/存档时机、原版存档语义、死亡窗滚动行为。
逐条处理口径见 [plan-playable-loop.md](../findings/plan-playable-loop.md) §8 +
[plan-platform-abstraction.md](../findings/plan-platform-abstraction.md) §6。

> 【2026-10-18 task-9 更新】「菜单输入门控派发」「关卡数据装载」从**未实现**转为**已接线**（菜单派发
> 输入驱动 + STG-01 装载，见 §3.1）；但两者仍有未证边界：菜单→商店中间动画时长（主循环只保证序列不
> 保证帧级）、spawn 帧级对齐/初生记录/坐标写者链、Boss/EN-9+ 后段（decode 视界仅 43 记录主段）。
>
> 【2026-10-18 task-37 更新】「玩家弹丸生成/运动/命中」从**未实现**转为**部分接线**（§3.1；game-shell
> 自有弹网格 + shell 命中管线 + `$4A` 白名单伤害）。**不是 H-16 收口**：精确受击盒（W15/W15′/W18）、
> 普通敌 HP（B4）、`$4A` 死亡回收、伤害路径选择（`auto_hit_path`）仍未证 ⇒ 0 伤/不回收 fail-closed；
> 玩家被击、敌弹友伤、`$34` 非 0 分派、`$3C` 连射升级未接。详见
> [plan-bullets-hit.md](../findings/plan-bullets-hit.md) §6/§8.3。

v1 复核（task-6）补充登记：

- **死亡窗/关末并发窗的 engine tick 细粒度行为未隔离**（G-S5-5）：`DeathRespawn`/`StageClear` 内
  跑 `engine::stage_tick` 是**接线约定**，不是已证 gameplay 段（A2 `death_chain_step` / A3
  `stage_clear_step` 尚未接线）；差分对拍时视作未证域。
- **除 `Combat→StageClear` 外的模式时序**：关始 intro 240f（R-ui-6）、A3 关末细段、A2 死亡链、
  R-respawn 玩家侧复位——菜单序列由 task-9 输入驱动接线；A2/A3 细粒度仍由平台/上层显式投递。
- **双状态源同步契约待定**：`Host.stage_index` vs `StageState.stage_counter`（`$180`）；
  `player.bomb_ammo` vs `FlowState.data.bomb_ammo`——接 game-data/死亡链任务卡的前置条款。
- **快照只做格式 fail-closed**（不做 stage 范围/mode×stage 组合语义校验；不恢复 `prev_buttons/pacer`）。
- **依赖门禁扫描面**只解析 `[dependencies]`（dev/build/target 段=后续加固）。
- **菜单域 `$16` START/SELECT 边沿**（bit4/bit5）不在 `pressed_bitmap`（仅方向+A/B 子集）；
  菜单派发改用独立 `menu_pressed_bitmap`（$18 域，含 START/SELECT）。
- **平台接入契约**：`Host::advance(now_ns)` 时间基准=相对 Host 创建（首帧=0）；PC `AssetSource`
  资产根=可执行文件目录（不得依赖 cwd），Android=APK `assets/` 根。
- **game-shell 依赖**：task-9 起含 `game-data`（纯解析；ROM 只读窗口 `game_data::rom::prg_bank`），
  零平台依赖门禁同步白名单。
- **HUD 可见性/模式门未证**（task-36 E5）：原版 HUD 的显示时机/可见性未取证；v2 **所有模式**
  （含 Boot/Title/Menu/Shop）恒画 HUD 数值域**占位编码**——**不声称**原版显示时机/可见性/布局一致，
  不改绘制行为。
- **每采样上屏次数**（task-36 E3/S2）：窗口模式每采样 **1 次** softbuffer present（`Renderer::present`
  只合成+计数、统一由 `refresh_view()` 上屏）；收口前为 2 次（tick 内上一帧 view + refresh 最新帧）
  **采样临时口径**。`presents` 计数语义仍=逻辑帧数；`display_signature`/`view_signature` 不变。

## 9. 验收与复现命令（task-5 v1 + task-9 可玩闭环 A）

```sh
# 1) trait 缝（形状锁 + 新 trait 契约 + 零外部依赖）
cargo test -p game-platform

# 2) ROM 只读窗口（task-9）
cargo test -p game-data

# 3) 平台无关宿主（单元 + 零平台依赖扫描 + mock 300 帧确定性 + STG-01 replay 对拍）
cargo test -p game-shell
cargo test -p game-shell --test mock_platform mock_platform_drives_300_frames_deterministically -- --nocapture
cargo test -p game-shell --test stg01_replay -- --nocapture
#   2026-10-18 task-37 记录值：mock-300 host=0x8170870e7c5e4ac6 canvas=0x963ac9a776a57625 advanced_frames=290
#    （task-29 旧值 0x3085adda845a078d；签名新增弹网格/计时器/shape/命中事件字段，非回归；两次 AssertEq）
#   STG-01 replay 记录值：host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807
#    （task-9/29 旧值 0x521d397a6c609f34；原因同上；C1 1207/1207、C6 571/571、C4 共同槽 16/16 未变）
#   对拍：C1 滚动 1207/1207 0 失配；C6 玩家列 571/571 一致；C4 共同槽 16/16（帧列不可对拍，见 §8）

# 4) 平台侧回归（shell-win 未改行为；selftest 两次逐行一致）
cargo run -p shell-win -- --selftest-frames 300
#   2026-10-18 task-37 后 host_signature=0x68547665bf1ab396（弹/命中接线语义更新；canvas/display 不变；
#   view 由 task-44 摘要扩展更新为 0x15dbc2c7b0ba0a5d，见 §4.2）

# 4b) STG-01 战斗弹丸/命中集成（task-37；真实 ROM，两次 run 全等）
cargo test -p game-shell --test stg01_combat -- --nocapture

# 5) 全量 + 总闸
cargo test --workspace
python gates/cli.py check    # CHECK: PASS（SKIP 显式打印；rust_test_count 上升=基线由 Lead 上调）
```

**平台接入注意**：`Host::advance(now_ns)` 的 `now_ns` 必须**相对 Host 创建时刻**（首帧=0，S1）；
PC `AssetSource` 资产根=**可执行文件目录**、Android=`APK assets/` 根（S3）；旧 `FRAME_MS=16` 不得与
`FixedStep` 并存（P3）。

**完成面**：game-platform 缝补全 + game-shell 平台无关宿主（固定步/输入消解/状态机/tick/合成/音频/
资产/存档/生命周期端口）+ **可玩闭环 A**（菜单派发 + STG-01 装载 + 滚动源 + spawn 落地 + FrameView +
确定性 replay 对拍）+ 契约门禁。
**后续**：shell/win 渲染扩展（task-10：FrameView→画面）、A2/A3 细粒度时序接线、敌坐标写者链与
初生记录 loader、Android 落地（§6）、差分/视觉战役。
