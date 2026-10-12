# spec: 弹丸系统（shot-system）→ docs/spec/shot-system.md

> 状态：2026-10-12 v1（任务 t36，spec-writer）。证据基座：`docs/findings/shot-behavior.md`（t29）+
> seed 逐字节复核（`reference/seed/gunnac.asm` bank02/bank03）+ `data/naming/ledger.jsonl` N-0035/N-0038..0048。
> 纪律：只收 A/B 级可观察行为；残项（A1-A5、$925F 分支、表界）如实留空（§5），**禁止发明**。
> 可判定性：生成调度/参数装载=**L2 纯函数可判**；运动积分=**L2 序列可判**（给定初值逐帧积分 vs Rust）。
> 矩阵格：本文件规则挂 **B-SYS-FIRE**（主武器射击生成；A 边沿投放仍见 [player-movement.md](player-movement.md) R-fire-*）。

## 0. 范围与关联格

| 模块 | 矩阵格 | 覆盖入口 | 强度 |
|---|---|---|---|
| 弹丸生成/运动/等级 | B-SYS-FIRE | 六型连发循环（**B 键按住门控**，t52 更正）/ 生成初值链 / 脚本机+16 向 LUT / 等级配置流 | **L2**（R-shot-gen/motion/level-*） |

不在范围：投放（R-fire-*）、拾取处理（N-0048 `$3B`/N-0030 `$0190`，见 shop-economy R-bomb-*）、
敌弹（enemy_script_stream N-0052 属 enemy 线）、武器槽齐射 type3/4/6 的逐型细节（§2 分派已证，逐型参数未证）。

## 1. 常量对照（同步 [constants.md](constants.md) v4 六表逐值）

| 常量 ID | 表 | 证据 | 级 | 关联 |
|---|---|---|---|---|
| C-lvl-offsets | `$9457` 级→配置偏移表（12 项） | `bank2:$9457` + seed 逐字节 | A | R-shot-level-1 |
| C-shot-cfg-stream | `$9463` 配置流（(周期,脚本参) 对） | `bank2:$9463` + seed 逐字节 | A | R-shot-level-2 |
| C-motion-lut-x | `$88BB` X 速度分量 LUT（16 项） | `bank2:$88BB` + seed 逐字节 | A | R-shot-motion-2 |
| C-motion-lut-y | `$88CF` Y 速度分量 LUT（16 项） | `bank2:$88CF` + seed 逐字节 | A | R-shot-motion-2 |
| C-action-stream | `$890B` 动作流（运动指令码序列） | `bank2:$890B`（代码级：`$87CF LDA $890B,Y`） | A | R-shot-motion-1 |
| C-action-durations | `$892F` 动作时长表（0=跳转） | `bank2:$892F`（代码级：`$87C0 LDA $892F,Y / BNE`） | A | R-shot-motion-1 |

## 2. 行为规则 —— R-shot-gen-*（生成侧）

### R-shot-gen-1: 六弹型并行计数调度：`$072E,X`（X=0..5）计数与 `$074A,Y` 周期比较，到点生成弹对象并 INC 回写
- 证据: `bank3:$A1FD-$A22B`（`LDA $072E,X`、`CMP $074A,Y`、`INC $072E,X` @`$A20C`）+ `docs/findings/shot-behavior.md` §1.1 + N-0035
- 可判定性: **机器可判定（L2 纯函数）**：给定计数器/周期快照单步比对生成决策；**L3 已判**【t57 升级登记（t55 转正落地）】
- 关联: B-SYS-FIRE、R-shot-level-2、N-0035（定性已由 N-0073 更正，计数调度语义维持）
- 【2026-10-12 t56 更正（t52 F1）】调用链上游受 **B 键按住门控**（见 R-shot-gen-2）——本条调度机制（计数 vs 周期）不变，"自动"表述废止

### R-shot-gen-2: 主武器 = **B 键按住连发**（非自动连射）：门控链 `$18` bit6 → `$39/$3A` 自动重复计时器 → `$32` bit2 六型生成器门
- 证据: `docs/findings/shot-gating-f1.md` §1（`bank3:$A1A9 LDA $18/AND #$40` B 按住→连发路径；`$A1B6 DEC $3A/BNE` 倒数、`$A1BA LDA $39/STA $3A` 重装（$39=8，t13 `LDA #$08/STA $39/$3A`）；`$A1BE LDA $32/AND #$04` 门）+ trace 实测（f1501 `press 1500 b` → `$18=$42/$16=$40`：bit6=B，N-0072）+ fire-hold 场景重采（计数递增 + 每 7 帧一发 + 槽填充可观测）+ `bank3:$A20C/$91B3` 计数 INC/复位例程（Ws-9 复位闭环待证）
- 可判定性: 机器可判定（L3：B 位→生成节奏断言）；**L3 已判**【t57 升级登记（t55 转正落地，706 帧轨迹重放全等）】
- 关联: B-SYS-FIRE、H-14（定谳文本以本条为准）、N-0072/N-0073、Ws-8（$32 bit2/bit3 置位时机）、Ws-9（计数复位闭环）
- 【2026-10-12 t56 更正（t52 F1 冲击波）】原"自动连射无按键判断（主武器不读输入）"**证伪**：循环体（`$A1FD-$A236`）无按键判断属实，但**调用方 `$A1A9` 存在 B 键按住门控**——t29 N-0035"自动连射"定性**撤回**（N-0073）；gameplay-stage1 无 B 长按故写链 0 命中（t50 F1"零活动"之谜=F1 门控）。原表述留档不删

### R-shot-gen-3: 生成初值链（弹对象五字段）：寿命/状态 `$040E←1`、脚本指针 `$047E←$0766,X`、坐标 `$04EE←$04E0`（玩家X）/`$055E←$04FC`（玩家Y）、弹型码 `$063E←X`
- 证据: `bank3:$A20F-$A22B`（`$055E,Y←$04FC`、`$04EE,Y←$04E0`、`$063E,Y←X`、`$040E,Y←1`、`$A228 LDA $0766,X/STA $047E,Y`）+ `docs/findings/shot-behavior.md` §1.1
- 可判定性: **机器可判定（L2 纯函数）**：生成事件写集断言；**L3 已判**【t57 升级登记：t55 golden 转正落地（commit 5baa6a8，L3 三断言 2559 项全绿），t51"前提 t50"条件达成】
- 【2026-10-12 t57 出生签名入规则（A 级，t55 断言已依赖）】**出生判定签名 = 坐标=玩家±9px**（抽取链窗口资格三判据之一）——检测时刻判据；初始化精确拷贝语义（`$04EE←$04E0`/`$055E←$04FC`）维持不变
- 关联: B-SYS-FIRE、N-0044（obj_lifetime_counter）、N-0045（obj_script_ptr）、N-0046/0047（obj_pos_x/y）

## 3. 行为规则 —— R-shot-motion-*（运动机 bank2:$87B3-$8810）

### R-shot-motion-1: 脚本机取指循环：寿命计数 `$040E,X` 逐帧 DEC；耗尽时按指针 `$047E,X` 取时长 `$892F,Y`——**非 0=定新时长并取运动码 `$890B,Y` 写 `$05CE,X`、指针++；0=跳转**（`$890B,Y`→新 Y→重取时长）
- 证据: `bank2:$87B3-$87D9` 逐字节解码（`BD 0E 04` LDA/`DE 0E 04` DEC/`BC 7E 04` LDY/`B9 2F 89` LDA $892F,Y/`D0 07` BNE/`B9 0B 89` LDA $890B,Y/`A8` TAY/`4C C0 87` JMP 回取/`9D CE 05` STA $05CE/`C8 98 9D 7E 04` 指针++）+ `docs/findings/shot-behavior.md` §1.3
- 可判定性: **机器可判定（L2 序列）**：给定脚本指针与计数器，逐帧推进 vs Rust 同构
- 关联: B-SYS-FIRE、C-action-stream、C-action-durations、N-0044、N-0045

### R-shot-motion-2: 速度 LUT 累加：按运动码 Y **无掩码**索引 `$88BB,Y`（X 分量）/`$88CF,Y`（Y 分量），带符号加到 `$04EE,X`/`$055E,X`；运动码最高位=负向标志（`BMI` 分支）
- 证据: `bank2:$87DA-$8810`（`BC CE 05` LDY $05CE/`B9 BB 88` LDA $88BB,Y/`30 07` BMI/`7D EE 04` ADC $04EE,X，Y 侧同构）+ LUT 逐值（C-motion-lut-x/y）+ `docs/findings/shot-behavior.md` §1.3
- 可判定性: **机器可判定（L2 序列）**：运动积分逐帧比对
- 关联: B-SYS-FIRE、C-motion-lut-x、C-motion-lut-y、A4（表界待 S3；索引规则已定谳=无掩码）
- 【2026-10-12 t57 语义补正（t55 实证）】索引 **Y 不做掩码**：motion≥16 时直接读**表间字节**（如 `$88CB=$F8`——即两 LUT 间夹缝区），原"16 向 LUT"表述隐含掩码语义会**伪造 +9 偏移**；"16 向"仅描述表内循环形态，不是索引规则

### R-shot-motion-3: 弹型差异本质 = 不同初始脚本指针（`$0766` 配置参）+ 不同发射周期（`$074A`），共享同一脚本机与速度 LUT（"配置流 + 脚本机 + LUT"三件套）
- 证据: §2/§3 交叉（配置流逐对喂 `$074A`/`$0766`，运动机唯一）+ `docs/findings/shot-behavior.md` §1.3 结论行
- 可判定性: 机器可判定（L2：换配置参→轨迹族变化断言）
- 关联: B-SYS-FIRE、R-shot-level-2

## 4. 行为规则 —— R-shot-level-*（等级配置；H-02 完整答案）

### R-shot-level-1: 等级→配置选择（`bank2:$9343-$938D` 装载流）：`$33`（weapon_level）<4 直接作索引；**≥4 变换为 `3+((级-4)&7)`**；经 `$9457` 偏移表定位配置段
- 证据: `bank2:$9368-$937C`（`LDA $33/CMP #$04/BCC`、`SBC #$04/AND #$07/ADC #$03`、`TAY/LDX $9457,Y`）+ `$9457` 逐值 [0,3,10,17,26,37,48,61,74,87,100,113]（C-lvl-offsets）+ `docs/findings/shot-behavior.md` §1.2
- 可判定性: **机器可判定（L2 纯函数）**：级值→配置段定位断言
- 关联: B-SYS-FIRE、C-lvl-offsets、A1（级≥4 变换使 $71 段可达性存疑）

### R-shot-level-2: 配置流逐对填充：`$9463` 段内 `(周期, 脚本参)` 对顺序写 `$074A,Y`（周期）与 `$0766,Y`（脚本参），**`$FF` 负值终止**；级 0=1 对（`05 0C`）→ 高级最多 6 对（级 6-10），级 11=5 对
- 证据: `bank2:$937C-$938D`（`LDA $9463,X / BMI 结束`、双 `STA $074A,Y`/`STA $0766,Y`）+ 配置流逐值（C-shot-cfg-stream，seed 逐字节核）+ `docs/findings/shot-behavior.md` §1.2
- 可判定性: **机器可判定（L2 纯函数）**：级→(周期,脚本参)序列断言
- 关联: B-SYS-FIRE、C-shot-cfg-stream、R-shot-gen-1

### R-shot-level-3: weapon_level 语义（H-02 完整答案）：升级=**同时激活的弹型数与发射频率变化**，非换弹型码；`$33` 为 0..255 递增计数，**溢出回卷按路径分野：kernel（slot==0）FF→$0C，deep（slot≠0）裸 INC FF→$00**
- 证据: §4 交叉（级 0=1 对 → 级高最多 6 对）+ kernel：N-0033（`INC $33` 溢出回卷置 $0C）/ pk-4（255→12）/pk-5（**12→13 递增继续**）；deep：t35 定谳（`$92B4` 裸 INC 溢出自然回卷 $00，`docs/findings/upgrade-branch-h02.md`、N-0058 范围更正）+ pk-8/pk-9 实测，**deep wrap 用例 pk-10 待 t39 补入** + `docs/findings/shot-behavior.md` §2 结论行
- 可判定性: 机器可判定（L2/L3：升级事件后生成器参数集变化断言）；deep wrap 断言待 pk-10（t39）
- 关联: B-SYS-FIRE、B-WPN-xx-LVx（25 格仍 L0：逐槽逐级的具体配置组合未逐格验证）、H-02（收口）、H-A（deep 越 12 → $9457 越界索引）、R-item-2
- 【2026-10-12 更正（t37）】原"级上限 `$0C`=12"**证伪**：pk-5 实测 12→13，`$0C` 是溢出回卷目标而非上限（全值域 0..255，回卷 FF→0C）；级↔配置映射由 `$9457` 12 项 + `3+((级-4)&7)` 变换吸收高值（A1 残项维持）
- 【2026-10-12 t43 scope 标注】上注"回卷 FF→0C"限 **kernel 路径**；deep 路径裸 INC 自然回卷 **FF→$00**（t35），两路径分野以 `$34==0` 为界
- 【2026-10-12 t44·追加范围（t39 同步）】索引变换公式**勘误**：`3+((级-4)&7)` → **`4+((级-4)&7)`**（t39 H-A 定谳：`AND #$07` 不清 C、`SBC` 无借位 ⇒ `ADC #$03` 进位恒 1，实算 +4；ha-2 ROM 实证 级 12→索引 4/X=26）；**值域恒 [0,11]**（级<4 直接 [0,3]，级≥4 变换 [4,11]，深路径 level≤255 由 mod 8 吸收）——**无越界，A1 残项闭合**

### R-shot-level-4: 深路径分支（slot≠0 同型拾取）三态行为——【2026-10-12 t44 转正，t35+t39 双定谳】
- 证据: `docs/findings/upgrade-branch-h02.md`（t35 分支全景：seed+Ghidra 逐分支一致）+ t39 实现与 L2 实测 **pk-8/pk-9/pk-10/pk-11/pk-12 + ha-1..4**（63/63 三方全等）；三态：①裸升（`$92B4` 裸 `INC $33`，溢出 FF→$00）②连收分支（`$018C==$FF`：不加级、`$018A+1`、满 30 奖励事件钩子 JSR $DBA9 只接不实现）③`$33≥8&&$34==0` 回 kernel（深路径固有 `$34≠0` 故不可达）
- 可判定性: **机器可判定（L2 已实证）**；`$018A` 计数/奖励链数值实现属移植面（t39 挂账）
- 关联: N-0032、R-item-2（kernel/deep 溢出二分）、H-A（已定谳：公式 4+、无越界）

### R-shot-gen-4: 射击模式分派（`$34` weapon_slot 驱动）：0=六型连发（本节 R-shot-gen-1；**B 键门控**，t56 更正措辞——原"自动连射"作废）；1=type3 齐射（填 `$0402` 槽）；2=type4 齐射；3=退出不发弹；>3=type6 齐射
- 证据: `bank3:$A1F5-A2B2`（`LDA $34/BNE` 分派、`$A244 CMP #$03/BEQ`、`$A271-$A291` LDX #$03 族）+ `docs/findings/shot-behavior.md` §2
- 可判定性: 机器可判定（L2 纯函数：槽值→分派分支断言）；逐型齐射参数未证（留 §5）
- 关联: B-SYS-FIRE、N-0032（weapon_slot=$34）

## 4.5 行为规则 —— R-shot-alloc-*（弹对象槽位分配，2026-10-12 t51 登记；t48 定谳）

> 证据：`docs/findings/shot-slot-geometry.md`（A 级：字节 + L2 实测）+ L2 用例 **shot-alloc-1..4 三方全等**。
> 可判定性：**L3 已判**【2026-10-12 t57 升级登记：t55 golden 转正落地（5baa6a8）——R-shot-alloc 三断言（分配合法/寿命递减/轨迹重放）L3-DIFF 2559 项全绿；t51 FA-2 式"前提 t50"条件达成，历史标注留档】。

### R-shot-alloc-1: 槽位几何：**4 组 × 5 = 20 槽**（`$80-$84`/`$9C-$A0`/`$B8-$BC`/`$D4-$D8`），组内 stride=1，**组距 $1C**（与对象网格同格交错）
- 证据: `docs/findings/shot-slot-geometry.md` §1（字节实证）+ §2 shot-alloc-1..4 三方全等 + churn 对照【2026-10-12 t56 口径更新（t52 F2b/N-0074）：t53 撤除的 churn 经 fire-hold 重采**正确口径**=真实弹数分布并发 0-9（众数 2-6、峰值 9、**容量 20 未饱和**）；t50"静态非寿命"反证解=无弹伪影+$1C 网格交错污染，槽 state=逐帧寿命倒数（$FF 起 −1）维持 t48 语义】
- 可判定性: 机器可判定（L2 数据解析/分配断言）；**L3 已判**（t57 升级，t55 转正落地）
- 关联: B-SYS-FIRE、N-0054（obj_field_grid_1c 同格）
- 【t51 撤销】t45"槽位几何未证"注记**已定谳撤销**：t45 的 18 例矛盾值= stride-1 跨 $18 组间空隙**伪读**（`oracle/traces/README.md` 原地更正）

### R-shot-alloc-2: 分配器 `bank2:$888D-$88BA` = **首次适应**（低偏移优先）扫描 20 槽
- 【2026-10-12 t57 结案注】t56 路由信号"f2350 槽位分配语义违例"**已结案=移植/实现差**（t55）：首分歧点=断言腿的出生检测把 **W7 网格驻留切换**误计为出生（预期集算法无误）；处置=出生判定补 R-shot-gen-3 签名（坐标=玩家±9px）→ 负例归零、L3 全绿；**非规格缺口，本规则无需改**（原"待联查"路由撤销）
- 证据: `docs/findings/shot-slot-geometry.md` §1 逐指令反汇编（`LDY #$80` 起始偏移）+ shot-alloc-2 用例
- 可判定性: 机器可判定（L2 纯函数）；**L3 已判**（t57 升级）
- 关联: B-SYS-FIRE

### R-shot-alloc-3: 空槽判定 = `state==0`；回收 = 寿命倒数归零
- 证据: `docs/findings/shot-slot-geometry.md` §1/§2（shot-alloc-3 用例）+ R-shot-motion-1（`$040E` 逐帧 DEC）互证
- 可判定性: 机器可判定（L2 纯函数）；**L3 已判**（t57 升级）
- 关联: B-SYS-FIRE、R-shot-motion-1、N-0044

### R-shot-alloc-4: 满载路径 = **SEC**（Y=A=`$F0`，`TYA/ADC #$18` 遗留值；case-4 ROM 实测更正）
- 证据: `docs/findings/shot-slot-geometry.md` §1（`$88B9: SEC/RTS`）+ §2 shot-alloc-4-full 用例（Y=$F0/A=$F0 三方全等；A 值系遗留，首轮手算已 ROM 实测更正）
- 可判定性: 机器可判定（L2 纯函数：满载出参断言）；**L3 已判**（t57 升级）
- 关联: B-SYS-FIRE

## 5. 待证残项（如实留空，禁止发明）

| # | 残项 | 级 | 去向 |
|---|---|---|---|
| ~~A1~~ | **【2026-10-12 t44 闭合（t39 H-A 定谳）】**：变换实为 `4+((级-4)&7)`，值域恒 [0,11] 无越界——"$71 段可达性存疑"随公式勘误消解 | ~~C~~ | 已闭合 |
| A2 | f2220 拾取是否同帧触发两处理器（编号圈+字母圈） | C | S2 分化拾取场景 |
| A3 | 弹数上限 `$20`=32（N-0048 路径）与 shop 信息页「导弹20 上限」口径差 | C | S2 商店购买 trace |
| A4 | `$88BB/$88CF` 16 向 LUT 精确索引规则/表长/重复模式；`$890B`/`$892F` 表界（观测区 $890B-$892E / $892F 起 FF 段，界外未证） | B | S3 表切分 |
| A5 | `$3B` 连收计数 → `$33` 升级的触发链 | C | S3 调用链 |
| ~~—~~ | slot≠0 同型 `$925F` 分支 → **R-shot-level-4 已转正**（t35+t39 双定谳）；残余=$925F 侧调用 $9177/$91B3 副作用（$32 bit2 守卫）属事件面，待 S 系列 | — | S 系列考证 |
| — | type3/4/6 齐射逐型参数（填槽规则/弹型码/初值） | C | S2/S3 |
| — | 弹对象碰撞/命中（与 H-16 hit-test 同域未证） | C | hit-test 取证 |

> **样本列更正（2026-10-12）**：t29 findings §1.3 表中"$890B 动作流"的样本字节 `41 C1 C1…` 实取自
> `$88F7` 区（相邻数据），**指针标签经代码级复核正确**（`$87CF LDA $890B,Y`），仅样本列错位；
> `$88CF` Y-LUT 样本列同理多含 `$88CB-$88CE` 四字节前导（`F8 F8 08 08`）。逐值以 constants.md v4（seed 逐字节）为准。

## 6. 门禁

`python3 gates/cli.py all` → 见任务 t36 输出。

## spec 产出契约

- 规则清单: R-shot-gen-1..4 + R-shot-motion-1..3 + R-shot-level-1..4（共 11 条，逐条带 bank:addr + 证据指针）
- 常量对照: [constants.md](constants.md) v4 已同步（六表逐值：C-lvl-offsets/C-shot-cfg-stream/C-motion-lut-x/C-motion-lut-y/C-action-stream/C-action-durations）
- 可判定性标注: 机器可判定 11 条（L2 纯函数 5、L2 序列 2、L2/L3 混合 4；R-shot-level-4 为登记条目待证）
- 门禁: `python3 gates/cli.py all`（见输出）
- 遗留假设: A1-A5 + $925F 分支 + 齐射逐型参数 + 碰撞命中（C/B 级，未入规则结论）
