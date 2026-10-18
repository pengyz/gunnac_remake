# P0-0 全量重采（GUI 模式，输入生效）：Batch A/B + 组 D 新证据（task-133）

> 日期：2026-10-18｜执行：l4-analyst｜状态：**Batch A 9 场景 / Batch B 3 场景全部合法且输入有效性断言通过（12/12）；两跑确定性 3/3**
> 根因背景（Lead 实测）：Mesen2 **testRunner（headless）模式 `emu.setInput` 不生效** → 本会话 P0-0 I1–I6、C1–C3、L5 B1 旧 trace **输入全未生效、语义无效**；
> GUI 默认模式（`tools/scenario_run.py`，Windows）输入生效。另有第二污染源：GUI 退出把 resume state 内嵌写 `RecentGames/gunnac_cn.rgd`，下次启动自动恢复 ⇒ 跨 run 状态污染；
> 已由 Lead 处置（`.rgd`/`gunnac_cn_11.mss` 备份为 `.bak-capturepollution`；`scenario_run.py` 每次 spawn 前自动清理 `[清场] 删除 Mesen2 resume 状态`）。
> 约束：不改代码/gates/基线/expected；不跑 gates check/status/visual；`analysis/`、`shots/` gitignored。

## 0. 结论摘要

| 项 | 值 |
|---|---|
| 场景数 | Batch A **9**（title-menu、gameplay-stage1、gs5-stg01-clear-anchor/full/c1/c2、stg03/stg06/stg08-timeline）+ Batch B **3**（tas-stg08b2-direct、tas-stg07-relay、tas-stg08-direct） |
| ledger 记录 / 有效 run | **29 条记录**（跨 3 个批次，含失败/重试/被替代尝试）；**最终有效 run=15**（=最后成功且断言通过、来自 resume 清理后的 clean 批次；title/gameplay/gs5-anchor 各两跑，其余 1 跑） |
| 合法性判据 | `rc=0 ∧ done!=null ∧ ¬timed_out ∧ trace_lines==done.trace_lines ∧ frame_errors=0` ⇒ **全部 PASS** |
| 输入有效性硬断言 | **12/12 场景 PASS**（watch 指标或语义断言；见 §2/§3） |
| 两跑确定性 | title-menu、gameplay-stage1、gs5-stg01-clear-anchor：**trace sha + shots 逐字节全同（3/3）** |
| 语义对照 | headless 基线 vs GUI：stg03/stg08 旧 expected 的 headless diff（1889–59685）→ GUI **diff=0 全命中**；stg08 f14600/16000/16600+ 直接等于旧 expected（见 §5） |
| 失败/重试 | **有历史失败（已解决，历史保留）**：gameplay v2 att#1 `rc=4294967295`（Lead kill Mesen；trace_lines=826、done=None）；title v2/v3 `57e25e3d`=resume 污染（被 clean 批次 `c4d6a83f` 替代）；c1/c2 v2 污染批次 trace `a3052466`/`afd2fc1f` 被替代；**最终选定 15 run 全部通过**（完整清单 §7） |
| settings | 批次前 sha `42bdec665af7cd38`（EmulationSpeed=1000）；批次后恢复 **100**，sha `f9c3ffc52deab8e7`；`AllowIoOsAccess=true` 保留；AutoSaveState/Rewind=false（不清场项） |
| **组 D 现状** | **2/6 已有来源**：#9 ending-4 / #10 ending-5（C1 `tas-stg08b2-direct` f33000–39000 五帧、C3 f33000–120000 均命中 `6ff345dc`，trigger=assisted）；**剩余 4/6 blocked:no-source**（dense 300f 采样仍零命中，§5.2）（fade-clear/ending-1/2/3，blob `14802bcf/19b88b1f/0f8e1169/f319f1fe`）；四联 #16/#19/#20/#21 与 #17/#18 的来源已找到（§5） |

## 1. 环境与方法

- 驱动：`python tools/scenario_run.py run <name> --timeout <T>`（Windows 默认 **GUI 模式**，`GUNNAC_HEADLESS` 未设置；
  Mesen 窗口出现属正常；串行锁仍生效）；`--timeout` 内为驱动 wall 超时，Mesen 自身 testRunner 超时参数不适用（GUI）。
- settings（用户 `C:\Users\Slaye\Documents\Mesen2\settings.json`）：批次前 `EmulationSpeed=1000`（提速）、
  `Preferences.EnableAutoSaveState=false`、`EnableRewind=false`、`Debug.ScriptWindow.AllowIoOsAccess=true`；
  每 spawn 前自动清理 `RecentGames/gunnac_cn.rgd`（`[清场] 删除 Mesen2 resume 状态`，Lead 修复）。
- 输入有效性断言（每场景，driver 自动执行）：trace `w` 中存在 `controller_held`/`controller_pressed` ⇒ 至少一次非 0；
  或 `game_action_mode` distinct>1；或 `stage_no` distinct>1；或 `player_x` range ≥4。
  无输入 watch 的场景（title/timeline）用 **shots 语义**兜底：新 shot vs 旧 expected 的 `diff_visual.compare`
  （256×240 真彩域 0 容差）diff 应=0，或较 headless 基线大幅下降（阈值仅用于"输入是否生效"判定，不是视觉门禁放宽）。
- headless 基线：`analysis/repro/recapture-gui/headless-baseline.json`（旧 headless shots sha + 各映射帧 headless diff）；
  driver/逐跑明细：`analysis/repro/recapture-gui/{ledger.jsonl,summary.json,settings-restore.json}`（均 gitignored）。

## 2. Batch A（P0-0 输入恢复，9 场景 / 11 跑）

| 场景 | 跑数 | frames/trace | **GUI trace sha（前 16）** | 对照：旧 headless trace sha（前 16） | controller_held 非零帧 | pressed 非零 | action_mode distinct | stage_no distinct | player_x range | shots 语义（GUI） | 两跑 | wall |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| title-menu | 2 | 1080/1080 | `c4d6a83fc8064d3b` | `62c9ea09580654ce`（无输入，旧） | —（无 watch） | — | — | — | — | expected `l4_boot-black` diff=**0** | **trace+shots 全同** | 4.8s×2 |
| gameplay-stage1 | 2 | 2400/2400 | `0fdd22821701b84a` | `94ec3a0718213d79`（旧） | **1049** | **195** | **4** | — | **243** | 3 帧 expected diff=**0/0/0** | **trace+shots 全同** | 144.5 / 132.9s |
| gs5-stg01-clear-anchor | 2 | 5000/5000 | `afd2fc1f1119d2cc` | — | **1094** | — | — | **2** | — | 28/28 shots 与 headless 不同（ratio 1.0） | **trace+shots 全同** | 15.3 / 15.2s |
| gs5-stg01-full | 1 | 15000/15000 | `d039dccd06d6699f` | — | **12601** | — | — | — | — | 25/25 与 headless 不同（1.0） | — | 39.8s |
| gs5-stg01-clear-c1 | 1 | 6000/6000 | `4c673738cfc7d941` | — | **1158** | — | — | **2** | — | 20/20 与 headless 不同（1.0） | — | 17.2s |
| gs5-stg01-clear-c2 | 1 | 5000/5000 | `b8c161e18f0f113f` | — | **1093** | — | — | **2** | — | 38/39 与 headless 不同（0.974） | — | 15.3s |
| stg03-timeline | 1 | 24000/24000 | `553310d0009ac61d` | `e9086b8e…`（旧） | —（无 watch） | — | — | — | — | **6 帧 expected diff=0**（原 headless 1889–59685） | — | 160.1s |
| stg06-timeline | 1 | 30000/30000 | `e219de85b58b3805` | `e7798edb…`（旧） | —（无 watch） | — | — | — | — | 107/108 shots 与 headless 不同（0.991） | — | 101.6s |
| stg08-timeline | 1 | 34000/34000 | `9ed2d790b451cd22` | `5f0fb228…`（旧） | —（无 watch） | — | — | — | — | **6 帧 expected diff=0**（原 headless 1668–24422） | — | 250.4s |

- `title-menu` 无输入 watch（菜单无输入），断言=语义 `l4_boot-black diff=0`；GUI trace sha `c4d6a83f…` 与 Lead 冷启动参考一致。
- `stg03-timeline` 6 帧：f3404→`l4_stg02-start`、f4600→`l4_clear-inject`、f5400→`l4_shop-window`、f5700→`l4_stg03-start`、
  f6500→`l4_combat-variety`、f12000→`l4_late-combat`，GUI diff 全 **0**。
- `stg08-timeline` 6 帧：f14600→`l4_stg07-start`、f16000→`l4_stg07-combat`、f16600/18000/22000/28000→四联，GUI diff 全 **0**。

## 3. Batch B（组 D 候选，3 场景 / 3 跑）

| 场景 | frames/trace | **GUI trace sha（前 16）** | 旧 headless trace sha（前 16） | controller_held 非零帧 | stage_no distinct | wall | 合法性/断言 |
|---|---|---|---|---|---|---|---|
| tas-stg08b2-direct（C1） | 40000/40000 | `9c06314c2f0cf0f1` | `486a157e…`（输入失效版） | **22511** | **3** | 278.6s | PASS |
| tas-stg07-relay（C2） | 120000/120000 | `27fa275a2d8a8625` | `e12e6ac4…`（输入失效版） | **30374** | **3** | 347.7s | PASS |
| tas-stg08-direct（C3） | 120000/120000 | `2c7962ec97c66107` | `d6944fe9…`（输入失效版） | **22512** | **3** | 344.7s | PASS |

> 三候选此前 headless 版输入未生效（`$18` 恒 0 / 无移动），本次 GUI 版 controller_held 单场景 2.2–3.0 万帧非零、
> `stage_no` 有 3 种取值 —— **输入确实生效**。组 D 三判据（blob/mode/trigger）用新 trace 重做后：
> **ending-4/5 已命中**（C1 f33000–39000 五帧、C3 f33000 起 59 帧 = `6ff345dc`）；**其余 4 对仍无源**（§5.1）。

## 4. 输入生效的判定依据（headless vs GUI 对照）

1. **trace sha 全面改变**：title `62c9ea09→c4d6a83f`、gameplay `94ec3a07→0fdd2282`、stg03 `e9086b8e→553310d0`、
   stg06 `e7798edb→e219de85`、stg08 `5f0fb228→9ed2d790`、C1/C2/C3 亦全变；
2. **shots 语义**：headless 下 stg03/stg08 与旧 expected 的 diff 为 1889–59685 像素；GUI 下对应帧 **diff=0**
   （Mesen 源截图 1:1 复现旧 expected）⇒ 旧 expected 是"输入正常"的真值，headless trace 是污染；
3. **watch 硬指标**：gameplay `controller_held` 1049 帧非零、`controller_pressed` 195、`action_mode` 4 种、
   player_x range=243（与 Lead 实测 128→243 一致）；gs5/C1-C3 的 held 非零帧 1093–30374；
4. **两跑确定性**：title/gameplay/gs5-anchor 两跑 trace sha + shots 逐字节相同（autosave/resume 清场后）⇒ 状态污染已消除。

## 5. 【关键新证据】旧 expected 来源命中（P0-0 后续重裁/组 D 用）

对 `stg08-timeline` GUI 版 **125 张 shots 全量扫描**（sha256 对照 `oracle/screens/expected/`）：

| expected | sha256（前 12） | 新 GUI stg08 命中帧 | 结论 |
|---|---|---|---|
| `l4_stg07-start` | `ddb322ac5702` | **f14600**（精确相等） | **#17 expected 来源确认** |
| `l4_stg07-combat` | `2824cb134987` | **f16000**（精确相等） | **#18 expected 来源确认** |
| `l4_stg06-late` / `l4_stg08-start` / `l4_stg08-combat` / `l4_stg08-late`（四联同 blob） | `62a8ef06c4b9` | **f16500–33900 共 64 帧**全部 `62a8ef06`（含 f16600/18000/22000/28000） | **#16/#19/#20/#21 expected 来源确认=同一冻结屏（OBS-1）**，四联同 blob 是真实同画面 |
| `l4_shop-window` | `988809e0b414` | f5400、f11400（与 stg03 相同屏） | 跨场景同屏（商店窗），可解释同 blob |
| `l4_stg02-start`（旧 expected=boot-black 复制品 `788d6984`） | `788d6984576b` | f3404、f16200 命中**黑屏**（boot-black 同图） | 旧 expected #14 **确为 boot-black 复制品**，不是关名屏 ⇒ 仍需重采 |
| `l4_ending-4` / `l4_ending-5`（同 blob） | `6ff345dc5c3f` | **C1 `tas-stg08b2-direct` f33000/f34500/f36000/f37500/f39000（5 帧全命中）**；**C3 `tas-stg08-direct` f33000–120000（59 帧全命中）** | **#9/#10 已有来源**（C1/C3 bot+assist，`trigger=injected/assisted`，不得冒充自然；来源帧域与原 expected 帧号不同，any-frame 命中） |
| **组 D 剩余 `l4_fade-clear`/`l4_ending-1/2/3`** | `14802bcf` / `19b88b1f` / `0f8e1169` / `f319f1fe` | **stg08-timeline 125 张 + C1 26 张 + C2 80 张 + C3 80 张全扫零命中**（C1 冻结窗 28500–33000 为 `f49e0149/11b703ce/3a7d95b3` 等，非 4 blob） | **#6/#7/#8/#11 仍 `blocked:no-source`**（f28500/30000/31500/33000 在 stg08 为 `62a8ef06` 冻结屏；f34500/36000 超 34000f；C1 捕获密度 1500 不足，见 §5.1） |

> **更正（两阶段）**：① stg08 GUI 重采后 `f16600–33900` 命中的是**四联 expected（`62a8ef06`，商店冻结屏）**，
> **不是** fade/ending 4 blob；② 进一步扫 **C1/C3** 后确认：**ending-4/5（`6ff345dc`）已在 C1 f33000–39000（5 帧）
> 与 C3 f33000–120000（59 帧）命中** ⇒ 组 D **2/6 有源**；**fade-clear/ending-1/2/3 的 4 blob 仍零命中** ⇒ 4/6 blocked。
> 因此"组 D 6 对全部无源"不成立，"组 D 来源全部已找到"也不成立；正确口径=**2 sourced（assisted）+4 blocked**。
> C1/C3 均为 bot+assist（注入三址+选关），按 §3.6 只能登记 `trigger=injected/assisted`，**不得冒充 natural**。
> 静态屏分组：`62a8ef06` 64 张（f16500–33900 商店冻结屏）、`d531d33a` 2 张（f7200/9300 商店菜单）、
> `988809e0` 2 张（f5400/11400 商店窗）、`788d6984` 2 张（f3404/16200 boot-black）——与 OBS-1「商店出店/冻结」一致。

### 5.1 C1 命中证据 + 剩余 4 blob 的 dense overlay 设计（待 Lead 批准，新 oracle 场景 = C4）

**C1 命中证据（`tas-stg08b2-direct`，26 张 shots）**：f33000/f34500/f36000/f37500/f39000 五帧全 = `6ff345dc5c3f454ee5545fd343387a906aa0373832997531d7dacd1ba8997091`
（= `l4_ending-4.png` 与 `l4_ending-5.png` 同 blob）；C3 `tas-stg08-direct` 从 f33000 起至 f120000 共 59 帧同 blob；
C2（relay）无命中（f36000 起为 `988809e0`/`d531d33a` 商店/冻结屏组）。

**剩余 4 blob**（`l4_fade-clear`=`14802bcf…`、`l4_ending-1`=`19b88b1f…`、`l4_ending-2`=`0f8e1169…`、`l4_ending-3`=`f319f1fe…`）：
C1 在 f28500–33000 的 1500f 间隔采样未见；fade 发生在 ending 前（ending-4/5 已从 f33000 开始）= 该 4500f 窗口捕获密度不足。

**建议新场景 `oracle/scenarios/tas-stg08b2-dense.txt`（C4 级，单独 commit + 证据说明；先设计后跑）**：
- 复制 `tas-stg08b2-direct.txt` **全部输入前缀/assist poke/bot/watch/probe_mem 不变**（行为等价，不引入新 poke）；
- 仅改 `capture_every 1500` → **`capture_every 300`**（覆盖 f27000–34500 窗口，约 25–30 帧/窗口；全 run 134 张，磁盘/时间成本 O(+100 张)）；
- 可选：再加显式 `capture 27000 fade-win-anchor` / `capture 30000 fade-mid` / `capture 31500 fade-late` / `capture 33000 ending-anchor`（4 张，便于人工定位）；
- 运行：`python tools/scenario_run.py run tas-stg08b2-dense --timeout 5400`（GUI，~5min@1000%）；判定：26+ 张 shots 中 any-frame 命中 4 blob 任一即来源找到；
- 若仍零命中 ⇒ 4 blob 不属于本链（可能来自其它路径/另一版本 run），组 D 剩余 4 对维持 blocked，并记录负证据。
- 预期（B 级）：fade-clear 为 ending 前过渡帧、ending-1/2/3 为结局多页，应在 f28500–34500 窗口；dense 采样可覆盖。

### 5.2 dense overlay `tas-stg08b2-dense` 结果（task-135；any-frame 4 blob 搜索）

场景已由 Lead 批准并提交（commit `3525019`；相对 `tas-stg08b2-direct` 仅 name/capture_every 1500→300/+4 显式锚点，输入 52 行逐字节不变）。GUI 运行（临时 speed=1000，跑完恢复 100）：

| 项 | 值 |
|---|---|
| 合法性 | `frames=40000 / trace_lines=40000 / done.trace_lines=40000 / frame_errors=0` ⇒ **PASS** |
| trace sha256 | `06089e78a3074bf4d00667b4032f3e37a5529fa47714a5ab2aa37f3b4ee2c755` |
| 输入断言 | controller_held 非零 **22512 帧**；stage_no distinct=[1,8,9] ⇒ **PASS** |
| shots / 时长 | **133 张**（capture_every 300）/ wall 330.7s |
| **any-frame 命中 4 blob** | **零命中**（`14802bcf` fade-clear / `19b88b1f` ending-1 / `0f8e1169` ending-2 / `f319f1fe` ending-3 全 133 张无匹配） |
| 对照：ending-4/5 | `6ff345dc` 命中 **f32100–39900（27 帧）** |
| settings | 临时 1000（sha `42bdec66…`）→ 恢复 **100**（sha `f9c3ffc5…`） |
| 机器可读 | `analysis/repro/recapture-gui/dense-run.json`（gitignored） |

> **组 D 现状（dense 后定稿）**：**2/6 sourced**（#9 ending-4 / #10 ending-5，C1/C3 assisted）；**4/6 `blocked:no-source`**
> （#6 fade-clear、#7/#8 ending-1/2、#11 ending-3，dense 300f 采样仍零命中）——负证据成立；4 blob 可能来自其它链/版本 run，不属本链。

### 5.3 P0-0-2 小 text 4 对×2 渲染（task-138 前可渲部分）

新 GUI trace 渲 `target/p0-0/render/g_run1/g_run2`（canonical env、39 诊断 env unset；4 对×2 跑）：

| pair | 两跑 png/idx | vs expected diff（像素） | 旧 headless actual diff | 新 actual sha（前 12） |
|---|---|---|---|---|
| gameplay-stage1_f01260 | ✅/✅ | **3742** | 10033 | `c39bbeffe098` |
| gameplay-stage1_f01500 | ✅/✅ | **3609** | 7219 | `7893d0a3475b` |
| gameplay-stage1_f02349 | ✅/✅ | **4382** | 6080 | `5737ae009af6` |
| title-menu_f00180_boot-black | ✅/✅ | **0**（像素级；sha 不同=PNG 编码） | 0 | `1b1940f30bf8` |

> 确定性 4/4；gameplay 三对差异较旧 headless 渲染下降（10033/7219/6080→3742/3609/4382），方向正确但仍 >0（P0-1 范围）。
> stg03/stg08 全对因 text 体积（stg03 text 1.27GB → 单次渲 536.6s）暂缓，等 task-138。产物：`target/p0-0/render/g-manifest-small.json`（gitignored）。

## 6. c1/c2 设计分叉（Lead 要求）

- trace sha 不同（`4c673738…` vs `b8c161e1…`），输入流分叉成立；
- shots 对照：**f2400 两场景同 sha `9ff234e6`**（gate-inject/clear-entry 同帧同图；Lead 原假设"f2400 不同"需更正），
  分叉从 **f3300** 起：c2 `shop-menu`=`d531d33a` vs c1 probe=`988809e0`，f3600 起亦不同（c1 继续关卡 vs c2 进商店）；
- 结论：c1/c2 是**不同输入设计**（c2 走商店出店分支），可用于后续对照，但不是"f2400 同图"的判据。

## 7. 失败/重试/替代 attempts 清单（如实；ledger 29 条，task-137 复核 P1 修正）

**口径**：`analysis/repro/recapture-gui/ledger.jsonl` 共 **29 条记录**（跨 3 个批次：v2 20:01 预 resume-修复、v3 20:11 仅 title 两跑后因 resume 污染停批、v4 21:14 clean 批次）。**最终有效 run=15** = "最后成功且断言通过"的 clean 批次运行；历史失败/被替代 run **不删除**：

| 场景 | attempts 总数 | 失败/被替代记录（ledger 实录） | 最终选择（trace sha 前 16） |
|---|---|---|---|
| title-menu | 8 | v2 att#1/#11/#2/#12：`assert_pass=false`（当时 driver 对无 watch 场景未做语义兜底）；其中 v2 att#2 / v3 att#2 trace=`57e25e3d`（**resume 污染**，被替代）；v3 att#1 `c4d6a83f` 与 att#2 `57e25e3d` 两跑不一致 → **停批** | v4 att#1/#2 均 `c4d6a83f`（两跑 sha 相同） |
| gameplay-stage1 | 5 | v2 att#1：`rc=4294967295`（**Lead kill Mesen**；trace_lines=826、done=None、run_valid=false）→ 同批 att#11 重跑有效；v2 批次结果因污染整体不采用 | v4 att#1/#2 均 `0fdd2282`（两跑同） |
| gs5-stg01-clear-anchor | 4 | 全部 run_valid（v2 批次污染整体不采用，非 run 失败） | v4 att#1/#2 均 `afd2fc1f` |
| gs5-stg01-full | 2 | v2/v4 均有效；v2 不采用（污染批次） | v4 `d039dccd` |
| gs5-stg01-clear-c1 | 2 | v2 trace=`a3052466`（**污染批次**，被替代） | v4 `4c673738` |
| gs5-stg01-clear-c2 | 2 | v2 trace=`afd2fc1f`（**与 anchor 同 sha = 污染串扰证据**，被替代） | v4 `b8c161e1` |
| stg03-timeline | 1 | — | `553310d0` |
| stg06-timeline | 1 | — | `e219de85` |
| stg08-timeline | 1 | — | `9ed2d790` |
| tas-stg08b2-direct（C1） | 1 | — | `9c06314c` |
| tas-stg07-relay（C2） | 1 | — | `27fa275a` |
| tas-stg08-direct（C3） | 1 | — | `2c7962ec` |

**选择规则**：①只采用 resume 清场钩子生效后的 clean 批次（v4）；②同场景多跑取**最后成功且 assert_pass 的 run**；③两跑场景要求 trace sha + shots 逐字节相同（v4 title/gameplay/gs5-anchor 均满足，3/3）。
**失败根因**：①v2 批次无 resume 清场（`.rgd` 内嵌 Savestate 污染）+ Lead kill Mesen（gameplay att#1 rc=-1）；②v3 批次 resume 污染仍在（title 第二跑 `57e25e3d`）→ 停批；③v4 clean 批次全部通过。
**F137-2（工具缺口注）**：ledger `resume_cleanup` 字段全 null、`output_tail` 无 `[清场]` 行——driver 未采集该字段（scenario_run 的 `[清场]` 打印在长期 stdout 早期，tail 截断丢失）；resume 清理**实际已生效**：复核方 GUI title×3 均同 sha `c4d6a83f` 独立验证；列为后续卡补记。
**备注**：`stg03/stg06/stg08/title` 无 `controller_held` watch，输入有效性由 shots 语义兜底；`c1/c2` 无共同显式帧，故 driver "common shots"=0，用 trace sha + 手工 f3300/f3600 对照补证。

## 8. settings 恢复记录（批次后）

| 项 | 值 |
|---|---|
| 批次前 settings sha256 | `42bdec665af7cd3840bd4184bf53d724862ad161f5f8f79abd649abf07771eb8`（EmulationSpeed=**1000**） |
| 批次后 settings sha256 | `f9c3ffc52deab8e74a76e43748a7d79fb505dbacbd4792bf2acc1c5ccacd7f41`（EmulationSpeed=**100**，本 driver 恢复） |
| `EmulationSpeed` 最终口径 | **各批次结束后由 Lead 统一恢复 100（最终核验时点=全部批次结束后）**；本报告批次 driver 结束已置 100，dense 批次临时 1000→100（`42bdec66→f9c3ffc5`）；当前确认 **speed=100** |
| `Debug.ScriptWindow.AllowIoOsAccess` | **true**（保留） |
| `Preferences.EnableAutoSaveState` / `EnableRewind` | false / false（保持 Lead 设置；待 Lead 最后统一恢复） |
| 备份 | `analysis/repro/recapture-gui/settings.json.bak-batch`（gitignored） |

## 9. 产物与复算

- 逐跑明细：`analysis/repro/recapture-gui/ledger.jsonl`（**29 条记录**=含失败/重试/被替代；最终有效 15 条，含 run_json/watch_assert/semantic_assert/shots_sha256）；
  汇总：`summary.json`；settings：`settings-restore.json`；headless 基线：`headless-baseline.json`（均 gitignored）。
- 复算输入有效性（示例）：
  ```bash
  python - <<'PY'
  import json
  for l in open("analysis/traces/gameplay-stage1.frames.jsonl",encoding="utf-8"):
      r=json.loads(l); w=r["w"]
      if w.get("controller_held"): print(r["f"], hex(w["controller_held"]), w.get("game_action_mode"), w.get("player_x")); break
  PY
  ```
- trace sha：`sha256sum analysis/traces/<scene>.frames.jsonl`（对照 §2/§3）。

## 10. 后续（Lead 已排）

1. **P0-0-2 重跑**：小 text 4 对×2 已完成（§5.3）；**stg03/stg08 全对等 task-138**（text replay 流式+早退）后渲；ending-4/5 现有源（assisted）可入可渲侧，剩余 4 对维持 blocked；
2. **组 D 定谳重做**（C1/C2/C3 新 trace，三判据）与 **22 对重裁**（新 actual vs expected，0 容差、不放宽）；
   命中项回填 mapping/`expected_source` 并更新手册 §3.6/§3.7/§7.x；
3. **L5 B1**：用新 GUI gs5-full/c1/c2/anchor 重跑 `s5_anchor_rederive.py`（预期 ALL MATCH）；
4. 旧 headless 结论（task-106/126/130 等）一律标注 **superseded by GUI recapture**（保留历史，dated 更正）。
