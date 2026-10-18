# 可玩闭环 A（平台无关）方案：菜单派发 + STG-01 数据装载 + FrameView 暴露 + 确定性跑关自检

> 任务卡：task-9（2026-10-18；前置 task-5 平台抽象层 v1 + task-1 PC shell v1 已落地）。
> 状态：**方案（实施前）**；实施后在 §10 回填状态（不改写本节历史主张）。
> 纪律：[AGENTS.md](../../AGENTS.md) §2/§3/§4/§9——未证不发明、证据 A/B 级才落结论、小步提交。
> 本次**不新增 game-core 接口**（结论见 §3）；若实施中发现必须，先报 Lead 批准再动。

## 0. 目标 / 非目标

**目标（平台无关侧闭环，使 STG-01 真正跑起来并可对拍）**

1. **菜单/模式派发**：把 Boot→Black→Title→Menu→Shop→Transition→StageStart→Combat 按已证序列接进
   `game_shell::Host` 主循环（输入驱动：`engine::InputLatch` + `engine::MenuInputGate` +
   `ui::ShopState::on_b` + flow 已证边集）。
2. **STG-01 装载**：用 engine/game-data 现有公开 API 构造并装载 `StageState`
   （调度游标 `decode_sched($A26D,43 记录)` / 脚本游标 `ScriptCursor::stg01` / 槽初始态 / 场景名），
   滚动源按 `stage::scroll_delta` 已证帧行为（A 级：与 `.scroll.jsonl` 逐帧 1:1，见 §6）。
3. **FrameView 暴露**：只读 view 供平台侧渲染合成（模式/关号/帧号、玩家、玩家子弹活弹数、
   敌槽原始型值与坐标状态、HUD 已证数值域）；未证域显式 `None`/TODO，不发明字段。
4. **确定性自检**：STG-01 3000 帧（覆盖 stage clear 前已证门）两次运行签名一致；输出敌人出生
   序列摘要；对拍列/口径逐列声明（§6），不得放宽阈值。
5. 文档回填 platform-shell.md「可玩闭环」边界。

**非目标**：不改 shell-win（task-10）、不改 game-core/game-platform、不做 Android、不做渲染图样
（task-10）、不实现未证域（H-01/H-16/Boss/商店消费/音效合成/调色板/文本）。

## 1. 层定位与动手前三问（AGENTS.md §4）

| 问 | 答 |
|---|---|
| 问题在哪一层？ | **平台无关宿主层（game-shell）**：v1 宿主空转（Boot 无派发、StageState 默认、FrameView 只有玩家）；数据/API 在 game-core 已备但未接线。不是 ROM 投影/code 语义问题。 |
| 消费者实际读哪个文件/数据？ | 宿主经 `AssetSource` 取得 ROM 字节（`ROM_ASSET_PATH`），从 **PRG bank8（`$A000` 窗）** 解 STG-01 数据；`StageState` 全部由 game-core 公开 API 构造；平台侧只消费 `Host` 的只读 view。 |
| 证据级别？ | 序列段/帧锚/输入锁存/滚动公式/转场尾/关始 intro/调度流=**A 级**（见 §4/§5 逐条指针）；引擎 tick 活跃段、spawn 落地次序、敌坐标写者链=**B 级或未证**（§6 显式声明）；无 C 级入实现。 |

## 2. 现有资产清点（复用，不重写）

| 资产 | 位置 | 用途 |
|---|---|---|
| `STG01_SCRIPT`(14 记录)/`ScriptCursor::stg01()` | `game-core::engine` | 脚本游标（已证数据） |
| `wave::decode_sched` / `SchedCursor::load` / `SchedItem` / `WAVE_FLAGS` | `game-core::wave`/`engine` | 调度流 `$A26D`（43 记录，A 级 golden） |
| `stage::scroll_delta(frame)` | `game-core::stage` | 逐帧滚动增量（f1193 −48 / f1194-1213 静止 / f1214+ −2 / f1318+120k −18） |
| `engine::land_pair` / `land_leader` | `game-core::engine` | spawn 写链落地（$400/$470/$518/$630/$534）；`land_leader` 另启滴灌首员 |
| `engine::InputLatch` | `game-core::engine` | `$18` 锁存（press@N → N+1f、保持 12f） |
| `engine::MenuInputGate` | `game-core::engine` | 菜单派发两层门（按下边沿 + 相位解锁参数化） |
| `ui::ShopState::on_b/on_start` | `game-core::ui` | 商店 B 切页/双 B 退出/Start 无效（R-shop-6/7/8） |
| `flow::FlowState::advance` + `ModeEvent` 边集 | `game-core::flow` | 模式迁移唯一入口（未证边 fail-closed） |
| `TRANSITION_TAIL_FRAMES=33`/`RUN_STAGE_INTRO_FRAMES=240`/`FLOW_TIMELINE_S5` | `game-core::engine` | 转场尾/关始 intro 常数与 S5 场景帧锚 |
| `stage_probe` | `game-core::engine` | 帧探针（tick/slot_types/live_shots/mode）——自检摘要用 |
| oracle traces | `oracle/traces/gameplay-stage1.{scroll,flow,enemy,stage-appear}.jsonl` | 对拍输入（只读） |

## 3. 最小接口需求结论（**无需改 game-core**）

- 构造 `StageState` 所需字段（`sched`/`script`/`scene`/`grid`/`wave`）均为 `pub`；
  `decode_sched`/`SchedCursor::load`/`land_pair`/`land_leader`/`InputLatch`/`MenuInputGate` 均为
  **现有公开 API**。逐项核对（实施前只读检查，结论：不需要新增 game-core 接口）。
- **新增 game-data 只读接口**（任务卡写范围允许）：`game_data::rom::prg_bank(rom, bank)`——
  iNES PRG bank 窗口（`file = 0x10 + bank*0x2000`）。理由：ROM 文件布局知识不应散落 game-shell；
  game-shell 只依赖纯解析 crate（需同步更新 `tests/contract.rs` 依赖白名单）。
- **不新增 game-core** ⇒ 无需 Lead 批准事项；若实施中发现缺口，暂停并按任务卡先报批。

## 4. 菜单 / 模式派发设计（输入驱动，接线口径逐条标级）

### 4.1 输入层（A 级）

- `$18` 锁存：press@N → 锁存 **N+1f**、保持 **12f**（`flow-response-latency.md` §1，A，10 press 全一致）
  ⇒ 直接用 `engine::InputLatch`：宿主把**按下沿**（A/B/START/方向）转成 `$18` 位后 `press()`，
  每帧 `tick_frame()`，取 `bitmap_18`。
- 位图（`$18`）：A=`0x80`、B=`0x40`、START=`0x10`、DOWN=`0x04`（A 级直读）；
  UP=`0x08`/LEFT=`0x02`/RIGHT=`0x01`（方向位沿用 `player-movement.md` R-move-1 A 级位序）。
- **标题段输入锁**：九班 `(450,550]`（A 级引证）⇒ `MenuInputGate.phase_unlocked` 在 frame>550 置真
  （相位计时源封存；用已证窗口边界，不发明相位表）。`MenuInputGate::dispatch` 负责层 1（仅按下沿派发）。

### 4.2 模式迁移表（全部走 `flow::FlowState::advance` 已证边；响应时延分级）

| 当前 Mode | 触发 | `ModeEvent` | 证据/级别 | 时效 |
|---|---|---|---|---|
| `Boot` | 逻辑帧 ≥180 | `Black` | R-ui-7 capture「boot-black f180」（A） | 帧锚 |
| `Black` | 逻辑帧 ≥360 | `Title` | R-ui-7 capture「title f360」（A） | 帧锚 |
| `Title` | START 派发（锁后） | `Menu` | R-ui-7（标题→菜单态）+ `MenuInputGate`（A/B 机制） | 锁存 +1f |
| `Menu` | START 派发 | `ShopArrival` | R-ui-6（菜单输入序列→商店 f1080）+ `$26:=$10` 转场旗 +13f（A） | 锁存+1f；**菜单→商店中间动画时长未证**=接线约定 |
| `Shop` | B 派发 | `ShopPage`→`ShopInfo` | R-shop-6（B 单击切信息页）+ `ShopState::on_b`（A） | 导航响应 +1f（A） |
| `ShopInfo` | B 派发 | `ExitTransition`→`Transition` | R-shop-7（两连 B 退出；press@1160→f1193 转场场景 +33f，A） | 锁存+1f；33f 用下一行 |
| `Transition` | 进入后 **33f** | `StageStart` | `TRANSITION_TAIL_FRAMES=33`（A 常数；分布未证） | 常数 |
| `StageStart` | 进入后 **240f** | `MidCombat`→`Combat` | R-ui-6 锚 f1260→f1500；`RUN_STAGE_INTRO_FRAMES=240`（A） | 常数 |
| 已证槽（本任务不接） | 死亡/GAME OVER/关末派发 | — | A2/A3 细粒度未接线（task-5 复核 S8） | 后续卡 |

> **口径（写进平台规格）**：主循环（可玩窗口）用上表输入驱动；其中 **帧锚 A 级**（180/360/33f/240f）
> 与 **响应时延 A 级**（+1f 锁存、+1f 导航）为已证；**菜单→商店中间动画时长（约 167f）未取证**
> ⇒ 主循环只保证**模式序列正确**，不声称该段帧级对齐。场景 replay 自检改用 capture 帧锚（§6）。

### 4.3 STG-01 装载时机

- 证据：滚动 trace 起点 **f1193**（`gameplay-stage1.scroll.jsonl`；与 `stage::scroll_delta` f1193
  装载 −48 锚一致，§6 逐帧验证）⇒ 关卡数据在 **ShopInfo→Transition 转场帧**装载。
- 宿主在 `ExitTransition` 成功的同一逻辑帧调用 `load_stg01()`（若 ROM 未装载 ⇒ fail-closed 不装载、
  返回 false 并登记）。
- `StageState` 构造（全部现有 API）：`scene="stg01"`、`script=ScriptCursor::stg01()`、
  `sched=SchedCursor::load(decode_sched(bank8, 0xA26D, 64))`、`grid.slots[0]=自机对象初态`
  （A 级 trace 快照：`gameplay-stage1.enemy.jsonl` f1264 槽0 `state=1/x=128/y=160`——与玩家
  复位锚点一致；仅此槽初始化，槽 1-7 语义未证不填）。
- `WAVE_FLAGS` 由 engine `wave_tick` 内 `register_wave_flags` 自动登记（现有行为，不重复实现）。

## 5. tick 序列扩展（宿主内）

每逻辑帧：

1. 输入采样 → 消解 → **`InputLatch`**（+1f 锁存）→ `MenuInputGate::dispatch`（仅标题/菜单/商店段消费）；
2. 自动帧锚模式推进（Boot@180/Black@360）；
3. 延迟事件队列（`VecDeque<(due_frame, ModeEvent)>`）到点投递（响应时延/33f/240f 用）；
4. **关卡引擎 tick**（`stage_loaded` 且 Mode ∈ `Transition|StageStart|Combat|DeathRespawn|StageClear`）：
   > 级别注：`Combat`=已证 gameplay 段；`Transition/StageStart`=**B 级接线依据**（装载后滚动在转场窗
   > 已推进=`.scroll.jsonl` f1193 起 A 级；出敌在 stage-start 窗内发生=`stage-appear` f1278∈[1260,1500)）；
   > `DeathRespawn/StageClear`=接线约定（G-S5-5 细粒度未隔离，platform-shell §8）。
   - 滚动源：`abs = STG01_LOAD_FRAME(1193) + (host_frame - stage_load_frame)`；
     `scroll_delta = -stage::scroll_delta(abs).1 as u16`（0/2/18/48，A 级逐帧公式）；
   - `engine::stage_tick`（帧序=行为→碰撞→主环不变）；
   - **消费 `StageEffects`**：`spawn_events` 逐项落地（`Pair→land_pair`；`Spawn→land_leader(...,None)`；
     其余 item 不落地=fail-closed）；`drip_events` 同规则（组长落地后由 engine `drip_step` 产员）；
   - 记录 `spawn_log: Vec<SpawnEvent>`（帧/槽/型值——对拍摘要用；**型值不命名**）；
   - `stage_clear_requested` 自动投递 `Combat→StageClear`（既有已证接线）。
   - **口径**：spawn 落地发生在 `stage_tick` 返回后同逻辑帧内；因 engine 未在 wave_tick 与
     behavior_frame 之间暴露 hook，组长首员在**下一帧** behavior_frame 才滴灌（相对 ROM 可能 −1f，
     未证；不声称帧级等价）。此为实现侧已知偏差，登记 §8。
5. 玩家移动：**`Combat` + `StageStart`**。
   > 【2026-10-18 task-9 更正（实施期 oracle 证据）】原方案写"StageStart 不跑"（保守假设"可动性未证"）；
   > 实施期对 `gameplay-stage1.player.jsonl` 复核发现 **f1301 首移**（x=128→130、held=$81），该帧落在
   > flow capture stage-start 窗 f1260→f1500 内 ⇒ 关始段玩家可动=A 级 trace 事实，故
   > `player_tick_active = Combat|StageStart`。C6 对拍 f1260..=1830 逐帧 571/571 一致（见 §10）。
   > `DeathRespawn` 窗仍冻结（R-respawn-2 A 级）、`Transition/StageClear` 不跑。
6. 渲染/音频/时钟：沿用 v1；`FrameView` 扩展见 §7。

## 6. FrameView 暴露（只读 view；字段逐条标来源/None 口径）

| 字段 | 类型 | 来源 | 级/口径 |
|---|---|---|---|
| `frame` | `u64` | `FrameClock` | 已证（确定性虚拟时钟） |
| `mode` | `Mode` | flow | 已证边集状态 |
| `stage` | `u8` | `$180` 关号 | A（E37C/初始化 bank00 证据） |
| `player` | `PlayerState` | `$04E0/$04FC/$0470/$048C/$018F/$0401` | A（R-move-1/players） |
| `live_bullets` | `u16` | `BulletGrid.state` 活槽计数 | A（槽几何 t48 定谳） |
| **`player_bullets_live`** | `u16` | `PLAYER_ONLY_SLOTS`∪`SHARED_SLOTS` 活槽 | A（t48 槽分类） |
| **`enemy_slots`** | `[EnemySlotView; 20]`（槽 8..27） | `ObjectGrid` | 型值 `$0400+槽`=A 地址面（**原始值不命名**）；坐标见下 |
| `EnemySlotView.type_code` | `Option<u8>` | `$0400+槽`（0=空槽⇒None） | A（原始型值，不命名） |
| `EnemySlotView.position` | `Option<(u8,u8)>` | `$04E0/$04FC+槽` | **v1=None**（坐标写者链未接线/未证；不得把 None 当 (0,0)） |
| **`hud.money_digits`** | `[(u8,bool);7]` | `ui::render_digits`（R-money-4） | A（显示位数/跳位规则已证；屏幕布局未证） |
| **`hud.bomb_ammo`** | `u8` | `$018F` | A（N-0017；图标/文本布局未证） |
| **`hud.stage`** | `u8` | `$180` | A（同上） |

- `Host::frame_view()` 公开只读；`compose_frame` 内部使用同一 view（v1 只画玩家占位块，其余不画）。
- **不暴露**：敌型语义名（未证）、敌坐标（写者链未证）、HUD 文本/图标布局（H-12 未证）、
  BG/nametable/调色板（未证）——显式留白，task-10 不得据此发明。

## 7. STG-01 装载与确定性自检

- `Host::load_stg01(&mut self) -> bool`：要求 `rom` 已由 `AssetSource` 装载；`game_data::rom::prg_bank(rom,8)`
  → `decode_sched($A26D, max=64)`（43 记录，truncated=false 断言）→ 构造 `StageState`（§4.3）。
- `Host::load_asset` 已有；self-test 用真实 ROM（tests 读 `../../rom/gunnac_cn.nes`，与 game-core 测试同口径）
  或 mock bytes（契约测试）。
- **f1193 帧锚对齐**：`STG01_LOAD_FRAME=1193`（A，scroll trace 起点）；宿主记录 `stage_load_frame`。
- **replay 自检**（`crates/game-shell/tests/stg01_replay.rs`）：
  1. `Host::new()` + `load_asset(真实 ROM)` + `load_stg01()`；
  2. 按 **capture 帧锚** replay 模式事件：f1080 Shop → f1200 Transition（+装载）→ f1260 StageStart →
     f1500 Combat（起点对齐 `.flow.jsonl` 已证帧锚；声明=replay 口径，非通用主循环规则）；
  3. 逐帧（f1194..=2400 或 3000 帧）推进引擎 tick（滚动源=§5，stage::scroll_delta）；
  4. 3000 帧（或至已证门 `stage_clear_requested`）两次独立运行：`signature()`/`FrameOutcome` 序列/
     spawn_log/`stage_probe` 全等；
  5. 输出 `mock-3000 signature` 与 spawn 摘要（首 10 + 总数）。
- **对拍列与口径（逐列声明，不放宽阈值）**：

| 列 | 对拍对象 | 口径 | 判定 |
|---|---|---|---|
| C1 滚动增量 | `.scroll.jsonl` `scroll_y` 逐帧差 vs `stage::scroll_delta` | f1193..2400，`delta_y = -(scroll_y(f)-scroll_y(f-1)) mod 256` 与公式逐帧比 | **可机器 1:1 断言**（已预验 1207/1207 一致） |
| C2 模式序列 | `.flow.jsonl`（Shop/Transition/StageStart/Combat） | 只断言**序列/顺序**与 capture 帧锚（replay 模式等帧）；主循环输入驱动不保证帧级 | 序列断言（帧锚限 replay） |
| C3 调度流 golden | `STG01_SCRIPT` 14 记录 + `$A26D` 43 记录 tick 锚（engine_tick 既有 golden） | 记录数/tick 首段 [1,33,49,65,81] | 可断言 |
| C4 敌人首现列 | `.stage-appear.jsonl` / `.enemy.jsonl` | **仅比较型值集合/槽序列**（帧列不可对拍：spawn 触发链含初生记录/生成器（H-01 未证）+behavior VM，引擎 v1 未接）；帧差与差异数如实报告 | 声明式部分对拍；不把未对上列当 0 差异 |
| C5 敌人坐标列 | `.enemy.jsonl` x/y | 引擎 v1 坐标写者链未接线（FrameView position=None） | **不对拍**（显式声明缺口） |

> 若 C4 型值集合/槽序列也无交集 ⇒ 如实记录"不可对拍"，不强行拟合/调阈值；根因登记未证域。

## 8. 未证域声明（本任务不实现/不声称）

| 域 | 状态 | 处理 |
|---|---|---|
| H-01 弹丸行为/武器槽语义 | 未证 | 不生成玩家弹（引擎现有 zero 活动）；只暴露活弹计数 |
| H-16 命中判定 | 未证 | 沿用 core `stage_tick` 管线，空/无弹时 inert；不新增 |
| Boss/EN-9+（`$A2D5+` 后段？） | 未证 | decode_sched max=64 截断（43 记录完整 STG-01 主段）；不发明后段 |
| **敌坐标写者链**（spawn 定位/behavior VM 位置写） | 未证 | FrameView position=None；不画/不发明 |
| **spawn 帧级对齐**（初生记录 loader/生成器调用门控） | 未证 | 只声明型值/槽序列部分对拍；帧差登记 |
| 死亡窗/GAME OVER/关末细段（A2/A3） | 已证未接线（task-5 S8） | 不接；`apply_event` 显式投递才动 |
| StageStart 玩家可动性 | ~~未证~~ **实施期定为 A 级（trace f1301 首移）** | 跑 `player::tick`（§3.5 更正注；C6 571/571 一致） |
| 菜单→商店中间动画时长、转场 33f 帧内分布 | 未证（S5-2 封存） | 主循环用已证常数/接线约定；不声称帧级 |
| 敌型语义名/图样、HUD 布局/文本、调色板/BG | 未证 | FrameView 不命名/不暴露语义；渲染图样=task-10 占位 |

## 9. 验收命令与提交计划

```sh
cargo test -p game-data          # 新增 rom 只读接口
cargo test -p game-shell         # 派发/装载/FrameView/3000 帧确定性
cargo test --workspace
python gates/cli.py check        # CHECK: PASS
# 平台侧回归（不改 shell-win，由 task-10 负责）：
cargo run -p shell-win -- --selftest-frames 300   # 仍确定性；若 host_signature 变化须记录新值+原因
```

提交计划（小步）：`feat(data)` rom 只读接口 → `feat(platform)` 菜单派发+模式时序 →
`feat(platform)` STG-01 装载+滚动源+spawn 消费 → `feat(platform)` FrameView 暴露+3000 帧自检 →
`docs(spec/plan)` 回填。

## 10. 实施状态（2026-10-18 回填）

**结论**：可玩闭环 A 交付完成（菜单派发 + STG-01 装载 + 滚动源 + spawn 落地 + FrameView + 3000 帧
确定性 replay 对拍）；**无需修改 game-core**（方案 §3 结论成立）；新增 game-data 只读 ROM 窗口接口。

**commit 列表（含并发归属留痕——内容正确、未重写历史）**

| commit | 主题 |
|---|---|
| `cce3bb1` | `docs(finding)`：本方案（任务卡要求先方案后实现） |
| `05e3135` | `feat(data)`：`game_data::rom::prg_bank` 只读窗口 + 边界测试 |
| `96415c1` | **task-9 game-shell 实现 + `tests/stg01_replay.rs` + Cargo.toml/Cargo.lock**（菜单派发/STG-01 装载/滚动源/spawn 落地/FrameView/signature 扩展/依赖白名单）。⚠ **归属偏差**：该 commit 由平台复审 docs 提交（Lead）并发捕获整个暂存区，message 未描述代码；内容完整、未丢失，Lead 已记录，不重提。 |
| `ddb1558` | **task-9 platform-shell §3 主体**（模块表/tick 序列/§3.1 可玩闭环 A/§8 未证域更新）。⚠ **归属偏差**：被 shell-porter task-18 复核收口提交并发捕获（其 commit message 未描述本内容）；内容完整。 |
| `docs(spec/finding)`（本回填 commit） | platform-shell §4.2/§8/§9 等 **剩余补差**（从 shell-porter `target/t9-platform-shell-final.md` 合并稿精确 pathspec 提交）+ 本 §10 |

> **并发事故纪律留痕**：并行施工下 `git add`+`git commit`（无精确 pathspec）会捕获他人暂存区/工作树。
> 本任务的两次归属偏差均由 Lead/shell-porter 主动通报并留痕（未 rewrite 历史）；后续纪律=
> `git add` 后核对 `git diff --cached --name-only`，或 `git commit -m … -- <精确 pathspec>`。

**实跑结果（本机 Windows，复现见 §9）**

- `cargo test -p game-data`：rom 窗口 2 例通过（含越界/短 ROM fail-closed）。
- `cargo test -p game-shell`：单元 41 + contract 2 + mock_platform 3 + stg01_replay 3 全绿。
- `cargo test --workspace`：exit 0（全绿）。
- `python gates/cli.py check`：**CHECK: PASS**（见交付汇报；rust_test_count 上升=基线待 Lead 上调）。
- `cargo run -p shell-win -- --selftest-frames 300` 两次逐行一致；`host_signature` 由
  `0xcde9dc99514b5f4b` → **`0xf51d5a517e7458b7`**（原因：`Host::signature()` 语义扩展——新增菜单锁存/
  门控/装载/spawn 日志/阶段计时字段覆盖；非回退，canvas/display 签名不变；shell-win 文档记录值由 task-10 更新）。
- **replay 3000 帧签名**：`host=0x521d397a6c609f34 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807`
  （两次独立运行全等；replay=f1193 装载、f1260 StageStart、f1500 Combat；engine_frames=3000-1193=1807）。
- mock-300 契约签名（task-5 门禁回归）：`host=0x63b5a6195287a6d8 canvas=0x963ac9a776a57625`
  （旧值 `0xefe39a9ba5e7c5b0`；原因=signature 语义扩展——新增菜单锁存/门控/装载/spawn 日志/阶段计时覆盖）。

**对拍结果（列口径按 §7，不放宽阈值）**

| 列 | 结果 |
|---|---|
| **C1 滚动列** | `stage::scroll_delta` vs `.scroll.jsonl`：**1207/1207 帧 0 失配**（A 级断言） |
| **C2 调度 golden** | `decode_sched($A26D)` = **43 记录**、tick 锚 `[1,33,49,65,81]`（A 级断言） |
| **C2 模式序列** | replay：`Shop(1080)→Transition(1193)→StageStart(1260)→Combat(1500)`，序列与 `.flow.jsonl` 一致（capture 帧锚口径）；主循环输入驱动序列同（单元测试断言 Boot→Black→Title→Menu→Shop→ShopInfo→Transition→StageStart→Combat） |
| **C6 玩家列** | `player::tick` vs `.player.jsonl` f1260..=1830（死亡窗剔除）：**571/571 帧一致**（A 级；含 StageStart 可动证据） |
| **C4 敌槽首现** | engine 21 事件（槽≥8: 20 事件/20 槽/3 型）；oracle trace 首现 16 槽/7 型；**共同槽 16/16**；engine 型集 ⊆ oracle 型集（0 未知型）。**帧列不可对拍**（spawn 触发链含初生记录/生成器 H-01 未证；trace 8 帧采样漏短命槽）——如实声明，不做阈值拟合 |
| **C5 敌坐标列** | **不对拍**（坐标写者链未接线；FrameView `position=None`） |

**偏差/扩展（相对方案）**

1. **StageStart 玩家可动**：方案原写"不跑"，实施期以 oracle 逐帧证据更正为 A 级并接线（§3.5 更正注；
   C6 571/571 佐证）。
2. **C6 玩家列**为方案 §7 表外**新增**对拍列（A 级、可机器断言），追加登记于此。
3. **engine tick 活跃段**扩展 `Transition|StageStart`（B 级依据，§5 级别注）；`DeathRespawn/StageClear`
   仍为接线约定。
4. **公开 API 增补**：`Host::load_stg01_from_bank8`（合成视界/门禁测试）、`apply_event_hooked`
   （显式投递含转场装载副作用）、`set_auto_flow`（replay 用 capture 帧锚）、`stage_abs_frame`/
   `stage_scroll_delta`/`spawn_log`/`frame_view` 等只读观测口；**未改 game-core**。
5. **game-data 依赖**：game-shell 依赖清单 +game-data（纯解析 crate）；`tests/contract.rs` 白名单同步。

**遗留缺口（未实现/未声称）**：spawn 帧级对齐与初生记录 loader（H-01/生成器调用门控未证）；敌坐标
写者链（spawn 定位/behavior VM 位置写）；Boss/EN-9+ 后段（decode 视界 43 记录主段之外）；死亡链/关末
细段自动接线（A2/A3 仍由 platform 显式投递）；HUD 布局/文本与全部渲染图样（task-10）；Audio 生产侧
（AUD-2/3）；Android。
