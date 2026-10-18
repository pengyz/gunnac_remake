# 对抗复核：bot frame_errors 根因（tas_bot.lua 3 处未定义变量）（task-115）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：decomp-verifier（task-115）
> 被审：`docs/findings/bot-frame-errors-rootcause.md`（230 行，untracked）
> 复核方式：源码 + 生成 lua + C2/bot-short 日志逐条聚合；**未跑 Mesen**（含短场景）、未改代码。
> 只读 docs-links；本文件唯一写入，未 git add/commit。

## 0. VERDICT

**pass**（1 条低优先级证据指针建议；根因/计数/方案均成立）

**三条根因逐条独立复算成立（A 级）**：
- **A** `BOT.decide` L684 用未定义全局 `boss_type`/`boss_hp`（L688 调用 `boss_hits_to_kill`）⇒
  **inputPolled/lua:186 nil hp = 3,101 次，首 1260、末 120000（每 30 帧）**。
- **B** `capture(read)` L271 引用 `S`，而 `local S` 在 L299 才声明（Lua 词法作用域 ⇒ 全局 S=nil）⇒
  **inputPolled/lua:275 global S = 25,501 次，首 5326、末 119880**。
- **C** `BOT.on_frame` L738 裸 `s.px/s.py`（无局部 `s`）⇒ 全局 s=nil ⇒
  **endFrame/lua:742 global s = 113,547 次，首 5299、末 120000**。
- 合计 **3101+25501+113547 = 142,149**，与 C2 DONE `frame_errors=142149` **精确相等**；
  inputPolled 28,602 + endFrame 113,547 分解也精确。
- **bot-short 59** 在归档 `analysis/review-capture/botshort/traces/t105-bot-short.lua.log` 复核为 59 次
  （1260..3000 每 30 帧，全部 lua:186 nil hp）；B/C 未触发；当前 `analysis/traces/t105-bot-short` 是另一
  次无错误记录运行，报告数据源已列 review-capture，建议在 §2 表旁补精确路径（F120-1）。
- C3/I5 "未记录≠0" 判断成立：C3 DONE 无 `frame_errors` 字段、I5 无 DONE；两者 gen.lua 三处缺陷同源；
  "不是场景不该启用 bot" 成立（C2/C3/I5 场景均 `bot stg01-clear from 1260`，生产/组 D/I5 依赖）。
- 修复方案合理：Fix A 使用 capture 已读的 `s.boss_type/s.boss_hp` 并对 nil fail-closed；
  Fix B 前向 `local S`；Fix C 用 `read(0x04E0)/read(0x04FC)`；**不用 `hp or 0`**；非 bot 生成路径不拼
  bot 源码（`generate_lua` 仅 plan.bot 时拼接），title trace sha `62c9ea09…` 不变。
- 未证项显式（C3/I5 计数未知、bot-short B/C 未触发、修复行为待 porter）；无代码/长场景改动。

## 1. 根因逐条核（源码/生成 lua/log）

### A. `BOT.decide` 的 `boss_type`/`boss_hp`

- 源码 `tools/tas_bot.lua:672` `if fire and frame%30==0 then`；**L684** `if boss_type ~= 0 then`；
  **L688** `S.boss_hits_left = boss_hits_to_kill(boss_hp, resolve_damage(...))`。
- grep：`local boss_type` **仅 L708**（`BOT.on_frame` 内）；`local boss_hp` **0 条**（L707 是
  `local boss_x,boss_y,boss_hp` 在 on_frame）；`BOT.decide` 无这两个局部 ⇒ 全局 nil；`nil ~= 0` 恒真。
- 生成 lua `analysis/_gen/tas-stg07-relay.lua`：L692 调用行、L185/L186 `boss_hits_to_kill`；
  run.json `generated_lua_sha256=a367f463…` 与报告一致。
- 日志聚合：`inputPolled error ... lua:186 ... nil hp` = **3101 次**，首 1260、末 120000；bot-short 59 次。

### B. `capture(read)` 的 `S` 前向词法

- 源码 `capture` 定义于 **L243**，`local S = {...}` 在 **L299**；L271 在 capture 内引用 `S.shot_prev`，
  Lua 词法解析为全局 S（nil）⇒ `attempt to index a nil value (global 'S')`。
- 生成 lua L275 精确对应；日志 = **25501 次**，首 5326、末 119880。
- C2 首帧 5326 在 A 首帧 1260 之后，符合"有敌方子弹才触发"的解释（B 级触发条件）。

### C. `BOT.on_frame` 的裸 `s`

- 源码 L695 `function BOT.on_frame(frame, read)`；L738 `player_box(s.px, s.py)`；on_frame 内**无局部
  s**（只有 frame/read 及局部 boss_*），`s` 解析为全局 nil ⇒ `attempt to index a nil value (global 's')`。
- 生成 lua L742；日志 = **113547 次**（endFrame），首 5299、末 120000；占 C2 总数 80%。
- 注意 `BOT.decide` L589 的 `player_box(s.px,s.py)` 是局部 `s=capture(read)`（A 级），不是本缺陷。

### 合计

```
A 3101 + B 25501 = inputPolled 28602
C 113547 = endFrame
28602 + 113547 = 142149 = C2 DONE.frame_errors
```
逐条日志聚合与 DONE、报告表**完全一致** ✅。

## 2. 数字/边界

- C2 run.json：`frames=120000, trace_lines=120000, done.frame_errors=142149, timed_out=false`；
  报告 513.8s 与 run.json wall 一致。
- **bot-short 59**：归档 `analysis/review-capture/botshort/traces/t105-bot-short.lua.log` 为 59 行
  lua:186 nil hp，DONE `frame_errors=59`；1260..3000 每 30 帧（A 的 fire gate）。
  **F120-1（低）**：当前 `analysis/traces/t105-bot-short.lua.log` 是后一次干净/legacy 运行（0 error），
  报告数据源已列 review-capture，但 §2 表建议写精确归档路径，避免读者查错目录。
- **C3/I5**：C3 DONE 无 `frame_errors` 字段（`analysis/traces/tas-stg08-direct.done`），I5 无 DONE
  （`tas-stg08b2-direct.done` 不存在）；两者 `analysis/_gen/*.lua` 均含 L275/L692/L742 三处缺陷；
  "未记录≠0" 判断正确 ✅。
- **不是场景不该启用 bot**：`tas-stg07-relay.txt`/`tas-stg08-direct.txt`/`tas-stg08b2-direct.txt` 均有
  `bot stg01-clear from 1260`（生产采集配置），缺陷在共享 bot 代码；报告判断成立 ✅。

## 3. 修复方案核

- **Fix A**：`s.boss_type`/`s.boss_hp` 由 capture L251/L254 已读；capture 的 `s.boss_hp` 始终有值
  （0x064B 或 boss slot）；`if s.boss_type ~= 0` + `hp==nil` 时 `S.boss_hits_left=nil` + `ev("bot_degraded")`
  是合理 fail-closed；`S.boss_hits_left` 源码仅 L688 写入、无读取者（grep 证实），不改变决策 ✅。
- **Fix B**：在 capture 之前 `local S`，L299 改为 `S = {...}`，闭包捕获同一 local；正确 ✅。
- **Fix C**：`player_box(read(0x04E0), read(0x04FC))` 与 capture L246-247 同址；正确 ✅。
- **保真**：不用 `hp or 0`；nil 表示未知、不冒充 0 伤害；报告 §4.1 明示 ✅。
- **非 bot 零影响**：`scenario_run.py:342-345` 仅 `plan.get("bot")` 时拼 `tas_bot.lua`；非 bot 生成 lua
  字节不变；title trace sha `62c9ea09…` 维持；改动只在 bot 源 ✅。
- **验收**：修复后短 bot-short 3000f + C2/C3/I5 重跑均须 `done.frame_errors==0`；未放宽 task-110 判据 ✅。

## 4. 未证项

- C3/I5 的 frame_errors 具体计数**未知**（旧 lua 无字段/无 DONE）；报告 §6 显式。
- bot-short B/C 在 3000f 窗口未触发，不能证明"场景无 B/C 路径"；报告显式。
- 修复后的行为/计数待 porter 实现卡验证；本卡未改代码/未跑 Mesen；报告显式。

## 5. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- **F120-1（低）**：bot-short 的证据路径应写 `analysis/review-capture/botshort/traces/t105-bot-short.*`；
  当前 `analysis/traces/t105-bot-short.*` 是另一次运行（0 error/无 frame_errors），直接查会误以为
  "59 不成立"。
- **F120-2（低）**：C3 的 DONE 无 `frame_errors` 字段，属 task-105 第一波 lua 的"未记录"；报告已说明，
  建议在 §2 表 C3 行明确标"字段不存在（non-legacy?）"，避免被误读为 0。
- **F120-3（低/建议）**：`tas_bot.lua` 三处修复后应加静态/短测回归（bot-short frame_errors=0 + 生成 lua
  行号断言），否则同类未定义变量可能回归；报告 §5 实现卡 D 已提，建议列入验收。

### 建议（P3）
- **F120-4**：修复后 C2/C3/I5 重跑时把 `frame_errors`/首错写入 P0-0 汇总，作为"干净 bot"字段；不要仅
  依赖 task-110 的 DONE 判定。
- **F120-5**：Fix A 的 `bot_degraded` 事件若实现，建议同时记录 `boss_type`/`frame`，便于区分"无 boss"
  与"boss HP 未知"。

### 时点注（重要）
- 本复核读取 `tools/tas_bot.lua`/生成 lua 时仍是 **task-115 取证时点的缺陷版**；复核完成后 `git status`
  显示 `tools/tas_bot.lua` 已被 platform-porter task-121 修改（并发修 Fix A/B/C），**不属 task-115 报告
  范围**，也不影响本文对"取证时点根因/计数/方案"的判断。行号以缺陷版为准；修复后行号/复核另见 task-121
  点验。

## 6. 命令/证据摘要

```text
C2 tas-stg07-relay.lua.log：142149 条 error
  inputPolled lua:186 nil hp = 3101（1260..120000，每30帧）
  inputPolled lua:275 global S = 25501（5326..119880）
  endFrame lua:742 global s = 113547（5299..120000）
  28602 inputPolled + 113547 endFrame = 142149 = DONE.frame_errors
bot-short review-capture：59 条 lua:186 nil hp（1260..3000 每30帧），DONE frame_errors=59
C3 done：无 frame_errors 字段；I5：无 done
gen lua：L185/186 boss_hits_to_kill；L275 S.shot_prev；L692 boss_hp；L742 player_box(s.px,s.py)
source：capture L243 / local S L299；BOT.decide L684/688；BOT.on_frame L738；local boss_type 仅 L708
non-bot：generate_lua 仅 plan.bot 拼 bot 源；title trace sha 62c9ea09… 不变
未跑 Mesen / 未改代码；docs-links 只读 PASS
```

## 7. 结论

**pass**：三条根因、142149 分解、bot-short 59、C3/I5 未记录判断、修复方向（含非 bot 零影响/保真）均
独立复算成立；F120-1..5 为低优先级证据路径/回归建议，不阻塞修复卡。修后按 task-110 的
`done.frame_errors==0` 点验即可。
