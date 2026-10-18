# 对抗复核：L5 A1 档 spec 口径更正（task-60）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：spec-writer（task-60）
> 被审文件：`docs/spec/l5-all-clear.md`（diff **+61/−3**，工作树未提交；sha256 `87f35ad872d4…`，280 行）
> 复核方式：独立读 golden/anchors/trace/l5_s5.rs + grep gates/completeness + docs-links gate；本文件为唯一写入，
> 未 `git add`/`git commit`。工作树另有 task-44/58 的代码改动，按文件隔离，不属本复核。

## 0. VERDICT

**pass**（2 条非阻塞 findings；F1 建议提交前补证据指针）

- 数字链**全部可复算**：s5-all-clear 44=39+5、stg01-chain 17=16+1、tas-clear-full 10=10+0、
  anchors 50=44 golden+6 附加、trigger 双基数 38/7/5（50 基）与 32/7/5（44 golden 基）、
  evidence 33 traces+12 shots+5 none、`l5_s5.rs:68` 锁 `(39,5)`。
- 诚实性成立：5 占位=1 条 f=0 未命名 `checkpoint=null` Transition + `stg08-start/combat/clear/all-clear-95f9`；
  "39 真锚≠39 自然锚（仅 7 natural）"写清；历史 12/32 用删除线留档、未 rewrite。
- §2.5 冻结点与 `l5-gap-inventory.md` §1.3 **逐字一致**；Boss 击杀/自然关末/流末边界全缺、STG-08 5 占位/
  OBS-1 不立号、缺口根三条均与 gap-inventory 一致。
- 门禁口径：`gates/cli.py` **无 L5 条目**（grep 0 命中）；A2 未启用/显式 SKIP 待批准、本卡未改 gates；
  completeness **§2.3/§2.3.1 存在**。
- **docs-links PASS 335 md/429 links**；本卡**仅改 `docs/spec/l5-all-clear.md`**（`git status` 中 docs/gates/data
  仅该 spec；代码改动为并发 task-44/58）。

## 1. 数字复算（≥8 项，全部独立）

| # | 声明 | 独立结果 |
|---|---|---|
| 1 | `s5-all-clear.l5.jsonl` **44=39+5** | 44 行；f>0=**39**，f=0=**5** ✅ |
| 2 | `stg01-chain.l5.jsonl` **17=16+1** | 17 行；f>0=**16**，f=0=**1** ✅ |
| 3 | `tas-clear-full.l5.jsonl` **10=10+0** | 10 行；f>0=**10**，f=0=**0** ✅ |
| 4 | anchors **50=44+6** | 50 条；`golden:true`=**44**、附加=**6**（6 条全 injected）✅ |
| 5 | **50 锚基** trigger 7/38/5 | `Counter({'injected':38,'natural':7,'placeholder':5})` ✅ |
| 6 | **44 golden 基** 7/32/5 | golden 子集 `Counter({'injected':32,'natural':7,'placeholder':5})`；附加 6 条全 injected ✅ |
| 7 | evidence **33/12/5** | 按 gap-inventory §1.4 同命令（`'analysis/traces' in evidence` 优先，再 `shots/`，否则 none）：`Counter({'traces':33,'shots':12,'none':5})` ✅ |
| 8 | `l5_s5.rs:68` 锁 `(39,5)` | `assert_eq!((real, ph), (39, 5), "G-S5-6 回填口径：39 真锚 / 5 占位")` 实测 ✅ |
| 9 | golden 3 场景/44 事件/**36 命名检查点** | 44 行；checkpoint 非 null=**36**（8 null 中 7 为 f>0 Transition、1 为 f=0 占位）✅ |
| 10 | 5 占位构成 | f=0 五条 = `None`(null Transition)、`stg08-start`、`stg08-combat`、`stg08-clear`、`all-clear-95f9` ✅ |
| 11 | "无 stg08-exit 字面" | golden/anchors 中无 `stg08-exit` ✅ |

## 2. 诚实性

- §R-l5-8 更正注（158-172 行）明确：44=39+5、32 占位由 injected 回填、5 占位**仍未验证/f=0 fail-closed**、
  `checkpoint=null` 未命名 Transition + stg08 三点 + all-clear；**严禁写成已验证**。核对 f=0 行，完全一致 ✅。
- **"39 真锚≠39 自然锚"**：7 natural 明确列出（boot-black/title-in/menu-in/stg01-shop/stg01-exit/stg01-start/
  stg01-combat），其余 32 injected；写入"injected/placeholder 不作自然通关证据" ✅。
- **历史 12/32**：§0 表第 8 行 `~~12 真锚/32 占位~~` 删除线 + 2026-10-18 更正注；R-l5-8 旧注"7 natural + 5
  injected + 32 占位"保留并注明是 2026-10-15 时点值；未 rewrite 历史 ✅。
- 5 占位**没有**在 §2.5 表写"已验证"：STG-08 行写"5 占位 fail-closed"、Ending 写"all-clear-95f9 占位" ✅。

## 3. §2.5 已证程度/缺口

- 冻结点 **f5033/f6893/f9164/f11093/f13977/f15959** 与 `l5-gap-inventory.md` §1.3 第 23-24 行**逐字一致**；
  STG-01 负结果（15000f 4 死）与 gap-inventory 一致 ✅。
- **Boss 击杀/自然关末/流末边界**：§2.5.1 小结明确"每关 Boss 击杀、自然关末三联（`$27 bit7`+`$2F` 倒数+
  `INC $180`）、流末记录（`$CA`）边界均未观测"；STG-08/Ending 无动态 trace；与 gap-inventory §0/§1.3 一致 ✅。
- **STG-08 5 占位/OBS-1**：写"OBS-1 候选区不立号"、5 占位 fail-closed；与 gap-inventory 第 26-29 行一致 ✅。
- **缺口根三条**（①输入轨 ②Boss/流末/关末 trace ③stg08/ending 合法来源）+ G-L5-1..4 映射：与 gap-inventory
  §0 第 5 条/§2 一致；另加可复现性/G-L5-7 门禁行，属补充（不冲突）✅。
- 矩阵引用 [completeness.md §2.3/§2.3.1](../spec/completeness.md)：实读存在（L171/L186），且 spec 明示不新增分母格 ✅。

## 4. 门禁/纪律

- `gates/cli.py` grep `L5|l5` **0 命中**；spec 写"12 项无 L5 条目"与当前 gate（PASS=11 + SKIP=1=12）一致；
  A2 未启用/显式 SKIP 待批准、**本卡未改 gates**（`git status gates` 无改动）✅。
- **docs-links PASS 335 md/429 links**（全量 check 当前 PASS=11 FAIL=0 SKIP=1）；spec 内相对链接均有效。
- **只改该 spec**：`git status --short docs/spec/l5-all-clear.md gates data` 仅该 spec（+61/−3）；其他代码改动
  为并发 task-44/58，无混合 ✅。
- 与 gap-inventory 的引用链存在且有效；本 spec 不新增数据/基线/gates 改动 ✅。

## 5. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- **F1（低，建议补指针）**：§2.5 冻结点（f5033/f6893/f9164/f11093/f13977/f15959）与 gap-inventory 文本
  一致，但**当前仓库 tracked golden/anchors 不含这些帧**（`s5-all-clear.l5.jsonl` 最大锚 f16221，`anchors.jsonl`
  f 列表也无这些值；动态 trace 在 `.gitignore` 的 `analysis/` 下）。按 task-62"与 trace 一致"的独立复核要求，
  只能确认**文档一致**、不能从 repo 独立复算到具体 trace 文件。建议在 §2.5 表每行补 `source_run`/analysis
  trace 文件名（或总注"见 gap-inventory §1.3 的 run 表"）——不阻塞，属证据指针精度。
- **F2（低，措辞）**：§R-l5-8 写"5 占位 = 1 条未命名 Transition 行（checkpoint=null）+ 4 个具名"，精确；
  但 golden 全表 `checkpoint=null` 共有 **8 行**（另 7 条 f>0 Transition 真锚）。建议把"5 占位中的 1 条"
  写成"f=0 占位中的 1 条"，避免读者把 null 总数误读为 5/1（不影响任何数字结论）。

### 建议（P3）
- **F3**：§2.5.2 evidence 33/12/5 与 gap-inventory 命令一致（混合 evidence 计 traces）；建议在 §4 复现命令
  处保留与 gap-inventory §1.4 相同的命令/schema，避免日后有人按"startswith"重新统计得到 23/22/5。
- **F4**：§2.5 的 freeze 总值/表沿引 gap-inventory；若后续 analysis trace 入库，建议同批升 A 并补 trace id。
- **F5**：§0 表第 8 行历史 12/32 已删除线留档；建议保留当前"时点值→现口径"的日期注（已有）即可。

## 6. 命令输出摘要

```text
s5-all-clear.l5.jsonl: 44 rows; f>0=39, f=0=5; 36 named checkpoints; null=8（f>0 7 + f=0 1）
stg01-chain.l5.jsonl: 17 rows; f>0=16, f=0=1
tas-clear-full.l5.jsonl: 10 rows; f>0=10, f=0=0
anchors: 50 = golden 44 + extra 6；trigger all = {injected:38, natural:7, placeholder:5}
         trigger golden = {injected:32, natural:7, placeholder:5}；extra 6 全 injected
evidence (gap-inventory 同命令) = Counter({'traces':33,'shots':12,'none':5})
l5_s5.rs:68 assert_eq!((real, ph), (39, 5), "G-S5-6 回填口径：39 真锚 / 5 占位")
f=0 行：None, stg08-start, stg08-combat, stg08-clear, all-clear-95f9
gates/cli.py grep L5/l5 = 0 命中；gates 无 L5 条目
completeness.md §2.3 L171 / §2.3.1 L186 存在
docs-links PASS 335 md/429 links；全量 check PASS=11 FAIL=0 SKIP=1
git status: docs/spec/l5-all-clear.md 仅此 spec（+61/−3）
§2.5 freeze points = gap-inventory §1.3 L23-24 文本一致（tracked trace 无这些 f，F1）
```

## 7. 结论

**pass**。数字链（44/17/10、50=44+6、双基数 38/7/5 与 32/7/5、evidence 33/12/5、l5_s5 (39,5)）、
诚实性（5 占位 fail-closed、39≠自然、12/32 删除线留档）、§2.5 缺口面、门禁/链接/改动范围均核实通过；
F1（冻结点缺 trace 指针）与 F2（null 计数措辞）为非阻塞建议，可在提交前顺手补，不需重审。
