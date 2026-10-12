//! l3-diff —— L3 帧级 trace 差分 CLI（被 tools/diff_traces.py 薄封装调用）。
//!
//! 裁定（多 golden 自动分派）：
//! - `*.player.jsonl`：game-core::player tick 序列逐帧状态向量 == oracle probe trace；
//! - `*.weapons.jsonl`：trace 中每次武器状态转移 ∈ {同型升级(含回卷)/异型切换/重生复位} 合法步。
//! 任一失配 ⇒ 退出码 1（不得放宽阈值；按 AGENTS.md §4 定位层）。

use std::process::ExitCode;

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("用法: l3-diff <golden.jsonl>...（*.player.jsonl / *.weapons.jsonl）");
        return ExitCode::from(2);
    }
    let mut total = 0usize;
    let mut failures = 0usize;
    for path in &paths {
        let jsonl = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("读取 L3 golden 失败（fail-closed）：{path}: {e}");
                return ExitCode::from(2);
            }
        };
        if path.ends_with(".player.jsonl") {
            let frames = match game_oracle::l3::parse_player_frames(&jsonl) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{path}: golden 解析失败：{e}");
                    return ExitCode::from(2);
                }
            };
            match game_oracle::l3::diff_player_trace(&frames) {
                Ok((compared, problems)) => {
                    total += compared;
                    if problems.is_empty() {
                        println!("[ok]   {path}: {compared} 帧逐帧状态向量全等（对齐 state(N)=tick(state(N-1), held(N))）");
                    } else {
                        failures += 1;
                        println!("[FAIL] {path}:");
                        for p in &problems {
                            println!("         {p}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{path}: 差分执行错误：{e}");
                    return ExitCode::from(2);
                }
            }
        } else if path.ends_with(".weapons.jsonl") {
            let events = match game_oracle::l3::parse_weapon_events(&jsonl) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{path}: golden 解析失败：{e}");
                    return ExitCode::from(2);
                }
            };
            let problems = game_oracle::l3::weapon_events_legal(&events);
            total += events.len();
            if problems.is_empty() {
                println!("[ok]   {path}: {} 次武器状态转移全部为合法步（同型升级/异型切换/重生复位）", events.len());
            } else {
                failures += 1;
                println!("[FAIL] {path}:");
                for p in &problems {
                    println!("         {p}");
                }
            }
        } else {
            eprintln!("{path}: 未知 golden 类型（*.player.jsonl / *.weapons.jsonl）");
            return ExitCode::from(2);
        }
    }
    if failures > 0 {
        println!("L3-DIFF: 失配（{failures} 个 golden，详见上；禁止调阈值放水）");
        ExitCode::from(1)
    } else {
        println!("L3-DIFF: 全绿（{total} 项比对）");
        ExitCode::SUCCESS
    }
}
