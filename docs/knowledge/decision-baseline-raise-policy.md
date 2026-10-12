---
type: decision
title: 基线政策分级：进度类上调由 captain 验收后代执行，下调/放宽/白名单仍须用户确认
date: 2026-10-12
status: confirmed
---

# decision：基线调整两类分级（2026-10-12 用户裁定）

## 现象

t9 建立知识库后 `ratchet:knowledge_entries` 出现 PENDING WARN（基线 0 / 现值 5）。
按旧口径"基线上调须用户确认"，成员上报并停工等待。用户裁定（2026-10-12）把基线动作**分级**：

- **进度类上调**（调到现值 + `--reason` 写明"确认无回退"证据 + commit 说明）
  → 由 **captain 验收后代执行**，无需逐次人工确认；
- **下调 / 放宽 / 白名单** → 仍必须用户确认。

首个执行案例：`knowledge_entries 0→5`（t9 验收后代执行，commit `5fc2533`）；
`docs/master-plan.md` §5.5 已原地修订留痕。

## 为什么

- 基线动作**风险不对称**：上调到现值是机械校准（记录进度；ratchet"只升不降"本就防回退），
  而下调/放宽/白名单是松开防线、掩盖差异的动作——只有后者需要用户闸门。
- 逐次人工确认进度类上调 = 流程空转源；"captain 验收后执行"保留了验收把关 + commit 留痕两道关，
  既不空转也不失守。

## 何时使用

- 遇到 ratchet **PENDING WARN**（现值高于基线）：**直接上报 captain，不停工等待**；
  由 captain 验收对应产出后代执行 `baseline update --reason` + commit 说明。
- **下调基线、放宽阈值、加白名单、改门禁语义**：仍必须用户确认（AGENTS.md §5、master-plan §5.5）。
- 反例：成员自行跑 `baseline update`（即使是上调）——执行权在 captain；成员只负责上报。

## 证据指针

- `docs/master-plan.md` §5.5（"【2026-10-12 修订·用户裁定】"原地留痕）
- commit `5fc2533`（`knowledge_entries` 基线 0→5 上调 + §5.5 修订——首个执行案例）
- `gates/cli.py check` 输出：`ratchet:knowledge_entries 基线 5 / 现值 5 —— OK`
- captain 回执（2026-10-12：t9 验收通过 + 基线政策通报，"遇 PENDING WARN 直接上报 captain"）
