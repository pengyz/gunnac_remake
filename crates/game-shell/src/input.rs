//! 输入策略（平台/宿主侧统一消解）——未证输入组合在进入 game-core 前 fail-closed。
//!
//! - 对向同持（L+R / U+D）在 game-core `player::velocity_from_input` 是未证组合
//!   （W6 未闭环）⇒ **panic**（fail-closed）；平台/宿主必须先消解为 0（本模块职责）。
//! - `$18` 位图语义＝A 级（R-move-1）：bit0=R / bit1=L / bit2=D / bit3=U / bit6=B / bit7=A；
//!   START/SELECT 不进 `$18` 本图（菜单域另用 `$16` 位）。
//! - `$16` 为边沿脉冲（本帧按下且上帧未按）。v1 只产出、不消费（射击/投放接线属后续步，
//!   未证不发明）。

use game_platform::Buttons;

/// 方向位（`$18` 位图，A 级 R-move-1）。
pub const DIR_RIGHT: u8 = 0x01;
pub const DIR_LEFT: u8 = 0x02;
pub const DIR_DOWN: u8 = 0x04;
pub const DIR_UP: u8 = 0x08;
/// B 键（bit6）与 A 键（bit7）——`$18` 位图同源（R-move-1）。
pub const BTN_B: u8 = 0x40;
pub const BTN_A: u8 = 0x80;

/// 对向同持消解：L+R 同持 ⇒ 两键都置 0；U+D 同持 ⇒ 两键都置 0。
///
/// 依据：W6 未闭环，core 对组合输入 panic（fail-closed）；平台侧消解＝与
/// game-desktop `held_from_keys` 同语义。其余键不受影响。
pub fn resolve_directions(buttons: Buttons) -> Buttons {
    let mut out = buttons;
    if buttons.left && buttons.right {
        out.left = false;
        out.right = false;
    }
    if buttons.up && buttons.down {
        out.up = false;
        out.down = false;
    }
    out
}

/// 消解后按钮 → `$18` 位图（方向 + A/B；START/SELECT 不入本图）。
pub fn held_bitmap(buttons: &Buttons) -> u8 {
    let mut held = 0u8;
    if buttons.right {
        held |= DIR_RIGHT;
    }
    if buttons.left {
        held |= DIR_LEFT;
    }
    if buttons.down {
        held |= DIR_DOWN;
    }
    if buttons.up {
        held |= DIR_UP;
    }
    if buttons.b {
        held |= BTN_B;
    }
    if buttons.a {
        held |= BTN_A;
    }
    held
}

/// `$16` 边沿位图（**方向 + A/B 子集**）：本帧按下且上帧未按。
///
/// **不含 START/SELECT**（菜单域 `$16` bit4=`START`/bit5=`SELECT`）——菜单/标题段
/// 使用 [`menu_pressed_bitmap`]（$18 域菜单位，含 START/SELECT）。
/// v1 本函数只产出、不消费（射击/投放等边沿行为未接线）。
pub fn pressed_bitmap(prev: &Buttons, now: &Buttons) -> u8 {
    let mut pressed = 0u8;
    if now.right && !prev.right {
        pressed |= DIR_RIGHT;
    }
    if now.left && !prev.left {
        pressed |= DIR_LEFT;
    }
    if now.down && !prev.down {
        pressed |= DIR_DOWN;
    }
    if now.up && !prev.up {
        pressed |= DIR_UP;
    }
    if now.b && !prev.b {
        pressed |= BTN_B;
    }
    if now.a && !prev.a {
        pressed |= BTN_A;
    }
    pressed
}

// ===== 菜单域 `$18` 位（A 级：flow-response-latency.md §1/§3 逐按直读 + R-move-1 位序） =====

/// 菜单域 START（`$18` bit4；A 级直读 `flow-response-latency.md` §1）。
pub const MENU_START: u8 = 0x10;
/// 菜单域 SELECT（`$18` bit5；flow `MENU_BTN_SELECT` 同源）。
pub const MENU_SELECT: u8 = 0x20;
/// 菜单域 DOWN（`$18` bit2；A 级直读）。
pub const MENU_DOWN: u8 = 0x04;
/// 菜单域 UP（`$18` bit3；方向位沿用 R-move-1）。
pub const MENU_UP: u8 = 0x08;
/// 菜单域 LEFT（`$18` bit1；方向位沿用 R-move-1）。
pub const MENU_LEFT: u8 = 0x02;
/// 菜单域 RIGHT（`$18` bit0；方向位沿用 R-move-1）。
pub const MENU_RIGHT: u8 = 0x01;
/// 菜单域 A（`$18` bit7；R-move-1 同源）。
pub const MENU_A: u8 = 0x80;
/// 菜单域 B（`$18` bit6；R-move-1 同源）。
pub const MENU_B: u8 = 0x40;

/// 菜单域 `$18` 按下沿位图（A/B/START/SELECT/方向全谱；喂 `engine::InputLatch`）。
///
/// 与 [`pressed_bitmap`] 的区别：本函数含 START/SELECT（菜单必需），供标题/菜单/商店派发；
/// 实机 gameplay 输入仍用 `$18` 方向+A/B 子集（[`held_bitmap`]）。
pub fn menu_pressed_bitmap(prev: &Buttons, now: &Buttons) -> u8 {
    let mut pressed = pressed_bitmap(prev, now);
    if now.start && !prev.start {
        pressed |= MENU_START;
    }
    if now.select && !prev.select {
        pressed |= MENU_SELECT;
    }
    pressed
}

#[cfg(test)]
mod tests {
    use super::{held_bitmap, pressed_bitmap, resolve_directions, BTN_A, BTN_B, DIR_DOWN, DIR_LEFT, DIR_RIGHT, DIR_UP};
    use game_platform::Buttons;

    fn dirs(right: bool, left: bool, down: bool, up: bool) -> Buttons {
        Buttons { right, left, down, up, ..Buttons::default() }
    }

    #[test]
    fn opposite_directions_resolve_to_zero() {
        let resolved = resolve_directions(dirs(true, true, true, true));
        assert!(!resolved.left && !resolved.right, "L+R 同持⇒两键 0");
        assert!(!resolved.up && !resolved.down, "U+D 同持⇒两键 0");
        assert_eq!(held_bitmap(&resolved), 0, "消解后方向位图=0");
    }

    #[test]
    fn single_directions_preserved() {
        assert_eq!(held_bitmap(&resolve_directions(dirs(true, false, false, false))), DIR_RIGHT);
        assert_eq!(held_bitmap(&resolve_directions(dirs(false, true, false, false))), DIR_LEFT);
        assert_eq!(held_bitmap(&resolve_directions(dirs(false, false, true, false))), DIR_DOWN);
        assert_eq!(held_bitmap(&resolve_directions(dirs(false, false, false, true))), DIR_UP);
    }

    #[test]
    fn non_direction_buttons_preserved_by_resolve() {
        let b = Buttons { a: true, b: true, start: true, select: true, right: true, left: true, ..Buttons::default() };
        let r = resolve_directions(b);
        assert!(r.a && r.b && r.start && r.select, "对向消解不影响 A/B/START/SELECT");
        assert_eq!(held_bitmap(&r), BTN_A | BTN_B, "START/SELECT 不入 $18 位图");
    }

    #[test]
    fn pressed_is_edge_only() {
        let prev = dirs(true, false, false, false);
        let now = dirs(true, false, false, true);
        assert_eq!(pressed_bitmap(&prev, &now), DIR_UP, "只报新按下（边沿）");
        assert_eq!(pressed_bitmap(&now, &now), 0, "保持⇒无边沿");
        let a = Buttons { a: true, ..Buttons::default() };
        assert_eq!(pressed_bitmap(&Buttons::default(), &a), BTN_A, "A 按下边沿");
    }
}
