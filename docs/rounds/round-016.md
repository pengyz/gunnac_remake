# 轮次日志 round-016 —— 🎯 L2 红利兑现 + 金钱破案 + M2→M3 派卡

- 日期：2026-10-12 ｜ Goal：goal-a8a36131（revision 2）

## ① t21（L2 子程序级差分基建）验收 ✅ —— 里程碑

- 6502 CPU 核（151 条全集/非法码 fail-closed/2A03 无 BCD/**JMP 间接页 bug 保真**）+ L2Bus bank 窗
- **33 用例三方裁定**：golden == ROM 原字节执行 == game-core::l2ports 同名函数（独立复跑 33/33）
- **subroutine-diff 门禁 SKIP→PASS**（第 9 项实转）；cargo test 15/15；bomb_drop 全链（含 t23 点名易错链）
- 栈页坑沉淀：$018E/$018F 在硬件栈页，剔栈噪声会误吞游戏写入 → write_stack 专用通道
- game-core::l2ports 已含首批真移植函数（钳制/取位/查表/投放）—— **移植第一天就带函数级裁判**
- 执策：rust_test_count 12→15（692f282）

## ② t22（金钱破案 + H-11/H-12 收口）验收 ✅

- **money_counter_bcd=$E0-$E6**（7 位 BCD=显示值本体）：加法器+渲染器+探针时间线三链吻合
- t18 负结果双根因更正（read_image 错序 + 编码误设）；**W1"疑 WRAM"证伪**（PRG-RAM 未映射）
- bomb_type_index=$0190：拾取『W』→ $0190 0→3 + HUD F→W + 弹数 3→4 三方同步（H-12 收口）
- 台账 27→31（含 N-0031 证伪撤回）；执策 naming 27→31（0e78f1c）
- 方法论 gotcha 上报：并行 read_image 结果序不可信 → 已排入 t26 知识条目

## ③ M2→M3 过渡派卡

| 任务 | 内容 | 负责 |
|---|---|---|
| t24 | S5 首模块移植 game-core::player + **L3 trace-diff 试点**（首个场景级用例激活第 10 门禁） | porter |
| t25 | S2 场景扩展二批：武器拾取/受击残机/A 键分工（H-01/02/14/15/W3 取证） | decomp-analyst |
| t26 | t22 证据消化：金钱/炸弹规则升级 + constants v3 + read_image gotcha 入库 | spec-writer |

## ④ 进度检查

- 门禁 9/11 实转（剩 trace-diff/visual-diff）｜矩阵 46 格 covered=6｜规则 33 条｜台账 31 条｜L2 用例 33
- 完整循环 14；M2 核心件齐备（L2 基建+S4 规则+首批移植函数），t24 开 M3（移植主线）
- 矩阵下一最优格：B-WPN 25 格（等 t25 武器拾取证据）→ S4 武器规格 → 移植

## ⑤ 下轮计划

t24/t25/t26 验收 → trace-diff 激活确认（L3）→ M2 收口判定 → 武器系统规格卡。
