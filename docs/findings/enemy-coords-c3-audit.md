# 敌坐标 C3 取证：`$2B`/`$4E` handler x/y 写点与速度对逐值审计（enemy-coords-c3-audit）

> 日期：2026-10-18｜角色：decomp-analyst（只读取证；**不跑 Mesen / 不改代码**）｜任务卡：task-157（可玩性并行线 goal ①）
> ROM：`rom/gunnac_cn.nes`（sha256 `4d76901c…c77ae`，未改）；静态工具=`tools/da6502.py`（纯反汇编）；trace=`oracle/traces/gameplay-stage1.enemy.jsonl`（tracked，8f 采样）+ `analysis/traces/gameplay-stage1.frames.jsonl`（GUI 重采，逐帧，iram blob 解码）。
> 本文件为唯一写入；未改代码/expected/门禁；未跑 gates check/status/visual。

> **【2026-10-19 更正（task-161 复核 F1/F2/F3 → task-162）】**
> **F1**：原"`$2B` y 每槽仅首 ~10 帧相位、其余逐帧全等"**被槽复用证伪**（GUI 同一槽会多次 spawn/复用、type 不变）。已改 **按 spawn 分段 + 槽复用重置**：24 个 spawn 段逐段 y 模型（vy_lo=0x90，首帧=init 后 1 tick）**全部 0 mismatch**；整槽连续仿真 mismatch（复核口径）slot8 **203/236**、slot9 **315/346**、slot10 **24/52**、slot11 **239/271**、slot12 **256/289**、slot13 **317/347**；slot9 **f1863–2001 段 110 mismatch 已解释**（旧占用尾帧 f1863 + 新 spawn f1864–1972 + 残留 f2001 未切分，见 §3.4）。C3 建议补 **reuse 重置条件**（y=8/vy pair/子像素相位）。
> **F2**：原"1,090/0"是 **预实现 surrogate**（oracle∩GUI 逐槽同 (帧,slot,type) 交叉），**不是 spec §4.2 正式验收**；§4.2 正式口径（V1_TYPES 过滤、共同 (帧,型) ≥20〔复核独立计=**501**〕、冻结分母、≤1px≥90%、Rust 侧）**留 C3 实现后**（§5）。
> **F3**：oracle 首样本 x（**32×4 + 33 + 35 + 231 + 24 + 48**）与 GUI 首次有效 x（**32×6 + 231 + 24 + 48**）**分开列**（§3.4）。
> **C3 状态**：**不计"已解锁"**——`$4E` 运动仍 fail-closed；`$2B` 可接但须按 spawn/reuse 重置，且待实现后 §4.2 正式对拍。

---

## 0. 分档结论（先读）

| 型 | x 初值 | y 初值 | x 速度对/规则 | y 速度对 | trace 对拍 | 级 | C3 可见化建议 |
|---|---|---|---|---|---|---|---|
| `$2F`（precedent） | **`#$80` 字面量**（`$89D7 STA $04E0,X`） | **`#$08` 字面量**（`$89DC STA $04FC,X`） | 无（静止） | 无（静止） | oracle 5 槽 + GUI 5 槽 **dx=dy=0 全程**（128,8 恒定） | **A** | 已可（C2 已接） |
| **`$2B`（member `0x2B`）** | **`$0470` param**（`$C599→$C59E`）；若 param=0 ⇒ **`$18`/`$E8` 字面量**按 `$2D` 奇偶（`$90F1/$90F5/$90F7`） | **`#$08` 字面量**（`$90CD`） | vx 初=0（`$90BF/$90C2`）；tick：目标 `$5F8`（`$90E9/$90FC`→**偶 x⇒$80；奇 x⇒(x&$FE)+$30 或 −$2F**）→ `$CD63` 每帧 ±`#$20` 调整 vx（`$9120-$9122`）→ `$CD08` 积分（A 级公式见 §3.3） | **`$588=#$90`（`$90D2`）+ `$5A4=#$00`（`$90BC`）⇒ 144/256=0.5625 px/f** | **按 spawn 分段**：24 段 y 模型全 0 mismatch（首帧=init 后 1 tick）；整槽连续仿真因**槽复用** mismatch 203/236…（§3.4）；oracle dy8 median 5(4–5)；GUI dy∈{0,1} avg 0.561–0.574 | **A**（写点/值/公式+分段逐帧） | **可接（须 reuse 重置）**：C3 需按 spawn/reuse 事件复位 y=8/vy pair/子像素相位（§7）；不计"已解锁" |
| **`$4E`（member `0x4E`）** | **脚本首字节**（`$C608` 写 `$04E0`）：`param&0x7F`→表 `$C8A7[16]`→脚本 `$C8B7+offset` 的 `[x,y]`；**dir bit7 翻转 x**（`$0470&0x80`） | **脚本第 2 字节**（`$C608` 写 `$04FC`；本关参数族 y=#08） | 速度对由 `$C64D` **脚本命令**写 `$0550/$056C`（A 地址）；**命令编码/时序未全解（C）** | 由 `$C64D` 写 `$0588/$05A4`（A 地址）；**脚本命令未全解（C）** | oracle 首样本 x=**16/240/232**（=0x10/0xF0/0xE8 与脚本首字节+dir 翻转一致）；dy8 median **18–19 / −7** 分段；GUI dy/f 值组 {−1,2,3,0} 变化（avg 0.25–2.83） | 初值 **A/B**；运动 **C** | **部分可接**：初始坐标可投；**运动 fail-closed**（需脚本 VM 解码卡） |

> **地址口径更正（B）**：任务卡写 `$4E` handler `bank02:$AE44`；实际 dispatch 为 **target `$AE44` + `bank_switch=04`** ⇒ `$A000` 窗=**bank4**，PRG 偏移 `0xE44`（seed 视图 `bank04:$8E44`，file `0x8E54`）。`$2B/$2F` 为 type<0x30（`$8000` 窗=bank2）。

---

## 1. 方法与证据口径

- ROM 地址→file offset：`file = 0x10 + bank*0x2000 + (addr - window_base)`（bank2 窗 `$8000`、bank4 窗 `$A000`、bank14 窗 `$C000`）。
- 静态：`python tools/da6502.py rom/gunnac_cn.nes <fileoff_hex> <cpu_hex> <len_hex>`（逐指令）。
- trace A：`oracle/traces/gameplay-stage1.enemy.jsonl` 143 行（f1264–2400，每 8 帧；`state/x/y` 各 28B）。
- trace B：GUI 重采 `analysis/traces/gameplay-stage1.frames.jsonl`（f1–2400 **逐帧**，`w.iram`=1280B=`$0300-$07FF`）⇒ 槽 8–27 的 state/x/y 解码：`$0400+s → iram+0x100+s`、`$04E0+s → +0x1E0+s`、`$04FC+s → +0x1FC+s`。
- trace 对拍口径按 [enemy-coords-impl-spec.md](enemy-coords-impl-spec.md) §4.2：`V1_TYPES` 过滤、共同 (帧,型) 采样点、≤1px、分母先冻结。

---

## 2. `$2F` 字面量先例（A）

`bank02:$899B`（type `$2F` leader；无 bank switch）：

| 地址 | file offset | 字节 | 指令 | 语义 |
|---|---|---|---|---|
| `bank02:$89D5` | `0x049E5` | `A9 80` | `LDA #$80` | 目标值 = **128** |
| `bank02:$89D7` | `0x049E7` | `9D E0 04` | `STA $04E0,X` | **x = 128（字面量）** |
| `bank02:$89DA` | `0x049EA` | `A9 08` | `LDA #$08` | 目标值 = 8 |
| `bank02:$89DC` | `0x049EC` | `9D FC 04` | `STA $04FC,X` | **y = 8（字面量）** |

trace 验证：oracle `$2F` 5 槽（slots 23–27）首坐标全 `(128,8)`、8f 位移全 0；GUI 逐帧 5 槽 `dx=dy=0`（118–307 帧/槽）⇒ **字面量先例 A 级成立**（无 param/表依赖）。

---

## 3. `$2B`（type `0x2B`=43，member）handler 逐值审计

dispatch：type `$2B` → target **`$90AB`**（bank2，无 switch）；**判定（B）**：`$90AB` 首句把行为指针 `$041C/$0438` 置为 **`$9120`**（`LDA #$91→$041C` 高字节、`LDA #$20→$0438` 低字节；`$CB4F` trampoline 证实 `$0438`=低/`$041C`=高）⇒ **`$90AB` 为 init（只跑一次）、per-frame tick 入口 = `$9120`**；trace 支持：y 仅在 spawn 后首帧回到 8（若每帧跑 init 则 y 恒 8，与 trace y 递增矛盾）。

### 3.1 init 写点（`$90AB-$90F7`，A）

| 地址 | file offset | 字节 | 指令/值 | 语义 | 级 |
|---|---|---|---|---|---|
| `bank02:$90B5` | `0x050C5` | `A9 93` / `9D C4 04` | `LDA #$93; STA $04C4,X` | 运动使能位：bit0→y 积分、bit1→x 积分（见 §3.3）| A |
| `bank02:$90BA` | `0x050CA` | `A9 00; 9D A4 05` | `STA $05A4,X` | **vy_hi=0** | A |
| `bank02:$90BF` | `0x050CF` | `9D 50 05` | `STA $0550,X` | **vx_lo=0** | A |
| `bank02:$90C2` | `0x050D2` | `9D 6C 05` | `STA $056C,X` | **vx_hi=0** | A |
| `bank02:$90C5` | `0x050D5` | `9D DC 05` | `STA $05DC,X` | 相位计数=0（`$CD9D` 用） | A |
| `bank02:$90C8` | `0x050D8` | `9D C0 05` | `STA $05C0,X` | 相位亚像素=0 | A |
| `bank02:$90CB` | `0x050DB` | `A9 08; 9D FC 04` | `LDA #$08; STA $04FC,X` | **y=8（字面量）** | A |
| `bank02:$90D0` | `0x050E0` | `A9 90; 9D 88 05` | `LDA #$90; STA $0588,X` | **vy_lo=0x90=144** ⇒ 0.5625 px/f | A |
| `bank02:$90D5` | `0x050E5` | `20 99 C5` | `JSR $C599` | **x := $0470 param**（若非 0）：`$C599`=`LDA $0470,X; BEQ +; STA $04E0,X; LDA #0; STA $0470,X; ORA #$FF; RTS`（file `0x1C5A9`） | A |
| `bank02:$90EE` | `0x050FE` | `20 F7 C4` / `A9 18` / `90 02` / `A9 E8` / `9D E0 04` | `JSR $C4F7`（`INC $2D; LDA $2D; LSR A` → C=$2D&1）；`LDA #$18; BCC; LDA #$E8; STA $04E0,X` | **param=0 时 x=24(0x18) 或 232(0xE8)，按 `$2D` 奇偶**（A） | A |
| `bank02:$90DA-$90E9` | `0x050EA` | `BD E0 04 4A 90 1A 0A 10 04 E9 2F B0 02 69 30 9D F8 05` | 目标 `$5F8` 计算 | **偶 x⇒$80；奇 x⇒A=(x&0xFE)，A<0x80⇒A+$30，否则 A−$2F**（A 级逐指令） | A |

### 3.2 tick 写点（`$9120-$914A`，A）

| 地址 | file offset | 指令/值 | 语义 | 级 |
|---|---|---|---|---|
| `bank02:$9120` | `0x05130` | `LDA #$20; JSR $CD63` | **vx 向目标 `$5F8` 调整：每帧 ±0x20（0.125 px/f²）**；`$CD63` 写 `$0550/$056C`（file `0x1CD73`）| A |
| `bank02:$9125` | `0x05135` | `LDA $05DC; BMI` / `LDA #$30; LDY #$04; JSR $CD9D` | 相位振荡：`$5C0+=#$30`，进位 `INC $05DC`，到 4 回卷（`$CD9D` file `0x1CDAD`） | A |
| `bank02:$9131` | `0x05141` | `$914B/$914F/$832E` 表 → `$518/$684/$534` | 形态/动画/翻转（非坐标） | A |
| `bank02:$9145` | `0x05155` | `JSR $CC51` | 动画脚本 + **坐标积分**（若 `$04C4` bit1/bit0 置位调 `$CD08`/`$CCD3`） | A |
| `bank02:$9148` | `0x05158` | `JMP $D60E` | 命中/状态转移尾 | A |

### 3.3 速度对与运动积分（A 级公式；与 game-core `motion_update` 同构）

- **y**：`$CCD3`（file `0x1CCE3`）：`$048C += $0588`；`A=$05A4`；若 A≥0：`A+=$04FC`（y）；`carry` 或 y≥`$F0` ⇒ Offscreen，否则回写 y；A<0 时对称（借位 ⇒ Offscreen）。
  ⇒ 以 `$2B` init 值：`$0588=$90,$05A4=$00` ⇒ **y 每帧 +144/256=0.5625 px，永不改速**。
- **x**：`$CD63`（file `0x1CD73`）：`if x < $05F8: $550 += A; $56C += carry`；`if x > $05F8: $550 -= A; $56C -= borrow`（A=`#$20`）；`$CD08`（file `0x1CD18`）：`$0470 += $0550`（取进位）；`x += $056C + carry`（溢出/借位⇒Offscreen）。
  ⇒ x 为“向目标点 `$5F8` 的恒加速（0.125 px/f²）振子”；因 vx 从 0 加速，过冲幅度≈初始距离 ⇒ 在 32..224 区间往复（与 trace 一致，见 §3.4）。

### 3.4 trace 对拍（A/B；**F1 更正：按 spawn 分段 + 槽复用重置**）

**静态侧（oracle 8f，143 行）**：`$2B` 9 槽，首样本 x = **32×4（slots8/9/12/13）、33(slot11)、35(slot10)、231、24、48**（F3：与 GUI 分开列；复核 F3 记 oracle "32×5"——若把 slot11 的 **33** 归入 32 组即得 32×5+35（9 槽），逐槽原始值见 §6-5；两口径差异=slot11 是否归 32 组，待点验统一）；y=8–11；**dy8 median=5（4–5）全槽**；dx8 median 随槽 −15..+14（往复）。
**GUI 逐帧（f1–2400）——槽复用分段（A）**：同一 (type,slot) 会被**多次 spawn 复用**（type 值不变、对象已回收重发；旧占用尾帧会残留 x/y，如 slot8 f1469=(102,26)、f2031=(97,66)；slot9 f1863=(34,4)、f2001=(183,69)）。

| 槽 | 旧占用尾帧（残留） | 新 spawn 首帧（y=8） |
|---|---|---|
| slot8 | f1469 / f2031 | **f1470 / f2032** |
| slot9 | f1479 / f1863 / f2001 | **f1480 / f1864 / f2002** |
| slot10 | f1489 | f1490 |
| slot11 | f1499 / f1853 / f1981 | f1500 / f1854 / f1982 |
| slot12 | f1509 / f1873 / f2011 | f1510 / f1874 / f2012 |
| slot13 | f1519 / f1883 / f2021 | f1520 / f1884 / f2022 |
| slot16/17/18 | — | f1894 / f1904 / f1992（各 1 段） |

**按 spawn 分段仿真（A）**——每段 init y=8、vy pair=`($588=$90,$5A4=$00)`，**首记录帧=init 后 1 tick（y-subpix 已进 0x90）**；以 `subpix0=0x90` 起仿 **24 段全部 mismatch=0**（n=24–112/段）：

| 槽 | 各 spawn 段（f 起-止 / n / init x） | 每段 mismatch |
|---|---|---|
| slot8 | 1342-1373/32/32；1470-1577/108/191；2032-2125/94/48 | 0 / 0 / 0 |
| slot9 | 1352-1381/30/32；1480-1591/112/191；1864-1972/109/24；2002-2093/92/48 | 0 / 0 / 0 / 0 |
| slot10 | 1362-1388/27/32；1490-1513/24/191 | 0 / 0 |
| slot11 | 1372-1402/31/32；1500-1609/110/191；1854-1889/36/231；1982-2072/91/48 | 0 / 0 / 0 / 0 |
| slot12 | 1382-1413/32/32；1510-1620/111/191；1874-1982/109/231；2012-2045/34/48 | 0 / 0 / 0 / 0 |
| slot13 | 1392-1420/29/32；1520-1630/111/191；1884-1993/110/24；2022-2115/94/48 | 0 / 0 / 0 / 0 |
| slot16/17/18 | 1894-2004/111/231；1904-2011/108/24；1992-2086/95/48 | 0 / 0 / 0 |

- **整槽连续（不按 spawn 重置）仿真必失配**（复核口径：slot8 **203/236**、slot9 **315/346**、slot10 **24/52**、slot11 **239/271**、slot12 **256/289**、slot13 **317/347**；本卡复算同量级，差异=首帧/子像素初相位口径）⇒ **"每槽逐帧全等"原表述作废，改"每 spawn 段全等"**。
- **slot9 f1863–2001 段 110 mismatch 解释（A）**：该窗口 = 旧占用尾帧 **f1863（y=4）** + 新 spawn **f1864–1972（init x=24,y=8，109 帧）** + 残留 **f2001（y=69，f2002 才再 spawn）**。若把 f1863 的 y=4 当新 spawn init、不在 f1864 复位，则整窗逐帧失配（≈110）；**按 spawn 边界重切后新 spawn 段 109 帧 0 mismatch** ⇒ 不是 vy 模型失效，是**未处理槽复用重置**。
- **x**：GUI 首次有效 x = **32×6（slots8–13）+ 231 + 24 + 48**（F3 与 oracle 分开）；复用段 init x∈{191,231,24,48}（=param 192 / param=0 字面 232 / 24 / 48 的 1-tick 后值）⇒ 与 §3.1 param/字面量规则一致；dx/f 呈加速/换向振荡（`$5F8` 目标规则）。
- **预实现 surrogate**：oracle 8f × GUI 逐帧同 (帧,slot,type) 共 **1,090** 样本、mismatch=0——这是**逐槽交叉一致性旁证**，**不是 spec §4.2 正式验收**（见 §5）。

**与 sched 生成侧交叉**（`$A26D` 43 记录，ctrl2 Spawn 的 `args=[member,?,?,param]`）：trace 窗口内 `$2B` 成员 param=`{32,192,0,48}`（ticks 33/49/97/113）⇒ 首段 init x=32、复用段 191/232(→231)/24/48 与规则一致（192 成员在 trace 首段未现，复用段 191 可对应）。
**级**：y 写点/字面量 **A**；y 逐帧模型 **A（24/24 spawn 段 0 mismatch，首帧=init+1tick）**；x 初值 **A**；x 轨迹公式 **A 指令 + B 观测**。**C3 必须按 spawn/reuse 事件重置**（§7），否则可见化会错。

---

## 4. `$4E`（type `0x4E`=78，member）handler 逐值审计

dispatch：type `$4E` → target **`$AE44` + bank_switch=04**（`$A000` 窗=bank4，PRG offset `0xE44`；file `0x8E54`）。

### 4.1 entry 写点（`$AE44-$AE65`，A）：**无直接 x/y/速度写**

| 地址 | file offset | 指令 | 语义 |
|---|---|---|---|
| `bank04:$AE44`（`$A000` 窗；seed `$8000` 窗视图 `bank04:$8E44`） | `0x08E54` | `LDA #$93; STA $04C4,X` | 运动使能（bit0/1 置位 ⇒ 每帧 x/y 积分） |
| `$AE49-$AE52` | `0x08E59` | `LDA #$03; LDY $0454,X; BEQ; LDA #$02; STA $06BC,X` | 节拍/状态 |
| `$AE55` | `0x08E65` | `JSR $C872` | **param→脚本初始化**（见 §4.2） |
| `$AE58` | `0x08E68` | `JSR $CB4F` | 行为状态指针 trampoline（`PLA/PLA/JMP ($24)`，非坐标） |
| `$AE5B` | `0x08E6B` | `JSR $C894` | **脚本逐帧推进**（→`$C64D`） |
| `$AE5E` | `0x08E6E` | `JSR $AE67` | 表 `$AE2C/$AE34/$AE3C`→`$684/$534/$518`（形态/翻转，非坐标） |
| `$AE61/$AE64` | `0x08E71` | `JSR $CC51` / `JMP $D60E` | 动画+积分 / 命中尾 |

### 4.2 坐标/速度写点（脚本解释器内，A 地址）

| 写点 | file offset | 语义 | 级 |
|---|---|---|---|
| `bank14:$C608` 的 `$C61B/$C63B`（`STA $04FC`）与 `$C626/$C635`（`STA $04E0`） | `0x1C62B/0x1C64B/0x1C636/0x1C645` | **从脚本流读 [x,y] 写坐标**；dir（`$0470&0x80`，存 `$0710`）翻转 x：x=0→0/FF，或 x=byte、dir⇒x=−x | A |
| `bank14:$C872` | `0x1C882` | `param = $0470`；`dir=$0470&0x80→$0710`；`idx=param&0x7F`；**`$C8A7[idx]`→脚本偏移**；`$14/$15=$C8B7`；`JSR $C608`；`$06D8:=idx` | A |
| `bank14:$C64D` 写 `$0550/$056C` | `$C771/$C776/$C77C/$C781`（file `0x1C781/0x1C786/0x1C78C/0x1C791`） | **脚本命令设置 vx 对** | A |
| `bank14:$C64D` 写 `$0588/$05A4` | `$C78C/$C791/$C7B1/$C7B4` + `$C7B7/$C7BA/$C7F3/$C7F8/$C7FD/$C803`（file `0x1C79C…0x1C813`） | **脚本命令设置 vy 对** | A |
| 目标 `$5F8/$614` | `$C679/$C6C2/$C6C8/$C72E/$C748/$C6E4…` | 脚本命令设置目标/时长 | A |

静态数据：
- `$C8A7` 16B 偏移表：`00 19 23 30 47 58 62 6D 78 7F 8B 96 9F AA B5 C4`；
- 脚本基址 `$C8B7`（首 64B）：`40 08 41 0C 1E 00 18 …`；
- 表项首字节（idx0..15）: `40,20,20,04,04,50,18,04,60,10,20,70,40,08,20,30`、第二字节多为 `08`（y=8）。

### 4.3 trace 对拍（A/B）

- **初值可证（A/B）**：本关 `$4E` 成员 param（sched ctrl2 args[3]）=**{9,137,6,134}**（ticks 65/81/137/145）→ `&0x7F`=**{9,6}**→ `$C8A7[9]=0x7F`、`[6]=0x62` → 脚本首字节 `[10,08]`/`[18,08]` ⇒ **x=16 或 24（dir bit7 翻转⇒240/232）、y=8**。oracle 8f 首样本 x=**16/240/232**（slots 8–12=16、13–15=240、16=232）、y=15–27（=8+8f×vy≈2 px/f 级） ⇒ **一致（A/B）**。
- **运动不可闭式（C）**：GUI 逐帧 dy/f 值组：slot8 `{-1:172,+2:119,+3:103,0:24}`、slot10 同族、slot13 `{+3:65,+2:55}`、slot14 `{-1:129,+2:52,+3:27}`…（avg 0.25–2.83）⇒ 脚本命令逐段改速度/换向；**命令编码/时序未全解**，不能用常量/单一速度对投影。
- **参数域风险（C）**：`param&0x7F` 若 >15 则 `$C8A7` 越界读（表仅 16B）；trace 窗口外 sched 记录有 param=64/132/11/129 等（其中 64→越界），语义未证。

### 4.4 结论（fail-closed 边界）

- **初始坐标**：可静态投影（表 `$C8A7`+脚本首 [x,y]+dir 翻转；A/B，trace 一致）；
- **速度/运动**：写点在场（`$C64D` A 地址），但**脚本命令集/时序/param 域未证（C）** ⇒ **不可静态投影**；
- ⇒ C3 对 `$4E`：**最多接"spawn 初始位置可见"**；运动保持 fail-closed（不发明速度曲线），并另立脚本 VM 解码卡（`$C64D` 命令集 + `$C8A7/$C8B7` 表 + param 域）。

---

## 5. trace 对拍口径（V1_TYPES）与所需判据

- **F2 口径更正**：原"1,090/0"是**预实现 surrogate**（oracle 8f × GUI 逐帧**逐槽同 (帧,slot,type)** 交叉；本卡 §3.4），**不是 spec §4.2 正式验收**。spec §4.2 正式口径：`V1_TYPES={$2F,$2B,$4E}` 过滤、**共同 (帧,型) 采样点 ≥20**（复核方独立计=**501**，本卡未独立重算，待实现后冻结）、**分母先冻结**、**≤1px ≥90%**、且 **Rust 侧存在**（C3 实现后）；正式验收**留 C3 实现后**。
- 预实现 surrogate 结果（旁证，不入正式达成率）：oracle∩GUI 逐槽 **1,090** 样本、坐标 **mismatch=0**；`$2B` **按 spawn 分段 24/24 段 0 mismatch**（§3.4）；`$4E` 初值 x/y 与脚本表一致（A/B）。
- C3 实现后建议验收（按 per-type 表，冻结分母后逐型给 样本数/达成率/%≤1px）：
  ① `$2B` y 列 0 容差（含 spawn/reuse 重置后逐帧）；② `$2B` x 列 ≤1px ≥90%；③ `$4E` 仅初值（同帧首现 ≤1px），运动列标 `fail-closed`、**不入达成率分母**（V1_TYPES 过滤下对 `$4E` 单列"未实现"）。

---

## 6. 复现命令（只读；PowerShell 可执行）

```powershell
# 1) 2F 字面量先例（file 0x049E5）
python tools/da6502.py rom/gunnac_cn.nes 049E5 89D5 0C
# 2) 2B init/tick（file 0x050BB / 0x05130）
python tools/da6502.py rom/gunnac_cn.nes 050BB 90AB 50
python tools/da6502.py rom/gunnac_cn.nes 05130 9120 1C
# 3) 4E entry（bank4 sw: file 0x08E54）+ 脚本解释器
python tools/da6502.py rom/gunnac_cn.nes 08E54 AE44 22
python tools/da6502.py rom/gunnac_cn.nes 1C5A9 C599 10
python tools/da6502.py rom/gunnac_cn.nes 1C882 C872 22
python tools/da6502.py rom/gunnac_cn.nes 1C618 C608 42
python tools/da6502.py rom/gunnac_cn.nes 1CD73 CD63 1A
python tools/da6502.py rom/gunnac_cn.nes 1CD18 CD08 21
python tools/da6502.py rom/gunnac_cn.nes 1CCE3 CCD3 26
python tools/da6502.py rom/gunnac_cn.nes 1CDAD CD9D 12
# 4) 表：$C8A7 / $C8B7（bank14）
python -c "d=open('rom/gunnac_cn.nes','rb').read()[0x10:0x10+0x20000];print('C8A7:',' '.join('%02X'%d[14*0x2000+(0xC8A7-0xC000)+i] for i in range(16)));print('C8B7:',' '.join('%02X'%d[14*0x2000+(0xC8B7-0xC000)+i] for i in range(32)))"
# 5) oracle 8f 统计（2B/4E/2F）
@'
import json,statistics,collections
rows=[json.loads(l) for l in open('oracle/traces/gameplay-stage1.enemy.jsonl',encoding='utf-8') if l.strip()]
tr=collections.defaultdict(list)
for r in rows:
    st=bytes.fromhex(r['state']); x=bytes.fromhex(r['x']); y=bytes.fromhex(r['y'])
    for s in range(8,28):
        if st[s]: tr[(st[s],s)].append((r['f'],x[s],y[s]))
for t in (0x2B,0x4E,0x2F):
    print('type $%02X'%t)
    for k in sorted([k for k in tr if k[0]==t]):
        v=tr[k]; dy=[b[2]-a[2] for a,b in zip(v,v[1:]) if b[0]-a[0]==8]
        print(' slot',k[1],'first',v[0],'dy8_med',statistics.median(dy) if dy else None)
'@ | python -
# 6) GUI iram 解码 + 2B y **按 spawn 分段**仿真（iram 1280B=$0300-$07FF：state+0x100/x+0x1E0/y+0x1FC）
@'
import json,collections
seq=collections.defaultdict(list)
for l in open('analysis/traces/gameplay-stage1.frames.jsonl',encoding='utf-8',errors='replace'):
    try: r=json.loads(l)
    except Exception: continue
    h=r['w'].get('iram');  b=bytes.fromhex(h) if h else b''
    if len(b)!=0x500: continue
    st=[b[0x100+s] for s in range(28)]; xs=[b[0x1E0+s] for s in range(28)]; ys=[b[0x1FC+s] for s in range(28)]
    for s in range(8,28):
        if st[s]: seq[(st[s],s)].append((r['f'],xs[s],ys[s]))
def segments(v):
    segs=[]; cur=[]; prev=None
    for p in v:
        f,x,y=p; new=prev is None
        if prev is not None:
            pf,px,py=prev
            new = (f-pf!=1) or (y==8 and x!=0 and py!=8) or abs(x-px)>40 or (y-py)<-3
        if new and cur: segs.append(cur); cur=[]
        cur.append(p); prev=p
    if cur: segs.append(cur)
    return segs
for k in sorted([k for k in seq if k[0]==0x2B]):
    for seg in segments(seq[k]):
        if len(seg)<10: continue
        sub=0x90; yy=seg[0][2]; bad=0   # 首记录帧=init 后 1 tick
        for f,x,y in seg:
            if yy!=y: bad+=1
            sub+=0x90; carry=1 if sub>255 else 0; sub&=0xFF; yy=(yy+carry)&0xFF
        print('slot%2d f%d-%d n=%d init=(%d,%d) mismatch=%d'%(k[1],seg[0][0],seg[-1][0],len(seg),seg[0][1],seg[0][2],bad))
'@ | python -
```

---

## 7. 不确定项 / 未做（显式）

1. **槽复用重置（F1；C3 必做）**：`$2B` init 只写 `$04FC/$0588/$05A4/$0550/$056C/$05DC/$05C0`，**未显式清 `$048C`（y 子像素）**；`$C599` 在 `$0470==0` 时**也不清 `$0470`（x 子像素）**。trace 24/24 spawn 段均以 `y-subpix=0x90`（=init 后 1 tick）起仿 0 mismatch，但**复用/重发时子像素/相位是否被 generation 清零未经 ROM 证据闭合（B）** ⇒ C3 建议：**spawn/reuse 时显式复位 y=8、vy pair、x/y 子像素（`$048C/$0470`）、`$05DC/$05C0` 相位、目标 `$5F8`**，并以"首帧=init 后 1 tick"为采样口径；否则槽复用会造成可见坐标错位（本卡 F1 的教训）。
2. **$4E 脚本命令集未全解（C）**：`$C64D` 命令编码/时序只解出"含 vx/vy 写点与 $5F8/$614 目标写点"，逐命令语义（0x20/0x40/0x80…）与脚本表全展开未做；因此 `$4E` 运动**不可静态投影**。
3. **param 域（C）**：`param&0x7F>15` 时 `$C8A7` 越界读（表 16B）；本关 trace 窗口内 params 仅 {9,137,6,134}（idx 9/6）在域内；窗口外记录（param=64/132/11/129 等）语义未证。
4. **$2B x 轨迹**：公式（`$5F8` 目标+`$CD63` 步长 `$20`+`$CD08` 积分）为 A 级指令；与 trace 的逐帧等价未做（留 C3 实现后按 §4.2 正式对拍）；`$0470` 的 leader 复制链（`$8A1A/$8A1D`）为 A，leader 的 param 来源=sched ctrl2 args[3]（B）。
5. **地址口径**：任务卡 `bank02:$AE44` 与 dispatch（bank_switch=04）冲突，本文按 **bank4 offset 0xE44** 引用；若后续复核确认其他窗口视图，需注记（B）。
6. **动态缺口**：本机只读，未重跑 Mesen；GUI trace 为 task-133 采集产物（analysis/ gitignored），oracle trace 为 tracked 8f 采样；逐帧结论基于该 GUI trace。
7. **F2 正式验收未做（口径警示）**：§5 的 1,090/0 是**预实现 surrogate**，不得作为 §4.2 达成率引用；正式验收（V1_TYPES/共同(帧,型)≥20〔复核计 501〕/冻结分母/≤1px≥90%/Rust 侧）留 C3 实现后。
8. **F3 计数口径待点验**：oracle 原始逐槽首样本 = **32×4（slots8/9/12/13）+ 33(slot11) + 35(slot10) + 231 + 24 + 48**；复核 F3 记 "32×5 + 35 + …"，若把 slot11 的 **33** 归入 32 组则完全一致（32×5+35=6 槽 + 3 槽=9）——差异仅在归组口径，本文按原始值列、待点验统一。
9. 未改代码/expected/门禁；未跑 gates check/status/visual；本文件唯一写入。

---

## 8. 自查

- 只写本文件；未 git add/commit；未改 `crates/`/`data/`/`gates/`/`rom/`/`reference/oracle`。
- 级：§2/§3.1/§3.2/§3.3/§4.1/§4.2（写点/值/公式）A；§3.4 **y 按 spawn 分段 24/24 段 0 mismatch（A）**、整槽连续 mismatch 属**未重置反例（A 观测）**、x 轨迹 A 公式+B 观测；§4.3 初值 A/B、运动 C；§0 分档表据此。
- 数字可复算：file offset/bytes/指令逐条给；trace 统计/分段仿真命令见 §6；`$2B` 分段 mismatch、F3 两数（oracle 32×4+33+35+231+24+48 / GUI 32×6+231+24+48）、surrogate 1,090/0 均可重跑。
- 对抗复核建议优先证伪：(a) `$2B` init y/vy 字面量与 `$C599` x 参数链；(b) `$2B` 目标 `$5F8` 公式与 `$CD63` 步长；(c) `$4E` 脚本初值映射（param→`$C8A7`→`$C8B7`→`$C608`）与 dir 翻转；(d) `$4E` 运动 fail-closed 是否成立；(e) 地址口径 bank4 vs 任务卡 bank02；(f) **F1 槽复用重置条件与 slot9 f1863–2001 段解释**；(g) **F2 surrogate vs §4.2 正式口径、F3 oracle/GUI 首 x 计数**。

【2026-10-19】decomp-analyst，task-162（基于 task-157，修正 F1/F2/F3）。
