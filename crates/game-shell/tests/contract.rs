//! game-shell 零平台依赖契约（AGENTS.md §2 + task-5 契约门禁）。
//!
//! 扫描范围＝本 crate 的 `src/`（去注释）与 `Cargo.toml` 的 `[dependencies]` 段。
//! 判据：宿主层不得出现窗口/音频设备/文件系统/网络/进程/平台时间等平台依赖；
//! 读资产必须走 `game_platform::AssetSource`，时间只允许整数纳秒注入或 game-core 虚拟时钟。

use std::fs;
use std::path::PathBuf;

/// 词面禁令（出现即违规；宁严勿漏，新增合法 API 须在 commit 说明理由）。
const FORBIDDEN_TOKENS: &[&str] = &[
    // 窗口/呈现/音频设备
    "winit",
    "softbuffer",
    "cpal",
    "sdl",
    "oboe",
    "aaudio",
    "jni",
    "android",
    // I/O 与平台服务
    "std::fs",
    "std::net",
    "std::process",
    "std::env",
    // 非确定性/平台时间
    "std::time",
    "Instant::now",
    "SystemTime",
    "thread_rng",
    "rand::",
    "getrandom",
    "tokio",
];

/// 允许的依赖（路径依赖；仅纯逻辑/纯契约 crate）。
const ALLOWED_DEPS: &[&str] = &["game-core", "game-platform"];

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

fn src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// 去行注释/块注释（红线约束代码；doc 注释里讨论平台词不受限）。
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            while let Some(c2) = chars.next() {
                if c2 == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = '\0';
            for c2 in chars.by_ref() {
                if prev == '*' && c2 == '/' {
                    break;
                }
                prev = c2;
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn collect_rs(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("读取 game-shell/src 失败") {
        let path = entry.expect("读目录项失败").path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(path);
        }
    }
}

#[test]
fn game_shell_source_has_no_platform_dependency_tokens() {
    let mut files = Vec::new();
    collect_rs(&src_dir(), &mut files);
    assert!(!files.is_empty(), "game-shell/src 没有 .rs 文件（扫描面为空=门禁失效）");
    let mut hits = Vec::new();
    for f in &files {
        let text = strip_comments(&fs::read_to_string(f).expect("读取源文件失败"));
        for token in FORBIDDEN_TOKENS {
            if text.contains(token) {
                hits.push(format!("{} 含禁词 {:?}", f.display(), token));
            }
        }
    }
    assert!(hits.is_empty(), "game-shell 平台依赖违规：\n{}", hits.join("\n"));
}

#[test]
fn game_shell_dependencies_are_pure_logic_only() {
    let text = fs::read_to_string(manifest_path()).expect("读取 Cargo.toml 失败");
    let mut in_deps = false;
    let mut entries = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_deps = t == "[dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        let code = t.split('#').next().unwrap_or("").trim();
        if code.is_empty() {
            continue;
        }
        let name = code.split('=').next().unwrap_or("").trim();
        entries.push(name.to_string());
    }
    assert!(!entries.is_empty(), "[dependencies] 段为空⇒扫描面失效（应至少有 game-core/game-platform）");
    for name in &entries {
        assert!(
            ALLOWED_DEPS.contains(&name.as_str()),
            "game-shell 只允许纯逻辑/纯契约依赖 {ALLOWED_DEPS:?}，发现 {:?}",
            name
        );
    }
}
