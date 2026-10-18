# 对抗复核：render-frame 输出参数化（task-94）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：platform-porter（task-94）
> 被审工作区：`crates/game-desktop/src/bin/render_frame.rs`（+428/−12）与
> `docs/findings/l4-baseline-freeze-plan.md`（+17，§4.1/§4.3 注）；均未提交。
> 复核方式：独立 `CARGO_TARGET_DIR=target/verify-98` 构建/测试 + 沙盒渲染 + CLI 失败路径 + 手册核对；
> 本文件为唯一写入，未 git add/commit。未跑 scenario_run/Mesen（l4-analyst 占串行锁）。

## 0. VERDICT

**needs_revision**（1 条阻塞 + 2 条重要；其余参数化/测试/默认渲染成立）

**成立的面**：`--out-dir`/`--log-path` 缺省行为与显式输出语义、日志字段（scene/env/argv/in/out）、
`out=`=png sha256 前 12、env 白名单不含 REPO_ROOT、默认/显式同帧 png sha 相同、8 条 CLI 单测、
game-desktop/workspace 测试、gates、缺 ROM/trace/参数错误/越界路径 exit2、`--help` exit0、
§4.3 BLOCKED 已解除标注。

**阻塞**：
- **F97-1（阻塞）沙盒可绕过**：`resolve_explicit` 只做**词法**规范化；在仓库内放一个 junction/symlink
  指向仓库外，`--out-dir target/verify-98/escape/out97`（escape 为 junction→%TEMP%）被接受，
  **exit 0 并把 png/idx/log 写到仓库外**（实测 `junction-test.png` 在 %TEMP% 下）。这直接违反
  "显式路径必须落 repo_root 内 / 沙盒语义不可绕过"；需 canonicalize（含父目录）后再比较真实根，
  或拒绝 reparse point/symlink。

**重要**：
- **F97-2（重要）手册 B4 未改**：task 要求 B4 的 `/tmp/...` 与"仓库内 fail-closed"矛盾要改并注明；
  当前 §4.1 B4 仍保留 `OLD=/tmp/l4-old; OLDSB=/tmp/reg-old-sandbox; NEWOUT=/tmp/reg-new`、
  `cp ... /tmp/render-frame-old` 等；23 行后新增的实现状态注只说"**需改为**仓库/沙盒内目录"，
  **示例本身未改**。其中新二进制 `--out-dir "$NEWOUT"`（/tmp/reg-new）现在会被 fail-closed 拒绝，
  B4 新旧二进制 A/B 回归按原样不可执行。
- **F97-3（重要）旧可选参数解析回归**：`parse_cli` 对未知 `--probe-file/--stream-file/--text-file`
  只 `i += 1` 跳过 flag 本身、不跳过其值；实测
  `render-frame 4600 name --probe-file analysis/traces/...probe.jsonl` 把 probe 路径当第 3 个位置参数
  （scene），随后 trace 路径变成 `analysis/traces/analysis/traces/...` ⇒ exit2；HEAD 旧实现会用默认
  scene。修复：显式跳过这些 legacy flag 的值，或统一解析所有带值 flag。

## 1. 参数化正确性 ✅

用 `target/verify-98/debug/render-frame.exe`、`REPO_ROOT=<repo>`、`stg03-timeline` 4600 帧：
```
explicit:
  target/verify-98/debug/render-frame.exe 4600 l4_clear-inject stg03-timeline \
    --out-dir target/verify-98/out --log-path target/verify-98/out/run.log
  exit=0；png=246083B；png sha12=0e988b21f0d1
  log: l4_clear-inject frame=4600 base=0 scroll=(0,0) nt_nz=256 probe=328899
       scene=stg03-timeline env={} argv=["4600",...,"--out-dir",...]
       in=frames:74b98c15f304,probe:6b8053672961,text:- out=0e988b21f0d1
default:
  ... 4600 l4_clear-inject stg03-timeline
  exit=0；oracle/screens/actual/l4_clear-inject.png sha12=0e988b21f0d1（与 explicit 相同）
```
- `in=frames`/`probe` sha 与源文件实测 sha 一致（`74b98c15f304` / `6b8053672961`）；text 缺 ⇒ `-`；
  `out=` 与 png sha12 一致 ✅。
- `env={}` 即使设置了 `REPO_ROOT`：白名单不含 REPO_ROOT ✅（8 条单测中 env 测试通过）。
- 缺省 log=`<out_dir>/.render.log`；默认渲染写 `oracle/screens/actual` 且同帧 png sha 与 explicit 相同 ✅。

## 2. 默认行为/渲染等价 ✅（静态 + 同帧 sha）

- 代码 diff 的渲染/判定段无改动；新增只在 CLI/输出路径/log 字段。
- 同帧默认 vs explicit png sha **完全相同**（`0e988b21f0d1`）；log 旧字段前缀保留、新字段追加行尾。
- 未用 HEAD 旧二进制复跑（避免额外构建/污染）；按 diff 静态核 + 同帧 sha 判默认行为未变。

## 3. fail-closed ✅ / ❌（F97-1）

实测（REPO_ROOT=repo）：
```
--out-dir ../outside_97            exit=2
--out-dir C:\outside_97            exit=2
帧号非法 abc                       exit=2
缺输出名                           exit=2
缺 ROM/trace（REPO_ROOT 空目录）    exit=2（旧 expect panic 已改）
--help                              exit=0（直接运行）
```
- `--help` 直接运行 exit0、USAGE 含新 flag/退出码2 ✅。
- `lexical_normalize`/`within_repo` 单测通过；`..`/绝对路径越界被拒 ✅。
- **但 junction/symlink 绕过（F97-1）**：仓库内 junction 指向 %TEMP%，`--out-dir` 词法仍在 repo_root，
  实际写仓库外且 exit0。属 fail-closed 语义缺口，需修。

## 4. 手册 §4 偏差

- **§4.3 BLOCKED**：新增 `【2026-10-18 task-94 更新】4.1 已实现…该 BLOCKED 条件已解除（历史文本保留）`
  ——合理 ✅。
- **§4.1 B4**：新注承认示例 `/tmp/...` 越界，但**未改 B4 命令**（`OLD/OLDSB/NEWOUT=/tmp/...` 仍在），
  新二进制 `--out-dir "$NEWOUT"` 会被 fail-closed 拒绝；见 F97-2。
- 新注对日志字段/默认路径/fail-closed/自检的描述与实测基本一致（out=sha/默认 sha 与本次实测同值）✅。

## 5. 测试/门禁

- `CARGO_TARGET_DIR=target/verify-98 cargo test -p game-desktop`：**exit 0**；`render_frame` 8 条 CLI 单测
  全过（默认值、参数、错误、越界、sha 向量、env 排序、usage、scene/base）。
- `cargo test --workspace`（同 target）：**exit 0**，全部 tests passed。
- `CARGO_TARGET_DIR=target/verify-98 python gates/cli.py check`：**exit 0，PASS=11 FAIL=0 SKIP=1**；
  docs-links PASS；未跑 Mesen。
- 8 条 CLI 测试有效（不是空壳）：失败/默认/越界/sha 向量都覆盖；**缺 legacy flag 值回归测试**（F97-3）。
- 写范围：`git status` 仅 `crates/game-desktop/src/bin/render_frame.rs` + 手册 §4 diff；无其它代码改动。

## 6. findings（四分类）

### 阻塞（P0）
- **F97-1**：`resolve_explicit` 仅词法 containment，junction/symlink 可绕过；实测仓库内 junction →
  %TEMP% 输出 exit0。需 canonicalize（含父目录，必要时创建后二次校验）或拒绝 reparse point。

### 重要（P1）
- **F97-2**：§4.1 B4 `/tmp` 命令未改；新二进制 `--out-dir "$NEWOUT"` 会被 fail-closed 拒；
  B4 A/B 回归不可按原样执行。应把 NEWOUT/OLDSB 改到仓库或 `REPO_ROOT` 沙盒内，并同步说明。
- **F97-3**：`parse_cli` legacy `--probe-file/--stream-file/--text-file` 只跳 flag 不跳值，默认 scene
  被覆盖为路径，旧用法回归；需跳过值/统一解析。
- **F97-4（中低）**：`--help` 直接 exit0，但 `--help` 位于位置参数后时 CLI 输出被管道/测试捕获行为
  需确认（不影响主要用法；建议单测）。

### 次要（P2）
- **F97-5（低）**：`within_repo` 统一 lower-case，POSIX 大小写敏感环境下可能把不同目录视为同根
  （Windows 目标下无影响；跨平台建议按平台处理或 canonicalize）。
- **F97-6（信息）**：自定义 `CARGO_TARGET_DIR=target/verify-98` 时 exe 的默认 repo_root 推导会落到
  repo/target，需显式 `REPO_ROOT`；正式 `target/debug` 不受影响，但手册/CI 应固定 `REPO_ROOT`。

### 建议（P3）
- **F97-7**：为 legacy flag 值解析、junction 逃逸（可 skip Windows）加单测；为 B4 新命令加一次 A/B
  sha 对照脚本（旧 binary + REPO_ROOT 沙盒 vs 新 `--out-dir` 仓库内路径）。
- **F97-8**：`--out-dir` 默认/显式日志字段已可做 P0-0 manifest；建议把 `out=sha12` 与 png 的校验
  纳入 wrapper 自检（本卡已人工核）。

## 7. 命令摘要

```text
build/test: CARGO_TARGET_DIR=target/verify-98 cargo test -p game-desktop → exit0（8 CLI tests pass）
            cargo test --workspace → exit0
            CARGO_TARGET_DIR=target/verify-98 python gates/cli.py check → exit0 PASS=11 FAIL=0 SKIP=1
render explicit: exit0；png sha12=0e988b21f0d1；log in=frames:74b98c15f304,probe:6b8053672961,text:- out=0e988b21f0d1
render default:  exit0；oracle/screens/actual/l4_clear-inject.png sha12=0e988b21f0d1（同 explicit）
path: ../outside_97 exit2；C:\outside_97 exit2；missing input exit2；bad args exit2；--help exit0
legacy: 4600 name --probe-file <path> → scene 被当路径，缺 trace exit2（旧实现默认 scene）
junction: target/verify-98/escape→%TEMP%，--out-dir target/verify-98/escape/out97 → exit0，
          实际写 %TEMP%/out97/junction-test.png（F97-1 阻塞）
B4: OLD=/tmp/l4-old; OLDSB=/tmp/reg-old-sandbox; NEWOUT=/tmp/reg-new 仍在（F97-2）
```

## 8. 结论

**needs_revision**：参数化输出/日志 sha/默认渲染/测试/gates 成立；但 **junction 沙盒绕过（F97-1）**
与 **B4 `/tmp` 未改（F97-2）**、**legacy flag 解析回归（F97-3）** 必须修/改；F97-4..8 可随文补。
修 F97-1/2/3 后点验即可放行。

---

## 点验（2026-10-18，task-103）

> 对象：task-100 修复后的 `crates/game-desktop/src/bin/render_frame.rs`（工作树，+645/−20）+
> 手册 §4（B4 改仓库内 `target/l4/b4-*`、F97 状态注）。只核 F97-1/2/3 + 抽查 F97-4/5/8；
> 构建用独立 `CARGO_TARGET_DIR=target/verify-103`；**未跑 gates check**（采集窗口纪律），仅跑
> docs-links 单门 + 定向 cargo test；`git status -- oracle/screens` **0 条**。本文件唯一写入，未 git add/commit。

### 点验 VERDICT：**pass**（1 条低优先级自检命令措辞）

- **F97-1 ✅**：自建 `mklink /J`（仓库内 `target/verify-103/escape`→`%TEMP%`），
  `--out-dir target/verify-103/escape/out` → **exit 2**（"真实路径在仓库外"），
  `%TEMP%` 下 **0 文件写入**；代码含 `real_path`（存在 canonicalize / 缺则最近存在祖先+拼尾）、
  `resolve_explicit` 真实 containment、out-dir/log 父目录**创建后二次 `ensure_real_within`**。
  11 条 CLI 单测中 `junction_or_symlink_escape_rejected_by_real_path` 通过（无权限环境会显式跳过）；
  F97-5 POSIX 大小写断言由 `#[cfg(not(windows))]` 测试覆盖；`strip_verbatim` 处理 `\\?\`。
- **F97-2 ✅**：手册 §4.1 B4 已改 `OLD/OLDSB/NEWOUT/OLDBIN` 到 `target/l4/b4-*`；新增说明
  `/tmp` 为何不可用（新参数真实路径必须在 repo_root 内）；§4.2 注与新规则一致、§4.3 BLOCKED 解除注保留。
- **F97-3 ✅**：
  - `1500 cli_legacy --probe-file analysis/traces/gameplay-stage1.probe.jsonl --out-dir ...` → **exit 0**，
    log `scene=gameplay-stage1`（旧默认保持）；
  - `--stream-file`/`--text-file` 组合同 exit 0、scene 正确；
  - `--probe-file` 在末尾缺值 → exit 0（旧宽容）；legacy 值均被解析进 `Cli`（`probe_file/stream_file/text_file`）。
- **F97-4/5/8 ✅ 抽查**：`cargo test -p game-desktop --bin render-frame` → **11/11 CLI 测试通过**；
  `--help` 任意位置有单测并通过；`sha256_hex12` 长度/known vectors 测试通过；junction 测试通过。
- **默认行为**：`1080 title-menu title-menu --out-dir target/...` → png sha12=**1b1940f030bf8**，
  log `out=1b1940f030bf8` 一致 ✅。
- **F103-1（低，措辞）**：task/手册自检文写"沙盒跑 `1080 title-menu`"，但若**只给两个位置参数**，
  scene 按历史默认=`gameplay-stage1`，输出 png sha12=`7da2c4be57c6`（非 1b1940…）；要得期望值须写全
  `1080 title-menu title-menu`。建议任务卡/手册把第三个位置参数 `title-menu` 写明（代码默认行为本身不变）。
- **纪律**：本轮未跑 `gates check/status/visual`，只调用只读 `gate_docs_links()`：**PASS 355 md/446 links**；
  `git status -- oracle/screens`=0；target/ 独立，未改 tracked oracle/screens。

### 命令摘要

```text
junction: mklink /J target/verify-103/escape %TEMP%/...；
          render-frame 4600 junction-test stg03-timeline --out-dir target/verify-103/escape/out
          → exit 2；%TEMP% outside files=0
legacy:   1500 cli_legacy --probe-file analysis/traces/gameplay-stage1.probe.jsonl --out-dir ... → exit0
          log scene=gameplay-stage1；--stream/--text 同；--probe-file 末尾缺值 → exit0
default:  1080 title-menu title-menu --out-dir target/verify-103/title-scene → png sha12=1b1940f030bf8，
          out= 匹配
tests:    cargo test -p game-desktop --bin render-frame → 11 passed/0 failed
docs-links: PASS 355 md/446 links（只读单门；未跑 gates check）
oracle:   git status -- oracle/screens = 0
```

### 结论

**pass**：F97-1/2/3 已实质闭环，F97-4/5/8 抽查通过；仅 F103-1 要求把默认行为自检命令的第三个
`title-menu` 位置参数写全（否则会被默认 scene=gameplay-stage1 的不同 sha 误导），不阻塞提交。

