# spec: 关卡时间线（stage-timeline）→ docs/spec/stage-timeline.md

> 状态：2026-10-12 v1（任务 t46，spec-writer，关卡系统规则面开张）。证据基座：
> `docs/findings/stage-timeline.md`（t42，STG-01 完整 + 其余定位）+ `docs/findings/enemy-dataface.md`
> （t32 行为脚本/t_005）+ seed listing（bankNN:AAAA 为 CPU 地址权威）。
> 纪律：**结构级 A/B 证据入规则；段参数语义（时刻/序号/掩码）留 S1 假设，不发明**（§5）。
> 矩阵格：**B-STG-01-TL**（首个关卡子格，按 §2.3 子格规则证据准入）；其余 TL/EN/BOSS 子格不预建。

## 0. 范围与关联格

| 模块 | 矩阵格 | 覆盖入口 | 强度 |
|---|---|---|---|
| STG-01 滚动时间线 | B-STG-01-TL | 波次链结构 / 滚动指令嵌入 / bank 图归属 / 目录结构 | **L2**（R-stage-1..6；帧级对齐待 S1） |

不在范围：段参数语义（S1）、目录消费例程（S2）、页目录读者（S3）、地形碰撞（S4）、关卡长度/切换条件（S5）、
其余关卡展开（S6）——全部假设留 §5；敌人生成行为（enemy 线）；Boss（B-SYS-BOSS L0）。

## 1. 常量/数据锚点（同步 [constants.md](constants.md)）

| ID | 内容 | 证据 | 级 | 关联 |
|---|---|---|---|---|
| C-wave-chain | 波次链区 `bank8:$AD00-$ADD0`（file 0x10D10-0x10DE0） | t42 §2.1 实录 | B | R-stage-1 |
| C-wave-dir | 目录 `bank8:$ACEF` 范围对（`44 AD D0 AD` = $AD44-$ADD0） | t42 §2.3 | B | R-stage-3 |
| C-scroll-vars | 滚动寄存器变量 `$1C/$1D`（NMI `$2005` 消费） | t42 §4 + S0 | B | R-stage-4 |

## 2. 行为规则 —— R-stage-*（结构级；多为 **L2 数据解析可判**）

### R-stage-1: 波次链 = 分段事件记录流：`bank8:$AD00-$ADD0`，段间 `00 FE` 分隔；段内重复三元组（自引指针 + 行为脚本指针 + 参数对）
- 证据: `docs/findings/stage-timeline.md` §2.1 实录（`39 FD 01 00 D6 B2 | 00 FE | 46 AD 03 00 E6 B2 10 FD 08 00 61 B6 | 00 FE | …`）+ `xxd -s 0x10D50 -l 96` 复现命令 + 自引指针递增结构（`$AD46/$AD54/$AD66/$AD78`，+8/+0x12）
- 可判定性: **机器可判定（L2 数据解析）**：解析器断言分段/三元组结构（流切分与指针域）
- 关联: B-STG-01-TL、S1（参数语义未证）

### R-stage-2: 段内行为脚本指针指向 bank8 窗内脚本流（与 t_005 生成记录同族）
- 证据: 指针值 `$B2E6/$B5C1/$B501/$B396/$B611/$B661`（bank8 $A000 窗内）+ t32 §1.1 行为脚本流格式（嵌套指针 + `FE/FC` 终止，`enemy-dataface.md`）+ `docs/findings/stage-timeline.md` §2.2（t_005 5 字节 `{参数×3, 脚本指针}`）
- 可判定性: 机器可判定（L2 数据解析：指针域校验 + 脚本流终止符扫描）
- 关联: B-STG-01-TL、N-0052（enemy_script_stream 同族）

### R-stage-3: `$ACEF` 目录 = 块范围对（`44 AD D0 AD` = 波次链块边界 $AD44-$ADD0）
- 证据: `docs/findings/stage-timeline.md` §2.3 + `xxd -s 0x10CE0 -l 176` 复现
- 可判定性: 机器可判定（L2 数据解析：范围对结构与块边界吻合断言）；**消费方式未证（S2）**
- 关联: B-STG-01-TL、S2

### R-stage-4: 滚动指令嵌入关卡数据流（**"数据流 = 混合指令流"定性**）：`bank8:$AE87 STA $1C` 写点在波次链同页数据区内，bank9 同族（$824B/$8BF3/$8BF8）；`$1C/$1D` 由 NMI `bank15:$EFC8/$EFCD STA $2005` 消费
- 证据: `docs/findings/stage-timeline.md` §4（写点清单 + `85 1C` 字节序列在数据流内）+ S0 NMI 消费链（`bank15:$EFC8/$EFCD`）+ 区分注记：`bank1:$AB3B-$AB4C` 直写 $2005 = 商店卷轴（与关卡滚动无关）
- 可判定性: 机器可判定（L2 数据解析：写点指令流扫描；L3 待滚动行为差分）
- 关联: B-STG-01-TL、C-scroll-vars、S1（参数对 `10 FD/08 FD` 型与滚动相容，语义未证）

### R-stage-5: bank 图归属：`bank8` = STG-01 关卡数据主 bank（t_005 生成记录 $ADD0-$BFFF + 行为脚本流 + 波次链 + 目录 + 滚动写点全族齐）；`bank9/10` 头同族 = STG-02/03 **候选**（定位级）
- 证据: `docs/findings/stage-timeline.md` §1（bank8 内容族实证；bank9 file 0x12010 / bank10 file 0x14010 头结构同族：`00 00 01` 前缀 + 指针簇 + `09 20 28` 公共尾段）
- 可判定性: 机器可判定（L2 数据解析：头结构指纹比对）；bank9/10 逐字段展开待后续批次（S6）
- 关联: B-STG-01-TL、S6

### R-stage-6: 时间线骨架：波次链段序列 = 关卡推进序（B 级）；STG-01 交叉验证 = gameplay-stage1 trace 对象 churn **1090/1090 帧全程活跃**（波次时刻表定位 ✓，帧级对齐待 S1）
- 证据: `docs/findings/stage-timeline.md` §5（段序列=推进序候选 + 逐时刻对齐待 S1 如实登记）+ `analysis/traces/gameplay-stage1.frames.jsonl` churn 实测
- 可判定性: 部分：段序列结构 L2 可判；**帧级对齐 L3 待 S1 参数语义定谳**（负结果如实登记）
- 关联: B-STG-01-TL、S1、S5（关卡长度/切换条件未证）

## 3. 汉化特有

本批次证据为数据结构层，未涉文本/渲染；汉化差异（H-08）不适用。

## 4. 待证空位（留空，禁止发明）

- [ ] 段参数语义（时刻/序号/掩码）→ **S1**（段消费代码定位 + 帧级对齐）
- [ ] `$ACEF` 目录消费方式 → **S2**（装载例程定位）
- [ ] bank2 `$8100-$8220` 页目录读者（`$A0-$BF` 页值表，seed 曾误判代码——数据表已实证）→ **S3**（LDA (zp),Y 族 xref 破译）
- [ ] 地形碰撞数据（`$AC80` tile 流 / `09 20 28` 公共段）→ **S4**
- [ ] 关卡长度/切换条件（终止符/计数）→ **S5**
- [ ] bank9/10 逐关归属与展开（STG-02/03 候选 → 实证）→ **S6**

## 5. 门禁

`python3 gates/cli.py all` → 见任务 t46 输出。

## spec 产出契约

- 规则清单: R-stage-1..6（共 6 条，逐条带 bank:addr + 复现命令指针）
- 常量对照: [constants.md](constants.md) 见 §1 三锚点（C-wave-chain/C-wave-dir/C-scroll-vars）
- 可判定性标注: 机器可判定 5 条（L2 数据解析）+ 部分 1 条（R-stage-6 帧级对齐待 S1）
- 门禁: `python3 gates/cli.py all`（见输出）
- 遗留假设: S1-S6（C/B 级，§4；索引见 completeness §7.1 S 系列）
