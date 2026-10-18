---
type: gotcha
title: Mesen2 testRunner(headless) 模式下 emu.setInput 不生效——DONE 有效但输入未注入、采集语义无效（含输入场景必须 GUI 模式）
date: 2026-10-18
status: confirmed
---

## 现象

**采集全流程"DONE、零错误"，但游戏根本没动**——最隐蔽的一类假绿：

- headless（testRunner）跑的 run 全部满足完成判据：`done!=null`、`timed_out=false`、
  `trace_lines==frames`、`frame_errors=0`、退出码 0；
- 但 RAM/画面证明**输入没注入**：`$18`（controller held）恒 0、Lua `emu.getInput` 读回为空、
  游戏停在**开场故事/菜单**；`flags27` 全 0、`stage_no` 全 1（无 `INC $180`）、`countdown2f` 从无 50 环、
  `flags26 bit4` 预期置位窗为空；
- 截图侧：`gameplay-stage1_f01260_stage-start.png` 实际是**开场故事文本**（sha `8f5c2ffa…`；
  该文件已被 GUI 重采覆盖，此处为 headless 期历史值），注入场景 `gs5-stg01-clear-anchor` f2400
  截图同样是开场故事（sha `4f3d7af2…`，同为历史值）；
- 设计上应分叉的两个输入序列 `c1↔c2` 实际 `mismatch=0`（f1..5000 全同）——输入若生效不可能全同；
- L5 B1 锚重建因此 `TRACE-DIFF: MISMATCH (3/8)`、SUMMARY 6/15（工具/文件都在，是输入层失效）。

**关键对比（同机同 ROM）**：**GUI（窗口）模式 `emu.setInput` 生效**——`emu.getInput` 返回
`{start=true}`；`gameplay-stage1` GUI trace `game_action_mode=7`（Combat/Stage 推进）、玩家 x 由 128 移到 243、
bomb 3→4。

## 为什么

- **testRunner（headless）模式不承载真实输入注入**：`emu.setInput` 在该模式下是 no-op（读回空），
  游戏收不到按键 ⇒ 所有依赖输入的推进（开始游戏/菜单/战斗/Boss/结局）都不会发生。
  这也解释了为什么"truncation/IO 开关"修完之后**结果仍然错**——本次是**第三条独立根因**（输入层）。
- **DONE/`trace_lines`/`frame_errors` 只证明采集脚本收尾，不证明输入/语义正确**：
  游戏在开场静止也能写满 N 帧 trace ⇒ 判据必须另加"输入有效性断言"。
- GUI 模式实时跑会慢，可临时改 Mesen 设置 `Emulation.EmulationSpeed=1000`（本机实测当前=1000）
  提速；批次结束**恢复 100**（否则后续采集/截图节奏与其它任务不一致）。

## 何时使用

- **任何含输入的场景采集前**（菜单/战斗/关末/组 D/L5 锚等）：确认走 **GUI 模式**，不是 testRunner。
- 看到"采集 DONE、trace_lines==frames、frame_errors=0，但游戏画面/状态不推进"时：**先查输入是否生效**
  （`$18`/`getInput`/`stage_no`/玩家坐标/截图），不要先怀疑 ROM/场景 DSL/渲染。
- 只有**纯无输入场景**（如 title-menu 静态锚、boot-black）可用 `GUNNAC_HEADLESS=1` 提速 headless；
  即便如此，也建议抽一帧断言画面语义。
- 复用历史 headless 采集产物（trace/截图/重裁结论）前：先确认它是否在输入失效期采集——若是，
  **数字只能作历史，语义作废**（见各 findings 的 superseded 横幅）。

## 处置（2026-10-18 起）

1. **`tools/scenario_run.py`：Windows 默认 GUI 模式**（`HEADLESS = GUNNAC_HEADLESS=="1"`；默认 `"0"`）；
   Windows GUI 分支 = `Mesen.exe <rom> <lua> /DoNotSaveSettings`（位置参数 + `AutoStartScriptOnLoad`），
   **不需要 Xvfb/显示服务**（Windows 桌面会话即可）；`GUNNAC_HEADLESS=1` 仅用于无输入场景。
2. **采集断言加输入有效性**（缺一不可）：①含按键场景 `controller_held/pressed`（或 `$18`）出现非 0；
   ②语义推进信号（`stage_no`/`action_mode` 进 Combat、玩家坐标变化、预期 flags 窗口如 `flags26 bit4`）；
   ③截图不是开场文本（对照已知开场 sha）；④两输入序列 `c1/c2` 必须分叉（mismatch>0），否则输入未生效。
3. GUI 批次提速：临时 `Emulation.EmulationSpeed=1000`（settings.json），**批次后恢复 100**；串行纪律不变。
4. 受影响的 headless 采集产物（P0-0 I1-I6、组 D C1-C3、L5 B1）→ **superseded**，按 task-133 GUI 重采后再定稿。

## 证据指针

- **实测对比（Lead 2026-10-18 20:00，本机 Windows Mesen2 2.1.1）**：headless `getInput` 空 / `$18`=0 /
  游戏停开场；GUI `getInput={start=true}`、`gameplay-stage1` GUI trace `game_action_mode=7`、玩家 x 128→243、
  bomb 3→4。复现：分别以 `GUNNAC_HEADLESS=1` 与默认（GUI）跑同一含输入场景，比较 trace `$18`/坐标/截图。
- **独立旁证（L5 B1 重建跑）**：`docs/findings/l5-b1-anchor-rederive-run1.md` §0（`MISMATCH 3/8`/SUMMARY 6/15、
  开场故事截图 sha、`flags27`=0、`c1↔c2 mismatch=0`）与 §2（`scenario_capture.lua` `pcall(emu.setInput, btn, 0)`
  未记录读回——输入是否生效此前无断言）。
- **驱动代码事实**：`tools/scenario_run.py`：`HEADLESS = os.environ.get("GUNNAC_HEADLESS", "0") == "1"`、
  `GUI_WINDOWS = (not HEADLESS) and os.name == "nt"`；GUI 分支命令 `[MESEN, ROM, lua, "/DoNotSaveSettings"]`；
  注释明确"2026-10-18 实测：testRunner 模式下 emu.setInput 不生效"。**该 GUI 默认改动为 task-133 工作区改动，
  引用前验证 `git log -1 -- tools/scenario_run.py` 已含 GUI 默认。**
- **提速设置**：`%USERPROFILE%\Documents\Mesen2\settings.json` → `Emulation.EmulationSpeed`（本机实测 1000；
  批次后改回 100）。改前备份、改后恢复（同 headless gotcha 的 settings 纪律）。
- **关联条目**：[gotcha-mesen2-headless-windows-io.md](gotcha-mesen2-headless-windows-io.md)
  （同一环境族的另外两个根因：Lua IO 开关、testRunner 100s 超时截断——三者独立，都要排查）。
- **重采**：task-133（GUI 模式）进行中；受影响 findings 已加 `superseded` 横幅（见各文件顶部）。
