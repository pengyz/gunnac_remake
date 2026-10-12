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

/// C-motion-lut-x（$88BB，16 项，有符号）。
pub const LUT_X: [i8; 16] = [9, 8, 5, 3, 0, -3, -5, -8, -9, -8, -5, -3, 0, 3, 5, 8];

/// C-motion-lut-y（$88CF，16 项，有符号）。
pub const LUT_Y: [i8; 16] = [0, 3, 6, 8, 9, 8, 6, 3, 0, -3, -6, -8, -9, -8, -6, -3];

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
pub fn lut_accumulate_axis(pos: u8, motion_code: u8, lut: &[i8; 16]) -> Option<u8> {
    let v = *lut.get((motion_code & 0x0F) as usize).expect("LUT 越界");
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
