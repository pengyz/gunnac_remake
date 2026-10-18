# 对抗复核：22 对重裁 GUI 版（task-143；含 task-146 口径修后版本）

> 日期：2026-10-18｜复核方：decomp-verifier（verifier 角色，证伪立场）｜被审：l4-analyst 的 task-143
> （`docs/findings/p0-0-readjudication-gui.md` 125 行 untracked；手册 §7.3/§7.4/§7.5/§8.2 dated；same-blob §9 dated 注；task-146 已把 replaced 口径修成 5 对）。
> 复核方式：**只读 + `tools.diff_visual.compare` 实跑**；未跑 gates check/status/visual（遵守任务卡）。actual=`target/p0-0/render/g_run1`（g_run2 对照）；expected=`oracle/screens/expected`（未改）。
> 口径：按 Lead **2026-10-18 更正** —— `replaced-invalid-asset` 实为 **5 对**（#14 stg02-start + 四联 #16/#19/#20/#21），"16 canonical 中 5 对 expected 待重采"（任务卡原文"#14 replaced"为旧口径，已由 Lead 更正；本复核按 5 对核）。
> 本文件为唯一写入，未 git add/commit；未改 expected/threshold/代码。

---

## 0. VERDICT

**needs_revision（Low-only：1 必改 + 3 Info）**——重裁数据面（18 对 diff/统计/去重/两跑确定性/22 对口径）、`p0-0-readjudication-gui.md`、`p0-0-render-gui-run.md`、手册 §7.3/§7.4/§7.5 与 same-blob §9 **全部独立复现通过**；
唯一必改 = **AR144-1**：手册 **§8.2 task-143 dated 注（L1735/L1738）仍按 1 对口径**写 `#14 仍 replaced-invalid-asset` / `#14 expected 待重采`，与 manifest/readjudication 的 **5 对**不一致（task-146 只修了两个 findings 文档，未覆盖手册）。

| ID | 分类 | 级 | 问题 | 处理 |
|---|---|---|---|---|
| **AR144-1** | **规格缺口（口径残留）** | **低（必改）** | 手册 §8.2 **L1735** `#14 仍 replaced-invalid-asset`、**L1738** `P0-0 全绿判定=否（4 blocked + #14 expected 待重采 + 18 对 diff>0）` —— 实为 **5 对 replaced**（#14 + 四联 #16/#19/#20/#21），16 canonical 中 **5 对 expected 待重采**；与 readjudication §0 L11/L17、render-gui-run L13/L64/L102/L105、manifest `replaced_invalid_asset:5` 不一致 | 改 L1735/L1738 为"**5 对 replaced（#14+四联）**；4 blocked + **5 对 expected 待重采** + 18 对 diff>0" |
| AR144-2 | 规格缺口（编号口径） | Info | readjudication §2 表 `#` 列是**行序**（1..22 按组重排：row 9=stg02-start、row 14=stg08-combat…），而正文/手册引用 **canonical 22 对编号**（#14 stg02-start、#16/#19/#20/#21 四联）⇒ 同表"#14"行实为 stg08-combat，易误读（render-gui-run §2 表同构） | 表头改"序"或加 canonical `pair#` 列（task-142/146 范围，可顺手） |
| AR144-3 | 报告/证据（信息） | Info | 手册 §8.2 **task-130 旧注 L1729 已写"5 对 replaced-invalid-asset"（正确）**，task-143 新注反而回到 1 对 ⇒ 新旧注自相矛盾（由 AR144-1 统一即可；旧注无需改） | 同 AR144-1 |
| AR144-4 | 纪律/口径（信息） | Info | 任务卡原文"#14 replaced"是旧口径；Lead 已更正 5 对。`p0-0-render-gui-run.md` 属 task-146 A 案（非本卡 deliverable），本复核**仅交叉核对**：其 L13/L64/L102/L105 已是 5 对、L88-89 已是 161–165s 区间 ✓ | 提交时按 Lead 口径说明 |

---

## 1. diff 抽样复算（`tools.diff_visual.compare`，256×240 真彩域 0 容差）

| 对 | expected_file | expected sha(12) | actual sha(12) | 复核 diff | ratio | bbox | 报告值 | 判定 |
|---|---|---|---:|---:|---|---|---|---|
| `title-menu_f00180_boot-black` | `title-menu_f00180_boot-black.png` | `788d6984576b` | `1b1940f30bf8` | **0** | 0.0000 | — | 0 / 0.0000 / — | ✓（sha 不同=PNG 编码，像素级 0） |
| `gameplay-stage1_f01260` | `gameplay-stage1_f01260_stage-start.png` | `38365db2cb4f` | `c39bbeffe098` | **3742** | 0.0609 | (0,8,255,239) | 3742 / 0.0609 / 0,8,255,239 | ✓ |
| `l4_clear-inject` | `l4_clear-inject.png` | `5cacbe294d53` | `246dd5712608` | **58159** | 0.9466 | (0,0,255,239) | 58159 / 0.9466 / 0,0,255,239 | ✓ |

- expected_file 映射取自 `g-manifest-v2.json` 的 `pairs[].expected_file`（group A 三对/title 用带 tag 文件名——报告 §1 方法描述正确）。

## 2. 全量 18 对统计/去重复算（A）

- **18 对 diff 逐对复算与报告 §2 表完全一致**：`3742/3609/4382/0/58159/28260/6118/19040/2495/2261/18681×4/11105/11806/8427×2`。
- **统计**：min **0** / median **9766.0** / max **58159** / sum **242,555**（与 §0/§3、手册 §8.2 一致）✓
- **expected_unique=13**（18 − 5 重复）：`title-menu↔l4_stg02-start`=`788d6984…`（1 组）、四联 4 张=`62a8ef06…`、`ending-4/5`=`6ff345dc…` ✓
- **actual_unique=14**（18 − 4）：四联 4 张=`406f3962…`、`ending-4/5`=`5aa8a4f8…`；`l4_stg07-combat`=`65de647d…`、`l4_stg07-start`=`c9070704…` **已不同图** ✓
- **actual==expected sha：0/18** ✓（`L4 0 容差未达成`表述如实）
- manifest summary 对照：`rendered=18 / blocked_no_source=4 / replaced_invalid_asset=5 / two_run_deterministic=18 / expected_matches=0` ✓

## 3. 22 对口径（按 Lead 5 对 replaced 更正）

| 项 | 复核值 | 依据 |
|---|---|---|
| 18 可渲 | **16 canonical + ending-4/5（2 assisted）** | manifest `rendered=18`、`rendered_breakdown={natural:10,injected:6,assisted_ending45:2}`；16=11 ok+5 replaced |
| 4 blocked | fade-clear/ending-1/2/3 | manifest `blocked_pairs` 逐名 ✓；dense 负证据：`analysis/traces/tas-stg08b2-dense.captures.jsonl` **133 条/133 张 shots**，逐张 sha 对 4 张 expected **全 NONE**（我独立扫描）✓ |
| **5 对 replaced** | #14 stg02-start + 四联 #16/#19/#20/#21 | manifest `replaced_invalid_asset=5`；readjudication §0 L11/L17 + §2 表 5 行 ✓；**16 canonical 中 5 对 expected 待重采** ✓ |
| 判定表述 | "0/18 actual==expected、L4 未达成、不得宣称 L4 通过/P0-0 全绿" | readjudication §0 L14/L17/§6、手册 §8.2 L1736-1738 ✓ 如实 |

- 与任务卡旧口径差异：任务卡正文"#14 replaced"应为 **5 对**（Lead 2026-10-18 更正）；readjudication/render-gui-run 已同步，**手册 §8.2 未同步（AR144-1）**。

## 4. 两跑确定性（A）

- `g_run1` vs `g_run2`：**18/18 对 png 相同 + 18/18 idx 相同 = 36/36**（独立 sha 比对，无例外）✓（报告 §0"36/36 渲染"准确）。

## 5. 手册落点与 same-blob §9

| 落点 | 复核 | 判定 |
|---|---|---|
| §7.3 **L1383-1388**（task-143 GUI） | 组 D=2 sourced（#9/#10 ending-4/5，assisted，diff 8427/8427）+4 blocked（dense 133 张零命中）；旧 headless 语境 superseded；证据指针到 readjudication/g-manifest ✓ | ✓ |
| §7.4 **L1420-1421** | #11 fade-clear 维持 blocked:no-source（dense 133 张零命中 `14802bcf…`）✓ | ✓ |
| §7.5 **L1481-1487** | expected_unique=13 / actual_unique=14；旧 headless actual 6-group 不再成立（stg07 两帧 `65de647d…`/`c9070704…`）；四联同 blob 与 Meso 源截图 `62a8ef06…` 同屏、登记 documented_duplicates；公式 18=14+4、13=18−5 ✓ | ✓ |
| §8.2 **L1734-1739** | 18 可渲/4 blocked、diff min0/median9766/max58159/sum242555、0/18、L4 未达成 ✓；**但 replaced 口径仍 "#14"（1 对）→ AR144-1** | **✗（仅口径）** |
| same-blob **§9 L168-176** | GUI 版：actual_unique=14、actual 同 blob 只剩 2 组（四联 406f3962… 真实同画面 + ending 5aa8a4f8…）；旧 headless actual 6-group 含 stg07 不再成立、作历史保留不删、不得再用于去重/基线；四联同 blob 结论仍成立 ✓ | ✓ 语义清楚 |

- 手册行号锚（readjudication §5 引 L1383/L1420/L1481/L1734）与当前手册**逐一对应** ✓。

## 6. 与旧 headless 重裁（task-130）差异对照（无夸大）

- 旧值（`p0-0-readjudication.md` §2 实读）：gameplay 10033/7219/6080、clear 58364、combat-variety 28563、late 5527、shop 18241、stg02 4476、stg03 2918、四联 18887×4、stg07 12208/13501、title 0 ⇒ 与 GUI 报告 §4 表**完全一致** ✓
- 新增值复算（§1/§2）后逐对比较：**改善 13 对**（gameplay 3、clear、combat-variety、stg02、stg03、四联 4、stg07 2，均下降）、**变差 2 对**（`l4_late-combat` 5527→**6118**、`l4_shop-window` 18241→**19040**）、title 0→0、ending-4/5 为新来源（8427，旧 blocked 不可比）。
- 报告 §4"多数改善 / 少数变差（late、shop）"**准确、无夸大**；`actual 同 blob 6-group→4-group+ending`、组 D 6 blocked→2 sourced+4 blocked 均与实测一致 ✓

## 7. 范围 / 纪律 / 门禁

- **未改 expected/threshold/代码**：`git status --porcelain oracle/screens tools/diff_visual.py` **空** ✓；expected 仍为旧件（§0 声明一致）。
- **未跑** gates check/status/visual（任务卡纪律）；仅跑 `tools.diff_visual.compare`（允许）+ 单门 docs-links。
- **docs-links PASS**（复核前 386 md/466 links；本复核文件另加 1 md）。
- 工作区并行状态：`p0-0-readjudication-gui.md`/`p0-0-render-gui-run.md` untracked（待入库）；`target/`/`analysis/` gitignored（manifest 指针已注明）；手册/ same-blob 为 M（task-143 dated 注）。
- 未改判定/阈值；未把 headless 旧结论当 GUI 值（§6 对照已分开）。

## 8. 复现命令（只读；PowerShell 可执行）

```powershell
# 1) 抽样 3 对 diff（与报告同口径）
@'
import sys,pathlib; sys.path.insert(0,'tools'); from diff_visual import compare
E=pathlib.Path('oracle/screens/expected'); G=pathlib.Path('target/p0-0/render/g_run1')
for name,exp in [('gameplay-stage1_f01260','gameplay-stage1_f01260_stage-start.png'),
                 ('l4_clear-inject','l4_clear-inject.png'),
                 ('title-menu_f00180_boot-black','title-menu_f00180_boot-black.png')]:
    dp,err=compare(E/exp,G/(name+'.png'))
    print(name,len(dp) if dp is not None else err)
'@ | python -
# 2) 全量 18 对 + 统计 + 去重（expected_file 取 manifest）
@'
import json,pathlib,sys,statistics,collections,hashlib
sys.path.insert(0,'tools'); from diff_visual import compare
M=json.loads(pathlib.Path('target/p0-0/render/g-manifest-v2.json').read_text(encoding='utf-8'))
E=pathlib.Path('oracle/screens/expected'); G=pathlib.Path('target/p0-0/render/g_run1')
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
rows=[]
for p in M['pairs']:
    dp,err=compare(E/p['expected_file'],G/(p['name']+'.png'))
    rows.append((p['name'],len(dp),sha(E/p['expected_file']),sha(G/(p['name']+'.png'))))
ds=[r[1] for r in rows]
print('min/median/max/sum',min(ds),statistics.median(ds),max(ds),sum(ds))
print('exp_unique',len({r[2] for r in rows}),'act_unique',len({r[3] for r in rows}),
      'eq',sum(1 for r in rows if r[2]==r[3]))
print('manifest',json.dumps(M['summary'],ensure_ascii=False)[:300])
'@ | python -
# 3) 两跑确定性 36/36
@'
import json,pathlib,hashlib
M=json.loads(pathlib.Path('target/p0-0/render/g-manifest-v2.json').read_text(encoding='utf-8'))
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
G1=pathlib.Path('target/p0-0/render/g_run1'); G2=pathlib.Path('target/p0-0/render/g_run2')
okp=oki=0
for p in M['pairs']:
    n=p['name']
    okp+=sha(G1/(n+'.png'))==sha(G2/(n+'.png'))
    oki+=sha(G1/(n+'.idx.json'))==sha(G2/(n+'.idx.json'))
print('png',okp,'idx',oki,'of',len(M['pairs']))
'@ | python -
# 4) dense 4-blocked 负证据（133 张零命中）
@'
import json,pathlib,hashlib
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
E=pathlib.Path('oracle/screens/expected')
exps={n:sha(E/(n+'.png')) for n in ['l4_fade-clear','l4_ending-1','l4_ending-2','l4_ending-3']}
cap=[json.loads(l) for l in open('analysis/traces/tas-stg08b2-dense.captures.jsonl',encoding='utf-8') if l.strip()]
hits=[r['shot'] for r in cap if sha(pathlib.Path(r['shot'])) in exps.values()]
print('dense shots',len(cap),'hits',hits or 'NONE')
'@ | python -
# 5) 单门
python -c "import sys;sys.path.insert(0,'gates');import cli;print(cli.gate_docs_links())"
```

## 9. 自查

- 只写本文件；未 git add/commit；未改 expected/threshold/代码/门禁；未跑 gates check/status/visual。
- 独立重导：3 对抽样 diff、18 对全量 diff/统计、expected/actual sha 去重、两跑确定性、dense 133 张 blob 扫描、manifest summary、手册 §7.3/§7.4/§7.5/§8.2 与 same-blob §9、旧 readjudication 数字对照——均实读/实跑，未采信被审自述。
- 未证不发明：task 卡旧口径（#14）与 Lead 更正（5 对）差异已在 §0 声明并按 5 对核；`p0-0-render-gui-run.md` 仅交叉核对（task-146 A 案），不作本卡 deliverable 判定依据。
- 建议修复顺序：AR144-1（手册 §8.2 两行口径，必改）→ AR144-2（表头编号，Info，可顺手）→ AR144-3/4 无需动作。

【2026-10-18】decomp-verifier，task-144。
