# 对抗复核：GUI 重采批次（输入有效性 + 两跑确定性 + resume 清理 + 组 D）（task-137）

> 日期：2026-10-18｜复核方：platform-verifier（verifier，证伪立场）｜被审：l4-analyst task-133
> 被审：`docs/findings/p0-0-recapture-gui-run.md`（162 行，untracked）
> 复核方式：只读 trace/ledger/shots + 独立短跑 title-menu 点验（GUI 串行锁空闲时）；**未跑 Mesen 长场景**；
> 未跑 gates check/status/visual；未改代码/gates/expected/scenario。本文件唯一写入。

## 0. VERDICT

**needs_revision（P1：失败/重跑清单不实）**

- **数据主体成立**：12/12 场景最终 run 合法（rc=0、done、¬timed_out、trace_lines==done.trace_lines、
  frame_errors=0）；输入断言复算通过；两跑 3/3 trace+shots 全同；组 D 新证据数字全对；settings 已恢复。
- **报告口径不实**：§0/§7 写"失败：无；无重跑需求"，但 `analysis/repro/recapture-gui/ledger.jsonl`
  有 **29 行尝试**，含 **title 第二跑 `57e25e3d`（resume 污染）** 与 **gameplay-stage1 attempt1
  `rc=-1`/trace_lines=826/done=None 失败**，以及早期 title/c1/c2 重跑；报告把这些全部省略。
- 其余 F137-2..5 为低优先级；不影响最终 15 条有效 run 的数据成立，但影响"失败清单如实"验收。

## 1. 输入有效性断言逐场景（独立复算）

从 `analysis/traces/<scene>.frames.jsonl` 直读：

| scene | trace sha16 | controller_held 非零 | pressed 非零 | action_mode distinct | stage_no distinct | player_x range |
|---|---|---:|---:|---:|---:|---:|
| gameplay-stage1 | `0fdd22821701b84a` | **1049** | **195** | **4** | — | **243** |
| gs5-stg01-clear-anchor | `afd2fc1f1119d2cc` | **1094** | — | — | **2** | — |
| gs5-stg01-full | `d039dccd06d6699f` | **12601** | — | — | 1（无推进但输入非0） | — |
| gs5-stg01-clear-c1 | `4c673738cfc7d941` | **1158** | — | — | **2** | — |
| gs5-stg01-clear-c2 | `b8c161e18f0f113f` | **1093** | — | — | **2** | — |
| tas-stg08b2-direct (C1) | `9c06314c2f0cf0f1` | **22511** | — | — | **3** | — |
| tas-stg07-relay (C2) | `27fa275a2d8a8625` | **30374** | — | — | **3** | — |
| tas-stg08-direct (C3) | `2c7962ec97c66107` | **22512** | — | — | **3** | — |

- gameplay 的 `player_x` 0→243、`controller_held` 1049 非零，与 Lead 实测一致；`game_action_mode`
  4 种、f1260=7（Combat）在 trace 内可复算。
- title/stg03/stg06/stg08 无 controller watch；用 **shots 语义**独立核：
  - title f180 = expected `788d6984…`（boot-black）精确相等；
  - stg03 六帧（3404/4600/5400/5700/6500/12000）与 expected 全部精确相等（`788d…/5cac…/9888…/24a4…/6f7f…/c0ba…`）；
  - stg08 六帧（14600/16000/16600/18000/22000/28000）与 expected 全部精确相等（`ddb3…/2824…/62a8…`）；
  - gameplay f1260/f1500/f2349 当前 shots sha 与报告/ledger 的 semantic `gui_diff=0` 一致（f1260 `38365db2…` stage 画面）。
- ⇒ **12/12 输入有效性成立**（report 的“watch 或语义兜底”口径正确）。

## 2. 两跑确定性（独立复算）

从 ledger 取每场景**最后两次 valid+assert_pass** 行：

| scene | trace sha16（两跑） | trace 相同 | shots 全字典相同 | shots 数 |
|---|---|---|---|---|
| title-menu | `c4d6a83f…` | ✅ | ✅ | 37/37 |
| gameplay-stage1 | `0fdd2282…` | ✅ | ✅ | 44/44 |
| gs5-stg01-clear-anchor | `afd2fc1f…` | ✅ | ✅ | 28/28 |

**3/3 PASS**；我另独立连跑 title-menu（GUI，resume 清理后）**6 次**，trace 全部 `c4d6a83f…`；
独立 2 次带 stdout 的跑均出现 **1 行 `[清场] 删除 Mesen2 resume 状态: gunnac_cn.rgd`** 且 trace 同为 c4d6。

**GUI vs headless**：report 的对照 `62c9ea09`（headless，无输入）vs `c4d6a83f`（GUI）成立；headless
旧 trace 不作为当前输入证据（superseded）。

## 3. resume 清理

- 代码 `tools/scenario_run.py:482-499 clear_mesen_resume_state()` + **L618-621 spawn 前调用**，
  GUI/headless 均生效；成功打印 `[清场] 删除 Mesen2 resume 状态`（独立 2x 实测各 1 行）。
- 运行中/结束后 `RecentGames/gunnac_cn.rgd` 会再次被 Mesen 写回（当前 9874B），属正常；**每次 spawn
  前清**才是判据，不能凭文件存在判失败。
- **F137-2（低）**：ledger 的 `resume_cleanup` 字段全为 null、`output_tail` 不含 `[清场]`；报告声称
  清理自动执行，但**审计链未记录**每次清场。建议 driver 把 removals 写进 ledger（字段/首行日志保留）。

## 4. 组 D / §5 新证据（独立复算）

- **C1**（`tas-stg08b2-direct`，26 shots）：`6ff345dc…` 命中 **f33000/34500/36000/37500/39000 共 5 帧**，
  全部精确等于 `l4_ending-4/5` ✅。
- **C3**（`tas-stg08-direct`，80 shots）：`6ff345dc…` 命中 **59 帧，f33000→120000 每 1500f 一帧**
  （min33000/max120000/interval 1500）✅。
- **C2**（`tas-stg07-relay`，80 shots）：`6ff345dc` **0 命中** ✅。
- **stg08-timeline**（125 shots）：四联 `62a8ef06…` 命中 **64 帧 f16500–33900**；#17 `ddb322ac…`
  **f14600 唯一**；#18 `2824cb13…` **f16000 唯一** ✅。
- **剩余 4 blob**：`14802bcf/19b88b1f/0f8e1169/f319f1fe` 在 C1(26)+C2(80)+C3(80)+stg08(125) **全零命中** ✅。
- ⇒ 报告 §5 的 **2 sourced（ending-4/5，assisted）+ 4 blocked（fade/ending-1/2/3）** 成立；
  C1/C3 的 bot+assist trigger 已如实标注，未冒充 natural。
- **F137-4（信息）**：task-137 卡里"gs5 anchor f2400 与 c1 f2400 不再同图"的例子**不成立**；
  实测两者同为 `9ff234e6`（f2400 gate-inject/clear-entry 同帧同图），报告 §6 已正确更正。
  c1/c2 的分叉从 f3300 起，trace sha 不同（`4c67…` vs `b8c1…`）。

## 5. 失败/重跑清单（本卡核心不通过项）

`analysis/repro/recapture-gui/ledger.jsonl` 共 **29 行**，不止报告选择的 15 条最终 valid：

- **row5 gameplay-stage1 attempt1**：`rc=-1`（4294967295）、`trace_lines=826`、`done=None`、
  `run_valid=false`、`assert_pass=false` —— 一次**失败 run**；
- **row3 title-menu attempt2**：trace `57e25e3d…`（resume/旧状态污染），run 本身完成但 `assert_pass=false`；
- row1/row2 title 早期尝试 c4d6；row4 c4d6；row11/12 c1/c2 早期 trace 与最终不同
  （c1 `a305…` vs 最终 `4c67…`；c2 `afd2…` vs 最终 `b8c1…`）；
- 最终 15 条 valid 是**在清理/重跑后**挑选的；报告 §0/§7 写"失败：无；无重跑需求"与 ledger 矛盾。
- **F137-1（P1，阻塞报告验收）**：必须补"attempts/失败/重跑"表：失败原因（疑似 Mesen 挂起/超时、
  resume 污染）、被替代 trace sha、最终重跑选择规则；不能只列最终 15 条。
  （备注：失败 run 的 `output_tail`/`mesen_log_tail` 在 ledger 中为空，需从 l4 stdout/临时日志补证。）

## 6. 报告口径/设置

- 每场景最终 run 的 `rc=0 ∧ done!=null ∧ ¬timed_out ∧ trace_lines==done.trace_lines ∧
  frame_errors=0`：**15/15 最终 run PASS** ✅；此点报告没夸大。
- `superseded`：报告 §10 明确旧 headless 结论（task-106/126/130）需标 superseded；本身满足"注明"，
  但实际旧文档的 dated 更正仍需另卡执行。
- settings **task-133 交付时**（`settings-restore.json`）sha256
  `f9c3ffc52deab8e74a76e43748a7d79fb505dbacbd4792bf2acc1c5ccacd7f41`：`EmulationSpeed=100`、
  `AllowIoOsAccess=true`、`EnableAutoSaveState=false`、`EnableRewind=false`；与报告 §8 的 after sha 完全一致 ✅。
- **复核时点（21:14+）当前 settings 已变**：sha `42bdec66…`、`EmulationSpeed=1000`；原因是 l4 新
  `tas-stg08b2-dense` GUI run（Mesen PID31364，21:14:56 启动，`/DoNotSaveSettings`）批次再次提速。
  这不是 task-133 的报告错误，但说明 **settings 最终恢复必须在所有批次（含后续 dense/P0-0-2）结束后
  再核**；task-137 的 settings 验收应写“最终态”，不能只看 task-133 交付时的 f9c3。
- 备份 `settings.json.bak-batch`、`.bak-capturepollution` 存在；当前 `.rgd` 存在属运行后写回（见 §3）。

## 7. findings（四分类）

### 阻塞（P0）
- 无（数据主体可用）。

### 重要（P1）
- **F137-1**：报告"失败：无；无重跑"与 ledger 29 行（含 rc=-1 失败、57e25e3d resume 污染、c1/c2
  早期 trace）不一致；需补失败/重跑清单后复审。**这是本卡 needs_revision 的唯一阻塞项。**

### 次要（P2）
- **F137-2**：resume 清理未进 ledger（字段 null/output_tail 无 `[清场]`）；我独立 2x 验证日志存在，
  但报告审计链不完整。
- **F137-3**：title-menu 无 controller watch；报告用 shots 语义兜底正确，但 task 卡要求"controller 非0"
  在该场景不可观测，建议卡面改用 gameplay 或增加 watch。
- **F137-4**：task 卡"f2400 anchor/c1 不再同图"例子错误；报告 §6 的更正正确，应同步卡面/Lead 口径。
- **F137-8**：settings 在 task-133 交付时为 `f9c3…/100`，但复核时 l4 新 dense 批次已改回 `42bdec…/1000`；
  task-137 点5 的"恢复 100"必须等**所有**批次完成后核最终态（并记录 sha），不能以本报告交付时点代替。

### 建议（P3）
- **F137-5**：把 ledger 的 attempt/失败记录做成 driver 标准输出（不只最终 15 条），以后复核可直接判定。
- **F137-6**：`RecentGames/.rgd` 每次 spawn 清 + 运行后写回；在报告/知识库固定"看 `[清场]` 日志而非
  当前文件"的审计口径。
- **F137-7**：settings 已恢复 100；建议批次报告把 `settings_after_sha` 与 `EmulationSpeed=100` 作为
  一票否决项写入后续任务模板。

## 8. 命令/证据摘要

```text
输入：gameplay held=1049/pressed=195/action distinct4/player_x range243
      anchor held=1094/stage distinct2；full held=12601；c1=1158/c2=1093
      C1 held=22511/stage3；C2=30374/stage3；C3=22512/stage3
两跑：title c4d6a83f（trace+shots 同）；gameplay 0fdd2282；anchor afd2fc1f（3/3）
独立 title GUI 6 连跑均 c4d6a83f；2 次 stdout 均含 1 行 [清场] 删除 Mesen2 resume 状态
组D：C1 5 帧(33000..39000)=6ff345dc；C3 59 帧(33000..120000, step1500)；C2 0
     stg08 四联 64 帧 62a8ef06；#17 f14600 ddb322ac；#18 f16000 2824cb13
     剩余 4 blob 在 C1+C2+C3+stg08 125 张全零命中
settings task-133 after sha f9c3ffc5…（speed=100）；复核时 l4 新 dense 批次已为 42bdec66…（speed=1000，需最终再恢复100）
ledger 29 行：含 gameplay rc=-1/826/None、title 57e25e3d、c1/c2 早期 trace；最终 15 valid
docs-links 只读 PASS；未改代码/gates/expected/scenario；未跑 gates check/status/visual
```

## 9. 结论

**needs_revision**：数据主体（输入/两跑/组 D/settings 交付时点）独立复算通过；但报告 **失败/重跑清单不实**
（F137-1），需补 ledger 的 29 行尝试与失败原因/替代关系后复审。F137-2..8 为低优先级证据链/settings
最终态改进（settings 当前被后续 dense 批次改回 1000，须所有批次结束后恢复 100）。

## 点验（2026-10-18，task-140）

> 只点验 task-139 修正项；其余 task-137 已通过。未跑 Mesen 长场景；只读 + sha 复算。

### 1) P1 修正（pass）
- 报告 §0 L14：已改为 **29 条 ledger 记录 / 最终有效 15 run**，并注明含失败/重试/被替代；“最后成功且
  assert_pass、来自 clean 批次”的选择规则写明；与 `ledger.jsonl` **29 行**一致。
- §7 L156-178 attempts 表与 ledger 对齐：
  - title **8 attempts**（早期 4 条 assert_pass=false；v2/v3 的 `57e25e3d` resume 污染；v4 两跑 c4d6）；
  - gameplay **5**（v2 att#1 `rc=4294967295`/trace_lines826/done=None；v4 0fdd）；
  - gs5-anchor **4**、full/c1/c2 **2**，c1 v2 `a3052466`、c2 v2 `afd2fc1f` 被 v4 `4c673738`/`b8c161e1` 替代；
  - 其余 timeline/C1-C3 **各1**；逐项合计=29，最终 15 有效等价于“29 条中最后成功+assert_pass 的 15 条”。
  - **历史失败/被替代 sha 保留，未删除** ✅。
- **F137-2 口径**：报告明确 ledger `resume_cleanup=null`、`output_tail` 无 `[清场]`，是 driver 未采集字段
  （stdout 早期/tail 截断），不是清理未生效；我独立 2x 运行仍见 `[清场]`，与报告说法一致 ✅。
- **F137-8 口径**：报告 §8 已写“各批次结束由 Lead 统一恢复 100、最终核验时点=全部批次结束”，
  task-133 交付时 after sha `f9c3…/100` ✅；点验早期曾看到 dense 进程期间 1000/`42bdec…`（见下），
  **dense 结束/恢复后已回到 f9c3…/100**，最终态符合口径。

### 2) §5.2 dense 数据（pass）
- `analysis/repro/recapture-gui/dense-run.json`：scene `tas-stg08b2-dense`，rc=0，40000/40000，
  `frame_errors=0`，trace sha `06089e78a3074bf4…`，input held=22512/stage distinct=[1,8,9]，wall330.7s；
- 磁盘 `shots/tas-stg08b2-dense_*` = **133 张**；独立扫 4 blob：
  `14802bcf/19b88b1f/0f8e1169/f319f1fe` **零命中**；`6ff345dc` 命中 **f32100–39900 共 27 帧**；
  与报告 §5.2 完全一致 ✅。
- `dense-run.json` 自称 `settings_speed_restored=100`、after sha `f9c3…`；点验早期（dense 进程运行中）
  曾读到 settings 1000/`42bdec…`，**dense 结束后已恢复 f9c3…/100**（见 §4 点验时点），最终一致 ✅。

### 3) §5.3 小 text 4 对（pass）
- `target/p0-0/render/g-manifest-small.json`：4 对，run1/run2 `png_same=true, idx_same=true` **4/4**；
- 新 png sha 与报告一致：`c39bbeffe098`/`7893d0a3475b`/`5737ae009af6`/`1b1940f30bf8`；
- expected diff 像素：**3742 / 3609 / 4382 / 0**，与报告 §5.3 一字不差；gameplay 三对较旧
  headless（10033/7219/6080）下降但仍 >0（P0-1 范围）；title expected_matches=false 但 diff=0
  （PNG 编码差异），报告已如实解释 ✅。
- `g_run1`/`g_run2` 各 12 文件存在；未改 expected/threshold。

### 4) F140-1（settings 时点，已消解 → pass）
- 点验早期（dense GUI 运行中）确实读到 `settings.json` sha `42bdec…/1000`、Mesen PID31364 dense 活跃；
  这是**批次运行期间临时态**，不是报告口径错误。
- dense 结束/恢复后（settings mtime 21:19:57）复核：sha **`f9c3ffc52deab8e74a76e43748a7d79fb505dbacbd4792bf2acc1c5ccacd7f41`**、
  **EmulationSpeed=100**、`AllowIoOsAccess=true`、AutoSave/Rewind=false；当前无 Mesen 进程。
- ⇒ task-140 点2 “settings 最终=100” **通过**；报告 §8 的“当前确认 speed=100”在最终时点成立。
- **建议**（低）：后续批次收尾仍固定“全进程结束后再核 settings sha+speed”，避免像本次点验早期误读临时态。

### 5) docs-links/纪律
- docs-links 只读门 PASS；`git status -- oracle/screens`=0；未跑 gates check/status/visual；
  未改代码/expected/scenario/settings；本文件为唯一追加写入。

### 点验结论
- **P1 修正通过**（29 行 attempts/失败原因/选择规则/历史保留均对齐）；
- **F137-2 通过**；**F137-8 通过**：最终 settings `f9c3…/100`、无残留 Mesen 进程（点验早期 dense
  运行中的 1000/42bdec 是临时态，dense 结束后已恢复，已记录时点）；
- **dense/small text 数据全部复算通过**；docs-links PASS。
- **task-140 结论：pass**（P1/F137-2/dense/small text/settings 最终态均通过；建议后续所有批次收尾固定
  “进程结束后再核 settings sha+speed”）。
