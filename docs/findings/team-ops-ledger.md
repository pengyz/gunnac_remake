# Agent 团队运维台账（append-only）

> 纪律定义见 [docs/agent-team-discipline.md](../agent-team-discipline.md)（用户裁定 2026-10-19）。
> 本台账只增不删；每条 dated。格式：`## 【YYYY-MM-DD HH:MM】<事件类型>`。

## 【2026-10-19 23:32】纪律落库 + 巡查定时器启用
- 判定人/分派人：用户（裁定）→ Leader 落库
- 内容：核心工作纪律写入 `docs/agent-team-discipline.md`（团队模式/完成判据=复核+测试+commit+push/
  Leader 唤醒规程 0-a-b-c-d/落库要求/当前目标三项目）；启用 **10 分钟定时器**唤醒 Leader 巡查。
- 当前目标对齐判定：方向=**L4 一致收敛（0 容差 diff→0）+ PC shell + 游戏可玩**——与在办
  P0-1 渲染差异收敛链一致 ✓（理由：上一轮已裁定"先做 L4 一致性验证"；管线体检 5 段可用，
  前置=phase-batch 重建渲染输入工作区）。
- 派单（随后落地）：P0-1 差异族攻坚 + 渲染工作区重建；详见下方派单记录。

## 【2026-10-19 23:32】派单记录（随纪律生效首轮）
- 分派人：Leader；时间：2026-10-19 23:32
- 任务：
  1. 渲染输入工作区重建（phase-batch，gameplay/stg03/stg08 差异大户先行）——l4-analyst；
  2. P0-1 差异族攻坚卡（按 l4-zero-tolerance-attack-plan.md 族序）——待工作区就绪后派实施；
  3. 完成判据不变：对抗审查 pass + 测试全绿 + commit + **push 远端**。
- 理由：目标① L4 收敛需要可迭代的渲染-比对环；工作区重建是唯一前置。


## 【2026-10-19 23:52】唤醒巡查（用户事件唤醒，规程步骤 0-a-b-c）
- 0 检查行任务：纪律文档提交并 push ✓（6fe7da7..b9c0dda）；此前 C3/PAIR_SPEC/S1 三批均已
  复核 pass 并 push（今日远端 0 领先）——无欠账。
- a 空闲判定：teammate 全部 inactive=可派（注意：昨夜 turn 基建故障，若唤醒无效按 429 同款
  处置=唤醒原任务不重派；子 agent 路由兜底）。
- b 方向判定：与目标（L4 一致收敛/PC shell/游戏可玩）一致 ✓——理由：用户上轮裁定"先做 L4
  一致性验证"；管线体检 5 段可用、唯一前置=渲染工作区重建。判定人：Lead（依据用户裁定）。
- c 派单（不冲突派满）：
  1) task-170 渲染工作区重建 → l4-analyst（gameplay/stg03/stg08 phase-batch + 2 对重渲 sha 自证）；
  2) P0-1 差异族攻坚待工作区就绪后派实施（族序见 l4-zero-tolerance-attack-plan.md）。
  理由：工作区是渲染-比对迭代环唯一前置；攻坚族与重建不冲突（代码 vs 工作区）。
- d 429：本轮未发生。


## 【2026-10-19 23:58】派单记录（overlay 批准 + 工作区重建补卡）
- 分派人：Lead；时间：2026-10-19 23:58
- 任务：
  1) gs5-anchor-render overlay（C4）已核（输入 43 行逐字节同）并提交 push（86e18d4）→
     l4-analyst 接续：采集 5000f → phase-batch → render f3450×2 → 0 容差实测 → §7.7 安装；
  2) task-170 渲染输入工作区重建补卡（口径确认：17 traces→phase text + rom 链接 +
     2 对重渲 sha 自证）→ l4-analyst 执行中。
- 理由：#14 pair Option B 需 render-ready trace（overlay）；工作区重建是 L4 迭代环前置；
  二者不冲突（采集面 vs 工作区）。
- 纪律：两项均走"复核 pass → commit + push"收口。


## 【2026-10-19 23:47】团队重组（用户裁定：使用当前模型 mimo）
- 判定人：用户（"必须重新组建团队，使用当前模型"）；执行：Lead。
- 事实核查：7/8 成员= mify/xiaomi/mimo-v2.6-pro；**l4-analyst 钉死 deepseek-flash**（创建期模型，
  interrupt 后不释放）。工具硬约束：同名重建=「name already used」、新名=「limit 8 reached」
  ⇒ 字面重建不可行（需 GUI/管理侧移除槽位后方可重编）。
- 等效重组执行：**l4-analyst 退役停用**（不再派单）；其任务全量转 mimo 队友——
  task-170 工作区重建 + task-165 overlay 采集链 → platform-porter（mimo）；证据面复核备份
  → decomp-auditor（mimo）。此后团队工作 100% 由 mimo 模型承担。
- 中断残留现场已交接：rom 链接完成、phase-batch 后台进程存活（由 platform-porter 收编）。
- 429：未发生。


## 【2026-10-20 00:05】#14 安装+工作区重建交付验收（执行方=退役成员，验收转 mimo 复核）
- 事实披露：task-165 安装 + task-170 重建最终由**已退役的 flash 成员 l4-analyst** 抢跑交付
  （中断后其 turn 续跑完成）。按纪律**成果验收以证据+独立复核为准**，与执行方模型无关；
  此后不再向其派单（团队 100% mimo 执行不变）。
- 交付要点：#14 expected 安装（788d6984→44e7800d，旧件归档）、pair 重指向 gs5-anchor-render:3450、
  manifest/mapping 重生成（--validate 声称 PASS/registry_ok=True）、f3450 两跑确定性、
  0 容差 diff=2352px（AR171-1 更正：2076 为旧 expected `788d` 历史值）；task-170 工作区 17/17 重建（三大户 text sha 前后全同、2 对重渲逐字节同）。
- 派单：task-171 合并独立复核 → decomp-auditor（mimo）四项重导（两跑 sha/0 容差 diff 复算/
  --validate 复算/工作区自证）；pass 后 Lead 分组 commit+push。
- 429：未发生。


## 【2026-10-20 00:20】唤醒巡查×2（定时器 00:13/00:23 批量）+ 安装批收口
- 0 检查行任务：task-171 复核 verdict=needs_revision（AR171-1 ledger 一行数字）→ 已修（2076→2352）
  → 安装批 commit+push（本条随批）；无其它欠账（3acc661/b100c31 已推）。
- a 空闲判定：decomp-auditor 完成 task-171 后空闲；platform-porter 空闲（task-170 待复核收口）；
  其余空闲 ⇒ 执行 b/c。
- b 方向判定：与目标一致 ✓（L4 收敛=#14 安装收口 + 2352 基线数字入账 = P0-1 修复目标第 14 对
  已定标；下一步=P0-1 差异族攻坚）。判定人：Lead。
- c 派单：① b100c31 例外收尾卡（刷新 v2 #14 run1 sha 或切 g-manifest-full 基线后移除例外/改
  allowlist）→ platform-porter；② P0-1 差异族攻坚首族（调色板 W-C1 9px+族映射，按 attack-plan
  族序）→ platform-porter；③ task-170 复核收口 → decomp-auditor 补一项（见下）。
  理由：安装批已定标，进入 diff 收敛主线；b100c31 收尾与攻坚不冲突（工具 vs 渲染代码）。
- d 429：未发生。
- Follow-up 登记：AR171-2（--validate 路径口径 g-manifest-full vs v2）、AR171-3（v2 target 内
  #14 idx_sha/diff_pixels 旧值，gitignored 无碍但建议刷新）。


## 【2026-10-20 00:25】唤醒巡查（定时器 23:53/00:03/00:13 三 occurrence 合并）
- 0 检查行任务：AR171-1 已真修（45093dd，L68=2352 核实）；安装批 d9e90ed 已推；3acc661/b100c31
  已推；工作树 clean、0 领先 ⇒ 无欠账。更正：00:20 条目"派单"当时只落库未实派——本轮补实派。
- a 空闲判定：全队空闲（decomp-auditor/platform-porter/shell-porter 等）⇒ 执行 b/c。
- b 方向判定：一致 ✓（#14 安装收口完成=基线数字定标 2352；进入 P0-1 diff 收敛主线=目标①正面
  战场）。判定人：Lead。
- c 派单（实派+建卡）：① task-171 b100c31 豁免收尾 → platform-porter（v2 sha 刷新/allowlist 化+
  反例测试）；② task-172 P0-1 首族攻坚 → shell-porter（attack-plan 族序收益最大者；渲染面+
  diff 降幅为证）；③ task-170 收口 → platform-porter（复核数据面已过，board 置 completed）。
  理由：例外收尾与主战场不冲突（tools vs crates）；派满 2 实施 1 收口。
- d 429：未发生。
- 备注：退役 flash 成员又自发完成 AR171-1 修复（45093dd）——成果经证据核收（L68 实值 2352），
  仍不向其派单；团队执行面维持 100% mimo。


## 【2026-10-20 00:28】唤醒巡查（定时器 00:23）+ 操作模式切换（用户模型裁定）
- 0 检查：f3bb1ae 已推；task-170 收口（board completed）；task-171 实现已抢在中断前完成并
  push（f0e9be1，allowlist 化+11/11 单测）——**待复核**；工作树 clean、0 领先。
- **重大发现**：list_agents 显示运行中 turn 的 platform-porter/shell-porter=**deepseek-flash**
  （创建期钉死；显示随 turn 状态变化，l4-analyst 反例证明可翻转）⇒ 队友 turn 无法保证 mimo。
  按用户裁定（"团队必须使用当前模型"）：**操作模式切换=实施/复核一律走显式 mimo 子 agent
  （provider=mify model=xiaomi/mimo-v2.6-pro）**；队友仅保留台账/收口辅助，不再承载实施。
- 处置：中断 shell-porter flash turn（零残留）；platform-porter flash turn 已自然完成
  （f0e9be1 成果按证据核收）。
- a：全队空闲 ⇒ b：方向一致 ✓（#14 定标后进 P0-1 收敛）⇒ c 派单（显式 mimo 子 agent）：
  ① f0e9be1 对抗复核（allowlist 语义/mutation 探针/--validate）→ mimo 子 agent；
  ② task-172 P0-1 首族攻坚实施 → mimo 子 agent（渲染面+diff 降幅为证）。
- d 429：未发生。


## 【2026-10-20 00:55】唤醒巡查（定时器批量）+ 族①首迭代（Lead 实施例外）
- 0 检查：f0e9be1 复核 pass（adversarial-review-v2-allowlist.md 待随批提交）；工作树=族①迭代
  改动+文档；无其它欠账。
- 重大执行事件：P0-1 实施子 agent **3 连崩**（后台×2+前台×1，mimo 路由长 turn 不稳）⇒ 按纪律
  "结果优先"由 **Lead 亲自实施**（偏离"Leader 不替代实施"，披露：实施=Lead、复核=独立 mimo
  子 agent，实施≠复核保持）。
- 族①首迭代结果（如实）：A 级模型镜像 A/B **混合**（#14 −214/gameplay +312/+10）⇒ 按 AGENTS
  §3.5 停试错、默认回退 legacy（回归核 2352/3742 全同）、模型路留 MODELGEO=1；未解交互项
  （t93 四字段映射/行提取窗/attr 相位/\ 批次）→ 取证卡优先。详见 p0-1-family-report.md。
- b 方向：仍与目标①一致（负结果也是收敛证据：排除几何单项根因，聚焦取证）。
- c 派单：族①首迭代复核 → 独立 mimo 子 agent；取证卡（t93 字段映射+\ 批次语义）待复核
  pass 后派。
- d 429：未发生。
