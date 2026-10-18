//! 敌槽坐标可见化（task-45；**只暴露已证坐标列**）。
//!
//! 证据基线：[enemy-coords-impl-spec.md](../../docs/findings/enemy-coords-impl-spec.md)
//! （task-71/75/77/79 复核 pass）。要点：
//!
//! - **`$2F` leader（v1 唯一可渲染坐标的敌型）**：tracked trace `gameplay-stage1.enemy.jsonl`
//!   全部 $2F 采样点恒 `(128,8)`（A 观测：5 槽/多帧）；handler `bank02:$899B` 含
//!   `STA $04E0,X`/`STA $04FC,X`（A 指令，归属 B）⇒ v1 暴露 **x/y 两列**（trace 对拍见
//!   `tests/enemy_coords_replay.rs`）。
//! - **`$2B`/`$4E` 成员型（fail-closed）**：handler init x/y 与速度对（`$550/$56C/$588/$5A4`）
//!   未逐值审计（spec §3 B 级归属 / §5.1-4）⇒ **不暴露坐标**（`position=None`），不画。
//!   trace 观测其运动为横摆/锯齿（非恒定速度），**禁止**用常量/预测/插值位置冒充已证坐标。
//! - **其余型**（`$12/$1A/$1C/$2A` 与低槽 `$05/$07`）：生成源未解码（W32）⇒ 不 spawn、不画。
//! - **九字段 `$865C`** 生成路径对 STG-01 普通敌型未证（W32）⇒ 不套用。
//!
//! 命中事件：`hits_this_frame`（task-37 已证事件面）已暴露；**闪烁/消失等命中视觉未证**
//! （`$D616` 死亡/回收门未证、HP0 不回收敌槽）⇒ 不发明视觉，仅在 FrameView 事件面/摘要登记。
use game_core::engine::{self, ObjectSlot};

/// v1 已证坐标的敌 leader 型（`$2F`；sched ctrl2 Spawn，A 级生成链）。
pub const ENEMY_LEADER_TYPE: u8 = 0x2F;

/// `$2F` leader 已证恒定坐标（trace 全采样点恒 `(128,8)`；A 观测 + handler 写点 A 指令）。
pub const ENEMY_LEADER_POSITION: (u8, u8) = (128, 8);

/// 已知但坐标写者未证、v1 fail-closed 不暴露坐标的敌成员型（`$2B`/`$4E`）。
pub const ENEMY_FAIL_CLOSED_COORDS: [u8; 2] = [0x2B, 0x4E];

/// 该型是否有**已证坐标列**可暴露/渲染（v1 仅 `$2F` 的 x/y 双列）。
pub const fn has_proven_coords(type_: u8) -> bool {
    type_ == ENEMY_LEADER_TYPE
}

/// 初始化新激活 `$2F` 槽的已证坐标与速度四元组（速度=0：trace 恒 (128,8) ⇒ 静止）。
///
/// 返回 `false` ⇒ 非 `$2F`（调用方 fail-closed 不设坐标/不画）。
pub fn init_proven_coords(slot: &mut ObjectSlot) -> bool {
    if !has_proven_coords(slot.type_) {
        return false;
    }
    slot.x = ENEMY_LEADER_POSITION.0;
    slot.y = ENEMY_LEADER_POSITION.1;
    // 静止：速度四元组清零（trace 恒 (128,8) ⇒ 无位移；不发明速度值）。
    slot.f550 = 0;
    slot.f56c = 0;
    slot.f588 = 0;
    slot.f5a4 = 0;
    true
}

/// 对已初始化槽执行已证 `motion_update`（速度=0 ⇒ 位置不变）；返回是否仍在上屏域。
///
/// 出屏（`MotionOutcome::Offscreen`）=A 级运动规则判定；v1 仅停止暴露坐标（不发明回收语义）。
pub fn step_proven_coords(slot: &mut ObjectSlot) -> bool {
    match engine::motion_update(slot) {
        engine::MotionOutcome::Continue { .. } => true,
        engine::MotionOutcome::Offscreen => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{has_proven_coords, init_proven_coords, step_proven_coords, ENEMY_LEADER_POSITION};
    use game_core::engine::ObjectSlot;

    #[test]
    fn only_leader_type_has_proven_coords() {
        assert!(has_proven_coords(0x2F));
        assert!(!has_proven_coords(0x2B));
        assert!(!has_proven_coords(0x4E));
        assert!(!has_proven_coords(0x12));
    }

    #[test]
    fn leader_init_sets_proven_static_position() {
        let mut slot = ObjectSlot { type_: 0x2F, ..ObjectSlot::default() };
        assert!(init_proven_coords(&mut slot));
        assert_eq!((slot.x, slot.y), ENEMY_LEADER_POSITION);
        assert!(step_proven_coords(&mut slot), "静止⇒仍上屏");
        assert_eq!((slot.x, slot.y), ENEMY_LEADER_POSITION, "速度=0⇒位置不变");
    }

    #[test]
    fn member_types_are_fail_closed() {
        for t in [0x2B, 0x4E] {
            let mut slot = ObjectSlot { type_: t, ..ObjectSlot::default() };
            assert!(!init_proven_coords(&mut slot), "成员型坐标未证⇒不初始化");
            assert_eq!((slot.x, slot.y), (0, 0), "不写坐标");
        }
    }
}
