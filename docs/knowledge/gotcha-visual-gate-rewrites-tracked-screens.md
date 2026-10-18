---
type: gotcha
title: gates check/status 会重写 tracked 的 oracle/screens（visual 门按当前 actual 重算 diff/report）——采集窗口跑它会把污染 actual 固化进 L4 基线
date: 2026-10-18
status: confirmed
---

## 现象

在 **P0-0 采集/渲染窗口**（`analysis/traces` 正被重采、`oracle/screens/actual` 正被 `render-frame` 覆盖）里跑
`python gates/cli.py check`（或 `status`）后，`git status` 出现 **tracked** 的 `oracle/screens/` 改动：

- `oracle/screens/actual/*.png`、`*.idx.json`、`.render.log`（本轮实测 `l4_clear-inject` 的 palette/tiles 全变）；
- `oracle/screens/diff/*.diff.png`（21 个 tracked 差异图）；
- `oracle/screens/visual-report.md`、`oracle/screens/whitelist-application.md`（generated 报告）。

本轮事件（2026-10-18）：P0-0 采集期间 `stg03-timeline` 是**截断 trace**（见 gotcha-mesen2-headless 的
`TestRunnerTimeout` 100s 条目），用它重渲的 `l4_clear-inject` actual 被 visual 门计入报告——差异像素
**58,101 → 58,392**；Lead 立即 `git checkout -- oracle/screens/` 回滚。

## 为什么

**两条写入路径叠加**（`check`/`status` 本身不渲染 actual，但它会把污染 actual 的后果固化进报告）：

1. **`actual/*` 的写入方 = `render-frame`**（P0-0 采集/渲染步）：`crates/game-desktop/src/bin/render_frame.rs`
   `--out-dir` 缺省即 `<repo_root>/oracle/screens/actual`（写 PNG + `.idx.json`，日志 append `.render.log`）。
   采集窗口用**当前 `analysis/traces`** 重渲染 ⇒ 若 trace 被截断/部分，actual 就是错帧——**这是污染源**。
2. **`diff/*`、`visual-report.md`、`whitelist-application.md` 的写入方 = visual 门**：
   `gates/cli.py gate_visual_diff` → `_gate_runner_dispatch("diff_visual.py", oracle/screens)` →
   `tools/diff_visual.py` 以**磁盘上当前 actual** 为输入重算，写 tracked `REPORT`
   （`visual-report.md`）、`write_diff_png`（`diff/*.diff.png`）、`whitelist-application.md`。
   ⇒ 即使 actual 只是"临时态"，跑一次 visual 门就把这个临时态**固化成 tracked 报告**。
3. **`status` 不是只读**：`gates/cli.py` 的 `cmd_status` 与 `cmd_check` **都调用 `run_all_gates()`**
   （status 的"只读预检"仅是文案）⇒ `status`/`check`/`visual` **三者都会触发 visual 门写 tracked 产物**。
   正常干净态下这是确定性的（重写字节全同、git 无感），所以平时察觉不到；**一旦 actual 处于采集中间态，
   就表现为"跑 status/check 把 oracle/screens 弄脏"**。

**危险**：部分/截断 trace 的 actual 一旦被固化/提交，会污染 L4 实际比对基线、差异账与后续测试基线
（P0-0 冻结前的中间态混入 = 基线不可信）；`git checkout` 回滚的是工作区文件，**不能回滚"已被渲染污染的事实"**。

## 何时使用

- **P0-0 采集/渲染窗口内**（`analysis/traces` 正在重采、`render-frame` 正在写 actual）：**不要跑**
  `gates check`/`gates status`/`gates visual`——需要门禁信息时用真正只读的：`gates/cli.py rom` /
  `seed` / `naming`（单门禁）或 `gates/cli.py baseline show`；`git status --short oracle/screens` 只用来看，
  不看它就不会变。
- 任何会改动 `analysis/traces` 或 `oracle/screens/actual` 的工作前后：先记 `git status --short oracle/screens`。
- **提交前**：`git status --short oracle/screens` 必须干净（或确属本卡显式基线变更并单独 commit）。
- **P0-0 新基线冻结时**：必须在**干净 trace 集**上一次性重生成 `actual`/`diff`/`visual-report`
  并作为**显式 commit**（message 标 `P0-0 baseline freeze` + Evidence/Confidence），不得让部分/截断 trace
  混入；`--strict` 输出行同批回填 manifest。

## 证据指针

- **写入方代码事实**：
  - `gates/cli.py`：`cmd_status`/`cmd_check` → `run_all_gates()`；`gate_visual_diff` →
    `_gate_runner_dispatch("diff_visual.py", ROOT / "oracle" / "screens", "L4 视觉")`。
  - `tools/diff_visual.py`：读 `oracle/screens/actual/*.png`（`ACTUAL.glob("*.png")`）；
    写 `oracle/screens/visual-report.md`（`REPORT.write_text`）、`oracle/screens/diff/*.diff.png`
    （`write_diff_png`）、`oracle/screens/whitelist-application.md`；**不写 actual**。
  - `crates/game-desktop/src/bin/render_frame.rs`：`--out-dir` 缺省 `repo_root/oracle/screens/actual`
    （写 `png`+`idx`；`--log-path` 缺省 `<out_dir>/.render.log`）——actual 的唯一常规写入方。
- **tracked 规模（可复算）**：`git ls-files oracle/screens/actual` = **513** 文件、`oracle/screens/diff` = **21**
  个 diff PNG；`oracle/screens/visual-report.md`、`oracle/screens/whitelist-application.md` 均在库
  ⇒ 上述写入都是**工作区可脏化**的 tracked 文件（不是 gitignored 产物）。
- **事件记录（2026-10-18，P0-0 采集窗口）**：截断 `stg03-timeline` trace 参与重渲后
  `l4_clear-inject` 差异 **58,101→58,392**（palette/tiles 全变）；Lead 已 `git checkout -- oracle/screens/`
  回滚。基线 `l4_clear-inject=58,101` 见 `oracle/screens/visual-report.md`。
- **可复现判据（干净窗口）**：在 `oracle/screens` 干净时连续跑两次 `python tools/diff_visual.py`，
  `git status --short oracle/screens` 应保持为空（确定性）；采集窗口内同样命令则会出现 diff/report
  改动（说明 actual 已处于中间态）——用这一"git status 空/非空"作为**唯一判据**，不要只看门禁 verdict。
- **本条的复核触发**：P0-0 采集暴露（Lead 2026-10-18 观测 + 回滚）；手册纪律行见
  `docs/findings/l4-baseline-freeze-plan.md` §2.2（采集窗口纪律）与 §8.2（冻结用干净 trace 集）。
