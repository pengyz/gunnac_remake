# 对抗复核：P0-0 组 D 来源定谳（C1/C2/C3 blob/mode/trigger）——adversarial-review-groupD-decision

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用 l4-analyst 自述**；l4-analyst 是决策作者，不能自审）
> 被审：`docs/findings/p0-0-groupD-source-decision.md`（**186 行，工作树 M**，+27/−4；含 §2.4 修复后重跑负证据）。
> 纪律：**未跑 Mesen/scenario_run/gates check/status/visual**；全部用 gitignored 产物（`analysis/traces/*.run.json/.done/frames.jsonl`、`shots/`、`analysis/repro/p0-0-groupD/`）直接复算。只写本报告、未 git add/commit。

---

## 0. VERDICT：**needs_revision**（核心裁定 `blocked:no-source` 成立；**2 处 A 级数字/范围错误必须改**，3 处低风险证据指针/措辞建议同步；修正后可提交/可 pass）

- **核心裁定成立（A）**：六对 expected 的 blob 比对在 C1（26 shots）/C2（80 shots）/C3（80 shots）**全 0/6**；目标帧（28500/30000/31500/33000/34500/36000）`stage_no=1/2`、`flags26=0x00/0x80`；`$26 bit7` 自 f5217 起在 stage1/2/3 战斗段大段置位，**不是 Ending 专属**；C2/C3 目标帧 shots **逐字节相同**；C4/C5 未评估。⇒ **组 D 六对在现有候选上无来源，不硬凑、不 staging 的裁定合理**。
- **F1（A，必改）**：文档 §2.2 L92 的 C3 全帧 stage_no 计数 `{1:26894, 2:65993, 3:23144, 8:3962}` **合计 119,993，不等于 120,000**；独立复算为 **`{1:26901, 2:65993, 3:23144, 8:3962}`（合计 120,000）**——`26894` 应改为 **26901**（差 7）。裁定不受影响，但 A 级数字不得错。
- **F2（A，必改）**：文档 §3.1 L123 称 `poke 0180=08/07` 只在 f1255–5216 反映为 **`stage_no=8`**——**对 C2 不成立**：C2 是 `poke 0180=07`，实测 `stage_no=7` 在 **f1255–5216（3,962 帧）**，**stage8 为 0 帧**；C3（08）才是 stage8 f1255–5216；C1（08）stage8 f1265–5216。应改为"08→stage8 / 07→stage7"；"C2 未进入 stage8"结论更强、裁定不变。
- **F3（低，证据指针）**：§1/§5 的 `analysis/traces/tas-stg08-direct.done`、`tas-stg07-relay.done` 现在指向**修复后最终重跑**（C3=120000/frame_errors0；C2=0），而非 §1 引用的**首轮** DONE（C3=120001/overshoot；C2=142149）。首轮证据仍完整保存在 `analysis/repro/p0-0-groupD/capture-tas-stg0{7,8}-*.json`（我已核）；建议 §5 指针明确区分"首轮 capture JSON"与"final rerun done/run.json"。
- **F4（低，时点措辞）**：C2 经 task-121 修复后重跑 **frame_errors=0**（§2.4 已写），但 §2.3 L99、§3.4 L128、§4.2 L148 仍以 "C2 DEGRADED/先修 frame_errors" 作为不可用/复用前置——应改为"**旧跑 DEGRADED；重跑合法但 blob 仍 0/6**"；C2 不可用结论改为由 blob/语义支撑，不再由 DEGRADED 支撑。
- **F5（低，机器可读一致性）**：`analysis/repro/p0-0-groupD/decision.json` 仍是首轮快照（C1 `BLOCKED:capture-termination (not retried)`、C2 `DEGRADED 142149`、C3 `overshoot 120001`），未反映 §2.4 的 final reruns；文件名为 decision，容易误读。建议更新或显式标注"首轮快照，final 以 §2.4/ledger-rerun 为准"。
- **F6（info）**：§1 判据把 C3 首轮"无 frame_errors 字段按 0"记 PASS——**缺字段≠0**（首波 lua 无 pcall/计数；旧 bot 路径错误可能未被记录）。§2.4 最终重跑已给 `frame_errors=0`，建议 §1 该行标"legacy/无计数字段；frame_errors 证据以 §2.4 final 为准"，不要用首轮缺席字段当 0 证据。
- **I5 采集层表述 PASS**：`docs/findings/p0-0-capture-run1.md` §4.4 写"I5 采集层 BLOCKED 解除（旧 over-budget/capture-termination 为旧 lua 历史）**但组 D 来源结论不变**"，与 C1 实测 40000/40000、26 shots、frame_errors=0、sha `486a157e…` 一致；表述准确（解除仅采集层，不是组 D 有来源）。
- **格式/范围**：`docs-links` **PASS**（368 md/454 links）；`git status` 仅两个 docs M（本决策 + capture-run1），**未改 expected/阈值/code/gates**；无伪造/越权。

---

## 1. C3 合法性与数字（独立复算）

| 项 | 首轮（§1 记录；`analysis/repro/p0-0-groupD/capture-tas-stg08-direct.json`） | 最终重跑（§2.4；当前 `analysis/traces/…`） |
|---|---|---|
| DONE | `{"frames":120001,"frames_plan":120000,"frames_actual":120001,"overshoot":true,"trace_lines":120000,"captures":80,"shots":80,"bot_stop":false,"finalize_by":"inputPolled-overshoot"}` | `{"frames":120000,"frames_plan":120000,"frames_actual":120000,"overshoot":false,"trace_lines":120000,"captures":80,"shots":80,"bot_stop":false,"finalize_by":"endFrame","frame_errors":0}` |
| trace sha256 | `d6944fe9a4de8b712eaeadb8f710624ba8bd983610791a9aafb62866a107a547`（120000 行） | **同 sha**（120000 行） |
| 判定 | 旧 lua 首轮：frame_errors 无字段（§0 首轮证据）；最终重跑：`frame_errors=0` PASS | PASS |

- **C3 首轮 DONE/overshoot/trace sha/120000 行全部与文档一致**（我独立从旧 capture JSON + 当前 trace 复算；当前 `.done` 已被 final 重跑覆盖，见 F3）。
- 当前 `analysis/traces/tas-stg08-direct.run.json`：frames=120000、trace_lines=120000、timed_out=false、done=§2.4 行；trace sha 同 `d6944fe9…`。
- C3 目标帧 shot sha 与文档 §2.1 前缀一致（见 §2）。
- 机械可读 final 证据 `capture-tas-stg08-direct-final.json` + `ledger-rerun.jsonl` 均记录 `valid:true/frame_errors:0`。

## 2. 三判据逐对（核心；A）

### 2.1 判据① blob：0/6（含全量 shots）

我直接从 `oracle/screens/expected/`、`shots/` 复算 sha256：

- 旧 expected 六张 sha 与文档 §2.1 **逐一相同**；`l4_ending-4` 与 `l4_ending-5` 同 blob（`6ff345dc…`）。
- 目标帧：C3/C2 六帧 shot sha 与文档 §2.1 全部一致，**六对全 ❌**；C1 同样六对全 ❌。
- **全量扫描**：C3 80 张、C2 80 张、C1 26 张 shots **BLOB_HITS: NONE**（与文档 §2.4/§0 一致）；
- **C3 vs C2 目标帧 shots 逐字节相同 = True**（跨场景等价，stage-select 未持久生效）。
- 结论：**不是"目标帧选错/相位±1"级别，而是候选 run 中根本没出现这 6 张旧 expected**；0/6 成立。

### 2.2 判据② mode / stage_no / $26 bit7

从当前 `frames.jsonl`（sha 见 §1）读目标帧：

| 帧 | C3 flags26/stage | C2 flags26/stage | C1 flags26/stage |
|---:|---|---|---|
| 28500 | 0x00 / 1 | 0x00 / 1 | 0x00 / 1 |
| 30000 | 0x00 / 1 | 0x00 / 1 | 0x00 / 1 |
| 31500 | 0x00 / 1 | 0x00 / 1 | 0x00 / 1 |
| 33000 | 0x80 / 2 | 0x80 / 2 | 0x80 / 2 |
| 34500 | 0x80 / 2 | 0x80 / 2 | 0x80 / 2 |
| 36000 | 0x80 / 2 | 0x80 / 2 | 0x80 / 2 |

- `$26 bit7` 从 **f5217** 起大段置位（C3/C2 bit7 共 **93,915** 帧，stage 分布 2:65,993 / 3:23,144 / 1:4,778；C1 同起点）；**不是 Ending 专属** → 原 §3.6 判据②作废成立。文档"f≈5217 起 stage2 战斗段"方向正确；严格说 bit7 也覆盖 stage1/3 段（不影响"非 Ending 专属"结论）。
- **stage_no 计数（独立复算，A）**：
  - C3：**{1:26,901, 2:65,993, 3:23,144, 8:3,962}**（合计 **120,000**）——文档 §2.2 的 `26,894` 错误（F1），其余一致；
  - C2：`{1:26,901, 2:65,993, 3:23,144, 7:3,962}`（**stage8=0**，stage7 f1255–5216）——文档 §3.1 说 C2 `stage_no=8` 错误（F2）；
  - C1：`{1:6,482, 2:29,566, 8:3,952}`，stage8 **f1265–5216**（与文档/任务卡一致）。
- "stage8 直入" **对 C1/C3 只在 poke 后 f1255/1265–5216**，之后被引擎覆盖为 1/2/3；C2 则只到 stage7。**C2/C3/C1 在 120000/40000f 内都没到 stage8 关末/ending**；语义判据不成立成立。

### 2.3 判据③ trigger

- C3：`poke 1255 0180 08` + assist 三 poke `01BE/01B6/01B7=01` + `bot stg01-clear from 1260` ⇒ `trigger=injected/assisted`；
- C2：`0180=07` 同结构；C1：`poke 1265 0180 08` + assist + bot。
- 三场景字段与文档一致；injected/assisted 标注真实，未冒充 natural；trigger 合法**不能替代** blob/语义——文档已明确。

## 3. C2 对照（旧 DEGRADED / 重跑合法；blob 仍 0/6）

| 项 | 首轮（旧 capture JSON） | 修复后重跑（当前） |
|---|---|---|
| DONE | `frame_errors=142149`、frames120000/trace120000 | `frame_errors=0`、120000/120000、finalize_by=endFrame |
| trace sha | `e12e6ac4d51afaaba8912b38ca77b98bf9f5ffe4f2bc040afe616fda61a71c24` | **同 sha**（120000 行） |
| blob | 0/6；目标帧 shots 与 C3 逐字节相同 | 0/6；全 80 shots NONE；目标帧仍与 C3 相同 |
| 结论 | 旧跑不可用（DEGRADED） | 采集合法，但 **blob/语义仍不合格 ⇒ C2 不可作组 D 来源** |

- 旧 `142149`、旧 trace sha、旧 DONE 字段均从 `capture-tas-stg07-relay.json` 独立核出，与文档 §1 一致；重跑 `frame_errors=0` 与 §2.4 一致。
- **C2 不可用结论成立，但理由应更新**：不是"当前 DEGRADED"，而是"重跑合法但 blob 0/6、目标帧 stage1/2"（F4）。

## 4. 裁定合理性与后续建议

- **证据充分、未硬凑（PASS）**：C1 26 / C2 80 / C3 80 全量 shots 无任何 expected blob；目标帧 mode 明确 stage1/2；`$26 bit7` 非 Ending；C4/C5 未评估；C1 重跑只解除采集层；保留旧 expected、不 staging、不替换；`blocked:no-source` 合理。
- **§4.2 建议基本合理、未见超证据**：更正 §3.6 判据②、停止跨 run 固定帧号、any-frame dense capture、先取得真到达 ending 的合法 run；C1/C4/C5 处置谨慎。**需改的时点项**：
  - C1 已重跑成功（40000/40000、frame_errors0），不再是"可选重试/旧 BLOCKED"；建议写"已重跑合法但 0/6，C4/C5 未评估"；
  - C2 已修 bot、重跑合法，§4.2 的"先修 frame_errors/标 DEGRADED"前置应改为"重跑已合法；若复用需重新做 blob/语义"（F4）。
- §4.2 与手册 §3.6 已存在的 dated 更正（manual L699-711）方向一致，不越证据。

## 5. Findings 四分类

### 5.1 证据/数字（A，**必改**）
- **F1**：§2.2 C3 stage_count `1:26894` → **`1:26901`**（其余 3 项正确；doc 合计 119,993 ≠ 120,000）。
- **F2**：§3.1 `0180=08/07 → stage_no=8` 需按 poke 值拆分：**08→8（C3 f1255–5216）/ 07→7（C2 f1255–5216）**；C2 stage8=0。

### 5.2 证据指针/时点（低）
- **F3**：§1/§5 的 `.done` 指针被 final rerun 覆盖；首轮证据在 `capture-*.json`，建议加"首轮 capture JSON"指针。
- **F4**：§2.3/§3.4/§4.2 的 C2 DEGRADED/先修 frame_errors 措辞过时；改为"旧跑 DEGRADED；重跑合法但 blob 0/6"。
- **F5**：`decision.json` 首轮快照未更新；建议更新或标注 snapshot。
- **F6（info）**：§1 C3 "无字段按 0" 仅是首轮约定；frame_errors 证据以 §2.4 final 为准。

### 5.3 行为/实现
- 不适用（本卡纯文档/取证；无代码改动）。C1/C2/C3 数字、blob、mode、trigger 均已独立复算；C4/C5 未评估不越界。

### 5.4 越权/伪造
- **无**。只读、未改 expected/阈值/code/gates；`git status` 仅两 docs M；`analysis/`、`shots/` gitignored；本报告仅新增文件、未 git add/commit。

## 6. 命令与输出摘要

```text
# 当前 artifacts（独立读）
tas-stg08-direct: run frames=120000 trace_lines=120000 timed_out=false wall=406.9
  done {frames:120000,frames_actual:120000,overshoot:false,trace_lines:120000,captures:80,shots:80,finalize_by:"endFrame",frame_errors:0}
  trace_sha d6944fe9a4de8b712eaeadb8f710624ba8bd983610791a9aafb62866a107a547 lines=120000
tas-stg07-relay: done … frame_errors=0; trace_sha e12e6ac4… lines=120000
tas-stg08b2-direct: done frames=40000,trace_lines=40000,captures/shots=26,frame_errors=0; sha 486a157e…

# 首轮（repro/capture JSON）
C3 DONE 120001/overshoot/inputPolled-overshoot; trace 120000; sha d6944fe9…
C2 DONE 142149; trace sha e12e6ac4…

# blob（A）
expected sha 6 张与 doc §2.1 相同；
C3/C2/C1 全 shots 扫描 BLOB_HITS NONE；目标帧 C3/C2 六对全 False；C3 vs C2 目标帧 shots identical=True

# frames（A）
C3 target (28500,30000,31500)=(flags0,stage1); (33000,34500,36000)=(0x80,stage2)
C2 同；C1 同
C3 stage {1:26901,2:65993,3:23144,8:3962} sum=120000  ← doc 26894 错
C2 stage {1:26901,2:65993,3:23144,7:3962} stage8=0（stage7 f1255-5216）
C1 stage8 f1265-5216 count3952
bit7: first f5217；C3/C2 93915 帧；stage {1:4778,2:65993,3:23144} ⇒ 非 Ending 专属
docs-links PASS（368 md/454 links）
```

## 7. 复核边界

1. 未跑 Mesen/scenario_run/gates check/status/visual；未重跑任何候选。
2. 使用当前 `analysis/traces/*` + `analysis/repro/p0-0-groupD/*` + `shots/` + 旧 expected；当前 `.done` 已被 §2.4 final 重跑覆盖，首轮 DONE 以 repro capture JSON 为准。
3. 裁定结论独立成立；F1/F2 是 A 级数字/范围错误，修复不改结论但**必须在提交前修正**（建议改后由作者/点验复核两行即可）。
4. 本报告只新增文件；未改 groupD 文档/expected/threshold/code/gates。

---

【2026-10-18】decomp-verifier，独立复核 task-106 组 D 定谳（HEAD `dabd6af`；文档 186 行）；blob 0/6、mode/stage 复算、C1/C2/C3 DONE 与 shas；发现 C3 计数 +7 与 C2 stage8 范围两处 A 级错，裁定本身成立。

---

## 点验（2026-10-18，task-129）

> 点验对象：`docs/findings/p0-0-groupD-source-decision.md`（**199 行**，工作树 M，`task-127` 修正后）；HEAD `e831a4c`；只点验 task-112 的 F1/F2 + F3-F6，不重复完整复核。只读 gitignored 产物复算；未跑 Mesen/gates check/status/visual；本文件只追加本段、未 git add/commit。

**VERDICT: pass**（F1/F2 两处 A 级错已改且独立复算一致；F3-F6 落点均到；核心裁定 `blocked:no-source` 不变；仅 3 条非阻塞残留见末尾）。

### F1（必核）PASS
- 文档 L96 已改为 **`{1:26901, 2:65993, 3:23144, 8:3962}`（合计 120,000）**，并注"f1–f7 为预热帧，旧稿 26894 少计 7 帧"。
- 我用 `analysis/traces/tas-stg08-direct.frames.jsonl`（120,000 行；trace sha `d6944fe9…`）逐行解析复算：**完全一致，合计 120,000**；前 7 行 `stage_no` 全为 1（预热帧说明可核）。
- 旧值 26,894 的少计差 7 与 f1–f7 预热帧吻合（旧稿问题已消除）。

### F2（必核）PASS
- 文档 §3.1（L129-132）已改为：`0180=08`（C3）⇒ `stage_no=8` 仅 **f1255–5216（3,962 帧）**；`0180=07`（C2）⇒ **stage_no=7** 仅 **f1255–5216（3,962 帧）、stage8=0**；并明示 C2 未进入 stage8。
- 独立复算：C2 `tas-stg07-relay` **stage8=0**；**stage7 n=3,962 / min=1,255 / max=5,216**；C3 `tas-stg08-direct` **stage8 n=3,962 / min=1,255 / max=5,216**。与文档一致。

### F3（低）PASS
- 文档 L35-37 已明确区分：首轮（旧 lua）证据在 `analysis/repro/p0-0-groupD/capture-tas-stg0{7,8}-*.json`（gitignored 首轮快照）；磁盘 `analysis/traces/*.done/.run.json` 已是 final rerun。
- 独立核：首轮 `capture-tas-stg08-direct.json` DONE=`frames_actual 120001/overshoot true/inputPolled-overshoot`、`capture-tas-stg07-relay.json` DONE=`frame_errors 142149`；当前 `.done` 为 final（C3/C2 均 120000、frame_errors=0）——L165-168 指针与磁盘一致。

### F4（低）PASS
- §2.3（L104-105）、§3.4（L136-138）、§4.2（L158-159）均已改为"旧跑 DEGRADED；重跑合法（frame_errors=0）但 blob 0/6、目标帧 stage1/2"；"C2 不可用"结论不再以旧 DEGRADED 为理由。
- 残留（非阻塞）：§6 验收自查 L194 仍写"C2 按 frame_errors 标 DEGRADED"（未加"首轮/final"限定）；建议提交前顺手改"首轮 DEGRADED；final rerun frame_errors=0"，不影响裁定。

### F5（低）PASS
- `analysis/repro/p0-0-groupD/decision.json` 已加 **`snapshot`**、**`final_rerun`**（C1/C2/C3 均 40000/120000、frame_errors=0、blob 0/6）、**`updated_at`** 字段；首轮 candidates 保留为快照，不再被误读为最终状态（独立读取确认）。

### F6（info）PASS
- §1 L33 已注明："无字段按 0"**仅首轮约定**；frame_errors 证据以 §2.4 final rerun=0 为准；旧 C3 首轮与 final 区分清楚。

### 复核边界与残留
- `docs-links` **PASS**（371 md/457 links，含并行 same-blob 文档；本报告追加后复扫）；`git status` 仅预期 docs M + 本复核报告 untracked；**expected/阈值/code/gates 未改**。
- 非阻塞残留：①§6 L194 的 C2 DEGRADED 措辞可加"首轮"限定；②§6 L199 `[ ] docs-links PASS（提交前复核）` 可勾选/改为已 PASS；③§4.2 item 3 "C1 可选重试"是修正前时点表述（C1 已重跑合法但 0/6），建议顺手同步或保留为历史建议。
- **结论：核心裁定 `blocked:no-source` 与 F1/F2/F3-F6 修正均成立，可提交 groupD 决策（199 行）。**

【2026-10-18】decomp-verifier，点验 task-127/task-129（HEAD `e831a4c`；199 行）；F1 计数 26901/65993/23144/3962=120000、F2 C2 stage7 n3962 1255–5216/stage8=0、C3 stage8 n3962 1255–5216 独立复算一致。
