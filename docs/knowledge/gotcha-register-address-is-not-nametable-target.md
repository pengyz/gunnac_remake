---
type: gotcha
title: 注册器地址≠nametable 目标——PPU $2006 锁存语义（十一击共同暗雷）
date: 2026-10-12
status: confirmed
---

# gotcha：注册器地址 ≠ nametable 目标（PPU `$2006/$2007` 协议暗雷）

> 【2026-10-12 t159 升级转正】~~候选注~~ **正式条目**：前置已齐——四自由度注（14 位掩码/
> 自增量/时序相位/读自增）+ 影响面收窄（port 消费侧）+ **t154/t158 链 pass**（adversarial-review-10
> 独立验证成立）；升级判定完成。
> 【2026-10-12 t142 升级前置注（AR3-D/AR3-E）】【t146 (a) 错位修正（R2-8）】**(a) 协议自由度注**
>（~~两自由度=14 位掩码+自增量~~ 补全为**四项独立自由度**）：**14 位掩码**（`ORA #3`/bit7 flag
> 坐实——地址只取低 14 位）+ **自增量**（`$2000` bit2 决定 1/32）+ **时序相位**（hi→lo 锁存顺序）
> + **读自增/读缓冲**（读路径独立行为）——落点重导须四项同时给定；AR3-C 四缺口当前数据集
> 零影响 **latent**，port 实现须全量补齐。**(b) 措辞收窄（AR3-E）**："影响面=十一击数据集消费侧"
> 收窄为 **"影响面=port 消费侧"**（t128/t130 数据侧解读本就基于锁存语义、不波及）。
> **升级条件**：t141 修复重跑 + 复核 verdict=pass 后升正（现在维持候选注）。

## 现象

L4 内容帧收敛**连续十一击**（t102..t123）在同一暗雷上绊倒：把 **`$2006`/`$2007` 的注册器地址**
当成 **nametable 落点目标**直接解读数据集——真实语义是 **`$2006` 高/低字节两次写锁存 VRAM 地址、
`$2007` 写后自增**。数据集里的"地址"是协议时序流，不是最终落点；不解释协议就重释落点 =
**port 消费侧**结论错位（文本/BG 放置层差、逐址比对不收敛；t128/t130 数据侧解读本就基于锁存语义
不受波及——AR3-E 收窄）。t131 按协议重解释后重建**曾报逐址精确**（~~inv0 `[74,74,74,83]` 与 t130
逐字节一致~~【AR3-A 假阳性更正】抽查面过窄巧合，待 t141 重跑）。

## 为什么

- NES PPU 是**寄存器协议机**：地址经 `$2006` 两字节锁存进入内部地址寄存器，`$2007` 访问后
  地址自动递增（步长 1/32 由 `$2000` bit2 决定）；读还有缓冲副作用（首读陈旧值）。
- 日志/probe 记下的 `$2006/$2007` 写值是**协议字节流**——把字节流里的地址字节当落点地址，
  等于把"寄存器操作序列"误读为"内存布局"。
- 与 [gotcha-observation-granularity-fabricates-invariants.md](gotcha-observation-granularity-fabricates-invariants.md)
  （时间轴网眼）、[gotcha-stack-page-game-var-watch-coverage.md](gotcha-stack-page-game-var-watch-coverage.md)
  （地址轴盲区）同族：**观测/解释层的缺陷伪装成数据问题**——本条是"协议层未解释"伪装成"数据错位"。
- 【t159 家族扩充】**投影/换算层缺陷**新员并入本家族：[gotcha-cpu-file-offset-conversion.md](gotcha-cpu-file-offset-conversion.md)
  （CPU↔file 换算漏 iNES 头）——"换算层错"伪装成"数据对不上"；配合
  [pattern-repair-selfcheck-grep.md](pattern-repair-selfcheck-grep.md)（修复自查）共同封口。

## 何时使用

- 消费任何 PPU 写日志/probe 数据集前：**先经 `$2006/$2007` 协议解释还原真实 VRAM 落点**，
  再做落点比对/断言。
- 写 L4/渲染断言时区分三元组：**注册器操作序列 → 内部地址状态 → 最终 VRAM/nametable 内容**；
  断言面在最后一层，证据链过前两层。
- 多轮收敛卡在同一层差且逐址比对不过时：优先怀疑**协议解释层**而非参数精度。
- W36 经验补充：Mesen2 `addMemoryCallback` 对 I/O 寄存器区无效，但 **CPU trace 双轨**
  可实测 `$2006/$2007`（t127 修正）。

## 证据指针

- t131 重试 #12 根修（commit 9d7fe3f：nametable 重建逐址精确）+ `docs/findings/stream-entry-r7.md` 相关节
- t127 PPU 写日志/`$2006/$2007` 回调 1,048 事件（CPU trace 双轨）
- NESdev Wiki：PPU registers（`$2006`/`$2007` 行为标准）
- 十一击数据集消费侧口径重释影响面清单（l4-visual-plan R-l4-linebuf-15 注，待复核 #2 校准）
