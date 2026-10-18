# L5 剩余面（G-L5-1/2/3/4）新 GUI trace 可行性盘点 + 后续采集建议（预研）

> 日期：2026-10-18｜执行：l4-analyst｜输入：**GUI 重采 trace**（task-133 v4 clean 批次；input-effective）+ P0-0-2 GUI 重渲/重裁（task-140/143）；**不跑 Mesen**（只读 trace），不改工具/expected/gates。
> 目的：Lead 要求"等 task-142/144 复核期间预研 L5 剩余面（G-L5-2/3/4）与 L5 后续采集建议，并给基线冻结材料"。
> 关联：`docs/spec/l5-all-clear.md` §2.5、`docs/findings/l5-gap-inventory.md` §2、`docs/findings/ending-sequence.md`、`l5-b1-anchor-rederive-run2.md`。

## 0. 结论摘要

| 缺口 | 新 GUI trace 能直接给什么 | 还缺什么 | 档位 |
|---|---|---|---|
| **G-L5-4 stg08+ending** | **A 级 trace 观测链已出现（`$95F9` 入口归属 B 级）**：C1/C3 `stage_no 1→8→9→1`；`$180==9` @f27731；`$26:=0x80`（`$95F9` 入口）@f27734；`$27 bit7`/`$26 bit4` @f27036；`$0300=0x10` @f27701 | stg08-start/combat/clear/all-clear-95f9 的**精确帧 capture**（现有 shots 仅 1500f 间隔，帧号不重合）；assisted 来源登记 | **A（trace 链）+ B（capture 锚）** |
| **G-L5-2 boss 击杀/HP/相位** | C1/C3 `boss_hp` **28 段**（连续 hp>0 区间；段长 **2–6387f**，17/28 在 40–700f；kill-edge（段末）间隔 **58–7076f**）；C2 **26 段**（段长 9–12990f，20/26 在 40–700） | **stage 归属不可判**（C1/C3 `$0180` 被 poke=08 锁定，全 run stage_no=8）；stg01 boss 无 watch；stg02-06 无 boss trace | **B 需采集**（自然/正确 stage 信号 + boss watch） |
| **G-L5-3 流末/自然关末** | gs5 链注入/自然链语义已验证（`$CA=0` 全程；注入 `$26 bit4` 链）；C1/C3 `trig_ca=5` 自 f22405 | **自然 `$CA bit0` 流末未观测**（gs5 `$CA` 全 0）；缺 `$CC/$C8` 游标 watch | **B 需采集** |
| **G-L5-1 能打到底的输入轨** | **C1/C3/C2 GUI bot 跑输入生效**（held 2.2–3.0 万帧、120000f 推进、到 ending）；gs5 全系 input-effective | 自然（无 poke/assist）八关 track 仍无；C1/C3 是 assisted/injected | **A（assisted 候选）+ B（natural）** |
| B1 锚 | **8/8 ALL MATCH + F1 4/4**（run2） | F2 跨脚本相位差（task-145） | 主判据达成 |

## 1. trace 覆盖清单（GUI 版；`analysis/traces/*.frames.jsonl`）

| trace | frames | 输入生效（held 非零帧） | stage_no 覆盖 | 语义 watch | boss_hp |
|---|---:|---:|---|---|---|
| title-menu | 1080 | 0（无输入场景） | — | zp/0320 | — |
| gameplay-stage1 | 2400 | **1049** | —（无 stage watch） | player_x/mode/bomb/pressed | — |
| gs5-stg01-clear-anchor | 5000 | **1094** | 1（3095）/2（1905） | flags26/27/countdown/gate300/trig_ca | — |
| gs5-stg01-full | 15000 | **12601** | 1（15000） | flags26/countdown/gate300 | — |
| gs5-stg01-clear-c1 | 6000 | **1158** | 1（3095）/2（2905） | 同上 | — |
| gs5-stg01-clear-c2 | 5000 | **1093** | 1（3095）/2（1905） | 同上 | — |
| stg03-timeline | 24000 | —（无 watch） | —（无 stage watch；只有内存 blob） | 无 | — |
| stg06-timeline | 30000 | — | — | 无 | — |
| stg08-timeline | 34000 | — | — | 无 | — |
| **tas-stg08b2-direct（C1）** | 40000 | **22511** | **1/8/9** | flags26/27/countdown/gate300/trig_ca/pstate/boss_hp/death/weapon/bomb/SCH4 | **28 段** |
| **tas-stg07-relay（C2）** | 120000 | **30374** | 1/7/8 | 同上（+f4c_traj） | **25 段** |
| **tas-stg08-direct（C3）** | 120000 | **22512** | **1/8/9** | 同上 | **28 段** |

> stg03/06/08-timeline 仍只有 zp/iram/oam blob，无 stage/flags 语义 watch —— 其"关内内容"只能靠 render 侧或新增 watch 重建；本盘点不臆断。

## 2. G-L5-4（stg08 关内 + ending）：**A 级 trace 观测链已出现**（`$95F9` 入口归属 B 级；最高价值）

C1（`tas-stg08b2-direct`）与 C3（`tas-stg08-direct`）GUI trace 的 `w` 字段（=RAM watch）显示同一链（C1 数字；C3 同）：

| 帧 | `stage_no`=$180 | `flags26`=$26 | `flags27`=$27 | `gate300`=$0300 | `trig_ca`=$CA | 语义（对照 l5-all-clear §1.2/§2） |
|---:|---|---|---|---|---|---|
| f913–1192 | 1 | **0x10（bit4）** | 0 | 0 | 0 | 菜单→开 stage 过渡窗（F1-a 同） |
| f1265–27730（C1；C3=1255–27730） | **8** | 0/0x10 段 | 0（至 f27035） | 0 | **5（自 f22405 起）** | stage8 段（poke `0180=08` 直入；assisted；C1=40000f） |
| **f27036** | 8 | **0x10（bit4）** | **0x80（bit7）** | 0 | **5** | **关末清屏入口**（`$27 bit7` 一次性入场 + `$26 bit4` 门） |
| f27036–27700 | 8 | 0x10 | 0x80 | 0 | **5** | 关末链（对应 G-L5-3 的 50→0/`$0300` 闸前段） |
| **f27701** | 8 | **0（bit4 清）** | 0x80 | **0x10** | **5** | `$0300≠0` 闸开 / `$26 bit4` 清（E36E） |
| **f27731** | **9** | 0 | 0x80 | 0x10 | 5 | **`$180==9` = 通关判定**（E392 `CMP #9` 后） |
| **f27734** | 9 | **0x80（bit7）** | 0x80 | 0x10 | 5 | **`$26:=$80`（A 级观测）**；`$95F9` 入口归属 B 级（E3A1 反汇编映射） |
| f27734–32034 | 9 | 0x80 | 0x80 | 0x10 | 5 | **ending 段**（staff roll/结局卡运行；`stage_no` 保持 9） |
| f32035+ | **1** | 0x80 | 0x80 | 0x10 | 5 | ending 结束回卷（stage_no 回 1；`$26 bit7` 仍 0x80） |

- **C3（`tas-stg08-direct`，120000f）同链**：stage8 f1255 起、**stage9 f27731–32031**、f32032+ 回 1；flags26/27/0300/trig_ca 转折帧与 C1 完全一致（f27036/f27701/f27731/f27734）。
- **证据等级**：`$180==9`（f27731）与 `$26:=$80`（f27734）是 **A 级 trace 观测**；`$95F9` **入口归属为反汇编映射（B 级）**（bank00:95F9 `LDA #$80/STA $26`，以既有 findings 为据；trace 只观测 `$26` 写，不直接观测 PC）；`$27 bit7`/`$26 bit4`/`$0300` 闸序列与 `ending-sequence.md`/`l5-all-clear` §1/§2 的已证序列一致。
- **5 占位（stg08-start/combat/clear/all-clear-95f9）帧候选**（均来自 C1/C3，assisted 来源）：
  - `stg08-start`：f1265（poke `0180=08` 后；assisted，**非自然关始**，需登记）；
  - `stg08-combat`：f1265–27035 内任选（建议取 stage8 内敌活动帧，如 f20000 附近；需以 shots/敌槽判定）；
  - `stg08-clear`：**f27036**（`$27 bit7`+`$26 bit4` 关末入口）或 f27700（闸前）；
  - `all-clear-95f9`：**f27734**（`$26:=$80` 入口）或 f27731（`$180==9`）。
- **还缺**：上述候选帧的**精确 capture**（现有 C1/C3 shots 为 `capture_every 1500`，帧号 f1265/27036/27731/27734 均不重合）⇒ **B 级采集**：新建 `tas-stg08-direct-anchors`（或复用 C3 输入 + 只加 capture 列表）抓 f1265/f20000/f27036/f27700/f27731/f27734/f28000 等 7 帧（GUI ~6min，trigger=assisted）。
- **注意**：C1/C3 的 stage8 全程 `$180=8` 是 poke 直入所致（**不是自然八关**）；`ending-4/5` 旧 expected 的 `6ff345dc` 在 C1/C3 出现在 **f33000+（stage_no 已回 1 的 ending 后段）**，与 f27734–32034 的 ending 段属不同子状态——mapping 时须按帧号/子状态登记，不能笼统称"ending 屏"。

## 3. G-L5-2（boss 击杀/HP/相位）：部分 A，stage 归属是阻塞点

- **C1/C3**：`boss_hp` 有效 **28 段**（定义=连续 `boss_hp>0` 帧区间；段长 **2–6387f**，其中 **17/28** 在 40–700f；
  kill-edge（相邻段末帧间隔）**58–7076f**）；**C2 26 段**（段长 9–12990f，20/26 在 40–700f；段末间隔 58–13286f；stage7/8）。
- **阻塞**：C1/C3 的 `$0180` 被 `poke 0180=08` 锁定 ⇒ **stage8 段（C1 f1265–27730 / C3 f1255–27730）`stage_no=8`**，
  **28 段 boss 无法归属到 stage1..8**（bot 八关顺序可见但无 stage 边界信号；C1/C3 实有 stage_no 1/8/9/1）。
- **缺口**：stg01 boss 无 `boss_hp` watch（gameplay-stage1/gs5 无该 watch）；stg02–stg06 无动态 trace。
- **可行建议**：新增/改 overlay：①在 C2（stage7/8 已自然 `$180` 推进）基础上加 `$0630/$064B/$048C` watch 以覆盖 stage7/8 boss；②为自然八关 bot（pending）加同 watch；③或对 `gs5-stg01-full` 加 boss watch overlay 补 stg01。均 **B 需 Mesen**。

## 4. G-L5-3（流末记录/自然关末）：注入链 A，自然流末仍缺

- gs5 链（anchor/c1/c2）`trig_ca`（$CA）**全程 0**（F1-c 复核一致）⇒ 当前 gs5 是 **`$26 bit4` 门位注入**（poke f2400），不是自然 `$CA bit0` 流末消费；自然关末链路未观测。
- C1/C3 的 `trig_ca` 从 **f22405 起 =5**（`$CA` bit0+bit2）——bot 后段出现了流末标志候选，但同样受 stage 归属限制，且缺 `$CC/$C8`（流记录游标）watch。
- **建议**：新采集/overlay 增加 `watch $CA/$CC/$C8`，在自然/assisted 全关跑中定位 `$CA bit0→$26 bit4→$27 bit7→$2F 50→0→$0300≠0→INC $180` 的自然观测（B 需 Mesen）。

## 5. G-L5-1（能打到底的输入轨）

- **C1/C2/C3 GUI bot 跑**输入生效（held 2.2–3.0 万帧、**C1 40000f；C2/C3 120000f** 推进、C1/C3 到 `$180==9`）⇒ 可作 **assisted/injected 证据轨**（不能冒充 natural；C1/C3 含 `0180` 选关 + 三址 assist）。
- 自然（无 poke/assist）八关 track 仍未达成；gs5 仅到 stage2、stg03/06/08-timeline 无输入语义。→ **B 需采集/裁定**。

## 6. 后续采集建议清单（按优先级；均"不跑 Mesen 先设计，批准后采集"）

| # | 场景/动作 | 目的 | 改动量 | 预估 | 判据 |
|---|---|---|---|---|---|
| S1 | `tas-stg08-direct-anchors`（复用 C3 输入 + capture f1265/f20000/f27036/f27700/f27731/f27734/f28000） | **G-L5-4** 4 占位锚 + ending 入口观测锚（`$26` 写 A 级；入口归属 B 级） | 新 oracle 场景（C4，输入不变） | GUI ~6min | shots 语义 = stage8 关内/关末/95F9；trace 链同 §2 |
| S2 | C2 relay overlay 加 `watch $CA/$CC/$C8/$0630/$064B/$048C` | **G-L5-2**（stage7/8 boss）+ **G-L5-3** 流末游标 | 改 watch（新 overlay，输入不变） | 120000f ~8min | boss kill 段/流末链可归属 stage |
| S3 | gs5-stg01-full overlay 加 boss watch | G-L5-2 stg01 boss | watch overlay | 15000f ~2min | stg01 boss HP/kill |
| S4 | 自然八关 bot（v8）轨迹 + 全 watch（stage/boss/$CA/$CC/$C8） | G-L5-1/2/3 自然面 | 需 bot 修复/新场景 | 长 | 全链自然观测 |
| S5 | stg03/stg06-timeline 增语义 watch overlay（stage/flags/trig） | 关 3/6 内容锚 | watch overlay | 时长同场景 | 段锚可判 |

> 优先级：**S1 > S2 > S3 > S5 > S4**（S1 直接补 L5 5 占位的 4 个 + ending 入口，成本最低；S4 是自然面终局，待 bot 侧）。

## 7. 基线冻结判断材料（P0-0 全景；供 Lead 审）

**P0-0 现状（GUI trace + task-138 二进制 `00c29d6a…`）**：
1. **22 对 = 18 可渲 + 4 blocked:no-source + #14 replaced-invalid-asset**；18 对两跑 **36/36 png+idx 相同**（确定性成立）；4 blocked=dense overlay 133 张 any-frame 零命中（fade-clear/ending-1/2/3 负证据）；ending-4/5 已由 C1/C3 f33000+ 命中 `6ff345dc`（assisted 来源）；#14 expected 仍=boot-black 复制品待 §7.2 重采。
2. **L4 0 容差未达成**：18 对 diff min **0** / median **9,766** / max **58,159** / sum **242,555**；**0/18 actual==expected sha** ⇒ 只能冻结"GUI trace 版 actual 基线"，**不得宣称 L4 通过 / P0-0 全绿**。
3. **L5 B1 主判据达成**：`TRACE-DIFF 8/8 ALL MATCH` + F1 4/4；**F2 跨脚本相位差未收口**（task-145，不阻塞 P0-0 冻结）。
4. 去重：expected_unique=13 / actual_unique=14（四联/ending-4/5 同图；stg07 新 trace 下已不同图，旧 6-group superseded）。

**建议冻结文本（Lead 审）**：
> "P0-0 以**显式不全绿**冻结：18 对 canonical/assisted actual（两跑 36/36 确定性，二进制 sha `00c29d6a…`）入库为 L4 基线；4 对组 D `blocked:no-source`（dense 负证据）+ #14 `replaced-invalid-asset`（expected 待重采）如实登记；L4 0 容差**未达成**（0/18 actual==expected，diff median 9,766），进入 P0-1 收敛；L5 B1 8/8 + F1 4/4 已达成，F2 跨脚本相位差由 task-145 单独修复。冻结 commit 前按 §8.1 步骤 4/5 跑一次 `diff_visual.py --strict` 记录 FAIL（B3），visual 门保持默认模式不翻转。"

## 8. 验收自查

- [x] 12 条 GUI trace 的输入/字段/覆盖盘点（§1）；未跑 Mesen；
- [x] G-L5-4 A 级 trace 观测链（f27036/f27701/f27731/f27734 + stage9 段；`$95F9` 归属 B 级）与 5 占位候选帧（§2）；
- [x] G-L5-2/-3/-1 的可行/缺口分档与阻塞点（§3/§4/§5）；采集建议 S1-S5（§6）；
- [x] 冻结建议文本（§7）；未改工具/expected/gates；
- [ ] docs-links PASS（提交前复核）；独立复核 pass。
