---
type: gotcha
title: 并发 Mesen 共写输出文件=幻影输入/假 trace——oracle 运行必须串行+归档隔离
date: 2026-10-12
status: confirmed
---

# gotcha：并发 Mesen 共写输出文件 → 幻影输入 / 假 trace

## 现象

t25 在 akey-role 场景观测到"无排程的方向位"，据此提出 **W8 假设**：实机开场存在游戏内虚拟输入
序列写入输入 RAM。t28 干净复核**证伪**：确定性驱动（防踩踏三件套）串行重跑 gameplay-stage1 /
weapon-pickup，1091+1081 帧全帧比对 **0 个计划外按键位**（仅 ±1 帧注入延迟边界）——
所谓"虚拟输入"实为**孤儿 run 干扰**：t12/t22 时代早于防踩踏的旧 Mesen run 与新 run
**共写同一输出文件**（`oracle/traces/*.jsonl`、`shots/*.png`），trace 变成两个模拟器的混合体。
t28 追踪定位孤儿产物（exp-shop-*/title-menu/shop/t25 三场景/t22 时代 gameplay-stage1），
现行驱动重跑已自动归档（`.archive/gameplay-stage1-20261012-155826` 等）。

## 为什么

- 同名输出文件**无实例隔离**时，并发 run 的写入互相覆盖/交错：读到的"逐帧 trace"可能是
  两个模拟器各自时间线的拼接——输入位被另一 run 的输入污染（**幻影输入**），帧序错位产生**假 trace**。
- 与 [gotcha-parallel-read-image-order-unreliable.md](gotcha-parallel-read-image-order-unreliable.md)
  同族：**并行不可信家族**——一个是读取端顺序不可信，一个是写入端隔离不可信；
  两者的共同解药都是"字节级锚定 + 单一事实源"。
- 症状具有欺骗性：幻影输入看起来像游戏行为（"游戏内虚拟输入序列"是个诱人的解释），
  只有**全帧期望比对**（expect(hold 段) vs 实测 controller_held）才能分辨外来位。

## 何时使用

- 任何 Mesen/oracle 场景运行前确认**防踩踏三件套**在位（`oracle/scenarios/README.md`）：
  **串行锁** `analysis/.scenario.lock` + **覆盖前时间戳归档** `.archive/<名>-<ts>/` + **自管 Xvfb**。
- 引用 trace 结论前核对产物时间戳与 `.archive/` 是否存在孤儿产物；t12/t22 时代旧 run 产物
  一律按 t28 复核口径重验后再引用（其 f1830+ 结论已 bit 级重验通过可作先例）。
- 观测到"计划外输入位/不可能的状态跳变"时，**先怀疑并发污染**（查锁/归档/孤儿 run），
  再怀疑游戏逻辑；证伪路径 = 干净串行 run 全帧期望比对。
- 反例（本条教训来源）：t25 依 akey-role 单 run 现象立 W8"游戏内虚拟输入"假设。

## 证据指针

- `docs/findings/orphan-run-recheck.md` §2.1（W8 证伪：1091+1081 帧 0 计划外按键位；孤儿产物清单）
- `docs/findings/ram-watch-s3.md` §9.5（【2026-10-12 更正（t28）】原地加注）
- `oracle/scenarios/README.md`（运行纪律：串行锁 + 覆盖前归档 + 自管 Xvfb）
- t28 任务输出（孤儿 run 分岔复核：t22 f1830+ 结论逐条 bit 级复核通过）
