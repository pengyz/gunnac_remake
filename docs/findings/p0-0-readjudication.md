# P0-0-3：22 对组 A–E 重裁（actual vs 旧 expected；不放宽阈值）（task-130）

> 【2026-10-18 更正·superseded】本文件基于 headless(testRunner) 采集，其 emu.setInput 不生效 ⇒ 输入未注入、语义无效；重采见 task-133（GUI 模式）；原数字保留作历史。

> 日期：2026-10-18｜角色：l4-analyst（执行/记录）｜状态：**22 对当前状态定稿：16 可渲（1 像素级 0 差异 + 15 差异>0）/ 6 组 D `blocked:no-source`；P0-0 不得宣称全绿**
> 任务：task-130（P0-0 手册 §6/§7.3/§7.4/§7.5/§8.2）。**说明（F3）**：任务卡写「组 A–F」，手册实际分组为
> **A–E**（title-menu=E，无 F 组）；本报告与手册统一按 A–E。输入 = P0-0-2 两跑 actual（`target/p0-0/render/run1|run2`，
> 两跑 png/idx 全同）+ 旧 expected（`oracle/screens/expected/`）；差异口径 = `tools/diff_visual.py` 的
> `compare()`（**256×240 原生逐像素、真彩域、0 容差**；未跑 gates visual，避免写 tracked `oracle/screens/`）。
> 纪律：不改 expected/threshold/代码；`analysis/`、`shots/`、`target/` gitignored 不入库。

## 0. 结论摘要（先读）

| 项 | 值 |
|---|---|
| 22 对分组（按手册） | A 3 / B 6 / C 6 / **D 6（blocked）** / E 1 |
| 可渲（actual 有图） | **16/22**（A3+B6+C6+E1） |
| **像素级差异 = 0（0 容差通过）** | **1/16**：`title-menu_f00180_boot-black`（E；diff=0，注意 sha 不同=PNG 编码差） |
| **像素级差异 > 0** | **15/16**（A3+B6+C6）：min `2918`（`l4_stg03-start`，4.75%）、median `12,854.5`（**16 可渲含 1 个 0 差异的口径；按 15 个正值算 = `13,501`**）、max `58,364`（`l4_clear-inject`，94.99%） |
| `blocked:no-source`（不渲染） | **6/22**：组 D #6-#11（ending-1..5/fade-clear，task-106/127 定谳） |
| `replaced-invalid-asset`（expected 待 §7.2 重采） | **5/22**：#14 `l4_stg02-start` + 四联 #16/#19/#20/#21（actual 已重渲，diff 见 §2） |
| 去重（manifest 22 名过滤） | expected_unique **17**（3 组）/ actual_unique **11**（actual 6-group，真实 OBS-1 冻结屏，task-126/128） |
| **P0-0 可宣称全绿？** | **否**：组 D 无源 + expected 5 对待重采 + 15 对 P0-1 差异未收敛（0 容差基线预期 FAIL by design） |

## 1. 方法与命令

- 差异像素数：`from tools.diff_visual import compare; compare(expected_path, actual_path)` —— TRUE_COLOR=True、
  256×240、逐像素 RGB 不等计数、0 容差（与 visual 门默认口径一致；**不启用白名单**）。
- 每对 expected 文件名：group A 三对 expected 带 tag 后缀（`gameplay-stage1_f01260_stage-start.png` 等），
  title 为 `title-menu_f00180_boot-black.png`，其余 `<name>.png`（映射见 manifest `expected_file`）。
- 两跑确定性：actual run1/run2 png+idx sha 全同（P0-0-2 manifest `deterministic=true`，本重裁复核 16/16）。
- 机器可读逐对数据：`analysis/repro/p0-0-readjudication/readjudication.json`（gitignored；含 diff_pixels/
  diff_ratio/diff_bbox/expected_file/sha/run2_same）。
- **未跑 gates check/status/visual**（visual 会写 tracked `oracle/screens/{actual,diff,visual-report.md}`）。

## 2. 逐对重裁表（22 行；diff = 像素差异数 / 61440；bbox=[x0,y0,x1,y1]）

| # | name | frame | scene | 组 | render_status | expected sha（前 12） | actual sha（前 12） | diff 像素 | 差异率 | bbox | 分类 | 处置 |
|---:|---|---:|---|---|---|---|---|---:|---:|---|---|---|
| 1 | gameplay-stage1_f01260 | 1260 | gameplay-stage1 | A | ok | `38365db2cb4f` | `62417db716bc` | 10033 | 16.33% | 0,14,255,239 | diff>0 | P0-1 差异；基线记录 |
| 2 | gameplay-stage1_f01500 | 1500 | gameplay-stage1 | A | ok | `7f4ac8a23511` | `05de7bb46cdf` | 7219 | 11.75% | 0,7,255,239 | diff>0 | 同上 |
| 3 | gameplay-stage1_f02349 | 2349 | gameplay-stage1 | A | ok | `4274a33f45f5` | `28f094fd515d` | 6080 | 9.90% | 0,14,255,239 | diff>0 | 同上 |
| 4 | l4_clear-inject | 4600 | stg03-timeline | B | ok | `5cacbe294d53` | `a666b16136f5` | 58364 | 94.99% | 0,0,255,239 | diff>0 | P0-1 差异（整帧级） |
| 5 | l4_combat-variety | 6500 | stg03-timeline | B | ok | `6f7fe4992e59` | `50b9d9ae983b` | 28563 | 46.49% | 0,0,255,239 | diff>0 | 同上 |
| 6 | l4_ending-1 | 30000 | — | D | **blocked:no-source** | `19b88b1ffce9` | — | — | — | — | blocked | 不渲染/不 staging（task-106/127） |
| 7 | l4_ending-2 | 31500 | — | D | **blocked:no-source** | `0f8e11694d53` | — | — | — | — | blocked | 同上 |
| 8 | l4_ending-3 | 33000 | — | D | **blocked:no-source** | `f319f1fe5256` | — | — | — | — | blocked | 同上 |
| 9 | l4_ending-4 | 34500 | — | D | **blocked:no-source** | `6ff345dc5c3f` | — | — | — | — | blocked | 同上（expected-4/5 同 blob） |
| 10 | l4_ending-5 | 36000 | — | D | **blocked:no-source** | `6ff345dc5c3f` | — | — | — | — | blocked | 同上 |
| 11 | l4_fade-clear | 28500 | — | D | **blocked:no-source** | `14802bcf460a` | — | — | — | — | blocked | 同上 |
| 12 | l4_late-combat | 12000 | stg03-timeline | B | ok | `c0baa0f8d21b` | `801e99d5143b` | 5527 | 9.00% | 0,0,255,239 | diff>0 | P0-1 差异 |
| 13 | l4_shop-window | 5400 | stg03-timeline | B | ok | `988809e0b414` | `b560e275d5e6` | 18241 | 29.69% | 0,0,255,234 | diff>0 | P0-1 差异 |
| 14 | l4_stg02-start | 3404 | stg03-timeline | B | **replaced-invalid-asset** | `788d6984576b` | `af6cedfbef08` | 4476 | 7.29% | 0,41,255,239 | expected-invalid | expected 待 §7.2 重采（=boot-black 复制品）；actual 已重渲 |
| 15 | l4_stg03-start | 5700 | stg03-timeline | B | ok | `24a4cb45c0d8` | `d808b2349802` | 2918 | 4.75% | 0,0,255,234 | diff>0 | P0-1 差异（最小） |
| 16 | l4_stg06-late | 22000 | stg08-timeline | C | **replaced-invalid-asset** | `62a8ef06c4b9` | `5f93bdbb2c84` | 18887 | 30.74% | 0,0,255,239 | expected-invalid | 四联 expected 待 §7.2 重采；actual=OBS-1 冻结屏 |
| 17 | l4_stg07-combat | 16000 | stg08-timeline | C | ok | `2824cb134987` | `5f93bdbb2c84` | 12208 | 19.87% | 0,0,255,239 | diff>0 | P0-1 差异；actual 6-group 成员（documented duplicate） |
| 18 | l4_stg07-start | 14600 | stg08-timeline | C | ok | `ddb322ac5702` | `5f93bdbb2c84` | 13501 | 21.97% | 0,0,255,239 | diff>0 | 同上 |
| 19 | l4_stg08-combat | 18000 | stg08-timeline | C | **replaced-invalid-asset** | `62a8ef06c4b9` | `5f93bdbb2c84` | 18887 | 30.74% | 0,0,255,239 | expected-invalid | 四联待重采；actual 冻结屏 |
| 20 | l4_stg08-late | 28000 | stg08-timeline | C | **replaced-invalid-asset** | `62a8ef06c4b9` | `5f93bdbb2c84` | 18887 | 30.74% | 0,0,255,239 | expected-invalid | 同上 |
| 21 | l4_stg08-start | 16600 | stg08-timeline | C | **replaced-invalid-asset** | `62a8ef06c4b9` | `5f93bdbb2c84` | 18887 | 30.74% | 0,0,255,239 | expected-invalid | 同上 |
| 22 | title-menu_f00180_boot-black | 180 | title-menu | E | ok | `788d6984576b` | `1b1940f30bf8` | **0** | **0.00%** | — | **pixel-equal** | **0 容差通过**（sha 不同=PNG 编码差；严格模式该对 PASS） |

> 说明：`render_status` 沿用 P0-0-2 manifest 的 **provisional** 语义（ok=渲染成功+两跑一致，不代表 expected 已确认；
> F125-1）；本表的"分类"是 task-130 重裁字段。所有 rendered 对 run1/run2 png/idx 全同（deterministic）。

## 3. 统计（按组与差异分布）

| 组 | 对数 | 可渲 | 0 差异 | diff>0 | blocked | replaced-invalid | 备注 |
|---|---:|---:|---:|---:|---:|---:|---|
| A | 3 | 3 | 0 | 3 | 0 | 0 | diff 6080–10033 |
| B | 6 | 6 | 0 | 5 | 0 | 1 | #14 expected 无效；diff 2918–58364 |
| C | 6 | 6 | 0 | 2 | 0 | 4 | 四联 expected 待重采；actual 6-group（OBS-1） |
| **D** | 6 | 0 | 0 | 0 | **6** | 0 | blocked:no-source，不 staging |
| E | 1 | 1 | **1** | 0 | 0 | 0 | title pixel-equal（0 容差通过） |
| 合计 | 22 | 16 | **1** | **15** | **6** | **5** | — |

- diff 分布（15 个 diff>0）：min **2918** / **Q1 6080** / median **12,854.5**（16 可渲含 1 个 0 差异口径；**15 正值口径 13,501**；mean 16,178.5）/ **Q3 18,887** / max **58,364**（全 61440 px 的 4.75%–94.99%）；
- **去重**：expected_unique=**17**（3 组：ending-4/5 ×2；stg02-start↔title ×2；四联 ×4）；
  actual_unique=**11**（1 组 ×6：`l4_stg06-late, l4_stg07-combat, l4_stg07-start, l4_stg08-combat, l4_stg08-late, l4_stg08-start`）；
  actual 6-group = 真实 OBS-1 冻结屏（task-126/128 pass），**登记 documented_duplicates、不剔除**；
- P0-0 前旧账面 expected unique 17/22 与本次一致（17），actual 侧旧 127 unique 是**未过滤目录**产物（F2），不得混用。

## 4. 与旧 §7.3/§7.4 的差异（dated 更正要点）

| 旧口径（沈） | task-130 重裁后现状 | 影响 |
|---|---|---|
| §7.3 "ending 五帧 actual 同 blob（TBLDRV 诊断态遗留）→ 逐帧独立重渲" | 组 D 已定谳 **blocked:no-source**：5 对（#6-#10）**不重渲/不 staging**，旧 expected 保留；§7b 的"actual 两两不同"核对**当前无 actual**（不适用） | P0-0 不得全绿；恢复条件=§3.6 更正块第 4 条 |
| §7.4 fade-clear 来源待证 | 同组 D：**#11 blocked:no-source**，不 staging | 同上 |
| §7.1 stg02-start expected 无效 → 重采 | 现状 **replaced-invalid-asset**：expected 仍无效（=boot-black 复制品，与 title expected 同 blob），**重采未执行**（待 §7.2 流程/§7.7 staging）；actual 已重渲（diff 4476） | expected 待换，P0-0 不得全绿 |
| §7.2 四联 expected 同 blob（OBS-1 B 级解释）→ 重采 | 现状 4 对 **replaced-invalid-asset**（expected 待 §3.7 选帧来源重评/重采）；actual 4 对=同一冻结屏（diff 18887 对旧 expected） | expected 待换 |
| （新）actual 同 blob 是否缺陷 | **真实 OBS-1 冻结屏**（task-126/128）：6 对同 blob `5f93bdbb…`=真实同画面；登记 documented_duplicates，不剔除、不当 0 容差通过 | §7.5/§8.2 需按 actual-only 组登记 |
| （新）title pair | **pixel-equal（diff=0）**：actual 与 expected sha 不同但像素全等（PNG 编码差）⇒ 0 容差下该对通过 | 严格模式 1/16 对已 PASS |

## 5. 后续 P0-0-4 输入（冻结前必须处理）

1. **组 D 6 对**：`blocked:no-source`（不 staging）；P0-0 冻结时 manifest 对应 6 行 blocked，不得宣称全绿；
2. **expected 待重采 5 对**（#14 + 四联 #16/#19/#20/#21）：走 §7.2/§3.7 选帧来源重评 → §7.7 安装 → manifest
   `expected_sha256/status` 回填；**在 §5 wrapper 之前完成**（B1）；
3. **actual 侧**：16 对 deterministic 全同；actual 6-group 按 documented_duplicates 登记（§7.5 registry_ok 扩展，
   actual-only 组）；title pair 像素 0 差异可直接进严格基线；
4. **P0-1 差异**：15 对 diff>0（median 12,854.5，最大 58,364）是后续 0 容差修复目标，**不是 P0-0 验收失败**；
5. **统计口径**：以 `analysis/repro/p0-0-readjudication/readjudication.json` + `target/p0-0/render/manifest.json`
   为准；任何环境/目录口径变化须 dated 更正。

## 6. 复算命令（只读）

```powershell
# 逐对 diff（0 容差真彩域，与 tools/diff_visual.compare 同口径）
python - <<'PY'
import json, pathlib, sys
sys.path.insert(0, "tools")
from diff_visual import compare
REPO=pathlib.Path("."); M=json.loads((REPO/"target/p0-0/render/manifest.json").read_text(encoding="utf-8"))
for p in M["pairs"]:
    if not p["render"]: continue
    e=REPO/p["expected_file"]; a=REPO/"target/p0-0/render/run1"/f'{p["name"]}.png'
    dp,err=compare(e,a)
    print(f'{p["name"]:30} diff={len(dp) if dp is not None else err} ratio={(len(dp)/61440 if dp is not None else -1):.4f}')
PY
```

## 7. 验收自查

- [x] 22 对逐对给出 name/frame/scene/组/expected sha/actual sha/diff 像素/差异率/bbox/分类/处置（§2）；
- [x] 差异口径 = diff_visual compare（256×240、真彩域、0 容差），未放宽阈值、未跑 gates visual（§1）；
- [x] 组 D 6 对 blocked:no-source 不 staging；5 对 replaced-invalid-asset 标 expected 待 §7.2 重采；
      actual 6-group 按 documented_duplicates 登记（§0/§3/§4）；
- [x] 更新手册 §7.3/§7.4/§7.5/§8.2 dated 更正（保留历史；P0-0 不得全绿）；
- [x] 顺手 F128-1/F128-2：`p0-0-same-blob-adjudication.md` 补 `compare.json` 字段/命令 + 51/45 静态组
      shots glob 名单/命令；
- [x] docs-links PASS（374 md/459 链接，2026-10-18 复核）；独立复核 pass（task-132 已 pass）。
