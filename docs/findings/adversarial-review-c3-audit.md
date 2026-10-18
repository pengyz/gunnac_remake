# 对抗复核：`$2B`/`$4E` 敌坐标 C3 审计（task-157）——adversarial-review-c3-audit

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用 decomp-auditor 自述**）
> 被审：`docs/findings/enemy-coords-c3-audit.md`（**230 行，untracked**，task-157）。
> 纪律：**未跑 Mesen/gates check/status/visual**；只读 `rom/gunnac_cn.nes`、`reference/seed`、`oracle/traces/gameplay-stage1.enemy.jsonl`、`analysis/traces/gameplay-stage1.frames.jsonl` 与 `tools/da6502.py`；本报告只写该文件、未 git add/commit。

---

## 0. VERDICT：**needs_revision**（handler 指针/静态写点基本可核；**`$2B` y "每槽逐帧全等" 被 trace 证伪**；spec §4.2 对拍口径未真正执行；需修正后才解锁 C3）

- **必须修（A/证据）**：
  1. **`$2B` y 的"—每槽仅首 ~10 帧相位偏移、其余逐帧全等"不成立**：GUI trace 的 0x2B **槽被复用/多次 spawn**（slot8 resets f1470/f2032；slot9 f1480/f1863/f2002；slot10 f1490；slot11 f1500/f1854/f1982；slot12 f1510/f1874/f2012；slot13 f1520/f1884/f2022）。按"整个 slot 连续"做 `+0x90` 仿真（audit §6 命令）得 slot8 **mismatch 203/236**、slot9 **315/346**、slot10 24/52、slot11 239/271、slot12 256/289、slot13 317/347，而非"仅首10帧"。**按 spawn 分段**后多数段可用 phase=0x90 匹配到 0/1 mismatch（slot8 三段 0/1/0；slot9 四段 0/0/**110**/0；slot10 0/0；slot11 0/1/0/0；slot12 0/1/0/0；slot13 0/1/0/0；slot16-18 全0），但 slot9 f1863–2001 段仍异常，需作者解释/重做。**y 静态初始化（#08、$0588=#90）仍 A；但 trace 结论必须改为"按 spawn 分段 + 槽复用重置"，并据此改 C3 建议。**
  2. **对拍口径未按 spec §4.2 执行**：audit §5 的 **1,090 mismatch=0** 是 `oracle ∩ GUI` **同 (帧,slot,型)** 的逐槽精确交叉（我已复算：1,090/0，A，强证据）；但 spec §4.2 要求 **`V1_TYPES` 过滤、共同 (帧,型) 采样点（≥20）、分母先冻结、≤1px≥90%、且对 Rust 侧**。当前报告没有冻结分母/达成率/per-type 表，也没有 Rust 侧；按 spec 口径 oracle∩GUI 的 common **501 个 (帧,型)**（我复算）而非 1090。应把 1090/0 标为 **"预实现 surrogate 交叉"**，另列 §4.2 正式验收（C3 后做），不能表述为已执行 §4.2 判据。
  3. **`$2B` oracle 首样本 x 计数**：audit §3.4 L85 写 oracle `x∈{32(×6),231,24,48}`；实际 oracle 9 槽首 x = **32×5、35、231、24、48**（slot10 首样本 x=35）；x=32×6 是 **GUI first-valid** 的计数（我复算 GUI：slots8–13 六槽首有效 x=32）。需拆开 oracle/GUI 两句或改数。
- **通过项（A/B）**：
  - `$2F` 先例字面量字节 `A9 80 / 9D E0 04 / A9 08 / 9D FC 04`（file 0x049E5）✓；`$2B` init/tick 字节与 audit §3.1/§3.2 表逐项一致（`A9 91…$041C/$0438`、`$04C4=#93`、`vy_lo=#90`、y=#08、`JSR $C599`、`$C4F7` 奇偶、`$90DA-$90E9` 目标公式、`$9120` `LDA #$20/JSR $CD63` 等）✓。
  - **`$4E` 地址口径更正成立（B）**：task 卡 `bank02:$AE44` 与 ROM/反汇编不符；bank4 offset `$E44`（file `0x08E54`）字节是合法 handler（`A9 93 / STA $04C4`→`JSR $C872`/`$CB4F`/`$C894`/`$AE67`/`$CC51`），bank2 `$AE44`（file `0x06E54`）是无关代码；audit 按 bank4 引用正确（既有 `enemy-coords-impl-spec` 亦记 `$4E→$AE44(bank04)`）。
  - `$4E` 坐标写点/初值：`$C608` 的 `$C61B/$C63B/$C626/$C635` 写 y/x、dir 翻转；`$C872` 的 `$0470&0x7F→$C8A7[$idx]→$C8B7+offset→JSR $C608`；`$C8A7` 16B=`00 19 23 30 47 58 62 6D 78 7F 8B 96 9F AA B5 C4`，`$C8B7` 首字节与 audit 一致；params {9,137,6,134}→idx 9/9/6/6→脚本 `[0x10,08]/[0x18,08]`，dir 翻转得 x=16/240/232 —— 与 oracle 首样本 x 一致 ✓；`$4E` entry 无直接 x/y/速度写 ✓。
  - `$4E` 运动 C（脚本命令/时序未解）→ fail-closed、max spawn 初始可见：与现有证据相符，建议未越证据（但 C3 实现需按 spawn 重置，见 §3）。
  - `oracle∩GUI` 1,090/0 交叉本身成立（A）；`$2F` 5槽 dx=dy=0 成立。
- **格式**：docs-links **PASS**（395 md/468 links）；报告未改 expected/threshold/code/gates；仅本报告新增。

---

## 1. Handler 指针逐值核（A，抽样全表）

`tools/da6502.py` 直接读 ROM：

- `$2F` precedent `bank02:$89D5`：`A9 80 / 9D E0 04 / A9 08 / 9D FC 04`（file 0x049E5）✓。
- `$2B` `bank02:$90AB`：`LDA #$91→$041C,X`、`LDA #$20→$0438,X`（行为指针）、`$04C4=#93`、`$05A4=0`、`$0550/$056C=0`、`$05DC/$05C0=0`、`y=#08 ($90CD)`、`vy_lo=#90 ($90D0)`、`JSR $C599`；`$90DA-$90E9` 的 `LDA $04E0,X; LSR A; BCC→$80; ASL; BPL/ADC #$30/SBC #$2F→$05F8` 逐字节与 audit 一致；`$9120 LDA #$20; JSR $CD63` 一致。
- `$4E` bank4 file`0x08E54`：`A9 93/STA $04C4; A9 03...; JSR $C872/$CB4F/$C894/$AE67/$CC51` ✓；bank2 file0x06E54 `$AE44` 不是同构 handler（`65 37 A8 B9...`），audit 的 bank4 更正是对的。
- `$C608/C872/C4F7/C599` 字节与 audit 逐项一致；`$C599`：`LDA $0470,X; BEQ→RTS; STA $04E0,X; LDA 0; STA $0470,X; ORA #$FF; RTS`，caller `$90D8 BEQ $90EE` 的 param=0/非0 分支语义与 audit 描述一致。

## 2. Trace 复算（A/B）

- oracle `gameplay-stage1.enemy.jsonl` **143 行（f1264–2400，8f）**；GUI `gameplay-stage1.frames.jsonl` **2400 帧逐帧、iram 1280B**；解码口径与 audit §1 一致（`$0400+0x100+s`、`$04E0+0x1E0+s`、`$04FC+0x1FC+s`）。
- **oracle∩GUI 同 (帧,slot,型)**：**1,090 样本，坐标 mismatch=0**（A，独立复算）——与 audit 一致；这是很强的 Mesen 内部一致性证据。
- **`$2B` oracle 首样本**（我列）：slot8(1344,32,9)、9(1352,32,8)、10(1368,35,11)、11(1376,33,10)、12(1384,32,9)、13(1392,32,8)、16(1896,231,9)、17(1904,24,8)、18(1992,48,8) ⇒ **x=32 出现 5 次，不是 6 次**；GUI first-valid 则确为 slots8–13 六槽 x=32（见 F3）。
- **`$2B` GUI 槽复用/重置**（关键反例）：
  ```
  slot8  resets f1470,f2032
  slot9  resets f1480,f1863,f2002
  slot10 resets f1490
  slot11 resets f1500,f1854,f1982
  slot12 resets f1510,f1874,f2012
  slot13 resets f1520,f1884,f2022
  ```
  例：slot8 f1469 `(x=102,y=26)` → f1470 `(191,8)`；slot9 f1479 `(94,25)` → f1480 `(191,8)`；slot10 f1489 `(82,23)` → f1490 `(191,8)`。整槽连续 `+0x90` 仿真 mismatch 如 §0；按 spawn 分段后多数段 phase=0x90 匹配 0/1，但 slot9 f1863–2001 段 mismatch=110（需作者解释）。
  ⇒ audit §3.4 "每槽仅首~10帧5处相位偏移、其余逐帧全等"**不能按字面成立**；y 静态初始化/速度公式仍 A，但 trace 验证必须按 **spawn 段**并处理槽复用。
- **`$4E` 初值**：oracle 首样本 x=16/240/232，与 `$C8A7/$C8B7`+dir 规则一致；GUI 4E 首有效样本 x=170/178/… 是 spawn 后运动采样，不是 init，audit 未误用（其 §4.3 用 oracle/静态，正确）。
- **spec §4.2**：共同 (帧,型) 点数我复算 **501**；audit 的 1,090 是逐槽样本（不是 spec 的每(帧,型)首序槽分母），且未给分母冻结/≤1px 达成率/Rust 侧。故 1,090/0 是"更强的逐槽交叉"但**不能替代 §4.2 正式判据**；C3 后需按 spec 重跑。

## 3. C3 建议/分档

- `$2B`：y init/vy 静态 A；x init（param/字面量）A（param 来源 B）；x 轨迹公式 A（`$5F8`/`$CD63`/`$CD08` 指令）但**未做逐帧仿真**（audit 自述）；C3 实现前应明确 **spawn 时重置**（槽复用），否则 y/坐标错位。
- `$4E`：entry 无直接坐标写；初值可静态投影（表+脚本首字节+dir）；运动命令集 C 未解 ⇒ **fail-closed，最多 spawn 初始位置可见**——audit 建议合理、未越证据；C3 也不应把 `$4E` 运动列计入达成率（audit 已建议）。
- **需补**：`$2B` 的 y 验证按 spawn 段；slot9 f1863–2001 段异常单独说明；`$2B` x 轨迹与 GUI 的逐帧等价留 C3 实现后验证（audit 已标 B/未做）。

## 4. Findings 四分类

### 4.1 证据不足/结论错误（A/中，必改）
- **F1**：`$2B` y "每槽逐帧全等"被槽复用/重置证伪；按 audit §6 的整槽仿真 mismatch 203–317/槽，不是"首10帧"。需改按 spawn 段 + 槽复用重置；否则 C3 y 可见化会错。
- **F2**：spec §4.2 未执行（1,090 是逐槽 oracle∩GUI，不是 V1_TYPES 共同(帧,型)/冻结分母/Rust 侧）；应标 surrogate，正式验收留 C3 后。
- **F3（低）**：oracle `$2B` 首 x "32×6"实为 GUI 6、oracle 5+x35；拆分/改数。

### 4.2 Handler/静态（pass）
- `$2F` 先例、`$2B` init/tick、`$4E` bank4 地址更正/entry/`$C608`/`$C872`/`$C8A7`/`$C8B7`/param 映射均逐字节可核；`$4E` 运动 C/fail-closed 判断有据；无伪造。

### 4.3 C3 建议（低，需加条件）
- `$2B` x 公式 A 但未 trace 仿真；C3 应标"先实现 spawn/复用 + x 公式，按 §4.2 对拍"；`$4E` 仅初始可见、运动 fail-closed 不变。audit 建议方向正确，需补槽复用条件。

### 4.4 越权/格式（无）
- 只读、未改代码/expected/gates；docs-links PASS；本报告仅新增文件、未 git add/commit。

## 5. 命令与输出摘要

```text
# ROM（da6502）
$89D5 A9 80 / 9D E0 04 / A9 08 / 9D FC 04
$90AB A9 91 9D 1C 04 / A9 20 9D 38 04 / ... y=#08, vy=#90, JSR C599
$90DA-$90E9 target formula；$9120 LDA #$20 JSR $CD63
bank4 $AE44 A9 93 STA $04C4 ... JSR C872/C894/AE67/CC51（bank2 AE44 非此 handler）
C8A7 00 19 23 30 47 58 62 6D 78 7F 8B 96 9F AA B5 C4
C8B7 40 08 41 0C ...
params 9/137/6/134 → idx9/9/6/6 → script [10,08]/[18,08] → x16/240/232（dir 翻转）
# trace
oracle 143 行；GUI 2400 帧；oracle∩GUI 1090 samples mismatch0
2B oracle first x: 32×5 + 35 + 231 + 24 + 48（audit 写 32×6 实为 GUI）
2B slot reuse resets: slot8 f1470/f2032, slot9 f1480/f1863/f2002, slot10 f1490, slot11 f1500/f1854/f1982, slot12 f1510/f1874/f2012, slot13 f1520/f1884/f2022
整槽 sim mismatch: 203/236,315/346,24/52,239/271,256/289,317/347（audit §6 命令起点 phase=0 亦同）
按 spawn 分段：多数段 phase0x90 →0/1；slot9 f1863-2001 段 mismatch110（待查）
spec common (帧,型)=501；audit 1090 为逐槽样本（非 §4.2 分母）
docs-links PASS（395 md/468 links）
```

## 6. 复核边界

1. 未跑 Mesen/gates check/status/visual；未改代码/expected/gates；只读当前 tracked oracle trace + gitignored GUI trace + ROM/seed。
2. `$2B` y 的 spawn 分段结论基于 GUI trace 槽复用检测（y 明显回落>5 为 reset 边界）；slot9 f1863–2001 异常需作者进一步核，不排除另一子状态。
3. `$4E` bank4 地址按 ROM 字节+既有 TYPE_DISPATCH 口径；若后续另有窗口视图证据，需另立 B 级修订。
4. 本报告只新增文件；未 git add/commit。

---

【2026-10-18】decomp-verifier，独立复核 task-157（HEAD `f58cf99`；C3 audit untracked）；handler 字节/表/初值、oracle∩GUI 1090/0 通过；`$2B` y 整槽"逐帧全等"被槽复用证伪、§4.2 未执行、oracle 首 x 计数错 ⇒ needs_revision。

---

## 点验（2026-10-19，task-163）

> 点验对象：task-162 修正后的 `enemy-coords-c3-audit.md`（**271 行**，未提交）；HEAD `d6c43ec`；只点验 task-161 的 F1/F2/F3 + 失效表述。未跑 Mesen/gates check/status/visual；`analysis/traces/gameplay-stage1.frames.jsonl`（2400 帧）+ tracked oracle 只读。本段只追加、未 git add/commit。

**VERDICT: pass**（F1 24 段 0 mismatch/复用帧表/slot9 解释/reset 证据、F2 surrogate、F3 原始值拆分、失效表述清理均验证；C3 审计可入库，C3 实现卡解锁。仅 1 条非阻塞编辑建议：把 §3.4/§7 的“待点验统一”改为“task-163 已裁定按原始值列示”。）

### F1（必核）PASS
- **按 spawn 分段仿真 24 段全部 mismatch=0**（独立重跑 audit §6 `segments()` 逻辑，`subpix0=0x90`、首帧=init 后1tick）：
  - slot8：f1342-1373/32/init32、1470-1577/108/191、2032-2125/94/48；
  - slot9：1352-1381/30/32、1480-1591/112/191、**1864-1972/109/24**、2002-2093/92/48；
  - slot10：1362-1388/27/32、1490-1513/24/191；
  - slot11：1372-1402/31/32、1500-1609/110/191、1854-1889/36/231、1982-2072/91/48；
  - slot12：1382-1413/32/32、1510-1620/111/191、1874-1982/109/231、2012-2045/34/48；
  - slot13：1392-1420/29/32、1520-1630/111/191、1884-1993/110/24、2022-2115/94/48；
  - slot16/17/18：1894-2004/111/231、1904-2011/108/24、1992-2086/95/48；
  - **合计 24 段，逐段 mismatch=0**，与 audit 表完全一致。
- **复用帧表与 trace 一致**：slot8 resets f1470/f2032；slot9 f1480/f1864/f2002；slot10 f1490；slot11 f1500/f1854/f1982；slot12 f1510/f1874/f2012；slot13 f1520/f1884/f2022；slot16/17/18 各 1 段；旧占用尾帧（如 slot8 f1469=(102,26)）保留在 trace，作为边界依据 ✓。
- **slot9 f1863–2001 解释成立**：该窗口 = 旧尾帧 f1863(y=4) + 新 spawn f1864–1972（109 帧 0 mismatch）+ 残留 f2001；按 spawn 边界重切后 0 mismatch，原 110 mismatch 是“把旧尾帧当新 spawn init、不重置”的人为分段错误，不是 vy 模型错误 ✓。
- **C3 reuse 重置条件 PASS（证据边界一致）**：`$2B` init 写 `$04FC/$0588/$05A4/$0550/$056C/$05DC/$05C0`，**不写 `$048C`**；`$C599` 在 `$0470==0` 时直接 RTS、**不清 `$0470`**（我按 ROM 字节/反汇编核）；audit 明确“复用重发时子像素/相位是否被 generation 清零未经 ROM 闭合（B）”，C3 建议显式复位 y=8/vy pair/`$048C`/`$0470`/`$05DC`/`$05C0`/`$5F8` 且“首帧=init 后1tick”采样——与证据边界一致，未越级。

### F2（surrogate 标注）PASS
- 已明确 **1,090/0 是预实现 surrogate**（oracle 8f × GUI 逐帧同 (帧,slot,type) 交叉），**不是 spec §4.2 正式验收**；正式口径（V1_TYPES、共同(帧,型)≥20〔我的独立计=501，audit 标待冻结〕、分母先冻结、≤1px≥90%、Rust 侧）留 C3 实现后；§5/§7 均复述。✓

### F3（33/32 归组）PASS（点验裁定）
- 原始值拆分正确：oracle 2B 首 x = **32×4（slots8/9/12/13）+33(slot11)+35(slot10)+231+24+48**；GUI first-valid = **32×6（slots8–13）+231+24+48**。
- **点验裁定（按 Lead 建议）**：采用 **原始逐槽值列示**为正文口径；把 slot11 的 33 归入“32 组”仅作**对账注**（32×5+35 的 9 槽总数一致），不得用归并值替代原始值。audit 当前已是“原始值列出 + 归并说明”，符合；建议把文中“待点验统一”改为“task-163 已裁定按原始值列示，32 组归并仅对账注”（非阻塞）。

### 失效表述/格式
- `逐帧全等` 仅出现在 F1 作废语境（L8/L116：原表述作废、改 spawn 段），无残留现行断言 ✓。
- `docs-links` **PASS（395 md/468 links）**；audit 未改 expected/threshold/code/gates；本报告只追加、未 git add/commit。
- **结论：task-161 F1/F2/F3 修正全部成立、可复算；C3 审计可入库，C3 实现卡（`$2B` 接线 + reuse 重置，`$4E` 仅初始位置/fail-closed）解锁。**

【2026-10-19】decomp-verifier，点验 task-162/task-163（HEAD `d6c43ec`）；24 spawn 段全 0 mismatch、slot9 边界/复用表一致、surrogate 标注/原始值裁定/失效表述清理通过；docs-links PASS。
