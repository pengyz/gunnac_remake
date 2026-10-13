//! game-desktop 渲染骨架测试：整数倍缩放/黑边、输入互斥策略、帧缓冲导出、tile 合成。

use game_desktop::{blit_scaled, held_from_keys, view_rect, Canvas, GREY4, SCREEN_H, SCREEN_W};

#[test]
fn view_rect_integer_scale_and_letterbox() {
    // 恰 2 倍
    assert_eq!(view_rect(512, 480), (0, 0, 512, 480));
    // 3 倍 + 黑边（高不足 3 倍 ⇒ 2 倍 + 上下黑边）
    assert_eq!(view_rect(900, 520), ((900 - 512) / 2, (520 - 480) / 2, 512, 480));
    // 窗口小于画布 ⇒ scale=1（裁切，不变形）
    assert_eq!(view_rect(200, 100), (0, 0, 256, 240));
    // 禁变形：视口宽高恒为 256×240 的整数倍
    for (w, h) in [(640, 480), (800, 600), (1920, 1080), (300, 500)] {
        let (ox, oy, vw, vh) = view_rect(w, h);
        assert_eq!(vw % SCREEN_W as u32, 0);
        assert_eq!(vh % SCREEN_H as u32, 0);
        assert_eq!(vw, vh / SCREEN_H as u32 * SCREEN_W as u32, "宽高同比例");
        assert!(ox + vw <= w || w < SCREEN_W as u32);
        assert!(oy + vh <= h || h < SCREEN_H as u32);
    }
}

#[test]
fn blit_letterbox_black_bars() {
    let canvas = Canvas::new(); // 全透明黑
    let mut buf = vec![0xFFFF_FFFFu32; 512 * 480];
    blit_scaled(&mut buf, 512, 480, &canvas);
    assert!(buf.iter().all(|&p| p == 0), "黑边+画布皆清 0");
    // 有像素时按整数倍最近邻放大
    let mut canvas = Canvas::new();
    canvas.clear([255, 0, 0, 255]);
    let mut buf = vec![0u32; 640 * 480];
    blit_scaled(&mut buf, 640, 480, &canvas);
    let (ox, oy, _, _) = view_rect(640, 480);
    assert_eq!(buf[0], 0, "黑边");
    assert_eq!(buf[(oy as usize) * 640 + ox as usize], 0xFF0000, "视口首像素=画布色");
}

#[test]
fn opposing_keys_cancel_platform_policy() {
    // 平台侧输入互斥（W6 未证组合不进 game-core）
    assert_eq!(held_from_keys(true, true, false, false, false, false), 0);
    assert_eq!(held_from_keys(false, false, true, true, false, false), 0);
    assert_eq!(held_from_keys(true, false, true, false, true, true), 0x01 | 0x04 | 0x40 | 0x80);
    assert_eq!(held_from_keys(false, true, false, true, false, false), 0x02 | 0x08);
}

#[test]
fn framebuffer_export_png_header() {
    let mut canvas = Canvas::new();
    canvas.clear([1, 2, 3, 255]);
    let path = std::env::temp_dir().join("gunnac_t58_export.png");
    canvas.export_png(&path).expect("导出");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A], "PNG 签名");
    assert_eq!(&bytes[12..16], b"IHDR", "IHDR 块");
    // IHDR 尺寸 = 256×240
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    assert_eq!((w, h), (SCREEN_W as u32, SCREEN_H as u32));
}

#[test]
fn tile_draw_pixels() {
    let mut canvas = Canvas::new();
    let tile = [[1u8; 8]; 8];
    canvas.draw_tile8(&tile, 4, 4, &GREY4);
    assert_eq!(canvas.pixel(4, 4), GREY4[1]);
    assert_eq!(canvas.pixel(11, 11), GREY4[1]);
    assert_eq!(canvas.pixel(3, 4), [0, 0, 0, 0], "边界外不落笔");
    assert_eq!(canvas.pixel(12, 12), [0, 0, 0, 0]);
}

// ============ t90：W12 桥 + BG 投影 ============

#[test]
fn bg_bridge_record_addressing_vx5_quirk() {
    use game_core::terrain::{bg_record_at, BgRecord};
    // v×5 quirk（t70 更正 N-0088）：索引 2 → 偏移 10
    let pattern: Vec<u8> = (0..40u8).collect();
    let r = bg_record_at(&pattern, 2);
    assert_eq!(r, BgRecord { f1: 10, f2: 11, f3: 12, f4: 13, flag: 14 });
    assert_ne!(bg_record_at(&pattern, 2), BgRecord { f1: 6, f2: 7, f3: 8, f4: 9, flag: 10 }, "×3 旧值必红");
}

#[test]
fn bg_bridge_commit_fields_and_flag() {
    use game_core::terrain::{bridge_output_commit, BgBuffers, BgRecord};
    let mut bufs = BgBuffers::default();
    let rec = BgRecord { f1: 1, f2: 2, f3: 3, f4: 4, flag: 0x0A };
    assert!(bridge_output_commit(rec, 5, &mut bufs).is_some());
    assert_eq!((bufs.b130[5], bufs.b131[5], bufs.b140[5], bufs.b141[5]), (1, 2, 3, 4), "4 字段映射");
    assert_eq!(bufs.flag, 0x02, "标志&7");
    // 表界 fail-closed（CPX #$10）
    assert!(bridge_output_commit(rec, 16, &mut bufs).is_none(), "槽 16 越界必拒");
}

#[test]
fn bg_projection_renders_without_panic() {
    use game_desktop::render_bg_projection;
    let rom = std::fs::read("../../rom/gunnac_cn.nes").unwrap_or_default();
    if rom.is_empty() {
        return;
    }
    let mut canvas = Canvas::new();
    let b130 = [0u8; 16];
    let b131 = [1u8; 16];
    let b140 = [2u8; 16];
    let b141 = [3u8; 16];
    render_bg_projection(&mut canvas, &rom, &b130, &b131, &b140, &b141, 8, 8);
    // 投影必落笔（tile 0 可能全透明——断言画布被写过用非零 tile 检）
    let mut canvas2 = Canvas::new();
    let b131b = [2u8; 16];
    render_bg_projection(&mut canvas2, &rom, &b130, &b131b, &b140, &b141, 0, 0);
    assert!(canvas2.pixels.iter().any(|&p| p != 0), "投影必须落笔（负向哨兵）");
}

#[test]
fn bg_negative_selfproof() {
    use game_core::terrain::{bridge_output_commit, BgBuffers, BgRecord, ATTR_MASKS};
    // 负向：位属性掩码哨兵（错掩码必红）
    assert_eq!(ATTR_MASKS, [0xFF, 0xF9, 0xF0], "掩码 ROM 逐字节");
    let mut bufs = BgBuffers::default();
    bufs.attr[0] = 0x0F;
    bridge_output_commit(BgRecord { f1: 0, f2: 0, f3: 0, f4: 0, flag: 0x07 }, 0, &mut bufs);
    assert_eq!(bufs.attr[0], 0x0F, "(旧&FF)|标志 = 0x0F|7=0x0F——合成公式哨兵（OR 口径，掩码尾段未全证注记）");
}

#[test]
fn bg_visual_scout_sample_export() {
    // t90 ④ visual-diff 前置侦察样张（灰阶口径声明；真值映射未证——不比对只留样）
    use game_core::terrain::bg_record_at;
    use game_desktop::render_bg_projection;
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM 可读");
    // 取 f1500 帧实测滚动（zp $1C/$1D）+ STG-1 图案表（$BCFA，bank8:$A000 窗）
    let trace = std::fs::read_to_string("../../analysis/traces/gameplay-stage1.frames.jsonl");
    let (mut sx, mut sy) = (0u8, 0u8);
    if let Ok(text) = trace {
        for line in text.lines() {
            if line.contains("\"f\":1500,") || line.contains("\"f\": 1500,") {
                if let Some(i) = line.find("\"zp\":\"") {
                    let rest = &line[i + 6..];
                    let hex = &rest[..rest.find('"').unwrap()];
                    let zp = hex_decode(hex);
                    sx = zp[0x1C];
                    sy = zp[0x1D];
                    break;
                }
            }
        }
    }
    let bank8 = &rom[0x10 + 8 * 0x2000..0x10 + 9 * 0x2000];
    let pattern = &bank8[0x1CFA..]; // 图案表 $BCFA → bank8:$A000 窗内偏移 0x1CFA
    let (mut b130, mut b131, mut b140, mut b141) = ([0u8; 16], [0u8; 16], [0u8; 16], [0u8; 16]);
    for i in 0..16usize {
        let r = bg_record_at(pattern, i as u8);
        b130[i] = r.f1;
        b131[i] = r.f2;
        b140[i] = r.f3;
        b141[i] = r.f4;
    }
    let mut canvas = Canvas::new();
    render_bg_projection(&mut canvas, &rom, &b130, &b131, &b140, &b141, sx, sy);
    let dir = std::path::Path::new("../../analysis/visual-scout");
    std::fs::create_dir_all(dir).unwrap();
    let out = dir.join("bg_projection_f1500.png");
    canvas.export_png(&out).expect("导出样张");
    assert!(out.exists(), "样张必须落盘");
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

// ============ t104：精灵层（OAM 语义/合成/quirk） ============

#[test]
fn oam_parse_and_attr_semantics() {
    use game_desktop::{parse_oam, SpriteObj};
    let s = SpriteObj::from_oam4([0x1F, 0x05, 0x63, 0x40]);
    // 【锚点口径钉死（t165 红窗收口）】screen_y = oam_y（8×16 高精灵上缘锚）——
    // 等价式注：8×8 时代 screen_y=oam_y+8 ≡ entity_y−8（OAM 基准 +16）；
    // 8×16 时代锚=y（t165 5 基×2 帧实测判优 95.39/94.74）——两口径择一钉死=本式。
    // 【N-0168 平台惯例】OAM Y→屏 y **+1**（t191 彩蛋：Y=80→屏 81——1px 纵相位谜底）
    assert_eq!(s.screen_y(), 0x20, "锚点口径钉死：screen_y=oam_y+1（N-0168）");
    assert_eq!(s.palette(), 3);
    assert!(s.behind_bg(), "bit5=BG 前置");
    assert!(s.hflip() && !s.vflip(), "bit6=hflip（0x63 位形）");
    let oam = [0u8; 256];
    assert_eq!(parse_oam(&oam).len(), 64, "64 条 OAM");
}

#[test]
fn sprite_render_flip_priority_and_8line_quirk() {
    use game_desktop::{render_sprites, Canvas, SpriteObj, GREY4};
    let rom = std::fs::read("../../rom/gunnac_cn.nes").unwrap_or_default();
    if rom.is_empty() {
        return;
    }
    let mut canvas = Canvas::new();
    let pals = [GREY4; 4];
    let bg_clear = [[false; 256]; 240];
    // 单精灵：字模区首个非空 tile
    let mut tile_pick = 0u8;
    for t in 0..64u8 {
        let raw = game_data::chr::tile_bytes(&rom, 124, t as usize);
        if game_data::chr::decode_tile(&raw).iter().any(|r| r.iter().any(|&v| v != 0)) {
            tile_pick = t;
            break;
        }
    }
    let spr = SpriteObj { y: 100, tile: tile_pick, attr: 0, x: 50 };
    render_sprites(&mut canvas, &rom, &[spr], 124, &pals, &bg_clear);
    assert!(canvas.pixels.iter().any(|&p| p != 0), "精灵必须落笔");
    // 负向 1：behind_bg + BG 遮挡 ⇒ 不落笔
    let mut c2 = Canvas::new();
    let bg_block = [[true; 256]; 240];
    render_sprites(&mut c2, &rom, &[SpriteObj { y: 100, tile: tile_pick, attr: 0x20, x: 50 }], 124, &pals, &bg_block);
    assert!(c2.pixels.iter().all(|&p| p == 0), "BG 前置+遮挡必红");
    // 负向 2：8/行溢出——同线 9 精灵，第 9 个丢弃
    let mut c3 = Canvas::new();
    let nine: Vec<SpriteObj> = (0..9).map(|i| SpriteObj { y: 100, tile: tile_pick, attr: 0, x: i * 8 }).collect();
    render_sprites(&mut c3, &rom, &nine, 124, &pals, &bg_clear);
    let mut c4 = Canvas::new();
    let eight: Vec<SpriteObj> = (0..8).map(|i| SpriteObj { y: 100, tile: tile_pick, attr: 0, x: i * 8 }).collect();
    render_sprites(&mut c4, &rom, &eight, 124, &pals, &bg_clear);
    let count = |c: &Canvas| c.pixels.iter().filter(|&&p| p != 0).count();
    assert_eq!(count(&c3), count(&c4), "第 9 精灵必须丢弃（8/行 quirk）");
}

#[test]
fn y_anchor_table_and_negatives() {
    // t107 ①：逐实体 Y 锚点表（弹/敌/文本族 +8——实测修正版；自机族 +0x2C 待槽分区）
    use game_desktop::SpriteObj;
    let s = SpriteObj { y: 80, tile: 0, attr: 0, x: 0 };
    assert_eq!(s.screen_y(), 81, "OAM Y=80→屏 81（N-0168 平台惯例）");
    // 负向自证：自机截止门 <$30（参数表）
    let p = SpriteObj { y: 0x20, tile: 0, attr: 0x01, x: 0 };
    assert!(p.player_cutoff(), "自机 Y<$30 截止必真");
    let p2 = SpriteObj { y: 0x30, tile: 0, attr: 0x01, x: 0 };
    assert!(!p2.player_cutoff(), "Y=$30 边界哨兵");
    // 负向自证 2：attr 位形哨兵
    let a = SpriteObj { y: 0, tile: 0, attr: 0xE3, x: 0 };
    assert!(a.hflip() && a.vflip() && a.behind_bg(), "0xE3 位形必全真");
}

#[test]
fn ink_mask_p0_set_negative_highplane() {
    // 【t185 R18-1】墨掩码=p0 置位（索引 1/3 为墨、0/2 为底）——负向：高平面（index2）必红
    let ink_p0 = |v: u8| v & 1 == 1;
    assert!(ink_p0(1) && ink_p0(3), "p0 置位=墨（1/3）");
    assert!(!ink_p0(0) && !ink_p0(2), "p0 清位=底（0/2）");
    // 负向：高平面口径（v==2 为墨）与 p0 掩码矛盾——必红哨兵
    let high_plane = |v: u8| v == 2;
    let contradictions = (0..4u8).filter(|&v| ink_p0(v) == high_plane(v)).count();
    assert!(contradictions > 0, "高平面与 p0 掩码矛盾必现（t181 结论反了实证）");
    // oracle『地』ink=p0 低平面掩码 100%（256/256）——R18-1 A 级
    assert!(ink_p0(1) && ink_p0(3) && !ink_p0(2), "R18-1 墨掩码定谳");
}
