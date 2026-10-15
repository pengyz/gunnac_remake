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
