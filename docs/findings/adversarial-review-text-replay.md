# 对抗点验：render_frame text replay 流式 + 早退（task-138/141）

> 日期：2026-10-18｜复核方：platform-verifier（verifier，证伪立场）｜被审：platform-porter task-138
> 被审：`crates/game-desktop/src/bin/render_frame.rs`（工作树 +283/−50）+ 手册 §11 注；
> `target/t138-fix.patch`（36836B，保留未动）。
> 复核方式：独立构建（`CARGO_TARGET_DIR=target/verify-t138`）、`cargo test -p game-desktop`、
> 旧/新二进制 A/B、新 stg03 性能/RAM 实测、text 文件单调扫描。**未跑 Mesen/gates**；未改源码/expected。

## 0. VERDICT

**needs_revision（仅 P2 文档数字；功能/性能/测试全通过）**

- 输出语义：gameplay f1260（无早退）old/new png+idx **逐字节相同**；stg03 f5700（early_exit=true）
  new 全文件 vs old+截断 prefix png+idx **相同**；`in=` text 全文 sha 与整读一致。
- 早退：`f > target + TEXT_MARGIN` 正确；text 文件实测 0 逆序；缺失/空 text 单测通过。
- 性能：新 stg03 **149.36s / 峰值 247.5MB**（<180s / <2GB）；日志 `early_exit=true`、`in_frames_read=6925`、
  `text_records_read=2393980`、`text_margin=64`；但申报 127.35s 未复现（仍达标，见 F141-2）。
- 测试：`cargo test -p game-desktop` **14 passed**；3 个新增流式/早退单测有效。
- **唯一不通过项**：手册 §11 的 **“读量≈全文件 19%”** 与实测不符（约 **10.1% 行 / 9.7% 字节**），
  需在提交前改正（F141-1）。

## 1. A/B 输出语义（自跑）

### 1.1 gameplay-stage1 f1260（无早退）
旧二进制 `target/verify-103/debug/render-frame.exe` vs 新 `target/verify-t138/debug/render-frame.exe`
（均 `REPO_ROOT=repo`，输出到 `target/t141/{a,b}`）：

| side | png sha256 前16 | idx sha256 前16 | 日志 `in=...text:` |
|---|---|---|---|
| old | `c39bbeffe098aa6e` | `ba1140d6b74ddcff` | `f05ce843840e` |
| new | `c39bbeffe098aa6e` | `ba1140d6b74ddcff` | `f05ce843840e` |

- 两次 wall 41.6/41.4s；png/idx **逐字节相同**；`in=` text sha=全文件整读 sha `f05ce843840e` ✅。
- new 日志追加 `in_frames_read=907 text_records_read=775968 early_exit=false text_margin=64`；
  旧前缀字段（`name frame base scroll nt_nz probe scene env argv in=... out=`）保持不变 ✅。

### 1.2 stg03-timeline f5700 `l4_stg03-start`（early_exit=true）
- new 全文件（1.268GB）输出：png `3f6272110fc0…`、idx `f131f592bf69…`；
- old 用 **截断 prefix**（f≤5764、123.7MB；等价于实际应用写集）+ `--text-file` 输出：
  **同 png/idx**；
- old 日志 text sha `63dba479753d`（prefix 不同，合理），new 日志 text sha `fdad599d24f2`（=全文件 sha）；
- 渲染 `out=` 两者一致；早退没有丢弃任何 f≤target 的写点（margin=64 只跳过 f>5764 的记录）✅。

## 2. 早退/单调性

- 代码 `read_text_writes(path, limit)`：`if rec.0 > limit { early_exit=true; break; }`；
  `limit = target + TEXT_MARGIN`，缺省 **64**（`TEXT_MARGIN` env 可配）；日志 `text_margin=64`。
- 生产者 `phase_batch.rs:29-73`：按 `trace.lines()` 帧序 push `merged`，无排序反转；实际文本文件独立扫描：
  ```
  gameplay-stage1: 775,968 lines, maxf=907,  inversions=0
  stg03-timeline:  23,609,155 lines, maxf=24000, inversions=0
  stg08-timeline:   6,443,437 lines, maxf=16231, inversions=0
  ```
  ⇒ 单调前提成立；early break 与整读前缀等价。
- 缺失/空 text：`read_text_writes` 返回空集，单测 `text_stream_early_exit_prefix_equivalent` 覆盖
  （empty path / missing path）；与旧 `read_to_string().unwrap_or_default()` 语义一致。
- **F141-3（信息）**：旧版“整读全文件”未独立复跑（536s 级）；本节用 old+截断 prefix 证明**实际应用写集**等价，
  若手册要写“old 全文件也同值”，应标 B 级或补跑。

## 3. 性能/RAM（new B，独立复跑）

```
command: target/verify-t138/debug/render-frame.exe 5700 l4_stg03-start stg03-timeline \
          --out-dir target/t141/b --log-path .../new-stg03.log
rc=0  wall=149.36s  peak RSS=247.5MB
log: in=frames:553310d0009a,probe:b71ee8fcbb84,text:fdad599d24f2 out=3f6272110fc0
     in_frames_read=6925 text_records_read=2393980 early_exit=true text_margin=64
```
- **<180s ✅**、**<2GB ✅**；RAM 与申报 247MB 一致。
- **F141-2（P3）**：申报 127.35s，本机独立复跑 **149.36s**（+17%，仍达标）；建议报告写
  “127–150s 随负载”或注明测量环境/负载，不要把 127.35 当固定值。

## 4. 日志字段/测试/手册

- 日志：旧前缀字段未变，末尾追加 `in_frames_read= text_records_read= early_exit= text_margin=`；
  gameplay（无早退）与 stg03（早退）都正确输出。
- `cargo test -p game-desktop`：**14 passed / 0 failed**（含新增
  `sha256_stream_matches_oneshot`、`sha256_file_stream_matches_and_cleans_up`、
  `text_stream_early_exit_prefix_equivalent`）；另有 smoke 5 passed；doc-tests 0。
- 流式 SHA 与整读：new game/stg03 的 `in=` text sha 分别等于整文件 sha `f05ce843840e`/`fdad599d24f2`；
  单测 known vectors/stream vs oneshot 通过。
- 手册 §11 注：实现描述（流式、TEXT_MARGIN、日志字段、A/B sha、RAM）与代码一致；
  **唯一数字错误是“读量≈19%”**（见 F141-1）。

## 5. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- **F141-1（文档数字，提交前改）**：手册 §11 写 `读量≈全文件19%`；实测 early exit 前缀为
  **2,393,979 records / 23,609,155 lines = 10.14%**（或 **123.7MB / 1269.0MB = 9.75% 字节**）。
  “19%”不成立，建议改为“≈10%（行/字节）或按实际读取字节数记录”。这是本卡唯一 needs_revision 项。

### 建议（P3）
- **F141-2**：性能申报 127.35s，本机复跑 149.36s（仍<180）；建议报告给区间/环境，避免固定值争议。
- **F141-3**：旧版整读全文件 vs new 全文件未直接复跑；当前 old+prefix A/B 已证明应用写集等价；
  手册若声称 old 全文件同值，请标 B 级或补跑。
- **F141-4**：`in_frames_read` 含触发早退的那条记录（6925），与 prefix 保留数（2393979）差 1 是设计口径；
  建议日志/文档注明，避免读者误以为丢记录。

## 6. 命令/证据摘要

```text
build: CARGO_TARGET_DIR=target/verify-t138 cargo test -p game-desktop → 14 passed; build render-frame ok
A/B gameplay: old/new png c39bbeffe098… / idx ba1140d6…; text sha f05ce843840e; early_exit=false
A/B stg03: new full png 3f6272110fc0… / idx f131f592bf69…; old prefix same; new text fdad599d24f2
new stg03: wall149.36s peak247.5MB; in_frames_read=6925 records2393980 early_exit=true margin64
monotonic: stg03 23,609,155 lines 0 inverse; stg08 6,443,437 0; gameplay 775,968 0
manual 19% → actual 10.14% lines / 9.75% bytes (F141-1)
docs-links 只读 PASS；oracle/screens 0；未跑 Mesen/gates；未改源码/expected；patch 保留
```

## 7. 结论

**needs_revision**（仅 F141-1 手册读量百分比；改一个数字即可）：
A/B sha 不变、早退/单调/空缺失、流式 SHA、日志字段、14 项测试、性能/RAM 均独立通过；
性能 149.36s 仍 <180s。改完手册 §11 的 19%→约 10% 后可提交，l4 可立即渲 stg03/stg08 全对。

## 点验追加（2026-10-18，F141-1 修复 → pass）

- Lead 已真正落盘；`Select-String -Path docs\findings\l4-baseline-freeze-plan.md -Pattern '10\.14|9\.75'` 结果：
  ```
  1812: >   text_records_read=2393980 early_exit=true`（读量≈全文件 **10%**：行 10.14%（2,393,979/23,609,155）/
  1813: >   字节 9.75%（123.7MB/1269.0MB），task-141 点验实测；其余被早退跳过；`in=` 仍全文 sha `fdad599d24f2`）。
  ```
- 与本次独立实测 **10.14% 行 / 9.75% 字节**一致；19% 已消除。F141-2/F141-3 按 Lead 口径进
  task-138 提交说明；无需改代码。
- **F141-1 关闭；task-141 verdict 更新为 pass**。A/B/早退/单调/流式 sha/日志/14 测试/性能 RAM
  全部通过，可提交 task-138（render_frame + 手册注 + 本复核报告）。
