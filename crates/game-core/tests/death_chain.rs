//! game-core::flow 死亡/GAME OVER/Continue 链集成测试（DEV-2，A2 F1-F13）。
//!
//! 证据基准：`docs/findings/game-over-continue.md`（§1 死亡语义链 / §2 GAME OVER 触发链 /
//! §3 Continue 机制 / §4 边集）；trace 锚 = gameplay-stage1 f1787（四写）/ f1788（trampoline+结算）/
//! f1915（同帧 0F→0→1）。先测试后实现（TDD）；每用例注记期望 RAM 终态。
//! 语义口径：F5 = dead 态**首帧**结算（非复活边）——任务测试清单“窗末结算”以 F5 更正为准，
//! 本文件双断言（首帧结算 + 窗末/复活边不再扣）锁死两个方向的突变。

use game_core::flow::*;
use game_core::ui::MoneyCounter;

/// 存活初值（stage 开局：`$0400:=1`、`$018D:=2`，bank15:$E0EF/$E1B6）。
fn alive() -> DeathChainState {
    DeathChainState::new()
}

/// 推进一个死亡窗全程：转移帧 + 127 个窗内帧 = 128f（f1787..f1914，$0400 恒 0x0F）。
fn run_death_window(st: &mut DeathChainState) {
    death_chain_step(st, true, 0); // 转移帧（四写签名）
    for _ in 0..127 {
        death_chain_step(st, false, 0); // 窗内行为帧
    }
}

/// ①死亡转移：`$0400` 1→0x0F + 四写签名态（trace f1787/f2046 严格双事件吻合）。
/// 期望 RAM 终态：`$0400=0x0F`、`$041C=0`、`$04A8=1`、`$0668|=0x10`；本帧不结算（F5 在下一帧）。
#[test]
fn f1_death_transition_four_write_signature() {
    let mut st = alive();
    st.state_vec_hi = 0xA0; // 常态行为向量（trace f1786 `$041C=A0`）
    let fx = death_chain_step(&mut st, true, 0);
    assert_eq!(st.phase, DeathPhase::Dead);
    assert_eq!(st.player_state, 0x0F, "$0400：1→0x0F（$D86B[1]=0x0F，bank14:$D621）");
    assert_eq!(st.state_vec_hi, 0x00, "$041C 清零（bank14:$D62A）");
    assert_eq!(st.old_state, 0x01, "$04A8 旧态备份（bank14:$D625）");
    assert_eq!(st.hit_flags & 0x10, 0x10, "$0668 |= $10（bank14:$D64C-D651）");
    assert_eq!(st.window_frames, 1, "转移帧=窗帧 1（f1787 起算）");
    assert_eq!(fx, DeathFx::None, "转移帧不结算不发声（trampoline f1788 才装）");
    assert_eq!((st.lives, st.shop_money), (2, 0), "转移帧不动 $018D/$018E");
}

/// ②128f 窗行为：冻结（输入采集无效 + 复撞免疫 F2）+ 帧计数（f1787-1914 = 128f）。
/// 期望 RAM 终态：窗内 `$0400` 恒 0x0F、`$04A8/$0668` 不因复撞重写、其余字段冻结。
#[test]
fn f2_window_freeze_immune_and_frame_count() {
    let mut st = alive();
    st.bomb_ammo = 9;
    st.stage = 5;
    death_chain_step(&mut st, true, 0);
    for _ in 0..127 {
        // 每帧复撞 + 全按键（F2 恒等转移零写入；冻结窗输入采集无效）
        let fx = death_chain_step(&mut st, true, 0xFF);
        assert_eq!(st.phase, DeathPhase::Dead, "窗内不得转移");
        assert_eq!(st.player_state, 0x0F, "$0400 恒 0x0F（窗内）");
        assert_eq!(st.old_state, 0x01, "复撞免疫：$04A8 不重写（$D86B[0x0F] 恒等→BEQ 跳过）");
        assert_eq!(st.bomb_ammo, 9, "冻结：输入不动 $018F");
        assert_eq!(st.stage, 5, "冻结：输入不动 $180");
        assert_ne!(fx, DeathFx::MenuTextPending, "冻结窗无菜单效果");
    }
    assert_eq!(st.window_frames, 128, "窗长 128f（gameplay-stage1 双事件实测）");
    assert_eq!(st.hit_flags & 0x10, 0x10, "复撞免疫：$0668 不清不重置");
}

/// ③结算（非终死）：`$018D` BCD −1 + `$018E` ÷2 —— dead 态**首帧**一次性（F5）。
/// 期望 RAM 终态：首帧后 `lives=0x01`、`shop_money=0x31`（99→49）、自向量 `$A3:$A5`；
/// 窗末/复活边不再扣（“窗末结算”方向的突变必红）。
#[test]
fn f5_death_cost_settles_once_at_dead_first_frame() {
    let mut st = alive();
    st.lives = 0x02;
    st.shop_money = 0x63; // 99
    death_chain_step(&mut st, true, 0); // 转移帧（f1787）：仅四写
    assert_eq!((st.lives, st.shop_money), (0x02, 0x63), "转移帧不结算（f1787 无 trampoline）");
    assert_eq!((st.state_vec_hi, st.state_vec_lo), (0x00, 0x00), "f1787 `$041C=0`");
    let fx = death_chain_step(&mut st, false, 0); // dead 首行为帧（f1788）
    assert_eq!(st.lives, 0x01, "$018D BCD −1（bank03:$A392 LDY #1/JSR sub_ED03）");
    assert_eq!(st.shop_money, 0x31, "$018E ÷2（bank03:$A39F LSR，99→49）");
    assert_eq!(
        (st.state_vec_hi, st.state_vec_lo),
        (0xA3, 0xA5),
        "自向量 trampoline `$A3:$A5`（bank03:$A3A2 JSR $CB4F；f1788 起常驻）"
    );
    assert_eq!(fx, DeathFx::DeathSoundPending, "死亡音效 $21 挂起（bank03:$A39A）");
    // 一次性：越过窗末与复活边不再扣（F5=首帧口径，非复活边）
    for _ in 0..200 {
        death_chain_step(&mut st, false, 0);
    }
    assert_eq!((st.lives, st.shop_money), (0x01, 0x31), "结算一次性（窗末/复活边不再扣）");
}

/// ④终死跳过减一：`$018D==0` ⇒ `$26|=8`（不可复活位）+ 不减一（bank03:$A390 BNE）。
/// 期望 RAM 终态：`lives=0x00`、`revive_flags|=0x08`、`shop_money` 仍减半（A39F 终死同路）。
#[test]
fn f6_terminal_death_arms_without_decrement() {
    let mut st = alive();
    st.lives = 0x00;
    st.shop_money = 0x0A;
    st.revive_flags = 0x00;
    death_chain_step(&mut st, true, 0);
    let fx = death_chain_step(&mut st, false, 0);
    assert_eq!(st.lives, 0x00, "终死跳过减一（bank03:$A390 BNE loc_A39A）");
    assert_eq!(st.revive_flags & 0x08, 0x08, "$26 bit3=不可复活位（bank03:$A387-A38E）");
    assert_eq!(st.shop_money, 0x05, "$018E 仍减半（bank03:$A39F 终死同路）");
    assert_eq!(st.phase, DeathPhase::Dead, "armed 后死窗继续");
    assert_eq!(fx, DeathFx::DeathSoundPending);
}

/// ⑤复活分叉（`$26&8==0`）：`$018F := max(4,$18F)`、`$0400:=1`、`$0710:=1`（bank15:$E2A2-E2BC）。
/// 期望 RAM 终态：ammo<4 回满 4；ammo≥4 保留（max 语义）；`$0400=1`；`$0710=1`。
#[test]
fn f4_respawn_fork_tops_up_bomb_ammo() {
    // ammo=2 → 回满 4（trace f2174 3→4 同族）
    let mut st = alive();
    st.bomb_ammo = 2;
    run_death_window(&mut st);
    let fx = death_chain_step(&mut st, false, 0); // 窗尽帧（f1915）：同帧释放+分叉
    assert_eq!(st.phase, DeathPhase::Alive);
    assert_eq!(st.player_state, 1, "$0400:=1（bank15:$E2B4）");
    assert_eq!(st.bomb_ammo, 4, "$018F := max(4,·)（bank15:$E2A8-E2B1）");
    assert_eq!(st.obj_grid_tail[0x012], 1, "$0710:=1（bank15:$E2B9；语义开放 R-G3）");
    assert_eq!(st.state_vec_hi, 0x00, "释放写 `$041C:=0`（bank14:$D111）");
    assert_eq!(st.hit_flags, 0x00, "释放写 `$0668:=0`（bank14:$D11A）");
    assert_eq!(fx, DeathFx::None);
    // ammo≥4 保留（max 语义，非恒等回 4）
    let mut st = alive();
    st.bomb_ammo = 7;
    run_death_window(&mut st);
    death_chain_step(&mut st, false, 0);
    assert_eq!(st.bomb_ammo, 7, "max(4,7)=7（bank15:$E2AB BCS 跳过回满）");
}

/// ⑥GAME OVER 分叉（`$26&8≠0`）：F7 一次性入场 → F8 Start 进菜单。
/// 期望 RAM 终态：`$27|=0x40`、`$188=0xFF`、`$0300=0`；Start 后 `$2F:=0`、`$017A-$017F` 减半。
#[test]
fn f7_gameover_fork_menu_entry() {
    let mut st = alive();
    st.lives = 0x00; // 终死 ⇒ settle 置 `$26|=8`
    st.entry_bytes = [0x3F, 0x21, 0xC0, 0xFF, 0x00, 0x60];
    run_death_window(&mut st);
    let fx = death_chain_step(&mut st, false, 0); // 窗尽：F7 分叉进 pending
    assert_eq!(st.phase, DeathPhase::GameOverPending);
    assert_eq!(st.entry_once_flags & 0x40, 0x40, "$27 bit6 一次性入场（bank15:$E2C1）");
    assert_eq!(st.intro_counter, 0xFF, "$188:=$FF（bank15:$E2D2）");
    assert_eq!(st.stage_advance_gate, 0x00, "$0300:=0（bank15:$E2D7）");
    assert_eq!(fx, DeathFx::GameOverSoundPending, "GAME OVER 音效 $11 挂起（bank15:$E2DD）");
    // 等待 Start（bank15:$E2ED 停留）
    let fx = death_chain_step(&mut st, false, 0);
    assert_eq!(st.phase, DeathPhase::GameOverPending, "无 Start 不转场");
    assert_eq!(fx, DeathFx::None);
    // F8：Start → 菜单（bank15:$E2F0 → $E3AB）
    let fx = death_chain_step(&mut st, false, MENU_BTN_START);
    assert_eq!(st.phase, DeathPhase::GameOverMenu);
    assert_eq!(st.menu_cursor, 0x00, "$2F:=0（bank15:$E403-E405）");
    assert_eq!(fx, DeathFx::MenuTextPending, "CONTINUE/GAME OVER 文本装载挂起（bank15:$E418）");
    // 菜单入口 $017A-$017F 逐字节减半（bank15:$E3B9-E3CD：保 bit7-6、6 位量÷2 存 5 位）
    assert_eq!(
        st.entry_bytes,
        [0x1F, 0x10, 0xC0, 0xDF, 0x00, 0x50],
        "$017A-$017F 减半（丢 bit0、bit5→bit4；bit7-6 保留）"
    );
}

/// ⑦菜单双出口：`$2F==0`+Start → 续关（F9）；`$2F==1`+Start → 回标题（F10）。
/// 期望 RAM 终态：续关回 `Alive`；回标题 `$180:=1` + `$017A-$017F` 清零（bank15:$E09A/$E0D1）。
#[test]
fn f9_f10_menu_dual_exit() {
    // (a) $2F==0 → 续关
    let mut st = alive();
    st.lives = 0;
    run_death_window(&mut st);
    death_chain_step(&mut st, false, 0); // F7 pending
    death_chain_step(&mut st, false, MENU_BTN_START); // F8 菜单
    let fx = death_chain_step(&mut st, false, MENU_BTN_START);
    assert_eq!(st.phase, DeathPhase::Alive, "F9 续关回关首 alive（bank15:$E481 → sub_E0D9）");
    assert_eq!(fx, DeathFx::None);
    // (b) $2F==1 → 回标题
    let mut st = alive();
    st.lives = 0;
    st.entry_bytes = [0x21, 0, 0, 0, 0, 0];
    st.stage = 7;
    run_death_window(&mut st);
    death_chain_step(&mut st, false, 0);
    death_chain_step(&mut st, false, MENU_BTN_START); // 菜单（$2F:=0）
    death_chain_step(&mut st, false, MENU_BTN_SELECT); // 翻到 $2F=1
    assert_eq!(st.menu_cursor, 1);
    let fx = death_chain_step(&mut st, false, MENU_BTN_START);
    assert_eq!(st.phase, DeathPhase::Title, "F10 回标题（bank15:$E484 → loc_E073）");
    assert_eq!(st.stage, 1, "$180:=1（bank15:$E09A-E09C）");
    assert_eq!(st.entry_bytes, [0; 6], "$017A-$017F 清零（bank15:$E0D1-E0D7）");
    assert_eq!(fx, DeathFx::None);
}

/// ⑧Continue 复位集（F12）：双金钱清零（`$E0-$E6`+`$018E`）+ 网格清（`$6FE-$7FD`）+ 仅 `$180` 保留。
/// 期望 RAM 终态：`money_counter=0`、`shop_money=0`、`lives=2`、`$0400=1`、网格清
/// （`$7FE-$7FF` 保留，AD66-1）、`$180` 与 `$017A-$017F`（减半值）带入。
#[test]
fn f12_continue_reset_set() {
    let mut st = alive();
    st.lives = 0;
    st.shop_money = 0x30;
    st.money_counter = MoneyCounter { digits: [0, 0, 1, 2, 3, 4, 5] };
    st.stage = 7;
    st.entry_bytes = [0x3F, 0x21, 0, 0, 0, 0];
    st.bomb_ammo = 6;
    st.bomb_type_index = 3;
    st.todo_0189 = 0x55;
    st.todo_c8 = 0x44;
    st.intro_counter = 0xFF;
    st.revive_flags = 0x08; // armed（终死过）
    st.old_state = 0x77;
    st.hit_flags = 0x10;
    st.state_vec_hi = 0xA3;
    st.state_vec_lo = 0xA5;
    st.obj_grid_tail.fill(0xAA);
    run_death_window(&mut st);
    death_chain_step(&mut st, false, 0); // F7
    death_chain_step(&mut st, false, MENU_BTN_START); // F8（此处 $017A-$017F 减半）
    assert_eq!(st.entry_bytes[0], 0x1F, "入口减半先行（E3B9）");
    let fx = death_chain_step(&mut st, false, MENU_BTN_START); // F9 续关
    assert_eq!(st.phase, DeathPhase::Alive);
    assert_eq!(fx, DeathFx::None);
    // 双金钱清零
    assert_eq!(st.money_counter, MoneyCounter::default(), "$E0-$E6 全 7 字节清零（bank15:$E10A sub_ECBF）");
    assert_eq!(st.shop_money, 0, "$018E 清零（bank15:$E168）");
    // 复位集
    assert_eq!(st.lives, 2, "$18D:=2（bank15:$E0EF）");
    assert_eq!(st.todo_0189, 0, "$0189 清零（bank15:$E162）");
    assert_eq!(st.bomb_type_index, 0, "$0190 清零（bank15:$E165）");
    assert_eq!(st.player_state, 1, "$0400:=1（bank15:$E1B6）");
    assert_eq!(st.intro_counter, 0x40, "$188:=$40（bank15:$E1DD）");
    assert_eq!(st.entry_once_flags, 0, "$27 清零（bank15:$E1C3）——再入场可重 arm");
    assert_eq!(st.todo_c8, 0, "$C8 清零（bank15:$E19B）");
    assert_eq!(st.revive_flags & 0x08, 0, "$26 bit3 清零=复活权恢复（bank15:$E0F4-E0F8）");
    assert_eq!(st.revive_flags & 0x10, 0x10, "$26 bit4 置位（同上）");
    // 网格清（$0400-$06FF + $6FE-$7FD）；$7FE-$7FF 保留（AR63-5/AD66-1）
    assert_eq!(st.old_state, 0, "网格具名单元清（$04A8）");
    assert_eq!(st.hit_flags, 0, "网格具名单元清（$0668）");
    assert_eq!(st.state_vec_lo, 0, "网格具名单元清（$0438）");
    assert_eq!(st.obj_grid_tail[0x000], 0, "$6FE 清（E18C 第四页 STA $6FE,X）");
    assert_eq!(st.obj_grid_tail[0x0FF], 0, "$7FD 清（清零域尾端）");
    assert_eq!(st.obj_grid_tail[0x012], 0, "$0710 清（清零域内）");
    assert_eq!(st.obj_grid_tail[0x100], 0xAA, "$7FE 保留（AD66-1：循环不清）");
    assert_eq!(st.obj_grid_tail[0x101], 0xAA, "$7FF 保留（AD66-1）");
    // 仅 $180 保留 + $017A-$017F 减半后带入
    assert_eq!(st.stage, 7, "$180 关号保留（续当前关）");
    assert_eq!(st.entry_bytes[0], 0x1F, "$017A-$017F 减半后带入（§3.2）");
    assert_eq!(st.bomb_ammo, 6, "$018F 不在 F12 复位集（保留）");
}

/// F11 菜单翻转：Select/Up(仅 $2F=1)/Down(仅 $2F=0) ⇒ `$2F^=1` + 音效 $45 挂起（bank15:$E43E-E460）。
#[test]
fn f11_menu_toggle_flips_cursor() {
    let mut st = alive();
    st.lives = 0;
    run_death_window(&mut st);
    death_chain_step(&mut st, false, 0);
    death_chain_step(&mut st, false, MENU_BTN_START); // 菜单，$2F=0
    // Select 翻转
    let fx = death_chain_step(&mut st, false, MENU_BTN_SELECT);
    assert_eq!((st.menu_cursor, fx), (1, DeathFx::MenuToggleSoundPending), "E458 `$2F^=1` + 音效 $45");
    // Down 在 $2F=1 时无效（E456 BNE 跳过）
    let fx = death_chain_step(&mut st, false, MENU_BTN_DOWN);
    assert_eq!((st.menu_cursor, fx), (1, DeathFx::None), "Down 仅 $2F==0 翻转");
    // Up 在 $2F=1 时翻转
    let fx = death_chain_step(&mut st, false, MENU_BTN_UP);
    assert_eq!((st.menu_cursor, fx), (0, DeathFx::MenuToggleSoundPending));
    // Up 在 $2F=0 时无效（E44C 不跳转→落 Down 检查）
    let fx = death_chain_step(&mut st, false, MENU_BTN_UP);
    assert_eq!((st.menu_cursor, fx), (0, DeathFx::None));
    // Down 在 $2F=0 时翻转
    let fx = death_chain_step(&mut st, false, MENU_BTN_DOWN);
    assert_eq!((st.menu_cursor, fx), (1, DeathFx::MenuToggleSoundPending));
}

/// fail-closed 负向自证：未证域输入不动作——F13 免死支（`$06A0≠0`/`$32` bit7）未接线
/// （findings §4.2 建议暂挂）；菜单/待决态的碰撞命中无已证转移 ⇒ 不动作。
#[test]
fn fail_closed_unproven_domains() {
    // GAME OVER 菜单态：碰撞命中（免死支/受击语义未接）⇒ 不动作
    let mut st = alive();
    st.lives = 0;
    run_death_window(&mut st);
    death_chain_step(&mut st, false, 0);
    death_chain_step(&mut st, false, MENU_BTN_START);
    assert_eq!(st.phase, DeathPhase::GameOverMenu);
    let before = st;
    let fx = death_chain_step(&mut st, true, 0);
    assert_eq!(st, before, "菜单态命中=未证域 fail-closed 不动作");
    assert_eq!(fx, DeathFx::None);
    // 待决态：碰撞命中同样不动作（F13 免死支不入行为断言）
    let mut st = alive();
    st.lives = 0;
    run_death_window(&mut st);
    death_chain_step(&mut st, false, 0); // F7 → pending
    assert_eq!(st.phase, DeathPhase::GameOverPending);
    let before = st;
    let fx = death_chain_step(&mut st, true, 0);
    assert_eq!(st, before, "pending 态命中=未证域 fail-closed 不动作");
    assert_eq!(fx, DeathFx::None);
}
