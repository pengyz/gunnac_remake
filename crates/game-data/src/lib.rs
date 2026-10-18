//! game-data —— 纯解析（无渲染）：ROM 原始字节 → 已证结构。
//!
//! 红线（AGENTS.md §2）：纯解析、无 I/O 到游戏逻辑、无渲染；解析语义只允许
//! 已证实的结构规则（docs/findings/ 证据指针进各模块文档），禁止猜测性结论。

#![forbid(unsafe_code)]

pub mod chr;
pub mod rom;
pub mod text;
