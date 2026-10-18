# P0-0-2（GUI trace）22 对重渲两跑 + 新二进制（text 流式+早退）（task-140）

> 日期：2026-10-18｜执行：l4-analyst｜状态：**18 对 ×2 跑全部确定性（36/36 png+idx 相同）；4 对组 D blocked:no-source 不渲；0 失败**
> 前置：GUI 重采 trace（task-133，input-effective；见 `p0-0-recapture-gui-run.md`）+ task-138 工程修复
> （render_frame text replay **流式 + 按目标帧早退**；工作树版本，本报告记录二进制 sha 供追溯）。
> 口径：canonical env（39 诊断 env 全 unset）；0 容差 diff（`tools/diff_visual.compare`，256×240 真彩域）；不跑 gates check/status/visual。

## 0. 结论摘要

| 项 | 值 |
|---|---|
| **渲染二进制** | `target/p0-0/build/debug/render-frame.exe`，sha256 `00c29d6a31c0f885f447476cbdb54b0919d6501caab0f6aeb0d3ccb1e1588b48`（task-138 后；旧 `ab115570…`） |
| 22 对口径 | **18 可渲**（16 canonical + **ending-4/5 2 对 assisted 来源**）+ **4 blocked:no-source**（fade-clear/ending-1/2/3）；**5 对 `replaced-invalid-asset`**（#14 stg02-start + 四联 #16/#19/#20/#21；expected 待 §7.2 重采） |
| 两跑确定性 | **18/18 对 png sha 相同、18/18 idx sha 相同**（36/36 渲染）；0 失败 |
| 与 expected 的关系 | **0/18 actual 与旧 expected sha 相等**（旧 expected=另一 run/输入域）；diff 见 §3；L4 0 容差仍未达成（P0-1 目标，非本卡失败） |
| 性能 | 单对 run1 wall 0.1–919.4s，**合计 5724.8s（1.59h）**；early_exit=true 8/18 对；无 OOM（流式读，RAM 有界） |
| 产物 | `target/p0-0/render/g_run1|g_run2/{png,idx.json,log}`；manifest `target/p0-0/render/g-manifest-v2.json`（gitignored） |
| settings | speed=100（GUI 采集批次后已恢复；本渲染不用 Mesen）；39 诊断 env 全 unset |

## 1. 方法与复现

```powershell
# 渲染（new binary；--out-dir 隔离两跑；REPO_ROOT 等基础 env，39 诊断 env 全 unset）
target/p0-0/build/debug/render-frame.exe <frame> <name> <scene> `
  --out-dir target/p0-0/render/g_run1 --log-path target/p0-0/render/g_run1/<name>.log
# 两跑比较：png sha256 + idx sha256；log 记录 in=/out=/early_exit/in_frames_read/text_records_read
```

- 输入：GUI 重采 trace（`analysis/traces/<scene>.{frames,probe}.jsonl`）+ phase-batch 新文本
  （`analysis/phase/<scene>-text.jsonl`；本批 4 个文本：gameplay-stage1 39MB/775,968 行、stg03-timeline **1.27GB**/23,609,155 行、
  stg08-timeline **339MB**/6,443,437 行、tas-stg08b2-direct 82MB/1,588,573 行）。
- **text 覆盖上限（重要）**：phase-batch 只对"该帧有 $2F/$30 指针写"的帧回放，故文本 max_f：
  gameplay **907**、stg03 **24000**、stg08 **16231**、C1 **27718**。目标帧高于 max_f 时 log `early_exit=false`（读完整文件、
  文本状态为最后已知状态）；低于时 `early_exit=true`（流式+早退生效）。
- 每对记录 `early_exit / in_frames_read / text_records_read / text_margin`（manifest `pairs[].run1/run2`）。

## 2. 逐对结果（18 行；两跑全部 png=idx 相同）

| # | name | frame | scene | 组 | source | render_status | expected sha（前 12） | actual run1/run2 sha（前 12） | 两跑 | early_exit | in_frames_read | text_records_read | diff 像素 | wall(run1/run2) |
|---:|---|---:|---|---|---|---|---|---|---|---|---|---|---:|---|
| 1 | gameplay-stage1_f01260 | 1260 | gameplay-stage1 | A | natural | ok | `38365db2cb4f` | `c39bbeffe098` | ✅/✅ | false | 907 | 775,968 | 3742 | 42.4/42.4s |
| 2 | gameplay-stage1_f01500 | 1500 | gameplay-stage1 | A | natural | ok | `7f4ac8a23511` | `7893d0a3475b` | ✅/✅ | false | 907 | 775,968 | 3609 | 47.0/47.5s |
| 3 | gameplay-stage1_f02349 | 2349 | gameplay-stage1 | A | natural | ok | `4274a33f45f5` | `5737ae009af6` | ✅/✅ | false | 907 | 775,968 | 4382 | 65.0/65.6s |
| 4 | title-menu_f00180_boot-black | 180 | title-menu | E | natural | ok | `788d6984576b` | `1b1940f30bf8` | ✅/✅ | false | 0 | 0 | **0** | 0.1/0.1s |
| 5 | l4_clear-inject | 4600 | stg03-timeline | B | injected | ok | `5cacbe294d53` | `246dd5712608` | ✅/✅ | **true** | 4665 | 1,640,692 | 58159 | 144.3/158.3s |
| 6 | l4_combat-variety | 6500 | stg03-timeline | B | injected | ok | `6f7fe4992e59` | `5af017eb7284` | ✅/✅ | **true** | 6925 | 2,393,980 | 28260 | 182.1/169.6s |
| 7 | l4_late-combat | 12000 | stg03-timeline | B | injected | ok | `c0baa0f8d21b` | `64d148fe2ede` | ✅/✅ | **true** | 12065 | 8,760,772 | 6118 | 267.1/251.5s |
| 8 | l4_shop-window | 5400 | stg03-timeline | B | injected | ok | `988809e0b414` | `3689760a47f5` | ✅/✅ | **true** | 6925 | 2,393,980 | 19040 | 170.9/157.9s |
| 9 | l4_stg02-start | 3404 | stg03-timeline | B | injected | **replaced-invalid-asset** | `788d6984576b`（boot-black 复制品） | `ebc6372f79d9` | ✅/✅ | **true** | 4623 | 1,588,038 | 2495 | 134.8/135.3s |
| 10 | l4_stg03-start | 5700 | stg03-timeline | B | injected | ok | `24a4cb45c0d8` | `3f6272110fc0` | ✅/✅ | **true** | 6925 | 2,393,980 | 2261 | 163.1/161.4s |
| 11 | l4_stg06-late | 22000 | stg08-timeline | C | natural | **replaced-invalid-asset** | `62a8ef06c4b9` | `406f3962559c` | ✅/✅ | false | 16231 | 6,443,437 | 18681 | 499.5/522.8s |
| 12 | l4_stg07-combat | 16000 | stg08-timeline | C | natural | ok | `2824cb134987` | `65de647de951` | ✅/✅ | **true** | 16065 | 6,234,046 | 11105 | 377.2/402.3s |
| 13 | l4_stg07-start | 14600 | stg08-timeline | C | natural | ok | `ddb322ac5702` | `c9070704ca5c` | ✅/✅ | **true** | 15566 | 5,633,469 | 11806 | 347.3/371.2s |
| 14 | l4_stg08-combat | 18000 | stg08-timeline | C | natural | **replaced-invalid-asset** | `62a8ef06c4b9` | `406f3962559c` | ✅/✅ | false | 16231 | 6,443,437 | 18681 | 439.4/429.4s |
| 15 | l4_stg08-late | 28000 | stg08-timeline | C | natural | **replaced-invalid-asset** | `62a8ef06c4b9` | `406f3962559c` | ✅/✅ | false | 16231 | 6,443,437 | 18681 | 633.8/598.9s |
| 16 | l4_stg08-start | 16600 | stg08-timeline | C | natural | **replaced-invalid-asset** | `62a8ef06c4b9` | `406f3962559c` | ✅/✅ | false | 16231 | 6,443,437 | 18681 | 394.0/401.9s |
| 17 | **l4_ending-4** | 34500 | tas-stg08b2-direct | D | **assisted** | **sourced-assisted** | `6ff345dc5c3f` | `5aa8a4f8bdc0` | ✅/✅ | false | 27718 | 1,588,573 | 8427 | 897.4/823.2s |
| 18 | **l4_ending-5** | 36000 | tas-stg08b2-direct | D | **assisted** | **sourced-assisted** | `6ff345dc5c3f` | `5aa8a4f8bdc0` | ✅/✅ | false | 27718 | 1,588,573 | 8427 | 919.4/819.7s |

> **不渲的 4 对（组 D blocked:no-source）**：`l4_fade-clear`(#11)/`l4_ending-1`(#6)/`l4_ending-2`(#7)/`l4_ending-3`(#8)——
> dense overlay 133 张 any-frame 全零命中（recapture §5.2），不 staging、不硬凑。
> **ending-4/5** 来源=C1/C3 bot+assist（f33000+ 命中 `6ff345dc`），`trigger=injected/assisted`、**不冒充 natural**；
> 帧号按手册映射取 f34500/f36000（该 run 内同 blob）。
> **`replaced-invalid-asset` 共 5 对**（与 manifest summary 一致）：#14 `stg02-start`（expected=boot-black 复制品 `788d6984`）+ 四联 #16/#19/#20/#21（expected=旧件 `62a8ef06`）——**16 canonical 中 5 对 expected 待 §7.2/§7.7 重采**；actual 均已重渲。

## 3. 与旧 headless-render（上轮 actual）的 diff 对照（0 容差像素数 / 61440）

| 组 | pair | 新 GUI-render diff | 旧 headless-render diff（task-130 表） | 变化 |
|---|---|---:|---:|---|
| A | gameplay-stage1_f01260 / f01500 / f02349 | 3742 / 3609 / 4382 | 10033 / 7219 / 6080 | **↓ 改善** |
| B | l4_clear-inject | 58159 | 58364 | ↓ 微降 |
| B | l4_combat-variety | 28260 | 28563 | ↓ 微降 |
| B | l4_late-combat | 6118 | 5527 | ↑ 变差 |
| B | l4_shop-window | 19040 | 18241 | ↑ 变差 |
| B | l4_stg02-start | 2495 | 4476 | ↓ 改善 |
| B | l4_stg03-start | 2261 | 2918 | ↓ 改善 |
| C | l4_stg06-late / stg08-{combat,late,start} | 18681 ×4 | 18887 ×4 | ↓ 微降 |
| C | l4_stg07-combat / l4_stg07-start | 11105 / 11806 | 12208 / 13501 | ↓ 改善 |
| D | l4_ending-4 / l4_ending-5 | 8427 / 8427 | （上轮无 actual，blocked） | 新增源（assisted） |
| E | title-menu_f00180 | **0** | 0 | 不变（像素级 0 差异） |

- 统计：18 对 diff min=**0**（title）、median=**9766**、max=**58159**（clear-inject）；合计 diff=242,555 px（未作验收阈值）。
- **关键**：新 trace 输入生效后渲染仍与旧 expected 有 diff（0/18 sha 相等），属 **P0-1 渲染差异**（L4 0 容差未达成）；
  本卡只冻结"GUI trace + 新二进制"的确定性 actual，不得据此宣称 L4 通过。

## 4. 性能与 task-138 修复验证

- **单次渲染对比（stg03 `l4_stg03-start` f5700）**：旧实现 `ab115570` **536.6s** → 新实现 `00c29d6a`：**隔离单跑 140.9s**；
  批次（4 worker 并行争用）manifest 实测 **run1/run2=163.1/161.4s**（渲染面复核独立重渲 **164.7s**）⇒ 统一口径 **161–165s 区间**
  （相对 536.6s 约 **−70%**，仍 <180s 达标）；**out sha `3f6272110fc0` 逐字节相同**（png 246083B / idx 2102B 同）⇒ 语义等价 + 性能达标。
- 全批：18 对 run1 合计 **5724.8s**；两跑合计 11283.8s；4 worker 并行墙钟 21:43–22:39（~56min）；
  无 OOM（流式读；早期 stg03 旧实现单进程需整文件 1.27GB 内存，现消除）。
- **early_exit 生效面**：8/18 对（stg03 全部 6 + stg07-start/combat，目标帧 ≤ 文本 max_f）；
  **stg08 另 4 对与 ending-4/5 因目标帧 > 文本 max_f（16231 / 27718）仍整文件读**（early_exit=false，wall 394–919s）。
  ⇒ 后续优化点（非本卡）：phase-batch 对"无指针写"帧的文本覆盖/分片，或按目标帧截断生成——已作为 P0-1/工具改进候选记录。
- 日志字段：`in=`（frames/probe/text sha）+ `out=` + `early_exit` + `in_frames_read` + `text_records_read` + `text_margin=64`。

## 5. 22 对口径（重述）与后续

| 类别 | 对数 | 明细 | 处置 |
|---|---:|---|---|
| 可渲（canonical） | **16** | A3 + B6 + C6 + E1；其中 **5 对 replaced-invalid-asset**（#14 + 四联） | actual 已冻结（两跑确定性）；**5 对 expected 待 §7.2 重采** |
| 可渲（assisted 来源） | **2** | #9 ending-4、#10 ending-5 | actual 已渲；expected 可保留旧件（blob 命中=6ff345dc），登记 assisted |
| blocked:no-source | **4** | #6 fade-clear、#7 ending-1、#8 ending-2、#11 ending-3 | 不渲、不 staging；dense 负证据（recapture §5.2） |
| 合计 | 22 | — | **P0-0 不得宣称全绿**（4 blocked + **5 对 expected 待重采** + 18 对 diff>0 属 P0-1） |

后续（Lead 排）：①本卡完成后派渲染面复核；②**22 对重裁**（新 actual vs expected，仍 0 容差，期望 18 对 diff 数字 + 4 blocked 口径更新）；③expected 重采/安装（#14 + 四联仍按 §7.2；ending-4/5 登记 assisted 来源）。

## 6. 产物与复算

- manifest：`target/p0-0/render/g-manifest-v2.json`（summary + 18 对逐对：输入 sha/输出 sha/early_exit 指标/diff；gitignored）；
- 渲染目录：`target/p0-0/render/g_run1|g_run2/`（18×2 png + idx + log）；
- 复算两跑确定性：
  ```powershell
  $n=0;$bad=0; Get-ChildItem target/p0-0/render/g_run1 -Filter '*.png' | ForEach-Object {
    $n++; $a=(Get-FileHash $_.FullName -Algorithm SHA256).Hash
    $b=(Get-FileHash ("target/p0-0/render/g_run2/"+$_.Name) -Algorithm SHA256).Hash
    if($a -ne $b){$bad++} }; "pairs=$n bad=$bad"
  ```
- 复算 diff：`python` + `tools.diff_visual.compare`（见 manifest `pairs[].diff_pixels`）。

## 7. 验收自查

- [x] 18 对 ×2 跑完成，两跑 png/idx **36/36 相同**，0 失败（§2）；
- [x] 二进制 sha 记录 `00c29d6a…`，per-pair early_exit/in_frames_read/text_records_read 记录（§2/§4）；
- [x] 4 对 blocked 不渲；ending-4/5 以 assisted 来源渲并标记（§5）；
- [x] 22 对口径重述、diff 对照、性能与 task-138 验证（§3/§4/§5）；
- [x] 未跑 gates check/status/visual；未改 expected/threshold/代码；产物均 target/（gitignored）；
- [ ] docs-links PASS（提交前复核）；独立复核 pass（渲染面复核卡）。
