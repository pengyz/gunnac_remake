//! game-desktop —— 桌面渲染层骨架（M3 渲染层，t58）。
//!
//! Stage A 口径：256×240 逻辑画布**整数倍缩放**（禁缩放变形，黑边保比例）；
//! CHR tile 解码语义=`game-data::chr`（chr_render.py 投影同源）；
//! 调色板=参数注入（真调色板未证，待 L4 oracle 对拍锁定——禁止发明）；
//! 游戏状态推进走 game-core 纯逻辑（虚拟时钟 FrameClock），本 crate 只做 I/O 与合成。
//!
//! 依赖替换注记（t58）：任务指定 winit+pixels；离线 cargo 缓存无 pixels（crates.io 403），
//! 采用同语义软件帧缓冲 **softbuffer**（winit 官方 CPU 缓冲表面）——架构缝不变
//! （winit 窗口 + CPU 帧缓冲 + Renderer trait）。

use game_data::chr::decode_tile;

pub const SCREEN_W: usize = 256;
pub const SCREEN_H: usize = 240;

/// RGBA 颜色（调色板项；颜色映射语义未证，参数化）。
pub type Rgba = [u8; 4];

/// chr_render.py 投影灰阶调色板（索引 0..3 → 灰阶），取证期默认；
/// **非**原版调色板（C 级未证）——L4 对拍后换真表。
pub const GREY4: [Rgba; 4] = [
    [0, 0, 0, 255],
    [85, 85, 85, 255],
    [170, 170, 170, 255],
    [255, 255, 255, 255],
];

/// 256×240 RGBA 软件帧缓冲。
pub struct Canvas {
    pub pixels: Vec<u8>, // SCREEN_W*SCREEN_H*4
}

impl Canvas {
    pub fn new() -> Self {
        Canvas { pixels: vec![0; SCREEN_W * SCREEN_H * 4] }
    }

    pub fn clear(&mut self, color: Rgba) {
        for px in self.pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&color);
        }
    }

    /// 8×8 tile 绘制（像素坐标；越界裁剪不回绕）。
    pub fn draw_tile8(&mut self, tile: &[[u8; 8]; 8], x: i32, y: i32, pal: &[Rgba; 4]) {
        for (py, row) in tile.iter().enumerate() {
            for (px, &idx) in row.iter().enumerate() {
                let (cx, cy) = (x + px as i32, y + py as i32);
                if cx < 0 || cy < 0 || cx >= SCREEN_W as i32 || cy >= SCREEN_H as i32 {
                    continue;
                }
                let o = (cy as usize * SCREEN_W + cx as usize) * 4;
                self.pixels[o..o + 4].copy_from_slice(&pal[(idx & 3) as usize]);
            }
        }
    }

    /// CHR tile 直绘（bank/tile 索引；解码语义=game-data::chr）。
    pub fn draw_chr(&mut self, rom: &[u8], bank: usize, tile: usize, x: i32, y: i32, pal: &[Rgba; 4]) {
        let mut t = [0u8; 16];
        let base = game_data::chr::CHR_FILE_BASE + bank * game_data::chr::CHR_BANK_SIZE;
        t.copy_from_slice(&rom[base + tile * 16..base + tile * 16 + 16]);
        self.draw_tile8(&decode_tile(&t), x, y, pal);
    }

    /// 取像素（测试/L4 真值源读取口）。
    pub fn pixel(&self, x: usize, y: usize) -> Rgba {
        let o = (y * SCREEN_W + x) * 4;
        [self.pixels[o], self.pixels[o + 1], self.pixels[o + 2], self.pixels[o + 3]]
    }

    /// L4 预备：帧缓冲导出（Rust 侧真值源）——PNG（零依赖 zlib-stored）。
    pub fn export_png(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut raw = Vec::with_capacity(SCREEN_H * (1 + SCREEN_W * 4));
        for row in self.pixels.chunks_exact(SCREEN_W * 4) {
            raw.push(0); // PNG filter=None
            raw.extend_from_slice(row);
        }
        let mut png = Vec::new();
        png.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&(SCREEN_W as u32).to_be_bytes());
        ihdr.extend_from_slice(&(SCREEN_H as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8bit RGBA
        write_chunk(&mut png, b"IHDR", &ihdr);
        write_chunk(&mut png, b"IDAT", &zlib_stored(&raw));
        write_chunk(&mut png, b"IEND", &[]);
        std::fs::write(path, png)
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

/// game-platform::Renderer 缝实现（占位语义：x,y=像素坐标，tile=8×8 块绘制）。
/// 注：trait 占位形状的坐标语义待 S4/L4 锁定（AGENTS.md §2）；此实现仅骨架自洽。
impl game_platform::Renderer for Canvas {
    fn clear(&mut self) {
        self.clear([0, 0, 0, 255]);
    }
    fn draw_tile(&mut self, tile: u8, x: u8, y: u8, palette: u8) {
        let pal = GREY4; // 占位：palette 参数语义未证，暂以灰阶 + palette 变体偏移
        let mut shifted = pal;
        let _ = palette;
        let blank = [[tile & 3; 8]; 8];
        self.draw_tile8(&blank, x as i32, y as i32, &shifted);
    }
    fn present(&mut self) {}
}

/// 整数倍缩放视口（Stage A 口径：禁变形，黑边保比例）。
/// 返回 (ox, oy, w, h)：窗口内 256×240 的整数倍视口（scale≥1；窗口小于画布时 scale=1 裁切）。
pub fn view_rect(win_w: u32, win_h: u32) -> (u32, u32, u32, u32) {
    let sw = SCREEN_W as u32;
    let sh = SCREEN_H as u32;
    let scale = std::cmp::max(1, std::cmp::min(win_w / sw, win_h / sh));
    let (w, h) = (sw * scale, sh * scale);
    let ox = win_w.saturating_sub(w) / 2;
    let oy = win_h.saturating_sub(h) / 2;
    (ox, oy, w, h)
}

/// 按整数倍缩放把画布合成到窗口缓冲（0x00RRGGBB，softbuffer 格式）；黑边清 0。
pub fn blit_scaled(buf: &mut [u32], win_w: u32, win_h: u32, canvas: &Canvas) {
    assert_eq!(buf.len(), (win_w * win_h) as usize, "窗口缓冲尺寸不符");
    buf.fill(0);
    let (ox, oy, w, h) = view_rect(win_w, win_h);
    let scale = w / SCREEN_W as u32;
    for y in 0..h {
        for x in 0..w {
            let (cx, cy) = ((x / scale) as usize, (y / scale) as usize);
            let o = (cy * SCREEN_W + cx) * 4;
            let (r, g, b) = (canvas.pixels[o] as u32, canvas.pixels[o + 1] as u32, canvas.pixels[o + 2] as u32);
            buf[((oy + y) * win_w + ox + x) as usize] = (r << 16) | (g << 8) | b;
        }
    }
}

/// 平台侧输入策略（game-platform 注：连发/宏属平台侧）：**对向键互斥**（L+R/U+D 同持
/// 在 game-core 是未证 panic（W6）——平台必须消解为 0），转 game-core held 位图。
/// 位图（B 级，bank00:9D84 用法）：bit0=右 bit1=左 bit2=下 bit3=上 bit6=B bit7=A。
pub fn held_from_keys(right: bool, left: bool, down: bool, up: bool, a: bool, b: bool) -> u8 {
    let mut held = 0u8;
    if right && !left { held |= 0x01; }
    if left && !right { held |= 0x02; }
    if down && !up { held |= 0x04; }
    if up && !down { held |= 0x08; }
    if b { held |= 0x40; }
    if a { held |= 0x80; }
    held
}

fn write_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(tag, data).to_be_bytes());
}

fn zlib_stored(data: &[u8]) -> Vec<u8> {
    // zlib 头 + stored-deflate 块（BFINAL=1,BTYPE=00）+ adler32 —— 零依赖 PNG 数据段
    let mut out = vec![0x78, 0x01];
    for chunk in data.chunks(0xFFFF) {
        let n = chunk.len() as u16;
        let final_block = chunk.len() < 0xFFFF;
        out.push(final_block as u8);
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&(!n).to_le_bytes());
        out.extend_from_slice(chunk);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn crc32(tag: &[u8], data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &x in tag.iter().chain(data.iter()) {
        c ^= x as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    !c
}
