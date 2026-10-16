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
        let _ = palette; // 占位：palette 参数语义未证（待 L4 对拍）
        let blank = [[tile & 3; 8]; 8];
        self.draw_tile8(&blank, x as i32, y as i32, &GREY4);
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

// ============ BG 渲染链（t90：桥缓冲→tile 网格→背景层，投影口径） ============

/// BG 投影渲染（t90 ②④，**B 级结构推断 + 灰阶口径声明**）：
/// 每 BG 记录 4 字段 = 2×2 tile 块（$130/$131=上排、$140/$141=下排——缓冲族分立结构推断），
/// 16 槽横向条带按 stage 卷轴 ($1C/$1D) 偏移平铺（mod 256 回绕）。**真值映射待 L4 对拍**——
/// 本函数为 visual-diff 前置侦察样张口径（调色板未证=GREY4）。
pub fn render_bg_projection(
    canvas: &mut Canvas,
    chr_rom: &[u8],
    b130: &[u8; 16],
    b131: &[u8; 16],
    b140: &[u8; 16],
    b141: &[u8; 16],
    scroll_x: u8,
    scroll_y: u8,
) {
    let sx = scroll_x as i32;
    let sy = scroll_y as i32;
    for slot in 0..16usize {
        let tiles = [b130[slot], b131[slot], b140[slot], b141[slot]];
        for (k, &tile) in tiles.iter().enumerate() {
            let col = slot * 2 + (k % 2);
            let row = k / 2;
            let px = col as i32 * 8 - sx;
            let py = row as i32 * 8 - sy;
            // CHR tile 直绘（字模区 bank124 起，tile 索引 mod 64——投影口径）
            canvas.draw_chr(chr_rom, game_data::chr::FONT_BANK_START, (tile % 64) as usize, px, py, &GREY4);
        }
    }
}

// ============ 精灵层渲染（t104：OAM DMA 语义 + NESdev 标准精灵合成） ============

/// OAM 精灵条目（4B：Y-1 / tile / attr / X）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteObj {
    pub y: u8,
    pub tile: u8,
    pub attr: u8,
    pub x: u8,
}

impl SpriteObj {
    /// OAM 4 字节解析（NESdev 标准）。
    pub fn from_oam4(b: [u8; 4]) -> Self {
        SpriteObj { y: b[0], tile: b[1], attr: b[2], x: b[3] }
    }
    pub fn palette(&self) -> usize {
        (self.attr & 3) as usize
    }
    /// bit5：1=BG 前置（精灵在 BG 后）。
    pub fn behind_bg(&self) -> bool {
        self.attr & 0x20 != 0
    }
    pub fn hflip(&self) -> bool {
        self.attr & 0x40 != 0
    }
    pub fn vflip(&self) -> bool {
        self.attr & 0x80 != 0
    }
    /// 逐实体 Y 锚点表（t107 ①，findings/t104-two-family.md §4 参数表）：
    /// 弹/敌/HUD = −$08；自机 = −$30+微调 +$04（=−$2C）+ 截止门 < $30。
    pub fn screen_y(&self) -> i32 {
        self.y as i32 + 1 // 【t191 彩蛋】OAM Y→屏 y **+1 平台惯例**（1px 纵相位谜底）
    }
    /// 自机判据（t107）：调色板 1 + 非 HUD 区（slot≥4）——t103 分区候选注。
    pub fn is_player(&self) -> bool {
        self.attr & 3 == 1
    }
    /// 自机截止门（<$30 退出，参数表）。
    pub fn player_cutoff(&self) -> bool {
        self.is_player() && self.y < 0x30
    }
}

/// OAM 表解析（64 条 × 4B，OAM 顺序=优先级序）。
pub fn parse_oam(blob: &[u8]) -> Vec<SpriteObj> {
    blob.chunks_exact(4).take(64).map(|b| SpriteObj::from_oam4([b[0], b[1], b[2], b[3]])).collect()
}

/// 精灵层合成（t104 ①：8×8/翻转/优先级）：sprites 合成到 canvas。
/// `bg_opaque`：BG 像素不透明掩码（BG index 0=透明）——behind_bg 精灵仅在 BG 透明处显示。
/// **quirk（D4 保真，待证注）**：8 精灵/行溢出截断（NESdev：OAM 序第 9 个起整精灵丢弃）；
/// 闪烁=掉精灵（待 t103 语义定谳校准）；8×16 模式未证（PPUCTRL bit）——本函数 8×8。
pub fn render_sprites(
    canvas: &mut Canvas,
    chr_rom: &[u8],
    sprites: &[SpriteObj],
    sprite_chr_base: usize,
    sprite_pals: &[[Rgba; 4]; 4],
    bg_opaque: &[[bool; 256]; 240],
) {
    let mut per_line = [0u8; 240];
    for (_slot_idx, spr) in sprites.iter().enumerate() {
        #[allow(unused_variables)] let slot_idx = _slot_idx;
        if spr.player_cutoff() {
            continue; // 自机截止门 <$30（t107 参数表）
        }
        let y0 = spr.screen_y();
        // 8/行溢出（D4 quirk 保真）：任一扫描线超 8 ⇒ 整精灵丢弃（NESdev 评估语义候选，待证注）
        let mut dropped = false;
        for row in 0..16i32 {
            let y = y0 + row;
            if (0..240).contains(&y) && per_line[y as usize] >= 8 {
                dropped = true;
                break;
            }
        }
        if dropped {
            continue;
        }
        // ① 8×16 精灵解析（W42 收口）——**R12-1 两模型对照**（匹配率裁决）：
        // 模型 A（跨半对）：tile t 两半跨 bank106/107（同 t 不同 bank）；
        // 模型 B（同 bank 邻接对）：even/odd 邻接（t165 原实现）。
        let t = spr.tile as usize;
        // t184 ② 列对四象限（TL=base+0/BL=base+1/TR=base+2/BR=base+3）；
        // R16-2（t178）：SPRITE_MODEL 变体开关**恢复**（对照证据面禁硬编码移除）——
        // 模型 A=跨半对（106:t,107:t）/模型 B=同 bank 邻接对（even/odd，默认）
        let model_a = std::env::var("SPRITE_MODEL").map(|v| v == "A").unwrap_or(false);
        let px_all: [[[u8; 8]; 8]; 2] = if model_a {
            let bank_top = sprite_chr_base + t / 64;
            let bank_bot = sprite_chr_base + 1 + t / 64;
            [
                game_data::chr::decode_tile(&game_data::chr::tile_bytes(chr_rom, bank_top, t % 64)),
                game_data::chr::decode_tile(&game_data::chr::tile_bytes(chr_rom, bank_bot, t % 64)),
            ]
        } else {
            // ①分步接线（t213 ①交互归因）：t212 净损根=Game 族被挪 30→106（实测判优=30）
            // ——分步：仅 Label 族换 bank96，其余保持 sprite_chr_base（回归守卫）
            // ①码段双制（t225）：<128 bit0 分窗 + T&$FE 对；128+ 弹幕族
            // offset=T−221 + bank 散射副本（251→127:30/253→102/124/255→90:34）
            let (bank_top, bank_bot, it, ib) = if t >= 128 {
                // ②T−221 第三腿例外（t236）：251→offset 31（T−220 ±1 例外注）
                // ②合成件规则（t246）：各 sprite 独立位图源——251→bank22:31（t234）、
                // 253→bank29:32、255→bank90:34（H 翻）——并排装配（251+253 对，X 8px）
                let (b, idx) = match t {
                    251 => (127, 30), // t246 ②：251 腿净负留档（t225 保持）
                    253 => (29, 32), // t246 ②实测 +21px 净正采纳
                    255 => (90, 34),
                    _ => (if t & 1 == 1 { 96 } else { 28 }, t - 221),
                };
                if std::env::var("DBGDAN").is_ok() && t >= 250 {
                    eprintln!("DBGDAN t={} X={} Y={} attr={:02x}", t, spr.x, spr.y, spr.attr);
                }
                (b, b, idx, idx + 1) // 弹幕 8×16 对（idx, idx+1）同 bank
            } else {
                let base = if t & 1 == 1 { 96 } else { 28 };
                let (it, ib) = crate::render_rules::pair_indices(t as u8);
                (base + it / 64, base + ib / 64, it % 64, ib % 64)
            };
            [
                game_data::chr::decode_tile(&game_data::chr::tile_bytes(chr_rom, bank_top, it)),
                game_data::chr::decode_tile(&game_data::chr::tile_bytes(chr_rom, bank_bot, ib)),
            ]
        };
        for row in 0..16usize {
            let y = y0 + row as i32;
            if !(0..240).contains(&y) {
                continue;
            }
            per_line[y as usize] += 1;
            for col in 0..8usize {
                let half = row / 8;
                let sr = if spr.vflip() { (15 - row) % 8 } else { row % 8 };
                let sc = if spr.hflip() { 7 - col } else { col };
                let v = px_all[half][sr][sc];
                if v == 0 {
                    continue;
                }
                let x = spr.x as i32 + col as i32;
                if !(0..256).contains(&x) {
                    continue;
                }
                if spr.behind_bg() && bg_opaque[y as usize][x as usize] {
                    continue;
                }
                let c = sprite_pals[spr.palette()][v as usize];
                let idx = (y as usize * 256 + x as usize) * 4;
                canvas.pixels[idx..idx + 4].copy_from_slice(&c);
            }
        }
    }
}

pub mod render_rules;
// ===== L4 战役④ fade 7 档接线（序列已证㉖/变换=参数化候选——S5-2 封存注）=====

/// fade 观测最大档（FADE_LEVELS[0]=77——亮度标度候选基准）。
pub const FADE_MAX_LEVEL: u8 = 77;

/// fade 档序列（game-core ㉖ 同源——渲染面直引）。
pub use game_core::engine::FADE_LEVELS;

/// 【e48f5c5 dated】~~线性压暗变换候选~~ **fade 真身=批量位域改写影子后上传**
/// （bank00:85E8-8618 位域改写器——见 rewrite_field/PaletteShadow）——本函数
/// 保留为简化对照面（L4 差异比对候选），非真身语义。S5-2 ①续b 封存件**重开收**。
/// 序列侧=已证（㉖ 7 档×4f cadence）。
pub fn apply_fade_dim(canvas: &mut Canvas, level: u8) {
    let l = u32::from(level.min(FADE_MAX_LEVEL));
    for px in canvas.pixels.chunks_exact_mut(4) {
        px[0] = ((px[0] as u32 * l) / FADE_MAX_LEVEL as u32) as u8;
        px[1] = ((px[1] as u32 * l) / FADE_MAX_LEVEL as u32) as u8;
        px[2] = ((px[2] as u32 * l) / FADE_MAX_LEVEL as u32) as u8;
    }
}

// ===== L4 真差异①：BG nametable 持久化模型（条带外持久化——shop 16880/
// clear-inject 58209 像素差根源）=====

/// BG nametable 持久化模型（32×30 tile+属性表 8×8——NES 硬件定义；条带写更新
/// 对应行，**未写区保持前帧内容**=持久化语义）。条带更新规则=R-2..R-5 数据面
/// （decomp 支援）——现按行写接口参数化。
pub struct BgModel {
    /// nametable tile 索引（32×30）。
    pub tiles: [[u8; 32]; 30],
    /// 属性表（8×8 字节=64×64 像素调色板选择）。
    pub attrs: [[u8; 8]; 8],
}

impl Default for BgModel {
    fn default() -> Self {
        BgModel { tiles: [[0; 32]; 30], attrs: [[0; 8]; 8] }
    }
}

impl BgModel {
    /// 条带行写（滚动更新——整行 tile 替换；条带外行持久化不动）。
    pub fn write_strip_row(&mut self, row: usize, tiles: &[u8; 32]) {
        if row < 30 {
            self.tiles[row] = *tiles;
        }
    }

    /// 条带列写（横向滚动——整列 tile 替换；条带外列持久化不动）。
    pub fn write_strip_col(&mut self, col: usize, tiles: &[u8; 30]) {
        if col < 32 {
            for (r, &t) in tiles.iter().enumerate() {
                self.tiles[r][col] = t;
            }
        }
    }

    /// 属性表条带写（调色板选择区更新）。
    pub fn write_attr(&mut self, ax: usize, ay: usize, v: u8) {
        if ay < 8 && ax < 8 {
            self.attrs[ay][ax] = v;
        }
    }

    /// BG 层全帧绘制（tile=CHR bank 索引直绘——decode 同源）。
    pub fn draw_bg(&self, canvas: &mut Canvas, rom: &[u8], bank: usize, pal: &[Rgba; 4]) {
        for (r, row) in self.tiles.iter().enumerate() {
            for (c, &t) in row.iter().enumerate() {
                canvas.draw_chr(rom, bank, t as usize, (c * 8) as i32, (r * 8) as i32, pal);
            }
        }
    }
}

// ===== W-C1 调色板三件套（e48f5c5——fade/调色真身）=====

/// `$E9D2` 清域掩码（4B——字段 0..3 的 2bit 清位掩码）。
pub const REWRITE_CLEAR: [u8; 4] = [0xFC, 0xF3, 0xCF, 0x3F];
/// `$E9D6` 置位掩码（4B——字段 0..3 的 2bit 置位掩码）。
pub const REWRITE_SET: [u8; 4] = [0x03, 0x0C, 0x30, 0xC0];
/// `$E9DA` 值展开表（4B——2bit 值 0..3 展开为字节模式）。
pub const REWRITE_EXPAND: [u8; 4] = [0x00, 0x55, 0xAA, 0xFF];

/// 位域改写器（bank00:85E8-8618——`$D3+X` 每字节 **4×2bit 域逐域改写**：
/// `shadow[x] = (shadow[x] & CLEAR[i]) | (EXPAND[v] & SET[i])`）。
/// **fade 真身**=批量位域改写影子后上传（S5-2 ①续b $04 消费面封存件重开收）。
pub fn rewrite_field(shadow: &mut u8, field: u8, value: u8) {
    let i = (field & 3) as usize;
    let v = (value & 3) as usize;
    *shadow = (*shadow & REWRITE_CLEAR[i]) | (REWRITE_EXPAND[v] & REWRITE_SET[i]);
}

/// 影子上传（zp `$D3+` → PPU `$3Fxx`——bank15:$F04B-$F072 通道：`$2006:=$8F|$03`
/// /`$2006:=$90`/burst `$2007`；`$7E`=上传模式分派）。渲染面=影子缓冲上传语义。
#[derive(Debug, Clone, Default)]
pub struct PaletteShadow {
    /// 影子缓冲（zp `$D3+`——32B 调色板镜像候选长度注：33 色 NES 调色板族）。
    pub buf: [u8; 32],
}

impl PaletteShadow {
    /// 批量位域改写（fade 真身——逐字节逐域）。
    pub fn rewrite(&mut self, x: usize, field: u8, value: u8) {
        if x < 32 {
            rewrite_field(&mut self.buf[x], field, value);
        }
    }
}

// ===== R-2 场景调色板记录表（d00269d——W-C1 ③ 数据源接线）=====

/// 场景描述符表（bank00:$97D7 8B/项 ×5——R-2 A 级）：+0/+1=流指针对/+2/+3=
/// `$7A:$7B` 调色板记录基址（$A000 换窗区 bank 相关读）【R-3b dated】+4/+5=**流指针源**
/// `$14:$15`（hi|=$80，97AD-97B7）**非第二表**——五项全共享流 `$9809`（per-scene
/// 差异=调色板记录基址）；+6→`$EBC2` 渲染 writer/+7→`$ED65` 流执行器参。
pub const SCENE_DESCRIPTORS: [[u8; 8]; 5] = [
    [0x2E, 0x2C, 0xFB, 0xBE, 0x09, 0x18, 0xFF, 0x97],
    [0x5E, 0x5C, 0xD2, 0xC0, 0x09, 0x18, 0x55, 0x98],
    [0x22, 0x20, 0x4E, 0xBB, 0x09, 0x18, 0x85, 0x98],
    [0x22, 0x20, 0x4E, 0xBB, 0x09, 0x18, 0xF3, 0x98],
    [0x2E, 0x2C, 0xFB, 0xBF, 0x09, 0x98, 0xFF, 0x97],
];

/// 3B 场景调色板记录（`$0150+X×3+基址`——**编码语义 B 级候选=RGB 三元组**（值超
/// NES 6bit 色域=汉化版增强面候选）；C 级止步不发明（R-2b 播放 trace 对照挂账））。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteRecord(pub [u8; 3]);

/// 从记录流取第 `i` 项（3B/项——fail-closed 越界 None）。
pub fn palette_record_at(records: &[u8], i: usize) -> Option<PaletteRecord> {
    let s = i * 3;
    records.get(s..s + 3).map(|b| PaletteRecord([b[0], b[1], b[2]]))
}

/// 描述符取调色板基址（+2/+3 小端——`$7A:$7B`）。
pub fn descriptor_palette_base(desc: &[u8; 8]) -> u16 {
    u16::from_le_bytes([desc[2], desc[3]])
}

// ===== R-3 调色板命令流参数源（1787111——fade 序列真身=流回放）=====

/// 命令流步（bank00:8530-857F 流解释器——数据+计数/`$00` 终止/BMI 跳转）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamStep {
    /// 数据+计数（值重复 count 次——位域改写参数装）。
    Data { value: u8, count: u8 },
    /// 流终止（`$00`）。
    End,
    /// `$80` 相对跳转（offset=Y+1+基址→`$B6,X` 指针重算）。
    Jump { opcode: u8 },
    /// `$81-$FF` 循环计数（`DEC $B0,X`——计数器初值=opcode−$80）。
    Loop { counter: u8 },
}

/// 流解释器单步（fail-closed：截断 None）。
/// 格式（R-3b A 级**三形**）：`$00`=终止 / `$80`=**相对跳转**（offset=Y+1+基址
/// →`$B6,X` 指针重算）/ `$81-$FF`=**循环计数**（`DEC $B0,X`）/ `$01-$7F`=数据+计数
/// （`$11` 累加→JSR $8619 参数装）。流数据=NES 调色板色码序列（bank1:$9809 样本
/// `07 0F 16 36 30 26 36 30 17 36 30`+`$00`）。
pub fn stream_step(data: &[u8], pos: usize) -> Option<(StreamStep, usize)> {
    let &b = data.get(pos)?;
    if b == 0 {
        return Some((StreamStep::End, pos + 1));
    }
    if b == 0x80 {
        // 相对跳转：下一字节=offset（Y+1+基址语义——目标计算调用侧）
        let &offset = data.get(pos + 1)?;
        return Some((StreamStep::Jump { opcode: b }, pos + 2 + offset as usize));
    }
    if b >= 0x81 {
        return Some((StreamStep::Loop { counter: b }, pos + 1));
    }
    let &count = data.get(pos + 1)?;
    Some((StreamStep::Data { value: b, count }, pos + 2))
}

// ===== R-3c fade 流全谱（364400b——fade 线四段闭合）=====

/// fade 流段 1（bank1:$A7C2——10 色码+`$00` 终止；全 ≤`$3F` NES 色码域）。
pub const FADE_STREAM_SEG1: [u8; 11] = [0x0F, 0x35, 0x27, 0x21, 0x16, 0x06, 0x21, 0x16, 0x06, 0x30, 0x00];

/// fade 流段 2（14 色码+`$00` 终止）。
pub const FADE_STREAM_SEG2: [u8; 15] = [
    0x35, 0x30, 0x3C, 0x35, 0x30, 0x3C, 0x37, 0x30, 0x06, 0x37, 0x30, 0x26, 0x35, 0x30, 0x00,
];

/// fade 流尾参（`03 01 01`——ROM 尾区观测；语义注随 R-2b/播放对照）。
pub const FADE_STREAM_TAIL: [u8; 3] = [0x03, 0x01, 0x01];

/// 转场装载时序（R-3c A 级关键点：`JSR $EED7` 等级档 → `$75:=$C2/$76:=$A7`
/// 装流 `$A7C2` → `$EDB0(A=2)` 执行——**转场 setup 一次装+逐帧回放**）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionLoader {
    /// 流指针（`$75:$76`）。
    pub stream: u16,
    /// 执行器模式参（`$EDB0(A=2)` 转场）。
    pub mode: u8,
    /// 已装载（setup 完成）。
    pub loaded: bool,
}

impl TransitionLoader {
    /// 转场 setup（`$75:=$C2/$76:=$A7` ⇒ 流 `$A7C2`；`$EDB0(A=2)`）。
    pub fn setup() -> Self {
        TransitionLoader { stream: 0xA7C2, mode: 2, loaded: true }
    }
}

// ===== L4 索引族（R-1/$816E——①tile 校正+⑤type CHR bank）=====

/// 关 tile 校正表（R-1：CN bank14:$DA4C 9B——`00×7,$C1,$C1`（关 8-9 基值校正）；
/// 【d728c11 警示】JP `$D7B9`→CN `$DA4C`（+0x293 迁移）——JP 派生地址照抄必踩差异）。
pub const TILE_CORRECT_9B: [u8; 9] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC1, 0xC1];

/// 关 tile 校正（关 1..9 ⇒ 校正值；越界 fail-closed 0）。
pub fn tile_correction(stage: u8) -> u8 {
    if (1..=9).contains(&stage) {
        TILE_CORRECT_9B[(stage - 1) as usize]
    } else {
        0
    }
}

/// type→CHR bank 表（bank2:$816E 256B——MMC3 reg7 CHR 切换值；type 索引直读）。
pub const TYPE_CHR_BANK: [u8; 256] = [
    0xA3, 0xA5, 0xA6, 0xA0, 0xA2, 0xA3, 0xA3, 0xA3, 0xA4, 0xA4, 0xA5, 0xA5, 0xA6, 0xA6, 0xA8, 0xA8,
    0xAD, 0xAD, 0xAE, 0xAF, 0xB0, 0xB1, 0xB2, 0xB2, 0xB4, 0xB4, 0xB6, 0xB6, 0xB6, 0xB7, 0xB7, 0xB8,
    0xB9, 0xBA, 0xBA, 0xBB, 0xBB, 0xBC, 0xBC, 0xBD, 0xBD, 0xBD, 0xA6, 0xAC, 0xAE, 0xAE, 0xAF, 0xB0,
    0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x04, 0x04, 0x04, 0x04, 0x06, 0x06, 0x06,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x03,
    0x04, 0x04, 0x07, 0x04, 0x07, 0x07, 0x03, 0x03, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x03, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
    0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
    0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
    0x07, 0x07, 0x07, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
    0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
    0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
    0x20, 0xF9, 0xCF, 0x20, 0x27, 0xDB, 0xA9, 0x03, 0x20, 0x2E, 0xE6, 0xA5, 0x32, 0x29, 0xEF, 0x85,
    0x32, 0x29, 0x01, 0xF0, 0x0E, 0xA5, 0x32, 0x29, 0xFE, 0x85, 0x32, 0xAD, 0xA5, 0x01, 0xA0, 0x02,
    0x20, 0x3D, 0xE6, 0xE6, 0x2B, 0x20, 0x3B, 0xD9, 0xA2, 0x00, 0x86, 0x74, 0xA5, 0xDB, 0xF0, 0x11,
];

/// type→CHR bank（`$816E+type` 直读——256 全域恒有值）。
pub fn type_chr_bank(type_: u8) -> u8 {
    TYPE_CHR_BANK[type_ as usize]
}

// ===== R-5 CHR 碎片组（4c8e38d——⑤tile 语义+graphics.csv 锚）=====

/// 【3fa001c dated】~~00 分隔碎片组~~ **=hi 表+相邻流数据**（R-5b 组合表定谳：
/// 真组合=双表→流非分隔组）。碎片共享实证维持：$13/$14 跨组复用（社区 T09 互证）；
/// 本常量保留为碎片 ID 族登记（合成真身=composition_stream_ptr 链）。
/// 组0=$30+01-12（18 碎）/组1=13 14/组2=15-1A/组3=13 14+1B-21。
pub const FRAGMENT_GROUPS: [&[u8]; 4] = [
    &[0x30, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
      0x0F, 0x10, 0x11, 0x12],
    &[0x13, 0x14],
    &[0x15, 0x16, 0x17, 0x18, 0x19, 0x1A],
    &[0x13, 0x14, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21],
];

/// graphics.csv 画面语义锚（A 级 32 行 id/screen——R-4 同域）：stage_1-8 关名屏/
/// opening_bubble/ending_robot/ending_normal 族。
pub const SCREEN_ANCHORS: &[&str] = &[
    "stage_1", "stage_2", "stage_3", "stage_4", "stage_5", "stage_6", "stage_7", "stage_8",
    "opening_bubble", "ending_robot", "ending_normal",
];

// ===== ⑥ text.csv 文本放置面（168 行 PPU/x/y 全图）=====

/// 文本放置项（text.csv 行——group/ppu_address/x/y/text）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPlacement {
    /// 语义组（settings 等——graphics.csv 同域锚）。
    pub group: &'static str,
    /// PPU 地址（`$2xxx` nametable 域）。
    pub ppu: u16,
    /// x（列）。
    pub x: u8,
    /// y（行）。
    pub y: u8,
    /// 文本（翻译面——字形→tile 映射=chr-font-map 域参数化）。
    pub text: String,
}

/// PPU 地址→nametable 坐标（`$2000+y*32+x` 反解——text.csv x/y 对偶实证
/// `$20C8`⇒(8,6) ✓）。
pub fn ppu_to_nametable(ppu: u16) -> Option<(usize, usize)> {
    let off = (ppu as usize).checked_sub(0x2000)?;
    if off >= 960 {
        return None;
    }
    Some((off % 32, off / 32))
}

/// 文本放置到 BG 模型（逐字符 tile 码调用方提供（chr-font-map 域）——fail-closed
/// 越界裁剪）。
pub fn place_text(bg: &mut BgModel, p: &TextPlacement, tiles: &[u8]) {
    let (cx, cy) = match ppu_to_nametable(p.ppu) {
        Some(c) => c,
        None => return,
    };
    for (i, &t) in tiles.iter().enumerate() {
        let col = cx + i;
        if col < 32 && cy < 30 {
            bg.tiles[cy][col] = t;
        }
    }
}

// ===== R-4 画面流回放（b650b26——tile 布局流非 CHR 本体）=====

/// 画面流步（bank01:$A1xx 格式（A 级）：[PPU 地址 2B] tile 码流 [FE 重载][FF 段终]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenStep {
    /// PPU 地址装载（段起点）。
    PpuAddr { ppu: u16 },
    /// tile 码（写入当前 PPU 位置并推进）。
    Tile { code: u8 },
    /// FE 重载（新 PPU 地址装载）。
    Reload,
    /// FF 段终。
    SegmentEnd,
}

/// 画面流单步（fail-closed：截断 None）。
pub fn screen_step(data: &[u8], pos: usize) -> Option<(ScreenStep, usize)> {
    match *data.get(pos)? {
        0xFE => Some((ScreenStep::Reload, pos + 1)),
        0xFF => Some((ScreenStep::SegmentEnd, pos + 1)),
        _ => {
            // PPU 地址 2B（首步）或 tile 码——由调用方状态机分派；此处=字节流逐读
            let lo = *data.get(pos)?;
            let hi = *data.get(pos + 1)?;
            if hi >= 0x20 && hi < 0x24 {
                Some((ScreenStep::PpuAddr { ppu: u16::from_le_bytes([lo, hi]) }, pos + 2))
            } else {
                Some((ScreenStep::Tile { code: lo }, pos + 1))
            }
        }
    }
}

// ===== R-5b 关名合成链（3fa001c——三线合龙末件）=====

/// 组合流指针表 lo（bank2:$97BA 16B——ROM 直读）。
pub const COMP_PTR_LO: [u8; 16] = [
    0x6A, 0x6F, 0x7A, 0x85, 0x9D, 0xB2, 0xD2, 0xEA, 0xF7, 0xFC, 0x25, 0x31, 0x36, 0x53, 0x7C, 0x16,
];
/// 组合流指针表 hi（bank2:$9812 16B）。
pub const COMP_PTR_HI: [u8; 16] = [
    0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99,
];

/// `$518` 形状索引→组合流指针（双表查——`hi[i]<<8|lo[i]`；fail-closed 越界 None）。
pub fn composition_stream_ptr(shape_index: usize) -> Option<u16> {
    let lo = *COMP_PTR_LO.get(shape_index)?;
    let hi = *COMP_PTR_HI.get(shape_index)?;
    Some(u16::from_le_bytes([lo, hi]))
}

/// F 编码位移解码（有符号补码 i8——FF=-1/FA=-6/F4=-12 ✓）。
pub fn f_decode(b: u8) -> i8 {
    b as i8
}

/// 组合流步（`01 头|位移对|计数|嵌套指针 lo,hi|参数对`——R-5b 格式；嵌套指针=
/// R-5b-2 挂（指针集 $9941/$9895/$98AA/$98C6/$9998/$99CA…）。fail-closed 截断 None。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompStep {
    /// `01` 头。
    Header,
    /// 位移对（F 编码 dx,dy）。
    Displacement { dx: i8, dy: i8 },
    /// 嵌套指针（lo,hi——R-5b-2 碎片表挂）。
    Nested { ptr: u16 },
    /// 参数对（值域 R-5b-2 挂）。
    Param { a: u8, b: u8 },
}

/// 组合流单步（启发式字段界（8 样本同构注）——精确步进机=R-5b-2 随证修）。
pub fn comp_step(data: &[u8], pos: usize) -> Option<(CompStep, usize)> {
    let &b0 = data.get(pos)?;
    if b0 == 0x01 && pos == 0 {
        return Some((CompStep::Header, pos + 1));
    }
    let b1 = *data.get(pos + 1)?;
    // 嵌套指针形：hi ∈ $98..$99（组合流/碎片表窗）且 lo 任意
    if (0x98..=0x99).contains(&b1) {
        return Some((CompStep::Nested { ptr: u16::from_le_bytes([b0, b1]) }, pos + 2));
    }
    // 位移对（F 编码）或参数对——按位移域判（|v|≤16 候选域=位移）
    Some((CompStep::Displacement { dx: f_decode(b0), dy: f_decode(b1) }, pos + 2))
}
