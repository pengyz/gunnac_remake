# spec: 弹丸系统（shot-system）→ docs/spec/shot-system.md

> 状态：2026-10-12 v1（任务 t36，spec-writer）。证据基座：`docs/findings/shot-behavior.md`（t29）+
> seed 逐字节复核（`reference/seed/gunnac.asm` bank02/bank03）+ `data/naming/ledger.jsonl` N-0035/N-0038..0048。
> 纪律：只收 A/B 级可观察行为；残项（A1-A5、$925F 分支、表界）如实留空（§5），**禁止发明**。
> 可判定性：生成调度/参数装载=**L2 纯函数可判**；运动积分=**L2 序列可判**（给定初值逐帧积分 vs Rust）。
> 矩阵格：本文件规则挂 **B-SYS-FIRE**（主武器射击生成；A 边沿投放仍见 [player-movement.md](player-movement.md) R-fire-*）。

## 0. 范围与关联格

| 模块 | 矩阵格 | 覆盖入口 | 强度 |
|---|---|---|---|
| 弹丸生成/运动/等级 | B-SYS-FIRE | 六型自动连射循环 / 生成初值链 / 脚本机+16 向 LUT / 等级配置流 | **L2**（R-shot-gen/motion/level-*） |

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
- 可判定性: **机器可判定（L2 纯函数）**：给定计数器/周期快照单步比对生成决策
- 关联: B-SYS-FIRE、R-shot-level-2、N-0035

### R-shot-gen-2: 自动连射无按键判断（H-14 定谳延续）：循环体内仅计数器/周期比较——主武器射击**不读输入**
- 证据: `bank3:$A1FD-$A22B` 循环体内无 `$16`/`$18` 引用（静态）+ `scenario:analysis/traces/akey-role`（hold-A 100 帧内 `$072E,X` 无 A 相关脉冲，动态反证）+ t25/t29 输出
- 可判定性: 机器可判定（L3：输入位变化时生成节奏不变）
- 关联: B-SYS-FIRE、H-14（已收口：主武器=计数器驱动自动连射 / A 边沿=投放 / B=弹种切换）

### R-shot-gen-3: 生成初值链（弹对象五字段）：寿命/状态 `$040E←1`、脚本指针 `$047E←$0766,X`、坐标 `$04EE←$04E0`（玩家X）/`$055E←$04FC`（玩家Y）、弹型码 `$063E←X`
- 证据: `bank3:$A20F-$A22B`（`$055E,Y←$04FC`、`$04EE,Y←$04E0`、`$063E,Y←X`、`$040E,Y←1`、`$A228 LDA $0766,X/STA $047E,Y`）+ `docs/findings/shot-behavior.md` §1.1
- 可判定性: **机器可判定（L2 纯函数）**：生成事件写集断言
- 关联: B-SYS-FIRE、N-0044（obj_lifetime_counter）、N-0045（obj_script_ptr）、N-0046/0047（obj_pos_x/y）

## 3. 行为规则 —— R-shot-motion-*（运动机 bank2:$87B3-$8810）

### R-shot-motion-1: 脚本机取指循环：寿命计数 `$040E,X` 逐帧 DEC；耗尽时按指针 `$047E,X` 取时长 `$892F,Y`——**非 0=定新时长并取运动码 `$890B,Y` 写 `$05CE,X`、指针++；0=跳转**（`$890B,Y`→新 Y→重取时长）
- 证据: `bank2:$87B3-$87D9` 逐字节解码（`BD 0E 04` LDA/`DE 0E 04` DEC/`BC 7E 04` LDY/`B9 2F 89` LDA $892F,Y/`D0 07` BNE/`B9 0B 89` LDA $890B,Y/`A8` TAY/`4C C0 87` JMP 回取/`9D CE 05` STA $05CE/`C8 98 9D 7E 04` 指针++）+ `docs/findings/shot-behavior.md` §1.3
- 可判定性: **机器可判定（L2 序列）**：给定脚本指针与计数器，逐帧推进 vs Rust 同构
- 关联: B-SYS-FIRE、C-action-stream、C-action-durations、N-0044、N-0045

### R-shot-motion-2: 16 向速度 LUT 累加：按运动码 Y 索引 `$88BB,Y`（X 分量）/`$88CF,Y`（Y 分量），带符号加到 `$04EE,X`/`$055E,X`；运动码最高位=负向标志（`BMI` 分支）
- 证据: `bank2:$87DA-$8810`（`BC CE 05` LDY $05CE/`B9 BB 88` LDA $88BB,Y/`30 07` BMI/`7D EE 04` ADC $04EE,X，Y 侧同构）+ LUT 逐值（C-motion-lut-x/y）+ `docs/findings/shot-behavior.md` §1.3
- 可判定性: **机器可判定（L2 序列）**：运动积分逐帧比对
- 关联: B-SYS-FIRE、C-motion-lut-x、C-motion-lut-y、A4（LUT 索引精确规则/表界待 S3）

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

### R-shot-level-3: weapon_level 语义（H-02 完整答案）：升级=**同时激活的弹型数与发射频率变化**，非换弹型码；`$33` 为 0..255 递增计数，**溢出 255 回卷 12（$0C=回卷目标，非上限）**
- 证据: §4 交叉（级 0=1 对 → 级高最多 6 对）+ N-0033（`INC $33` 溢出回卷置 $0C）+ L2 对拍 pk-4（255→12 回卷）/pk-5（**12→13 递增继续**）+ `docs/findings/shot-behavior.md` §2 结论行
- 可判定性: 机器可判定（L2/L3：升级事件后生成器参数集变化断言）
- 关联: B-SYS-FIRE、B-WPN-xx-LVx（25 格仍 L0：逐槽逐级的具体配置组合未逐格验证）、H-02（收口）、R-item-2
- 【2026-10-12 更正（t37）】原"级上限 `$0C`=12"**证伪**：pk-5 实测 12→13，`$0C` 是溢出回卷目标而非上限（全值域 0..255，回卷 FF→0C）；级↔配置映射由 `$9457` 12 项 + `3+((级-4)&7)` 变换吸收高值（A1 残项维持）

### R-shot-level-4: 待证分支登记（不入结论）：slot≠0 时同型拾取深路径 `$925F`（t31 线索，t35 在考）
- 证据: t31 输出线索（`bank2:$925F` 同型分支）—— **未复核，仅登记**
- 可判定性: 待 t35 取证后定
- 关联: N-0032（$34 拾取处理）、H-02 残项

### R-shot-gen-4: 射击模式分派（`$34` weapon_slot 驱动）：0=六型自动连射（本节 R-shot-gen-1）；1=type3 齐射（填 `$0402` 槽）；2=type4 齐射；3=退出不发弹；>3=type6 齐射
- 证据: `bank3:$A1F5-A2B2`（`LDA $34/BNE` 分派、`$A244 CMP #$03/BEQ`、`$A271-$A291` LDX #$03 族）+ `docs/findings/shot-behavior.md` §2
- 可判定性: 机器可判定（L2 纯函数：槽值→分派分支断言）；逐型齐射参数未证（留 §5）
- 关联: B-SYS-FIRE、N-0032（weapon_slot=$34）

## 5. 待证残项（如实留空，禁止发明）

| # | 残项 | 级 | 去向 |
|---|---|---|---|
| A1 | `$33`≥4 索引变换使配置 $71 段可达性存疑（级↔配置非单调？） | C | S2 升级全序列 trace |
| A2 | f2220 拾取是否同帧触发两处理器（编号圈+字母圈） | C | S2 分化拾取场景 |
| A3 | 弹数上限 `$20`=32（N-0048 路径）与 shop 信息页「导弹20 上限」口径差 | C | S2 商店购买 trace |
| A4 | `$88BB/$88CF` 16 向 LUT 精确索引规则/表长/重复模式；`$890B`/`$892F` 表界（观测区 $890B-$892E / $892F 起 FF 段，界外未证） | B | S3 表切分 |
| A5 | `$3B` 连收计数 → `$33` 升级的触发链 | C | S3 调用链 |
| — | slot≠0 同型 `$925F` 分支（R-shot-level-4） | C | t35 在考 |
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
