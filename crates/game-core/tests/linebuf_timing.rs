//! R-l4-linebuf 不变量测试（t96 ①⑤；几何按 t98 R-l4-linebuf-6 定谳——
//! 目标地址=$8F:($8E+$0F)，4 组×8B 步进 +$20）+ 负向自证。

use game_core::terrain::Nametable;

#[test]
fn invariant_upload_geometry_linebuf6() {
    // 不变量 2/6：NMI 上传 = 4 组×8B（基址 $8F:($8E+$0F)，组间 +$20）
    let mut nt = Nametable::default();
    let mut row = [0u8; 32];
    for (i, b) in row.iter_mut().enumerate() {
        *b = (i + 1) as u8; // 1..32 每字节可辨
    }
    nt.nmi_flush(&row, &[0; 8], 0xC0, 0x22, 0xE8, 1); // 基址 $22C0+$0F=$22CF
    // 组 0 → $22CF..（rel 0x2CF）：row_buf[0..8] = 1..8
    assert_eq!(nt.tiles[0][0x2CF], 1);
    assert_eq!(nt.tiles[0][0x2D6], 8);
    // 组 1 → +$20（rel 0x2EF）：row_buf[8..16] = 9..16
    assert_eq!(nt.tiles[0][0x2EF], 9);
    // 组 3 → +$60（rel 0x32F）：row_buf[24..32] = 25..32
    assert_eq!(nt.tiles[0][0x32F], 25);
    assert_eq!(nt.tiles[0][0x336], 32);
    // 读后即废（不变量 3 等效）：缓冲自身不被清零
    assert_eq!(row[0], 1);
}

#[test]
fn invariant_pending_type_and_palette_source() {
    // 不变量 3/4：$160=挂起类型（0=无、3=调色板）；调色板源 $161-$167
    let mut nt = Nametable::default();
    let row = [1u8; 32];
    nt.nmi_flush(&row, &[0; 8], 0xC0, 0x22, 0xE8, 0); // pending=0 不上传
    assert_eq!(nt.tiles[0][0x2CF], 0);
    // 属性源域（$D3-$DA → ($8F|3):$90 + $0F 偏移）
    nt.nmi_flush(&row, &[0xE4; 8], 0xC0, 0x22, 0xE8, 3);
    assert_eq!(nt.attrs[0][(0x23F7 - 0x23C0) as usize], 0xE4, "属性基址 $23E8+$0F=$23F7");
}

#[test]
fn linebuf_negative_selfproof() {
    // 负向自证：非 PPU 域地址（相位陷阱值）不得写入
    let mut nt = Nametable::default();
    nt.nmi_flush(&[9u8; 32], &[0; 8], 0x00, 0x00, 0x00, 3);
    assert_eq!(nt.tiles[0][0x0F], 0, "相位陷阱地址写入必红");
    assert_eq!(nt.tiles[0][0], 0);
    // 负向自证 2：错几何必红（组步进/偏移哨兵）
    let mut nt2 = Nametable::default();
    nt2.nmi_flush(&[7u8; 32], &[0; 8], 0xC0, 0x22, 0xE8, 1);
    assert_eq!(nt2.tiles[0][0x2CE], 0, "$22CF 前一字节不得有内容（偏移哨兵）");
    assert_ne!(nt2.tiles[0][0x2CF], 0, "基址字节必有内容");
}

#[test]
fn text_direct_address_system_and_negative() {
    // t115 ①：文本直址体系（t2 记录内嵌 PPU 地址直写）——N-0127 根因修复
    use game_core::terrain::{place_text_records, Nametable, TextPlacement};
    let mut nt = Nametable::default();
    let recs = vec![TextPlacement { ppu_addr: 0x2345, tiles: vec![7, 8, 9] }];
    place_text_records(&mut nt, &recs);
    assert_eq!(nt.tiles[0][0x345], 7);
    assert_eq!(nt.tiles[0][0x347], 9);
    // 负向自证：非 PPU 域地址 fail-closed
    let mut nt2 = Nametable::default();
    place_text_records(&mut nt2, &[TextPlacement { ppu_addr: 0x1234, tiles: vec![5] }]);
    assert_eq!(nt2.tiles[0][0], 0, "非 PPU 域必红");
    // 负向自证 2：跨行连续写（$2007 自增语义）
    let mut nt3 = Nametable::default();
    place_text_records(&mut nt3, &[TextPlacement { ppu_addr: 0x23BE, tiles: vec![4, 5, 6] }]);
    assert_eq!(nt3.tiles[0][0x3BE], 4);
    assert_eq!(nt3.tiles[0][0x3BF], 5, "跨至属性区前界");
}

#[test]
fn text_channel_four_stage_and_budget() {
    use game_core::terrain::*;
    // 地址合成三类字节（§4 参数 2）
    assert!(matches!(classify_text_byte(0x30), TextByte::Data(_)));
    assert!(matches!(classify_text_byte(0x10), TextByte::Offset(_)));
    assert!(matches!(classify_text_byte(0xFE), TextByte::Control(_)));
    assert!(matches!(classify_text_byte(0x80), TextByte::Control(_)));
    // 数据合成：(byte & $1F) + $14 | $20
    assert_eq!(synth_addr_lo(0x30, 0x00), 0x30, "(0x10)+0 | 0x20");
    assert_eq!(synth_addr_lo(0x2F, 0x08), 0x37, "(0x0F)+8=0x17 | 0x20 = 0x37");
    assert_ne!(synth_addr_lo(0x30, 0x00), 0x10, "ORA #$20 不可丢（负向）");
    // 预算门（$E828）：$7E? $30 : $50；−count < 4 ⇒ flush
    let mut ch = TextChannel { tier7e: false, ..Default::default() };
    assert_eq!(vram_budget(false), 0x50);
    assert_eq!(vram_budget(true), 0x30);
    ch.count = 0x4D;
    assert!(budget_needs_flush(&ch), "$50−$4D=3<4 必 flush");
    ch.count = 0x40;
    assert!(!budget_needs_flush(&ch), "$50−$40=16 不 flush");
    ch.tier7e = true;
    ch.count = 0x2D;
    assert!(budget_needs_flush(&ch), "$30 档同语义");
    // 单字节推进：数据→写地址+计数；偏移→续址；$FE→指针推进
    let mut ch2 = TextChannel { addr_lo: 0x00, addr_hi: 0x23, ..Default::default() };
    let w = text_channel_byte(&mut ch2, 0x35);
    assert_eq!(w, Some((0x2335, 0x35)));
    assert_eq!(ch2.count, 1);
    let w2 = text_channel_byte(&mut ch2, 0x04); // 偏移（ADC $14 续址）
    assert_eq!(w2, None);
    assert_eq!(ch2.addr_lo, 0x04, "数据不自增 $14（位置=字节低 5 位）；偏移 +4 续址");
    let mut ch3 = TextChannel { stream_lo: 0xFF, stream_hi: 0x96, ..Default::default() };
    text_channel_byte(&mut ch3, 0xFE);
    assert_eq!((ch3.stream_lo, ch3.stream_hi), (0x00, 0x97), "指针进位推进（$E6DF）");
}
