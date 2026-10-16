#[test]
fn bg_persistence_model() {
    use game_desktop::BgModel;
    let mut bg = BgModel::default();
    // 条带行写：目标行更新，他行持久化
    let row = [7u8; 32];
    bg.write_strip_row(5, &row);
    assert_eq!(bg.tiles[5][0], 7);
    assert_eq!(bg.tiles[4][0], 0, "条带外持久化（未写行不动）");
    assert_eq!(bg.tiles[6][0], 0);
    // 条带列写：目标列更新，他列持久化
    let col = [3u8; 30];
    bg.write_strip_col(10, &col);
    assert_eq!(bg.tiles[0][10], 3);
    assert_eq!(bg.tiles[5][10], 3, "行写交叠区=列写覆盖（写序后者胜）");
    assert_eq!(bg.tiles[5][9], 7, "条带外列持久化");
    // 属性写+越界 fail-closed
    bg.write_attr(2, 3, 0x1B);
    assert_eq!(bg.attrs[3][2], 0x1B);
    bg.write_strip_row(30, &row);
    assert_eq!(bg.tiles[29][0], 0, "越界行不写");
}

/// W-C1 位域改写器+调色板影子（fade 真身）。
#[test]
fn palette_rewrite_face() {
    use game_desktop::{rewrite_field, PaletteShadow, REWRITE_CLEAR, REWRITE_EXPAND, REWRITE_SET};
    // 三表 ROM 吻合锚
    assert_eq!(REWRITE_CLEAR, [0xFC, 0xF3, 0xCF, 0x3F]);
    assert_eq!(REWRITE_SET, [0x03, 0x0C, 0x30, 0xC0]);
    assert_eq!(REWRITE_EXPAND, [0x00, 0x55, 0xAA, 0xFF]);
    // 字段改写：field0 写 v=2 ⇒ 低 2bit=10，他域保留
    let mut s = 0xFFu8;
    rewrite_field(&mut s, 0, 2);
    assert_eq!(s, 0xFE, "field0: FF→FE（&FC|AA&03=FC|02）");
    // field2 写 v=1 ⇒ bit4-5=01
    let mut s = 0xFFu8;
    rewrite_field(&mut s, 2, 1);
    assert_eq!(s, 0xDF, "field2: FF→DF（&CF|55&30=CF|10）");
    // 逐域独立性
    let mut s = 0x00u8;
    rewrite_field(&mut s, 0, 3);
    rewrite_field(&mut s, 3, 3);
    assert_eq!(s, 0xC3, "两域置位互不干扰");
    // 影子缓冲批量改写+越界 fail-closed
    let mut sh = PaletteShadow::default();
    sh.rewrite(0, 0, 3);
    assert_eq!(sh.buf[0], 0x03);
    sh.rewrite(32, 0, 3);
    assert_eq!(sh.buf[0], 0x03, "越界不写");
}
