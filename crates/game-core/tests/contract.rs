//! 红线机器把关（AGENTS.md §2）：game-core 禁平台依赖、禁真实时钟、禁线程 RNG。
//!
//! 扫描范围 = game-core 的 `src/`（本测试自身不在扫描面内，避免词面自触发）与 Cargo.toml。
//! 命中即 FAIL：任何"顺手"引入的非确定性/平台依赖会在 `cargo test` 与 rust-tests 门禁被拦。

use std::fs;
use std::path::PathBuf;

/// 词面禁令：出现即违规（宁严勿漏；新增合法 API 时需在 commit 中写明理由）。
const FORBIDDEN_TOKENS: &[&str] = &[
    "Instant::now",
    "SystemTime",
    "std::time",
    "thread_rng",
    "rand::",
    "getrandom",
    "std::fs",
    "std::net",
    "std::process",
    "tokio",
];

/// 依赖禁令：game-core 的 Cargo.toml 里出现即违规（唯一允许 game-platform）。
const FORBIDDEN_DEPS: &[&str] = &[
    "winit", "sdl", "glutin", "tokio", "rand", "getrandom", "chrono", "instant",
];

/// 去掉行注释与块注释后再扫描：红线约束的是**代码**，文档里讨论禁词不受限。
/// （朴素实现足够：game-core 源码不含字符串字面量里的 "//" 或 "/*"。）
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

fn collect_rs(dir: PathBuf, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(&dir).expect("读取 game-core/src 失败") {
        let path = entry.expect("读目录项失败").path();
        if path.is_dir() {
            collect_rs(path, out);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(path);
        }
    }
}

#[test]
fn game_core_source_has_no_platform_or_nondeterminism_tokens() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs(src, &mut files);
    assert!(!files.is_empty(), "game-core/src 没有 .rs 文件（扫描面为空=门禁失效）");
    let mut hits = Vec::new();
    for f in &files {
        let text = strip_comments(&fs::read_to_string(f).expect("读取源文件失败"));
        for token in FORBIDDEN_TOKENS {
            if text.contains(token) {
                hits.push(format!("{} 含禁词 {:?}", f.display(), token));
            }
        }
    }
    assert!(hits.is_empty(), "game-core 红线违规：\n{}", hits.join("\n"));
}

#[test]
fn game_core_deps_only_game_platform() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = fs::read_to_string(&manifest).expect("读取 Cargo.toml 失败");
    for dep in FORBIDDEN_DEPS {
        assert!(
            !text.contains(dep),
            "game-core Cargo.toml 含禁依赖 {:?}（AGENTS.md §2：禁止平台依赖）",
            dep
        );
    }
}

#[test]
fn frame_clock_is_deterministic_and_advances_only_by_tick() {
    use game_core::{FrameClock, TARGET_FPS};
    use game_platform::Clock;

    let mut a = FrameClock::new();
    let mut b = FrameClock::new();
    for _ in 0..TARGET_FPS {
        a.tick();
    }
    for _ in 0..TARGET_FPS {
        b.tick();
    }
    assert_eq!(a.frame(), TARGET_FPS);
    assert_eq!(a.now_ms(), 1000);
    assert_eq!(a, b, "同驱动序列必须产生同一时钟状态（确定性）");
}

#[test]
fn lcg_rng_is_seed_deterministic() {
    use game_core::LcgRng;
    use game_platform::GameRng;

    let mut a = LcgRng::seed(0x1234);
    let mut b = LcgRng::seed(0x1234);
    let seq_a: Vec<u32> = (0..8).map(|_| a.next_u32()).collect();
    let seq_b: Vec<u32> = (0..8).map(|_| b.next_u32()).collect();
    assert_eq!(seq_a, seq_b, "同种子必须同序列（差分可复现根基）");
}
