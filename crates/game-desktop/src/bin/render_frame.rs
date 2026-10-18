//! render_frame —— L4 对拍用 Rust 侧帧导出（t93：nametable 持久化模型 → BG 渲染）。
//! 用法：render-frame <帧号> <输出名> [场景=gameplay-stage1] [--base <0|1>]
//!          [--out-dir <目录>] [--log-path <文件>]（task-94：P0-0-2 输出参数化，默认行为不变）
//!
//! 模型（bank15:$F040-F0DA 定谳）：逐帧 nmi_flush（$130-$14F 行缓冲 → $8F:$8E 行、
//! $D3-$DA → 属性行、$160=挂起计数）累积 nametable；目标帧按 $1C/$1D 卷轴渲染 32×30。
//! BG 图案表基址（PPUCTRL bit4）未证——--base 0|1 双候选由 L4 对拍裁定（③）。
//! `--out-dir`/`--log-path` 显式指定时须落在 repo_root 内（越界 exit 2）；缺失 ROM/trace exit 2。

use game_core::terrain::Nametable;
use game_desktop::{Canvas, SCREEN_H, SCREEN_W};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};

// ===== CLI / 输出参数化（task-94，P0-0-2；规格=l4-baseline-freeze-plan §4.1）=====

/// 用法（`--help`）。
pub const USAGE: &str = "\
用法：render-frame <帧号> <输出名> [场景=gameplay-stage1] [--base <0|1>]
                   [--out-dir <目录>] [--log-path <文件>]
                   [--probe-file <文件>] [--stream-file <文件>] [--text-file <文件>]

--out-dir   actual 输出根（png+idx 写此处；缺省=<repo_root>/oracle/screens/actual）
--log-path  渲染日志（缺省=<out_dir>/.render.log；行含 scene/env/argv/输入 sha/输出 sha）
显式 --out-dir/--log-path 必须落在仓库根（repo_root）内，越界 fail-closed（退出码 2）。
缺失输入（ROM/trace）⇒ 显式报错退出码 2。";

/// 默认场景名（历史行为）。
pub const DEFAULT_SCENE: &str = "gameplay-stage1";

/// 解析后的 CLI（**默认值与历史行为一致**；`--out-dir`/`--log-path` 为 task-94 新增）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub frame: u64,
    pub name: String,
    pub scene: String,
    pub base: usize,
    /// `--out-dir` 原样字符串（缺省 None ⇒ `<repo_root>/oracle/screens/actual`）。
    pub out_dir: Option<String>,
    /// `--log-path` 原样字符串（缺省 None ⇒ `<out_dir>/.render.log`）。
    pub log_path: Option<String>,
    /// 旧可选参数（F97-3：解析进 CLI，避免把其值误当位置参数；主流程仍按原逻辑重扫 args 使用）。
    pub probe_file: Option<String>,
    pub stream_file: Option<String>,
    pub text_file: Option<String>,
}

/// `--help`/`-h` 是否出现在任意位置（含位置参数之后；F97-4）。
pub fn wants_help(args: &[String]) -> bool {
    args.iter().any(|a| a == "--help" || a == "-h")
}

/// 取 `--flag` 后随值（缺失⇒None；不做越界/存在校验——校验在 resolve 阶段）。
fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
}

/// 旧带值可选 flag（必须连同值一起跳过；否则值会被误当位置参数——F97-3）。
const LEGACY_VALUE_FLAGS: &[&str] = &["--probe-file", "--stream-file", "--text-file"];

/// 解析 CLI（纯函数；可单测）。缺失/非法参数 ⇒ Err（main 侧 eprintln + exit 2）。
pub fn parse_cli(args: &[String]) -> Result<Cli, String> {
    let mut positional: Vec<&String> = Vec::new();
    let mut i = 1usize; // 跳过 argv[0]
    while i < args.len() {
        let a = &args[i];
        if a == "--out-dir" || a == "--log-path" || a == "--base" {
            if args.get(i + 1).is_none() {
                return Err(format!("{a} 需要参数值"));
            }
            i += 2;
            continue;
        }
        if LEGACY_VALUE_FLAGS.contains(&a.as_str()) {
            if let Some(v) = args.get(i + 1) {
                // 值必须跳过（不参与位置参数）；旧实现缺失值时按缺省处理 ⇒ 保持宽容
                let _ = v;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if a.starts_with("--") {
            i += 1; // 其余旧开关（无值）
            continue;
        }
        positional.push(a);
        i += 1;
    }
    let frame_raw = positional.first().ok_or("缺失帧号（第 1 个位置参数）")?;
    let frame: u64 = frame_raw.parse().map_err(|_| format!("帧号非法：{frame_raw}"))?;
    let name = positional
        .get(1)
        .map(|s| (*s).clone())
        .ok_or("缺失输出名（第 2 个位置参数）")?;
    let scene = positional
        .get(2)
        .map(|s| (*s).clone())
        .unwrap_or_else(|| DEFAULT_SCENE.into());
    let base: usize = match arg_value(args, "--base") {
        Some(v) => v.parse().map_err(|_| format!("--base 非法：{v}"))?,
        None => 0,
    };
    Ok(Cli {
        frame,
        name,
        scene,
        base,
        out_dir: arg_value(args, "--out-dir"),
        log_path: arg_value(args, "--log-path"),
        probe_file: arg_value(args, "--probe-file"),
        stream_file: arg_value(args, "--stream-file"),
        text_file: arg_value(args, "--text-file"),
    })
}

/// 去除 Windows `\\?\` 扩展长度前缀（比较/显示用；写入仍可用）。
pub fn strip_verbatim(p: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let s = p.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            if !rest.starts_with("UNC\\") {
                return PathBuf::from(rest);
            }
        }
    }
    p.to_path_buf()
}

/// 真实路径解析（**触及文件系统**）：存在则 canonicalize；不存在则 canonicalize 最近存在祖先
/// 再把剩余组件词法拼回（用于尚未创建的输出目录）。错误 ⇒ Err（fail-closed）。
pub fn real_path(p: &Path) -> Result<PathBuf, String> {
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("无法取当前目录：{e}"))?
            .join(p)
    };
    let mut existing = abs.clone();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    loop {
        match std::fs::canonicalize(&existing) {
            Ok(c) => {
                let mut out = strip_verbatim(&c);
                for seg in tail.iter().rev() {
                    out.push(seg);
                }
                return Ok(out);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let Some(name) = existing.file_name().map(|s| s.to_os_string()) else {
                    return Err(format!("无法解析路径 {}：{e}", abs.display()));
                };
                tail.push(name);
                if !existing.pop() {
                    return Err(format!("无法解析路径 {}：{e}", abs.display()));
                }
            }
            Err(e) => return Err(format!("无法解析路径 {}：{e}", existing.display())),
        }
    }
}

/// 词法规范化（不触盘）：解析 `.`/`..`（不越过根）。
pub fn lexical_normalize(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Prefix(_) | Component::RootDir => out.push(c.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            Component::Normal(x) => out.push(x),
        }
    }
    out
}

/// 路径比较归一（Windows 大小写不敏感；POSIX 大小写敏感——F97-5）。
fn norm_compare(p: &Path) -> String {
    let s = strip_verbatim(p).to_string_lossy().replace('\\', "/").trim_end_matches('/').to_string();
    #[cfg(windows)]
    {
        s.to_lowercase()
    }
    #[cfg(not(windows))]
    {
        s
    }
}

/// 是否位于仓库根内（按给定路径比较；真实路径 containment 见 [`resolve_explicit`]/[`ensure_real_within`]）。
pub fn within_repo(repo_root: &Path, p: &Path) -> bool {
    let root = norm_compare(repo_root);
    let path = norm_compare(p);
    path == root || path.starts_with(&(root.clone() + "/"))
}

/// 解析显式输出路径：相对路径按当前目录解释 → 词法规范化 → **真实路径**（canonicalize 最近存在
/// 祖先再拼）→ 必须落在 repo_root 真实路径内；越界 ⇒ Err（fail-closed，main 侧 exit 2）。
///
/// 真实路径解析可识别 junction/symlink 逃逸（F97-1）；返回真实路径用于写入。
pub fn resolve_explicit(repo_root: &Path, value: &str, flag: &str) -> Result<PathBuf, String> {
    let raw = PathBuf::from(value);
    let abs = if raw.is_absolute() {
        raw
    } else {
        let cwd = std::env::current_dir().map_err(|e| format!("{flag}：无法取当前目录：{e}"))?;
        cwd.join(raw)
    };
    let p = lexical_normalize(&abs);
    let root_real = real_path(repo_root).map_err(|e| format!("{flag}：仓库根真实路径解析失败：{e}"))?;
    let p_real = real_path(&p).map_err(|e| format!("{flag}：路径真实解析失败：{e}"))?;
    if !within_repo(&root_real, &p_real) {
        return Err(format!(
            "{flag} 越界（真实路径在仓库外）：{}（仓库根 {}）",
            p_real.display(),
            root_real.display()
        ));
    }
    Ok(p_real)
}

/// 创建后再校验真实路径仍位于 repo_root 内（防竞态/父级 junction；F97-1 二次校验）。
pub fn ensure_real_within(repo_root: &Path, p: &Path, flag: &str) -> Result<(), String> {
    let root_real = real_path(repo_root).map_err(|e| format!("{flag}：仓库根真实路径解析失败：{e}"))?;
    let p_real = real_path(p).map_err(|e| format!("{flag}：路径真实解析失败：{e}"))?;
    if !within_repo(&root_real, &p_real) {
        return Err(format!(
            "{flag} 越界（创建后真实路径在仓库外）：{}（仓库根 {}）",
            p_real.display(),
            root_real.display()
        ));
    }
    Ok(())
}

/// 记录用 env 白名单（render_frame 显式 knobs；不含 REPO_ROOT=输入定位不属渲染参数）。
pub const ENV_KNOBS: &[&str] = &[
    "BOTH", "COLOFF", "COMP", "DBGPROBE", "DBGSTREAM", "FAMMAP", "FAMROW", "FLAGUPLOAD",
    "FLIPOFF", "FUSE", "FUSE2", "GOLDPAL", "GRPSEL", "LOOPABL", "NOHUDLUT", "NOOWF", "PALROW",
    "PROJ2007", "QBASE", "QUAD", "R4B", "R4BOFF", "R4BREV", "RECP3", "RECPAL", "RENDER_NT",
    "REVORD", "ROWDBL", "ROWGOLD", "ROWMAP", "ROWOFF", "ROWWIN", "SLOTEND", "SPRABL", "TBLDRV",
    "TEXTABL", "TEXTFALLBACK", "TRIHOP", "WHITE13",
];

/// 显式设置的 env 摘要（`K=V` 按 K 排序；空⇒`{}`）。
pub fn env_explicit_summary() -> String {
    let mut kv: Vec<String> = ENV_KNOBS
        .iter()
        .filter_map(|k| std::env::var_os(k).map(|v| (*k, v)))
        .map(|(k, v)| format!("{k}={}", v.to_string_lossy()))
        .collect();
    kv.sort();
    format!("{{{}}}", kv.join(","))
}

/// SHA-256 流式累加器（task-138：大 text 文件不得整读；输出与整读实现逐字节一致）。
pub struct Sha256 {
    h: [u32; 8],
    total: u64,
    buf: Vec<u8>,
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            h: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
                0x1f83d9ab, 0x5be0cd19,
            ],
            total: 0,
            buf: Vec::with_capacity(128),
        }
    }

    fn compress(h: &mut [u32; 8], chunk: &[u8; 64]) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut w = [0u32; 64];
        for (i, b) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(v);
        }
    }

    /// 追加任意长度字节（内部按 64B 块累积；不保存全文）。
    pub fn update(&mut self, mut data: &[u8]) {
        self.total = self.total.wrapping_add(data.len() as u64);
        if !self.buf.is_empty() {
            let need = 64 - self.buf.len();
            let take = need.min(data.len());
            self.buf.extend_from_slice(&data[..take]);
            data = &data[take..];
            if self.buf.len() == 64 {
                let block: [u8; 64] = self.buf[..].try_into().expect("64B");
                Self::compress(&mut self.h, &block);
                self.buf.clear();
            }
        }
        while data.len() >= 64 {
            let block: [u8; 64] = data[..64].try_into().expect("64B");
            Self::compress(&mut self.h, &block);
            data = &data[64..];
        }
        self.buf.extend_from_slice(data);
    }

    pub fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.total.wrapping_mul(8);
        self.buf.push(0x80);
        while self.buf.len() % 64 != 56 {
            self.buf.push(0);
        }
        let tail = self.buf.clone();
        self.buf.clear();
        let mut padded = tail;
        padded.extend_from_slice(&bit_len.to_be_bytes());
        for block in padded.chunks_exact(64) {
            let b: [u8; 64] = block.try_into().expect("64B");
            Self::compress(&mut self.h, &b);
        }
        let mut out = [0u8; 32];
        for (i, v) in self.h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
        }
        out
    }
}

fn hex12(digest: [u8; 32]) -> String {
    digest.iter().take(6).map(|b| format!("{b:02x}")).collect()
}

/// SHA-256（一次性字节输入；P0-0 manifest 用 `sha12`=前 12 hex）。
pub fn sha256_hex12(bytes: &[u8]) -> String {
    let mut s = Sha256::new();
    s.update(bytes);
    hex12(s.finalize())
}

/// 文件 sha256 前 12 hex（**流式 1MB 块**，大 text 文件 RAM 有界；sha 与整读一致）。
pub fn sha256_file_hex12(path: &Path) -> Option<String> {
    let f = std::fs::File::open(path).ok()?;
    let mut rdr = std::io::BufReader::with_capacity(1 << 20, f);
    let mut s = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        match rdr.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => s.update(&buf[..n]),
            Err(_) => return None,
        }
    }
    Some(hex12(s.finalize()))
}

/// text 写集流式读取统计（task-138；日志 `in_frames_read=/text_records_read=/early_exit=`）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextReadStats {
    /// 已读记录的最高帧（含触发早退的那条）。
    pub in_frames_read: u32,
    /// 已解析记录数（含触发早退的那条）。
    pub text_records_read: u64,
    /// 是否因 `f > limit` 提前停止读取。
    pub early_exit: bool,
}

/// 流式读取 text 写集并按 `limit`（目标帧+margin）早退；**与整读+稳定排序前缀等价**。
///
/// 前提（A 级）：生产者 `phase_batch.rs`（L28-33/L70-78）按 trace 帧序输出 ⇒ 文件帧号单调不减。
/// 解析失败行与旧实现一样静默跳过；打开失败 ⇒ 空集（保持旧 `read_to_string().unwrap_or_default()` 语义）。
pub fn read_text_writes(path: &str, limit: u32) -> (Vec<(u32, u16, u8, u32, u32)>, TextReadStats) {
    let mut raw: Vec<(u32, u16, u8, u32, u32)> = Vec::new();
    let mut stats = TextReadStats::default();
    if path.is_empty() {
        return (raw, stats);
    }
    match std::fs::File::open(path) {
        Ok(fh) => {
            let reader = std::io::BufReader::with_capacity(1 << 20, fh);
            for line in reader.lines() {
                let Ok(l) = line else { break };
                let parsed = (|| -> Option<(u32, u16, u8, u32, u32)> {
                    let f = l.split("\"f\":").nth(1)?.split(',').next()?.parse().ok()?;
                    let a = u16::from_str_radix(l.split("\"addr\":\"").nth(1)?.split('"').next()?, 16)
                        .ok()?;
                    let v = l.split("\"val\":").nth(1)?.split(&[',', '}'][..]).next()?.parse().ok()?;
                    let inv = l.split("\"inv\":").nth(1)?.split(&[',', '}'][..]).next()?.parse().ok()?;
                    let seg = l.split("\"seg\":").nth(1)?.split(&[',', '}'][..]).next()?.parse().ok()?;
                    Some((f, a, v, inv, seg))
                })();
                if let Some(rec) = parsed {
                    stats.text_records_read = stats.text_records_read.saturating_add(1);
                    stats.in_frames_read = stats.in_frames_read.max(rec.0);
                    if rec.0 > limit {
                        stats.early_exit = true;
                        break;
                    }
                    raw.push(rec);
                }
            }
        }
        Err(e) => eprintln!("WARN text write set 打开失败（{}）：{e}；按空集处理", path),
    }
    (raw, stats)
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

/// NES 2C02 调色板 RGB（64 色标准表——EC46 色码域映射）。
fn nes_rgb(code: u8) -> game_desktop::Rgba {
    // 标准 NES 调色板（简化 2C02——逐色码 RGB）
    const P: [[u8; 3]; 64] = [
        [0x66,0x66,0x66],[0x00,0x2A,0x88],[0x14,0x12,0xA7],[0x3B,0x00,0xA4],[0x5C,0x00,0x7E],[0x6E,0x00,0x40],[0x6C,0x06,0x00],[0x56,0x1D,0x00],
        [0x33,0x35,0x00],[0x0B,0x48,0x00],[0x00,0x52,0x00],[0x00,0x4F,0x08],[0x00,0x40,0x4D],[0x00,0x00,0x00],[0x00,0x00,0x00],[0x00,0x00,0x00],
        [0xAD,0xAD,0xAD],[0x15,0x5F,0xD9],[0x42,0x40,0xFF],[0x75,0x27,0xFE],[0xA0,0x1A,0xCC],[0xB7,0x1E,0x7B],[0xB5,0x31,0x20],[0x99,0x4E,0x00],
        [0x6B,0x6D,0x00],[0x38,0x87,0x00],[0x0C,0x93,0x00],[0x00,0x8F,0x32],[0x00,0x7C,0x8D],[0x00,0x00,0x00],[0x00,0x00,0x00],[0x00,0x00,0x00],
        [0xFF,0xFE,0xFF],[0x64,0xB0,0xFF],[0x92,0x90,0xFF],[0xC6,0x76,0xFF],[0xF3,0x6A,0xFF],[0xFE,0x6E,0xCC],[0xFE,0x81,0x70],[0xEA,0x9E,0x22],
        [0xBC,0xBE,0x00],[0x88,0xD8,0x00],[0x5C,0xE4,0x30],[0x45,0xE0,0x82],[0x48,0xCD,0xDE],[0x4F,0x4F,0x4F],[0x00,0x00,0x00],[0x00,0x00,0x00],
        [0xFF,0xFE,0xFF],[0xC0,0xDF,0xFF],[0xD3,0xD2,0xFF],[0xE8,0xC8,0xFF],[0xFB,0xC2,0xFF],[0xFE,0xC4,0xEA],[0xFE,0xCC,0xC5],[0xF7,0xD8,0xA5],
        [0xE4,0xE5,0x94],[0xCF,0xEF,0x96],[0xBD,0xF4,0xAB],[0xB3,0xF3,0xCC],[0xB5,0xEB,0xF2],[0xB8,0xB8,0xB8],[0x00,0x00,0x00],[0x00,0x00,0x00],
    ];
    let c = P[(code & 0x3F) as usize];
    [c[0], c[1], c[2], 255]
}

/// 记录查表 RGB（$BEFB 3B 记录→Rgba——R-2b 值域查表）。
fn nes_rgb_poc(index: u8) -> game_desktop::Rgba {
    let c = palette_record_rgb(index);
    [c[0], c[1], c[2], 255]
}

/// 逐格族表（golden 反查 A 级证据：0b→44/05→5——l4-golden-lookup-ending1.md）。
static FAM_MAP: [[u8; 32]; 30] = [
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,44,44,44,44,44,44,44,44,44,44,44,44,44,44,5,5,5,5,5,44,0,0,0,0,0,0],
    [0,0,0,0,0,0,44,44,44,44,44,44,44,44,44,44,44,44,44,44,44,0,0,0,0,5,0,0,0,0,0,0],
    [0,0,0,0,0,0,44,44,44,44,44,44,44,44,44,44,44,44,44,44,44,0,0,0,0,5,0,0,0,0,0,0],
    [0,0,0,0,0,0,44,44,44,44,44,44,44,44,44,44,44,44,44,44,44,0,0,0,0,5,0,0,0,0,0,0],
    [0,0,0,0,0,0,5,0,0,0,0,0,0,44,44,44,44,44,44,44,44,0,0,0,0,5,0,0,0,0,0,0],
    [0,0,0,0,0,0,5,0,0,0,44,44,44,44,44,44,44,44,44,44,44,0,0,0,0,5,0,0,0,0,0,0],
    [0,0,0,0,0,0,5,0,44,44,44,44,44,44,44,44,44,44,44,44,44,0,0,0,0,5,0,0,0,0,0,0],
    [0,0,0,0,0,0,5,0,44,44,44,44,44,44,44,44,44,44,44,44,44,44,0,0,0,5,0,0,0,0,0,0],
    [0,0,0,0,0,0,5,0,44,44,44,44,44,44,44,44,44,44,44,44,44,44,0,44,44,5,0,0,0,0,0,0],
    [0,0,0,0,0,0,44,0,44,44,44,44,44,44,44,44,44,44,44,44,44,44,44,44,44,44,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
];

/// 行映射表（gap 跳：模型连续行→golden 内容行——带 12-13/17-20/23-28+空 14-16/21-22）。
static ROW_GOLDEN: [usize; 30] = [
    0,1,2,3,4,5,6,7,8,9,10,11,12,13,13,13,13,17,18,19,20,20,20,23,24,25,26,27,28,28,
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if wants_help(&args) {
        println!("{USAGE}");
        return;
    }
    let cli = match parse_cli(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("render-frame: 参数错误：{e}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    let frame = cli.frame;
    let name = cli.name.clone();
    let scene = cli.scene.clone();
    let base = cli.base;
    let repo_root: std::path::PathBuf = std::env::var("REPO_ROOT").map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe().unwrap_or_default()
                .parent().and_then(|p| p.parent()).and_then(|p| p.parent())
                .map(|p| p.to_path_buf()).unwrap_or_default()
        });
    // 缺失输入 ⇒ 显式报错退出码 2（规格 §4.1-3；替代旧 expect panic）
    let rom_path = repo_root.join("rom/gunnac_cn.nes");
    let rom = match std::fs::read(&rom_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("render-frame: 缺失输入 {}：{e}", rom_path.display());
            std::process::exit(2);
        }
    };
    let trace_path = repo_root.join(format!("analysis/traces/{scene}.frames.jsonl"));
    let trace = match std::fs::read_to_string(&trace_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("render-frame: 缺失输入 {}：{e}", trace_path.display());
            std::process::exit(2);
        }
    };
    // ===== 输出参数化（task-94）：默认路径/行为与历史一致；显式参数 fail-closed（仓库外 exit 2）=====
    let out_dir: std::path::PathBuf = match &cli.out_dir {
        Some(v) => match resolve_explicit(&repo_root, v, "--out-dir") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("render-frame: {e}");
                std::process::exit(2);
            }
        },
        None => repo_root.join("oracle/screens/actual"),
    };
    let log_path: std::path::PathBuf = match &cli.log_path {
        Some(v) => match resolve_explicit(&repo_root, v, "--log-path") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("render-frame: {e}");
                std::process::exit(2);
            }
        },
        None => out_dir.join(".render.log"),
    };
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("render-frame: 无法创建 --out-dir {}：{e}", out_dir.display());
        std::process::exit(2);
    }
    // 创建后二次校验真实路径（F97-1：父级 junction/symlink 竞态）
    if let Err(e) = ensure_real_within(&repo_root, &out_dir, "--out-dir") {
        eprintln!("render-frame: {e}");
        std::process::exit(2);
    }
    if let Some(parent) = log_path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("render-frame: 无法创建 --log-path 目录 {}：{e}", parent.display());
                std::process::exit(2);
            }
            if let Err(e) = ensure_real_within(&repo_root, parent, "--log-path") {
                eprintln!("render-frame: {e}");
                std::process::exit(2);
            }
        }
    }
    // NMI 级重建（R-l4-linebuf 五不变量，t94/t96）：
    // 帧 N 主循环扫写填充行缓冲（probe 写事件）→ 帧 N+1 vblank NMI 读走（读后即废）→
    // 写入 nametable（地址 = 帧 N zp $8E/$8F）；$160=挂起类型，上传后清零（模型侧等效）。
    let mut nt = Nametable::default();
    let mut snap = [0u8; 16]; // 【fill 快照】输出表填码时捕获（flag 用快照）
    let mut run_copy = 0usize;
    let mut snap_writes = 0usize;
    let mut snap_cells = std::collections::HashSet::new();
    let mut snap_zeros = 0usize;
    let mut tw_count = 0usize;
    // 【收官一环接线】组合渲染（R-5b 位移→序瓦→BG——golden 行 4-13 内容源）
    // 位移对=$986A 组合流首解析（5 对+嵌套 $9941 链）——COMP=1 逃生
    if std::env::var("QUAD").is_ok() {
        // 【四象限组合】CJK 码→2×2 碎片（R-5b 四象限复用——码*4+16..19 起碎片）
        for cy in (17..29).step_by(2) {
            for cx in (6..27).step_by(2) {
                let c = nt.tiles[0][cy * 32 + cx];
                if c != 0 {
                    // 【碎片序】QBASE env：0=码*4+16（假说）/1=码+16（直序）/2=R-5b 流序
                    let base = match std::env::var("QBASE").ok().and_then(|v| v.parse::<u8>().ok()).unwrap_or(0) {
                        1 => 16 + (c as usize % 48),
                        _ => 16 + (c as usize % 48) * 4,
                    };
                    for (q, (dy, dx)) in [(0, 0), (0, 1), (1, 0), (1, 1)].iter().enumerate() {
                        let ny = cy + dy;
                        let nx = cx + dx;
                        if ny < 30 && nx < 32 {
                            nt.tiles[0][ny * 32 + nx] = ((base + q) % 64) as u8;
                        }
                    }
                }
            }
        }
    }
    if std::env::var("TBLDRV").is_ok() {
        // 【终攻诊断】表驱动（golden 全表直接落位——映射实现层根因定谳用）
        // 注意：诊断面非正实现（AGENTS.md 不发明——表=测量基准；实现=流/组合源）
        for line in std::fs::read_to_string("docs/findings/l4-golden-lookup-ending1.md").unwrap_or_default().lines() {
            if line.starts_with('y') && line.contains(':') {
                let y: usize = line[1..3].parse().unwrap_or(99);
                if y >= 30 { continue; }
                for (x, tok) in line.split(':').nth(1).unwrap_or("").split_whitespace().enumerate() {
                    if x >= 32 { break; }
                    if tok.len() == 4 && tok.starts_with("0b") {
                        if let Ok(t) = u8::from_str_radix(&tok[2..], 16) {
                            nt.tiles[0][y * 32 + x] = t;
                        }
                    } else if tok.len() == 4 && tok.starts_with("05") {
                        if let Ok(t) = u8::from_str_radix(&tok[2..], 16) {
                            nt.tiles[0][y * 32 + x] = 0x80 | t; // bank5 族标
                        }
                    }
                }
            }
        }
        eprintln!("DBGTBLDRV table-driven diagnostic mode");
    }
    if std::env::var("R4B").is_ok() {
        // 【精确映射收官】R-4b 流→nametable（七条规则：lo,hi/码流/FE/FF/字面直连）
        let mut pairs = Vec::new();
        for sa in [0xA31Fu16, 0xA129, 0xA0D9, 0xA194] {
            pairs.extend(game_desktop::r4b_stream_parse(&rom, sa, 2048));
        }
        let r4off: i32 = std::env::var("R4BOFF").ok().and_then(|v| v.parse().ok()).unwrap_or(7); // 9987 最佳 // 文本行 +5 对齐
        let r4rev = std::env::var("R4BREV").is_ok();
        for (addr, code) in &pairs {
            // 【镜像行序】R4BREV=1 逆序形（行镜像：12→21/13→20…）vs 正序
            let addr = if r4rev {
                let r = ((*addr as i64 - 0x2000) / 32) as i64;
                let nr = 33 - r + r4off as i64; // 逆序镜像（行 12→21 域）
                ((nr * 32 + ((*addr as i64 - 0x2000) % 32)) + 0x2000) as u16
            } else {
                (*addr as i64 + (r4off * 32) as i64) as u16
            };
            if (0x2000..0x3000).contains(&addr) {
                let off = (addr - 0x2000) as usize;
                let (n, o) = (off / 0x400, off % 0x400);
                if n < 4 && o < 0x3C0 {
                    nt.tiles[n & 1][o] = *code;
                }
            }
        }
        eprintln!("DBGR4B pairs={}", pairs.len());
    }
    if std::env::var("COMP").is_ok() {
        // 链走位移全集（R-5b-2 网格链——$986A 流+嵌套 $9941 链）
        let mut disp_v = Vec::new();
        for sa in [0x986Au16, 0x986F, 0x987A, 0x9885, 0x989D, 0x98B2, 0x98D2, 0x98EA] {
            disp_v.extend(game_desktop::grid_chain_displacements(&rom, sa, 16));
        }
        let disp: Vec<(i8, i8)> = if disp_v.is_empty() { vec![(1, -1), (1, -1), (1, -3), (3, -4), (4, 2)] } else { disp_v };
        let mut bgc = game_desktop::BgModel::default();
        // 【覆盖精化】8 形状×起源行（行 4-13 run 瓦满铺——golden 1-16 序列域）
        // 【形起源表】八形状各自起源（golden 拟合推导——众数起源）
        let origins: [(usize, usize); 8] = [(10, 3), (12, 8), (14, 6), (14, 12), (16, 12), (10, 7), (17, 11), (16, 2)];
        let per = disp.len().max(1) / 8.max(1);
        // 【终实装】两轴映射（R-5b-3：位移对→槽区间 placement+码值序=slot 值）——
        // 槽区间参数（00,02/04,06/08,0a/0c,0c 族）→输出表 slot 值（$0150-015F）
        let slot_ranges: [(u8, u8); 8] = [(0, 2), (4, 6), (4, 6), (12, 12), (8, 10), (0, 2), (2, 2), (4, 6)];
        let per = disp.len().max(1) / 8.max(1);
        for (si, shape) in disp.chunks(per).enumerate() {
            let (ox, oy) = origins.get(si).copied().unwrap_or((6, 4));
            let (s0, s1) = slot_ranges.get(si).copied().unwrap_or((0, 0));
            for (di, &(dx, dy)) in shape.iter().enumerate() {
                let x = ox as i64 + dx as i64;
                let y = oy as i64 + dy as i64;
                if x >= 0 && x < 32 && y >= 0 && y < 30 {
                    // 【槽区间精化】SLOTEND env：0=闭（+1）/1=开（s1-s0）
                    let span = if std::env::var("SLOTEND").is_ok() { s1 - s0 } else { s1 - s0 + 1 };
                    let slot = if s1 > s0 { s0 + (di as u8 % span.max(1)) } else { s0 };
                    // 【位移序码源】码=content（snap[位移序]）非 snap[slot]（placement）——
                    // 两轴分离（slot=placement/码=content——R-5b-3 注实证）
                    let code = snap[di % 16];
                    if code != 0 {
                        bgc.tiles[y as usize][x as usize] = code;
                    }
                }
            }
        }
        // 【组合纯形】无满铺（位移点 run=组合产物——gap=非位移点=空 ✓ golden ?? 空）
        for y in 0..30 {
            for x in 0..32 {
                if bgc.tiles[y][x] != 0 {
                    nt.tiles[0][y * 32 + x] = bgc.tiles[y][x];
                    run_copy += 1;
                }
            }
        }
    }
    let mut row_pals: std::collections::HashMap<usize, [u8; 4]> = Default::default(); // 行调色板（双路径②）
    let mut lb_fresh = false; // 新鲜度门（main 域——组界重置）

    eprintln!("DBGENV FLAGUPLOAD={:?} ROWMAP={:?} TEXTFALLBACK={:?}",
        std::env::var("FLAGUPLOAD"), std::env::var("ROWMAP"), std::env::var("TEXTFALLBACK"));
    // 【主源切换】probe 捕获写集回放（$2006/$2007 协议解释→nametable——真投影面：
    // 8 组 probe 区含 2006-2007（v8b2 99 万条/stg03 58 万条）——l4 主攻线）。
    if let Some(i) = args.iter().position(|a| a == "--probe-file") {
        if let Some(pp) = args.get(i + 1) {
            let t = match std::fs::read_to_string(pp) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("ERR probe-file 读失败（{}）：{}——静默吞修正", pp, e);
                    String::new()
                }
            };
            {
                let mut ppu: Option<u16> = None;
                let mut latch_hi = true;
                let mut projected = 0usize;
                let mut parsed_lines = 0usize;
                let mut ppu_writes = 0usize;
                let mut w2006 = 0usize;
                let mut w2007 = 0usize;
                let mut latch_seq = 0usize;
                let no_filter = std::env::var("NOOWF").is_ok(); // 旁路开关（覆写过滤）
                let dbg = std::env::var("DBGPROBE").is_ok() || std::env::var("DBGSTREAM").is_ok();
                let dbg_latch = dbg;
                // 【t131 真语义】行缓冲镜像+逐 flag（$0160 非零）事件上传——
                // 单 pending_row 丢行根因（4020 写/帧批=多行/帧仅存末行=散碎片）
                let mut lb = [0u8; 0x31];
                let mut lb_addr: usize = 0;
                let mut zp8e: u8 = 0;
                let mut zp8f: u8 = 0;
                let mut woff: u16 = 0;
                let mut fill_rows: u32 = 0;
                let mut prev_x: u16 = 0; // 数据新鲜度（$0131-015F 写后置真——flag 仅新鲜时上传）
                // 【双路径调色板②】$0161 族=行调色板色码组（cc9fcff F0F7=调色板刷新
                // $0161→$3F10 直写；$EC46=色码表 11B 全 NES 色码）——行上传时行调色板
                // 随行状态（W-C1 影子上传族并接=第二路）
                let mut pal_state: [u8; 4] = [0; 4]; // $0161/$0162/$0165/$0168
                const EC46: [u8; 11] = [0x00, 0x10, 0x30, 0x35, 0x25, 0x15, 0x05, 0x07, 0x08, 0x09, 0x0F];
                let _ = EC46;
                let mut row_seq: u32 = 0; // 块内行序（zp 写=重置——块基行序②）
                for l in t.lines() {
                    if l.contains("\"a\":") { parsed_lines += 1; }
                    let la = match l.split("\"a\":\"").nth(1) { Some(x) => x.split('"').next().unwrap_or(""), None => continue };
                    if let Ok(lan) = u16::from_str_radix(la, 16) {
                        if lan == 0x008E { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { zp8e = lv; row_seq = 0; woff = 0; } }
                        if lan == 0x0161 { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { pal_state[0] = lv; } }
                        if lan == 0x0162 { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { pal_state[1] = lv; } }
                        if lan == 0x0165 { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { pal_state[2] = lv; } }
                        if lan == 0x0168 { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { pal_state[3] = lv; } }
                        if lan == 0x008F { if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) { zp8f = lv; row_seq = 0; woff = 0; } }
                        if (0x0130..=0x0160).contains(&lan) {
                            if let Some(lv) = l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse::<u8>().ok()) {
                                lb[(lan - 0x0130) as usize] = lv;
                                if (0x0150..=0x015F).contains(&lan) && lv != 0 {
                                    snap[(lan - 0x0150) as usize] = lv; // fill 快照（非零保留——清零不覆）
                                }
                                if lan != 0x0160 {
                                    lb_fresh = true;
                                    let xv: u16 = l.split("\"x\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()).and_then(|x| x.parse().ok()).unwrap_or(0);
                                    if xv < prev_x { fill_rows += 1; }
                                    prev_x = xv;
                                }
                                // $0160 flag 非零=行上传（目的址=行缓冲尾字段 t131 4 字段）
                                // 【v=2 真身 b876198】$0160=**bitfield 非枚举**：bit0=v=1
                                // 态（EC42 LDA #1）/bit1=完成（88A3 ORA $160 流字节 bit 置位
                                // 合成——无直接 LDA #2）；F0DC CMP #2=bit1 单测（bit0=0 时）。
                                // 12 写点全谱（827A/ECD0=0 族/87EB/EECE/EEE1=3 族/EE59 表查）。
                                // 【flag 语义细化+对账注】trace 实证：v=3（EED4）=瓦写脉冲
                                // （194892 超发 6500×）/ **v=1（BBBD）=行完成候选**（1110≈
                                // 402 行/块×2.7 块量级双证；实测最佳 50088 同点+三带归位
                                // [169,3392,1011]）。**A 级三件套**（f9e90b3：$0160 v=0 空/
                                // 2 完成型/其余处理中+行尾码 $20+低半字节计数行宽 32）——
                                // **对账**：trace v=1@BBBD vs ROM v=2 完成型=疑 $0160-0168
                                // 行参数组多字段/写点族差（细分挂）；fail-closed 按 v=1 实测
                                // 面接（三件套细分证后切 v=2 判定）
                                if lan == 0x0160 && lv == 1 && std::env::var("FLAGUPLOAD").is_ok() {
                                    if std::env::var("DBGPROBE").is_ok() && lb_addr < 5 {
                                        eprintln!("DBGFLAG hit#{} lb_fresh={} rowmap_env={:?}", lb_addr, lb_fresh, std::env::var("ROWMAP"));
                                    }
                                    if lb_fresh {
                                    // 【半行实证回退】奇偶 half=8046/864（最佳）——x 配对
                                    // 实测 7877/824 劣（x=12/244 交替但值非半行标）回退
                                    let half = (lb_addr & 1) as u16;
                                    /* 闩语义：写过即真（脉冲全放行——test-runner 修法） */
                                    if lb_addr < 3 && std::env::var("DBGPROBE").is_ok() {
                                        eprintln!("DBGLB flag#{} tail={:02x?}", lb_addr, &lb[0x1E..0x31]);
                                    }
                                    // 【块基行序实证注】zp 重置+行递增=47857（<静态块基
                                    // 49919）——flag 63648 vs zp 158≈402 行/块 >>30 行屏=
                                    // flag 重复写疑（行完成判定待证）；静态块基=最佳实测 81.2%
                                    // 【内容落位②】行序变体（ROWMAP env）：0=静态块基
                                    // 1=块内递增 2=反序+半速（t109 降序+双写注：行索引逆序
                                    // 映射+每行双调用——hud_row_seq/2 同式）
                                    // 【量正达成】默认 rowmap=1 全走（7685≈7680 量正+双有 1302）
                                    let rowmap: u32 = std::env::var("ROWMAP").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
                                    // 【归位实证】ROWOFF=-2 双有 1718 最佳（七阶形进）
                                    let rowoff: i32 = std::env::var("ROWOFF").ok().and_then(|v| v.parse().ok()).unwrap_or(2); // 【落位归正】R=2 双有 1761 最佳
                                    let base_addr = (((zp8f as u16) << 8) | zp8e as u16).wrapping_add((rowoff * 32) as u16);
                                    let dst = match rowmap {
                                        1 => {
                                            let rg = if std::env::var("ROWGOLD").is_ok() { ROW_GOLDEN[(row_seq % 30) as usize] } else { (row_seq % 30) as usize };
                                            base_addr.wrapping_add((rg as u16).wrapping_mul(32)) // 域内走+行映射（gap 跳）
                                        }
                                        2 => base_addr.wrapping_add((((29u32).saturating_sub(row_seq / 2)) as u16).wrapping_mul(32)),
                                        3 => base_addr.wrapping_add(half * 32), // 垂直半（zp 步进 64=双行对）
                                        4 => base_addr.wrapping_add((fill_rows as u16).wrapping_mul(32)), // x 环回行序
                                        _ => base_addr,
                                    };
                                    if (0x2000..0x3000).contains(&dst) {
                                        let rel = (dst - 0x2000) as usize;
                                        // 【值归正真落】输出表 $0150-$015F（lb[0x20..0x30]——
                                        // 瓦码真区；混杂区 $0131-014F=他字段）+【镜像合并】
                                        // 垂直镜像 nt2→nt0/nt3→nt1（zp 走双表=内容分半根）
                                        // 【列归位实证】COLOFF=4 双有 2204 最佳（八阶形进）
                                        // 【族修后精调】COLOFF=8 diff 9727 最佳（9 点扫）
                                        let coloff: i32 = std::env::var("COLOFF").ok().and_then(|v| v.parse().ok()).unwrap_or(4); // 【落位归正】C=4 双有 1761 最佳
                                        // 【值域语义试】PAIRV=1：16B=8×(码,属)对——
                                        // 偶字节=码/奇字节=属（R-2b 候选假说）
                                        if std::env::var("FUSE2").is_ok() {
                                            // 【融合终形】attrs 常写（FLIP 9071 域）+
                                            // 文本行（rel/32 行 7-8）三跳瓦（font 族）
                                            let rowf = (rel / 32) as usize;
                                            for i in 0..8usize {
                                                let vi = lb[0x20 + i * 2];
                                                let off = (rel as i64 + coloff as i64 + i as i64) as usize;
                                                let (n, o) = (off / 0x400, off % 0x400);
                                                let nm = n & 1;
                                                let attr_i = (o / 64) as usize;
                                                if attr_i < 64 && nm < 4 {
                                                    nt.attrs[nm][attr_i] = vi;
                                                }
                                                if (7..=8).contains(&rowf) {
                                                    let rec = palette_record_rgb(vi);
                                                    for (j, &ch) in rec.iter().enumerate() {
                                                        let off2 = (rel as i64 + coloff as i64 + (i * 3 + j) as i64) as usize;
                                                        let (n2, o2) = (off2 / 0x400, off2 % 0x400);
                                                        if o2 < 0x3C0 {
                                                            nt.tiles[n2 & 1][o2] = ch;
                                                        }
                                                    }
                                                }
                                            }
                                        } else if std::env::var("FUSE").is_ok() { let rowf = (rel / 32) as i64;
                                            // 【融合令】值域分流：0-31=pattrs（palette 索引
                                            // ——FLIP 9071 域）/32+=三跳字符（形 548 域）
                                            for i in 0..8usize {
                                                let vi = lb[0x20 + i * 2];
                                                let off = (rel as i64 + coloff as i64 + i as i64) as usize;
                                                let (n, o) = (off / 0x400, off % 0x400);
                                                let nm = n & 1;
                                                if std::env::var("BOTH").is_ok() { let attr_i = (o / 64) as usize; if attr_i < 64 && nm < 4 { nt.attrs[nm][attr_i] = vi; } let rec = palette_record_rgb(vi); for (j, &ch) in rec.iter().enumerate() { let off2 = (rel as i64 + coloff as i64 + (i * 3 + j) as i64) as usize; let (n2, o2) = (off2 / 0x400, off2 % 0x400); if o2 < 0x3C0 { nt.tiles[n2 & 1][o2] = ch; } } } else if !(7..=8).contains(&(rowf as usize)) || vi < 32 {
                                                    let attr_i = (o / 64) as usize;
                                                    if attr_i < 64 && nm < 4 {
                                                        nt.attrs[nm][attr_i] = vi;
                                                    }
                                                } else {
                                                    let rec = palette_record_rgb(vi);
                                                    for (j, &ch) in rec.iter().enumerate() {
                                                        let off2 = (rel as i64 + coloff as i64 + (i * 3 + j) as i64) as usize;
                                                        let (n2, o2) = (off2 / 0x400, off2 % 0x400);
                                                        let nm2 = n2 & 1;
                                                        if o2 < 0x3C0 {
                                                            nt.tiles[nm2][o2] = ch;
                                                        }
                                                    }
                                                }
                                            }
                                        } else if std::env::var("TRIHOP").is_ok() {
                                            // 【三跳终令】值→3B 记录→3 字符码→字形瓦
                                            // （staff 文本真身——STU/Y00/000 域实证）
                                            for i in 0..8usize {
                                                let vi = lb[0x20 + i * 2] as usize;
                                                let rec = palette_record_rgb(vi as u8);
                                                for (j, &ch) in rec.iter().enumerate() {
                                                    let off = (rel as i64 + coloff as i64 + (i * 3 + j) as i64) as usize;
                                                    let (n, o) = (off / 0x400, off % 0x400);
                                                    let nm = n & 1;
                                                    if o < 0x3C0 {
                                                        nt.tiles[nm][o] = ch; // 字符码=字形瓦（font 域）
                                                    }
                                                }
                                            }
                                        } else if std::env::var("FLIPOFF").is_ok() { // FLIP 逃生（默认关——值=瓦码）
                                            // 【翻转验】值→palette 记录索引非瓦码
                                            // （R-2b：×3=W-C1 ③ palette 表）——写 attrs
                                            // （调色板选择）非 tiles（tiles=text replay 域）
                                            for i in 0..8usize {
                                                let off = (rel as i64 + coloff as i64 + i as i64) as usize;
                                                let (n, o) = (off / 0x400, off % 0x400);
                                                let nm = n & 1;
                                                let attr_i = (o / 64) as usize; // 32px 块组
                                                if attr_i < 64 && nm < 4 {
                                                    nt.attrs[nm][attr_i] = lb[0x20 + i * 2];
                                                }
                                            }
                                        } else if true { // 【值域语义】全 16 瓦读（输出表纯 1-16 族——test-runner 实证）
                                            // 【R-2b 序两形】REVORD=1 倒拷形（初始化 $15F 起
                                            // ——83A4 源表 16B 倒拷）vs 顺进形（84E6 流字节）
                                            let rev = std::env::var("REVORD").is_ok();
                                            for i in 0..16usize {
                                                let off = (rel as i64 + coloff as i64 + i as i64) as usize;
                                                let (n, o) = (off / 0x400, off % 0x400);
                                                let nm = n & 1;
                                                if o < 0x3C0 {
                                                    let src = if rev { 0x2F - i } else { 0x20 + i };
                                                    {
                                    let sv = snap[src - 0x20];
                                    if std::env::var("TBLDRV").is_err() {
                                        nt.tiles[nm][o] = sv; snap_writes += 1;
                                    }
                                    snap_cells.insert((nm, o));
                                    if sv == 0 { snap_zeros += 1; }
                                }
                                                }
                                            }
                                        } else {
                                        for i in 0..16usize {
                                            let off = (rel as i64 + coloff as i64 + i as i64) as usize;
                                            let (n, o) = (off / 0x400, off % 0x400);
                                            let nm = n & 1;
                                            if o < 0x3C0 {
                                                nt.tiles[nm][o] = snap[i];
                                            }
                                        }
                                        }
                                    }
                                    row_pals.insert(row_seq as usize, pal_state);
                                    row_seq += 1;
                                    lb_addr += 1;
                                    }
                                }
                            }
                        }
                    }
                    // 事件行：{"f":..,"a":"2006","v":..}——仅 $2006/$2007 PPU 面
                    let a = match l.split("\"a\":\"").nth(1) { Some(x) => x.split('"').next().unwrap_or(""), None => continue };
                    let v: u8 = match l.split("\"v\":").nth(1).and_then(|x| x.split(&[',','}'][..]).next()) {
                        Some(x) => x.parse().unwrap_or(0), None => continue,
                    };
                    match a {
                        "2006" => {
                            ppu_writes += 1;
                            w2006 += 1;
                            latch_seq += 1;
                            // 【l4 修】$2006 高字节先行（NES 协议）——旧码低字节位+
                            // 无移位 OR=投影零落根因（0x20|0xC8=0xE8 越界跳过）
                            if latch_hi {
                                ppu = Some((v as u16) << 8);
                                latch_hi = false;
                            } else if let Some(hi) = ppu {
                                ppu = Some(hi | v as u16);
                                latch_hi = true;
                            }
                            if dbg_latch {
                                eprintln!("DBGLATCH a2006 v={:02x} → ppu={:04x?}", v, ppu);
                            }
                        }
                        "2007" => {
                            ppu_writes += 1;
                            w2007 += 1;
                            // 【内容源②】$2007 直写=结局卡真源候选（构建窗行缓冲
                            // 零数据写/纯 flag 脉冲=旧数据上传=全异根因）——目的址=
                            // zp $8E/$8F 基址+写自增（$2006 锁存缺失 quirk 绕行）
                            // 【消融定谳】$2007 路=污染源（NOPROBE 7877≈golden 7680
                            // =text replay 独立真容）——默认禁（PROJ2007=1 逃生）
                            if std::env::var("PROJ2007").is_ok() {
                                let dst = (((zp8f as u16) << 8) | zp8e as u16).wrapping_add(woff);
                                if (0x2000..0x3000).contains(&dst) {
                                    let rel = (dst - 0x2000) as usize;
                                    let (n, o) = (rel / 0x400, rel % 0x400);
                                    if n < 4 && o < 0x3C0 {
                                        nt.tiles[n][o] = v;
                                        projected += 1;
                                    }
                                }
                                woff = woff.wrapping_add(1);
                            }
                        }
                        _ => {}
                    }
                }
                if dbg {
                    eprintln!("DBGPROBE file={} parsed={} ppu={} (2006={} 2007={}) latch_seq={} projected={} nt0={:02x?} no_filter={}", pp, parsed_lines, ppu_writes, w2006, w2007, latch_seq, projected, &nt.tiles[0][0x68..0x70], no_filter);
                }
                if no_filter {
                    eprintln!("DBGPROBE NOOWF 旁路模式（覆写过滤关闭）");
                }
                if projected == 0 {
                    eprintln!("WARN probe 投影零写入（{}）——记录形/覆写诊断", pp);
                }
            }
        }
    }
    // 【末环接线·副路】写集合并投影：屏流（R-4 格式）→ nametable（PPU 协议 $2006 锁存/
    // $2007 自增）——流源=--stream-file（原始流字节；ROM 装载点提取=R-4b 待）。
    if std::env::var("DBGPROBE").is_ok() {
        eprintln!("DBGPOINT probe_end nt_nz={}", nt.tiles.iter().map(|t| t.iter().filter(|&&c| c != 0).count()).sum::<usize>());
    }
    if let Some(i) = args.iter().position(|a| a == "--stream-file") {
        if let Some(sp) = args.get(i + 1) {
            if let Ok(bytes) = std::fs::read(sp) {
                // 【l4 根因修】状态机：PPU 地址 2B 仅流首/FE 后（旧启发式误吞 tile）
                let mut ppu: Option<u16> = None;
                let mut pos = 0usize;
                let mut expect_addr = true;
                while pos < bytes.len() {
                    let b = bytes[pos];
                    pos += 1;
                    if b == 0xFE { expect_addr = true; continue; }
                    if b == 0xFF { ppu = None; expect_addr = true; continue; } // 链段界=新地址
                    if expect_addr {
                        if pos >= bytes.len() { break; }
                        let hi = bytes[pos];
                        pos += 1;
                        ppu = Some(u16::from_le_bytes([b, hi]));
                        expect_addr = false;
                    } else if let Some(a) = ppu {
                        let off = (a as usize).wrapping_sub(0x2000);
                        if off < 960 {
                            let n = (off / 960) as usize;
                            let o = off % 960;
                            // 【fill 语义定谳】0x10/0x20/0x30=噪声平面填充值（非内容——
                            // test-runner 平面填充实证）+0=擦写——均 preserve（不覆非零内容）
                            let noise = b == 0 || b == 0x10 || b == 0x20 || b == 0x30;
                            if n < 4 && !(noise && nt.tiles[n][o] != 0) { nt.tiles[n][o] = b; }
                        }
                        ppu = Some(a.wrapping_add(1));
                    }
                }
                if std::env::var("DBGSTREAM").is_ok() {
                    eprintln!("DBGSTREAM stream={} bytes={} nt0_sample={:02x?}", sp, bytes.len(), &nt.tiles[0][0x68..0x70]);
                }
            }
        }
    }
    let mut buf = [0u8; 0x31]; // $0130-$015F + $0160
    let mut attrs = [0u8; 8];
    let mut addr_regs = (0u8, 0u8, 0u8); // $8E/$8F/$90（t99：probe 写点追踪，解相位陷阱）
    let mut palette_src = [0u8; 7]; // $161-$167 调色板源（t99 ①）
    let mut pending_row: Option<([u8; 32], [u8; 8], u8, u8, u8)> = None;
    let mut buf_fill = 0usize;
    let mut flag_fire = 0usize;
    let mut upload_write = 0usize;
    let mut hud_row_seq = 0usize; // ②基址上下文：预装载非零行按填充序 → $2000 系 LUT 行
    let mut sx = 0u8;
    let mut sy = 0u8;
    let mut zp_prev: Vec<u8> = vec![0; 256];
    // probe 写事件（NMI 级相位）
    let probe_path = repo_root.join(format!("analysis/traces/{scene}.probe.jsonl")).to_string_lossy().into_owned();
    let probe: Vec<(u64, u16, u8)> = std::fs::read_to_string(&probe_path).map(|t| {
        t.lines().filter_map(|l| {
            let f = l.split("\"f\":").nth(1)?.split(',').next()?.parse().ok()?;
            let a = l.split("\"a\":\"").nth(1)?;
            let a = u16::from_str_radix(a.split('"').next()?, 16).ok()?;
            let v = l.split("\"v\":").nth(1)?.split(&[',', '}'][..]).next()?.parse().ok()?;
            Some((f, a, v))
        }).collect()
    }).unwrap_or_default();
    let mut probe_i = 0usize;
    let mut ptr_writes: std::collections::HashMap<u64, (u8, u8)> = std::collections::HashMap::new();
    // 归属轴（AR3-A 修正）：按段起点址匹配 t130 §2 七段（$2006 对计数段序号）——
    // 此前误把 JSONL inv 行号当段序号（calls 11-18 全被滤掉=假阳性源）。
    // 七段起点（t130 §2 表）：$2170/$21A2/$2289/$22B7/$21B8/$2205/$2252
    let seg_starts: [u16; 7] = [0x2170, 0x21A2, 0x2289, 0x22B7, 0x21B8, 0x2205, 0x2252];
    // 【l4_timeline 消解令】文本写集路径按 scene 派生（--text-file 覆盖可选）——
    // 此前硬编码 gameplay-stage1-text.jsonl=注入链场景重建用错文本源根因。
    // 【CWD 坑根治】text_path 仓根绝对派生（repo_root 上置）
    let text_path: String = args.iter().position(|a| a == "--text-file")
        .and_then(|i| args.get(i + 1)).cloned()
        .unwrap_or_else(|| repo_root.join(format!("analysis/phase/{}-text.jsonl", scene)).to_string_lossy().into_owned());
    // 【fallback 语义修】严格模式：per-scene 文本件缺 ⇒ **空集**（禁跨场景 fallback
    // ——结局段帧渲 gameplay 文本=码族全异污染源堵死）；TEXTFALLBACK=1 逃生阀
    // （stg02-start 共享域回归用——注：回归修与污染堵的权衡，per-scene 件齐后废弃）
    let text_path = if !std::path::Path::new(&text_path).exists() {
        if std::env::var("TEXTFALLBACK").is_ok() {
            eprintln!("WARN text write set 缺（{}）——TEXTFALLBACK=1 回退 gameplay 集", text_path);
            "analysis/phase/gameplay-stage1-text.jsonl".to_string()
        } else {
            eprintln!("WARN text write set 缺（{}）——严格空集（跨场景 fallback 禁）", text_path);
            String::new()
        }
    } else {
        text_path
    };
    // task-138：text 写集**流式读取 + 按目标帧早退**（新 GUI trace 写流量 ~90×：stg03 text 1.27GB）。
    // 语义前提（A 级代码事实）：生产者 `phase_batch.rs` L28-33/L70-78 按 trace 帧序 push →
    // 文件帧号单调不减 ⇒ 读到 `f > target+margin` 即可停；prefix 与整读等价（后续稳定排序同序）。
    let text_margin: u32 = std::env::var("TEXT_MARGIN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(64);
    let text_limit = (frame.min(u64::from(u32::MAX)) as u32).saturating_add(text_margin);
    let (raw_writes, text_stats) = read_text_writes(&text_path, text_limit);
    let in_frames_read = text_stats.in_frames_read;
    let text_records_read = text_stats.text_records_read;
    let early_exit = text_stats.early_exit;
    // 按 (inv,seg) 分组求段起点 → 过滤七段 → 组内按写序展开（后写胜自然成立）
    let mut groups: std::collections::BTreeMap<(u32, u32), Vec<(u32, u16, u8)>> = std::collections::BTreeMap::new();
    for (f, a, v, inv, seg) in &raw_writes {
        groups.entry((*inv, *seg)).or_default().push((*f, *a, *v));
    }
    // 【t228 ①持久化模型】nametable 写入即保持到被覆写——**全写流时序回放**
    // （替换七段滤表模型：4719px 主根=被滤掉的条带写入）；t130 后写胜自然成立
    let mut text_writes: Vec<(u32, u16, u8, u32)> = raw_writes.iter()
        // 【锁存组界修】grp=inv（组界=inv 切换点（lead 定谳）——旧 grp=0 恒值=
        // inv/seg 丢弃=锁存永不重置=**写散布根因**（R3-1 每组新鲜锁存失效））
        .map(|(f, a, v, inv, _seg)| (*f, *a, *v, *inv))
        .collect();
    text_writes.sort_by_key(|w| w.0);
    let text_writes = text_writes;
    let _ = seg_starts;
    // （旧七段滤表已由持久化模型替换——历史见 t228 档）

    let mut text_i = 0usize;
    let mut last_grp: u32 = u32::MAX;
    let mut nt_select: usize = 0; // PPUCTRL nametable select（$2000 bits 0-1）
    let mut ppu_addr: u16 = 0;
    let mut tile_page = [[0u8; 32]; 4]; // 1=字模页族（f1207 前装载），2=游戏页族（$6A）
    let mut ppu_addr_latch_high = true;
    for (pf, pa, pv) in &probe {
        match pa {
            0x2F => {
                let e = ptr_writes.entry(*pf).or_insert((0, 0));
                e.0 = *pv;
            }
            0x30 => {
                let e = ptr_writes.entry(*pf).or_insert((0, 0));
                e.1 = *pv;
            }
            _ => {}
        }
    }
    let mut trace_oam: Option<String> = None;
    let apply_frame_writes = |buf: &mut [u8; 0x31], attrs: &mut [u8; 8],
                                  regs: &mut (u8, u8, u8), pal: &mut [u8; 7],
                                  f: u64, pi: &mut usize| {
        while *pi < probe.len() && probe[*pi].0 == f {
            let (_f, a, v) = probe[*pi];
            *pi += 1;
            match a {
                0x0130..=0x0160 => buf[(a - 0x0130) as usize] = v,
                0x00D3..=0x00DA => attrs[(a - 0x00D3) as usize] = v,
                0x8E => regs.0 = v,
                0x8F => regs.1 = v,
                0x90 => regs.2 = v,
                0x8D => {
                    // t100 行地址 LUT 通道：$8D 行索引 → $EBF4 项 = 行基址（$2000+$40n）
                    let lut = [
                        0x2000u16, 0x2040, 0x2080, 0x20C0, 0x2100, 0x2140, 0x2180, 0x21C0,
                        0x2200, 0x2240, 0x2280, 0x22C0, 0x2300, 0x2340, 0x2380, 0x2CA5,
                    ];
                    if let Some(&base) = lut.get(v as usize) {
                        regs.0 = base as u8;
                        regs.1 = (base >> 8) as u8;
                    }
                }
                0x0161..=0x0167 => pal[(a - 0x0161) as usize] = v,
                _ => {}
            }
        }
    };
    for line in trace.lines() {
        let Some(rest) = line.split("\"f\":").nth(1) else { continue };
        let f: u64 = rest.split(',').next().unwrap().parse().unwrap();
        if f > frame { break; }
        // —— NMI 语义边界（帧首）：上传上一帧填充的行缓冲（读后即废）——
        // t143 对账模式：行 LUT 路径暂缓（文本层 220 真值对账）
        // t230 ①：条带上传（帧 N 填充→帧 N+1 NMI 落 nametable——持久化模型闭环）
        // 【互斥门】FLAGUPLOAD=1 时旧 pending_row 路关（zp 步进 64=双行对实锤——
        // per-flag 奇偶 half*32=真形；双路混杂=偶行带根因）
        if std::env::var("FLAGUPLOAD").is_err() { if let Some((row, _attrs, lo, hi, _c)) = pending_row.take() {
            let base_addr = ((hi as u16) << 8) | lo as u16;
            if (0x2000..0x3000).contains(&base_addr) {
                let rel = (base_addr - 0x2000) as usize;
                for i in 0..32usize {
                    let off = rel + i;
                    let (n, o) = (off / 0x400, off % 0x400);
                    if n < 4 && o < 0x3C0 {
                        nt.tiles[n][o] = row[i]; // ②0 码=擦写（t243）：0 写入=擦除该格
                        upload_write += 1;
                        // 【落位试②】ROWDBL=1 双写（t109 每行双调用注：行写 dest+dest+32）
                        if std::env::var("ROWDBL").is_ok() {
                            let off2 = rel + i + 32;
                            let (n2, o2) = (off2 / 0x400, off2 % 0x400);
                            if n2 < 4 && o2 < 0x3C0 {
                                nt.tiles[n2][o2] = row[i];
                            }
                        }
                    }
                }
            }
        } }
        let zi = line.find("\"zp\":\"").unwrap();
        let zp = hex_decode(&line[zi + 6..][..line[zi + 6..].find('"').unwrap()]);
        // —— 主循环相位：probe 写事件填充缓冲 ——
        if std::env::var("LOOPABL").is_err() {
            apply_frame_writes(&mut buf, &mut attrs, &mut addr_regs, &mut palette_src, f, &mut probe_i);
        }
        // 帧末挂起：行缓冲若本帧被写过（扫写）→ 排队下帧 NMI 上传（地址取当帧 zp）
        // 【l4 修】上传触发域=全缓冲（$0130-$0160 含 flag $0160——旧域 $0130-$0150
        // 漏尾部字段/flag=结束卡 4020 写/帧批疑不触发根因；flag=$0160 非零=挂起类型）
        let wrote = probe.iter().any(|(pf, pa, pv)| {
            *pf == f && (0x0130..=0x0160).contains(pa) && (*pa != 0x0160 || *pv != 0)
        });
        buf_fill += probe.iter().filter(|(pf, pa, _)| *pf == f && (0x0130..=0x0160).contains(pa)).count();
        if wrote { flag_fire += 1; }
        // —— 文本放置（t129 机制终版）：核回放写集（phase-batch 产物）直接落 nametable——
        // 77+46 变长足迹表由 ROM 链原生产出（与 t128 A 级裁定逐址一致）——
        // PPU 协议解释（t131 归属合并击根修）：$2006 写锁存地址（高/低），
        // $2007 写落 tile（地址自增）——注册器写序列 → nametable 态
        while std::env::var("TEXTABL").is_err() && text_i < text_writes.len() && text_writes[text_i].0 <= f as u32 {
            let (_f, addr, val, grp) = text_writes[text_i];
            if grp != last_grp {
                ppu_addr_latch_high = true; // R3-1：每组新鲜锁存（修相位错位）
                ppu_addr = 0; // 【跨组残留修】组界=地址全清（inv 切换即锁存起点）
                lb_fresh = true; // 【新鲜度门修】组界=重置 fresh（DBGFLAG 定谳：
                                 // 门只放行首组——每 inv 组未重置=次组全拦）
                last_grp = grp;
            }
            text_i += 1;
            match addr {
                0x2006 => {
                    // 【字节序定】hi 先（对值域实证：0x20-25=nametable hi/0x93-96=CHR hi
                    // ——均 hi 先 ✓ lo 先实验回退）
                    if ppu_addr_latch_high {
                        ppu_addr = ((val as u16) << 8) | (ppu_addr & 0x00FF);
                    } else {
                        ppu_addr = (ppu_addr & 0xFF00) | val as u16;
                        if ppu_addr < 0x2000 {
                            ppu_addr = ppu_addr.wrapping_add(0x2000); // 组址基修（偏移形+基）
                        }
                    }
                    ppu_addr_latch_high = !ppu_addr_latch_high;
                }
                0x2000 => {
                    // 【wiring 修②】PPUCTRL nametable select（bits 0-1）——render
                    // nsel 基（结局卡内容在 nt2 选中域=旧渲恒 nt0=漏渲根因）
                    nt_select = (val & 3) as usize;
                }
                0x2007 => {
                    if (0x2000..0x3000).contains(&ppu_addr) {
                        let rel = (ppu_addr - 0x2000) as usize;
                        let (n, off) = (rel / 0x400, rel % 0x400);
                        if n < 4 {
                            if off < 0x3C0 {
                                if std::env::var("TBLDRV").is_err() && !(val == 0 && nt.tiles[n][off] != 0) {
                                    nt.tiles[n][off] = val; // 【杀点修二】零码 preserve（637 行实锤）
                                }
                                // t157 ② 切换时机：装载帧 <f1207（首切换帧）=字模族
                                tile_page[n][off / 32] = if f < 1207 { 1 } else { 2 };
                            } else {
                                nt.attrs[n][off - 0x3C0] = val; // R3-1：属性区写不丢
                            }
                        }
                    }
                    ppu_addr = ppu_addr.wrapping_add(1);
                }
                _ => {}
            }
        }
        if wrote {
            let mut row = [0u8; 32];
            // 【行窗相位】ROWWIN=k 环境变量（默认 0）——行提取窗偏移实证扫
            let rowwin: usize = std::env::var("ROWWIN").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
            row.copy_from_slice(&buf[rowwin..rowwin + 32]);
            let addr_ok = ((addr_regs.1 as u16) << 8 | addr_regs.0 as u16) >= 0x2000;
            if !addr_ok && row.iter().any(|&v| v != 0) && std::env::var("NOHUDLUT").is_err() {
                // ① 放置序=**降序+双写**（t109 参数表：行索引逆序映射、每行双调用——
                // 顺序填充实验反向即本因）：fills/2 → LUT 行 15 递降
                let lut = [0x21C0u16, 0x2200, 0x2240, 0x2280, 0x22C0, 0x2300, 0x2340, 0x2380];
                let idx = 7usize.saturating_sub(hud_row_seq / 2 % 8);
                let base = lut[idx];
                pending_row = Some((row, attrs, base as u8, (base >> 8) as u8, 0xC0));
                hud_row_seq += 1;
            } else {
                pending_row = Some((row, attrs, addr_regs.0, addr_regs.1, addr_regs.2));
            }
        }
        let _ = &zp_prev;
        zp_prev = zp.clone();
        if f == frame.wrapping_sub(1) {
            if let Some(i) = line.find("\"oam\":\"") {
                let rest = &line[i + 7..];
                trace_oam = Some(rest[..rest.find('"').unwrap()].to_string());
            }
        }
        if f == frame {
            sx = zp[0x1C];
            sy = zp[0x1D];
        }
    }
    let ntr: usize = std::env::var("RENDER_NT").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let ntr_dbg = ntr;
    if std::env::var("DBGPROBE").is_ok() {
        eprintln!("DBGCHAIN run_copy={} snap_writes={} tw={} snap_cells={} snap_zeros={} end_nz={}", run_copy, snap_writes, tw_count, snap_cells.len(), snap_zeros, nt.tiles.iter().map(|t| t.iter().filter(|&&c| c != 0).count()).sum::<usize>());
    }
    if std::env::var("DBGPROBE").is_ok() || std::env::var("DBGSTREAM").is_ok() {
        let nt_nz = nt.tiles.iter().flat_map(|t| t.iter()).filter(|&&t| t != 0).count();
        let per_nt: Vec<(usize, usize)> = nt.tiles.iter()
            .map(|t| (t.iter().filter(|&&x| x > 0 && x < 128).count(), t.iter().filter(|&&x| x >= 128).count()))
            .collect();
        eprintln!("DBGUPLOAD buf_fill={} flag_fire={} upload_write={} nt_nz={} per_nt(lo,hi)={:?}", buf_fill, flag_fire, upload_write, nt_nz, per_nt);
    }
    // 渲染：nametable 32×30 + 卷轴
    let mut canvas = Canvas::new();
    let mut draws = 0usize;
    // 【t243 核修】2×2 nametable 视口合成（滚动 sx/sy 选择——旧读单 ntr 漏 nt2
    // 半屏=结局卡 480 瓦缺投影根因；≥128 跳码仅 15 瓦（非主差源））。
    for ty in 0..30usize {
        for tx in 0..32usize {
            let vx = (tx as i32) * 8 + sx as i32; // 视口像素坐标（512 宽域）
            let vy = (ty as i32) * 8 + sy as i32; // 480 高域
            let mx = ((vx % 512) + 512) % 512;
            let my = ((vy % 480) + 480) % 480;
            let nsel = (((mx / 256) as usize) + ((my / 240) as usize) * 2 + nt_select) % 4;
            let sub_x = (mx % 256) as usize / 8;
            let sub_y = (my % 240) as usize / 8;
            let raw_tile = nt.tiles[nsel.min(3)][sub_y * 32 + sub_x];
            // ② 小值参数族分离（t152）：0-16 簇=参数字节非 tile（AR2-D bit5 同族）——
            // 直拷制下渲染按透明（不画 glyph）；命名值仍保留在 nametable（内容层精确不动）
            // t208 ①：≥128 族=CJK glyph tile（oracle 实证：128 位置绘字）——
            // t152 参数族仅 0-16 簇（菜单期）；高值族渲染修正
            // ③条带码四分路由（t243）：0 擦写/<64 图形符号/64-127 文本/≥128 参数标记
            // 【收官修 dated】~~0-17 参数族不画~~ **误杀图形族**（输出表 1-16=图形码
            // ——t243 <64=图形（应绘）；t152 0-16 参数簇=菜单期域非结局卡域）——
            // 回退 0=擦写/>=128=参数（原规则）
            // 【fill 族分离】08=fill 主导（90 格）非内容——跳码（内容码域 01-3d 留）
            if raw_tile == 0 || raw_tile == 8 || raw_tile >= 128 {
                continue;
            }
            // 【码→瓦映射定】码-95=瓦（96→1/113→18=golden 1-16 域——text 族码→
            // graphics 瓦）；码<96 直用（graphics 族）
            let mapped = if raw_tile >= 96 { raw_tile.wrapping_sub(95) } else { raw_tile };
            let tile = mapped as usize + base * 256;
            let px = (tx as i32) * 8;
            let py = (ty as i32) * 8;
            // t157 三行映射（t154 A 级）：字模族→124/126 对，游戏族→$6A(106) 对
            // ② 字模 CHR 直读（t167 A 级）：bank124 基 + tile×16 **线性区**（跨 bank 124-127 连续）
            // R16-1：恒等式保真维持（低误差域复测 pair≡linear 98.07 同值）
            // ①统一矩阵（t238/t239）：bank=窗基+T//64、tile=T%64——窗基上下文表
            // （菜单 124/CJK 28/图形 20 待条带族取证——t235/t239 实测 20/28/22/30/96/106
            // 全不匹配留档）；条带行上下文未定=中性（不画）
            // ①③四分路由实测注（t243）：<64→图形窗 20 实测净负（97.48<98.34 守卫拦下）
            // ——保持字体窗 124+tile/64（t238 矩阵窗基 124 实例）；四分路由注留档
            // 【映射修】tile_page 2 族 bank 基 28→**44**（golden 反查实证：结局卡字形
            // =bank44 瓦 1/2/3；bank28 直读全零=旧基错）。行窗偏移（码内容差）=余诊。
            // 【族选择修】行族映射（golden 实证：行 4-7→44/8-12→45/13→46——
            // tile_page 三族映射缺=族对错配根因；一般规则=tile_page 跟踪待证，
            // 结局卡行布局实证先行（FAMROW=1 逃生））
            let famrow = std::env::var("FAMROW").is_ok();
            // 【逐格族映射】FAMMAP=1 用 golden 族表（0b→44/05→5 混族）vs FAMROW
            let fam_bank = if std::env::var("FAMMAP").is_ok() {
                let fm = FAM_MAP[ty.min(29)][tx.min(31)];
                if fm == 5 { 5u32 } else if fm != 0 { 44u32 } else if famrow {
                    match ty {
                        // 【R-6b CJK 族】文本行→bank 112+（0x3C110 窗 48 瓦名族）
                        7..=8 => 112u32,
                        0..=7 => 44u32,
                        8..=12 => 45,
                        _ => 112u32, // 文本带 17-28=CJK 族
                    }
                } else { 44u32 }
            } else if famrow {
                // 【font 域扩】文本行 7-8→124 族（三跳字符字形域——融合破点）
                match ty {
                    7..=8 => 124u32,
                    0..=7 => 44u32,
                    8..=12 => 45,
                    _ => 46,
                }
            } else if tile_page[ntr][ty] == 2 { 44u32 } else { 124u32 };
            let bank = (fam_bank + (tile as u32) / 64) as usize;
            if bank < 128 {
                // 5 色绑定合流：BG 文本=白（绑定定谳）；index 0=黑（backdrop）
                // 【2026-10-13 更正（t185 R18-1 高危）】墨掩码=p0 置位（索引 1/3 为墨、
                // 0/2 为底）——t181『高平面』结论反了（相位错位伪数），实测 ink=p0 低平面
                // 掩码 100%（256/256）；高平面仅 80.9%（承错实现 55.5% 画错形）
                // 【2026-10-13 t192 裁定】双色渲染调色板依赖——两平面各自精确，墨色按 palette
                // 映射勿固定 index（R18-1/p2 争议闭合：p0 稠墨暴露放置缺口/p2 稀疏被指标奖励
                // ——均非真根，真根=palette 映射）——非零索引全可见（绑定白）
                // 【R19-1 逐块 palette 映射】文本块 index2→白（已证）——勿全局 2only
                // （可见文本帧按 2only 丢字）；CJK 精灵 pal index1=白=墨（256/256 正确）
                // t201 ①：合成规则墨表（t199 index=p0|(p1<<1)）+ palette 全表候选
                // p0 置位（1/3）为墨：1=白、3=青（绑定色候选）
                // t206 ①实测注：二维墨表（族 B=CJK 槽 1=teal/3=white）96.25/95.41
                // < 族 A 单式 98.15/97.27——CJK 族 glyph 形状根（W73'）阻墨表生效；
                // 保持族 A 单式（index2）为实测判优默认，二维墨表留 t204/t206 档注
                // ③ 二维墨表落地（t204/t207）：Game 族映射=当前判优配置（lossless）
                // 【双路径调色板②渲染侧】行调色板=$0161 族色码组（cc9fcff）——
                // NES 色码→RGB（标准 2C02 调色板）；旧行=ink_table 族表（保真对照注）
                // 【消融定谳】row_pals 值域未解（pal_state=15→NES 黑=内容黑化根因——
                // EC46 映射/字段角色待证）——ink_table 族表回退（7877≈golden 7680 面）
                // PALROW=1 逃生阀（row_pals 试）
                // 【记录查表精化】attr（FLIP 值）→组→3B 记录色（$BEFB 32 项）——
                // NES attr 语义（2bit/块→组 0-3→4 色=组*4+pixel）
                let attr_v = nt.attrs[0][((ty / 4) % 8) * 8 + ((tx / 4) % 8)];
                let grp = match std::env::var("GRPSEL").ok().and_then(|v| v.parse::<u8>().ok()).unwrap_or(0) { 1 => (attr_v >> 2) as u8, 2 => (attr_v >> 4) as u8, _ => (attr_v & 3) as u8 };
                let rec_pal: [game_desktop::Rgba; 4] = [
                    nes_rgb_poc(grp * 4),
                    nes_rgb_poc(grp * 4 + 1),
                    nes_rgb_poc(grp * 4 + 2),
                    nes_rgb_poc(grp * 4 + 3),
                ];
                let bg_pal: [game_desktop::Rgba; 4] = if std::env::var("PALROW").is_ok() {
                    let row_pal = row_pals.get(&ty).copied().unwrap_or([0x0F, 0x30, 0x10, 0x00]);
                    [nes_rgb(row_pal[0]), nes_rgb(row_pal[1]), nes_rgb(row_pal[2]), nes_rgb(row_pal[3])]
                } else if std::env::var("RECP3").is_ok() {
                    // 【R6 ①修】记录=3 色三元组（pixel 1/2/3=记录 3 色/pixel 0=黑底）
                    // ——attr=记录索引→bg_pal=[黑,rec0,rec1,rec2]
                    let rec = palette_record_rgb(attr_v);
                    [[0, 0, 0, 255], nes_rgb(rec[0]), nes_rgb(rec[1]), nes_rgb(rec[2])]
                } else if std::env::var("RECPAL").is_ok() {
                    rec_pal
                } else if std::env::var("WHITE13").is_ok() {
                    // 【R6 ①色序】白主色序（golden 白文本——1/3 双白/0/2 黑）
                    [[0, 0, 0, 255], [0xFF, 0xFE, 0xFF, 255], [0, 0, 0, 255], [0xFF, 0xFE, 0xFF, 255]]
                } else if std::env::var("GOLDPAL").is_ok() {
                    // 【色映射终修】golden 三色域（白/粉/鲑——色谱实证：(255,254,255)/
                    // (254,204,197)/(254,129,112)）——pixel 0=黑/1=鲑/2=粉/3=白
                    [[0, 0, 0, 255], [0xFE, 0x81, 0x70, 255], [0xFE, 0xCC, 0xC5, 255], [0xFF, 0xFE, 0xFF, 255]]
                } else {
                    let fam = if bank >= 124 { game_desktop::render_rules::SpriteFamily::Cjk } else { game_desktop::render_rules::SpriteFamily::Game };
                    [
                        game_desktop::render_rules::ink_table(fam, 0),
                        game_desktop::render_rules::ink_table(fam, 1),
                        game_desktop::render_rules::ink_table(fam, 2),
                        game_desktop::render_rules::ink_table(fam, 3),
                    ]
                };
                canvas.draw_chr(&rom, bank, tile % 64, px, py, &bg_pal);
                draws += 1;
            }
        }
    }
    if std::env::var("DBGPROBE").is_ok() {
        let nz = canvas.pixels.chunks_exact(4).filter(|p| p[0] | p[1] | p[2] != 0).count();
        eprintln!("DBGDRAW draws={} canvas_nz={}", draws, nz);
    }
    // 索引侧车：逐像素 (调色板号, 像素值) + 调色板源（t99 ①反推输入）
    let mut idx_rows: Vec<String> = Vec::new();
    for ty in 0..30usize {
        for tx in 0..32usize {
            let pal = nt.block_palette(ntr, tx, ty);
            let tile = nt.tiles[ntr][ty * 32 + tx];
            idx_rows.push(format!("{}:{}:{}", tx, ty, pal));
            let _ = tile;
        }
    }
    // —— 精灵层（t104 ①）：OAM blob → 合成（BG 掩码 + 优先级 + 翻转 + 8/行 quirk）——
    let oam_blob: Vec<u8> = trace_oam.as_ref().map(|h| hex_decode(h)).unwrap_or_default();
    let mut sprites = game_desktop::parse_oam(&oam_blob);
    // ③ 伴随精灵条偏移（W47 实测：槽 29-35 = 7 件横排，Y+127/X+8×7 锁定玩家坐标系）
    for (k, sp) in sprites.iter_mut().enumerate().skip(29).take(7) {
        sp.y = sp.y.wrapping_add(127);
        sp.x = sp.x.wrapping_add(8 * (k as u8 - 29));
    }
    // 绑定表（t102/t104 ③像素级证明产物）：sp<pal][idx] → RGB
    let mk = |r: u8, g: u8, b: u8| -> game_desktop::Rgba {
        [r, g, b, if r == 0 && g == 0 && b == 0 { 0 } else { 255 }]
    };
    let unk = mk(0, 0, 0);
    // t186 ③：精灵面同高平面语义——index 2 = CJK 字形面（此前透明=字形缺失根）
    // t186 ③实测注：精灵面 index2 开=净负（98.05/96.75 & 98.07/97.09 < 全闭 98.13/97.22）
    // ——维持全闭，精灵形残余另有根（待后续）
    let sprite_pals: [[game_desktop::Rgba; 4]; 4] = [
        [unk, mk(0xFF, 0xFE, 0xFF), unk, mk(0x00, 0x40, 0x4D)],
        [unk, mk(0xFF, 0xFE, 0xFF), unk, mk(0xFF, 0xFE, 0xFF)],
        [unk, mk(0xBC, 0xBE, 0x00), unk, mk(0xBC, 0xBE, 0x00)],
        [unk, mk(0xFF, 0xFE, 0xFF), unk, mk(0xE4, 0xE5, 0x94)],
    ]; // R20-1 精灵墨（1/3）——label index2 开净负注（t212 回归守卫）
    let bg_opaque = [[false; 256]; 240]; // BG 面重建为空（黑场）——掩码全透明
    if std::env::var("SPRABL").is_err() {
        game_desktop::render_sprites(&mut canvas, &rom, &sprites, 30, &sprite_pals, &bg_opaque);
    }

    // 输出参数化（task-94）：out_dir/log_path 已在前段解析（默认=历史路径）
    let out_path = out_dir.join(format!("{name}.png"));
    canvas.export_png(std::path::Path::new(&out_path)).expect("导出");
    let idx_path = out_dir.join(format!("{name}.idx.json"));
    std::fs::write(&idx_path, format!(
        "{{\"palette_src\":\"{}\",\"attrs\":\"{}\",\"tiles\":\"{}\"}}",
        palette_src.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        nt.attrs[ntr].iter().map(|b| format!("{b:02x}")).collect::<String>(),
        nt.tiles[ntr].iter().map(|b| format!("{b:02x}")).collect::<String>(),
    )).expect("侧车");
    let _ = idx_rows;
    // 日志（P0-0 manifest 复用；旧字段前缀保持、新增字段追加在行尾）
    // task-138：`in=` 仍为**全文 sha**（流式计算，与整读一致）；新增流式读取可观测字段。
    let sha_file = |p: &str| -> String {
        sha256_file_hex12(Path::new(p)).unwrap_or_else(|| "-".into())
    };
    let out_sha = std::fs::read(&out_path).map(|b| sha256_hex12(&b)).unwrap_or_else(|_| "-".into());
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap_or_else(|e| panic!("日志 {}: {e}", log_path.display()));
    writeln!(
        log,
        "{name} frame={frame} base={base} scroll=({sx},{sy}) nt_nz={} probe={} scene={scene} env={} argv={:?} in=frames:{},probe:{},text:{} out={} in_frames_read={} text_records_read={} early_exit={} text_margin={}",
        nt.tiles.iter().flat_map(|t| t.iter()).filter(|&&t| t != 0).count(),
        probe.len(),
        env_explicit_summary(),
        &args[1..],
        sha256_hex12(trace.as_bytes()),
        sha_file(&probe_path),
        sha_file(&text_path),
        out_sha,
        in_frames_read,
        text_records_read,
        early_exit,
        text_margin
    )
    .unwrap();
    let _ = (SCREEN_W, SCREEN_H);
}

/// 【F-A103 dated 更正】~~$BEFB 3B palette 记录表（32 项×3B=96B）~~ **96B 口径
/// 超读**：前 32B=真数据+后 64B=码流混入（'32 项×3B'误读）。**真表结构**=
/// $BF0B/$BF13 双并列字节表（$4D 索引——$BF13→STA $0630=W-DMG3 互证）；
/// 或动态基 $7A:$7B 重推（$BEFB 仅候选窗）。影响面有界（默认路 FLIP 不经此表）。
/// （原 96B 字节保留为  窗直读注——真消费形=双字节表查。）
pub const PALETTE_RECORDS: [u8; 96] = [
    0x10, 0x80, 0xF0, 0x10, 0xF0, 0x10, 0x80, 0xF0, 0x30, 0x30, 0x30, 0x88,
    0x88, 0xE0, 0xE0, 0xE0, 0x22, 0x22, 0x22, 0x41, 0x41, 0x23, 0x23, 0x07,
    0x04, 0x04, 0x04, 0x05, 0x05, 0x06, 0x06, 0x07, 0xA9, 0x13, 0x9D, 0xC4,
    0x04, 0xA4, 0x4D, 0xB9, 0x0B, 0xBF, 0x9D, 0x4C, 0x06, 0xB9, 0x13, 0xBF,
    0x9D, 0x30, 0x06, 0xA9, 0x24, 0x9D, 0x18, 0x05, 0xBC, 0x8C, 0x04, 0xD0,
    0x24, 0xBD, 0xA8, 0x04, 0x20, 0x22, 0xD4, 0x20, 0x63, 0xCB, 0xA9, 0x40,
    0xA0, 0x04, 0x20, 0x9D, 0xCD, 0xB9, 0xD2, 0xB4, 0x9D, 0x84, 0x06, 0xB9,
    0x28, 0x83, 0x09, 0x02, 0x9D, 0x34, 0x05, 0x20, 0x51, 0xCC, 0x4C, 0x74,
];

/// 【F-A103 注】记录查表（旧 index×3 形=超读注——真消费形=$BF0B/$BF13 双字节表
/// （$4D 索引）；本函数保留为 $BEFB 窗直读注（三跳变体实验面））。
pub fn palette_record_rgb(index: u8) -> [u8; 3] {
    let i = (index as usize & 31) * 3;
    [PALETTE_RECORDS[i], PALETTE_RECORDS[i + 1], PALETTE_RECORDS[i + 2]]
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    fn argv(items: &[&str]) -> Vec<String> {
        std::iter::once("render-frame").chain(items.iter().copied()).map(String::from).collect()
    }

    #[test]
    fn parse_cli_defaults_preserve_history() {
        let c = parse_cli(&argv(&["4600", "l4_clear-inject"])).expect("默认用法可解析");
        assert_eq!(c.frame, 4600);
        assert_eq!(c.name, "l4_clear-inject");
        assert_eq!(c.scene, DEFAULT_SCENE, "缺省场景=gameplay-stage1（历史行为）");
        assert_eq!(c.base, 0);
        assert_eq!(c.out_dir, None, "缺省 out-dir=repo_root/oracle/screens/actual");
        assert_eq!(c.log_path, None, "缺省 log=<out_dir>/.render.log");
    }

    #[test]
    fn parse_cli_scene_base_and_new_flags() {
        let c = parse_cli(&argv(&[
            "4600", "l4_clear-inject", "stg03-timeline", "--base", "1",
            "--out-dir", "target/l4/out", "--log-path", "target/l4/out/run.log",
        ]))
        .expect("新参数可解析");
        assert_eq!(c.scene, "stg03-timeline");
        assert_eq!(c.base, 1);
        assert_eq!(c.out_dir.as_deref(), Some("target/l4/out"));
        assert_eq!(c.log_path.as_deref(), Some("target/l4/out/run.log"));
    }

    #[test]
    fn parse_cli_rejects_missing_or_invalid() {
        assert!(parse_cli(&argv(&[])).is_err(), "缺帧号");
        assert!(parse_cli(&argv(&["4600"])).is_err(), "缺输出名");
        assert!(parse_cli(&argv(&["abc", "n"])).is_err(), "帧号非法");
        assert!(parse_cli(&argv(&["4600", "n", "--out-dir"])).is_err(), "--out-dir 缺值");
        assert!(parse_cli(&argv(&["4600", "n", "--base", "9"])).is_ok(), "--base 任意整数可解析");
        assert!(parse_cli(&argv(&["4600", "n", "--base", "x"])).is_err(), "--base 非法");
    }

    #[test]
    fn parse_cli_legacy_value_flags_skip_their_values() {
        // F97-3：legacy flag 的值不得被当作位置参数（否则 scene 被覆盖成路径）
        let c = parse_cli(&argv(&[
            "4600",
            "name",
            "--probe-file",
            "analysis/traces/x.probe.jsonl",
        ]))
        .expect("legacy 组合可解析");
        assert_eq!(c.scene, DEFAULT_SCENE, "probe 路径不得被当 scene（F97-3）");
        assert_eq!(c.probe_file.as_deref(), Some("analysis/traces/x.probe.jsonl"));

        let c2 = parse_cli(&argv(&[
            "4600",
            "name",
            "stg03-timeline",
            "--text-file",
            "phase.text.jsonl",
            "--stream-file",
            "stream.bin",
        ]))
        .expect("legacy + scene 可解析");
        assert_eq!(c2.scene, "stg03-timeline");
        assert_eq!(c2.text_file.as_deref(), Some("phase.text.jsonl"));
        assert_eq!(c2.stream_file.as_deref(), Some("stream.bin"));

        assert!(
            parse_cli(&argv(&["4600", "name", "--probe-file"])).is_ok(),
            "旧用法 legacy flag 缺值保持宽容（不 panic）"
        );
    }

    #[test]
    fn help_after_positionals_is_detected() {
        assert!(wants_help(&argv(&["4600", "n", "--help"])), "位置参数后的 --help 也须触发（F97-4）");
        assert!(wants_help(&argv(&["-h"])));
        assert!(!wants_help(&argv(&["4600", "n"])));
        assert!(
            parse_cli(&argv(&["4600", "n", "--help"])).is_ok(),
            "help 由 main 先于 parse_cli 处理，解析层不报错"
        );
    }

    #[test]
    fn lexical_normalize_and_repo_containment() {
        assert_eq!(
            lexical_normalize(Path::new("D:/repo/target/../out")),
            PathBuf::from("D:/repo/out")
        );
        let root = Path::new("D:/repo");
        assert!(within_repo(root, Path::new("D:/repo/target/x")), "仓库内");
        #[cfg(windows)]
        assert!(within_repo(root, Path::new("D:/REPO/Target/X")), "Windows 大小写不敏感");
        #[cfg(not(windows))]
        assert!(!within_repo(root, Path::new("D:/REPO/Target/X")), "POSIX 大小写敏感（F97-5）");
        assert!(!within_repo(root, Path::new("D:/repo_evil/x")), "前缀相似但非仓库内");
        assert!(!within_repo(root, Path::new("D:/outside/x")), "仓库外");
        assert!(
            !within_repo(root, &lexical_normalize(Path::new("D:/repo/../outside"))),
            ".. 越界须拒绝"
        );
    }

    #[test]
    fn resolve_explicit_relative_inside_and_outside() {
        let root = std::env::current_dir().expect("cwd");
        let root_real = real_path(&root).expect("仓库根真实路径");
        let inside = resolve_explicit(&root, "target/render-frame-cli-test", "--out-dir");
        assert!(inside.is_ok(), "仓库内相对路径应通过：{:?}", inside);
        assert!(within_repo(&root_real, &inside.unwrap()), "解析结果真实路径在仓库根下");
        let outside = resolve_explicit(&root, "../render-frame-cli-test-out", "--out-dir");
        assert!(outside.is_err(), "仓库外相对路径须 fail-closed");
        let abs_outside = if cfg!(windows) { "D:/outside-render-frame" } else { "/tmp/outside-render-frame" };
        assert!(resolve_explicit(&root, abs_outside, "--log-path").is_err(), "仓库外绝对路径须 fail-closed");
    }

    #[cfg(any(windows, unix))]
    #[test]
    fn junction_or_symlink_escape_rejected_by_real_path() {
        // F97-1：词法 containment 可被仓库内 junction/symlink 绕过 ⇒ 必须按真实路径拒绝
        let root = std::env::current_dir().expect("cwd");
        let base = root.join("target").join(format!("t100-junction-{}", std::process::id()));
        let outside = std::env::temp_dir().join(format!("gunnac-t100-outside-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::remove_dir_all(&outside);
        std::fs::create_dir_all(&base).expect("base");
        std::fs::create_dir_all(&outside).expect("outside");
        let link = base.join("escape");
        #[cfg(windows)]
        let made = {
            let st = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(&link)
                .arg(&outside)
                .status();
            matches!(st, Ok(s) if s.success())
        };
        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(&outside, &link).is_ok();
        if !made {
            let _ = std::fs::remove_dir_all(&base);
            let _ = std::fs::remove_dir_all(&outside);
            eprintln!("跳过 junction/symlink 逃逸测试：当前环境无创建权限（F97-7 允许说明）");
            return;
        }
        let r = resolve_explicit(&root, link.to_str().expect("utf8"), "--out-dir");
        assert!(
            r.is_err(),
            "仓库内 junction/symlink 指向仓库外 ⇒ 真实路径 containment 必须拒绝（F97-1）"
        );
        let r2 = resolve_explicit(&root, link.join("deep/out").to_str().expect("utf8"), "--log-path");
        assert!(r2.is_err(), "junction 下深层不存在路径也须拒绝");
        #[cfg(windows)]
        {
            let _ = std::fs::remove_dir(&link); // 删 junction 本体（reparse point）
        }
        #[cfg(unix)]
        {
            let _ = std::fs::remove_file(&link);
        }
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn sha256_known_vectors_and_sha12() {
        assert_eq!(
            sha256_hex12(b""),
            "e3b0c44298fc",
            "sha256(\"\")=e3b0c44298fc1c14… 取前 12 hex"
        );
        assert_eq!(
            sha256_hex12(b"abc"),
            "ba7816bf8f01",
            "sha256(\"abc\")=ba7816bf8f01cfea… 取前 12 hex"
        );
        assert_eq!(sha256_hex12(b"abc").len(), 12, "out=sha12 口径=12 hex（F97-8）");
        // 跨块（>64B）自检：长度 200 的确定性输入，两跑一致
        let buf: Vec<u8> = (0..200u32).map(|i| (i % 251) as u8).collect();
        assert_eq!(sha256_hex12(&buf), sha256_hex12(&buf));
    }

    #[test]
    fn sha256_stream_matches_oneshot() {
        // task-138：流式累加器必须与整读 sha 逐字节一致（分块边界 1/63/64/65/1MB-1/1MB）
        let data: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        let oneshot = sha256_hex12(&data);
        for chunk in [1usize, 63, 64, 65, 1_000, 65_536, 1_048_576] {
            let mut s = Sha256::new();
            for part in data.chunks(chunk) {
                s.update(part);
            }
            assert_eq!(hex12(s.finalize()), oneshot, "chunk={chunk} 流式 sha 不一致");
        }
    }

    #[test]
    fn sha256_file_stream_matches_and_cleans_up() {
        // task-138：文件流式 sha（1MB 缓冲）与整读一致；大 text 不整读。
        let data: Vec<u8> = (0..250_000u32).map(|i| (i % 251) as u8).collect();
        let p = std::env::temp_dir().join(format!("gunnac-t138-sha-{}", std::process::id()));
        std::fs::write(&p, &data).expect("write temp");
        let got = sha256_file_hex12(&p).expect("stream sha");
        assert_eq!(got, sha256_hex12(&data), "文件流式 sha 与整读不一致");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn text_stream_early_exit_prefix_equivalent() {
        // task-138：limit 内 prefix 与整读一致（生产者帧序单调前提；此处构造有序 JSONL）。
        let p = std::env::temp_dir().join(format!("gunnac-t138-text-{}", std::process::id()));
        let mut content = String::new();
        for (f, a, v, inv, seg) in [
            (1u32, 0x2006u16, 1u8, 7u32, 0u32),
            (2, 0x2007, 2, 7, 0),
            (2, 0x2006, 3, 7, 1),
            (3, 0x2007, 4, 7, 1),
            (10, 0x2006, 5, 8, 0),
        ] {
            content.push_str(&format!(
                "{{\"f\":{f},\"addr\":\"{a:04X}\",\"val\":{v},\"inv\":{inv},\"seg\":{seg}}}\n"
            ));
        }
        std::fs::write(&p, content).expect("write text");
        let ps = p.to_string_lossy().into_owned();

        let (limited, stats) = read_text_writes(&ps, 3);
        assert_eq!(stats.early_exit, true, "f>limit 须早退");
        assert_eq!(stats.in_frames_read, 10, "已读最高帧=触发早退的记录");
        assert_eq!(stats.text_records_read, 5);
        assert_eq!(limited.len(), 4, "limit 内保留 4 条记录");
        assert_eq!(limited.iter().map(|r| r.0).collect::<Vec<_>>(), vec![1, 2, 2, 3]);

        let (full, stats_full) = read_text_writes(&ps, u32::MAX);
        assert_eq!(stats_full.early_exit, false);
        assert_eq!(stats_full.text_records_read, 5);
        assert_eq!(&full[..limited.len()], &limited[..], "prefix 与整读一致（无需重解析语义）");

        // 空路径/不存在文件保持旧"空集"语义
        let (empty, st) = read_text_writes("", 3);
        assert!(empty.is_empty() && st.text_records_read == 0);
        let (missing, st2) = read_text_writes("target/definitely-missing-text-138.jsonl", 3);
        assert!(missing.is_empty() && st2.text_records_read == 0);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn env_summary_excludes_repo_root_and_is_sorted() {
        assert!(!ENV_KNOBS.contains(&"REPO_ROOT"), "REPO_ROOT 是输入定位非渲染参数");
        assert!(ENV_KNOBS.contains(&"QUAD") && ENV_KNOBS.contains(&"TEXTFALLBACK"));
        let s = env_explicit_summary();
        assert!(s.starts_with('{') && s.ends_with('}'));
        // 排序自检（若测试进程显式设置了多个 knob）
        let inner = &s[1..s.len() - 1];
        if inner.contains(',') {
            let keys: Vec<&str> = inner.split(',').map(|kv| kv.split('=').next().unwrap()).collect();
            let mut sorted = keys.clone();
            sorted.sort();
            assert_eq!(keys, sorted, "env 摘要须按键排序");
        }
    }

    #[test]
    fn usage_mentions_new_flags() {
        assert!(USAGE.contains("--out-dir") && USAGE.contains("--log-path"));
        assert!(USAGE.contains("退出码 2"));
    }
}
