# L5 B1 锚证据重建跑（gs5-stg01-full/c1/c2）+ A9（task-131）

> 【2026-10-18 更正·superseded】本文件基于 headless(testRunner) 采集，其 emu.setInput 不生效 ⇒ 输入未注入、语义无效；重采见 task-133（GUI 模式）；原数字保留作历史。

> 日期：2026-10-18｜执行：platform-verifier（test-runner 执行 + 记录）｜状态：**BLOCKED / needs_revision**
> 任务：task-131（L5 B1 补跑 + s5_anchor_rederive + A9）；环境：Windows Mesen2 2.1.1 headless，
> `python tools/scenario_run.py run <name> --timeout 1200`；只读/短场景，未改代码/scenario/gates/expected。
> 产物：`analysis/traces/gs5-*`（gitignored）、stdout 见 §3/§4；本报告唯一入库文件（untracked）。

## 0. 结论（先读）

**未通过验收**：3 条短场景都 DONE（trace_lines==frames、frame_errors=0），但 **`s5_anchor_rederive.py`
`TRACE-DIFF: MISMATCH (3/8)`、SUMMARY 6/15、F1/F2 多条 FAIL**；不是工具/文件缺失，而是**输入没有生效**
（游戏仍停在开场故事/菜单，未进入 stage1 战斗与关末链）。A9 静态面 PASS（natural 7/7），与本次 Mesen
输入问题不矛盾。

**关键证据（强烈指向 `emu.setInput` 在当前 headless testRunner 下未注入）**：
- `gameplay-stage1` 预采集 f1260 本应 stage-start，截图 `shots/gameplay-stage1_f01260_stage-start.png`
  实际是**开场故事文本**（sha `8f5c2ffa…`）；
- 新跑 `gs5-stg01-clear-anchor` f2400 gate-inject 截图仍是开场故事（sha `4f3d7af2…`）；c1 f2400 同 sha；
- anchor trace：`flags27` 全 0、`stage_no` 全 1（无 INC $180）、`countdown2f` 从无 50 环；c1/c2 同；
- `gs5-stg01-full`：`flags26 bit4` 置位窗 = []（预期 f913..1192）；
- c1/c2 设计上 f3201+ 应因输入分叉；实际 `c1↔c2 mismatch=0`（f1..5000 全同）。
- 生成 lua 的 presses 调度存在；`scenario_capture.lua:199` 为 `pcall(emu.setInput, btn, 0)`，但**未记录
  控制器读回/未断言输入生效**；`inputs.jsonl` 仅 bot 场景录制，非 bot 无读回证据。

⇒ **L5 B1 锚重建被输入层阻塞**；当前 3 条 trace 不可作为锚证据。**同时** P0-0 的 I2/I3/I4 非 title
trace（gameplay/stg03/stg08）可能同样受输入未生效影响；P0-0-2 actual 六同 blob 的"源画面同图"也可能
只是**开场静态屏**，task-126 OBS-1 结论需在输入修复后重验（不得提交当前基线）。

## 1. 三条短场景执行（DONE 有效，但语义无效）

| 场景 | frames | trace_lines | done/timed_out | frame_errors | wall | trace sha256 前 16 |
|---|---:|---:|---|---:|---:|---|
| gs5-stg01-full | 15000 | 15000 | DONE / false | 0 | 47.1s | `74d8fde79a880b1b` |
| gs5-stg01-clear-c1 | 6000 | 6000 | DONE / false | 0 | 17.5s | `85c4f30467ea8243` |
| gs5-stg01-clear-c2 | 5000 | 5000 | DONE / false | 0 | 14.5s | `25aae5d47489dd46` |
| （补）gs5-stg01-clear-anchor | 5000 | 5000 | DONE / false | 0 | 14.5s | `25aae5d47489dd46`（同 c2） |

- 命令均为 `python tools/scenario_run.py run <name> --timeout 1200`；DONE 判据
  `done!=null ∧ ¬timed_out ∧ trace_lines==done.trace_lines ∧ frame_errors==0` 全过。
- 但 **DONE 只证明采集器写完，不证明输入/语义正确**；以下 RAM/画面证明语义不对。

## 2. `s5_anchor_rederive.py` 输出（完整 stdout）

运行：`PYTHONIOENCODING=utf-8 python -X utf8 tools/s5_anchor_rederive.py`（**不加 `-X utf8` 时**，
Windows GBK 会在 F2 的 `↔` 处抛 `UnicodeEncodeError`，工具需修输出编码；见 findings）。

```text
[FAIL] RAM stg01-clear f2401: flags27=0x00 ∧ countdown2f=61（f2400 flags27=0）
[FAIL] RAM flyout-vector f2794: countdown2f=67 ∧ vec41c=0 ∧ vec438=0（f2793=1，判定帧唯一）
[FAIL] RAM gate300-open f3066: gate300=0x00 ∧ flags26=0x10（bit4 清）
[FAIL] RAM inc-180 f3096: stage_no 1→1（INC $180 写帧）
[FAIL] RAM stg02-shop f3116: stage_no=1 ∧ player_state=104（$9BC3 init）
[ok] SHOT stg02-exit f3400: capture tag=exit-fade4 记录 1 条 + 截图/RAM dump 实存
[ok] SHOT stg02-start f3450: capture tag=stg02-start 记录 1 条 + 截图/RAM dump 实存
[ok] SHOT stg02-combat f3900: capture tag=stg02-combat 记录 1 条 + 截图/RAM dump 实存

TRACE-DIFF: MISMATCH (3/8)

── 复核 #74 更正口径复算（F1/F2）──
[FAIL] F1 bit4 过渡窗定界（Run A）: flags26 bit4 置位窗 = []（f913..1192=菜单→开 stage 过渡窗；非「全程 0」）
[FAIL] F1 过渡窗语义（开 stage）: f1193 bit4 清 + intro_ctr=0（关始引入；窗口期间 countdown2f=17 非倒数环）
[ok] F1 $CA 全程 0（此条原断言成立）: trig_ca 非零帧数 = 0
[FAIL] F1 关末语义三联未出现（Run A）: 倒数环（50→0·8f DEC）=无；INC $180 转移=[5217, 7413, 12631]（flags27 腿：轻量面未 watch $27，经同前缀 anchor run f913-1192 同窗 flags27=0 旁证）
[FAIL] F2 c1↔c2 共享段（18 字段 × f1..3200）: f1..3200 mismatch=0；f3201..5000 mismatch=0 帧（设计性分歧：c1 B 试探 vs c2 down 导航）
[ok] F2 c2↔anchor 全程（18 字段 × 5000f）: mismatch=0
[ok] F2 gameplay-stage1↔full（7 字段 × 2400f）: mismatch=0（zp/iram blob 按场景 watch 映射解：wram=$6000-7FFF SRAM 区非 $0000 基址）
SUMMARY: 6/15 通过（对拍 8 + 更正口径 7）→ MISMATCH
```

## 3. A9 `l5_probe_check.py`（独立）

`python -X utf8 tools/l5_probe_check.py --selftest`：
```text
SELFTEST: ok (repo files present + status parser 4/4)
  note: full chain needs cargo + engine_tick::emit_probe5 (see --help)
```
随后按工具例行路径（同命令无 `--selftest`）：
```text
emit: CARGO_TARGET_DIR=target/l5probe cargo test -p game-core --test engine_tick -- --ignored emit_probe5
emit: | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out
artifacts: 12/12 present in analysis/traces
passA: natural_ok=7 (expect 7) ok-injected=4 unknown=33 mismatch=0 provenance-note=0 | diff_l5 rc=1
passB: ok-injected=27 unknown=17 mismatch=0 | diff_l5 rc=1
L5-DIFF natural 7/7
A9: PASS (local probe5 natural face OK; real-machine Mesen2 face still needs env)
```
⇒ A9 **工具/静态面可用**；本次失败在 Mesen 输入/采集层，不在 l5_probe_check。

## 4. 输入未生效的证据（关键）

1. **画面**：
   - `shots/gameplay-stage1_f01260_stage-start.png` sha12 `8f5c2ffa6910` → 打开为**开场故事文本**
     （不是 stage-start）；
   - `shots/gs5-stg01-clear-anchor_f02400_gate-inject.png` sha12 `4f3d7af258a8` → 开场故事；
   - `shots/gs5-stg01-clear-c1_f02400_probe.png` 同 sha，说明 c1 也未进关。
2. **trace 语义**：
   - anchor/c1/c2 `flags27` 全程 0、`stage_no` 全程 1、`vec41c=0/vec438=0`；
   - `countdown2f` 在 anchor 只有零星 50（4399/4655/4911），**不在预期 f2401**；
   - `gs5-stg01-full` `flags26 bit4` 窗 []；
   - c1↔c2 0 mismatch（设计上应有 f3201..5000 分歧）——输入分叉也未生效。
3. **代码/工具**：生成 lua 的 `PLAN.presses/holds` 存在；`scenario_capture.lua:199`
   `pcall(emu.setInput, btn, 0)` 未检查返回值/未记录读回；非 bot 场景无 `inputs.jsonl`。
4. **对照**：P0-0 title-menu（无输入）正常；所有**依赖按键输入**的 gs5/gameplay/stg 场景都停在
   开场/菜单——指向 headless `emu.setInput` 注入失效（或端口/API 语义不匹配），**非** s5 工具缺文件。

## 5. 影响与建议

- **L5 B1 BLOCKED**：3 条短 trace DONE 但语义无效，不能作为 B1 锚证据；`TRACE-DIFF` 3/8、F1/F2 不达。
- **P0-0 连带风险**：I2 gameplay-stage1、I3 stg03-timeline、I4 stg08-timeline 等有输入场景
  可能同样停在开场/静态屏；P0-0-2 actual 六同 blob 的"源画面同图"可能只是开场静态，task-126 的
  OBS-1 结论**需在输入修复后重验**，当前 P0-0 基线不得提交。
- **建议修复卡**：在 `scenario_capture.lua` 增加 `emu.setInput` 返回值/控制器读回（如 `emu.getInput`）
  断言，并先跑一个最小 `press start` 场景验证游戏状态变化；核对 Mesen 2.1.1 `emu.setInput` 端口/API
  （testRunner 下是否有 override 限制）。修复后再跑这三场 + s5_anchor_rederive，才重新 B1。
- **工具健壮性**：`s5_anchor_rederive.py` 在 Windows 非 UTF-8 控制台会在 F2 `↔` 处
  `UnicodeEncodeError`；建议 `sys.stdout.reconfigure(encoding='utf-8')` 或避免该字符。
- 未改代码/gates/基线/expected；未跑 gates check/status/visual；未新增 overlay。本报告只记录失败。

## 6. 命令/环境摘要

```text
env: Windows Mesen2 2.1.1 headless；scenario_run 当前工作树；python 3.12
runs:
  gs5-stg01-full       15000/15000 DONE frame_errors0 wall47.1s sha12 74d8fde79a88
  gs5-stg01-clear-c1    6000/6000  DONE frame_errors0 wall17.5s sha12 85c4f30467ea
  gs5-stg01-clear-c2    5000/5000  DONE frame_errors0 wall14.5s sha12 25aae5d47489
  anchor rerun          5000/5000  DONE frame_errors0 wall14.5s sha12 25aae5d47489
s5_anchor_rederive: TRACE-DIFF MISMATCH 3/8；SUMMARY 6/15；F1/F2 FAIL（见 §2）
A9: natural 7/7 PASS；A9 PASS
visual evidence: gameplay-stage1 f1260 / anchor f2400 / c1 f2400 均为开场故事
```

## 7. 结论

**BLOCKED / needs_revision**：3 个场景采集层 DONE 有效，但输入未生效导致 s5 锚重建 3/8、F1/F2 失败；
A9 静态面 PASS。必须先修 headless 输入注入（或确认 Mesen 输入 API/端口），再重跑 L5 B1；当前
`gs5-*` trace 与可能的 P0-0 I2/I3/I4/P0-0-2 同 blob 结论均不得作为基线证据。
