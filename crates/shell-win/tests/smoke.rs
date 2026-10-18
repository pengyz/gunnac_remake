//! shell-win v1 契约测试：PC 平台 trait 实现 + 呈现语义 + 确定性自检。
//!
//! 覆盖（task-1 + task-18 findings 收口）：
//! - 输入映射（platform-shell.md §4 键位表）；
//! - 存储路径（Windows `%APPDATA%` / Linux `~/.local/share`）；
//! - 整数倍缩放 + 黑边（`view_rect`/`blit_scaled`）；
//! - `AssetSource`/`Storage`/`Audio`/`Lifecycle`/`Input` 的 PC 实现真实行为与 fail-closed
//!   （含 Windows drive-relative 前缀拒绝 B1、exe 目录资产根解析 S1）；
//! - CHR 背景占位合成（灰阶投影；非原版调色板）；
//! - `--selftest-frames` 确定性（两次运行报告逐字段一致）+ 异 cwd 资产定位。

use game_platform::{
    AppEvent, AssetSource, Audio, Buttons, Input, Lifecycle, Renderer, Storage, CANVAS_LEN, CANVAS_W,
};
use shell_win::{
    asset_root, blit_scaled, compose_canvas, map_key, run_selftest, storage_dir, view_rect,
    ChrBackground, FileStorage, FrameRenderer, FsAssets, KeyboardInput, LifecycleQueue, PcAudio,
    CLEAR_RGBA, GREY4, WINDOW_TITLE,
};

/// 测试用 CHR ROM 最小长度（`CHR_FILE_BASE + 128*CHR_BANK_SIZE` = 0x40010；
/// 与 `game_data::chr` t2 A 级布局同源，测试侧显式常量避免依赖解析器内部值）。
const CHR_ROM_MIN: usize = 0x20010 + 128 * 0x400;

/// 构造全灰（index=1）的 CHR 测试 ROM：每个 tile 的 p0 全 1、p1 全 0 ⇒ 像素索引 1。
fn fixture_chr_rom() -> Vec<u8> {
    let mut rom = vec![0u8; CHR_ROM_MIN];
    let bank = 124usize; // FONT_BANK_START（game_data::chr::FONT_BANK_START）
    let base = 0x20010 + bank * 0x400;
    for tile in 0..64usize {
        let off = base + tile * 16;
        rom[off..off + 8].fill(0xFF);
        rom[off + 8..off + 16].fill(0x00);
    }
    rom
}

fn pixel(canvas: &[u8], x: usize, y: usize) -> [u8; 4] {
    let o = (y * CANVAS_W + x) * 4;
    [canvas[o], canvas[o + 1], canvas[o + 2], canvas[o + 3]]
}

// ===================== 输入映射 =====================

#[test]
fn input_mapping_matches_platform_shell_section4() {
    assert!(map_key("z").map(|b| b.a).unwrap_or(false), "Z→A");
    assert!(map_key("x").map(|b| b.b).unwrap_or(false), "X→B");
    assert!(map_key("Enter").map(|b| b.start).unwrap_or(false), "Enter→START");
    assert!(map_key("ShiftRight").map(|b| b.select).unwrap_or(false), "RShift→SELECT");
    assert!(map_key("ArrowUp").map(|b| b.up).unwrap_or(false), "↑");
    assert!(map_key("ArrowDown").map(|b| b.down).unwrap_or(false), "↓");
    assert!(map_key("ArrowLeft").map(|b| b.left).unwrap_or(false), "←");
    assert!(map_key("ArrowRight").map(|b| b.right).unwrap_or(false), "→");
    assert!(map_key("q").is_none(), "未映射键=None");
}

#[test]
fn keyboard_input_tracks_press_and_release() {
    let mut input = KeyboardInput::new();
    assert_eq!(input.buttons(), Buttons::default());
    assert!(input.apply_key("z", true));
    assert!(input.buttons().a, "按下置位");
    assert!(input.apply_key("ArrowRight", true));
    assert!(input.buttons().right);
    assert!(input.apply_key("z", false));
    assert!(!input.buttons().a, "抬起清除");
    assert!(!input.apply_key("q", true), "未映射键返回 false");
    assert_eq!(input.buttons().right, true, "未映射键不改状态");
}

#[test]
fn window_title_is_locked() {
    assert_eq!(WINDOW_TITLE, "gunnac-remake (Stage A)");
}

// ===================== 存储路径 =====================

#[test]
fn storage_path_contains_gunnac() {
    let p = storage_dir();
    assert!(p.to_string_lossy().contains("gunnac"), "存储目录含 gunnac");
}

// ===================== 呈现：整数倍缩放 + 黑边 =====================

#[test]
fn view_rect_integer_scaling_and_letterbox() {
    assert_eq!(view_rect(512, 480), (0, 0, 512, 480), "2× 整窗");
    assert_eq!(view_rect(800, 600), (144, 60, 512, 480), "2× + 黑边居中");
    assert_eq!(view_rect(1024, 768), (128, 24, 768, 720), "3×（宽/高余量黑边居中）");
    // 窗口小于画布：scale≥1（不缩小变形；blit 侧裁切）
    let (ox, oy, w, h) = view_rect(200, 200);
    assert_eq!((ox, oy, w, h), (0, 0, 256, 240), "小窗保持 1×（裁切）");
}

#[test]
fn blit_scaled_fills_letterbox_black_and_scales_pixels() {
    let canvas = vec![0x80u8; CANVAS_LEN];
    let mut out = vec![0xFFFF_FFFFu32; 800 * 600];
    let (ox, oy, w, h) = blit_scaled(&canvas, &mut out, 800, 600);
    assert_eq!((ox, oy, w, h), (144, 60, 512, 480));
    assert_eq!(out[0], 0, "视口外=黑边");
    assert_eq!(out[600 * 800 - 1], 0, "右下角视口外=黑边");
    assert_eq!(out[(60 * 800 + 144) as usize], 0x808080, "视口内=缩放像素");
    assert_eq!(out[(60 * 800 + 144 + 511) as usize], 0x808080, "2× 横向铺满视口");
}

#[test]
fn blit_scaled_clips_when_window_smaller_than_canvas() {
    let canvas = vec![0x20u8; CANVAS_LEN];
    let mut out = vec![0u32; 100 * 100];
    let (_, _, w, h) = blit_scaled(&canvas, &mut out, 100, 100);
    assert_eq!((w, h), (256, 240), "1× 视口大于窗口");
    assert!(out.iter().all(|&p| p == 0x202020), "不越界、不换行写坏缓冲");
}

// ===================== CHR 背景占位合成 =====================

#[test]
fn chr_background_fails_closed_on_short_rom() {
    let mut renderer = FrameRenderer::new();
    assert!(!renderer.set_chr_rom(&[0u8; 16]), "ROM 尺寸不足⇒fail-closed");
    assert_eq!(renderer.background(), &ChrBackground::black(), "背景保持黑");
    renderer.present();
    assert_eq!(pixel(renderer.composed_canvas(), 0, 0), CLEAR_RGBA);
}

#[test]
fn chr_background_renders_decoded_tiles_and_overlays_game_canvas() {
    let rom = fixture_chr_rom();
    let mut renderer = FrameRenderer::new();
    assert!(renderer.set_chr_rom(&rom), "尺寸足够⇒背景生成");
    renderer.present();
    assert_eq!(
        pixel(renderer.composed_canvas(), 0, 0),
        GREY4[1],
        "CHR 背景：tile 像素索引 1 ⇒ 灰阶占位色"
    );

    // game-shell 画布中的非清屏像素覆盖背景；清屏像素保留背景。
    let o = (10 * CANVAS_W + 20) * 4;
    renderer.canvas_mut()[o..o + 4].copy_from_slice(&[0xFF, 0x00, 0x00, 0xFF]);
    renderer.present();
    let composed = renderer.composed_canvas();
    assert_eq!(pixel(composed, 20, 10), [0xFF, 0x00, 0x00, 0xFF], "画布内容覆盖背景");
    assert_eq!(pixel(composed, 0, 0), GREY4[1], "其余像素保留 CHR 背景");
    assert_eq!(renderer.presents(), 2, "present 计数");
}

#[test]
fn compose_canvas_treats_clear_color_as_transparent() {
    let mut background = vec![0u8; CANVAS_LEN];
    for px in background.chunks_exact_mut(4) {
        px.copy_from_slice(&GREY4[2]);
    }
    let mut game = vec![0u8; CANVAS_LEN];
    for px in game.chunks_exact_mut(4) {
        px.copy_from_slice(&CLEAR_RGBA);
    }
    let out = compose_canvas(&background, &game);
    assert_eq!(pixel(&out, 0, 0), GREY4[2], "清屏色视为无内容");

    let o = (3 * CANVAS_W + 4) * 4;
    game[o..o + 4].copy_from_slice(&[1, 2, 3, 255]);
    let out = compose_canvas(&background, &game);
    assert_eq!(pixel(&out, 4, 3), [1, 2, 3, 255]);
}

#[test]
fn frame_renderer_size_resize_contract() {
    let mut renderer = FrameRenderer::new();
    assert_eq!(renderer.size(), (256, 240), "头less 输出面=逻辑画布 1×");
    renderer.resize(800, 600);
    assert_eq!(renderer.size(), (800, 600), "resize 更新输出面");
    assert_eq!(renderer.canvas_mut().len(), CANVAS_LEN, "逻辑画布尺寸锁定");
}

// ===================== Audio（ApuWrite 流消费；静音占位显式） =====================

#[test]
fn audio_consumes_apu_write_stream_and_is_explicitly_silent() {
    let mut audio = PcAudio::new();
    assert!(audio.silent(), "静音占位显式");
    Audio::write_apu(&mut audio, 0, 0x4000, 0xAD);
    Audio::write_apu(&mut audio, 1, 0x4002, 0xFF);
    Audio::end_frame(&mut audio, 0);
    assert_eq!(audio.writes_total(), 2, "写流真实消费");
    assert_eq!(
        audio.recent().copied().collect::<Vec<_>>(),
        vec![(0, 0x4000, 0xAD), (1, 0x4002, 0xFF)],
        "写序保真"
    );
    assert_eq!(audio.frames_total(), 1);
    assert_eq!(audio.last_frame(), Some(0));
    let pcm = audio.synth_pcm(16);
    assert_eq!(pcm.len(), 16);
    assert!(pcm.iter().all(|&s| s == 0), "合成器未证期 PCM=全 0（显式占位）");
}

// ===================== Storage（字节块存取） =====================

#[test]
fn file_storage_roundtrip_and_fail_closed() {
    let dir = std::env::temp_dir().join(format!(
        "gunnac-shellwin-storage-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let mut storage = FileStorage::new(&dir);
    assert_eq!(storage.load(0), None, "空槽⇒None");
    assert!(storage.save(0, &[1, 2, 3, 4]), "保存成功（目录自动创建）");
    assert_eq!(storage.load(0), Some(vec![1, 2, 3, 4]), "往返一致");
    assert_eq!(storage.load(1), None, "未写槽⇒None");
    assert!(storage.slot_path(0).to_string_lossy().contains("slot0.sav"));
    let _ = std::fs::remove_dir_all(&dir);
}

// ===================== AssetSource（std::fs 字节读取） =====================

#[test]
fn fs_assets_reads_relative_files_and_fails_closed() {
    // 显式根=仓库根（编译期绝对路径；测试不依赖进程 cwd —— S1 口径）。
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("仓库根")
        .to_path_buf();
    let mut assets = FsAssets::new(&repo);
    let manifest = assets
        .read_all("crates/shell-win/Cargo.toml")
        .expect("仓库相对文件可读");
    assert!(manifest.starts_with(b"[package]"), "整块字节读取");
    assert_eq!(assets.read_all("rom/missing-asset.nes"), None, "缺失⇒None");
    assert_eq!(assets.read_all("../AGENTS.md"), None, "`..` 越界⇒None");
    assert_eq!(assets.read_all("C:\\AGENTS.md"), None, "绝对路径⇒None");
    // B1：Windows drive-relative 前缀（"C:foo" 不是绝对路径但会逃逸资产根）同样 fail-closed。
    assert_eq!(assets.read_all("C:foo"), None, "drive-relative 前缀⇒None");
    assert_eq!(assets.read_all("D:foo\\x"), None, "盘符前缀+反斜杠⇒None");
    let rom = assets.read_all("rom/gunnac_cn.nes");
    assert!(
        rom.map(|r| r.len() >= CHR_ROM_MIN).unwrap_or(false),
        "ROM 资产可读且覆盖 CHR 域"
    );
}

#[test]
fn fs_assets_detect_resolves_dev_asset_root() {
    // 开发态：exe 目录无 ROM ⇒ debug 回退 CARGO_MANIFEST_DIR 仓库根（S1）。
    let Some(root) = asset_root() else {
        // release 且 exe 旁无 ROM ⇒ 按设计 fail-closed（debug 构建必须能解析）。
        assert!(!cfg!(debug_assertions), "debug 构建必须能解析开发态资产根");
        return;
    };
    assert!(root.join("rom/gunnac_cn.nes").is_file(), "资产根含 ROM");
    let mut assets = FsAssets::detect();
    assert_eq!(assets.root(), Some(root.as_path()));
    assert!(
        assets
            .read_all("rom/gunnac_cn.nes")
            .map(|r| r.len() >= CHR_ROM_MIN)
            .unwrap_or(false),
        "自动检测根可读 ROM"
    );
}

#[test]
fn selftest_locates_assets_from_foreign_cwd() {
    // S1：异 cwd 下仍经 exe 目录/开发态回退定位资产根（不依赖进程 cwd）。
    let dir = std::env::temp_dir().join(format!(
        "gunnac-shellwin-foreign-cwd-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("临时 cwd");
    let exe = env!("CARGO_BIN_EXE_shell-win");
    let out = std::process::Command::new(exe)
        .args(["--selftest-frames", "5"])
        .current_dir(&dir)
        .output()
        .expect("spawn shell-win --selftest-frames 5");
    let _ = std::fs::remove_dir_all(&dir);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "异 cwd 自检退出码 0；stderr={stderr}");
    assert!(stdout.contains("selftest: OK"), "自检完成：{stdout}");
    assert!(
        stdout.contains("selftest: rom_len=Some(262160)"),
        "异 cwd 仍定位 ROM（exe/资产根，不依赖 cwd）：{stdout}"
    );
    assert!(
        stderr.contains("shell-win: 资产根="),
        "启动始终打印实际资产根：stderr={stderr}"
    );
}

// ===================== Lifecycle（quit/resume 最小） =====================

#[test]
fn lifecycle_queue_is_fifo_and_nonblocking() {
    let mut lifecycle = LifecycleQueue::new();
    assert_eq!(lifecycle.poll_event(), None, "空队列非阻塞⇒None");
    lifecycle.push(AppEvent::Suspend);
    lifecycle.push(AppEvent::Resume);
    lifecycle.push(AppEvent::Quit);
    assert_eq!(lifecycle.len(), 3);
    assert_eq!(lifecycle.poll_event(), Some(AppEvent::Suspend));
    assert_eq!(lifecycle.poll_event(), Some(AppEvent::Resume));
    assert_eq!(lifecycle.poll_event(), Some(AppEvent::Quit));
    assert_eq!(lifecycle.poll_event(), None);
    assert!(lifecycle.is_empty());
}

// ===================== 确定性自检（--selftest-frames） =====================

#[test]
fn selftest_is_deterministic_and_drives_host() {
    let a = run_selftest(90);
    let b = run_selftest(90);
    assert_eq!(a, b, "连跑两次：报告逐字段一致（确定性）");
    assert_eq!(a.frames, 90);
    assert_eq!(a.advanced, 90, "固定步：每采样 1 逻辑帧");
    assert_eq!(a.presents, 90, "每逻辑帧一次 present");
    assert_eq!(a.frame, 90, "虚拟时钟=推进帧数");
    assert_eq!(format!("{:?}", a.mode), "Combat", "已证边集推进到 Combat");
    assert_eq!(a.stage, 1, "关号=1");
    assert_eq!(a.audio_writes, 2, "半程注入的 ApuWrite 流被真实消费");
    assert_eq!(a.audio_frames, 90, "每逻辑帧一次 end_frame");
    assert!(a.host_signature != 0 && a.canvas_signature != 0);
    assert!(
        a.rom_len.map(|n| n >= CHR_ROM_MIN).unwrap_or(false),
        "ROM 经 AssetSource 装载（覆盖 CHR 域）"
    );
}

#[test]
fn selftest_zero_frames_is_well_defined() {
    let a = run_selftest(0);
    let b = run_selftest(0);
    assert_eq!(a, b);
    assert_eq!((a.frames, a.advanced, a.presents, a.frame), (0, 0, 0, 0));
    assert_eq!(format!("{:?}", a.mode), "Boot", "0 帧不推进");
    assert_eq!(a.audio_writes, 0);
}
