# 基于 AI 的全自动反编译管线（Gun-Nac 版）

> 设计目标：把"ROM → 语义化规格 → Rust"的全过程变成**可自动迭代的回路**，
> AI 干活、门禁裁判、人只看门禁红灯和白名单。
> 纪律与红线以 [AGENTS.md](../AGENTS.md) 为准；本文只讲管线结构。

## 1. 与 xiyou_remake 的关键差异（先想清楚再抄）

| 维度 | xiyou（J2ME） | 本项目（NES 6502） | 应对 |
|---|---|---|---|
| 反编译可信度 | CFR/Vineflower 有背书，bytecode→Java 可信 | **没有可信 6502 反编译器**（Hex-Rays 不支持 6502）；Ghidra 伪代码是"投影" | 反编译产物一律降级为线索；**真值 = ROM + Mesen2 执行** |
| 参考版可运行 | 反混淆 Java + shim 可直接跑（自研 oracle） | 反汇编/伪代码不能直接跑 | oracle 用**原 ROM 跑在 Mesen2**；另建 **L2 子程序级差分**（6502 CPU 核直跑原字节，见 §5） |
| 信息形态 | Java 源码结构清晰 | 6502 平铺内存 + bank 切换 + 数据/代码混排 | S1 先做 bank 图与代码/数据分离；LLM 擅长 6502 惯用法识别，正好补位 |
| 难点分布 | 搬运 Java→Rust | **反编译本身**是主成本，搬运反而简单（STG 逻辑小） | 管线重心在 S1–S4；完备性矩阵盯"还剩多少没反编译" |

## 2. 真值层级（所有角色共用的心智模型）

```
第 0 层 真相    rom/ 的字节 + Mesen2 执行行为（oracle）        ← 永不修改，门禁校验 sha256
第 1 层 投影    IDA 反汇编 listing / Ghidra 伪代码 / trace 导出  ← 可再生，冻结副本进 reference/seed/
第 2 层 语义    命名台账 + docs/spec/ 行为规则                   ← AI 再创作，必须带证据指针
第 3 层 实现    crates/ Rust                                    ← 由差分测试裁定等价性
```

冲突裁决永远向下：spec 与 trace 冲突 ⇒ spec 错；Rust 与 oracle 冲突 ⇒ Rust 错；
投影与 ROM 冲突 ⇒ 投影错（重生成）。

## 3. 阶段（S0–S6）

| 阶段 | 内容 | 主要工具 | 产物 | 阶段门禁 |
|---|---|---|---|---|
| **S0 ROM 取证** | iNES 头、MMC3 bank 图、向量表（RESET/NMI/IRQ）、字符串/文本表定位 | python + hexdump | `docs/findings/rom-forensics.md` | rom-integrity 建立 |
| **S1 静态投影** | IDA（`m65`）建库 + 自动分析；Ghidra analyzeHeadless 出伪代码；bank 窗口/跳转表/数据表边界标注 | IDA 9.1 + ida_mcp、Ghidra | `reference/seed/gunnac.asm`（冻结）、`analysis/ida/`、`analysis/ghidra/` | seed-integrity 建立 |
| **S2 动态证据** | Mesen2 Lua harness：输入回放、trace（PC/寄存器/内存读写）、逐帧 RAM dump、截图 | Mesen2 2.1.1（已验证自动化接口） | `oracle/scenarios/*.txt`、`analysis/traces/` | 场景可复现 |
| **S3 符号与数据表恢复** | LLM 逐子程序考证语义命名；数据表（武器/敌生成/商店价格/弹速）识别与字段切分 | decomp-analyst 角色 + 台账 | `data/naming/`、`docs/findings/` | naming-ledger |
| **S4 规格书** | 行为规则提炼：移动/射击/武器升级/道具/金钱/关卡时间线/Boss 血量 | spec-writer 角色 | `docs/spec/`、`constants.md` | docs-links/format |
| **S5 Rust 移植** | game-data 解析 → game-core 纯逻辑（60Hz 固定 tick）→ 平台壳 | porter 角色 | `crates/` | rust-tests、L1–L3 |
| **S6 差分验证** | 子程序级 + 帧级 + 视觉级差分；完备性收敛 | verifier 角色 + oracle | `data/status/baselines/`、bug 台账 | trace/visual/all |

## 4. 全自动回路（"AI 全自动"的具体形态）

```
        ┌──────────────────────────────────────────────────────┐
        │ 完备性矩阵（分母独立于测试，机械导出）                    │
        │  符号面: IDA/Ghidra 子程序全集  数据面: 表+xref          │
        │  行为面: 场景/武器/关卡清单      规则面: spec 条目覆盖    │
        └──────────────┬───────────────────────────────────────┘
                       ▼ 每轮取"未覆盖且依赖已清"的一格
        ┌──────────────────────────────────────────────────────┐
        │ 任务卡队列（prompts/README.md 模板）                    │
        │  decomp-analyst → spec-writer → porter → verifier     │
        │  （角色任务可并行，写路径互斥；依赖用任务依赖表达）        │
        └──────────────┬───────────────────────────────────────┘
                       ▼ 产出落仓库（台账/spec/代码），不落聊天
        ┌──────────────────────────────────────────────────────┐
        │ 门禁裁定（gates/cli.py）                                │
        │  绿 → 矩阵标记覆盖（带证据指针），进入下一轮              │
        │  红 → 四分类路由：移植错→porter / 规格缺口→spec-writer / │
        │       参考投影错→decomp-analyst / 平台差异→白名单(需人确认)│
        └──────────────────────────────────────────────────────┘
```

- **人只出现在三个位置**：白名单确认、门禁恒红的裁决、里程碑验收。
- **防钝化三约束**（移植自 xiyou completeness 方法论）：新用例准入要声明覆盖面；
  覆盖只增不减；每轮报告给出"下一最优目标"。
- **绿 ≠ 完整**：测试全绿只说明"声明过的问题没回归"；还剩多少没反编译看矩阵，不看测试。

## 5. 差分金字塔（验证层，L2 是 NES 特有的强武器）

| 层 | 对象 | 方法 | 裁定物 |
|---|---|---|---|
| L1 | 资产解析 | CHR/调色板/关卡表 golden test（解析产物 vs 手工核对基准） | `data/golden/` |
| **L2** | **子程序级** | **6502 CPU 核 harness 直接执行 ROM 原字节**：给定入参 RAM/寄存器快照，跑单个子程序到 RTS，比对出参/内存写集 vs Rust 同名函数 | `oracle/subroutines/` |
| L3 | 帧级 | 同输入序列 + 固定 RNG，逐 tick 比对状态向量（玩家坐标/弹数/敌槽位/金钱） | oracle trace vs game-core |
| L4 | 视觉级 | 截图逐像素（调色板映射白名单外） | `emu.takeScreenshot` vs Rust 帧缓冲 |
| L5 | 流程级 | 全关卡自动跑（通关/死亡/商店买卖/存读档） | 场景脚本全绿 |

L2 的价值：把"一个函数是否移植正确"从"整局游戏跑出来才知道"降为**毫秒级单测**，
这是 6502 无 MMU、子程序边界清晰带来的独特红利（J2ME 项目做不到）。
注意隔离：子程序里若有 PPU/APU 寄存器访问，harness 需 stub 这些地址的读写行为。

## 6. 工具映射（本机已验证状态）

| 能力 | 工具 | 状态 |
|---|---|---|
| 6502 反汇编 | IDA 9.1 `m65.so`（MOS 6502/65C02）+ **ida_mcp**（AI 直接驱动 IDA） | ✅ 本机 `/home/peng/ida-pro-9.1` |
| 6502 伪代码 | **Ghidra analyzeHeadless**（SLEIGH 反编译器，架构参数化） | 待装（Hex-Rays 无 6502，别指望） |
| 动态 trace / 输入注入 / 截图 / 内存回调 | **Mesen2 2.1.1 Lua**：`.lua` 位置参数装载、`setInput`/`takeScreenshot`/`addMemoryCallback`/`addEventCallback`/`stop` | ✅ `tools/mesen_capture.lua` 已跑通 |
| 子程序级执行 | 自研 6502 CPU 核 harness（无 PPU 的纯 CPU 核 ~500 行，或 `mos6502` 类 crate） | 待建 |
| 资产抽取 | CHR→PNG 脚本（已跑通）、YY-CHR、Tilemap Studio、FamiTracker | ✅ 部分 |
| 文档 | NESdev Wiki（MMC3/PPU/APU）、Disch/Anomie docs | — |

## 7. 命名台账 schema（两层，移植自 xiyou）

- **机械层** `data/naming/remap-table.json`：确定性重映射（`sub_9A2C`→`f_042`、`unk_0320`→`v_0320`），
  不做语义判断，由 `tools/remap/run.py` 生成与校验。
- **语义层** `data/naming/ledger.jsonl`：一条 rename 一条记录：

```json
{
  "id": "N-XXXX", "kind": "subroutine|ram|table|const",
  "old": "<原始机械名>", "ref": "<机械层条目 id>",
  "new": "<语义名>",
  "evidence": "<可复现指针：bank:addr / 字节偏移 / trace id / 命令>",
  "confidence": 0, "level": "A|B", "date": "YYYY-MM-DD", "status": "applied|hypothesis|reverted"
}
```

> ⚠️ **以上为格式示例，字段值全部是虚构占位**，不代表任何真实符号的考证结论；
> 引用证据时必须回到实际取证产物（汇编投影 / trace / hexdump），禁止把示例数据当证据。

规则与 xiyou 相同：A/B 级才能 `applied`；C 级一律 `hypothesis`（代码用 `_todo_` 前缀）；
`evidence` 必须含可复现指针（`bank:addr` / 字节偏移 / trace id / 可重跑命令）；证伪追加 `reverted` 不删历史。

## 8. 风险登记

| ID | 风险 | 对策 |
|---|---|---|
| R1 | MMC3 bank 切换 + IRQ 扫描线（split scroll/HUD） | S1 先建 bank 图；spec 里行为描述与 bank 实现解耦；Rust 侧不需要复刻 MMC3，只需复刻视觉效果 |
| R2 | 数据/代码混排，自动反汇编有噪声 | IDA/Ghidra 双投影交叉验证 + trace 确认执行过该地址 |
| R3 | L2 harness 中 PPU/APU 依赖 | stub 寄存器行为；有副作用的子程序降级到 L3 验证 |
| R4 | 汉化文本/字模是二次修改产物 | 文本表单独考证；与日/美版差异记 findings，不混入"原版行为"结论 |
| R5 | 音乐（APU 寄存器流）保真成本 | Stage A 用软件 APU 原样渲染或先静音占位（记 scope-decision）；Stage B 重制 |
| R6 | "AI 全自动"产出幻觉（编造行为） | 证据分级纪律 + 门禁裁定 + 完备性矩阵防"绿=完整"错觉 |
