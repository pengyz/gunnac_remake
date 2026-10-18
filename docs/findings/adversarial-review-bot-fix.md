# 对抗复核：tas_bot.lua 3 处未定义变量修复（task-121）——adversarial-review-bot-fix

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用实施方自述**）
> 被审：`tools/tas_bot.lua` 工作树修复（HEAD `6ece96c` 上 M；`git diff --stat` **21 insertions / 4 deletions**，仅此 1 文件）+ 取证文档 task-121 实施状态注（已入库 `95173db` 的 `bot-frame-errors-rootcause.md` L234-249）。
> 纪律：**未跑 gates check/status/visual、未跑长场景**；Mesen 空闲时短跑（bot-short 3000f + title-menu 1080f）；本报告只写该文件、未 git add/commit。

---

## 0. VERDICT：**pass**（代码修复 0 阻塞；2 条低/信息 finding，均为文档/死分支，不影响行为；C2/C3/I5 长场景按任务约束未跑、未声称干净）

- **Fix A/B/C 逐行正确**：A 用 `s.boss_type`/`local hp=s.boss_hp`，nil ⇒ `S.boss_hits_left=nil`+`ev("bot_degraded")`，**无 `hp or 0` 代码**；B 仅新增一处前向 `local S`（L47，capture L247 之前）且原 `local S = {`（L303）改 `S = {`（无第二个 local S/无遮蔽）；C 改为 `player_box(read(0x04E0), read(0x04FC))`（与 capture L250-251 同源），on_frame 已无 `s.px/s.py`，**BOT.decide 的合法局部 `s`（L429/L593）未被误改**。
- **短场景实测 PASS**：`target/t105/bot-short.txt` 3000f → exit 0、trace_lines=3000、**DONE `frame_errors=0`（修复前 59）**、lua.log 0 error；events 0 个 `bot_degraded`；inputs 由修复前 2,941 行恢复为 **3,000 行**（正好补回原 59 个错误帧跳过记录的输入），trace/events 行数保持 3,000/59。修复前证据 `analysis/review-capture/botshort/traces` 59 次 L186 nil hp 仍在，可对照。
- **非 bot 零影响 PASS**：`scenario_run gen title-menu` 生成 lua sha256=`c5e6b9a0ff52ecb11f8c39f977d88c64b8706c31cbd37a5fd285eb45a6e22ebd`（与修复前一致，**字节中立**）；`run title-menu` 1080/1080/37/37、trace sha=`62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331`（不变）、exit 0；`test_scenario_run.py` **8/8 通过**（额外 `test_tas_bot.py` 16/16 通过）。
- **未引入新问题**：`S.boss_hits_left` 仍**无读取者**（仅初始化 L319 + 写入 L698/L701），Fix A 不改变任何决策；`bot_degraded` 在短跑未触发、events 行数不变，仓库无该 kind 的判据/门禁消费者；`evN` 400k cap 不受影响。
- **低风险 findings**：①取证文档实施状态注把 capture 坐标行写成 **L251-252**，修复后实际为 **L250-251**（注释行偏移；纯文档）；②Fix A 的 `hp==nil`/`bot_degraded` 分支在现有 `readByte` 语义（失败回 0，capture 的 `s.boss_hp` 恒为数值）下**实际不可达**（防御性保留可接受，但"显式 DEGRADED"不会发生）；③C2/C3/I5 长场景按纪律未跑，**不能从本次短测推断生产 bot 场景 frame_errors=0**（task-121 注也已写明待重跑）。

---

## 1. 逐行核 diff（A：工作树实体）

`git diff --stat`: `tools/tas_bot.lua | 25 +++--`（21 insertions, 4 deletions）；仅此文件 M；`git diff --check` exit 0。三处 hunk：

### 1.1 Fix B（前向 `local S`；L47 / L303）

```diff
@@ -41,6 +41,10 @@
 local PLAN = PLAN or error(...)
 local BOT = {}
+-- task-121 Fix B（取证 §1-B）：S 必须前向声明到 capture 之前。
+-- 原 `local S` 在 capture 之后（源 L299）⇒ capture 闭包按 Lua 词法解析成全局 S=nil ...
+local S
@@ -296,7 +300,7 @@
 -- ---------------- 策略状态
-local S = {
+S = {
```

- 代码静态核：`local S` **仅出现 1 处**（L47；L45-46 为注释）；`S = {` 仅 L303；`capture` 定义 L247 在 L47 之后 ⇒ 闭包捕获同一 local（`S.shot_prev` L271 不再落到全局）。
- 没有第二个 `local S` 遮蔽；没有改变 S 初始化内容。

### 1.2 Fix A（`BOT.decide` fire gate；L692-702）

```diff
-    if boss_type ~= 0 then
+    if s.boss_type ~= 0 then
       local cf, pen = ...
       local path = ...
       local operand = ...
-      S.boss_hits_left = boss_hits_to_kill(boss_hp, resolve_damage(path, operand))
+      local hp = s.boss_hp
+      if hp == nil then
+        S.boss_hits_left = nil
+        ev("bot_degraded", frame, { why = "boss_hp_nil", boss_type = s.boss_type })
+      else
+        S.boss_hits_left = boss_hits_to_kill(hp, resolve_damage(path, operand))
+      end
     end
```

- 静态核：BOT.decide 范围内已无裸 `boss_type`/`boss_hp`（只剩注释、`s.*`、事件字段名）；无 `hp or 0` 代码（仅注释明令禁止）；`local s = capture(read)`（L429）仍为合法局部 s。
- 语义：`s.boss_hp` 来自 capture L291（动态 boss 槽 `$0630+n` 或 `$064B`），与原注释意图一致；`S.boss_hits_left` 仍只写不读（§3）。

### 1.3 Fix C（`BOT.on_frame` shot_die；L749-756）

```diff
       local hit_player = boxes_overlap({ ... },
-                                       player_box(s.px, s.py))
+                                       -- task-121 Fix C（取证 §1-C）：on_frame 无本地 `s`（全局 nil）...
+                                       player_box(read(0x04E0), read(0x04FC)))
```

- 静态核：`BOT.on_frame` 范围（L~707-780）已无 `s.px/s.py` 代码（仅注释）；`player_box(read(0x04E0), read(0x04FC))` 与 capture L250-251 同源地址（原 capture 的 `s.px=read(0x04E0)`、`s.py=read(0x04FC)`）。
- **未误改** `BOT.decide` 内 L593 的 `player_box(s.px, s.py)`（合法局部 s）；`local s = capture(read)` L429 未改。
- 语义备注（info）：capture 的 `s.px/py` 是 inputPolled（上一帧终态）快照，on_frame 现读 endFrame 当前值；原 on_frame 无正确实现（全局 nil），此改动只影响 **events 归因字段**，不影响移动/射击决策，可接受。

## 2. 短场景实测（bot-short 3000f）

我独立跑（Mesen 空闲窗口，串行锁）：

```text
python tools/scenario_run.py run target/t105/bot-short.txt --timeout 300   # 输出覆盖到 analysis/review-botfix/bot
  frames: 3000 trace_lines: 3000 captures: 0 shots: 0 wall: 10.5s
  DONE: {"frames":3000,"frames_plan":3000,"frames_actual":3000,"overshoot":false,
         "trace_lines":3000,"captures":0,"shots":0,"bot_stop":false,
         "finalize_by":"endFrame","frame_errors":0}
  exit 0
```

| 证据 | 修复前（analysis/review-capture/botshort，task-109 复核跑） | 修复后（analysis/review-botfix/bot） |
|---|---|---|
| DONE frame_errors | **59** | **0** |
| lua.log error 行 | 59（L186 nil hp；frame 1260..3000 每 30 帧） | **0** |
| trace_lines | 3000 | 3000 |
| events.jsonl | 59 行（0 bot_degraded） | **59 行（0 bot_degraded）** |
| inputs.jsonl | 2,941 行（59 个错误帧整段 pcall 跳过记录/注入） | **3,000 行**（59 帧补回） |
| exit | 0（task-109 时） | 0 |

- **行为差异仅在原错误帧**：A 修复后不再抛错，因此 `BOT.decide` 返回、`setInput`/`record_input`/poke 正常执行 ⇒ inputs 2,941→3,000（精确 +59）；trace/events 行数不变（错误发生在 fire-gate 内原有事件之后，未见额外事件落盘；bot_degraded 未触发）。
- 报告不据此声称 C2/C3/I5 干净（§4）。

## 3. 非 bot 零影响 + 测试

| 项 | 结果 |
|---|---|
| `python tools/scenario_run.py gen title-menu` | lua sha256=`c5e6b9a0ff52ecb11f8c39f977d88c64b8706c31cbd37a5fd285eb45a6e22ebd`（与修复前一致；生成器仅 `plan["bot"]` 存在时拼 `tas_bot.lua`，非 bot 不含 bot 源） |
| `run title-menu` | 1080/1080/37/37、exit 0、trace sha256=`62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331`（与修复前及旧归档一致） |
| `test_scenario_run.py` | **8/8 passed**（pytest） |
| 额外 `test_tas_bot.py` | 16/16 passed（非卡面要求，旁证） |

- 生成 lua 中的 "BOT" 字样仅为 capture.lua 注释/占位（`BOT.wants_stop` 等），非 bot 源；sha 与修复前逐字节一致是零影响的硬证据。

## 4. 是否引入新问题 / 未覆盖项

| 项 | 结论 |
|---|---|
| `S.boss_hits_left` 读取者 | **仍无**（全文件仅 L319 初始化、L698/L701 写、注释）；Fix A 不改变任意决策，纯诊断/预留变量 |
| `ev("bot_degraded")` 影响 | 短跑 0 次；events 行数不变；`tas_bot.py`/`v5_posthoc.py` 只按已知 kind 过滤（未见 `bot_degraded` 消费），无门禁/判据读取 events 行数；`evN` 400k cap 无影响 |
| 分支可达性 | 现有 `readByte` 失败返回 0，capture 的 `s.boss_hp` 恒为数值 ⇒ `hp==nil`/bot_degraded 实际不可达（防御性保留；若未来 capture 允许 nil，语义已正确） |
| C2/C3/I5 长场景 | **未跑**（任务纪律：长场景/Mesen 交 l4-analyst）；C2 142149 旧产物仍 DEGRADED、不得进基线；生产场景 `frame_errors=0` 待重跑验证（task-121 注亦写待办） |
| 文档注行号 | task-121 注（rootcause L234-249）说 capture 坐标 "L251-252"，修复后实际 **L250-251**（插入 4 行导致偏移）；建议下次 doc-only 校正，不改结论 |
| Diff 范围 | 仅 `tools/tas_bot.lua`（+取证文档注已在 HEAD `95173db` 入库）；未改判定逻辑/门禁/capture.lua/gates；`diff --check` 干净 |

## 5. Findings 四分类

### 5.1 实现正确性（无）
- Fix A/B/C 与 task-115 取证方案逐字一致；无 `hp or 0`；无 local S 遮蔽；on_frame 无误伤 decide 局部 s。

### 5.2 文档/注释（低）
- **F1（低）**：`bot-frame-errors-rootcause.md` 实施状态注把 capture 坐标行写成 L251-252，实际 L250-251（L47 前向声明 +4 行所致）。纯文档，下次回填时校正。
- **F2（info）**：`hp==nil`/bot_degraded 分支当前不可达（capture 数值化）；若保留作防御，建议注释"当前 readByte 永不 nil，此分支为防御"，避免误以为 DEGRADED 会被触发。

### 5.3 行为风险（低/待验证）
- **F3（低，覆盖真实感）**：短 bot-short 仅覆盖 A（以及修复后的 inputs 恢复）；B/C 修复路径未在短场景触发（bot-short 3000f 无对应敌方子弹/弹消失窗口），C2/C3/I5 长场景按纪律未跑。**不能仅凭本卡声称生产 bot 场景 frame_errors=0**；需 task-121 后续长卡验证（input/events/trace 与旧 DEGRADED 产物对比）。
- 未发现修复引入非 bot 行为变化（gen sha/ trace sha 硬证据）。

### 5.4 越权/伪造（无）
- 写范围仅 `tools/tas_bot.lua` + 取证文档注（已入库）；未改判定逻辑/门禁/其他工具；本复核只新增本报告、未 git add/commit；短跑前 Mesen 空闲，未抢占 l4-analyst 长任务。

## 6. 命令与输出摘要

```text
# diff
git diff --stat tools/tas_bot.lua  -> 1 file changed, 21 insertions(+), 4 deletions(-)
git diff --check                    -> exit 0
git status --porcelain              -> 仅 M tools/tas_bot.lua（取证文档注已在 HEAD 95173db）

# 静态核
local S code lines: [L47]；S = { L303；capture 定义 L247
hp or 0 代码：无（仅注释）
on_frame s.px/s.py 代码：无（仅注释）；player_box(read(0x04E0),read(0x04FC)) L755
BOT.decide local s=capture(read) L429 / 合法 s.px,s.py L593 未改
boss_hits_left 引用：L319(init), L691(注释), L698/L701(写) —— 无读取

# bot-short 修复后
exit 0；trace_lines 3000；DONE frame_errors=0；lua.log errors 0；events 59 行/bot_degraded 0；inputs 3000 行
修复前证据（analysis/review-capture/botshort）：DONE frame_errors=59；lua.log 59 次 L186 nil hp；inputs 2941 行

# 非 bot
gen title-menu sha256 c5e6b9a0ff52ecb11f8c39f977d88c64b8706c31cbd37a5fd285eb45a6e22ebd
run title-menu 1080/1080/37/37 trace_sha 62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331 exit 0
python -m pytest tools/test_scenario_run.py -q -> 8 passed
python -m pytest tools/test_tas_bot.py -q    -> 16 passed (extra)
```

## 7. 复核边界

1. 未跑长场景/C2/C3/I5（任务纪律；Mesen 锁优先 l4-analyst）；短跑仅 bot-short 3000f + title-menu 1080f。
2. 未跑 `gates check/status/visual`（按 Lead 纪律）；docs-links 只做只读扫描 → PASS。
3. 未改被审文件/其他工作树文件；本报告唯一新增。
4. C2/C3/I5 的 frame_errors=0 属**未证/待长卡**，不把短场景结果外推为生产干净。

---

【2026-10-18】decomp-verifier，复核 task-121（HEAD `6ece96c`；tas_bot.lua +21/−4）；bot-short frame_errors 0/59 对照、title gen/trace 零影响、8/8 测试。
