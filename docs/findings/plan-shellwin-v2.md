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
| `player_bullets_live: u16` | 槽分类计数（A 级 `PLAYER_ONLY_SLOTS`∪`SHARED_SLOTS`） | 不画；入 view 摘要 | **无坐标面**（H-01 未证）|
| `enemy_slots: [EnemySlotView; 20]` | 槽 8..=27；`$0400` 型值 A 级；坐标 **v1 恒 None**（写者链未证） | 仅 `position=Some && type_code=Some` 画 8×8 占位块（图样未证→`TODO(shell-v2)`）；`None` 一律不画；`type_code` **不反推坐标**、不命名语义 | 运行态 None⇒不画；单测用合成 view 覆盖 Some 路径 |
| `hud.money_digits: [(u8,bool);7]` | R-money-4 已证显示桁位（逐位左起/前导零跳位/至少个位） | **2×2 点阵占位块**：bit0..3 二进制编码 digit(0..9)；固定占位槽位；`show=false` 不画 | 字形/位图/布局未证 |
| `hud.bomb_ammo: u8`（`$018F`） | A 级 | **8 位位行占位**（8×2 像素，bit i→第 i 列）固定槽位 | 图标/文本未证 |
| `hud.stage: u8`（`$180`） | A 级 | **4 位位行占位**（4×2 像素）固定槽位 | 布局未证 |

**锁定口径**（Lead + platform-porter 已对齐，实施必须遵守）：
1. `position=None` ⇒ 不画（不得当 (0,0)）；
2. 子弹不画（无坐标面）；
3. HUD 只画数值域、显式占位编码，文档/汇报**不得声称**原版 HUD 像素/布局一致；
4. 模式画面只做占位区分；派发在 `Host::tick` 内，shell 只透传按键；`set_auto_flow(false)` 仅测试用；
5. view 摘要**平台无关**（仅 FrameView 字段；不混 renderer 尺寸/缩放/present 计数等平台态），
   既有 `host/canvas/display` 三签名语义不动。

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
       - 子弹：不画（TODO；仅摘要）
  4) 整数缩放/黑边 → softbuffer（v1 已有）
```

**帧内时序（避免覆盖层 1 帧滞后）**：`Host::advance()` 内部每逻辑帧执行 compose+present（present 计数
语义不变）；**推进>0 帧后** shell 取 `host.frame_view()`（本帧最新）→ `FrameRenderer::set_frame_view`
（存储 + 重合成）→ 窗口 `refresh()` 再上屏一次。呈现结果=最新帧 view；`present()` 计数仍=逻辑帧数
（refresh 不计数）。多帧追帧时中间帧使用上一 view 覆盖层、最终采样帧用最新 view——按样本收敛，文档登记。
**selftest** 同口径：每帧 `advance` 后 `set_frame_view(host.frame_view())`；报告 `view_digest` 取最终 view。

## 4. view 摘要签名（独立、平台无关）

`pub fn frame_view_digest(view: &FrameView) -> u64`：FNV-1a 64 over
`frame`、`snapshot::mode_tag(mode)`、`stage`、玩家 6 字段、`live_bullets`、`player_bullets_live`、
20×敌槽（slot/type_code/position）、HUD（7×(digit,show)、bomb_ammo、stage）。
**不混入**：窗口尺寸/缩放/present 计数等平台态。用途=`--selftest-frames` 新输出行
`selftest: view_signature=0x…`（两次一致）；**不改** `Host::signature()`。

## 5. 未证域声明（fail-closed；本卡一律不画/不发明）

| 域 | 状态 | v2 处置 |
|---|---|---|
| 敌坐标写者链（spawn 定位/行为 VM 位置写） | 未证 | `position` 恒 None⇒不画；`type_code` 不反推坐标 |
| 敌精灵图样/尺寸/调色板/优先级 | 未证 | 8×8 纯色占位块（类型值→占位色）；`TODO(shell-v2)` |
| 玩家子弹坐标/弹道（H-01） | 未证 | 不画；摘要只登记 `player_bullets_live` |
| HUD 字形/位图/屏幕坐标/文本/图标 | 未证 | 2×2 点阵/位行占位编码 + 固定占位槽；**非原版 HUD** |
| 原版 BG/nametable/调色板/文本流/结局卡 | 未证 | CHR 背景沿用 v1 灰阶占位；不新增 |
| 模式画面原版文字/美术/布局 | 未证 | 模式枚举→占位角标/纯色，不发明 |
| 菜单输入门控细节（含 B1 跨键吞按） | task-9 域/另卡 | shell 不实现派发；只透传按键 |
| L4 像素级一致 | 未达成 | 本卡不声称；仅提供可见性占位画面 |

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

**结论**：**实施中**——方案落盘；实现/门禁/复核结果待实现 commit 后回填（不改写 §0-§7 历史主张）。
