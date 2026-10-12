# 轮次日志 round-009 —— 三重验收轮（M0 基本收官）

- 日期：2026-10-12 ｜ Goal：goal-a8a36131（revision 2）

## ① 验收 ×3（全部独立复现后放行）

### t3 gates/cli.py 门禁骨架 ✅
| 项 | 结果 |
|---|---|
| verify 三命令（我亲跑） | `status` exit 0（11 门禁、7 SKIP 全显式带原因）｜`rom` exit 0｜`check` exit 0（PASS=4 FAIL=0 SKIP=7，0 静默跳过） |
| ratchet 基线表 | 7 指标逐行对照 OK，TOLERATED 空 |
| README↔cli 同步 | 注册表 11 项对齐（selftest 自动比对）；selftest/baseline/show/tolerate/update 为新增子命令，porter 称已同步 —— **交 t4 实测终审** |
| 冲突处置 | bug-ledger 按方案 A 收口，docs-links 现绿（24 md/28 链接全有效） |

### t2 S0 ROM 取证 ✅（对照 round-004 基准，全中）
| 项 | 结果 |
|---|---|
| 向量表 | NMI=$EF85 / RESET=$E000 / IRQ=$F20B —— **与我的独立基准逐值一致** |
| far-call 计数 | 报告 18 处 ↔ 我独立复现 `20 49 E6` 计数 **18=18** |
| 结构纪律 | 已证实/假设分节 ✓；H1-H9 假设隔离 ✓；路径契约 ✓（da6502.py/chr_render.py 入 tools/） |
| 超预期 | far-call 跳板约定（JSR $E649+内联4字节）、bank11 JMP 入口表、文本记录格式 {PPU,tile,FE/FF}、汉化线索链、字模区 bank124-127 —— S1 的路标已铺好 |

### t6 证据规格化 ✅
| 项 | 结果 |
|---|---|
| 规则 | 12 条 R-xxx 全带证据指针+可判定性+关联格；11 项交互边界留"待 S2 取证"未发明 |
| 证据抽查 | E-R04 更正指针 rom:8516-8523 实测 `-------$` ✓（$@8523）|
| 矩阵登记 | completeness §2.4 三格升 **L1**（web 证据封顶规则执行正确）、版本 v0.1 |
| 事故恢复 | 429 中断后 attempt 2 终验补齐，契约报告完整 |

## ② 调度（4 项）

| 任务 | 内容 | 负责 |
|---|---|---|
| t4 重试 | t3 已交付，唤醒四场景取证 + README↔cli 终审 | verifier |
| t7 | S1 双投影（IDA+Ghidra → seed 冻结 + 符号/数据面分母机械导出） | decomp-analyst |
| t8 | S2 oracle 场景采集基建（场景 DSL + trace 导出 + 首批 3 场景，补 gameplay 实机画面） | porter |
| t9 | docs/knowledge/ 知识库建立（README + 教训条目化，激活 knowledge-format 校验） | spec-writer |

## ③ 进度检查

- **M0 基建：收官在望** —— 团队 ✅ / 矩阵 v0.1 ✅ / git 基线 ✅ / 门禁骨架 ✅（t4 终审后正式关闭）
- **完整循环计数：3/3 达成** 🎯（t1、t5、t2/t3/t6 均走完"派→执行→交付→独立验收"；Goal 判据之一满足，持续计数）
- 矩阵收敛：规则面首批 L1（3 格）；分母 44 格，强度分布 L1×3 / L0×41
- 下一最优格：gameplay 证据（t8 在途）→ 解锁 B-SYS-ITEM/MONEY/SHOP 交互边界；符号/数据面（t7 在途）→ 分母增长
- 限流风险：已缓解（本轮全员恢复产出）

## ④ 追加：t4 评审裁定（needs_revision，质量回路首轮运转）

**t4 verdict = needs_revision**（自动跟进 t10 repair→porter / t11 review r2→verifier 已生成，不重复建任务）：

| 项 | 结果 |
|---|---|
| ① 门禁清单 11/11 ↔ README 逐字 | ✅（独立脚本比对） |
| ② SKIP/TOLERATED 显式打印 | ✅（构造场景实测；TOLERATED 不能洗绿 FAIL） |
| ④ commit-msg 证据门禁 | ✅（临时 git 仓 7 次真实 commit 实测） |
| ⑤ 无绕过/无第二套扫描 | ✅ |
| ③ **ratchet 单调性** | ❌ **R1(high)**：手工改低 baseline.json 无机器检出，可把 check 翻绿零留痕 —— "只升不降"只在 update 命令内强制 |
| R2/R3/R4 (low) | 命令表↔子命令面漂移+selftest 单向；单门禁输出头错误；commit-msg 别名『信心:』/删除路径语义未文档化 |

**captain 处置**：R1 = 裁判的尺子有洞，定为最高优先级 —— 指令 porter 收尾当前步骤后立即切 t10（t8 暂挂）；
修复方向照准 verifier 建议（baseline.json sha 锚点 + history 交叉校验）。verifier 待 t10 完成后复审（t11）。

**意义**：质量回路（review→findings→自动 repair→复审）首次完整触发，评审方独立取证、
被评审方零抵赖、跟进任务自动生成 —— 全自动反编译流水线的"验收"环节经受住了第一次真实对抗。

## ⑤ 下轮计划

t10 修复验收 → t11 复审 pass 才算 t3 收口 → t7/t8/t9 验收 → M0 正式关闭 + M1 进度核算。
