# docs/spec/constants.md — 常量对照表（逐值带来源）

> 状态：2026-10-12 v1（任务 t6 随 shop-economy 同步建立）。
> 规则：只收 **A/B 级证据**的数值/模板；来源必须可复现（`rom:<偏移>` / trace / 截图）；
> **含义未证的数值单列 §2「登记不解释」**，禁止望文生义。行随模块规格增长，只增不删。

## 1. 已解释常量

| 常量 ID | 值 | 含义 | 来源 | 级 | 关联 |
|---|---|---|---|---|---|
| C-shop-p1 | `$10` | 商店 1 号记录价格行数值 | `rom:8929` | A | R-shop-1 |
| C-shop-p2 | `$6` | 商店 2 号记录价格行数值 | `rom:8958` | A | R-shop-1 |
| C-shop-p3 | `$4` | 商店 3 号记录价格行数值 | `rom:8986` | A | R-shop-1 |
| C-money-template | `-------$` | 商店金钱显示行模板（破折号填充 + 货币符） | `rom:8516–8523`（`$`@8523） | A | R-shop-2、R-money-1 |
| C-vel-h | 速度对 (208,2) = **+2.8125 px/帧** | 自机正向速度（right/down） | `bank03:A0BE-A0CB`/`A08D-A098` + probe RAM diff（`ram-watch-s3.md` §1） | A | R-move-6 |
| C-vel-h-neg | 速度对 (48,253) = **−2.8125 px/帧** | 自机反向速度（left/up）；无输入段 (0,0) | 同上 | A | R-move-6 |
| C-clamp-x | `$0E`–`$F3`（14–243） | 玩家 X 钳制范围（触钳亚像素归零） | `bank03:A0D0-A0E3` + probe diff | A | R-move-3 |
| C-clamp-y | `$14`–`$DF`（20–223） | 玩家 Y 钳制范围（触钳亚像素归零） | `bank03:A09D-A0B0` + probe diff | A | R-move-4 |
| C-bomb-consume | `$A336,Y` = **[1, 2, 3, 2]**（Y=0..3） | 炸弹消耗量表逐值（投放按 $3B 索引；不足回退 Y=1） | `bank03:A336-A339` seed 字节 `01 02 03 02` | A | R-fire-3 |
| C-money-bcd | `money_counter_bcd` = `$E0-$E6` 7 位 BCD（$E0=最高位；值=显示值） | `bank14:D009-D046` 渲染 + probe f2160=4400/f2220=5700/f2280=6800/f2340=7200 | A | R-money-4 |
| C-money-addend-tbl | 加数表 `$DCCF+type×6` 逐值：type 1..10 = **+10/+30/+100/+150/+250/+400/+600/+800/+1000/+2000**（行 6 字节 BCD，倒序配对 $E5..$E0）；type 0 = no-op | `bank14:$DCD0-$DD0F` seed 字节逐行 | A | R-money-5 |
| C-money-cap | 溢出封顶=**六工作位全 9**（ROM 态 `$E0-$E5` 全 9、`$E6` 恒 0）——显示值 **9999990**（七位含尾 0；"999999"系六工作位简称，【t80 精化】） | `bank14:DBF1-DBFD` | A | R-money-5 |
| C-extend-threshold | `extend_threshold_bcd` = `$E7-$EC` = **008000（8000）**，达标置 `$ED=$40` | `bank14:DAE7-DAFC` + probe 恒 0,0,8,0,0,0 | A | R-money-6 |
| C-bomb-type-index | `bomb_type_index` = `$0190` 值域 **0-3**（四弹种）；映射 0=F/3=W 实测、1=B/2=T 推定 | `bank15:E138-E13D` + probe f2220 拾「W」0→3 | A（值域）/B（映射 1=B、2=T） | R-bomb-1、R-bomb-3 |

## 1.5 关卡结构锚点（2026-10-12 t46，结构级；语义未证项见 stage-timeline §4）

| 常量 ID | 值 | 来源 | 级 | 关联 |
|---|---|---|---|---|
| C-wave-chain | 波次链区 `bank8:$AD00-$ADD0`（file 0x10D10-0x10DE0）：`00 FE` 分隔分段事件记录流 | `docs/findings/stage-timeline.md` §2.1 实录 + xxd 复现 | B | R-stage-1 |
| C-wave-dir | 目录 `bank8:$ACEF` = 块范围对（`44 AD D0 AD` → $AD44-$ADD0） | stage-timeline §2.3 | B | R-stage-3 |
| C-scroll-vars | 滚动寄存器变量 `$1C/$1D`（NMI `bank15:$EFC8/$EFCD STA $2005` 消费） | stage-timeline §4 + S0 | B | R-stage-4 |

## 1.6 敌对象结构锚点（2026-10-12 t60，结构级；参数语义留 B 系假设）

| 常量 ID | 值 | 来源 | 级 | 关联 |
|---|---|---|---|---|
| C-spawn-rec | t_005 生成记录 5 字节 `{参数×3, 行为脚本指针(2,LE,bank8 窗内)}`（bank8:$ADD0-$BFFF，4656B） | `docs/findings/enemy-dataface.md` §1.1 实录 | A/B | R-enemy-1 |
| C-grid-1c | 关卡对象网格：**28 槽**（**槽 n 地址 = 字段基址+n（stride 1）；字段组距 $1C**——t67/N-0086 寻址更正，原 n×$1C 槽寻址误读；字段族：$0400 类型/状态、$0470/$048C 亚像素、$04E0/$04FC 坐标、$0550/$056C/$0588/$05A4 速度、$05C0+ 倒计时簇） | enemy-dataface §2.1 + N-0011..0016/0086 | A | R-enemy-3/4 |
| C-grid-70 | 弹对象网格：**$70 间距**六字段（$040E/$047E/$04EE/$055E/$05CE/$063E） | shot-behavior §1.3 + t48/t52 | A | R-enemy-3 |

## 2. 登记不解释（存在性已证，含义未证）

| 常量 ID | 值 | 来源 | 级 | 关联 | 备注 |
|---|---|---|---|---|---|
| C-money-unk | `10000000000` | `rom:4759` | A | R-money-3、H-03 | **禁止解释**：上限/显示常量/渲染数值均属猜测，待 S2/S3 取证 |
| C-bomb-type-tbl | `$A33A,Y` = **[2, 3, 4, 4]**（Y=0..3） | `bank03:A33A-A33D` seed 字节 `02 03 04 04` | A | R-fire-3、H-11 | **禁止解释**：类型值 ↔ 炸弹字母（F/B/T/W）映射未证（H-11） |
| C-vel-tables | `$A2FB`/`$A30B` 速度查表（`bank03:A073/A079` 索引） | `bank03:A2FB-A31F` seed 字节 | A（存在）/B（分划） | R-move-5、W6 | **禁止解释**：亚像素/整数分划与索引语义未证（W6） |

## 2.5 弹丸六表（2026-10-12 t36，逐值带 bank 指针；seed 逐字节核）

| 常量 ID | 值（逐值） | 来源 | 级 | 关联 |
|---|---|---|---|---|
| C-lvl-offsets | `$9457` = **[0,3,10,17,26,37,48,61,74,87,100,113]**（hex `00 03 0A 11 1A 25 30 3D 4A 57 64 71`，12 级→配置流内偏移） | `bank2:$9457`（file 0x5467） | A | R-shot-level-1 |
| C-shot-cfg-stream | `$9463` 配置流逐级 `(周期,脚本参)` 对（`$FF` 终止）：L0=(05,0C)｜L1=(03,0C)(02,0B)(02,0D)｜L2=(03,0C)(03,0A)(03,0E)｜L3=(03,0C)(03,09)(03,0F)(02,04)｜L4=(03,0C)(03,20)(03,22)(03,05)(03,03)｜L5=(03,0C)(03,08)(03,00)(03,14)(03,16)｜L6=(02,18)(02,1A)(03,10)(03,12)(03,07)(03,01)｜L7=(03,0C)(02,10)(02,12)(03,20)(03,22)(02,04)｜L8=(03,0C)(02,1C)(02,1E)(03,10)(03,12)(03,04)｜L9=(03,0C)(02,0B)(02,0D)(03,04)(03,06)(03,02)｜L10=(03,0C)(03,10)(03,12)(03,08)(03,00)(02,04)｜L11=(03,0C)(03,10)(03,12)(03,14)(03,16) | `bank2:$9463`（file 0x5473）+ `$9457` 分段 | A | R-shot-level-2 |
| C-motion-lut-x | `$88BB` = **09 08 05 03 00 FD FB F8 F7 F8 FB FD 00 03 05 08**（有符号 9,8,5,3,0,−3,−5,−8,−9,−8,−5,−3,0,3,5,8）；**索引无掩码**（Y 直索，motion≥16 读表间字节如 `$88CB=$F8`——t55 实证，勿按 mod 16 解读） | `bank2:$88BB`（file 0x48CB） | A | R-shot-motion-2 |
| C-motion-lut-y | `$88CF` = **00 03 06 08 09 08 06 03 00 FD FA F8 F7 F8 FA FD**（0,3,6,8,9,8,6,3,0,−3,−6,−8,−9,−8,−6,−3）；**索引无掩码**（同 X 侧；表界/夹缝区字节语义 = A4 素材） | `bank2:$88CF`（file 0x48DF） | A | R-shot-motion-2 |
| C-action-stream | `$890B` 观测区 `$890B-$892E`（36B）= `00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F 0A 0C 0E 0C 06 0C 02 0C 10 0C 12 0C 0B 0C 0D 0C 08 0C 00 0C`；**表界未证（A4）** | `bank2:$890B`（file 0x491B；代码级 `$87CF LDA $890B,Y`） | A | R-shot-motion-1 |
| C-action-durations | `$892F` 起观测 `FF`×16+；语义由代码证实：**0=跳转**（`$87C0 LDA $892F,Y/BNE`），非 0=新时长；**表界未证（A4）** | `bank2:$892F`（file 0x493F） | A | R-shot-motion-1 |

## 3. 版本记录

- `v1 (2026-10-12) 建立：C-shop-p1..p3、C-money-template、C-money-unk（后者不解释）`
- `v2 (2026-10-12, t23) 移动/射击常量：C-vel-h、C-vel-h-neg、C-clamp-x、C-clamp-y、C-bomb-consume（逐值）；C-bomb-type-tbl、C-vel-tables 入登记不解释`
- `v3 (2026-10-12, t26) 金钱/炸弹常量：C-money-bcd（$E0-$E6 七位 BCD）、C-money-addend-tbl（type 1..10 逐值 10/30/100/150/250/400/600/800/1000/2000；行 11+ 表界未证不入表）、C-money-cap（999999）、C-extend-threshold（8000→$ED=$40）、C-bomb-type-index（$0190 0-3，映射 0=F/3=W 实测 1=B/2=T 推定）`
- `v4 (2026-10-12, t36) 弹丸六表逐值（§2.5）：C-lvl-offsets（12 项）、C-shot-cfg-stream（12 级逐对）、C-motion-lut-x/y（16 项）、C-action-stream（观测区 36B，界未证）、C-action-durations（FF 段+0=跳转语义）——全部 seed 逐字节核；t29 样本列错位已在 shot-system.md §5 更正（指针标签经代码级复核正确）`
- `v5 (2026-10-12, t46) 关卡结构锚点（§1.5）：C-wave-chain（波次链区 $AD00-$ADD0 分段记录流）、C-wave-dir（$ACEF 块范围对）、C-scroll-vars（$1C/$1D 滚动寄存器）——结构级 B 级，语义留 S1/S2`
- `v6 (2026-10-12, t57) LUT 表注补正：索引无掩码（motion≥16 读表间字节 $88CB=$F8，t55 实证——"16 向"仅为表内形态非索引规则，掩码解读会伪造 +9 偏移）；表界/夹缝区=A4 素材`
- `v7 (2026-10-12, t60) 敌对象结构锚点（§1.6）：C-spawn-rec（t_005 5 字节记录）、C-grid-1c（$1C 28 槽字段阵列）、C-grid-70（$70 弹对象）——参数语义留 B1-B5`
- `v8 (2026-10-12, t80) C-money-cap 措辞精化：六工作位计（ROM 态 $E0-$E5 全 9、$E6 恒 0，显示值 9999990）——"999999"=六工作位简称；R-money-5 加法器补 borrow 链进位 quirk/type0 无操作/Y=type*6-6 索引更正（t77 ROM 逐字节）`
