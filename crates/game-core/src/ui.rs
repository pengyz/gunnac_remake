//! game-core::ui —— 金钱系统 + 商店状态机 + 价目记录（M3 UI/商店/金钱，t77）。
//!
//! 语义来源：`docs/spec/shop-economy.md` R-money-1..6 / R-shop-1..8 + `docs/spec/ui-flow.md`
//! R-ui-*；常量=C-money-bcd/C-money-addend-tbl/C-money-cap/C-extend-threshold（constants v3）。
//! **未证不发明**：卖出机制/金钱消费侧（TODO）、价目字段切分（H-05）、A 键切页语义（TODO）。
//!
//! UI 输入语义注记（t52 后口径）：**A/B 键分工随场景**——实机 B=主武器连发（B 门控），
//! 商店 B=切页/双 B 退出；同一物理键两套语义，平台侧输入策略不得跨场景混用。
//!
//! DEV-1 增补（A1 商店购买链接线，R-shop-purchase-*）：`shop_purchase_step` 状态流
//! （见本文件 §A1 购买链接线）——证据 `docs/findings/shop-purchase-flow.md` §1-§4 +
//! `docs/spec/shop-purchase.md`；与 R-shop-5..8 的信息页状态机（`ShopState`）**分立**
//! （两流的门控关系 W-NEW1 未证，禁止混接）。
//!
//! quirk 保真：加法器进位 = 逐位 `CMP #$0A/SBC #$0A` 的 borrow 携带（下一 ADC 加 C）；
//! 溢出封顶 $E0-$E5 全 9（$E6 个位不参与加法——全部加数为 10 的倍数）。

/// C-money-addend-tbl 加数表原始 66 字节（`bank14:$DCCF` 起 11×6，type 0 未用）。
/// 每事件 type T 的 6 字节 = 表尾对齐（byte[6T-k]→digit $E0+(5-k)，k=0..5），
/// 即 $E5=十位 … $E0=百万位（与 ROM `DEY/DEX` 回走序一致，L2 对拍锚定）。
pub const MONEY_ADDEND_TBL: [u8; 66] = [
    0x03, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0,
    0, 0x03, 0, 0, 0, 0, 0x01, 0, 0, 0, 0,
    0, 0x01, 0x05, 0, 0, 0, 0, 0x02, 0x05, 0, 0,
    0, 0, 0x04, 0, 0, 0, 0, 0, 0x06, 0, 0,
    0, 0, 0, 0x08, 0, 0, 0, 0, 0x01, 0, 0,
    0, 0, 0, 0x02, 0, 0, 0, 0, 0, 0x05, 0x08,
];

/// C-money-bcd：7 位 BCD（$E0=最高位 … $E6=个位）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoneyCounter {
    pub digits: [u8; 7],
}

impl MoneyCounter {
    /// 显示数值（十进制值；测试/L3 比对口）。
    pub fn value(&self) -> u64 {
        self.digits.iter().fold(0u64, |acc, &d| acc * 10 + d as u64)
    }
}

/// R-money-5 入账加法器（bank14:DBCE 逐指令语义）：type×6 索引加数表，
/// 六位逐位 BCD 进位加法（进位=single borrow 链），末位进位 ⇒ $E0-$E5 全 9 封顶。
/// type=0：ROM `BEQ` 直接返回（无操作）。
pub fn money_add_event(money: &mut MoneyCounter, event_type: u8) -> bool {
    if event_type == 0 {
        return false; // $DBD4 BEQ：type 0 无操作
    }
    let base = event_type as usize * 6;
    let mut carry = false;
    for k in 0..6usize {
        let digit_idx = 5 - k; // $E0+X，X=5..0
        let addend = MONEY_ADDEND_TBL[base - k]; // Y=base-k 回走
        let mut sum = money.digits[digit_idx] + addend + u8::from(carry);
        carry = false;
        if sum >= 0x0A {
            sum -= 0x0A;
            carry = true;
        }
        money.digits[digit_idx] = sum;
    }
    if carry {
        money.digits[..6].fill(9); // 封顶 $E0-$E5
        return true;
    }
    false
}

/// R-money-6 extend 阈值检查（bank14:DAE7）：与 `extend_threshold_bcd`（=8000）逐位比较，
/// 达标置 `$ED=$40`（返回 true=置位）。阈值=C-extend-threshold（$E7-$EC = 0,0,8,0,0,0）。
pub fn extend_check(money: &MoneyCounter) -> bool {
    // 值语义（probe：7200 逼近 8000）；窗口对齐边角（$E0 位≠0 时逐位窗与值比较的分歧）
    // 未证——取值比较并注记。
    money.value() >= 8000
}

/// R-money-4 渲染前导零抑制：逐位输出（左起），全零前缀跳位；至少显示末位 0。
/// 返回每位 (数字, 是否显示)。语义=bank14:D009-D046（`$05/BNE` 跳位、`CPY #7` 七位）。
pub fn render_digits(money: &MoneyCounter) -> [(u8, bool); 7] {
    let mut out = [(0u8, false); 7];
    let mut seen_nonzero = false;
    for i in 0..7 {
        let d = money.digits[i];
        seen_nonzero |= d != 0;
        let show = seen_nonzero || i == 6; // 至少显示个位
        out[i] = (d, show);
    }
    out
}

/// 商店面板状态机（R-shop-5..8）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopState {
    /// 主面板（静态屏——无输入不变，R-shop-5）。
    Main,
    /// 信息页（B 单击切入，R-shop-6）。
    Info,
    /// 已退出（双 B——近黑过渡后进实机，R-shop-7）。
    Exited,
}

impl Default for ShopState {
    fn default() -> Self {
        ShopState::Main
    }
}

impl ShopState {
    /// B 键单击：Main→Info；Info（双 B）→Exited（R-shop-6/7）。
    /// 注：返回 Main 的切页语义与 A 键分工未证（H-05 域）——不发明。
    pub fn on_b(self) -> ShopState {
        match self {
            ShopState::Main => ShopState::Info,
            ShopState::Info => ShopState::Exited,
            ShopState::Exited => ShopState::Exited,
        }
    }

    /// Start 键：**无效果**（R-shop-8，exp-shop-start 反证）。
    pub fn on_start(self) -> ShopState {
        self
    }
}

/// R-shop-1 价目记录解析（rom:8914-9000 记录区）：4 槽位 `N.` 标记 + `$` 价格串；
/// 槽 4 无价格串（0xFF 填充截断）⇒ None。**字段切分未证（H-05）**——仅提取已证字段。
pub fn parse_price_records(rom: &[u8]) -> [Option<u16>; 4] {
    let mut out = [None; 4];
    for slot in 0..4u8 {
        let marker = [b'1' + slot, b'.'];
        let Some(mpos) = rom.windows(2).position(|w| w == marker) else {
            continue;
        };
        // 标记后找 ASCII '$' + 数字串
        let mut i = mpos + 2;
        while i < rom.len() && rom[i] != b'$' {
            if rom[i] == 0xFF {
                break; // FF 填充截断（槽 4）
            }
            i += 1;
        }
        if i >= rom.len() || rom[i] != b'$' {
            continue;
        }
        let mut v: u16 = 0;
        let mut j = i + 1;
        let mut any = false;
        while j < rom.len() && rom[j].is_ascii_digit() {
            v = v * 10 + (rom[j] - b'0') as u16;
            any = true;
            j += 1;
        }
        if any {
            out[slot as usize] = Some(v);
        }
    }
    out
}

/// 金钱消费侧（卖出/购买扣款）：**TODO 未证不发明**（R-money-2 消费渠道仅 B 级）。
/// 注：商店购买扣款（价 10/6/4）已证并入 §A1 购买链接线（`shop_purchase_step`），
/// 此处仅保留未证的「卖出/关内消费」语义占位。
pub fn money_spend(_money: &mut MoneyCounter, _price: u16) -> Option<()> {
    None
}

// ===== §A1 商店购买链接线（R-shop-purchase-*，DEV-1）=====
//
// 证据：`docs/findings/shop-purchase-flow.md` §1 触发链 / §2 扣减 / §3 校验 / §4 生效写入（A 级）
// + `docs/spec/shop-purchase.md` R-shop-purchase-1..10。RAM 字段=命名台账 A 级名。
// 只接已证规格：消息**文案内容**未证 ⇒ `ShopPrompt` 状态枚举（不发明字符串）；
// 声音（$ED3B 音效 $43/$44）与文本重绘（$A06C/$A091/$A09F/E6D9）属渲染线，不在此落。
// 扣减/封顶/递增一律调用 `l2ports` 同名移植函数（差分已裁，勿重写逻辑）。

/// `$16` 手柄位（bank00:$9D02-$9D3E 主环 / bank00:$9E40 sub_9E40 子面板 / bank01:$A0CF ack）。
pub const BTN_A: u8 = 0x80;
pub const BTN_B: u8 = 0x40;
pub const BTN_START: u8 = 0x10;
pub const BTN_UP: u8 = 0x08;
/// Down 组 = bit2|bit5（bank00:$9D13 `AND #$24` ⇒ `INC $0400`）。
pub const BTN_DOWN: u8 = 0x24;
/// 选中 = A|B|Start（bank00:$9D3C `AND #$D0` ⇒ RTS 跳转表分发）。
pub const BTN_SELECT: u8 = 0xD0;
/// 消息 ack = A|B（bank01:$A0CF `AND #$C0`）。
pub const BTN_ACK: u8 = 0xC0;
/// 子面板左移 = bit1（sub_9E40 bank00:$9E42 `AND #$02`）。
pub const BTN_LEFT: u8 = 0x02;
/// 子面板右移 = bit0|bit5（sub_9E40 bank00:$9E5D `AND #$21`）。
pub const BTN_RIGHT: u8 = 0x21;

/// 购买链提示消息（**状态枚举**——消息文案内容未证，禁止发明字符串）。
/// 变体名 = findings §3 已证语义名 / 文本指针见 `docs/spec/shop-purchase.md`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopPrompt {
    /// 余额不足（三价共享出口 bank01:$A05B，文本指针 $A13D）。
    InsufficientFunds,
    /// 武器满级 `$33`≥7（bank00:$9DCC，文本指针 $A408）。
    WeaponMax,
    /// 连射满级 `$3C`≥4（bank00:$9E94，文本指针 $A4D4）。
    RapidMax,
    /// 翼已购（`$32` bit7=1，bank00:$9D98，文本指针 $A436）。
    WingAlreadyOwned,
    /// 未满足翼前置（同型 `$33`≥2 且无翼，bank00:$9DB5，文本指针 $A3B0）。
    WingRequired,
    /// 发货阶段门未过（`$0180`∉[2,7]，bank00:$9F03-$9F16，文本指针 $A62A）。
    ShipStageBlocked,
    /// 退出隐藏门触发消息（AR65-2，review-65 §1.3）：bank01:$A03D `text $A65C` + `JSR $A09F`
    /// （ack 门 bank01:$A0CF `AND #$C0`，同 R-shop-purchase-8 六类消息同一路等待语义）；
    /// **文本 `$A65C` 内容未证 ⇒ 登记不解释**，只落状态枚举；ack 后才合流 bank01:$A048 转场。
    ExitHidden,
}

/// 消息 ack 后的返回面板（bank00:$9D5E 重入武器面板 / bank00:$9CEC 回主环 / bank01:$A048 退出转场）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopMsgReturn {
    /// 回主面板环（`$0400` 复位 0；序列入口 bank00:$9CEC 文本→渲染→复位，
    /// `LDA #0`/`STA $400` 实际在 bank00:$9CF7/$9CF9——AD65-1 指针更正）。
    #[default]
    MainRing,
    /// 重入武器面板（`$0401` 复位 := `$34`，bank00:$9D6C）。
    WeaponMenu,
    /// 退出转场（AR65-2：ack 后合流 bank01:$A048 `text $A31F/$E6D9/LDA #$03/JSR $EE03/JMP $EEFB`
    /// ⇒ phase=Exited 交 flow，本线不发明转场语义）。
    Exit,
}

/// 购买链相位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopPurchasePhase {
    /// 主面板环：`$0400` 0..3 环绕 + `$16&$D0` 分发（bank00:$9CFC-$9D55）。
    #[default]
    MainRing,
    /// 武器子面板：`$0401` 环绕 [0..5]（5=翼），init := `$34`（bank00:$9D5E-$9E1B）。
    WeaponMenu,
    /// 连射确认环（bank00:$9EA8-$9EF0）：A 购买 / B 返回。
    RapidMenu,
    /// 发货槽选择环：`$0401` 环绕 [`stage`..7]（bank00:$9F1C-$9F5F）。
    ShipSlotMenu,
    /// 发货弹种选择环：`$0401` 环绕 [0..3]（bank00:$9F75-$9FA4）。
    ShipTypeMenu,
    /// 消息展示中（内容未证）；A/B ack 后按 `msg_back` 返回。
    Message,
    /// 退出（bank01:$A023 → 开 stage 转场，交 `flow`）。
    Exited,
}

/// 一步的主效果（RAM 终态真相在 `ShopPurchaseState` 字段；此处只补状态不可见的效果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopFx {
    /// 导航/无操作。
    None,
    /// 武器购买生效：far-call `bank02:$922B` = 关内拾取同例程（`weapons::pickup_handler` 已接）。
    WeaponBought,
    /// 翼购买：`$32` bit7 已写（bank00:$9E0E-$9E12）；far-call `bank02:$92C5`（翼收尾例程）
    /// **未移植** ⇒ 显式挂起，不发明其写集（R-shop-purchase-9 缺口）。
    WingBoughtFarCallPending,
    /// 连射购买生效（`INC $3C`，bank00:$9EDF）。
    RapidBought,
    /// 发货购买生效（`$0178+Y` 打包写，bank00:$9FBD-$9FDA）。
    ShipBought,
}

/// 购买链 RAM 镜像（字段括注 ROM 地址；命名=台账 A 级名）。
///
/// `delivery` 为 `$0178..$017F` 8 字节镜像：购买写点 `$0178+Y`，Y=`$0403`∈[`stage`..7]
/// （bank00:$9F3E `LDA #7` 上界）；N-0201 语义槽仅证 `$0178-$017E`，
/// 兑现/购买两链索引基差=1 未对账（findings W-NEW5）——索引 ≥8 时 fail-closed 不动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShopPurchaseState {
    /// 相位。
    pub phase: ShopPurchasePhase,
    /// 提示消息（内容未证）。
    pub prompt: Option<ShopPrompt>,
    /// 消息返回目标。
    pub msg_back: ShopMsgReturn,
    /// `$0400` shop_slot_cursor（N-0002）：主面板 0..3。
    pub cursor: u8,
    /// `$0401` ui_select_cursor（N-0003）：武器 [0..5] / 槽 [stage..7] / 弹种 [0..3]。
    pub item_cursor: u8,
    /// `$0402` 槽光标下界（进发货菜单同写，bank00:$9F22；= `$0180`）。
    pub ship_lo: u8,
    /// `$0403` 发货槽索引（N-0201）。
    pub ship_index: u8,
    /// `$018E` 商店金钱（0..255）。
    pub money: u8,
    /// `$0180` 关卡号（发货阶段门 2..7）。
    pub stage: u8,
    /// `$0178..$017F` 发货槽（打包 `(type<<6)|qty`，findings §4）。
    pub delivery: [u8; 8],
    /// `$33`/`$34`/`$018F`（`weapons::WeaponState`）。
    pub weapon: crate::weapons::WeaponState,
    /// `$018C`/`$018A`/`$32`（`weapons::PickupContext`；`flags`= `$32`，bit7=翼已购〔本线置位点
    /// bank00:$9E0E `ORA #$80`〕）。AD65-3 互注：bit7 另一真读者=kernel 门
    /// （`weapons.rs` bank02:$922B dispatch）——同一 RAM 位两处真读者，改语义须双向对账；
    /// bit5=退出隐藏门写（bank01:$A039，登记不解释）。
    pub pickup: crate::weapons::PickupContext,
    /// `$3C` 连射级（0..4）。
    pub rapid_level: u8,
    /// `$18` 手柄2 状态（退出隐藏选项门触发源(a) `($18&$33)==$11`，bank01:$A023-$A029；
    /// 触发源(b) `$0180==8` 见 step_main_ring 退出入口注）。
    pub pad2: u8,
}

fn to_main_ring(st: &mut ShopPurchaseState) -> ShopFx {
    // bank00:$9CE9 `JSR $A06C`（清行，渲染线）→ $9CEC 序列入口（文本→渲染→复位，
    // `LDA #0`/`STA $400` 在 bank00:$9CF7/$9CF9——AD65-1）→ 主环。
    st.phase = ShopPurchasePhase::MainRing;
    st.cursor = 0;
    st.prompt = None;
    ShopFx::None
}

fn prompt_to_main(st: &mut ShopPurchaseState, p: ShopPrompt) {
    st.prompt = Some(p);
    st.msg_back = ShopMsgReturn::MainRing;
    st.phase = ShopPurchasePhase::Message;
}

fn prompt_to_weapon(st: &mut ShopPurchaseState, p: ShopPrompt) {
    st.prompt = Some(p);
    st.msg_back = ShopMsgReturn::WeaponMenu;
    st.phase = ShopPurchasePhase::Message;
}

fn prompt_to_exit(st: &mut ShopPurchaseState, p: ShopPrompt) {
    st.prompt = Some(p);
    st.msg_back = ShopMsgReturn::Exit;
    st.phase = ShopPurchasePhase::Message;
}

/// A1 购买链接线一步：输入 `$16` 手柄位，推进一帧状态流（纯逻辑、确定性、无 I/O）。
///
/// 语义逐段对应 ROM：主环移动/分发（bank00:$9D02-$9D55）→ 四入口
/// 武器 `$9D5E` / 连射 `$9E86` / 发货 `$9F03` / 退出 `bank01:$A023`；
/// 扣减（价 10/6/4，`$018E`）→ 校验（不足/满级/满仓/wing 4 类）→ 生效写入。
/// 越界/未证输入一律 fail-closed 不动作（不发明）。
pub fn shop_purchase_step(st: &mut ShopPurchaseState, buttons: u8) -> ShopFx {
    match st.phase {
        ShopPurchasePhase::MainRing => step_main_ring(st, buttons),
        ShopPurchasePhase::WeaponMenu => step_weapon_menu(st, buttons),
        ShopPurchasePhase::RapidMenu => step_rapid_menu(st, buttons),
        ShopPurchasePhase::ShipSlotMenu => step_ship_slot_menu(st, buttons),
        ShopPurchasePhase::ShipTypeMenu => step_ship_type_menu(st, buttons),
        ShopPurchasePhase::Message => step_message(st, buttons),
        ShopPurchasePhase::Exited => ShopFx::None,
    }
}

fn step_main_ring(st: &mut ShopPurchaseState, buttons: u8) -> ShopFx {
    // 光标移动（bank00:$9D02-$9D23）：Up 优先；`DEC $0400` 负 ⇒ 3；`INC $0400` ==4 ⇒ 0。
    if buttons & BTN_UP != 0 {
        let n = st.cursor.wrapping_sub(1);
        st.cursor = if n & 0x80 != 0 { 3 } else { n };
    } else if buttons & BTN_DOWN != 0 {
        let n = st.cursor.wrapping_add(1);
        st.cursor = if n == 4 { 0 } else { n };
    }
    // 选中（bank00:$9D3A `AND #$D0`）→ RTS 跳转表 bank00:$9D56（字=入口−1）四入口。
    if buttons & BTN_SELECT == 0 {
        return ShopFx::None;
    }
    match st.cursor {
        0 => {
            // 武器入口 bank00:$9D5E：商品光标 init := `$34`（$9D6C）。
            st.item_cursor = st.weapon.slot;
            st.phase = ShopPurchasePhase::WeaponMenu;
            ShopFx::None
        }
        1 => {
            // 连射入口 bank00:$9E86：`$3C`≥4 ⇒ 满级消息（$9E94-$9EA5），否则确认环。
            if st.rapid_level >= 4 {
                prompt_to_main(st, ShopPrompt::RapidMax);
            } else {
                st.phase = ShopPurchasePhase::RapidMenu;
            }
            ShopFx::None
        }
        2 => {
            // 发货入口 bank00:$9F03：`$0180`∈[2,7] 才可购（$9F03-$9F19），否则消息回主环。
            if (2..=7).contains(&st.stage) {
                st.item_cursor = st.stage;
                st.ship_index = st.stage;
                st.ship_lo = st.stage;
                st.phase = ShopPurchasePhase::ShipSlotMenu;
            } else {
                prompt_to_main(st, ShopPrompt::ShipStageBlocked);
            }
            ShopFx::None
        }
        3 => {
            // 退出入口 bank01:$A023：隐藏选项门**两触发源 OR**（AR65-1，review-65 §1.2）——
            // (a) `($18&$33)==$11`（bank01:$A023-$A029 `c9 11 f0 07`→$A032 置位块）；
            // (b) `$0180==8`（bank01:$A02B-$A030 `ad 80 01 c9 08 d0 16`——不等于 8 才跳 $A048）。
            // 任一触发 ⇒ `$32` bit5 置位（bank01:$A037-$A03B `ORA #$20/STA $32`，登记不解释）
            // + 文本 `$A65C` 消息 + ack 等待（bank01:$A03D-$A045 `JSR $A09F`，ack 门 $A0CF `AND #$C0`），
            // ack 后才合流 bank01:$A048；不触发 ⇒ 直接合流 $A048。
            // 转场（`text $A31F/$E6D9/LDA #$03/JSR $EE03/JMP $EEFB`）交 flow，本线不发明。
            if st.pad2 & 0x33 == 0x11 || st.stage == 8 {
                st.pickup.flags |= 0x20;
                // AR65-2：消息 ack 等待相位（第 7 变体 `ExitHidden`，文案未证只落枚举）。
                prompt_to_exit(st, ShopPrompt::ExitHidden);
            } else {
                st.phase = ShopPurchasePhase::Exited;
            }
            ShopFx::None
        }
        // `$0400` 已证值域 0..3（N-0002）；越界 fail-closed 不动作。
        _ => ShopFx::None,
    }
}

fn step_weapon_menu(st: &mut ShopPurchaseState, buttons: u8) -> ShopFx {
    // 子光标移动 sub_9E40（bank00:$9E40-$9E72）：边界 $11=0/$10=5（bank00:$9D77-$9D7D）。
    if buttons & BTN_LEFT != 0 {
        st.item_cursor = if st.item_cursor == 0 {
            5
        } else {
            st.item_cursor.wrapping_sub(1)
        };
    } else if buttons & BTN_RIGHT != 0 {
        st.item_cursor = if st.item_cursor == 5 {
            0
        } else {
            st.item_cursor.wrapping_add(1)
        };
    }
    if buttons & BTN_B != 0 {
        // B ⇒ 清行回主环（bank00:$9D82-$9D88 → $9CE9/$9CEC）。
        return to_main_ring(st);
    }
    if buttons & BTN_A == 0 {
        return ShopFx::None;
    }
    // A ⇒ 校验链（bank00:$9D91-$9DE0，先校验后扣款）。
    if st.item_cursor == 5 {
        // 翼条目（光标=5）：已购 ⇒ wing_already_owned（$9D98-$9DAA）。
        if st.pickup.flags & 0x80 != 0 {
            prompt_to_weapon(st, ShopPrompt::WingAlreadyOwned);
            return ShopFx::None;
        }
    } else if st.item_cursor == st.weapon.slot {
        // 同型：无翼且 `$33`≥2 ⇒ wing_required；有翼 ⇒ 满级检查（$9DAD-$9DE0）。
        if st.pickup.flags & 0x80 == 0 {
            if st.weapon.level >= 2 {
                prompt_to_weapon(st, ShopPrompt::WingRequired);
                return ShopFx::None;
            }
        } else if st.weapon.level >= 7 {
            prompt_to_weapon(st, ShopPrompt::WeaponMax);
            return ShopFx::None;
        }
    }
    // 扣款（价 10，bank00:$9DE8-$9DF4 = `l2ports::digit_conversion` 同片段，W-NEW4 旧名遗留）。
    let (money, ok) = crate::l2ports::digit_conversion(st.money);
    if !ok {
        prompt_to_main(st, ShopPrompt::InsufficientFunds);
        return ShopFx::None;
    }
    st.money = money;
    // 生效（bank00:$9DFA-$9E1B）：光标=5 ⇒ `$32` bit7 + far-call `$92C5`（未移植，挂起）；
    // 否则 far-call `bank02:$922B`（关内拾取同例程）——A=`$0401`（far-call 跳板 $E649 保活 A）。
    if st.item_cursor == 5 {
        st.pickup.flags |= 0x80;
        ShopFx::WingBoughtFarCallPending
    } else {
        crate::weapons::pickup_handler(&mut st.weapon, st.item_cursor, &mut st.pickup);
        ShopFx::WeaponBought
    }
}

fn step_rapid_menu(st: &mut ShopPurchaseState, buttons: u8) -> ShopFx {
    if buttons & BTN_B != 0 {
        return to_main_ring(st);
    }
    if buttons & BTN_A == 0 {
        return ShopFx::None;
    }
    // 扣款（价 6，bank00:$9ECD-$9ED9）。
    let (money, ok) = crate::l2ports::shop_spend_rapid(st.money);
    if !ok {
        prompt_to_main(st, ShopPrompt::InsufficientFunds);
        return ShopFx::None;
    }
    st.money = money;
    // 生效：`INC $3C`（bank00:$9EDF）；==4 ⇒ 重入 $9E86 走满级消息回主环（$9EE1-$9EA5）。
    st.rapid_level = st.rapid_level.wrapping_add(1);
    if st.rapid_level == 4 {
        prompt_to_main(st, ShopPrompt::RapidMax);
    }
    ShopFx::RapidBought
}

fn step_ship_slot_menu(st: &mut ShopPurchaseState, buttons: u8) -> ShopFx {
    // 槽光标移动 sub_9E40：边界 $11=`$0402`(=stage)/$10=7（bank00:$9F39-$9F42）；
    // `$0403` 每环跟踪 `$0401`（bank00:$9F45-$9F50）。
    if buttons & BTN_LEFT != 0 {
        st.item_cursor = if st.item_cursor == st.ship_lo {
            7
        } else {
            st.item_cursor.wrapping_sub(1)
        };
    } else if buttons & BTN_RIGHT != 0 {
        st.item_cursor = if st.item_cursor == 7 {
            st.ship_lo
        } else {
            st.item_cursor.wrapping_add(1)
        };
    }
    st.ship_index = st.item_cursor;
    if buttons & BTN_B != 0 {
        return to_main_ring(st);
    }
    if buttons & BTN_A == 0 {
        return ShopFx::None;
    }
    if st.ship_index as usize >= st.delivery.len() {
        // `$0403` 越出已证域（W-NEW5 未对账）⇒ fail-closed 不动作。
        return ShopFx::None;
    }
    // A ⇒ 弹种环：`$0401` := 旧弹种（ROL×3/AND #$03 = 高 2 位，bank00:$9F62-$9F72）。
    st.item_cursor = (st.delivery[st.ship_index as usize] >> 6) & 0x03;
    st.phase = ShopPurchasePhase::ShipTypeMenu;
    ShopFx::None
}

fn step_ship_type_menu(st: &mut ShopPurchaseState, buttons: u8) -> ShopFx {
    // 弹种光标移动 sub_9E40：边界 $11=0/$10=3（bank00:$9F8C-$9F92）。
    if buttons & BTN_LEFT != 0 {
        st.item_cursor = if st.item_cursor == 0 {
            3
        } else {
            st.item_cursor.wrapping_sub(1)
        };
    } else if buttons & BTN_RIGHT != 0 {
        st.item_cursor = if st.item_cursor == 3 {
            0
        } else {
            st.item_cursor.wrapping_add(1)
        };
    }
    if buttons & BTN_B != 0 {
        return to_main_ring(st);
    }
    if buttons & BTN_A == 0 {
        return ShopFx::None;
    }
    if st.ship_index as usize >= st.delivery.len() {
        return ShopFx::None;
    }
    // 扣款（价 4，bank00:$9FAB-$9FB7）。
    let (money, ok) = crate::l2ports::shop_spend_ship(st.money);
    if !ok {
        prompt_to_main(st, ShopPrompt::InsufficientFunds);
        return ShopFx::None;
    }
    st.money = money;
    // 生效（bank00:$9FBD-$9FDA）：`$0178+Y` := `(type<<6)|qty`，
    // qty = `shop_ship_qty_next`（min((old&$3F)+1,$3F)，bank00:$9FC0），
    // type 打包 = ROR×3/AND #$C0 = `($0401&3)<<6`；购后停留弹种环（$9FE0）。
    let qty = crate::l2ports::shop_ship_qty_next(st.delivery[st.ship_index as usize]);
    st.delivery[st.ship_index as usize] = ((st.item_cursor & 0x03) << 6) | qty;
    ShopFx::ShipBought
}

fn step_message(st: &mut ShopPurchaseState, buttons: u8) -> ShopFx {
    // ack（bank01:$A0CF `AND #$C0` A/B 任一）⇒ 清行回目标面板（sub_A06C 渲染线不落）。
    if buttons & BTN_ACK == 0 {
        return ShopFx::None;
    }
    st.prompt = None;
    match st.msg_back {
        ShopMsgReturn::MainRing => {
            st.phase = ShopPurchasePhase::MainRing;
            st.cursor = 0;
        }
        ShopMsgReturn::WeaponMenu => {
            st.phase = ShopPurchasePhase::WeaponMenu;
            st.item_cursor = st.weapon.slot;
        }
        ShopMsgReturn::Exit => {
            // AR65-2：ack 后合流 bank01:$A048 转场（交 flow，本线不发明）。
            st.phase = ShopPurchasePhase::Exited;
        }
    }
    ShopFx::None
}

/// 商店金钱入账一步（bank02:$8FA9，A1-CAP-2）：`money := shop_money_earn(money)`（+3，
/// 进位即封顶 `$FF`）。**触发时机未接线**（谁在何时 +3 未证）——此处只接 RAM 写效果。
pub fn shop_money_credit(st: &mut ShopPurchaseState) {
    st.money = crate::l2ports::shop_money_earn(st.money);
}
