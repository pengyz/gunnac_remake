//! game-core::ui —— 金钱系统 + 商店状态机 + 价目记录（M3 UI/商店/金钱，t77）。
//!
//! 语义来源：`docs/spec/shop-economy.md` R-money-1..6 / R-shop-1..8 + `docs/spec/ui-flow.md`
//! R-ui-*；常量=C-money-bcd/C-money-addend-tbl/C-money-cap/C-extend-threshold（constants v3）。
//! **未证不发明**：卖出机制/金钱消费侧（TODO）、价目字段切分（H-05）、A 键切页语义（TODO）。
//!
//! UI 输入语义注记（t52 后口径）：**A/B 键分工随场景**——实机 B=主武器连发（B 门控），
//! 商店 B=切页/双 B 退出；同一物理键两套语义，平台侧输入策略不得跨场景混用。
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
pub fn money_spend(_money: &mut MoneyCounter, _price: u16) -> Option<()> {
    None
}
