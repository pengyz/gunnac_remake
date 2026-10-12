//! L3 帧级 trace 差分回归（rust-tests 门禁内跑）：game-core::player tick 序列 == oracle probe trace。
//!
//! golden = oracle/traces/gameplay-stage1.player.jsonl（Mesen2 确定性执行产物，
//! 两次重跑 sha256 一致；死亡/重生窗口已按 spec §4 未证域显式剔除并分段重播种）。

use std::path::PathBuf;

fn load_golden() -> Vec<game_oracle::l3::PlayerFrame> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../oracle/traces/gameplay-stage1.player.jsonl");
    let jsonl = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读取 L3 golden 失败（fail-closed）：{:?}: {e}", path));
    game_oracle::l3::parse_player_frames(&jsonl).unwrap_or_else(|e| panic!("L3 golden 解析失败：{e}"))
}

#[test]
fn l3_player_trace_all_green() {
    let frames = load_golden();
    let (compared, problems) =
        game_oracle::l3::diff_player_trace(&frames).expect("L3 差分执行错误");
    assert!(problems.is_empty(), "L3 帧级差分不一致：\n{}", problems.join("\n"));
    assert!(compared >= 900, "比对帧数异常偏低：{compared}");
}

#[test]
fn l3_golden_has_multiple_segments() {
    // 死亡/重生窗口剔除后应为多段（f 缺口 = 分段边界，重播种语义）
    let frames = load_golden();
    let mut gaps = 0usize;
    for w in frames.windows(2) {
        if w[1].f != w[0].f + 1 {
            gaps += 1;
        }
    }
    assert!(gaps >= 3, "分段数不足（期望 ≥3 个死亡/重生缺口）：{gaps}");
}

fn load_events() -> Vec<game_oracle::l3::TraceEvent> {
    let mut out = Vec::new();
    for scene in ["gameplay-stage1", "weapon-pickup"] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../oracle/traces/{scene}.weapons.jsonl"));
        let jsonl = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("读取武器事件 golden 失败：{:?}: {e}", path));
        out.extend(game_oracle::l3::parse_weapon_events(&jsonl).expect("解析"));
    }
    out
}

#[test]
fn l3_weapon_events_all_legal() {
    let events = load_events();
    assert!(!events.is_empty(), "事件 golden 为空 = L3 回归空转");
    let mut problems = game_oracle::l3::weapon_events_legal(&events);
    problems.extend(game_oracle::l3::bomb_events_legal(&events));
    assert!(problems.is_empty(), "事件不合法：\n{}", problems.join("\n"));
}

#[test]
fn l3_golden_contains_bomb_events() {
    // FA-3 收口判据：golden 含 kind=bomb 事件（R-bomb-2 断言源）
    let events = load_events();
    let n_bomb = events
        .iter()
        .filter(|e| matches!(e, game_oracle::l3::TraceEvent::Bomb(_)))
        .count();
    assert!(n_bomb > 0, "kind=bomb 事件缺失 = R-bomb-2 无断言源");
}

#[test]
fn l3_events_reject_illegal_steps() {
    use game_oracle::l3::{bomb_events_legal, weapon_events_legal, BombEvent, TraceEvent, WeaponEvent};
    let bad_weapon = [
        TraceEvent::Weapon(WeaponEvent { f: 1, before_slot: 0, before_level: 5, after_slot: 0, after_level: 3 }),
        TraceEvent::Weapon(WeaponEvent { f: 2, before_slot: 1, before_level: 1, after_slot: 1, after_level: 4 }),
        TraceEvent::Weapon(WeaponEvent { f: 3, before_slot: 1, before_level: 2, after_slot: 3, after_level: 5 }),
    ];
    assert_eq!(weapon_events_legal(&bad_weapon).len(), 3, "weapon 非法步必须全数检出");
    let good_weapon = [
        TraceEvent::Weapon(WeaponEvent { f: 4, before_slot: 0, before_level: 5, after_slot: 0, after_level: 6 }),
        TraceEvent::Weapon(WeaponEvent { f: 5, before_slot: 0, before_level: 255, after_slot: 0, after_level: 12 }),
        TraceEvent::Weapon(WeaponEvent { f: 6, before_slot: 1, before_level: 2, after_slot: 3, after_level: 2 }),
        TraceEvent::Weapon(WeaponEvent { f: 7, before_slot: 3, before_level: 9, after_slot: 0, after_level: 0 }),
    ];
    assert!(weapon_events_legal(&good_weapon).is_empty(), "weapon 合法步被误判");
    // bomb：R-fire-4 扣减不符（独立维度口径：弹种切换不与弹数耦合）
    let bad_bomb = [
        TraceEvent::Bomb(BombEvent { f: 61, before_type: 0, before_ammo: 4, before_idx: 0,
                                     after_type: 0, after_ammo: 2, after_idx: 0 }), // 应扣 1 得 3
    ];
    let problems = bomb_events_legal(&bad_bomb);
    assert_eq!(problems.len(), 1, "bomb 非法步必须检出：{problems:?}");
    let good_bomb = [
        TraceEvent::Bomb(BombEvent { f: 1, before_type: 0, before_ammo: 4, before_idx: 0,
                                     after_type: 0, after_ammo: 3, after_idx: 0 }), // R-fire-4 扣 1
        TraceEvent::Bomb(BombEvent { f: 2, before_type: 0, before_ammo: 1, before_idx: 0,
                                     after_type: 0, after_ammo: 4, after_idx: 0 }), // R-respawn-4 回满
        // 口径更正反例链同构：f2175 弹种 0→3 且弹数不变 = 独立维度切换，合法
        TraceEvent::Bomb(BombEvent { f: 3, before_type: 0, before_ammo: 3, before_idx: 0,
                                     after_type: 3, after_ammo: 3, after_idx: 0 }),
    ];
    assert!(bomb_events_legal(&good_bomb).is_empty(), "bomb 合法步被误判：{:?}", bomb_events_legal(&good_bomb));
}

#[test]
fn l3_shot_replay_machine_self_consistent() {
    // 轨迹重放机器自证：由 game-core::shot::tick 生成的轨迹必须被 diff_shot_trajectory 全等接受
    use game_core::shot::{script_fetch, spawn_init, tick, ACTION_DURATIONS, ACTION_STREAM};
    let mut obj = spawn_init(0, 2, 0, 100, 50, 0x0C);
    let mut frames = vec![game_oracle::l3::ShotFrame {
        f: 1, slot: 0x80, win: 1, lifetime: obj.lifetime, script_ptr: obj.script_ptr, pos_x: obj.pos_x,
        pos_y: obj.pos_y, motion_code: obj.motion_code, shot_type: obj.shot_type,
    }];
    for f in 2..40u32 {
        if tick(&mut obj).is_none() {
            break;
        }
        frames.push(game_oracle::l3::ShotFrame {
            f, slot: 0x80, win: 1, lifetime: obj.lifetime, script_ptr: obj.script_ptr, pos_x: obj.pos_x,
            pos_y: obj.pos_y, motion_code: obj.motion_code, shot_type: obj.shot_type,
        });
    }
    let (compared, problems) = game_oracle::l3::diff_shot_trajectory(&frames).expect("执行错误");
    assert!(problems.is_empty(), "重放机器自证失败：{problems:?}");
    assert!(compared >= 5, "轨迹过短：{compared}");
    // 脚本机跳转语义（duration=0 → 指针跳转）：合成表单测
    let (dur, code, ptr) = script_fetch(0, &[0, 0xFF, 0x0A], &[2, 5, 7]);
    assert_eq!((dur, code, ptr), (0x0A, 7, 3), "0 时长跳转到 action[0]=2 再取");
    let _ = ACTION_DURATIONS[0];
    let _ = ACTION_STREAM[0];
}

#[test]
fn l3_alloc_first_fit_semantics() {
    use game_core::shot::{shot_slot_alloc, SlotAlloc};
    // 首次适应：组序×组内序，低偏移优先；满载 Full
    let mut probe = [0u8; 0x100];
    assert_eq!(shot_slot_alloc(&probe), SlotAlloc::Found { slot: 0x80, a: 0 });
    probe[0x80] = 1;
    probe[0x81] = 1;
    assert_eq!(shot_slot_alloc(&probe), SlotAlloc::Found { slot: 0x82, a: 0 });
    for slot in [0x80, 0x81, 0x82, 0x83, 0x84, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0,
                 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8] {
        probe[slot] = 1;
    }
    assert_eq!(shot_slot_alloc(&probe), SlotAlloc::Full, "20 槽全占必须满载");
    // L3 分配断言：合成出生帧全向量（槽 $80/$81 出生）= 首次适应前二空槽 ✓
    let rows: Vec<game_oracle::l3::ShotFrame> = (0u8..20).map(|i| {
        let slot = [0x80u8,0x81,0x82,0x83,0x84,0x9C,0x9D,0x9E,0x9F,0xA0,0xB8,0xB9,0xBA,0xBB,0xBC,0xD4,0xD5,0xD6,0xD7,0xD8][i as usize];
        let lt = if slot <= 0x81 { 1 } else { 0 };
        game_oracle::l3::ShotFrame {
            f: 10, slot, win: 1, lifetime: lt, script_ptr: 0, pos_x: 0, pos_y: 0,
            motion_code: 0, shot_type: 0,
        }
    }).collect();
    assert!(game_oracle::l3::alloc_events_legal(&rows).is_empty(), "合法出生帧被误判");
}

#[test]
fn l3_alloc_negative_selfproof() {
    // 负向自证：期望反转/非首次适应的出生集必须被检出（断言不得空转）
    use game_oracle::l3::{alloc_events_legal, ShotFrame};
    let mut rows = Vec::new();
    // 出生前帧全向量（f9）：全空
    let slots = [0x80u8, 0x81, 0x82, 0x83, 0x84, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0,
                 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8];
    for &slot in &slots {
        rows.push(ShotFrame { f: 9, slot, win: 0, lifetime: 0, script_ptr: 0,
                              pos_x: 0, pos_y: 0, motion_code: 0, shot_type: 0 });
    }
    // f10：槽 $B8 出生（非首次适应——$80 才是首空槽）⇒ 必须报违例
    for &slot in &slots {
        let lt = if slot == 0xB8 { 10 } else { 0 };
        rows.push(ShotFrame { f: 10, slot, win: 1, lifetime: lt, script_ptr: 0,
                              pos_x: 0, pos_y: 0, motion_code: 0, shot_type: 0 });
    }
    let problems = alloc_events_legal(&rows);
    assert!(!problems.is_empty(), "非首次适应出生必须被检出（负向自证）");
}

#[test]
fn l3_lifetime_negative_selfproof() {
    // 负向自证：寿命中途无装载回升（239→255）必须被检出
    use game_oracle::l3::{lifetime_decrements_legal, ShotFrame};
    let rows = vec![
        ShotFrame { f: 1, slot: 0x80, win: 1, lifetime: 240, script_ptr: 0,
                    pos_x: 0, pos_y: 0, motion_code: 0, shot_type: 0 },
        ShotFrame { f: 2, slot: 0x80, win: 1, lifetime: 255, script_ptr: 0,
                    pos_x: 0, pos_y: 0, motion_code: 0, shot_type: 0 },
    ];
    let problems = lifetime_decrements_legal(&rows);
    assert!(!problems.is_empty(), "无装载回升必须被检出（负向自证）");
    // 合法：递减 + 归 1 后装载（255 为时长表值）
    let ok = vec![
        ShotFrame { f: 1, slot: 0x80, win: 1, lifetime: 2, script_ptr: 0,
                    pos_x: 0, pos_y: 0, motion_code: 0, shot_type: 0 },
        ShotFrame { f: 2, slot: 0x80, win: 1, lifetime: 1, script_ptr: 0,
                    pos_x: 0, pos_y: 0, motion_code: 0, shot_type: 0 },
        ShotFrame { f: 3, slot: 0x80, win: 1, lifetime: 255, script_ptr: 0,
                    pos_x: 0, pos_y: 0, motion_code: 0, shot_type: 0 },
    ];
    assert!(lifetime_decrements_legal(&ok).is_empty(), "合法递减/装载被误判");
}

#[test]
fn l3_enemy_clamp_negative_selfproof() {
    use game_oracle::l3::{enemy_clamp_legal, EnemyRow};
    // 负向自证：槽 0 越钳必须检出
    let bad = vec![EnemyRow {
        f: 1,
        state: vec![1; 28],
        x: { let mut v = vec![0x80u8; 28]; v[0] = 0x05; v },
        y: vec![0xA0; 28],
    }];
    assert!(!enemy_clamp_legal(&bad).is_empty(), "槽 0 越钳必须检出（负向自证）");
    // 合法：界内 + 未放置模板槽（(0,0)）忽略
    let ok = vec![EnemyRow {
        f: 2,
        state: vec![1; 28],
        x: { let mut v = vec![0x80u8; 28]; v[5] = 0; v },
        y: { let mut v = vec![0xA0u8; 28]; v[5] = 0; v },
    }];
    assert!(enemy_clamp_legal(&ok).is_empty(), "合法行被误判");
}

#[test]
fn l3_money_events_negative_selfproof() {
    use game_oracle::l3::{money_events_legal, MoneyEvent};
    // 负向自证：不可分解 delta / 负 delta 必红
    let bad = vec![
        MoneyEvent { f: 1, before: vec![0; 7], after: vec![0, 0, 0, 0, 0, 1, 5] }, // delta=15
        MoneyEvent { f: 2, before: vec![0, 0, 1, 0, 0, 0, 0], after: vec![0, 0, 0, 5, 0, 0, 0] }, // 负
    ];
    assert_eq!(money_events_legal(&bad).len(), 2, "非法金钱事件必须全数检出");
    // 合法：单加数 + 捆绑（200=100+100）+ 封顶
    let good = vec![
        MoneyEvent { f: 3, before: vec![0; 7], after: vec![0, 0, 0, 0, 1, 0, 0] }, // +100
        MoneyEvent { f: 4, before: vec![0; 7], after: vec![0, 0, 0, 2, 0, 0, 0] }, // +200 捆绑
        MoneyEvent { f: 5, before: vec![0; 7], after: vec![9, 9, 9, 9, 9, 9, 0] }, // 封顶
    ];
    assert!(money_events_legal(&good).is_empty(), "合法事件被误判");
}

#[test]
fn l3_tick_hard_assertions_and_negatives() {
    use game_oracle::l3::{appear_records_check, AppearRow};
    // 合法样例（小域）：干净段 A 网格 2 tick 各 1 事件——但分布/86 校验需全量；
    // 负向自证 1：干净段非网格帧必红
    let bad = vec![AppearRow { f: 1342, slot: 0, x: 0, y: 0 }];
    let (problems, _, _) = appear_records_check(&bad);
    assert!(problems.iter().any(|p| p.contains("非网格帧")), "非网格帧必红：{problems:?}");
    // 负向自证 2：帧序倒退必红
    let bad2 = vec![
        AppearRow { f: 1351, slot: 0, x: 0, y: 0 },
        AppearRow { f: 1341, slot: 1, x: 0, y: 0 },
    ];
    let (problems2, _, _) = appear_records_check(&bad2);
    assert!(problems2.iter().any(|p| p.contains("倒退")), "顺序违例必红");
    // 负向自证 3：槽越界必红
    let bad3 = vec![AppearRow { f: 1341, slot: 28, x: 0, y: 0 }];
    let (problems3, _, _) = appear_records_check(&bad3);
    assert!(problems3.iter().any(|p| p.contains("28 槽")), "槽越界必红");
}
