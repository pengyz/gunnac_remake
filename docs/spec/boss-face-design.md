# Boss 行为面接口设计稿（考证面先行——lead ③预备令）

> 日期：2026-10-16｜角色：porter（接口设计稿——未证不发明；考证面=取证优先级清单）。
> 背景：W-BOSSID 绑定（待 decomp）后接 boss 行为 VM；**E-W07 终 boss 多相=完成度最大未知**。

## 1. 接口三层设计（game-core 纯函数面——bot/引擎双消费）

### 1.1 BossDmgPath（boss 伤害路——与 HitDmgPath 同族但 boss 特化）
- **现状已证**：boss HP=RAM `$0630+槽`（表[$4D|$4C] 每敌型一表，bank04-07 ~45 点）；
  击杀判定=HP 归 0→`$D616` 死亡门（同杂兵）。
- **待证（考证面①）**：boss 是否走 `auto_hit_path` 三路同选（$6E bit7 开关）？或 boss
  专属减伤/相位护甲（多相期伤害系数变化）？——**E-W07 多相相位切换时伤害系数**=关键未知。
- 接口草案：`boss_damage_apply(hp, hit_path, phase_coeff: Option<u8>) -> u8`
  （`phase_coeff=None`=无相位系数（现证面）；Some=多相期系数（待证参数化））。

### 1.2 Boss 行为 VM 接口（行为面——behavior-vm 同族）
- **现状已证**：boss 生成=杂兵清零后（Soig §4 互证）；boss_type 事件面（v5 trace 98 例
  =入场/退场对）；boss spawn 血量直采（−100/−90/−80 族=be912bf W-BOSSUNIT 裁点弹药）。
- **待证（考证面②）**：boss 行为脚本载体（波次流 type≥$30 族？独立表？）；多相切换
  判据（HP 阈值表/固定时序/两者）；相位间行为差异（弹幕模式/移动模式切换）。
- 接口草案：`boss_behavior_step(slot, phase) -> BossEffect`（phase 由 HP/时序驱动——
  判据未证前参数化 fail-closed）。

### 1.3 W-BOSSUNIT/W-BOSSID 绑定面（单位/身份）
- **现状**：击杀数单位（发/秒）未裁；BOSSID=敌型表索引（$4D|$4C 双角色已证）。
- **待证（考证面③）**：Soig sheet2「As usual/2/-」三值语义=常规公式/2 发/不可用的
  **单位换算**（发=单发伤害计数？秒=DPS 计时？）——对照面=实测 hp_delta 事件计数
  （be912bf 方法复用）。

## 2. E-W07 终 boss 多相考证面（最大未知——优先级清单）

| # | 取证项 | 方法 | 阻塞面 |
|---|---|---|---|
| 1 | 多相相位数+切换判据 | Mesen2 插桩（HP 写点+相位旗 trace） | W-BOSSID 绑定 |
| 2 | 相位伤害系数 | hp_delta 逐相位统计（be912bf 方法） | #1 |
| 3 | 相位行为差异 | 弹幕 spawn 模式对比（wave 流 type 族） | #1 |
| 4 | 击杀数单位 | Soig sheet2 vs 实测事件计数对照 | W-BOSSUNIT |
| 5 | 终 boss 结局链衔接 | $180==9 → $95F9 与终 boss 死亡时序 | OBS-1/结局段 |

## 3. fail-closed 边界
- 相位判据/伤害系数未证前：`phase_coeff=None`（无系数）+`boss_behavior_step` 单相直通；
- 多相状态机不发明（E-W07 取证后按 ①②③ 表逐项实装）；
- 击杀数单位未裁前：boss_hits_to_kill 维持「发」口径（hp/damage 整除）+单位注。
