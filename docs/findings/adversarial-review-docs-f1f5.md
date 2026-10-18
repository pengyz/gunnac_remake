# 对抗复核：task-27 docs 收口（F1/F2/F4/F5）——adversarial-review-docs-f1f5

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不得复用被审方结论**）
> 被审对象（**工作区未提交 2 docs**）：`docs/findings/plan-shellwin-v1.md`（+16/−4 行区域：F2 归属注 L193-197、F1 时点标注 §9.2 L204 / §9.3 L235-237、F5 自足指针 §9.6 L289-295）、`docs/findings/plan-diff-visual-strict.md`（+2 行：F4 docs 侧半句 L39-40）。
> 复核时 HEAD `167d8a9`（与文档内"task-27 收口时点 HEAD 167d8a9"一致）。本文只写本文件；未改被审文档、未 git add/commit。

---

## 0. VERDICT：**pass（1 条非阻塞文档精度 finding F1；其余事实/合规/一致性全部通过）**

- **F1 事实**：`cargo test -p shell-win` 实跑 = lib **5** + smoke **19** = **24 passed / 0 failed**（与 §9.6 L287、F1 注释"task-18 后 24=lib5+smoke19"一致；"task-1 时点 20"与 shell-verifier 报告 C10/旧 §9.2 一致）。
- **F5 事实**：当前 HEAD 即 `167d8a9`；`cargo test --workspace` exit 0 全绿（56 个 test-result 行全 ok）；`gates/cli.py check` **PASS=11 FAIL=0 SKIP=1[addr-bridge] TOLERATED=0**——与 §9.6 L289-290/292-295 实测值一致；其"task-25 复核在 HEAD 0cf42f0 独立复跑同值（见 adversarial-review-shell-fixes.md §4）"经核对：该报告已入库、§4 确为 24/全绿/CHECK PASS=11，交叉引用成立。
- **F2 事实**：三处留痕真实且指向正确——①`docs/knowledge/gotcha-git-add-commit-atomicity.md` L47-55（L48 明记 ddb1558 误含 task-9 platform-shell §3 模块表/tick）；②`docs/findings/plan-playable-loop.md` §10 L214-226（L221 明记 ddb1558 归属偏差 + L224-226 纪律）；③`git show -s --format=%B b94ede8` 含"platform-shell §3 主体随 ddb1558 入库；内容完整、未 rewrite 历史"。新注链接目标均存在、docs-links PASS。
- **合规**：`git status` 仅两个被审 docs 为 M（其余未跟踪文件属并行任务）；`git status --porcelain -- tools` 为空（F4 tools 侧确未动，另卡）✓；历史数字**原地保留**（diff 为"原行 + 追加注"，无删除；§9.6 "见交付汇报"改为实测值属补强）✓；docs-links PASS（319 md / 359 links，含本复核文件）。
- **F4 半句精度**：被要求核的"缺 actual 时 `STRICT: FAIL diff=0`"**准确**（临时样例实测 exit 1、`STRICT: FAIL diff=0`；尺寸不符同为 `FAIL diff=0`）。但半句把**"解码错误"**也列入"外差异须看 FAIL/PASS 本身"——实测**不成立**（见 F1）。

---

## 1. F1 时点标注（§9.2 L204 / §9.3 L235-237）——通过

| 检查项 | 结果 |
|---|---|
| 当前实测单 crate | `cargo test -p shell-win` → lib **5** + smoke **19** = **24 passed**（HEAD `167d8a9`）|
| §9.6 L287 | lib **5** + smoke **19** = **24**（task-1 基线 20 + 本卡新增 4）✓ |
| §9.2 L204（本卡改后） | 保留历史"lib 3 + smoke 17 = 20" + 追加"（**task-1 时点 20；task-18 后 24 = lib 5 + smoke 19，见 §9.6**）"✓ |
| §9.3 L235-237（本卡改后） | "smoke.rs 17 项 / crate 级 20 = lib3+smoke17"保留 + 追加"以上为 task-1 时点数字；task-18 后 24 = lib5+smoke19，见 §9.6"✓ |
| 一致性 | 三处（§9.2/§9.3/§9.6）数字互指一致；task-1 时点 20 与 shell-verifier E1 时点一致；无"20 vs 24"未注时点的并列冲突（均带时点/交叉引）✓ |

## 2. F5 自足指针（§9.6 L289-295）——通过

- 回填值在**当前 HEAD `167d8a9`** 复核实测：`cargo test --workspace` **exit 0 全绿**（56 个 test target 结果行全 ok）；`python gates/cli.py check` → **CHECK: PASS；PASS=11 FAIL=0 SKIP=1 TOLERATED=0**；单 crate **24 passed**。与 L289/L290/L292-294 逐值一致 ✓。
- 交叉引 `adversarial-review-shell-fixes.md` §4：文件已入库（tracked），§4 表载"lib5+smoke19=24 / workspace exit 0 全绿 / CHECK PASS=11 FAIL=0 SKIP=1"，与 task-25 复核（HEAD `0cf42f0`）一致 ✓；链接有效（docs-links PASS）。
- 不再出现"见交付汇报"式非自足指针（L289/L290 已改为 HEAD+实测值）✓。

## 3. F2 归属注（plan-shellwin L193-197）——通过

- 事实：ddb1558 自身 message 确实只写"期间 HEAD 含并发 task-9 未提交改动；本 commit 只含 task-18 写范围内 6 个文件"，未点名误卷内容（`git show -s --format=%B ddb1558` 复核）✓。
- 三处留痕**逐处可核**（行号与加注一致）：
  1. `gotcha-git-add-commit-atomicity.md` **L47-55**（L48：ddb1558 误含 platform-porter task-9 platform-shell.md 主体 §3 模块表/tick 序列；L49 task-9 最终稿另存 target/）✓；
  2. `plan-playable-loop.md` §10 **L214-226**（L221：ddb1558 = task-9 platform-shell §3 主体…归属偏差；L224-226 并发纪律）✓；
  3. `b94ede8` message："platform-shell §3 主体随 ddb1558 入库；内容完整、未 rewrite 历史"✓。
- 新注表述"历史不 rewrite；此后 commit message 应直接点明捕获的他人内容"与 knowledge 纪律一致，无过度指控 ✓。

## 4. F4 docs 侧半句（plan-diff-visual-strict L39-40）——核心准确，**1 点不精确（F1 finding）**

**我构造的临时样例实测**（module 路径重定向到 `%TEMP%`，不写仓库；tools 未改）：

| 样例 | 实跑结果 | docs 半句描述 |
|---|---|---|
| expected 有、actual 全缺（3 对） | exit 1；`L4 视觉：0 通过 / 0 比对；分类记录 3`；**`STRICT: FAIL diff=0`** | ✓ 准确（"缺 actual 时 FAIL diff=0"）|
| actual 存在但尺寸不符（1×1 PNG） | exit 1；**`STRICT: FAIL diff=0`** | ✓ 准确（尺寸不符→FAIL，diff 不计） |
| actual 为损坏 PNG（解码错误） | **未捕获 `AssertionError` 异常**（进程 exit 1 走 traceback，**无 `STRICT: FAIL` 行**、无 `STRICT: PASS`） | ✗ **不准确**：docs L35（既有）与新 L39-40 把"解码错误"也列为"须看 FAIL/PASS 本身" |
| 全对相同 + `--strict` | exit 0；`STRICT: PASS diff=0` | ✓（task-21 已核） |

**F1（低，非阻塞，文档精度）**：
- 位置：`docs/findings/plan-diff-visual-strict.md` **L35**（既有行"…或尺寸不符/解码错误 ⇒ `STRICT: FAIL`"）与**新增 L39-40**（"缺失 actual/尺寸不符/**解码错误**等外差异…须看 FAIL/PASS 本身"）。
- 实测事实：解码错误（非 PNG/损坏 PNG）在 `compare()`→`decode_png()` 抛 `AssertionError`，**未进入 strict 汇总**，不会打印 `STRICT: FAIL`（进程以 traceback exit 1）。
- 修法（二选一，均 1 句）：(a) docs 侧把"解码错误"从枚举移除，或改为"**解码错误当前会抛异常（exit≠0、无 STRICT 行）**，待 tools 卡统一为 FAIL 行"；(b) 由 tools 卡（另卡）在 `main()` 捕获 `compare` 解码异常并置 `strict_fail=True`，届时 docs 表述可保留。
- 性质：该不精确**继承自既有 L35**（task-21 原文），非 task-27 新引入；新半句的核心（缺 actual/尺寸不符 → FAIL 且 diff 不计）经实测**准确**，故非阻塞。

## 5. 合规与无新错误抽查

- **与 HEAD 一致**：文档内"task-27 收口时点 HEAD `167d8a9`"= 复核时 HEAD，实测值可复现（见 §2）✓。
- **只改 2 docs**：`git status --porcelain` 仅两文件 M；`git status --porcelain -- tools` 空；未动 gates/data/rom/reference ✓。
- **原地标注**：§9.2/§9.3 原历史数字保留并追加时点注（非删除）；§9.6"见交付汇报"两行改为 HEAD+实测值（补强、原措辞属弱指针，替换不构成历史主张删除）✓。
- **无新错误**：§9.1/§9.2/§9.3/§9.6 改动后交叉一致；F2 注引用的行号 L47-55/L214-226 与当前文件一致；docs-links PASS（319 md / 359 links）✓。
- 其余未跟踪文件（`adversarial-review-playable-a.md`、`l4-baseline-freeze-plan.md`）属并行任务，不在本次 2 docs 写范围。

## 6. 命令与输出摘要

```text
HEAD = 167d8a9（= 文档所称 task-27 收口时点）
cargo test -p shell-win        → running 5 → 5 passed ; running 19 → 19 passed ; exit 0   (=24)
cargo test --workspace         → exit 0；56 个 test-result 行全部 ok
python gates/cli.py check      → CHECK: PASS ; SUMMARY PASS=11 FAIL=0 SKIP=1 TOLERATED=0
git status --porcelain         → 仅 M docs/findings/plan-shellwin-v1.md、M docs/findings/plan-diff-visual-strict.md（其余为并行任务未跟踪 docs）
git status --porcelain -- tools → 空（F4 tools 侧未动）
git show -s --format=%B b94ede8 → 含 "platform-shell §3 主体随 ddb1558 入库；内容完整、未 rewrite 历史"
F2 留痕行号核对：gotcha L47-55 ✓ / plan-playable-loop L214-226 ✓
strict 临时样例（module 路径重定向，不写仓库）：
  缺 actual → exit 1 ; STRICT: FAIL diff=0      ✓
  尺寸不符 → exit 1 ; STRICT: FAIL diff=0      ✓
  损坏 PNG → AssertionError 未捕获（exit 1 traceback，无 STRICT 行） ✗ docs 枚举含"解码错误"
docs-links → PASS（319 个 md / 359 条本地链接全部有效）
```

---

## 7. 复核边界

1. 只复核 task-27 的 2 docs 改动（HEAD `167d8a9` 工作区）；未重审 task-18/21 实现（此前 task-25 已 pass）。
2. F4 的"解码错误"行为以当前 `tools/diff_visual.py`（未改）为准；tools 侧修复属另卡，本复核不动 tools。
3. 未实跑 GUI / release（不在本 docs 卡范围）。
4. 建议（非阻塞）：F1 那句若在本次提交前顺手修正最好；若不改，请在后续 tools 卡把解码错误纳入 `STRICT: FAIL` 后再同步 docs（避免文档与行为长期不一致）。

---

【2026-10-18】decomp-verifier，复核 HEAD `167d8a9`；被审=工作区未提交 2 docs（task-27）。
