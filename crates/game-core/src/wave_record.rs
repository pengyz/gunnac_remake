//! game-core::wave_record —— 波次记录机（op5 流 `$CC/$CD` 记录机）CA/C9 判定链
//! + `$D86B` 状态转移表（批次4a）。
//!
//! 语义来源：`docs/findings/wave-ca-flags.md` §2（逐位语义，A 级逐指令）+ §3
//! （17/17 全谱相关性）+ §6（C9 侧同源快查）+ `docs/findings/stage-wave-crosscut.md`
//! §1/§2（八关模板恒定性）。**未证不发明**：$ED56 副支效果（S3-3b）、sub_8D8D 写表流
//! 内容语义（S2-5）⇒ 仅出判定出参，写集内容不发明。
//!
//! 判定链字节锚（bank00 固定窗=引擎全局，八关同源）：bit0 主支 8CFC-8D0A、bit0 副支
//! 8CA1-8CBE、bit7 XOR 8CC9-8CDB、bit1 8CDD-8CEB、bit3 8B4A-8B58、bit4&5 8A80-8A9C、
//! bit6 8C10-8C1B、sub_8DBD 8DBD-8DCB。

/// CA/C9 判定链出参（逐位语义表 wave-ca-flags §2 的机器可判定投影）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CaJudgment {
    /// CA bit0 主支：`$26|=$10` 臂关末门（8D02-8D06）。
    pub arm_stage_end_gate: bool,
    /// CA bit0 主支第二效果：`$D1:=0`（8D08-8D0A）。
    pub d1_clear: bool,
    /// CA bit0 副支条件成立 ⇒ `JSR $ED56`（效果未证 S3-3b，仅挂 pending）。
    pub aux_ed56_pending: bool,
    /// 速度恢复段（8CDD）执行 ⟺ **CA bit7 ≠ C9 bit1**（8CC9-8CDB 双支推导）。
    pub speed_restore: bool,
    /// CA bit1：`$82/$83` 滚动速度对恢复（8CDD-8CEB；仅恢复段内有效）。
    pub restore_scroll_pair: bool,
    /// CA bit3：`$C8:=6` 请求（8B4A-8B58）。
    pub force_c8_6_request: bool,
    /// bit3 且 `$C8==6` 已等 ⇒ 直接 RTS（8B54 `BEQ locret`，短路后段）。
    pub bail_rts: bool,
    /// CA bit4&5 **组合判**（8A80 `AND #$30/CMP #$30`）⇒ 分派 `$8A90`。
    pub write_table1_dispatch: bool,
    /// 写表1 执行 = 组合判 ∧ `$CB` bit1 门（分派≠执行）。
    pub write_table1_exec: bool,
    /// CA bit6：写表2（`LDY #$0C / JSR sub_8D8D`，8C10-8C18）。
    pub write_table2: bool,
}

/// `sub_8DBD`（8DBD-8DCB）：`SEC ⟺ C9 bit2=0 ∧ C9(bit0|bit3)≠0`。
pub fn sub_8dbd_sec(c9: u8) -> bool {
    c9 & 0x04 == 0 && (c9 & 0x09) != 0
}

/// C9 bit5 死分支 quirk（BUG-021，D4 保真）：`89ED LDA $C9 / AND #$20 / F0 00`
/// ——BEQ+0 目标=下一条指令，**零效果**。判定形态保留不优化；`probe` 返回 BEQ 条件值
/// （真=bit5 清），调用方**不得**据此分支（bit5 翻转对出参零影响，测试⑤锁死）。
pub fn c9_bit5_dead_branch_probe(c9: u8) -> bool {
    c9 & 0x20 == 0
}

/// CA/C9 判定链（波次记录机记录处理的标志分派，逐位语义 A 级）。
///
/// `flags_26`=`$26`、`flags_cb`=`$CB`、`state_c8`=`$C8`（bit3 门/bail 用）。
/// 副支 `$ED56` 效果未证 ⇒ 只出 `aux_ed56_pending`，写集不发明。
pub fn ca_judgment(c9: u8, ca: u8, flags_26: u8, flags_cb: u8, state_c8: u8) -> CaJudgment {
    let arm = ca & 0x01 != 0;
    let speed_restore = (ca & 0x80 != 0) != (c9 & 0x02 != 0);
    let combo = ca & 0x30 == 0x30;
    CaJudgment {
        arm_stage_end_gate: arm,
        d1_clear: arm, // 8D02-8D0A 同支双写（$26|=关末门 与 $D1:=0）
        aux_ed56_pending: !arm
            && sub_8dbd_sec(c9)
            && flags_26 & 0x88 == 0,
        speed_restore,
        restore_scroll_pair: speed_restore && ca & 0x02 != 0,
        force_c8_6_request: ca & 0x08 != 0,
        bail_rts: ca & 0x08 != 0 && state_c8 == 6,
        write_table1_dispatch: combo,
        write_table1_exec: combo && flags_cb & 0x02 != 0,
        write_table2: ca & 0x40 != 0,
    }
}

/// `$D86B` 状态转移表（bank14:$D86B，256 项，A 级字节精确）：
/// 索引=旧态（`$0400+槽`），值=新态（`$D62F` 命中收尾：`$D619/$D65C LDA $D86B,Y/X`）。
/// 已知锚：`$01→$0F`（玩家受击）、`$22→$12`（召唤物受击相位切换）、`$4A→$4A`
/// （中 boss 恒等——不靠此表死亡，走 `$D84F` 死亡门）、`$5F→$17`。
pub const STATE_TRANSITION: [u8; 256] = [
    0x00, 0x0F, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
    0x11, 0x11, 0x12, 0x11, 0x11, 0x11, 0x12, 0x12,
    0x12, 0x12, 0x2A, 0x12, 0x2C, 0x2D, 0x2E, 0x2F,
    0x12, 0x13, 0x12, 0x13, 0x12, 0x12, 0x12, 0x12,
    0x12, 0x13, 0x13, 0x14, 0x13, 0x3D, 0x13, 0x12,
    0x12, 0x12, 0x12, 0x12, 0x13, 0x14, 0x12, 0x13,
    0x12, 0x13, 0x4A, 0x17, 0x12, 0x12, 0x12, 0x13,
    0x13, 0x12, 0x12, 0x16, 0x12, 0x13, 0x12, 0x13,
    0x13, 0x11, 0x13, 0x12, 0x16, 0x13, 0x16, 0x17,
    0x16, 0x13, 0x62, 0x12, 0x64, 0x11, 0x66, 0x12,
    0x12, 0x12, 0x16, 0x16, 0x11, 0x16, 0x12, 0x16,
    0x13, 0x13, 0x16, 0x16, 0x13, 0x13, 0x12, 0x12,
    0x16, 0x16, 0x17, 0x12, 0x13, 0x13, 0x12, 0x16,
    0x13, 0x13, 0x12, 0x13, 0x13, 0x12, 0x12, 0x87,
    0x15, 0x12, 0x12, 0x12, 0x13, 0x17, 0x12, 0x13,
    0x12, 0x12, 0x16, 0x12, 0x13, 0x13, 0x96, 0x13,
    0x13, 0x15, 0x12, 0x13, 0x12, 0x9D, 0x13, 0x13,
    0xA0, 0x13, 0x12, 0x12, 0x13, 0x13, 0x14, 0x13,
    0x13, 0x12, 0x16, 0x16, 0x16, 0xAD, 0x13, 0xAF,
    0x12, 0x13, 0x13, 0x14, 0x13, 0x13, 0x13, 0x13,
    0x12, 0xB9, 0x12, 0x12, 0x13, 0x14, 0x11, 0x14,
    0x12, 0x14, 0x11, 0x17, 0x16, 0x17, 0x16, 0x11,
    0x12, 0x12, 0xCA, 0x16, 0x13, 0x13, 0xCE, 0xCF,
    0xA9, 0x24, 0x85, 0x55, 0xA9, 0x00, 0x85, 0x57,
    0xA9, 0x12, 0x85, 0x56, 0xAD, 0x8D, 0x01, 0xC9,
    0x10, 0x90, 0x11, 0x4A, 0x4A, 0x4A, 0x09, 0x01,
    0x85, 0x58, 0x20, 0xC5, 0xCF, 0xA9, 0x1B, 0x85,
    0x56, 0xAD, 0x8D, 0x01, 0x29, 0x0F, 0x38, 0x2A,
    0x85, 0x58, 0x20, 0xC5, 0xCF, 0xA9, 0x08, 0x85,
];

/// 状态转移查表（`$D86B[idx]`）。
pub fn state_transition(old_state: u8) -> u8 {
    STATE_TRANSITION[old_state as usize]
}

// ===== 批次4d④：终段投放器触发链（enemy-dropper-trigger.md，A 级）=====

/// `$FFFF` 流尾停泊哨兵：thr=16 位**精确相等**比较（`bank00:81A3-81AD`），
/// tick=16px 滚动进度（Wt-2）且滚动尽头冻结（关1 实测冻结 824）⇒ tick 永不达
/// `$FFFF` ⇒ **thr=$FFFF 记录永不触发**（其后内容=流尾停泊区；B 级意图=防越流尾）。
pub const STREAM_END_SENTINEL: u16 = 0xFFFF;

/// thr 触发判定（`81A3 LDA $84/CMP $86/BNE` + `81A9 LDA $85/CMP $87/BEQ`）：
/// 16 位精确相等（非 ≥——tick 单调递增时每个 thr 值恰命中一次，除 `$FFFF`）。
pub fn thr_fires(tick: u16, thr: u16) -> bool {
    tick == thr
}

/// 流尾停泊判定。
pub fn is_stream_end(thr: u16) -> bool {
    thr == STREAM_END_SENTINEL
}

/// op7 jump-if-wave-busy 出参（`bank00:8252-8275`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op7Action {
    /// `$C8≠0` ⇒ 跳转体（与 op3 共享 `$825A`）：流指针换目标 + **tick 重基**
    /// （新流头 2B 同写 `$84/$85`（tick）与 `$86/$87`（首阈值））。
    JumpRebase { target: u16 },
    /// `$C8=0` ⇒ 跳过 2B 直行（`8256 INY / JMP $82EF`）。
    Skip2B,
}

/// op7 分派（正常流程不可达——前驱 thr=`$FFFF` 哨兵；防御性尾段）。
pub fn op7_action(wave_state_c8: u8, target: u16) -> Op7Action {
    if wave_state_c8 != 0 {
        Op7Action::JumpRebase { target }
    } else {
        Op7Action::Skip2B
    }
}

/// op7 跳转目标字节登记（(关, 目标)——不可达段仅登记字节，不反推语义，S4b-5）。
pub const OP7_TARGETS: [(u8, u16); 3] = [(1, 0x080F), (2, 0x1B0F), (8, 0x090F)];

/// 终段投放模板判定（enemy-dropper-trigger §4）：末流 = **count=1 单体 +
/// C9 bit0 far-call bank11:$8012 + CA bit0 臂关末门**。
pub fn is_terminal_dropper(count: u8, c9: u8, ca: u8) -> bool {
    count == 1 && c9 & 0x01 != 0 && ca & 0x01 != 0
}

/// 终段投放实例登记（型值**不命名**——对象类型表值）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropperInstance {
    /// 关号。
    pub stage: u8,
    /// 流址（op5 装载）。
    pub stream: u16,
    /// 清单 count。
    pub count: u8,
    /// 清单型值序列（不命名）。
    pub types: &'static [u8],
    /// 记录 C9 标志。
    pub c9: u8,
    /// 记录 CA 标志（⚠ 与型值 `$CA` 同名异物）。
    pub ca: u8,
}

/// 已证实例（clean 模板对）：S1 `[01][$5F]`、S8 `[01][$CA]`。
/// S4 流2 `[02][$87,$87]`（CA=$01）为**多体例外**不入登记。
/// 【2026-01-15 矛盾裁定（412f96b）】far-call 门在 **C9**（`C9 bit0 ∧ $26 bit3=0`），
/// 与 CA 位无关——S4 流2 `C9=$FB` ⇒ far-call 照常条件触发（原"无 far-call"表述作废）。
pub const TERMINAL_DROPPERS: &[DropperInstance] = &[
    DropperInstance { stage: 1, stream: 0xA3E4, count: 1, types: &[0x5F], c9: 0xFB, ca: 0x05 },
    DropperInstance { stage: 8, stream: 0xB889, count: 1, types: &[0xCA], c9: 0xFB, ca: 0x81 },
];

/// far-call 触发门（`bank00:89C9-89D3`，2026-01-15 矛盾裁定 A 级）：
/// `C9 bit0==1 ∧ ($26 & $08)==0` → `JSR $E649` 内联 `0B 0C 12 80`（bank11:$8012）。
/// 第三门 `$26` bit3 = 运行期抑制（语义未名 S4b-8）——本函数为**静态门判定**，
/// 调用方传入运行期 `$26` 值。
pub fn far_call_fires(c9: u8, flags_26: u8) -> bool {
    c9 & 0x01 != 0 && flags_26 & 0x08 == 0
}
