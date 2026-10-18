# 对抗复核：scenario_run.py headless（Windows Mesen2 testRunner）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：Lead 的
> `tools/scenario_run.py` headless 改动（未提交，diff 93+/37−）
> 复核方式：独立实跑 title-menu（正常/清理后双跑/并发/缺 ROM/缺 Mesen/配置开关切换）+
> 读代码/产物 + docs-links gate；本文件为唯一写入，未 git add/commit。
> 只读边界：未改 `tools/scenario_run.py`；仅在本地按用户任务允许临时改 Mesen settings 测试并**已恢复**
> （备份/还原 sha256 见 §5）；未创建外部 worktree。

## 0. VERDICT

**needs_revision**（1 条阻塞 + 2 条次要/建议）

**headless 主路径成立**：`python tools/scenario_run.py run title-menu --timeout 600` 在本机成功：
**1080 frames / 1080 trace_lines / 37 captures / 37 shots / DONE / wall 3.5-4.5s**；两次清理后运行
trace sha 相同、**37/37 shots 逐字节相同**；`[Mesen.exe, "--testRunner", <lua>, <rom>]` 实跑成功；
并发第二实例报"另一场景运行中"；缺 ROM 超时显式报未完成；配置开关结论已实测。

但 **F1 是明确的验收阻塞**：`normalize_paths_in_outputs` **没有清掉 Windows JSON 转义形式的绝对路径**，
`analysis/traces/title-menu.captures.jsonl` 仍含 `D:\\workspace\\gunnac_remake\\...` —— 直接违反
"run.json/captures 不得残留绝对路径"。

## 1. headless 主路径/确定性

### 1.1 正常实跑 ——✅

```
python tools/scenario_run.py run title-menu --timeout 600
frames: 1080 trace_lines: 1080 captures: 37 shots: 37 wall: 3.5s
DONE: {"frames":1080,"trace_lines":1080,"captures":37,"shots":37,"bot_stop":false}
trace: analysis\traces\title-menu.frames.jsonl
```
- 与上游 `--testRunner [lua script] [rom file]` 语义一致（命令构造 `[Mesen.exe,"--testRunner",lua,rom]`，
  实跑无位置参数歧义）✅。
- `run.json`/captures 产物落 `analysis/traces`（gitignored）；trace 1080 行、shots 37 个 ✅。

### 1.2 确定性 ——⚠️ 清理后成立，首次双跑有 1 张 shot 不同

- **首次连续两跑**（运行前存在旧 `shots/title-menu_*.png`）：trace sha 相同
  （`62c9ea09580654ce…`），但 **36/37 shots 相同，`title-menu_f00420_probe.png` 不同**。
- 将 `shots/title-menu_*.png` 全部移走后**双跑**：两次均 1080/1080/37/37/DONE，trace sha 相同，
  **37/37 shots 逐字节相同（diff=0）**。⇒ headless 路径本身可确定，但**工具不清理/不归档 per-scenario
  shots**，旧 shot 会参与 glob 计数并与新产物混在一起，导致"两跑逐字节相同"的验收入口不稳定。
- 建议：run 前对 `shots/<name>_*` 做 archive/清空（与 traces archive 一致），或 runner 只统计/比对本次
  run 实际写出的文件（记录 write set），否则验收必须额外约定"先清 shots"。

## 2. 配置前置（独立实测）——✅

当前 `%USERPROFILE%\Documents\Mesen2\settings.json` 两份键均存在：
```
Debug.ScriptWindow.AllowIoOsAccess = true
ScriptWindowConfig.AllowIoOsAccess = true
```
我按允许范围临时切换并恢复（原 sha256 `5FEB2659…` → 恢复后 `5FEB2659…` 完全一致）：
- **B：顶层 `ScriptWindowConfig.AllowIoOsAccess=false`、`Debug.ScriptWindow.AllowIoOsAccess=true`** →
  **实跑成功**（1080/1080/37/37/DONE）⇒ 顶层键对 headless 无效。
- **A：`Debug.ScriptWindow.AllowIoOsAccess=false`、顶层=true** → **失败**（wall 30.1s，
  `timed_out=True, done=False, trace_lines=0, captures=0`，脚本未落 DONE）。
⇒ **`Debug.ScriptWindow.AllowIoOsAccess=true` 是唯一有效开关**，结论成立；工具代码没有硬编码配置，
  属环境前置，需在 P0-0 启动卡/README 显式记录（建议 INFO）。

## 3. 并发/失败路径

- **串行锁**：并发启动第二个 `title-menu` → stderr 报
  `Error: 另一场景运行中（串行纪律）：analysis/.scenario.lock 被占用`；第一实例正常 1080/37/37/DONE ✅
  （msvcrt 锁 1 字节实现可用，等价锁语义成立；POSIX flock 分支保留）。
- **缺 ROM**：`GUNNAC_ROM=no_such_rom.nes` → 10s timeout 后显式
  `!! 未完成：timed_out=True, done=False, trace_lines=0`（非静默成功）✅；但没有前置快速失败，建议在
  run 前 `ROM.is_file()` 检查（F3 低）。
- **缺 Mesen**：`GUNNAC_MESEN=no_such_mesen.exe` → **Python `FileNotFoundError` traceback 直接冒出**
  （line 506 `subprocess.Popen`），不是统一的 `ScenarioError`/非零中文错误。建议捕获并给明确
  "Mesen 不存在：<path>"（F2 低/中）。
- **DONE 不落/超时**：配置 A 与缺 ROM 均显式 `timed_out=True, done=False`，不冒充完成 ✅。
- **产物落点**：仍写 `analysis/traces`（gitignored）✅。

## 4. 兼容性/不回归（代码审查）

- POSIX 路径保留：`fcntl` 条件导入；`HEADLESS` 默认仅 `.exe` 为 1；非 headless 仍走 Xvfb +
  `Mesen <rom> <lua> /DoNotSaveSettings`；`pgrep` 清场仅非 headless；Xvfb finally 清理保留 ✅。
- `GUNNAC_MESEN`/`GUNNAC_HEADLESS` 覆盖语义清晰；Linux 无扩展名 Mesen 默认 headless=0，符合 POSIX 路径 ✅。
- **路径归一 F1 失败**（见下）；`run.json.outputs` 已是相对路径（但用 `\`，可接受），`captures.jsonl`
  仍是绝对路径。
- 写范围：`git status` 显示仅 `tools/scenario_run.py` 修改（+本次复核 docs 文件未提交）；未改 crates/data/gates ✅。

## 5. findings（四分类）

### 阻塞（P0）
- **F1（阻塞）`normalize_paths_in_outputs` 对 Windows JSON 转义绝对路径失效**：
  `tools/scenario_run.py:414-433` 只替换 **未转义** 的 `str(ROOT)+"/"` 和 `str(ROOT)+"\\"`；但 Windows
  `captures.jsonl` 内 JSON 序列化为 **`D:\\workspace\\gunnac_remake\\...`**（双反斜杠），实际未被替换：
  `analysis/traces/title-menu.captures.jsonl:1` 仍为
  `"shot":"D:\\workspace\\gunnac_remake\\shots/title-menu_f00030_probe.png"`、
  `"ram":"D:\\workspace\\gunnac_remake\\analysis\\traces/..."`。
  修复：同时替换 `str(ROOT).replace('\\','\\\\')` 的两种分隔符变体（或加载 JSON 递归归一字符串），
  并加回归断言 `D:\|C:\|/workspace|gunnac_wt` 零命中；这是 task 明确验收项，必须修。

### 重要（P1）
- **F2（中）确定性验收入口不稳定**：首次两跑 36/37 shots 相同（f420 差异），清理旧 shots 后两跑
  37/37 相同。建议 run 前 archive/清空 `shots/<name>_*`（或记录本次 write set 后比对），否则
  "两跑 37/37"结论依赖调用者手动清理，验收不可靠。

### 次要（P2）
- **F3（低）缺 ROM 无前置检查**：`GUNNAC_ROM=no_such_rom.nes` 会启动 Mesen 并等 timeout 10s 才报错；
  建议 `ROM.is_file()`/`MESEN.is_file()` 前置检查，缺 ROM 直接 `ScenarioError`。
- **F4（低）缺 Mesen 抛 Python traceback**：`GUNNAC_MESEN=no_such_mesen.exe` 在
  `subprocess.Popen` 抛 `FileNotFoundError`，非统一错误信息；建议捕获。
- **F5（低）`run.json.outputs` 用反斜杠相对路径**：虽非绝对，跨平台可移植性建议统一 POSIX `/`
  （与 trace/captures 输出风格一致）。

### 建议（P3）
- **F6**：把 `Debug.ScriptWindow.AllowIoOsAccess=true` 写进启动卡/README 环境前置（本复核已实测唯一有效）。
- **F7**：测试/验收命令固定"清 shots → run1/run2 → 比较 write set"，避免旧产物污染计数。

## 6. 命令/环境摘要

```text
title-menu headless: 1080 frames / 1080 trace_lines / 37 captures / 37 shots / DONE / wall 3.5-4.5s
clean two-run: trace sha 相同（62c9ea09580654ce…）；37/37 shots 逐字节相同
first two-run（旧 shots 存在）: trace 相同；36/37 shots（title-menu_f00420_probe.png 不同）
并发第二实例: "另一场景运行中（串行纪律）：analysis/.scenario.lock 被占用"
config B（顶层 false / Debug true）: 成功；config A（Debug false / 顶层 true）: 30.1s timeout/
  timed_out=True done=False trace_lines=0
settings 原/还原 sha256: 5FEB2659263C963520FAC9AF08390A075BE510200DDB9291480CA3A2BABC4708
缺 ROM: timeout 10s 后 !! 未完成（非静默）；缺 Mesen: FileNotFoundError traceback（line 506）
绝对路径残留: analysis/traces/title-menu.captures.jsonl:1 `D:\\\\workspace\\\\gunnac_remake\\\\...`
docs-links: PASS 347 md/438 links；全量 check PASS=11 FAIL=0 SKIP=1
```

## 7. 结论

**needs_revision**。headless 命令/配置/确定性（清理后）/锁/缺 ROM 失败路径基本成立；但 **F1 绝对路径
归一失效是 task 明确验收项且当前实测 FAIL**，必须先修；F2（旧 shots 污染）建议同批修/加验收前置，
F3/F4/F5 为健壮性建议。修 F1+F2 后点验即可放行。

---

## 复审（2026-10-18，task-84）

> 对象：`tools/scenario_run.py` 当前工作区（**756 行 / sha256 `d1bfd9c99590…`**，与 Lead 的 F1-F6 修正一致）。
> 只核 F1-F5；主路径/配置/并发上轮已核。只读 + 本文件写入，未 git add/commit；运行前后无残留 Mesen 进程，
> 串行锁按纪律等待其他 agent 场景结束后执行。

### 复审 VERDICT：**pass**（1 条低优先级路径细节）

- **F1 ✅**：`python tools/scenario_run.py run title-menu --timeout 600` 成功（1080/1080/37/37/DONE/wall 5.0s）；
  `title-menu.captures.jsonl`/`run.json` 均 **零 `D:`/`C:` 绝对路径**；手工向 captures 注入
  `"x":"D:\\fake.png"` 后调用 `normalize_paths_in_outputs` → **ScenarioError fail-closed**（断言生效）。
- **F2 ✅**：预置伪造残留 `shots/title-menu_f99999_fake.png` 后运行 → 本次 `shots` 计数仍 **37（不含 fake）**，
  fake **已归档**且不在 shots；连续两跑（hash 记录后复跑）**37/37 shots + trace 逐字节相同（diff=0）**。
  （归档位置见 F84-1 备注。）
- **F3 ✅**：`GUNNAC_ROM=no_such.nes` → **0.28s 立即 ScenarioError**
  `"ROM 不存在：D:\...\no_such.nes（GUNNAC_ROM 可覆盖）"`，不再等 timeout。
- **F4 ✅**：`GUNNAC_MESEN=no_such_mesen.exe`（HEADLESS=1）→ **0.18s 立即 ScenarioError**
  `"Mesen2 不存在：...（GUNNAC_MESEN 可覆盖；...）"`，无 Python traceback。
- **F5 ✅**：`run.json` 的 `outputs.trace`/`shots`/`ram_dumps` 全 `/` POSIX 相对路径
  （`analysis/traces/...`、`shots/...`）；captures 也无绝对路径。

### 逐项证据

```text
F1 run: 1080/1080/37/37 DONE wall 5.0s
  captures bytes=6352, b'D:'=False, b'C:'=False
  run.json bytes=4521, b'D:'=False, b'C:'=False
  outputs.trace=analysis/traces/title-menu.frames.jsonl
  outputs.shots[:2]=['shots/title-menu_f00030_probe.png','shots/title-menu_f00060_probe.png']
  outputs.ram_dumps[:2]=['analysis/traces/title-menu_f00030_probe.ram.hex',...]
  注入 "D:\\fake" → ScenarioError（normalize_paths_in_outputs 断言）
F2 fake: shots/title-menu_f99999_fake.png 预置 → 本次 shots=37（不含 fake）；
  连续两跑 shots diff=0（37/37）、trace sha 相同
F3 缺 ROM: wall=0.28s，ScenarioError "ROM 不存在..."
F4 缺 Mesen: wall=0.18s，ScenarioError "Mesen2 不存在..."
F5 全部 outputs 都是 / 分隔相对路径
```

### 剩余问题（低优先级）

- **F84-1（低，路径位置与任务措辞不符）**：伪造 fake 实际落在
  `analysis/traces/.archive/title-menu-<stamp>/shots/title-menu_f99999_fake.png`（`archive_existing` 的 run
  归档目录），**不是** task 卡预期的 `.archive/<name>-<stamp>-leftovers/shots/`。说明 `archive_existing`
  已先把整个 per-scenario shots 归档走，`_clear_leftovers()` 在本次路径上未必被触发；**功能结果正确**
  （fake 不计数、不在 shots、两跑干净），但若验收要求按 `_leftovers/` 字面路径，需统一归档位置/测试断言。
- **F84-2（信息）**：`captures.jsonl` 的 `ram` 路径是相对路径但仍有 JSON 转义反斜杠
  （raw `analysis\\\\traces/...`）；无绝对路径、可被 Path 解析，仅跨平台可移植性不如 `/`，可顺手统一。
- **F84-3（信息）**：复核期其他 agent 的 Mesen 场景占锁被正确处理（报"另一场景运行中"），我按纪律等待后
  才跑；非缺陷。

### 结论

**pass**：F1-F5 全部实质修复并实测通过；F84-1 归档位置措辞为低优先级，不影响"残留不进本次计数/两跑干净"
的功能验收。可提交 `tools/scenario_run.py`。

