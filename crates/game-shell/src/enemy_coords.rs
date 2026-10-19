//! 敌槽坐标可见化（task-45/164；**只暴露已证坐标列**，C3 扩容）。
//!
//! 证据基线：[enemy-coords-c3-audit.md](../../docs/findings/enemy-coords-c3-audit.md)
//! （task-157/161/162/163 复核 pass）+ [enemy-coords-impl-spec.md](../../docs/findings/enemy-coords-impl-spec.md)。
//!
//! | 型 | 坐标证据 | v1(C3) 处置 |
//! |---|---|---|
//! | `$2F` leader | x=#$80、y=#$08 字面量（`$89D5-$89DC` A）+ trace 恒 (128,8) | 暴露 x/y（静止） |
//! | `$2B` member | init y=8/`$588=$90,$5A4=0`、x=param（`$C599`）或 param=0 时 `$18/$E8` 按 `$2D` 奇偶（A）；目标 `$5F8`（`$90DA-$90E9` A）；tick `$CD63` ±#$20 + `$CD08/$CCD3` 积分（A）⇒ trace 按 spawn 分段 24/24 段 0 mismatch（A） | **暴露 x/y 全接线**（含 spawn/reuse 显式复位） |
//! | `$4E` member | 初始位置：`param&$7F`→`$C8A7[16]`→`$C8B7` 脚本首 `[x,y]` + dir bit7 翻转（A/B） | **仅暴露初始位置**；运动 fail-closed（`$C64D` 脚本命令集/时序未证 C，`TODO(c3-vm)`） |
//!
//! **复用/重发（F1）**：spawn/reuse 时显式复位 y=8、vy pair、x/y 子像素（`$0470/$048C`）、相位
//! （`$05DC/$05C0`）、目标 `$5F8`（`$2B`）——按 audit §7-1（B 级：ROM init 未显式清这些字段，
//! 但 trace 24/24 段以"首帧=init 后 1 tick（y-subpix 0x90）"起仿 0 mismatch ⇒ 显式复位是保真做法）。
//!
//! **禁止**：用 type_code 反推坐标；预测/插值位置冒充已证坐标；为 `$4E` 发明速度曲线。
use game_core::engine::ObjectSlot;

/// `$2F` leader（x/y 字面量 A 级）。
pub const ENEMY_LEADER_TYPE: u8 = 0x2F;
/// `$2F` 已证恒定坐标（trace 全采样 `(128,8)`；字面量 `$80/$08`）。
pub const ENEMY_LEADER_POSITION: (u8, u8) = (128, 8);

/// `$2B` member（init 坐标 + vy pair + tick 目标趋近/积分 A 级）。
pub const ENEMY_MEMBER_2B: u8 = 0x2B;
/// `$4E` member（仅初始位置 A/B；运动 fail-closed C）。
pub const ENEMY_MEMBER_4E: u8 = 0x4E;

/// v1(C3) 可暴露坐标的敌型（V1_TYPES；`$4E` 仅初始位置列）。
pub const ENEMY_RENDER_TYPES: [u8; 3] = [ENEMY_LEADER_TYPE, ENEMY_MEMBER_2B, ENEMY_MEMBER_4E];

/// `$2B` init：vy_lo=`$90`（`$90D0` A）⇒ 0.5625 px/f；vy_hi=0（`$90BA` A）。
pub const ENEMY_2B_VY_LO: u8 = 0x90;
/// `$2B` tick：`$CD63` 每帧 vx 调整量 `#$20`（`$9120` A）。
pub const ENEMY_2B_VX_STEP: u8 = 0x20;
/// `$2B` 相位增量 `$5C0 += #$30`（`$CD9D` A）。
pub const ENEMY_2B_PHASE_STEP: u8 = 0x30;
/// `$2B` 相位计数回卷=4（`$CD9D` A 级调用 `LDY #$04`）。
pub const ENEMY_2B_PHASE_WRAP: u8 = 4;

/// `$4E` 初始位置表（`$C8A7[16]` 偏移 → `$C8B7` 脚本首 `[x,y]` 两字节；bank14 ROM 静态 A 级）。
///
/// 表内容（idx0..15）= `(x,y)`：
/// `(40,08) (20,08) (20,08) (04,18) (04,40) (50,08) (18,08) (04,18)`
/// `(60,08) (10,08) (20,08) (70,08) (40,04) (08,08) (20,08) (30,08)`。
pub const ENEMY_4E_SPAWN_TABLE: [(u8, u8); 16] = [
    (0x40, 0x08),
    (0x20, 0x08),
    (0x20, 0x08),
    (0x04, 0x18),
    (0x04, 0x40),
    (0x50, 0x08),
    (0x18, 0x08),
    (0x04, 0x18),
    (0x60, 0x08),
    (0x10, 0x08),
    (0x20, 0x08),
    (0x70, 0x08),
    (0x40, 0x04),
    (0x08, 0x08),
    (0x20, 0x08),
    (0x30, 0x08),
];

/// 该型是否有**已证坐标列**可暴露/渲染（V1_TYPES）。
pub const fn has_proven_coords(type_: u8) -> bool {
    matches!(type_, ENEMY_LEADER_TYPE | ENEMY_MEMBER_2B | ENEMY_MEMBER_4E)
}

/// `$2F`/`$2B` 运动已接线；`$4E` 仅初始位置（运动 fail-closed）。
pub const fn has_proven_motion(type_: u8) -> bool {
    matches!(type_, ENEMY_LEADER_TYPE | ENEMY_MEMBER_2B)
}

/// `$2B` 目标 `$5F8` 计算（`bank02:$90DA-$90E9` A 级逐指令）：
/// 偶 x ⇒ `$80`；奇 x ⇒ `A=x&0xFE`，`A<$80 ⇒ A+$30`，否则 `A-$2F`。
pub const fn enemy_2b_target(x: u8) -> u8 {
    if x & 1 == 0 {
        0x80
    } else {
        let a = x & 0xFE;
        if a < 0x80 { a.wrapping_add(0x30) } else { a.wrapping_sub(0x2F) }
    }
}

/// spawn/reuse 时显式复位（F1；B 级保真做法，见 audit §7-1）并写 init 值。
///
/// 返回 `Some(target)`=坐标已初始化（`$2B` 返回目标 `$5F8`；其余型返回值未用）；
/// `None` ⇒ fail-closed（`$4E` param 域越界等，调用方保持 `position=None` 不画）。
///
/// `param_zero_toggle`：`$2B` param=0 路径的 `$2D` 奇偶模型（每次 param=0 init 翻转一次；
/// 首值取 `$E8` 与 trace 复用段观测 x=231≈232 相容——B 级模型，见 audit §7-4）。
pub fn reset_for_spawn(
    slot: &mut ObjectSlot,
    type_: u8,
    param: u8,
    param_zero_toggle: &mut u8,
) -> Option<u8> {
    // 显式复位公共状态：x/y 速度四元组、y 子像素、相位、目标（目标由返回值交给宿主保存）。
    // `$0470`（x 子像素/成员 param 双义面）**不在公共复位内**：leader 的 `$0470`=成员 drip param
    // （`drip_step` 消费，必须保留）；成员型在各自分支读 param 后显式清零（`$C599` 语义）。
    slot.f550 = 0;
    slot.f56c = 0;
    slot.f588 = 0;
    slot.f5a4 = 0;
    slot.f48c = 0;
    slot.f5dc = 0;
    slot.f5c0 = 0;
    match type_ {
        ENEMY_LEADER_TYPE => {
            slot.x = ENEMY_LEADER_POSITION.0;
            slot.y = ENEMY_LEADER_POSITION.1;
            slot.f4c4 = 0;
            Some(0)
        }
        ENEMY_MEMBER_2B => {
            // `$04C4=#$93`（`$90B5` A）：bit0=y 积分、bit1=x 积分。
            slot.f4c4 = 0x93;
            slot.y = 8; // `$90CB` A
            slot.f588 = ENEMY_2B_VY_LO; // `$90D0` A
            slot.f5a4 = 0; // `$90BA` A
            let x = if param != 0 {
                param // `$C599`: x := $0470 param（param≠0）
            } else {
                // `$90EE-$90F7`: JSR $C4F7（INC $2D; LSR A）⇒ C=LSB；C=0 ⇒ #$18，C=1 ⇒ #$E8。
                let t = *param_zero_toggle;
                *param_zero_toggle = t.wrapping_add(1);
                if t & 1 == 0 { 0xE8 } else { 0x18 }
            };
            slot.x = x;
            slot.f470 = 0; // `$C599` 成功后清 $0470（显式复位）
            let target = enemy_2b_target(x);
            Some(target)
        }
        ENEMY_MEMBER_4E => {
            // `$C872`: param=$0470；dir=$0470&$80；idx=param&$7F；越界 ⇒ fail-closed（C 级 param 域）。
            let idx = param & 0x7F;
            if idx > 15 {
                return None;
            }
            let (xb, yb) = ENEMY_4E_SPAWN_TABLE[idx as usize];
            let dir = param & 0x80 != 0;
            // `$C608` x 写点：dir bit7 翻转 x（audit §4.2；x=0 & dir ⇒ $FF 特例）。
            let x = if dir {
                if xb == 0 { 0xFF } else { 0u8.wrapping_sub(xb) }
            } else {
                xb
            };
            slot.x = x;
            slot.y = yb;
            slot.f4c4 = 0x93; // entry `$AE44`: `LDA #$93; STA $04C4,X`（A）
            // 运动 fail-closed：不写速度四元组、不积分（`$C64D` 脚本命令未全解 C ⇒ TODO(c3-vm)）。
            Some(0)
        }
        _ => None,
    }
}

/// 有符号定点积分（`$CD08/$CCD3` A 级公式；**高字节按二补码符号分支**——`engine::motion_update`
/// 是无符号实现，对负速度会误判溢出，故本模块按 audit §3.3 的"A<0 对称"语义自行积分）。
///
/// 返回是否仍在上屏域；出屏（x 越出 0..=255 或 y≥`$F0`）⇒ false（不发明回收语义）。
fn integrate_signed(slot: &mut ObjectSlot) -> bool {
    // ---- x（`$CD08`）：`$0470 += $0550`（取进位）；`x += $056C + carry`（溢出/借位⇒Offscreen）----
    if slot.f4c4 & 0x02 != 0 {
        let total = i32::from(slot.x) * 256 + i32::from(slot.f470);
        let v = i32::from(slot.f56c as i8) * 256 + i32::from(slot.f550);
        let next = total + v;
        if !(0..=0xFFFF).contains(&next) {
            return false;
        }
        slot.x = (next >> 8) as u8;
        slot.f470 = (next & 0xFF) as u8;
    }
    // ---- y（`$CCD3`）：`$048C += $0588`（取进位）；`y += $05A4 + carry`；y≥$F0⇒Offscreen ----
    if slot.f4c4 & 0x01 != 0 {
        let total = i32::from(slot.y) * 256 + i32::from(slot.f48c);
        let v = i32::from(slot.f5a4 as i8) * 256 + i32::from(slot.f588);
        let next = total + v;
        if !(0..=0xFFFF).contains(&next) {
            return false;
        }
        slot.y = (next >> 8) as u8;
        slot.f48c = (next & 0xFF) as u8;
        if slot.y >= 0xF0 {
            return false;
        }
    }
    true
}

/// 每逻辑帧坐标推进（已初始化槽）。
///
/// - `$2F`：静止（零速）。
/// - `$2B`（`$9120` A 级）：`$CD63` vx 向目标 `$5F8` ±#$20 调整 → 相位 `$CD9D` → 有符号定点
///   积分（`$CD08/$CCD3` 语义）；溢出/出屏 ⇒ 返回 false（停止暴露坐标，不发明回收语义）。
/// - `$4E`：运动 fail-closed（保持初始位置静态可见；TODO(c3-vm)）。
pub fn step_proven_coords(slot: &mut ObjectSlot, type_: u8, target: u8) -> bool {
    match type_ {
        ENEMY_LEADER_TYPE => true,
        ENEMY_MEMBER_2B => {
            // $CD63：x<target ⇒ vx += #$20（低字节进位到高）；x>target ⇒ vx -= #$20（借位）。
            if slot.x < target {
                let (lo, carry) = slot.f550.overflowing_add(ENEMY_2B_VX_STEP);
                slot.f550 = lo;
                slot.f56c = slot.f56c.wrapping_add(u8::from(carry));
            } else if slot.x > target {
                let (lo, borrow) = slot.f550.overflowing_sub(ENEMY_2B_VX_STEP);
                slot.f550 = lo;
                slot.f56c = slot.f56c.wrapping_sub(u8::from(borrow));
            }
            // $CD9D：相位 $5C0 += #$30，进位 ⇒ INC $5DC；到 4 回卷（bit7 置位时跳过）。
            if slot.f5dc & 0x80 == 0 {
                let (v, carry) = slot.f5c0.overflowing_add(ENEMY_2B_PHASE_STEP);
                slot.f5c0 = v;
                if carry {
                    slot.f5dc = slot.f5dc.wrapping_add(1);
                    if slot.f5dc >= ENEMY_2B_PHASE_WRAP {
                        slot.f5dc = 0;
                    }
                }
            }
            integrate_signed(slot)
        }
        ENEMY_MEMBER_4E => true, // 运动 fail-closed（初始位置静态可见）
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proven_types_and_motion_flags() {
        assert!(has_proven_coords(0x2F) && has_proven_coords(0x2B) && has_proven_coords(0x4E));
        assert!(!has_proven_coords(0x12));
        assert!(has_proven_motion(0x2F) && has_proven_motion(0x2B));
        assert!(!has_proven_motion(0x4E), "$4E 运动 fail-closed");
    }

    #[test]
    fn leader_init_sets_proven_static_position() {
        let mut toggle = 0u8;
        let mut slot = ObjectSlot { type_: 0x2F, ..ObjectSlot::default() };
        assert_eq!(reset_for_spawn(&mut slot, 0x2F, 0, &mut toggle), Some(0));
        assert_eq!((slot.x, slot.y), ENEMY_LEADER_POSITION);
        assert!(step_proven_coords(&mut slot, 0x2F, 0));
        assert_eq!((slot.x, slot.y), ENEMY_LEADER_POSITION, "静止");
    }

    #[test]
    fn leader_reset_preserves_drip_param() {
        // `$0470` 双义：对 leader 是成员 drip param（`drip_step` 消费）——复位**不得清零**。
        let mut toggle = 0u8;
        let mut slot =
            ObjectSlot { type_: 0x2F, f470: 0xC0, ..ObjectSlot::default() };
        assert_eq!(reset_for_spawn(&mut slot, 0x2F, 0xC0, &mut toggle), Some(0));
        assert_eq!(slot.f470, 0xC0, "leader 的 $0470=成员 param 必须保留");
        assert_eq!((slot.x, slot.y), ENEMY_LEADER_POSITION);
    }

    #[test]
    fn member_2b_init_and_reuse_reset() {
        let mut toggle = 0u8;
        let mut slot = ObjectSlot { type_: 0x2B, f470: 0xC0, ..ObjectSlot::default() };
        // 先把旧占用状态污染成非零（模拟槽复用残留）。
        slot.x = 0x77;
        slot.y = 0x66;
        slot.f48c = 0x55;
        slot.f5dc = 0x02;
        slot.f5c0 = 0x10;
        let target = reset_for_spawn(&mut slot, 0x2B, 0xC0, &mut toggle).expect("param 192");
        assert_eq!((slot.x, slot.y), (0xC0, 8), "x=param、y=8");
        assert_eq!(slot.f588, ENEMY_2B_VY_LO);
        assert_eq!(slot.f5a4, 0);
        assert_eq!((slot.f470, slot.f48c), (0, 0), "子像素显式复位");
        assert_eq!((slot.f5dc, slot.f5c0), (0, 0), "相位显式复位");
        assert_eq!(target, enemy_2b_target(0xC0));
        assert_eq!(enemy_2b_target(0xC0), 128, "偶 x ⇒ $80");
        // 首 tick：y-subpix 进 0x90（首帧=init 后 1 tick 口径）。
        assert!(step_proven_coords(&mut slot, 0x2B, target));
        assert_eq!(slot.y, 8);
        assert_eq!(slot.f48c, 0x90, "首帧 y 子像素=0x90");
        assert_eq!((slot.f5dc, slot.f5c0), (0, 0x30), "相位 +0x30");
    }

    #[test]
    fn member_2b_param_zero_parity_and_target() {
        // param=0：$18/$E8 按 $2D 奇偶（B 级模型：首值 $E8）。
        let mut toggle = 0u8;
        let mut a = ObjectSlot { type_: 0x2B, ..ObjectSlot::default() };
        reset_for_spawn(&mut a, 0x2B, 0, &mut toggle);
        let mut b = ObjectSlot { type_: 0x2B, ..ObjectSlot::default() };
        reset_for_spawn(&mut b, 0x2B, 0, &mut toggle);
        assert_eq!((a.x, b.x), (0xE8, 0x18), "param=0 交替 $E8/$18");
        assert_eq!(enemy_2b_target(0xE8), 0x80);
        assert_eq!(enemy_2b_target(0xE7), 0xE6u8.wrapping_sub(0x2F), "奇 x ≥$80 ⇒ A-$2F");
        assert_eq!(enemy_2b_target(0x21), 0x20 + 0x30, "奇 x <$80 ⇒ A+$30");
    }

    #[test]
    fn member_4e_initial_position_table_and_dir() {
        let mut toggle = 0u8;
        // idx9 → (0x10,08)；idx6 → (0x18,08)。
        let mut slot = ObjectSlot { type_: 0x4E, ..ObjectSlot::default() };
        assert_eq!(reset_for_spawn(&mut slot, 0x4E, 9, &mut toggle), Some(0));
        assert_eq!((slot.x, slot.y), (0x10, 0x08));
        assert!(!has_proven_motion(0x4E));
        assert!(step_proven_coords(&mut slot, 0x4E, 0), "运动 fail-closed=静态可见");
        assert_eq!((slot.x, slot.y), (0x10, 0x08), "$4E 静态（不发明运动）");
        assert_eq!(reset_for_spawn(&mut slot, 0x4E, 0x86, &mut toggle), Some(0));
        assert_eq!((slot.x, slot.y), (0xE8, 0x08), "idx6 0x18 + dir ⇒ 0xE8(232)");
        // 越界 param ⇒ fail-closed。
        assert_eq!(reset_for_spawn(&mut slot, 0x4E, 0x10, &mut toggle), None);
    }
}
