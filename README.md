# gunnac_remake

《**加纳战机 / Gun-Nac**》（Compile, 1990, NES）的 Rust 重制工程 —— 采用 **AI 全自动反编译 + 门禁裁判** 的迭代流水线推进。

> 目标：不做模拟器。把 ROM 里的游戏逻辑反编译、理解、再创作成行为一致的 Rust 原生 PC 游戏。
> 终态（Stage A）：8 关可通关，与原 ROM（Mesen2 oracle）差分验证收敛 —— L1–L3/L5 全绿，L4 白名单外像素一致。

## 当前状态（2026-10-12）

| 维度 | 进度 |
|---|---|
| 里程碑 | **M0 基建 ✅ · M1 取证/投影 ✅** · M3 移植进行中（player/weapons/shots ✅ → enemy/stage） |
| 门禁 | **10/11 启用**（rom/seed 完整性、命名台账、L2 子程序级差分、L3 帧级差分、完备性 ratchet…仅剩 visual-diff） |
| 差分验证 | L2 **74 用例**三方全等（golden==ROM 原字节执行==Rust 函数）｜L3 帧级 1853 项全绿 |
| 完备性矩阵 | 48 格（行为面），覆盖 8；规则 50+ 条（全部带证据指针） |
| 命名台账 | 64 条（A/B 级证据落账，含证伪撤回流程） |

## 方法论一句话

**ROM 字节 + 模拟器执行是唯一真相；反汇编/伪代码只是投影；LLM 负责语义化，判定权在门禁。**

```
S0 ROM 取证 → S1 双投影（IDA+Ghidra）→ S2 oracle 场景化（Mesen2）
→ S3 符号/数据表考证 → S4 行为规格 → S5 Rust 移植 → S6 差分验证
     （每步产出落仓库、带证据指针；门禁裁定；四分类路由回流）
```

## 仓库结构

| 路径 | 内容 |
|---|---|
| `rom/` | 原始 ROM（入库，sha256 门禁校验，只读归档） |
| `reference/seed/` | 冻结的 6502 反汇编投影（D3 锚点，91,601 行） |
| `crates/` | `game-data`（解析）/ `game-core`（纯逻辑，确定性 60Hz）/ `game-platform`（trait 缝）/ `game-oracle`（6502 CPU 核 + 差分宿主） |
| `oracle/` | Mesen2 场景 DSL + 输入回放 + trace/RAM/截图采集 |
| `gates/` | 门禁 CLI（`python3 gates/cli.py all`）+ git hooks |
| `docs/` | 规格书 / 考证记录 / 知识库 / 轮次日志 / [总体方案](docs/master-plan.md) |
| `data/` | 命名台账、完备性矩阵、门禁基线（ratchet 只升不降） |
| `AGENTS.md` | AI 协作纪律总纲（证据分级 A/B/C、门禁表、提交规范） |

## 快速开始

```bash
# 1. 校验 ROM 完整性（入库副本，sha256 门禁锚点）
sha256sum -c SHA256SUMS          # 应显示: OK

# 2. 跑门禁总闸（应 PASS=10 FAIL=0 SKIP=1）
python3 gates/cli.py all

# 3. 跑测试与差分
cargo test --workspace            # L1 golden + L2 双实现对拍 + L3 回归
python3 tools/diff_subroutines.py # L2 三方裁定
```

## 工程纪律（详见 [AGENTS.md](AGENTS.md)）

- 证据分级：A（字节/trace 硬证据）/ B（可复现推理）/ C（猜测只进假设区）
- 反编译结论必须带可复现指针（`bank:addr` / 字节偏移 / trace id）
- 原版 bug/quirk **保真保留**（Stage A），只登记 [bug 台账](docs/bug-ledger.md)
- 门禁失败禁止调阈值/加白名单掩盖；白名单唯一例外需用户确认
- 证伪结论原地标注更正，不删历史

## 文档入口

- [总体方案与里程碑](docs/master-plan.md) · [方法论调研](docs/methodology.md) · [AI 反编译管线](docs/ai-decompile-pipeline.md)
- [完备性矩阵](docs/spec/completeness.md)（"绿 ≠ 完整"的度量）· [轮次日志](docs/rounds/)（流水线运转实录）
- [知识库](docs/knowledge/)（并行不可信家族、粒度陷阱、栈页采样盲区…已实证教训）

> 本项目为个人学习/研究用途。Gun-Nac 版权归原权利方，公开发布前需权利评估。
