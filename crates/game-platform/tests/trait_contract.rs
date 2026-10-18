//! game-platform trait 契约测试框架（shell 契约测试面 ①——2026-10-18 lead 令）。
//!
//! 验收面：Renderer/Audio/Storage/Clock/GameRng/Input 六 trait 的 mock 实现 +
//! 确定性验证（同输入序列→同 mock 态）+ 形状锁（canvas 256×240×4/流序保持/存档往返）。
//! trait 形状=platform-shell.md §2 锁定面——形状变更须双改（trait+本框架）。

use game_platform::{Audio, Buttons, Clock, GameRng, Input, Renderer, Storage};

// ---------------- Mock 实现（最小面——只记录不渲染） ----------------

struct MockRenderer {
    canvas: Vec<u8>,
    present_count: u32,
}

impl MockRenderer {
    fn new() -> Self {
        Self { canvas: vec![0; 256 * 240 * 4], present_count: 0 }
    }
}

impl Renderer for MockRenderer {
    fn canvas_mut(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
    fn present(&mut self) {
        self.present_count += 1;
    }
}

#[derive(Default)]
struct MockAudio {
    writes: Vec<(u32, u16, u8)>,
    frames: Vec<u32>,
}

impl Audio for MockAudio {
    fn write_apu(&mut self, frame: u32, reg: u16, value: u8) {
        self.writes.push((frame, reg, value));
    }
    fn end_frame(&mut self, frame: u32) {
        self.frames.push(frame);
    }
}

#[derive(Default)]
struct MockStorage {
    slots: [Option<Vec<u8>>; 8],
    save_count: u32,
}

impl Storage for MockStorage {
    fn load(&mut self, slot: u8) -> Option<Vec<u8>> {
        self.slots.get(slot as usize).cloned().flatten()
    }
    fn save(&mut self, slot: u8, data: &[u8]) -> bool {
        if let Some(s) = self.slots.get_mut(slot as usize) {
            *s = Some(data.to_vec());
            self.save_count += 1;
            true
        } else {
            false
        }
    }
}

struct MockClock {
    ms: u64,
}

impl Clock for MockClock {
    fn now_ms(&self) -> u64 {
        self.ms
    }
}

struct MockRng {
    state: u64,
}

impl GameRng for MockRng {
    fn next_u32(&mut self) -> u32 {
        // xorshift 确定性（禁平台 RNG——contract.rs 红线同源）
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x >> 32) as u32
    }
}

struct MockInput {
    buttons: Buttons,
}

impl Input for MockInput {
    fn buttons(&self) -> Buttons {
        self.buttons
    }
}

// ---------------- 形状锁（trait 契约——变更须双改） ----------------

#[test]
fn renderer_shape_256x240_rgba() {
    let mut r = MockRenderer::new();
    assert_eq!(r.canvas_mut().len(), 256 * 240 * 4, "canvas 须为 256×240 RGBA");
    r.present();
    assert_eq!(r.present_count, 1);
}

#[test]
fn audio_stream_order_preserved() {
    let mut a = MockAudio::default();
    a.write_apu(1, 0x4000, 0x30);
    a.write_apu(1, 0x4001, 0x08);
    a.write_apu(2, 0x4015, 0x0F);
    a.end_frame(1);
    a.end_frame(2);
    assert_eq!(a.writes, vec![(1, 0x4000, 0x30), (1, 0x4001, 0x08), (2, 0x4015, 0x0F)]);
    assert_eq!(a.frames, vec![1, 2], "帧序保持");
}

#[test]
fn storage_roundtrip() {
    let mut s = MockStorage::default();
    assert!(s.save(0, b"abc"), "slot0 保存");
    assert_eq!(s.load(0), Some(b"abc".to_vec()), "slot0 往返");
    assert_eq!(s.load(1), None, "slot1 空");
    assert!(!s.save(9, b"x"), "越界槽拒");
}

// ---------------- 确定性验证（同输入→同态） ----------------

fn run_sequence() -> (Vec<(u32, u16, u8)>, Vec<u8>, Vec<Option<Vec<u8>>>) {
    let mut rng = MockRng { state: 0x1234_5678_9abc_def0 };
    let mut audio = MockAudio::default();
    let mut storage = MockStorage::default();
    let mut renderer = MockRenderer::new();
    for frame in 0..60u32 {
        let r = (rng.next_u32() >> 24) as u8;
        audio.write_apu(frame, 0x4000, r);
        renderer.canvas_mut()[(frame as usize * 4) % (256 * 240 * 4)] = r;
        audio.end_frame(frame);
        if frame % 20 == 0 {
            storage.save(0, &[r, frame as u8]);
        }
        renderer.present();
    }
    let canvas: Vec<u8> = renderer.canvas.clone();
    let slots: Vec<Option<Vec<u8>>> = (0..8).map(|i| storage.load(i)).collect();
    (audio.writes, canvas, slots)
}

#[test]
fn determinism_same_input_same_state() {
    let a = run_sequence();
    let b = run_sequence();
    assert_eq!(a, b, "同输入序列→同 mock 态（确定性锁）");
}

#[test]
fn rng_deterministic_from_seed() {
    let mut r1 = MockRng { state: 42 };
    let mut r2 = MockRng { state: 42 };
    for _ in 0..100 {
        assert_eq!(r1.next_u32(), r2.next_u32(), "同种子→同序列");
    }
    let mut r3 = MockRng { state: 43 };
    assert_ne!(
        (0..8).map(|_| MockRng { state: 42 }.next_u32()).collect::<Vec<_>>(),
        (0..8).map(|_| r3.next_u32()).collect::<Vec<_>>(),
        "异种子→异序列（抽验）"
    );
}
