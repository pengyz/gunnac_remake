//! game-core::terrain —— 地形脚本机（VM 解释器 + 槽状态机 + 记录装载）。
//!
//! **架构归位注记（t71，t65 转办）**：本模块原为 `game-core::enemy` 的 VM 内核（t62 移植），
//! t65 定性为**地形脚本机**（`R-enemy-2` 行为脚本流的消费端；规则编号引用保持可追溯）。
//! 行为零变化迁移：语义/数值/golden 与 t62 逐字节一致（重构红线）。
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

/// 地形 VM 槽状态（zp $A4/$B0/$B6/$BC/$C2/$AA + 流 PC $0197/$019D，X=槽 0-7）。
/// （t71 迁移：原 `EnemySlot`——t65 定性地形脚本机后归位改名。）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TerrainSlot {
    pub x: u8,        // $A4 X 坐标
    pub param: u8,    // $B0 指令参/计数
    pub sub_lo: u8,   // $B6 子流指针低
    pub sub_hi: u8,   // $BC 子流指针高
    pub loop_cnt: u8, // $C2 循环计数
    pub opcode: u8,   // $AA 当前指令码
    pub pc_lo: u8,    // $197 流 PC 低
    pub pc_hi: u8,    // $19D 流 PC 高（0=槽停用）
}

impl TerrainSlot {
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

// ============ W12 桥输出面（t90 兑现 t71 挂接点；t64 定谳） ============

/// 5 字节 BG 记录（4 字段 + 标志字节；t64 字段映射 A 级）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BgRecord {
    pub f1: u8,   // → $130,X
    pub f2: u8,   // → $131,X
    pub f3: u8,   // → $140,X
    pub f4: u8,   // → $141,X
    pub flag: u8, // 低 3 位 → $07 标志
}

/// `$D3,X` 位属性掩码（$E9D2/$E9D6/$E9DA 三引用位，ROM 逐字节）。
/// 合成选择规则（Y 变址）**未全证**——掩码导出，选择注记不发明。
pub const ATTR_MASKS: [u8; 3] = [0xFF, 0xF9, 0xF0];

/// 图案索引→记录寻址（**v×5 quirk 保真**：ASL/ROL 两轮=×4 + ADC=+v，t70 更正 N-0088）。
pub fn bg_record_at(pattern: &[u8], index: u8) -> BgRecord {
    let off = index as usize * 5;
    let b = pattern.get(off..off + 5).expect("图案表越界（5B 记录）");
    BgRecord { f1: b[0], f2: b[1], f3: b[2], f4: b[3], flag: b[4] }
}

/// BG 缓冲族（$0130/$0131/$0140/$0141 + $07 标志 + $D3 位属性；16 槽输出表 $0150）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BgBuffers {
    pub b130: [u8; 16],
    pub b131: [u8; 16],
    pub b140: [u8; 16],
    pub b141: [u8; 16],
    pub attr: [u8; 16],
    pub flag: u8,
}

/// W12 桥**单记录搬运**（t64 字段映射；t71 挂接点兑现）：
/// 记录 4 字段 → $130/$131/$140/$141 槽位 + 标志&7 → $07 + 位属性合成（掩码口径）。
pub fn bridge_output_commit(record: BgRecord, slot: u8, bufs: &mut BgBuffers) -> Option<()> {
    let i = slot as usize;
    if i >= 16 {
        return None; // $0150 表界 CPX #$10（fail-closed）
    }
    bufs.b130[i] = record.f1;
    bufs.b131[i] = record.f2;
    bufs.b140[i] = record.f3;
    bufs.b141[i] = record.f4;
    bufs.flag = record.flag & 7;
    // 位属性合成：(旧 & 掩码) | 标志位——掩码选择规则未全证（Y 变址），取掩码[0] 兜底+注记
    bufs.attr[i] = (bufs.attr[i] & ATTR_MASKS[0]) | (record.flag & 7);
    Some(())
}

/// 桥帧（t64 同步时机：VM 槽循环后，**8 输出/帧**，输出偏移 $0E 步进 −2）。
/// `output[i]` = $150,i 图案索引；`i ∈ {0,2,...,14}` 偶偏移 8 槽。
pub fn bridge_commit_frame(output: &[u8; 16], pattern: &[u8], bufs: &mut BgBuffers) {
    let mut off = 0x0Ei32;
    while off >= 0 {
        let slot = off as u8;
        let rec = bg_record_at(pattern, output[slot as usize]);
        bridge_output_commit(rec, slot, bufs);
        off -= 2;
    }
}

/// terrain VM 单帧执行环（t71 归位模块跑真脚本流；t64 同步：6 槽/帧）：
/// 每活动槽执行**一条**指令（已证子集：$00 结束 / $FC X 界门 / $FD/$FE 循环 / $FF 跳转 /
/// $80-$FB 参数指令）；未证操作码 fail-closed 返回 Err（不发明）。
/// `mem` = 流内存读（($14),Y 语义由调用方拼装：PC 为流指针）。
pub fn vm_run_frame(
    slots: &mut [TerrainSlot],
    mem: &dyn Fn(u16) -> u8,
) -> Result<(), String> {
    for slot in slots.iter_mut() {
        if !slot.active() {
            continue;
        }
        let pc = u16::from_le_bytes([slot.pc_lo, slot.pc_hi]);
        let op = mem(pc);
        match op {
            0x00 => slot.pc_hi = 0, // 流结束退槽（$8462）
            0xFC => {
                // X 界门（$8471-$8483）：参数 + 界门语义
                let d = mem(pc.wrapping_add(1));
                let (nx, exit) = crate::stage::vm_x_adjust_gate(slot.x, d);
                slot.x = nx;
                slot.pc_lo = pc.wrapping_add(2) as u8;
                slot.pc_hi = (pc.wrapping_add(2) >> 8) as u8;
                if exit {
                    slot.pc_hi = 0; // 界内退槽（$8481 BEQ $8462）
                }
            }
            0xFD => {
                slot.loop_cnt = mem(pc.wrapping_add(1));
                let np = pc.wrapping_add(2);
                slot.pc_lo = np as u8;
                slot.pc_hi = (np >> 8) as u8;
            }
            0xFE => {
                let lo = mem(pc.wrapping_add(1));
                let hi = mem(pc.wrapping_add(2));
                let (c, taken, nlo, nhi) = vm_loop_step(slot.loop_cnt, lo, hi);
                slot.loop_cnt = c;
                if taken {
                    slot.pc_lo = nlo;
                    slot.pc_hi = nhi;
                } else {
                    let np = pc.wrapping_add(3);
                    slot.pc_lo = np as u8;
                    slot.pc_hi = (np >> 8) as u8;
                }
            }
            0xFF => {
                let lo = mem(pc.wrapping_add(1));
                let hi = mem(pc.wrapping_add(2));
                slot.pc_lo = lo;
                slot.pc_hi = hi;
            }
            0x02..=0x7F => {
                // 嵌套调用（t59 表）：B0=码自身，B6/BC=子流指针(2B)——3 字节记录
                let p0 = mem(pc.wrapping_add(1));
                let p1 = mem(pc.wrapping_add(2));
                slot.opcode = op;
                slot.param = op;
                slot.sub_lo = p0;
                slot.sub_hi = p1;
                let np = pc.wrapping_add(3);
                slot.pc_lo = np as u8;
                slot.pc_hi = (np >> 8) as u8;
            }
            0x80..=0xFB => {
                // 指令+3 参数（$84A9-$84C4）：AA/B0/B6/BC + PC 前进（4 字节记录）
                let p0 = mem(pc.wrapping_add(1));
                let p1 = mem(pc.wrapping_add(2));
                let p2 = mem(pc.wrapping_add(3));
                let y_after = 3u8;
                let (b0, b6, bc, lo, hi) = vm_fetch_params(
                    pc as u8, (pc >> 8) as u8, y_after, p0, p1, p2,
                );
                slot.opcode = op;
                slot.param = b0;
                slot.sub_lo = b6;
                slot.sub_hi = bc;
                slot.pc_lo = lo;
                slot.pc_hi = hi;
            }
            _ => return Err(format!("未证操作码 ${:02X}（fail-closed，不发明）", op)),
        }
    }
    Ok(())
}

// ============ nametable 持久化模型（t93 ②；bank15:$F040-F0DA NMI 消费链定谳） ============

/// NES 背景 nametable 状态（**4 个 NT** ×（32×30 tiles + 8×8 属性）——t99 实测
/// 上传地址含 $2Bxx（NT2），模型不得钉死 NT0）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nametable {
    pub tiles: [[u8; 960]; 4],
    pub attrs: [[u8; 64]; 4],
}

impl Default for Nametable {
    fn default() -> Self {
        Nametable { tiles: [[0; 960]; 4], attrs: [[0; 64]; 4] }
    }
}

impl Nametable {
    /// NMI 行冲刷（bank15:$F040-F0DA 定谳）：
    /// - `$160` 行挂起计数=0 ⇒ 无更新；
    /// - 行缓冲 `$130-$14F`（32 tile）→ PPU 行（地址 `$8F:$8E`，行距 $20）；
    /// - `$D3-$DA` 8 字节 → 属性行（地址 `($8F|3):$90`）。
    /// 字段映射（t93 ③ 定谳）：记录 4 字段 → 行内列 (x, x+1, x+16, x+17)（桥写集反推）。
    pub fn nmi_flush(&mut self, row_buf: &[u8; 32], attrs: &[u8; 8], addr_lo: u8, addr_hi: u8, attr_lo: u8, pending: u8) {
        if pending == 0 {
            return;
        }
        // R-l4-linebuf-6（t98 A 级）：目标地址 = $8F:($8E+$0F)；4 组×8B，组间步进 +$20
        let base = ((addr_hi as u16) << 8 | addr_lo as u16).wrapping_add(0x0F);
        for grp in 0..4u16 {
            let start = base.wrapping_add(grp * 0x20);
            for j in 0..8u16 {
                let raw = start.wrapping_add(j);
                if !(0x2000..0x3000).contains(&raw) {
                    continue;
                }
                let rel = (raw - 0x2000) as usize;
                let (nt, off) = (rel / 0x400, rel % 0x400);
                if nt < 4 && off < 0x3C0 {
                    self.tiles[nt][off] = row_buf[(grp * 8 + j) as usize];
                }
            }
        }
        let attr_raw = (((addr_hi as u16 | 3) << 8) | attr_lo as u16).wrapping_add(0x0F);
        if (0x23C0..0x3000).contains(&attr_raw) {
            let arel = (attr_raw - 0x2000) as usize;
            let ant = arel / 0x400;
            let aoff = arel % 0x400;
            if ant < 4 && (0x3C0..0x400).contains(&aoff) {
                let idx = aoff - 0x3C0;
                if idx + 8 <= 64 {
                    self.attrs[ant][idx..idx + 8].copy_from_slice(attrs);
                }
            }
        }
    }

    /// 属性字节 → 2×2 块调色板号（NES 规范：2 bit/块，行对/列对取位）。
    pub fn block_palette(&self, nt: usize, tile_x: usize, tile_y: usize) -> u8 {
        let ax = tile_x / 4;
        let ay = tile_y / 4;
        let b = self.attrs[nt][ay * 8 + ax];
        let shift = ((tile_y % 4) / 2) * 4 + ((tile_x % 4) / 2) * 2;
        (b >> shift) & 3
    }
}
