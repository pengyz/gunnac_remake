//! shell/win 冒烟自测（platform-shell.md §3 五件）。

/// 输入映射（Z/X/Enter/RShift/方向→Buttons）。
#[test]
fn input_mapping() {
    use shell_win::map_key;
    assert!(map_key("z").map(|b| b.a).unwrap_or(false), "Z→A");
    assert!(map_key("x").map(|b| b.b).unwrap_or(false), "X→B");
    assert!(map_key("Enter").map(|b| b.start).unwrap_or(false), "Enter→START");
    assert!(map_key("ShiftRight").map(|b| b.select).unwrap_or(false), "RShift→SELECT");
    assert!(map_key("ArrowUp").map(|b| b.up).unwrap_or(false), "↑");
    assert!(map_key("ArrowLeft").map(|b| b.left).unwrap_or(false), "←");
    assert!(map_key("q").is_none(), "未映射键=None");
}

/// 存储路径（cfg 差异——目录名 gunnac）。
#[test]
fn storage_path() {
    let p = shell_win::storage_dir();
    assert!(p.to_string_lossy().contains("gunnac"), "存储目录含 gunnac");
}

/// 60Hz 帧间隔（16ms 节流基）。
#[test]
fn frame_interval() {
    assert_eq!(shell_win::FRAME_MS, 16, "60Hz 帧间隔");
}
