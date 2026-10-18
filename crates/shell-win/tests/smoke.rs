//! shell/win 冒烟自测（platform-shell.md §3 五件）。

/// 输入映射（Z/X/Enter/RShift/方向→Buttons）。
#[test]
fn input_mapping() {
    use shell_win::map_key;
    assert!(map_key("z").map(|b| b.a).unwrap_or(false), "Z→A");
    assert!(map_key("x").map(|b| b.b).unwrap_or(false), "X→B");
    assert!(map_key("Enter").map(|b| b.start).unwrap_or(false), "Enter→START");
    assert!(map_key("ShiftRight").map(|b| b.select).unwrap_or(false), "RShift→SELECT");
    assert!(map_key("ArrowUp").map(|b| b.up).unwrap_or(false), "↑");
    assert!(map_key("ArrowLeft").map(|b| b.left).unwrap_or(false), "←");
    assert!(map_key("q").is_none(), "未映射键=None");
}

/// 存储路径（cfg 差异——目录名 gunnac）。
#[test]
fn storage_path() {
    let p = shell_win::storage_dir();
    assert!(p.to_string_lossy().contains("gunnac"), "存储目录含 gunnac");
}

/// 60Hz 帧间隔（16ms 节流基）。
#[test]
fn frame_interval() {
    assert_eq!(shell_win::FRAME_MS, 16, "60Hz 帧间隔");
}

/// ②帧驱动（60Hz 固定步+追帧上限）。
#[test]
fn frame_driver_tick() {
    let mut d = shell_win::FrameDriver::new();
    let n = d.tick();
    assert!(n <= 4, "追帧上限");
}

/// ②blit 整数缩放（视口+像素）。
#[test]
fn blit_integer_scale() {
    let canvas = vec![0x80u8; 256 * 240 * 4];
    let mut out = vec![0u32; 512 * 480];
    let (ox, oy, w, h) = shell_win::blit_scaled(&canvas, &mut out, 512, 480);
    assert_eq!((w, h), (512, 480), "2 倍整数缩放");
    assert_eq!((ox, oy), (0, 0), "居中偏移");
    assert_eq!(out[0] & 0xFF, 0x80, "像素 blit");
}

/// ②视口（保比例黑边）。
#[test]
fn view_rect_aspect() {
    let (ox, oy, w, h) = shell_win::view_rect(800, 600);
    assert_eq!((w, h), (512, 480), "2 倍（min(800/256,600/240)=2）");
    assert_eq!((ox, oy), (144, 60), "黑边居中");
}

/// ③音频件（ApuWrite 流消费+静音占位显式）。
#[test]
fn audio_sink_stream() {
    let mut a = shell_win::AudioSink::new();
    a.write_apu(0, 0x4000, 0xAD);
    a.write_apu(1, 0x4002, 0xFF);
    assert_eq!(a.writes.len(), 2, "流消费");
    assert_eq!(a.writes[0], (0, 0x4000, 0xAD), "三元组");
    assert!(a.silent, "静音占位显式（合成器未证）");
    let pcm = a.synth_pcm(10);
    assert_eq!(pcm.len(), 10, "PCM 面");
    assert!(pcm.iter().all(|&s| s == 0), "静音期零样本");
}
