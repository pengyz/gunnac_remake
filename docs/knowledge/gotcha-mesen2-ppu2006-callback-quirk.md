---
type: gotcha
title: Mesen2 addMemoryCallback 对 $2006 锁存寄存器不触发
date: 2026-10-16
status: confirmed
---

# Mesen2 addMemoryCallback $2006 锁存寄存器不触发

> 【2026-10-20 更正】本文"$2006 锁存寄存器不触发（write-only latch quirk）"=**误诊**（原文保留作历史）：
> 真因是 `emu.addMemoryCallback(fn, "write")` 的**字符串**第二参被 Mesen2 2.1.1 **静默取整为 0（=read 回调）**，
> 历史 probe 日志**全部是读流**（变址存储可见=假读 dummy read）——下文"$2007 事件 1292 条"实为 `LDA $2007`
> **读**事件，非写点。补法=**数值枚举双流注册**（read=0/write=1，已落地）；双流注册后写点已可见：
> gameplay 实测 `$2005` **4,898** / `$2006` **25,686** / `$2007` **47,474** 条写（数字引
> [probe-setter-blindspot-fix.md](../findings/probe-setter-blindspot-fix.md) §3.3）。另：Mesen2 2.1.1
> **无 addBreakPoint API**（`emu.addBreakPoint=nil`）——下文绕行法 2 不可用（废止）。

## 现象

`emu.addMemoryCallback(fn, "write", 0x2006, 0x2007)` 注册成功（cb=true）——同一回调段内
**$2007 事件正常（1292 条）/$2006 零事件**（$2005 同组同零）；probe 写点 PPU 域 a 全谱仅 2007。
双组登记均成功：`{"probe":"2006-2007","cb":true}` + `{"probe":"2005-2006","cb":true}`。

## 为什么

$2006（PPUADDR）是写只锁存寄存器——Mesen2 内存写回调对锁存类 PPU 寄存器不触发（write-only
latch quirk）；$2007（PPUDATA）走数据口正常触发。回放侧若依赖 $2006 写事件重建锁存地址序
= 静默零源（四级漏斗 parsed→ppu(2006=/2007=)→latch_seq→projected 现形：2006=0 ⇒ latch_seq=0）。

## 何时使用

任何 PPU 锁存寄存器（$2005/$2006）写事件采集需求前——同坑必现。绕行三法：
1. **行缓冲推导**（主路）：$0130-0160 行缓冲写点含目的址字段（t131 4 字段+标志字节）——
   回放侧重建 nametable 目的址。
2. **PC 断点插桩**：对 $2006 STA 站点 emu.addBreakPoint 捕获锁存序列（定案捕获）。
   > 【2026-10-20 更正】本法**不可用**：Mesen2 2.1.1 无 addBreakPoint API（`emu.addBreakPoint=nil`，A）。
   > 现行补法=数值枚举双流回调（已落地，见文首更正），$2006/$2007 写点已直接可见。
3. **PPUDATA 反推**：$2007 写点 PC 分批+增量步进反推（需锁存初值）。

## 证据指针

- stg03-timeline probe 采集实证：`analysis/traces/stg03-timeline.probe.jsonl` a 全谱
  （PPU 域仅 "2007"×1292）——`python3 -c` 逐行解析计数（2026-10-16 实跑）。
- $2007 写点 PC 谱=4 站点×323：F17E/F181/F18C/F18F（NMI 上传批 STA 对）。
- 四级漏斗诊断行：`DBGPROBE=1 render-frame ... --probe-file ...`（parsed=598571
  ppu=1292 (2006=0 2007=1292) latch_seq=0 projected=0）——tools/diff_visual.py 报告面。
