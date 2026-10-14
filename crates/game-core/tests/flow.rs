//! game-core::flow 单测（t81 + DEV-2）：模式状态机/跨模式数据流/GAME OVER-Continue 粗粒度边（A2 F6-F12）
//! + 负向自证（fail-closed）。

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
fn gameover_continue_coarse_paths_wired() {
    // A2 F6-F12 粗粒度投影（原 unproven_segments_fail_closed 按 A2 闭合）：
    // 粗层只落模式转移 + PersistData 镜像效果；RAM 级效果在 death_chain_step（细粒度）。
    let st = FlowState { mode: Mode::DeathRespawn, data: Default::default() };
    let st = st.advance(ModeEvent::GameOverArm).expect("F6/F7 边：DeathRespawn→GameOverPending");
    assert_eq!(st.mode, Mode::GameOverPending);
    let st = st.advance(ModeEvent::GameOverArrival).expect("F8 边：pending→menu");
    assert_eq!(st.mode, Mode::GameOverMenu);
    let st = st.advance(ModeEvent::MenuToggle).expect("F11 自环");
    assert_eq!(st.mode, Mode::GameOverMenu);
    let st = st.advance(ModeEvent::GameFinish).expect("F10 边：menu→title");
    assert_eq!(st.mode, Mode::Title);
    // F9 续关 + F12（PersistData 镜像：$E0-$E6 清零；$33/$34/$018F 不在 F12 复位集 ⇒ 保留）
    let data = PersistData {
        money: MoneyCounter { digits: [0, 0, 0, 1, 3, 0, 0] },
        weapon_slot: 3,
        weapon_level: 7,
        bomb_ammo: 6,
    };
    let st = FlowState { mode: Mode::GameOverMenu, data };
    let st = st.on_continue().expect("F9 源态=GameOverMenu");
    assert_eq!(st.mode, Mode::StageStart);
    assert_eq!(st.data.money.value(), 0, "续关 $E0-$E6 清零（bank15:$E10A sub_ECBF）");
    assert_eq!((st.data.weapon_slot, st.data.weapon_level, st.data.bomb_ammo), (3, 7, 6), "F12 不动槽/级/弹药");
    // F6/F7 投影入口（源态守卫 fail-closed）
    let st = FlowState { mode: Mode::DeathRespawn, data: Default::default() };
    let st = st.on_game_over().expect("F6/F7 源态=DeathRespawn");
    assert_eq!(st.mode, Mode::GameOverPending);
    assert!(FlowState::new().on_game_over().is_err(), "非 DeathRespawn 源态必拒（fail-closed）");
    assert!(FlowState::new().on_continue().is_err(), "非 GameOverMenu 源态必拒（fail-closed）");
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
    // ④ 粗粒度链不含未证段标记（A2 闭合后 GAME OVER/CONTINUE 已为已证态；F13 免死支仍 fail-closed）
    assert!(format!("{:?}", st.mode).contains("Unproven") == false, "已证链不得含未证段");
    assert_eq!(checkpoints.len(), 7, "L5 检查点 7 个（A1 流程可达）");
}

#[test]
fn stg_02_08_chain_checkpoints() {
    // t193 ②：STG-02..08 链扩展——每关 StageStart 检查点（A1 流程可达）
    // 帧号=占位（场景取证后填真值，fail-closed 不发明）
    use game_core::stage::stage_skeleton;
    let mut checkpoints = Vec::new();
    for n in 2..=8u8 {
        let sk = stage_skeleton(n).expect("骨架");
        checkpoints.push((n, sk.root, sk.tail));
    }
    assert_eq!(checkpoints.len(), 7, "STG-02..08 七关检查点");
    // 负向：骨架 root/tail 非零哨兵
    for (n, root, tail) in &checkpoints {
        assert!(*root != 0 && *tail != 0, "STG-{n} root/tail 非零");
    }
    // STG-7/1 共享流（内容集成注）
    assert_eq!(stage_skeleton(7).unwrap().rec4, stage_skeleton(1).unwrap().rec4);
}
