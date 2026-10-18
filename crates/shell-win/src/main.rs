//! shell-win 可执行入口：winit 0.30 窗口 + softbuffer CPU 面 + game-shell 宿主 60Hz 主循环。
//!
//! 主循环（每采样点，platform-shell.md §4）：
//! 输入采样 → `game_shell::Host::advance(now_ns, …)`（固定步 60Hz）→ `Renderer::present`
//! （软缓冲合成 + 整数倍缩放/黑边上屏）。宿主返回的宿主状态/画布/音频写流全部真实消费。
//!
//! `--selftest-frames N`：无窗口确定性自检（见 `shell_win::run_selftest`），打印签名后正常退出。
//!
//! 未证域（AGENTS.md §3 / platform-shell.md §8）：菜单输入门控不自动派发（本主循环不投递
//! `ModeEvent`；状态推进接线属后续已证战役），敌/弹/HUD/文本不画，音效合成=静音占位显式。

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use game_platform::{AppEvent, Renderer, CANVAS_H, CANVAS_W};
use game_shell::{Host, FRAME_NS, ROM_ASSET_PATH};
use shell_win::{
    blit_scaled, FsAssets, FrameRenderer, KeyboardInput, LifecycleQueue, PcAudio, SelftestReport,
    WINDOW_TITLE,
};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

/// 命令行用法。
const USAGE: &str = "用法：shell-win [--selftest-frames N]\n  \
     （无参数=打开 PC 窗口运行；--selftest-frames N=无窗口确定性自检 N 帧后退出）";

/// 窗口逻辑渲染器：`FrameRenderer`（合成/计数）+ softbuffer 面（整数缩放/黑边上屏）。
struct WindowRenderer {
    core: FrameRenderer,
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
}

impl WindowRenderer {
    fn new() -> Self {
        Self { core: FrameRenderer::new(), window: None, surface: None }
    }

    fn has_window(&self) -> bool {
        self.window.is_some()
    }

    /// 窗口创建后挂接 softbuffer 面。
    fn attach(
        &mut self,
        window: Arc<Window>,
        surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    ) {
        self.window = Some(window);
        self.surface = Some(surface);
    }

    /// CHR 背景 ROM（尺寸不足⇒`false`，fail-closed 保持黑背景）。
    fn set_chr_rom(&mut self, chr_rom: &[u8]) -> bool {
        self.core.set_chr_rom(chr_rom)
    }

    /// 合成结果上屏（softbuffer blit；无窗口/零尺寸时 no-op）。
    fn blit(&mut self) {
        let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) else {
            return;
        };
        let size = window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return;
        };
        if let Err(err) = surface.resize(w, h) {
            eprintln!("shell-win: softbuffer resize 失败：{err}");
            return;
        }
        let Ok(mut buffer) = surface.buffer_mut() else {
            eprintln!("shell-win: softbuffer 取缓冲失败");
            return;
        };
        blit_scaled(self.core.composed_canvas(), &mut buffer, size.width, size.height);
        if let Err(err) = buffer.present() {
            eprintln!("shell-win: softbuffer present 失败：{err}");
        }
    }
}

impl Renderer for WindowRenderer {
    fn canvas_mut(&mut self) -> &mut [u8] {
        self.core.canvas_mut()
    }

    fn present(&mut self) {
        self.core.present();
        self.blit();
    }

    fn size(&self) -> (u32, u32) {
        match self.window.as_ref() {
            Some(window) => {
                let size = window.inner_size();
                (size.width, size.height)
            }
            None => self.core.size(),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.core.resize(width, height);
    }
}

/// PC 宿主应用：平台 trait 实现 + game-shell 宿主接线。
struct App {
    host: Host,
    input: KeyboardInput,
    renderer: WindowRenderer,
    audio: PcAudio,
    lifecycle: LifecycleQueue,
    assets: FsAssets,
    start: Instant,
    next_frame: Instant,
    asset_loaded: bool,
    /// 上一逻辑帧的 flow 模式（宿主状态真实消费：模式变化打印到 stderr，便于排障/验收）。
    last_mode: Option<game_core::flow::Mode>,
}

impl App {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            host: Host::new(),
            input: KeyboardInput::new(),
            renderer: WindowRenderer::new(),
            audio: PcAudio::new(),
            lifecycle: LifecycleQueue::new(),
            assets: FsAssets::repo(),
            start: now,
            next_frame: now,
            asset_loaded: false,
            last_mode: None,
        }
    }

    /// 经 `AssetSource` 装载 ROM 字节（一次）；失败 fail-closed（无 CHR 背景，仍可运行）。
    fn load_assets(&mut self) {
        if self.asset_loaded {
            return;
        }
        self.asset_loaded = true;
        if self.host.load_asset(&mut self.assets, ROM_ASSET_PATH) {
            let rom = self.host.rom().map(|r| r.to_vec());
            if let Some(rom) = rom {
                let ok = self.renderer.set_chr_rom(&rom);
                if !ok {
                    eprintln!("shell-win: CHR 背景不可用（ROM 尺寸不足），fail-closed 用黑背景");
                }
            }
        } else {
            eprintln!(
                "shell-win: 资产缺失 {ROM_ASSET_PATH}（根={:?}），CHR 背景 fail-closed 用黑背景",
                self.assets.root()
            );
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.lifecycle.push(AppEvent::Resume);
        if self.renderer.has_window() {
            return; // Android 恢复等重复 resumed：保留窗口/面
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(WINDOW_TITLE)
                        .with_inner_size(LogicalSize::new(
                            CANVAS_W as u32 * 2,
                            CANVAS_H as u32 * 2,
                        )),
                )
                .expect("shell-win: 创建窗口失败"),
        );
        let context = softbuffer::Context::new(window.clone()).expect("shell-win: softbuffer context");
        let surface = softbuffer::Surface::new(&context, window.clone())
            .expect("shell-win: softbuffer surface");
        self.renderer.attach(window, surface);
        // 固定步/计时基准从窗口就绪时刻起算（monotonic）。
        self.start = Instant::now();
        self.next_frame = self.start;
        self.load_assets();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.lifecycle.push(AppEvent::Suspend);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.lifecycle.push(AppEvent::Quit);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.renderer.resize(size.width, size.height);
            }
            WindowEvent::Focused(focused) => {
                // 平台策略占位（不承载游戏行为）：失焦挂起、回焦恢复。
                self.lifecycle.push(if focused {
                    AppEvent::Resume
                } else {
                    AppEvent::Suspend
                });
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                self.input.apply_code(code, state == ElementState::Pressed);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // 生命周期事件 → 宿主门控（Suspend 停逻辑帧/呈现；Quit 退出）。
        self.host.poll_lifecycle(&mut self.lifecycle);
        if self.host.quit_requested() {
            event_loop.exit();
            return;
        }
        if !self.renderer.has_window() {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        // 60Hz 固定步：每帧仅喂整数 ns，折算/追帧上限由 game-shell `FixedStep` 决定。
        let now = Instant::now();
        if now < self.next_frame {
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
            return;
        }
        while now >= self.next_frame {
            self.next_frame += Duration::from_nanos(FRAME_NS);
        }
        let now_ns = now.duration_since(self.start).as_nanos() as u64;
        // 只经 game-shell 宿主驱动 game-core：输入采样→tick→（宿主内部）音频消费/渲染合成/present。
        let frames = self
            .host
            .advance(now_ns, &self.input, &mut self.renderer, &mut self.audio);
        if frames > 0 {
            // 宿主状态真实消费：模式/关号/帧号变化打印（排障与验收观测面；不承载游戏行为）。
            let mode = self.host.mode();
            if self.last_mode != Some(mode) {
                self.last_mode = Some(mode);
                eprintln!(
                    "shell-win: mode={mode:?} stage={} frame={}",
                    self.host.stage_index(),
                    self.host.frame()
                );
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
    }
}

/// 打印自检报告（两次运行输出必须逐行一致）。
fn print_selftest(report: &SelftestReport) {
    println!(
        "selftest: frames={} advanced={} presents={}",
        report.frames, report.advanced, report.presents
    );
    println!(
        "selftest: mode={:?} stage={} frame={}",
        report.mode, report.stage, report.frame
    );
    println!("selftest: host_signature=0x{:016x}", report.host_signature);
    println!("selftest: canvas_signature=0x{:016x}", report.canvas_signature);
    println!("selftest: display_signature=0x{:016x}", report.display_signature);
    println!(
        "selftest: audio_writes={} audio_frames={}",
        report.audio_writes, report.audio_frames
    );
    println!("selftest: rom_len={:?}", report.rom_len);
    println!("selftest: OK");
}

/// 解析 `--selftest-frames N`（未给出⇒`Ok(None)`；非法参数⇒`Err`）。
fn parse_selftest_frames(args: &[String]) -> Result<Option<u64>, String> {
    let mut frames = None;
    let mut i = 0usize;
    while i < args.len() {
        let arg = &args[i];
        if let Some(rest) = arg.strip_prefix("--selftest-frames=") {
            frames = Some(parse_frames(rest)?);
        } else if arg == "--selftest-frames" {
            let Some(value) = args.get(i + 1) else {
                return Err("--selftest-frames 需要帧数 N".to_string());
            };
            frames = Some(parse_frames(value)?);
            i += 1;
        } else {
            return Err(format!("未知参数：{arg}"));
        }
        i += 1;
    }
    Ok(frames)
}

fn parse_frames(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("--selftest-frames 需要非负整数，收到：{value:?}"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    match parse_selftest_frames(&args) {
        Ok(Some(frames)) => {
            let report = shell_win::run_selftest(frames);
            print_selftest(&report);
            return Ok(());
        }
        Ok(None) => {}
        Err(err) => {
            eprintln!("shell-win: {err}\n{USAGE}");
            std::process::exit(2);
        }
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
