---
type: decision
title: L4 判据回到 0 容差（白名单外像素一致）——"带声明签署"不构成 L4 达成
date: 2026-10-18
status: confirmed
---

## 现象

仓库里长期并存两套 L4 判据，且结论相反：

- **规划/门禁判据**（[l4-visual-plan.md](../spec/l4-visual-plan.md) §1、[master-plan.md](../master-plan.md) §6）：
  "白名单外像素 100% 一致（0 容差默认）"；
- **签署判据**（[l4-signing-criteria.md](../spec/l4-signing-criteria.md)）：阈值判定式
  （`diff 4552<7680`、ending 五帧"转正 5/5"）+ 差异分类账 + §4 缺口声明，并标"L4 100% 签署达成"。

2026-10-18 用户裁定：**Stage A 的 L4 判据回到 0 容差**；签署口径仅作过程里程碑，**不构成 L4 达成**。
依据实测：`oracle/screens/visual-report.md` 现行 22 对截图中仅 **1 对**逐像素相等（4.55%），
合计差 **280,132/1,351,680 = 20.72%**（同点 79.28%），白名单 `gates/visual_whitelist.json` 至今不存在。

## 为什么

- **裁判是差分测试，不是文档签字**（AGENTS.md §1）。阈值判定式允许单帧 12.5% 像素差且按帧各自基准，
  与"白名单外像素一致"不是同一件事；把过程签署当完成，会让 DoD 失真、后续工作失去靶心。
- **口径分裂会污染下游判断**：审计方独立复算后发现同一签署文件内"对位率"存在 43.0%/55%/61% 三值，
  且 2026-10-16 "NOT READY（0 容差远）"与 2026-10-18 "100% 收官"之间缺裁定记录。
- 用户已明确：残余缺口（调色板映射 W-C1、nametable 持久化模型等）**不接受"带声明封存"**，转 P0 攻坚。

## 何时使用

- 任何 Stage A 进度汇报、DoD 核对、里程碑签署前：L4 一律按 0 容差判，**未达成**。
- 引用历史签署材料时：必须带"过程里程碑/非 L4 达成"限定，不得复述"L4 已完成/像素级一致"。
- 后续 L4 攻坚排期：调色板映射 W-C1 + nametable 持久化 + 族映射 + 3B 表残余 + stg02-start 回归为 P0。

## 证据指针

- 裁定记录：`docs/spec/l4-signing-criteria.md` 顶部【2026-10-18 用户裁定·Lead 记录】；
  `docs/master-plan.md` §6 同条注（两处为原地标注，不删历史）。
- 实测数字与复现命令：`docs/findings/rewrite-completion-audit.md` §4（C5 命令）、
  `oracle/screens/visual-report.md`（22 对/1 zero/280,132 差/20.72%）。
- 判据原文：`docs/spec/l4-visual-plan.md` §1；`docs/master-plan.md` §6。
- 签署口径原文（历史）：`docs/spec/l4-signing-criteria.md` §1/§4/§6。
