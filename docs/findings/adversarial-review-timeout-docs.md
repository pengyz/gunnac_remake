# 对抗复核：Mesen2 100s 超时文档同步（task-91）—— 快速点验

> 复核方：`shell-verifier`（task-92，verifier）；被审：task-91 / `spec-writer`（docs 同步）+ 工具超时修复（工作区）
> 被审文件：`docs/knowledge/gotcha-mesen2-headless-windows-io.md`（**117 行**，SHA16=`6e06f899dadb5a9d`，M）、
> `docs/findings/l4-baseline-freeze-plan.md`（**1881 行**，SHA16=`42931109209e79da`，M）、
> `tools/scenario_run.py`（**759 行**，SHA16=`1c946edfe863e007`，M；headless 支持已入库 `5a675b8`，本次仅加 Mesen 级 `--timeout`）。
> 纪律：[AGENTS.md](../../AGENTS.md) §5（证伪立场）+ 本卡约束：**只读/静态核验，未运行 `scenario_run.py`**（l4-analyst 正持串行锁采 P0-0）。
> **VERDICT：pass** —— 知识 gotcha 现象④/机理/证据指针、手册 §2.1/§2.2 超时注与三项判据、代码事实（`mesen_timeout_s=max(120,int(timeout_s)+120)` + `--timeout=` 参数）、Mesen.exe `TestRunnerTimeout` 字符串旁证**逐项核实**；docs-links PASS；仅 3 文件（2 docs + 工具修复）有改动。附 3 条 Info（不阻塞提交）。

## 1. 逐项核

### 1.1 知识库 gotcha（现象④ / 为什么 / 证据指针）

| 位置 | 文档断言 | 我的核验 | 结果 |
|---|---|---|---|
| 现象④（L31-35） | 长场景（stg08 34000f / tas-stg08b2 40000f）未显式 `--timeout=<秒>` 会在 **~100s** 被 Mesen 自身强停；症状 `trace_lines<frames`、`done` 缺失、`wall≈100-120s`；**`run.json.timed_out` 可能仍为 false**（Mesen 自停≠驱动超时；驱动在进程退出后给 20s 宽限再收工）；必须同核 `done!=null && trace_lines==frames` | 与 `tools/scenario_run.py:577-591` 行为一致：`timed_out` 仅在 `now-t0 > timeout_s`（驱动级）时置 true；`proc.poll()!=None` 后进入 `dead_since` 分支、`now-dead_since>20` 才 break（**20s 宽限**代码事实）⇒ Mesen 自停在驱动超时前发生时 `timed_out` 保持 false ✓ | ✅ |
| 为什么（L46-50） | `TestRunnerTimeout` 默认 **100s**（上游源码 `CommandLineHelper.cs:31`，**Mesen2 源码不在本仓库、未独立复核**）；长场景必须显式 `--timeout=<秒>`（Mesen 级）；驱动 headless 分支**已自动**追加 `--timeout=max(120, timeout_s+120)`；手工 `Mesen.exe --testRunner` 才需自带 | ①上游源码无从核（仓库无 Mesen2 源码），文档已**如实标注"未独立复核"**，未夸大为已证；②自动追加代码见 §2（实际 `max(120, int(timeout_s)+120)`）✓；③"手工才需自带"逻辑与 headless 分支条件一致 ✓ | ✅（证据分级如实） |
| 证据指针（L74-84） | 驱动集成代码锚：`MESEN` 平台选择、`HEADLESS` 判定、headless 命令 `[MESEN,"--testRunner",f"--timeout={mesen_timeout_s}",lua,ROM]`、`mesen_timeout_s=max(120,int(timeout_s)+120)`；headless 支持已入库 `5a675b8`；Mesen 级超时=2026-10-18 工作区修复（task-91，复核后入库），引用前验证 `git log`/`mesen_timeout_s` | 逐项命中：代码 `tools/scenario_run.py:59-68`（MESEN/HEADLESS）、`:548-554`（注释+var+命令）；`mesen_timeout_s = max(120, int(timeout_s) + 120)`（L550）；命令 L554 恰为 `["--testRunner", f"--timeout={mesen_timeout_s}", lua, ROM]`；`git log -1 -- tools/scenario_run.py` = `5a675b8 feat(tools): … headless …` ✓（Mesen 级超时确为工作区 M 未提交） | ✅ |
| 证据指针（L85-89） | 100s 依据=`CommandLineHelper.cs:31`（不在仓库、未独立复核）；本机 `Mesen.exe` 二进制内含 `TestRunnerTimeout` 字符串可作旁证；验收判据=`done!=null && trace_lines==frames`（不能只看 `timed_out`） | Mesen.exe 字节搜索：ASCII `TestRunnerTimeout` **命中 4 次**（UTF-16LE 0 次）——"二进制含该字符串"成立（仅"配置项存在"旁证，文档措辞一致）；判据与手册 §2.2 一致 ✓ | ✅ |
| 任务链（L112-117） | task-80/81/82/83/91 链 + 权威落档=P0-0 手册 §1 | 各卡/报告均在库或本会话可溯源；手册 §1 已更新（task-85/88 复核定稿）✓ | ✅ |

### 1.2 手册 P0-0（§2.1 / §2.2 / 判据）

| 位置 | 文档断言 | 核验 | 结果 |
|---|---|---|---|
| §2.1（L178-181） | 长场景超时注：I3/I4/I5/I5b/I6/I6b 预计 >100s；Mesen testRunner `TestRunnerTimeout` 默认 100s；`scenario_run.py` headless 自动传 `--timeout`（见 §2.2）；手工调用必须自带 `--timeout=<≥场景预算秒>`，采集后必须核 `done!=null && trace_lines==frames` | 与 gotcha/代码一致；I3/I4/I5/I5b/I6/I6b 确为 >100s 场景（§2.1 表帧数 24000/34000/40000 等，按历史 ~30-60fps 估算远超 100s）✓ | ✅ |
| §2.2 超时注（L247-254） | 区分**驱动级** `--timeout <秒>`（`run_scenario(timeout_s=…)`）与**Mesen 级** `--timeout=max(120, timeout_s+120)`（headless 分支自动、`--testRunner` 参数区内、无需手填）；手工调用必须自带；`TestRunnerTimeout` 默认 100s（`CommandLineHelper.cs:31`）；~100s 被强停（trace 截断/done 缺失/wall≈100-120s）；**t.timed_out 不一定 true**（Mesen 自停、驱动 20s 宽限收工）⇒ 必须校验 done/trace | 与代码 L548-554/577-591 逐项一致；`--timeout` CLI 定义 L721/723（`type=float, default=1800.0`）；20s 宽限代码事实 ✓ | ✅ |
| §2.2 判据（L265-267） | 逐场景判据：`timed_out=false`、**`done!=null`**、**`trace_lines==frames`**，**三项缺一不可**（100s 截断会 timed_out=false 但 done 缺失/trace<frames ⇒ 截断产物不得入基线） | 三项齐全且与代码字段对应（run.json 字段 `timed_out/done/trace_lines/frames`，scenario_run.py 实际写入）；"缺一不可"表述明确 ✓ | ✅ |

### 1.3 代码事实 / 二进制旁证

- `tools/scenario_run.py:550`：`mesen_timeout_s = max(120, int(timeout_s) + 120)` ✅（与文档一致，详见 Info-1 的 `int()` 精度差异）。
- `:554`：`[str(MESEN), "--testRunner", f"--timeout={mesen_timeout_s}", str(lua_path), str(ROM)]` ✅（Mesen 级参数在 `--testRunner` 参数区内、lua/ROM 之前）。
- `:721/:723`：`p_run/p_all.add_argument("--timeout", type=float, default=1800.0)` ✅（驱动级 `--timeout`）。
- 工具 diff（HEAD→工作区）**仅 5 行**：2 行注释 + 1 行 `mesen_timeout_s` + 1 行命令改为含 `--timeout=`；无其它逻辑改动 ✅。
- `tools/mesen2/Mesen.exe`（sha `d2eb03c2…`）含 ASCII `TestRunnerTimeout` ×4 ✅。

### 1.4 范围 / 门禁

- `git status --porcelain`：仅 **3 个 M**——两个 docs + `tools/scenario_run.py`（工具修复待随本卡提交）；`git diff --stat` = 48 insertions/7 deletions，全部为超时相关（+ tool 5 行）；无其它 docs 被本卡改动 ✓。
- `python gates/cli.py status` → **docs-links PASS（348 md/442 links）**、**knowledge-format PASS（24 条）**、`SUMMARY: PASS=11 / FAIL=0 / SKIP=1 / TOLERATED=0`、`STATUS: OK` ✓。
- 未运行 `scenario_run.py`（遵守串行锁约束）；动态 100s 截断/`timed_out=false` 行为在本轮为**静态代码+二进制+文档交叉核**，未见需实跑才能排除的矛盾（见 Info-2）。

## 2. findings 四分类

### 2.1 实现/工具（B）——无

工具改动最小且与文档命令逐字符对得上（`--testRunner --timeout=N <lua> <rom>`；`max(120,int(timeout_s)+120)`）。

### 2.2 证据/文档（E）——无阻塞；Info：

- **Info-1（精确性，建议顺手改）**：文档两处写 `max(120, timeout_s+120)`，代码为 `max(120, **int(timeout_s)**+120)`（L550）。CLI `--timeout` 为 float，如传 `600.5` 文档公式=720.5 vs 代码=720（差 0.5s 语义等价）。建议文档写成 `max(120, int(timeout_s)+120)` 或注明"按秒取整"。
- **Info-2（证据等级已如实）**：`TestRunnerTimeout` 默认 100s 的最终依据是上游 `CommandLineHelper.cs:31`（Mesen2 源码不在仓库，文档已标"未独立复核"）；本机仅以 Mesen.exe 字符串存在 + 症状观测为旁证。未实跑 Mesen 验证 100s 强停（本卡禁止跑 scenario_run；且该验证会与 P0-0 采集抢串行锁/机器）⇒ 文档正确标了 B/观测级，未写成已证，符合纪律。
- **Info-3（表述小优化）**：gotcha "为什么"首条（L41-43）仍写 "`--testRunner <lua> <rom>`（参数顺序固定：先脚本、后 ROM）"，而驱动实际命令在 `--testRunner` 与 lua 之间插入了 `--timeout=N`——"先脚本后 ROM"的**相对顺序**仍成立，但建议补一句"`--timeout` 等选项置于两个位置参数之前（驱动已按此生成）"，避免读者误以为命令行完全字面固定。

### 2.3 规格缺口（S）——无（判据三项已入 §2.2；后续执行只需按 §2.1/§2.2 执行）

### 2.4 平台差异（P）

- 本轮为静态复核，未跑 Mesen2（避免抢锁）；100s 截断/`timed_out=false` 的**动态**复现属于 P0-0 执行期观测面（l4-analyst 正在跑的场景会自然覆盖；执行后可用 `run.json.timed_out/done/trace_lines` 回填验证）。

## 3. 命令与输出摘要（只读）

| # | 命令 | 摘要 |
|---|---|---|
| 1 | `git status --porcelain` / `git diff --stat` | 仅 3 个 M（两 docs + tool）；48+/7− 全为超时改动 |
| 2 | 读 `tools/scenario_run.py:540-591,721-723` + `git diff tools/scenario_run.py` | `mesen_timeout_s=max(120,int(timeout_s)+120)`；命令含 `--timeout=`；20s 宽限逻辑；diff 仅 5 行 |
| 3 | Python 字节搜索 `tools/mesen2/Mesen.exe` | ASCII `TestRunnerTimeout` 命中 **4** 次（UTF-16LE 0） |
| 4 | 读知识 gotcha L31-35/L46-50/L74-84/L85-89 + `git diff` | 现象④/机理/证据指针/任务链全部与代码/文档互洽 |
| 5 | 读手册 L178-181/L247-254/L265-267 + `git diff` | §2.1 超时注/§2.2 双级超时/判据三项"缺一不可"全部到位 |
| 6 | `python gates/cli.py status` | docs-links PASS（348/442）、knowledge-format PASS（24）、PASS=11/FAIL=0/SKIP=1、STATUS OK |
| 7 | `git log -1 -- tools/scenario_run.py` | `5a675b8 feat(tools): … headless …`（headless 已入库；超时修复=工作区 M 待提交） |

## 4. 结论

**pass**：知识 gotcha（现象④/机理/证据指针/任务链）与 P0-0 手册（§2.1/§2.2/三项判据）已把"Mesen testRunner 100s 默认超时 + 驱动自动 Mesen 级 `--timeout`"同步为一致口径，
且代码事实逐字符匹配（`max(120,int(timeout_s)+120)`、`--testRunner --timeout=N <lua> <rom>`）、Mesen.exe `TestRunnerTimeout` 字符串旁证成立；docs-links/knowledge-format PASS；
改动仅 3 文件（2 docs + 工具超时 5 行），无跑 scenario_run、遵守串行锁约束。仅 3 条 Info（`int()` 精度写法、上游源码/100s 动态验证已如实标级、`--testRunner` 参数位说明），**不阻塞提交**。

> 本复核只写本文件；未改被审文件/资产，未 `git add`/`commit`；未运行 `scenario_run.py`。复核方≠实施方（AGENTS.md §5）。
