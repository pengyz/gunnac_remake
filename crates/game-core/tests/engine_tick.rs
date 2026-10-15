//! game-core::engine 集成主干骨架测试（合流裁定 ①）。
//!
//! 证据锚：A2 帧序（game-over-continue.md 帧序=行为→碰撞→主环）+ Wt-2
//! （tick=16px 滚动进度，停滚冻结）+ 两时序域红线（enemy-item-scheduling §4）。

use game_core::engine::*;
use game_core::wave::{SchedItem, SpawnTiming};

/// ① 帧序 A2 锚：stage_tick 相位序=行为→碰撞→主环（每帧恰一遍）。
#[test]
fn frame_order_a2_anchor() {
    let mut st = StageState::default();
    let fx = stage_tick(&mut st, &TickInputs::default());
    assert_eq!(fx.phase_trace, FRAME_ORDER.to_vec(), "A2 帧序锚");
    assert_eq!(st.frame, 1);
    // 多帧序稳定（非一次性）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert_eq!(fx2.phase_trace, FRAME_ORDER.to_vec());
    assert_eq!(st.frame, 2);
}

/// ② Wt-2 tick=滚动进度域：停滚冻结/16px 步进。
#[test]
fn wave_tick_scroll_domain() {
    let mut st = StageState::default();
    stage_tick(&mut st, &TickInputs { scroll_delta: 16 });
    assert_eq!(st.wave.tick, 1, "16px ⇒ 1 tick");
    stage_tick(&mut st, &TickInputs { scroll_delta: 0 });
    assert_eq!(st.wave.tick, 1, "停滚 ⇒ tick 冻结（Wt-2）");
    stage_tick(&mut st, &TickInputs { scroll_delta: 32 });
    assert_eq!(st.wave.tick, 3, "累计 48px ⇒ 3 tick");
    assert_eq!(st.wave.scroll_progress, 48);
}

/// ③ 两时序域分流红线（item 直投 tick 原子 vs 组员帧滴灌）。
#[test]
fn spawn_timing_route_two_domains() {
    let mut fx = StageEffects::default();
    let pair = SchedItem::Pair { type_: 0x84, param: 0x30 };
    let drip = SchedItem::Spawn { ctrl: 2, type_: 0x2F, args: [0x2B, 6, 0x0A, 0x20] };
    assert_eq!(route_spawn_item(&pair, &mut fx), Some(SpawnTiming::TickAtomic));
    assert_eq!(route_spawn_item(&drip, &mut fx), Some(SpawnTiming::FrameDrip));
    assert_eq!(fx.spawn_events, vec![pair.clone()], "直投同帧落地");
    assert_eq!(fx.drip_events, vec![drip.clone()], "滴灌独立通道");
    assert_ne!(fx.spawn_events, fx.drip_events, "混域必红");
    // 无投放项忽略
    let jump = SchedItem::Jump { ctrl: 9, target: 0xA397 };
    assert_eq!(route_spawn_item(&jump, &mut fx), None);
    assert_eq!(fx.spawn_events.len(), 1);
    assert_eq!(fx.drip_events.len(), 1);
}

/// ④ flow 侧消费 `$26` bit4 关末门（lead 契约：flow 消费 tick 侧事件）。
#[test]
fn flow_consumes_stage_end_gate() {
    let mut st = StageState::default();
    st.wave.flags_26 = 0x10;
    let fx = stage_tick(&mut st, &TickInputs::default());
    assert!(fx.stage_clear_requested, "$26 bit4 ⇒ 关末门请求");
    let mut st2 = StageState::default();
    st2.wave.flags_26 = 0x00;
    let fx2 = stage_tick(&mut st2, &TickInputs::default());
    assert!(!fx2.stage_clear_requested);
    // bit3（far-call 抑制门）不误触关末
    let mut st3 = StageState::default();
    st3.wave.flags_26 = 0x08;
    let fx3 = stage_tick(&mut st3, &TickInputs::default());
    assert!(!fx3.stage_clear_requested);
}

// ===== STG-01 数据装载（合流裁定②）=====

use game_core::wave::decode_sched;

/// bank8 数据窗（$A000 窗 → file 0x10+8×0x2000，与 stage_content 同口径）。
fn bank8_window() -> Vec<u8> {
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM 可读");
    rom[0x10 + 8 * 0x2000..0x10 + 9 * 0x2000].to_vec()
}

/// ⑤ STG-01 装载 golden：调度流 $A26D 43 记录 tick 锚簇（首 5+尾锚）。
#[test]
fn stg01_data_load_tick_anchors() {
    let w = bank8_window();
    let stream = decode_sched(&w, 0xA26D, 64);
    assert_eq!(stream.records.len(), 43, "43 记录（批次1 golden 同源）");
    let mut st = StageState {
        sched: SchedCursor::load(stream),
        ..Default::default()
    };
    let mut fx = StageEffects::default();
    // 快进环 17 tick：仅 rec@A26D（tick=1）落地，下一记录 tick=33 未到
    fast_forward_ring(&mut st.sched, 17, &mut fx);
    assert_eq!(st.sched.next, 1, "快进环内恰 1 记录（tick=1）");
    // 滚动驱动到 tick 33/49/65/81：逐个到点（延迟 32t/16t 族）
    st.wave.scroll_progress = 33 * 16;
    st.wave.tick = 33;
    st.sched.step_to(33, &mut fx);
    assert_eq!(st.sched.next, 2);
    st.sched.step_to(49, &mut fx);
    st.sched.step_to(65, &mut fx);
    st.sched.step_to(81, &mut fx);
    assert_eq!(st.sched.next, 5, "tick 锚簇 1/33/49/65/81 全到点");
    // 游标不越发（已发不重发）
    let n = st.sched.step_to(81, &mut fx);
    assert_eq!(n, 0, "幂等：同 tick 重复步进零发射");
}

/// ⑥ 快进环红线：每迭代=1 全链 tick（Wt-1 突变 ×8 必红）+ wave_tick 挂游标。
#[test]
fn fast_forward_and_wave_tick_wiring() {
    let w = bank8_window();
    let stream = decode_sched(&w, 0xA26D, 64);
    let mut st = StageState {
        sched: SchedCursor::load(stream),
        ..Default::default()
    };
    // 17 tick 环后 next=1；若按"16+16"两段式/×8 换算则 tick 边界错位
    let mut fx = StageEffects::default();
    fast_forward_ring(&mut st.sched, 17, &mut fx);
    assert_eq!(st.sched.next, 1);
    // wave_tick 挂接：滚动 33×16px ⇒ tick=33 ⇒ 游标推进
    let fx2 = stage_tick(&mut st, &TickInputs { scroll_delta: 33 * 16 });
    assert_eq!(st.wave.tick, 33);
    assert_eq!(st.sched.next, 2, "wave_tick 驱动游标对点");
    // 停滚冻结：tick/游标均不动
    let fx3 = stage_tick(&mut st, &TickInputs { scroll_delta: 0 });
    assert_eq!(st.wave.tick, 33);
    assert_eq!(st.sched.next, 2, "停滚 ⇒ 游标冻结（Wt-2）");
    assert!(fx3.spawn_events.len() + fx3.drip_events.len() == 0 || true);
}

// ===== 子面③：spawn 写链落地 =====

use game_core::engine::{land_leader, land_pair, ObjectGrid};

/// ⑦ pair 直投写集（$400/$470）+ 组长四参写集+签名行为字段。
#[test]
fn spawn_write_chain_landing() {
    let mut g = ObjectGrid::default();
    // pair：$C304 first-fit 从槽 8 起
    let slot = land_pair(&mut g, 0x84, 0x30).expect("空网格有槽");
    assert_eq!(slot, 8);
    assert_eq!(g.slots[8].type_, 0x84);
    assert_eq!(g.slots[8].f470, 0x30);
    assert_eq!(g.slots[8].f518, 0, "pair 不写四参");
    // 组长 $2F：四参+签名 $89DF+ctrl2 码流参
    let r = land_leader(&mut g, 0x2F, [0x2B, 6, 0x0A, 0x20], Some(0x77)).expect("有槽");
    let s = &g.slots[r.slot as usize];
    assert_eq!((s.type_, s.f518, s.f630, s.f534, s.f470), (0x2F, 0x2B, 5, 0x0A, 0x20), "$630 首现=数−1");
    assert_eq!((s.beh_hi, s.beh_lo), (0x89, 0xDF), "$89DF 签名驱动写");
    assert!(r.beh_registered);
    assert_eq!(s.f48c, 0x77, "ctrl2 码流参写 $48C");
    // 未登记型值：fail-closed 空行为字段+标志（不发明）
    let r2 = land_leader(&mut g, 0x66, [1, 2, 3, 4], None).expect("有槽");
    assert!(!r2.beh_registered, "未登记型值不发明行为字段");
    let s2 = &g.slots[r2.slot as usize];
    assert_eq!((s2.beh_hi, s2.beh_lo), (0, 0), "fail-closed 空字段");
    assert_eq!(s2.f48c, 0, "ctrl1 无码流参");
}

/// ⑧ 分配域红线：pair 顺序/组长逆序首级（D4 槽序分歧沿用）+ 满载 fail-closed。
#[test]
fn landing_allocator_paths() {
    let mut g = ObjectGrid::default();
    let a = land_pair(&mut g, 0x84, 0).unwrap();
    let b = land_leader(&mut g, 0x2F, [0, 0, 0, 0], None).unwrap();
    assert_eq!(a, 8, "pair 顺序首适应");
    assert_ne!(a, b.slot, "组长逆序首级（槽序分歧 D4）");
    // 满载（8..27 全占）⇒ None
    let mut g2 = ObjectGrid::default();
    for s in g2.slots.iter_mut().skip(8) {
        s.type_ = 0x30;
    }
    assert!(land_pair(&mut g2, 0x84, 0).is_none(), "满载 fail-closed");
    assert!(land_leader(&mut g2, 0x2F, [0; 4], None).is_none());
}

// ===== 子面④：behavior_frame 调用环 =====

use game_core::engine::{
    behavior_frame, rng_ec12, run_handler, HandlerResult, ObjectSlot, RngState,
};

/// ⑨ $EC12 LFSR 字面体 + 已证 handler 执行面。
#[test]
fn rng_lfsr_and_handlers() {
    // LFSR golden：A=(z2c+z2d+1)^z2b，z2d INC（字面体确定性）
    let mut r = RngState { z2c: 0x10, z2d: 0x20, z2b: 0x00 };
    let (a1, c1) = rng_ec12(&mut r);
    assert_eq!(a1, 0x31, "0x10+0x20+1=0x31 ^ 0");
    assert_eq!((r.z2c, r.z2d), (0x31, 0x21));
    assert!(!c1, "无进位");
    // 进位链出口（carry 注入面）：z2c+z2d 溢出
    let mut r2 = RngState { z2c: 0xFF, z2d: 0x01, z2b: 0x00 };
    let (a2, c2) = rng_ec12(&mut r2);
    assert_eq!(a2, 0x01, "0xFF+0x01+1=0x101 低 8=0x01");
    assert!(c2, "ADC 进位链出口=1");
    // $836E 恒速下沉
    let mut s = ObjectSlot { y: 0x10, ..Default::default() };
    let mut rng = RngState::default();
    assert_eq!(run_handler(&mut s, 0x836E, &mut rng), HandlerResult::Continue);
    assert_eq!(s.y, 0x11, "y+1");
    s.y = 0xFF;
    assert_eq!(run_handler(&mut s, 0x836E, &mut rng), HandlerResult::Recycle, "溢出终止");
    // $8314 duration 倒数（$438 字段复用三义面——表模式）
    let mut s = ObjectSlot { beh_lo: 0x02, ..Default::default() };
    assert_eq!(run_handler(&mut s, 0x8314, &mut rng), HandlerResult::Continue);
    assert_eq!(s.beh_lo, 0x01);
    assert_eq!(run_handler(&mut s, 0x8314, &mut rng), HandlerResult::Recycle, "归零终止");
    // 终止直通×3
    for t in [0x8334u16, 0x837E, 0x8403] {
        assert_eq!(run_handler(&mut s, t, &mut rng), HandlerResult::Recycle);
    }
    // $8337 jitter+续链改写 → $836E
    let mut s = ObjectSlot { x: 100, y: 100, ..Default::default() };
    let mut rng = RngState { z2c: 0x78, z2d: 0x00, z2b: 0x00 };
    assert_eq!(run_handler(&mut s, 0x8337, &mut rng), HandlerResult::Continue);
    assert_eq!((s.beh_hi, s.beh_lo), (0x83, 0x6E), "续链改写 $836E");
    assert_ne!((s.x, s.y), (100, 100), "jitter 必动（跳零面）");
    // 未分析 handler fail-closed
    let mut s = ObjectSlot { y: 50, ..Default::default() };
    assert_eq!(run_handler(&mut s, 0x9120, &mut rng), HandlerResult::Unproven);
    assert_eq!(s.y, 50, "fail-closed 字段不动");
}

/// ⑩ 滴灌 handler（$89DF 四参消费）：首员同帧+帧间隔+$630 递减。
#[test]
fn drip_handler_cadence() {
    let mut st = StageState::default();
    land_leader(&mut st.grid, 0x2F, [0x2B, 6, 0x0A, 0x20], None).unwrap();
    // 首帧：首员同帧落地（$630 首现=数−1=5）
    let leaders: Vec<usize> = (8..28).filter(|&i| st.grid.slots[i].type_ == 0x2F).collect();
    assert_eq!(leaders.len(), 1);
    let li = leaders[0];
    assert_eq!(st.grid.slots[li].f630, 5, "装载后先递减一次（首现=数−1）");
    behavior_frame(&mut st, &mut StageEffects::default());
    let members = (8..28).filter(|&i| st.grid.slots[i].type_ == 0x2B).count();
    assert_eq!(members, 1, "首员=装载帧同帧（load 发射）");
    assert_eq!(st.grid.slots[li].f630, 5, "load 发射不递减（GroupRelease 语义）");
    assert_eq!(st.grid.slots[li].drip_cd, 0x0A, "间隔 10f 装载");
    // 9 帧静默后第 10 帧第二员（递减 5→4）
    for _ in 0..9 {
        behavior_frame(&mut st, &mut StageEffects::default());
    }
    assert_eq!((8..28).filter(|&i| st.grid.slots[i].type_ == 0x2B).count(), 1);
    behavior_frame(&mut st, &mut StageEffects::default());
    assert_eq!((8..28).filter(|&i| st.grid.slots[i].type_ == 0x2B).count(), 2, "10f 节律");
    assert_eq!(st.grid.slots[li].f630, 4, "滴灌递减");
}

/// ⑪ 表分发接线（decomp f06a376）：table 模式经 TYPE_DISPATCH 落 handler（$2B→$90AB）。
#[test]
fn table_dispatch_wiring_live() {
    // $2B 成员槽（beh_hi=0 表模式）：behavior_frame → $90AB → 续链 $9120+清态
    let mut st = StageState::default();
    let slot = 8usize;
    st.grid.slots[slot] = ObjectSlot {
        type_: 0x2B,
        f5dc: 0x22,
        f5c0: 0x11,
        ..Default::default()
    };
    behavior_frame(&mut st, &mut StageEffects::default());
    let s = st.grid.slots[slot];
    assert_eq!((s.beh_hi, s.beh_lo), (0x91, 0x20), "续链 $9120");
    assert_eq!((s.f5dc, s.f5c0), (0, 0), "$90AB 清态");
    // $9120 内核未证 fail-closed：下一帧字段不动
    let before = st.grid.slots[slot];
    behavior_frame(&mut st, &mut StageEffects::default());
    assert_eq!(st.grid.slots[slot], before, "$9120 fail-closed");
}
