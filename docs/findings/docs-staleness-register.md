# 过期文档台账（docs-staleness-register）

> 日期：2026-10-18｜角色：spec-writer（task-40）｜时点 HEAD：`b92139a`｜性质：**纪律台账（docs-only）**，不改 `data/`、不重导、不修文档本体。
> 用途：把"已知过期/部分过期、但仍在仓库里可被引用"的文档集中登记，避免下游拿旧快照当现状（AGENTS.md §3"数字可独立复算"、§3.6"原地更正"）。
> **使用规则**：引用下表文档的数字前先查本表；**登记 ≠ 修复**——修复须另卡执行并在本表注销/更新（带 commit 指针）。本表"过期依据"一律给可复现命令或已入库证据指针。

## 1. 汇总表

| # | 路径 | 过期形态 | 过期依据（简） | 能否本机重导 | 建议卡 |
|---|---|---|---|---|---|
| S1 | `data/status/rewrite-workset.json`（镜像） | **快照过期**：镜像=2026-10-15 重导，落后 HEAD **7 段** | 重导实测：HEAD `b92139a` 下 `core-cited 61 / spec-cited 318`，镜像 `core-cited 54 / spec-cited 325`；delta 7 段全为 spec→core | **部分**：只读重导到 `%TEMP%` 可行（见 §3-A）；正式重导会写 `data/`+`analysis/` ⇒ 须 data 写授权 | 另卡重导入库（Lead 授权），同步 S2 文档 |
| S2 | `docs/spec/rewrite-workset.md` §4（L154-170） | **旧快照**：自标"2026-10-15 快照"；`unexplored 247 / ≥core 49` 与现镜像（unexplored 0 / core 54）及 HEAD（core 61）均不符 | 文档 L154 自述；[rewrite-completion-audit](../findings/rewrite-completion-audit.md) §9 第 6 条（建议重跑生成器并同步 §3.3/§4，预期 core 61/spec 318） | **是**（同一生成器；但文档回填需人工改） | 与 S1 同卡：重导入库 + §3.3/§4 回填 |
| S3 | `docs/spec/m3-inventory.md` | **严重过期**（2026-10-12 v1）：§2 DAG 写 `shots/enemy/stage ❌ 未动`；底数知识库 10/L2 54/测试 17/PASS=10 SKIP=1；§3 Stage A 表含"白名单候选用户确认"旧前置 | [rewrite-completion-audit](../findings/rewrite-completion-audit.md) §1.4（"严重过期，**禁止**用作当前子系统状态源"）；本卡复算：现 `shot.rs 460 行/enemy.rs 1161/stage.rs 265` + L2 121 例 | **否**（人工盘点文档，无生成器；数字可重采、整篇须重写/退役） | `docs(spec)`：m3-inventory 退役重定向（标 superseded → rewrite-workset 现快照 + completeness.md + platform-shell.md），或 v2 重写 |
| S4 | `data/status/coverage_map.json` | **过期**（`generated: 2026-10-12`，t34）：`stats.l2_cases=54`、`rules_total=44` | [rewrite-completion-audit](../findings/rewrite-completion-audit.md) §1.4；本卡复算：`oracle/subroutines/l2_cases.jsonl` 现 **121** 例（54≠121） | **否**（仓库内无生成器脚本；为 t34 一次性审计的机器可读版，人读版 `docs/findings/coverage-audit.md`） | 另卡重建 coverage_map（schema 1 沿用）并同步引用方 |
| S5 | `docs/findings/l4-final-localization-ending1.md` | **时点失效**：标题"7753 态残差 7380px/261 格"；现行 ending-1 diff = **4552** | [rewrite-completion-audit](../findings/rewrite-completion-audit.md) §4.4；[l4-zero-tolerance-attack-plan](../findings/l4-zero-tolerance-attack-plan.md) §1.2（判"时点失效"）；现行 `oracle/screens/visual-report.md` L15 `l4_ending-1.png: 差异像素 4552` | **否**（须 P0-0 新基线 + 重做定位） | P0-0 后 P0-3 重做定位账；在此之前引用须带"7753 态时点/失效" |
| S6 | `docs/findings/residual-pixel-ledger.md` | **部分失效**：AR2-A（自机 −$30 锚点）一族已撤销；"每像素可归因/全族勾销"判据不成立（其余 9 族定位仍有效） | [rewrite-completion-audit](../findings/rewrite-completion-audit.md) §4.4；l4-zero-tolerance-attack-plan §1.2；`adversarial-audit-l4.md` §0（AR2-A 撤销） | **否**（须 P0-0 后重做） | P0-1/P0-3 后重做；先引用时须带"AR2-A 已撤销"限定 |
| S7 | 生成物文案：`oracle/screens/whitelist-application.md`、`oracle/screens/visual-report.md`、`tools/diff_visual.py` 文案 | **口径过期**：仍列 W-C1/W-C4 为"强候选/申请项/候选未启用"；W-C1/W-C4 已于 2026-10-18 用户裁定撤销 | 撤销落档 `7625df8`（[l4-visual-plan](../spec/l4-visual-plan.md) §2 撤销注）；复核 [adversarial-review-wc1-revocation](../findings/adversarial-review-wc1-revocation.md) pass；生成物为 `tools/diff_visual.py` 输出（L189-190 报告头、L223-225 候选段、L227-236 申请清单），**按纪律不直接改生成物** | **是**：改 tools 文案后 `python tools/diff_visual.py` 重生成（gates visual 亦会重写） | `tools(docs)`：diff_visual 生成文案同步（W-C1/W-C4 撤销 + strict "外差异"提示；另卡） |

**已核对为现行（无需处理，防误标）**：`oracle/screens/visual-report.md` 的**数值**现行（`ab6383c` 生成，与 22 对实测一致，[rewrite-completion-audit](../findings/rewrite-completion-audit.md) §1.4）；`data/status/baselines/baseline.json` 现行（knowledge 23 / tests 366 / naming 206 / completeness 19）；`data/status/completeness_matrix.json` 与 ratchet 同步（audit §1.4，但"证据覆盖"≠移植完成）。

## 2. 逐条要点（补充影响面）

- **S1/S2 影响**：任何"core-cited 54 段 / spec-cited 325 段"的引用都是**镜像时点**（2026-10-15）值；HEAD 现为 61/318。delta 7 段（spec→core）：`bank00:8185-81A2`、`bank00:82EF-82FC`、`bank00:8DDF-8EC7`、`bank00:929E-9319`、`bank00:93C1-9409`、`bank14:CCD3-CD07`、`bank14:D679-D6C5`（与 [rewrite-completion-audit](../findings/rewrite-completion-audit.md) §1.3 逐段一致）。注意"7 段 delta"本身是**重导结果**，不是把镜像改小。
- **S3 影响**：m3-inventory 的 DAG **顺序**仍有历史参考价值，但其**状态列/底数/Stage A 现状**不可当现状；其 §3 第 5 行"L4 白名单候选用户确认"前置已随 W-C1/W-C4 撤销而作废（现行 L4 判据=0 容差，见 [master-plan §6](../master-plan.md) 与 [decision-l4-zero-tolerance-ruling](../knowledge/decision-l4-zero-tolerance-ruling.md)）。
- **S4 影响**：L2 覆盖引用一律以 `oracle/subroutines/l2_cases.jsonl`（121 例）为准；coverage_map 只在追溯 t34 审计口径时使用。
- **S5/S6 影响**：两者都属 L4 定位账，**必须绑定 P0-0 可复现基线**后重做；在此之前只可作"历史时点材料"，不得当现行残差账。
- **S7 影响**：读者读到生成物会看到与 spec 矛盾的"白名单候选/申请"文案；**修复入口是 tools 文案**（改 `tools/diff_visual.py` 对应行后再重生成），不要手改 `oracle/screens/*` 生成物。

## 3. 复现命令（只读；工作目录=仓库根；本机用 `python`）

**A. workset HEAD 只读重导到 `%TEMP%`（不写仓库）**——本卡执行过，输出即上表 S1 数字：

```powershell
@'
import sys, json, tempfile, pathlib, io, contextlib, collections
sys.path.insert(0,'tools')
import workset_inventory as wi
tmp = pathlib.Path(tempfile.mkdtemp(prefix='ws_reg_'))
wi.OUT_DIR = tmp/'workset'; wi.MIRROR = tmp/'rewrite-workset.json'
with contextlib.redirect_stdout(io.StringIO()):
    wi.main()
new = json.load(open(tmp/'rewrite-workset.json', encoding='utf-8'))
old = json.load(open('data/status/rewrite-workset.json', encoding='utf-8'))
print('HEAD   ', collections.Counter(s['status'] for s in new['segments']))
print('mirror ', collections.Counter(s['status'] for s in old['segments']))
om = {s['seg']: s for s in old['segments']}
for s in new['segments']:
    if om[s['seg']]['status'] != s['status']:
        print('DELTA', s['seg'], om[s['seg']]['status'], '->', s['status'])
'@ | Set-Content -Path $env:TEMP\ws_rederive.py -Encoding UTF8
python $env:TEMP\ws_rederive.py
# 期望 HEAD   {'L2-diff':15,'core-cited':61,'spec-cited':318,'findings-cited':0,'unexplored':0}
#      mirror {'L2-diff':15,'core-cited':54,'spec-cited':325}
#      DELTA 7 行（见 §2 S1/S2）
```

**B. 镜像/文档时点与计数核对（S1/S2/S3/S4）**

```bash
git log -1 --format='%h %ad %s' --date=iso -- data/status/rewrite-workset.json   # 61ed634 2026-10-15 21:38
git log -1 --format='%h %ad %s' --date=iso -- data/status/coverage_map.json      # c729ebd 2026-10-12 17:59
python -c "import json;d=json.load(open('data/status/coverage_map.json',encoding='utf-8'));print('generated',d['generated'],'l2_cases',len(d['l2_cases']),'rules_total',d['stats']['rules_total'])"
python -c "print('l2_cases.jsonl =',sum(1 for l in open('oracle/subroutines/l2_cases.jsonl',encoding='utf-8') if l.strip()))"   # 121
grep -n "2026-10-15 快照\|unexplored | 247\|≥ core-cited" docs/spec/rewrite-workset.md | head
grep -n "shots.*未动\|enemy.*未动\|stage.*未动\|L2 用例 \*\*54\*\*\|知识库 10 条" docs/spec/m3-inventory.md
python -c "import pathlib;[print(f,(pathlib.Path('crates/game-core/src')/f).read_text(encoding='utf-8').count(chr(10))) for f in ['shot.rs','enemy.rs','stage.rs']]"
```

**C. S5/S6 时点核对**

```bash
head -3 docs/findings/l4-final-localization-ending1.md     # "7753态残差7380px/261格"
grep -n "l4_ending-1.png" oracle/screens/visual-report.md   # 现行 diff 4552
grep -n "AR2-A" docs/findings/residual-pixel-ledger.md | head   # 撤销注
```

**D. S7 生成物旧文案核对**

```bash
grep -n "W-C1\|W-C4" oracle/screens/whitelist-application.md oracle/screens/visual-report.md | head
grep -n "W-C1\|W-C4" tools/diff_visual.py | head            # 生成源（L189-190/223-225/227-236）
```

## 4. 维护纪律 / 变更记录

- 新增过期项：在本表加行（路径/形态/依据/能否重导/建议卡），**不改被登记文档本体**（属另卡）。
- 注销过期项：修复 commit 入库后在对应行加「【YYYY-MM-DD 已修复】commit `<hash>`」，不删历史（AGENTS.md §3.6）。
- 本表不复制易变数字的"最新值"作为结论，只给"过期依据 + 复算命令"，让读者自行复算。

| 日期 | 变更 |
|---|---|
| 2026-10-18 | 建表（task-40）：登记 S1-S7（5 项来自任务卡指定 + S6/S7 为同类已知过期扩展），每项给证据指针/复算命令/建议卡；S1 的 7 段 delta 由只读重导实测确认。 |
