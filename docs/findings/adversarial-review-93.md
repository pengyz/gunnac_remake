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

---

## 段 B：②快进环 Wt-1 红线 + ③两时序域混域哨兵 + ④表分发转活

### B.1 ② 快进环 Wt-1 红线——**×8 突变咬 ✓ / 跳拍突变盲区（F-B93）**

实现 `fast_forward_ring`：`for t in 1..=ticks { cursor.step_to(t) }`（每迭代=1 全链 tick）+
doc 红线"**禁止跨关换算常数**（Wt-1）"在案 ✓。

| 突变 | 实测 | 裁定 |
|---|---|---|
| M2a **×8 帧域混算**（`ticks*8`——Wt-1 突变原款） | ✅ **2 测红**（`fast_forward_and_wave_tick_wiring`+`stg01_data_load_tick_anchors`） | 咬 |
| M2b **跳拍**（`step_by(2)`——每 2 tick 步进一拍） | ⚠️ **21 测全绿** | **盲区（F-B93）** |

**F-B93（必修，1 行级）**："每迭代=1 全链 tick（不跳拍）"声明无哨兵可咬——STG-01 脚本流在
tick 1..=17 内**仅 tick=1 有记录**（rec 触发 1/0xC0/0xD8…），跳偶数拍的中间 tick 效果不可见；
合成哨兵（crafted SchedCursor 挂触发 tick=2 的记录 → 断言其在环内 fire）即咬。
**与 F-B4/C88/C89/C90 同族第 5 例**（样本值域窄一格——本例=tick 值域窄）。
注：M2a 能咬说明"17 tick 总量"面已锁，缺口在**逐拍中间步进**面。

### B.2 ③ 两时序域分流（混域哨兵）——**双向咬合** ✅
`route_spawn_item`：Pair→`TickAtomic`（fx.spawn_events 直投同帧）/Spawn→`FrameDrip`
（fx.drip_events 帧滴灌）——与 #84 两时序域红线（delay=tick 域/$534=帧域）接线一致；
测试③ 三断言（直投落 spawn 通道/滴灌独立通道/spawn≠drip 混域必红）。

| 突变 | 实测 |
|---|---|
| M3a 滴灌误入直投道 | ✅ 1 红（`spawn_timing_route_two_domains`） |
| M3b 分类翻转（直投→滴灌） | ✅ 1 红 |

**双向咬合 ✓**——混域哨兵判别力完整。

### B.3 ④ 表分发转活+验证对——**4/4 咬合** ✅
`TYPE_DISPATCH[0x2B].target=0x90AB`（→$9120 trace 吻合）/`0x38→0xB4A3`/`0x51→0xB00A`/
`0x56→0xB289`——**4 验证对**（lead 称 3 对，实测面 4 对=覆盖更全）逐值断言在案；
**M4 目标错位突变（0x90AB→0x90AA）→ `type_dispatch_table_wiring` 红** ✓ 表内容被锁。
"转活"面（type_bank_switch 激活）同测覆盖 ✓。

---

## 段 C：⑤战斗命中组装 + ⑥⑬⑭ 双断言 mutation

### C.1 ⑤ combat_frame 命中组装（b8186fd）——**组装面=已证原语装配，无新发明** ✅
六步组装逐项映射已证真值：①弹推进 LUT 运动机（#87 shot 面）②7×10 注册+attr $30
（#87 段C 注册 golden 同构）③敌框形取框（3b64b18：shape_stream_addr+f518 形索引+
f534 bit6/7 翻转+shape_data 流）④hit_resolves 4 边含端点+attr 相交（#87 M12b/M12c 咬合面）
⑤弹命中 HP−2=`$D688` 弹路（#87 段C.1 listing `SBC #2` ✓）+弹回收 $87FD+一框一伤
（#87 段C.2 三写集 ✓）⑥HP 归 0→$D616 死亡门语义（**爆炸/释放链 fail-closed 不发明** ✓ 边界诚实）。
测试⑫ golden：**14→12 实测锚**（#82 trace 中 boss 真值）+7 发致死槽回收+远弹零伤+
属性过滤（$30×$0F 几何叠也无伤）——判定面与 #82/#87 实测锚**同源咬合** ✓。

### C.2 ⑥ 用例⑬⑭ 双断言（tick $0330/$04D0）mutation——**双咬** ✅

| 突变 | 实测 | 裁定 |
|---|---|---|
| M6a 锚点漂移（STG01_SCRIPT 0x0330→0x0332） | ✅ `stg01_natural_run_to_stage_gate` 红 | 咬（$0330 锚被锁） |
| M6b 等值改不等式（`==tick`→`<=tick`） | ✅ 同测红 | 咬（**精确相等**语义被锁——与 81A3 CMP 真值同构） |

还原后 engine_tick **22/22 复绿**。⑬⑭ 双点（op5 双流装载 0x330/0x4D0）+自然跑完成判据
（$26 bit4 臂→flow 消费）断言链判别力完整。
