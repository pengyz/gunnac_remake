# shell-win v1 方案（PC 第一实现：game-platform trait 落地 + game-shell 宿主接线）

> 任务卡：task-1（2026-10-18 用户裁定：game-platform 是唯一 trait 缝；PC/Windows/Linux 是第一实现，
> Android 等同缝接入）。前置 task-5 已交付（`game-platform` v1 缝补全 + `game-shell` 平台无关宿主 +
> [platform-shell.md](../spec/platform-shell.md) v2）。
> 状态：**方案（实施前）**；实现完成后在 §9 回填状态（不改写本节历史主张）。
> 纪律依据：[AGENTS.md](../../AGENTS.md) §2/§3/§4/§9——未证不发明、证据 A/B 级才落结论、小步提交。

## 0. 目标与非目标

**目标（shell-win v1，一个逻辑步）**

1. 把 `crates/shell-win` 从 177 行库骨架做成**可运行的 PC 宿主**：`[[bin]] shell-win`
   （winit 0.30 `EventLoop` + softbuffer CPU 面 + 60Hz 固定步），窗口标题
   `gunnac-remake (Stage A)`，整数倍缩放 + 黑边。
2. 实现 `game-platform` 的 **PC 实现**：`Renderer`（softbuffer 呈现/尺寸/缩放）、`Audio`
   （真实消费 game-shell 的 `ApuWrite` 流；PCM=静音占位显式）、`Storage`
   （`%APPDATA%/gunnac`；Linux `~/.local/share/gunnac`）、`AssetSource`（`std::fs` 读
   `rom/gunnac_cn.nes`）、`Input`（winit 键码→`Buttons`；对向同持由 game-shell 统一消解）、
   `Lifecycle`（quit/resume 最小）。
3. 主循环只经 `game_shell::Host` 驱动：输入采样→`Host::advance(now_ns, …)`（固定步 60Hz）→
   呈现；宿主返回的状态/画布/音频写流全部真实消费。
4. 自检模式 `cargo run -p shell-win -- --selftest-frames N`：无窗口跑 N 逻辑帧、打印确定性
   状态签名、正常退出；连跑两次签名一致。
5. 渲染可见性骨架：CHR 背景（`game-data::chr` 解码，平台侧占位）+ 玩家 8×8（game-shell
   已证坐标面 `$04E0/$04FC`）；未暴露/未证域显式不画 + TODO。
6. 先本方案，后实现；实现完成后回填 [platform-shell.md](../spec/platform-shell.md) §4 状态与本文 §9。

**非目标（本任务不做）**

- 不实现 Android（只按 platform-shell.md §5/§6 清单预留同 trait 缝）。
- 不实现未证游戏行为：H-01 弹丸/武器槽语义、H-16 命中、Boss/EN-9+、商店消费侧、音效/音乐
  触发与 PCM 合成（AUD-2/3）、原版调色板/BG nametable/文本流/结局卡、菜单输入派发、关卡数据
  装载、触摸布局/连发、原版存档语义。
- 不改 `game-core` 语义、不改 `game-platform`/`game-shell`（task-5 写范围）；不改 `rom/`
  `reference/` `data/`；不引入平台依赖到 `game-core`/`game-platform`/`game-shell`。
- 不声称 L4 像素级一致：CHR 背景/灰阶调色板是**可见性骨架占位**，不是原版 BG 渲染。

## 1. 层定位与动手前三问（AGENTS.md §4）

| 问 | 答 |
|---|---|
| 问题在哪一层？ | **平台实现层**：`game-platform` 已有 trait 缝，`game-shell` 已有平台无关宿主；缺的是 PC 平台的设备实现（窗口/呈现/输入/音频/资产/存储/生命周期）与主循环接线。**不是**反汇编投影/数据表/core 语义问题。 |
| 消费者实际读的是哪个文件/数据？ | `shell-win` 实现 `game-platform` trait 并注入 `game_shell::Host`；宿主只消费 `game-core` 公开 API（`flow::FlowState`/`engine::stage_tick`/`player::tick`）。ROM 字节只经 `AssetSource` 取得；CHR 背景解码走 `game-data::chr`（纯解析，与 `game-desktop` 同源语义）。游戏状态不落盘（Storage 只存宿主快照字节）。 |
| 证据是什么级别？ | trait 形状/分层＝**架构合同**（AGENTS.md §2 + platform-shell.md §2 已锁定面）；60Hz 固定步/输入消解/宿主边集＝A/B 级（game-shell 已交付并带契约测试）；CHR 背景 tile 解码＝A 级（`game-data::chr` t2 布局），**调色板映射＝C 级未证**（灰阶占位显式）；玩家坐标＝A 级（`$04E0/$04FC`）。 |

## 2. 与 platform-shell.md v2 的对照（本卡只补 §4 的 shell-pc 实现面）

| platform-shell v2 合同 | shell-win v1 落地方式 | 状态 |
|---|---|---|
| `Renderer::canvas_mut/present`（256×240 RGBA） | `FrameRenderer`：持有 game-shell 逻辑画布；`present()` 合成后计数 | ✅ 本卡 |
| `Renderer::size/resize`（v1 新增占位） | `WindowRenderer` 覆盖：`size`=窗口内尺寸；`resize` 更新缓冲 | ✅ 本卡（输出面语义=窗口像素） |
| `Audio::write_apu/end_frame`（ApuWrite 流消费） | `PcAudio`：真实接收 game-shell `AudioPipe` 转发的事件；PCM=静音占位显式 | ✅ 本卡（合成器未证，显式静音） |
| `Storage::load/save`（字节块） | `FileStorage`：`%APPDATA%/gunnac`（Win）/`~/.local/share/gunnac`（Linux），`slot{n}.sav` | ✅ 本卡（字节存取；原版存档语义未证） |
| `AssetSource::read_all` | `FsAssets`：`std::fs`，根=仓库工作目录（失败回退 `CARGO_MANIFEST_DIR` 上级）；缺失⇒`None` fail-closed | ✅ 本卡 |
| `Input::buttons` | `KeyboardInput`：winit `PhysicalKey`→`Buttons`（Z/X/Enter/RShift/方向）；对向同持交给 game-shell `resolve_directions` | ✅ 本卡 |
| `Lifecycle::poll_event` / `AppEvent` | `LifecycleQueue`：关窗→`Quit`；焦点丢失/获得→`Suspend`/`Resume`（平台策略占位） | ✅ 本卡（最小） |
| 60Hz 固定步（`game-shell` `FixedStep`/`Host::advance`） | `main.rs` 只喂整数 ns（`Instant` 相对启动时刻）；`about_to_wait` 用 winit `ControlFlow::WaitUntil` 节流 | ✅ 本卡 |
| 渲染合成在 game-shell（`compose_frame`） | 保留（本卡不改 game-shell）；shell 只做**呈现侧**整数缩放/黑边/CHR 背景占位合成 | ✅ 本卡 |
| 未证域 fail-closed | 见 §6；代码内 `TODO(shell-v1)` 逐条登记 | ✅ 本卡 |

> 契约边界：`Renderer`/`Audio`/`Storage`/`AssetSource`/`Input`/`Lifecycle` 的**形状**沿用
> game-platform v1（本卡不改 trait）；shell 只实现与注入。

## 3. 迁移计划（删除骨架里的重复宿主逻辑）

| 旧骨架（`crates/shell-win/src/lib.rs` 177 行） | 处置 | 理由 |
|---|---|---|
| `FrameDriver`（`Instant` 节流+accum 固定步） | **删除**，改用 `game_shell::FixedStep`（经 `Host::advance`） | 固定步语义已在 game-shell 平台无关层锁一次；两套=两套时钟语义 |
| `GameHost`/`HostState`（Title→Stage→Ending `press_start` 简化状态机） | **删除**，改用 `game_shell::Host`（只走 game-core flow 已证边集 + 关号守卫） | 旧状态机是骨架占位，会绕过已证边集/守卫（重复宿主） |
| `AudioSink`（ApuWrite Vec + `synth_pcm` 静音） | **改名 `PcAudio` 并保留为平台 `Audio` 实现**（不是重复宿主逻辑：它是 trait 平台侧） | `Audio` trait 必须由平台实现；PCM 仍静音占位显式 |
| `map_key`（字符串键名→Buttons） | **保留**（平台输入策略语义表，供单元测试/键名映射复用） | 与 platform-shell.md §4 键位一致；winit 码→规范名在 `key_name` |
| `storage_dir`（cfg Windows/Unix 路径） | **保留**（`FileStorage` 的路径策略） | platform-shell.md §4/§5 已定 |
| `view_rect`/`blit_scaled`（整数缩放/黑边） | **保留并修正**：`blit_scaled` 先清黑边再 blit（旧实现未清视口外像素） | 呈现语义=整数倍+黑边；修正属实现缺陷，不改游戏行为 |
| （新增）`FrameRenderer`/`WindowRenderer` | 呈现侧合成与上屏 | 见 §4 |
| （新增）`FsAssets`/`FileStorage`/`LifecycleQueue`/`KeyboardInput` | 其余 trait 的 PC 实现 | 见 §2 |
| （新增）`run_selftest` | 无窗口确定性自检 | 见 §5 |

**主循环接线（每采样点）**

```
winit KeyboardInput/CloseRequested/Focused
      │
      ▼
LifecycleQueue ──poll──► Host::poll_lifecycle（suspend/quit 门控）
      │
now_ns = start.elapsed().as_nanos()
      ▼
Host::advance(now_ns, &input, &mut renderer, &mut audio)   // FixedStep：0..=4 逻辑帧
      │  每逻辑帧：输入采样→resolve→player/engine tick（按模式门）→AudioPipe.flush→
      │            compose_frame(canvas)→renderer.present()
      ▼
WindowRenderer::present：FrameRenderer 合成（CHR 背景 + game-shell 画布）→
                        softbuffer 整数缩放/黑边上屏
```

## 4. 呈现侧合成（可见性骨架；非 L4 像素目标）

- **逻辑画布**：game-shell `compose_frame` 写 256×240 RGBA（清屏不透明黑 + 玩家 8×8 块，
  坐标 `$04E0/$04FC` A 级）。shell **不**改写画布内容，只消费它。
- **CHR 背景**：平台侧预生成 256×240 RGBA 背景 = 字模区（`chr::FONT_BANK_START`=bank124）
  tile 按 `(tx+ty*3) % 64` 平铺，2bpp 解码=`game-data::chr::decode_tile`（t2 A 级），
  调色板=灰阶 `GREY4`（**C 级未证，占位显式**；复制自 `game-desktop::GREY4` 语义）。
  这不是原版 BG/nametable 渲染（未证），只为可见性骨架。
- **合成规则（占位，v1）**：game-shell 画布中**非清屏色** `(0,0,0,255)` 的像素覆盖 CHR 背景，
  其余保留背景。v1 game-shell 只画玩家块（白色）⇒ 屏上=CHR 背景+玩家 8×8。
  L4 渲染战役接管后删除该平台侧合成。
- **缩放**：`view_rect` 整数倍（`min(w/256,h/240)`，≥1）+ 居中黑边；`blit_scaled` 先整屏清 0
  再逐像素写视口（softbuffer `0x00RRGGBB`）。
- **显式不画（TODO(shell-v1)）**：敌精灵槽/子弹槽——game-shell v1 的渲染契约只暴露玩家坐标
  与活弹数（`FrameView.live_bullets` 不画），敌人槽（`ObjectGrid.slots`）无整数屏坐标、
  子弹行为（H-01）未证 ⇒ **不画、不发明**；HUD/文本/结局卡/调色板影子上传同属未证域。

## 5. 自检模式（`--selftest-frames N`）

- 无窗口：`Host` + `FrameRenderer` + `PcAudio` + `FsAssets`（真实读 ROM）+ 脚本化
  `KeyboardInput`；每帧调用 `Host::advance((f+1)*FRAME_NS, …)`（同时验证固定步折算=1 帧）。
- 确定性脚本：前 7 条 `ModeEvent`（`Black,Title,Menu,ShopArrival,ExitTransition,StageStart,
  MidCombat`，全部 game-core flow 已证边）按固定帧投递进入 `Combat`；按钮按 8 帧循环脚本
  （含 L+R / U+D 同持，验证宿主侧消解）；半程注入 2 条 `ApuWrite` 验证音频流真实消费。
- 打印（两次运行逐行一致）：逻辑帧数/推进帧数/present 次数/终态 mode+stage/宿主签名
  `Host::signature()`/画布 hash/合成输出 hash/音频消费计数/ROM 字节数。
- 退出码 0；无窗口、无网络、无用户目录写盘（Storage 不在自检里落盘；由单元测试覆盖）。

## 6. 未证域声明（fail-closed；实现侧不得顺手补齐）

| 域 | 状态 | v1 处理 |
|---|---|---|
| H-01 弹丸/武器槽语义、弹幕行为 | 未证 | 不画子弹、不接行为；`TODO(shell-v1)` |
| H-16 命中判定/受击伤害链 | 未证 | 不引入碰撞行为 |
| Boss / EN-9+ 段 | 未证 | 不 spawn（core `boss_segment_unproven` fail-closed） |
| 商店消费侧/经济闭环 | 未证 | flow 只到 `Shop` 状态 |
| 音效/音乐触发与 PCM 合成（AUD-2/3） | 未证 | `Audio` 只消费写流；PCM 静音占位显式；cpal 设备接入留后续 |
| 原版调色板映射 / BG nametable / 文本流 / 结局卡 | 未证 | CHR 背景=灰阶占位；不画 nametable/文本/结局卡 |
| 菜单输入门控/派发 | 机制已证、接线未定谳 | 不自动从按键转移；`ModeEvent` 只由测试/显式接口投递 |
| 关卡数据装载（脚本/调度/shape/初始槽） | 未接 | `StageState` 空；`stage_clear_requested` 不会自然发生 |
| 玩家射击/A 边沿投放接线 | 部分已证、未接 | `$16` 边沿由 game-shell 产出但不消费 |
| 死亡窗/GAME OVER 菜单自动接线 | 未接 | 不自动迁移 |
| 触摸虚拟手柄布局/连发、Android 后端细节 | 未证 | 只留 platform-shell.md §5/§6 清单 |
| 原版存档语义（电池存档/密码） | 未证 | Storage=字节块占位，不自动保存 |
| `StageStart` 段玩家可动性、死亡窗滚动 | 未隔离 | game-shell 已按模式门 fail-closed（本卡不重复实现） |
| L4 像素级一致 | 未达成（0 容差裁定） | 本卡不声称；平台侧合成是可见性占位 |

## 7. 文件级改动计划

| 文件 | 改动 |
|---|---|
| `crates/shell-win/Cargo.toml` | 增 `game-shell`/`game-data` 依赖；增 `[[bin]] name="shell-win" path="src/main.rs"` |
| `crates/shell-win/src/lib.rs` | 删 `FrameDriver`/`GameHost`；保留/修正 `map_key`/`storage_dir`/`view_rect`/`blit_scaled`；新增 `FrameRenderer`（Renderer 头less/合成）、`PcAudio`（Audio）、`FsAssets`（AssetSource）、`FileStorage`（Storage）、`KeyboardInput`（Input）、`LifecycleQueue`（Lifecycle）、`run_selftest`；lib 内单测锁语义 |
| `crates/shell-win/src/main.rs` | 新增：`[[bin]]` 入口；`--selftest-frames N` 分支；winit 0.30 `ApplicationHandler` + softbuffer 呈现实装（`WindowRenderer`） |
| `crates/shell-win/tests/smoke.rs` | 重写为 v1 契约测试（映射/路径/缩放/trait 实现/自检确定性） |
| `docs/spec/platform-shell.md` | **仅 §4 状态回填**（+ 顶部实施状态行同步为事实） |
| 本文件 | 实现后 §9 回填 |

**明确不改**：`crates/game-core/**`、`crates/game-platform/**`、`crates/game-shell/**`、`rom/`、
`reference/`、`data/`、`gates/`；`crates/game-desktop/**` 不修改（仅复制可见性骨架语义到 shell-win，
不抽取反向依赖——`shell-win` 不依赖 `game-desktop`）。

## 8. 验收与复现命令（计划值；实测见 §9）

```sh
cargo build -p shell-win
cargo test -p shell-win
cargo test --workspace
python gates/cli.py check                      # CHECK: PASS
cargo run -p shell-win -- --selftest-frames 300          # 两次签名一致
cargo run -p shell-win -- --selftest-frames 300
# GUI 手动验收（非自动化）：窗口标题 gunnac-remake (Stage A)、整数倍缩放黑边
cargo run -p shell-win
```

**完成判据**：上列命令全绿；selftest 两次输出签名逐行一致且退出码 0；
platform-shell.md §4 状态与实现相符（不声称未实现能力）；独立对抗复核 verdict=pass。

## 9. 实施状态（2026-10-18 回填）

**结论**：**v1 交付完成**——`shell-win` 从 177 行库骨架变成可运行 PC 宿主：
六 trait PC 实现 + winit/softbuffer 主循环（经 `game_shell::Host::advance` 固定步 60Hz）+
`--selftest-frames` 无窗口确定性自检；旧 `FrameDriver`/`GameHost` 重复宿主逻辑已删除。
**未证域全部 fail-closed/TODO**（§6；§9.3 逐条列）。

**§9.1 commit 列表（本任务写范围）**

| commit | 主题 |
|---|---|
| `1be66e5` | `docs(platform)`：本方案（对照/迁移/呈现/自检设计 + 未证域声明） |
| `7b5e4da` | `feat(platform)`：shell-win v1 实现（Cargo/lib/main/tests；六 trait + 主循环 + 自检） |
| 本次 docs commit | `docs(spec)`：platform-shell.md §4.1/§4.2 状态回填 + 本 §9 回填 |

**§9.2 门禁与实测输出（本机 Windows，2026-10-18）**

| 验收项 | 实际命令 | 结果 |
|---|---|---|
| 编译 | `cargo build -p shell-win` | 成功（0 error；仅既有 game-core `scroll_frames` dead_code 警告） |
| 单元/集成测试 | `cargo test -p shell-win` | lib 3 + smoke 17 = **20 passed / 0 failed** |
| 全量 | `cargo test --workspace` | 全绿（rust-tests 门禁实测 338，基线 326） |
| 总闸 | `python gates/cli.py check` | **CHECK: PASS**（PASS=11 FAIL=0 SKIP=1[addr-bridge] TOLERATED=0；rust_test_count ratchet WARN 见 §9.4） |
| 自检（1） | `cargo run -p shell-win -- --selftest-frames 300` | 退出码 0，输出见下 |
| 自检（2） | 同上第二次 | 退出码 0，stdout 与第一次**逐行完全一致** |
| GUI 启动 | `target/debug/shell-win.exe` | 进程存活；`MainWindowTitle`=`gunnac-remake (Stage A)`；stderr `mode=Boot stage=1 frame=1`（宿主状态真实消费） |

自检 stdout（两次一致；`host_signature`/`canvas_signature`/`display_signature` 仅作确定性回归锚，
**不**声称等价真机 trace）：

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

**§9.3 交付文件清单**

- `crates/shell-win/Cargo.toml`：`[[bin]] shell-win` + `game-shell`/`game-data` 依赖（平台依赖仍只在本 crate）
- `crates/shell-win/src/lib.rs`：PC 六 trait 实现（`FsAssets`/`FileStorage`/`PcAudio`/`KeyboardInput`/
  `LifecycleQueue`/`FrameRenderer`）+ `ChrBackground`/`compose_canvas`/`view_rect`/`blit_scaled` +
  `run_selftest`；删除 `FrameDriver`/`GameHost`，`AudioSink`→`PcAudio`
- `crates/shell-win/src/main.rs`：winit 0.30 `ApplicationHandler` + softbuffer `WindowRenderer`；
  `Host::advance` 固定步主循环；`--selftest-frames N`
- `crates/shell-win/tests/smoke.rs`：20 项契约测试（键位/路径/缩放黑边/CHR 合成/六 trait/自检确定性）
- `docs/spec/platform-shell.md` §4.1/§4.2：状态回填 + 自检记录值

**§9.4 遗留项 / TODO（未证域，未实现/未声称）**

1. **菜单输入门控未接线**：主循环不自动投递 `ModeEvent`（菜单派发边集未定谳）⇒ 窗口启动后停在
   `Boot`，按已证坐标显示玩家块；推进到战斗只能经 `Host::apply_event`（测试/后续已证战役）。
2. **敌精灵/子弹/HUD/文本/结局卡不画**：game-shell v1 渲染契约未暴露（`FrameView` 只有玩家与活弹数）；
   `TODO(shell-v1)` 已登记；H-01/H-16 未证不发明。
3. **音频 PCM=静音占位显式**：`PcAudio` 真实消费 `ApuWrite` 流（计数/写序），`synth_pcm` 全 0；
   cpal 设备流与 APU 合成器（AUD-2/3）属后续任务卡。
4. **关卡数据未装载**：`StageState` 空 ⇒ `stage_clear_requested` 不会自然发生；自检用已证
   `ModeEvent` 脚本推进，不冒充数据装载。
5. **存储不自动保存**：`FileStorage` 只做字节块存取（原版存档语义未证）；主循环未接保存时机。
6. **CHR 背景为可见性占位**：灰阶 `GREY4` 非原版调色板（C 级未证），不做 nametable/精灵优先级；
   L4 像素目标仍 0 容差攻坚（独立任务）。
7. **`shell-win`→`shell-pc` 更名**：待用户裁定（platform-shell.md §4 注；本任务不改名/不移动）。
8. **ratchet WARN**：`rust_test_count` 现值 338 > 基线 326（只升不降=WARN 非 FAIL）；
   `data/status/baselines/` 属本任务写范围外 ⇒ **报 Lead 处置**（用 `gates/cli.py baseline update --reason`）。
9. **GUI 视觉验收**：窗口标题/启动已实跑验证；整数缩放/黑边/CHR 背景像素的肉眼验收为手动项
   （本机无自动截图对拍；L4 门禁不覆盖 shell 呈现）。

**§9.5 需 Lead/用户裁定事项**

1. 是否将 `shell-win` 更名 `shell-pc`（平台矩阵口径）。
2. 菜单输入门控/主循环自动事件派发是否为下一张任务卡（决定窗口可玩性；当前 fail-closed）。
3. `rust_test_count` 基线上调（`data/status/` 写范围外，需 Lead/门禁属主执行）。
4. 独立对抗复核（verifier 角色）为本步骤完成前置——本产出在复核 pass 前只算"待审稿"。

