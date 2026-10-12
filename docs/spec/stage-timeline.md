# spec: 关卡时间线（stage-timeline）→ docs/spec/stage-timeline.md

> 状态：2026-10-12 v1（任务 t46，spec-writer，关卡系统规则面开张）。证据基座：
> `docs/findings/stage-timeline.md`（t42，STG-01 完整 + 其余定位）+ `docs/findings/enemy-dataface.md`
> （t32 行为脚本/t_005）+ seed listing（bankNN:AAAA 为 CPU 地址权威）。
> 纪律：**结构级 A/B 证据入规则；段参数语义（时刻/序号/掩码）留 S1 假设，不发明**（§5）。
> 矩阵格：**B-STG-01-TL**（首个关卡子格，按 §2.3 子格规则证据准入）；其余 TL/EN/BOSS 子格不预建。

## 0. 范围与关联格

| 模块 | 矩阵格 | 覆盖入口 | 强度 |
|---|---|---|---|
| STG-01 滚动时间线 | B-STG-01-TL | 波次链结构 / 滚动指令嵌入 / bank 图归属 / 目录顺序扫入口 | **L2/部分**（R-stage-1..6；帧级对齐**不成立**——t47 负结论） |
| spawn 节律 | B-SYS-SPAWNRHYTHM | 16 帧周期连发 / 突发簇 / 独立定时机制（2026-10-12 t49 约束①准入） | **L2→L3 部分**（R-spawn-1..3） |

不在范围：段参数语义（S1）、目录消费例程（S2）、页目录读者（S3）、地形碰撞（S4）、关卡长度/切换条件（S5）、
其余关卡展开（S6）——全部假设留 §5；敌人生成行为（enemy 线）；Boss（B-SYS-BOSS L0）。

## 1. 常量/数据锚点（同步 [constants.md](constants.md)）

| ID | 内容 | 证据 | 级 | 关联 |
|---|---|---|---|---|
| C-wave-chain | 波次链区 `bank8:$AD00-$ADD0`（file 0x10D10-0x10DE0） | t42 §2.1 实录 | B | R-stage-1 |
| C-wave-dir | 目录 `bank8:$ACEF` 范围对（`44 AD D0 AD` = $AD44-$ADD0） | t42 §2.3 | B | R-stage-3 |
| C-scroll-vars | 滚动寄存器变量 `$1C/$1D`（NMI `$2005` 消费） | t42 §4 + S0 | B | R-stage-4 |

## 2. 行为规则 —— R-stage-*（结构级；多为 **L2 数据解析可判**）

### R-stage-1: 波次链 = 分段事件记录流（**记录格式已证 A 级**）：`bank8:$AD00-$ADD0`，段间 `00 FE` 分隔；段内=自指首字 + 行为脚本指针×1-2 + 参数字节；**链末 FF 终止**
- 证据: `docs/findings/wave-semantics-s1.md` §1（A 级字节实证：FE 分段 **8 块**、自指首字、链末 FF、脚本指针×1-2）+ `docs/findings/stage-timeline.md` §2.1 实录 + `xxd -s 0x10D50 -l 96` 复现
- 可判定性: **机器可判定（L2 数据解析）**：解析器断言 FE 分段/首字/FF 链尾结构
- 关联: B-STG-01-TL、Wv-1（首字用途未定）
- 【2026-10-12 t49 修正（t47 定谳）】原文"自引指针递增（+8/+0x12）""参数对递增小整数"**系地址伪影**——块首字=**自块起点−6**（7/7 规律实证，顺序布局副产物），参数实为 3,5,6,5,5,5,6 **非单调**；"递增"表述删除，t42 原观察留档为伪影更正

### R-stage-1a: 段参数语义三裁定（t47 S1 定谳，入规则注不发明）
- 裁定：**时刻（线性帧映射）✗ 证伪**（帧级网格拟合最优 5/9≈偶然命中率，未达显著）；**序号 ✗ 证伪**（参数 3,5,6,5,5,5,6 非单调）；**掩码/标志 部分支持（B 级）**（`04 FD/08 FD/10 FD` = 单比特 0x04/0x08/0x10 + 常量伴字节 $FD 形态；`08 00` 变体（无 FD）并存 → 字段语义待消费者定谳 Wv-2）
- 证据: `docs/findings/wave-semantics-s1.md` §2（拟合表 + 非单调实测 + 字节形态）
- 可判定性: 仅人工核对（语义未定谳；掩码形态可 L2 字节断言）
- 关联: B-STG-01-TL、Wv-2

### R-stage-2: 段内行为脚本指针指向 bank8 窗内脚本流（与 t_005 生成记录同族）
- 证据: 指针值 `$B2E6/$B5C1/$B501/$B396/$B611/$B661`（bank8 $A000 窗内）+ t32 §1.1 行为脚本流格式（嵌套指针 + `FE/FC` 终止，`enemy-dataface.md`）+ `docs/findings/stage-timeline.md` §2.2（t_005 5 字节 `{参数×3, 脚本指针}`）
- 可判定性: 机器可判定（L2 数据解析：指针域校验 + 脚本流终止符扫描）
- 关联: B-STG-01-TL、N-0052（enemy_script_stream 同族）

### R-stage-3: `$ACEF` 目录 = **顺序扫入口**（B 级收口）：`{1, $AD44 波次链起点, $ADD0 生成记录起点, 0}`——随机访问语义弱化
- 证据: `docs/findings/wave-semantics-s1.md` §3（收口）+ `docs/findings/stage-timeline.md` §2.3 + `xxd -s 0x10CE0 -l 176` 复现
- 可判定性: 机器可判定（L2 数据解析：目录四元组结构断言）；消费例程仍待反查（间接寻址 Wv-1/Wv-2 族）
- 关联: B-STG-01-TL、S2（语义已收口为顺序扫入口；消费例程留 W 系）
- 【2026-10-12 t49 收口（t47）】"块范围对/消费方式未证（S2）"→ 定谳**顺序扫入口**

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
- 可判定性: 部分：段序列结构 L2 可判；**帧级对齐不成立**【2026-10-12 t49 登记（t47 负结论）】——时刻/序号两假设证伪后无帧映射基础，**L3 升格否定**（t47 明确"不建议升格"），STG-01-TL 维持 L2/部分
- 关联: B-STG-01-TL、S1（已裁定，见 R-stage-1a）、S5（关卡长度/切换条件未证）

## 2.5 行为规则 —— R-spawn-*（spawn 节律观察规则，2026-10-12 t49 新格准入）

> 格子 **B-SYS-SPAWNRHYTHM** 按约束①准入（t47 证据包：trace A 级 104 spawn 事件）；
> 事件定义 = 对象槽 `$0400+n×$1C` 激活（0→非 0）。可判定性标注 **L2→L3 部分**。

### R-spawn-1: spawn 节律 = 16 帧周期连发（三段实测：f1471-1551、f1655-1731、f1804-1900，各段 +1/16f）
- 证据: `docs/findings/wave-semantics-s1.md` §2 spawn 节律旁证（gameplay-stage1 对象槽激活事件 104 个，逐帧 iram 28 槽位 0→非 0 检测）+ `analysis/traces/gameplay-stage1.frames.jsonl`
- 可判定性: **机器可判定（L2 事件序列断言 → L3 部分）**：16 帧周期断言素材已备（逐帧事件流）
- 关联: B-SYS-SPAWNRHYTHM、Wv-3（定时机制未定位）

### R-spawn-2: 突发簇（脱离周期）：f1301 +3、f1787-91 +7、f1916 +8——**f1916 与重生复位同帧**
- 证据: 同上（104 事件帧表）+ 与 `docs/findings/respawn-window.md` E1 坐标复位帧（f1916）交叉吻合
- 可判定性: 机器可判定（L2 事件序列断言）；突发归因（重生/脚本触发）未证
- 关联: B-SYS-SPAWNRHYTHM、B-SYS-RESPAWN、Wv-3

### R-spawn-3: 节律由**独立定时机制**驱动（16 帧计时器候选）——非波次链参数直接编码（与 R-stage-1a 时刻✗ 证伪互证）
- 证据: `docs/findings/wave-semantics-s1.md` §2 结论行（时刻映射证伪 ⇒ 节律源另寻）；定时 var/例程**未定位** → Wv-3
- 可判定性: 部分（机制定位后可 L3 全判）；当前=观察级规则
- 关联: B-SYS-SPAWNRHYTHM、Wv-3、R-stage-1a

## 3. 汉化特有

本批次证据为数据结构层，未涉文本/渲染；汉化差异（H-08）不适用。

## 4. 待证空位（留空，禁止发明）

- [ ] 段参数语义（时刻/序号/掩码）→ **S1**（段消费代码定位 + 帧级对齐）
- [ ] `$ACEF` 目录消费方式 → **S2**（装载例程定位）
- [ ] bank2 `$8100-$8220` 页目录读者（`$A0-$BF` 页值表，seed 曾误判代码——数据表已实证）→ **S3**（LDA (zp),Y 族 xref 破译）
- [ ] 地形碰撞数据（`$AC80` tile 流 / `09 20 28` 公共段）→ **S4**
- [ ] 关卡长度/切换条件（终止符/计数）→ **S5**
- [ ] bank9/10 逐关归属与展开（STG-02/03 候选 → 实证）→ **S6**
- [ ] t47 波次语义遗留 **Wv-1..5**（`docs/findings/wave-semantics-s1.md` §7）：Wv-1 链首字用途（自基址/反向链）、Wv-2 `XX FD` 标记+参数对语义、Wv-3 16 帧节律定时机制、Wv-4 地形流消费例程、Wv-5 链尾段（`C8 AD` 后续）——**与 ram-watch W 系列编号撞车，引用须带来源文件**

## 5. 门禁

`python3 gates/cli.py all` → 见任务 t46 输出。

## spec 产出契约

- 规则清单: R-stage-1..6 + R-stage-1a（S1 三裁定注）+ R-spawn-1..3（共 10 条，逐条带 bank:addr/scenario + 复现指针）
- 常量对照: [constants.md](constants.md) 见 §1 三锚点（C-wave-chain/C-wave-dir/C-scroll-vars）
- 可判定性标注: 机器可判定 7 条（L2 数据解析/事件序列）+ 部分/人工 3 条（R-stage-1a 语义未定谳、R-stage-6 帧级对齐不成立、R-spawn-3 机制未定位）
- 门禁: `python3 gates/cli.py all`（见输出）
- 遗留假设: S1-S6（S1 已裁定见 R-stage-1a，S2 收口）+ t47 Wv-1..5（§4，编号与 ram-watch W 系列撞车须带来源）；索引见 completeness §7.1
