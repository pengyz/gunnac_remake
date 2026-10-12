//! CHR 区 tile 解码（S2/S0 取证语义，投影级；ROM 为真相）。
//!
//! t2 CHR 布局（A 级）：128 × 1KB bank（CHR 总 128KB），bank n = file `0x20010 + n*0x400`；
//! 字模/字体区 bank124-127。2bpp NES tile = 16 字节/块（64 tile/bank），
//! 解码语义与 `tools/chr_render.py` 一致：`c = bit(p0,7-px) | bit(p1,7-px)<<1` → 索引 0..3。
//!
//! 注意（t33 L4 计划）：**调色板映射未证**（C 级）——本模块只解码出 0..3 索引，
//! 颜色映射由调用方提供参数（真调色板待 oracle/screens 对拍锁定，禁止发明）。

pub const CHR_BANK_SIZE: usize = 0x400;
pub const CHR_TILE_SIZE: usize = 16;
pub const CHR_TILES_PER_BANK: usize = CHR_BANK_SIZE / CHR_TILE_SIZE;
pub const CHR_BANKS: usize = 128;
pub const CHR_FILE_BASE: usize = 0x20010;
/// 字模区 bank 起（t2：bank124-127）。
pub const FONT_BANK_START: usize = 124;

/// CHR bank n 的 1KB 切片（bank ∈ [0,128)；越界 fail-closed）。
pub fn chr_bank(rom: &[u8], bank: usize) -> &[u8] {
    assert!(bank < CHR_BANKS, "CHR bank 越界（t2 布局 128×1KB）");
    let start = CHR_FILE_BASE + bank * CHR_BANK_SIZE;
    let end = start + CHR_BANK_SIZE;
    assert!(rom.len() >= end, "ROM 尺寸不足以容纳 CHR bank {bank}");
    &rom[start..end]
}

/// bank 内第 `tile` 块的 16 字节。
pub fn tile_bytes(rom: &[u8], bank: usize, tile: usize) -> [u8; CHR_TILE_SIZE] {
    assert!(tile < CHR_TILES_PER_BANK, "tile 越界（64 tile/bank）");
    let bank_data = chr_bank(rom, bank);
    let mut out = [0u8; CHR_TILE_SIZE];
    out.copy_from_slice(&bank_data[tile * CHR_TILE_SIZE..(tile + 1) * CHR_TILE_SIZE]);
    out
}

/// 2bpp tile 解码：返回 8×8 索引矩阵（行优先，值域 0..3）。
/// 语义 = `tools/chr_render.py`（`c = bit(p0,7-px) | bit(p1,7-px)<<1`）。
pub fn decode_tile(tile: &[u8; CHR_TILE_SIZE]) -> [[u8; 8]; 8] {
    let mut out = [[0u8; 8]; 8];
    for py in 0..8 {
        let (p0, p1) = (tile[py], tile[py + 8]);
        for px in 0..8 {
            out[py][px] = (((p0 >> (7 - px)) & 1) | (((p1 >> (7 - px)) & 1) << 1)) as u8;
        }
    }
    out
}
