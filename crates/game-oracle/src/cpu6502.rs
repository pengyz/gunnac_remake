//! 6502 CPU 核（L2 子程序级差分执行器的参考执行侧）。
//!
//! 职责：执行 ROM 原字节子程序片段，产出寄存器 + 内存写集。
//! 范围：仅 CPU 语义（无 PPU/APU/时序）；PPU/APU 寄存器访问由 Bus 侧 stub。
//! 指令集：官方 151 条；非法操作码一律报错（fail-closed，不猜语义）。
//! ADC/SBC 按 2A03（NES CPU）行为只做二进制运算——D 标志存而不用。

/// Bus 抽象：CPU 核不关心内存映射（L2 场景见 `l2::L2Bus`）。
pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
    /// 调用约定压栈（JSR/PHA/中断）：默认等同 `write`；
    /// L2 总线覆写为"不记入差分写集"（栈操作是约定副产物）。
    /// 注意：游戏变量也可能落在栈页（如 $018E/$018F），故剔除只能走这条专用通道，
    /// 不能按地址区间一刀切。
    fn write_stack(&mut self, addr: u16, val: u8) {
        self.write(addr, val);
    }
}

pub const FLAG_C: u8 = 0x01;
pub const FLAG_Z: u8 = 0x02;
pub const FLAG_I: u8 = 0x04;
pub const FLAG_D: u8 = 0x08;
pub const FLAG_B: u8 = 0x10;
pub const FLAG_U: u8 = 0x20;
pub const FLAG_V: u8 = 0x40;
pub const FLAG_N: u8 = 0x80;

/// 单步执行结果：RTS/RTI/BRK 是 L2 的收尾信号（BRK 视为异常）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepResult {
    Continue,
    Jsr,
    Rts,
    Rti,
    Brk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub p: u8,
    pub pc: u16,
}

impl Cpu {
    pub const fn new(a: u8, x: u8, y: u8, sp: u8, p: u8, pc: u16) -> Self {
        Self { a, x, y, sp, p, pc }
    }

    fn set_zn(&mut self, v: u8) {
        self.p = if v == 0 { self.p | FLAG_Z } else { self.p & !FLAG_Z };
        self.p = if v & 0x80 != 0 { self.p | FLAG_N } else { self.p & !FLAG_N };
    }

    fn flag(&self, f: u8) -> bool {
        self.p & f != 0
    }

    fn set_flag(&mut self, f: u8, on: bool) {
        if on {
            self.p |= f;
        } else {
            self.p &= !f;
        }
    }

    fn fetch(&mut self, bus: &mut impl Bus) -> u8 {
        let v = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }

    fn fetch16(&mut self, bus: &mut impl Bus) -> u16 {
        let lo = self.fetch(bus) as u16;
        let hi = self.fetch(bus) as u16;
        (hi << 8) | lo
    }

    fn push(&mut self, bus: &mut impl Bus, v: u8) {
        bus.write_stack(0x0100 | self.sp as u16, v);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self, bus: &mut impl Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read(0x0100 | self.sp as u16)
    }

    /// (zp,X) 间接变址
    fn addr_izx(&mut self, bus: &mut impl Bus) -> u16 {
        let zp = self.fetch(bus).wrapping_add(self.x);
        let lo = bus.read(zp as u16) as u16;
        let hi = bus.read(zp.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }

    /// (zp),Y 间接变址
    fn addr_izy(&mut self, bus: &mut impl Bus) -> u16 {
        let zp = self.fetch(bus);
        let lo = bus.read(zp as u16) as u16;
        let hi = bus.read(zp.wrapping_add(1) as u16) as u16;
        ((hi << 8) | lo).wrapping_add(self.y as u16)
    }

    fn addr_zp(&mut self, bus: &mut impl Bus) -> u16 {
        self.fetch(bus) as u16
    }

    fn addr_zpx(&mut self, bus: &mut impl Bus) -> u16 {
        self.fetch(bus).wrapping_add(self.x) as u16
    }

    fn addr_zpy(&mut self, bus: &mut impl Bus) -> u16 {
        self.fetch(bus).wrapping_add(self.y) as u16
    }

    fn addr_abs(&mut self, bus: &mut impl Bus) -> u16 {
        self.fetch16(bus)
    }

    fn addr_abx(&mut self, bus: &mut impl Bus) -> u16 {
        self.fetch16(bus).wrapping_add(self.x as u16)
    }

    fn addr_aby(&mut self, bus: &mut impl Bus) -> u16 {
        self.fetch16(bus).wrapping_add(self.y as u16)
    }

    fn branch(&mut self, bus: &mut impl Bus, take: bool) {
        let off = self.fetch(bus) as i8;
        if take {
            self.pc = self.pc.wrapping_add(off as u16);
        }
    }

    fn adc(&mut self, v: u8) {
        let c = if self.flag(FLAG_C) { 1u16 } else { 0 };
        let sum = self.a as u16 + v as u16 + c;
        let r = sum as u8;
        self.set_flag(FLAG_C, sum > 0xFF);
        // 2A03 无 BCD：溢出按二进制补码
        self.set_flag(FLAG_V, (!(self.a ^ v) & (self.a ^ r) & 0x80) != 0);
        self.a = r;
        self.set_zn(r);
    }

    fn cmp(&mut self, reg: u8, v: u8) {
        let r = reg.wrapping_sub(v);
        self.set_flag(FLAG_C, reg >= v);
        self.set_zn(r);
    }

    fn asl_val(&mut self, v: u8) -> u8 {
        self.set_flag(FLAG_C, v & 0x80 != 0);
        let r = v << 1;
        self.set_zn(r);
        r
    }

    fn lsr_val(&mut self, v: u8) -> u8 {
        self.set_flag(FLAG_C, v & 0x01 != 0);
        let r = v >> 1;
        self.set_zn(r);
        r
    }

    fn rol_val(&mut self, v: u8) -> u8 {
        let carry_in = if self.flag(FLAG_C) { 1 } else { 0 };
        self.set_flag(FLAG_C, v & 0x80 != 0);
        let r = (v << 1) | carry_in;
        self.set_zn(r);
        r
    }

    fn ror_val(&mut self, v: u8) -> u8 {
        let carry_in = if self.flag(FLAG_C) { 0x80 } else { 0 };
        self.set_flag(FLAG_C, v & 0x01 != 0);
        let r = (v >> 1) | carry_in;
        self.set_zn(r);
        r
    }

    /// 执行一条指令。非法操作码/BRK 以信号或 panic-free 错误返回（由调用方处置）。
    pub fn step(&mut self, bus: &mut impl Bus) -> Result<StepResult, String> {
        let op = self.fetch(bus);
        use StepResult::*;
        match op {
            // ---- 立即/零页/绝对 存储与加载族
            0xA9 => { let v = self.fetch(bus); self.a = v; self.set_zn(v); }
            0xA5 => { let a = self.addr_zp(bus); let v = bus.read(a); self.a = v; self.set_zn(v); }
            0xB5 => { let a = self.addr_zpx(bus); let v = bus.read(a); self.a = v; self.set_zn(v); }
            0xAD => { let a = self.addr_abs(bus); let v = bus.read(a); self.a = v; self.set_zn(v); }
            0xBD => { let a = self.addr_abx(bus); let v = bus.read(a); self.a = v; self.set_zn(v); }
            0xB9 => { let a = self.addr_aby(bus); let v = bus.read(a); self.a = v; self.set_zn(v); }
            0xA1 => { let a = self.addr_izx(bus); let v = bus.read(a); self.a = v; self.set_zn(v); }
            0xB1 => { let a = self.addr_izy(bus); let v = bus.read(a); self.a = v; self.set_zn(v); }
            0xA2 => { let v = self.fetch(bus); self.x = v; self.set_zn(v); }
            0xA6 => { let a = self.addr_zp(bus); let v = bus.read(a); self.x = v; self.set_zn(v); }
            0xB6 => { let a = self.addr_zpy(bus); let v = bus.read(a); self.x = v; self.set_zn(v); }
            0xAE => { let a = self.addr_abs(bus); let v = bus.read(a); self.x = v; self.set_zn(v); }
            0xBE => { let a = self.addr_aby(bus); let v = bus.read(a); self.x = v; self.set_zn(v); }
            0xA0 => { let v = self.fetch(bus); self.y = v; self.set_zn(v); }
            0xA4 => { let a = self.addr_zp(bus); let v = bus.read(a); self.y = v; self.set_zn(v); }
            0xB4 => { let a = self.addr_zpx(bus); let v = bus.read(a); self.y = v; self.set_zn(v); }
            0xAC => { let a = self.addr_abs(bus); let v = bus.read(a); self.y = v; self.set_zn(v); }
            0xBC => { let a = self.addr_abx(bus); let v = bus.read(a); self.y = v; self.set_zn(v); }
            0x85 => { let a = self.addr_zp(bus); bus.write(a, self.a); }
            0x95 => { let a = self.addr_zpx(bus); bus.write(a, self.a); }
            0x8D => { let a = self.addr_abs(bus); bus.write(a, self.a); }
            0x9D => { let a = self.addr_abx(bus); bus.write(a, self.a); }
            0x99 => { let a = self.addr_aby(bus); bus.write(a, self.a); }
            0x81 => { let a = self.addr_izx(bus); bus.write(a, self.a); }
            0x91 => { let a = self.addr_izy(bus); bus.write(a, self.a); }
            0x86 => { let a = self.addr_zp(bus); bus.write(a, self.x); }
            0x96 => { let a = self.addr_zpy(bus); bus.write(a, self.x); }
            0x8E => { let a = self.addr_abs(bus); bus.write(a, self.x); }
            0x84 => { let a = self.addr_zp(bus); bus.write(a, self.y); }
            0x94 => { let a = self.addr_zpx(bus); bus.write(a, self.y); }
            0x8C => { let a = self.addr_abs(bus); bus.write(a, self.y); }

            // ---- 逻辑
            0x29 => { let v = self.fetch(bus); self.a &= v; self.set_zn(self.a); }
            0x25 => { let a = self.addr_zp(bus); self.a &= bus.read(a); self.set_zn(self.a); }
            0x35 => { let a = self.addr_zpx(bus); self.a &= bus.read(a); self.set_zn(self.a); }
            0x2D => { let a = self.addr_abs(bus); self.a &= bus.read(a); self.set_zn(self.a); }
            0x3D => { let a = self.addr_abx(bus); self.a &= bus.read(a); self.set_zn(self.a); }
            0x39 => { let a = self.addr_aby(bus); self.a &= bus.read(a); self.set_zn(self.a); }
            0x21 => { let a = self.addr_izx(bus); self.a &= bus.read(a); self.set_zn(self.a); }
            0x31 => { let a = self.addr_izy(bus); self.a &= bus.read(a); self.set_zn(self.a); }
            0x09 => { let v = self.fetch(bus); self.a |= v; self.set_zn(self.a); }
            0x05 => { let a = self.addr_zp(bus); self.a |= bus.read(a); self.set_zn(self.a); }
            0x15 => { let a = self.addr_zpx(bus); self.a |= bus.read(a); self.set_zn(self.a); }
            0x0D => { let a = self.addr_abs(bus); self.a |= bus.read(a); self.set_zn(self.a); }
            0x1D => { let a = self.addr_abx(bus); self.a |= bus.read(a); self.set_zn(self.a); }
            0x19 => { let a = self.addr_aby(bus); self.a |= bus.read(a); self.set_zn(self.a); }
            0x01 => { let a = self.addr_izx(bus); self.a |= bus.read(a); self.set_zn(self.a); }
            0x11 => { let a = self.addr_izy(bus); self.a |= bus.read(a); self.set_zn(self.a); }
            0x49 => { let v = self.fetch(bus); self.a ^= v; self.set_zn(self.a); }
            0x45 => { let a = self.addr_zp(bus); self.a ^= bus.read(a); self.set_zn(self.a); }
            0x55 => { let a = self.addr_zpx(bus); self.a ^= bus.read(a); self.set_zn(self.a); }
            0x4D => { let a = self.addr_abs(bus); self.a ^= bus.read(a); self.set_zn(self.a); }
            0x5D => { let a = self.addr_abx(bus); self.a ^= bus.read(a); self.set_zn(self.a); }
            0x59 => { let a = self.addr_aby(bus); self.a ^= bus.read(a); self.set_zn(self.a); }
            0x41 => { let a = self.addr_izx(bus); self.a ^= bus.read(a); self.set_zn(self.a); }
            0x51 => { let a = self.addr_izy(bus); self.a ^= bus.read(a); self.set_zn(self.a); }
            0x24 => { let a = self.addr_zp(bus); let v = bus.read(a); self.set_flag(FLAG_Z, self.a & v == 0); self.set_flag(FLAG_V, v & 0x40 != 0); self.set_flag(FLAG_N, v & 0x80 != 0); }
            0x2C => { let a = self.addr_abs(bus); let v = bus.read(a); self.set_flag(FLAG_Z, self.a & v == 0); self.set_flag(FLAG_V, v & 0x40 != 0); self.set_flag(FLAG_N, v & 0x80 != 0); }

            // ---- 算术
            0x69 => { let v = self.fetch(bus); self.adc(v); }
            0x65 => { let a = self.addr_zp(bus); let v = bus.read(a); self.adc(v); }
            0x75 => { let a = self.addr_zpx(bus); let v = bus.read(a); self.adc(v); }
            0x6D => { let a = self.addr_abs(bus); let v = bus.read(a); self.adc(v); }
            0x7D => { let a = self.addr_abx(bus); let v = bus.read(a); self.adc(v); }
            0x79 => { let a = self.addr_aby(bus); let v = bus.read(a); self.adc(v); }
            0x61 => { let a = self.addr_izx(bus); let v = bus.read(a); self.adc(v); }
            0x71 => { let a = self.addr_izy(bus); let v = bus.read(a); self.adc(v); }
            0xE9 => { let v = self.fetch(bus) ^ 0xFF; self.adc(v); }
            0xE5 => { let a = self.addr_zp(bus); let v = bus.read(a) ^ 0xFF; self.adc(v); }
            0xF5 => { let a = self.addr_zpx(bus); let v = bus.read(a) ^ 0xFF; self.adc(v); }
            0xED => { let a = self.addr_abs(bus); let v = bus.read(a) ^ 0xFF; self.adc(v); }
            0xFD => { let a = self.addr_abx(bus); let v = bus.read(a) ^ 0xFF; self.adc(v); }
            0xF9 => { let a = self.addr_aby(bus); let v = bus.read(a) ^ 0xFF; self.adc(v); }
            0xE1 => { let a = self.addr_izx(bus); let v = bus.read(a) ^ 0xFF; self.adc(v); }
            0xF1 => { let a = self.addr_izy(bus); let v = bus.read(a) ^ 0xFF; self.adc(v); }

            // ---- 比较
            0xC9 => { let v = self.fetch(bus); self.cmp(self.a, v); }
            0xC5 => { let a = self.addr_zp(bus); let v = bus.read(a); self.cmp(self.a, v); }
            0xD5 => { let a = self.addr_zpx(bus); let v = bus.read(a); self.cmp(self.a, v); }
            0xCD => { let a = self.addr_abs(bus); let v = bus.read(a); self.cmp(self.a, v); }
            0xDD => { let a = self.addr_abx(bus); let v = bus.read(a); self.cmp(self.a, v); }
            0xD9 => { let a = self.addr_aby(bus); let v = bus.read(a); self.cmp(self.a, v); }
            0xC1 => { let a = self.addr_izx(bus); let v = bus.read(a); self.cmp(self.a, v); }
            0xD1 => { let a = self.addr_izy(bus); let v = bus.read(a); self.cmp(self.a, v); }
            0xE0 => { let v = self.fetch(bus); self.cmp(self.x, v); }
            0xE4 => { let a = self.addr_zp(bus); let v = bus.read(a); self.cmp(self.x, v); }
            0xEC => { let a = self.addr_abs(bus); let v = bus.read(a); self.cmp(self.x, v); }
            0xC0 => { let v = self.fetch(bus); self.cmp(self.y, v); }
            0xC4 => { let a = self.addr_zp(bus); let v = bus.read(a); self.cmp(self.y, v); }
            0xCC => { let a = self.addr_abs(bus); let v = bus.read(a); self.cmp(self.y, v); }

            // ---- 移位
            0x0A => { self.a = self.asl_val(self.a); }
            0x06 => { let a = self.addr_zp(bus); let v = self.asl_val(bus.read(a)); bus.write(a, v); }
            0x16 => { let a = self.addr_zpx(bus); let v = self.asl_val(bus.read(a)); bus.write(a, v); }
            0x0E => { let a = self.addr_abs(bus); let v = self.asl_val(bus.read(a)); bus.write(a, v); }
            0x1E => { let a = self.addr_abx(bus); let v = self.asl_val(bus.read(a)); bus.write(a, v); }
            0x4A => { self.a = self.lsr_val(self.a); }
            0x46 => { let a = self.addr_zp(bus); let v = self.lsr_val(bus.read(a)); bus.write(a, v); }
            0x56 => { let a = self.addr_zpx(bus); let v = self.lsr_val(bus.read(a)); bus.write(a, v); }
            0x4E => { let a = self.addr_abs(bus); let v = self.lsr_val(bus.read(a)); bus.write(a, v); }
            0x5E => { let a = self.addr_abx(bus); let v = self.lsr_val(bus.read(a)); bus.write(a, v); }
            0x2A => { self.a = self.rol_val(self.a); }
            0x26 => { let a = self.addr_zp(bus); let v = self.rol_val(bus.read(a)); bus.write(a, v); }
            0x36 => { let a = self.addr_zpx(bus); let v = self.rol_val(bus.read(a)); bus.write(a, v); }
            0x2E => { let a = self.addr_abs(bus); let v = self.rol_val(bus.read(a)); bus.write(a, v); }
            0x3E => { let a = self.addr_abx(bus); let v = self.rol_val(bus.read(a)); bus.write(a, v); }
            0x6A => { self.a = self.ror_val(self.a); }
            0x66 => { let a = self.addr_zp(bus); let v = self.ror_val(bus.read(a)); bus.write(a, v); }
            0x76 => { let a = self.addr_zpx(bus); let v = self.ror_val(bus.read(a)); bus.write(a, v); }
            0x6E => { let a = self.addr_abs(bus); let v = self.ror_val(bus.read(a)); bus.write(a, v); }
            0x7E => { let a = self.addr_abx(bus); let v = self.ror_val(bus.read(a)); bus.write(a, v); }

            // ---- 递增/递减
            0xE6 => { let a = self.addr_zp(bus); let v = bus.read(a).wrapping_add(1); bus.write(a, v); self.set_zn(v); }
            0xF6 => { let a = self.addr_zpx(bus); let v = bus.read(a).wrapping_add(1); bus.write(a, v); self.set_zn(v); }
            0xEE => { let a = self.addr_abs(bus); let v = bus.read(a).wrapping_add(1); bus.write(a, v); self.set_zn(v); }
            0xFE => { let a = self.addr_abx(bus); let v = bus.read(a).wrapping_add(1); bus.write(a, v); self.set_zn(v); }
            0xC6 => { let a = self.addr_zp(bus); let v = bus.read(a).wrapping_sub(1); bus.write(a, v); self.set_zn(v); }
            0xD6 => { let a = self.addr_zpx(bus); let v = bus.read(a).wrapping_sub(1); bus.write(a, v); self.set_zn(v); }
            0xCE => { let a = self.addr_abs(bus); let v = bus.read(a).wrapping_sub(1); bus.write(a, v); self.set_zn(v); }
            0xDE => { let a = self.addr_abx(bus); let v = bus.read(a).wrapping_sub(1); bus.write(a, v); self.set_zn(v); }
            0xE8 => { self.x = self.x.wrapping_add(1); self.set_zn(self.x); }
            0xC8 => { self.y = self.y.wrapping_add(1); self.set_zn(self.y); }
            0xCA => { self.x = self.x.wrapping_sub(1); self.set_zn(self.x); }
            0x88 => { self.y = self.y.wrapping_sub(1); self.set_zn(self.y); }

            // ---- 转移/栈/标志
            0x4C => { self.pc = self.fetch16(bus); }
            0x6C => {
                let ptr = self.fetch16(bus);
                // 6502 JMP 间接页跨越 bug（原样保真）
                let lo = bus.read(ptr) as u16;
                let hi = bus.read((ptr & 0xFF00) | ((ptr as u8).wrapping_add(1) as u16)) as u16;
                self.pc = (hi << 8) | lo;
            }
            0x20 => {
                let lo = self.fetch(bus) as u16;
                self.push(bus, (self.pc >> 8) as u8);
                self.push(bus, self.pc as u8);
                let hi = self.fetch(bus) as u16;
                self.pc = (hi << 8) | lo;
                return Ok(Jsr);
            }
            0x60 => {
                let lo = self.pop(bus) as u16;
                let hi = self.pop(bus) as u16;
                self.pc = ((hi << 8) | lo).wrapping_add(1);
                return Ok(Rts);
            }
            0x40 => {
                self.p = (self.pop(bus) & !FLAG_B) | FLAG_U;
                let lo = self.pop(bus) as u16;
                let hi = self.pop(bus) as u16;
                self.pc = (hi << 8) | lo;
                return Ok(Rti);
            }
            0x00 => return Ok(Brk),
            0x48 => { self.push(bus, self.a); }
            0x08 => { self.push(bus, self.p | FLAG_B | FLAG_U); }
            0x68 => { self.a = self.pop(bus); self.set_zn(self.a); }
            0x28 => { self.p = (self.pop(bus) & !FLAG_B) | FLAG_U; }
            0x18 => self.set_flag(FLAG_C, false),
            0x38 => self.set_flag(FLAG_C, true),
            0x58 => self.set_flag(FLAG_I, false),
            0x78 => self.set_flag(FLAG_I, true),
            0xB8 => self.set_flag(FLAG_V, false),
            0xD8 => self.set_flag(FLAG_D, false),
            0xF8 => self.set_flag(FLAG_D, true),
            0xEA => {}
            0xAA => { self.x = self.a; self.set_zn(self.x); }
            0xA8 => { self.y = self.a; self.set_zn(self.y); }
            0x8A => { self.a = self.x; self.set_zn(self.a); }
            0x98 => { self.a = self.y; self.set_zn(self.a); }
            0xBA => { self.x = self.sp; self.set_zn(self.x); }
            0x9A => { self.sp = self.x; }

            // ---- 分支
            0x10 => { let t = !self.flag(FLAG_N); self.branch(bus, t); }
            0x30 => { let t = self.flag(FLAG_N); self.branch(bus, t); }
            0x50 => { let t = !self.flag(FLAG_V); self.branch(bus, t); }
            0x70 => { let t = self.flag(FLAG_V); self.branch(bus, t); }
            0x90 => { let t = !self.flag(FLAG_C); self.branch(bus, t); }
            0xB0 => { let t = self.flag(FLAG_C); self.branch(bus, t); }
            0xD0 => { let t = !self.flag(FLAG_Z); self.branch(bus, t); }
            0xF0 => { let t = self.flag(FLAG_Z); self.branch(bus, t); }

            other => return Err(format!("非法/未实现操作码 ${:02X} @ PC=${:04X}", other, self.pc.wrapping_sub(1))),
        }
        Ok(Continue)
    }
}
