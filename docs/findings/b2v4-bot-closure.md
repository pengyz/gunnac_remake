# B2 v4 bot 闭环验证 —— 关 1 自然通关（负结果定谳 + 闭环 pass）

> 日期：2026-10-15（B2 v4 续做）｜ 角色：verifier（差分验证员收果）
> 背景：前实施体中断（基建故障）；工具面已完整落盘（tools/tas_bot.py 397 行 / tools/tas_bot.lua 603 行 /
> tools/test_tas_bot.py 9/9）+ bot 集成钩子（tools/scenario_capture.lua、tools/scenario_run.py）。
> 本轮职责 = **跑起来 + 收果 + 定谳**（不改策略实现）。
> 判据来源：[arena-midboss.md](arena-midboss.md) §5（bot 判据：瞄准 tol_x / 7 发伤害预算 / HP 差分逐发确认 /
> 复活重定位 / 逃逸走廊）+ [b2-tas-input.md](b2-tas-input.md)（v2/v3 轨教训：开环对复活不可观测 ⇒ record-to-scenario 闭环）。
> 硬红线：poke 禁用（通关必须自然）；不改 ROM/台账/门禁。证据分级依 AGENTS.md §3。

## 0. 验收对照（开工声明，未达不结案）

| # | 验收 | 状态 |
|---|---|---|
| ① | bot 运行结果定谳（通关或负结果+归因） | ✅ **负结果定谳**（§1）：4 死 → GAME OVER f8795，bot 停机 f9096；**未及中 boss**（tick 停 778 < 824 波次 1 装载线）；`$26` bit4 全程仅 f913-1192 菜单过渡窗（关末语义自然触发 = 未观测）。逐发归因见 §3，第四候选结论见 §4 |
| ② | 闭环对拍 verdict | ✅ **pass**（§2）：bot 实时决策 → inputs.jsonl 9096 帧 → record-to-scenario 纯 hold 开环轨 → 回放 → 9096 帧 × 28 watch 字段逐帧对拍 **0 mismatch**（`analysis/traces/tas-stg01-bot1.closure.json`）；确定性双证：同工具态独立两遍 bot run 逐帧 0 mismatch + 输入日志逐帧全等 |
| ③ | golden 回填（若通关） | **N/A（未通关）**：`python3 tools/tas_bot.py anchors tas-stg01-bot1` 九项锚全空（clear_gate/stg01-clear/flyout/gate300-open/inc-180/stg02-shop/stg02-exit/stg02-start/stg02-combat 均 "—"）；G-S5-6 injected 锚与台账 trigger 分级**不动**（自然 trigger 未产生，无替换对象） |
| ④ | 三件复跑不降 | ✅ cargo test --workspace **144**（基线 144）/ `tools/diff_subroutines.py` **L2-DIFF 121/121** / `gates/cli.py check` **PASS=12 FAIL=0 SKIP=0 TOLERATED=0（0 静默跳过）** / `tools/test_tas_bot.py` **9/9**（§6） |
| ⑤ | 顺手 3 advisory（复核 #81 a/b/c） | ✅ 全落地（§7）：arena-midboss §2.3③ "max 7" 补 "arena 段" 限定；§1.1 混叠表补 §7④ 口径指针注；§7② 窗口首行注（含对齐偏移实测核对） |

## 1. bot 运行结果（负结果定谳）

**命令**：`python3 tools/scenario_run.py run tas-stg01-bot1`（场景面自查：`oracle/scenarios/tas-stg01-bot1.txt`；
前缀 = gameplay-stage1 同源 f1-1200 + `bot stg01-clear from 1250` + 28 watch + probe_mem `$05DC/$064B` + capture_every 300）。

| 项 | 值 | 备注 |
|---|---|---|
| 场景 sha256 | `00598fcd849ac157…03b22c` | run.json 记录全值 |
| 生成 lua sha256 | `7f30d338bf837922…69efd19` | `analysis/_gen/tas-stg01-bot1.lua`（PLAN 前置 + bot 决策器 + 执行器整拼） |
| 实际帧 | **9096**（计划 30000，bot 提前收机：GAME OVER + 300f） | DONE `{"frames":9096,"bot_stop":true}`；wall 168.6s（run1）/162.9s（run2） |
| 死亡序列 | f5609 / f7045 / f8183 / **f8795**（pstate 15 起点，events.jsonl） | 4 死；残机时序 2→(f5610)1→(f5773 奖命)2→(f7046)1→(f8184)0→f8795 零残机死 |
| GAME OVER | f8796 `$26` bit3 置位（0 残机臂） | f9096 bot 停机（gameover+300f，`wants_stop`） |
| 时间线推进 | tick **778**（末帧 `$85:$84 = 3×256+10`） | **对照**：v2 轨 tick 824 = 波次 1 装载 f9689 = 中 boss 入场——本轨差 46 tick，arena **未到达** |
| 中 boss（type `$4A`=74） | **全程未出现** | `$041B`（slot27 占用者型码）全程谱 = 47/90/26/23/75/73/71/46/19/69/80…，无 74；`$064B` 观测值 0-5（普通小怪占用 slot27） |
| `$26` bit4（关末门） | **仅 f913-1192**（280 帧菜单→开 stage 过渡窗） | f>2000 零置位 ⇒ 关末语义自然触发 = **未观测**（与 v2 同口径判读） |
| `stage_no` | 恒 1 | `INC $180` 未发生 |
| pstate 直方图 | {1: 7218, 108: 987, 15: 512, 0: 376, 124: 3} | 战斗活跃 7218f（v2 对照 9592f）；死亡窗 512f = 4×128f |

**负结果主干**：本工具面（当前落盘态）**未能自然通关关 1**，且未及内容线末段——差额在**前段生存**（f5609 起每
~1300-1900f 一死，第 4 死耗尽残机）与**时间线推进不足**（tick 778 < 824），中 boss 击杀判据（HP 14→0 → `$26` bit4
自然置位）**无样本可验**。死因按 v2/v3 同源口径不发明：f8183 帧 slot27 占用者 (28,200) 距玩家 (14,205) 15px = 接触带
（§5.3 判据 ~17px）——接触死候选；其余 3 死近距对象不在 watch 面（槽 8-26/敌弹未接线，probe 300f 粒度），
**不宣称死因**（想坐实需 v2-diag 式 30f 快照，非本轮验收项）。

**工具态史（诚实登记，B 级推断）**：`tools/tas_bot.lua` mtime 13:15:54 晚于前实施体 12:39 的首跑（其产物已归档
`analysis/traces/.archive/tas-stg01-bot1-20261015-131847/`，生成 lua sha `bc08426d…` ≠ 当前 `7f30d338…`）⇒
**旧 run 与本 run 不同工具态、不可混用**（输入轨首分歧 f4240，属预期差异）。行为差异可考：旧 Lua 无 R-bot-4b
炸弹探针（events 无 bomb 类），现行 Lua 有（本 run f4244 bomb_probe 1 次）——具体改动内容不可复原（未入库、
旧生成 lua 已被覆盖），仅登记差异事实。**本文件一切结论以当前工具态为准**，其确定性与闭环由 §2 双证。

## 2. 闭环验证（record-to-scenario 闭环，verdict = **pass**）

口径（B2 §7 教训）：bot 实时决策（只读 RAM→按键）→ 逐帧录制 `inputs.jsonl` → `tas_gen record-to-scenario`
转写**纯 press/hold 开环轨**（生成物无条件指令、无 poke）→ `scenario_run` 回放 → 与 bot run 逐帧 watch **全字段**
状态对拍（两遍一致才算闭环成立）。

| 环 | 结果 | 证据 |
|---|---|---|
| 录制 | `analysis/traces/tas-stg01-bot1.inputs.jsonl` 9096 行逐帧按键 | run2 产物（run1 同名归档） |
| 转写 | `oracle/scenarios/tas-stg01-bot1-replay.txt`（record-to-scenario 生成 + 同源 watch/probe 面 + 锚帧 capture） | 纯 hold 轨；`--skip-run` 可单验生成物 |
| 回放 | `tas-stg01-bot1-replay` run 9096 帧 DONE | `analysis/traces/tas-stg01-bot1-replay.run.json` |
| **对拍** | **9096 帧 × 28 字段 = 0 mismatch**，`verdict=pass` | `analysis/traces/tas-stg01-bot1.closure.json`（mismatch_total=0, frame_count_equal=true） |
| 确定性（补充双证） | 同工具态独立两遍 bot run（run1/run2）：9096 帧 × 28 字段 **0 mismatch** + inputs.jsonl **逐帧全等** | `analysis/traces/tas-stg01-bot1-det.json`（compare verdict=pass）；`analysis/traces/tas-stg01-bot1-r1.*` 为 run1 快照 |

⇒ **闭环成立**：录制轨回放与实时决策 run 逐帧一致；bot 决策环在同一工具态下确定性可复现（两遍输入日志逐帧全等）。

## 3. 逐发归因（`python3 tools/tas_bot.py attrib tas-stg01-bot1`）

自机弹 **1198** 发（events.jsonl 逐出生/消失帧配对 × 框相交 × HP 差分）：

| 类 | 计数 | 说明 |
|---|---|---|
| boss_confirmed（框相交 + HP−2 逐发确认） | **3** | 下表 |
| boss_no_damage（框相交 + 0 差分） | 49 | **不构成第四候选证据**（分解见下） |
| object_slot8..27（撞普通对象槽） | 169 | 分散 20 槽（slot26=33、slot8=18、slot25=18…） |
| offscreen（出画） | 193 | 判据 x≤2/x≥252/y≤2/y≥250 |
| unknown | 784 | **显式缺口**：死因未在观察面（die y≤8 顶缘清弹 877 发中 777 无框重叠；寿命/动作倒数 `$040E+Y` 与槽 1-7 对象未接线——不发明分类） |

**boss_confirmed 3 行（HP 差分逐发确认，§5.2 判据）**：

| 弹出生→消失 | 出生→落点 | slot27 占用者位置 | ΔHP |
|---|---|---|---|
| f3876→f3890 | (77,187)→(77,70) | (88,57) | **2**（type $5A 3→1，f3890） |
| f4730→f4742 | (212,187)→(212,88) | (224,77) | **2**（type $49 4→2，f4729） |
| f8424→f8426 | (194,186)→(194,177) | (197,167) | **2**（type $49 2→0，f8426） |

**boss_no_damage 49 行分解（防误读）**：
- 46 发 die y=0-8（**顶缘出画清弹**，与 slot27 顶缘倒数敌 type `$2F` @(128,8) 框近似相交）——该敌 `$064B`
  实测 5→0 每 10f 等步长（**倒数语义非 HP**，W18 同族）⇒ "0 差分" 无判别力；
- 3 发中屏（f4555→4570、f4611→4625、f8445→8448）：形表缺省框近似（BOX_DEFAULT 21×21）/ 消失帧落在目标
  HP 已 0 的尸体上下文（f8426 已 2→0）——几何近似与一帧运动滞后（弹速 ~8px/帧，消失位置=末见帧位置）混杂。
⇒ 本 run 的 boss_no_damage 类**无一**是"同框相交 + 0 差分 + 目标可伤"的干净样本。

## 4. 第四候选（受击保护/命中冷却）结论：**不坐实、不排除——显式挂起（带首个正向签名）**

判据背景（arena-midboss §2.3 收口请求）：三解已收敛"命中覆盖率"单因后，第四候选"受击保护/命中冷却"
此前"未正式排除"，升 A 需逐发归因 + `$05DC` 写点探针。本轮数据：

| 面 | 数据 | 读法 |
|---|---|---|
| 当前 run（工具态权威） | **无中 boss 样本**（未到达 arena，§1） | 对第四候选**无判别力** |
| stale 工具态旁证（12:39 run，sha `bc08426d…`，仅参考不入结论基线） | arena f9916 中 boss HP14 入场 → **f10068 常规弹命中 −2**（arena 窗口 f9900-10300 A 键 0 按压 = 无炸弹介入）→ HP 12 冻结至 f10800；**f10135-10202 共 18 发**自机弹在 boss 框内消失（`hit_slot=27` 同帧唯一重叠，槽 8-26 均不重叠）且 **HP 全程 0 差分** | **正向签名首次出现**（"同框消费 + 0 伤害"） |
| 对照组（同 stale run） | type `$50` 敌 f9214/f9220 两发同框短命弹**各 HP−2**；type `$49` f4563/f4601 同法 −2×2 | 同签名=正常伤害路径存在 ⇒ 中 boss 侧 18×0 差分非"机制不可伤弹" |
| 混杂因子（坐实拦路虎） | 18/18 全落 f10135-10202 **贴身窗**：boss y 161→207 压入玩家出生带（弹出生 y≈206-223 即框内，寿命仅 1-5f），紧随 f10203 接触死亡（boss (102,207) vs 玩家 (103,223) dy=16） | "出生重叠窗/接触窗语义"（弹出生即被框消费但不走伤害路径）与"受击保护/命中冷却"**不可分** |

**定谳**：第四候选**不坐实**（无干净判别样本——全部正向签名被贴身窗混杂污染），**亦不可排除**（对照组证明
同签名可对应伤害；中 boss 18 发 0 差分无其他解释已被逐发排除召唤物吸收——`hit_slot` 同帧扫描槽 8-26 无重叠）。
维持 arena-midboss §2.3 "未正式排除" 状态并**升级优先级**。**坐实/排除判别实验**（下一步，非本轮验收）：
中距对准（boss y≈110-150、非接触窗）单发 ×10 + 逐帧 watch `$064B/$05DC/$05F7/$04DF/$04C`（现 probe_mem
300f 粒度不足判冷却窗）；若同框消失 + 全字段 0 变化连续出现 ⇒ **坐实**；若逐发 −2 ⇒ **排除**。

## 5. 反作弊自查（无 poke / 通关必须自然）

| 检查 | 结果 | 复现 |
|---|---|---|
| 决策器写能力 | **零写调用**（唯一命中 = 文件头纪律注释本身） | `grep -nE 'emu\.write\|writeMemory\|poke\|memory\.write' tools/tas_bot.lua` |
| bot 场景 poke | 场景文件零 poke 行；解析器 **fail-closed**（bot+poke 同场景 → `ScenarioError: bot 场景禁 poke`，实测拒绝） | `tools/scenario_run.py` bot 校验段（纪律锁） |
| 生成 lua 写路径 | `PLAN["pokes"]={}`（空）⇒ 执行器固有 poke 写点（scenario_capture.lua 公共能力）**不可达**；PLAN.bot = `stg01-clear from 1250` | `analysis/_gen/tas-stg01-bot1.lua` PLAN 行直读 |
| `$26` bit4 来源 | 全程仅 f913-1192 过渡窗自然出现（280 帧），与 v2 盲开环同口径；**无注入** | frames.jsonl 逐帧 `flags26` |
| 炸弹使用 | 本 run 仅 f4244 bomb_probe 1 次（R-bot-4b 机制探针，游戏内 A 键=自然输入，非 poke） | inputs.jsonl A 键行 |

## 6. 三件复跑（验收④，2026-10-15 实跑）

```
cargo test --workspace        → 144 passed / 0 failed（基线 144 不降；gates ratchet:rust_test_count 144/144 OK）
python3 tools/diff_subroutines.py → L2-DIFF: 121/121 通过（golden==ROM 执行==game-core 同名函数）
python3 gates/cli.py check    → SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0；CHECK: PASS（0 静默跳过）
python3 tools/test_tas_bot.py → tas_bot 闭环驱动回归: 9/9 通过
```

## 7. 顺手 3 advisory（复核 #81 残余观察，验收⑤）

| # | 落点 | 内容 |
|---|---|---|
| a | [arena-midboss.md](arena-midboss.md) §2.3③ | "并发 max 7" 补 **"arena 段"** 限定 + 注（全轨快照 max 实为 19 @ f6870，防跨口径误读） |
| b | [arena-midboss.md](arena-midboss.md) §1.1 混叠表注 | 补 §7④ 口径指针注：脚本只输出三汇总数（213/197/16），逐槽展开 = 同口径按槽过滤计数，此处略 |
| c | [arena-midboss.md](arena-midboss.md) §7② | 窗口首行注：`04837 8827` 首行落在指令中间（`.byte $1F` 垃圾 1 行 = `$8826 BCC $8847` 操作数字节）；注册段 `$884D-$8886` 自第二行起完整可读且逐指令吻合（本轮复跑核：`04837 8827 60` 尾段 `$8875 LDA $055E,X / ADC #$05 / … / $8881 TXA / STA $07D4,Y / $8885 LDA #$30` 吻合）；**对齐读法实测**：`04836 8826` 首行干净（`$8826 BCC $8847`） |

## 8. 复现命令 + 纪律自查

```bash
# ① bot run（当前工具态，确定性：连跑两遍 inputs/trace 均逐帧全等）
python3 tools/scenario_run.py run tas-stg01-bot1
# ② 闭环（录制→转写→回放→对拍）
python3 tools/tas_bot.py closure oracle/scenarios/tas-stg01-bot1.txt   # → closure.json verdict=pass, mismatch=0
# ③ 逐发归因 + 锚提取（负结果锚面全空）
python3 tools/tas_bot.py attrib tas-stg01-bot1
python3 tools/tas_bot.py anchors tas-stg01-bot1
# ④ 反作弊：写能力扫描 + fail-closed 解析 + PLAN 面（见 §5 表逐条）
grep -nE 'emu\.write|writeMemory|poke|memory\.write' tools/tas_bot.lua
# ⑤ 三件复跑（§6）
```

**纪律自查**：
- 零 ROM/台账/门禁/golden 改动（G-S5-6 injected 锚与 trigger 分级维持原样，§0③）；本文件为新增 findings。
- **负结果如实**：未通关、未及中 boss、关末门未自然触发——不宣称任何"接近通关"；旧工具态 run 达 arena 的事实
  **降级旁证**登记（§1 工具态史），不与当前工具态结论混用。
- **归因诚实**：unknown 784 显式保留（观察面缺口不发明分类）；boss_no_damage 49 逐条分解防误读（§3）；
  第四候选**不坐实不排除**（§4），坐实所需判别实验明确列出；死因只对 f8183 给接触候选（15px 几何实测），其余不宣称。
- **闭环真一致**：0 mismatch 为机器输出（closure.json/det.json），两遍独立 run 输入逐帧全等；
  生成物 = 纯按键场景（无条件指令/无 poke），可 `--skip-run` 单验转写。
- A/B/C 分级：run 数据（帧号/HP/坐标/输入）= A（trace/events 逐帧可重导）；工具态史推断（R-bot-4b 差异）= B；
  死因候选 = C（明示）；第四候选判别力评估 = B（数据 A 级、解释挂起）。
- 遗留缺口（不发明，沿用既有挂账）：W18 `$05DC` 语义写点探针、Wt-6 弹速接线、`$4C` 伤害模式旗、
  W15′ 玩家框注册；新增：bot 前段生存（tick 778<824 差额）与 arena 到达性属**策略实现面**（porter/实施侧），
  非 verifier 收果范围。
