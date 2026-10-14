//! L2 用例回归（rust-tests 门禁内跑）：golden / ROM 执行 / game-core 同名函数三方全等。
//!
//! golden 输入 = ROM 原始字节切片（fail-closed：ROM 缺失即 FAIL，不静默 skip）。
//! 用例文件 = oracle/subroutines/l2_cases.jsonl（先写用例再实现，TDD 产物）。

use std::path::PathBuf;

fn rom_bytes() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rom/gunnac_cn.nes");
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("读取 ROM 失败（fail-closed）：{:?}: {e}", path))
}

fn load_cases() -> Vec<game_oracle::l2::Case> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../oracle/subroutines/l2_cases.jsonl");
    let jsonl = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读取 case 文件失败：{:?}: {e}", path));
    game_oracle::l2::parse_cases(&jsonl).unwrap_or_else(|e| panic!("case 解析失败：{e}"))
}

#[test]
fn l2_cases_triple_diff_all_green() {
    let rom = rom_bytes();
    let cases = load_cases();
    assert!(!cases.is_empty(), "case 集为空 = L2 门禁空转");
    let mut failures = Vec::new();
    for case in &cases {
        match game_oracle::l2::diff_case(&rom, case) {
            Ok((_, _, problems)) => failures.extend(problems.into_iter().map(|p| format!("{}: {p}", case.id))),
            Err(e) => failures.push(format!("{}: 执行错误 {e}", case.id)),
        }
    }
    assert!(failures.is_empty(), "L2 差分不一致：\n{}", failures.join("\n"));
}

#[test]
fn l2_covers_required_subroutines() {
    let cases = load_cases();
    let subs: std::collections::BTreeSet<&str> = cases.iter().map(|c| c.subroutine.as_str()).collect();
    for required in [
        "player_x_clamp_hi", "player_x_clamp_lo", "player_y_clamp_hi", "player_y_clamp_lo",
        "digit_conversion", "consumption_lookup", "bomb_drop",
        "death_money_halve", "bcd_sub_ammo",
    ] {
        assert!(subs.contains(required), "缺子程序用例：{required}");
    }
}

#[test]
fn cpu_core_rejects_illegal_opcodes() {
    use game_oracle::cpu6502::{Bus, Cpu};
    struct OneByte(u8);
    impl Bus for OneByte {
        fn read(&mut self, _addr: u16) -> u8 {
            self.0
        }
        fn write(&mut self, _addr: u16, _val: u8) {}
    }
    let mut cpu = Cpu::new(0, 0, 0, 0xFD, 0x24, 0x8000);
    let mut bus = OneByte(0x02); // 非法操作码
    assert!(cpu.step(&mut bus).is_err(), "非法操作码必须报错（fail-closed，不猜语义）");
}
