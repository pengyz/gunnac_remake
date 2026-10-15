//! game-core::wave_record 波次记录机 CA/C9 判定链测试（批次4a，TDD）。
//!
//! 证据基准：`docs/findings/wave-ca-flags.md` §2（逐位语义，A 级逐指令）+
//! §3（17/17 全谱相关性）+ §6（C9 侧）；`docs/findings/stage-wave-crosscut.md` §2
//! （变化族全景）。golden = `wave_stream_decode.py wave 1..8` 全谱实测值。
//! 先测试后实现；负向哨兵锁死组合判/XOR/零效果 quirk。

use game_core::wave_record::*;

/// 八关 17 条 op5 波次流 (C9, CA) 全谱 golden（wave_stream_decode 实测）。
const CA_GOLDEN: [(u8, u8); 17] = [
    (0xF2, 0x00), // S1 $A3CD 流1
    (0xFB, 0x05), // S1 $A3E4 流2
    (0xF2, 0x00), // S2 $B1CE
    (0xFB, 0x31), // S2 $B1E5
    (0xF2, 0x70), // S3 $A5BF
    (0xFB, 0x31), // S3 $A5DC
    (0xE2, 0x00), // S4 $AF51
    (0xFB, 0x01), // S4 $AF68
    (0xF2, 0x70), // S5 $B7D7
    (0xFB, 0x31), // S5 $B801
    (0xE2, 0x00), // S6 $A5D9
    (0xFB, 0x01), // S6 $A5F0
    (0xFB, 0xB1), // S7 $AF7F
    (0xF2, 0x70), // S8 $B830
    (0xF2, 0x70), // S8 $B851
    (0xF2, 0x70), // S8 $B86D
    (0xFB, 0x81), // S8 $B889
];

/// ① CA 八位判定 golden（逐位语义表 §2）：代表性值全字段断言。
#[test]
fn ca_judgment_golden_representative() {
    // CA=$70（S3 流1，bits4,5,6）：臂门否、速度恢复（b7=0≠C9 b1=1）、
    // 写表1 组合判真（$70&$30==$30）、写表2 真（bit6）
    let j = ca_judgment(0xF2, 0x70, 0x00, 0x02, 0x00);
    assert_eq!(
        j,
        CaJudgment {
            arm_stage_end_gate: false,
            d1_clear: false,
            aux_ed56_pending: false,
            speed_restore: true,
            restore_scroll_pair: false,
            force_c8_6_request: false,
            bail_rts: false,
            write_table1_dispatch: true,
            write_table1_exec: true,
            write_table2: true,
        }
    );
    // CA=$B1（S7，bits0,4,5,7）：臂门真、速度恢复**否**（b7=1==C9 b1=1）、写表1 真、写表2 否
    let j = ca_judgment(0xFB, 0xB1, 0x00, 0x02, 0x00);
    assert!(j.arm_stage_end_gate && j.d1_clear, "bit0 主支双效果（$26|=关末门 + $D1:=0）");
    assert!(!j.speed_restore, "b7==C9 b1 ⇒ 跳过恢复段");
    assert!(j.write_table1_dispatch && j.write_table1_exec);
    assert!(!j.write_table2, "$B1 bit6=0");
    // CA=$81（S8 流4，bits0,7）：写表1 组合判**假**（$81&$30=$10≠$30）
    let j = ca_judgment(0xFB, 0x81, 0x00, 0x00, 0x00);
    assert!(!j.write_table1_dispatch, "$81 仅 bit4 ⇒ 组合判不成立");
    assert!(!j.speed_restore, "b7=1==C9 b1=1 ⇒ 不恢复");
    // CA=$31（bits0,4,5）：写表1 真、写表2 否
    let j = ca_judgment(0xFB, 0x31, 0x00, 0x00, 0x00);
    assert!(j.write_table1_dispatch);
    assert!(!j.write_table2);
    // CA=$05（S1 流2，bits0,2）：恢复滚动对**否**（bit1=0）、bit3 否
    let j = ca_judgment(0xFB, 0x05, 0x00, 0x00, 0x00);
    assert!(!j.restore_scroll_pair);
    assert!(!j.force_c8_6_request);
    assert!(j.speed_restore, "b7=0≠C9 b1=1 ⇒ 恢复");
    // CA=$00：全静默除速度恢复
    let j = ca_judgment(0xF2, 0x00, 0x00, 0x00, 0x00);
    assert_eq!(
        (j.arm_stage_end_gate, j.write_table1_dispatch, j.write_table2, j.speed_restore),
        (false, false, false, true)
    );
}

/// ② 17/17 全谱不变量（§3 相关性表）+ aux 派生式。
#[test]
fn ca_judgment_full_spectrum_invariants() {
    for &(c9, ca) in &CA_GOLDEN {
        let j = ca_judgment(c9, ca, 0x00, 0x00, 0x00);
        // bit0 主支：臂门与 $D1:=0 同支同真值
        assert_eq!(j.arm_stage_end_gate, ca & 1 != 0, "C9={c9:#X} CA={ca:#X}");
        assert_eq!(j.d1_clear, j.arm_stage_end_gate, "第二效果与臂门同支");
        // bit7 XOR bit1：速度恢复 = b7≠C9b1（八关 C9 bit1 恒 1）
        assert_eq!(j.speed_restore, (ca & 0x80 != 0) != (c9 & 0x02 != 0));
        // 写表1 组合判 = AND#$30==#$30（非 OR）
        assert_eq!(j.write_table1_dispatch, (ca & 0x30) == 0x30);
        // 写表2 = bit6
        assert_eq!(j.write_table2, ca & 0x40 != 0);
        // 全谱观测：bit7 仅 $B1/$81（与 §3 观测面一致）
        if ca & 0x80 != 0 {
            assert!(matches!(ca, 0xB1 | 0x81), "bit7 观测面仅 2 条，出现 {ca:#X}");
        }
    }
}

/// ③ sub_8DBD 真值表（SEC ⟺ C9 bit2=0 ∧ C9(bit0|bit3)≠0）+ aux 副支合成。
#[test]
fn sub_8dbd_and_aux_branch() {
    // (bit0,bit2,bit3) 8 组合真值表
    let expect = |c9: u8| c9 & 0x04 == 0 && (c9 & 0x09) != 0;
    for c9 in 0..=0x0Fu8 {
        assert_eq!(sub_8dbd_sec(c9), expect(c9), "c9={c9:#X}");
    }
    assert!(sub_8dbd_sec(0xFB), "$FB（bit0|3 置、bit2 清）⇒ SEC");
    assert!(!sub_8dbd_sec(0xF2), "$F2（bit0|3 清）⇒ CLC");
    // aux 副支合成：CA bit0=0 ∧ SEC ∧ $26&$88==0 ⇒ $ED56 挂 pending
    // 合成可达例：CA=$00/C9=$FB/$26=0（全谱无此对——判定式合成）
    let j = ca_judgment(0xFB, 0x00, 0x00, 0x00, 0x00);
    assert!(j.aux_ed56_pending, "bit0=0+SEC+$26 干净 ⇒ aux pending");
    // 三闸逐一关断
    let j2 = ca_judgment(0xFB, 0x01, 0x00, 0x00, 0x00);
    assert!(!j2.aux_ed56_pending, "bit0=1 关断（主支互斥）");
    let j3 = ca_judgment(0xFB, 0x00, 0x88, 0x00, 0x00);
    assert!(!j3.aux_ed56_pending, "$26&$88≠0 关断");
    let j4 = ca_judgment(0xF2, 0x00, 0x00, 0x00, 0x00);
    assert!(!j4.aux_ed56_pending, "sub_8DBD CLC 关断");
}

/// ④ bit4&5 **组合判**哨兵（AND#$30==#$30，非 OR——单 bit 合成用例必红）+ bit3/bail 语义。
#[test]
fn ca_combined_dispatch_and_bit3_negatives() {
    // 单 bit4 / 单 bit5 ⇒ 组合判假（OR 判据必红）
    let j = ca_judgment(0xFB, 0x10, 0x00, 0x00, 0x00);
    assert!(!j.write_table1_dispatch, "仅 bit4 ⇒ 不分派");
    let j = ca_judgment(0xFB, 0x20, 0x00, 0x00, 0x00);
    assert!(!j.write_table1_dispatch, "仅 bit5 ⇒ 不分派");
    // 双 bit ⇒ 真
    assert!(ca_judgment(0xFB, 0x30, 0x00, 0x00, 0x00).write_table1_dispatch);
    // 写表1 执行还受 $CB bit1 门（分派≠执行）
    let j = ca_judgment(0xFB, 0x31, 0x00, 0x00, 0x00);
    assert!(j.write_table1_dispatch && !j.write_table1_exec, "$CB bit1=0 ⇒ 只分派不执行");
    let j = ca_judgment(0xFB, 0x31, 0x00, 0x02, 0x00);
    assert!(j.write_table1_exec, "$CB bit1=1 ⇒ 执行");
    // bit3：请求 force $C8:=6；$C8==6 已等 ⇒ bail RTS（短路）
    let j = ca_judgment(0xFB, 0x08, 0x00, 0x00, 0x06);
    assert!(j.force_c8_6_request && j.bail_rts, "bit3+$C8==6 ⇒ RTS 短路");
    let j = ca_judgment(0xFB, 0x08, 0x00, 0x00, 0x05);
    assert!(j.force_c8_6_request && !j.bail_rts, "$C8≠6 ⇒ 落写");
    let j = ca_judgment(0xFB, 0x00, 0x00, 0x00, 0x03);
    assert!(!j.bail_rts, "无 bit3 不 bail");
}

/// ⑤ C9 bit5 死分支 quirk（BUG-021，D4 保真）：判定形态保留、零效果。
#[test]
fn c9_bit5_dead_branch_quirk_zero_effect() {
    // 探测函数保留判定形态（AND #$20/F0 00 的 BEQ 条件）
    assert_eq!(c9_bit5_dead_branch_probe(0xF2), 0xF2 & 0x20 == 0);
    assert_eq!(c9_bit5_dead_branch_probe(0xD2), true, "bit5 清 ⇒ BEQ 条件真");
    assert_eq!(c9_bit5_dead_branch_probe(0xF2), false, "bit5 置 ⇒ 假");
    // 零效果性质：bit5 翻转不改变判定链出参（全谱 17 对逐一对拍）
    for &(c9, ca) in &CA_GOLDEN {
        let a = ca_judgment(c9, ca, 0x11, 0x02, 0x06);
        let b = ca_judgment(c9 ^ 0x20, ca, 0x11, 0x02, 0x06);
        assert_eq!(a, b, "bit5 翻转零效果（C9={c9:#X} CA={ca:#X}）");
    }
}

/// ⑥ $D86B 状态转移表：字节精确（LDA $D86B,idx 读值）+ 锚点 + 表尾注。
#[test]
fn d86b_transition_table_byte_exact() {
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM 可读");
    let off = 0x10 + 14 * 0x2000 + (0xD86B & 0x1FFF);
    let rom_tbl = &rom[off..off + 256];
    assert_eq!(STATE_TRANSITION.len(), 256);
    for i in 0..256usize {
        assert_eq!(STATE_TRANSITION[i], rom_tbl[i], "表项 {i:#X} 字节精确");
    }
    // 锚点（arena-midboss §2.2 实测转移）
    assert_eq!(state_transition(0x01), 0x0F, "玩家受击 → dead");
    assert_eq!(state_transition(0x22), 0x12, "召唤物受击相位切换");
    assert_eq!(state_transition(0x4A), 0x4A, "中 boss 恒等（不靠此表死亡）");
    assert_eq!(state_transition(0x5F), 0x17);
    assert_eq!(state_transition(0x02), 0x02);
    assert_eq!(state_transition(0x12), 0x12);
    // 负向哨兵：$01 非恒等（identity 映射必红）
    assert_ne!(state_transition(0x01), 0x01);
}
