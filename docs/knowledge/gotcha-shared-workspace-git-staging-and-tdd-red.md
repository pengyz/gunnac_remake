---
type: gotcha
title: 共享工作区两大提交坑：暂存区污染与 TDD RED 挂红
applies_to: [multi-agent, git-workflow, tdd]
verified: 2026-10-15
---

# 共享工作区两大提交坑

## 坑 1：暂存区是共享资源（2026-10-15 实例）
`git add <我的路径> && git commit` 非原子——commit 提交**整个暂存区**。多 agent 并行时
他人已暂存产物会被一并扫入（实例：port-engineer 首提误扫 8 个他人文件）。**修复**：
`git reset --soft HEAD~1` → `git restore --staged <他人路径>`（内容零改动）→ 重发定向提交。
**预防**：提交前 `git diff --cached --name-only` 核暂存纯度；或一律 `git commit -- <pathspec>` 定向
（注意：untracked 文件须先 add 再 pathspec 提交，否则"未匹配任何 git 已知文件"）。

## 坑 2：TDD RED 不可跨回合挂红（2026-10-15 实例）
RED 态测试文件（编译不过）落在共享工作树 ⇒ cargo test 全仓红 ⇒ **pre-commit 闸门拦住所有成员的提交**
（实例：wave_sched.rs RED 挂红，lead 的 T5 提交被拒）。**预防**：RED 只存在于即将闭合的回合内；
实现落地前将测试暂改名 `.rs.pending`；或实现+测试同 commit 落地。共享工作区纪律：**任何时点工作树须可 commit**。

## 坑 3：半成品挂树 + mutation 还原 = 归零覆盖（2026-10-15 实例 2）
半成品 WIP 隔夜挂树，期间他人 mutation 实验的 `git checkout --` 还原会把 WIP 一并归零
（实例：port-engineer enemy_behavior 半成品被 mutation 归零覆盖）。**预防**：WIP 即做即 commit
（同回合 RED→GREEN→commit）；mutation 还原只还原自己本回合改动的文件（还原前 diff 确认属主）。

## 坑 1 补充：replace 静默未中根治配方（2026-10-15 第 3-6 次实例）
同一 replace 连续 3 次静默未中（全角冒号/反引号截串/漏 \$）后根治：**①hexdump 直读目标行真字节**（不猜字符形态）②断言式替换（`assert s.count(old)==1`）③改后 grep 复验。链条：猜形态→未中→grep 0→断言拦→hexdump→真形态→断言过→grep 1。教训：中文全角标点（：efbc9a）/反引号/转义是最常见三坑。

## 坑 4：git checkout -- 不回暂存区污染（2026-10-16 实例）
暂存区被他人 add 污染后 `git checkout -- <file>` **不生效**（工作树与暂存一致时无 diff 可还原）；
真回退须 `git restore --staged --source=HEAD <file>`（暂存+工作树双还原）或 `git reset HEAD <file>`+checkout 组合。暂存区污染的衍生坑：处置动作本身踩坑。

## 关联
- 修复验证：git diff 复核+暂存纯度表列；门禁恢复后重提
- 参考：adversarial-review 系列"声称已修原地未动"教训同族（改动面必须机器核对）
