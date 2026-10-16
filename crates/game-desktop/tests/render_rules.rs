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

/// R-2 场景调色板记录表（描述符+3B 记录）。
#[test]
fn scene_palette_table_face() {
    use game_desktop::{
        descriptor_palette_base, palette_record_at, PaletteRecord, SCENE_DESCRIPTORS,
    };
    // 描述符表 5 项 ROM 吻合（+2/+3 基址域=$A000 换窗区）
    assert_eq!(SCENE_DESCRIPTORS.len(), 5);
    assert_eq!(descriptor_palette_base(&SCENE_DESCRIPTORS[0]), 0xBEFB);
    assert_eq!(descriptor_palette_base(&SCENE_DESCRIPTORS[1]), 0xC0D2);
    assert_eq!(descriptor_palette_base(&SCENE_DESCRIPTORS[4]), 0xBFFB);
    // 3B 记录取项+越界 fail-closed
    let recs = [0x10, 0x80, 0xF0, 0x10, 0xF0, 0x10];
    assert_eq!(palette_record_at(&recs, 0), Some(PaletteRecord([0x10, 0x80, 0xF0])));
    assert_eq!(palette_record_at(&recs, 1), Some(PaletteRecord([0x10, 0xF0, 0x10])));
    assert_eq!(palette_record_at(&recs, 2), None, "越界 fail-closed");
}

/// R-3 命令流解释器（fade 参数源=流回放）。
#[test]
fn palette_command_stream_face() {
    use game_desktop::{stream_step, StreamStep};
    // 数据+计数对
    let d = [0x4D, 0x02, 0x00];
    assert_eq!(stream_step(&d, 0), Some((StreamStep::Data { value: 0x4D, count: 2 }, 2)));
    // $00 终止
    assert_eq!(stream_step(&d, 2), Some((StreamStep::End, 3)));
    // $80 相对跳转（offset=2 ⇒ pos 跳 0+2+2=4）
    let d2 = [0x80, 0x02, 0x00, 0x00, 0x11];
    assert_eq!(stream_step(&d2, 0), Some((StreamStep::Jump { opcode: 0x80 }, 4)));
    // $81-$FF 循环计数（DEC $B0,X）
    let d3 = [0x83, 0x00];
    assert_eq!(stream_step(&d3, 0), Some((StreamStep::Loop { counter: 0x83 }, 1)));
    // 流数据=NES 色码域（bank1:$9809 样本）
    let sample = [0x07, 0x0F];
    assert_eq!(stream_step(&sample, 0), Some((StreamStep::Data { value: 0x07, count: 0x0F }, 2)));
    // 截断 fail-closed（data 后缺 count）
    assert_eq!(stream_step(&[0x4D], 0), None);
}

/// R-3c fade 流全谱+转场装载时序。
#[test]
fn fade_stream_spectrum_and_loader() {
    use game_desktop::{
        TransitionLoader, FADE_STREAM_SEG1, FADE_STREAM_SEG2, FADE_STREAM_TAIL,
    };
    // 双段 ROM 直读吻合（≤$3F 色码域）
    assert_eq!(FADE_STREAM_SEG1.len(), 11);
    assert_eq!(FADE_STREAM_SEG2.len(), 15);
    assert!(FADE_STREAM_SEG1[..10].iter().all(|&c| c <= 0x3F), "段1 色码域");
    assert!(FADE_STREAM_SEG2[..14].iter().all(|&c| c <= 0x3F), "段2 色码域");
    assert_eq!(FADE_STREAM_SEG1[10], 0x00, "段1 终止");
    assert_eq!(FADE_STREAM_SEG2[14], 0x00, "段2 终止");
    assert_eq!(FADE_STREAM_TAIL, [0x03, 0x01, 0x01]);
    // 转场装载：流 $A7C2+mode2（setup 一次装+逐帧回放）
    let t = TransitionLoader::setup();
    assert_eq!(t.stream, 0xA7C2);
    assert_eq!(t.mode, 2);
    assert!(t.loaded);
}
