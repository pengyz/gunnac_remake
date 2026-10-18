//! shell/win（platform-shell.md §3——Windows/Linux 同源跨平台壳）。
//!
//! 五件：窗口（winit+softbuffer）/60Hz 帧驱动/呈现（blit 整数缩放）/
//! 输入映射（Z/X/Enter/RShift/方向→Buttons）/音频（APU 合成模型+跨平台面）。
//! OS 差异 cfg(windows)/cfg(unix)=存储路径/音频后端（差异实现非分叉）。

/// 60Hz 帧间隔（毫秒——确定性 60Hz）。
pub const FRAME_MS: u64 = 16; // 1000/60≈16.67——帧驱动节流基

/// 输入映射（platform-shell.md §3：Z/X/Enter/RShift/方向→Buttons）。
/// 键码→按钮（winit 键盘面——虚拟键码映射表）。
pub fn map_key(key: &str) -> Option<game_platform::Buttons> {
    use game_platform::Buttons;
    let mut b = Buttons::default();
    match key {
        "z" | "Z" => { b.a = true; Some(b) }
        "x" | "X" => { b.b = true; Some(b) }
        "Enter" => { b.start = true; Some(b) }
        "ShiftRight" | "RShift" => { b.select = true; Some(b) }
        "ArrowUp" => { b.up = true; Some(b) }
        "ArrowDown" => { b.down = true; Some(b) }
        "ArrowLeft" => { b.left = true; Some(b) }
        "ArrowRight" => { b.right = true; Some(b) }
        _ => None,
    }
}

/// 存储目录解析（cfg 差异：Windows=%APPDATA%/gunnac/；Linux=~/.local/share/gunnac/）。
pub fn storage_dir() -> std::path::PathBuf {
    #[cfg(windows)]
    {
        std::env::var("APPDATA")
            .map(|d| std::path::PathBuf::from(d).join("gunnac"))
            .unwrap_or_else(|_| std::path::PathBuf::from("gunnac"))
    }
    #[cfg(unix)]
    {
        std::env::var("HOME")
            .map(|d| std::path::PathBuf::from(d).join(".local/share/gunnac"))
            .unwrap_or_else(|_| std::path::PathBuf::from("gunnac"))
    }
    #[cfg(not(any(windows, unix)))]
    {
        std::path::PathBuf::from("gunnac")
    }
}
