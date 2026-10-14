# 对抗复核 #67：A3 结局段取证（fresh context·证伪立场）

> 被审：`docs/findings/ending-sequence.md`（399 行，A3 结局段：通关触发链 / staff roll→结局卡流程 /
> 三结局分支 / 结局卡机制 / 双版对照 / H7 定谳）。
> 审核方：独立 verifier #67（fresh context subagent，实施≠审核；不采信被审自述、不复用其结论、独立重导）。
> 纪律：AGENTS.md §5 对抗复核 + §3 证据分级 + §3.6 证伪更正；"未贴出的运行结果不存在"。
> 数据源（只读）：`rom/gunnac_cn.nes`（sha256 `4d76901c…06c77ae` 门禁锚定）、
> `reference/us/gunnac_us.nes`、`reference/jp/gunnac_jp.nes`、`reference/seed/gunnac.asm`、
> `analysis/traces/gameplay-stage1_*.ram.hex`（运行时锚点）、`docs/findings/*.md`、`docs/spec/stage-timeline.md`、`gates/`。
> 未改动任何既有文件；本文件为唯一产出。

**裁定：pass** —— 五项核心主张（通关触发链 / staff roll→结局卡 / 三结局=难度 / H7 美版自带署名 /
双版对照）经 **6 组独立重导 + 运行时 RAM 锚点**全部成立，**零结论证伪**；攻击点 1-4 全部裁定成立
（其中攻击点 2 的"唯一性"表述存在**负证据推理链缺环**，本审补扫后结论仍为真，记 AR67-1 非阻断必补注）；
H7 定谳维持。余 7 条 findings 全部为低危机械更正/口径统一/跨文档路由项（AR67-1..7），
无一影响结论或搬运正确性；AR67-2/3/4/7 建议按 §3.6 dated 原地更正后归档，AR67-5 移交 spec 侧。
gates 12/12 复跑全绿。

## 0. 发现总表（四分类，高危优先）

| ID | 分类 | 级 | 问题 | 处理 |
|---|---|---|---|---|
| AR67-1 | **证据缺陷（负证据推理链缺环，结论仍真）** | 低中 | §4.1"`⇒` **运行时改写只能经设置屏 `STA (4),Y` 间接写**"标为"A 级负结果"，但推理链只覆盖**直接写**字节族，未排除**其他间接写通道**；且所列扫描族 `85/86/84/E6/C6/9D/99 4C` 不含 `8D/95/96/D6/F6/EE/CE/FE/DE`。本审扩展 16 opcode 族复扫（三版）+ 全 PRG `4C 00` 指针数据扫描（仅 9 处，唯一 RAM 指针源=设置屏字段表 `bank00:$94D2`）+ 通用 zp 写循环排查（仅冷启动清零 `bank15:$E03C`）后**结论成立** | 补注推理链三环或降 A→B；**非阻断**（结论为真，见 §2.2） |
| AR67-2 | 规格缺口（界址越界 0x0A） | 低 | §2.2/§5.3 "staff roll 流本体 CPU `$9955-$9BCC`，file `0x1965-0x1BDC`"——本审独立流解析：流尾 `FF` 实在 **CPU `$9BC2` / file `0x1BD2`**，其后即 code（`ff a5 26 10 0c …`）；`$9BCC/0x1BDC` 越界 10B | dated 原地更正两处界址 |
| AR67-3 | 规格缺口（地址偏 3） | 低 | §1.1 "`bank15:$E1B8` 开局 `AND #$80` 清位"——`$E1B8`=`8D 00 04` `STA $0400`；实链在 **`$E1BB` A5 26 / `$E1BD` 29 80 / `$E1BF` 85 26**（本审逐字节） | dated 更正地址 |
| AR67-4 | 规格缺口（计数瑕疵） | 低 | §1.1"`$0300`（关末推进闸）写点**仅 3 处**"后列 **4 址**（`bank11:$82C1/$833F` + `bank15:$E2D7/$E31D`）；本审全 ROM 复扫恰为这 4 个真写点（字节结论对，计数措辞错） | "仅 3 处"→"仅 4 处" |
| AR67-5 | 规格缺口（跨文档对账未闭环） | 低中 | §1.2 "loc_E16B 关间装载按 `$180` 取关数据"未与 t79/t82 目录表对账：装载器 `bank00:$8000`（`LDA $180 / ASL×3 / TAY / LDA $A000,Y`）以 **`$180` 原值**为行索引；运行时关1 `$180=01`（RAM 探针，§1.4）⇒ 关1 取**行1（$A048/$A3C4 行）**；而 `b-stg-01-en.md` 表列 "STG-1=行0（$A000/$A16F）"——**一格错位嫌疑**；且 t79 复现 `xxd -s 0x10010 -l 64` 仅覆盖行0-7，行8 候选 `5a b4 6b b2 0a 0a 2c b8`（$A040，结构合形：root/pattern/参数对(0A,0A)/tail）未入表 | 路由 spec/decomp 侧复核目录行映射（**不影响结局链任何结论**） |
| AR67-6 | 规格缺口（H7 分类 2 小漏） | 低 | 7 span 分类 4 项基本合理；span `0x1B89-0x1BAA` 内含 **COMPILE 字标 tile**（US `76 77 78 79 7a 7f 80 81 82` → CN ASCII `COMPILE`）未点名（被审归"尾部 logo"泛类）；各 span 内含行首定位前缀宽度字节差（`0C↔0B`、`03↔02`、`09↔2F`）未说明 | 分类补注 |
| AR67-7 | 纪律（坐标/转写口径滑动） | 低 | ①§4.1"四条名称记录（US，**file 0x1527** 起）"——记录实起 CPU `$9537`（file `0x1547`），`$952F` 是指针表；repro⑥ 实际用 PRG 0x1527 口径，"file"字样误置（与文首坐标注 `file=PRG+0x10` 冲突）；②"`@2011 JIUBAN`"实为 `@2011␣␣JIUBAN`（**双空格**）；③"bank11:`$82C1/$833F`"按其自家坐标注（bank00/01 映 $8000/$A000 ⇒ 奇数 bank 映 $A000）应记 `$A2C1/$A33F`，或注明沿用 seed"无证据者约定 $8000"记法 | 口径统一 + 转写修正 |
| AR67-8 | 确认维持（勿动） | — | 通关链（E2FC 块 + E366-E3A8 逐字节 + `$E649` 内联分派解码）、`$FFF8/$FFF9` 双标志区分、staff roll 流程/流格式/**43 条记录**（本审复数吻合）、结局卡调度（971E-9760 分支目标全复算）、`$97D3` 表 + scene 链逐字节、**三结局=难度**（字段表/难度名表/`STA (4),Y` 机制/默认 1）、NOVICE 无卡 / EXPERT 常驻、结局文本同源、双版 US==CN / JP 同构、**H7 定谳**、假设区纪律、复现命令可用性、门禁 12/12 | 全部维持（§1/§2 逐项锚） |

## 1. 独立重导（6 组，全部 ROM 直读/独立解析，不采信被审自述）

### 1.1 重导①：三段 hexdump + 独立 6502 反汇编逐字节对上 ✅

（a）**通关判定链 PRG 0x1E366-0x1E3B0**（bank15）raw：

```
ad 00 03 d0 03 4c 1c e2 a5 26 29 ef 85 26 a9 04 20 03 ee 20 56 ed
ee 80 01 ad 80 01 cd f8 ff d0 0b a9 02 20 03 ee 20 14 db 4c 73 e0
c9 09 f0 03 4c 6b e1 a9 02 20 03 ee 20 14 db 20 49 e6 00 01 f9 95
4c 73 e0 a9 04 20 03 ee
```

本审独立解码（非 seed 复读）与被审 §1.1 伪码**逐条一致**：`E366 AD 00 03 LDA $0300 / D0 03 BNE $E36E / 4C 1C E2 JMP $E21C`；
`E36E A5 26 29 EF 85 26`（$26&=~$10）；`E374 A9 04 / JSR $EE03 / JSR $ED56`；`E37C EE 80 01 INC $0180`；
`E37F AD 80 01 / E382 CD F8 FF CMP $FFF8 / E385 D0 0B BNE $E392`；`E387…E38F`（==FFF8 支：LDA #2/JSR $EE03/JSR $DB14/JMP $E073）；
`E392 C9 09 CMP #9 / E394 F0 03 BEQ $E399 / E396 4C 6B E1 JMP $E16B`；`E399…E3A1`（通关支）；
`E3A1 20 49 E6 JSR $E649 / 00 01 F9 95`（内联：R6:=$00、R7:=$01、JMP $95F9）；`E3A8 4C 73 E0`（回退）。
分支目标全部按相对偏移复算吻合 ✓。另复核门内 `E2F3: a5 26 29 10 d0 03 4c 1c e2` = `LDA $26/AND #$10/BNE $E2FC/JMP $E21C` ✓；
`$26 bit4` 置位点 `bank00:$8D00 F0 0A`（前置分支）/`$8D02 09 10 85 26`（ORA #$10/STA $26）✓ 与被审同址。

（b）**结局卡调度 PRG 0x171E-0x1770**（bank00）raw：

```
20 27 ed 20 d9 e5 a5 18 29 10 f0 f4 a4 4c f0 32 ad be 01 d0 2d
78 b9 d3 97 20 63 97 a0 3c 20 30 ed 20 27 ed 20 d9 e5 a5 16 29 10
f0 f4 a4 4c c0 03 f0 ee a5 4e d0 ea a9 00 20 63 97 20 27 ed
4c 5a 97 4c 73 e0 …
```

独立解码+分支复算与 §3.1 **逐条一致**：`9724 LDA $18/AND #$10/BEQ $971E`；`972A LDY $4C / 972C F0 32 BEQ $9760`
（$4C==0 → `9760 4C 73 E0 JMP $E073` 无卡回标题）；`972E LDA $1BE/D0 2D BNE $9760`；`9733 SEI/9734 B9 D3 97 LDA $97D3,Y/9737 JSR $9763`；
`973A A0 3C/JSR $ED30`（60f）；`9745 LDA $16/AND #$10/BEQ $973F`；`974B LDY $4C/CPY #3/BEQ $973F`
（$4C==3 → 忽略 START 卡常驻）；`9751 LDA $4E/D0 EA BNE $973F`；`9755 LDA #0/JSR $9763`；`975A JSR $ED27/JMP $975A`；`9760 JMP $E073` ✓。

（c）**场景记录表 PRG 0x17D3-0x17FF** raw：

```
00 01 02 03 | 2e 2c fb be 09 18 ff 97 | 5e 5c d2 c0 09 18 55 98
            | 22 20 4e bb 09 18 85 98 | 22 20 4e bb 09 18 f3 98 | 2e 2c fb bf 09 98 ff 97
```

选择表 `00 01 02 03` + scene0-4 五条 8B 记录与 §3.2 **逐字节一致**（scene1=`5E 5C D2 C0 09 18 55 98`、
scene2/3=`22 20 4E BB …85/…F3`、scene4=`2E 2C FB BF 09 98`）✓。

### 1.2 重导②：`$4C` 写点唯一性扫描（扩展族）+ 间接写通道 + 指针数据三重排查 ✅

- **直接写族（本审扩至 16 opcode）**：`85 4C STA $4C / 86 STX / 84 STY / E6 INC / C6 DEC / 8D STA $004C /
  9D STA $004C,X / 99 STA $004C,Y / 95 STA $4C,X / 96 STX $4C,Y / F6 INC $4C,X / D6 DEC $4C,X /
  EE/CE/FE/DE abs 族` 三版穷举。命中逐条语境排除后，**真写点全 ROM 唯一 = `bank15:$E04D`**
  （ctx `8d 80 01 85 4c a9 ff` = `LDA #$01/STA $0180/STA $4C`，冷启动默认 $4C:=1 ✓）；
  其余命中全部为跨指令字节对（如 bank00 `$859C`=`JSR $E62E`尾+`JMP $86AE`、bank04 `$96BB`=`JMP $D674`尾+`JMP $D10C`头、
  bank06 `$81C1`=`JMP $845A`尾+`JMP $A087`头、bank15 `$E3A7`=内联参数 `95`+`JMP $E073` 头），逐条可信。
  **被审结论成立**；其列示族（`85/86/84/E6/C6/9D/99`）不完备但结论未受影响（AR67-1）。
- **间接写机制正证**：设置屏字段指针装载 `bank00:$935E` 一带独立解码：`AD AF 01 LDA $01AF / 0A ASL / A8 TAY /
  B9 CE 94 LDA $94CE,Y / 85 04 / B9 CF 94 LDA $94CF,Y / 85 05 / A0 00 / B1 04`；
  字段表 `bank00:$94CE` = `80 01 AE 01 4C 00 4E 00 F1 00 BE 01 B6 01 B7 01`（US/CN 同字节）——
  第 3 项 = **`4C 00`** ⇒ GAME LEVEL 字段时 `$04/$05:=$004C`；编辑写点 `bank00:$93C3: a0 00 91 04`
  = `LDY #0 / STA ($04),Y` ⇒ **(4) 确指向 $004C** ✓（JP 同机制，`91 04` 同字节）。
- **负证据补环（本审）**：①全 PRG `4C 00` 字节对仅 **9 处**——唯一 RAM 指针源=字段表（bank00 `$94D2`）；
  其余 8 处为 code（bank04 `$90FA`=JMP $B100 操作数）或 tile/数值序列数据（bank11 `$AA5F` 为单调递减数值表 0x011F…0x0013，非指针）。
  ②通用 zp 写循环（`STA $00,X` 族）仅冷启动清零 `bank15:$E03C`（`A9 00 AA 95 00 9D 00 01…`，被审已注"冷启动 RAM 清零后"）。
  ⇒"运行时改写只能经设置屏"**成立**（补三环后）；被审以 A 级负结果直接"⇒"缺一环=AR67-1。
- `bank15:$E04D` 上下文独立复核：`A9 01 8D 80 01 85 4C`（$180:=1、$4C:=1）✓；难度名表 `bank00:$952F`=`37 95 46 95 55 95 64 95`
  ={`$9537/$9546/$9555/$9564`}，四条记录直读 = `NOVICE/INTERMEDIATE/ADVANCED/EXPERT`（PPU `$21D3` 落点，`FF` 尾）✓ 与 §4.1 表一致。

### 1.3 重导③：`$FFF8/$FFF9` 三版直读 ✅

```
CN/US/JP: file 0x20008 = 0x0A，0x20009 = 0x00（前区 00 00 00 00 0a 00 …）
```

三版同值 ✓。语义链独立复核：`E382 CD F8 FF CMP $FFF8` 在 `E392 C9 09 CMP #9` **之前** ⇒
哨兵先判；出厂 0x0A 不在可达值集（关内 1..8 → INC 后 2..9，9 已被 CMP #9 截走）⇒ **永不触发** ✓（休眠位定性成立）。
`$FFF9`（0x00）另证：`bank15:$E08E` 钳制块独立解码 = `LDA $FFF9 / BNE +12 / LDA $0180 / CMP #$01 / BEQ +2 / LDA #$01 / STA $0180`
——flag==0 时 **$180 钉 1**，与 hidden-option-gate-fff9.md § 表述互证 ✓；两标志区分清楚（AR67-8 维持）。

### 1.4 重导④：staff roll 流解析复数 ✅（43 条吻合）+ 运行时 `$180` 锚点

独立解析（自 `$9955` 起：`<0x20`=位置前缀、`$FE`=记录分隔、`≥$80` 非 FE=流尾）：

- 流尾 `FF` 在 **CPU `$9BC2` / file `0x1BD2`**（AR67-2）；流长 622B。
- **66 个 FE 分隔块 = 43 条非空文本记录 + 23 空行分隔**。43 条全文与被审 §2.2 列表**逐条同序吻合**
  （COMPILE STAFF → @2011␣␣JIUBAN）。⇒ "43 条记录"**复数吻合** ✓（R-E1 收口成立）；
  前缀字节实测 {03,04,06,07,08,09,0A,0C,0E} 全 <0x20 ✓ 格式口径成立。
- 社区"43 条记录"对账成立；"168 行=其 text.csv 行数"口径不强行对账（被审处理妥当）✓。
- **运行时锚点（本审新增）**：`analysis/traces/gameplay-stage1_f*.ram.hex` 45 个探针全数
  `$0180=01`（关1 全程）、`$004C=01`（GAME LEVEL 默认=INTERMEDIATE ✓ 佐证 §4.1 默认值主张）。

### 1.5 重导⑤：H7 diff span 重跑 ✅（7 span / 95B 完全一致）

CN↔US `0x1990-0x1BCE` 独立逐字节比对：

```
0x1996-0x1999 (4B)  CN BISU            US 徽标tile 0xCE 0xDE 0xB2 0xBD
0x19DD-0x19E4 (8B)  CN PAPAGENA        US 徽标tile
0x1A3C-0x1A47 (12B) CN SIIPU.RA.SU.    US BC B2 5E CC DF D7 BD CC DF D7 BD 5E（假名/徽标+5E 切换）
0x1A55-0x1A55 (1B)  CN (               US {
0x1A5B-0x1A5B (1B)  CN )               US }
0x1B89-0x1BAA (34B) CN 0C COMPILE FE 0E AND …  US 0B vwxyz 7f 80 81 82 …（COMPILE 字标）
0x1BAC-0x1BCE (35B) CN NTED BY NEXOFT CORP. FE 09 @2011␣␣JIUBAN
                    US SENTED BY `abcdefg CORP. FE 2F ijklmno FE FE
span count: 7  total bytes: 95  claimed==recomputed: True
```

被审清单**逐 span 完全一致**（含 95B 总量）✓。
同位字节直读：US `TAKA`@6729、`POCHI`@6735、`NAKAMORI`@6749、`CYBER`@6742 全 ASCII 同位 ✓；
`SIIPU` US 无 ASCII ✓；`JIUBAN` 仅 CN（US 同位=字标 tile `lmno`）✓；`SPECIAL THANKS` CN=US@{6682,6943} ✓；
`TOKYO SHOSEKI` CN/US 同位、JP 无（美日互斥维持）✓。**H7 成立**（§3）。

### 1.6 重导⑥（附加）：US==CN / JP 同构 / `$0300` 写点 / scene 链

- **US==CN**：PRG `0x1E2F3-0x1E3B0`（含门内+通关链）**逐字节全等** `True` ✓（覆盖被审 §5.1 的 0x1E366-0x1E3B0 口径并外扩）。
- **JP 同构**：bank15 通关链同形——JP `$E39C: c9 09 f0 03 4c 75 e1 a9 02 20 0d ee 20 81 d8`
  （`CMP #9/BEQ/JMP $E175/JSR $EE0D/JSR $D881`）；bank15 内 JMP/JSR 目标 `+0x0A`（`E16B→E175`、`EE03→EE0D`）✓；
  内联分派 JP=`00 01 72 95`（R6:=0/R7:=1/JMP **$9572**，JP staff roll 入口）——结构同构、bank00 布局漂移如实。
  与被审"JP 同构 +0x0A（B 级模式验证，gap 处不伪造映射）"口径相符 ✓。
- **`$0300` 写点**：全 ROM 复扫（STA/STX/STY/INC/DEC/abs,X/abs,Y/zp,X 族）真写点 = **4** 处：
  bank11 `$A2C1/$A33F`（置值）+ bank15 `$E2D7/$E31D`（`A9 00 8D 00 03` 清零）——与被审所列 4 址一致
  （计数措辞"仅 3 处"=AR67-4；bank11 记 `$82C1` vs 窗口记法=AR67-7③）。
- **scene 链逐字节**：scene1 链 `DE E9 08 0A 8F 98 46 20 | DE E9 03 0A 67 98 46 22 | 00 00` ✓ 与 §3.2 示例**完全一致**；
  scene2 链 `DE E9 0A 0A 8F 98 46 20 | 00 00`（1 条）；scene3 链 `DE E9 02 0A 8F 98 46 20 | DE E9 08 0A 05 99 C6 20 | 00 00`（2 条）；
  scene0/4 链 `DE E9 05 0A 23 98 86 20 | 00 00` ✓。⇒ "scene2/3 同 CHR($22/$20) 同基址($BB4E) **不同 tile 链**"
  **精确成立**（1 条 0A×0A@$2046 vs 2 条 02×0A@$2046+08×0A@$20C6）；补充观察：scene1 rec1/scene2/scene3 rec1 共用数据块 `$988F`
  不同 w/h——B 级"图片 tile 块"解读方向合理（handler `$E9DE` 字段语义被审已如实列 R-E4）✓。

## 2. 攻击点裁定

### 攻击点 1："$180==9=通关"——为什么是 9 不是 8？—— **裁定：成立（表述与语义一致）**

1. **运行时定锚（A 级）**：gameplay-stage1 全部 45 个 RAM 探针 `$0180=01` ⇒ 关内编号**1-based**，关1=$180 1。
2. **机制链（逐字节）**：开局 `bank15:$E04A A9 01 8D 80 01`（:=1）、标题钳 `bank15:$E08E-$E09C`（$FFF9==0 时钉 1）、
   关末 `E37C INC $0180`、`E392 CMP #9`。⇒ **INC 在比较之前**：关8（$180=8）清完 → 9 → 通关支。
   "9" 不是"9 个关"，是"8 关 + INC-after-clear"；**"第 8 关清完"表述与 $180 语义一致** ✓。
   若比较值为 8 则第 7 关清完即终局——9 是唯一正确常数。
3. **与 t79/t87 编号体系对账**：关卡数=8（`docs/spec/stage-timeline.md` "STG×8"、目录表 8 行、stg02-08-en STG-1..8）✓ 无第 9 关。
   装载器 `bank00:$8000` 以 `$180` **原值×8** 为行索引（seed 逐行 + 本审字节）⇒ 行空间 0-based；
   结合关1 `$180=1` ⇒ 关N=行N；**行0 为 stock 流程永不装载的空位**——"AREA-0 + 八关（0..8 槽位）"假设
   **部分支持**（社区 research 第 7 条"AREA-0 教训"旁证存在 AREA-0 概念），但注意：**"9" 的语义来源是
   1-based 关号+INC 时机，不是 9 个可玩关**。t79 目录行↔STG 映射的 0/1 基错位嫌疑 = AR67-5（跨文档，另案）。
4. **`$FFF8` 与 9 的关系**：哨兵比较在 `CMP #9` **之前**；0x0A > 终端值 9 且 $180 可达集上界=9（debug 直跳 `AND #3/ADC #1`≤4）
   ⇒ 0x0A 不可达、死路径 ✓；若把 $FFF8 改小（≤9）则在第 $FFF8 关清完提前回标题=「提前终局」语义自洽 ✓。
   "第 8 关清完" vs "$180 语义一致性"：**一致**。

### 攻击点 2：三结局=难度"写点唯一性"——(4) 目标验证？负证据完备性？—— **裁定：成立（负证据补环后；表述层记 AR67-1）**

1. **(4) 指针真指向 $004C？——已验证 ✓**：字段表 `bank00:$94CE` 第 3 项=`4C 00`（US/CN 同字节；JP 同表不同序）；
   装载 `LDA $01AF/ASL/TAY/LDA $94CE,Y/STA $04 + LDA $94CF,Y/STA $05`（$01AF=字段序，GAME LEVEL=第 3 字段）；
   编辑 `bank00:$93C3: LDY #0 / STA ($04),Y` ⇒ GAME LEVEL 字段编辑**确写 $004C** ✓（JP 同构）。
2. **其他间接写 $4C 通道？——本审三重排查（结论为真，但被审未做）**：①16 opcode 直写族三版穷举=仅 `E04D` 真命中；
   ②全 PRG `4C 00` 指针数据仅 9 处、唯一 RAM 指针源=字段表；③通用 zp 写循环仅冷启动清零。
   ⇒ "运行时改写只能经设置屏"**成立**。被审由直写唯一直接"⇒"唯一间接通道=**负证据推理链缺环**（AR67-1）：
   列示族不含 `8D/95/96/D6/F6/EE/CE/FE/DE`，间接 STA `(zp),Y/(zp,X)` 站点（PRG 内数百字节对，多为跨指令/数据）
   未声明排除方法。**结论不反转，仅证据链需补注或降 A→B**。
3. 分支只读 `$4C/$1BE/$4E`（971E 链逐指令复算）✓；"分数/隐藏标志不是分支变量"随链成立 ✓。

### 攻击点 3：`$97D3[$4C]` 表——$4C=0 无卡 / 3 不可掉证据？scene2/3 精确性？—— **裁定：成立**

1. **表项直读**：`$97D3: 00 01 02 03` ✓（scene 号=恒等映射）。`$4C=0` 无卡的证据在**代码守卫**而非表项：
   `972A LDY $4C / 972C BEQ $9760` → `9760 JMP $E073`（回标题）——$4C=0 根本不走到 `LDA $97D3,Y` ✓；
   `$4C=3` 不可掉：`974B LDY $4C / CPY #3 / BEQ $973F`（回等 START 环）✓。两点均逐字节成立（§1.1b）。
2. **scene2/3 "同 CHR 不同 tile 链"精确性 ✓**：记录字节 `22 20 4E BB 09 18 85 98` vs `22 20 4E BB 09 18 F3 98`——
   前 6 字节全同（CHR 对 $22/$20、基址 $BB4E、复用参数 09/18），仅链指针 `$9885/$98F3` 不同；
   链内容独立展开：scene2=1 条（`0A 0A,$988F,$2046`）、scene3=2 条（`02 0A,$988F,$2046` + `08 0A,$9905,$20C6`）
   ⇒ 两卡差异**确在 tile 块布局/内容而非 CHR** ✓（B 级字段语义维持 R-E4 如实）。
   scene1=`5E/$5C`+基址 `$C0D2`+链 `$9855`（2 条）✓ 与 TL;DR #8 一致。

### 攻击点 4：H7"美版自带署名"——US 同位直读 + 7 span 分类？—— **裁定：成立（分类 2 小漏记 AR67-6）**

1. **US 同位字节直读（A 级）**：`TAKA`@6729、`POCHI`@6735、`NAKAMORI`@6749、`{CYBER}`@6742 全 ASCII 与 CN 同位 ✓；
   `SIIPU.RA.SU.` 槽位 US=`BC B2 5E CC DF D7 BD CC DF D7 BD 5E`（徽标/假名 tile + `5E` 切换标记）无 ASCII ✓；
   尾部 US=`PRESENTED BY ` + 字标 tile（`` `abcdefg ``=0x60-0x67）+ 图形块（`FE 2F ijklmno`），CN=ASCII `NEXOFT CORP.`+`@2011␣␣JIUBAN` ✓；
   `JIUBAN` 全 ROM 仅 CN ✓。⇒ "美版构建自带署名行、九班仅字形层+括号+水印"**成立**。
2. **7 span/95B**：逐 span 复算与被审清单**完全一致**（`match: True`）。分类 4 项合理；
   2 处未点名（AR67-6）：span6 含 **COMPILE 字标**（非人名）、span 内含行首前缀宽度字节差（`0C↔0B`/`03↔02`/`09↔2F`——
   定位参数随字形宽度调整）。**不改变 H7 定性**。

## 3. H7 裁定

**H7 定谳维持：`TOOL & CONVERTER SIIPU.RA.SU. TAKA / POCHI (CYBER) NAKAMORI` = 美版构建自带 SPECIAL THANKS 署名行
（工具/转换器致谢），非汉化团队署名**——US 同位字节含 TAKA/POCHI/NAKAMORI/CYBER 全套 ASCII（A 级直读），
`SIIPU.RA.SU.` 处 US=徽标/假名 tile 仅字形层差异；九班改动=字形层（徽标→ASCII 罗马字）+括号风格（`{}`→`()`）+
尾部 `@2011 JIUBAN` 水印（CN 独有）。base-attribution H7 可按被审建议升格结案（正式销账归 base-attribution 侧登记）。
**附条件**：AR67-6 分类补注不阻断本裁定。

## 4. 纪律检查

| 项 | 判定 |
|---|---|
| 证据分级 A/B/C 分区 | ✅ §0/§1 逐条标级，B/C 入 §4.3 假设区（C-E1..C-E5）；残项 R-E1..R-E6 如实（R-E1/R-E5 收口注得当） |
| 证据指针可复现 | ⚠ 基本 ✅；3 处指针瑕疵：流尾界址（AR67-2）、`$E1B8` 地址（AR67-3）、名称记录 "file 0x1527" 口径（AR67-7①） |
| 未证不发明 | ⚠ 1 处越级：§4.1 负结果"⇒只能经设置屏"推理链缺环（AR67-1，结论经本审补证为真） |
| 假设区纪律 | ✅ 社区命名映射（C-E1）、卡面语义（C-E2/C-E3）、`$4E` 语义（C-E4）、`$1BE` 可见差异（C-E5）均入候选区未入结论 |
| 复现命令可用性 | ✅ 8 条命令抽验可跑（①③④⑤⑥⑦输出与正文吻合）；⑥"file 0x1527"注释口径小滑动（AR67-7①） |
| 门禁真绿 | ✅ 本审复跑 `python3 gates/cli.py check`：**PASS=12 FAIL=0 SKIP=0 TOLERATED=0**（含 0 静默跳过） |
| 双版/坐标桥纪律 | ✅ JP 点位全为字节模式验证、未手工换算；gap 处声明；addr-bridge 门禁全量复验绿 |

## 5. 处理建议（路由）

| ID | 路由 | 动作 |
|---|---|---|
| AR67-1 | 回被审（decomp-analyst 文档层） | §4.1 补三环推理（16 族直写扫描 + `4C 00` 指针数据扫描 + 通用 zp 写循环）或把"⇒运行时改写只能经设置屏"降 A→B 并注明动态 R-E6 收口路径 |
| AR67-2 | 回被审 | §2.2/§5.3 界址 dated 更正：流尾 `FF`=CPU `$9BC2`/file `0x1BD2` |
| AR67-3 | 回被审 | §1.1 dated 更正：`$E1BB-$E1BF`（LDA $26/AND #$80/STA $26） |
| AR67-4 | 回被审 | "仅 3 处"→"仅 4 处" |
| AR67-5 | **路由 spec/decomp（t82 目录表另案）** | 复核目录行 0/1 基映射与行8 完整性（`xxd -l 64`→72）；本文件仅登记，不改 A3 结论 |
| AR67-6 | 回被审 | H7 分类补注 COMPILE 字标 + 前缀宽度字节 |
| AR67-7 | 回被审 | 坐标口径统一（file vs PRG）+ `@2011␣␣JIUBAN` 转写 + bank11 窗口记法注 |

## 6. 复现命令（本审独立重导，全部实跑）

```bash
# ① 三段 hexdump（通关链/结局卡调度/场景表）
python3 -c "
cn=open('rom/gunnac_cn.nes','rb').read()[0x10:]
print(cn[0x1E366:0x1E3B0].hex(' ')); print(cn[0x171E:0x1770].hex(' ')); print(cn[0x17D3:0x17FF].hex(' '))"
# ② $4C 写点扩展族（16 opcode，三版）+ $4C 00 指针数据扫描
#    （真写点唯一 bank15:$E04D；指针数据唯一源=bank00:$94D2 字段表）
# ③ $FFF8/$FFF9 三版
python3 -c "
for n,p in [('CN','rom/gunnac_cn.nes'),('US','reference/us/gunnac_us.nes'),('JP','reference/jp/gunnac_jp.nes')]:
    d=open(p,'rb').read(); print(n, hex(d[0x20008]), hex(d[0x20009]))"
# ④ staff roll 流解析（FE 分块计数 + 流尾定位）
#    → 66 块=43 非空+23 空；FF=file 0x1BD2（CPU $9BC2）
# ⑤ H7 diff span（0x1990-0x1BCE 逐字节）→ 7 span/95B，match: True
# ⑥ 运行时锚点
python3 -c "
import glob
for f in sorted(glob.glob('analysis/traces/gameplay-stage1_f*.ram.hex'))[:3]:
    r=bytes.fromhex(open(f).read().strip()); print(f, hex(r[0x180]), hex(r[0x4C]))"
# ⑦ US==CN / JP 链
python3 -c "
us=open('reference/us/gunnac_us.nes','rb').read()[0x10:0x20010]
cn=open('rom/gunnac_cn.nes','rb').read()[0x10:0x20010]
print(us[0x1E2F3:0x1E3B0]==cn[0x1E2F3:0x1E3B0])"
# ⑧ gates
python3 gates/cli.py check   # PASS=12 FAIL=0 SKIP=0 TOLERATED=0
```

## 7. 验收对照（任务书 ①-⑤）

| # | 要求 | 本审结论 |
|---|---|---|
| ① | ≥4 独立重导 | ✅ 6 组（三段 hexdump+独立反汇编 / $4C 扩展族+间接+指针三重扫描 / $FFF8 三版 / 流解析复数+RAM 锚 / H7 span 复算 / US==CN+JP+$0300+scene 链） |
| ② | 攻击点 1-4 各有裁定 | ✅ A1 成立 / A2 成立（表述层 AR67-1）/ A3 成立 / A4 成立（AR67-6 小漏） |
| ③ | H7 裁定明确 | ✅ 维持"美版构建自带署名"，升格条件成立 |
| ④ | verdict 明确 | ✅ **pass**（零结论证伪；7 条低危 findings 全部路由） |
| ⑤ | gates 全绿 | ✅ PASS=12 FAIL=0 SKIP=0 TOLERATED=0（实跑输出 §6⑧） |
