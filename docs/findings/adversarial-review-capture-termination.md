# 对抗复核：capture.lua 终止条件修复（frame>=frames + finalize 兜底）（task-105）——adversarial-review-capture-termination

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用实施方/Lead 自述**）
> 被审：`tools/scenario_capture.lua` 工作区修复（HEAD `d06c392` 上 M；**287 行 / 12602B / mtime 18:03:32 / sha256 `8471839B…`**，116+/65−）+ 手册 task-105 注（`docs/findings/l4-baseline-freeze-plan.md` L287-305）。
> 复核窗口纪律：未跑 `gates check/status/visual`；Mesen 仅跑短场景（C3 由 l4-analyst 执行，其完成后我借用空闲锁）；产物在 gitignored `analysis/`。
> 方法：静态 diff/代码路径逐条核 + title-menu ×2 + `target/t105/bot-short.txt` + 旧 C3/I5 归档反例 + C3 新 lua 重跑产物独立复算。**本报告只写该文件、未 git add/commit。**

---

## 0. VERDICT：**pass**（终止修复本身 0 阻塞；1 条中 finding F1 为"韧性改动掩盖 bot 错误"的验收缺口，建议另卡/在 P0-0 验收收口）

- **非 bot 回归 PASS（必须项）**：title-menu 两次新跑均 **1080 帧/1080 trace/37 shots/37 captures、wall 3.5s**，**trace sha256 = `62c9ea09580654ce…` 两跑一致，且与旧归档 `title-menu-20261018-180231` 相同**；37/37 shots 两跑逐字节相同；DONE 新字段齐全且 `overshoot=false/finalize_by=endFrame/frame_errors=0`。⇒ 非 bot 行为未变。
- **终止修复正确性 PASS**：`frame >= frames or bot_stop`（capture.lua L281）+ `finalize()` 幂等（L162-164）+ inputPolled 兜底（L218）+ overshoot 帧不写 trace（L228 `frame <= frames`）+ DONE 增 `frames_plan/frames_actual/overshoot/trace_lines/bot_stop/finalize_by/frame_errors`（L175-177）；`frames` 字段语义在非 bot 不变（=actual=plan）。
- **旧 lua 反例 PASS（独立核归档）**：旧 C3 归档 `tas-stg08-direct-20261018-180319` = **119994 行、无 `.done`**、gen.lua 旧条件 `if frame == frames or bot_stop`；旧 I5 归档 `tas-stg08b2-direct-20261018-173713` = **39998 行、无 `.done`**。均与文档"119994/120000、39998/40000 卡死"一致；修复前缺陷成立。
- **overshoot/inputPolled 兜底：已获动态证据，但来自"第一波修复版 lua"（手册已诚实标注）**：C3 新跑（18:03:20 启动）DONE=`frames_actual=120001, frames_plan=120000, overshoot=true, trace_lines=120000, finalize_by="inputPolled-overshoot"`，trace 恰 120000 行（f1..f120000，overshoot 帧 120001 未写），lua.log 有 overshoot/finalize 行，trace sha256=`d6944fe9…`；C3 gen.lua 无 `frameErrors` 逐帧 pcall 韧性（仅有 finalize 的 flush pcall；中间版本），**最终工作树版本仅动态覆盖 title-menu（非 overshoot）与 bot-short（非 overshoot/bot_stop=false）**——最终版覆盖缺口见 F2（低，代码审查支持；手册 L299 已声明"第一波修复版"）。
- **bot 短场景 DONE PASS（但为 full-plan 收尾，非 bot_stop）**：`t105-bot-short` 用最终 lua 跑 3000/3000、exit 0、DONE 字段 `frames=3000, frames_plan=3000, frames_actual=3000, overshoot=false, trace_lines=3000, bot_stop=false, finalize_by=endFrame, frame_errors=59`；DONE 落盘。**bot_stop 早停路径本用例未触发**（bot 未 wants_stop，跑满 3000），仅代码审查；且 59 次 `frame_errors` 暴露 F1。
- 格式：`docs-links` **PASS**（359 md/449 links）；本卡只新增本报告，未改被审文件。

---

## 1. 修复代码逐条核（A：工作树实体）

| 修复点 | 实现位置 | 核对结论 |
|---|---|---|
| 终止条件 `frame >= frames or bot_stop` | capture.lua L281 | ✓ 非 bot 正常帧与旧 `==` 行为等价（frame 只能 =frames 或跳过）；bot_stop 保留 |
| `finalize()` 幂等 | L162-164（`doneWritten` L50） | ✓ 重复回调直接 return；trace/cap flush/close 用 pcall |
| inputPolled 兜底 | L218 `if frame > frames and not doneWritten then finalize(false,"inputPolled-overshoot")` | ✓ 覆盖"渲染帧被跳过、endFrame 不回调"；C3 动态命中 |
| overshoot 帧不写 trace | L222 `if frame <1 or doneWritten then return`；L228 `if frame <= frames then ... trace/cap/bot.on_frame` | ✓ C3 trace 恰 120000 行（f1..f120000），overshoot 帧未写 |
| DONE 字段 | L175-177 格式：`frames, frames_plan, frames_actual, overshoot, trace_lines, captures, shots, bot_stop, finalize_by, frame_errors` | ✓ 全部落盘；非 bot `frames`=plan=actual |
| 非 bot 语义不变 | 同上 + title-menu 实测 | ✓ trace sha 与旧归档一致 |
| 卡面外"韧性"改动 | L51 `frameErrors`；L190-216 inputPolled 整段 pcall；endFrame 工作体 pcall（L~244-268）；`BOT.wants_stop` pcall（L273-280）；DONE 加 `frame_errors` | 见 F1/F2：**合理但与 task-105 卡面不完全一致，且改变错误路径行为**（错误不再中断 DONE；inputPolled 错误帧会跳过该帧 setInput/record/poke） |

> 手册 task-105 注（L287-305）覆盖终止/overshoot/判据/pcall"补强"；L299 明确 C3 实测为**第一波修复版 lua**，与实际时间线一致（C3 gen.lua 无 frameErrors；当前文件 mtime 18:03:32 晚于 C3 启动 18:03:20），**未夸大**。

## 2. 非 bot 回归（必须；我独立复跑）

| 跑 | 命令/输出 | frames/trace/shots | trace sha256 | DONE |
|---|---|---|---|---|
| title-new#1 | `scenario_run.py run title-menu`（新 lua；输出覆盖到 `analysis/review-capture/title1`） | 1080/1080/37/37 wall 3.5s | `62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331` | `overshoot=false, finalize_by=endFrame, frame_errors=0` |
| title-new#2 | 同上（title2） | 1080/1080/37/37 wall 3.5s | **同 #1** | 同 #1 |
| 旧归档 | `analysis/traces/.archive/title-menu-20261018-180231` | — | **同** `62c9ea09…` | — |

- 37/37 shots 两跑**逐字节相同**（逐文件 sha 对比 `all_bytes_equal=True`）；37 个 capture/shot 数一致。
- 结论：**task-105 非 bot 行为不变（A）**。注：两次跑使用的工作树 `scenario_run.py` 已含 task-110 的 DONE 判据（mtime 18:04:40），本卡不评价 task-110；title 两跑 exit 0。

## 3. 旧 lua 反例（A：归档实体，独立计数）

| 证据 | 旧 C3（`tas-stg08-direct-20261018-180319`） | 旧 I5（`tas-stg08b2-direct-20261018-173713`） |
|---|---|---|
| frames.jsonl 行数 | **119994**（f1..f119994） | **39998**（f1..f39998） |
| `.done` | **不存在** | **不存在** |
| lua.log | 仅 started+4 poke，无 done/overshoot | 同 |
| gen.lua 终止条件 | L1003 旧 `if frame == frames or bot_stop then`；无 `finalize`/`doneWritten` | 同（旧 harness） |
| 旧 trace sha256 | `ab89fcd3…`（119994 行） | 未列（同旧版） |

- 与 `docs/findings/p0-0-capture-run1.md` 的 I5 R2/R3 `39998/40000 无 DONE`、任务卡的 C3 `119994/120000 无 DONE` 独立吻合。旧缺陷=跳帧越过 `frames` 后 `==` 永不命中，且无兜底。
- **修复后 C3 重跑**（第一波 lua）：120000 行、DONE `overshoot=true/frames_actual=120001/finalize_by=inputPolled-overshoot`、lua.log 含 overshoot+done（§0）；旧/新对照成立。
- C3 新 trace 首/末：`f1`…`f120000`（120000 行），**overshoot 帧 120001 未写 trace**；DONE 记 actual=120001——符合设计。

## 4. bot 短场景与 pcall 韧性（F1 核心证据）

- `target/t105/bot-short.txt`（frames 3000，`bot stg01-clear from 1260`，4 poke）用**最终工作树 lua**跑：
  - 3000/3000 trace、exit 0、wall 10.5s、captures/shots=0、DONE `frames_plan=3000, frames_actual=3000, overshoot=false, bot_stop=false, finalize_by=endFrame, frame_errors=59`。
- `frame_errors=59` 的唯一错误消息：
  `inputPolled error at frame 1260/1290/…/3000: t105-bot-short.lua:186: attempt to perform arithmetic on a nil value (local 'hp')`（每 30 帧一次，共 59 次；生成 lua L186 是 `boss_hits_to_kill` 的 `math.ceil(hp/…)`，调用点 L692 `boss_hits_to_kill(boss_hp, …)`；`tools/tas_bot.lua` L688 同源）。
- **含义**：temp bot 场景的 `stg01-clear` 决策在特定帧拿到 nil `boss_hp`；**pcall 兜底使运行不中断并落 DONE（这是韧性改动想要的效果）**，但该帧的 `setInput/record_input/poke` 因整段 pcall 中断而被跳过（cap.lua L190-216）——即 bot 行为在该 59 帧缺失注入，trace 仍完整。**不是 capture.lua 的终止 bug，但会被 DONE 成功掩盖**；当前 driver 判据（含 task-110 版）不检查 `frame_errors`，该 run 会被当"合法完成"。
- 结论：pcall 韧性**方向合理**（防"逐帧工作体异常→DONE 前中断"；手册 L296-298 述及），但 **task-105 卡面未列 frameErrors/pcall、且未定义 `frame_errors>0` 的验收语义**；F1 建议：
  1. P0-0 基线验收加 `done.frame_errors == 0`（或显式 DEGRADED/需复核），否则错误被静默吞掉；
  2. 修 temp bot-short 的 nil `boss_hp`（或明确该政策不适用此场景）——非 capture.lua 范围；
  3. 手册 DONE 字段清单 L291 补 `finalize_by/frame_errors`（L296-301 正文已提，但清单遗漏；F4 低）。

## 5. 跨卡整合缺口（复核中发现 → Lead 已立 task-110）

- **复核开始时**（HEAD `d06c392`）`tools/scenario_run.py` run 判据为 `done is not None ∧ ¬timed_out ∧ trace_lines == plan.frames`（HEAD L743-746）：bot 早停（`trace_lines<plan`）或极端 overshoot（`trace_lines≠plan`）即使 DONE 正常也 `RUN: FAIL`、exit 1——**与 task-105 新 DONE 语义冲突**。本卡将其列为跨卡 finding（不属 capture.lua 修复范围）。
- **复核期间**工作树已出现 task-110 修复（`scenario_run.py` M，mtime 18:04:40）：新增 `done_ok()`（L453-474）按 DONE `trace_lines` 判定、legacy 回退 plan；run 分支改调用（L768-779）；新增 `tools/test_scenario_run.py`（四态单测）。**本卡不将其计入 pass**，待 task-110 独立点验；我的 title/bot 两次跑在此工作树版本上 exit 0（但 bot-short trace_lines=plan，旧/新判据均通过，未覆盖早停分支）。
- 手册 L275-278/L303-305 已同步 task-110 判据（以 DONE 为准、plan 仅上限），与代码意图一致。

## 6. Findings 四分类

### 6.1 证据不足/行为风险（中）
- **F1（中，验收缺口）**：`frameErrors/pcall` 韧性改动使逐帧异常不再中断 DONE，但 **`frame_errors>0` 未被任何验收判据拒绝**；bot-short 实测 59 次错误仍"成功"。风险=bot/capture 决策错误被静默吞掉、P0-0 数据降质；且该改动超出 task-105 卡面（含 DONE 字段扩展）。建议：driver/P0-0 验收加 `frame_errors==0`（或显式 DEGRADED 标记进入复核），另卡修 bot-short nil hp；本卡终止修复本身不受影响。

### 6.2 规格/文档（低）
- **F2（低，覆盖时点）**：最终工作树 lua（含 frameErrors/pcall）未动态覆盖 overshoot/bot_stop；C3 overshoot 证据为第一波 lua（手册已声明，未夸大）；bot-short 为 full-plan（bot_stop=false）。建议要么接受"代码审查 + 第一波实测"证据，要么用最终 lua 再跑一次任意 overshoot 场景（C3/C2 重跑时顺带；不阻塞本卡）。
- **F3（低，文档清单）**：手册 L291 DONE 字段清单未列 `finalize_by/frame_errors`（正文 L296-301 已提）；建议补齐清单。
- **F4（低，跨卡，已路由）**：scenario_run run 判据旧版对 bot 早停/overshoot 误判；Lead 已立 task-110，复核期间工作树已见修复；本卡仅登记，待 task-110 点验。

### 6.3 实现/行为
- 无阻断缺陷；终止/finalize/overshoot 逻辑与实测一致。非 bot trace sha 未变（62c9ea09…）。
- 额外改动范围：`frameErrors`/pcall/`frame_errors` 非卡面最小修复，但属合理韧性补强；副作用（错误帧跳过输入注入）见 F1。

### 6.4 越权/伪造
- 无。被审方未改判定逻辑/门禁（capture.lua + 手册注）；未发现伪造数据或把未证写成已证；手册对 C3"第一波修复版"、`frame_errors` 补强均显式。**本复核只新增报告文件，未改工作树其他文件、未 git add/commit。**

---

## 7. 命令与输出摘要

```text
# 版本/实体
tools/scenario_capture.lua 287 lines / 12602B / mtime 18:03:32 / sha256 8471839B…
git diff: 116 insertions, 65 deletions（当前 HEAD d06c392；仅 capture.lua + 手册注）

# 非 bot 回归（新 lua，隔离输出 analysis/review-capture/title{1,2}）
title#1: frames 1080 trace_lines 1080 captures 37 shots 37 wall 3.5s
  DONE {"frames":1080,"frames_plan":1080,"frames_actual":1080,"overshoot":false,
        "trace_lines":1080,"captures":37,"shots":37,"bot_stop":false,
        "finalize_by":"endFrame","frame_errors":0}
  trace_sha 62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331
title#2: 同 #1；两跑 37/37 shots 逐字节相同
旧归档 title-menu-20261018-180231 trace sha 同 62c9ea09…（前 32 位一致）

# bot-short（最终 lua）
3000/3000、exit 0、wall 10.5s、DONE overshoot=false/bot_stop=false/finalize_by=endFrame/frame_errors=59
errors: "attempt to perform arithmetic on a nil value (local 'hp')" ×59（frame 1260..3000，步长 30）

# 旧 lua 反例（归档实核）
旧 C3 tas-stg08-direct-20261018-180319: frames 119994 行、无 .done、gen.lua `frame == frames`
旧 I5 tas-stg08b2-direct-20261018-173713: frames 39998 行、无 .done

# 修复后 C3 重跑（第一波 lua；l4-analyst 执行，我独立核）
DONE {"frames":120001,"frames_plan":120000,"frames_actual":120001,"overshoot":true,
      "trace_lines":120000,"captures":80,"shots":80,"bot_stop":false,
      "finalize_by":"inputPolled-overshoot"}
trace 120000 行（f1..f120000；overshoot 帧不写），sha256 d6944fe9…
lua.log: scenario overshoot(inputPolled-overshoot): frame=120001 plan=120000 … done … finalize_by=inputPolled-overshoot

# 跨卡（复核开始时 HEAD）
scenario_run.py L743-746 ok = done is not None ∧ ¬timed_out ∧ trace_lines == plan.frames
# 复核期间工作树 task-110（M，mtime 18:04:40）已有 done_ok()（L453-474）+ run 调用（L768-779）+ tools/test_scenario_run.py
docs-links: PASS（359 md / 449 links）
```

## 8. 复核边界

1. 未跑 `gates check/status/visual`（Lead 纪律）；docs-links 只做只读链接扫描（PASS）。
2. Mesen 使用与 l4-analyst 串行协调：C3 完成后借空闲锁跑 title ×2 + bot-short，结束即回锁；未抢跑。
3. **C3 overshoot 动态证据来自第一波 lua**（gen.lua 无 frameErrors/pcall；手册 L299 已注明），最终 lua（含 pcall/frameErrors/sha 8471839B）动态覆盖=title-menu ×2 + bot-short（均非 overshoot，bot_stop=false）；最终版 overshoot/bot_stop 仅代码审查支持（F2）。
4. 旧 lua 反例来自归档 + gen.lua 实体，未用旧 lua 重跑（避免长时间/额外 Mesen 占用）；C3/I5 证据与任务卡/采集汇总一致。
5. task-110 修复在本复核期间出现于工作树（scenario_run.py M + 单测），**未纳入本卡 pass**，待其独立点验。
6. 被审文件/工作树其他改动（capture.lua、manual、scenario_run.py、test_scenario_run.py）均未由本复核修改；本报告为唯一新增文件。

---

【2026-10-18】decomp-verifier，复核 task-105（HEAD `d06c392`；capture.lua 工作树 sha `8471839B…`）；非 bot 回归 + 旧 C3/I5 反例 + C3 overshoot 兜底 + bot 短场景。
