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
