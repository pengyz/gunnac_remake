//! game-core::wave 敌波调度器测试（B1 移植，TDD）。
//!
//! 证据基准：`docs/findings/enemy-timeline.md`——
//! §1.3 记录机 bank00:86CE 字节语义（A 级逐 handler 清点）；§2.2 STG-01 调度流
//! bank8:$A26D 逐记录解码（A 级）；§8.1 Wt-1 tick 定谳（delay 1 tick=8f 稳态；
//! 关首快进环 17 tick 压入 f1193，rec1 的 32 tick=环内 16+稳态 16 全额兑现）。
//! golden 复现：`python3 tools/wave_timeline.py sched 1`（记录/累计 tick 全表）。
//! 先测试后实现（TDD）；负向哨兵锁死口径（+1 quirk / delay 1:1 / 未知 ctrl fail-closed /
//! 自环不展开）。

use game_core::wave::*;

/// bank8 = $8000 窗 → file 0x10+8×0x2000（同 tests/stage_content.rs 口径）。
fn bank8_window() -> Vec<u8> {
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM 可读");
    rom[0x10 + 8 * 0x2000..0x10 + 9 * 0x2000].to_vec()
}

/// STG-01 调度流 43 记录 (addr, 触发累计 tick) golden（`wave_timeline.py sched 1` 全表）。
const STG01_RECORDS: [(u16, u32); 43] = [
    (0xA26D, 1),
    (0xA275, 33),
    (0xA27C, 49),
    (0xA283, 65),
    (0xA28A, 81),
    (0xA291, 89),
    (0xA299, 97),
    (0xA2A0, 113),
    (0xA2A7, 129),
    (0xA2AC, 137),
    (0xA2B3, 145),
    (0xA2BA, 161),
    (0xA2BF, 177),
    (0xA2C9, 257),
    (0xA2CE, 289),
    (0xA2D5, 337),
    (0xA2DC, 369),
    (0xA2E1, 385),
    (0xA2E5, 417),
    (0xA2EC, 545),
    (0xA2F3, 553),
    (0xA2FA, 569),
    (0xA301, 577),
    (0xA308, 705),
    (0xA30F, 769),
    (0xA316, 785),
    (0xA31D, 793),
    (0xA324, 865),
    (0xA32B, 881),
    (0xA332, 889),
    (0xA339, 905),
    (0xA345, 953),
    (0xA34C, 969),
    (0xA353, 977),
    (0xA35A, 993),
    (0xA366, 1009),
    (0xA36D, 1025),
    (0xA374, 1033),
    (0xA37B, 1049),
    (0xA382, 1089),
    (0xA389, 1121),
    (0xA390, 1137),
    (0xA397, 1237),
];

/// ① 记录机解码 golden：43 记录地址+触发 tick 逐条吻合；自环 @A397 停止展开。
#[test]
fn sched_decode_stg01_records_and_ticks_golden() {
    let w = bank8_window();
    let s = decode_sched(&w, 0xA26D, 64);
    assert_eq!(s.records.len(), 43, "43 条记录（工具全表）");
    assert!(!s.truncated, "视界未截断");
    assert_eq!(s.unknown_ctrl, None, "STG-01 全流无未证 ctrl");
    assert_eq!(s.loop_target, Some(0xA397), "[9]JMP$A397 自环");
    assert_eq!(s.loop_tick, Some(1337), "自环下一圈触发 tick（1237+100）");
    for (i, &(addr, tick)) in STG01_RECORDS.iter().enumerate() {
        assert_eq!(s.records[i].addr, addr, "记录 {i} 地址");
        assert_eq!(s.records[i].tick, tick, "记录 {i} 触发累计 tick");
    }
}

/// ② 项解码 golden：ctrl/pair 逐字节语义（记录机 handler 清点口径）。
#[test]
fn sched_decode_stg01_items_golden() {
    let w = bank8_window();
    let s = decode_sched(&w, 0xA26D, 64);
    // 首记录 @A26D：[13]reg(1C,02) [14]reg(04,67)（音画寄存器写，登记不解释）
    assert_eq!(
        s.records[0].items,
        vec![
            SchedItem::Reg { ctrl: 13, a: 0x1C, b: 0x02 },
            SchedItem::Reg { ctrl: 14, a: 0x04, b: 0x67 },
        ]
    );
    assert_eq!(s.records[0].delay, 0x00, "流头 delay 字节");
    // @A275：[2]T=$2F(2B,06,0A,20)——f1341 组长签名记录（四参=组员type/数/间隔f/参数）
    assert_eq!(
        s.records[1].items,
        vec![SchedItem::Spawn {
            ctrl: 2,
            type_: 0x2F,
            args: [0x2B, 0x06, 0x0A, 0x20]
        }]
    );
    assert_eq!(s.records[1].delay, 0x20, "rec1 gap=32 tick（Wt-1 全额兑现口径）");
    // @A2A7：[10] 流内定型（type 取自流字节、param=第 2 操作数）
    assert_eq!(
        s.records[8].items,
        vec![SchedItem::SetType { type_: 0x2A, param: 0x00 }]
    );
    // @A2E1：全流唯一 (type,param) 对（type≥$10 直接出敌项）
    assert_eq!(
        s.records[17].items,
        vec![SchedItem::Pair { type_: 0x44, param: 0x00 }]
    );
    // @A339：双 [1] 项同记录（两条 $2E 记录连发）
    assert_eq!(
        s.records[30].items,
        vec![
            SchedItem::Spawn { ctrl: 1, type_: 0x2E, args: [0x4D, 0x04, 0x28, 0x0E] },
            SchedItem::Spawn { ctrl: 1, type_: 0x2E, args: [0x4D, 0x04, 0x28, 0x8E] },
        ]
    );
    // @A397：[9] 流跳转（目标=自址）
    assert_eq!(
        s.records[42].items,
        vec![SchedItem::Jump { ctrl: 9, target: 0xA397 }]
    );
}

/// ③ tick→帧映射（Wt-1 口径）+ trace 帧锚（findings §2.2/§2.3）。
#[test]
fn sched_stg01_tick_to_frame_anchors() {
    // 关首快进环：tick 1..=17 全部压入 f1193 单视频帧
    assert_eq!(stg01_tick_to_frame(1), 1193);
    assert_eq!(stg01_tick_to_frame(17), 1193);
    // 稳态：tick 18+k → f1221+8k（1 tick=8f；28f 相位=Wt-1' 残余口径）
    assert_eq!(stg01_tick_to_frame(18), 1221);
    // trace 帧锚（§2.2 表 + §2.3 校验点）
    assert_eq!(stg01_tick_to_frame(33), 1341, "rec@$A275 组长锚 f1341");
    assert_eq!(stg01_tick_to_frame(49), 1469);
    assert_eq!(stg01_tick_to_frame(65), 1597);
    assert_eq!(stg01_tick_to_frame(81), 1725);
    assert_eq!(stg01_tick_to_frame(89), 1789);
    assert_eq!(stg01_tick_to_frame(129), 2109);
    assert_eq!(stg01_tick_to_frame(161), 2365);
}

/// ④ 负向哨兵：口径错位必红（mutation 可咬）。
#[test]
fn sched_negative_sentinels() {
    let w = bank8_window();
    let s = decode_sched(&w, 0xA26D, 64);
    // 哨兵①：首记录 tick=delay+1（sub_86A0 `$51:=(@0)+1`）——丢 +1 必红
    assert_eq!(s.records[0].tick, 1);
    assert_ne!(s.records[0].tick, 0, "丢 +1 quirk 必红");
    // 哨兵②：后续 delay 1:1 装载（86BD `B1 14 85 51` 无移位无 +1）——31/33 必红
    assert_eq!(s.records[1].tick, 33);
    assert_ne!(s.records[1].tick, 32, "多算/少算 gap 必红");
    assert_ne!(s.records[1].tick, 34);
    // 哨兵③：自环不得展开（43 条 + 停止；无限展开/少条必红）
    assert_eq!(s.loop_target, Some(0xA397));
    assert_ne!(s.records.len(), 44, "自环再展开必红");
    // 哨兵④：未知 ctrl fail-closed（ctrl 15 无 handler——不得当 pair/记录尾续解）
    let synth = [0x05u8, 0x0F, 0x44, 0x10, 0x00];
    let s2 = decode_sched(&synth, 0xA000, 8);
    assert_eq!(s2.unknown_ctrl, Some(15), "未证 ctrl 必须显式 fail-closed");
    assert_eq!(s2.records.len(), 1, "未知 ctrl 后不得续解");
    assert_eq!(s2.records[0].items, vec![SchedItem::Unknown { ctrl: 15 }]);
    // 哨兵⑤：pair/ctrl 分界——type≥$10 是 pair（两字节），< $10 才是 ctrl
    let synth2 = [0x00u8, 0x10, 0x22, 0x00, 0x00];
    let s3 = decode_sched(&synth2, 0xA000, 8);
    assert_eq!(
        s3.records[0].items,
        vec![SchedItem::Pair { type_: 0x10, param: 0x22 }],
        "0x10 起头=pair（误当 ctrl 必红）"
    );
    assert_eq!(s3.records.len(), 2, "ctrl0 记录分隔后续解第 2 条（空项）");
}
