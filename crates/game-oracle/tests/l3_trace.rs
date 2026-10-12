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
