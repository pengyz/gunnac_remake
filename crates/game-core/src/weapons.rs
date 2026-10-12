//! game-core::weapons —— 武器/弹药状态机骨架（M3 第二模块）。
//!
//! 覆盖：拾取处理（bank2:$922B-$9258，t25 N-0032/33）、weapon_slot($34)/weapon_level($33)
//! 状态机、弹药 $018F 消耗/回满 4（R-fire-*/R-respawn-4）、射击模式分派**查表骨架**。
//! **不覆盖**：弹丸行为（待 t29 考证，H-01 留 TODO 不发明）、受击判定（H-16 未证域）。
//!
//! 原版 quirks 保真（D4）：升级回卷只在 `INC $33` 溢出 255→0 时置 $0C（bank2:$923D-$9243），
//! $0C 并非等级上限——过 $0C 后继续递增（pk-5 用例即此边界）。

use crate::l2ports::{self, BombDrop, BombDropResult};

/// 武器/弹药状态（字段=命名台账 A 级名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WeaponState {
    pub slot: u8,      // $34 weapon_slot（N-0032）
    pub level: u8,     // $33 weapon_level（N-0033）
    pub bomb_ammo: u8, // $018F bomb_ammo（压缩 BCD，N-0017）
}

/// 拾取处理结果（bank2:$922B 两分支的可观察面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupOutcome {
    /// 同型（`CMP $34` 相等）：`INC $33` 升级；溢出 255→0 回卷置 $0C。
    Leveled { level: u8 },
    /// 异型（`BNE $9248`）：`STA $34` 切换武器槽（等级不动）。
    Switched { slot: u8 },
}

/// 拾取处理（bank2:$922B-$9258 同型/异型语义；碰撞上下文门槛 $018C/$32/Y 由调用方满足）。
///
/// 注意（t31 L2 实测发现）：ROM 内联升级 kernel（$923D `INC $33`）经 `922F TAY/BNE $925F`
/// 只在 **slot==0 同型**时直达；slot≠0 同型落入 $925F 未证深路径（H-02 两说未闭环）——
/// L2 双实现对拍覆盖 slot==0 kernel 与异型切换；slot≠0 同型的升级语义待 H-02 定谳后补。
pub fn pickup_handler(state: &mut WeaponState, pickup_type: u8) -> PickupOutcome {
    if pickup_type == state.slot {
        state.level = state.level.wrapping_add(1);
        if state.level == 0 {
            state.level = 0x0C; // 溢出回卷（bank2:$9241 LDA #$0C）
        }
        PickupOutcome::Leveled { level: state.level }
    } else {
        state.slot = pickup_type;
        PickupOutcome::Switched { slot: state.slot }
    }
}

/// 死亡/重生复位（R-respawn-4 + bank03:$A376 STA $34/$A378 STA $33 + N-0017 init=4）：
/// slot:=0、level:=0、bomb_ammo:=4。
pub fn respawn(state: &mut WeaponState) {
    state.slot = 0;
    state.level = 0;
    state.bomb_ammo = 4;
}

/// 投放消耗（R-fire-3/4）：复用 l2ports 的取小 + ED03 BCD 扣减；成功则 mode:=7 由调用方写入。
pub fn deploy_bomb(state: &mut WeaponState, pressed: u8, bomb_type_idx: u8) -> Option<BombDropResult> {
    match l2ports::bomb_drop(pressed, 0, state.bomb_ammo, bomb_type_idx) {
        BombDrop::NotFired { .. } => None,
        BombDrop::Fired { ammo_new, bomb_type, y } => {
            state.bomb_ammo = ammo_new;
            Some(BombDropResult { ammo_new, bomb_type, y })
        }
    }
}

/// 射击模式（bank03:A1F5/A23D 的 $34 分派结构；弹型行为=H-01 未证，留 TODO 不发明）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireMode {
    /// slot=0：六弹型自动连射生成器（bank03:$A1FD-$A22B，N-0035）。
    AutoSix,
    /// slot=1..2：bank03:$A271 路径。
    Slot12,
    /// slot=3：bank03:$A23A 回路特例（`CMP #$03/BEQ`）。
    Slot3,
    /// slot>=4：bank03:$A24C X=6 循环路径。
    Slot4Plus,
}

/// $34→射击模式映射查表骨架（分支结构照搬 A1F5/A23D-A256，B 级）。
/// 各模式的弹型行为函数 = **TODO(H-01)**：t29 考证后落地，此处不发明。
pub fn fire_mode_dispatch(slot: u8) -> FireMode {
    match slot {
        0 => FireMode::AutoSix,
        1..=2 => FireMode::Slot12,
        3 => FireMode::Slot3,
        _ => FireMode::Slot4Plus,
    }
}

/// 弹型行为占位（H-01）：签名预留，语义待 t29 考证；返回 None=未实现（fail-closed）。
pub fn fire_behavior(_mode: FireMode, _level: u8) -> Option<()> {
    // TODO(H-01): 弹丸行为/射速/弹型表待 t29 考证后实现（禁止发明）。
    None
}
