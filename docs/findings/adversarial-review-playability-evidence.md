# 对抗复核：可玩性取证 H-16 / 坐标 / Boss（task-43）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：decomp-auditor（task-38）
> 被审文件：`docs/findings/playability-evidence-h16-coords.md`（**360 行**，sha256 `6d49e94c…`，mtime 14:23:15；
> 时点 HEAD `b92139a`，审计期声明工作区有并发施工）。
> 复核方式：独立重扫 seed/ROM/trace + 代码 grep/字节重解，不采信自述；本文件为唯一写入，未 `git add`/`git commit`。
> 边界：当前工作区另有 task-37（game-shell）并发 WIP，不在被审文件声明范围；本复核按被审文件声明的 HEAD
> `b92139a` 判断，并对当前 WIP 的影响单独标注。

## 0. VERDICT

**pass**（4 条非阻塞 findings；F1-F2 建议顺手核对）

被审文件的**核心实证与分级成立**：
- H-16 判定链本体（框注册/4 边+属性阅读端/一框一伤/伤害/回收）与 `shot.rs` 实现有 A 级依据；
  **命中侧样本 12 个接触 + 1 写集基准**，**未命中侧明确"样本不足"**（未伪称 ≥10/侧）；接入建议可执行。
- 敌坐标写者链 **x `$04E0` 69 处 / y `$04FC` 75 处**、bank 分布、已实现/未实现划分与"坐标恒 0 根因"可复算；
  W12"前提证伪"与 `completeness.md` §7.1 一致。
- Boss/EN-9+ 的 `01 4A`/`01 5F` 字节、`process_waves` 不解析 spawn、9 个 fail-closed 点、EN-9+ B 级族级判断
  均与代码/ROM 一致；EN-9+ 不得逐条实装的结论正确。
- ≥5 条 ROM/trace/代码指针独立验证通过；未证项（W15/W15′/W18/B4/$4C/EN-9+ 对齐）显式标注，未发明。
- docs-links PASS（当前 332 md/417 links，全部有效）。

## 1. H-16 核验

| 声明 | 独立结果 |
|---|---|
| `shot_hitbox_register` 7×10 框 x−4..x+3 / y−5..y+5、owner=槽偏移、attr=$30 | 代码 `shot.rs:305-316` 完全一致（7/10 为 **span**：right−left=7、bottom−top=10；非像素宽高；建议措辞"span 7×10"避免歧义，F4）✅ |
| 4 边重叠+属性相交 `$D78B-$D846`、一框一伤 `$D7DA`、三路伤害 `$D674`、弹回收 `$87FD` | 代码 `boxes_overlap/attrs_intersect/hit_resolves/clear_hit_entry/damage_apply` 与 seed 指针（bank14:$D78B/$D7DA/$D674、bank02:$87FD 等）均可查 ✅ |
| 命中样本盘：`hit_samples.jsonl` 13 行 =10 shot_enemy+2 player_enemy+1 release | 实读 13 行且 kind 计数 **10/2/1** ✅；未命中侧"样本不足"（无入库 golden、8 帧采样不能区分 HP/属性/自然退场）**如实** ✅ |
| 独立 trace 候选：fire-hold 共同帧 49、候选对 24、next8 消失 4 | 重跑 D4：**shots 唯一帧 387（lifetime>0 为 386）/ enemy 136，共同 49，pairs 24，gone 4** ✅（候选人逐条可复算） |
| 玩家样本 f1824 slot15 type78 @(240,169) vs 玩家 (243,160) | 重解 trace：**slot15 type78 (240,169) vs player (243,160)** 命中 ✅；但 dx/dy 符号约定需注明（见 F2） |
| C1 `shape_data` 仅测试、生产不设置 | **HEAD b92139a 成立**：`git grep shape_data b92139a` 仅 engine.rs/tests；但当前工作区 task-37 已新增 `Host::load_shape_data`（host.rs:466/473/481）——见 F3 |
| C2 生产无 `spawn_init/schedule_step/shot_slot_alloc` 调用 | HEAD b92139a grep 仅 shot.rs 定义 + tests/L2/L3，无 production ✅ |
| C3 `/C5/C7` 未走 `shot::tick`、f630 当 HP、无玩家被击路径 | engine.rs `combat_frame`/`land_leader`/`process_waves` 代码核对一致 ✅ |
| Boss/EN-9+ 9 个 fail-closed 点 | stage.rs:165-171/216-219、wave_record.rs:185-188、engine.rs:721-749、enemy.rs BossBehavior 仅测试等均可查 ✅ |

## 2. 敌坐标写者链核验

- **D1 静态扫描**（独立重跑）：`$04E0` **n=69**：b00:1/b02:1/b03:10/b04:3/b05:11/b06:13/b07:14/b14:16；
  `$04FC` **n=75**：b00:1/b02:2/b03:12/b04:3/b05:7/b06:12/b07:14/b14:24 —— 与手册 §3.1 **逐 bank 全同** ✅。
- 已实现写入：`run_handler`（0x8337/0x836E 族）与 `motion_update` 的 `slot.x/slot.y` 写点存在；
  `land_pair/land_leader` 只落 type/参数/f630 等，**不写 x/y**（engine.rs:463-500）——"生成层/行为 handler 承担
  绝大多数写入、坐标恒 0 根因"成立 ✅。
- **W12**：`completeness.md:647` 原文"**~~W12~~ 前提证伪收口（t64：VM 输出不进 $04xx——BG 搬运桥 $85A0
  定谳，N-0080/0082）**"——被审文件 §3.3 引用与现行结论一致 ✅；"W12 桥"措辞确应更正。
- **oracle trace schema/覆盖**：`enemy` 文件字段 `{f,state,x,y}`、Δ8 采样（f1264 起）与文件实测一致；
  gameplay-stage1 143 / fire-hold 136 / weapon-pickup 136 / stage-appear 112 行均可复算 ✅。
- `StageProbe` 仅 `slot_types[28]`、无 x/y（engine.rs:924-960）✅；"坐标不可对拍"与 FrameView `position=None`
  声明一致；§3.5 最小证据清单自洽。

## 3. Boss / EN-9+ 核验

- **ROM 字节**（独立直读）：file `0x103F2`=`01 4a`、`0x10409`=`01 5f`；`$A3CD` 流含 `…01 4A
  FB 05 07…`、`$A3E4` 流含 `… A5 01 5F 03 11 A4 03 00 00 FE FB A3…`、`$802A` 目录尾
  `B9 06 A0 85 9F B9 07 A0 85` —— 与手册 §4.2 **逐字节一致** ✅。
- `process_waves`（engine.rs:721-749）只做 CA/far-call/关末门，不解析流内 spawn ⇒ `$A3CD/$A3E4` 永不 spawn ✅；
  `stage.rs:216-219 boss_segment_unproven`、`stage.rs:165-171 record_index≥8 None`、
  `wave_record.rs:185-188 TERMINAL_DROPPERS`（仅测试引用）、`BossBehavior`（仅测试引用）—— 9 点核对通过 ✅。
- **EN 8×5B**：8 条 5B 重解得 script `$B3E6/$B2B4/$B2B4/$B3E6/$B235/$B235/$B21D/$B2B4`，与手册逐条一致 ✅。
- **EN-9+ `$ADF8+`**：raw 字节与手册一致；按 5B 步进首条 ptr=`$B0B2`、次条实际为 **`$90B3`**（手册写
  `$90F3`，见 F1）；按 4B 步进首条 ptr=`$B2B4`、次条 `$F310`——**"5B/4B 均不能全域对齐"的结论仍成立**，
  EN-9+ 只能作 B 级族级判断 ✅。

## 4. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- **F1（低，建议核对）**：§4.4 EN-9+ 5B 步进"次条 ptr=`$90F3` 不成立"——按手册对 EN 5B 记录的
  **ptr 取第 4/5 字节**（同 `$B3E6` 解法），`ADF8` 第二条 5B 记录 `10 10 f3 b3 90` 的 ptr 应为 **`$90B3`**，
  不是 `$90F3`。结论（次条不在 `$B2B4/$B3xx` 族、不可全域对齐）**不受影响**；建议改正数值。
- **F2（低，建议注明）**：§2.3 玩家样本 `dx=−3,dy=+9` 的符号约定与 `hit-sampling-h16.md` E1
  （`dx=3,dy=12`，玩家−对象）**相反**；重解 trace 确认 slot15 type78 @(240,169)、player (243,160)，
  建议明确"dx=对象−玩家"或统一为"玩家−对象"，避免下游误读。
- **F3（信息，建议加时点注）**：§1.2 C1 写"生产侧（game-shell/src/host.rs）从不设置 `shape_data`"——
  在**被审 HEAD `b92139a` 成立**（grep 仅 engine.rs/tests）；但当前工作区 task-37 WIP 已新增
  `Host::load_shape_data`（host.rs:466/473/481）与 `game-shell/src/shape.rs`。建议在 C1 加"审计时点
  b92139a；task-37 WIP 已部分接线，重新核当前工作区"，否则 task-37 读者会把已修复点当仍不可达。

### 建议（P3）
- **F4（低）**：§1.1/§2.1 的"7×10 框"建议写成"**span** 7×10（x−4..x+3 / y−5..y+5）"；实现是
  left=x−4/right=x+3（宽 8 地址）/top=y−5/bottom=y+5（高 11 地址），"7×10"是右−左/下−上的跨度，
  与读者直觉的像素宽高不同。
- **F5（环境说明）**：我实跑 `gates/cli.py check` 当前 **FAIL=rust-tests**（task-37 `game-shell`
  未提交 WIP 的 `stg01_combat` 编译错；非本取证产出）；**docs-links 本身 PASS 332 md/417 links**。
  被审文件已声明工作区并发施工，此处记录环境事实，不算交付缺陷。

## 5. 引用/分级核验（≥5 条）

已独立验证（seed/ROM/trace/代码）：
1. `bank02:$8847-$888A`（弹框注册）seed 行存在；
2. `bank14:$D78B`（A 阅读端）/`$D7DA`（一框一伤）/`$D674`（伤害）/`$D846`（B 端收尾）seed 行存在；
3. `bank02:$87FD`（弹回收）seed 行存在；
4. `bank00:$865C`（生成层九字段）seed 行存在；
5. `rom/gunnac_cn.nes` file `0x103F2`/`0x10409` = `01 4a`/`01 5f`；`$A3CD/$A3E4` 流、`$A16F` 记录族、
   `$ADF8+` raw 均与手册一致；
6. `oracle/subroutines/hit_samples.jsonl` 13 行（10/2/1）；
7. `oracle/traces/{gameplay-stage1,fire-hold,weapon-pickup}.enemy.jsonl` 143/136/136 行、`stage-appear` 112 行、
   schema `{f,state,x,y}`、Δ8 采样；
8. 代码：`shot.rs` 命中框/重叠/属性/清框/伤害 API；`engine.rs` `combat_frame/process_waves/StageProbe/
   motion_update/land_leader`；`stage.rs`/`wave_record.rs` fail-closed；`enemy.rs` BossBehavior；
9. docs：`hit-sampling-h16.md`、`arena-midboss.md`、`completeness.md:647` W12 引文。

## 6. 纪律/链接/环境

- 被审文件为 `docs/findings` 新增，**只读**取证，未改 crates/data/gates/rom/reference（本卡未 git add/commit）；
  当前工作区 crates 改动属并发 task-37，与 task-38 无关。
- **docs-links PASS**：`gates check` 输出 332 md/417 本地链接全部有效；被审文件内相对链接逐条 0 断链。
- 全量 `check` 当前 FAIL=rust-tests（task-37 WIP），不属本产出；task-43 要求的 docs-links 已 PASS。
- 未证不发明：W15/W15′/W18/B4/$4C/EN-9+ 均显式未证，未写成已证；"样本不足"如实（未命中侧）。

## 7. 结论

**pass**。H-16 现状/样本/接入建议、敌坐标写者链（69/75、W12、trace 对照）、Boss/EN-9+ 结论与代码/ROM
证据**均经独立复算成立**；未证项与样本不足披露诚实。F1（EN-9+ 次条 ptr `$90B3`）/F2（dx 符号约定）/
F3（C1 时点 vs task-37 WIP）建议顺手修/注，不阻塞交付。
