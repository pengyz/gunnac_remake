//! game-desktop 主循环：winit 窗口 + softbuffer 软件帧缓冲 + game-core 60Hz tick 集成。
//!
//! 可见性骨架（t58）：CHR tile 背景 + 玩家 sprite（NES 8×8 语义，tile 参数化——
//! 精灵图样/调色板映射未证，不发明）+ 逻辑画布整数倍缩放输出。
//! 时间只经 game-core::FrameClock（虚拟时钟，AGENTS.md §2）；平台只喂输入/呈现。

mod app;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
        std::time::Instant::now(),
    ));
    let mut app = app::App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
