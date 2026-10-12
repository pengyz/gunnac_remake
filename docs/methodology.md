# 加纳战机（Gun-Nac）Rust 重写 —— 方法论调研

> 状态：调研稿 v1 ｜ 目标：不是写模拟器，而是把游戏逻辑用 Rust 改写成 PC 原生游戏

## 0. 目标识别（已完成）

| 项 | 结论 | 证据 |
|---|---|---|
| 游戏 | **Gun-Nac（ガンナック，加纳战机）** | ROM 内字符串 `"1990 @TOKYO SHOSEKI / PROGRAMED BY COMPILE`；开场剧情 "A NATION CONSISTING OF SEVEN…"；维基百科中文条目即名"加纳战机" |
| 开发/发行 | Compile 开发，1990；日版 Tonkin House / 美版 ASCII | Wikipedia / GameFAQs |
| 类型 | 竖版卷轴 STG，8 关，5 主武器 + 4 炸弹 + 金钱商店系统 | Wikipedia Gameplay |
| ROM 格式 | iNES，PRG 128KB + CHR 128KB，**Mapper 4 (MMC3)**，无 trainer | 文件头 `4E 45 53 1A 08 10 41`，总长 262160 = 16 + 256KB |
| 版本 | 中文汉化版（九班汉化），CHR 内含中文字模 | CHR 图块渲染可见汉字（见 `shots/`、`_chr_bank0.png`） |

参考模拟器运行截图：`shots/auto_f*.png`（Mesen2 自动采集）。

## 1. 业界有没有人干过"ROM → PC 原生游戏"？

有，而且已经形成三个成熟流派（互相可以组合）：

### 流派 A：匹配反编译 → 移植（decomp → port）

把二进制**逐函数反编译成可编译的 C/C++ 源码**，编译产物与原版逐指令/逐字节对齐（matching decompilation），然后在源码层换掉渲染/音频/输入后端。

- 代表项目：
  - **Super Mario 64**（2019 年 n64decomp，业界标志性事件）、塞尔达 OoT/Majora's Mask、Mario Kart 64、Banjo-Kazooie 等 N64 decomp 系
  - **devilution**（Diablo 1 反编译）→ **devilutionX**（跨平台重实现）
  - **re3 / reVC**（GTA III / Vice City）
  - **OpenLara**（Tomb Raider 1–3）、**The Force Engine**（TIE Fighter）、**Cannonball**（OutRun 引擎级重实现）
  - NES 层面：SMB1（smbdis）、SMB3、Metroid、Zelda1 等完整反汇编项目
- 特点：产物真的是"源代码"，可维护可改；代价是人力密集（大游戏动辄数千小时），社区用 decomp.me、asm-differ、splat、mips_to_c 等工具协作。

### 流派 B：静态重编译（static recompilation）

不追求人类可读源码，把机器码 **lift 成中间表示 → 生成 C/C++ → 重新编译成原生代码**，配上平台 HLE 层（图形/音频/输入）。

- 代表项目：
  - **N64Recomp**（工具）→ **Zelda 64 Recompiled**、Mario 64 系重编译
  - **2s2h**（Majora's Mask PC 移植）、**PS1 Recompiled**（恶魔城 月下夜想曲 等）
- NES 6502 上**没有 N64Recomp 级别的现成工具**，但 6502 只有 56 条指令、无缓存/MMU、bank 切换是显式写寄存器，自建 lifter 可行；通用 lifter 可参考 **rev.ng**（lift 到 LLVM IR 并可回编译）、Ghidra p-code 导出。
- 特点：一次投入、机械保真；但产物是"怪物代码"，改玩法/加特性困难。适合"像素级复刻"目标。

### 流派 C：黑盒行为重实现（engine reimplementation / clean-room）

不逐行翻译，而是把原版当成**行为规格**：录像、追踪、数据表逆向，在新引擎里重写。**OpenRA、OpenTTD、OpenRCT2、OpenMW**、Spelunky Classic HD、OpenTyrian、NXEngine（洞窟物语）都属此类。
- 特点：代码质量最好、最利于增强（宽屏、高帧率、modding）；风险是"手感漂移"，必须靠差分测试压住。

### 流派 D：资产抽取 + 新引擎重写（STG/小游戏最常见）

把 CHR 图块、调色板、关卡数据、音乐从 ROM 抽出来，玩法代码全新写。大量 "NES 游戏 Unity/Godot/SDL 重制" 走这条路。
- 特点：最快见效；对 Gun-Nac 这类"逻辑小、资产多"的游戏性价比最高。

## 2. 本项目推荐方案：混合路线（C+D 为主，A/B 作规格后盾）

**"逆向做规格恢复 + 资产抽取 + Rust 原生重写 + 差分验证"**：

```
P0 工具链与基线     Mesen2 + IDA + Ghidra + 截图/录像管线跑通（本期已基本完成）
P1 规格恢复        逆向数值与规则：武器等级、速度、冷却、判定盒、掉落表、商店价格、
                   敌人生成脚本、Boss 血量、关卡时间线（IDA 静态 + Mesen2 trace 动态）
P2 资产管线        CHR→PNG 图块/调色板/动画帧；关卡地图（nametable/滚动表）；
                   音乐（NSF/APU 寄存器流）；中文字模与文本表
P3 Rust 引擎骨架    固定 60Hz tick（手感基础！）、tile/sprite 渲染、输入、碰撞、对象池
P4 内容移植        逐关、逐 Boss、逐武器；数据表驱动（TOML/RON，数值直接抄逆向结果）
P5 差分验证        同一输入录像（input movie）喂给"参考模拟器(Mesen2) vs Rust 版"，
                   逐帧比对画面/关键 RAM 状态；golden frame 回归测试
P6 PC 化增强        宽屏、高分辨率、任意分辨率 UI、成就、重映射、可选 60fps+平滑
```

核心原则：

1. **先规格后代码**：数值和规则全部从 ROM/运行时证据里来，不拍脑袋。IDA 看静态结构（表、指针、bank 布局），Mesen2 trace 看动态行为（每帧谁在动、写哪些地址）。
2. **固定时间步 60Hz**：NES 的一切手感都是"每帧 1/60 秒"的离散语义，Rust 版逻辑 tick 必须锁 60Hz，渲染可插值/放大。
3. **数据驱动**：敌人属性、生成表、掉落表做成外置数据表，便于 diff、便于后续 mod。
4. **参考模拟器当 oracle**：Mesen2 是我们唯一的"真值来源"，自动化采集/回放管线本期已打通。
5. **忠实 vs 修正分离**：精灵 8/行限制、闪烁、慢动作掉帧等"硬件事故"默认复刻（保手感），但做成开关。

## 3. 工具清单

### 3.1 静态逆向

| 工具 | 用途 | 本机状态 |
|---|---|---|
| **IDA Pro 9.1** | 6502 反汇编（`procs/m65.so` 支持 "MOS Technology 6502 / 65C02"）、数据表/指针/bank 布局分析；已装 **ida_mcp 插件**（AI 可直接驱动 IDA） | ✅ `/home/peng/ida-pro-9.1` |
| IDA Hex-Rays | ❌ **不支持 6502**（本机插件只有 hexx64/hexarm/hexmips/hexppc/hexrv/hexarc） | — |
| **Ghidra** | 6502 有 SLEIGH 规格，反编译器是架构参数化的 → **能出 6502 C 伪代码**（NES 逆向社区主力方案） | 建议安装 |
| Binary Ninja / radare2 | 备选；BN 有 6502 插件 | — |
| IDA loader | 无现成 iNES loader，需按 raw binary 手工分段（PRG@0x8000/0xC000 + MMC3 bank 窗口） | — |

### 3.2 动态分析（真值来源）

| 工具 | 用途 | 本机状态 |
|---|---|---|
| **Mesen2 2.1.1** | 调试器、trace logger、事件查看器（PPU/sprite/tile/IRQ）、内存断点、savestate、**Lua 自动化** | ✅ `tools/mesen2/` |
| Mesen2 Lua API | `setInput`（注入输入）、`takeScreenshot`/`getScreenBuffer`（导出画面）、`addMemoryCallback`（内存断点）、`addEventCallback`（帧/输入事件）、`stop` | ✅ 已验证 |
| 自动化装载 | **`.lua` 路径作为命令行位置参数**：`Mesen rom.nes script.lua /DoNotSaveSettings`（需在 settings.json 打开 `AllowIoOsAccess`） | ✅ 已验证 |
| FCEUX / BizHawk | 备选；BizHawk 的 TAStudio/BK2 录像对输入序列回放有用 | fceux 2.5.0 打包版在本机崩溃，弃用 |

### 3.3 资产抽取

- CHR 图块：自写脚本已可导出（`_chr_bank0.png`）；图形工具 **YY-CHR / NES Screen Tool / Tilemap Studio**
- 调色板 / 动画帧：Mesen2 Palette/Tile/Sprite Viewer
- 音乐音效：**FamiTracker / 0CC-Famitracker** 导入 NSF；或保留 APU 寄存器流用软件 APU 渲染（`nes-apu` 类库）；FamiTone 类工具可导出到自研引擎
- 文本：中文字模在 CHR，文本指针表需逆向（IDA 静态 + Mesen2 内存回调动态定位）

### 3.4 文档

- **NESdev Wiki**（mapper 4/MMC3 bank 与 IRQ、PPU 时序、APU、控制器）——逆向 NES 的"官方圣经"
- Disch's Documents、Anomie's Documents（PPU/APU 细节）
- Gun-Nac 攻略/资料（关卡与道具、商店经济设定校对）

### 3.5 Rust 技术栈建议

- 渲染/窗口：**bevy**（全家桶）或 **sdl2/macroquad + wgpu**（更可控）
- 音频：**cpal**（底层）/ **kira**、oddio（混音）
- 数据表：**serde + TOML/RON**；数学：**glam**
- 测试：criterion（性能）、golden image 比对（`image` crate 逐像素 diff）
- 参考实现形态：`logic` crate（纯逻辑，no_std 可选，可差分测试）+ `app` crate（平台壳）

### 3.6 验证方法（防"手感漂移"的保险）

1. **输入录像回放**：录制（帧号, 输入位）序列 → Rust 版与 Mesen2 各跑一遍
2. **逐帧画面 diff**：截图/`getScreenBuffer` 像素比对（容忍调色板/滤镜差异，先比"实体位置"再比像素）
3. **状态探针**：Lua `addMemoryCallback` 把 RAM 关键地址（玩家坐标、弹数、敌人槽位）导出成 CSV，与 Rust 版同名状态对齐
4. **golden frame 回归**：每个关卡/系统固定输入脚本 + 基准截图，CI 里跑

## 4. Gun-Nac 特定难点

1. **MMC3 bank 切换 + IRQ 扫描线**：原版用 IRQ 做 HUD/背景不同速滚动（split scroll）。重写不需要模拟 MMC3，但要读懂它产生的视觉效果并等价实现。
2. **数值体系**：5 武器 × 5 级、4 炸弹、金钱经济（关后商店）、Wing 加血机制、掉落表 —— 全部要逆向成数据表。
3. **敌人/弹幕行为**：Compile 系 STG 的敌人是"生成脚本 + 行为 ID"驱动，关卡时间线是核心资产。
4. **硬件限制的人为痕迹**：8 sprites/scanline、闪烁、bandwidth 限制导致的掉帧 —— 保留还是修掉要做产品决策。
5. **汉化文本与中文字模**：字模在 CHR，文本可能改了排版/指针，需单独逆向。
6. **音乐**：APU 寄存器级曲子，要么软件 APU 原样渲染（保真），要么重编曲（好听但不同）。

## 5. 工作量粗估

STG 是所有类型里最适合重写的：无复杂物理/寻路/文本系统，逻辑集中在"对象池 + 弹幕 + 关卡时间线"。

| 里程碑 | 内容 | 粗估 |
|---|---|---|
| M0 | 工具链 + 自动采集/回放管线（本期基本完成） | 1 周 |
| M1 | 引擎骨架：60Hz tick、tile 渲染、输入、玩家移动/射击手感对齐 | 2–3 周 |
| M2 | 武器/炸弹/道具/商店系统 + 第 1 关全内容 | 3–4 周 |
| M3 | 其余 7 关 + Boss + 结局 | 6–10 周 |
| M4 | 差分验证体系 + PC 化增强（宽屏、UI、音频重制） | 3–4 周 |

## 6. 合规提示

ROM 与美术/音乐版权归属 Compile 系（现行权利方多为 City Connection/Success 一系）；九班汉化本身也是未授权衍生。个人学习/内部研究没问题；**公开发布（尤其带原素材）前需要权利评估**。若要发布，"clean-room 重写 + 自制素材"是更安全的路线，或先联系权利方（Gun-Nac 商标/IP 目前无人积极运营，也有谈授权的现实可能）。

## 7. 本期已落地的基建

- `AGENTS.md` —— AI 纪律总纲（证据分级/门禁表/角色纪律，移植自 xiyou_remake）
- `prompts/` —— 四角色提示词（decomp-analyst / spec-writer / porter / verifier）
- `gates/README.md` —— 门禁设计（ratchet 基线、SKIP 显式、白名单唯一例外）
- `docs/ai-decompile-pipeline.md` —— AI 全自动反编译管线（S0–S6 + 完备性驱动回路）
- harness 技能 `remake-decomp-analyst` / `remake-spec-writer` / `remake-porter` / `remake-verifier`
  （`~/.agents/skills/`，供 harness 稳定调用角色契约）
- `rom/gunnac_cn.nes` —— ROM 可写副本（sha256 `4d76901c…`）
- `tools/mesen2/` —— Mesen2 2.1.1（Linux x64）
- `tools/mesen_capture.lua` —— 全自动采集脚本（输入注入 + 定时截图 + 定时退出）
- `shots/` —— 运行截图证据
- `_chr_bank0.png` —— CHR 图块渲染（中文字模可见）
- Mesen2 配置已放开 Lua 的 IO 权限（`AllowIoOsAccess=true`）
