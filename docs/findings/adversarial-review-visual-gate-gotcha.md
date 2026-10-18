# 对抗复核：visual 门改写 tracked screens 的 gotcha + 采集期纪律（task-101）

> 复核方：`shell-verifier`（task-102，verifier）；被审：task-101 / `spec-writer`
> 被审文件：
> - 新知识条目 `docs/knowledge/gotcha-visual-gate-rewrites-tracked-screens.md`（**73 行**，SHA16=`3cd393f616e8502e`，untracked）
> - `docs/knowledge/README.md` 索引行（M，+1 行）与 `docs/findings/l4-baseline-freeze-plan.md` 采集期纪律（M，§2.2 L247-256 / §8.2 L1572-1577）
> 纪律：[AGENTS.md](../../AGENTS.md) §5（证伪立场）+ 本轮 **采集窗口约束**：**未跑** `gates check`/`status`/`visual`、未跑 Mesen；
> 只用只读代码/文件核 + 只读单门函数（`gate_docs_links()`/`gate_knowledge_format()`，不触发 `run_all_gates`）。
> 复核时点 `git status --short oracle/screens` **0 改动**（工作区未处于扰动态）。
> **VERDICT：needs_revision** —— 核心事实（actual 写入方=render-frame；diff/report/whitelist 写入方=visual 门；**status 也非只读**；事件/回滚/判据/手册纪律/格式）**全部核实通过**；
> 但 **README 索引行有一处错误表述**（"`gates check`/`status` 会用当前 analysis trace 写 tracked `oracle/screens/actual`（render-frame）"——把 actual 的写入者错记成门禁；门禁只读 actual、写 diff/report）。改该 1 行即可 pass。

## 1. 事实性（读代码核）

| 断言 | 独立核 | 结果 |
|---|---|---|
| `oracle/screens/actual/*` 写入方 = **render-frame**（`--out-dir` 缺省） | `crates/game-desktop/src/bin/render_frame.rs`：`--out-dir` 缺省 = `repo_root.join("oracle/screens/actual")`（L441-459 段：`None => repo_root.join("oracle/screens/actual")`；`--log-path` 缺省=`<out_dir>/.render.log`）；写 png+idx+log ✓。即 actual **唯一常规写入方=render-frame** | ✅ |
| `diff/*`、`visual-report.md`、`whitelist-application.md` 写入方 = **visual 门**（`tools/diff_visual.py`） | `gates/cli.py:820-821` `gate_visual_diff()` → `_gate_runner_dispatch("diff_visual.py", ROOT/"oracle"/"screens", "L4 视觉")`；`tools/diff_visual.py`：`ACTUAL=SCREENS/"actual"` 只读（L32/L168），写 `REPORT=SCREENS/"visual-report.md"`（L256）、`DIFF_DIR/*.diff.png`（L124-146）、`whitelist-application.md`（L258-271），**没有写 actual 的代码** ✓ | ✅ |
| **"status 也不是只读"**（`cmd_status`/`cmd_check` 都 `run_all_gates()`） | `gates/cli.py:974-975 cmd_status → run_all_gates()`；`:1001-1002 cmd_check → run_all_gates()`；`run_all_gates()`（:875）会执行 `gate_visual_diff`（:820）→ 调 `diff_visual.py` 写 tracked 文件 ⇒ **status/check/visual 三者都会触发 visual 门写 tracked 产物**；"status 只读预检"只是文案 ✓（**断言成立**） | ✅ |
| tracked 规模 | `git ls-files oracle/screens/actual` = **513** 文件；`git ls-files 'oracle/screens/diff/*.png'` = **21**；`visual-report.md`、`whitelist-application.md` 均 tracked ✓（与 gotcha 一致） | ✅ |

## 2. 事件记录与回滚准确性

- **基线 58,101 可核**：`oracle/screens/visual-report.md:13` = `l4_clear-inject.png: 差异像素 58101`（当前已回滚后的基线）✓。
- **58,392 事件**：`git grep 58392` 全仓 **0 命中**——该值是回滚前的瞬时工作区差异（Lead 观测），未入库/无法在回滚后独立复现；gotcha 已写明"本轮事件（2026-10-18）… Lead 立即回滚"，属**事件记录（B 级）**而非可复算数字，表述可接受（Info-1：建议补一句"该值来自回滚前现场观测，回滚后不可复算"以免读者试图 grep）。
- **回滚命令 `git checkout -- oracle/screens/`**：本事件中被改的是 **tracked** 的 actual（513 之一）/diff（21）/report——`git checkout -- <pathspec>` 可正确恢复 tracked 修改 ✓（Info-2：若 visual 门生成了**新**（untracked）diff png/actual 文件，checkout 不会删除；建议补 fallback `git clean -fd oracle/screens`）。
- **"干净窗口连跑 diff_visual 应 git clean"**：静态核 `tools/diff_visual.py` 无 `time/datetime/random/uuid` 等非确定输入（grep 0 命中）；报告行由 `sorted(EXPECTED.glob)` / 固定模板生成（L156-256），diff PNG 用固定 IHDR + `zlib.compress` + 固定像素映射（L124-146），无时间戳 ⇒ 同一 actual/expected 下两次运行字节相同、git 应保持 clean **（B：静态推理；采集窗口禁跑 visual，未动态复跑）** ✓。

## 3. 手册纪律行 / 验收 checkbox 一致性

- **§2.2 L247-256**（任务卡写 L247-256，实测一致）：明确"采集/渲染窗口内禁用 `gates check`/`status`/`visual`——三者都跑 visual 门，用磁盘当前 actual 重算并重写 tracked diff/report/whitelist；actual 由 render-frame（P0-0 渲染步）用当前 analysis/traces 写入；截断/部分 trace 的错帧会被 visual 门固化进 tracked 报告（本轮 58,101→58,392，Lead 已 `git checkout -- oracle/screens/` 回滚）；只读门禁用单门 rom/seed/naming 或 baseline show；若必须跑 check：先确认 `git status --short oracle/screens` 干净、跑后脏则 checkout 回滚" ✓ **可执行、与 gotcha 一致**。
- **§8.2 L1572-1577**：新增验收 checkbox "`oracle/screens` 干净性：采集窗口未跑 gates check/status/visual（或跑后已回滚）；冻结时 actual/diff/visual-report 由「干净 trace 集」一次性重生成并作为显式 commit（`P0-0 baseline freeze` + Evidence/Confidence），提交前 `git status --short oracle/screens` 无残留；禁止部分/截断 trace actual 混入基线" ✓ 可执行、与知识条目/§2.2 一致。
- 手册中无"visual 门重渲染 actual"的错误表述（§2.2 明确 actual 由 render-frame 写）✓；其余正文（§4.1/§4.2 等）由 task-94/100 更新，非本卡范围。

## 4. 格式 / README / 残留错误表述

- **knowledge-format**：只读单门 `gate_knowledge_format()` → **PASS（25 条知识条目）** ✓（新 gotcha frontmatter `type: gotcha`/title/date/status 合规，`gotcha-` 前缀 ✓）。
- **docs-links**：只读单门 `gate_docs_links()` → **PASS（354 md / 445 links 全部有效）** ✓（任务卡写 444 links；实际 445 = README 新增索引行带来的链接，**非失败**，Info-3）。
- **README 索引行**：已追加在 `docs/knowledge/README.md` 末尾（README 行：`| gotcha-visual-gate-rewrites-tracked-screens.md | gotcha | … |`）；README 内 22 条链接**全部可解析**、0 broken ✓。
- **残留错误表述（F-1，必改）**：README 该行文案写 "**`gates check`/`status` 会用当前 analysis trace 写 tracked `oracle/screens/actual`（render-frame）并固化 diff/report**"——
  这与事实不符：门禁 **不写 actual**（actual 由 render-frame 在采集/渲染步写；visual 门只**读** actual 并写 diff/report/whitelist）。该句正是任务卡第 5 项要防的"visual 门重渲染 actual"错误表述（虽为索引摘要，但会误导读者）。
  建议改为："`gates check`/`status` 都会跑 visual 门：**读**磁盘当前 `oracle/screens/actual` 重算并写 tracked `diff/*`、`visual-report.md`、`whitelist-application.md`；**`actual` 由 `render-frame` 采集/渲染步写入**——采集窗口跑门禁会把中间态固化进 tracked 报告；采集窗口禁用，跑后 `git checkout -- oracle/screens/`；冻结需干净 trace 集一次性重生成"。
  （gotcha 正文/手册 §2.2/§8.2 均无此错误——仅 README 摘要行 1 处。）

## 5. findings 四分类

- **实现/工具（B）**：无（知识/纪律 docs-only；代码事实核对全部成立）。
- **证据/文档（E）**：
  - **F-1（必改，Low）**：README 索引行错误表述（见 §4）——改为"门禁读 actual 写 diff/report；actual 由 render-frame 写"。
- **规格缺口（S）**：无（手动纪律行/验收项已可执行）。
- **平台差异（P）**：无。

### Info（不阻塞提交）

- **Info-1**：事件值 58,392 为回滚前 Lead 现场观测，回滚后不可复算（`git grep 58392` = 0）；gotcha 可注明"不可复算、基线 58,101 见 visual-report.md"。
- **Info-2**：`git checkout -- oracle/screens/` 只恢复 tracked（本事件足够，因为 actual/diff/report 都 tracked）；若 visual 生成**新 untracked** 文件，建议补 `git clean -fd oracle/screens`。
- **Info-3**：docs-links 实际 354 md/**445** links vs 卡面 444——差 1 = README 新索引行自身链接；PASS 不受影响。
- **Info-4（跨卡）**：手册附录 D.3 :1658 仍写 "`--out-dir/--log-path` 尚不存在"（task-94/100 已实现），属其它卡 stale 文本，建议由对应卡顺手更正（非本卡阻塞）。

## 6. 命令与输出摘要（只读，遵守采集窗口约束）

| # | 命令/操作 | 摘要 |
|---|---|---|
| 1 | 读 `gates/cli.py:974-975/1001-1002/875/820-821` | `cmd_status`/`cmd_check` → `run_all_gates()`；`gate_visual_diff` → `diff_visual.py oracle/screens` ✓ |
| 2 | 读 `tools/diff_visual.py`（L32/124-146/156-271） | 读 ACTUAL；写 REPORT/diff/whitelist；**不写 actual**；无 time/random ✓ |
| 3 | 读 `render_frame.rs`（out-dir/log-path 段 L441-459） | `--out-dir` 缺省 `repo_root/oracle/screens/actual`；png/idx/log ✓ |
| 4 | `git ls-files` 计数 | actual=**513**、diff png=**21**；visual-report/whitelist tracked ✓ |
| 5 | `git status --porcelain oracle/screens` | **0 行**（复核时点工作区干净） |
| 6 | 读 `oracle/screens/visual-report.md:13` | `l4_clear-inject.png: 差异像素 58101` ✓；`git grep 58392`=0（事件值不可复算） |
| 7 | 只读单门 `gate_docs_links()` / `gate_knowledge_format()`（**非** status/check） | docs-links **PASS（354 md/445 links）**；knowledge-format **PASS（25 条）** ✓ |
| 8 | README 链接检查（python 解析） | 22 links、0 broken；新 gotcha 索引行存在 ✓ |

## 7. 结论

**needs_revision（仅 F-1 一处 README 文案必改；3+1 条 Info 可选）**：
- 事实性全部成立：actual 写入方=render-frame；diff/report/whitelist 写入方=visual 门；**`status` 也非只读**（代码级确认 `cmd_status`/`cmd_check` 都 `run_all_gates()`）；tracked 规模 513/21/2 份报告一致。
- 手册 §2.2 纪律行与 §8.2 验收 checkbox 可执行、与 gotcha 一致；回滚命令对本事件正确；"干净窗口 diff_visual git clean"为确定性静态推理成立（未动态跑，遵守采集窗口禁令）。
- 唯一阻塞 = README 索引行把 actual 写入者写成门禁（与 gotcha 正文/手册自相矛盾，也是任务卡第 5 项要消灭的错误表述）。改该行后即可 **pass**；无需跑门禁/Mesen（只读单门已 PASS）。

> 本复核只写本文件；未改被审文件/资产，未 `git add`/`commit`；未跑 `gates check`/`status`/`visual` 与 Mesen（采集窗口纪律）。复核方≠实施方（AGENTS.md §5）。

---

# 点验（2026-10-18，task-104）—— README 索引行 F-1 修正

> 被审：`docs/knowledge/README.md` **L70**（Lead 修正后；M，+1 行的内容已改）；只核这 1 行。
> **VERDICT：pass** —— F-1 已修且与 gotcha 正文/手册 §2.2 一致；无"门禁写 actual"残留；README 链接可解析（仅单门 `gate_docs_links()` 核，未跑 status/check/visual/Mesen）。

## 点验 §A：README L70 与正文/手册一致性

- **README L70 现文**（逐字核）：
  "`gates check`/`status` 都会跑 visual 门：**读**磁盘当前 `oracle/screens/actual` 重算并写 tracked `diff/*`、`visual-report.md`、`whitelist-application.md`；**`actual` 由 `render-frame` 采集/渲染步写入**——采集窗口跑门禁会把中间态固化进 tracked 报告；采集窗口禁用，跑后 `git checkout -- oracle/screens/`；冻结需干净 trace 集一次性重生成" ✓
- **与 gotcha 正文一致**：gotcha L23 "`check`/`status` 本身不渲染 actual"；L25-27 "actual 写入方=render-frame（`--out-dir` 缺省）"；L28-32 "diff/report/whitelist 写入方=visual 门、以磁盘当前 actual 为输入"；L33-36 "status 不是只读"——**四要素逐项对上** ✓。
- **与手册 §2.2 一致**：手册 L247-255 "三者都跑 visual 门，用磁盘当前 actual 重算并重写 tracked diff/report/whitelist；**actual/* 由 render-frame 写入**；禁用/回滚/冻结纪律" ✓。
- **残留检查**：全文件/知识库/手册中无"门禁写 actual"事实断言；README L70 与手册 L249-250 均明写 actual 由 render-frame 写；gotcha 正文未改、无矛盾 ✓。

## 点验 §B：链接 / 范围 / 约束

- **链接**：**仅**调用只读单门 `gate_docs_links()` → **PASS（355 md/446 links）**（未跑 `gates status`/`check`/`visual`）；`docs/knowledge/README.md` 内 22 links、直接解析 **0 broken**（新 gotcha 索引行链接可解析）✓。
- **范围**：本修正仅 `docs/knowledge/README.md` L70（工作区 M）；未改 gotcha 正文/手册/其它文件 ✓。
- **约束**：未跑 Mesen/门禁总闸（采集窗口纪律）✓。

## 点验结论

**pass**：F-1 唯一阻塞项已按 task-102 建议修正，README L70 与 gotcha 正文/手册 §2.2 口径一致（actual=render-frame；门禁读 actual、写 diff/report/whitelist；status 非只读；采集窗口禁用/回滚/冻结纪律齐），无"门禁写 actual"残留；链接 PASS（单门）且 0 broken。**task-101 产出（gotcha + 手册纪律 + README 索引行）可提交**；knowledge_entries 24→25 基线由 Lead 随提交上调。

> 本点验只追加本节到本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
