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

/// L4 索引族：①tile 校正+⑤type CHR bank。
#[test]
fn index_family_faces() {
    use game_desktop::{tile_correction, type_chr_bank, TILE_CORRECT_9B, TYPE_CHR_BANK};
    // ① tile 校正（CN $DA4C 00×7,C1,C1）
    assert_eq!(TILE_CORRECT_9B, [0, 0, 0, 0, 0, 0, 0, 0xC1, 0xC1]);
    assert_eq!(tile_correction(1), 0);
    assert_eq!(tile_correction(8), 0xC1, "关 8-9 校正");
    assert_eq!(tile_correction(9), 0xC1);
    assert_eq!(tile_correction(0), 0, "越界 fail-closed");
    assert_eq!(tile_correction(10), 0);
    // ⑤ type CHR bank（bank2:$816E 直读——$A0-$BD CHR bank 族）
    assert_eq!(TYPE_CHR_BANK.len(), 256);
    assert_eq!(type_chr_bank(0), 0xA3);
    assert_eq!(type_chr_bank(1), 0xA5);
    assert!(type_chr_bank(0x30) >= 0xA0 || type_chr_bank(0x30) <= 0x07, "bank 值域");
}

/// R-5 碎片组+画面锚。
#[test]
fn fragment_groups_and_screen_anchors() {
    use game_desktop::{FRAGMENT_GROUPS, SCREEN_ANCHORS};
    // 4 组+碎片共享实证（$13/$14 跨组）
    assert_eq!(FRAGMENT_GROUPS.len(), 4);
    assert!(FRAGMENT_GROUPS[1].contains(&0x13) && FRAGMENT_GROUPS[3].contains(&0x13), "13 跨组共享");
    assert!(FRAGMENT_GROUPS[1].contains(&0x14) && FRAGMENT_GROUPS[3].contains(&0x14), "14 跨组共享");
    // 组0=$30 头+01-12 碎（19 字节；碎=18——$30 头注）
    assert_eq!(FRAGMENT_GROUPS[0].len(), 19);
    assert_eq!(FRAGMENT_GROUPS[0][0], 0x30);
    // 画面锚族
    assert!(SCREEN_ANCHORS.contains(&"stage_1"));
    assert!(SCREEN_ANCHORS.contains(&"ending_robot"));
}

/// ⑥ text.csv 文本放置面。
#[test]
fn text_placement_face() {
    use game_desktop::{place_text, ppu_to_nametable, BgModel, TextPlacement};
    // PPU↔nametable 对偶（$20C8⇒(8,6)——csv 行实证）
    assert_eq!(ppu_to_nametable(0x20C8), Some((8, 6)));
    assert_eq!(ppu_to_nametable(0x2146), Some((6, 10)));
    assert_eq!(ppu_to_nametable(0x2400), None, "属性区 fail-closed");
    // 文本放置（tile 码调用方提供）
    let mut bg = BgModel::default();
    let p = TextPlacement {
        group: "settings", ppu: 0x20C8, x: 8, y: 6, text: "건낙 조종실".to_string(),
    };
    place_text(&mut bg, &p, &[0x7C, 0x7E, 0x7C]);
    assert_eq!(bg.tiles[6][8], 0x7C);
    assert_eq!(bg.tiles[6][9], 0x7E);
    assert_eq!(bg.tiles[6][10], 0x7C);
    assert_eq!(bg.tiles[6][7], 0, "前位不动");
}

/// R-4 画面流回放（tile 布局流格式）。
#[test]
fn screen_stream_replay_face() {
    use game_desktop::{screen_step, ScreenStep};
    // [PPU 2B][tiles][FE][PPU 2B][tiles][FF] 格式（状态机——l4 根因修）
    let d = [0xC8, 0x20, 0x2D, 0x2D, 0xFE, 0x46, 0x21, 0x03, 0xFF];
    let (s0, n0) = screen_step(&d, 0, true).unwrap();
    assert_eq!(s0, ScreenStep::PpuAddr { ppu: 0x20C8 });
    let (s1, n1) = screen_step(&d, n0, false).unwrap();
    assert_eq!(s1, ScreenStep::Tile { code: 0x2D }, "tile 码（横线「-------」注记域）");
    assert_eq!(screen_step(&d, n1, false).unwrap().0, ScreenStep::Tile { code: 0x2D });
    assert_eq!(screen_step(&d, 4, false).unwrap().0, ScreenStep::Reload);
    let (s4, n4) = screen_step(&d, 5, true).unwrap();
    assert_eq!(s4, ScreenStep::PpuAddr { ppu: 0x2146 }, "FE 后新地址");
    assert_eq!(screen_step(&d, n4, false).unwrap().0, ScreenStep::Tile { code: 0x03 });
    assert_eq!(screen_step(&d, 8, true).unwrap().0, ScreenStep::SegmentEnd);
    // 截断 fail-closed
    assert_eq!(screen_step(&[0xC8], 0, true), None);
}

/// fade 线端到端组合（四件套：流→解释器→改写器→影子——R-2/R-3/R-3b/R-3c 链）。
#[test]
fn fade_line_end_to_end() {
    use game_desktop::{
        rewrite_field, stream_step, PaletteShadow, StreamStep, FADE_STREAM_SEG1,
    };
    // 流段 1 逐字节回放→位域改写影子（值=色码，域=调色板 2bit×4）
    let mut sh = PaletteShadow::default();
    let mut pos = 0;
    let mut steps = 0;
    while let Some((step, next)) = stream_step(&FADE_STREAM_SEG1, pos) {
        pos = next;
        match step {
            StreamStep::End => break,
            StreamStep::Data { value, count } => {
                for k in 0..count.min(32) {
                    // 值→4 域逐域改写（模拟 8619 参数装→改写器——域序 0..3）
                    for f in 0..4u8 {
                        rewrite_field(&mut sh.buf[k as usize % 32], f, value & 3);
                    }
                }
                steps += 1;
            }
            _ => {}
        }
    }
    // R-3b 对格式读法（[data][count] 对）：10 字节=5 对+终止（「10 码」=字节数）
    assert_eq!(steps, 5, "段1 五对回放（$00 终止）");
    // 影子终态=末对值 0x06 低 2bit=10（逐域改写收敛——0x30=count 非值注）
    assert_eq!(sh.buf[0] & 0x03, 0x02, "末值 0x06&3=02 位域收敛");
}

/// R-5b 关名合成链（三线合龙）。
#[test]
fn composition_chain_face() {
    use game_desktop::{
        comp_step, composition_stream_ptr, f_decode, CompStep, COMP_PTR_HI, COMP_PTR_LO,
    };
    // 双表 ROM 吻合+指针（$518 形状 0 ⇒ $986A 流）
    assert_eq!(composition_stream_ptr(0), Some(0x986A));
    assert_eq!(composition_stream_ptr(7), Some(0x98EA));
    assert_eq!(composition_stream_ptr(16), None, "越界 fail-closed");
    assert_eq!(COMP_PTR_LO[0], 0x6A);
    assert_eq!(COMP_PTR_HI[0], 0x98);
    // F 编码（补码 i8）
    assert_eq!(f_decode(0xFF), -1);
    assert_eq!(f_decode(0xFA), -6);
    assert_eq!(f_decode(0xF4), -12);
    // 组合流：01 头+嵌套指针检测（$9941）
    let stream = [0x01, 0x01, 0xFF, 0x41, 0x99];
    assert_eq!(comp_step(&stream, 0), Some((CompStep::Header, 1)));
    assert_eq!(comp_step(&stream, 1).unwrap().0, CompStep::Displacement { dx: 1, dy: -1 });
    assert_eq!(comp_step(&stream, 3).unwrap().0, CompStep::Nested { ptr: 0x9941 });
    // 截断 fail-closed
    assert_eq!(comp_step(&[0x01], 1), None);
}

/// R-5b-2 位移网格链（组合链末环）。
#[test]
fn displacement_grid_chain() {
    use game_desktop::parse_disp_grid;
    // 位移对+尾三字节（count,chain）
    let d = [0x01, 0xFF, 0xFA, 0x06, 0x03, 0xAA, 0x98];
    let g = parse_disp_grid(&d).unwrap();
    assert_eq!(g.pairs, vec![(1, -1), (-6, 6)]);
    assert_eq!(g.count, 3);
    assert_eq!(g.chain, Some(0x98AA), "链指针（$9895→$98AA 链续实证形）");
    // 链尾（hi=0 ⇒ None）
    let d2 = [0x01, 0x01, 0x00, 0x00, 0x00];
    let g2 = parse_disp_grid(&d2).unwrap();
    assert_eq!(g2.chain, None);
    // 截断 fail-closed
    assert_eq!(parse_disp_grid(&[0x01]), None);
    assert_eq!(parse_disp_grid(&[0x01, 0x02, 0x03, 0x04]), None, "奇数位移对界 fail-closed");
}

/// ①内容投影管线：屏流→BgModel 全量回放。
#[test]
fn screen_stream_projection() {
    use game_desktop::{project_screen_stream, BgModel};
    // [PPU 20C8][tile 7C 7E][FE][PPU 2146][tile 03][FF]
    let d = [0xC8, 0x20, 0x7C, 0x7E, 0xFE, 0x46, 0x21, 0x03, 0xFF];
    let mut bg = BgModel::default();
    let n = project_screen_stream(&mut bg, &d);
    assert_eq!(n, 3, "三 tile 投影");
    assert_eq!(bg.tiles[6][8], 0x7C);
    assert_eq!(bg.tiles[6][9], 0x7E, "$2007 自增");
    assert_eq!(bg.tiles[10][6], 0x03, "FE 后新地址投影");
    // 持久化：未覆盖区不动
    assert_eq!(bg.tiles[0][0], 0);
}

/// 收官一环：组合渲染器（位移位置+序瓦）。
#[test]
fn composition_renderer() {
    use game_desktop::{composition_render, BgModel};
    let mut bg = BgModel::default();
    let disp = [(0i8, 0i8), (1, 0), (2, 0), (0, 1)];
    let n = composition_render(&mut bg, &disp, 1, (6, 4));
    assert_eq!(n, 4);
    assert_eq!(bg.tiles[4][6], 1);
    assert_eq!(bg.tiles[4][7], 2, "序瓦 run");
    assert_eq!(bg.tiles[4][8], 3);
    assert_eq!(bg.tiles[5][6], 4, "位移 (0,1)");
    // 越界 fail-closed
    let mut bg2 = BgModel::default();
    let n2 = composition_render(&mut bg2, &[(-1, 0), (0, -1)], 1, (0, 0));
    assert_eq!(n2, 0, "越界不写");
}
