//! 玩家主武器弹丸接线（task-37）：B 键长按门控 + 计数驱动生成 + `shot::tick` 运动/寿命 +
//! shell 侧命中管线（注册/敌框/4 边属性/清框回收）+ 受限额伤害。
//!
//! 证据边界（详见 `docs/findings/plan-bullets-hit.md` 与 task-38
//! `docs/findings/playability-evidence-h16-coords.md` §2.4）：
//! - **生成**：B 键按住（`$18` bit6）→ `$39/$3A` 重复计时器（默认 reload=7；`$94DF` 表）→ 六型生成器；
//!   在途计数 `$072E,X` < 在途上限 `$074A,Y` ⇒ 生成并 INC 回写（R-shot-gen-1/2/3，A 级）；
//! - **运动/寿命**：直接调用已证 `shot::tick`（DEC 寿命/脚本机取指/LUT 双轴/溢出死亡，A 级）；
//! - **命中**：`shot_hitbox_register`（弹 7×10 框）+ `fetch_shape_boxes`（敌框形 A 级）+
//!   `hit_resolves`（4 边+属性相交 A 级）+ `clear_hit_entry`（清框）+ 弹回收（`$87FD`）；
//! - **伤害（受限额）**：仅 `type==$4A`（STG-01 中 boss，HP=14 A 级快照）可接 `Const2`；
//!   其余敌型 **0 伤 + `fail_closed=true` 事件登记**（B4/W18 未定谳，不用 `f630` 当通用敌 HP）；
//! - **不接**：玩家被击（W15/W15′ 未证）、敌弹友伤（过滤 `type_code & 0x80`）、敌 HP 死亡回收
//!   （`$D616` 死亡门对 `$4A` 亦未证 ⇒ 不回收敌槽）。
//!
//! 玩家弹保存在 game-shell 自有 [`BulletGrid`]（**不再写 `StageState.bullets`**）：
//! engine `combat_frame` 的弹/命中管线对空网格惰性 ⇒ 无双步进/无双份命中。

use game_core::enemy::{fetch_shape_boxes, shape_stream_addr};
use game_core::engine::StageState;
use game_core::shot::{
    self, clear_hit_entry, damage_apply, hit_resolves, shot_hitbox_register, shot_slot_alloc,
    spawn_init, BulletGrid, HitBoxEntry, HitDmgPath, ShotObject, SlotAlloc, PLAYER_ONLY_SLOTS,
    SHARED_SLOTS,
};

/// `$39` 重复计时器重装值——**默认 STG-01（`$3C` 连射级=0）= 7f**：
/// `bank02:$9391 LDA $94DF,Y / $9394 STA $39`（Y=`$3C` 连射级；表 `$94DF`=`07 05 04 03 02`）。
/// 说明：`$9200/$92C9` 的 `LDA #$08/STA $39/$3A` 只是初始化值，随后被武器配置装载覆盖
/// （t13 旧值 8 已按 A 级 ROM 直读 + `oracle/traces/fire-hold.shots.jsonl` 出生帧
/// `1301,1308,1315,1322,…` Δ=7 动态对拍更正）。
pub const SHOT_TIMER_RELOAD: u8 = 7;
/// `$94DF` 连射级 → `$39` reload 表（A 级直读，`$3C`=0..4；≥4 商店满级不再升）。
pub const RAPID_RELOAD_TABLE: [u8; 5] = [7, 5, 4, 3, 2];
/// 连射级 → reload（`$3C` 域 0..4；域外 ⇒ None fail-closed，不发明折返）。
pub fn reload_for_rapid(rapid_level: u8) -> Option<u8> {
    RAPID_RELOAD_TABLE.get(rapid_level as usize).copied()
}
/// 六型生成器上限（X=0..5；R-shot-gen-1 六弹型并行）。
pub const MAX_ACTIVE_TYPES: usize = 6;
/// STG-01 中 boss 型值（波次流 `$A3CD` 内 `01 4A`，A 级）。
pub const BOSS_MID_TYPE: u8 = 0x4A;
/// 中 boss 初始 HP（`$0630+27`=14，`arena-midboss` A 级快照）——仅白名单型使用。
pub const BOSS_MID_HP: u8 = 14;

/// 玩家弹槽序（分配器首次适应序：4 组 × 5 槽，组内 stride 1）。
pub const PLAYER_SLOT_ORDER: [u8; 20] = [
    0x80, 0x81, 0x82, 0x83, 0x84, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC,
    0xD4, 0xD5, 0xD6, 0xD7, 0xD8,
];

/// 敌命中事件（**已证命中管线的直接结果**；damage 受白名单限额）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyHitEvent {
    /// 宿主逻辑帧。
    pub frame: u64,
    /// 被命中敌槽（8..=27）。
    pub enemy_slot: u8,
    /// 命中弹槽（`$80` 起）。
    pub bullet_slot: u8,
    /// 实际伤害（白名单型=2；其余 fail-closed=0）。
    pub damage: u8,
    /// 伤害是否 fail-closed（非 `$4A` 型=true：命中已判定但 0 伤）。
    pub fail_closed: bool,
}

/// 玩家子弹视图（坐标=弹网格 `$04EE/$055E` A 地址面；型值不命名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BulletView {
    /// 弹槽偏移（`$80` 起；bit7 天然置）。
    pub slot: u8,
    /// `$04EE+Y` x。
    pub x: u8,
    /// `$055E+Y` y。
    pub y: u8,
    /// `$063E+Y` 型码（0..5 玩家六型；不命名）。
    pub type_code: u8,
}

/// 玩家域槽判定（`PLAYER_ONLY_SLOTS`∪`SHARED_SLOTS`）。
fn is_player_slot(slot: u8) -> bool {
    PLAYER_ONLY_SLOTS.contains(&slot) || SHARED_SLOTS.contains(&slot)
}

/// 弹是否玩家弹（槽属玩家域 **且** 型码 bit7 清；敌弹 `ENEMY_SHOT_TYPE=$80` 必须过滤）。
fn is_player_bullet(grid: &BulletGrid, slot: u8) -> bool {
    is_player_slot(slot) && grid.state[slot as usize] != 0 && grid.type_code[slot as usize] & 0x80 == 0
}

/// `$39/$3A` 重复计时器步进（bank03:$A1AF/$A1B6-$A1BA，A 级）：
/// 未按 B ⇒ `$3A:=1`（下按帧即发）；按住 ⇒ DEC，归零帧重装 `$39` 并返回"本帧生成"。
pub fn fire_gate(b_held: bool, timer: &mut u8) -> bool {
    if !b_held {
        *timer = 1;
        return false;
    }
    *timer = timer.saturating_sub(1);
    if *timer == 0 {
        *timer = SHOT_TIMER_RELOAD;
        true
    } else {
        false
    }
}

/// 在途弹计数（`$072E,X` 语义）：玩家域槽中 `type_code==type_idx` 的活弹数。
///
/// 派生自弹网格（每帧重算，无双源漂移；ROM 显式计数器等价假设见 plan §6）。
pub fn inflight_for_type(grid: &BulletGrid, type_idx: u8) -> u16 {
    PLAYER_SLOT_ORDER
        .iter()
        .filter(|&&slot| is_player_bullet(grid, slot) && grid.type_code[slot as usize] == type_idx)
        .count() as u16
}

/// 六型生成器一步（R-shot-gen-1/3，A 级）：X=0..6，在途 < 上限 ⇒ `shot_slot_alloc` 首适应 +
/// `spawn_init` 写六字段。返回本帧生成槽列表（供测试/摘要；运动由 [`step_player_bullets`] 做）。
///
/// `weapon_level`=`$33`（`flow.data.weapon_level`）；配置对=`shot::config_pairs_for_level`
/// （级 0=1 对 `(05,0C)`）。满载/上限达 ⇒ 该型不生成（fail-closed，不强制回收）。
pub fn generate_shots(
    grid: &mut BulletGrid,
    player_x: u8,
    player_y: u8,
    weapon_level: u8,
) -> Vec<u8> {
    let pairs = shot::config_pairs_for_level(weapon_level);
    let mut spawned = Vec::new();
    for (type_idx, &(cap, script_param)) in pairs.iter().take(MAX_ACTIVE_TYPES).enumerate() {
        let type_idx = type_idx as u8;
        if inflight_for_type(grid, type_idx) >= u16::from(cap) {
            continue;
        }
        let SlotAlloc::Found { slot, .. } = shot_slot_alloc(&grid.state) else {
            break; // 20 槽满载（SEC 路径，A 级）⇒ 本帧停止
        };
        let obj = spawn_init(slot, type_idx, type_idx, player_x, player_y, script_param);
        write_object(grid, slot, &obj);
        spawned.push(slot);
    }
    spawned
}

fn write_object(grid: &mut BulletGrid, slot: u8, obj: &ShotObject) {
    let i = slot as usize;
    grid.state[i] = obj.lifetime;
    grid.action_ptr[i] = obj.script_ptr;
    grid.x[i] = obj.pos_x;
    grid.y[i] = obj.pos_y;
    grid.motion[i] = obj.motion_code;
    grid.type_code[i] = obj.shot_type;
}

fn read_object(grid: &BulletGrid, slot: u8) -> ShotObject {
    let i = slot as usize;
    ShotObject {
        lifetime: grid.state[i],
        script_ptr: grid.action_ptr[i],
        pos_x: grid.x[i],
        pos_y: grid.y[i],
        motion_code: grid.motion[i],
        shot_type: grid.type_code[i],
    }
}

/// 释放弹槽（`state:=0`；其它字段保留=ROM 回收写集，A：R-shot-alloc-3）。
fn release(grid: &mut BulletGrid, slot: u8) {
    grid.state[slot as usize] = 0;
}

/// 玩家弹逐帧运动/寿命（**直接调用已证 `shot::tick`**，R-shot-motion-1/2 整体）：
/// 寿命 DEC/脚本机取指/LUT 双轴推进；死亡（寿命 0/溢出/借位）⇒ 回收。返回本帧释放槽数。
///
/// 只处理玩家弹（槽域 + `type_code & 0x80 == 0`）；敌弹槽一律跳过（防友伤）。
pub fn step_player_bullets(grid: &mut BulletGrid) -> u16 {
    let mut released = 0u16;
    for &slot in PLAYER_SLOT_ORDER.iter() {
        if !is_player_bullet(grid, slot) {
            continue;
        }
        let mut obj = read_object(grid, slot);
        match shot::tick(&mut obj) {
            Some(()) => write_object(grid, slot, &obj),
            None => {
                release(grid, slot);
                released += 1;
            }
        }
    }
    released
}

/// 敌槽命中的伤害结果（受限额）：仅 `type==$4A` 白名单施 `Const2`（HP=`$0630+27`=14 A 级）；
/// 其余敌型 0 伤 + `fail_closed`，且**不写** `f630`（B4/W18 未定谳，禁止当通用敌 HP）。
///
/// 返回 `(damage, fail_closed)`；死亡回收（`$D616` 死亡门）对 `$4A` 亦未证 ⇒ 不回收敌槽。
pub fn apply_hit_damage(stage: &mut StageState, slot_index: usize) -> (u8, bool) {
    let enemy_type = stage.grid.slots[slot_index].type_;
    if enemy_type != BOSS_MID_TYPE {
        return (0, true);
    }
    let hp = stage.grid.slots[slot_index].f630;
    let new_hp = damage_apply(hp, HitDmgPath::Const2, 0);
    stage.grid.slots[slot_index].f630 = new_hp;
    (2, false)
}

/// shell 侧命中判定（task-38 §2.4）：弹注册（`shot_hitbox_register`）→ 敌框形（
/// `fetch_shape_boxes`，数据由装载侧喂 ROM bank2）→ 4 边+属性（`hit_resolves`）→
/// 清框（`clear_hit_entry`）+ 弹回收（`$87FD`）→ 受限额伤害（[`apply_hit_damage`]）。
///
/// **命中粒度 = 逐矩形消费**（task-42 F1 去重 → task-58 槽级 → task-67 按 F1-new 收口）：
/// 敌槽/矩形外层 → 弹内层（按槽序取首颗命中弹，`break`）→ **继续下一矩形**——每矩形每帧最多
/// 1 伤、同矩形同帧不回打；**不做**槽级跳过，故同一敌的多个矩形可各被 1 弹同帧命中 = engine
/// `combat_frame`（L236-285，每矩形 `break`）与 task-38 §2.4 L104 判据的 per-rect 语义
/// （task-63 复核多矩形探针已对拍：shell=2 伤 / engine=2 伤）。
///
/// 敌弹防护：玩家弹按 `type_code & 0x80 == 0` 过滤；敌对象槽若 `type_ & 0x80 != 0` 也跳过。
/// `shape_data` 缺失/无对应流 ⇒ 该敌槽跳过（fail-closed，不造框）。
pub fn resolve_player_hits(
    bullets: &mut BulletGrid,
    stage: &mut StageState,
    frame: u64,
) -> Vec<EnemyHitEvent> {
    let mut events = Vec::new();
    for si in 8..28usize {
        let enemy = stage.grid.slots[si];
        if enemy.type_ == 0 || enemy.type_ & 0x80 != 0 {
            continue;
        }
        let Some(addr) = shape_stream_addr(enemy.f518) else {
            continue;
        };
        // 取框在不可变借用作用域内完成（随后施伤需 &mut stage）。
        let rects = {
            let Some((_, stream)) = stage.shape_data.iter().find(|(a, _)| *a == addr) else {
                continue; // 框形数据缺失 ⇒ fail-closed（不造框）
            };
            fetch_shape_boxes(
                stream,
                enemy.x,
                enemy.y,
                enemy.f534 & 0x40 != 0,
                enemy.f534 & 0x80 != 0,
            )
        };
        let enemy_attr = enemy.f4c4 & 0xF0;
        for (left, right, top, bottom) in rects {
            let enemy_box = HitBoxEntry {
                left,
                right,
                top,
                bottom,
                owner: si as u8,
                attr: enemy_attr,
            };
            // 弹内层：按槽序取首颗命中弹（确定性）
            for &bullet_slot in PLAYER_SLOT_ORDER.iter() {
                if !is_player_bullet(bullets, bullet_slot) {
                    continue;
                }
                let bi = bullet_slot as usize;
                let shot_box = shot_hitbox_register(bullets.x[bi], bullets.y[bi], bullet_slot);
                if !hit_resolves(&shot_box, &enemy_box) {
                    continue;
                }
                // 命中：清框（`$D7DA` 一框一伤）+ 弹回收（`$87FD` 死亡处理器语义）
                let _cleared = clear_hit_entry(&enemy_box);
                release(bullets, bullet_slot);
                let (damage, fail_closed) = apply_hit_damage(stage, si);
                events.push(EnemyHitEvent {
                    frame,
                    enemy_slot: si as u8,
                    bullet_slot,
                    damage,
                    fail_closed,
                });
                break; // 本矩形只消费一颗弹（per-rect：同矩形同帧不回打；其余矩形继续）
            }
        }
    }
    events
}

/// 玩家子弹视图（20 槽，按 [`PLAYER_SLOT_ORDER`]；空槽=None）。
pub fn player_bullet_views(grid: &BulletGrid) -> [Option<BulletView>; 20] {
    let mut out = [None; 20];
    for (i, &slot) in PLAYER_SLOT_ORDER.iter().enumerate() {
        if is_player_bullet(grid, slot) {
            out[i] = Some(BulletView {
                slot,
                x: grid.x[slot as usize],
                y: grid.y[slot as usize],
                type_code: grid.type_code[slot as usize],
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage_with_shape() -> StageState {
        let mut stage = StageState::default();
        let addr = shape_stream_addr(0).expect("形 0 地址");
        stage.shape_data.push((addr, vec![1, 0xF0, 16, 0xF0, 16]));
        stage
    }

    /// 双矩形框形（相对敌坐标：左框 x∈[-40,-30]、右框 x∈[+30,+40]；y∈[-8,+8]）——
    /// 用于「同一敌不同矩形同帧各 1 伤」回归（task-67）。
    fn stage_with_two_rects() -> StageState {
        let mut stage = StageState::default();
        let addr = shape_stream_addr(0).expect("形 0 地址");
        // [count=2] + 两框 4B 有符号偏移 (左,右,上,下)
        stage
            .shape_data
            .push((addr, vec![2, 0xD8, 0xE2, 0xF8, 8, 30, 40, 0xF8, 8]));
        stage
    }

    fn enemy_at(stage: &mut StageState, slot: usize, type_: u8, x: u8, y: u8, attr: u8, hp: u8) {
        stage.grid.slots[slot] = game_core::engine::ObjectSlot {
            type_,
            x,
            y,
            f518: 0,
            f4c4: attr,
            f630: hp,
            ..game_core::engine::ObjectSlot::default()
        };
    }

    fn bullet_at(grid: &mut BulletGrid, slot: u8, x: u8, y: u8, type_idx: u8) {
        let i = slot as usize;
        grid.state[i] = 1;
        grid.action_ptr[i] = 0x0C;
        grid.x[i] = x;
        grid.y[i] = y;
        grid.motion[i] = 0;
        grid.type_code[i] = type_idx;
    }

    #[test]
    fn fire_gate_matches_39_3a_chain() {
        let mut timer = 1u8;
        assert!(fire_gate(true, &mut timer), "首按帧：$3A=1→DEC 0 ⇒ 生成");
        assert_eq!(timer, SHOT_TIMER_RELOAD);
        for _ in 0..(SHOT_TIMER_RELOAD - 1) {
            assert!(!fire_gate(true, &mut timer), "按住期 DEC 未归零不生成");
        }
        assert!(fire_gate(true, &mut timer), "第 7 帧（reload=7）再次生成");
        assert!(!fire_gate(false, &mut timer));
        assert_eq!(timer, 1, "B 松开 ⇒ $3A:=1");
        assert_eq!(reload_for_rapid(0), Some(7), "默认 STG-01 reload=7（bank02:$94DF[0]）");
        assert_eq!(reload_for_rapid(4), Some(2), "连射满级 reload=2");
        assert_eq!(reload_for_rapid(5), None, "域外 fail-closed");
    }

    #[test]
    fn generate_respects_inflight_cap_and_slot_order() {
        let mut g = BulletGrid::default();
        let spawned = generate_shots(&mut g, 128, 160, 0);
        assert_eq!(spawned, vec![0x80], "首次适应=组 0 首槽");
        assert_eq!(g.state[0x80], 1, "寿命=1（R-shot-gen-3）");
        assert_eq!(g.action_ptr[0x80], 0x0C, "脚本指针=配置参");
        assert_eq!(g.x[0x80], 128);
        assert_eq!(g.y[0x80], 160);
        for _ in 1..5 {
            let _ = generate_shots(&mut g, 128, 160, 0);
        }
        assert_eq!(inflight_for_type(&g, 0), 5, "达在途上限（cap=5）");
        assert!(generate_shots(&mut g, 128, 160, 0).is_empty(), "上限达 ⇒ 该型不生成");
    }

    #[test]
    fn tick_moves_and_releases_player_bullets() {
        // lifetime=1 → tick：DEC 0 取指（durations[0x0C]=0xFF/action=0x0C）→ 运动码 0x0C
        // LUT_X[0x0C]=0 / LUT_Y[0x0C]=-9 ⇒ (100,100)→(100,91)
        let mut g = BulletGrid::default();
        bullet_at(&mut g, 0x80, 100, 100, 0);
        assert_eq!(step_player_bullets(&mut g), 0, "未死亡");
        assert_eq!(g.state[0x80], 0xFF, "新寿命=durations[0x0C]");
        assert_eq!(g.motion[0x80], 0x0C);
        assert_eq!((g.x[0x80], g.y[0x80]), (100, 91), "LUT 双轴推进");
        // 负向借位（y=0 + LUT_Y=-9）⇒ 死亡回收
        let mut g2 = BulletGrid::default();
        bullet_at(&mut g2, 0x80, 100, 0, 0);
        g2.motion[0x80] = 0x0C;
        g2.state[0x80] = 0xFF;
        assert_eq!(step_player_bullets(&mut g2), 1, "溢出/借位 ⇒ 死亡回收");
        assert_eq!(g2.state[0x80], 0, "槽已释放");
    }

    #[test]
    fn hit_geometry_and_damage_whitelist() {
        let mut stage = stage_with_shape();
        enemy_at(&mut stage, 8, 0x2B, 100, 100, 0x10, 4);
        let mut g = BulletGrid::default();
        bullet_at(&mut g, 0x80, 100, 100, 0);
        let events = resolve_player_hits(&mut g, &mut stage, 7);
        assert_eq!(events.len(), 1, "弹/敌框重叠且属性相交 ⇒ 命中");
        assert_eq!(events[0], EnemyHitEvent {
            frame: 7,
            enemy_slot: 8,
            bullet_slot: 0x80,
            damage: 0,
            fail_closed: true,
        }, "非 $4A ⇒ 0 伤 fail-closed");
        assert_eq!(stage.grid.slots[8].f630, 4, "非白名单不写 f630（未证 HP 域）");
        assert_eq!(g.state[0x80], 0, "命中弹回收（$87FD）");

        // 负例 1：敌框远离 ⇒ 0 事件、弹不回收
        let mut stage2 = stage_with_shape();
        enemy_at(&mut stage2, 8, 0x2B, 200, 200, 0x10, 4);
        let mut g2 = BulletGrid::default();
        bullet_at(&mut g2, 0x80, 100, 100, 0);
        assert!(resolve_player_hits(&mut g2, &mut stage2, 8).is_empty(), "框远离 ⇒ 无命中");
        assert_ne!(g2.state[0x80], 0, "未命中弹不回收");

        // 负例 2：属性不相交 ⇒ 无命中（`$04C4 & 0x30 == 0`）
        let mut stage3 = stage_with_shape();
        enemy_at(&mut stage3, 8, 0x2B, 100, 100, 0x00, 4);
        let mut g3 = BulletGrid::default();
        bullet_at(&mut g3, 0x80, 100, 100, 0);
        assert!(resolve_player_hits(&mut g3, &mut stage3, 9).is_empty(), "属性不交 ⇒ 无命中");
        assert_ne!(g3.state[0x80], 0, "属性不交不回收弹");

        // 负例 3：敌弹（type bit7 置）不参与命中/不被回收
        let mut stage4 = stage_with_shape();
        enemy_at(&mut stage4, 8, 0x2B, 100, 100, 0x10, 4);
        let mut g4 = BulletGrid::default();
        bullet_at(&mut g4, 0x80, 100, 100, 0x80);
        assert!(resolve_player_hits(&mut g4, &mut stage4, 10).is_empty(), "敌弹不参与（友伤防护）");
        assert_ne!(g4.state[0x80], 0, "敌弹不被玩家命中逻辑回收");
    }

    #[test]
    fn two_bullets_same_frame_same_enemy_box_one_damage() {
        // F1 回归（task-42）：同帧 2 弹同敌槽同框 ⇒ 一框一伤、同帧不回打
        //（engine::combat_frame 敌/矩形外层 + 每矩 break 语义；task-38 §2.4）。修复前=2 伤。
        let mut stage = stage_with_shape();
        enemy_at(&mut stage, 8, BOSS_MID_TYPE, 100, 100, 0x10, BOSS_MID_HP);
        let mut g = BulletGrid::default();
        bullet_at(&mut g, 0x80, 100, 100, 0);
        bullet_at(&mut g, 0x81, 100, 100, 0);
        let events = resolve_player_hits(&mut g, &mut stage, 55);
        assert_eq!(events.len(), 1, "同帧同框只允许 1 次伤害（修复前=2）");
        assert_eq!(events[0].bullet_slot, 0x80, "低槽序弹优先消费（确定性）");
        assert!(!events[0].fail_closed);
        assert_eq!(stage.grid.slots[8].f630, BOSS_MID_HP - 2, "仅扣 1 次（14→12）");
        assert_ne!(g.state[0x81], 0, "第二弹未被消费（本帧已回打）");

        // 非白名单同帧 2 弹：仍只 1 条 fail_closed 事件、0 伤
        let mut stage2 = stage_with_shape();
        enemy_at(&mut stage2, 8, 0x2B, 100, 100, 0x10, 4);
        let mut g2 = BulletGrid::default();
        bullet_at(&mut g2, 0x80, 100, 100, 0);
        bullet_at(&mut g2, 0x81, 100, 100, 0);
        let ev2 = resolve_player_hits(&mut g2, &mut stage2, 56);
        assert_eq!(ev2.len(), 1, "非白名单同帧同框也只登记 1 次");
        assert!(ev2[0].fail_closed);
        assert_eq!(ev2[0].damage, 0);
        assert_eq!(stage2.grid.slots[8].f630, 4, "不写 f630");
        assert_ne!(g2.state[0x81], 0, "第二弹不消费");
    }

    #[test]
    fn two_bullets_same_frame_different_rects_of_same_enemy_two_damage() {
        // task-67：同一敌不同矩形、同帧各被 1 弹命中 ⇒ 2 伤（engine `combat_frame` 对拍）。
        // 修复前（槽级 slot_hit）=1 伤且第二弹保留（本用例红）。
        let mut stage = stage_with_two_rects();
        enemy_at(&mut stage, 8, BOSS_MID_TYPE, 128, 100, 0x10, BOSS_MID_HP);
        let mut g = BulletGrid::default();
        bullet_at(&mut g, 0x80, 93, 100, 0); // 左框 88..98 × 92..108
        bullet_at(&mut g, 0x81, 163, 100, 0); // 右框 158..168 × 92..108
        let events = resolve_player_hits(&mut g, &mut stage, 77);
        assert_eq!(events.len(), 2, "同一敌不同矩形同帧各 1 伤（engine 对拍；旧槽级=1）");
        assert_eq!(events[0].enemy_slot, 8);
        assert_eq!(events[1].enemy_slot, 8);
        assert!(events.iter().all(|e| e.damage == 2 && !e.fail_closed));
        assert_eq!(stage.grid.slots[8].f630, BOSS_MID_HP - 4, "14→10（每矩形各 2 伤）");
        assert_eq!(g.state[0x80], 0, "弹0 已回收");
        assert_eq!(g.state[0x81], 0, "弹1 已回收");
    }

    #[test]
    fn boss_whitelist_const2_14_to_zero_without_recycle() {
        let mut stage = stage_with_shape();
        enemy_at(&mut stage, 8, BOSS_MID_TYPE, 100, 100, 0x10, BOSS_MID_HP);
        let mut total = 0u8;
        for i in 0..7u8 {
            let mut g = BulletGrid::default();
            bullet_at(&mut g, 0x80, 100, 100, 0);
            let events = resolve_player_hits(&mut g, &mut stage, 100 + u64::from(i));
            assert_eq!(events.len(), 1, "第 {} 发命中", i + 1);
            assert_eq!(events[0].damage, 2, "$4A 白名单 Const2");
            assert!(!events[0].fail_closed);
            total = total.saturating_add(events[0].damage);
        }
        assert_eq!(total, 14, "7 发 ×2 = 14（engine_tick ⑫ 同口径）");
        assert_eq!(stage.grid.slots[8].f630, 0, "HP 14→0");
        assert_ne!(stage.grid.slots[8].type_, 0, "死亡回收门未证 ⇒ 不回收敌槽（fail-closed）");
    }

    #[test]
    fn views_only_player_bullets() {
        let mut g = BulletGrid::default();
        bullet_at(&mut g, 0x80, 10, 20, 2);
        bullet_at(&mut g, 0x85, 30, 40, 0x80); // 敌独占槽
        let views = player_bullet_views(&g);
        assert_eq!(views[0], Some(BulletView { slot: 0x80, x: 10, y: 20, type_code: 2 }));
        assert_eq!(views[1..].iter().filter(|v| v.is_some()).count(), 0, "敌弹/敌独占槽不入视图");
    }
}
