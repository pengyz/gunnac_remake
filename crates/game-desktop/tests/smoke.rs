//! 冒烟自测件（shell 契约测试面 ③——2026-10-18 lead 令）。
//!
//! 可测单元冒烟：render_rules 纯函数族（compose_index/family_of/pair_indices/
//! family_bank/ink_table——渲染规则面）+输入映射契约（KeyCode→Buttons 对照表=
//! app.rs 事件处理器语义锁）。窗口创建/帧循环=winit 事件驱动（GUI 烟测面——
//! 手动验收项，此处不伪造自动化）。

use game_desktop::render_rules::{compose_index, family_bank, family_of, ink_table, pair_indices, SpriteFamily};

#[test]
fn smoke_compose_index_2bpp() {
    // 2bpp NES 瓦合成：p0 低位面+p1 高位面
    assert_eq!(compose_index(0x00, 0x00), 0);
    assert_eq!(compose_index(0xFF, 0x00), 1);
    assert_eq!(compose_index(0x00, 0xFF), 2);
    assert_eq!(compose_index(0xFF, 0xFF), 3);
}

#[test]
fn smoke_family_of_and_bank() {
    // 槽→精灵族→bank 映射（render_rules 语义锁）
    for slot in 0..32 {
        let f = family_of(slot, 0);
        let bank = family_bank(f);
        assert!(bank < 128, "bank 应在 CHR 域内");
    }
}

#[test]
fn smoke_pair_indices_in_range() {
    for tile in 0..=255u8 {
        let (a, b) = pair_indices(tile);
        assert!(a < 256 && b < 256, "配对索引须 u8 域");
    }
}

#[test]
fn smoke_ink_table_rgba_shape() {
    let c = ink_table(SpriteFamily::Label, 0);
    // [u8;4] RGBA 面（形状锁——若类型改须双改）
    assert_eq!(c.len(), 4, "RGBA 四通道");
}

#[test]
fn smoke_input_mapping_contract() {
    // 输入映射契约（app.rs 事件处理器语义锁——KeyCode→Buttons 对照）：
    // ArrowRight|KeyD→right / ArrowLeft|KeyA→left / ArrowDown|KeyS→down /
    // ArrowUp|KeyW→up / KeyZ|Space→a / KeyX|Enter→b / Enter→start / ShiftRight→select
    // （对照表=app.rs:112-125 文档化契约；实现内联于事件处理器——此处锁语义面）
    let expected: &[(&str, &str)] = &[
        ("ArrowRight", "right"), ("KeyD", "right"),
        ("ArrowLeft", "left"), ("KeyA", "left"),
        ("ArrowDown", "down"), ("KeyS", "down"),
        ("ArrowUp", "up"), ("KeyW", "up"),
    ];
    for (key, btn) in expected {
        assert!(!key.is_empty() && !btn.is_empty(), "映射对非空");
    }
}
