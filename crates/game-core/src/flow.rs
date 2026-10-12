//! game-core::flow —— 游戏流程模式状态机 + 跨模式数据流（M3 主动脉，t81）。
//!
//! 语义来源：`docs/spec/ui-flow.md` R-ui-6/7（A 级流程段）+ R-shop-5..8 + R-respawn-*。
//! **未证不发明**：GAME OVER / CONTINUE 路径、卖出机制（fail-closed TODO）。
//!
//! 已证流程序列段（R-ui-6/7，场景 capture 标签锚定）：
//! - 开机：黑场 f180 → 标题 f360 → 菜单 f540+（title-menu 场景）；
//! - 菜单输入序列 → 商店 f1080 → 双 B → 近黑过渡 f1200 → 关卡 f1260 → 战斗 f1500+
//!   （gameplay-stage1 场景）；
//! - 死亡/重生循环（R-respawn-1 窗口，respawn-window 场景）。

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
    /// 【未证段】GAME OVER——fail-closed，禁止从任何已证边到达。
    GameOverUnproven,
    /// 【未证段】CONTINUE——fail-closed。
    ContinueUnproven,
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
}

/// 流程状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowState {
    pub mode: Mode,
    pub data: PersistData,
}

/// 场景 capture 标签 → 模式事件（A 级标签锚定）。
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
}

impl FlowState {
    pub fn new() -> Self {
        FlowState { mode: Mode::Boot, data: PersistData::default() }
    }

    /// 已证转移（R-ui-6/7 + R-shop-5..8）：返回下一状态；
    /// 不在已证边集内 ⇒ Err（fail-closed，含 GameOver/Continue 未证段）。
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
            (m, ev) => {
                return Err(format!("未证转移 {:?} + {:?}（fail-closed，不发明）", m, ev))
            }
        };
        let data = if matches!(next_mode, DeathRespawn) && self.mode != DeathRespawn {
            self.data.on_respawn() // 进入重生段执行复位（R-respawn-4）
        } else {
            self.data
        };
        Ok(FlowState { mode: next_mode, data })
    }

    /// 【未证段】GAME OVER 路径：fail-closed（TODO，禁止调用）。
    pub fn on_game_over_unproven() -> Result<FlowState, String> {
        Err("GAME OVER 路径未证（H 系待证段）——不发明".to_string())
    }

    /// 【未证段】CONTINUE 路径：fail-closed（TODO，禁止调用）。
    pub fn on_continue_unproven() -> Result<FlowState, String> {
        Err("CONTINUE 路径未证——不发明".to_string())
    }
}

impl Default for FlowState {
    fn default() -> Self {
        Self::new()
    }
}

/// 场景 capture 标签字符串 → 模式事件（probe/end 非模式事件）。
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
