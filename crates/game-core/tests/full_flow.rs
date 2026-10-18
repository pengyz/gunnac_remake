//! 游戏宿主状态机 harness（shell 契约测试面 ②——2026-10-18 lead 令）。
//!
//! title→八关→结局 全流程驱动测试（game-core flow 已证面）：Mode/ModeEvent 状态机
//! 全链——Boot→Black→Title→Menu→ShopArrival→Shop→(出店→关始→战斗→清关→推进)×8→
//! Ending。段锚四连序（shop/start/combat/clear）逐关断言；结局入口=AllClear
//! （StageClear→Ending=`$180==9`→`$95F9` 语义，l5-all-clear R-l5-4）。
//! 边序=flow.rs 转移表实证（未证转移 fail-closed 不发明）。

use game_core::flow::{FlowState, Mode, ModeEvent};

/// 单关段链（st 须处于 Shop 态——出店→关始→战斗→清关）。段锚四连序内嵌断言。
fn drive_stage(st: FlowState, stage: u8) -> FlowState {
    let st = st.advance(ModeEvent::ExitTransition).expect("出店转场");
    assert_eq!(st.mode, Mode::Transition, "关 {stage} 出店段（Transition）");
    let st = st.advance(ModeEvent::StageStart).expect("关始");
    assert_eq!(st.mode, Mode::StageStart, "关 {stage} 开始段锚");
    let st = st.advance(ModeEvent::MidCombat).expect("战斗");
    assert_eq!(st.mode, Mode::Combat, "关 {stage} 战斗段锚");
    let st = st.advance(ModeEvent::StageClear).expect("清关");
    assert_eq!(st.mode, Mode::StageClear, "关 {stage} 清关段锚");
    st
}

#[test]
fn full_flow_title_to_ending() {
    let mut st = FlowState::default();
    assert_eq!(st.mode, Mode::Boot, "开机初始化");
    st = st.advance(ModeEvent::Black).expect("黑场");
    st = st.advance(ModeEvent::Title).expect("标题");
    st = st.advance(ModeEvent::Menu).expect("菜单");
    st = st.advance(ModeEvent::ShopArrival).expect("首店到达");
    assert_eq!(st.mode, Mode::Shop, "关 1 商店主面板");

    // 八关段链：关 1 起 Shop 态驱动；StageClear→(StageAdvance→Shop|AllClear→Ending)
    for stage in 1..=8u8 {
        st = drive_stage(st, stage);
        if stage < 8 {
            st = st.advance(ModeEvent::StageAdvance).expect("关推进");
            assert_eq!(st.mode, Mode::Shop, "关 {} 商店主面板（推进后）", stage + 1);
        }
    }

    // 结局入口（StageClear→AllClear→Ending=$180==9 语义）
    st = st.advance(ModeEvent::AllClear).expect("全清");
    assert_eq!(st.mode, Mode::Ending, "八关后结局段");
}

#[test]
fn full_flow_death_respawn_loop() {
    // 死亡/重生循环段（R-respawn-1）：Combat→DeathRespawn
    let mut st = FlowState::default();
    st = st.advance(ModeEvent::Black).ok().unwrap();
    st = st.advance(ModeEvent::Title).ok().unwrap();
    st = st.advance(ModeEvent::Menu).ok().unwrap();
    st = st.advance(ModeEvent::ShopArrival).ok().unwrap();
    st = st.advance(ModeEvent::ExitTransition).ok().unwrap();
    st = st.advance(ModeEvent::StageStart).ok().unwrap();
    st = st.advance(ModeEvent::MidCombat).ok().unwrap();
    assert_eq!(st.mode, Mode::Combat);
    st = st.advance(ModeEvent::DeathRespawn).expect("死亡重生边");
    assert_eq!(st.mode, Mode::DeathRespawn);
}

#[test]
fn full_flow_gameover_continue_chain() {
    // GAME OVER 链（F6-F11）：DeathRespawn→GameOverArm→GameOverArrival→Continue
    let mut st = FlowState::default();
    st = st.advance(ModeEvent::Black).ok().unwrap();
    st = st.advance(ModeEvent::Title).ok().unwrap();
    st = st.advance(ModeEvent::Menu).ok().unwrap();
    st = st.advance(ModeEvent::ShopArrival).ok().unwrap();
    st = st.advance(ModeEvent::ExitTransition).ok().unwrap();
    st = st.advance(ModeEvent::StageStart).ok().unwrap();
    st = st.advance(ModeEvent::MidCombat).ok().unwrap();
    st = st.advance(ModeEvent::DeathRespawn).ok().unwrap();
    st = st.advance(ModeEvent::GameOverArm).expect("F6/F7 边");
    assert_eq!(st.mode, Mode::GameOverPending);
    st = st.advance(ModeEvent::GameOverArrival).expect("F8 边");
    assert_eq!(st.mode, Mode::GameOverMenu);
    st = st.advance(ModeEvent::Continue).expect("继续边");
    assert_eq!(st.mode, Mode::StageStart, "Continue→关始（实证边）");
}
