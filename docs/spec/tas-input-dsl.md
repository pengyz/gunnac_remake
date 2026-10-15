# spec: TAS 条件输入 DSL + 输入生成规则（tas-input-dsl）→ docs/spec/tas-input-dsl.md

> 状态：2026-10-15 v1（campaign B2，verifier/oracle 验证工程师）。**设计先行**：本文件是
> `tools/tas_gen.py` 的规格（C4 条件输入 DSL + 闪避/射击/炸弹规则）；实施见同文件 §6 实施注。
> 判定物 = `oracle/scenarios/tas-*.txt`（**纯输入轨**）+ 真机 trace（`$26 bit4` 自然置位）。
> 证据分级依 AGENTS.md §3；**规则只允许来自已证参数（弹速表/敌型特征/移动常量/命中包络），
> 未证语义（弹速表语义 Wt-6、敌弹 RAM 布局、boss 血量）一律显式 unknown，不发明**。

## 0. 目标与红线

| 项 | 内容 |
|---|---|
| 目标 | 输入 = f(波次时间线, 弹幕参数) → 按键序列（何时左/右/射击/炸弹），真机自然通关关 1 |
| 通关判据 | **`$26 bit4` 自然置位**（波次记录机流末记录 → `bank00:8CFC LDA $CA/AND #1/ORA $26`，A 级）——**无 poke** |
| 红线 | **只生成输入（按键）**；禁 poke/禁改 ROM/禁改 game 状态（README 写注入纪律：poke 仅限 G-S5-6 门位注入语境） |
| 交付形态 | **开环输入脚本**（scenario DSL `press`/`hold`）——可确定性回放复现；条件逻辑在**生成期**解析 |

## 1. 条件输入 DSL（C4）——两层设计

### 1.1 语义定位（l5-plan C4 对照）

l5-plan C4 给二选一：`when <watch条件> then press/hold`（运行时反应）**或**离线生成
"工具辅助输入轨"后回放开环播放。本设计取**后者为交付物、前者为生成工具**：

```
┌─ 生成期（条件 DSL，tools/tas_gen.py）──────────────────────┐
│  输入：波次时间线（B1 §2.2/§2.3）+ 弹幕参数 + 规则集        │
│  条件求值：对时间线模型（frame→预期敌群集合）求值 when 子句  │
│  输出：oracle/scenarios/tas-*.txt（开环 press/hold 序列）    │
└──────────────────────────────────────────────────────────┘
                    ↓ 真机 Mesen2 回放（无条件执行）
              trace 判定（$26 bit4 / $0180 / 死亡窗）
```

**为什么不开环交付 + 运行时条件不分家**：L5 判定物必须是**确定性输入轨道**
（`oracle/scenarios/*.txt`，README「场景 *.txt 是入库的真相源」）；运行时 `when` 分支
会让"同一场景两次跑出不同输入"，破坏 A5 输入纯净对账与 L3 帧级差分。因此：

- **交付物 = 开环轨**（唯一真值）；
- ~~**生成期条件**（本文件 §2）是表达力来源，解析后展开成 `hold`/`press`；~~
  【2026-10-15 更正（复核 #78 F1）】展开实现不存在（when 死语法）——现档位=**显式拒绝**
  （见 §1.2 更正注）；已验证的生成期条件载体 = 多段 `band <起帧>`；
- **运行时条件**（§5 bot 模式）只是构造开环轨的**探索工具**：其产物必须走
  "记录输入 → 回放复核"闭环，锚点标注 `trigger=bot-derived`（若回放复现则升 `natural`）。

### 1.2 生成期条件 DSL 语法（plan 文件，`data/tas/*.tas`）

```
stage 1                          # 关号（B1 §1.1 目录 $180）
frames <N>                       # 总帧预算
prefix gameplay-stage1           # 同源前缀（确定性对账：f1080/1200/1260/1500 四锚）
fire <起>-<止>                    # B 长按连发（R-tas-1）
bomb budget <N>                  # 炸弹预算（$018F 上限 20；R-tas-2）
band x <lo>-<hi> y <lo>-<hi>     # 闪避走廊（钳制范围内；R-tas-3）
sweep period <F> amp <PX>        # 横扫节律（R-tas-4）
avoid x <lo>-<hi> for <F> from <帧>   # 生成期条件产物：禁入 x 区间（R-tas-5）
when <pred> then <action> [for <F>]   # 【未实施·显式拒绝】条件子句——见下方 2026-10-15 更正注
```

> 【2026-10-15 更正（复核 #78 F1）】`when` 语法**降级为"未实施（显式拒绝）"**——原"生成期求值/展开"
> 承诺与实施背离（解析器收集 `when` 后**零消费者**，静默丢弃 exit=0 = fail-open 漏洞）。现行档位：
> - `when` 子句**一律 PlanError 拒绝**（exit=1 不落盘）；子句动作动词白名单 = {hold, press, avoid}，
>   白名单外动词（含 poke 类写注入）按各自语义拒绝——**写注入伪装在子句内同样 fail-closed**；
> - 档位理由：展开档位需对谓词 `spawn_near`/`cluster_active`/`slot_count` 定义时间线模型求值语义
>   （未证——未证不发明）；已验证的条件相位切换路径 = **多段 `band <起帧>`**（v3 实测有效）；
> - **生成物不残留条件指令**的承诺依旧成立（因 when 根本不进生成管线，显式拒绝）；补实施展开档位
>   时须先定谳谓词语义（挂 W-tas-4）。

- ~~`when` 谓词（生成期，对时间线模型求值）：`spawn_near(x, y, r)` / `cluster_active(<记录>)` /
  `frame in [a,b]` / `slot_count >= n`~~（谓词求值未实施，见上更正注）
- ~~`when` 动作：`hold <方向> <F>f` / `press a`（炸弹）/ `avoid x <区间> <F>f`~~（动作展开未实施；
  白名单定义保留为将来展开档位的动作面）
- ~~所有 `when` 在生成期展开为最终 `hold`/`press`；**生成物中不残留任何条件指令**。~~
  【2026-10-15 更正（复核 #78 F1）】"生成期展开"实现不存在（死语法）；现档位=显式拒绝（见上注）。

### 1.3 运行时条件（bot 模式，§5）——工具面（非交付物）

`when watch(<addr>) <op> <val> then <action>` 在 Lua harness 求值（读 RAM → 按键），
仅用于探索可存活输入；**产出必须转写为开环轨再回放复核**。

## 2. 生成期规则集（R-tas-*，逐条挂已证参数）

| 规则 | 内容 | 证据（级） |
|---|---|---|
| **R-tas-1 射击保持** | 战斗段全程 **B 长按**（`hold <起>-<止> b`）：主武器 = `$18` bit6 按住 → `$39/$3A` 自动重复计时器（$39=8）→ 六型生成器，**每 ~7 帧一发** | A：`bank3:$A1A9 LDA $18/AND #$40` 门控链 + fire-hold 动态（`docs/findings/shot-gating-f1.md` §1；R-shot-gen-2） |
| **R-tas-2 炸弹应急** | A 边沿触发投放（`$16` bit7）；消耗 `bomb_ammo` $018F（消耗量表 `[1,2,3,2]`，`bank03:$A336`）；投放态 `$0401=7` 在途（寿命=行为链合成值，非恒定）→ **同窗不重复投放** | A：R-fire-1/2/3 + N-0049（`bank03:$A2B4/A2BA/A2F7`）；上限 20（商店「导弹20个 上限」） |
| **R-tas-3 移动走廊** | 走廊 ⊆ 钳制域：X `$0E-$F3`、Y `$14-$DF`；位移速度 ±2.8125 px/帧（亚像素对 `(208,2)`/`(48,253)`） | A：C-clamp-x/y、C-vel-h（R-move-3/4/6，`bank03:A084-A0E8`） |
| **R-tas-4 横扫节律** | 常速横扫（不停留）：移动目标相对敌弹瞄准/敌接触更难命中——**经验规则（C 级，明示）**，非已证弹道 | C（明示为启发式；效果以真机 trace 判定，不冒充证据） |
| **R-tas-5 敌群禁入区** | 由 B1 §2.2/§2.3 时间线生成 `avoid`：出敌簇激活窗 ± 延迟内，禁入簇 x 区间 ± margin；margin = **16px**（命中包络候选 \|dx\|≤12/\|dy\|≤12 +4px 余量） | A（时间线/坐标）+ B（命中包络候选，`docs/findings/hit-sampling-h16.md` §2；**非判定盒定谳 W15**） |
| **R-tas-6 弹幕参数边界** | 弹速表 `bank5:$AC5F`（8 项 `D3 D7 DB DF E3 DF DB D7`，Y=AND#7 定宽）**语义未闭环（Wt-6）**：生成器只登记"存在弹速参数表 + 三读点 $0684/$0534/$0518"，**不得据其推导闪避弹道**；闪避余量一律经验值 + 真机判定 | A（表字节/读点）+ 显式 unknown（表语义） |
| **R-tas-7 通关判定** | 真机 trace `$26 bit4` 自然置位（波次记录机流末记录消费）→ 关末链；`$0180` 1→2 | A：`bank00:8CFC` + G-S5-6 Run C 链（trigger=injected 先例；本次要求 natural） |

**未证语义清单（生成器显式不消费）**：敌弹 RAM 布局（两 SoA：`$1C` 网格 28 槽 + `$70` 弹网格，
敌弹归属未证）、弹速表语义（Wt-6）、boss 血量/弹型（B1 §1.4 波次记录流未全解）、
命中判定盒常数（W15）。⇒ 闪避规则只能是**几何回避 + 真机迭代**，不能是弹道预测。

## 3. 波次时间线输入（B1 §2.2/§2.3 → 生成器数据源）

- **静态**：`python3 tools/wave_timeline.py sched 1`（调度流逐记录：`(组员type, 组员数, 出敌间隔f, 参数)`）
  + `python3 tools/wave_timeline.py script 1`（脚本层 tick 排程）。
- **动态**：`python3 tools/wave_timeline.py spawns`（120 出敌事件帧锚；**口径注**：`0→非0` 判定式，
  槽复用写入不可见 → 生成器按槽复用保守处理：同槽再激活视为新威胁）。
- 生成器内嵌 STG-01 时间线（由工具输出重导，见 §6 复现命令）；关 2..8 时间线已定位
  （B1 §4.1，17 条 op5 波次流）但**未解码到内容级** → 本次只生成关 1 输入（显式范围）。

## 4. 关 1 通关所需的引擎事实（生成器的"终点"模型）

| 事实 | 内容 | 证据 |
|---|---|---|
| 关末门 | `$26 bit4` 由**波次记录机记录边界**置位（`$CA bit0` → `bank00:8CFC`） | A（B1 §1.4/§5） |
| STG-01 波次记录流 | 2 条：`$A3CD`@tick `$330`（rec9）、`$A3E4`@tick `$4D0`（rec11） | A（`wave_timeline.py script 1`） |
| 记录内容 | `$A3CD`：$C9=$F2/$CA=$00，spawn 清单 `[01][4A]`；`$A3E4`：$C9=$FB/**$CA=$05（bit0=1）**，spawn 清单 `[01][5F]` | A（ROM 字节 + `bank00:8985/898A` 装载 `$CA/$C9`） |
| 完成语义 | 记录状态链等待其 spawn 清单位占用的敌槽清空（`bank00:8A04 $C9 bit4` → 槽扫描等待；`$D2=0` 无超时）→ 记录完 → `$CA bit0` → **`$26 bit4`** | A（`bank00:8A0A-8A3B/8B25-8B49/8CC9-8D0C`） |
| ⇒ 通关条件 | **tick `$4D0` 波次记录 2 的 spawn（type=$5F）及其后敌槽清空**（自然通关=击杀该对象 + 清场） | A/B（记录格式 A；"type $5F=boss"命名未证，按表值称呼） |

**tick→帧换算（B1 §8.1 裁定）**：稳态 1 tick = 8 帧；关首快进环 = f1193 单帧 17 tick
（B2 移植口径必守）。`$84` 初值 `$10`（f1193）→ tick T 的稳态帧 ≈ `1221 + (T−0x10)×8`
**仅在恒 8f/tick 前提下成立**；脚本 rec6/rec8/rec12 有 set-speed 记录（`$81`=$88/$01/$08），
~~tick 是否随滚动变速（Wt-2）未证 → 生成器对终点帧号**给区间不给点值**，真机 `$84` watch 实测。~~
【2026-10-15 更正（复核 #78 §4，Wt-2 关闭）】tick **随滚动变速已证**（tick = 16px 滚动进度，A 级，
见 §7 Wt-2 行）；生成器对终点帧号仍**给区间不给点值**——依据降级为 W-tas-3 尾段外推未实测
（tick 816→1232），真机 `$84` watch 实测口径不变。

## 5. bot 模式（运行时条件，工具面）

- 语法：`when watch(<addr>) <op> <val> then <action> [for <F>]`（在 harness 内逐帧求值）。
- 用途：探索可存活输入（如 `when pstate==0x0F then hold left 30`（死亡窗）/ 敌接近时横移）。
- **闭环纪律**：bot 产出 `<name>.inputs.jsonl`（逐帧按键）→ `tools/tas_gen.py record-to-scenario`
  转写开环 → **同输入重放**必须复现同 trace（确定性对账），否则该轨作废。
- 锚点标注：bot 直接观测到的事件 = `trigger=bot-derived`；回放复现后才可标 `natural`。

## 6. 实施注（tools/tas_gen.py）

- CLI：`python3 tools/tas_gen.py gen <plan.tas> [-o oracle/scenarios/…]` / `… record-to-scenario …` / `… report …`
- 生成物头部强制写：**规则出处注**（R-tas-* 逐条）+ **无 poke 声明** + 时间线数据来源命令。
- 生成器**不得**内嵌任何写 RAM/改状态能力（只输出 press/hold；`poke` 明确拒绝）。
- 复现命令（时间线数据源）：

```bash
python3 tools/wave_timeline.py sched 1      # B1 §2.2 调度流（避让区数据源）
python3 tools/wave_timeline.py spawns       # B1 §2.3 出敌事件（避让区数据源）
python3 tools/tas_gen.py gen data/tas/stg01.tas   # 生成输入轨（产物 = oracle/scenarios/tas-stg01-*.txt）
python3 tools/scenario_run.py run tas-stg01-v1    # 真机回放 + trace 判定
```

## 7. 遗留假设（不发明）

| # | 项 | 级 | 影响 |
|---|---|---|---|
| Wt-6 | 弹速表语义（表宽/索引/单位）未闭环 | C | 无法做弹道预测闪避（R-tas-6 显式禁用） |
| W15 | 玩家命中判定盒常数（包络 \|dx\|≤12,\|dy\|≤12 为 2 样本候选） | B | 避让 margin 16px 为经验值 |
| W-tas-1 | 敌弹 RAM 布局（`$70` 弹网格敌弹归属）未证 | C | bot 模式只能按"非本机弹的对象"保守回避 |
| W-tas-2 | type $5F 对象（波次记录 2 spawn）为 boss 的命名未证 | C | 一律按表值称呼（纪律：不发明名字） |
| W-tas-3 | tick→帧尾段外推未实测（tick 816→1232：arena 停滚期 tick 不推进、rec12 set-speed 后区间未测；tick558-815 的 16f/tick 已实测 257 tick 全 16f） | C | 中 boss 击杀后继续 trace 实测 |
| W-tas-4 | `when` 生成期展开档位未建（谓词 `spawn_near`/`cluster_active`/`slot_count` 求值语义未证）——现档位=显式拒绝（§1.2 更正注，复核 #78 F1） | C | 补实施前先定谳谓词语义；条件相位切换暂用多段 `band <起帧>` |
| Wt-2 | ~~tick 与滚动耦合（8f/tick 是否恒定）~~【2026-10-15 更正（复核 #78 §4，Wt-2 关闭）】tick = **16px 滚动进度**（A 级，非恒 8f/tick：8f/2f/ramp/16f 随滚动变速；三段全等 + `$8B` 逐帧 −2 直证 + bank00:8D3A 减速表逐指令——见 enemy-timeline §8 Wt-2 行裁定） | C→已裁定 | 已解；**W-tas-3 尾段外推（tick 816→1232）未实测仍为假设**（终点帧号给区间不变） |
