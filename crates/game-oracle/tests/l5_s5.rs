//! DEV-3 S5-ALL-CLEAR 八关通关跑 L5 判定测试（R-l5-1/8；spec `docs/spec/l5-all-clear.md`）。
//!
//! 判定口径（l5-plan §2）：`l5_legality` 真闸（场景内帧序单调 + 模式序列过已证边集）
//! + `diff_l5` 0 容差对拍（golden=oracle 真值/占位登记 vs `s5_chain_run()` 移植侧运行）。
//! 通关判定真伪 = `$180` 真算术派发（`stage_clear_step`），负向测试锁非硬编码。

use game_oracle::l5::{
    diff_l5, l5_legality, parse_l5, parse_s5_anchors, s5_chain_run, s5_chain_run_for, L5Event,
};

fn golden() -> Vec<L5Event> {
    let jsonl = std::fs::read_to_string("../../oracle/traces/s5-all-clear.l5.jsonl").expect("S5 golden");
    parse_l5(&jsonl).expect("解析")
}

/// ① R-l5-8：S5 golden 解析 + 合法性真闸全绿 + 范围口径注。
#[test]
fn s5_golden_parses_and_legality_gate() {
    let evs = golden();
    let problems = l5_legality(&evs);
    assert!(problems.is_empty(), "S5 全程链合法性（已证边集真闸）：{problems:?}");
    // 口径注（R-l5-8）：3 场景 / 44 事件 / 36 命名检查点（8 关×4 + 标题 3 + 结局 1）
    let scenes: std::collections::BTreeSet<_> = evs.iter().map(|e| e.scene.as_str()).collect();
    let named = evs.iter().filter(|e| e.checkpoint.is_some()).count();
    assert_eq!(scenes.len(), 3, "3 场景（s5-title / s5-stg01 / s5-chain）");
    assert_eq!(evs.len(), 44, "44 事件（标题 3 + STG-01 段 5 + STG-02 真锚段 4 + 占位 32）");
    assert_eq!(named, 36, "36 命名检查点（8 关×4 + 标题 3 + 结局 1）");
    // 每关 4 检查点齐（关始/战斗段/关末/商店段，任务书②）
    for n in 1..=8u8 {
        let tag = format!("stg{n:02}");
        for kind in ["shop", "start", "combat", "clear"] {
            let cp = format!("{tag}-{kind}");
            assert!(
                evs.iter().any(|e| e.checkpoint.as_deref() == Some(cp.as_str())),
                "缺检查点 {cp}"
            );
        }
    }
    // ── G-S5-6 真帧锚诚实锁：锚台账 ↔ golden 1:1 + trigger 分级约束（防发明帧锚）──
    // 台账 = oracle/traces/s5-all-clear.anchors.jsonl（逐锚判定式/证据指针/触发来源）。
    let ledger = std::fs::read_to_string("../../oracle/traces/s5-all-clear.anchors.jsonl")
        .expect("S5 锚台账");
    let anchors = parse_s5_anchors(&ledger).expect("锚台账解析");
    let ganchors: Vec<_> = anchors.iter().filter(|a| a.golden).collect();
    assert_eq!(ganchors.len(), evs.len(), "台账 golden 条目数 == golden 事件数");
    for (i, (g, a)) in evs.iter().zip(ganchors.iter()).enumerate() {
        assert_eq!(
            (&g.scene, g.f, &g.mode, &g.checkpoint),
            (&a.scene, a.f, &a.mode, &a.checkpoint),
            "[{i}] golden ↔ 台账 全等（scene/f/mode/checkpoint）"
        );
        if a.f == 0 {
            assert_eq!(a.trigger, "placeholder", "[{i}] f=0 必须标注 placeholder（fail-closed）");
        } else {
            assert!(
                a.trigger == "natural" || a.trigger == "injected",
                "[{i}] 真锚 trigger 必须分级（natural/injected）"
            );
            assert!(a.source_run.is_some(), "[{i}] 真锚必须带 source_run");
            assert!(!a.predicate.is_empty(), "[{i}] 真锚必须带判定式（可机器重导）");
        }
    }
    // 回填口径显式（R-l5-8 更新）：12 真锚（7 trace 锚 + 5 触发注入链锚）/ 32 占位
    let real = ganchors.iter().filter(|a| a.f > 0).count();
    let ph = ganchors.iter().filter(|a| a.f == 0).count();
    assert_eq!((real, ph), (12, 32), "G-S5-6 回填口径：12 真锚 / 32 占位");
}

/// ② S5-ALL-CLEAR 链可跑通：移植侧运行 vs golden 0 容差对拍 + 运行面（到店/出店/购买/骨架）。
#[test]
fn s5_chain_runs_and_diffs_golden() {
    let run = s5_chain_run();
    let problems = diff_l5(&golden(), &run.events);
    assert!(problems.is_empty(), "S5 全程链 0 容差对拍：{problems:?}");
    assert_eq!(run.stages_cleared, 8, "8 关全清（E37C `INC $180` 实测）");
    assert_eq!(run.shop_visits, 8, "R-l5-6 每关 1 次到店");
    assert_eq!(run.shop_exits, 8, "出店全过（含 stage-8 退出隐藏门 ack 链 AR65-1/2）");
    assert!(run.ending_reached, "通关判定 → `$95F9` 结局入口");
    assert_eq!(run.final_mode, game_core::flow::Mode::Ending, "粗粒度机终态=结局段");
    // t193 八关骨架驱动面（root/tail 逐关消费）
    assert_eq!(run.skeletons.len(), 8);
    for (i, (n, root, tail)) in run.skeletons.iter().enumerate() {
        assert_eq!(*n, i as u8 + 1);
        assert!(*root != 0 && *tail != 0, "STG-{n} root/tail 非零");
    }
    // DEV-1 购买流集成（R-shop-purchase-*）：关 1 到店=￥0（R-shop-6 截图「你有 ──￥0」同源）；
    // 关 2/3 各购 1 次（价 10）后 `$33`≥2 无翼 ⇒ WingRequired 门（bank00:$9DAD-$9DE0）拦后续
    assert_eq!(run.purchase_spent, 20, "DEV-1 购买扣款 2×10（不足/翼前置门不计）");
}

/// ③ 通关判定真伪（非硬编码）：7 关负向跑不得产生结局段；golden 篡改必被对拍检出。
#[test]
fn s5_all_clear_judgement_is_state_derived() {
    // 负向①：只跑 7 关 ⇒ `$180` 到 8 ≠ 9 ⇒ 无 Ending（E392 真判定）
    let run7 = s5_chain_run_for(7);
    assert!(!run7.ending_reached, "7 关不得判通关（`$180==9` 真算术）");
    assert_eq!(run7.stages_cleared, 7);
    assert_eq!(run7.final_mode, game_core::flow::Mode::StageClear, "7 关终态停在关末段");
    assert_eq!(run7.shop_visits, 7, "7 关 7 次到店（不到第 8 关店）");
    assert!(
        !run7.events.iter().any(|e| e.mode == "Ending"),
        "7 关链不得含结局段事件"
    );
    // 负向②：golden 篡改（帧号/模式）必被 diff_l5 检出（0 容差）
    let g = golden();
    let mut bad = g.clone();
    bad[6].f += 1; // stg01-combat 帧 +1
    assert!(diff_l5(&g, &bad).iter().any(|p| p.contains("帧号不等")), "帧号差必检出");
    let mut bad2 = g.clone();
    let last = bad2.len() - 1;
    bad2[last].mode = "Title".into(); // 结局段改标题
    assert!(diff_l5(&g, &bad2).iter().any(|p| p.contains("模式不等")), "模式差必检出");
}

/// ④ 合法性真闸负向：未证转移/未证模式必拒（防 S5 链发明关末直跳/结局伪造）。
#[test]
fn s5_legality_gate_rejects_invented_joints() {
    let g = golden();
    // 负向①：关末→关始直跳（E396 实走 E16B→`$9BC3` 商店 init：直跳不发明）
    let mut bad = g.clone();
    let i = bad
        .iter()
        .position(|e| e.checkpoint.as_deref() == Some("stg02-start"))
        .expect("stg02-start");
    // 把 stg02-start 前件改为 StageClear ⇒ 同时造出 Shop→StageClear 与 StageClear→StageStart
    // 两组未证关节，真闸必须全部拒绝
    bad[i - 1].mode = "StageClear".into();
    assert!(
        l5_legality(&bad).iter().any(|p| p.contains("非法模式序列")),
        "StageClear→StageStart 必拒（负向）"
    );
    // 负向②：结局段伪造为未证模式（GAME OVER 动态锚仍缺 ⇒ golden 禁入）
    let mut bad2 = g.clone();
    let last = bad2.len() - 1;
    bad2[last].mode = "GameOverMenu".into();
    assert!(
        l5_legality(&bad2).iter().any(|p| p.contains("未证模式")),
        "未证模式必 fail-closed"
    );
    // 负向③：帧序倒退（占位段也不许）
    let mut bad3 = g.clone();
    bad3[6].f = 100; // stg01-combat 1500→100 < 1260
    assert!(l5_legality(&bad3).iter().any(|p| p.contains("帧序倒退")), "帧序倒退必检出");
}
