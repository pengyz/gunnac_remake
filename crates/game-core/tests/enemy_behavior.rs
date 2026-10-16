//! game-core::enemy 敌型行为面测试（批次4b，TDD）。
//!
//! 证据基准：`docs/findings/enemy-timeline.md` §3（敌型/弹幕特征表，A/B 级——
//! 行为指针+字段签名）+ §2.3（120 事件坐标锚）+ `docs/findings/arena-midboss.md`
//! §2.2/§3（中 boss HP 语义链，A 级）+ `docs/findings/stage-wave-crosscut.md` §3
//! （移植清单）。**型值不命名**（对象类型表值）；行为 VM 脚本体（t59 族）未证不发明。

use game_core::enemy::*;

/// ⑦ spawn 三写字节 golden（bank00:8B89-8B95 同帧三写 $400/$438/$41C）+
/// hi/lo 交换哨兵（行为指针读法 $41C=hi / $438=lo）。
#[test]
fn spawn_triple_write_golden() {
    // ROM 实例：8B89 `LDA #$19/STA $400,Y` + `LDA #$B0/STA $438,Y` + `LDA #$8D/STA $41C,Y`
    let t = spawn_triple_write(0x19, 0x8DB0);
    assert_eq!(t.type_code, 0x19);
    assert_eq!(t.beh_hi, 0x8D, "$41C+槽:=hi（LDA #$8D/STA $41C）");
    assert_eq!(t.beh_lo, 0xB0, "$438+槽:=lo（LDA #$B0/STA $438）");
    // 交换哨兵：hi/lo 反装必红
    assert_ne!(t.beh_hi, t.beh_lo);
    let t2 = spawn_triple_write(0x2F, 0x89DF);
    assert_eq!((t2.beh_hi, t2.beh_lo), (0x89, 0xDF), "$89DF ⇒ hi:$89 lo:$DF");
    assert_ne!((t2.beh_hi, t2.beh_lo), (0xDF, 0x89), "hi/lo 交换必红");
}

/// ⑧ 型签名登记表（行为指针+字段签名驱动，型值语义不命名）：
/// B1 §3 trace 实测 (type → 行为指针/出生坐标)；$2E 行为指针**未观测**不发明。
#[test]
fn type_signature_registry() {
    // $2F：$89DF、x=128,y=8 恒定（B1 §3/§2.3 trace 9 签名）
    let s = type_signatures(0x2F);
    assert_eq!(s.len(), 1);
    assert_eq!((s[0].beh, s[0].spawn), (BehaviorField::BehPtr(0x89DF), Some((128, 8))));
    // $2A：$9010、(29,10)（f1789/2109/2365 三点）
    let s = type_signatures(0x2A);
    assert_eq!((s[0].beh, s[0].spawn), (BehaviorField::BehPtr(0x9010), Some((29, 10))));
    // $2B/$4E 组员族：【c4e6b68】$DC=表分发种子（出生帧瞬态快照，B1"恒 $00DC"作废）；坐标 ⇒ None
    for t in [0x2Bu8, 0x4E] {
        let s = type_signatures(t);
        assert_eq!(s[0].beh, BehaviorField::DispatchSeed(0xDC), "c4e6b68 改标：种子非指针");
        assert_eq!(s[0].spawn, None, "成员坐标摆动不登记恒定值");
    }
    // $07 三行为指针（底部点 $A631 / 尾段流 $AB30 / $AAD9）——同型多指针按观测对登记
    let s = type_signatures(0x07);
    let ptrs: Vec<u16> = s
        .iter()
        .map(|x| match x.beh {
            BehaviorField::BehPtr(p) => p,
            BehaviorField::DispatchSeed(_) => panic!("$07 须 BehPtr"),
        })
        .collect();
    assert_eq!(ptrs, vec![0xA631, 0xAB30, 0xAAD9], "同型多指针均登记（勿 1:1 强绑）");
    // $02 编队：$AB74（5 槽同帧 216f 轮）
    assert_eq!(type_signatures(0x02)[0].beh, BehaviorField::BehPtr(0xAB74));
    // 扩表观测对（trace spawns 全量）：$2A→$9079、$2B→$0079、$05→$B011
    assert!(type_signatures(0x2A).iter().any(|s| s.beh == BehaviorField::BehPtr(0x9079)));
    assert!(type_signatures(0x2B).iter().any(|s| s.beh == BehaviorField::DispatchSeed(0x79)));
    assert_eq!(type_signatures(0x05)[0].beh, BehaviorField::BehPtr(0xB011));
    // 恒定坐标仅 2 例（$2F/$2A 主指针）；其余 None
    for s in type_signatures(0x07) {
        assert_eq!(s.spawn, None, "$07 坐标不恒定不登记");
    }
    // beh=$0000 暂态**不登记**（非行为指针——登记即发明必红）
    for sig in TYPE_SIGNATURES {
        match sig.beh {
            BehaviorField::BehPtr(p) => assert_ne!(p & 0x8000, 0, "BehPtr 须 hi bit7=1"),
            BehaviorField::DispatchSeed(v) => assert_ne!(v, 0, "种子非 $00"),
        }
    }
    // $2E 组长族：【2026-10-15 补】$89E4@(128,8) 五点实测（stg02 §4.4 帧锚 run 补证）
    let s = type_signatures(0x2E);
    assert_eq!(s.len(), 1, "$2E 登记观测对（原空缺口已由 run 补上）");
    assert_eq!((s[0].beh, s[0].spawn), (BehaviorField::BehPtr(0x89E4), Some((128, 8))));
    // run 新增自向量对（hi bit7=1）
    assert!(type_signatures(0x58).iter().any(|x| x.beh == BehaviorField::BehPtr(0xB3B9)));
    assert!(type_signatures(0x51).iter().any(|x| x.beh == BehaviorField::BehPtr(0xB04C)));
    assert!(type_signatures(0x05).iter().any(|x| x.beh == BehaviorField::BehPtr(0xAE6B)));
    // c4e6b68 改标一致性：BehPtr 恒 hi bit7=1、DispatchSeed 恒 hi=$00（双类判据互斥）
    for sig in TYPE_SIGNATURES {
        match sig.beh {
            BehaviorField::BehPtr(p) => {
                assert_ne!(p & 0x8000, 0, "BehPtr 须 hi bit7=1（{p:#X}）")
            }
            BehaviorField::DispatchSeed(v) => {
                assert_ne!(v, 0, "种子非 $00（出生帧 $00 另义）");
            }
        }
    }
    // 未登记型值 ⇒ 空
    assert!(type_signatures(0x66).is_empty(), "清单型值 $66 不命名不发明");
}

/// ⑪ GroupRelease 集成（组长族四参→释放状态；批次2 节律面复用）：
/// STG-01 rec@A275 [2]T=$2F(2B,06,0A,20)：首现 $630=5、间隔 10f 帧域、6 员。
#[test]
fn leader_family_release_integration() {
    let (mut fam, first_seen) =
        leader_family_spawn([0x2B, 0x06, 0x0A, 0x20]).expect("count=6 合法");
    assert_eq!(fam.member_type, 0x2B, "$518 组员 type");
    assert_eq!(fam.param, 0x20, "$470 参数");
    assert_eq!(fam.release.interval_frames, 0x0A, "$534 帧域直接兑现");
    assert_eq!(first_seen, 5, "$630 首现=数−1");
    assert_eq!(fam.release.remaining, 5);
    // 节律：首员=装载帧（组员与组长同帧），其后 10f/员
    let mut frames = vec![0u32];
    for fr in 1..=50 {
        if fam.release.tick_frame() {
            frames.push(fr);
        }
    }
    assert_eq!(frames, vec![0, 10, 20, 30, 40, 50], "帧域 10f×6");
    // count=0 fail-closed（不发明）
    assert!(leader_family_spawn([0x2B, 0x00, 0x0A, 0x20]).is_none());
}

/// ⑨ 中 boss（type $4A）HP 语义链（arena-midboss §2.2/§3.2 A 级）：
/// HP 初值 14、弹命中 −2、物撞 −=对方 $0630、HP 归 0 ⇒ 死亡门（$D84F 族）；
/// `$D86B[$4A]=$4A` 恒等——不靠转移表死亡。
#[test]
fn midboss_hp_chain() {
    let mut m = Midboss::new();
    assert_eq!(m.hp, 14, "HP 初值 14（v2-diag f9930 快照）");
    assert_eq!(m.attr, 0x13, "属性掩码 $04DF=$13（快照）");
    assert_eq!(m.hit_cnt, 0x00, "$05F7 恒 0——**非 HP**（W18 反证收口）");
    assert!(!m.hit_shot(), "14→12 未死（f10080 实测锚）");
    assert_eq!(m.hp, 12);
    // 7 发弹命中即杀（14−2×7=0）
    let mut m2 = Midboss::new();
    let mut dead = false;
    for i in 0..7 {
        dead = m2.hit_shot();
        if i < 6 {
            assert!(!dead, "第 {} 发未死", i + 1);
        }
    }
    assert!(dead && m2.hp == 0, "第 7 发归 0 ⇒ 死亡门");
    // 物撞伤害=对方 $0630（非 −2）
    let mut m3 = Midboss::new();
    assert!(!m3.hit_body(5), "14−5=9 未死");
    assert_eq!(m3.hp, 9, "物撞 −=对方 HP");
    // HP 下溢饱和 0 不回绕
    let mut m4 = Midboss::new();
    assert!(m4.hit_body(20), "20>14 ⇒ 归 0 死亡");
    assert_eq!(m4.hp, 0);
    // $D86B[$4A]=$4A 恒等（中 boss 不靠转移表死亡——identity 突变必红）
    assert_eq!(game_core::wave_record::state_transition(0x4A), 0x4A);
    // 相位续点登记（$AC34/$AC75，B 级链——值 A 级）
    assert_eq!(Midboss::BEH_CONTINUATIONS, [0xAC34, 0xAC75]);
}

/// ⑩ 负向哨兵：伤害方向/饱和/签名空值（mutation 可咬）。
#[test]
fn enemy_behavior_negative_sentinels() {
    // 物撞按 −2 误算必红
    let mut m = Midboss::new();
    m.hit_body(2);
    assert_eq!(m.hp, 12, "物撞 2 点=对方 HP 2——恰与弹同值；换 5 点判别");
    let mut m2 = Midboss::new();
    m2.hit_body(5);
    assert_ne!(m2.hp, 12, "物撞 5 点若得 12=误用弹公式必红");
    // 弹命中 −2 非 −1/−对方
    let mut m3 = Midboss::new();
    m3.hit_shot();
    assert_eq!(m3.hp, 12);
    assert_ne!(m3.hp, 13, "−1 误算必红");
    // 已死后再击不回绕（饱和）
    let mut m4 = Midboss::new();
    m4.hit_body(20);
    assert!(!m4.hit_shot(), "hp=0 后 hit 保持 0 且已死态");
    assert_eq!(m4.hp, 0);
    // spawn 三写 type 不动、beh 拆分正确（type 混入 beh 必红）
    let t = spawn_triple_write(0x44, 0x00DC);
    assert_eq!(t.type_code, 0x44);
    assert_eq!((t.beh_hi, t.beh_lo), (0x00, 0xDC), "$00DC ⇒ hi:00 lo:DC");
}

// ===== 批次4c：组员族运动机（enemy-motion-kinematics §2-3）=====

/// ⑫ 运动机三表字节精确（bank05:$AC5F/$AC67/$AC6F）+ mod-8 步进 golden。
#[test]
fn motion_tables_byte_exact_and_step() {
    use game_core::enemy::{motion_step, TRI_WAVE_518, TRI_WAVE_534, TRI_WAVE_684};
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM 可读");
    for (addr, tbl, name) in [
        (0xAC5Fusize, &TRI_WAVE_684[..], "684"),
        (0xAC67, &TRI_WAVE_534[..], "534"),
        (0xAC6F, &TRI_WAVE_518[..], "518"),
    ] {
        let off = 0x10 + 5 * 0x2000 + (addr & 0x1FFF);
        assert_eq!(tbl, &rom[off..off + 8], "{name} 表 8B 字节精确");
    }
    // $684=对称三角波（B1 §3 复核 #76 F2 真值）
    assert_eq!(TRI_WAVE_684, [0xD3, 0xD7, 0xDB, 0xDF, 0xE3, 0xDF, 0xDB, 0xD7]);
    // 步进 golden：INC→mod8 索引三表写
    let o = motion_step(0x00, 0x00);
    assert_eq!(o.counter_next, 0x01);
    assert_eq!((o.v684, o.v534, o.v518), (0xD7, 0x43, 0x24), "idx=1 三表");
    // mod-8 回绕：counter=7→next=8→idx=0
    let o = motion_step(0x07, 0x00);
    assert_eq!(o.counter_next, 0x08);
    assert_eq!(o.v684, 0xD3, "idx=8&7=0 回绕首项");
    // F-C89 必修（#89 段B）：idx≥4 掩码宽度哨兵——idx∈{0,1} 采样下 &7→&3/&1 恒等
    //（掩码宽度面补咬；表常量已① 24B ROM 比对锁死）
    let o = motion_step(0x04, 0x00); // next=5 ⇒ idx=5（&3=1、&1=1 皆错位）
    assert_eq!(
        (o.counter_next, o.v684, o.v534, o.v518),
        (0x05, 0xDF, 0x03, 0x24),
        "idx=5 三表值（&7→&3 必红：idx1 值 0xD7/0x43/0x24）"
    );
    let o = motion_step(0x06, 0x00); // next=7 ⇒ idx=7（&3=3 错位）
    assert_eq!(
        (o.counter_next, o.v684, o.v534, o.v518),
        (0x07, 0xD7, 0x03, 0x24),
        "idx=7 三表值（&3 ⇒ idx3=0xDF/0x43/0x24 必红）"
    );
    // counter 8-bit 回绕（INC 全字节）：next=0 ⇒ idx=0 ⇒ 首项
    let o = motion_step(0xFF, 0x00);
    assert_eq!(o.counter_next, 0x00);
    assert_eq!(o.v684, 0xD3, "idx=0&7=0 ⇒ 表首项");
}

/// ⑬ 相位 EOR 只吃 bit0（$2B&1——全字节 EOR 必红）+ 字段复用两段生命周期。
#[test]
fn motion_phase_bit0_and_field_reuse() {
    use game_core::enemy::motion_step;
    // bit0 判据：$2B=0x02 与 $2B=0x00 同出参（全字节 EOR 必红）
    for c in 0..8u8 {
        assert_eq!(
            motion_step(c, 0x00),
            motion_step(c, 0x02),
            "仅 bit0 入 EOR（counter={c}）"
        );
        assert_eq!(
            motion_step(c, 0xFF),
            motion_step(c, 0x01),
            "高位全忽略"
        );
    }
    // bit0=1 ⇒ $534 翻转（EOR 语义）
    let a = motion_step(0x00, 0x00);
    let b = motion_step(0x00, 0x01);
    assert_eq!(b.v534, a.v534 ^ 0x01, "EOR 逐位：bit0 翻转");
    // 字段复用（enemy-motion §3.2）：运动机重写 $518/$534——与四参链同字段两段生命周期
    //（投放控制期=四参；运动期=速度分量）。断言=运动步出参占用这两字段名（接口面锁死）。
    let o = motion_step(0x03, 0x00);
    assert_ne!(o.v518, o.v534, "两字段独立进给");
    // 型值迁移登记（Wt-5 时间线实证：$2B→$12→$4E 同槽三段）
    assert_eq!(
        TYPE_MIGRATION_SLOT09,
        [(1479, 0x2B), (1592, 0x12), (1627, 0x4E)],
        "迁移序 $2B→$12→$4E（A 级时间线，F-D90 更正帧 1592 逐帧直读）"
    );
}

/// ⑭ 派发双模式判据（bank02:82F8 bit7 分派，t59 双模式面）。
#[test]
fn dispatch_mode_bit7_gate() {
    use game_core::enemy::{dispatch_mode, DispatchMode, TYPE_SIGNATURES};
    // bit7 边界：0x7F 表分发 / 0x80 自向量
    assert_eq!(dispatch_mode(0x7F), DispatchMode::TableDispatch);
    assert_eq!(dispatch_mode(0x80), DispatchMode::SelfVector);
    // 已知指针归类：$89DF/$89E4/$9010/$A631/$B011/$AE6B/$B3B9/$B04C（hi≥$80=自向量）
    for ptr in [0x89DFu16, 0x89E4, 0x9010, 0xA631, 0xB011, 0xAE6B, 0xB3B9, 0xB04C] {
        assert_eq!(
            dispatch_mode((ptr >> 8) as u8),
            DispatchMode::SelfVector,
            "{ptr:#X} hi bit7=1 ⇒ 自向量"
        );
    }
    // $00DC/$00F8 等 hi bit7=0 ⇒ 表分发（跳转目标语义待裁定——只判模式）
    for ptr in [0x00DCu16, 0x00F8, 0x00AC, 0x004C] {
        assert_eq!(
            dispatch_mode((ptr >> 8) as u8),
            DispatchMode::TableDispatch,
            "{ptr:#X} hi bit7=0 ⇒ 表分发（值语义挂账中）"
        );
    }
    // 登记表一致性（c4e6b68 双类）：BehPtr 项归自向量、DispatchSeed 项归表分发
    for sig in TYPE_SIGNATURES {
        match sig.beh {
            BehaviorField::BehPtr(p) => {
                assert_eq!(dispatch_mode((p >> 8) as u8), DispatchMode::SelfVector)
            }
            BehaviorField::DispatchSeed(_) => {
                assert_eq!(dispatch_mode(0x00), DispatchMode::TableDispatch)
            }
        }
    }
}

// ===== 批次4f：t59 相位 handler =====

/// ⑮ $836E 恒速下沉 + $8314 duration 倒数（终止双条件 → $D10C）。
#[test]
fn t59_phase_handlers() {
    use game_core::enemy::{
        handler_const_descent, handler_duration_tick, TERMINATE_RECYCLE_EXIT,
    };
    // 恒速下沉：y+1，溢出终止
    assert_eq!(handler_const_descent(0x00), Some(0x01));
    assert_eq!(handler_const_descent(0xFE), Some(0xFF));
    assert_eq!(handler_const_descent(0xFF), None, "y 溢出 ⇒ $D10C 终止（BCS）");
    // duration 倒数：DEC 归零终止
    assert_eq!(handler_duration_tick(0x05), Some(0x04));
    assert_eq!(handler_duration_tick(0x02), Some(0x01));
    assert_eq!(handler_duration_tick(0x01), None, "倒数归零 ⇒ $D10C（BEQ）");
    assert_eq!(handler_duration_tick(0x00), Some(0xFF), "DEC 回绕后非零续行（D4 保真字面）");
    // 唯一出口登记
    assert_eq!(TERMINATE_RECYCLE_EXIT, 0xD10C);
}

/// ⑯ $83DE 参数表字节精确 + $8341 RNG jitter 算术。
#[test]
fn t59_param_tables_and_jitter() {
    use game_core::enemy::{rng_jitter_offsets, PARAM_TABLE_518, PARAM_TABLE_684};
    // 参数表 ROM 字节精确（bank02 $8406/$840B，5 项各）
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let b2 = |a: usize| 0x10 + 2 * 0x2000 + (a & 0x1FFF);
    assert_eq!(&PARAM_TABLE_518[..], &rom[b2(0x8406)..b2(0x8406) + 5], "$8406 表");
    assert_eq!(&PARAM_TABLE_684[..], &rom[b2(0x840B)..b2(0x840B) + 5], "$840B 表");
    // 族值断言（$0A/$0B/$0C 族、$5F-$67 族）
    assert_eq!(PARAM_TABLE_518, [0x0A, 0x0B, 0x0C, 0x0C, 0x0C]);
    assert_eq!(PARAM_TABLE_684, [0x5F, 0x61, 0x63, 0x65, 0x67]);
    // RNG jitter 字面链偏移（SBC borrow+ADC 进位合成）：carry=1 时 nibble≥7 ⇒ +1..+8、
    // nibble<7 ⇒ −7..−1（非对称——"±7"是近似注，字面链为准）
    let (dx, dy) = rng_jitter_offsets(0x07, true);
    assert_eq!((dx, dy), (1, -8), "x: 7−7+carry1=+1；y: 0−8+borrow0=−8");
    // rng=$F0 carry=1 ⇒ x 侧 0−7、y 侧 15−7=8
    let (dx, dy) = rng_jitter_offsets(0xF0, true);
    assert_eq!((dx, dy), (-7, 8), "双向 ±7/±8 边界");
    // carry 面敏感（x 侧 borrow 双态——carry 注入必红）
    let a = rng_jitter_offsets(0x08, true);
    let b = rng_jitter_offsets(0x08, false);
    assert_ne!(a, b, "carry_after_rng 须入 x 侧算术");
    // y 侧 carry=rng bit3（确定性）：bit3 翻转必影响
    let a = rng_jitter_offsets(0x00, true);
    let b = rng_jitter_offsets(0x08, true);
    assert_ne!(a.1, b.1, "y 侧进位=rng bit3");
}

/// ⑰ jitter 数值面 256 值穷举锁（behavior-vm-handlers §3.2 表复核）+ 续链写原语。
#[test]
fn t59_jitter_exhaustive_and_chain() {
    use game_core::enemy::{chain_rewrite, rng_jitter_offsets};
    let mut yvals = std::collections::BTreeSet::new();
    for rng in 0..=255u8 {
        for carry in [true, false] {
            let (dx, dy) = rng_jitter_offsets(rng, carry);
            // 跳零（SBC/ADC 链 0 偏移永不出现——每帧必动 ≥1px，D4 保真）
            assert!(dx != 0 && dy != 0, "rng={rng:#X} c={carry} 零偏移禁现");
            // 并集界 −8..−1 ∪ +1..+9
            assert!((-8..=-1).contains(&dx) || (1..=9).contains(&dx), "dx={dx} 越域");
            assert!((-8..=-1).contains(&dy) || (1..=9).contains(&dy), "dy={dy} 越域");
        }
        yvals.insert(rng_jitter_offsets(rng, true).1);
        // y 侧确定性：与 carry_after_rng 无关
        assert_eq!(
            rng_jitter_offsets(rng, true).1,
            rng_jitter_offsets(rng, false).1,
            "y 侧只看 rng bit3"
        );
    }
    // y 值域全集=−8..−1∪+1..+9（**含 +1**——§3.2 表"bit3=1⇒+2..+9"漏叉例的字节实证）
    let expect: std::collections::BTreeSet<i8> =
        (-8i8..=-1).chain(1i8..=9).collect();
    assert_eq!(yvals, expect, "y 值域穷举真值");
    assert_eq!(rng_jitter_offsets(0x78, true).1, 1, "rng=$78（h=7,bit3=1）⇒ dy=+1 叉例");
    // 分支公式锁（test-runner 独立复算背书）：dy(bit3=1,n)=n−7+[n≥7]、dy(bit3=0,n)=n−8+[n≥8]
    for n in 0..=15u8 {
        let e1 = n as i8 - 7 + i8::from(n >= 7);
        let e0 = n as i8 - 8 + i8::from(n >= 8);
        assert_eq!(rng_jitter_offsets((n << 4) | 0x08, true).1, e1, "bit3=1 n={n}");
        assert_eq!(rng_jitter_offsets(n << 4, true).1, e0, "bit3=0 n={n}");
    }
    // 续链写原语（8337 型：$438:=lo 先 / $41C:=hi 后——写序照 8B8E-8B95 同型）
    assert_eq!(chain_rewrite(0x836E), (0x6E, 0x83), "(lo, hi) 写序对");
    assert_eq!(chain_rewrite(0x838E), (0x8E, 0x83));
}

/// ⑱ 调用环双序轮转 + MMC3 按型切 bank（bank02:827D-82CD）。
#[test]
fn t59_call_loop_and_bank_switch() {
    use game_core::enemy::{
        call_loop_round1, call_loop_slots, type_bank_switch, CALL_LOOP_SLOT_TABLE,
    };
    // 首轮 0..7
    assert_eq!(call_loop_round1(), [0, 1, 2, 3, 4, 5, 6, 7]);
    // 次轮双序：flag=false 升序 8..27 / flag=true 降序 27..8（公平性交替面）
    let asc = call_loop_slots(false);
    let desc = call_loop_slots(true);
    assert_eq!(asc, core::array::from_fn(|i| i as u8 + 8));
    assert_eq!(desc, core::array::from_fn(|i| 27 - i as u8));
    // 双序互逆（遍历同集）——同负载次序分歧=D4 保真
    let mut a = asc.to_vec();
    let mut d = desc.to_vec();
    a.sort();
    d.sort();
    assert_eq!(a, d);
    assert_ne!(asc.to_vec(), desc.to_vec(), "次序分歧必现（公平轮转）");
    // 表 40B 字节精确 vs ROM（bank2 $82D0）
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let o = 0x10 + 2 * 0x2000 + (0x82D0 & 0x1FFF);
    assert_eq!(&CALL_LOOP_SLOT_TABLE[..], &rom[o..o + 40], "$82D0 表字节精确");
    // MMC3 切 bank 判据：type<$30 不切 / ≥$30 查表 / 同值跳过（CMP $01/BEQ）
    let tbl = [0u8; 0x30].iter().copied().chain([7, 3, 7]).collect::<Vec<u8>>();
    assert_eq!(type_bank_switch(0x2F, 0, &tbl), None, "<$30 不切");
    assert_eq!(type_bank_switch(0x30, 0, &tbl), Some(7), "$816E+$30 表值");
    assert_eq!(type_bank_switch(0x31, 0, &tbl), Some(3));
    assert_eq!(type_bank_switch(0x30, 7, &tbl), None, "同值跳过（防重写）");
}

/// ⑲ 进场族/参数化段族模板登记 + 段表字节精确。
#[test]
fn t59_entry_and_segment_templates() {
    use game_core::enemy::{
        chain_rewrite, EntryTemplate, ENTRY_TEMPLATES, PARAM_SEGMENTS, SEG_A_T518, SEG_A_T534,
        SEG_A_T684, SEG_C_T518, SEG_C_T684,
    };
    // 进场族 6 型：续链写对与 chain_rewrite 一致（lo/hi 拆分互证）
    for t in &ENTRY_TEMPLATES {
        let (lo, hi) = chain_rewrite(t.chain_target);
        assert_eq!(
            u16::from(hi) << 8 | u16::from(lo),
            t.chain_target,
            "续链写对互证（{:04X}）",
            t.entry
        );
    }
    // 已知对 golden
    assert_eq!(ENTRY_TEMPLATES[0], EntryTemplate { entry: 0x83BA, chain_target: 0x83DC, ed3b_param: 0x23, preset: None, tail: 0x83DC });
    assert_eq!(ENTRY_TEMPLATES[2].preset, Some((0x00, 0x05)), "$845A 预置 $072C:=0/$0710:=05");
    assert_eq!(ENTRY_TEMPLATES[5].preset, Some((0x03, 0x08)), "$85EA 预置");
    assert_eq!(ENTRY_TEMPLATES[5].tail, 0x85D3, "$85EA 尾跳 85D3 族");
    // 段表 ROM 字节精确（5 表）
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let f = |a: usize| 0x10 + 2 * 0x2000 + (a & 0x1FFF);
    for (addr, tbl, name) in [
        (0x83AEusize, &SEG_A_T518[..], "A518"),
        (0x83B2, &SEG_A_T684[..], "A684"),
        (0x83B6, &SEG_A_T534[..], "A534"),
        (0x844C, &SEG_C_T518[..], "C518"),
        (0x8453, &SEG_C_T684[..], "C684"),
    ] {
        assert_eq!(tbl, &rom[f(addr)..f(addr) + tbl.len()], "{name} 表字节精确");
    }
    // 段族结构：A/B/C 尾 $CC6A（公共运动尾）、D/E/F 尾 $84D1（波形驱动）
    for s in &PARAM_SEGMENTS[..3] {
        assert_eq!(s.tail, 0xCC6A);
    }
    for s in &PARAM_SEGMENTS[3..] {
        assert_eq!(s.tail, 0x84D1);
        assert!(s.t518.is_empty() && s.t684.is_empty() && s.t534.is_empty(), "D/E/F 走波形驱动无表写");
    }
    // A 段独有第三表（$534）；B 段空表=相位翻转支
    assert_eq!(PARAM_SEGMENTS[0].t534, &SEG_A_T534[..]);
    assert!(PARAM_SEGMENTS[1].t534.is_empty());
    // Y 索引基数登记面（A=4/B=5/C=7/D=$0E/**E=$0F**/F=$0F——decomp 补谳字节锚）
    assert_eq!(
        PARAM_SEGMENTS.map(|s| s.cd9d_y),
        [Some(4), Some(5), Some(7), Some(0x0E), Some(0x0F), Some(0x0F)],
        "E 段 $84C3 LDA #$50/LDY #$0F（ROM f=$44D3 直读）"
    );
    assert_eq!(PARAM_SEGMENTS[4].entry, 0x84C3, "E 入口=$84C3（$84C7=JSR 子址）");
    assert_eq!(PARAM_SEGMENTS[4].cd9d_a, Some(0x50));
    // B 段双入口（$83DC 装 A=$30 / $83DE 旁路=调用者 A=$40，$836E 尾带落点）
    assert_eq!(PARAM_SEGMENTS[1].bypass_entry, Some(0x83DE));
    assert_eq!(PARAM_SEGMENTS[1].cd9d_a, Some(0x30));
}

/// ⑳ $84D1 相位机六表字节精确 + 阈判/翻转原语。
#[test]
fn t59_phase_machine_tables() {
    use game_core::enemy::{
        phase_flip_534, phase_threshold_hit, PHASE_FLAG, PHASE_PTR_HI, PHASE_PTR_LO,
        PHASE_T518, PHASE_T684, PHASE_THRESHOLD,
    };
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let f = |a: usize| 0x10 + 2 * 0x2000 + (a - 0x8000);
    for (addr, tbl, name) in [
        (0x854Dusize, &PHASE_T518[..], "518对"),
        (0x8565, &PHASE_T684[..], "684对"),
        (0x857D, &PHASE_THRESHOLD[..], "相位阈"),
        (0x858A, &PHASE_PTR_LO[..], "指针lo"),
        (0x8597, &PHASE_PTR_HI[..], "指针hi"),
        (0x85A4, &PHASE_FLAG[..], "标志"),
    ] {
        assert_eq!(tbl, &rom[f(addr)..f(addr) + tbl.len()], "{name} 字节精确");
    }
    // 尺寸结构：24/24/13/13/13/13（$85B1=#19 进场⑤ 边界收口）
    assert_eq!(f(0x854D) + 24, f(0x8565));
    assert_eq!(f(0x8565) + 24, f(0x857D));
    assert_eq!(f(0x857D) + 13, f(0x858A));
    assert_eq!(f(0x85A4) + 13, f(0x85B1), "尾表贴 #19 入口");
    // 阈判：<8 窗（wrapping 差）
    assert!(phase_threshold_hit(0x05, 0x00), "5−0=5<8 命中");
    assert!(phase_threshold_hit(0x07, 0x00));
    assert!(!phase_threshold_hit(0x08, 0x00), "8−0=8 不命中（边界）");
    assert!(phase_threshold_hit(0x02, 0xFF), "回绕 3<8 命中");
    assert!(!phase_threshold_hit(0x00, 0x10), "未到阈（回绕大差）不命中");
    // 相位翻转：($2B&1)^快照（bit0 判据同 Wt-6）
    assert_eq!(phase_flip_534(0x12, 0x00), 0x12);
    assert_eq!(phase_flip_534(0x12, 0x01), 0x13);
    assert_eq!(phase_flip_534(0x12, 0xFF), 0x13, "仅 bit0 入 XOR");
    // 标志表 bit7 开关存在性（翻转相位分布非全零）
    assert!(PHASE_FLAG.iter().any(|v| v & 0x80 != 0));
    assert!(PHASE_FLAG.iter().any(|v| v & 0x80 == 0));
}

/// ㉑ 敌侧框形取框（bank14:$D511-$D5AE）：指针表/子框流/翻转/钳位。
#[test]
fn shape_box_fetch() {
    use game_core::enemy::{fetch_shape_boxes, shape_stream_addr, SHAPE_PTR_HI, SHAPE_PTR_LO};
    // 指针表 88B×2 字节精确 vs ROM（bank2 R6 窗）
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let r6 = |a: usize| 0x10 + 2 * 0x2000 + (a - 0x8000);
    assert_eq!(&SHAPE_PTR_LO[..], &rom[r6(0x97BA)..r6(0x97BA) + 88], "lo 表");
    assert_eq!(&SHAPE_PTR_HI[..], &rom[r6(0x9812)..r6(0x9812) + 88], "hi 表");
    // 形 $27（中 boss）：流 $9936 = [01][F9 07 F9 07] = ±7 对称 14×14
    assert_eq!(shape_stream_addr(0x27), Some(0x9936));
    let stream = &rom[r6(0x9936)..r6(0x9936) + 5];
    let boxes = fetch_shape_boxes(stream, 128, 100, false, false);
    assert_eq!(boxes, vec![(121, 135, 93, 107)], "x±7/y±7 对称框");
    // 翻转判别（非对称框）：x=−2..+1 ⇒ flip_x 左'=-右=-1、右'=-左=+2
    let asym = [1u8, 0xFE, 0x01, 0x00, 0x04]; // −2..+1, 0..+4
    assert_eq!(
        fetch_shape_boxes(&asym, 50, 50, false, false),
        vec![(48, 51, 50, 54)],
        "无翻转原样"
    );
    assert_eq!(
        fetch_shape_boxes(&asym, 50, 50, true, false),
        vec![(49, 52, 50, 54)],
        "flip_x: 左'=-右=-1、右'=-左=+2（对称框不可判此必红）"
    );
    assert_eq!(
        fetch_shape_boxes(&asym, 50, 50, false, true),
        vec![(48, 51, 46, 50)],
        "flip_y: 上'=-下=-4、下'=-上=0"
    );
    // 钳位：负偏移钳 0、正溢出钳 FF
    let edge = [1u8, 0xFC, 0x04, 0xFC, 0x04]; // −4..+4
    assert_eq!(fetch_shape_boxes(&edge, 2, 2, false, false), vec![(0, 6, 0, 6)]);
    assert_eq!(fetch_shape_boxes(&edge, 253, 253, false, false), vec![(249, 255, 249, 255)]);
    // 子框数多框 + 流不足 fail-closed（已解析部分）
    let two = [2u8, 0xFF, 0x01, 0xFF, 0x01, 0xF0, 0x10, 0xF0, 0x10];
    assert_eq!(fetch_shape_boxes(&two, 100, 100, false, false).len(), 2);
    assert_eq!(fetch_shape_boxes(&two[..6], 100, 100, false, false).len(), 1, "流不足截断");
}

/// ㉒ 表分发全表嵌码（decomp f06a376 接线律）。
#[test]
fn type_dispatch_table_wiring() {
    use game_core::enemy::{type_bank_switch, TypeDispatch, TYPE_DISPATCH};
    // 验证 4/4 对（f06a376）：$2B→$90AB、$38→bank3:$B4A3、$51→bank4:$B00A、$56→bank4:$B289
    assert_eq!(TYPE_DISPATCH[0x2B].target, 0x90AB, "$2B→$90AB→$9120（trace 吻合）");
    assert_eq!(TYPE_DISPATCH[0x38].target, 0xB4A3);
    assert_eq!(TYPE_DISPATCH[0x51].target, 0xB00A);
    assert_eq!(TYPE_DISPATCH[0x56].target, 0xB289);
    // type<$30 不切 bank；type≥$30 带 bank_switch
    assert_eq!(TYPE_DISPATCH[0x2B].bank_switch, None, "<$30 不切");
    assert_eq!(TYPE_DISPATCH[0x38].bank_switch, Some(3), "$38→bank3");
    assert_eq!(TYPE_DISPATCH[0x51].bank_switch, Some(4));
    assert_eq!(TYPE_DISPATCH[0x56].bank_switch, Some(4));
    // type_bank_switch 消费律：$38 当前 bank≠3 ⇒ Some(3)；==3 ⇒ None（防重写）
    let tbl: Vec<u8> = (0..=0x38u8).map(|i| if i == 0x38 { 3 } else { 0 }).collect();
    assert_eq!(type_bank_switch(0x38, 7, &tbl), Some(3));
    assert_eq!(type_bank_switch(0x38, 3, &tbl), None, "同值防重写");
    // 全表完整性：非登记型值目标=0（fail-closed 空转面）
    let _ = TypeDispatch { target: 0, bank_switch: None };
}

/// ㉓ S4b-2 ③ 行为链断言面（六表↔轨迹配对数值）。
#[test]
fn phase_chain_assertion_face() {
    use game_core::enemy::{
        phase_t832e_value, DRIVER_ASSIGNMENTS, PARAM_TABLE_518, PARAM_TABLE_684, PHASE_T832E,
        PHASE_T914B, PHASE_T914F,
    };
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let r6 = |a: usize| 0x10 + 2 * 0x2000 + (a - 0x8000);
    // 三表字节精确
    assert_eq!(&PHASE_T914B[..], &rom[r6(0x914B)..r6(0x914B) + 4], "$914B");
    assert_eq!(&PHASE_T914F[..], &rom[r6(0x914F)..r6(0x914F) + 8], "$914F");
    assert_eq!(&PHASE_T832E[..], &rom[r6(0x832E)..r6(0x832E) + 4], "$832E");
    // $2B 首段取值族（trace ($518,$684,$534)∈{$25,$29}×{$F7,$FB,$FF}×{$02,$42}）
    assert!(PHASE_T914B.iter().all(|&v| v == 0x25 || v == 0x29));
    assert!(PHASE_T914F[..4].iter().all(|&v| v == 0xF7 || v == 0xFB || v == 0xFF));
    // $914F 循环 5 相位（F7→FB→FF→FB→F7）
    assert_eq!(&PHASE_T914F[..4], &[0xF7, 0xFB, 0xFF, 0xFB], "循环前四+回首 F7");
    // $832E |2 语义（$00|2=$02、$40|2=$42）
    assert_eq!(phase_t832e_value(0x00), 0x02);
    assert_eq!(phase_t832e_value(0x40), 0x42);
    // $12 迁移段 = $8406/$840B 表逐值（trace $684=$5F→$61→$63→$65→$67 序列）
    assert_eq!(&PARAM_TABLE_684[..5], &[0x5F, 0x61, 0x63, 0x65, 0x67], "迁移序列=段 B 表");
    assert!(PARAM_TABLE_518[..3].iter().all(|&v| (0x0A..=0x0C).contains(&v)), "$518=$0A-$0C 族");
    // 型值↔驱动 8 行指派（内态族分野）
    let inner: Vec<u8> = DRIVER_ASSIGNMENTS.iter().filter(|d| d.inner_state).map(|d| d.type_code).collect();
    assert_eq!(inner, vec![0x14, 0x15, 0x17], "$84D1 相位机=内态族");
    assert!(DRIVER_ASSIGNMENTS.iter().any(|d| d.type_code == 0x2B && d.driver.contains("9120")));
}

/// ㉔ 运动更新真身（099c510 ①：亚像素对积分+出屏门）。
#[test]
fn motion_update_subpixel_integration() {
    use game_core::engine::{motion_update, phase_counter_bump, MotionOutcome, ObjectSlot};
    // y 线性（vy=$00:$90 恒定 → y+=1、亚像素+=0x90/帧）
    let mut s = ObjectSlot { y: 0x40, f588: 0x90, f5a4: 0x00, ..Default::default() };
    let r = motion_update(&mut s);
    assert_eq!(s.f48c, 0x90, "亚像素累加");
    assert_eq!(s.y, 0x40, "未进位 y 不动");
    assert!(matches!(r, MotionOutcome::Continue { .. }));
    for _ in 0..2 {
        motion_update(&mut s);
    }
    assert_eq!(s.y, 0x41, "0x90×3=0x1B0 ⇒ y+1 亚像素 0xB0（mod256 回绕）");
    assert_eq!(s.f48c, 0xB0);
    // x 对称（vx_lo/vx_hi）
    let mut s = ObjectSlot { x: 0x20, f470: 0xFF, f550: 0x01, f56c: 0x00, ..Default::default() };
    let r = motion_update(&mut s);
    assert_eq!(s.f470, 0x00, "亚像素回绕");
    assert_eq!(s.x, 0x21, "亚像素进位入坐标");
    assert_eq!(r, MotionOutcome::Continue { subpixel_carry_x: true, subpixel_carry_y: false });
    // 出屏：y≥$F0
    let mut s = ObjectSlot { y: 0xEF, f588: 0x00, f5a4: 0x01, ..Default::default() };
    assert_eq!(motion_update(&mut s), MotionOutcome::Offscreen, "y≥$F0 出屏");
    // 出屏：坐标溢出
    let mut s = ObjectSlot { y: 0xFF, f588: 0x00, f5a4: 0x01, ..Default::default() };
    assert_eq!(motion_update(&mut s), MotionOutcome::Offscreen, "坐标溢出出屏");
    // $5C0→$5DC 连接件
    let mut s = ObjectSlot { f5dc: 5, ..Default::default() };
    phase_counter_bump(&mut s);
    assert_eq!(s.f5dc, 6);
}

/// ㉜ 外部数据接线（Soig B 级登记面）：F/B/T/W 等级时长+boss 杂兵清零互证。
#[test]
fn soig_external_data_registry() {
    use game_core::enemy::{BOSS_SPAWN_CLEARS_MOBS, SPECIAL_READY_TIMER, SPECIAL_WEAPON_TIMERS};
    // ready timer 27（四武器同值）
    assert_eq!(SPECIAL_READY_TIMER, 27);
    // 逐格字面（xlsx sheet1 解析复导）：F/B/T/W × L1-4（含 -2/-3/-5 修正）
    assert_eq!(SPECIAL_WEAPON_TIMERS[0], [180, 300, 358, 358], "F");
    assert_eq!(SPECIAL_WEAPON_TIMERS[1], [240, 298, 418, 418], "B");
    assert_eq!(SPECIAL_WEAPON_TIMERS[2], [300, 358, 416, 416], "T（-2*1/-2*2）");
    assert_eq!(SPECIAL_WEAPON_TIMERS[3], [238, 297, 355, 355], "W（-2/-3/-5）");
    // L4=same as L3（四武器同构）
    for w in &SPECIAL_WEAPON_TIMERS {
        assert_eq!(w[2], w[3], "L4=L3");
    }
    // boss 杂兵清零互证登记
    assert!(BOSS_SPAWN_CLEARS_MOBS);
}

/// ㉝ W-DMG3 伤害面武器参数化（ROM 两立即数）。
#[test]
fn damage_apply_weapon_parameterized() {
    use game_core::shot::{damage_apply, ShotDmgPath};
    // $D688 路：−2HP（旧行为兼容）
    assert_eq!(damage_apply(10, true, 0, ShotDmgPath::D688Minus2), 8);
    // $CEA2 路：−3HP（新——表 −3 族码径候选）
    assert_eq!(damage_apply(10, true, 0, ShotDmgPath::CEA2Minus3), 7);
    // 物撞路：对方 HP（武器维不影响）
    assert_eq!(damage_apply(10, false, 4, ShotDmgPath::D688Minus2), 6);
    assert_eq!(damage_apply(10, false, 4, ShotDmgPath::CEA2Minus3), 6);
    // 饱和 0
    assert_eq!(damage_apply(1, true, 0, ShotDmgPath::CEA2Minus3), 0);
    assert_eq!(damage_apply(2, true, 0, ShotDmgPath::CEA2Minus3), 0);
}
