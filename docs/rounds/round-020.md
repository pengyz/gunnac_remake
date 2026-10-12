# 轮次日志 round-020 —— DAG 调度原则确立 + t29 验收 + 互斥并行批

- 日期：2026-10-12 ｜ Goal：goal-a8a36131（rev 2）｜ Goal round 10-11

## ① 调度原则修正（用户指令，覆盖 round-017/19 的过度保守）

**"避免过度并行" ≠ 全部串行** —— 正确做法是**按 DAG 拆任务**：
- **写路径互斥的任务并行派**（docs/findings ｜ docs/spec ｜ data/status ｜ crates/ 各自独立）
- **共享写路径的任务用依赖串起来**（shots 移植 = f(t31, t29)；敌人移植 = f(t32)）
- TDD 红相独占 crates/ 写路径窗口（round-017 学习项保留，但只约束该路径，不冻结全队）

## ② t29（弹丸行为考证）验收 ✅

- 弹型参数全链（六型循环/弹对象字段/运动机 16 向 LUT/动作流/时长表）—— **LUT 字节抽验实存**
- **H-02 完整答案**：weapon_level $33 = 12 级配置选择，升级=并行弹型数/频率变化（非换弹型码）
- Q1/Q3 收尾：f2220 双写=同帧两处理器（同值巧合）、$0401=bomb_deploy_state（N-0049）、N-0050 撤回
- 台账 37→50、data 面 7→13；执策 naming 37→50 + 清账提交（knowledge 8→10/t30 产物/t29 产物/rounds 一并落地）
- D3 锚点入库债清偿（reference/seed + hooks，round-010 发现）

## ③ 互斥并行批（DAG 派发）

| 任务 | 写路径 | 负责 | 后继依赖 |
|---|---|---|---|
| t31 weapons 骨架（在飞） | crates/game-core | porter | shots 移植等它 |
| t32 敌人/弹幕数据面考证 | docs/findings + data/naming | decomp-analyst | 敌人移植等它 |
| t33 L5/L4 规划 + M3 盘点 | docs/spec | spec-writer | L5 场景实施 |
| t34 L2/L3 覆盖审计 + 假覆盖排查 | data/status + docs/findings | verifier | completeness 融合 |

## ④ 进度

门禁 10/11 ｜ L2 54 用例 ｜ L3 951 帧 ｜ 矩阵 47 格 covered=7 ｜ 台账 50 ｜ 知识 10 ｜ 规则 42+ ｜ 完整循环 19。
待绿清账：rust_test_count 17→19（t31 报告后执策）。
