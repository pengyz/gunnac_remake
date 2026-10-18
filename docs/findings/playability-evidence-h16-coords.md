# 可玩性取证：H-16 命中判定 + 敌坐标写者链 + Boss/EN-9+（playability-evidence-h16-coords）

> 日期：2026-10-18｜角色：decomp-analyst（只读取证，不写代码）｜任务卡：task-38
> 对象：HEAD `b92139a`（审计期工作区有 shell-win/task-18 并发施工，不在本取证范围）
> 纪律：AGENTS.md §3（A=硬证据/B=可复现静态推理/C=假设；未证不发明；数字可独立复算）
> 性质：**现状盘点 + 接入判定 + 最小取证路径**，不新增游戏语义结论；所有"未证"项显式标注。
> 复现：命令见 §6（`python`，非损坏的 `python3` 别名；工作目录=仓库根）

---

## 0. 结论速览（TL;DR）

| # | 问题 | 结论（精确） |
|---|---|---|
| 1 | **H-16 命中判定本体** | **判定链本体 A 级已证且 `shot.rs` 已实现**（框注册 `bank2:$8847-$888A`、4 边重叠/属性相交 `bank14:$D78B-$D846`、一框一伤清框 `$D7DA`、伤害三路 `$D674`、弹回收 `$87FD`）；**但**残项三件未闭：**W15 玩家被击最上游调用点未定位 / W15′ 玩家框注册点未定位 / W18 `$05DC` 语义双读（小怪 HP vs 死亡倒数）未定谳**，另 **$4C 伤害模式旗（0=一击死）C→B** 与 **普通小怪 HP 字段未定谳（B4）**。→ 玩家弹×敌可接（带 fail-closed），**玩家×敌不可接**。 |
| 2 | **敌坐标写者链** | **静态写点全集：x `$04E0` 69 处 / y `$04FC` 75 处**（seed 逐条；§3.1）；**ObjectSlot 面已实现写者仅 3 处 x + 4 处 y**（bank02 handler 族 `$8337/$836E` 体 + `motion_update` `bank14:$CCD3` 族；玩家面另有 2x+2y 钳制写），其余 **66 x / 71 y 写点**落在 **生成层 `$865C` + banks4-7 行为 handler 体**（Rust `run_handler` 白名单未覆盖）。→ host 内敌槽 `x/y` 恒 0 的**根因可定位**；坐标可见化需 5 项最小接线（§3.5）。 |
| 3 | **Boss / EN-9+** | **B-STG-01-BOSS 维持 L2/部分**（`completeness.md` 原格：✅链定位/脚本格式/EN 族分立/出场触发/巡逻语义；⚠条数展开/W32/W30；❌逐条语义）。STG-01 boss 实体链**静态 A 级**：波次流 `$A3CD` 内 `01 4A`（file `0x103F2`）= 中 boss type `$4A`；波次流 `$A3E4` 内 `01 5F`（file `0x10409`）= 终盘 boss type `$5F`。Rust 侧 **9 个 fail-closed 点**全未接线（§4.3）；**EN-9+ `$ADF8+` 的「5B 同格式延续」本轮复核为 B 级族级判断——按 5B/4B 步进均不能全域对齐（§4.4），不得用于逐条实装**。 |

**给 task-37 的一句话**：弹丸生成/运动/命中**可以接**，但敌 HP/死亡链必须按"中 boss type `$4A` HP=14（A 级）"白名单接入、其余敌型 fail-closed；**玩家被击一律不接**（W15/W15′ 未证）。

---

## 1. 现状盘点：代码 / 测试 / 数据三面

### 1.1 `shot.rs` 已实现面与已证依据（A 级）

| 已实现 API | 已证依据（ROM 指针） | 级 |
|---|---|---|
| `shot_hitbox_register`（7×10 框 x−4..x+3 / y−5..y+5，钳 0/FF，owner=槽偏移，attr=`$30`） | 弹侧注册 `bank2:$8847-$888A`（`$884D/$885B/$8868/$8875/$8881/$8885`） | A |
| `boxes_overlap`（4 边比较） | 阅读端 A `bank14:$D78B-$D7EC`（`$07BE,Y≥$72 ∧ $73≥$07A9,Y ∧ $0794,Y≥$70 ∧ $71≥$077F,Y`） | A |
| `attrs_intersect`（团队过滤） | `$04C4,X & $07E8,Y ≠ 0`（`bank14:$D78B` 内） | A |
| `hit_resolves` = 几何 ∧ 属性 | 同行判定（几何重叠≠命中） | A |
| `clear_hit_entry`（清 attr/top/bottom，一框一伤） | 命中收尾 `$D7DA` 三写 `$07E8/$07A9/$07BE:=0` | A |
| `damage_apply` + `HitDmgPath::{Const2,Field0630,Zp12}` | `bank14:$D674` 三路（`$D684 SBC #2`/`$D69C SBC $0630,Y`/`$D705`）；5e45dc5 全 SBC 穷举 | A（模型）/B（路选择） |
| `Midboss::hit_shot/hit_body`（HP=14，弹 −2/物撞 −对方 `$0630`） | `arena-midboss.md` §3.2/§3.1（f9930 HP=14→f10080 12 = 一发命中） | A |
| `enemy_slot_alloc` / `shot_slot_alloc` / 槽交叠裁定（24=20+12−8） | `bank2:$888D` / `bank3:$AC42-$AC63`（`shot-slot-geometry.md`、`arena-midboss.md` §1.1） | A |

> 框=注册面、判定=阅读端：二者消费**同一张框表**（21 条/帧，`$74` 索引）；`shot.rs` 注释"模型框≠游戏命中框（W-tas 注）"应读作"注册框本身不等于命中，须经 `$D78B` 4 边+属性判定"——实现已含两者，语义无误。

### 1.2 `engine::combat_frame` 实际可达性（本审计新增核对，A 级代码事实）

`engine.rs` `stage_tick` 帧序 = `wave_tick → behavior_frame → combat_frame → main_loop_step`（L97-108），`combat_frame` 已接线（L209-286）。但：

| # | 事实（代码指针） | 影响（接入判定） |
|---|---|---|
| C1 | `shape_data` **仅测试代码填充**（`crates/game-core/tests/engine_tick.rs:288/317/328`），生产侧（`crates/game-shell/src/host.rs`）从不设置 ⇒ `combat_frame` 的 `state.shape_data.iter().find(...)` 恒 `None` ⇒ 敌框集空 ⇒ **命中判定在 host 内不可达** | 必须由装载侧喂 `shape_data`（数据源 = `shape_stream_addr($0518)` → `$97BA/$9812` 表 → ROM bank2 窗流字节；`engine_tick.rs:283` 用 `r6(a)=0x10+2*0x2000+(a-0x8000)` 即 bank2 窗） |
| C2 | 玩家弹**从未生成**：全仓无生产代码调用 `shot::spawn_init`/`schedule_step`/`shot_slot_alloc`（仅 tests/L2/L3 用例调用；`host.rs` 只读 `bullets.state` 计数）⇒ `live_bullets` 恒 0 | task-37 核心接线点 |
| C3 | 弹推进只做 LUT：`combat_frame` 逐帧 `lut_accumulate_axis`（L215-225），**未走 `shot::tick`**（缺 lifetime 递减、脚本机取指/跳转、寿命归零死亡） | 弹不会按寿命消亡/换向；须改用 `shot::tick` 或补齐同语义 |
| C4 | **未过滤敌弹**：敌弹 `type_code` bit7=1（`ENEMY_SHOT_TYPE=$80`），`combat_frame` 对全部非空弹槽统一注册 `attr=$30` 并判定打敌 ⇒ 敌弹存在时会出现"敌弹打敌"友伤（当前敌弹也不生成，属潜在误判） | 必须按 `PLAYER_ONLY_SLOTS ∪ SHARED_SLOTS` / `type_code & 0x80==0` 过滤 |
| C5 | 敌 HP 直接用 `grid.slots[si].f630`；但 `f630` 对**组长**=组员剩余数（`land_leader` 装载即减一，`engine.rs:485-487/196-199`），对 **Pair 落地=0** ⇒ 命中即 0 → 一击死（含"未命中"以外的 fail-open 风险） | 敌 HP 语义未定谳（B4/W18）⇒ 只可按 A 级白名单（type `$4A` HP=14）施伤，其余 fail-closed |
| C6 | 死亡门简化为 `slots[si]=default()`（L277-280 注释"爆炸/释放链未证 fail-closed"） | 与 `$D616`+`$D86B` 状态转移/死亡段有差；可接"事件登记"，不接爆炸/子代/槽扫描结算 |
| C7 | 无玩家被击路径：阅读端 B `bank14:$D7ED-$D846`（"我打到了玩家"）未接线；无 body collision（物撞 `Field0630` 路径仅 `Midboss::hit_body` 单元面） | task-37 不得接玩家受伤；STG-01"可玩"若要求敌我伤害闭环则本卡阻塞（见 §5） |

### 1.3 测试覆盖（现有，构造级）

- `crates/game-core/tests/engine_tick.rs` ⑫ `combat_frame_hit_chain`（L279-338）：中 boss 形 `$27` 三态——①必中 14→12 + 弹回收；②7 发致死槽回收；③远弹零伤；④属性 `$0F` 不与 `$30` 相交零伤 + 未命中弹不回收。
- `crates/game-core/tests/enemy_behavior.rs` L794-812：`boss_behavior_step` 节拍/死亡门单测；L146 `state_transition(0x4A)=0x4A`、L219-220（`0x5F→0x17`）。
- 上述均为**构造状态**（`shape_data` 人工喂入），不等于 trace 样本。

---

## 2. H-16：已证面 / 残项 / 样本盘 / 接入建议

### 2.1 已证面（A 级，直接支撑 task-37）

1. **统一框表**（`enemy-closeout-w15.md` §1.1/1.2；`arena-midboss.md` §2.1）：`$0780` 左/`$0795` 右/`$07AA` 上/`$07BF` 下/`$07D4` 归属/`$07E9` 属性，21 条/帧（`CPY #$15`），弹/敌/玩家同表。
2. **弹框 7×10**：`bank2:$8847-$888A`（A）。
3. **敌框**：`$0518` 形索引 → `$97BA/$9812` 流指针表 → 流 `[子框数][4B/框 有符号偏移]`，`$0534` 翻转，终值=对象 `$04E0/$04FC`+偏移（`bank14:$D511-$D5AE`，A；`enemy.rs::fetch_shape_boxes` 已实现）。
4. **双阅读端**（`bank14:$D78B-$D7EC` A 端 / `$D7ED-$D846` B 端，A）：A 端 X=被击者（受损方，`$D674` 减伤写点作用于 X——复核 #80 F8 更正），扫描 21 条 4 边+`$04C4,X & $07E8,Y`，命中清被中条目框 `$D7DA` 一框一伤、`STY $6D`、SEC；B 端同 4 边+属性但 `$07D3,Y≠0` 跳过 ⇒ **只碰玩家框（归属 0）**。
5. **伤害/死亡链**（`arena-midboss.md` §2.2，A）：`$D62F` 命中结算（弹归属 bit7 ⇒ `$87FD` 回收；对象 ⇒ `$0668|=#$10` + `$D86B` 状态转移）；`$D674` 伤害（弹 −2 / 物撞 −`$0630` / `$4C==0` 一击死）；HP 0 下溢 → 死亡门 `$D84F` 族。
6. **命中触发清除链**（`enemy-system.md` R-enemy-hit-1，A）：接触帧 `$05DC+n` 归零 → 类型清 0 → 槽释放（f1404 写集基准 + 12 样本）。

### 2.2 残项（未证，必须 fail-closed）

| # | 残项 | 现有证据 | 级 | 对 task-37 的影响 |
|---|---|---|---|---|
| W15 | **玩家被击最上游调用点**未定位 | 阅读端 B 本体 A 级；但"谁在何处调用 `$D7ED`（敌/弹 tick 尾调用点族，`arena-midboss.md` §2.2 记 bank2/bank4 族）"未逐点枚举；`hit-sampling-h16.md` §5 记"w15 触发例程静态反查未定位" | C→B | **玩家受伤不可接** |
| W15′ | **玩家框注册点**未定位 | `arena-midboss.md` §6 W15′："玩家框注册（同表哪条？专属条目？）+ 玩家框常数（样本包络 \|dx\|≤12,\|dy\|≤12）" | B | 同上；玩家判定盒常数仅有 2+1 样本包络（见 §2.3），不是定谳常数 |
| W18 | **`$05DC` 语义双读**：小怪 HP 计数 vs 死亡倒数 | `hit-sampling-h16.md` §1/§5（respawn-window 55 次自然递减倾向倒数读；HP 初值 4 不排除）；中 boss `$05F7($05DC+27)` 恒 0 = **非 HP**（A 反证，`arena-midboss.md` §3.1） | B（小怪） | 小怪 HP/伤害/死亡率不可判定 |
| B4 | **普通小怪 HP 字段未定谳**（四候选） | `completeness.md` §2.4.12 备注："HP 字段四候选未定谳（B4）"；中 boss 已定 `$0630+27`=14（A，特例） | B | 同上；`f630` 直接当 HP 是**未证混用** |
| $4C | 伤害模式旗 `$D67C`（非 0 走减伤公式，0=一击死）语义/写点 | `arena-midboss.md` §6：C→B；`shot.rs` 三路模型已参数化 | C→B | 不能排除"某些被击者 `$4C==0` 一击死" |
| W-tas | 弹路 `Const2` vs `Field0630` 的**逐型选择**（`$6E` bit7 开关 + `auto_hit_path`） | `shot.rs` `auto_hit_path`/`weapon_hit_flags` 注释：机制 A 级、武器→标志映射 B 级待矩阵 | B | 玩家弹默认 `Const2`（弹路，A）可接；**敌物撞/穿透路不可接** |

### 2.3 样本盘（命中/未命中，逐条可复现）

**命中侧（已有 golden，A 级载体）**：`oracle/subroutines/hit_samples.jsonl` **13 行** = 10×`shot_enemy` + 2×`player_enemy` + 1×`release_golden`（R-enemy-hit-1 的 L3 部分口径，`oracle/subroutines/README.md` 登记）。→ **"≥10 样本/侧（命中）"达标**：命中侧 12 个接触样本 + 1 条写集基准；但注意其来源是 `analysis/traces/*`（可再生工作区、**不入库**），**只有摘要 golden 可复现**，原始逐帧样本不可直接重放。

**独立复核（本审计新算，入库 trace 可复现）**：`fire-hold.shots.jsonl`（387 帧 / 20 槽）∩ `fire-hold.enemy.jsonl`（136 帧、**8 帧采样**）= **49 共同帧**；按 t66 包络（`|dx|≤9,|dy|≤15`）得 **24 个候选接触对**（同一帧多对），其中
- **4 对**在下一采样帧敌槽消失（命中候选）：`f1432/eslot11`、`f1440/eslot12`、`f1448/eslot13`、`f1560/eslot12`；
- **20 对**敌槽持续存在（未命中候选，但混有多 HP/属性过滤/非敌对象等混杂，见 `slot27 type47 @(128,8)` 族）。

**玩家侧命中候选（独立）**：`gameplay-stage1.enemy.jsonl` + `.player.jsonl` 在死亡窗起点 f1831 前一采样帧 f1824：`slot15 type=78 @(240,169)` vs 玩家 `(243,160)` ⇒ `dx=−3,dy=+9`（与 `hit-sampling-h16.md` E1 `f1830 slot8 type78 @(240,148) vs (243,160), dx=3,dy=12` 同型同带）；另两死亡窗（f2047/f2131）在 8 帧采样下 **无近距对象** ⇒ 采样漏失。

**未命中侧结论：样本不足（如实）**。理由：①入库 golden 无"未命中"样本；②入库 trace 的未命中候选无法与"多 HP 未死/属性过滤/自然退场/非敌条目"区分；③判定 $05DC/HP 的字段不在 trace watch 列（`$064B` 等仅 analysis 逐帧有、已不入库）。→ 需要 1 帧粒度 trace（至少 `$0400/$04E0/$04FC/$05DC` 列）或 Mesen2 重采，当前**不能**声称"≥10 未命中样本"。

**判定盒常数现状**：弹框 7×10（A）；敌框=形表逐型（A）；**玩家框常数未定谳**（仅包络 |dx|≤12、|dy|≤12，2+1 样本，B/C）。

### 2.4 task-37 可执行接入建议（可接 / 必须 fail-closed）

| 判定面 | 建议 | 依据 |
|---|---|---|
| 玩家弹生成/运动/寿命 | **可接**：`shot::spawn_init + tick`（含 lifetime/script_fetch/LUT）+ `shot_slot_alloc`；射速阈值用已证周期表（`$074A`，`shot-system.md` R-shot-gen-*） | A（`shot-system.md`；L2 15 例） |
| 弹注册 + 敌框 + 4 边/属性判定 | **可接**：`shot_hitbox_register`+`fetch_shape_boxes`+`hit_resolves`；`shape_data` 由装载侧喂 ROM bank2 窗（按 `$97BA/$9812` 表） | A（§1.1） |
| 命中后清框/弹回收 | **可接**：`clear_hit_entry` + `bullets.release`（一框一伤、同帧不回打） | A（`$D7DA`/`$87FD`） |
| 伤害 | **受限可接**：`type==$4A`（中 boss）用 `damage_apply(Const2)` + HP=14 白名单；**其他敌型 fail-closed（0 伤害+登记事件）**，直到 HP 字段（B4）与 `$4C` 定谳 | A（中 boss）/未证（其余） |
| 槽回收/死亡表现 | **只登记事件，不做表现**：可"HP 归 0 ⇒ 槽释放"（=构造测试语义），但**不得**接爆炸子代/`$D86B` 状态转移/得分（未证）；`$D616` 与 `$D86B` 行为差异登记 | 未证（fail-closed） |
| 玩家被击/敌弹打玩家/物撞 | **不可接**（W15/W15′/W18 未证） | §2.2 |
| 敌弹友伤防护 | 必须过滤 `type_code & 0x80` / 槽位集 | C4（§1.2） |
| FrameView | 只暴露已证：玩家弹坐标/活弹数/命中事件（事件=槽释放 + 敌型 + 帧号）；敌槽 HP/型语义不暴露 | 同 plan-playable-loop §6 口径 |

**验收建议（供 task-37 自检）**：①同输入两次 selftest 签名一致；②命中正例/负例各 ≥2（远弹零伤、属性不交零伤、敌弹不参与）；③`type $4A` 7 发致死（14→0）与 `engine_tick.rs` ⑫ 同口径；④未证类型命中必须产生"fail-closed 登记"而非静默零伤或一击死。

---

## 3. 敌坐标写者链

### 3.1 静态写者全集（A 级：seed 逐条扫描，可复算）

| 数组 | 写点总数 | 分 bank | workset 档位分布 |
|---|---|---|---|
| `$04E0`（x 坐标） | **69** | bank00:1 / 02:1 / 03:10 / 04:3 / 05:11 / 06:13 / 07:14 / 14:16 | L2-diff 1 / core-cited 7 / **spec-cited 61** |
| `$04FC`（y 坐标） | **75** | bank00:1 / 02:2 / 03:12 / 04:3 / 05:7 / 06:12 / 07:14 / 14:24 | L2-diff 2 / core-cited 5 / **spec-cited 68** |
| `$0470` x 亚像素 | 14 | b00:4 b03:2 b06:1 b14:6 b15:1 | — |
| `$048C` y 亚像素 | 28 | b00:3 b03:2 b05:10 b06:1 b07:4 b14:6 b15:2 | — |
| `$0550/$056C` x 速度对 | 26 / 31 | b03-07 + b14 为主 | — |
| `$0588/$05A4` y 速度对 | 42 / 45 | b03-07 + b14 为主 | — |

写点分布（节选，完整列表见 §6 命令 D1）：`bank00:8674/867E`（生成层 `$865C-$869E`）、`bank02:834C/8359/8376`（jitter handler）、`bank03:A00C/A011/A0B2/A0E5`（玩家钳制，绝对槽 0）、`bank03:A4B6-A9AD` 段 6+7 处、`bank05:8000-8459` 段 6+5 处、`bank06:8497-9FFF` 段 8+10 处、`bank07:A4D6-A772/A930-AB74/ADF4-BFFF` 段、`bank14:C53B-C582/C583-C5CE/C608-C63D/C994-CA31/CA32-CAC8/CD08-CD28` 族、`bank14:D221/D22C/D235/D245`。

> 口径注：seed 为冻结线性扫描投影，个别写点可能含伪命中（未逐条复核）；但"**生成层/行为 handler 承担绝大多数写入**"的分布结论不受影响。workset 的 core-cited/spec-cited 是"引用扫描"档位（见 [rewrite-completion-audit.md](rewrite-completion-audit.md) §1.1），此处只用于快速定位"已接线/未接线"区段。

### 3.2 Rust 已实现写者 vs 未实现写者（坐标恒 0 的根因）

| 写者族 | ROM 写点 | Rust 实现 | 状态 |
|---|---|---|---|
| 玩家钳制 | `bank03:A00C/A011/A0B2/A0E5` | `player.rs`（host 直接写玩家状态，不写 `ObjectSlot`） | 已实现（玩家面） |
| RNG jitter / 恒速下沉 handler 族 | `bank02:834C`（x）、`8359/8376`（y）（`$8337`/`$836E` 体） | `engine::run_handler` 0x8337（`slot.x/y ±` 经 `rng_jitter_offsets`）+ 0x836E（`handler_const_descent` y+1） | 已实现 |
| 运动更新 | `bank14:CCEB/CCF4`（y）、`CD1C/CD25`（x）（`$CCD3/$CD08` 族） | `engine::motion_update`（亚像素+速度对，L982-1002） | 已实现 |
| 生成层九字段 | `bank00:8674/867E`（`$865C-$869E`） | **无**（`land_pair/land_leader` 只写 type/参数，不写 x/y） | **缺** |
| banks4-7 行为 handler 体 | bank04:3+3 / bank05:11+7 / bank06:13+12 / bank07:14+14 写点（含 boss/midboss 族） | `run_handler` 白名单仅 8 目标（0x836E/0x8314/0x8334/0x837E/0x8403/0x8337/0x90AB/0x9120）；banks4-7 handler 仅有**形态分类表**（`BANK4/5/6/7_HANDLERS`）与单测 | **缺** |
| bank14 子对象生成/公共族 | `bank14:C3EF/C42C/C56B…/C626/C635` 等 | 无 | **缺** |
| boss/midboss 伤/坐标 helper | `bank14:D221/D22C/D235/D245` 等 | 无 | **缺** |

**精确计数**：x 写点 ObjectSlot 面已实现 **3**（bank02 handler 族 1 + motion 2），未实现 **66/69**；y 写点已实现 **4**（bank02 handler 族 2 + motion 2），未实现 **71/75**（玩家面 2+2 另计，不进 `ObjectSlot`）。⇒ host 内 `grid.slots[i].x/y` 除上述 handler/motion 触及者外恒 0；且 **motion_update 的速度字段 `$0550/$056C/$0588/$5A4` 也由未实现 handler 写入** ⇒ 实际上连 motion 也不会动。

**结论**：坐标不可见化不是单点 bug，而是"**生成层 + 行为 handler 层两段写者链未接线**"。`land_*` 只落了"谁（type）/参数"，没落"在哪（x/y）"。

### 3.3 W12 双 SoA 桥现状（引用权威结论）

`completeness.md` §7.1 W 表（L647）：**「~~W12~~ 前提证伪收口（t64：VM 输出不进 `$04xx`——BG 搬运桥 `$85A0` 定谳，N-0080/0082）」**；v0.19 记录同条（v0.19：R-enemy-2 "行为脚本流"→"BG/地形脚本机"；三层 RAM 分立模型）。
⇒ **W12 不是"待接的桥"，而是已被证伪的旧假设**。坐标的真实写者是 §3.1 的生成层 + 行为 handler 层，**不是** terrain VM。task-38 不再沿用"W12 桥"措辞（原任务卡表述据此更正；见 §7）。

### 3.4 oracle enemy trace 坐标列对照：可行性

- **schema**（`oracle/traces/README.md` 及文件实测）：`{"f","state","x","y"}`，`state/x/y` 各为 **28 槽 × 1 字节 hex**（`$0400+n/$04E0+n/$04FC+n`）；**8 帧采样**（f1264 起、Δ8）。
- 入库覆盖：`gameplay-stage1.enemy.jsonl` 143 帧（f1264-2400）；`fire-hold.enemy.jsonl` 136 帧（f1264-2344）；`weapon-pickup.enemy.jsonl` 136 帧。
- **已可对拍**：spawn **型值集合/槽序列**（plan-playable-loop §10 C4 已做：engine 21 事件（槽≥8：20 事件/20 槽/3 型）vs oracle 首现 16 槽/7 型，共同槽 16/16，engine 型集 ⊆ oracle 型集）。
- **当前不可对拍**：坐标列——①Rust `StageProbe` 只暴露 `slot_types[28]`（`engine.rs:924-965`），**无 slot x/y**；②`FrameView.enemy_slots[].position` v1=`None`（plan-playable-loop §6 显式声明"坐标写者链未证，不得把 None 当 (0,0)"）；③slot 身份跨实现不可强绑（分配器近似）。
- 另有 `gameplay-stage1.stage-appear.jsonl`（112 行 spawn 事件 `{f,slot,x,y}`）可作**出生坐标**对照面（多数 x/y=0，记录值来源=生成层，正是 §3.2 缺口）。

### 3.5 坐标可见化最小证据清单（按依赖排序）

1. **生成层坐标写者定谳/接线**（A 级输入已有）：`bank00:$865C-$869E` 九字段（类型=`($9F)[$10&$1F]`、X=`$11<<4`、Y=`$92−$91+$E0`、参数块 `$8D-$92`；`R-stage-tick-4`/`boss-trigger-patrol.md` §1）——先判"STG-01 普通 spawn 走哪条写者"（生成层？handler init？记录装载机 `$874C`？），再接线第一条可复现路径。
   > 已证线索：`land_pair/land_leader` 落地的四参来自调度记录；`$874C-$8754` 记录装载机只写（形/HP/翻转/x-亚）——**x/y 初值来源仍未定谳**，这是本卡最小必答项。
2. **handler 坐标写者族最小实现**：banks5/6/7 各选 1 个代表 handler 形态（速度对/直查/子对象生成）实现其 x/y+速度写；以 `BANK7_HANDLERS`/`BANK5_HANDLERS` 表为索引。
3. **暴露面对拍**：`StageProbe` 增 `slot_x[28]/slot_y[28]`（纯只读镜像，不改语义）或 FrameView 填 `position=Some((x,y))`；加确定性 replay 导出 JSONL。
4. **对拍口径**：共同 8 帧采样帧集（如 f1304-1768 ∩）；槽身份按（type+首现序）对齐而非槽号硬绑；坐标差 ≤1px 记 pass、>1px 记差异分类；帧差=已知 8 帧采样漏失，不做阈值拟合。
5. **接受判据**：首个场景（STG-01 前段，如 f1341-1597 六段之一）坐标列共同帧 ≥20 且 ≤1px 比例 ≥90%，同时 spawn 型值/槽序列不退化（plan §10 C4 基线 16/16）。

---

## 4. Boss / EN-9+

### 4.1 B-STG-01-BOSS 子格现状（`completeness.md` 原格摘要）

- 强度：**L2/部分**（整格维持不升；巡逻子项 t91 已升）。
- ✅ 链定位（STG-1 目录尾指针 `$A16F` → 5B 记录族 ~30 指针 → `$A4xx-$A6xx` VM 字节码脚本族）/ 脚本格式（自环 + `$FC` X 调整）/ 与 EN 族分立（`$B2xx` vs `$A4xx`）/ **出场触发**（R-stage-tick-4：VM 流推进 + `$865C` 九字段生成链，A 级）/ **巡逻语义**（R-boss-patrol-1..3，A 级）。
- ⚠ 记录条数/逐条展开、**W32 实体识别未分**、W30 flow 对齐、W31 子脚本二读。
- ❌ 逐条语义（`$A4xx` 脚本逐条）。
- **B-SYS-BOSS**（最终 Boss 多相，E-W07）= **B / L0**。

### 4.2 STG-01 boss 实体链：静态字节实证（本审计新增，A 级）

`reference/seed` / ROM 直读（file 偏移=0x10 + bank×0x2000 + 窗内偏移）：

| 事实 | 字节证据 |
|---|---|
| 目录尾字段 → `$9F/$A0` | `bank00:802A: B9 06 A0 85 9F B9 07 A0 85`（`LDA $A006,Y / STA $9F / LDA $A007,Y / STA $A0`） |
| 波次流 `$A3CD`（STG-01 record1）内 spawn = **`01 4A`** | file `0x103F2` = `01 4a`（`$A3CD` 流：`F2 00 08 02 00 00 00 E2 A3 00 0A 00 00 00 0D 14 00 00 00 00 00 01 4A FB 05 …`）→ count=1 × type `$4A`（中 boss） |
| 波次流 `$A3E4`（record2）内 spawn = **`01 5F`** | file `0x10409` = `01 5f`（`… 00 00 A5 01 5F 03 11 A4 03 00 00 FE FB A3 …`）→ count=1 × type `$5F`（终盘 boss） |
| 两条波次流均在 STG-01 脚本中被 op5 装载 | `STG01_SCRIPT`：`(0x04C8, op5, $A3CD)`、`(0x04DA, op5, $A3E4)`——但 `engine::process_waves` **只消费 C9/CA/far-call，不解析流内容 spawn**（`engine.rs:721-749`） |
| `$D86B` 表佐证 | `state_transition(0x4A)=0x4A`（恒等，不靠状态表死亡）、`state_transition(0x5F)=0x17`（受击转移候选）——`enemy_behavior.rs:146/219-220`、`wave_record.rs:88/107` |

⇒ STG-01 的 boss 链是 **中 boss `$4A`（HP14/形 `$27` 已 A 级，行为/攻击 B 级未证）→ 终盘 boss `$5F`（HP/行为/攻击全未证）**。`$4A` 的"生成可玩"还缺 §3.2 生成层接线。

### 4.3 Rust 侧 fail-closed 点清单（9 项，全部未接线）

1. `engine::process_waves` 不解析波次流内容（只 C9/CA/far-call）⇒ `$A3CD/$A3E4` 的 `01 4A`/`01 5F` **永不 spawn**。
2. `run_handler` 白名单不含 banks4-7 的任何 handler 目标（`0xA6EC/$ACE5/$AEB0` 等）；boss 类型 → `HandlerResult::Unproven`（字段不动）。
3. `enemy::BossBehavior/boss_behavior_step` 仅测试使用（`enemy_behavior.rs:794-812`），**无生产调用者**（单相直通 + 节拍递减占位，`enemy.rs:1073-1098`）。
4. `stage::boss_segment_unproven()` 显式 `Err("Boss/EN-9+ 段未证…")`（`stage.rs:216-219`）。
5. `stage::WaveSchedule::record_index(n≥8)` 返回 `None`（注释"EN-9+/Boss 段 fail-closed"，`stage.rs:165-171`）。
6. `wave_record::TERMINAL_DROPPERS`（stage1 `$A3E4` types `[0x5F]`）**定义无消费者**（`wave_record.rs:185-188`，全仓仅测试引用）。
7. `TYPE_DISPATCH` 有 `$CA/$CB/$CD` 目标与 bank 切换值（`enemy.rs:864-867`），但仅数据面。
8. `$5F` HP/行为/弹型 = `arena-midboss.md` §4 表"未证（不发明）"。
9. 最终 Boss 多相（E-W07）：`boss-face-design.md` §3 fail-closed——`phase_coeff=None` + 单相直通；相位数/切换判据/伤害系数均待 trace（`boss-chain-survey.md` §4 已把机制 A 级、参数未证分离）。

### 4.4 EN-9+「5B 同格式延续」复核：B 级族级判断，逐条对齐未证

- EN 记录族 `$ADD0-$ADF7` = **8×5B 可逐条解**（本审计重解，字节与 `stage.rs::parse_spawn_records` 一致）：`bf 08 20 e6 b3`→script `$B3E6`；`8b 1a 11 b4 b2`→`$B2B4`；`9b 14 11 b4 b2`→`$B2B4`；`87 16 10 e6 b3`→`$B3E6`；`87 0e 21 35 b2`→`$B235`；`85 13 11 35 b2`→`$B235`；`85 18 21 1d b2`→`$B21D`；`90 12 10 b4 b2`→`$B2B4`。
- **EN-9+ `$ADF8+`** raw：`1d 11 b4 b2 b0 10 10 f3 b3 90 10 11 3f b2 85 1b …`；按 5B 步进首条 ptr=`$B0B2`、次条 `$90F3`（非 `$B2B4`/`$B3xx` 族）；按 4B 步进首条可读作 `{1d,11,ptr=$B2B4}`（恰合 w28 "1D 11 B4 B2 族"）但次条 `$F310` 不成立。
- ⇒ **`$ADF8+` 的记录边界/字段切分未证**；`w28-boss.md` §2.3 的"同格式延续"只能作 **B 级族级判断**（EN 族继续存在），**不能**作"5B 逐条格式已证"。实现前必须先定界（见 §4.5 第 1 步）。

### 4.5 最小取证路径（静态先行、动态受限）

| 步 | 内容 | 产出 / 判定 | 阻塞 |
|---|---|---|---|
| S1 | **BOSS 记录族定界**：`$A16F` 起逐字节；先验 5B 步进只在前 3 条成立（`$A41F`/`$A6CA`/`$A486`），之后失配（`$81A4`）⇒ 变长/区边界待证（W32 邻域） | 记录族边界/条数/字段切分定谳（A/B）；产出 type 值列表 | 是（4.2/4.4 的实装前提） |
| S2 | **`$865C` 生成层九字段**接线与 type 表解析（`($9F)[$10&$1F]` 的索引面） | 生成写集 L2 断言（R-stage-tick-4 已给 A 级字段指针） | 依赖 S1 的记录布局 |
| S3 | **boss 型 init / HP 表**：`$ADED`（初 HP）`$ADE5`（重装）`$ADE9`（节拍周期）$A3B4/$A3BC（行为参对）逐字节（`boss-chain-survey.md` §4 已给机制，表值未全录）；中 boss `$4A` HP14 已有 A 级快照 | 每实体 HP/节拍参数 A 级表 | 相位参数需动态 |
| S4 | **EN-9+ 定界**：从 `$ADF8` 起用 S1 同法定记录边界（先确认是否 5B/4B 混排） | 边界 + 逐条字段 | 是（EN-9+ 实装前提） |
| S5 | **动态 trace（阻塞）**：到 boss 段（中 boss f9930+ 已由 B2 v2-diag arena 采过，但 `analysis/traces/` 可再生成不入库；`tools/scenario_run.py` 依赖 Linux/Mesen2——本机 Windows 不可执行） | boss 出场帧/HP 曲线/相位切换锚 | **样本不足（如实）**：动态锚在干净检出不可得；需 Linux oracle 宿主 |
| S6 | **接入判据**：S1-S4 未出 A/B 级前，Rust 保持 §4.3 九点 fail-closed；出参只允许"Boss 段未证事件"（不画/不动/不伤） | 复核 pass = 未证不发明检查 + fail-closed 测试 | — |

---

## 5. 任务卡建议（供 Lead 派生）

| 卡 | 角色 | 范围 | 完成判据 |
|---|---|---|---|
| task-37 接入细化（现有卡） | porter | 弹生成/运动/命中接线 + FrameView 暴露；§2.4 表 | `shot::tick` 生命周期 + 敌弹过滤 + type `$4A` 白名单施伤 + 未证类型 fail-closed 登记 + selftest ×2 + 命中正/负例各 ≥2 |
| **新卡 A：敌坐标写者链最小接线** | decomp-analyst + porter | §3.5 五步；先定"x/y 初值来源"（生成层 vs handler init vs 记录装载机） | 一条可复现路径实现 + `StageProbe slot_x/y` 暴露 + 与 oracle enemy trace 共同帧 ≤1px ≥90%（≥20 帧） |
| **新卡 B：H-16 玩家判定盒（W15/W15′）** | decomp-analyst | 阅读端 B `$D7ED` 调用点族逐点枚举 + 玩家框注册点反查 + 玩家框常数样本扩采 | 调用点 ≥1 A 级 + 玩家框条目归属定谳 + 判定盒常数（≥10 样本/侧）或显式负结论 |
| **新卡 C：Boss/EN-9+ 记录族定界** | decomp-analyst | §4.5 S1+S4（记录边界/条数/字段切分/type 列表）+ S3 表值 | 记录族 A/B 级定谳 + type 列表 + 每实体 HP/节拍表；**不接行为** |
| **新卡 D：敌 HP 字段定谳（B4/W18）** | decomp-analyst | 小怪 HP 字段四候选 + `$05DC/$05F7`/`$6F4` 三角关系；中 boss 已 A 级作对照 | 小怪 HP 字段/命中减伤 A 级（或显式双读结论）+ L2 用例 ≥3 |
| 顺序建议 | — | A 与 task-37 可并行（无写范围重叠）；B/C/D 取证先行；boss 动态线归 Linux oracle 宿主 | — |

---

## 6. 复现命令（全部只读；`python`，工作目录=仓库根）

**D1 坐标/亚像素/速度写者静态全集（§3.1）**

```bash
python - <<'PY'
import re, collections
LINE=re.compile(r'^bank(\d\d):([0-9A-F]{4})\s+((?:[0-9A-F]{2} )*[0-9A-F]{2})\s+(\S+)\s*(.*)$')
seed=open('reference/seed/gunnac.asm',encoding='utf-8',errors='replace').read()
for base in ('$4E0','$4FC','$470','$48C','$550','$56C','$588','$5A4'):
    hits=[]
    for ln in seed.splitlines():
        m=LINE.match(ln)
        if not m: continue
        bank,addr,mnem,oper=int(m.group(1)),int(m.group(2),16),m.group(4),m.group(5).strip()
        if mnem in ('STA','STX','STY') and oper.upper().startswith(base):
            hits.append((bank,addr,mnem,oper))
    per=collections.Counter(b for b,*_ in hits)
    print(base,'n=%d'%len(hits),{f'b{b:02d}':c for b,c in sorted(per.items())})
PY
# 期望：$4E0 n=69 / $4FC n=75（分 bank 见 §3.1）
```

**D2 STG-01 boss 实体链字节（§4.2）**

```bash
python - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read()
off=lambda b,a:0x10+b*0x2000+(a&0x1FFF)
for b,a,n,tag in [(8,0xA3CD,32,'wave $A3CD'),(8,0xA3E4,48,'wave $A3E4'),(0,0x802A,9,'dir tail')]:
    d=rom[off(b,a):off(b,a)+n]
    print(tag,' '.join('%02x'%x for x in d))
print('file 0x103F2:',rom[0x103F2:0x103F4].hex(' '),'(期望 01 4a)')
print('file 0x10409:',rom[0x10409:0x1040B].hex(' '),'(期望 01 5f)')
PY
```

**D3 BOSS 记录族 / EN / EN-9+ 字节（§4.2/§4.4）**

```bash
python - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read(); off=lambda a:0x10+8*0x2000+(a-0xA000)
for a,n,tag in [(0xA16F,80,'BOSS family $A16F'),(0xADD0,40,'EN 8x5B'),
                (0xADF8,40,'EN-9+ $ADF8+')]:
    d=rom[off(a):off(a)+n]
    print('==',tag)
    for i in range(0,len(d),16):
        print(' %04X: %s'%(a+i,' '.join('%02x'%x for x in d[i:i+16])))
PY
```

**D4 H-16 样本盘（§2.3）**

```bash
# golden 摘要（13 行）
python -c "import json;[print(json.dumps(json.loads(l),ensure_ascii=False)) for l in open('oracle/subroutines/hit_samples.jsonl',encoding='utf-8') if l.strip()]"
# 入库 trace 候选接触对（fire-hold，8 帧采样；输出 24 对 + 4 命中候选）
python - <<'PY'
import json, collections
def enemy(name):
    out={}
    for l in open('oracle/traces/'+name,encoding='utf-8'):
        if not l.strip(): continue
        r=json.loads(l); out[r['f']]=(bytes.fromhex(r['state']),bytes.fromhex(r['x']),bytes.fromhex(r['y']))
    return out
def shots(name):
    out=collections.defaultdict(list)
    for l in open('oracle/traces/'+name,encoding='utf-8'):
        if not l.strip(): continue
        r=json.loads(l)
        if r['lifetime']>0: out[r['f']].append(r)
    return out
en= enemy('fire-hold.enemy.jsonl'); sh=shots('fire-hold.shots.jsonl'); common=sorted(set(en)&set(sh))
print('common frames',len(common)); n=0; gone=0
for f in common:
    st,ex,ey=en[f]; st2=en.get(f+8)
    for r in sh[f]:
        for slot in range(8,28):
            if st[slot]==0: continue
            if abs(r['pos_x']-ex[slot])<=9 and abs(r['pos_y']-ey[slot])<=15:
                n+=1; g=(st2 is not None and st2[0][slot]==0); gone+=g
                print('f=%d eslot=%d type=%d dx=%d dy=%d gone_next8=%s'%(f,slot,st[slot],abs(r['pos_x']-ex[slot]),abs(r['pos_y']-ey[slot]),g))
print('pairs',n,'gone',gone)
PY
```

**D5 代码事实核对（§1.2/§3.2/§4.3）**

```bash
grep -rn "shape_data" crates/                      # 仅 engine.rs + engine_tick.rs 测试（生产未喂）
grep -rn "spawn_init\|schedule_step\|shot_slot_alloc" crates/   # 生产无调用（仅 tests/L2/L3）
grep -rn "boss_behavior_step\|BossBehavior" crates/ # 仅 enemy.rs 定义 + 测试（无生产调用）
grep -rn "TERMINAL_DROPPERS" crates/                # 仅 wave_record.rs 定义 + 测试
grep -n "state.clear\|boss_segment_unproven" crates/game-core/src/stage.rs
grep -n "motion_update\|fn land_pair\|fn land_leader" crates/game-core/src/engine.rs
```

**D6 上轮审计口径（供交叉引用）**：[rewrite-completion-audit.md](rewrite-completion-audit.md) §7 P1（H-16/Boss 行）与 §8 C9。

---

## 7. 不确定项 / 样本不足 / 未做（显式）

1. **未证不发明**：本文所有"已证"均带 ROM/seed/trace 指针；W15/W15′/W18/B4/$4C/EN-9+ 对齐**未证**，未写成结论。
2. **动态样本不足**：`analysis/traces/`（逐帧 samples/golden 的原始载体）不入库、`analysis/ida` 同；`tools/scenario_run.py` `import fcntl`（POSIX-only）本机不可执行；⇒ 逐帧命中/未命中/释放**不可在本机重采**，只能依赖入库 golden 摘要 + 8 帧采样 trace（候选级）。
3. **8 帧采样限制**：enemy trace Δ8 ⇒ 帧级释放/命中不可判定；"命中候选/未命中候选"已在 §2.3 逐条列并标混杂因素。
4. **未重跑** cargo/build；未改 `crates/`、`data/`、`gates/`、`rom/`、`reference/`。写范围仅本文件（未用 `data/naming/`——本轮无 rename/新语义名可登记）。
5. **W12 措辞更正**：任务卡原文"W12 桥/VM $A4 → $04xx"与 t64 已收口结论不符（W12 前提已证伪）；本文按现行结论记（§3.3），不改历史 findings。
6. **B-STG-01-BOSS 记录族**：w28 "5B 记录族"在首 3 条后可解性失配（§4.4），"30 指针"只能作 B 级；区边界/W32 实体识别仍是未证项。
7. **本卡不产出实装结论**：§2.4/§3.5/§4.5 是给 task-37 与后续卡的接入判据，不是实装声明。

---

## 8. 自查与纪律记录

- 只写本文件（`docs/findings/playability-evidence-h16-coords.md`）；未 git add/commit；未动他文件。
- 证据分级：§1.1/§2.1/§2.3/§3.1/§4.2/§4.4 为 A 级字节/trace 可复现；§3.2 计数为 A 级静态扫（个别伪命中未逐条核，已注）；B 级处均显式标注（W15/W15′/W18/B4/$4C/EN-9+ 对齐）。
- 数字可复算：69/75 写点、24 候选对/4 命中候选、13 行 golden、8×5B EN、`01 4A`/`01 5F` 字节——均附命令。
- 对抗复核建议（给复核方）：优先证伪 5 点——(a) D1 写点计数与 §3.2 已实现/未实现划分；(b) D2 `01 4A/01 5F` 定位与 stream 归属；(c) D3 EN-9+ 步进失配与"EN 8×5B 可解"对照；(d) D4 候选对/样本盘结论（尤其"未命中样本不足"是否成立）；(e) §1.2 C1-C7 代码事实（shape_data/无弹/HP 混用/敌弹友伤）。

【2026-10-18 初版】decomp-analyst，HEAD `b92139a`，任务 task-38。
