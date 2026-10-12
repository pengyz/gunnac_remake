# 轮次日志 round-006 —— t5 验收轮

- 日期：2026-10-12 ｜ Goal：goal-a8a36131（revision 2）

## ① t5（git 基线）验收 ✅ **通过（5/5 独立复现）**

| 验收项 | 结果 | 证据 |
|---|---|---|
| SHA256SUMS 内容+校验 | ✅ | `sha256sum -c` → 成功，值与 round-004 基准逐字一致 |
| .gitignore 命中 | ✅ | `git check-ignore -v` 逐项命中（*.nes@L6 / shots/@L12 / analysis/@L9 / mesen2.zip@L18）；docs/rounds/ 正确地未被忽略 |
| ROM 未入库 | ✅ | `git ls-files | grep .nes` 为空（SHA256SUMS 跟踪模式，无需转用户） |
| 提交规范 | ✅ | 94fd3c5，type=chore，正文说明基线内容/影响 |
| 跟踪清单 | ✅ | 22 文件（AGENTS/docs/prompts/gates README/tools 源脚本），无二进制混入 |

成员附加报告质量高：主动标注 hooks SKIP 缺口、主动发现 chr_render.py 路径问题、ROM 模式决策留痕。

## ② 本轮 findings（captain 验收产出）

| # | 发现 | 分类 | 处置 |
|---|---|---|---|
| F-01 | `analysis/chr_render.py`（源码性质）随 analysis/ 忽略规则丢失跟踪 —— 与 da6502.py 同类路径问题第二次出现 | 约定缺口 | 已裁定移入 tools/（路径契约已是 t2 验收检查项）；**建议把"可复用工具入 tools/"写进 AGENTS.md §2 红线表**（纪律变更 → 待用户确认） |
| F-02 | 提交 scope `repo` 不在 AGENTS.md §9 scope 清单（rom/ref/oracle/data/core/platform/gates/docs/tools） | 纪律文档缺口 | 低危；建议 §9 增补 `repo`（housekeeping 提交需要合法 scope）→ **待用户确认** |
| F-03 | tools/test.lua（早期探针脚本）被误跟踪 | 卫生 | 低危；下次触碰 tools/ 时清理或标注用途 |

## ③ 任务看板

| 任务 | 状态 | 验收 |
|---|---|---|
| t1 完备性矩阵 v0 | ✅ completed | ✅ round-005（证据抽查 6/6） |
| t5 git 基线 | ✅ completed | ✅ 本轮（5/5） |
| t2 ROM 取证 | 🔄 in_progress | 待（对照 round-004 基准表） |
| t3 门禁骨架 | 🔄 claimed | 待（verify 三命令） |
| t4 评审 t3 | ⏸️ 暂停待 t3 | verifier 取证已备（8 子命令/11 门禁/4 场景），SHA256SUMS 腿已独立复核闭合 |
| **t6 现有证据规格化（新派）** | 📤 pending → spec-writer | 待 |

## ④ 进度检查（矩阵约束③口径）

- 各面格数：行为/内容面 44｜规则面 44（下限）｜符号/数据面未计
- 强度分布：44 格全 L0；t6 完成后预期 B-SYS-SHOP/MONEY/UI 三格升 L1（规则面首批）
- 下一最优格：维持 round-005 判定 —— gameplay 证据（S2）+ 投影（S1）解锁分母增长
- **完整循环计数：2/3**（t1、t5 各走完"派→执行→交付→独立验收"闭环）

## ⑤ 下轮计划

1. t2/t3 落盘即验（t2 五项取证 + chr_render.py 路径合规；t3 verify 三命令实贴）
2. t4 唤醒裁定（t3 completed 后）
3. 空出人手即派：t7 S2 gameplay 场景捕获（decomp-analyst）、t8 S1 双投影
