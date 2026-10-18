# 对抗复核：scenario_run GUI 默认模式修复（headless setInput 失效）（task-134）

> 日期：2026-10-18｜复核方：platform-verifier（verifier）｜被审：tools/scenario_run.py GUI 默认修复（task-134）
> 复核方式：静态代码三分支 + GUI/headless 短场景对照；**未跑 gates check/status/visual**；
> 未改代码/gates/expected/scenario。Mesen 串行锁已交还 l4-analyst（task-133）；本文件唯一写入。

## 0. VERDICT

**pass with findings（F134-1 中）**：GUI 模式确实恢复输入注入，静态分支正确；但 **title-menu trace
在 GUI 两次一致后、经一次 headless 再 GUI 出现新 sha**（c4d6a83f×2 → 62c9 headless → ffecf11c），
说明 **GUI trace 不是对"任意前后模式序列"都确定性**，l4 重采必须连续 GUI 并在每场景后做输入/确定性断言。
另：title-menu 场景**没有 watch controller_held/pressed**，任务卡要求在这条 trace 上核 controller 非 0
**无法核**；本复核用 gameplay-stage1 的 controller/action_mode/player_x 证据补足。

## 1. 静态代码核（通过）

`tools/scenario_run.py`（工作树 M）：
- **L69** `HEADLESS = os.environ.get("GUNNAC_HEADLESS","0") == "1"`：Windows 默认 **0（GUI）**；
  只有显式 `GUNNAC_HEADLESS=1` 才 headless ✅。
- **L71** `GUI_WINDOWS = (not HEADLESS) and os.name == "nt"` ✅。
- **L599-622** 三分支：
  - HEADLESS：`[MESEN, "--testRunner", f"--timeout={mesen_timeout_s}", lua, ROM]`（**仅此分支传 timeout**）✅；
  - **GUI_WINDOWS**：`[MESEN, ROM, lua, "/DoNotSaveSettings"]`，**无 Xvfb/DISPLAY** ✅；
  - POSIX：自行 Xvfb + `DISPLAY` + `/DoNotSaveSettings`（原逻辑保留）✅。
- **L528** `if (not HEADLESS) and os.name != "nt": pgrep ...`：pgrep 仅 POSIX GUI；Windows GUI/headless 跳过 ✅。
- Windows GUI 用 `Popen` 列表参数，路径含空格安全；`cwd=ROOT`；退出/超时共用 kill 逻辑 ✅。
- 未发现门禁/判定逻辑改动；仅工具分支 + 注释。

### 小建议（非阻塞）
- **F134-2（低）**：headless 分支未传 `/DoNotSaveSettings`；若 testRunner 会写 Mesen 设置，长期可能改变
  `EmulationSpeed`/窗口配置。建议 headless 也加（若 Mesen 接受）或在批次前后校验 settings sha。
- **F134-3（低）**：默认 GUI 对所有平台（POSIX 也 Xvfb GUI）；CI/无显示环境需显式 `GUNNAC_HEADLESS=1`，
  建议 README/知识库补一句默认模式与 override。

## 2. 动态短场景

### 2.1 title-menu GUI 两次
命令（默认 GUI，`EmulationSpeed=1000` 当前）：
```
python tools/scenario_run.py run title-menu --timeout 600   # 19:57:22, 4.0s
python tools/scenario_run.py run title-menu --timeout 600   # 19:57:48, 4.5s
```
- 两跑 `DONE`：1080/1080、frame_errors=0、timed_out=false；
- **trace sha256 前16 = `c4d6a83fc8064d3b` 两次相同** ✅；shots 37，`title-menu_f01080_probe.png`
  为**游戏标题画面**（sha `d1d12da27c08027d`，3012B），非开场故事 ✅。
- **F134-4（重要/中）**：`title-menu` trace 的 `w` 只有 `0320`/`zp`，**没有 controller_held/pressed**；
  因此任务卡"title menu controller 非 0"在该场景**不可观测**（不是失败，是断言落错场景）。建议
  改用 `gameplay-stage1` 断言，或另建带 `watch 0018/0016` 的短输入场景（不改本卡 scope）。

### 2.2 headless 对照
```
GUNNAC_HEADLESS=1 python tools/scenario_run.py run title-menu --timeout 600  # 3.5s
```
- trace sha 前16 = **`62c9ea09580654ce`**，与 GUI `c4d6a83f` **不同** ✅；DONE 1080/1080、errors0；
- 该 trace 同样无 controller watch；shots 回到**开场故事**（非标题/菜单），与 headless 输入失效解释一致 ✅；
- 这证明 GUI↔headless 行为差异来自模式，不是场景文件变化。

### 2.3 **F134-1（中）：跨模式后 GUI title 出现第三种 sha**
在 GUI c4d6×2、headless 62c9 之后，再跑一次默认 GUI：
```
python tools/scenario_run.py run title-menu --timeout 600   # 20:00 后
```
得到 trace sha 前16 = **`ffecf11c586bf92d`**（shots 仍标题画面 d1d12a，DONE 1080/1080）。
三次 GUI 结果：c4d6、c4d6、**ffecf11c**；说明 **GUI trace 会受前序 headless/持久状态影响**（或 GUI
启动时序/RNG），不是"任意顺序两跑必同"。task-133 重采若混用模式或复用前序状态，确定性断言可能假过/失败；
**建议 l4 必须连续 GUI 跑、每场景两跑并保存 sha，再与旧锚比较；若不一致先重置/冷启动 Mesen 状态**。

## 3. gameplay-stage1 GUI 输入证据（本复核补跑，Lead 已测可互证）

命令：`python tools/scenario_run.py run gameplay-stage1 --timeout 900`（GUI，151.2s，
DONE 2400/2400、frame_errors=0；**已跑完，锁交还 l4**）。

实测（`analysis/traces/gameplay-stage1.frames.jsonl`，sha16 `0fdd22821701b84a`）：
```
controller_held 非零 1049 帧（first f=241 val=16/start）
controller_pressed 非零 195 帧（first f=241）
game_action_mode 值 {0,7,110,124}，f1260=7（Combat）
player_x f1200=128 → f2349=243；player_y f1500=160
bomb_ammo f1260=3、f1500=2、f2349=4
shots/gameplay-stage1_f01260_stage-start.png sha 38365db2… = 地区1战斗 start 画面（非开场故事）
```
⇒ **GUI 输入确实生效**；与 Lead 报告的 `action_mode=7 / x 128→243 / bomb 3→4` 一致。DONE 本身
不证明输入（headless 也 DONE），但 controller/action/坐标/shots 四链证明。

## 4. settings 风险

- 当前 `%USERPROFILE%\Documents\Mesen2\settings.json`：`"EmulationSpeed": 1000`、
  `"AllowIoOsAccess": true`（与 task-133 提速口径一致）。
- `scenario_run.py` **不读取/不写 EmulationSpeed**；GUI 分支 `"/DoNotSaveSettings"` 防 Mesen 回写；
  但 **headless 分支没有该参数**（F134-2），且 GUI 退出/崩溃仍可能由 Mesen 自己保存。
- 本复核**未改 settings**；按 Lead 纪律，l4 批次结束须恢复 `EmulationSpeed=100` 并记录前后 sha；
  `AllowIoOsAccess=true` 保持。

## 5. 结论/建议

- **GUI 默认修复正确**：静态三分支/pgrep/`--timeout` 条件均正确；GUI title 两次 c4d6 一致；headless
  对照 62c9 不同；gameplay GUI 输入四链生效。
- **F134-1（中）**：GUI title 第三次（headless 之后）出现 ffecf11c，说明 GUI trace 非全序确定；
  l4 重采必须连续 GUI、双跑 sha、必要时冷启动，不要把跨模式混合结果当确定。
- **F134-4（中）**：title-menu 无 controller watch，任务卡的 controller 断言在该场景不可观测；
  请用 gameplay-stage1（本复核已给）或另设带 watch 的短场景。
- F134-2/3（低）：headless 设置回写风险/默认 GUI 文档。
- 已释放 Mesen 锁；未改代码/gates/expected/scenario；未跑 gates check/status/visual；本文件唯一写。

## 6. 命令/证据摘要

```text
static: HEADLESS default0 L69；GUI_WINDOWS L71；branches L599/604/610；pgrep guard L528；
        Windows GUI [Mesen.exe, ROM, lua, /DoNotSaveSettings]；timeout only headless
GUI title #1: sha c4d6a83f, 1080/1080, wall4.0s, shot f1080=d1d12a（标题）
GUI title #2: sha c4d6a83f, 1080/1080, wall4.5s
headless title: sha 62c9ea09, 1080/1080, wall3.5s, shots=开场
GUI title #3 after headless: sha ffecf11c, 1080/1080（F134-1）
GUI gameplay: sha 0fdd2282, 2400/2400, 151.2s；
  controller_held 1049 非零 / pressed 195 / action_mode f1260=7 / player_x 128→243
  shots f1260=38365db2（地区1 start）
settings: EmulationSpeed=1000, AllowIoOsAccess=true（未改）
Mesen 锁已交还 l4-analyst；未跑 gates check/status/visual
```

**verdict: pass（含 F134-1 中风险需 l4 重采时核；F134-4 title 断言落错场景）**。
