//! L5 流程级差分测试（t172）：STG-01 首关全程链——标题→菜单→商店→过渡→关卡→
//! 战斗→死亡重生循环（fail-closed：GAME OVER/CONTINUE 禁入）。

use game_oracle::l5::{diff_l5, l5_legality, parse_l5, L5Event};

fn ev(scene: &str, f: u32, mode: &str, cp: Option<&str>) -> L5Event {
    L5Event { scene: scene.into(), f, mode: mode.into(), checkpoint: cp.map(|s| s.into()) }
}

#[test]
fn stg01_chain_parses_and_legality() {
    // STG-01 全程链（4 场景接力）：title-menu → shop → respawn-window → gameplay-stage1
    let golden = vec![
        ev("title-menu", 180, "Black", Some("boot-black")),
        ev("title-menu", 360, "Title", Some("title-in")),
        ev("title-menu", 540, "Menu", Some("menu-in")),
        ev("gameplay-stage1", 1080, "Shop", Some("shop-in")),
        ev("gameplay-stage1", 1200, "Transition", Some("shop-exit")),
        ev("gameplay-stage1", 1260, "StageStart", Some("stg01-start")),
        ev("gameplay-stage1", 1500, "Combat", Some("combat")),
        ev("respawn-window", 1260, "StageStart", None),
        ev("respawn-window", 1350, "Combat", None),
        ev("respawn-window", 1400, "DeathRespawn", Some("respawn-window")),
    ];
    let problems = l5_legality(&golden);
    assert!(problems.is_empty(), "全程链合法性（已证模式）：{problems:?}");
}

#[test]
fn l5_fail_closed_unproven_modes() {
    // ④ 未证段 fail-closed：GAME OVER/CONTINUE 禁入 golden（负向自证）
    let bad = vec![ev("gameplay-stage1", 2400, "GameOverUnproven", None)];
    let problems = l5_legality(&bad);
    assert!(!problems.is_empty(), "未证段必须 fail-closed（负向）");
    assert!(problems[0].contains("fail-closed"), "违规含 fail-closed 标记：{problems:?}");
}

#[test]
fn l5_diff_three_way_and_negative() {
    let g = vec![ev("gameplay-stage1", 1260, "StageStart", Some("stg01-start"))];
    let r = vec![ev("gameplay-stage1", 1260, "StageStart", Some("stg01-start"))];
    assert!(diff_l5(&g, &r).is_empty(), "全等=无差");
    let r2 = vec![ev("gameplay-stage1", 1261, "StageStart", Some("stg01-start"))];
    assert!(!diff_l5(&g, &r2).is_empty(), "帧号差必检出（0 容差）");
    let r3 = vec![ev("gameplay-stage1", 1260, "Combat", Some("stg01-start"))];
    assert!(diff_l5(&g, &r3).iter().any(|p| p.contains("模式不等")), "模式差必检出");
}

#[test]
fn l5_parse_roundtrip() {
    let jsonl = r#"{"scene":"gameplay-stage1","f":1260,"mode":"StageStart","checkpoint":"stg01-start"}
{"scene":"gameplay-stage1","f":1500,"mode":"Combat"}"#;
    let evs = parse_l5(jsonl).expect("解析");
    assert_eq!(evs.len(), 2);
    assert_eq!(evs[0].checkpoint.as_deref(), Some("stg01-start"));
    assert_eq!(evs[1].checkpoint, None);
}

#[test]
fn l5_illegal_sequence_fails_closed() {
    // R14-1：非法模式序列必须 fail（t81 边集校验实装后）
    let bad = vec![
        ev("gameplay-stage1", 100, "Black", None),
        ev("gameplay-stage1", 200, "Combat", None), // Black→Combat 非法
    ];
    let problems = l5_legality(&bad);
    assert!(problems.iter().any(|p| p.contains("非法模式序列")), "Black→Combat 必拒（负向）：{problems:?}");
    // t81 现成负向边复用：Boot→Combat 拒 / Combat→Shop 拒
    let bad2 = vec![
        ev("s", 1, "Boot", None),
        ev("s", 2, "Combat", None),
    ];
    assert!(l5_legality(&bad2).iter().any(|p| p.contains("非法模式序列")), "Boot→Combat 必拒");
    let bad3 = vec![
        ev("s", 1, "Combat", None),
        ev("s", 2, "Shop", None),
    ];
    assert!(l5_legality(&bad3).iter().any(|p| p.contains("非法模式序列")), "Combat→Shop 必拒");
    // 帧序倒退负向
    let bad4 = vec![
        ev("s", 100, "Combat", None),
        ev("s", 50, "Combat", None),
    ];
    assert!(l5_legality(&bad4).iter().any(|p| p.contains("帧序倒退")), "帧序倒退必检出");
}

#[test]
fn l5_scope_note_3scenes_10events() {
    // 低危口径注（R14-2）：3 scene/10 事件/8 命名检查点（stg01-chain.l5.jsonl 对账）
    let jsonl = std::fs::read_to_string("../../oracle/traces/stg01-chain.l5.jsonl").expect("golden");
    let evs = parse_l5(&jsonl).expect("解析");
    let scenes: std::collections::BTreeSet<_> = evs.iter().map(|e| e.scene.as_str()).collect();
    let named = evs.iter().filter(|e| e.checkpoint.is_some()).count();
    assert_eq!(scenes.len(), 3, "3 场景");
    assert_eq!(evs.len(), 10, "10 事件");
    assert_eq!(named, 8, "8 命名检查点（t81 flow+t84 内容 id 落档）");
}
