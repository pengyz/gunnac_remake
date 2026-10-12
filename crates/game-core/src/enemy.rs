//! game-core::enemy —— 敌生成层装载 + 行为脚本 VM（M3 enemy 模块，t62）。
//!
//! 语义来源：`docs/findings/enemy-script-vm.md`（t59 逐码定谳）+ `docs/spec/enemy-system.md`
//! R-enemy-1..6。**未证不发明**：命中/清除段（H-16）、HP 字段（B4）、双 SoA 桥（W12）留 TODO。
//!
//! quirk 保真（D4）：
//! - 记录装载 X 坐标 = `p0 − $10`（`SBC #$0F` 前 carry=0（LSR 链置位）⇒ 实减 16；
//!   findings 记载的『p0−$0F』系进位疏漏，L2 用例锁真值）；
//! - p1>>4==0 ⇒ 循环计数初值 `$10`（0 编码即 16）；
//! - 位 3（槽停用）记录只消费 2 字节（p0/p1），启用记录 4 字节——变长记录流；
//! - `$FC` X 调整为**界门**而非钳制：结果 ∈ [$20,$E0) ⇒ 退槽，界外 ⇒ 续行（字面指令流语义）。

/// VM 槽状态（zp $A4/$B0/$B6/$BC/$C2/$AA + 流 PC $0197/$019D，X=槽 0-7）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EnemySlot {
    pub x: u8,        // $A4 X 坐标
    pub param: u8,    // $B0 指令参/计数
    pub sub_lo: u8,   // $B6 子流指针低
    pub sub_hi: u8,   // $BC 子流指针高
    pub loop_cnt: u8, // $C2 循环计数
    pub opcode: u8,   // $AA 当前指令码
    pub pc_lo: u8,    // $197 流 PC 低
    pub pc_hi: u8,    // $19D 流 PC 高（0=槽停用）
}

impl EnemySlot {
    pub fn active(&self) -> bool {
        self.pc_hi != 0
    }
}

/// 记录装载效果（bank00:$83F6-$8432，4B 变长记录流）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadEffect {
    /// 普通记录（4 字节）：槽 slot 初始化。
    Spawn {
        slot: u8,
        x: u8,
        loop_cnt: u8,
        pc_lo: u8,
        pc_hi: u8,
    },
    /// 槽停用记录（2 字节，p1 bit3）：循环计数仍装载，槽停用。
    Disable { slot: u8, loop_cnt: u8 },
    /// 段控制（p0≥$80）：$A3 := p0&$7F。
    Segment { count: u8 },
    /// 表结束（p0=0）。
    End,
}

/// 记录装载机语义（bank00:$83F6）。
/// `p0`=0 结束 / ≥$80 段控制；否则记录：`p1&7`=槽、`p1 bit3`=停用、`p1>>4`=循环计数（0→$10），
/// 启用记录续取 `p2/p3` 流指针；**X 坐标 = p0−$10**（进位 quirk）。
pub fn load_record(p0: u8, p1: u8, p2: u8, p3: u8) -> LoadEffect {
    if p0 == 0 {
        return LoadEffect::End;
    }
    if p0 & 0x80 != 0 {
        return LoadEffect::Segment { count: p0 & 0x7F };
    }
    let slot = p1 & 7;
    let mut loop_cnt = p1 >> 4;
    if loop_cnt == 0 {
        loop_cnt = 0x10;
    }
    let carry = (p1 >> 3) & 1; // LSR×4 链末端 carry = p1 bit3
    if carry == 1 {
        return LoadEffect::Disable { slot, loop_cnt };
    }
    LoadEffect::Spawn {
        slot,
        x: p0.wrapping_sub(0x10), // SBC #$0F 且 carry=0 ⇒ p0−$10（quirk）
        loop_cnt,
        pc_lo: p2,
        pc_hi: p3,
    }
}

/// `$FC` X 相对调整（bank00:$8471-$8483）——**界门语义**（非钳制）：
/// `x = x + d`（8 位回绕，CLC/ADC）；结果 ≥$E0 或 <$20 ⇒ Continue（续行脚本）；
/// 结果 ∈ [$20,$E0) ⇒ Exit（$19D:=0 退槽）。X 本身不钳。
pub fn vm_x_adjust(x: u8, d: u8) -> (u8, bool) {
    let new_x = x.wrapping_add(d);
    let exit = !(new_x >= 0xE0 || new_x < 0x20);
    (new_x, exit)
}

/// `$FD` 循环计数装入（bank00:$848B）：`$C2,X := 参数`。
pub fn vm_loop_load(counter: u8) -> u8 {
    counter
}

/// `$FE` 循环步（bank00:$8492-$8498 + $849A 指针装载）：
/// `DEC $C2`；非 0 ⇒ 跳 2B 指针（taken=true, 新 PC=ptr）；0 ⇒ 跳过 2B 续行（taken=false, PC+2）。
pub fn vm_loop_step(counter: u8, ptr_lo: u8, ptr_hi: u8) -> (u8, bool, u8, u8) {
    let new_c = counter.wrapping_sub(1);
    if new_c != 0 {
        (new_c, true, ptr_lo, ptr_hi)
    } else {
        (new_c, false, 0, 0) // 跳过路径的 PC 由调用方 +2
    }
}

/// 指令+参数取指（bank00:$84A9-$84C4）：`AA=opcode`，`B0/B6/BC`=3 参数（B6/BC 亦可为子流指针），
/// 流 PC 前进 3（`TYA/SEC/ADC $14`，含 +1 进位 quirk）。
/// `y`=三参数取尽后 `TYA` 处的 Y（入口 Y+3），返回 (param, sub_lo, sub_hi, pc_lo, pc_hi)。
pub fn vm_fetch_params(
    pc_lo: u8,
    pc_hi: u8,
    y: u8,
    p0: u8,
    p1: u8,
    p2: u8,
) -> (u8, u8, u8, u8, u8) {
    let sum = pc_lo as u16 + y as u16 + 1; // TYA/SEC/ADC $14（C=1）
    let new_lo = sum as u8;
    let new_hi = pc_hi.wrapping_add((sum > 0xFF) as u8);
    (p0, p1, p2, new_lo, new_hi)
}

/// 输出表写界（bank00:$84E2/$8511 `CPX #$10`）：索引 ≥$10 不写（fail-open 跳过）。
pub fn output_table_write(index: u8, value: u8) -> Option<(u8, u8)> {
    if index >= 0x10 {
        None
    } else {
        Some((index, value))
    }
}
