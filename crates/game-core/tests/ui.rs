//! game-core::ui 单测（t77）：BCD 加法器/封顶/阈值/前导零抑制/商店状态机/价目解析 + 负向自证。

use game_core::ui::*;

fn money(d: [u8; 7]) -> MoneyCounter {
    MoneyCounter { digits: d }
}

#[test]
fn money_addend_table_values() {
    // C-money-addend-tbl 逐值（type 1..10 = 10/30/100/150/250/400/600/800/1000/2000）
    let expect = [10u32, 30, 100, 150, 250, 400, 600, 800, 1000, 2000];
    for (t, &want) in expect.iter().enumerate() {
        let mut m = money([0; 7]);
        money_add_event(&mut m, (t + 1) as u8);
        assert_eq!(m.value(), want as u64, "type {} 加数", t + 1);
    }
}

#[test]
fn money_carry_and_cap() {
    // 进位链：90 + 10 = 100
    let mut m = money([0, 0, 0, 0, 0, 9, 0]);
    money_add_event(&mut m, 1);
    assert_eq!(m.value(), 100);
    // 封顶：999999 + 2000 → 999999（$E0-$E5 全 9）
    let mut m = money([9, 9, 9, 9, 9, 9, 0]);
    let capped = money_add_event(&mut m, 10);
    assert!(capped);
    assert_eq!(m.value(), 9999990, "ROM 态：$E0-$E5 全 9、$E6=0（六工作位封顶）");
    // type 0 无操作（ROM BEQ）
    let mut m = money([0; 7]);
    assert!(!money_add_event(&mut m, 0));
    assert_eq!(m.value(), 0);
}

#[test]
fn extend_threshold_and_render() {
    assert!(!extend_check(&money([0, 0, 0, 7, 2, 0, 0])), "7200 未达 8000");
    assert!(extend_check(&money([0, 0, 0, 8, 0, 0, 0])), "8000 达标");
    assert!(extend_check(&money([0, 0, 0, 8, 0, 0, 1])));
    // 前导零抑制（R-money-4）：000000 → 仅个位显示
    let r = render_digits(&money([0; 7]));
    assert_eq!(r.iter().filter(|(_, show)| *show).count(), 1);
    assert!(r[6].1);
    // 004400 → 4 位显示
    let r = render_digits(&money([0, 0, 0, 4, 4, 0, 0]));
    assert_eq!(r.iter().filter(|(_, show)| *show).count(), 4);
}

#[test]
fn shop_state_machine() {
    assert_eq!(ShopState::Main.on_b(), ShopState::Info, "B 切信息页");
    assert_eq!(ShopState::Info.on_b(), ShopState::Exited, "双 B 退出");
    assert_eq!(ShopState::Main.on_start(), ShopState::Main, "Start 无效（R-shop-8）");
    assert_eq!(ShopState::Info.on_start(), ShopState::Info);
    assert_eq!(ShopState::Exited.on_b(), ShopState::Exited);
}

#[test]
fn price_records_from_rom() {
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM 可读");
    let prices = parse_price_records(&rom);
    assert_eq!(prices, [Some(10), Some(6), Some(4), None], "R-shop-1 四槽价目（槽4 FF 截断）");
}

#[test]
fn ui_negative_selfproof() {
    // 负向自证 1：加法器结果错位必红（90+10 ≠ 110）
    let mut m = money([0, 0, 0, 0, 0, 9, 0]);
    money_add_event(&mut m, 1);
    assert_ne!(m.value(), 110, "进位错位哨兵");
    // 负向自证 2：Start 误退必红
    assert_ne!(ShopState::Main.on_start(), ShopState::Exited);
    // 负向自证 3：封顶溢出不得回绕（999999+2000 ≠ 199999/0）
    let mut m = money([9, 9, 9, 9, 9, 9, 0]);
    money_add_event(&mut m, 10);
    assert_eq!(m.value(), 9999990, "封顶回绕哨兵（ROM 态 $E0-$E5 全 9）");
    // 负向自证 4：金钱消费侧未证（不得实现）
    assert!(money_spend(&mut money([0; 7]), 10).is_none(), "卖出/消费 TODO 不发明");
}
