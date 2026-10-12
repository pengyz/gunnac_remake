# spec: 敌对象系统（enemy-system）→ docs/spec/enemy-system.md

> 状态：2026-10-12 v1（任务 t60，spec-writer，enemy 模块合同开张）。证据基座：
> `docs/findings/enemy-dataface.md`（t32：t_001/t_005 重分类 + 双 SoA 字段体系）+
> t48/t52 定谳合流（shot-slot-geometry / shot-gating-f1 的弹对象侧）+ t29（$70 SoA/脚本机）。
> 纪律：已证部分入规则；命中/受击判定（H-16 域）与参数语义（B 系假设）**不发明**（§4）。
> 矩阵格：**B-SYS-ENEMY**（系统级机制格，约束①准入）；B-STG-xx-EN 逐关子格仍按"证据后准入"不预建。

## 0. 范围与关联格

| 模块 | 矩阵格 | 覆盖入口 | 强度 |
|---|---|---|---|
| 敌对象生命周期/双 SoA/生成记录 | B-SYS-ENEMY | t_005 生成记录 / 行为脚本流 / 双 SoA 网格 / RAM 绑定流 / 生命周期已证链 | **L2**（R-enemy-1..6） |

不在范围：命中/受击判定（**H-16 域维持待证**——t60 脚本机考证在途可分批回填）、HP 字段定谳（B4）、
逐关生成编排（B-STG-xx-TL/EN 子格，stage-timeline.md）、弹对象运动机（R-shot-motion-* 已覆盖）。

## 1. 常量/数据锚点（同步 [constants.md](constants.md)）

| ID | 内容 | 证据 | 级 | 关联 |
|---|---|---|---|---|
| C-spawn-rec | t_005 生成记录：5 字节 `{参数×3, 脚本指针(2,LE)}` | enemy-dataface §1.1 | A/B | R-enemy-1 |
| C-grid-1c | 关卡对象网格：**$1C 间距 28 槽**（n×$1C 字段阵列） | enemy-dataface §2.1 + t13/t18/t25 锚点 | A | R-enemy-3/4 |
| C-grid-70 | 弹对象网格：**$70 间距**（六字段，t29） | shot-behavior §1.3 + t48/t52 | A | R-enemy-3 |

## 2. 行为规则 —— R-enemy-*（结构级；L2 数据解析可判为主）

### R-enemy-1: 敌人生成记录表 t_005（`bank8:$ADD0-$BFFF`，4656B）：5 字节记录 `{参数×3, 行为脚本指针(2, LE, bank8 窗内)}`
- 证据: `docs/findings/enemy-dataface.md` §1.1 实录（rec0 `bf 08 20 e6 b3`→$B3E6、rec1 `8b 1a 11 b4 b2`→$B2B4 …共 6 条样本）+ 消费侧 xref（指针字 `$B2B4` 本体内被引 8 次，记录间自引用 ✓）+ `xxd -s 0x10DE0` 复现命令
- 可判定性: **机器可判定（L2 数据解析）**：5 字节切分/指针域（bank8 窗内）断言
- 关联: B-SYS-ENEMY、C-spawn-rec、B1（三参数语义未定：$80-$BF/$08-$1A/$10-$21 域）

### R-enemy-2: 行为脚本流（`bank8:$B2B4` 族）：嵌套脚本指针（自/互引 `$B2B4/$B2CA/$B2BE/$B279`）+ `FE/FC` 终止符；供 bank2 解释器 `($14),Y` 取指消费
- 证据: `docs/findings/enemy-dataface.md` §1.1 实录（file 0x112C4：`02 79 B2 04 00 00 FE | B4 B2 …`）+ t29 §1.3 同族（弹对象脚本机取指）+ `docs/spec/stage-timeline.md` R-stage-2（同族指针）
- 可判定性: 机器可判定（L2 数据解析：指针域/终止符扫描）；脚本指令语义待 t60 脚本机考证（在途可分批）
- 关联: B-SYS-ENEMY、R-stage-2、R-shot-motion-1（同族取指）

### R-enemy-3: **双 SoA 网格并存**：关卡对象 `$1C` 间距 **28 槽**（敌/玩家/道具通用）+ 弹对象 `$70` 间距（t29 六字段）——两套布局独立建模
- 证据: `docs/findings/enemy-dataface.md` §2.1/§2.2（raw `LDA/DEC $xxxx,X` 字段扫描 + t18/t25/t29 锚点合流）+ t48（$70 网格槽位几何 4 组×5）+ t52（槽迭代 +$7E/+$1C×3 交错实证）+ N-0054（obj_field_grid_1c）
- 可判定性: 机器可判定（L2 数据解析：字段基址+n×$1C 布局断言）；两系统对象关系未证（B5）
- 关联: B-SYS-ENEMY、C-grid-1c、C-grid-70、B5、Ws-7（$1C/$70 同址交错残余）

### R-enemy-4: `$1C` 网格字段阵列（槽 n 字段基址 = base+n×$1C）：类型/状态 `$0400`、x/y 亚像素 `$0470/$048C`、x/y 坐标 `$04E0/$04FC`（钳制 `$0E-$F3`/`$14-$DF`）、速度四元组 `$0550/$056C/$0588/$05A4`、倒计时簇 `$05C0+`
- 证据: `docs/findings/enemy-dataface.md` §2.1 字段表（访问热度 LDA,X：坐标 88/91、$0710=63 等）+ t18 N-0011..0016 锚点（玩家=槽 0 绝对寻址路径，B3）+ `docs/spec/player-movement.md` R-move-3/4 钳制值互证
- 可判定性: 机器可判定（L2：字段写集/钳制断言）；HP 字段（$05DC/$0630/$06A0/$06BC 候选）**未定谳** → B4
- 关联: B-SYS-ENEMY、N-0011..0016、B3、B4

### R-enemy-5: t_001 对象 RAM 绑定流（`bank1:$B2A4-$BFFF`，3420B）：变长记录 `{参数(2-3), RAM 槽指针(2, LE)}`，375 指针**单调 +2**（间距 4/5 主、偶发 8）
- 证据: `docs/findings/enemy-dataface.md` §1.2（指针序列 $009D→$009F→$00A1…375 字实录）+ branch-align 77.2%/63.4% 维持 data 分类（t13『4 字节组』修正为变长）
- 可判定性: 机器可判定（L2 数据解析：变长切分/指针单调性断言）；字段语义未定 → B2
- 关联: B-SYS-ENEMY、B2

### R-enemy-6: 生命周期已证链：**生成**（t_005 记录 → 行为脚本流）→ **活动**（解释器 tick + $1C 网格字段驱动）——**命中/清除段待证**（H-16 域）
- 证据: §2.1/§1.1 交叉（生成→脚本→字段活动的静态链完整）+ gameplay-stage1 对象 churn 1090/1090 帧活跃（stage-timeline R-stage-6 旁证）；命中/清除**无已证规则**（hit-test 未取证，见 [player-movement.md](player-movement.md) §4 H-16）
- 可判定性: 部分（生成/活动段 L2 可判）；命中→清除段**待证不发明**
- 关联: B-SYS-ENEMY、H-16、B4（HP 字段=命中侧前置）

## 3. 汉化特有

本批次为结构层，未涉文本/渲染；H-08 不适用。

## 4. 待证空位（留空，禁止发明）

- [ ] 命中/受击判定（H-16 域）——t60 脚本机考证在途可分批回填；当前**维持待证**
- [ ] HP 字段定谳（$05DC/$0630/$06A0/$06BC 候选四选一）→ **B4**
- [ ] t_005 三参数语义（坐标/属性/时序候选）→ **B1**
- [ ] t_001 RAM 槽指针语义（对象属性绑定？）→ **B2**
- [ ] 玩家槽绑定（= $1C 系统固定槽？）→ **B3**（N-0011/12 名称维持，字段阵列语义并存）
- [ ] $1C/$70 两系统对象关系（同物异址/两类对象）→ **B5**、Ws-7 同址所有权残余
- [ ] bank8 $AD00 波次链语义 → 本域 Wv 系列（stage-timeline/wave-semantics 管辖，互链）

## 5. 门禁

`python3 gates/cli.py all` → 见任务 t60 输出。

## spec 产出契约

- 规则清单: R-enemy-1..6（共 6 条，逐条带 bank:addr/表址 + 复现指针）
- 常量对照: [constants.md](constants.md) 见 §1 三锚点（C-spawn-rec/C-grid-1c/C-grid-70）
- 可判定性标注: 机器可判定 5 条（L2 数据解析）+ 部分 1 条（R-enemy-6 命中/清除段待证）
- 门禁: `python3 gates/cli.py all`（见输出）
- 遗留假设: B1-B5（C/B 级，§4；索引见 completeness §7.1 B 系列）+ H-16 命中域
