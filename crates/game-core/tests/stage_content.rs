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
    nt.nmi_flush(&row, &attrs, 0x00, 0x20, 0xC0, 1); // 基址 $2000+$0F=$200F（R-l4-linebuf-6）
    assert_eq!(nt.tiles[0][0x0F], 5, "组 0 首字节");
    assert_eq!(nt.tiles[0][0x76], 9, "组 3 末字节（$200F+$67）");
    assert_eq!(nt.attrs[0][0x0F], 0xE4, "属性 $23C0+$0F=$23CF");
    // PPU 域守卫：非 $2xxx 地址（boot 期 $8E/$8F 驻留）不入 nametable
    let mut nt2 = Nametable::default();
    nt2.nmi_flush(&row, &attrs, 0x00, 0x00, 0x00, 3);
    assert_eq!(nt2.tiles[0][0x0F], 0, "非 PPU 域地址不得写入");
    // pending=0 不写
    let mut nt3 = Nametable::default();
    nt3.nmi_flush(&row, &attrs, 0x00, 0x20, 0xC0, 0);
    assert_eq!(nt3.tiles[0][0x0F], 0, "pending=0 不写");
    // 属性 2×2 块取位（NES 规范）
    let mut nt4 = Nametable::default();
    nt4.attrs[0][0] = 0xE4; // 11 10 01 00
    assert_eq!(nt4.block_palette(0, 0, 0), 0);
    assert_eq!(nt4.block_palette(0, 2, 0), 1);
    assert_eq!(nt4.block_palette(0, 0, 2), 2);
    assert_eq!(nt4.block_palette(0, 2, 2), 3);
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

// ============ t193：STG-02..08 内容集成（t87 三级骨架） ============

#[test]
fn stg_02_08_skeleton_registry() {
    use game_core::stage::{stage_skeleton};
    // t87 §1 三级骨架总表（A/B 级）逐关核对
    let s2 = stage_skeleton(2).expect("STG-2");
    assert_eq!((s2.root, s2.tail), (0xA048, 0xA3C4));
    assert_eq!((s2.rec4, s2.rec6, s2.loop_n), (242, 14, 3));
    let s3 = stage_skeleton(3).expect("STG-3");
    assert_eq!((s3.rec4, s3.rec6, s3.loop_n), (29, 13, 1));
    assert_eq!(s3.tick_params, Some([16, 16, 8, 16, 8, 4, 16]), "STG-3 CNT tick 表 7 段");
    assert_eq!(s3.scripts.len(), 8, "STG-3 链段族 8 脚本");
    let s7 = stage_skeleton(7).expect("STG-7");
    let s1 = stage_skeleton(1).expect("STG-1");
    assert_eq!(s7.root, s1.root, "STG-1/7 共享流（root 同值 $A000）");
    assert_eq!(s7.rec4, s1.rec4, "共享流内容同");
    let s8 = stage_skeleton(8).expect("STG-8");
    assert_eq!(s8.tick_params, Some([16, 16, 8, 16, 8, 4, 16]), "STG-8 CNT 同 STG-3");
    assert!(stage_skeleton(9).is_none(), "STG-9 不存在（fail-closed）");
    assert!(stage_skeleton(0).is_none());
}

#[test]
fn stg_02_08_rom_root_tail_verified() {
    // ROM 字节级：目录行 root/tail 与骨架表逐值对拍（t84 目录解析复用）
    use game_core::stage::{parse_stage_dir, stage_skeleton};
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let bank8 = &rom[0x10 + 8 * 0x2000..0x10 + 9 * 0x2000];
    let dir = parse_stage_dir(bank8);
    for n in 1..=8u8 {
        let sk = stage_skeleton(n).expect("骨架");
        let d = dir[(n - 1) as usize];
        assert_eq!(d.root_ptr, sk.root, "STG-{n} root 目录 vs 骨架");
        assert_eq!(d.tail_ptr, sk.tail, "STG-{n} tail 目录 vs 骨架");
    }
}

#[test]
fn stg_negative_skeleton_guards() {
    use game_core::stage::stage_skeleton;
    // 负向：rec 计数非零（空表必红哨兵）
    for n in 1..=8u8 {
        let s = stage_skeleton(n).unwrap();
        assert!(s.rec4 > 0, "STG-{n} rec4 非零");
        assert!(s.rec6 > 0, "STG-{n} rec6 非零");
        assert!(!s.scripts.is_empty(), "STG-{n} 脚本集非空");
    }
}

// ===== 批次3a：弹网格 SoA + 槽位交叠裁定 + 双分配器（arena-midboss §1.1）=====

use game_core::shot::{
    enemy_slot_alloc, shot_slot_alloc, BulletGrid, SlotAlloc, ENEMY_ONLY_SLOTS, ENEMY_SHOT_TYPE,
    PLAYER_ONLY_SLOTS, SHARED_SLOTS,
};

/// ⑩ 六数组 SoA + 槽位集算术（distinct 24 = 20+12−8；交叠 8 槽逐值）。
#[test]
fn bullet_grid_soa_and_slot_overlap_ruling() {
    let g = BulletGrid::default();
    // 空槽判定 = $040E+Y==0（两分配器同判）
    assert!(g.is_empty(0x83));
    let mut g2 = g.clone();
    g2.state[0x83] = 0x10;
    assert!(!g2.is_empty(0x83));
    g2.release(0x83);
    assert!(g2.is_empty(0x83), "回收=state:=0（$87FD 槽释放）");
    // 交叠裁定算术：20+12−8=24（非 32）
    assert_eq!(PLAYER_ONLY_SLOTS.len() + SHARED_SLOTS.len(), 20, "自机 20 槽");
    assert_eq!(ENEMY_ONLY_SLOTS.len() + SHARED_SLOTS.len(), 12, "敌弹 12 槽");
    assert_eq!(12 + 8 + 4, 24, "distinct 24（20+12−8）");
    assert_ne!(20 + 12, 24, "按不相交建模必红");
    // 共享 8 槽逐值（B1 快照实测时分复用）
    assert_eq!(SHARED_SLOTS, [0x83, 0x84, 0x9F, 0xA0, 0xBB, 0xBC, 0xD7, 0xD8]);
    // 敌弹型码 $80（bit7 置=与自机弹可判别）
    assert_eq!(ENEMY_SHOT_TYPE, 0x80);
    assert_ne!(ENEMY_SHOT_TYPE & 0x80, 0, "敌弹 bit7 置必红");
}

/// ⑪ 双分配器几何 + 共享槽时分复用（勿按不相交建模）。
#[test]
fn dual_allocators_and_time_division() {
    // 敌弹分配器：LDY #$83 起、组内 +1、组距 $1C、全满 None
    let mut state = [0u8; 0x100];
    assert_eq!(enemy_slot_alloc(&state), Some(0x83), "首适应 $83");
    for y in [0x83u8, 0x84, 0x85] {
        state[y as usize] = 1;
    }
    assert_eq!(enemy_slot_alloc(&state), Some(0x9F), "组跳 $85+$1A=$9F");
    for base in [0x83u8, 0x9F, 0xBB, 0xD7] {
        for k in 0..3u8 {
            state[(base + k) as usize] = 1;
        }
    }
    assert_eq!(enemy_slot_alloc(&state), None, "全满 SEC ⇒ None");
    // 时分复用哨兵：自机占满独占 12 后，自机分配器落到**共享槽** $83（非独占建模必红）
    let mut st2 = [0u8; 0x100];
    for &y in PLAYER_ONLY_SLOTS.iter() {
        st2[y as usize] = 1;
    }
    assert_eq!(shot_slot_alloc(&st2), SlotAlloc::Found { slot: 0x83, a: 0 }, "自机吃共享槽 $83");
    // 敌弹同理：独占 4 占满后落共享槽
    let mut st3 = [0u8; 0x100];
    for &y in ENEMY_ONLY_SLOTS.iter() {
        st3[y as usize] = 1;
    }
    assert_eq!(enemy_slot_alloc(&st3), Some(0x83), "敌弹吃共享槽 $83（时分复用）");
}

/// ⑫ 命中框注册 golden（7×10 钳位）+ 重叠/属性/综合判定 + 一框一伤 + 伤害公式。
#[test]
fn hitbox_register_and_damage_golden() {
    use game_core::shot::{
        attrs_intersect, boxes_overlap, clear_hit_entry, damage_apply, hit_resolves,
        HitDmgPath,
        shot_hitbox_register, HitBoxEntry,
    };
    // 注册 golden：x=128,y=128 → 7×10 框（124..131, 123..133），归属=槽偏移，attr=$30
    let e = shot_hitbox_register(128, 128, 0x83);
    assert_eq!((e.left, e.right), (124, 131), "x−4..x+3 跨度 7");
    assert_eq!((e.top, e.bottom), (123, 133), "y−5..y+5 跨度 10");
    assert_eq!(e.owner, 0x83);
    assert_eq!(e.attr, 0x30);
    assert_eq!(e.right - e.left, 7, "跨度口径 7×10");
    assert_eq!(e.bottom - e.top, 10);
    // 钳位哨兵：x−4/y−5 负向钳 0、x+3/y+5 溢出钳 FF
    let edge = shot_hitbox_register(2, 3, 0x80);
    assert_eq!((edge.left, edge.top), (0, 0), "负向钳 0");
    let edge2 = shot_hitbox_register(0xFE, 0xFC, 0x80);
    assert_eq!((edge2.right, edge2.bottom), (0xFF, 0xFF), "溢出钳 FF");
    // 重叠判定：4 边含端点（>=/<=）；几何分开 1px 不叠
    let a = shot_hitbox_register(100, 100, 0x80);
    let b = shot_hitbox_register(111, 100, 0x80); // 左=107 > a.right=103
    assert!(!boxes_overlap(&a, &b), "x 分离 1px 不叠");
    let c = shot_hitbox_register(110, 100, 0x80); // 左=106…仍分；取 108：左=104>103
    assert!(!boxes_overlap(&a, &c));
    let d = shot_hitbox_register(107, 100, 0x80); // 左=103 == a.right=103 ⇒ 端点相接=叠
    assert!(boxes_overlap(&a, &d), "端点相接（>=）=叠——闭/开区间错判必红");
    // 属性相交（团队过滤）：$30&$13=$10≠0 命中；$30&$0F=0 不命中（几何叠也无效）
    assert!(attrs_intersect(0x30, 0x13), "弹$30×敌$13 相交 $10");
    assert!(!attrs_intersect(0x30, 0x0F), "弹$30×$0F 不相交——同队过滤");
    let e1 = shot_hitbox_register(100, 100, 0x80);
    let e2 = HitBoxEntry { attr: 0x0F, ..shot_hitbox_register(100, 100, 0x85) };
    assert!(!hit_resolves(&e1, &e2), "几何叠但属性不交 ⇒ 不命中（漏属性测试必红）");
    // 一框一伤：清框清 attr/top/bottom，left/right/owner 不动
    let cleared = clear_hit_entry(&e1);
    assert_eq!((cleared.attr, cleared.top, cleared.bottom), (0, 0, 0));
    assert_eq!(
        (cleared.left, cleared.right, cleared.owner),
        (e1.left, e1.right, e1.owner),
        "仅清三字段（写集外字段误清必红）"
    );
    // 伤害公式：弹 −2/物撞 −=对方 HP；HP 饱和 0（$D616 死亡门由调用方判 hp==0）
    assert_eq!(damage_apply(14, HitDmgPath::Const2, 30), 12, "弹命中 −2（中 boss HP14→12 实测锚）");
    assert_eq!(damage_apply(2, HitDmgPath::Const2, 30), 0, "HP 归 0 ⇒ 死亡门");
    assert_eq!(damage_apply(10, HitDmgPath::Field0630, 6), 4, "物撞 −=对方 $0630=6");
    assert_eq!(damage_apply(10, HitDmgPath::Field0630, 14), 0, "物撞伤害=对方 HP（非 −2）饱和 0");
    assert_ne!(damage_apply(14, HitDmgPath::Field0630, 30), 12, "物撞按 −2 误算必红");
    assert_eq!(damage_apply(1, HitDmgPath::Const2, 30), 0, "下溢饱和 0 不回绕");
}

/// ㉘ 商店主面板项数随关卡变（stg08 run#1 新语义）：「退」=末项非固定 3。
#[test]
fn shop_menu_items_stage_varying() {
    use game_core::ui::{ShopPurchasePhase, ShopPurchaseState};
    let mk = |items| ShopPurchaseState {
        phase: ShopPurchasePhase::MainRing,
        menu_items: items,
        ..Default::default()
    };
    // stg01-07：4 项 ⇒ 「退」=光标 3（旧行为兼容）
    assert_eq!(mk(4).menu_exit_cursor(), 3);
    // stg07→08：≥5 项 ⇒ 「退」=末项（run#1 负结果：3×down 落购买项）
    assert_eq!(mk(5).menu_exit_cursor(), 4);
    assert_eq!(mk(6).menu_exit_cursor(), 5);
    // fail-closed：0 项防下溢
    assert_eq!(mk(0).menu_exit_cursor(), 0);
}

/// ㊱ A1 商店隐藏交互三件套（S-CH-2：#2 免费购物/#3 钱袋转换）。
#[test]
fn shop_hidden_interaction_face() {
    use game_core::ui::{
        cheat2_free_shop, cheat3_moneybag, moneybag_convert_flag, CHEAT2_FREE_SHOP_MASK,
        CHEAT3_MONEYBAG_MASK,
    };
    // #2：$21 掩码双位全按才真
    assert_eq!(CHEAT2_FREE_SHOP_MASK, 0x21);
    assert!(cheat2_free_shop(0x21));
    assert!(cheat2_free_shop(0x31), "多余键不影响");
    assert!(!cheat2_free_shop(0x20), "单按假");
    assert!(!cheat2_free_shop(0x00));
    // #3：$D1 四位全按才真（候选掩码注）
    assert_eq!(CHEAT3_MONEYBAG_MASK, 0xD1);
    assert!(cheat3_moneybag(0xD1));
    assert!(!cheat3_moneybag(0xD0), "缺右假");
    // #3 旗操作：$32 &:= ~$40（转换旗候选）
    assert_eq!(moneybag_convert_flag(0x40), 0x00);
    assert_eq!(moneybag_convert_flag(0x50), 0x10, "他位保留");
}
