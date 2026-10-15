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

/// bank8 = $A000 数据窗 → file 0x10+8×0x2000（数据 bank 映射 CPU $A000-$BFFF；同 stage_content.rs 口径）。
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

// ===== 批次2：记录处理出参（L2 对拍面）+ 组员帧域节律 =====

/// ⑤ 四参写链 golden：双关签名法（stg02 §3.2 10/10 + B1 §2.2）——
/// ctrl→leader_type 映射、四参原值、ctrl2/12 码流读、step 判据=type&1。
#[test]
fn quad_write_golden_cross_stage() {
    // STG-01 @A275 [2]T=$2F(2B,06,0A,20)：ctrl2 ⇒ $2F、读码流、step（$2F&1=1）
    let e = item_effect(
        &SchedItem::Spawn { ctrl: 2, type_: 0x2F, args: [0x2B, 0x06, 0x0A, 0x20] },
        0x77,
    );
    assert_eq!(
        e,
        ItemEffect::QuadWrite {
            leader_type: 0x2F,
            args: [0x2B, 0x06, 0x0A, 0x20],
            write_48c: Some(0x77),
            code_stream_step: true,
        }
    );
    // STG-02 @AEF1 [1]T=$2E(56,08,1E,00)：ctrl1 ⇒ $2E、不读码流、不 step（$2E&1=0）
    let e = item_effect(
        &SchedItem::Spawn { ctrl: 1, type_: 0x2E, args: [0x56, 0x08, 0x1E, 0x00] },
        0x99,
    );
    assert_eq!(
        e,
        ItemEffect::QuadWrite {
            leader_type: 0x2E,
            args: [0x56, 0x08, 0x1E, 0x00],
            write_48c: None,
            code_stream_step: false,
        },
        "ctrl1 不读码流（write_48c=None）"
    );
    // ctrl11/12 → $2C/$2D（872F/877F LDA #imm）
    assert_eq!(
        item_effect(&SchedItem::Spawn { ctrl: 11, type_: 0x2C, args: [1, 2, 3, 4] }, 0),
        ItemEffect::QuadWrite {
            leader_type: 0x2C,
            args: [1, 2, 3, 4],
            write_48c: None,
            code_stream_step: false
        }
    );
    assert_eq!(
        item_effect(&SchedItem::Spawn { ctrl: 12, type_: 0x2D, args: [1, 2, 3, 4] }, 0x42),
        ItemEffect::QuadWrite {
            leader_type: 0x2D,
            args: [1, 2, 3, 4],
            write_48c: Some(0x42),
            code_stream_step: true
        }
    );
}

/// ⑥ 其余 ctrl 路径出参（stg02 §6.6 判据：含 ctrl0/10/13/14 路径）。
#[test]
fn item_effect_ctrl10_pair_reg_paths() {
    // ctrl10：type/param 双写 + 码流读 + **无条件** step（87C4-87C8）
    assert_eq!(
        item_effect(&SchedItem::SetType { type_: 0x2A, param: 0x00 }, 0x5A),
        ItemEffect::SetTypeWrite { type_: 0x2A, param: 0x00, write_48c: 0x5A, code_stream_step: true }
    );
    // pair：$400+槽/$470+槽 两写
    assert_eq!(
        item_effect(&SchedItem::Pair { type_: 0x44, param: 0x00 }, 0),
        ItemEffect::PairWrite { type_: 0x44, param: 0x00 }
    );
    // ctrl13/14 透传；ctrl0/jump 无写集；未证 ctrl fail-closed
    assert_eq!(
        item_effect(&SchedItem::Reg { ctrl: 13, a: 0x1C, b: 0x02 }, 0),
        ItemEffect::RegWrite { ctrl: 13, a: 0x1C, b: 0x02 }
    );
    assert_eq!(item_effect(&SchedItem::Jump { ctrl: 9, target: 0xA397 }, 0), ItemEffect::NoWrite);
    assert_eq!(item_effect(&SchedItem::Unknown { ctrl: 15 }, 0), ItemEffect::Unproven);
}

/// ⑦ 组员帧域节律 golden（B1 §2.3 + stg02 §3.3）：首员与组长同帧、$534 直接以帧兑现。
#[test]
fn member_schedule_frame_domain_golden() {
    // STG-01 rec@A28A (4E,5,0F,89)：f1725 起 5 员 ×15f（f1770 槽复用不可见——判定式口径）
    assert_eq!(
        member_schedule(0x05, 0x0F, 1725),
        Some(vec![1725, 1740, 1755, 1770, 1785])
    );
    // STG-01 rec@A275 (2B,6,0A,20)：f1341 起 6 员 ×10f
    assert_eq!(
        member_schedule(0x06, 0x0A, 1341),
        Some(vec![1341, 1351, 1361, 1371, 1381, 1391])
    );
    // STG-02 rec@AEF1 (56,08,1E,00)：间隔 30f=$1E 直接兑现（首员=组长帧 f3776）
    assert_eq!(
        member_schedule(0x08, 0x1E, 3776),
        Some(vec![3776, 3806, 3836, 3866, 3896, 3926, 3956, 3986])
    );
}

/// ⑧ `$630` 首现=数−1 + 状态机节律 + 负向哨兵（mutation 可咬）。
#[test]
fn group_release_first_seen_and_negatives() {
    // 首现 = count−1（装载后先递减一次，stg02 §3.4）
    let (mut g, first) = GroupRelease::load(6, 10).expect("count=6 合法");
    assert_eq!(first, 5, "$630 首现=数−1");
    assert_ne!(first, 6, "首现=数 必红（未递减）");
    assert_eq!(g.remaining, 5);
    // 节律：同帧投首员（load 即投），其后每 10f 一员，共 6 员
    let mut frames = vec![0u32]; // 首员=装载帧
    for f in 1..=50 {
        if g.tick_frame() {
            frames.push(f);
        }
    }
    assert_eq!(frames, vec![0, 10, 20, 30, 40, 50], "帧域间隔 10f 直接兑现");
    assert_eq!(g.remaining, 0, "6 员投尽");
    assert!(!g.tick_frame(), "余 0 不再投");
    // 哨兵①：间隔是**帧**不是 tick（×8 换算必红）
    let (mut g2, _) = GroupRelease::load(2, 15).unwrap();
    let hit: Vec<u32> = (1..=15).filter(|_| g2.tick_frame()).collect();
    assert_eq!(hit, vec![15], "15 帧整，非 15×8=120");
    // 哨兵②：count=0 / interval=0 未观测 ⇒ fail-closed None
    assert_eq!(GroupRelease::load(0, 10), None);
    assert_eq!(GroupRelease::load(6, 0), None);
    assert_eq!(member_schedule(0, 10, 0), None);
    assert_eq!(member_schedule(6, 0, 0), None);
}

/// ⑨ tick 对账规则（A 级 10/10，stg02 §3.4）：trace `$84` = 工具累计 `+Nt` − 1。
#[test]
fn trace_tick_accounting_rule() {
    for (plus_n, s84) in [(20u32, 0x13u32), (31, 0x1E), (57, 0x38), (97, 0x60)] {
        assert_eq!(plus_n - 1, s84, "+Nt={plus_n} ↔ $84={s84:#X}");
    }
    // 复用到 STG-01 记录：tick 字段即工具 +Nt，trace 值=tick−1
    let w = bank8_window();
    let s = decode_sched(&w, 0xA26D, 64);
    assert_eq!(s.records[1].tick, 33);
    assert_eq!(s.records[1].tick - 1, 32, "$84 口径 = tick−1");
}

/// ⑩ F-B4 合成交叉哨兵（#84 必修）：step 判据=**type&1**（8763-876E `LDA $13/AND #1`），
/// 非 ctrl 判据——可达输入中 ctrl1↔$2E(偶)/ctrl2↔$2F(奇) 恒等价（decode 内绑定），
/// 合成 `ctrl:1 + type_:0x2F` 交叉对打破等价：type&1 ⇒ step=true，ctrl 判据 ⇒ false。
#[test]
fn step_criterion_is_type_bit_not_ctrl_synthetic() {
    // 合成交叉（decode 不可达，仅证判据字节语义）：ctrl=1 但 type=$2F 奇数
    let e = item_effect(
        &SchedItem::Spawn { ctrl: 1, type_: 0x2F, args: [1, 2, 3, 4] },
        0,
    );
    assert_eq!(
        e,
        ItemEffect::QuadWrite {
            leader_type: 0x2F,
            args: [1, 2, 3, 4],
            write_48c: None,
            code_stream_step: true,
        },
        "step 判据=type&1（$2F 奇 ⇒ true）"
    );
    // 反向合成：ctrl=2 但 type=$2E 偶数 ⇒ step=false（ctrl 判据必红）
    let e2 = item_effect(
        &SchedItem::Spawn { ctrl: 2, type_: 0x2E, args: [1, 2, 3, 4] },
        0x55,
    );
    assert_eq!(
        e2,
        ItemEffect::QuadWrite {
            leader_type: 0x2E,
            args: [1, 2, 3, 4],
            write_48c: Some(0x55), // ctrl2 读码流判据仍是 ctrl（8783-8793 附加支入口）
            code_stream_step: false,
        },
        "type 偶 ⇒ 不 step"
    );
}

// ===== 组员节律帧锚对拍面（lead 待料指令：stg0X 各关锚 vs GroupRelease 出参）=====

/// ⑫ 跨关节律帧锚对拍（stg02 §3.3 双关四例 + B1 §2.3 三簇）：
/// member_schedule 出参须**覆盖**实测锚；两 API（schedule/GroupRelease）出参自洽。
#[test]
fn member_rhythm_cross_stage_anchors() {
    use game_core::wave::GroupRelease;
    // 锚表：(组员type,数,间隔f, 参数, leader_frame, 实测帧锚子集)
    let anchors: [([u8; 4], u32, &[u32]); 4] = [
        // STG-01 rec@A275 (2B,6,0A,20)：f1341-1391 六连 +10f（B1 §2.3）
        ([0x2B, 6, 0x0A, 0x20], 1341, &[1341, 1351, 1361, 1371, 1381, 1391]),
        // STG-01 rec@A27C (2B,6,0A,C0)：f1469-1519 六连 +10f
        ([0x2B, 6, 0x0A, 0xC0], 1469, &[1469, 1479, 1489, 1499, 1509, 1519]),
        // STG-02 rec@AEEA (2B,6,0A,30)：f3580-3630 六连 +10f（stg02 §3.3）
        ([0x2B, 6, 0x0A, 0x30], 3580, &[3580, 3590, 3600, 3610, 3620, 3630]),
        // STG-02 rec@AEF1 (56,08,1E,00)：f3806/3836/3866/3896/3926 = 30f=$1E
        //（实测子集 k=1..5；首员=组长帧 f3776 同帧模型，观测 0→非0 口径见注）
        ([0x56, 8, 0x1E, 0x00], 3776, &[3806, 3836, 3866, 3896, 3926]),
    ];
    for (args, leader_frame, observed) in anchors {
        let [_, count, interval, _] = args;
        let sched = member_schedule(count, interval, leader_frame).expect("合法四参");
        for &f in observed {
            assert!(
                sched.contains(&f),
                "锚 f{f} 必须 ∈ 出参（args={args:?} leader={leader_frame}）"
            );
        }
        // 出参=等差帧序列（间隔=间隔字节**直接兑现**）
        for w in sched.windows(2) {
            assert_eq!(w[1] - w[0], u32::from(interval), "帧域直接兑现（args={args:?}）");
        }
        assert_eq!(sched.len(), usize::from(count), "员数=组员数");
        assert_eq!(sched[0], leader_frame, "首员与组长同帧（B1 §2.3 双证）");
        // 两 API 自洽：GroupRelease 逐步出帧 == member_schedule 全序列
        let (mut g, first) = GroupRelease::load(count, interval).expect("合法");
        assert_eq!(first, count - 1, "$630 首现=数−1");
        let mut released = vec![leader_frame];
        for k in 1..u32::from(count) * u32::from(interval) {
            if g.tick_frame() {
                released.push(leader_frame + k);
            }
        }
        assert_eq!(released, sched, "GroupRelease 与 member_schedule 出参自洽");
    }
    // 负向哨兵：间隔是帧非 tick（×8 必红）——锚 30f 若 ×8=240f 锚全失
    let bad = member_schedule(8, 30 * 8, 3776).unwrap();
    for &f in &[3806u32, 3836, 3866, 3896, 3926] {
        assert!(!bad.contains(&f), "×8 换算必红（f{f} 不得出现在换算序列）");
    }
    // STG-03 帧锚未采（S3-2 挂账）——只证值域形状不证帧
    let s3 = member_schedule(4, 0x0F, 0).unwrap();
    assert_eq!(s3, vec![0, 15, 30, 45], "STG-03 主导 15f 形状（帧锚待采不作锚断言）");
}
