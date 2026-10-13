//! R-l4-linebuf 五不变量测试（t96 ①⑤）：移植侧行缓冲时序对齐 + 负向自证。

use game_core::terrain::Nametable;

#[test]
fn invariant_nmi_upload_then_discard() {
    // 不变量 2/3：NMI vblank 上传（4 组×8B 步进 +$20）后缓冲读后即废（无清零——模型侧等效）
    let mut nt = Nametable::default();
    let mut row = [0u8; 32];
    row[0] = 7;
    nt.nmi_flush(&row, &row[0..8].try_into().unwrap(), 0xC0, 0x22, 0xE8, 1);
    assert_eq!(nt.tiles[0][22 * 32], 7, "addr $22C0 = 行 22");
    assert_eq!(row[0], 7, "读后即废：缓冲自身不被清零（不变量 3 等效）");
}

#[test]
fn invariant_pending_type_and_palette_source() {
    // 不变量 3/4：$160=挂起类型（0=无、3=调色板）；调色板源 $161-$167（模型侧注记）
    let mut nt = Nametable::default();
    let row = [1u8; 32];
    nt.nmi_flush(&row, &[0; 8], 0xC0, 0x22, 0xE8, 0); // pending=0 不上传
    assert_eq!(nt.tiles[0][22 * 32], 0);
    // 属性源域（$D3-$DA → $23C0+）
    nt.nmi_flush(&row, &[0xE4; 8], 0xC0, 0x22, 0xE8, 3);
    assert_eq!(nt.attrs[0][(0x23E8 - 0x23C0) as usize], 0xE4);
}

#[test]
fn linebuf_negative_selfproof() {
    // 负向自证：非 PPU 域地址（快照相位陷阱值 $0000）不得写入
    let mut nt = Nametable::default();
    nt.nmi_flush(&[9u8; 32], &[0; 8], 0x00, 0x00, 0x00, 3);
    assert_eq!(nt.tiles[0][0], 0, "相位陷阱地址写入必红");
    // 越行界（col≠0）不得整块写
    let mut nt2 = Nametable::default();
    nt2.nmi_flush(&[9u8; 32], &[0; 8], 0x08, 0x22, 0xE8, 1);
    assert_eq!(nt2.tiles[0][14 * 32], 0, "非对齐行写入必红");
}
