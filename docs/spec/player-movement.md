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

不在范围：判定点/受击盒（§4 留空）、掉落物语义（H-12/N-0027）、等级/红圈语义（H-12/W3/W4）、
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

### R-move-1: 输入位图语义：`controller_held`=$18（bit0=R, bit1=L, bit2=D, bit3=U, bit7=A），`controller_pressed`=$16 为边沿脉冲
- 证据: `bank03:A084-A0E8` 移动例程读 `$18` 位、`bank03:A2B4 LDA $16/AND #$80` 边沿用法 + `trace:analysis/traces/gameplay-stage1.frames.jsonl`（逐帧 zp watch 与 6 段 hold 一一对应：right 段 bit0=1…A 段 bit7=128）；`docs/findings/ram-watch-s3.md` §3
- 可判定性: 机器可判定（L3 帧级 trace diff：输入位图 ∈ 状态向量逐 tick 比对）
- 关联: B-SYS-MOVE、B-SYS-FIRE、N-0018、N-0019

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

### R-fire-2: 投放前置校验链（顺序固定）：① `$0401`（game_action_mode）==0 否则退出 ② `$018F`（bomb_ammo）≠0 否则退出
- 证据: `bank03:A2BA-A2C2`（`LDA $0401/BNE locret`、`LDA $018F/BEQ locret`）+ N-0024（$0401 同址多用途按模式分列）
- 可判定性: 机器可判定（L2：入参快照逐分支比对）
- 关联: B-SYS-FIRE、N-0017、N-0024

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

## 4. 待取证空位（留空，禁止发明）

- [ ] **主武器射击例程本体**（A 按住/边沿与出弹的代码路径）未定位；A 边沿同时驱动 press_every 射击段与投放例程的**分工机制未证** → **H-14**
- [ ] `game_action_mode`（$0401）除 0/7 外的值域语义（N-0024 同址多用途）→ **H-15**
- [ ] `$3B`（查表索引）的取值来源与值域（与炸弹槽/字母圈对应？→ H-11 族）
- [ ] 判定点/受击盒（矩阵 §2.6 遗留项）
- [ ] 红圈掉落物语义、HUD 等级/残机语义（H-12、W3/W4，留命名台账）
- [ ] up 段后 Y 自动回中机制（W5，C 级）；速度对查表 `$A2FB/$A30B`（W6，B 级，`bank03:A073/A079`）

## 5. 门禁

`python3 gates/cli.py all` → 见任务 t23 输出（矩阵同步用 `tools/matrix_sync.py --export && --check`）。

## spec 产出契约

- 规则清单: R-move-1..6 + R-fire-1..6（共 12 条，逐条带 bank:addr + scenario/trace 指针）
- 常量对照: [constants.md](constants.md) 已同步（C-vel-h/C-vel-h-neg/C-clamp-x/C-clamp-y/C-bomb-consume/C-bomb-type-tbl）
- 可判定性标注: 机器可判定 12 条（其中 L2 子程序级 4、L3 帧级 6、混合 2；无限弹药子项封顶 B 级）
- 门禁: `python3 gates/cli.py all`（见输出）
- 遗留假设: H-11、H-12、H-14、H-15、W5、W6（C/B 级，未入规则结论）
