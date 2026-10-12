---
type: gotcha
title: 断言存在≠断言生效——假绿空转（解析层级错致断言白跑）必做负向自证
date: 2026-10-12
status: confirmed
---

# gotcha：断言存在 ≠ 断言生效（假绿空转）

## 现象

porter 在 commit `caed518` 自查发现：`ss-1/ss-2`（shot 侧 L2 用例）的 **out.exit 分支落点断言空转**
——`out.exit` 误从 case 顶层解析（golden 实际写在 `out` 内层），断言对象解析错层、
**恒等比较两边都是默认值**，测试全绿却什么都没验。修复后做**负向自证**（故意改坏实现→断言转红→复原），
确认断言链路生效后绿灯才可信。教训一句话：**断言存在 ≠ 断言生效**。

## 为什么

- 断言代码"在场"只保证它被编写，不保证它被**喂到正确对象**：解析层级错、字段名漂移、
  空集合循环、短路 return、默认值恒等——任何一环都让断言静默跳过或白比。
- 绿的语义被偷换：本应是"跑对了"，实际是"没跑到"。与 t34 覆盖审计的**空断言假覆盖**同族
  （一个是断言没写，一个是断言没接上），与
  [gotcha-stack-page-game-var-watch-coverage.md](gotcha-stack-page-game-var-watch-coverage.md)（采样面盲区）、
  [gotcha-observation-granularity-fabricates-invariants.md](gotcha-observation-granularity-fabricates-invariants.md)
  （观测粒度）并列：**验证链自身的缺陷伪装成通过**。

## 何时使用

- 新增/改动断言后必须做**负向自证**：故意注入缺陷 → 必须转红 → 复原；转不了红的断言=没生效。
- 读绿灯先问三件事：断言对象**解析层级**对吗？比较两侧**都不是默认值**吗？循环**真迭代到东西**了吗？
- 解析 schema/字段路径/golden 布局变更时，负向自证跟着复跑（`caed518` 的错层正是布局理解错）。
- 复核他人"全绿"声明时抽查一个断言做负向自证（比读代码可靠）。

## 证据指针

- commit `caed518`（"exit 分支落点断言修复（ss-1/2 空转转实）"：out.exit 解析层级错 + 负向自证复原）
- `oracle/subroutines/l2_cases.jsonl` ss-1/ss-2（exit 分支落点用例）
- t34 `docs/findings/coverage-audit.md`（空断言假覆盖族先例：10 例抽查含边界负例的动因）
- t50/t53 任务记录（F2 churn 统计同族教训：口径错的"证据"也属验证链缺陷）
