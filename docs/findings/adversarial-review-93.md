# 对抗复核 #93 — 集成主干七子面（eda5769…7eb2123，STG-01 完成判据）

> 日期：2026-10-15｜角色：adversarial-reviewer（证伪立场；实施方=port-engineer≠审核方）
> 被审：集成主干七子面 `eda5769` 装载→`5bba977` spawn→`ed1c09a` 调用环→`98d3fac` 表分发→
> `3b64b18` 框形→`b8186fd` combat→`7eb2123` 关末门判据（骨架 `a17c7b5`）。
> 七证伪靶：①关末门全链逐环 ②快进环 Wt-1 红线 ③两时序域混域哨兵 ④表分发转活 ⑤战斗命中组装
> ⑥用例⑬⑭ 双断言 mutation ⑦对拍账目。拆段落盘。**STG-01 判据=引擎层关 1 可推进至关末门**。

## 段 A：①关末门全链逐环独立重导——**七环全通** ✅

### A.1 链环逐环裁定（滚动→tick→thr→op5→CA bit0→$26 bit4→flow）

| 环 | 实现落点 | 独立重导 | 裁定 |
|---|---|---|---|
| 滚动→tick | `stage_tick`：scroll/16 进 tick | 测试"scroll=0x04D0×16 ⇒ tick=0x04D0"；Wt-2（#90 已证 16px/tick） | ✅ |
| tick→thr **精确相等** | `step_to`：`while threshold==tick` **逐 tick 中间步进不跳拍** | 与 81A3-81AD 双字节 CMP 等值语义（#90 逐指令）同构 ✓ | ✅ |
| thr→op5 装载 | 尾 thr 链 `threshold:=rec.thr`（82EF 语义） | **0x330/0x4D0 双锚决定性验证**（见 A.2） | ✅ |
| op5→CA bit0 | `register_wave_flags`（WAVE_FLAGS 表填 c9/ca）→ `process_waves` | ca_judgment（#88 已验）复用 ✓ | ✅ |
| CA bit0→$26 bit4 | `flags_26 |= 0x10`（arm_stage_end_gate ⇒ 关末门臂） | 与 8D02-8D0A"$26\|=关末门+$D1:=0 同支"（#88）一致 | ✅ |
| $26 bit4→flow 消费 | 测试④ `flow_consumes_stage_end_gate`：`fx.stage_clear_requested` | ⑬ line 373-375 stage_tick 主循环消费断言 | ✅ |

### A.2 thr 语义决定性裁定（尾 thr——0x330/0x4D0 双锚）
`wave_timeline.py script 1` 真值 14 记录与 `ScriptCursor::stg01()` 嵌码**逐值全等**；
**op5 双流装载点**：测试⑬ 驱动 t=0..=0x330 断言 `$A3CD` 装载（tick $0330）、t=0x331..=0x4D0 断言
`$A3E4` 装载（tick $04D0）——恰为 rec8/rec10 的**尾 thr**（0x0330/0x04D0）⇒
`ScriptRecord.thr`=**记录尾 thr=下一记录触发 tick**（B1 §1.4 口径）✓✓ 机械模拟逐步吻合：
boot 簇 rec1-5@tick0 → 0xC0/0xD8/0x120 → **0x330 op5 ✓** → 0x4C8 → **0x4D0 op5 ✓** → 0x4DA/0x4E8。
（字段 doc"触发 tick"表述=下一记录触发，与头注"本记录触发=前一记录尾 thr"自洽。）

**流 quirk 登记**：rec13 尾 thr=$0000 ⇒ rec14（op7 jump-if-wave-busy）在严格等值下**不可达**
（0x4E8 后 threshold=0，tick 单调增不返 0）——**ROM 语义同不可达**（82EF 装 0 后 81A3 永不等），
实现忠实保留 quirk ✓（与 dropper §0"op7 正常流程不可达"结论同向，实例不同：彼为 $FFFF 哨兵
波次流、此为 $0000 尾 thr 脚本流）。

### A.3 ① 结论
**七环全通、每环与既证 listing 真值（#88/#90/#84）同构**；"STG-01 判据=引擎层关 1 可推进至关末门"
的链路基础成立（完成判据的实测面=测试⑬⑭，段 C mutation 试咬）。
