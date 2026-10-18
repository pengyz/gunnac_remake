# 对抗复核：P0-0-2 GUI trace 重渲（抽样 sha + 36/36 + diff + 22 对口径）（task-142）

> 日期：2026-10-18｜复核方：platform-verifier（verifier，证伪立场）｜被审：l4-analyst task-140
> 被审：`docs/findings/p0-0-render-gui-run.md`（128 行，untracked）；manifest `target/p0-0/render/g-manifest-v2.json`；
> 产物 `target/p0-0/render/g_run1|g_run2`（各18 png/idx/log）；二进制 `target/p0-0/build/debug/render-frame.exe`
> sha `00c29d6a31c0f885…`。
> 复核方式：只读 hash + manifest + `tools.diff_visual.compare` + 用同一二进制重渲 2 对到 target 沙盒；
> **未跑 Mesen/gates**；未改源码/expected/threshold。本文件唯一写入。

## 0. VERDICT

**needs_revision（P2：报告两处口径与 manifest/实测不符；渲染数据主体通过）**

- **抽样重渲通过**：early_exit=true `l4_stg03-start` 与 false `gameplay-stage1_f01260` 用报告二进制重渲，
  png/idx 与 g_run1 **逐字节相同**；日志 `in=`/`in_frames_read`/`text_records_read`/`early_exit` 与 manifest 一致。
- **两跑 36/36 通过**：18 对 × png+idx 全部 g_run1==g_run2。
- **diff 通过**：gameplay f1260 `tools.diff_visual.compare` = **3742 px**，与报告/manifest 一致；0 容差未放宽。
- **22 对口径主体**：18 可渲（16 canonical + ending-4/5 assisted）、4 blocked、0/18 expected sha 匹配、
  L4 0 容差未达成——这些成立。
- **needs_revision 两处**：
  1. **F142-1**：报告 §0/§5 只把 **#14** 标为 `replaced-invalid-asset`；manifest 与 §2 表实际 **5 对**
     （stg02-start + 四个 C 四联）标该状态；§5 canonical16 的构成说明不完整。
  2. **F142-2**：报告 §4 写 stg03 `l4_stg03-start` 新实现 **140.9s**；但 manifest 该行 run1/run2 为
     **163.1/161.4s**，本次独立重渲 **164.7s**（仍 <180s）；140.9 无法复现/与自身 manifest 矛盾。

## 1. 抽样重渲（同二进制/输入）

二进制 sha `00c29d6a31c0f885f447476cbdb54b0919d6501caab0f6aeb0d3ccb1e1588b48`（与报告一致）。
输出到 `target/t142/sample`，与 `g_run1` 对比：

| sample | early_exit | png sha16（sample = g_run1） | idx sha16（sample = g_run1） | 重渲 wall |
|---|---|---|---|---:|
| l4_stg03-start f5700 | **true** | `3f6272110fc00ba1` | `f131f592bf69301b` | 164.7s |
| gameplay-stage1_f01260 f1260 | **false** | `c39bbeffe098aa6e` | `ba1140d6b74ddcff` | 38.0s |

日志（sample 与 manifest run1 一致）：
- stg03：`in=frames:553310d0009a,probe:b71ee8fcbb84,text:fdad599d24f2`，`out=3f6272110fc0`，
  `in_frames_read=6925 text_records_read=2393980 early_exit=true text_margin=64`；
- gameplay：`in=frames:0fdd22821701,probe:a840f5703cfd,text:f05ce843840e`，`out=c39bbeffe098`，
  `in_frames_read=907 text_records_read=775968 early_exit=false text_margin=64`。
⇒ **两样本均 PASS**；early_exit 与全读边界均正确。

## 2. 两跑确定性（36/36）

独立遍历 `g_run1` 18 png +18 idx 与其 `g_run2` 同名文件 sha256：
```
png+idx checked=36 bad=[]  → 36/36 全同
```
manifest `two_run_deterministic=18`、`two_run_fail=[]`；目录各 **18 png/18 idx/18 log** 与报告一致。

## 3. diff 复算

`tools.diff_visual.compare(exp, actual)`（256×240 真彩、0 容差）：
```
expected gameplay-stage1_f01260_stage-start.png
actual   target/p0-0/render/g_run1/gameplay-stage1_f01260.png
diff_pixels = 3742
```
- 与报告 §2 row1 / manifest `diff_pixels=3742` 一致；diff 只作记录，未作放宽/阈值。
- 18 对 expected_matches 全 false（manifest summary `expected_matches=0`）；报告"0/18 actual==expected、
  L4 0 容差未达成、不得宣称 L4 通过"成立 ✅。

## 4. 22 对口径（主体成立，F142-1）

manifest 汇总：
```
pairs_total=22 rendered=18 blocked_no_source=4
rendered_breakdown={natural:10, injected:6, assisted_ending45:2}
replaced_invalid_asset=5
expected_matches=0 early_exit_pairs=8 two_run_deterministic=18
blocked_pairs=['l4_fade-clear','l4_ending-1','l4_ending-2','l4_ending-3']
```
- 18 可渲=16 canonical+2 assisted；4 blocked；ending-4/5 `source=assisted`（C1/C3 bot+assist 命中
  `6ff345dc`），未冒充 natural ✅。
- **F142-1（P2）**：**`replaced_invalid_asset` 实际 5**（`l4_stg02-start` + `l4_stg06-late` /
  `l4_stg08-combat` / `l4_stg08-late` / `l4_stg08-start`），而报告 §0 L13 只写 **#14**、
  §5 L101 也只标注 #14；§2 表其实已有 5 行 `replaced-invalid-asset`。需统一为 **5 对**（并说明 16 canonical
  中 5 对 expected 待重采），否则 22 对口径不完整。
- **F142-3（信息）**：task 卡写 `early_exit=true 8 / false 8`；实际 **true=8、false=10**
  （gameplay 3 + title 1 + stg08 4 + ending-4/5 2）。报告表 true/false 是对的，task 卡算术少 2（title/ending）。
- **blocked 4** 与 dense 负证据（recapture §5.2）一致；未 staging/硬凑 ✅。

## 5. 性能/early_exit（F142-2）

- 报告 §4 主张 stg03 `l4_stg03-start` 140.9s；但：
  - manifest 该 pair `run1.wall_s=163.1`、`run2.wall_s=161.4`；
  - 本次独立重渲 **164.7s**；
  - 三项均仍 **<180s**，但没有 140.9 的证据。
- **F142-2（P2）**：应把报告 §4 改为 **manifest/实测 161–165s**（或注明 140.9 的独立测量环境），
  并相应更新“−74%”（按 536.6→163 约 −70%，仍显著）。
- 早期 wall 分布 0.1–919.4s：manifest 有 title 0.1、ending 897.4/919.4 等，报告 §0/§4 分布方向正确。
- **early_exit=true 8 对**（stg03 6 + stg07-start/combat 2）与 manifest 一致；
  另 10 对 false（gameplay3/title1/stg08 4/ending2）因目标帧 > text max_f 或 text 为空，与报告 §2 表一致。

## 6. 纪律/证据

- 未跑 Mesen/gates check/status/visual；未改源码/expected/threshold；只读 manifest/shots/产物 + target 沙盒重渲。
- `target/t138-fix.patch` 未动；本报告唯一写入。
- docs-links 只读门传；`git status -- oracle/screens` 需为 0（本复核未写 expected）。

## 7. 命令/证据摘要

```text
bin sha 00c29d6a31c0f885…（=报告）；g_run1/g_run2 各18 png/idx/log
36/36 png+idx sha 全同
sample true  l4_stg03-start: png 3f6272110fc0… / idx f131f592bf69…；early=true/in=6925/records2393980
sample false gameplay_f01260: png c39bbeffe098… / idx ba1140d6…；early=false/in=907/records775968
diff compare gameplay f1260 = 3742 px（0 容差）
manifest: 22/18/4, replaced_invalid_asset=5, expected_matches=0, early_exit=8
F142-1 replaced口径（报告只 #14 vs manifest 5）；F142-2 140.9s vs manifest 163.1/实测164.7
docs-links PASS；oracle/screens 0；未跑 Mesen/gates；未改源码/expected
```

## 8. 结论

**needs_revision**：抽样 sha/36:36/diff/22 对主体全通过；需修两处报告口径：
① `replaced-invalid-asset` 应为 **5 对**（不是仅 #14）；② stg03 性能改为 **manifest/实测约 161–165s**
（140.9 无证据，仍 <180s）。另 task 卡 false8 应为 false10（报告表已正确）。修完即可提交，渲染数据本身可用。

## 点验（2026-10-18，task-147）

> 只点验 task-146 的两处修正；其余 task-142 已全过。

### 1) F142-1（5 对 replaced）→ pass
- `p0-0-render-gui-run.md`：
  - §0 L13：**5 对 `replaced-invalid-asset`**（#14 stg02-start + 四联 #16/#19/#20/#21，expected 待 §7.2）；
  - §2 注 L64：同 5 对，明确“16 canonical 中 5 对 expected 待重采”；
  - §5 L102/L105：canonical16 行注明 **5 对 replaced**，合计 22 行说明 **4 blocked + 5 对 expected 待重采**。
- `p0-0-readjudication-gui.md`：§0 L11/L17、§2 注 L53、§8 L122 均同步 **5 对 replaced（#14+四联）**。
- 与 manifest summary 实读一致：
  `rendered=18, blocked_no_source=4, replaced_invalid_asset=5, expected_matches=0, deterministic=18, early_exit=8`；
  §2 表 5 行 `replaced-invalid-asset`（stg02-start + C 四联）与 summary 吻合 ✅。

### 2) F142-2（161–165s 区间）→ pass
- `p0-0-render-gui-run.md` §4 L88-90 现写：
  - 隔离单跑 **140.9s**；批内 manifest **163.1/161.4s**（4 worker 争用）；渲染面复核独立重渲 **164.7s**；
  - 统一口径 **161–165s 区间**（约 −70%，<180s 达标，out sha `3f6272110fc0…` 不变）。
- manifest 实读该 pair：`run1.wall_s=163.1`、`run2.wall_s=161.4`，落在区间内 ✅；
  我 task-142 独立 sample 164.7s 也落区间内；140.9 被明确标为隔离单跑而非批内值，口径自洽。
- 旧 536.6s→区间 161–165s 约 −69%~−70%，报告“约 −70%”成立；不是之前的 −74% 固定值 ✅。

### 3) docs-links/纪律
- docs-links 只读 **PASS 386 md/466 links**；`oracle/screens` 0；未跑 Mesen/gates；未改源码/expected。
- 本文件为唯一追加写入；task-142 关闭，**verdict 更新为 pass**。
