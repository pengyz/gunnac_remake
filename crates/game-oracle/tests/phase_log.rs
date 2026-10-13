//! t127 ④：PPU 写日志工具测试 + 负向自证。

use game_oracle::cpu6502::{Bus, Cpu, StepResult};
use game_oracle::l2::L2Bus;

#[test]
fn ppu_log_records_only_2000_2007_and_gate() {
    let rom = std::fs::read("../../rom/gunnac_cn.nes").unwrap();
    let mut bus = L2Bus::new(&rom, 0, 8).unwrap();
    bus.write(0x001A, 0xA1);
    bus.write(0x001B, 0x1E);
    bus.write(0x2006, 0x21);
    bus.write(0x2007, 0x4A);
    bus.write(0x1234, 0x99); // RAM 写不入 PPU 日志
    assert_eq!(bus.ppu_log.len(), 2, "仅 $2000-$2007 入日志");
    assert_eq!(bus.ppu_log[0].addr, 0x2006);
    assert_eq!(bus.ppu_log[0].gate_a, 0xA1, "门控态逐写点快照");
    assert_eq!(bus.ppu_log[1].val, 0x4A);
    // 负向：非 PPU 地址不入
    assert!(bus.ppu_log.iter().all(|w| (0x2000..=0x2007).contains(&w.addr)));
}

#[test]
fn phase_dataset_chain_produces_writes() {
    // 工具冒烟：真实流入口（$ABFD，bank8）经文本链产出 $2006/$2007 写序列
    let rom = std::fs::read("../../rom/gunnac_cn.nes").unwrap();
    let mut bus = L2Bus::new(&rom, 0, 8).unwrap();
    bus.write(0x002F, 0xFD);
    bus.write(0x0030, 0xAB);
    bus.write(0x0014, 0x50);
    bus.write(0x0015, 0x21);
    bus.writes.clear();
    bus.ppu_log.clear();
    let mut cpu = Cpu::new(0, 0, 0, 0xFD, 0x24, 0x96CE);
    let ret = 0xFFFEu16.wrapping_sub(1);
    bus.write(0x01FD, (ret >> 8) as u8);
    bus.write(0x01FC, ret as u8);
    bus.writes.clear();
    bus.ppu_log.clear();
    let mut depth = 0i32;
    for _ in 0..50_000u32 {
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
            _ => break,
        }
    }
    assert!(!bus.ppu_log.is_empty(), "文本链必须产出 PPU 写（负向哨兵）");
    // AR3-D 层1 真比对（非自比对）：段1 内容 = t130 §2 表 A 级判读（74 74 74 83…）
    let data: Vec<u8> = bus.ppu_log.iter().filter(|w| w.addr == 0x2007).map(|w| w.val).collect();
    assert!(data.len() >= 4 && data[0] == 0x4A && data[1] == 0x4A && data[2] == 0x4A && data[3] == 0x53, "段1 内容=t130 §2 真值（真比对）：实际 {:?}", &data[..4.min(data.len())]);
    // 地址对：段1 起点 $2170（高 $21 低 $70）
    let addr_writes: Vec<u8> = bus.ppu_log.iter().filter(|w| w.addr == 0x2006).map(|w| w.val).collect();
    assert!(addr_writes.len() >= 2 && addr_writes[0] == 0x21 && addr_writes[1] == 0x70, "段1 起点 $2170（$2006 对真比对）：实际 {:?}", &addr_writes[..2.min(addr_writes.len())]);
}
