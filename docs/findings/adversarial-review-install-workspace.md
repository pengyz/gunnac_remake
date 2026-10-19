# 对抗复核：expected #14 安装 + 渲染输入工作区重建（task-171 终验）

> 日期：2026-10-20｜复核方：mimo（verifier 角色，证伪立场）｜实施方：退役 flash 成员（task-165/166/170 产物）+ platform-porter 补丁 `b100c31`
> 复核范围：①f3450 两跑确定性；②0 容差 diff 复算；③#14 安装核（expected/归档/pair/validate）；④task-170 工作区自证；⑤4 份 dated 文档一致性；⑥`oracle/screens` 变更面；⑦`b100c31` 的 `re_render` 例外审（纪律红线）；⑧00:08 重写来源披露。
> 方式：**独立重导**（重算 sha/diff/validate/测试，不采信被审自述）；**只读**，未跑 Mesen/gates check/status/visual；仅跑允许的 `python -m pytest` 10/10 与 `tools/l4_render_all.py --validate`（只读）。本文件为唯一写入；未 git add/commit。
> 终态以 Lead 收口后为准：`oracle/screens/actual`=新渲染 `27fa3737…`、expected=`44e7800d…`、diff/report 统一再生成=**2352 px**。

---

## 0. VERDICT

**needs_revision（Low-only：1 处 tracked 文档数字）**——**安装数据面全部独立复现通过**（f3450 确定性、2352 diff、expected/归档/pair/validate、工作区 17/17、oracle/screens 四件面、4 份 dated 文档），b100c31 例外为**窄、可追溯、有测试**的工程豁免，不构成"为通过校验放宽判定"红线。
唯一必须改：`docs/findings/team-ops-ledger.md` L68 仍写 `0 容差 diff=2076px`（HEAD 已含，clean）——实际为 **2352**（2076=旧 expected `788d` 的历史值）。改这 1 行即可 commit，无需重跑。

---

## 1. Findings 四分类

| ID | 分类 | 级 | 问题 | 处理 |
|---|---|---|---|---|
| **AR171-1** | 纪律/规格缺口（tracked 文档数字） | **低（必改）** | `docs/findings/team-ops-ledger.md` L68（HEAD 已提交、工作区 clean）写 **diff=2076px**；实测/install-plan/readjudication/s1-source/manual/manifest-full 均为 **2352**（2076=旧 expected `788d6984` vs actual 的历史值） | 改为 **2352px**（或 2352/0.038281）后随批/补一个 docs commit；安装数据不受影响 |
| AR171-2 | 报告/证据（验证路径口径） | Info | 卡面要求 `--validate target/p0-0/render/g-manifest-v2.json` → **FAIL**（`pairs 需 22 行，实际 18`）；正确 PASS 路径是 **`target/p0-0/render/g-manifest-full.json`**（install-plan §8 L113/L133 已用后者） | 复核/报告统一指向 `g-manifest-full.json`；`g-manifest-v2.json` 是旧 18 行 gitignored artifact（不得作为 22 行 validate 输入） |
| AR171-3 | 报告/证据（陈旧字段） | Info | `g-manifest-v2.json` #14 entry：`idx_sha=436120b680f7c3c2a4f8ee…` 尾段为伪造式错值（真实 `436120b680f7c3c23b6f3b…`；前 16 位相同故文档引用前缀无误）、`diff_pixels=2076` 未更新；install-plan L216/manual L1545 "v2 entry 已更新"仅部分成立 | target/ gitignored，不入 commit；建议重生成/清掉陈旧字段，避免再被引用 |
| AR171-4 | 纪律（`re_render` 例外审） | Info | `b100c31` 仅对 **#14** 跳过 `g-manifest-v2` 旧 run1 sha 交叉核对（`re_render=true`）；未改 gates/validate/determinism/expected 校验，有单测断言、commit 含 Evidence/Confidence | **不判红线**；建议收口后刷新 v2 基线或切换 `g-manifest-full` 作基线并移除例外；若保留，建议把布尔豁免收窄为"允许的历史 sha=`ebc6372f…`"白名单并保留其余 21 对严格核对 |
| AR171-5 | 报告/证据（披露） | Info | 00:08 `visual-report.md`/`diff.png` 重写**非本会话所为**（我只 import `compare/decode_png` 与 `--validate`，无 CLI 调用/无写）；我观察到中间态 2424=新 expected vs 旧 tracked actual `164b6e48`，收口后 actual=`27fa` → 2352（见 §9） | 如实披露；终态一致，无需回滚 |
| AR171-6 | 报告/证据（工作区报告措辞） | Info | `workspace-rebuild.md` §0/§7 写 "17 traces"，§2 表列 18 行（含 overlay `gs5-anchor-render` 行，text 标 "—"）；17 个重建对象与 JSON `traces` 一致，overlay 行是 t165 追加上下文 | 可括注"另 overlay 1 个（t165 生成）"；非阻塞 |

---

## 2. f3450 两跑确定性（独立重算，A）

| 项 | run1 | run2 | 判定 |
|---|---|---|---|
| png sha256 | `27fa3737335346322615d8cdc7ba21ff50d764b1a05f7928a09a51baf40d6281` | 同左 | ✓ 同 |
| idx sha256 | `436120b680f7c3c23b6f3bf60ada9b67ceab23a914b95a54fc41e55b778ccab2` | 同左 | ✓ 同 |
| 文件大小 | png 246,083B / idx 2,102B | 同 | ✓ |

- 同时独立重算 **18/18 对** run1 vs run2 的 png+idx：**18/18 全同，0 bad**（逐对循环重算，非采信 manifest）✓
- `t165/step0-result.json` 与磁盘一致（png/idx sha、wall 27.0/27.2s）；deterministic=true。

---

## 3. 0 容差 diff 复算（A）

`tools/diff_visual.compare(oracle/screens/expected/l4_stg02-start.png, oracle/screens/actual/l4_stg02-start.png)`：

| 项 | 独立复算 | 报告/实测 |
|---|---|---|
| expected sha/size | `44e7800dac894205b248ee5e0e3ebe436ec604c48235f203e2cdea1d8c8368f3` / **1,411B** | 同 ✓ |
| actual sha/size | `27fa373733534632…` / 246,083B | 同 ✓ |
| **diff 像素** | **2,352** | 2352 ✓ |
| ratio | **0.03828125**（2352/61440） | 0.038281 ✓ |
| bbox | **(4, 9, 249, 239)** | 同 ✓ |
| 分解 | 条带内(y<16)=**9** / 条带外=**2,343** | 2343+9 ✓ |
| diff png 红点计数 | **2,352**（逐像素解 PNG 统计） | 与 diff 一致 ✓ |
| visual-report | L23 `差异像素 2352`、L47 `2352/61440；未覆盖 2343 + W-C1 9`；22 结果 / 通过 1 / 分类 21 / **未分类 0** | 同 ✓ |

- **2076/2424 来源已解释**：2076 = 旧 expected `788d` vs actual（历史值）；2424 = 新 expected vs 旧 tracked actual `164b6e48`（收口前）；2352 = 新 expected vs 新 actual `27fa`（终态）——三者各自可复算，见 §9。

---

## 4. #14 安装核（A）

| 检查 | 结果 |
|---|---|
| 新 expected | `oracle/screens/expected/l4_stg02-start.png` = **1,411B `44e7800dac894205…`** ✓ |
| 旧件归档 | `.archive/20261020-000358/l4_stg02-start.png` = **258B `788d6984576bb13490a00df106ec8ec4a1c2f966e544debd99f7ffce8d65c029`** ✓；与 **git HEAD blob 258B 同 sha**（用 `git cat-file blob` 独立取字节核）✓；归档目录名与 docs 一致（**20261020-000358**，卡面 20260120 为笔误） |
| pair 映射 | manifest-full：`scene=gs5-anchor-render, frame=3450, expected_source=gs5-anchor-render:3450:stg02-start, trigger=injected, status=ok, reuse=independent` ✓；`analysis/repro/mapping.tsv` 同 ✓（22 行 ok13/dup5/blocked4） |
| `--validate`（**正确路径**） | `target/p0-0/render/g-manifest-full.json` → **MANIFEST-CHECK: PASS**；`pairs=22 rendered=18 blocked=4 replaced=0`；`DEDUPE: expected 14=18-4 actual 14=18-4 registry_ok=True`；exit=0 ✓ |
| 卡面路径 | `target/p0-0/render/g-manifest-v2.json` → **FAIL**（`pairs 需 22 行，实际 18`）→ AR171-2 |
| expected sha 全等 | present 18 / **mismatch 0**（用 `PAIR_SPEC.expected_file` 映射逐对重算；g-manifest-full entry 无 expected_file 字段，install-plan "经 v2 映射"措辞属实现细节）✓ |
| expected full unique | expected 目录 **22 文件 / unique 18** ✓（stg02↔title 组拆开） |
| 4 blocked | ending-1/2/3/fade-clear 仍 `expected_sha256=""`、不安装 ✓；expected_matches 0/18 ✓ |
| 两跑 | 18/18 deterministic ✓ |
| 备份 | `g-manifest-v2.json.pre-task165.bak` 存在（旧 entry：scene stg03-timeline/frame 3404/actual ebc6372f…）✓ |

---

## 5. `oracle/screens` 变更面（A）

`git status --porcelain oracle/screens` + `git diff --stat`：**恰好 4 件 M + 1 目录 untracked**，无其它对：
- `M oracle/screens/actual/l4_stg02-start.png`（Bin 246083→246083；新 sha 27fa）
- `M oracle/screens/expected/l4_stg02-start.png`（Bin 258→1411）
- `M oracle/screens/diff/l4_stg02-start.diff.png`（Bin 1121→2429；红点 2352）
- `M oracle/screens/visual-report.md`（仅 902→2352 两处，见 `git diff`）
- `?? oracle/screens/expected/.archive/`（仅 1 文件 258B；`git check-ignore` 未忽略，可随批提交）
- `oracle/screens/whitelist-application.md` 未变；其它 expected/actual/diff 文件未动 ⇒ **无越权** ✓

---

## 6. 4 份 dated 文档一致性（A/B）

| 文档 | 关键点核验 |
|---|---|
| `p0-0-expected-install-plan.md` §8（L199-231） | diff=2352/ratio .038281/bbox(4,9,249,239)、expected 44e7800d/1411B、archive 20261020-000358/258B 788d、pair gs5-anchor-render:3450:stg02-start、mapping 22 行、unique18、validate PASS 块、blocked 不安装 ✓（L208 idx 前缀 `436120b680f7c3c2…` 与真实前 16 位一致，✓） |
| `p0-0-readjudication-gui.md` §9（L130-139） | #14 Option B/2352/44e7800d/27fa+436120b6、archive、unique18/registry 等式、expected_matches 0 ✓ |
| `l4-baseline-freeze-plan.md` §7.5-7.7（L1537-1546） | 同上（2352/归档/unique/validate 行）✓ |
| `p0-0-groupD-s1-source.md` §8（L183-190） | 同上（#14/归档/2352/unique18/validate）✓ |
| **残留** | `team-ops-ledger.md` L68 **2076px**（AR171-1）——唯一不一致点 |

---

## 7. task-170 工作区自证（独立重导）

| 项 | 复核结果 |
|---|---|
| 17/17 traces → phase text | 17 个 trace 均有 `analysis/phase/<scene>-text.jsonl`（0 缺失）；5 个真实写集 + 12 个 0B（`e3b0c442…`）✓ |
| 三大户前后 sha | 当前实测 = `phase_before` = `phase_after`：gameplay `f05ce843840e1cb6…`(39,044,268B)、stg03 `fdad599d24f2a7c7…`(1,268,980,076B)、stg08 `255883bd72d9f0b5…`(339,266,155B) ✓ |
| 其它真实写集 | tas-stg08b2-direct `cd067aae…`（before/after/cur 同）、tas-stg08b2-dense `d464ab33…`（新建，after=cur）✓ |
| rom 链接 | `target/p0-0/rom/gunnac_cn.nes` 262,160B sha `4d76901c…` = `rom/gunnac_cn.nes` 同 sha；`fsutil hardlink list` 显示两路径为同一 hardlink ✓ |
| 2 对重渲 vs g_run1 | `workspace-check/{gameplay-stage1_f01260,l4_stg03-start}` png+idx **2/2 全同**（独立 sha）✓ |
| overlay text | `analysis/phase/gs5-anchor-render-text.jsonl` 81,322,458B sha `fbe7c209279d97f9…`（=t165 step0 记载）✓ |
| 17/17 清单 | `workspace-rebuild.json` traces 17 个与 §2 表一致；overlay 行另计（AR171-6 Info） |

---

## 8. `b100c31` 的 `re_render` 例外审（纪律红线）

- 变更：`PAIR_SPEC` **仅 #14** 加 `re_render: True`；`build_manifest` 对 `spec.get("re_render")` 的 pair **跳过 g-manifest-v2 run1 sha 交叉否决**（其余 21 对仍严格）；`expected_file` 核对未跳过；`--validate`/determinism/expected sha/schema/dedupe/registry 均未改；单测 10/10（我用 `python -m pytest tools/test_l4_render_all.py -q` 实跑 **10 passed in 0.34s**），commit 含 `Evidence:`/`Confidence: 95`。
- **评估**：①范围窄（单 pair、单检查项）；②可追溯（注释+commit message+单测断言+旧 sha 记录）；③未改 gate/validate、未动 expected/threshold；④豁免对象是 **gitignored 旧 18 行 artifact**（其 run1 sha 本就因 Option B 重渲而变）⇒ **不构成"为通过校验放宽判定"的红线**，属"陈旧基线 vs 新重渲"的工程例外。
- **建议（收口后）**：把 `g-manifest-v2.json` #14 run1 sha 刷新为 `27fa3737…`（或直接把基线切到 `g-manifest-full.json`）后**移除 `re_render` 布尔**；若保留，建议改成"仅允许历史 sha=`ebc6372f…`"的显式 allowlist，并加"其余 pair 仍严格核对"的反例测试——避免未来 #14 任何重渲漂移被永久静默。
- 另：`g-manifest-v2.json` #14 的 `idx_sha` 尾段错值 + `diff_pixels=2076` 未更新（AR171-3），与"entry 已更新"的自述不完全一致（target/ gitignored，不入 commit）。

---

## 9. 00:08 重写来源披露（如实）

- **本会话未运行 `tools/diff_visual.py` CLI**：复核中仅 `import diff_visual.compare/decode_png`（纯读函数）与 `python -X utf8 tools/l4_render_all.py --validate …`（只读）；未调用 `main()`、未写 `oracle/screens/{diff,visual-report,whitelist-application}`。
- 观察时序：我首次查看时 diff/report 已是 **2424** 状态（`visual-report` 旧值 902→??），对应 = 新 expected vs **旧 tracked actual `164b6e48`**（`oracle/screens/actual/l4_stg02-start.png` 旧件，可复算：2424/0.039453）；**不是**我产生。随后 Lead 收口把 actual 更新为 `27fa3737` 并再生成 ⇒ **2352/2343+9**，diff png 红点=2352、git diff 仅两处（902→2352）✓。
- 结论：00:08 重写由其他成员/CLI 运行产生；终态已一致，无需回滚；披露无误报。

---

## 10. 复现命令（只读）

```powershell
# 1) f3450 两跑 sha
python -c "import hashlib,pathlib; [print(p,hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()) for p in ('target/p0-0/render/g_run1/l4_stg02-start.png','target/p0-0/render/g_run2/l4_stg02-start.png')]"
python -c "import hashlib,pathlib; [print(p,hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()) for p in ('target/p0-0/render/g_run1/l4_stg02-start.idx.json','target/p0-0/render/g_run2/l4_stg02-start.idx.json')]"
# 2) 0 容差 diff 复算（2352/0.038281/(4,9,249,239)；条带内9/外2343）
python -c "import sys;sys.path.insert(0,'tools');from diff_visual import compare;dp,e=compare(__import__('pathlib').Path('oracle/screens/expected/l4_stg02-start.png'),__import__('pathlib').Path('oracle/screens/actual/l4_stg02-start.png'));xs=[d[0] for d in dp];ys=[d[1] for d in dp];print(len(dp),len(dp)/61440,(min(xs),min(ys),max(xs),max(ys)),sum(1 for d in dp if d[1]<16),sum(1 for d in dp if d[1]>=16))"
# 3) validate（正确路径=full）
python -X utf8 tools/l4_render_all.py --validate target/p0-0/render/g-manifest-full.json; echo exit=$LASTEXITCODE
python -X utf8 tools/l4_render_all.py --validate target/p0-0/render/g-manifest-v2.json; echo exit=$LASTEXITCODE   # 预期 FAIL（18 行旧件）
# 4) expected/归档/HEAD blob
python -c "import hashlib,pathlib; [print(p,pathlib.Path(p).stat().st_size,hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()) for p in ('oracle/screens/expected/l4_stg02-start.png','oracle/screens/expected/.archive/20261020-000358/l4_stg02-start.png')]"
# 5) oracle/screens 面
git status --porcelain oracle/screens; git diff --stat -- oracle/screens
# 6) workspace 自证
python -c "import json,hashlib,pathlib;W=json.loads(pathlib.Path('analysis/repro/workspace-rebuild/workspace-rebuild.json').read_text(encoding='utf-8'));sh=lambda p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest();[print(s,sh('analysis/phase/%s-text.jsonl'%s)[:16],(W['phase_before'].get(s+'-text.jsonl') or {}).get('sha256','')[:16]) for s in ('gameplay-stage1','stg03-timeline','stg08-timeline')]"
fsutil hardlink list target\p0-0\rom\gunnac_cn.nes
# 7) 例外单测
python -X utf8 -m pytest tools/test_l4_render_all.py -q
# 8) 文档残留（2076/2352）
Select-String -Path docs/findings/team-ops-ledger.md,docs/findings/p0-0-expected-install-plan.md -Pattern '2076|2352'
```

---

## 11. 自查

- 只写本文件；未 git add/commit；未改代码/expected/threshold/gates；未跑 Mesen/gates check/status/visual；跑过许可的 pytest（10/10）与只读 `--validate`。
- 独立重导：sha/diff/bbox/条带分解/diff png 红点、validate PASS/FAIL 双路径、expected 映射 0 mismatch、unique 18、18/18 两跑、mapping 22 行、oracle/screens 面、4 文档行、工作区 text/rom/2 对/overlay、b100c31 代码+单测+commit 字段、2076/2424/2352 三值来源。
- 未证不发明：00:08 来源只能披露"非本会话"（无他人会话日志）；`g-manifest-v2` 陈旧字段属 gitignored artifact，不据此判 install 失败；安装数据面以 `g-manifest-full` + 磁盘文件为准。
- 建议顺序：AR171-1（1 行）→ 其余 Info；改完即达 **pass**，无需重跑；`b100c31` 例外建议在冻结收口后清理（非本批阻塞）。

【2026-10-20】mimo（verifier），task-171（终验）。
