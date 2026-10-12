//! game-core::enemy 单测（t62）：装载机 quirk + VM 内核 + 输出表界。

use game_core::enemy::*;

#[test]
fn load_record_x_is_p0_minus_10_quirk() {
    // quirk：SBC #$0F 前 carry=0（LSR 链）⇒ 实减 $10（findings 的 −$0F 系进位疏漏）
    let e = load_record(0x20, 0x13, 0xB4, 0xB2);
    assert_eq!(e, LoadEffect::Spawn { slot: 3, x: 0x10, loop_cnt: 1, pc_lo: 0xB4, pc_hi: 0xB2 });
}

#[test]
fn load_record_unpack_and_quirks() {
    // p1>>4==0 ⇒ 循环计数 $10（0 编码即 16）
    assert_eq!(load_record(0x30, 0x0B, 0, 0), LoadEffect::Disable { slot: 3, loop_cnt: 0x10 });
    // 位 3 停用记录（2 字节）
    assert_eq!(load_record(0x30, 0x0F, 0, 0), LoadEffect::Disable { slot: 7, loop_cnt: 0x10 });
    // 段控制/结束
    assert_eq!(load_record(0x83, 0, 0, 0), LoadEffect::Segment { count: 3 });
    assert_eq!(load_record(0x00, 0, 0, 0), LoadEffect::End);
}

#[test]
fn x_adjust_is_band_gate_not_clamp() {
    // 界门语义（字面指令流）：界内 [$20,$E0) ⇒ 退槽；界外 ⇒ 续行；X 不钳
    assert_eq!(vm_x_adjust(0x10, 0x05), (0x15, false), "界外续行（不钳）");
    assert_eq!(vm_x_adjust(0x30, 0x05), (0x35, true), "界内退槽");
    assert_eq!(vm_x_adjust(0xDE, 0x05), (0xE3, false), "≥E0 续行");
    assert_eq!(vm_x_adjust(0xFF, 0x01), (0x00, false), "回绕界外续行");
}

#[test]
fn loop_step_and_output_bound() {
    assert_eq!(vm_loop_step(3, 0x34, 0x12), (2, true, 0x34, 0x12), "未尽跳指针");
    assert_eq!(vm_loop_step(1, 0x34, 0x12), (0, false, 0, 0), "尽跳过");
    assert_eq!(output_table_write(0x0F, 7), Some((0x0F, 7)));
    assert_eq!(output_table_write(0x10, 7), None, "CPX #$10 界");
    assert_eq!(vm_fetch_params(0x10, 3, 3, 7, 0x50, 0x60), (7, 0x50, 0x60, 0x14, 3), "PC 前进+1 进位");
}
