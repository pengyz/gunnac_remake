# 对抗复核 #72 — DEV-3 S5-ALL-CLEAR 八关通关跑（开发双关：未证转移发明/段级机制来源/S5 判定真伪）

> 独立复审（fresh context，证伪立场）。被审对象：commit `c18b56a`（feat，段级链架构+关末/通关/结局链）
> + `8dda6d9`（ratchet 135→144）。
> 方法：不采信 commit 自述/不复用被审结论；ROM 字节直读重导（hexdump+seed 反汇编交叉）、
> mutation 独立实测验测试咬合力、对拍独立复跑贴数；每项贴实测输出。
>
> **Verdict: needs_revision**（轻——单点测试咬合缺口 F1，1 行可修；核心 ROM 证据链与 S5 披露全过）。
> 焦点①新边逐条 ROM 重导 **全过**（4/4 字节级独立复现）；焦点③ **覆盖层级裁定 = 链驱动状态机层**
> （非 ROM/Mesen 动态回放，G-S5-6 自认），达成度意见见 §4——**"8 关可通关"Stage A DoD 不应据本 commit 判满**。
> 附 1 项测试咬合缺口（F1，**建议修**）、1 项低危测试强度（F2，可选）、1 条覆盖措辞观察（F3，advisory）。

---

## 0. 对拍独立复跑（贴数，未采信自述）

| 复跑项 | 命令 | 实测输出 | 判定 |
|---|---|---|---|
| workspace 测试 | `cargo test --workspace` | **144 passed / 0 failed** | ✅ 与 8dda6d9 自述一致 |
| L2 子程序级 | `python3 tools/diff_subroutines.py` | **L2-DIFF: 121/121 通过（golden==ROM==game-core）** | ✅ |
| 门禁总闸 | `python3 gates/cli.py check` | **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0** | ✅ |
| S5 golden 对拍 | `cargo test -p game-oracle --test l5_s5` | **4 passed / 0 failed** | ✅ |
| 关末/结局链 | `cargo test -p game-core --test stage_clear` | **5 passed / 0 failed** | ✅ |

ratchet 135→144 留痕核：`data/status/baselines/baseline_history.jsonl` 末条 `old:135,new:144`，
reason 含"既有 135 全数复跑通过（cargo test 144/144），l2-diff 121/121、gates 12/12 复跑不降"；
`baseline.json` `rust_test_count:144`；SHA256SUMS 同批更新 ⇒ **留痕完整，只升不降 OK**。

---

## 1. 三焦点核验总表

| 焦点 | 核验物 | 判定 | 证据位 |
|---|---|---|---|
| ①a 战斗→关末边 `Combat→StageClear` | E2F3 `LDA $26/AND #$10/BNE E2FC` 门 | **过（A）** | §2.1 |
| ①b 关末→商店边 `StageClear→Shop` | E396 `JMP E16B`→E171 `JSR E649/00 01 C3 9B`→`$9BC3` | **过（A）** | §2.2 |
| ①c 关末→结局边 `StageClear→Ending` | E392 `CMP #9/BEQ`→E3A1 `JSR E649/00 01 F9 95`→`$95F9` | **过（A）** | §2.3 |
| ①d 关末→标题哨兵边 `StageClear→Title` | E382 `CMP byte_FFF8`(=0x0A 三版同值)→E387-E38F | **过（A，死分支保真）** | §2.4 |
| ①e 负测试真拒（StageClear→StageStart） | M3 mutation | **真拒（咬合）** | §3 |
| ①f 负测试真拒（帧序倒退） | M4 mutation | **真拒（咬合）** | §3 |
| ①g 负测试真拒（未证模式 fail-closed） | M5 mutation | **★不咬合（F1）** | §3/F1 |
| ② 段表 6 行 bank:addr 真支持 | 逐行 ROM 直读 | **过（6/6）** | §2.5 |
| ②b 关间商店 8 次对账（第 8 关末不回店） | A3 关末链 INC→CMP#9 分支 | **对账干净（8 次精确）** | §2.6 |
| ③ S5 判定覆盖层级 | golden 帧锚分级 + diff 语义 | **链驱动状态机层（非动态回放）** | §4 |
| ④ 对拍贴数 | §0 | **全绿** | §0 |
| ⑤ mutation ≥3 | M1-M5 | **5 实测，4 咬合 + 1 缺口** | §3 |

---

## 2. 焦点①+②：未证转移发明核查 + 段级机制来源（ROM 直读重导）

### 2.1 `Combat→StageClear`（E2F3 门）——过

`reference/seed/gunnac.asm` bank15:E2F3 逐指令 + ROM hexdump 双证：

```
bank15:E2F3  A5 26   LDA $26
bank15:E2F5  29 10   AND #$10      ; $26 bit4 门
bank15:E2F7  D0 03   BNE loc_E2FC  ; 置位 → 入关末块
bank15:E2F9  4C 1C E2 JMP sub_E21C ; 未置位 → 常规主环（不入关末）
```

⇒ `Combat→StageClear` 的进入条件 = `$26 bit4` 置位。置位上游 bank00:$8D02 `ORA #$10/STA $26`
被审方标 **B 级**（调用图未逐指令展开）——**分级诚实**，未拔高。边本身字节级成立。

### 2.2 `StageClear→Shop`（E396→E16B→`$9BC3`）——过（本关核心新边）

关键质疑点："E396 JMP loc_E16B 首个 far-call = `$9BC3` 商店 init"是否为**未证转移发明**。独立重导：

- E396 字节（hexdump 独立复现）：`4c 6b e1` = `JMP loc_E16B` ✓
- E16B 块（seed L87473-87478）：
  ```
  bank15:E16B  20 90 E5  JSR sub_E590   ; 非 far-call：PPU mask（E590 AND #$E7/STA $2001）
  bank15:E16E  20 56 ED  JSR sub_ED56   ; 非 far-call：服务
  bank15:E171  20 49 E6  JSR sub_E649   ; ← 首个 far-call（内联分派约定）
  bank15:E174  00         BRK          ; [ret+1]=0x00 → R6←bank0
  bank15:E175  01 C3      ORA ($C3,X)  ; [ret+2]=0x01 → R7←bank1
  bank15:E177  9B                       ; [ret+3]=0xC3 / [ret+4]=0x9B → JMP ($24)=$9BC3
  ```
- 分派约定独立解码（sub_E672 L88057-88062）：`LDA (6),Y(Y=4)→STA $25` / `(Y=3)→STA $24`
  ⇒ `[ret+4]=0x9B→$25`、`[ret+3]=0xC3→$24` ⇒ **JMP 目标 `$9BC3`** ✓；`[ret+1]=0/[ret+2]=1→R6/R7`
- `$9BC3` = 商店 init：`shop-purchase-flow.md` L91-92（A1 A 级）"far-call `00 01 C3 9B` =
  `bank00:$9BC3` 商店 init"；`sub_E590` 经查为 PPU mask（非 far-call）⇒ **"E16B 首个 far-call"措辞准确**
  （E16B/E16E 两 JSR 非跨 bank 调用）。

⇒ **边成立（A 级字节链闭合）**。E396 目标 = E16B（商店 init 路），**而非**关始段 ⇒ `StageClear→StageStart`
直跳确不存在，fail-closed 负测试锁定合理（非发明）。

### 2.3 `StageClear→Ending`（E392→E3A1→`$95F9`）——过

hexdump 段 `...c9 09 f0 03 4c 6b e1 a9 02 20 03 ee 20 14 db 20 49 e6 00 01 f9 95...` 独立解码：

```
E392  C9 09      CMP #9        ; $180==9？
E394  F0 03      BEQ loc_E399  ; 是 → 通关支
E396  4C 6B E1   JMP loc_E16B  ; 否 → 关间（商店）
E399  A9 02      LDA #2        ; 通关支收尾服务（挂起）
E39B  20 03 EE   JSR sub_EE03
E39E  20 14 DB   JSR $DB14
E3A1  20 49 E6   JSR sub_E649  ; far-call（同 E171 约定）
E3A4  00 01 F9 95              ; R6←0/R7←1/JMP ($24)=$95F9
```

⇒ `$180==9` → `JMP $95F9` 结局入口 ✓（与 §2.2 同一 sub_E672 约定，`[ret+3]=F9/[ret+4]=95→$95F9`）。
`bank00:95F9` 入口 `LDA #$80/STA $26`（ending_entry 镜像）见 ending-sequence.md §2，A 级。

### 2.4 `StageClear→Title`（E382 `$FFF8` 哨兵）——过（死分支保真）

hexdump `ad 80 01 cd f8 ff d0 0b` = `LDA $180 / CMP byte_FFF8 / BNE E392`；命中 → E387-E38F `JMP loc_E073`（回标题）。
`$FFF8` 三版独立直读：**CN=0x0A / US=0x0A / JP=0x0A**（file 0x20008，实测贴 §0.①）。
可达 `$180` 最大 = 9（8 关清完）< 0x0A ⇒ **不可达死分支**；被审方按 D4 保真保留（EarlyEnd 边 + 测试）✓。

### 2.5 段表抽 6 行 bank:addr 真支持（焦点②）

| 段 | 表内指针 | 独立核 | 判定 |
|---|---|---|---|
| 标题段 | R-ui-7 f180/360/540 | golden `s5-title` 3 事件帧锚（title-menu trace） | 支持（A，trace 锚） |
| 商店段 | E396→E16B→`$9BC3` | §2.2 字节链 | 支持（A） |
| 关始段 | `bank00:$8000-$8025`（`$180` 目录行索引） | seed L13-30：`LDA $180/ASL×3/TAY/LDA $A000,Y`（8 字节/行） | 支持（A） |
| 战斗段 | R-ui-6 f1500+ / R-stage-tick-* | 表内已标 **A(STG-01)/B(STG-02..08 内容)** | 支持（分级诚实） |
| 关末段 | `bank15:$E2FC-$E396` | §2.1 + hexdump PRG 0x1E366-0x1E3B0 全对 | 支持（A） |
| 通关判定/结局 | E392/E3A1→`$95F9` | §2.3 | 支持（A） |

6/6 行的 bank:addr 或规则号真支持该段；**未发现地址错配/规则号冒挂**。

### 2.6 关间商店"每关必到 8 次"对账（含第 8 关末不回店）——干净

A3 关末链分支（§2.3 hexdump）：`INC $180` → `CMP #9`：
- `$180≠9`（第 1..7 关清完）→ E396 `JMP E16B` → 商店 init（回店）
- `$180==9`（第 8 关清完）→ 通关支 → `$95F9` 结局（**不回店**）✓

到店次数对账：
```
title→Shop            = 到店①（关 1 前）
关1 清→$180=2→Shop    = 到店②（关 2 前）
关2 清→$180=3→Shop    = 到店③（关 3 前）
...
关7 清→$180=8→Shop    = 到店⑧（关 8 前）
关8 清→$180=9→Ending  = 无店（通关支）✓
```

⇒ **8 次到店 = 1（title）+ 7（关间装载链），每关开始前恰 1 次；第 8 关末确实不回店。**
主张与 A3 关末链**精确对账，无多算/漏算**。golden `stg01-shop..stg08-shop` 8 检查点 + 测试
`run.shop_visits==8` 一致。R-shop-4 升格用 **in-place dated 注**（shop-economy L50："进店时机升格 A 级"，
标题保留 B 级"显示时机完整逻辑未证"）——**只升"进店时机"子项，未整条拔高，符合 §3.6 证伪更正文化**。

---

## 3. mutation 独立实测（焦点①负测试咬合力）——5 实测，4 咬合 + 1 缺口

改代码/闸 → 跑测试 → 看是否 FAILED（真咬合）→ 还原（`git status` 确认 clean）。全部独立实测：

| # | mutation | 预期咬合 | 实测 | 判定 |
|---|---|---|---|---|
| M1 | `INC $180` 改常量（`st.stage=5`） | dispatch 测试 + 链测试 | `stage_clear_dispatch_three_way` **FAILED**、`s5_chain_runs_and_diffs_golden` **FAILED** | ✅咬合 |
| M2 | AllClear 阈值 `==9`→`==8`（提前通关） | 7 关负向不出结局 | `s5_all_clear_judgement_is_state_derived` **FAILED**、dispatch **FAILED**、chain **FAILED** | ✅咬合 |
| M3 | 关节边偷加 `StageClear→StageStart` | fail-closed 边集负测试 | `stage_clear_fail_closed_and_edges` **FAILED**（`reject(StageClear,StageStart)` 咬） | ✅咬合 |
| M4 | 关闭 l5_legality 帧序倒退检出 | 帧序倒退负测试 | `s5_legality_gate_rejects_invented_joints` **FAILED**（负向③咬） | ✅咬合 |
| M5 | 关闭未证模式 fail-closed（`other→Boot`） | 未证模式负测试 | **全 4 passed（不咬合！）** | ★**缺口 F1** |

**M1 附注**：7 关负向测试在 M1 下**正确地未咬合**（变异令结局更难达，`!ending_reached` 空真）——
这是单向锁的正常表现；双向由 M2（提前通关）补锁。设计合理。

**M3 附注**：`s5_legality_gate_rejects_invented_joints` 负向① 在 M3 下未咬合，因其**同时注入两缺陷**
（Shop→StageClear + StageClear→StageStart）且用 `.any()`，Shop→StageClear 仍被拒故仍 pass。
真正的 `StageClear→StageStart` 单点锁在 `stage_clear_fail_closed_and_edges`（M3 证其咬合）。
⇒ 覆盖存在但**分居两测试**，见 F2。

**M5 根因（F1）**：`s5_legality_gate_rejects_invented_joints` 负向② 断言
`.any(|p| p.contains("fail-closed"))`；但把 `GameOverMenu`→Boot 后，相邻边 `StageClear→Boot` 仍非法，
其报错串 `"…（t81 已证边集外，fail-closed）"` **也含 "fail-closed"** ⇒ 断言被"边违规"误满足，
即使未证模式 fail-closed 已删也 pass。**断言不特异，负测试不咬合**。

---

## 4. 焦点③（核心）：S5 判定真伪 + 覆盖层级裁定

### 4.1 `s5_chain_run()` "端到端跑通"是**什么层级**——裁定

**裁定 = game-core/oracle 链驱动的状态机仿真，不是 ROM/Mesen 动态回放。** 直接证据：

1. `s5_chain_run_for` docstring **自述**："**移植侧运行**——驱动 game-core 状态机产出检查点链，
   供 `diff_l5` 对拍"。驱动对象 = `FlowState`（粗边）+ `StageClearState::stage_clear_step`（关末镜像）+
   `shop_purchase_step`（购买流）+ `stage_skeleton(n)`（骨架）——**全是 game-core 内部状态机**，无 Mesen 调用。
2. `diff_l5(golden, rust)` 语义 = golden vs `s5_chain_run()` 输出逐事件比对。**两侧都不是 ROM 执行**；
   golden 本身非 Mesen 产出（见下帧锚分级）。故 "diff 0 容差" 是 **golden↔移植侧结构自洽回归锁**，
   **不是** ROM↔port 行为等价判据。
3. golden 帧锚分级（实测 `oracle/traces/s5-all-clear.l5.jsonl`）：**仅 7 事件有真 trace 锚**
   （s5-title f180/360/540 + s5-stg01 f1080/1200/1260/1500，gameplay-stage1/title-menu 实测），
   **其余 37 事件 `f=0` 占位**（未取证段，fail-closed 不发明帧号）。占位段的帧维对拍**空真**（0==0），
   仅"模式序列+检查点名"维承载结构意义。
4. **G-S5-6 自认**："8 关全程 oracle 动态 trace（帧号真值）**未跑**（l5-plan C1-C3 DSL 扩展前置）；golden f=0 占位"。

⇒ 被审方**披露诚实**（G-S5-6 显式 + R-l5-8 帧锚分级显式），但承载"跑通"的层级是
**静态字节证据背书的链驱动状态机仿真**。逐段机制（关末/通关/结局/商店 init）有 **A 级字节链**
（§2 独立复现），但"8 关动态可通关"未经真机验证。

### 4.2 "S5-ALL-CLEAR 链可跑通"是否有覆盖夸大——裁定

**准确定界**：验收表②"L5 golden 扩八关 + S5-ALL-CLEAR 链可跑通…`s5_chain_run` 端到端跑通（R-l5-8）✅"——
- 读作"**段级链在 game-core 状态机层可端到端走通到 `$95F9`**" ⇒ **成立**（R-l5-8 明确帧锚分级，非夸大）。
- 读作"**游戏 8 关真机可通关**" ⇒ **不成立**（G-S5-6 未跑动态）。

措辞本身有 R-l5-8 + G-S5-6 兜底，**未构成硬性伪语义/覆盖伪造**；但"链可跑通"四字**脱离上下文易被误读为
动态可通关**。建议（F3）：验收②行补 claim-boundary 注（"状态机层；8 关动态 trace 未跑 G-S5-6"），
使"跑通"层级一望即知。

### 4.3 **S5 达成度评估意见（供 captain 裁定 Stage A DoD）**

Stage A DoD 若含"**8 关可通关**"，本 commit 的**真实达成度分层**如下：

| 层 | 内容 | 达成 | 依据 |
|---|---|---|---|
| 链架构层 | 标题→[商店→关始→战斗→关末]×8→通关→结局 结构 | **达成** | §2 段表 6/6 + golden 44 事件 |
| 段机制层 | 关末状态机/三路派发/结局入口/商店 init | **达成（A 级字节忠实镜像）** | §2.1-2.4 hexdump 独立复现 |
| 状态机可遍历层 | game-core 链驱动 8 关走通到 `$95F9` | **达成** | s5_chain_run + diff 0（结构自洽） |
| 逐关战斗内容层 | STG-02..08 关卡内容可打通性 | **未达成（B 级骨架）** | 战段表自标 B；stage_skeleton ≠ 可玩验证 |
| 动态回放层 | ROM/Mesen 8 关真机 trace | **未达成** | **G-S5-6 显式未跑** |
| 帧锚层 | 关 2..8 真实帧号 | **未达成（f=0 占位）** | golden 37/44 占位 |

**给 captain 的裁定建议**：
- 若 Stage A"8 关可通关"= **流程/机制链贯通**（关末→通关→结局的控制流 + 段级可达）⇒ **本 commit 基本达成**
  （链架构 + 机制字节忠实 + 状态机遍历全绿），可计入 Stage A 进度。
- 若 Stage A"8 关可通关"= **行为一致可玩通**（真机/ROM 逐关打通至结局，帧级/视觉级对拍）⇒ **未达成**，
  不应据本 commit 判满。缺口 = G-S5-6（8 关动态 trace）+ STG-02..08 内容考证（现 B 级）。
- **建议口径**：Stage A"8 关可通关"记 **"机制链已落地（A 级字节背书），动态可通关性待 G-S5-6 + 逐关内容"**，
  按链架构层记进度、按动态层挂待办；**不要**以"s5_chain_run 跑通"直接充抵"8 关真机可通关"。
- 一句话：**"能走通这条链" ≠ "能打通这 8 关"**。前者本 commit 已证，后者未证。

---

## 5. findings（四分类）

| ID | 分类 | 严重度 | 描述 | 处理 |
|---|---|---|---|---|
| **F1** | 移植/测试缺口 | **中（建议修）** | `s5_legality_gate_rejects_invented_joints` 负向②（未证模式 fail-closed）断言用 `.contains("fail-closed")`，被边违规报错串误满足 ⇒ M5 证其不咬合；未来 golden 改动可静默放入未证模式 | 回 porter：断言改 `.contains("未证模式")`（或 `.contains("未证模式") && .contains("fail-closed")`）特异化，使 M5 咬合 |
| **F2** | 测试强度 | 低（可选） | 同测试负向① 同时注入两缺陷 + `.any()`，`StageClear→StageStart` 单点不独立锁（分居 stage_clear.rs 的 `reject`） | 可选：负向① 断言计数 ≥2 或特异断言 `StageClear→StageStart` 串；现状靠 stage_clear.rs 兜底已咬合（M3） |
| **F3** | 覆盖措辞 | advisory | 验收②"链可跑通"脱离上下文易误读为动态可通关 | 验收② 行补"状态机层（G-S5-6 未跑动态）"claim-boundary 注（§4.2） |

**非问题（已证伪的潜在质疑）**：
- "未证转移发明"——4 新边**全部 ROM 字节级独立复现**（§2.1-2.4），无发明；`StageClear→StageStart` 直跳
  确不存在（E396 走 E16B 商店路），fail-closed 负测试锁定正确。
- "段级机制来源"——段表 6/6 bank:addr 真支持；8 次到店对账干净（第 8 关末不回店 ✓）。
- "通关判定硬编码嫌疑"——`stage_clear_step` 对 `$0180` 真算术递进 + 三路派发；M2（阈值 9→8）被 7 关
  负向咬合 ⇒ **确非硬编码**。
- "ratchet 无留痕"——135→144 history + SHA 链完整，只升不降。

---

## 6. 修正建议（按优先级）

1. **F1（建议修，1 行）**：`crates/game-oracle/tests/l5_s5.rs` 负向② 断言特异化——
   `.any(|p| p.contains("未证模式"))`（匹配 `l5_legality` 未证模式分支专属串"未证模式"），
   使"未证模式 fail-closed"被删时 M5 真咬合。改后重跑 `cargo test -p game-oracle --test l5_s5` 确绿。
2. **F3（advisory，1 行注）**：`docs/spec/l5-all-clear.md` §5 验收② 行 "S5-ALL-CLEAR 链可跑通"后补
   "（game-core 状态机层；8 关动态 trace 未跑 G-S5-6）"，消除"可跑通"歧义。
3. **F2（可选）**：负向① 断言强化（≥2 缺陷或特异串），或在注释明示"单点锁在 stage_clear.rs"。
4. **不改**：核心 ROM 证据链（§2）与 S5 披露（G-S5-6/R-l5-8）无需改动——已诚实且经独立复现。

## 7. 纪律自查

- 未证不发明：4 新边全 A 级字节链；`$26 bit7` 支/$0300 上游/第 4 关语义/staff roll 内部/8 关动态
  逐项入 G-S5-1..7，**未拔高、未伪装**；第 4 关特判 `todo_*` 登记不解释 ✓。
- ratchet：135→144 留痕完整，只升不降，SHA 链 ✓。
- 门禁真绿：gates 12/12 实跑（§0），SKIP/TOLERATED 显式打印（status 预检 TOLERATED（无））。
- G-S5-1..7 完备性：对照 A3 关末/结局链 + 8 关动态，逐项核 **无遗漏**（bit7/$0300/第4关/staff-roll/
  并发窗/8关动态/关始细粒度——7 项齐）。
- 复审未采信 commit 自述：全部 §0-§3 为独立复跑/直读，mutation 独立改码实测后还原（`git status` clean）。

---

## 8. Verdict

**needs_revision**（轻——仅 F1 单点测试咬合缺口）。

- **过**：焦点①4 新边 ROM 逐条重导全过（A 级）；焦点②段表 6/6 + 8 次到店对账干净；焦点③S5 披露诚实、
  覆盖层级已裁定为**链驱动状态机层**；对拍 144/121/12/4 全绿；mutation 4/5 咬合。
- **修**：F1（未证模式负测试不咬合，M5 证）——1 行断言特异化即闭合。
- **advisory**：F2/F3（不阻断）。
- **给 captain 的 Stage A 口径**：S5"8 关可通关"**按链架构/机制层记进度**（已达成、A 级背书），
  **动态可通关层挂 G-S5-6 + STG-02..08 内容待办**，勿以 s5_chain_run 跑通充抵真机可通关（§4.3）。
