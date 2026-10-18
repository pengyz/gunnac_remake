# 轮次日志 round-025 —— 🔧 基建修复 + 分层落地（平台抽象层 → PC shell → 可玩闭环）+ L4 判据回到 0 容差

- 日期：2026-10-18 ｜ Goal：goal-a8a36131（rev 2，沿用 round-024 记录）｜ 覆盖：2026-10-18 会话阶段（`04559b1` 12:17 → HEAD `b92139a` 14:14，51 提交）
- 补记背景：round-024 落档 commit `fc83cd3`（2026-10-14 15:36）之后至本卡 HEAD 共 **615 提交**无轮次日志（仓库累计 859 提交；2026-10-14 以来 626；2026-10-18 当日 125 = 会话前 74 + 会话段 51）。本卡按用户/Lead 指令补记本会话阶段的判据、落地与账本变化；**不重写历史**；round-024 的判据记录经本卡核验未见落地文本、且其"98% 判据"不作现行口径（见 ⑤ 第 0 条，dated 注记）。
- 纪律：AGENTS.md（证据 A/B/C、未证不发明、数字可独立复算、小步提交、结论被推翻原地标注）。本卡 docs-only，未改代码/门禁/基线/`data/`。

## ⓪ 本会话主线（一句话）

**先修基建（Windows 下门禁/钩子/文本写入不可用）→ 立平台抽象层 v1（trait 缝 + game-shell，复核 pass）→ PC shell v1（六 trait 宿主 + 确定性自检，复核 pass）→ 可玩闭环 A（STG-01 replay + FrameView 暴露，复核 pass）→ PC shell v2/可玩闭环 B（FrameView 渲染，复核 pass）+ B1 菜单跨键修复（复核 pass）→ L4 判据反转（"100% 签署"仅过程里程碑 → 判定=未达成，回到 0 容差）并出 P0 攻坚方案/P0-0 手册。**

## ① 基建硬阻塞修复（LF 链 / 钩子 / 门禁解码 / 生成物 / 误卷留痕）

| # | 问题 | 修复 commit | 结果/证据 |
|---|---|---|---|
| 1 | Windows 检出 CRLF 致 sha256 门禁误报（rom/seed/baseline 链） | `04559b1`（`.gitattributes eol=lf` 全仓库 LF 归一）、`5bb56c1`（gates 文本写入 `newline="\n"`）、`7fd50b1`/`e7b8285`（tools 轨道/状态产物显式 LF，两处缺 `encoding` 的 GBK 隐患） | 检出侧+写入侧双修；gotcha 入知识库 `062440a`（[gotcha-windows-crlf-sha-gate](../knowledge/gotcha-windows-crlf-sha-gate.md)） |
| 2 | Windows 占位别名 `python3` 致 git hooks 不可用 | `9271cf0`（hooks 解释器探测 `python3→python→py`） | commit-msg/pre-commit 钩子恢复可执行 |
| 3 | 门禁子进程用 locale(cp936) 解码 cargo/git 输出 → reader 线程异常、check 崩溃 | `a11161d`（子进程统一 `encoding="utf-8", errors="replace"`） | `gates check` 在中文 Windows 可稳定出 SUMMARY（此前复核已在 [adversarial-review-completion-audit](../findings/adversarial-review-completion-audit.md) §3 记录该崩溃与非假绿边界） |
| 4 | `visual-report` 写出字面 `\n`（单行不可读） | `ab6383c` | report 恢复可读 Markdown；生成物重生成字节一致 |
| 5 | 并行 git 暂存区误卷（`ddb1558` 带 task-9 docs、`96415c1` 带 task-9 代码） | 留痕 `3e1715f`（[gotcha-git-add-commit-atomicity](../knowledge/gotcha-git-add-commit-atomicity.md)）；归属注见 `ba911b4`（plan-shellwin-v1 §9.1） | 历史不 rewrite；此后提交须 `git diff --cached --name-only` 核对 |

> 知识库变化（当日基线链）：`knowledge_entries` **19→21**（`76ba3ad` 补登既有 2 条）→ **22**（`062440a` 新增 gotcha-windows-crlf-sha-gate）→ **23**（`24093be` 新增 decision-l4-zero-tolerance-ruling）。另 `3e1715f` 复用既有 gotcha 条目留痕（不计新增）。

## ② 平台抽象层 v1（trait 缝 + 平台无关宿主 game-shell，复核 pass）

- **方案**：`0809317`（[plan-platform-abstraction](../findings/plan-platform-abstraction.md)，trait 缝补全 + game-shell 设计 + 未证域声明）。
- **实现**：`c43f08d`（trait 补全：AssetSource / Lifecycle·AppEvent / Renderer size·resize）、`98ad36c`（新增 crate `game-shell`：60Hz 固定步/输入消解/宿主状态机/tick 序列/合成/音频，零平台依赖）、`6362895`（[platform-shell](../spec/platform-shell.md) v2：分层图/PC-Android 矩阵/Android 落地清单）。
- **复核**：`9bb0fd9` 首轮 **needs_revision**（见 [adversarial-review-platform-v1](../findings/adversarial-review-platform-v1.md) §0「Verdict」）→ `b5e68b6` 修 E1/S8/S1/S3 阻塞项 → `96415c1` 复审 **pass**（同报告「复审 VERDICT：pass」节）→ 残留 R1-R5 数字/scope 收口 `cc011a8`。
- **账本**：`ff5aa7c` rust_test_count 280→326；`fe8ce68` 338→339（复核修正新增 mock 断言）。
- **门禁实证**：复审报告记 `PASS=11 FAIL=0 SKIP=1 TOLERATED=0`、`docs-links` PASS（`b5e68b6` 单 commit 状态下的边界亦已注明）。

## ③ PC shell v1 + 可玩闭环 A（复核 pass）

### PC shell v1（shell-win 六 trait 宿主）

- 方案 `1be66e5`（[plan-shellwin-v1](../findings/plan-shellwin-v1.md)）；实现 `7b5e4da`（Windows：winit 0.30 + softbuffer，六 trait 实现 + game-shell 接线 + `--selftest-frames` 确定性自检）；回填 `a93704b`（platform-shell §4.1/§4.2）。
- 复核 `5a4485a` = **pass**（[adversarial-review-shellwin-v1](../findings/adversarial-review-shellwin-v1.md) §0：7 条非阻塞 findings，B1/S1 转后续卡）。
- findings 收口 `ddb1558`（B1 drive-relative 拒绝 / S1 exe 资产根 + E1-E5/S2 文档口径）；其复核 pass 见 `167d8a9`（[adversarial-review-shell-fixes](../findings/adversarial-review-shell-fixes.md) §0：0 阻塞，测试 24 = lib 5 + smoke 19）。
- docs 收口 `ba911b4`/`ba07d31`（F1/F2/F4/F5 + 独立复核 pass，见 [adversarial-review-docs-f1f5](../findings/adversarial-review-docs-f1f5.md) §0）。
- 账本：`16fb5b2` 326→338。

### 可玩闭环 A（平台无关侧：菜单派发 + STG-01 数据装载 + FrameView 暴露）

- 方案 `cce3bb1`（[plan-playable-loop](../findings/plan-playable-loop.md)）；ROM 只读 PRG bank 窗口接口 `05e3135`（`game_data::rom::prg_bank`）；spec 回填 `b94ede8`；自检读数回填 `bde85bf`。
- 复核 `c8ada9b` = **pass**（[adversarial-review-playable-a](../findings/adversarial-review-playable-a.md) §0；STG-01 replay 对拍与确定性签名独立复现；门禁 `PASS=11 FAIL=0 SKIP=1`、ratchet 355/355）。
- 账本：`c83a19a` 339→355。

## ④ PC shell v2 + 可玩闭环 B + B1 菜单跨键修复（均复核 pass）

- **shell v2/可玩闭环 B**：方案 `a630666`（[plan-shellwin-v2](../findings/plan-shellwin-v2.md)）；实现 `4a0672b`（FrameView 渲染：模式占位画面/HUD 数值域占位/敌槽 Some 路径）；状态回填 `703130a`/`d619cf2`（platform-shell §4/§8，含 `view_signature` 新值）；账本 `9405a35` 357→366。
  复核 `b92139a` = **pass**（[adversarial-review-playable-b](../findings/adversarial-review-playable-b.md) §0；`cargo test -p shell-win` lib 12 + smoke 21 = 33、workspace 366 = 365 passed + 1 ignored、`CHECK: PASS`、rust-tests 366/366）。
- **B1（可玩闭环 A 复核 finding）菜单跨键吞按**：`1700934`（`MenuInputGate` 触发键域分离——修「跨键组合吞按」）；方案 [plan-menuinput-fix](../findings/plan-menuinput-fix.md)；记录值更新 `38d2e89`（`plan-playable-loop` §10 mock-300 非回归说明）；账本 `cc4c692` 355→357。
  复核 `06a0dc4` = **pass**（[adversarial-review-menuinput-fix](../findings/adversarial-review-menuinput-fix.md) §0；2 条非阻塞 info，0 阻塞）。
- **未证边界**（各方案/复核已显式登记）：敌精灵/子弹/HUD 文本/结局卡图样、HUD 可见性/模式门、命中判定盒 H-16 残项、音频 PCM 静音占位、Boss/EN-9+ —— 一律 TODO/fail-closed，未发明。

## ⑤ L4 判据回到 0 容差（当日反转链 + P0 攻坚）

**判据链（must-read；含对 round-024 判据记录的核验与更正）**：

0. round-024 记录 2026-10-14 用户授权判据="98% 级行为一致 + 分类账完备"，并称"入档于 master-plan.md Stage A 判据节"。**本卡核验**：现行 [master-plan](../master-plan.md) §6 原文仍为"L4 白名单外像素一致"；**除 round-024/本日志外，全仓 md 中 `98% 级` 零命中**（无任何落地文档），且 `git log -S '98% 级行为一致' -- docs/master-plan.md` 无提交 ⇒ **该判据未见落地文本证据**（仅见于 round-024 轮次记录）；**现行唯一有效判据以 master-plan §6 原文 + 2026-10-18 用户裁定（`24093be`）为准**——round-024 的"98% 判据"不作为现行口径。
1. 2026-10-18 早段（本会话前）按**判定式阈值**推进至 diff 4552<7680 并出"100% 签署/战役收官"（`33348f2`→`22d896c`→`2fe5249`/`548086d`/`42fe298`；数字链复审见 [l4-signing-criteria](../spec/l4-signing-criteria.md) §1）；
2. **同日用户新裁定**（`24093be`，[decision-l4-zero-tolerance-ruling](../knowledge/decision-l4-zero-tolerance-ruling.md)）：Stage A 的 L4 判据**回到 [master-plan §6](../master-plan.md) / [l4-visual-plan §1](../spec/l4-visual-plan.md) 的 0 容差（白名单外像素 100% 一致）**；上述签署仅"过程里程碑"，**不构成 L4 达成** ⇒ **L4 判定=未达成**（现行 22 对仅 1 对逐像素相等=4.55%，合计差 280,132/1,351,680=20.72%，同点 79.28%；见 [rewrite-completion-audit](../findings/rewrite-completion-audit.md) §4）；
3. 落档与复核：`7625df8`（K1 撤销 43.0%/K7 分母语义 + **撤销 W-C1/W-C4 白名单候选**——11,156 双非黑异色像素两侧颜色 100% 在 64 色 P 表内，非平台差异；复核 [adversarial-review-spec-k1k7](../findings/adversarial-review-spec-k1k7.md) 首轮 needs_revision→F1 定点复读 **pass**、[adversarial-review-wc1-revocation](../findings/adversarial-review-wc1-revocation.md) **pass**）；
4. **0 容差验收判据落地**：`64b3a44`（`tools/diff_visual.py --strict`，默认分类账行为不变）+ `53aceb8`（[l4-visual-plan](../spec/l4-visual-plan.md) 判据行「L4 0 容差 = `--strict` exit 0」；现基线 `STRICT: FAIL diff=280132`，见 [plan-diff-visual-strict](../findings/plan-diff-visual-strict.md) §3）；
5. **P0 攻坚方案**：`0cf42f0`（[l4-zero-tolerance-attack-plan](../findings/l4-zero-tolerance-attack-plan.md) 957 行，首轮 [adversarial-review-l4-attack-plan](../findings/adversarial-review-l4-attack-plan.md) needs_revision→第二轮 **pass**；差异按像素性质三分 262,352/6,624/11,156，执行序 P0-1..P0-5）；
6. **P0-0 基线冻结手册（在飞，未入库）**：`docs/findings/l4-baseline-freeze-plan.md`（task-16，二轮修正后 1406 行）经 task-30 复核 needs_revision（F1/F2/F3 阻塞）、task-33 复审 needs_revision（B1-B5）→ 修正后 task-35 定点复核**进行中**；执行环境需用户提供 Linux+Mesen2（当前 Windows 只能只读预研，零验收数字）。

## ⑥ 账本 / 基线 / 门禁（本会话变化）

**基线（`data/status/baselines/baseline.json` 现値）**：rom_files_verified 1｜seed_projection_files 1｜naming_ledger_entries 206｜assets_manifest_entries 0｜knowledge_entries **23**｜rust_test_count **366**｜completeness_cells 19。

**rust_test_count 上调链**（全部为"进度类上调"，Lead/门禁属主授权，逐条见 `baseline_history.jsonl`）：
`279→280`（`76ba3ad`）→`280→326`（`ff5aa7c`，平台层）→`326→338`（`16fb5b2`，PC shell v1）→`338→339`（`fe8ce68`，平台层复核修正）→`339→355`（`c83a19a`，可玩闭环 A）→`355→357`（`cc4c692`，B1 修复）→`357→366`（`9405a35`，shell v2）。会话段净增 **+87**（279→366；以 280 计 +86）。

**knowledge_entries**：19→21→22→23（①② 两新条目 + 两补登，见 ①）。

**门禁状态**：本项目门禁表 12 项中，**11 PASS + 1 显式 SKIP**（`addr-bridge`：JP ROM 未入库，资产政策预期）为现行常态——`adversarial-review-shell-fixes` §4、`adversarial-review-playable-a` §5、`adversarial-review-playable-b` §8、`adversarial-review-menuinput-fix` 均实测 `CHECK: PASS（PASS=11 FAIL=0 SKIP=1 TOLERATED=0）`；`docs-links` 历次 PASS。**注意**：本卡期间工作区有 task-36 未提交改动（shell-win/plan-shellwin-v2/platform-shell），故未重跑全量 `check`，上列数字绑定各复核报告时点（HEAD `0cf42f0`/`b92139a` 附近）。

## ⑦ 在飞 / 遗留（本卡时点 HEAD `b92139a` + 工作区）

| 项 | 状态 | 指针 |
|---|---|---|
| P0-0 手册定点复核（task-35） | 在飞（手册 1406 行未入库） | `docs/findings/l4-baseline-freeze-plan.md`、`adversarial-review-l4-baseline-freeze-plan.md`（untracked） |
| L4 P0 采集中枢（P0-1..P0-5） | 阻塞于 Linux+Mesen2 环境 + 用户裁定样本集 | l4-zero-tolerance-attack-plan §6.2/§6.4 |
| shell v2 findings 收口（task-36） | 在飞（工作区改动：`crates/shell-win/**`、`plan-shellwin-v2.md`、`platform-shell.md`） | 复核报告 playable-b §5 E3-E6 |
| 弹丸/命中接线（task-37）、H-16/坐标/Boss 取证（task-38）、L5 八关缺口（task-39） | 在飞 | 各卡 findings |
| Stage B（现代化） | 未开始（Stage A 判据未齐：L4 未达成、L5 未全绿） | master-plan §6 |

## 附：本会话代表性提交清单（按主题；哈希均可 `git show` 核验）

| 主题 | commits（时:分） |
|---|---|
| 基建修复 | `04559b1` 12:17 / `5bb56c1` 12:19 / `76ba3ad` 12:20 / `062440a` 12:20 / `9271cf0` 12:23 / `ab6383c` 12:23 / `7fd50b1` 12:25 / `e7b8285` 12:27 / `a11161d` 12:52 / `3e1715f` 13:15 |
| 平台抽象层 v1 | `0809317` 12:37 / `c43f08d` 12:38 / `98ad36c` 12:42 / `6362895` 12:43 / `ff5aa7c` 12:44 / `9bb0fd9` 13:00 / `b5e68b6` 12:59 / `96415c1` 13:14 / `cc011a8` 13:27 |
| L4 裁定/口径 | `24093be` 12:43 / `64b3a44` 13:16 / `7625df8` 13:20 / `53aceb8` 13:22 / `0cf42f0` 13:27 |
| PC shell v1 | `1be66e5` 12:49 / `7b5e4da` 12:54 / `a93704b` 12:54 / `16fb5b2` 12:55 / `5a4485a` 13:25 / `ddb1558` 13:12 / `167d8a9` 13:33 / `ba911b4` 13:40 / `ba07d31` 13:41 |
| 可玩闭环 A | `cce3bb1` 13:02 / `05e3135` 13:13 / `b94ede8` 13:18 / `bde85bf` 13:21 / `c83a19a` 13:21 / `c8ada9b` 13:39 |
| shell v2 / B1 | `a630666` 13:42 / `4a0672b` 13:48 / `703130a` 13:51 / `9405a35` 13:56 / `1700934` 13:58 / `38d2e89` 14:01 / `d619cf2` 14:07 / `06a0dc4` 14:13 / `b92139a` 14:14 |
| docs/审计 | `36163e1` 13:00（完成度审计+复核）、`7625df8`（口径）、`ba911b4`/`ba07d31`（docs 收口） |
