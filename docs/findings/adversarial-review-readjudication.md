# 对抗复核：P0-0-3 22 对组 A–E 重裁（diff 抽样/统计/去重/手册同步）——adversarial-review-readjudication

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用 l4-analyst 自述**）
> 被审：`docs/findings/p0-0-readjudication.md`（**128 行，untracked**，task-130）；手册 `l4-baseline-freeze-plan.md` 工作树 M（§7.3/§7.4/§7.5/§8.2 dated 更正）；`p0-0-same-blob-adjudication.md` 工作树 M（F128-1/2 点改）。
> 纪律：**未跑 Mesen/gates check/status/visual**；只读 `target/p0-0/render/run1`、`oracle/screens/expected`、`target/p0-0/render/manifest.json`、`analysis/repro/p0-0-readjudication/readjudication.json` + `tools/diff_visual.compare` 复算。本报告只写该文件、未 git add/commit。

---

## 0. VERDICT：**pass**（核心数字/抽样/统计/去重/分组全部独立复算一致；2 条低风险文档措辞/历史块建议，不阻塞 P0-0-3 提交）

- **差异复算 PASS**：我用 `tools.diff_visual.compare`（`TRUE_COLOR=True`、256×240、逐像素 RGB 0 容差、无白名单）独立复算 **全部 16 可渲对**，diff_pixels / ratio / bbox **与 readjudication 表/JSON 全 16/16 一致**；任务建议的 **#4/#13/#22**：`58364/94.99%/0,0,255,239`、`18241/29.69%/0,0,255,234`、`0/0.00%/None` 全部命中。
- **统计 PASS**：16 可渲 / 6 blocked；**0 差异 1/16**（title `f00180_boot-black`：diff=0，actual sha `1b1940f3…` ≠ expected `788d6984…`，PNG 编码差，严格 0 容差 PASS）；15/16 diff>0：min **2918**、max **58364**；`replaced-invalid-asset` **5 对**（#14=4476，#16/#19/#20/#21=18887×4）；`blocked:no-source` 6 对组 D（无 actual 文件）。
- **去重 PASS**：直接扫文件 sha：expected_unique=**17**（3 组：ending-4/5×2、stg02-start↔title×2、四联×4）、actual_unique=**11**（1 组×6 = `5f93bdbb2c84…`，OBS-1 冻结屏 documented_duplicates，不剔除）；manifest `dedupe` 与实扫一致。
- **分组/状态 PASS**：A 3 / B 6 / C 6 / D 6 / E 1；D 6 对 `render=false` 且 run1 无对应 png；同 blob actual 6 对名单与 manifest 完全一致；两跑 png/idx **16/16 同**（直接文件 sha）。
- **手册/同 blob 同步 PASS（正文点）**：§7.3/§7.4/§7.5/§8.2 的 task-130 dated 注与 readjudication 一致、历史原文保留；"P0-0 不得宣称全绿"在位；same-blob F128-1/2（compare.json 格式/命令、51/45 静态组）实测可复算。
- **未改 expected/threshold/code/gates**：`git status -- oracle/screens/expected` 空；工作树仅 3 个 docs 改动（readjudication untracked + manual + same-blob）；docs-links PASS（374 md/459 links）。

**低风险建议（不阻塞）**：
- **F1（低，统计口径措辞）**：`median=12,854.5` 是 **16 可渲（含 1 个 0 差异）** 的中位；若严格按"**15 个 diff>0**"算，中位是 **13,501**（12208 与 13501 之间）。readjudication §0 L16/§3 L74 与 manual §8.2 L1683 都写"15（个 diff>0）…median 12,854.5"，建议注明"16 可渲含零差异"或分别给两个口径；值本身与任务卡期望 12,854.5 一致。L74 还有未填的 `Q1?` 占位。
- **F2（低，手册历史块）**：§7.5 旧 B2 代码块（L1425-1431）仍用 `oracle/screens/l4-manifest.json` 并断言 `len(E)/len(A) in (21,22)`——与紧邻的任务-130 实况注（expected_unique=17、actual_unique=11）**数值口径冲突**；若该块属历史判据，建议显式标"历史/被 task-130 注替代"，否则按 17/11 更新。正文 note（L1441-1447）本身正确。
- **F3（info）**：任务卡标题写"组 A–F"，但 22 对/manifest/readjudication 实际只有 **A–E**（3+6+6+6+1=22），无 F 组；本报告按实际 A–E 复核，未把 F 当作缺失。
- **F4（info）**：readjudication/same-blob 自查 checkbox `[ ] docs-links PASS` 未勾选；外部复扫为 PASS，提交前可勾选。

---

## 1. 方法/口径（A）

- `tools/diff_visual.py::compare(exp, act)`：`decode_png` 后要求 **256×240**；`TRUE_COLOR=True`（L95）逐像素 RGB 全等计数；**无白名单、无阈值、无缩放**——与 readjudication §1 声明一致。
- expected 路径取 `manifest.pairs[].expected_file`；actual 取 `target/p0-0/render/run1/<name>.png`；run1/run2 png+idx 文件 sha 直接比较。
- 抽样按任务建议 **#4 l4_clear-inject / #13 l4_shop-window / #22 title-menu_f00180_boot-black**；随后扩展到全部 16 对（避免只抽样漏错）。
- 独立复算输出与 `analysis/repro/p0-0-readjudication/readjudication.json` **逐对 diff/ratio/bbox 全等**（16/16）。

## 2. 抽样 + 全量 diff（A）

| # | name | 期望/实际 diff | ratio | bbox | readjudication 同值? |
|---|---|---:|---:|---|---|
| 4 | `l4_clear-inject` | **58,364** | 94.99% | `0,0,255,239` | ✓ |
| 13 | `l4_shop-window` | **18,241** | 29.69% | `0,0,255,234` | ✓ |
| 22 | `title-menu_f00180_boot-black` | **0** | 0.00% | None | ✓（expected sha 788d6984… ≠ actual sha 1b1940f3…，像素全等） |

- 全量 16 行输出与 doc §2 / JSON 完全一致；样例另核对：
  - #14 `l4_stg02-start` diff=4,476 / 7.29% / bbox `0,41,255,239`；
  - #16/#19/#20/#21 四对 diff=18,887 / 30.74% / `0,0,255,239`；
  - #15 `l4_stg03-start` 2,918 / 4.75%（min）；#1 10,033 etc.
- **差异率分母=61440**（256×240）；doc 表与 JSON 一致。
- **0 容差**：title pair 严格 PASS；不是 sha 相等，但像素级 0 差异（编码差）——doc 明确标注，未冒充 blob 相等。

## 3. 统计复算（A）

| 项 | 独立值 | doc |
|---|---|---|
| 22 对 | 22 | 22 |
| 可渲 / blocked | 16 / 6 | 16/6 ✓ |
| 0 差异 | 1（title） | 1/16 ✓ |
| diff>0 | 15 | 15/16 ✓ |
| min / max | **2918 / 58364** | 2918 / 58364 ✓ |
| diff 分布中位 | **12,854.5（16 可渲含 0）**；**13,501（仅 15 个正值）** | doc 写 median 12,854.5 但称"15 个 diff>0"（F1 口径措辞） |
| replaced-invalid-asset | 5（4476 + 18887×4） | 5 ✓ |
| blocked 6 | D #6-#11，render=false，无 png | ✓ |
| 两跑确定性 | png 16/16、idx 16/16 | manifest 16/16 ✓ |

> 15 个正值排序为 `[2918, 4476, 5527, 6080, 7219, 10033, 12208, 13501, 18241, 18887×4, 28563, 58364]`；第 8 个=13501。16 个含 0 后中位=(12208+13501)/2=12854.5。**数值 12,854.5 对应"16 可渲"口径，不是"15 个 diff>0"口径**（F1）。

## 4. 分组与去重（A）

- 组计数：A 3 / B 6 / C 6 / D 6 / E 1；manifest 与 readjudication 一致。
- 抽样组判定：
  - #13 `l4_shop-window` 属 **B**（scene stg03-timeline；expected 988809e0…，actual b560e275…，diff18241）；
  - #17 `l4_stg07-combat` 属 **C**（scene stg08-timeline；actual=5f93bdbb… 六组之一，expected 2824cb13… 不同 blob，diff12208）；
  - #16/#19/#20/#21 C 组 replaced-invalid-asset（expected 四联同 blob 62a8ef06…，待重采）。
- **D 6 对**：manifest `render=false/status=blocked:no-source`；run1 无 `l4_ending*.png`/`l4_fade-clear*.png`；不 staging/不渲染 ✓。
- **同 blob actual 6 对**：直接文件 sha 全为 `5f93bdbb2c84…`（`l4_stg06-late,l4_stg07-combat,l4_stg07-start,l4_stg08-combat,l4_stg08-late,l4_stg08-start`）⇒ documented_duplicates 登记、不剔除；expected 侧 3 个重复组（ending-4/5、stg02↔title、四联4）也一致。
- **去重数**：直接扫 manifest 22 名对应 expected 文件 = **17 unique**；16 rendered actual 文件 = **11 unique**；expected reduction5 / actual reduction5；与 doc §3/manifest 一致。

## 5. 手册同步与 same-blob F128 点改（A/B）

- §7.3（ending 五帧）：保留原动作并加 task-106/113 + task-130 dated 注——组 D `blocked:no-source`、不重渲/不 staging；§7b actual 核对当前无 actual；"P0-0 不得宣称全绿"在位（L1364-1366）✓。
- §7.4（fade-clear）：task-130 注 `#11 维持 blocked:no-source`、旧 expected 14802bcf… 保留（L1393-1394）✓。
- §7.5（去重）：task-130 注 expected_unique=17（3 组）/actual_unique=11（actual 6-group OBS-1 documented_duplicates；不剔除；registry 扩展）与 readjudication 一致（L1441-1447）✓；历史 B2 代码块口径冲突见 F2。
- §8.2（验收）：task-130 注 16 可渲 + 1 像素 0 差异 + 15 diff>0 min/median/max + 6 blocked + 5 replaced + dedupe17/11；"本清单在组D来源+expected5对重采前维持不通过；P0-0不得宣称全绿"（L1682-1688）✓；median 措辞同 F1。
- **same-blob F128-1/2**：
  - F128-1 compare.json 实际为 `{frame:{rc,exists,sha}}`（`target/p0-0/adjudicate/compare.json` 六帧；f5000 不同、f10000-34000=5f93bdbb… 同），命令注释与该格式一致 ✓。
  - F128-2 静态组：直接从 `shots/stg08-timeline_f*` 扫：`2b2d6ab04350…` **51 张**（9000–33600）、`d531d33a65d3…` **45 张**（7500–33900），另有小重复组（988809e0…×3 等）——与 doc 名单/命令一致 ✓。
- 历史保留：§7.3 旧标题/§7.4 旧内容、§3.6 旧判据均仍在（dated 注而非删除），符合 AGENTS §3.6。

## 6. Findings 四分类

### 6.1 证据/口径（低）
- **F1（低）**：median 12,854.5 实为 **16 可渲含 0 差异**中位；按 doc 文字"15 个 diff>0"算应为 **13,501**。建议 readjudication §0 L16/§3 L74、manual §8.2 L1683 统一口径（"16 可渲含 1 个 0" 或并列 13,501/12,854.5）；另 L74 `Q1?` 未填。
- **F2（低）**：§7.5 旧 B2 代码块（L1425-1431）`len(E)/len(A) in (21,22)` 与旧 `oracle/screens/l4-manifest.json` 路径与 task-130 注的 17/11 不一致；建议标历史/替换。
- **F3（info）**：任务卡"组 A–F"与文档 A–E 不一致；22 对无 F 组（应为 A3+B6+C6+D6+E1）。

### 6.2 统计/实现（无）
- 所有 diff/统计/去重/组别/同 blob/两跑确定性均独立复算一致；replaced/blocked 状态正确；未放宽 0 容差、未启用白名单。

### 6.3 文档/格式（info）
- **F4**：readjudication/same-blob 自查 `[ ] docs-links PASS` 未勾选；提交前可更新（外部扫 PASS 374 md/459 links）。
- manual/readjudication 对 C1/C2 的旧 BLOCKED/DEGRADED 描述均为 dated 历史保留；task-130 注已反映 P0-0-2 重跑；不误导。

### 6.4 越权/伪造（无）
- 未改 expected/threshold/code/gates；工作树仅 3 docs 改动（readjudication untracked + manual/same-blob M），`oracle/screens/expected` 无变化；本报告只新增文件、未 git add/commit。

## 7. 命令与输出摘要

```text
# 独立 diff（全部 16 可渲；TRUE_COLOR=True, 256×240, 0 容差）
l4_clear-inject             58364 / 0.9499 / [0,0,255,239]
l4_shop-window              18241 / 0.2969 / [0,0,255,234]
title-menu_f00180_boot-black    0 / 0.0000 / None
l4_stg02-start               4476 / 0.0729 / [0,41,255,239]
l4_stg06-late               18887 / 0.3074 / [0,0,255,239] （同 #19/#20/#21）
l4_stg03-start               2918 / 0.0475 / [0,0,255,234]（min）
l4_clear-inject             58364（max）
全 16 对 diff/ratio/bbox 与 readjudication.json 全等。

# 统计
16 可渲 / 6 blocked；0 差异 1/16；diff>0 15/16
min=2918；median(all16)=12854.5；median(15 positive)=13501；max=58364
replaced 5：4476 + 18887×4
run1/run2 png 16/16、idx 16/16 同

# 去重/分组（直接扫文件）
expected_unique=17（3 组）；actual_unique=11（6-group 5f93bdbb2c84）
A3/B6/C6/D6/E1；D run1 无 png、manifest render=false/blocked:no-source

# same-blob F128
compare.json {5000:不同, 10000/14600/22000/28000/34000:5f93bdbb2c84}
shots/stg08-timeline_f*：2b2d6ab04350… 51 张；d531d33a65d3… 45 张（另有小重复组）
docs-links PASS（374 md/459 links）
```

## 8. 复核边界

1. 未跑 Mesen/gates check/status/visual；未重渲任何实际图；所有 diff 用 `tools.diff_visual.compare` 在 gitignored 产物上复算。
2. 未修改 expected/threshold/manifest/readjudication/manual/same-blob 文档；本报告唯一新增文件。
3. `target/p0-0/render/run1` 为 P0-0-2 当前产物；manifest 22 名过滤后 expected17/actual11；旧 `oracle/screens/actual` 127 unique 未用于本轮（doc F2 已声明不混用）。
4. F1/F2 为低风险文档口径/历史块建议，不改变 P0-0-3 的 22 对状态或 `blocked:no-source`/replaced 结论。

---

【2026-10-18】decomp-verifier，独立复核 task-130（HEAD `7d7deca`；readjudication 128 行 + manual/same-blob M）；diff 全 16 对、统计/去重/分组/同 blob/手册点改复算；P0-0 不得宣称全绿。
