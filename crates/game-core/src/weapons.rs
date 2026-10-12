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

/// 拾取现场（深路径三态所需；字段= $018C/$018A/$32）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PickupContext {
    /// $018C 拾取门槛/成本值（$FF=免检特判）。
    pub threshold: u8,
    /// $018A 同型连收计数（N-0048）。
    pub consecutive: u8,
    /// $32 位标志（bit7=kernel 门；bit2=侧调用守卫）。
    pub flags: u8,
    /// JSR $DBA9 奖励事件钩子（bank14 计数/奖励例程族）——只接事件不实现数值（money 链未证）。
    pub reward_pending: bool,
}

/// 拾取处理结果（含深路径三态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupOutcome {
    /// 内联 kernel 升级：INC $33，**溢出 255→0 回卷置 $0C**。
    KernelLevel { level: u8 },
    /// 深路径裸升（$92B4）：INC $33，**自然回卷 255→0（无 $0C）**。
    DeepLevel { level: u8 },
    /// 连收分支（$018C=$FF）：不加级，$018A+1（满 30 触发奖励事件钩子）。
    Consecutive { consecutive: u8, reward: bool },
    /// 异型切换：$34 := 拾取值。
    Switched { slot: u8 },
}

fn kernel_level(state: &mut WeaponState) -> PickupOutcome {
    state.level = state.level.wrapping_add(1);
    if state.level == 0 {
        state.level = 0x0C;
    }
    PickupOutcome::KernelLevel { level: state.level }
}

/// 拾取处理完整 dispatch（bank2:$922B 深路径定谳版，t39）：
/// 异型 → 切换；同型且（圈值==0 且 $018C=$FF 且 $32<0）→ 内联 kernel；
/// 同型深路径：$018C=$FF → 连收分支（$33≥8 且 $34==0 时回 kernel）；否则 → 裸升。
pub fn pickup_handler(state: &mut WeaponState, pickup_type: u8, ctx: &mut PickupContext) -> PickupOutcome {
    if pickup_type != state.slot {
        state.slot = pickup_type;
        return PickupOutcome::Switched { slot: state.slot };
    }
    let kernel_entry = pickup_type == 0 && ctx.threshold == 0xFF && ctx.flags & 0x80 != 0;
    if kernel_entry {
        return kernel_level(state);
    }
    if ctx.threshold == 0xFF {
        ctx.consecutive = ctx.consecutive.wrapping_add(1);
        let mut reward = false;
        if ctx.consecutive >= 30 {
            ctx.consecutive -= 30;
            reward = true;
            ctx.reward_pending = true; // JSR $DBA9 事件钩子
        }
        if state.level >= 8 && state.slot == 0 {
            return kernel_level(state); // $9296 JMP $923D 回 kernel
        }
        return PickupOutcome::Consecutive { consecutive: ctx.consecutive, reward };
    }
    state.level = state.level.wrapping_add(1); // $92B4 裸 INC（自然 FF→00）
    PickupOutcome::DeepLevel { level: state.level }
}

/// $9457 配置偏移表（12 项；数值语义=射击配置偏移，登记不解释）。
pub const CONFIG_OFFSETS: [u8; 12] = [0x00, 0x03, 0x0A, 0x11, 0x1A, 0x25, 0x30, 0x3D, 0x4A, 0x57, 0x64, 0x71];

/// 级→配置索引（bank2:$9368-$937A，**H-A 定谳**）。
///
/// 指令流：`LDA $33 / CMP #$04 / BCC 直取 / SBC #$04 / AND #$07 / ADC #$03 / TAY / LDX $9457,Y`。
/// **H-A 定谳**：level<4 ⇒ 索引=level（0..3）；level≥4 ⇒ 索引 = 3+((level-4)&7)+**C**，
/// 其中 ADC 进位恒为 1（AND 不清 C，SBC 无借位）⇒ 实为 **4+((level-4)&7) ∈ [4,11]**。
/// 值域恒在 [0,11]（12 项表内）——**深路径 level 可至 255 也不会越界**（高值 mod 8 周期吸收）；
/// findings §1 记载的 `3+` 公式漏了 ADC 进位（证伪线索，报 spec-writer 勘误）。
pub fn config_index_for_level(level: u8) -> u8 {
    if level < 4 {
        level
    } else {
        4 + ((level - 4) & 7)
    }
}

/// 级→$9457 配置偏移查表（越界防护：索引构造保证 ∈ [0,11]）。
pub fn config_offset_for_level(level: u8) -> u8 {
    CONFIG_OFFSETS[config_index_for_level(level) as usize]
}
