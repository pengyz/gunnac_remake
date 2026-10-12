//! L2 子程序级差分：case 格式 + ROM 执行侧（6502 核）+ Rust 移植侧（game-core::l2ports）。
//!
//! case JSONL（oracle/subroutines/l2_cases.jsonl，先写用例再实现）：
//! `{id, subroutine, entry:"bankNN:AAAA", stops:["bankNN:AAAA"...], context:{r6,r7},
//!   in:{regs:{a,x,y,p}, ram:{"hexaddr":byte}}, out:{regs:{a,x,y}, writes:{"hexaddr":byte}}}`
//!
//! 三方裁定（缺一不可）：
//! 1. 6502 核执行 **ROM 原字节** 产出 out_rom；
//! 2. out_rom == case.out（golden = 期望=ROM 执行结果的记录）；
//! 3. game-core 同名函数产出 out_rust == out_rom（移植等价）。
//!
//! 差分面 = regs{a,x,y} + 内存写集（P 是执行副产物且分支已由输入 P/比较语义决定，
//! 不列入移植比对面；ROM 侧仍完整模拟标志位）。bank 上下文按 t2 布局（rom-forensics §2）：
//! $8000 窗=R6、$A000 窗=R7、$C000=bank14、$E000=bank15 固定。

use crate::cpu6502::{Bus, Cpu, StepResult};
use std::collections::BTreeMap;

pub const MAX_STEPS: u64 = 10_000;

/// L2 执行总线：RAM + PRG-RAM + ROM bank 窗 + PPU/APU 寄存器 stub。
pub struct L2Bus {
    ram: [u8; 0x800],
    prg_ram: [u8; 0x2000],
    prg: Vec<u8>,
    r6: usize,
    r7: usize,
    pub writes: BTreeMap<u16, u8>,
    pub fault: Option<String>,
}

impl L2Bus {
    pub fn new(rom_file: &[u8], r6: usize, r7: usize) -> Result<Self, String> {
        if rom_file.len() < 0x10 + 0x20000 || &rom_file[0..4] != b"NES\x1a" {
            return Err("ROM 头/长度不符（需 iNES、PRG 128K）".into());
        }
        let prg = rom_file[0x10..0x10 + 0x20000].to_vec();
        if r6 > 15 || r7 > 15 {
            return Err(format!("bank 上下文越界 r6={r6} r7={r7}"));
        }
        Ok(Self { ram: [0; 0x800], prg_ram: [0; 0x2000], prg, r6, r7, writes: BTreeMap::new(), fault: None })
    }
}

impl Bus for L2Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x401F => 0, // PPU/APU stub：读 0（L2 纯函数不依赖）
            0x4020..=0x5FFF => 0,
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0x9FFF => self.prg[self.r6 * 0x2000 + (addr - 0x8000) as usize],
            0xA000..=0xBFFF => self.prg[self.r7 * 0x2000 + (addr - 0xA000) as usize],
            0xC000..=0xDFFF => self.prg[14 * 0x2000 + (addr - 0xC000) as usize],
            0xE000..=0xFFFF => self.prg[15 * 0x2000 + (addr - 0xE000) as usize],
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.writes.insert(addr, val);
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = val,
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize] = val,
            0x2000..=0x401F | 0x4020..=0x5FFF => {} // PPU/APU stub：写忽略（已记入写集）
            _ => {
                // MMC3 寄存器/ROM 窗写入：当前 L2 片段不触发；触发即记故障（fail-closed，不猜语义）
                self.fault = Some(format!("ROM 窗写入未实现：${:04X}=${:02X}", addr, val));
            }
        }
    }

    /// 调用约定压栈：写入 RAM 但**不记入差分写集**（栈操作是约定副产物）。
    /// 游戏变量可合法落在栈页（$018E/$018F 就是），故剔除只走本通道、不按地址一刀切。
    fn write_stack(&mut self, addr: u16, val: u8) {
        if (0x0000..=0x1FFF).contains(&addr) {
            self.ram[(addr & 0x07FF) as usize] = val;
        }
    }
}

/// 一条 L2 用例。
#[derive(Debug, Clone)]
pub struct Case {
    pub id: String,
    pub subroutine: String,
    pub entry: u16,
    pub stops: Vec<u16>,
    pub r6: usize,
    pub r7: usize,
    pub in_regs: (u8, u8, u8, u8), // a,x,y,p
    pub in_ram: Vec<(u16, u8)>,
    pub out_regs: (u8, u8, u8), // a,x,y
    pub out_writes: Vec<(u16, u8)>,
    /// 可选分支落点断言（决策类片段）
    pub out_exit: Option<u16>,
    /// 被调用者内部暂存单元（如 ED03 的 $04-$06）：case 显式声明后从两侧写集剔除
    /// （可见的比对面裁剪，不是全局白名单；栈页固定剔除见 `is_stack`）。
    pub scratch: Vec<u16>,
}

/// 出参（差分面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Out {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub writes: BTreeMap<u16, u8>,
    /// 分支落点（stops 命中地址）——决策类片段的可观察出参（golden 可选字段 exit）
    pub exit: u16,
}

// ---------------- 迷你 JSON（零外部依赖；仅覆盖 case 结构所需）

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Num(f64),
    Str(String),
    Bool(bool),
    Null,
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

pub fn parse_json(s: &str) -> Result<Json, String> {
    let b: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    let v = parse_value(&b, &mut i)?;
    skip_ws(&b, &mut i);
    if i != b.len() {
        return Err(format!("JSON 尾部残留于 {i}"));
    }
    Ok(v)
}

fn skip_ws(b: &[char], i: &mut usize) {
    while *i < b.len() && b[*i].is_whitespace() {
        *i += 1;
    }
}

fn parse_value(b: &[char], i: &mut usize) -> Result<Json, String> {
    skip_ws(b, i);
    match b.get(*i) {
        None => Err("JSON 提前结束".into()),
        Some('{') => {
            *i += 1;
            let mut out = Vec::new();
            skip_ws(b, i);
            if b.get(*i) == Some(&'}') {
                *i += 1;
                return Ok(Json::Obj(out));
            }
            loop {
                skip_ws(b, i);
                let key = match parse_value(b, i)? { Json::Str(s) => s, _ => return Err("对象键非字符串".into()) };
                skip_ws(b, i);
                if b.get(*i) != Some(&':') { return Err(format!("期望 ':' 于 {i}")); }
                *i += 1;
                let val = parse_value(b, i)?;
                out.push((key, val));
                skip_ws(b, i);
                match b.get(*i) {
                    Some(',') => *i += 1,
                    Some('}') => { *i += 1; return Ok(Json::Obj(out)); }
                    other => return Err(format!("对象分隔符异常 {other:?} 于 {i}")),
                }
            }
        }
        Some('[') => {
            *i += 1;
            let mut out = Vec::new();
            skip_ws(b, i);
            if b.get(*i) == Some(&']') { *i += 1; return Ok(Json::Arr(out)); }
            loop {
                out.push(parse_value(b, i)?);
                skip_ws(b, i);
                match b.get(*i) {
                    Some(',') => *i += 1,
                    Some(']') => { *i += 1; return Ok(Json::Arr(out)); }
                    other => return Err(format!("数组分隔符异常 {other:?} 于 {i}")),
                }
            }
        }
        Some('"') => {
            *i += 1;
            let mut s = String::new();
            while let Some(c) = b.get(*i) {
                *i += 1;
                match c {
                    '"' => return Ok(Json::Str(s)),
                    '\\' => {
                        let e = b.get(*i).ok_or("转义截断")?;
                        *i += 1;
                        s.push(match e {
                            'n' => '\n', 't' => '\t', 'r' => '\r', '"' => '"', '\\' => '\\', '/' => '/',
                            'u' => {
                                let hex: String = b.get(*i..*i + 4).ok_or("\\u 截断")?.iter().collect();
                                *i += 4;
                                char::from_u32(u32::from_str_radix(&hex, 16).map_err(|_| "坏 \\u")?)
                                    .ok_or("坏 \\u 码点")?
                            }
                            other => return Err(format!("未知转义 {other}")),
                        });
                    }
                    other => s.push(*other),
                }
            }
            Err("字符串截断".into())
        }
        Some(c) if c.is_ascii_digit() || *c == '-' => {
            let start = *i;
            while let Some(c) = b.get(*i) {
                if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E') { *i += 1; } else { break; }
            }
            let num: String = b[start..*i].iter().collect();
            num.parse::<f64>().map(Json::Num).map_err(|e| e.to_string())
        }
        Some('t') => { expect_word(b, i, "true")?; Ok(Json::Bool(true)) }
        Some('f') => { expect_word(b, i, "false")?; Ok(Json::Bool(false)) }
        Some('n') => { expect_word(b, i, "null")?; Ok(Json::Null) }
        Some(c) => Err(format!("意外字符 {c} 于 {i}")),
    }
}

fn expect_word(b: &[char], i: &mut usize, w: &str) -> Result<(), String> {
    for c in w.chars() {
        if b.get(*i) != Some(&c) { return Err(format!("期望 {w}")); }
        *i += 1;
    }
    Ok(())
}

fn obj_get<'a>(obj: &'a [(String, Json)], key: &str) -> Result<&'a Json, String> {
    obj.iter().find(|(k, _)| k == key).map(|(_, v)| v).ok_or_else(|| format!("缺字段 {key}"))
}

fn as_u8(v: &Json) -> Result<u8, String> {
    match v { Json::Num(n) if *n >= 0.0 && *n <= 255.0 && n.fract() == 0.0 => Ok(*n as u8), other => Err(format!("非字节值 {other:?}")) }
}

fn as_usize(v: &Json) -> Result<usize, String> {
    match v { Json::Num(n) if *n >= 0.0 && n.fract() == 0.0 => Ok(*n as usize), other => Err(format!("非计数 {other:?}")) }
}

/// "bankNN:AAAA" → (bank 号, CPU 地址)。
fn parse_bank_addr(s: &str) -> Result<(usize, u16), String> {
    let (bank_s, addr_s) = s.split_once(':').ok_or_else(|| format!("地址标签缺 ':'：{s}"))?;
    let bank: usize = bank_s.strip_prefix("bank").ok_or("标签须 bankNN:AAAA")?.parse().map_err(|e| format!("{e}"))?;
    let addr = u16::from_str_radix(addr_s, 16).map_err(|e| format!("{e}"))?;
    Ok((bank, addr))
}

fn parse_hex_key(s: &str) -> Result<u16, String> {
    u16::from_str_radix(s, 16).map_err(|e| format!("RAM/写集键须为 hex：{s} ({e})"))
}

fn byte_map(v: &Json) -> Result<Vec<(u16, u8)>, String> {
    match v {
        Json::Obj(entries) => entries.iter().map(|(k, v)| Ok((parse_hex_key(k)?, as_u8(v)?))).collect(),
        other => Err(format!("期望对象映射，得 {other:?}")),
    }
}

/// 解析 JSONL 用例文件。
pub fn parse_cases(jsonl: &str) -> Result<Vec<Case>, String> {
    let mut out = Vec::new();
    for (lineno, line) in jsonl.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let v = parse_json(line).map_err(|e| format!("第 {} 行: {e}", lineno + 1))?;
        let obj = match v { Json::Obj(o) => o, _ => return Err(format!("第 {} 行非对象", lineno + 1)) };
        let id = match obj_get(&obj, "id")? { Json::Str(s) => s.clone(), _ => return Err("id 非字符串".into()) };
        let subroutine = match obj_get(&obj, "subroutine")? { Json::Str(s) => s.clone(), _ => return Err("subroutine 非字符串".into()) };
        let (entry_bank, entry) = match obj_get(&obj, "entry")? { Json::Str(s) => parse_bank_addr(s)?, _ => return Err("entry 非字符串".into()) };
        let stops = match obj_get(&obj, "stops")? {
            Json::Arr(items) => items.iter().map(|it| match it {
                Json::Str(s) => Ok(parse_bank_addr(s)?.1),
                _ => Err("stops 须为字符串数组".to_string()),
            }).collect::<Result<Vec<u16>, String>>()?,
            _ => return Err("stops 非数组".into()),
        };
        let ctx = match obj_get(&obj, "context")? { Json::Obj(o) => o.clone(), _ => return Err("context 非对象".into()) };
        let r6 = as_usize(obj_get(&ctx, "r6")?)?;
        let r7 = as_usize(obj_get(&ctx, "r7")?)?;
        // bank 标签 ↔ 映射窗一致性（t2 §2）：$8000 窗=R6、$A000 窗=R7
        let expected = match entry {
            0x8000..=0x9FFF => r6,
            0xA000..=0xBFFF => r7,
            _ => return Err(format!("{id}: entry 须落在 $8000-$BFFF 映射窗")),
        };
        if expected != entry_bank {
            return Err(format!("{id}: entry bank{entry_bank:02} 与 context 不符（窗寄存器={expected}）"));
        }
        let in_o = match obj_get(&obj, "in")? { Json::Obj(o) => o.clone(), _ => return Err("in 非对象".into()) };
        let regs = match obj_get(&in_o, "regs")? { Json::Obj(o) => o.clone(), _ => return Err("in.regs 非对象".into()) };
        let in_regs = (
            as_u8(obj_get(&regs, "a")?)?, as_u8(obj_get(&regs, "x")?)?,
            as_u8(obj_get(&regs, "y")?)?, as_u8(obj_get(&regs, "p")?)?,
        );
        let in_ram = byte_map(obj_get(&in_o, "ram")?)?;
        let out_o = match obj_get(&obj, "out")? { Json::Obj(o) => o.clone(), _ => return Err("out 非对象".into()) };
        let oregs = match obj_get(&out_o, "regs")? { Json::Obj(o) => o.clone(), _ => return Err("out.regs 非对象".into()) };
        let out_regs = (as_u8(obj_get(&oregs, "a")?)?, as_u8(obj_get(&oregs, "x")?)?, as_u8(obj_get(&oregs, "y")?)?);
        let out_writes = byte_map(obj_get(&out_o, "writes")?)?;
        let out_exit = match out_o.iter().chain(obj.iter()).find(|(k, _)| k == "exit") {
            Some((_, Json::Str(s))) => Some(parse_bank_addr(s)?.1),
            Some((_, other)) => return Err(format!("exit 须为 bankNN:AAAA：{other:?}")),
            None => None,
        };
        let scratch = match obj.iter().find(|(k, _)| k == "scratch") {
            Some((_, Json::Arr(items))) => items.iter().map(|it| match it {
                Json::Str(s) => parse_hex_key(s),
                other => Err(format!("scratch 须为 hex 字符串数组：{other:?}")),
            }).collect::<Result<Vec<u16>, String>>()?,
            Some((_, other)) => return Err(format!("scratch 非数组：{other:?}")),
            None => Vec::new(),
        };
        out.push(Case { id, subroutine, entry, stops, r6, r7, in_regs, in_ram, out_regs, out_writes, out_exit, scratch });
    }
    Ok(out)
}

/// 调用约定哨兵：stops 为空的用例按"调用子程序"执行——预压返回地址使最终 RTS 落到
/// STOP_SENTINEL（永不取指执行），即跑到 RTS/RTI 收集出参（任务约定）。
const STOP_SENTINEL: u16 = 0x0001;

/// ROM 执行侧：6502 核跑 ROM 原字节片段。
pub fn run_rom_case(rom: &[u8], case: &Case) -> Result<Out, String> {
    let mut bus = L2Bus::new(rom, case.r6, case.r7)?;
    for (a, v) in &case.in_ram {
        bus.write(*a, *v); // 预置 RAM（同时会进写集——先清空再跑）
    }
    bus.writes.clear();
    let (a, x, y, p) = case.in_regs;
    let mut cpu = Cpu::new(a, x, y, 0xFD, p, case.entry);
    let call_mode = case.stops.is_empty();
    let mut depth = 0i32; // JSR/RTS 配对：片段内层 RTS 归位，仅深度 0 的 RTS 才是逃逸
    if call_mode {
        // 压入 (STOP_SENTINEL-1)，RTS 后 PC = STOP_SENTINEL（栈写入已被 is_stack 剔除）
        let ret = STOP_SENTINEL.wrapping_sub(1);
        bus.write(0x01FD, (ret >> 8) as u8);
        bus.write(0x01FC, ret as u8);
        bus.writes.clear();
    }
    let stops: Vec<u16> = if call_mode { vec![STOP_SENTINEL] } else { case.stops.clone() };
    for _ in 0..MAX_STEPS {
        if stops.contains(&cpu.pc) {
            return Ok(Out { a: cpu.a, x: cpu.x, y: cpu.y, writes: bus.writes, exit: cpu.pc });
        }
        match cpu.step(&mut bus)? {
            StepResult::Continue => {}
            StepResult::Jsr => depth += 1,
            StepResult::Rts => {
                if depth > 0 {
                    depth -= 1;
                } else if !call_mode {
                    return Err(format!("{}: 未到 stops 即遇 RTS（case 边界错？）", case.id));
                }
            }
            StepResult::Rti => {
                return Err(format!("{}: RTI 终止（本 harness 仅支持 RTS 收尾，case 需显式 stops）", case.id));
            }
            StepResult::Brk => return Err(format!("{}: BRK 触发（异常路径）", case.id)),
        }
        if let Some(f) = &bus.fault {
            return Err(format!("{}: {f}", case.id));
        }
    }
    Err(format!("{}: 超过 MAX_STEPS({MAX_STEPS})（未收敛）", case.id))
}

/// Rust 移植侧：game-core 同名函数。
pub fn run_rust_case(case: &Case) -> Result<Out, String> {
    use game_core::l2ports;
    let mut writes = BTreeMap::new();
    let mut exit_at = 0u16;
    let (a_in, x_in, y_in, _) = case.in_regs;
    let ram = |addr: u16| -> u8 {
        case.in_ram.iter().find(|(a, _)| *a == addr).map(|(_, v)| *v).unwrap_or(0)
    };
    let (a, x, y) = match case.subroutine.as_str() {
        "player_x_clamp_hi" => {
            let (nx, ny) = l2ports::player_x_clamp_hi(a_in, y_in);
            writes.insert(0x04E0, nx);
            writes.insert(0x0470, ny);
            (nx, x_in, ny)
        }
        "player_x_clamp_lo" => {
            let (nx, ny) = l2ports::player_x_clamp_lo(a_in, y_in);
            writes.insert(0x04E0, nx);
            writes.insert(0x0470, ny);
            (nx, x_in, ny)
        }
        "player_y_clamp_hi" => {
            let (ny, vel) = l2ports::player_y_clamp_hi(a_in, y_in);
            writes.insert(0x04FC, ny);
            writes.insert(0x048C, vel);
            (ny, x_in, vel)
        }
        "player_y_clamp_lo" => {
            let (ny, vel) = l2ports::player_y_clamp_lo(a_in, y_in);
            writes.insert(0x04FC, ny);
            writes.insert(0x048C, vel);
            (ny, x_in, vel)
        }
        "digit_conversion" => {
            let (nv, stepped) = l2ports::digit_conversion(ram(0x018E));
            if stepped {
                writes.insert(0x018E, nv);
            }
            (nv, x_in, y_in)
        }
        "consumption_lookup" => {
            let ammo = ram(0x018F);
            let idx = ram(0x003B);
            let y = l2ports::consumption_lookup(ammo, idx);
            (ammo, x_in, y)
        }
        "bomb_drop" => {
            match l2ports::bomb_drop(ram(0x0016), ram(0x0401), ram(0x018F), ram(0x003B)) {
                l2ports::BombDrop::NotFired { a } => (a, x_in, y_in),
                l2ports::BombDrop::Fired { ammo_new, bomb_type, y } => {
                    writes.insert(0x0765, bomb_type);
                    writes.insert(0x018F, ammo_new);
                    writes.insert(0x04FD, ram(0x04FC)); // y 快照
                    writes.insert(0x04E1, ram(0x04E0)); // x 快照
                    writes.insert(0x0401, 7); // mode := 7
                    (7, x_in, y)
                }
            }
        }
        "movement_tick" => {
            let st = game_core::player::PlayerState {
                x: ram(0x04E0), y: ram(0x04FC), x_subpix: ram(0x0470), y_subpix: ram(0x048C),
                ..Default::default()
            };
            let held = ram(0x0018);
            let vx = game_core::player::VelocityPair { subpix: ram(0x0550), int: ram(0x056C) };
            let vy = game_core::player::VelocityPair { subpix: ram(0x0588), int: ram(0x05A4) };
            let ns = game_core::player::movement_tick(&st, held, vx, vy);
            writes.insert(0x04FC, ns.y);
            writes.insert(0x048C, ns.y_subpix);
            // 退出寄存器 = ROM 片段副产物（按指令流导出）：X 门跳过时 A=held&3、Y=亚像素；
            // X 门执行时 A=像素结果、Y=X 亚像素（TAY 覆写）。
            if held & 0x03 != 0 {
                writes.insert(0x04E0, ns.x);
                writes.insert(0x0470, ns.x_subpix);
                (ns.x, x_in, ns.x_subpix)
            } else {
                (held & 0x03, x_in, ns.y_subpix)
            }
        }
        "pickup_handler" => {
            let mut st = game_core::weapons::WeaponState {
                slot: ram(0x0034), level: ram(0x0033), bomb_ammo: ram(0x018F),
            };
            let mut ctx = game_core::weapons::PickupContext {
                threshold: ram(0x018C),
                consecutive: ram(0x018A),
                flags: ram(0x0032),
                reward_pending: false,
            };
            let before = st.level;
            match game_core::weapons::pickup_handler(&mut st, a_in, &mut ctx) {
                game_core::weapons::PickupOutcome::KernelLevel { level } => {
                    writes.insert(0x0033, level);
                    if ctx.consecutive != ram(0x018A) {
                        writes.insert(0x018A, ctx.consecutive); // 回 kernel 前的连收计数写入
                    }
                    // 退出寄存器 = ROM 副产物：kernel 回卷路径 A=$0C，否则 A=$32 入参
                    let a_out = if before == 0xFF { 0x0C } else { ram(0x0032) };
                    (a_out, x_in, a_in) // 同型路径经 TAY ⇒ Y=拾取值
                }
                game_core::weapons::PickupOutcome::DeepLevel { level } => {
                    writes.insert(0x0033, level);
                    (ram(0x0032) & 0x04, x_in, a_in) // 收尾 LDA $32/AND #$04
                }
                game_core::weapons::PickupOutcome::Consecutive { consecutive, .. } => {
                    writes.insert(0x018A, consecutive);
                    (ctx.threshold, x_in, a_in) // 退出前 LDA $018C
                }
                game_core::weapons::PickupOutcome::Switched { slot } => {
                    writes.insert(0x0034, slot);
                    // 异型分支不走 TAY（$922D 直跳 $9248）⇒ Y=入参 Y；STA $34 不改 A
                    (slot, x_in, y_in)
                }
            }
        }
        "level_config_index" => {
            let level = ram(0x0033);
            let idx = game_core::weapons::config_index_for_level(level);
            let off = game_core::weapons::CONFIG_OFFSETS[idx as usize];
            // 退出寄存器 = ROM 副产物：A=Y=索引，X=表值；STX $04 为暂存（scratch 过滤）
            (idx, off, idx)
        }
        "shot_schedule_step" => {
            let (counter, period) = (ram(0x072E), ram(0x074A));
            let (new_counter, spawn) = game_core::shot::schedule_step(counter, period);
            // 片段止于分支落点（INC $072E 在 JSR 之后未达）——出参=分支落点；决策可自证
            assert_eq!(new_counter, if spawn { counter.wrapping_add(1) } else { counter });
            exit_at = if spawn { 0xA205 } else { 0xA232 };
            (counter, x_in, y_in)
        }
        "shot_spawn_init" => {
            let obj = game_core::shot::spawn_init(
                y_in, x_in, ram(0x0011), ram(0x04E0), ram(0x04FC), ram(0x0766 + ram(0x0011) as u16),
            );
            writes.insert(0x055E + y_in as u16, obj.pos_y);
            writes.insert(0x04EE + y_in as u16, obj.pos_x);
            writes.insert(0x063E + y_in as u16, obj.shot_type);
            writes.insert(0x040E + y_in as u16, obj.lifetime);
            writes.insert(0x047E + y_in as u16, obj.script_ptr);
            (obj.script_ptr, ram(0x0011), y_in) // 退出：A=脚本参（LDA $0766,X），X=$11，Y=slot
        }
        "shot_script_fetch" => {
            let slot_off = x_in as u16;
            let (dur, code, new_ptr) = game_core::shot::script_fetch(
                ram(0x047E + slot_off),
                &game_core::shot::ACTION_DURATIONS,
                &game_core::shot::ACTION_STREAM,
            );
            writes.insert(0x040E + slot_off, dur);
            writes.insert(0x05CE + slot_off, code);
            writes.insert(0x047E + slot_off, new_ptr);
            (new_ptr, x_in, new_ptr) // 退出：A=Y=ptr+1（TYA/INY）
        }
        "shot_lut_accumulate" => {
            let slot_off = x_in as u16;
            let code = ram(0x05CE + slot_off);
            let nx = game_core::shot::lut_accumulate_axis(ram(0x04EE + slot_off), code, &game_core::shot::LUT_X)
                .ok_or_else(|| format!("{}: X 轴溢出/借位入死亡路径（未证域，不入比对面）", case.id))?;
            let ny = game_core::shot::lut_accumulate_axis(ram(0x055E + slot_off), code, &game_core::shot::LUT_Y)
                .ok_or_else(|| format!("{}: Y 轴溢出/借位入死亡路径（未证域，不入比对面）", case.id))?;
            writes.insert(0x04EE + slot_off, nx);
            writes.insert(0x055E + slot_off, ny);
            (ny, x_in, code) // 退出：A=Y 轴结果（STA 前驻留），Y=运动码
        }
        "shot_config_loader" => {
            let mut i = x_in as usize;
            let mut slot = 0u16;
            loop {
                let period = game_core::shot::CFG_STREAM[i];
                if period & 0x80 != 0 {
                    break;
                }
                writes.insert(0x074A + slot, period);
                writes.insert(0x0766 + slot, game_core::shot::CFG_STREAM[i + 1]);
                i += 2;
                slot += 1;
            }
            (0xFF, i as u8, slot as u8) // 退出：A=终止负值，X=偏移+2n，Y=对数
        }
        "shot_alloc" => {
            // bank2:$888D 弹槽分配器 —— 语义=game-core::shot::shot_slot_alloc（t48 定谳；
            // 内联臂转同名函数对拍，三方裁定归位）
            let mut state = [0u8; 0x100];
            for y in 0..0x100usize {
                state[y] = ram(0x040E + y as u16);
            }
            match game_core::shot::shot_slot_alloc(&state) {
                game_core::shot::SlotAlloc::Found { slot, a } => (a, x_in, slot),
                game_core::shot::SlotAlloc::Full => (0xF0, x_in, 0xF0), // 满载：A=Y=$F0
            }
        }
        "enemy_load_record" => {
            let (p0, p1, p2, p3) = (ram(0x0300), ram(0x0301), ram(0x0302), ram(0x0303));
            let slot_off = |slot: u8| slot as u16;
            match game_core::enemy::load_record(p0, p1, p2, p3) {
                game_core::enemy::LoadEffect::Spawn { slot, x, loop_cnt, pc_lo, pc_hi } => {
                    writes.insert(0x00C2 + slot_off(slot), loop_cnt);
                    writes.insert(0x00A4 + slot_off(slot), x);
                    writes.insert(0x0197 + slot_off(slot), pc_lo);
                    writes.insert(0x019D + slot_off(slot), pc_hi);
                    writes.insert(0x00B0 + slot_off(slot), 0);
                    (0, slot, y_in + 4)
                }
                game_core::enemy::LoadEffect::Disable { slot, loop_cnt } => {
                    writes.insert(0x00C2 + slot_off(slot), loop_cnt);
                    writes.insert(0x019D + slot_off(slot), 0);
                    (0, slot, y_in + 2)
                }
                game_core::enemy::LoadEffect::Segment { count } => {
                    writes.insert(0x00A3, count);
                    (count, x_in, y_in)
                }
                game_core::enemy::LoadEffect::End => {
                    writes.insert(0x00A2, 0);
                    (0, x_in, y_in)
                }
            }
        }
        "enemy_x_adjust" => {
            let slot_off = x_in as u16;
            let (new_x, exit) = game_core::enemy::vm_x_adjust(ram(0x00A4 + slot_off), ram(0x0300));
            writes.insert(0x00A4 + slot_off, new_x);
            exit_at = if exit { 0x8462 } else { 0x845E };
            if exit { (0, x_in, y_in + 1) } else { (new_x, x_in, y_in + 1) }
        }
        "enemy_loop_load" => {
            let slot_off = x_in as u16;
            let c = game_core::enemy::vm_loop_load(ram(0x0300));
            writes.insert(0x00C2 + slot_off, c);
            (c, x_in, y_in + 1)
        }
        "enemy_loop_step" => {
            let slot_off = x_in as u16;
            let (c, taken, _lo, _hi) = game_core::enemy::vm_loop_step(
                ram(0x00C2 + slot_off), ram(0x0300), ram(0x0301),
            );
            writes.insert(0x00C2 + slot_off, c);
            if taken {
                writes.insert(0x0014, ram(0x0300));
                writes.insert(0x0015, ram(0x0301));
                (ram(0x0300), x_in, 0) // 跳转装载后 LDY #0
            } else {
                (a_in, x_in, y_in + 2) // 跳过 2B
            }
        }
        "enemy_fetch_params" => {
            let slot_off = x_in as u16;
            // （$14),Y 间接寻址：流基址=$14/$15；入口 Y 起三次取参（两次 INY）；TYA 处 Y=入+2
            let base = ram(0x0014) as u16 | (ram(0x0015) as u16) << 8;
            let (p0, p1, p2, lo, hi) = game_core::enemy::vm_fetch_params(
                ram(0x0014), ram(0x0015), y_in + 2, ram(base + y_in as u16),
                ram(base + y_in as u16 + 1), ram(base + y_in as u16 + 2),
            );
            writes.insert(0x00AA + slot_off, a_in);
            writes.insert(0x00B0 + slot_off, p0);
            writes.insert(0x00B6 + slot_off, p1);
            writes.insert(0x00BC + slot_off, p2);
            writes.insert(0x0197 + slot_off, lo);
            writes.insert(0x019D + slot_off, hi);
            (hi, x_in, y_in + 2)
        }
        other => return Err(format!("未知 subroutine {other}（game-core 同名函数缺失）")),
    };
    Ok(Out { a, x, y, writes, exit: exit_at })
}

/// 单用例三方裁定。返回 (rom 侧出参, rust 侧出参, 问题列表)。
pub fn diff_case(rom: &[u8], case: &Case) -> Result<(Out, Out, Vec<String>), String> {
    let out_rom = run_rom_case(rom, case)?;
    let out_rust = run_rust_case(case)?;
    let drop_scratch = |mut o: Out| {
        o.writes.retain(|a, _| !case.scratch.contains(a));
        o
    };
    let out_rom_c = drop_scratch(out_rom.clone());
    let out_rust_c = drop_scratch(out_rust.clone());
    let golden = Out {
        a: case.out_regs.0,
        x: case.out_regs.1,
        y: case.out_regs.2,
        writes: case.out_writes.iter().copied().collect(),
        exit: case.out_exit.unwrap_or(0),
    };
    let cmp_rom = if case.out_exit.is_some() { out_rom_c.clone() } else { Out { exit: 0, ..out_rom_c.clone() } };
    let cmp_rust = if case.out_exit.is_some() { out_rust_c.clone() } else { Out { exit: 0, ..out_rust_c.clone() } };
    let mut problems = Vec::new();
    if cmp_rom != golden {
        problems.push(format!("ROM 执行 ≠ golden：rom={cmp_rom:?} golden={golden:?}"));
    }
    if cmp_rust != cmp_rom {
        problems.push(format!("Rust 移植 ≠ ROM：rust={cmp_rust:?} rom={cmp_rom:?}"));
    }
    Ok((out_rom, out_rust, problems))
}
