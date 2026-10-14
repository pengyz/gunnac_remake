# GAME OVER / Continue 机制取证（A2 游戏环缺口②）— 死亡→GAME OVER 触发链/Continue 机制/代价

> 任务：死亡→GAME OVER 触发链、Continue 机制与代价。产出=本文件（增量落盘）。
> **语义口径（前置）**：真死亡 = **`$0400` 1→0x0F**（`$D86B` 转移表 idx1→0x0F，A 级）——**不是** `$0401`
> （$0401=对象网格 slot1 状态/商店光标/CHR 影子三义，见 game-action-mode-domain.md）。
> "209f 节拍"=投放物行为链，与死亡无关（AR57-3）。九班=美版基底（D3①），分歧以 JP 为语义权威。
> 证据分级依 AGENTS.md §3；本文件结论与假设分区，假设不入结论。

## 0. 结论速览（TL;DR）

| # | 结论 | 级 |
|---|---|---|
| 1 | 死亡=碰撞事件经 `bank14:D613/D654` 转移例程写 `$0400:=0x0F`（+`$041C:=0`、`$04A8:=旧态`、`$0668\|=$10`），**四写签名与 trace 双事件逐一吻合** | A |
| 2 | 0x0F 行为=自向量 `$A3:$A5`（`JMP $A3A5`）死亡窗口：移动冻结+爆炸动画（`$05DC` 帧索引 0→39）+**命数 `$018D` BCD −1**+**金钱 `$018E` ÷2**，末帧 `JMP $D10C` 释放 `$0400:=0` | A |
| 3 | 释放后主循环 `bank15:E29A` 分叉：`$26&8==0`→复活（弹药 `max(4,$18F)`、`$0400:=1`、`$0710:=1`）；`$26&8≠0`→GAME OVER 链 | A |
| 4 | **`$018D` = 命数（BCD）**：stage 开局 `:=2`（=共 3 机），每次死亡 −1；**归零后再死**才置 `$26\|=8`（GAME OVER 待决） | A |
| 5 | GAME OVER 菜单 `bank15:$E3AB-$E484`：文本 "CONTINUE"/"GAME OVER"（`bank15:$E4A0` 记录块）+ 光标 `$2F` 二选一，**Start 确认**；`$2F==0`→续关 `sub_E0D9`，`$2F==1`→回标题 `loc_E073` | A（静态） |
| 6 | Continue 代价：入口 `$017A-$017F` 逐字节减半变换（语义开放；【2026-10-14 注（#63 AR63-7）】实测位级=6 位数量÷2 存 5 位）；续关复位 `\$18D:=2 / \$189,\$190,\$18E:=0 / 网格全清`；~~**分数 `$E0-$E6`、关口号 `$180` 保留**~~ 【2026-10-14 更正（复核 #63 AR63-1）】"保留 $E0-$E6"**被证伪**——续关 `bank15:$E10A JSR sub_ECBF`（ECBF-ECC9 清零 $E0-$E6 全 7 字节，进菜单必经 $26 bit7=0）⇒ **双金钱清零（$E0-$E6+$018E）**，仅 $180 关号保留；原"E0D9 段无写点"=内联扫描假负结果（漏子程序调用） | A（写点）/ C（$17A 语义） |
| 7 | 双版：`$D86B` 表 CN==JP **逐字节全等**；死亡结算 bank03 JP Δ0 同址；bank15 菜单链 JP **+0x0A 漂移（效应段口径——【2026-10-14 收窄（#63 AR63-8）】非整体，JP 调用点 E065 同址即反例）**；bank14 族 relocation（−0x2C6 同块）；**US==CN 全链逐字节全等**（九班未触碰本链） | A |
| 8 | **负结果**：现有双版 trace 场景段**未达 GAME OVER 屏**（两次死亡后 `$18D=0` 仍复活；第三次死才进 GAME OVER）——菜单/续关=静态定谳，动态证据止于 `$26\|=8` 前一步 | A（负结果） |

---

## 1. 死亡语义链（触发 → 0x0F 行为 → 释放/复活）

### 1.1 触发：碰撞判定 → 状态转移（A 级代码 + A 级 trace 签名）

**转移应用点**（两处同构，均在 bank14）：

```
bank14:D613  20 2F D6   JSR  sub_D62F        ; 见下：置 $668+Y bit $10
bank14:D616  BC 00 04   LDY  $400,X
bank14:D619  B9 6B D8   LDA  $D86B,Y         ; ★状态转移表
bank14:D61C  DD 00 04   CMP  $400,X
bank14:D61F  F0 0C      BEQ  +12             ; 恒等→不动
bank14:D621  9D 00 04   STA  $400,X          ; ★新状态（1→0x0F）
bank14:D624  98         TYA
bank14:D625  9D A8 04   STA  $4A8,X          ; ★旧状态备份
bank14:D628  A9 00      LDA  #0
bank14:D62A  9D 1C 04   STA  $41C,X          ; ★状态向量/计时清零
bank14:D62D  38         SEC                  ; =命中成立
bank14:D62E  60         RTS
（Y 版同构：bank14:D654-D673，写 $400,Y/$4A8,Y/$41C,Y）
sub_D62F 尾：bank14:D64C-D651  LDA $668,Y / ORA #$10 / STA $668,Y   ; ★hit flag
```

**trace 四写签名**（CN gameplay-stage1，死亡帧 f1787 / f2046，全 RAM（zp+iram $0300-$07FF+OAM+SRAM）严格双事件过滤）：

| 字节 | f1786→1787 | f2045→2046 | 对应写点 |
|---|---|---|---|
| `$0400` | 01→**0F** | 01→**0F** | `D621/D664`（`STA $400`） |
| `$041C` | A0→**00** | A0→**00** | `D62A/D66E`（`STA $41C,X`） |
| `$04A8` | 00→**01** | （=1 不变） | `D625/D669`（旧态备份） |
| `$0668` | 00→**10** | 00→**10** | `D64C-D651`（`ORA #$10`） |

四写全中、双事件一致 ⇒ **触发写点定谳**（A 级）。

**判定盒**：`sub_D7A3`（`bank14:D7A3-$D7EC`）= 对象槽 X 对命中盒表（`$7A9/$7BE/$77F/$794` 索引 Y）逐条比对 + 掩码测试 `LDA $4C4,X / AND $7E8,Y / BNE 命中`；命中→`SEC`。调用方=bank04-07 行为脚本经 `JMP $D674/$D60E/$D5E7/$D5F3/$D5FD`（xref 全列）进入转移。H-16 §2 已推定盒包络 |dx|≤12、|dy|≤12（2 样本）；**玩家被击的最上游调用点仍 W15（H-16 遗留）**，本链自"转移应用"起为 A 级，之上一段 B 级（同族碰撞链）。

### 1.2 `$D86B` 状态转移表全表（A 级 dump，CN==JP 全等）

表址 `bank14:$D86B`（PRG 0x1D86B）。**转移条件**：表仅在碰撞事件时应用（`D613/D654` 路径），非每帧扫描。

```
idx 00-0F: 00 0f 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f
idx 10-1F: 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f
idx 20-2F: 11 11 12 11 11 11 12 12 12 12 2a 12 2c 2d 2e 2f
idx 30-3F: 12 13 12 13 12 12 12 12 12 13 13 14 13 3d 13 12
```

| 观察 | 含义 | 级 |
|---|---|---|
| `table[1]=0x0F` | **玩家 1→死亡**（与 trace 互证） | A |
| `table[0x0F]=0x0F` 恒等 | **死亡窗口内复撞无效**（防重入）；=0x0F 期"无敌"的表级机制 | A（表值）/B（无敌=行为推断） |
| `table[7]=7` 等 idx<0x20 恒等（除 [1]） | 稳定态不受转移影响 | A |
| idx 0x20+ 映射 0x11/0x12/2A/2C-2F/3D 族 | 流送/道具态受击降级族（码域完整枚举=game-action-mode 残项） | A（写值）/C（语义） |
| JP 对照（`$D5A5`，relocation 见 §5）**0x40 字节全等** | 表本体双版零差异 | A |

### 1.3 0x0F 行为：死亡窗口（A 级 trace + A 级向量链）

**行为分发器** `bank02:82F8-8311`：

```
82F8  LDA $41C,X / BPL 8307     ; bit7=1 → 自向量；0 → 表分发
82FD  STA $25 / LDA $438,X / STA $24 / JMP ($24)   ; 自向量=$41C:$438
8307  LDA $80CE,Y / STA $25 / LDA $7FFF,Y / STA $24 / JMP ($24)  ; 表[Y]（Y=状态码）
8314  DEC $438,X / BEQ → JMP $D10C                  ; 相位倒数尽→释放（另一族）
```

- 表分发 **`Y=0x0F` → 向量 `bank03:$A33E`**（表实取：lo 表 `$7FFF+Y`/hi 表 `$80CE+Y`）。
- `A33E`：`LDA $6A0 / BEQ A34B / LDA #1 / STA $400 / JMP $A039`——`$06A0≠0`（stage-clear 保护，`bank15:E32E:=2`）则**免死立即复活**。
- `A34B`：`$32 bit7` 分叉——置位=非标准模式（音效 $22 + `LDA #$78/JSR $916E` + 立即复活）；**清位（常规）→ `A36E` 死亡结算**：

```
A36E  LDA $32 / AND #4 / BEQ / (清 $33/$34) / JSR $9177
A37F  JSR $CDBF
A382  LDA $18D / BNE A392        ; ★命数（BCD）
A387  JSR sub_ED56 / LDA $26 / ORA #8 / STA $26   ; ★命数已 0→GAME OVER 待决
A390  D0 08        BNE loc_A39A                ; ★终死跳过减一（【2026-10-14 补（#63 AR63-2）】$18D==0 时置 $26|=8 后跳过 BCD 减一，$18D 恒 0——无借位，sub_ED03 钳 0 兜底）
A392  LDY #1 / JSR sub_ED03 / STA $18D            ; ★命数 BCD −1（sub_ED03=BCD 减法；**仅非终死执行**）
A39A  LDA #$21 / JSR $ED3B       ; 死亡音效
A39F  LSR $18E                   ; ★金钱减半（已登记 shop-purchase-flow #55）
A3A2  JSR $CB4F                  ; ★自向量 trampoline
```

- **`bank14:$CB4F` trampoline**（`PLA/CLC/ADC#1/STA $438,X / PLA/ADC#0/STA $41C,X / **STA $25**（CB5E）/ **STA $24**（CB56）/ JMP ($24)`——【2026-10-14 补（#63 AR63-4）】二指令漏则 JMP($24) 断链）=
  把 JSR 返回点装入槽行为向量并跳转 ⇒ **`JSR $CB4F` 装入 `$A3:$A5`（=返回点 `$A3A5`）**。
  trace 实证：f1788 起 `$041C/$0438 = $A3/$A5` 常驻至窗口末帧（双事件一致，A 级）。
- **窗口行为 `bank03:$A3A5`**（每帧经自向量进入）：`LDA #$50/LDY #$28/JSR $CD9D`（爆炸相位/帧计时）
  → 形状表 `$A3ED/$A3F5` 与 `$A3FD/$A405`（**以 `$05DC+X` 为动画帧索引**，trace 实测 0→39）→ `JSR $CC65`
  → 动画尽：`LDA #0/STA $49 / JSR $EEFB / JSR $920B / JMP $D10C`（★释放）。
- **0x0F 行为小结**：移动冻结（respawn-window R-respawn-2 A 级：x/y/sx/sy 恒定、输入采集无效）、
  投放例程调用方层封锁（H-16/W3）、爆炸动画推进、死亡结算一次性（命数/金钱在首帧扣）。
  窗长 gameplay-stage1 实测**两次均 128f**（f1787-1914 / f2046-2173）；respawn-window 场景冻结窗
  85f/129f（窗长随事件源变化 → H-16 续问维持）。

### 1.4 释放 → 复活（A 级）

```
bank14:D10C  A9 00      LDA #0
bank14:D10E  9D 00 04   STA $400,X      ; ★释放：状态:=0
bank14:D111  9D 1C 04   STA $41C,X      ; 向量清零（+后继清字段）
────────────────────────────────────────
bank15:E29A  AD 00 04   LDA $400        ; 主循环逐帧检查
bank15:E29D  F0 03      BEQ E2A2        ; ==0 → 复活/分叉
bank15:E29F  4C F3 E2   JMP E2F3        ; ≠0 → 常规
bank15:E2A2  A5 26 / 29 08 / D0 17      ; $26 bit3 → GAME OVER 链（§2）
bank15:E2A8  AD 8F 01   LDA $18F        ; 复活路径：
bank15:E2AB  C9 04 / B0 02 / A9 04      ;   $18F := max(4,$18F)（弹药回满）
bank15:E2B1  8D 8F 01   STA $18F
bank15:E2B4  A9 01 / 8D 00 04           ;   $0400 := 1（复活）
bank15:E2B9  8D 10 07   STA $710        ;   $0710 := 1（字段语义开放）
```

- 同帧内 `0x0F→0`（释放）→ `0→1`（复活），watch 只见 `0x0F→1`（A 级：f1915/f2174）。
- trace 佐证：f1915 ammo 1→**4**、f2174 ammo 3→**4**（`max(4,·)` 两样本 ✓）、`$0710` 0→1（f1920 dump ✓）、
  `$041C` A3→00、`$0668` 10→00。
- 行为重启：f1916 `$041C/$0438=$A0/$3C`（自向量 `$A03C`）= 表[1] `$A000` 首帧 → `CB4F` 链 → 常态飞行动作向量。

**死亡语义链验收①：A 级（触发写点/表/窗口行为/释放/复活）+ B 级（判定盒上游 W15）指针齐备 ✓**

---

## 2. GAME OVER 触发链（核心）

### 2.1 命数计数器：`$018D`（BCD）——A 级（写点 + 动态双证）

| 事实 | 证据 | 级 |
|---|---|---|
| `$018D` = 命数（**BCD**），stage 开局 `:=2`（实操=共 3 机：2 次免费复活+第 3 死进 GAME OVER） | `bank15:$E0EF: A9 02 / 8D 8D 01`（写点）+ ram.hex f925 起 `$018D=2` | A |
| 每次死亡 −1（BCD 减 1） | `bank03:$A392: LDY #1 / JSR sub_ED03 / STA $18D`（sub_ED03=BCD 减法：`STY 6/STA 4/AND #$F0…SBC…ADC #$A` 全表）；ram.hex：f1740=2 → f1800=**1**（死 1@f1787）→ f2100=**0**（死 2@f2046） | A |
| 归零后再死 → 置 GAME OVER 待决 | `bank03:$A382-A38E: LDA $18D / BNE 减一 / JSR sub_ED56 / LDA $26 / ORA #8 / STA $26`（$26 bit3 = **不可复活位**） | A |
| `$26 bit3` stage 开局清零 | `bank15:$E0F4-E0F8: LDA $26 / AND #$F7 / ORA #$10 / STA $26`（续关/开局重置复活权） | A |

> 注：`$018D` 不在既有 frames watch 集内（watch 只含 $0188/$018E/$018F/$0190），故 t40 系列逐帧 diff 未见它；
> 本轮用 probe ram.hex（60 帧全 RAM dump）+ 写点双证补齐。**建议后续场景加 `watch 018d lives`**。

### 2.2 触发分支链（A 级静态链，全指针）

```
[死亡释放] bank14:D10C  $0400:=0
   ↓ 主循环 bank15:E29A（逐帧）
   ├─ $26&8 == 0 → E2A2 复活（§1.4）
   └─ $26&8 ≠ 0 → E2BF GAME OVER 链：
        E2BF  LDA $27 / ORA #$40 / CMP $27 / BEQ …   ; 一次性入场（$27 bit6）
        E2C9  LDA $26 / BPL / JMP loc_E073           ; bit7（模式位）→ 直回标题
        E2D0  LDA #$FF / STA $188 / LDA #0 / STA $300
        E2DA  JSR sub_ED56 / LDA #$11 / JSR sub_ED3B ; ★GAME OVER 音效 $11
        E2E2  LDA $16 / AND #$10 / BNE E2F0          ; Start（$16=pressed）
        E2E8  LDA $300 / BNE E2F0                    ; 或 $0300≠0
        E2ED  JMP sub_E21C                           ; 否则停留等待
        E2F0  JMP loc_E3AB                           ; ★→ GAME OVER/CONTINUE 菜单
```

### 2.3 文本记录与装载点（A 级字节）

- **文本串**（grep rom 字节，ASCII 直存，符合 text-code-table t150"文本 tile 直存"）：
  - CN/US：file **0x1e4de** `CONTINUE`、file **0x1e4e9** `GAME OVER`（`bank15:$E4CE/$E4D9`）；
  - JP：file 0x1e4e8/0x1e4f3（`bank15:$E4D8/$E4E3`，+0x0A 漂移，内容同英文）。
- **记录格式**（text-record-write t117 流）：`[ppu_lo ppu_hi] ASCII 0xFE … 0xFF`；流基址 `bank15:$E4A0`
  （`E418` 装载指针），四标签记录自 `$E4B9` 位置对起：
  | 串 | PPU 地址 | 位置（行,列） |
  |---|---|---|
  | ` HI SCORE` | $20CC | (6,12) |
  | `SCORE` | $214C | (10,12) |
  | `CONTINUE` | $224C | (18,12) |
  | `GAME OVER` | $22AC | (21,12) |
- **装载点**：`bank15:$E418-E422: LDA #$A0/STA $75 / LDA #$E4/STA $76`（流指针 `$75/$76:=$E4A0`）
  `+ LDA #4 / JSR sub_EDB0`；经 `sub_E487` 光标 + `$CDDE/$CEF1` 帧服务。

### 2.4 菜单循环 `bank15:$E3AB-$E484`（A 级静态链）

```
E3AB-E415  场景装载（CHR 对 $7C/$60/$66、清表、$E4E3 调色板、文本流）
E418  $75/$76 := $E4A0（文本块）          E425 ← 循环顶
E428  LDA #$90 / LDY $2F / BEQ / LDA #$10 / STA $64   ; 光标闪烁参数按 $2F
E432  INC $2B / JSR $CDDE / JSR sub_E59D（自动连发输入包装）
E43A  AND #$10 / BNE E472                 ; ★Start 确认（$16 位 $10）
E43E  LDA $16 / AND #$20（Select）→ 翻转  ; E444 AND #$08（Up，仅 $2F=1）→ 翻转
E44E  AND #$04（Down，仅 $2F=0）→ 翻转    ; E458 $2F := $2F^1 + 音效 $45
E463  LDA $2B / AND #4 / JSR sub_E487     ; 光标重绘（$55:=$8A/$A2 两档）
E46F  JMP E425                            ; 循环
E472  JSR sub_E487 / JSR $CEF1 / LDA #4 / JSR sub_EE03
E47D  LDA $2F / BNE E484
E481  JMP sub_E0D9                        ; ★CONTINUE → 续关（§3）
E484  JMP loc_E073                        ; ★GAME OVER → 回标题（§3）
sub_E487: 光标精灵（$55:=$8A if $2F==0 else $A2 / $56:=$50 / $58:=$4F）
          → $8A=138≈行17-18（CONTINUE 行18）/ $A2=162≈行21（GAME OVER 行21）✓
```

- 确认键=**Start**（`sub_E59D` = 连发包装：`$16/$18` pressed/held + `$4B` 重复计时），方向/Select 只翻转选项。

### 2.5 $017A-$017F 入口变换（~~"分数区"~~【2026-10-14 更正（#63 AR63-3 术语终局）】$E0-$E6 定名 **money_counter_bcd（金钱计数器，N-0028）**；"分数"=W2 开放假设（C 级）不得入结论；$018E=**商店金钱**——本文档原双"金钱/分数"用语以本注为准）（`$017A-$017F`，A 级写点 / C 级语义）

```
E3B9  LDX #5
E3BB  LDA $17A,X / PHA / AND #$C0 / STA $04 / PLA / LSR A / AND #$1F / ORA $04 / STA $17A,X
E3CC  DEX / BPL E3BB                      ; 逐字节：保 bit7-6、~~低 5 位减半（bit5 丢）~~【2026-10-14 原地更正（#64 AR63-7）】实为 6 位数量÷2 存 5 位（丢 bit0、bit5→bit4）
```

- 全 ROM 该字段**仅 3 个触点**（`E0D3` 标题清零 / `E3BB` 读 / `E3C9` 写）。
- 与发货槽数组 `$0178-$017F`（N-0201，社区打包 `>>6=目的地、&63=数量`）重叠 ⇒ **若打包成立，本变换=槽 2-7 数量减半（目的地位保留）**=续关代价（B 级待核）。
- 映射 `bank00:$9FDA STA $178,Y` 为数组写点（N-0007/N-0201 语义未合流）→ **`$017A-$017F` 语义开放（C 级，残项 R-G1）**。

### 2.6 显式负结果（验收②的负结果声明）

- **现有双版 trace 场景段未达 GAME OVER 屏**：CN gameplay-stage1 两次死亡后 `$018D=0`，但第三次死亡才会置
  `$26|=8`；respawn-window 亦只有 2 事件。JP 各场景零死亡（投放例程调用方层封锁，H-16/W3）。
  ⇒ 菜单/续关/文本装载为**静态链定谳**（上表全部 A 级代码指针 + 文本字节），动态证据链止于
  "`$26|=8` 前一步"（死亡结算 A382-A39F 已动态双证）。
- 复现到 GAME OVER 的最小场景设计（建议交 verifier/场景组）：gameplay-stage1 输入轨道 + 追加 1 次死亡
  （或直接 `ram_poke 018d 00` 后 1 次死亡）→ 期待 `$26|=8` → Start → `$0400/$041C/$0438` 变为菜单态、
  PPU $224C/$22AC 出现 CONTINUE/GAME OVER。

**GAME OVER 触发链验收②：定谳（触发条件+分支链+文本+菜单全指针）+ 显式负结果（未达屏）✓**

---

## 3. Continue 机制（A 级静态链 + RAM 复位范围表）

### 3.1 两个出口

| 选项 | 光标 | 出口 | 语义 |
|---|---|---|---|
| **CONTINUE** | `$2F=0`，`$55=$8A` | `bank15:$E481 → JMP sub_E0D9` | **续关**：重开当前关（`$180` 关口号保留） |
| **GAME OVER** | `$2F=1`，`$55=$A2` | `bank15:$E484 → JMP loc_E073` | 结束：回标题（`$180:=1`） |

### 3.2 续关复位范围（`sub_E0D9` 入口全走 vs 关间跳转 `loc_E16B`）

关键结构：**关间推进不走 `E0D9` 顶部，而是 `bank15:$E396 → JMP loc_E16B`（E16B 位于 E160-E168 清零段之后）**——
即"每关开始"与"续关"共用 E16B 尾段，但**续关多走顶部初始化**：

| 处置 | 字段 | 写点 | 说明 |
|---|---|---|---|
| **清零（仅续关/开局）** | `$018D` **命数 :=2** | `bank15:$E0EF` | 复活权重置 |
| | `$0189 / $0190 / $018E` := 0 | `bank15:$E160-E168` | **金钱清零**（$018E）+ 炸弹类型索引（$0190）+ $0189 |
| | 对象网格 `$0400-$06FF` + `$06FE-$07FF` 全清 | `bank15:$E18C` 循环（`STA $400,X/$500,X/$600,X/$6FE,X`） | ~~28 槽全灭~~【2026-10-14 原地更正（#64 AR63-5）】槽计数口径同修（网格范围已正 $6FE-$7FD） |
| | `$C8 := 0`、`$27/$16/$18` 清 | `E19B / E1C1-E1C7` | 计数/输入残留 |
| **重设** | `$0400 := 1`（玩家态） | `bank15:$E1B6` | 复活 |
| | `$188 := $40（或模式位下 $FF）` | `bank15:$E1DD` | intro 计数 |
| **保留** | `$180` 关口号 | （无写）E1F9 按 `$180` 取关乐 `LDA $E50D,Y` | **续当前关** ✓ |
| | ~~`$E0-$E6` 玩法所得 BCD 分数｜（E0D9 段无写点）｜**分数保留**~~ 【2026-10-14 更正（复核 #63 AR63-1）】"保留 $E0-$E6"**被证伪**——续关 `bank15:$E10A JSR sub_ECBF`（ECBF-ECC9 清零 $E0-$E6 全 7 字节，进菜单必经 $26 bit7=0）⇒ **双金钱清零（$E0-$E6+$018E）**，仅 $180 关号保留；原"E0D9 段无写点"=内联扫描假负结果（漏子程序调用）（sub_ECBF 调用清零） |
| | `$017A-$017F` | 入口已减半（§2.5）；标题路径才清（`E0D1-E0D7`，续关跳过） | 减半后带入 |

> 对照：**关间推进**（`loc_E16B`）不经 `$18D:=2`/金钱清零/网格全清 ⇒ 命数、金钱跨关携带（B 级结构推断，
> E16B 落点在清零段之后=A 级指针）。

### 3.3 代价汇总（死亡 / GAME OVER / Continue 三级）

| 事件 | 代价 | 写点 | 级 |
|---|---|---|---|
| 每次死亡 | 命数 −1（BCD）、**金钱 `$018E` ÷2** | `bank03:$A392 / $A39F` | A |
| GAME OVER 入口 | `$017A-$017F` 逐字节减半变换（疑=发货槽 2-7 数量减半） | `bank15:$E3B9-E3CD` | A（写点）/C（语义） |
| Continue | 金钱清零、炸弹类型索引清零、命数重置 2、网格重置；~~**分数/关号保留**~~ 【2026-10-14 更正（复核 #63 AR63-1）】"保留 $E0-$E6"**被证伪**——续关 `bank15:$E10A JSR sub_ECBF`（ECBF-ECC9 清零 $E0-$E6 全 7 字节，进菜单必经 $26 bit7=0）⇒ **双金钱清零（$E0-$E6+$018E）**，仅 $180 关号保留；原"E0D9 段无写点"=内联扫描假负结果（漏子程序调用） | `bank15:$E0EF/$E168/$E18C` | A |
| GAME OVER（结束） | 回标题；`$17A-$17F` 清零、`$32:=$40`、`$3C:=0`、`$F1/$4E` 复位 | `bank15:$E073-E0D7` | A |

### 3.4 模式位注（C 级假设区，不入结论）

- `$26 bit7`：置位时死亡→直接 `JMP loc_E073`（跳过菜单），且 `$188:=$FF`/`$32 bit7` 联动——疑似演示/特殊模式旗标。
- `$32 bit7`：置位时 0x0F 行为走"速爆+立即复活"分支（`A34B → A34F`），不走命数/金钱结算。
- `$06A0`（槽 0 字段，`E32E := 2` 于关末）：非 0 时 0x0F 免死立即复活（`A33E`）。
  以上三者语义未定谳（C 级，归入 game-action-mode 域残项一并考证）。

## 4. 状态转移图 + flow.rs 边集扩展建议（验收④）

### 4.1 状态转移图（玩家死亡/结束环）

```
            ┌────────────────────────────────────────────────────────┐
            │                                                        │
[ALIVE $0400=1]──碰撞(D613/D654 转移, $D86B[1]=0x0F)──▶[DEAD $0400=0x0F]
            ▲                                                        │
            │                                        128f 窗（$A3A5 爆炸链，
            │                                         $05DC 帧索引 0→39，
            │                                         命数-1/金钱÷2 首帧）
            │                                                        ▼
            │                                            [释放 $0400=0 (D10C)]
            │                                                        │
            │                          ┌── $26&8==0（E2A2）──────────┤
            │                          ▼                             │
            └───── $0400:=1 / 弹药 max(4) / $0710:=1 ◀───────────────┤
                                                                     │ $26&8≠0（E2BF）
                                                                     ▼
                                                    [GAME OVER 待决（$27|=$40, 音效$11）]
                                                                     │ Start / $0300≠0
                                                                     ▼
                                                    [GAME OVER 菜单 E3AB（$2F 光标）]
                                                        │                        │
                                        $2F=0 + Start   │                        │  $2F=1 + Start
                                                        ▼                        ▼
                                        [续关 sub_E0D9]（$180 保留，        [回标题 loc_E073]
                                         $18D:=2/$18E:=0/网格清）           （$180:=1，$17A 清）
                                                        │                        │
                                                        ▼                        ▼
                                                   [ALIVE 关首]              [TITLE]
```

### 4.2 flow 边集扩展建议表（供 flow.rs fail-closed 段补边；建议全部入 t81 同级规则）

| # | 边（from → to） | guard | 证据（A 级指针） | trace 锚 | 建议规则名 |
|---|---|---|---|---|---|
| F1 | player.alive(1) → player.dead(0x0F) | 碰撞命中（转移表 `$D86B[1]`） | `bank14:$D613-D62E`/`$D654-D673`；四写签名 `$0400/$041C/$04A8/$0668` | gameplay-stage1 f1787、f2046 | R-death-trigger |
| F2 | player.dead(0x0F) → player.dead(0x0F) | 自环（复撞无效） | `$D86B[0x0F]=0x0F` 恒等 | 窗内 128f `$0400` 恒 0x0F | R-death-immune |
| F3 | player.dead(0x0F) → player.released(0) | 窗尽（爆炸链尾） | `bank03:$A3EA JMP $D10C` → `bank14:$D10E STA $400,X` | f1915/f2174 同帧 0F→0→1 | R-death-release |
| F4 | player.released(0) → player.alive(1) | `$26&8==0` | `bank15:$E2A2-E2BC`（`$18F:=max(4,…)/$0400:=1/$0710:=1`） | f1915 ammo 1→4、f2174 3→4 | R-respawn |
| F5 | ~~player.dead → player.alive~~【2026-10-14 修（#63 AR63-6）】**dead 态首帧**（非复活边）| 首帧结算：`$18D` BCD −1（终死跳过）、`$18E` ÷2 | `bank03:$A392/$A39F` | ram.hex f1740=2→f1800=1→f2100=0 | R-death-cost |
| F6 | player.dead → gameover.pending | `$18D==0` 时死亡 | `bank03:$A387-A38E`（`$26\|=8`） | （静态；动态缺=§2.6） | R-gameover-arm |
| F7 | player.released(0) → gameover.pending | `$26&8≠0` | `bank15:$E2BF-E2E2`（`$27\|=$40`、音效 $11） | （静态） | R-gameover-enter |
| F8 | gameover.pending → gameover.menu | Start（`$16&$10`）或 `$0300≠0` | `bank15:$E2E2-E2F0` | （静态） | R-gameover-menu |
| F9 | gameover.menu → stage.start | `$2F==0`+Start | `bank15:$E47D-E481 → sub_E0D9` | （静态） | R-continue |
| F10 | gameover.menu → title | `$2F==1`+Start | `bank15:$E484 → loc_E073` | （静态） | R-gameover-finish |
| F11 | gameover.menu → gameover.menu | Up/Down/Select 翻转 `$2F^=1`+光标重绘 | `bank15:$E43E-E463`、`sub_E487` | （静态） | R-menu-toggle |
| F12 | stage.start → alive | 续关复位集（`$18D:=2/$189,$190,$18E:=0/**$E0-$E6:=0**（sub_ECBF）/网格清 **$6FE-$7FD**（AR63-5）/$0400:=1`） | `bank15:$E0EF/$E160-E168/$E18C/$E1B6` | （静态） | R-continue-reset |
| F13 | dead(0x0F) 免死支 | `$06A0≠0` 或 `$32 bit7`（模式）→ 立即复活 | `bank03:$A33E-$A36B` | （静态，C 级语义） | R-death-bypass（暂挂） |

> 说明：F1-F5 为 A 级动态+静态双证；F6-F12 为 A 级静态链（动态待 §2.6 场景补采）；F13 依赖模式位语义（C 级），
> 建议 flow.rs 先以 fail-closed 标注待解、不入行为断言。

---

## 5. 双版对照（JP 语义一致性 + 美版基底注）

### 5.1 坐标桥换算（`data/reference/addr-bridge-jp-cn.json`，未手工换算）

| CN 点位 | 语义 | 桥结果 | JP 点位 | 字节比对 |
|---|---|---|---|---|
| `bank03:$A382` | 命数检查/递减 | **same-site Δ0** | `bank03:$A382` | 仅 JSR 目标随 bank15 漂移（`20 56 ED`→`20 60 ED`）✓ |
| `bank03:$A3A5` | 死亡窗行为 | **same-site Δ0** | `bank03:$A3A5` | 仅 JSR 目标随 bank14 漂移（`20 9D CD`→`20 D7 CA`）✓ |
| `bank03:$A33E/$A36E/$A38C/$A39F` | 死亡链其余 | 桥块级 gap（unmapped） | 模式验证同址（PRG 0x633E/636E/638C/639F） | 结构全同 |
| `bank15:$E29A/$E2A2/$E2A8/$E3AB/$E47D/$E4CE/$E4D9` | 主循环/菜单/文本 | 桥块级 gap | 模式验证**一致 +0x0A 漂移**：`$E2A4/$E2AC/$E2B2/$E3B5/$E487/$E4D8/$E4E3` | 同构指令序列逐条命中；`$E3AB→$E3B5` 调用目标漂移精确吻合（`$EE03→$EE0D`、`$E590→$E59A` 均 +0x0A；`$DB14→$D881` = bank14 −0x293） |
| `bank14:$D86B` | 状态转移表 | gap（relocation 域） | `$D5A5` | **0x40 字节全等** |
| `bank14:$CB4F/$D10C` | trampoline/释放 | gap（relocation 域） | `$C889/$CE46` | 同构（CN=JP+0x2C6 同块漂移，与 w-p1-relocation-audit B 段口径一致） |

- 桥覆盖缺口如实声明：bank02/03/14/15 相关块在桥中为 `unmapped`/gap（生成器块级粒度），
  本表 JP 点位=**字节模式验证**（非手工坐标换算）；bank14 漂移与 `docs/findings/w-p1-relocation-audit.md` 互证。

### 5.2 JP 语义一致性结论

| 项 | 结论 | 级 |
|---|---|---|
| `$D86B` 转移表 | CN==JP **全等**（idx1→0x0F 同在） | A |
| 死亡结算（命数 BCD −1 / `$26\|=8` / 金钱 ÷2） | JP 同址同构（bank03 Δ0） | A |
| 复活/分叉（`$0400==0` 检查、`max(4)` 弹药、`$26&8` 分支） | JP 同构（bank15 +0x0A 一致漂移） | A |
| GAME OVER/CONTINUE 文本与菜单分叉 | JP 同英文串、同 `A5 2F D0 03`（`$2F` 分叉）模式 | A |
| 结构差异层 | ~~bank15 整体 +0x0A~~【2026-10-14 原地更正（#64 AR63-8）】**效应段口径**（JP 调用点 E065 同址即反例）、bank14 relocation（−0x2C6 同块）、bank03 零漂移 | A |

### 5.3 美版基底注（九班 vs US）

- **US==CN 逐字节全等**（三段实测）：`bank15` GAME OVER/菜单链 0x1E290-0x1E520、`bank03` 死亡段 0x633E-0x63B0、
  文本块 0x1E4B0-0x1E4E2（含 CONTINUE/GAME OVER 串）。
- ⇒ **九班汉化未触碰死亡→GAME OVER→Continue 全链**（文本保留英文）；九班↔US 差异层在别处（剧情文本/CHR，
  见 rom-forensics / text 系 findings）；本链行为 = US 基底原样（D3① 与实测一致）。JP 侧为语义权威且本链与 US 基底同构。

## 6. 复现命令 + 验收清单 + 遗留

### 6.1 复现命令（独立重导）

```bash
# ① 死亡四写签名 + $0400 时间线（CN gameplay-stage1）
python3 - <<'PY'
import json
rows=[]
for line in open('analysis/traces/gameplay-stage1.frames.jsonl'):
    r=json.loads(line); d=r['w']; ir=bytes.fromhex(d['iram'])
    rows.append((r['f'], ir[0x100], ir[0x11C], ir[0x1A8], ir[0x368]))  # $400 $41C $4A8 $668
prev=None
for f,a,b,c,d in rows:
    if (a,b,c,d)!=prev: print(f'f{f}: $400={a:#04x} $41C={b:#04x} $4A8={c:#04x} $668={d:#04x}'); prev=(a,b,c,d)
PY
# ② 命数动态（ram.hex 全 RAM 60 帧采样）
python3 - <<'PY'
import glob,re
for p in sorted(glob.glob('analysis/traces/gameplay-stage1_f*.ram.hex'), key=lambda p:int(re.search(r'_f(\d+)',p).group(1))):
    f=int(re.search(r'_f(\d+)',p).group(1)); ram=bytes.fromhex(open(p).read().strip())
    if f in (925,1200,1740,1800,1920,2040,2100,2220): print(f, 'lives=$18D:',ram[0x18D], 'money=$18E:',ram[0x18E])
PY
# ③ 文本串与记录（三版 grep）
python3 -c "
import re
for n,p in [('CN','rom/gunnac_cn.nes'),('US','reference/us/gunnac_us.nes'),('JP','reference/jp/gunnac_jp.nes')]:
    d=open(p,'rb').read()
    print(n, {s: hex(d.find(s)) for s in [b'CONTINUE', b'GAME OVER']})"
# ④ 关键代码点（seed 投影）
sed -n '/^bank15:E29A/,/^bank15:E2F3/p' reference/seed/gunnac.asm   # 复活/分叉
sed -n '/^bank03:A33E/,/^bank03:A3ED/p' reference/seed/gunnac.asm   # 死亡行为/结算
grep -n "9D 00 04\|9D 1C 04" reference/seed/gunnac.asm | sed -n '/D10/p'  # 释放
sed -n '/^bank15:E3AB/,/^bank15:E487/p' reference/seed/gunnac.asm   # 菜单
# ⑤ $D86B 全表 + JP 全等
python3 -c "
cn=open('rom/gunnac_cn.nes','rb').read()[0x10:]; jp=open('reference/jp/gunnac_jp.nes','rb').read()[0x10:]
t=cn[0x1D86B:0x1D86B+0x40]; t2=jp[0x1D5A5:0x1D5A5+0x40]
print(t.hex()); print('CN==JP:', t==t2)"
# ⑥ US 基底全等
python3 -c "
us=open('reference/us/gunnac_us.nes','rb').read(); cn=open('rom/gunnac_cn.nes','rb').read()
for lo,hi in [(0x1E290,0x1E520),(0x633E,0x63B0)]:
    print(hex(lo), us[0x10+lo:0x10+hi]==cn[0x10+lo:0x10+hi])"
```

### 6.2 验收清单（对照任务书①-⑤）

| # | 要求 | 结论 | 状态 |
|---|---|---|---|
| ① | 死亡语义链（触发/0x0F 行为/复活转移）A/B 级证据指针 | §1.1-1.4：触发四写签名（A）、`$D86B` 全表（A）、0x0F 自向量窗口链（A）、释放/复活（A）；判定盒上游 W15（B） | ✅ |
| ② | GAME OVER 触发链定谳（或显式负结果+静态链） | §2：`$018D` 命数（A 双证）、`$26\|=8` 触发（A）、分支链/文本/菜单全指针（A 静态）+ **负结果：trace 未达 GAME OVER 屏**（§2.6） | ✅ |
| ③ | Continue 机制（或负结果） | §3：CONTINUE/GAME OVER 双出口 + RAM 复位范围表（A 写点）+ 代价三级表；菜单动态未实测（随 ② 负结果） | ✅ |
| ④ | flow 边集扩展建议表 | §4.2：F1-F13 共 13 条边（guard/证据/trace 锚/规则名） | ✅ |
| ⑤ | `python3 gates/cli.py check` 全绿 + 自查留痕 | §6.3 | ✅（见下） |

### 6.3 自查留痕（门禁 + 纪律自查）

- 门禁：`python3 gates/cli.py check` **PASS=12 FAIL=0 SKIP=0 TOLERATED=0**（2026-10-14 实跑，全绿）：

```
[PASS] rom-integrity / seed-integrity / assets-manifest / naming-ledger / knowledge-format /
       docs-links / rust-tests(108) / subroutine-diff(L2) / trace-diff(L3) / visual-diff(L4) /
       completeness-ratchet(19) / addr-bridge(v2 全量复验)
[OK  ] ratchet 全项基线不降；baseline-integrity sha 链一致
SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0
CHECK: PASS（含 0 个静默跳过：所有 SKIP 均已显式打印）
```
- 纪律自查：
  1. 证据分级：本文 A 级均含可复现指针（bank:addr / PRG 偏移 / trace 帧号 / ram.hex 帧号）；B/C 级已标注；
  2. 未把反汇编器伪代码当事实：`$D86B` 区被 IDA 线性扫成指令，本文按**字节 dump** 呈现（`xxd` 可验）；
  3. 语义口径遵循 A5/AR57 更正：真死亡=`$0400` 1→0x0F，未回退 `$0401` 旧模型；
  4. 坐标桥未手工换算：桥 unmapped 处明确声明并以字节模式验证补证（不伪造坐标映射）；
  5. 假设区（§3.4/§2.5/§1.3 `$0710`）与结论分离。

### 6.4 遗留假设/残项

| # | 项 | 级 | 归属 |
|---|---|---|---|
| R-G1 | `$017A-$017F` 语义（游戏内仅 3 触点；减半变换=续关代价的解释待事件对齐） | C | 发货槽/分数域考证 |
| R-G2 | `$26 bit7 / $32 bit7 / $06A0` 模式位语义（免死/直回标题分支） | C | game-action-mode 域 |
| R-G3 | `$0710` 字段语义（复活写 1，`A7D6` 族按 $E8 钳制） | C | 对象网格字段图 |
| R-G4 | GAME OVER 菜单动态（§2.6 场景设计待采：`$26\|=8`→Start→菜单态） | — | oracle 场景组 |
| R-G5 | 死亡窗长 128f 的计时源精确归属（`$CD9D` 相位 vs `$05DC` 索引饱和） | B | H-16 续问 |
| R-G6 | 玩家被击的最上游判定调用点（W15） | B | H-16 |
