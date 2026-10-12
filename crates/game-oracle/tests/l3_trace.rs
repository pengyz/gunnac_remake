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

fn load_weapon_events() -> Vec<game_oracle::l3::WeaponEvent> {
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
    let events = load_weapon_events();
    assert!(!events.is_empty(), "武器事件 golden 为空 = L3 回归空转");
    let problems = game_oracle::l3::weapon_events_legal(&events);
    assert!(problems.is_empty(), "武器转移非法：\n{}", problems.join("\n"));
}

#[test]
fn l3_weapon_events_reject_illegal_steps() {
    use game_oracle::l3::{weapon_events_legal, WeaponEvent};
    let bad = [
        // 等级倒退（非回卷：255→0C 才是回卷）
        WeaponEvent { f: 1, before_slot: 0, before_level: 5, after_slot: 0, after_level: 3 },
        // 跨两阶跳升
        WeaponEvent { f: 2, before_slot: 1, before_level: 1, after_slot: 1, after_level: 4 },
        // 切换同时改等级
        WeaponEvent { f: 3, before_slot: 1, before_level: 2, after_slot: 3, after_level: 5 },
    ];
    let problems = weapon_events_legal(&bad);
    assert_eq!(problems.len(), 3, "非法步必须全数检出：{problems:?}");
    // 合法步：同型升级/溢出回卷/异型切换/重生复位
    let good = [
        WeaponEvent { f: 4, before_slot: 0, before_level: 5, after_slot: 0, after_level: 6 },
        WeaponEvent { f: 5, before_slot: 0, before_level: 255, after_slot: 0, after_level: 12 },
        WeaponEvent { f: 6, before_slot: 1, before_level: 2, after_slot: 3, after_level: 2 },
        WeaponEvent { f: 7, before_slot: 3, before_level: 9, after_slot: 0, after_level: 0 },
    ];
    assert!(weapon_events_legal(&good).is_empty(), "合法步被误判");
}
