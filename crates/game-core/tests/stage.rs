//! game-core::stage 单测（t73）：卷轴推进/脉冲相位/边界三段/双通道分立 + 负向自证。

use game_core::stage::*;

#[test]
fn base_scroll_dy_minus2_and_wrap_quirk() {
    assert_eq!(scroll_delta(1300), (0, -2), "基础卷轴 −2/帧");
    // mod 256 回绕 quirk（f1317 y=0 → f1318 y=238 实测）
    let st = StageScroll { x: 0, y: 0 };
    let st = scroll_tick(1318, &st);
    assert_eq!(st.y, 238, "0−18 mod 256 = 238（回绕）");
}

#[test]
fn pulse_phase_exact_1318_plus_120k() {
    for f in [1318u32, 1438, 1558, 2398] {
        assert!(is_pulse_frame(f), "脉冲帧 f{f} 必中");
        assert_eq!(scroll_delta(f), (0, -18));
    }
}

#[test]
fn boundary_three_segments() {
    assert_eq!(scroll_delta(1193), (0, -48), "边界装载 −48");
    assert_eq!(scroll_delta(1194), (0, 0), "静止段");
    assert_eq!(scroll_delta(1213), (0, 0), "静止段末帧");
    assert_eq!(scroll_delta(1214), (0, -2), "基础卷起始");
}

#[test]
fn ui_scroll_double_write_quirk_and_channel_separation() {
    // $AB3B 同值双写 quirk
    assert_eq!(UiScroll { value: 85 }.ppu_scroll_writes(), (85, 85));
    // 双通道分立（R-bg-scroll-4）：StageScroll 与 UiScroll 类型隔离——stage tick 不触 UI
    let st = StageScroll { x: 1, y: 2 };
    let ui = UiScroll { value: 9 };
    let st2 = scroll_tick(1300, &st);
    assert_eq!(ui.value, 9, "关卡卷轴推进不得影响 UI 通道（分立断言）");
    assert_eq!(st2, StageScroll { x: 1, y: 0 });
}

#[test]
fn scroll_negative_selfproof() {
    // 负向自证：相位边界必红——非脉冲帧不得判脉冲/−18
    for f in [1317u32, 1319, 1300, 1298, 1437] {
        assert!(!is_pulse_frame(f), "非脉冲帧 f{f} 误判（必红）");
        assert_ne!(scroll_delta(f).1, -18, "非脉冲帧不得 −18");
    }
    // 边界三段负向：静止段不得 −2
    assert_ne!(scroll_delta(1200).1, -2, "静止段误推必红");
    // 边界装载非 −2
    assert_ne!(scroll_delta(1193).1, -2);
}
