//! game-oracle —— 差分宿主占位（L1–L3/L5 比对宿主，S6 落地）。
//!
//! 契约（AGENTS.md §2 `oracle/` 行）：比对逻辑**不得为"通过测试"放宽阈值**；
//! 白名单唯一例外 = `gates/visual_whitelist.json`（L4，需用户确认）。
//! 本占位不放任何比对/阈值逻辑 —— 差分实现必须与 verifier 的裁定同 commit 落地。

#![forbid(unsafe_code)]

/// 差分宿主骨架（占位）。真实现到来前不提供任何"判定"入口，防止被误用为放水通道。
#[derive(Debug, Default)]
pub struct DiffHost;

impl DiffHost {
    pub const fn new() -> Self {
        Self
    }
}
