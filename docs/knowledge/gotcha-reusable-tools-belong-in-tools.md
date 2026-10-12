---
type: gotcha
title: 可复用工具必须落 tools/，analysis/ 只放可再生产出——同类路径坑已踩两次
date: 2026-10-12
status: confirmed
---

# gotcha：可复用工具入 tools/（入库），analysis/ 只放可再生产出

## 现象

同一类路径契约错误在项目早期**发生了两次**，都是"源码性质的可复用工具被放进 `analysis/`"：

1. `da6502.py`（表驱动 6502 反汇编器，64 行）落在 `analysis/tools/`——round-003 巡检发现，
   round-004 确认已迁入 `tools/`（"round-003 的路径指引生效，10 分钟内新落盘"）。
2. `chr_render.py`（CHR 渲染脚本）落在 `analysis/`——round-006 F-01 登记为**同类路径问题第二次出现**，
   裁定移入 `tools/`，round-007 §② 确认 `tools/chr_render.py` 已就位。

两次的后果相同：`analysis/` 是不入库的可再生工作区（git 基线 94fd3c5 的 `.gitignore` 忽略 `analysis/`），
源码放进去 = **随忽略规则静默丢失跟踪**，工具本身也无法被门禁/他人复用。

## 为什么

AGENTS.md §2 红线表把仓库切成两类：

- **入库**：`tools/`（工具源码）、`gates/`、`docs/`、`prompts/` 等——是团队资产；
- **不入库**：`analysis/`（IDA/Ghidra 投影、trace 导出、CHR 渲染等**可再生工作区**）。

"可再生"指的是**产物**（跑一次脚本就有的 png/trace/listing），不是**生成器本身**。
判断标准一句话：**删掉它还能不能重来？** 能 → 可再生产物 → `analysis/`；
不能（需要人写出来）→ 源码资产 → `tools/`。

## 何时使用

- 写任何脚本/工具/生成器落盘前，先答"这是产物还是生成器？是长期复用还是一次性？"
  长期复用的生成器 → `tools/`；一次性/可再生产物 → `analysis/`。
- 验收他人产出时把"路径契约符合"当检查项（round-006 起已成为 t2 验收检查项）。
- 见到 `analysis/` 下出现 `.py`/`.lua`/可执行脚本 → 立即按本条处理（迁入 `tools/` 并留痕）。
- 备注：把"可复用工具入 tools/"写进 AGENTS.md §2 红线表属**纪律变更，待用户确认**
  （round-006 F-01）；未确认前按本条执行。

## 证据指针

- `docs/rounds/round-003.md` §②（问题发现与处置：da6502.py 落 analysis/tools/）
- `docs/rounds/round-004.md` §①（迁入 tools/ 生效验证）
- `docs/rounds/round-006.md` §② F-01（chr_render.py，同类问题第二次；裁定移入 tools/）
- `docs/rounds/round-007.md` §②（`tools/chr_render.py` 已移入 ✓）
- commit `94fd3c5`（git 基线：`.gitignore` 忽略 `analysis/`，`tools/*.lua`、`tools/da6502.py` 入库）
