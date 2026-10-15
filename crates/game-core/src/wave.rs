//! game-core::wave —— 敌波调度器记录机解码（bank00:86AE 流，B1/B2 内容线）。
//!
//! 语义来源：`docs/findings/enemy-timeline.md` §1.3（记录机 bank00:86CE 记录格式
//! `[delay@0][项…][ctrl]`，A 级逐 handler 清点）+ §2.2（STG-01 调度流 bank8:$A26D
//! 逐记录解码，A 级）+ §8.1（Wt-1 tick 定谳）；八关同模板（`docs/findings/stg02-timeline.md`
//! §6 交接口径）。
//!
//! **两时序域分离（stg02 §6.1 红线，勿混）**：
//! - **tick 域**：记录 delay / 波次 thr / 脚本 tick——1 tick = 16px 滚动进度（Wt-2 裁定）；
//!   稳态帧率由速度表驱动，**禁止换算常数跨关复用**；
//! - **帧域**：组员出敌间隔（`$534`，10f/15f/30f 直接兑现）。
//!
//! **未证不发明**（stg02 §6.5）：type 字节=对象类型表值（不命名）；ED3B/DB4C/ED6C 参数、
//! 行为 VM 脚本、事件机消费流（Wt-4）语义未证 ⇒ 解码透传不解释。
//!
//! 记录触发 tick 口径（工具 `+Nt`）：首记录 = 流头字节+1（sub_86A0 `$51:=(@0)+1` quirk），
//! 其后 tick_N = tick_{N-1} + delay_N（86BD `$51:=($14),Y` 1:1 装载，无移位无 +1）；
//! trace `$84` = 本 tick − 1（stg02 §3.4 对账，10/10）。

/// bank 数据窗基址（MMC3 R7 数据 bank 映射 CPU `$A000-$BFFF`；窗偏移 = addr − $A000）。
const WINDOW_BASE: u16 = 0xA000;

/// 调度流记录项（bank00:86CE 记录机 handler 字节语义，A 级清点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedItem {
    /// `(type≥$10, param)` 对——直接出敌项（type→`$400+槽`、param→`$470+槽`）。
    Pair {
        type_: u8,
        param: u8,
    },
    /// ctrl 1/2/11/12：定型出敌（type=$2E/$2F/$2C/$2D）+ 四参
    /// `(组员type, 组员数, 间隔帧, 参数)`（写点 `$518/$630/$534/$470`，八关同源写链）。
    Spawn {
        ctrl: u8,
        type_: u8,
        args: [u8; 4],
    },
    /// ctrl 3..9：流跳转（2B 新流基址，头字节=新 delay）。
    Jump {
        ctrl: u8,
        target: u16,
    },
    /// ctrl 10：流内定型（type 取流字节、param=第 2 操作数）。
    SetType {
        type_: u8,
        param: u8,
    },
    /// ctrl 13/14：寄存器写 `$E63D`（Y=0,1 / Y=4,5，音画参数——透传不解释）。
    Reg {
        ctrl: u8,
        a: u8,
        b: u8,
    },
    /// 未证 ctrl（handler 表缺失）——fail-closed 停止解码。
    Unknown {
        ctrl: u8,
    },
}

/// 调度流一条记录：`[delay@0][项…][ctrl]`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedRecord {
    /// 记录头 CPU 地址（bank `$A000` 窗）。
    pub addr: u16,
    /// 记录头 delay 字节 = 上一记录触发至本记录触发的 gap tick
    /// （首记录经 `$51:=(@0)+1` 带 +1 quirk——见 `tick`）。
    pub delay: u8,
    /// 项序列（不含记录终止 ctrl）。
    pub items: Vec<SchedItem>,
    /// 触发累计 tick（工具 `+Nt` 口径；trace `$84` = tick − 1）。
    pub tick: u32,
}

/// 调度流解码结果。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedStream {
    /// 按触发序排列的记录。
    pub records: Vec<SchedRecord>,
    /// 流跳转回已见记录址（`[9]JMP` 自环/环）⇒ 停止展开，环目标。
    pub loop_target: Option<u16>,
    /// 环目标下一圈触发 tick（= 环前记录 tick + 环目标 delay 字节）。
    pub loop_tick: Option<u32>,
    /// 视界截断（达 `max_records` 或读出 bank 窗）——不完整视界，勿据此下范围结论。
    pub truncated: bool,
    /// 命中的未证 ctrl（fail-closed：停止解码，其后字节不续解）。
    pub unknown_ctrl: Option<u8>,
}

/// 解码敌波调度流（bank00:86CE 记录机字节语义）。
///
/// `window` = 数据 bank 的 `$A000` 窗字节（偏移 = addr − $A000）；`stream_start` =
/// 流头 CPU 地址（脚本 op2 `$4F/$50` 装载值）；`max_records` = 解码视界（防自环/长流
/// 无限展开——超出置 `truncated`）。
///
/// fail-closed 约定：未知 ctrl（handler 表外）→ 记录含 `Unknown` 项并**停止**；
/// 越窗读 → 当前记录收尾 + `truncated`；跳转目标越窗 → `truncated` 停止。
pub fn decode_sched(window: &[u8], stream_start: u16, max_records: usize) -> SchedStream {
    let byte_at = |addr: u16| -> Option<u8> {
        let off = addr.wrapping_sub(WINDOW_BASE) as usize;
        if addr >= WINDOW_BASE && off < window.len() {
            Some(window[off])
        } else {
            None
        }
    };

    let mut out = SchedStream::default();
    let mut cur = stream_start;
    let mut tick: u32 = 0;
    let mut seen: Vec<u16> = Vec::new();
    let mut first = true;

    loop {
        // tick 推进：首记录 $51:=(@0)+1（+1 quirk）；后续 1:1 装载（86BD）。
        let Some(delay) = byte_at(cur) else {
            out.truncated = true;
            break;
        };
        if first {
            tick = u32::from(delay) + 1;
            first = false;
        } else {
            tick += u32::from(delay);
        }
        // 环检测（工具同口径：记录址已见 ⇒ 环，tick 已推进到"下一圈"）。
        if seen.contains(&cur) {
            out.loop_target = Some(cur);
            out.loop_tick = Some(tick);
            break;
        }
        if out.records.len() >= max_records {
            out.truncated = true;
            break;
        }
        seen.push(cur);

        // ── 项扫描（[delay@0] 之后至记录终止）──
        let mut items: Vec<SchedItem> = Vec::new();
        let mut y = cur.wrapping_add(1);
        let mut next: Option<u16> = None; // ctrl0 分隔 ⇒ 下一记录址
        let mut jumped: Option<u16> = None; // ctrl3..9 跳转目标
        let mut stopped = false; // 未知 ctrl / 越窗 ⇒ 无后继
        loop {
            let Some(v) = byte_at(y) else {
                out.truncated = true;
                stopped = true;
                break;
            };
            if v >= 0x10 {
                // (type,param) 对：type≥$10。
                let Some(param) = byte_at(y.wrapping_add(1)) else {
                    out.truncated = true;
                    stopped = true;
                    break;
                };
                items.push(SchedItem::Pair { type_: v, param });
                y = y.wrapping_add(2);
                continue;
            }
            y = y.wrapping_add(1);
            match v {
                // ctrl0：记录分隔（下一 delay=结束点字节）。
                0 => {
                    next = Some(y);
                    break;
                }
                // ctrl1/2/11/12：定型出敌 + 4 参（写链 bank00:8743-8763 八关同源）。
                1 | 2 | 11 | 12 => {
                    let mut args = [0u8; 4];
                    let mut ok = true;
                    for (i, slot) in args.iter_mut().enumerate() {
                        match byte_at(y.wrapping_add(i as u16)) {
                            Some(b) => *slot = b,
                            None => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if !ok {
                        out.truncated = true;
                        stopped = true;
                        break;
                    }
                    let type_ = match v {
                        1 => 0x2E,
                        11 => 0x2C,
                        2 => 0x2F,
                        _ => 0x2D,
                    };
                    items.push(SchedItem::Spawn { ctrl: v, type_, args });
                    y = y.wrapping_add(4);
                }
                // ctrl3..9：流跳转（2B 新基址，头字节=新 delay）——记录终止。
                3..=9 => match (byte_at(y), byte_at(y.wrapping_add(1))) {
                    (Some(lo), Some(hi)) => {
                        let target = u16::from_le_bytes([lo, hi]);
                        items.push(SchedItem::Jump { ctrl: v, target });
                        jumped = Some(target);
                    }
                    _ => {
                        out.truncated = true;
                        stopped = true;
                    }
                },
                // ctrl10：流内定型（type 取流字节、param=第 2 操作数）。
                10 => match (byte_at(y), byte_at(y.wrapping_add(1))) {
                    (Some(a), Some(b)) => {
                        items.push(SchedItem::SetType { type_: a, param: b });
                        y = y.wrapping_add(2);
                    }
                    _ => {
                        out.truncated = true;
                        stopped = true;
                        break;
                    }
                },
                // ctrl13/14：寄存器写 $E63D（音画参数，透传不解释）。
                13 | 14 => match (byte_at(y), byte_at(y.wrapping_add(1))) {
                    (Some(a), Some(b)) => {
                        items.push(SchedItem::Reg { ctrl: v, a, b });
                        y = y.wrapping_add(2);
                    }
                    _ => {
                        out.truncated = true;
                        stopped = true;
                        break;
                    }
                },
                // 未证 ctrl：fail-closed 停止（不发明 handler）。
                other => {
                    items.push(SchedItem::Unknown { ctrl: other });
                    out.unknown_ctrl = Some(other);
                    stopped = true;
                    break;
                }
            }
            if jumped.is_some() {
                break;
            }
        }

        out.records.push(SchedRecord {
            addr: cur,
            delay,
            items,
            tick,
        });

        if let Some(t) = jumped {
            cur = t;
            continue;
        }
        if let Some(n) = next {
            cur = n;
            continue;
        }
        let _ = stopped;
        break;
    }
    out
}

/// tick→视频帧映射（**STG-01 专用**，B1 §8.1 Wt-1 实测口径）。
///
/// - tick 1..=17：关首快进环（bank00:806A-80D9 do-while，每迭代=1 全链 tick）
///   全部压入 **f1193** 单视频帧（三计数器独立闭合：`$84=$FF+17=$10`、`$8D` 17 步、
///   `$51=32−16=$10`）；
/// - tick 18+k（k≥0）：稳态 **8 帧/tick**，首稳态派发点 f1221（快进帧后 28f 相位 = Wt-1' 残余）。
///
/// ⚠ **禁止跨关复用本换算**（stg02 §6.1 红线）：稳态帧率由速度表驱动
/// （STG-02 实测 109f 首 tick → 16f/tick ramp），跨关映射待速度表考证后参数化。
/// 组员间隔（`$534`）是**帧域**，不经本函数。
pub fn stg01_tick_to_frame(tick: u32) -> u32 {
    if tick <= 17 {
        1193
    } else {
        1221 + (tick - 18) * 8
    }
}

// ===== 批次2：记录处理出参（L2 对拍面）+ 组员投放节律（帧域）=====

/// 记录项处理出参（L2 型差分面：6502 核跑 `bank00:872C-87EA` 写集 == 本枚举值）。
///
/// 写点归属（八关同源，stg02 §3.1）：四 ctrl 收敛同一写链 `bank00:8743-8763`；
/// ctrl2/12 附加 `bank00:8783-878E`；ctrl10 `bank00:87A4-87CA`。
/// 槽选择（`$C2C6` 分配器）与组员槽字段写主**未证** ⇒ 本枚举只出写值，不出槽位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemEffect {
    /// `(type,param)` 对（type≥$10）：`$400+槽:=type`、`$470+槽:=param`（86E6 出敌写链）。
    PairWrite { type_: u8, param: u8 },
    /// ctrl1/2/11/12 四参写链（8743-8763）：`$400+槽:=leader_type`、
    /// `args→$518/$630/$534/$470`（=组员type/组员数/间隔帧/参数）。
    QuadWrite {
        leader_type: u8,
        args: [u8; 4],
        /// ctrl2/12 附加 `($3E),0→$48C+槽`（8783-878E）；ctrl1/11 不读码流 ⇒ None。
        write_48c: Option<u8>,
        /// 码流步进 `INC $3E/$3F`（8763-876E）：判据=**leader_type&1**（非 ctrl 判据）。
        code_stream_step: bool,
    },
    /// ctrl10 流内定型（87A4-87CA）：`$400+槽:=type`、`$470+槽:=param`、
    /// `$48C+槽:=($3E),0`、码流**无条件**步进（87C4-87C8）。
    SetTypeWrite {
        type_: u8,
        param: u8,
        write_48c: u8,
        code_stream_step: bool,
    },
    /// ctrl13/14（87CD-87F0）：寄存器写 `$E63D(Y=0,1 / 4,5)` 透传（音画参数语义未证）。
    RegWrite { ctrl: u8, a: u8, b: u8 },
    /// ctrl0 记录分隔 / ctrl3..9 流跳转（8795-87A3 只改流指针）：无槽写集。
    NoWrite,
    /// 未证 ctrl（handler 表外）：**不发明出参**（fail-closed）。
    Unproven,
}

/// 项 → 处理出参。`code_stream_byte` = 处理时刻 `($3E),0` 值（仅 ctrl2/12/10 消费）。
pub fn item_effect(item: &SchedItem, code_stream_byte: u8) -> ItemEffect {
    match *item {
        SchedItem::Pair { type_, param } => ItemEffect::PairWrite { type_, param },
        SchedItem::Spawn { ctrl, type_, args } => {
            let write_48c = if ctrl == 2 || ctrl == 12 {
                Some(code_stream_byte)
            } else {
                None
            };
            ItemEffect::QuadWrite {
                leader_type: type_,
                args,
                write_48c,
                code_stream_step: type_ & 1 != 0,
            }
        }
        SchedItem::SetType { type_, param } => ItemEffect::SetTypeWrite {
            type_,
            param,
            write_48c: code_stream_byte,
            code_stream_step: true,
        },
        SchedItem::Reg { ctrl, a, b } => ItemEffect::RegWrite { ctrl, a, b },
        SchedItem::Jump { .. } => ItemEffect::NoWrite,
        SchedItem::Unknown { .. } => ItemEffect::Unproven,
    }
}

/// 组员投放事件帧序列（**帧域**——`$534` 间隔字节直接以帧兑现，勿乘 tick 换算）。
///
/// 可观察契约（A 级 trace 双关四例 10f/15f/30f）：**首员与组长同帧**（STG-01 f1725
/// slot13 `$4E` 与 slot24 `$2F` 同帧；f1341 slot8 与 slot27 同帧），其后每
/// `interval_frames` 帧一员，共 `count` 员 ⇒ 帧 = `leader_frame + k×interval`，
/// k=0..count−1。组员槽字段写主未证（stg02 §3.4）⇒ 本函数只出帧序列，不出字段。
///
/// fail-closed：`count==0` 或 `interval_frames==0` 均未观测 ⇒ 返回 `None`（不发明）。
pub fn member_schedule(count: u8, interval_frames: u8, leader_frame: u32) -> Option<Vec<u32>> {
    if count == 0 || interval_frames == 0 {
        return None;
    }
    Some(
        (0..u32::from(count))
            .map(|k| leader_frame + k * u32::from(interval_frames))
            .collect(),
    )
}

/// 组员投放状态机（`$630` 递减字段 + `$534` 帧域间隔；节律机构可观察契约）。
///
/// 装载即投首员（stg02 §3.4「装载后先递减一次」）⇒ **`$630` 首现 = count−1**；
/// 其后每 `interval_frames` 帧投一员、`$630` 递减，至 0 止（总投员 = count）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupRelease {
    /// `$630` 当前值（未投余数；首现 = count−1）。
    pub remaining: u8,
    /// `$534` 出敌间隔（**帧**）。
    pub interval_frames: u8,
    next_in: u8,
}

impl GroupRelease {
    /// 装载四参 count/interval：投首员。返回 (状态, `$630` 首现值)。
    /// `count==0` / `interval_frames==0` 未观测 ⇒ `None`（fail-closed 不发明）。
    pub fn load(count: u8, interval_frames: u8) -> Option<(Self, u8)> {
        if count == 0 || interval_frames == 0 {
            return None;
        }
        let first_seen = count - 1; // 装载后先递减一次（首员即投）
        Some((
            GroupRelease {
                remaining: first_seen,
                interval_frames,
                next_in: interval_frames,
            },
            first_seen,
        ))
    }

    /// 推进一帧：返回本帧是否投员（间隔到且尚有余员）。
    pub fn tick_frame(&mut self) -> bool {
        if self.remaining == 0 {
            return false;
        }
        self.next_in -= 1;
        if self.next_in == 0 {
            self.remaining -= 1;
            self.next_in = self.interval_frames;
            return true;
        }
        false
    }
}

// ===== 批次4d⑤：item 排程两时序域 + $4B 饱和计数（enemy-item-scheduling §1/§4）=====

/// 投放时序域（**红线**，enemy-item-scheduling §4）：一条记录=tick 原子批量投放，
/// 但 ctrl1/2/11/12 组员流是**跨 tick 帧滴灌**（$534 帧间隔逐员）——两域不可混。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnTiming {
    /// tick 原子：记录内全量一次落地（pair/set10 直投、ctrl13/14 寄存器写）。
    TickAtomic,
    /// 帧滴灌：四参挂载后组员调度器按 `$534` 帧间隔展开（`GroupRelease`）。
    FrameDrip,
}

/// 项 → 投放时序域（无投放项 ⇒ None）。
pub fn spawn_timing(item: &SchedItem) -> Option<SpawnTiming> {
    match item {
        SchedItem::Pair { .. } | SchedItem::SetType { .. } | SchedItem::Reg { .. } => {
            Some(SpawnTiming::TickAtomic)
        }
        SchedItem::Spawn { .. } => Some(SpawnTiming::FrameDrip),
        SchedItem::Jump { .. } | SchedItem::Unknown { .. } => None,
    }
}

/// `$4B` 饱和计数（`bank00:86F3-86F8`：`INC $4B / BNE / DEC $4B`）——`$FF` 不回绕 0。
pub fn spawn_count_bump(counter: u8) -> u8 {
    counter.saturating_add(1)
}

// ===== 批次4e：S4b-6/7 闭合面（enemy-item-scheduling.md §6，A 级）=====
//
// $EEE5 累加器加权进位派发 + $4B 清算链 + 双分配器差异。深层派发义（$EF49 表/
// 关号钳制/权重数值义）= S4b-6c 留白不发明。

/// `$A325` 权重表（bank02 `$A325`，file 0x4325，18B 字节精确）：
/// **表区与代码混排**（含 `JMP $E63D`/`JMP $D10C` 指令字节）——权重数值义未证（S4b-6c），
/// 仅登记字节。
pub const A325_WEIGHTS: [u8; 18] = [
    0x4C, 0x3D, 0xE6, 0x00, 0x00, 0x00, 0x80, 0xC0, 0x40, 0x00, 0x00, 0x00, 0x40, 0xC0, 0x80,
    0x4C, 0x0C, 0xD1,
];

/// `$4B` 清算权重（`bank02:8663-8675`：`CPY #$12 / < $12 ⇒ LDA $A325,Y / else LDA #1`）。
pub fn settle_weight(count_4b: u8) -> u8 {
    if (count_4b as usize) < A325_WEIGHTS.len() {
        A325_WEIGHTS[count_4b as usize]
    } else {
        1
    }
}

/// `$EEE5` 累加器加权进位（`bank15:EEE5-EF1E`，A 级字节）：
/// `CLC/ADC $48/STA $48`（`$48 += weight`）⇒ 无进位返回；有进位 `INC $49`（**饱和不回 0**）。
/// 返回 `(lo, hi, carried)`。其后高半字节门/派发表/关号钳制 = S4b-6c 留白。
pub fn eee5_accumulate(acc_lo: u8, acc_hi: u8, weight: u8) -> (u8, u8, bool) {
    let sum = u16::from(acc_lo) + u16::from(weight); // CLC/ADC
    let lo = (sum & 0xFF) as u8;
    let carried = sum > 0xFF;
    if carried {
        // INC $49 / BNE / DEC $49——$FF 不回 0（饱和）
        (lo, acc_hi.saturating_add(1), true)
    } else {
        (lo, acc_hi, false)
    }
}

/// pair 项逐项累加（`86EE LDA #8 / JSR $EEE5`——**pair 权重恒 8**；三组长/set10 不调）。
pub fn pair_item_accumulate(acc_lo: u8, acc_hi: u8) -> (u8, u8, bool) {
    eee5_accumulate(acc_lo, acc_hi, 8)
}

/// `$4B` 清算链（pair 项 → `$4B` 计数（`spawn_count_bump`）→ `$A325` 权重 → `$EEE5` 累加器）：
/// `bank02:8663` 查权重灌入累加器后 `$4B:=0`。返回 (新 lo, 新 hi, 权重)。
pub fn pair_settle(acc_lo: u8, acc_hi: u8, count_4b: u8) -> (u8, u8, u8) {
    let weight = settle_weight(count_4b);
    let (lo, hi, _) = eee5_accumulate(acc_lo, acc_hi, weight);
    (lo, hi, weight)
}

/// 对象网格槽视图（28 槽系统，分配器判据三元组；槽 0..7 = HUD 保留区不扫描）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotView {
    /// `$400+槽` 型/状态（0=空槽）。
    pub type_: u8,
    /// `$668+槽` 标志（回收判据 bit7）。
    pub flags_668: u8,
    /// `$41C+槽` 行为指针 hi（回收判据 ≠0）。
    pub vec_41c: u8,
}

/// 槽域：对象网格 28 槽系统的可分配段 **8..=27**（0..7=HUD 保留，N-0120 分区）。
pub const SLOT_RANGE: core::ops::RangeInclusive<u8> = 8..=0x1B;

/// pair 项分配器（`$C304`，S4b-7）：**单遍顺序 first-fit**（Y=8→$1B），空槽判
/// `$400+槽==0`，**零副作用**。失败 ⇒ None（SEC）。
pub fn alloc_c304(slots: &[SlotView; 28]) -> Option<u8> {
    for slot in SLOT_RANGE {
        if slots[slot as usize].type_ == 0 {
            return Some(slot);
        }
    }
    None
}

/// ctrl/set10 分配器出参（`$C2C6` 三级回退，S4b-7）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocOutcome {
    /// ①逆序空槽命中（零副作用）。
    Empty { slot: u8 },
    /// ②顺序重指派窗命中（type∈[$20,$28)）——命中走初始化（`$CBAF/$D10C`，调用方执行）。
    Reassign { slot: u8 },
    /// ③逆序回收候选命中（`$668 bit7=0 ∧ $41C≠0`）——同上初始化副作用。
    Recycle { slot: u8 },
    /// SEC 失败。
    Full,
}

/// ctrl/set10 分配器（`$C2C6`，S4b-7）：**三级回退**——
/// ①逆序 Y=$1B→8 空槽（零副作用）→ ②顺序 Y=8→$1B `type∈[$20,$28)` 重指派窗
/// → ③逆序回收候选（`$668+槽 bit7=0 ∧ $41C+槽≠0`）→ SEC。
/// ②/③ 命中附带槽初始化副作用（`JSR $CBAF/JSR $D10C`——本函数只出判定，初始化由调用方）。
/// **保真面（D4）**：与 `alloc_c304` 槽序不同（顺序 vs 逆序首级）——同负载轨迹可分歧。
pub fn alloc_c2c6(slots: &[SlotView; 28]) -> AllocOutcome {
    // ① 逆序空槽
    for slot in SLOT_RANGE.rev() {
        if slots[slot as usize].type_ == 0 {
            return AllocOutcome::Empty { slot };
        }
    }
    // ② 顺序重指派窗 [$20,$28)
    for slot in SLOT_RANGE {
        if (0x20..0x28).contains(&slots[slot as usize].type_) {
            return AllocOutcome::Reassign { slot };
        }
    }
    // ③ 逆序回收候选
    for slot in SLOT_RANGE.rev() {
        let s = &slots[slot as usize];
        if s.flags_668 & 0x80 == 0 && s.vec_41c != 0 {
            return AllocOutcome::Recycle { slot };
        }
    }
    AllocOutcome::Full
}
