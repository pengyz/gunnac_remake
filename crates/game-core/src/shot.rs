//! game-core::shot —— 弹丸系统（M3 第三模块：六型调度/运动机/等级配置）。
//!
//! 语义来源：`docs/spec/shot-system.md` R-shot-gen-1..4 / R-shot-motion-1..3 / R-shot-level-1..4
//! （A 级，逐条搬运）+ `docs/spec/constants.md` v4 六表（数值真值源）。
//! **未证不发明**：A4（动作流/时长表界）、逐型齐射参数、碰撞命中（H-16）留空。
//! 弹对象 SoA（t32 $70 间距网格）：字段基址 $040E(lifetime)/$047E(script_ptr)/$04EE(pos_x)/
//! $055E(pos_y)/$05CE(motion_code)/$063E(shot_type)，槽位=基址+slot（字节步长）。
//!
//! quirk 保真（D4）：16 向 LUT 的 BMI 负向语义——负值走带进位加法的借位死亡判定
//! （bank2:$87E1/$87F6），正向溢出与负向借位都进死亡处理器（$87FD）。

use crate::weapons::CONFIG_OFFSETS;

/// C-lvl-offsets（$9457，12 项）——与 weapons::CONFIG_OFFSETS 同源（t39 已核）。
pub const LVL_OFFSETS: [u8; 12] = CONFIG_OFFSETS;

/// C-shot-cfg-stream（$9463 起 124 字节，(周期,脚本参) 对 + $FF 分段终止）。
pub const CFG_STREAM: [u8; 124] = [
    0x05, 0x0C, 0xFF, 0x03, 0x0C, 0x02, 0x0B, 0x02, 0x0D, 0xFF, 0x03, 0x0C,
    0x03, 0x0A, 0x03, 0x0E, 0xFF, 0x03, 0x0C, 0x03, 0x09, 0x03, 0x0F, 0x02,
    0x04, 0xFF, 0x03, 0x0C, 0x03, 0x20, 0x03, 0x22, 0x03, 0x05, 0x03, 0x03,
    0xFF, 0x03, 0x0C, 0x03, 0x08, 0x03, 0x00, 0x03, 0x14, 0x03, 0x16, 0xFF,
    0x02, 0x18, 0x02, 0x1A, 0x03, 0x10, 0x03, 0x12, 0x03, 0x07, 0x03, 0x01,
    0xFF, 0x03, 0x0C, 0x02, 0x10, 0x02, 0x12, 0x03, 0x20, 0x03, 0x22, 0x02,
    0x04, 0xFF, 0x03, 0x0C, 0x02, 0x1C, 0x02, 0x1E, 0x03, 0x10, 0x03, 0x12,
    0x03, 0x04, 0xFF, 0x03, 0x0C, 0x02, 0x0B, 0x02, 0x0D, 0x03, 0x04, 0x03,
    0x06, 0x03, 0x02, 0xFF, 0x03, 0x0C, 0x03, 0x10, 0x03, 0x12, 0x03, 0x08,
    0x03, 0x00, 0x02, 0x04, 0xFF, 0x03, 0x0C, 0x03, 0x10, 0x03, 0x12, 0x03,
    0x14, 0x03, 0x16, 0xFF,
];

/// C-motion-lut-x 域（$88BB 起 32B ROM 字节）：16 项正式表 + 表间字节（$88CB-$88CE）
/// + 下表首段——**无掩码索引**（t55 实测：motion=16 读 $88CB=$F8，掩码会伪造 +9）。
pub const LUT_X: [i8; 32] = [
    9, 8, 5, 3, 0, -3, -5, -8, -9, -8, -5, -3, 0, 3, 5, 8,
    -8, -8, 8, 8, 0, 3, 6, 8, 9, 8, 6, 3, 0, -3, -6, -8,
];

/// C-motion-lut-y 域（$88CF 起 32B ROM 字节）：同上无掩码索引。
pub const LUT_Y: [i8; 32] = [
    0, 3, 6, 8, 9, 8, 6, 3, 0, -3, -6, -8, -9, -8, -6, -3,
    -8, 12, -8, 12, 89, 91, 91, 93, 93, 93, 91, 91, 89, 91, 91, 93,
];

/// C-action-stream（$890B 观测区 36B；表界未证=A4，越界访问 fail-closed）。
pub const ACTION_STREAM: [u8; 36] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
    0x0C, 0x0D, 0x0E, 0x0F, 0x0A, 0x0C, 0x0E, 0x0C, 0x06, 0x0C, 0x02, 0x0C,
    0x10, 0x0C, 0x12, 0x0C, 0x0B, 0x0C, 0x0D, 0x0C, 0x08, 0x0C, 0x00, 0x0C,
];

/// C-action-durations（$892F 起观测 32B；0=跳转，非 0=新时长；表界未证=A4——动作流 36B
/// 对应尾部 4 字节时长未观测，越界访问 fail-closed panic）。
pub const ACTION_DURATIONS: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0x0A, 0xFF, 0x0A, 0xFF, 0x05, 0xFF, 0x05, 0xFF,
    0x01, 0xFF, 0x01, 0xFF, 0x04, 0xFF, 0x04, 0xFF,
];

/// 弹对象（SoA 六字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShotObject {
    pub lifetime: u8,     // $040E
    pub script_ptr: u8,   // $047E
    pub pos_x: u8,        // $04EE
    pub pos_y: u8,        // $055E
    pub motion_code: u8,  // $05CE
    pub shot_type: u8,    // $063E
}

/// 六型调度单步（R-shot-gen-1，bank3:$A1FD-$A20C）：`counter < period` ⇒ 生成并 INC 回写。
/// 返回 (新计数, 是否生成)。slot 分配（JSR $888D）属调用方前置。
pub fn schedule_step(counter: u8, period: u8) -> (u8, bool) {
    if counter < period {
        (counter.wrapping_add(1), true)
    } else {
        (counter, false)
    }
}

/// 生成初值链（R-shot-gen-3，bank3:$A20F-$A22B 五字段）：寿命=1、脚本指针=配置参、
/// 坐标=玩家、弹型=X（type index）。
pub fn spawn_init(slot: u8, type_idx: u8, cfg_idx: u8, player_x: u8, player_y: u8, script_param: u8) -> ShotObject {
    // slot/cfg_idx 为指令流上下文参数（槽位寻址与 $0766 查表由调用方完成）
    let _ = (slot, cfg_idx);
    ShotObject {
        lifetime: 1,
        script_ptr: script_param,
        pos_x: player_x,
        pos_y: player_y,
        motion_code: 0,
        shot_type: type_idx,
    }
}

/// 脚本机取指（R-shot-motion-1，bank2:$87BD-$87D8）：
/// `duration[ptr]`：非 0 ⇒ 新寿命=duration、运动码=action[ptr]、指针++；0 ⇒ 跳转（Y=action[ptr] 重取）。
/// 表界未证（A4）：观测区 36B，越界即 panic（fail-closed 不发明）。
pub fn script_fetch(ptr: u8, durations: &[u8], actions: &[u8]) -> (u8, u8, u8) {
    let mut y = ptr as usize;
    loop {
        let dur = *durations.get(y).expect("动作时长表越界（A4 表界未证）");
        if dur != 0 {
            let code = *actions.get(y).expect("动作流越界（A4 表界未证）");
            return (dur, code, (y as u8).wrapping_add(1));
        }
        y = *actions.get(y).expect("动作流越界（A4 表界未证）") as usize; // 跳转
    }
}

/// 16 向 LUT 累加单轴一步（R-shot-motion-2，bank2:$87DA-$8818 quirk 保真）：
/// 带符号加（负值走 BMI 分支）；正向溢出/负向借位 ⇒ None（进死亡处理器 $87FD）。
pub fn lut_accumulate_axis(pos: u8, motion_code: u8, lut: &[i8]) -> Option<u8> {
    let v = *lut.get(motion_code as usize).expect("LUT 域越界（A4 表界未证，fail-closed）");
    if v >= 0 {
        let sum = pos as u16 + v as u16;
        if sum > 0xFF {
            None // 正向溢出 → 死亡
        } else {
            Some(sum as u8)
        }
    } else {
        let sum = pos as i32 + v as i32;
        if sum < 0 {
            None // 负向借位 → 死亡
        } else {
            Some(sum as u8)
        }
    }
}

/// 等级→(周期,脚本参) 序列（R-shot-level-1/2）：`4+((级-4)&7)` 变换定位段（t39 H-A 口径），
/// 逐对读 CFG_STREAM 至负值终止。
pub fn config_pairs_for_level(level: u8) -> Vec<(u8, u8)> {
    let idx = crate::weapons::config_index_for_level(level) as usize;
    let mut i = LVL_OFFSETS[idx] as usize;
    let mut out = Vec::new();
    loop {
        let period = CFG_STREAM[i];
        if period & 0x80 != 0 {
            break; // $FF 负值终止
        }
        let param = CFG_STREAM[i + 1];
        out.push((period, param));
        i += 2;
    }
    out
}

/// 弹对象逐帧步进（R-shot-motion-1/2 整体，bank2:$87B3-$8818）：
/// 寿命==0 ⇒ 死亡（RTS）；否则 DEC，归零帧取新动作（脚本机）；随后 LUT 累加双轴。
/// 溢出/借位 ⇒ None（死亡处理器 $87FD，边界判定未证域——死亡帧不入 L3 比对面）。
pub fn tick(obj: &mut ShotObject) -> Option<()> {
    if obj.lifetime == 0 {
        return None;
    }
    obj.lifetime -= 1;
    if obj.lifetime == 0 {
        let (dur, code, ptr) = script_fetch(obj.script_ptr, &ACTION_DURATIONS, &ACTION_STREAM);
        obj.lifetime = dur;
        obj.motion_code = code;
        obj.script_ptr = ptr;
    }
    obj.pos_x = lut_accumulate_axis(obj.pos_x, obj.motion_code, &LUT_X)?;
    obj.pos_y = lut_accumulate_axis(obj.pos_y, obj.motion_code, &LUT_Y)?;
    Some(())
}

/// 弹槽分配结果（bank2:$888D 退出约定：命中 ⇒ C=0/A=state=0/Y=槽偏移；满载 ⇒ SEC/Y=A=$F0）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotAlloc {
    /// 首次适应命中（组序×组内序，低偏移优先）。
    Found { slot: u8, a: u8 },
    /// 满载：SEC，Y=A=$F0（`TYA/ADC #$18` 遗留值，t48 case-4 ROM 实测）。
    Full,
}

/// 弹槽分配器（bank2:$888D-$88BA，t48 定谳 `docs/findings/shot-slot-geometry.md`）：
/// 4 组×5 槽（$80-$84/$9C-$A0/$B8-$BC/$D4-$D8，组内 stride 1、组距 $1C），
/// 空槽判定 state[Y]==0，首次适应首命中返回；回收=state:=0（调用方写）。
/// `state` 按槽偏移 Y 索引（$040E+Y 的镜像数组）。
pub fn shot_slot_alloc(state: &[u8]) -> SlotAlloc {
    const GROUPS: [u8; 4] = [0x80, 0x9C, 0xB8, 0xD4];
    for base in GROUPS {
        for k in 0..5u8 {
            let y = base.wrapping_add(k);
            let a = *state.get(y as usize).expect("state 数组须覆盖槽偏移");
            if a == 0 {
                return SlotAlloc::Found { slot: y, a };
            }
        }
    }
    SlotAlloc::Full
}

// ===== 批次3：弹网格 SoA + 敌弹槽分配 + 槽位交叠裁定（arena-midboss §1.1 A 级）=====

/// 弹网格六数组 SoA 镜像（同槽偏移 Y 索引六字段，arena-midboss §1.1）：
/// `$040E+Y` 寿命/动作倒数（**空槽判定==0**）、`$047E+Y` 动作流指针、`$04EE+Y` x、
/// `$055E+Y` y、`$05CE+Y` 运动指令码、`$063E+Y` 型码。
/// 自机弹与**敌弹同一弹网格**（非独立表、非 28 槽对象网格）；槽位集交叠裁定见
/// `PLAYER_ONLY_SLOTS`/`SHARED_SLOTS`/`ENEMY_ONLY_SLOTS`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulletGrid {
    /// `$040E+Y` 寿命/动作倒数（空槽判定 == 0）。
    pub state: [u8; 0x100],
    /// `$047E+Y` 动作流指针。
    pub action_ptr: [u8; 0x100],
    /// `$04EE+Y` x。
    pub x: [u8; 0x100],
    /// `$055E+Y` y。
    pub y: [u8; 0x100],
    /// `$05CE+Y` 运动指令码。
    pub motion: [u8; 0x100],
    /// `$063E+Y` 型码：敌弹恒 `$80`（bit7 置）；自机弹=生成器 X=0..5（bit7 清）+杂值 7。
    pub type_code: [u8; 0x100],
}

impl Default for BulletGrid {
    fn default() -> Self {
        BulletGrid {
            state: [0; 0x100],
            action_ptr: [0; 0x100],
            x: [0; 0x100],
            y: [0; 0x100],
            motion: [0; 0x100],
            type_code: [0; 0x100],
        }
    }
}

impl BulletGrid {
    /// 空槽判定（两分配器同判 `$040E+Y==0`）。
    pub fn is_empty(&self, slot: u8) -> bool {
        self.state[slot as usize] == 0
    }

    /// 回收（弹回收路 `bank2:$87FD` 槽释放语义）：state:=0。
    pub fn release(&mut self, slot: u8) {
        self.state[slot as usize] = 0;
    }
}

/// 敌弹型码（`bank3:$ABFB LDA #$80/STA $063E,Y`；实测独占 4 槽 197/197 恒 `$80`）。
pub const ENEMY_SHOT_TYPE: u8 = 0x80;

/// 槽位交叠裁定（arena-midboss §1.1，A 级算术 + 快照实测）：**distinct 24 = 20+12−8**。
/// 禁止按"20+12 不相交"建模——8 槽共享**时分复用**（两分配器同判空槽首适应，
/// 同一瞬时不会双主，同一槽位两族交替占用；运行时安全由同判 `state==0` 保证，
/// 不由槽位独占保证）。
///
/// 自机独占 12（低偏移 3×4 组）。
pub const PLAYER_ONLY_SLOTS: [u8; 12] = [
    0x80, 0x81, 0x82, 0x9C, 0x9D, 0x9E, 0xB8, 0xB9, 0xBA, 0xD4, 0xD5, 0xD6,
];
/// 共享 8（时分复用交叠槽）。
pub const SHARED_SLOTS: [u8; 8] = [0x83, 0x84, 0x9F, 0xA0, 0xBB, 0xBC, 0xD7, 0xD8];
/// 敌弹独占 4（每组高偏移 1）。
pub const ENEMY_ONLY_SLOTS: [u8; 4] = [0x85, 0xA1, 0xBD, 0xD9];

/// 敌弹槽分配器（`bank3:$AC42-$AC63`）：4 组×3 槽（`$83-$85/$9F-$A1/$BB-$BD/$D7-$D9`，
/// `LDY #$83` 起组内连续查 3、`ADC #$1A` 组跳），空槽判定 `state[Y]==0` 首适应；
/// 全满 SEC ⇒ `None`。与自机分配器共享空槽判定 ⇒ 共享 8 槽时分复用。
/// `state` 按槽偏移 Y 索引（`$040E+Y` 的镜像）。
pub fn enemy_slot_alloc(state: &[u8]) -> Option<u8> {
    const GROUPS: [u8; 4] = [0x83, 0x9F, 0xBB, 0xD7];
    for base in GROUPS {
        for k in 0..3u8 {
            let y = base.wrapping_add(k);
            if *state.get(y as usize).expect("state 数组须覆盖槽偏移") == 0 {
                return Some(y);
            }
        }
    }
    None
}

// ===== 批次3b：命中框注册 + 重叠判定 + 伤害公式（arena-midboss §2 A 级）=====

/// 命中框表条目（21 条/帧，索引 `$74`；弹/敌/玩家同表注册）。
///
/// 弹侧注册（`bank2:$8847-$888A`）：左=x−4（钳 0）/右=x+3（钳 FF）/上=y−5（钳 0）/
/// 下=y+5（钳 FF）/归属=槽偏移（bit7 置=弹条目）/属性掩码=常量 `$30`。
/// **弹命中框跨度 7×10px**（x−4..x+3、y−5..y+5；跨度口径 7×10，含端点覆盖 8×11px——
/// 复核 #80 F8 注）。
///
/// ⚠ 模型框≠游戏命中框（W-tas 注）：7×10 是**清弹/注册框**；游戏命中判定走
/// `bank14:$D78B/$D7ED` 双阅读端 4 边比较 + **属性相交**（`$04C4,X & $07E8,Y ≠ 0`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitBoxEntry {
    /// `$0780` 左。
    pub left: u8,
    /// `$0795` 右。
    pub right: u8,
    /// `$07AA` 上。
    pub top: u8,
    /// `$07BF` 下。
    pub bottom: u8,
    /// `$07D4` 归属（弹=槽偏移 bit7 置；对象=槽号 0..27 bit7 清）。
    pub owner: u8,
    /// `$07E9` 属性掩码（弹侧常量 `$30`；敌侧 `$04C4,X & $F0`）。
    pub attr: u8,
}

/// 弹侧命中框注册（`bank2:$8847-$888A`）：7×10 钳位四边 + 归属槽偏移 + 属性 `$30`。
/// `slot` = 弹槽偏移（`$80` 起，天然 bit7 置=弹归属判别）。
pub fn shot_hitbox_register(x: u8, y: u8, slot: u8) -> HitBoxEntry {
    HitBoxEntry {
        left: x.saturating_sub(4),                 // 8847 x−4 钳 0
        right: x.saturating_add(3).min(0xFF),      // 885B x+3 钳 FF（u8 自然饱和即 FF）
        top: y.saturating_sub(5),                  // 8868 y−5 钳 0
        bottom: y.saturating_add(5).min(0xFF),     // 8875 y+5 钳 FF
        owner: slot,                               // 8881 TXA
        attr: 0x30,                                // 8885 LDA #$30
    }
}

/// 重叠判定（`bank14:$D78B-$D7EC` / `$D7ED-$D846` 双阅读端 4 边比较）：
/// `$07BE,Y≥$72` 且 `$73≥$07A9,Y` 且 `$0794,Y≥$70` 且 `$71≥$077F,Y`
/// = 本函数 `a.bottom>=b.top && b.bottom>=a.top && a.right>=b.left && b.right>=a.left`。
/// **命中还需要属性相交**（见 `hit_resolves`）——几何重叠≠命中。
pub fn boxes_overlap(a: &HitBoxEntry, b: &HitBoxEntry) -> bool {
    a.bottom >= b.top && b.bottom >= a.top && a.right >= b.left && b.right >= a.left
}

/// 属性相交测试（团队过滤 `LDA $04C4,X / AND $07E8,Y ≠ 0 → 命中`）。
pub fn attrs_intersect(a_attr: u8, b_attr: u8) -> bool {
    a_attr & b_attr != 0
}

/// 综合命中（几何 4 边 + 属性相交；双阅读端同判据）。
pub fn hit_resolves(a: &HitBoxEntry, b: &HitBoxEntry) -> bool {
    boxes_overlap(a, b) && attrs_intersect(a.attr, b.attr)
}

/// 一框一伤（命中收尾 `$D7DA`：AND 结果 bit7 清 → 清被中条目框 `$07E8/$07A9/$07BE:=0`）。
/// 返回清框后的条目（attr/top/bottom 清 0；left/right/owner 不动）。
///
/// 写集↔字段名映射（arena-midboss §1.4/§2.1 表，索引=条目号−1 的 −1 偏移）：
/// `$07E8`=`$07E9` 属性掩码 −1、`$07A9`=`$07AA` 上 −1、`$07BE`=`$07BF` 下 −1
/// ⇒ 清框三写 = (attr, top, bottom)；left `$0780`/right `$0795`/归属 `$07D4` 不在写集。
pub fn clear_hit_entry(e: &HitBoxEntry) -> HitBoxEntry {
    HitBoxEntry {
        left: e.left,
        right: e.right,
        top: 0,
        bottom: 0,
        owner: e.owner,
        attr: 0,
    }
}

/// 伤害公式（`bank14:$D674`，X=被击者）：
/// - `$4C==0`（语义未证域）→ 直接 HP:=0（一击死）——**参数化待证，不入默认路径**；
/// - 弹命中（归属 bit7 置）：HP −= **2**（`$D684-$D688 SBC #$02`）；
/// - 物撞（归属 bit7 清）：HP −= **对方 `$0630`**（`$D69C`；撞击伤害=对方 HP 值）。
/// 返回新 HP（0/下溢饱和 0）；HP 归 0 ⇒ 调用方走 `$D616` 状态转移死亡门。
pub fn damage_apply(hp: u8, owner_is_shot: bool, opponent_hp: u8) -> u8 {
    let dmg: u16 = if owner_is_shot { 2 } else { u16::from(opponent_hp) };
    u16::from(hp).saturating_sub(dmg).min(0xFF) as u8
}
