# 对抗复核：跨脚本相位对齐门修复（capture.lua +43 / A-B 影响面）（task-151）

> 日期：2026-10-18｜复核方：platform-verifier（verifier，证伪立场）｜被审：platform-porter task-145
> 被审：`tools/scenario_capture.lua`（工作树 **+43 行最终版**）+ `docs/findings/cross-script-phase-gap.md`（198 行）。
> 复核方式：读 diff + `.archive`/current trace sha + 独立跑 `s5_anchor_rederive.py` + 短 GUI title
> 复跑 + impact manifest 扫；**未跑 gates check/status/visual**；未改源码/expected。本文件唯一写入。

## 0. VERDICT

**pass**（1 条 P3 注释不一致；功能/证据全过）

- 代码 diff **恰好 +43**：诊断（script load/phase align/scenario start）+ **仅晚装模式 gate**
  （`load_game_frame>1` 时等 game frame≥7 再计数/注入；headless 装载帧=1 ⇒ gate0 完全不变）；
  无 +32/+42 实验中间代码残留（`git diff --stat` = 43 insertions only）。
- A/B：**不变 4**（title `c4d6a83f…`、gameplay `0fdd2282…`、anchor `afd2fc1f…`、full `d039dccd…`）；
  **变 2**（c1 `4c673738→a3052466`、c2 `b8c161e1→afd2fc1f`）；与 `.archive`/recapture 基准逐一对上。
- **F2 独立第三跑**：`s5_anchor_rederive.py` → **TRACE-DIFF 8/8**、F1 **4/4**、F2 **3/3**、
  **SUMMARY 15/15 ALL MATCH**；3 场景 `scenario start: game_frame=7 align_gate=7` 一致。
- **确定性/headless**：anchor/c1/c2 多跑 sha 稳定；headless title 仍 `62c9ea09…`（archive 实测）；
  c2 一次 f4650 `frame_errors=1`（archive 232245）当前重跑 frame_errors=0，报告如实。
- **影响面**：P0-0 render manifest 18 pairs **无 gs5/c1/c2 输入**；15 run 中 13 不变，仅 c1/c2 superseded；
  无需重采/重渲。capture.lua+报告可入库。

## 1. 代码正确性/残留

```
git diff --stat -- tools/scenario_capture.lua
 tools/scenario_capture.lua | 43 +++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 43 insertions(+)
```

- 脚本装载处：`pcall(emu.getState)` 记录 `script load: game_frame=3`（GUI）；headless=1。
- callback 首个 inputPolled：`phase_align_checked` 一次；
  - `load_game_frame>1` ⇒ `phase_align_gate=7`，log `phase align…align_gate=7`；
  - 期间每 callback `gfc<gate` 直接 `return`（不计数/不注入/不写 trace）；
  - `frame==1` 时 log `scenario start: game_frame=7 align_gate=7`。
- headless：load=1 ⇒ gate=0，callback 无 return，逻辑与旧版完全一致。
- **实验残留检查**：仅此 43 行；无 `PHASE_ALIGN_FRAME=8`/中间 gate、无 +32/+42 试验代码；
  `git status` 仅 capture.lua + 本报告/他人报告，无其它源码改动 ✅。
- **F151-1（P3，注释）**：顶部注释写 `load_game_frame > 0`，实现/报告用 `>1`（headless=1 不 gate）。
  行为正确，建议注释改 `>1` 与实现/§2.1 一致，避免下次误读。

## 2. A/B trace sha（current + archive）

当前 `analysis/traces` 实读：

| 场景 | 修复后 sha16 | 结论 |
|---|---|---|
| title-menu | `c4d6a83fc8064d3b` | 不变（我本轮 GUI 复跑亦 c4d6，log gate=7） |
| gameplay-stage1 | `0fdd22821701b84a` | 不变 |
| gs5-stg01-clear-anchor | `afd2fc1f1119d2cc` | 不变 |
| gs5-stg01-full | `d039dccd06d6699f` | 不变 |
| gs5-stg01-clear-c1 | **`a3052466eb54df1a`** | 变（旧 archive `4c673738…`） |
| gs5-stg01-clear-c2 | **`afd2fc1f1119d2cc`** | 变（旧 archive `b8c161e1…`；=anchor） |

- `.archive` 独立核：c1 旧 `4c673738…`（225522）、新 `a3052466…`（231013/231930）；
  c2 旧 `b8c161e1…`（225357）、新 `afd2fc1f…`（225921/230108/232111/232245）；
  anchor 多跑均 `afd2fc1f…`；**A/B 表可复算** ✅。
- **确定性**：anchor/c1/c2 多跑 archive sha 稳定；title/gameplay/full 与 clean v4 一致；
  **不回归**。
- **headless**：archive `title-menu-20261018-231801/232544` = `62c9ea09580654ce`；GUI 当前 c4d6；
  报告“headless 仍 62c9”成立（本轮未再占 Mesen 重跑 headless）。
- **c2 DEGRADED 个案**：archive `gs5-stg01-clear-c2-20261018-232245` done `frame_errors=1`；
  当前最终 c2 run `frame_errors=0`；报告 §3.4 如实披露，重跑干净。

## 3. F2 独立第三跑

```
$ python -X utf8 tools/s5_anchor_rederive.py
[ok] ... RAM 5/5, SHOT 3/3
TRACE-DIFF: ALL MATCH (8/8)
[ok] F1 bit4 过渡窗定界 ... [(913,1192)]
[ok] F1 过渡窗语义 ... f1193 bit4 清 + intro_ctr=64
[ok] F1 $CA 全程 0
[ok] F1 关末语义三联未出现
[ok] F2 c1↔c2 共享段: f1..3200 mismatch=0；f3201..5000 mismatch=1800 帧（设计性分歧）
[ok] F2 c2↔anchor: mismatch=0
[ok] F2 gameplay-stage1↔full: mismatch=0
SUMMARY: 15/15 通过（对拍 8 + 更正口径 7）→ ALL MATCH
```
- **8/8 + F1 4/4 + F2 3/3 + 15/15** 与报告一致；c2 与 anchor 逐字节同 ✅。
- **first_poll 日志**：anchor/c1/c2 当前 lua.log 均
  `script load: game_frame=3` → `phase align…align_gate=7` → `scenario start: game_frame=7 align_gate=7`；
  full 同 7；title/gameplay GUI 也 7；headless title 为 `game_frame=1…align_gate=0`（archive/历史）✅。

## 4. 影响面

- `target/p0-0/render/g-manifest-v2.json` 18 pairs 名称中 **无 `gs5*` / c1/c2**；P0-0 render 不引用
  c1/c2 输入；**无需重渲** ✅。
- clean recapture 15 run 中 **13 不变**；**仅 c1/c2 被替代**（superseded），与报告 §4 一致；
  L5 F2 消费者已用新 c1/c2 通过 15/15。
- 旧 c1/c2 trace/shots 保留在 `.archive/`/`target/t145/`，不删除历史；中间版 c2 `62ecf43b` 也在 archive
  （非交付代码），报告已声明废弃，无代码残留。

## 5. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- 无（数据/影响面均通过）。

### 建议（P3）
- **F151-1**：`scenario_capture.lua` 顶部注释 `load_game_frame > 0` 应改 `>1`（与实现/§2.1 对齐）。
- **F151-2**：`PHASE_ALIGN_FRAME=7` 是当前 GUI clean 模态值；若未来首 callback >7，gate 无法回退，
  建议后续把 `scenario start game_frame !=7` 升级为 run 判据（报告 §6 已记录，非本卡阻塞）。
- **F151-3**：archive 保留 c2 `62ecf43b` 中间版（历史），交付代码已无；如提交说明需列“已废弃”可引用
  report §4。

## 6. 命令/证据摘要

```text
diff: capture.lua +43 insertions only（无 +32/+42 残留）；headless gate=0
A/B: title c4d6a83f / gameplay 0fdd2282 / anchor afd2fc1f / full d039dccd 不变
     c1 4c673738→a3052466；c2 b8c161e1→afd2fc1f（=anchor）
F2: TRACE-DIFF ALL MATCH 8/8；F1 4/4；F2 3/3；SUMMARY 15/15
logs: c1/c2/anchor/full/title/gameplay scenario start game_frame=7 align_gate=7（GUI）
headless title archive=62c9ea09580654ce
c2 archive 232245 frame_errors=1；当前重跑=0
render manifest 18 pairs 无 gs5/c1/c2 → 不重渲
docs-links 只读 PASS；oracle/screens 0；未跑 gates check/status/visual；未改 expected
```

## 7. 结论

**pass**：+43 对齐门/诊断实现正确、无实验残留；A/B 4 不变/2 变与 archive 一致；F2 独立第三跑
8/8+F1 4/4+F2 3/3+15/15；first_poll=7 三场景一致；确定性/headless 不回归；影响面仅 c1/c2，
P0-0 render 不引用。仅 F151-1 注释小修建议，不阻塞入库。
