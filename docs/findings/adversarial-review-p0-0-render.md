# 对抗复核：P0-0-2 22 对重渲两跑 + manifest + 同 blob（task-114）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：l4-analyst（task-114）
> 被审：`docs/findings/p0-0-render-run1.md`（151 行，untracked）+
> `target/p0-0/render/{manifest.json,summary.json,run1,run2}`（gitignored）。
> 复核方式：只读 manifest/run 目录，独立 sha256 复算、dedupe 复算、trace/capture 输入对照；
> **未跑 Mesen/scenario_run/gates check/status/visual**；只读 docs-links。本文件唯一写入，未 git add/commit。

## 0. VERDICT

**pass**（2 条中低优先级证据/语义建议；主结果成立）

- 两跑确定性 **16/16 png + 16/16 idx** 独立复算成立；run1/run2 各 16 png/idx/log；manifest 22 行；
  status 分布 **ok 11 / replaced-invalid-asset 5 / blocked:no-source 6 / stale 0**；失败 0。
- `expected_unique=14 (reduction 5)`、`actual_unique=11 (reduction 5)`、expected 3 组/actual 1 组与
  manifest 一致；binary sha `ab115570…`；`oracle/screens` 未被改写（git status 0）。
- **actual 侧 6 对（stg06-late, stg07-combat/start, stg08-combat/late/start）确实同 blob
  `5f93bdbb2c84`**；旧 expected 组四同 blob `62a8ef…`，#17/#18 expected 不同。报告标"待 §7.5/§8 裁定"，
  **没有**把它写成 0 容差通过、没有改 expected/threshold，态度如实。
- **关键补充**：我复核到六帧的 **源 capture shots 全部同 sha `2b2d6ab04350`（6240B）**，而 trace 在六帧
  不同——这**强支持"同一 OBS-1 冻结屏"**，即 actual 同 blob 不是渲染器凭空丢输入；但旧 expected
  #17/#18 不同，仍不能据此裁定 expected/actual 哪一侧有效；报告"待裁定"是合理的，建议把 capture 同图
  证据补入（F125-2）。
- 报告未超证据：未改 expected、未声称组 D 有源、未声称 0 容差通过；`blocked:no-source` 6 对如实。

## 1. 两跑/文件核验 ✅

独立读取 `target/p0-0/render/manifest.json` + run1/run2：

```
run1/run2 各：16 *.png、16 *.idx.json、16 *.log
16/16 对 png sha256 相同；16/16 对 idx sha256 相同；
每对 run1/run2 png = manifest actual_run1/run2_sha256，idx = actual_idx_run1/2_sha256；
deterministic_ok=16 / fail=0；summary.json 与 manifest.summary 一致。
```

- `manifest.pairs` **22 行**；status 计数 `{ok:11, blocked:no-source:6, replaced-invalid-asset:5}`；
  `failed_or_blocked_render=0`、`skip:stale-input` 无。
- render binary sha `ab115570b8ebaf8c…520f4`，与报告 §1 一致。
- **F125-3（低）**：报告总耗时 3976.6s **不能从 manifest 复算**——manifest `wall_s` 16 对两跑合计仅
  **3153.11s**；3976.6−3153.11≈823.5s 应为 phase-batch 3 场景耗时，但 manifest/报告没有单独机器字段；
  建议 manifest 记 `phase_batch_wall_s`/`total_wall_s`（或在报告给 phase-batch 日志）。
- **F125-4（低）**：报告写"39 个诊断 env 全部 unset"，但 manifest `env` 只有 9 个键，`render_env` 是一句
  中文说明，**没有 39 项清单**；建议存 unset 名单/计数（可复核），否则该数字只能人工信任。

## 2. status/dedupe/expected 未改 ✅（但 status 语义需注明）

- dedupe 独立复算：
  ```
  expected: 22 png → unique 14，3 个重复组（6ff345…×2、788d69…×2、62a8ef…×4）⇒ reduction 5 按 Σ(len−1)
  actual:   16 png → unique 11，1 组六同 blob 5f93bd…（reduction 5）
  ```
  manifest `expected_unique=14/expected_reduction=5/actual_unique=11/actual_reduction=5` 与报告一致。
- expected 组：
  - ending-4/5 = 6ff345dc5c3f；
  - stg02-start/title = 788d6984576b；
  - stg06-late/stg08-combat/late/start = 62a8ef06c4b9；
  - actual 六组 = 5f93bdbb2c84。
- **旧 expected 未改写**：`git status --porcelain -- oracle/screens` = **0**；报告未改 expected/threshold。
- **F125-1（中，语义）**：manifest 中有 **13 个 rendered pair 的 actual != expected**（含 `status=ok` 的 8 个：
  l4_clear-inject / l4_combat-variety / l4_late-combat / l4_shop-window / l4_stg03-start /
  l4_stg07-combat / l4_stg07-start / title-menu）；若按手册 §5 `status=ok`（映射确认/expected==actual）语义，
  这些行不应是 `ok`。报告 §2 写"status 规则"是 P0-0-2 的**暂定分类**，并不是 §3.3 校验后的 mapping 状态；
  建议将字段明确标为 **render_status** 或注明"P0-0-2 provisional，ok≠blob confirmed"，避免后续 wrapper 把
  `ok` 当映射确认。
- 状态分布本身与 task 卡/manifest 一致；`replaced-invalid-asset` 5 对（#14 + 四联）与报告说明一致。

## 3. 关键 actual 同 blob 发现（证据核验）

独立 sha256（六对 actual run1 = run2）：
```
l4_stg06-late   = 5f93bdbb2c84612b218a3b196e4034284d5826fc59ec7d3d77e11aa328cff046
l4_stg07-combat = 5f93bdbb2c84…（同）
l4_stg07-start  = 5f93bdbb2c84…（同）
l4_stg08-combat = 5f93bdbb2c84…（同）
l4_stg08-late   = 5f93bdbb2c84…（同）
l4_stg08-start  = 5f93bdbb2c84…（同）
```
- 旧 expected：stg06-late / stg08-combat / stg08-late / stg08-start = `62a8ef06c4b9`；stg07-combat =
  `2824cb134987`；stg07-start = `ddb322ac5702`（后两与 actual 不同）。
- **新增独立证据（报告未列）**：六帧源 capture：
  ```
  shots/stg08-timeline_f14600_stg07-start-approx.png  sha12 2b2d6ab04350
  shots/stg08-timeline_f16000_stg07-combat.png        sha12 2b2d6ab04350
  shots/stg08-timeline_f16600_stg08-start-approx.png  sha12 2b2d6ab04350
  shots/stg08-timeline_f18000_stg08-combat.png        sha12 2b2d6ab04350
  shots/stg08-timeline_f22000_stg06-late.png          sha12 2b2d6ab04350
  shots/stg08-timeline_f28000_stg08-late.png          sha12 2b2d6ab04350
  （均 6240B；trace 六帧 sha 各不相同：ceba0e/5fedf2/c643a9/e8e5ee/1d4ca5/b1b3a8…）
  ```
  ⇒ **源画面本身六帧同图**，actual 同 blob 与源 OBS-1 冻结屏一致，**不是渲染器独立缺陷**的充分证据；
  但 #17/#18 旧 expected 不同，说明 expected 与源 capture 不一致（expected 可能过期/来自别的帧/资产），
  **仍不足以裁定 expected 哪侧正确**——报告"待 §7.5/§8 裁定"成立。
- **F125-2（低/中）**：报告只引 expected 同 blob 与旧说明，未列上述 capture 同图证据；建议补入，能显著
  把"OBS-1 解释"从 B 级提高到 **capture A 级旁证**（但仍不等于 expected 有效裁定）。

## 4. 报告证据边界

- 报告 §2 明确 actual/expected sha、§4 明确 reuse/duplicate 待 mapping/§7 回填；§5 把 actual 同 blob
  列 **待裁定**，不冒充结论；组 D 6 对 blocked:no-source；未声称 expected 重采已完成。
- 未改 expected/threshold、未声称组 D 有源；符合"不超证据"。
- **F125-5（信息）**：§5 表 actual 同 blob 写"#16-#21 六对"，但 status 分布 #17/#18 为 ok、其余
  replaced；实际它们与 expected 均不同；建议同表标出每对 expected sha/差异，避免 reader 以为只有 4 对
  expected 不匹配。

## 5. 门禁/纪律

- **未跑** Mesen/scenario_run/gates check/status/visual；只读 `gate_docs_links()`：
  **PASS 368 md/454 links**。
- `git status -- oracle/screens` = **0**；报告未改写 expected；本复核只写本文件。
- 产物均在 `target/`（gitignored）；报告 untracked；未 git add/commit。

## 6. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- **F125-1（中，语义）**：manifest `status=ok` 含 8 个 actual≠expected 的 pair；若 §5 `ok=映射确认/
  expected==actual`，则状态不成立。报告声明了 P0-0-2 暂定分类且 expected 重采未定稿，但建议明确字段为
  `render_status`/加"provisional, not blob confirmed"，避免后续误用。非本轮渲染/确定性失败，属语义澄清。

### 次要（P2）
- **F125-2（中低）**：actual 同 blob 的证据缺源 capture 同图对比；本复核已补（六帧 shot 同 sha
  `2b2d6ab04350`），建议写入报告；仍保留待裁定（expected #17/#18 不同）。
- **F125-3（低）**：总耗时 3976.6s 不可由 manifest 复算（pair wall sum 3153.11s；缺 phase-batch wall
  字段/日志）。
- **F125-4（低）**：env "39 项 unset" 无机器清单（manifest env 仅 9 键 + 一句 prose）；建议存列表/计数。
- **F125-5（低）**：§5 actual 六对与 expected 差异建议逐对列 expected sha，避免 status 分布被误读。

### 建议（P3）
- **F125-6**：dedupe 输出（expected/actual 组、reduction 公式）建议落 `analysis/repro/dedupe.tsv`，便于
  wrapper §5.3 直接消费；本卡手工复算已一致。
- **F125-7**：actual 同 blob 裁定卡应同时核 **源 capture 六张同图 + expected 旧件来源**；若 expected 过期，
  按 §7.2 重采而不是把 actual 当缺陷。

## 7. 命令/证据摘要

```text
manifest: 22 pairs；16 render；status {ok:11, blocked:no-source:6, replaced-invalid-asset:5}
run1/run2：16 png + 16 idx + 16 log；16/16 png sha 相同；16/16 idx sha 相同；0 fail
binary sha256 ab115570b8ebaf8c…520f4；repo HEAD dabd6afd…
expected_unique=14（reduction 5）：groups 6ff345×2 / 788d69×2 / 62a8ef×4
actual_unique=11（reduction 5）：5f93bdbb2c84 ×6
actual 六对 sha 均 5f93bdbb2c84；expected 四 62a8ef、stg07-combat 2824cb、stg07-start ddb322
源 capture 六帧 shots 均 2b2d6ab04350（6240B）；trace 六帧 sha 不同
actual≠expected rendered pairs = 13（其中 status=ok 8）
pair wall sum=3153.11s；报告 total 3976.6s（含 phase-batch，缺机器字段）
docs-links PASS 368 md/454 links；oracle/screens status 0；未跑 Mesen/gates check/status/visual
```

## 8. 结论

**pass**：两跑确定性、manifest 22 行/status/dedupe、binary sha、actual 6 同 blob 与旧 expected 未改
均独立核实；报告把 actual 同 blob 标"待裁定"、未超证据。F125-1 建议澄清 `status=ok` 的 provisional 语义，
F125-2 建议补源 capture 同图证据（本复核已发现），F125-3/4/5 为低优先级补充；不阻塞 P0-0-2 提交，但
F125-1 建议在 wrapper/后续 mapping 透传前明确。
