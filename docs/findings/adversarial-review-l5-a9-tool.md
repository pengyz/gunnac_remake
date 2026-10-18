# 对抗复核：L5 A9 本机对拍工具（task-72）—— 证伪立场独立复核

> 复核方：`shell-verifier`（task-74，verifier 角色）；被审：task-72 / `l4-analyst`
> 被审对象：`tools/l5_probe_check.py`（新增，193 行，SHA256 前 16=`73781c1a7c9fa153`，未提交）
> ＋ `docs/findings/l5-a9-probe-check.md`（新增，170 行，SHA256 前 16=`cad202c262be8d2b`，未提交）
> 环境：Windows + Python 3.12.1 + cargo；复核用**独立 target** `target/l5probe-review-74`（仓库内，非共享 target）。
> 纪律：[AGENTS.md](../../AGENTS.md) §5（证伪立场；自己不采信自述、逐条实跑）。
> **verdict：needs_revision** —— 工具核心行为（发射/对拍/计数/退出码四类边界/封装合规）**全部独立实跑通过**（见 §1）；
> 但**文档实跑记录有 3 处与真实输出不一致**（`(null)` vs `None`、发射 transcript 非工具输出、`python3` 不可用），
> 另 2 条 Low 级防护建议（共享 target 未拒、陈旧产物可蒙混）。均为**行级/防护级**修正，工具判定逻辑不改。

## 0. 复核方法

- 只读被审 2 文件 + 自跑脚本（不改脚本/文档）；边界场景逐条造（移走 traces、清 PATH、越界 target）。
- 用独立 target `target/l5probe-review-74` 跑全链路（首次新 target，排除既有产物/共享 target 干扰）；
  对拍计数另用**手动 diff_l5 + PowerShell 重算**，不复用脚本自身解析结果。
- 产物：`analysis/traces/`（12 件，gitignored）；日志 `target/review-l5a9/*.log`（可再生、不入库）。
- 未改被审文件/资产，未 `git add`/`commit`；只写本文件。

## 1. 独立实跑结果（全部我自己执行）

### 1.1 全链路（核心验收）

```
python tools/l5_probe_check.py --target-dir target/l5probe-review-74 --timeout 1800
→ FULL_EXIT=0
emit: CARGO_TARGET_DIR=target\l5probe-review-74 cargo test -p game-core --test engine_tick -- --ignored emit_probe5
artifacts: 12/12 present in analysis/traces
passA: natural_ok=7 (expect 7) ok-injected=4 unknown=33 mismatch=0 provenance-note=0 | diff_l5 rc=1 (1=expected)
passB: ok-injected=27 unknown=17 mismatch=0 | diff_l5 rc=1
L5-DIFF natural 7/7
A9: PASS (local probe5 natural face OK; real-machine Mesen2 face still needs env)
```
- 独立 target 实际写入 272 个文件（`target/l5probe-review-74/**`），未触碰共享 target 语义路径（CARGO_TARGET_DIR 已在日志显式打印）。
- **独立重算**（手动 `diff_l5` + 自己按 `^\[([a-z-]+)\s*\]` 计数，不复用脚本）：
  passA `ok=7 / ok-injected=4 / unknown=33 / mismatch=0`；passB `ok-injected=27 / unknown=17 / mismatch=0` ⇒ 与脚本/文档一致。
- **12 件产物字节**与文档 §2.1 表**逐项相等**：`stg01..08` 合计 `4,714,197`、`gameplay-stage1.probe5.jsonl` `1,090,744`、`respawn-window` `501,208`、`s5-chain` `6,931`、`clear-run.trace.jsonl` `2,440`。

### 1.2 四类边界 + 补充路径（逐条实跑）

| 场景 | 命令 | 实测 | exit |
|---|---|---|---|
| 解析器自检 | `--selftest` | `SELFTEST: ok (repo files present + status parser 4/4)` | **0** ✅ |
| 自检不依赖 traces | 移走 `analysis/traces` 后 `--selftest` | 仍 `SELFTEST: ok` | **0** ✅ |
| 无 traces 且不发射 | 移走 `analysis/traces` 后 `--skip-emit` | `SKIP: analysis/traces missing 12 artifacts: [...]` | **2** ✅ |
| 阈值不满足 | `--skip-emit --expect-natural 8` | `L5-DIFF natural 7/8` + `A9: FAIL` | **1** ✅ |
| target 越界 | `--skip-emit --target-dir %TEMP%\l5probe_outside` | `ERROR: --target-dir must be inside repo ...` | **3** ✅ |
| 无 cargo | `PATH=''` 下 `python tools/l5_probe_check.py` | `SKIP: cargo not in PATH -- cannot emit probe5 (...) ` | **2** ✅ |
| 正路径（复用产物） | `--skip-emit` | `natural_ok=7 ... unknown=33` + `A9: PASS` | **0** ✅ |

### 1.3 封装合规 / 计数口径 / 写范围

- **未改判定/门禁/基线**：`tools/diff_l5.py`、`gates/cli.py`、`data/status/baselines/baseline.json` 运行前后 SHA256 前 16 均不变
  （`e496735c…` / `b7a08bdf…` / `06174d29…`）；`git status --porcelain` 对这些路径 **0 改动**；gates 无新增项。
- **产物不入库**：`git check-ignore` 证明 `analysis/`（`.gitignore:11`）与 `target/`（`.gitignore:13`）均忽略；脚本固定写 `analysis/traces/`。
- **写范围**：本次 task-72 仅新增 2 文件（`git status` 可见两 untracked）；未触其它路径。
- **计数口径独立核实**：
  - passA `natural_ok=7` = 7 natural 锚（title 3 + stg01 4，全在 `title-menu/gameplay-stage1` 前缀域）；
  - `ok-injected=4` = f≤3840 的 4 条 injected 锚；
  - `unknown=33` = **f3900 超发射窗（0x0F00=3840）1 行 + s5-chain 32 行**（手动 diff 输出逐行确认：`[unknown] s5-stg01 f=3900 stg02-combat` + 32 条 s5-chain）；
  - passB `ok-injected=27`（s5-chain 真锚）+ `unknown=17` = **5 条 f=0 占位 + 12 条非 chain 行**（3 s5-title + 9 s5-stg01），与文档 §3 一致；
  - 两 pass `mismatch=0`；`diff_l5` 两 pass 均 **rc=1（预期，unknown 计入 bad）**，脚本未用该 rc 做验收（与文档一致）。
- **emit 真执行**：脚本默认路径 cargo 返回 0 且 12 产物齐；新 target 首跑成功（非复用旧产物）。

## 2. findings 四分类

### 2.1 实现错误（B）——无（工具判定逻辑与计数解析均正确）

### 2.2 证据/文档不一致（E，需修 = 阻塞项）

- **E1（Low，文档 §2.2 逐行样本非真实输出）**：文档 `[ok] s5-stg01 f=1200 Transition **(null)**`、
  `[ok-injected] ... f=3400 Transition **(null)**`——实际 `diff_l5` 打印的是 **`None`**（Python `print(None)`）；
  且实际 `ok-injected` 行尾含 `trigger=injected …… 不计 natural` 后缀，文档样本省略。属"实跑记录"不逐字，
  会使读者误认工具输出格式。建议：改为真实输出，或标"示意（字段摘自实际输出）"。
- **E2（Low，文档 §2.1 发射 transcript 非工具输出）**：文档展示
  `running 1 test ... test result: ok. 1 passed; ... finished in 0.21s`，但工具**成功路径不打印** emit 的 stdout/stderr
  （只在 SKIP 时打印 `  | ...` tail；我实跑成功输出仅 `emit: ...` + `artifacts: 12/12` 两行）。
  建议：标注该块为"手动 cargo 命令输出"，或让工具成功时也打印 tail（同时可兼作 §E5 的"确实跑了 1 个 test"证据）。
- **E3（Low，可复现性）**：文档 §1/§5 命令均写 `python3 tools/l5_probe_check.py`；本机（文档 §2 声明的 Windows 环境）
  **无可用 `python3`**（Microsoft Store 别名报 "Python was not found"），实际必须用 `python`（我全程用 `python` 跑通）。
  建议 §1/§5 改 `python` 或注明"Windows 上 `python3` 别名可能不可用，用 `python`"。

### 2.3 规格/防护缺口（S，建议项，不阻塞本次 A9 判定）

- **E4（Low→Medium，防共享 target 未真正拦截）**：`--target-dir` 只检查"解析后位于仓库外"⇒ ERROR 3；
  但**仓库内共享 target 被接受**：实测 `--skip-emit --target-dir target` 与 `--target-dir .` **均未报错**（继续执行对拍）。
  若使用者（非 --skip-emit）传 `--target-dir target`，会直接写共享 `target/`；传 `.` 会把 cargo target 写到仓库根——
  与文档 §1"禁止共享 target / 防污染假红"的设施纪律不符。建议：拒绝 `resolve() == REPO` 与 `resolve() == REPO/'target'`，
  并建议要求 target 位于 `REPO/target/` 之下且 ≠ `REPO/target`（本卡默认 `target/l5probe` 自然满足）。
- **E5（Low，陈旧产物可蒙混 emit 成功判定）**：`emit_probe5 ok = rc==0 且 12 文件存在`；不清理/不校验产物新鲜度，
  也不校验 emitter 测试真的跑了 1 个（`cargo test -- --ignored emit_probe5` 若 filter 无法匹配会 0 tests、rc=0，
  叠加既有 `analysis/traces` 旧产物即可 false PASS）。建议：emit 前清空/改名 12 产物（或记录 mtime/sha），
  并断言输出含 `1 passed`（或复用 E2 的 tail 打印）。
- **Info-1**：passB（injected 链）mismatch 不进入退出码（脚本只看 passA natural/mismatch）；文档已标 passB"信息面"，
  但若 passB 出现 mismatch，工具仍 `A9: PASS`——建议至少打印 `WARN: passB mismatch>0`（不改验收口径）。
- **Info-2**：`KINDS` 常量（:52）定义后未使用（dead code）；`--selftest` 的解析样例只覆盖 4 种状态，未覆盖
  `ok-provenance-note/ok-trigger-pending`。
- **Info-3**：`TRACES.mkdir()` 在 `--skip-emit`/无 cargo 缺失检查**之前**执行 ⇒ "无 traces" SKIP 场景会留下空
  `analysis/traces/` 目录（我造该场景时即观察到；无害但可在 SKIP 时避免/记录）。

### 2.4 平台差异（P）

- **P1（环境，非缺陷）** 真机 run 面（B5，Mesen2 场景 → StageProbe）仍不可本机验证；文档 §4.1 已如实声明
  "probe5 仅 natural/engine 面、合成数据非实机 trace"。WSL2/Mesen2 环境就绪后此局限会变，届时需另卡补真机面。
- **P2** 本机 `python3` 别名不可用（见 E3）；`SendKeys`/GUI 无关本卡。

## 3. 被确认清单（可复算）

| # | 结论 | 证据 |
|---|---|---|
| C1 | 全链路本机可跑，`exit 0` + `L5-DIFF natural 7/7` | §1.1 实跑（新 target `target/l5probe-review-74`） |
| C2 | passA/ passB 计数与文档逐项一致（7/4/33/0；27/17/0） | 手动 diff 重算（PowerShell 独立解析） |
| C3 | 四类边界 + 无 cargo/SKIP 全部按文档语义返回（0/2/1/3） | §1.2 表 |
| C4 | `diff_l5` rc=1 属预期、未用作验收；验收=natural 命中数+mismatch=0 | 脚本 :184-189；实跑两 pass rc=1 |
| C5 | 不改 diff_l5 阈值/判定、不改 gates/基线 | 前后 SHA 不变 + git status 0 |
| C6 | 产物 12 件字节与文档表相等；产物/target 均 gitignored | §1.1/§1.3；`git check-ignore` |
| C7 | 工具执行真实 emitter（新 target 首跑即 12 件） | emit 日志 + target 272 files |
| C8 | unknown 33 / 17 口径解释正确（超窗+s5-chain / 占位+非链） | 手动 diff 逐行；文档 §3 |
| C9 | probe5 仅 natural/engine 面、真机面需 Mesen2 的局限如实 | 文档 §4.1；代码 emit 合成数据（engine 脚本/timeline，无 ROM 读取） |
| C10 | docs-links PASS | `gates status`：343 md/436 links PASS、STATUS OK |
| C11 | 写范围仅 2 新增文件 | `git status`：仅 `tools/l5_probe_check.py` + `docs/findings/l5-a9-probe-check.md` untracked |

## 4. 命令与输出摘要（自跑）

| # | 命令 | 摘要 |
|---|---|---|
| 1 | `python tools/l5_probe_check.py --target-dir target/l5probe-review-74 --timeout 1800` | exit 0；12/12 artifacts；passA 7/4/33/0 rc=1；passB 27/17/0 rc=1；`L5-DIFF natural 7/7` |
| 2 | 手动 `diff_l5 --probe gameplay-stage1 ... --scene-map ...`（rc=1）+ 手动 `diff_l5 --probe s5-chain ...` | 独立计数 `ok=7 ok-injected=4 unknown=33` / `ok-injected=27 unknown=17`；确认 `None`、`trigger=injected` 后缀 |
| 3 | `--selftest`（含移走 traces 后） | exit 0；`SELFTEST: ok ... parser 4/4` |
| 4 | 移走 traces + `--skip-emit` | exit 2；`SKIP: analysis/traces missing 12 artifacts` |
| 5 | `--skip-emit --expect-natural 8` | exit 1；`L5-DIFF natural 7/8` |
| 6 | `--skip-emit --target-dir %TEMP%\l5probe_outside` | exit 3；`ERROR: --target-dir must be inside repo` |
| 7 | `PATH=''` + `python tools/l5_probe_check.py`（无 cargo） | exit 2；`SKIP: cargo not in PATH` |
| 8 | `--skip-emit --target-dir target` / `--target-dir .` | **未 ERROR**（接受，暴露 E4） |
| 9 | 运行前后 SHA256（diff_l5/gates/baseline） | 不变（`e496735c…`/`b7a08bdf…`/`06174d29…`）；git status 对这些路径 0 |
| 10 | `git check-ignore -v analysis/traces/... target/l5probe/x` | `.gitignore:11 analysis/`、`:13 target/` |
| 11 | `python gates/cli.py status` | docs-links **PASS**（343 md/436 links）、STATUS OK |
| 12 | 12 产物字节 vs 文档 §2.1 表 | 全部相等；stg 合计 4,714,197 |

## 5. 修正建议（按优先级）

1. **E1/E2/E3（必改，docs 行级）**：把 §2.2 的 `(null)` 改真实 `None` 并补 `trigger=injected` 尾注（或标示意）；
   §2.1 发射 transcript 标"手动 cargo 输出"（或工具成功时打印 tail）；§1/§5 `python3`→`python`（或注 Windows 别名限制）。
2. **E4（建议，防护）**：`--target-dir` 增加"拒绝仓库根与共享 `target/`"校验（要求位于 `REPO/target/` 下且 ≠ `REPO/target`）。
3. **E5（建议，健壮性）**：emit 前清理/快照 12 产物，或断言 emitter 输出含 `1 passed`（可兼作 E2 的 tail 打印）。
4. **Info-1/2/3（可选）**：passB mismatch 打 WARN；删/用 `KINDS`；SKIP 前避免建空 traces 目录。
5. 复审判据：E1-E3 文档改毕即 **pass**（E4/E5 属加固建议，可在 A9 后续卡吸收；工具核心行为不动）。

## 6. 复核结论

**needs_revision（仅文档证据一致性 E1-E3；工具核心=通过）**：A9 封装**可跑、计数正确、四类边界退出码符合语义**，
未改 diff_l5 阈值/门禁/基线、产物 gitignored、独立 target 实跑无污染；`unknown 33/17` 口径与 `diff_l5 rc=1 预期` 均独立核实。
阻塞项仅为文档"实跑记录"与本机真实输出不一致（`(null)`→`None`、发射 transcript、`python3` 不可用）；
另有共享 target 未拒/陈旧产物蒙混两条 Low 防护建议。E1-E3 修毕可转 **pass**，无需重跑工具逻辑（我实跑日志已存 `target/review-l5a9/`）。

> 本复核只写本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。

---

# 复审（2026-10-18，task-78）—— task-76 的 E1-E5 收口定点复审

> 被复审版：`tools/l5_probe_check.py`（**208 行**，SHA256 前 16=`54920780c7b9a8d6`）＋
> `docs/findings/l5-a9-probe-check.md`（**205 行**，SHA256 前 16=`ae1e516e76b69842`），均未提交；
> 与 Lead 通告（`54920780…` / `ae1e516e…`）一致。范围：只核 E1-E5 + §8 落点表（上轮已跑通全链路/边界，不重跑全量）。
> **VERDICT（复审）：pass** —— E1-E5 **逐项独立实跑/核码通过**；仅剩 2 条 Info（提交时勾 §7 复选框；E5 判定可再收紧），不阻塞提交。

## 复审 §A：E1-E5 逐项核

| # | 要求 | 我的独立核 | 结果 |
|---|---|---|---|
| E1 | §2.2 逐字真实输出（`Transition None` 非 `(null)`；ok-injected 行尾 `trigger=injected 不计 natural 通过`；汇总口径） | 用 `subprocess` 原样捕获 `diff_l5` stdout 存 `target/review-l5a9/exact-passA.txt`：`[ok] s5-stg01 f=1200 Transition None ` **逐字节命中**；4 条 `[ok-injected]` 行尾**全部**含 `trigger=injected 不计 natural 通过`；汇总行含 `（unknown/ok-injected 不计）`；`[unknown]` 计数=**33** ⇒ 与文档 §2.2 样本/汇总**逐字一致** | ✅ |
| E2 | 工具成功路径打印 emit 真实行（含 `1 passed`）；§2.1 与工具 stdout 一致 | 实跑 `python tools/l5_probe_check.py --target-dir target/l5probe-review-74`（增量）→ stdout 含 **`emit: \| test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.28s`**，随后 `artifacts: 12/12`、passA/passB、`L5-DIFF natural 7/7`、exit **0**；与文档 §2.1 结构一致（耗时 0.37s 例，文档已注明随缓存变化） | ✅ |
| E3 | 命令 `python`（本机）+ Linux 注；无本机不可用 `python3` **命令**残留 | 文档所有 `l5_probe_check.py` 命令（:16/:32-35/:134/:169-171/:190）均为 **`python`**；`python3` 仅出现在解释性注（:7/:38/:39/:202，"Windows 用 python；Linux 用 python3"）⇒ 无命令残留；我全程 `python` 实跑通过 | ✅ |
| E4 | `--target-dir` 拒 `REPO`/`REPO/target`/仓库外（自跑 → ERROR 3） | 自跑：`--skip-emit --target-dir target` → **exit 3** `ERROR: ... must not be repo root or shared target (got target; use e.g. target/l5probe)`；`--target-dir .` → **exit 3**（同上 `got .`）；`--target-dir %TEMP%\l5probe_outside2` → **exit 3** `ERROR: ... must be under repo target/ (... shared/external target forbidden)`；合法 `--target-dir target/l5probe` → **exit 0**（A9 PASS） | ✅ |
| E5 | emit 成功判定含"输出含 `1 passed` + 12 产物齐"（构造 0 tests 验证不 false PASS） | ①核码：`emit_probe5` 返回 `ok = rc==0 and ran and files_ok`，其中 `ran = any("1 passed" in ln for ln in lines)`；②**对原模块做构造测试**（import 原文件 + monkeypatch `subprocess.run`）：输出 `0 passed`+既有 12 产物 ⇒ **ok=False**（旧版会 True）；`1 passed`+产物 ⇒ **ok=True**；`1 passed` 但 rc=101 ⇒ **ok=False** ✓ | ✅（见 §B Info-1） |
| §8 | 落点表与 E1-E5 对应 | §8 表 5 行（E1→§2.2、E2→§2.1+工具、E3→§0/§1/§5/§7、E4→§1+工具、E5→§1+工具）+ Info 行；与实现/文档落点一致 | ✅ |

## 复审 §B：其它核查（合规/范围/门禁）

- **未改判定/门禁/基线**：`tools/diff_l5.py`、`gates/cli.py`、`data/status/baselines/baseline.json` SHA256 前 16 仍为 `e496735c…` / `b7a08bdf…` / `06174d29…`（与 task-74 复核时同值）；`git status` 对三者 **0 改动**。
- **写范围**：相关改动仅 2 个 untracked 文件（`tools/l5_probe_check.py` + `docs/findings/l5-a9-probe-check.md`）；`git status` 无其它 tools/docs 路径被本卡改动。
- **docs-links**：`python gates/cli.py status` → **docs-links PASS（345 md/437 links）、STATUS: OK**（本次点改后重跑 ✓，对应文档 §7:193 应由 [ ] 改 [x]）。
- **产出/日志**：`analysis/traces/` 12 件齐（产物 gitignored：`.gitignore:11`）；复核日志 `target/review-l5a9/{e2-fullrun,e4-*,exact-passA,test_e5}*`（可再生、不入库）。
- **未回归**：`--selftest` 0 / 无 traces `--skip-emit` 2 / `--expect-natural 8` 1 / 无 cargo 2 的语义与 task-74 时一致（task-76 未改动这些路径逻辑；本轮抽查 E4 三类 + 合法路径已实跑）。

## 复审 §C：剩余问题（均 Info，不阻塞）

- **Info-1（E5 可再收紧）**：判据用子串 `"1 passed" in ln`；若 emitter 未来跑 N>1 个测试（如 "11 passed"）仍会命中子串，且**未清理旧 12 产物**（部分写入回归 + 旧文件可满足 files_ok）。
  当前 filter `emit_probe5` 实际恒为 1 test（37 filtered out），故不影响本卡；建议后续改为匹配 `test result: ok. 1 passed;` 并 emit 前清理/快照产物（可留 task-72 后续小卡）。
- **Info-2（文档提交时勾选）**：§7:193 `[ ] docs-links PASS（本次点改后重跑）` 现已可勾 `[x]`（我实跑 PASS）；§7:194 `[ ] 独立对抗复核 pass` 可随本复审通过勾选。
- **Info-3（时点）**：§2 记录 HEAD `3d8b557` 为当时实跑时点（当前 HEAD `32fc2bb`），属正常 dated 记录，无需改。
- **Info-4（示例耗时）**：§2.1 示例 `finished in 0.37s` vs 我实跑 `0.28s`——文档已注"随构建缓存/机器状态变化"，无问题。

## 复审 §D：命令与输出摘要（自跑）

| # | 命令 | 摘要 |
|---|---|---|
| 1 | `python tools/l5_probe_check.py --target-dir target/l5probe-review-74` | exit **0**；`emit: \| test result: ok. 1 passed; ... (0.28s)`；`artifacts: 12/12`；passA 7/4/33/0 rc=1；passB 27/17/0 rc=1；`L5-DIFF natural 7/7` |
| 2 | `python -c` subprocess 原样捕获 `diff_l5` passA 输出 | `Transition None`/4×`trigger=injected 不计 natural 通过`/汇总含 `unknown/ok-injected 不计`/unknown=33 —— 与 §2.2 逐字一致 |
| 3 | `--skip-emit --target-dir target` / `.` | exit **3**（repo root/shared target 拒绝） |
| 4 | `--skip-emit --target-dir %TEMP%\l5probe_outside2` | exit **3**（under repo target/ 拒绝） |
| 5 | `--skip-emit --target-dir target/l5probe` | exit **0**（合法路径仍可用，A9 PASS） |
| 6 | 原模块 `emit_probe5` 构造测试（monkeypatch subprocess） | `0 passed`+旧产物 ⇒ **False**；`1 passed`+产物 ⇒ **True**；`1 passed`+rc101 ⇒ **False** |
| 7 | 运行前后 SHA256（diff_l5/gates/baseline） | 不变；git status 0 |
| 8 | `python gates/cli.py status` | docs-links **PASS（345 md/437 links）**、STATUS OK |
| 9 | 被审文件 sha16/行数 | 工具 `54920780c7b9a8d6`/208 行；文档 `ae1e516e76b69842`/205 行（与通告一致） |

## 复审结论

**pass**：task-74 的 E1-E5 全部**闭环并独立复核通过**——文档实跑记录已逐字对齐真实输出（E1）、工具成功路径打印 `1 passed` 真实 emit 行（E2）、命令改 `python`+Linux 注（E3）、
target 防护三类非法路径 **exit 3** 且合法子 target 可用（E4）、emit 成功判定加 `1 passed`+12 产物新鲜度并以"0 tests+旧产物"构造测试验证不再 false PASS（E5）；
§8 落点表与 E1-E5 对应；未改 diff_l5/门禁/基线；写范围仅 2 文件；docs-links PASS。
剩余仅 Info（§7 复选框勾选、E5 子串/旧产物可再收紧），**不阻塞提交**：`tools/l5_probe_check.py` + `docs/findings/l5-a9-probe-check.md` 可入库。

> 本复审只追加本节到本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
