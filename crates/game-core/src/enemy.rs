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

/// 行为字段登记值（**c4e6b68 裁定**双类——出生帧瞬态快照定谳）：
/// bit7=1 ⇒ 自向量真指针；bit7=0 ⇒ 表分发模式种子（**非指针**）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorField {
    /// 自向量指针（`$41C:$438` hi bit7=1，`bank02:82F8` 直跳目标；$89DF/$89E4/$A631 族）。
    BehPtr(u16),
    /// 表分发种子（hi bit7=0——`$438` 出生帧快照值族，per-type/per-instance 状态字段，
    /// 8314 `DEC $438` duration 候选 S4b-9 挂账；f+1 起 threaded 链接管改写）。
    DispatchSeed(u8),
}

/// 敌型签名登记（**观察登记**非行为模型）：trace 实测 `type → (行为字段, 出生坐标)`。
/// 出生坐标仅登记**恒定**观测（$2F=(128,8) 9 签名、$2A=(29,10) 3 点、$2E=(128,8) 5 点）；
/// 摆动/后动族坐标不恒定 ⇒ `spawn=None`（不发明）。同型多值按观测对登记，勿 1:1 强绑。
/// 【c4e6b68 dated】B1 §3「行为指针恒 $00DC」作废——$00DC 系 `$438` **出生帧瞬态快照**
/// （0→非0 判定式恰落出生帧=表分发模式唯一可见帧；f+1 起改写直跳链入口，
/// 如 $56 ($00,$DC)→($B2,$C0)→链进 $B2D0→$B2F8 threaded 自续链）——按
/// `DispatchSeed` 改标（非残值非纯标记：per-type 状态字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeSignature {
    /// 对象类型码（不命名）。
    pub type_code: u8,
    /// 行为字段（双类判据=hi bit7，见 `BehaviorField`/`dispatch_mode`）。
    pub beh: BehaviorField,
    /// 恒定出生坐标（x,y）；不恒定 ⇒ None。
    pub spawn: Option<(u8, u8)>,
}

/// 已证签名全集（B1 §3 特征表 + §2.3 trace 锚 + `wave_timeline.py spawns` 全量观测对）。
/// 观测口径=槽激活 `0→非0` 判定式（槽复用盲区——被覆盖写不可见，签名集**可能不全**）。
/// `beh=$0000` 观测（装载暂态/被中写先于指针写）**不登记**——非行为指针签名。
/// 【c4e6b68 裁定收口】trace `beh=` 列 hi bit7=0 值（$00DC/$00F8/$00AC/$00CA/$004C…）
/// = **出生帧瞬态快照**：0→非0 判定式恰落出生帧（表分发模式唯一可见帧），
/// `($00 种子/$438 值)`，f+1 起改写为 threaded 自续链入口（$56: ($00,$DC)→($B2,$C0)
/// →链进 $B2D0→$B2F8）。$438 先写于 $41C（8B8E-8B95）=中间态可见性解释，非主因。
/// 种子值族=per-type/per-instance 状态字段（8314 `DEC $438` duration 候选，S4b-9
/// 挂账）——非残值亦非纯标记。B1 §3「行为指针恒 $00DC」作废（快照伪影，
/// enemy-timeline §3 三处 dated）。
pub const TYPE_SIGNATURES: &[TypeSignature] = &[
    // $2F 组长族（ctrl2/12 出敌型）：$89DF、(128,8) 恒定、128f×9
    TypeSignature { type_code: 0x2F, beh: BehaviorField::BehPtr(0x89DF), spawn: Some((128, 8)) },
    // $2A 流定型（ctrl10）：$9010、(29,10) 恒定（3 点）；另一观测指针 $9079（坐标不恒定）
    TypeSignature { type_code: 0x2A, beh: BehaviorField::BehPtr(0x9010), spawn: Some((29, 10)) },
    TypeSignature { type_code: 0x2A, beh: BehaviorField::BehPtr(0x9079), spawn: None },
    // $2E 组长族（ctrl1）：【2026-10-15 补（stg02 §4.4 帧锚 run）】$89E4、(128,8)
    // 五点恒定（f3776/3936/4512/4704/4832）——原"beh 未观测"缺口由 19000f run 补上
    TypeSignature { type_code: 0x2E, beh: BehaviorField::BehPtr(0x89E4), spawn: Some((128, 8)) },
    // $2B/$4E 组员族：$00DC 恒定；坐标摆动（x 序列 128→102→82→98→102→90）；
    // $2B 另有 $0079 单点观测（(34,4)——样本 1 不证恒定 ⇒ None）
    TypeSignature { type_code: 0x2B, beh: BehaviorField::DispatchSeed(0xDC), spawn: None },
    TypeSignature { type_code: 0x2B, beh: BehaviorField::DispatchSeed(0x79), spawn: None },
    TypeSignature { type_code: 0x4E, beh: BehaviorField::DispatchSeed(0xDC), spawn: None },
    // $07 三指针：底部点 $A631（y=155×4+177×1 不恒定）/ 尾段流 $AB30（y=13）/ $AAD9（y=80）
    TypeSignature { type_code: 0x07, beh: BehaviorField::BehPtr(0xA631), spawn: None },
    TypeSignature { type_code: 0x07, beh: BehaviorField::BehPtr(0xAB30), spawn: None },
    TypeSignature { type_code: 0x07, beh: BehaviorField::BehPtr(0xAAD9), spawn: None },
    // $02 编队（5 槽同帧 216f 轮）：$AB74
    TypeSignature { type_code: 0x02, beh: BehaviorField::BehPtr(0xAB74), spawn: None },
    // $05 尾段混编：$B011 / $AE6B（各单点观测，stg02 §4.4）
    TypeSignature { type_code: 0x05, beh: BehaviorField::BehPtr(0xB011), spawn: None },
    TypeSignature { type_code: 0x05, beh: BehaviorField::BehPtr(0xAE6B), spawn: None },
    // stg02 §4.4 run 新观测自向量指针对（hi bit7=1）：$58→$B3B9、$51→$B04C
    TypeSignature { type_code: 0x58, beh: BehaviorField::BehPtr(0xB3B9), spawn: None },
    TypeSignature { type_code: 0x51, beh: BehaviorField::BehPtr(0xB04C), spawn: None },
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
/// `f1479 $2B`（摆动族）→ `f1592 $12`（驻留）→ `f1627 $4E`（直线下降）。
/// 【2026-10-15 更正（F-D90）】~~f1595~~ → **f1592**：1595=§2.1 正弦拟合右极混记，
/// 迁移帧=1592 逐帧直读。行为相位随型值切换；**迁移触发语义未证**（不发明）。
pub const TYPE_MIGRATION_SLOT09: [(u32, u8); 3] = [(1479, 0x2B), (1592, 0x12), (1627, 0x4E)];

/// 行为分发模式（`bank02:82F8-8311`，A 级逐指令：`LDA $41C,X / BPL $8307`）：
/// `vec_hi` bit7=1 ⇒ **自向量**（`$41C:$438` 为跳转目标，`STA $25/LDA $438/STA $24/JMP ($24)`）；
/// bit7=0 ⇒ **表分发**（`$80CE[Y]:$7FFF[Y]`，Y=状态码 `$400+槽`——此模式下 `$41C:$438`
/// **非跳转目标**）。
///
/// 关联挂账：trace `beh=` 列 hi bit7=0 值的语义裁定（模式标记/写中间态/另有语义）转
/// decomp-analyst——本函数只落分派判据字节义，不预判那批值的语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchMode {
    /// bit7=1 自向量（$41C:$438 直接跳转）。
    SelfVector,
    /// bit7=0 表分发（状态表间接跳转）。
    TableDispatch,
}

/// 分派模式判据（bit7 测试）。
pub fn dispatch_mode(vec_hi: u8) -> DispatchMode {
    if vec_hi & 0x80 != 0 {
        DispatchMode::SelfVector
    } else {
        DispatchMode::TableDispatch
    }
}

// ===== 批次4f：t59 行为 VM 相位 handler（behavior-vm-scripts §3/§5，A 级）=====
//
// threaded-code 状态机的确定性引擎原语。轨迹族↔handler 配对=B 级待 S4b-2 相位
// 实测（不入行为断言）；$83xx handler 全集=S4b-9 待枚举（本文 6 样本面）。

/// `$836E` 恒速下沉 handler（族②/④引擎核）：`LDA #1/CLC/ADC $4FC,X` —— y+1/帧，
/// **溢出（y=$FF）即终止**（`BCS → $D10C`）；续行 `LDA #$40 / JMP $83DE`
/// （A:=`$40` 交参数化 handler）。返回 None=终止（槽回收）。
pub fn handler_const_descent(y: u8) -> Option<u8> {
    let (y_next, carry) = y.overflowing_add(1);
    if carry {
        None // BCS → $D10C（终止即回收）
    } else {
        Some(y_next)
    }
}

/// `$8314` duration 倒数 handler（行为终止条件①）：`DEC $438,X / BEQ` ——
/// 倒数归零 ⇒ None（`JMP $D10C`）；否则 Some(倒数值)（`RTS` 续行）。
/// `$438` 字段三义之一（直跳 lo/duration/表模式无效——按模式分支勿混，D4 保真）。
pub fn handler_duration_tick(duration_438: u8) -> Option<u8> {
    let d = duration_438.wrapping_sub(1);
    if d == 0 {
        None
    } else {
        Some(d)
    }
}

/// `$83DE` 参数化 handler 参数表（bank02 `$8406`——`$518,X := $8406+Y`，$0A/$0B/$0C 族）。
pub const PARAM_TABLE_518: [u8; 5] = [0x0A, 0x0B, 0x0C, 0x0C, 0x0C];

/// `$83DE` 参数表（bank02 `$840B`——`$684,X := $840B+Y`，$5F/$61/$63/$65/$67 族）。
pub const PARAM_TABLE_684: [u8; 5] = [0x5F, 0x61, 0x63, 0x65, 0x67];

/// `$8341` RNG jitter 原语（摆动候选族引擎核）：`JSR $EC12/TAY` 后双向抖动——
/// x 侧 `AND #$0F/SBC #7/ADC $4E0,X`、y 侧 `TYA/LSR×4/SBC #7/ADC $4FC,X`。
/// **carry 语义**：SBC/ADC 链逐位依赖进位——x 侧进位=$EC12 返回 carry（S4b-9 挂账
/// 未证）；y 侧进位=LSR 链末位（rng bit3，确定性）。返回 (dx, dy) 有符号偏移。
pub fn rng_jitter_offsets(rng: u8, carry_after_rng: bool) -> (i8, i8) {
    // x 侧：A=rng&$0F，SBC #7（borrow=1−C），ADC 尾步与 $4E0 合成偏移 = (A−7−!C)+C_after_sbc
    let a_x = rng & 0x0F;
    let (sub_x, borrow_x) = a_x.overflowing_sub(7 + if carry_after_rng { 0 } else { 1 });
    let c_after_sbc_x = !borrow_x;
    let dx = (i16::from(sub_x) + i16::from(c_after_sbc_x as u8)) as i8;
    // y 侧：A=rng>>4，进位=rng bit3（LSR×4 末移出位）
    let a_y = rng >> 4;
    let carry_y = rng & 0x08 != 0;
    let (sub_y, borrow_y) = a_y.overflowing_sub(7 + if carry_y { 0 } else { 1 });
    let c_after_sbc_y = !borrow_y;
    let dy = (i16::from(sub_y) + i16::from(c_after_sbc_y as u8)) as i8;
    (dx, dy)
}

/// `$D10C` = 槽回收/终止**唯一出口**（行为机与分配器②③路径共用——回收即终止，
/// 终止即回收）。枚举仅登记语义归属，调用侧负责槽字段清写（CB42-CBA1 初始化段）。
pub const TERMINATE_RECYCLE_EXIT: u16 = 0xD10C;

/// threaded-code 续链写（`$438,X:=lo / $41C,X:=hi`——handler 自改写指针→下一 handler，
/// behavior-vm-scripts §1 自续链族 8337→$836E/8381→$838E 实证）。返回 `(lo, hi)`=
/// **写序对**（lo 先 hi 后，与出生三写 8B8E-8B95 同型——中间态可见性同源）。
pub fn chain_rewrite(next_handler: u16) -> (u8, u8) {
    ((next_handler & 0xFF) as u8, (next_handler >> 8) as u8)
}

// ===== 批次4g-2：调用环（bank02:827D-82CD，A 级逐指令）=====

/// 调用环槽序表（bank02 `$82D0` 40B，字节精确）：前 20=槽 8..27 **升序**（基 Y=0）、
/// 后 20=槽 27..8 **降序**（基 Y=$14）——分组判据 `($54&$2B&2)!=0` 选基址。
pub const CALL_LOOP_SLOT_TABLE: [u8; 40] = [
    0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
    0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1B, 0x1A, 0x19, 0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12,
    0x11, 0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09, 0x08,
];

/// 首轮：槽 0..7 全槽轮（`827D-828C`：`LDY $400,X` type≠0 ⇒ `JSR $82F8` 分发）。
pub fn call_loop_round1() -> [u8; 8] {
    core::array::from_fn(|i| i as u8)
}

/// 次轮槽序（`829E-82CB` 20 次：`LDX $82D0,Y`）：
/// `group_flag`=`($54&$2B&2)!=0` —— true ⇒ 基 Y=$14 降序 27..8；false ⇒ 升序 8..27。
/// 双序=帧组交替公平性（D4 保真面：同负载槽遍历次序可分歧）。
pub fn call_loop_slots(group_flag: bool) -> [u8; 20] {
    let base = if group_flag { 20 } else { 0 };
    core::array::from_fn(|i| CALL_LOOP_SLOT_TABLE[base + i])
}

/// MMC3 按型切 bank（`82AA-82C0`）：type≥`$30` ⇒ 查 `$816E+type` 表取 R7 bank 值，
/// **与 zp `$01` 缓存同值则跳过**（`CMP $01/BEQ` 防重写）；否则 `$8000:=7`（bank select）
/// + `$8001:=值`（bank data）。返回 Some(新 bank 值)=需切换；None=type<$30 或同值。
/// 表区 `$816E+type`（type $30..$FF ⇒ $819E..$826D）——语义=per-type R7 窗映射
/// （CHR/代码 bank 配置，N-0151 域；**表值语义未证**仅登记）。
pub fn type_bank_switch(type_: u8, current_bank: u8, bank_table: &[u8]) -> Option<u8> {
    if type_ < 0x30 {
        return None;
    }
    let v = *bank_table.get(type_ as usize)?;
    if v == current_bank {
        None
    } else {
        Some(v)
    }
}

// ===== 批次4g-3：进场族/参数化段族模板（behavior-vm-handlers §1，A 级登记）=====

/// 进场族模板（6 型同构：改写续链+CBAF+8689+863B+ED3B 登记+[$4E 门/字段预置/y+=$91 支]
/// +尾跳参数化段）。字段预置=($072C 相位索引, $0710 循环上限)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryTemplate {
    /// 进场入口。
    pub entry: u16,
    /// 续链目标（`$438/$41C` 改写值，见 `chain_rewrite`）。
    pub chain_target: u16,
    /// `ED3B` 登记参数（音画域 S2-5，仅透传）。
    pub ed3b_param: u8,
    /// 字段预置 (072C, 0710)；无预置 ⇒ None。
    pub preset: Option<(u8, u8)>,
    /// 尾跳目标（参数化段入口）。
    pub tail: u16,
}

/// 进场族全集（6 型）。
pub const ENTRY_TEMPLATES: [EntryTemplate; 6] = [
    EntryTemplate { entry: 0x83BA, chain_target: 0x83DC, ed3b_param: 0x23, preset: None, tail: 0x83DC },
    EntryTemplate { entry: 0x8410, chain_target: 0x8432, ed3b_param: 0x24, preset: None, tail: 0x8432 },
    EntryTemplate { entry: 0x845A, chain_target: 0x8486, ed3b_param: 0x24, preset: Some((0x00, 0x05)), tail: 0x8486 },
    EntryTemplate { entry: 0x8497, chain_target: 0x84C3, ed3b_param: 0x24, preset: None, tail: 0x84C3 },
    EntryTemplate { entry: 0x85B1, chain_target: 0x85D3, ed3b_param: 0x26, preset: None, tail: 0x85D3 },
    EntryTemplate { entry: 0x85EA, chain_target: 0x8616, ed3b_param: 0x20, preset: Some((0x03, 0x08)), tail: 0x85D3 },
];

/// 段 A 表（`$83AE`→`$518`）——【099c510 dated】~~速度参~~ **形状索引**
///（$CC6A=形状/OAM 写入尾；$518→$970B/$9762 流指针）。
pub const SEG_A_T518: [u8; 4] = [0x0A, 0x0B, 0x0C, 0x0C];
/// 段 A 表（`$83B2`→`$684`）——【099c510 dated】~~速度参2~~ **tile 基址**
///（流字节+$684→$0201 OAM）。
pub const SEG_A_T684: [u8; 4] = [0x5F, 0x61, 0x65, 0x67];
/// 段 A 表（`$83B6`→`$534`，第三表——A 段独有）——【099c510 dated】
/// **OAM attr**（&$E3/bit6 翻转/|2=attr 位），非速度参。
pub const SEG_A_T534: [u8; 4] = [0x01, 0x00, 0x01, 0x00];
/// 段 C 表（`$844C`→`$518`）——【099c510 dated】**形状索引**（非速度参）。
pub const SEG_C_T518: [u8; 7] = [0x0B, 0x0C, 0x0D, 0x0E, 0x0D, 0x0C, 0x0C];
/// 段 C 表（`$8453`→`$684`）——【099c510 dated】**tile 基址**（非速度参）。
pub const SEG_C_T684: [u8; 7] = [0x61, 0x63, 0x69, 0x6D, 0x69, 0x65, 0x67];

/// 参数化段族（6 型 A-F：`LDA #A/LDY #Y/JSR $CD9D` → 失败/超限 `$D10C`；成功 ⇒
/// Y 索引查参数表写 `$518/$684/$534` → `JMP $CC6A` 公共运动尾）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamSegment {
    /// 段入口。
    pub entry: u16,
    /// `$CD9D` A 参字面量（运动模式选择器——**内核 S4b-9b 未证**，仅签名登记）；
    /// None=调用者供给。【更正】E 段入口=`$84C3`（`LDA #$50/LDY #$0F/JSR $CD9D`，
    /// ROM 直读 f=$44D3）——原记 `$84C7` 系 JSR 子址（seed 该区失真读点偏移，decomp 补谳）。
    pub cd9d_a: Option<u8>,
    /// `$CD9D` Y 参字面量（表索引基数）；None=调用者供给。
    pub cd9d_y: Option<u8>,
    /// 旁路入口（跳过 A 装载的落点——B 段 `$83DE`=`$836E` 尾带 `A=$40` 落点，
    /// `$83DC` 入口才装 `A=$30`；双入口两态）。
    pub bypass_entry: Option<u16>,
    /// →`$518` 表（None=D/E/F 段走波形驱动无此写）。
    pub t518: &'static [u8],
    /// →`$684` 表。
    pub t684: &'static [u8],
    /// →`$534` 表（None=B 段走 `$2B&1` 相位翻转支）。
    pub t534: &'static [u8],
    /// 尾跳（`$CC6A` 公共运动尾 或 `$84D1` 波形驱动入口）。
    pub tail: u16,
}

/// 参数化段全集（A-F）。B 段表=`PARAM_TABLE_518/684`（$8406/$840B，批次4f）。
pub const PARAM_SEGMENTS: [ParamSegment; 6] = [
    ParamSegment { entry: 0x838E, cd9d_a: Some(0x40), cd9d_y: Some(4), bypass_entry: None, t518: &SEG_A_T518, t684: &SEG_A_T684, t534: &SEG_A_T534, tail: 0xCC6A },
    ParamSegment { entry: 0x83DC, cd9d_a: Some(0x30), cd9d_y: Some(5), bypass_entry: Some(0x83DE), t518: &PARAM_TABLE_518, t684: &PARAM_TABLE_684, t534: &[], tail: 0xCC6A },
    ParamSegment { entry: 0x8432, cd9d_a: Some(0x40), cd9d_y: Some(7), bypass_entry: None, t518: &SEG_C_T518, t684: &SEG_C_T684, t534: &[], tail: 0xCC6A },
    ParamSegment { entry: 0x8486, cd9d_a: Some(0x50), cd9d_y: Some(0x0E), bypass_entry: None, t518: &[], t684: &[], t534: &[], tail: 0x84D1 },
    ParamSegment { entry: 0x84C3, cd9d_a: Some(0x50), cd9d_y: Some(0x0F), bypass_entry: None, t518: &[], t684: &[], t534: &[], tail: 0x84D1 },
    ParamSegment { entry: 0x862A, cd9d_a: Some(0x60), cd9d_y: Some(0x0F), bypass_entry: None, t518: &[], t684: &[], t534: &[], tail: 0x84D1 },
];

/// `$CD9D(A,Y)` 运动模式选择器三态出口（**内核 S4b-9b 未证——仅签名登记，无实现**）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cd9dOutcome {
    /// BCC=继续（参数化段落表写字段）。
    Continue,
    /// BEQ=`$D10C` 终止（回收即终止）。
    Terminate,
    /// BNE=另一支（语义未名）。
    OtherBranch,
}

// ===== 批次4g-4：$84D1 多相波形驱动六表（behavior-vm-handlers §4，A 级字节）=====
//
// 轨迹相位机核心：$5DC+X 计数（**与 Wt-6 三角波共享**——波形驱动与三角波表同相位）
// × $072C+X 相位索引 × $0710+X 循环上限。六表数值语义=S4b-2 配对面（待证不发明）。

/// `$854D` 参数对（→`$518`，24B）。
pub const PHASE_T518: [u8; 24] = [
    0x0A, 0x0B, 0x0C, 0x0C, 0x0C, 0x00, 0x00, 0x00, 0x0A, 0x0B, 0x0C, 0x0D, 0x0C, 0x0C, 0x0C,
    0x00, 0x0B, 0x0C, 0x0D, 0x0E, 0x0D, 0x0C, 0x0C, 0x0C,
];
/// `$8565` 参数对（→`$684`，24B）。
pub const PHASE_T684: [u8; 24] = [
    0x5F, 0x61, 0x63, 0x65, 0x67, 0x00, 0x00, 0x00, 0x5F, 0x61, 0x63, 0x69, 0x63, 0x65, 0x67,
    0x00, 0x61, 0x63, 0x69, 0x6D, 0x69, 0x63, 0x65, 0x67,
];
/// `$857D` 相位阈表（13B）。
pub const PHASE_THRESHOLD: [u8; 13] = [0x00, 0x00, 0x02, 0x04, 0x06, 0x00, 0xFF, 0x01, 0x02, 0x03, 0x05, 0x06, 0x07];
/// `$858A` 子流指针对 lo（13B）。
pub const PHASE_PTR_LO: [u8; 13] = [0x00, 0xF6, 0x08, 0x06, 0xFC, 0x00, 0xF0, 0x15, 0xF9, 0x13, 0xEC, 0x10, 0xFA];
/// `$8597` 子流指针对 hi（13B）。
pub const PHASE_PTR_HI: [u8; 13] = [0x00, 0xF6, 0x0C, 0xF3, 0x07, 0x00, 0xF2, 0x07, 0x11, 0xEC, 0x05, 0xF8, 0xFC];
/// `$85A4` 符号标志表（13B——bit7=相位翻转开关）。
pub const PHASE_FLAG: [u8; 13] = [0x90, 0x88, 0x00, 0x08, 0x80, 0x10, 0x88, 0x80, 0x90, 0x00, 0x88, 0x80, 0x08];

/// 相位阈判（`$5DC+X − $857D,Y < 8` 才落表步）：计数过阈不足 8 ⇒ 命中。
pub fn phase_threshold_hit(counter_5dc: u8, threshold: u8) -> bool {
    counter_5dc.wrapping_sub(threshold) < 8
}

/// `$05` bit7 相位翻转写 `$534+X`（`($2B&1) ^ $12` 快照——与 Wt-6 运动机
/// `$AC67` EOR 同型语义面）：返回新 `$534` 值。
pub fn phase_flip_534(snapshot_534: u8, zp_2b: u8) -> u8 {
    (zp_2b & 1) ^ snapshot_534
}

// ===== 敌侧框形取框（bank14:$D511-$D5AE，A 级——arena-midboss §2.1 消费链）=====

/// 框形指针 lo 表（bank2 `$97BA`，88B=形索引 0..87）。
pub const SHAPE_PTR_LO: [u8; 88] = [
0x6A, 0x6F, 0x7A, 0x85, 0x9D, 0xB2, 0xD2, 0xEA, 0xF7, 0xFC, 0x25, 0x31, 0x36, 0x53, 0x7C, 0x16,
    0x48, 0x6F, 0x81, 0x8C, 0xB1, 0xB6, 0xBB, 0xD6, 0xD6, 0xD6, 0x77, 0x15, 0x21, 0x21, 0x77, 0x76,
    0x25, 0xA8, 0xAD, 0x3C, 0x31, 0x36, 0x31, 0x36, 0x31, 0x36, 0x73, 0x7C, 0xB2, 0xB6, 0xB6, 0xB6,
    0xD2, 0xDB, 0xDB, 0x0B, 0x36, 0x26, 0x39, 0x5E, 0x77, 0x77, 0x8C, 0xA8, 0xBC, 0xCA, 0x36, 0x36,
    0x1E, 0x09, 0x23, 0x28, 0x28, 0x6C, 0x6C, 0x8D, 0xA8, 0xB7, 0xC3, 0xDE, 0xE9, 0xFD, 0x18, 0x30,
    0xDC, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48,
];

/// 框形指针 hi 表（bank2 `$9812`，88B）。
pub const SHAPE_PTR_HI: [u8; 88] = [
0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99,
    0x9C, 0x9C, 0x9C, 0x9C, 0x9C, 0x9C, 0x9C, 0x9C, 0x9C, 0x9C, 0x9A, 0x9D, 0x9D, 0x9D, 0x9A, 0x9D,
    0x99, 0x99, 0x99, 0x9D, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x9D, 0x9D, 0x9D,
    0x99, 0x99, 0x99, 0x9A, 0x99, 0x9A, 0x9A, 0x9A, 0x9A, 0x9A, 0x9A, 0x9A, 0x9A, 0x9A, 0x99, 0x99,
    0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9B, 0x9C, 0x9C,
    0x9D, 0x9E, 0x9E, 0x9E, 0x9E, 0x9E, 0x9E, 0x9E,
];

/// 框形流地址（`LDA $97BA,Y / LDA $9812,Y`——形索引 Y≤87）。
pub fn shape_stream_addr(shape: u8) -> Option<u16> {
    let i = shape as usize;
    if i >= SHAPE_PTR_LO.len() {
        return None;
    }
    Some(u16::from(SHAPE_PTR_LO[i]) | (u16::from(SHAPE_PTR_HI[i]) << 8))
}

/// 敌侧框形取框（`$D511-$D5AE` 逐指令，A 级）：框形流 `[子框数][4B/框
/// (左,右,上,下) 有符号偏移]`；翻转（`$0534` bit6=x / bit7=y）：左'=-右/右'=-左、
/// 上'=-下/下'=-上；终值=对象 `$04E0/$04FC` + 偏移（负向钳 0/正溢出钳 FF）。
/// 返回 `(左,右,上,下)` 框集（子框数=流首字节；fail-closed：流不足 ⇒ 已解析部分）。
pub fn fetch_shape_boxes(
    stream: &[u8],
    x: u8,
    y: u8,
    flip_x: bool,
    flip_y: bool,
) -> Vec<(u8, u8, u8, u8)> {
    if stream.is_empty() {
        return Vec::new();
    }
    let count = stream[0] as usize;
    let mut out = Vec::new();
    let mut i = 1usize;
    for _ in 0..count {
        if i + 4 > stream.len() {
            break;
        }
        let (mut l, mut r, mut t, mut b) = (
            stream[i] as i8,
            stream[i + 1] as i8,
            stream[i + 2] as i8,
            stream[i + 3] as i8,
        );
        i += 4;
        if flip_x {
            core::mem::swap(&mut l, &mut r);
            l = -l;
            r = -r;
        }
        if flip_y {
            core::mem::swap(&mut t, &mut b);
            t = -t;
            b = -b;
        }
        let clamp_add = |base: u8, off: i8| -> u8 {
            if off >= 0 {
                base.saturating_add(off as u8)
            } else {
                base.saturating_sub(off.unsigned_abs())
            }
        };
        out.push((clamp_add(x, l), clamp_add(x, r), clamp_add(y, t), clamp_add(y, b)));
    }
    out
}

/// 表分发项（type → handler 目标 + type≥$30 前置 MMC3 切 bank 值）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeDispatch {
    /// handler 目标（`$80CE+type:$7FFF+type`；type≥$30 时在切换后 bank 窗内）。
    pub target: u16,
    /// 前置切 bank（`$816E+type` 值，MMC3 reg7；None=type<$30 不切）。
    pub bank_switch: Option<u8>,
}

/// 表分发全表（255 项，data/reference/type-dispatch-table.json 嵌码——decomp
/// f06a376 接线律：type<$30 bank2 通用族不切 bank；type≥$30 前置切 $A000 窗
/// bank=$816E+type。验证 4/4：$2B→$90AB→$9120 与 trace f+1 ($91,$20) 吻合）。
pub const TYPE_DISPATCH: [TypeDispatch; 256] = [
    TypeDispatch { target: 0x0000, bank_switch: Some(0x00) },
    TypeDispatch { target: 0xA000, bank_switch: None },
    TypeDispatch { target: 0xAB6A, bank_switch: None },
    TypeDispatch { target: 0xAB97, bank_switch: None },
    TypeDispatch { target: 0xAC64, bank_switch: None },
    TypeDispatch { target: 0xAE05, bank_switch: None },
    TypeDispatch { target: 0xB093, bank_switch: None },
    TypeDispatch { target: 0xA601, bank_switch: None },
    TypeDispatch { target: 0xA465, bank_switch: None },
    TypeDispatch { target: 0xA4E3, bank_switch: None },
    TypeDispatch { target: 0x8334, bank_switch: None },
    TypeDispatch { target: 0x8334, bank_switch: None },
    TypeDispatch { target: 0x8334, bank_switch: None },
    TypeDispatch { target: 0x8314, bank_switch: None },
    TypeDispatch { target: 0xD10C, bank_switch: None },
    TypeDispatch { target: 0xA33E, bank_switch: None },
    TypeDispatch { target: 0x8337, bank_switch: None },
    TypeDispatch { target: 0x8381, bank_switch: None },
    TypeDispatch { target: 0x83BA, bank_switch: None },
    TypeDispatch { target: 0x8410, bank_switch: None },
    TypeDispatch { target: 0x845A, bank_switch: None },
    TypeDispatch { target: 0x8497, bank_switch: None },
    TypeDispatch { target: 0x85B1, bank_switch: None },
    TypeDispatch { target: 0x85EA, bank_switch: None },
    TypeDispatch { target: 0x8CCE, bank_switch: None },
    TypeDispatch { target: 0x8CF7, bank_switch: None },
    TypeDispatch { target: 0x8DB8, bank_switch: None },
    TypeDispatch { target: 0x8E50, bank_switch: None },
    TypeDispatch { target: 0x8E53, bank_switch: None },
    TypeDispatch { target: 0x8E90, bank_switch: None },
    TypeDispatch { target: 0x8F1E, bank_switch: None },
    TypeDispatch { target: 0x8F66, bank_switch: None },
    TypeDispatch { target: 0x8A8C, bank_switch: None },
    TypeDispatch { target: 0x8AE0, bank_switch: None },
    TypeDispatch { target: 0x8B08, bank_switch: None },
    TypeDispatch { target: 0x8B63, bank_switch: None },
    TypeDispatch { target: 0x8BD0, bank_switch: None },
    TypeDispatch { target: 0x8C09, bank_switch: None },
    TypeDispatch { target: 0x8C88, bank_switch: None },
    TypeDispatch { target: 0x8C88, bank_switch: None },
    TypeDispatch { target: 0x8334, bank_switch: None },
    TypeDispatch { target: 0x8334, bank_switch: None },
    TypeDispatch { target: 0x8FCE, bank_switch: None },
    TypeDispatch { target: 0x90AB, bank_switch: None },
    TypeDispatch { target: 0x8953, bank_switch: None },
    TypeDispatch { target: 0x8957, bank_switch: None },
    TypeDispatch { target: 0x898A, bank_switch: None },
    TypeDispatch { target: 0x899B, bank_switch: None },
    TypeDispatch { target: 0xB183, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xB22C, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xB2E3, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xB35C, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xB407, bank_switch: Some(0x03) },
    TypeDispatch { target: 0x8334, bank_switch: Some(0x03) },
    TypeDispatch { target: 0x8334, bank_switch: Some(0x03) },
    TypeDispatch { target: 0x8334, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xB4A3, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xA209, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA27D, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA4C0, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA562, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA04D, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA235, bank_switch: Some(0x06) },
    TypeDispatch { target: 0x8334, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA611, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA6DE, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA735, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA7A8, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA7F4, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA886, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA918, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA9AF, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xAA72, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xAAEA, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xABDB, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xAD2C, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xAD87, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xADF1, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xAE44, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xAE83, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA000, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB00A, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB0D4, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB15B, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB1AA, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB21B, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB289, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB323, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB3A8, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB436, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA0E2, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB480, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB4E5, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xAF2F, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB588, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB9DB, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xB648, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA129, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA000, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xB06B, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA13A, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xA2F4, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xB526, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xB73C, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xB6EE, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB733, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB81E, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB8EF, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB9E4, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xBA33, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xBB1E, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xBB5C, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xBBEF, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xBC97, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA2FC, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA2FC, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xBD44, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xA3F1, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xBDDF, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xBE41, bank_switch: Some(0x04) },
    TypeDispatch { target: 0xB7F9, bank_switch: Some(0x03) },
    TypeDispatch { target: 0xAB17, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xABC6, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xAC77, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xACBA, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xACFC, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xAD4A, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xA000, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xADD4, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xAE42, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xAE97, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xAEC2, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xA106, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xA1CE, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xA3AC, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xA4E3, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xAF9B, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB0CB, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB128, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xA915, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB1EC, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB2B2, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB311, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB3AA, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB40D, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB4D6, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB55F, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB604, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB647, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB6AD, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB7E9, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB833, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB8B1, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xB92F, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xBA47, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xBA9B, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xBB42, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xBBA2, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xBF1B, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xA9E0, bank_switch: Some(0x05) },
    TypeDispatch { target: 0xA3CC, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xA582, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xA61D, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xA000, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA2C0, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA355, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA37E, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA3EE, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA49B, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA4E0, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA556, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA5AC, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA66B, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA6F5, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA80A, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA874, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xAD5D, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xADFA, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xAEAB, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xAF59, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB02A, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB104, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB20E, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB2B1, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB423, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB479, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB62B, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB68B, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB6FB, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB78B, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB7FE, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB8E9, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xB9BC, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBA44, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBAF9, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBB81, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBBE9, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBC5A, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBCDD, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBD36, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBDA6, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xBDE8, bank_switch: Some(0x06) },
    TypeDispatch { target: 0xA6EC, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xACE5, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xAE3A, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xAEB0, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xAF21, bank_switch: Some(0x07) },
    TypeDispatch { target: 0xB0C7, bank_switch: Some(0x07) },
    TypeDispatch { target: 0x03A0, bank_switch: Some(0x20) },
    TypeDispatch { target: 0x03AB, bank_switch: Some(0xF9) },
    TypeDispatch { target: 0x03AB, bank_switch: Some(0xCF) },
    TypeDispatch { target: 0x03AC, bank_switch: Some(0x20) },
    TypeDispatch { target: 0x03AE, bank_switch: Some(0x27) },
    TypeDispatch { target: 0x03B0, bank_switch: Some(0xDB) },
    TypeDispatch { target: 0x03A6, bank_switch: Some(0xA9) },
    TypeDispatch { target: 0x03A4, bank_switch: Some(0x03) },
    TypeDispatch { target: 0x03A4, bank_switch: Some(0x20) },
    TypeDispatch { target: 0x0483, bank_switch: Some(0x2E) },
    TypeDispatch { target: 0x0483, bank_switch: Some(0xE6) },
    TypeDispatch { target: 0x0483, bank_switch: Some(0xA5) },
    TypeDispatch { target: 0x0483, bank_switch: Some(0x32) },
    TypeDispatch { target: 0x06D1, bank_switch: Some(0x29) },
    TypeDispatch { target: 0x06A3, bank_switch: Some(0xEF) },
    TypeDispatch { target: 0x0683, bank_switch: Some(0x85) },
    TypeDispatch { target: 0x0483, bank_switch: Some(0x32) },
    TypeDispatch { target: 0x0483, bank_switch: Some(0x29) },
    TypeDispatch { target: 0x0484, bank_switch: Some(0x01) },
    TypeDispatch { target: 0x0484, bank_switch: Some(0xF0) },
    TypeDispatch { target: 0x0484, bank_switch: Some(0x0E) },
    TypeDispatch { target: 0x0485, bank_switch: Some(0xA5) },
    TypeDispatch { target: 0x0485, bank_switch: Some(0x32) },
    TypeDispatch { target: 0x048C, bank_switch: Some(0x29) },
    TypeDispatch { target: 0x048C, bank_switch: Some(0xFE) },
    TypeDispatch { target: 0x048D, bank_switch: Some(0x85) },
    TypeDispatch { target: 0x048E, bank_switch: Some(0x32) },
    TypeDispatch { target: 0x048E, bank_switch: Some(0xAD) },
    TypeDispatch { target: 0x048E, bank_switch: Some(0xA5) },
    TypeDispatch { target: 0x048F, bank_switch: Some(0x01) },
    TypeDispatch { target: 0x048F, bank_switch: Some(0xA0) },
    TypeDispatch { target: 0x048A, bank_switch: Some(0x02) },
    TypeDispatch { target: 0x048A, bank_switch: Some(0x20) },
    TypeDispatch { target: 0x048B, bank_switch: Some(0x3D) },
    TypeDispatch { target: 0x048B, bank_switch: Some(0xE6) },
    TypeDispatch { target: 0x048B, bank_switch: Some(0xE6) },
    TypeDispatch { target: 0x048C, bank_switch: Some(0x2B) },
    TypeDispatch { target: 0x048C, bank_switch: Some(0x20) },
    TypeDispatch { target: 0x048C, bank_switch: Some(0x3B) },
    TypeDispatch { target: 0x0483, bank_switch: Some(0xD9) },
    TypeDispatch { target: 0x0483, bank_switch: Some(0xA2) },
    TypeDispatch { target: 0x048F, bank_switch: Some(0x00) },
    TypeDispatch { target: 0x0490, bank_switch: Some(0x86) },
    TypeDispatch { target: 0x0489, bank_switch: Some(0x74) },
    TypeDispatch { target: 0x0489, bank_switch: Some(0xA5) },
    TypeDispatch { target: 0x0489, bank_switch: Some(0xDB) },
    TypeDispatch { target: 0x0489, bank_switch: Some(0xF0) },
    TypeDispatch { target: 0x03B1, bank_switch: Some(0x11) },
];

// ===== S4b-2 ③ 行为链断言面（f8c02b2 六表↔轨迹配对）=====

/// `$914B` 表（4B——$2B 首段 `$518` 形状索引族 {$25,$29}；【099c510 dated】
/// 5-6f 步进=**图形动画节奏**非运动节奏）。
pub const PHASE_T914B: [u8; 4] = [0x25, 0x25, 0x29, 0x25];

/// `$914F` 表（8B——`$684` tile 基址族 {$F7,$FB,$FF}；循环 F7→FB→FF→FB→F7）。
/// 【099c510 dated】~~x 轨迹 104f=4×26f 亚像素合成比~~ 作废——步进=图形动画
/// 节奏；x 轨迹真身=($56C:$550) 速度对积分（非本表驱动）。
pub const PHASE_T914F: [u8; 8] = [0xF7, 0xFB, 0xFF, 0xFB, 0x27, 0x27, 0x27, 0x25];

/// `$832E` 表（4B——`$534` OAM attr 族；运行面 `|2`=attr 位合成：`$00|2=$02`、
/// `$40|2=$42`——【099c510 dated】|2=**attr 位**非速度语义）。
pub const PHASE_T832E: [u8; 4] = [0x00, 0x00, 0x00, 0x40];

/// `$832E` 表运行值（`|2` 合成——{$02,$42} 族）。
pub fn phase_t832e_value(raw: u8) -> u8 {
    raw | 0x02
}

/// 型值↔驱动指派总表（S4b-2 §4.2 8 行——**内态族**（$84D1 相位机）与出生型值分野）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriverAssignment {
    /// 型值（低区）或型值域起点（高区）。
    pub type_code: u8,
    /// 驱动族描述（不发明语义名——指派标签）。
    pub driver: &'static str,
    /// 内态族（$84D1 相位机状态）而非出生型值。
    pub inner_state: bool,
}

/// 指派 8 行（f8c02b2 定谳）。
pub const DRIVER_ASSIGNMENTS: [DriverAssignment; 8] = [
    DriverAssignment { type_code: 0x2B, driver: "$9120 三角族", inner_state: false },
    DriverAssignment { type_code: 0x12, driver: "$83DC 段族 B", inner_state: false },
    DriverAssignment { type_code: 0x10, driver: "jitter/跳板", inner_state: false },
    DriverAssignment { type_code: 0x11, driver: "jitter/跳板", inner_state: false },
    DriverAssignment { type_code: 0x14, driver: "$84D1 相位机", inner_state: true },
    DriverAssignment { type_code: 0x15, driver: "$84D1 相位机", inner_state: true },
    DriverAssignment { type_code: 0x17, driver: "$84D1 相位机", inner_state: true },
    DriverAssignment { type_code: 0x4E, driver: "bank4 直线", inner_state: false },
];
