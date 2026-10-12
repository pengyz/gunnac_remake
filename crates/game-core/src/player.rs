//! game-core::player —— 自机移动/射击（S5 首模块）。
//!
//! 语义来源：`docs/spec/player-movement.md` R-move-1..6 / R-fire-1..6（A 级，逐条搬运）+
//! `docs/spec/constants.md` v2（速度对/钳制/消耗表）。**未证语义不发明**：
//! H-11（类型值语义）、H-14（主武器射击例程本体）、H-15（mode 值域）、W5/W6 留空。
//!
//! 移动 tick = ROM 片段 `bank03:A08D-A0EB` 的语义（L2 双实现对拍见 game-oracle::l2
//! "movement_tick" 用例；L3 帧级对拍见 oracle/traces/gameplay-stage1.player.jsonl）。
//! **对齐约定**：`state(N) = tick(state(N-1), held(N))` —— oracle 940 帧实证 0 失配
//! （输入当帧生效；帧计数 = 场景 DSL 的 inputPolled 序）。
//!
//! 按键角色（t25 §9 收口 H-14，ram-watch-s3.md §9.2）：**主武器 = 计数器驱动自动连射**
//! （bank03:$A1FD-$A22B 六弹型循环，源坐标=$04E0/$04FC，全程无按键测试）——不在输入面；
//! **A 边沿（$16 bit7）= 投放**（bank03:$A2B4）；**B 边沿（$16 bit6，bank03:$A193/A1A9
//! `AND #$40`）= 弹种切换**——切换规则本身 C 级未证（N-0036/E-R06 呼应），此处不发明。
//!
//! 原版 quirks 保真（D4）：Y 轴正向分支**没有** X 轴的溢出钳制检查（`bank03:A09A` 后无
//! `BCS`，而 X 的 `bank03:A0CE` 有）——溢出时 Y 回绕不钳，行为照搬不"修好"。

use crate::l2ports::{self, BombDrop};

/// 速度对（R-move-6 / C-vel-h、C-vel-h-neg）：(亚像素增量, 有符号整数增量)，
/// 1/256 px 定点：(208,2)=+2.8125、(48,253)=−2.8125、无输入=(0,0)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VelocityPair {
    pub subpix: u8,
    pub int: u8,
}

impl VelocityPair {
    pub const ZERO: Self = Self { subpix: 0, int: 0 };
    pub const POS: Self = Self { subpix: 208, int: 2 }; // C-vel-h
    pub const NEG: Self = Self { subpix: 48, int: 253 }; // C-vel-h-neg
}

/// 自机状态（L3 状态向量主体；字段=命名台账 A 级名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerState {
    pub x: u8,           // $04E0 player_x
    pub y: u8,           // $04FC player_y
    pub x_subpix: u8,    // $0470 player_x_subpix
    pub y_subpix: u8,    // $048C player_y_subpix
    pub bomb_ammo: u8,   // $018F（压缩 BCD）
    pub action_mode: u8, // $0401 game_action_mode
}

/// 输入位图 → 速度对（R-move-1/R-move-6）。bit0=R, bit1=L, bit2=D, bit3=U。
///
/// 反向同持（L+R / U+D）属未证输入组合（W6 未闭环）——fail-closed，禁止发明语义。
pub fn velocity_from_input(held: u8) -> (VelocityPair, VelocityPair) {
    let vx = match (held & 0x01 != 0, held & 0x02 != 0) {
        (true, false) => VelocityPair::POS,
        (false, true) => VelocityPair::NEG,
        (false, false) => VelocityPair::ZERO,
        (true, true) => panic!("未证输入组合 L+R 同持（W6 未闭环）——禁止发明语义"),
    };
    let vy = match (held & 0x04 != 0, held & 0x08 != 0) {
        (true, false) => VelocityPair::POS,
        (false, true) => VelocityPair::NEG,
        (false, false) => VelocityPair::ZERO,
        (true, true) => panic!("未证输入组合 U+D 同持（W6 未闭环）——禁止发明语义"),
    };
    (vx, vy)
}

/// 单轴积分 + 钳制（R-move-5 + R-move-3/4）。
///
/// 语义 = `subpix'=(subpix+vs)&0xFF`（进位 C）、`a=vi+pixel+C`（8 位回绕）：
/// - 正向（vi<0x80，含 0）：X 轴有溢出钳制（`B0 04`）+ `CMP #$F4`；**Y 轴无溢出检查（quirk）** + `CMP #$E0`；
/// - 反向（vi≥0x80）：借位钳下界 + `CMP 下限`；
/// - 触钳 ⇒ 返回 (钳值, 0)（亚像素归零，R-move-3/4）。
fn integrate_axis(pixel: u8, subpix: u8, v: VelocityPair, cfg: AxisClamp) -> (u8, u8) {
    let sum = subpix as u16 + v.subpix as u16;
    let subpix_new = sum as u8;
    let carry = (sum > 0xFF) as u16;
    let total = v.int as u16 + pixel as u16 + carry;
    let a = total as u8;
    let cout = total > 0xFF;
    if v.int & 0x80 == 0 {
        if (cfg.overflow_clamps && cout) || a >= cfg.hi_cmp {
            (cfg.hi_set, 0)
        } else {
            (a, subpix_new)
        }
    } else if !cout || a < cfg.lo {
        (cfg.lo, 0)
    } else {
        (a, subpix_new)
    }
}

struct AxisClamp {
    hi_cmp: u8,
    hi_set: u8,
    lo: u8,
    overflow_clamps: bool,
}

const X_CLAMP: AxisClamp = AxisClamp { hi_cmp: 0xF4, hi_set: 0xF3, lo: 0x0E, overflow_clamps: true };
const Y_CLAMP: AxisClamp = AxisClamp { hi_cmp: 0xE0, hi_set: 0xDF, lo: 0x14, overflow_clamps: false };

/// 移动 tick（ROM 片段 `bank03:A08D-A0EB` 语义）：Y 轴恒积分 → X 门（`held&3==0` 跳过 X，
/// 钱包式保留 X 亚像素——`bank03:A0B8-A0BC`）→ X 轴积分。
///
/// `vel_x`/`vel_y` 为速度对 RAM（$0550/$056C、$0588/$05A4）的取值；L3 用 `tick()` 由
/// held 自动导出（R-move-6），L2 用例直接给定（片段级对拍）。
pub fn movement_tick(st: &PlayerState, held: u8, vel_x: VelocityPair, vel_y: VelocityPair) -> PlayerState {
    let (y, y_subpix) = integrate_axis(st.y, st.y_subpix, vel_y, Y_CLAMP);
    let (x, x_subpix) = if held & 0x03 != 0 {
        integrate_axis(st.x, st.x_subpix, vel_x, X_CLAMP)
    } else {
        (st.x, st.x_subpix)
    };
    PlayerState { x, y, x_subpix, y_subpix, ..*st }
}

/// 帧 tick（L3 语义）：速度对由当帧 held 导出后走 `movement_tick`。
pub fn tick(st: &PlayerState, held: u8) -> PlayerState {
    let (vx, vy) = velocity_from_input(held);
    movement_tick(st, held, vx, vy)
}

/// 输入位图位（bank00:9D84/9D8D、bank03:A193/A2B4 代码用法，B 级）。
pub const EDGE_B: u8 = 0x40;
pub const EDGE_A: u8 = 0x80;

/// B 边沿判定（弹种切换入口；**切换规则未证**——N-0036 C 级，动作本体留 S4 后续）。
pub fn bomb_type_switch_edge(pressed: u8) -> bool {
    pressed & EDGE_B != 0
}

/// 射击/投放（R-fire-1..6）：A 边沿（$16 bit7）驱动投放例程 `bank03:A2B4-A2FA`。
///
/// 主武器射击例程本体**未定位**（H-14）——本函数只搬已证的投放链；`pressed` 为
/// `controller_pressed` 位图（边沿脉冲）。
pub fn fire(st: &mut PlayerState, pressed: u8, bomb_type_idx: u8) -> Option<l2ports::BombDropResult> {
    match l2ports::bomb_drop(pressed, st.action_mode, st.bomb_ammo, bomb_type_idx) {
        BombDrop::NotFired { .. } => None,
        BombDrop::Fired { ammo_new, bomb_type, y } => {
            st.bomb_ammo = ammo_new;
            st.action_mode = 7; // R-fire-6：mode := 7
            Some(l2ports::BombDropResult { ammo_new, bomb_type, y })
        }
    }
}
