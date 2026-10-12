//! L2 同名移植函数（game-core 纯逻辑侧）—— 语义 = ROM 子程序字节的行为等价重写。
//!
//! 每个函数的证据指针（A 级）与 L2 用例（oracle/subroutines/l2_cases.jsonl）一一对应：
//! 差分裁定 = 6502 核执行 ROM 原字节的出参 == 本文件函数出参（见 game-oracle::l2）。
//! 只搬已考证语义，未考证的行为不发明（AGENTS.md §3/§1）。

/// 玩家 X 坐标高钳（bank03:A0D0-A0E8，N-0011）：`CMP #$F4 / BCC 直存 / LDA #$F3 / LDY #0`。
///
/// 语义：候选值 ≥ $F4 ⇒ X=$F3 且亚像素速度分量清零（R-move-3 触钳归零）；否则原值保留。
/// RAM 效果：A→$04E0（player_x），Y→$0470（亚像素累加器）。
pub fn player_x_clamp_hi(cand: u8, vel: u8) -> (u8, u8) {
    if cand >= 0xF4 {
        (0xF3, 0)
    } else {
        (cand, vel)
    }
}

/// 玩家 X 坐标低钳（bank03:A0DD-A0E8，N-0011）：`CMP #$0E / BCS 直存 / LDA #$0E / LDY #0`。
pub fn player_x_clamp_lo(cand: u8, vel: u8) -> (u8, u8) {
    if cand < 0x0E {
        (0x0E, 0)
    } else {
        (cand, vel)
    }
}

/// 玩家 Y 坐标高钳（bank03:A09D-A0B7，N-0012）：`CMP #$E0 / BCC 直存 / LDA #$DF / LDY #0`。
///
/// RAM 效果：A→$04FC（player_y），Y→$048C（亚像素累加器）。
pub fn player_y_clamp_hi(cand: u8, vel: u8) -> (u8, u8) {
    if cand >= 0xE0 {
        (0xDF, 0)
    } else {
        (cand, vel)
    }
}

/// 玩家 Y 坐标低钳（bank03:A0AA-A0B7，N-0012）：`CMP #$14 / BCS 直存 / LDA #$14 / LDY #0`。
pub fn player_y_clamp_lo(cand: u8, vel: u8) -> (u8, u8) {
    if cand < 0x14 {
        (0x14, 0)
    } else {
        (cand, vel)
    }
}

/// 十进制取位一步（bank00:9DE8-9DF6，N-0023）：`LDA $018E / CMP #$0A / BCS / SBC #$0A / STA $018E`。
///
/// 语义：v ≥ 10 ⇒ 回写 v-10 并返回 (v-10, true)；v < 10 ⇒ 不写、跳出（(v, false)）。
pub fn digit_conversion(v: u8) -> (u8, bool) {
    if v >= 10 {
        (v - 10, true)
    } else {
        (v, false)
    }
}

/// 消耗量表 `$A336-$A339` = **[1, 2, 3, 2]**（C-bomb-consume，A 级：seed 字节 `01 02 03 02`，
/// docs/spec/constants.md v2；Y=0..3 对应炸弹档位）。
pub const CONSUMPTION_TABLE: [u8; 4] = [1, 2, 3, 2];

/// 炸弹类型表 `$A33A-$A33D` = **[2, 3, 4, 4]**（C-bomb-type-tbl，A 级：seed 字节 `02 03 04 04`；
/// **登记不解释**——数值语义属 H-11 待证，禁止望文生义）。
pub const BOMB_TYPE_TABLE: [u8; 4] = [2, 3, 4, 4];

/// 消耗量表查表判定（bank03:A2C4-A2CF，R-fire-3 回退档）：
/// `LDY $3B / LDA $018F / CMP $A336,Y / BCS 保留 / LDY #1`。
///
/// 语义：弹药（字节比较）≥ 表[idx] ⇒ 返回 idx；否则回退档 1。
pub fn consumption_lookup(bomb_ammo: u8, idx: u8) -> u8 {
    assert!(idx < 4, "消耗量表索引越界（已证域 0..3）：{idx}");
    if bomb_ammo >= CONSUMPTION_TABLE[idx as usize] {
        idx
    } else {
        1
    }
}

/// ED03 扣减语义（bank15:ED03-ED26，N-0017）：`$018F` 为**压缩 BCD**（高半字节十位），
/// `SBC` 逐半字节借位扣减：低半字节减 deduct，借位则 +10 并高半字节 -1，高半字节下溢 ⇒ 返回 0。
///
/// 返回 (新弹药 BCD, ED03 返回时的 Y)：无借位 Y=deduct；有借位 Y=低半字节结果（ED16 `TAY`）。
pub fn bcd_sub_ammo(ammo: u8, deduct: u8) -> (u8, u8) {
    let low = (ammo & 0x0F) as i16 - deduct as i16;
    if low >= 0 {
        ((ammo & 0xF0) | low as u8, deduct)
    } else {
        let low2 = (low + 10) as u8;
        let high = (ammo >> 4) as i16 - 1;
        if high < 0 {
            (0, low2)
        } else {
            (((high as u8) << 4) | low2, low2)
        }
    }
}

/// 投放例程（bank03:A2B4-A2FA）差分出参。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BombDrop {
    /// R-fire-2 前置链任一不满足即 RTS：`a` = 退出时 A 值，其余不动、零写入。
    NotFired { a: u8 },
    /// 触发投放：新弹药（BCD）、类型表值（→$0765）、ED03 返回 Y。
    Fired { ammo_new: u8, bomb_type: u8, y: u8 },
}

/// 投放例程语义（R-fire-2/3/4，bank03:A2B4-A2FA + JSR $ED03）：
///
/// 1. R-fire-2 前置链（顺序固定）：`$16`bit7（A 键边沿）→ `$0401==0` → `$018F≠0`，任一不满足 RTS；
/// 2. R-fire-3 查表：idx=`$3B`，弹药不足消耗量 ⇒ 回退档 sel=1；类型 `TYPE_TABLE[sel]` → `$0765`；
/// 3. R-fire-4 取小扣减：`deduct = min(CONSUMPTION_TABLE[sel], 弹药字节)`，经 ED03 BCD 扣减回写 `$018F`；
/// 4. 坐标快照 `$04FC→$04FD`、`$04E0→$04E1`，`mode:=7`（`$0401`）。
pub fn bomb_drop(pressed: u8, mode: u8, bomb_ammo: u8, idx: u8) -> BombDrop {
    if pressed & 0x80 == 0 {
        return BombDrop::NotFired { a: pressed & 0x80 };
    }
    if mode != 0 {
        return BombDrop::NotFired { a: mode };
    }
    if bomb_ammo == 0 {
        return BombDrop::NotFired { a: 0 };
    }
    let sel = consumption_lookup(bomb_ammo, idx);
    let bomb_type = BOMB_TYPE_TABLE[sel as usize];
    let cost = CONSUMPTION_TABLE[sel as usize];
    let deduct = if cost < bomb_ammo { cost } else { bomb_ammo };
    let (ammo_new, y) = bcd_sub_ammo(bomb_ammo, deduct);
    BombDrop::Fired { ammo_new, bomb_type, y }
}
