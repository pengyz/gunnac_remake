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

/// 弹对象轨迹帧（L3 对拍；slot 全覆盖+帧降采样口径见 tools/diff_traces.py 抽取注释）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShotFrame {
    pub f: u32,
    pub slot: u8,
    /// 槽内弹生命周期序号（0=非轨迹行；同 win=同弹，跨 win 重播种）
    pub win: u32,
    pub lifetime: u8,
    pub script_ptr: u8,
    pub pos_x: u8,
    pub pos_y: u8,
    pub motion_code: u8,
    pub shot_type: u8,
}

/// 解析弹轨迹 golden JSONL（逐槽行）。
pub fn parse_shot_frames(jsonl: &str) -> Result<Vec<ShotFrame>, String> {
    let mut out = Vec::new();
    for (lineno, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v = crate::l2::parse_json(line).map_err(|e| format!("第 {} 行: {e}", lineno + 1))?;
        let obj = match v { Json::Obj(o) => o, _ => return Err(format!("第 {} 行非对象", lineno + 1)) };
        out.push(ShotFrame {
            f: obj_num(&obj, "f")? as u32,
            slot: obj_num(&obj, "slot")? as u8,
            win: obj_num(&obj, "win").unwrap_or(0) as u32,
            lifetime: obj_num(&obj, "lifetime")? as u8,
            script_ptr: obj_num(&obj, "script_ptr")? as u8,
            pos_x: obj_num(&obj, "pos_x")? as u8,
            pos_y: obj_num(&obj, "pos_y")? as u8,
            motion_code: obj_num(&obj, "motion_code")? as u8,
            shot_type: obj_num(&obj, "shot_type")? as u8,
        });
    }
    if out.is_empty() {
        return Err("弹轨迹 golden 为空".into());
    }
    Ok(out)
}

/// 弹轨迹对拍（逐槽存活段）：出生帧为种子，此后逐帧 tick（脚本机+LUT 累加）比对；
/// 死亡帧（tick 返回 None）为未证域边界，不入比对面。帧缺口=分段重播种。
pub fn diff_shot_trajectory(frames: &[ShotFrame]) -> Result<(usize, Vec<String>), String> {
    use game_core::shot::{tick, ShotObject};
    use std::collections::BTreeMap;

    fn advance(o: &mut ShotObject, n: u32) -> bool {
        for _ in 0..n {
            if tick(o).is_none() {
                return false;
            }
        }
        true
    }

    let mut by_slot: BTreeMap<(u8, u32), Vec<&ShotFrame>> = BTreeMap::new();
    for r in frames.iter().filter(|r| r.lifetime != 0 && r.win > 0) {
        by_slot.entry((r.slot, r.win)).or_default().push(r);
    }
    let mut problems = Vec::new();
    let mut compared = 0usize;
    for ((_slot, _win), rows) in &by_slot {
        let mut obj: Option<ShotObject> = None;
        for w in rows.windows(2) {
            let (prev, cur) = (*w[0], *w[1]);
            if cur.f <= prev.f {
                return Err(format!("槽 {} 帧序乱序", cur.slot));
            }
            let ok = {
                let o = obj.get_or_insert(ShotObject {
                    lifetime: prev.lifetime, script_ptr: prev.script_ptr,
                    pos_x: prev.pos_x, pos_y: prev.pos_y,
                    motion_code: prev.motion_code, shot_type: prev.shot_type,
                });
                advance(o, cur.f - prev.f)
            };
            if !ok {
                problems.push(format!("slot={} f{}: 对象提前死亡（比对窗口应止于死亡帧前）", cur.slot, cur.f));
                obj = None;
                continue;
            }
            compared += 1;
            let o = obj.as_ref().expect("已播种");
            let got = (o.lifetime, o.script_ptr, o.pos_x, o.pos_y, o.motion_code, o.shot_type);
            let exp = (cur.lifetime, cur.script_ptr, cur.pos_x, cur.pos_y, cur.motion_code, cur.shot_type);
            if got != exp {
                problems.push(format!("slot={} f{}: sim={:?} oracle={:?}", cur.slot, cur.f, got, exp));
                if problems.len() >= 10 {
                    problems.push("…（失配过多，截断显示）".to_string());
                    return Ok((compared, problems));
                }
            }
        }
    }
    Ok((compared, problems))
}

/// 槽位分配语义断言（R-shot-gen-1..3 + t48 首次适应）：出生帧的槽集必须等于
/// 当帧状态向量下首次适应的前 |S| 个空槽（shot_slot_alloc 同源）。
/// 同帧既有出生又有死亡的帧跳过（帧内生成/回收顺序未证，不发明）。
pub fn alloc_events_legal(frames: &[ShotFrame]) -> Vec<String> {
    use game_core::shot::{shot_slot_alloc, SlotAlloc};
    use std::collections::{BTreeMap, BTreeSet};
    let mut state: BTreeMap<u32, BTreeMap<u8, u8>> = BTreeMap::new(); // f → slot → lifetime
    for r in frames {
        state.entry(r.f).or_default().insert(r.slot, r.lifetime);
    }
    let mut problems = Vec::new();
    let mut prev: BTreeMap<u8, u8> = BTreeMap::new();
    for (f, cur) in &state {
        if cur.len() != 20 {
            continue; // 非全向量帧（窗口采样帧）不断言分配
        }
        let mut spawns = BTreeSet::new();
        let mut deaths = BTreeSet::new();
        for (slot, lt) in cur {
            let before = prev.get(slot).copied().unwrap_or(0);
            if before == 0 && *lt != 0 {
                spawns.insert(*slot);
            } else if before != 0 && *lt == 0 {
                deaths.insert(*slot);
            }
        }
        if !spawns.is_empty() && deaths.is_empty() {
            // 出生帧：首次适应在生成前状态上取 |S| 个空槽，结果集合必须 == spawns
            // （真值源=抽取端补的出生前帧全向量 prev；同帧死亡跳过=帧内顺序未证）
            let mut probe = [0u8; 0x100];
            for (slot, lt) in prev.iter() {
                probe[*slot as usize] = *lt;
            }
            let mut expected = BTreeSet::new();
            for _ in 0..spawns.len() {
                match shot_slot_alloc(&probe) {
                    SlotAlloc::Found { slot, .. } => {
                        expected.insert(slot);
                        probe[slot as usize] = 1;
                    }
                    SlotAlloc::Full => break,
                }
            }
            if expected != spawns {
                problems.push(format!(
                    "f{}: 出生槽集 {:?} ≠ 首次适应期望 {:?}（槽位分配语义违例）",
                    f, spawns, expected
                ));
            }
        }
        prev = cur.clone();
    }
    problems
}

/// 寿命递减断言（t52 F2b 语义）：同窗（slot,win）内寿命逐帧递减；归 1 帧装载时长表值
/// （refill 集合 = C-action-durations 值域）。违规明细空=全合法。
pub fn lifetime_decrements_legal(frames: &[ShotFrame]) -> Vec<String> {
    use std::collections::{BTreeMap, BTreeSet};
    let durs: BTreeSet<u8> = game_core::shot::ACTION_DURATIONS.iter().copied().collect();
    let mut by_win: BTreeMap<(u8, u32), Vec<&ShotFrame>> = BTreeMap::new();
    for r in frames.iter().filter(|r| r.lifetime != 0 && r.win > 0) {
        by_win.entry((r.slot, r.win)).or_default().push(r);
    }
    let mut problems = Vec::new();
    for ((slot, win), rows) in &by_win {
        for w in rows.windows(2) {
            let (prev, cur) = (*w[0], *w[1]);
            let gap = cur.f as i64 - prev.f as i64;
            if gap <= 0 {
                problems.push(format!("slot={slot} win={win} f{}: 帧序非正", cur.f));
                continue;
            }
            // 非确定行走：v>1 递减；v==1 装载时长表值
            let mut poss: BTreeSet<u8> = BTreeSet::new();
            poss.insert(prev.lifetime);
            for _ in 0..gap {
                let mut nxt = BTreeSet::new();
                for v in poss {
                    if v > 1 {
                        nxt.insert(v - 1);
                    } else if v == 1 {
                        nxt.extend(durs.iter().copied());
                    }
                }
                poss = nxt;
            }
            if !poss.contains(&cur.lifetime) {
                problems.push(format!(
                    "slot={slot} win={win} f{}: 寿命 {}→{} 违反递减/装载语义（t52 F2b）",
                    cur.f, prev.lifetime, cur.lifetime
                ));
            }
        }
    }
    problems
}

/// 敌对象 churn 行（$1C 网格 28 槽快照；R-enemy-6 活动段）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyRow {
    pub f: u32,
    pub state: Vec<u8>, // $0400 类型/状态
    pub x: Vec<u8>,     // $04E0 x 坐标
    pub y: Vec<u8>,     // $04FC y 坐标
}

fn hex_bytes(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("hex 串奇数长度".into());
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

/// 解析 enemy churn golden JSONL。
pub fn parse_enemy_rows(jsonl: &str) -> Result<Vec<EnemyRow>, String> {
    let mut out = Vec::new();
    for (lineno, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v = crate::l2::parse_json(line).map_err(|e| format!("第 {} 行: {e}", lineno + 1))?;
        let obj = match v { Json::Obj(o) => o, _ => return Err(format!("第 {} 行非对象", lineno + 1)) };
        let get = |k: &str| -> Result<Vec<u8>, String> {
            match obj.iter().find(|(kk, _)| kk == k) {
                Some((_, Json::Str(s))) => hex_bytes(s),
                _ => Err(format!("缺字段 {k}")),
            }
        };
        out.push(EnemyRow {
            f: obj_num(&obj, "f")? as u32,
            state: get("state")?,
            x: get("x")?,
            y: get("y")?,
        });
    }
    if out.is_empty() {
        return Err("enemy churn golden 为空".into());
    }
    Ok(out)
}

/// R-enemy-4 坐标钳断言（A 级，**限定写路径**）：槽 0=玩家（B3 绑定 + R-move-3/4 钳
/// $0E-$F3/$14-$DF 双证）逐帧断言；其余槽的坐标写路径含 W12 未证桥（VM $A4 → $04xx
/// 写链未考），实测存在界外放置/中间态（fire-hold f1304/f1368）——**不入断言面**
/// （规则限定：钳仅作用于已证移动写路径；全槽断言待 W12 桥取证后升格，报 spec-writer）。
/// 返回违规明细（空=全合法）。
pub fn enemy_clamp_legal(rows: &[EnemyRow]) -> Vec<String> {
    let mut problems = Vec::new();
    for r in rows {
        if r.state.len() != 28 || r.x.len() != 28 || r.y.len() != 28 {
            problems.push(format!("f{}: 网格行尺寸非 28 槽", r.f));
            continue;
        }
        for s in 0..1usize {
            if r.x[s] == 0 && r.y[s] == 0 {
                continue;
            }
            if r.x[s] < 0x0E || r.x[s] > 0xF3 {
                problems.push(format!("f{}: 槽{} x=${:02X} 越 R-enemy-4 钳 $0E-$F3", r.f, s, r.x[s]));
            }
            if r.y[s] < 0x14 || r.y[s] > 0xDF {
                problems.push(format!("f{}: 槽{} y=${:02X} 越 R-enemy-4 钳 $14-$DF", r.f, s, r.y[s]));
            }
            if problems.len() >= 10 {
                problems.push("…（截断）".to_string());
                return problems;
            }
        }
    }
    problems
}
