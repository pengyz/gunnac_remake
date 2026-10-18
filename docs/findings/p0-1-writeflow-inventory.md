# P0-1 预研：L2-DIFF 写流清单与五族投影归类（p0-1-writeflow-inventory）

> 日期：2026-10-18｜角色：decomp-analyst（只读取证/归类；**不改代码、不跑 Mesen/cargo**）｜任务卡：task-95
> 前置：L4 0 容差攻坚方案（[l4-zero-tolerance-attack-plan.md](l4-zero-tolerance-attack-plan.md)，下称"方案"）P0-1 写流投影。
> 纪律：AGENTS.md §3（A/B/C 分级；未证不发明；数字可复算）；本文件是**唯一写入产物**；未改 `crates/`/`data/`/`gates/`/`rom/`/`reference/oracle`，未 git add/commit。
> 数据源（只读）：`oracle/subroutines/l2_cases.jsonl`（CN 121 例）与 `l2_cases_jp.jsonl`（JP 121 例）、`crates/game-oracle/src/l2.rs`、`crates/game-oracle/src/bin/l2_diff.rs`、`crates/game-core/src/terrain.rs`、`docs/findings/{l4-zero-tolerance-attack-plan,write-set-attribution,text-record-write}.md`、`adversarial-review-69/70/71/72/73.md`。
> 动态缺口：本卡不采集；本机无 Mesen2 串行锁/不跑长任务 ⇒ 动态字段需求**只给清单**，不产出实测。

> **【2026-10-18 更正（task-96 复核 F1-F4 → task-98）】**
> F1（中）：原文 `$0130-$016F(32B)` 自相矛盾（该范围=64B）。已改**双写**：**row_buf=`$0130-$014F`(32B)**、**probe 窗口=`$0130-$016F`(64B，含 `$0150-$015F` 表与 `$160-$167`，其中 palette 源=`$161-$167`)**；§0/§5 字段清单已补 `$161-$167`/`$0150-$015F`。
> F2（低）：§1.2 页 1 示例 `$0190` 非实际写地址 → 改为实际集合 `$018A/$018E/$018F/$0199/$019A/$019F/$01A0`。
> F3（info）：术语 "208 条写点" → "**208 条末值写集条目**"（同址折叠、非原始写次数）。
> F4（info）：`Out.writes` 引用修正为 `l2.rs:133`（`L2Bus.writes` 另注 `l2.rs:28`）。

---

## 0. 结论速览

| # | 项 | 结论 |
|---|---|---|
| 1 | **L2-DIFF 当前写流差异** | **零差异**：CN/JP 各 121/121 全绿（仓库留痕 A：`adversarial-review-69.md:63-64`、`-73.md:18`；本卡未重跑）。"缺写/多写/值差"三类当前均为 0；**"顺序差"结构上不可观测**（见 #3）。 |
| 2 | **L2 写流语料规模（A 复算）** | 121 例 / 29 子程序 / **208 条末值写集条目**（同址折叠，非原始写次数；§2.2）/ **54 个唯一地址**；写集大小分布 `{0:30, 1:41, 2:25, 3:3, 4:8, 5:8, 6:6}`；地址分类：对象 RAM `$0400-$07FF` **119** 条、zp **58**、页 1 游戏变量 **30**、**渲染相邻（PPU 寄存器）仅 1 条**。 |
| 3 | **格式限制（A 代码事实）** | `Out.writes` = `BTreeMap<u16,u8>`（`Out.writes`=`l2.rs:133`；`L2Bus.writes`=`l2.rs:28`）⇒ **同址多次写折叠为末值；写序丢失**；`L2Bus.ppu_log` 虽有 `step/addr/val/gate_a/gate_b` 有序日志（`l2.rs:37-43`），但 **`Out` 不含 ppu_log、`diff_case` 不比较不导出**（`l2.rs:752-781`）⇒ "顺序差/重复写"**不能**从 L2-DIFF 得出。 |
| 4 | **五族投影归类（本卡交付）** | ①empty-cell（方案族①投影/nametable 内容空）：**0/208 条**（语料无 row_buf `$0130-$014F`(32B) / probe 窗口 `$0130-$016F`(64B) / attr `$D3-$DA` / `$8E/$8F` / `$160` 写；详见 §1.2/§5）；②fade/ending（族②调色板/滚动）：**1/208 条**=`$2005=55`（`sc-1-ui-2005-double`）；③四联（族③写进但渲不出）：**0/208**；④0（0 值写；方案族④=3B 表 0 像素，双读见 §3）：**49/208 条 / 17 地址**；⑤spur（族⑤多画/资产错配）：当前 **0**，风险面=25 例 ≥3 写 + 30 例空写 + 同址折叠（§2.3）。 |
| 5 | **对 P0-1 的判定** | **L2 语料不能作为 P0-1 写流投影的主数据源**（渲染相邻条目 1/208=0.48%，且无顺序信息）；P0-1 的 BG 写流必须来自 phase/probe 动态写流（`write-set-attribution.md`：775,968 写点/9,756 调用，`analysis/` 不入库）+ `$2006/$2007` 协议序。L2 只适合作**值级回归**（已绿）与**写集投影规则反例清单**（§4）。 |
| 6 | **必须动态采集的字段** | 每写点 `(step/cycle, 14-bit addr, val, gate_a, gate_b, 帧/tick, scene)`；BG 写源分类（NMI `nmi_flush` vs `$2007` 直写 vs 文本回放 vs row-buffer）；**row_buf=`$0130-$014F`(32B) / probe 窗口=`$0130-$016F`(64B，含 `$0150-$015F` 表与 `$160-$167`)**；**palette 源=`$161-$167`**；attr=`$D3-$DA`(8B)；`$8E/$8F/$160`；palette `$3F00-$3F1F` 写序；tile code/page 上下文。清单见 §5。 |

---

## 1. 语料与写流总账（A 级：可复算 §6 D1/D2）

### 1.1 语料一致性

| 项 | 值 | 级 |
|---|---|---|
| `l2_cases.jsonl`（CN） | **121 例**、29 子程序 | A（D1） |
| `l2_cases_jp.jsonl`（JP） | **121 例**；**case id 序列与 CN 完全一致**（D1 断言 True） | A |
| 差分面 | regs{a,x,y} + writes + 可选 exit（`l2.rs:127-136`） | A |
| 三方裁定 | golden == 6502 核执行 ROM == game-core 同名函数（`l2.rs:1-14`、`bin/l2_diff.rs:47-72`） | A |
| 当前结果 | CN/JP 均 **121/121 通过**（留痕，本卡未重跑） | A（留痕） |

### 1.2 写点总账（CN 121 例）

- **208 条末值写集条目**（同址折叠、非原始写次数；§2.2）、**54 个唯一地址**；有写用例 **91**、空写用例 **30**（`writes={}`）。
- 写集大小分布：`0→30、1→41、2→25、3→3、4→8、5→8、6→6`。
- 地址分类（按地址域，D2 输出）：

| 地址域 | 条目数 | 占 208 | 说明 |
|---|---|---|---|
| 对象 RAM `$0400-$07FF` | **119** | 57.2% | 玩家/弹丸/敌对象字段（游戏状态） |
| zp `$00-$FF` | **58** | 27.9% | 临时/指针/计数器（游戏状态） |
| 页 1 游戏变量 `$0100-$01FF` | **30** | 14.4% | `$018A/$018E/$018F/$0199/$019A/$019F/$01A0`（金钱/炸药等；**非栈囚**，README 已注：页 1 游戏变量照常记录） |
| **渲染相邻（PPU 寄存器）** | **1** | **0.48%** | `$2005`（`ui_scroll_write`） |
| nametable 行缓冲（row_buf `$0130-$014F` 32B；probe 窗口 `$0130-$016F` 64B）/ attr `$D3-$DA` / `$8E/$8F/$160` | **0** | **0%** | **语料完全不覆盖 P0-1 写流**（A） |

### 1.3 子程序×写点表（29 子程序，A 复算）

| subroutine | cases | writes=0 | entries | unique addrs |
|---|---|---|---|---|
| movement_tick | 14 | 0 | 44 | 4 |
| bomb_drop | 9 | 3 | 30 | 5 |
| money_add_event | 4 | 0 | 24 | 6 |
| pickup_handler | 12 | 0 | 13 | 3 |
| player_x_clamp_hi / lo | 4 / 3 | 0 / 0 | 8 / 6 | 2 / 2 |
| player_y_clamp_hi / lo | 3 / 2 | 0 / 0 | 6 / 4 | 2 / 2 |
| death_money_halve | 6 | 0 | 6 | 1 |
| terrain_load_record | 4 | 0 | 9 | 7 |
| terrain_fetch_params | 1 | 0 | 6 | 6 |
| terrain_loop_step / load | 2 / 1 | 0 / 0 | 4 / 1 | 3 / 1 |
| terrain_x_adjust | 3 | 0 | 3 | 1 |
| shot_config_loader | 2 | 0 | 8 | 6 |
| shot_script_fetch | 2 | 0 | 6 | 3 |
| shot_lut_accumulate | 4 | 0 | 8 | 2 |
| shot_spawn_init | 1 | 0 | 5 | 5 |
| shop_ship_qty_next / money_earn / spend_rapid / spend_ship | 5 / 5 / 3 / 2 | 0 / 0 / 1 / 1 | 5 / 5 / 2 / 1 | 1 / 1 / 1 / 1 |
| digit_conversion | 5 | 2 | 3 | 1 |
| bcd_sub_ammo | 6 | 6 | 0 | 0 |
| consumption_lookup | 7 | 7 | 0 | 0 |
| level_config_index | 4 | 4 | 0 | 0 |
| shot_alloc | 4 | 4 | 0 | 0 |
| shot_schedule_step | 2 | 2 | 0 | 0 |
| **ui_scroll_write** | **1** | 0 | **1** | **1（`$2005`）** |

> 覆盖缺口（对 P0-1 重要）：**地形子程序（5 个）只写 VM 状态**（`$00AA/$00B0/$00B6/$00BC/$0197/$019D/$0014/$0015/$00C2` 等），**不写行缓冲 row_buf `$0130-$014F`(32B)**；行缓冲→nametable 的提交在 `Nametable::nmi_flush`（`terrain.rs:307`），由 NMI 路径调用，**不在 L2 覆盖面**（A：语料 208 条末值写集条目无 row_buf / probe 窗口 `$0130-$016F` 任意写）。

### 1.4 54 地址全清单（A，供反例/值域审计；`n`=条目数）

| 地址 | n | 写者子程序 | 期望值集合 |
|---|---|---|---|
| `$0004` | 5 | shop_ship_qty_next | {1,6,63} |
| `$0014/$0015` | 1/1 | terrain_loop_step | {52} / {18} |
| `$0033/$0034` | 9/2 | pickup_handler | {0,1,6,9,12,13,255} / {3,5} |
| `$00A2/$00A3/$00A6/$00A7` | 1/1/3/1 | terrain_load_record / x_adjust | {0}/{3}/{21,53,227}/{16} |
| `$00AC/$00B2/$00B8/$00BE` | 1 each | terrain_fetch_params | {66}/{7}/{80}/{96} |
| `$00B3/$00C4/$00C5` | 1/3/2 | terrain_load_record / loop_* | {0}/{0,2,5}/{1,16} |
| `$00E0-$00E5` | 4 each | money_add_event | 0/9 等（见 §2.3） |
| `$018A` | 2 | pickup_handler | {1} |
| `$018E` | 17 | death_money_halve, digit_conversion, shop_* | **{0,1,3,6,9,49,64,89,103,127,245,255}** |
| `$018F` | 6 | bomb_drop | {0,3,9,25} |
| `$0199/$019A/$019F/$01A0` | 1/1/1/2 | terrain_* | {20}/{180}/{3}/{0,178} |
| `$0401` | 6 | bomb_drop | {7} |
| `$0413/$0415` | 2/1 | shot_script_fetch / shot_spawn_init | {10,255}/{1} |
| `$0470` | 15 | movement_tick, player_x_clamp_* | {0,3,5,7,48,52,208} |
| `$0483/$0485` | 2/1 | shot_script_fetch / shot_spawn_init | {1,17}/{12} |
| `$048C` | 19 | movement_tick, player_y_clamp_* | {0,2,4,6,48,152,208} |
| `$04E0/$04E1` | 15/6 | movement_tick, clamp_*, bomb_drop | {14,17,102,103,200,243}/{100} |
| `$04F3/$04F5` | 4/1 | shot_lut_accumulate / shot_spawn_init | {91,100,108,109}/{128} |
| `$04FC/$04FD` | 19/6 | movement_tick, clamp_*, bomb_drop | {0,20,97,120,128,130,223}/{50} |
| `$0563/$0565` | 4/1 | shot_lut_accumulate / shot_spawn_init | {41,50,53}/{160} |
| `$05D3` | 2 | shot_script_fetch | {0,10} |
| `$0645` | 1 | shot_spawn_init | {3} |
| `$074A-$074C` | 2/1/1 | shot_config_loader | {3,5}/{2}/{2} |
| `$0765-$0768` | 6/2/1/1 | bomb_drop / shot_config_loader | {2,3,4}/{12}/{11}/{13} |
| **`$2005`** | **1** | **ui_scroll_write** | **{85}** |

---

## 2. 四类写流差异清单：缺写 / 多写 / 值差 / 顺序差

### 2.1 当前状态（A：留痕，未重跑）

| 差异类 | 当前计数 | 依据 | 可复算性 |
|---|---|---|---|
| **缺写**（golden 有、Rust 无） | **0** | CN/JP 121/121（`l2.rs:775-780` 结构相等；复核留痕） | cargo 未跑（避锁）；留痕 A |
| **多写**（golden 无、Rust 有） | **0** | 同上 | 留痕 A |
| **值差**（同址不同末值） | **0** | 同上 | 留痕 A |
| **顺序差**（同址多写次序/重复） | **不可观测（N/A）** | `Out.writes` 是 `BTreeMap` 末值映射（`Out.writes`=`l2.rs:133`；`L2Bus.writes`=`l2.rs:28`）；`ppu_log` 有序但未入 `Out`/未比较（`l2.rs:752-781`） | 需扩展/动态（§5） |

### 2.2 结构性盲区（A 代码事实，必须登记）

1. **同址多写折叠**：`L2Bus::write` 每次 `writes.insert(addr,val)`（`l2.rs:72-73`）⇒ 同一 case 内先写后写只留末值。已知实例：`ui_scroll_write` 的 `$2005` **同值双写**——Rust 侧注释明写"同值双写（Out 写集按地址去重，形状由单测断言）"（`l2.rs:676-683`），即 **L2 语料无法验证双写形状**（需 `ppu_log` 或单测）。
2. **调用内写序丢失**：`ppu_log` 有 `step` 但 `Out` 不含它；`diff_case` 只比 `Out`（`l2.rs:752-781`）⇒ 任何"先写 A 再写 B" vs "先写 B 再写 A"在 L2 中不可区分。
3. **entry/stops 外写不可见**：L2 只框住子程序边界（case `entry`/`stops`），NMI 内的 `nmi_flush`/`$2006/$2007` 直写不在任何 case 中（§1.3）。

### 2.3 "可观测但当前为零"的风险面（供 P0-1 反例清单）

| 风险 | 计数 | 清单（case id） |
|---|---|---|
| **空写用例**（必须保持 `writes={}`） | **30/121（24.8%）** | `digit-1,digit-5,cons-1..7,bd-6..8,ha-1..4,ss-1,ss-2,shot-alloc-1..4,sp-r3-poor5,sp-s2-poor3,ed3-1..6` |
| **≥3 写用例**（多字段原子写，投影需全量） | **25/121（20.7%）** | size=3:3 / 4:8 / 5:8 / 6:6（D2 可列） |
| **多值地址**（跨用例期望值不同；硬编码/查表漂移风险） | **26 个地址** | `$0004,$0033,$0034,$00A6,$00C4,$00C5,$00E0-$00E5,$018E,$018F,$01A0,$0413,$0470,$0483,$048C,$04E0,$04F3,$04FC,$0563,$05D3,$074A,$0765` |
| **0 值写**（投影不得把 0 当"无写"） | **49/208 条 / 17 地址** | `$0033,$00A2,$00B3,$00C4,$00E0-E5(部分),$018E,$018F,$01A0,$0470,$048C,$04FC,$05D3` |

> 说明：26 个多值地址中值域最宽 `$018E`（0..255，17 条写点，5 个子程序）；若 Rust 侧以查表/常量近似，L2 已覆盖其边界（当前绿），但**写入时机/次数**不在比对面。

---

## 3. 五族投影归类（对 L4 方案 §3.0 五族；方案行号 201-209）

> 口径注（B 级映射）：Lead 卡面简写"①empty-cell ②fade/ending ③四联 ④0 ⑤spur"对应方案 §3.0 的
> ①投影/nametable 内容空、②调色板索引绑定（fade-clear/ending 是主要像素样本）、③写进但渲不出（含非独立"四联"帧）、④3B 表残余（**0 直接像素**）、⑤stg02 资产错配（spurious）。其中"④0"本卡**双读**：④a=0 值写（写流语义），④b=方案族④（3B 表，0 像素）——两者均列出，避免混用。

| 族（卡面简写 → 方案族） | L2 写流候选（条/总量） | 用例 id / 地址 / 期望值 | 可静态投影规则 | 可得性 |
|---|---|---|---|---|
| **①empty-cell**（方案①投影/nametable 内容空） | **0/208（0%）** | 无（语料无 row_buf `$0130-$014F`/probe `$0130-$016F`/`$D3-$DA`/`$8E/$8F/$160` 写） | 不适用：① 的输入是 `Nametable::nmi_flush(row_buf[32], attrs[8], addr_lo, addr_hi, attr_lo, pending)`（`terrain.rs:307`），其**输入快照不在 L2 语料** | **必须动态**（§5） |
| **②fade/ending**（方案②调色板索引绑定；fade-clear/ending 为主要像素样） | **1/208（0.48%）** | `sc-1-ui-2005-double`, `bank1:$AB3B`, `$2005=85`（0x55） | `$2005 := A`（滚动寄存器直写；写集按地址去重，双写形状不可判） | 静态可投影**末值**；写序必需动态 |
| **③四联**（方案③写进但渲不出；四联=非独立 4 帧） | **0/208（0%）** | 无（L2 无 tile code / 窗口基 / 跳码写） | 不适用：③ 的判据需要 tile code≥128/08 与 `tile_page`/CHR 窗基上下文（方案 §5.2） | **必须动态/场景级** |
| **④0**（双读：④a 0 值写 / ④b 方案族④ 3B 表） | ④a=**49/208（23.6%）/17 地址**；④b=**0/208** | ④a：`$0470×8`（x-hi-2/3,x-lo-2/3,mt-04,mt-05 等）、`$048C×12`（y-hi-2,y-lo-2,mt-01..06 等）、`$018E×4,$018F×3,$00E0-E5` 等（D2 全列） | **S2 零值保真**：`value==0` 的写必须进入写集（投影层不得跳过/折叠为零写）；末值语义成立（同址后写 0 时前值丢失=ROM 行为） | 静态可判值；0 与"缺写"的区分见 §4-S6（需序） |
| **⑤spur**（方案⑤资产错配/多画；泛化=多余写） | 当前 **0**；风险面=30 空写 + 25 多写 + 同址折叠 | —（当前无 diff；风险清单 §2.3） | **S1/S3 反例保护**：Rust 写集必须与 golden **结构相等**（`l2.rs:775-780`）；空写用例必须保持空集；多写用例不得漏地址 | 静态可判（集合级）；spurious 顺序/重复不可判 |

**关键判定（A）**：①/③/⑤（渲染像素相关主体）在 L2 语料中**没有任何直接写流候选**——L2 的 208 条末值写集条目 99.5% 是游戏状态 RAM。因此：**P0-1 写流投影不能以 L2 为数据源**；L2 只能验证"同名子程序写集（末值）等价"，不能验证渲染写流。

---

## 4. 可静态投影规则（表 + 伪代码）

| 规则 | 内容 | 适用 L2 用例 | 级 |
|---|---|---|---|
| **S1 末值赋值** | 投影写集 = 最后一次写的值：`for (addr,val) in ordered_writes: shadow[addr]=val`；只对最终映射断言 | 全部 91 有写用例 | A（`BTreeMap` 语义；写集比较） |
| **S2 零值保真** | `val==0` 的写**也是写**；禁止"0 即清空/跳过"优化进入投影；若目标语义是清屏，必须由写者显式产生 0 写 | 49 条/17 地址（§2.3） | A（golden 写集含 0） |
| **S3 空写断言** | `writes=={}` 的用例必须保持空集（不得因初始化/默认值产生多写） | 30 例（§2.3） | A |
| **S4 寄存器直写** | `$2005`（以及 `$2000/$2001/$2006/$2007`）按**写值直通**，投影为寄存器写入事件而非 nametable 单元；同址多写需有序日志 | `sc-1-ui-2005-double`（1 例） | A（值）/B（形状需序） |
| **S5 页 1 游戏变量** | `$0100-$01FF` 中 `$018A/$018E/$018F/$0199/$019A/$019F/$01A0` 是**游戏变量**（非栈囚），照常进入写集；不得以"栈页过滤"误删 | 30 条末值写集条目（§1.2） | A（README/语料） |
| **S6 顺序盲区** | 同址多次写 / 写序差异**不可**由 L2 `writes` 判定；不得从 L2 反推"顺序差=0"；如需序，必须扩 `Out`（有序 `ppu_log`/有序写）或动态 trace | 全部（结构性） | A（`l2.rs:28`(L2Bus.writes)/`133`(Out.writes)/`752-781`） |

伪代码（P0-1 投影骨架，仅静态面）：

```text
shadow: Map<u16, u8> = {}
for w in ordered_writes:            # 仅当有序日志可用（L2 扩展或动态 trace）
    shadow[w.addr] = w.val          # S1
assert 0-writes in shadow for every golden 0-entry        # S2
assert shadow == golden.writes (last-value map)           # S3/S5/S1
# 渲染相邻写（$2000-$2007/$3F00 域）必须走事件流而非 shadow 单元表  # S4
# 顺序/重复：L2 不可判，禁止据此断言"顺序差=0"               # S6
```

---

## 5. 必须动态 trace 的部分与所需字段

> 来源约束：`analysis/` 不入库（`write-set-attribution.md:3` 的 775,968 写点/9,756 调用为历史离线数据）；phase/probe 采集需 Mesen2 串行锁（当前 l4-analyst 占用）。以下为**字段需求清单**，本卡不采集。

| 部分 | 为何不能静态投影 | 必需字段 | 期望产源 |
|---|---|---|---|
| ① empty-cell 主体 | `nmi_flush` 的输入（row_buf/attrs/addr/pending）是运行时状态，不在 L2 | **row_buf=`$0130-$014F`(32B)**、**probe 窗口=`$0130-$016F`(64B，含 `$0150-$015F` 表与 `$160-$167`)**、**palette 源=`$161-$167`**、attr=`$D3-$DA`(8B)、`$8E/$8F`、`$160` 的**帧首快照**；`nmi_flush` 调用时刻帧号 | phase/probe（方案 §5.1：`--probe-file` 事件 + `apply_frame_writes`） |
| ② palette/scroll | `$2006/$2007` 地址锁存与自增、`$2005` 双写形状 | 每次 PPU 写：`(step, addr, val, gate_a=$1A, gate_b=$1B)`（字段已存在于 `l2.rs:37-43`，但 L2 未导出）；`$3F00-$3F1F` 写序；**palette 源=`$161-$167`**；attr 组来源（`$D3-$DA`） | L2 `ppu_log` 导出扩展 **或** oracle phase 日志 |
| ③ 写进但渲不出 | tile code 与窗口基/CHR bank 上下文决定"渲不出" | 逐写点 `tile_code`、`$2000` CHR 模式位、窗基映射（FR 规则） | 需要 render_frame `--probe-file` + `idx.json`（方案 §5.1/§5.2） |
| ④0 零值 vs 缺写 | 末值映射无法区分"写了 0"与"没写" | **有序写日志**（addr/val/step）；或 `ppu_log` 全量导出 | L2 扩展（低改动）或动态 |
| ⑤ spurious | 需要渲染器**全帧写集**与 expected/actual 对照 | 全写流（行缓冲+NMI+直写）逐写事件 + 每帧 ntr | 方案 §5.1 全写流纳入投影输入 |
| 顺序差（全局） | 结构性不可观测（S6） | 有序写日志（RAM+PPU） | 同上 |

**建议的低成本 L2 扩展（A/B）**：`Out` 增 `ordered_ppu_log: Vec<PpuWrite>`（结构已存在，`l2.rs:37-43`）+ `ordered_writes: Vec<(u16,u8)>`（`L2Bus` 每次 write 追加）；`diff_case` 增加"golden 不含序则不比较、含序则逐项比较"的可选面孔。**这不能替代 P0-1 的 BG 写流**（L2 覆盖不到 NMI），但可把 ④0 的顺序盲区补上。

---

## 6. 复现命令（PowerShell 可执行；`@'…'@ | python -`）

**D1 语料一致性（§1.1；期望 CN=121、JP=121、id 序列相同）**

```powershell
@'
import json
def load(p): return [json.loads(l) for l in open(p,encoding='utf-8') if l.strip()]
cn=load('oracle/subroutines/l2_cases.jsonl'); jp=load('oracle/subroutines/l2_cases_jp.jsonl')
print('CN',len(cn),'JP',len(jp),'ids_same',[c['id'] for c in cn]==[c['id'] for c in jp])
print('subroutines',len({c['subroutine'] for c in cn}))
'@ | python -
```

**D2 写点总账/零值/多值/地址清单（§1.2/§1.4/§2.3/§3）**

```powershell
@'
import json, collections
rows=[json.loads(l) for l in open('oracle/subroutines/l2_cases.jsonl',encoding='utf-8') if l.strip()]
cls=collections.Counter(); zero=collections.Counter(); byaddr=collections.defaultdict(set); nzero=0
for r in rows:
    w={int(a,16):v for a,v in r['out'].get('writes',{}).items()}
    for a,v in w.items():
        # probe 窗口 $0130-$016F(64B)=row_buf $0130-$014F(32B)+$0150-$015F 表+$160-$167（palette 源 $161-$167）
        if 0x0130<=a<=0x016F or 0x00D3<=a<=0x00DA or a in (0x8E,0x8F,0x160) or 0x2000<=a<=0x2007: cls['renderer-adjacent']+=1
        elif 0x0400<=a<=0x07FF: cls['object RAM']+=1
        elif a<0x0100: cls['zp']+=1
        elif a<0x0200: cls['stack-page vars']+=1
        else: cls['other']+=1
        if v==0: zero[a]+=1; nzero+=1
        byaddr[a].add(v)
print('cases',len(rows),'entries',sum(len(r['out'].get('writes',{})) for r in rows),'unique_addrs',len(byaddr))
print('class',dict(cls)); print('zero entries',nzero,'addrs',len(zero))
print('size_dist',dict(sorted(collections.Counter(len(r['out'].get('writes',{})) for r in rows).items())))
print('empty_write_cases',[r['id'] for r in rows if not r['out'].get('writes')])
print('multi_value_addrs',sum(1 for a,v in byaddr.items() if len(v)>1))
for a in sorted(byaddr):
    if len(byaddr[a])>1: print('$%04X'%a, sorted(byaddr[a]))
'@ | python -
```

**D3 格式/顺序盲区引用（§2.2/§4-S6）**

```powershell
Select-String -Path crates/game-oracle/src/l2.rs -Pattern 'writes: BTreeMap|pub struct Out|pub static|ppu_log|diff_case|同值双写'
Select-String -Path crates/game-core/src/terrain.rs -Pattern 'fn nmi_flush'
Select-String -Path docs/findings/l4-zero-tolerance-attack-plan.md -Pattern '### 3.0|族①|族②|族③|族④|族⑤|P0-1 \| 写流'
```

**D4 差分留痕（§2.1；不重跑 cargo）**

```powershell
Select-String -Path docs/findings/adversarial-review-69.md,docs/findings/adversarial-review-73.md -Pattern 'L2-DIFF: 121/121'
```

---

## 7. 不确定项 / 未做（显式）

1. **未重跑 L2/未跑 cargo**（避串行锁/CPU）："121/121 全绿"为仓库留痕（A 级留痕，非本卡实测）；本卡所有写集数字来自 `l2_cases.jsonl` 静态解析（A）。
2. **"五族"映射含 B 级解释**：卡面简写与方案 §3.0 族名不完全同名（②fade/ending、③四联、④0），本卡已逐族双写并标注"④0"双读；若 Lead/复核以方案族名为准，以方案 §3.0 为准。
3. **顺序差分类为空**：不是"已证无顺序差"，而是 **L2 格式不可观测**（S6，A）；不得写作"顺序差=0"。
4. **动态部分未采集**：§5 字段清单为需求，非实测；`analysis/` 历史 phase 数据不入库，需 P0-0/P0-1 重采。
5. **L2 覆盖 15 段/6,865B**（`rewrite-completion-audit.md` §5：L2-diff 15/394 段）只是"机器差分覆盖面"，与本卡"写流投影数据源"判定不冲突。
6. 本卡不改代码/配置；不 git add/commit；只写本文件。

---

## 8. 自查

- 只写本文件；未 git add/commit；未动 `crates/`/`data/`/`gates/`/`rom/`/`reference/oracle`。
- 级：§1.1/§1.2/§1.3/§1.4/§2.1(清单)/§2.2/§2.3/§3(计数)/§4(规则)/§5(字段引用) 为 A/B（清单与代码事实 A；"五族映射"B）；所有未证/未采均显式。
- 数字可复算：121/121、208/54、30/91、49/17、26 多值、1/208 渲染相邻，均由 D1/D2 输出。
- 对抗复核建议优先证伪：(a) 208/54/分类计数与 D2 一致性；(b) "L2 语料 0 条 nametable/attr 写"是否漏扫地址域；(c) S1-S6 规则与 `l2.rs` 语义是否相符；(d) 五族映射是否过宽（尤其 ④0 双读）；(e) 是否误把"顺序差不可观测"写成"无顺序差"。

【2026-10-18 初版】decomp-analyst，task-95。
