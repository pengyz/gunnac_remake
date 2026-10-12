//! game-core::weapons 单元测试（深路径三态 + H-A 索引变换）。
//! 语义来源：docs/findings/upgrade-branch-h02.md（t35 定谳）+ bank2:$9368 指令流。

use game_core::weapons::*;

fn st(slot: u8, level: u8) -> WeaponState {
    WeaponState { slot, level, bomb_ammo: 4 }
}

fn ctx_ff() -> PickupContext {
    PickupContext { threshold: 0xFF, consecutive: 0, flags: 0x04, reward_pending: false }
}

#[test]
fn kernel_path_wraps_ff_to_0c() {
    let mut s = st(0, 0xFF);
    let mut c = PickupContext { threshold: 0xFF, consecutive: 0, flags: 0x80, reward_pending: false };
    assert_eq!(pickup_handler(&mut s, 0, &mut c), PickupOutcome::KernelLevel { level: 0x0C });
    assert_eq!(s.level, 0x0C);
}

#[test]
fn deep_bare_inc_wraps_ff_to_00() {
    // t35 撤回范围：深路径裸升溢出自然 0（无 $0C）——与 kernel 分歧
    let mut s = st(3, 0xFF);
    let mut c = PickupContext { threshold: 55, consecutive: 0, flags: 0x04, reward_pending: false };
    assert_eq!(pickup_handler(&mut s, 3, &mut c), PickupOutcome::DeepLevel { level: 0x00 });
    assert_eq!(s.level, 0x00);
}

#[test]
fn deep_ff_threshold_counter_no_level() {
    let mut s = st(3, 5);
    let mut c = ctx_ff();
    assert_eq!(pickup_handler(&mut s, 3, &mut c), PickupOutcome::Consecutive { consecutive: 1, reward: false });
    assert_eq!(s.level, 5, "连收分支不加级");
    assert_eq!(c.consecutive, 1);
}

#[test]
fn counter_wraps_30_and_triggers_reward_event() {
    let mut s = st(3, 5);
    let mut c = PickupContext { threshold: 0xFF, consecutive: 29, flags: 0x04, reward_pending: false };
    assert_eq!(pickup_handler(&mut s, 3, &mut c), PickupOutcome::Consecutive { consecutive: 0, reward: true });
    assert!(c.reward_pending, "满 30 触发 JSR $DBA9 事件钩子（只接事件不实现数值）");
}

#[test]
fn counter_path_reenters_kernel_when_level8_slot0() {
    let mut s = st(0, 8);
    let mut c = ctx_ff();
    assert_eq!(pickup_handler(&mut s, 0, &mut c), PickupOutcome::KernelLevel { level: 9 });
    assert_eq!(c.consecutive, 1, "回 kernel 前连收计数照常 +1");
}

#[test]
fn config_index_never_out_of_bounds() {
    // H-A 定谳：level≥4 ⇒ 4+((level-4)&7) ∈ [4,11]（ADC 进位恒 1）；level<4 ⇒ level
    for level in 0..=255u8 {
        let idx = config_index_for_level(level) as usize;
        assert!(idx < CONFIG_OFFSETS.len(), "越界索引 level={level} idx={idx}");
    }
    assert_eq!(config_index_for_level(2), 2);
    assert_eq!(config_index_for_level(3), 3);
    assert_eq!(config_index_for_level(4), 4);
    assert_eq!(config_index_for_level(11), 11);
    assert_eq!(config_index_for_level(12), 4, "深路径高位 mod 8 周期吸收（进位后 4+0）");
    assert_eq!(config_index_for_level(255), 7);
    assert_eq!(config_offset_for_level(12), CONFIG_OFFSETS[4]);
}
