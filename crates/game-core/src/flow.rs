//! game-core::flow —— 游戏流程模式状态机 + 跨模式数据流（M3 主动脉，t81）+ 死亡/GAME OVER/Continue 链（DEV-2，A2）。
//!
//! 语义来源：`docs/spec/ui-flow.md` R-ui-6/7（A 级流程段）+ R-shop-5..8 + R-respawn-*；
//! 死亡/GAME OVER/Continue 链 = `docs/spec/game-over-continue.md`（R-gameover-* / R-death-* /
//! R-continue-*），证据基准 `docs/findings/game-over-continue.md`（A2，§1-§4）。
//! **未证不发明**：残留 fail-closed = F13 免死支（`$06A0≠0`/`$32` bit7，C 级语义 findings §3.4，
//! 不入行为断言）、`$26` bit7 直回标题支（C 级 §3.4）、卖出机制；游戏环显示/音效 ⇒ 挂起枚举
//! （`DeathFx`），不发明其写集/渲染。
//!
//! 已证流程序列段（R-ui-6/7，场景 capture 标签锚定）：
//! - 开机：黑场 f180 → 标题 f360 → 菜单 f540+（title-menu 场景）；
//! - 菜单输入序列 → 商店 f1080 → 双 B → 近黑过渡 f1200 → 关卡 f1260 → 战斗 f1500+
//!   （gameplay-stage1 场景）；
//! - 死亡/重生循环（R-respawn-1 窗口，respawn-window 场景）。
//!
//! 两台状态机分立（ui.rs ShopState/ShopPurchaseState 同型先例）：
//! - **粗粒度**：`Mode`/`ModeEvent`/`FlowState::advance`（场景 capture 标签锚定，L5 合法性同源）；
//! - **细粒度**：`DeathChainState`/`death_chain_step`（A2 F1-F12 RAM 镜像 + 帧步进；
//!   粗粒度 `FlowState::on_game_over`/`on_continue` 为其投影，两层口径以细粒度为准）。

use crate::ui::MoneyCounter;

/// 游戏模式（已证段 + 待证段如实标注）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 开机初始化。
    Boot,
    /// 黑场（R-ui-7 f180）。
    Black,
    /// 标题（R-ui-7 f360）。
    Title,
    /// 菜单（R-ui-7 f540+）。
    Menu,
    /// 商店主面板（R-ui-6 f1080）。
    Shop,
    /// 商店信息页（R-shop-6，B 切入）。
    ShopInfo,
    /// 近黑过渡（R-shop-7/R-ui-6 f1200）。
    Transition,
    /// 关卡开始（R-ui-6 f1260）。
    StageStart,
    /// 实机战斗（R-ui-6 f1500+）。
    Combat,
    /// 死亡/重生循环段（R-respawn-1）。
    DeathRespawn,
    /// GAME OVER 待决（F6/F7 目标：`$26|=8` 后等待 Start，bank15:$E2BF-E2F0；
    /// 原 `GameOverUnproven` 未证占位按 A2 闭合）。
    GameOverPending,
    /// GAME OVER/CONTINUE 菜单（F8-F11：`$2F` 光标双出口，bank15:$E3AB-$E484；
    /// 原 `ContinueUnproven` 未证占位按 A2 闭合）。
    GameOverMenu,
}

/// 跨模式保持数据（②：金钱/武器槽/等级/弹药——respawn 复位与 shop 结束态有证据）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PersistData {
    pub money: MoneyCounter,
    pub weapon_slot: u8,   // $34
    pub weapon_level: u8,  // $33
    pub bomb_ammo: u8,     // $018F
}

impl PersistData {
    /// 死亡/重生复位（R-respawn-4 + bank03:$A376）：槽/级清 0、弹药回满 4；**金钱保持**。
    pub fn on_respawn(self) -> PersistData {
        PersistData {
            money: self.money,
            weapon_slot: 0,
            weapon_level: 0,
            bomb_ammo: 4,
        }
    }

    /// F12 续关复位（R-continue-reset）的 PersistData 镜像：`$E0-$E6` 清零
    /// （bank15:$E10A `JSR sub_ECBF`，双金钱清零之一）。
    /// `$33/$34/$018F` 不在 F12 复位集 ⇒ 保留；完整复位集（`$18D:=2`/`$189,$190,$18E:=0`/
    /// 网格清 `$6FE-$7FD`/`$0400:=1`/`$180` 保留）见 `DeathChainState`（`apply_continue_reset`）。
    pub fn on_continue(self) -> PersistData {
        PersistData { money: MoneyCounter::default(), ..self }
    }
}

/// 流程状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowState {
    pub mode: Mode,
    pub data: PersistData,
}

/// 场景 capture 标签 → 模式事件（A 级标签锚定）。
///
/// `GameOverArm`/`GameOverArrival`/`MenuToggle`/`Continue`/`GameFinish` 为 A2 静态链事件
/// （A 级汇编指针，见各自变体注）；无 capture 标签锚——GAME OVER 屏动态未观测
/// （findings §2.6 负结果），`event_from_tag` 不映射（golden 禁入，防发明动态锚）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeEvent {
    Black,
    Title,
    Menu,
    ShopArrival,
    ShopPage,
    ExitTransition,
    StageStart,
    MidCombat,
    DeathRespawn,
    /// F6/F7：死亡分叉进 GAME OVER 待决（bank03:$A387-A38E `$26|=8` / bank15:$E2A2-E2E2）。
    GameOverArm,
    /// F8：待决 → 菜单（Start `$16&$10` 或 `$0300≠0`，bank15:$E2E2-E2F0 → $E3AB）。
    GameOverArrival,
    /// F11：菜单 `$2F` 翻转自环（bank15:$E43E-E460）。
    MenuToggle,
    /// F9：`$2F==0`+Start → 续关（bank15:$E481 `JMP sub_E0D9`）。
    Continue,
    /// F10：`$2F==1`+Start → 回标题（bank15:$E484 `JMP loc_E073`）。
    GameFinish,
}

impl FlowState {
    pub fn new() -> Self {
        FlowState { mode: Mode::Boot, data: PersistData::default() }
    }

    /// 已证转移（R-ui-6/7 + R-shop-5..8 + A2 F6-F12 粗粒度）：返回下一状态；
    /// 不在已证边集内 ⇒ Err（fail-closed，不发明；F13 免死支等未证边不入边集）。
    pub fn advance(self, ev: ModeEvent) -> Result<FlowState, String> {
        use Mode::*;
        let next_mode = match (self.mode, ev) {
            (Boot, ModeEvent::Black) => Black,
            (Black, ModeEvent::Title) => Title,
            (Title, ModeEvent::Menu) => Menu,
            (Menu, ModeEvent::Menu) => Menu, // R-ui-7 菜单态持续（f540-900 多次采样同态）
            (Menu, ModeEvent::ShopArrival) => Shop,
            (Shop, ModeEvent::ShopPage) => ShopInfo,
            (ShopInfo, ModeEvent::ShopPage) => Shop,
            (Shop | ShopInfo, ModeEvent::ExitTransition) => Transition,
            (Transition, ModeEvent::StageStart) => StageStart,
            (StageStart, ModeEvent::MidCombat) => Combat,
            (Combat, ModeEvent::MidCombat) => Combat,
            (Combat, ModeEvent::DeathRespawn) => DeathRespawn,
            (DeathRespawn, ModeEvent::MidCombat) => Combat,
            (DeathRespawn, ModeEvent::DeathRespawn) => DeathRespawn,
            // ===== A2 死亡/GAME OVER/Continue 链粗粒度边（findings §4.2 F6-F12）=====
            (DeathRespawn, ModeEvent::GameOverArm) => GameOverPending, // F6/F7
            (GameOverPending, ModeEvent::GameOverArrival) => GameOverMenu, // F8
            (GameOverMenu, ModeEvent::MenuToggle) => GameOverMenu,     // F11 自环
            (GameOverMenu, ModeEvent::Continue) => StageStart,         // F9 → 关首
            (GameOverMenu, ModeEvent::GameFinish) => Title,            // F10
            (m, ev) => {
                return Err(format!("未证转移 {:?} + {:?}（fail-closed，不发明）", m, ev))
            }
        };
        let data = match (self.mode, next_mode) {
            (m, DeathRespawn) if m != DeathRespawn => self.data.on_respawn(), // R-respawn-4
            (GameOverMenu, StageStart) => self.data.on_continue(), // F12：$E0-$E6 清零
            _ => self.data,
        };
        Ok(FlowState { mode: next_mode, data })
    }

    /// F6/F7 GAME OVER 链入口（R-gameover-arm/enter，bank03:$A387-A38E / bank15:$E2BF-E2E2）。
    /// 粗粒度投影：模式 → `GameOverPending`（源态须为 `DeathRespawn`——窗尽分叉，fail-closed）。
    /// RAM 级效果（`$26|=8`、`$27|=$40`、`$188:=$FF`、`$0300:=0`、音效 `$11` 挂起）在
    /// `death_chain_step`（F5-F7）落，此处不重复镜像。
    pub fn on_game_over(self) -> Result<FlowState, String> {
        if self.mode != Mode::DeathRespawn {
            return Err(format!(
                "GAME OVER 入口源态须为 DeathRespawn（F6/F7 窗尽分叉），实际 {:?}（fail-closed）",
                self.mode
            ));
        }
        Ok(FlowState { mode: Mode::GameOverPending, data: self.data })
    }

    /// F9+F12 续关（R-continue / R-continue-reset，bank15:$E481 → sub_E0D9）。
    /// 粗粒度投影：`data.money`（`$E0-$E6`）清零（bank15:$E10A `JSR sub_ECBF`）+ 模式 → `StageStart`
    /// （源态须为 `GameOverMenu`，fail-closed）。完整复位集见 `DeathChainState`（`apply_continue_reset`）。
    pub fn on_continue(self) -> Result<FlowState, String> {
        if self.mode != Mode::GameOverMenu {
            return Err(format!(
                "续关源态须为 GameOverMenu（F9 菜单 `$2F==0` 出口），实际 {:?}（fail-closed）",
                self.mode
            ));
        }
        Ok(FlowState { mode: Mode::StageStart, data: self.data.on_continue() })
    }
}

impl Default for FlowState {
    fn default() -> Self {
        Self::new()
    }
}

/// 场景 capture 标签字符串 → 模式事件（probe/end 非模式事件）。
/// GAME OVER 链无 capture 标签（§2.6 负结果）⇒ 不映射，防发明动态锚。
pub fn event_from_tag(tag: &str) -> Option<ModeEvent> {
    Some(match tag {
        "boot-black" => ModeEvent::Black,
        "title" => ModeEvent::Title,
        "menu-540" | "menu-720" | "menu-900" | "menu-end" => ModeEvent::Menu,
        "shop-arrival" => ModeEvent::ShopArrival,
        "shop-page" => ModeEvent::ShopPage,
        "exit-transition" => ModeEvent::ExitTransition,
        "stage-start" => ModeEvent::StageStart,
        "mid-combat" => ModeEvent::MidCombat,
        _ => return None,
    })
}

// ===== DEV-2 死亡/GAME OVER/Continue 链（A2 F1-F13，findings game-over-continue.md §1-§4）=====
//
// 架构模板 = ui.rs `ShopPurchaseState`/`shop_purchase_step`（状态相位 + RAM 镜像 + 挂起枚举）。
// 帧内次序 = 行为段 → 碰撞段 → 主环段（ROM 帧序；trace 锚：f1787 转移帧 / f1788 结算帧 /
// f1915 同帧 0F→0→1）。结算算术调用 l2ports 同名移植函数（差分已裁，勿重写逻辑）。

/// 死亡窗长（gameplay-stage1 双事件实测 128f：f1787-1914 / f2046-2173）。
/// 窗长随事件源变化（respawn-window 场景 85f/129f，findings §1.3）——计时源精确归属未定谳
/// （R-G5：`$CD9D` 相位 vs `$05DC` 索引饱和）⇒ 为 `DeathChainState::window_len` 字段而非常量语义。
pub const DEATH_WINDOW_FRAMES: u16 = 128;

/// 菜单输入位（bank15:$E43A `AND #$10` Start / $E440 `#$20` Select / $E446 `#$08` Up /
/// $E450 `#$04` Down——注意与 ui.rs 商店域 `BTN_DOWN=0x24`/`BTN_SELECT=0xD0` 分域，勿混用）。
pub const MENU_BTN_START: u8 = 0x10;
pub const MENU_BTN_SELECT: u8 = 0x20;
pub const MENU_BTN_UP: u8 = 0x08;
pub const MENU_BTN_DOWN: u8 = 0x04;

/// 死亡链相位（A2 §4.1 状态图）。`released`（`$0400=0`）为窗尽同帧瞬态（trace f1915/f2174
/// 同帧 0F→0→1，watch 只见 0F→1）⇒ 不单列相位，F3 释放写与 F4/F7 分叉折叠于同一帧。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathPhase {
    /// `$0400=1` 存活（F1 源 / F4、F12 目标；关首=ALIVE）。
    Alive,
    /// `$0400=0x0F` 死亡窗（F1 目标；F5 首帧结算；F2 自环免疫）。
    Dead,
    /// gameover.pending（F7 目标：`$26&8≠0` 分叉；F8 源）。
    GameOverPending,
    /// gameover.menu（F8 目标；F9-F11 源，`$2F` 光标）。
    GameOverMenu,
    /// TITLE（F10 目标：`loc_E073` 回标题，链终局）。
    Title,
}

/// 死亡链挂起效果（游戏环显示/音效未证 ⇒ 挂起枚举，不发明其写集/渲染；变体名=已证语义名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathFx {
    /// 无挂起效果。
    None,
    /// 死亡音效 `$21`（bank03:$A39A `LDA #$21/JSR $ED3B`）——音效线未接，挂起。
    DeathSoundPending,
    /// GAME OVER 音效 `$11`（bank15:$E2DD-E2DF）——挂起。
    GameOverSoundPending,
    /// 菜单翻转音效 `$45`（bank15:$E45E-E460）——挂起。
    MenuToggleSoundPending,
    /// 菜单文本装载（bank15:$E418 `$75/$76:=$E4A0` 文本流：CONTINUE/GAME OVER，
    /// findings §2.3 记录格式）——显示线未接，挂起。
    MenuTextPending,
}

/// 死亡链 RAM 镜像 + 相位（字段括注 ROM 地址；命名=台账/规格已证名，未证语义 `todo_*` 登记不解释）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeathChainState {
    /// 相位。
    pub phase: DeathPhase,
    /// 窗帧计数（模型侧）：转移帧=1；窗满 `window_len` 后下一帧释放+分叉（f1787-1914 / f1915）。
    pub window_frames: u16,
    /// 窗长（模型侧；默认 `DEATH_WINDOW_FRAMES`，R-G5 开放 ⇒ 参数化）。
    pub window_len: u16,
    /// F5 一次性结算标记（模型侧）。
    pub settled: bool,
    /// `$0400` 对象网格 slot0 玩家态（1=ALIVE / 0x0F=DEAD / 0=释放）。
    pub player_state: u8,
    /// `$041C` 状态向量高（trampoline `bank14:$CB4F` 装入 `$A3`）。
    pub state_vec_hi: u8,
    /// `$0438` 状态向量低（trampoline 装入 `$A5`）。
    pub state_vec_lo: u8,
    /// `$04A8` 转移旧态备份（bank14:$D625）。
    pub old_state: u8,
    /// `$0668` 命中标志（bit4=`$10`，bank14:$D64C-D651；释放清零 $D11A）。
    pub hit_flags: u8,
    /// 对象网格 `$06FE-$07FF` 尾段镜像（索引 = addr − `$06FE`）：`[0..0x100]`=`$6FE-$7FD`
    /// （续关清零域尾段，AR63-5）；`[0x100..0x102]`=`$7FE-$7FF`（保留段，AD66-1：E18C 循环不清）；
    /// `[0x012]`=`$0710`（复活写 1，R-G3 语义开放）。
    pub obj_grid_tail: [u8; 0x102],
    /// `$26` 标志字节：bit3=不可复活位（GAME OVER 待决）、bit4=关末门（ending-sequence）、
    /// bit7=模式位（C 级 findings §3.4——分支不接）。
    pub revive_flags: u8,
    /// `$2F` 菜单光标（0=CONTINUE / 1=GAME OVER）。
    pub menu_cursor: u8,
    /// `$E0-$E6` 金钱计数器（N-0028 money_counter_bcd）。
    pub money_counter: MoneyCounter,
    /// `$017A-$017F` 入口减半域（findings §2.5 逐字节变换；语义开放 R-G1）。
    pub entry_bytes: [u8; 6],
    /// `$180` 关口号（续关保留）。
    pub stage: u8,
    /// `$188` 关卡介绍倒数计数器（N-0200）。
    pub intro_counter: u8,
    /// `$0189`（语义未定谳，登记不解释——F12 复位清零 bank15:$E162）。
    pub todo_0189: u8,
    /// `$018D` 命数（BCD，stage 开局 :=2）。
    pub lives: u8,
    /// `$018E` 商店金钱（0..255 二进制字节）。
    pub shop_money: u8,
    /// `$018F` 炸弹弹药（N-0017；初始化 bank00:9C5D :=4）。
    pub bomb_ammo: u8,
    /// `$0190` 炸弹类型索引（N-0030）。
    pub bomb_type_index: u8,
    /// `$27` 一次性入场标志（bit6=GAME OVER 入场已过 bank15:$E2C1；bit7=结局入场 ending-sequence §1）。
    pub entry_once_flags: u8,
    /// `$0300` 关末推进闸（ending-sequence §6：非零=飞离/关末动画完成事件）。
    pub stage_advance_gate: u8,
    /// `$C8`（语义未定谳，登记不解释——F12 复位清零 bank15:$E19B）。
    pub todo_c8: u8,
}

impl DeathChainState {
    /// stage 开局初值（bank15:$E0EF `$18D:=2` / $E1B6 `$0400:=1` / bank00:9C5D `$018F:=4`）。
    pub fn new() -> Self {
        DeathChainState {
            phase: DeathPhase::Alive,
            window_frames: 0,
            window_len: DEATH_WINDOW_FRAMES,
            settled: false,
            player_state: 1,
            state_vec_hi: 0,
            state_vec_lo: 0,
            old_state: 0,
            hit_flags: 0,
            obj_grid_tail: [0; 0x102],
            revive_flags: 0,
            menu_cursor: 0,
            money_counter: MoneyCounter::default(),
            entry_bytes: [0; 6],
            stage: 1,
            intro_counter: 0,
            todo_0189: 0,
            lives: 2,
            shop_money: 0,
            bomb_ammo: 4,
            bomb_type_index: 0,
            entry_once_flags: 0,
            stage_advance_gate: 0,
            todo_c8: 0,
        }
    }
}

impl Default for DeathChainState {
    fn default() -> Self {
        Self::new()
    }
}

/// A2 死亡链一帧推进（F1-F12）：帧内次序 = 行为段 → 碰撞段 → 主环段（ROM 帧序）。
/// `hit` = 本帧碰撞命中（F1 触发源）；`buttons` = `$16` 位图（菜单段 F8-F11）。
/// 未证域输入 fail-closed 不动作（F13 免死支、菜单/待决态命中等）。
pub fn death_chain_step(st: &mut DeathChainState, hit: bool, buttons: u8) -> DeathFx {
    // ── ① 行为段（bank02:82F8 表/自向量分发）──
    let mut fx = DeathFx::None;
    match st.phase {
        DeathPhase::Alive => { /* 常规战斗行为不在本链 */ }
        DeathPhase::Dead => {
            if st.window_frames >= st.window_len {
                // F3 窗尽释放（bank14:$D10C-D11A）：`$0400:=0`/`$041C:=0`/`$0668:=0`
                //（同块 `$0470/$048C/$04C4` 亚像素/命中盒字段属移动域，本链不镜像）。
                st.player_state = 0;
                st.state_vec_hi = 0;
                st.hit_flags = 0;
            } else {
                st.window_frames += 1;
                if !st.settled {
                    fx = settle_first_frame(st); // F5
                }
            }
        }
        DeathPhase::GameOverPending => {
            // F8（bank15:$E2E2-E2F0）：Start（`$16&$10`）或 `$0300≠0` → 菜单；否则停留（E2ED JMP sub_E21C）。
            if buttons & MENU_BTN_START != 0 || st.stage_advance_gate != 0 {
                // 菜单入场（bank15:$E3AB-E422）：`$017A-$017F` 减半（E3B9-E3CD）+ `$2F:=0`（E403）
                for b in &mut st.entry_bytes {
                    *b = (*b & 0xC0) | ((*b >> 1) & 0x1F);
                }
                st.menu_cursor = 0;
                st.phase = DeathPhase::GameOverMenu;
                fx = DeathFx::MenuTextPending; // E418 `$75/$76:=$E4A0` 文本流
            }
        }
        DeathPhase::GameOverMenu => {
            fx = step_game_over_menu(st, buttons); // F9/F10/F11
        }
        DeathPhase::Title => { /* 链终局 */ }
    }
    // ── ② 碰撞段（bank14:D613-D651 转移应用）──
    if hit {
        match st.phase {
            DeathPhase::Alive => {
                // F1 四写签名（`$D86B[1]=0x0F`；trace f1787/f2046）：`$0400:=0x0F`/`$041C:=0`/
                // `$04A8:=旧态`/`$0668|=$10`。
                st.old_state = st.player_state;
                st.player_state = 0x0F;
                st.state_vec_hi = 0;
                st.hit_flags |= 0x10;
                st.phase = DeathPhase::Dead;
                st.window_frames = 1;
                st.settled = false;
            }
            // F2 复撞免疫（`$D86B[0x0F]=0x0F` 恒等 → D61F BEQ 零写入）/ 未证域 fail-closed。
            _ => {}
        }
    }
    // ── ③ 主环段（bank15:E29A 逐帧分叉）──
    if st.phase == DeathPhase::Dead && st.player_state == 0 {
        if st.revive_flags & 0x08 == 0 {
            // F4 复活（bank15:$E2A2-E2BC）：`$18F:=max(4,$18F)`、`$0400:=1`、`$0710:=1`。
            st.bomb_ammo = st.bomb_ammo.max(4);
            st.player_state = 1;
            st.obj_grid_tail[0x012] = 1;
            st.phase = DeathPhase::Alive;
            st.window_frames = 0;
            st.settled = false;
        } else {
            // F7 GAME OVER 入场（bank15:$E2BF-E2E2）：一次性 `$27` bit6（E2C3 `CMP $27` 短路）。
            // 注：E2C9 `$26` bit7 → `loc_E073` 直回标题支为 C 级语义（§3.4）——未接线，登记不解释。
            if st.entry_once_flags & 0x40 == 0 {
                st.entry_once_flags |= 0x40;
                st.intro_counter = 0xFF; // E2D2
                st.stage_advance_gate = 0; // E2D7
                fx = DeathFx::GameOverSoundPending; // E2DD 音效 `$11`
            }
            st.phase = DeathPhase::GameOverPending;
        }
    }
    fx
}

/// F5 dead 态首帧结算（bank03:$A36E-A3A2，findings §1.3/F5——首帧口径，非复活边）：
/// 终死（`$18D==0`）置 `$26|=8` 并跳过减一（A387-A390 BNE）；否则 `$018D` BCD −1
/// （A392 `LDY #1/JSR sub_ED03/STA $18D`——复用 `l2ports::bcd_sub_ammo`，同一 ROM 子程序）；
/// `$018E` ÷2 两路同做（A39F `LSR`）；trampoline 装入自向量 `$A3:$A5`（A3A2 `JSR $CB4F`）；
/// 死亡音效 `$21` 挂起（A39A）。一次性：仅 dead 首行为帧调用。
fn settle_first_frame(st: &mut DeathChainState) -> DeathFx {
    if st.lives == 0 {
        st.revive_flags |= 0x08;
    } else {
        st.lives = crate::l2ports::bcd_sub_ammo(st.lives, 1).0;
    }
    st.shop_money = crate::l2ports::death_money_halve(st.shop_money);
    st.state_vec_hi = 0xA3;
    st.state_vec_lo = 0xA5;
    st.settled = true;
    DeathFx::DeathSoundPending
}

/// 菜单环（bank15:$E425-E484）：Start 确认（E43A `$16&$10`）→ `$2F` 双出口（E47D）；
/// Select/Up（仅 `$2F=1`）/Down（仅 `$2F=0`）⇒ `$2F^=1` + 音效挂起（E43E-E460）。
fn step_game_over_menu(st: &mut DeathChainState, buttons: u8) -> DeathFx {
    if buttons & MENU_BTN_START != 0 {
        if st.menu_cursor == 0 {
            // F9 续关（bank15:$E481 `JMP sub_E0D9`）：F12 复位集 → ALIVE 关首。
            apply_continue_reset(st);
        } else {
            // F10 回标题（bank15:$E484 `JMP loc_E073`）：`$180:=1`（E09A-E09C）、
            // `$017A-$017F` 清零（E0D1-E0D7）。同块 `$32:=$40`/`$3C:=0`/`$F1/$4E` 复位
            // 属 shop/未证域字段（不在本链镜像）——登记于 spec。
            st.entry_bytes = [0; 6];
            st.stage = 1;
            st.phase = DeathPhase::Title;
        }
        return DeathFx::None;
    }
    // F11 翻转（bank15:$E43E-E456）：Select 无条件；Up 仅 `$2F=1`；Down 仅 `$2F=0`。
    let toggle = buttons & MENU_BTN_SELECT != 0
        || (buttons & MENU_BTN_UP != 0 && st.menu_cursor == 1)
        || (buttons & MENU_BTN_DOWN != 0 && st.menu_cursor == 0);
    if toggle {
        st.menu_cursor ^= 1;
        return DeathFx::MenuToggleSoundPending; // E458-E460 音效 `$45`
    }
    DeathFx::None
}

/// F12 续关复位集（bank15:$E0EF-E1DD，R-continue-reset）+ `$26` 复活权（§2.1 E0F4-E0F8）：
/// `$18D:=2`、`$189/$190/$18E:=0`、`$E0-$E6:=0`（E10A sub_ECBF，双金钱清零）、
/// 网格清 `$0400-$06FF`+`$6FE-$7FD`（E18C 四页；`$7FE-$7FF` 不清——AD66-1）、`$0400:=1`、
/// `$27/$C8:=0`、`$188:=$40`（E1D3-E1DD `$26` bit7 下 `$FF`）、`$26` 两写点净效应 `&=$80`（见注 1）；
/// **仅 `$180` 保留**（续当前关），`$017A-$017F` 减半值带入（§3.2）。
/// 注 1（已裁定，复核 #69）：bank15:$E1BB-E1BF `$26:=$26&$80` 与 E0F4 瞬态写构成两写点序——
/// 净效应 `$26&=$80`（bit4 终态=0），已按终态镜像；原缺口记录如下（历史）
/// 已裁定（#69）并接线（终态镜像见下）。
/// 注 2：`$16/$18` 清（E1C5-E1C7）= 输入锁存写；本链按步传参无锁存 ⇒ 不镜像。
fn apply_continue_reset(st: &mut DeathChainState) {
    st.lives = 2; // E0EF
    st.revive_flags = (st.revive_flags & 0xF7) | 0x10; // E0F4-E0F8 瞬态写（先行）
    st.revive_flags &= 0x80; // E1BB-E1BF 终态写（后继必达——复核 #69 裁定：两写点净效应 $26&=$80，bit4=0）
    st.todo_0189 = 0; // E162
    st.bomb_type_index = 0; // E165
    st.shop_money = 0; // E168
    st.money_counter = MoneyCounter::default(); // E10A sub_ECBF
    // 网格清（E18C）：具名单元 + 尾段 `$6FE-$7FD`；`$7FE-$7FF` 保留（AD66-1）
    st.player_state = 0;
    st.state_vec_hi = 0;
    st.state_vec_lo = 0;
    st.old_state = 0;
    st.hit_flags = 0;
    st.obj_grid_tail[..0x100].fill(0);
    st.todo_c8 = 0; // E19B
    st.entry_once_flags = 0; // E1C3（再入场可重 arm）
    st.player_state = 1; // E1B6
    st.intro_counter = if st.revive_flags & 0x80 != 0 { 0xFF } else { 0x40 }; // E1D3-E1DD
    st.phase = DeathPhase::Alive; // F12 → ALIVE 关首
    st.window_frames = 0;
    st.settled = false;
}
