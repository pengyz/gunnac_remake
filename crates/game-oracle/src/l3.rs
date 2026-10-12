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

/// 事件 golden 行（L3 trace 对拍）：kind=weapon（$34/$33 转移）或 kind=bomb（$0190/$018F 转移）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEvent {
    Weapon(WeaponEvent),
    Bomb(BombEvent),
}

/// 武器状态转移（weapon_slot $34 / weapon_level $33）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponEvent {
    pub f: u32,
    pub before_slot: u8,
    pub before_level: u8,
    pub after_slot: u8,
    pub after_level: u8,
}

/// 弹药/弹型转移（bomb_type_index $0190 / bomb_ammo $018F，$0100 页游戏变量）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BombEvent {
    pub f: u32,
    pub before_type: u8,
    pub before_ammo: u8,
    pub before_idx: u8,
    pub after_type: u8,
    pub after_ammo: u8,
    pub after_idx: u8,
}

fn obj_num(obj: &[(String, Json)], key: &str) -> Result<i64, String> {
    obj.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| match v { Json::Num(n) => Some(*n as i64), _ => None })
        .ok_or_else(|| format!("缺字段 {key}"))
}

/// 解析事件 golden JSONL（weapon/bomb 双类）。
pub fn parse_weapon_events(jsonl: &str) -> Result<Vec<TraceEvent>, String> {
    let mut out = Vec::new();
    for (lineno, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let v = crate::l2::parse_json(line).map_err(|e| format!("第 {} 行: {e}", lineno + 1))?;
        let obj = match v { Json::Obj(o) => o, _ => return Err(format!("第 {} 行非对象", lineno + 1)) };
        let f = obj_num(&obj, "f")? as u32;
        let kind = match obj.iter().find(|(k, _)| k == "kind") {
            Some((_, Json::Str(s))) => s.clone(),
            _ => "weapon".to_string(), // 兼容 t31 旧格式（无 kind=weapon）
        };
        let before = match obj.iter().find(|(k, _)| k == "before") {
            Some((_, Json::Obj(o))) => o.clone(),
            _ => return Err(format!("第 {} 行缺 before", lineno + 1)),
        };
        let after = match obj.iter().find(|(k, _)| k == "after") {
            Some((_, Json::Obj(o))) => o.clone(),
            _ => return Err(format!("第 {} 行缺 after", lineno + 1)),
        };
        match kind.as_str() {
            "weapon" => out.push(TraceEvent::Weapon(WeaponEvent {
                f,
                before_slot: obj_num(&before, "slot")? as u8,
                before_level: obj_num(&before, "level")? as u8,
                after_slot: obj_num(&after, "slot")? as u8,
                after_level: obj_num(&after, "level")? as u8,
            })),
            "bomb" => out.push(TraceEvent::Bomb(BombEvent {
                f,
                before_type: obj_num(&before, "type")? as u8,
                before_ammo: obj_num(&before, "ammo")? as u8,
                before_idx: obj_num(&before, "idx").unwrap_or(0) as u8,
                after_type: obj_num(&after, "type")? as u8,
                after_ammo: obj_num(&after, "ammo")? as u8,
                after_idx: obj_num(&after, "idx").unwrap_or(0) as u8,
            })),
            other => return Err(format!("第 {} 行未知 kind {other}", lineno + 1)),
        }
    }
    Ok(out)
}

/// weapon 转移合法性：同型升级（含 FF→0C 回卷）/ 异型切换 / 重生复位（R-respawn-*）。
pub fn weapon_events_legal(events: &[TraceEvent]) -> Vec<String> {
    let mut problems = Vec::new();
    for e in events {
        let (f, bs, bl, as_, al) = match e {
            TraceEvent::Weapon(w) => (w.f, w.before_slot, w.before_level, w.after_slot, w.after_level),
            TraceEvent::Bomb(_) => continue,
        };
        let leveled = as_ == bs
            && (al == bl.wrapping_add(1) && !(bl == 0xFF && al != 0x0C)
                || (bl == 0xFF && al == 0x0C));
        let switched = as_ != bs && al == bl;
        let respawned = as_ == 0 && al == 0;
        if !(leveled || switched || respawned) {
            problems.push(format!(
                "f{f}: (slot={bs},level={bl})→(slot={as_},level={al}) 不属于 同型升级/异型切换/重生复位 任一合法步"
            ));
        }
    }
    problems
}

/// bomb 转移合法性（t38 FA-3 断言源；口径按 t36/spec-writer 原地更正）：
///
/// **弹数($018F)与弹种($0190)是两个独立维度，不存在"同帧同步"**——t22"三方同步"系 60 帧
/// probe 粒度的同簇观察；A 级反例链：f2174 弹数 3→4 → f2175 弹种 0→3 且弹数不变
/// （同帧 $34 0→3）。因此：
/// - 弹数递减 ⇒ 必须等于 R-fire-4 扣减（min(消耗表[idx], 弹药) 经 ED03 BCD 扣减）；
/// - 弹数回满 ⇒ 必须回 **4**（R-respawn-4 / N-0017 init）；
/// - 弹种切换 ⇒ 独立事件，如实入 golden（值域映射见 R-bomb-3，1=B/2=T 仍属推定不约束）。
pub fn bomb_events_legal(events: &[TraceEvent]) -> Vec<String> {
    use game_core::l2ports::{bcd_sub_ammo, CONSUMPTION_TABLE};
    let mut problems = Vec::new();
    for e in events {
        let b = match e {
            TraceEvent::Bomb(b) => *b,
            TraceEvent::Weapon(_) => continue,
        };
        if b.after_ammo == b.before_ammo {
            continue; // 弹种切换（或无事）：独立维度，不与弹数耦合断言
        }
        if b.after_ammo == 4 && b.after_ammo > b.before_ammo {
            continue; // 回满：R-respawn-4 / init ⇒ 4
        }
        let idx = b.before_idx.min(3) as usize;
        let cost = CONSUMPTION_TABLE[idx].min(b.before_ammo);
        let (expect, _) = bcd_sub_ammo(b.before_ammo, cost);
        if b.after_ammo != expect {
            problems.push(format!(
                "f{}: 弹数 {}→{} 不等于 R-fire-4 扣减（idx={} 消耗={} 期望 {}）",
                b.f, b.before_ammo, b.after_ammo, b.before_idx, cost, expect
            ));
        }
    }
    problems
}
