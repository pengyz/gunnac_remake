# 对抗复核：scenario_run headless 显式 `--timeout`（Mesen2 TestRunner 100s 截断修复）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：Lead 的
> `tools/scenario_run.py` `--timeout` 修复（工作区未提交）
> 复核方式：代码/二进制静态核验 + 现有产物反证 + 短场景实跑；长 >100s 动态由 l4-analyst 的 P0-0 采集
> 使用修复后驱动实际覆盖（本卡不与其争串行锁）。本文件为唯一写入，未 git add/commit；未改工具。

## 0. VERDICT

**pass（静态面）/ 动态 >100s 待 P0-0 采集确认**（1 条中优先级失败路径建议）

**修复代码正确**：headless 命令为
`[MESEN, "--testRunner", f"--timeout={mesen_timeout_s}", lua, ROM]`，且
`mesen_timeout_s = max(120, int(timeout_s) + 120)`；POSIX GUI 分支未改动。修复前症状在现有产物中
可复现：`analysis/traces/stg08-timeline.run.json` = `frames=34000, trace_lines=18171, done=None,
timed_out=False, wall_s=120.6`（与"~100s 强停 + 20s 宽限"一致）。修复后的长场景正在由 l4-analyst
以 `--testRunner --timeout=3720` 实跑（我观察到进程命令），按 Lead 协调不在本卡争锁；P0-0 完成时以
`done != null && trace_lines == frames` 确认即可。

**F90-1（中）**：单场景 `run` 在 `ok=False`（done=None/timed_out）时只打印 `!! 未完成`，**CLI 仍 exit 0**
（`main` 的 `run` 分支固定 `return 0`；只有 `run-all` 聚合时 return 1）——自动化 P0-0 若只看退出码会
误判成功；建议 `run` 返回 1（或抛 ScenarioError）。不影响 `--timeout` 修复本身，但属"失败路径"验收缺口。

## 1. 静态/代码核验

- 当前工具：`tools/scenario_run.py` 759 行（工作区 M）。
- headless 分支（约 L545-556）：
  ```python
  mesen_timeout_s = max(120, int(timeout_s) + 120)
  proc = subprocess.Popen(
      [str(MESEN), "--testRunner", f"--timeout={mesen_timeout_s}", str(lua_path), str(ROM)],
      cwd=ROOT, ...)
  ```
  - 参数顺序：`--testRunner` 参数区内、lua/ROM 之前；与上游 `--testRunner [lua script] [rom file]`
    的**相对顺序**不冲突（Mesen 观察到 `--testRunner --timeout=3720 <lua> <rom>` 正常启动）✅。
  - 计算式 `max(120, timeout_s+120)` 保证 Mesen 级超时恒大于驱动级 timeout；驱动 default 1800
    ⇒ Mesen 1920s ✅。
  - GUI/POSIX 分支仍为 `[Mesen, ROM, lua, /DoNotSaveSettings]` + Xvfb，未被改动 ✅。
- **`TestRunnerTimeout` 依据**：本机 `tools/mesen2/Mesen.exe` 二进制 ASCII 搜索 `TestRunnerTimeout`
  **命中 4 次**；`timeout=` 字符串未直接命中（Mesen2 源码不在本仓库，上游
  `UI/Utilities/CommandLineHelper.cs:31` 未独立读）。task-92 已独立核同样的代码/二进制事实，可交叉引用；
  本卡以"命令被 Mesen 接受 + 修复前产物截断"作动态旁证。
- **修复前反证**（现存 run.json，非本卡新跑）：
  ```
  stg08-timeline.run.json: frames=34000 trace_lines=18171 done=None timed_out=False wall_s=120.6
  ```
  ⇒ 不是 IO/脚本层截断，症状与 100s testRunner 强停一致；修复前若未传 `--timeout` 会复现 ✅。
- **短场景实跑（本卡）**：`gs5-stg01-clear-anchor --timeout 600` →
  `frames=5000 / trace_lines=5000 / captures=28 / shots=28 / done!=None / wall=18.0s`；
  该场景在当前环境加速，未达到 >100s，但验证了新增参数**不破坏短/中场景**；title-menu 短场景的
  1080/37/37 已在 task-84 独立通过（timeout 改动只是多一个 arg，未触碰输出路径）✅。

## 2. 动态 >100s 验证口径（按 Lead 协调）

- 我尝试 `stg08-timeline --timeout 600` 时被串行锁拒绝（l4-analyst 正在用同一锁跑 P0-0 长场景）；
  观察到其 Mesen 命令已含 `--testRunner --timeout=3720`，即修复代码已生效于实际采集。
- **本卡不与之争锁**；动态 >100s 验收在 P0-0 采集产出复核（task-86）时确认：
  `done != null` 且 `trace_lines == frames`（对 stg08 34000f / tas-stg08b2 40000f）。
- 若届时仍出现 `trace_lines < frames`、`done=None`、`wall≈100-120s` ⇒ 修复失败，按 needs_revision 处理。

## 3. 失败路径/其他

- `ok = done_info is not None and not timed_out and n_lines > 0`（L685）逻辑正确；`!! 未完成` 显式打印。
- **F90-1（中，见 §0）**：`main` 的 `run` 分支在 `run_scenario` 返回后固定 `return 0`，不检查
  `ok`/manifest；单场景失败（含 Mesen 内部 timeout 落 done=None）退出码仍为 0。建议 `run` 分支
  `m = run_scenario(...); return 0 if (m["done"] and not m["timed_out"]) else 1`，或统一抛 ScenarioError。
- **F90-2（信息）**：Mesen2 源码不在仓库，100s 默认值只能以上游代码/二进制字符串为旁证；task-92
  已如实标注，本复核沿用，未把"未独立读源码"写成已证。
- 未改工具/未 git add/commit；复核期间串行锁按纪律等待，未强杀其他 Mesen。

## 4. findings（四分类）

### 阻塞（P0）
- 无（就 `--timeout` 修复本身；动态确认待 P0-0）。

### 重要（P1）
- **F90-1（中）**：`run` 子命令 `ok=False` 时 CLI exit 0，不满足严格失败路径；建议改为非零退出。
  不阻塞 `--timeout` 修复提交，但 P0-0 自动化建议先修。

### 次要（P2）
- **F90-2（低）**：`TestRunnerTimeout` 默认 100s 的上游源码未在仓库/未独立读，只有二进制字符串 +
  截断产物旁证；文档/知识库继续保持"未独立复核源码"标注即可。

### 建议（P3）
- **F90-3**：P0-0 完成后把 `stg08-timeline`/`tas-stg08b2-direct` 的 `done/trace_lines==frames`
  作为固定验收字段写入 run 清单检查；不要只看 `timed_out`（本次旧产物 `timed_out=False` 但 done=None）。
- **F90-4**：`--timeout` 加 `max(120, ...)` 的对齐测试可写入工具 selftest（如给定 timeout=10 生成
  `--timeout=120`、timeout=600 生成 720），防回归。

## 5. 命令/环境摘要

```text
tool: tools/scenario_run.py 759 行（工作区 M）
headless: [MESEN, "--testRunner", f"--timeout={mesen_timeout_s}", lua, ROM]
mesen_timeout_s = max(120, int(timeout_s)+120)
binary Mesen.exe: ASCII 'TestRunnerTimeout' ×4；'timeout=' 字面未独立命中（源码不在仓库）
pre-fix artifact: stg08-timeline.run.json = frames34000/trace18171/done=None/timed_out=False/wall120.6
short run: gs5-stg01-clear-anchor = 5000/5000/28/28 done!=None wall18.0s
observed current l4 run: Mesen.exe --testRunner --timeout=3720 ... tas-stg08b2-direct
attempt stg08-timeline while lock held: ScenarioError "另一场景运行中"（按纪律等待，未强跑）
failure path: ok = done_info is not None and not timed_out and n_lines>0; main run 仍 return 0 (F90-1)
```

## 6. 结论

**pass（静态面）/ 动态 >100s 待 P0-0 采集确认**：命令构造/超时公式、POSIX 不回归、短场景不回归与
修复前截断反证均成立；P0-0 用修复后驱动跑出的 `done/trace_lines==frames` 是最终动态验收。
F90-1（单场景失败仍 exit 0）为中优先级失败路径建议，建议在 P0-0 自动化前顺手修，不阻塞本次提交。

---

## 点验（2026-10-18，task-93）

> 对象：`tools/scenario_run.py` commit `93a8025`（`fix(tools): scenario_run run 分支未完成时非零退出（task-90 F90-1）+ timeout 复核报告`）。
> 只核 F90-1 的 run/run-all 判据/退出码；只读 + 本文件写入，未 git add/commit。锁被 l4-analyst 占用时按
> 纪律等待（本轮未触发长时间争锁）。

### 点验 VERDICT：**pass**

- **代码**：`main` 的 `run` 分支不再固定 `return 0`；判据为
  `m.done is not None ∧ ¬m.timed_out ∧ trace_lines == frames`，不满足打印 `RUN: FAIL(...)` 并 **return 1**；
  `run-all` 同步增加 `trace_lines == frames` 判据 ✅。
- **失败路径实测**：`title-menu --timeout 1` → 驱动级 timeout，输出
  `trace_lines=256/1080, timed_out=True, done=False`，`RUN: FAIL`，**exit=1** ✅。
- **正常路径实测**：`title-menu --timeout 600` → **1080/1080/37/37 DONE，exit=0** ✅。
- 退出码与 P0-0 验收统一判据 `done != null ∧ ¬timed_out ∧ trace_lines == frames` 一致：
  失败退出码由单场景/manifest 判据驱动，不再依赖 `timed_out` 单字段（旧 Mesen 100s 自停场景
  `timed_out=False` 但 done=None/截断，也会被新判据判 FAIL）✅。
- **剩余 info**：`run-all` 用 `m["done"]` 真值判据（与 `is not None` 等价）；无阻塞问题。

### 命令/结果摘要

```text
git log: 93a8025 fix(tools): scenario_run run 分支未完成时非零退出（task-90 F90-1）
run title-menu --timeout 1:
  frames=1080 trace_lines=256 captures=8 shots=8 wall=1.0s
  !! 未完成 (timed_out=True, done=False, trace_lines=256)
  RUN: FAIL (done=False timed_out=True trace_lines=256/frames=1080)
  exit=1
run title-menu --timeout 600:
  frames=1080 trace_lines=1080 captures=37 shots=37 DONE
  exit=0
run-all 判据: m["done"] and not m["timed_out"] and trace_lines == frames
```

### 结论

**pass**：F90-1 已闭环；失败/成功退出码与 P0-0 判据一致，旧 100s 自停截断（done=None/`timed_out=False`）
也会被单场景判据抓住。动态 >100s（stg08/tas-stg08b2）仍按 Lead 协调由 P0-0 采集产出（task-86）确认。

