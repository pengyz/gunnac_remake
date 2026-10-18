# 对抗复核：敌槽可见化（已证坐标列渲染 + 命中事件）—— task-45 / task-111

> 复核方：`shell-verifier`（task-111，verifier）；被审：task-45 / `shell-porter`
> 被审版：工作树（HEAD `199b542`）+ task-45 未提交改动：
> `crates/game-shell/src/{enemy_coords.rs(新),host.rs,lib.rs,render.rs}`、
> `crates/game-shell/tests/enemy_coords_replay.rs(新)`、`crates/shell-win/src/lib.rs`、
> `crates/shell-win/tests/smoke.rs`、`docs/findings/plan-enemy-render.md(新)`、`docs/spec/platform-shell.md`
> 纪律：AGENTS.md §5（证伪立场，实施≠复核）+ 采集窗口约束：**未跑 `gates check/status/visual`、未跑 Mesen/scenario_run**；只跑定向 cargo test/selftest 与只读核验。
> **VERDICT：needs_revision** —— 证据合规/渲染契约/四签名/对拍数字（N=28/28、318 槽帧、两次 replay 全等、fail-closed）全部独立核实通过，GUI 实拍我**自跑复现为逐字节相同**；
> 但 **F1（必改，Low→Med）：`enemy_coords_replay` 的"类型集合断言（spec §4.2 步骤 2）"与实际实现不符**——只断言"rust 暴露坐标型 ⊆ `RENDER_TYPES={$2F}`"，未实现 spec 步骤 2/6 的 **`V1_TYPES={$2F,$2B,$4E}` 过滤后集合相等/反例保护**；`plan-enemy-render.md` §4 却写"spec §4.2 原样，未放宽"。需补断言或显式登记收窄口径（二选一，见 §6）。

## 1. 证据合规（最重要；只读核）

### 1.1 `$2F` leader x/y 证据（A）

- **trace**：`oracle/traces/gameplay-stage1.enemy.jsonl`（**143 行**、8f 采样 f1264-2400、`state/x/y` 各 28×1B hex）。独立解析：
  - slots 8-27 型集合 = `{$12:154, $1A:207, $1C:46, $2A:34, $2B:233, $2F:151, $4E:265}`（槽帧样本数）；
  - **`$2F` 共 151 槽帧样本、跨 108 帧、5 个槽（23-27，首现 f1344/1472/1600/1728/2176），全部 x/y = `(128,8)`**（unique set = `{(128,8)}`）✓ 与 plan "5 槽/多帧恒 (128,8)" 一致。
- **handler ROM/seed**：`bank02:$899B`（file `0x49AB`，seed raw 同址一致）为 handler 入口；handler 体内：
  `$89D5: A9 80`（LDA #$80）→ `$89D7: 9D E0 04`（STA $04E0,X）；
  `$89DA: A9 08`（LDA #$08）→ `$89DC: 9D FC 04`（STA $04FC,X）。
  ⇒ plan 的"handler `$899B` 含 `STA $04E0,X`/`$04FC,X`"成立（两条 STA 在 $89D7/$89DC），且**写值是字面量 `$80`/`$08`（A）**——比"trace 观测"更强：x=$80=128、y=$08=8 有指令级依据（Info-4：plan 现写"A 指令/B 归属"，建议补字面量证据）。
- **结论**：`(128,8)` 不是臆造常量，是 **trace 恒量 + handler 字面写值双证**；v1 用常量初始化可接受（但见 Info-4 的表述建议）。

### 1.2 实现是否"只用已证列" / fail-closed

| 检查 | 核验 | 结果 |
|---|---|---|
| 只初始化/暴露 `$2F` | `enemy_coords::has_proven_coords(type_) = type_ == 0x2F`；`init_proven_coords` 仅该型置 `(128,8)`+速度四元组 0；host `update_enemy_coords` 仅 8..28 槽、非 proven 每帧清 ready/onscreen | ✅ |
| 无 type_code 反推/预测 | 代码仅做型值相等判断；未用型值映射坐标；`position` 读取的是 `s.x/s.y`（由已证常量初始化/零速 motion_update 保持） | ✅ |
| `$2B/$4E` fail-closed | 非 proven ⇒ ready/onscreen=false ⇒ FrameView `position=None`；shell 仅 `Some` 画；单测 + replay 测试 fail-closed 断言均覆盖 | ✅ |
| `$12/$1A/$1C/$2A`、低槽 `$05/$07` | host 只遍历槽 8..27；这些型无坐标写者接线（不 spawn/不画）；FrameView enemy_slots 仅 8..27（低槽不暴露） | ✅（注意：oracle trace **实见** `$12/$1A/$1C/$2A`（154/207/46/34 样本），来自 wave/EN 内容层 W32 未解码；plan 表 §1 "坐标写者证据=无生成源" 易误读为"ROM 无生成"，见 Info-5） |
| 槽复用/清 0 | 非 proven 每帧清 ready/onscreen；type 变化/清 0 后重新初始化，不沿用旧坐标 | ✅ |
| 出屏 | `step_proven_coords` 走 A 级 `engine::motion_update`（零速⇒位置不变；出屏⇒停止暴露，不发明回收语义） | ✅ |
| FrameView 映射 | view 循环 `slot = 8+i`（i=0..19），数组 `enemy_coord_ready[slot]` 索引正确；`position=Some` 仅 ready∧onscreen | ✅ |

## 2. 对拍口径（自跑）

```
cargo test -p game-shell --test enemy_coords_replay -- --nocapture
→ C2 共同 (帧,型) 采样点 N=28 / ≤1px 通过=28
→ C2 $2F 已证坐标暴露槽帧数=318
→ test c2_enemy_proven_coords_match_oracle_columns ... ok
→ test result: ok. 1 passed; ... finished in 21.88s
```

- **两次 replay 全等**：测试内 `assert_eq!(first, second, "replay 两次独立运行…")` ✓（我复跑通过）。
- **fail-closed 断言**：非 `RENDER_TYPES` 型在采样帧上 `position` 必 None ✓；`$2F` 出现时坐标必 `(128,8)` ✓。
- **阈值**：共同采样点 ≥20、每点 |dx|/|dy|≤1、达成率 ≥90%——代码与 spec §4.2 步骤 3/5 **原值，未放宽** ✓；分母先算 total 再算 rate（spec"先冻结"语义）✓。
- **独立复算 oracle 侧**：`$2F` 首现槽=slot27@f1344；oracle 首序槽出现的帧数=38；Rust 侧共同采样点=28（因 Rust 暴露帧与 oracle 采样帧交集），与测试输出 N=28 一致（属于"共同点"定义，不是 oracle 全量）。
- **F1（必改）**：测试注释写"类型集合断言（spec §4.2 步骤 2）"，但实际只断言 **rust 暴露坐标型 ⊆ `RENDER_TYPES={$2F}`**；spec §4.2 步骤 2/6 要求：
  `types(oracle) ∩ V1_TYPES` 与 `types(rust) ∩ V1_TYPES` **必须相等**（`V1_TYPES={$2F,$2B,$4E}`），且 rust 不得出现 V1_TYPES 外已实现型；oracle 额外型须打印未实现清单。
  测试既未构造两流型集合、也未做相等/反例断言；`RENDER_TYPES` 也与 spec `V1_TYPES` 不同。**plan-enemy-render.md §4 的"口径（spec §4.2 原样，未放宽）"因此不成立**（阈值确实未放宽，但类型集合断言缺项）。
  影响：Rust 侧若漏 spawn `$2B/$4E`（或误 spawn 其它型）本测试不会发现；"已证坐标列对拍"只覆盖 x/y 值，未覆盖 spec 的类型集合验收项。

## 3. 渲染契约

| 检查 | 核验 | 结果 |
|---|---|---|
| 4×4 占位块 | `ENEMY_VISIBLE_PLACEHOLDER_SIZE=4`、唯一色 `[255,128,0]`；doc/常量均标"尺寸/图样/颜色未证= TODO(shell-v2)" | ✅ |
| `position=None` 不画 | `for slot in enemy_slots { if let Some((x,y)) = slot.position { fill_block(4×4) } }`；单测断言 None 处仍为背景色 | ✅ |
| 命中事件不改敌像素 | 绘制循环**不读取** `hits_this_frame`；事件仅入 `frame_view_digest`（`:734` 区域）与 Host 事件面；render 文档明示"不改敌像素、不发明命中特效" | ✅（Info-3：无直接"有/无 hit 事件画面一致"单测，静态可证；建议补 1 行断言） |
| 单测/smoke | shell-win lib `enemy_slots_draw_only_when_proven_position_present`（Some 画/None 不画）；smoke 集成断言 `$2F` pos 恒 (128,8)、非 `$2F` pos None、画布出现敌占位色像素 | ✅ |
| GUI 实拍可复算 | 实现方截图 `target/gui-shellwin-v2-enemies.png`：528×519、橙色 (255,128,0) **64px**、bbox **(264,47)-(271,54)**、client(8,31) 白——与 plan §6 完全一致 ✓。**我自跑 GUI**（PostMessage 推进至 Combat@frame912，等待 spawn 后 PrintWindow 采样）得到 `target/review-enemy/gui-enemy-probe0.png`：与实现方截图 **sha16 相同（`b787a2c2d59a5626`，39,428B）**、64px/同 bbox ✓ ⇒ GUI 证据可独立再生（逐字节）。 | ✅ |

## 4. 四签名不变（自跑）

`cargo run -q -p shell-win -- --selftest-frames 300` 连跑两次，stdout **逐字节相同**（SHA256=`FA5E87A3792FD410FBBAF0745714D05FC6E3D02787B0F77C062404F22CBA1A72`），四签名：

```
host_signature=0x68547665bf1ab396
canvas_signature=0x963ac9a776a57625
display_signature=0xe99af5849b632fbd
view_signature=0x15dbc2c7b0ba0a5d
```
= plan §5 记录值，**全部未变** ✓。理由如实：selftest 用已证事件边、不经转场装载（`stage_loaded=false`）⇒ 无 spawn ⇒ 敌槽 position 全 None、画布无敌占位块；plan 明确"不声称 selftest 覆盖敌槽" ✓。

## 5. 全量定向测试 / 写范围

- `cargo test -p game-shell -p shell-win` → **全部 test binary green**（58+2+1+3+1+3+14+22 等，`test result: ok`，exit 0；含 stg01_replay/stg01_combat 长跑）。
- 写范围：`git status` 中 task-45 相关 = game-shell（host/lib/render/enemy_coords.rs 新/tests 新）+ shell-win（lib/smoke）+ `plan-enemy-render.md`（新）+ `platform-shell.md`；**game-core / game-platform / data / gates 零改动** ✓（其它 M 文件属并行卡）。

## 6. Findings（四分类）

### 实现/工具（B）—— 无（核心实现正确；仅 F1 测试口径缺口）

- **F1（必改，Low→Med）对拍口径缺 spec 类型集合断言 / 文档声明失实**：
  - 位置：`crates/game-shell/tests/enemy_coords_replay.rs` 注释 L225（"spec §4.2 步骤 2"）与断言 L225-236；`docs/findings/plan-enemy-render.md` §4（"spec §4.2 原样，未放宽"）。
  - 二选一修复：
    (a) **补断言**：从 replay samples 收集 `rust_types`，与 oracle 的 `types` 各与 `V1_TYPES={$2F,$2B,$4E}` 取交集后 `assert_eq!`；并对 V1_TYPES 外已实现型 FAIL（spec 步骤 2/6）；若 Rust 当前 spawn 集合不满足，需先修生成/接线或登记豁免。
    (b) **显式登记收窄**：若本卡只承诺 `$2F` 坐标列（`$2B/$4E` 坐标 fail-closed、生成源未接），在 plan §4 与测试注释明确写"本卡口径=仅 $2F 坐标列；spec §4.2 类型集合相等断言未实现（差异已登记）"，并交 Lead 认可；不得再写"spec §4.2 原样"。
- **Info-1**：N=28 为实测值，测试仅断言 `total>=20`（未硬钉 28）；建议 `assert_eq!(total, 28)` 或把期望分母记入 plan/常量，防覆盖回退。
- **Info-2**："命中事件不改敌像素"无直接单测（当前由绘制循环不读 hits 静态保证）；建议补"有/无 hit 事件渲染结果相同"的 1 条断言。
- **Info-3**：handler 值证据可加强——`$89D5 A9 80`/`$89DA A9 08` 是字面量 A 级；plan 写"A 指令/B 归属"未体现值来源，建议引用 `$89D5-$89DC` 字节。
- **Info-4**：plan §1 表 `$12/$1A/$1C/$2A` 行"坐标写者证据=无生成源"易误读为"ROM 无生成"——oracle trace 实见这些型（154/207/46/34 样本），建议改为"无坐标写者证据（wave/EN 生成源未解码 W32）"。
- **Info-5**：spec C2 原设计的独立工具 `tools/enemy_coords_compare.py` + FrameView JSONL 导出未采用（改为 replay 内联测试）；实现更直接、但 trace 可追溯性弱于 spec 卡片，建议 plan 登记该实现方式变更。

### 证据/文档（E）：F1 已含；无其它阻塞
### 规格缺口（S）：F1(b) 的 scope 收窄若被接受，需在 spec/plan 双处登记（当前仅 plan 隐含）
### 平台差异（P）：无（GUI/无头测试均在 Windows 本机；未跑 Mesen）

## 7. 命令与输出摘要

| # | 命令/操作 | 摘要 |
|---|---|---|
| 1 | 解析 `oracle/traces/gameplay-stage1.enemy.jsonl` | 143 行；`$2F` 151 槽帧/108 帧/5 槽，全部 (128,8)；slots8-27 型集合 7 型 |
| 2 | ROM/seed `bank02:$899B` | entry bytes；`$89D5 A9 80`→`$89D7 9D E0 04`；`$89DA A9 08`→`$89DC 9D FC 04`（x=128,y=8 字面量 A） |
| 3 | `cargo test -p game-shell --test enemy_coords_replay -- --nocapture` | **N=28 / ≤1px=28**、`$2F` 暴露 318 槽帧、1 passed（21.88s）；两次 replay assert 全等；fail-closed 断言 |
| 4 | `cargo test -p game-shell -p shell-win` | 全部 test binary ok（exit 0；含 STG01 长跑/冒烟） |
| 5 | selftest ×2 | stdout 逐字节相同 SHA `FA5E87A3…`；host/canvas/display/view = 0x68547665…/0x963ac9a7…/0xe99af584…/0x15dbc2c7…（未变） |
| 6 | GUI 自跑（PostMessage→Combat@frame912→PrintWindow） | 截图与实现方 `gui-shellwin-v2-enemies.png` **sha 相同 `b787a2c2d59a5626`**；64px、bbox (264,47)-(271,54) |
| 7 | 独立像素解析实现方截图 | 528×519、64px、bbox 同、client marker 白 ✓ |
| 8 | 写范围 `git status`/`git diff --stat` | 仅 game-shell/shell-win/两 docs；game-core/platform/data/gates 未动 ✓ |

## 8. 结论

**needs_revision**：task-45 的**核心交付**（只渲染 `$2F` 已证 x/y；`$2B/$4E`/其余型 fail-closed；4×4 占位且 None 不画；命中不改像素；GUI/签名/对拍数字）**全部独立复核通过**，GUI 实拍甚至逐字节再生；唯一阻塞是 **F1：测试注释与 plan 声称 spec §4.2 口径原样/步骤 2 已实现，但类型集合相等断言实际缺失**。按 §6 二选一（补断言 或 显式登记收窄并经 Lead 认可）+ Info 建议后即可 **pass**，无需重跑 Mesen（建议重跑 `cargo test -p game-shell --test enemy_coords_replay` 与 selftest 确认无回归）。

> 本复核只写本文件；未改被审代码/文档/资产，未 `git add`/`commit`；未跑 `gates check/status/visual` 与 Mesen（采集窗口纪律）。复核方≠实施方（AGENTS.md §5）。

---

# 点验（2026-10-18，task-123）—— F1 收窄登记 + Info-1..5

> 被审：task-118 修正后的 `crates/game-shell/tests/enemy_coords_replay.rs`（**含 F1 收窄登记头**）、
> `docs/findings/plan-enemy-render.md` §4/§8、`docs/spec/platform-shell.md` §8（M）。
> 范围：只核 task-111 的 **F1（必核）+ Info-1..5**；其余结论 task-111 已通过，不重跑 Mesen/GUI。
> 纪律：未跑 `gates check/status/visual`、未跑 Mesen/scenario_run；只跑定向 cargo test/selftest 与只读文件核。
> **VERDICT：pass** —— F1 已按方案 (b) 在**测试/plan/platform-shell 三处**显式登记收窄口径且无失实残留；Info-1..5 全部落地；自跑 replay（N=28 钉死）、`-p game-shell -p shell-win`（smoke **23**）与 selftest ×2（四签名不变）全绿。

## 点验 §A：F1（收窄口径登记）—— ✅

| 检查 | 结果 |
|---|---|
| 测试文件 | `enemy_coords_replay.rs` 头部 L1-19：明写"**本卡收窄口径（显式登记，task-118 F1）：只对拍 `$2F` 坐标列**"；"spec §4.2 **步骤 2/6** 的 `V1_TYPES={$2F,$2B,$4E}` 过滤后**类型集合相等/反例保护断言未实现**（$2B/$4E 坐标写者未逐值审计、生成源未接；差异见 plan §4/§8）"；保留 fail-closed 断言与"步骤 6 方向性部分覆盖"说明 ✓ |
| plan §4 | L69 明写"**口径（task-118 F1 显式登记收窄；不得表述为 'spec §4.2 原样'）**"；L71-76 列已实现/已断言（N=28 钉死、1px/90%、fail-closed）；L77-80 "**未实现（差异登记，交 Lead 认可）**：spec §4.2 步骤 2/6 …类型集合相等断言未实现…由后续 C3/gen 卡覆盖"；L83-87 另登记 Info-5 实现方式变更 ✓ |
| platform-shell | §8 L317-325 新增"**对拍口径收窄登记（task-118 F1）**：`enemy_coords_replay` 只承诺 `$2F` 坐标列；spec §4.2 步骤 2/6 的 `V1_TYPES` 过滤后类型集合相等断言**未实现**（$2B/$4E 未审计、生成源未接；差异登记 …）"；§表格行（L113）同步标注"**收窄口径**…N=28（钉死；≥20）…类型集合相等断言未实现" ✓ |
| 残留 grep | 三文件 grep **"原样" = 0**；"未放宽"仅 `platform-shell:219` 的"**阈值未放宽**"（属合法阈值声明，非 spec 口径原样）；"步骤 2"仅出现在"**未实现**/步骤 3 计数定义"等正确语境；无"spec §4.2 步骤 2 已实现/原样"失实残留 ✓ |

## 点验 §B：Info-1..5 —— ✅

- **Info-1（N=28 钉死）**：`const EXPECTED_COMMON_SAMPLES: usize = 28`（L36）；采点后 `assert_eq!(total, EXPECTED_COMMON_SAMPLES, "共同采样点分母须=28…")`（L272-275），并**保留** `assert!(total >= 20)` 与 `rate >= 0.90`（L276-278）✓。
- **Info-2（命中不改敌像素）**：`shell-win/tests/smoke.rs::hit_events_do_not_change_enemy_pixels` 存在——构造同槽 `$2F` Some((40,50)) 的 base 与 with_hit，断言 `composed_canvas()` **逐字节相同**（"命中事件不得改变敌像素"）、`frame_view_digest` **不同**（"事件面可观测"）✓；smoke `#[test]` 计数 = **23**（自跑 smoke 23 passed）✓。
- **Info-3（handler 字面量）**：plan §1 `$2F` 行（L14/L27）补 **`$89D5 A9 80`→`$89D7 9D E0 04`（STA $04E0,X）、`$89DA A9 08`→`$89DC 9D FC 04`（STA $04FC,X）** 字面量 A 级证据 ✓（与我在 task-111 独立 ROM 读数一致）。
- **Info-4（`$12/$1A/$1C/$2A` 表述）**：plan §1 行改为"**无坐标写者证据**（wave/EN 生成源未解码 W32；**oracle trace 实见 154/207/46/34 样本** ⇒ 存在但坐标链未证）" ✓（与我 task-111 解析的 154/207/46/34 一致）。
- **Info-5（实现方式变更登记）**：plan §4 L83-87 登记"spec C2 原设计独立工具 `tools/enemy_coords_compare.py` + FrameView JSONL 导出**未采用**；改为 replay 内联测试（直接读 oracle trace、内存比较）；优点=零新增工具/依赖；缺点=trace 可追溯性弱于独立工具+导出产物（无独立 JSONL 中间物）；如需跨实现复用另卡" ✓；测试头 L16-19 同步 ✓。

## 点验 §C：自跑测试 / 签名

```
cargo test -p game-shell --test enemy_coords_replay -- --nocapture
→ C2 共同 (帧,型) 采样点 N=28 / ≤1px 通过=28；$2F 暴露 318 槽帧
→ test result: ok. 1 passed（27.01s）  exit 0        # assert_eq total=28 生效
cargo test -p game-shell -p shell-win
→ 全部 test binary ok；smoke 23 passed；exit 0
selftest ×2
→ stdout 逐字节相同 SHA=FA5E87A3792FD410FBBAF0745714D05FC6E3D02787B0F77C062404F22CBA1A72
→ host=0x68547665bf1ab396 / canvas=0x963ac9a776a57625 / display=0xe99af5849b632fbd / view=0x15dbc2c7b0ba0a5d（四签名不变）
```

- **写范围/门禁**：均只读/定向测试；三文件直接链接检查 plan 2/2、platform-shell 14/14 有效；`git status` 仍为 task-45 文件集（+ 本复核文件），无额外 tracked 改动；未跑 `gates check/status/visual` 与 Mesen ✓。

## 点验结论

**pass**：F1 已按方案 (b) 在测试/plan/platform-shell 三处**显式登记收窄口径**（只承诺 `$2F` 坐标列；spec §4.2 步骤 2/6 类型集合相等断言未实现、差异登记、交 Lead 认可），无"spec §4.2 原样/步骤 2 已实现"失实残留；Info-1..5 全部落地（N=28 钉死、hit 不改像素 smoke 23、字面量证据、`$12` 表述、实现方式变更登记）；自跑 replay/pkgs/selftest 全绿且四签名不变。**task-45 全部（代码+文档+复核）可提交**；建议提交后清理当时遗留的 `shell-win.exe` 进程。

> 本点验只追加本节到本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
