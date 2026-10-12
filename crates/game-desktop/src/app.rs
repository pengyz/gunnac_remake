//! winit 应用体（窗口/输入/60Hz 帧驱动）。

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use game_core::player::{self, PlayerState};
use game_core::FrameClock;
use game_data::chr;
use game_desktop::{blit_scaled, held_from_keys, view_rect, Canvas, GREY4, SCREEN_H, SCREEN_W};

const FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);

pub struct App {
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    canvas: Canvas,
    clock: FrameClock,
    state: PlayerState,
    keys: KeyState,
    rom: Vec<u8>,
    next_frame: Instant,
}

#[derive(Default)]
struct KeyState {
    right: bool, left: bool, down: bool, up: bool, a: bool, b: bool,
}

impl App {
    pub fn new() -> Self {
        let rom = std::fs::read("rom/gunnac_cn.nes").unwrap_or_default();
        App {
            window: None,
            surface: None,
            canvas: Canvas::new(),
            clock: FrameClock::new(),
            state: PlayerState { x: 128, y: 160, x_subpix: 0, y_subpix: 0, bomb_ammo: 4, action_mode: 0 },
            keys: KeyState::default(),
            rom,
            next_frame: Instant::now(),
        }
    }

    fn draw(&mut self) {
        // 游戏状态推进（game-core 纯逻辑，60Hz 一 tick）
        let held = held_from_keys(
            self.keys.right, self.keys.left, self.keys.down, self.keys.up,
            self.keys.a, self.keys.b,
        );
        self.state = player::tick(&self.state, held);
        self.clock.tick();

        // 合成：清屏 + CHR 字模区 tile 铺排（可见性骨架）+ 玩家 8×8 sprite
        self.canvas.clear([0, 0, 0, 255]);
        if self.rom.len() > chr::CHR_FILE_BASE + 128 * chr::CHR_BANK_SIZE {
            for ty in 0..(SCREEN_H / 8) {
                for tx in 0..(SCREEN_W / 8) {
                    let tile = ((tx + ty * 3) % chr::CHR_TILES_PER_BANK) as usize;
                    self.canvas.draw_chr(&self.rom, chr::FONT_BANK_START, tile, (tx * 8) as i32, (ty * 8) as i32, &GREY4);
                }
            }
        }
        // 玩家 sprite：NES 8×8 语义（tile 图样未证——占位 tile 索引参数化，禁发明）
        self.canvas.draw_chr(&self.rom, chr::FONT_BANK_START, 2, self.state.x as i32, self.state.y as i32, &GREY4);

        // 呈现：整数倍缩放合成到窗口缓冲
        if let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) {
            let size = window.inner_size();
            if size.width == 0 || size.height == 0 {
                return;
            }
            surface.resize(
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            ).expect("surface resize");
            let mut buffer = surface.buffer_mut().expect("surface buffer");
            blit_scaled(&mut buffer, size.width, size.height, &self.canvas);
            buffer.present().expect("present");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (ox, oy, w, h) = view_rect(SCREEN_W as u32 * 2, SCREEN_H as u32 * 2);
        let _ = (ox, oy);
        let window = Arc::new(event_loop.create_window(
            Window::default_attributes()
                .with_title("gunnac-remake (Stage A)")
                .with_inner_size(winit::dpi::LogicalSize::new(w, h)),
        ).expect("窗口创建"));
        let context = softbuffer::Context::new(window.clone()).expect("softbuffer context");
        let surface = softbuffer::Surface::new(&context, window.clone()).expect("softbuffer surface");
        self.window = Some(window);
        self.surface = Some(surface);
        self.next_frame = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(code), state, .. }, .. } => {
                let pressed = state == ElementState::Pressed;
                match code {
                    KeyCode::ArrowRight | KeyCode::KeyD => self.keys.right = pressed,
                    KeyCode::ArrowLeft | KeyCode::KeyA => self.keys.left = pressed,
                    KeyCode::ArrowDown | KeyCode::KeyS => self.keys.down = pressed,
                    KeyCode::ArrowUp | KeyCode::KeyW => self.keys.up = pressed,
                    KeyCode::KeyZ | KeyCode::KeyK => self.keys.a = pressed,
                    KeyCode::KeyX | KeyCode::KeyJ => self.keys.b = pressed,
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // 60Hz 帧驱动：虚拟时钟推进 + 等到下一帧（确定性节拍；渲染为 I/O 观测面）
        let now = Instant::now();
        if now >= self.next_frame {
            self.next_frame += FRAME;
            if let Some(w) = self.window.as_ref() {
                w.request_redraw();
            }
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(self.next_frame));
        }
    }
}
