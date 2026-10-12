---
type: pattern
title: 验收 = 独立复现证据指针（不看报告写得对不对）
date: 2026-10-12
status: confirmed
---

# pattern：证据抽查独立复现验收法

## 现象

captain 验收不采信产出报告的自述，而是**抽样证据指针并独立重跑**，逐条比对：

- round-005（t1 矩阵验收）：证据抽查 **6/6 命中**——`F B T W`@9626、`WEAPON-0`@5419、
  `JIUBAN`@7113、`INVESTIGATE THE EIGHT AREAS`@12036、价目区 `$10/$6/$4`@8914–9000、
  截图 sha256——全部与既有基准及实测一致才判通过；
- round-006（t5 git 基线验收）：**5/5 独立复现**（`sha256sum -c`、`git check-ignore -v` 逐项、
  `git ls-files | grep .nes` 为空、commit 规范、跟踪清单），且哈希值与 round-004 基准**逐字**比对；
- round-007 §④：后续验收"对照 round-004 基准"——基准先行，抽查对照基准，形成闭环。

同轮的反面教训也在：报告声称与实测可以不一致（round-006 F-01/F-02/F-03 全是验收实测才发现的）。

## 为什么

- 差分验证裁判的是代码，验收裁判的是**证据链**：指针可复现 ⇒ 结论可核验；
  指针是编的/抄错的 ⇒ 结论不可信（防"AI 全自动"幻觉产出，pipeline R6 / master-plan §7 三道闸之一）。
- 独立复现 = 用**验收人自己的命令**重跑（`grep -abo`、`sha256sum -c`、`git check-ignore`、门禁命令），
  而不是在报告里找答案；报告只是待验证的声明。
- "完成 ≠ 声称"（round-005）：跑过、贴出输出、且被独立重跑命中，才叫完成。

## 何时使用

- 每轮 captain 验收（master-plan §5 五条协议之②：证据指针抽查 A/B 级真实性）；
- review/verification 类任务裁定前；跨会话引用他人结论前（同 knowledge README 规则 6）。
- 抽样方法：每份产出抽 ≥2 条 A 级指针（偏移/哈希/命令输出）+ 1 条结构性声明（文件清单/计数）；
  与既有基准（round 日志、SHA256SUMS、矩阵登记）逐字比对。
- 抽查失败时按 AGENTS.md §4 定位层（投影错？报告错？验收脚本自己错？——先证伪自己的复现命令）。

## 证据指针

- `docs/master-plan.md` §5（captain 验收协议五条：门禁实跑/证据抽查/契约完整性/矩阵纪律/上报用户）
- `docs/rounds/round-005.md`（t1 验收：证据抽查独立复现 6/6；"完成≠声称"）
- `docs/rounds/round-006.md` §①（t5 验收 5/5 独立复现；哈希与 round-004 基准逐字一致）
- `docs/rounds/round-007.md` §④（对照 round-004 基准验收的闭环）
- `docs/rounds/round-001.md`（验收缺失 → 验收协议五条的由来）
