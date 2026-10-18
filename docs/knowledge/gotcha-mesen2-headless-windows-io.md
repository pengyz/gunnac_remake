---
type: gotcha
title: Mesen2 2.1.1 headless 采集：Windows 版 testRunner + Debug.ScriptWindow.AllowIoOsAccess（WSL2 Linux 版 testRunner 实测 std::bad_cast）
date: 2026-10-18
status: confirmed
---

## 现象

多段式坑（环境层 + 配置层 + 超时层 + **输入层**），都会让采集**看起来在跑、实际没产物/产物被截断/语义无效**：

> **【2026-10-18 追加·输入层（独立第三/四根因）】** headless **testRunner 模式下 `emu.setInput` 不生效**
> ⇒ 采集 DONE/trace 满行但**输入未注入、语义无效**（游戏停开场），含输入场景必须走 GUI 模式——
> 见 [gotcha-mesen2-testrunner-setinput-noop.md](gotcha-mesen2-testrunner-setinput-noop.md)。
> 本条与 IO 开关、100s 超时截断是**彼此独立**的根因，排查时都要过一遍。

1. **本机 WSL2 Ubuntu 22.04 里跑 Linux 版 Mesen2 2.1.1 的 testRunner 调用：崩**——
   `terminate called after throwing an instance of 'std::bad_cast'`，**exit 134**（SIGABRT / core dumped）。
   本机复现（2026-10-18；Windows 宿主经 `wsl.exe` 调 WSL2，命令与输出如实抄录）：
   ```text
   $ wsl.exe -d Ubuntu-22.04 -- bash -lc 'cd /mnt/d/workspace/gunnac_remake && \
       timeout -k 3 20s ./tools/mesen2/Mesen --testRunner tools/scenario_capture.lua rom/gunnac_cn.nes'
   terminate called after throwing an instance of 'std::bad_cast'
     what():  std::bad_cast
   timeout: the monitored command dumped core
   # exit=134
   ```
   **范围限定**：以上只声明**本机 WSL2 Ubuntu 22.04 实测**（环境验证记录：GUI 与 testRunner 两种调用
   均崩；本机复核时 GUI 无显示调用表现为**挂起**（`timeout -k` 后 exit 137，未取得崩溃输出），故
   **testRunner 面的 raw 输出是唯一可复现判据**；不写成"任何环境/任何 invocation"的通用结论）。
   崩溃发生在初始化段——换 ROM/脚本/权限不作为修复手段，先换宿主。
2. **换 Windows 版 Mesen2 2.1.1 x64 后能启动，但 headless 采集会"静默缺产物"**：Lua 脚本能跑、帧号能推进，
   但需要 Lua `io`/`os` 完成的落盘（trace / shots / DONE）不生效——Lua 的 IO 权限默认关闭。
3. **改错位置不生效**：把 `%USERPROFILE%\Documents\Mesen2\settings.json` **顶层**的
   `ScriptWindowConfig.AllowIoOsAccess` 改 `true`，headless 下 io/os 仍不可用。
4. **长场景被 testRunner 内置超时截断（~100s）**：`stg08-timeline`（34000f）/`tas-stg08b2-direct`（40000f）
   等长场景若未显式给 `--timeout=<秒>`，会在 **~100s** 处被 Mesen **自身**强停——症状：`trace_lines` 明显
   少于 `frames`、`done` 文件缺失、`wall≈100-120s`；**注意 `run.json.timed_out` 可能仍为 `false`**
   （是 Mesen 自停而非驱动超时，驱动只在进程退出后给 20s 宽限再收工）——只看 `timed_out` 会漏判，
   必须同时核 `done!=null && trace_lines==frames`。

## 为什么

- Linux 版 2.1.1 在本机 WSL2 环境是**不可用路径**（testRunner 调用初始化即 `std::bad_cast`/exit 134，
  复现命令+输出见「现象」①；不是配置/资产问题）——不要在这条路径上调参或重试，直接换宿主。
- Windows 版的 headless 入口是 **testRunner**：`Mesen.exe --testRunner <lua 脚本> <rom 文件>`
  （**参数顺序固定**：先脚本、后 ROM）；testRunner 无窗口，**不需要显示/Xvfb**；
  脚本用 `emu.stop(code)` 结束，testRunner 随后自行退出（驱动以 DONE 文件 + 退出状态判完成）。
- IO 开关的**生效键在 `Debug` 段**：`Debug.ScriptWindow.AllowIoOsAccess: true`。
  settings.json 里可能同时存在顶层 `ScriptWindowConfig` 段（本机实测两段并存），**只改顶层不改变 headless 行为**。
- **长场景预算 > testRunner 默认看门狗**：Mesen testRunner 自带 `TestRunnerTimeout`（默认 **100s**；
  上游源码 `CommandLineHelper.cs:31`，Mesen2 源码不在本仓库，未独立复核）⇒ 长场景必须在命令行显式
  `--timeout=<秒>`（Mesen 级参数）。`tools/scenario_run.py` 的 headless 分支**已自动**追加
  `--timeout=max(120, timeout_s+120)`（`timeout_s`=驱动级 `--timeout` 参数）——**手工**调用
  `Mesen.exe --testRunner` 才需自带；只给驱动级 `--timeout` 而绕过驱动时对 Mesen 无效。
- `AutoStartScriptOnLoad` 是 GUI 脚本窗口的装载行为；**未观察到** `--testRunner` 路径依赖它
  （**未做 A/B**，本条不据此下断言——本机 settings 该键当前=True，成功跑不能区分"依赖/不依赖"）。

## 何时使用

- 在 **Windows 本机**跑 oracle 采集（`python tools/scenario_run.py run <name>`）前：确认 `tools/mesen2/Mesen.exe` 就位 + IO 开关已开。
- **换机 / 重装 Mesen / 重置 settings 后第一次跑采集**：IO 开关是**机器级配置**（在
  `%USERPROFILE%\Documents\Mesen2\settings.json`，不在仓库、git 不同步）——这是最容易复发的点。
- 症状是「Lua 跑了但 trace/shots/DONE 不落或缺文件」时：**先查 `Debug.ScriptWindow.AllowIoOsAccess`**，
  不要先怀疑 ROM/场景 DSL/驱动。
- WSL/Linux 下遇到 `std::bad_cast` 立即停：不要重试 Linux 版，改走 Windows 宿主 headless
  （POSIX 采集路径的另一分支另行评估）。
- 采集**长场景**（stg08 34000f / tas-stg08b2 40000f 等，预计 >100s）时：优先用驱动（自动传 Mesen 级
  `--timeout`）；**手工** `Mesen.exe --testRunner` 必须自带 `--timeout=<≥场景预算秒>`，且采集后核
  `done!=null && trace_lines==frames`（防 ~100s 截断产物混入基线）。
- 改 settings.json 前**先备份**：它是用户级文件，会影响本机全部 Mesen 脚本。

## 证据指针

- **WSL2 `std::bad_cast` 复现（本机 2026-10-18 实测）**：testRunner 命令 + 原始输出 + `exit=134`
  见「现象」① 的 `text` 块；同环境 GUI 无显示调用表现为挂起（exit 137，未取得崩溃输出）。
  task-83 复核 §8.3 记录"复核方尝试 WSL2 复现未取得输出（挂起/包装退出）"——故本条**只把 testRunner
  raw 输出作为可复现判据**，GUI 面不作为通用断言。
- **驱动集成（代码事实）**：`tools/scenario_run.py`（用内容锚，行号会漂移）：
  `elif os.name == "nt": MESEN = ROOT / "tools" / "mesen2" / "Mesen.exe"`（POSIX 仍取无扩展名 `Mesen`）；
  `HEADLESS = os.environ.get("GUNNAC_HEADLESS", "1" if MESEN.suffix.lower() == ".exe" else "0") == "1"`；
  headless 分支命令（含 2026-10-18 长场景超时修复）：
  `[str(MESEN), "--testRunner", f"--timeout={mesen_timeout_s}", str(lua_path), str(ROM)]`，
  其中 `mesen_timeout_s = max(120, int(timeout_s) + 120)`（无 Xvfb）；
  非 headless 分支保留自管 Xvfb + 位置参数装载；`GUNNAC_MESEN` 可覆盖可执行路径。
  入库状态：headless 支持**已入库 commit `5a675b8`**（`feat(tools): scenario_run.py 支持 Mesen2
  headless（Windows testRunner）采集`；task-80 复核 pass）；**Mesen 级 `--timeout` 自动追加=2026-10-18
  工作区修复（task-91，复核后入库）——引用前验证 `git log -1 -- tools/scenario_run.py` 与源码含
  `mesen_timeout_s`**。
- **长场景 ~100s 截断（testRunner 内置超时）**：`TestRunnerTimeout` 默认 **100s** 的依据=上游源码
  `CommandLineHelper.cs:31`（Mesen2 源码不在本仓库，未独立复核；本机 `Mesen.exe` 二进制内含
  `TestRunnerTimeout` 字符串可作配置项存在的旁证）；驱动自动传 `--timeout` 的代码见上一条；
  症状识别见「现象」④；**采集后验收判据=`run.json.done != null && trace_lines == frames`**
  （**不能只看 `timed_out`**：Mesen 自停时驱动判 `timed_out=false` 但 `done` 缺失）。
- **环境实测（2026-10-18，Windows 本机；Lead 环境打通）**：
  `python tools/scenario_run.py run title-menu --timeout 600` → 输出行
  `frames: 1080  trace_lines: 1080  captures: 37  shots: 37  wall: ~4s` + `DONE: {...}`；
  **两跑** `title-menu.frames.jsonl` sha256 相同、**37/37** `shots/title-menu_f*.png` 逐字节相同（确定性）。
  产物落 `analysis/traces/`、`shots/`（均 `.gitignore` 内、可再生）。
  运行日志旁证：`title-menu.lua.log` =「scenario title-menu started…」+「scenario done at frame 1080」；
  `title-menu.done` = `{"frames":1080,"trace_lines":1080,"captures":37,"shots":37,"bot_stop":false}`；
  `title-menu.mesen.log` 无显示相关错误（仅 CPU uninitialized read 提示）。
  本机 `analysis/traces/.archive/title-menu-*` 多跑快照旁证：`title-menu.frames.jsonl` sha256
  **跨 11 跑全同**（旁证天然确定性）；`shots` 逐字节一致性以 task-80 独立复核的两次**干净重跑**为准
  （归档快照混有中断/实验跑，不宜混算——本机既有快照中见过 1 个 37-shot 集合不同的实验跑，未定性，未采信）。
- **`emu.takeScreenshot()` 在 headless 可用**：证据=上述 headless 跑产出的 37 个 `shots/*.png`。
- **版本**：`tools/mesen2/Mesen.exe` FileVersion **2.1.1**（ProductVersion `2.1.1+137ae7ce3bf3f539…`）；
  `tools/mesen2/` 与 `tools/mesen2.zip` 在 `.gitignore`（可再生件不入库）。
- **IO 开关位置**：`%USERPROFILE%\Documents\Mesen2\settings.json` → `Debug.ScriptWindow.AllowIoOsAccess: true`
  （本机 settings 实测：`Debug.ScriptWindow` 与顶层 `ScriptWindowConfig` 两段并存，生效键为 `Debug` 段）。
  **判据 = 已 A/B 实测（2026-10-18，task-83 复核 §2/§3）**：`Debug=false`（顶层=true）⇒ **90s 超时失败、
  `trace_lines=0`、无 DONE/lua.log**；`Debug=true`（顶层=false）⇒ **1080 帧/1080 trace/37 shots/3.5s 成功**
  ⇒ `Debug` 段**必要**，顶层键**既不充分也不必要**（对 headless 无效）；settings 改前已备份、改后恢复原 sha。
- **旧口径旁证（非 headless）**：`docs/methodology.md` 记录 Mesen2 2.1.1 自动化与 `AllowIoOsAccess=true`
  为既有前置（L94/L96/L165）——旧说法对应 GUI 位置参数装载 `Mesen rom.nes script.lua /DoNotSaveSettings`；
  headless testRunner 是另一条路径，别把两者命令行混用。
- **任务链 / 落档**：环境验证卡 **task-80**（`scenario_run.py` headless 独立复核）、**task-81**（P0-0 手册 §1
  环境更正）、**task-82**（本条）、**task-83**（本条知识条目对抗复核 **pass**，A/B 实测见
  `docs/findings/adversarial-review-knowledge-mesen2-headless.md` §2/§3）、**task-91**（长场景 testRunner
  100s 超时文档同步：现象④/为什么/证据指针）；权威书面落档 = P0-0 执行手册
  `docs/findings/l4-baseline-freeze-plan.md` §1「环境接收」（task-81 dated 更正后为 Windows headless 口径；
  **引用前验证 §1 已更新**）。
