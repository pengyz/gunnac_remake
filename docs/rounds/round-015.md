# 轮次日志 round-015 —— t18 验收 + M2 中盘三连派卡

- 日期：2026-10-12 ｜ Goal：goal-a8a36131（revision 2）

## ① t18（S3 语义命名第一批）验收 ✅

| 项 | 核验 |
|---|---|
| 坐标 A 级证据 | player_x=$04E0/player_y=$04FC，钳制常量（CMP #$F4/LDA #$F3、CMP #$E0/LDA #$DF）在台账证据实存；探针实测钳点（243/14/223）三方一致 |
| 负结果纪律 | 金钱源 12 编码穷举空集 → **如实存档** _todo_money_display_src（不编造、指明 WRAM 方向） |
| 证伪文化 | t_003"数值曲线表"猜测证伪（实为代码前数据岛，含 STA $04FC 真指令流）→ 原地日期更正 |
| 方法论沉淀 | mixed_code 判据 = branch-align%（数据 43-77% vs 代码 84-97%，阈值 80% 固化进 tools/remap/run.py） |
| 台账 | 10→27 条（matrix_sync --check 无漂移）；naming 10→27 代执行（aa7ab77） |

## ② M2 中盘三连（全员派发）

| 任务 | 一句话 | 负责 |
|---|---|---|
| **t21** | L2 子程序级差分基建：6502 CPU 核 harness + subroutine-diff 激活（首批纯函数：钳制/取位/查表） | porter |
| **t22** | 金钱 WRAM 定位（负结果定向补证）+ HUD 残机/等级/红圈 + H-11/H-12 收口 | decomp-analyst |
| **t23** | S4 行为规则第一批：自机移动/射击（速度对/钳制/投放例程 → player-movement.md） | spec-writer |

## ③ 进度检查

- 门禁 8/11 启用；矩阵 44 格（L2×4）；规则面 21 条；台账 27 条；Rust 12 tests
- 完整循环 12；M2 进度过半（crates/解析器 ✅、规则回填 ✅、L2 基建在途、S4 开张）
- 下一最优格兑现链：t21 → subroutine-diff 激活 → t23 规则可直接转 L2 用例

## ④ 下轮计划

t21/t22/t23 验收 → M2 收口条件盘点（L2 首批用例 + S4 移动/射击规格）→ 首个行为模块 Rust 移植卡。
