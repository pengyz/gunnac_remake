//! game-oracle —— 差分宿主（L2 子程序级差分先落地；L3/L5 后续）。
//!
//! 契约（AGENTS.md §2 `oracle/` 行）：比对逻辑**不得为"通过测试"放宽阈值**；
//! 白名单唯一例外 = `gates/visual_whitelist.json`（L4，需用户确认）。
//!
//! - `cpu6502`：6502 CPU 核（仅 CPU+RAM，PPU/APU stub），执行 ROM 原字节子程序；
//! - `l2`：case 格式（JSONL）+ 三方裁定（golden / ROM 执行 / game-core 同名函数）。

#![forbid(unsafe_code)]

pub mod cpu6502;
pub mod l2;
