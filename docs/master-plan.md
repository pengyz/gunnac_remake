# Gun-Nac 重制工程 —— 长线目标章程与总体方案（master-plan）

> 状态：2026-10-12 定稿 v1（用户指令"分析并把长线目标补充完善"的产物）。
> 纪律以 [AGENTS.md](../AGENTS.md) 为准；管线设计见 [ai-decompile-pipeline.md](ai-decompile-pipeline.md)；
> 方法论调研见 [methodology.md](methodology.md)。人类决策记录在本文件。

## 0. 长线目标（完善后陈述）

> **用 AgentTeams 团队架构持续推进《加纳战机》(NES) 的 AI 全自动反编译 → Rust 重制，
> 确定并运转迭代流水线（完备性矩阵驱动）；captain 每轮负责验收、进度检查、调度。
> 目标终态 = Stage A 行为一致**：Rust 版 8 关可通关，与原 ROM（Mesen2 oracle）差分验证收敛
> （L1–L3/L5 全绿，L4 白名单外像素一致），迭代流水线可无人值守连续运转。

## 1. 目标分解（三个动词的可检验定义）

| 动词 | 可检验定义 | 完成证据 |
|---|---|---|
| **确定迭代流水线** | 回路七步落文档 + 完备性矩阵 + 任务卡/轮次日志模板 + 门禁命令表 + 失败四分类路由，并**实际跑通至少 3 轮完整循环** | `docs/iteration-pipeline.md` + `docs/rounds/` 轮次日志 + 矩阵收敛记录 |
| **agent-teams 推进** | 团队常驻：analyst / spec-writer / porter / verifier 四成员（对应 4 个 skill），每轮由矩阵派发任务卡，产出落仓库 | 团队 roster + 任务历史 + 台账/spec/代码产出 |
| **captain 验收+进度检查+调度** | 每轮固定动作：①验收（跑门禁、抽查证据指针）②进度检查（矩阵/里程碑/风险）③调度（派发下一格任务） | 轮次日志中三项均有记录；需用户确认事项显式上报 |

**边界**：captain 不做实现、不批自己的产出；白名单/基线上调/范围决策必须用户确认（AGENTS.md §5）。

## 2. 里程碑（Goal 级路线图）

| 里程碑 | 内容 | 完成判据 | 状态 |
|---|---|---|---|
| **M0 基建** | 团队组建 + gates/cli.py 门禁骨架 + hooks + 完备性矩阵 v0 | `gates/cli.py status/check/rom` 可跑，矩阵 v0 建立 | 本轮启动 |
| **M1 取证与投影** | S0 ROM 取证、S1 IDA+Ghidra 双投影（seed 冻结）、S2 Mesen2 oracle 场景化 | rom-forensics 落档；seed 门禁启用；≥3 个可复现场景 | |
| **M2 核心系统** | S3–S5：玩家移动/射击、5 武器升级、道具/金钱/商店 —— 规格+移植+L2 差分 | 核心系统规格齐备；L2 用例全绿 | |
| **M3 全内容** | 8 关关卡时间线、敌人生成、Boss、汉化文本 | 全关卡可玩；矩阵四维持续收敛 | |
| **M4 = Stage A** | 差分收敛 + 通关验证 | L1–L3/L5 全绿、L4 白名单外像素一致、L5 全关卡自动通关 | |
| M5 = Stage B | 现代化（宽屏/高清/UI/音频重制/BUG 清偿） | **另立目标**，不在本 Goal 内 | — |

## 3. 迭代流水线（每轮循环，captain 驱动）

```
① 取格   从完备性矩阵取"未覆盖且依赖已清"的下一最优格
② 派卡   任务卡（角色 skill + 范围 + 证据输入 + 完成判据门禁），AgentTeams 派发
③ 执行   成员按角色契约工作，产出落仓库（台账/spec/代码/知识库）
④ 裁判   gates/cli.py 门禁裁定：绿→登记覆盖；红→四分类路由
          （移植错→porter | 规格缺口→spec-writer | 投影错误→decomp-analyst | 平台差异→白名单(需用户确认)）
⑤ 验收   captain：跑门禁贴输出 + 抽查证据指针（A/B 级）+ 核对产出契约模板完整性
⑥ 进度   更新矩阵/里程碑表/风险登记；写 docs/rounds/round-NNN.md 轮次日志
⑦ 调度   派发下一批（并行取互斥写路径的格）；blocked/失败项进下一轮队列
```

## 4. 团队编制（成员 ↔ 角色 ↔ skill ↔ 产出）

| 成员 | 角色 | harness skill | 任务类型 | 产出归宿 |
|---|---|---|---|---|
| decomp-analyst | 反编译分析员 | `remake-decomp-analyst` | work | `data/naming/` + `reference/` + `docs/findings/` |
| spec-writer | 规格作者 | `remake-spec-writer` | work/requirements | `docs/spec/` |
| porter | 改写工程师 | `remake-porter` | implementation/repair | `crates/` + `tools/` + `gates/` |
| verifier | 差分验证员 | `remake-verifier` | verification/review | `gates/` + `data/status/` + bug 台账 |
| **captain** | 调度+验收 | （本章程 §1/§3） | — | 轮次日志 + 进度台账 |

## 5. captain 验收协议（每轮必做，缺一不可）

1. **门禁**：对应任务的完成判据命令实际执行，输出贴入轮次日志（完成≠声称）。
2. **证据抽查**：产出中的 A/B 级证据指针抽样核实（地址/偏移/trace id 真实存在）。
3. **契约完整性**：产出模板字段齐全（Evidence/Confidence/门禁/遗留假设），缺失打回。
4. **矩阵纪律**：覆盖只增不降；绿≠完整，进度以矩阵为准。
5. **上报用户**：白名单变更、基线**下调/放宽**、范围/方向决策、恒红门禁 —— 一律显式上报，不自决。
   > 【2026-10-12 修订·用户裁定】"基线上调"细化为两类：**进度类上调**（调到现值 + `--reason`
   > 写明"确认无回退"证据 + commit 说明）由 **captain 验收后代执行并留痕**，无需逐次人工确认；
   > **下调/放宽/白名单**仍必须用户确认。首个执行案例：knowledge_entries 0→5（t9 验收后代执行）。

## 6. Goal 完成判据（整体达成时逐条核对）

- [ ] 迭代流水线文档定稿，且实际跑通 ≥3 轮完整循环（轮次日志为证）
- [ ] 完备性矩阵 v0 建立且四维（符号/数据/行为/规则）均有收敛记录
- [ ] Stage A：8 关可通关（L5 全绿），L1–L3 全绿，L4 白名单外像素一致
- [ ] `gates/cli.py all` 本地可跑且与 CI 等效；SKIP/TOLERATED 全部显式
- [ ] 台账（naming/bug）与知识库齐备，提交纪律（Evidence/Confidence）全程执行

## 7. 风险登记

| 风险 | 对策 |
|---|---|
| AI 幻觉产出（编造行为/证据） | 证据 A/B/C 分级 + 门禁裁判 + captain 抽查，三道闸 |
| 无人值守时回路空转 | 矩阵"下一最优格"机械可取；轮次日志强制记录，不写日志=轮次不算数 |
| 验证成本超前于移植 | L2 子程序级差分把验证粒度降到函数级（pipeline §5） |
| 法律/版权 | 仅内部研究；对外发布前权利评估（methodology §6） |
| 精力漂移（多线程开新坑） | 里程碑表是唯一进度真相；新需求进 Stage B 待办，不动 Stage A 范围 |
