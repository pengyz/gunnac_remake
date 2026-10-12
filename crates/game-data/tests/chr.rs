//! CHR 解码 golden（bank124 tile2 = 字模区实字节手推矩阵）。

use game_data::chr::{chr_bank, decode_tile, tile_bytes, CHR_BANKS, CHR_FILE_BASE};

#[test]
fn chr_bank_offset_matches_t2_layout() {
    // bank n = file 0x20010 + n*0x400（t2 A 级）
    assert_eq!(CHR_FILE_BASE, 0x20010);
    let rom = vec![0u8; CHR_FILE_BASE + 128 * 0x400];
    assert_eq!(chr_bank(&rom, 124).len(), 0x400);
    assert_eq!(chr_bank(&rom, CHR_BANKS - 1).len(), 0x400);
}

#[test]
fn decode_tile_golden_font_bank124_tile2() {
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM 可读");
    let t = tile_bytes(&rom, 124, 2);
    assert_eq!(t, [0x00,0x00,0x00,0x7c,0xff,0xff,0xe7,0xe7,0x00,0x00,0x00,0x7c,0xfe,0xc6,0xc6,0xc6]);
    let g = decode_tile(&t);
    // 手推矩阵（2bpp：c = bit(p0,7-px) | bit(p1,7-px)<<1）
    assert_eq!(g[0], [0,0,0,0,0,0,0,0]);
    assert_eq!(g[3], [0,3,3,3,3,3,0,0]);
    assert_eq!(g[4], [3,3,3,3,3,3,3,1]);
    assert_eq!(g[5], [3,3,1,1,1,3,3,1]);
    assert_eq!(g[6], [3,3,1,0,0,3,3,1]);
    assert_eq!(g[7], [3,3,1,0,0,3,3,1]);
}

#[test]
fn decode_tile_plane_semantics() {
    // 平面语义单测：p0=低位平面 p1=高位平面
    let mut t = [0u8; 16];
    t[0] = 0b1000_0000; // px0 低位
    t[8] = 0b0100_0000; // px1 高位
    let g = decode_tile(&t);
    assert_eq!(g[0], [1, 2, 0, 0, 0, 0, 0, 0]);
}
