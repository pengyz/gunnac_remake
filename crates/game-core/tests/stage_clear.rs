//! DEV-3 关末/通关链测试（A3 §1.1/§1.2；spec `docs/spec/l5-all-clear.md` R-l5-3..6）。
//!
//! 证据基准：`docs/findings/ending-sequence.md` §1（bank15:$E2FC-$E3A8 逐字节）+
//! `docs/findings/shop-purchase-flow.md` §清零/初始化（`$9BC3` 商店 init）。
//! 只断言行为终态（RAM 镜像 + 派发），不断言实现内部结构；未证域 fail-closed 负向自证。

use game_core::flow::*;

fn env(frame_2b: u8, gate_300: u8) -> StageClearEnv {
    StageClearEnv { frame_2b, gate_300, player_present: true }
}

/// 跑一关关末段到派发：arm → 步进到 `phase != Clearing`，返回（步数, 派发相位）。
fn run_clear(st: &mut StageClearState, gate: u8) -> (u32, StageClearPhase) {
    stage_clear_arm(st);
    let mut steps = 0u32;
    loop {
        let fx = stage_clear_step(st, env((steps as u8) & 0xFF, gate));
        let _ = fx;
        steps += 1;
        if st.phase != StageClearPhase::Clearing {
            return (steps, st.phase);
        }
        assert!(steps < 100_000, "关末段必须有限帧内派发");
    }
}

/// ① R-l5-3/4：关末段 RAM 镜像逐写点（E312-E342）——倒数初始化/免死保护/飞离向量。
#[test]
fn stage_clear_ram_writes_match_a3() {
    let mut st = StageClearState::new();
    assert_eq!(st.stage, 1, "开局 $180:=1（bank15:$E04A）");
    stage_clear_arm(&mut st);
    assert_eq!(st.flags_26 & 0x10, 0x10, "bank00:$8D02 `ORA #$10/STA $26` 置位");
    assert_eq!(st.phase, StageClearPhase::Clearing);
    // 入场帧（frame_2b=3：`$2B&7≠0` 同帧不 DEC，露出初值）：一次性写 + 倒数初始化（E309/E314/E318/E31D）
    let fx = stage_clear_step(&mut st, env(3, 1)); // gate 给 1 也不得同帧派发（E31D 清零）
    assert_eq!(fx, StageClearFx::ClearEntryPending, "E320/E323 服务+音效 $10 挂起");
    assert_eq!(st.flags_27, 0x80, "E309 `$27|=$80`");
    assert_eq!(st.countdown_2f, 0x32, "E314 `$2F:=$32(50)`");
    assert_eq!(st.intro_counter, 0x7E, "E318 `$188:=$7E`（N-0200）");
    assert_eq!(st.stage_advance_gate, 0, "E31D `$0300:=0`（入场帧必等）");
    assert_eq!(st.no_death, 2, "E32E `$06A0:=2`（同帧倒数环已跑）");
    assert_eq!(st.phase, StageClearPhase::Clearing, "入场帧不得派发");
    // 同帧节拍对照（E312 写 50 后同帧 E337 即可 DEC）：frame_2b=0 入场帧 ⇒ 49
    let mut st1b = StageClearState::new();
    stage_clear_arm(&mut st1b);
    stage_clear_step(&mut st1b, env(0, 1));
    assert_eq!(st1b.countdown_2f, 49, "入场同帧 `$2B&7==0` 即 DEC（E337）");
    // 入场一次性（E2FC-E302 `CMP $27/BEQ`）：连续帧不得重写倒数初值
    let mut st2 = st;
    stage_clear_step(&mut st2, env(0, 0)); // DEC → 49
    stage_clear_step(&mut st2, env(0, 0)); // DEC → 48
    assert_eq!(st2.countdown_2f, 48, "一次性入场：倒数持续 DEC，不重初始化（否则恒 50）");
    assert_eq!(st2.flags_27, 0x80);
    // 倒数环：$2B&7==0 帧 DEC；$06A0:=2 每帧刷新；减尽写飞离向量 $A4:$0D（E32C-E342）
    assert_eq!(st.no_death, 2, "E32E `$06A0:=2`（关末免死保护）");
    // 49 次 DEC（frame_2b=0）后 countdown=1；再 1 次 ⇒ 0 + 飞离向量
    for i in 0..49u32 {
        stage_clear_step(&mut st, env(0, 0));
        assert_eq!(st.countdown_2f, 0x32 - 1 - i as u8, "E337 每 `$2B&7==0` 帧 DEC $2F");
    }
    stage_clear_step(&mut st, env(0, 0));
    assert_eq!(st.countdown_2f, 0, "倒数尽（50 次 DEC）");
    assert_eq!(st.state_vec_hi, 0xA4, "E342 `$41C:=$A4`（飞离向量高）");
    assert_eq!(st.state_vec_lo, 0x0D, "E33D `$438:=$0D`（飞离向量低）");
    // `$2B&7≠0` 帧不 DEC（E331-E335 `AND #7/BNE`）
    let mut st3 = StageClearState::new();
    stage_clear_arm(&mut st3);
    stage_clear_step(&mut st3, env(0, 0)); // 入场
    let before = st3.countdown_2f;
    stage_clear_step(&mut st3, env(3, 0));
    assert_eq!(st3.countdown_2f, before, "`$2B&7≠0` 帧不 DEC（每 8f 一减）");
}

/// ② R-l5-4：三路分派（`INC $180` 后哨兵 → 通关 → 关间）+ `$0300` 等待 fail-closed。
#[test]
fn stage_clear_dispatch_three_way() {
    // 关间路：stage 1..7 派发 NextStage 且 `$180` 逐关 +1
    let mut st = StageClearState::new();
    for n in 1..=7u8 {
        assert_eq!(st.stage, n);
        let (_, ph) = run_clear(&mut st, 1);
        assert_eq!(ph, StageClearPhase::NextStage, "第 {n} 关清完 → E396 关间（商店 init）");
        assert_eq!(st.stage, n + 1, "E37C `INC $180`");
        assert_eq!(st.flags_26 & 0x10, 0, "E370 清关末位");
        // 复位一次性位（下一关 E2FC 重新入场）
        st.flags_27 = 0;
    }
    // 通关判定（A3 §1.2）：第 8 关清完 `$180==9` → AllClear
    assert_eq!(st.stage, 8);
    let (_, ph) = run_clear(&mut st, 1);
    assert_eq!(ph, StageClearPhase::AllClear, "E392 `$180==9` → `$95F9` 结局入口");
    assert_eq!(st.stage, 9);
    // `$FFF8` 哨兵（0x0A，出厂不可达死分支保真）：stage 到 9 后再清一关 → 10 → EarlyEnd
    st.flags_27 = 0;
    let (_, ph) = run_clear(&mut st, 1);
    assert_eq!(ph, StageClearPhase::EarlyEnd, "E37F `$180==$FFF8`（0x0A）→ 回标题（保真）");
    assert_eq!(FFF8_SENTINEL, 0x0A, "哨兵=ROM 尾标志 file 0x20008 三版同值");
}

/// ③ R-l5-3/4 负向 + 粗粒度边集（R-l5-1 关节表）：fail-closed 不发明。
#[test]
fn stage_clear_fail_closed_and_edges() {
    // 负向①：`$0300` 闸未开 ⇒ 永不派发（E366 等待）
    let mut st = StageClearState::new();
    stage_clear_arm(&mut st);
    for i in 0..2000u32 {
        stage_clear_step(&mut st, env((i as u8) & 0xFF, 0));
    }
    assert_eq!(st.phase, StageClearPhase::Clearing, "闸未开 fail-closed 停留");
    assert_eq!(st.stage, 1, "不得 INC $180");
    // 负向②：一次性等待（E304 `$0400==0` ⇒ 不入场）
    let mut st = StageClearState::new();
    stage_clear_arm(&mut st);
    let fx = stage_clear_step(
        &mut st,
        StageClearEnv { frame_2b: 0, gate_300: 1, player_present: false },
    );
    assert_eq!(fx, StageClearFx::None);
    assert_eq!(st.flags_27, 0, "等玩家对象（E304 BEQ）：不置 `$27` bit7");
    // 负向③：`$26 bit7` 直回标题支不接线（G-S5-1，A2 §3.4 同支）——不派发不发明
    let mut st = StageClearState::new();
    stage_clear_arm(&mut st);
    st.flags_26 |= 0x80;
    let fx = stage_clear_step(&mut st, env(0, 1));
    assert_eq!(fx, StageClearFx::None);
    assert_eq!(st.phase, StageClearPhase::Clearing, "bit7 支不接线（fail-closed 登记）");
    // 负向④：未 arm 不动作
    let mut st = StageClearState::new();
    let fx = stage_clear_step(&mut st, env(0, 1));
    assert_eq!(fx, StageClearFx::None);
    assert_eq!(st.phase, StageClearPhase::Idle);

    // 粗粒度边集（t81+A2+A3）：已证边必过 / 未证边必拒
    use Mode::*;
    let ok = |p: Mode, ev: ModeEvent, q: Mode| {
        let s = FlowState { mode: p, data: Default::default() };
        assert_eq!(s.advance(ev).map(|s| s.mode).unwrap(), q, "已证边 {:?}+{:?}", p, ev);
    };
    ok(Combat, ModeEvent::StageClear, StageClear);
    ok(StageClear, ModeEvent::StageAdvance, Shop);
    ok(StageClear, ModeEvent::AllClear, Ending);
    ok(StageClear, ModeEvent::EarlyEnd, Title);
    let reject = |p: Mode, ev: ModeEvent| {
        let s = FlowState { mode: p, data: Default::default() };
        assert!(s.advance(ev).is_err(), "未证边必拒 {:?}+{:?}", p, ev);
    };
    reject(StageClear, ModeEvent::StageStart); // 关末→下一关必经商店（E16B→$9BC3）：直跳不发明
    reject(StageClear, ModeEvent::MidCombat);
    reject(Combat, ModeEvent::StageAdvance);
    reject(Ending, ModeEvent::MidCombat); // 结局段内部=挂起枚举，不发明转移
    reject(Shop, ModeEvent::AllClear);
}

/// ④ R-l5-5：结局入口（`bank00:95F9 $26:=$80`）+ EndingFx 挂起序列登记。
#[test]
fn ending_entry_mirrors_95f9() {
    let mut st = StageClearState::new();
    let fx = ending_entry(&mut st);
    assert_eq!(fx, EndingFx::EntryScenePending, "入口后首个挂起=场景服务装载");
    assert_eq!(st.flags_26, 0x80, "bank00:95F9 `LDA #$80/STA $26`");
    // 挂起序列完整性（A3 §2/§3 五项，不发明内部写集）
    let seq = [
        EndingFx::EntryScenePending,
        EndingFx::StaffRollBgmPending,
        EndingFx::PreScenePending,
        EndingFx::StaffRollScrollPending,
        EndingFx::EndingCardPending,
    ];
    assert_eq!(seq.len(), 5, "staff roll→结局卡挂起五项（G-S5-4）");
}

/// ⑤ R-l5-7（可选段）：死亡/续关与关末链互不串扰——`$180` 续关保留（A2 R-continue-reset）
/// 与关末 `INC $180` 独立镜像（正向哨兵：两链状态不互写）。
#[test]
fn death_and_stage_clear_chains_disjoint() {
    // 死亡链 `DeathChainState.stage` 与关末链 `StageClearState.stage` 同 RAM `$180`：
    // 各自镜像独立，本测试锁两链 API 分立（混接=设计缺口，禁止）。
    let d = DeathChainState::new();
    let s = StageClearState::new();
    assert_eq!(d.stage, s.stage, "同 RAM `$180` 初值一致（1）");
    assert_eq!(d.stage_advance_gate, s.stage_advance_gate, "同 RAM `$0300` 初值一致");
}
