---
type: gotcha
title: git 索引并发无隔离——add/commit 原子窗口 + 精确 pathspec，误卷不删历史
date: 2026-10-12
status: confirmed
---

# gotcha：git add/commit 并发卷入（索引共享，原子性缺失）

## 现象

并行成员各自 `git add`/`git commit` 时，**共享索引**把他人在途/已暂存文件卷进自己的提交。
两起实证：

1. **t33 三份规划文档被卷入 `50d0ce2`**（t31 weapons 验收提交）：`docs/spec/{l5-plan,l4-visual-plan,
   m3-inventory}.md` 在 t33 `git add` 之后、`git commit` 之前被并行提交扫走——t33 自己的 commit 报
   "无暂存内容"，产物却出现在他人 commit 的 stat 里。
2. **t34 `data/status/coverage_map.json` 被卷入 `54f2809`**（t32 敌人数据面验收提交）：
   `git log --diff-filter=A` 证实该文件在 t32 的验收 commit 中**首次入库**，而它是 t34 审计任务的产物。

## 为什么

- git 的工作树 + 暂存索引是**进程间共享的单一资源**，没有 per-actor 索引；
  `git add -A` / `git add .` 会把别人此刻的改动一并暂存。
- **add 与 commit 之间的窗口越大**，并发卷入概率越高；窗口内任何他人 commit 都可能把你的暂存扫走
  （对方也是 `add -A` 时概率最高）。
- 与 [gotcha-concurrent-mesen-output-corruption.md](gotcha-concurrent-mesen-output-corruption.md)
  （Mesen 共写输出）、[gotcha-parallel-read-image-order-unreliable.md](gotcha-parallel-read-image-order-unreliable.md)
  （读图序）同族：**共享工件无隔离 = 并行不可信家族**第三员。

## 何时使用

- 提交用**精确 pathspec**：`git add <明确文件列表>`，禁 `add -A`/`add .`；
  最好 `git commit -m … -- <pathspec>` 直提，压缩 add/commit 窗口为原子操作。
- 提交前 `git status --short` 核对**暂存清单只含自己的文件**再回车。
- 发现误卷：**不删历史**（不 reset/rebase 已落地 commit），在后续 commit message 或轮次日志注明
  "文件 X 实为 tXX 产物，随 commit Y 卷入"，保持作者/归属可追溯。
- 反例（本条来源）：`add -A` + 独立 commit 步骤在 4 成员并行轮中连续两次误卷。

## 证据指针

- commit `50d0ce2`（stat 含 t33 三份 docs/spec 规划文档）+ t33 任务输出/上报（"git 索引并发现象"）
- commit `54f2809`（`git log --diff-filter=A -- data/status/coverage_map.json` 首次入库于此，
  实为 t34 审计产物）+ t34 任务输出
- 族内互链：Mesen 共写输出 gotcha、read_image 错序 gotcha（并行不可信家族）
