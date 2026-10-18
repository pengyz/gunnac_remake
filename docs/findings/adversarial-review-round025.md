# 对抗复核：round-025 轮次日志 + 过期文档台账（task-47）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：spec-writer（task-40）
> 被审文件：
> - `docs/rounds/round-025.md`（**98 行**，sha256 `4289780a…`，mtime 14:23:04）
> - `docs/findings/docs-staleness-register.md`（**92 行**，sha256 `6eaffe71…`，mtime 14:21:52）
> 复核方式：独立重算/重跑只读命令，不采信自述；不修改被审文件；本文件为唯一写入，未 `git add`/`git commit`。
> 边界：当前工作区有并发任务（game-shell/task-37 等）未提交改动，P0-0 手册等也未入库；本复核只判这两份 **docs-only** 产出。

## 0. VERDICT

**pass**（2 条非阻塞 findings；F1 建议提交前顺手修）

- round-025 的**关键事实全部可复算**：提交计数/时点、615/859/626/125、test 279→366 链、
  knowledge 19→23、基线值、各复核报告 verdict/测试数均一致；**"98% 判据无落地文本"结论成立**
  （master-plan §6 仍为 0 容差；除 round-024/本日志外全仓 `98% 级` 零命中；`git log -S` 无提交）。
- 过期台账 S1-S6（含 S1 的 7 段 delta/S1-S2 数值、S2 247、S3 m3-inventory、S4 54 vs 121、
  S5 4552、S6 AR2-A）逐项复算**与仓库现状一致**；"能否本机重导"判断成立（S1 可只读重导；
  S3/S4 无生成器；S5/S6 缺 analysis inputs/Mesen2；S7 可改 tools 后重生成）。
- **F1（低）**：S7 的 `tools/diff_visual.py` **行号引用已过期/错位**（详见 §4），建议改为内容锚或更新行号；
  不影响 S7 的"过期"结论。
- **F2（信息）**：round-025 §7 把 task-35 记为"进行中"，为本卡时点（HEAD `b92139a` + 工作区）历史快照；
  现在 task-35 已 pass，提交前可加"时点"限定或更新，以免读者误读为仍未复核。

## 1. 复核方法（独立重导，非采信自述）

- **提交/计数**：`git rev-parse`/`git rev-list --count`/`git show -s --format` 独立复算 round-025 的
  51/615/859/626/125 与代表 commit 时点。
- **账本链**：读 `data/status/baselines/baseline.json` + `baseline_history.jsonl`，核对 rust_test_count 七次
  上调与 knowledge 19→21→22→23；读 gate 输出核对 knowledge 23/rust-tests 现值/基线。
- **98% 判据**：全仓 md grep `98% 级`；`git grep '98% 级' -- '*.md'`（tracked）；
  `git log -S '98% 级行为一致' -- docs/master-plan.md`；读 round-024 原文与 master-plan §6 原文。
- **过期台账**：按台账 §3 复现命令复算——S1 用 `tools/workset_inventory.py` **只读重导到 %TEMP%** 并逐段 diff；
  S2/S3/S4/S5/S6/S7 逐项读原文件 + 计数/哈希/grep 核对。
- **能否本机重导**：检查 `analysis/traces`、`analysis/phase`、`tools/mesen2`、`tools/` 生成器存在性。
- **纪律**：`git status --short data gates`（应无改动）、round-024/master-plan 未被改（历史不删）、
  `python gates/cli.py check` 的 docs-links 结果（全量 check 有并发 WIP 干扰，见 §3 说明）。
- 只读复核，未跑采集/渲染（Windows 无 Mesen2/analysis 输入，符合 P0-0 §9 边界）。

## 2. round-025 事实核验（逐项）

| # | 声明 | 我的独立结果 |
|---|---|---|
| R1 | 覆盖会话段 `04559b1` 12:17→`b92139a` 14:14，**51 提交** | `git rev-list --count 04559b1^..b92139a` = **51**；`git show` 时点一致 ✅ |
| R2 | round-024 落档 `fc83cd3`（10-14 15:36）之后 **615 提交**无轮次日志 | `fc83cd3..b92139a` = **615**；`fc83cd3` 时点 2026-10-14 15:36:00 ✅ |
| R3 | 仓库累计 **859**；10-14 以来 **626**；当日 **125=74+51** | `rev-list --count b92139a`=**859**；`--since=2026-10-14`=**626**；`--since=2026-10-18`=**125**；125−51=74 ✅ |
| R4 | **98% 判据** round-024 有记录，但称"入档 master-plan"无落地文本 | round-024 L8-10 确写"98% 级行为一致+分类账完备…入档于 master-plan"；**master-plan §6 L73/L75 仍是 0 容差**，无 98%；`git grep '98% 级' -- '*.md'` tracked 仅 round-024；`git log -S '98% 级行为一致' -- docs/master-plan.md` **空**；文件系统全仓 md 仅 round-024/round-025 命中 ⇒ round-025 的"无落地文本/不作现行口径"**成立** ✅ |
| R5 | rust_test_count 链 279→280→326→338→339→355→357→366（+87） | `baseline_history.jsonl` 七条 transition 全同（`76ba3ad`/`ff5aa7c`/`16fb5b2`/`fe8ce68`/`c83a19a`/`cc4c692`/`9405a35`），算术 +87（279→366）/以 280 计 +86 ✅（注：是 **7 次**上调，任务卡若说"八次"应为口径笔误；文档本身未称八次） |
| R6 | knowledge 19→21→22→23 | `baseline_history` 三条 transition（`76ba3ad`/`062440a`/`24093be`）+ gate `knowledge-format PASS 23` + baseline knowledge_entries=23 ✅ |
| R7 | 基线现値 1/1/206/0/23/366/19 | `data/status/baselines/baseline.json` metrics 全同（schema 外字段）✅ |
| R8 | 各复核 pass/测试数：playable-B 33/366、playable-A 355、shell-fixes 24、平台/菜单/ docs 等 | 读报告：playable-B §1 "lib12+smoke21=33、workspace 365+1=366、CHECK PASS"；playable-A "rust-tests 355/355"；shell-fixes "24 = lib5+smoke19"；平台 `9bb0fd9` needs_revision→`96415c1` pass；菜单 `06a0dc4` pass；docs `ba911b4` pass ✅ |
| R9 | 代表 commit 清单（基建/平台/PC shell/可玩/L4） | 全部 56 个被引 hash `git cat-file -t` 均存在；抽查 `04559b1/7fd50b1/e7b8285/9271cf0/a11161d/ab6383c/3e1715f/76ba3ad/062440a/24093be/64b3a44/0cf42f0/...` 均有对应提交 ✅ |
| R10 | 覆盖关键事件且"待复核项不写成已完成" | 基线修复/平台 v1/PC shell v1-v2/可玩 A-B/B1/L4 反转/P0 方案覆盖；§7 把 task-35/36/37/38/39 均标"在飞/遗留"，**未把 task-35 写成 pass**（时点为 b92139a）；现 task-35 已 pass、task-38 已完成，属历史快照（F2 仅建议加时点）✅ |

**小结**：未发现 round-025 的编造/夸大；615/859/626/125、51、test/knowledge/基线、98% 无落地文本均可复算；
唯一可改进是历史时点的在飞状态（F2）。

## 3. 过期台账核验（S1-S7）

| # | 台账声明 | 我的独立结果 |
|---|---|---|
| **S1** | 镜像=2026-10-15 重导；HEAD core61/spec318，镜像 core54/spec325；delta 7 段全 spec→core | 按台账 §3-A 用 `workset_inventory.py` 只读重导到 `%TEMP%`：**HEAD `{spec-cited:318, core-cited:61, L2-diff:15}`；mirror `{spec-cited:325, core-cited:54, L2-diff:15}`；DELTA=7**，逐段正是 `bank00:8185-81A2/82EF-82FC/8DDF-8EC7/929E-9319/93C1-9409、bank14:CCD3-CD07、bank14:D679-D6C5`（与台账 §2 一致）；镜像 commit `61ed634` 2026-10-15 21:38 ✅ |
| **S2** | `docs/spec/rewrite-workset.md` §4 旧快照：unexplored 247 / ≥core 49 | L154 "2026-10-15 快照"；L159 "≥ core-cited 49"；L162 "unexplored 247" ✅（当前镜像/HEAD 数值见 S1） |
| **S3** | `m3-inventory.md`：shots/enemy/stage ❌未动；知识库10/L2 54/测试17/PASS=10 SKIP=1；含白名单候选旧前置 | L17 "知识库 10 条；L2 用例 54"；L18 "Rust 测试 17；PASS=10 SKIP=1"；L36-38 shots/enemy/stage "❌ 未动"；L55/L70 "白名单候选用户确认" ✅；我复算 `shot.rs 460 / enemy.rs 1161 / stage.rs 265` 行（台账 §3-B 同值）、L2 现 121 例 ✅ |
| **S4** | `coverage_map.json` generated 2026-10-12：l2_cases=54、rules_total=44；现 l2_cases.jsonl=121 | `coverage_map.json` generated=**2026-10-12**、len(l2_cases)=**54**、rules_total=**44**；`oracle/subroutines/l2_cases.jsonl` 有效行 **121** ✅ |
| **S5** | `l4-final-localization-ending1.md` 标题 7753 态残差 7380px/261格；现 ending-1 diff=4552 | 文件首行标题"7753态残差7380px/261格"；`oracle/screens/visual-report.md` L15 `l4_ending-1.png: 差异像素 4552` ✅ |
| **S6** | `residual-pixel-ledger.md` AR2-A（自机 −$30 锚点）族已撤销；"每像素可归因/全族勾销"不成立；其余 9 族仍有效 | L57-59 原文"该族 ✗ 撤销待 W47（自机本体槽）…全族勾销不成立（9/10 维持、1 族待重核）" ✅（台账的"已撤销/9 族维持"与原文一致；"待 W47"细节建议保留） |
| **S7** | 生成物（`oracle/screens/whitelist-application.md`、`visual-report.md`、`tools/diff_visual.py`）仍列 W-C1/W-C4 强候选/申请 | 三文件确实仍有 W-C1/W-C4 文案；但台账给的 **`tools/diff_visual.py` 行号 L189-190/L223-225/L227-236 与现状不符**（当前 W-C1/W-C4 在 L99/L201/L213/L220/L254/L264/L265；报告头在 L220-222，申请清单在 L254/L264/L265）——**F1**；结论"口径过期"成立 ✅ |

**"能否本机重导"判断**：
- S1 **部分**：只读重导到 `%TEMP%` 我复现成功；正式重导会写 `data/`+`analysis/`，需 data 授权 ✅。
- S2 **是**：同一 `workset_inventory.py` 可重导 JSON；文档回填需人工 ✅。
- S3 **否**：`tools/` 只有 `workset_inventory.py`，无 m3/coverage 生成器；人工盘点文档，数字可重采、整篇须重写/退役 ✅。
- S4 **否**：无 coverage_map 生成器（tools 中无）✅。
- S5/S6 **否**：`analysis/traces`、`analysis/phase` 不存在、`tools/mesen2` 不存在（`analysis/` 仅有 visual-scout/workset）⇒ 无法本机重做 P0-0 级定位 ✅。
- S7 **是**：改 `tools/diff_visual.py` 文案后重跑 `python tools/diff_visual.py` 可重生成（会写 oracle/screens）✅。
- **台账"已核对为现行"** 的 baseline knowledge 23/tests 366/naming 206/completeness 19 与我读 baseline.json 一致；visual-report 当前数值（ending-1 4552、22 对总差 280132）与现有报告一致 ✅。

## 4. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- **F1（低，建议提交前修）**：`docs-staleness-register.md` S7 的 **tools 行号引用过期/错位**
  （台账 L17 写 `tools/diff_visual.py` L189-190/L223-225/L227-236；现状 W-C1/W-C4 在 L99/L201/L213/
  L220/L254/L264/L265；报告段在 L220-222、申请清单 L254/L264/L265）。建议改为"文件 + 内容锚"
  （如 `工具 docstring`、`"平台差异候选 W-C1"` 段、`"白名单候选（未启用）"` 段），避免以后行号再漂移。
  不影响 S7 的"过期"结论。

### 建议（P3）
- **F2（信息/低）**：round-025 §7（L82）仍把 task-35 记为"进行中"，为本卡时点（HEAD b92139a）历史快照；
  现在 task-35 已 pass、task-38 已完成。提交前可在 §7 标题/行加"本卡时点"限定（已具备）或补一句
  "提交时最新状态见任务板"，避免读者误读；不属事实错误。
- **F3（可选）**：round-025 L72 的 test 链实际是 **7 次** transition（279→280→326→338→339→355→357→366，
  +87）；正文未声称"八次"，若引用处（任务卡）写八次建议校正为七次。
- **F4（可选）**：round-025 L76 的门禁状态绑定各复核报告时点（未重跑全量 `check`）；本复核实跑
  `gates/cli.py check` 当前 **FAIL=rust-tests**（`game-shell` 的 task-37 未提交 WIP 编译错，
  `stg01_combat` 引用未实现的 Host 方法），**非 task-40 改动**；**docs-links 本身 PASS 330 md/417 links**。
  建议 round-025 已在说明中保留"数字绑定报告时点"，提交时无需改；复核报告记录此环境事实。

## 5. 纪律核验

- **docs-only**：`git status --short data gates` 为空（无 data/gates 改动）；round-024、master-plan 无修改
  （历史未删）；被审两份文件为新增 docs。工作区其他 crates 修改为并发 task-37 等，不属 task-40。
- **docs-links PASS**：`gates check` 输出 `docs-links 330 md/417 链接全部有效` ✅；P0-0 手册/复核等
  未链接断链。
- **全量 check**：当前 `SUMMARY: PASS=10 FAIL=1 SKIP=1 TOLERATED=0`，唯一 FAIL=rust-tests 为并发
  task-37 `game-shell` WIP 编译错（非 docs 产出）；round-025 已声明"本卡期间未重跑全量 check、数字绑定
  报告时点"，诚实。task-47 的 docs-links 验收 PASS。
- **未把待复核项写成已完成**：round-025 §7 列 task-35/36/37/38/39 为在飞/遗留，未伪造 pass；
  §5.0 明确 round-024 98% 判据无落地文本，未沿用旧判据为现行。

## 6. 命令输出摘要

```text
# 提交计数
rev-list --count b92139a                         = 859
rev-list --count fc83cd3..b92139a                = 615
rev-list --count 04559b1^..b92139a               = 51
rev-list --count --since=2026-10-14 b92139a      = 626
rev-list --count --since=2026-10-18 b92139a      = 125   (125-51=74)
fc83cd3 timestamp                               = 2026-10-14 15:36:00 +0800

# 98% 判据
git grep '98% 级' -- '*.md' (tracked)            = 仅 docs/rounds/round-024.md:9
git log -S '98% 级行为一致' -- docs/master-plan.md = 空
master-plan §6                                  = "L4 白名单外像素一致"（0 容差），无 98%
全仓 md 文件扫描                                 = 仅 round-024/round-025 命中

# 账本
baseline_history: 279→280→326→338→339→355→357→366（7 次；+87）
knowledge: 19→21→22→23；gate knowledge-format PASS 23
baseline metrics: rom1/seed1/naming206/assets0/knowledge23/rust366/completeness19

# 过期台账复算
workset HEAD   = {spec-cited:318, core-cited:61, L2-diff:15}
workset mirror = {spec-cited:325, core-cited:54, L2-diff:15}
workset DELTA  = 7（bank00:8185-81A2/82EF-82FC/8DDF-8EC7/929E-9319/93C1-9409、
                  bank14:CCD3-CD07、bank14:D679-D6C5）
rewrite-workset §4 = core 49 / unexplored 247（L154 自标 2026-10-15 快照）
m3-inventory = knowledge10/L2 54/tests17/PASS10/SKIP1；shots/enemy/stage ❌未动
coverage_map = generated 2026-10-12 / l2_cases 54 / rules_total 44；l2_cases.jsonl 121
l4 ending-1 title 7753态/7380px/261格；visual-report 现行 diff 4552
residual-pixel-ledger AR2-A: "撤销待 W47; 9/10 维持"
S7 tools 现状 W-C1/W-C4: L99/201/213/220/254/264/265（台账给 L189-190 等 → F1）
analysis/traces=False, analysis/phase=False, tools/mesen2=False；tools 仅 workset_inventory.py

# 门禁
gates check: docs-links PASS 330 md/417 links；SUMMARY PASS=10 FAIL=1 SKIP=1
             FAIL=rust-tests（task-37 game-shell WIP 编译错，非 task-40）
```

## 7. 结论

**pass**：round-025 的计数/提交/账本/98% 判据核验**全部可复算**；过期台账 S1-S6 与仓库现状一致，
"能否本机重导"判断成立；纪律（docs-only/历史不删/docs-links PASS）成立。
唯一需顺手改的是 **F1：S7 的 tools 行号引用**（低）；F2-F4 为提交前可选措辞/环境说明。
建议 Lead 提交这两份 docs；F1 可同批修（不影响 pass 结论）。
