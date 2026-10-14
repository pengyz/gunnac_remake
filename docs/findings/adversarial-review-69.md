# 对抗复核 #69：DEV-2 死亡/GAME OVER/Continue 接线（被审 af3cd89 + 64ef557）

> 复核人：独立对抗复核员（fresh context，证伪立场；非实施方）。
> 被审：`af3cd89`（feat core 11 文件）+ `64ef557`（ratchet 125→135）。
> 方法：不采信被审方自述——spec 逐条重导字节/trace 证据、mutation 独立重放（含新增）、
> 对拍三件独立复跑、缺口两写点字节直读裁定。证据分级依 AGENTS.md §3。
> **Verdict: needs_revision（限 1 项必修 AR69-1；三焦点全部通过；缺口裁定翻转 A2 §3.2 一处终态值）**

## 0. Verdict 与主张逐条裁定

| 被审主张 | 复核裁定 | 依据（本文件节） |
|---|---|---|
| 1. flow.rs 7 处 fail-closed 闭合（F1-F13；F13/直回标题/卖出保留 fail-closed） | **成立** | §2.4 grep 复核：F13/卖出/渲染零发明，catch-all 保留 |
| 2. 细粒度 DeathChainState/death_chain_step（F1-F12 全行为） | **成立**（除 F12 `$26` bit4 终态，见 AR69-1） | §2.1-2.3 + §3 |
| 3. 未证不发明（F13/$26 bit7/动画计时源/文本渲染暂挂） | **成立** | §2.4 |
| 4. death_chain.rs 10 用例 + AR68-1 + mutation 自检 3/3 | **成立**（mutation 我方 5/5 独立复现：3 重放全红 + 2 新全红） | §2.5 |
| 5. l2：death_money_halve 新增 + dmh-*/ed3-*×6 + CN/JP 双集 + l2-diff 121/121 双版 | **成立** | §2.6 |
| 6. spec R-* 13 条带证据 + R-death-bypass 暂挂 | **成立**（除 R-continue-reset `$26` 终态表述，见 AR69-1） | §2.1-2.3 |
| 7. E1BB 缺口上报（三处留痕，未接线） | **留痕属实；本复核裁定缺口：净效应 bit4=0，A2 表被部分证伪** | §3 |

## 1. 三焦点核验表

### 1.1 焦点一：未发明抽验（≥5，全部独立重导）

| # | 抽验项 | 独立重导方式 | 结果 |
|---|---|---|---|
| 1 | **F5 首帧口径**（"首帧=转移次帧"：f1787 四写→f1788 结算） | ①ROM 字节 bank03:A382-A3A2 逐指令复读（`ad 8d 01 d0 0b 20 56 ed a5 26 09 08 85 26 d0 08 a0 01 20 03 ed 8d 8d 01 a9 21 20 3b ed 4e 8e 01 20 4f cb`）：结算链（$18D 检查/终死 BNE 跳减一/LDY #1 JSR $ED03/STA $18D/音效 $21/LSR $18E）与 trampoline `JSR $CB4F` **同一次例程调用内先后执行**；②trace 逐帧直读 gameplay-stage1.frames.jsonl：f1787 `$041C=0x00`（trampoline 未装 ⇒ 本次调用未发生）、f1788 `$041C:$0438=$A3:$A5`（trampoline 已装 ⇒ 结算已跑）；f2046/f2047 同构双事件 ✓ | **成立（A 级）**。结算帧=转移次帧 ✓；转移帧不结算（f1787 无 trampoline）✓。注：`$018D` 2→1 仅 60f 采样（f1740/f1800）可见，帧内定位骑在 trampoline 锚上（A）+采样（B） |
| 2 | **F8 减半公式逐位** `(b&$C0)\|((b>>1)&$1F)` | ROM 字节 bank15:E3B9 起 `a2 05 bd 7a 01 48 29 c0 85 04 68 4a 29 1f 05 04 9d 7a 01 ca 10` = LDX#5/LDA $17A,X/PHA/AND #$C0/STA $04/PLA/LSR A/AND #$1F/ORA $04/STA $17A,X/DEX/BPL —— 逐指令对上公式与实现 `(*b&0xC0)\|((*b>>1)&0x1F)`；测试向量手算复核（3F→1F、21→10、C0→C0、FF→DF、00→00、60→50）全对 | **成立（A 级）** |
| 3 | **F12 复位集 vs A2 §3.2 逐项对账** | 字节直读：E0EF `a9 02 8d 8d 01`（$18D:=2 ✓）；E160 `a9 00 8d 89 01 8d 90 01 8d…`（$189/$190/$18E:=0 ✓）；E10A→sub_ECBF `a0 06 a9 00 99 e0 00 88 10 fa 60`（Y=6..0 STA $E0,Y ⇒ $E0-$E6 全 7 字节 ✓）；E18C 循环 `9d 00 04 9d 00 05 9d 00 06 9d fe 06 ca d0 f1`（X=0..255 四页，$6FE+255=$7FD，**$7FE-$7FF 不清** ✓ AD66-1）；E19B `85 c8`（$C8:=0 ✓）；E1B6 `a9 01 8d 00 04`（$0400:=1 ✓）；E1C1-E1C7（$27/$16/$18:=0 ✓）；E1D3-E1DD `a0 40 a5 26 29 80 f0 02 a0 ff 8c 88 01`（$188:=bit7?$FF:$40 ✓）；E0D3 `a2 05 9d 7a 01 ca 10 fa`（$17A-$17F 清零**仅标题路径**，续关带入减半值 ✓）；$180 全段无写点（保留 ✓） | **成立（A 级）**——但 `$26` 项终态被 §3 裁定证伪（AR69-1） |
| 4 | **F13/音效/渲染未发明** | grep 写集/字符串：`06A0/速爆/免死/0x32` 仅存于注释（3 处）；`卖出/sell` 仅 TODO 注释（flow.rs:7 + ui.rs:161-163）；`render/ppu/sprite/tile/palette/CHR` 在 flow.rs **零命中**；音效仅 DeathFx 挂起枚举（$21/$11/$45 三个已证 ID + $E4A0 文本指针），无音效写集落地；advance() catch-all `未证转移 … fail-closed` 保留 | **成立（A 级 grep）** |
| 5 | **F7 入场写集**（bonus） | bank15:E2BF 起 0x32 字节逐指令：`a5 27 09 40 c5 27 f0 1b 85 27`（$27 bit6 一次性 CMP 短路 ✓）/`a5 26 10 03 4c 73 e0`（bit7→JMP $E073 直回标题，**未接线** ✓ 与暂挂一致）/`a9 ff 8d 88 01`（$188:=$FF ✓）/`a9 00 8d 00 03`（$0300:=0 ✓）/`a9 11 20 3b ed`（音效 $11 ✓ 挂起）/`a5 16 29 10 … ad 00 03 d0 03 4c 1c e2 4c…`（Start/$0300≠0 双触发，否则 JMP $E21C 停留 ✓） | **成立（A 级）** |
| 6 | **F10 回标题**（bonus） | E09A `a9 01 8d 80 01`（$180:=1 ✓）+ E0D1 `a2 05 9d 7a 01 ca 10 fa`（$17A-$17F 6 字节清 ✓） | **成立（A 级）** |

**结论：未发明抽验 6/6 成立。** 未发现任何发明的写集/渲染/音效/F13 行为；暂挂项与 fail-closed 边与注释自洽。

### 1.2 焦点二：测试真验行为 + mutation 独立实测

**抽 3 条验 RAM 终态断言非自证**（断言值均独立来自 ROM 语义，非实现回读）：
- `f5_death_cost_settles_once_at_dead_first_frame`：lives 0x02→**0x01**（BCD −1）、shop_money 0x63→**0x31**（99→49 除半）、自向量 (0xA3,0xA5)、+200 步后不再扣（双向锁）——非自证 ✓
- `f7_gameover_fork_menu_entry`：entry_bytes `[0x3F,0x21,0xC0,0xFF,0x00,0x60]`→**`[0x1F,0x10,0xC0,0xDF,0x00,0x50]`**（6 字节具体减半值，我方手算复核全对）——非自证 ✓
- `f12_continue_reset_set`：17+ 项具体终态（双金钱清零/lives=2/网格哨兵 `$7FE/$7FF=0xAA` 保留/仅 stage=7 保留/bomb_ammo=6 保留）——非自证 ✓

**mutation 独立重放（5/5，含 2 新；每次改后跑 `cargo test -p game-core --test death_chain`，改后还原并 sha256 校验 `795c17b5…db2f2e` 逐次一致）**：

| # | 突变 | 被咬死（预期↔实测） | 结果 |
|---|---|---|---|
| M1（自检①重放） | 结算移窗末（settle 移入窗尽分支） | f5 首帧断言必红 ↔ **f5+f6+f4 红** | 红 ✓ |
| M2（自检②重放） | `max(4,·)` 改恒 4 | f4 ammo=7 保留分支必红 ↔ **f4 红** | 红 ✓ |
| M3（自检③重放） | 多清 `$7FE-$7FF`（`[..0x100]`→全清） | f12 哨兵必红 ↔ **f12 红** | 红 ✓ |
| M4（**新**） | 跳过减一（删 `bcd_sub_ammo(lives,1)`） | f5 lives=0x01 必红 ↔ **f5 红**（f6 过=终死路径本就不减，选择性正确） | 红 ✓ |
| M5（**新**） | F8 掩码位损（`&0x1F`→`&0x0F`） | f7 减半值必红 ↔ **f7+f12 红** | 红 ✓ |

**结论：审方自检 3/3 经我独立复放属实；新增 2 处亦全被咬死。测试面真实、断言非自证、无静默通过面。**
（附注：flow.rs 旧 `unproven_segments_fail_closed`→`gameover_continue_coarse_paths_wired` 一对一替换同数；
fail-closed 负向覆盖未丢——源态守卫 2 断言 + advance 未证边 2 断言 + death_chain `fail_closed_unproven_domains` 仍红向自证。
小噪点：tests/flow.rs:123 用 `format!("{:?}").contains("Unproven")` 字符串断言，语义弱但无害。）

### 1.3 焦点三：对拍独立复跑（三件 + 双版 + 三向链）

| 项 | 命令 | 独立实测输出 |
|---|---|---|
| workspace 测试 | `cargo test --workspace` | **135/135 全绿**（各 crate test result 求和=135；death_chain.rs 10 + flow.rs 7 计数核对） |
| l2-diff CN | `python3 tools/diff_subroutines.py` | **L2-DIFF: 121/121 通过**（golden==ROM 执行==game-core 同名函数） |
| l2-diff JP | `python3 tools/diff_subroutines.py --rom reference/jp/gunnac_jp.nes` | **L2-DIFF: 121/121 通过** |
| 门禁 | `python3 gates/cli.py check` | **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0**（0 静默跳过） |
| ratchet | baseline.json + history | rust_test_count **135/135 OK**；baseline-integrity sha 链一致；history 条目含"确认无回退"+metrics 快照+baseline_sha256；64ef557 含 Evidence/Confidence 字段（data/status/ 触及合规） |
| L2 集增量 | wc -l | l2_cases.jsonl **109→121**（+12）；l2_cases_jp.jsonl 同步 121；`dmh-*`×6+`ed3-*`×6 双集 id/入参全对齐 |

**l2ports 三向链抽验（death_money_halve ↔ bank03:A39F 字节）**：
- 函数 `money >> 1` ↔ ROM `bank03:A39F: 4e 8e 01`（LSR $18E）——**CN==JP 同址同字节 Δ0** ✓；l2 用例 entry=bank03:A39F / stops=bank03:A3A2（截于 trampoline 前）✓
- `bcd_sub_ammo` ↔ `bank15:ED03: 84 06 85 04 29 f0 85 05…`（STY $06/STA $04/AND #$F0/STA $05，与 N-0017 语义注一致）；**JP=ED0D +0x0A** 同字节 ✓；JP 用例集 entry 同步 ED0D ✓
- 三向（golden JSON == 6502 核 ROM 执行 == Rust 函数）由 l2-diff 121/121 三方缺一不可声明背书 ✓

**结论：对拍三件 + 双版 + ratchet + 三向链全部独立复跑属实。**

### 1.4 纪律核（增量/证据指针/基线留痕）

- 增量：core（11 文件）与 data（3 文件）分提；无门禁改动混入 ✓；小步提交 ✓
- 证据指针：spec 13 条逐条挂 findings §-指针（抽 6 条重导全对）；findings A/B/C 分级与假设区分离 ✓
- 基线留痕：baseline_history.jsonl + SHA256SUMS 同步；commit message Evidence/Confidence 齐 ✓
- 三处留痕（E1BB 缺口）核实：commit message 尾行 ✓ / spec R-continue-reset 注 1 + §2 E1BB 缺口行 ✓ / flow.rs `apply_continue_reset` 注 1 ✓——**属实**

## 2. 缺口裁定（核心）：bank15:$E1BB-E1BF `$26:=$26&$80` vs A2 §3.2

### 2.1 独立字节直读（两处写点，全程 seed 投影 + ROM 原字节双证）

```
第一写点 bank15:E0F4-E0FA（seed L87415-87421）：
  E0F4  A5 26    LDA $26
  E0F6  29 F7    AND #$F7        ; bit3(不可复活位)清
  E0F8  09 10    ORA #$10        ; bit4 置 1
  E0FA  85 26    STA $26
第二写点 bank15:E1BB-E1BF（seed L87506-87508）：
  E1BB  A5 26    LDA $26
  E1BD  29 80    AND #$80        ; 只留 bit7
  E1BF  85 26    STA $26
```

### 2.2 两写点执行序（控制流，A 级）

`sub_E0D9`（续关入口 `bank15:$E481 JMP sub_E0D9`）与共享尾段入口 `loc_E16B`（关间推进 `bank15:$E396 JMP loc_E16B`）的执行序：

```
E0D9 入口 → E0EF($18D:=2) → 【E0F4-E0FA 第一写点】→ E10A(JSR sub_ECBF 清 $E0-$E6，仅 bit7=0 路径)
  → E128(BPL→E160) → E160-E168($189/$190/$18E:=0) → 【E16B 共享尾段入口】→ E18C(网格四页清)
  → E19B($C8:=0) → E1B6-E1B8($0400:=1) → 【E1BB-E1BF 第二写点】→ E1C1-E1C7($27/$16/$18:=0)
  → E1D3-E1DD($188:=bit7?$FF:$40) → …
```

- **第二写点在第一写点之后**，线性后继、无分支跳过（E1B6→E1BB→E1C1 连续；A2 §3.2 自身已认定 E1B6 与 E1C1-E1C7 为续关路径写点，E1BB 恰在两者之间 ⇒ 依 A2 自己的表也必执行）。
- **两写点都在续关路径上，同一调用内先后执行。**

### 2.3 净效应裁定（A 级）

> **裁定：续关序列完成时 `$26 = 进入前 $26 & $80`。bit3=0（复活权恢复，A2 语义正确保留）、bit7 原样保留、`bit4 终值 = 0`。**
> A2 §3.2 的 `$26:=($26&$F7)|$10` 是**瞬态中间写**（E0FA 写入，E1BF 覆盖），不是序列终态。
> **续关后 $26 的 bit4 该 0。**

佐证（`$26` 全 ROM 写集穷举，8 处）：bank00:8D06（`$26\|=$10`，$CA&1 事件触发，条件置位）/ bank00:8DE1（`$26:=0`）/ bank00:8F8A（`\|=$80`）/ bank00:95FB（`=$80`）/ bank03:838E（`\|=$08` 终死 arm）/ E0FA（第一写点）/ **E1BF（第二写点）** / E372（`&=$EF` 关推进清 bit4）。
bit4 读点：bank15:E2F3（主环玩法门 `AND #$10/BNE`，bit4=0 则 JMP $E21C 等待）+ bank00:9C55（`$018F:=4` 块被 bit4 门控）。
⇒ E0F8 瞬态置 1 的功能=E0FA..E1BF 窗口内被调子程序可见 bit4=1；玩法期 bit4=1 只能由 bank00:8D06 事件再置（触发源动态未证，挂起）。

### 2.4 给 spec-writer 的裁定意见

1. **R-continue-reset 改写为两写点+净效应**："`$26`：E0F4-E0FA 瞬态 `($26&$F7)|$10`（bit3 清、bit4 瞬置）→ E1BB-E1BF 终态 `$26:=$26&$80` ⇒ **净效应 `$26&=$80`（bit3=0 复活权恢复 ✓；bit4=0；bit7 保留）**"。A2 §3.2 该行按此原地更正（证伪更正文化，不删历史）。
2. E1BB 从"规格缺口（未列写点）"转为**已裁定写点**，从 §2 遗留缺口清单销账，改记本裁定（复核 #69）。
3. 挂起新探针（AD69-2）：`bank00:9C5D $018F:=4`（bit4 门控块）是否经续关路径间接可达（`bank00:881A JMP ($0181)` 动态脚本向量，E1A8 在共享尾段内被调）⇒ 静态不可判定，需 continue 场景动态 trace（并入 R-G4 场景组）。若可达，F12 复位集需增补 `$018F:=4`，现"保留 bomb_ammo"断言同翻。

## 3. findings 四分类与修正建议

| # | 现象 | 分类 | 处理 |
|---|---|---|---|
| AR69-1（中） | R-continue-reset 把 `$26:=($26&$F7)\|$10` 记为终态；实现 `apply_continue_reset` 落 `\|0x10`；测试 f12 断言 `bit4==1` —— 三者被本裁定证伪（ROM 净效应 bit4=0） | 规格缺口（A2 §3.2 漏 E1BB）派生实现偏差 | 回 spec-writer 按 §2.4 改 R-continue-reset → 回 porter：`revive_flags` 终态改 `&= 0x80`（或显式两步镜像：瞬态+终态），f12 断言 `bit4==0` + 增补"瞬态窗"注；跑 death_chain + l2-diff + gates 复绿 |
| AD69-2（低，挂起） | `$018F:=4`（bank00:9C5D，bit4 门控）续关可达性未知；F12"bomb_ammo 保留"断言系 A2 表外推 | 规格缺口（挂起） | 指派 oracle 场景组：continue 动态 trace 查 `$018F`（R-G4 场景一并采 GAME OVER 屏） |
| AD69-3（提示） | tests/flow.rs:123 字符串断言 `contains("Unproven")` 语义弱 | 无害噪点 | 可选：改为枚举穷举断言 |

**修正建议（最小修复批）**：
1. spec：R-continue-reset `$26` 行改两写点+净效应（§2.4 措辞照抄即可）；findings game-over-continue.md §3.2 表 `$26` 行同型原地更正（标注复核 #69 裁定）。
2. core：`apply_continue_reset` 改 `st.revive_flags &= 0x80;`（E0F8 瞬态如需镜像另设字段/注释）；f12 断言改 `st.revive_flags & 0x10 == 0` 并注 E1BB。
3. 复验：`cargo test --workspace`（135）+ `python3 tools/diff_subroutines.py`（121，双版）+ `python3 gates/cli.py check`（12/12）；断言变化不减测试数，ratchet 不动。

## 4. 附录：复现命令与留痕

```bash
# 对拍三件（本复核实跑）
cargo test --workspace                                    # 135/135
python3 tools/diff_subroutines.py                          # 121/121
python3 tools/diff_subroutines.py --rom reference/jp/gunnac_jp.nes   # 121/121
python3 gates/cli.py check                                 # PASS=12 FAIL=0
# 缺口裁定字节直读
python3 - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read(); prg=rom[0x10:0x10+0x20000]
for cpu in (0xE0EF,0xE0F4,0xE1B6,0xE1BB,0xE1C1,0xE1D3):
    o=0x1E000+(cpu&0x1FFF); print(f"{cpu:04X}:", prg[o:o+6].hex(' '))
PY
# F5 trace 锚（f1787 无 trampoline / f1788 $A3:$A5）
# analysis/traces/gameplay-stage1.frames.jsonl 逐帧读 $041C/$0438（见 §1.1-1）
```

- mutation 留痕：5 突变逐条改/测/还原，flow.rs sha256 还原后恒 `795c17b5d05b1043e6a12c93380d205cfac84633b5d0bd417e409de243db2f2e`；工作区 `git diff` 复核后净空。
- 本复核未改动任何被审文件；唯一产出=本文件。

---
**Verdict: needs_revision**（必修仅 AR69-1〔1 处 spec 终态 + 1 处实现 + 1 处断言〕；AD69-2 挂起探针、AD69-3 可选）。
三焦点（未发明 6/6、mutation 5/5、对拍三件双版）全部独立复核通过；被审方缺口披露纪律合规。
