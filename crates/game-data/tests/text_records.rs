//! L1 golden：文本记录解析器 —— golden 输入 = ROM 原始字节切片，期望 = 已证结构。
//!
//! 格式证据（A 级）：`docs/findings/data-tables.md` §2——
//! 记录 = `{ppu_lo, ppu_hi, tile 码流…, 终止字节}`，终止字节 ≥ $FE；
//! `bank15:E6EF` `LDA ($12),Y / CMP #$FE / BCC …` ⇒ 字节 < $FE = tile 码；
//! $FE = 记录结束继续下一条（`bank00:96DD`），$FF = 整块结束（`bank00:96E1`）。
//! 记录实录见同文件 §1（商店面板 4 槽位表，与本测试 golden 逐字节一致）。
//!
//! 运行依赖 `rom/gunnac_cn.nes`（sha256 锚点见 data/status/baselines/SHA256SUMS）：
//! golden 必须来自真实 ROM 字节，缺失即 FAIL（fail-closed，禁静默 skip）。

use game_data::text::{parse_text_block, RecordEnd, TextParseError, TextRecord};
use std::path::PathBuf;

fn rom_bytes() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rom/gunnac_cn.nes");
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!("读取 ROM 失败（golden 输入=ROM 原始字节切片，fail-closed）：{:?}: {}", path, e)
    })
}

fn rec(ppu: u16, tiles: &[u8], end: RecordEnd) -> TextRecord {
    TextRecord { ppu_addr: ppu, tiles: tiles.to_vec(), end }
}

/// 商店面板块（file 0x22BB..0x2329，16 条记录）——期望逐条对应
/// `docs/findings/data-tables.md` §1 的 4 槽位表（名称/价格/尾随记录）。
#[test]
fn golden_shop_panel_block_matches_documented_table() {
    let rom = rom_bytes();
    let slice = &rom[0x22BB..0x2329];
    let block = parse_text_block(slice).expect("商店面板块解析失败");
    assert_eq!(block.consumed, slice.len(), "整块应恰好消费到 FF 块尾");
    assert_eq!(block.end, RecordEnd::BlockEnd);
    assert_eq!(
        block.records,
        vec![
            rec(0x2223, &[0xC6, 0xC7, 0xC8], RecordEnd::Continue),
            rec(0x2243, &[0xD6, 0xD7, 0xD8, 0x3F], RecordEnd::Continue),
            rec(0x2267, &[0xAD, 0xAE, 0xAF], RecordEnd::Continue),
            rec(0x2275, &[0x9F, 0xC8], RecordEnd::Continue),
            // 槽 1 名称："1." + 3 字形（findings §1 $2285 记录）
            rec(0x2285, &[0x31, 0x2E, 0xBD, 0xBE, 0xBF], RecordEnd::Continue),
            // 槽 1 价格："1 <D8> $10"（$2294 记录）
            rec(0x2294, &[0x31, 0x20, 0xD8, 0x20, 0x24, 0x31, 0x30], RecordEnd::Continue),
            rec(0x22A7, &[0xA3, 0xA4, 0xA5], RecordEnd::Continue),
            rec(0x22B5, &[0x9F, 0xC8], RecordEnd::Continue),
            rec(0x22C5, &[0x32, 0x2E, 0x93, 0x94, 0x95], RecordEnd::Continue),
            rec(0x22D4, &[0x31, 0x20, 0xD8, 0x20, 0x24, 0x36], RecordEnd::Continue),
            rec(0x22E7, &[0xE9, 0xEA, 0xEB], RecordEnd::Continue),
            rec(0x22F5, &[0x9F, 0xC8], RecordEnd::Continue),
            rec(0x2305, &[0x33, 0x2E, 0xF9, 0xFA, 0xFB], RecordEnd::Continue),
            rec(0x2314, &[0x31, 0x20, 0xD8, 0x20, 0x24, 0x34], RecordEnd::Continue),
            rec(0x2327, &[0xCB, 0xCC], RecordEnd::Continue),
            // 槽 4 名称：FF 截断（E-R05 现象在记录级复现，findings §1）
            rec(0x2345, &[0x34, 0x2E, 0xDB, 0xDC], RecordEnd::BlockEnd),
        ]
    );
}

/// FF 填充之后的第二块（file 0x232F..0x2342）。
#[test]
fn golden_second_panel_block() {
    let rom = rom_bytes();
    let slice = &rom[0x232F..0x2342];
    let block = parse_text_block(slice).expect("第二块解析失败");
    assert_eq!(block.consumed, slice.len());
    assert_eq!(block.end, RecordEnd::BlockEnd);
    assert_eq!(
        block.records,
        vec![
            rec(0x226D, &[0x0E, 0x0F, 0x20, 0x20, 0x0E, 0x0F], RecordEnd::Continue),
            rec(0x228D, &[0x1E, 0x1F, 0x20, 0x20, 0x1E, 0x1F, 0x2E], RecordEnd::BlockEnd),
        ]
    );
}

/// 金钱行记录（file 0x213F..0x214D）：`F6 F7 F8` + `-------$`（$2D×7 + $24）。
/// findings §1 E-R04 引用的 file 0x2144 是记录内 $2D 游程位置；记录起点 = 0x213F。
#[test]
fn golden_money_row_record() {
    let rom = rom_bytes();
    let slice = &rom[0x213F..0x214D];
    let block = parse_text_block(slice).expect("金钱行记录解析失败");
    assert_eq!(block.consumed, slice.len());
    assert_eq!(
        block.records,
        vec![rec(
            0x2203,
            &[0xF6, 0xF7, 0xF8, 0x2D, 0x2D, 0x2D, 0x2D, 0x2D, 0x2D, 0x2D, 0x24],
            RecordEnd::BlockEnd
        )]
    );
}

/// 手工夹具：FE=记录结束继续，FF=整块结束。
#[test]
fn fixture_fe_continues_and_ff_stops() {
    let block = parse_text_block(&[0x00, 0x20, 0xAA, 0xFE, 0x01, 0x20, 0xBB, 0xFF])
        .expect("夹具解析失败");
    assert_eq!(
        block.records,
        vec![
            rec(0x2000, &[0xAA], RecordEnd::Continue),
            rec(0x2001, &[0xBB], RecordEnd::BlockEnd),
        ]
    );
    assert_eq!(block.consumed, 8);
}

/// 手工夹具：tile 流控制字节（$5E/$DE…）在解析级保持原样（语义是渲染级的，findings §2）；
/// 空 tile 流也合法（ppu 后直接终止）。
#[test]
fn fixture_control_bytes_stay_raw_in_tile_stream() {
    let block = parse_text_block(&[0x00, 0x20, 0x00, 0x5E, 0xDE, 0xDF, 0xFD, 0xFE, 0x01, 0x20, 0xFF])
        .expect("夹具解析失败");
    assert_eq!(
        block.records,
        vec![
            rec(0x2000, &[0x00, 0x5E, 0xDE, 0xDF, 0xFD], RecordEnd::Continue),
            rec(0x2001, &[], RecordEnd::BlockEnd),
        ]
    );
    assert_eq!(block.consumed, 11);
}

/// 截断：tile 流没有 ≥FE 终止即耗尽 ⇒ 错误（禁半截记录静默通过）。
#[test]
fn truncated_stream_is_error() {
    assert_eq!(
        parse_text_block(&[0x00, 0x20, 0xAA]),
        Err(TextParseError::TruncatedStream { at: 0 })
    );
}

/// 截断：记录头只剩 1 字节 ⇒ 错误。
#[test]
fn truncated_header_is_error() {
    assert_eq!(
        parse_text_block(&[0x00, 0x20, 0xAA, 0xFE, 0x01]),
        Err(TextParseError::TruncatedHeader { at: 4 })
    );
}

/// 空输入不是合法块。
#[test]
fn empty_input_is_error() {
    assert_eq!(parse_text_block(&[]), Err(TextParseError::EmptyInput));
}
