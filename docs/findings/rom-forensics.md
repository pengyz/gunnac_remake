# S0 ROM 取证报告 — rom/gunnac_cn.nes（九班汉化版《加纳战机/Gun-Nac》）

> 纪律：每条结论分 **已证实 / 假设** 两节（AGENTS.md §3）。已证实项均带字节偏移指针与
> 可复现命令（hexdump/python）。假设项不得作为后续 rename/规格依据。
> 反汇编辅助工具：[tools/da6502.py](../../tools/da6502.py)、CHR 渲染：[tools/chr_render.py](../../tools/chr_render.py)
> （均为**可再生投影**，ROM 字节是最终真相）。

## 0. 文件元信息与偏移约定

| 项 | 值 |
|---|---|
| 路径 | `rom/gunnac_cn.nes`（只读归档，262160 = 0x40010 字节） |
| SHA256 | `4d76901c1b2455f2bdae8e8ac99b1779441f9f2155614dab222c229ae06c77ae` |
| SHA256SUMS | 根目录 [SHA256SUMS](../../SHA256SUMS)（已含该哈希，`sha256sum -c SHA256SUMS` 可复验） |
| 结构 | iNES 头 16B（file 0x0-0xF）+ PRG 128KB（file 0x10-0x2000F）+ CHR 128KB（file 0x20010-0x4000F） |

**偏移换算约定（全文通用）**：

- `PRG offset` = file offset − 0x10。8K PRG bank `k`（k=0..15）= PRG 0x2000×k。
- `CHR offset` = file offset − 0x20010。1K CHR bank `n`（n=0..127）= CHR 0x400×n = file 0x20010+0x400×n。
- CPU 地址取决于映射窗口（见 §2）；`bank15:$XXXX` 特指固定窗 $E000-$FFFF（CPU = PRG − 0x10000）。

复现（哈希与头）：

```
$ sha256sum rom/gunnac_cn.nes
4d76901c1b2455f2bdae8e8ac99b1779441f9f2155614dab222c229ae06c77ae  rom/gunnac_cn.nes
$ xxd -l 16 rom/gunnac_cn.nes
00000000: 4e45 531a 0810 4100 0000 0000 0000 0000  NES...A.........
```

## 1. iNES 头解析

### 已证实（A 级，file 0x0-0xF）

| 字节 | 值 | 含义 |
|---|---|---|
| 0x0-0x3 | `4E 45 53 1A` | iNES magic |
| 0x4 | `08` | PRG = 8×16KB = **128KB** |
| 0x5 | `10` | CHR = 16×8KB = **128KB** |
| 0x6 | `41` | mapper 低 4 位 = 4；bit0=1 → **垂直镜像（vertical）**；bit1=0 无电池 RAM；bit2=0 无 trainer；bit3=0 非 4-screen |
| 0x7 | `00` | mapper 高 4 位 = 0；(bit2-3)=00 → iNES 1.0（非 NES2.0） |
| 0x8-0xF | 全 0 | PRG-RAM size byte = 0（iNES1 惯例按 8KB 计） |

→ **Mapper 4（MMC3），PRG 128K + CHR 128K，垂直镜像**。文件大小 262160 = 16+0x20000+0x20000 与头声明自洽。

### 假设（B/C 级）

- PRG-RAM 8KB（$6000-$7FFF）：由头字节 0x8=0 的 iNES1 惯例 + 复位代码读写 $07FE/$07FF（bank15:$E022）推断。**B 级**。
- 运行时镜像实际取自 MMC3 $A000 写入（复位 bank15:$E07E `LDA #$00 / STA $A000` → 与头一致 vertical）。写值语义按 MMC3 标准解释属 **B 级**。

## 2. MMC3 bank 布局与 $8000-$FFFF 映射窗口

### 已证实（A 级）

**PRG = 16 × 8KB bank**（bank k = PRG 0x2000k）。全部 `$8000` bank-select 写入的立即数为 0x06/0x07（PRG 寄存器 R6/R7）或 0x00/0x01（CHR 寄存器 R0/R1），**bit6=0 → PRG mode 0，bit7=0 → CHR A12 不反转**。

| CPU 窗口 | 映射（mode 0） | 证据 |
|---|---|---|
| $8000-$9FFF | R6（8K 可切换） | bank15:$F001 `LDA #$06/STA $8000` 后 `LDA #$0B/STA $8001/JSR $8009`（R6=$0B 时执行 $80xx 代码） |
| $A000-$BFFF | R7（8K 可切换） | bank15:$EFE0 `LDA #$07/STA $8000` + 数据写 |
| $C000-$DFFF | 固定倒数第 2 个 8K = **bank 14**（PRG 0x1C000） | mode 0 标准行为；复位后未见对 $C000-$DFFF 的代码跳转 |
| $E000-$FFFF | 固定最后 8K = **bank 15**（PRG 0x1E000） | RESET/NMI/IRQ 向量均落此窗（§3） |

**far-call 跳板约定（banked call）**：`JSR $E649` 后紧跟 4 个内联字节 `(R6值, R7值, 目标lo, 目标hi)`；
跳板（bank15:$E649-…，PRG 0x1E649）取回返回地址读出 4 字节、经 $E61F（R6=A）/$E62E（R7=A）换 bank、
调用目标、返回前恢复。全 ROM 共 **18 处** `20 49 E6` 调用点（复现见 §7 python 扫描）。示例：

| 调用点 (PRG) | R6 | R7 | 目标 | 说明 |
|---|---|---|---|---|
| 0x1E087（bank15:$E087） | $0B | $0C | $8000 | 复位主流程 far-call 进主程序 bank 对 |
| 0x1E0AF | $00 | $01 | $8DDF | bank0/1 单元 |
| 0x01CDC（bank0） | $02 | $03 | $9698 | bank2/3 单元 |
| 0x0E8FB（bank7） | $00 | $01 | $8807 | |
| 0x1F254 | $0B | $0C | $800C | |

**far-call 目标 = bank 11 入口跳转表**：bank 11 = R6=$0B 时映射 $8000，其开头即 JMP 表
（PRG 0x16000，`xxd -s 0x16010 -l 24 rom/gunnac_cn.nes`）：

```
CPU $8000: 4C A3 88  JMP $88A3      CPU $800C: 4C 94 88  JMP $8894
CPU $8003: 4C A7 88  JMP $88A7      CPU $800F: 4C 9B 88  JMP $889B
CPU $8006: 4C 0B 89  JMP $890B      CPU $8012: 4C 2C 8E  JMP $8E2C
CPU $8009: 4C 16 80  JMP $8016      CPU $8015: 60        RTS
```

与 far-call 目标 $8000/$800C/$800F/$8012 **逐一吻合**（A 级）；NMI 尾部 `JSR $8009`（bank15:$F009）落 $8016 帧例程。

**MMC3 寄存器写入统计**（PRG 内 `8D xx xx` 绝对寻址扫描）：
`STA $8000`×33、`STA $8001`×32、`STA $A000`×4、`STA $A001`×2、`STA $C000`×6、`STA $C001`×1、
`STA $E000`×5、`STA $E001`×2（含义依次：bank 选择/数据、镜像、PRG-RAM 保护、IRQ 计数 latch/reload、IRQ 禁/启）。

**bank 影子寄存器（RAM）与中断换 bank**：

- `$01AA`/`$01AB` = NMI/IRQ 回调使用的 R6/R7 影子：NMI 尾部 bank15:$EFD5-`STA $8000`/`LDA $01AA`…`LDA $01AB`，IRQ bank15:$F21C 同款。
- `$00AC/$00AD` = NMI/IRQ 共用回调向量：bank15:$F251 `JMP ($00AC)`（NMI 经 $EFF3、IRQ 经 $F232 调用）。
- `$01A3,Y` 影子 + `bank15:$E63D`（`STY $22 / STY $8000 / STA $8001`，Y=寄存器号 A=bank 号）为**索引式换 bank**，23 处调用点（PRG 0x007D6、0x012B4-0x012C9、0x0425E、0x06513、0x0E9F2、0x1D9E3、0x1E3D9-0x1E3E7 等）。
- CHR 寄存器使用实录（A 级字节）：
  - R0/R1 立即数对：($7C,$7E)、($36,$38)、($14,$16) — PRG 0x00FB1-0x00FC0、0x00FD0-0x00FDF、0x02104-0x02125、0x02B60-0x02B6A。
  - 场景装载（PRG 0x012C0-0x012D9，bank0:$92C0）：`R3=$66, R4=$7F, R0=$7C, R1=$7E`（经 $E63D）。
  - R2-R5 装载环（PRG 0x02AEC，bank1:$AAEC）：`LDY #$02 / STY $8000 / LDA $AB11,Y / STA $8001 / INY / CPY #$06`，表 CPU $AB11（PRG 0x02B11）+2 起取 `08 09 0A 0B`（`xxd -s 0x2B21 -l 8 rom/gunnac_cn.nes`）→ R2-R5 = CHR 1K bank 8,9,10,11。
- 复位（bank15:$E000）关键序列：SEI/CLD/TXS → 关渲染等 3 帧 → $4015=$0F、$4017=$40 →
  温启动签名比对 `$07FE=$AC,$07FF=$53`（bank15:$E022）→ 清 RAM → $E60C（R6=0）→ …
  → `$A001=$00`（PRG-RAM 使能，bank15:$E06A）→ `$E000=$00`（IRQ 禁，bank15:$E06D）→ far-call($0B,$0C,$8000)。

### 假设（B/C 级）

- `$01A9` = 帧/分屏计数器，配合交替装载 R0/R1 = ($7C,$7E) 与 ($36,$38) 并写 IRQ latch（PRG 0x00FB5-0x00FDF，`LDA $01A9/INC/…/STA $C000=$30`）→ **推断为 IRQ 分屏 CHR 切换/状态栏**。B 级（静态推断，待 Mesen2 trace）。
- bank 4-7、8-10、13-14 的运行时映射位置由数据表驱动（$01A3,Y / $0400/$0401 / $01AA/$01AB 运行时赋值），静态未穷举。C 级。
- 8K bank 用途初分类（JSR 密度 + 熵，python 复现见 §7）：bank 0-7、11、14、15 = 含代码；bank 8-10 = 纯数据候选；bank 12-13 = 数据（音乐/关卡）候选。B/C 级。

## 3. 向量表 $FFFA-$FFFF

### 已证实（A 级）

```
$ xxd -s 0x20000 -l 16 rom/gunnac_cn.nes
00020000: 0000 0000 0000 0000 0a00 85ef 00e0 0bf2  ................
```

file 0x2000A-0x2000F = `85 EF 00 E0 0B F2`（= PRG 0x1FFFA，CPU $FFFA）：

| 向量 | CPU 地址 | 所在 bank |
|---|---|---|
| NMI ($FFFA) | **$EF85** | bank 15 固定窗 $E000-$FFFF（PRG 0x1EF85） |
| RESET ($FFFC) | **$E000** | bank 15（PRG 0x1E000 = 复位入口） |
| IRQ ($FFFE) | **$F20B** | bank 15（PRG 0x1F20B） |

- 三向量全部落在固定最后 8K bank 15（不受 R6/R7 影响），自洽。
- `bank15:$FFF8 = $0A`、`bank15:$FFF9 = $00`：复位代码 bank15:$E08E `AD F9 FF / D0 0C`（`LDA $FFF9 / BNE`）**读取 $FFF9 作为标志分支** —— 是专用标志字节而非向量/填充（A 级）。
- NMI（$EF85）：保存寄存器 → 写 $2001 关渲染 → `INC $23`（帧计数）→ `$E000`（禁 IRQ）→ OAM DMA（$4014=$02）→ `JSR $F030/$F0DC` → 恢复 $2001/滚动（$2005/$2000）→ 若 $01AD≠0 则 R6←$01AA、R7←$01AB → `JSR $F251`（`JMP ($00AC)` 回调）→ 弹栈恢复 R7/R6 → RTI。
- IRQ（$F20B）：`STA $E000`（应答/禁）→ 保存 → R6←$01AA、R7←$01AB → `JSR $F251`（`JMP ($00AC)`）→ 恢复 bank → RTI。

### 假设（B/C 级）

- `$FFF9` 是**版本/区域标志**（当前 0x00），复位时据此分支初始化差异。B 级（用途细节待 S1 反汇编 $E08E 分支两路）。
- `$23` 为全局帧计数、`$1B/$1A/$1C/$1D` 为渲染控制影子（NMI 中直写 $2001/$2000/$2005）。B 级。
- NMI 与 IRQ 共用 $00AC 回调向量，运行时由场景代码改写（B 级）。

## 4. PRG 字符串 / 文本表 / 关卡数据候选区（含汉化文本指针线索）

### 已证实（A 级，字节可 hexdump 复现）

**文本记录格式**（多处独立成立，B 级解释）：`{ppu_lo, ppu_hi, 字符/tile 码…, $FE 或 $FF 终止}`。
首 2 字节为小端 PPU nametable 地址，其后为待写 tile 码。

| 区域 | PRG 范围 | 内容（首记录实录） | 复现 |
|---|---|---|---|
| 标题/版权文本（bank0） | 0x0109E-0x01123 | `62 20 "HI SCORE" FE`；`72 20 "SCORE" FE`；`66 22 "1990 @TOKYO SHOSEKI" FE`；`A6 22 "PROGRAMED BY COMPILE" FE`；`E4 22 "PRESENTED BY NEXOFT CORP." FE`；`2A 23 "LICENSED BY" FE`；`64 23 "NINTENDO OF AMERICA INC." FF` | `xxd -s 0x10AE -l 144 rom/gunnac_cn.nes` |
| 字模 tile 码块（bank0） | 0x01155-0x0116F | `6C 22 E0 E1 E2 FE`；`8C 22 F0 F1 F2 FE`；`AC 22 E3 E4 E5 FE`；`CC 22 F3 F4 F5 FF`（**tile 码 $E0-$F5 → 汉字字模**，见 §5） | `xxd -s 0x1165 -l 32 rom/gunnac_cn.nes` |
| 商店文本（bank0） | 0x0151A、0x01525 | `"WEAPON-0"`、`"BOMBER-0"` | `xxd -s 0x1528 -l 32 rom/gunnac_cn.nes` |
| STAFF 字幕（bank0） | 0x01959-0x01BC0 | `"COMPILE STAFF"`、`"GAME DESIGN & PROGRAM"`、…、`"TOKYO SHOSEKI"`、`"PRESENTED BY NEXOFT CORP."`（格式：`{1 字节前缀, 文本, FE}`，与标题记录不同系） | `xxd -s 0x1969 -l 576 rom/gunnac_cn.nes` |
| **汉化组签名（bank0）** | 0x01BB1-0x01BBF | `FE 09 40 "2011  JIUBAN" FE FE FE FF`（"@2011  JIUBAN" = 九班汉化署名） | `xxd -s 0x1BC0 -l 32 rom/gunnac_cn.nes` |
| 开场剧情文本（bank1） | 0x02CDC-0x02F8B | `23 22 "IN THE VAST UNIVERSE,FAR" …`、`62 22 "AWAY,THERE IS A PEACEFUL" …` … `"GENERATING ALL THE THREATS...."`（每行 {PPU 地址, ASCII, FE}，行地址 $2223/$2262/$2263… 复用同一行 → 滚动文本） | `xxd -s 0x2CEC -l 160 rom/gunnac_cn.nes` |
| 结局文本（bank1） | 0x02FE4-0x03155 | `"THANKS TO GUN-NAC,"`…`"TO PRESERVE PEACE."`、`"OPTION DEMO"` | `xxd -s 0x2FF4 -l 352 rom/gunnac_cn.nes` |
| GAME OVER/续关文本（bank15） | 0x1E4B9-0x1E4E8 | `"HI SCORE"`、`"SCORE"`、`"CONTINUE"`、`"GAME OVER"` | `xxd -s 0x1E4C8 -l 48 rom/gunnac_cn.nes` |
| 大字模文本块（bank15 固定窗） | 0x1FF00-0x1FFEF | `"1990-12-17 USA"` + 大字模 tile 码块（"SSS EEE EEE … Y Y OOO U U …" = 宽字模排布）+ `" AGAIN!"`（$FFE0 起） | `xxd -s 0x1FF10 -l 240 rom/gunnac_cn.nes` |

**charmap / 字形索引表候选（bank0）**：

- PRG 0x01190-0x01220：连续递增 tile 索引流（`0A 0B 0C … 84`）带 `00` 间隙 —— 字符码→tile 索引表。
- PRG 0x01830-0x018F8：索引表片段与记录字节交错（`13 14 1B 1C …`、`00 00 4C 4D 4E 4F …`、
  ASCII 连续段 `20-5D`）。
- 表中出现非恒等映射段（如 `LMNOPQORSTUVWXYZ[\]^`、`,-cde1fghijklmn;<=>opqBrDEFGHstKuvwxyz{|}~`
  处乱序/换位）——与恒等段（0x018B2 起 `20-5D` 连续）并存。**汉化文本指针/映射线索**。

**汉化文本线索汇总（已证实的字节事实）**：

1. 署名 `@2011 JIUBAN`（PRG 0x01BB2）。
2. 文本记录中出现 tile 码 $E0-$F5（PRG 0x01155-0x0116F）超出 ASCII 区 —— 直指汉字字模（§5）。
3. charmap 表存在重排/乱序段（上表）——字符码到字模的映射被改写痕迹。
4. PRG 内英文原文（剧情/字幕）仍完整保留 —— 汉化以**字模替换 + charmap 重映射**为主（推断，见假设）。

### 假设（B/C 级，不得据此 rename）

- **记录表 PRG 0x01066-0x0109D**：8 字节记录 `{参数(2), 指针(2), PPU 地址(2), 例程 $EB1E(2)}`，
  末记录 `{00 00, $909E, 00 00, 00 00}` 中 **$909E 恰指标题文本起点 PRG 0x0109E**（强线索，B 级）；
  其余指针 $916F/$917D/$9187/$9229/$9243 的字段布局与落点解释未定。C 级。
- PRG 0x0185B/0x01863/0x01886 处 8 字节记录 `DE E9 08 0A 8F 98 46 20` 等：
  形如 `{例程 $E9DE(固定 bank), 参数, 指针 $988F, PPU $2046}`。B/C 级。
- PRG 0x02C10-0x02CDB：**脚本/事件数据候选**（重复记录 `04 FE 01 3C 04 FE 02 78 7A`、
  `03 DC AC` 疑似指向剧情文本 $ACDC）。C 级。
- PRG 0x16261（bank11）：**36 项指针表**（$8297,$82A2,$82C3,$82CF,$831F,$890A,…，均落 bank11 窗内）
  —— 对象/脚本处理子程序表候选。C 级。
- **关卡数据候选区**：bank 8-10（PRG 0x10000-0x16000）高熵、无 ASCII、含自引用指针字
  （bank 9 头重复 $A16F；bank 10 头 $A071/$A0D5/$A057/$A396/$A372…）→ 关卡/敌人布局表候选；
  bank 12-13（PRG 0x18000-0x1C000）周期性音符样数据 → 音乐/SFX 数据候选。均 **C 级**，待 trace 定性。
- 汉化方案 = 保留英文 tile 码 + charmap 映射到汉字字模 + 字模图形替换；剧情文本未见中文编码字节流。C 级。

## 5. CHR 布局与字模区

### 已证实（A 级）

- CHR = 0x20000 = **128 × 1KB bank**（MMC3 CHR 寄存器按 1KB 计）；bank n = CHR 0x400n = file 0x20010+0x400n。
- 全空（全 00/FF）1K bank：**42, 43, 47, 51, 64-71, 80-83**（python 逐 bank 统计，§7）。
- **字模区 = 末尾 4KB：CHR bank 124-127（file 0x3F010-0x40010）**，渲染（`tools/chr_render.py`）为
  **汉字字模**（可读字样如"开始设置你有导弹级 / 注意上限哪个键"、"该阶初地区声音武器 / 不能涡轮买 转换"）。
  渲染产物：`analysis/chr_bank124.png` … `analysis/chr_bank127.png`（可再生）。
  各 bank sha256 前 12 位：124=`33f2b619eaaf`、125=`48c49e1a88d8`、126=`6f7be5e8b8a6`、127=`c3d6d138c540`（互不相同）。
- 装载证据：R0=$7C / R1=$7E（= CHR bank 124-125 / 126-127 的 2K 页）出现在
  PRG 0x00FB3-0x00FC0（bank0:$8FB3）、0x02119-0x02125、0x02B60-0x02B6A；场景装载 PRG 0x012C0-0x012D9
  同时设 R3=$66、R4=$7F（bank 127 亦被 R4 装入 PPU $1C00-$1FFF）。
- 文本 tile 码 $E0-$FF 与 PPU BG 页 $1000（复位 `$2000=$20`）+ R4=$7F 组合下落点 = bank 127 偏移 0x200+
  （tile $E0 → 1K bank 127 +0x200），与汉字字模渲染吻合（A 级字节 + 渲染）。
- 其余文本/图形 CHR 装载：R2-R5 ← 表 `08 09 0A 0B`（PRG 0x02B13-0x02B16）、R0/R1 ← ($14,$16)、($36,$38)。
- 密度扫描（uniq 字节数/1K）高位 bank：2、15、36、60-62、89、107（图形密集）；bank 42/43/47/51/64-71/80-83 空。

### 假设（B/C 级）

- bank 124/127 呈现字样集 A、bank 125/126 呈现字样集 B（两套汉字字库，供 R0/R1 两个 2K 页使用）——
  汉化组将汉字字模放入 ROM 尾部 4KB。**B 级**（渲染相似、字节不等，具体字库-字模对应留 S2）。
- ASCII/大字 Latin 字模位置未定：候选在 CHR bank 102（R3=$66 场景）、bank 8-11（R2-R5 表）或图形密集 bank。**C 级**，S2 任务。
- 汉字按 16×16 = 2×2 tile 排布（C 级）。
- 空 bank（64-71 等）为原版留白或汉化未用空间（C 级）。

## 6. 遗留假设清单（不得作为结论/命名依据）

| # | 假设 | 级 | 建议去向 |
|---|---|---|---|
| H1 | 记录表 PRG 0x01066-0x0109D 字段布局（仅 $909E→0x0109E 落点为 B） | C | S1 反汇编文本引擎 |
| H2 | `DE E9 …` 8 字节记录语义 | C | S1 |
| H3 | 脚本/事件区 PRG 0x02C10-0x02CDB | C | S1 + Mesen2 trace |
| H4 | bank11 36 项指针表语义 | C | S1 |
| H5 | bank 8-10 = 关卡/敌人数据、bank 12-13 = 音乐数据 | C | S1 + trace |
| H6 | 汉化 = charmap 重映射 + 字模替换方案 | C | 与原版 ROM 对照（若可得）|
| H7 | ASCII 字模 CHR 位置 | C | S2 CHR 考证 |
| H8 | $01A9 IRQ 分屏 CHR 切换 | B | Mesen2 插桩 |
| H9 | $FFF9 = 版本/区域标志 | B | S1 分支两路反汇编 |

## 7. 复现命令附录

```bash
# 哈希与头
sha256sum rom/gunnac_cn.nes && xxd -l 16 rom/gunnac_cn.nes
# 向量表
xxd -s 0x20000 -l 16 rom/gunnac_cn.nes
# 复位入口（bank15:$E000 = PRG 0x1E000 = file 0x1E010）
python3 tools/da6502.py rom/gunnac_cn.nes 1E010 E000 120
# NMI/IRQ
python3 tools/da6502.py rom/gunnac_cn.nes 1EF95 EF85 60
python3 tools/da6502.py rom/gunnac_cn.nes 1F21B F20B 48
# far-call 跳板 $E649 与 bank 设置助手 $E60C/$E61F/$E62E/$E63D
python3 tools/da6502.py rom/gunnac_cn.nes 1E659 E649 60
# bank11 入口 JMP 表（R6=$0B → $8000）
xxd -s 0x16010 -l 24 rom/gunnac_cn.nes
# 标题文本记录
xxd -s 0x10AE -l 144 rom/gunnac_cn.nes
# 汉化签名
xxd -s 0x1BC0 -l 32 rom/gunnac_cn.nes
# 剧情/结局文本
xxd -s 0x2CEC -l 160 rom/gunnac_cn.nes; xxd -s 0x2FF4 -l 352 rom/gunnac_cn.nes
# CHR 字模渲染（bank 127 = file 0x3FC10）
python3 tools/chr_render.py rom/gunnac_cn.nes 3FC10 400 16 analysis/chr_bank127.png
# $8000/$8001 写入点与 far-call 点扫描、bank 熵/空 bank、指针表扫描
# （本报告 §2/§4/§5 所用 python 单行脚本；关键断言均可由上述 hexdump 直接复核）
```

## 8. 结论摘要

- **Mapper 4/MMC3，PRG 128K + CHR 128K，垂直镜像**；mode 0 映射：R6→$8000、R7→$A000、bank14 固定 $C000、bank15 固定 $E000。
- 向量 NMI=$EF85 / RESET=$E000 / IRQ=$F20B，全在 bank15 固定窗；$FFF9 为被复位读取的标志字节。
- 游戏以 far-call 跳板（`JSR $E649`+内联 (R6,R7,target)）跨 16KB 单元调用，bank11 $8000 起为入口 JMP 表。
- 文本以 `{PPU 地址, tile 码, FE/FF}` 记录分布于 bank 0/1/15；charmap 表 + tile 码 $E0-$F5 +
  CHR bank 124-127 汉字字模 + `@2011 JIUBAN` 署名构成汉化文本的完整线索链。
- 关卡/音乐数据候选在 bank 8-13，语义未定（假设区 H5）。
