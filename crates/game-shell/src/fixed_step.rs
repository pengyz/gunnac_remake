//! 60Hz 固定步驱动（平台无关；时间只以整数纳秒注入）。
//!
//! - 逻辑帧率＝NES 60Hz（[game-core `TARGET_FPS`] 同源）；
//! - 平台单调时钟推进 ⇒ 本模块把流逝时间折算为「应推进的逻辑帧数」；
//! - 追帧上限防 spiral of death；时钟回拨⇒0 帧（fail-closed，不发明负帧）；
//! - 平台时间源（`Instant` 等）只允许出现在平台壳层；本 crate 只收整数 ns。

/// 逻辑帧时长（ns）：`1_000_000_000 / 60`（整数除 40ns/秒漂移，可忽略）。
pub const FRAME_NS: u64 = 1_000_000_000 / 60;

/// 单次采样最多推进的逻辑帧数（追帧上限；与 shell-win 现行 `FrameDriver` 同值）。
pub const MAX_CATCHUP_FRAMES: u32 = 4;

/// 60Hz 固定步折算器。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedStep {
    last_ns: u64,
    accum_ns: u64,
    max_frames: u32,
}

impl FixedStep {
    /// 以 `start_ns` 为基准（首个 `advance(start_ns)` ⇒ 0 帧）。
    pub const fn new(start_ns: u64) -> Self {
        Self { last_ns: start_ns, accum_ns: 0, max_frames: MAX_CATCHUP_FRAMES }
    }

    /// 自定义追帧上限（`max_frames` 至少 1）。
    pub const fn with_max_frames(start_ns: u64, max_frames: u32) -> Self {
        Self { last_ns: start_ns, accum_ns: 0, max_frames: if max_frames == 0 { 1 } else { max_frames } }
    }

    /// 推进到 `now_ns`，返回本次应执行的逻辑帧数（0..=`max_frames`）。
    ///
    /// 时钟回拨（`now_ns < last_ns`）⇒ 不累计、不推进（fail-closed）；
    /// 积压超过上限 ⇒ 丢弃剩余积压（不雪崩追赶）。
    pub fn advance(&mut self, now_ns: u64) -> u32 {
        if now_ns > self.last_ns {
            self.accum_ns = self.accum_ns.saturating_add(now_ns - self.last_ns);
        }
        self.last_ns = now_ns;
        let mut frames = 0u32;
        while self.accum_ns >= FRAME_NS && frames < self.max_frames {
            self.accum_ns -= FRAME_NS;
            frames += 1;
        }
        if self.accum_ns >= FRAME_NS {
            self.accum_ns = 0; // 超限积压丢弃（追帧上限语义）
        }
        frames
    }

    /// 最近一次采样时间（ns）。
    pub const fn last_ns(&self) -> u64 {
        self.last_ns
    }

    /// 当前未折算的余量（ns；不足一帧）。
    pub const fn accum_ns(&self) -> u64 {
        self.accum_ns
    }

    /// 追帧上限。
    pub const fn max_frames(&self) -> u32 {
        self.max_frames
    }
}

#[cfg(test)]
mod tests {
    use super::{FixedStep, FRAME_NS, MAX_CATCHUP_FRAMES};

    #[test]
    fn frame_ns_is_60hz() {
        assert_eq!(FRAME_NS, 16_666_666);
        // 60 帧累计与 1s 误差 ≤ 40ns（整数除）
        assert!((FRAME_NS * 60).abs_diff(1_000_000_000) <= 40);
    }

    #[test]
    fn advance_from_baseline_is_zero() {
        let mut s = FixedStep::new(1_000);
        assert_eq!(s.advance(1_000), 0, "同刻⇒0 帧");
        assert_eq!(s.accum_ns(), 0);
    }

    #[test]
    fn advance_accumulates_frames() {
        let mut s = FixedStep::new(0);
        assert_eq!(s.advance(FRAME_NS), 1, "1 帧时长⇒1 帧");
        assert_eq!(s.advance(FRAME_NS * 3), 2, "再 2 帧时长⇒2 帧");
        assert_eq!(s.accum_ns(), 0);
        // 半帧不推进
        assert_eq!(s.advance(FRAME_NS * 3 + FRAME_NS / 2), 0, "半帧⇒0 帧");
        assert_eq!(s.accum_ns(), FRAME_NS / 2);
    }

    #[test]
    fn catchup_is_capped_and_backlog_dropped() {
        let mut s = FixedStep::new(0);
        assert_eq!(s.advance(FRAME_NS * 100), MAX_CATCHUP_FRAMES, "追帧上限={MAX_CATCHUP_FRAMES}");
        assert_eq!(s.accum_ns(), 0, "超限积压丢弃");
    }

    #[test]
    fn clock_regression_does_not_advance() {
        let mut s = FixedStep::new(FRAME_NS * 10);
        assert_eq!(s.advance(0), 0, "时钟回拨⇒0 帧（fail-closed）");
        assert_eq!(s.advance(0), 0);
        assert_eq!(s.advance(FRAME_NS), 1, "回拨后按新基准继续");
    }

    #[test]
    fn custom_cap() {
        let mut s = FixedStep::with_max_frames(0, 2);
        assert_eq!(s.advance(FRAME_NS * 10), 2);
        assert_eq!(s.max_frames(), 2);
    }
}
