# 对抗复核：L5 B1 GUI 重导 8/8 + GUI trace 可行性预研（task-148）——adversarial-review-l5-run2

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用 l4-analyst 自述**）
> 被审：`docs/findings/l5-b1-anchor-rederive-run2.md`（**138 行，untracked**）、`docs/findings/l5-gui-trace-feasibility.md`（**109 行，untracked**）。
> 纪律：**未跑 Mesen/gates check/status/visual**；只读 `analysis/traces/*` + 跑 `tools/s5_anchor_rederive.py`/`l5_probe_check.py`；本报告只写该文件、未 git add/commit。

---

## 0. VERDICT：**needs_revision**（8/8 anchors、F1 4/4、A9 复核成立；**G-L5-4 表 2 处 A 级数值错**、**F2 结论已被后续 trace 重采覆盖且 96/1900 无法从当前 trace 复算**、boss 段长/ C2 段数口径不可复现）

- **通过项（A）**：
  - `python -X utf8 tools/s5_anchor_rederive.py` 在**当前 trace**上实跑：**TRACE-DIFF ALL MATCH (8/8)**、**F1 4/4**（bit4 窗 `[(913,1192)]`、f1193 intro_ctr=64、`$CA` 全程0、关末三联未见）；脚本 exit 0。
  - `l5_probe_check.py`：`artifacts 12/12`、`natural_ok=7`、`L5-DIFF natural 7/7`、`A9: PASS`（与报告一致）。
  - G-L5-4 关键帧链（stage/flags）：f913–1192 stage1/flags26=0x10；f1265(C1)/f1255(C3)–27730 stage8；**f27731 stage9=$180==9**；**f27734 flags26 bit7=0x80**；f27036 flags27 bit7=0x80；f27701 gate300=0x10；C1/C3 两 trace 转折帧一致；`trig_ca=5` 首帧 **f22405** 可核。
  - `boss_hp` 正段/`>0→0` kill edge **28 段（C1/C3）** 可核（C2 当前 trace 为 26 段，见 F3）。
  - run1（headless MISMATCH 3/8）报告的**语义解释**（`emu.setInput` 不生效、`stage_no` 全1、无 countdown 环）与当前 GUI 对照一致；run1 文件已标 superseded，作为历史证据正确。
  - S1–S5 建议**未越证据**：均以"需 Meson/新 watch/新场景"为前提、标注预估，不作既成事实；S1 复用 C3 输入+显式 capture、trigger=assisted 的注记合理。
  - docs-links **PASS**（388 md/466 links）；报告未改 expected/threshold/code/gates。
- **必改/需日期化（见 §3）**：
  1. **F1（A）**：G-L5-4 表 f27036 / f27036–27700 的 `trig_ca` 应为 **5**（不是 0；自 f22405 起=5）；f27701 的 `flags26` 应为 **0**（不是 0x10；f27700=0x10、f27701 bit4 清并同时 gate300=0x10）。报告表与自身 §4「trig_ca 自 f22405=5」矛盾。
  2. **F2（A/时点）**：run2 报告的 F2 **1/3（c1↔c2 96、c2↔anchor 1900、首差 f172）在当前 trace 已不可复算**——当前 `s5_anchor_rederive.py` 输出 **F2 3/3、SUMMARY 15/15 ALL MATCH**；当前 `gs5-*` trace mtime 22:53–23:00 晚于报告 22:49，且 pre-fix GUI trace 未在 `.archive` 留存（archive 只有 20:20 headless），**96/1900 无独立 trace 可核**。报告必须加 dated 注/标注 superseded，或在归档补 pre-fix GUI trace；否则 F2 失败结论与当前 workspace 矛盾。
  3. **F3（A）**：G-L5-2「段长 40–700f」「C2 25 段」无法按常规段定义复现：C1/C3 28 段长度 **2–6387f（仅17/28在40–700）**，kill-edge 间隔 58–7076；C2 当前 **26 段**（报告 25）。需定义"段"或修正；28 段 count 本身可核。

---

## 1. run2 8/8 / F1 复算（A）

我直接运行（未跑 Mesen）：

```text
python -X utf8 tools/s5_anchor_rederive.py
TRACE-DIFF: ALL MATCH (8/8)
[ok] F1 bit4 过渡窗定界（Run A）: flags26 bit4 置位窗 = [(913, 1192)]
[ok] F1 过渡窗语义（开 stage）: f1193 bit4 清 + intro_ctr=64（窗口期间 countdown2f=199）
[ok] F1 $CA 全程 0: trig_ca 非零帧数 = 0
[ok] F1 关末语义三联未出现（Run A）: 倒数环=无；INC $180 转移=无
...
SUMMARY: 15/15 通过（对拍 8 + 更正口径 7）→ ALL MATCH
exit 0
```

- 与 run2 报告 §1/§2 的 8/8、F1 4/4 一致（F1 逐项值一致）。
- **首次运行时** `gs5-stg01-clear-c1.frames.jsonl` 正被并行 GUI 采集写入（文件 2027/6000 行、末行截断），脚本 `json.JSONDecodeError` 退出；待 Mesen 结束（c1=6000 行、mtime 22:57）后重跑即得上述 15/15。**这不是报告工具缺陷**，而是复核窗口内有并发采集；结果必须以串行完成后的 trace 为准。

## 2. F2：报告失败数字 vs 当前 trace（核心时点问题）

- run2 报告（22:49）称：F2 1/3，c1↔c2 共享段 96 帧（首差 f172/f173）、f3201–5000=1800、c2↔anchor=1900；SUMMARY 13/15。
- 我在 **当前 trace** 上复算（c1/c2/anchor 均完整；mtime 22:53–23:00，晚于报告）：
  ```text
  F2 c1↔c2: f1..3200 mismatch=0 ; f3201..5000 mismatch=1800（设计性分歧）
  F2 c2↔anchor: mismatch=0
  SUMMARY 15/15 → ALL MATCH
  ```
- 当前 `gs5-stg01-clear-c2.frames.jsonl` 与 anchor **sha 同为 `afd2fc1f…`**（5000 行），c1 `a3052466…`（6000 行）；报告所述早期跨脚本差异已不存在。
- **pre-fix GUI trace 无法独立复核**：`analysis/traces/.archive/gs5-*` 仅含 20:18–20:20 的 headless 旧跑（且 headless 输入无效），没有报告时点的 GUI pre-fix trace；因此 96/1900/首差 f172 只能来自报告自述，当前 workspace 反证其已修复。报告必须 **dated 更新**（例如："22:49 记录为 pre-task-145 快照；task-145 修复后现为 15/15；96/1900 为历史数字，原始 trace 未归档"），否则不能作为现状入库。
- run2 报告 §4"跨脚本早期帧非确定性、task-145 在修"的根因陈述在当时可成立，但作为**未日期化的当前结论**已过时。

## 3. G-L5-4 证据链（A：逐帧复算）

从 C1 `tas-stg08b2-direct` / C3 `tas-stg08-direct` 的 `frames.jsonl` 读取（两 trace 同值）：

| 帧 | stage_no | flags26 | flags27 | gate300 | trig_ca | 报告表 |
|---:|---|---|---|---|---|---|
| 913–1192 | 1 | 0x10 | 0 | 0 | 0 | ✓ |
| 1265(C1)/1255(C3)–27730 | 8 | 0 / 0x10 段 | 0→0x80（f27036起） | 0 | 0→5（f22405起） | ✓（分段粗） |
| **f27036** | 8 | 0x10 | 0x80 | 0 | **5** | ❌ 报告写 trig_ca=0 |
| f27036–27700 | 8 | 0x10 | 0x80 | 0 | **5** | ❌ 报告写 trig_ca=0 |
| f27700 | 8 | **0x10** | 0x80 | 0 | 5 | — |
| **f27701** | 8 | **0** | 0x80 | **0x10** | 5 | ❌ 报告写 flags26=0x10 |
| **f27731** | **9** | 0 | 0x80 | 0x10 | 5 | ✓ `$180==9` |
| **f27734** | 9 | **0x80** | 0x80 | 0x10 | 5 | ✓ `$26:=$80` |
| 27734–32034（C1）/32031（C3） | 9 | 0x80 | 0x80 | 0x10 | 5 | ✓ 段 |
| f32035+（C1）/32032+（C3） | 1 | 0x80 | 0x80 | 0x10 | 5 | ✓ 回卷 |

- **结论**：帧号链、stage/flags/gate 转折点成立；**表中 f27036/45 的 trig_ca、f27701 的 flags26 是 A 级数值错误**（与其 §4 trig_ca=5@f22405 自相矛盾），必须修正。
- **claim level（B，非 A）**：trace 直接可核的是 `$180/ $26/ $27/ $0300/ $CA`；**`$95F9` 本身不是 trace 字段**（无 PC），"f27734 = `$95F9` 入口"由 `$26:=$80` + `bank00:95F9` 反汇编映射推得，属 **B（可复现静态推理）**，不是 A 级直接观测。报告标题「A 级入口链」应区分：链值 A、`$95F9` 归属 B。
- C3 stage9 区间 `f27731–32031`、f32032+ 回1；C1 `f27731–32034`、f32035+ 回1——报告 §2 的行号段与 trace 一致。

## 4. G-L5-2 boss_hp / G-L5-3 trig_ca（A/B）

- **28 段 count 可核**：C1/C3 `boss_hp` 正段（连续 >0）与 `>0→0` kill edge 均 **28**；C2 当前为 **26**（正段/kill edge 都是 26，报告写 25）。
- **段长 40–700f 不成立**：C1/C3 28 段长度分布 `[2,8,9,11,11,14,18,20,50×6,92,101,115,115,120,280,306,545,550,674,679,840,4988,6387]`，**min 2 / max 6387，仅 17/28 落在 40–700**；kill-edge 间隔 58–7076。若"段"有特殊定义（如仅有效 boss 段），报告未给定义/筛选式，无法复算。
- **`trig_ca=5` 首帧 f22405 可核**（C1/C3；C2 f22461），报告 §4 正确。
- **stage 归属结论成立**：C1/C3 的 stage8 是 poke `0180=08` 段（C1 f1265/C3 f1255–27730），boss 28 段无法自然归属 stage1..8；报告"全 run stage_no=8"措辞不准（实际 stage 1/8/9；应为"stage8 段"），不改结论。
- C2 段数/段长口径需与 C1/C3 统一；报告 C2 25 与当前 26 不一致（低风险数值）。

## 5. run1/可行性/S1-S5

- **run1 对照语义正确（历史）**：`l5-b1-anchor-rederive-run1.md` 已标 superseded；其结论 headless `emu.setInput` 不生效、stage_no 全1、countdown 无环、c1/c2 全0，与当前 GUI 8/8 对照自洽；不硬凑。旧 headless trace/截图未在本轮复算（当前 `analysis/traces` 已是 GUI；archive 为 20:20 headless 旧跑），故仅核其语义链，不核其数字（报告已声明历史）。
- **S1–S5 未越证据**：均为"新场景/overlay/需 Mesen"的采集建议，触发/assisted 标注明确，S4 自然八关仍标 pending；S1 的 7 帧 capture 列表与 4 个占位锚对应（stg08-start/combat/clear/all-clear-95f9），S2 以 C2 25 段为前提需按 F3 修正段数。估算时间（GUI ~6min 等）为预估、非实测，报告已标"预估"。
- **8/8、F1 4/4、A9**：当前脚本通过；与可行性文档互链一致。

## 6. Findings 四分类

### 6.1 证据/数字（A，需修）
- **F1**：G-L5-4 表 f27036/45 `trig_ca` 0→**5**；f27701 `flags26` 0x10→**0**（f27700=0x10，f27701 bit4 清）；否则与自身 §4 及 trace 矛盾。
- **F2**：F2 失败结论（96/1900/首差 f172/13/15）是 **pre-fix 时点快照**；当前 trace 15/15，且 pre-fix GUI trace 未归档，不能独立复算 → 需 dated 标注 superseded/补归档；否则入库会误导。
- **F3**：boss 段长 40–700/ C2 25 段不可复算（实际 min2/max6387、17/28在范围；C2 26）；需给"段"定义或修正。

### 6.2 claim 级/措辞（低/中）
- **F4**：`$95F9` 归属是 B 级（trace 无 PC）；"A 级入口链"应区分链值 A/映射 B。
- **F5**：C1/C3 "全 run stage_no=8" → 应写"stage8 段（f1255/1265–27730）"；stage_no 实际 1/8/9。
- **F6**：C1/C2/C3 输入轨表述"C1/C2/C3 120000f"中 C1 实为 40000f；不影响 assisted 结论，建议精确。

### 6.3 行为/工具（无阻塞）
- `s5_anchor_rederive.py` 在 trace 完整时 15/15；首次 JSONDecodeError 是**并行采集写入中断**，非工具逻辑；但工具对半写 trace 无 fail-fast 提示（可考虑读前稳定/明确报错）——低风险。
- A9 `natural 7/7`、`L5-DIFF natural 7/7` 与报告一致。

### 6.4 越权/伪造（无）
- 报告未改 expected/threshold/code/gates；未冒充 natural（C1/C3 assisted 明确）；F2 负证据（当时点）未硬凑全绿；run1 superseded 诚实。当前工作树 `scenario_capture.lua` M 属 task-145 并行改动，不是这两份报告本次改动。

## 7. 命令与输出摘要

```text
# 当前 trace 上实跑（Mesen 未跑）
python -X utf8 tools/s5_anchor_rederive.py
TRACE-DIFF: ALL MATCH (8/8)
F1 4/4 ok；F2 3/3 ok（c1↔c2 pre=0/post=1800；c2↔anchor=0；gameplay↔full=0）
SUMMARY: 15/15 → ALL MATCH；exit 0
（首跑遇并行 c1 写入 JSONDecodeError @ c1 line，等 c1 完整 6000 行后重跑）

python -X utf8 tools/l5_probe_check.py
artifacts 12/12；natural_ok=7；L5-DIFF natural 7/7；A9: PASS

# G-L5-4 逐帧
C1/C3: 27036 stage8 flags26=0x10 flags27=0x80 gate0 trig=5
        27700 flags26=0x10 gate0；27701 flags26=0 gate=0x10 trig=5
        27731 stage9 flags260 gate0x10 trig5；27734 flags26=0x80
# boss
C1/C3 kill edges=28；segment lens 2..6387（17/28 in 40..700）；C2=26（报告25）
trig_ca first5=f22405（C1/C3）/f22461（C2）
# traces
gs5 anchor sha afd2fc1f… 5000；c2 sha afd2fc1f… 5000（mtime 22:53–23:00，晚于报告22:49）
c1 sha a3052466… 6000；full d039dccd… 15000
docs-links PASS（387 md/466 links）
```

## 8. 复核边界

1. 未跑 Mesen/gates check/status/visual；`analysis/traces` 在复核期间被并行 GUI 重采覆盖（c1 中途半写→完整；c2 23:00 更新），所有结论以串行完成后的当前 trace 为准，并已标时间点；pre-fix F2 trace 无法恢复，F2 96/1900 仅报告自述、无独立核。
2. 未修改两份报告/工具/expected/gates；本报告只新增文件、未 git add/commit。
3. run1 headless 数字未重算（trace 已被 GUI 覆盖/archive 为旧 headless），仅核其语义链与 superseded 声明。
4. F2/F3 属报告时点/数值口径问题，不改变 B1 8/8 + F1 4/4 主判据，但影响入库时的"当前结论"准确性。

---

【2026-10-18】decomp-verifier，独立复核 task-148（HEAD `f66c195`；两份 L5 报告 untracked）；当前 s5 15/15、8/8、F1 4/4、A9 PASS；发现 G-L5-4 表 2 处 A 级数值错、F2 96/1900 已被后续 trace 覆盖不可复算、boss 段长/C2 段数不可复现。

---

## 点验（2026-10-18，task-150）

> 点验对象：task-149 修正后的 `l5-gui-trace-feasibility.md`（110 行）与 `l5-b1-anchor-rederive-run2.md`（190 行）；HEAD `023c1d8`；只点验 task-148 的 F1/F2/F3/B 级措辞，不重复完整复核。**未跑 Mesen/gates check/status/visual**；`s5_anchor_rederive.py` 用最终稳定 trace 复跑。本段只追加、未 git add/commit。

**VERDICT: needs_revision**（F1 表数值、F2 三节/sha/first_poll、F3 boss 口径、`$95F9` B 级均已修且复算通过；**残留 4 处措辞/表格未同步**，修完即可 pass/入库。）

### F1（G-L5-4 表）PASS
- 当前表 L44-46 已为：f27036 `trig_ca=5`、f27036–27700 `trig_ca=5`；f27701 `flags26=0`（bit4 清）、`gate300=0x10`、`trig_ca=5`——与 C1/C3 trace 逐帧一致（f27700 flags26=0x10，f27701 转0/gate=0x10）。已修复 task-148 F1。

### F2（dated 三节/链/first_poll）PASS（复跑）
- 报告已含三节：§1 pre-fix 22:49（F2 1/3，不可复算）、§1.2 中间版 23:00（c2 `first_poll=8`，trace 已被覆盖）、§1.1 post-fix（F2 3/3、SUMMARY15/15）；superseded 链与归档可核：c1 `4c673738→a3052466`（archive c1 4c673738 存在）、c2 `b8c161e1→62ecf43b→afd2fc1f`（archive 两旧版存在）、anchor/final c2 均 `afd2fc1f`（同 sha）。
- 我在 trace 稳定后复跑 `python -X utf8 tools/s5_anchor_rederive.py`：**TRACE-DIFF 8/8、F1 4/4、F2 3/3、SUMMARY 15/15 ALL MATCH、exit 0**（c1↔c2 pre=0/post=1800；c2↔anchor=0；gameplay↔full=0）。
- `first_poll=7` 一致：当前三份 `*.lua.log` 均 `script load: game_frame=3` → `phase align: late-load mode ... align_gate=7` → `scenario start: game_frame=7 align_gate=7`；最终 trace c1 `a3052466…`、anchor/c2 `afd2fc1f…` 与报告 §1.1 一致。中间版 c2 旧 trace `62ecf43b…` archive 存在（其 lua.log 仅存 script load 行，first_poll=8 本身不可从该 log 再核，但属历史声明、不阻塞）。
- **注意**：复跑数次期间并行 GUI 重采多次半写 c2（348/2595 行），首次 s5 曾 14/15；待 c2 完整 5000 行（sha afd2fc1f）后复跑得 15/15。报告 post-fix 结论成立。

### F3（boss 口径）PASS
- 独立复算（当前 trace）：C1/C3 连续 `boss_hp>0` **28 段**，段长 **2–6387f**、**17/28** 在 40–700f，kill-edge 间隔 **58–7076f**；C2 **26 段**，段长 **9–12990f**、20/26 在 40–700f，间隔 58–13286f。与报告 L64-65 完全一致。已修复 task-148 F3。

### B 级归属 PASS
- `$95F9` 已明确：`$180==9`/`$26:=$80` 为 A 级 trace 观测，`$95F9` 入口归属为反汇编映射（B 级）；S1/验收自查同步标注（L11/36/48/53/85/107）✓。

### 残留未同步（needs_revision 原因）
1. **feasibility L66 仍写"全 run `stage_no=8`"**：实际 C1/C3 stage_no 为 **1/8/9/1**，应改"stage8 段（C1 f1265/C3 f1255–27730）"；"28 段无法归属 stage1..8"结论不变。task-150 点名"stage8 段表述"未落。
2. **feasibility L78 仍写"C1/C2/C3 ... 120000f 推进"**：C1 实为 **40000f**；应改"C1 40000f；C2/C3 120000f"。task-150 点名 C1=40000f 未落。
3. **run2 L23 仍写"F2 剩余 2 条失败…不宣称 F1/F2 全通过"**：该句当前时态，与 post-fix F2 3/3/15/15 矛盾；应标 pre-fix 历史或改为已修复。
4. **run2 §5 表 L159-160 仍写 run2 F2 1/3、SUMMARY13/15**（L161 根因行已写 post-fix 3/3）：应把表改为 post-fix（pre-fix 括号保留），L168"退出码1=MISMATCH"也应注明 pre-fix=1/post-fix=0；否则报告内部自相矛盾。

### 其它
- `docs-links` **PASS（389 md/466 links）**；两份报告未改 expected/threshold/code/gates；并行 `scenario_capture.lua` M 属 task-145，不是本轮报告改动。
- **结论：F1/F2/F3/B 修正均成立并可复算；仅上述 4 处文字/表格同步即可 pass。** 修后两份 L5 报告可入库。

【2026-10-18】decomp-verifier，点验 task-149/task-150（HEAD `023c1d8`）；最终 trace s5 15/15、first_poll=7、boss 28/C2 26 与 F1 表值一致；残留 L66/L78/run2 L23/§5 四处待同步。

---

## 点验（2026-10-18，task-153）

> 点验对象：task-152 修正后的 `l5-gui-trace-feasibility.md`（111 行）与 `l5-b1-anchor-rederive-run2.md`（190 行）；HEAD `023c1d8`；只点验 task-150 的 4 处文字残留，不重复复算。未跑 Mesen/gates；本段只追加、未 git add/commit。

**VERDICT: pass**（task-150 的 4 处已全部同步；无阻塞残留；1 处可选一致性建议见末尾。）

1. **feasibility L66-67 PASS**：已改 "`$0180` 被 poke 0180=08 锁定 ⇒ **stage8 段（C1 f1265–27730 / C3 f1255–27730）`stage_no=8`**，28 段 boss 无法归属 stage1..8；**C1/C3 实有 stage_no 1/8/9/1**"；无"全 run stage_no=8"残留。
2. **feasibility L79 PASS**：已改 "**C1 40000f；C2/C3 120000f** 推进、C1/C3 到 `$180==9`"；与 trace 帧数一致。
3. **run2 L23 PASS**：保留 pre-fix 失败句但已显式标 "**这是 pre-fix 历史时态**；task-145 对齐修复后 post-fix **F2=3/3、SUMMARY=15/15**，见 §1.1/§4.1"，不再与当前结论矛盾。
4. **run2 §5 表 + L168 PASS**：L159-160 run2 列已改 "**post-fix 3/3**（pre-fix 1/3…）" / "**post-fix 15/15**（pre-fix 13/15）"；L168 已注 "退出码：**pre-fix=1 / post-fix=0**"。

- **其余已由 task-150 点验通过**（本段不重复）：F1 表 trig_ca=5/flags26=0、F2 dated 三节+superseded 链+first_poll=7、F3 boss 28/C2 26 口径、`$95F9` B 级归属、当前 s5 8/8+F1 4/4+F2 3/3+15/15。
- `docs-links` **PASS（389 md/466 links）**；两份报告未改 expected/threshold/code/gates；并行 `scenario_capture.lua` M 属 task-145。
- **可选非阻塞**：feasibility §0 L14 的 G-L5-1 摘要仍写"C1/C2/C3 … 120000f 推进"（L79 已修）；建议顺手改成 C1 40000f/C2/C3 120000f，避免与 §5 表不一致。不影响本次 pass。
- **结论：4 处修复全部落地、无阻塞；两份 L5 报告可入库。**

【2026-10-18】decomp-verifier，点验 task-152/task-153（HEAD `023c1d8`）；L66-67/L79/run2 L23/§5 表+L168 全部按 task-150 要求同步；docs-links PASS。
