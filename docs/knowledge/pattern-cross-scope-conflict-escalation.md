---
type: pattern
title: 跨 scope 冲突处置：不越界不变通，带选项上报裁定
date: 2026-10-12
status: confirmed
---

# pattern：跨 scope 冲突 → 上报裁定（不自决、不变通）

## 现象

t3 实现门禁时，docs-links 预扫描发现 AGENTS.md §6 引用的 `docs/bug-ledger.md` 不存在
（16 条本地链接 1 条失效）。修复它要动 `docs/`——**不在 porter 的 inScope 内**。
porter 的处置：不越界、不绕过，按纪律**上报并附三个选项**请 captain 裁定：

| 选项 | 裁定 | 理由 |
|---|---|---|
| A. 补 bug-ledger stub | ✅ 批准执行 | 台账是 D4 纪律必需物，stub 为零逻辑脚手架 |
| B. baseline TOLERATED | ❌ 否决 | 与"白名单唯一例外=visual"设计原则冲突（porter 自己点出张力） |
| C. 保持严格红 | ⤵️ 降为兜底 | A 落地后不需要让 pre-commit 全红阻塞全队 |

同类先例：round-006 F-01/F-02（红线/纪律文档建议）都按"纪律变更 → 待用户确认"上报而非直接改。
round-007 §③ 把"成员连续两次主动上报跨 scope 冲突/路径问题而非自行变通"记为**门禁文化成型**标志。

## 为什么

- 任务卡的 inScope 是硬边界：写路径互斥 + 成员并行，自决越界会踩他人在途产出、破坏验收口径。
- 裁定权与执行权分离：范围/纪律/白名单类决策归 captain（白名单/基线/纪律变更再升用户，AGENTS.md §5）；
  成员的价值在**把冲突整理成可裁定的选项**，而不是自行变通。
- 上报带选项（≥2）+ 各自张力，比"请示怎么办"高效得多——round-007 一拍即裁。

## 何时使用

- 任何"要改的文件不在自己 inScope"或"发现的问题属于他人产出/纪律文档/门禁本身"的时刻。
- 上报格式：**冲突描述（哪条纪律 × 哪个缺口）+ ≥2 个选项 + 各自代价/张力 + 推荐倾向**。
- 涉及 AGENTS.md/门禁/白名单/基线的变更一律附"需用户确认"标注，不自决。
  > 【2026-10-12 更正】基线变更细分两类：**进度类上调**由 captain 验收后代执行（成员遇 PENDING WARN
  > 上报 captain 即可、不停工等待），**下调/放宽/白名单**仍需用户确认——详见
  > [decision-baseline-raise-policy.md](decision-baseline-raise-policy.md)。
- 反例：为了让自己任务过门禁而顺手改他人文件、加白名单、放宽阈值——一律禁止。

## 证据指针

- `docs/rounds/round-007.md` §①（t3 跨 scope 冲突裁定全过程与三选项裁定表）
- `docs/rounds/round-007.md` §③（纪律健康度：连续两次主动上报）
- `docs/rounds/round-006.md` §② F-01/F-02（纪律变更建议 → 待用户确认的先例）
- `docs/rounds/round-003.md` §②（路径问题的上报指引先例）
