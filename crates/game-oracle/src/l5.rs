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
    let mut problems = Vec::new();
    use game_core::flow::Mode;
    let mut last: BTreeMap<String, u32> = BTreeMap::new();
    let mut modes_per_scene: BTreeMap<String, Mode> = BTreeMap::new();
    for e in events {
        // 契约共演（b 案①）：f=0 占位行=帧未知哨兵**显式豁免**帧序单调性
        //（ok-trigger-pending 同族注记，不冒充真帧）；真帧行域内单调照查（强化非弱化）
        if e.f != 0 {
            if let Some(&pf) = last.get(&e.scene) {
                if e.f < pf {
                    problems.push(format!("{} f{}: 帧序倒退（前 f{}）", e.scene, e.f, pf));
                }
            }
            last.insert(e.scene.clone(), e.f);
        }
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
            // DEV-3（R-l5-8）：关末/结局段模式——**f=0 占位段专用**（t193 先例：占位=无动态锚
            // 显式声明，非发明帧号）；GAME OVER/CONTINUE 动态锚仍缺 ⇒ 维持禁入（A2 §2.6）。
            "StageClear" => Mode::StageClear,
            "Ending" => Mode::Ending,
            other => {
                // ④ 未证段 fail-closed（GAME OVER/CONTINUE 禁止出现在 golden）
                problems.push(format!("{} f{}: 未证模式 {other}（fail-closed，不发明）", e.scene, e.f));
                continue;
            }
        };
        // R14-1 边集校验（桩实装→实装）：场景内相邻模式对必须过 t81 已证边集
        // （FlowState::advance 反演：存在 ModeEvent 使 prev→cur）
        if let Some(&prev) = modes_per_scene.get(&e.scene) {
            if prev != m && !edge_proven(prev, m) {
                problems.push(format!(
                    "{} f{}: 非法模式序列 {:?}→{:?}（t81 已证边集外，fail-closed）",
                    e.scene, e.f, prev, m
                ));
            }
        }
        modes_per_scene.insert(e.scene.clone(), m);
    }
    problems
}

/// t81+A2+A3 已证边集反演：存在 ModeEvent 使 prev.advance(ev) == cur
/// （含 DEV-2 死亡链事件族与 DEV-3 关末/通关/结局事件族——见 flow.rs `advance` 边注）。
fn edge_proven(prev: game_core::flow::Mode, cur: game_core::flow::Mode) -> bool {
    use game_core::flow::{FlowState, ModeEvent};
    let events = [
        ModeEvent::Black, ModeEvent::Title, ModeEvent::Menu, ModeEvent::ShopArrival,
        ModeEvent::ShopPage, ModeEvent::ExitTransition, ModeEvent::StageStart,
        ModeEvent::MidCombat, ModeEvent::DeathRespawn,
        ModeEvent::GameOverArm, ModeEvent::GameOverArrival, ModeEvent::MenuToggle,
        ModeEvent::Continue, ModeEvent::GameFinish,
        ModeEvent::StageClear, ModeEvent::StageAdvance, ModeEvent::AllClear, ModeEvent::EarlyEnd,
    ];
    events.iter().any(|ev| {
        FlowState { mode: prev, data: Default::default() }
            .advance(*ev)
            .map(|s| s.mode == cur)
            .unwrap_or(false)
    })
}

/// L5 差分：golden（oracle 真值）vs rust（移植侧）逐事件对拍。
/// 容差 0（帧号/模式/检查点全等）；缺项即差。
pub fn diff_l5(golden: &[L5Event], rust: &[L5Event]) -> Vec<String> {
    let mut problems = Vec::new();
    if golden.len() != rust.len() {
        problems.push(format!("事件数不等 golden {} vs rust {}", golden.len(), rust.len()));
    }
    for (i, (g, r)) in golden.iter().zip(rust.iter()).enumerate() {
        // 契约共演（b 案②）：f=0 占位行 **pending 不入 0 容差比对面**（显式挂起不冒充）；
        // 真帧行同域 0 容差（scene/f/mode/checkpoint 全等）
        if g.f == 0 {
            continue;
        }
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

/// 显式挂起清单（b 案②配套）：golden 中 f=0 占位行的行号（pending 不入 0 容差面
/// ——ok-trigger-pending 同族注记，不冒充 natural/真帧）。
pub fn l5_pending_rows(events: &[L5Event]) -> Vec<usize> {
    events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.f == 0)
        .map(|(i, _)| i)
        .collect()
}

// ===== G-S5-6 真帧锚台账（oracle/traces/s5-all-clear.anchors.jsonl；golden 回填诚实性锁）=====

/// 锚台账条目：每条 golden 事件一条（`golden`=true）+ 附加动态锚（`golden`=false）。
/// `trigger`：`natural`=自然输入轨道 trace 锚 / `injected`=触发注入链（poke `$26 bit4` 门位，
/// 不冒充自然通关）/ `placeholder`=f=0 占位（fail-closed 不发明帧号）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S5Anchor {
    pub id: String,
    pub golden: bool,
    pub checkpoint: Option<String>,
    pub mode: String,
    pub scene: String,
    pub f: u32,
    pub trigger: String,
    pub source_run: Option<String>,
    pub predicate: String,
}

/// 解析锚台账 JSONL（字段：id/golden/checkpoint?/mode/scene/f/trigger/source_run?/predicate）。
pub fn parse_s5_anchors(jsonl: &str) -> Result<Vec<S5Anchor>, String> {
    let mut out = Vec::new();
    for (lineno, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v = crate::l2::parse_json(line).map_err(|e| format!("第 {} 行: {e}", lineno + 1))?;
        let o = match v {
            crate::l2::Json::Obj(o) => o,
            _ => return Err(format!("第 {} 行非对象", lineno + 1)),
        };
        let get_str = |k: &str| -> Option<String> {
            o.iter().find(|(key, _)| key == k).and_then(|(_, v)| match v {
                crate::l2::Json::Str(s) => Some(s.clone()),
                _ => None,
            })
        };
        let get_num = |k: &str| -> Option<f64> {
            o.iter().find(|(key, _)| key == k).and_then(|(_, v)| match v {
                crate::l2::Json::Num(n) => Some(*n),
                _ => None,
            })
        };
        let get_bool = |k: &str| -> Option<bool> {
            o.iter().find(|(key, _)| key == k).and_then(|(_, v)| match v {
                crate::l2::Json::Bool(b) => Some(*b),
                _ => None,
            })
        };
        let id = get_str("id").ok_or_else(|| format!("第 {} 行缺 id", lineno + 1))?;
        let golden = get_bool("golden").ok_or_else(|| format!("第 {} 行缺 golden", lineno + 1))?;
        let mode = get_str("mode").ok_or_else(|| format!("第 {} 行缺 mode", lineno + 1))?;
        let scene = get_str("scene").ok_or_else(|| format!("第 {} 行缺 scene", lineno + 1))?;
        let f = get_num("f").ok_or_else(|| format!("第 {} 行缺 f", lineno + 1))? as u32;
        let trigger =
            get_str("trigger").ok_or_else(|| format!("第 {} 行缺 trigger", lineno + 1))?;
        let predicate =
            get_str("predicate").ok_or_else(|| format!("第 {} 行缺 predicate", lineno + 1))?;
        out.push(S5Anchor {
            id,
            golden,
            checkpoint: get_str("checkpoint"),
            mode,
            scene,
            f,
            trigger,
            source_run: get_str("source_run"),
            predicate,
        });
    }
    if out.is_empty() {
        return Err("锚台账为空".into());
    }
    Ok(out)
}

// ===== DEV-3 S5-ALL-CLEAR 段级接力驱动（R-l5-1/8；docs/spec/l5-all-clear.md）=====

/// 关内拾取收入占位常量（`bank02:8FA9-$8FB5`：`$018E += 3`/次〔A 级〕；每关拾取**次数**未取证
/// ⇒ G-S5-6 占位，仅 S5 驱动脚手架，不入行为断言）。
const S5_PLACEHOLDER_PICKUPS_PER_STAGE: u8 = 20;

/// `$018C` 拾取门槛占位（shop 期值未证——shop-purchase 测试先例：显式取非 `$FF` 走裸升，
/// 不发明默认值；G-S5-6 同族）。
const S5_PLACEHOLDER_PICKUP_THRESHOLD: u8 = 0x00;

/// S5 通关跑报告（段级链运行结果；判定物 = `events` 过 `l5_legality` 真闸 + `diff_l5` 对拍）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S5Run {
    /// 检查点链（R-l5-8 命名；帧锚分级：trace 锚 vs f=0 占位）。
    pub events: Vec<L5Event>,
    /// 已清关数（`INC $180` 实测）。
    pub stages_cleared: u8,
    /// 商店段到店次数（R-l5-6：每关 1 次）。
    pub shop_visits: u8,
    /// 商店出店成功次数（`phase==Exited`；含 stage-8 退出隐藏门 ack 链，AR65-1/2）。
    pub shop_exits: u8,
    /// DEV-1 购买扣款累计（价 10 路实测；不足/满级分支不计）。
    pub purchase_spent: u8,
    /// 八关骨架消费记录（n, root, tail）——t193/t87 `stage_skeleton` 驱动面。
    pub skeletons: Vec<(u8, u16, u16)>,
    /// 通关判定到达结局入口（`$180==9` → `$95F9`）。
    pub ending_reached: bool,
    /// 粗粒度机终态（全 8 关= `Mode::Ending`；<8 关负向跑 = `Mode::StageClear`）。
    pub final_mode: game_core::flow::Mode,
}

fn ev(scene: &str, f: u32, mode: &str, cp: Option<&str>) -> L5Event {
    L5Event { scene: scene.into(), f, mode: mode.into(), checkpoint: cp.map(|s| s.into()) }
}

/// 出厂配置：8 关全程（A3 §1.2 `$180==9` 通关判定）。
pub fn s5_chain_run() -> S5Run {
    s5_chain_run_for(8)
}

/// S5 段级接力驱动（**移植侧运行**——驱动 game-core 状态机产出检查点链，供 `diff_l5` 对拍）。
///
/// 链形态（R-l5-1）：标题段 → [商店段 → 关始段 → 战斗段 → 关末段]×N → 通关判定 → 结局段。
/// 状态机来源：粗粒度 `FlowState`（已证边推进，未证边 `expect` fail-closed）+
/// 细粒度 `StageClearState::stage_clear_step`（A3 §1.1 逐写点，`$180` **真算术派发**，
/// 非硬编码）+ DEV-1 `shop_purchase_step`（购买流）+ `stage_skeleton(n)`（t193 八关骨架）。
///
/// 帧锚分级（R-l5-8 + G-S5-6 动态回填）：真锚 = title-menu/gameplay-stage1 trace 锚（trigger=natural）
/// + `gs5-stg01-clear-anchor` 关末/关 2 链锚（trigger=injected：poke f2400 `$26 bit4` 门位触发，
/// 自然可达性=Run A 负结果——锚不冒充自然通关）；其余段 f=0 占位（fail-closed 不发明帧号）。
/// 逐锚判定式与证据指针 = `oracle/traces/s5-all-clear.anchors.jsonl`（golden 同步锁测试）。
/// `$0300` 推进闸 = 外部事件（G-S5-2）：**占位约定 = 倒数尽后开启**（非 ROM 时序主张；
/// 真机动态时点实测 f3066 见 anchors 台账 gate300-open 条）。
///
/// `stages`≠8 仅用于负向测试（<8 不得产生结局段）；出厂 = 8。
pub fn s5_chain_run_for(stages: u8) -> S5Run {
    use game_core::flow::{
        ending_entry, stage_clear_arm, stage_clear_step, FlowState, ModeEvent, StageClearEnv,
        StageClearPhase, StageClearState,
    };
    use game_core::stage::stage_skeleton;
    use game_core::ui::{
        shop_purchase_step, ShopFx, ShopPurchasePhase, ShopPurchaseState, BTN_A, BTN_ACK, BTN_B,
        BTN_DOWN, BTN_SELECT,
    };

    let mut flow = FlowState::default();
    let mut clear = StageClearState::new();
    let mut shop = ShopPurchaseState::default();
    shop.pickup.threshold = S5_PLACEHOLDER_PICKUP_THRESHOLD;
    let mut run = S5Run {
        events: Vec::new(),
        stages_cleared: 0,
        shop_visits: 0,
        shop_exits: 0,
        purchase_spent: 0,
        skeletons: Vec::new(),
        ending_reached: false,
        final_mode: game_core::flow::Mode::Boot,
    };

    // ── 标题段（R-ui-7 trace 锚 title-menu f180/360/540）──
    flow = flow.advance(ModeEvent::Black).expect("R-ui-7 已证边 Boot→Black");
    run.events.push(ev("s5-title", 180, "Black", Some("boot-black")));
    flow = flow.advance(ModeEvent::Title).expect("R-ui-7 Black→Title");
    run.events.push(ev("s5-title", 360, "Title", Some("title-in")));
    flow = flow.advance(ModeEvent::Menu).expect("R-ui-7 Title→Menu");
    run.events.push(ev("s5-title", 540, "Menu", Some("menu-in")));

    for n in 1..=stages {
        // t193 八关骨架驱动（关始段内容面：目录 root/tail + 调度/内容记录）
        let sk = stage_skeleton(n).expect("八关骨架必在（t193 STG-01..08）");
        run.skeletons.push((n, sk.root, sk.tail));
        // 帧锚分级（R-l5-8 + G-S5-6 动态回填）：(scene, f) 逐事件查表——
        // 真锚来源 = `oracle/traces/s5-all-clear.anchors.jsonl`（trigger 字段：natural / injected），
        // 未回填 = ("s5-chain", 0) 占位（fail-closed 不发明帧号）。
        // s5-stg01 场景 = gameplay-stage1 前缀锚（trigger=natural）+ gs5-stg01-clear-anchor
        // 关末/关 2 锚（trigger=injected：触发 = poke f2400 `$26 bit4` 门位；自然可达性 = Run A
        // 负结果，锚不冒充自然通关，见 docs/findings/gs5-6-dynamic-replay.md）。
        let anchor = |kind: &str| -> (&'static str, u32) {
            match (n, kind) {
                (1, "shop") => ("s5-stg01", 1080),
                (1, "exit") => ("s5-stg01", 1200),
                (1, "start") => ("s5-stg01", 1260),
                (1, "combat") => ("s5-stg01", 1500),
                (1, "clear") => ("s5-stg01", 2401),
                (2, "shop") => ("s5-stg01", 3116),
                (2, "exit") => ("s5-stg01", 3400),
                (2, "start") => ("s5-stg01", 3450),
                (2, "combat") => ("s5-stg01", 3900),
                // ── (b) 同域回填锚（run #3/stg04 run 实测——trigger/injected 逐项登记
                // 由锚台账负责；本表仅真帧域坐标）──
                (2, "clear") => ("s5-chain", 4601),
                (3, "shop") => ("s5-chain", 5314),
                (3, "exit") => ("s5-chain", 5604),
                (3, "start") => ("s5-chain", 5700),
                // 【裁定① dated】f6500=倒数期残影作废（判定式 dated 注已入台账）——
                // 重锚 f6000（双 run trace#f6000+双 probe 截图四证；$A418 真战斗窗）
                (3, "combat") => ("s5-chain", 6000),
                (3, "clear") => ("s5-chain", 6401),
                (4, "shop") => ("s5-chain", 7116),
                (4, "exit") => ("s5-chain", 7404),
                (4, "start") => ("s5-chain", 7500),
                (4, "combat") => ("s5-chain", 9000),
                // 未采段（stg05-08）= f=0 占位（pending 不发明帧号）
                _ => ("s5-chain", 0),
            }
        };

        // ── 商店段（R-l5-6：n=1=R-ui-6 到店；n≥2=E396→E16B→`$9BC3` 商店 init）──
        if n == 1 {
            flow = flow.advance(ModeEvent::ShopArrival).expect("R-ui-6 Menu→Shop");
        } else {
            flow = flow
                .advance(ModeEvent::StageAdvance)
                .expect("R-l5-4/6 StageClear→Shop（E396→E16B→$9BC3）");
        }
        run.shop_visits += 1;
        let (scene_shop, f_shop) = anchor("shop");
        run.events.push(ev(scene_shop, f_shop, "Shop", Some(&format!("stg{:02}-shop", n))));
        // 商店 init 复位（bank00:$9BC3 init 链 → `$9CEC` 序列 → `$9CF7/$9CF9` `$0400:=0`，AD65-1）：
        // 相位/光标回主面板；持续字段（`$018E`/`$33/$34`/`$32` 等 RAM）跨店保持。
        shop.phase = ShopPurchasePhase::MainRing;
        shop.cursor = 0;
        shop.prompt = None;
        // DEV-1 购买流（R-shop-purchase-*）：主面板 cursor0=武器入口 → A 购买（价 10）→ B 回主环
        shop.stage = clear.stage; // `$0180` 同步（发货阶段门读口）
        shop_purchase_step(&mut shop, BTN_SELECT); // cursor=0 ⇒ 武器入口（bank00:$9D5E）
        let fx = shop_purchase_step(&mut shop, BTN_A);
        if fx == ShopFx::WeaponBought {
            run.purchase_spent = run.purchase_spent.saturating_add(10); // bank00:$9DE8-$9DF4
        }
        // 购后回主环（B）：成功/不足 ack ⇒ 落主环（bank00:$9D82-$9D88）；
        // 翼前置/满级消息 ack 后落**武器环**（`msg_back=WeaponMenu`）⇒ 再 B 回主环
        for _ in 0..2 {
            if shop.phase == ShopPurchasePhase::MainRing {
                break;
            }
            shop_purchase_step(&mut shop, BTN_B);
        }
        // 出店（机内退出 = 主面板 cursor3 退出入口 bank01:$A022 → 开 stage 转场，R-shop-7 面）
        for _ in 0..3 {
            shop_purchase_step(&mut shop, BTN_DOWN); // `$0400` 0→3
        }
        shop_purchase_step(&mut shop, BTN_SELECT); // 退出入口
        if shop.phase == ShopPurchasePhase::Message {
            // 退出隐藏门（AR65-1/2：`$0180==8` 或 `($18&$33)==$11`）⇒ ack 后合流（bank01:$A0CF）
            shop_purchase_step(&mut shop, BTN_ACK);
        }
        if shop.phase == ShopPurchasePhase::Exited {
            run.shop_exits += 1;
        }

        // ── 关始段（R-ui-6 f1260 / R-shop-7 近黑过渡 → 开 stage）──
        flow = flow.advance(ModeEvent::ExitTransition).expect("R-shop-7 Shop→Transition");
        let (scene_exit, f_exit) = anchor("exit");
        run.events.push(ev(scene_exit, f_exit, "Transition", None));
        flow = flow.advance(ModeEvent::StageStart).expect("R-ui-6 Transition→StageStart");
        let (scene_start, f_start) = anchor("start");
        run.events.push(ev(scene_start, f_start, "StageStart", Some(&format!("stg{:02}-start", n))));

        // ── 战斗段（R-ui-6 f1500+；推进机制 R-stage-tick-*）──
        flow = flow.advance(ModeEvent::MidCombat).expect("R-ui-6 StageStart→Combat");
        let (scene_combat, f_combat) = anchor("combat");
        run.events.push(ev(scene_combat, f_combat, "Combat", Some(&format!("stg{:02}-combat", n))));
        // 关内拾取收入（bank02:8FA9 `$018E+=3`/次——次数占位 G-S5-6；关 1 前到店=￥0 同源）
        shop.money = shop.money.saturating_add(3u8.saturating_mul(S5_PLACEHOLDER_PICKUPS_PER_STAGE));

        // ── 关末段（A3 §1.1 关末块；R-l5-3/4）──
        stage_clear_arm(&mut clear);
        flow = flow.advance(ModeEvent::StageClear).expect("A3 §1.1 E2F3 Combat→StageClear");
        let (scene_clear, f_clear) = anchor("clear");
        run.events.push(ev(scene_clear, f_clear, "StageClear", Some(&format!("stg{:02}-clear", n))));
        // 细粒度步进到派发（虚拟帧推进；`$2B` 相位 0,1,2,…；G-S5-2 占位：倒数尽后开 `$0300` 闸）
        let mut dispatched = false;
        for i in 0u32..10_000 {
            let gate = if clear.countdown_2f == 0 { 1 } else { 0 };
            let _fx = stage_clear_step(
                &mut clear,
                StageClearEnv {
                    frame_2b: (i & 0xFF) as u8,
                    gate_300: gate,
                    player_present: true,
                },
            );
            if clear.phase != StageClearPhase::Clearing {
                dispatched = true;
                break;
            }
        }
        assert!(dispatched, "关末段派发必须有限帧可达（A3 §1.1；fail-closed 不发明等待语义）");
        match clear.phase {
            StageClearPhase::NextStage => {
                // E396 关间：`INC $180` 已落；StageClear→Shop 关节在下一循环消费（R-l5-6）
                run.stages_cleared += 1;
            }
            StageClearPhase::AllClear => {
                // 通关判定（A3 §1.2）：`$180==9` 真算术 ⇒ `$95F9` 结局入口
                run.stages_cleared += 1;
                flow = flow.advance(ModeEvent::AllClear).expect("A3 §1.2 E392-E3A1");
                let _fx = ending_entry(&mut clear); // bank00:95F9 `$26:=$80`
                run.events.push(ev("s5-chain", 0, "Ending", Some("all-clear-95f9")));
                run.ending_reached = true;
                break;
            }
            StageClearPhase::EarlyEnd => {
                // `$FFF8` 哨兵（0x0A）：出厂不可达（D4 保真）；S5 出厂配置不预期到达
                flow = flow.advance(ModeEvent::EarlyEnd).expect("A3 §1.1 E387-E38F");
                break;
            }
            _ => {}
        }
    }
    run.final_mode = flow.mode;
    run
}
