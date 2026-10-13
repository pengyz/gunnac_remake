//! render_frame —— L4 对拍用 Rust 侧帧导出（t93：nametable 持久化模型 → BG 渲染）。
//! 用法：render-frame <帧号> <输出名> [场景=gameplay-stage1] [--base <0|1>]
//!
//! 模型（bank15:$F040-F0DA 定谳）：逐帧 nmi_flush（$130-$14F 行缓冲 → $8F:$8E 行、
//! $D3-$DA → 属性行、$160=挂起计数）累积 nametable；目标帧按 $1C/$1D 卷轴渲染 32×30。
//! BG 图案表基址（PPUCTRL bit4）未证——--base 0|1 双候选由 L4 对拍裁定（③）。

use game_core::terrain::Nametable;
use game_desktop::{Canvas, SCREEN_H, SCREEN_W};
use std::io::Write;

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let frame: u64 = args.get(1).expect("帧号").parse().expect("帧号");
    let name = args.get(2).expect("输出名").clone();
    let scene = args.get(3).cloned().unwrap_or_else(|| "gameplay-stage1".into());
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
    let mut ptr_writes: std::collections::HashMap<u64, (u8, u8)> = std::collections::HashMap::new();
    // 归属轴（AR3-A 修正）：按段起点址匹配 t130 §2 七段（$2006 对计数段序号）——
    // 此前误把 JSONL inv 行号当段序号（calls 11-18 全被滤掉=假阳性源）。
    // 七段起点（t130 §2 表）：$2170/$21A2/$2289/$22B7/$21B8/$2205/$2252
    let seg_starts: [u16; 7] = [0x2170, 0x21A2, 0x2289, 0x22B7, 0x21B8, 0x2205, 0x2252];
    let raw_writes: Vec<(u32, u16, u8, u32, u32)> = std::fs::read_to_string("analysis/phase/gameplay-stage1-text.jsonl")
        .map(|t| t.lines().filter_map(|l| {
            let f = l.split("\"f\":").nth(1)?.split(',').next()?.parse().ok()?;
            let a = u16::from_str_radix(l.split("\"addr\":\"").nth(1)?.split('"').next()?, 16).ok()?;
            let v = l.split("\"val\":").nth(1)?.split(&[',', '}'][..]).next()?.parse().ok()?;
            let inv = l.split("\"inv\":").nth(1)?.split(&[',', '}'][..]).next()?.parse().ok()?;
            let seg = l.split("\"seg\":").nth(1)?.split(&[',', '}'][..]).next()?.parse().ok()?;
            Some((f, a, v, inv, seg))
        }).collect())
        .unwrap_or_default();
    // 按 (inv,seg) 分组求段起点 → 过滤七段 → 组内按写序展开（后写胜自然成立）
    let mut groups: std::collections::BTreeMap<(u32, u32), Vec<(u32, u16, u8)>> = std::collections::BTreeMap::new();
    for (f, a, v, inv, seg) in &raw_writes {
        groups.entry((*inv, *seg)).or_default().push((*f, *a, *v));
    }
    // ③ 滤表钉死最后写者（R3-3）：同起点只保留最后 (inv,seg) 组（真覆盖族同起点不误归属）
    let mut chosen: std::collections::HashMap<u16, ((u32, u32), Vec<(u32, u16, u8)>)> = std::collections::HashMap::new();
    for (key, ws) in &groups {
        let mut hi = None;
        let mut start = None;
        for (_f, a, v) in ws {
            if *a == 0x2006 {
                if hi.is_none() {
                    hi = Some(*v);
                } else {
                    start = Some((hi.unwrap() as u16) << 8 | *v as u16);
                    break;
                }
            }
        }
        if let Some(st) = start {
            if seg_starts.contains(&st) {
                let e = chosen.entry(st).or_insert((*key, Vec::new()));
                // R3-3 修正：按**最后写时刻**（组末帧）钉死，非 (inv,seg) 字典序
                let t_new = ws.iter().map(|w| w.0).max().unwrap_or(0);
                let t_old = e.1.iter().map(|w| w.0).max().unwrap_or(0);
                if (t_new, *key) >= (t_old, e.0) {
                    e.0 = *key;
                    e.1 = ws.iter().map(|(f, a, v)| (*f, *a, *v)).collect();
                }
            }
        }
    }
    let mut text_writes: Vec<(u32, u16, u8, u32)> = Vec::new(); // 带组序号（消费序分组）
    for (_st, (key, ws)) in &chosen {
        for (f, a, v) in ws {
            text_writes.push((*f, *a, *v, (key.0 << 16) | key.1)); // 组标识
        }
    }
    text_writes.sort_by_key(|w| w.0);
    let text_writes = text_writes;
    let mut text_i = 0usize;
    let mut last_grp: u32 = u32::MAX;
    let mut ppu_addr: u16 = 0;
    let mut ppu_addr_latch_high = true;
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
        // t143 对账模式：行 LUT 路径暂缓（文本层 220 真值对账）
        let _ = pending_row.take();
        let zi = line.find("\"zp\":\"").unwrap();
        let zp = hex_decode(&line[zi + 6..][..line[zi + 6..].find('"').unwrap()]);
        // —— 主循环相位：probe 写事件填充缓冲 ——
        apply_frame_writes(&mut buf, &mut attrs, &mut addr_regs, &mut palette_src, f, &mut probe_i);
        // 帧末挂起：行缓冲若本帧被写过（扫写）→ 排队下帧 NMI 上传（地址取当帧 zp）
        let wrote = probe.iter().any(|(pf, pa, _)| *pf == f && (0x0130..=0x0150).contains(pa));
        // —— 文本放置（t129 机制终版）：核回放写集（phase-batch 产物）直接落 nametable——
        // 77+46 变长足迹表由 ROM 链原生产出（与 t128 A 级裁定逐址一致）——
        // PPU 协议解释（t131 归属合并击根修）：$2006 写锁存地址（高/低），
        // $2007 写落 tile（地址自增）——注册器写序列 → nametable 态
        while text_i < text_writes.len() && text_writes[text_i].0 <= f as u32 {
            let (_f, addr, val, grp) = text_writes[text_i];
            if grp != last_grp {
                ppu_addr_latch_high = true; // R3-1：每组新鲜锁存（修相位错位）
                last_grp = grp;
            }
            text_i += 1;
            match addr {
                0x2006 => {
                    if ppu_addr_latch_high {
                        ppu_addr = ((val as u16) << 8) | (ppu_addr & 0x00FF);
                    } else {
                        ppu_addr = (ppu_addr & 0xFF00) | val as u16;
                    }
                    ppu_addr_latch_high = !ppu_addr_latch_high;
                }
                0x2007 => {
                    if (0x2000..0x3000).contains(&ppu_addr) {
                        let rel = (ppu_addr - 0x2000) as usize;
                        let (n, off) = (rel / 0x400, rel % 0x400);
                        if n < 4 && off < 0x3C0 {
                            nt.tiles[n][off] = val;
                        }
                    }
                    ppu_addr = ppu_addr.wrapping_add(1);
                }
                _ => {}
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
            let raw_tile = nt.tiles[ntr][ty * 32 + tx];
            // ② 小值参数族分离（t152）：0-16 簇=参数字节非 tile（AR2-D bit5 同族）——
            // 直拷制下渲染按透明（不画 glyph）；命名值仍保留在 nametable（内容层精确不动）
            if raw_tile <= 16 || raw_tile >= 128 {
                continue; // 参数族（0-16 簇）+ 高值族（≥128）非 glyph tile（t152 实测）
            }
            let tile = raw_tile as usize + base * 256;
            let px = (tx as i32) * 8 - sx as i32;
            let py = (ty as i32) * 8 - sy as i32; // t131 实测：+8 更劣（94.17<94.50）——偏移非主因
            let bank = font_bank + (tile / 64) * 2; // t143/t152 实测最优 // 2KB 对：R0=124/125（值 0-127），R1=126/127（128-255） // t143 实测最优（92.20%）——W45 映射=码表转换待证（见账）
            if bank < 128 {
                // 5 色绑定合流：BG 文本=白（绑定定谳）；index 0=黑（backdrop）
                let bg_pal: [game_desktop::Rgba; 4] = [
                    [0, 0, 0, 255],
                    [0xFF, 0xFE, 0xFF, 255],
                    [0xFF, 0xFE, 0xFF, 255],
                    [0xFF, 0xFE, 0xFF, 255],
                ];
                canvas.draw_chr(&rom, bank, tile % 64, px, py, &bg_pal);
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
