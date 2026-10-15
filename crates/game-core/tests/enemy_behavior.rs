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
    assert_eq!((s[0].beh_ptr, s[0].spawn), (0x89DF, Some((128, 8))));
    // $2A：$9010、(29,10)（f1789/2109/2365 三点）
    let s = type_signatures(0x2A);
    assert_eq!((s[0].beh_ptr, s[0].spawn), (0x9010, Some((29, 10))));
    // $2B/$4E 组员族：行为指针恒 $00DC；出生坐标不恒定 ⇒ None（不发明）
    for t in [0x2Bu8, 0x4E] {
        let s = type_signatures(t);
        assert_eq!(s[0].beh_ptr, 0x00DC);
        assert_eq!(s[0].spawn, None, "成员坐标摆动不登记恒定值");
    }
    // $07 三行为指针（底部点 $A631 / 尾段流 $AB30 / $AAD9）——同型多指针按观测对登记
    let s = type_signatures(0x07);
    let ptrs: Vec<u16> = s.iter().map(|x| x.beh_ptr).collect();
    assert_eq!(ptrs, vec![0xA631, 0xAB30, 0xAAD9], "同型多指针均登记（勿 1:1 强绑）");
    // $02 编队：$AB74（5 槽同帧 216f 轮）
    assert_eq!(type_signatures(0x02)[0].beh_ptr, 0xAB74);
    // 扩表观测对（trace spawns 全量）：$2A→$9079、$2B→$0079、$05→$B011
    assert!(type_signatures(0x2A).iter().any(|s| s.beh_ptr == 0x9079));
    assert!(type_signatures(0x2B).iter().any(|s| s.beh_ptr == 0x0079));
    assert_eq!(type_signatures(0x05)[0].beh_ptr, 0xB011);
    // 恒定坐标仅 2 例（$2F/$2A 主指针）；其余 None
    for s in type_signatures(0x07) {
        assert_eq!(s.spawn, None, "$07 坐标不恒定不登记");
    }
    // beh=$0000 暂态**不登记**（非行为指针——登记即发明必红）
    for sig in TYPE_SIGNATURES {
        assert_ne!(sig.beh_ptr, 0x0000, "$0000 装载暂态禁入签名表");
    }
    // $2E 组长族：行为指针未观测 ⇒ 空（不发明必红）
    assert!(type_signatures(0x2E).is_empty(), "$2E beh 未观测不发明");
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
        [(1479, 0x2B), (1595, 0x12), (1627, 0x4E)],
        "迁移序 $2B→$12→$4E（A 级时间线）"
    );
}
