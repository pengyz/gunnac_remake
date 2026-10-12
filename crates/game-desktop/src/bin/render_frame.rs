//! render_frame —— L4 对拍用 Rust 侧帧导出（V3，t92）：从 oracle trace 帧实测状态渲染
//! BG 投影帧（256×240，灰阶口径）→ actual/<名>.png。
//! 用法：render-frame <帧号> <输出名> [场景=gameplay-stage1]

use game_desktop::{render_bg_projection, Canvas};
use std::io::Write;

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let frame: u64 = args.get(1).expect("帧号").parse().expect("帧号数值");
    let name = args.get(2).expect("输出名").clone();
    let scene = args.get(3).cloned().unwrap_or_else(|| "gameplay-stage1".into());
    let rom = std::fs::read("rom/gunnac_cn.nes").expect("ROM");
    let trace = std::fs::read_to_string(format!("analysis/traces/{scene}.frames.jsonl")).expect("trace");
    let mut found = None;
    for line in trace.lines() {
        if line.contains(&format!("\"f\":{frame},")) {
            found = Some(line.to_string());
            break;
        }
    }
    let row = found.expect("帧行");
    let zi = row.find("\"zp\":\"").expect("zp");
    let zp_hex = &row[zi + 6..];
    let zp = hex_decode(&zp_hex[..zp_hex.find('"').unwrap()]);
    let (sx, sy) = (zp[0x1C], zp[0x1D]);
    let ptr = u16::from_le_bytes([zp[0x7A], zp[0x7B]]);
    let bank8 = &rom[0x10 + 8 * 0x2000..0x10 + 9 * 0x2000];
    let off = ptr.wrapping_sub(0xA000) as usize;
    let pattern: &[u8] = bank8.get(off..).unwrap_or(&[]);
    let (mut b130, mut b131, mut b140, mut b141) = ([0u8; 16], [0u8; 16], [0u8; 16], [0u8; 16]);
    for i in 0..16usize {
        let r = game_core::terrain::bg_record_at(pattern, i as u8);
        b130[i] = r.f1;
        b131[i] = r.f2;
        b140[i] = r.f3;
        b141[i] = r.f4;
    }
    let mut canvas = Canvas::new();
    render_bg_projection(&mut canvas, &rom, &b130, &b131, &b140, &b141, sx, sy);
    std::fs::create_dir_all("oracle/screens/actual").unwrap();
    let out_path = format!("oracle/screens/actual/{name}.png");
    let out = std::path::Path::new(&out_path);
    canvas.export_png(out).expect("导出");
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open("oracle/screens/actual/.render.log").unwrap();
    writeln!(log, "{name} frame={frame} ptr={ptr:#06x} scroll=({sx},{sy}) pattern_len={}", pattern.len()).unwrap();
}
