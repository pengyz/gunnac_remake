# 敌坐标可见化：STG-01 最小实现规格（enemy-coords-impl-spec）

> 日期：2026-10-18｜角色：decomp-analyst（只读规格，不改代码）｜任务卡：task-71（前置 task-38 取证）
> 纪律：AGENTS.md §3（A/B/C 分级；未证不发明）；本卡**只产出规格**，不实现、不改 crates。
> 动态缺口：本机无 Mesen2 / `analysis/traces` 不入库 ⇒ 仅以**仓库内 tracked traces**（`oracle/traces/*.enemy.jsonl`，8 帧采样）为准；逐帧动态样本不足处显式标注。
> 复现：`python`（本机 `python3` 别名损坏）；命令见 §6（PowerShell 可执行形式）。

> **【2026-10-18 更正（task-75 复核 needs_revision → task-77 修订）】**
> F75-1：**"槽 8-27 实见 9 型"错误**——实测 **槽 8-27 只有 7 型**（`$12/$1A/$1C/$2A/$2B/$2F/$4E`）；`$05` 只在 slots 2-3、`$07` 在 slots 1-6（低槽，与系统型 `2` 同槽段共存）。全文改为"**全槽敌型 9 = 8-27 的 7 型 + 低槽 boss 候选 2 型**；系统对象按**型值** `1/2/$0F` 排除（不以槽段排除）"。
> F75-2：§4.2 类型集合断言改为"**按 v1 已接型（`V1_TYPES={$2F,$2B,$4E}`）过滤后相等**"；oracle 额外型（`$12/$1A/$1C/$2A`）入"未实现/fail-closed"清单，**不算 FAIL**（原"全集合相等 ⇒ C2 判据不可达"已修）。
> F75-3/4/5：补 `$A006/$A007=$A16F` 32B 类型表 hexdump（§6 D5）；"共同样本 ≥20"统一为"**共同 (帧,型) 采样点数**"；90% 达成率**先固定分母**（=共同采样点总数 N，排除低槽/短命/回绕后）。

---

## 0. 结论速览

| # | 项 | 结论 |
|---|---|---|
| 1 | **STG-01 实际敌型（开工范围）** | tracked trace 全槽敌型共 **9 个 = 槽 8-27 的 7 型**（`$12/$1A/$1C/$2A/$2B/$2F/$4E`）**+ 低槽 1-7 boss 候选 2 型**（`$05` slots2-3、`$07` slots1-6，与系统型 `2` 同槽段共存）。系统对象按**型值** `1/2/$0F` 排除（**不以槽段排除**；低槽可复用）。v1 只接其中 ctrl2 双证的 `$2F/$2B/$4E` 3 型（§1.3/§5.1）。 |
| 2 | **生成层可达性（决定 v1 能接哪些）** | sched 流 `$A26D` 43 记录（A 级解码，§1.2）：仅 **ctrl2 家族**可由 sched+trace 双证（type `$2F` leader ×16，成员 `$2B/$4E` 由 args[0] 指定且 trace 实见）；**ctrl1 家族（type `$2E` + 成员 `$40-$50`）trace 实见=0**（语义未证）；其余 `$12/$1A/$1C/$2A`（8-27）+ `$05/$07`（低槽 boss 候选）在 sched 中无对应 Spawn 项 ⇒ 来自 wave 流/EN 内容层（**未解码，W32**）⇒ **v1 开工只接 ctrl2 家族（`$2F`→`$2B/$4E`）+ 其余 fail-closed/未实现清单**。 |
| 3 | **x/y 初值来源** | **不在生成层**：`land_pair/land_leader` 仅写 type/参数（A 代码事实），九字段 `$865C` 只覆盖尾记录/back-half 路径（A 级字段映射见 §2）；普通型 x/y 初值由**各型 handler init 承担**（候选写点见 §3，B 级：handler 归属/窗口待 raw 审计）。 |
| 4 | **运动更新规则** | 引擎 `motion_update` = **A 级**（`bank14:$CCD3/$CD08`，099c510）：x：`$470+=vx_lo($550)`，`$4E0+=vx_hi($56C)+carry`；y：`$48C+=vy_lo($588)`，`$4FC+=vy_hi($5A4)+carry`；出屏（溢出/`y≥$F0`）⇒ Offscreen。**per-type 写者**（A 级候选 disasm，见 §3 表）。 |
| 5 | **trace 对拍口径** | 可对拍：**v1 已接型集合（过滤后）/ 每型首现顺序 / 8 帧采样帧上的 x/y**；不可对拍：精确 spawn 帧（8f 采样+1 帧滴灌偏差）、槽号身份（分配器差异）、型值 `1/2/$0F` 系统对象、短命型。验收：**≥20 个共同 (帧,型) 采样点（先冻结分母 N）、≤1px 达成率 ≥90%、`V1_TYPES` 过滤后集合相等**（脚本设计 §4；Rust 侧需先加 slot x/y 探查出口）。 |
| 6 | **动态缺口** | 仅 143 行 8f 采样 trace（f1264-2400）；无逐帧 init 证据；`analysis/traces` 需 Mesen2/Linux 重采（`scenario_run.py` POSIX-only）。 |

---

## 1. STG-01 开工范围（A 级：tracked trace + ROM 数据）

### 1.1 trace 实见敌型（全槽 12 型中敌型 **9**；`oracle/traces/gameplay-stage1.enemy.jsonl`，143 帧 / 8 帧采样 / 28 槽）

来源：`{"f","state","x","y"}`，`state/x/y` 各 28×1B hex（= `$0400/$04E0/$04FC + n`）。统计（全槽；系统型 `1/2/$0F` 按型值剔除）。**槽域实测（A）**：slots 8-27 型集合 = `{$12,$1A,$1C,$2A,$2B,$2F,$4E}`（**7 型**）；slots 0-7 型集合 = `{$01,$02,$05,$07,$0F}`（含 2 个低槽 boss 候选）。下表按型值列 9 个敌型：

| 型值 | 首现帧 | 首现槽/坐标（样例） | 8 帧位移签名（dx8 / dy8，字节差） | 备注 |
|---|---|---|---|---|
| `$12` | f1376 | slot8 (102,26) | **0 / 0**（11 槽全静态） | 静止型（炮台候选） |
| `$1A` | f1424（slot27 (90,25)）；f1632（slot26 (177,71)） | 5 槽 | dy8≈**+4..+5**；dx8 ≈ −13..+13 振荡 | 下漂+横摆 |
| `$1C` | f1792 | slot25 (76,151) | dy8≈**+4..+5**；dx8 振荡 | 下漂+横摆（同族） |
| `$2A` | f1792 | slot23 (22,15)；slot9 (34,80) | dy8 **−22..−49**（slot9/14）或 **+14..+16**（slot23/25）；dx8 振荡 | 上下两种运动（模 256 回绕需注意） |
| `$2B` | f1344 | slot8 (32,9) 等 9 槽 | dy8≈**+5**；dx8 振幅大（−30..+39） | 快速横摆+下漂 |
| `$2F` | f1344 | slot27 (128,8) 等 5 槽 | **0 / 0**（恒 (128,8)） | **leader/滴灌器**（成员型 `$2B/$4E`） |
| `$4E` | f1600 | slot10 (16,15) 等 9 槽 | dy8≈**+19**（≈2.4px/f）；dx8 振荡 | 下速+横摆 |
| `$05` | f2176 | slot2 (133,148)/slot3 (133,132) | dx8≈**+22..+23**；dy8=0 | **低槽 slots2-3**（boss 部件候选；**不在 8-27 集合**） |
| `$07` | f1264 slot1 (128,106)；f2208 slots2-6 (88..172,56) | 6 槽 | 含字节回绕（dy8 原始值 98 等）⇒ **逐帧运动待证** | **低槽 slots1-6**（与系统型 `2` 同槽段共存；**不在 8-27 集合**） |

> **排除与槽域（A 级，实测）**：按**型值**排除 slot0 型 `1`（玩家）、型 `$0F`（玩家死亡态）、型 `2`（系统/滚动对象）——**不以槽段排除**（低槽 1-7 可复用：type `2` f1264-2168 与 boss 候选 `$05/$07` f2176+ 同槽段共存；且 `$2F` leader 也常驻低槽 23-27 之外的高槽）。依据：引擎 `ObjectGrid` 注释"槽 0..7 与 8..27 分区——分配域 8..=27"（`engine.rs` L429，A 代码）+ trace 实见分布（本表槽域列，A）。

### 1.2 sched 流解码（A 级：`bank8:$A26D`，43 记录，`wave.rs::decode_sched` 同口径）

复现 §6 D1。结果摘要：

| 项 | 值 |
|---|---|
| 记录数 | 43（tick 1..1237；末记录 Jump `$A397`→`$A38F` 自环） |
| Spawn ctrl2 | type **`$2F`** ×16，args=`[成员型, ?, ?, y参数]`，成员型 ∈ `{$2B(43), $4E(78)}` |
| Spawn ctrl1 | type **`$2E`** ×21，args[0] ∈ `{$40..$50}`（64/65/68/77/79/80…）——**trace 实见 0 次**（生成语义未证） |
| Pair | `type $44, param 0` ×1（trace 未见） |
| Reg/SetType/Jump | 其余记录（音画参数/定型/自环） |

⇒ **双证交集**：`$2F`（leader）+ `$2B/$4E`（成员）由 sched ctrl2 args[0] 与 trace 实见共同支持（A 级生成链）。ctrl1 家族与 `$44` 仅静态解码、无 trace 实例 ⇒ **fail-closed**。

### 1.3 生成层归属判定（每型）

| 型值 | 生成层来源（级） | 依据 |
|---|---|---|
| `$2F` | **sched ctrl2 Spawn**（A） | §1.2；Rust `land_leader`（`engine.rs:475-511`，A 代码） |
| `$2B/$4E` | **leader 滴灌成员**（A） | leader type `$2F` 的 `$518`=成员型（args[0]）；`drip_step` 按 `$534` 帧间隔 `land_pair`（`engine.rs:173-205`，A）；trace 成员型实见 ✓ |
| `$12/$1A/$1C/$2A`（槽 8-27）+ `$05/$07`（低槽 boss 候选） | **未解码：wave 流（op5 `$A3CD/$A3E4`）/EN 内容层**（未证 W32） | sched 43 记录中无对应 Spawn/Pair 项；EN `$ADD0-$ADF7` 8×5B 已读（A 字节）但 type↔记录映射未证（task-61 §1.5 / w28-boss §2.2） |
| `$2E` + `$40-$50` | sched ctrl1（静态解码 A；**trace 零实例**） | 语义未证 ⇒ v1 不接 |

---

## 2. 九字段映射（`bank00:$865C-$869E`，A 级逐指令；适用尾记录/back-half 生成路径）

| # | 字段 | 来源/公式（A 指令） | 写入 |
|---|---|---|---|
| 1 | `$48C,Y` | 入口 A（`STA $48C,Y`） | 码流附加参 |
| 2 | `$470,X` | `$10` | x 亚像素初值 |
| 3 | **type `$400,X`** | `($9F)[$10 & $1F]`（`$9F/$A0`=dir 尾字段 $A006/$A007 = STG-1 `$A16F` ⇒ 32B 类型表） | 类型 |
| 4 | **X `$4E0,X`** | `$11 << 4` | x 坐标初值 |
| 5 | **Y `$4FC,X`** | `$92 + $E0 − $91`（含借位语义） | y 坐标初值 |
| 6 | `$614,X` | `$8E + 2*$11` | 参数 |
| 7 | `$72C,X` | `((（$8E+2*$11) & $1C) >> 2) + $90` | 参数 |
| 8 | `$5F8,X` | `$8F` | 参数 |
| 9 | `$710,X` | `$8D` | 参数 |

> 边界（A 代码事实）：`land_pair/land_leader` 落地**不写 x/y**（仅 type/`$470/$518/$630/$534/$48C` 与行为字段）⇒ **普通 sched 型的 x/y 初值不在生成层**，必须由该型 handler init 写（§3）。`$865C` 九字段用于经 `($9F)` 尾记录触发的生成路径（boss/back-half），**STG-01 普通敌型是否走此路径未证**（W32）⇒ v1 按 fail-closed：普通型坐标来自 handler，不套用九字段。
>
> **`$A16F` 32B 类型表独立复现（F75-3）**：dir STG-1 行 `bank8:$A000` = `00 A0 FA BC 09 09 6F A1` ⇒ `$A006/$A007` = **`6F A1` = `$A16F`**；`$A16F` 32B = `CF 1B 10 1F A4 CF 14 10 CA A6 8F 1D 11 86 A4 12 60 05 A4 81 1D 21 35 A4 9D 1A 22 29 A4 82 14 13`（hexdump 命令 §6 D5，A 字节）。

---

## 3. 各型 handler 候选与运动写点（A 级 disasm，B 级归属）

handler 目标来自 **ROM dispatch 表**（`enemy.rs` `TYPE_DISPATCH` = A 级 ROM 数据，256 项）；type<0x30 目标在 `$8000` 窗（bank2），type≥0x30 目标在 `$A000` 窗 + `bank_switch`。**窗口/bank 归属细节为 B 级**（待 raw 区逐段审计；type `$05/$07` 目标 `$AE05/$A601` 在 `$A000` 窗，bank 未定，候选列出）。表中 `$12/$1A/$1C/$2A/$2B/$2F/$4E` = **槽 8-27 的 7 型**（v1 候选）；`$05/$07` = **低槽 boss 候选**（不在 8-27 集合，单列）。

| 型 | dispatch 目标 | 候选 PRG/文件偏移 | 候选 handler 的坐标/速度写点（da6502 实读） | 级 |
|---|---|---|---|---|
| `$2F`（leader） | `$899B`（bank2 窗） | file `0x49AB` | `STA $04E0,X`/`STA $04FC,X`（由 `$0630` 驱动）；`STA $0668/$04C4/$06BC/$06A0`；滴灌子体写 `$0400,Y/$0470,Y` | A（指令）/B（归属） |
| `$2B`（成员） | `$90AB`（bank2 窗） | file `0x50BB` | **`STA $05A4,X`/`$0550,X`/`$056C,X`/`$0588,X`（速度对）**；`STA $04FC,X`；`STA $04E0,X`；`$05F8/$0518/$0684/$0534` | A/B |
| `$4E`（成员） | `$AE44`（bank4, sw=04） | file `0x8E54` | **`ADC $04E0`→`STA $04E0,X`**；`STA $04FC,X`；**`$05A4/$0588`（y 速度对）**；`$0614/$04C4/$0518/$0684/$0534` | A/B |
| `$12` | `$83BA`（bank2 窗） | file `0x43CA` | `ADC $072C,X`→`STA $04FC/$04E0`（运动）；`$0518/$0534/$0684/$072C/$0710` | A/B |
| `$1A` | `$8DB8`（bank2 窗） | file `0x4DC8` | `LDA $048C,X`→`$06A0`；`$0588/$05A4/$04FC`（y 速度对+坐标）；`$0518/$0534/$0684` | A/B |
| `$1C` | `$8E53`（bank2 窗） | file `0x4E63` | `LDA $048C,X`→`$06A0`；`$0588/$05A4/$04FC`；`$0518/$0534/$0684` | A/B |
| `$2A` | `$8FCE`（bank2 窗） | file `0x4FDE` | `LDA $04E0`→`$0710`；`$06A0→$048C`；子体生成写 `$0518/$0534/$04C4/$0588/$0614/$05A4,Y/$0684,Y` | A/B |
| `$05`（**低槽 boss 候选**） | `$AE05`（`$A000` 窗，bank 未定） | 候选 bank1-5 file `0x2E15/0x4E15/0x6E15/0x8E15/0xAE15` | 待定 bank 后审计（候选 bank3 起始 `LDA #$05; STA $0401,X`；bank5 起始 `BPL/JSR $C366`） | B（候选） |
| `$07`（**低槽 boss 候选**） | `$A601`（`$A000` 窗，bank 未定） | 候选 bank1-5 file `0x2611/0x4611/0x6611/0x8611/0xA611` | 候选 bank3 起始 `LDA #$4F; STA $0684,X; … LDA $056C,X`（含速度字段读） | B（候选） |
| `$2E`/ctrl1 成员 `$40-$50` | `$898A` / bank4 `$A611..$AE83` | — | 静止解码可做，但 trace 零实例 ⇒ **v1 不接** | C（未证） |

> 运动规则（A，`motion_update`，`engine.rs:967-1002`）：x：`($470,X += $550,X)` 进位；`$04E0,X += $56C,X + carry`；y 同构用 `$588/$5A4`；出屏判定 `y>=0xF0` 或任一溢出。**per-type 差异 = 哪些型在 init/tick 写 `$550/$56C/$588/$5A4`**（上表 A 级 disasm 已列候选写点；字段语义已证，**逐型写值/节奏待 raw 区审计**=B）。
> trace 旁证（A 观测）：`$2B` dy8≈+5 ⇒ 速度对 y 分量 ≈0.6px/f；`$4E` dy8≈+19 ⇒ ≈2.4px/f；`$12` 静止 ⇒ 速度对=0（或未写）；与上表写点族相容（B）。

---

## 4. trace 对拍口径与验收脚本设计

### 4.1 可比/不可比列

| 列 | 可比性 | 口径 |
|---|---|---|
| `state`（28 槽型值） | **可比（过滤集合级）** | 每 8 帧采样帧上，**按 `V1_TYPES` 过滤后**的型值集合相等；oracle 额外型入"未实现/fail-closed"清单（不算 FAIL）；允许槽号不同（分配器差异） |
| 每型首现顺序 | **可比（序级）** | 按 `(型, 首现帧)` 排序比较；允许 ±8 帧（采样粒度） |
| `x/y`（28 槽） | **条件可比** | 先按 `(型, 首现序)` 建立槽映射，再在**共同采样帧**比较；仅比较两实现的同型同序槽；**系统型 `1/2/$0F`、短命型（<8f）、回绕帧排除**（低槽 boss 候选 `$05/$07` 不在 v1 比较集） |
| 精确 spawn 帧 | **不可比** | 8 帧采样 + 1 帧滴灌延迟已知偏差（`host.rs` 注释"落地发生在 stage_tick 返回后同逻辑帧；1 帧滴灌延迟已知偏差"） |
| 帧内/逐帧运动 | **不可比（当前）** | 只有 8 帧采样；逐帧需 Mesen2 重采 |

### 4.2 验收脚本设计（≥20 共同 **(帧,型)** 采样点；≤1px ≥90%；**分母先冻结**）

```text
输入：oracle = oracle/traces/gameplay-stage1.enemy.jsonl（8f 采样）
      rust   = 新导出（待实现）：同 schema 的 28 槽 state/x/y 逐帧或 8f 采样
      V1_TYPES = {$2F, $2B, $4E}   # v1/C2 已接型（F75-2）
步骤：
1) 解析两流 → frames[type] -> [(f, slot_order, x, y)]（槽序 = 该型首次出现顺序）
2) 类型集合断言（F75-2）：types(oracle) 与 types(rust) 各自与 V1_TYPES 取交集后必须相等；
   rust 不得出现 V1_TYPES 以外的已实现型；
   oracle 额外型（$12/$1A/$1C/$2A 及低槽 $05/$07）列入“未实现/fail-closed”清单，不参与 PASS/FAIL
3) 共同采样点（F75-4 口径统一）：对每个 (type∈V1_TYPES, order) 取两流均有值的采样帧；
   每个 (帧,型) 计 1 点（同帧同型多槽只取首序槽，避免重复计数）；要求共同采样点 ≥20
4) 固定分母（F75-5）：先冻结 N = 步骤 3 得到的共同采样点总数（已排除低槽/短命型/回绕帧）；
   N < 20 ⇒ 判 INCONCLUSIVE（不判 PASS/FAIL）；达成率 = 通过点数 / N
5) 像素差断言：每个采样点 |dx|<=1 且 |dy|<=1（字节模 256 差）；达成率 >=90% ⇒ PASS
6) 反例保护：rust 出现 oracle 不存在型，或实现 V1_TYPES 外型 ⇒ FAIL；
   oracle 有而 rust 无的型 = 未实现清单（不算 FAIL，但必须打印）
7) 输出：per-type 表（样本数/通过数/达成率/最大偏差帧）+ 分母 N + 总 PASS/FAIL/INCONCLUSIVE
复现基准（当前 oracle 自身）：同一脚本对 oracle 重采样应得 100%（自洽校验）
```

> Rust 侧前置（本卡范围外）：`StageProbe` 增 `slot_x[28]/slot_y[28]` 或 FrameView `position=Some((x,y))` + 导出 JSONL（task-38 §3.5 第 3 步）。**未完成前本验收不可执行**（如实）。

---

## 5. fail-closed 清单 + 后续实现卡拆分

### 5.1 fail-closed（v1 不得实现/不得声称）

1. **未实现生成源（未解码）**：`$12/$1A/$1C/$2A`（槽 8-27）+ `$05/$07`（**低槽 boss 候选**），wave 流/EN 内容层 W32 未证 → 不 spawn、不画、不声称坐标；两流对拍时=oracle 额外型，入未实现清单（§4.2 步骤 2/6）。
2. **ctrl1 家族**（`$2E` + 成员 `$40-$50`）：trace 零实例 ⇒ 保持 `land_leader` 现状（可落地但坐标/行为 fail-closed 空）。
3. **type `$05/$07` bank 归属未定**（`$A000` 窗候选 bank1-5）→ 不接。
4. **per-type 速度对写值/节奏**（`$550/$56C/$588/$5A4`）未逐值审计 ⇒ 先只接**已 A 级写点确认**的型（`$2B/$4E`），其余运动由 trace 签名驱动前须复核。
5. **玩家/敌弹/H-16 命中链**不在本卡（另一域），坐标可见化不得顺带接线。
6. **x/y 初值**：未拿到 handler init 逐字段（B）前，v1 允许用 trace 首现坐标作**临时期望值**，但必须在代码注释与 FrameView 中标注"B 级临时，非 ROM 级已证"。

### 5.2 后续实现卡拆分建议

| 卡 | 角色 | 范围 | 完成判据 |
|---|---|---|---|
| **C1（前置）trace 导出/对拍工具** | porter | `StageProbe` 暴露 slot x/y（只读镜像，不改语义）+ `tools/enemy_coords_compare.py`（§4.2 脚本） | 现有 oracle 自校验 100%；Rust 导出 schema 与 oracle 一致；工具只读 |
| **C2 ctrl2 家族坐标落地** | porter | `$2F` leader + `$2B/$4E` 成员：sched ctrl2 spawn（已 A）+ `land_leader` 成员参数 + handler init 坐标/速度（`bank02:$899B/$90AB`、`bank04:$AE44` 的 A 级写点）+ `motion_update` 接线 | C1 脚本：这 3 型共同 **(帧,型) 采样点 ≥20**、**分母先冻结**、≤1px ≥90%；**V1_TYPES 过滤后集合相等**（oracle 额外型入未实现清单，不判 FAIL）；selftest ×2 签名一致 |
| **C3 bank2 型 handler 批** | decomp-analyst + porter | `$12/$1A/$1C/$2A` handler init/运动（`bank02:$83BA/$8DB8/$8E53/$8FCE`）+ 生成源仍 fail-closed；先补 handler 归属 A 级（raw 区重解码） | 每型 trace 签名对齐（≤1px ≥90%）；生成源未证期间用注入槽测试 |
| **C4 wave 流/EN 内容层解码** | decomp-analyst | op5 流 `$A3CD/$A3E4` 内容解码 + EN 记录 type↔记录映射（W32） | 生成源 A/B 定谳 + 解出 `$12/$1A/$1C/$2A/$05/$07` 的 spawn 记录；不接行为 |
| **C5 boss 坐标/实体** | decomp-analyst | `$05/$07`（末期）+ `$4A/$5F`（task-61 已定 HP 表）bank 归属与 init | bank 归属 A 级 + init x/y 写点 A 级；boss 行为仍 fail-closed |

---

## 6. 复现命令（PowerShell 可执行；`@'…'@ | python -`）

**D1 sched 流解码（§1.2；期望 43 记录 / ctrl2×16 / ctrl1×21 / Pair×1）**

```powershell
@'
rom=open('rom/gunnac_cn.nes','rb').read()
window=rom[0x10+8*0x2000:0x10+8*0x2000+0x2000]
BASE=0xA000
def ba(a): return window[(a-BASE)&0x1FFF]
out=[]; cur=0xA26D; tick=0; seen=[]; first=True
while True:
    delay=ba(cur); tick=delay+1 if first else tick+delay; first=False
    if cur in seen or len(out)>=64: break
    seen.append(cur); items=[]; y=cur+1; nxt=None; jmp=None
    while True:
        v=ba(y)
        if v>=0x10: items.append(('Pair','$%02X'%v,ba(y+1))); y+=2; continue
        y+=1
        if v==0: nxt=y; break
        if v in (1,2,11,12):
            args=[ba(y+i) for i in range(4)]; t={1:0x2E,11:0x2C,2:0x2F}.get(v,0x2D)
            items.append(('Spawn',v,'$%02X'%t,args)); y+=4; continue
        if 3<=v<=9: jmp=ba(y)|(ba(y+1)<<8); items.append(('Jump',v,'$%04X'%jmp)); break
        if v==10: items.append(('SetType',ba(y),ba(y+1))); y+=2; continue
        if v in (13,14): items.append(('Reg',v,ba(y),ba(y+1))); y+=2; continue
        items.append(('Unknown',v)); break
    out.append((cur,delay,tick,items))
    if jmp is not None: cur=jmp; continue
    if nxt is not None: cur=nxt; continue
    break
print('records',len(out))
for a,d,t,it in out: print('tick=%-4d $%04X d=%-3d %s'%(t,a,d,it))
'@ | python -
```

**D2 trace 类型/运动签名（§1.1，期望全槽 9 型：槽 8-27=7 + 低槽 2；系统型 1/2/$0F 排除）**

```powershell
@'
import json, collections
rows=[json.loads(l) for l in open('oracle/traces/gameplay-stage1.enemy.jsonl',encoding='utf-8') if l.strip()]
traj=collections.defaultdict(list)
for r in rows:
    st=bytes.fromhex(r['state']); x=bytes.fromhex(r['x']); y=bytes.fromhex(r['y'])
    for slot in range(28):
        if st[slot]: traj[(st[slot],slot)].append((r['f'],x[slot],y[slot]))
by=collections.defaultdict(list)
for (t,s),v in traj.items(): by[t].append((s,v))
def med(L): return sorted(L)[len(L)//2] if L else None
for t in sorted(by):
    if t in (1,2,15): continue
    print('type $%02X slots=%d'%(t,len(by[t])))
    for s,v in sorted(by[t])[:5]:
        dx=[b[1]-a[1] for a,b in zip(v,v[1:]) if b[0]-a[0]==8]
        dy=[b[2]-a[2] for a,b in zip(v,v[1:]) if b[0]-a[0]==8]
        print('  slot%2d first=%s dx8=%s dy8=%s'%(s,v[0],(med(dx)),(med(dy))))
'@ | python -
```

**D3 候选 handler 逐指令（§3；da6502 = 纯反汇编只读工具）**

```powershell
python tools/da6502.py rom/gunnac_cn.nes 49AB 899B 180    # type $2F leader (bank2 $899B)
python tools/da6502.py rom/gunnac_cn.nes 50BB 90AB 180    # type $2B member
python tools/da6502.py rom/gunnac_cn.nes 8E54 AE44 180    # type $4E member (bank4 $AE44)
python tools/da6502.py rom/gunnac_cn.nes 43CA 83BA 180    # type $12
python tools/da6502.py rom/gunnac_cn.nes 4DC8 8DB8 180    # type $1A
python tools/da6502.py rom/gunnac_cn.nes 4E63 8E53 180    # type $1C
python tools/da6502.py rom/gunnac_cn.nes 4FDE 8FCE 180    # type $2A
# 歧义型（$A000 窗 bank 未定）候选：
python tools/da6502.py rom/gunnac_cn.nes 2E15 AE05 80     # type $05 bank1 候选
python tools/da6502.py rom/gunnac_cn.nes 6E15 AE05 80     # type $05 bank3 候选
python tools/da6502.py rom/gunnac_cn.nes 6611 A601 80     # type $07 bank3 候选
```

**D4 dispatch 目标表（§1.3/§3；期望 256 项）**

```powershell
Select-String -Path crates/game-core/src/enemy.rs -Pattern 'TypeDispatch \{ target'
# 或 python 解析同文件 TYPE_DISPATCH 数组（索引即型值）
```

**D5 九字段生成链逐指令 + `$A16F` 32B 类型表（§2；F75-3）**

```powershell
Select-String -Path reference/seed/gunnac.asm -Pattern '^bank00:(865C|866[0-9A-F]|867[0-9A-F]|868[0-9A-F]|869[0-9A-F])'
@'
rom=open('rom/gunnac_cn.nes','rb').read()
b=0x10+8*0x2000
print('dir STG-1 row:', rom[b:b+8].hex(' '))
print('$A006/$A007 bytes:', rom[b+6:b+8].hex(' '), '(= $A16F)')
print('$A16F 32B type table:', rom[b+0x16F:b+0x16F+32].hex(' '))
'@ | python -
# 期望：dir=00a0fabc09096fa1；$A006/$A007=6fa1；$A16F 32B=cf1b101fa4cf1410caa68f1d1186a4126005a4811d2135a49d1a2229a4821413
```

---

## 7. 不确定项 / 未做（显式）

1. **动态样本不足**：仅仓库内 8 帧采样 trace（143 行）；无逐帧坐标/init 证据；`analysis/traces` 与 Mesen2 需 Linux 宿主（`scenario_run.py` POSIX-only）⇒ §4 验收脚本**已设计未执行**（Rust 探查出口也未实现）。
2. **handler 归属 B 级**：dispatch 目标=A 级 ROM 数据；但 `$A000` 窗 bank 动态归属（**低槽 boss 候选** `$05/$07`）与部分 raw 区 handler 边界需重解码确认；本文候选 disasm 仅取 PRG 偏移视图。
3. **生成源未证**：`$12/$1A/$1C/$2A`（槽 8-27）+ `$05/$07`（低槽 boss 候选）的 spawn 记录在 sched 中不存在，wave 流/EN 内容层（W32）未解码 ⇒ 不可实现（=oracle 额外型，入未实现清单）。
4. **ctrl1 家族零实例**：静态解码存在但 trace 未观测，语义未证（不发明）。
5. **逐型速度对写值/节奏**：字段语义 A、写点候选 A，但具体每帧写值需逐帧审计（当前仅 trace 8 帧签名旁证）。
6. **低槽 boss 候选 `$05/$07` 字节回绕**：dy8 原始差值含 256 回绕，运动方向/幅度待逐帧确认。
7. 只读卡：未改 crates/data/gates/rom/reference；未 git add/commit；仅本文件。

---

## 8. 自查

- 只写本文件（`docs/findings/enemy-coords-impl-spec.md`）。
- 级：§1.1 trace 统计/§1.2 sched 解码/§2 九字段指令/§3 dispatch 目标与 da6502 字节/§4 oracle schema 为 A；§1.3 归属、§3 handler 归属与速度写值、§4 验收阈值为 B（设计口径）；未证项均显式 fail-closed。
- 数字可复算：43 记录 / **全槽 9 型（槽 8-27=7 + 低槽 boss 候选 2）** / 8f 采样 143 帧 / §4 阈值（分母冻结）均附 D1-D5 命令。
- 对抗复核建议优先证伪：(a) §1.1 型集合与系统槽排除；(b) §1.2 ctrl1/ctrl2 计数与 trace 交集；(c) §2 "生成层不写 x/y"的代码事实；(d) §3 候选 handler 偏移与写点（da6502 逐条）；(e) §4 可对拍列/阈值是否过宽或过严。

【2026-10-18 初版】decomp-analyst，task-71。
