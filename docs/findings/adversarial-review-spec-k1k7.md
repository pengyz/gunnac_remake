# 对抗复核：spec 口径修正（K1/K7）——adversarial-review-spec-k1k7

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不得复用被审方结论**）
> 被审对象（task-13 产出，**工作区未提交**）：`docs/spec/l4-signing-criteria.md`（134 行）与 `docs/spec/l4-visual-plan.md`（408 行）的未提交改动；`git diff --stat` = **2 files changed, +66 / −2**；复核时 HEAD `9bb0fd9`。
> 纪律：AGENTS.md §3/§5（证据 A/B/C 分级、数字可独立复算、未证不发明、对抗复核）。本文只写本文件；未改被审文档、未 git add/commit。
> 复核范围：K1 撤销链/双值、K7 分母语义与 dg 来源、互链闭环、纪律（历史留档/无三值并存/无来源残留/只改两文件/docs-links 真 PASS）。

---

## 0. VERDICT：**needs_revision（低风险；仅 1 处必改 F1，修 1 句即 pass）**

- **被审方 4 条关键声明全部核实为真**：①43.0% 确有历史推导 `1761/4099=42.96%≈43.0%`（`0438b2c` 引入 / `4d5e880` 快照在案），后随源行被 `b4586e7` 替换（1761→4675）与 `22d896c` 归正 4236 而失效 ⇒ 撤销为签署面数字但留推导链；②现行双值 `4236/7680=55.16%`（`22d896c`）、`4675/7680=60.87%`（`33348f2`）逐条复算+提交体佐证；③`dg=8046` 有代码指针（`render_frame.rs:306`「奇偶 half=8046/864」）且 `864/7680=11.25%` vs `864/8046=10.74%` 复算一致；④互链闭环（decision 条目 ↔ 两 spec ↔ master-plan §6 ↔ 24093be），终态 `docs-links` PASS。
- **被证伪 1 处（F1，唯一必改）**：`l4-signing-criteria.md` L67「原表"11%"**系形合率舍入值**」与 `l4-visual-plan.md` L369-373 的 K7 更正（原表公式 `864/7680=11.25%` 系**占阈值比**，形合率须 `/8046=10.74%`）**口径不一致**——同一历史值 11% 在两份已更正文件中被定性为两个不同度量。
- **过程观察（非阻塞，已消解）**：复核中途 `python gates/cli.py check` 曾 **FAIL**（`docs-links`：仓库根遗留 `.tmp_signing_1be66e5.md` / `.tmp_visual_1be66e5.md` 的链接目标不存在）；终态两文件已不在工作区，**`docs-links` PASS（313 md / 289 links）**；13:04 曾全量 `check` **PASS=11 FAIL=0 SKIP=1**。复核末尾全量 `check` 的唯一 FAIL 项 `rust-tests`（`game-shell` E0432 编译错）系**其他任务并发施工**所致，与本被审任务无关。建议提交前确认无 `.tmp_*` 残留。
- 其余：历史主张原地留档（无删除）、无三值并存、无未给来源的新数字、只改两文件（tracked diff 仅 2 文件）——均核实通过。

---

## 1. K1 复核：43.0% 推导链与双值（声明 1/2）

### 1.1 43.0% 不是编造——原推导确实存在

| 步 | 证据（我独立执行） | 结论 |
|---|---|---|
| 引入 | `git log -S'1761/4099' -- docs/spec/l4-signing-criteria.md` → `0438b2c`（草稿 v1）、`b4586e7`（终版入面） | 原行由 `0438b2c` 引入 |
| 快照 | `git show 4d5e880:docs/spec/l4-signing-criteria.md` L15：`| 双有（形合） | 1761 / **43.0% 对位率**（1761/4099） | 实测 | ✓ 算术精确 |` | 43.0% = **1761/4099**，来源在案 |
| 算术 | `1761/4099 = 42.9617% ≈ 43.0%` | 复算吻合 |
| 失效 | `git show b4586e7 -- docs/spec/l4-signing-criteria.md` 的 diff：删除 `| 双有（形合） | 1761 / 43.0% 对位率（1761/4099） |`，新增 `| 双有（形合归位） | **4675 = 61%** … 超 4099 阈 ✓ |`；同提交 §4 item1 `对位率 43.0%`→`61%` | 源行被替换（1761→4675）属实 |
| 再归正 | `git show -s --format='%B' 22d896c` 提交体：「双有 4236（55%）/内容 4754……双有轨迹：525→934→3865→4675→**4236**（量控后归正）」 | 4236 归正于 `22d896c`，有提交体佐证 |
| 落文 | `git log -S'4236' -- docs/spec/l4-signing-criteria.md` → **`548086d`**（终签 docs 提交）首次把 4236 写进 spec；`22d896c` 本身未改 spec | 「22d896c 实测、548086d 落文」链条自洽 |

⇒ 被审注（L40-58）「原推导 1761/4099=42.96%≈43.0%……源行已在 b4586e7 被替换（1761→4675），再经 22d896c 归正 4236，当前文件内已无可复算来源」**逐句成立**。43.0% 是「历史实测+分母口径不当（4099 是"超 4099 阈"的阈值，不是内容量）」而非编造；撤销合理，且历史主张以删除线+注留档。
> 口径注（不影响结论）：原推导的分母 4099 是"对位阈"而非内容量，这本身也是撤销该值的正当理由之一，注中未点明，可在 L46 补半句（非阻塞）。

### 1.2 现行双值逐条复算

| 值 | 公式 | 复算 | 提交体佐证 |
|---|---|---|---|
| 量控后 | `4236/7680` | **55.1562% ≈ 55.16%** | `22d896c` body「双有 4236（55%）」 |
| 量控前 | `4675/7680` | **60.8724% ≈ 60.87%** | `33348f2` body「双有 4675（61%！超 4099 阈 ✓）」 |
| 9896 态历史 | `(14344−9896)/14344` | **31.0095% ≈ 31.0%** | `4d5e880` 快照 L13 同式 |
| 9896 态历史 | `51544/61440`（`61440−9896=51544`） | **83.8932% ≈ 83.9%** | `4d5e880` 快照 L14 |

⇒ 「双值注时点」口径成立：55.16%（量控后，`22d896c`）/ 60.87%（量控前，`33348f2`），不构成三值并存（43.0% 已撤销并标删除线；61% 为 60.87% 的历史取整值且原地注时点）。
> 证据级别注（非阻塞）：双有 `4236/4675` 的「实测」出处是提交体自述（B 级）+ `render_frame.rs` 历史调参注释（如 L319-341 记录 双有 1302/1718/1761/2204 等），仓库内**没有**可一键复算双有的脚本/产物；均为**本次修订前既有**证据面，不构成本次未证发明，但后续若再引用建议补可复算命令。

---

## 2. K7 复核：7680 分母语义与 dg 来源（声明 3）

### 2.1 7680 是阈值而非内容量——有据

- `l4-visual-plan.md` L360：`diff | 差异像素 | **14344**（ending-1 基准 7680）`；判定式 `4552<7680`（`l4-signing-criteria.md` §1）——**7680 = ending-1 判定基准**。
- 内容量是独立度量：`l4-visual-plan.md` L356-362【2026-10-17 度量标定】表：`内容量 dg | 渲染非零像素（内容面） | **8046**`；`双有=864`；表头「当前态（8046 态/奇偶 half）」。
- dg=8046 的代码指针核实：`crates/game-desktop/src/bin/render_frame.rs` L306 = `// 【半行实证回退】奇偶 half=8046/864（最佳）——x 配对` ✓（文件存在、行号命中、注释原文一致）。
- 复核线同式先例：`docs/findings/adversarial-review-102.md` §B.1 L47：`| 形合率 | 864/8046=**10.7%**≈11% | "双有 864=11% 形合率" | ✓ 舍入成立 |`——即 **形合率分母=内容量** 早有复核先例 ✓。

### 2.2 数值复算

`864/7680 = 11.2500%`（占阈值比）；`864/8046 = 10.7383% ≈ 10.74%`（形合率）→ 被审注的 `11.25% / 10.74%` 双值**逐字正确**；L363 表内「864/7680=11% ~~形合率~~ **占阈值比**」保留原值+删除线+更名，历史留档合规。

### 2.3 残留检查

- 全仓 `形合率` 只出现在：两 spec 的更正注（正确用法）、`adversarial-review-102.md`（先例）、本复核方/审计 findings（历史引用）——**spec 内已无「形合率=双有/7680」的现行断言残留**。
- 全仓 `43.0%` 只出现在：`l4-signing-criteria.md` L37（删除线+撤销注）、L40/43/44（撤销说明）、审计/复核 findings（历史记录）——**无未撤销的现行 43.0% 断言残留**。

### 2.4 **F1（唯一必改）**：两份已更正文件对历史 11% 的定性互相矛盾

| 文件:行 | 现文 | 问题 |
|---|---|---|
| `l4-signing-criteria.md` L67 | `→ 864/7680=11.25%  864/8046=10.74%（前者=占阈值比，后者=形合率；l4-visual-plan.md 原表"11%"系形合率舍入值）。` | 「原表 11% 系**形合率**舍入值」——但原表公式写的是 `864/7680=11%`（=占阈值比 11.25% 的舍入），并非 `/8046` 形合率；且与同注前半句"前者=占阈值比"自相矛盾 |
| `l4-visual-plan.md` L369-373 | 更正注：原表「864/7680=11% 形合率」= **分母语义误用**；`864/7680` 只能称 **占阈值比（11.25%）**；形合率须 `/8046=10.74%` | 该表述正确、自洽 |

两者对同一历史值 11% 的归属（形合率 vs 占阈值比）不一致；虽两数均舍入为 11%、不影响任何下游数字，但本任务的全部目的就是消除该口径混淆，故须统一。
> 修复建议（1 句）：`l4-signing-criteria.md` L67 句尾改为「…原表"11%"系 **864/7680=11.25% 的舍入**——原标签"形合率"误用分母，更正后引用须带分母名（占阈值比≠形合率）」。改后两文件口径一致，即 pass。

---

## 3. 互链闭环复核（声明 4）

链接图（我按文件实际链接逐一核对，目标文件均存在；终态 `docs-links` PASS 312 md/285 links）：

- `l4-visual-plan.md` L384-399【2026-10-18 判据互链注】：显式记录 **2026-10-16 NOT READY（L352 原文保留）→ 2026-10-18 终局（L401-408 原文保留）** 的判据切换，并链接 `[master-plan §6](../master-plan.md)`、`[decision-l4-zero-tolerance-ruling](../knowledge/decision-l4-zero-tolerance-ruling.md)`（两次）、`l4-signing-criteria.md`、`[rewrite-completion-audit](../findings/rewrite-completion-audit.md)`；援引 commit `24093be`。
- `l4-signing-criteria.md` L10-18【用户裁定·Lead 记录】链接 `master-plan §6`、`l4-visual-plan §1`；K1 注（L40）链接复核报告/审计；K7 注（L60-67）链接 `l4-visual-plan`、`adversarial-review-102`；`24093be` 在 L40/L58/L129 引用。
- `master-plan.md` L75-78【2026-10-18 用户裁定】维持 0 容差、判定"带声明签署不构成 L4 达成"、引用 `rewrite-completion-audit` §4。
- `docs/knowledge/decision-l4-zero-tolerance-ruling.md` L12/L14 链接回两 spec + master-plan；其证据指针段引用 signing §1/§4/§6、visual-plan §1、audit §4、visual-report。
  ⇒ **闭环成立**：「判据切换无记录」裂缝已被 visual-plan 互链注 + decision 条目 + 两文件裁定注三方补上；10-16 NOT READY 与 10-18 终局原文均保留、只加注不改写。
> 非阻塞建议（F3）：`l4-signing-criteria.md` 顶部裁定注（L10-18）目前未直链 decision 知识条目（仅由 visual-plan 互链反达）；建议 L13 后补一条 `[decision-l4-zero-tolerance-ruling](../knowledge/decision-l4-zero-tolerance-ruling.md)` 直链，减少传递依赖。

---

## 4. 纪律复核

1. **只改 docs/spec/ 两文件**：`git status --porcelain` 的 tracked 修改恰为两份 spec；`git --no-pager diff --stat docs/spec/` = `2 files changed, 66 insertions(+), 2 deletions(-)` ✓。另有其他并行任务的未跟踪/暂存文件（`docs/findings/adversarial-review-shellwin-v1.md`、`docs/findings/l4-zero-tolerance-attack-plan.md`、staged `docs/findings/plan-playable-loop.md`），**不属本任务写范围**。
2. **历史主张原地留档不删**：`git diff -U0` 删除行仅 2 行（`l4-signing-criteria` 数字纪律注原句、`l4-visual-plan` 度量表原单元格），其内容均在新文本中原样保留（`~~43.0%~~`、`864/7680=11% ~~形合率~~`）；10-16 NOT READY 段与 10-18 终局段原文未动 ✓。
3. **无未证发明**：新增数字均带算式（`4236/7680`、`4675/7680`、`864/7680`、`864/8046`、`(14344−9896)/14344`、`51544/61440`）或文件/提交指针（`render_frame.rs:306`、`4d5e880`/`b4586e7`/`22d896c`/`33348f2`/`548086d`、`adversarial-review-102 §B.1`）。
4. **无三值并存**：43.0% 删除线+撤销注；55.16%/60.87% 均注时点与提交；61% 保留为 60.87% 的历史取整值并注"以 60.87% 复算为准" ✓。
5. **docs-links 门禁**：见 §5——终态真 PASS；但复核中途曾因仓库根遗留 `.tmp_signing_1be66e5.md`/`.tmp_visual_1be66e5.md`（task-13 内部 diff 基线副本，错误地放在仓库根而非 `%TEMP%`）导致 `check` FAIL；二文件随后已不在工作区。**建议**：提交前 `git status` 确认无 `.tmp_*`；此类临时基线一律放系统临时目录。
6. **spec 自洽性（除 F1）**：K1/K7 注之间、与 §4 item1/§6 追加注、与 visual-plan 互链注交叉阅读无其他冲突（三处均把 55%/61% 限定为"占阈值比/时点值"，把"形合率"限定为 dg 分母）。

---

## 5. 我执行的命令与输出摘要

```text
# 1) 数字复算
1761/4099=42.9617% ; 4236/7680=55.1562% ; 4675/7680=60.8724%
864/7680=11.2500% ; 864/8046=10.7383% ; (14344-9896)/14344=31.0095% ; 51544/61440=83.8932%

# 2) 43.0% 推导链
git log -S'1761/4099' -- docs/spec/l4-signing-criteria.md          → 0438b2c / b4586e7
git show 4d5e880:docs/spec/l4-signing-criteria.md  (L15)           → 双有 1761 / 43.0% 对位率（1761/4099）
git show b4586e7 -- docs/spec/l4-signing-criteria.md               → -|双有 1761… |+4675 = 61%…；§4 item1 43.0%→61%
git show -s --format='%B' 22d896c                                  → 「双有 4236（55%）… 4552… 双有轨迹 …→4236」
git show -s --format='%B' 33348f2                                  → 「双有 4675（61%！超 4099 阈 ✓）…」
git log -S'4236' -- docs/spec/l4-signing-criteria.md               → 548086d（首次落文）

# 3) K7 来源
render_frame.rs:306 = "// 【半行实证回退】奇偶 half=8046/864（最佳）——x 配对"
l4-visual-plan.md L356-362 度量标定表：diff 14344（ending-1 基准 7680）/ 内容量 dg 8046 / 双有 864
adversarial-review-102.md §B.1 L47：形合率 864/8046=10.7%≈11% ✓ 舍入成立

# 4) 门禁（复核期间 HEAD 9bb0fd9→cce3bb1，工作区有并发施工）
# 4a) 13:02 check：FAIL ← docs-links（仓库根 .tmp_signing_1be66e5.md/.tmp_visual_1be66e5.md 的
#     ../master-plan.md、l4-visual-plan.md、../findings/rewrite-completion-audit.md、../knowledge/pattern-… 目标不存在）
# 4b) 13:04 两临时文件已不在工作区，check：SUMMARY PASS=11 FAIL=0 SKIP=1 TOLERATED=0；CHECK: PASS
# 4c) 13:06（本复核文件写入后）docs-links：PASS（313 个 md / 289 条本地链接全部有效）
#     同期全量 check 的 rust-tests 失败（cargo: E0432 could not compile game-shell）系**其他任务并发施工**
#     （game-shell/game-data 工作区改动）所致，非本被审任务；本任务门禁要求=docs-links，已 PASS。

# 5) 残留扫描
grep '43\.0%' docs/spec docs/findings  → 仅删除线/撤销注/历史审计引用
grep '形合率' 两 spec                   → 仅正确用法（/dg）与更正注；无「形合率=双有/7680」现行断言
grep '形合率=双有/7680'  → 无现行残留（审计/findings 中为历史引用）
```

---

## 6. Findings 四分类与修正建议

### 6.1 口径问题
- **F1（必改，唯一阻塞）** `docs/spec/l4-signing-criteria.md` **L67**：「原表"11%"系形合率舍入值」与 `l4-visual-plan.md` L369-373 矛盾（原表公式 `864/7680` ⇒ 占阈值比 11.25%；形合率应为 `864/8046`=10.74%）。修法见 §2.4，一句替换即可。修后 verdict → pass。

### 6.2 纪律/过程
- **F2（非阻塞，终态已消解）** 仓库根遗留 `.tmp_signing_1be66e5.md`、`.tmp_visual_1be66e5.md`（任务内部 diff 基线副本）曾使 `gates/cli.py check` 的 docs-links **FAIL**；复核终态已不在工作区。建议：临时基线固定放 `%TEMP%`，提交前 `git status` 确认无 `.tmp_*`。

### 6.3 证据不足（均为既有面，非本次引入）
- **E1（低，非阻塞）** `双有 4236/4675` 的"实测"目前只有提交体自述（B 级）与 `render_frame.rs` 历史调参注释旁证，无仓库内可复算脚本/产物；本次引用沿用了既有签署面数字，不构成新增未证，但建议后续把双有/内容量度量固化为可复算命令。
- **E2（低，非阻塞）** K1 注（L46）未点明「原推导分母 4099 是阈值而非内容量」——这是撤销 43.0% 的另一个正当理由，补半句可让撤销理由更完备。

### 6.4 表述越权
- **无**。本次修订总体方向是收紧口径（0 容差裁定、签署=过程里程碑、对位率改名"占阈值比"、形合率改 dg 分母），未发现把未证写成已证或夸大完成度的表述；历史主张均以删除线/追加注留档。

### 6.5 互链建议（非阻塞）
- **F3** `l4-signing-criteria.md` L13 后补 decision 知识条目直链（现仅经 visual-plan 间接可达）。

---

## 7. 复核边界与不确定项

1. 只复核工作区未提交改动（HEAD `9bb0fd9`；两 spec 未提交）；若提交前再改，需对 diff 复审（重点 F1 句）。
2. 双有/内容量数值未独立重跑渲染（需 `render_frame.rs` 全链与 oracle 场景）；本次只做算术复算+提交体/代码注释溯源，与任务要求的"复算+推导链"一致。
3. `docs-links` 复核以终态工作区为准（PASS）；中途 FAIL 已记录并归因 `.tmp_*`，若提交时仍存在会再次 FAIL（pre-commit 会拦，属预期保护）。
4. 未审 K2/其它 spec 内容（不在 task-14 范围）；未验证 `Master plan §6` 的历史流变（仅验证其现行裁定注存在且与两 spec 一致）。

---

【2026-10-18】decomp-verifier，复核 HEAD `9bb0fd9`；被审对象=工作区未提交的两份 spec 改动（+66/−2）。

---

## 复审（2026-10-18）——F1 定点复读：**VERDICT = pass**

> 复审范围：只核 spec-writer 的 F1 修复句 + 三项可选非阻塞项（①decision 直链 ②4099 阈值说明 ③E1 双有出处 TODO）+ 门禁 docs-links；不做全量重审（§1-§7 结论继续有效）。
> 复审时点：HEAD `cce3bb1`，两 spec 仍为工作区未提交改动（`git diff --stat` = 2 files, +98/−5；行数 signing 145 / visual-plan 426——比 F1 初稿多出的行来自同期并发插入，见下）。

### 1. F1 复读（通过）
- 实际落点：`docs/spec/l4-signing-criteria.md` **L78**（非交接所述 L72——同期并发插入使行号后移；语句内容与要求一致）：
  `→ 864/7680=11.25%  864/8046=10.74%（前者=占阈值比，后者=形合率；l4-visual-plan.md 原表"11%"系 **864/7680=11.25% 的舍入**——原标签"形合率"误用分母，更正后引用须带分母名（占阈值比≠形合率））。`
- 与 `l4-visual-plan.md` L380 / L387-392 的 K7 更正**完全一致**：原表公式 `864/7680` ⇒ 占阈值比；`864/7680=11.2500%`、`864/8046=10.7383%≈10.74%` 复算一致；两文件已无「原表 11% 系形合率舍入值」旧表述。**F1 闭环。**

### 2. 三项可选非阻塞项（全部落实，无新错误）
- **① decision 直链**：`l4-signing-criteria.md` L19-20 新增 `[decision-l4-zero-tolerance-ruling](../knowledge/decision-l4-zero-tolerance-ruling.md)`（commit `24093be`）——目标文件存在、docs-links 覆盖通过 ✓。
- **② 4099 阈值说明**：L52 新增「**4099=当时的形合判定阈值、非内容量**——§4 item1 原文"超 4099 阈"，与 7680 同属阈值类」——与 §4 item1 L104 原文「超 4099 阈」及历史 `b4586e7` 口径一致，消解了「用阈值当分母」这一撤销理由未点明的问题 ✓。
- **③ E1 双有出处 TODO**：L67-69 新增「`oracle/screens/visual-report.md` 只导出 diff 像素、未导出"双有"计数；后续需固化"双有"的可复算导出命令（本轮不阻塞复核，登记为待办，**不以自述冒充复算**）」——如实登记证据缺口、无过度声明 ✓。

### 3. 门禁实跑
```text
python gates/cli.py check
→ [PASS] docs-links：314 个 md / 307 条本地链接全部有效
→ SUMMARY: PASS=11 FAIL=0 SKIP=1 TOLERATED=0 ; CHECK: PASS ; EXIT=0
→ [PASS] rust-tests：cargo test --workspace 通过；测试数 355（基线 339）
```
说明：`docs-links` **PASS（314 md / 307 links）**——交接自报 303 links，实跑 307 的差异来自同期其它 docs 新增，PASS 结论不受影响。此前的 `.tmp_*` 残留与 `rust-tests` 并发失败均已消失（全量 check 现全绿）。

### 4. 并发改动提示（非阻塞，供 Lead 确认归属）
两份 spec 还含**来自 L4 攻坚任务的并行改动**（非 spec-writer 本次 F1 范围）：`l4-signing-criteria.md` L21-31「撤销 W-C1/W-C4 白名单候选」块（引用 `l4-zero-tolerance-attack-plan`/`adversarial-review-l4-attack-plan`）；`l4-visual-plan.md` §2 W-C1/W-C4 撤销注与交叉引用。快检：`11,156/280,132=3.9824%`（3.98%）与攻击计划 §0/§3.2 一致；P 表 `render_frame.rs:19-28` 存在（64 项）；被引 `adversarial-review-l4-attack-plan.md` 存在；docs-links 覆盖通过——**未见错误**。该部分不在本次 F1 复读范围，若随同批提交，建议 Lead 确认其已有独立复核覆盖。

### 5. 复审结论
F1 已闭环，三项可选全部落实且无新错误，门禁 docs-links 真 PASS ⇒ **VERDICT = pass**。两份 spec + 本复核报告可提交。提交前建议：`git status` 确认无 `.tmp_*`；两 spec 同一 commit（或拆分并各自带 Evidence）；本报告 §0 的 needs_revision 由本节覆盖为 pass。
