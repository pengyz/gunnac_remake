# 完备性矩阵 v0（completeness matrix）—— 迭代队列的分母来源

> 状态：2026-10-12 建立 v0（任务 t1，spec-writer）。
> 用途：为 master-plan §3 迭代流水线①"取格"提供**分母**；为"绿 ≠ 完整"提供收敛度量。
> 纪律以 [AGENTS.md](../../AGENTS.md) 为准；管线定义见 [ai-decompile-pipeline.md](../ai-decompile-pipeline.md) §4。
>
> **本文件三条铁律**：
> 1. **分母独立于测试**——分母来自 ROM/外部资料/运行截图的证据清单，不来自"测试写了多少"。
> 2. **无证据不入分母**——任何猜测、惯例推断、"这类游戏应该有"的东西只能进 §7 假设区，禁止混入分母。
> 3. **绿 ≠ 完整**——测试全绿只说明已声明的问题没回归；还剩多少没覆盖看本矩阵。

---

## 0. 坐标系：四个面 + 两条轴

矩阵分四面（沿用 pipeline §4）：

| 面 | 含义 | v0 分母来源 |
|---|---|---|
| **行为面/内容面** | 可观察行为与游戏内容单元（武器/炸弹/关卡/系统） | 本文件 §2，人工+证据建立 |
| **符号面** | 全部代码/RAM 符号（IDA/Ghidra 子程序全集、向量、RAM） | §3 占位：**S1 后由 IDA/Ghidra 机械导出** |
| **数据面** | 全部数据表 + xref（表边界、字段切分） | §4 占位：**S1 后由 IDA/Ghidra 机械导出** |
| **规则面** | docs/spec/ 行为规则条目（R-xxx）覆盖 | §5，由行为面派生 |

两条轴（**编号撞车，切勿混淆**）：

- **验证强度轴（矩阵轴）L0–L4**：一格的分子被验证到多强（§6）。
- **差分层（金字塔轴）L1–L5**：差分测试的层级（资产/子程序/帧/视觉/流程），定义在 pipeline §5，由 verifier 操作。
  本文出现的 L0–L4 **一律指验证强度轴**；提到金字塔时写"差分层 L1–L5"。

**格子 ID 规则**：`<面>-<对象>[-<子对象>]`，如 `B-WPN-03-LV2`（行为面-武器3号-2级）、`B-STG-05`、`B-SYS-SHOP`。
行为面前缀 `B`，符号面 `S`，数据面 `D`，规则面 `R`。格子一经入分母，ID 永不复用。

**分母 vs 分子**：
- **分母格**：一个需要被还原/移植/验证的内容单元（带证据指针，§1）。
- **分子登记**：对该格完成的验证记录（强度 L0–L4 + 证据指针绑定 scenario/trace，§6）。
  分母建立 ≠ 覆盖；覆盖只通过分子登记体现。

---

## 1. 证据指针登记（E-xx）与可复现命令

指针格式：`rom:<字节偏移>` / `web:<URL>` / `shot:<文件>` / `scenario:<id>`（S2 后启用）/ `trace:<id>`（S2 后启用）。
等级依 AGENTS.md §3：**A**=字节/像素/trace 硬证据；**B**=可复现静态推理；**C**=猜测（只进 §7）。

### 1.1 ROM 证据（A 级）

ROM：`rom/gunnac_cn.nes`（只读，永不修改）
sha256 = `4d76901c1b2455f2bdae8e8ac99b1779441f9f2155614dab222c229ae06c77ae`，
iNES 头 `4e 45 53 1a 08 10 41 00…`：PRG 8×16K=128K、CHR 16×8K=128K、mapper 4（MMC3）、垂直镜像——与 AGENTS.md §0 描述一致。
本 ROM 为**九班汉化版**（`rom:7106–7118` `\t@2011  JIUBAN`，标题画面含 `LICENSED BY NINTENDO OF AMERICA INC.`，即美版基底）。
按 AGENTS.md R4：**汉化特有行为单列，不混入原版行为结论**；下列 ASCII 串是 ROM 物理存在的字节（A 级），
是否在本版运行时实际显示，需 S2 动态佐证（已获像素佐证的单独标注）。

复现命令（任何一条均可在仓库根目录重跑）：

```bash
sha256sum rom/gunnac_cn.nes
grep -abo '<串>' rom/gunnac_cn.nes
python3 -c "d=open('rom/gunnac_cn.nes','rb').read(); print(d[<off>:<off>+N])"
```

| ID | 指针 | 内容 | 级 |
|---|---|---|---|
| E-R01 | `rom:4272` (0x10B0) | `HI SCORE` / `SCORE` 及标题版权串 `1990 @TOKYO SHOSEKI / PROGRAMED BY COMPILE / PRESENTED BY NEXOFT CORP. / LICENSED BY NINTENDO OF AMERICA INC.`（~4286–4394）；**像素佐证**：`shot:shots/auto_f00360.png` | A |
| E-R02 | `rom:4759` (0x1297) | `10000000000`（含义待考证 → H-03） | A |
| E-R03 | `rom:5419` (0x152B) | `WEAPON-0`；`rom:5430 (0x1536)` `BOMBER-0` —— 等级/槽位显示标签（尾位数字疑为运行时改写 → H-04） | A |
| E-R04 | `rom:8516–8523` (0x2144–0x214B) | `-------$` 金钱显示串（破折号+货币符，`$` 位于 8523）；**像素佐证**：`shot:shots/auto_f01080.png` 中 `你有 ──￥` | A |
| E-R05 | `rom:8914–9000` | 编号价目记录区：4 条编号记录（ASCII `1.`/`2.`/`3.`/`4.` + 图形码商品名）各后随一条价格串，模板 `1 <图形码> $<n>`：`$10`@8929、`$6`@8958、`$4`@8986；4 号记录后未见 ASCII 价格串（@8993 附近即被 `ff` 填充截断） | A |
| E-R06 | `rom:9095` (0x2387) | 平行串记录 `1 2 3 4 5 `（5 个数字） | A |
| E-R07 | `rom:9534` (0x253E) | 平行串记录 `1 2 3 4 5 6 7 8`（8 个数字） | A |
| E-R08 | `rom:9626` (0x259A) | 平行串记录 `F B T W`（4 个字母） | A |
| E-R09 | `rom:12036` (0x2F04) | 开场文本 `HIS MISSION IS TO / INVESTIGATE THE EIGHT AREAS / AND RESTORE PEACE`；`rom:11879 SEVEN PLANETS`；`rom:12132 A POWERFUL INVADER IN ONE / rom:12160 OF THE EIGHT AREAS` | A |
| E-R10 | `rom:12278` (0x2FF6) | 结局文本 `THANKS TO GUN-NAC, THE INVADER IS DEFEATED…`；`rom:12264 (0x2FE8) OPTION DEMO`（标题菜单串） | A |
| E-R11 | `rom:124126` (0x1E4DE) `CONTINUE`；`rom:124137` (0x1E4E9) `GAME OVER` | A |
| E-R12 | `rom:7106–7118` (0x1BC2) | 汉化标记 `\t@2011  JIUBAN`（九班；`2011`@7107、`JIUBAN`@7113） | A |

> 结构观察（B 级）：E-R05/E-R06/E-R07/E-R08 同处 0x2200–0x2700 文本记录区，四组记录前缀结构一致
> （`\xfe\xe9"` 起、`F#A…,B…` 键位提示结尾），相互平行 ⇒ 疑为同一组商店/说明面板文本（价目、武器编号、关卡编号、炸弹字母）。
> 记录布局的字段切分归 S1 数据面（H-05）。

### 1.2 外部资料证据（Wikipedia，B 级——外部叙述性资料，须由 ROM/trace 复核后才可作 spec 结论）

来源：[en.wikipedia.org/wiki/Gun-Nac](https://en.wikipedia.org/wiki/Gun-Nac)（2026-10-12 抓取）。引文为原文摘录：

| ID | 原文摘录 | 支撑 |
|---|---|---|
| E-W01 | "throughout **eight different levels**, fights a host of enemies from a space ship" | 8 关 |
| E-W02 | "The game contains **five primary weapons**, as well as **four types of secondary bombs**. Examples of primary weapons are a standard gun, a flamethrower and a type of guided boomerang. All weapons have unlimited ammunition, and they are obtained by collecting **circles with numbers** that correspond to one of the five different weapons." | 5 主武器 + 4 炸弹 + 获取方式 |
| E-W03 | "The primary weapons can be upgraded by collecting multiple '**Power P's**' …" / "When a player has acquired a particular type of weapon, he can upgrade weapon level by collecting **several of the same circles in a row**. Each time another matching circle is collected, the weapon is upgraded. This is **limited to five upgrades**." | 武器 5 级；两种升级说法并存 → H-02 |
| E-W04 | "The secondary bombs work on a similar principle; they are obtained by collecting **circles with corresponding letters**… The bombs… are **limited in supply**" / "When a bomb is upgraded, it becomes **twice as powerful**, but also consumes twice as much ammunition." | 炸弹字母圈 + 弹药上限 + 升级翻倍 |
| E-W05 | "The ship can be upgraded to a larger ship by acquiring a '**wing**', which allows the player to **sustain two enemy hits rather than one**, and allows the player to **upgrade his weapons further** than the standard ship would allow." | 翼装道具效果 |
| E-W06 | "Money can be found **randomly floating** through the levels, or being **released by destroyed objects or enemies**; it can be used to purchase upgrades in a **shop in between levels**." | 金钱/掉落/商店 |
| E-W07 | "The American release had a number of things removed or changed, such as **the first phase of the last boss**, the intro and the ending." | 最终 Boss 多相 + 美版删改 → H-06 |

### 1.3 运行截图证据（A 级，Mesen2 自动捕获，`tools/mesen_capture.lua` 产线）

| ID | 指针（sha256） | 可见内容 |
|---|---|---|
| E-S01a | `shot:shots/auto_f00360.png`（8b72d2c0…） | 标题画面：版权文字（与 E-R01 逐字吻合）+ `加纳战机` 字样 |
| E-S01b | `shot:shots/auto_f00540.png`（4e8021cd…） | 标题画面（彩色 logo 态） |
| E-S01c | `shot:shots/auto_f00720.png` / `auto_f00900.png` | 标题菜单：`开始 / 设置`（对应 E-R10 `OPTION DEMO`） |
| E-S02 | `shot:shots/auto_f01080.png`…`final_f05400.png`（26 张同帧，efcfe736…） | 商店界面：柜台+店员+货架背景；`你有 ──￥`（↔E-R04 金钱显示）；`收集相同子弹，子弹级增加`（↔E-W02/E-W03 升级规则的像素佐证）；`…键，B键 转换`（按键提示） |
| E-S03 | `shot:shots/auto_f00180.png`（788d6984…） | 开机初始帧（黑场） |

> S2 建立后，scenario/trace 证据以 `scenario:oracle/scenarios/<id>`、`trace:analysis/traces/<id>` 形式追加进本表。

---

## 2. 行为面/内容面分母（v0 已建立）

> 格子入分母的唯一通道：本表新增行 + 证据列非空。无证据的需求描述一律进 §7 假设区或"待取证"清单（§2.6）。

### 2.1 主武器 × 等级（B-WPN-<槽>-<级>，5 槽 × 5 级 = 25 格）

- 槽位轴证据：E-R06（ROM 平行串 `1 2 3 4 5 `）+ E-W02（five primary weapons / numbered circles）。
- 等级轴证据：E-W03（limited to five upgrades）+ E-R06（5 数字）+ E-S02（`收集相同子弹，子弹级增加` 像素佐证）。
- 槽位语义名（标准枪/火焰/回旋镖等）与槽号的对应关系**未取得证据** → H-01，表中槽位一律以编号称呼。

| 格 ID | 内容单元 | 证据指针 | 级 | 分子 |
|---|---|---|---|---|
| B-WPN-01-LV1 | 武器槽#1 · 等级 1 | E-R06, E-R03, E-W02, E-W03, E-S02 | A/B | L0 |
| B-WPN-01-LV2 | 武器槽#1 · 等级 2 | 同上 | A/B | L0 |
| B-WPN-01-LV3 | 武器槽#1 · 等级 3 | 同上 | A/B | L0 |
| B-WPN-01-LV4 | 武器槽#1 · 等级 4 | 同上 | A/B | L0 |
| B-WPN-01-LV5 | 武器槽#1 · 等级 5 | 同上 | A/B | L0 |
| B-WPN-02-LV1 | 武器槽#2 · 等级 1 | E-R06, E-R03, E-W02, E-W03, E-S02 | A/B | L0 |
| B-WPN-02-LV2 | 武器槽#2 · 等级 2 | 同上 | A/B | L0 |
| B-WPN-02-LV3 | 武器槽#2 · 等级 3 | 同上 | A/B | L0 |
| B-WPN-02-LV4 | 武器槽#2 · 等级 4 | 同上 | A/B | L0 |
| B-WPN-02-LV5 | 武器槽#2 · 等级 5 | 同上 | A/B | L0 |
| B-WPN-03-LV1 | 武器槽#3 · 等级 1 | E-R06, E-R03, E-W02, E-W03, E-S02 | A/B | L0 |
| B-WPN-03-LV2 | 武器槽#3 · 等级 2 | 同上 | A/B | L0 |
| B-WPN-03-LV3 | 武器槽#3 · 等级 3 | 同上 | A/B | L0 |
| B-WPN-03-LV4 | 武器槽#3 · 等级 4 | 同上 | A/B | L0 |
| B-WPN-03-LV5 | 武器槽#3 · 等级 5 | 同上 | A/B | L0 |
| B-WPN-04-LV1 | 武器槽#4 · 等级 1 | E-R06, E-R03, E-W02, E-W03, E-S02 | A/B | L0 |
| B-WPN-04-LV2 | 武器槽#4 · 等级 2 | 同上 | A/B | L0 |
| B-WPN-04-LV3 | 武器槽#4 · 等级 3 | 同上 | A/B | L0 |
| B-WPN-04-LV4 | 武器槽#4 · 等级 4 | 同上 | A/B | L0 |
| B-WPN-04-LV5 | 武器槽#4 · 等级 5 | 同上 | A/B | L0 |
| B-WPN-05-LV1 | 武器槽#5 · 等级 1 | E-R06, E-R03, E-W02, E-W03, E-S02 | A/B | L0 |
| B-WPN-05-LV2 | 武器槽#5 · 等级 2 | 同上 | A/B | L0 |
| B-WPN-05-LV3 | 武器槽#5 · 等级 3 | 同上 | A/B | L0 |
| B-WPN-05-LV4 | 武器槽#5 · 等级 4 | 同上 | A/B | L0 |
| B-WPN-05-LV5 | 武器槽#5 · 等级 5 | 同上 | A/B | L0 |

> 【2026-10-12 t56 控制方案备注】25 格 B-WPN-xx-LVx 仍 L0，但**语义基础已变**（t52 F1）：
> 主武器触发=B 键长按连发（非自动连射）、弹种切换=拾取字母圈——逐槽逐级验证设计（射击时机断言、
> 升级观测路径）须按新控制方案编排；级配制语义（R-shot-level）不受影响；【t75 同步】射击时机语义补正：`$072E` 在途计数（+1/−1 成对）+`$074A` 在途上限（N-0093，"周期表"作废）。

> 级轴计数备注：`WEAPON-0`（E-R03）提示运行时显示可能 0 起算；**分母按 5 档计**（任务卡与 E-W03 "five upgrades" 一致），
> "0..4 还是 1..5"的显示映射属实现细节考证 → H-04。

### 2.2 炸弹（BMB-<字母>，4 格）

字母轴证据：E-R08（ROM 平行串 `F B T W`）+ E-W02（four types of secondary bombs）+ E-W04（letters 圈获取）。

| 格 ID | 内容单元 | 证据指针 | 级 | 分子 |
|---|---|---|---|---|
| B-BMB-F | 炸弹"F"（语义名待证 → H-01）：获取/存量上限/发射/威力/升级翻倍 | E-R08, E-W02, E-W04 | A/B | L0 |
| B-BMB-B | 炸弹"B"（同上） | E-R08, E-W02, E-W04 | A/B | L0 |
| B-BMB-T | 炸弹"T"（同上） | E-R08, E-W02, E-W04 | A/B | L0 |
| B-BMB-W | 炸弹"W"（同上） | E-R08, E-W02, E-W04 | A/B | L0 |

> 升级语义（威力×2 / 耗弹×2）来自 E-W04，属 B 级叙述性资料；入 spec 结论前须 ROM/trace 复核（§6 强度规则）。

### 2.3 关卡（B-STG-01..08，8 格）

关数轴证据：E-R09（`INVESTIGATE THE EIGHT AREAS` / `OF THE EIGHT AREAS` / `SEVEN PLANETS`）+ E-R07（平行串 `1 2 3 4 5 6 7 8`）+ E-W01（eight different levels）+ E-R10（结局文本，通关归宿）。

| 格 ID | 内容单元 | 证据指针 | 级 | 分子 |
|---|---|---|---|---|
| B-STG-01 | 第 1 区域：进场/时间线/敌人/地形/出口（子格见下） | E-R09, E-R07, E-W01 | A/B | L0 |
| B-STG-02 | 第 2 区域（同上） | 同上 | A/B | L0 |
| B-STG-03 | 第 3 区域（同上） | 同上 | A/B | L0 |
| B-STG-04 | 第 4 区域（同上） | 同上 | A/B | L0 |
| B-STG-05 | 第 5 区域（同上） | 同上 | A/B | L0 |
| B-STG-06 | 第 6 区域（同上） | 同上 | A/B | L0 |
| B-STG-07 | 第 7 区域（同上） | 同上 | A/B | L0 |
| B-STG-08 | 第 8 区域（同上；疑含最终 Boss → H-06） | 同上 + E-W07 + E-R09（`A POWERFUL INVADER IN ONE OF THE EIGHT AREAS`） | A/B | L0 |

#### 2.3.1 关卡子格登记（按证据准入，2026-10-12 t46 首格）

| 格 ID | 内容单元 | 证据指针 | 级 | 分子 |
|---|---|---|---|---|
| B-STG-01-TL | STG-01 滚动时间线：波次链结构 / 滚动指令嵌入 / bank 图归属 / 目录顺序扫入口 / **BG 时间线** | stage-timeline.md §1-5/§2.6 + wave-semantics-s1.md（记录格式 A 级）+ bg-content-line.md §4（BG 时间线 A 级）, gameplay-stage1 `$1C/$1D` 逐帧向量 | B/A | **L3 已判**（t70 升格（captain 裁定）：R-bg-scroll-1..4 帧级确定性值序列零未证依赖=标准 L3 断言；注：波次链帧级对齐 S1 负结论维持（t47），两断言面分离） |
| B-STG-01-EN | STG-01 敌群：内容层 8 记录 + 目录入口 + 1:1 + **时刻层（t85 升格）**——✅已判=内容层+时刻层（tick 86 点/间隔三档 8·10·15f/事件分布）；⚠残余=W28′ 逐块公式/参数 W26/自指 W27；~~EN-9+~~ 同格式延续（t83 收口）；❌其余 7 关展开 | b-stg-01-en.md + w28-boss.md（86 tick 帧表+6 段域+分布 A 级）, gameplay-stage1 正寻址 | A/B | **L3 已判**（t85 升格（captain 裁定）：tick 帧表+间隔档=观测级确定性断言面，与 R-bg-scroll 同标准；R-stage-tick-1..3） |
| B-STG-02-EN | STG-2 敌群：✅调度记录 242/14+脚本集 {A39B}+共享族归属；⚠tick 参数（子流 W29）；❌内容 5B 逐条 | stg02-08-en.md §3（$97 解析 rec4/rec6 计数） | B | **L2**（t88 约束①；覆盖只记已判） |
| B-STG-03-EN | STG-3 敌群：✅记录 29/13+**CNT tick 表 7 段**+链段族 8 脚本；⚠间隔换算（W28′）；❌逐条语义 | stg02-08-en.md §3（CNT=16,16,8,16,8,4,16） | B | **L2**（t88；覆盖只记已判） |
| B-STG-04-EN | STG-4 敌群：✅记录 214/38+**BOSS 族 16 脚本**（$A4xx）；⚠tick 参数；❌Boss 触发 | stg02-08-en.md §3 | B | **L2**（t88；覆盖只记已判） |
| B-STG-05-EN | STG-5 敌群：✅记录 219/41+**EN 族 13 脚本**（最全）；⚠tick 参数；❌逐条语义 | stg02-08-en.md §3 | B | **L2**（t88；覆盖只记已判） |
| B-STG-06-EN | STG-6 敌群：✅记录 250/10+脚本 {B4E6}；—；❌tick、语义 | stg02-08-en.md §3 | B | **L2**（t88；覆盖只记已判） |
| B-STG-07-EN | STG-7 敌群：✅与 STG-1 **共享流**（内容同，N-0108）；—；❌差异化内容（如有） | stg02-08-en.md §3 + N-0108 | B | **L2**（t88；覆盖只记已判） |
| B-STG-08-EN | STG-8 敌群：✅记录 52/19+**CNT tick 表 7 段**（同 STG-3）；⚠间隔换算（W28′）；❌逐条语义 | stg02-08-en.md §3 | B | **L2**（t88；覆盖只记已判） |
| B-STG-01-BOSS | STG-01 BOSS 生成链：**三分区更新（t91）**——✅链定位/脚本格式/EN 族分立/**出场触发**（VM 流推进+$865C 生成链）/**巡逻语义**（R-boss-patrol-1/2/3 升格）；⚠条数展开/W32 实体识别/W30 flow 对齐；❌逐条语义 | w28-boss.md + boss-trigger-patrol.md §1-2（$865C 九字段+$A41F/$A435/$A486 巡逻实录）, N-0109/0110 | A/B | **L2/部分**（整格维持不升：巡逻子项已升（t91 captain 裁定）；R-boss-patrol-1..3+R-stage-tick-4） |

> 其余 TL/EN/BOSS 子格按下方规则**证据后准入，不预建**；**BOSS 子格预登记待证（下批）**【t82 注】。
> 【t74 优先级注】~~下一格候选首选 W21（$AC80 流格式定谳）~~ →【**2026-10-12 t78 解锁（captain 裁定）**】
> **B-STG-xx-EN/BOSS 子格准入解锁**：格式族已定谳（`$97` 解释器 4B/6B 记录流，N-0096..0098）+
> **替代路径 = 以 VM 指令流为规格对象**（`$AC80` 字节对齐**不阻塞**，残余=W21′ 字节级归属）——
> 各子格仍按约束①**逐格证据准入**，解锁的是"**取证路径可用**"；W15 触发例程仍为 H-16 独立缺口。

**关卡子格登记规则**（约束①的具体化，防猜测入分母）：每关的
`B-STG-xx-TL`（滚动时间线）、`B-STG-xx-EN`（敌人生成/波次）、`B-STG-xx-BOSS`（关底 Boss）
**不预先入分母**；由 S2 场景捕获或 S1 表识别取得证据后，按 §8 约束①声明覆盖面、当场入分母。
各关的主题/名称（动物、工厂等）目前仅开场文本有暗示（`ANIMALS, PLANTS AND PRODUCTS FROM THE FACTORIES`，E-R09），
属 C 级 → H-07，**不入分母**。

### 2.4 系统（B-SYS-<名>）

| 格 ID | 内容单元 | 证据指针 | 级 | 分子 |
|---|---|---|---|---|
| B-SYS-SHOP | 商店系统：进出时机（关间）、货架商品、价目、购买流程、按键切换 | E-W06, E-R05, E-R04, E-S02, + S2: shop/exp-shop-* | A/B | **L2**（R-shop-1..8，§2.4.2） |
| B-SYS-MONEY | 金钱系统：获取/显示/累计/上限/消费 | E-W06, E-R04, E-S02, E-R05, E-R02（`10000000000` 不解释 → H-03）, + S2: shop f1140, + t22: N-0028/0029, bank14:DBCE/DAE7/D009 | A/B | **L2**（R-money-1..6；消费侧仍空 H-09 残项） |
| B-SYS-ITEM | 道具系统：编号圈（换武器）、字母圈（换炸弹）、升级圈、翼装 wing（E-W05：多挨一击+武器可继续升级） | E-W02, E-W03, E-W04, E-W05, E-S02, + S2: shop f1140（导弹20上限）, + t22: N-0030（bomb_type_index 拾取三方同步） | A/B | **L2**（R-item-1、R-bomb-1..3；映射 1=B/2=T 推定 H-11 残项） |
| B-SYS-DROP | 掉落系统：关内随机浮动金钱、被毁对象/敌人掉钱、道具掉落 | E-W06, E-W02 | B | L0 |
| B-SYS-BOSS | Boss 系统：最终 Boss 多相（E-W07 "first phase of the last boss"）；各关 Boss 子格按 §2.3 规则登记 | E-W07, E-R09 | B | L0 |
| B-SYS-UI | 流程系统：标题/菜单（开始·设置）/DEMO/GAME OVER/CONTINUE/结局文本 | E-R01, E-R10, E-R11, E-S01, E-S03, + S2: title-menu/shop/gameplay-stage1 | A | **L2**（R-ui-1..8，§2.4.2） |
| B-SYS-SCORE | 计分/最高分系统（`HI SCORE`/`SCORE` 显示） | E-R01, E-S01 | A | L0 |
| B-SYS-MOVE | 自机移动：输入位图→位移 / X·Y 钳制 / 亚像素累积 / 速度对（2026-10-12 t23 按约束①准入） | N-0011..0016, bank03:A084-A0E8, ram-watch-s3 §1, scenario: gameplay-stage1 | A | **L2**（R-move-1..6，§2.4.3） |
| B-SYS-FIRE | 射击/投放：A 边沿投放例程流 / 消耗查表 / 坐标快照 / mode:=7 / 主武器弹药 | N-0017..0021/N-0024, bank03:A2B4-A2FA, ram-watch-s3 §3-4, scenario: gameplay-stage1 | A | **L2**（R-fire-1..6，§2.4.3） |
| B-SYS-RESPAWN | 死亡/重生事件：事件窗口 / 移动冻结 / 坐标重置 128/160 / 弹药回满 4（2026-10-12 t27 按约束①准入；受击判定/死亡语义不在此格，见 H-16/W3） | oracle/traces/gameplay-stage1.player.jsonl 帧域 f1831-1916/f2047-2101/f2131-2175, t24 L3 发现 | A | **L2**（R-respawn-1..4，§2.4.5） |
| B-SYS-SPAWNRHYTHM | spawn 节律：16 帧周期连发 / 突发簇 / 独立定时机制（2026-10-12 t49 约束①准入，t47 证据包） | wave-semantics-s1.md §2（104 事件 A 级）, gameplay-stage1 对象槽激活流（寻址=字段基址+n，t67/N-0086 更正） | A | **L2→L3 部分**（R-spawn-1..3，§2.4.10） |
| B-SYS-ENEMY | 敌对象系统：生成记录 / 行为脚本流 / 双 SoA 网格 / RAM 绑定流 / 生命周期已证链（2026-10-12 t60 约束①准入；命中/清除段=H-16 待证） | enemy-dataface.md §1-2（t_005 5 字节实录/双 SoA 字段扫描/t_001 375 指针）, t48/t52 弹侧定谳合流 | A/B | **L2**（R-enemy-1..6，§2.4.12） |

#### 2.4.1 分子登记记录（L1，规则面覆盖；t6 2026-10-12）

> 格式按 §6.2；强度封顶 L1（web: 证据不得超 L1）。交互边界（购买按键效果、金钱增减时机、菜单跳转）
> 已在两份规格的"待 S2 取证空位"留白，**未发明**。

```
格: B-SYS-SHOP
强度: L1
证据: rom:8914–9000（价目记录 $10@8929/$6@8958/$4@8986）、rom:8516–8523、rom:8570/8575 等面板尾部 A/B 字符、shot:shots/auto_f01080.png、web:E-W06
工件: docs/spec/shop-economy.md#R-shop-1..4、docs/spec/constants.md
日期: 2026-10-12
备注: H-05（记录布局）、4 号商品价格缺串

格: B-SYS-MONEY
强度: L1
证据: rom:8516–8523（C-money-template）、rom:4759（仅登记不解释）、rom:8929/8958/8986、shot:shots/auto_f01080.png、web:E-W06
工件: docs/spec/shop-economy.md#R-money-1..3、docs/spec/constants.md
日期: 2026-10-12
备注: H-03（10000000000 含义禁解释）

格: B-SYS-UI
强度: L1
证据: rom:4272/4283/4291/4313/4336/4364/4378/12264/12635/124126/124137/12278/12299/12036、shot:shots/auto_f00360.png/auto_f00540.png/auto_f00720.png/auto_f00900.png
工件: docs/spec/ui-flow.md#R-ui-1..5
日期: 2026-10-12
备注: H-06（美版删改差异未证）；菜单串映射未证
```

#### 2.4.2 分子登记记录（L2，S2 场景证据回填；t17 2026-10-12）

> 格式按 §6.2；L2 = 绑定可复现场景 + trace/截图（scenario:analysis/traces/<id> + 帧号）。
> 已证部分入规则，未闭环交互边界继续留空位 + 新 H（H-09..H-13），**未发明**。

```
格: B-SYS-SHOP
强度: L2
证据: scenario:analysis/traces/shop（f1080 shop-arrival sha efcfe736 / f1140 shop-page sha fec87b88）、scenario:analysis/traces/exp-shop-start（2000 帧无输入反证）、scenario:analysis/traces/exp-shop-b（frames 2000）、scenario:analysis/traces/gameplay-stage1（f1200 exit-transition / f1260 stage-start）、trace:analysis/traces/shop#f1080/f1140、trace:analysis/traces/gameplay-stage1#f1200/f1260
工件: docs/spec/shop-economy.md#R-shop-5..8
日期: 2026-10-12
备注: H-10（A 单击/购买确认效果）、H-13（进入/离开完整条件）

格: B-SYS-MONEY
强度: L2
证据: scenario:analysis/traces/shop（f1140：「你有 ──￥0」显示行数值位）、trace:analysis/traces/shop#f1140 + 原 L1 证据（rom:8516–8523 等）
工件: docs/spec/shop-economy.md#R-money-1..4
日期: 2026-10-12
备注: H-09（增减精确时机与数值未证）、H-03（10000000000 不解释）

格: B-SYS-ITEM
强度: L2
证据: scenario:analysis/traces/shop（f1140：「导弹20个 上限。」）+ scenario:analysis/traces/gameplay-stage1（HUD 计数随输入变化 f1260/f1500/f2349）、trace:analysis/traces/shop#f1140、trace:analysis/traces/gameplay-stage1#f1260/f1500/f2349
工件: docs/spec/shop-economy.md#R-item-1、docs/spec/ui-flow.md#R-ui-8
日期: 2026-10-12
备注: H-11（导弹↔F/B/T/W 映射）、H-12（HUD 语义留台账）

格: B-SYS-UI
强度: L2
证据: scenario:analysis/traces/title-menu（f180 boot-black→f360 title→f540-900 menu→f1079 end，1080 行）、scenario:analysis/traces/shop（1450 行）、scenario:analysis/traces/gameplay-stage1（2400 行：f1080→f1200→f1260→f1500→f2349）、trace:analysis/traces/title-menu#f180/f360/f540/f1079、trace:analysis/traces/gameplay-stage1#f1080/f1200/f1260/f1500/f2349
工件: docs/spec/ui-flow.md#R-ui-6..8
日期: 2026-10-12
备注: H-06（开场/结局显示条件）、H-12（HUD 语义）、H-13（序列中各输入独立作用未隔离）
```

#### 2.4.3 分子登记记录（L2，S4 行为规则第一批；t23 2026-10-12）

> 格式按 §6.2；L2 = 绑定可复现场景 + trace/RAM diff + bank:addr 双向证据（升 L3 待 Rust 侧差分）。
> 未证语义（红圈/等级/分工）继续留空（H-11/H-12/H-14/H-15），**未发明**。

```
格: B-SYS-MOVE
强度: L2
证据: scenario:analysis/traces/gameplay-stage1（6 段 hold + 60 帧 probe RAM）、trace:analysis/traces/gameplay-stage1#f1300-f2280（hold 段逐段位移）、bank03:A084-A0E8（移动例程）、data/naming/ledger.jsonl N-0011..N-0016
工件: docs/spec/player-movement.md#R-move-1..6、docs/spec/constants.md#C-vel-h/C-clamp-x
日期: 2026-10-12
备注: W6（速度查表 $A2FB/$A30B 分划）、W5（up 后 Y 回中）、H-15（$0401 值域）

格: B-SYS-FIRE
强度: L2
证据: scenario:analysis/traces/gameplay-stage1（press_every 12 A + press 1500 b）、trace:analysis/traces/gameplay-stage1#f2160/f2280（$018F 3→4 HUD 同步）、bank03:A2B4-A2FA（投放例程）、bank03:A336-A33D（消耗/类型表）、data/naming/ledger.jsonl N-0017..N-0021/N-0024
工件: docs/spec/player-movement.md#R-fire-1..6、docs/spec/constants.md#C-bomb-consume
日期: 2026-10-12
备注: H-11（类型表映射）、H-14（A 射击/投放分工）、H-15（$0401 值域）
```

#### 2.4.4 分子登记记录（L2 增补证据；t26 2026-10-12）

> 格式按 §6.2；t22 证据消化增补 B-SYS-MONEY/ITEM 的 bindings（强度仍 L2，仅证据扩充，覆盖只增）。

```
格: B-SYS-MONEY
强度: L2
证据: 原 L2 证据 + bank14:DBCE-DBFF（加法器）、bank14:DAE7-DAFC（阈值）、bank14:D009-D046（渲染）、data/naming/ledger.jsonl N-0028/0029/0031（N-0025/W1 证伪撤回留档）、trace:analysis/traces/gameplay-stage1#f2160/f2220/f2280/f2340（BCD 计数器逐帧值 4400/5700/6800/7200）
工件: docs/spec/shop-economy.md#R-money-1..6、docs/spec/constants.md#C-money-bcd/C-money-addend-tbl/C-money-cap/C-extend-threshold
日期: 2026-10-12
备注: H-09 残项（消费侧）、H-03 残项（达上限行为）；显示值 f2280=6800（并行读图错序教训 → knowledge gotcha-parallel-read-image-order-unreliable）

格: B-SYS-ITEM
强度: L2
证据: 原 L2 证据 + bank15:E138-E13D（bomb_type_index 初始化）、data/naming/ledger.jsonl N-0030、trace:analysis/traces/gameplay-stage1#f2220（拾『W』$0190 0→3/HUD F→W/$018F 3→4 三方同步）
工件: docs/spec/shop-economy.md#R-bomb-1..3、docs/spec/constants.md#C-bomb-type-index
日期: 2026-10-12
备注: H-11 残项（1=B/2=T 推定、上限映射）、H-12 残项（等级/残机/红圈 slot）
```

#### 2.4.5 分子登记记录（L2，死亡/重生事件规格化；t27 2026-10-12）

> 格式按 §6.2；受击判定（hit-test）= spec 未证域（H-16）**不入本格**；死亡语义（残机/结束条件）未证不写。

```
格: B-SYS-RESPAWN
强度: L2
证据: trace:oracle/traces/gameplay-stage1.player.jsonl#f1831-1916/f2047-2101/f2131-2175（事件遮蔽缺口 86/55/45 帧 + 边界状态）、trace:oracle/traces/gameplay-stage1.player.jsonl#f2102-2130（移动冻结直接证据）、trace:oracle/traces/gameplay-stage1.player.jsonl#f1917/f2176（复位锚点 128/160 双向）、shot:shots/gameplay-stage1_f01860..f02220_probe.png（画面佐证，t12 probe f1830 后分岔告警→数值以 trace 为准）、t24 任务输出（L3 试点发现：弹药回满 4）
工件: docs/spec/player-movement.md#R-respawn-1..4
日期: 2026-10-12
备注: W5 收口（=重生坐标重置）；H-16（hit-test 未证域，t25 oracle/scenarios/hit-test.txt 可重跑）；死亡语义（残机/结束条件）W3 未证不写
```

#### 2.4.6 分子登记记录（L2 增补证据，弹丸系统规格化；t36 2026-10-12）

> 格式按 §6.2；R-shot-* 挂 B-SYS-FIRE（证据扩充，覆盖只增）；残项 A1-A5/$925F 分支如实留空。

```
格: B-SYS-FIRE
强度: L2
证据: 原 L2 证据 + bank3:$A1FD-$A22B（六型生成循环）、bank3:$A1F5-A2B2（$34 分派）、bank2:$9343-$938D（等级配置装载）、bank2:$87B3-$8810（脚本机）、bank2:$88BB/$88CF/$890B/$892F（LUT/动作流/时长表，seed 逐字节）、bank2:$9457/$9463（级表/配置流）、data/naming/ledger.jsonl N-0035/N-0038..0048
工件: docs/spec/shot-system.md#R-shot-gen-1..4/R-shot-motion-1..3/R-shot-level-1..4、docs/spec/constants.md#弹丸六表
日期: 2026-10-12
备注: H-02 收口（升级=并行弹型数/频率）、H-14 收口（三线分工）；残项 A1-A5、$925F 分支（t35 在考）、齐射逐型参数、碰撞命中（H-16 同域）
```

#### 2.4.7 分子登记记录（L2 增补 + FA-1/FA-2 处置；t37 2026-10-12）

> 格式按 §6.2；FA-1 反向规则补齐（pk-* 用例获 R-item-2/3 承载）；FA-2 声明降级（R-respawn-2/3/4"已可判"→"部分"）。

```
格: B-SYS-ITEM
强度: L2
证据: 原 L2 证据 + L2 对拍用例 pk-1..pk-7（oracle/subroutines/l2_cases.jsonl，ROM 子程序 vs game-core::weapons 双实现）+ bank2:$922B-$9258（同型 INC $33 回卷置 $0C / 异型 STA $34）
工件: docs/spec/shop-economy.md#R-item-1..3、data/status/coverage_map.json（pk-* 映射回填）
日期: 2026-10-12
备注: FA-1 闭环（pk-* 7 用例已获规则承载，此前不得记 R-item-1/R-bomb-1 覆盖的纪律解除）；深路径 pk-8/9（$925F）挂 R-shot-level-4 待证

格: B-SYS-RESPAWN
强度: L2
证据: 证据不变（§2.4.5）；声明降级：R-respawn-2/3/4 可判定性"L3 已可判"→"L3 部分"（窗口遮蔽 + 状态向量无弹药字段，当前零断言）
工件: docs/spec/player-movement.md#R-respawn-1..4（FA-2 降级注 + 真覆盖需求尾注）
日期: 2026-10-12
备注: FA-2 闭环（可判≠已判）；真覆盖需求=窄窗重生场景 + $018F/$04E0 watch（证据轨道变更→单独 commit 需求交 porter/analyst，未擅动）
```

#### 2.4.8 分子登记记录（L3 升格 + t39 规则同步；t44 2026-10-12）

> 格式按 §6.2；FA-2 真覆盖登记（t40 采证 → t44 登记）：R-respawn-2/3/4 升 **L3 已判**；
> 追加范围（t39 同步）：R-shot-level-3 公式勘误/A1 闭合/R-shot-level-4 转正。

```
格: B-SYS-RESPAWN
强度: L2（覆盖实质已达 L3 已判：三条规则断言均有逐帧向量实证）
证据: scenario:oracle/scenarios/respawn-window.txt（240 窄窗全采样，capture_every 10）+ trace:analysis/traces/respawn-window.frames.jsonl（逐帧 $018F/$04E0/$04FC + zp/iram/wram）+ docs/findings/respawn-window.md §3（E1 f1831-1915：冻结 85f→f1915 弹药:=4→f1916 坐标复位；E2 f2046-2174：冻结 129f→f2174 弹药:=4→f2175 坐标复位；时序定谳=冻结→N-1 弹药回满→N 坐标复位）
工件: docs/spec/player-movement.md#R-respawn-1..4（FA-2 降级注撤销、t27 分段遮蔽观察升 A 级）
日期: 2026-10-12
备注: FA-2 闭环（真覆盖需求两项由 t40 落齐）；残余=E1/E2 窗长差异（86 vs 129）→ H-16 续问、死亡语义 W3

格: B-SYS-FIRE
强度: L2
证据: 原 L2/§2.4.6 证据 + t39 同步：ha-1..4（索引变换 ROM 实证 级 12→索引 4/X=26）+ pk-10/11/12（deep 溢出/不加级/回 kernel）L2 三方全等 63/63
工件: docs/spec/shot-system.md#R-shot-level-3（公式勘误 4+((级-4)&7)、值域 [0,11] 无越界）/R-shot-level-4（转正）
日期: 2026-10-12
备注: A1 闭合（越界疑虑随公式勘误消解）；$018A 奖励链数值实现属移植面挂账
```

#### 2.4.9 分子登记记录（L2，关卡子格首格；t46 2026-10-12）

> 格式按 §6.2；B-STG-01-TL 按 §2.3 子格规则证据准入（首个关卡子格）；覆盖实质=结构级，帧级对齐待 S1。

```
格: B-STG-01-TL
强度: L2
证据: bank8:$AD00-$ADD0（波次链分段记录流，00 FE 分隔）、bank8:$ACEF（块范围对）、bank8:$AE87（滚动写点 STA $1C）、bank9/10 头同族（STG-02/03 候选定位）、scenario:analysis/traces/gameplay-stage1（对象 churn 1090/1090 帧全程活跃）、docs/findings/stage-timeline.md §1-5
工件: docs/spec/stage-timeline.md#R-stage-1..6、docs/spec/stage-timeline.md#R-stage-chain-1..3（t78 四层链架构）、docs/spec/stage-timeline.md#R-bg-scroll-1..4（t70 升格依据）、docs/spec/constants.md#C-wave-chain/C-wave-dir/C-scroll-vars
日期: 2026-10-12
备注: 【t49 更新】t47 L3 升格否定（波次链帧级对齐）；【**2026-10-12 t70 升格**】BG 时间线 R-bg-scroll-1..4 A 级入档（bg-content-line §4）→ **STG-01-TL 升 L3 已判**（captain 裁定：帧级确定性值序列标准 L3 断言形态）；波次链帧级对齐 S1 负结论维持（两断言面分离）；S2 收口（顺序扫入口）；其余子格不预建
```

#### 2.4.10 分子登记记录（L2→L3 部分，spawn 节律试探格；t49 2026-10-12）

> 格式按 §6.2；B-SYS-SPAWNRHYTHM 按约束①准入（t47 证据包）；L3 完整升格待 W3（定时机制定位）。

```
格: B-SYS-SPAWNRHYTHM
强度: L2（覆盖实质：事件序列可断言；帧级节律 L3 部分——16 帧周期/突发簇帧点可断言，定时机制未定位）
证据: trace:analysis/traces/gameplay-stage1.frames.jsonl（对象槽激活事件 104 个；测址口径 t67/N-0086 更正为字段基址+n，帧表素材保留待重采复核）+ docs/findings/wave-semantics-s1.md §2（16 帧周期三段 f1471-1551/f1655-1731/f1804-1900；突发簇 f1301 +3/f1787-91 +7/f1916 +8 与重生复位同帧）
工件: docs/spec/stage-timeline.md#R-spawn-1..3
日期: 2026-10-12
备注: W3（16 帧计时器 var/例程未定位）= L3 完整升格前置；f1916 突发与 B-SYS-RESPAWN E1 复位同帧交叉（归因未证）
```

#### 2.4.11 分子登记记录（L2→L3 前提标注，槽位分配建档；t51 2026-10-12）

> 格式按 §6.2；R-shot-alloc-* 建档 + R-shot-gen-1..3 升格标注：~~L3（前提 t50 golden 转正，落地后改已判）~~
> **【2026-10-12 t57 升级登记】L3 已判**（t55 golden 转正落地 commit 5baa6a8：三断言 L3-DIFF 2559 项全绿、负例归零）；
> t45 槽位几何未证注记撤销（t48 定谳）；f2350 路由信号结案=移植错（t55，规则无需改）。

```
格: B-SYS-FIRE
强度: L2
证据: 原 L2/§2.4.6 证据 + t48 定谳：docs/findings/shot-slot-geometry.md（槽位几何 4 组×5 槽/组距 $1C、分配器 bank2:$888D 首次适应、空槽=state==0、回收=寿命归零、满载 SEC Y=A=$F0）+ L2 用例 shot-alloc-1..4 三方全等（~~churn 9≤20 对照~~ 已随 t50 F2 证伪撤除：20 槽 state 系 B5 交错区静态数据）
工件: docs/spec/shot-system.md#R-shot-alloc-1..4（L2 已实证→L3 前提 t50）、#R-shot-gen-1..3（L3 前提标注）、oracle/traces/README.md（t45 注记撤销）
日期: 2026-10-12
备注: L3 升格前提=t50 shots golden 转正（--shots-extract 入默认抽取链）；Ws-6（$05CE 写点）留抽取器六字段补证
```

#### 2.4.12 分子登记记录（L2，enemy 模块合同开张；t60 2026-10-12）

> 格式按 §6.2；B-SYS-ENEMY 约束①准入（t32 证据包 + t48/t52 弹侧定谳合流）；
> 命中/受击判定维持 H-16 待证（t60 脚本机考证在途可分批回填）。

```
格: B-SYS-ENEMY
强度: L2
证据: docs/findings/enemy-dataface.md §1-2（t_005 5 字节记录实录/行为脚本流 $B2B4 族/双 SoA 字段阵列 raw 扫描/t_001 变长绑定流 375 指针）+ t48（$70 槽位几何）+ t52（槽迭代 +$7E/+$1C×3 交错）+ N-0011..0016/N-0054 锚点 + gameplay-stage1 对象 churn 1090/1090 帧
工件: docs/spec/enemy-system.md#R-enemy-1..6、docs/spec/constants.md#C-spawn-rec/C-grid-1c/C-grid-70
日期: 2026-10-12
备注: 命中/清除段待证（H-16）；HP 字段四候选未定谳（B4）；B-STG-xx-EN 逐关子格仍证据后准入不预建
```

#### 2.4.13 分子登记记录（L2 增补：VM 指令集回填；t61 2026-10-12）

> 格式按 §6.2；R-enemy-2 语义回填（t59 VM 定谳）+ R-enemy-7/8 新增；N-0078 位置更正联动。

```
格: B-SYS-ENEMY
强度: L2
证据: 原 L2/§2.4.12 证据 + t59：docs/findings/enemy-script-vm.md §1-2（VM 本体 bank00:$844A-$8584 指令集七类逐码 + 槽字段 $A4/$B0/$B6/$BC/$C2/$AA/流 PC $0197/$019D/输出表 $0150 + sub_8619 执行器 + 记录装载机 bank00:$83F6-$8432 四字节格式）+ N-0075/0076/0077（N-0078 位置更正：bank2"解释器"标签证伪）
工件: docs/spec/enemy-system.md#R-enemy-2（VM 逐码表）/#R-enemy-7/#R-enemy-8、#R-enemy-2a（两层模型注）
日期: 2026-10-12
备注: 两层模型（生成层 VM 槽 0-7 vs 更新层 $04xx 28 槽）入注；W12 数据流桥待证；命中段 H-16 维持待证；t_005 五字节 vs 装载机四字节分流（W11）【t63 勘误同步】记录装载机 X 坐标=p0−$10（原记 $0F 系进位疏漏，en-ld-1 实证）；$FC=界门非钳制；R-enemy-4 钳制断言限移动写路径（f1304/f1368 界外放置实测）
```

#### 2.4.14 分子登记记录（L2，B-STG-01-EN 三分区；t82 2026-10-12）

> 格式按 §6.2；约束①三分区照录（覆盖只记已判部分）；BOSS 子格预登记待证（下批）。

```
格: B-STG-01-EN
强度: L2
证据: ✅已判：bank8:$ADD0-$ADF7（8×5B 生成记录：X=175/123/139/119/119/117/117/128、4 种 VM 行为脚本 $B3E6/$B2B4/$B235/$B21D）+ bank8:$A000 关卡目录（8B×8，A 级新发现）+ 调度层 $AD44-$ADBD 8 段↔记录 8 条 1:1 对齐复核；⚠部分：时刻层（W28 $FD 计数→帧换算）/p1·p2 参数语义（W26）/STG-1 root 自指（W27）；❌未覆盖：BOSS 子格/EN-9+ 子表族（$ADF8+）/其余 7 关展开（入口已备）
工件: docs/spec/stage-timeline.md#R-stage-dir-1..2、#R-stage-chain-3（EN 联动注）、#R-stage-tick-1..3（t85 时刻层升格依据）、docs/findings/b-stg-01-en.md §4
日期: 2026-10-12
备注: 【t85 更新】时刻层升 **L3 已判**（captain 裁定：tick 帧表+间隔三档=观测级断言面）；EN-9+ 收口=同格式延续；残余 W28′/W26/W27（N-0105）；BOSS 子格→B-STG-01-BOSS（t85 入册）
```

#### 2.4.15 分子登记记录（L2/部分，BOSS 三分区；t85 2026-10-12）

> 格式按 §6.2；约束①三分区照录，**不升**（巡逻形态=候选不发明）。

```
格: B-STG-01-BOSS
强度: L2/部分
证据: ✅链定位（STG-1 目录尾指针 $A16F→5B 记录族 30 指针→$A4xx-$A6xx VM 字节码脚本族）/脚本格式（自环+$FC X 调整）/与 EN 族分立（$B2xx vs $A4xx）；⚠条数展开/实体边界未分；❌出场触发/逐条语义未证（w28-boss.md §2 + N-0103）
工件: docs/spec/stage-timeline.md#R-stage-boss-1（预规则，候选标注）
日期: 2026-10-12
备注: 【t91 更新】~~巡逻形态候选~~ → **巡逻语义三条已升**（R-boss-patrol-1/2/3，A 级：FC ±v 往复/FE 自环驻留/CALL 运动原语）；~~出场触发下批~~ → **已定谳**（R-stage-tick-4：VM 流推进+波次 END+$865C 九字段生成链，N-0109）；❌触发区→✅；余 W32 实体识别/W30 flow 对齐/W31 子脚本二读
```

#### 2.4.16 分子登记记录（L2 批量：B-STG-02..08-EN 三分区；t88 2026-10-12）

> 格式按 §6.2（批量七格压缩块）；约束①覆盖只记已判；证据包=`docs/findings/stg02-08-en.md` §3。

```
格: B-STG-02-EN / B-STG-03-EN / B-STG-04-EN / B-STG-05-EN / B-STG-06-EN / B-STG-07-EN / B-STG-08-EN
强度: L2（各格覆盖只记已判部分：三级骨架/记录计数/tick 表）
证据: $97 格式逐关解析（rec4/rec6 计数、CNT tick 参数、脚本指针集，N-0106）+ 目录表 bank8:$A000 8B×8（R-stage-dir-1）+ 5 脚本族普查（N-0107）+ STG-1/7 共享流（N-0108）；各关三分区明细=stg02-08-en.md §3 表
工件: docs/spec/stage-timeline.md#R-stage-dir-1..2（+脚本族归属注）、docs/findings/stg02-08-en.md §3
日期: 2026-10-12
备注: ⚠间隔档映射 W28′（STG-3/8 CNT 序列 16,16,8,16,8,4,16→15/8/4f 候选）/W29 子流 tick 参数；❌逐条语义未覆盖；W27 收口（共享配置池）残余 W27′ 边界
```

### 2.5 分母统计（v0）

| 面 | 格数 | 备注 |
|---|---|---|
| 行为面/内容面（§2） | 25 + 4 + 8 + 12 + 3 = **52** | 武器 25 / 炸弹 4 / 关卡 8 / 系统 12 / 关卡子格 3（t85 +B-STG-01-BOSS）；其余子格不预建 |
| 规则面（§5） | 46（下限，按行为格派生） | 每格 ≥1 条 R- 规则 |
| 符号面（§3） | **未计**（占位） | S1 后机械导出 |
| 数据面（§4） | **未计**（占位） | S1 后机械导出 |

### 2.6 待取证清单（明确**不在**分母，取证后按约束①准入）

| 候选格 | 缺什么证据 |
|---|---|
| 判定点/受击盒 | 移动/射击已按约束①准入（B-SYS-MOVE/B-SYS-FIRE，t23）；判定点数值仍缺——待 hitbox 观测场景 |
| 命数/死亡重生/CONTINUE 语义 | 仅 `CONTINUE` 串（E-R11），行为未观测（gameplay-stage1 f2349 为运行中止于战斗态，非 GAME OVER） |
| 音频（APU）行为 | 按 pipeline R5 先 scope-decision，不入行为面 |
| 各关主题/名称、各关 Boss | 见 §2.3 与 H-06/H-07 |
| 商店 4 号商品价格 | E-R05 记录区价格串缺失 → S2/S3 |

---

## 3. 符号面分母（S1 机械导出已建立，2026-10-12 t7）

> 由 IDA 9.1（M6502）逐 bank 建库分析机械导出，全集登记在
> `data/naming/remap-table.json`（机械层，pipeline §7，`tools/remap/run.py` 生成/校验）；
> 本表只登记**计数与覆盖率**，不登记语义名。冻结投影：`reference/seed/gunnac.asm`（91,601 行，sha256 `96b2c071…73721e`）。

| 分母项 | 计数 | 计数规则（机械） |
|---|---|---|
| 子程序 | **276** | IDA 入口（JSR/JMP 绝对目标 + 段起点）建函数，RTS/RTI 边界取 IDA 函数范围；f_000–f_275 机械名 |
| 中断向量 | **3** | NMI=$EF85、RESET=$E000、IRQ=$F20B（bank15 固定窗，file 0x2000A） |
| far-call 跳板调用点 | **18** | `JSR $E649`+内联(R6,R7,target)，12 个不同目标（$8000/$800C/$800F/$8012/$8807/$881A/$8DDF/$922B/$92C5/$95F9/$9698/$9BC3） |
| bank11 入口表 | **7** | bank11:$8000 起 JMP 表（$88A3/$88A7/$890B/$8016/$8894/$889B/$8E2C） |
| RAM 直接页符号 | **560** | 0x0000–0x07FF 操作数使用点（`v_XXX` 机械名 + 使用计数） |
| 映射寄存器符号 | **414** | 0x2000–0x401F 操作数使用点（PPU/APU/手柄寄存器，`r_XXXX`） |
| 字节覆盖 | **131,072/131,072** | code 74,436B + data 56,636B = PRG 全字节覆盖（线性扫描，无空洞） |

- Ghidra 12.1.4（`6502:LE:16:default`）analyzeHeadless 伪代码投影同步导出：`analysis/ghidra/bank00.c…bank15.c`（19,434 行 C，函数级反编译，可再生）。
- 与 S0 交叉核对（remap-table.json `cross_checks_s0`）：far-call 18 ✓、bank11 入口表 7 ✓、bank14/15 固定窗 ✓、向量 ✓ —— 全部 PASS。
- 遗留：跨 bank 代码 xref 机械层不计（经 far-call/指针间接），S3 考证时按 bank 窗口换算。

## 4. 数据面分母（S1 机械导出已建立，2026-10-12 t7）

> 全集 = 数据表 + xref；机械规则：**每个被 ≥2 个不同 intra-bank 代码 xref 索引的连续数据区段计一表**。

| 分母项 | 计数 | 说明 |
|---|---|---|
| 机械数据表 | **5** | t_000/t_001/t_005/t_006/t_007，见 `data/naming/remap-table.json` `data_tables`；【2026-10-12 更正（t18）】原 8 张经种子反汇编重分类：t_002/t_003/t_004 branch-align 84-97% ⇒ mixed_code（移出分母，`mixed_regions` 字段），t_007 边界修正 `$FD2A-$FFF9`（向量表移出） |
| 已知数据锚点候选 | **7**（不入分母） | charmap 表 ×2 + 文本记录区 ×5，登记于 `remap-table.json` `known_data_candidates` |
| CHR 资源 | **128 × 1KB bank** | 字模区 bank124–127（file 0x3F010–0x40010，汉字字模）；空 bank 42,43,47,51,64–71,80–83（S0 §5） |

- 文本记录格式（S0 已证实，纳入数据面候选）：`{PPU地址, tile码…, FE/FF}`，记录区：标题/版权 `PRG 0x0109E-0x01123`、剧情 `0x02CDC-0x02F8B`、结局 `0x02FE4-0x03155`、STAFF 字幕 `0x01959-0x01BC0`（含 `@2011 JIUBAN` 汉化签名）、大字模 `0x1FF00-0x1FFEF`。
- charmap/字形索引表候选：`PRG 0x01190-0x01220`、`PRG 0x01830-0x018F8`（含重排段，汉化文本映射线索）。
- 已知数据锚点（**不构成分母**，保留 v0 条目，均有 §1 证据指针）：
  - 商店价目记录区 `rom:8914–9000`（E-R05）；
  - 平行串记录区 `rom:9095/9534/9626`（E-R06/07/08）与文本记录区 `rom:8516–8523`（E-R04）；
  - 字符串/文本表区 `rom:0x1000–0x3100`（E-R01…E-R10）；
  - CHR 128K（iNES 头 CHR=16×8K；`_chr_bank0.png` 抽样渲染已跑通）。

## 5. 规则面分母（由行为面派生）

- 定义：对每个行为面格，`docs/spec/<module>.md` 中至少 1 条带证据指针的规则（R-xxx，配方见 skill/角色契约）才算该格有规格覆盖。
- v0 分母 = 44（行为格数 ×1，下限）；一格可挂多条规则（边界/异常路径按角色契约必须覆盖），但分母只升不降。
- 规则条目与格 ID 互相引用：`关联: B-WPN-03-LV2` 之类，便于反查。

---

## 6. 分子登记规则（验证强度 L0–L4 + 证据绑定）

### 6.1 强度定义（矩阵轴 L0–L4；≠ 差分金字塔 L1–L5）

| 强度 | 含义 | 必备证据 | 对应金字塔（参考） |
|---|---|---|---|
| **L0** | 已入分母，未验证 | （分母证据指针即可） | — |
| **L1** | 规格级：已有带 A/B 级证据指针的 spec 规则，规则可判定 | `bank:addr` / `rom:offset` / `shot:` / `web:`（须注明复核状态） | — |
| **L2** | 场景级：绑定可复现场景，Mesen2 跑通并留 trace/截图 | `scenario:oracle/scenarios/<id>` + `trace:analysis/traces/<id>` | 至少差分层 L3 有 oracle 侧轨迹 |
| **L3** | 差分级：Rust 侧与 oracle 在某差分层通过 | scenario/trace + 对应层门禁绿 + 基线登记 | 差分层 L1–L3 之一全绿 |
| **L4** | 收敛级：多层差分全绿且基线 ratchet 登记 | 上述 + `data/status/baselines/` 记录 + captain 验收 | 差分层 L1–L3 全绿；涉视觉再加 L4 白名单外一致；涉流程再加 L5 |

单调性：一格强度**只升不降**（降级=回退，需 commit 说明 + captain 验收，见 §8 约束②）。

### 6.2 分子登记记录（每格一条主记录，可追加）

```markdown
格: <cell-id>
强度: L0|L1|L2|L3|L4
证据: <指针列表：bank:addr / rom:offset / scenario:<id> / trace:<id> / shot:<file>>
工件: <产出路径：docs/spec/<module>.md#R-xxx | data/golden/… | data/status/baselines/…>
日期: YYYY-MM-DD
备注: <遗留假设引用 H-xx，没有则"无">
```

> ⚠️ **以上为格式示例，字段值是虚构占位**，不是任何真实格的验证结论；引用证据必须回到 §1 实际取证产物。

**绑定硬规则**：
- L2 起**必须**同时绑 `scenario:` 与 `trace:`（或差分层 L1 的 `data/golden/` 指针）；只有 `web:` 证据的格**封顶 L1**。
- 外部资料（E-Wxx）永远是 B 级：由 ROM/trace 复核过的结论升级为 A 级并在备注写复核指针，未复核的不得写入 spec 结论（AGENTS.md §3.1）。
- 汉化特有差异（E-R12 相关）单独登记，不与原版行为共用分子记录。

---

## 7. 假设区（C 级，**不入分母**、禁止 rename/入 spec 结论）

| ID | 假设 | 现状 | 取证路径 |
|---|---|---|---|
| H-01 | 炸弹字母语义（F/B/T/W 各代表什么）与 5 武器槽的语义名/编号对应（Wikipedia 只给了 standard gun / flamethrower / guided boomerang 三个例子且无编号） | C | S1 代码+道具图块交叉；S2 捡取场景 |
| H-02 | 武器升级机制两种说法并存（E-W03："Power P's" 与 "连收同型圈"）；可能一个是武器切换、一个是升级 | C | S2 拾取场景 trace |
| H-03 | `rom:4759 "10000000000"` 含义（金钱上限？分数上限？数字渲染常量？） | C | S1 数据表 + trace |
| H-04 | `WEAPON-0`/`BOMBER-0` 尾位数字是否运行时改写、等级显示 0 起算还是 1 起算 | C | S2 商店面板 trace（RAM 写点） |
| H-05 | 0x2200–0x2700 记录区的字段切分（价目/编号/字母平行记录的确切布局） | C（结构平行是 B 级观察） | S1 数据面机械导出 |
| H-06 | 美版删改（E-W07：最终 Boss 第一形态、intro、ending）与本汉化 ROM 的实际差异——注意 E-R09/E-R10 的 intro/ending 文本**确实存在于 ROM**，是否显示未证 | C | S2 全流程场景（intro/ending/Boss 相位） |
| H-07 | 各关主题（动物/植物/工厂等，E-R09 暗示）与关名 | C | S2 逐关场景 + S1 关卡表 |
| H-08 | 汉化版相对美版的行为差异（文本字模、断行、结算数值） | C | L4 视觉差分 + 文本表考证（AGENTS.md R4） |
| H-09 | 金钱增减的精确时机与数值——【2026-10-12 t26 更新】增加侧已证（事件加数表 type 1..10=10/30/100/150/250/400/600/800/1000/2000、逐位进位、封顶 999999，R-money-5）；**残项=消费侧（购买扣减）/死亡惩罚/过关结算** | C（残项） | S2 购买/结算场景 + RAM diff（$E0-$E6） |
| H-10 | 商店内 A 键单击与"购买确认"的效果（采样帧 trace f1180/f1340 留证但效果未定） | C | S2 单变量场景（隔离 A/B/Start） |
| H-11 | 「导弹」↔ 炸弹字母映射——【2026-10-12 t26 更新】bomb_type_index $0190 收口（0=F/3=W 实测 A 级，R-bomb-3）；**残项=1=B/2=T 推定（B 级）、20 上限对应弹种、其余弹种上限** | C（残项） | S2 单字母拾取场景实测 1=B/2=T |
| H-12 | 实机 HUD 语义——【2026-10-12 t26 更新】字母行归因**已收口**（= bomb_type_index $0190，f2220 拾「W」三方同步，R-bomb-2）；**残项=等级 var、残机 var（W3）、红圈掉落物 slot 偏移（W4）**，继续留命名台账 | C（残项） | t18 后续 RAM diff / object slot trace |
| H-13 | 商店进入/离开的完整条件（菜单序列各输入的独立作用、关间序列、GAME OVER 后路径） | C | S2 单变量场景矩阵 |
| H-14 | 控制方案分工——【2026-10-12 t36 收口】→【**t56 更正（t52 F1 冲击波）**】t36 定谳两处作废，正确：**A 边沿=投放**（R-fire-1）、**B 长按=主武器连发**（`$18` bit6→`$39/$3A`→`$32` bit2 门控链，R-shot-gen-2）、**弹种切换=拾取字母圈触发**（R-bomb-2/3）；~~主武器=计数器驱动自动连射/B=弹种切换~~（N-0073 撤回 N-0035 定性、N-0072 更正 t25）；齐射型由 `$34` 分派（R-shot-gen-4）维持 | ~~C~~ | 已收口（t52 定谳版为准，t36 文本作废留档） |
| H-15 | `game_action_mode`（$0401）除 0/7 外的值域语义（N-0024 同址多用途）——【2026-10-14 A5 收口】[game-action-mode-domain.md](../findings/game-action-mode-domain.md)：**"0/7 之外值域"问题不成立**——$0401 同址三义：gameplay=对象网格 slot1 状态字节（0=空闲可投/7=投放物在途，~~209f 寿命~~【2026-10-14 更正（复核 #57 AR57-6）】寿命=行为链合成值（相位链 27+180+2f，209f=chain-A 实测值非恒定，他链实测 254f，禁硬编码），N-0204 A 级）、shop=ui_select_cursor（N-0003）、scene=MMC3 CHR bank 影子 R1（**110/124=CHR 影子值，非枚举非位组合**，N-0205 A/B 级）；**残项=状态码域枚举（C）、CHR 1K/2K 单位语义（B）** | ~~C~~ | 已收口（写点全表+值域语义表+JP 对照，复现 R1-R5 见 findings）；残项另立 |
| H-16 | 受击判定（hit-test）本身未证：判定盒/触发条件/无敌窗口——【2026-10-12 t44 更新】事件内时序**已定谳**（t40：冻结→N-1 弹药回满→N 坐标复位）；**残项=E1/E2 窗长差异（86 vs 129）影响因子** + 判定盒/触发条件 | C（残项） | t25 `oracle/scenarios/hit-test.txt` 重跑 + 窗长差异对照（受击时刻/无敌帧/演出？）；死亡语义（残机/结束条件）同域未证（W3） |
| N-0037 | `_todo_slot_letter_binding`：HUD 行 1 字母位来源未定（letter-bank 或查表）——N-0032"字母=weapon_slot($34)"绑定已证伪撤回（f2220 分离事件：$34=3 而字母仍 'A'，位图 rows 40-51） | C | S2/S3 字形表查表考证 + 像素级绑定证明（pattern-visual-binding-needs-pixel-proof） |
| Q1 | `$34` 与 `$0190` 同写语义（f2220 拾『W』同写 3：弹/武器共用索引？） | C | S2 分化拾取场景 |
| Q2 | HUD 字母位来源（letter-bank 或查表；与 N-0037 同域） | C | S2/S3 |
| Q3 | N-0036 的 $0401 窗口时间线需在干净 run 重采（akey-role 列入防踩踏后重跑清单） | C | 下批 S2（串行锁下重跑） |
| Q4 | 注入 **+1 帧延迟**约定写入 `oracle/scenarios/README.md`（差分比对需按此对齐） | B | 建议 t12/t24 owner 补文档 |

### 7.1 假设索引（H / W / Q 三系列互通 —— 2026-10-12 t30 整理；**t41 刷新**）

> 本节为**集中索引**（主表）；各系列明细留在其登记文件，改状态须同步本表。

| 系列 | 明细位置 | 条目与状态 |
|---|---|---|
| **H**（矩阵假设） | 本节上表 + `docs/findings/upgrade-branch-h02.md` | **H-01/H-02 收口**（t25 两说合一 + t35 深路径定谳：同型/异型 × $018C==$FF 2×2 + $32 bit2 门控；kernel 回卷 $0C / deep 裸 INC 回卷 $00 **两路径语义分野**，范围收窄见 shot-behavior §1.2 更正 + N-0058）｜H-03..H-13、H-16 见上表｜**H-14 收口**（t36 三线分工定谳）｜**H-15 收口**（2026-10-14 A5：$0401 同址三义定谳，110/124=CHR 影子非值域，见 game-action-mode-domain.md + N-0204..0206）｜**H-A 新立**（t35）：$33 可越 12（deep 路径）→ `$9457` 越界索引行为 = 必答项 |
| **W**（RAM watch 假设） | `docs/findings/ram-watch-s3.md` §7/§8.4/§9.5 | W1 证伪（真源 $E0-$E6）｜W2 观察中｜W3 未闭环（等级/残机 _todo_）｜W4 未闭环（slot 偏移）｜**W5 收口**（=重生坐标重置）｜W6 B 级维持（速度查表分划）｜W7 → [gotcha-parallel-read-image-order-unreliable](../knowledge/gotcha-parallel-read-image-order-unreliable.md)｜**W8 证伪**（孤儿 run 干扰）→ [gotcha-concurrent-mesen-output-corruption](../knowledge/gotcha-concurrent-mesen-output-corruption.md) |
| **S**（t42 关卡结构假设） | `docs/findings/stage-timeline.md` §7 | S1 段参数语义（时刻/序号/掩码，C）｜S2 $ACEF 目录消费方式（C）｜S3 bank2 $8100 页目录读者（C）｜S4 地形碰撞数据（C）｜S5 关卡长度/切换条件（C）｜S6 bank9/10 逐关归属（B，候选定位级）——S 系列与 H/W/Q 并列互通，收口须同步本表 |
| **Wv**（wave 域，t51 消歧：原 W1-W5 → Wv-1..5） | `docs/findings/wave-semantics-s1.md` §7、`docs/findings/bg-content-line.md` | Wv-1 链首字用途/**时刻读**——**双重证伪**（t47 S1 拟合✗ + t69 滚动×波次链对齐负证：120f 脉冲 vs 波次簇无对应）｜Wv-2 `XX FD` 标记语义：**循环计数部分 A 级**（VM `$FD`=循环计数装入已证，$AC80/波次链 FD 字节对应未证→W21）｜Wv-3 定时机制——**BG 侧负注**（t69：BG 定时=120f 非 16f；16f 节律属 spawn 域）｜Wv-4 地形流消费例程（已收口→VM 链，S4）｜Wv-5 链尾段（C） |
| **Ws**（shot 域，t51 消歧） | `docs/findings/shot-slot-geometry.md` §4、`docs/findings/shot-gating-f1.md` §4 | Ws-6：`$05CE`（motion_code）写点（脚本机运行时写，六字段补证）｜**Ws-7** 弹 state 与网格字段**同址所有权**（时分复用？B 级；原"交错语义待 t52"占位经 t52 F2b 收口转正；【t57 增量】t55 实证存在**网格驻留切换**现象——断言腿曾将其误计为出生（f2350 结案素材），时分复用假设方向获旁证）｜**Ws-8**（t52 W8）：`$32` bit2/bit3 置位时机（模式来源，C）｜~~Ws-9~~ **收口（t75/N-0093）**：`$072E` 在途弹计数（生成 +1/消亡 −1 成对）+ `$074A` 在途上限（"周期表"旧称谓作废）——计数闭环语义定谳 |
| **B**（t32 敌数据面，与 B-WPN/B-SYS 前缀无撞——专指假设编号） | `docs/findings/enemy-dataface.md` §3 | B1 t_005 三参数语义（C）｜B2 t_001 RAM 槽指针语义（C）｜B3 玩家=$1C 系统固定槽（B）｜B4 HP 字段四候选 $05DC/$0630/$06A0/$06BC（C）｜B5 $1C/$70 两系统对象关系（C，Ws-7 互链）｜B6 bank8 $AD00 链=波次脚本链（C，Wv 族互链） |
| **W（enemy）**（t59/t64 脚本机遗留，⚠ 与 ram-watch W 系列**编号撞车**——引用带来源） | `docs/findings/enemy-script-vm.md` §6、`docs/findings/enemy-final-bridge.md` §7 | W10 t_005"时序"参数未证（C）｜W11 4B 装载机流 vs 5B t_005 子系统归属（C，N-0079）｜~~W12~~ **前提证伪收口**（t64：VM 输出不进 $04xx——BG 搬运桥 $85A0 定谳，N-0080/0082）｜**W14** 命中触发清除（HP 消减→释放）例程（C，R-enemy-clear-1 之对照链）｜**W15** 玩家受击触发例程/判定盒/无敌窗口（C，=H-16 本体）｜**W16** `$65` bit7 死亡状态标志（B，N-0084 候选：双事件交替翻转实证；**写者仍未定位**——`bank11:$9D68` 系 data-island 第六例伪命中，N-0095）｜**W15′** 玩家框注册点未单列（t72 残余）｜**W24** 框表重叠判定阅读端（间接寻址 0 命中，N-0095）——碰撞模型=B 级推定｜**W18** `$05DC` 双读：HP vs 死亡倒数备选（命中清除与自然清除共用字段，t66 §1）——语义二选一未定谳｜**W21→W21′**（t78 重释）：格式族已定（`$97` 记录流同族，N-0097/0098）；~~5B 切分~~ **错误格式假设作废**（5B=桥侧 `$7A/$7B` 图案表规格，与 `$AC80` 流无关）；残余=**W21′ 字节级归属**（4B/6B 对齐不稳，N-0099）——B-STG 子格准入**不以此为阻塞**（VM 流=替代规格对象）｜**W22**：t42 `$AC87` 指针线索证伪=`LDA #$87/LDY #$AC` 系传参 |
| **W（stg）**（t79/t83 关卡内容层遗留，⚠ 编号撞车——引用带来源） | `docs/findings/b-stg-01-en.md`、`docs/findings/w28-boss.md` | W26 p1/p2 参数语义（C，`$05-$22` 值域簇模式候选，N-0105）｜~~W27~~ **收口（t88/N-0108，B 级）**：STG-1/7 root=`$A000` 自指=**共享配置池**（共享流解析全同+流内引用他关图案表 B6E1/BBAF/B149）；残余=**W27′** 共享池边界/差异化机制（B）｜~~W28~~ **实用解锁**（t83）→ **W28′**（逐块参数↔间隔公式未全闭，B；素材=实测对 [(16,15),(8,8)] + STG-3/8 CNT 序列 16,16,8,16,8,4,16→15/8/4f 候选）｜**W29（新立，t87）**：各关子流 tick 参数提取（root 无 CNT 的关，B）｜**W30** flow 跃迁对齐未闭（C，N-0111）｜**W31** CALL 子脚本语义二读（B，N-0110）｜**W32** BOSS 实体 vs 后半波次分界判据（C，`($9F)` 区双读：32 项类型表 vs 5B 记录族；生成通道通用，区分在类型/参数层，N-0111）｜**W33（t94）** Mesen2 回调内 PC 轨迹不可得（getState 限制，C）→ 子帧 PC 级证据待 API/替代法；同文件残余 W34（`$160` 各写者五族语义，B）/W35（`$D3-$DA` 位属性上传支路，C，5,577 写路径未追）｜**W36**（t97）Mesen2 I/O 寄存器回调盲区（`$2006` 硬件写点不可插桩，B——W33 替代法成立的依据）｜~~W37~~ **收口（t112/t109）**：双写=**双存语义**（`$8D/$8E/$8F` 每行同步双存，3,155 事件实测——非上下半行/双步进候选）；~~W39~~ 随之收口｜**W38** t96 编号窗与 t97 run 场景口径对应（C）｜**W39** `$8E/$8F` 双调用/帧具体语义（上下半行？滚动双步进？C）｜**W42** `$1A` bit5（8×16 模式位）直读值（C，R-sprite-5 的 8×8 判定前置）｜~~W43~~ **收口（t116/N-0126）**：HUD 槽分配=**直写体系**（`bank00:$8F24` 槽 0：tile=`$BF` 字模/X=`$4C`/attr=0）+ `$8FEE` 序列器（`$31` 阈值 `$1D/$32/$64` 三段）｜**W44** 8/行溢出闪烁运行时观测（C，密集场景 trace；D4 保真对象）｜~~W45~~ **收口（t125/t123）**：R7 模式 0 字模 CHR 映射定谳（124/126=R0:`$7C`/R1:`$7E`）——`chr-font-map.json` 256 tile 基 + R7 双态（linebuf-14）闭合 |｜~~W46~~ **收口（t119/t117）**：`$04`=**VRAM 预算残差共享暂存**（自机 Y 微摆=预算状态，linebuf-12 预算门联动）｜~~W47~~ **部分定谳（t116/N-0128）**：伴随精灵条已证（槽 29-35 七件横排，R-sprite-10）；残余=**W47′ 自机本体槽**（`$04` 残差致跟踪失效，W46 前置）｜**W48** OAM 非连续簇语义（HUD 4-8/61-63 断簇，C）｜**W49** 伴随条实体身份（翼装/道具伴随候选，C）｜**W50（新立，t117）** 装配源变形例程（`$2F/$30` 多用途复用语义；**探针法已给**待执行，B）｜**W47′** 自机本体槽（残余，W50 探针法/多帧跟踪重做待）｜**Wg-1/Wg-2（新立，t135；原裸 W51/W52 撞车已更正）** glyph 层 CHR 形状/行偏移（渲染侧残余，待 t132+ 复审后攻）——Wg=**glyph 域**（t51 三域规则：新遗留自携域前缀） |
| **Q**（t28 孤儿 run 复核遗留） | `docs/findings/orphan-run-recheck.md` §四 | Q1 **收口**（t29：$0190 拾取处理器 + $3B 连收计数）｜Q2 → N-0037 同域｜Q3 干净重跑已做（t29 akey-role）｜Q4 已落地（`oracle/scenarios/README.md` +1 帧约定） |

> 【t51 W 系列消歧规则】三域并存，引用必须带域前缀：**W**=ram-watch（W1-W8，历史深嵌不改名）、
> **Wv**=wave（t47 原 W1-W5 已改）、**Ws**=shot（t48 原 W6 已改）；**新遗留一律自携域前缀**，禁再裸用 W。
>
> 知识沉淀去向：被证伪/收口的方法论教训入 `docs/knowledge/`（并行不可信家族 ×3：read_image 错序 /
> Mesen 共写 / git 索引卷入；采样双陷阱：粒度伪造不变量 / 栈页 watch 盲区；视觉绑定模式），
> 假设本身留在本表——**知识条目 ≠ 假设**，两者引用互见但不混写。
>
> 【t41 挂账 → **t43 已清**】R-item-2 / R-shot-level-3 的"回卷 $0C"措辞已按 kernel/deep **路径 scope 标注**
> （kernel(slot==0) FF→$0C / deep(slot≠0) 裸 INC FF→$00，t35 两路径分野）；deep wrap 用例 pk-10 待 t39 补入。

---

## 8. 防钝化三约束（硬性准入规则）

**约束① 新用例声明覆盖面（准入）**
任何新 scenario / 差分用例 / golden 入库时，必须在登记记录里**声明它提升哪些格**（cell-id 列表 + 期望强度）；
未声明覆盖面的用例**不入 oracle/ 不算覆盖**（可以保留为探索性产物，但矩阵不认）。
新分母格（含子格）准入同样走此约束：先给证据指针（§1 格式），再入分母。

**约束② 覆盖只增不减（ratchet）**
格子强度只升不降；删除分母格、降强度、放宽判定阈值一律视为回退，必须单独 commit 写明理由并经 captain 验收，
且在轮次日志留痕。白名单/基线变更按 AGENTS.md §5 由用户确认。

**约束③ 每轮给下一最优格（反空转）**
每轮轮次报告（`docs/rounds/round-NNN.md`）必须给出"下一最优格"。机械取法（顺序即优先级）：

1. 取 L0 且依赖已清（前置格 ≥L1）的格；
2. 同批内按面优先级：行为面核心系统（B-SYS-ITEM/MONEY/SHOP）→ 武器/炸弹 → 关卡 → 其他；
3. 同面内按证据就绪度（已有 scenario/trace 指针的优先）；
4. 取不出格 ≠ 无事可做：优先补 §2.6 待取证清单的证据（那是分母增长的入口）。

**绿 ≠ 完整**：门禁全绿只证明"已声明的问题没回归"。任何轮次报告若只写"测试全绿"而不写矩阵收敛数据（各面格数、强度分布、下一最优格），视为该轮未完成。

---

## 9. 维护规程

| 动作 | 谁 | 门禁/审核 |
|---|---|---|
| 新增/拆分分母格 | spec-writer（证据列必填） | captain 抽查证据指针（master-plan §5.2） |
| 分子登记/升强度 | verifier / spec-writer | 证据绑定硬规则（§6.2）+ 对应差分门禁 |
| 降强度/删格 | 任何成员提议 | 单独 commit + 理由 + captain 验收（约束②） |
| 每轮收敛记录 | captain | 轮次日志含：各面格数、强度分布、下一最优格（约束③） |
| 机器可读镜像同步 | 任何矩阵修订后 | `python3 tools/matrix_sync.py --export && --check`（completeness-ratchet + 自检） |

> **机器可读镜像**（2026-10-12 t20）：`data/status/completeness_matrix.json` 是本文件的门禁输入面
> （completeness-ratchet 消费；`covered ⇔ strength ≥ L1`）。单一真相源仍是**本文件**；
> 同一格两处的强度/证据引用不得矛盾，由 `tools/matrix_sync.py --check` 机械比对（44 格 + §2.4.x 登记记录绑定）。

版本记录：`v0 (2026-10-12) 建立：行为面 44 格、规则面 44（下限）、符号/数据面占位、分子全 L0`。
后续修订在本行下追加，不删历史。
`v0.1 (2026-10-12, t6) B-SYS-SHOP/MONEY/UI 规则面覆盖（R-shop-1..4、R-money-1..3、R-ui-1..5 → docs/spec/shop-economy.md、ui-flow.md），三格分子登记 L1（§2.4.1，web 证据封顶）；constants.md 建立（C-shop-p1..p3、C-money-template、C-money-unk 不解释）；E-R04 指针精确化为 rom:8516–8523。`
`v0.2 (2026-10-12, t17) S2 证据回填：R-shop-5..8、R-money-4、R-item-1、R-ui-6..8 入规则（绑 scenario:analysis/traces/<id>+帧号）；B-SYS-SHOP/MONEY/ITEM/UI 四格升 L2（§2.4.2）；空位闭环状态更新（部分闭环 3 项），新增 H-09..H-13（未闭环项）；§2.6 自机 gameplay 入口解锁。`
`v0.3 (2026-10-12, t20) 机器可读镜像 data/status/completeness_matrix.json 建立（44 行为面格 + 四面计数按现状：规则面 44 下限/21 条目/覆盖 4，符号面 276+3+18+7+560+414，数据面 8+7+128）；tools/matrix_sync.py --check 机械一致性自检（格 ID/强度/证据引用/content/绑定）；§2.4.2 记录补 trace: 指针；completeness-ratchet 门禁激活。`
`v0.4 (2026-10-12, t23) S4 行为规则第一批：B-SYS-MOVE/B-SYS-FIRE 按约束①准入（证据 N-0011..0024 + bank03:A084-A0E8/A2B4-A2FA + gameplay-stage1 RAM diff），行为面 44→46，两格 L2（§2.4.3）；R-move-1..6/R-fire-1..6 入 docs/spec/player-movement.md（规则面 21→33 条目）；constants.md v2（速度对 ±2.8125、钳制 $0E-$F3/$14-$DF、消耗表 $A336,Y=[1,2,3,2] 逐值；类型表/速度查表登记不解释）；新增 H-14/H-15；§2.6 判定点留守。`
`v0.5 (2026-10-12, t26) t22 证据消化：R-money-4 扩写（money_counter_bcd $E0-$E6=显示值本体+前导零抑制渲染）+ R-money-5/6（加法器 bank14:DBCE 逐值加数表 10..2000/进位/封顶 999999、extend 阈值 8000→$ED=$40）+ R-bomb-1..3（bomb_type_index $0190 四值域、拾取三方同步、映射 0=F/3=W 实测 1=B/2=T 推定）；constants.md v3；H-09/H-11/H-12 原地更新残项（H-12 字母归因收口）；N-0025/W1 证伪留档；知识条目 gotcha-parallel-read-image-order-unreliable 入库（§2.4.4 增补记录）。`
`v0.6 (2026-10-12, t27) 死亡/重生事件规格化：B-SYS-RESPAWN 按约束①准入（证据 oracle/traces/gameplay-stage1.player.jsonl 帧域 f1831-1916/f2047-2101/f2131-2175 + t24 L3 发现），行为面 46→47，L2（§2.4.5）；R-respawn-1..4（窗口/移动冻结/坐标重置 128/160/弹药回满 4，L3 已可判）入 player-movement.md；W5 收口（=重生坐标重置，ram-watch-s3 §7 原地更正）；新增 H-16（hit-test 未证域，关联 t25 hit-test.txt）；死亡语义未证不写。`
`v0.7 (2026-10-12, t36) 弹丸系统规格化（关键路径解锁）：R-shot-gen-1..4/R-shot-motion-1..3/R-shot-level-1..4（11 条）入 docs/spec/shot-system.md（B-SYS-FIRE 证据扩充 §2.4.6，覆盖只增）；constants.md v4 弹丸六表逐值（级表/配置流/双 LUT/动作流/时长表，seed 逐字节核）；H-02/H-14 收口；残项 A1-A5 + $925F 分支（t35 在考）如实留空；t29 样本列错位更正（指针标签经代码级复核正确）。`
`v0.8 (2026-10-12, t37) t34 审计处置：FA-1 反向规则 R-item-2/3 补齐（pk-1..7 对拍用例获承载，coverage_map 回填 l2_cases_without_rule 7→0）；FA-2 声明降级（R-respawn-2/3/4 "L3 已可判"→"L3 部分"，真覆盖需求=窄窗重生场景+$018F/$04E0 watch，列单独 commit 需求未擅动）；FA-5 digit-* 部分覆盖如实现状登记（R-money-4 注）；R-shot-level-3 "级上限 $0C" 证伪更正（pk-5 实测 12→13，$0C=回卷目标）；§2.4.7 登记。`
`v0.9 (2026-10-12, t44) FA-2 真覆盖登记：R-respawn-2/3/4 升 L3 已判（t40 证据包 respawn-window.md §3：双事件逐帧向量+240 窄窗 capture+时序定谳），FA-2 降级注撤销、t27 分段遮蔽观察升 A 级；追加范围（t39 同步）：R-shot-level-3 公式勘误 3+→4+((级-4)&7)（ha-2 实证，值域 [0,11] 无越界，A1 闭合）、R-shot-level-4 转正（t35+t39 双定谳）、R-item-2 deep 用例 pk-10/11/12 转正；残余转注 E1/E2 窗长差异→H-16 续问、死亡语义 W3；§2.4.8 登记。`
`v0.10 (2026-10-12, t46) 关卡系统规则面开张：stage-timeline.md R-stage-1..6（波次链分段记录流/行为脚本指针族/$ACEF 块范围对/滚动指令嵌入=混合指令流定性/bank 图归属/时间线骨架+churn 交叉验证）；B-STG-01-TL 首个关卡子格按证据准入（§2.3.1，L2，帧级对齐待 S1），其余子格不预建；行为面 47→48；S1-S6 入 §7.1 S 系列；constants §1.5 关卡结构锚点。`
`v0.11 (2026-10-12, t49) t47 消化：R-stage 修正（"参数对递增"=地址伪影更正：块首字=自起点−6 7/7；记录格式 FE 分段 8 块/自指首字/链末 FF 升 A 级）+ R-stage-1a 三裁定（时刻✗/序号✗/掩码部分支持）+ $ACEF 收口顺序扫入口；STG-01-TL L3 升格否定如实登记（维持 L2/部分）；B-SYS-SPAWNRHYTHM 新格约束①准入（R-spawn-1..3，L2→L3 部分，16 帧节律+突发簇 104 事件 A 级）；t47 W1-W5 入 §7.1（与 ram-watch W 撞车标警告）；行为面 48→49。`
`v0.12 (2026-10-12, t51) t48 登记：R-shot-alloc-1..4 建档（槽位几何 4 组×5/组距 $1C、分配器首次适应/空槽=state==0/回收=寿命归零/满载 SEC；L2 shot-alloc-1..4 三方全等）+ R-shot-gen-1..3/alloc-* 升格标注"L3（前提 t50 golden 转正，落地后改已判）"（FA-2 先例不虚高）；t45 槽位几何未证注记撤销（18 例矛盾值=伪读，oracle/traces/README 原地更正）；W 系列三域消歧（W=ram-watch 不改/Wv=wave 原 W1-W5/Ws=shot 原 W6，新遗留自携域前缀）；B-SYS-FIRE 证据扩充（§2.4.11）。`
`v0.13 (2026-10-12, t53) F2 churn 证伪更正：shot-slot-geometry §3 churn 统计口径不成立原地更正（20 槽 state=B5 交错区静态非寿命数据/20 帧不递减/同址交错；921 帧零解释被 F1 反证）；R-shot-alloc-* 证据链撤除 churn 引用（几何/分配断言不依赖）；Ws-7（W5/Ws 交错语义）入索引标待 t52；知识条目 gotcha-false-green-idle-assertions 入库（断言存在≠生效，caed518 ss-1/2 空转教训）。`
`v0.14 (2026-10-12, t56) 控制方案大修（t52 冲击波更正扫荡）：R-shot-gen-2 证伪更正（主武器=B 键按住连发，门控链 $18 bit6→$39/$3A→$32 bit2；N-0035"自动连射"撤回 N-0073、t25"B=弹种切换"更正 N-0072）；H-14 定谳文本两处作废重写（A=投放/B 长按连发/弹种切换=拾取字母圈）；R-move-1 补 bit6=B；R-shot-alloc-1 churn 口径更新（fire-hold 0-9 众数 2-6、容量 20 未饱和，N-0074）；B-WPN 25 格控制方案备注；Ws-7/8/9 入索引。`
`v0.15 (2026-10-12, t57) t55 后续：f2350"待联查"结案（=移植/实现差：断言腿出生检测误计 W7 网格驻留切换，补 R-shot-gen-3 签名后负例归零；非规格缺口，R-shot-alloc-2 无需改）；R-shot-motion-2 LUT 索引无掩码语义补正（motion≥16 读表间字节 $88CB=$F8，"16 向"非掩码规则）+ constants v6 表注；R-shot-gen-3 出生签名（坐标=玩家±9px）入规则（A 级）；R-shot-gen/alloc-* L3 前提标注升级"已判"（t55 转正落地）；Ws-7 增量（网格驻留切换现象实证）。`
`v0.16 (2026-10-12, t60) enemy 模块合同开张：enemy-system.md R-enemy-1..6（t_005 生成记录 5 字节/行为脚本流 FE·FC 终止+嵌套指针/双 SoA 网格 $1C 28 槽+$70 弹对象/字段阵列/t_001 变长绑定流/生命周期已证链——命中清除段 H-16 待证）；B-SYS-ENEMY 约束①准入（§2.4.12，L2），行为面 49→50；B1-B6 入 §7.1 B 系列；constants §1.6 三锚点。`
`v0.17 (2026-10-12, t61) R-enemy-2 回填（t59 VM 定谳）：指令集逐码语义表（$00 结束/$02-$7F 嵌套调用/$80-$FB 指令+3参/$FC X 调整钳 $20-$E0/$FD 循环计数/$FE 循环跳 2B/$FF 跳转，8 类全 L2 可判）+ R-enemy-7 VM 槽字段 + R-enemy-8 记录装载机 4B 格式 + 两层模型注（W12 数据流桥待证）；N-0078 位置更正联动（bank2"解释器"标签证伪→bank00:$844A）；t59 W10/11/12 入 §7.1（撞车警告）；H-16 维持待证。`
`v0.18 (2026-10-12, t63) t62 三勘误入库：R-enemy-8『X=p0−$0F』→p0−$10（SBC 前 carry=0/LSR 链置位，en-ld-1 ROM 实证）；R-enemy-2 $FC『钳 $20-$E0』→**界门**（界内 [$20,$E0) 退槽/界外续行，X 不钳）；R-enemy-4 钳制断言**写路径限定**（仅玩家槽 0 移动路径已证，其余槽含 W12 未证桥写入、f1304/f1368 界外放置实测→全槽断言待 W12 升格）；findings/enemy-script-vm.md 两处原地更正；§2.4.13 备注同步。`
`v0.19 (2026-10-12, t65) VM 定性反转扫荡（t64 冲击波）：R-enemy-2"行为脚本流"→**BG/地形脚本机**（W12 前提证伪：VM 输出→$0130/$0140 BG 缓冲族+位属性，非 $04xx 敌网格；三层 RAM 分立模型替换两层注；t62 Rust 模块命名调整建议报 captain 转 porter）；R-enemy-clear-1 入档（$6F4,X 到期→退场序列，B 级 L2，自然清除非命中触发）；命中段负结论登记（升格判据不满足=缺 A 级触发例程/样本，H-16 维持，W14/W15/W16 入索引）；S4 地形消费例程收口→VM 链（格式残余 W21）；N-0084 死亡位候选入索引。`
`v0.20 (2026-10-12, t67) t66 登记：R-enemy-hit-1 入档（A 级命中清除链：接触帧 $05DC+n 归零→类型清 0→槽释放，f1404 字段列差分+12 例样本集）；包络候选不发明（玩家 |dx|≤12,|dy|≤12/弹×敌 |dx|≤9,|dy|≤15，B 级→W15 注，触发例程 W15 未定位）；寻址更正扫荡（n×$1C 槽寻址误读→字段基址+n stride 1+N-0086，findings/规则原地更正，两套 SoA 语言统一）；W18（$05DC 双读）入索引。`
`v0.21 (2026-10-12, t70) t69 登记：R-bg-scroll-1..4 入档（基础 dy=−2px/帧/120 帧单帧脉冲 dy=−18（10/10）/f1193 阶段边界/商店卷轴区分界）**L3 已判**（captain 裁定：帧级确定性值序列标准 L3 断言）；B-STG-01-TL 升 L3 已判（波次链帧级对齐 S1 负结论维持，两断言面分离）；enemy-final-bridge ×3→×5 更正传播（N-0088）；Wv-1 时刻读双重证伪/Wv-2 循环计数部分 A 级/Wv-3 BG 侧 120f 负注/W21+W22 入索引；t42 $AC87 线索证伪（传参非指针）。`
`v0.22 (2026-10-12, t74) 标注状态刷新（t68 条件履约，FA-2/t57 先例）：R-enemy-hit-1 升"L3 部分"（12 样本入 golden+HitBox 谓词+负向自证三=样本驱动断言面成立；三-way ROM 对拍待 W15 后升"已判"，照实不虚高）；双清除链对照表入档（命中清除 A 级 vs 自然清除 B 级）；R-spawn-1/2 帧表正寻址重采复核注（t72 在途）；W21 优先级注（B-STG 其余子格准入前置）。`
`v0.23 (2026-10-12, t75) t72 消化：R-spawn 节律证伪重写（16 帧周期+f1301 簇系误址伪影→正寻址 8-10 帧主导（112 激活/110 释放，Δ10×19/Δ8×15）+新突发簇 11 帧点，N-0094）；R-enemy-hit-2 命中框注册系统入档（A 级：统一框表 $0780 族 21 条/帧+弹侧 7×10 框 X−4..X+3/Y−5..Y+5，碰撞重叠判定 B 级推定/W24 阅读端待证）；R-shot-gen-1 在途弹计数语义补正（$072E ±1 成对/$074A 在途上限，"周期表"作废，N-0093）；Ws-9 收口；W15′/W24 入索引、W16 写者注。`
`v0.24 (2026-10-12, t78) t76 登记+B-STG 解锁（captain 裁定）：R-stage-chain-1..3 入档（四层链架构 ($88) 根流→$97 解释器→($A1) 4B 装载机→VM→桥→BG；6B/4B 记录解码+$FD/$FE/$FF 控制字节 L2 可判）；Wv-2 收口（t47"10 FD/08 FD"=$FD 循环计数命令实解）+t47"FE=段末/FF=链末"重释（解释器控制字节）；B-STG-xx-EN/BOSS 准入解锁（格式族定谳+VM 流替代路径，$AC80 对齐不阻塞，W21′ 残余）；t69"5B 试切分"重释为错误格式假设（5B=桥侧图案表规格）；N-0096..0099 联动；B-STG-01-TL bindings 扩链架构规则。`
`v0.25 (2026-10-12, t82) t79 登记：B-STG-01-EN 子格约束①三分区入矩阵（✅内容层 8 记录+目录入口+1:1/⚠时刻层 W28·参数 W26·自指 W27/❌BOSS·EN-9+·其余 7 关——覆盖只记已判，L2）；R-stage-dir-1..2 关卡目录表规则（bank8:$A000 8B×8 {root 流/图案表/参数对/尾指针}+bank00:$8000 装载+$0180=关号×8，A 级新发现）；R-stage-chain-3 EN 联动注（调度 8 段↔内容 8 条 1:1）；W(stg) W26/27/28 入索引；BOSS 子格预登记待证；行为面 50→51。`
`v0.26 (2026-10-12, t85) t83 登记（captain 双裁定）：B-STG-01-EN 时刻层升 **L3 已判**（R-stage-tick-1..3：间隔三档 8/10/15f+86 tick 帧表+事件分布 1×73/2×8/3×1/5×4，观测级确定性断言面与 R-bg-scroll 同标准；W28 实用解锁→残余 W28′ 逐块公式）；B-STG-01-BOSS 三分区入矩阵（L2/部分不升，R-stage-boss-1 预规则巡逻形态候选）；EN-9+ 同格式延续收口；W(stg) 残项更新（N-0105）；行为面 51→52。`
`v0.27 (2026-10-12, t88) t87 批量登记：B-STG-02..08-EN 七子格三分区入矩阵（✅骨架/计数/tick 表、⚠间隔换算·W29、❌逐条语义——覆盖只记已判，行为面 52→59）；R-stage-dir 五脚本族归属注（EN $B2xx·$B3xx/BOSS-后半 $A4xx·$A6xx/链段 $ADxx=调度段脚本池 t47 同源定清/共享配置 $A39B/散户）；W27 收口（共享配置池 N-0108）残余 W27′；W29 新立；W28′ 补实测对 [(16,15),(8,8)]+STG-3/8 CNT 素材；【追加 t86】R-stage-tick-1 混突发注扩大段 D/F（平行 +4 轨迹 2236+8k/f1892 离轨点）+出现层硬断言面注（干净段 A/C/E 入断言，混轨段观测面不入）。`
`v0.28 (2026-10-12, t91) t89 登记（captain 双裁定）：R-boss-patrol-1/2/3 入档升格（FC ±v 左右往复/FE 自环驻留/CALL 运动原语，A/B 级 L2 已实证，$A41F/$A435/$A486 实录）；R-stage-tick-4 出场触发规则（VM 流推进本身+波次 END 字节+$865C 九字段写 $04xx 网格→sub_8619 分派，A 级，t32 生成侧补全注）；BOSS 三分区更新（❌触发→✅，整格维持 L2/部分）；W30/W31/W32 入索引。`
`v0.29 (2026-10-12, t95) t94 登记：R-l4-linebuf 五条时序不变量入档（l4-visual-plan §3.5：bridge 整块扫写/NMI 4 组×8B 上传 $2007 步进 +$20/$160 谁写谁清/调色板源 $161-$167/采样口径=NMI 级——L2+L4 断言面）；W-NEW 收口（t93 快照悖论=相位非矛盾，原地更正）；BG 缓冲族↔行缓冲联动注（R-enemy-2a）；W33 入索引（W34/W35 同文件残余注）。`
`v0.30 (2026-10-12, t98) t97 登记：R-l4-linebuf 五→**六不变量**（地址维补全：目标地址=$8F:($8E+$0F)、组内 +$20/帧批 $40 族 2 组、滚动行 8 帧/行）+ 跨任务互证注（8 帧行×t69 −2px/帧=16px 行高自洽）；t96 相位注精确化（快照不可用≠写流不可用，W33 替代法成立）；W36/37/38 入索引。`
`v0.31 (2026-10-12, t101) t100 登记：R-l4-linebuf-7 BG 装载通道（$8D→$EBF4 行地址 LUT 16 项 [15]=$2CA5 特例→行缓冲→NMI $2007，"未观测机制"证伪=无 DMA/暗通道）+ linebuf-8 调色板通道（$8844/$88DA 族→$161-$169→$160 流尾置型→NMI $F105→$3F00/$3F10，3,346 事件实测）；W-NEW2 收口（t99 零写之谜=采样窗缺口，t99 路由 b 解锁）；四层观测模型全绿注（BG/属性/调色板/地址相位）；W37 部分收（双写=双调用）→W39 新立。`
`v0.32 (2026-10-12, t105) t103 登记：R-sprite-1..5 入档（entry 四元组 Y=$F8 隐藏协议/attr 位语义+$88E3·$88F7 (tile,attr) 旋转动画表/NMI $2003+$4014 整页 DMA/OAM 槽分区 $10·$F0·$54 bit0/8·行溢出闪烁 D4 保真）；W42/43/44 入索引；L4 双层联动注（内容帧=BG+sprite 合取）；BUG 登记通道预告。`
`v0.33 (2026-10-12, t108) t106 登记：R-sprite-6 Y 锚点表（逐实体：自机 −$30 含 +$04 微调·<$30 截止门/弹敌 −$08，40px 错位定界）+ R-sprite-7 OAM f−1 相位（NMI DMA→下帧显示）+ R-l4-linebuf-9 nametable 基址上下文（$2000 系 HUD vs $2Bxx 系关卡，+$0F 半行 32B/组间 +$20）；t102『半行快照』族 a 收口（基址上下文非数据矛盾）；W45 入索引（W42 联动）；双层联动注 +Y 锚点维——L4 断言面终补。`
`v0.34 (2026-10-12, t110) t107 消化：R-sprite-6 锚点语义正名（screen=stored+8 存值偏移；自机 +$2C 族 palette-1 误分类弃用，uniform +8 最优）+ R-sprite-7 f−1 实测注 + 绑定进度注（#B53120 定谳 5 槽，3 色 1/3，剩 #E4E594/#64B0FF）+ chr-font-map.json 256 tile 为 W45 基入 data 面 + F1 两族待证注（HUD 行放置序/自机槽分区）。`
`v0.35 (2026-10-12, t112) t109 登记：R-l4-linebuf-10 放置序降序+双写（行索引逆序映射+每行 $8D/$8E/$8F 同步双存，3,155 事件——W37 收口=双存语义，W39 随收）+ R-sprite-8 OAM 槽布局（f1500 全 64 槽：HUD 4-8·61-63/游戏 9-13+/槽 0 序列器专用）+ R-sprite-9 HUD 序列器（$8FEE：$31 计数 29/50/100 三段→tile 切换+$160 挂起）；t103 预留区表述修正（槽 0=序列器专用）；W46/47/48 入索引（+$04 残差注 W46 联动）。`
`v0.36 (2026-10-12, t116) t113 登记：R-l4-linebuf-11 **双装载体系二分**（文本直址 t2 内嵌 PPU 地址直写 $2006/$2007（f360/f1080 零事件实证）vs 行 LUT 关卡 BG——N-0127 根因注：移植端行 LUT 误用=落点全错，t111 帧位不可判谜底=体系错配收口）+ R-sprite-10 伴随精灵条（槽 29-35 七件 Y+127/X+8×7，85% 主导玩家坐标系锁定，W49 候选）；W43 收口（HUD 直写体系）；W47 部分定谳→W47′ 自机本体槽残余（W46 前置）。`
`v0.37 (2026-10-12, t119) t117 登记：R-l4-linebuf-12 **文本装配链四段**（流指针 $2F/$30→$12/$13、地址合成 (字节&$1F)+$14|$20 三类字节、$E6DF 推进 $77=$20 标记、$E828 VRAM 写预算门 $20 计数·$50/$30 双档·<4 递归排空、$E7DC 寄存器写 $1A/$1B 门控）——双装载文本侧全链闭合；W46 收口（$04=VRAM 预算残差共享暂存，自机 Y 微摆=预算状态）；预算节流=像素差源注（不复刻即帧差）；W50 新立（探针法已给）/W47′ 维持。`
`v0.38 (2026-10-12, t125) t123 消化：t121『值定位说』证伪（写值层差真因=记录制误用+R7 时序，偏移+游程制回正 +1 推进 4A×3 实证）；R-l4-linebuf-13 偏移+tile 游程记录制（<$20 偏移/$20-$7F 游程/≥$80 控制↔$0180 标志链）+ linebuf-14 R7 模式双态（模式 0=字模 CHR R0:$7C/R1:$7E/模式 1=$36/$38 图形段）；W45 正式收口；定位账三小项=调参域注记（游程边界/排空节奏/R7 相位）。`
`v0.39 (2026-10-12, t135) t131 F2 修正：R-l4-linebuf-15 **PPU 协议规则**（$2006 hi/lo 锁存+$2007 写自增+读缓冲副作用，NESdev 比对）——十一击共同暗雷根修（注册器地址≠nametable 目标）；t127 数据集口径重释注（消费侧均受影响，影响面清单占位待对抗复核 #2 校准）；glyph 层残余 W51/W52 新立（CHR 形状/行偏移，待 t132+ 复审后攻）；『注册器地址≠nametable 目标』gotcha 知识库候选（待审稿）。`

---

## 10. v0 建立时的核对输出

- 门禁 `python3 gates/cli.py check`：**SKIP —— `gates/cli.py` 尚未建立**（M0 基建任务另行交付；本任务为矩阵文档）。
  > 【2026-10-12 更新】`gates/cli.py` 已交付并实跑：`check` = **PASS**（`SUMMARY: PASS=4 FAIL=0 SKIP=7 TOLERATED=0`，无静默跳过）。上句保留为 v0 建立时的历史记录。
  人工可核验的最小证据（下列命令实际执行于 2026-10-12，仓库根目录）：

```text
$ sha256sum rom/gunnac_cn.nes
4d76901c1b2455f2bdae8e8ac99b1779441f9f2155614dab222c229ae06c77ae  rom/gunnac_cn.nes

$ grep -abo '1 2 3 4 5 ' rom/gunnac_cn.nes       → 9095 (0x2387)
$ grep -abo '1 2 3 4 5 6 7 8' rom/gunnac_cn.nes  → 9534 (0x253E)
$ grep -abo 'F B T W' rom/gunnac_cn.nes          → 9626 (0x259A)
$ grep -abo 'WEAPON-0' rom/gunnac_cn.nes         → 5419 (0x152B)
$ grep -abo 'BOMBER-0' rom/gunnac_cn.nes         → 5430 (0x1536)
$ grep -abo 'INVESTIGATE THE EIGHT AREAS' rom/gunnac_cn.nes → 12036 (0x2F04)
$ python3 逐偏移 dump：E-R05 价目串 $10@8929 / $6@8958 / $4@8986 均在 8914–9000 记录区
$ sha256sum shots/*.png：6 个互异帧（标题×3、菜单×2、商店×26 同帧），商店帧 = efcfe736…
```

- 遗留假设：H-01…H-08（§7），全部 C 级、未入分母。
- 覆盖面声明（约束①对本任务自身生效）：本任务不产生 scenario/用例，覆盖面 = 分母建立本身（44 格全部 L0 入册）。
