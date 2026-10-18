# Boss/EN-9+ 记录族定界 + 敌 HP 字段定谳 + W15 取证路径（boss-en9-hp-evidence）

> 日期：2026-10-18｜角色：decomp-analyst（只读取证）｜任务卡：task-61（来源 task-38 报告 §3/§5 建议卡 C/D）
> 对象：HEAD `2bcb92a`｜纪律：AGENTS.md §3（A/B/C 分级；未证不发明；数字可复算）
> 本卡**不接行为**：只给定界/语义/接入判据与后续取证路径；动态样本不足处如实声明（本机无 Mesen2、`analysis/traces` 不入库）。
> 复现：`python`（本机 `python3` 别名损坏）；工作目录=仓库根；命令见 §5。

> **【2026-10-18 更正（task-64 复核 needs_revision → task-66 修订）】seed 只有部分解码，本文 xref 一律改双口径。**
> 实测：seed 91,601 行中 **56,636 行（61.8%）是单字节 raw dump**（未解码），分 bank 例：bank02 raw 7,193 / 解码 466、bank03 raw 4,962 / 解码 1,382、bank04 raw 7,257 / 解码 387（复现 §5 D7）。
> ⇒ 原文中凡"**全 seed 唯一调用点 / 全集 / 唯一写点 / 33 入口**"的单口径结论**均不成立**（只对"seed 解码面"成立）；本文已改为"**seed 解码面 vs ROM raw 扫描候选**"双口径，并逐条列出 raw 扫描新增调用点。
> ROM raw 扫描（§5 D6 可复现）：`$D7ED` **8 处**（bank04:$81E1 + raw：bank02:$8CEB/$8D35/$8E13/$8E75/$8F07/$8F3B/$8F97）；`$D4FE` **9 处**（seed 6 + raw：bank03:$ADDA/$B05A/$B0FE）；`STA $7D4,Y` **2 处**（bank14:$D53A + raw：bank02:$8882）；`$D674` **64 个字节候选**（≥4 raw 真调用：bank02:$ACA0 / bank03:$B3F0 / bank03:$B6EA / bank03:$B95F）。
> 等级调整：**W15 / W15′ / B4（HP 字段）由 A 降为 B**（待 raw 区审计/重解码后再升）；本文件其余 A 级表字节/生成链指令不受影响。

---

## 0. 结论速览

| # | 项 | 结论（级） |
|---|---|---|
| 1 | **Boss 记录族定界** | `$A16F` **不是**统一 5B 记录族起点：`$865C` 以 `($9F)[$10&$1F]` 读**类型表**，dir 尾字段（`bank00:$802A-$8032`）指向 `$A16F` ⇒ `$A16F` = **32B 类型表**（指令/字节 A，语义解释 B）；5B 贪心扫描（ptr∈`$A400-$B000`）在 `$A16F-$A17E`(3 条)后出现 `$A182-$A196`(4)/`$A19A-$A1A9`(3)/`$A1B7-$A1D0`(5)/`$A1D4-$A1ED`(5)/`$A1F0-$A209`(5)/`$A228-$A24B`(7)/`$A24F-$A263`(4) 多段短串（A 扫描），段间 **4/4/14/4/3/31/4B** 空隙（见 §1.2）⇒ **记录族为变长/混排、边界=W32 未证**。 |
| 2 | **Boss 型值列表** | A 级：STG-01 波次流 spawn `01 4A`/`01 5F`（file `0x103F2`/`0x10409`）；stage-8 终盘 `$CA`（`TERMINAL_DROPPERS`+`$B889` 流）；dispatch 表映射 `$4A→$ABDB(bank4)`、`$5F→$B9DB(bank3)`、`$61→$A129(bank4)`、`$62/$64/$65/$A0/$A1/$A2/$CA..$CF→bank7 12 体`。 |
| 3 | **HP/节拍表** | A 级字节 + A 级消费代码：`$4A` HP=`bank4:$ABCB[$4C]`=**`[0C,0E,18,28]`（14@$4C=1）**；`$5F` HP=`bank3:$BBF2[$4C]`=`[3C,50,64,78]`、节拍=`bank3:$BBF6[$4D]`；`$61` HP=`bank4:$A1F9[$4D]`=`[04,06,08,0A,0C,0F,14,19,1E]`、beat=`bank4:$A201`；`$CB` HP 重装=`bank7:$ADE5[$4C]`=`[0A,1E,32,50]`、节拍=`$ADE9[$4C]`=`[B4,78,5A,50]`、HP 初装=`$ADED[$4D]`=`[0A,14,19,23]`。**语义标签 B**（HP/beat 归属由消费代码位置读出，未逐值动态验证）。 |
| 4 | **EN-9+ 边界候选** | EN 8×5B=`$ADD0-$ADF7`（A，逐条可解）；5B 贪心扫描（ptr∈`$B000-$C000`，非重叠）得：`$ADD0-$ADFD`(9，第 9 条自 `$ADF8` 起、ptr=`$B0B2`)、`$AE01-$AE1F`(6)、`$AE22-$AE36`(4)、`$AE3A-$AE53`(5)、`$AE57-$AE6B`(4)、`$AE6F-$AE7E`(3)、`$AE82-$AE91`(3)、`$AE94-$AEA3`(3)、`$AEB4-$AEC3`(3)、`$AED2-$AEE1`(3)；段间空隙 **4/3/4/4/4/4/3/17/15B**（A 扫描）。**指针不都落 `$B0-$B4`**：`$AED2` 串首条=`$BE41`（A）。⇒ EN-9+ 起点候选 `$ADF8`（B），但非连续 5B 流（段间 3–4B/17/15B）⇒ 仅 B 级族级延续；**不接行为**。 |
| 5 | **敌 HP 定谳（B4/W18）** | **HP=`$0630,X`（B，待 raw 区审计）**：`$D674` 链逐指令（A）读/写/清零 `$630,X`；`$4C==0` 一击死；弹路 `$6E bit7` ⇒ −2；物撞 ⇒ −`$630,Y`；死亡 ⇒ `$D616` 经 `$D86B[256]` 状态转移。**双口径**：seed 解码面=27 直接+6 包装（33 入口）；ROM raw 扫描=**64 字节候选**（≥4 raw 真调用 bank02:$ACA0 / bank03:$B3F0 / bank03:$B6EA / bank03:$B95F）⇒ 覆盖结论降 B。**`$05DC`=相位/动画计数（非 HP）**：`bank14:$CD9D-$CDC7` 溢出计数、`bank05:$8C8D` `$5DC&7` 表索引（A 指令；语义 B）。`$05F7` 恒 0=非 HP（A 反证）；`$06F4`=到期倒数（A）。 |
| 6 | **W15/W15′（双口径，B）** | **W15**：seed 解码面 `$D7ED` 调用点 `bank04:$81E1`（type `$61` handler entry `bank04:$8129`；命中 ⇒ `$0400:=#$0F`+`$41C:=0`，A 指令）；**ROM raw 扫描 8 处**（+bank02:$8CEB/$8D35/$8E13/$8E75/$8F07/$8F3B/$8F97）⇒ 不再是"唯一调用点"。**W15′**：seed 解码面 `$D4FE` 6 处；**raw 扫描 9 处**（+bank03:$ADDA/$B05A/$B0FE）；`$7D4` 写者 **2 处**（bank14:$D53A + raw bank02:$8882 第二框表注册写者）⇒ "唯一注册入口"不成立。候选玩家面=`bank03:$A140-$A14F` 块（X=0 待动态）。⇒ 完整路径 **B（待 raw 区审计/重解码）**，仍不可接。 |

---

## 1. Boss / EN-9+ 记录族定界

### 1.1 目录表与生成链（A 级逐指令/字节）

- **目录表** `bank8:$A000`（8 关 × 8B，file `0x10010`）实读（A）：
  `STG-1: 00 A0 FA BC 09 09 6F A1`（[+0..1]=root `$A000`；[+2..3]=`$BCFA`；[+4]=09；[+5]=09；**[+6..7]=`$A16F`**）
  `STG-8: 6A AC 49 B1 0A 0A 7D AF`（尾 `$AF7D`）
- **尾字段消费** `bank00:$802A-$8032`（A）：`LDA $A006,Y / STA $9F / LDA $A007,Y / STA $A0` ⇒ `$9F/$A0` = 该 stage 的 `$A16F`。
- **生成例程** `bank00:$865C-$869E`（A 逐指令；`R-stage-tick-4` 同源）：
  `STA $48C,Y`；`X:=Y`；`$470,X:=$10`；`Y:=$10&#$1F`；**`LDA ($9F),Y / STA $400,X`（类型）**；
  `$4E0,X:=$11<<4`（X 坐标）；`$4FC,X:=$92+$E0-$91`（Y 坐标）；`$614,X:=$8E+2*$11`；`$72C,X:=((…&$1C)>>2)+$90`；`$5F8,X:=$8F`；`$710,X:=$8D`。
- **分派器** `bank00:$8619`（A）：被 `$83CD`/`$856C` 调用；`A bit7` → `JSR $C2C6` 分配槽 + `LDA ($40,X)`/`($42,X)` → `JSR $865C`（即记录流指针 `$40/$42` 由调度机装载，**记录→zp 字段的解码层在 bank00:$83xx 波次机**）。

### 1.2 `$A16F` 结构复核（**对 w28「5B 记录族 $A16F」的更正**）

| 证据 | 内容 | 级 |
|---|---|---|
| `$865C` 指令 | `LDA $10 / AND #$1F / TAY / LDA ($9F),Y / STA $400,X` ⇒ `$9F` 指向**32 项类型表**，`$10&$1F`=表索引 | A |
| dir 尾字段 | STG-1 `$A16F`（上 §1.1）⇒ 类型表基址=`$A16F` | A |
| 32B 实读 | `$A16F: CF 1B 10 1F A4 CF 14 10 CA A6 8F 1D 11 86 A4 12 60 05 A4 81 1D 21 35 A4 9D 1A 22 29 A4 82 14 13`（idx8=`$CA`，与 stage-8 boss 型 `$CA` 一致——B 级旁证） | A（字节）/B（类型表解释） |
| 5B 步进 | `$A16F` 起前 3 条 ptr=`$A41F/$A6CA/$A486` ✓；第 4 条 ptr=`$81A4` 失配（A 重算）；5B 贪心扫描（ptr∈`$A400-$B000`）：`$A16F-$A17E`(3)/`$A182-$A196`(4)/`$A19A-$A1A9`(3)/`$A1B7-$A1D0`(5)/`$A1D4-$A1ED`(5)/`$A1F0-$A209`(5)/`$A228-$A24B`(7)/`$A24F-$A263`(4)，**段间空隙 4/4/14/4/3/31/4B**；全范围扫描 `$A000-$BFFB`（§5 D3 同一命令）自 `$B886` 起另见 3–5 条短串（ptr∈`$A5xx-$AFxx`，为他区数据/未归属本族，**不计入本族**） | A（扫描） |

⇒ **判定**：`$A16F` = 类型表（B）；原 w28 "`$A16F` 起 5B 记录族（~30 指针）"**不能作为逐条实装依据**；记录族真实边界/条数/字段切分=**W32 未证**（`completeness.md` B-STG-01-BOSS 的 ⚠ 项仍成立）。类型表 32 项中可读出的 boss 候选值（B）：idx8 `$CA`；其余 `$1B/$1D/$11/$12/$14/$22/$29/$35/$60/$81/$86/$8F/$9D/$A4/$CF…` 是否皆为型值**未证**（`$A4` 重复出现，亦可能混有非型值字节）。

### 1.3 Boss 型值列表（A/B 分级）

| 型值 | 依据 | 级 | HP/节拍证据 |
|---|---|---|---|
| `$4A`（STG-01 中 boss） | 波次流 `$A3CD` spawn `01 4A`（file `0x103F2`）；dispatch `$4A→$ABDB`（bank4） | A | handler `bank04:$8BDB`：`LDA #$80/STA $668`；`LDA #$13/STA $4C4`；`LDA $ABCB[$4C]/STA $630,X`（HP）；`LDA #$FE/STA $614`；`$5F8:=$20+`…；**HP 表 [0C,0E,18,28]（14@$4C=1）**（A） |
| `$5F`（STG-01 终盘） | 波次流 `$A3E4` spawn `01 5F`（file `0x10409`）；dispatch `$5F→$B9DB`（bank3） | A | handler `bank03:$B9DB`：`LDA #$80/STA $668`；`LDA #$20/STA $4C4`；`LDA #$25/STA $518`；**HP `$BBF2[$4C]`=[3C,50,64,78]**；**beat `$BBF6[$4D]`=[B4,A0,8C,78,64,50,3C,32]**（A） |
| `$61`（接触型 boss/敌，杀玩家） | dispatch `$61→$A129`（bank4）；handler entry `bank04:$8129`（A） | A | HP `$A1F9[$4D]`=[04,06,08,0A,0C,0F,14,19,1E]；beat `$A201`=[1E,1B,19,17,15,13,11,0F…]；几何/初速 `$A1F1` 4B×2（[E0,00,FE,FF]/[20,40,01,00]）（A 字节/B 索引语义） |
| `$62/$64/$65/$A0/$A1/$A2` | `BANK7_HANDLERS` 12 体表（`enemy.rs` 数据=A 级 ROM 表）；dispatch 均 bank7 | A（表）/B（行为） | handler 体未实装（fail-closed）；行为语义未证 |
| `$CA/$CB/$CC/$CD/$CE/$CF` | 同上；`$CA` 亦见 stage-8 终盘（`$B889` 流）；`$CB/$CD` 生成链见 `boss-chain-survey.md` §1 | A（表）/B（行为） | `$CB` 表组（§1.4）；相位数/伤害系数未证 |

> 注：seed 地址与 MMC3 窗口：type≥$30 时 `$A000` 窗前置切 bank（dispatch 律），故如 `$4A→target $ABDB, bank_switch=04` 的实际 PRG 位置 = bank4 内偏移 `0xBDB`（本报告以 **seed 规范地址**引用：`bank04:$8BDB` 为 $8000 窗视图、`$ABDB` 为 $A000 窗视图，同一 PRG 字节）。此映射为 B 级（banking 语义按现有 dispatch 律推读）。

### 1.4 HP / 节拍表地址与语义分级（A 字节 + A 消费代码；语义标签 B）

| 表（seed 规范地址） | 字节 | 消费代码（A） | 语义（B） |
|---|---|---|---|
| `bank07:$ADE1` | `8C 5A 3C 1E`（+`0A 1E 32 50` 为下一表） | `bank07:AD31 LDA $ADE1,Y→$6BC,X` | 节拍初装（按 $48C 状态？Y 来源待逐条——B） |
| `bank07:$ADE5` | `0A 1E 32 50` = [10,30,50,80] | `bank07:AD46 / ADA6 LDA $ADE5,Y→$630,X` | HP 重装（$4C 索引） |
| `bank07:$ADE9` | `B4 78 5A 50` = [180,120,90,80] | `bank07:ADAC LDA $ADE9,Y→$6BC,X` | 节拍周期（$4C 索引） |
| `bank07:$ADED` | `0A 14 19 23` = [10,20,25,35] | `bank07:AD4D LDA $ADED,Y→$630,X`（Y=$4D） | HP 初装（$4D 索引） |
| `bank04:$ABCB` | `0C 0E 18 28` = [12,14,24,40] | `bank04:8BF2`（$4A handler） | `$4A` HP（$4C 索引；**14@$4C=1**） |
| `bank03:$BBF2` | `3C 50 64 78` = [60,80,100,120] | `bank03:B9F2`（$5F handler） | `$5F` HP（$4C） |
| `bank03:$BBF6` | `B4 A0 8C 78 64 50 3C 32` | `bank03:B9FB`（$5F handler） | `$5F` 节拍初装/周期（$4D） |
| `bank04:$A1F9` | `04 06 08 0A 0C 0F 14 19 1E` | `bank04:813A`（$61 handler） | `$61` HP（$4D） |
| `bank04:$A201` | `1E 1B 19 17 15 13 11 0F` | `bank04:8140`（$61 handler `→$6BC`） | `$61` 节拍（$4D） |

> 语义分级：**表地址/字节/消费指令=A**；"哪一列是 HP/哪一列是节拍"由消费指令的写目标（`$630` vs `$6BC`/`$6A0`）读出=**A 强 B**；"难度档/关卡驱动"（`$4C`/`$4D` 语义）未逐值动态验证=**B**。`$4C==0` 在 `$D674` 是"一击死"模式旗（见 §2），与 boss 表 `$4C` 索引含义的关系**未证**（同址多义风险，登记）。

### 1.5 EN-9+ 边界候选（B 级族级；短串=A 扫描）

- **EN 8×5B（A，逐条可解）** `$ADD0-$ADF7`：`{p0,p1,p2,ptr_lo,ptr_hi}`，ptr=`$B3E6/$B2B4/$B2B4/$B3E6/$B235/$B235/$B21D/$B2B4`（与 `stage.rs::parse_spawn_records` 一致）。
- **5B 贪心非重叠扫描**（ptr∈`$B000-$C000`；命令 §5 D3，范围 `$A000-$BFFB`；A 级扫描输出，与 §0.4 **同一清单**）：
  `$ADD0-$ADFD`(9；前 8 条=EN 文档族，**第 9 条自 `$ADF8` 起**、ptr=`$B0B2`) →
  `$AE01-$AE1F`(6) → `$AE22-$AE36`(4) → `$AE3A-$AE53`(5) → `$AE57-$AE6B`(4) → `$AE6F-$AE7E`(3) → `$AE82-$AE91`(3) → `$AE94-$AEA3`(3) → `$AEB4-$AEC3`(3) → `$AED2-$AEE1`(3)；
  **段间空隙 = 4/3/4/4/4/4/3/17/15B**（相邻 run 边界差）。
- **指针域更正**：上列短串 ptr 多数落 `$B0xx-$B4xx`，但**并非全部**——`$AED2-$AEE1` 串首条 ptr=**`$BE41`**（A）；"均落 `$B0-$B4`"不成立。
- **范围一致化**：`$AEE1` 之后至 `$BFFB` 仍有 ≥3 连串（如 `$B2B2/$B888/$B914/$B996/$BB1C/$BEA6` 等，ptr 域与步长均异，疑指针表/他区数据），**未归属 EN 本族**；§1.2 的"至 `$B885`"表述已改为"全范围扫描 `$A000-$BFFB`、`$B886` 起为他区/未归属"。
- **判定**：EN-9+ 起点**候选=`$ADF8`**（B；由"第 9 条 5B 块对齐"支持），但 `$ADF8` 之后**不是连续 5B 流**（A 扫描反证"全域 5B"；段间 3–4B/17/15B 空隙 ⇒ 变长/混排），记录归属/间隔字节语义**未证** ⇒ 仅支持"EN 族在 `$ADD0` 之后仍有 5B 形短串"的**族级**结论，**不得逐条实装**。与 task-38 §4.4 结论一致（原 w28"5B 同格式延续"降为 B 级）。
- 复现：§5 D2（表/记录字节）、D3（短串扫描输出，含 gap/ptr 打印）。

---

## 2. 敌 HP 字段定谳（B4 / W18）

### 2.1 通用伤害链 `bank14:$D674`（指令 A 级；调用面双口径：seed 解码 27+6 / ROM raw 64 候选）

```
$D674: JSR $D78B            ; 阅读端A：X=被击者，扫 21 条框（4 边+属性）
       BCC ret              ; 无命中返回
       JSR $D62F            ; 命中结算（回收弹/置命中旗/状态转移对方）
       LDA $4C : BEQ $D6BB  ; ★$4C==0 ⇒ 一击死（HP:=0）
       LDY $6E : BPL $D690  ; $6E=命中条目归属；bit7=弹
       LDA $630,X : SEC : SBC #2   ; ★弹路：HP := $630,X − 2
       → $D6A3 STA $630,X（或 $D6BB 归零→$D616）
$D690: LDA $668,Y : ORA #$20 : STA $668,Y      ; 物撞标记
       LDA $630,X : SEC : SBC $630,Y           ; ★物撞：HP := $630,X − $630,Y
       → STA $630,X
$D6BB: LDA #0 : STA $630,X : JMP $D616         ; ★死亡：HP 清零 + 状态转移
$D616: LDY $400,X : LDA $D86B,Y : CMP $400,X   ; ★$D86B[256] 状态转移表
       STA $400,X / TYA→$4A8,X / $41C,X:=0
```

- **结论 B4（B 级，待 raw 区审计/重解码）**：HP 字段=`$0630,X` 由 `$D674` **指令链本身**读出（**指令层仍 A**：读/减/写/清零全在该字段）；**调用覆盖面必须双口径**——**seed 解码面**=27 直接调用（bank04×1/bank05×11/bank06×11/bank07×4）+6 包装（$D84F×5/$D860×1）；**ROM raw 扫描**=**64 个字节候选**，其中 **≥4 raw 真调用**：`bank02:$ACA0`（$A000 窗视图；seed 窗别名 `$8CA0`）、`bank03:$B3F0`、`bank03:$B6EA`、`bank03:$B95F`（§5 D6）⇒ 原文"33 入口"仅为 seed 面覆盖，ROM 级覆盖未定 ⇒ **等级 B**。
- **候选→真调用判定规则（D6）**：①在 seed 已解码区（行内含 `JSR/JMP` 指令）⇒ 直接计入真调用；②在 raw 区（seed 为 data 行）⇒ 先记为**字节候选**，仅当满足任一条件才记"raw 真调用"：该 3B 起点与同段已证控制流/跳转目标吻合，或该字节处存在前序 `JSR $C304` 类调用上下文（本轮采用复核方已裁定的 ≥4 例）；其余候选**保留为候选**，不得用于"全网覆盖"或"不可伤类型"结论。
- `$D84F`/`$D860` = "伤害+死亡检查"包装（`JSR $D674` 后 `$630,X==0` ⇒ carry）；`$D60E`/`$D75D`/`$D674` 直调 = 命中反应/状态转移（`$D60E` 不扣血，仅 `$D62F`+`$D616`）。
- `$4C==0 ⇒ 一击死`（A）：`$4C` 语义与写点未定谳（C→B），且与 boss 表索引 `$4C` **同址多义风险**——实施必须参数化 + 登记，不得默认。
- **残余（未证）**：哪些类型**不**走 `$D674`（不可伤）需逐型 handler 审计；普通小怪 HP 初值按 type init 表（散点 A 级候选，未逐型收全）；动态验证（HP 首现/掉血）需 Mesen2。

### 2.2 `$05DC` 双读（W18）——判定"**相位/动画计数，非 HP**"（A 代码 + B 语义）

| 证据 | 内容 | 级 |
|---|---|---|
| `bank14:$CD9D-$CDC7` | `$5C0,X += A`；进位 ⇒ `INC $5DC,X`；`CMP Y`：`≥Y ⇒ $5DC:=0`、否则 `Y:=$5DC`；另有 `$5DC/$5C0:=0` 清零路 | A |
| `bank05:$8C8D-$8CA0` | `INC $5DC,X; LDA $5DC,X; AND #7; TAY; LDA $AC5F,Y; STA $684,X`（**mod-8 表索引=动画/形态**） | A |
| `bank14:$CBxx` | `LDA $5DC,Y / STA $5DC,Y / DEC …`（相位族读写） | A |
| 中 boss `$05F7`（`$05DC+27`）恒 0 | `arena-midboss.md` §3.1（A 反证"非 HP"） | A |
| hit_samples `release_golden` | 释放写集含 `hit_cnt:0`（`$05DC` 清计数）——与"释放清计数"相容 | A（golden）/B（归因） |

⇒ **W18 结论方向**：`$05DC` 不是 HP（HP=`$0630`），hit-sampling 观测的"接触帧归零"=**释放/相位清计数**，不是 HP 归零信号。**B 级**：`$05DC` 的精确语义（相位计数 vs 三角波帧计数 vs 动画帧）未在动态层逐帧验证；`$05F7` 对中 boss 恒 0 的原因未证。

**`$6F4`**：到期倒数（自然清除，R-enemy-clear-1 / `bank03:$A479` 等 DEC 写点，A 写点）；与 HP 无冲突。

### 2.3 L2 判定（规格提案；本卡无写权限，需 porter/verifier 卡落地）

> 本机无 6502 执行器写权限（`tools/da6502.py` 仅反汇编；`l2_cases.jsonl` 需 golden+核三方），故**不产出实测 L2**，给 4 条可判定断言规格（入参/期望出参均来自上述 A 级指令语义）：

| 编号 | 被测 | 入参 | 期望 | 判定式 |
|---|---|---|---|---|
| L2-HP-1 | `$D674` 弹路（`game-core::shot::damage_apply(Const2)` 同构） | hp=14、`$4C=1`、`$6E=$80`（bit7=1） | hp=12、`$630,X=12` | 期望值 = 14−2（A 指令） |
| L2-HP-2 | `$D674` 物撞路 | hp_X=10、hp_Y=3、`$6E` bit7=0 | hp_X=7 | 期望值 = 10−3（A 指令 `SBC $630,Y`） |
| L2-HP-3 | `$D674` `$4C=0` 一击死 | hp=14、`$4C=0`、任意命中 | hp=0 + 状态转移（`$D616`/`$D86B`） | 期望值 = 0（A 指令） |
| L2-HP-4 | `$05DC` 相位计数 | `$5C0=0xFF`、A=1、Y=4（阈值） | `$5DC=1`、Y=$5DC（未到阈）；再来 3 次溢出 ⇒ 达 Y ⇒ `$5DC=0` | 期望 = 进位计数/阈值归零（A 指令） |

**动态样本不足声明**：①逐型 HP 初值（255 型）需 Mesen2 写断点（`STA $630,X`）动态采集；②`$4C/$4D` 语义需设置屏/难度单变量场景；③`$05DC` 需逐帧 watch 验证。→ 转后续动态卡（Linux oracle 宿主）。

---

## 3. W15 / W15′ 最小取证路径

### 3.1 W15 收窄（双口径，B）

- **`$D7ED`（阅读端 B："我打到了玩家"）调用点——双口径**：**seed 解码面**=`bank04:$81E1`（唯一解码命中，`JSR $D7ED`）；**ROM raw 扫描=8 处**：`bank04:$81E1` + **bank02:$8CEB/$8D35/$8E13/$8E75/$8F07/$8F3B/$8F97**（§5 D6；7 处在 bank02 raw 区未解码）⇒ 原文"全 seed 唯一调用点"作废。上下文（A 指令）：
  - 所属 handler **entry = `bank04:$8129`**（dispatch `$61→$A129`，§1.3），初始化：`$4C4:=$93`、`$518:=$37/$3A`、`$684:=$CB/$AB`、HP=`$A1F9[$4D]`、`$630` 等；
  - 调用前 `JSR $CC6A`（运动更新）、`$534 |= 1`（翻转位）；命中后：
    `STA $0400 := #$0F`（**玩家槽 0 类型=0x0F 死亡态**）、`STA $041C := 0`（玩家行为字段清零）——A 指令。
- **其它 `$0400` 写点（seed 解码面 A 级 9 处，用途需分辨）**：`bank00:978E`（结局域）、`bank00:9CF9/9D23`（商店光标——`$0400` 同址多义）、`bank01:A98B/AB88`（菜单光标）、`bank03:A345`（死亡链，`$A343-$A4B5` 段）、`bank04:81E8`（本 handler=玩家死亡）、`bank15:E1B8/E2B6`（续关/重生）。
- ⇒ W15 现状态：**seed 解码面**已定位"敌接触杀玩家"路径（type `$61` handler，A 指令）；但 **raw 区另有 7 个 `$D7ED` 调用点未解码审计**，"玩家被弹/其它敌型伤害"普遍路径仍未定位 ⇒ **W15=B（待 raw 区审计/重解码）**。

### 3.2 W15′ 玩家框注册（双口径，B 候选）

- 框表归属写点（`TXA → $7D4,Y`）**双口径**：**seed 解码面= `bank14:$D53A`**；**ROM raw 扫描=2 处**：`bank14:$D53A` + **`bank02:$8882`（第二框表注册写者，raw 区未解码）**（§5 D6）⇒ 原文"唯一写点"作废。注册例程 entry `bank14:$D4FE`（A：`LDY $518,X / LDA $97BA,Y / LDA $9812,Y / … / JSR $D548` 取框 → 写 6 数组 + `INC $74`）。
- `$D4FE` 调用点**双口径**：**seed 解码面 6 处**（`A14F`/`A700`/`AADC`/`AB3A`/`AC39`/`AC3F`，均 bank03）+ **ROM raw 扫描 3 处**（**bank03:$ADDA / bank03:$B05A / bank03:$B0FE**）= **9 处**（§5 D6）。
- **候选玩家面 = `bank03:$A140-$A14F` 块**：`LDA $6A0`（↓）、`AND #1`、`INC $534`（绝对地址=玩家槽 0 字段）、`JSR $CC6A`、`JSR $D4FE`（A 指令）；若该块执行时 **X=0**，则 `owner=TXA=0` ⇒ 注册玩家框条目（阅读端 B 的 `$7D3,Y≠0` 跳过逻辑因此只留玩家条目）。**X=0 未静态定谳**（B 候选；反例：A700 在子对象生成循环中，X=分配槽；且 raw 区 3 个新调用点未解码，玩家面候选未穷举）。
- ⇒ W15′ 最小求证：**先重解码 bank02/bank03 raw 区**（`$D4FE` 9 调用点 + `$8882` 第二注册写者），再动态断点 `$D4FE` 记录 X 值 + `$74` 增量 + `$7D4,Y` 值；若出现 owner=0 条目即证玩家框注册点。当前等级 **B（待 raw 区审计/重解码）**。

### 3.3 需采 trace / 场景（可执行清单，供 Linux oracle 卡）

| 项 | 内容 |
|---|---|
| 场景 | `hit-test`（t25 已有场景名）+ `respawn-window`；另加"type `$61` 生成/接触"场景（可用 STG-01 后段或注入 `$61` 槽） |
| Mesen2 断点 | `bank14:$D4FE`（注册，记 X/`$74`）、`bank14:$D7ED`（记 X、返回值 C、`$6D`）、`bank14:$D674`（记 X/`$4C`/`$6E`）、`bank14:$D62F`、`bank14:$D616`、`bank03:$A14F`（玩家面注册候选）、`bank04:$81E1` |
| watch 列 | `$74`；框表 6 数组 `$0780/$0795/$07AA/$07BF/$07D4/$07E9`（21 条）；`$0400/$04E0/$04FC/$04C4/$0518/$0534/$0630/$05DC/$05F7/$06F4`（28 槽）；`$0400/$041C/$018D/$018E`（玩家/死亡） |
| 断言 | ①命中帧 `$0400: 1→0x0F`（或 `$630,0` 归零路）与 `$41C:=0` 同帧；②`$D7ED` 返回 C=1 的帧 ≥10（命中）与 C=0 近距帧 ≥10（未命中）；③`$D4FE` 至少一次 X=0 且 `$74` 增、`$7D4,Y=0`；④`$630,X` 递减值 ∈ {2, `$630,Y`} |
| 验收 | W15：>1 条调用点族（含动态发现）或显式负结论；W15′：owner=0 注册点定谳 + 玩家框常数（≥10 样本/侧）。本机不可执行（`scenario_run.py` 依赖 `fcntl`=POSIX），需 Linux/Mesen2 宿主。 |

---

## 4. 不确定项 / 未做（显式）

1. **动态样本不足**：本机无 Mesen2、`analysis/traces` 不入库、`tools/scenario_run.py` 为 POSIX-only ⇒ **不产出动态实测**；§2.3 的 4 条是待落地 L2 规格，不是实测结果。
2. **`$A16F`=类型表**为 B 级解释（指令/字节 A）；记录族真实边界 W32 未证；boss 实体 vs 后半波次仍未分离。
3. **EN-9+ 仅 B 级族级**：短串扫描 A、归属/格式未证；不得实装。
4. **banking 语义**：`type≥$30` 的 `$A000` 窗切换按现有 dispatch 律（B）；seed 地址为窗口视图，跨窗表（`$ADEx` 等 bank7、`$ABCB` bank4）以 seed 规范地址引用。
5. **B4 结论边界（B，待 raw 审计）**：HP=`$0630,X` 的**字段判定**由 `$D674` 指令链支持（指令 A）；但调用覆盖面双口径（seed 27+6 / raw 64 候选，≥4 raw 真调用），"不走该链的类型不可伤/另路"**需 raw 区审计**；HP 初值 per-type 仅 $4A/$5F/$61/$CB 有 A 级表。
6. **W15 未全闭（B，双口径）**：seed 解码面唯一 `$D7ED` 调用点已定位（type `$61`），但 **raw 扫描另有 7 处调用点未解码审计**；W15′ 注册点仅 B 候选（`$D4FE` raw 3 新点 + `$8882` 第二注册写者，X=0 待动态）。
7. 只读卡：未改 `crates/`/`data/`/`gates/`/`rom/`/`reference/`，未 git add/commit；仅本文件。
8. **【2026-10-18 新增·P1 另卡】** seed 部分解码（61.8% raw）是本轮 xref 失真的根因 ⇒ 建议另卡把"**核查 seed 是否解码；统计 xref 前必须做 ROM raw 扫描（`20/4C <lo> <hi>`）并双口径**"写入 `reference/seed/README` 与 `docs/knowledge/`（本卡不改，避免越写范围）。

---

## 5. 复现命令（只读；**PowerShell 可执行形式**——`@'…'@ | python -`，或把中间段存为 `.py` 后 `python 文件.py`）

> 说明：本机 `python3` 别名损坏，统一用 `python`；以下命令在 **Windows PowerShell（5.1+）/ PowerShell 7** 均可执行（here-string 形态经本机 Windows PowerShell 5.1 实测可跑）。

**D1 目录表/生成链/类型表（§1）**

```powershell
@'
rom=open('rom/gunnac_cn.nes','rb').read()
def f(bank,cpu): return 0x10+bank*0x2000+(cpu & 0x1FFF)
print('dir:', ' '.join('%02x'%x for x in rom[f(8,0xA000):f(8,0xA000)+0x40]))
print('type-table candidate $A16F:', ' '.join('%02x'%x for x in rom[f(8,0xA16F):f(8,0xA16F)+32]))
print('spawn rec1:', rom[0x103F2:0x103F4].hex(' '), '(01 4a)')
print('spawn rec2:', rom[0x10409:0x1040B].hex(' '), '(01 5f)')
'@ | python -
# 生成链/分派器逐指令（seed 反汇编）：PowerShell 用 Select-String
Select-String -Path reference/seed/gunnac.asm -Pattern '^bank00:(865C|8619|802[0-9A-F]|803[0-9A-F])'
```

**D2 HP/节拍表（§1.4，A 字节）**

```powershell
@'
rom=open('rom/gunnac_cn.nes','rb').read()
def f(bank,cpu): return 0x10+bank*0x2000+(cpu & 0x1FFF)
def show(t,b,c,n): print(t, ' '.join('%02x'%x for x in rom[f(b,c):f(b,c)+n]))
show('bank7 $ADE1',7,0xADE1,16); show('bank7 $ADE5',7,0xADE5,8)
show('bank7 $ADE9',7,0xADE9,8); show('bank7 $ADED',7,0xADED,8)
show('bank4 $ABCB ($4A HP)',4,0xABCB,4); show('bank3 $BBF2 ($5F HP)',3,0xBBF2,12)
show('bank4 $A1F9 ($61 HP)',4,0xA1F9,9); show('bank4 $A201 ($61 beat)',4,0xA201,9)
'@ | python -
```

**D3 EN/EN-9+ 字节 + 5B 短串扫描（§1.5；打印 gap 与 ptr 以复核 §0.4/§1.5 清单）**

```powershell
@'
rom=open('rom/gunnac_cn.nes','rb').read()
def f8(a): return 0x10 + 8*0x2000 + (a-0xA000)
for a,n in [(0xADD0,40),(0xADF8,64),(0xAE01,32),(0xAE30,40),(0xAE57,24),(0xAE7D,24)]:
    print('$%04X:'%a, ' '.join('%02x'%rom[f8(a)+i] for i in range(n)))
print('== 5B greedy scan (range $A000-$BFFB)')
for lo,hi in [(0xB000,0xC000),(0xA400,0xB000)]:
    print('-- ptr range', hex(lo), hex(hi)); prev=None; a=0xA000
    while a <= 0xBFFB:
        n=0; a2=a; ptrs=[]
        while a2+5 <= 0xC000:
            p = rom[f8(a2)+3] | (rom[f8(a2)+4]<<8)
            if lo <= p < hi: n+=1; ptrs.append('$%04X'%p); a2+=5
            else: break
        if n>=3:
            gap = '' if prev is None else ' gap=%d' % (a-prev)
            print(' run $%04X-$%04X recs=%d%s ptrs=%s' % (a,a2,n,gap,','.join(ptrs)))
            prev=a2; a=a2
        else: a+=1
'@ | python -
```

**D4 伤害/HP 链与调用点（§2）**

```powershell
Select-String -Path reference/seed/gunnac.asm -Pattern 'bank14:D674|bank14:D616|bank14:D86B|bank14:D62F|bank14:D84F|bank14:D860'
Select-String -Path reference/seed/gunnac.asm -Pattern '\b(JSR|JMP)\s+\$(sub_|loc_)?D674'   # seed 解码面调用点
Select-String -Path reference/seed/gunnac.asm -Pattern 'bank14:CD9|bank14:CDA|bank14:CDB|bank14:CDC|bank05:8C8|bank05:8C9'
```

**D5 W15/W15′ 调用点（§3，seed 解码面）**

```powershell
@'
import re
seed=open('reference/seed/gunnac.asm',encoding='utf-8',errors='replace').read().splitlines()
for tok in ['D7ED','D4FE','D53A','D539','7D4','0400']:
    print('==',tok)
    for ln in seed:
        if tok in ln and re.search(r'\b(JSR|JMP|STA|TXA)\b',ln): print(' ',ln[:100])
'@ | python -
```

**D6【新增·双口径必跑】ROM raw 扫描（§0/§2.1/§3.1/§3.2；输出候选 + seed 解码状态）**

```powershell
@'
import re
rom=open('rom/gunnac_cn.nes','rb').read()
prg=rom[0x10:0x10+0x20000]
WIN={0:0x8000,1:0xA000,2:0x8000,3:0xA000,4:0x8000,5:0x8000,6:0x8000,7:0xA000,
     8:0xA000,9:0x8000,10:0x8000,11:0x8000,12:0xA000,13:0x8000,14:0xC000,15:0xE000}
SEED=re.compile(r'^bank(\d\d):([0-9A-F]{4})\s+((?:[0-9A-F]{2} )*[0-9A-F]{2})\s+(\S+)\s*(.*)$')
DATA=re.compile(r'^\$[0-9A-F]{2}(?:,\$[0-9A-F]{2})*$')
seed=open('reference/seed/gunnac.asm',encoding='utf-8',errors='replace').read().splitlines()
decoded=set()
for ln in seed:
    m=SEED.match(ln)
    if not m: continue
    if DATA.match(m.group(4)) and not m.group(5).strip(): continue   # 该行是 raw data dump
    decoded.add('bank%s:%s' % (m.group(1), m.group(2)))
def loc(off):
    b=off//0x2000; return 'bank%02d:$%04X'%(b, WIN[b]+off%0x2000), 'bank%02d:%04X'%(b, WIN[b]+off%0x2000)
def scan(pat, name):
    hits=[]; i=0
    while True:
        j=prg.find(pat,i)
        if j<0: break
        hits.append(j); i=j+1
    print('==', name, 'candidates=', len(hits))
    for j in hits:
        disp, key = loc(j); print('  ', disp, 'seed_decoded=%s' % (key in decoded))
    return hits
scan(bytes([0x20,0xED,0xD7]), 'JSR $D7ED')
scan(bytes([0x4C,0xED,0xD7]), 'JMP $D7ED')
scan(bytes([0x20,0xFE,0xD4]), 'JSR $D4FE')
scan(bytes([0x4C,0xFE,0xD4]), 'JMP $D4FE')
scan(bytes([0x99,0xD4,0x07]), 'STA $7D4,Y')
scan(bytes([0x20,0x74,0xD6]), 'JSR $D674')
scan(bytes([0x4C,0x74,0xD6]), 'JMP $D674')
'@ | python -
# 期望（复核方口径）：$D7ED=8；$D4FE=9；STA $7D4,Y=2；$D674 3B 候选=9+55=64
# 真调用判定：seed_decoded=True ⇒ 直接算真调用；False ⇒ 记 raw 候选，需满足"与已证控制流/自向量目标吻合"才升真调用
# 注：地址按 seed 规范窗口（workset WIN 表；bank02=$8000 窗）打印；复核方若按 bank02=$A000 窗视图计，bank02:$8CA0 同字节别名=$ACA0。
```

**D7【新增】seed 解码覆盖率（双口径根源；§0 更正块引用的数字）**

```powershell
@'
import re, collections
seed=open('reference/seed/gunnac.asm',encoding='utf-8',errors='replace').read().splitlines()
SEED=re.compile(r'^bank(\d\d):([0-9A-F]{4})\s+((?:[0-9A-F]{2} )*[0-9A-F]{2})\s+(\S+)\s*(.*)$')
DATA=re.compile(r'^\$[0-9A-F]{2}(?:,\$[0-9A-F]{2})*$')
dec=collections.Counter(); raw=collections.Counter()
for ln in seed:
    m=SEED.match(ln)
    if not m: continue
    b=int(m.group(1))
    if DATA.match(m.group(4)) and not m.group(5).strip(): raw[b]+=1
    else: dec[b]+=1
tot=len(seed); raws=sum(raw.values())
print('lines=%d decoded=%d raw=%d raw%%=%.1f%%' % (tot, sum(dec.values()), raws, raws/tot*100))
for b in range(16): print(' bank%02d decoded=%-5d raw=%-5d' % (b, dec[b], raw[b]))
'@ | python -
# 期望：lines=91601 raw=56636 raw%=61.8%；bank02 466/7193；bank03 1382/4962；bank04 387/7257
```

---

## 6. 自查

- 只写本文件（`docs/findings/boss-en9-hp-evidence.md`）；未 git add/commit；未动其它文件。
- **双口径纪律（2026-10-18 修订后）**：凡"调用点/写者/入口"类结论一律给 **seed 解码面 + ROM raw 扫描候选** 两栏（§2.1/§3.1/§3.2/§0 已改）；禁止再单称"唯一/全集"（seed 仅 38.1% 解码，见 §5 D7）。
- 级：§1.1/§1.3(表)/§1.4(字节+消费)/§2.1(指令链)/§2.2 为 A（字节/指令/表可复算）；**W15 / W15′ / B4（调用覆盖面）降为 B（待 raw 区审计/重解码）**；§1.2 类型表解释、§1.5 EN-9+ 归属、§2.2 `$05DC` 语义标签为 B；未证项（W32/`$4C` 语义/逐型 HP 初值/玩家被弹路径/raw 区 7 个 `$D7ED` 调用点）均显式保留。
- 动态样本不足已如实声明（§2.3/§4.1），未外推。
- 定点复核建议（task-66 只核 3 项）：(a) §0/§2.1/§3.1/§3.2/§4 双口径与 W15/W15′/B4 降级是否落实且无残余"唯一/全集"；(b) §5 **D6 raw 扫描**是否可跑并复现 8/9/2/64 与 ≥4 raw 真调用；(c) §1.5/§0.4 段清单与 gap/ptr（含 `$BE41`）是否与 D3 输出逐项一致。

【2026-10-18 初版】decomp-analyst，HEAD `2bcb92a`，任务 task-61。

【2026-10-18 更正（task-66；依据 task-64 复核 needs_revision + ROM raw 扫描）】
- **双口径**：seed 仅 38.1% 解码（91,601 行中 56,636=61.8% raw）⇒ 原文"全 seed 唯一/全集/33 入口"仅对 seed 解码面成立；已改 seed 解码面 vs ROM raw 扫描候选。
- **raw 新增**（D6 可复现）：`$D7ED` 8 处（+bank02:$8CEB/$8D35/$8E13/$8E75/$8F07/$8F3B/$8F97）；`$D4FE` 9 处（+bank03:$ADDA/$B05A/$B0FE）；`STA $7D4,Y` 2 处（+bank02:$8882）；`$D674` 64 候选（≥4 raw 真调用 bank02:$ACA0 / bank03:$B3F0 / bank03:$B6EA / bank03:$B95F）。
- **降级**：W15 / W15′ / B4 调用覆盖面 A→**B（待 raw 区审计/重解码）**；字段级/指令级结论（HP=`$0630,X`、`$D674` 链、`$05DC` 非 HP）保留（指令 A）。
- **E2-E5 精度修正**：boss 段间空隙 4/4/14/4/3/31/4B；EN 段间空隙 4/3/4/4/4/4/3/17/15B；"指针均落 $B0-$B4"更正（`$AED2` 串含 `$BE41`）；§0.4 与 §1.5 清单统一；§1.2 "至 $B885" 与 §5 D3 全范围 `$A000-$BFFB` 一致化；§5 全部命令改 **PowerShell 可执行**（`@'…'@ | python -`）。
- **P1 另卡建议**：把"seed 部分解码 ⇒ xref 必须 ROM raw 扫描并双口径"写入 `reference/seed/README` 与 `docs/knowledge/`（本卡不改，避免越写范围）。
