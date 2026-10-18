# 对抗复核：scenario_run 判据以 DONE 为准 + frame_errors==0（task-110）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：platform-porter（task-110）
> 被审工作区：`tools/scenario_run.py`（done_ok/run/run-all 判据）+ `tools/test_scenario_run.py`（8 单测）+
> 手册 `docs/findings/l4-baseline-freeze-plan.md` §2.2 注；均未提交。
> 复核纪律：**未跑 gates check/status/visual**（采集窗口）；Mesen 仅跑短 title-menu（~4s），未重跑长场景；
> 只读 docs-links；本文件唯一写入，未 git add/commit。

## 0. VERDICT

**pass**（2 条低优先级同步建议）

- `done_ok()` 判据**逐项正确**：`done!=null ∧ ¬timed_out ∧ trace_lines>0 ∧`
  `int(done.frame_errors)==0`；新 DONE 有 `trace_lines` 时要求磁盘行数相等，legacy 无该字段时回退
  `trace_lines==plan.frames`。
- `run`/`run-all` 均改用 `done_ok()`：合法 DONE（含 bot 早停/overshoot）exit 0，失败 exit 1；
  `run-all` 对 `frame_errors>0` 打 `RUN-ALL: DEGRADED` 并 return 1。
- `tools/test_scenario_run.py` 独立执行 **8/8 PASS**（bot 早停、overshoot、legacy、失败、非 bot 等价、
  run/run-all 退出码、frame_errors>0 FAIL）。
- 实测 `title-menu --timeout 1200` → **exit 0 / 1080/1080/37/37、trace sha `62c9ea09…`**；
  `--timeout 1` → **exit 1** + `RUN: FAIL`。
- C3 现有 run.json（`overshoot=true, frames_actual=120001, trace_lines=120000, done`）→
  `done_ok=True`；C2（`frame_errors=142149`）→ `done_ok=False`；C2 lua.log 首条异常与 Lead 提供一致：
  `inputPolled error at frame 1260: ... arithmetic on a nil value (local 'hp')`。
- 手册 §2.2（277-281/306-310 行）判据已同步；写范围仅 3 文件（tool/test/manual），未改判定算法/门禁。

## 1. 判据代码核验

`tools/scenario_run.py` `done_ok()` 实读：
```
done = manifest.done
if done is None or manifest.timed_out: return False
n_lines = manifest.trace_lines
if n_lines <= 0: return False
if int(done.get("frame_errors",0) or 0) != 0: return False
if "trace_lines" in done: return n_lines == int(done["trace_lines"])
return n_lines == plan.frames   # legacy 回退
```
- 新格式 bot 早停（`done.trace_lines < plan`）与 overshoot（`trace_lines` 与 plan 无关）均以 DONE 为准 ✅；
- legacy DONE 无 `trace_lines` 回退 plan；无 `frame_errors` 视为 0 ✅；
- `frame_errors>0`（如 C2 142149）直接 False ✅；
- `run` 失败打印 `RUN: FAIL(done=... timed_out=... trace_lines=.../frames=... done_trace_lines=... frame_errors=...)`；
  `run-all` 对 `done_ok=False` 且有 frame_errors 的打 `RUN-ALL: DEGRADED <name> frame_errors=... 首条=...` 并 return 1 ✅。

## 2. 四态单测（自跑）

```
python tools/test_scenario_run.py
  PASS test_bot_early_stop_done_ok
  PASS test_bot_overshoot_done_ok
  PASS test_cmd_run_all_exit_codes
  PASS test_cmd_run_exit_codes
  PASS test_failure_states_reject
  PASS test_frame_errors_reject_degraded
  PASS test_legacy_done_without_trace_lines_falls_back_to_plan
  PASS test_non_bot_plan_equality_still_ok
scenario_run 完成判据回归: 8/8 通过（exit 0）
```
覆盖 bot 早停/overshoot/legacy/失败 + frame_errors>0 FAIL + run/run-all 退出码；单测有效，不是空壳。

## 3. 实测

### 3.1 title-menu 成功/驱动 timeout

```
python tools/scenario_run.py run title-menu --timeout 1200
  exit=0；frames=1080 trace_lines=1080 captures=37 shots=37
  DONE: {frames:1080, frames_plan:1080, frames_actual:1080, overshoot:false,
         trace_lines:1080, ..., frame_errors:0}
  trace sha256 前 12 = 62c9ea095806 ✅

python tools/scenario_run.py run title-menu --timeout 1
  trace_lines=248/1080; timed_out=True; done=None
  RUN: FAIL（done=False timed_out=True trace_lines=248/frames=1080
             done_trace_lines=None frame_errors=0）
  exit=1 ✅
```

### 3.2 C3/C2 现有 run.json（不重跑长场景）

| run | done | top-level | done_ok |
|---|---|---|---|
| C3 `tas-stg08-direct` | `{frames:120001, frames_plan:120000, frames_actual:120001, overshoot:true, trace_lines:120000, ...}` | frames=120000, trace_lines=120000, timed_out=false | **True** ✅ |
| C2 `tas-stg07-relay` | `{frames:120000, ..., trace_lines:120000, frame_errors:142149}` | frames=120000, trace_lines=120000, timed_out=false | **False** ✅ |

- C2 `lua.log` 首条：`inputPolled error at frame 1260: ... tas-stg07-relay.lua:186: attempt to perform
  arithmetic on a nil value (local 'hp')`——与 Lead 提供的 task-115 根因线索一致，佐证 frame_errors
  不是空泛计数 ✅。
- C3 无 `frame_errors` 字段（任务-105 前 legacy DONE 形态）⇒ 按"legacy 缺字段视为 0"判 PASS，符合规格；
  不影响 C2 FAIL。

## 4. 手册 §2.2 同步

- 277-281 行已写：`timed_out=false ∧ done!=null ∧ trace_lines>0 ∧ trace_lines==done.trace_lines ∧
  done.frame_errors==0`；legacy 回退 `trace_lines==frames(plan)`，无 frame_errors 视为 0。
- 306-310 行已写 `run`/`run-all` 判据、`RUN: FAIL` 含 frame_errors + 首条异常、`RUN-ALL: DEGRADED`；
  单测文件引用正确 ✅。
- **F116-1（低）**：手册 183/266 行仍有旧句式 `done!=null && trace_lines==frames`（无 DONE.trace_lines/
  frame_errors 限定）；虽为防 100s 截断的说明，但 bot overshoot 语境下会与 §2.2 不一致，建议同批补
  "§2.2 DONE 判据"指针。

## 5. 写范围/纪律

- `git status`：`M tools/scenario_run.py`、`M docs/findings/l4-baseline-freeze-plan.md`、
  `?? tools/test_scenario_run.py`；无 crates/data/gates 改动 ✅。
- 未跑 gates check/status/visual；只跑 docs-links 只读单门：**PASS**（本轮末尾核）。
- 未 git add/commit；本报告唯一写入。

## 6. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- **F116-1（低）**：手册 183/266 行旧 `trace_lines==frames` 句式未带 §2.2 新判据指针（bot overshoot 时
  旧句会误导），建议补互链。
- **F116-2（低）**：C3 的 DONE 无 `frame_errors` 字段被按 legacy=0 通过；若后续新采集应统一带字段，
  建议在测试/报告注明"C3 是 legacy DONE 形态"，避免被误读成"已证 frame_errors=0"。
- **F116-3（低/建议）**：`test_scenario_run.py` 的 run/run-all 测试 monkeypatch `run_scenario`，未覆盖
  `done_ok` 与真实 manifest 序列化组合；现有 C3/C2 手工核补足，建议加一条"从 run.json 读入 → done_ok"
  的回归（本次已人工做）。

### 建议（P3）
- **F116-4**：失败打印已含 frame_errors/首条 lua 异常；建议 wrapper/CI 直接消费 `RUN: FAIL` 文本时同时
  输出 `lua.log` 首条（本卡已实现）。
- **F116-5**：`run-all` 对 DEGRADED 与 failed 都 return 1，但输出区分了 `RUN-ALL: DEGRADED`/
  `未完成`；建议 P0-0 自动化按这两个前缀分别告警。

## 7. 命令摘要

```text
python tools/test_scenario_run.py → exit0；8/8 PASS
title-menu --timeout 1200 → exit0；1080/1080/37/37；DONE frame_errors=0；trace sha 62c9ea095806
title-menu --timeout 1 → exit1；timed_out=True done=None trace_lines=248；RUN: FAIL
C3 tas-stg08-direct.run.json → done_ok True（overshoot=true, frames_actual=120001, trace_lines=120000）
C2 tas-stg07-relay.run.json → done_ok False（frame_errors=142149）
C2 lua.log: inputPolled error at frame 1260 ... arithmetic on a nil value (local 'hp')（与 task-115 线索一致）
manual §2.2: 277-281/306-310 已同步 DONE 判据
git status: tools/scenario_run.py M；tools/test_scenario_run.py ??；manual M
```

## 8. 结论

**pass**：DONE 优先判据、frame_errors==0、legacy 回退、四态单测、短场景成功/超时失败、C3 overshoot PASS /
C2 frame_errors FAIL、手册 §2.2 同步均核实通过；F116-1..3 为低优先级同步/测试建议，不阻塞提交。
