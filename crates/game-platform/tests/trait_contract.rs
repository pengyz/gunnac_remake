//! game-platform trait 契约测试框架（shell 契约测试面 ①——2026-10-18 lead 令）。
//!
//! 验收面：Renderer/Audio/Storage/Clock/GameRng/Input 六 trait 的 mock 实现 +
//! 确定性验证（同输入序列→同 mock 态）+ 形状锁（canvas 256×240×4/流序保持/存档往返）。
//! trait 形状=platform-shell.md §2 锁定面——形状变更须双改（trait+本框架）。
//!
//! v1 增补（task-5）：AssetSource/Lifecycle 契约测试 + Renderer 输出面 size/resize
//! （默认=占位回退，平台实现覆盖）。

use game_platform::{
    AppEvent, AssetSource, Audio, Buttons, Clock, GameRng, Input, Lifecycle, Renderer, Storage,
    CANVAS_H, CANVAS_LEN, CANVAS_W,
};

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

// ---------------- Clock / Input mock 真断言（复核：原 mock 从未被构造） ----------------

#[test]
fn clock_and_input_mocks_are_asserted() {
    let mut clock = MockClock { ms: 0 };
    for _ in 0..60 {
        clock.ms += 16;
    }
    assert_eq!(clock.now_ms(), 960, "Clock::now_ms 真实断言（60×16ms）");
    let mut input = MockInput { buttons: Buttons::default() };
    assert_eq!(input.buttons(), Buttons::default(), "Input::buttons 空态");
    input.buttons = Buttons { a: true, start: true, ..Buttons::default() };
    assert!(input.buttons().a, "Input::buttons A");
    assert!(input.buttons().start, "Input::buttons START");
    assert!(!input.buttons().b, "Input::buttons B 未按");
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

// ---------------- v1 新增：AssetSource（占位契约） ----------------

/// mock 资产源（内存文件表；不存在⇒None=fail-closed）。
struct MockAssetSource {
    files: Vec<(&'static str, Vec<u8>)>,
}

impl AssetSource for MockAssetSource {
    fn read_all(&mut self, path: &str) -> Option<Vec<u8>> {
        self.files.iter().find(|(p, _)| *p == path).map(|(_, b)| b.clone())
    }
}

#[test]
fn asset_source_read_and_missing_fail_closed() {
    let mut src = MockAssetSource { files: vec![("rom/gunnac_cn.nes", vec![0x4E, 0x45, 0x53, 0x1A])] };
    assert_eq!(src.read_all("rom/gunnac_cn.nes"), Some(vec![0x4E, 0x45, 0x53, 0x1A]), "present⇒字节");
    assert_eq!(src.read_all("rom/missing.nes"), None, "缺失⇒None（不提供默认内容）");
}

// ---------------- v1 新增：Lifecycle/AppEvent（占位最小枚举） ----------------

#[derive(Default)]
struct MockLifecycle {
    events: std::collections::VecDeque<AppEvent>,
}

impl Lifecycle for MockLifecycle {
    fn poll_event(&mut self) -> Option<AppEvent> {
        self.events.pop_front()
    }
}

#[test]
fn lifecycle_event_queue_order_and_drain() {
    let mut lc = MockLifecycle::default();
    assert_eq!(lc.poll_event(), None, "空队列⇒None");
    lc.events.push_back(AppEvent::Suspend);
    lc.events.push_back(AppEvent::Resume);
    lc.events.push_back(AppEvent::Quit);
    assert_eq!(lc.poll_event(), Some(AppEvent::Suspend), "事件序保持 #1");
    assert_eq!(lc.poll_event(), Some(AppEvent::Resume), "事件序保持 #2");
    assert_eq!(lc.poll_event(), Some(AppEvent::Quit), "事件序保持 #3");
    assert_eq!(lc.poll_event(), None, "取尽后 None");
}

// ---------------- v1 新增：Renderer 输出面 size/resize（默认=占位回退） ----------------

#[test]
fn renderer_canvas_constants_locked() {
    assert_eq!((CANVAS_W, CANVAS_H, CANVAS_LEN), (256, 240, 245_760), "256×240 RGBA 常量");
    let r = MockRenderer::new();
    assert_eq!(r.canvas.len(), CANVAS_LEN, "mock canvas 长度=逻辑画布");
}

#[test]
fn renderer_size_resize_default_placeholder() {
    let mut r = MockRenderer::new();
    assert_eq!(r.size(), (256, 240), "默认 size=逻辑画布 1×（占位回退）");
    r.resize(1920, 1080); // 默认 resize=无平台动作
    assert_eq!(r.size(), (256, 240), "默认 resize 不改变尺寸（占位契约）");
    assert_eq!(r.canvas_mut().len(), CANVAS_LEN, "canvas 形状不因 resize 默认实现改变");
}

/// 覆盖 size/resize 的实现（模拟平台侧输出面）。
struct SizedRenderer {
    canvas: Vec<u8>,
    size: (u32, u32),
}

impl Renderer for SizedRenderer {
    fn canvas_mut(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
    fn present(&mut self) {}
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        self.canvas = vec![0; CANVAS_LEN]; // 逻辑画布不变，仅输出面变化
    }
}

#[test]
fn renderer_size_resize_override() {
    let mut r = SizedRenderer { canvas: vec![0; CANVAS_LEN], size: (512, 480) };
    assert_eq!(r.size(), (512, 480), "平台实现覆盖 size");
    r.resize(1024, 960);
    assert_eq!(r.size(), (1024, 960), "平台实现覆盖 resize");
    assert_eq!(r.canvas_mut().len(), CANVAS_LEN, "逻辑画布与输出面尺寸解耦");
}
