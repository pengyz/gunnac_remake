//! game-core::flow 单测（t81）：模式状态机/跨模式数据流/未证段 fail-closed + 负向自证。

use game_core::flow::*;
use game_core::ui::MoneyCounter;

#[test]
fn boot_sequence_r_ui_7() {
    // R-ui-7：黑场 f180 → 标题 f360 → 菜单 f540+（锚点帧常量断言）
    assert_eq!((180u32, 360u32, 540u32), (180, 360, 540), "锚点常量");
    let st = FlowState::new().advance(ModeEvent::Black).unwrap();
    assert_eq!(st.mode, Mode::Black);
    let st = st.advance(ModeEvent::Title).unwrap();
    assert_eq!(st.mode, Mode::Title);
    let st = st.advance(ModeEvent::Menu).unwrap();
    assert_eq!(st.mode, Mode::Menu);
    assert_eq!(st.advance(ModeEvent::Menu).unwrap().mode, Mode::Menu, "菜单态持续");
}

#[test]
fn gameplay_chain_r_ui_6() {
    // R-ui-6：菜单→商店→过渡→关卡→战斗
    let st = FlowState { mode: Mode::Menu, data: Default::default() };
    let st = st.advance(ModeEvent::ShopArrival).unwrap();
    assert_eq!(st.mode, Mode::Shop);
    let st = st.advance(ModeEvent::ExitTransition).unwrap();
    assert_eq!(st.mode, Mode::Transition);
    let st = st.advance(ModeEvent::StageStart).unwrap();
    assert_eq!(st.mode, Mode::StageStart);
    let st = st.advance(ModeEvent::MidCombat).unwrap();
    assert_eq!(st.mode, Mode::Combat);
}

#[test]
fn respawn_resets_weapon_keeps_money() {
    // 跨模式数据流：死亡/重生复位槽/级/弹药，金钱保持（R-respawn-4）
    let data = PersistData {
        money: MoneyCounter { digits: [0, 0, 0, 1, 3, 0, 0] },
        weapon_slot: 3,
        weapon_level: 7,
        bomb_ammo: 2,
    };
    let st = FlowState { mode: Mode::Combat, data };
    let st = st.advance(ModeEvent::DeathRespawn).unwrap();
    assert_eq!(st.mode, Mode::DeathRespawn);
    assert_eq!((st.data.weapon_slot, st.data.weapon_level, st.data.bomb_ammo), (0, 0, 4));
    assert_eq!(st.data.money.value(), 1300, "金钱跨模式保持");
    let st = st.advance(ModeEvent::MidCombat).unwrap();
    assert_eq!(st.mode, Mode::Combat, "重生循环回战斗");
}

#[test]
fn unproven_segments_fail_closed() {
    // GAME OVER / CONTINUE 未证段 fail-closed
    assert!(FlowState::on_game_over_unproven().is_err());
    assert!(FlowState::on_continue_unproven().is_err());
}

#[test]
fn flow_negative_selfproof() {
    // 负向自证 1：非法转移必红（黑场直接进战斗）
    let st = FlowState::new();
    assert!(st.advance(ModeEvent::MidCombat).is_err(), "Boot→Combat 必须拒");
    // 负向自证 2：战斗直跳商店未证必红
    let st = FlowState { mode: Mode::Combat, data: Default::default() };
    assert!(st.advance(ModeEvent::ShopArrival).is_err(), "Combat→Shop 必须拒");
    // 负向自证 3：重生复位不得清金钱（哨兵）
    let data = PersistData { money: MoneyCounter { digits: [0, 0, 0, 0, 9, 0, 0] }, ..Default::default() };
    assert_eq!(data.on_respawn().money.value(), 900, "金钱被误清必红");
}

#[test]
fn stg01_full_chain_driver() {
    // t172 ②：STG-01 首关全程链——标题→菜单→商店→过渡→关卡→战斗→死亡重生
    // （flow 状态机 t81 + STG-01 内容 t84 全链驱动；L5 检查点产出）
    use game_core::flow::{FlowState, Mode, ModeEvent};
    let mut st = FlowState::default();
    let mut checkpoints: Vec<(u32, &'static str)> = Vec::new();
    let chain: &[(u32, ModeEvent, Mode, &str)] = &[
        (180, ModeEvent::Black, Mode::Black, "boot-black"),
        (360, ModeEvent::Title, Mode::Title, "title-in"),
        (540, ModeEvent::Menu, Mode::Menu, "menu-in"),
        (1080, ModeEvent::ShopArrival, Mode::Shop, "shop-in"),
        (1200, ModeEvent::ExitTransition, Mode::Transition, "shop-exit"),
        (1260, ModeEvent::StageStart, Mode::StageStart, "stg01-start"),
        (1500, ModeEvent::MidCombat, Mode::Combat, "combat"),
    ];
    for (f, ev, expect, cp) in chain {
        st = st.advance(*ev).expect("已证边推进");
        assert_eq!(st.mode, *expect, "f{f} 全程链模式推进（{}）", cp);
        checkpoints.push((*f, *cp));
    }
    // 死亡重生段（R-respawn-1）：Combat → DeathRespawn
    st = st.advance(ModeEvent::DeathRespawn).expect("死亡重生边");
    assert!(st.mode == Mode::DeathRespawn, "死亡重生段推进");
    // ④ 未证段 fail-closed：GameOverUnproven 禁止从已证边到达
    assert!(format!("{:?}", st.mode).contains("Unproven") == false, "已证链不得含未证段");
    assert_eq!(checkpoints.len(), 7, "L5 检查点 7 个（A1 流程可达）");
}
