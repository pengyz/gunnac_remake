//! 文本记录解析器（L1）—— 结构规则全部来自 A 级证据，禁止在此层发明语义。
//!
//! 证据（`docs/findings/data-tables.md` §2「文本记录格式」，双投影一致）：
//! - 记录 = `{ppu_lo, ppu_hi, tile 码流…, 终止字节}`；终止字节 **≥ $FE**；
//!   消费引擎 `bank15:sub_E6DF`（`bank15:E6EF` `LDA ($12),Y / CMP #$FE / BCC …`）
//!   ⇒ 字节 < $FE = 记录内 tile 码，≥ $FE ⇒ 记录结束。
//! - 流级终止规则（`bank00:$96D8-$971B` 同族消费器）：**$FE** = 本记录结束、继续读下一条
//!   （`bank00:96DD` `CMP #$FE/BEQ`）；**$FF** = 整块结束（`bank00:96E1`）。
//! - 记录边界 = 上一记录终止字节后 1 字节即下一记录 `ppu_lo`（五区 hexdump 复核）。
//!
//! 边界（本层不解释，渲染层再做）：tile 流控制字节（$5E/$86-$9F/$A6-$DD/$A0-$A5/$B0/
//! $DE-$DF，见 findings §2）在解析结果中保持原样；块结束后的填充/续块指针（00/FF/…）
//! 不属于块，由调用方在 `consumed` 之后自行处理。

use std::fmt;

/// 记录终止语义（u8 ≥ $FE 只有 $FE/$FF 两种，穷尽）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordEnd {
    /// $FE：本记录结束，其后仍是记录（流继续）。
    Continue,
    /// $FF：整块结束。
    BlockEnd,
}

impl RecordEnd {
    fn from_u8(b: u8) -> Option<Self> {
        match b {
            0xFE => Some(RecordEnd::Continue),
            0xFF => Some(RecordEnd::BlockEnd),
            _ => None,
        }
    }
}

/// 一条文本记录：PPU 目标地址 + 原样 tile 码流 + 终止语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRecord {
    /// 目标 PPU 地址（记录头 `ppu_lo, ppu_hi`，小端组 u16）。
    pub ppu_addr: u16,
    /// tile 码流（全部 < $FE 的原始字节；控制码语义不在本层解释）。
    pub tiles: Vec<u8>,
    pub end: RecordEnd,
}

/// 一个文本块：有序记录 + 实际消费字节数 + 结束方式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBlock {
    pub records: Vec<TextRecord>,
    /// 从输入起点消费的字节数（含块尾 $FF；无 $FF 时 = 输入长度）。
    pub consumed: usize,
    /// 最后一条记录的终止语义（`Continue` = 输入在记录边界耗尽而未见 $FF）。
    pub end: RecordEnd,
}

/// 解析错误：截断一律报错，禁止半截记录静默通过。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextParseError {
    /// 输入为空：不是合法块。
    EmptyInput,
    /// 记录头不足 2 字节（`at` = 该残缺头的偏移）。
    TruncatedHeader { at: usize },
    /// tile 流没有 ≥$FE 终止即耗尽（`at` = 该记录起点偏移）。
    TruncatedStream { at: usize },
}

impl fmt::Display for TextParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextParseError::EmptyInput => write!(f, "空输入不是合法文本块"),
            TextParseError::TruncatedHeader { at } => write!(f, "记录头截断于 {at:#x}（<2 字节）"),
            TextParseError::TruncatedStream { at } => {
                write!(f, "记录流无 ≥$FE 终止即耗尽（记录起点 {at:#x}）")
            }
        }
    }
}

impl std::error::Error for TextParseError {}

/// 解析一个文本块（从记录边界开始，到 $FF 块尾或输入耗尽）。
pub fn parse_text_block(data: &[u8]) -> Result<TextBlock, TextParseError> {
    if data.is_empty() {
        return Err(TextParseError::EmptyInput);
    }
    let mut records = Vec::new();
    let mut pos = 0usize;
    let mut end = RecordEnd::Continue;
    loop {
        let start = pos;
        if data.len() - pos == 1 {
            return Err(TextParseError::TruncatedHeader { at: start });
        }
        if data.len() - pos < 2 {
            break; // 恰好在记录边界耗尽（此前是 FE 终止）：块尾未见 FF
        }
        let ppu_addr = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;
        let tiles_start = pos;
        loop {
            if pos >= data.len() {
                return Err(TextParseError::TruncatedStream { at: start });
            }
            if data[pos] >= 0xFE {
                break;
            }
            pos += 1;
        }
        let terminator = RecordEnd::from_u8(data[pos]).expect("≥$FE 只能是 $FE/$FF");
        pos += 1;
        records.push(TextRecord {
            ppu_addr,
            tiles: data[tiles_start..pos - 1].to_vec(),
            end: terminator,
        });
        end = terminator;
        if terminator == RecordEnd::BlockEnd {
            break;
        }
    }
    Ok(TextBlock { records, consumed: pos, end })
}
