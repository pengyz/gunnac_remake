# cross-script-phase-gap —— GUI 模式跨脚本早期帧相位差：根因定位与修复（task-145）

> 日期：2026-10-18；实施：platform-porter；范围：`tools/scenario_capture.lua`（+43 行最终版）+ 本文。
> 结论先行：**根因 = Windows GUI 模式下 Mesen2 先启动模拟再装载 Lua 脚本，`inputPolled` 回调注册
> 相对开机的帧偏移随"脚本装载/解析延迟"（脚本大小、磁盘缓存、机器时序）变化** ⇒ 各场景 trace 的
> `f` 计数相对游戏开机存在 0–N 帧不等的相位差（同脚本同缓存条件下确定，跨脚本/跨批次不确定）。
> 修复：**GUI 晚装模式统一把场景起点对齐到游戏 `frameCount ≥ 7`**（headless 装载帧=1 ⇒ 完全不变）；
> 实测仅 **c1/c2** 两个历史 trace 需要替换（其余 13 run sha 不变），F2 复算 **3/3、SUMMARY 15/15**。

---

## 1. 现象与根因

### 1.1 现象（A 级：clean 批次 trace 逐帧对比）

clean 批次（v4，`p0-0-recapture-gui-run.md` §7）取到的 gs5 三场景 trace：

| 场景 | trace sha（clean v4） | 预兆帧：`countdown2f` 0→253 的 f | 相对 anchor 相位 |
|---|---|---|---|
| `gs5-stg01-clear-anchor` | `afd2fc1f1119d2cc` | f175 | 基准 |
| `gs5-stg01-clear-c1` | `4c673738cfc7d941` | f174 | 早 1 帧 |
| `gs5-stg01-clear-c2` | `b8c161e18f0f113f` | f172 | 早 3 帧 |

- anchor↔c2 首差 **f172**、anchor↔c1 首差 **f174**；共享输入段（f1..3200）出现 96 帧级早期差异；
  而同一次运行内同一脚本两跑 sha 相同（确定性成立）⇒ 不是随机噪声，而是**每个脚本各自的固定相位偏移**。
- 三场景输入前缀逐字节相同（仅 capture/watch 列表不同）⇒ 差异不能由输入设计解释。

### 1.2 根因（A 级：插桩实测）

Windows GUI 是 Mesen2 主路径（`scenario_run.py` `HEADLESS=False`；headless testRunner 的
`emu.setInput` 无效，见 F134-1）。GUI 启动顺序 = **ROM 先开机 → 再装载 Lua 脚本**，脚本装载时
游戏已跑到 `frameCount=3`：

```text
script load: game_frame=3          # 脚本装载时刻（GUI 模式，固定观测到 3）
phase align: late-load mode (script_load_game_frame=3), align_gate=7
scenario start: game_frame=7       # 首个被计入场景的 inputPolled 对应游戏帧
```

- `emu.addEventCallback("inputPolled")` 注册完成前，模拟器已继续跑帧；**注册延迟 ≈ 脚本解析+IO 时间**，
  随生成的 lua 大小/文件缓存/机器时序变化：
  - clean 批 cold 条件下：anchor（18.5KB）首计数帧≈游戏帧 7、c1（18.3KB）≈6、c2（18.2KB）≈4
    ⇒ lua `f` 相对开机相位差 0/1/3 帧 —— 与 §1.1 的 f175/174/172 完全一致（同一 game 帧事件在
    三个脚本里分别落在 lua f175/f174/f172）；
  - warm 缓存下三者装载延迟趋同（均 game_frame=7）⇒ 这正是"重新跑一遍就对齐/换台机器又漂移"的原因。
- 历史 contamination（resume state，`53d80cb` F134-1 第二源）会进一步改变起始状态，属同族问题；
  本卡的相位差在 resume 清场后**依然存在**（clean v4 c1/c2 即证据），故不是 resume 清理的重复。

### 1.3 为什么"同脚本两跑确定、跨脚本不齐"

注册延迟由**脚本字节数/解析时间**近似决定（cold 时最明显）且同一文件重复运行时机稳定 ⇒
同脚本两跑同 sha；不同脚本大小不同 ⇒ 各自固定偏移，跨脚本对不齐。

---

## 2. 修复形态（最终版：`tools/scenario_capture.lua` +43 行）

### 2.1 实现的规则

1. 脚本装载时记录 `script load: game_frame=<load_game_frame>`（`emu.getState().frameCount`）。
2. **仅当 `load_game_frame > 1`（GUI 晚装；headless testRunner 实测=1）** 判定为晚装模式：
   场景起点统一等到游戏 `frameCount ≥ PHASE_ALIGN_FRAME`（=7）才开始计数/注入输入；期间
   callback 只 `return`（不注入、不写 trace、不改游戏状态——游戏本身仍在自然开机运行）。
3. `scenario start: game_frame=<N> align_gate=<G>` 落 lua.log，作为每次运行可审计的相位证据。
4. headless（装载帧=1）`align_gate=0` ⇒ 行为/字节完全不变（实测 headless `title-menu` trace sha
   仍 `62c9ea09580654ce`）。

对齐后 GUI 三场景 `scenario start: game_frame=7` 一致（§3），场景内的输入/捕获/判定语义均不变
（只是把"装载期已跑的帧"排除在场景外；首个输入帧为 240，远晚于对齐点）。

### 2.2 候选方案对比（为何选 gate，而不是 reset/驱动启动序列）

| 候选 | 实测/结论 | 取舍 |
|---|---|---|
| ① 脚本装载后 `emu.reset()` | `pcall(emu.reset)` 在顶层返回 `ok=false err=This function cannot be called outside a callback`；若放进首个 callback 内 reset，会把所有 GUI trace 起点整体改写（13 个 clean run 全部失效）且引入 reset 语义风险 | 否决 |
| ② 对齐游戏帧门（本方案） | 实测只影响历史上相位偏移的 c1/c2（13 run sha 不变，见 §3） | **采用** |
| ③ 驱动注入固定启动序列 / 预装载 lua | GUI 无 `DoNotAutoStart` 类开关（strings 核验只有 `/DoNotSaveSettings`、`/LoadLastSession`）；headless 输入无效不可用 | 否决 |
| ④ 用 PLAN 计数帧=game frame 绝对编号 | 会整体平移全部 GUI trace（含 13 个 clean run） | 否决 |

`PHASE_ALIGN_FRAME=7` 取值依据：GUI 暖缓存/clean 批 anchor、title、gameplay、full 等首个计数帧
即为 7；取 7 可让"已经对齐的脚本零变化"，同时把 cold 时偏早注册的 c1/c2 拉回 7。

---

## 3. A/B 验收（最终版 capture.lua，GUI 模式，2026-10-18 23:0x–23:2x）

### 3.1 4+2 场景 trace sha（对照 clean 批次 v4）

| 场景 | clean v4 baseline sha | 修复后 sha | 变化 |
|---|---|---|---|
| title-menu | `c4d6a83fc8064d3b` | `c4d6a83fc8064d3b` | **不变** |
| gameplay-stage1 | `0fdd22821701b84a` | `0fdd22821701b84a` | **不变** |
| gs5-stg01-clear-anchor | `afd2fc1f1119d2cc` | `afd2fc1f1119d2cc` | **不变** |
| gs5-stg01-full | `d039dccd06d6699f` | `d039dccd06d6699f` | **不变** |
| gs5-stg01-clear-c1 | `4c673738cfc7d941` | **`a3052466eb54df1a`** | 变（修正相位） |
| gs5-stg01-clear-c2 | `b8c161e18f0f113f` | **`afd2fc1f1119d2cc`** | 变（修正后与 anchor 逐字节同） |

三场景最终运行日志（必须 3 个一致，实测均 7）：

```text
# anchor
script load: game_frame=3
phase align: late-load mode (script_load_game_frame=3), align_gate=7
scenario start: game_frame=7 align_gate=7
# c1
script load: game_frame=3
phase align: late-load mode (script_load_game_frame=3), align_gate=7
scenario start: game_frame=7 align_gate=7
# c2
script load: game_frame=3
phase align: late-load mode (script_load_game_frame=3), align_gate=7
scenario start: game_frame=7 align_gate=7
```

- headless 对照：`GUNNAC_HEADLESS=1 run title-menu` → `script load: game_frame=1`、
  `scenario start: game_frame=7 align_gate=0`（不对齐），trace sha 仍 `62c9ea09580654ce`（旧 headless 基准）。

### 3.2 F2 / 锚对拍复算（独立工具，非本次改动的实现方逻辑）

```text
$ python -X utf8 tools/s5_anchor_rederive.py
...
TRACE-DIFF: ALL MATCH (8/8)

── 复核 #74 更正口径复算（F1/F2）──
[ok] F2 c1↔c2 共享段（18 字段 × f1..3200）: f1..3200 mismatch=0；f3201..5000 mismatch=1800 帧
     （设计性分歧：c1 B 试探 vs c2 down 导航）
[ok] F2 c2↔anchor 全程（18 字段 × 5000f）: mismatch=0
[ok] F2 gameplay-stage1↔full（7 字段 × 2400f）: mismatch=0
SUMMARY: 15/15 通过（对拍 8 + 更正口径 7）→ ALL MATCH
```

⇒ F2 **3/3**、SUMMARY **15/15**；c2 与 anchor 现为**同一 trace sha**（逐字节），c1 仅从 f3201 起
按设计分叉（1800 帧），共享段 0 mismatch。

### 3.3 同脚本两跑确定性（不回归）

- anchor：23:14 与 23:20 两次运行均 `afd2fc1f…`（且 clean v4 两次也是该 sha）；
- c2：23:14 / 23:24 两次运行均 `afd2fc1f…`（其中 23:24 为最终版一次 `frame_errors=0` 干净跑）；
- c1：23:14 与最终版均 `a3052466…`；
- title/gameplay/full：与 clean v4 单/双跑 sha 完全一致（§3.1）。

### 3.4 运行合法性

各场景 `rc=0`、DONE `timed_out=false`、`trace_lines==done.trace_lines`、`frame_errors=0`
（c2 一次运行曾于 f4650 触发 Mesen 脚本 1s 执行时限 `frame_errors=1`，重跑即 0；该瞬时现象已由
task-110 判据标 DEGRADED，不会静默进基线）。

---

## 4. 对现有基线的影响面（Lead 口径）

- **P0-0 clean 批次 15 run 中 13 run 不受影响**（trace sha/shots 不变）：title/gameplay/anchor/full
  以及 stg03/stg06/stg08/其余 Bot run 的 GUI trace 生成路径不含相位变化（它们本来就在 game_frame=7 起步）。
- **仅 c1/c2 两条历史 trace 被替代（superseded 登记）**：

| 场景 | 旧（clean v4，作废） | 新（task-145 修正） |
|---|---|---|
| gs5-stg01-clear-c1 | `4c673738cfc7d941` | `a3052466eb54df1a` |
| gs5-stg01-clear-c2 | `b8c161e18f0f113f` | `afd2fc1f1119d2cc` |

- c1/c2 的 **shots 也随重跑刷新**（捕获帧对齐修正；旧 shot 值如需对照见
  `analysis/traces/.archive/` 与 `target/t145/` 本卡备份）；二者只被 **L5 F2/锚对拍消费者**使用，
  P0-0 渲染基准（expected/actual 22 对）不引用 c1/c2 trace，故**不需要重采 15 run、不需要重渲**。
- 曾出现的中间版 `c2=62ecf43b…` 是"gate=8"试验产物（比 anchor 晚 1 帧，仍不齐），**已废弃**，
  不作为交付；最终 = `afd2fc1f…`（=anchor）。

---

## 5. 复现命令（A 级）

```text
# 1) 最终版 A/B（GUI 默认；每场景 ~15–250s）
python tools/scenario_run.py run title-menu --timeout 600
python tools/scenario_run.py run gameplay-stage1 --timeout 600
python tools/scenario_run.py run gs5-stg01-clear-anchor --timeout 600
python tools/scenario_run.py run gs5-stg01-full --timeout 900
python tools/scenario_run.py run gs5-stg01-clear-c1 --timeout 600
python tools/scenario_run.py run gs5-stg01-clear-c2 --timeout 600
# 2) 每场景 lua.log 核相位（必须均为 7）：
grep -E 'script load|phase align|scenario start:' analysis/traces/gs5-stg01-clear-*.lua.log
# 3) F2/锚对拍复算：
python -X utf8 tools/s5_anchor_rederive.py     # 期望 SUMMARY 15/15 ALL MATCH
# 4) headless 不受影响：
GUNNAC_HEADLESS=1 python tools/scenario_run.py run title-menu   # sha=62c9ea09580654ce
```

---

## 6. 限制与后续

1. 相位对齐兜底条件：若极端慢机使首个 inputPolled 时 `game_frame > 7`，对齐门无法回退 ⇒
   `scenario start: game_frame=` 会 >7；**该行即运行相位证据**，出现 ≠7 应重跑/换负载窗口
   （driver 已把 lua.log 归档；后续卡可将此检查升级为 run 判据）。
2. 本修复不改变场景语义（不 reset、不重放、不改输入计划），只把"装载期帧"排除在场景外；
   `PHASE_ALIGN_FRAME=7` 是 clean 批次 GUI 模态值，若未来 Mesen 启动顺序/速度变化需重新标定。
3. c1/c2 旧 trace/shots 已在 `analysis/traces/.archive/` 留存（`run` 隔离归档机制），本卡另备份于
   `target/t145/`（gitignored）供对照；不删除历史。
