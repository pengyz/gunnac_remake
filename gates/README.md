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
| `python3 gates/cli.py all` | 全量（所有门禁 + selftest） | 里程碑收尾 |
| `python3 gates/cli.py selftest` | 工具链自证：ratchet 只升不降 / 基线↔历史完整性（历史下界·快照覆盖·sha 链·状态锚点）/ commit-msg 证据门禁 / SKIP·TOLERATED 显式打印 / 注册表↔本文件清单 + 命令面双向相等 | 改门禁后；`all` 内含 |
| `python3 gates/cli.py baseline show\|update\|tolerate` | 基线管理：show=基线+历史；update=只升不降+强制 `--reason` 留痕；tolerate=登记显式 TOLERATED（强制 `--bug` BUG-xxx） | 基线/tolerance 变更 |
| `python3 gates/cli.py commit-msg <msgfile>` | commit-msg 钩子入口（证据门禁本体） | 钩子自动调用 |

> 锚点与基线位置：ROM sha256 锚点 = `rom/SHA256SUMS`（若存在）否则 `data/status/baselines/SHA256SUMS`
> （sha256sum 兼容格式）；ratchet 基线 = `data/status/baselines/baseline.json`，
> 更新留痕 = `data/status/baselines/baseline_history.jsonl`（每条必须带 reason，且携带全量
> `metrics` 快照 + `baseline_sha256` 链）。**历史文件是下界锚，禁止删除/清空**；新仓库 bootstrap
> 需先落 genesis 记录（带 reason + metrics 快照），否则 baseline-integrity 按『历史缺失』FAIL。

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
   "确认无回退"的证据。**手工改写/删历史机器可检出（t10 R1 + t14 F1 修复）**，缺一不可：
   - 唯一写入口 `baseline update --reason`（拒绝下调、拒绝低于历史下界、拒绝空说明）；
   - `baseline_history.jsonl` **存在性前置**：缺失/为空/损坏 ⇒ FAIL『历史缺失：基线单调性下界不可验证』
     （禁 vacuous 通过）；每条记录带全量 `metrics` 快照 + `baseline_sha256`（sha 链），
     任一 metric 无下界可依 ⇒ FAIL；『metric 现值 < 历史下界』⇒ FAIL『基线被手工调低』；
     『当前 baseline.json sha ≠ 历史链尾』⇒ FAIL 断链；损坏 JSON ⇒ 显式 [FAIL] 解析失败行（禁裸 traceback）；
   - `baseline.json` 与 `baseline_history.jsonl` 均纳入 `SHA256SUMS` sha 锚点（rom-integrity 同验：
     删/改任一状态文件即 FAIL）。
   非自洽/单侧改写不复存在（协调一致的多侧伪造仍靠 `data/status/` 证据门禁与评审把关）：
   污染状态上跑 `baseline update` 会在历史永久留下 `tamper_recovered` 痕迹；
   降基线的提交本身也走 `data/status/` 敏感路径证据门禁（见 hooks）。
3. **失败先定位层**（AGENTS.md §4）：门禁红 ≠ 反复重跑；也 ≠ 调阈值/加白名单。
4. **白名单唯一例外**：`gates/visual_whitelist.json`（S6 建立），变更需用户确认。
5. **门禁命令必须可本地复跑**：CI 与本地 `check` 完全等效，不许存在"只有 CI 会跑"的检查。

## hooks

```bash
bash gates/install-hooks.sh   # 挂接 pre-commit + commit-msg
```

- `pre-commit` → `gates/cli.py check`
- `commit-msg` → 触及 `data/naming/`、`data/status/`、`reference/`、`rom/` 的提交必须含
  `Evidence`/`Confidence` 字段（别名集合与本表严格一致：仅 `Evidence:`/`证据:` 与
  `Confidence:`/`置信度:`，不接受未文档化别名如『信心:』/『置信:』；缺失即拒）

敏感路径集合以 `gates/cli.py` 的 `SENSITIVE_PREFIXES` 为唯一源（文案由集合派生，防漂移）。
`data/status/` 纳入（t14 F1）：基线/历史是裁判的尺子，"降基线无痕提交"必须带证据字段；
baseline 更新本就强制 `--reason`，摩擦可接受。

触及语义（t10 R4②）：删除（D）与改名（R/C）的新旧两侧路径均计入"触及"，删敏感路径同样要证据字段。
另有『前置对象不得静默消失』守卫：门禁前置对象（台账/知识库/seed 等）一旦建立（对应 baseline 指标 >0
即登记 `established`），此后对象消失必须 FAIL『前置对象消失』，不得静默回 SKIP/PASS。

钩子本体是 `gates/hooks/pre-commit`、`gates/hooks/commit-msg` 薄封装，只 `exec python3 gates/cli.py …`；
扫描/裁定逻辑唯一存在于 `gates/cli.py`（无第二套扫描逻辑）。挂接幂等，外来 hook 自动备份。
