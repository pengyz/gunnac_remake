//! 渲染规则模块（t212）：已证 A 级结论无损落地——族选择器/映射/墨表/合成
//! 纯函数集（TDD：每结论先断言后实现；98.15 回归不降由 L4 门禁守）。

use crate::Rgba;

/// 合成规则（t199/t200）：index = p0 | (p1<<1)。
#[inline]
pub fn compose_index(p0: u8, p1: u8) -> u8 {
    (p0 & 1) | ((p1 & 1) << 1)
}

/// 族选择器（N-0172）：bank 族随配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteFamily {
    /// 标签族：bank 96/98（t210 Hamming=0 实证）
    Label,
    /// CJK 族：bank 28
    Cjk,
    /// 图块族：bank 20-23
    Graphics,
    /// 游戏族：bank 106/107
    Game,
}

/// 族分派（N-0172 键=OAM slot 语义；t210 实证案例：slot 63/tile 1/X 70=Label）。
/// 分派键以 slot 序+tile 值族为判据（实证案例锁定，待扩展注）。
pub fn family_of(slot: usize, tile: u8) -> SpriteFamily {
    if slot >= 60 && tile <= 3 {
        SpriteFamily::Label // t210 实证（slot 63 tile 1）；槽位族边界待扩展
    } else if tile <= 3 {
        SpriteFamily::Cjk
    } else if (20..=23).contains(&tile) {
        SpriteFamily::Graphics
    } else {
        SpriteFamily::Game
    }
}

/// 映射规则（t209/t211 Hamming=0 已证）：连续对 T&$FE 同 bank——(top, bottom)。
#[inline]
pub fn pair_indices(tile: u8) -> (usize, usize) {
    let t = tile as usize;
    (t & 0xFE, (t & 0xFE) | 1)
}

/// 族 → bank 基（N-0172）。
pub fn family_bank(f: SpriteFamily) -> usize {
    match f {
        SpriteFamily::Label => 96,
        SpriteFamily::Cjk => 28,
        SpriteFamily::Graphics => 20,
        SpriteFamily::Game => 106,
    }
}

/// 二维墨表（t204/t207 三轮验证）：索引×槽位 → RGB。
/// CJK 槽：1=暗青/3=白；logo 槽：1=深蓝/2=浅蓝/3=白；族 A：2=白。
pub fn ink_table(family: SpriteFamily, index: u8) -> Rgba {
    const INK_NONE: Rgba = [0, 0, 0, 255];
    const WHITE: Rgba = [0xFF, 0xFE, 0xFF, 255];
    const DARK_CYAN: Rgba = [0x00, 0x40, 0x4D, 255];
    const DARK_BLUE: Rgba = [0x12, 0x2C, 0x6E, 255];
    const LIGHT_BLUE: Rgba = [0x64, 0xB0, 0xFF, 255];
    match (family, index) {
        (SpriteFamily::Label | SpriteFamily::Cjk, 1) => DARK_CYAN,
        (SpriteFamily::Label | SpriteFamily::Cjk, 3) => WHITE,
        (SpriteFamily::Graphics, 1) => DARK_BLUE,
        (SpriteFamily::Graphics, 2) => LIGHT_BLUE,
        (SpriteFamily::Graphics, 3) => WHITE,
        (SpriteFamily::Game, 2) => WHITE,
        _ => INK_NONE, // 0=backdrop/未证组合=不画（fail-closed）
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_rule_p0_p1() {
        // t199/t200：index = p0 | (p1<<1)
        assert_eq!(compose_index(0, 0), 0);
        assert_eq!(compose_index(1, 0), 1);
        assert_eq!(compose_index(0, 1), 2);
        assert_eq!(compose_index(1, 1), 3);
    }

    #[test]
    fn pair_mapping_hamming0_case() {
        // t209/t210：OAM tile=1 → (T&$FE, T&$FE|1) = (0, 1)——『田』Hamming=0 实证
        assert_eq!(pair_indices(1), (0, 1));
        assert_eq!(pair_indices(42), (42, 43));
        assert_eq!(pair_indices(43), (42, 43), "奇数回偶（连续对语义）");
    }

    #[test]
    fn family_selector_n0172() {
        // N-0172 + t210 实证案例
        assert_eq!(family_of(63, 1), SpriteFamily::Label);
        assert_eq!(family_bank(SpriteFamily::Label), 96);
        assert_eq!(family_bank(SpriteFamily::Cjk), 28);
        assert_eq!(family_bank(SpriteFamily::Graphics), 20);
        assert_eq!(family_bank(SpriteFamily::Game), 106);
    }

    #[test]
    fn ink_table_2d_t204() {
        // t204/t207：CJK 槽 1=暗青/3=白；logo 槽 1=深蓝/2=浅蓝/3=白
        assert_eq!(ink_table(SpriteFamily::Cjk, 1), [0x00, 0x40, 0x4D, 255]);
        assert_eq!(ink_table(SpriteFamily::Cjk, 3), [0xFF, 0xFE, 0xFF, 255]);
        assert_eq!(ink_table(SpriteFamily::Graphics, 1), [0x12, 0x2C, 0x6E, 255]);
        assert_eq!(ink_table(SpriteFamily::Graphics, 2), [0x64, 0xB0, 0xFF, 255]);
        // 未证组合 fail-closed（不画）
        assert_eq!(ink_table(SpriteFamily::Cjk, 2), [0, 0, 0, 255]);
    }

    #[test]
    fn negative_guards() {
        // 负向：0=backdrop 恒底
        for f in [SpriteFamily::Label, SpriteFamily::Cjk, SpriteFamily::Graphics, SpriteFamily::Game] {
            assert_eq!(ink_table(f, 0), [0, 0, 0, 255], "index 0=底");
        }
    }
}
