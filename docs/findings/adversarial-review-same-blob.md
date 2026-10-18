# 对抗复核：actual 六同 blob 裁定（真实 OBS-1 vs 渲染缺陷）（task-126）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：l4-analyst（task-126）
> 被审：`docs/findings/p0-0-same-blob-adjudication.md`（144 行，untracked）
> 复核方式：只读 manifest/trace/shots/adjudicate 产物 + 独立 sha256、trace 内容键、沙盒补渲对照；
> **未跑 Mesen/scenario_run/gates check/status/visual**；只读 docs-links。本文件唯一写入，未 git add/commit。

## 0. VERDICT

**pass**（1 条低优先级证据指针建议）

**裁定 `真实同画面（OBS-1 冻结屏）` 成立**，关键证据独立复算：
- 6 对 actual png 确实同 `5f93bdbb2c84…`；expected 四联同 `62a8ef06…`，但 stg07-combat/start
  expected 分别 `2824cb13…`/`ddb322ac…`（不同）——报告如实列出，未把 actual 同 blob 当 expected 有效。
- 6 帧源 Mesen shots **全部同 sha** `2b2d6ab04350b637…`（6240B，我复核 51 张静态组）；trace 的
  `bg_nmi`/`oam`/`shot_objs` 逐帧**完全相同**，仅 `iram`/`zp` 不同；6 对 render log 均
  `nt_nz=1684 scroll=(0,0) probe=240995`。
- 沙盒补渲 `compare.json`：f5000=`a1b59af2…` 不同；f10000/14600/22000/28000/34000 全 `5f93bdbb…`
  相同 ⇒ render 对 frame 有响应，同图只出现于冻结窗；未发现"empty write stream→固定图"回退
  （`TEXTFALLBACK` 仅 text 缺失时启用，本次 text sha 存在；`pending_row` 是行装配非固定回退）。
- 实际输出/expected 未改；组 D 仍 `blocked:no-source`；报告推荐把 actual 6 组登记为
  `documented_duplicates`（不剔除、不当作 6×独立 0 容差通过），与证据一致。

## 1. 逐对数字（独立 hash）

| name | frame | actual sha16 | expected sha16 | 源 shot sha16 | frames/probe/text sha12 |
|---|---:|---|---|---|---|
| l4_stg06-late | 22000 | `5f93bdbb2c84612b` | `62a8ef06c4b9bcd1` | `2b2d6ab04350b637` | `5f0fb2289cae`/`66502e3071d0`/`a0b2496fe7d2` |
| l4_stg07-combat | 16000 | `5f93bdbb2c84612b` | `2824cb134987c7b3` | `2b2d6ab04350b637` | 同上 |
| l4_stg07-start | 14600 | `5f93bdbb2c84612b` | `ddb322ac57021bc6` | `2b2d6ab04350b637` | 同上 |
| l4_stg08-combat | 18000 | `5f93bdbb2c84612b` | `62a8ef06c4b9bcd1` | `2b2d6ab04350b637` | 同上 |
| l4_stg08-late | 28000 | `5f93bdbb2c84612b` | `62a8ef06c4b9bcd1` | `2b2d6ab04350b637` | 同上 |
| l4_stg08-start | 16600 | `5f93bdbb2c84612b` | `62a8ef06c4b9bcd1` | `2b2d6ab04350b637` | 同上 |

- run1/run2 png/idx 一致性：manifest `two_run_agreement=16/16`，本复核逐对重算 16/16 通过。
- sources：`shots/stg08-timeline_f{14600,16000,16600,18000,22000,28000}_*.png` 均 **6240B**、
  sha `2b2d6ab04350…`；扩展扫描 `2b2d6ab0…` 共 **51 张（f9000–33600）**、`d531d33a…` 共
  45 张（f7500–33900），与报告 §3 一致。
- `actual_unique=11 / reduction 5`、`expected_unique=14 / reduction 5`、expected 组
  `6ff345×2 / 788d69×2 / 62a8ef×4` 均与 manifest/报告一致。
- 13 rendered pair actual≠expected（含 8 个 P0-0-2 `status=ok` 的渲染分类；status 语义见 task-125
  F125-1，不在本裁定范围）；报告 §0/§5 已如实说 13/13，不把 expected 判有效。

## 2. 输入侧/trace 语义 ✅

- 文件级：6 对共享同一 scene `stg08-timeline`，frames/probe/text 三个文件 sha **相同**；唯一不同是
  `frame` 参数（22000/16000/14600/18000/28000/16600）。
- 帧级独立比较 `analysis/traces/stg08-timeline.frames.jsonl`：
  ```
  bg_nmi      unique=1（6 帧全同）
  oam         unique=1
  shot_objs   unique=1
  iram        unique=6
  zp          unique=6
  ```
  ⇒ 报告"决定画面的内容键相同、仅易变计数不同"**成立**；这是 actual 同图的直接输入侧依据。
- render log 六对均 `nt_nz=1684`、`scroll=(0,0)`、`probe=240995`、`env={}`，与报告 §2 一致。

## 3. 渲染侧 ✅

- `target/p0-0/adjudicate/compare.json` 实读：
  - f5000 → `a1b59af2b4cebe79…`（不同）；
  - f10000/f14600/f22000/f28000/f34000 → 均 `5f93bdbb2c84612b…`（同）；
  - 6 个文件均 rc=0/exists=true。
- `render_frame` 无"空写流→固定图"通用回退：
  - `TEXTFALLBACK` 仅在 text 文件缺失时启用；本 run text sha `a0b2496f…` 存在；
  - `pending_row` 是帧内多行重写保留末行，不是固定图 fallback；
  - 沙盒 f5000 不同证明 render 对 frame/输入有响应，不是忽略帧。
- 报告 §4 "render 输出亮/色与 Mesen 源屏不同属 P0-1 L4 差异" 是**额外观察**，不用于本裁定；
  本裁定只依赖"源同图 + 内容键 + 渲染冻结窗"三链，边界清楚。

## 4. 结论强度/处置

- **结论 `真实同画面（OBS-1 冻结屏）`**：源 Mesen 截图同 sha（A）+ trace 内容键同（A）+ 沙盒补渲
  冻结窗同图/窗口外不同（A）三链充分；**不是渲染/输入映射缺陷**。
- **不是 6×独立 0 容差通过**：报告建议 actual 6 组登记 `documented_duplicates`、`actual_unique=11`、
  不当作独立 pair；与 manifest dedupe 公式一致，正确。
- **expected 四联同 blob 与 actual 六组不是同一名单**：expected 四联仅 4（62a8ef），actual 六组多
  stg07-combat/start；报告 §5 明确区分，未混用。
- **组 D 仍 `blocked:no-source`**：报告未声称组 D 有源、未改 expected/threshold ✅。

## 5. 证据边界/低优先级

- **F128-1（低）**：报告 §4 的沙盒补渲数字依赖 `target/p0-0/adjudicate/compare.json`（gitignored）；
  建议在报告中写该文件/命令，当前正文已提路径但未给 compare.json 的完整字段；可补一次。
- **F128-2（低）**：报告 §3 的 `2b2d`/`d531` 51/45 静态组是扩展扫描；建议给 `shots` glob 命令/
  逐组名单（当前 `analysis/` 产物 gitignored，复核需重新 glob）。
- **F128-3（信息）**：actual 同 blob **不改变** P0-0 "not all green"（组 D blocked + 13 expected 差异）；
  报告已写，无需改。
- **F128-4（信息）**：expected #17/#18 的来源"另一历史 run"引用 groupD 决策 §3；本报告不必证明，
  但 §7.1/§7.2 重采时需保留该跨 run 帧域假设为 B 级。

## 6. 纪律

- 未跑 Mesen/scenario_run/gates check/status/visual；只读 docs-links 单门（本轮末尾复算）。
- `git status -- oracle/screens` 应为 0（本复核未写 expected）；被审文档 untracked；本文件唯一写入。
- 未改代码/gates/基线/expected/threshold。

## 7. 命令摘要

```text
actual 6 对 png sha 均 5f93bdbb2c84612b…；run1/run2 16/16 一致
expected：4 组 62a8ef06c4b9；stg07-combat 2824cb134987；stg07-start ddb322ac5702
shots/stg08-timeline_f*：2b2d6ab04350 ×51（9000-33600）/ d531d33a65d3 ×45（7500-33900）
trace 6 帧：bg_nmi/oam/shot_objs unique=1；iram/zp unique=6
compare.json：f5000 a1b59af2…；f10000/14600/22000/28000/34000 5f93bdbb…
render log：nt_nz=1684 scroll=(0,0) probe=240995 ×6
dedupe：actual_unique=11/reduction5；expected_unique=14/reduction5
docs-links：只读门 PASS；oracle/screens status 0；未跑 Mesen/gates check/status/visual
```

## 8. 结论

**pass**：`真实同画面（OBS-1）` 结论有源截图 + trace 内容键 + 沙盒补渲三链支撑，6 对 actual 同 blob
非渲染/输入映射缺陷；expected 差异/组 D 边界未被越权处理。F128-1/2 为低优先级证据指针，建议
提交前补 compare.json/静态组清单；不阻塞。actual 6 组登记 `documented_duplicates`、不按独立 0 容差
通过的处置正确。
