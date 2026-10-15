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

// ===== 批次4b：敌型行为面（spawn 三写 + 型签名登记 + 中 boss HP 链）=====
//
// 证据：`docs/findings/enemy-timeline.md` §3（行为指针+字段签名，A/B 级）+
// `docs/findings/arena-midboss.md` §2.2/§3.1（中 boss HP 语义链，A 级）。
// 红线：型值=对象类型表值**不命名**；行为 VM 脚本体（t59 族）未证不发明——
// 本节只落三写/签名登记/HP 算术，运动与弹型行为不发明。

/// spawn 三写出参（`bank00:8B89-8B95` 同帧三写 `$400/$438/$41C`，A 级字节）：
/// `type_code→$400+槽`、`beh_lo→$438+槽`、`beh_hi→$41C+槽`。
/// 行为指针读法（B1 §3）：`$41C+槽`=hi / `$438+槽`=lo。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnTriple {
    /// `$400+槽` 对象类型码（不命名）。
    pub type_code: u8,
    /// `$41C+槽` 行为指针高字节。
    pub beh_hi: u8,
    /// `$438+槽` 行为指针低字节。
    pub beh_lo: u8,
}

/// 三写装载（`8B8B STA $400,Y / 8B90 STA $438,Y / 8B95 STA $41C,Y`）。
pub fn spawn_triple_write(type_code: u8, beh_ptr: u16) -> SpawnTriple {
    SpawnTriple {
        type_code,
        beh_hi: (beh_ptr >> 8) as u8,
        beh_lo: (beh_ptr & 0xFF) as u8,
    }
}

/// 敌型签名登记（**观察登记**非行为模型）：trace 实测 `type → (行为指针, 出生坐标)`。
/// 出生坐标仅登记**恒定**观测（$2F=(128,8) 9 签名、$2A=(29,10) 3 点）；
/// 摆动/后动族（$2B/$4E/$02/$07）坐标不恒定 ⇒ `spawn=None`（不发明）。
/// 同型多行为指针按观测对登记（$07 双指针），勿 1:1 强绑。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeSignature {
    /// 对象类型码（不命名）。
    pub type_code: u8,
    /// 行为指针（`$41C:$438`，trace 实测值）。
    pub beh_ptr: u16,
    /// 恒定出生坐标（x,y）；不恒定 ⇒ None。
    pub spawn: Option<(u8, u8)>,
}

/// 已证签名全集（B1 §3 特征表 + §2.3 trace 锚 + `wave_timeline.py spawns` 全量观测对）。
/// 观测口径=槽激活 `0→非0` 判定式（槽复用盲区——被覆盖写不可见，签名集**可能不全**）。
/// `beh=$0000` 观测（装载暂态/被中写先于指针写）**不登记**——非行为指针签名。
pub const TYPE_SIGNATURES: &[TypeSignature] = &[
    // $2F 组长族（ctrl2/12 出敌型）：$89DF、(128,8) 恒定、128f×9
    TypeSignature { type_code: 0x2F, beh_ptr: 0x89DF, spawn: Some((128, 8)) },
    // $2A 流定型（ctrl10）：$9010、(29,10) 恒定（3 点）；另一观测指针 $9079（坐标不恒定）
    TypeSignature { type_code: 0x2A, beh_ptr: 0x9010, spawn: Some((29, 10)) },
    TypeSignature { type_code: 0x2A, beh_ptr: 0x9079, spawn: None },
    // $2B/$4E 组员族：$00DC 恒定；坐标摆动（x 序列 128→102→82→98→102→90）；
    // $2B 另有 $0079 单点观测（(34,4)——样本 1 不证恒定 ⇒ None）
    TypeSignature { type_code: 0x2B, beh_ptr: 0x00DC, spawn: None },
    TypeSignature { type_code: 0x2B, beh_ptr: 0x0079, spawn: None },
    TypeSignature { type_code: 0x4E, beh_ptr: 0x00DC, spawn: None },
    // $07 三指针：底部点 $A631（y=155×4+177×1 不恒定）/ 尾段流 $AB30（y=13）/ $AAD9（y=80）
    TypeSignature { type_code: 0x07, beh_ptr: 0xA631, spawn: None },
    TypeSignature { type_code: 0x07, beh_ptr: 0xAB30, spawn: None },
    TypeSignature { type_code: 0x07, beh_ptr: 0xAAD9, spawn: None },
    // $02 编队（5 槽同帧 216f 轮）：$AB74
    TypeSignature { type_code: 0x02, beh_ptr: 0xAB74, spawn: None },
    // $05 尾段混编：$B011（单点观测）
    TypeSignature { type_code: 0x05, beh_ptr: 0xB011, spawn: None },
];

/// 按型值取签名（观测对全集；未登记/不发明 ⇒ 空）。
pub fn type_signatures(type_code: u8) -> Vec<&'static TypeSignature> {
    TYPE_SIGNATURES
        .iter()
        .filter(|s| s.type_code == type_code)
        .collect()
}

/// 中 boss（type `$4A`）HP 语义链（arena-midboss §2.2/§3.2，A 级）：
/// HP 字段=`$0630+槽`（本型 slot27=`$064B`）初值 **14**（v2-diag f9930 快照）；
/// 弹命中 −2（`$D688 SBC #$02`）/ 物撞 −=对方 `$0630`（`$D69C`）；HP 归 0 ⇒
/// `$D84F` 族死亡门（本结构只出死亡判定，爆炸/释放链不发明）。
/// `$D86B[$4A]=$4A` 恒等——**不靠状态转移表死亡**（`wave_record::state_transition`）。
/// `$05F7`（`$05DC+27`）恒 0 = **非 HP**（W18 反证收口，`hit_cnt` 独立字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Midboss {
    /// `$0630+27` 血量。
    pub hp: u8,
    /// `$04DF`（`$04C4+27`）属性掩码（快照 `$13`）。
    pub attr: u8,
    /// `$05F7`（`$05DC+27`）命中倒数（恒 0——非 HP）。
    pub hit_cnt: u8,
}

impl Default for Midboss {
    fn default() -> Self {
        Self::new()
    }
}

impl Midboss {
    /// HP 初值（A 级快照 14）。
    pub const HP_INIT: u8 = 14;
    /// 行为相位续点（`$0437:$0453`，快照 `$AC34`/`$AC75`；链语义 B 级挂档不发明）。
    pub const BEH_CONTINUATIONS: [u16; 2] = [0xAC34, 0xAC75];

    /// 初值装载（HP=14/attr=$13/hit_cnt=0——快照口径）。
    pub fn new() -> Self {
        Midboss {
            hp: Self::HP_INIT,
            attr: 0x13,
            hit_cnt: 0,
        }
    }

    /// 弹命中一发（`damage_apply` 弹路 −2）：返回**本发是否致死**（hp 归 0 且此前 >0）。
    pub fn hit_shot(&mut self) -> bool {
        self.take_damage(true, 0)
    }

    /// 物撞（伤害=对方 `$0630`，非 −2）：返回本撞是否致死。
    pub fn hit_body(&mut self, opponent_hp: u8) -> bool {
        self.take_damage(false, opponent_hp)
    }

    fn take_damage(&mut self, owner_is_shot: bool, opponent_hp: u8) -> bool {
        if self.hp == 0 {
            return false; // 已死不重复致死（饱和不回绕）
        }
        self.hp = crate::shot::damage_apply(self.hp, owner_is_shot, opponent_hp);
        self.hp == 0
    }
}

/// 组长族释放装载（四参 `(组员type, 组员数, 间隔帧, 参数)` → 字段写 + `GroupRelease`）。
/// 字段写点=`$518/$630/$534/$470`（批次2 `wave::item_effect` 同链）；组员节律=帧域
/// `$534` 直接兑现（批次2 `wave::GroupRelease`）。返回 (族状态, `$630` 首现=数−1)。
/// `count==0`/`interval==0` 未观测 ⇒ None（fail-closed）。
pub fn leader_family_spawn(args: [u8; 4]) -> Option<(LeaderFamily, u8)> {
    let [member_type, count, interval_frames, param] = args;
    let (release, first_seen) = crate::wave::GroupRelease::load(count, interval_frames)?;
    Some((
        LeaderFamily {
            member_type,
            interval_frames,
            param,
            release,
        },
        first_seen,
    ))
}

/// 组长族状态（$2E/$2F 组长四参镜像 + 组员释放机构）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaderFamily {
    /// `$518+槽` 组员 type。
    pub member_type: u8,
    /// `$534+槽` 出敌间隔（**帧**）。
    pub interval_frames: u8,
    /// `$470+槽` 参数。
    pub param: u8,
    /// 组员释放节律（`$630` 递减 + 帧域间隔）。
    pub release: crate::wave::GroupRelease,
}

// ===== 批次4c：组员族运动机（enemy-motion-kinematics §2-3，A 级字节）=====
//
// 摆动动力学引擎核心=mod-8 计数器×三表进给（`bank05:8C8D-8CA9` 逐指令）。
// **B 级拟合不入行为断言**（§2.1 正弦方程 x(t)≈127+63·sin/T≈104f=B 级拟合，
// 引擎真身是本节三表；轨迹族①-④是观察面，行为相位选择随型值/行为指针——
// 脚本体（t59 族）未证不发明）。

/// 三角波表 `$AC5F`（→`$684+槽`，B1 §3 复核 #76 F2 真值：对称三角波）。
pub const TRI_WAVE_684: [u8; 8] = [0xD3, 0xD7, 0xDB, 0xDF, 0xE3, 0xDF, 0xDB, 0xD7];

/// 三角波表 `$AC67`（→`$534+槽`，经 `($2B&1) EOR`）。
pub const TRI_WAVE_534: [u8; 8] = [0x03, 0x43, 0x43, 0x43, 0x03, 0x03, 0x03, 0x03];

/// 三角波表 `$AC6F`（→`$518+槽`）。
pub const TRI_WAVE_518: [u8; 8] = [0x24, 0x24, 0x24, 0x24, 0x21, 0x24, 0x24, 0x24];

/// 运动机一步出参（`bank05:8C8D-8CA9` 写集）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionOut {
    /// `$5DC+槽` 计数器 INC 后值（8-bit 自然回绕；索引用其 mod-8）。
    pub counter_next: u8,
    /// `$684+槽` := `$AC5F[idx]`。
    pub v684: u8,
    /// `$534+槽` := `($2B&1) EOR $AC67[idx]`。
    pub v534: u8,
    /// `$518+槽` := `$AC6F[idx]`。
    pub v518: u8,
}

/// 运动机一步（`8C8D INC $5DC,X / AND #$07 / TAY` + 三表写）。
///
/// **字段复用两段生命周期**（§3.2）：`$518/$534+槽` 被本机重写——四参链（投放控制期）
/// 与运动机（速度分量期）共用同字段，**不可混用语义**（stg02 §3.4 写主现身）。
/// `phase_2b` = zp `$2B` 值（**仅 bit0** 入 EOR）。
pub fn motion_step(counter: u8, phase_2b: u8) -> MotionOut {
    let counter_next = counter.wrapping_add(1);
    let idx = (counter_next & 7) as usize;
    MotionOut {
        counter_next,
        v684: TRI_WAVE_684[idx],
        v534: (phase_2b & 1) ^ TRI_WAVE_534[idx],
        v518: TRI_WAVE_518[idx],
    }
}

/// 型值迁移时间线（Wt-5 升级素材，A 级实测——关1 slot09 同槽三段）：
/// `f1479 $2B`（摆动族）→ `f1595 $12`（驻留）→ `f1627 $4E`（直线下降）。
/// 行为相位随型值切换；**迁移触发语义未证**（不发明）。
pub const TYPE_MIGRATION_SLOT09: [(u32, u8); 3] = [(1479, 0x2B), (1595, 0x12), (1627, 0x4E)];
