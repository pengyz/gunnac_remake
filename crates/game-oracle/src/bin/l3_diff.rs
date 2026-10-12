//! l3-diff —— L3 帧级 trace 差分 CLI（被 tools/diff_traces.py 薄封装调用）。
//!
//! 裁定：game-core::player tick 序列逐帧状态向量 == oracle probe trace。
//! 任一帧失配 ⇒ 退出码 1（不得放宽阈值；按 AGENTS.md §4 定位层）。

use std::process::ExitCode;

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "oracle/traces/gameplay-stage1.player.jsonl".to_string());
    let jsonl = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("读取 L3 golden 失败（fail-closed）：{path}: {e}");
            return ExitCode::from(2);
        }
    };
    let frames = match game_oracle::l3::parse_player_frames(&jsonl) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("L3 golden 解析失败：{e}");
            return ExitCode::from(2);
        }
    };
    println!("== l3-diff：{} 帧（{}）==", frames.len(), path);
    match game_oracle::l3::diff_player_trace(&frames) {
        Ok((compared, problems)) => {
            if problems.is_empty() {
                println!("[ok]   {compared} 帧逐帧状态向量全等（player_x/y + 亚像素；对齐 state(N)=tick(state(N-1), held(N))）");
                println!("L3-DIFF: {compared}/{compared} 通过");
                ExitCode::SUCCESS
            } else {
                for p in &problems {
                    println!("[FAIL] {p}");
                }
                println!("L3-DIFF: 失配（详见上；禁止调阈值放水）");
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("L3 差分执行错误：{e}");
            ExitCode::from(2)
        }
    }
}
