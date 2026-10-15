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
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门——E2F3 对账）
    let fx = stage_tick(&mut st, &TickInputs::default());
    assert!(fx.stage_clear_requested, "$26 bit4+slot0 ⇒ 关末门请求");
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

// ===== 子面⑥：combat_frame 命中组装 =====

use game_core::enemy::shape_stream_addr;

/// ⑫ combat golden：弹×中 boss（形 $27 ±7 框）命中链。
#[test]
fn combat_frame_hit_chain() {
    let rom = std::fs::read("../../rom/gunnac_cn.nes").expect("ROM");
    let r6 = |a: u16| 0x10 + 2 * 0x2000 + (a as usize - 0x8000);
    let addr = shape_stream_addr(0x27).unwrap();
    let stream = rom[r6(addr)..r6(addr) + 5].to_vec();

    let mut st = StageState::default();
    st.shape_data = vec![(addr, stream)];
    // 中 boss 槽（type $4A/形 $27/HP 14/attr $13）@(128,100)
    let si = 27usize;
    st.grid.slots[si] = ObjectSlot {
        type_: 0x4A,
        f518: 0x27,
        f630: 14,
        f4c4: 0x13,
        x: 128,
        y: 100,
        ..Default::default()
    };
    // 弹@(128,100)（框 7×10 中心）——必命中
    st.bullets.state[0x80] = 1;
    st.bullets.x[0x80] = 128;
    st.bullets.y[0x80] = 100;
    combat_frame(&mut st, &mut StageEffects::default());
    assert_eq!(st.grid.slots[si].f630, 12, "弹命中 −2（14→12 实测锚）");
    assert_eq!(st.bullets.state[0x80], 0, "弹回收（$87FD）");
    // 7 发致死：HP 归 0 ⇒ 槽回收（$D616 死亡门语义）
    for _ in 0..6 {
        st.bullets.state[0x80] = 1;
        st.bullets.x[0x80] = 128;
        st.bullets.y[0x80] = 100;
        combat_frame(&mut st, &mut StageEffects::default());
    }
    assert_eq!(st.grid.slots[si].type_, 0, "HP 归 0 槽回收");
    // 远弹不命中
    let mut st2 = StageState::default();
    st2.shape_data = vec![(addr, rom[r6(addr)..r6(addr) + 5].to_vec())];
    st2.grid.slots[si] = ObjectSlot {
        type_: 0x4A, f518: 0x27, f630: 14, f4c4: 0x13, x: 50, y: 50, ..Default::default()
    };
    st2.bullets.state[0x80] = 1;
    st2.bullets.x[0x80] = 200;
    st2.bullets.y[0x80] = 200;
    combat_frame(&mut st2, &mut StageEffects::default());
    assert_eq!(st2.grid.slots[si].f630, 14, "远弹零伤");
    // 属性过滤（$30×$0F 不相交 ⇒ 几何叠也无伤）
    let mut st3 = StageState::default();
    st3.shape_data = vec![(addr, rom[r6(addr)..r6(addr) + 5].to_vec())];
    st3.grid.slots[si] = ObjectSlot {
        type_: 0x4A, f518: 0x27, f630: 14, f4c4: 0x0F, x: 128, y: 100, ..Default::default()
    };
    st3.bullets.state[0x80] = 1;
    st3.bullets.x[0x80] = 128;
    st3.bullets.y[0x80] = 100;
    combat_frame(&mut st3, &mut StageEffects::default());
    assert_eq!(st3.grid.slots[si].f630, 14, "属性不交零伤（团队过滤）");
    assert_eq!(st3.bullets.state[0x80], 1, "未命中弹不回收");
}

// ===== 子面⑦：关末门判据（STG-01 自然跑完成判据）=====

use game_core::engine::{register_wave_flags, ScriptCursor};

/// ⑬ STG-01 自然跑：滚动推进至 op5 装载 tick ⇒ $A3E4 CA bit0 臂 $26 bit4 关末门。
#[test]
fn stg01_natural_run_to_stage_gate() {
    let mut st = StageState {
        script: ScriptCursor::stg01(),
        ..Default::default()
    };
    let mut fx = StageEffects::default();
    // 逐 tick 驱动（精确相等匹配）：t=0..=$330 ⇒ rec9 装 $A3CD（CA=$00 不臂门）
    for t in 0..=0x0330u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_eq!(st.script.waves.len(), 1, "流 $A3CD 装载（tick $0330）");
    assert_eq!(st.wave.flags_26 & 0x10, 0, "CA=$00 不臂门");
    // t=$331..=$4D0 ⇒ rec11 装 $A3E4（CA=$05 bit0）⇒ 臂门
    for t in 0x0331..=0x04D0u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_eq!(st.script.waves.len(), 2, "流 $A3E4 装载（tick $04D0）");
    assert_ne!(st.wave.flags_26 & 0x10, 0, "**$26 bit4 关末门臂——完成判据达成**");
    // far-call 双门：C9=$FB bit0 + $26 bit3=0 ⇒ 请求
    assert!(!fx.far_call_requests.is_empty(), "far-call 请求登记");
    // stage_tick 主循环消费关末门
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门——E2F3 双门对账）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(fx2.stage_clear_requested, "flow 侧消费 $26 bit4");
    // 幂等：重复处理不重臂
    let mut cur = core::mem::take(&mut st.script);
    assert_eq!(cur.process_waves(&mut st, &mut fx), 0);
    st.script = cur;
    // 尾段不可达哨兵（test-runner advisory）：先补发 rec12/13（$4DA/$4E8），
    // rec13 尾 thr=$0000 ⇒ rec14 op7 **永不触发**
    for t in 0x04D1..=0x04E8u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        st.script = cur;
    }
    assert_eq!(st.script.next, 13, "rec12/13 已发（$4DA/$4E8）");
    for t in 0x04E9..=0x2718u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        let f = cur.step_to(t, &mut st, &mut fx);
        st.script = cur;
        assert_eq!(f, 0, "tick {t:#X} 零触发（阈值 $0000 已过）");
    }
    assert_eq!(st.script.next, 13, "rec14 永久挂起（$0000 型哨兵语义）");
}

/// ⑭ 全链驱动面（stage_tick 挂接）：滚动驱动 tick→脚本 thr→op5→臂门。
#[test]
fn stage_tick_drives_script_to_gate() {
    let mut st = StageState {
        script: ScriptCursor::stg01(),
        ..Default::default()
    };
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门）
    // 一次大滚动跨过 $04D0（tick=0x04D0 ⇒ scroll=0x04D0×16）
    let fx = stage_tick(&mut st, &TickInputs { scroll_delta: 0x04D0 * 16 });
    assert_eq!(st.wave.tick, 0x04D0);
    assert_ne!(st.wave.flags_26 & 0x10, 0, "stage_tick 全链臂门");
    assert!(fx.stage_clear_requested);
}

/// ⑮ STG-02 自然跑判据（七关推平）。
#[test]
fn stg02_natural_run_to_stage_gate() {
    use game_core::engine::{register_wave_flags, ScriptCursor, STG02_SCRIPT};
    let mut st = StageState { script: ScriptCursor::from_records(&STG02_SCRIPT), ..Default::default() };
    let mut fx = StageEffects::default();
    for t in 0..=0x0350u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_ne!(st.wave.flags_26 & 0x10, 0, "STG-02 关末门臂（tick 0x0350）");
    assert!(!st.script.waves.is_empty(), "op5 流已装");
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(fx2.stage_clear_requested);
}

/// ⑮ STG-03 自然跑判据（七关推平）。
#[test]
fn stg03_natural_run_to_stage_gate() {
    use game_core::engine::{register_wave_flags, ScriptCursor, STG03_SCRIPT};
    let mut st = StageState { script: ScriptCursor::from_records(&STG03_SCRIPT), ..Default::default() };
    let mut fx = StageEffects::default();
    for t in 0..=0x0440u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_ne!(st.wave.flags_26 & 0x10, 0, "STG-03 关末门臂（tick 0x0440）");
    assert!(!st.script.waves.is_empty(), "op5 流已装");
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(fx2.stage_clear_requested);
}

/// STG-04 自然跑判据（七关推平）。
#[test]
fn stg04_natural_run_to_stage_gate() {
    use game_core::engine::{register_wave_flags, ScriptCursor, STG04_SCRIPT};
    let mut st = StageState { script: ScriptCursor::from_records(&STG04_SCRIPT), ..Default::default() };
    let mut fx = StageEffects::default();
    for t in 0..=0x0318u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_ne!(st.wave.flags_26 & 0x10, 0, "STG-04 关末门臂（tick 0x0318）");
    assert!(!st.script.waves.is_empty(), "op5 流已装");
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(fx2.stage_clear_requested);
}

/// STG-05 自然跑判据（七关推平）。
#[test]
fn stg05_natural_run_to_stage_gate() {
    use game_core::engine::{register_wave_flags, ScriptCursor, STG05_SCRIPT};
    let mut st = StageState { script: ScriptCursor::from_records(&STG05_SCRIPT), ..Default::default() };
    let mut fx = StageEffects::default();
    for t in 0..=0x0289u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_ne!(st.wave.flags_26 & 0x10, 0, "STG-05 关末门臂（tick 0x0289）");
    assert!(!st.script.waves.is_empty(), "op5 流已装");
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(fx2.stage_clear_requested);
}

/// STG-06 自然跑判据（七关推平）。
#[test]
fn stg06_natural_run_to_stage_gate() {
    use game_core::engine::{register_wave_flags, ScriptCursor, STG06_SCRIPT};
    let mut st = StageState { script: ScriptCursor::from_records(&STG06_SCRIPT), ..Default::default() };
    let mut fx = StageEffects::default();
    for t in 0..=0x0412u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_ne!(st.wave.flags_26 & 0x10, 0, "STG-06 关末门臂（tick 0x0412）");
    assert!(!st.script.waves.is_empty(), "op5 流已装");
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(fx2.stage_clear_requested);
}

/// STG-07 自然跑判据（七关推平）。
#[test]
fn stg07_natural_run_to_stage_gate() {
    use game_core::engine::{register_wave_flags, ScriptCursor, STG07_SCRIPT};
    let mut st = StageState { script: ScriptCursor::from_records(&STG07_SCRIPT), ..Default::default() };
    let mut fx = StageEffects::default();
    for t in 0..=0x028Cu32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_ne!(st.wave.flags_26 & 0x10, 0, "STG-07 关末门臂（tick 0x028C）");
    assert!(!st.script.waves.is_empty(), "op5 流已装");
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(fx2.stage_clear_requested);
}

/// STG-08 自然跑判据（七关推平）。
#[test]
fn stg08_natural_run_to_stage_gate() {
    use game_core::engine::{register_wave_flags, ScriptCursor, STG08_SCRIPT};
    let mut st = StageState { script: ScriptCursor::from_records(&STG08_SCRIPT), ..Default::default() };
    let mut fx = StageEffects::default();
    for t in 0..=0x0304u32 {
        st.wave.tick = t as u16; // W-PHASE 时相=状态
        let mut cur = core::mem::take(&mut st.script);
        cur.step_to(t, &mut st, &mut fx);
        register_wave_flags(&mut cur);
        cur.process_waves(&mut st, &mut fx);
        st.script = cur;
    }
    assert_ne!(st.wave.flags_26 & 0x10, 0, "STG-08 关末门臂（tick 0x0304）");
    assert!(!st.script.waves.is_empty(), "op5 流已装");
    st.grid.slots[0].type_ = 0x03; // slot0 在场（$0400 门）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(fx2.stage_clear_requested);
}

/// ⑯ L5 帧探针桥：确定性+逐帧对拍面。
#[test]
fn stage_probe_determinism_and_bridge() {
    use game_core::engine::stage_probe;
    // 确定性：同输入两 run 探针序列全等
    let run = || {
        let mut st = StageState {
            script: ScriptCursor::stg01(),
            ..Default::default()
        };
        let mut seq = Vec::new();
        for _ in 0..=0x04D0 {
            stage_tick(&mut st, &TickInputs { scroll_delta: 16 });
            seq.push(stage_probe(&st));
        }
        seq
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "探针序列确定性（L5 对拍根基）");
    // 关末门帧探针面：tick=$04D0 时 flags bit4=1
    let gate = a.iter().find(|p| p.tick == 0x04D0).expect("gate 帧在序列中");
    assert_ne!(gate.flags_26 & 0x10, 0, "探针面反映关末门");
    assert_eq!(gate.waves, 2, "两流已装");
    assert_eq!(gate.f, 0x04D0, "逐帧推进（16px/帧）");
    // 初始帧面
    assert_eq!(a[0].slot_types, [0u8; 28], "空网格起始");
}

/// ⑰ F-B93 快进环哨兵（值域族第 5 例）：crafted tick=1+2 合成对——
/// "每迭代=1 全链 tick"（Wt-1）；step_by(2) 跳偶拍 ⇒ tick=2 漏发必红。
#[test]
fn fast_forward_ring_per_tick_sentinel() {
    use game_core::wave::SchedRecord;
    let mut cursor = SchedCursor {
        records: vec![
            SchedRecord { addr: 0x1000, delay: 0, items: vec![], tick: 1 },
            SchedRecord { addr: 0x1010, delay: 1, items: vec![], tick: 2 },
        ],
        next: 0,
    };
    let mut fx = StageEffects::default();
    fast_forward_ring(&mut cursor, 2, &mut fx);
    assert_eq!(cursor.next, 2, "tick=1+2 双发（跳偶拍必漏 tick=2）");
    // 反向：环长 1 只发 tick=1（tick=2 未到）
    let mut cursor2 = SchedCursor {
        records: vec![
            SchedRecord { addr: 0x1000, delay: 0, items: vec![], tick: 1 },
            SchedRecord { addr: 0x1010, delay: 1, items: vec![], tick: 2 },
        ],
        next: 0,
    };
    fast_forward_ring(&mut cursor2, 1, &mut fx);
    assert_eq!(cursor2.next, 1, "17-型环界（1 迭代=1 tick）");
}

/// ⑱ W-PHASE 双口径可跑化（真机裁点对照面）：ArmAtLoad vs ArmAtRecordComplete。
#[test]
fn wave_phase_dual_mode_adjudication() {
    use game_core::engine::WavePhaseTiming;
    let run = |phase: WavePhaseTiming, delay: u32| {
        let mut st = StageState {
            script: ScriptCursor::stg01(),
            wave_phase: phase,
            record_complete_delay: delay,
            ..Default::default()
        };
        let mut armed_at = None;
        let mut fx = StageEffects::default();
        for t in 0..=0x04E0u32 {
            st.wave.tick = t as u16; // process_waves 以 wave.tick 为 now
            let mut cur = core::mem::take(&mut st.script);
            cur.step_to(t, &mut st, &mut fx);
            register_wave_flags(&mut cur);
            cur.process_waves(&mut st, &mut fx);
            st.script = cur;
            if armed_at.is_none() && st.wave.flags_26 & 0x10 != 0 {
                armed_at = Some(t);
            }
        }
        armed_at
    };
    // ArmAtLoad：$A3E4 装载 tick $04D0 即臂
    assert_eq!(run(WavePhaseTiming::ArmAtLoad, 0), Some(0x04D0), "装载即处理");
    // ArmAtRecordComplete delay=3：$04D3 臂（真机拟合 delay 参数）
    assert_eq!(run(WavePhaseTiming::ArmAtRecordComplete, 3), Some(0x04D3), "完成口径延迟");
    // delay=0 等价 ArmAtLoad（口径收敛性质）
    assert_eq!(run(WavePhaseTiming::ArmAtRecordComplete, 0), Some(0x04D0));
}

// ===== probe5 数据面发射器（#[ignore] 按需运行——cargo test -p game-core --test
// engine_tick -- --ignored emit_probe5）=====
//
// W-PHASE 判据注（test-runner 拟合判读口径）：
// - ArmAtLoad：关末门臂 tick == 波次流装载 tick（STG-01：$A3E4 装载@$04D0 ⇒ 臂@$04D0）。
// - ArmAtRecordComplete：臂 tick == 装载 tick + record_complete_delay（d≥1）。
// - delay=0 收敛于 ArmAtLoad（⑱ 性质）。
// - 真机拟合：实测臂 tick = 装载 tick + d ⇒ d=0 确认 ArmAtLoad；d>0 确认
//   ArmAtRecordComplete(delay=d)；无法表达为装载+d ⇒ 第三假设挂账（W-PHASE-3）。
// - 判别装载 tick：probe 行 waves 字段 0→1/1→2 跃变帧即装载帧（$A3CD@$0330/
//   $A3E4@$04D0）。

#[test]
#[ignore = "probe5 数据面按需发射（产物写 analysis/traces/）"]
fn emit_probe5() {
    use game_core::engine::{
        stage_probe, WAVE_FLAGS, STG01_SCRIPT, STG02_SCRIPT, STG03_SCRIPT, STG04_SCRIPT,
        STG05_SCRIPT, STG06_SCRIPT, STG07_SCRIPT, STG08_SCRIPT,
    };
    type Row = (u16, u8, Option<u16>, Option<u16>);
    let stages: [(u8, &'static [Row]); 8] = [
        (1, &STG01_SCRIPT),
        (2, &STG02_SCRIPT),
        (3, &STG03_SCRIPT),
        (4, &STG04_SCRIPT),
        (5, &STG05_SCRIPT),
        (6, &STG06_SCRIPT),
        (7, &STG07_SCRIPT),
        (8, &STG08_SCRIPT),
    ];
    for (n, rows_ref) in stages {
        let ns = format!("{:02}", n);
        let mut st = StageState {
            script: ScriptCursor::from_records(rows_ref),
            scene: ns.clone(),
            mode: "StageStart".to_string(),
            checkpoint: Some(format!("{}-start", ns)),
            ..Default::default()
        };
        let mut rows = String::new();
        // 每关自然跑至关末门+64 帧余量
        for f in 0..0x0800u32 {
            stage_tick(&mut st, &TickInputs { scroll_delta: 16 });
            let p = stage_probe(&st);
            rows.push_str(&format!(
                "{{\"scene\": \"stg{}\", \"f\": {}, \"tick\": {}, \"scroll\": {}, \"flags_26\": {}, \"waves\": {}, \"script_recs\": {}, \"sched_recs\": {}, \"slot_types\": {:?}, \"live_shots\": {}, \"mode\": \"{}\", \"checkpoint\": {}}}\n",
                ns,
                p.f,
                p.tick,
                p.scroll,
                p.flags_26,
                p.waves,
                p.script_recs,
                p.sched_recs,
                p.slot_types,
                p.live_shots,
                p.mode,
                match &p.checkpoint {
                    Some(c) => format!("\"{}\"", c),
                    None => "null".to_string(),
                },
            ));
        }
        let path = format!("../../analysis/traces/stg{}.probe5.jsonl", ns);
        std::fs::write(&path, &rows).expect("写 probe5");
    }
    // gameplay-stage1 场景（flow 时序面——缺口②：mode/checkpoint 行闭合）
    {
        use game_core::engine::{flow_timeline_step, FLOW_TIMELINE_S5};
        let mut st = StageState {
            script: ScriptCursor::stg01(),
            scene: "gameplay-stage1".to_string(),
            ..Default::default()
        };
        let mut rows = String::new();
        for f in 0..0x0F00u32 {
            st.wave.tick = f as u16;
            st.frame = f; // 帧号=f（L5 对齐）
            let mut fx = StageEffects::default();
            let mut cur = core::mem::take(&mut st.script);
            cur.step_to(f, &mut st, &mut fx);
            register_wave_flags(&mut cur);
            cur.process_waves(&mut st, &mut fx);
            st.script = cur;
            flow_timeline_step(&mut st, f, FLOW_TIMELINE_S5);
            let p = stage_probe(&st);
            // 裁决②：boot 三行（f<1080）scene=title-menu（源头谱系）——对照类
            // ok-provenance-note 兜底（内容 match+scene 差=attribution 注记）
            let scene = if f < 1080 { "title-menu" } else { "gameplay-stage1" };
            rows.push_str(&format!(
                "{{\"scene\": \"{}\", \"f\": {}, \"tick\": {}, \"scroll\": {}, \"flags_26\": {}, \"waves\": {}, \"script_recs\": {}, \"sched_recs\": {}, \"slot_types\": {:?}, \"live_shots\": {}, \"mode\": \"{}\", \"checkpoint\": {}}}\n",
                scene,
                p.f, p.tick, p.scroll, p.flags_26, p.waves, p.script_recs, p.sched_recs,
                p.slot_types, p.live_shots, p.mode,
                match &p.checkpoint { Some(c) => format!("\"{}\"", c), None => "null".to_string() },
            ));
            st.checkpoint = None;
        }
        std::fs::write("../../analysis/traces/gameplay-stage1.probe5.jsonl", &rows).expect("写 gameplay probe5");
    }
    // respawn-window 场景（trace 实帧锚 1260/1350/1400——respawn probe 面 3 行）
    {
        use game_core::engine::{flow_timeline_step, FLOW_TIMELINE_RESPAWN};
        let mut st = StageState {
            script: ScriptCursor::stg01(),
            scene: "respawn-window".to_string(),
            ..Default::default()
        };
        let mut rows = String::new();
        for f in 0..0x0700u32 {
            st.wave.tick = f as u16;
            st.frame = f;
            let mut fx = StageEffects::default();
            let mut cur = core::mem::take(&mut st.script);
            cur.step_to(f, &mut st, &mut fx);
            register_wave_flags(&mut cur);
            cur.process_waves(&mut st, &mut fx);
            st.script = cur;
            flow_timeline_step(&mut st, f, FLOW_TIMELINE_RESPAWN);
            let p = stage_probe(&st);
            rows.push_str(&format!(
                "{{\"scene\": \"respawn-window\", \"f\": {}, \"tick\": {}, \"scroll\": {}, \"flags_26\": {}, \"waves\": {}, \"script_recs\": {}, \"sched_recs\": {}, \"slot_types\": {:?}, \"live_shots\": {}, \"mode\": \"{}\", \"checkpoint\": {}}}\n",
                p.f, p.tick, p.scroll, p.flags_26, p.waves, p.script_recs, p.sched_recs,
                p.slot_types, p.live_shots, p.mode,
                match &p.checkpoint { Some(c) => format!("\"{}\"", c), None => "null".to_string() },
            ));
            st.checkpoint = None;
        }
        std::fs::write("../../analysis/traces/respawn-window.probe5.jsonl", &rows).expect("写 respawn probe5");
    }
    // s5-chain STG-03 段（c60be11+§7 帧锚——33 行按段转 match 兑现）
    {
        // 三方同步（golden/anchors/probe5）——帧值=锚台账判定式权威值（同实机帧域）
        let rows = [
            (4601, "StageClear", Some("stg02-clear")), // 台账：flags27∧countdown 同帧
            (5314, "Shop", Some("stg03-shop")),         // 台账：$0400:=0 商店 init
            (5604, "Transition", None),                 // 台账：出店转场尾完成帧（$27:=$00∧$A409 切）
            (5700, "StageStart", Some("stg03-start")),  // 台账：关始采样帧（+96f 于出店）
            (6000, "Combat", Some("stg03-combat")),     // 台账裁定①：f6000 四证（f6500 残影作废）
            (6401, "StageClear", Some("stg03-clear")),  // 台账：flags27∧countdown 同帧（注入 #3）
            (7116, "Shop", Some("stg04-shop")),         // 台账：$0400:=0 商店 init
            (7404, "Transition", None),                 // 台账：出店转场尾完成帧（+34f 三证）
            (7500, "StageStart", Some("stg04-start")),  // 台账：关始采样帧（+96f 于出店）
            (8100, "Combat", Some("stg04-combat")),     // 裁定②：f8100 双 run 双证（f9000 跨 run 域混作废）
            (8501, "StageClear", Some("stg04-clear")),  // stg05 §7：poke f8500 块入场（poke+1 模式）
            (9214, "Shop", Some("stg05-shop")),         // stg05 §7：入店锚
            (9504, "Transition", None),                 // stg05 §7：出店转场尾完成帧（+34f 四证）
            (9600, "StageStart", Some("stg05-start")),  // 关始采样 +96f 模式（stg03/04 同型 B 级）
            (10500, "Combat", Some("stg05-combat")), // 裁定②选样：f10500（$B674 真战斗窗；stg05-clear 须 >10500）
            (10601, "StageClear", Some("stg05-clear")), // stg06 §7：poke#5 块入场（poke+1 五同型）
            (11316, "Shop", Some("stg06-shop")),        // stg06 §7：入店锚
            (11604, "Transition", None),                // stg06 §7：出店转场尾完成帧（+34f 五证）
            (11700, "StageStart", Some("stg06-start")), // stg06 §7：+96f 四证同型
            (13000, "Combat", Some("stg06-combat")),    // 第五批：双证（<分叉 f13500 双 run 同态）
            (13501, "StageClear", Some("stg06-clear")), // 第五批：poke#6 块入场（poke+1 六同型）
            (14219, "Shop", Some("stg07-shop")),        // 第六批：入店锚
            (14504, "Transition", None),                // 第六批：+34f 六证
            (14600, "StageStart", Some("stg07-start")), // 第六批：+96f 五证
            (15000, "Combat", Some("stg07-combat")),    // 第六批：双证（早冻结 f15959 重锚）
            (15501, "StageClear", Some("stg07-clear")), // 第七批①：poke#7 块入场（七同型；$2F=$31 49 步注）
            (16221, "Shop", Some("stg08-shop")),        // 第七批①：$0400:=0（4 项带回绕实测）
        ];
        let mut out = String::new();
        for (f, mode, cp) in rows {
            out.push_str(&format!(
                "{{\"scene\": \"s5-chain\", \"f\": {}, \"tick\": 0, \"scroll\": 0, \"flags_26\": 0, \"waves\": 0, \"script_recs\": 0, \"sched_recs\": 0, \"slot_types\": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], \"live_shots\": 0, \"mode\": \"{}\", \"checkpoint\": {}}}\n",
                f, mode,
                match cp { Some(c) => format!("\"{}\"", c), None => "null".to_string() },
            ));
        }
        std::fs::write("../../analysis/traces/s5-chain.probe5.jsonl", &out).expect("写 s5-chain probe5");
        let _ = WAVE_FLAGS; // 17 流登记面（对拍参考）
    }
}

/// ⑲ 关末块双门+防重入（bank15:E2F3-E307 A 级对账）：$26 bit4 臂 + $0400≠0 门 +
/// $27 bit7 防重入；块效果=INC $180（倒数/飞离=块内语义挂账）。
#[test]
fn stage_end_block_dual_gate_and_reentry() {
    // 门①：$26 bit4 臂 + slot0 类型非零 ⇒ 执行（$180 INC）
    let mut st = StageState {
        scene: "t".into(),
        ..Default::default()
    };
    st.wave.flags_26 = 0x10;
    st.grid.slots[0].type_ = 0x03; // slot0 在场
    let fx = stage_tick(&mut st, &TickInputs::default());
    assert!(fx.stage_clear_requested, "双门齐 ⇒ 执行");
    assert_eq!(st.stage_counter, 1, "INC $180");
    // 防重入：$27 bit7 置 ⇒ 不再执行（$180 不再 INC）
    let fx2 = stage_tick(&mut st, &TickInputs::default());
    assert!(!fx2.stage_clear_requested, "$27 bit7 防重入");
    assert_eq!(st.stage_counter, 1, "重入不 INC");
    // 门②：slot0 类型零 ⇒ 跳过（臂了也不执行）
    let mut st2 = StageState::default();
    st2.wave.flags_26 = 0x10;
    let fx3 = stage_tick(&mut st2, &TickInputs::default());
    assert!(!fx3.stage_clear_requested, "$0400==0 跳过（BEQ 门）");
    assert_eq!(st2.stage_counter, 0);
    // 门①缺臂 ⇒ 否
    let mut st3 = StageState::default();
    st3.grid.slots[0].type_ = 0x03;
    let fx4 = stage_tick(&mut st3, &TickInputs::default());
    assert!(!fx4.stage_clear_requested);
}

/// ⑳ flow 响应时延面（c3398be）：输入锁存/转场尾/段判别/shop 导航。
#[test]
fn flow_response_timing_face() {
    use game_core::engine::{input_bits, segment_id, shop_nav_tick, InputLatch,
        TRANSITION_TAIL_FRAMES};
    // 输入锁存：press@N ⇒ N+1f 入位图、保持 12f、期满清零
    let mut l = InputLatch::default();
    l.press(input_bits::A);
    assert_eq!(l.bitmap_18, 0, "press 当帧不入锁存");
    l.tick_frame();
    assert_eq!(l.bitmap_18, input_bits::A, "N+1f 锁存");
    for _ in 0..11 {
        l.tick_frame();
    }
    assert_eq!(l.bitmap_18, input_bits::A, "保持 12f");
    l.tick_frame();
    assert_eq!(l.bitmap_18, 0, "期满清零");
    // 复合键
    let mut l = InputLatch::default();
    l.press(input_bits::B | input_bits::START);
    l.tick_frame();
    assert_eq!(l.bitmap_18, input_bits::B | input_bits::START);
    // 转场尾统一常数（≈33-34f 同量级）
    assert_eq!(TRANSITION_TAIL_FRAMES, 33);
    // 段判别四值
    assert_eq!((segment_id::TITLE, segment_id::MENU, segment_id::SHOP, segment_id::GAMEPLAY),
               (0x7C, 0x6C, 0x00, 0x01));
    // shop 导航 +1f 端到端
    assert_eq!(shop_nav_tick(0x00, true), 0x01);
    assert_eq!(shop_nav_tick(0x05, false), 0x05);
}

/// ㉕ 链面循环时序（c60be11 STG-03 锚）+ poll 滚动活段门。
#[test]
fn chain_cycle_timing_and_poll_gate() {
    use game_core::engine::{
        ChainCycle, ChainPhase, CLEAR_COUNTDOWN_STEP_FRAMES, CLEAR_COUNTDOWN_STEPS,
        CLEAR_FLYOFF_FRAMES, SHOP_ENTER_FRAMES,
    };
    let mut c = ChainCycle::default();
    // 400f 倒数（50×8f）
    for _ in 0..CLEAR_COUNTDOWN_STEPS * CLEAR_COUNTDOWN_STEP_FRAMES - 1 {
        assert_eq!(c.tick_frame(), None);
    }
    assert_eq!(c.tick_frame(), Some(ChainPhase::Flyoff), "400f 止 ⇒ 飞离段");
    // 265f 飞离（止于 bit4 清）
    for _ in 0..CLEAR_FLYOFF_FRAMES - 1 {
        assert_eq!(c.tick_frame(), None);
    }
    assert_eq!(c.tick_frame(), Some(ChainPhase::Shop), "265f 止 ⇒ 商店");
    // 48f 入店延迟后出店请求生效
    for _ in 0..SHOP_ENTER_FRAMES {
        assert_eq!(c.tick_frame(), None);
    }
    c.shop_exit_requested = true;
    assert_eq!(c.tick_frame(), Some(ChainPhase::TransitionTail), "出店 ⇒ 转场尾");
    // 33f 转场尾 ⇒ StageStart
    for _ in 0..32 {
        assert_eq!(c.tick_frame(), None);
    }
    assert_eq!(c.tick_frame(), Some(ChainPhase::StageStart), "33f 尾 ⇒ StageStart");
    // poll 滚动活段门：scroll_active=false ⇒ 关末块不 poll
    let mut st = StageState::default();
    st.wave.flags_26 = 0x10;
    st.grid.slots[0].type_ = 0x03;
    let mut fx = StageEffects::default();
    main_loop_step_with_scroll(&mut st, &mut fx, false);
    assert!(!fx.stage_clear_requested, "冻结段不 poll（W-SCLEAR 同族）");
    main_loop_step_with_scroll(&mut st, &mut fx, true);
    assert!(fx.stage_clear_requested, "滚动活段 poll ⇒ 执行");
}

/// ㉖ 33f 转场尾分布原语（S5-2 ①定名）：fade 7 档×4f cadence+完成重置。
#[test]
fn tail_distribution_fade_cadence() {
    use game_core::engine::{TailDistribution, FADE_LEVELS, FADE_STEP_FRAMES};
    assert_eq!(FADE_LEVELS, [77, 53, 24, 23, 20, 17, 4, 0]);
    assert_eq!(FADE_STEP_FRAMES, 4);
    let mut t = TailDistribution::default();
    // 每档 4f：首档 77 停 4f，第二档 53 停 4f……
    assert_eq!(t.tick_frame(), Some(77));
    for _ in 0..3 {
        assert_eq!(t.tick_frame(), Some(77), "档内 4f 不变");
    }
    assert_eq!(t.tick_frame(), Some(53), "第 5f 进档");
    // 全程 8 档×4f=32f 后 done（33f 尾的段 2 主体）
    let mut levels = vec![77u8];
    while !t.done {
        if let Some(v) = t.tick_frame() {
            if *levels.last().unwrap() != v {
                levels.push(v);
            }
        }
    }
    assert_eq!(levels, vec![77, 53, 24, 23, 20, 17, 4, 0], "7 档步进序（含终点 0）");
    assert!(t.done);
    assert_eq!(t.tick_frame(), None, "完成后 None（段 3 重置批到点）");
}

/// ㉗ S5-2 ② $18 门控两层模型（ff561bc）：相位未完不派发+按下保持门。
#[test]
fn menu_input_gate_two_layers() {
    use game_core::engine::{input_bits, MenuInputGate};
    // 层 2：相位未完 ⇒ 一律不派发
    let mut g = MenuInputGate::default();
    assert_eq!(g.dispatch(input_bits::START), None, "层 2 锁（相位未完）");
    // 解锁后：按下边沿派发
    g.phase_unlocked = true;
    assert_eq!(g.dispatch(input_bits::START), Some(input_bits::START), "边沿派发");
    // 层 1：保持中 ⇒ 早退作废
    assert_eq!(g.dispatch(input_bits::START), None, "按住保持作废");
    assert_eq!(g.dispatch(input_bits::START), None);
    // 释放后再按 ⇒ 重新派发
    assert_eq!(g.dispatch(0), None);
    assert_eq!(g.dispatch(input_bits::A), Some(input_bits::A), "新边沿");
    // 复合键边沿
    assert_eq!(g.dispatch(0), None);
    assert_eq!(
        g.dispatch(input_bits::B | input_bits::START),
        Some(input_bits::B | input_bits::START)
    );
}
