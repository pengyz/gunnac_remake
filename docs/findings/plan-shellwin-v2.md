# shell-win v2 方案（可玩闭环 B：FrameView 渲染 + 模式画面 + 渲染契约）

> 任务卡：task-10（前置 task-9 已通过 task-11 复核 pass）。
> 状态：**方案（实施前）**；实现完成后在 §8 回填状态（不改写本节历史主张）。
> 纪律依据：[AGENTS.md](../../AGENTS.md) §2/§3/§4/§9（未证不发明；证据 A/B 级；小步提交）+
> [platform-shell.md](../spec/platform-shell.md) §4/§8 + [plan-playable-loop.md](plan-playable-loop.md) §8。

## 0. 目标与非目标

**目标（可玩闭环 B，shell 侧一个逻辑步）**

1. `shell-win` 的呈现合成消费 `game_shell::Host::frame_view()`（task-9 已交付的**只读** FrameView）：
   - CHR 背景（沿用 v1 可见性占位）+ 玩家 8×8（game-shell `compose_frame` 已画，shell 只叠覆盖层）；
   - **敌槽**：仅 `position: Some(_) && type_code: Some(_)` 时画 8×8 占位块（v1 运行态 `position` 恒 None
     ⇒ 等价"不画"；**禁止用 `type_code` 反推坐标**）；
   - **玩家子弹**：**不画**（FrameView 无坐标面；H-01/L3 shot golden 未转正）——只在 view 摘要登记
     `player_bullets_live`；
   - **HUD**：只画已证数值域（`money_digits` 的 `(digit, show)` 显示桁位、`bomb_ammo`、`stage`），
     以**显式占位编码**呈现（2×2 点阵 / 位行），字形/位图/屏幕坐标/布局/文本**未证不发明**。
2. **模式画面**：`Mode` 枚举→占位标记（`mode_tag`→固定占位色 8×8 角标），Boot/Title/Menu/Shop/Stage
   等可见区分；**不发明原版文字/美术/布局**。
3. **自检/契约**：`--selftest-frames` 输出新增**独立 view 摘要签名**（仅 FrameView 字段 FNV，
   不改 `Host::signature()` 语义）；两次一致；smoke 增渲染合成断言（模式标记/敌槽 Some 路径/HUD 编码/
   view 摘要敏感性）；增"按键透传→Host::tick 内菜单派发→模式推进"集成测试（shell 侧只做透传）。
4. 文档：本方案 + [platform-shell.md](../spec/platform-shell.md) §4/§4.1/§4.2 状态回填；
   §4.2 按最终 HEAD 时点记录新签名值。

**非目标（本卡不做）**

- 不改 `crates/game-shell/**`（FrameView/菜单派发/签名语义属 task-9；B1 MenuInputGate 跨键吞按由
  platform-porter 另卡修）；若字段缺失按 fail-closed 不画 + §5 未证域表登记（需新增接口先报 Lead）。
- 不改 `game-core`/`game-platform`/`game-data`、`rom/`、`reference/`、`data/`、`gates/`、`tools/`。
- 不画：BG/nametable 真实内容、敌坐标（未证）、子弹、HUD 文本/图标/原版布局、结局卡、调色板真表。
- 不声称"像素级一致/原版 HUD 一致/可玩性达成"；L4 0 容差仍为未达成（另线攻坚）。

## 1. 层定位与动手前三问（AGENTS.md §4）

| 问 | 答 |
|---|---|
| 问题在哪一层？ | **平台呈现层**：game-shell 已提供只读 `FrameView`（task-9）；shell 缺的是把 view 画到平台画布上的合成（呈现细节属实现侧语义）。不是反汇编/数据表/core 语义问题。 |
| 消费者读哪个数据？ | `Host::frame_view()` 字段：玩家（已证 `$04E0/$04FC`）、敌槽（`$0400+槽` 原始型值，坐标 `None`）、`hud.money_digits`（R-money-4 显示规则结果）/`bomb_ammo`（`$018F`）/`stage`（`$180`）、`player_bullets_live`（槽分类计数）。游戏状态仍全部在 game-shell/core；shell 只读 view + 写像素。 |
| 证据级别？ | 字段级证级见 §2 表：玩家坐标/`$0400` 型值/`$018F`/`$180`/活弹计数=A 级；`money_digits` 显示规则=R-money-4 已证；**HUD 字形/布局、敌图样、坐标、弹道=未证（C/缺口）⇒ 占位或不做**。 |

## 2. 与 task-9 接口对照（`game_shell::FrameView`；字段全部 `Copy`/只读）

| FrameView 字段 | 来源/证级 | shell v2 处置 | 备注 |
|---|---|---|---|
| `frame: u64` | 虚拟时钟 | 入 view 摘要；不画 | |
| `mode: Mode` | flow 已证段 | **占位模式标记**（`mode_tag`→占位色 8×8 角标）；不做原版文字/美术 | 模式画面=可见区分，不声称还原 |
| `stage: u8`（`$180`） | A 级 | HUD 位行占位编码 + view 摘要 | 布局/图标未证 |
| `player: PlayerState` | `$04E0/$04FC` A 级 | 玩家 8×8 仍由 game-shell `compose_frame` 画；shell 不重复画 | 保持单一绘制源 |
| `live_bullets: u16` | 引擎活弹计数（A 级地址面） | 不画；入 view 摘要 | 全槽计数 |
| `player_bullets_live: u16` | 槽分类计数（A 级 `PLAYER_ONLY_SLOTS`∪`SHARED_SLOTS`） | 不画（绘制源=坐标面 `player_bullets`）；入 view 摘要 | 计数本身不定位 |
| `player_bullets: [Option<BulletView>;20]`（task-37） | **坐标面 A 级**：`BulletView{slot,x=$04EE,y=$055E,type_code=$063E}`；存活=弹网格空槽判 | **task-44：按已证坐标画 4×4 占位块**（图样/尺寸/颜色未证→`TODO(shell-v2)`）；型值不命名/不用于视觉；入 view 摘要 | 首次"玩家输出可见"；不发明弹型视觉 |
| `hits_this_frame: [Option<EnemyHitEvent>;4]`（task-37） | 事件面（命中管线可观测结果） | **不画布**；入 view 摘要 | 事件≠画面，避免发明命中特效 |
| `enemy_slots: [EnemySlotView; 20]` | 槽 8..=27；`$0400` 型值 A 级；坐标 **v1 恒 None**（写者链未证） | 仅 `position=Some && type_code=Some` 画 8×8 占位块（图样未证→`TODO(shell-v2)`）；`None` 一律不画；`type_code` **不反推坐标**、不命名语义 | 运行态 None⇒不画；单测用合成 view 覆盖 Some 路径 |
| `hud.money_digits: [(u8,bool);7]` | R-money-4 已证显示桁位（逐位左起/前导零跳位/至少个位） | **2×2 点阵占位块**：bit0..3 二进制编码 digit(0..9)；固定占位槽位；`show=false` 不画 | 字形/位图/布局未证 |
| `hud.bomb_ammo: u8`（`$018F`） | A 级 | **8 位位行占位**（8×2 像素，bit i→第 i 列）固定槽位 | 图标/文本未证 |
| `hud.stage: u8`（`$180`） | A 级 | **4 位位行占位**（4×2 像素）固定槽位 | 布局未证 |

**锁定口径**（Lead + platform-porter 已对齐，实施必须遵守）：
1. `position=None` ⇒ 不画（不得当 (0,0)）；
2. 玩家子弹**只画已证坐标面**（task-37 `player_bullets`）；图样/尺寸/颜色/弹道未证 ⇒ 占位块，不发明弹型视觉；`hits_this_frame` 不画布；
3. HUD 只画数值域、显式占位编码，文档/汇报**不得声称**原版 HUD 像素/布局一致；
4. 模式画面只做占位区分；派发在 `Host::tick` 内，shell 只透传按键；`set_auto_flow(false)` 仅测试用；
5. view 摘要**平台无关**（仅 FrameView 字段；不混 renderer 尺寸/缩放/present 计数等平台态），
   既有 `host/canvas/display` 三签名语义不动；task-44 起摘要含玩家子弹/命中事件字段。

## 3. 呈现管线（shell 侧合成，v2）

```
game_shell::Host::tick
  └─ compose_frame(canvas): 清屏(黑) + 玩家 8×8            ← 不改（task-9）
        ▼
shell FrameRenderer.present()
  1) display = CHR 背景（v1 占位，256×240 RGBA）
  2) 非清屏色 (0,0,0,255) 的 game-canvas 像素覆盖背景       ← v1 已有（玩家块）
  3) draw_frame_view_overlay(display, view)                ← **v2 新增**
       - 模式标记：8×8 角标（mode_tag→MODE_MARKER_COLORS）
       - HUD：money_digits 2×2 点阵 + bomb_ammo 8bit 位行 + stage 4bit 位行（固定占位槽）
       - 敌槽：position/type_code 均 Some 时 8×8 占位块（v1 运行态无）
       - 玩家子弹（task-44）：Some(BulletView) 按 `$04EE/$055E` 画 4×4 占位块
         （图样/尺寸/颜色未证 → TODO(shell-v2)；不发明弹型视觉）
       - hits_this_frame：不画布（事件面；仅入 view 摘要）
  4) 整数缩放/黑边 → softbuffer（v1 已有）
```

**帧内时序（避免覆盖层 1 帧滞后）**：`Host::advance()` 内部每逻辑帧执行 compose+present（present 计数
语义不变）；**推进>0 帧后** shell 取 `host.frame_view()`（本帧最新）→ `FrameRenderer::set_frame_view`
（存储 + 重合成）→ 窗口 `refresh()` 再上屏一次。呈现结果=最新帧 view；`present()` 计数仍=逻辑帧数
（refresh 不计数）。多帧追帧时中间帧使用上一 view 覆盖层、最终采样帧用最新 view——按样本收敛，文档登记。
**selftest** 同口径：每帧 `advance` 后 `set_frame_view(host.frame_view())`；报告 `view_digest` 取最终 view。

**每采样上屏次数（task-36 E3 收口 / S1-S2 口径注）**：窗口模式 `Renderer::present`（`Host::tick` 内）
只做合成+计数、**不 blit**；窗口上屏统一在 `refresh_view()`（最新 view）执行 ⇒ **每采样 1 次
softbuffer present**。收口前为 **2 次**（tick 内 blit 上一帧 view + refresh 再 blit 最新帧），属
**采样临时口径**（无可见错误，`task-12` E3）；`presents` 计数语义不变（=逻辑帧数），
`display_signature`/`view_signature` 均不变。

## 4. view 摘要签名（独立、平台无关）

`pub fn frame_view_digest(view: &FrameView) -> u64`：FNV-1a 64 over
`frame`、`snapshot::mode_tag(mode)`、`stage`、玩家 6 字段、`live_bullets`、`player_bullets_live`、
20×敌槽（slot/type_code/position）、**20×玩家子弹**（None/Some tag + slot/x/y/type_code，task-44）、
**4×命中事件**（None/Some tag + enemy_slot/bullet_slot/damage/fail_closed/frame，task-44）、
HUD（7×(digit,show)、bomb_ammo、stage）。
**不混入**：窗口尺寸/缩放/present 计数等平台态。用途=`--selftest-frames` 新输出行
`selftest: view_signature=0x…`（两次一致）；**不改** `Host::signature()`。

## 5. 未证域声明（fail-closed；本卡一律不画/不发明）

| 域 | 状态 | v2 处置 |
|---|---|---|
| 敌坐标写者链（spawn 定位/行为 VM 位置写） | 未证 | `position` 恒 None⇒不画；`type_code` 不反推坐标 |
| 敌精灵图样/尺寸/调色板/优先级 | 未证 | 8×8 纯色占位块（类型值→占位色）；`TODO(shell-v2)` |
| 玩家子弹**图样/尺寸/颜色/弹道**（H-01 余项） | 坐标面已证（task-37 `$04EE/$055E`）；视觉未证 | **task-44：按已证坐标画 4×4 纯色占位块**；不发明弹型视觉/弹道特效；`TODO(shell-v2)` |
| 敌命中事件（`hits_this_frame`）的画面表现（命中特效/闪光） | 未证 | **不画布**；事件仅入 view 摘要 |
| HUD 字形/位图/屏幕坐标/文本/图标 | 未证 | 2×2 点阵/位行占位编码 + 固定占位槽；**非原版 HUD** |
| **HUD 可见性/显示时机（模式门）** | 未证 | v2 **所有模式**（含 Boot/Title/Menu/Shop）恒画 HUD 数值域占位；**不声称原版显示时机/可见性一致**；不改绘制行为（task-36 E5） |
| 原版 BG/nametable/调色板/文本流/结局卡 | 未证 | CHR 背景沿用 v1 灰阶占位；不新增 |
| 模式画面原版文字/美术/布局 | 未证 | 模式枚举→占位角标/纯色，不发明 |
| 菜单输入门控细节（含 B1 跨键吞按） | task-9 域/另卡 | shell 不实现派发；只透传按键 |
| L4 像素级一致 | 未达成 | 本卡不声称；仅提供可见性占位画面 |

> 【摘要哨兵歧义（E4，task-36 已登记、不修）】`frame_view_digest` 对敌槽 `position=None` 使用
> `(0xFF,0xFF)` 哨兵，与未来真实 `Some((255,255))` 理论同摘要；v1 `position` 恒 `None`
> （坐标写者链未证）⇒ **无实际影响**。若未来接线真实坐标，需先给 None/Some 加 tag 区分
> （会改变 `view_signature` 记录值，须按新时点更新 spec §4.2）。

## 6. 文件级改动计划

| 文件 | 改动 |
|---|---|
| `crates/shell-win/src/lib.rs` | 新增 `frame_view_digest`、`MODE_MARKER_COLORS`、`draw_frame_view_overlay`（模式标记/HUD 占位编码/敌槽 Some 路径）、`compose_canvas_with_view`；`FrameRenderer` 增 `view` 状态 + `set_frame_view`；`SelftestReport` 增 `view_digest`；`run_selftest` 每帧回填 view；lib 单测 |
| `crates/shell-win/src/main.rs` | `about_to_wait`：advance>0 帧后 `frame_view()`→`set_frame_view`→`refresh()`；`WindowRenderer::refresh`（重合成+上屏，不计 present）；resumed 初始 view；`print_selftest` 增 view 行 |
| `crates/shell-win/tests/smoke.rs` | 渲染合成断言（模式标记/敌槽 Some/HUD 编码/view 摘要敏感性/黑边缩放）；按键透传→模式推进集成测试；selftest 报告含 view_digest |
| `docs/spec/platform-shell.md` | §4/§4.1 渲染内容行/自检行状态回填；§4.2 新 selftest 记录值（最终 HEAD 时点） |
| 本文件 | 实现后 §8 回填 |

**明确不改**：`crates/game-{core,platform,shell}/**`、`rom/`、`reference/`、`data/`、`gates/`、`tools/`。

## 7. 验收与复现命令（计划值；实测见 §8）

```sh
cargo build -p shell-win
cargo test -p shell-win                 # 含新增渲染/模式推进测试
cargo test --workspace
python gates/cli.py check               # CHECK: PASS
cargo run -p shell-win -- --selftest-frames 300   # ×2：stdout 逐行一致（含 view_signature）
cargo run -p shell-win                             # GUI 手动：模式角标/HUD 占位/黑边缩放；按键透传后模式推进（stderr 打点）
```

**完成判据**：上列命令全绿；selftest 两次一致且新增 `view_signature`；未证字段（敌坐标/子弹/HUD 字形）
确实未画或占位并标 `TODO(shell-v2)`；spec 状态与实现相符（不夸大"可玩/像素一致"）；独立复核 pass。

## 8. 实施状态（2026-10-18 回填）

**结论**：**实现完成，待独立对抗复核**（spec §4 状态回填待 task-29 的 `platform-shell.md` 未提交改动入库后
以精确 pathspec 补；见 §8.4）。

### 8.1 commit 列表（task-10 写范围）

| commit | 主题 |
|---|---|
| `a630666` | `docs(platform)`：本方案（接口对照/呈现管线/view 摘要/未证域声明） |
| `4a0672b` | `feat(platform)`：shell-win v2 FrameView 渲染（模式占位画面/HUD 数值域占位/敌槽 Some 路径） |
| 待补 | `docs(spec)`：platform-shell §4.1/§4.2 状态回填（阻塞于 task-29 并发写，见 §8.4） |

### 8.2 实测（本机 Windows，2026-10-18；HEAD `4a0672b` + 并发 task-29 工作树）

| 验收项 | 命令 | 结果 |
|---|---|---|
| 单 crate | `cargo test -p shell-win` | lib **12** + smoke **21** = **33 passed / 0 failed** |
| 渲染覆盖层 | lib 单测 | view 摘要确定性/敏感性、模式占位色、HUD 2×2/位行编码、敌槽 Some 门、present 计数语义 |
| 模式推进（集成） | `tests/smoke.rs::menu_dispatch_progresses_modes_with_platform_input_only` | 1100 帧按键透传→Host 内派发，实测序列 `Boot→Black→Title→Menu→Shop→ShopInfo→Transition→StageStart→Combat`（mode_tag 0..8），`stage_loaded=true`、`gameplay_tick>0`；shell 不实现派发 |
| 自检 | `cargo run -p shell-win -- --selftest-frames 300` ×2 | stdout 逐行一致、exit 0；`view_signature=0xf6603346bf818ffd`（新增行）；`display_signature=0xe99af5849b632fbd`（v2 覆盖层改变合成）；`canvas_signature=0x963ac9a776a57625` 不变 |
| 自检 host 签名时点 | 同上 | `host_signature=0x5d7156266d6ac19d`（=task-29 B1 修后的树口径；platform-porter 独立复现同值。值链：task-1 `0xcde9…` → task-9 `0xf51d…` → task-29 `0x5d71…`；**非回归**） |
| GUI 实跑 | `target/debug/shell-win.exe` + PrintWindow 截图 | 窗口标题 `gunnac-remake (Stage A)`；CHR 背景；**Title 模式黄色角标**（`MODE_MARKER_COLORS[2]`）；HUD 占位位行；整数缩放上屏（单元测试锁 2×/3×/黑边）。截图 `target/gui-shellwin-v2-printwindow.png`（可再生不入库） |
| GUI 模式推进 | 键注入 | **环境限制**：本会话 `SendKeys`/前台授权未送达 winit 窗口（stderr 仅自动锚 Boot→Black→Title）。**复核方（task-12）用 `PostMessage(WM_KEYDOWN)` 注入成功**并 PrintWindow 截图像素采样验证 StageStart 角标/HUD 位块 ⇒ 该链**可自动化**（E6）；后续 GUI 验收方法见 §8.5。键驱动推进以集成测试为证据、手动验收项。

### 8.3 未证域落实（与 §5 一致）

敌槽 `position=None`⇒不画（运行态 20 槽全 None；集成测试断言）；`type_code` 不反推坐标；玩家子弹不画
（仅 `view_digest` 登记计数，集成测试断言"计数变化不改像素"）；HUD 只画已证数值域且为占位编码
（点阵/位行，非原版字形/布局）；模式画面仅占位角标；未改 game-shell/core/platform。

### 8.4 待办

1. **spec §4 状态回填**：`docs/spec/platform-shell.md` 当前有 task-29 未提交改动（触发键分离行/签名值链），
   为避免再次误卷，待其入库后只补 §4.1 渲染内容/自检行 + §4.2 新值（精确 pathspec）。
   ——【2026-10-18 已收口】task-29 提交 `1700934` 后，本卡以精确 pathspec 补 §4.1 渲染内容（v2 覆盖层）/
   自检（view_signature）行、§4.2 新值（display `0xe99af5849b632fbd`、view `0xf6603346bf818ffd`、
   host 值链 cde9→f51d→5d71 时点口径）与 §4 未证/未接措辞（同一 docs commit）。
2. **独立对抗复核**（task-12）：本产出在复核 pass 前只算"待审稿"。
3. GUI 键驱动模式推进的手动验收（环境限制已如实登记；自动化方法见 §8.5 E6）。

### 8.5 task-36 复核 findings 收口（2026-10-18）

来源：`task-12` 复核（verdict=pass，非阻塞 findings E3-E6/S1-S2）。本卡只做小修/登记，**不改游戏行为、
不改已记录签名语义**。

| finding | 处置 | 证据/位置 |
|---|---|---|
| **E5** HUD 可见性/模式门未证 | plan §5 增行 + platform-shell §8 增条：v2 **所有模式**恒画 HUD 数值域占位；**不声称原版显示时机/可见性一致**；不改绘制行为 | 本文件 §5；platform-shell §8 |
| **E3** 双 blit/present | 窗口模式 `Renderer::present` **跳过 blit**（只合成+计数），统一由 `refresh_view()` 最新 view 上屏 ⇒ **每采样 1 次 softbuffer present**（旧=2 次，采样临时口径）；`presents` 计数语义不变（=逻辑帧数）、`display_signature`/`view_signature` 不变 | `crates/shell-win/src/main.rs` `WindowRenderer::present` 注释；本文件 §3 |
| **E4** 摘要哨兵歧义 | **仅登记不修**：`frame_view_digest` 对 `position=None` 用 `(0xFF,0xFF)`，与未来 `Some((255,255))` 理论同摘要；v1 None 恒真无实际影响 | `crates/shell-win/src/lib.rs` `frame_view_digest` 注释；本文件 §5 注 |
| **S1/S2** 2 次 present 采样临时口径 | plan §3 显式标注（收口前 2 次 / 收口后 1 次每采样），并注明 presents 计数语义不变 | 本文件 §3 |
| **E6** GUI 自动化验证方法 | 写入本节 + §8.2：**`PostMessage(WM_KEYDOWN)` 注入 winit 窗口 + `PrintWindow` 截图像素采样**（复核方已成功复现模式推进与角标/HUD 位块颜色）；后续 GUI 验收复用该方法（本卡不实现脚本） | 本文件 §8.2/E6；复核报告 `docs/findings/adversarial-review-playable-b.md` §2 |

**task-36 实测（本机 Windows，2026-10-18）**

| 验收项 | 命令 | 结果 |
|---|---|---|
| 单 crate | `cargo test -p shell-win` | lib 12 + smoke 21 = **33 passed / 0 failed** |
| 自检 | `cargo run -p shell-win -- --selftest-frames 300` ×2 | stdout 逐行一致、exit 0；四签名不变：host=`0x5d7156266d6ac19d` / canvas=`0x963ac9a776a57625` / display=`0xe99af5849b632fbd` / view=`0xf6603346bf818ffd`（E3 不改合成结果） |
| 全量 | `cargo test --workspace` | exit 0 全绿 |
| 总闸 | `python gates/cli.py check` | **CHECK: PASS** |

> **提交说明**：按 task-36 卡指示，本卡改动**不 git add/commit**（留在工作树），完成后回报改动与实测，
> 由 Lead/独立复核后统一提交。

### 8.6 task-44 玩家子弹渲染（2026-10-18 回填）

**来源**：task-44（前置 task-37 FrameView 弹丸字段复核 pass）。目标=首次"玩家输出可见"；只画**已证坐标面**。

| commit | 主题 |
|---|---|
| `3dc4333` | `feat(platform)`：shell-win 渲染玩家子弹（已证坐标面 4×4 占位块 + view 摘要扩展 + F4 注释更正 + 契约测试） |
| 本次 docs commit | `docs(spec/finding)`：plan §2/§3/§4/§5 + platform-shell §4 区回填（本 §8.6） |

**实现口径**
- `player_bullets: [Option<BulletView>;20]`（task-37：`slot`+`x=$04EE`/`y=$055E`/`type_code=$063E` A 地址面；
  存活=弹网格空槽过滤）⇒ `Some` 按坐标画 **4×4 纯色占位块**（`PLAYER_BULLET_PLACEHOLDER_RGBA`，
  唯一色便于 GUI 抽样）；`type_code` **不命名/不用于视觉**（仅入摘要）；图样/尺寸/颜色/弹道未证
  ⇒ `TODO(shell-v2)`，**不发明弹型视觉**。
- `hits_this_frame`：**不画布**（事件面；避免发明命中特效），仅入 view 摘要。
- `frame_view_digest` 扩展：玩家子弹 20 槽（None/Some 显式 tag + slot/x/y/type_code）+
  命中事件 4 条（tag + enemy_slot/bullet_slot/damage/fail_closed/frame）；仍平台无关、独立于 `Host::signature()`。
- F4：lib 顶部/`draw_frame_view_overlay`/main.rs 未证域注释里过期的「FrameView 无坐标面/子弹不画」
  已更正为「坐标面已证、视觉未证⇒占位块」。

**task-44 实测（本机 Windows，2026-10-18；HEAD `3dc4333`）**

| 验收项 | 命令 | 结果 |
|---|---|---|
| 单 crate | `cargo test -p shell-win` | lib **14** + smoke **22** = **36 passed / 0 failed**（本卡 +3：lib 子弹占位/摘要敏感 2 + smoke 实弹渲染 1） |
| 全量 | `cargo test --workspace` | exit 0 全绿 |
| 总闸 | `python gates/cli.py check` | **CHECK: PASS**（PASS=11 FAIL=0 SKIP=1 TOLERATED=0；期间一次 docs-links FAIL 归因并发未跟踪复核文件，对方落定后复跑 PASS） |
| 自检 | `cargo run -p shell-win -- --selftest-frames 300` ×2 | stdout 逐行一致、exit 0；`host=0x68547665bf1ab396`（task-37 语义不变）/ `canvas=0x963ac9a776a57625`（不变）/ `display=0xe99af5849b632fbd`（selftest 无关卡装载⇒不产弹，不变）/ **`view=0x15dbc2c7b0ba0a5d`**（摘要扩展，**非回归**） |
| GUI 实弹 | `PostMessage(WM_KEYDOWN)` 注入（Title→Menu→Shop→ShopInfo→Transition→StageStart→Combat）+ 按住 B（X 键）2s + `PrintWindow` 像素采样 | 到达 Combat（stderr `mode=Combat ... frame=1085`）；截图 `target/gui-shellwin-v2-bullets.png`（可再生不入库）中子弹占位色像素 **128px**、bbox `(264,99)-(271,232)`（窗口 528×519=客户区 2×）⇒ 子弹按已证坐标面真实上屏 |
| 黑边/缩放 | `view_rect`/`blit_scaled` 既有单测 | 保持（本卡未改缩放/blit 代码） |

**未证/边界（与 §5 一致）**：玩家弹**图样/尺寸/颜色/弹道**（H-01 余项）未证⇒占位块；`hits_this_frame`
不画布；敌弹/敌坐标/敌图样仍未接/不画。**不声称**任何原版弹型视觉或像素一致；L4 不适用。

