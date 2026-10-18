# bot 场景 frame_errors 根因取证（nil 变量 / boss_hits_to_kill）—— bot-frame-errors-rootcause

> 日期：2026-10-18｜角色：decomp-verifier（取证；**只读取证，不改代码/不跑 Mesen**）
> 背景：task-106 C2 `tas-stg07-relay` DONE 报 `frame_errors=142149 ⇒ DEGRADED`；task-109 复核 bot-short 见 59 次 `attempt to perform arithmetic on a nil value (local 'hp')`。本卡定位根因并给最小保真修复方案；实现由后续 porter 卡执行。
> 数据源（A 级）：`tools/tas_bot.lua`、`tools/scenario_run.py` bot 生成路径、`analysis/_gen/tas-stg07-relay.lua`、`analysis/traces/{tas-stg07-relay,tas-stg08-direct,t105-bot-short}.*`、`analysis/traces/.archive/*`、`analysis/review-capture/botshort/*`。**本卡未跑任何 Mesen 长场景**。

---

## 0. 结论速览

**根因不是单一 `boss_hp` 缺省，而是 `tas_bot.lua` 中 3 处未定义变量（A 级：代码+生成 lua+错误日志+计数）。**
C2 `frame_errors=142149` 精确分解（逐条日志聚合，恰好相等）：

| # | 触发函数 | 未定义变量 | 生成 lua 行 | C2 错误数 | 首帧 | 触发条件（A/B） |
|---|---|---|---|---|---|---|
| A | `BOT.decide`（fire gate 内） | `boss_type`、`boss_hp` | L692（源 L688；`boss_hits_to_kill` 在 L186） | **3,101** | **1260** | `fire and frame%30==0`；`boss_type` 全局 nil ⇒ `nil ~= 0` 恒真，随后 `boss_hits_to_kill(nil, …)` 算术 nil；**每 30 帧一次** |
| B | `capture(read)`（`BOT.decide` 首行调用） | `S` | L275（源 L271） | **25,501** | **5326** | 有敌方子弹（`$040E+slot≠0` 且 `$063E+slot==0x80`）时读 `S.shot_prev`；`capture` 在 `local S`（源 L299）之前定义 ⇒ 按 Lua 词法解析成全局 S=nil |
| C | `BOT.on_frame`（shot 消失归因） | `s` | L742（源 L738） | **113,547** | **5299** | 自机弹消失（`S.shot_prev` 有、`act` 无）时走 `player_box(s.px, s.py)`；on_frame 无 `s` 局部 ⇒ 全局 s=nil；此后几乎每帧弹消失 |
| **合计** | — | — | — | **142,149** | — | 与 C2 DONE `frame_errors=142149` 完全一致 |

- **每 30 帧一次只对应 A**；C2 总数主要是 C（每帧级 113,547），B 为 25,501——task 卡"frame_errors 主要来自 nil hp"的假设**不完整**，真实缺陷有三处。
- **C2 的 frame_errors=142149 是 inputPolled 28,602 次 + endFrame 113,547 次之和**（capture.lua 两个回调都累加 `frameErrors`；C2 日志逐条聚合无遗漏）。
- bot-short（3000f，最终 lua）只有 A：**59 次、1260..3000 每 30 帧**；B/C 未触发（3000f 窗口内未出现对应敌方子弹/弹消失路径，或未达到）。
- **C3/I5：不能从现有 DONE 量化**——C3 是 task-105 **第一波 lua**（无 `frameErrors` 字段/无逐帧 pcall），DONE 无 `frame_errors`、lua.log 无 error 行**只说明"未记录"不是"0"**；其 gen.lua 含完全相同三处缺陷、inputs/events 显示 bot 确实在跑。I5 旧跑无 DONE（旧终止 bug），旧 lua 同样无 `frame_errors`；**最终 lua 重试预期会暴露**，必须重跑实测。
- **不是"场景不该启用 bot"**：C2/C3/I5 都是 `bot stg01-clear from 1260`（生产采集场景，组 D/I5 依赖），bot 是设计内；这是共享 bot 代码缺陷，修代码而非关 bot。bot-short 是 task-105 临时验证场景（poke stage8 用同一政策），其 59 次是同一缺陷；不属生产场景但必须同修。

---

## 1. 根因逐条（A 级：源码 + 生成 lua + 运行日志）

### A. `BOT.decide`：`boss_type`/`boss_hp` 未定义（每 30 帧）

源码 `tools/tas_bot.lua`（`BOT.decide` 从 L424 开始，局部 `local s = capture(read)` L425）：

```lua
672  if fire and (frame % 30 == 0) then
...
683   -- boss 命中倒计时（boss_hits_to_kill 镜像：hp $064B / damage=2）
684   if boss_type ~= 0 then                      -- ← boss_type 在 decide 作用域未定义（全局 nil）
685     local cf, pen = weapon_hit_flags(S.weapon_class)
686     local path = auto_hit_path(cf, pen)
687     local operand = path == "Zp12" and read(0x0012) or read(0x0630 + 27)
688     S.boss_hits_left = boss_hits_to_kill(boss_hp, …)  -- ← boss_hp 未定义（全局 nil）
689   end
690 end
```

- 生成 lua（`analysis/_gen/tas-stg07-relay.lua`，与 run.json `generated_lua_sha256=a367f463…` 一致）：**L186 是 `boss_hits_to_kill` 函数体 `return math.ceil(hp / …)`；L692 是调用行**。
- `boss_type` 在 `BOT.decide` 内**没有任何局部定义**（只有 `s.boss_type` 在 capture L251、`local boss_type` 在 `BOT.on_frame` L708；另有 `local bt = s.boss and s.boss.type or 0` L443）。因此 `boss_type ~= 0` 恒真（nil≠0），每 30 帧必进该块；`boss_hp` 同样未定义 ⇒ `hp` nil ⇒ 算术异常。
- **每 30 帧一次**的直接来源 = L672 的 `frame % 30 == 0`。
- `S.boss_hits_left` 其余引用：仅 L315 初始化 nil、L688 写入；**当前无读取者**（A：grep 全文件），所以该块即使跳过/写 nil 也不改变决策，属诊断/预留字段；但未定义变量必须修，否则每 30 帧抛错、inputPolled 整帧输入注入被 pcall 跳过（见 §2）。
- C2 计数 3,101；bot-short 59（1260..3000，每 30 帧，正好 59 次）。

### B. `capture(read)` 里 `S.shot_prev`：`local S` 声明过晚（Lua 词法作用域）

源码 `tools/tas_bot.lua`：

```lua
243 local function capture(read)       -- 定义在 local S 之前
...
267   s.bullets = {}
268   for _, yslot in ipairs(SHOT_SLOTS) do
269     if read(0x040E + yslot) ~= 0 and read(0x063E + yslot) == 0x80 then
271       local prev = S.shot_prev and S.shot_prev[yslot] or nil  -- ← 解析为全局 S
...
298 -- ---------------- 策略状态
299 local S = { ... }                    -- 晚了 56 行：capture 闭包捕获不到这个 local
```

- 生成 lua L275 对应源 L271；错误消息 `attempt to index a nil value (global 'S')` 与"全局 S"一致。
- **触发条件**：`BOT.decide` 每帧调用 `capture(read)`（L425）；capture 扫描敌方弹（`$040E+yslot` active 且 `$063E+yslot==0x80`），一旦存在敌方子弹就执行 L271 ⇒ 全局 S=nil ⇒ 报错。C2 首帧 5326（此前无敌方子弹/或 `shot_prev` 路径未命中），之后 25,501 帧。
- C2 计数 25,501；bot-short 3000f 内未触发（无对应敌方子弹窗口）。

### C. `BOT.on_frame`：`s.px/s.py` 的 `s` 未定义（每帧弹消失）

源码 `tools/tas_bot.lua`（`BOT.on_frame` L695）：

```lua
709   for yslot, a in pairs(act) do ... end
717   for yslot, a in pairs(S.shot_prev) do
718     if not act[yslot] then
...
737       local hit_player = boxes_overlap({ ... }, 
738                                          player_box(s.px, s.py))  -- ← on_frame 无 s 局部
```

- `on_frame` 只有 `frame/read` 参数与局部 `act/boss_x/boss_y/boss_hp/boss_type/...`；`s` 是**未定义全局**⇒ nil ⇒ `attempt to index a nil value (global 's')`（生成 lua L742）。
- **触发条件**：`S.shot_prev` 中某弹在下一帧消失（`not act[yslot]`），进入 shot_die 归因并计算玩家框；bot 持续射击后 **几乎每帧都有弹消失** ⇒ C2 首帧 5299 后 113,547 次（占 C2 总数 80%）。
- C2 计数 113,547；bot-short 3000f 未触发（无弹消失窗口）。
- 注意：**line 589 的 `player_box(s.px,s.py)` 在 `BOT.decide` 内，s 是局部（L425），不是本缺陷**；只有 on_frame L738 是全局 s。

### 附带发现：`frame_errors` 的语义
`tools/scenario_capture.lua` 的 inputPolled（L214）与 endFrame（L266）都在 pcall 失败时 `frameErrors++`，因此 DONE 的 `frame_errors` 是**两类回调错误之和**，不能只按 task-109 的 nil hp 解释。C2 `28,602 + 113,547 = 142,149` 正好等于 DONE 值（A）。

## 2. 量化（A：run.json/DONE/日志逐条聚合；未跑新 Mesen）

| 场景 | 帧数 | DONE frame_errors | 首条错误 | A/B/C 分解 | 备注 |
|---|---|---|---|---|---|
| **C2 `tas-stg07-relay`** | 120,000（trace120000、80 shots、513.8s） | **142,149** | frame 1260：inputPolled L186 nil hp（A） | A=3,101；B=25,501（首5326）；C=113,547（首5299） | 日志 142,149 条 error（inputPolled 28,602 / endFrame 113,547），逐条聚合与 DONE 一致 |
| **C3 `tas-stg08-direct`** | 120,000（trace120000、80 shots、overshoot 120001） | **无字段** | 无记录 | **未知** | 第一波 lua 无 `frameErrors`/逐帧 pcall；DONE 无 frame_errors、lua.log 0 error 行=**未记录**；gen.lua 含同样三处缺陷；inputs 91,399 行/events 134,677 行显示 bot 活跃 ⇒ 最终 lua 重跑需实测 |
| **I5 `tas-stg08b2-direct`** | 40,000（旧跑 39,998 行、无 DONE） | **无字段（无 DONE）** | 旧 lua 无错误日志 | **未知** | 旧终止 bug 使 DONE 缺失；旧 gen.lua 同样含三处缺陷；最终 lua + 修复终止后重跑会记录 frame_errors（预期受影响，不能当 0） |
| **bot-short `t105-bot-short`** | 3,000 | **59**（我 task-109 复核跑，最终 lua） | frame 1260：inputPolled L186 nil hp | **A=59；B=0；C=0** | 每 30 帧 1260..3000；B/C 在此短窗口未触发；平台早期跑（旧 lua）无 frame_errors 不可比 |

> C2 错误帧范围：A 1260..120000；B 5326..119880（敌方弹存在窗）；C 5299..120000（弹消失窗）。三者均为 **inputPolled/endFrame 内可捕获异常**，task-105 pcall 保证 DONE/收尾仍落盘（trace 120000 完整），但 DONE 如实 `frame_errors` 标记 DEGRADED——**当前采集产物不能当干净基线**。

## 3. "缺省值缺失" vs "该场景不该启用 bot"

- **不是缺省值缺失**：三处都是**未定义变量的编程错误**，不是某场景少传参数：
  - `boss_type/boss_hp` 在 `BOT.decide` 未定义（应取 `s.boss_type`/`s.boss_hp`，二者已由 capture 读取）；
  - `S` 是函数定义早于 `local S` 的 Lua 词法作用域陷阱（B）；
  - `s` 在 `BOT.on_frame` 未定义（应读玩家坐标，capture 已用 `read(0x04E0)/read(0x04FC)`）。
- **不是"场景本不该启用 bot"**：C2/C3/I5 均 `bot stg01-clear from 1260`（八关参数化政策，生产采集/组 D/I5 依赖）；bot 是设计内组件，缺陷在共享 bot 代码。bot-short 是 task-105 临时 test fixture（stage8 poke + 同一政策），它暴露同一代码缺陷；是测试场景但不是"另一个政策无关"。
- **C3/I5 不是"干净"**：C3 第一波 lua 无错误记录能力，I5 旧跑无 DONE；两者 gen 代码与 C2 完全同源，**在拿到最终 lua 重跑 frame_errors=0 前不得声称可用**。
- 场景适用性另项（低优先）：bot-short 的 `assist-mode temp-task105-short-proof`/poke stage8 与 `stg01-clear` 组合只用于短测；生产 C2/C3/I5 是正常场景配置（C2 `assist-mode stage-relay-7plus`、C3 `stage-select-fallback`、I5 同系），无"应关闭 bot"的证据。

## 4. 最小保真修复方案（拟改文件/行/伪代码；不改判定逻辑/不放宽 frame_errors）

### 4.1 代码修复（`tools/tas_bot.lua`，仅 bot 源；建议一个 commit）

**Fix A（L684-689，`BOT.decide` fire gate）**：用 capture 已读的 `s.boss_type`/`s.boss_hp`，并对"无 boss/无 HP"显式 fail-closed：

```lua
-- 原 L684: if boss_type ~= 0 then
if s.boss_type ~= 0 then
  local cf, pen = weapon_hit_flags(S.weapon_class)
  local path = auto_hit_path(cf, pen)
  local operand = path == "Zp12" and read(0x0012) or read(0x0630 + 27)
  local hp = s.boss_hp
  if hp == nil then
    S.boss_hits_left = nil          -- 未知 ≠ 0 伤害；不发明
    ev("bot_degraded", frame, { why = "boss_hp_nil", boss_type = s.boss_type })
  else
    S.boss_hits_left = boss_hits_to_kill(hp, resolve_damage(path, operand))
  end
end
```

> `S.boss_hits_left` 现无读取者（§1-A），此块不影响移动/射击决策；若 porter 愿更小改动，可先只改成 `s.boss_type/s.boss_hp` 两处（消除错误），DEGRADED 事件为可选诊断。**不要**用 `hp or 0` 静默当 0 伤害——0 是"已击杀"语义，和"未知"不同；若必须 fail-closed，也应 `S.boss_hits_left = nil` + 显式事件。

**Fix B（L243 前 / L299）**：把 `S` 前向声明到 `capture` 之前（Lua local 若在闭包后才声明，闭包内是全局）：

```lua
-- 在 L243 capture 定义之前（例如 L43 local BOT 附近）加：
local S
-- 原 L299: local S = {   --> 改为：
S = {
```

> 这样 capture 内 L271 捕获到同一个 local S；S 初始化仍在原位，表内容不变。

**Fix C（L738，`BOT.on_frame`）**：玩家位置用与 capture 相同的读地址（A 级：capture L246-247 `s.px=read(0x04E0), s.py=read(0x04FC)`）：

```lua
-- 原 L737-738:
local hit_player = boxes_overlap({ ... }, player_box(s.px, s.py))
-- 改为：
local hit_player = boxes_overlap({ ... },
                                 player_box(read(0x04E0), read(0x04FC)))
```

> 不新增状态、不改变正常路径（当 s 本应等于该两地址读数时完全等价）。

### 4.2 非 bot 场景零影响论证

- `tools/scenario_run.py::generate_lua`（L342-345）**仅当 `plan["bot"]` 存在**才把 `tools/tas_bot.lua` 拼进生成 lua；非 bot 场景（title-menu 等）生成物根本不含 bot 源码 ⇒ 上述改动对非 bot **字节中立**；任务-109 已核 title trace sha `62c9ea09…` 不变。
- 修复只替换 bot 内部的未定义变量为**已存在**的捕获值/读地址；正常数据路径不变（无新分支进入决策逻辑）；不触碰 `scenario_capture.lua` 终止/frame_errors 判据、不触碰 gates/基线。
- Bot 场景在错误帧上行为会变化（原 pcall 整帧跳过 setInput/record/poke；修复后正常执行该帧 logic），这是**修复目标**，必须重跑 C2/C3/I5 并更新预期产物。

### 4.3 验收（不放宽）

- 修复后短测：`target/t105/bot-short.txt`（3000f）+ 生产 C2/C3/I5（按 Mesen 窗口）均须 `done.frame_errors == 0`；若仍 >0，保持 DEGRADED 并不得进基线。
- 判定逻辑不改：task-110 的 DONE/trace_lines 判据保留；`frame_errors` 只作为额外"干净度"门槛（见实现卡拆分）。
- DONE 字段不变（不新增/不改名；`frame_errors` 已有）。

## 5. 实现卡拆分建议（给 Lead/porter）

| 卡 | 内容 | 验收 |
|---|---|---|
| **A（bugfix，最小）** | 上述 Fix A/B/C；可加 `bot_degraded` 诊断事件 | bot-short 3000f `frame_errors=0`；C2/C3/I5 重跑 `frame_errors=0`（长卡，需 Mesen 窗口） |
| **B（验收/遥测）** | P0-0 验收加 `done.frame_errors==0`（或显式 DEGRADED 复核）；首次错误日志带 stack（capture.lua 现在只 log message） | 不改 task-110 判据；文档 §2.2 采集判据补一条 |
| **C（重跑/基线）** | 用修复后 bot + 最终 capture.lua 重跑 C2/C3/I5；记录 DONE frame_errors/首错、trace/captures；更新 p0-0 采集汇总/组 D 选帧 | 不得用修复前 C2 DEGRADED 产物；I5 重试不再被污染 |
| **D（可选诊断）** | `tools/scenario_run.py gen` 后静态检查 bot 源：`S.` 引用早于 `local S`、`BOT.decide` 内 `boss_type/boss_hp`、`BOT.on_frame` 内裸 `s` | 防同类回归；非必需 |

## 6. 证据分级 / 未证显式

- **A（硬证据）**：C2/bot-short 错误日志逐条计数、DONE/run.json `frame_errors`、源码行、生成 lua 行、run.json generated_lua_sha；C2 142149=28,602+113,547；bot-short 59。
- **B（可复现静态推理）**：三处触发条件由源码+Lua 词法作用域/read 地址推出；C2 首帧 A/B/C 时间顺序与 bot 战斗进程一致。
- **C/未证（显式）**：
  1. C3/I5 的 frame_errors **未量化**（C3 第一波 lua 无字段、I5 无 DONE）；"最终 lua 重跑会受影响"是 B 级推断，不是实测。
  2. bot-short 的 B/C 本窗口未触发，不能据此说该场景无 B/C 路径；只是 3000f 未命中。
  3. `S.boss_hits_left` 无读取者=当前代码事实；若未来接入决策，先补"未知/0"语义测试。
  4. 未改任何代码/未跑 Mesen；修复方案为拟案，实际行为由 porter 实现卡验证。

## 7. 复现命令（只读，A 级）

```text
# 1) 生成 lua 行号/上下文（run.json 引用的 gen sha=a367f463…）
python tools/scenario_run.py gen oracle/scenarios/tas-stg07-relay.txt   # 只写 analysis/_gen（gitignored）
# 实际已存：analysis/_gen/tas-stg07-relay.lua
G186:  return math.ceil(hp / math.max(damage, 1))       ; boss_hits_to_kill 函数体
G275:  local prev = S.shot_prev and S.shot_prev[yslot] or nil ; capture 内 global S
G692:  S.boss_hits_left = boss_hits_to_kill(boss_hp, …) ; decide 内 nil boss_hp/boss_type
G742:  player_box(s.px, s.py)                            ; on_frame 内 global s

# 2) 日志逐条聚合（C2）
python - <<'PY'
import re,pathlib,collections
L=pathlib.Path('analysis/traces/tas-stg07-relay.lua.log').read_text(encoding='utf-8',errors='replace').splitlines()
c=collections.Counter()
for l in L:
    m=re.search(r'(inputPolled|endFrame) error at frame (\d+): .*lua:(\d+): (.*)$',l)
    if m: c[(m.group(1),m.group(3),m.group(4)[:50])]+=1
print(c)  # (inputPolled,186,nil hp)=3101 ; (inputPolled,275,global S)=25501 ; (endFrame,742,global s)=113547
PY

# 3) DONE/run
analysis/traces/tas-stg07-relay.done: {"frames":120000,…,"frame_errors":142149}
analysis/traces/tas-stg07-relay.run.json: frames=120000 trace_lines=120000 done.frame_errors=142149 wall=513.8
```

## 8. 边界

1. 本卡只读；未改 `tools/tas_bot.lua`/`scenario_run.py`/`scenario_capture.lua`/文档；只新增本文件。
2. 未跑 Mesen（含短场景）；C2/C3/I5/bot-short 数字全部来自既有 run.json/DONE/lua.log/归档；C3/I5 缺口见 §6-C。
3. `frame_errors` 语义只按当前 capture.lua 代码（inputPolled+endFrame）解释；若 task-110 后续改判据，本卡结论不受影响（错误本身仍在 bot）。
4. C2 的 142149 是修复前（task-105 pcall 已捕获）数据；不得用作基线；修复后重跑前 C2/C3/I5 的 bot 产物标 DEGRADED。

---

【2026-10-18】decomp-verifier，只读取证 task-115；根因=3 处未定义变量（boss_type/boss_hp、capture 的 S 前向声明、on_frame 的 s），C2 142149 逐条分解；未跑 Mesen。

---

## 实施状态（2026-10-18 task-121 回填；不改上文取证结论）

- **Fix A 已实施**（`tools/tas_bot.lua` `BOT.decide` fire gate）：改 `s.boss_type` + `local hp = s.boss_hp`；
  `hp==nil` ⇒ `S.boss_hits_left=nil` + `ev("bot_degraded", frame, {why="boss_hp_nil", boss_type=s.boss_type})`；
  **未使用 `hp or 0`**（"未知"≠"0 伤害"）。
- **Fix B 已实施**：`local S` 前向声明到 `local BOT` 之后（capture 定义之前），原 `local S = {` 改 `S = {`
  ⇒ capture 闭包捕获同一 local（消除 global `S`）。
- **Fix C 已实施**：`BOT.on_frame` 的 shot_die 玩家框改 `player_box(read(0x04E0), read(0x04FC))`
  （与 capture L251-252 同源地址；`BOT.decide` 内 L593 的 `s.px/s.py` 为合法局部，未改）。
- **自证（本卡实跑，短场景）**：
  - `target/t105/bot-short.txt` 3000f：`done.frame_errors=0`（修复前 59）、trace_lines=3000、exit 0；
  - 非 bot 零影响：`python tools/scenario_run.py gen title-menu` 生成 lua sha 仍为 `c5e6b9a0…`（字节中立）；
    `run title-menu` 1080/1080/37/37、trace sha `62C9EA09580654CE691662C4D35EF98D0E722BDEAA88CCD893172A5D9389A331`（不变）、exit 0；
  - `tools/test_scenario_run.py` 8/8 通过。
- **未做/待办（交后续卡）**：C2/C3/I5 长场景重跑（需 Mesen 窗口）确认 `frame_errors=0`；
  修复前 C2 `142149` 产物继续标 DEGRADED、不得进基线；`S.boss_hits_left` 仍无读取者（未来接决策前补"未知/0"语义测试）。
