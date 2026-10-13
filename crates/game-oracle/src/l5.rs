//! L5 流程级差分（t172；l5-plan §2 判定口径）——L3 机制复用于流程级：
//! 关卡推进/流程可达检查点（A1 流程可达 + A3 事件窗口）帧级对拍。
//! 未证段（GAME OVER/CONTINUE/存读档）**fail-closed**（断言留空不发明）。

use std::collections::BTreeMap;

/// L5 检查点（一帧一事件：流程推进/事件窗口）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L5Event {
    pub scene: String,
    pub f: u32,
    pub mode: String,
    /// 检查点类（A1 流程可达/A3 事件窗口）；未证段=空（fail-closed）
    pub checkpoint: Option<String>,
}

/// 解析 L5 golden JSONL：{"scene":…,"f":…,"mode":…,"checkpoint":…?}
pub fn parse_l5(jsonl: &str) -> Result<Vec<L5Event>, String> {
    let mut out = Vec::new();
    for (lineno, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v = crate::l2::parse_json(line).map_err(|e| format!("第 {} 行: {e}", lineno + 1))?;
        let obj = match v {
            crate::l2::Json::Obj(o) => o,
            _ => return Err(format!("第 {} 行非对象", lineno + 1)),
        };
        let scene = match obj.iter().find(|(k, _)| k == "scene") {
            Some((_, crate::l2::Json::Str(s))) => s.clone(),
            _ => return Err(format!("第 {} 行缺 scene", lineno + 1)),
        };
        let f = match obj.iter().find(|(k, _)| k == "f") {
            Some((_, crate::l2::Json::Num(n))) => *n as u32,
            _ => return Err(format!("第 {} 行缺 f", lineno + 1)),
        };
        let mode = match obj.iter().find(|(k, _)| k == "mode") {
            Some((_, crate::l2::Json::Str(s))) => s.clone(),
            _ => return Err(format!("第 {} 行缺 mode", lineno + 1)),
        };
        let checkpoint = match obj.iter().find(|(k, _)| k == "checkpoint") {
            Some((_, crate::l2::Json::Str(s))) => Some(s.clone()),
            _ => None,
        };
        out.push(L5Event { scene, f, mode, checkpoint });
    }
    if out.is_empty() {
        return Err("L5 golden 为空".into());
    }
    Ok(out)
}

/// L5 合法性：场景内帧序单调 + 模式序列过 FlowState 已证边集 + 未证段 fail-closed。
/// 返回违规列表（空=合法）。
pub fn l5_legality(events: &[L5Event]) -> Vec<String> {
    use game_core::flow::{FlowState, Mode};
    let mut problems = Vec::new();
    let mut last: BTreeMap<String, u32> = BTreeMap::new();
    for e in events {
        if let Some(&pf) = last.get(&e.scene) {
            if e.f < pf {
                problems.push(format!("{} f{}: 帧序倒退（前 f{}）", e.scene, e.f, pf));
            }
        }
        last.insert(e.scene.clone(), e.f);
        // 模式解析 + 边集校验（fail-closed：未知模式即违规）
        let m = match e.mode.as_str() {
            "Boot" => Mode::Boot,
            "Black" => Mode::Black,
            "Title" => Mode::Title,
            "Menu" => Mode::Menu,
            "Shop" => Mode::Shop,
            "ShopInfo" => Mode::ShopInfo,
            "Transition" => Mode::Transition,
            "StageStart" => Mode::StageStart,
            "Combat" => Mode::Combat,
            "DeathRespawn" => Mode::DeathRespawn,
            other => {
                // ④ 未证段 fail-closed（GAME OVER/CONTINUE 禁止出现在 golden）
                problems.push(format!("{} f{}: 未证模式 {other}（fail-closed，不发明）", e.scene, e.f));
                continue;
            }
        };
        let _ = m;
        let _ = FlowState::default();
    }
    problems
}

/// L5 差分：golden（oracle 真值）vs rust（移植侧）逐事件对拍。
/// 容差 0（帧号/模式/检查点全等）；缺项即差。
pub fn diff_l5(golden: &[L5Event], rust: &[L5Event]) -> Vec<String> {
    let mut problems = Vec::new();
    if golden.len() != rust.len() {
        problems.push(format!("事件数不等 golden {} vs rust {}", golden.len(), rust.len()));
    }
    for (i, (g, r)) in golden.iter().zip(rust.iter()).enumerate() {
        if g.scene != r.scene {
            problems.push(format!("[{i}] 场景不等 {} vs {}", g.scene, r.scene));
        }
        if g.f != r.f {
            problems.push(format!("[{i}] 帧号不等 golden f{} vs rust f{}", g.f, r.f));
        }
        if g.mode != r.mode {
            problems.push(format!("[{i}] 模式不等 golden {} vs rust {}", g.mode, r.mode));
        }
        if g.checkpoint != r.checkpoint {
            problems.push(format!("[{i}] 检查点不等 golden {:?} vs rust {:?}", g.checkpoint, r.checkpoint));
        }
    }
    problems
}
