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
    let base: usize = args.iter().position(|a| a == "--base")
        .and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(0);
    let rom = std::fs::read("rom/gunnac_cn.nes").expect("ROM");
    let trace = std::fs::read_to_string(format!("analysis/traces/{scene}.frames.jsonl")).expect("trace");
    let mut nt = Nametable::default();
    let (mut sx, mut sy) = (0u8, 0u8);
    for line in trace.lines() {
        let Some(rest) = line.split("\"f\":").nth(1) else { continue };
        let f: u64 = rest.split(',').next().unwrap().parse().unwrap();
        if f > frame { break; }
        let zi = line.find("\"zp\":\"").unwrap();
        let zp = hex_decode(&line[zi + 6..][..line[zi + 6..].find('"').unwrap()]);
        let bi = line.find("\"bg_nmi\":\"");
        let bg = bi.map(|i| hex_decode(&line[i + 10..][..line[i + 10..].find('"').unwrap()]));
        if let Some(bg) = bg {
            if bg.len() >= 0x31 && bg[0x30] != 0 && f == 1193 {
                eprintln!("DBG f1193 bg.len={} p30={} row[0..8]={:02x?} zp8E={:02x} zp8F={:02x} zp90={:02x}", bg.len(), bg[0x30], &bg[0..8], zp[0x8E], zp[0x8F], zp[0x90]);
            }
            if bg.len() >= 0x31 && bg[0x30] != 0 {
                let mut row = [0u8; 32];
                row.copy_from_slice(&bg[0x00..0x20]);
                let mut attrs = [0u8; 8];
                attrs.copy_from_slice(&bg[0x31..0x39].get(0..8).unwrap_or(&[0; 8]).try_into().unwrap_or([0; 8]));
                nt.nmi_flush(&row, &attrs, zp[0x8E], zp[0x8F], zp[0x90], bg[0x30]);
            }
        }
        if f == frame {
            sx = zp[0x1C];
            sy = zp[0x1D];
        }
    }
    // 渲染：nametable 32×30 tile + 卷轴偏移（mod 256 环绕）
    let mut canvas = Canvas::new();
    let font_bank = game_data::chr::FONT_BANK_START;
    for ty in 0..30usize {
        for tx in 0..32usize {
            let tile = nt.tiles[ty * 32 + tx] as usize + base * 256;
            let px = (tx as i32) * 8 - sx as i32;
            let py = (ty as i32) * 8 - sy as i32;
            // CHR tile：bank = tile/64（字模区起），tile%64
            let bank = font_bank + tile / 64;
            if bank < 128 {
                canvas.draw_chr(&rom, bank, tile % 64, px, py, &GREY4);
            }
        }
    }
    std::fs::create_dir_all("oracle/screens/actual").unwrap();
    let out_path = format!("oracle/screens/actual/{name}.png");
    canvas.export_png(std::path::Path::new(&out_path)).expect("导出");
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open("oracle/screens/actual/.render.log").unwrap();
    writeln!(log, "{name} frame={frame} base={base} scroll=({sx},{sy}) nametable_nonzero={}", nt.tiles.iter().filter(|&&t| t != 0).count()).unwrap();
    let _ = (SCREEN_W, SCREEN_H);
}
