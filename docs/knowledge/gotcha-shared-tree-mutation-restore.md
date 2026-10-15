---
type: gotcha
title: 共享树 mutation 实验的还原方式——禁 git checkout 归零，用逆串还原；红绿归因对照当场基线
date: 2026-10-15
status: confirmed
---

# gotcha：共享树 mutation 还原覆盖半成品 + TDD-RED 污染归因

## 现象

两段实证（#88 复核窗口，adversarial-reviewer × port-engineer 并行）：

1. **`git checkout -- <file>` 归零还原覆盖他人挂树半成品**：reviewer 的 mutation
   （enemy.rs hi/lo 交换）还原时用 `git checkout`，把 port-engineer 未提交的
   TYPE_SIGNATURES 扩表**一并归零**——checkout 语义=对齐 HEAD，不是"撤销我的一处替换"。
2. **他人 TDD-RED 污染 mutation 红绿归因**：port-engineer 先行测试上树（实现未跟上）
   使基线带 1 红；reviewer 的 M4b2 突变红被**同函数先行断言遮蔽**（FAIL 停在更早的 assert），
   无法归因是"突变咬合"还是"基线已红"。

## 根因

- `git checkout` 的撤销粒度是**整个文件对齐 HEAD**，在共享树上等价于删除所有未提交改动；
- mutation 归因需要**单变量**：基线红/绿、突变红/绿两态差分，先行断言失败会吞掉后续断言的归因信号。

## 规避（双向纪律）

1. **mutation 还原一律逆串精确替换**（保存原文→替换注入→跑测→精确逆替换），**禁 git checkout**；
   若必须 checkout，先确认 `git diff --stat <file>` 为空（文件无他人改动）。
2. **归因对照当场基线**：注入前先跑一次取基线红绿计数，突变红数须**净增**才算咬合；
   同函数多断言时注意 FAIL 停点遮蔽（必要时先修基线或用独立用例）。
3. 他人半成品挂树时主动知会（同回合闭合原则：改动不出回合）；commit 被 rust-tests 挂红时
   先 `git status crates/` 区分"自己红/他人 TDD-RED"再定位（AGENTS.md §4 定位层）。

关联：gotcha-git-add-commit-atomicity（索引并发）/ a7fcc78 kb（提交双坑之二=TDD RED 挂红）。
