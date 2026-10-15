# B2 TAS 输入生成战役 —— 条件输入 DSL（C4）+ 生成器 + 关 1 真机验证

> 日期：2026-10-15（campaign B2，派生自 B1 收口 + G-S5-6 负结果）｜ 角色：verifier（oracle 验证工程师）
> 目标：用 B1 波次时间线生成**真机可通关**的输入脚本（l5-plan C1-C4 工具+输入战役）。
> 证据分级依 AGENTS.md §3；**输入生成无 poke/无状态篡改**（README 写注入纪律），真机结果如实报（含负结果）。
> 设计文档先行：[../spec/tas-input-dsl.md](../spec/tas-input-dsl.md)（C4 条件输入 DSL + R-tas-* 规则）。

## 0. 验收对照（开工声明，未达不结案）

| # | 验收 | 状态 |
|---|---|---|
| ① | 条件输入 DSL/生成器设计+实施（可跑） | ✅ §1/§2——`docs/spec/tas-input-dsl.md`（设计）+ `tools/tas_gen.py`（实施，含 timeline/gen/record-to-scenario 三面）+ `data/tas/*.tas`（计划）+ 生成物 `oracle/scenarios/tas-stg01-v2*.txt` 真机跑通 |
| ② | 关 1 自然通关（`$26 bit4` 自然置位）或显式负结果 | ⚠️ **显式负结果**（§4）：~~生成轨存活 **9204 帧**（盲开环 1787 → 5.1×）~~【2026-10-15 更正（复核 #78 F3/F4）】生成轨**战斗活跃 9592f（pstate==1）**；f9204 = **1 帧 pstate15 瞬态（非真死）**，真死对照 = f10140（128f 冻结+残机−1）→ 同类对比 **5.68×**（f10140 vs 盲开环 f1787），抵达**中 boss 竞技场**（tick 816/824），但 **type $4A 对象未击杀 → 记录 1 不完成 → 滚动不恢复 → 时间线停在 tick 824**，arena 内 6 死 → GAME OVER（f11299）；`$26 bit4` 关末语义自然触发未观测 |
| ③ | save-state（C2）/RNG（C1）探测结论 | ✅ §5/§6——**C2 可行**（`emu.createSavestate/loadSavestate/getState/setState` 存在 + `setState(getState())` round-trip ok；`createSavestate` 须 exec 回调上下文=错误信息原文）；**C1 = 无需冻结**（RNG 状态机 zp `$2B/$2C/$2D`，纯输入决定论，重复 run 全字段 0 差异） |
| ④ | golden 回填 | **不适用**（②未达成——按验收口径"若 ② 达成"才回填；injected 锚不动、占位 fail-closed 不发明） |
| ⑤ | 三件复跑不降 | ✅ §7——144/144 + 121/121 + 12/12 |

---

## 1. 条件输入 DSL（C4）设计（详见 spec）

**两层裁定**：交付物 = **开环输入轨**（`oracle/scenarios/tas-*.txt`，确定性回放）；
条件逻辑在**生成期**对时间线模型求值后展开为 `press`/`hold`（生成物不残留条件指令）；
运行时条件（bot 模式）只是构造开环轨的探索工具，产出须走"记录→回放复核"闭环。

规则集 R-tas-1..7 逐条挂已证参数（spec §2）：射击保持（B 长按，A 级）/ 炸弹应急（A 边沿投放 + 消耗表）/
移动走廊（钳制域+速度对，A 级）/ 横扫（**C 级经验启发式，明示**）/ 避让 margin（命中包络候选 12px+4px，B 级）/
**弹速表语义未证（Wt-6）→ 显式不做弹道预测** / 通关判定 `$26 bit4`（A 级）。

## 2. 生成器实施（tools/tas_gen.py）

- `gen <plan.tas>`：计划 DSL → scenario（纯 press/hold）+ 报告 JSON（含时间线投影与假设清单）；
- `timeline`：B1 §2.2 调度流逐记录解码 + tick→帧投影（复用 `wave_timeline.iter_sched_records`，
  该函数为本轮对 `decode_sched` 的**结构化重构**，输出逐字节不变（`diff` 自证 §7）；
- `record-to-scenario`：bot 输入日志（逐帧按键）→ 开环轨（RLE）；
- **fail-closed**：计划里出现 `poke/poke_every/snapshot/restore` 直接拒绝（生成器无写注入能力）。

计划文件 `data/tas/stg01.tas`（+ 死因取证变体 `stg01-diag.tas`）；生成物头注强制含规则出处 + 无 poke 声明。

## 3. 关 1 自然通关路径（引擎事实，A/B 级）

| 事实 | 内容 | 证据 |
|---|---|---|
| 关末门 | 波次记录机记录边界消费 `$CA bit0` → `bank00:8CFC` 置 `$26 bit4` | A（B1 §1.4） |
| STG-01 波次记录流 | `$A3CD`（rec9 @tick `$330`=816）、`$A3E4`（rec11 @tick `$4D0`=1232） | A（`wave_timeline.py script 1`） |
| 记录 1 = **中 boss 竞技场** | `$A3CD`：$C9=$F2/$CA=$00、spawn 清单 `[01][4A]`、字段[2]=08；装载时保存滚动对（`bank00:8992-899E`）→ `$194/$195 := tick+8`（**减速截止 tick 824**）→ `sub_8D3A` 按 `$8D84,Y` 表逐 tick 减速到 0 | A（ROM 字节 + §4 trace 实测减速曲线） |
| 记录 1 完成条件 | `$C9 bit4` → 槽扫描等待（`bank00:8A0A-8A3B`）：spawn 清单对象（type `$4A`）被清空 → 记录完 → 恢复 `$80/$81/$82/$83/$7F`（`bank00:8CC9-8CFA` 滚动恢复） | A |
| 记录 2 = **boss 段** | `$A3E4`：$C9=$FB/**$CA=$05（bit0=1）**、spawn 清单 `[01][5F]` → 记录完 → **`$26 bit4`** | A（ROM 字节 + `bank00:8985/8D00`） |
| ⇒ 通关条件 | **击杀 type `$4A`（记录 1）→ 滚动恢复 → 时间线推进到 tick 1232 → 击杀 type `$5F`（记录 2）+ 槽清空** → `$26 bit4` | A/B（命名未证：一律按表值称呼，禁发明） |

## 4. 真机结果（tas-stg01-v2，13000f；`python3 tools/scenario_run.py run tas-stg01-v2`）

**输入**：gameplay-stage1 同源前缀（f1..1200，确定性对账）+ R-tas-1 B 长按（f1250-12999）+
R-tas-4 二维三角扫（x 40-200 / y 150-215，不触钳）+ R-tas-2 时间线驱动炸弹 ×12。

| 项 | 实测 | 对照 |
|---|---|---|
| **首死** | ~~**f9204**（tick 785）~~【2026-10-15 更正（复核 #78 F3）】**f9204 = 1 帧 pstate15 瞬态**（tick 785；f9205 续移 (116,158)、残机不变、金钱不变——**非真死**）；真死对照 = **f10140**（128f 位置冻结 + 残机−1 + 金钱下降） | ~~盲开环（G-S5-6a）f1787；探针（B 长按+单轴扫）f2945 ⇒ **5.1× / 3.1×**~~【2026-10-15 更正（复核 #78 F3）】同类对比 **5.68×**（f10140 vs 盲开环 f1787）/ **3.44×**（f10140 vs 探针 f2945）；原 5.1×/3.1× 系 f9204（1f 瞬态）对 128f 真死窗的**非同类对比**（双口径并列；方向不变，改进反而更大） |
| 死亡序列 | f9204 / 10140 / 10448 / 10736 / 11014 / 11298（6 死；**f9204=1f 瞬态**，其余 5 窗各 128f 真死） | ~~死亡窗 641f + 复活 1777f 非活跃，存活 **10582f**~~【2026-10-15 更正（复核 #78 F4）】分解更正见表下 dated 全表（pstate 全帧直方图口径） |
| 命数 | 2 → 3（f3228 奖命）→ 4（f9426 奖命）→ 3/2/1/0 | 盲开环 4 死→GAME OVER |
| **时间线推进** | tick 16→**824**（f9916 起停） | 波次记录 1 装载 tick 816 = **f9689（投影值与实测全等）** |
| 中 boss 竞技场 | ~~f9689 出敌~~【2026-10-15 更正（复核 #78 F2）】**f9689 = 波次记录 1 装载帧（tick 816；投影 5561+(816−558)×16=9689 与实测全等）**，非出敌；slot27 type $4A **首现 f9930 自顶部入画**（f9900 全场无 type 74；出敌点 ≈ f9910-9930=滚动停止前后；下行轨迹 f9930 (153,15)→f9960 (148,35)→f9990 (155,54)→**f10020 (153,75)=§4.1 自有首点**，快照 30f 粒度）→ f9706-9916 滚动减速（tick 间隔 16→17→18→19→21→24→29→40→59）→ f9916 起滚动 0（tick 停） | §3 记录 1 语义逐项吻合 |
| GAME OVER | f11299 `$26 bit3`（0 残机臂） | 盲开环 f5182 |
| `$26 bit4` | 仅 f913-1192（菜单→开 stage 过渡窗，非关末语义）；**关末语义自然触发未观测** | G-S5-6 F1 同口径 |
| `stage_no` | 恒 1（`INC $180` 未发生） | — |
| **v3（相位切换版）** | 首死同 f9204（前缀同源）；arena 5 死 → GAME OVER；tick 同停 824 | §4.1 v3 表 |

> 【2026-10-15 更正（复核 #78 F3/F4）】上表"首死/死亡序列"口径更正 + **时间分解全表**（口径 = pstate
> 全帧直方图 `{1:9592, 0:1777, 108:987, 15:641, 124:3}`，v2 trace 逐帧直读可重导）：
>
> | 口径段 | 帧数 | 区间/语义 |
> |---|---|---|
> | **战斗活跃（pstate==1）** | **9592f** | f1193-11425 区间 − 641f 死窗（"存活"唯一口径） |
> | 死亡窗（pstate==15） | 641f | f9204（**1f 瞬态，非真死**）+ f10140/10448/10736/11014/11298（各 128f 真死：冻结+残机−1） |
> | 前奏菜单（pstate 108/124） | 990f | f203-1192（987+3）——旧"存活 10582f"误计入此段 |
> | boot + GAME OVER 后（pstate==0） | 1777f | f1-202 + f11426-13000（**非"复活非活跃"**：复活即回 pstate=1，无非活跃窗） |
>
> 旧分解"死亡窗 641f + 复活 1777f 非活跃，存活 10582f"作废（13000−641−1777 把前奏 990f 计入存活）；
> "存活 9204f"= 首个 pstate15 事件帧（1f 瞬态）——§0② 与本表"存活"同名双义一并消除，
> **战斗活跃口径 = 9592f = pstate==1**。

**负结果主干（②）**：本输入轨**未能自然通关关 1**——差额不在"闪避存活"（~~存活 9204f~~
【2026-10-15 更正（复核 #78 F3/F4）】战斗活跃 9592f；f9204=1f 瞬态、真死 f10140 起，已到
内容线末段竞技场）而在 **中 boss（type `$4A`）未击杀**：记录 1 不完成 → 滚动不恢复 →
时间线冻结在 tick 824 → 波次记录 2（type `$5F`，tick 1232）与关末门永不触发。
arena 内 f10140 起每 ~290f 一死（复活后 ~170f 再死），残机耗尽 → GAME OVER。
**"哪段弹幕不可避"未证**：arena 弹幕形态/中 boss 血量属未证语义（Wt-6/敌弹 RAM 布局 W-tas-1），
死因取证（§4.1）只到"敌对象接触"层级——**不宣称不可避**，如实登记为未证缺口。

### 4.1 死因取证（同输入诊断 run，capture_every 30 全 RAM 快照）

`oracle/scenarios/tas-stg01-v2-diag.txt`（同 v2 输入轨 + 每 30 帧 2KB RAM 快照，383 点）→
`analysis/traces/tas-stg01-v2-diag_f*_arena.ram.hex`。取证结果（机器可从快照重导）：

| 发现 | 内容 | 级 |
|---|---|---|
| **中 boss = slot27 type `$4A`（74）** | f10020 (153,75) → f10260 (70,186) → f10560 (194,154) → f10800 (127,158) → f11400 (93,166)：**降至 y≈150-190 带后左右徘徊（x 70-194）**，字段 ~~`$0630`=$0C 恒定~~【2026-10-15 更正（复核 #78 F5 附项①）】`$0630`：f10020=**$0E**、**f10260 起**=$0C（"恒定"仅限 f10260 起）、`$05DC`（命中倒数）=00 恒定、`$06F4`（到期）=$10 | A（RAM 快照逐点） |
| **死因 = 敌对象接触（非弹幕）** | v2 横扫带 y 150-215 与中 boss 徘徊带 **y≈150-190 重叠** ⇒ 6 次死亡帧的快照上下文均为近距离敌对象（含 f10140 前 30f slot24/25 type `$12` @(136,169)/(162,172) 与玩家 (142,193) 同带）；**未观测到独立敌弹对象**（~~弹网格活动槽 0-4，型码 0-2 系自机弹~~【2026-10-15 更正（复核 #78 F5 附项③）】弹网格活动槽以首组 Y=$80-$84 为主，f10110/10140 含**组 2 槽 Y=$9C/$9D 偶发活跃**（型码 0-2 系自机弹无误）） | A（快照）+ **unknown（敌弹 RAM 布局 W-tas-1 未证，不宣称"无弹幕"）** |
| 未击杀原因未闭环 | `$05DC`/`$0630` 全程不变 ⇒ 本轨自机弹**未在中 boss 上产生可观测命中**（落点错位 or 需炸弹 or 判定域未证——三解并列，不发明） | A（字段不变）+ 显式缺口 |

> 【2026-10-15 更正（复核 #78 F5 附项②）】f10736 死因上下文偏弱：f10710 快照最近对象 **79.5px**、
> f10740 slot27 距 **14.9px**——快照 30f 粒度**未直接拍到接触帧**（与 v3 f11355 同款局限）；
> 上表"6 次死亡帧的快照上下文均为近距离敌对象"对 f10736 **不宣称接触帧**，按"到达接触前
> （快照粒度 30f）"口径读（v2 侧此前缺同款"不宣称"注）。

**v3 改进（C4 生成期条件相位切换）**：按时间线预测点（波次记录 1 tick 816 ≈ f9689）提前切底带
`y 208-223`（脱离接触包络 + 自机弹自下贯穿 boss 带）——`oracle/scenarios/tas-stg01-v3.txt`（f9550 起切带）。
**v3 实测 = 改善有限（诚实登记）**：主战斗段输入与 v2 逐指令同源 ⇒ 首死同帧 f9204；
arena 5 死（f10218/10506/10782/11080/11355）→ GAME OVER。死因取证（~~383 快照~~
【2026-10-15 更正（复核 #78 F5）】**433 点快照**（13000/30，run.json captures=433；"383"系 v2-diag 点数的复制笔误）：

| 死亡帧 | 玩家 | 近距对象（±30f 快照） | 判读 |
|---|---|---|---|
| f10218 | (172,199) | slot27 中 boss @(148,182) | 底带边缘仍被 boss 下探触及（dy=17） |
| f10506 | (115,163) | slot27 @(130,159) | **复活点 (128,160) 落在 boss 徘徊带内**——死亡窗 128f 后复活，开环输入无法按复活时刻重定位 |
| f10782 | (155,154) | slot25 type `$22` @(132,148) | boss 召唤物（type `$22` 系，f10500+ 出现） |
| f11080 | (109,163) | slot27 @(124,155) | 同上：复活点带内接触 |
| f11355 | (139,156) | （30f 内无近距对象） | 到达接触前（快照粒度 30f），不宣称不可避 |

**v3 教训（C4 表达力边界）**：开环轨对"复活时刻"不可观测——若要吃满相位切换收益，
需 `record-to-scenario` 闭环（bot 在复活帧重定位 → 记录 → 回放）或 save-state 段级接力（C2）；
本战役按验收口径止步于负结果 + 教训登记。

### 4.2 确定性对账（C1 侧证，§6）——**PASS**

v2（13000f）与 v2-diag（11500f，两次独立 Mesen2 run，输入轨逐指令同源）前 **11500 帧 × 17 watch
字段逐帧比对 = 0 mismatch**（含 `rng_2b/2c/2d` RNG 状态三字节逐帧）；死亡帧序列全等
（9204/10140/10448/10736/11014/11298）。⇒ **输入轨决定论成立**（C1 无需冻结 RNG）。

## 5. C2 save-state 探测（l5-plan C2）——**可行**

| 探针 | 结果 | 证据 |
|---|---|---|
| API 枚举（`oracle/probes/c2_api_probe.lua`） | `emu` 60 键：**`createSavestate`/`loadSavestate`/`getState`/`setState` 全存在**；`saveStateFile/loadStateFile/saveState/loadState` = nil | A（`oracle/probes/c2_api_probe.out.txt`，探针输出原文留档） |
| round-trip | `setState(getState())` **ok=true**（315 键状态表：cpu/ppu/mapper/apu/frameCount/clockRate…） | A（同上） |
| `createSavestate` 约束 | **必须在 exec memory 回调上下文内**（错误信息原文：`This function must be called inside an exec memory operation callback for the main CPU`） | A（`oracle/probes/c2_relay_probe.lua` 实跑） |
| 接线残项 | exec 回调注册成功（ok=true）但探针选点（NMI 向量 `$EF85`）未触发 → **段级接力"已跑通"未证**（机制面成立，工程接线待 callbackType 枚举确认） | A（`oracle/probes/c2_relay_probe2.lua` 实跑，输出 `c2_relay_probe2.out.txt`） |

**结论**：Mesen2 Lua **支持 save-state**（C2 选项成立）——段级接力（关 1 通关态 → 关 2 起点）
的实现路径 = harness 内 exec 回调中 `createSavestate` → 落盘 → 下段 `loadSavestate`；
`getState/setState`（纯标量表，可 JSON 化）为备选（**其是否含 RAM 未证**：315 键中未见 RAM 键）。

## 6. C1 RNG 探测（l5-plan C1）——**无需冻结（确定性成立）**

- **RNG 状态机定位（A 级）**：zp `$2B/$2C/$2D`；`bank15:$EC12`（`$2C := ($2C+$2D+1) EOR $2B`，`$2D++`，RTS）
  与 `bank15:$EC1F`（`$2D++`；`A := (($2C),Y) EOR $2C EOR $2D + $2B → $2C`，RTS）；消费点族
  `bank15:E12C/E134/E140`（`bomb_type_index` 初始化等）。
- **熵源**：`$2B` 的 `INC` 站点（`bank00:8F0F/9320`、`bank01:A0EA`、`bank02:8261`、`bank15:E432`）
  ——全部为游戏内事件驱动，**无墙钟/无外部熵** ⇒ RNG 流是输入轨的纯函数。
- **实证**：同输入轨两次 run（v2 / v2-diag）全 watch 字段（含 `rng_2b/2c/2d` 逐帧）对拍 = 0 差异（§7）。
- **结论**：C1 的"注入/冻结 RNG"对 TAS **不必要**——确定性由输入轨保证（R-l5 系"同输入同 trace"先例）。
  若移植端需要对拍 RNG 流，直接按 `$2B/$2C/$2D` 状态机实现即可（W4/S4 的 `GameRng` 占位 LCG 可据此替换）。

## 7. 三件复跑 + 纪律自查（验收⑤）

```
# ① 工具重构回归（wave_timeline.decode_sched → iter_sched_records，输出逐字节不变）：
python3 tools/wave_timeline.py sched 1 | diff /tmp/sched_before.txt -   → 无差异
python3 tools/wave_timeline.py spawns  | diff /tmp/spawns_before.txt -  → 无差异
# ② 三件复跑（2026-10-15 实跑）：
cargo test --workspace → 144 passed / 0 failed（12 个 test result 全 ok，合计 144；基线 144 不降）
python3 tools/diff_subroutines.py → L2-DIFF: 121/121 通过（golden==ROM 执行==game-core 同名函数）
python3 gates/cli.py check → SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0；CHECK: PASS（0 静默跳过）
   （rom-integrity/seed-integrity/naming-ledger 206/rust-tests 144/subroutine-diff/trace-diff/
     visual-diff/completeness-ratchet 19/addr-bridge 全 PASS；基线对照 8 项全 OK）
# ③ 确定性对账（C1）：v2 ↔ v2-diag 11500 帧 × 17 字段 = 0 mismatch（§4.2）
# ④ 生成器可复现：tas-stg01-v2.txt 重生成 sha256 = run.json 记录的 DSL sha256（0882cb26…）全等
```

**纪律自查**：
- 输入生成无作弊：生成器无写注入能力（poke 类指令 fail-closed 拒绝）；真机场景全程 0 poke
  （`tas-stg01-v2*` 文件可查）；门位 `$26 bit4` 全程只在 f913-1192 过渡窗自然出现（非注入）。
- 闪避规则出处：移动常量/钳制/命中包络/射击门控/炸弹消耗全部 A/B 级指针；**弹速表（Wt-6）显式不消费**；
  横扫节律明示为 C 级启发式，不冒充已证弹道。
- 真机结果诚实：负结果主干（未通关）如实报；"弹幕不可避"未证即不宣称；投影帧号（tick→帧）标注外推假设。
- 不动 ROM/参考轨道/台账（零 rename）；oracle 侧只增场景与生成工具，既有阈值/门禁零改动。

> 【2026-10-15 补记（B2 修复循环，复核 #78 F1-F5 + A1）】修复收尾复跑（四件全绿 + 锁同步）：
> - **F1 TDD**：`python3 tools/test_tas_gen.py` → **14/14 通过**（RED 3/14 → 修 → GREEN；新增
>   `tools/test_tas_gen.py`，咬合 when 死语法/fail-open、when 内嵌写注入、白名单外动词、畸形 when、
>   路径穿越、CLI exit=1 不落盘、既有 fail-closed 面不回退）；fail-closed 全矩阵 15/15 留证
>   （顶层 poke/poke_every/snapshot/restore/POKE + when 内嵌 poke/snapshot/restore/frobnicate/
>   hold/press/avoid/畸形 + `--name ../evil` → 全部 exit=1 且不落盘）。
> - `cargo test --workspace` → **144 passed / 0 failed**（基线 144 不降；Python 工具测试不入 cargo 计数）。
> - `python3 tools/diff_subroutines.py` → **L2-DIFF: 121/121 通过**。
> - `python3 gates/cli.py check` → **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0；CHECK: PASS**（0 静默跳过）。
> - `tools/tas_gen.py gen` 三场景重生成 → **sha 三重锁全等不变**：v2 `0882cb26…` / diag `ea4ded70…` /
>   v3 `06eaa85d…`（重生成 == 入库 == run.json scenario_sha256）——F1/A1 修复不改生成物字节
>   （when 拒绝不触既有计划、路径消毒不触既有名、`BOMB_DEPLOY_DEDUP_HINT` 改名不触输出注释文本），
>   **run.json 历史记录零改写**（不 falsify 过往 run）；`.report.json` 随重生成更新（假设注改写 +
>   `unimplemented_dsl` 声明 + diag 归一 bands 字段），不在 sha 锁面。仓库无锁测试（sha 仅存于
>   run.json 与本文件 §7 记录）→ 无需同步测试记录。

## 8. 遗留缺口（不发明）

| # | 项 | 去向 |
|---|---|---|
| Wt-2′ | ~~tick 与滚动耦合~~ **本轮实测裁定：tick = 16px 滚动进度**（2px/f×8f = 8px/f×2f = 1px/f×16f = 16px 三段全等 + 减速段 tick 间隔随滚动减速递增 + 滚动 0 时 tick 停）→ 可回填 B1 Wt-2 | ~~建议 B1 原地更正 + 挂 S1~~【2026-10-15 落地（复核 #78 修复循环）】已回填：enemy-timeline §8 Wt-2 行 dated 裁定关闭（引复核 #78 §4 四证链）+ tas-input-dsl §7/§4 同步 + tas_gen.py tick 注释同步；**保留 W-tas-3**（tick 816→1232 尾段外推未实测，不因裁定关闭） |
| W-tas-1 | 敌弹 RAM 布局（`$70` 弹网格敌弹归属）未证 → arena 弹幕形态不可判 | §4.1 快照 + t32 两 SoA 交接 |
| W-tas-2 | type `$4A`/`$5F` 对象（中 boss/boss）血量/弹型未证 | 波次记录机记录字段全解（B1 Wt-8） |
| W-tas-3 | tick→帧尾段外推（16f/tick 假设至 tick 1232） | 中 boss 击杀后继续 trace 实测 |
| W15 | 玩家命中判定盒常数（包络候选） | 命中例程定位 |
