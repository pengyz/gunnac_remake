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
use game_shell::{FrameView, Host, FRAME_NS, ROM_ASSET_PATH};

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

/// PC 资产根解析（platform-shell.md §4/§5 S3）：**可执行文件目录优先**，不依赖进程 cwd。
///
/// 解析顺序：
/// 1. `exe_dir/rom/gunnac_cn.nes` ⇒ `exe_dir`（发布态：ROM 与 exe 同目录）；
/// 2. `exe_dir/../rom/gunnac_cn.nes` ⇒ `exe_dir/..`（发布态：ROM 在 exe 上一级）；
/// 3. **仅 debug 构建**回退 `CARGO_MANIFEST_DIR` 的仓库根（开发态 `cargo run`/`cargo test`）；
/// 4. 均不命中 ⇒ `None`（fail-closed；`FsAssets` 随后一律返回 `None`）。
///
/// 资产只经 `game_platform::AssetSource` 读取；本函数只决定 PC 文件根路径（平台侧语义）。
pub fn asset_root() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(root) = asset_root_from_exe(&exe) {
            return Some(root);
        }
    }
    #[cfg(debug_assertions)]
    {
        if let Some(root) = manifest_repo_root() {
            if root.join(ROM_ASSET_PATH).is_file() {
                return Some(root);
            }
        }
    }
    None
}

/// 从 exe 路径解析资产根：`exe_dir` 优先，其次 `exe_dir/..`；均需含 [`ROM_ASSET_PATH`]。
///
/// 纯路径解析 + `is_file` 判定（无 cwd 依赖），供 [`asset_root`] 与单测复用。
pub fn asset_root_from_exe(exe: &Path) -> Option<PathBuf> {
    let exe_dir = exe.parent()?;
    let mut candidates: Vec<PathBuf> = vec![exe_dir.to_path_buf()];
    if let Some(parent) = exe_dir.parent() {
        candidates.push(parent.to_path_buf());
    }
    candidates.into_iter().find(|c| c.join(ROM_ASSET_PATH).is_file())
}

/// 开发态仓库根（`CARGO_MANIFEST_DIR` 上两级）；**仅 debug 构建**可用（release 不内嵌源码路径）。
#[cfg(debug_assertions)]
fn manifest_repo_root() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().and_then(Path::parent).map(Path::to_path_buf)
}

/// `AssetSource` PC 实现：资产根 + 相对路径 ⇒ `std::fs` 读整块字节。
///
/// fail-closed：根未解析（[`asset_root`] 返回 `None`）、绝对路径、Windows 盘符/前缀路径
/// （`C:foo` 一类 drive-relative）、含 `..` 的越界路径、文件缺失 ⇒ 一律 `None`，
/// **不**返回默认内容、不依赖进程 cwd（AGENTS.md §3 未证不发明；解析/校验属 `game-data` 域）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsAssets {
    root: Option<PathBuf>,
}

impl FsAssets {
    /// 指定资产根目录（显式根，测试/嵌入场景）。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: Some(root.into()) }
    }

    /// 平台资产根自动检测（[`asset_root`]：exe 目录优先，debug 回退仓库根）。
    pub fn detect() -> Self {
        Self { root: asset_root() }
    }

    /// 资产根目录（未解析⇒`None`；启动时应打印该值，见 `main.rs`）。
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }
}

/// 相对路径安全检查（拒绝绝对路径、Windows 盘符/前缀路径与 `..` 越界）。
fn safe_relative(path: &str) -> bool {
    let p = Path::new(path);
    if p.is_absolute() {
        return false;
    }
    !p.components().any(|c| {
        matches!(
            c,
            // Windows drive-relative（"C:foo"）不是绝对路径，但会逃逸资产根（S1/B1）。
            Component::Prefix(_) | Component::ParentDir | Component::RootDir
        )
    })
}

impl AssetSource for FsAssets {
    fn read_all(&mut self, path: &str) -> Option<Vec<u8>> {
        let root = self.root.as_ref()?;
        if !safe_relative(path) {
            return None;
        }
        std::fs::read(root.join(path)).ok()
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

// ===================== FrameView 覆盖层（v2：模式画面 / HUD 占位 / 敌槽） =====================

/// 模式标记占位色（`game_shell::snapshot::mode_tag` 0..=13 索引）。
///
/// **占位调色板**：仅用于"模式可见区分"，**非原版调色板/美术/文字/布局**——不得据此写
/// 渲染一致性结论（`TODO(shell-v2)`：L4 战役锁定真实模式画面后删除）。
pub const MODE_MARKER_COLORS: [[u8; 4]; 14] = [
    [0x40, 0x40, 0x40, 0xFF], // 0 Boot
    [0x00, 0x00, 0x80, 0xFF], // 1 Black
    [0x80, 0x80, 0x00, 0xFF], // 2 Title
    [0x00, 0x80, 0x00, 0xFF], // 3 Menu
    [0x80, 0x00, 0x80, 0xFF], // 4 Shop
    [0x00, 0x80, 0x80, 0xFF], // 5 ShopInfo
    [0x80, 0x40, 0x00, 0xFF], // 6 Transition
    [0x80, 0x00, 0x00, 0xFF], // 7 StageStart
    [0xFF, 0xFF, 0xFF, 0xFF], // 8 Combat
    [0xFF, 0x00, 0x00, 0xFF], // 9 DeathRespawn
    [0x00, 0xFF, 0x00, 0xFF], // 10 GameOverPending
    [0x00, 0x00, 0xFF, 0xFF], // 11 GameOverMenu
    [0x00, 0xFF, 0xFF, 0xFF], // 12 StageClear
    [0xFF, 0xFF, 0x00, 0xFF], // 13 Ending
];

/// 敌槽占位色（`type_code & 3` 索引；**非原版图样/调色板**，只用于占位区分，语义不命名）。
pub const ENEMY_PLACEHOLDER_COLORS: [[u8; 4]; 4] = [
    [0xFF, 0x00, 0xFF, 0xFF],
    [0x00, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0x00, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
];

/// HUD 占位槽位（**固定占位布局，非原版 HUD 屏幕坐标**；`TODO(shell-v2)`）。
pub const HUD_MONEY_SLOT_X: i32 = 8;
/// 金钱桁位占位 y。
pub const HUD_MONEY_SLOT_Y: i32 = 224;
/// 金钱桁位占位间距（每桁 4px：2×2 点阵 + 2px 间隔）。
pub const HUD_MONEY_SLOT_PITCH: i32 = 4;
/// 炸弹弹药位行占位 x（8 bit × 2px）。
pub const HUD_BOMB_SLOT_X: i32 = 8;
/// 炸弹弹药位行占位 y。
pub const HUD_BOMB_SLOT_Y: i32 = 216;
/// 关号位行占位 x（4 bit × 2px，位于炸弹位行右侧）。
pub const HUD_STAGE_SLOT_X: i32 = 28;
/// 关号位行占位 y。
pub const HUD_STAGE_SLOT_Y: i32 = 216;

/// HUD 占位色（**非原版调色板**）。
pub const HUD_PLACEHOLDER_RGBA: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

/// 像素块填充（像素坐标；越界裁剪不回绕）。
fn fill_block(dst: &mut [u8], x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
    for dy in 0..h {
        for dx in 0..w {
            let (cx, cy) = (x + dx, y + dy);
            if cx < 0 || cy < 0 || cx >= CANVAS_W as i32 || cy >= CANVAS_H as i32 {
                continue;
            }
            let o = (cy as usize * CANVAS_W + cx as usize) * 4;
            if o + 4 <= dst.len() {
                dst[o..o + 4].copy_from_slice(&color);
            }
        }
    }
}

/// 桁位占位编码：digit(0..9) 的 bit0..3 → 2×2 像素点阵（bit 序：左上/右上/左下/右下）。
///
/// **占位编码，非原版字形/位图**（HUD 字形/布局未证；`TODO(shell-v2)`）。
fn draw_digit_2x2(dst: &mut [u8], x: i32, y: i32, digit: u8, color: [u8; 4]) {
    let v = digit & 0x0F;
    for bit in 0..4u8 {
        if v & (1 << bit) != 0 {
            let bx = x + i32::from(bit % 2) * 2;
            let by = y + i32::from(bit / 2) * 2;
            fill_block(dst, bx, by, 2, 2, color);
        }
    }
}

/// 数值位行占位编码：bit i → 第 i 个 2×2 块（低 bit 在左）。
///
/// **占位编码，非原版 HUD 图标/布局**（`TODO(shell-v2)`）。
fn draw_bit_row(dst: &mut [u8], x: i32, y: i32, value: u8, bits: u8, color: [u8; 4]) {
    for i in 0..bits {
        if value & (1 << i) != 0 {
            fill_block(dst, x + i32::from(i) * 2, y, 2, 2, color);
        }
    }
}

/// FNV-1a 64 偏移基。
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64 素数。
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *hash ^= u64::from(b);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

fn hash_u64(hash: &mut u64, v: u64) {
    hash_bytes(hash, &v.to_le_bytes());
}

/// `FrameView` 确定性摘要（FNV-1a 64；**平台无关**：只覆盖 view 字段，不混 renderer 状态）。
///
/// 用途：`--selftest-frames` 的 `view_signature`（两次一致）与排障对照；
/// **不改** `game_shell::Host::signature()` 语义（两者独立）。
pub fn frame_view_digest(view: &FrameView) -> u64 {
    let mut h = FNV_OFFSET;
    hash_u64(&mut h, view.frame);
    hash_bytes(&mut h, &[game_shell::snapshot::mode_tag(view.mode)]);
    hash_bytes(&mut h, &[view.stage]);
    let p = view.player;
    hash_bytes(
        &mut h,
        &[p.x, p.y, p.x_subpix, p.y_subpix, p.bomb_ammo, p.action_mode],
    );
    hash_u64(&mut h, u64::from(view.live_bullets));
    hash_u64(&mut h, u64::from(view.player_bullets_live));
    for slot in &view.enemy_slots {
        hash_bytes(&mut h, &[slot.slot]);
        hash_bytes(&mut h, &[slot.type_code.unwrap_or(0)]);
        // 摘要哨兵歧义（E4 已登记，v1 不修）：`position=None` 用 `(0xFF,0xFF)` 与未来真实
        // `Some((255,255))` 理论同摘要；因 v1 position 恒 None（坐标写者链未证）无实际影响。
        // 若未来接线真实坐标，需先给 None/Some 加 tag 区分（会改变 view_signature 记录值）。
        let pos = slot.position.unwrap_or((0xFF, 0xFF));
        hash_bytes(&mut h, &[pos.0, pos.1]);
    }
    hash_bytes(&mut h, &[view.hud.bomb_ammo, view.hud.stage]);
    for (digit, show) in view.hud.money_digits {
        hash_bytes(&mut h, &[digit, u8::from(show)]);
    }
    h
}

/// 把 `FrameView` 覆盖层画到 256×240 RGBA 画布（**平台侧占位呈现**）：
///
/// - **模式标记**：8×8 角标（`mode_tag`→[`MODE_MARKER_COLORS`]）——模式可见区分，非原版画面；
/// - **HUD**：`money_digits` 的 `(digit, show)` 逐桁 2×2 点阵 + `bomb_ammo` 8 位位行 +
///   `stage` 4 位位行（固定占位槽，**非原版字形/布局/文本**）；
/// - **敌槽**：仅 `position: Some(_)` 且 `type_code: Some(_)` 画 8×8 占位块（v1 运行态 position
///   恒 `None` ⇒ 不画；**禁止用 type_code 反推坐标**；图样未证=`TODO(shell-v2)`）；
/// - **玩家子弹**：**不画**（FrameView 无坐标面；H-01 未证）——只在 [`frame_view_digest`] 登记计数。
pub fn draw_frame_view_overlay(target: &mut [u8], view: &FrameView) {
    let tag = usize::from(game_shell::snapshot::mode_tag(view.mode));
    let mode_color = MODE_MARKER_COLORS[tag.min(MODE_MARKER_COLORS.len() - 1)];
    fill_block(target, 0, 0, 8, 8, mode_color);

    for (i, (digit, show)) in view.hud.money_digits.iter().enumerate() {
        if *show {
            let x = HUD_MONEY_SLOT_X + i as i32 * HUD_MONEY_SLOT_PITCH;
            draw_digit_2x2(target, x, HUD_MONEY_SLOT_Y, *digit, HUD_PLACEHOLDER_RGBA);
        }
    }
    draw_bit_row(target, HUD_BOMB_SLOT_X, HUD_BOMB_SLOT_Y, view.hud.bomb_ammo, 8, HUD_PLACEHOLDER_RGBA);
    draw_bit_row(target, HUD_STAGE_SLOT_X, HUD_STAGE_SLOT_Y, view.hud.stage & 0x0F, 4, GREY4[2]);

    for slot in &view.enemy_slots {
        if let (Some(pos), Some(type_code)) = (slot.position, slot.type_code) {
            let color = ENEMY_PLACEHOLDER_COLORS[usize::from(type_code) & 3];
            fill_block(target, i32::from(pos.0), i32::from(pos.1), 8, 8, color);
        }
    }
    // TODO(shell-v2)：玩家子弹坐标/弹道未证（H-01）⇒ 不画；活弹计数只入 view 摘要。
}

/// 在 v1 合成（CHR 背景+game-shell 画布）之上叠加 `FrameView` 覆盖层。
///
/// **占位呈现**：不声称原版 BG/精灵/HUD/模式画面像素；L4 战役接管后删除。
pub fn compose_canvas_with_view(background: &[u8], game_canvas: &[u8], view: &FrameView) -> Vec<u8> {
    let mut out = compose_canvas(background, game_canvas);
    draw_frame_view_overlay(&mut out, view);
    out
}

/// 软缓冲渲染器（`game_platform::Renderer` 的 PC 头less 实现）：持有 game-shell 逻辑画布、
/// 平台侧 CHR 背景、`FrameView` 覆盖层与合成结果；`present()`=合成 + 计数（窗口上屏由
/// `main.rs` 的 `WindowRenderer` 在 `present()` 后追加 softbuffer blit）。
#[derive(Debug)]
pub struct FrameRenderer {
    canvas: Vec<u8>,
    background: ChrBackground,
    display: Vec<u8>,
    /// 最近一帧只读视图（平台侧覆盖层数据源；`None`=不画覆盖层）。
    view: Option<FrameView>,
    presents: u32,
    size: (u32, u32),
}

impl FrameRenderer {
    /// 新建（背景=黑；画布=清屏色；view=None）。
    pub fn new() -> Self {
        let mut canvas = vec![0u8; CANVAS_LEN];
        for px in canvas.chunks_exact_mut(4) {
            px.copy_from_slice(&CLEAR_RGBA);
        }
        let background = ChrBackground::black();
        let display = background.pixels.clone();
        Self {
            canvas,
            background,
            display,
            view: None,
            presents: 0,
            size: (CANVAS_W as u32, CANVAS_H as u32),
        }
    }

    /// 更新只读视图并立即重合成（平台侧覆盖层；`Host::frame_view()` 的最新值）。
    ///
    /// 约定：`Host::tick` 内部的 `present()` 使用上一帧 view；shell 在 `advance()` 后调用本方法
    /// 用本帧最新 view 重合成再上屏（present 计数不再增加）。
    pub fn set_frame_view(&mut self, view: FrameView) {
        self.view = Some(view);
        self.recompose();
    }

    /// 当前只读视图（若有）。
    pub const fn frame_view(&self) -> Option<FrameView> {
        self.view
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

    /// 用当前画布+背景+覆盖层重算合成结果（不计数；`set_chr_rom`/`set_frame_view`/测试用）。
    pub fn recompose(&mut self) {
        self.display = match self.view.as_ref() {
            Some(view) => compose_canvas_with_view(self.background.pixels(), &self.canvas, view),
            None => compose_canvas(self.background.pixels(), &self.canvas),
        };
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
    /// 平台侧合成输出（CHR 背景+玩家块+FrameView 覆盖层）hash。
    pub display_signature: u64,
    /// `FrameView` 字段摘要（独立、平台无关；[`frame_view_digest`]）。
    pub view_digest: u64,
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
/// - 每帧用 `Host::frame_view()` 最新只读视图重合成平台侧覆盖层（模式标记/HUD 占位/敌槽）；
/// - 真实 `AssetSource`（`std::fs` 读 ROM）→ CHR 背景；无窗口、无网络、不写用户目录。
pub fn run_selftest(frames: u64) -> SelftestReport {
    let mut host = Host::new();
    let mut assets = FsAssets::detect();
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
        if n > 0 {
            // 平台侧：用本帧最新只读视图重合成覆盖层（避免覆盖层滞后一帧；不增 present 计数）。
            renderer.set_frame_view(host.frame_view());
        }
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
        view_digest: frame_view_digest(&host.frame_view()),
        audio_writes: audio.writes_total(),
        audio_frames: audio.frames_total(),
        rom_len: host.rom().map(|r| r.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        asset_root_from_exe, key_name, map_key, safe_relative, KeyboardInput, CLEAR_RGBA,
    };
    use game_platform::{Buttons, Input as _};
    use winit::keyboard::KeyCode;

    fn unique_temp_dir(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("gunnac-shellwin-{tag}-{}", std::process::id()))
    }

    #[test]
    fn asset_root_from_exe_prefers_exe_dir_then_parent() {
        let base = unique_temp_dir("asset-root");
        let _ = std::fs::remove_dir_all(&base);

        // 布局 A（发布态）：ROM 与 exe 同目录 ⇒ 资产根=exe 目录。
        let game = base.join("game");
        std::fs::create_dir_all(game.join("rom")).unwrap();
        std::fs::write(game.join(super::ROM_ASSET_PATH), b"fake").unwrap();
        assert_eq!(
            asset_root_from_exe(&game.join("shell-win.exe")),
            Some(game.clone()),
            "exe 目录优先"
        );

        // 布局 B（发布态）：ROM 在 exe 上一级 ⇒ 资产根=exe/..（读 `exe/../rom/...`）。
        let bundle = base.join("bundle");
        let bin = bundle.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(bundle.join("rom")).unwrap();
        std::fs::write(bundle.join(super::ROM_ASSET_PATH), b"fake").unwrap();
        assert_eq!(
            asset_root_from_exe(&bin.join("shell-win.exe")),
            Some(bundle),
            "回退 exe/../rom"
        );

        // 两候选均无 ROM ⇒ None（fail-closed；不猜进程 cwd）。
        assert_eq!(
            asset_root_from_exe(&base.join("nowhere").join("x.exe")),
            None
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn safe_relative_rejects_escape_paths() {
        assert!(safe_relative("rom/gunnac_cn.nes"), "普通相对路径放行");
        assert!(!safe_relative("../AGENTS.md"), "`..` 越界拒绝");
        assert!(!safe_relative("/etc/passwd"), "unix 绝对路径拒绝");
        #[cfg(windows)]
        {
            // B1：drive-relative 前缀（`C:foo`）不是绝对路径但会逃逸资产根。
            assert!(!safe_relative("C:foo"), "Windows drive-relative 前缀拒绝");
            assert!(!safe_relative("D:foo\\x"), "盘符前缀+反斜杠拒绝");
            assert!(!safe_relative("C:\\Windows\\x"), "盘符绝对路径拒绝");
        }
    }

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

#[cfg(test)]
mod view_tests {
    use super::{
        compose_canvas_with_view, draw_frame_view_overlay, frame_view_digest, ChrBackground,
        FrameRenderer, FrameView, CLEAR_RGBA, ENEMY_PLACEHOLDER_COLORS, GREY4, HUD_BOMB_SLOT_X,
        HUD_BOMB_SLOT_Y, HUD_MONEY_SLOT_PITCH, HUD_MONEY_SLOT_X, HUD_MONEY_SLOT_Y,
        HUD_PLACEHOLDER_RGBA, HUD_STAGE_SLOT_X, HUD_STAGE_SLOT_Y, MODE_MARKER_COLORS,
    };
    use game_core::flow::Mode;
    use game_platform::{Renderer, CANVAS_LEN, CANVAS_W};
    use game_shell::EnemySlotView;

    /// 基线 FrameView（Combat + 其余默认：空敌槽/零 HUD；task-37 起用 `..Default::default()`）。
    fn view_fixture() -> FrameView {
        FrameView { mode: Mode::Combat, ..FrameView::default() }
    }

    fn black_canvas() -> Vec<u8> {
        let mut c = vec![0u8; CANVAS_LEN];
        for px in c.chunks_exact_mut(4) {
            px.copy_from_slice(&CLEAR_RGBA);
        }
        c
    }

    fn px(canvas: &[u8], x: usize, y: usize) -> [u8; 4] {
        let o = (y * CANVAS_W + x) * 4;
        [canvas[o], canvas[o + 1], canvas[o + 2], canvas[o + 3]]
    }

    #[test]
    fn frame_view_digest_is_deterministic_and_sensitive() {
        let base = view_fixture();
        assert_eq!(frame_view_digest(&base), frame_view_digest(&base), "同视图⇒同摘要");

        let mut changed = base;
        changed.frame = 1;
        assert_ne!(frame_view_digest(&base), frame_view_digest(&changed), "frame 敏感");

        let mut changed = base;
        changed.mode = Mode::Menu;
        assert_ne!(frame_view_digest(&base), frame_view_digest(&changed), "mode 敏感");

        let mut changed = base;
        changed.player.x = base.player.x.wrapping_add(1);
        assert_ne!(frame_view_digest(&base), frame_view_digest(&changed), "player 敏感");

        let mut changed = base;
        changed.hud.bomb_ammo = 3;
        assert_ne!(frame_view_digest(&base), frame_view_digest(&changed), "HUD 敏感");

        let mut changed = base;
        changed.enemy_slots[0].type_code = Some(5);
        assert_ne!(frame_view_digest(&base), frame_view_digest(&changed), "敌槽型值敏感");

        let mut changed = base;
        changed.player_bullets_live = 2;
        assert_ne!(frame_view_digest(&base), frame_view_digest(&changed), "玩家子弹计数敏感");
    }

    #[test]
    fn mode_marker_uses_placeholder_color_per_mode_tag() {
        let mut canvas = black_canvas();
        let mut view = view_fixture();
        view.mode = Mode::Menu; // mode_tag=3
        draw_frame_view_overlay(&mut canvas, &view);
        assert_eq!(px(&canvas, 0, 0), MODE_MARKER_COLORS[3], "Menu 角标占位色");
        assert_eq!(px(&canvas, 7, 7), MODE_MARKER_COLORS[3], "8×8 角标");

        let mut canvas = black_canvas();
        view.mode = Mode::Combat; // mode_tag=8
        draw_frame_view_overlay(&mut canvas, &view);
        assert_eq!(px(&canvas, 0, 0), MODE_MARKER_COLORS[8], "Combat 角标占位色");
        assert_ne!(MODE_MARKER_COLORS[3], MODE_MARKER_COLORS[8], "模式可见区分");
    }

    #[test]
    fn hud_money_digits_use_explicit_2x2_placeholder_encoding() {
        let mut canvas = black_canvas();
        let mut view = view_fixture();
        // digit=5 (0b0101)：bit0⇒左上块、bit2⇒左下块；show=false 的桁位不画。
        view.hud.money_digits[0] = (5, true);
        view.hud.money_digits[1] = (9, false);
        draw_frame_view_overlay(&mut canvas, &view);
        let (x0, y0) = (HUD_MONEY_SLOT_X as usize, HUD_MONEY_SLOT_Y as usize);
        assert_eq!(px(&canvas, x0, y0), HUD_PLACEHOLDER_RGBA, "bit0→左上 2×2");
        assert_eq!(px(&canvas, x0 + 1, y0 + 1), HUD_PLACEHOLDER_RGBA, "2×2 块");
        assert_eq!(px(&canvas, x0, y0 + 2), HUD_PLACEHOLDER_RGBA, "bit2→左下 2×2");
        assert_eq!(px(&canvas, x0 + 2, y0), CLEAR_RGBA, "bit1=0⇒右上不画");
        let x1 = (HUD_MONEY_SLOT_X + HUD_MONEY_SLOT_PITCH) as usize;
        assert_eq!(px(&canvas, x1, y0), CLEAR_RGBA, "show=false 桁位不画");
    }

    #[test]
    fn hud_bomb_and_stage_use_bit_row_placeholder() {
        let mut canvas = black_canvas();
        let mut view = view_fixture();
        view.hud.bomb_ammo = 0b0000_0101; // bit0/bit2
        view.hud.stage = 3; // bit0/bit1
        draw_frame_view_overlay(&mut canvas, &view);
        let (bx, by) = (HUD_BOMB_SLOT_X as usize, HUD_BOMB_SLOT_Y as usize);
        assert_eq!(px(&canvas, bx, by), HUD_PLACEHOLDER_RGBA, "bomb bit0");
        assert_eq!(px(&canvas, bx + 4, by), HUD_PLACEHOLDER_RGBA, "bomb bit2（第 2 列）");
        assert_eq!(px(&canvas, bx + 2, by), CLEAR_RGBA, "bomb bit1=0");
        let (sx, sy) = (HUD_STAGE_SLOT_X as usize, HUD_STAGE_SLOT_Y as usize);
        assert_eq!(px(&canvas, sx, sy), GREY4[2], "stage bit0");
        assert_eq!(px(&canvas, sx + 2, sy), GREY4[2], "stage bit1");
    }

    #[test]
    fn enemy_slots_draw_only_when_position_and_type_present() {
        let mut view = view_fixture();
        view.enemy_slots[0] = EnemySlotView { slot: 8, type_code: Some(1), position: Some((40, 50)) };
        view.enemy_slots[1] = EnemySlotView { slot: 9, type_code: None, position: Some((60, 50)) };
        view.enemy_slots[2] = EnemySlotView { slot: 10, type_code: Some(2), position: None };
        let mut canvas = black_canvas();
        draw_frame_view_overlay(&mut canvas, &view);
        assert_eq!(
            px(&canvas, 40, 50),
            ENEMY_PLACEHOLDER_COLORS[1],
            "position+type 均 Some⇒8×8 占位块"
        );
        assert_eq!(px(&canvas, 60, 50), CLEAR_RGBA, "type=None 不画（空槽）");
        assert_eq!(px(&canvas, 10, 50), CLEAR_RGBA, "position=None 不画（不按 (0,0) 画）");
    }

    #[test]
    fn compose_with_view_keeps_player_canvas_and_adds_overlay() {
        let background = ChrBackground::black();
        let mut game = black_canvas();
        // game-shell 画布：玩家块（白色）在 (10,20)。
        let o = (20 * CANVAS_W + 10) * 4;
        game[o..o + 4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        let mut view = view_fixture();
        view.mode = Mode::Title;
        let out = compose_canvas_with_view(background.pixels(), &game, &view);
        assert_eq!(px(&out, 10, 20), [0xFF, 0xFF, 0xFF, 0xFF], "玩家块保留");
        assert_eq!(px(&out, 0, 0), MODE_MARKER_COLORS[2], "叠加模式角标");
    }

    #[test]
    fn frame_renderer_set_frame_view_recomposes_without_counting_present() {
        let mut renderer = FrameRenderer::new();
        renderer.present();
        let before = game_shell::canvas_hash(renderer.composed_canvas());
        assert_eq!(renderer.presents(), 1);

        let mut view = view_fixture();
        view.mode = Mode::Shop;
        renderer.set_frame_view(view);
        let after = game_shell::canvas_hash(renderer.composed_canvas());
        assert_ne!(before, after, "覆盖层改变合成结果");
        assert_eq!(renderer.presents(), 1, "set_frame_view 不增加 present 计数");
        assert_eq!(px(renderer.composed_canvas(), 0, 0), MODE_MARKER_COLORS[4]);
    }
}
