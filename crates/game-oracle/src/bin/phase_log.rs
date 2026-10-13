//! phase-log —— t127 帧内相位数据集生成：自研 6502 核回放文本放置链，
//! 逐写点记录 $2000-$2007（步序/写值/门控态）→ JSONL（交 analyst 定位）。
//! 用法：phase-log <流指针 hex> <累加器 lo> <hi> <R7> [bank6] [输出名]

use game_oracle::cpu6502::{Bus, Cpu, StepResult};
use game_oracle::l2::L2Bus;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ptr = u16::from_str_radix(args.get(1).expect("流指针"), 16).expect("hex");
    let acc_lo: u8 = args.get(2).and_then(|v| u8::from_str_radix(v, 16).ok()).unwrap_or(0);
    let acc_hi: u8 = args.get(3).and_then(|v| u8::from_str_radix(v, 16).ok()).unwrap_or(0x23);
    let r7: usize = args.get(4).and_then(|v| v.parse().ok()).unwrap_or(8);
    let r6: usize = args.get(5).and_then(|v| v.parse().ok()).unwrap_or(0);
    let out_name = args.get(6).cloned().unwrap_or_else(|| format!("phase_{ptr:04X}"));
    let rom = std::fs::read("rom/gunnac_cn.nes").expect("ROM");
    let mut bus = L2Bus::new(&rom, r6, r7).expect("总线");
    // 运行时态输入（traces 采样口径）：$2F/$30 流指针、$14/$15 累加器
    bus.write(0x002F, ptr as u8);
    bus.write(0x0030, (ptr >> 8) as u8);
    bus.write(0x0014, acc_lo);
    bus.write(0x0015, acc_hi);
    // 预算配额字段（t118 $E828：$05/$06/$21）——预置满额便于观测写路径
    bus.write(0x0005, 0x50);
    bus.write(0x0006, 0x30);
    bus.write(0x0021, 0x00);
    bus.writes.clear();
    bus.ppu_log.clear();
    // 入口 bank00:$96CE（流指针装载→文本链）；停止哨兵=RTS 回调用者
    let entry = u16::from_str_radix(args.get(7).unwrap_or(&"96CE".into()), 16).unwrap_or(0x96CE);
    let mut cpu = Cpu::new(0, 0, 0, 0xFD, 0x24, entry);
    let ret = 0xFFFEu16.wrapping_sub(1);
    bus.write(0x01FD, (ret >> 8) as u8);
    bus.write(0x01FC, ret as u8);
    bus.writes.clear();
    bus.ppu_log.clear();
    std::fs::create_dir_all("analysis/phase").unwrap();
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open("analysis/phase/.log").unwrap();
    let mut depth = 0i32;
    for _ in 0..200_000u32 {
        if cpu.pc == 0xFFFF {
            break;
        }
        bus.steps += 1;
        match cpu.step(&mut bus) {
            Ok(StepResult::Continue) => {}
            Ok(StepResult::Jsr) => depth += 1,
            Ok(StepResult::Rts) => {
                if depth > 0 {
                    depth -= 1;
                } else {
                    break;
                }
            }
            Ok(StepResult::Rti) | Ok(StepResult::Brk) => break,
            Err(e) => {
                writeln!(log, "{out_name} STEP-ERR at {:04X}: {e}", cpu.pc).unwrap();
                break;
            }
        }
        if bus.fault.is_some() {
            break;
        }
    }
    let path = format!("analysis/phase/{out_name}.jsonl");
    let mut f = std::fs::File::create(&path).unwrap();
    for w in &bus.ppu_log {
        writeln!(f, "{{\"step\":{},\"addr\":\"{:04X}\",\"val\":{},\"gate_a\":{},\"gate_b\":{}}}", w.step, w.addr, w.val, w.gate_a, w.gate_b).unwrap();
    }
    writeln!(log, "{out_name} ptr={ptr:04X} acc={acc_lo:02X}/{acc_hi:02X} r7={r7} entry={entry:04X} steps={} ppu={} ramwrites={} fault={:?}", bus.steps, bus.ppu_log.len(), bus.writes.len(), bus.fault).unwrap();
}
