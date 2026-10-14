# W-P1 搬家区逐字节审计（bank14 / bank15 / bank00）

> 【2026-10-14】独立反编译审计员产出。任务：证明/证伪"九班汉化补丁在代码搬移区未注入游戏逻辑改动"。
> 对照：`reference/jp/gunnac_jp.nes`（sha 08ead6a2…）vs `rom/gunnac_cn.nes`（sha 4d76901c…）。
> 证伪立场：默认找"补丁动了游戏逻辑"的证据，找不到才输出"未注入"。
> **只读分析**：本文件是本次审计唯一写入；未改 ROM/台账/门禁/基线。

## 0. 裁定（先说结论）

**W-P1 维持（不关闭）** —— 反例指针见 §6。

- ✅ **"未注入"方向成立**：三个搬家区**未发现任何新增钩子调用 / 新增 JSR / 条件分支语义改写 / 循环边界改写 / 立即数常量改写 / 数值表值改写**。全部 203(bank14) + 215(bank15) 处差异中的**操作数类差异**逐个核对，位移量与搬家偏移**严格自洽**（bank14 内 +0x2C6 / +0x293 两段；bank15 内 −0x00A；跨 bank 目标一律 −0x00A）。
- ❌ **"搬移纯布局"方向被证伪**：发现 **2 处 JP 侧存在的代码块在九班侧消失**（合计 61B）：
  - **反例 1（严重）**：bank14 `$D772–$D7A4`（51B）——关卡介绍期 OAM 精灵发射块，**活代码**；其配套精灵记录表在九班侧原样保留（成为孤儿数据）。
  - **反例 2（弱）**：bank15 `$E093–$E09C`（10B）——初始化期强制 `$01BE:=0`；静态扫描显示两版 ROM 内**均无其他 `$01BE` 写者** ⇒ 推断级判为"实效中性"，但字节级删除事实成立。

故不能签署"搬移区只动布局"。**"注入游戏逻辑"没有证据；"改动游戏逻辑（删除）"有证据**。

---

## 1. 方法、坐标系与可复现命令

### 1.1 坐标系（A 级，先锁）

| 项 | 值 | 证据 |
|---|---|---|
| 文件头 | JP `flags6=0x40` / CN `flags6=0x41`（mirroring 位） | `python3 -c "print(open('reference/jp/gunnac_jp.nes','rb').read()[:16].hex())"` |
| PRG | 128K = 16 × 8KB bank，bankN 文件偏移 `0x10+N*0x2000` | iNES 头 |
| bank14 | 文件 `0x1C010–0x1E00F`，CPU 窗 `$C000–$DFFF` | bank 内偏移 = CPU − $C000 |
| bank15 | 文件 `0x1E010–0x2000F`，CPU 窗 `$E000–$FFFF`（固定尾窗，含向量） | 向量在 `$FFFA` |
| bank00 | 文件 `0x00010–0x0200F`，CPU 窗 `$8000–$9FFF` | R6 映射 |

> 坐标桥教训沿用 `docs/findings/jp-cn-parity.md`：**JP 地址不可直套九班**，一律按本表换算。

### 1.2 主方法（可复现）

1. **逐字节对齐**：`difflib.SequenceMatcher(None, JP_bank, CN_bank, autojunk=False).get_matching_blocks()`
   → 得到"完全相同块(EQ)"与"差异间隙(GAP)"的**一一配对**。EQ 块按定义逐字节同（`Confidence=100`），
   搬家"内容级等价"即由此直接成立；**所有可疑点落在 GAP 里**，故 GAP 是审计主战场。
2. **位移自洽性**：对每个 EQ 块记 `delta = CN_off − JP_off`，做直方图；对每个 GAP 用"前一字节=指令操作码"反推该 GAP 是否为操作数。
3. **删除检测**：对 JP 独有块做**全 PRG 签名搜索**（含无操作数子签名）确认 CN 侧无重定位副本。
4. **语义分级**：字节级=A 级；"这是显示代码/这是文本表"= B 级（调用图/数据流）；"具体含义"= C 级，只进假设区。

复现入口（一行）：

```bash
cd /home/peng/workspace/gunnac_remake && python3 - <<'EOF'
import difflib
jp=open('reference/jp/gunnac_jp.nes','rb').read(); cn=open('rom/gunnac_cn.nes','rb').read()
for b in (14,15,0):
    o=0x10+b*0x2000; J,C=jp[o:o+0x2000],cn[o:o+0x2000]
    mb=[a for a in difflib.SequenceMatcher(None,J,C,autojunk=False).get_matching_blocks() if a.size>0]
    from collections import Counter
    print(f'bank{b:02d} EQ blocks={len(mb)} matched={sum(a.size for a in mb)}B',
          Counter(a.b-a.a for a in mb if a.size>=8).most_common(4))
EOF
```

---

## 2. bank14（+659B 级移位）逐块分类

### 2.1 全局结构（A 级）

对齐结果：**202 个 EQ 块（逐字节全同）/ 203 个 GAP**。EQ 块的 delta 只有两个取值：

| 段 | JP 范围 | CN 范围 | delta | 性质 |
|---|---|---|---|---|
| **A 段** | `0x0000–0x1771` | `0x02C6–0x1A37` | **+0x2C6 (+710)** | 代码+数据整体后移 710B |
| **B 段** | `0x17A5–0x1D6C` | `0x1A38–0x1FFF` | **+0x293 (+659)** | 代码+数据整体后移 659B |

（A/B 段之间为差异区，见 §2.3 反例 1。）

复现：

```bash
python3 -c "
import difflib;jp=open('reference/jp/gunnac_jp.nes','rb').read();cn=open('rom/gunnac_cn.nes','rb').read()
J,C=jp[0x1C010:0x1E010],cn[0x1C010:0x1E010]
for a in difflib.SequenceMatcher(None,J,C,autojunk=False).get_matching_blocks():
    if a.size>=200: print(f'JP {a.a:04X}-{a.a+a.size-1:04X} -> CN {a.b:04X} d={a.b-a.a:+d} size={a.size}')"
```

### 2.2 GAP 分类表（203 处，全数分类，无遗漏）

| # | 类别 | 处数 | 判定 | 证据 |
|---|---|---|---|---|
| G1 | **2B 绝对地址操作数更新** | **147** | 纯搬家 | GAP 前一字节= `ABS/ABX/ABY/IND` 寻址操作码（`20/4C/AD/B9/BD/8D/…`），GAP 长度恰=2，即指令唯一变的是 `$hhll`；逐个核对目标地址按本段 delta 平移或按 bank15 −10 平移 |
| G2 | **1B 立即数=指针拼装** | **4** | 纯搬家 | JP `0x1471/0x1475/0x1490/0x1494`：`A9 f1 85 14 A9 c5 85 15` → CN `A9 b7 85 14 A9 c8 85 15`，即零页指针 `$14/$15 := $C5F1 → $C8B7`，**delta 恰=+0x2C6** ✓（构造的是 A 段内表地址，随搬家更新） |
| G3 | **1B 跨 bank JSR/JMP 低位** | **12** | 纯搬家 | 目标全在 bank15，`$E647→$E63D` `$ED45→$ED3B` `$E60B→$E601` `$E8BB→$E8B1` `$E63D→…` 等，**低字节一律 −0x0A** ✓ 与 bank15 的 −10 移位自洽 |
| G4 | **精灵记录表数据差** | **37** | 显示布局数据 | JP `$D7C2–$D830` ↔ CN `$DA55–$DAC3`，4B 记录 `(X,Y,tile,attr)`；差异全在 X/tile 字段（见 §2.4） |
| G5 | **九班新增：字模映射表** | **1（710B）** | 文本支持数据 | 见 §5.1 |
| G6 | **JP 独有代码块** | **1（51B）** | ⚠️ **反例** | 见 §2.3 |
| G7 | JP 尾部 FF 填充 | 1（659B） | 无内容 | JP `0x1D6D–0x1FFF`=FF；CN 同区已被 B 段内容用满 |

复现（G1/G2/G3 判别器）：

```bash
python3 -c "
import sys;sys.path.insert(0,'tools');import da6502 as D,difflib
jp=open('reference/jp/gunnac_jp.nes','rb').read();cn=open('rom/gunnac_cn.nes','rb').read()
J,C=jp[0x1C010:0x1E010],cn[0x1C010:0x1E010]
mb=sorted([a for a in difflib.SequenceMatcher(None,J,C,autojunk=False).get_matching_blocks() if a.size>0],key=lambda x:x.a)
prev=0
for a in mb:
    if a.a>prev:
        g=prev; n=a.a-prev
        op=D.T.get(J[g-1]) if g else None
        print(f'GAP {g:04X} len={n} prev={J[g-1]:02X} -> {op[0] if op else \"?\"} | JP={J[g:g+n].hex()} CN={C[a.b:a.b+n].hex()}')
    prev=a.a+a.size"
```

### 2.3 ⚠️ 反例 1：JP `$D772–$D7A4`（51B）九班侧消失（A 级）

**字节事实**：

| 项 | JP | CN |
|---|---|---|
| 代码位置 | bank14:`$D772–$D7A4`（off `0x1772–0x17A4`，51B） | **无对应** |
| 该处 CN 内容 | — | off `0x1A38` 直接是 JP off `0x17A5` 的内容（=B 段起点） |
| 入口分支 | `$D726: 90 4A BCC $D772` | `$D9EC: 90 4A BCC $DA38`（指向尾段） |

JP 侧 51B 反汇编（`bank14:$D772`）：

```
D772  AC 80 01   LDY $0180        ; 关卡号/介绍索引
D775  BE B9 D7   LDX $D7B9,Y      ; 取表值
D778  C0 08      CPY #$08
D77A  D0 05      BNE +5
D77C  A0 78      LDY #$78
D77E  20 3E D7   JSR $D73E        ; 发射精灵记录 Y=$78
D781  A0 67      LDY #$67
D783  20 3E D7   JSR $D73E        ; 发射精灵记录 Y=$67
D786  A9 64      LDA #$64
D788  85 56      STA $56          ; X
D78A  AC 80 01   LDY $0180
D78D  8A         TXA
D78E  18         CLC
D78F  79 4B D8   ADC $D84B,Y
D792  85 58      STA $58          ; tile
D794  20 FF CC   JSR $CCFF        ; OAM 发射器
D797  A5 58      LDA $58
D799  18         CLC
D79A  69 02      ADC #$02
D79C  85 58      STA $58
D79E  A9 6C      LDA #$6C
D7A0  85 56      STA $56          ; X
D7A2  20 FF CC   JSR $CCFF        ; OAM 发射器
```

**可达性（B 级，可复现推理）**：

```
D70D  AD 88 01   LDA $0188        ; $0188 = stage_intro_counter（N-0200，A 级台账）
D710  F0 5F      BEQ end
D712  10 0E      BPL $D722        ; → 落到 CMP #$7E，A=$0188
D722  C9 7E      CMP #$7E
D724  F0 0D      BEQ A路
D726  90 4A      BCC B路(=$D772)  ; $0188 ∈ [0x01,0x7D] 必走
```

`$0188` = 关卡介绍倒数计数器（初值 64、每 4 帧 DEC、273 帧全程，N-0200 A 级）
⇒ **每关开场 4.5 秒内每帧执行该 51B 块**，绝非死代码。

**语义（B 级）**：`$CCFF`（九班 `$CFC5`）= OAM 精灵发射器（N-0118 A 级：参数 `$55=Y/$56=X/$57=attr/$58=tile → $0200+4n → NMI DMA`）。
故该块=**精灵发射序列，只写 OAM，不改游戏状态** ⇒ 属**显示层**，不是数值/状态机逻辑。

**孤儿数据佐证（A 级）**：该块发射的两条记录在九班表里**原样存在**（随 +659 搬到 `$DABC`/`$DACD`）：

| 记录 | JP 表址 | 内容 | CN 表址 | 内容 | 结论 |
|---|---|---|---|---|---|
| Y=$67 | `$D829` | `76 68 2c 00` | `$DABC` | `76 68 2c 00` | 全同 |
| Y=$78 | `$D83A` | `6e 7e 34 00` | `$DACD` | `6e 7e 34 00` | 全同 |

九班侧主循环（`$DA04` 起）遇到 `byte0==0` 记录即返回，**永远走不到这两条** ⇒ 数据留下、代码没了。
数据保留 + 代码消失 = 更符合"补丁侧删代码"，而非"基版差异"（见 §7 H2）。

**全 PRG 签名搜索（A 级，可复现）**：

```bash
python3 -c "
cn=open('rom/gunnac_cn.nes','rb').read()[0x10:0x20010]
jp=open('reference/jp/gunnac_jp.nes','rb').read()[0x10:0x20010]
for h in ['C0 08 D0 05 A0 78','A0 67 20','A9 64 85 56','A9 6C 85 56','AC 80 01 BE']:
    p=bytes.fromhex(h); f=lambda b:[hex(i) for i in range(len(b)) if b.startswith(p,i)]
    print(h,'JP=',f(jp),'CN=',f(cn))"
```

输出：每个签名 **JP=1 处（就是该块），CN=0 处**。

### 2.4 G4 显示数据差异（4B 精灵记录，A 级）

`$D73E` 主循环读表 `$D7C2,Y`（4B/记录，`byte0==0` 终止），JP 8 条 / CN 8 条：

| # | JP `(X,Y,tile,attr)` | CN `(X,Y,tile,attr)` | 字段差 |
|---|---|---|---|
| 1 | `54 70 71 01` | `64 70 71 01` | X |
| 2 | `5e 70 73 01` | `6c 70 73 01` | X |
| 3 | `68 70 75 01` | `74 70 75 01` | X |
| 4 | `72 70 77 01` | `7c 70 77 01` | X |
| 5 | `84 70 79 01` | `84 70 79 01` | — |
| 6 | `8e 70 7b 01` | `8c 70 7b 01` | X |
| 7 | `98 70 77 01` | `94 70 7d 01` | X+tile |
| 8 | `a2 70 7d 01` | `ff ff 31 01` | 全字段（CN 屏外隐藏） |
| 终止 | `00 62 60 02` | `00 65 60 10` | — |

判定：**显示布局/字模索引数据**（Y 与 attr 全同、X 呈 JP 步距 0x0A ↔ CN 步距 0x08，与"日文→中文字宽"一致）。
第 8 条在九班被置为屏外（X=$FF）＝**可见精灵数 8→7**，属显示层差异，非数值表/逻辑参数。
⚠️ 但"少一个精灵"仍是**可见行为差**，与 §2.3 叠加，构成开场画面差异。

---

## 3. bank15（+10B 级移位）逐块分类

### 3.1 全局结构（A 级）

对齐：EQ 块 4224B @ delta=**−0x00A (−10)**，3170B @ delta=0（FF 填充与尾部文本区）。
FF 填充：JP `0x132B–0x16FF`(981B) / CN `0x1321–0x16FF`(991B) ⇒ 差 10B 全由 §3.2 的删除造成，
对齐器把 FF 配到 FF 产生的"CN 新增 10B"是**填充伪影**，非新增内容。

### 3.2 ⚠️ 反例 2：JP `$E093–$E09C`（10B）九班侧消失（A 级）

| 项 | JP | CN |
|---|---|---|
| 代码 | `E093 AD BE 01` `LDA $01BE`<br>`E096 F0 02` `BEQ +2`<br>`E098 A9 00` `LDA #$00`<br>`E09A 8D BE 01` `STA $01BE` | **无** |
| 上文分支 | `E091 D0 16 BNE $E0A9` | `E091 D0 0C BNE $E09F`（目标随 10B 删除更新 ✓） |

语义：初始化路径（`$E000` RESET 之后）强制 `$01BE := 0`（仅在非 0 时写）。

**实效评估（B 级，推断）**：全 PRG 静态扫描 `$01BE` 的读写：

```bash
python3 -c "
for n,rom in (('JP',open('reference/jp/gunnac_jp.nes','rb').read()),('CN',open('rom/gunnac_cn.nes','rb').read())):
    p=rom[0x10:0x20010];h=[]
    for i in range(len(p)-2):
        if p[i+1]==0xBE and p[i+2]==0x01 and p[i] in (0xAD,0x8D,0xAE,0x8E,0xBD,0x9D,0xB9,0x99,0xC6,0xE6,0xCE,0xEE):
            h.append((hex(i),hex(p[i])))
    print(n,h)"
```

输出：

- JP：`0x1427(LDA)`、`0x603c(LDA)`、`0x15326(LDA,X)`、`0x1e093(LDA)`、`0x1e09a(STA)` ← **唯一写者就是被删块**
- CN：`0x147c/0x160e/0x172e(LDA)`、`0x603c(LDA)`、`0x15326(LDA,X)` ← **零写者**

`$01BE` 只作"是否跳过 staff roll / 是否走分支"的门（bank00 `$972E LDA $01BE / BNE`、bank00 `$903C` 段、bank03 `$604C` 段）。
两版在冷启动都经 RESET 的 RAM 清零循环（`$E039–$E042` 清 `$0000–$01FF`）⇒ `$01BE=0`；
JP 的删除块只在"热复位 + SRAM 魔数有效（`$07FE/$07FF=$AC53`）"路径上多写一次 0。
⇒ **推断级（B）：行为中性**；但**字节级删除事实成立**，且"无其他写者"依赖静态扫描完备性（见 §7 R2）。

### 3.3 GAP 分类表（bank15）

| # | 类别 | 处数 | 判定 |
|---|---|---|---|
| H1 | 1B 指令操作数（JSR/JMP 低位，bank 内目标 −10） | 181 | 纯搬家，逐个低字节 −0x0A |
| H2 | 2B 绝对地址操作数 | 23 | 纯搬家 |
| H3 | JP 独有 10B 代码块 | 1 | ⚠️ 反例 2 |
| H4 | 向量/尾部 ASCII 文本 | 11+ | 文本层（见 §3.4） |
| H5 | FF 填充伪影 | 5 | 无内容 |

向量核对（A 级）：NMI `$EF8F→$EF85`、IRQ `$F215→$F20B`（均 −10 ✓）、RESET `$E000` 不变 ✓。

```bash
python3 -c "
jp=open('reference/jp/gunnac_jp.nes','rb').read();cn=open('rom/gunnac_cn.nes','rb').read()
print('JP vec',jp[0x2000A:0x20010].hex(),'CN vec',cn[0x2000A:0x20010].hex())"
```

### 3.4 尾部 `0x1F00–0x1FF9` = ASCII 文本（A 级 → C 级含义）

JP：`1990-06-21 MOU  JIKI MASTER-UP, KOREGA FC SAIGO NO SIGOTONI NARU…`（日文开发者寄语）
CN：`1990-12-17 USA   [SSS/EEE 字形] … AGAIN!`（换版日期 + ASCII 图形）
判定：**文本层改写**（同址异容，结构同为 ASCII 段 + 向量），与"九班=翻译补丁"自洽；非逻辑。

---

## 4. bank00（463B 段 = 文本引擎；其余 170 段分类）

### 4.1 a) JP 初始化例程搬家定位 —— **搬家非删除**（A 级，本次最强证据）

JP 侧读 `$9745` 参数表的例程 = bank00:`$96E3–$9740`（94B）。
签名搜索在九班 bank00 命中 **`$9775`（+0x92 = +146B）**，94B **逐字节对拍仅 13B 不同**：

| JP off | JP 字节 → CN 字节 | 指令上下文 | 性质 |
|---|---|---|---|
| +0x05 | `BB→B1` | `JSR $E8BB → $E8B1` | bank15 目标 −10 |
| +0x11,+0x17,+0x1D,+0x22,+0x27,+0x2C,+0x31,+0x35 | `45→D7,46→D8,47→D9,48→DA,49→DB,4A→DC,4B→DD,4C→DE` | `LDA $9745..$974C,Y → $97D7..$97DE,Y` | 表址 +0x92 |
| +0x47 | `6F→65` | `JSR $ED6F → $ED65` | bank15 −10 |
| +0x4D,+0x51 | `CC→C2` | `JSR/JMP $EBCC → $EBC2` | bank15 −10 |
| +0x5C | `BA→B0` | `JMP $EDBA → $EDB0` | bank15 −10 |

**0 个操作码差、0 个立即数差、0 个分支偏移差** ⇒ **纯搬家**，A 级。

复现：

```bash
python3 -c "
jp=open('reference/jp/gunnac_jp.nes','rb').read();cn=open('rom/gunnac_cn.nes','rb').read()
A=jp[0x10+0x16E3:0x10+0x1741]; B=cn[0x10+0x1775:0x10+0x17D3]
print(len(A),len(B),[(i,hex(a),hex(b)) for i,(a,b) in enumerate(zip(A,B)) if a!=b])"
```

**参数表 `$9745 → $97D7`（5 条 × 8B）字段级核对（A 级）**：

| 字段 | JP 记录0 | CN 记录0 | 判定 |
|---|---|---|---|
| [0][1]（值字段） | `2e 2c` | `2e 2c` | **全同**（5/5 条全同） |
| [2][3]（指针→名称） | `79 ba` | `fb be` | 指针重定向 |
| [4][5]（指针→正文，bit7=标志） | `77 17` | `09 18` | 指针重定向，**bit7 标志两版一致**（0-3 条清零/第 4 条置位） |
| [6][7]（指针） | `6d 97` | `ff 97` | 指针重定向 |

⇒ **数值字段未动、只有地址操作数重定向** = 指针层（翻译必然）。

### 4.2 b) 463B 段逐块复核（bank00 `0x16E3–0x18B1`）

| CN 范围 | 长度 | 内容 | 判定 |
|---|---|---|---|
| `0x16E3–0x1774` | 146B | **新代码**：`LDA ($12),Y / INY / CMP #$20` 偏移+tile 游程解析环；`JSR $ED27/$E5D9` 等待-按键环；`$0180/$01BE/$4C/$4E` 门控；`$14/$15:=$2040 / $10:=$C0` PPU 设置 | **文本渲染/演出驱动代码**（翻译支持）|
| `0x1775–0x17D2` | 94B | JP `$96E3` 初始化例程（§4.1） | 搬家 |
| `0x17D3–0x17D6` | 4B | `00 01 02 03` 索引表 | 全同 |
| `0x17D7–0x17FF` | 41B | 参数表（5×8B + 尾） | 指针重定向 |
| `0x1801–0x18B1` | 177B | 文本数据（JP 文本码/ASCII `COMPILE STAFF`…） | 文本层 |

结论：463B 段=**文本引擎 + 搬家例程 + 指针表 + 文本**，**不含游戏数值/状态机改动** ✓ 与任务预判一致。

### 4.3 c) 其余 170 段分类汇总（bank00 全行）

严格 diff run = **170 段 / 4449B**。按 GAP 级分类（difflib 对齐后 279+96+… 见下）：

| 类别 | 段数 | 字节量级 | 判定 | 证据等级 |
|---|---|---|---|---|
| **1–2B 指令操作数更新** | **98** | ~130B | 纯搬家/指针重写（`abs-operand` 20 + `rel` 1 + `imm` 11 + 1B 操作数高位/低位 66） | A（前字节=操作码） |
| **长段（≥3B）内容改写** | **72** | ~4319B | 分四类，见下 | A/B |
| ├ 文本数据（staff roll / UI 字符串） | ~24 | ~1500B | 文本层（JP 文本码 ↔ 中文/ASCII，`FE/FF` 记录制保留） | A（`COMPILE STAFF`/`SPECIAL THANKS`/`GOLFER…` ASCII 直读） |
| ├ 文本引擎/菜单代码 | ~30 | ~1800B | 引擎层重写（`STA $01AE`/`LDA $0401`/`JSR $E6xx` 段落整体插删，位移量 0/35/111/140/367/392 多值并存 ⇒ 非单一搬家） | A（字节）+B（代码判定） |
| ├ 参数/指针表重定向 | ~10 | ~600B | 指针层 | A |
| └ **未逐块定性** | ~8 | ~400B | **遗留（§7 R1）** | — |

> 说明：bank00 的长段**不是**搬家型（EQ delta 直方图为 0/35/111/140/367/392 多值，且 150 个 GAP 是非对称的插/删），
> 属**内容重写区**（文本引擎 + 文本 + UI）。因此"搬家块内容级等价"的证明义务在 bank00 只覆盖 §4.1 那 94B 例程（已完成），
> 其余长段按"翻译/引擎重写"归类，未逐块给出指令级等价证明（不发明，见 §7 R1）。

复现（分类器）：

```bash
python3 -c "
import sys;sys.path.insert(0,'tools');import da6502 as D
from collections import Counter
jp=open('reference/jp/gunnac_jp.nes','rb').read();cn=open('rom/gunnac_cn.nes','rb').read()
J,C=jp[0x10:0x2010],cn[0x10:0x2010]
runs=[];cur=None
for i in range(0x2000):
    if J[i]!=C[i]: cur=[i,i] if cur is None else [cur[0],i]
    else:
        if cur: runs.append(tuple(cur)); cur=None
if cur: runs.append(tuple(cur))
st=Counter()
for s,e in runs:
    n=e-s+1; op=D.T.get(J[s-1]) if s else None
    st['abs' if (op and n==2 and op[1] in (D.ABS,D.ABX,D.ABY,D.IND)) else
       'rel' if (op and n==1 and op[1]==D.REL) else
       'imm' if (op and n==1 and op[1]==D.IMM) else
       'short' if n<=2 else 'long']+=1
print(len(runs),st)"
```

---

## 5. 九班侧新增内容清单（逐项判定）

| # | 位置 | 大小 | JP 侧对应 | 内容 | 判定 | 结论 |
|---|---|---|---|---|---|---|
| **N1** | bank14 CN `$C000–$C2C5`（off `0x0000–0x02C5`） | **710B** | **JP 该处无内容**（JP bank14 直接从 `0x0000` 起为引擎代码） | **142 × 5B 记录**（值域 `$00–$D9`，第 5 字节 ∈{0,1}，前 7 条呈 16×16 字形四象限规律 `LL=UL+$10 / LR=UR+$10`） | **中文字模/字形组合映射表（数据）** | ✅ **翻译支持数据**，非游戏逻辑 |
| **N2** | bank00 CN `$96E3–$9774` | 146B | JP 该处为 staff roll 初始化例程（已搬走） | 文本记录解析环 + staff roll 演出驱动 + PPU 设置 | **文本渲染引擎/演出驱动（代码）** | ✅ **翻译支持代码**，非游戏逻辑（只写 PPU/OAM 缓冲与演出变量） |
| **N3** | bank15 CN | 0B | — | 无（"新增 10B"= FF 填充伪影） | — | ✅ 无新增 |

N1 复现（结构验证）：

```bash
python3 -c "
cn=open('rom/gunnac_cn.nes','rb').read()[0x1C010:0x1E010]
b=cn[:0x2C6]; r=[b[i:i+5] for i in range(0,0x2C6,5)]
print(len(b),len(r),sorted(set(x[4] for x in r)),
      sum(1 for x in r if x[2]==(x[0]+0x10)&0xFF and x[3]==(x[1]+0x10)&0xFF))"
```

**未发现**任何新增钩子调用（无新增 JSR 指向新代码块）、无新增条件分支、无新增循环体、无新增数值常量。

---

## 6. 裁定

```
W-P1 维持（不关闭）
```

| 命题 | 裁定 | 依据 |
|---|---|---|
| "补丁在搬移区**注入**了游戏逻辑改动" | **不成立**（未找到证据） | §2.2 G1-G3、§3.3 H1-H2、§4.1：全部操作数更新与搬家偏移严格自洽；§5：新增内容 2 项均为翻译支持（字模表/文本引擎），无钩子/无分支改写 |
| "搬移区是**纯布局**（只搬家、不改语义）" | **不成立（被证伪）** | §2.3 反例 1：51B 活代码消失（开场期 OAM 发射）；§3.2 反例 2：10B 初始化代码消失 |

**反例指针**（可直接跳转复核）：

1. `bank14:$D772–$D7A4`（文件 `0x1D782–0x1D7B4`）—— 51B，活代码，显示层；数据表条目仍在（`$DABC/$DACD`）。
2. `bank15:$E093–$E09C`（文件 `0x1E0A3–0x1E0AC`）—— 10B，初始化清理；静态推断实效中性。

**对 L2 94/94 结论的影响**：L2 是"JP 字节执行 ↔ Rust"对拍，**不覆盖 JP↔九班**，故不受本次反例影响；
但"我方移植的行为模型 = 两版共享逻辑真相"这句需要**加限定**：两版在开场显示与 `$01BE` 初始化上**并非逐字节同源**，
应以 JP（社区标准 08ead6a2）为行为基准，九班侧这两处差异登记为已知偏差（见 §7 R3 建议）。

---

## 7. 遗留清单（未定 / 需后续）

| ID | 项 | 现状 | 需要什么才能定谳 |
|---|---|---|---|
| **R1** | bank00 72 个长段中 ~8 段未逐块定性（~400B） | 已归入"内容重写区"，未给指令级等价证明 | 逐段反汇编 + 调用图；或 L3 场景级 JP↔九班双跑 |
| **R2** | 反例 2 的"实效中性"依赖静态扫描无 `$01BE` 其他写者 | B 级（推断） | Mesen2 内存回调：`write $01BE` 全程抓取；若出现间接/索引写者则升级为真实差异 |
| **R3** | 反例 1 的**可见影响面**（开场少哪些精灵） | B 级（代码语义） | Mesen2 JP↔九班开场同步截图 diff（oracle 已有 shots/ 与 shots_jp/ 两套，但因中日字模不同不能直接比；需 OAM 级比对或 PPU 调色板归一化） |
| **R4** | 两处删除的**来源归属**：补丁侧删除 vs 基版差异 | 见 §7 H1/H2 两假设 | 找到九班补丁原始 IPS/PPF 或另一份 JP 早期 dump 作交叉 |
| **R5** | bank14 G4 第 8 条精灵被置屏外（`ff ff 31 01`）是"故意隐藏"还是"版式调整" | C 级 | 同 R3 |

### 假设区（C 级，禁止据此改名/落 spec）

- **H1（补丁侧删除，倾向）**：两处删除均在"搬家区"内、无翻译动机；且反例 1 的数据表条目被保留 ⇒ 更像补丁作者为省空间/改版式而删。
- **H2（基版微差，备选）**：九班补丁可能基于另一 JP dump（`flags6` 0x40→0x41 亦是 dump 变体信号），若如此两处删除早于翻译。
- **H3（N1 表含义）**：710B 表判为"中文字形组合映射表"是 C 级（依据 5B 规整 + 前 7 条四象限规律 + 值域），未做运行时读取追踪。

---

## 8. 自查留痕（哪些判定是推断级）

| 判定 | 等级 | 备注 |
|---|---|---|
| bank14 EQ 块逐字节全同 / delta=+710、+659 | **A** | difflib 匹配块按定义 byte-identical；命令见 §1.2 |
| bank14 147×2B=绝对操作数、4×1B=指针拼装、12×1B=bank15 目标 | **A** | 前字节=操作码 + 操作数值可核（§2.2） |
| bank14 51B 块不存在于 CN 全 PRG | **A** | §2.3 签名搜索，5 个签名 JP 各 1 命中 / CN 0 命中 |
| bank14 51B 块"每帧执行 273 帧" | **B** | 由 `$0188` 语义（N-0200 A 级）+ 分支可达性推得；未跑 trace |
| bank14 51B 块"只写 OAM、不改游戏状态" | **B** | 依 N-0118（`$CFC5`=OAM 发射器）+ 块内无其他 STA/RMW |
| bank14 孤儿数据 `$DABC/$DACD` 无人引用 | **B** | 主循环 `byte0==0` 终止 ⇒ 走不到；未做动态确认 |
| bank15 10B 块删除 + 分支目标更新 | **A** | 字节直读 |
| bank15 10B 块"实效中性" | **B/推断** | 静态扫描无其他 `$01BE` 写者；扫描完备性未证（R2） |
| bank15 向量 −10 自洽 | **A** | 直读 |
| bank00 94B 例程搬家 +146、13B 全为地址低位 | **A** | §4.1 对拍 |
| bank00 参数表"值字段未动、仅指针重定向、bit7 标志一致" | **A** | §4.1 字段表 |
| bank00 146B 新代码="文本渲染/演出驱动" | **B** | 由 `LDA ($12),Y` 解析环 + PPU 参数写入推得；未追调用图 |
| bank00 72 长段四分类 | **B**（代码/文本判定）+**A**（字节） | 分类器见 §4.3；~8 段留 R1 |
| N1=中文字模映射表 | **C** | H3 |
| 两处删除的来源归属 | **未定** | H1/H2 并存 |

**未证不发明声明**：本文件未对任何"未定"块给出结论性命名；R1–R5 全部进遗留清单；
H1–H3 明确标 C 级，未写入 `data/naming/` 任何条目（本次审计未触碰台账）。

**门禁**：`python3 gates/cli.py check`（本次仅新增本文件，未动 `rom/ reference/ data/ gates/`）。
