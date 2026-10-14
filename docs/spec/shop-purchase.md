# spec: 商店购买链接线（shop-purchase）→ docs/spec/shop-purchase.md

> 状态：2026-10-14 v1（任务 DEV-1，porter）。证据基准 = [../findings/shop-purchase-flow.md](../findings/shop-purchase-flow.md)
> （A1 定谳，§1-§4 四环 A 级）+ 本轮对 `reference/seed/gunnac.asm` 冻结投影的逐字节复核（增补证据见各规则）。
> 纪律（AGENTS.md §3）：**只收录 A/B 级证据支撑的可观察行为**；消息**文案内容**未证 ⇒ 只落状态枚举；
> 未证不发明（W-NEW* 留 findings §7 假设区，不入本文件结论）。
> 实现锚：`crates/game-core/src/ui.rs::shop_purchase_step`（纯逻辑、确定性、无 I/O）；
> 测试锚：`crates/game-core/tests/shop_purchase.rs`（14 用例，每用例注记期望 RAM 终态）。
> 与 [shop-economy.md](shop-economy.md) R-shop-5..8（信息页状态机 `ShopState`）**分立**：
> 两流的门控关系（首店/有款判定，findings W-NEW1）未证，禁止混接。

## 0. 范围与 RAM 契约

| RAM | 台账名 | 语义（已证） | 证据 |
|---|---|---|---|
| `$0400` | shop_slot_cursor（N-0002） | 主面板光标 0..3 | findings §1；bank00:$9D02-$9D23 |
| `$0401` | ui_select_cursor（N-0003） | 子面板光标：武器 [0..5]（5=翼）/ 槽 [stage..7] / 弹种 [0..3] | findings §1；bank00:$9D6C/$9D7B/$9F3E/$9F90 |
| `$0402`/`$0403` | —/delivery 槽索引（N-0201） | 进发货菜单三者同写 `$0180`；`$0403` 跟踪 `$0401` | findings §1；bank00:$9F1C-$9F22/$9F45-$9F50 |
| `$018E` | 商店金钱 | 0..255；扣减 10/6/4、入账 +3 封顶 `$FF` | findings §2；N-0202 更正 |
| `$0178+Y` | delivery_slots_7（N-0201） | 打包 `(type<<6)\|qty`，Y=`$0403` | findings §4；bank00:$9FC0-$9FDA |
| `$33`/`$34` | weapon_level/weapon_slot | 级/型（far-call `$922B` 生效） | findings §4；N-0032/33 |
| `$3C` | rapid_level | 连射级 0..4 | findings §3/§4；bank00:$9EDF |
| `$32` | flags | bit7=翼已购；bit5=退出隐藏门写（登记不解释） | findings §3/§4；bank01:$A037 |
| `$0180` | 关卡号 | 发货阶段门 2..7 | bank00:$9F03-$9F0E；W-NEW5 |
| `$18` | 手柄2 状态 | 退出隐藏选项门 `($18&$33)==$11` | bank01:$A023-$A029 |

**按键位（A 级，bank00:$9D02-$9D3E / sub_9E40 / bank01:$A0CF）**：bit7=A、bit6=B、bit4=Start、
bit3=Up、bit2|bit5=Down 组（`AND #$24`）、bit1=左、bit0|bit5=右组（`AND #$21`）；
主环选中 = `$16 & $D0`（A/B/Start 任一）；消息 ack = `$16 & $C0`（A/B 任一）。

**字节级复现（A 级，本文件新增指针一律可复跑）**：
```
python3 - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read(); prg=rom[0x10:0x10+0x20000]
def show(bank,cpu,n):
    o=bank*0x2000+(cpu&0x1FFF); print(f"bank{bank:02d}:{cpu:04X}", prg[o:o+n].hex(' '))
for args in [(0,0x9D02,0x24),(0,0x9D56,8),(0,0x9D6C,4),(0,0x9D91,0x50),(0,0x9DE8,0xD),
             (0,0x9E40,0x38),(0,0x9E94,0x12),(0,0x9EDF,3),(0,0x9F03,0x14),(0,0x9F1C,7),
             (0,0x9F62,0x11),(0,0x9FAB,0x30),(1,0xA023,0x22),(1,0xA05B,0x0E),(1,0xA0CF,5),
             (2,0x8FA9,0x0E)]:
    show(*args)
PY
```

## 1. 行为规则

### R-shop-purchase-1: 主面板光标 `$0400` 0..3 环绕；Up `DEC` 0→3、Down `INC` 3→0；选中键 `$16&$D0` 经 RTS 跳转表分发四入口
- 证据: findings §1（触发链表 + `bank00:$9D02-$9D55` 指令序列 `DEC $400/BPL/LDA #3`、`INC $400/CMP #4/LDA #0`、`AND #$D0` + `bank00:$9D56` 表字 = 入口−1）；零输入零写入（同 R-shop-5 静态屏族）
- 可判定性: 机器可判定（`tests/shop_purchase.rs::main_cursor_wraps_0_to_3`；上面 hex 复现第 1-2 行）
- 关联: N-0002、R-shop-purchase-2、B-SYS-SHOP

### R-shop-purchase-2: 四入口 = 武器 `$9D5E` / 连射 `$9E86` / 发货 `$9F03` / 退出 `bank01:$A023`；武器子光标 `$0401` init := `$34`，环绕 [0..5]（5=翼条目）
- 证据: findings §1 表（表项 `$9D5D/$9E85/$9F02/$A022` +1）；`bank00:$9D6C` `LDA $34/STA $401`；子光标移动 `bank00:$9E40-$9E72`（边界寄存器 `$11=0/$10=5`，bank00:$9D77-$9D7D）；翼条目判定 `bank00:$9D94 CMP #$05`
- 可判定性: 机器可判定（`weapon_buy_same_type_levels_up`/`weapon_buy_switches_type`/`wing_*`）
- 关联: R-shop-purchase-1/5、N-0003、findings §1「子光标」

### R-shop-purchase-3: 扣减同构三价 10/6/4（`$018E`），不足共享出口 `bank01:$A05B` ⇒ 提示状态 + 回主环（`$0400` 复位 0），零写入
- 证据: findings §2 三扣减点字节序列（`bank00:$9DE8/9ECD/9FAB`，`CMP #价/BCS/JMP $A05B/SBC/STA`）+ §3 类 1（`$A05B` → 清行 → 文本 `$A13D` → `$A09F` → `JMP $9CEC`）；回主环复位 `bank00:$9CEC` `LDA #0/STA $400`（本轮 seed 复核）；价 10 片段 = `l2ports::digit_conversion`（findings §5：digit-1..5 同片段）
- 可判定性: 机器可判定（`insufficient_funds_never_writes` 三价分支；l2-diff sp-r3-poor5/sp-s2-poor3/digit-*）
- 关联: R-shop-purchase-8、A1-DED-1/2/3、W-NEW4（旧名 digit_conversion 伪语义遗留）

### R-shop-purchase-4: 校验先于扣款，共 6 个已证分支：不足 / 武器满级 `$33`≥7 / 连射满级 `$3C`≥4 / 翼已购 / 翼前置未满足 / 发货数量封顶；另发货入口有阶段门 `$0180`∈[2,7]
- 证据: findings §3 类 1/2（`bank00:$9DCC CMP #$07`→消息 `$A408`；`$9E94 CMP #$04`→消息 `$A4D4`；`$9FC8 CMP #$40`→`LDA #$3F`；翼 `$9D98-$9DCC`：已购→`$A436`、同型 `$33`≥2 无翼→`$A3B0`）+ 本轮 seed 复核 `bank00:$9F03-$9F16`（`LDA $180/CMP #8/BCS 消息 $A62A/CMP #2/BCS 可购`）；扣款在全部校验之后（bank00:$9DE3-$9DE8 起扣）
- 可判定性: 机器可判定（`weapon_level_cap_branch`/`rapid_buy_and_level_cap`/`wing_validation_branches`/`ship_qty_caps_at_63`/`ship_stage_gate_blocks_purchase`）
- 关联: R-shop-purchase-3/5、findings §3

### R-shop-purchase-5: 生效写入四型——武器 far-call `bank02:$922B`（关内拾取同例程 ⇒ 同型 `$33`+1 / 异型 `$34` 换型）；翼 `$32` bit7 + far-call `$92C5`；连射 `INC $3C`（==4 转满级提示）；发货 `$0178+Y := ($0401<<6)|qty`（qty=`min((old&$3F)+1,$3F)`）
- 证据: findings §4 表（`bank00:$9E01` far-call 内联 `02 03 2B 92`；`$9E0E-$9E1B` `ORA #$80/STA $32` + `02 03 C5 92`；`$9EDF INC $3C` + `$9EE1-$9EA5` ==4 回 `$9E86` 满级消息；`$9FC0-$9FDA` 打包 ROL×3/ROR×3 = 高 2 位语义）+ far-call 跳板 `bank15:$E649-$E68F`（入口 `STA $04` 保活 A、尾 `LDA $04/JMP ($24)` ⇒ 目标收 A=`$0401`，本轮 seed 复核）
- 可判定性: 机器可判定（`weapon_buy_*`/`rapid_buy_and_level_cap`/`ship_buy_packs_delivery_slot`；l2-diff sq-*/pk-*）
- 关联: R-item-2/3（同一例程）、R-shop-purchase-9（$92C5 未移植）、N-0201

### R-shop-purchase-6: 发货子流两环：槽环 `[stage..7]`（`$0402`/`$0403` 同写 `$0180`、`$0403` 逐环跟踪 `$0401`）→ A 取旧弹种（`(slot>>6)&3`）→ 弹种环 [0..3] → A 购买；购买后停留弹种环
- 证据: findings §1（`$0402`/`$0403` 三者同写 `bank00:$9F1C-$9F22`）+ 本轮 seed 复核：槽环边界 `$11=$0402/$10=7`（`bank00:$9F39-$9F42`）、同步 `$9F45-$9F50`、取弹种 `ROL×3/AND #$03`（`$9F62-$9F72`）、弹种环边界 `$11=0/$10=3`（`$9F8C-$9F92`）、购后 `JMP $9F86`（`$9FE0`）
- 可判定性: 机器可判定（`ship_buy_packs_delivery_slot`/`ship_qty_caps_at_63`）
- 关联: R-shop-purchase-4/5、W-NEW5（`$0177+Y` 兑现 vs `$0178+Y` 购买索引基差=1 未对账——**索引 ≥8 时实现 fail-closed 不动作**，未证不发明）

### R-shop-purchase-7: 退出入口 `bank01:$A023`：隐藏选项门 `($18&$33)==$11` ⇒ `$32` bit5 置位（登记不解释）；转场（`JSR $EE03` A=3 / `JMP $EEFB`）交 flow，本线不发明
- 证据: 本轮 seed 复核 `bank01:$A023-$A058`（`LDA $18/AND #$33/CMP #$11/BEQ` → `$32 |= $20` + 文本 `$A65C`；`$A31F` + `LDA #$03/JSR $EE03/JMP $EEFB`）；findings §1 退出入口（表项 `$A022` +1）
- 可判定性: 机器可判定（`exit_entry_and_hidden_gate`）
- 关联: R-ui-7（flow 侧 ExitTransition→StageStart）、R-shop-purchase-1

### R-shop-purchase-8: 提示消息只落状态枚举（文案内容未证）：6 类 = InsufficientFunds(`$A13D`)/WeaponMax(`$A408`)/RapidMax(`$A4D4`)/WingAlreadyOwned(`$A436`)/WingRequired(`$A3B0`)/ShipStageBlocked(`$A62A`)；ack（`$16&$C0`）后：不足/阶段门/连射满级→主环（cursor 复位 0），翼两分支/武器满级→重入武器面板（`$0401` 复位 := `$34`）
- 证据: findings §3（消息文本指针 + JP↔社区 state() 对照 0x1FC4/0x21A0/0x21C3/0x21E0）+ 本轮 seed 复核返回目标：`bank00:$9DAA/$9DC9/$9DE0` `JMP $9D5E`（重入武器面板，`$9D6C` init）、`bank01:$A069` `JMP $9CEC` 与 `bank00:$9EA5/$9F19` `JMP $9CEC`（回主环）、ack 门 `bank01:$A0CF AND #$C0`
- 可判定性: 机器可判定（`weapon_level_cap_branch`/`rapid_buy_and_level_cap`/`insufficient_funds_never_writes` 的 phase/prompt/ack 断言）；**文案字形仅人工核对**（未入断言）
- 关联: R-shop-purchase-3/4、findings §3

### R-shop-purchase-9（缺口，显式挂起）: 翼购买的 far-call `bank02:$92C5`（翼收尾例程）**未移植**——实现以 `ShopFx::WingBoughtFarCallPending` 显式挂起，禁止发明其写集
- 证据: findings §4（`bank00:$9E14` + 内联 `02 03 C5 92`）；`bank02:$92C5` 起始片段（`STA $3A/$39`、`$32` `AND #$F7/ORA #$01`、`LDA $968F,Y/STA $01A5` 等）本轮只登记不移植（docs/findings/upgrade-branch-h02.md 收尾族语义未定谳）
- 可判定性: 机器可判定（`wing_buy_sets_bit7_and_flags_unported_call` 断言挂起标记）
- 关联: R-shop-purchase-5、H-02 域

### R-shop-purchase-10: 商店金钱入账 +3、进位即封顶 `$FF`（`bank02:$8FA9`）已接 `shop_money_credit`；**触发时机未接线**（谁在何时 +3 未证）
- 证据: findings §2「金钱双轨更正」（`CLC/ADC #$03`、溢出封顶 `$FF`、JP 同址同字节）；l2-diff se-1..5（109/109）
- 可判定性: 机器可判定（`money_credit_caps_at_ff`）
- 关联: R-shop-purchase-3、A1-CAP-2、W-NEW2（`$E0-$E6` ↔ `$018E` 关系未证）

## 2. 未证域（禁止发明，留 findings 假设区）

- **W-NEW1**：欢迎页流 ↔ 主菜单环门控（首店/有款判定）未取证 ⇒ `ShopState`（R-shop-5..8）与本线 `shop_purchase_step` 不混接；
- **W-NEW2**：`$E0-$E6`（关卡入账 BCD）与 `$018E`（商店金钱）语义关系未证；
- **W-NEW5**：兑现 `$0177+Y`（Y=`$0180`）vs 购买 `$0178+Y`（Y=`$0403`）索引基差=1 未对账 ⇒ 索引 ≥8 fail-closed；
- 消息**文案内容**、音效 `$43/$44`、文本重绘（`$A06C/$A091/$A09F`）属渲染/音频线，不在本线断言域。

## spec 产出契约

- 规则清单: R-shop-purchase-1..10（共 10 条，逐条带证据指针 = findings 对应节 + bank:addr）
- 常量对照: 价 10/6/4 = C-shop-p1/p2/p3（[constants.md](constants.md) v3）；封顶 `$FF`/63 = A1-CAP-2/A1-CAP-1
- 可判定性标注: 机器可判定 9 条 + 缺口显式挂起 1 条（R-shop-purchase-9）；文案字形人工核对项不在断言域
- 门禁: `cargo test --workspace` 122 全绿（108 既有 + 14 新增）；`l2-diff` 109/109；`python3 gates/cli.py check` 12 PASS
- 遗留假设: W-NEW1/2/5（findings §7）、R-shop-purchase-9 缺口、H-10（信息页 A 键动态效果）仍待 S2
