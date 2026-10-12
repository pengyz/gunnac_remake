//! game-core::enemy —— 敌对象网格语义（$1C 网格 28 槽 + 命中/清除链）。
//!
//! **架构归位注记（t71）**：t62 移植的 VM 内核/槽状态机/记录装载已迁往 `game-core::terrain`
//! （t65 定性：地形脚本机，`R-enemy-2` 消费端）；本模块保留敌对象网格侧语义（t68 命中清除链
//! + R-enemy-3/4 网格钳）。公开 API 迁移对照：
//! `enemy::EnemySlot`→`terrain::TerrainSlot`、`enemy::load_record`→`terrain::load_record`、
//! `enemy::vm_*`→`terrain::vm_*`（语义零变化，golden 不动）。
//!
//! 未证不发明：命中判定盒包络（W15 例程未定位，参数化候选）、$05DC HP 读备选（W18）。

// ============ 命中/清除链（t68；t66 样本集 + findings/hit-sampling-h16.md A 级） ============

/// `$05DC+n` 命中/清除倒数字段（W18 双读取**倒数读**实现：55 次自然递减旁证；
/// HP 读备选留 TODO，t66 双读未收口不发明）。
pub fn hit_countdown_tick(hit: u8) -> (u8, bool) {
    let new = hit.wrapping_sub(1);
    (new, new == 0)
}

/// 命中清除释放写集（golden 基准 = f1404 slot#8 字段列差分，t66 §1）：
/// 类型清 0（槽释放）+ $041C 族/亚像素族/倒数清 0 + $05C0 速度族置 $10。
/// 注：单样本基准（f1404）——超出此写集的行为未证，不发明。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GridSlot {
    pub type_: u8,     // $0400+n 类型/状态
    pub f1c: u8,       // $041C+n（f1404 差分列）
    pub x_subpix: u8,  // $0470+n
    pub y_subpix: u8,  // $048C+n
    pub speed: u8,     // $05C0+n 速度族
    pub hit_cnt: u8,   // $05DC+n 命中/清除倒数
}

/// 执行释放（f1404 写集逐值）。返回释放后的槽状态（写集与旧值无关，入参=写目标标识）。
pub fn hit_clear_release(_slot: &GridSlot) -> GridSlot {
    GridSlot {
        type_: 0,
        f1c: 0,
        x_subpix: 0,
        y_subpix: 0,
        speed: 0x10,
        hit_cnt: 0,
    }
}

/// 判定盒（**包络不发明**——W15 触发例程未定位）：`|dx|≤dx_max && |dy|≤dy_max`。
/// 默认值为样本包络候选（t66 §3：全部样本 |dx|≤9/|dy|≤15，框候选 ≤16×16；
/// 任务卡 |dx|≤12 候选取 dx 默认）——仅参数默认，非例程常数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitBox {
    pub dx_max: u8,
    pub dy_max: u8,
}

impl HitBox {
    /// 弹×敌候选盒（dx≤12 任务卡候选 / dy≤16 样本包络）。
    pub const SHOT_ENEMY: HitBox = HitBox { dx_max: 12, dy_max: 16 };
    /// 玩家×敌候选盒（t66 §2 两样本一致：|dx|≤12、|dy|≤12）。
    pub const PLAYER_ENEMY: HitBox = HitBox { dx_max: 12, dy_max: 12 };

    /// 接触判定（坐标差绝对值包络；W15 例程未定位——此谓词=样本拟合候选，非例程复刻）。
    pub fn contact(self, dx: i16, dy: i16) -> bool {
        let (ax, ay) = (dx.unsigned_abs() as u16, dy.unsigned_abs() as u16);
        ax <= self.dx_max as u16 && ay <= self.dy_max as u16
    }
}
