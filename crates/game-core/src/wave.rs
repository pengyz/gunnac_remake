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
