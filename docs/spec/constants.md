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

## 2. 登记不解释（存在性已证，含义未证）

| 常量 ID | 值 | 来源 | 级 | 关联 | 备注 |
|---|---|---|---|---|---|
| C-money-unk | `10000000000` | `rom:4759` | A | R-money-3、H-03 | **禁止解释**：上限/显示常量/渲染数值均属猜测，待 S2/S3 取证 |
| C-bomb-type-tbl | `$A33A,Y` = **[2, 3, 4, 4]**（Y=0..3） | `bank03:A33A-A33D` seed 字节 `02 03 04 04` | A | R-fire-3、H-11 | **禁止解释**：类型值 ↔ 炸弹字母（F/B/T/W）映射未证（H-11） |
| C-vel-tables | `$A2FB`/`$A30B` 速度查表（`bank03:A073/A079` 索引） | `bank03:A2FB-A31F` seed 字节 | A（存在）/B（分划） | R-move-5、W6 | **禁止解释**：亚像素/整数分划与索引语义未证（W6） |

## 3. 版本记录

- `v1 (2026-10-12) 建立：C-shop-p1..p3、C-money-template、C-money-unk（后者不解释）`
- `v2 (2026-10-12, t23) 移动/射击常量：C-vel-h、C-vel-h-neg、C-clamp-x、C-clamp-y、C-bomb-consume（逐值）；C-bomb-type-tbl、C-vel-tables 入登记不解释`
