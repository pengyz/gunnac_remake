//! 命中/清除链测试（t68）：样本 golden 驱动 + f1404 释放写集 + 负向自证。
//! golden 载体 = oracle/subroutines/hit_samples.jsonl（t66 §3 A 级样本 12 例）。

use game_core::enemy::*;

fn load_samples() -> Vec<Sample> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../oracle/subroutines/hit_samples.jsonl");
    let text = std::fs::read_to_string(path).expect("hit_samples.jsonl 可读");
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(parse_line)
        .collect()
}

#[derive(Debug)]
struct Sample { kind: String, dx: i16, dy: i16 }

fn parse_line(l: &str) -> Sample {
    // 极简字段提取（数值为十进制，字段顺序固定）
    let get = |k: &str| -> i16 {
        let pat = format!("\"{k}\":");
        let i = l.find(&pat).unwrap_or_else(|| panic!("缺字段 {k}"));
        let rest = &l[i + pat.len()..];
        let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(rest.len());
        rest[..end].parse().expect("数值")
    };
    let kind = if l.contains("shot_enemy") { "shot_enemy" } else if l.contains("player_enemy") { "player_enemy" } else { "release" };
    if kind == "release" {
        Sample { kind: kind.into(), dx: 0, dy: 0 }
    } else {
        Sample { kind: kind.into(), dx: get("dx"), dy: get("dy") }
    }
}

#[test]
fn hit_samples_all_classify_with_default_box() {
    let samples = load_samples();
    let shot: Vec<_> = samples.iter().filter(|s| s.kind == "shot_enemy").collect();
    let player: Vec<_> = samples.iter().filter(|s| s.kind == "player_enemy").collect();
    assert_eq!(shot.len(), 10, "弹×敌样本 10 例");
    assert_eq!(player.len(), 2, "玩家×敌样本 2 例");
    for s in &shot {
        assert!(HitBox::SHOT_ENEMY.contact(s.dx, s.dy), "样本命中未触发（必红）：{:?}", s);
    }
    for s in &player {
        assert!(HitBox::PLAYER_ENEMY.contact(s.dx, s.dy), "玩家样本命中未触发（必红）：{:?}", s);
    }
}

#[test]
fn hit_clear_release_matches_f1404_golden() {
    // f1404 slot#8 字段列差分 = golden 基准
    let before = GridSlot { type_: 18, f1c: 131, x_subpix: 224, y_subpix: 208, speed: 176, hit_cnt: 4 };
    let after = hit_clear_release(&before);
    assert_eq!(after, GridSlot { type_: 0, f1c: 0, x_subpix: 0, y_subpix: 0, speed: 0x10, hit_cnt: 0 });
}

#[test]
fn hit_countdown_is_decrement_reading() {
    // W18 倒数读：逐帧 DEC，归零触发（HP 读备选留 TODO 不实现）
    let mut v = 4u8;
    let mut fired = false;
    for _ in 0..4 {
        let (nv, hit) = hit_countdown_tick(v);
        v = nv;
        if hit { fired = true; }
    }
    assert!(fired && v == 0, "倒数读必须 4 帧后归零触发");
    // 自然递减旁证（respawn-window 55 次）：非命中环境亦递减——tick 无条件 DEC
    assert_eq!(hit_countdown_tick(9), (8, false));
}

#[test]
fn hit_negative_selfproof() {
    // 负向自证 1：非接触距离不得判命中（命中误触发必红）
    assert!(!HitBox::SHOT_ENEMY.contact(20, 0), "dx 越盒必须判非接触");
    assert!(!HitBox::SHOT_ENEMY.contact(0, 17), "dy 越盒必须判非接触");
    assert!(!HitBox::PLAYER_ENEMY.contact(3, 13), "玩家盒 dy=13 越界必须判非接触");
    // 负向自证 2：错误槽释放必红——释放写集必须逐值命中 f1404 基准（错误写集会被上测检出）
    let wrong = hit_clear_release(&GridSlot::default());
    assert_ne!(wrong.speed, 0, "写集若误清 speed=0 与基准 16 不符（本断言即哨兵）");
    assert_eq!(wrong.speed, 0x10);
}
