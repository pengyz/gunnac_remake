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
| E-R04 | `rom:8523` (0x214B) | `-------$` 金钱显示串（破折号+货币符）；**像素佐证**：`shot:shots/auto_f01080.png` 中 `你有 ──￥` | A |
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

**关卡子格登记规则**（约束①的具体化，防猜测入分母）：每关的
`B-STG-xx-TL`（滚动时间线）、`B-STG-xx-EN`（敌人生成/波次）、`B-STG-xx-BOSS`（关底 Boss）
**不预先入分母**；由 S2 场景捕获或 S1 表识别取得证据后，按 §8 约束①声明覆盖面、当场入分母。
各关的主题/名称（动物、工厂等）目前仅开场文本有暗示（`ANIMALS, PLANTS AND PRODUCTS FROM THE FACTORIES`，E-R09），
属 C 级 → H-07，**不入分母**。

### 2.4 系统（B-SYS-<名>）

| 格 ID | 内容单元 | 证据指针 | 级 | 分子 |
|---|---|---|---|---|
| B-SYS-SHOP | 商店系统：进出时机（关间）、货架商品、价目、购买流程、按键切换 | E-W06, E-R05, E-R04, E-S02 | A/B | L0 |
| B-SYS-MONEY | 金钱系统：获取/显示/累计/上限/消费 | E-W06, E-R04, E-S02（`你有 ──￥`）, E-R05（价目）, E-R02（`10000000000` 待证 → H-03） | A/B | L0 |
| B-SYS-ITEM | 道具系统：编号圈（换武器）、字母圈（换炸弹）、升级圈、翼装 wing（E-W05：多挨一击+武器可继续升级） | E-W02, E-W03, E-W04, E-W05, E-S02 | A/B | L0 |
| B-SYS-DROP | 掉落系统：关内随机浮动金钱、被毁对象/敌人掉钱、道具掉落 | E-W06, E-W02 | B | L0 |
| B-SYS-BOSS | Boss 系统：最终 Boss 多相（E-W07 "first phase of the last boss"）；各关 Boss 子格按 §2.3 规则登记 | E-W07, E-R09 | B | L0 |
| B-SYS-UI | 流程系统：标题/菜单（开始·设置）/DEMO/GAME OVER/CONTINUE/结局文本 | E-R01, E-R10, E-R11, E-S01, E-S03 | A | L0 |
| B-SYS-SCORE | 计分/最高分系统（`HI SCORE`/`SCORE` 显示） | E-R01, E-S01 | A | L0 |

### 2.5 分母统计（v0）

| 面 | 格数 | 备注 |
|---|---|---|
| 行为面/内容面（§2） | 25 + 4 + 8 + 7 = **44** | 武器 25 / 炸弹 4 / 关卡 8 / 系统 7；子格未计 |
| 规则面（§5） | 44（下限，按行为格派生） | 每格 ≥1 条 R- 规则 |
| 符号面（§3） | **未计**（占位） | S1 后机械导出 |
| 数据面（§4） | **未计**（占位） | S1 后机械导出 |

### 2.6 待取证清单（明确**不在**分母，取证后按约束①准入）

| 候选格 | 缺什么证据 |
|---|---|
| 自机移动/射击手感、判定点 | 无 gameplay 截图/trace（现有 shots 只有标题与商店）→ S2 首批场景 |
| 命数/死亡重生/CONTINUE 语义 | 仅 `CONTINUE` 串（E-R11），行为未观测 |
| 音频（APU）行为 | 按 pipeline R5 先 scope-decision，不入行为面 |
| 各关主题/名称、各关 Boss | 见 §2.3 与 H-06/H-07 |
| 商店 4 号商品价格 | E-R05 记录区价格串缺失 → S2/S3 |

---

## 3. 符号面分母（占位）

> **S1 后由 IDA/Ghidra 机械导出** —— v0 不做任何人工估数。

- 全集定义：代码符号 = IDA（`m65`）自动分析的子程序/入口/中断向量（RESET/NMI/IRQ）；RAM 符号 = 0x0000–0x07FF 直接页符号与映射寄存器使用点。
- 计数规则（机械）：子程序以入口+RTS/RTI 边界计数，跳转表目标按 xref 计入；bank 窗口内代码区全覆盖。
- 导出产物落 `data/naming/remap-table.json`（机械层，pipeline §7），本表只登记**计数与覆盖率**，不登记语义名。
- 在 S1 导出前，本面分母=未计；**禁止**用"大概几百个函数"之类的估计填表。

## 4. 数据面分母（占位）

> **S1 后由 IDA/Ghidra 机械导出** —— 全集 = 数据表 + xref（表边界、字段切分、指针表）。

- 计数规则（机械）：以 IDA/Ghidra 识别的数据对象 + 每个被≥2 个代码 xref 索引的连续区段计一表；CHR/调色板/文本表按资源计。
- 已知数据锚点（**不构成分母**，仅供 S1 交叉核对，均有 §1 证据指针）：
  - 商店价目记录区 `rom:8914–9000`（E-R05）；
  - 平行串记录区 `rom:9095/9534/9626`（E-R06/07/08）与文本记录区 `rom:8523`（E-R04）；
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

版本记录：`v0 (2026-10-12) 建立：行为面 44 格、规则面 44（下限）、符号/数据面占位、分子全 L0`。
后续修订在本行下追加，不删历史。

---

## 10. v0 建立时的核对输出

- 门禁 `python3 gates/cli.py check`：**SKIP —— `gates/cli.py` 尚未建立**（M0 基建任务另行交付；本任务为矩阵文档）。
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
