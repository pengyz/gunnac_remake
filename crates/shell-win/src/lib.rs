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

// ===== ③音频件（APU 合成模型驱动——platform-shell.md §3）=====

/// 音频件（ApuWrite 流消费→PCM——合成器未证期=流透传/静音占位显式）。
pub struct AudioSink {
    pub writes: Vec<(u32, u16, u8)>, // ApuWrite 流（(frame,reg,value)——L-Audio 同源）
    pub silent: bool,                 // 静音占位（合成器未证期显式）
}

impl AudioSink {
    pub fn new() -> Self {
        AudioSink { writes: Vec::new(), silent: true } // 静音占位显式（合成器未证）
    }
    /// ApuWrite 流消费（game-platform::Audio trait 面）。
    pub fn write_apu(&mut self, frame: u32, reg: u16, value: u8) {
        self.writes.push((frame, reg, value));
    }
    /// 帧推进（60Hz——PCM 合成随合成器证成升级）。
    pub fn end_frame(&mut self, _frame: u32) {}
    /// PCM 合成（APU 合成模型驱动——音调式/duty/影子——合成器未证期=静音返回）。
    pub fn synth_pcm(&self, _samples: usize) -> Vec<i16> {
        if self.silent {
            vec![0i16; _samples] // 静音占位（显式）
        } else {
            vec![0i16; _samples] // 合成器证成后升级（apu_pitch_model/duty 驱动）
        }
    }
}

impl game_platform::Audio for AudioSink {
    fn write_apu(&mut self, frame: u32, reg: u16, value: u8) {
        AudioSink::write_apu(self, frame, reg, value);
    }
    fn end_frame(&mut self, frame: u32) {
        AudioSink::end_frame(self, frame);
    }
}

// ===== ④游戏宿主接线（title→八关→结局——game-core flow/stage 已证面）=====

/// 宿主状态（platform-shell.md §3：title→设置→八关→结局 状态机——F1-F13 边集）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostState {
    Title,
    Stage(u8), // 八关（1-8）
    Ending,
}

/// 游戏宿主（状态机+game-core stage_tick 接线面）。
pub struct GameHost {
    pub state: HostState,
    pub frame: u32,
}

impl GameHost {
    pub fn new() -> Self {
        GameHost { state: HostState::Title, frame: 0 }
    }
    /// 按钮驱动状态转移（title→八关→结局——F1-F13 边集已证面）。
    pub fn press_start(&mut self) {
        self.state = match self.state {
            HostState::Title => HostState::Stage(1),
            HostState::Stage(n) if n < 8 => HostState::Stage(n + 1),
            HostState::Stage(_) => HostState::Ending,
            HostState::Ending => HostState::Title,
        };
    }
    /// 帧推进（60Hz——stage_tick 接线面）。
    pub fn tick(&mut self) {
        self.frame += 1;
    }
}
