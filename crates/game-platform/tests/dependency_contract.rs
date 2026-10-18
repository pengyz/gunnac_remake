//! game-platform 零外部依赖 + 零平台代码（AGENTS.md §2 架构合同 + task-5 契约门禁）。
//!
//! 扫描范围＝本 crate 的 `src/`（去注释）与 `Cargo.toml` 的 `[dependencies]` 段；
//! 测试自身不在扫描面内（避免词面自触发）。

use std::fs;
use std::path::PathBuf;

/// 词面禁令（trait 缝出现即违规）：平台 crate 名 / 平台 I/O / 非确定性来源。
const FORBIDDEN_TOKENS: &[&str] = &[
    "winit",
    "softbuffer",
    "cpal",
    "sdl",
    "std::fs",
    "std::net",
    "std::process",
    "std::time",
    "tokio",
    "rand::",
    "getrandom",
];

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
    for entry in fs::read_dir(dir).expect("读取 game-platform/src 失败") {
        let path = entry.expect("读目录项失败").path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(path);
        }
    }
}

#[test]
fn game_platform_dependencies_section_is_empty() {
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
        if !code.is_empty() {
            entries.push(code.to_string());
        }
    }
    assert!(
        entries.is_empty(),
        "game-platform 的 [dependencies] 必须为空（trait 缝零外部依赖），实际: {entries:?}"
    );
}

#[test]
fn game_platform_source_has_no_platform_or_nondeterminism_tokens() {
    let mut files = Vec::new();
    collect_rs(&src_dir(), &mut files);
    assert!(!files.is_empty(), "game-platform/src 没有 .rs 文件（扫描面为空=门禁失效）");
    let mut hits = Vec::new();
    for f in &files {
        let text = strip_comments(&fs::read_to_string(f).expect("读取源文件失败"));
        for token in FORBIDDEN_TOKENS {
            if text.contains(token) {
                hits.push(format!("{} 含禁词 {:?}", f.display(), token));
            }
        }
    }
    assert!(hits.is_empty(), "game-platform 红线违规：\n{}", hits.join("\n"));
}
