//! l2-diff —— L2 子程序级差分 CLI（被 tools/diff_subroutines.py 薄封装调用）。
//!
//! 三方裁定（见 game_oracle::l2）：case golden == 6502 核执行 ROM 原字节 == game-core 同名函数。
//! 任一不一致 ⇒ 退出码 1（不得放宽阈值；修复按 AGENTS.md §4 定位层）。

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut rom_path = String::from("rom/gunnac_cn.nes");
    let mut cases_path = String::from("oracle/subroutines/l2_cases.jsonl");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--rom" => { i += 1; rom_path = args.get(i).cloned().unwrap_or_default(); }
            "--cases" => { i += 1; cases_path = args.get(i).cloned().unwrap_or_default(); }
            other => {
                eprintln!("未知参数 {other}（用法：l2-diff [--rom ROM] [--cases CASES.jsonl]）");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let rom = match std::fs::read(&rom_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("读取 ROM 失败（fail-closed）：{rom_path}: {e}");
            return ExitCode::from(2);
        }
    };
    let jsonl = match std::fs::read_to_string(&cases_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("读取 case 文件失败：{cases_path}: {e}");
            return ExitCode::from(2);
        }
    };
    let cases = match game_oracle::l2::parse_cases(&jsonl) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("case 解析失败：{e}");
            return ExitCode::from(2);
        }
    };

    println!("== l2-diff：{} 条用例（{}）==", cases.len(), cases_path);
    let mut failures = 0usize;
    for case in &cases {
        match game_oracle::l2::diff_case(&rom, case) {
            Ok((_rom_out, _rust_out, problems)) => {
                if problems.is_empty() {
                    println!("[ok]   {} ({})", case.id, case.subroutine);
                } else {
                    failures += 1;
                    println!("[FAIL] {} ({})", case.id, case.subroutine);
                    for p in problems {
                        println!("         {p}");
                    }
                }
            }
            Err(e) => {
                failures += 1;
                println!("[FAIL] {} ({}) 执行错误：{e}", case.id, case.subroutine);
            }
        }
    }
    println!(
        "L2-DIFF: {}/{} 通过（golden==ROM 执行==game-core 同名函数，三者缺一不可）",
        cases.len() - failures,
        cases.len()
    );
    if failures > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
