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
| B-STG-01-TL | STG-01 滚动时间线：波次链结构 / 滚动指令嵌入 / bank 图归属 / 目录结构（帧级对齐待 S1） | stage-timeline.md §1-5（bank8 $AD00-$ADD0 链/$ACEF 目录/$AE87 滚动写点）, gameplay-stage1 churn 1090/1090 帧 | B | **L2**（R-stage-1..6；帧级对齐 L3 待 S1） |

> 其余 TL/EN/BOSS 子格按下方规则**证据后准入，不预建**。

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
工件: docs/spec/stage-timeline.md#R-stage-1..6、docs/spec/constants.md#C-wave-chain/C-wave-dir/C-scroll-vars
日期: 2026-10-12
备注: S1（段参数语义）未证前帧级对齐不可判——L3 前置；S2-S6 假设留 stage-timeline §4；其余 TL/EN/BOSS 子格不预建（证据后准入）
```

### 2.5 分母统计（v0）

| 面 | 格数 | 备注 |
|---|---|---|
| 行为面/内容面（§2） | 25 + 4 + 8 + 10 + 1 = **48** | 武器 25 / 炸弹 4 / 关卡 8 / 系统 10 / **关卡子格 1**（t46 +B-STG-01-TL）；其余子格不预建 |
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
| H-14 | A 边沿/主武器/炸弹键分工机制——【2026-10-12 t36 收口】**定谳**：主武器=计数器驱动自动连射（`bank3:$A1FD-$A22B` 无按键判断，R-shot-gen-2）、A 边沿=投放（R-fire-1）、B=弹种切换（N-0030 路径）；齐射型由 `$34` 分派（R-shot-gen-4） | ~~C~~ | 已收口（t25 静态定谳 + t29/t36 规格化） |
| H-15 | `game_action_mode`（$0401）除 0/7 外的值域语义（N-0024 同址多用途） | C | S2 多场景 $0401 采样 |
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
| **H**（矩阵假设） | 本节上表 + `docs/findings/upgrade-branch-h02.md` | **H-01/H-02 收口**（t25 两说合一 + t35 深路径定谳：同型/异型 × $018C==$FF 2×2 + $32 bit2 门控；kernel 回卷 $0C / deep 裸 INC 回卷 $00 **两路径语义分野**，范围收窄见 shot-behavior §1.2 更正 + N-0058）｜H-03..H-13、H-15、H-16 见上表｜**H-14 收口**（t36 三线分工定谳）｜**H-A 新立**（t35）：$33 可越 12（deep 路径）→ `$9457` 越界索引行为 = 必答项 |
| **W**（RAM watch 假设） | `docs/findings/ram-watch-s3.md` §7/§8.4/§9.5 | W1 证伪（真源 $E0-$E6）｜W2 观察中｜W3 未闭环（等级/残机 _todo_）｜W4 未闭环（slot 偏移）｜**W5 收口**（=重生坐标重置）｜W6 B 级维持（速度查表分划）｜W7 → [gotcha-parallel-read-image-order-unreliable](../knowledge/gotcha-parallel-read-image-order-unreliable.md)｜**W8 证伪**（孤儿 run 干扰）→ [gotcha-concurrent-mesen-output-corruption](../knowledge/gotcha-concurrent-mesen-output-corruption.md) |
| **S**（t42 关卡结构假设） | `docs/findings/stage-timeline.md` §7 | S1 段参数语义（时刻/序号/掩码，C）｜S2 $ACEF 目录消费方式（C）｜S3 bank2 $8100 页目录读者（C）｜S4 地形碰撞数据（C）｜S5 关卡长度/切换条件（C）｜S6 bank9/10 逐关归属（B，候选定位级）——S 系列与 H/W/Q 并列互通，收口须同步本表 |
| **Q**（t28 孤儿 run 复核遗留） | `docs/findings/orphan-run-recheck.md` §四 | Q1 **收口**（t29：$0190 拾取处理器 + $3B 连收计数）｜Q2 → N-0037 同域｜Q3 干净重跑已做（t29 akey-role）｜Q4 已落地（`oracle/scenarios/README.md` +1 帧约定） |

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
