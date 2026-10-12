# 轮次日志 round-013 —— 🎯 M1 关闭（S2 场景证据到手，M2 开工）

- 日期：2026-10-12 ｜ Goal：goal-a8a36131（revision 2）

## ① t12（S2 oracle 场景采集）验收 ✅（对账全闭环）

| 声称 | 独立核验 | 结果 |
|---|---|---|
| 3 场景 1080/1450/2400 帧 | trace 行数 wc -l 逐个 = **1080/1450/2400** | ✅ 精确相等 |
| 确定性 trace sha `4b11e9be…` | sha256sum title-menu.frames.jsonl = `4b11e9be6619…` | ✅ 逐字一致 |
| 127 截图 + 127 RAM hex | 采集点 37+49+41=127 ✓（总文件 257/232 含探索探针帧，口径差异非偏差） | ✅ |
| run.json 证据清单 | analysis/traces/*.run.json ×6（3 场景 + 3 探索 run） | ✅ |
| gates | all rc=0、selftest 37/37 | ✅ |

**关键解谜**：商店卡死问题终结 —— B 切信息页（f1140『导弹20个上限』）、两连 B 退出进实机（f1260+）、
Start 无效（exp-shop-start 反证）。gameplay 实机画面（星域/自机/敌机/弹幕/HUD）采齐 = **矩阵 §2.6 入口解锁**。
运行纪律沉淀：场景串行（并发 Mesen 互扰）、Mesen 偶发 flake 显式报"未完成"不静默。

## ② 里程碑：M1 ✅（master-plan 已更新）

S0 rom-forensics ✅ ｜ S1 双投影 + seed 冻结 91,601 行 ✅ ｜ S2 3 场景 4,930 帧 trace 确定性实证 ✅

## ③ M2 首批派卡（三线并行）

| 任务 | 内容 | 负责 |
|---|---|---|
| t17 | S2 证据回填：B-SYS 交互规则 11 空位 + 矩阵 L2 登记（绑 scenario/trace） | spec-writer |
| t18 | S3 语义命名第一批：玩家坐标/金钱/HUD/弹数 watch 地址（RAM diff 动态取证）+ t_002/t_007 修正 | decomp-analyst |
| t19 | M2：crates/ 工作区骨架 + game-data 首解析器（L1 golden，先测试后实现）+ O2 文案修正（步骤 0 单独 commit） | porter |

## ④ 进度检查

- 矩阵收敛预期：t17 后 B-SYS 四格 L2；行为面 44 格 → 规则面覆盖率上升；符号面语义命名开张（t18）
- 完整循环计数：10
- Rust 侧即将开张（t19）：rust-tests 门禁激活后 M2 有机器裁判

## ⑤ 下轮计划

t17/t18/t19 验收 → M2 进度核算 → L2 差分准备（oracle/subroutines 建设 = subroutine-diff 门禁激活）。
