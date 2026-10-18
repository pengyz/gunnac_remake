//! 敌框形数据装载（task-37）：ROM bank2（CPU `$8000` 窗）→ `StageState.shape_data`。
//!
//! 证据（A 级）：
//! - 框形指针表 `bank2:$97BA/$9812`（`enemy::SHAPE_PTR_LO/HI`，88 形）给出框形流 CPU 地址；
//! - 流格式 `[子框数][4B/框(左,右,上,下) 有符号偏移]`（arena-midboss §2.1：`$D511-$D5AE` 消费链）；
//! - bank2 落在 CPU `$8000` 窗（N-0094 双窗注：同 bank 双窗例 `$A325`=$A000 窗 / `$8406`=$8000 窗
//!   ——窗基按引用地址判；框形地址均为 `$98xx-$9Exx` ⇒ `$8000` 窗）；
//! - file offset=`0x10 + bank*0x2000 + (cpu_addr-$8000)`（iNES 头 + 8KB PRG bank）。
//!
//! 边界：解析越界/空流 ⇒ 跳过该形（fail-closed，不补零/不发明）；地址重复只保留首个。

use game_core::enemy::shape_stream_addr;
use game_core::engine::StageState;

/// 框形数据所在 PRG bank（bank2，CPU `$8000` 窗）。
pub const SHAPE_BANK: usize = 2;
/// 该 bank 的 CPU 窗基址（R6 → `$8000-$9FFF`）。
pub const SHAPE_WINDOW_BASE: u16 = 0x8000;
/// 框形形索引上限（表长 88）。
pub const SHAPE_COUNT_MAX: usize = 88;

/// 从 bank2 的 8KB 窗口字节解析全部框形流，写入 `stage.shape_data`；返回装载形数。
pub fn load_shape_data_from_bank2(bank2: &[u8], stage: &mut StageState) -> usize {
    let mut loaded = 0usize;
    for shape in 0..SHAPE_COUNT_MAX {
        let Some(addr) = shape_stream_addr(shape as u8) else {
            continue;
        };
        if addr < SHAPE_WINDOW_BASE {
            continue; // 非 $8000 窗地址 ⇒ 不在本 bank 窗（fail-closed 跳过）
        }
        let off = usize::from(addr - SHAPE_WINDOW_BASE);
        let Some(&count) = bank2.get(off) else {
            continue;
        };
        let need = 1 + usize::from(count) * 4;
        let Some(stream) = bank2.get(off..off + need) else {
            continue; // 流越窗（截断视界 ⇒ 跳过，不补零）
        };
        if stage.shape_data.iter().any(|(a, _)| *a == addr) {
            continue; // 表内地址重复 ⇒ 只保留首个
        }
        stage.shape_data.push((addr, stream.to_vec()));
        loaded += 1;
    }
    loaded
}

#[cfg(test)]
mod tests {
    use super::{load_shape_data_from_bank2, SHAPE_BANK, SHAPE_WINDOW_BASE};
    use game_core::enemy::shape_stream_addr;
    use game_core::engine::StageState;

    fn synthetic_bank2() -> Vec<u8> {
        let mut w = vec![0u8; 0x2000];
        // 形 0 的流：1 个子框 (左,右,上,下)=(-16,16,-16,16)
        let addr = shape_stream_addr(0).expect("形 0 地址");
        let off = usize::from(addr - SHAPE_WINDOW_BASE);
        w[off] = 1;
        w[off + 1] = 0xF0;
        w[off + 2] = 16;
        w[off + 3] = 0xF0;
        w[off + 4] = 16;
        w
    }

    #[test]
    fn loads_synthetic_shape_stream() {
        assert_eq!(SHAPE_BANK, 2);
        let mut stage = StageState::default();
        let n = load_shape_data_from_bank2(&synthetic_bank2(), &mut stage);
        assert_eq!(n, stage.shape_data.len(), "装载数=实际入表数");
        assert!(n >= 1);
        let addr = shape_stream_addr(0).unwrap();
        let stream = stage.shape_data.iter().find(|(a, _)| *a == addr).expect("形 0 已装载").1.clone();
        assert_eq!(stream, vec![1u8, 0xF0, 16, 0xF0, 16]);
        // fetch_shape_boxes 端到端：框 = x±16/y±16
        let rects = game_core::enemy::fetch_shape_boxes(&stream, 100, 100, false, false);
        assert_eq!(rects, vec![(84, 116, 84, 116)]);
    }

    #[test]
    fn fail_closed_on_short_window_and_empty_streams() {
        let mut stage = StageState::default();
        assert_eq!(load_shape_data_from_bank2(&[], &mut stage), 0, "空窗 ⇒ 0");
        let addr = shape_stream_addr(0).unwrap();
        let off = usize::from(addr - SHAPE_WINDOW_BASE);
        let w = synthetic_bank2();
        // 截断窗（形 0 起始地址越界）⇒ 跳过该形（不补零/不 panic）
        let mut stage2 = StageState::default();
        let _ = load_shape_data_from_bank2(&w[..off], &mut stage2);
        assert!(!stage2.shape_data.iter().any(|(a, _)| *a == addr), "越窗形被跳过（不补零）");
    }
}
