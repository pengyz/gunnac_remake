# L5 B1 锚证据重导 · run2（GUI 重采 trace 版）：TRACE-DIFF **8/8 ALL MATCH**；post-task-145 F2 **3/3 / 15/15**（dated 更新）

> 日期：2026-10-18｜执行：l4-analyst｜输入：**GUI 模式重采 trace**（task-133 v4 clean 批次：`gs5-stg01-clear-anchor`、`gs5-stg01-clear-c1`、`gs5-stg01-clear-c2`、`gs5-stg01-full`、`gameplay-stage1`；均 input-effective）；run1（headless 版，MISMATCH 3/8）保留为证据：`l5-b1-anchor-rederive-run1.md`。
> 本卡无需 Mesen：只读既有 trace/截图，跑 `tools/s5_anchor_rederive.py` + `tools/l5_probe_check.py`；不跑 gates check/status/visual；不改工具/expected。
> **【2026-10-18 dated 更新 / task-149】** 本报告 F2 分**三节**（均保留，不互相覆盖）：
> ① **§1 pre-task-145 快照（22:49）**：F2=1/3；trace 已覆盖、不可复算；
> ② **§1.2 中间 instrumentation 版（23:00）**：F2=1/3（c2 `first_poll=8` 残留 1 帧相位差）；trace 已被覆盖（c2=`62ecf43b…`）、不可复算；
> ③ **§1.1 post-fix 最终对齐版（trace mtime 23:10/23:13/23:11；23:17 我与 Lead 双跑复现）**：F2=**3/3**、SUMMARY=**15/15 ALL MATCH**（§1.1 全文输出）。

## 0. 结论摘要

| 项 | run2（GUI，本次） | run1（headless，对照） |
|---|---|---|
| `TRACE-DIFF`（8 锚） | **ALL MATCH (8/8)** ✅ | MISMATCH (3/8) |
| F1 更正口径（4 条） | **4/4 OK** ✅ | 1/4 OK |
| F2 更正口径（3 条） | **pre-fix（22:49 快照）1/3** → **post-fix（23:17 对齐版）3/3** ✅（c1↔c2 pre=0/post=1800；c2↔anchor=0；gameplay↔full=0） | 1/3 OK（当时是"全 0 假一致"） |
| SUMMARY | pre-fix **13/15** → **post-fix 15/15 ALL MATCH**（8 对拍 + F1 4 + F2 3） | 6/15 |
| A9 routine（`l5_probe_check.py`） | **natural 7/7 OK；A9: PASS**（工具面不变） | — |
| 判定 | **B1 锚证据已重建（8/8 MATCH）；post-fix F2 3/3（task-145 对齐修复后）**；pre-fix 失败留档（§4.1） | BLOCKED |

> **时点口径**：§1/§3/§4 的 F2 失败数字（96/1900/首差 f172）是 **22:49 pre-task-145 快照**；§1.1/§4.1 的 post-fix 数字（0/1800/0）才是**当前最终 trace**。二者都保留、不互相覆盖。

**关键区分**：run1 的失败根因是 **headless `emu.setInput` 不生效**（全 0 假一致 + 3/8 锚 miss）；run2 输入已生效（8/8 锚命中、stage 推进/商店链全部出现），**F2 剩余 2 条失败是"不同 Lua 脚本 run 之间的早期帧非确定性"**，不是输入未注入（**这是 pre-fix 历史时态**；task-145 对齐修复后 post-fix **F2=3/3、SUMMARY=15/15**，见 §1.1/§4.1）。

## 1. 命令与输出（**22:49 pre-task-145 快照**；trace 已被覆盖、不可复算）

```powershell
# 1) B1 锚重导 + F1/F2 更正口径（GUI trace；UTF-8 控制台避免 F2 的 ↔ 在 GBK 下报错）
python -X utf8 tools/s5_anchor_rederive.py
# 2) A9 routine（natural 面 7/7；工具面不变）
python -X utf8 tools/l5_probe_check.py
```

### `s5_anchor_rederive.py` 输出（本机实测，2026-10-18 23:0x）

```
[ok] RAM stg01-clear f2401: flags27=0x80 ∧ countdown2f=50（f2400 flags27=0）
[ok] RAM flyout-vector f2794: countdown2f=0 ∧ vec41c=164 ∧ vec438=13（f2793=1，判定帧唯一）
[ok] RAM gate300-open f3066: gate300=0x10 ∧ flags26=0x00（bit4 清）
[ok] RAM inc-180 f3096: stage_no 1→2（INC $180 写帧）
[ok] RAM stg02-shop f3116: stage_no=2 ∧ player_state=0（$9BC3 init）
[ok] SHOT stg02-exit f3400: capture tag=exit-fade4 记录 1 条 + 截图/RAM dump 实存
[ok] SHOT stg02-start f3450: capture tag=stg02-start 记录 1 条 + 截图/RAM dump 实存
[ok] SHOT stg02-combat f3900: capture tag=stg02-combat 记录 1 条 + 截图/RAM dump 实存

TRACE-DIFF: ALL MATCH (8/8)

── 复核 #74 更正口径复算（F1/F2）──
[ok] F1 bit4 过渡窗定界（Run A）: flags26 bit4 置位窗 = [(913, 1192)]（f913..1192=菜单→开 stage 过渡窗；非「全程 0」）
[ok] F1 过渡窗语义（开 stage）: f1193 bit4 清 + intro_ctr=64（关始引入；窗口期间 countdown2f=199 非倒数环）
[ok] F1 $CA 全程 0（此条原断言成立）: trig_ca 非零帧数 = 0
[ok] F1 关末语义三联未出现（Run A）: 倒数环（50→0·8f DEC）=无；INC $180 转移=无（flags27 腿：轻量面未 watch $27，经同前缀 anchor run f913-1192 同窗 flags27=0 旁证）
[FAIL] F2 c1↔c2 共享段（18 字段 × f1..3200）: f1..3200 mismatch=96；f3201..5000 mismatch=1800 帧（设计性分歧：c1 B 试探 vs c2 down 导航）
[FAIL] F2 c2↔anchor 全程（18 字段 × 5000f）: mismatch=1900
[ok] F2 gameplay-stage1↔full（7 字段 × 2400f）: mismatch=0（zp/iram blob 按场景 watch 映射解：wram=$6000-7FFF SRAM 区非 $0000 基址）
SUMMARY: 13/15 通过（对拍 8 + 更正口径 7）→ MISMATCH
```

### `l5_probe_check.py` 输出（摘）

```
artifacts: 12/12 present in analysis/traces
passA: natural_ok=7 (expect 7) ok-injected=4 unknown=33 mismatch=0 provenance-note=0 | diff_l5 rc=1 (1=expected: unknown counts as bad)
passB: ok-injected=27 unknown=17 mismatch=0 | diff_l5 rc=1
L5-DIFF natural 7/7
A9: PASS (local probe5 natural face OK; real-machine Mesen2 face still needs env)
```

### 1.1 post-fix 复测（2026-10-18 23:10–23:13 最终对齐版 trace；Lead 与我双跑复现 23:17；可复算）

**trace mtime**：anchor 23:10、c1 23:11、c2 23:13（full 23:17 收尾）；**修复落盘特征**（`analysis/traces/*.lua.log`）：三场景均 `script load: game_frame=3`、`phase align: late-load mode … align_gate=7`、
`scenario start: game_frame=7 align_gate=7` ⇒ **first inputPolled 相位统一=7**（pre-fix 中间版 c2=8）。

```powershell
python -X utf8 tools/s5_anchor_rederive.py
```
关键输出（全文）：
```
TRACE-DIFF: ALL MATCH (8/8)
[ok] F1 bit4 过渡窗定界: [(913, 1192)]
[ok] F1 过渡窗语义: f1193 bit4 清 + intro_ctr=64
[ok] F1 $CA 全程 0: trig_ca 非零帧数 = 0
[ok] F1 关末语义三联未出现: 无倒数环、无 INC $180 转移
[ok] F2 c1↔c2 共享段（18 字段 × f1..3200）: f1..3200 mismatch=0；f3201..5000 mismatch=1800 帧（设计性分歧）
[ok] F2 c2↔anchor 全程（18 字段 × 5000f）: mismatch=0
[ok] F2 gameplay-stage1↔full（7 字段 × 2400f）: mismatch=0
SUMMARY: 15/15 通过（对拍 8 + 更正口径 7）→ ALL MATCH
```

**最终 trace sha（16）**：anchor `afd2fc1f1119d2cc`、c1 `a3052466eb54df1a`、c2 **`afd2fc1f1119d2cc`**（与 anchor 同）、full `d039dccd06d6699f`、gameplay-stage1 `0fdd22821701b84a`。
**superseded 链（c1/c2）**：
- c1：recapture 基准 `4c673738cfc7d941`（task-133 v4 pre-fix）→ **`a3052466eb54df1a`（task-145 对齐最终版）**；
- c2：recapture 基准 `b8c161e18f0f113f` → 中间 instrumentation 版 `62ecf43b9fba2c36` → **`afd2fc1f1119d2cc`（对齐最终版，与 anchor 同 sha）**；
- anchor：`afd2fc1f1119d2cc`（前后不变）；full：`d039dccd06d6699f`（不变）。

### 1.2 中间 instrumentation 版快照（2026-10-18 23:00；F2=1/3；trace 已被覆盖、不可复算）

- 状态：`scenario_capture.lua` 已注入 `script load: game_frame` / `first inputPolled` 取证（experimental），但**尚未对齐**；
  trace：anchor `afd2fc1f1119d2cc`（first_poll=7）、c1 `a3052466eb54df1a`（first_poll=7）、**c2 `62ecf43b9fba2c36`（first_poll=8）**；
- 实测：`TRACE-DIFF 8/8`、F1 4/4，但 **F2 1/3**（c1↔c2 f1..3200 mismatch=**6 帧**：f174/202/204/205/211/932；c2↔anchor mismatch=**6 帧**；f3201..5000=1800 设计分歧）；
  ⇒ 定位到 **1 帧相位差**（c2 first_poll=8 vs 7）是根因；随后 task-145 以 `align_gate=7` 对齐（§1.1 结果 3/3、15/15）。
- 该版 trace 已被对齐版覆盖（c2 `62ecf43b…` → `afd2fc1f…`），仅作 dated 历史保留。

## 2. 8 锚结果（run2 全部命中 vs run1 3/8）

| # | 锚 | run2 | run1 |
|---:|---|---|---|
| 1 | RAM stg01-clear f2401（flags27=0x80 ∧ countdown2f=50） | ✅ | ✅ |
| 2 | RAM flyout-vector f2794（countdown2f=0 ∧ vec41c=164 ∧ vec438=13） | ✅ | ❌（vec41c=0） |
| 3 | RAM gate300-open f3066（gate300=0x10 ∧ flags26 bit4 清） | ✅ | ❌ |
| 4 | RAM inc-180 f3096（stage_no 1→2） | ✅ | ❌（1→1） |
| 5 | RAM stg02-shop f3116（stage_no=2 ∧ player_state=0） | ✅ | ❌ |
| 6 | SHOT stg02-exit f3400（capture tag exit-fade4 + 截图/RAM dump 实存） | ✅ | ✅*（截图实存但语义=开场故事） |
| 7 | SHOT stg02-start f3450（capture tag stg02-start） | ✅ | ✅*（同上） |
| 8 | SHOT stg02-combat f3900（capture tag stg02-combat） | ✅ | ✅*（同上） |

> \* run1 中截图"实存"仅文件层命中，内容为开场故事（输入未生效），故整体判 MISMATCH；run2 语义与锚一致。

## 3. F1/F2 逐条（run2；**pre-fix 22:49 快照**；post-fix 见 §1.1）

| 项 | 判据 | run2 结果 | 说明 |
|---|---|---|---|
| F1-a | bit4 过渡窗定界（Run A） | ✅ `[(913,1192)]` | 非「全程 0」；与预期窗一致 |
| F1-b | 过渡窗语义（f1193 bit4 清 + intro_ctr=64） | ✅ | countdown2f=199 非倒数环 |
| F1-c | `$CA` 全程 0 | ✅ trig_ca 非零帧=0 | 原断言成立 |
| F1-d | 关末语义三联未出现（Run A） | ✅ | 无倒数环、无 INC $180 转移 |
| F2-a | c1↔c2 共享段 f1..3200 = 0 差异 | ❌ **96 帧差异** | 首差 f172/f173（`countdown2f` 0 vs 253）、f200（`player_state` 0 vs 124）；f3201..5000 = 1800 帧（=设计性分歧全段）✔ |
| F2-b | c2↔anchor 全程 5000f = 0 差异 | ❌ **1900 帧差异** | 首差 f172（`countdown2f` 253 vs 0）；anchor 与 c2 **功能输入行逐字节相同**（仅 capture 列表不同） |
| F2-c | gameplay-stage1↔full 7 字段×2400f = 0 差异 | ✅ mismatch=0 | watch 映射解已按 SRAM 区解 |

> **post-fix 更新（23:17）**：F2-a/b 已由 task-145 对齐修复转为 ✅（f1..3200 mismatch=0；c2↔anchor=0），F2=3/3、SUMMARY 15/15（§1.1）；
> 上表 ❌ 行是 **pre-fix 历史快照**，保留不删。

## 4. F2 失败根因分析（**pre-fix 快照**，如实；post-fix 结论见 §4.1）

1. **不是输入失效**：run2 的 8/8 锚（含 `stage_no 1→2`、商店 init、关名/战斗截图）全部命中；相对 run1（全 0 假一致）输入已生效。
2. **是跨脚本 run 的早期帧非确定性**：
   - `anchor` 与 `c2` 的**功能输入行完全相同**（`diff` 只有 capture 列表/name/注释差异），但两者 trace 从 **f172** 起出现 `countdown2f`/`player_state` 等差异，至 1900 帧；
   - `c1` 与 `anchor` 在 f1..3200 输入同源（c1 仅 f3200+ 加 B 试探），trace 也有 **1806 帧差异**（首差 f174）；
   - 同一脚本两跑（anchor att#1/att#2）trace sha 相同（`afd2fc1f…`）⇒ 单脚本内确定性成立；**不同 Lua 脚本（capture/watch 列表不同）之间出现早期 intro 相位差**。
3. 可疑机制（待工程验证，不当结论）：GUI 实时模式下脚本回调/截图（capture）与 `emu.setInput` 的帧对齐可能受脚本执行时序影响，导致 intro 段输入相位/帧计数 ±N；或脚本生成差异（capture/watch 长度）改变 inputPolled 时序。**需 platform-porter 侧最小实验**：同输入前缀 + 仅改 capture 列表的 A/B 两场景，比对早期 trace。
4. **影响面**：B1 的 8 锚与 F1 不受影响（目标帧/语义仍命中）；F2 的两条"跨脚本 0 差异"判据在 GUI 模式下暂不成立，**需修 scenario_run/脚本时序或按实际口径修订 F2**（新卡；本报告只记录负证据）。
5. **不宣称**：本报告不宣称"F1/F2 全通过"；B1 主判据（8/8 ALL MATCH）成立，但 L5 的跨脚本确定性面仍待收口。

### 4.1 post-fix 结论（task-145）：根因=脚本装载相位，已对齐

- **根因确认**（instrumentation 证据）：pre-fix 三个脚本的 `first inputPolled game_frame` 不同——anchor/c1=7、**c2=8**（`script load: game_frame=3` 相同）；
  1 帧相位差导致 intro 段 `countdown2f`/`player_state` 少数帧不一致（中间版实测 c1↔c2 pre=6 帧、c2↔anchor=6 帧，首差 f174/202/204/205/211/932）。
- **修复**：`scenario_capture.lua` 增加 late-load 对齐（`align_gate=7`，脚本装载后等到 game_frame≥gate 才起算/注入输入）⇒ 三场景 `scenario start: game_frame=7` 一致。
- **复测**：F2 3/3、SUMMARY 15/15 ALL MATCH（§1.1）；title-menu trace sha `c4d6a83f…` 不变（无输入场景）；gameplay/anchor/full 亦不变（A/B 决策：**仅 c1/c2 变化**，不重采 15 run）。
- **影响面**（Lead 已批）：仅 L5 F2 消费者（c1/c2 新 sha 见 §1.1）；P0-0 冻结基准（15 run）保持有效；旧 sha 作 dated 历史保留。

## 5. run1 vs run2 对照（证据保留）

| 项 | run1（headless，19:58 文档） | run2（GUI，本报告） |
|---|---|---|
| TRACE-DIFF | MISMATCH 3/8 | **ALL MATCH 8/8** |
| F1 | 1/4 | **4/4** |
| F2 | 1/3（全 0 假一致） | **post-fix 3/3**（pre-fix 1/3，真实跨脚本差异） |
| SUMMARY | 6/15 | **post-fix 15/15**（pre-fix 13/15） |
| 根因 | `emu.setInput` 不生效（GUI 前） | 输入生效；F2 跨脚本相位差已由 task-145 对齐修复（post-fix 3/3） |
| 证据 | `l5-b1-anchor-rederive-run1.md`（保留） | 本文件（pre-fix §1/§3/§4 + post-fix §1.1/§4.1）+ `analysis/traces/gs5-*` |

## 6. 产物与复算

- trace：`analysis/traces/{gs5-stg01-clear-anchor,gs5-stg01-clear-c1,gs5-stg01-clear-c2,gs5-stg01-full,gameplay-stage1}.frames.jsonl`（GUI 版，gitignored）；
- 工具输出：本报告 §1 全量粘贴；`l5_probe_check.py` 输出见 §1；
- 复算：`python -X utf8 tools/s5_anchor_rederive.py`（**退出码：pre-fix=1 / post-fix=0**，L5-DIFF 全 MATCH）；`python -X utf8 tools/l5_probe_check.py`（A9 PASS）；
- 首差证据（F2）：
  ```bash
  python -X utf8 - <<'PY'
  import json
  def load(n):
      d={}
      for l in open(f"analysis/traces/{n}.frames.jsonl",encoding="utf-8"):
          r=json.loads(l); d[r["f"]]=r["w"]
      return d
  a,b=load("gs5-stg01-clear-anchor"),load("gs5-stg01-clear-c2")
  for f in (172,174,200,204):
      print(f,{k:(a[f].get(k),b[f].get(k)) for k in a[f] if a[f].get(k)!=b[f].get(k)})
  PY
  ```

## 7. 验收自查

- [x] `s5_anchor_rederive.py` 实跑：TRACE-DIFF **8/8 ALL MATCH**（§1/§2）；**post-fix 复测 F2 3/3、SUMMARY 15/15（§1.1）**；
- [x] F1 4/4、pre-fix F2 1/3 与 post-fix F2 3/3 双节如实（§3/§1.1/§4.1）；根因=脚本装载相位，已修复；
- [x] `l5_probe_check.py` A9 natural 7/7 PASS（工具面不变，§1）；
- [x] run1 保留对照、未硬凑；未改工具/expected/gates；
- [ ] docs-links PASS（提交前复核）；独立复核 pass。
