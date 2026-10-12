# gates/ — 门禁工具链

单一真相源是 `gates/cli.py`（AGENTS.md §5 的门禁表即其命令映射）。
**禁止**内嵌第二套扫描逻辑、禁止绕过/弱化/删除门禁；改门禁必须单独成 commit 并写明基线影响。
设计原则移植自 xiyou_remake（PAL3Decomp 教训）。

## 命令

| 命令 | 作用 | 何时跑 |
|---|---|---|
| `python3 gates/cli.py status` | 只读预检：门禁状态 + 基线对照（**动手前必跑**） | 每次会话开工 |
| `python3 gates/cli.py check` | 总闸（pre-commit 自动调用） | 任何 commit |
| `python3 gates/cli.py rom` | ROM sha256 完整性 | 怀疑 `rom/` 被改 |
| `python3 gates/cli.py seed` | `reference/seed/` 投影完整性（D3 冻结基准） | 怀疑投影基准被改 |
| `python3 gates/cli.py naming` | 命名台账证据完整性 | 改 `data/naming/` 后 |
| `python3 gates/cli.py trace` | 差分 L1–L3（含子程序级 L2） | 改 game-data/game-core 后 |
| `python3 gates/cli.py visual` | 视觉 L4 | 改渲染后 |
| `python3 gates/cli.py all` | 全量 | 里程碑收尾 |

## 门禁清单（随阶段启用，SKIP 显式打印）

| 门禁 | 内容 | 状态 |
|---|---|---|
| `rom-integrity` | `rom/` sha256 对照 SHA256SUMS | 启用（P0） |
| `seed-integrity` | `reference/seed/` 反汇编投影 sha+行数对照 baseline | S1 交付后启用 |
| `assets-manifest` | CHR/PRG 资产清单 ratchet（对照 baseline.json） | 启用 |
| `naming-ledger` | 每条 rename 带 evidence 可复现指针 | 台账创建后自动启用 |
| `knowledge-format` | 知识库 frontmatter/type/文件名前缀 | 启用 |
| `docs-links` | 文档相对链接有效性 | 启用 |
| `rust-tests` | `cargo test --workspace` + 测试数 ratchet | crates/ 创建后自动启用 |
| `subroutine-diff` | L2：ROM 原字节执行 vs Rust 函数出入参比对 | oracle/subroutines 建立后启用 |
| `trace-diff` | L3：帧级状态向量逐 tick 比对（oracle vs Rust） | S6 启用 |
| `visual-diff` | L4：截图逐像素（白名单外） | S6 启用 |
| `completeness-ratchet` | 完备性矩阵覆盖只增不降 | 矩阵建立后启用 |

## 设计原则

1. **SKIP/TOLERATED 显式打印**：静默通过的门禁等于没有门禁。
2. **基线 ratchet**：`data/status/baselines/baseline.json` 只升不降；更新基线必须在 commit 中说明
   "确认无回退"的证据。
3. **失败先定位层**（AGENTS.md §4）：门禁红 ≠ 反复重跑；也 ≠ 调阈值/加白名单。
4. **白名单唯一例外**：`gates/visual_whitelist.json`（S6 建立），变更需用户确认。
5. **门禁命令必须可本地复跑**：CI 与本地 `check` 完全等效，不许存在"只有 CI 会跑"的检查。

## hooks

```bash
bash gates/install-hooks.sh   # 挂接 pre-commit + commit-msg
```

- `pre-commit` → `gates/cli.py check`
- `commit-msg` → 触及 `data/naming/`、`reference/`、`rom/` 的提交必须含 `Evidence`/`Confidence`
