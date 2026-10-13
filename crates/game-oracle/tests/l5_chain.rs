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
