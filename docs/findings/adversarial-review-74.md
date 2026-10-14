# 对抗复核 #74 — G-S5-6 动态回放战役（真帧锚非发明 / golden 回填诚实性 / 负结果完备）

> 独立复审（fresh context，证伪立场）。被审对象：`e5ce426`（poke DSL）/ `319732d`（五场景）/
> `a75c18e`（golden+台账）/ `f5bce09`（findings+spec）。
> 方法：不采信被审自述、不复用其结论——逐锚从 `analysis/traces/*.frames.jsonl` 原始 trace 独立
> 重导判定式、截图/capture 逐一视觉核验、确定性对账独立复算、mutation 5 项实测、对拍三件独立
> 复跑贴数。每项给出实测输出。
>
> **Verdict: needs_revision**（中偏轻——真帧锚非发明、诚实锁、负结果三条主线经独立重导全部
> 实证成立，无伪造自然锚/无占位污染/无门禁弱化；但 **2 项证据表述与实测不符（M 级，须修）**：
> ① Run A 判定式「$26 bit4 全程 0」为假（实测 f913..1192 置位 280 帧，且同错复读入台账
> gate-26-bit4 判定式）；② 确定性对账「c1↔c2 全 watch × 5000 帧 0 mismatch」为假（f3201 起
> 1800 帧设计性分歧，链帧段 0 差异为真）。另 4 项低危 + 3 项 advisory。负结果结论本身不变。

---

## 0. 对拍独立复跑（贴数，未采信自述）

| 复跑项 | 命令 | 实测输出 | 判定 |
|---|---|---|---|
| workspace 测试 | `cargo test --workspace` | **144 passed / 0 failed** | ✅ 与自述一致 |
| L2 子程序级 | `python3 tools/diff_subroutines.py` | **L2-DIFF: 121/121 通过（golden==ROM 执行==game-core）** | ✅ |
| 门禁总闸 | `python3 gates/cli.py check` | **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0**（trace-diff/visual-diff 两闸实跑亦绿，无静默跳过） | ✅ |
| 锚锁测试 | `cargo test -p game-oracle --test l5_s5` | **4 passed**（①含台账 1:1 诚实锁） | ✅ |
| 确定性对账（独立复算，非对账脚本） | 逐帧 JSONL 比对 | gameplay-stage1↔gs5-stg01-full **7 字段×f1..2400 = 0 mismatch** ✅；c2↔anchor **全 5000 帧 0 mismatch** ✅；c1↔c2 **f1..3200 = 0 mismatch，f3201..5000 = 1800 帧分歧**（见 F2） | ⚠️ 一项自述过界 |
| TRACE-DIFF（战役锚重导） | 无入库命令（见 F3） | 本人独立重导：RAM 锚 **5/5** + 截图锚 **3/3** 全对 | ⚠️ 实质成立，复现命令缺落地 |

> 注：gameplay-stage1 对账须按其场景 watch 映射取值（`game_action_mode`=$0401、`zp`=$0000-00FF、
> `iram`=$0300-07FF、`wram`=$6000-7FFF——后者是 SRAM 区，不能按 $0000 基址解）。

## 1. 三焦点核验总表

| 焦点 | 核验物 | 判定 | 证据位 |
|---|---|---|---|
| ①a 五 RAM 判定式锚独立重导 | f2401/2794/3066/3096/3116 | **过（5/5 精确重导）** | §2.1 |
| ①b poke 注入锚 f2400 + injected trigger 标注真实性 | 场景 poke 命令 ↔ lua.log readback ↔ 台账 trigger | **过（对账干净）** | §2.2 |
| ①c 截图锚 + 自然锚核验 | f3400/3450/3900/1260/1500/1080/1200/180 | **过（视觉+capture 实存）** | §2.3 |
| ①d 台账↔golden 1:1 锁真咬 | mutation M1-M4 | **过（4/4 咬合）** | §5 |
| ①e trigger 归因锁 | mutation M5 | **★不咬合（A1 缺口）** | §5 |
| ②a 三处口径一致性（findings/commit/golden+spec） | 12 真锚（7+5）/32 占位 | **一致** | §3.1 |
| ②b "不冒充自然通关"并列声明 | findings/spec/README/场景头/l5.rs 注 | **过（五处并列）** | §3.2 |
| ②c f=0 占位标记显式性 | golden 隐式 / 台账显式 / 锁断言 | **可接受，显式面不足（F4）** | §3.3 |
| ②d 复核 #72 达成度口径引用 | 对照 adversarial-review-72 | **无越级**（A3 标题措辞） | §3.4 |
| ③a Run A 负结果复算（死亡窗/残机/bit3/GAME OVER） | gs5-stg01-full trace | **全对** | §4.1 |
| ③b Run A「$26 bit4 全程 0」判定式 | 同上 | **★失准（F1）** | §4.1 |
| ③c 关 2+ 输入脚本不可预写（B1/B2 清单） | 缺口论证 | **合理** | §4.2 |
| ③d Run B 口径收窄（$CA 不触发 bit4） | inject trace + 8CFC/E2F3 字节点 | **成立** | §4.3 |
| ④ 对拍贴数 | §0 | **全绿** | §0 |
| ⑤ poke DSL 纪律面（变更范围/README 注/伪造自然锚对账） | git diff 范围 + poke 计数 | **过**（README 日期笔误 F5、inject 头注 stale F6） | §6 |
| ⑥ mutation ≥2 | M1-M5 | **5 实测，4 咬合 + 1 缺口** | §5 |

---

## 2. 焦点①：真帧锚非发明（重点核）

### 2.1 RAM 判定式锚 5/5 独立重导（逐帧重跑，不采信台账判定式文本）

对 `analysis/traces/gs5-stg01-clear-anchor.frames.jsonl`（5000 帧 × 18 watch）逐帧重导：

| 锚 | 台账判定式 | 独立重导实测 | 判定 |
|---|---|---|---|
| stg01-clear f2401 | `flags27 bit7 ∧ countdown2f==50`（E309+E312） | f2400 flags26=0x10（poke 落点）→ **f2401 flags27=0x80 ∧ countdown2f=50** ∧ $0188=126 ∧ $06A0=2（E32C 免死）；f2400 及之前 flags27=0 | ✅ 精确 |
| flyout-vector f2794 | `countdown2f==0 ∧ $041C==$A4 ∧ $0438==$0D` | f2793 countdown2f=1 → **f2794 countdown2f=0 ∧ vec41c=164 ∧ vec438=13**；f2795 vec438 即变 100（判定帧唯一精确）；倒数环 f2402(49)→f2794(0) 共 49 次 DEC | ✅ 精确 |
| gate300-open f3066 | `gate300==0x10 ∧ 同帧 $26 bit4 清` | **f3066 gate300=16 ∧ flags26=0**（f3065 flags26=16/gate300=0）同帧翻转 | ✅ 精确 |
| inc-180 f3096 | `$0180=2 写帧（stage_no 1→2）` | f3095 stage_no=1 → **f3096 stage_no=2** | ✅ 精确 |
| stg02-shop f3116 | `stage_no==2 ∧ player_state==0` | **f3116 stage_no=2 ∧ player_state=0**（f3115 ps=1） | ✅ 精确 |

附加动态锚同批核过：出店导航 f3251/3291/3331 player_state 1/2/3（3×down，held=0x04 对帧）✓；
f3370 A 点按（held=0x80 @f3371）→ f3404 intro_ctr=63/ps=1 关始引入 ✓——出店语义
（光标 3×down+「4.退」+A，R-shop-7 收窄）实测成立。

### 2.2 injected trigger 标注真实性（对照场景 poke 命令与帧号）

- 台账 11 条 `trigger=injected` **全部** `source_run=gs5-stg01-clear-anchor`；该场景唯一注入 =
  `poke 2400 0026 10`（`analysis/traces/gs5-stg01-clear-anchor.lua.log`：`scenario … pokes=1` /
  `poke f2400 $0026:=10 ok=true readback=10`）→ f2400 flags26=16 trace 落点吻合。
- 台账 7 条 `trigger=natural` 的 source_run = title-menu（3）/ gameplay-stage1（4）；两场景
  **0 条 poke 命令**（grep 计数 0/0），capture/截图证据齐（f180 boot-black capture、
  f1080/1200/1260/1500 shots 与台账 evidence 指针逐一同名）。
- Run B（`poke_every 1 2400-2500 00ca 01`，101 次 poke）= 负结果 run，**零锚入库** ✓
  （注入未生效不回填，fail-closed 正确）。
- **poke↔trigger 对账结论：注入锚只出自含 poke 的 run、自然锚 run 零 poke——无 poke 伪造自然锚。**

### 2.3 截图锚视觉核验（capture 记录 + PNG 实看）

| 锚 | 截图实看 | 判定 |
|---|---|---|
| stg02-exit f3400（injected） | 全屏近黑（fade4；capture `exit-fade4`，fade 序列 f3385-3400 齐） | ✅ |
| stg02-start f3450（injected） | **「地区-2」**+自机+HUD 8200 | ✅ |
| stg02-combat f3900（injected） | 自机+敌弹（橙色螺旋弹）+HUD 8800 | ✅ |
| stg01-start f1260（natural） | **「地区-1」**——与 f3450 同口径采样成立 | ✅ |
| stg01-shop/exit/combat f1080/1200/1500（natural） | shots 同名实存（tags 吻合） | ✅ |
| Run A f5400/f6600（负结果截图） | **「游戏结束」score 37850** / **HI SCORE 80000·SCORE 37850·CONTINUE\|GAME OVER** | ✅（score 与 findings 一致） |

**重导计数：11 锚独立重导（≥6 要求达成；injected 8 + natural 3 混合；RAM 判定式 6 + 截图/capture 5）。**

## 3. 焦点②：golden 回填诚实性

### 3.1 三处口径一致性

| 处 | 表述 | 判定 |
|---|---|---|
| findings §0/§3/§4 | 12 真锚（7 natural+5 injected）/ 32 占位；台账 50 = 44 golden（1:1）+ 6 附加 | ✅ 精确 |
| commit a75c18e | 12 真锚（7+5 全标注）/ 32 占位 | ✅（"50 条↔golden 1:1 / source_run sha256" 措辞过头见 A2） |
| spec R-l5-8 增行 | trace 锚 7 + injected 5 + f=0 占位 32 | ✅ |
| golden + 台账机器点数 | golden:true 44 = f>0×12（natural 7+injected 5）+ f=0×32（全 placeholder）；golden:false 6 全 injected | ✅ 全吻合 |

### 3.2 "不冒充自然通关"

findings §2.1/§4、spec G-S5-6′、`oracle/scenarios/README.md` 纪律注、anchor/c1/c2/inject 四场景
头注、`l5.rs` s5_chain_run_for 注释——**五处并列声明 trigger=注入、自然可达性=负结果**；
golden 回填段（§4）与 spec 均注明真锚段并入 s5-stg01 的 scene 归属口径。未发现将注入锚表述为
自然通关之处。Run A 负结果与回填并列声明 ✓。

### 3.3 f=0 占位 32 条的标记

- 台账层：显式 `trigger="placeholder"` + `predicate="未取证（fail-closed 占位，不发明帧号）"` + `source_run=null`；
- golden 层：**仅隐式**（f=0 + scene=s5-chain），无显式占位/trigger 字段；
- 锁层：`f=0 ⇔ placeholder` 双向断言（M4 实测咬合，见 §5）。
  → 可接受（R-l5-8 口径 + 台账 + 锁三重兜底），但 golden 单独被消费时占位与自然/注入锚不可区分
  （**F4**：建议 golden 增 trigger 字段或 R-l5-8 写明"帧锚语义以台账为准"）。

### 3.4 复核 #72 达成度口径引用

findings L4 引"#72 裁定：链驱动仿真"与 `adversarial-review-72.md` §4.1 原文
（"链驱动状态机仿真，不是 ROM/Mesen 动态回放"）一致 ✓。#72 给 captain 的达成度口径
（"8 关可通关"不得判满、动态可通关性待 G-S5-6+逐关内容）**未被越级引用**：本战役只声明关 1 段
动态链走通、自然可通关性=负结果、关 2..8 未跑（spec G-S5-6′ 分层与 #72 口径兼容）。
唯一措辞风险=findings 标题"S5-ALL-CLEAR 链真机动态层"可读作全链已达（**A3**，正文自限为关 1 段）。

## 4. 焦点③：负结果完备

### 4.1 Run A（gs5-stg01-full 15000f）独立复算

| 项 | findings 主张 | 独立复算 | 判定 |
|---|---|---|---|
| 死亡窗 ×4（$0400=0x0F 各 128f） | f1787→1915 / f2046→2174 / f4933→5061 / f5181→5309 | (1787,1914)/(2046,2173)/(4933,5060)/(5181,5308) 各 128f（箭头=变更帧 1915/2174/5061/5309，口径自洽） | ✅ |
| 残机 $018D 链 | 2→1(f1788)→0(f2047)→+1(f3191)→0(f4934)→0(f5181) | 逐帧全等 | ✅ |
| f5182 $26 bit3=0x08 | 置位 | f5182 flags26=0x08 首帧 | ✅ |
| f5309 链 | $0400:=0 + $0188:=255→127 | f5308 ps=15 → f5309 ps=0/$0188=255 → f5310=127 | ✅ |
| f5400/f6600 截图 | 「游戏结束」(score 37850) / CONTINUE\|GAME OVER | 视觉核验全等（score 37850 精确） | ✅ |
| 状态冻结 | 至 f15000 | f7000=f15000=(ps0/intro127/flags26=0x08) | ✅ |
| $CA 全程 0 | f1..15000 | trig_ca 非零帧数 **0** | ✅ |
| **$26 bit4 全程 0** | f1..15000 | **❌ 假：f913..1192 置位 280 帧（连续）** | **F1** |

**F1 定界**：该窗口=菜单→开 stage 过渡窗（f1193 起 intro_ctr=64 关始引入、bit4 清），
**非关末语义**——同前缀 anchor run 同窗 flags27 bit7=0（flags27 bit7 全 run 仅 f2401..3403=
poke 触发的关末段）、countdown2f=199（非 50→0 倒数环）、无飞离向量/$0300 闸。五 run 同前缀
皆有此窗。**负结果结论（自然关末未到达、盲开环 4 死→GAME OVER、自然通关输入脚本写不出）
不受影响**；但"$26 bit4 全程 0"作为数据断言为假，且同错复读入 `anchors.jsonl` gate-26-bit4
判定式（"自然置位未观测：Run A 15000f $CA/$26 bit4 全程 0"）——判定式须改为
「关末语义（flags27 bit7 入场+倒数环）自然触发未观测；bit4 自然置位=f913-1192 过渡窗（非关末语义）」。

### 4.2 关 2+ 输入脚本不可预写（B1/B2）

关 2 内容面确仅 f3900 一帧采样（视觉核验：有敌弹但调度未取证）；B1=STG-01..08 敌群/弹幕时间线
取证、B2=条件输入 DSL/TAS + save-state 接力（C2）+ RNG 冻结（C1）——与 l5-plan C1-C4 缺口登记
一致，**清单合理且是负结果的正确出口**（不是把负结果写成不可通关断言）。✓

### 4.3 Run B 口径收窄（$CA 注入不触发 $26 bit4）

- inject trace：trig_ca=1 自 f2400 持续至 f6000（悬置无人消费）✓；**f2400 后 bit4 从未置位**
  （全 run bit4 窗仅 (913,1192)，与 Run A 同）✓；尾段 GAME OVER 链 f2956 死亡→f3084
  ps:=0/$0188:=255→f3085=127 ✓ 与 findings 一致。
- 字节点背书：`reference/seed/gunnac.asm` bank00:8CFC `LDA $CA` / 8D02 `LDA $26`、
  bank15:E2F3 `LDA $26` 门均实存 ✓。
- **口径收窄成立**（8CFC 消费点=波次记录边界，非逐帧）。F6：inject 场景头注仍写预期链
  "ROM 自然执行 E2F3 关末块…→关 2"未加证伪更正注（AGENTS.md §3.6 原地标注纪律）。

## 5. mutation 实测（5 项，4 咬合 + 1 缺口）

| # | 篡改 | 预期 | 实测 | 判定 |
|---|---|---|---|---|
| M1 | 删台账 1 条 golden 锚（stg01-clear） | 锁红 | `s5_golden_parses_and_legality_gate` **FAILED**（43≠44 计数断言） | ✅ 咬合 |
| M2 | 台账帧号 f2401→2402 | 锁红 | **FAILED** @l5_s5.rs:47 `[7] golden ↔ 台账 全等` | ✅ 咬合 |
| M3 | golden 帧号 f3116→3117 | 锁红 | **双红**：锁① `[8] 全等` + diff② `s5_chain_runs_and_diffs_golden` FAILED（l5.rs 查表↔golden 也被咬住） | ✅ 双保险 |
| M4 | 占位标记篡改（stg03-shop trigger placeholder→natural，f=0 不动） | 锁红 | **FAILED** @l5_s5.rs:53 `[13] f=0 必须标注 placeholder` | ✅ 咬合 |
| M5 | **trigger 降级**（stg01-clear injected→natural） | 应红 | **4/4 全绿**——锁咬不住归因篡改 | **★缺口（A1）** |

全部篡改已 `git checkout` 还原（工作树净）。

## 6. poke DSL 纪律面

- **变更范围**（`git diff --stat e5ce426^..f5bce09`，14 文件）：既有场景**零改动**（仅新增 5 个
  gs5 场景 + README 增行；gameplay-stage1/title-menu 等未触）✓；`gates/` `data/status/` **零改动**
  ✓；`l5_legality`/`diff_l5`/边集/阈值**零弱化**（diff 无相关行）✓；golden 变更仅 24 行帧锚回填 ✓。
- **README 纪律注在文**：禁用于"让考题变简单"的作弊式改状态、产出锚必须标 trigger=injected ✓
  （日期笔误 2026-01-15 应为 2026-10-15，**F5**）。
- **poke 未伪造自然锚**：§2.2 对账（11 injected ↔ 1 run/1 poke；natural run 零 poke）✓。
- 实现面：inputPolled 时点写 CPU 总线（memType=8）+ readback 日志，poke_every 解析期展开，
  帧界 1..frames 校验；README 另补记既有 `press_every`/`probe_mem` 文档行（非行为变更）。

## 7. findings 清单（四分类路由）

| # | 级 | 分类 | 内容 | 路由 |
|---|---|---|---|---|
| F1 | **M（须修）** | 证据表述失准 | Run A「$26 bit4 全程 0（f1..15000）」为假（f913..1192×280f 置位）；同错入台账 gate-26-bit4 判定式。负结果结论不受影响，但判定式不可重导 | 回 verifier：findings §2.1 + anchors.jsonl 判定式原地更正（§3.6 dated 注），改为"关末语义自然触发未观测"+bit4 过渡窗定界 |
| F2 | **M（须修）** | 证据表述失准 | 「c1↔c2 全 watch × 5000 帧 0 mismatch」为假（f3201..5000 1800 帧设计性分歧：c1 press_every B 试探 vs c2 down 导航）。真口径=f1..3200（链帧 f1..3116）0 差异 + c2↔anchor 全程 0 差异 | 回 verifier：findings §2.3 括注与 §2.4 对账表收窄到共享输入段 |
| F3 | L | 复现性 | "TRACE-DIFF: ALL MATCH" 无入库复现命令（§6①"复跑见 git 记录"非命令；tools/ 无锚重导入口；gates trace-diff 是另一物） | 回 verifier：入库轻量锚重导脚本或 §6 贴可粘贴命令 |
| F4 | L | 显式性 | golden 无 trigger/占位字段，自然/注入/占位在 golden 层不可区分（台账+锁兜底） | 建议 golden 增 trigger 字段或 R-l5-8 写明"帧锚语义以台账为准" |
| F5 | L | 文档 | README poke 纪律注日期笔误 2026-01-15→2026-10-15 | 1 行修 |
| F6 | L | 文档 | gs5-stg01-clear-inject.txt 头注 stale（预期链写成事实，已被 §2.2 负结果推翻，缺 §3.6 更正注） | 原地加 dated 更正注 |
| A1 | advisory | 测试咬合 | M5：trigger 归因降级不被锁咬（另 parse_s5_anchors 不解析 evidence 字段，证据指针无机器校验） | 建议锁加"source_run 场景含 poke ⇒ trigger=injected"（或台账记 scenario sha/poke 数） |
| A2 | advisory | 措辞 | commit a75c18e"50 条↔golden 1:1 / source_run sha256"过头（实为 44 条 1:1+6 附加；source_run 是 run 名，sha256 在 run.json） | 下次 commit 措辞 |
| A3 | advisory | 标题口径 | findings 标题"S5-ALL-CLEAR 链真机动态层"可读作全链已达（正文自限关 1 段 12/44） | 建议标题收窄"关 1 段" |

## 8. Verdict 与修正建议

**Verdict: needs_revision**——修 F1/F2 两处证据表述（含台账判定式同步）即可转 pass；
F3-F6 一并修；A1 建议顺手加锁断言。三条验收主线独立重导结论：

1. **真帧锚非发明：成立**（11 锚独立重导全对；poke↔trigger 对账干净；锁 M1-M4 咬合）。
2. **诚实性：成立**（12/32、7+5 三处口径一致；五处"不冒充自然通关"并列；#72 无越级；
   占位 fail-closed 未动）。
3. **负结果：结论成立**（死亡窗/残机/bit3/GAME OVER 链复算全对，B1/B2 合理；Run B 口径收窄
   成立），但**判定式文本 F1 失准须更正**。

对拍 144/121/12/TRACE-DIFF(独立重导 8/8) 实测全绿；门禁/阈值/边集/既有场景零改动核过；
mutation 4/5 咬合 + 1 归因锁缺口（A1）。
