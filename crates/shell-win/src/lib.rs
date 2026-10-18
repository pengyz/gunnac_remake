//! shell-win —— PC（Windows/Linux）平台实现（platform-shell.md §4 首实现；task-1）。
//!
//! 分层合同（[platform-shell.md](../../docs/spec/platform-shell.md) §1）：
//! `game-core`（纯逻辑）← `game-platform` trait 缝 ← `game-shell`（平台无关宿主）← **本 crate**
//! （PC 设备/窗口/输入/音频/资产/存储/生命周期实现）。
//!
//! 本 crate 是**平台侧**：winit/softbuffer/`std::fs`/`std::time` 只允许出现在这里
//! （AGENTS.md §2 红线；`game-core`/`game-platform`/`game-shell` 零平台依赖由各自契约测试把关闭）。
//!
//! v1 内容（task-1）：
//! - 实现 `game_platform` 的 PC 版 trait：`Renderer`（软缓冲合成/整数缩放/黑边）、`Audio`
//!   （真实消费 game-shell 的 `ApuWrite` 流；PCM=静音占位显式）、`Storage`（`%APPDATA%/gunnac`；
//!   Linux `~/.local/share/gunnac`）、`AssetSource`（`std::fs`）、`Input`（winit 键码→`Buttons`）、
//!   `Lifecycle`（quit/resume 最小）；
//! - 主循环（`src/main.rs`）只经 `game_shell::Host` 驱动：输入采样→`Host::advance`（60Hz 固定步）
//!   →呈现；宿主返回的宿主状态/画布/音频写流全部真实消费；
//! - `--selftest-frames N` 无窗口确定性自检（见 [`run_selftest`]）；
//! - 可见性骨架：CHR 背景（平台侧占位）+ 玩家 8×8（game-shell 已证坐标面）；未证域显式不画。
//!
//! **未证不发明**（AGENTS.md §3）：CHR 背景的调色板=灰阶占位（C 级未证，非原版调色板）；
//! 敌精灵/子弹/HUD/文本/结局卡不画（game-shell v1 未暴露/未证）；音效/音乐合成器未证 ⇒ 静音
//! 占位显式。`TODO(shell-v1)` 逐条登记。

#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::path::{Component, Path, PathBuf};

use game_core::flow::ModeEvent;
use game_platform::{
    AppEvent, AssetSource, Audio, Buttons, Input, Lifecycle, Renderer, Storage, CANVAS_H, CANVAS_LEN,
    CANVAS_W,
};
use game_shell::{Host, FRAME_NS, ROM_ASSET_PATH};

use winit::keyboard::KeyCode;

/// 窗口标题（platform-shell.md §4 已锁定字符串）。
pub const WINDOW_TITLE: &str = "gunnac-remake (Stage A)";

/// 60Hz 帧间隔毫秒（**历史占位常量**：帧驱动真值=game-shell `FRAME_NS`；
/// 保留仅为呈现/文档参考，主循环不据此节流）。
pub const FRAME_MS: u64 = 16;

/// game-shell 画布清屏色（`compose_frame` 清屏 RGBA：不透明黑）。
/// 平台侧合成占位以它判"无内容像素"（见 [`compose_canvas`]）。
pub const CLEAR_RGBA: [u8; 4] = [0, 0, 0, 0xFF];

/// CHR 背景灰阶调色板（**投影口径占位**：复制自 `game-desktop::GREY4`，语义相同）。
///
/// **非原版调色板**（调色板映射未证=C 级）——L4 渲染战役锁定真表后替换；
/// 本表只用于可见性骨架，不得据此写游戏/渲染结论。
pub const GREY4: [[u8; 4]; 4] = [
    [0, 0, 0, 255],
    [85, 85, 85, 255],
    [170, 170, 170, 255],
    [255, 255, 255, 255],
];

// ===================== 输入映射（platform-shell.md §4：Z/X/Enter/RShift/方向） =====================

/// 键盘规范名→`Buttons`（单键置位；platform-shell.md §4 锁定的平台输入策略表）。
///
/// 对向同持（L+R/U+D）不在此消解——由 `game_shell::Host` 的 `resolve_directions` 统一消解
/// （未证组合在进入 game-core 前 fail-closed）。
pub fn map_key(key: &str) -> Option<Buttons> {
    let mut b = Buttons::default();
    match key {
        "z" | "Z" => b.a = true,
        "x" | "X" => b.b = true,
        "Enter" => b.start = true,
        "ShiftRight" | "RShift" => b.select = true,
        "ArrowUp" => b.up = true,
        "ArrowDown" => b.down = true,
        "ArrowLeft" => b.left = true,
        "ArrowRight" => b.right = true,
        _ => return None,
    }
    Some(b)
}

/// winit 物理键码→键盘规范名（与 [`map_key`] 表一一对应；不可映射键⇒`None`）。
pub fn key_name(code: KeyCode) -> Option<&'static str> {
    Some(match code {
        KeyCode::KeyZ => "z",
        KeyCode::KeyX => "x",
        KeyCode::Enter => "Enter",
        KeyCode::ShiftRight => "ShiftRight",
        KeyCode::ArrowUp => "ArrowUp",
        KeyCode::ArrowDown => "ArrowDown",
        KeyCode::ArrowLeft => "ArrowLeft",
        KeyCode::ArrowRight => "ArrowRight",
        _ => return None,
    })
}

/// 单键置位（`one` 为 [`map_key`] 产出的单键位集）。
fn set_button(dst: &mut Buttons, one: Buttons, pressed: bool) {
    if one.up {
        dst.up = pressed;
    }
    if one.down {
        dst.down = pressed;
    }
    if one.left {
        dst.left = pressed;
    }
    if one.right {
        dst.right = pressed;
    }
    if one.a {
        dst.a = pressed;
    }
    if one.b {
        dst.b = pressed;
    }
    if one.start {
        dst.start = pressed;
    }
    if one.select {
        dst.select = pressed;
    }
}

/// 键盘输入源（`game_platform::Input` PC 实现）：持有按键状态，宿主每逻辑帧采样。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyboardInput {
    buttons: Buttons,
}

impl KeyboardInput {
    /// 无按键。
    pub const fn new() -> Self {
        Self { buttons: Buttons { up: false, down: false, left: false, right: false, a: false, b: false, start: false, select: false } }
    }

    /// 规范键名按下/抬起（未映射键⇒`false`，不改变状态）。
    pub fn apply_key(&mut self, key: &str, pressed: bool) -> bool {
        match map_key(key) {
            Some(one) => {
                set_button(&mut self.buttons, one, pressed);
                true
            }
            None => false,
        }
    }

    /// winit 物理键码按下/抬起（未映射键⇒`false`）。
    pub fn apply_code(&mut self, code: KeyCode, pressed: bool) -> bool {
        match key_name(code) {
            Some(name) => self.apply_key(name, pressed),
            None => false,
        }
    }

    /// 直接设置按钮位（确定性自检/契约测试口；**不是**游戏输入策略）。
    pub fn set_buttons(&mut self, buttons: Buttons) {
        self.buttons = buttons;
    }

    /// 当前按钮位。
    pub const fn snapshot(&self) -> Buttons {
        self.buttons
    }
}

impl Input for KeyboardInput {
    fn buttons(&self) -> Buttons {
        self.buttons
    }
}

// ===================== 路径策略（存储/资产） =====================

/// 存储目录（platform-shell.md §4/§5：Windows=`%APPDATA%/gunnac/`；Linux=`~/.local/share/gunnac/`）。
pub fn storage_dir() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("APPDATA")
            .map(|d| PathBuf::from(d).join("gunnac"))
            .unwrap_or_else(|_| PathBuf::from("gunnac"))
    }
    #[cfg(unix)]
    {
        std::env::var("HOME")
            .map(|d| PathBuf::from(d).join(".local/share/gunnac"))
            .unwrap_or_else(|_| PathBuf::from("gunnac"))
    }
    #[cfg(not(any(windows, unix)))]
    {
        PathBuf::from("gunnac")
    }
}

/// 资产根目录：优先"当前工作目录含 `rom/gunnac_cn.nes`"（开发态 `cargo run` 从仓库根跑），
/// 否则回退 `CARGO_MANIFEST_DIR` 的仓库根（`crates/shell-win` 上两级）。
///
/// 资产只经 `game_platform::AssetSource` 读取；本函数只决定 PC 文件根路径（平台侧语义）。
pub fn repo_root() -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join(ROM_ASSET_PATH).is_file() {
            return cwd;
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.parent().and_then(Path::parent);
    root.map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."))
}

/// `AssetSource` PC 实现：根目录 + 相对路径 ⇒ `std::fs` 读整块字节。
///
/// fail-closed：绝对路径/含 `..` 的路径拒绝（`None`）；文件缺失⇒`None`，**不**返回默认内容
/// （AGENTS.md §3 未证不发明；解析/校验属 `game-data` 域）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsAssets {
    root: PathBuf,
}

impl FsAssets {
    /// 指定资产根目录。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// 默认资产根（[`repo_root`]）。
    pub fn repo() -> Self {
        Self::new(repo_root())
    }

    /// 资产根目录（只读）。
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// 相对路径安全检查（拒绝绝对路径与 `..` 越界）。
fn safe_relative(path: &str) -> bool {
    let p = Path::new(path);
    if p.is_absolute() {
        return false;
    }
    !p.components()
        .any(|c| matches!(c, Component::ParentDir | Component::RootDir))
}

impl AssetSource for FsAssets {
    fn read_all(&mut self, path: &str) -> Option<Vec<u8>> {
        if !safe_relative(path) {
            return None;
        }
        std::fs::read(self.root.join(path)).ok()
    }
}

/// `Storage` PC 实现：字节块存取到 [`storage_dir`]（`slot{n}.sav`）。
///
/// **占位语义**：原版存档语义（电池存档/密码）未证 ⇒ 本实现只是宿主快照字节存取，
/// 不自动保存、不解码游戏存档（拒绝伪造原版语义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStorage {
    dir: PathBuf,
}

impl FileStorage {
    /// 指定存储目录（不存在时 `save` 递归创建）。
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// 默认存储目录（[`storage_dir`]）。
    pub fn default_pc() -> Self {
        Self::new(storage_dir())
    }

    /// 槽位文件路径（`slot{n}.sav`）。
    pub fn slot_path(&self, slot: u8) -> PathBuf {
        self.dir.join(format!("slot{slot}.sav"))
    }

    /// 存储目录（只读）。
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

impl Storage for FileStorage {
    fn load(&mut self, slot: u8) -> Option<Vec<u8>> {
        std::fs::read(self.slot_path(slot)).ok()
    }

    fn save(&mut self, slot: u8, data: &[u8]) -> bool {
        if std::fs::create_dir_all(&self.dir).is_err() {
            return false;
        }
        std::fs::write(self.slot_path(slot), data).is_ok()
    }
}

// ===================== 音频（ApuWrite 流消费；PCM 静音占位显式） =====================

/// `Audio` PC 实现：真实消费 game-shell 转发来的 `ApuWrite` 流（写序保真登记），
/// PCM 输出=**静音占位显式**（APU 合成器 AUD-2/3 未证；cpal 设备接入留后续任务卡）。
///
/// 写流保留最近 [`PcAudio::RECENT_LIMIT`] 条用于排障/契约测试；全部事件计数与写序 hash
/// 由 game-shell `AudioPipe` 进宿主签名（本层只做设备侧消费，不重复造语义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcAudio {
    recent: VecDeque<(u32, u16, u8)>,
    writes_total: u64,
    frames_total: u64,
    last_frame: Option<u32>,
    silent: bool,
}

impl PcAudio {
    /// 最近写流窗口上限（防长跑无界增长；计数不丢）。
    pub const RECENT_LIMIT: usize = 256;

    /// 静音占位（合成器未证期=true；显式可查询，不是"忘了实现"）。
    pub const fn new() -> Self {
        Self {
            recent: VecDeque::new(),
            writes_total: 0,
            frames_total: 0,
            last_frame: None,
            silent: true,
        }
    }

    /// 已消费的 APU 写事件总数。
    pub const fn writes_total(&self) -> u64 {
        self.writes_total
    }

    /// 已消费的 `end_frame` 次数（=逻辑帧数）。
    pub const fn frames_total(&self) -> u64 {
        self.frames_total
    }

    /// 最近一次 `end_frame` 的帧号。
    pub const fn last_frame(&self) -> Option<u32> {
        self.last_frame
    }

    /// 最近写流窗口（保序；最多 [`PcAudio::RECENT_LIMIT`] 条）。
    pub fn recent(&self) -> impl Iterator<Item = &(u32, u16, u8)> {
        self.recent.iter()
    }

    /// 静音占位是否生效（合成器未证期恒 true——PCM 输出全 0）。
    pub const fn silent(&self) -> bool {
        self.silent
    }

    /// PCM 合成（**静音占位显式**：合成器未证期返回全 0 样本）。
    ///
    /// `TODO(shell-v1)`：APU 写流→PCM 合成属已证音频战役（AUD-2/3），完成后替换本实现。
    pub fn synth_pcm(&self, samples: usize) -> Vec<i16> {
        vec![0i16; samples]
    }
}

impl Default for PcAudio {
    fn default() -> Self {
        Self::new()
    }
}

impl Audio for PcAudio {
    fn write_apu(&mut self, frame: u32, reg: u16, value: u8) {
        self.writes_total += 1;
        if self.recent.len() == Self::RECENT_LIMIT {
            self.recent.pop_front();
        }
        self.recent.push_back((frame, reg, value));
    }

    fn end_frame(&mut self, frame: u32) {
        self.frames_total += 1;
        self.last_frame = Some(frame);
    }
}

// ===================== 生命周期（quit/resume 最小） =====================

/// `Lifecycle` PC 实现：平台事件队列（非阻塞 `poll_event`）。
///
/// 桌面映射（最小）：关窗→`Quit`；焦点丢失/获得→`Suspend`/`Resume`（平台策略占位，
/// 不承载游戏行为；Android 映射见 platform-shell.md §5/§6）。
#[derive(Debug, Default)]
pub struct LifecycleQueue {
    events: VecDeque<AppEvent>,
}

impl LifecycleQueue {
    /// 空队列。
    pub const fn new() -> Self {
        Self { events: VecDeque::new() }
    }

    /// 压入一个平台事件（平台事件处理处调用）。
    pub fn push(&mut self, event: AppEvent) {
        self.events.push_back(event);
    }

    /// 待处理事件数。
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// 队列是否为空。
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Lifecycle for LifecycleQueue {
    fn poll_event(&mut self) -> Option<AppEvent> {
        self.events.pop_front()
    }
}

// ===================== 呈现：整数缩放 + 黑边 + CHR 背景占位合成 =====================

/// 整数倍缩放视口（Stage A 口径：禁缩放变形，黑边保比例）。
///
/// 返回 `(ox, oy, w, h)`：窗口内 256×240 的整数倍视口。窗口小于画布时 scale=1（裁切，
/// 不缩小变形）。与 `game-desktop::view_rect` 同语义（复制而非反向依赖）。
pub fn view_rect(win_w: u32, win_h: u32) -> (u32, u32, u32, u32) {
    let sw = CANVAS_W as u32;
    let sh = CANVAS_H as u32;
    let scale = std::cmp::max(1, std::cmp::min(win_w / sw, win_h / sh));
    let (w, h) = (sw * scale, sh * scale);
    let ox = win_w.saturating_sub(w) / 2;
    let oy = win_h.saturating_sub(h) / 2;
    (ox, oy, w, h)
}

/// 按整数倍缩放把 256×240 RGBA 画布合成到窗口缓冲（`0x00RRGGBB`，softbuffer 格式）；
/// **先整屏清 0（黑边）**再逐像素写视口。
///
/// 返回视口 `(ox, oy, w, h)`；窗口小于画布时按窗口裁切（不越界写）。
pub fn blit_scaled(canvas: &[u8], out: &mut [u32], win_w: u32, win_h: u32) -> (u32, u32, u32, u32) {
    let (ox, oy, w, h) = view_rect(win_w, win_h);
    out.fill(0);
    if win_w == 0 || win_h == 0 {
        return (ox, oy, w, h);
    }
    let scale = std::cmp::max(1, std::cmp::min(win_w / CANVAS_W as u32, win_h / CANVAS_H as u32));
    for y in 0..h {
        for x in 0..w {
            if ox + x >= win_w || oy + y >= win_h {
                continue; // 窗口小于画布：裁切（不越界/不换行）
            }
            let sx = (x / scale) as usize;
            let sy = (y / scale) as usize;
            let si = (sy * CANVAS_W + sx) * 4;
            let di = ((oy + y) * win_w + ox + x) as usize;
            if si + 3 < canvas.len() && di < out.len() {
                let (r, g, b) = (canvas[si] as u32, canvas[si + 1] as u32, canvas[si + 2] as u32);
                out[di] = (r << 16) | (g << 8) | b;
            }
        }
    }
    (ox, oy, w, h)
}

/// 8×8 tile 绘制（像素坐标；越界裁剪不回绕）。
fn draw_tile8(dst: &mut [u8], tile: &[[u8; 8]; 8], x: i32, y: i32, pal: &[[u8; 4]; 4]) {
    for (py, row) in tile.iter().enumerate() {
        for (px, &idx) in row.iter().enumerate() {
            let (cx, cy) = (x + px as i32, y + py as i32);
            if cx < 0 || cy < 0 || cx >= CANVAS_W as i32 || cy >= CANVAS_H as i32 {
                continue;
            }
            let o = (cy as usize * CANVAS_W + cx as usize) * 4;
            if o + 4 <= dst.len() {
                dst[o..o + 4].copy_from_slice(&pal[(idx & 3) as usize]);
            }
        }
    }
}

/// 平台侧 CHR 背景（256×240 RGBA，可见性骨架；**非原版 BG/nametable/调色板语义**）。
///
/// 图样来源：字模区（`game_data::chr::FONT_BANK_START`=bank124）tile 按
/// `(tx+ty*3) % 64` 平铺；2bpp 解码=`game_data::chr::decode_tile`（t2 A 级布局），
/// 调色板=灰阶 [`GREY4`]（C 级未证占位）。语义与 `game-desktop` 可见性骨架同口径
/// （复制到平台侧，避免反向依赖工具壳）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChrBackground {
    pixels: Vec<u8>,
}

impl ChrBackground {
    /// 全黑背景（ROM 缺失/fail-closed 时的回退）。
    pub fn black() -> Self {
        let mut pixels = vec![0u8; CANVAS_LEN];
        for px in pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&CLEAR_RGBA);
        }
        Self { pixels }
    }

    /// 从 CHR ROM 生成背景；ROM 尺寸不足 CHR 域⇒`None`（fail-closed，不发明内容）。
    pub fn from_chr_rom(chr_rom: &[u8]) -> Option<Self> {
        let need = game_data::chr::CHR_FILE_BASE
            + game_data::chr::CHR_BANKS * game_data::chr::CHR_BANK_SIZE;
        if chr_rom.len() < need {
            return None;
        }
        let mut bg = Self::black();
        for ty in 0..(CANVAS_H / 8) {
            for tx in 0..(CANVAS_W / 8) {
                let tile = ((tx + ty * 3) % game_data::chr::CHR_TILES_PER_BANK) as usize;
                let bytes = game_data::chr::tile_bytes(chr_rom, game_data::chr::FONT_BANK_START, tile);
                let decoded = game_data::chr::decode_tile(&bytes);
                draw_tile8(&mut bg.pixels, &decoded, (tx * 8) as i32, (ty * 8) as i32, &GREY4);
            }
        }
        Some(bg)
    }

    /// RGBA 像素（256×240×4）。
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

/// 平台侧呈现合成（**v1 占位**）：`game_canvas`（game-shell 写出的 256×240 RGBA）中
/// **非清屏色** [`CLEAR_RGBA`] 的像素覆盖 CHR 背景，其余保留背景。
///
/// v1 game-shell 只画玩家 8×8 占位块（白色）⇒ 屏上=CHR 背景+玩家块。
/// `TODO(shell-v1)`：L4 渲染战役接管真实 BG/精灵合成后删除本函数；本合成**不**声称
/// 原版渲染语义（不做 nametable/调色板/精灵优先级）。
pub fn compose_canvas(background: &[u8], game_canvas: &[u8]) -> Vec<u8> {
    let mut out = background.to_vec();
    let n = out.len().min(game_canvas.len()) & !3usize;
    for o in (0..n).step_by(4) {
        let (r, g, b, a) = (
            game_canvas[o],
            game_canvas[o + 1],
            game_canvas[o + 2],
            game_canvas[o + 3],
        );
        if !(r == CLEAR_RGBA[0] && g == CLEAR_RGBA[1] && b == CLEAR_RGBA[2] && a == CLEAR_RGBA[3]) {
            out[o..o + 4].copy_from_slice(&game_canvas[o..o + 4]);
        }
    }
    out
}

/// 软缓冲渲染器（`game_platform::Renderer` 的 PC 头less 实现）：持有 game-shell 逻辑画布、
/// 平台侧 CHR 背景与合成结果；`present()`=合成 + 计数（窗口上屏由 `main.rs` 的
/// `WindowRenderer` 在 `present()` 后追加 softbuffer blit）。
#[derive(Debug)]
pub struct FrameRenderer {
    canvas: Vec<u8>,
    background: ChrBackground,
    display: Vec<u8>,
    presents: u32,
    size: (u32, u32),
}

impl FrameRenderer {
    /// 新建（背景=黑；画布=清屏色）。
    pub fn new() -> Self {
        let mut canvas = vec![0u8; CANVAS_LEN];
        for px in canvas.chunks_exact_mut(4) {
            px.copy_from_slice(&CLEAR_RGBA);
        }
        let background = ChrBackground::black();
        let display = background.pixels.clone();
        Self { canvas, background, display, presents: 0, size: (CANVAS_W as u32, CANVAS_H as u32) }
    }

    /// 设置 CHR 背景 ROM；尺寸不足⇒`false`（fail-closed，背景保持原值）。
    pub fn set_chr_rom(&mut self, chr_rom: &[u8]) -> bool {
        match ChrBackground::from_chr_rom(chr_rom) {
            Some(bg) => {
                self.background = bg;
                self.recompose();
                true
            }
            None => false,
        }
    }

    /// 当前 CHR 背景（只读）。
    pub fn background(&self) -> &ChrBackground {
        &self.background
    }

    /// 最近一次 `present()` 的合成结果（256×240 RGBA）。
    pub fn composed_canvas(&self) -> &[u8] {
        &self.display
    }

    /// 已 `present()` 次数。
    pub const fn presents(&self) -> u32 {
        self.presents
    }

    /// 用当前画布+背景重算合成结果（不计数；`set_chr_rom`/测试用）。
    pub fn recompose(&mut self) {
        self.display = compose_canvas(self.background.pixels(), &self.canvas);
    }
}

impl Renderer for FrameRenderer {
    fn canvas_mut(&mut self) -> &mut [u8] {
        &mut self.canvas
    }

    fn present(&mut self) {
        self.recompose();
        self.presents += 1;
    }

    fn size(&self) -> (u32, u32) {
        self.size
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }
}

impl Default for FrameRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// ===================== 确定性自检（--selftest-frames N） =====================

/// 自检事件脚本（帧号, 事件）：全部落在 `game_core::flow` **已证边集**内，
/// 依次 `Boot→Black→Title→Menu→Shop→Transition→StageStart→Combat`（与 game-shell
/// mock 契约测试同一边集；不发明菜单输入门控）。
pub const SELFTEST_EVENTS: [(u64, ModeEvent); 7] = [
    (10, ModeEvent::Black),
    (14, ModeEvent::Title),
    (18, ModeEvent::Menu),
    (22, ModeEvent::ShopArrival),
    (26, ModeEvent::ExitTransition),
    (30, ModeEvent::StageStart),
    (34, ModeEvent::MidCombat),
];

/// 自检按钮脚本：8 帧循环（右/左/上/下/L+R 同持/U+D 同持/A/B），覆盖对向消解。
fn selftest_buttons(frame: u64) -> Buttons {
    let mut b = Buttons::default();
    match frame % 8 {
        0 => b.right = true,
        1 => b.left = true,
        2 => b.up = true,
        3 => b.down = true,
        4 => {
            b.right = true;
            b.left = true;
        }
        5 => {
            b.up = true;
            b.down = true;
        }
        6 => b.a = true,
        _ => b.b = true,
    }
    b
}

/// 自检确定性报告（两次 `run_selftest` 必须逐字段相等）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelftestReport {
    /// 请求的逻辑帧数。
    pub frames: u64,
    /// 实际推进的逻辑帧数（固定步折算和）。
    pub advanced: u64,
    /// `Renderer::present` 次数。
    pub presents: u32,
    /// 终态模式。
    pub mode: game_core::flow::Mode,
    /// 关号（`$180`）。
    pub stage: u8,
    /// 虚拟时钟帧号。
    pub frame: u64,
    /// `Host::signature()`（宿主确定性签名）。
    pub host_signature: u64,
    /// 宿主记录的上一帧画布 hash。
    pub canvas_signature: u64,
    /// 平台侧合成输出（CHR 背景+玩家块）hash。
    pub display_signature: u64,
    /// 音频写事件消费计数。
    pub audio_writes: u64,
    /// `end_frame` 消费计数。
    pub audio_frames: u64,
    /// ROM 字节数（`AssetSource` 装载成功⇒Some）。
    pub rom_len: Option<usize>,
}

/// 无窗口确定性自检：`Host` + PC trait 实现驱动 `frames` 个逻辑帧，返回确定性报告。
///
/// 设计（同 game-shell mock 契约测试口径）：
/// - 每帧 `Host::advance((f+1)*FRAME_NS, …)`（同时验证固定步=每采样 1 帧）；
/// - 固定 `ModeEvent` 调度（[`SELFTEST_EVENTS`]，全已证边）+ 固定按钮脚本；
/// - 半程注入 2 条 `ApuWrite`（验证音频流真实消费；**非**已证音频生产者）；
/// - 真实 `AssetSource`（`std::fs` 读 ROM）→ CHR 背景；无窗口、无网络、不写用户目录。
pub fn run_selftest(frames: u64) -> SelftestReport {
    let mut host = Host::new();
    let mut assets = FsAssets::repo();
    let mut renderer = FrameRenderer::new();
    let mut audio = PcAudio::new();
    let mut input = KeyboardInput::new();

    host.load_asset(&mut assets, ROM_ASSET_PATH);
    if let Some(rom) = host.rom() {
        renderer.set_chr_rom(rom);
    }

    let inject_at = frames / 2;
    let mut next_event = 0usize;
    let mut advanced = 0u64;
    for f in 0..frames {
        while next_event < SELFTEST_EVENTS.len() && SELFTEST_EVENTS[next_event].0 == f {
            host.apply_event(SELFTEST_EVENTS[next_event].1)
                .expect("selftest 事件全在 game-core flow 已证边集内");
            next_event += 1;
        }
        if frames > 1 && f == inject_at {
            // 音频流消费契约验证（注入≠已证生产者；见 game-shell audio.rs）。
            host.audio_mut().stream_mut().write(0, 0x15, 0x0F);
            host.audio_mut().stream_mut().write(1, 0x00, 0x30);
        }
        input.set_buttons(selftest_buttons(f));
        let n = host.advance((f + 1) * FRAME_NS, &input, &mut renderer, &mut audio);
        debug_assert_eq!(n, 1, "固定步：每采样恰 1 逻辑帧");
        advanced += u64::from(n);
    }

    SelftestReport {
        frames,
        advanced,
        presents: renderer.presents(),
        mode: host.mode(),
        stage: host.stage_index(),
        frame: host.frame(),
        host_signature: host.signature(),
        canvas_signature: host.last_canvas_sig(),
        display_signature: game_shell::canvas_hash(renderer.composed_canvas()),
        audio_writes: audio.writes_total(),
        audio_frames: audio.frames_total(),
        rom_len: host.rom().map(|r| r.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::{key_name, map_key, KeyboardInput, CLEAR_RGBA};
    use game_platform::{Buttons, Input as _};
    use winit::keyboard::KeyCode;

    #[test]
    fn key_name_matches_map_key_table() {
        let pairs: [(KeyCode, &str, fn(Buttons) -> bool); 8] = [
            (KeyCode::KeyZ, "z", |b: Buttons| b.a),
            (KeyCode::KeyX, "x", |b| b.b),
            (KeyCode::Enter, "Enter", |b| b.start),
            (KeyCode::ShiftRight, "ShiftRight", |b| b.select),
            (KeyCode::ArrowUp, "ArrowUp", |b| b.up),
            (KeyCode::ArrowDown, "ArrowDown", |b| b.down),
            (KeyCode::ArrowLeft, "ArrowLeft", |b| b.left),
            (KeyCode::ArrowRight, "ArrowRight", |b| b.right),
        ];
        for (code, name, is_set) in pairs {
            assert_eq!(key_name(code), Some(name), "winit 码→规范名");
            let b = map_key(name).expect("规范名映射");
            assert!(is_set(b), "规范名 {name} 对应按钮置位");
            let mut input = KeyboardInput::new();
            assert!(input.apply_code(code, true), "物理键码按下");
            assert_eq!(input.buttons(), b, "KeyboardInput 状态");
            assert!(input.apply_code(code, false), "抬起");
            assert_eq!(input.buttons(), Buttons::default(), "抬起后归零");
        }
    }

    #[test]
    fn unmapped_keys_are_ignored() {
        assert_eq!(key_name(KeyCode::KeyQ), None);
        assert_eq!(map_key("q"), None);
        let mut input = KeyboardInput::new();
        assert!(!input.apply_code(KeyCode::KeyQ, true));
        assert_eq!(input.buttons(), Buttons::default());
    }

    #[test]
    fn clear_color_is_opaque_black() {
        assert_eq!(CLEAR_RGBA, [0, 0, 0, 0xFF]);
    }
}
