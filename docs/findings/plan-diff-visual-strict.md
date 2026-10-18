# diff_visual `--strict`（L4 0 容差验收判据）方案与实施记录

> 任务卡：task-21（用户 2026-10-18 裁定：批准 diff_visual 门禁硬化——0 容差语义"任意外差异=FAIL"；两步走）。
> 状态：**v1 实施完成**（默认行为保留；`--strict` 已可用；默认翻转待 P0-0 基线冻结后另卡）。
> 纪律依据：[AGENTS.md](../../AGENTS.md) §5 门禁表 + 判据裁定
> [decision-l4-zero-tolerance-ruling.md](../knowledge/decision-l4-zero-tolerance-ruling.md)；
> L4 判定口径见 [l4-visual-plan.md](../spec/l4-visual-plan.md)。

## 0. 问题与目标

现行 `tools/diff_visual.py` 的默认退出码语义（t92 ⑤，**分类账模式**）：

> exit 0 = 全部对通过 **或** 全部差异有显式四分类记录；exit 1 = 存在未分类差异。

因 `oracle/screens/visual-report.md` 已对 22/22 对差异给出四分类记录，**默认门禁 PASS ≠ 像素一致**
（现状 1/22 对逐像素相等、合计差 280,132/1,351,680=20.72%）。用户裁定 L4 判据回到 0 容差后，
需要给门禁一个可执行的"0 容差"判据；但若立刻把默认改为"任意外差异=FAIL"，pre-commit 总闸会立即恒红，
所有提交被阻断。故分两步：

1. **本卡**：新增 `--strict` 模式（0 容差验收判据），**默认行为不变**；
2. **默认翻转**（pre-commit `visual-diff` 切 strict）= 待 **P0-0 基线冻结**
   （task-16「L4 P0-0：冻结可复现基线」方案，文档待建）+ 独立复核后另卡执行（本卡不翻转，
   `tools/diff_visual.py` 内留 `TODO(task-21)`）。

## 1. 两种模式（实现口径）

| 模式 | 调用 | 退出码 | 用途 |
|---|---|---|---|
| **默认（分类账）** | `python tools/diff_visual.py` | 0=全部对通过或全部差异有分类记录；1=存在未分类差异；2=输入目录缺失 | pre-commit / 既有流水线（**行为不变**） |
| **`--strict`（0 容差）** | `python tools/diff_visual.py --strict` | 0=**任意外差异为零**；1=存在任意外差异；2=输入目录缺失 | **L4 0 容差验收判据** |

**`--strict` 判定细则**（`tools/diff_visual.py` `main()`）：

- 任一对存在差异像素（含已被四分类记录归因的差异）⇒ `STRICT: FAIL`；
- 缺失 Rust 侧实际图（actual/ 未生成）或尺寸不符/解码错误 ⇒ `STRICT: FAIL`（外差异，不只是像素差）；
- 白名单 `gates/visual_whitelist.json`：**当前不存在=0 条豁免**；若将来存在条目，按其豁免语义整对豁免
  （白名单=容差的具名化，沿用既有语义，本卡不改判定逻辑）；
- 输出固定行：`STRICT: FAIL diff=<总差异像素数>` / `STRICT: PASS diff=0`。
  ——**注意：`diff=` 只计像素差**；缺失 actual/尺寸不符/解码错误等**"外差异"不计入 diff 数值**，
  须看 **FAIL/PASS 本身**（实测缺 actual 时输出 `STRICT: FAIL diff=0`），勿把 `diff=0` 误读为"无外差异"。

**明确不说的事**：`--strict` 不改变差异计算口径（256×240 原生逐像素、真彩域、禁缩放）；
不把"分类账合规"当 0 容差达成；不启用任何白名单候选（W-C1/W-C4 已由用户裁定撤销）。

## 2. 判据落位

**L4 0 容差验收判据 = `python tools/diff_visual.py --strict` exit 0。**

- 脚本 docstring（`tools/diff_visual.py` 顶部）已写明两模式、退出码语义与默认翻转 TODO；
- `tools/` 无 `README.md`（未新建；本文件的"两模式"节 + 脚本 docstring 为权威说明）；
- 待 [l4-visual-plan.md](../spec/l4-visual-plan.md) 的并发未提交改动（task-13/20，由 Lead 提交）入库后，
  由本卡补一行判据到其 L4 验收节（**只加一行、精确 pathspec 提交**；当前不改该文件，避免误卷）。

## 3. 验收证据（本机 Windows，2026-10-18）

| 验收项 | 命令 | 结果 |
|---|---|---|
| 默认行为不变（回归） | `python tools/diff_visual.py` | exit **0**；`L4 视觉：1 通过 / 22 比对；分类记录 21；未分类 0`（与改动前一致） |
| 现基线 0 容差判据 | `python tools/diff_visual.py --strict` | exit **1**；`STRICT: FAIL diff=280132`（=现行 22 对差异总像素数，与 visual-report 口径一致） |
| 全对相同临时样本 | 复制工具到临时 root + expected/actual 放同名同字节 PNG，`--strict` | exit **0**；`STRICT: PASS diff=0` |
| 总闸 | `python gates/cli.py check` | **CHECK: PASS**（visual-diff 门禁仍走默认模式，PASS 不变） |
| 生成物无脏写 | 跑完默认/严格模式后 `git status --short oracle/screens` | 空（report/diff 重生成字节一致） |

临时样本复现（零依赖；已在实施时执行）：

```sh
# 1) 全对相同临时样本 ⇒ --strict exit 0
TMP=$(python -c "import tempfile;print(tempfile.mkdtemp())")
mkdir -p "$TMP/tools" "$TMP/oracle/screens/expected" "$TMP/oracle/screens/actual"
cp tools/diff_visual.py "$TMP/tools/"
cp oracle/screens/expected/<任一 expected>.png "$TMP/oracle/screens/expected/"
cp oracle/screens/expected/<同上>.png "$TMP/oracle/screens/actual/"
python "$TMP/tools/diff_visual.py" --strict   # 期望: STRICT: PASS diff=0 / exit 0

# 2) 现基线 ⇒ --strict exit 1
python tools/diff_visual.py --strict          # 期望: STRICT: FAIL diff=280132 / exit 1

# 3) 默认回归 ⇒ exit 0（分类账口径，行为不变）
python tools/diff_visual.py
```

## 4. 未完成 / 边界（显式）

1. **默认翻转未做**（本卡范围外）：pre-commit `visual-diff` 仍走默认分类账模式；翻转需
   P0-0 基线冻结 + 独立复核后另卡（`TODO(task-21)` 已登记在脚本）。
2. **判据进 spec 的一行未加**：等 `l4-visual-plan.md` 的 task-13/20 未提交改动由 Lead 入库后补
   （Lead 已指示；只加一行、精确 pathspec 提交）。
3. **`--strict` 的差异计算口径未改**：仍是 256×240 真彩域逐像素；若 P0-0 重采基线后口径变化，
   由 P0-0 任务卡同步。
4. **白名单语义**：文件不存在=0 条；未来启用须用户确认 + 登记（沿用 l4-visual-plan §2 流程，
   本卡不新增/不启用）。
5. **GUI/L4 像素目标未达成**：本卡只提供 0 容差**判据**，不改变基线也未声称 L4 已达成
   （当前判定=未达成，见 decision 知识条目）。
