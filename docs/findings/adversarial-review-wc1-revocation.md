# 对抗复核：W-C1/W-C4 白名单候选撤销——adversarial-review-wc1-revocation

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不得复用被审方结论**）
> 被审对象（task-20 产出，**工作区未提交，docs-only**）：`docs/spec/l4-visual-plan.md`（§1/§2/治理 + 撤销注）、`docs/spec/l4-signing-criteria.md`（顶部裁定注追加）、`docs/findings/rewrite-completion-audit.md`（§7 追加 dated 引用同步注，+9 行）。复核时 HEAD `ddb1558`。
> 纪律：AGENTS.md §3/§5（证据 A/B/C、数字可独立复算、未证不发明、历史留档不删）。本文只写本文件；未改被审文档、未 git add/commit。

---

## 0. VERDICT：**pass（撤销证据成立、记录合规；4 条非阻塞 findings + 1 条提交前置条件）**

- **被审核心声明独立复现全部成立（A 级）**：22 对差异按像素性质分解 = missing 262,352 + spurious 6,624 + **color 11,156**（合计 280,132）；11,156 个 color 像素的 expected/actual 两侧颜色 **100% 落在 render_frame.rs:19-28 的 P 表内**（我解析源码 P 表 = **64 项 / 54 唯一色**，比方案 C7 的 NES∪GOLD 集合更强的检验）；`11,156/280,132=3.9824%≈3.98%`；Top 对 `白→暗青 3,239 / 暗青→白 2,269 / 粉→白 1,680 / 鲑→白 775 / 红→白 600 / 淡紫→白 510 / 浅蓝→白 427` 逐条一致；白↔暗青 5,508=49.37%≈49.4%、Top5 彩色→白 3,992=35.78%≈35.8%。
- **记录合规**：撤销为**原地标注**（历史候选主张以删除线+原备注留档，未删除）；有日期（2026-10-18）与证据指针（attack plan §0/§3.2 + C7、独立复现 §1.6）；候选语义与机制名/历史流程标签区分正确（机制名引用如 `l4-render-support.md` R-6/W-C1 接线面**未被改动**）。
- **范围合规**：只改 docs（`tools/`/`oracle/`/`gates/`/`data/`/`rom/`/`reference/` 工作区零改动；`tools/diff_visual.py` 未偷偷改）；生成物 `oracle/screens/*.md` 未改（正确，按 task-20 约定生成器文案同步属另卡）。
- **audit 追加**：`+9 行、0 删除`，只增不删，内容与 P0/P2 行原文对得上。
- **docs-links**：工作区 **PASS（314 md / 312 links）**；但存在**提交前置条件**——signing-criteria L24-25 的 markdown 链接指向**未跟踪**的 `l4-zero-tolerance-attack-plan.md` / `adversarial-review-l4-attack-plan.md`，提交时必须一并纳入，否则干净检出上 docs-links 会 FAIL（见 §5）。

---

## 1. 撤销证据独立重算（A 级）

> 方法：自行解析 `crates/game-desktop/src/bin/render_frame.rs` L19-28 的 `const P: [[u8;3];64]` 为 RGB 集合；用 `tools/diff_visual.py::decode_png` 逐像素解码 22 对 expected/actual（**只复用解码器，不复用被审 C7 脚本结论**），自行分类/计数/发对。

| 量 | 被审声明 | 我的独立重算 | 结论 |
|---|---|---|---|
| P 表规模 | 64 项 / 54 唯一色（复核 #8 声明） | **64 项 / 54 唯一色** | ✓ |
| 差异三分 | missing 262,352 / spurious 6,624 / color 11,156 | **262,352 / 6,624 / 11,156（合计 280,132）** | ✓ |
| color 像素颜色域 | expected/actual 两侧 100% 在 P 表内 | **expected 11,156/11,156、actual 11,156/11,156（100%）** | ✓（我的检验用源码 P 表，非 NES∪GOLD 并集，更强） |
| color 占比 | 11,156/280,132=3.98% | **3.9824%** | ✓ |
| Top 对 | 3,239 / 2,269 / 1,680 / 775 / 600 / 510 / 427 | **逐条一致**（多出的次对：白→粉 190、黄→橄榄 190、鲑→粉 180…） | ✓ |
| 白↔暗青 | 5,508=49.4% | **5,508 / 49.37%** | ✓ |
| Top5 彩色→白 | 3,992=35.8% | **3,992 / 35.78%** | ✓（注：全部 color→白 = 6,706=60.1%，3,992 是所列 5 对之和，注中已用"=`粉→白1,680、鲑…、红…、淡紫…、浅蓝…`"枚举式表达，可复算） |
| boot-black 0 容差 | title-menu boot-black 帧 0 | `visual-report` 该帧 diff=0（唯一 0 差异帧） | ✓ |

**证据分级裁定（精确）**：
- 「**非 RGB 映射 / 非色彩空间 / 非平台差异**」= **A 级**（若两侧 RGB 值全在同一 NES 64 色 P 表内，且 boot-black 帧 0 容差，则差异不可能是 RGB 映射/色彩空间平台差异——已被像素域穷举证明）；
- 「**残差=选哪个调色板索引（ink/attr/$3F00）**」= **B+ 归因**（排除法必然指向索引选择；机制链指针 A 级：`l4-render-support.md` R-6 上传通道 `bank15:$F04B-$F072`、位域改写器 `bank00:85E8-8618`、三表 `$E9D2/$E9D6/$E9DA`、数据源 `$0150+…`），但**尚无逐索引 trace 直证**"某写点选了错索引"。
  被审注把整段标为"依据（A 级，用户裁定）"——**排除性结论 A 级成立**；"索引绑定"归因建议标 B+（见 F3，低）。
- ⇒ **撤销 W-C1/W-C4 作为"平台差异白名单候选"的判据成立**（白名单只接平台差异；本差异落在同一色域内的索引选择，属实现/行为面，不是平台面）✓。

---

## 2. 撤销记录合规复核

| 要求 | 复核结果 |
|---|---|
| 原地标注、历史主张不删 | ✓ `l4-visual-plan` §2 W-C1/W-C4 行：`~~强候选~~ → 【2026-10-18 用户裁定·撤销】`，原备注原文留档（L34/L37）；§1 色彩空间行保留原句+追加撤销括注（L23）；治理段新增一句"不在启用流程内"（L63）；§互链注残余缺口处**原地补充**"〔W-C1 白名单候选 2026-10-18 撤销——非平台差异，见 §2 撤销注〕"（L415），未删原主张。 |
| 裁定日期 | ✓ 各处均带 2026-10-18。 |
| 用户裁定指针 | **部分**：证据指针齐全（attack plan §0/§3.2 + C7、独立复现 §1.6），但**未直接引用裁定记录本身**——仓库内现行裁定记录是 `l4-zero-tolerance-attack-plan.md` **§6.2**（"用户 2026-10-18 本轮裁定②：撤销 W-C1/W-C4 白名单候选"）；`24093be`/`decision-l4-zero-tolerance-ruling.md` 只记录 0 容差与"W-C1 转 P0 攻坚"，**不含候选撤销**（其 L27/L33 仍把 W-C1 记为 P0 缺口，属机制面表述，非候选）。→ **F1（低，非阻塞）**：建议 visual-plan L42 / signing L21 各补一句指向 §6.2（或后续补 decision 条目），使"用户裁定"可追到记录而非只追到分析文档。 |
| 候选 vs 机制名/历史流程标签区分 | ✓ 抽查：机制名（`l4-render-support.md` R-6/W-C1 调色板接线面、`$3F00` 上传链、precise-source-search 等的 W-C1 机制标签）**全部未改**（文件 git-clean）；历史流程/复核文档（adversarial-review-26/99/103、rounds 等）未改；候选语境（spec 表/裁定注/audit 注/生成物）已更新或按另卡保留。独立统计：全仓 docs+oracle 共 **109 处 / 22 文件**出现 W-C1/W-C4；候选语境约 17 行——抽查分类合理。**F2（低，非阻塞）**：spec-writer 的"45 条命中分类"未见仓库落档（复核方只能看到结果面），建议把分类表落一条 findings 注以便后续可审；同时撤销注可加半句"**仅撤销白名单候选项；W-C1 机制名/历史标签沿用**"，防读者误以为机制面作废。 |

---

## 3. 是否越权 / 范围（docs-only）

- `git status --porcelain -- tools oracle gates data rom reference` = **空** ⇒ 未改生成器/生成物/门禁/数据/ROM/参考轨 ✓。
- `tools/diff_visual.py` **未改**（task-20 约定其文案同步属另卡）✓；`oracle/screens/visual-report.md`、`oracle/screens/whitelist-application.md` **未改**（生成物）✓。
- 被审改动文件 = `docs/spec/l4-visual-plan.md`、`docs/spec/l4-signing-criteria.md`、`docs/findings/rewrite-completion-audit.md`（其余工作区改动属并行 task-9/18 等，非本卡）✓。
- **已知残留（非本卡错误，F4 低）**：生成物 `oracle/screens/whitelist-application.md` 仍写"W-C1/W-C4 强候选…申请中（未启用）"，`oracle/screens/visual-report.md` 仍含"W-C1 调色板映射候选"分类文案；`tools/diff_visual.py` 下次 visual 门禁运行会**重新生成同样旧文案**。建议 tools 同步卡尽快跟进，并在提交说明中注明"生成物未同步"。

---

## 4. audit 追加注（+9 行）复核

- `git diff docs/findings/rewrite-completion-audit.md` = **纯 +9 行、0 删除**，插入于 §7 表后；未改任何历史主张/复核结论 ✓。
- 内容核对：所指 P0 行原文「范围 = W-C1 调色板映射 + nametable 持久化模型」与 P2 行末句「决定"共用映射表 vs 白名单"并留用户确认记录」均**逐字存在**（audit L369/L375）✓；"白名单决策项关闭（不启用、`gates/visual_whitelist.json` 仍无条目）"与 `Test-Path gates/visual_whitelist.json` = False 一致 ✓；证据指针指向 attack plan §0/§3.2/§7 与 review §1.6，均在 ✓。
- **F5（低，措辞）**：注中"上表 P0/P2 行**已被**同日用户裁定更新"易被读成表格行被改（实际未改，仅追加本注）；建议改为"上表 P0/P2 行的**签署面语义应按本注更新**（原文留档不删）"。

---

## 5. docs-links 与提交前置条件（重要）

- 工作区实测：`python gates/cli.py check` → `docs-links` **PASS（314 个 md / 312 条本地链接全部有效）**；`SUMMARY: PASS=11 FAIL=0 SKIP=1 TOLERATED=0`（复核时 cargo 339/基线 339 亦 PASS）。
- **提交前置条件（必须）**：`l4-signing-criteria.md` **L24-25** 的 markdown 链接指向两个**当前未跟踪**的文件：
  - `docs/findings/l4-zero-tolerance-attack-plan.md`（**UNTRACKED**）
  - `docs/findings/adversarial-review-l4-attack-plan.md`（**UNTRACKED**）
  若本次提交只带"两份 spec + audit + 复核报告"而不带这两个文件，**干净检出上 docs-links 会 FAIL**（两个链接目标不存在）；且 `l4-zero-tolerance-attack-plan.md` §6.2 正是"用户撤销 W-C1/W-C4"的唯一库内裁定记录（F1 所指）。⇒ **提交必须同时纳入这两份 findings（及其互链）**，或先把 L24-25 改为非链接的纯指针。建议 Lead 提交前在干净工作树跑一次 `python gates/cli.py check` 复核。

---

## 6. Findings 四分类与修正建议

### 6.1 证据不足（低，非阻塞）
- **F1** 撤销注未引用"用户裁定记录"本身：`l4-visual-plan.md` L42 / `l4-signing-criteria.md` L21 各补 `（裁定记录：l4-zero-tolerance-attack-plan.md §6.2 用户裁定②；24093be/decision 不覆盖候选撤销）` 一句。
- **F2** "45 条命中分类"未落档：建议把分类表（候选语境/机制名/历史标签）作为一段 findings 落库；并在撤销注补"仅撤销候选；机制名/历史标签沿用"半句。

### 6.2 口径问题（低，非阻塞）
- **F3** 证据分级措辞：`l4-visual-plan.md` L42「依据（A 级…）」——建议拆分"排除 RGB/色彩空间=**A**；索引绑定归因=**B+**（机制指针 A 级）"，与项目证据分级一致。

### 6.3 表述/残留（低，非阻塞）
- **F4** 生成物旧文案（`oracle/screens/whitelist-application.md` 仍"强候选/申请中"、`visual-report.md` 仍"W-C1 候选"）——按 task-20 属另卡；建议 tools 同步卡排期，提交说明注明"生成物暂未同步"。
- **F5** audit 追加注「P0/P2 行已被更新」措辞改为「签署面语义按本注更新（原文留档）」。

### 6.4 越权/伪造
- **无**。未发现伪造证据或超范围改动；撤销方向（收紧白名单、转机制攻坚）与 24093be 0 容差裁定一致，未借撤销夹带行为/门禁改动。

### 6.5 提交前置条件（非 findings，Lead 必做）
- 见 §5：两处 markdown 链接指向 UNTRACKED attack-plan 文件 ⇒ 提交批次**必须包含** `l4-zero-tolerance-attack-plan.md` + `adversarial-review-l4-attack-plan.md`（否则 docs-links FAIL），并建议提交前干净树跑 check。

---

## 7. 命令与输出摘要

```text
# 1) 独立重算（自写 l4_c7_indep.py；P 表从源码解析，不复用被审脚本结论）
P table entries: 64 unique: 54
class: {'color': 11156, 'spurious': 6624, 'missing': 262352} total 280132
color total 11156 in P-table: expected 11156 actual 11156
11156/280132=3.9824%
Top: (255,254,255)->(0,64,77) 3239 ; (0,64,77)->(255,254,255) 2269 ;
     (254,204,197)->(255,254,255) 1680 ; (254,129,112)->... 775 ;
     (181,49,32)->... 600 ; (254,196,234)->... 510 ; (100,176,255)->... 427
white<->darkcyan 5508 49.37% ; top5 color->white 3992 35.78%

# 2) audit 追加性
git diff docs/findings/rewrite-completion-audit.md → 纯 +9 行（0 删除），插入 §7 表后

# 3) 范围
git status --porcelain -- tools oracle gates data rom reference → 空
Test-Path gates/visual_whitelist.json → False
git status --porcelain -- tools/diff_visual.py → 空

# 4) 未跟踪引用（提交前置）
git ls-files --error-unmatch docs/findings/l4-zero-tolerance-attack-plan.md → UNTRACKED
git ls-files --error-unmatch docs/findings/adversarial-review-l4-attack-plan.md → UNTRACKED
l4-signing-criteria.md L24-25 以 markdown 链接引用上述两文件

# 5) 门禁
python gates/cli.py check → [PASS] docs-links：314 个 md / 312 条本地链接全部有效
                            → SUMMARY: PASS=11 FAIL=0 SKIP=1 TOLERATED=0 ; CHECK: PASS ; EXIT=0
```

---

## 8. 复核边界

1. 复核对象=工作区未提交 docs 改动（HEAD `ddb1558`）；若提交前再改需复读相关段（重点 §5 前置条件与 F1 指针）。
2. 未重跑 Mesen2/oracle 采集（Windows 无 Mesen2，攻击计划 §6.2 已裁定 Linux 采集环境另卡）；本次像素统计全部基于库内 `oracle/screens/{expected,actual}` 现有 22 对 PNG，可跨平台复算。
3. 未审 W-C1/W-C4 之外的其它 L4 攻击计划内容（task-8 范围）；未验证 `l4-zero-tolerance-attack-plan.md` 自身的全部 C1-C10（只核 C7 相关与 §6.2 裁定记录）。
4. "45 条命中分类"未能定位库内记录（F2），本复核以结果面抽查替代。

---

【2026-10-18】decomp-verifier，复核 HEAD `ddb1558`；被审=工作区未提交 W-C1/W-C4 撤销 docs 改动。
