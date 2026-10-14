# G-S5-6 动态回放战役 —— S5-ALL-CLEAR 链真机动态层·关 1 段（Mesen2 段级 trace）

> 【2026-10-15 更正（复核 #74）】A3：原标题「S5-ALL-CLEAR 链真机动态层（Mesen2 段级 trace）」
> 可读作全链已达；收窄为「…真机动态层**·关 1 段**」——正文自限：关 1 段 12 真锚/44，
> 关 2..8 未跑（spec G-S5-6′ 分层与复核 #72 口径兼容）。原标题留档于此。

> 日期：2026-10-15（campaign G-S5-6）｜ 角色：verifier（oracle 验证工程师）
> 目标：把 S5-ALL-CLEAR 链从**状态机层**（复核 #72 裁定：链驱动仿真）推到**真机动态层**
> （Mesen2 真跑段级 trace），产出**真帧锚**回填 `oracle/traces/s5-all-clear.l5.jsonl` 的
> f=0 占位，并暴露内容缺口（关 2+ 输入脚本可行性）。
> 证据分级依 AGENTS.md §3；**真帧锚只从 trace 帧号来，凡未观测一律保持占位/显式负结果，不发明**。
> 判定物 = `docs/spec/l5-plan.md` §2 + `docs/spec/l5-all-clear.md` R-l5-8（golden 口径）。

## 0. 验收对照（开工声明，未达不结案）

| # | 验收 | 状态 |
|---|---|---|
| ① | 回放方案落盘（渐进式或更优） | ✅ §1（(c) 渐进式 + Run A/B/C 三段实施） |
| ② | 关 1 全程段级 trace（战斗/关末/商店/关 2 开始四段帧锚） | ✅ §2.3/§3——战斗段 f1500（natural）+ 关末段 f2401/2794/3066 + 商店段 f3116/3300/3380/3400 + 关 2 开始 f3404/3450（**trigger=injected**：门位 `$26 bit4` 注入；自然可达性=Run A 负结果显式并列） |
| ③ | golden 回填（真锚数 N 显式） | ✅ §4——**12 真锚 / 32 占位**（~~台账 50 条 1:1 诚实锁测试~~【2026-10-15 更正（复核 #74）】A2：台账 **44 golden 条 1:1 + 6 附加（合计 50 条）**诚实锁测试） |
| ④ | 内容缺口清单（关 2+ 输入脚本可行性结论） | ✅ §5——**负结果：自然通关输入脚本写不出**（G-S5-6a/6b + B1/B2 需求清单） |
| ⑤ | 三件复跑不降 + gates 全绿 | ✅ §6——144/144 + 121/121 + 12/12 + TRACE-DIFF ALL MATCH |

---

## 1. 回放方案设计（验收①）

### 1.1 三选项裁定

| 选项 | 描述 | 裁定 |
|---|---|---|
| (a) 一条长脚本连打 | 8 关输入预写成单场景（数万帧） | **否**：关 2+ 敌群内容未接（B-STG-02..08 L0），输入脚本无法预写；且无快照接力（l5-plan C2），单跑超时风险高 |
| (b) 段级短脚本接力 | 每关段独立 scenario + capture 锚点 | **部分采纳**：作为采集单位；但场景间无 save-state 续跑（C2 未建），接力只能是"同前缀重放 + 延长尾段" |
| (c) **渐进式** | 关 1 全程先行（gameplay-stage1 已有 2400f 战斗段 → 补关末/商店/关 2 开始），产出方法学后批量关 2..8 | **采纳**：每步产物可独立复核；失败面可控（关 2 内容缺口在关 1 方法学落地后才暴露） |

**裁定 = (c) 渐进式**，实施形态：

```
Run A（自然回放）：gs5-stg01-full = gameplay-stage1 输入前缀（逐字节同源，锚 f1080/1200/1260/1500 确定性复现）
                    + f2400 起被动尾段（自动射击+低速摆动，帧预算 15000）
                    → 目标：自然走到关末段（$26 bit4 置位/$0300≠0/INC $180）→ 商店段 → 关 2 开始
Run B（触发回放，仅当 Run A 自然不可达）：同前缀 + 显式触发注入（$CA bit0=1，= 流末记录近端上游，
                    bank00:8CFC A 级读点）→ ROM 自然执行 E2F3 关末块 → E396→E16B→$9BC3 → 关 2
                    （触发来源标注 = 注入；链路动态 = A 级真机执行；自然可达性另册）
```

### 1.2 采集面（长跑性能预算）

gameplay-stage1 重 watch 面（wram 8KB/iram/shot_objs/oam/zp 全量 + probe_mem + capture_every 60）
2400f = 490s（≈4.9 fps）。长跑 15000f 若沿用 ⇒ ~50min+，且 probe_mem 流量大。
**长跑轻量面**（单字节 watch ×14 + capture_every 600）：

| watch | 用途 |
|---|---|
| `0180` | 关号（INC $180 实测锚） |
| `0026` | 关末门 bit4（E2F3）/bit7（G-S5-1 不接线仅观测） |
| `002b`/`002f` | 关末倒数环相位/计数（E328-E342） |
| `00ca` | 关末触发近端上游（bank00:8CFC 读 bit0 → $26 bit4；Run B 注入口） |
| `0300` | 关末推进闸（E366 等待位） |
| `0400` | 玩家态（0x0F 死亡窗 / 0 清零=商店 init AD65-1 / 非零=关内） |
| `018d`/`018e` | 命数 / 商店金钱（$9BC3 init 清零面） |
| `0188` | 关始引入计数（E312 `$188:=$7E` 同族） |
| `041c`/`0438` | 飞离向量写点（E342 `$0438:=$0D`/`$041C:=$A4`） |
| `0018` | controller_held（A5 输入纯净对账） |

截图/RAM dump：capture_every 600 + 检查点旁证（RAM dump 2KB/点）。
帧锚判定全部可由 `.frames.jsonl` 机器重导（§3 逐锚给判定式），截图只作旁证。

### 1.3 确定性与复现口径

- 输入前缀与 `oracle/scenarios/gameplay-stage1.txt` **逐指令同源**（press_len 4 + 同帧号表），
  ⇒ f1080/1200/1260/1500 四锚应在新 run 复现（帧号全等）——作为**确定性对账项**（§2 表）。
- 帧计数约定沿用 README（schedule(N) == trace.held(N+1)）；run.json 的 sha256 为复现证据。
- 复现命令：`python3 tools/scenario_run.py run gs5-stg01-full`（产物 `analysis/traces/gs5-stg01-full.*`）。

---

## 2. Run A：关 1 全程自然回放（结果落盘）

### 2.1 Run A = `gs5-stg01-full`（15000 帧自然盲开环）——**负结果：自然关末未到达**

复现：`python3 tools/scenario_run.py run gs5-stg01-full`（15000f / 260.9s / 57.5fps 轻量面；
`analysis/traces/gs5-stg01-full.run.json`：scenario sha256=`ba11a5a9ad87e300…`）。

| 项 | 实测结果 |
|---|---|
| 确定性对账（f1080/1200/1260/1500） | **✅ 0 差异**：gameplay-stage1 ↔ gs5-stg01-full 共有 7 字段（$0401/$0188/$018E/$18/$26/$0300/$0400）× 2400 帧逐帧全等（0 mismatch）——同源前缀确定性成立，四锚复现 |
| 自然关末到达 | **❌ 未到达**：~~`$CA`（流末记录标志）/`$26 bit4` 全程 0（f1..15000）~~【2026-10-15 更正（复核 #74）】F1：`$CA` 全程 0 成立；「$26 bit4 全程 0」**为假**（实测 f913..1192 置位 280 帧=菜单→开 stage 过渡窗，非关末语义，见表下更正注）→ **关末语义自然触发未观测**（判据=flags27 bit7 入场+倒数环+INC $180 三联）；`$0300` 关末推进闸从未进入关末语义 |
| 死亡/残机动态 | 4 次死亡窗（`$0400=0x0F`，各 128 帧）：**f1787→1915 / f2046→2174 / f4933→5061 / f5181→5309**；残机 `$018D`：起始 2 → 死亡 f1788 减 1 → 死亡 f2047 减 0 → **f3191 加 1**（奖命事件）→ 死亡 f4934 减 0 → 死亡 f5181（0 残机）→ **GAME OVER** |
| GAME OVER 链 | f5182 `$26 bit3`（=0x08）置位 → f5309 `$0400:=0` + `$0188:=255→127` → **f5400 截图「游戏结束」**（score 37850）→ **f6600 截图 HI SCORE/SCORE/**CONTINUE\|GAME OVER** 菜单** → 至 f15000 状态冻结（盲 A 点按不选择；Run B 盲 B 也不选择——菜单导航输入未证，登记缺口 §5） |

> 【2026-10-15 更正（复核 #74）】F1：上表「$26 bit4 全程 0（f1..15000）」为假，原地更正：
> **关末语义自然触发未观测**；`$26 bit4` 自然置位窗 = **f913-1192（菜单→开 stage 过渡窗，280 帧）**——
> 该窗 flags26=0x10 持续、countdown2f=199（非 50→0 倒数环）、无飞离向量/$0300 闸；f1193 起
> bit4 清 + intro_ctr=64 关始引入。同前缀 anchor run 同窗 flags27 bit7=0（flags27 bit7 全 run 仅
> f2401..3403=poke 触发的关末段），五 run 同前缀皆有此窗。**关末语义判据 = flags27 bit7 入场 +
> $2F 50→0 倒数环（每 8f DEC）+ INC $180 三联**：Run A 三联全未出现（倒数环无、`$0180` 恒 1；
> flags27 腿经同前缀 anchor run 同窗旁证）。负结果结论（自然关末未到达、盲开环 4 死→GAME OVER、
> 自然通关输入脚本写不出）不受影响。机器复算：`python3 tools/s5_anchor_rederive.py`（§6①）。

**结论（验收④核心输入）**：盲开环输入（自动射击+摆动，C4 条件输入未建）**无法自然通关关 1**——
存活面撑不过 ~4000 帧战斗（1 死/~1200f、奖命 ~1/3200f，不可持续），且关 1 流末记录
（`$CA` 置位）在 15000 帧预算内从未出现（关长 > 4000 战斗帧，未闭环精确长度）。
**S5-ALL-CLEAR 链的"自然可通关性"= 显式负结果**（B1/B2 需求清单见 §5）。

### 2.2 Run B = `gs5-stg01-clear-inject`（$CA 注入）——**负结果：注入点不消费**

`$CA bit0:=1`（poke_every 1 2400-2500；lua.log `ok=true readback=01`）**未触发 `$26 bit4`**：
`bank00:8CFC`（`$CA&1 → $26|=$10`）的消费点在**波次记录边界处理**（bank00:8Cxx 记录引擎，
`$CA ← ($CC),1` 于记录装载 bank00:8987），非逐帧；注入值悬置（trace `ca=1` 持续至 run 末）无人消费。
**G-S5-2 口径收窄（本战役动态证据）**：`$26 bit4` 置位只发生在**流末记录被记录引擎消费的记录边界**，
不是任意时刻的 `$CA` 状态。Run B 尾段复现 GAME OVER 链（f2956 死亡→f3084 `pstate:=0`/`$0188:=255`）。

### 2.3 Run C = `gs5-stg01-clear-c1` / `gs5-stg01-clear-c2` / `gs5-stg01-clear-anchor`（$26 bit4 门位注入）——关末→商店→关 2 动态链 ✅

复现：`python3 tools/scenario_run.py run gs5-stg01-clear-anchor`（5000f / 94.8s；
scenario sha256=`796d1f590b6dadb7…`）。**三 run 同链帧（确定性见证）**：c1/c2/anchor 的
f1..3116 全 watch 字段 0 差异（~~c1↔c2 全程 0 mismatch；c2↔anchor 全程 0 mismatch~~
【2026-10-15 更正（复核 #74）】F2：c1↔c2 **f1..3200 全 18 字段 0 mismatch**（链帧 f1..3116 ✓），
f3201..5000 = 1800 帧**设计性分歧**（c1 press_every B 试探 vs c2 down 导航，两场景脚本差异仅
f3116 后——非确定性破坏）；c2↔anchor **全 5000 帧 18 字段 0 mismatch** ✓）。

**段级事件时间线（真机执行，R-l5-3/4/6 全链动态复现）**：

| 帧 | 事件 | RAM 判定式（trace 可重导） | 规则对照 |
|---|---|---|---|
| f2400 | **触发注入**（trigger=injected） | poke `$26:=$10`（当时 $26=0x00；E2F3 A 级门位） | G-S5-6 触发来源 |
| f2401 | 关末段入场 | `$27 bit7=0x80`（E309 一次性入场）∧ `$2F:=50`（E312）∧ `$0188:=126`（E312 `$7E`）∧ `$06A0:=2`（E32C 免死） | R-l5-3 §1-2 |
| f2402..2794 | 倒数环 | `$2F` 50→0，每 8 帧 DEC（`$2B&7==0`） | R-l5-3 §3 |
| f2794 | 飞离向量写对 | `$041C:=0xA4(164)` / `$0438:=0x0D(13)`（E342）同帧倒数尽 | R-l5-3 §3 |
| f3066 | `$0300≠0` 闸开 + bit4 清 | `gate300==0x10` ∧ `$26&~$10` 同帧（E366 达成→E36E） | R-l5-4 / G-S5-2 动态时点 |
| f3096 | **`INC $180`（$0180=2 写帧）** | `stage_no` 1→2（E37C；E374 淡出服务窗 30f） | R-l5-4（任务书②「关 2 开始」锚） |
| f3116 | **商店 init（$9BC3）** | `player_state:=0`（AD65-1 `$9CF7/$9CF9`）+ st=2 | R-l5-6（E396→E16B→`$9BC3`） |
| f3300 | 商店主菜单 | 截图「1.武器 2.连射 3.炸弹 4.退」 | R-l5-6（`$9D56` 四入口视觉锚） |
| f3251/3291/3331 | 光标 3×down（`$0400` 1/2/3） | `player_state` 递增（N-0003 ui_select_cursor） | 出店导航 |
| f3370 | A 选「4.退」→ f3380「谢谢」确认页 | 截图 | R-shop-7 出店面（口径收窄见 §5） |
| f3400 | 出店近黑过渡（dark_frac≈0.94） | 截图（与 stg01-exit f1200 dark≈0.99 同口径） | R-shop-7 |
| f3404..3676 | 关始引入序列 | `$0188` 63→0（每 4f）+ `$0400:=1` + 向量复位 | G-S5-7（细粒度素材） |
| f3450 | 「地区-2」屏 | 截图（与 stg01-start f1260「地区-1」同口径） | R-ui-6 关始段 |
| f3900 | **关 2 战斗段** | 截图（自机+敌弹；HUD 8800） | R-l5-7（关 2 内容面首批动态素材） |
| f4030 | 关 2 首死（`$0400=0x0F`）→ f4031 `$26 bit3`（0 残机 GAME OVER 臂） | trace | A2/G-S5-5 素材 |

**C1→C2 出店语义修正（实测）**：C1 用 B 点按出店（R-shop-7「双 B 出店」）**未出店**——
B 在主菜单态=面板转换（f3300 主菜单↔f3900 武器子面板振荡）；实测出店 = **光标 3×down 到
「4.退」+ A**（f3380「谢谢」→ f3400 近黑 → f3450「地区-2」）。R-shop-7 口径收窄见 §5。

### 2.4 确定性对账（同源前缀 + 链帧）

| 对账 | 范围 | 结果 |
|---|---|---|
| gameplay-stage1 ↔ gs5-stg01-full | 7 共有字段 × 2400 帧 | **0 mismatch**（f1080/1200/1260/1500 四锚复现） |
| gs5-stg01-clear-c1 ↔ c2 | ~~全 watch × 5000 帧~~【2026-10-15 更正（复核 #74）】F2：全 18 字段 × f1..3200（链帧 f1..3116） | ~~**0 mismatch**~~ **f1..3200 = 0 mismatch**；f3201..5000 = 1800 帧设计性分歧（c1 B 试探 vs c2 down 导航，脚本差异仅 f3116 后） |
| gs5-stg01-clear-c2 ↔ anchor | 全 watch（18 字段）× 5000 帧 | **0 mismatch** ✓ |

> 【2026-10-15 更正（复核 #74）】F2 附注：gameplay-stage1 ↔ full 对账须按其场景 watch 映射取值：
> `game_action_mode`=$0401、`zp`=$0000-00FF、`iram`=$0300-07FF、`wram`=$6000-7FFF——后者是
> **SRAM 区，不能按 $0000 基址解**（7 字段=$0401/$0188/$018E/$18/$26/$0300/$0400）。
> 三项对账机器复算：`python3 tools/s5_anchor_rederive.py`（§6①）。

## 3. 真帧锚台账（验收③回填源；`oracle/traces/s5-all-clear.anchors.jsonl`）

台账 50 条 = 44 golden 事件（1:1）+ 6 附加动态锚（任务书②指定捕获项）。判定式双类：
**RAM 判定式锚**（机器精确重导）/ **截图锚**（capture 记录+截图文件+RAM 上下文核验）。

> 【2026-10-15 更正（复核 #74）】A2：commit `a75c18e` 措辞「50 条↔golden 1:1 / source_run sha256」
> 过头，原地注记（commit 不可改，以本注为准）：实为 **44 条 golden 1:1**（golden:true 与 golden
> 事件全等锁）**+ 6 附加**（golden:false，不入 golden）；`source_run` 是 run 名，sha256 在
> `analysis/traces/<run>.run.json`。

| 锚 | 帧 | 类 | 判定式 | trigger |
|---|---|---|---|---|
| stg01-clear | 2401 | RAM | `flags27 bit7 ∧ countdown2f==50`（E309+E312） | injected |
| stg02-shop | 3116 | RAM | `stage_no==2 ∧ player_state==0`（$9BC3 init） | injected |
| stg02-exit | 3400 | 截图 | 近黑过渡（dark≈0.94）+ capture `exit-fade4` | injected |
| stg02-start | 3450 | 截图 | 「地区-2」+ capture `stg02-start` | injected |
| stg02-combat | 3900 | 截图 | 战斗段采样（自机+敌弹）+ capture `stg02-combat` | injected |
| gate-26-bit4 | 2400 | RAM | poke 注入帧（自然置位未观测） | injected |
| flyout-vector | 2794 | RAM | `countdown2f==0 ∧ $041C==$A4 ∧ $0438==$0D` | injected |
| gate300-open | 3066 | RAM | `gate300≠0 ∧ $26 bit4 清` | injected |
| inc-180 | 3096 | RAM | `stage_no==2` 首帧 | injected |
| stg02-shop-menu | 3300 | 截图 | 四入口菜单（$9D56 视觉锚） | injected |
| （标题 3 + STG-01 前缀 4） | 180/360/540/1080/1200/1260/1500 | 截图/trace | R-ui-6/7 capture 锚 | **natural** |
| 其余 32 | 0 | — | fail-closed 占位 | placeholder |

**trace 对拍（golden↔真机 trace 段，机器重导）**：逐判定式重导 8/8 全对（RAM 锚 5/5 精确重导、
截图锚 3/3 capture+截图存在核验）——`TRACE-DIFF: ALL MATCH`（复现命令 §6）。
真帧锚总数 **12/44**（7 natural + 5 injected）；占位 32/44。**6 个附加动态锚**（非 golden）补齐
任务书②指定捕获项（$26 bit4 置位帧/$0300≠0 帧/$0180=2 写帧/$9BC3 init 帧/主菜单帧/退出帧）。

## 4. golden 回填（验收③）——12 真锚 / 32 占位

`oracle/traces/s5-all-clear.l5.jsonl` 回填（**scene 归属口径**：真锚段并入 `s5-stg01` 场景——
该场景 = 同一输入谱系 run 族（gameplay-stage1 前缀在 gs5-stg01-full/c1/c2/anchor 全部 0 差异复现）；
占位段保持 `s5-chain` 场景 f=0——帧序单调校验不被占位污染，R-l5-8 口径不变）：

| 事件 | 回填前 | 回填后 | 来源 |
|---|---|---|---|
| stg01-clear | s5-chain f0 | **s5-stg01 f2401** | anchor run（injected） |
| stg02-shop | s5-chain f0 | **s5-stg01 f3116** | anchor run（injected） |
| stg02-exit（Transition） | s5-chain f0 | **s5-stg01 f3400** | anchor run（injected） |
| stg02-start | s5-chain f0 | **s5-stg01 f3450** | anchor run（injected） |
| stg02-combat | s5-chain f0 | **s5-stg01 f3900** | anchor run（injected） |
| 其余 32（stg02-clear 起…all-clear-95f9） | f0 | **f0 占位保持** | fail-closed 不发明 |

`crates/game-oracle/src/l5.rs` `s5_chain_run_for` 同步（anchor 查表）；**诚实锁测试**（并入
`l5_s5.rs` ①）：台账 ↔ golden 1:1 全等 + trigger 分级约束（f=0 ⇔ placeholder；真锚必带
source_run+判定式）+ 显式计数（12 真/32 占位）。**分级诚实**：injected 锚一律不冒充自然通关——
Run A 自然可达性负结果与回填并列声明。

> 【2026-10-15 更正（复核 #74）】F4：golden 层显式性注记：**分级在台账层 + 锁兜底**——
> golden（`s5-all-clear.l5.jsonl`）侧不补 trigger/占位显式字段（一行不可行：需 `L5Event`/
> `parse_l5`/诚实锁三处同步扩展；无锁比对的第二真相源反而引入漂移面）。分级语义单一真相源 =
> 台账 `s5-all-clear.anchors.jsonl` 的 `trigger` 三分（natural/injected/placeholder）+ `l5_s5` ①
> 锁（f=0 ⇔ placeholder 双向断言、真锚必带 source_run+判定式）；golden 单独被消费时
> **帧锚语义以台账为准**（R-l5-8 口径）。

## 5. 内容缺口清单（验收④——关 2+ 输入脚本可行性结论）

**结论：关 2+（及关 1 尾段）的"自然通关输入脚本"在当前取证面下写不出来——显式负结果。**

| # | 缺口 | 实测依据 | 需求清单（B1/B2） |
|---|---|---|---|
| G-S5-6a | **自然可通关性未达** | Run A：盲开环 15000f → 4 死（~1200f/死）+1 奖命 → GAME OVER（f5182）；~~`$CA`/`$26 bit4` 全程 0~~【2026-10-15 更正（复核 #74）】F1 同错复读处：`$CA` 全程 0 成立；`$26 bit4` 全程 0 为假（实测 f913-1192 置位 280 帧=菜单→开 stage 过渡窗，非关末语义）→ **关末语义三联自然触发未观测**（§2.1 更正注）（关 1 流末记录 > 4000 战斗帧，长度未闭环） | **B1**：STG-01..08 敌群/弹幕时间线取证（波次链逐时刻对齐，stage-timeline S1 假设定谳）→ 才能生成通关输入轨；**B2**：条件输入 DSL（l5-plan C4）或 TAS 工具辅助输入生成 + save-state 接力（C2）+ RNG 冻结（C1） |
| G-S5-6b | 关 2+ 战斗段输入脚本不可预写 | 关 2 内容面仅 f3900 一帧采样（敌弹形态首批）；敌群调度未取证（B-STG-02..08 L0） | 同 B1；建议先行 t-S2 逐关场景（每关独立短场景，fight 段开环采集） |
| G-S5-2′ | **`$CA` 注入不触发 `$26 bit4`（口径收窄）** | Run B：`$CA:=1` 悬置无人消费；8CFC 消费点=波次记录边界处理（记录引擎），非逐帧 | 流末记录字节语义（`($CC),1` bit0）定谳 + 记录边界帧定位（`$CC/$C8` 游标 watch） |
| G-S5-2″ | `$0300` 推进闸动态时点已观测（f3066） | bank11 写点归属仍 B 级（$82C1/$833F 调用图未展开） | bank11 飞离/关末动画域逐指令展开 |
| G-SHOP-1 | **R-shop-7 口径收窄**：「双 B 出店」仅信息页入口态 | C1：主菜单态 B=面板转换振荡（f3300↔f3900）；实测出店=光标 4.退+A（f3370→f3380 谢谢→f3400 近黑） | spec R-shop-7 原地更正（见 l5-all-clear §3 增行） |
| G-SYS-UI-1 | GAME OVER/CONTINUE 菜单导航未证 | Run A：盲 A（15000f）/Run B：盲 B 均不选择（状态冻结 f5309-15000 / f3085-6000）；f5400「游戏结束」/f6600「CONTINUE\|GAME OVER」两屏实存 | Start/up/down+A 组合探针场景（B-SYS-UI 残项 → A2 game-over-continue 前置） |
| G-SYS-UI-2 | 奖命事件语义未定位 | Run A f3191 `$018D` 0→1（score 37850 区间；与射击得分相关——C2 无尾段射击未触发） | 计分表/奖命阈值取证（score domain） |
| G-S5-5′ | 死亡窗与关末门并发 | 本次关末段全程 `$06A0:=2` 免死（E32C）未触并发 | 保持 fail-closed（G-S5-5 不变） |

## 6. 对拍与三件复跑（验收⑤）——实跑留痕

```
# ① trace 对拍（golden/台账 ↔ 真机 trace 段；逐判定式机器重导）：
#    复现命令（入库，2026-10-15 更正（复核 #74）F3：原「复跑见 git 记录」非命令，补入库入口）：
python3 tools/s5_anchor_rederive.py
#    → RAM 锚 5/5 精确重导（stg01-clear f2401 / flyout f2794 / gate300 f3066 / inc180 f3096 /
#      shop f3116）+ 截图锚 3/3 capture+截图核验（exit f3400 / start f3450 / combat f3900）
#      ⇒ TRACE-DIFF: ALL MATCH (8/8)；同脚本并复算 F1/F2 更正口径（合计 15/15）
# ② diff_l5（golden↔驱动）+ 合法性真闸 + 诚实锁：
cargo test -p game-oracle --test l5_s5 -- --nocapture   → 4 passed（① 含台账 1:1 诚实锁）
# ③ 三件复跑（全量）：
cargo test --workspace                                → 144 passed / 0 failed（基线 144 不降）
python3 tools/diff_subroutines.py                     → L2-DIFF: 121/121 通过
python3 gates/cli.py check                            → SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0
```

纪律自查：真帧锚非发明（逐锚判定式+证据指针+机器重导全对）；injected 锚显式分级不冒充自然
通关（Run A 负结果并列）；占位 32 条 fail-closed 未动；合法性真闸未放宽（l5_legality/边集
零改动，仅帧锚数据回填+台账锁测试加严）；oracle 侧仅**增** `poke/poke_every` DSL（写注入，
README 纪律注：禁用于迁就考题），既有场景/阈值/门禁零改动。


