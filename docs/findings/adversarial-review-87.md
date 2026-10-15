# 对抗复核 #87 — port-engineer #84 修订 + 批次 3（9903367 + db90827）

> 日期：2026-10-15｜角色：adversarial-reviewer（证伪立场；实施方=port-engineer≠审核方）
> 被审：`9903367`（#84 两必修收口：F-B4 合成交叉哨兵 + bank8_window 注释）+
> `db90827`（批次 3：BulletGrid 六数组 SoA + 槽位交叠裁定 + 双分配器 + 命中框 7×10 + 伤害公式）。
> 六证伪靶（lead 转被审自带）：①交叠 8 槽逐值 ②伤害物撞=$0630,Y 对方 ③一框一伤仅清三字段
> ④敌弹分配器组跳算术 ⑤F-B4 mutation-kill 三态 ⑥新哨兵 ⑩⑪⑫ 试咬。
> 纪律：mutation 即做即还原（共享树）。拆段落盘，每段一 commit。

## 段 A：⑤F-B4 收口复验 + ⑥新哨兵 ⑩⑪⑫ mutation 试咬——**7/7 全咬** ✅

### A.1 ⑤ F-B4 mutation-kill 三态（#84 必修复验）

新用例 `step_criterion_is_type_bit_not_ctrl_synthetic`（9903367）：合成交叉对
`Spawn{ctrl:1,type_:0x2F}`⇒step=true、反向 `Spawn{ctrl:2,type_:0x2E}`⇒step=false
（且 write_48c 仍 Some——码流读判据=ctrl，与 step 判据分离 ✓ 语义正确）。

| 态 | 操作 | 实测 |
|---|---|---|
| 突变 | `code_stream_step: type_&1` → `ctrl==2\|\|ctrl==12`（#84 M4 原突变） | —— |
| **杀** | 跑 `step_criterion` 用例 | ✅ **FAILED 精准咬合**（#84 时该突变 9 测全绿的盲区已闭） |
| 还原 | `git checkout` | ✅ 零残留，wave_sched **10/10 复绿** |

**F-B4 正式收口确认** ✓（mutation-kill 三态完整）。

### A.2 ⑥ 新哨兵 ⑩⑪⑫ 逐个试咬（即做即还原）

| # | 突变 | 目标哨兵 | 实测 |
|---|---|---|---|
| M10a | `SHARED_SLOTS` 表内 0x84→0x85 | ⑩ | ✅ 红 |
| M10b | `ENEMY_SHOT_TYPE` 0x80→0x7F（去 bit7） | ⑩ | ✅ 红 |
| M11 | 敌弹组距 `GROUPS[1]` 0x9F→0x9D | ⑪ | ✅ 红 |
| M12a | 命中框 `x−4`→`x−3` | ⑫ 注册 golden | ✅ 红 |
| M12b | `boxes_overlap` ≥→>（端点开区间） | ⑫ 端点相接断言 | ✅ 红 |
| M12c | `clear_hit_entry` 误清第 4 字段 owner | ⑫ 仅清三字段 | ✅ 红 |
| M12d | `damage_apply` 弹伤 2→1 | ⑫ 伤害 golden | ✅ 红 |

**7/7 全咬**（含 ⑤）；全部 `git checkout` 还原后 `stage_content 15/15 + wave_sched 10/10` 复绿、
`git status crates/` 干净。哨兵面覆盖声明（⑩⑪⑫ 各锁其口径）**成立**。
