//! Audio 流消费（契约已接线；v1 无生产者）。
//!
//! game-core 侧 APU 写流＝`ApuWriteStream`（`(frame, reg, value)` 三元组，写序保真，
//! audio-golden.md 同源）。本模块每逻辑帧把流事件**保序**转发给 `game_platform::Audio`：
//! 逐事件 `write_apu(event.frame, event.reg as u16, event.value)` → `end_frame(本逻辑帧)`。
//!
//! **v1 生产者未接**：APU 写侧（AUD-2/3 音频驱动内部体、音效/音乐触发）未证，
//! `StageEffects::far_call_requests` 到音频写的映射属猜测 ⇒ 不接线（未证不发明）。
//! `stream_mut()` 是后续已证音频写侧的接线口，也供契约测试注入事件。

use game_core::ApuWriteStream;
use game_platform::Audio;

/// FNV-1a 64 偏移基。
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64 素数。
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *hash ^= u64::from(b);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

/// ApuWrite 流 → Audio 的消费管道（平台无关）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioPipe {
    stream: ApuWriteStream,
    flushed_events: u64,
    write_hash: u64,
}

impl AudioPipe {
    /// 空管道。
    pub fn new() -> Self {
        Self { stream: ApuWriteStream::default(), flushed_events: 0, write_hash: FNV_OFFSET }
    }

    /// 写入侧接线口（已证音频生产者后续接入；契约测试注入用）。
    pub fn stream_mut(&mut self) -> &mut ApuWriteStream {
        &mut self.stream
    }

    /// 待消费事件数。
    pub fn queued(&self) -> usize {
        self.stream.events.len()
    }

    /// 已转发事件总数（确定性签名用）。
    pub const fn flushed_events(&self) -> u64 {
        self.flushed_events
    }

    /// 已转发事件序 hash（FNV-1a；确定性签名用）。
    pub const fn write_hash(&self) -> u64 {
        self.write_hash
    }

    /// 消费本帧流：逐事件保序转发 `write_apu`，最后 `end_frame(frame)`。返回转发事件数。
    pub fn flush(&mut self, audio: &mut dyn Audio, frame: u32) -> usize {
        let events = core::mem::take(&mut self.stream.events);
        let n = events.len();
        for e in &events {
            audio.write_apu(e.frame, u16::from(e.reg), e.value);
            hash_bytes(&mut self.write_hash, &e.frame.to_le_bytes());
            hash_bytes(&mut self.write_hash, &[e.reg, e.value]);
            self.flushed_events += 1;
        }
        audio.end_frame(frame);
        n
    }
}

impl Default for AudioPipe {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::AudioPipe;
    use game_platform::Audio;

    #[derive(Default)]
    struct RecordingAudio {
        writes: Vec<(u32, u16, u8)>,
        frames: Vec<u32>,
    }

    impl Audio for RecordingAudio {
        fn write_apu(&mut self, frame: u32, reg: u16, value: u8) {
            self.writes.push((frame, reg, value));
        }
        fn end_frame(&mut self, frame: u32) {
            self.frames.push(frame);
        }
    }

    #[test]
    fn consumes_stream_in_order_and_ends_frame() {
        let mut pipe = AudioPipe::new();
        assert_eq!(pipe.queued(), 0);
        pipe.stream_mut().write(1, 0x00, 0x30);
        pipe.stream_mut().write(1, 0x01, 0x08);
        pipe.stream_mut().write(2, 0x15, 0x0F);
        assert_eq!(pipe.queued(), 3);

        let mut audio = RecordingAudio::default();
        let n = pipe.flush(&mut audio, 1);
        assert_eq!(n, 3, "本帧转发 3 事件");
        assert_eq!(
            audio.writes,
            vec![(1, 0x0000, 0x30), (1, 0x01, 0x08), (2, 0x15, 0x0F)],
            "写序=流序，frame 用事件自身帧"
        );
        assert_eq!(audio.frames, vec![1], "end_frame=本逻辑帧（每帧恰一次）");
        assert_eq!(pipe.queued(), 0, "流已排空");
        assert_eq!(pipe.flushed_events(), 3);
        assert_ne!(pipe.write_hash(), super::FNV_OFFSET, "写 hash 已更新");
    }

    #[test]
    fn empty_stream_still_ends_frame() {
        let mut pipe = AudioPipe::new();
        let mut audio = RecordingAudio::default();
        assert_eq!(pipe.flush(&mut audio, 7), 0);
        assert!(audio.writes.is_empty());
        assert_eq!(audio.frames, vec![7], "空流也推进 end_frame（帧同步）");
        assert_eq!(pipe.write_hash(), super::FNV_OFFSET, "空流 hash 不变");
    }
}
