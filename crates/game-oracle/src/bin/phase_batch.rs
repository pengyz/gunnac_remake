//! phase-batch —— t129：全调用批量回放（帧内相位数据集合并）。
//! 用法：phase-batch <场景> [输出名]——遍历 trace 全 $2F/$30 调用帧，核回放 → 合并写集。
use game_oracle::cpu6502::{Bus, Cpu, StepResult};
use game_oracle::l2::L2Bus;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scene = args.get(1).cloned().unwrap_or_else(|| "gameplay-stage1".into());
    let out_name = args.get(2).cloned().unwrap_or_else(|| format!("{scene}-text"));
    let rom = std::fs::read("rom/gunnac_cn.nes").expect("ROM");
    let trace = std::fs::read_to_string(format!("analysis/traces/{scene}.frames.jsonl")).expect("trace");
    // probe 指针写（$2F/$30）按帧
    let probe = std::fs::read_to_string(format!("analysis/traces/{scene}.probe.jsonl")).unwrap_or_default();
    let mut ptrs: std::collections::HashMap<u64, (u8, u8)> = std::collections::HashMap::new();
    for l in probe.lines() {
        if !l.contains("\"a\"") { continue; }
        let f: u64 = match l.split("\"f\":").nth(1).and_then(|s| s.split(',').next()).and_then(|s| s.parse().ok()) { Some(v) => v, None => continue };
        let a = match l.split("\"a\":\"").nth(1).and_then(|s| s.split('"').next()).and_then(|s| u16::from_str_radix(s, 16).ok()) { Some(v) => v, None => continue };
        let v = match l.split("\"v\":").nth(1).and_then(|s| s.split(&[',', '}'][..]).next()).and_then(|s| s.parse().ok()) { Some(v) => v, None => continue };
        match a {
            0x2F => ptrs.entry(f).or_insert((0, 0)).0 = v,
            0x30 => ptrs.entry(f).or_insert((0, 0)).1 = v,
            _ => {}
        }
    }
    let mut merged: Vec<(u32, u16, u8, u32)> = Vec::new();
    let mut n = 0;
    for line in trace.lines() {
        let f: u64 = match line.split("\"f\":").nth(1).and_then(|s| s.split(',').next()).and_then(|s| s.parse().ok()) { Some(v) => v, None => continue };
        let Some(&(slo, shi)) = ptrs.get(&f) else { continue };
        let zi = line.find("\"zp\":\"").unwrap();
        let zp_hex = &line[zi + 6..];
        let zp: Vec<u8> = (0..zp_hex.find('"').unwrap() / 2).map(|i| u8::from_str_radix(&zp_hex[2 * i..2 * i + 2], 16).unwrap()).collect();
        let r7 = 8usize; // bank8 默认（变体见 dual_track）
        let mut bus = L2Bus::new(&rom, 0, r7).expect("bus");
        bus.write(0x002F, slo);
        bus.write(0x0030, shi);
        bus.write(0x0014, zp[0x14]);
        bus.write(0x0015, zp[0x15]);
        bus.write(0x0005, 0x50);
        bus.write(0x0006, 0x30);
        let mut cpu = Cpu::new(0, 0, 0, 0xFD, 0x24, 0x96CE);
        let ret = 0xFFFEu16.wrapping_sub(1);
        bus.write(0x01FD, (ret >> 8) as u8);
        bus.write(0x01FC, ret as u8);
        bus.writes.clear();
        bus.ppu_log.clear();
        let mut depth = 0i32;
        for _ in 0..20_000u32 {
            if cpu.pc == 0xFFFF { break; }
            bus.steps += 1;
            match cpu.step(&mut bus) {
                Ok(StepResult::Continue) => {}
                Ok(StepResult::Jsr) => depth += 1,
                Ok(StepResult::Rts) => { if depth > 0 { depth -= 1; } else { break; } }
                _ => break,
            }
        }
        for w in &bus.ppu_log {
            merged.push((f as u32, w.addr, w.val, n as u32)); // 归属键：调用序号
        }
        n += 1;
    }
    std::fs::create_dir_all("analysis/phase").unwrap();
    let path = format!("analysis/phase/{out_name}.jsonl");
    let mut fo = std::fs::File::create(&path).unwrap();
    for (f, a, v, inv) in &merged {
        writeln!(fo, "{{\"f\":{f},\"addr\":\"{a:04X}\",\"val\":{v},\"inv\":{inv}}}").unwrap();
    }
    println!("phase-batch: {n} 调用回放，{} 写点 → {path}", merged.len());
}
