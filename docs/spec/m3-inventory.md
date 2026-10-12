# M3 中期盘点（m3-inventory）→ docs/spec/m3-inventory.md

> 状态：2026-10-12 v1（任务 t33，spec-writer）。盘点时点 = M2 进行中（t28/t30 后）。
> 用途：captain 调度的进度真相快照 + **模块移植顺序 DAG** + Stage A 判据逐项现状。
> 规划关联：[l5-plan.md](l5-plan.md)（L5 流程级）、[l4-visual-plan.md](l4-visual-plan.md)（L4 视觉级）。

## 1. 完备性矩阵收敛数据（machine matrix 实测）

| 面 | 分母 | 覆盖/状态 | 明细 |
|---|---|---|---|
| 行为面 | **47** 格 | **7 格 L2** / 40 格 L0 | 覆盖=B-SYS-SHOP/MONEY/ITEM/UI/MOVE/FIRE/RESPAWN；未覆盖=武器 25（B-WPN×25）+ 炸弹 4（B-BMB×4）+ 关卡 8（B-STG×8）+ 系统 3（DROP/BOSS/SCORE） |
| 规则面 | 47（下限） | **42 条目 / 7 格覆盖** | shop-economy 18 + ui-flow 8 + player-movement 16；⚠ machine-matrix `rule_entries` 元数据仍写 21（t20 硬编码遗留，**待办**：matrix_sync 改为动态计数） |
| 符号面 | 机械导出 ✅ | 276 子程序 + 3 向量 + 18 far-call + 7 bank11 表 + 560 RAM + 414 MMIO | 字节覆盖 131,072/131,072；语义命名 37 条台账（N-0001..0037） |
| 数据面 | 机械导出 ✅ | 8 机械表 + 7 锚点候选（不入分母）+ 128 CHR bank | t_002/003/004 重分类 mixed_code、t_007 边界修正（ram-watch-s3 §6） |

其他底数：命名台账 37 条（含 reverted：N-0022/0034；证伪撤回：N-0025/0031/0032 绑定；`_todo_`：N-0009/0010/0026/0027/0036/0037）；
知识库 10 条；L2 用例 **54**（oracle/subroutines/l2_cases.jsonl）；L3 试点 **951/951**（gameplay-stage1 player）；
Rust 测试 17；gate 全景 **PASS=10 SKIP=1**（仅 visual-diff SKIP）。

## 2. 模块移植顺序 DAG（player → weapons → shots → enemy → stage）

```
                ┌─────────────── spec 先行（S4）───────────────┐
                ▼                                           ▼
   [player ✅] ──► [weapons 🟡] ──► [shots ❌] ──► [enemy ❌] ──► [stage ❌]
        │              │                │             │             │
        └── [respawn ✅spec/未移植]     │             │             │
        └── [ui/shop/money ✅spec/未移植]◄─────────────┘（掉落/金钱）  │
                                          [boss ❌] ◄───────────────┘
```

| 节点 | 现状 | 依赖（spec 证据） | 下一步 |
|---|---|---|---|
| **player** | ✅ 已移植（`game-core::player.rs` 145 行：movement_tick/velocity_from_input/fire；L2 47 例 + L3 951 帧绿） | R-move-1..6、R-fire-1..6 | 收尾：L3 扩到全场景 |
| **weapons** | 🟡 部分（`weapons.rs` 94 行 5 fn） | N-0032/33（$34 槽/$33 级）、H-01/H-02 已收口、R-item-1 | 补 R-wpn-* 规格（射击模式 bank03:$A1F5 五型）→ 移植 |
| **shots** | ❌ 未动 + **规格缺口** | N-0035（shot_type_counters $072E,X 六弹型循环）已命名但 **R-shot-\* 规则未写** | **spec-writer 先行**：射击生成器规则（含射速阈值 $074A,Y）再移植 |
| **enemy** | ❌ 未动 | 生成表（数据面 t_00x）、object slot 区 $04xx 步 0x1C（N-0027 待定偏移）、hit-test（H-16 未证） | S3 表识别 + hit-test 取证双前置 |
| **stage** | ❌ 未动 | **B-STG-01..08 全 L0**（时间线/敌人/Boss 零取证）；滚动/M3 IRQ 行为 | M3 主战场：逐关 S2 场景 + 时间线规格 |
| ui/shop/money | ✅ spec L2（R-shop-1..8/R-money-1..6/R-bomb-1..3）未移植 | 可随时并行移植（与 player 无耦合） | 建议尽早 port：便宜且 L3 断言字段已命名 |
| respawn | ✅ spec L2（R-respawn-1..4，L3 已可判）未移植 | 依赖 player + 死亡触发（hit-test H-16） | 与 weapons 并行 |
| boss | ❌ | B-SYS-BOSS L0（仅 E-W07 最终 Boss 多相 B 级） | 随 stage |

**排序理由**：player 是坐标/输入基座（一切对象挂玩家坐标快照）；weapons/shots 是输出面（依赖 player 坐标 + RNG）；
enemy 是交互面（依赖 shots 判定 + slot 区）；stage 是编排面（依赖全部）。
**并行安全**：ui/shop/money 与 respawn 不阻塞主线，可与 weapons 同批；**shots 的 spec 是当前关键路径第一缺口**。

## 3. Stage A 完成判据逐项现状（master-plan §6 + Goal 终态）

| # | 判据 | 现状 | 缺口 |
|---|---|---|---|
| 1 | 迭代流水线定稿 + ≥3 轮完整循环 | ✅ round-001..017+，完整循环 ≥14 | — |
| 2 | 矩阵 v0 建立且四维收敛记录 | ✅ v0.6，四面均有收敛记录 | machine-matrix 元数据小修（§1 ⚠） |
| 3 | **8 关可通关（L5 全绿）** | ❌ 未开始 | l5-plan 已出；前置=STG 时间线取证 + DSL C1-C3 扩展 |
| 4 | **L1–L3 全绿** | 🟡 局部绿 | L1 文本 golden ✅；L2 54 例绿但仅 player/文本域；L3 仅 player 试点 —— **模块面**随 DAG 扩 |
| 5 | **L4 白名单外像素一致** | ❌ SKIP | l4-visual-plan 已出；前置=V1-V5 建设 + 白名单候选用户确认 |
| 6 | `gates/cli.py all` 本地可跑 + CI 等效 | 🟡 本地 ✅（PASS=10 SKIP=1） | **CI 无 workflow**（AGENTS.md §9 承诺 push/PR 等效检查）——待办 |
| 7 | 台账/知识库齐备 + 提交纪律全程执行 | ✅ | naming 37 / bug-ledger / knowledge 10；Evidence 门禁多轮实证 |

**总评**：基建与证据链（M0-M2 前半）扎实；**内容面（8 关）与视觉面是 Stage A 的两座大山**，
关键路径 = shots 规格 → enemy（hit-test/生成表）→ stage 时间线 → L5。

## 4. 风险与待办队列（供调度）

| # | 项 | 级 | 归属 |
|---|---|---|---|
| 1 | R-shot-\* 规格缺口（N-0035 已命名未规格化） | 高（关键路径） | spec-writer（下一批） |
| 2 | B-STG-01..08 时间线取证（8×L0） | 高 | S2 场景 + S3 表（M3 主线） |
| 3 | hit-test（H-16）取证件 | 高 | t25 hit-test.txt 重跑（需 C1 种子控制更稳） |
| 4 | DSL 扩展 C1-C3（种子/快照/分段） | 高 | porter（tools/scenario_run.py） |
| 5 | L4 V1-V5 建设 + 白名单候选用户确认 | 中 | verifier + 用户 |
| 6 | CI workflow 缺失 | 中 | porter（gates 等效） |
| 7 | machine-matrix `rule_entries` 元数据动态化 | 低 | spec-writer（matrix_sync 小修） |
| 8 | GAME OVER/CONTINUE、残机 var（W3） | 中 | S2 场景 |
