# spec: 自机移动/射击（player-movement）→ docs/spec/player-movement.md

> 状态：2026-10-12 v1（任务 t23，spec-writer，S4 行为规则第一批）。
> 证据基座：`docs/findings/ram-watch-s3.md`（t18，代码+RAM diff 双向 A 级）、`data/naming/ledger.jsonl` N-0011..N-0027、
> `reference/seed/gunnac.asm`（bank03:A084-A0E8 移动例程 / bank03:A2B4-A2FA 投放例程）、
> `scenario:analysis/traces/gameplay-stage1`（6 段 hold + 60 帧 probe RAM）。
> 纪律：只收 A/B 级证据支撑的可观察行为；未证语义（红圈/等级/分工）留空位（§4），**禁止发明**。
> 分子强度：B-SYS-MOVE/B-SYS-FIRE 登记 **L2**（绑 scenario/trace；升 L3 待 Rust 侧差分）。

## 0. 范围与关联格

| 模块 | 矩阵格 | 本版覆盖入口 | 分子强度 |
|---|---|---|---|
| 自机移动 | B-SYS-MOVE | 输入位图→位移相关性 / X·Y 钳制 / 亚像素累积 / 速度对 | **L2**（R-move-1..6） |
| 射击/投放 | B-SYS-FIRE | A 边沿触发投放例程流 / 消耗 / 坐标快照 / mode:=7 / 主武器弹药 | **L2**（R-fire-1..6） |
| 死亡/重生事件 | B-SYS-RESPAWN | 事件窗口 / 移动冻结 / 坐标重置 128/160 / 弹药回满 4（2026-10-12 t27 按约束①准入） | **L2**（R-respawn-1..4） |

不在范围：判定点/受击盒（**受击判定 hit-test 本身 = spec 未证域 → H-16**，§4 留空）、死亡语义
（残机扣除/游戏结束条件，W3 未证不写）、掉落物语义（H-12/N-0027）、等级/红圈语义（H-12/W3/W4）、
商店交互（[shop-economy.md](shop-economy.md)）。

## 1. 常量对照（同步 [constants.md](constants.md)）

| 常量 ID | 值 | 证据 | 级 | 关联 |
|---|---|---|---|---|
| C-vel-h | 速度对 (208,2) = **+2.8125 px/帧**（right/down） | `bank03:A0BE-A0CB`/`A08D-A098` + probe RAM diff | A | R-move-6 |
| C-vel-h-neg | 速度对 (48,253) = **−2.8125 px/帧**（left/up） | 同上 | A | R-move-6 |
| C-clamp-x | X 钳制 **$0E–$F3**（14–243） | `bank03:A0D0-A0E3` + probe diff | A | R-move-3 |
| C-clamp-y | Y 钳制 **$14–$DF**（20–223） | `bank03:A09D-A0B0` + probe diff | A | R-move-4 |
| C-bomb-consume | 消耗量表 `$A336,Y` 逐值 **[1, 2, 3, 2]**（Y=0..3） | `bank03:A336-A339`（seed 字节 `01 02 03 02`） | A | R-fire-3 |
| C-bomb-type-tbl | 类型表 `$A33A,Y` 逐值 **[2, 3, 4, 4]**——**登记不解释**（映射 → H-11） | `bank03:A33A-A33D`（seed 字节 `02 03 04 04`） | A | R-fire-3、H-11 |

## 2. 行为规则 —— R-move-*

### R-move-1: 输入位图语义：`controller_held`=$18（bit0=R, bit1=L, bit2=D, bit3=U, **bit6=B**, bit7=A），`controller_pressed`=$16 为边沿脉冲
- 证据: `bank03:A084-A0E8` 移动例程读 `$18` 位、`bank03:A2B4 LDA $16/AND #$80` 边沿用法 + `trace:analysis/traces/gameplay-stage1.frames.jsonl`（逐帧 zp watch 与 6 段 hold 一一对应：right 段 bit0=1…A 段 bit7=128）；`docs/findings/ram-watch-s3.md` §3
- 可判定性: 机器可判定（L3 帧级 trace diff：输入位图 ∈ 状态向量逐 tick 比对）
- 关联: B-SYS-MOVE、B-SYS-FIRE、N-0018、N-0019、N-0072（bit6=B）
- 【2026-10-12 t56 更正（t52/N-0072）】补 **bit6=B 键**（trace 实测 f1501 `press 1500 b` → `$18=$42`=bit1+bit6）；原四位图缺 B 位，B 长按连发门控依赖此位（R-shot-gen-2）

### R-move-2: 六段 hold 方向相关性：位移符号与方向位完全吻合（right/left/right/down/up/right 六段无一例外）
- 证据: `scenario:analysis/traces/gameplay-stage1`（hold 1300-1450 R / 1470-1620 L / 1650-1800 R / 1830-1950 D / 1980-2100 U / 2130-2280 R）+ probe RAM diff（`gameplay-stage1_f*_probe.ram.hex`）：X 右段 184→243、左段→14、Y 段恒 128；Y 下段 160→174→223、上段 223→54→37、X 段恒 160（`docs/findings/ram-watch-s3.md` §1）
- 可判定性: 机器可判定（L3：6 段输入序列逐 tick 比对 $04E0/$04FC）
- 关联: B-SYS-MOVE、N-0011、N-0012

### R-move-3: X 轴钳制边界行为：$04E0 被钳在 $0E–$F3；触钳时亚像素累加器 $0470 归零
- 证据: `bank03:A0D0-A0E3`（`CMP #$F4/LDA #$F3`、`CMP #$0E/LDA #$0E`、钳制分支 `A0E3 LDY #0`）+ probe diff：right 段到 **243(=$F3)** 不再增、left 段到 **14(=$0E)** 不再减（`ram-watch-s3.md` §1）
- 可判定性: 机器可判定（L2 子程序级 diff：给定入参跑移动例程到 RTS 比对写集；L3 帧级同理）
- 关联: B-SYS-MOVE、C-clamp-x、N-0011、N-0013

### R-move-4: Y 轴钳制边界行为：$04FC 被钳在 $14–$DF；触钳时亚像素累加器 $048C 归零
- 证据: `bank03:A09D-A0B0`（`CMP #$E0/LDA #$DF`、`CMP #$14/LDA #$14`、`A0B0` 归零）+ probe diff：down 段到 **223(=$DF)** 不再增、up 段递减至 37（未触 $14 下限）（`ram-watch-s3.md` §1）
- 可判定性: 机器可判定（L2/L3，同 R-move-3）
- 关联: B-SYS-MOVE、C-clamp-y、N-0012、N-0014

### R-move-5: 亚像素累积语义：像素位置每帧 `ADC <subpix>` 后由速度对的整数项进位修正；亚像素对（$0470/$048C）瞬态值 16/64/96 与累加轨迹一致
- 证据: `bank03:A0C2 ADC $0470 / A0E8 STY $0470`（X）、`bank03:A091 ADC $048C / A0B5 STY $048C`（Y）+ probe 瞬态值（`ram-watch-s3.md` §1、N-0013/14）
- 可判定性: 机器可判定（L2 子程序级 diff 比对 subpix 写集）；"1/256 分数"分划为 B 级（W6 待 S1 查表复核）
- 关联: B-SYS-MOVE、N-0013、N-0014、W6

### R-move-6: 速度对复利关系：位置每帧由（亚像素, 有符号整数）速度对合成；水平/垂直对称，无输入段归 (0,0)
- 证据: `bank03:A0BE-A0CB`（`LDA $0550/ADC` + `LDA $056C BMI` 判向）、`bank03:A08D-A098`（$0588/$05A4）+ probe：right 段 (208,2)=+2.8125px/帧、left 段 (48,253)=−2.8125、down/up 同值、无输入段 (0,0)；60 帧段位移 168.75px 与实测一致（`ram-watch-s3.md` §1）
- 可判定性: 机器可判定（L3：速度对 ∈ 状态向量逐 tick 比对）
- 关联: B-SYS-MOVE、C-vel-h、C-vel-h-neg、N-0015、N-0016

## 3. 行为规则 —— R-fire-*

### R-fire-1: A 边沿触发投放例程入口：`$16` bit7 脉冲门（`AND #$80 / BEQ` 退出）
- 证据: `bank03:A2B4-A2B8`（`LDA $16/AND #$80/BEQ locret_A2FA`）+ `trace:analysis/traces/gameplay-stage1.frames.jsonl`（A 按下帧 $16=128 脉冲；N-0019 代码用法 `bank00:9D84 AND #$40 / 9D8D AND #$80` 同族）
- 可判定性: 机器可判定（L2 子程序级 diff：给定 $16 快照跑例程到 RTS 比对分支）
- 关联: B-SYS-FIRE、N-0019

### R-fire-2: 投放前置校验链（顺序固定）：① `$0401`（~~game_action_mode~~【2026-10-14 更正（复核 #57）】旧名，N-0204 收敛为 `obj_state_slot1`）==0 否则退出 ② `$018F`（bomb_ammo）≠0 否则退出
- 证据: `bank03:A2BA-A2C2`（`LDA $0401/BNE locret`、`LDA $018F/BEQ locret`）+ N-0024（$0401 同址多用途按模式分列）
- 【2026-10-14 A5 语义补全】①的 `$0401`=**对象网格 slot1 状态字节**（N-0204）：0=slot1 空闲=可投放、7=投放物在途（~~209f 寿命~~【2026-10-14 更正（复核 #57 AR57-6）】**寿命=行为链合成值（相位链 27+180+2f；209f 为 chain-A 实测值，非恒定常数——他链实测 254f（JP f2115-2368），移植端禁止硬编码 209）**，`bank03:A2F7` 置 7、`bank14:D10E` 释放清 0）；动态印证 gam401-probe 门读 5 次 v=0=5 次成功投放。死亡/重生窗（E1/E2）内投放例程整体不被调用（门读都不发生）——投放封锁是调用方层（归 H-16/W3），非本链拒绝
- 可判定性: 机器可判定（L2：入参快照逐分支比对）
- 关联: B-SYS-FIRE、N-0017、N-0024、N-0204

### R-fire-3: 消耗与类型查表：按 `$3B` 索引 `CMP $A336,Y` 消耗量表（逐值 [1,2,3,2]）与 `LDA $A33A,Y` 类型表（写 `$0765`）；弹药不足消耗量时 `LDY #1` 回退档
- 证据: `bank03:A2C4-A2D3`（`LDY $3B`、`CMP $A336,Y/BCS`、`LDY #1`、`LDA $A33A,Y/STA $765`）+ seed 字节 `$A336=01 02 03 02`、`$A33A=02 03 04 04`；回退档 Y=1 = 消耗 2（`ram-watch-s3.md` §4）
- 可判定性: 机器可判定（L2：查表值与分支逐值可断言）；类型值语义未证（H-11）
- 关联: B-SYS-FIRE、C-bomb-consume、C-bomb-type-tbl、H-11

### R-fire-4: 炸弹消耗计算与回写：消耗量与现有弹药取小（`CMP $018F` 后 `TAY` 选择），经 `JSR $ED03` 扣减回写 `$018F`
- 证据: `bank03:A2D6-A2E6`（`LDA $A336,Y/CMP $018F/TAY/LDA $018F/BCC`、`JSR $ED03/STA $018F`）+ HUD/$018F 双向印证（f2160→f2280 HUD 行 `F3→F4` 与 `$018F: 3→4` 同步，`ram-watch-s3.md` §3）
- 可判定性: 机器可判定（L2/L3：$018F ∈ 状态向量）
- 关联: B-SYS-FIRE、N-0017

### R-fire-5: 投放坐标快照：投放时刻玩家坐标复制到炸弹出生槽 `$04E0→$04E1`、`$04FC→$04FD`
- 证据: `bank03:A2E9-A2F2`（`LDA $04FC/STA $04FD`、`LDA $04E0/STA $04E1`）+ N-0020/21（反向印证坐标语义）
- 可判定性: 机器可判定（L2 子程序级 diff：出参写集断言）
- 关联: B-SYS-FIRE、B-SYS-MOVE、N-0011、N-0012、N-0020、N-0021

### R-fire-6: 投放后模式写入：`$0401 := 7`（game_action_mode 行动态）；主武器弹药无限（E-W02 B 级 + press_every 段内 zp 无 12 帧周期递减的反向观测）
- 证据: `bank03:A2F5-A2F7`（`LDA #$07/STA $0401`）+ N-0024；无限弹药：`web:Wikipedia E-W02`（B 级）+ `trace:gameplay-stage1.frames.jsonl`（A 段逐帧 zp 扫描无递减项，B 级反向印证，`ram-watch-s3.md` §4）
- 可判定性: 机器可判定（L3：$0401 ∈ 状态向量）；无限弹药封顶 B 级待 L3 正向差分
- 关联: B-SYS-FIRE、N-0024、E-W02

## 3.5 行为规则 —— R-respawn-*（2026-10-12 t27 追加：t24 L3 发现规格化）

> 证据主干：`oracle/traces/gameplay-stage1.player.jsonl`（确定性 L3 trace，f1260-2400）的帧号段与边界状态
> + t24 任务输出（L3 试点发现）+ `shot:shots/gameplay-stage1_f0{1860,1920,2040,2100,2160,2220}_probe.png`
> （画面佐证；t24 通报 t12 时代 probe 于 f1830 后与确定性 run 分岔，**数值一律以 trace 为准**）。

### R-respawn-1: 死亡/重生事件窗口存在：确定性 trace 中三段帧域被事件遮蔽（行缺口 86/55/45 帧）
- 证据: `oracle/traces/gameplay-stage1.player.jsonl` 行缺口 = **f1831-1916 / f2047-2101 / f2131-2175**（955 行 / 1140 帧域，缺 185 行）+ t24 任务输出（"死亡/重生事件…帧域 f1831-1916/f2047-2101/f2131-2175"）
- 可判定性: **机器可判定（L3 已可判）**：trace 帧域遮蔽本身即断言点；窗口内行为待实现后由 L3 状态向量比对收口
- 关联: B-SYS-RESPAWN

### R-respawn-2: 事件期间移动冻结：输入仍在采集，但位置与亚像素保持恒定
- 证据: `oracle/traces/gameplay-stage1.player.jsonl` **f2102-2130 直接证据**：`held` 在 0/128 间变化而 `x/y/sx/sy` 恒为 `128/37/0/96`（29 帧冻结，位于缺口 2、3 之间）；缺口内特征由 t24 L3 观测同证
- 可判定性: **机器可判定（L3 已判）**（x/y/sx/sy ∈ 状态向量逐 tick 断言冻结）——【2026-10-12 t44 升格】t40 证据包（`respawn-window.md` §3）逐帧向量实证：E1 f1831-1915（85f）/ E2 f2046-2174（129f）x/y/sx/sy 全程恒定而 held 持续变化（E1: $04/$84、E2: $01/$81）；【FA-2 降级注已撤销】原"零断言/待新场景"注记由 t40 采证补足（原记载：2026-10-12 t37 降级）
- 关联: B-SYS-RESPAWN、R-move-1（输入位图仍更新）
- 备注【2026-10-12 t44 收口】：t27 B 级观察"缺口 2+冻结段+缺口 3 疑为同一事件分段遮蔽"**升级 A 级实证**——t40 E2 实测 f2046-2174 为**单一连续冻结事件**（129f），t24 三段遮蔽窗是 harness 分段掩码的产物；事件内细分时序定谳=冻结→N-1 弹药回满→N 坐标复位

### R-respawn-3: 重生坐标重置：事件结束后玩家复位到 **(128, 160)** 锚点
- 证据: `oracle/traces/gameplay-stage1.player.jsonl` 边界双向：**f1917**（缺口 1 后）`x=128, y=165`（160 + Down 位移）、**f2176**（缺口 3 后）`y=160, x=133`（128 + Right 位移）；复位锚点 (128,160) 与开局起点一致（f1260 首行 `x=128,y=160`）
- 可判定性: **机器可判定（L3 已判）**（复位帧的状态断言）——【2026-10-12 t44 升格】t40 双事件复位帧实证：f1916 `x=128/y=162`（160+Down 位移）、f2175 `x=130/y=160`（128+Right 位移）——锚点 (128,160) 双事件一致；【FA-2 降级注已撤销】（原记载：t37"可判≠已判"）
- 关联: B-SYS-RESPAWN、N-0011、N-0012
- **W5 收口**：原"up 段后 Y 自动回中（f2160=37 → f2220=160）"即本事件的坐标重置（确定性 trace：f2130 y=37 → f2176 y=160），**非自动回中机制**

### R-respawn-4: 重生弹药回满：`$018F`（bomb_ammo）复位为 **4**
- 证据: t24 任务输出（L3 状态向量观测："弹药回满"）+ 初始化同值 `bank00:9C5D STA $018F=4`（N-0017）；`player.jsonl` 不含弹药字段，弹药断言走 L3 状态向量
- 可判定性: **机器可判定（L3 已判）**——【2026-10-12 t44 升格】t40 逐帧 `bomb_ammo` watch 实证：f1915/f2174 弹药突变至 **4**（此前分别为 1、3），断言载体 `analysis/traces/respawn-window.frames.jsonl`；【FA-2 真覆盖需求两项已落齐并撤销】t37 尾注的需求（窄窗场景 + $018F/$04E0/$04FC watch）由 t40 `oracle/scenarios/respawn-window.txt`（240 窄窗全采样）+ 向量扩展满足
- 关联: B-SYS-RESPAWN、N-0017、R-fire-2（$018F 非零校验）
- **时序定谳（t40）**：事件内顺序 = **冻结 → N-1 弹药回满 → N 坐标复位**（E1: 冻结 1831-1915→弹药 1915→坐标 1916；E2: 2046-2174→2174→2175）

## 4. 待取证空位（留空，禁止发明）

- [x] ~~主武器射击例程本体未定位/分工机制未证（H-14）~~ → 【t36 收口】→ **【2026-10-12 t56 更正（t52 F1 冲击波）】t36 定谳文本两处有误，正确分工**：**A 边沿=投放**（R-fire-1 ✓）/**B 长按=主武器连发**（`$18` bit6→`$39/$3A` 计时器→`$32` bit2 门，R-shot-gen-2）/**弹种切换=拾取字母圈触发**（R-bomb-2/3，非 B 键）；~~"主武器=计数器驱动自动连射/B=弹种切换"~~ 作废（N-0072/N-0073）；详见 [shot-system.md](shot-system.md)
- [x] ~~`game_action_mode`（$0401）除 0/7 外的值域语义（N-0024 同址多用途）→ **H-15**~~ → 【2026-10-14 A5 收口】"0/7 之外值域"问题不成立：$0401 同址三义（gameplay=对象网格 slot1 状态字节 N-0204 / shop=ui_select_cursor N-0003 / scene=CHR bank 影子 R1 N-0205），110/124=scene 影子值非枚举非位组合；详见 [game-action-mode-domain.md](../findings/game-action-mode-domain.md)
- [ ] **受击判定（hit-test）本身未证**（判定盒/触发条件/无敌窗口）→ **H-16**；t25 `oracle/scenarios/hit-test.txt` 场景可重跑取证（t25 W3：hit-test 未进实机，退出路径 run 间分岔）
- [ ] **死亡语义**（残机扣除/游戏结束条件）未证不写（残机 var 仍 W3 `_todo_`）
- [ ] **E1/E2 窗长差异续问**：t40 实测 E1=86f（f1831-1915）vs E2=129f（f2046-2174）——同一事件类窗长不一致，影响因子（受击时刻/无敌帧/演出差异？）未证 → **H-16 续问**
- [ ] `$3B`（查表索引）的取值来源与值域（与炸弹槽/字母圈对应？→ H-11 族）
- [ ] 判定点/受击盒（矩阵 §2.6 遗留项）
- [ ] 红圈掉落物语义、HUD 等级/残机语义（H-12、W3/W4，留命名台账）
- [x] ~~up 段后 Y 自动回中机制（W5）~~ → **【2026-10-12 t27 收口】W5 = 死亡/重生事件的坐标重置**（R-respawn-3；确定性 trace f2130 y=37 → f2176 y=160 复位锚点），非"自动回中"机制
- [ ] 速度对查表 `$A2FB/$A30B`（W6，B 级，`bank03:A073/A079`）

## 5. 门禁

`python3 gates/cli.py all` → 见任务 t23 输出（矩阵同步用 `tools/matrix_sync.py --export && --check`）。

## spec 产出契约

- 规则清单: R-move-1..6 + R-fire-1..6 + R-respawn-1..4（共 16 条，逐条带 bank:addr + scenario/trace 指针）
- 常量对照: [constants.md](constants.md) 已同步（C-vel-h/C-vel-h-neg/C-clamp-x/C-clamp-y/C-bomb-consume/C-bomb-type-tbl）
- 可判定性标注: 机器可判定 16 条（R-respawn-1..4 **均 L3 已判**【2026-10-12 t44 升格，t40 证据包 respawn-window.md §3】；FA-2 降级注已撤销；无限弹药子项封顶 B 级）
- 门禁: `python3 gates/cli.py all`（见任务 t27 输出）
- 遗留假设: H-11、H-12、H-14、H-15、**H-16（受击判定 hit-test 未证域）**、W6（C/B 级，未入规则结论）；**W5 已收口**（= R-respawn-3 重生坐标重置）
