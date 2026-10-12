//! L3 帧级 trace 差分：game-core::player tick 序列 vs oracle probe trace（Mesen2 执行）。
//!
//! 对齐约定（940 帧 oracle 实证 0 失配，t24）：`state(N) = tick(state(N-1), held(N))`——
//! 输入当帧生效；帧计数 = 场景 DSL 的 inputPolled 序。
//! 差分面 = 逐帧 (player_x, player_y, x_subpix, y_subpix)；mode/ammo 属 H-15 未证域，
//! 不入比对面（见 docs/spec/player-movement.md §4）。

use crate::l2::Json;
use game_core::player::{tick, PlayerState};

/// golden 行：{f, held, x, y, sx, sy}（oracle/traces/<场景>.player.jsonl）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerFrame {
    pub f: u32,
    pub held: u8,
    pub x: u8,
    pub y: u8,
    pub sx: u8,
    pub sy: u8,
}

fn field_u64(obj: &[(String, Json)], key: &str) -> Result<u64, String> {
    obj.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| match v {
            Json::Num(n) => Some(*n as u64),
            _ => None,
        })
        .ok_or_else(|| format!("缺字段 {key}"))
}

/// 解析 L3 golden JSONL。
pub fn parse_player_frames(jsonl: &str) -> Result<Vec<PlayerFrame>, String> {
    let mut out = Vec::new();
    for (lineno, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v = crate::l2::parse_json(line).map_err(|e| format!("第 {} 行: {e}", lineno + 1))?;
        let obj = match v {
            Json::Obj(o) => o,
            _ => return Err(format!("第 {} 行非对象", lineno + 1)),
        };
        let f = field_u64(&obj, "f")?;
        let row = PlayerFrame {
            f: f as u32,
            held: field_u64(&obj, "held")? as u8,
            x: field_u64(&obj, "x")? as u8,
            y: field_u64(&obj, "y")? as u8,
            sx: field_u64(&obj, "sx")? as u8,
            sy: field_u64(&obj, "sy")? as u8,
        };
        out.push(row);
    }
    if out.len() < 2 {
        return Err("golden 至少 2 帧（种子帧 + ≥1 比对帧）".into());
    }
    Ok(out)
}

/// 差分：种子 = 第 0 帧；此后逐帧 `tick(held(N))` 与 golden 比对。
/// 返回 (比对帧数, 失配明细)。
pub fn diff_player_trace(frames: &[PlayerFrame]) -> Result<(usize, Vec<String>), String> {
    let mut st = PlayerState {
        x: frames[0].x,
        y: frames[0].y,
        x_subpix: frames[0].sx,
        y_subpix: frames[0].sy,
        ..Default::default()
    };
    let mut problems = Vec::new();
    let mut compared = 0usize;
    for w in frames.windows(2) {
        let (prev, cur) = (w[0], w[1]);
        if cur.f != prev.f + 1 {
            // 帧号缺口 = 分段边界（死亡/重生等未证语义窗口已被抽取端剔除）：
            // 以新段首帧为种子（种子帧自身不比对），后续逐帧递推比对。
            st = PlayerState {
                x: cur.x, y: cur.y, x_subpix: cur.sx, y_subpix: cur.sy,
                ..Default::default()
            };
            continue;
        }
        st = tick(&st, cur.held);
        compared += 1;
        if (st.x, st.y, st.x_subpix, st.y_subpix) != (cur.x, cur.y, cur.sx, cur.sy) {
            problems.push(format!(
                "f{}（held={}）: sim=({},{},{},{}) oracle=({},{},{},{})",
                cur.f, cur.held, st.x, st.y, st.x_subpix, st.y_subpix, cur.x, cur.y, cur.sx, cur.sy
            ));
            if problems.len() >= 10 {
                problems.push("…（失配过多，截断显示）".to_string());
                break;
            }
        }
    }
    Ok((compared, problems))
}

/// 武器状态转移事件（L3 trace 对拍）：{f, before:{slot,level}, after:{slot,level}}。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponEvent {
    pub f: u32,
    pub before_slot: u8,
    pub before_level: u8,
    pub after_slot: u8,
    pub after_level: u8,
}

fn obj_num(obj: &[(String, Json)], key: &str) -> Result<i64, String> {
    obj.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| match v { Json::Num(n) => Some(*n as i64), _ => None })
        .ok_or_else(|| format!("缺字段 {key}"))
}

/// 解析武器事件 golden JSONL。
pub fn parse_weapon_events(jsonl: &str) -> Result<Vec<WeaponEvent>, String> {
    let mut out = Vec::new();
    for (lineno, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let v = crate::l2::parse_json(line).map_err(|e| format!("第 {} 行: {e}", lineno + 1))?;
        let obj = match v { Json::Obj(o) => o, _ => return Err(format!("第 {} 行非对象", lineno + 1)) };
        let f = obj_num(&obj, "f")?;
        let before = match obj.iter().find(|(k, _)| k == "before") {
            Some((_, Json::Obj(o))) => o.clone(),
            _ => return Err(format!("第 {} 行缺 before", lineno + 1)),
        };
        let after = match obj.iter().find(|(k, _)| k == "after") {
            Some((_, Json::Obj(o))) => o.clone(),
            _ => return Err(format!("第 {} 行缺 after", lineno + 1)),
        };
        out.push(WeaponEvent {
            f: f as u32,
            before_slot: obj_num(&before, "slot")? as u8,
            before_level: obj_num(&before, "level")? as u8,
            after_slot: obj_num(&after, "slot")? as u8,
            after_level: obj_num(&after, "level")? as u8,
        });
    }
    Ok(out)
}

/// 武器事件合法性：每个 trace 转移必须等于 pickup_handler 的某个合法步
/// （同型升级含回卷 / 异型切换）或 respawn 复位（slot=0, level=0，R-respawn-*）。
/// 返回违规明细（空=全合法）。
pub fn weapon_events_legal(events: &[WeaponEvent]) -> Vec<String> {
    let mut problems = Vec::new();
    for e in events {
        let leveled = e.after_slot == e.before_slot
            && (e.after_level == e.before_level.wrapping_add(1)
                && !(e.before_level == 0xFF && e.after_level != 0x0C)
                || (e.before_level == 0xFF && e.after_level == 0x0C));
        let switched = e.after_slot != e.before_slot && e.after_level == e.before_level;
        let respawned = e.after_slot == 0 && e.after_level == 0;
        if !(leveled || switched || respawned) {
            problems.push(format!(
                "f{}: (slot={},level={})→(slot={},level={}) 不属于 同型升级/异型切换/重生复位 任一合法步",
                e.f, e.before_slot, e.before_level, e.after_slot, e.after_level
            ));
        }
    }
    problems
}
