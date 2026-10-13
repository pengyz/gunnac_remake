# AGENTS.md — AI 纪律总纲（单一真相源）

> 面向所有 AI 实例（DSH/Claude Code/Codex/其他）。本文件是纪律的**唯一权威**；
> 其他文档与本文件冲突时，以本文件为准并提交更正。方法论背景见
> [docs/methodology.md](docs/methodology.md)，自动化管线设计见
> [docs/ai-decompile-pipeline.md](docs/ai-decompile-pipeline.md)。
> 纪律框架移植自 xiyou_remake（PAL3Decomp 教训），按 NES/6502 场景适配。

## 0. 项目与已锁定决策

《加纳战机 / Gun-Nac》（Compile 1990，NES，Mapper 4/MMC3，PRG 128K + CHR 128K，
九班汉化中文版）重制：**基于 AI 的全自动反编译** → 理解 → **Rust 改写为 PC 原生游戏** → 差分验证。
两阶段：**Stage A 行为一致** → **Stage B 现代化**。

| # | 决策 | 内容 |
|---|---|---|
| D1 | 语言 | Rust（桌面优先，预留 wasm）。C++/TS 已否决 |
| D2 | 路线 | 先行为一致后现代化；A 阶段预留扩展缝（分辨率/输入/音频/存档/UI） |
| D3 | 锚点 | **ROM 字节 + Mesen2 执行是最终真相**（`rom/` 永不修改，sha256 门禁校验）。IDA/Ghidra 反汇编与伪代码是**可再生投影**，不是真相；`reference/seed/` 投影冻结为基准 |
| D4 | Bug | Stage A **保真保留原版 quirks**（8 sprites/scanline、闪烁、掉帧、溢出），只登记 [docs/bug-ledger.md](docs/bug-ledger.md)（`BUG-xxx`），Stage B 修复 |
| D5 | 反编译路线 | 6502 **没有 Hex-Rays**：Ghidra（6502 SLEIGH 反编译器）出 C 伪代码 + IDA（`m65` 模块）静态分析/数据表/xref + Mesen2 动态 trace 三方交叉；**LLM 负责语义化，判定权在门禁** |

## 1. 哲学（决策优先级）

**逻辑一致性 > 代码优雅 > 工具便利 > 个人偏好。**

- **裁判是差分测试，不是"看起来对"**。6502→Rust 搬运的正确性由 trace/状态快照/截图 diff 判定；
  人肉 review 只能发现风格问题，不能证明等价。
- **机器码是最终真相，反汇编/伪代码只是投影**。反编译是有损的：我们不是"恢复"原源码，
  而是再创作一个行为完全一致的程序。任何投影与 ROM 行为冲突时，以 ROM 为准。
- **参考轨道永久存活**：`rom/` 与 Mesen2 oracle 一切验证的基准，永不删除、永不迁就 Rust 侧。
- **保真期的行为只增不减**：原版有的行为（含 bug）必须保留；原版没有的行为（含修复）禁止引入。

## 2. 仓库结构与红线

| 路径 | 职责 | 红线 |
|---|---|---|
| `rom/` | 原始 ROM 只读归档（唯一资产真相源） | **任何字节修改 = 严重违规**；`gates/cli.py rom` 校验 sha256 |
| `reference/seed/` | 冻结投影：规范化 6502 反汇编 listing | 不可变基准（类似 xiyou 的 seed/a.java），重生成需 commit 说明 |
| `reference/src/` | 命名/注释后的参考汇编轨道（可汇编、可比对） | 只允许语义等价的改名/注释/拆分；行为变化必须差分证据不变 |
| `oracle/` | Mesen2 自动化宿主：Lua harness、输入回放、trace/RAM dump、截图 | 比对逻辑不得为"通过测试"放宽阈值 |
| `analysis/` | 可再生工作区（IDA idb、Ghidra project、trace 导出、CHR 渲染） | 不入库；由 `rom/` + 工具可复现 |
| `docs/spec/` | 规格书（P2 产物） | 每条结论必须带证据指针（汇编地址/字节偏移/trace id） |
| `docs/findings/` | 考证记录 | **必须区分"已证实 / 假设"**，假设不得混入结论 |
| `docs/knowledge/` | 跨会话共享知识库 | 写入规则见其 README；引用前必须验证仍存在 |
| `data/naming/` | 命名台账（rename 证据账本，两层：机械 remap + 语义 ledger） | 每条 rename 必须有 evidence 字段（可复现指针） |
| `data/golden/`、`data/status/baselines/` | golden 与门禁基线（ratchet） | 基线只升不降，更新必须写 commit 说明 |
| `crates/` | game-data（资产解析）/ game-core（纯逻辑）/ game-platform（trait）/ game-oracle（差分宿主） | game-core 无 I/O、确定性（虚拟时钟 + GameRng）；禁止平台依赖 |
| `gates/` | 门禁工具与 git hooks | 不得绕过/弱化/删除门禁；改门禁必须单独 commit 并写明基线影响 |
| `prompts/` | 角色提示词 | 新任务必须先选角色 prompt（见 §7） |
| `assets/` `target/` | 可再生工作区 | 不入库 |

## 3. 证据纪律（本项目最核心的纪律）

证据分级（与台账/commit message 中的 `Confidence` 配套）：

| 级 | 定义 | 例 |
|---|---|---|
| **A** | 硬证据：字节偏移、汇编行（bank:addr）、Mesen2 trace、RAM dump、CHR hexdump | `bank3:$9A2C` 处 `STA $0320`；trace 显示每帧写 `0x0320`；CHR tile 表偏移 `0x1A000` |
| **B** | 代码语境 + 调用图/数据流推断（可复现的静态推理） | 子程序只被 NMI 路径调用 ⇒ 帧同步逻辑；数据表两端被同一索引寄存器寻址 |
| **C** | 直觉/剧情猜测/命名相似 | "这个表可能是弹幕速度表" |

规则：

1. **A/B 级才能落台账、改名、写入 spec 结论**；C 级只能进 findings 的"假设区"，禁止据此 rename。
2. **禁止伪语义命名**：`table_0320`、`enemyThing2` 这类地址/序号后缀伪装的名字没有语义价值。
   不确定含义就保留 `_todo_`，并在台账记 C 级假设。
3. **禁止猜测性格式结论**：数据表/资源格式的"已证实"必须能给出字节级复现命令（hexdump/python 一行验证）
   或运行时 trace 佐证。
4. **禁止批量未验证 rename**：一个符号一条台账记录，证据独立可查。
5. **禁止试错式修改**：不理解机制前不动解析器/核心逻辑。先考证（hexdump → 反汇编 → Mesen2 插桩 trace），
   画出完整使用图景，再动手。
6. **证伪更正文化**：结论被推翻后，在原文**原地标注日期更正**（`> 【2026-10-12 更正】…`），
   不删除历史主张。

## 4. 动手前三问（跨层修改的强制预检）

修改任何跨层内容（ROM 投影/参考轨道/oracle/核心逻辑/门禁/台账规则）前，必须**只读地**回答，
并把答案写进 commit message：

1. **问题在哪一层？**（反汇编投影错？数据表理解错？Rust 移植错？oracle/门禁本身错？）
2. **消费者实际读的是哪个文件/哪份数据？**（解析器读 ROM 原始字节还是转换产物？golden 从哪来？）
3. **证据是什么级别？**（答不出 A/B 级证据 ⇒ 不许动手，先去取证。）

## 5. 强制检查点（门禁表）

先跑只读预检 `python3 gates/cli.py status`。**SKIP 与 TOLERATED 必须显式打印，绝不静默通过。**

| 时机 | 门禁 | 命令 | 失败处理 |
|---|---|---|---|
| 动手前 | 状态预检 | `python3 gates/cli.py status` | 看清 SKIP/恒红项再动 |
| **任何 commit** | 总闸（pre-commit 自动） | `python3 gates/cli.py check` | 任一 FAIL ⇒ 不提交 |
| 改 `data/naming/` `data/status/` `reference/` `rom/` | 证据门禁（commit-msg 自动） | commit message 含 `Evidence`/`证据` | 缺证据字段 ⇒ 拒绝 commit |
| **任何步骤完成后** | **独立对抗复核**（2026-10-12 用户裁定） | 审核 agent（verifier 角色，**不得为实施方**）以**证伪立场**独立重导证据：重跑复现命令/重推证据链/查纪律合规（证据指针、未证不发明、门禁真绿） | **复核 verdict=pass 才算步骤完成**；findings → 四分类路由修复 → 复审；未过复核的产出只算"待审稿" |
| 改参考投影/轨道后 | 投影回归 | `python3 gates/cli.py seed` | seed sha 不变；src 轨道 trace 不变 |
| 改 game-data/game-core 后 | 差分（L1–L3） | `python3 gates/cli.py trace` | 任何回退 ⇒ 停止，先定位层（§4） |
| 改渲染后 | 视觉（L4） | `python3 gates/cli.py visual` | 白名单外像素必须一致 |
| 里程碑收尾 | 全量 | `python3 gates/cli.py all` | 对照管线文档阶段 Gate |

> 【2026-10-12 更正】敏感路径增补 `data/status/`：t11 F1 攻击链暴露"降基线提交可机器无痕入库"，
> t14 修复已把实现纳入 commit-msg 强制，本表与 §9 同步跟进（gates/cli.py 为实现权威）。

> 【2026-10-12 新增·对抗复核纪律】**每个步骤完成后必须派出独立审核 agent 做对抗复核**，
> 确认方案合理性并修正可能问题，**只有复核通过才视作该步骤完成**（用户方法论更新）。
> 对抗立场=以"证伪它"为默认姿态：不采信被审方自述、不复用其结论、独立重导证据；
> 复核产出=verdict（pass/needs_revision）+ findings（四分类）+ 修正建议；
> 被审方与审核方不得为同一 agent（实施≠审核）。captain 验收与门禁裁定不变，三闸并行。

- **门禁失败不是"多跑几次"能解决的**：按 §4 定位层，修复根因，不许调阈值/加白名单掩盖。
- 视觉白名单（字体/调色板映射等）是**唯一**允许的例外清单，变更需用户确认并记录。

## 6. Bug 台账纪律（D4 的执行细则）

- 运行/差分中发现原版可疑行为 ⇒ 记入 `docs/bug-ledger.md`（现象/复现/影响/trace id），
  代码侧以 `BUG-xxx` 注释引用，**Stage A 不修**。
- 差分测试遇到已登记 bug：登记为"已知偏差"，基线显式 TOLERATED 并注明 BUG 编号。
- 禁止"顺手修 bug"——那会直接击穿一致性验证。

## 7. 提示词纪律

详见 [prompts/README.md](prompts/README.md)。要点：

- 新任务必须先选定角色 prompt（decomp-analyst / spec-writer / porter / verifier）；
- 任何 prompt 的产出必须包含：证据引用、完成判据（跑哪个门禁）、遗留假设清单；
- **产出的归宿是仓库**（台账/spec/代码/知识库），不是聊天记录——聊天里的结论不许作为后续依据；
- 全自动回路中任务卡由完备性矩阵派生（[docs/ai-decompile-pipeline.md](docs/ai-decompile-pipeline.md) §6）。

## 8. 知识库

`docs/knowledge/`：跨会话共享知识（6 类：architecture/gotcha/pattern/debug/decision/reference）。
遇到平台坑（Mesen2/IDA/Ghidra/6502 banking）、门禁失效、被纠正的方法、确认有效的取舍时写入；
**引用前必须验证**文件/命令仍存在。

## 9. 提交规范

```
<type>(<scope>): <subject>          # type: feat/fix/docs/kb/gate/refactor/test/chore
                                    # scope: rom/ref/oracle/data/core/platform/gates/docs/tools
<空行>
<正文：改了什么、为什么、证据与影响>
<触及 data/naming/ / data/status/ / reference/ / rom/ 时必须含：>
Evidence: <A/B 级证据描述>
Confidence: <0-100>
```

> 【2026-10-12 更正】敏感清单增补 `data/status/`（与 §5 同步，见该处更正注）。

小步提交：一个 commit 一个逻辑变更。门禁改动单独成 commit，不得与功能变更混提。
安全网：本地 `pre-commit`（gates check）+ `commit-msg`（证据门禁）实时拦截；CI 在 push/PR 时跑同等检查。
