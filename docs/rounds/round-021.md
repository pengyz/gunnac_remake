# 轮次日志 round-021 —— M3 模块潮 + git 误卷归属更正

- 日期：2026-10-12 ｜ Goal：goal-a8a36131（rev 2）｜ 覆盖 t58–t75 时段

## ① M3 模块潮（Rust 主战场，用户指令后加码）

| 模块 | 状态 | 验证面 |
|---|---|---|
| 渲染骨架 game-desktop（首个可见的游戏） | ✅ t58 | CHR golden + export_png |
| enemy（装载/命中清除/框系统） | ✅ t62/t68 | L2 89→90 + 命中样本 12 例 |
| terrain（VM 内核归位 t71，行为零变化重构） | ✅ | L2/L3 原样全绿 |
| stage 卷轴（−2px/帧 + mod256 回绕 quirk + 双通道） | ✅ t73 | L3 1207 帧 $1C/$1D 全等 |
| **在飞**：UI/商店/金钱（t77）、W21（t76） | 🔄 | — |

计分：cargo **52** ｜ L2 **90** ｜ L3-DIFF **4181 项** ｜ 台账 **95**（5 reverted）｜ 矩阵 50 格 covered=10（**B-STG-01-TL=首个 L3 内容格**）

## ② 关键更正链（本时段证伪/撤回实录）

- **控制方案大修**（t52）：主武器=B 长按连发（推翻"自动连射/B=切换"，N-0072/73）
- **VM 定性反转**（t64/t65）：敌行为机→**BG/地形脚本机**（W12 前提证伪，三层 RAM）
- **spawn 节律证伪**（t72/t75）：16f→8-10f 主导（N-0094）
- **寻址统一**（t66/t67）：n×$1C 误读→字段基址+n（N-0086，六文件扫荡）
- **W9 收口**（t72/t75）：$072E=在途弹计数（±1 成对），"周期表"作废
- stride ×3→×5（N-0088）、churn 口径重算、R-shot-level-3 公式 3+→4+ 进位勘误

## ③ git 误卷归属更正（gotcha 第三例，按纪律不删历史）

> 【归属更正 2026-10-12】commit `d09ed94`（t75）卷入共享索引中的他人在途产物：
> `docs/findings/enemy-closeout-w15.md` 实为 **t72 产物**、baseline 四件（ledger/baselines）
> 实为 **captain 验收在途产物**（后由 `c6da336` 正常承接，Evidence 95，内容零损）。
> 教训入库：他人暂存文件在共享索引时，无 pathspec 的 commit 会连带收走 —— **提交前先
> `git status` 核对索引整体**（gotcha-git-add-commit-atomicity 第三例，50d0ce2/54f2809 后）。

## ④ 在飞与队列

t76 W21（B-STG 子格前置，双标首选）｜ t77 UI/商店/金钱模块 ｜ 余缺口：W15 三-way 对拍（W24 阅读端）、B-STG EN/BOSS 子格、visual-diff（渲染内容化后）
