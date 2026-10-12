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
    let mut total_events = 0usize;
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
            let n_weapon = events.iter().filter(|e| matches!(e, game_oracle::l3::TraceEvent::Weapon(_))).count();
            let n_bomb = events.len() - n_weapon;
            let mut problems = game_oracle::l3::weapon_events_legal(&events);
            problems.extend(game_oracle::l3::bomb_events_legal(&events));
            total += events.len();
            if problems.is_empty() {
                println!("[ok]   {path}: {n_weapon} weapon + {n_bomb} bomb 转移全合法（武器：同型升级/异型切换/重生复位；弹药/弹种：R-fire-4 扣减·R-respawn-4 回满·独立维度切换）");
            } else {
                failures += 1;
                println!("[FAIL] {path}:");
                for p in &problems {
                    println!("         {p}");
                }
            }
        } else if path.ends_with(".shots.jsonl") {
            let frames = match game_oracle::l3::parse_shot_frames(&jsonl) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{path}: golden 解析失败：{e}");
                    return ExitCode::from(2);
                }
            };
            match game_oracle::l3::diff_shot_trajectory(&frames) {
                Ok((compared, mut problems)) => {
                    let mut alloc = game_oracle::l3::alloc_events_legal(&frames);
                    alloc.extend(game_oracle::l3::lifetime_decrements_legal(&frames));
                    let n_alloc = if alloc.is_empty() { "全合法" } else { "有违例" };
                    problems.extend(alloc);
                    total += compared;
                    if problems.is_empty() {
                        println!("[ok]   {path}: 弹轨迹 {compared} 帧逐槽全等（出生帧播种+逐帧 tick）；槽位分配+寿命递减 {n_alloc}");
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
        } else if path.ends_with(".enemy.jsonl") {
            let rows = match game_oracle::l3::parse_enemy_rows(&jsonl) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{path}: golden 解析失败：{e}");
                    return ExitCode::from(2);
                }
            };
            let churn: usize = rows.iter().map(|r| r.state.iter().filter(|&&s| s != 0).count()).sum();
            let problems = game_oracle::l3::enemy_clamp_legal(&rows);
            total += rows.len();
            if problems.is_empty() {
                println!("[ok]   {path}: churn 快照 {} 行（活跃槽计 {churn}）R-enemy-4 钳全合法", rows.len());
            } else {
                failures += 1;
                println!("[FAIL] {path}:");
                for p in &problems {
                    println!("         {p}");
                }
            }
        } else if path.ends_with(".scroll.jsonl") {
            let rows = match game_oracle::l3::parse_scroll_rows(&jsonl) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{path}: golden 解析失败：{e}");
                    return ExitCode::from(2);
                }
            };
            match game_oracle::l3::diff_scroll_trace(&rows) {
                Ok((compared, problems)) => {
                    total += compared;
                    if problems.is_empty() {
                        println!("[ok]   {path}: $1C/$1D 逐帧全等 {compared} 帧（R-bg-scroll-1..3）");
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
        } else if path.ends_with(".money.jsonl") {
            let events = match game_oracle::l3::parse_money_events(&jsonl) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("{path}: golden 解析失败：{e}");
                    return ExitCode::from(2);
                }
            };
            let problems = game_oracle::l3::money_events_legal(&events);
            total += events.len();
            if problems.is_empty() {
                println!("[ok]   {path}: 金钱事件 {} 个全合法（R-money-5 加数表/封顶）", events.len());
            } else {
                failures += 1;
                println!("[FAIL] {path}:");
                for p in &problems {
                    println!("         {p}");
                }
            }
        } else if path.ends_with(".flow.jsonl") {
            let rows = match game_oracle::l3::parse_flow_rows(&jsonl) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{path}: golden 解析失败：{e}");
                    return ExitCode::from(2);
                }
            };
            let problems = game_oracle::l3::flow_sequence_legal(&rows);
            total += rows.len();
            if problems.is_empty() {
                println!("[ok]   {path}: 流程模式序列 {} 事件全合法（R-ui-6/7 已证边集）", rows.len());
            } else {
                failures += 1;
                println!("[FAIL] {path}:");
                for p in &problems {
                    println!("         {p}");
                }
            }
        } else if path.ends_with(".stage-appear.jsonl") {
            let rows = match game_oracle::l3::parse_appear_rows(&jsonl) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{path}: golden 解析失败：{e}");
                    return ExitCode::from(2);
                }
            };
            let (problems, match_n, total) = game_oracle::l3::appear_records_check(&rows);
            total_events += rows.len();
            if problems.is_empty() {
                println!("[ok]   {path}: 敌出现 {total} 事件网格域全合法；记录集 x 命中 {match_n}/{total}（W12/W28 注记观测面）");
            } else {
                failures += 1;
                println!("[FAIL] {path}:");
                for p in &problems {
                    println!("         {p}");
                }
            }
        } else {
            eprintln!("{path}: 未知 golden 类型（*.player.jsonl / *.weapons.jsonl / *.shots.jsonl）");
            return ExitCode::from(2);
        }
    }
    if failures > 0 {
        println!("L3-DIFF: 失配（{failures} 个 golden，详见上；禁止调阈值放水）");
        ExitCode::from(1)
    } else {
        println!("L3-DIFF: 全绿（{total} 项比对 + 敌出现观测 {total_events} 事件）");
        ExitCode::SUCCESS
    }
}
