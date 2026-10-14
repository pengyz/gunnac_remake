//! DEV-1 A1 商店购买链接线集成测试（R-shop-purchase-*）。
//!
//! 证据基准：`docs/findings/shop-purchase-flow.md` §1-§4（A 级）+ `docs/spec/shop-purchase.md`。
//! 输入 = `$16` 手柄位（bank00:$9D02-$9D3E / 子面板 sub_9E40）；每测试给出期望 RAM 终态
//! （字段名后括注 ROM 地址）。测试只断言行为终态，不断言实现内部结构。
//!
//! 未证不发明：消息**文案内容**未证 ⇒ 只断言 `ShopPrompt` 状态枚举；声音/文本渲染不在断言域。
//!
//! TDD 留痕（AD65-2，review-65 §6-4）：本轮 AR65-1/AR65-2 新增用例先行落盘再改实现，
//! RED 输出留档 `analysis/tdd-red-dev1-ar65-20261014.log`（RED-1 E0599 编译面 + RED-2 行为面
//! 复现审方分歧态 `stage=8,pad2=0 ⇒ $32 bit5 0x00 vs 0x20` / `Exited vs Message`）。

use game_core::ui::*;
use game_core::weapons::{PickupContext, WeaponState};

fn st() -> ShopPurchaseState {
    ShopPurchaseState::default()
}

/// ①光标移动环绕（bank00:$9D02-$9D23）：Up(bit3) `DEC $0400` 0→3 回卷；
/// Down(`AND #$24`) `INC $0400` 3→0 回卷。期望 RAM 终态 = `$0400` 序列。
#[test]
fn main_cursor_wraps_0_to_3() {
    let mut s = st();
    assert_eq!(s.cursor, 0, "初始 $0400=0（bank00:$9CF7 STA $400）");
    shop_purchase_step(&mut s, BTN_UP);
    assert_eq!(s.cursor, 3, "期望 $0400=3（0 DEC 后为负 ⇒ LDA #3）");
    shop_purchase_step(&mut s, BTN_UP);
    assert_eq!(s.cursor, 2);
    shop_purchase_step(&mut s, BTN_DOWN);
    assert_eq!(s.cursor, 3);
    shop_purchase_step(&mut s, BTN_DOWN);
    assert_eq!(s.cursor, 0, "期望 $0400=0（INC 后 ==4 ⇒ 清 0）");
    // 无输入不动（静态屏同族）：期望 RAM 终态 = 全部字段不变
    let before = s;
    shop_purchase_step(&mut s, 0);
    assert_eq!(s, before, "零输入零写入");
}

/// ②武器购买成功流（同型）：钱够(≥10) + 级未满 ⇒ money−10 / `$33`+1 / `$34` 不变。
/// 期望 RAM 终态：$018E=10、$33=2、$34=2、phase=WeaponMenu。
/// （生效= far-call bank02:$922B = 关内拾取同例程；$018C≠$FF 时同型走裸升 `$92B4`。）
#[test]
fn weapon_buy_same_type_levels_up() {
    let mut s = st();
    s.money = 20; // $018E
    s.weapon.level = 1; // $33
    s.weapon.slot = 2; // $34
    s.pickup.threshold = 0x00; // $018C（shop 期值未证——显式取非 $FF 走裸升，不发明默认）
    shop_purchase_step(&mut s, BTN_SELECT); // cursor=0 ⇒ 武器入口 bank00:$9D5E
    assert_eq!(s.phase, ShopPurchasePhase::WeaponMenu);
    assert_eq!(s.item_cursor, 2, "$0401 init := $34（bank00:$9D6C）");
    let fx = shop_purchase_step(&mut s, BTN_A);
    assert_eq!(fx, ShopFx::WeaponBought);
    assert_eq!(s.money, 10, "期望 $018E=20−10（bank00:$9DF4 STA $18E）");
    assert_eq!(s.weapon.level, 2, "期望 $33=1+1（far-call $922B 同型升）");
    assert_eq!(s.weapon.slot, 2, "期望 $34=2（同型不换）");
    assert_eq!(s.phase, ShopPurchasePhase::WeaponMenu, "购后回武器环（bank00:$9E0B）");
    assert_eq!(s.prompt, None, "成功路径无提示");
}

/// ②武器购买成功流（异型）：选中 ≠ `$34` ⇒ `$34` 换型、`$33` 不变、money−10。
/// 期望 RAM 终态：$018E=10、$33=3、$34=3。
#[test]
fn weapon_buy_switches_type() {
    let mut s = st();
    s.money = 20;
    s.weapon.level = 3;
    s.weapon.slot = 2;
    shop_purchase_step(&mut s, BTN_SELECT);
    shop_purchase_step(&mut s, BTN_RIGHT); // sub_9E40：2→3
    assert_eq!(s.item_cursor, 3);
    let fx = shop_purchase_step(&mut s, BTN_A); // 异型 ⇒ 跳过同型校验链（bank00:$9DAF BNE）
    assert_eq!(fx, ShopFx::WeaponBought);
    assert_eq!(s.money, 10);
    assert_eq!(s.weapon.slot, 3, "期望 $34=3（$922B 异型路径换型）");
    assert_eq!(s.weapon.level, 3, "期望 $33 不变（异型不加级）");
}

/// ③不足分支：money<价 ⇒ 状态=提示（InsufficientFunds）+ 不扣钱 + 零 RAM 写入。
/// 三价 10/6/4 同构共享出口 bank01:$A05B（findings §2/§3）。
/// 期望 RAM 终态：$018E 不变、$33/$34/$3C/$0178+Y 零写入。
#[test]
fn insufficient_funds_never_writes() {
    // 价 10（武器）：money=9
    let mut s = st();
    s.money = 9;
    s.weapon.level = 1;
    s.weapon.slot = 1;
    shop_purchase_step(&mut s, BTN_SELECT);
    shop_purchase_step(&mut s, BTN_A);
    assert_eq!(s.prompt, Some(ShopPrompt::InsufficientFunds));
    assert_eq!(s.phase, ShopPurchasePhase::Message);
    assert_eq!(s.money, 9, "期望 $018E=9（CMP 早退 JMP $A05B，不写）");
    assert_eq!(s.weapon.level, 1);
    assert_eq!(s.weapon.slot, 1);
    // 价 6（连射）：money=5
    let mut s = st();
    s.money = 5;
    shop_purchase_step(&mut s, BTN_DOWN); // $0400 0→1
    shop_purchase_step(&mut s, BTN_SELECT);
    assert_eq!(s.phase, ShopPurchasePhase::RapidMenu);
    shop_purchase_step(&mut s, BTN_A);
    assert_eq!(s.prompt, Some(ShopPrompt::InsufficientFunds));
    assert_eq!(s.money, 5);
    assert_eq!(s.rapid_level, 0, "期望 $3C=0（不 INC）");
    // 价 4（发货）：money=3
    let mut s = st();
    s.money = 3;
    s.stage = 3; // $0180
    s.delivery = [0; 8];
    shop_purchase_step(&mut s, BTN_DOWN);
    shop_purchase_step(&mut s, BTN_DOWN); // $0400=2
    shop_purchase_step(&mut s, BTN_SELECT);
    shop_purchase_step(&mut s, BTN_A); // 选槽 ⇒ 弹种环
    shop_purchase_step(&mut s, BTN_A); // 购买失败
    assert_eq!(s.prompt, Some(ShopPrompt::InsufficientFunds));
    assert_eq!(s.money, 3);
    assert_eq!(s.delivery, [0u8; 8], "期望 $0178+Y 零写入");
}

/// ④满级分支（武器）：同型 + 已购翼 + `$33`≥7 ⇒ prompt=WeaponMax 且不扣钱
/// （校验先于扣款，bank00:$9DCC-$9DE0）。ack 后回武器面板且 `$0401` 复位 := `$34`。
/// 期望 RAM 终态：$018E=50、$33=7、$34=3 不变。
#[test]
fn weapon_level_cap_branch() {
    let mut s = st();
    s.money = 50;
    s.weapon.level = 7;
    s.weapon.slot = 3;
    s.pickup.flags = 0x80; // $32 bit7 = wing 已购
    shop_purchase_step(&mut s, BTN_SELECT);
    shop_purchase_step(&mut s, BTN_A);
    assert_eq!(s.prompt, Some(ShopPrompt::WeaponMax));
    assert_eq!(s.phase, ShopPurchasePhase::Message);
    assert_eq!(s.money, 50, "满级不扣钱（扣款在校验后）");
    assert_eq!(s.weapon.level, 7);
    // ack（A/B 任一，bank01:$A0CF AND #$C0）⇒ 重入武器面板（bank00:$9DAA JMP $9D5E）
    shop_purchase_step(&mut s, BTN_ACK);
    assert_eq!(s.phase, ShopPurchasePhase::WeaponMenu);
    assert_eq!(s.item_cursor, 3, "$0401 复位 := $34");
    assert_eq!(s.prompt, None);
}

/// ④满级分支（连射）+ 生效 `INC $3C`：钱够 ⇒ $3C 3→4、money−6，且 ==4 触满级消息
/// （bank00:$9EDF-$9EA5）；已满级再入 ⇒ 直接消息不扣钱。
/// 期望 RAM 终态：$018E=6、$3C=4。
#[test]
fn rapid_buy_and_level_cap() {
    let mut s = st();
    s.money = 12;
    s.rapid_level = 3; // $3C
    shop_purchase_step(&mut s, BTN_DOWN); // $0400=1
    shop_purchase_step(&mut s, BTN_SELECT);
    let fx = shop_purchase_step(&mut s, BTN_A);
    assert_eq!(fx, ShopFx::RapidBought);
    assert_eq!(s.money, 6, "期望 $018E=12−6（bank00:$9ED9）");
    assert_eq!(s.rapid_level, 4, "期望 $3C=3+1（bank00:$9EDF INC $3C）");
    assert_eq!(s.prompt, Some(ShopPrompt::RapidMax), "INC 后 ==4 ⇒ $A4D4 消息");
    // ack 回主环（cursor 复位 0）；再入连射 ⇒ 满级消息、不扣钱
    shop_purchase_step(&mut s, BTN_ACK);
    assert_eq!(s.phase, ShopPurchasePhase::MainRing);
    assert_eq!(s.cursor, 0, "回主环复位 $0400=0（序列入口 bank00:$9CEC；`LDA #0/STA $400` 在 $9CF7/$9CF9——AD65-1）");
    shop_purchase_step(&mut s, BTN_DOWN);
    shop_purchase_step(&mut s, BTN_SELECT);
    assert_eq!(s.prompt, Some(ShopPrompt::RapidMax), "$3C≥4 入口即消息（bank00:$9E94）");
    assert_eq!(s.money, 6, "期望 $018E 不变");
    assert_eq!(s.rapid_level, 4);
}

/// ⑤发货购买 + 打包写入 `$0178+Y`：money−4；`$0178+$0403 := ($0401<<6)|min((old&$3F)+1,$3F)`
/// （bank00:$9FAB-$9FDA）。槽光标环绕 `[stage..7]`、弹种光标环绕 `[0..3]`（sub_9E40 边界）。
/// 期望 RAM 终态：$018E=16、$0178+4=0x86、phase=ShipTypeMenu。
#[test]
fn ship_buy_packs_delivery_slot() {
    let mut s = st();
    s.money = 20;
    s.stage = 3; // $0180 ⇒ 槽域 [3..7]
    s.delivery = [0; 8];
    s.delivery[4] = 0x45; // 旧槽值：type=(0x45>>6)=1、qty=5
    shop_purchase_step(&mut s, BTN_DOWN);
    shop_purchase_step(&mut s, BTN_DOWN); // $0400=2（发货）
    shop_purchase_step(&mut s, BTN_SELECT);
    assert_eq!(s.phase, ShopPurchasePhase::ShipSlotMenu);
    assert_eq!(s.item_cursor, 3, "$0401 init := $0180（bank00:$9F1C）");
    assert_eq!(s.ship_index, 3, "$0403 同写");
    assert_eq!(s.ship_lo, 3, "$0402 同写");
    shop_purchase_step(&mut s, BTN_RIGHT); // 槽 3→4
    assert_eq!(s.item_cursor, 4);
    assert_eq!(s.ship_index, 4, "$0403 跟踪 $0401（bank00:$9F45-$9F50）");
    shop_purchase_step(&mut s, BTN_A); // 选槽 ⇒ 弹种环（$0401 := 旧弹种）
    assert_eq!(s.phase, ShopPurchasePhase::ShipTypeMenu);
    assert_eq!(s.item_cursor, 1, "$0401 := (0x45>>6)&3=1（bank00:$9F6A-$9F72）");
    shop_purchase_step(&mut s, BTN_RIGHT); // 弹种 1→2
    let fx = shop_purchase_step(&mut s, BTN_A);
    assert_eq!(fx, ShopFx::ShipBought);
    assert_eq!(s.money, 16, "期望 $018E=20−4（bank00:$9FB7）");
    assert_eq!(s.delivery[4], 0x86, "期望 $0178+4=(2<<6)|6=0x86（qty 5→6）");
    assert_eq!(s.phase, ShopPurchasePhase::ShipTypeMenu, "购后停留弹种环（bank00:$9FE0）");
}

/// ⑤满仓分支：qty 已 63($3F) ⇒ 保持 63（`shop_ship_qty_next` 封顶），弹种位照写。
/// 期望 RAM 终态：$0178+2=0xFF（type=3、qty=63）、$018E=6。
#[test]
fn ship_qty_caps_at_63() {
    let mut s = st();
    s.money = 10;
    s.stage = 2;
    s.delivery = [0; 8];
    s.delivery[2] = 0x7F; // type=1、qty=63
    shop_purchase_step(&mut s, BTN_DOWN);
    shop_purchase_step(&mut s, BTN_DOWN);
    shop_purchase_step(&mut s, BTN_SELECT); // 槽环：item=2
    shop_purchase_step(&mut s, BTN_A); // 弹种环：item := 1
    shop_purchase_step(&mut s, BTN_RIGHT);
    shop_purchase_step(&mut s, BTN_RIGHT); // 弹种 1→3
    shop_purchase_step(&mut s, BTN_A); // 购买
    assert_eq!(s.delivery[2], 0xFF, "期望 (3<<6)|0x3F=0xFF（qty 封顶 63）");
    assert_eq!(s.money, 6);
    assert_eq!(
        s.delivery[2] & 0x3F,
        0x3F,
        "负向哨兵：qty 不得回绕 64→0"
    );
}

/// ⑤发货阶段门（bank00:$9F03-$9F19）：`$0180`∉[2,7] ⇒ 消息回主菜单、零写入。
/// 期望 RAM 终态：prompt=ShipStageBlocked、delivery 不变。
#[test]
fn ship_stage_gate_blocks_purchase() {
    for stage in [1u8, 8u8] {
        let mut s = st();
        s.money = 20;
        s.stage = stage;
        s.delivery = [0x11; 8];
        shop_purchase_step(&mut s, BTN_DOWN);
        shop_purchase_step(&mut s, BTN_DOWN);
        shop_purchase_step(&mut s, BTN_SELECT);
        assert_eq!(s.prompt, Some(ShopPrompt::ShipStageBlocked), "stage={stage}");
        assert_eq!(s.phase, ShopPurchasePhase::Message);
        assert_eq!(s.delivery, [0x11; 8], "零写入");
        assert_eq!(s.money, 20);
    }
}

/// 校验 4 类·wing（bank00:$9D98-$9DCC）：
/// (a) `$32` bit7 已购 + 光标=5 ⇒ wing_already_owned；(b) 同型 + `$33`≥2 + 无翼 ⇒ wing_required；
/// 两者皆不扣钱。期望 RAM 终态：$018E 不变、$32 不变。
#[test]
fn wing_validation_branches() {
    // (a) 已购翼
    let mut s = st();
    s.money = 50;
    s.weapon.slot = 0;
    s.pickup.flags = 0x80;
    shop_purchase_step(&mut s, BTN_SELECT);
    for _ in 0..5 {
        shop_purchase_step(&mut s, BTN_RIGHT); // $0401 0→5
    }
    assert_eq!(s.item_cursor, 5);
    shop_purchase_step(&mut s, BTN_A);
    assert_eq!(s.prompt, Some(ShopPrompt::WingAlreadyOwned));
    assert_eq!(s.money, 50);
    assert_eq!(s.pickup.flags, 0x80, "$32 不变");
    // (b) 同型 ≥2 级且无翼
    let mut s = st();
    s.money = 50;
    s.weapon.slot = 2;
    s.weapon.level = 2;
    shop_purchase_step(&mut s, BTN_SELECT); // item := 2（同型）
    shop_purchase_step(&mut s, BTN_A);
    assert_eq!(s.prompt, Some(ShopPrompt::WingRequired));
    assert_eq!(s.money, 50);
    assert_eq!(s.weapon.level, 2);
}

/// ⑥退出分支（bank01:$A023）：主环光标 3 + 选中（Start∈`$16&$D0`）⇒ 零购买写入；
/// 隐藏选项门（bank01:$A023-$A03B，两触发源见 ⑥' 用例）：`($18&$33)==$11` ⇒ `$32` bit5 置位
/// 并停消息相位（AR65-2：ack 后才转场，断言见 `exit_hidden_gate_*` 用例）；门不满足 ⇒ phase=Exited。
/// 期望 RAM 终态：门不满足 phase=Exited、$018E 不变；门满足时 $32 |= $20。
#[test]
fn exit_entry_and_hidden_gate() {
    let mut s = st();
    s.money = 7;
    for _ in 0..3 {
        shop_purchase_step(&mut s, BTN_DOWN); // $0400=3
    }
    assert_eq!(s.cursor, 3);
    let fx = shop_purchase_step(&mut s, BTN_START);
    assert_eq!(s.phase, ShopPurchasePhase::Exited);
    assert_eq!(s.money, 7, "退出零写入 $018E");
    assert_eq!(fx, ShopFx::None, "退出相位交 flow 开 stage（本线不发明转场）");
    // 隐藏门满足
    let mut s = st();
    s.money = 7;
    s.pad2 = 0x11; // $18
    s.cursor = 3;
    shop_purchase_step(&mut s, BTN_START);
    assert_eq!(s.pickup.flags, 0x20, "期望 $32 bit5（($18&$33)==$11 分支）");
    assert_eq!((s.phase, s.prompt), (ShopPurchasePhase::Message, Some(ShopPrompt::ExitHidden)), "AR68-1：trigger-(a)-alone 须停消息相位+ExitHidden（锁死非对称分支，ack 前不转场）");
    // 门不满足：$32 不变
    let mut s = st();
    s.pad2 = 0x00;
    s.cursor = 3;
    shop_purchase_step(&mut s, BTN_START);
    assert_eq!(s.pickup.flags, 0x00);
}

/// ⑥'退出隐藏门触发源(b)（AR65-1，review-65 §1.2）：`$0180==8` 单独成立（`$18` 组合不成立）也置
/// `$32` bit5（bank01:$A02B `ad 80 01 c9 08 d0 16`——不触发才跳 $A048）；触发后停消息相位
/// （AR65-2：文本 `$A65C` + bank01:$A045 `JSR $A09F`，ack 门 bank01:$A0CF `AND #$C0`），
/// ack 前不转场、ack 后才合流 `$A048` 转场（`text $A31F/$E6D9/LDA #$03/JSR $EE03/JMP $EEFB`，交 flow）。
/// 期望 RAM 终态：$32|=0x20、prompt=ExitHidden、ack 前 phase=Message、ack 后 phase=Exited。
#[test]
fn exit_hidden_gate_stage8_triggers_with_message_ack() {
    let mut s = st();
    s.money = 7;
    s.stage = 8; // $0180
    s.pad2 = 0x00; // $18：触发源(a) ($18&$33)==$11 不成立
    s.cursor = 3;
    let fx = shop_purchase_step(&mut s, BTN_START);
    assert_eq!(s.pickup.flags, 0x20, "期望 $32 bit5（stage==8 触发源(b)，bank01:$A02B-$A03B）");
    assert_eq!(s.phase, ShopPurchasePhase::Message, "ack 前不转场（bank01:$A045 JSR $A09F）");
    assert_eq!(s.prompt, Some(ShopPrompt::ExitHidden), "第 7 变体：文本 $A65C 登记不解释");
    assert_eq!(fx, ShopFx::None);
    assert_eq!(s.money, 7, "退出链零购买写入 $018E");
    // 非 ack 输入不放行（ack 门 `$16&$C0`，bank01:$A0CF）
    shop_purchase_step(&mut s, BTN_UP);
    assert_eq!(s.phase, ShopPurchasePhase::Message, "ack 门未开不放行");
    // ack（A/B 任一）后合流转场
    shop_purchase_step(&mut s, BTN_B);
    assert_eq!(s.phase, ShopPurchasePhase::Exited, "ack 后合流 $A048 转场（交 flow）");
    assert_eq!(s.prompt, None);
}

/// ⑥'退出隐藏门双触发源同时成立（`$0180==8` + `($18&$33)==$11`，review-65 §6-1）：`$32` bit5
/// `ORA #$20` 幂等不双写（终态 0x20 非 0x40），仍停消息相位、ack 后转场。
/// 期望 RAM 终态：$32==0x20、prompt=ExitHidden、ack 后 phase=Exited。
#[test]
fn exit_hidden_gate_dual_trigger_sets_bit5_once() {
    let mut s = st();
    s.money = 7;
    s.stage = 8;
    s.pad2 = 0x11; // 两触发源同时成立
    s.cursor = 3;
    shop_purchase_step(&mut s, BTN_START);
    assert_eq!(s.pickup.flags, 0x20, "双触发不双写：$32 终态 bit5 单置（bank01:$A039 ORA #$20）");
    assert_eq!(s.phase, ShopPurchasePhase::Message);
    assert_eq!(s.prompt, Some(ShopPrompt::ExitHidden));
    shop_purchase_step(&mut s, BTN_A);
    assert_eq!(s.phase, ShopPurchasePhase::Exited, "ack 后合流 $A048 转场");
}

/// ⑥'退出隐藏门触发源(b) 边界（AR65-1 负向哨兵）：ROM `CMP #$08/BNE` 是**精确 ==8**（非 ≥8）——
/// stage=7 / stage=9 且组合键不成立时不得置位、不得停消息相位（直接合流 $A048 转场）。
/// 期望 RAM 终态：$32 不动、prompt=None、phase=Exited。
#[test]
fn exit_hidden_gate_stage_boundary_is_exact_8() {
    for stage in [7u8, 9u8] {
        let mut s = st();
        s.money = 7;
        s.stage = stage;
        s.pad2 = 0x00;
        s.cursor = 3;
        shop_purchase_step(&mut s, BTN_START);
        assert_eq!(s.pickup.flags, 0x00, "stage={stage}：$0180==8 不成立，$32 不得置位");
        assert_eq!(s.prompt, None, "stage={stage}：无 $A65C 消息（直接合流 $A048）");
        assert_eq!(s.phase, ShopPurchasePhase::Exited, "stage={stage}：直接转场");
    }
}

/// 翼购买生效（bank00:$9E0E-$9E12）：money−10 + `$32` bit7 置位；
/// far-call `bank02:$92C5` **未移植** ⇒ 以 `WingBoughtFarCallPending` 挂起（不发明其写集）。
#[test]
fn wing_buy_sets_bit7_and_flags_unported_call() {
    let mut s = st();
    s.money = 30;
    s.weapon.slot = 0;
    shop_purchase_step(&mut s, BTN_SELECT);
    for _ in 0..5 {
        shop_purchase_step(&mut s, BTN_RIGHT);
    }
    let fx = shop_purchase_step(&mut s, BTN_A);
    assert_eq!(fx, ShopFx::WingBoughtFarCallPending, "$92C5 未移植必须显式挂起");
    assert_eq!(s.money, 20, "期望 $018E=30−10");
    assert_eq!(s.pickup.flags, 0x80, "期望 $32 bit7（ORA #$80/STA $32）");
}

/// 入账接线（bank02:$8FA9，A1-CAP-2）：+3，进位即封顶 $FF（非 256 截断）。
/// 触发时机未接线（未证）——此处只验 RAM 写效果经 `l2ports::shop_money_earn`。
#[test]
fn money_credit_caps_at_ff() {
    let mut s = st();
    s.money = 252;
    shop_money_credit(&mut s);
    assert_eq!(s.money, 255);
    let mut s = st();
    s.money = 253;
    shop_money_credit(&mut s);
    assert_eq!(s.money, 255, "进位封顶（se-4：ADC 进位 ⇒ LDA #$FF）");
    let mut s = st();
    s.money = 100;
    shop_money_credit(&mut s);
    assert_eq!(s.money, 103);
}

/// 负向自证：把实现改坏时必须变红的哨兵。
#[test]
fn shop_purchase_negative_selfproof() {
    // 1) 不足分支若误扣钱必红（价 10、money=9）
    let mut s = st();
    s.money = 9;
    s.weapon.level = 1;
    s.weapon.slot = 1;
    shop_purchase_step(&mut s, BTN_SELECT);
    shop_purchase_step(&mut s, BTN_A);
    assert_ne!(s.money, 0, "不足扣款哨兵：money 不得被扣");
    assert_ne!(s.prompt, None, "不足必须给出提示状态");
    // 2) 满级分支若先扣款后校验必红（money=50、$33=7、已购翼）
    let mut s = st();
    s.money = 50;
    s.weapon.level = 7;
    s.weapon.slot = 3;
    s.pickup.flags = 0x80;
    shop_purchase_step(&mut s, BTN_SELECT);
    shop_purchase_step(&mut s, BTN_A);
    assert_ne!(s.money, 40, "满级不得走扣款路径");
    // 3) 打包若把 type 放低 6 位必红（type=2、qty=6 ⇒ 0x86 非 0x16）
    let mut s = st();
    s.money = 20;
    s.stage = 3;
    s.delivery = [0; 8];
    s.delivery[4] = 0x45;
    shop_purchase_step(&mut s, BTN_DOWN);
    shop_purchase_step(&mut s, BTN_DOWN);
    shop_purchase_step(&mut s, BTN_SELECT);
    shop_purchase_step(&mut s, BTN_RIGHT);
    shop_purchase_step(&mut s, BTN_A);
    shop_purchase_step(&mut s, BTN_RIGHT);
    shop_purchase_step(&mut s, BTN_A);
    assert_ne!(s.delivery[4], 0x16, "打包字段错位哨兵（type<<6|qty）");
    assert_ne!(s.delivery[4], 0x85, "qty 未递增哨兵");
    // 4) 状态构造器签名自证（防止误用 weapons 状态直接当 shop 状态）
    let _w: WeaponState = WeaponState::default();
    let _p: PickupContext = PickupContext::default();
}
