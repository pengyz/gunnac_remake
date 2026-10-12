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
