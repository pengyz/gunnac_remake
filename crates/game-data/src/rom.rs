//! ROM 文件布局只读接口（iNES 容器 → PRG bank 窗口）。
//!
//! 证据（ROM 文件布局 = iNES 容器事实 + 仓库既有逐字节读取点，A 级）：
//! - iNES 头 16B（`0x10`），PRG ROM 自 `0x10` 起、8KB 为一 bank；
//! - 仓库同口径读取点：`tools/wave_timeline.py::fb`（`0x10 + bank*0x2000 + (addr-$A000)`）、
//!   `crates/game-core/tests/engine_tick.rs::bank8_window`（`0x10 + 8*0x2000`）；
//! - MMC3 R7 数据 bank 把 8KB PRG bank 映射到 CPU `$8000`/`$A000` 窗（bank8 = `$A000-$BFFF` 数据窗）。
//!
//! 本模块只做**只读切片/边界检查**，不解析 NES 头字段、不做映射/校验（未证不发明）。

/// iNES 容器头长度（16 字节）。
pub const INES_HEADER_LEN: usize = 0x10;

/// 8KB PRG bank 大小（MMC3 R7 数据 bank 窗口）。
pub const PRG_BANK_SIZE: usize = 0x2000;

/// 取 PRG bank 窗口（`file = 0x10 + bank*0x2000`，长度 `PRG_BANK_SIZE`）。
///
/// 越界/ROM 过短 ⇒ `None`（fail-closed；不返回补零/截断内容）。
pub fn prg_bank(rom: &[u8], bank: usize) -> Option<&[u8]> {
    let start = INES_HEADER_LEN.checked_add(bank.checked_mul(PRG_BANK_SIZE)?)?;
    let end = start.checked_add(PRG_BANK_SIZE)?;
    rom.get(start..end)
}

#[cfg(test)]
mod tests {
    use super::{prg_bank, INES_HEADER_LEN, PRG_BANK_SIZE};

    fn synthetic_rom(banks: usize) -> Vec<u8> {
        let mut rom = vec![0u8; INES_HEADER_LEN + banks * PRG_BANK_SIZE];
        for bank in 0..banks {
            let base = INES_HEADER_LEN + bank * PRG_BANK_SIZE;
            rom[base] = bank as u8;
            rom[base + PRG_BANK_SIZE - 1] = (bank as u8).wrapping_add(0x80);
        }
        rom
    }

    #[test]
    fn prg_bank_slices_exact_window() {
        let rom = synthetic_rom(10);
        let b8 = prg_bank(&rom, 8).expect("bank8 存在");
        assert_eq!(b8.len(), PRG_BANK_SIZE);
        assert_eq!(b8[0], 8, "头字节=bank 标记");
        assert_eq!(b8[PRG_BANK_SIZE - 1], 0x88, "尾字节=bank 标记");
        assert_eq!(prg_bank(&rom, 0).unwrap()[0], 0);
    }

    #[test]
    fn prg_bank_fail_closed_out_of_range() {
        let rom = synthetic_rom(10);
        assert!(prg_bank(&rom, 10).is_none(), "越界 bank ⇒ None");
        assert!(prg_bank(&rom, usize::MAX).is_none(), "溢出 bank ⇒ None");
        assert!(prg_bank(&[], 0).is_none(), "空 ROM ⇒ None");
        assert!(prg_bank(&rom[..INES_HEADER_LEN + PRG_BANK_SIZE - 1], 0).is_none(), "短 ROM ⇒ None");
    }
}
