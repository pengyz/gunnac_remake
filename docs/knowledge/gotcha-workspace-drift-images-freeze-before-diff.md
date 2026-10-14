---
type: gotcha
title: L4 比对禁读工作区漂移图——冻结 A/B 或新鲜渲染立即测
date: 2026-10-12
status: confirmed
---

# gotcha：工作区 `actual/` 漂移图污染（比对必须冻结态/新鲜渲染）

> 并行不可信家族**第 4 员**（并发产物污染系）：与 [gotcha-parallel-read-image-order-unreliable.md](gotcha-parallel-read-image-order-unreliable.md)
> （读图序）、[gotcha-concurrent-mesen-output-corruption.md](gotcha-concurrent-mesen-output-corruption.md)
> （Mesen 共写输出）、[gotcha-git-add-commit-atomicity.md](gotcha-git-add-commit-atomicity.md)
> （git 索引卷入）同族——**共享工件被并行写者持续覆写**，读到的不是你以为的那一份。

## 现象

t189 复审活例：L4 像素比对首轮读工作区 `oracle/screens/actual/` 图得 **95.32%**，
按冻结 A/B 重测得 **98.07%**——2.75 个百分点纯属**图被换了**。历次 L4 数字微差
（各重试间小幅抖动）以此为**统一解释候选**：`actual/` 目录被并发实验持续覆写，
比对器读到的渲染帧与当次运行不对应。

## 为什么

- `oracle/screens/actual/` 是**共享工作区**：多个并行实验/重试各自渲染写入同名文件，
  后写覆盖先写（与 Mesen 共写输出同构）。
- 比对数字的前提是"被比的两张图=同一次运行的产出"——工作区漂移图违反此前提，
  差异混合了**真像素差 + 图版本差**，数字不可归因。

## 何时使用

- **L4 比对前**：①冻结 A/B（把双方渲染拷入不可变快照目录再比）或 ②新鲜渲染后**立即**
  测量（渲染与比对同进程/紧邻，中间不留他人写入窗口）。
- **禁读**工作区 `actual/` 漂移图做结论性数字；数字进 findings/spec 时标注比对模式
  （冻结 A/B / 新鲜即测）与快照指针。
- 历史数字复核先问：当时比的是哪份图？无快照指针的 L4 百分比降级为参考值。
- 读图序问题另见并行家族第 1 员（同图多张也有顺序坑）。

## 证据指针

- t189 O1 测量纪律（首轮 95.32% vs 冻结 A/B 98.07% 对照实录）
- `oracle/screens/actual/` 并发覆写现象（多实验共用渲染输出路径）
- 家族三员互链（见文首）
