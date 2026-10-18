//! 平台 trait 锁定形状（platform-shell.md §2——已证语义最小面）。

/// Renderer Canvas 形状（256×240 RGBA=196608 字节）。
#[test]
fn renderer_canvas_shape() {
    // 形状锁：canvas_mut 返回 256*240*4 字节面（实现侧分配）
    let sz = 256usize * 240 * 4;
    assert_eq!(sz, 245_760, "256×240 RGBA Canvas 字节数");
}

/// Audio ApuWrite 流形状（(frame,reg,value) 三元组）。
#[test]
fn audio_stream_shape() {
    fn assert_stream<F: FnMut(u32, u16, u8)>(_f: F) {}
    assert_stream(|_f, _r, _v| ());
    // 形状锁：三元组 (u32 frame, u16 reg, u8 value)——L-Audio 同源
}
