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
