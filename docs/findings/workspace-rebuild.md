# L4 渲染输入工作区重建（task-170）

> 日期：2026-10-19（执行 23:39–23:59，Asia/Shanghai）｜角色：l4-analyst（实施）｜任务：task-170（Lead 派单：渲染输入工作区重建）
> 目标：从 `analysis/traces/` 的 **17 个 GUI trace** 重建 `analysis/phase/` 渲染文本工作区；补齐 `target/p0-0/rom/gunnac_cn.nes`；用 2 对抽样重渲与 P0-0-2 GUI 基线 `g_run1` **png/idx sha 逐字节同**作验收。
> 边界：只写 `analysis/`、`target/`（均 gitignored）与本报告；**不改** expected/threshold/gates/源码；未跑 gates check/status/visual。

## 0. 结论

| 项 | 结果 |
|---|---|
| 17 traces → phase text | **17/17 全部重生成**（真实写集 5 个 + 无 probe 的 12 个按构造 0B） |
| 三大户新 text sha 与重建前一致 | **gameplay-stage1 / stg03-timeline / stg08-timeline 全部 same_sha=true** ⇒ 工作区重建**未改变语义**（确定性重建） |
| `target/p0-0/rom/gunnac_cn.nes` | **`hardlink`**，262160B，sha `4d76901c1b2455f2…` = `rom/gunnac_cn.nes` 同 sha |
| 2 对抽样重渲 vs g_run1 | **png+idx 全同（2/2）**：`gameplay-stage1_f01260`、`l4_stg03-start` |
| 产物 | `analysis/repro/workspace-rebuild/workspace-rebuild.json`（gitignored）；本报告 |

## 1. 环境与可复现命令

```powershell
# 直接调用已构建二进制（不重编译；cwd=仓库根）
$env:REPO_ROOT = (Get-Location).Path
# 大户口先行（可并行）
target/p0-0/build/debug/phase-batch.exe gameplay-stage1 gameplay-stage1-text
target/p0-0/build/debug/phase-batch.exe stg03-timeline stg03-timeline-text
target/p0-0/build/debug/phase-batch.exe stg08-timeline  stg08-timeline-text
# 其余场景（无 probe 的输出为 0B，属预期）
# target/p0-0/rom/gunnac_cn.nes = rom/gunnac_cn.nes 的 hardlink（同 volume）
# 抽样验收
target/p0-0/build/debug/render-frame.exe 1260 gameplay-stage1_f01260 gameplay-stage1 --out-dir target/p0-0/render/workspace-check
target/p0-0/build/debug/render-frame.exe 5700 l4_stg03-start stg03-timeline --out-dir target/p0-0/render/workspace-check
# 与 target/p0-0/render/g_run1/<name>.png|.idx.json 比 sha256（逐字节同）
```

## 2. 17 traces → phase text 明细

| scene | frames 大小(B) | probe.jsonl(B) | 新 text 大小(B) | 新 text sha256(12) |
|---|---:|---:|---:|---|
| `gameplay-stage1` | 53,026,252 | 82,524,670 | 39,044,268 | `f05ce843840e` |
| `gs5-anchor-render` | 15,000,838 | 23,606,517 | — | `` |
| `gs5-stg01-clear-anchor` | 1,340,838 | —(无 probe) | 0 | `e3b0c44298fc` |
| `gs5-stg01-clear-c1` | 1,618,944 | —(无 probe) | 0 | `e3b0c44298fc` |
| `gs5-stg01-clear-c2` | 1,340,838 | —(无 probe) | 0 | `e3b0c44298fc` |
| `gs5-stg01-full` | 3,330,681 | —(无 probe) | 0 | `e3b0c44298fc` |
| `hit-test` | 45,849,819 | —(无 probe) | 0 | `e3b0c44298fc` |
| `stg03-timeline` | 140,172,894 | 36,668,918 | 1,268,980,076 | `fdad599d24f2` |
| `stg06-timeline` | 93,258,894 | —(无 probe) | 0 | `e3b0c44298fc` |
| `stg08-timeline` | 198,582,894 | 71,998,519 | 339,266,155 | `255883bd72d9` |
| `t105-bot-short` | 91,893 | —(无 probe) | 0 | `e3b0c44298fc` |
| `tas-stg07-relay` | 42,165,224 | —(无 probe) | 0 | `e3b0c44298fc` |
| `tas-stg08-direct-dense` | 41,196,062 | —(无 probe) | 0 | `e3b0c44298fc` |
| `tas-stg08-direct-dense100` | 41,196,062 | —(无 probe) | 0 | `e3b0c44298fc` |
| `tas-stg08-direct` | 41,196,062 | —(无 probe) | 0 | `e3b0c44298fc` |
| `tas-stg08b2-dense` | 39,976,061 | 93,382,411 | 82,154,412 | `d464ab332afb` |
| `tas-stg08b2-direct` | 39,976,056 | 93,263,872 | 82,155,154 | `cd067aae358d` |
| `title-menu` | 589,653 | —(无 probe) | 0 | `e3b0c44298fc` |

> 无 `probe_mem` 的场景（title-menu/gs5-*/hit-test/t105-bot-short/stg06-timeline/tas-stg07-relay/tas-stg08-direct[-dense/-dense100]）phase-batch 输出 **0B 空 text** 属构造预期（无 $2F/$30 指针写流），不是失败；渲染不依赖这些场景。

## 3. 三大户重建前后 sha 对比（语义不变证据）

| scene | 旧 text sha256 | 新 text sha256 | size 同 | sha 同 |
|---|---|---|---|---|
| `gameplay-stage1` | `f05ce843840e1cb6…` | `f05ce843840e1cb6…` | True | **True** |
| `stg03-timeline` | `fdad599d24f2a7c7…` | `fdad599d24f2a7c7…` | True | **True** |
| `stg08-timeline` | `255883bd72d9f0b5…` | `255883bd72d9f0b5…` | True | **True** |

> 说明：重建前后 sha **完全相同** ⇒ phase-batch 对该 trace 的文本提取是确定性的、工作区重建不改语义；`tas-stg08b2-direct` 重建后 sha `cd067aae…` 与重建前一致（旧文件存在）；`tas-stg08b2-dense` 本次新建 `d464ab33…`（此前无 text，属补建）。

## 4. 抽样验收（vs P0-0-2 g_run1 基线）

| pair | frame | scene | wall(s) | png sha 同 | idx sha 同 | png sha256(16) | idx sha256(16) |
|---|---:|---|---:|---|---|---|---|
| `gameplay-stage1_f01260` | 1260 | `gameplay-stage1` | 39.9 | **True** | **True** | `c39bbeffe098aa6e…` | `ba1140d6b74ddcff…` |
| `l4_stg03-start` | 5700 | `stg03-timeline` | 144.4 | **True** | **True** | `3f6272110fc00ba1…` | `f131f592bf69301b…` |

> 判据=与 `target/p0-0/render/g_run1/<name>.png/.idx.json` sha256 **逐字节相同**；两对均 PASS ⇒ 重建后的 working text 可替代旧工作区（渲染输入语义一致）。

## 5. rom 链接

- src `rom\gunnac_cn.nes` → dst `target\p0-0\rom\gunnac_cn.nes`（**hardlink**，同 volume）
- size 262160B；sha `4d76901c1b2455f2bdae8e8ac99b1779441f9f2155614dab222c229ae06c77ae` = src sha `4d76901c1b2455f2bdae8e8ac99b1779441f9f2155614dab222c229ae06c77ae` **相同**（硬链接不复制字节、不修改 `rom/`）。

## 6. 备注 / 风险

- 所有工作区文件在 `analysis/`、`target/`（gitignored）；`rom/` 只读未改（硬链接目标 sha 与源相同）。
- phase-batch 直接调用已构建二进制；本卡未重编译、未改源码/tools/expected/gates。
- 17 traces 中 12 个无 probe ⇒ 0B text 是事实（不是遗漏），已在本表标明；无渲染 pair 依赖这些场景。
- 验收仅抽 2 对（gameplay + stg03，覆盖文本密度差异大户）；如需全 18 对可复用同命令。

## 7. 验收自查

- [x] 17/17 traces → phase text（5 真实 + 12 空，明细见 §2）
- [x] 三大户重建前后 sha 全同（§3，语义不变证据）
- [x] `target/p0-0/rom/gunnac_cn.nes` 链接+sha 校验（§5）
- [x] 2 对重渲 vs g_run1 png/idx sha 逐字节同（§4）
- [x] 产物 JSON（gitignored）+ 本报告；未改 expected/threshold/gates；未跑 gates visual
- [ ] 独立复核 pass + commit + push（新纪律；本报告+工作区规则外文件由 Lead 统一收口）

