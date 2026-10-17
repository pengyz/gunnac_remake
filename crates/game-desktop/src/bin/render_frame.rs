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

/// NES 2C02 调色板 RGB（64 色标准表——EC46 色码域映射）。
fn nes_rgb(code: u8) -> game_desktop::Rgba {
    // 标准 NES 调色板（简化 2C02——逐色码 RGB）
    const P: [[u8; 3]; 64] = [
        [0x66,0x66,0x66],[0x00,0x2A,0x88],[0x14,0x12,0xA7],[0x3B,0x00,0xA4],[0x5C,0x00,0x7E],[0x6E,0x00,0x40],[0x6C,0x06,0x00],[0x56,0x1D,0x00],
        [0x33,0x35,0x00],[0x0B,0x48,0x00],[0x00,0x52,0x00],[0x00,0x4F,0x08],[0x00,0x40,0x4D],[0x00,0x00,0x00],[0x00,0x00,0x00],[0x00,0x00,0x00],
        [0xAD,0xAD,0xAD],[0x15,0x5F,0xD9],[0x42,0x40,0xFF],[0x75,0x27,0xFE],[0xA0,0x1A,0xCC],[0xB7,0x1E,0x7B],[0xB5,0x31,0x20],[0x99,0x4E,0x00],
        [0x6B,0x6D,0x00],[0x38,0x87,0x00],[0x0C,0x93,0x00],[0x00,0x8F,0x32],[0x00,0x7C,0x8D],[0x00,0x00,0x00],[0x00,0x00,0x00],[0x00,0x00,0x00],
        [0xFF,0xFE,0xFF],[0x64,0xB0,0xFF],[0x92,0x90,0xFF],[0xC6,0x76,0xFF],[0xF3,0x6A,0xFF],[0xFE,0x6E,0xCC],[0xFE,0x81,0x70],[0xEA,0x9E,0x22],
        [0xBC,0xBE,0x00],[0x88,0xD8,0x00],[0x5C,0xE4,0x30],[0x45,0xE0,0x82],[0x48,0xCD,0xDE],[0x4F,0x4F,0x4F],[0x00,0x00,0x00],[0x00,0x00,0x00],
        [0xFF,0xFE,0xFF],[0xC0,0xDF,0xFF],[0xD3,0xD2,0xFF],[0xE8,0xC8,0xFF],[0xFB,0xC2,0xFF],[0xFE,0xC4,0xEA],[0xFE,0xCC,0xC5],[0xF7,0xD8,0xA5],
        [0xE4,0xE5,0x94],[0xCF,0xEF,0x96],[0xBD,0xF4,0xAB],[0xB3,0xF3,0xCC],[0xB5,0xEB,0xF2],[0xB8,0xB8,0xB8],[0x00,0x00,0x00],[0x00,0x00,0x00],
    ];
    let c = P[(code & 0x3F) as usize];
    [c[0], c[1], c[2], 255]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let frame: u64 = args.get(1).expect("帧号").parse().expect("帧号");
    let name = args.get(2).expect("输出名").clone();
    let scene = args.get(3).cloned().unwrap_or_else(|| "gameplay-stage1".into());
    let base: usize = args.iter().position(|a| a == "--base")
        .and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(0);
    let repo_root: std::path::PathBuf = std::env::var("REPO_ROOT").map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe().unwrap_or_default()
                .parent().and_then(|p| p.parent()).and_then(|p| p.parent())
                .map(|p| p.to_path_buf()).unwrap_or_default()
        });
    let rom = std::fs::read(repo_root.join("rom/gunnac_cn.nes")).expect("ROM");
    let trace = std::fs::read_to_string(repo_root.join(format!("analysis/traces/{scene}.frames.jsonl"))).expect("trace");
    // NMI 级重建（R-l4-linebuf 五不变量，t94/t96）：
    // 帧 N 主循环扫写填充行缓冲（probe 写事件）→ 帧 N+1 vblank NMI 读走（读后即废）→
    // 写入 nametable（地址 = 帧 N zp $8E/$8F）；$160=挂起类型，上传后清零（模型侧等效）。
    let mut nt = Nametable::default();
    let mut row_pals: std::collections::HashMap<usize, [u8; 4]> = Default::default(); // 行调色板（双路径②）
    // 【主源切换】probe 捕获写集回放（$2006/$2007 协议解释→nametable——真投影面：
    // 8 组 probe 区含 2006-2007（v8b2 99 万条/stg03 58 万条）——l4 主攻线）。
    if let Some(i) = args.iter().position(|a| a == "--probe-file") {
        if let Some(pp) = args.get(i + 1) {
            let t = match std::fs::read_to_string(pp) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("ERR probe-file 读失败（{}）：{}——静默吞修正", pp, e);
                    String::new()
                }
            };
            {
                let mut ppu: Option<u16> = None;
                let mut latch_hi = true;
                let mut projected = 0usize;
                let mut parsed_lines = 0usize;
                let mut ppu_writes = 0usize;
                let mut w2006 = 0usize;
                let mut w2007 = 0usize;
                let mut latch_seq = 0usize;
                let no_filter = std::env::var("NOOWF").is_ok(); // 旁路开关（覆写过滤）
                let dbg = std::env::var("DBGPROBE").is_ok() || std::env::var("DBGSTREAM").is_ok();
                let dbg_latch = dbg;
                // 【t131 真语义】行缓冲镜像+逐 flag（$0160 非零）事件上传——
                // 单 pending_row 丢行根因（4020 写/帧批=多行/帧仅存末行=散碎片）
                let mut lb = [0u8; 0x31];
                let mut lb_addr: usize = 0;
                let mut zp8e: u8 = 0;
                let mut zp8f: u8 = 0;
                let mut woff: u16 = 0;
                let mut lb_fresh = false;
                let mut fill_rows: u32 = 0;
                let mut prev_x: u16 = 0; // 数据新鲜度（$0131-015F 写后置真——flag 仅新鲜时上传）
                // 【双路径调色板②】$0161 族=行调色板色码组（cc9fcff F0F7=调色板刷新
                // $0161→$3F10 直写；$EC46=色码表 11B 全 NES 色码）——行上传时行调色板
                // 随行状态（W-C1 影子上传族并接=第二路）
                let mut pal_state: [u8; 4] = [0; 4]; // $0161/$0162/$0165/$0168
                const EC46: [u8; 11] = [0x00, 0x10, 0x30, 0x35, 0x25, 0x15, 0x05, 0x07, 0x08, 0x09, 0x0F];
                let _ = EC46;
                let mut row_seq: u32 = 0; // 块内行序（zp 写=重置——块基行序②）
                for l in t.lines() {
                    if l.contains("\"a\":") { parsed_lines += 1; }
                    let la = match l.split("\"a\":\"").nth(1) { Some(x) => x.split('"').next().unwrap_or(""), None => continue };
                    if let Ok(lan) = u16::from_str_radix(la, 16) {
                        if lan == 0x008E { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { zp8e = lv; row_seq = 0; woff = 0; } }
                        if lan == 0x0161 { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { pal_state[0] = lv; } }
                        if lan == 0x0162 { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { pal_state[1] = lv; } }
                        if lan == 0x0165 { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { pal_state[2] = lv; } }
                        if lan == 0x0168 { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { pal_state[3] = lv; } }
                        if lan == 0x008F { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { zp8f = lv; row_seq = 0; woff = 0; } }
                        if (0x0130..=0x0160).contains(&lan) {
                            if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) {
                                lb[(lan - 0x0130) as usize] = lv;
                                if lan != 0x0160 {
                                    lb_fresh = true;
                                    let xv: u16 = l.split("\"x\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse().ok()).unwrap_or(0);
                                    if xv < prev_x { fill_rows += 1; }
                                    prev_x = xv;
                                }
                                // $0160 flag 非零=行上传（目的址=行缓冲尾字段 t131 4 字段）
                                // 【v=2 真身 b876198】$0160=**bitfield 非枚举**：bit0=v=1
                                // 态（EC42 LDA #1）/bit1=完成（88A3 ORA $160 流字节 bit 置位
                                // 合成——无直接 LDA #2）；F0DC CMP #2=bit1 单测（bit0=0 时）。
                                // 12 写点全谱（827A/ECD0=0 族/87EB/EECE/EEE1=3 族/EE59 表查）。
                                // 【flag 语义细化+对账注】trace 实证：v=3（EED4）=瓦写脉冲
                                // （194892 超发 6500×）/ **v=1（BBBD）=行完成候选**（1110≈
                                // 402 行/块×2.7 块量级双证；实测最佳 50088 同点+三带归位
                                // [169,3392,1011]）。**A 级三件套**（f9e90b3：$0160 v=0 空/
                                // 2 完成型/其余处理中+行尾码 $20+低半字节计数行宽 32）——
                                // **对账**：trace v=1@BBBD vs ROM v=2 完成型=疑 $0160-0168
                                // 行参数组多字段/写点族差（细分挂）；fail-closed 按 v=1 实测
                                // 面接（三件套细分证后切 v=2 判定）
                                if lan == 0x0160 && lv == 1 && lb_fresh && std::env::var("FLAGUPLOAD").is_ok() {
                                    // 【配对精化】x=12/244 交替=真对（558/509 均分实锤）
                                    // ——half 按 x 值非奇偶（x=244⇒第二半）
                                    let xv: u16 = l.split("\"x\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse().ok()).unwrap_or(0);
                                    let half = if xv >= 244 { 1 } else { 0 };
                                    lb_fresh = false;
                                    if lb_addr < 3 && std::env::var("DBGPROBE").is_ok() {
                                        eprintln!("DBGLB flag#{} tail={:02x?}", lb_addr, &lb[0x1E..0x31]);
                                    }
                                    // 【块基行序实证注】zp 重置+行递增=47857（<静态块基
                                    // 49919）——flag 63648 vs zp 158≈402 行/块 >>30 行屏=
                                    // flag 重复写疑（行完成判定待证）；静态块基=最佳实测 81.2%
                                    // 【内容落位②】行序变体（ROWMAP env）：0=静态块基
                                    // 1=块内递增 2=反序+半速（t109 降序+双写注：行索引逆序
                                    // 映射+每行双调用——hud_row_seq/2 同式）
                                    let rowmap: u32 = std::env::var("ROWMAP").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
                                    let base_addr = ((zp8f as u16) << 8) | zp8e as u16;
                                    let dst = match rowmap {
                                        1 => base_addr.wrapping_add((row_seq as u16).wrapping_mul(32)),
                                        2 => base_addr.wrapping_add((((29u32).saturating_sub(row_seq / 2)) as u16).wrapping_mul(32)),
                                        3 => base_addr.wrapping_add(half * 32), // 分半（双调用 dest/half 行）
                                        4 => base_addr.wrapping_add((fill_rows as u16).wrapping_mul(32)), // x 环回行序
                                        _ => base_addr,
                                    };
                                    if (0x2000..0x3000).contains(&dst) {
                                        let rel = (dst - 0x2000) as usize;
                                        for i in 0..32usize {
                                            let off = rel + i;
                                            let (n, o) = (off / 0x400, off % 0x400);
                                            if n < 4 && o < 0x3C0 {
                                                nt.tiles[n][o] = lb[i];
                                            }
                                        }
                                    }
                                    row_pals.insert(row_seq as usize, pal_state);
                                    row_seq += 1;
                                    lb_addr += 1;
                                }
                            }
                        }
                    }
                    // 事件行：{"f":..,"a":"2006","v":..}——仅 $2006/$2007 PPU 面
                    let a = match l.split("\"a\":\"").nth(1) { Some(x) => x.split('"').next().unwrap_or(""), None => continue };
                    let v: u8 = match l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()) {
                        Some(x) => x.parse().unwrap_or(0), None => continue,
                    };
                    match a {
                        "2006" => {
                            ppu_writes += 1;
                            w2006 += 1;
                            latch_seq += 1;
                            // 【l4 修】$2006 高字节先行（NES 协议）——旧码低字节位+
                            // 无移位 OR=投影零落根因（0x20|0xC8=0xE8 越界跳过）
                            if latch_hi {
                                ppu = Some((v as u16) << 8);
                                latch_hi = false;
                            } else if let Some(hi) = ppu {
                                ppu = Some(hi | v as u16);
                                latch_hi = true;
                            }
                            if dbg_latch {
                                eprintln!("DBGLATCH a2006 v={:02x} → ppu={:04x?}", v, ppu);
                            }
                        }
                        "2007" => {
                            ppu_writes += 1;
                            w2007 += 1;
                            // 【内容源②】$2007 直写=结局卡真源候选（构建窗行缓冲
                            // 零数据写/纯 flag 脉冲=旧数据上传=全异根因）——目的址=
                            // zp $8E/$8F 基址+写自增（$2006 锁存缺失 quirk 绕行）
                            // 【消融定谳】$2007 路=污染源（NOPROBE 7877≈golden 7680
                            // =text replay 独立真容）——默认禁（PROJ2007=1 逃生）
                            if std::env::var("PROJ2007").is_ok() {
                                let dst = (((zp8f as u16) << 8) | zp8e as u16).wrapping_add(woff);
                                if (0x2000..0x3000).contains(&dst) {
                                    let rel = (dst - 0x2000) as usize;
                                    let (n, o) = (rel / 0x400, rel % 0x400);
                                    if n < 4 && o < 0x3C0 {
                                        nt.tiles[n][o] = v;
                                        projected += 1;
                                    }
                                }
                                woff = woff.wrapping_add(1);
                            }
                        }
                        _ => {}
                    }
                }
                if dbg {
                    eprintln!("DBGPROBE file={} parsed={} ppu={} (2006={} 2007={}) latch_seq={} projected={} nt0={:02x?} no_filter={}", pp, parsed_lines, ppu_writes, w2006, w2007, latch_seq, projected, &nt.tiles[0][0x68..0x70], no_filter);
                }
                if no_filter {
                    eprintln!("DBGPROBE NOOWF 旁路模式（覆写过滤关闭）");
                }
                if projected == 0 {
                    eprintln!("WARN probe 投影零写入（{}）——记录形/覆写诊断", pp);
                }
            }
        }
    }
    // 【末环接线·副路】写集合并投影：屏流（R-4 格式）→ nametable（PPU 协议 $2006 锁存/
    // $2007 自增）——流源=--stream-file（原始流字节；ROM 装载点提取=R-4b 待）。
    if let Some(i) = args.iter().position(|a| a == "--stream-file") {
        if let Some(sp) = args.get(i + 1) {
            if let Ok(bytes) = std::fs::read(sp) {
                // 【l4 根因修】状态机：PPU 地址 2B 仅流首/FE 后（旧启发式误吞 tile）
                let mut ppu: Option<u16> = None;
                let mut pos = 0usize;
                let mut expect_addr = true;
                while pos < bytes.len() {
                    let b = bytes[pos];
                    pos += 1;
                    if b == 0xFE { expect_addr = true; continue; }
                    if b == 0xFF { ppu = None; expect_addr = true; continue; } // 链段界=新地址
                    if expect_addr {
                        if pos >= bytes.len() { break; }
                        let hi = bytes[pos];
                        pos += 1;
                        ppu = Some(u16::from_le_bytes([b, hi]));
                        expect_addr = false;
                    } else if let Some(a) = ppu {
                        let off = (a as usize).wrapping_sub(0x2000);
                        if off < 960 {
                            let n = (off / 960) as usize;
                            let o = off % 960;
                            if n < 4 { nt.tiles[n][o] = b; }
                        }
                        ppu = Some(a.wrapping_add(1));
                    }
                }
                if std::env::var("DBGSTREAM").is_ok() {
                    eprintln!("DBGSTREAM stream={} bytes={} nt0_sample={:02x?}", sp, bytes.len(), &nt.tiles[0][0x68..0x70]);
                }
            }
        }
    }
    let mut buf = [0u8; 0x31]; // $0130-$015F + $0160
    let mut attrs = [0u8; 8];
    let mut addr_regs = (0u8, 0u8, 0u8); // $8E/$8F/$90（t99：probe 写点追踪，解相位陷阱）
    let mut palette_src = [0u8; 7]; // $161-$167 调色板源（t99 ①）
    let mut pending_row: Option<([u8; 32], [u8; 8], u8, u8, u8)> = None;
    let mut buf_fill = 0usize;
    let mut flag_fire = 0usize;
    let mut upload_write = 0usize;
    let mut hud_row_seq = 0usize; // ②基址上下文：预装载非零行按填充序 → $2000 系 LUT 行
    let mut sx = 0u8;
    let mut sy = 0u8;
    let mut zp_prev: Vec<u8> = vec![0; 256];
    // probe 写事件（NMI 级相位）
    let probe_path = repo_root.join(format!("analysis/traces/{scene}.probe.jsonl")).to_string_lossy().into_owned();
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
    // 【l4_timeline 消解令】文本写集路径按 scene 派生（--text-file 覆盖可选）——
    // 此前硬编码 gameplay-stage1-text.jsonl=注入链场景重建用错文本源根因。
    // 【CWD 坑根治】text_path 仓根绝对派生（repo_root 上置）
    let text_path: String = args.iter().position(|a| a == "--text-file")
        .and_then(|i| args.get(i + 1)).cloned()
        .unwrap_or_else(|| repo_root.join(format!("analysis/phase/{}-text.jsonl", scene)).to_string_lossy().into_owned());
    // 【fallback 语义修】严格模式：per-scene 文本件缺 ⇒ **空集**（禁跨场景 fallback
    // ——结局段帧渲 gameplay 文本=码族全异污染源堵死）；TEXTFALLBACK=1 逃生阀
    // （stg02-start 共享域回归用——注：回归修与污染堵的权衡，per-scene 件齐后废弃）
    let text_path = if !std::path::Path::new(&text_path).exists() {
        if std::env::var("TEXTFALLBACK").is_ok() {
            eprintln!("WARN text write set 缺（{}）——TEXTFALLBACK=1 回退 gameplay 集", text_path);
            "analysis/phase/gameplay-stage1-text.jsonl".to_string()
        } else {
            eprintln!("WARN text write set 缺（{}）——严格空集（跨场景 fallback 禁）", text_path);
            String::new()
        }
    } else {
        text_path
    };
    let raw_writes: Vec<(u32, u16, u8, u32, u32)> = std::fs::read_to_string(&text_path)
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
    // 【t228 ①持久化模型】nametable 写入即保持到被覆写——**全写流时序回放**
    // （替换七段滤表模型：4719px 主根=被滤掉的条带写入）；t130 后写胜自然成立
    let mut text_writes: Vec<(u32, u16, u8, u32)> = raw_writes.iter()
        // 【锁存组界修】grp=inv（组界=inv 切换点（lead 定谳）——旧 grp=0 恒值=
        // inv/seg 丢弃=锁存永不重置=**写散布根因**（R3-1 每组新鲜锁存失效））
        .map(|(f, a, v, inv, _seg)| (*f, *a, *v, *inv))
        .collect();
    text_writes.sort_by_key(|w| w.0);
    let text_writes = text_writes;
    let _ = seg_starts;
    // （旧七段滤表已由持久化模型替换——历史见 t228 档）

    let mut text_i = 0usize;
    let mut last_grp: u32 = u32::MAX;
    let mut ppu_addr: u16 = 0;
    let mut tile_page = [[0u8; 32]; 4]; // 1=字模页族（f1207 前装载），2=游戏页族（$6A）
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
        // t230 ①：条带上传（帧 N 填充→帧 N+1 NMI 落 nametable——持久化模型闭环）
        // 【消融定谳】旧 pending_row 路=text replay 合成=7877≈golden 7680（真源！）
        // ——per-flag/$2007 路=污染（禁用态）；bypass 撤
        if true { if let Some((row, _attrs, lo, hi, _c)) = pending_row.take() {
            let base_addr = ((hi as u16) << 8) | lo as u16;
            if (0x2000..0x3000).contains(&base_addr) {
                let rel = (base_addr - 0x2000) as usize;
                for i in 0..32usize {
                    let off = rel + i;
                    let (n, o) = (off / 0x400, off % 0x400);
                    if n < 4 && o < 0x3C0 {
                        nt.tiles[n][o] = row[i]; // ②0 码=擦写（t243）：0 写入=擦除该格
                        upload_write += 1;
                        // 【落位试②】ROWDBL=1 双写（t109 每行双调用注：行写 dest+dest+32）
                        if std::env::var("ROWDBL").is_ok() {
                            let off2 = rel + i + 32;
                            let (n2, o2) = (off2 / 0x400, off2 % 0x400);
                            if n2 < 4 && o2 < 0x3C0 {
                                nt.tiles[n2][o2] = row[i];
                            }
                        }
                    }
                }
            }
        } }
        let zi = line.find("\"zp\":\"").unwrap();
        let zp = hex_decode(&line[zi + 6..][..line[zi + 6..].find('"').unwrap()]);
        // —— 主循环相位：probe 写事件填充缓冲 ——
        if std::env::var("LOOPABL").is_err() {
            apply_frame_writes(&mut buf, &mut attrs, &mut addr_regs, &mut palette_src, f, &mut probe_i);
        }
        // 帧末挂起：行缓冲若本帧被写过（扫写）→ 排队下帧 NMI 上传（地址取当帧 zp）
        // 【l4 修】上传触发域=全缓冲（$0130-$0160 含 flag $0160——旧域 $0130-$0150
        // 漏尾部字段/flag=结束卡 4020 写/帧批疑不触发根因；flag=$0160 非零=挂起类型）
        let wrote = probe.iter().any(|(pf, pa, pv)| {
            *pf == f && (0x0130..=0x0160).contains(pa) && (*pa != 0x0160 || *pv != 0)
        });
        buf_fill += probe.iter().filter(|(pf, pa, _)| *pf == f && (0x0130..=0x0160).contains(pa)).count();
        if wrote { flag_fire += 1; }
        // —— 文本放置（t129 机制终版）：核回放写集（phase-batch 产物）直接落 nametable——
        // 77+46 变长足迹表由 ROM 链原生产出（与 t128 A 级裁定逐址一致）——
        // PPU 协议解释（t131 归属合并击根修）：$2006 写锁存地址（高/低），
        // $2007 写落 tile（地址自增）——注册器写序列 → nametable 态
        while std::env::var("TEXTABL").is_err() && text_i < text_writes.len() && text_writes[text_i].0 <= f as u32 {
            let (_f, addr, val, grp) = text_writes[text_i];
            if grp != last_grp {
                ppu_addr_latch_high = true; // R3-1：每组新鲜锁存（修相位错位）
                last_grp = grp;
            }
            text_i += 1;
            match addr {
                0x2006 => {
                    // 【字节序定】hi 先（对值域实证：0x20-25=nametable hi/0x93-96=CHR hi
                    // ——均 hi 先 ✓ lo 先实验回退）
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
                        if n < 4 {
                            if off < 0x3C0 {
                                nt.tiles[n][off] = val;
                                // t157 ② 切换时机：装载帧 <f1207（首切换帧）=字模族
                                tile_page[n][off / 32] = if f < 1207 { 1 } else { 2 };
                            } else {
                                nt.attrs[n][off - 0x3C0] = val; // R3-1：属性区写不丢
                            }
                        }
                    }
                    ppu_addr = ppu_addr.wrapping_add(1);
                }
                _ => {}
            }
        }
        if wrote {
            let mut row = [0u8; 32];
            // 【行窗相位】ROWWIN=k 环境变量（默认 0）——行提取窗偏移实证扫
            let rowwin: usize = std::env::var("ROWWIN").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
            row.copy_from_slice(&buf[rowwin..rowwin + 32]);
            let addr_ok = ((addr_regs.1 as u16) << 8 | addr_regs.0 as u16) >= 0x2000;
            if !addr_ok && row.iter().any(|&v| v != 0) && std::env::var("NOHUDLUT").is_err() {
                // ① 放置序=**降序+双写**（t109 参数表：行索引逆序映射、每行双调用——
                // 顺序填充实验反向即本因）：fills/2 → LUT 行 15 递降
                let lut = [0x21C0u16, 0x2200, 0x2240, 0x2280, 0x22C0, 0x2300, 0x2340, 0x2380];
                let idx = 7usize.saturating_sub(hud_row_seq / 2 % 8);
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
    let ntr: usize = std::env::var("RENDER_NT").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let ntr_dbg = ntr;
    if std::env::var("DBGPROBE").is_ok() || std::env::var("DBGSTREAM").is_ok() {
        let nt_nz = nt.tiles.iter().flat_map(|t| t.iter()).filter(|&&t| t != 0).count();
        let per_nt: Vec<(usize, usize)> = nt.tiles.iter()
            .map(|t| (t.iter().filter(|&&x| x > 0 && x < 128).count(), t.iter().filter(|&&x| x >= 128).count()))
            .collect();
        eprintln!("DBGUPLOAD buf_fill={} flag_fire={} upload_write={} nt_nz={} per_nt(lo,hi)={:?}", buf_fill, flag_fire, upload_write, nt_nz, per_nt);
    }
    // 渲染：nametable 32×30 + 卷轴
    let mut canvas = Canvas::new();
    let mut draws = 0usize;
    // 【t243 核修】2×2 nametable 视口合成（滚动 sx/sy 选择——旧读单 ntr 漏 nt2
    // 半屏=结局卡 480 瓦缺投影根因；≥128 跳码仅 15 瓦（非主差源））。
    for ty in 0..30usize {
        for tx in 0..32usize {
            let vx = (tx as i32) * 8 + sx as i32; // 视口像素坐标（512 宽域）
            let vy = (ty as i32) * 8 + sy as i32; // 480 高域
            let mx = ((vx % 512) + 512) % 512;
            let my = ((vy % 480) + 480) % 480;
            let nsel = (mx / 256) as usize + ((my / 240) as usize) * 2;
            let sub_x = (mx % 256) as usize / 8;
            let sub_y = (my % 240) as usize / 8;
            let raw_tile = nt.tiles[nsel.min(3)][sub_y * 32 + sub_x];
            // ② 小值参数族分离（t152）：0-16 簇=参数字节非 tile（AR2-D bit5 同族）——
            // 直拷制下渲染按透明（不画 glyph）；命名值仍保留在 nametable（内容层精确不动）
            // t208 ①：≥128 族=CJK glyph tile（oracle 实证：128 位置绘字）——
            // t152 参数族仅 0-16 簇（菜单期）；高值族渲染修正
            // ③条带码四分路由（t243）：0 擦写/<64 图形符号/64-127 文本/≥128 参数标记
            if raw_tile == 0 || raw_tile >= 128 {
                continue; // 擦写格不画/参数标记不画
            }
            let tile = raw_tile as usize + base * 256;
            let px = (tx as i32) * 8;
            let py = (ty as i32) * 8;
            // t157 三行映射（t154 A 级）：字模族→124/126 对，游戏族→$6A(106) 对
            // ② 字模 CHR 直读（t167 A 级）：bank124 基 + tile×16 **线性区**（跨 bank 124-127 连续）
            // R16-1：恒等式保真维持（低误差域复测 pair≡linear 98.07 同值）
            // ①统一矩阵（t238/t239）：bank=窗基+T//64、tile=T%64——窗基上下文表
            // （菜单 124/CJK 28/图形 20 待条带族取证——t235/t239 实测 20/28/22/30/96/106
            // 全不匹配留档）；条带行上下文未定=中性（不画）
            // ①③四分路由实测注（t243）：<64→图形窗 20 实测净负（97.48<98.34 守卫拦下）
            // ——保持字体窗 124+tile/64（t238 矩阵窗基 124 实例）；四分路由注留档
            // 【映射修】tile_page 2 族 bank 基 28→**44**（golden 反查实证：结局卡字形
            // =bank44 瓦 1/2/3；bank28 直读全零=旧基错）。行窗偏移（码内容差）=余诊。
            let bank = if tile_page[ntr][ty] == 2 { 44 } else { 124 } + tile / 64;
            if bank < 128 {
                // 5 色绑定合流：BG 文本=白（绑定定谳）；index 0=黑（backdrop）
                // 【2026-10-13 更正（t185 R18-1 高危）】墨掩码=p0 置位（索引 1/3 为墨、
                // 0/2 为底）——t181『高平面』结论反了（相位错位伪数），实测 ink=p0 低平面
                // 掩码 100%（256/256）；高平面仅 80.9%（承错实现 55.5% 画错形）
                // 【2026-10-13 t192 裁定】双色渲染调色板依赖——两平面各自精确，墨色按 palette
                // 映射勿固定 index（R18-1/p2 争议闭合：p0 稠墨暴露放置缺口/p2 稀疏被指标奖励
                // ——均非真根，真根=palette 映射）——非零索引全可见（绑定白）
                // 【R19-1 逐块 palette 映射】文本块 index2→白（已证）——勿全局 2only
                // （可见文本帧按 2only 丢字）；CJK 精灵 pal index1=白=墨（256/256 正确）
                // t201 ①：合成规则墨表（t199 index=p0|(p1<<1)）+ palette 全表候选
                // p0 置位（1/3）为墨：1=白、3=青（绑定色候选）
                // t206 ①实测注：二维墨表（族 B=CJK 槽 1=teal/3=white）96.25/95.41
                // < 族 A 单式 98.15/97.27——CJK 族 glyph 形状根（W73'）阻墨表生效；
                // 保持族 A 单式（index2）为实测判优默认，二维墨表留 t204/t206 档注
                // ③ 二维墨表落地（t204/t207）：Game 族映射=当前判优配置（lossless）
                // 【双路径调色板②渲染侧】行调色板=$0161 族色码组（cc9fcff）——
                // NES 色码→RGB（标准 2C02 调色板）；旧行=ink_table 族表（保真对照注）
                // 【消融定谳】row_pals 值域未解（pal_state=15→NES 黑=内容黑化根因——
                // EC46 映射/字段角色待证）——ink_table 族表回退（7877≈golden 7680 面）
                // PALROW=1 逃生阀（row_pals 试）
                let bg_pal: [game_desktop::Rgba; 4] = if std::env::var("PALROW").is_ok() {
                    let row_pal = row_pals.get(&ty).copied().unwrap_or([0x0F, 0x30, 0x10, 0x00]);
                    [nes_rgb(row_pal[0]), nes_rgb(row_pal[1]), nes_rgb(row_pal[2]), nes_rgb(row_pal[3])]
                } else {
                    let fam = if bank >= 124 { game_desktop::render_rules::SpriteFamily::Cjk } else { game_desktop::render_rules::SpriteFamily::Game };
                    [
                        game_desktop::render_rules::ink_table(fam, 0),
                        game_desktop::render_rules::ink_table(fam, 1),
                        game_desktop::render_rules::ink_table(fam, 2),
                        game_desktop::render_rules::ink_table(fam, 3),
                    ]
                };
                canvas.draw_chr(&rom, bank, tile % 64, px, py, &bg_pal);
                draws += 1;
            }
        }
    }
    if std::env::var("DBGPROBE").is_ok() {
        let nz = canvas.pixels.chunks_exact(4).filter(|p| p[0] | p[1] | p[2] != 0).count();
        eprintln!("DBGDRAW draws={} canvas_nz={}", draws, nz);
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
    // t186 ③：精灵面同高平面语义——index 2 = CJK 字形面（此前透明=字形缺失根）
    // t186 ③实测注：精灵面 index2 开=净负（98.05/96.75 & 98.07/97.09 < 全闭 98.13/97.22）
    // ——维持全闭，精灵形残余另有根（待后续）
    let sprite_pals: [[game_desktop::Rgba; 4]; 4] = [
        [unk, mk(0xFF, 0xFE, 0xFF), unk, mk(0x00, 0x40, 0x4D)],
        [unk, mk(0xFF, 0xFE, 0xFF), unk, mk(0xFF, 0xFE, 0xFF)],
        [unk, mk(0xBC, 0xBE, 0x00), unk, mk(0xBC, 0xBE, 0x00)],
        [unk, mk(0xFF, 0xFE, 0xFF), unk, mk(0xE4, 0xE5, 0x94)],
    ]; // R20-1 精灵墨（1/3）——label index2 开净负注（t212 回归守卫）
    let bg_opaque = [[false; 256]; 240]; // BG 面重建为空（黑场）——掩码全透明
    if std::env::var("SPRABL").is_err() {
        game_desktop::render_sprites(&mut canvas, &rom, &sprites, 30, &sprite_pals, &bg_opaque);
    }

    std::fs::create_dir_all(repo_root.join("oracle/screens/actual")).unwrap();
    let out_path = repo_root.join(format!("oracle/screens/actual/{name}.png")).to_string_lossy().into_owned();
    canvas.export_png(std::path::Path::new(&out_path)).expect("导出");
    let idx_path = repo_root.join(format!("oracle/screens/actual/{name}.idx.json")).to_string_lossy().into_owned();
    std::fs::write(&idx_path, format!(
        "{{\"palette_src\":\"{}\",\"attrs\":\"{}\",\"tiles\":\"{}\"}}",
        palette_src.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        nt.attrs[ntr].iter().map(|b| format!("{b:02x}")).collect::<String>(),
        nt.tiles[ntr].iter().map(|b| format!("{b:02x}")).collect::<String>(),
    )).expect("侧车");
    let _ = idx_rows;
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open(repo_root.join("oracle/screens/actual/.render.log")).unwrap();
    writeln!(log, "{name} frame={frame} base={base} scroll=({sx},{sy}) nt_nz={} probe={}", nt.tiles.iter().flat_map(|t| t.iter()).filter(|&&t| t != 0).count(), probe.len()).unwrap();
    let _ = (SCREEN_W, SCREEN_H);
}
