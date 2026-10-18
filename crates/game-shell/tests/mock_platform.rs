//! mock platform 契约测试（task-5 门禁）：
//!
//! 1. 用**真实实现**的 mock 覆盖 `game-platform` 全部 trait（Input/Renderer/Audio/Storage/
//!    AssetSource/Lifecycle），驱动 `Host` 300 帧；
//! 2. 固定输入脚本 + 固定 `ModeEvent` 调度（全部落在 flow 已证边集内），走完
//!    title→八关→结局；
//! 3. **连跑两次**，比较宿主确定性签名/画布 hash/音频记录/存档字节/呈现次数——必须全等；
//! 4. 打印签名（复现命令见 `docs/findings/plan-platform-abstraction.md` §9）。
//!
//! 纪律：mock 不含时间/线程/真实设备；事件调度只投递已证边，未证边 fail-closed（断言 Err）。

use game_core::flow::{Mode, ModeEvent};
use game_platform::{
    AppEvent, AssetSource, Audio, Buttons, Input, Lifecycle, Renderer, Storage, CANVAS_LEN,
};
use game_shell::{canvas_hash, Host, ROM_ASSET_PATH};

// ===================== mock platform 实现（真实消费，不空实现） =====================

struct ScriptedInput {
    buttons: Buttons,
}

impl Input for ScriptedInput {
    fn buttons(&self) -> Buttons {
        self.buttons
    }
}

struct MockRenderer {
    canvas: Vec<u8>,
    presents: u32,
    size: (u32, u32),
    resizes: u32,
}

impl MockRenderer {
    fn new() -> Self {
        Self { canvas: vec![0u8; CANVAS_LEN], presents: 0, size: (1280, 720), resizes: 0 }
    }
}

impl Renderer for MockRenderer {
    fn canvas_mut(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
    fn present(&mut self) {
        self.presents += 1;
    }
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        self.resizes += 1;
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
}

impl Storage for MockStorage {
    fn load(&mut self, slot: u8) -> Option<Vec<u8>> {
        self.slots.get(slot as usize).cloned().flatten()
    }
    fn save(&mut self, slot: u8, data: &[u8]) -> bool {
        if let Some(entry) = self.slots.get_mut(slot as usize) {
            *entry = Some(data.to_vec());
            true
        } else {
            false
        }
    }
}

struct MockAssets {
    rom: Vec<u8>,
}

impl MockAssets {
    fn new() -> Self {
        // 伪 NES 头（仅契约测试字节；不解析、不声称原版内容）
        Self { rom: vec![0x4E, 0x45, 0x53, 0x1A, 0x02, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] }
    }
}

impl AssetSource for MockAssets {
    fn read_all(&mut self, path: &str) -> Option<Vec<u8>> {
        if path == ROM_ASSET_PATH {
            Some(self.rom.clone())
        } else {
            None
        }
    }
}

#[derive(Default)]
struct MockLifecycle {
    events: std::collections::VecDeque<AppEvent>,
}

impl Lifecycle for MockLifecycle {
    fn poll_event(&mut self) -> Option<AppEvent> {
        self.events.pop_front()
    }
}

struct MockPlatform {
    input: ScriptedInput,
    renderer: MockRenderer,
    audio: MockAudio,
    storage: MockStorage,
    assets: MockAssets,
    lifecycle: MockLifecycle,
}

impl MockPlatform {
    fn new() -> Self {
        Self {
            input: ScriptedInput { buttons: Buttons::default() },
            renderer: MockRenderer::new(),
            audio: MockAudio::default(),
            storage: MockStorage::default(),
            assets: MockAssets::new(),
            lifecycle: MockLifecycle::default(),
        }
    }
}

// ===================== 确定性输入/事件脚本 =====================

/// 8 帧循环输入（含对向同持组合，用于验证消解）；每 64 帧加 START/SELECT。
fn scripted_buttons(frame: u64) -> Buttons {
    let mut b = Buttons::default();
    match frame % 8 {
        0 => b.right = true,
        1 => b.left = true,
        2 => b.up = true,
        3 => b.down = true,
        4 => {
            b.right = true;
            b.left = true;
        }
        5 => {
            b.up = true;
            b.down = true;
        }
        6 => b.a = true,
        _ => b.b = true,
    }
    if frame % 64 == 0 {
        b.start = true;
        b.select = true;
    }
    b
}

/// title→八关→结局 的事件序列（44 条，全部 game-core flow 已证边集）。
fn event_sequence() -> Vec<ModeEvent> {
    let mut seq = vec![
        ModeEvent::Black,
        ModeEvent::Title,
        ModeEvent::Menu,
        ModeEvent::ShopArrival,
        ModeEvent::ExitTransition,
        ModeEvent::StageStart,
        ModeEvent::MidCombat,
    ];
    for stage in 1..=8u8 {
        seq.push(ModeEvent::StageClear);
        if stage < 8 {
            seq.push(ModeEvent::StageAdvance);
            seq.push(ModeEvent::ExitTransition);
            seq.push(ModeEvent::StageStart);
            seq.push(ModeEvent::MidCombat);
        } else {
            seq.push(ModeEvent::AllClear);
        }
    }
    seq
}

// ===================== 300 帧运行 =====================

/// 一次完整运行的确定性观测面（两次运行必须全等）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct RunResult {
    host_signature: u64,
    canvas_signature: u64,
    renderer_canvas_signature: u64,
    present_count: u32,
    audio_writes: Vec<(u32, u16, u8)>,
    audio_frames: Vec<u32>,
    advanced_frames: u32,
    rom_len: Option<usize>,
    storage_slot0: Option<Vec<u8>>,
    final_mode: Mode,
    final_stage: u8,
    all_clear: bool,
    final_frame: u64,
    applied_events: usize,
}

/// suspend 窗口（帧域，不含事件帧；用于验证生命周期门控）。
const SUSPEND_AT: u64 = 200;
/// resume 帧。
const RESUME_AT: u64 = 210;
/// 资产装载帧。
const ASSET_AT: u64 = 3;
/// 存档保存帧。
const SAVE_AT: u64 = 100;
/// 音频注入帧（验证流消费；不等同于已证音频生产者）。
const AUDIO_INJECT_AT: u64 = 150;

fn run_once() -> RunResult {
    let mut host = Host::new();
    let mut platform = MockPlatform::new();
    let events = event_sequence();
    let mut next_event = 0usize;
    let mut applied_events = 0usize;
    let mut advanced_frames = 0u32;

    for f in 0..300u64 {
        // 生命周期：suspend→resume（宿主应停逻辑帧/停呈现）
        match f {
            SUSPEND_AT => platform.lifecycle.events.push_back(AppEvent::Suspend),
            RESUME_AT => platform.lifecycle.events.push_back(AppEvent::Resume),
            _ => {}
        }
        host.poll_lifecycle(&mut platform.lifecycle);

        // 已证 flow 事件调度（每 4 帧一条，共 44 条，全部落在 300 帧内）
        while next_event < events.len() && 10 + 4 * next_event as u64 == f {
            host.apply_event(events[next_event]).expect("事件序列全在已证边集内");
            next_event += 1;
            applied_events += 1;
        }

        // AssetSource / Storage 真实消费
        if f == ASSET_AT {
            assert!(host.load_asset(&mut platform.assets, ROM_ASSET_PATH), "AssetSource 装载");
            assert!(!host.load_asset(&mut platform.assets, "rom/missing.nes"), "缺失 fail-closed");
        }
        if f == SAVE_AT {
            assert!(host.save_snapshot(0, &mut platform.storage), "Storage 保存快照");
        }
        // Audio 流注入（消费契约验证；生产者 v1 未接）
        if f == AUDIO_INJECT_AT {
            host.audio_mut().stream_mut().write(0, 0x15, 0x0F);
            host.audio_mut().stream_mut().write(1, 0x00, 0x30);
        }

        platform.input.buttons = scripted_buttons(f);
        let out = host.tick(&platform.input, &mut platform.renderer, &mut platform.audio);
        if out.advanced {
            advanced_frames += 1;
        }

        // 渲染契约：每逻辑帧画布=256×240 RGBA；present 只在推进帧
        assert_eq!(platform.renderer.canvas.len(), CANVAS_LEN);
        assert_eq!(platform.renderer.presents, advanced_frames, "present 次数=逻辑帧数");
    }

    assert_eq!(applied_events, 44, "事件序列总数");
    assert_eq!(next_event, events.len(), "事件全部投递");

    let host_signature = host.signature();
    let canvas_signature = host.last_canvas_sig();
    let renderer_canvas_signature = canvas_hash(&platform.renderer.canvas);

    RunResult {
        host_signature,
        canvas_signature,
        renderer_canvas_signature,
        present_count: platform.renderer.presents,
        audio_writes: platform.audio.writes,
        audio_frames: platform.audio.frames,
        advanced_frames,
        rom_len: host.rom().map(|r| r.len()),
        storage_slot0: platform.storage.slots[0].clone(),
        final_mode: host.mode(),
        final_stage: host.stage_index(),
        all_clear: host.all_clear(),
        final_frame: host.frame(),
        applied_events,
    }
}

#[test]
fn mock_platform_drives_300_frames_deterministically() {
    let first = run_once();
    let second = run_once();

    assert_eq!(first, second, "连跑两次：宿主签名/画布/音频/存档/呈现全等");
    // 终态断言（title→八关→结局 全部走已证边集）
    assert_eq!(first.final_mode, Mode::Ending, "8 关通关→结局");
    assert_eq!(first.final_stage, 8, "关号=8（E37C 累计）");
    assert!(first.all_clear, "AllClear 已走");
    // 生命周期：suspend 窗口内不推进逻辑帧；故逻辑帧总数=300-(RESUME_AT-SUSPEND_AT)
    assert_eq!(first.advanced_frames, 300 - (RESUME_AT - SUSPEND_AT) as u32, "suspend 停逻辑帧");
    assert_eq!(first.final_frame, u64::from(first.advanced_frames), "虚拟时钟=推进帧数");
    // 渲染：present 次数=推进帧数；画布 hash 与宿主记录一致
    assert_eq!(first.present_count, first.advanced_frames);
    assert_eq!(first.canvas_signature, first.renderer_canvas_signature);
    // 音频：每逻辑帧一次 end_frame；注入的 2 条写保序消费
    assert_eq!(first.audio_frames.len(), first.advanced_frames as usize, "每帧 end_frame");
    assert_eq!(first.audio_writes, vec![(0, 0x0015, 0x0F), (1, 0x0000, 0x30)], "写序保真");
    // 资产/存档：mock 真实消费
    assert_eq!(first.rom_len, Some(16), "AssetSource 字节已装载");
    assert!(first.storage_slot0.is_some(), "Storage 快照已写入");

    println!(
        "mock-300 signature: host={:#018x} canvas={:#018x} advanced_frames={} storage_len={}",
        first.host_signature,
        first.canvas_signature,
        first.advanced_frames,
        first.storage_slot0.as_ref().map(|b| b.len()).unwrap_or(0)
    );
}

#[test]
fn mock_platform_traits_are_individually_asserted() {
    let mut platform = MockPlatform::new();
    let mut host = Host::new();

    // Input：脚本按钮真实消费，对向同持在宿主侧消解
    platform.input.buttons = Buttons { right: true, left: true, ..Buttons::default() };
    let mut renderer = MockRenderer::new();
    let mut audio = MockAudio::default();
    let out = host.tick(&platform.input, &mut renderer, &mut audio);
    assert_eq!(out.buttons, Buttons::default(), "Input→消解");

    // Renderer：size/resize 覆盖 + present 计数 + 画布尺寸
    assert_eq!(renderer.size(), (1280, 720), "平台 size 覆盖");
    renderer.resize(1920, 1080);
    assert_eq!(renderer.size(), (1920, 1080), "平台 resize 覆盖");
    assert_eq!(renderer.resizes, 1);
    assert_eq!(renderer.presents, 1);
    assert_eq!(renderer.canvas.len(), CANVAS_LEN);

    // Audio：空流也每帧 end_frame
    assert!(audio.writes.is_empty());
    assert_eq!(audio.frames, vec![0]);

    // Storage：保存/读取/损坏 fail-closed
    assert!(host.save_snapshot(2, &mut platform.storage));
    let mut restored = Host::new();
    assert!(restored.load_snapshot(2, &mut platform.storage), "快照往返");
    assert_eq!(restored.mode(), host.mode());
    platform.storage.slots[3] = Some(vec![0u8; 8]);
    assert!(!restored.load_snapshot(3, &mut platform.storage), "损坏快照 fail-closed");

    // AssetSource：present/缺失
    assert!(host.load_asset(&mut platform.assets, ROM_ASSET_PATH));
    assert!(host.rom().is_some());
    assert!(!host.load_asset(&mut platform.assets, "rom/missing.nes"));

    // Lifecycle：suspend/resume/quit 经宿主门控
    platform.lifecycle.events.push_back(AppEvent::Suspend);
    assert_eq!(host.poll_lifecycle(&mut platform.lifecycle), 1);
    let out = host.tick(&platform.input, &mut renderer, &mut audio);
    assert!(!out.advanced, "suspend 门控");
    platform.lifecycle.events.push_back(AppEvent::Resume);
    host.poll_lifecycle(&mut platform.lifecycle);
    let out = host.tick(&platform.input, &mut renderer, &mut audio);
    assert!(out.advanced, "resume 恢复");
    platform.lifecycle.events.push_back(AppEvent::Quit);
    host.poll_lifecycle(&mut platform.lifecycle);
    let out = host.tick(&platform.input, &mut renderer, &mut audio);
    assert!(!out.advanced && out.quit, "quit 停止");
}

#[test]
fn mock_events_only_use_proven_flow_edges() {
    // 未证边 fail-closed：首帧直接 MidCombat 必须 Err（宿主不在 Combat）。
    let mut host = Host::new();
    assert!(host.apply_event(ModeEvent::MidCombat).is_err());
    assert_eq!(host.mode(), Mode::Boot);
    // 已证事件序列首尾一致（44 条，与 300 帧脚本共用）
    assert_eq!(event_sequence().len(), 44);
}
