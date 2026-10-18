//! shell/win（platform-shell.md §3——Windows/Linux 同源跨平台壳）。
//!
//! 五件：窗口（winit+softbuffer）/60Hz 帧驱动/呈现（blit 整数缩放）/
//! 输入映射（Z/X/Enter/RShift/方向→Buttons）/音频（APU 合成模型+跨平台面）。
//! OS 差异 cfg(windows)/cfg(unix)=存储路径/音频后端（差异实现非分叉）。

/// 60Hz 帧间隔（毫秒——确定性 60Hz）。
pub const FRAME_MS: u64 = 16; // 1000/60≈16.67——帧驱动节流基

/// 输入映射（platform-shell.md §3：Z/X/Enter/RShift/方向→Buttons）。
/// 键码→按钮（winit 键盘面——虚拟键码映射表）。
pub fn map_key(key: &str) -> Option<game_platform::Buttons> {
    use game_platform::Buttons;
    let mut b = Buttons::default();
    match key {
        "z" | "Z" => { b.a = true; Some(b) }
        "x" | "X" => { b.b = true; Some(b) }
        "Enter" => { b.start = true; Some(b) }
        "ShiftRight" | "RShift" => { b.select = true; Some(b) }
        "ArrowUp" => { b.up = true; Some(b) }
        "ArrowDown" => { b.down = true; Some(b) }
        "ArrowLeft" => { b.left = true; Some(b) }
        "ArrowRight" => { b.right = true; Some(b) }
        _ => None,
    }
}

/// 存储目录解析（cfg 差异：Windows=%APPDATA%/gunnac/；Linux=~/.local/share/gunnac/）。
pub fn storage_dir() -> std::path::PathBuf {
    #[cfg(windows)]
    {
        std::env::var("APPDATA")
            .map(|d| std::path::PathBuf::from(d).join("gunnac"))
            .unwrap_or_else(|_| std::path::PathBuf::from("gunnac"))
    }
    #[cfg(unix)]
    {
        std::env::var("HOME")
            .map(|d| std::path::PathBuf::from(d).join(".local/share/gunnac"))
            .unwrap_or_else(|_| std::path::PathBuf::from("gunnac"))
    }
    #[cfg(not(any(windows, unix)))]
    {
        std::path::PathBuf::from("gunnac")
    }
}

// ===== ②五件深实（窗口/帧驱动/呈现——platform-shell.md §3）=====

/// 60Hz 帧驱动节流（Instant 节流——固定步）。
pub struct FrameDriver {
    last: std::time::Instant,
    accum: std::time::Duration,
}

impl FrameDriver {
    pub fn new() -> Self {
        FrameDriver { last: std::time::Instant::now(), accum: std::time::Duration::ZERO }
    }
    /// 帧推进（60Hz 固定步——返回应推进的帧数）。
    pub fn tick(&mut self) -> u32 {
        let now = std::time::Instant::now();
        self.accum += now - self.last;
        self.last = now;
        let frame = std::time::Duration::from_millis(FRAME_MS);
        let mut n = 0;
        while self.accum >= frame {
            self.accum -= frame;
            n += 1;
        }
        n.min(4) // 追帧上限（防 spiral of death）
    }
}

/// blit 整数倍缩放（256×240 RGBA→窗口（保比例黑边——game-desktop view_rect 同语义））。
pub fn blit_scaled(
    canvas: &[u8], // 256*240*4
    out: &mut [u32], // 窗口像素（softbuffer ARGB）
    win_w: u32, win_h: u32,
) -> (u32, u32, u32, u32) {
    let (ox, oy, w, h) = view_rect(win_w, win_h);
    for y in 0..h {
        for x in 0..w {
            let sx = (x * 256 / w) as usize;
            let sy = (y * 240 / h) as usize;
            let si = (sy * 256 + sx) * 4;
            let di = ((oy + y) * win_w + ox + x) as usize;
            if si + 3 < canvas.len() && di < out.len() {
                let (r, g, b) = (canvas[si] as u32, canvas[si + 1] as u32, canvas[si + 2] as u32);
                out[di] = 0xFF00_0000 | (r << 16) | (g << 8) | b;
            }
        }
    }
    (ox, oy, w, h)
}

/// 整数倍缩放视口（shell 侧——game-desktop view_rect 同语义）。
pub fn view_rect(win_w: u32, win_h: u32) -> (u32, u32, u32, u32) {
    let (sw, sh) = (256u32, 240u32);
    let scale = std::cmp::max(1, std::cmp::min(win_w / sw, win_h / sh));
    let (w, h) = (sw * scale, sh * scale);
    let ox = win_w.saturating_sub(w) / 2;
    let oy = win_h.saturating_sub(h) / 2;
    (ox, oy, w, h)
}
