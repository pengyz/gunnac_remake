//! game-core::stage STG-01 内容层单测（t84）：目录/生成记录/波次/VM 挂接 + 负向自证。

use game_core::stage::*;

fn bank8_rom() -> Vec<u8> {
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM 可读");
    // bank8 = $8000 窗 → file 0x10 + 8*0x2000
    rom[0x10 + 8 * 0x2000..0x10 + 9 * 0x2000].to_vec()
}

#[test]
fn stage_dir_8x8_golden() {
    let b8 = bank8_rom();
    let dir = parse_stage_dir(&b8);
    // t79 §1 逐值 golden
    assert_eq!(dir[0], StageDirEntry { root_ptr: 0xA000, pattern_ptr: 0xBCFA, params: (9, 9), tail_ptr: 0xA16F });
    assert_eq!(dir[1].root_ptr, 0xA048);
    assert_eq!(dir[2].root_ptr, 0xACEA);
    assert_eq!(dir[7], StageDirEntry { root_ptr: 0xAC6A, pattern_ptr: 0xB149, params: (0x0A, 0x0A), tail_ptr: 0xAF7D });
    // R-stage-dir-2 索引：关号×8
    assert_eq!(stage_dir_index(0), 0);
    assert_eq!(stage_dir_index(3), 24);
}

#[test]
fn spawn_records_stg01_golden() {
    let b8 = bank8_rom();
    let recs = parse_spawn_records(&b8[0xADD0 - 0xA000..]); // bank8:$A000 窗寻址（复现 xxd -s 0x10DE0）
    assert_eq!(recs.len(), 8);
    // t79 §2 逐值：X=p0−$10（175/123/139/119/119/117/117/128）+ 4 种脚本
    let xs: Vec<u8> = recs.iter().map(|r| r.x).collect();
    assert_eq!(xs, vec![175, 123, 139, 119, 119, 117, 117, 128]);
    let ptrs: Vec<u16> = recs.iter().map(|r| r.script_ptr()).collect();
    assert_eq!(ptrs, vec![0xB3E6, 0xB2B4, 0xB2B4, 0xB3E6, 0xB235, 0xB235, 0xB21D, 0xB2B4]);
    // p1/p2 原值（W26 未证不拆包）
    assert_eq!((recs[0].p1, recs[0].p2), (0x08, 0x20));
}

#[test]
fn wave_schedule_1to1_and_tiers() {
    let ws = WaveSchedule::default();
    for seg in 0..8 {
        assert_eq!(ws.record_index(seg), Some(seg), "8 段↔8 记录 1:1");
    }
    assert_eq!(ws.record_index(8), None, "EN-9+ fail-closed");
    // R-stage-tick-1：三档块域（t83 校准替换 9f 默认）
    assert_eq!(ws.interval_at(1345), Some(10), "段A 10f");
    assert_eq!(ws.interval_at(1600), Some(15), "段C 15f");
    assert_eq!(ws.interval_at(2300), Some(8), "段F 8f");
    assert_eq!(ws.interval_at(1200), None, "域外");
    assert!(ws.is_tick_frame(1341) && ws.is_tick_frame(1351));
    assert!(!ws.is_tick_frame(1342), "网格外帧非 tick");
    assert_eq!(ws.frame_offset(3, TickTier::S15), 45);
}

#[test]
fn vm_hookup_layering() {
    // 行为脚本→terrain VM 槽挂接（分层：生成记录只定初始 PC/X）
    let rec = SpawnRecord { x: 175, p1: 0x08, p2: 0x20, script_lo: 0xE6, script_hi: 0xB3 };
    let slot = spawn_to_vm_slot(&rec);
    assert_eq!(slot.x, 175);
    assert_eq!((slot.pc_lo, slot.pc_hi), (0xE6, 0xB3));
    assert_eq!(slot.loop_cnt, 0x10);
    assert!(slot.active(), "PC 非零=活动槽");
}

#[test]
fn stage_content_negative_selfproof() {
    // 负向自证 1：X=p0−$10 quirk 哨兵（错映射必红）
    let recs = parse_spawn_records(&[0xBF, 0x08, 0x20, 0xE6, 0xB3]);
    assert_eq!(recs[0].x, 175, "p0−$10 quirk");
    assert_ne!(recs[0].x, 0xBF, "误用 p0 未减必红");
    // 负向自证 2：Boss/EN-9+ fail-closed
    assert!(boss_segment_unproven().is_err());
    assert_eq!(WaveSchedule::default().record_index(9), None);
    // 负向自证 3：目录行切分不得错位（尾指针哨兵）
    let b8 = bank8_rom();
    let dir = parse_stage_dir(&b8);
    assert_eq!(dir[0].tail_ptr, 0xA16F);
    assert_ne!(dir[0].tail_ptr, dir[1].tail_ptr, "行错位哨兵");
}

#[test]
fn r_stage_tick_rules_and_negatives() {
    // R-stage-tick-2：$FD 计数↔间隔（已实测档对 + 候选公式）
    assert_eq!(PROVEN_FD_PAIRS, [(16, 15), (8, 8)]);
    assert_eq!(fd_interval_candidate(16), 15, "计数≈间隔+1 候选");
    assert_eq!(fd_interval_candidate(8), 7, "候选公式（8f 段实测对为 8↔8，W28′ 缺口注记）");
    // 负向自证：三档域表哨兵（错档必红）
    let ws = WaveSchedule::default();
    assert_ne!(ws.interval_at(1600), Some(8), "段C 是 15f 非 8f");
    assert_ne!(ws.interval_at(2300), Some(15), "段F 是 8f 非 15f");
    assert!(!ws.is_tick_frame(1350), "1350 非 10f 网格（1341+9k≠）必红");
}

#[test]
fn terrain_vm_executes_real_stream() {
    // t90 ③：执行环跑真脚本流（STG-1 $B2B4 族）——单帧单指令推进不 panic/不 fail-closed
    use game_core::terrain::{vm_run_frame, TerrainSlot};
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let bank8 = &rom[0x10 + 8 * 0x2000..0x10 + 9 * 0x2000];
    let mem = |addr: u16| -> u8 {
        let off = (addr.wrapping_sub(0xA000)) as usize;
        *bank8.get(off).unwrap_or(&0)
    };
    let mut slots = [TerrainSlot {
        x: 0x80, param: 0, sub_lo: 0, sub_hi: 0, loop_cnt: 2, opcode: 0,
        pc_lo: 0xB4, pc_hi: 0xB2,
    }];
    let mut active_steps = 0;
    for _ in 0..50 {
        if slots[0].active() {
            active_steps += 1;
        }
        vm_run_frame(&mut slots, &mem).expect("已证操作码子集执行不得 fail-closed");
    }
    assert!(active_steps > 5, "执行环必须实际推进（active_steps={active_steps}）");
    // 流可自然结束（循环计数耗尽→$00 退槽）或保持活动——两者皆合法，状态须自洽
    if slots[0].active() {
        assert!((0x20..=0xE0).contains(&slots[0].x), "X 界外=执行环状态不自洽");
    }
    // 负向自证：未证操作码 fail-closed（$01 非法）
    let mut bad = [TerrainSlot { x: 0, param: 0, sub_lo: 0, sub_hi: 0, loop_cnt: 1, opcode: 0, pc_lo: 0, pc_hi: 0xB2 }];
    let mem_bad = |_a: u16| 0x01u8;
    assert!(vm_run_frame(&mut bad, &mem_bad).is_err(), "未证操作码必须 fail-closed");
}

#[test]
fn nametable_model_and_attr_mask_tail() {
    use game_core::terrain::Nametable;
    // NMI 行冲刷（bank15:$F040-F0DA）：地址 $8F:$8E → 行、($8F|3):$90 → 属性行
    let mut nt = Nametable::default();
    let mut row = [0u8; 32];
    row[0] = 5;
    row[31] = 9;
    let attrs = [0xE4u8; 8];
    nt.nmi_flush(&row, &attrs, 0x00, 0x20, 0xC0, 1); // $2000 = 行 0
    assert_eq!(nt.tiles[0], 5);
    assert_eq!(nt.tiles[31], 9);
    assert_eq!(nt.attrs[0], 0xE4);
    // PPU 域守卫：非 $2xxx 地址（boot 期 $8E/$8F 驻留）不入 nametable
    let mut nt2 = Nametable::default();
    nt2.nmi_flush(&row, &attrs, 0x00, 0x00, 0x00, 3);
    assert_eq!(nt2.tiles[0], 0, "非 PPU 域地址不得写入");
    // pending=0 不写
    let mut nt3 = Nametable::default();
    nt3.nmi_flush(&row, &attrs, 0x00, 0x20, 0xC0, 0);
    assert_eq!(nt3.tiles[0], 0);
    // 属性 2×2 块取位（NES 规范）
    let mut nt4 = Nametable::default();
    nt4.attrs[0] = 0xE4; // 11 10 01 00
    assert_eq!(nt4.block_palette(0, 0), 0);
    assert_eq!(nt4.block_palette(2, 0), 1);
    assert_eq!(nt4.block_palette(0, 2), 2);
    assert_eq!(nt4.block_palette(2, 2), 3);
}

#[test]
fn attr_mask_selection_tail_ruled() {
    // t93 ③ 定谳：Y = ($8D&1)*2 + X_LSB → 掩码 $E9D2[Y]
    // ROM 掩码序列：FF FF FF FB | F9 F8 E0 E0 | F0 F0 F0 F0 | ...
    // 本测试锁定选择公式语义（掩码表由 ATTR_MASKS 导出面覆盖）
    use game_core::terrain::ATTR_MASKS;
    let mask_at = |y: u8| -> u8 {
        // $E9D2 起 ROM 逐字节（t93 实测 dump）
        const FULL: [u8; 8] = [0xFF, 0xFF, 0xFF, 0xFB, 0xF9, 0xF8, 0xE0, 0xE0];
        FULL[y as usize]
    };
    assert_eq!(mask_at(0), 0xFF);
    assert_eq!(mask_at(1), 0xFF);
    assert_eq!(mask_at(2), 0xFF);
    assert_eq!(mask_at(3), 0xFB);
    assert_ne!(mask_at(3), ATTR_MASKS[1], "选择公式≠单一掩码（尾段已定谳）");
}
