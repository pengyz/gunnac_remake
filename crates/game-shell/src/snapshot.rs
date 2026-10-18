//! 存档缝占位编解码（`game_platform::Storage` 的宿主快照格式 v1）。
//!
//! **占位语义**：原版存档语义（电池存档/密码）未取证 ⇒ 本格式只是宿主内部快照，
//! 非游戏行为、不自动保存（平台按用户/生命周期显式调用）。格式冻结为 v1：
//!
//! ```text
//! magic 4B "GNSH" | version 1B (=1) | mode_tag 1B | stage 1B | flags 1B |
//! player 6B (x,y,x_subpix,y_subpix,bomb_ammo,action_mode) |
//! persist 10B (money.digits 7B, weapon_slot, weapon_level, bomb_ammo)
//! ```
//!
//! 不含：虚拟时钟帧号（`FrameClock` 无回写 API，未申请 core 只读接口）、引擎 `StageState`
//! （含 Vec 内部态，格式未锁定）——完整存档=后续步，超出 v1 范围（显式缺口，不发明）。
//!
//! **v1 只做格式 fail-closed**（magic/version/长度/flags/mode tag）；**不做语义校验**
//! （如 `1<=stage<=8`、mode×stage 不可能组合）——语义校验/宿主状态一致性=后续卡加固项
//! （复核 S7 已登记）；`restore` 不恢复 `prev_buttons`/`pacer`/`FrameClock`（显式缺口）。

use game_core::flow::{Mode, PersistData};
use game_core::player::PlayerState;
use game_core::ui::MoneyCounter;

/// 快照魔数（`GNSH`）。
pub const SNAPSHOT_MAGIC: [u8; 4] = *b"GNSH";
/// 快照格式版本（变更须升版本并保留旧版解码=不破坏已有存档）。
pub const SNAPSHOT_VERSION: u8 = 1;
/// v1 快照定长（4+1+1+1+1+6+10）。
pub const SNAPSHOT_LEN: usize = 24;

/// flags 字节位：bit0=已通关（结局到达）。
const FLAG_ALL_CLEAR: u8 = 0x01;

/// 宿主最小快照（占位；只含宿主/flow/player 已证状态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostSnapshot {
    /// 粗粒度 flow 模式。
    pub mode: Mode,
    /// 关号（`$180`，1..=8）。
    pub stage: u8,
    /// 已通关（`AllClear` 边已走）。
    pub all_clear: bool,
    /// 玩家状态（全部字段）。
    pub player: PlayerState,
    /// 跨模式保持数据（`$34/$33/$018F/$E0-$E6`）。
    pub persist: PersistData,
}

/// 模式 → 一字节 tag（宿主内部格式，非游戏语义）。
pub const fn mode_tag(mode: Mode) -> u8 {
    match mode {
        Mode::Boot => 0,
        Mode::Black => 1,
        Mode::Title => 2,
        Mode::Menu => 3,
        Mode::Shop => 4,
        Mode::ShopInfo => 5,
        Mode::Transition => 6,
        Mode::StageStart => 7,
        Mode::Combat => 8,
        Mode::DeathRespawn => 9,
        Mode::GameOverPending => 10,
        Mode::GameOverMenu => 11,
        Mode::StageClear => 12,
        Mode::Ending => 13,
    }
}

/// tag → 模式（未知 tag ⇒ None，fail-closed）。
pub const fn mode_from_tag(tag: u8) -> Option<Mode> {
    Some(match tag {
        0 => Mode::Boot,
        1 => Mode::Black,
        2 => Mode::Title,
        3 => Mode::Menu,
        4 => Mode::Shop,
        5 => Mode::ShopInfo,
        6 => Mode::Transition,
        7 => Mode::StageStart,
        8 => Mode::Combat,
        9 => Mode::DeathRespawn,
        10 => Mode::GameOverPending,
        11 => Mode::GameOverMenu,
        12 => Mode::StageClear,
        13 => Mode::Ending,
        _ => return None,
    })
}

/// 编码快照（定长 `SNAPSHOT_LEN` 字节）。
pub fn encode_snapshot(snapshot: &HostSnapshot) -> Vec<u8> {
    let mut out = Vec::with_capacity(SNAPSHOT_LEN);
    out.extend_from_slice(&SNAPSHOT_MAGIC);
    out.push(SNAPSHOT_VERSION);
    out.push(mode_tag(snapshot.mode));
    out.push(snapshot.stage);
    out.push(if snapshot.all_clear { FLAG_ALL_CLEAR } else { 0 });
    out.extend_from_slice(&[
        snapshot.player.x,
        snapshot.player.y,
        snapshot.player.x_subpix,
        snapshot.player.y_subpix,
        snapshot.player.bomb_ammo,
        snapshot.player.action_mode,
    ]);
    out.extend_from_slice(&snapshot.persist.money.digits);
    out.push(snapshot.persist.weapon_slot);
    out.push(snapshot.persist.weapon_level);
    out.push(snapshot.persist.bomb_ammo);
    debug_assert_eq!(out.len(), SNAPSHOT_LEN);
    out
}

/// 解码快照（magic/version/长度/未知 tag 任一不合法 ⇒ None=fail-closed，不猜测）。
pub fn decode_snapshot(bytes: &[u8]) -> Option<HostSnapshot> {
    if bytes.len() != SNAPSHOT_LEN {
        return None;
    }
    if bytes[0..4] != SNAPSHOT_MAGIC {
        return None;
    }
    if bytes[4] != SNAPSHOT_VERSION {
        return None;
    }
    let mode = mode_from_tag(bytes[5])?;
    let stage = bytes[6];
    let all_clear = bytes[7] & FLAG_ALL_CLEAR != 0;
    if bytes[7] & !FLAG_ALL_CLEAR != 0 {
        return None; // 未定义 flags 位 ⇒ fail-closed（防格式漂移误读）
    }
    let player = PlayerState {
        x: bytes[8],
        y: bytes[9],
        x_subpix: bytes[10],
        y_subpix: bytes[11],
        bomb_ammo: bytes[12],
        action_mode: bytes[13],
    };
    let mut digits = [0u8; 7];
    digits.copy_from_slice(&bytes[14..21]);
    let persist = PersistData {
        money: MoneyCounter { digits },
        weapon_slot: bytes[21],
        weapon_level: bytes[22],
        bomb_ammo: bytes[23],
    };
    Some(HostSnapshot { mode, stage, all_clear, player, persist })
}

#[cfg(test)]
mod tests {
    use super::{
        decode_snapshot, encode_snapshot, mode_from_tag, mode_tag, HostSnapshot, SNAPSHOT_LEN,
        SNAPSHOT_MAGIC, SNAPSHOT_VERSION,
    };
    use game_core::flow::{Mode, PersistData};
    use game_core::player::PlayerState;
    use game_core::ui::MoneyCounter;

    fn sample() -> HostSnapshot {
        HostSnapshot {
            mode: Mode::Combat,
            stage: 3,
            all_clear: false,
            player: PlayerState { x: 128, y: 160, x_subpix: 96, y_subpix: 208, bomb_ammo: 4, action_mode: 0 },
            persist: PersistData {
                money: MoneyCounter { digits: [0, 0, 0, 1, 2, 3, 4] },
                weapon_slot: 2,
                weapon_level: 5,
                bomb_ammo: 4,
            },
        }
    }

    #[test]
    fn snapshot_roundtrip() {
        let s = sample();
        let bytes = encode_snapshot(&s);
        assert_eq!(bytes.len(), SNAPSHOT_LEN);
        assert_eq!(&bytes[0..4], &SNAPSHOT_MAGIC);
        assert_eq!(bytes[4], SNAPSHOT_VERSION);
        assert_eq!(decode_snapshot(&bytes), Some(s), "编解码往返");
    }

    #[test]
    fn snapshot_fail_closed_on_corruption() {
        let bytes = encode_snapshot(&sample());
        assert_eq!(decode_snapshot(&[]), None, "空");
        assert_eq!(decode_snapshot(&bytes[..23]), None, "截断");
        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        assert_eq!(decode_snapshot(&bad_magic), None, "魔数错");
        let mut bad_ver = bytes.clone();
        bad_ver[4] = 99;
        assert_eq!(decode_snapshot(&bad_ver), None, "版本未知");
        let mut bad_tag = bytes.clone();
        bad_tag[5] = 200;
        assert_eq!(decode_snapshot(&bad_tag), None, "模式 tag 未知");
        let mut bad_flags = bytes.clone();
        bad_flags[7] = 0x80;
        assert_eq!(decode_snapshot(&bad_flags), None, "未定义 flags 位");
    }

    #[test]
    fn all_clear_flag_roundtrip() {
        let mut s = sample();
        s.all_clear = true;
        s.mode = Mode::Ending;
        let decoded = decode_snapshot(&encode_snapshot(&s)).expect("解码");
        assert!(decoded.all_clear);
        assert_eq!(decoded.mode, Mode::Ending);
    }

    #[test]
    fn mode_tags_are_bijective() {
        let modes = [
            Mode::Boot,
            Mode::Black,
            Mode::Title,
            Mode::Menu,
            Mode::Shop,
            Mode::ShopInfo,
            Mode::Transition,
            Mode::StageStart,
            Mode::Combat,
            Mode::DeathRespawn,
            Mode::GameOverPending,
            Mode::GameOverMenu,
            Mode::StageClear,
            Mode::Ending,
        ];
        let mut tags = Vec::new();
        for m in modes {
            let tag = mode_tag(m);
            assert_eq!(mode_from_tag(tag), Some(m), "tag 双向一致: {m:?}");
            tags.push(tag);
        }
        tags.sort_unstable();
        tags.dedup();
        assert_eq!(tags.len(), modes.len(), "tag 唯一");
    }
}
