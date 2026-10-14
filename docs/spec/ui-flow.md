# spec: 界面与流程（ui-flow）→ docs/spec/ui-flow.md

> 状态：2026-10-12 v1（任务 t6，spec-writer）→ **v2 同日 S2 回填**（任务 t17：t12 场景证据入规则）。
> 证据登记 E-xx 见 [completeness.md](completeness.md) §1；S2 场景证据指针格式 `scenario:analysis/traces/<id>` + 帧号。
> 纪律（AGENTS.md §3 / 角色契约）：只收录 **A/B 级证据支撑的可观察行为**；
> 显示时机/跳转逻辑等交互边界一律留"待 S2 取证"空位（§4），**禁止发明**。汉化特有文本单列 §3。
> 分子强度：规则面覆盖升 **L2**（绑 scenario/trace，completeness §6.2 硬规则）；含 web: 证据的格不超 L1。

## 0. 范围与关联格

| 模块 | 矩阵格 | 本版覆盖入口 | 分子强度 |
|---|---|---|---|
| 界面与流程 | B-SYS-UI | 标题画面（版权/计分显示）/ 标题菜单 / 文本库存在性 / 开机序列 / 已证流程序列段 / HUD 观测登记 | **L2**（R-ui-6..8 绑 scenario/trace） |

不在范围：商店面板（见 [shop-economy.md](shop-economy.md)）、计分规则（B-SYS-SCORE，仅关联）、
关卡内 HUD 的**行为规则**（R-ui-8 仅做观测登记，语义留台账 H-12）。

## 1. 文本库登记（本版已证存在性；显示条件未证）

| 串 | 指针 | 已证内容 |
|---|---|---|
| `HI SCORE` / `SCORE` | `rom:4272` / `rom:4283`（另一组 `rom:124107` / `124118`） | 计分显示标签 |
| `1990 @TOKYO SHOSEKI` | `rom:4291` | 标题版权行 |
| `PROGRAMED BY COMPILE` | `rom:4313` | 标题版权行 |
| `PRESENTED BY NEXOFT CORP.` | `rom:4336`（再现 `rom:7079`） | 标题版权行 |
| `LICENSED BY` + `NINTENDO OF AMERICA INC.` | `rom:4364` + `rom:4378` | 标题版权行 |
| `OPTION DEMO` | `rom:12264`、`rom:12635` | 菜单相关串 |
| `CONTINUE` | `rom:124126` | 结束流程串 |
| `GAME OVER` | `rom:124137` | 结束流程串 |
| `THANKS TO GUN-NAC,` / `THE INVADER IS DEFEATED` | `rom:12278` / `rom:12299` | 结局文本 |
| 开场文本（`INVESTIGATE THE EIGHT AREAS` 等） | `rom:12036` 等（E-R09） | 开场文本 |

## 2. 行为规则

### R-ui-1: 标题画面显示 HI SCORE 与 SCORE 两组计分显示位
- 证据: `rom:4272`/`rom:4283` + `shot:shots/auto_f00360.png`（像素可见 `HI SCORE 80000` / `SCORE 0`）
- 可判定性: 机器可判定（L4 截图 diff）
- 关联: B-SYS-UI、B-SYS-SCORE（后者分子仍 L0：计分规则本身未证）

### R-ui-2: 标题画面显示四行版权文本，内容与 ROM 文本库逐字一致
- 证据: `rom:4291`/`4313`/`4336`/`4364`+`4378` + `shot:shots/auto_f00360.png`（像素逐字吻合：`1990 @TOKYO SHOSEKI / PROGRAMED BY COMPILE / PRESENTED BY NEXOFT CORP. / LICENSED BY NINTENDO OF AMERICA INC.`）
- 可判定性: 机器可判定（L4 截图 diff + 文本串比对）
- 关联: B-SYS-UI

### R-ui-3: 标题菜单显示两个选项（汉化像素：「开始」「设置」）；ROM 文本库含 `OPTION DEMO` 串
- 证据: `shot:shots/auto_f00720.png` / `auto_f00900.png`（两选项 + 光标两态）+ `rom:12264`/`rom:12635`
- 可判定性: 机器可判定（截图 diff）；**菜单项像素文本 ↔ ROM 串的映射关系未证**（不发明，见 §4）
- 关联: B-SYS-UI

### R-ui-4: 文本库含 `GAME OVER` 与 `CONTINUE` 串；同区含 `HI SCORE`/`SCORE` 串（B 级观察：暗示结束画面含计分显示位）
- 证据: `rom:124137` / `rom:124126`（同区 `rom:124107`/`124118`）
- 可判定性: 机器可判定（字节复现）；显示时机仅人工核对（§4）
- 关联: B-SYS-UI

### R-ui-5: 文本库含开场文本与结局文本；本版是否实际显示、显示条件均未证
- 证据: 开场 `rom:12036` 等（E-R09：`INVESTIGATE THE EIGHT AREAS`、`SEVEN PLANETS`）；结局 `rom:12278`/`12299` 等（E-R10）
- 可判定性: 机器可判定（字节复现）；显示条件待 S2（美版删改差异属 H-06，**不得入结论**）
- 【2026-10-14 更正（AR54/复核 #54）】**引文归属勘误**："是否显示未证"引文出自**本条 R-ui-5**
  （曾被 base-attribution.md 附带清单引作 completeness E-R09/R10——错引；completeness E-R09/R10
  只有字节证据行）。附带升级注：开场文本**显示**已获像素证据（f240，sha 36dde3a1，见
  base-attribution.md E-B03）——"是否实际显示"可就开场页部分升级，结局页显示与显示条件仍未证（H-06）。
- 关联: B-SYS-UI、H-06

> 【2026-10-12 S2 回填】以下 R-ui-6..8 由 t12 场景证据提炼（B 级，绑 scenario/trace）。

### R-ui-6: 已证流程序列段：菜单输入序列 → 商店（f1080）→ 两连 B → 近黑过渡（f1200）→ 关卡开始（f1260，画面显示「地区-1」标签）→ 实机战斗（f1500+）
- 证据: `scenario:analysis/traces/gameplay-stage1`（trace f1080 shop-arrival → f1200 exit-transition → f1260 stage-start → f1500 mid-combat；frames.jsonl 2400 行）+ `scenario:oracle/scenarios/gameplay-stage1.txt`（输入序列注记）+ `scenario:analysis/traces/shop`（frames.jsonl 1450 行）；三场景并行复现一致（round-013 对账）
- 可判定性: 机器可判定（trace 帧序列 + 截图 diff）；序列中各输入的**独立作用**未逐一隔离（H-13）
- 关联: B-SYS-UI、B-SYS-SHOP、R-shop-7

### R-ui-7: 开机序列：黑场（f180）→ 标题（f360）→ ~~菜单态（f540–f900）~~；开机路径未出现开场文本（负证据）
- 证据: `scenario:analysis/traces/title-menu`（trace f180 boot-black〔sha `788d6984…`〕→ f360 title → ~~f540/720/900 menu~~ → f1079 menu-end；frames.jsonl 1080 行）
- 【2026-10-14 更正（AR54/复核 #54）】**第二处冲突更正**："菜单态（f540–f900）"失准——f540 像素
  （`shots/gameplay-stage1_f00540_probe.png` 族，sha 4e8021cd）实为**无菜单标题态**（字标上色动画），
  菜单 **f630 后**才在（f607–609 菜单行缓冲写入、f660 起可见 `◆开始/设置`；证据行"f540/720/900 menu"
  中 f540 的 menu 标签同错，f720/900 的 menu ✓）。正确窗口：标题态 f360–~f600，菜单态 f660–900。
  复现：probe.jsonl $0130–$014F 写点聚集 f607–609（0xac→$0133 @f609）。
  （本条"开机路径未出现开场文本（负证据）"与像素的**第一处冲突**另行处理：f240 即开场文本页、
  `menunoinput` f960 自动进第 2 开场页——负证据双重推翻，登记于 base-attribution.md 附带清单，待 spec 勘误裁定）
- 可判定性: 机器可判定（trace 帧序列）；开场文本在其它路径（DEMO？）是否出现**未证**（H-06）
- 关联: B-SYS-UI、R-ui-3、R-ui-5、H-06

### R-ui-8: 实机 HUD 观测登记（仅登记，语义留台账）：左上字母+数字行与红圈数字随输入/进程变化
- 证据: `scenario:analysis/traces/gameplay-stage1` + `shot:shots/gameplay-stage1_f01260_stage-start.png`（「A2/F3/1」）、`f01500_mid-combat.png`（「A2/F2/1」+ 红圈①）、`f02349_end.png`（「A0/W4/1」，运行中止于战斗态）
- 可判定性: 机器可判定（截图 diff）；**各行语义、字母切换（F↔W）归因、红圈数字含义未证 → H-12（留命名台账，禁入规则结论）**
- 关联: B-SYS-UI、B-SYS-SCORE、H-12

## 3. 汉化特有（单列，AGENTS.md R4：不与原版行为混写）

- 标题字标为中文「加纳战机」（`shot:shots/auto_f00540.png` 彩色态 / `auto_f00360.png` 单色态）。
- 标题菜单像素文本为中文「开始」「设置」（`shot:shots/auto_f00720.png`）；对应 ROM 原串为 `OPTION DEMO`（映射未证）。
- 汉化标记 `rom:7106–7118` `\t@2011  JIUBAN`（E-R12）。
- 以上属汉化版特有文本/渲染；与原版（含美版删改，H-06）的差异单列，**禁止混入原版行为结论**。

## 4. 待 S2 取证空位（留空，禁止发明；逐项建场景后回填规则）

> 【2026-10-12 S2 回填后状态】已证部分入规则（R-ui-6..8），未闭环部分继续留空 + 新 H 编号。

- [x] **部分闭环**｜菜单跳转逻辑：开机→标题→菜单→商店→关卡开始→实机的序列段已证（R-ui-6、R-ui-7）；
      **设置面板内容与返回路径未证**
- [ ] **未闭环**｜「开始」「设置」与 `OPTION`/`DEMO` 串的映射及各自功能（输入序列已留证，串映射未隔离）
- [ ] **未闭环**｜DEMO（演示）模式行为与触发条件
- [ ] **未闭环**｜`GAME OVER` → `CONTINUE` 流程（gameplay-stage1 f2349 为运行中止于战斗态，**未观测到 GAME OVER**）
- [x] **部分闭环**｜开场文本 / 结局文本显示条件：开机路径**未出现**开场文本（R-ui-7 负证据）；其余路径与结局未证（H-06）
- [ ] **未闭环**｜HI SCORE 的计算与持久化行为

## 5. 门禁

`python3 gates/cli.py check` → **PASS**（实跑 2026-10-12）：
`SUMMARY: PASS=4 FAIL=0 SKIP=7 TOLERATED=0`；`CHECK: PASS（含 0 个静默跳过：所有 SKIP 均已显式打印）`。
证据指针最小复现：completeness.md §10 命令（`grep -abo`/截图 sha256）。

## spec 产出契约

- 规则清单: R-ui-1..8（共 8 条，逐条带证据指针）
- 常量对照: 本模块无常量（文本库登记见 §1；相关常量在 [constants.md](constants.md)）
- 可判定性标注: 机器可判定 7 条 / 仅人工核对 1 条（R-ui-4 显示时机部分）
- 门禁: `python3 gates/cli.py check` → PASS（见 §5）；`gates/cli.py all` 见任务 t17 输出
- 遗留假设: H-06、H-12、H-13（C 级）；菜单串映射/DEMO/GAME OVER/HI SCORE 未证项见 §4
