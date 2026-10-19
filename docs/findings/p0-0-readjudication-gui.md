# P0-0-3（GUI trace 版）：22 对组 A–E 重裁（task-143）

> 日期：2026-10-18｜角色：l4-analyst｜输入：**GUI 重采 trace**（task-133，input-effective）+ **task-138 新二进制** `00c29d6a31c0f885f447476cbdb54b0919d6501caab0f6aeb0d3ccb1e1588b48`；
> actual = `target/p0-0/render/g_run1`（与 `g_run2` 两跑 36/36 png+idx 相同）；expected = `oracle/screens/expected/`（旧件，未改）；
> 差异口径 = `tools/diff_visual.compare`（**256×240 真彩域、0 容差、不放宽**）；不跑 gates check/status/visual；不改 expected/threshold/代码。

## 0. 结论摘要（先读）

| 项 | 值 |
|---|---|
| 22 对口径 | **18 可渲**（16 canonical + **ending-4/5 2 对 assisted 来源**）+ **4 `blocked:no-source`**（fade-clear/ending-1/2/3）；**5 对 `replaced-invalid-asset`**（#14 stg02-start + 四联 #16/#19/#20/#21；expected 待 §7.2 重采） |
| 两跑确定性 | 18/18 对 png+idx 相同（36/36 渲染） |
| diff 统计（18 对） | **min 0**（title，像素级 0）/ **median 9,766** / **max 58,159**（clear-inject）/ **sum 242,555 px**（全 61440px/对） |
| actual==expected sha | **0/18**（旧 expected 属另一输入域；L4 0 容差**未达成**，属 P0-1 目标） |
| 组 D | **2 sourced（#9/#10 ending-4/5，assisted）+ 4 blocked**（dense 133 张 any-frame 零命中，负证据） |
| 去重（18 rendered） | **expected_unique=13**（title↔stg02、四联×4、ending-4/5）；**actual_unique=14**（四联×4、ending-4/5） |
| **L4 / P0-0 可宣称通过？** | **否**：0/18 actual==expected、4 对无源、**5 对 expected 待重采**、18 对 diff>0；**不得宣称 L4 通过 / P0-0 全绿** |

## 1. 方法与口径

- actual：`g_run1/<name>.png`+`.idx.json`（run2 同 sha；manifest `target/p0-0/render/g-manifest-v2.json`，gitignored）；
- expected：旧件按 `<name>.png`（group A 三对与 title 用带 tag 文件名）；只读，未 staging/未替换；
- diff：`tools.diff_visual.compare(expected, actual)` → 逐像素 RGB 不等计数（0 容差，未启用白名单）；bbox=差异像素包围盒；
- 分类：`diff>0`=渲染差异（P0-1）；`pixel-equal`=0 像素差异；`expected-invalid`=expected 资产无效待重采；`blocked`=无源不渲。

## 2. 逐对重裁表（22 行；diff = 像素差异数 / 61440；bbox=[x0,y0,x1,y1]）

> **编号注（AR144-2）**：本表 `#` 列为**行序**（按组 A–E 重排），非手册 canonical pair#；canonical 对应：
> row9=#14 stg02-start；row11=#16 stg06-late；row14=#19 stg08-combat；row15=#20 stg08-late；row16=#21 stg08-start（四联）。

| # | name | frame | scene | 组 | source | render/readjudication | expected sha（前 12） | actual sha（前 12） | diff 像素 | 差异率 | bbox | 分类 | 处置 |
|---:|---|---:|---|---|---|---|---|---|---:|---:|---|---|---|
| 1 | `gameplay-stage1_f01260` | 1260 | gameplay-stage1 | A | natural | ok | `38365db2cb4f` | `c39bbeffe098` | 3742 | 0.0609 | 0,8,255,239 | diff>0 | P0-1 差异；actual 冻结 |
| 2 | `gameplay-stage1_f01500` | 1500 | gameplay-stage1 | A | natural | ok | `7f4ac8a23511` | `7893d0a3475b` | 3609 | 0.0587 | 0,7,255,239 | diff>0 | P0-1 差异；actual 冻结 |
| 3 | `gameplay-stage1_f02349` | 2349 | gameplay-stage1 | A | natural | ok | `4274a33f45f5` | `5737ae009af6` | 4382 | 0.0713 | 0,8,255,239 | diff>0 | P0-1 差异；actual 冻结 |
| 4 | `title-menu_f00180_boot-black` | 180 | title-menu | E | natural | ok | `788d6984576b` | `1b1940f30bf8` | 0 | 0.0000 | — | pixel-equal | 像素级 0 差异（sha 不同=PNG 编码） |
| 5 | `l4_clear-inject` | 4600 | stg03-timeline | B | injected | ok | `5cacbe294d53` | `246dd5712608` | 58159 | 0.9466 | 0,0,255,239 | diff>0 | P0-1 差异；actual 冻结 |
| 6 | `l4_combat-variety` | 6500 | stg03-timeline | B | injected | ok | `6f7fe4992e59` | `5af017eb7284` | 28260 | 0.4600 | 0,0,255,239 | diff>0 | P0-1 差异；actual 冻结 |
| 7 | `l4_late-combat` | 12000 | stg03-timeline | B | injected | ok | `c0baa0f8d21b` | `64d148fe2ede` | 6118 | 0.0996 | 0,0,255,238 | diff>0 | P0-1 差异；actual 冻结 |
| 8 | `l4_shop-window` | 5400 | stg03-timeline | B | injected | ok | `988809e0b414` | `3689760a47f5` | 19040 | 0.3099 | 0,0,255,221 | diff>0 | P0-1 差异；actual 冻结 |
| 9 | `l4_stg02-start` | 3404 | stg03-timeline | B | injected | replaced-invalid-asset | `788d6984576b` | `ebc6372f79d9` | 2495 | 0.0406 | 1,0,254,231 | expected-invalid(+diff>0) | expected 待 §7.2 重采；actual 已渲 |
| 10 | `l4_stg03-start` | 5700 | stg03-timeline | B | injected | ok | `24a4cb45c0d8` | `3f6272110fc0` | 2261 | 0.0368 | 0,1,255,234 | diff>0 | P0-1 差异；actual 冻结 |
| 11 | `l4_stg06-late` | 22000 | stg08-timeline | C | natural | replaced-invalid-asset | `62a8ef06c4b9` | `406f3962559c` | 18681 | 0.3041 | 0,8,255,239 | expected-invalid(+diff>0) | expected 待 §7.2 重采；actual 已渲 |
| 12 | `l4_stg07-combat` | 16000 | stg08-timeline | C | natural | ok | `2824cb134987` | `65de647de951` | 11105 | 0.1807 | 0,0,255,236 | diff>0 | P0-1 差异；actual 冻结 |
| 13 | `l4_stg07-start` | 14600 | stg08-timeline | C | natural | ok | `ddb322ac5702` | `c9070704ca5c` | 11806 | 0.1922 | 0,0,255,239 | diff>0 | P0-1 差异；actual 冻结 |
| 14 | `l4_stg08-combat` | 18000 | stg08-timeline | C | natural | replaced-invalid-asset | `62a8ef06c4b9` | `406f3962559c` | 18681 | 0.3041 | 0,8,255,239 | expected-invalid(+diff>0) | expected 待 §7.2 重采；actual 已渲 |
| 15 | `l4_stg08-late` | 28000 | stg08-timeline | C | natural | replaced-invalid-asset | `62a8ef06c4b9` | `406f3962559c` | 18681 | 0.3041 | 0,8,255,239 | expected-invalid(+diff>0) | expected 待 §7.2 重采；actual 已渲 |
| 16 | `l4_stg08-start` | 16600 | stg08-timeline | C | natural | replaced-invalid-asset | `62a8ef06c4b9` | `406f3962559c` | 18681 | 0.3041 | 0,8,255,239 | expected-invalid(+diff>0) | expected 待 §7.2 重采；actual 已渲 |
| 17 | `l4_ending-4` | 34500 | tas-stg08b2-direct | D | assisted | sourced-assisted | `6ff345dc5c3f` | `5aa8a4f8bdc0` | 8427 | 0.1372 | 0,8,255,231 | sourced-assisted(diff>0) | assisted 来源已确认；actual 已渲（不冒充 natural） |
| 18 | `l4_ending-5` | 36000 | tas-stg08b2-direct | D | assisted | sourced-assisted | `6ff345dc5c3f` | `5aa8a4f8bdc0` | 8427 | 0.1372 | 0,8,255,231 | sourced-assisted(diff>0) | assisted 来源已确认；actual 已渲（不冒充 natural） |
| 19 | `l4_fade-clear` | 28500 | — | D | — | **blocked:no-source** | `14802bcf460a` | — | — | — | — | blocked | 不渲/不 staging（dense 133 张零命中） |
| 20 | `l4_ending-1` | 30000 | — | D | — | **blocked:no-source** | `19b88b1ffce9` | — | — | — | — | blocked | 不渲/不 staging（dense 133 张零命中） |
| 21 | `l4_ending-2` | 31500 | — | D | — | **blocked:no-source** | `0f8e11694d53` | — | — | — | — | blocked | 不渲/不 staging（dense 133 张零命中） |
| 22 | `l4_ending-3` | 33000 | — | D | — | **blocked:no-source** | `f319f1fe5256` | — | — | — | — | blocked | 不渲/不 staging（dense 133 张零命中） |

> 说明：`render_status` 沿用 P0-0-2 语义；`source` 为 trigger 分类（natural/injected/assisted）。**`replaced-invalid-asset` 共 5 对**（#14 + 四联 #16/#19/#20/#21，expected 待 §7.2 重采）。#9/#10 的 expected 为旧件（`6ff345dc` 同 blob），
> 来源=C1/C3 bot+assist f33000+（assisted，**不冒充 natural**）。#14 expected=`788d6984`=title boot-black 同图（复制品，invalid）。

## 3. 统计与去重（新事实）

| 组 | 对数 | 可渲 | 0 差异 | diff>0 | blocked | 备注 |
|---|---:|---:|---:|---:|---:|---|
| A | 3 | 3 | 0 | 3 | 0 |  |
| B | 6 | 6 | 0 | 6 | 0 |  |
| C | 6 | 6 | 0 | 6 | 0 |  |
| D | 6 | 2 | 0 | 2 | 4 | ending-4/5 assisted |
| E | 1 | 1 | 1 | 0 | 0 | pixel-equal |
| 合计 | 22 | 18 | 1 | 17 | 4 | — |

- diff 分布（18 对）：min **0** / median **9,766** / max **58,159**；sum **242,555**；分位 Q1≈3,742、Q3≈18,681（data: manifest）。
- **expected_unique=13**：重复组 ①`title-menu_f00180_boot-black`↔`l4_stg02-start`（`788d6984…`）②四联 4 张（`62a8ef06…`）③`l4_ending-4/5`（`6ff345dc…`）；
- **actual_unique=14**：重复组 ①四联 4 张=`406f3962…`（真实同画面，Mesen 源截图同屏 `62a8ef06…`）②ending-4/5=`5aa8a4f8…`；
  **`l4_stg07-combat`/`l4_stg07-start` 在新 GUI trace 下已不同图**（`65de647d…`/`c9070704…`）⇒ **旧 headless actual 6-group（OBS-1 语境）不再成立/作废为 superseded**，
  详见 `p0-0-same-blob-adjudication.md` §9 dated 注；四联同 blob 的「真实同画面」结论仍成立（源截图证据不变）。

## 4. 与旧 headless 重裁（task-130）的差异

| 项 | 旧 headless 版（task-130） | GUI 版（本报告） |
|---|---|---|
| 可渲/blocked | 16 可渲 / 6 blocked（组 D 全 blocked） | **18 可渲 / 4 blocked**（ending-4/5 有源） |
| diff（18 对可比部分） | gameplay 10033/7219/6080；clear 58364；combat-variety 28563；late 5527；shop 18241；stg02 4476；stg03 2918；四联 18887×4；stg07 12208/13501；title 0 | gameplay **3742/3609/4382**；clear **58159**；combat-variety **28260**；late **6118**；shop **19040**；stg02 **2495**；stg03 **2261**；四联 **18681×4**；stg07 **11105/11806**；title **0**；ending-4/5 **8427×2** |
| actual 同 blob | 6-group（含 stg07 两帧） | **4-group（四联）+ ending-4/5**；stg07 不再同图 |
| 组 D | 6 blocked:no-source | **2 sourced（assisted）+4 blocked**（dense 负证据） |
| 结论 | blocked:no-source；不得全绿 | 同左（4 blocked）+ ending-4/5 可渲；**L4 未达成** |

> 变化解读：①新 trace 输入生效后多数对 diff 下降（gameplay/stg02/stg03/stg07 明显；四联 18887→18681）；
> ②少数变差（late-combat 5527→6118、shop-window 18241→19040）——属 P0-1 渲染差异，不改变口径；
> ③title 保持像素级 0 差异（sha 不同仅 PNG 编码）；④旧 headless 6-group 作废为 superseded（见 §3）。

## 5. 手册更新落点（dated，历史保留）

| 节 | 行号 | 内容 |
|---|---|---|
| §7.3 组 D ending | L1383 | task-143 最新：2 sourced（ending-4/5 assisted）+4 blocked；4 blocked 不渲；旧 headless 语境 superseded |
| §7.4 fade-clear | L1420 | #11 维持 blocked:no-source（dense 133 张零命中） |
| §7.5 去重 | L1481 | expected_unique=13 / actual_unique=14；旧 6-group 作废；registry 按组对账 |
| §8.2 验收 | L1734 | 18 可渲/4 blocked；diff min0/median9766/max58159/sum242555；0/18 actual==expected；P0-0 不得全绿 |

## 6. P0-0-4 输入与后续

1. **L4 0 容差未达成**：0/18 actual==expected sha（18 对 diff>0，median 9766）⇒ P0-1 差异收敛是下一阶段；**不得宣称 L4 通过 / P0-0 全绿**；
2. **组 D**：2 sourced（ending-4/5，assisted；expected 旧件保留，mapping 标 `trigger=assisted`）+ 4 blocked（fade-clear/ending-1/2/3；dense 负证据）；
3. **expected 待重采**：#14 stg02-start（boot-black 复制品）+ 四联 expected（62a8ef06 旧件）按 §7.2/§7.7；ending-4/5 登记 assisted 来源；
4. **去重登记**：actual 四联/ending-4/5 组按 documented_duplicates；expected 三组登记；`registry_ok` 按组对账（旧 6-group 口径作废）；
5. **P0-0-4 冻结判断**：在 4 blocked + expected 重采完成前，仅能以「显式不全绿」记录；本报告提供 18 对 diff 基线供 P0-1 验收。

## 7. 复算命令（只读）

```powershell
# 逐对 diff（0 容差真彩域；与 tools/diff_visual.compare 同口径）
python - <<'PY'
import json, pathlib, sys; sys.path.insert(0,'tools'); from diff_visual import compare
M=json.loads(pathlib.Path('target/p0-0/render/g-manifest-v2.json').read_text(encoding='utf-8'))
E=pathlib.Path('oracle/screens/expected'); G=pathlib.Path('target/p0-0/render/g_run1')
for p in M['pairs']:
    dp,err=compare(E/p['expected_file'], G/(p['name']+'.png'))
    print(p['name'], len(dp) if dp is not None else err)
PY
```

## 8. 验收自查

- [x] 22 对逐对（18 可渲 + 4 blocked）给出 name/frame/scene/组/source/status/expected sha/actual sha/diff/ratio/bbox/分类/处置（§2）；
- [x] diff 口径 = diff_visual.compare（256×240 真彩域 0 容差），未放宽；未跑 gates visual（§1）；
- [x] 组 D=2 sourced（assisted）+4 blocked；**5 对 replaced-invalid-asset（#14+四联）**；actual 两跑 36/36 相同（§0/§3）；
- [x] **明确 L4 0 容差未达成 / 不得宣称 L4 通过 / P0-0 不得全绿**（§0/§4/§6）；
- [x] 手册 §7.3/§7.4/§7.5/§8.2 已 dated 更新（§5 行号）；same-blob 文档 §9 dated 注（GUI 版 6-group 作废）；
- [ ] docs-links PASS（提交前复核）；独立复核 pass（task-142 之后派）。

## 9. 安装后 dated 更新（2026-10-20 · task-165 Option B 执行）

> 本 section 覆盖下文行级历史值（§2 row9 等保留原样）：以本节为**安装后现状**。

- **#14 `l4_stg02-start`**：定性由「invalid/invalid-asset」更正为 **frame-mismatch（错帧覆盖，非伪造）**——f3404 在原作/重采均为过场黑屏 `788d6984…`；Lead 裁定 **Option B**：pair 重指向 **`gs5-anchor-render:3450`**（overlay commit `86e18d4`），expected 替换为 **`44e7800d…`（1411B 真「地区-2」屏）**，actual 用 overlay trace **重渲两跑 `27fa3737…`/`436120b6…`（确定性 PASS）**；**diff=2352 px（ratio 0.038281）**；旧 2495 为旧 expected/旧 actual 历史值，Option A 的 3943 已否决。
- **四联 #16/#19/#20/#21 + #17/#18**：expected **bytes 不变**，仅登记来源：`expected_source=stg08-timeline:<22000/16600/18000/28000/14600/16000>:<tag>`；status：主帧 `l4_stg06-late`=ok、其余 3=documented-duplicate（duplicate_of=l4_stg06-late）、#17/#18=ok；**trigger natural→injected**（stg08-timeline 含 `poke 2400 $26/$0400`，原 natural 为错标，已更正）。
- **ending-4/5**：bytes 不变；expected_source tag 更正为 `tas-stg08b2-direct:34500/36000:probe`；trigger=assisted、documented-duplicate。
- **数量（安装后）**：expected full 22 **unique 18**（原 17；stg02↔title 组拆开）；rendered expected_unique **14**（原 13）、actual_unique **14**；`replaced-invalid-asset` **5→0**；`blocked:no-source` **4 不变**（组 D 终态）；registry `expected 14=18−4 / actual 14=18−4`、`registry_ok=true`；manifest `--validate` **PASS**、two-run **18/18 PASS**；`expected_matches` 仍 0/18（L4 0 容差未达成）。
- **旧 expected 归档**：`oracle/screens/expected/.archive/20261020-000358/l4_stg02-start.png`（`788d6984…`/258B，历史保留）。
- **未做**：strict/visual（写 tracked；由 §8.1 冻结卡统一）；4 blocked 不安装；diff>0 依旧（P0-1 渲染缺陷）。
