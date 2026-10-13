//! render_frame —— L4 对拍用 Rust 侧帧导出（t93：nametable 持久化模型 → BG 渲染）。
//! 用法：render-frame <帧号> <输出名> [场景=gameplay-stage1] [--base <0|1>]
//!
//! 模型（bank15:$F040-F0DA 定谳）：逐帧 nmi_flush（$130-$14F 行缓冲 → $8F:$8E 行、
//! $D3-$DA → 属性行、$160=挂起计数）累积 nametable；目标帧按 $1C/$1D 卷轴渲染 32×30。
//! BG 图案表基址（PPUCTRL bit4）未证——--base 0|1 双候选由 L4 对拍裁定（③）。

use game_core::terrain::Nametable;
use game_desktop::{Canvas, GREY4, SCREEN_H, SCREEN_W};
use std::io::Write;

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let frame: u64 = args.get(1).expect("帧号").parse().expect("帧号");
    let name = args.get(2).expect("输出名").clone();
    let scene = args.get(3).cloned().unwrap_or_else(|| "gameplay-stage1".into());
    let tbank: usize = std::env::var("TEXT_BANK").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let tb_early: usize = std::env::var("TEXT_BANK_EARLY").ok().and_then(|v| v.parse().ok()).unwrap_or(tbank);
    let tb_late: usize = std::env::var("TEXT_BANK_LATE").ok().and_then(|v| v.parse().ok()).unwrap_or(tbank);
    let tb_phase: u64 = std::env::var("TEXT_PHASE").ok().and_then(|v| v.parse().ok()).unwrap_or(9999);
    let base: usize = args.iter().position(|a| a == "--base")
        .and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(0);
    let rom = std::fs::read("rom/gunnac_cn.nes").expect("ROM");
    let trace = std::fs::read_to_string(format!("analysis/traces/{scene}.frames.jsonl")).expect("trace");
    // NMI 级重建（R-l4-linebuf 五不变量，t94/t96）：
    // 帧 N 主循环扫写填充行缓冲（probe 写事件）→ 帧 N+1 vblank NMI 读走（读后即废）→
    // 写入 nametable（地址 = 帧 N zp $8E/$8F）；$160=挂起类型，上传后清零（模型侧等效）。
    let mut nt = Nametable::default();
    let mut buf = [0u8; 0x31]; // $0130-$015F + $0160
    let mut attrs = [0u8; 8];
    let mut addr_regs = (0u8, 0u8, 0u8); // $8E/$8F/$90（t99：probe 写点追踪，解相位陷阱）
    let mut palette_src = [0u8; 7]; // $161-$167 调色板源（t99 ①）
    let mut pending_row: Option<([u8; 32], [u8; 8], u8, u8, u8)> = None;
    let mut hud_row_seq = 0usize; // ②基址上下文：预装载非零行按填充序 → $2000 系 LUT 行
    let mut sx = 0u8;
    let mut sy = 0u8;
    let mut zp_prev: Vec<u8> = vec![0; 256];
    // probe 写事件（NMI 级相位）
    let probe_path = format!("analysis/traces/{scene}.probe.jsonl");
    let probe: Vec<(u64, u16, u8)> = std::fs::read_to_string(&probe_path).map(|t| {
        t.lines().filter_map(|l| {
            let f = l.split("\"f\":").nth(1)?.split(',').next()?.parse().ok()?;
            let a = l.split("\"a\":\"").nth(1)?;
            let a = u16::from_str_radix(a.split('"').next()?, 16).ok()?;
            let v = l.split("\"v\":").nth(1)?.split(&[',', '}'][..]).next()?.parse().ok()?;
            Some((f, a, v))
        }).collect()
    }).unwrap_or_default();
    let mut probe_i = 0usize;
    let mut iram: Vec<u8> = Vec::new();
    let mut ptr_writes: std::collections::HashMap<u64, (u8, u8)> = std::collections::HashMap::new();
    for (pf, pa, pv) in &probe {
        match pa {
            0x2F => {
                let e = ptr_writes.entry(*pf).or_insert((0, 0));
                e.0 = *pv;
            }
            0x30 => {
                let e = ptr_writes.entry(*pf).or_insert((0, 0));
                e.1 = *pv;
            }
            _ => {}
        }
    }
    let mut trace_oam: Option<String> = None;
    let apply_frame_writes = |buf: &mut [u8; 0x31], attrs: &mut [u8; 8],
                                  regs: &mut (u8, u8, u8), pal: &mut [u8; 7],
                                  f: u64, pi: &mut usize| {
        while *pi < probe.len() && probe[*pi].0 == f {
            let (_f, a, v) = probe[*pi];
            *pi += 1;
            match a {
                0x0130..=0x0160 => buf[(a - 0x0130) as usize] = v,
                0x00D3..=0x00DA => attrs[(a - 0x00D3) as usize] = v,
                0x8E => regs.0 = v,
                0x8F => regs.1 = v,
                0x90 => regs.2 = v,
                0x8D => {
                    // t100 行地址 LUT 通道：$8D 行索引 → $EBF4 项 = 行基址（$2000+$40n）
                    let lut = [
                        0x2000u16, 0x2040, 0x2080, 0x20C0, 0x2100, 0x2140, 0x2180, 0x21C0,
                        0x2200, 0x2240, 0x2280, 0x22C0, 0x2300, 0x2340, 0x2380, 0x2CA5,
                    ];
                    if let Some(&base) = lut.get(v as usize) {
                        regs.0 = base as u8;
                        regs.1 = (base >> 8) as u8;
                    }
                }
                0x0161..=0x0167 => pal[(a - 0x0161) as usize] = v,
                _ => {}
            }
        }
    };
    for line in trace.lines() {
        let Some(rest) = line.split("\"f\":").nth(1) else { continue };
        let f: u64 = rest.split(',').next().unwrap().parse().unwrap();
        if f > frame { break; }
        // —— NMI 语义边界（帧首）：上传上一帧填充的行缓冲（读后即废）——
        if let Some((row, at, lo, hi, al)) = pending_row.take() {
            nt.nmi_flush(&row, &at, lo, hi, al, 1);
        }
        if let Some(i) = line.find("\"iram\":\"") {
            let rest = &line[i + 8..];
            iram = hex_decode(&rest[..rest.find('"').unwrap()]);
        }
        let zi = line.find("\"zp\":\"").unwrap();
        let zp = hex_decode(&line[zi + 6..][..line[zi + 6..].find('"').unwrap()]);
        // —— 主循环相位：probe 写事件填充缓冲 ——
        apply_frame_writes(&mut buf, &mut attrs, &mut addr_regs, &mut palette_src, f, &mut probe_i);
        // 帧末挂起：行缓冲若本帧被写过（扫写）→ 排队下帧 NMI 上传（地址取当帧 zp）
        let wrote = probe.iter().any(|(pf, pa, _)| *pf == f && (0x0130..=0x0150).contains(pa));
        // —— 文本通道（t118 ①②③四段链复刻）：$2F/$30 流指针（RAM 中转 W50 实证）→
        // iram 流字节 → 地址合成三类 → nametable 直写（$E6DF/$E828/$E7DC 语义）——
        if let Some(&(slo, hi)) = ptr_writes.get(&f) {
            let mut ch = game_core::terrain::TextChannel {
                stream_lo: slo, stream_hi: hi,
                addr_lo: zp[0x14], addr_hi: zp[0x15], count: 0, tier7e: false,
            };
            for _ in 0..64 {
                let sp = (ch.stream_hi as u16) << 8 | ch.stream_lo as u16;
                let b = if sp >= 0x8000 {
                    // ② 文本流 bank 绑定（R7 语义）：ROM 直指流按 TEXT_BANK 变体读
                    let off = sp.wrapping_sub(0xA000) as usize;
                    let bk = if f < tb_phase { tb_early } else { tb_late };
                    *rom.get(0x10 + bk * 0x2000 + off).unwrap_or(&0xFE)
                } else {
                    let sa = sp.wrapping_sub(0x0300) as usize;
                    match iram.get(sa) { Some(&v) => v, None => break }
                };
                if let Some((addr, tile)) = game_core::terrain::text_channel_byte(&mut ch, b) {
                    if (0x2000..0x3000).contains(&addr) {
                        let rel = (addr - 0x2000) as usize;
                        let (n, off) = (rel / 0x400, rel % 0x400);
                        if n < 4 && off < 0x3C0 {
                            nt.tiles[n][off] = tile;
                        }
                    }
                }
                if b >= 0xFE { break; }
            }
        }
        if wrote {
            let mut row = [0u8; 32];
            row.copy_from_slice(&buf[0x00..0x20]);
            let addr_ok = ((addr_regs.1 as u16) << 8 | addr_regs.0 as u16) >= 0x2000;
            if !addr_ok && row.iter().any(|&v| v != 0) {
                // ① 放置序=**降序+双写**（t109 参数表：行索引逆序映射、每行双调用——
                // 顺序填充实验反向即本因）：fills/2 → LUT 行 15 递降
                let lut = [0x2000u16, 0x2040, 0x2080, 0x20C0, 0x2100, 0x2140, 0x2180, 0x21C0,
                           0x2200, 0x2240, 0x2280, 0x22C0, 0x2300, 0x2340, 0x2380, 0x2CA5];
                let idx = 15usize.saturating_sub(hud_row_seq / 2);
                let base = lut[idx];
                pending_row = Some((row, attrs, base as u8, (base >> 8) as u8, 0xC0));
                hud_row_seq += 1;
            } else {
                pending_row = Some((row, attrs, addr_regs.0, addr_regs.1, addr_regs.2));
            }
        }
        let _ = &zp_prev;
        zp_prev = zp.clone();
        if f == frame.wrapping_sub(1) {
            if let Some(i) = line.find("\"oam\":\"") {
                let rest = &line[i + 7..];
                trace_oam = Some(rest[..rest.find('"').unwrap()].to_string());
            }
        }
        if f == frame {
            sx = zp[0x1C];
            sy = zp[0x1D];
        }
    }
    // 渲染：nametable 32×30 + 卷轴
    let mut canvas = Canvas::new();
    let font_bank = game_data::chr::FONT_BANK_START;
    let ntr: usize = std::env::var("RENDER_NT").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    for ty in 0..30usize {
        for tx in 0..32usize {
            let tile = nt.tiles[ntr][ty * 32 + tx] as usize + base * 256;
            let px = (tx as i32) * 8 - sx as i32;
            let py = (ty as i32) * 8 - sy as i32;
            let bank = font_bank + tile / 64;
            if bank < 128 {
                canvas.draw_chr(&rom, bank, tile % 64, px, py, &GREY4);
            }
        }
    }
    // 索引侧车：逐像素 (调色板号, 像素值) + 调色板源（t99 ①反推输入）
    let mut idx_rows: Vec<String> = Vec::new();
    for ty in 0..30usize {
        for tx in 0..32usize {
            let pal = nt.block_palette(ntr, tx, ty);
            let tile = nt.tiles[ntr][ty * 32 + tx];
            idx_rows.push(format!("{}:{}:{}", tx, ty, pal));
            let _ = tile;
        }
    }
    // —— 精灵层（t104 ①）：OAM blob → 合成（BG 掩码 + 优先级 + 翻转 + 8/行 quirk）——
    let oam_blob: Vec<u8> = trace_oam.as_ref().map(|h| hex_decode(h)).unwrap_or_default();
    let mut sprites = game_desktop::parse_oam(&oam_blob);
    // ③ 伴随精灵条偏移（W47 实测：槽 29-35 = 7 件横排，Y+127/X+8×7 锁定玩家坐标系）
    for (k, sp) in sprites.iter_mut().enumerate().skip(29).take(7) {
        sp.y = sp.y.wrapping_add(127);
        sp.x = sp.x.wrapping_add(8 * (k as u8 - 29));
    }
    // 绑定表（t102/t104 ③像素级证明产物）：sp<pal][idx] → RGB
    let mk = |r: u8, g: u8, b: u8| -> game_desktop::Rgba {
        [r, g, b, if r == 0 && g == 0 && b == 0 { 0 } else { 255 }]
    };
    let unk = mk(0, 0, 0);
    let sprite_pals: [[game_desktop::Rgba; 4]; 4] = [
        [unk, mk(0xFF, 0xFE, 0xFF), unk, mk(0x00, 0x40, 0x4D)],
        [unk, mk(0xFF, 0xFE, 0xFF), unk, mk(0xFF, 0xFE, 0xFF)],
        [unk, unk, unk, mk(0xBC, 0xBE, 0x00)],
        [unk, unk, unk, unk],
    ];
    let bg_opaque = [[false; 256]; 240]; // BG 面重建为空（黑场）——掩码全透明
    game_desktop::render_sprites(&mut canvas, &rom, &sprites, 124, &sprite_pals, &bg_opaque);

    std::fs::create_dir_all("oracle/screens/actual").unwrap();
    let out_path = format!("oracle/screens/actual/{name}.png");
    canvas.export_png(std::path::Path::new(&out_path)).expect("导出");
    let idx_path = format!("oracle/screens/actual/{name}.idx.json");
    std::fs::write(&idx_path, format!(
        "{{\"palette_src\":\"{}\",\"attrs\":\"{}\",\"tiles\":\"{}\"}}",
        palette_src.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        nt.attrs[ntr].iter().map(|b| format!("{b:02x}")).collect::<String>(),
        nt.tiles[ntr].iter().map(|b| format!("{b:02x}")).collect::<String>(),
    )).expect("侧车");
    let _ = idx_rows;
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open("oracle/screens/actual/.render.log").unwrap();
    writeln!(log, "{name} frame={frame} base={base} scroll=({sx},{sy}) nt_nz={} probe={}", nt.tiles.iter().flat_map(|t| t.iter()).filter(|&&t| t != 0).count(), probe.len()).unwrap();
    let _ = (SCREEN_W, SCREEN_H);
}
