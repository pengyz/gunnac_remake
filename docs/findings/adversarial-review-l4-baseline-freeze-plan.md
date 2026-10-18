# 对抗复核：L4 P0-0 基线冻结执行手册（task-30）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审方：decomp-analyst（l4-analyst）
> 被审对象：[l4-baseline-freeze-plan.md](l4-baseline-freeze-plan.md)
> **复核版本：783 行，sha256 `9f548f2ac153caf7…`（mtime 13:41:36，l4-analyst 已 inactive）**。
> 说明：Lead 转述"728 行"为前一次保存；复核时终稿已追加 **附录 D（降级 A 预研，+55 行）**，
> 本复核按 783 行定稿执行（D1-D6 也纳入核验）。
> 纪律：只读重导 + 本文件为唯一写入；未改 `crates/`、`oracle/`、`tools/`、`rom/`、`reference/`、`data/`；
> 未 `git add`/`git commit`。复核立场：不采信手册自述，逐条重算/重跑只读命令；以"他人照做能否执行"为准。

## 0. 结论（VERDICT）

**needs_revision**

手册的**事实核验部分扎实**（附录 A 22 行与方案 §2.6 **0 mismatch**、附录 B 39 env 与源码**逐项一致**、
组 D 帧同余算术成立、D3/D5/D6 可复算、Mesen2/工具/bin/输出路径事实全部核对为真），
边界纪律（Windows 不可采集/降级 A 零验收数字/trigger=assisted/39 env unset/P0-0 vs P0-4 分工/门禁两步走）
也基本落实。但存在 **3 个阻塞性执行缺陷** + 5 个重要缺陷 + 8 个次要/建议项；**修正后需复审**，当前版本不宜直接作为
P0-0 执行依据。

阻塞项简述（详见 §3）：
1. **F1（阻塞）**：§3.2 overlay 生成器对 `stg03-timeline`/`stg08-timeline` **无条件再追加标准 8 个 `probe_mem`**，
   而这两个源场景**本已各含 8 个** → overlay 产生 **16 条注册**；`capture.lua` 对每条注册都注册回调（无去重）⇒
   probe 事件重复、头行 16 条 ⇒ §2.4 判据/§3.3 签名/后续渲染全部受影响。
2. **F2（阻塞）**：§7.5/§8.2/§5.1 的**去重验收口径错**：expected `unique=17` 是 **P0-0 前**的现状，
   完成 stg02+四联重采后应为 **21 或 22**；actual 去重脚本扫描整个 `oracle/screens/actual/`
   （当前 **257 个 PNG / 127 unique**），不是 22 对 canonical 产物 ⇒ `actual 22 unique` 判据无法按脚本验收。
3. **F3（阻塞）**：§5.3 自称"完整参考实现"，但 `PAIRS` 为空、`all([])=True` 会**空跑打印 `TWO-RUN: PASS`**，
   且写出的 manifest 每组只有 name/scene/frame/两个 run sha/deterministic **6 个字段**，**不符合 §5.1 schema**
   （缺 expected/input/env/argv/status/dedupe/strict）。执行卡照抄会产出零对"通过"的假基线。

## 1. 复核方法（独立重导，非采信自述）

- 逐节读完全文（§0-§12 + 附录 A-D）；对 §1/§2/§3/§7/§8 核"命令/期望/失败回退/BLOCKED"是否具体、无歧义。
- 独立解析/重算：附录 A 与方案 §2.6 逐行比对；附录 B 与 `render_frame.rs` env grep 比对；
  组 D 帧同余算术 + 场景文件帧数/probe 配置；D3 覆盖、D5 非黑像素画像、D6 env 计数；
  overlay 生成器**按手册代码原样模拟**并统计 probe_mem；去重脚本在原目录实跑计数；报告文件相对链接检查。
- 只读实跑/抽查（≥3 节）：§2 场景文件事实、§3 overlay 模拟、§7 去重脚本计数、附录 A/B/D 算术、
  `gates/cli.py rom`、`scenario_run.py` 解析路径/输出字段、`capture.lua` probe 上限与截图命名。
- 未执行真实采集（Windows 无 Mesen2/`analysis/`，符合手册 §9 边界）；未跑 render/采集类命令。
- 被审文件在复核期间稳定（783 行/sha 固定）；只读复核不写受保护路径。

## 2. 独立验证为真的事实（先确认手册可信面，避免误伤）

| # | 项 | 我的独立复核结果 |
|---|---|---|
| V1 | **附录 A 22 行 vs 方案 §2.6** | 解析两表 22/22 行，逐项比 name/frame/scroll/probe/expected sha/actual sha：**0 mismatch**（附录 A 自述属实） |
| V2 | **附录 B 39 诊断 env** | `render_frame.rs` 中 `env::var("…")` 去重 = **40**（39 诊断 + `REPO_ROOT`）；附录 B 39 项**逐项一致**，`REPO_ROOT` 单列允许；D6 脚本可复算 |
| V3 | **组 D 来源帧同余** | `tas-clear-full-v5.txt`：`frames 300000`、`capture_every 1500 probe`、`bot stg01-clear from 1260`、3 个 assist poke（1255 01be/01b6/01b7）——与手册 §3.2/§3.4 一致；`28500=19×1500`、`30000=20×1500`、`31500=21`、`33000=22`、`34500=23`、`36000=24`，全部整除且 ≤36500 ⇒ **算术成立**（"同余"只证帧可被 v5 的 capture_every 覆盖，来源仍靠 §3.3 ③ blob 判定——手册已如此标注，正确） |
| V4 | **候选场景覆盖（D3）** | title 1080≥180；gameplay 2400≥2349；stg03 24000≥12000；stg02 19000≥12000；stg04 26000≥12000；stg08 34000≥28000；stg07 32000≥28000；v5 300000≥36000；v4/v3/v2 30000≥30000 —— **全部 True** |
| V5 | **probe 配置（D4）** | gameplay-stage1 15 条 probe_mem（8 标准 + 7 额外，含 002f/0030、2006/2007 重复）；stg03/stg08 各 8 条；v5 0 条 —— 与手册 §2.1/D4 一致 |
| V6 | **独立 12 对内容画像（D5）** | 按附录 D 脚本复算 12 对 expected/actual 非黑像素，数值**逐项全同**（如 clear-inject exp 58,209/act 876；fade exp 59,371/act 0；title 0/0）——D5 数据面属实（但仅为画像，非验收，手册标注正确） |
| V7 | **工具/路径事实** | Mesen2 **2.1.1** 有 `docs/methodology.md`/`docs/ai-decompile-pipeline.md` 依据；`MESEN=tools/mesen2/Mesen`；bin 名 `render-frame`/`phase-batch` 存在；`scenario_run.py` 支持 `list` / `run <name\|路径>` / `--timeout`；shots 默认 `shots/<name>_f%05d_<tag>.png`；`run.json` 含 `trace_lines/done/timed_out/scenario_sha256`；`gates/cli.py status`/`rom` 均有效（rom 实际执行 PASS）；`diff_visual.py` 确有默认/`--strict` 两模式；`analysis/.scenario.lock` 串行锁存在 |
| V8 | **纪律边界** | §9.1/§9.2 明确 Windows 不可采集、`render-frame` 无 analysis 会 panic；附录 D 标注降级 A "零验收数字"；§7.6 trigger=assisted（v5 3 poke）；§5.2/附录 B 39 env canonical unset；全文"17/22"仅 §9.3 一句**勘误说明**（"旧任务卡 17/22 已修正为 12/22"），无活性残留 |
| V9 | **分工与门禁** | §7 开头（501-505 行）明确 **P0-0 含资产裁定与真值重采/去重；P0-4 保留重渲后重判/自然链替换**——与 Lead 裁定一致；§8.3（599-606 行）明确 P0-0 不翻默认门禁、默认翻转留 P0-6 全绿后——与 `--strict` 两步走一致（task-24 R1 已收口） |
| V10 | **文档链接** | 本手册全部相对链接解析：**0 断链**（§12 的 docs-links PASS 自验收可成立） |

## 3. findings 四分类

### 3.1 阻塞（P0：照做会执行失败/产出不可信；必须修后复审）

#### F1（阻塞）overlay 生成器重复注册 probe_mem（§3.2，行 268-308）

- **问题**：`make()` 末尾无条件 `out += [f"probe_mem {lo} {hi}" for lo,hi in STD_PROBES]`（行 294）；
  但组 B/组 C 的源场景 `stg03-timeline.txt`/`stg08-timeline.txt` **本已各含同样 8 条 `probe_mem`**（行 299-303 调用）。
  按手册代码原样模拟：`stg03` 源 8 条 → overlay **16 条（unique 8）**。
- **后果**：`tools/scenario_capture.lua` 对 `PLAN.probe_mem` 每条都 `emu.addMemoryCallback`（:140-150，无 dedupe）
  ⇒ 同一写点注册 **2 个回调**、probe.jsonl 事件重复、头行 16 条；§2.4 "每文件 8 头行"/§3.3 ① 签名判据全失败；
  更严重的是 `render_frame` 消费重复 probe 事件，可能重复应用写（尤其 row/PPU 协议路径）⇒ **canonical actual 不可信**。
- **证据**：实测源 `grep -c '^probe_mem '` = 8（stg03/stg08）；按手册 make 模拟 = 16；`capture.lua` 无 dedupe。
- **修正建议**：`make()` 先读源 `probe_mem` 集合，**只追加源缺失的区间**（或去重后统一输出）；
  生成后加断言 `grep -c '^probe_mem ' == 8`（组 B/C）并落 `analysis/repro/probe_counts.tsv`；
  §2.4 判据改为"按场景/overlay 的期望头行数"（见 F4）。

#### F2（阻塞）去重/unique 验收口径错误（§5.1:411、§7.5:546-560、§8.2:588-597）

- **问题 1（expected 17 已过期）**：当前 expected `unique=17/22`（重复组：stg02=title、四联 4 张、ending4/5）；
  P0-0 §7 会把 stg02 与四联 4 张**替换为逐帧真值**、ending4/5 若裁定 documented duplicate。
  完成后 expected unique 应为 **21**（ending4/5 保留重复）或 **22**（若两帧也替换不同）。
  §5.1 schema（411 行）、§7.5（559 行）、§8.2 checklist（593 行）仍写 **17** ⇒ 用 P0-0 前数字验收 P0-0 后产物。
- **问题 2（actual 去重扫描面错）**：§7.5 脚本 `for d in ("oracle/screens/expected","oracle/screens/actual")` 直接
  `Path(d).glob("*.png")`。当前 `oracle/screens/actual/` 有 **257 张实验 PNG（127 unique）**（不是 22 对）；
  即使 P0-0 写入 canonical 产物，目录仍混有大量实验图 ⇒ 脚本永远报 `unique=127 of 257`，
  §7.5/§8.2 的 `actual 22 unique` 判据**永远无法通过**（除非按 manifest/独立目录过滤）。
- **问题 3（口径冲突）**：§7.3（539 行）允许 actual 有 `documented duplicate`（五帧 actual 两两不同"除经裁定的
  documented duplicate"），但 §7.5（559 行）/§8.2（593 行）无条件要求 actual 22 unique。二者需统一。
- **证据**：当前实测 expected 17/22、actual 257/127；脚本行 548-560；schema 411；checklist 593。
- **修正建议**：去重脚本改为按 manifest 的 22 个 pair 名（或 `actual_canonical_run1/`）过滤；
  验收数字改为 **expected 21（或 22，取决于 ending4/5 裁定）、actual 22（或 22−documented dup）**；
  §5.1 schema 示例值同步；§8.2 checklist 用"P0-0 后口径"并写明公式。

#### F3（阻塞）§5.3 wrapper 非完整实现、空跑假绿、manifest 与 §5.1 schema 不符（行 434-481）

- **问题**：§5.3 标题/验收称"本手册给完整参考实现""manifest 落盘且与 §5.1 schema 一致"，但：
  1. `PAIRS = []`（行 442-444），且无读 `mapping.tsv`/manifest 的代码；`all([])` 为 True ⇒ **0 对也打印
     `TWO-RUN: PASS`** 并写空 `pairs: []`；
  2. 写出的 manifest 只有 `{pairs:[name,scene,frame,run1 sha,run2 sha,deterministic], two_run_deterministic}`（474-476 行），
     **缺** §5.1 要求的 `env/canonical/expected_sha256/expected_source/input_sha256/render_env/render_argv/status/reuse/
     dedupe/strict/capture_tag/trigger`；
  3. 不含 §7 资产裁定/去重/`--strict` 记录步骤，不能产出 P0-0 的完整交付物。
- **后果**：执行卡照抄 ⇒ 得到一个"通过但空/不完整"的基线，违反 P0-0 目标（可复现+逐对 provenance+资产有效）。
- **修正建议**：把 wrapper 补成真正读 `analysis/repro/mapping.tsv`（或 manifest 模板）+ 填充 §5.1 全字段 +
  `PAIRS` 非空断言（`if not PAIRS: raise`）+ 资产裁定/dedupe/`--strict` 调用；或明确降级为"草稿骨架"，
  并把"完整实现"列为执行卡任务（当前自述与内容不符，至少必须改自述）。

### 3.2 重要（P1：可执行但判据/事实不可信，应修）

#### F4（重要）probe 头行数判据"每文件 8"错误；与实际场景/overlay 不符（§2.2:142-152、§2.4:193-202、§3.3:314-328）

- **事实**：`capture.lua` 对每个 `probe_mem` 写 **1 条独立头行**（`{"probe":"lo-hi","cb":true}`，:150），
  **不是"首行含 8 条"**；且各输入头行数不同：title-menu **无 probe_mem/无 .probe.jsonl**（0）；
  gameplay-stage1 **15 条**（已含 8+7 重复）；stg03/stg08 **8 条**（若按现 overlay bug 则 16 条）；v5 overlay **8 条**。
- **问题**：§2.2 判据（151-152 行）"逐场景 .probe.jsonl 首行含 8 条 … 总行数含 8 头行"、
  §2.4（196-198 行）"`grep -c '"probe"' … 每文件应为 8" 对 gameplay/title/overlay 均不成立；
  §3.3（317 行）"事件行数 == 旧 probe= 值（±0 头行扣除）"未说明各文件头行数不同。
- **修正建议**：给出**逐场景期望头行表**（title 无文件；gameplay 15；stg03/stg08 8；overlay 8）并以此做断言；
  §3.3 写明"总行数 − 该场景头行数 = 事件数"；同时修 F1 使 overlay 为 8。

#### F5（重要）§7.2 用 v5 frames.jsonl 选帧，但无"选定后再 capture"的两遍机制（行 518-530）

- **问题**：§7.2 要求用组 D overlay 的 `frames.jsonl` watch 选出 stg08-start/combat/late、stg06-late 的**任意帧**
  （如"首个 stage_no==08 且 intro_ctr>0"），然后"capture 后与 actual 语义比对"。
  但 overlay 的 captures 是**预先固定的**（§3.2 只含 28500/30000/.../36000），capture.lua 在运行中按预置
  captures/capture_every 出图；**无法在事后对任意帧补 capture**。
- **后果**：按现流程，§7.2 的 4 张 expected 无法产出于所选帧；只能拿到 capture_every 1500 的固定帧（未必命中）。
- **修正建议**：明确**两遍**流程：① run1 只出 frames.jsonl → 用 watch 选帧；
  ② 用选定帧列表生成 overlay2（`capture <frame> l4f<frame>`）并重跑 → 再取 shots；或一次跑时把候选窗口
  全量 capture（代价大），并在 §2.2/§7.2 给出命令与产物路径。

#### F6（重要）§7.1 又把"902=文本内容本身差"错指到 `l4-signing-criteria` §4.5（行 516）

- **问题**：§7.1 裁定句写"方案 `l4-signing-criteria` §4.5 的'902=文本内容本身差'由 spec-writer dated 更正"。
  但 task-24 复审 E6 已确认：该表述**不在 `l4-signing-criteria.md` §4.5**（§4.5 是 fallback 不适用域），
  正确出处是 `l4-visual-plan.md:349-350@1be66e5`。本手册把已修正的错误又带回来了（回归）。
- **修正建议**：改为"方案 `l4-visual-plan.md:349-350@1be66e5` 的 '902=文本内容本身差' 由 spec-writer dated 更正
  （非 l4-signing-criteria §4.5）"。

#### F7（重要）E4 要求 Python ≥3.8，但手册脚本/仓库工具使用 3.10+ 的 `Path.write_text(newline=)`（E4:44、§3.2:296、§5.3:474）

- **事实**：`pathlib.Path.write_text(..., newline="\n")` 的 `newline` 参数 **Python 3.10 才加入**；
  手册 §3.2 生成器（296 行）与 §5.3 wrapper（474-476 行）都用了它；仓库既有工具 `tools/scenario_run.py:570`、
  `tools/diff_visual.py:242` 也已用 `newline="\n"`（即当前仓库实际要求 ≥3.10，不是 3.8）。
- **后果**：若用户环境按 E4 装 3.8/3.9，P0-0 第一步生成 overlay/wrapper 就 `TypeError`；E4 通过但执行失败。
- **修正建议**：E4 改 **python3 ≥3.10**（推荐）或把手册片段改为 `open(path,"w",encoding="utf-8",newline="\n")`；
  并在 E4 校验里加 `python3 -c "import sys; assert sys.version_info >= (3,10)"`（或对应版本判断）。

#### F8（重要）§7.3 允许 actual documented duplicate，§7.5/§8.2 要求 actual 22 unique——口径需统一（行 539 vs 559/593）

- **问题**：§7.3 判据"五帧 actual 两两不同 blob（**除经裁定的 documented duplicate**）"；§7.5 却写
  "actual：22 个互不同 blob（若仍有重复 ⇒ blocked 排查）"，§8.2 checklist 同。
  若 ending 静态页确实同图，按 §7.5 会被判"渲染未区分"而 blocked，与 §7.3 的裁定通道矛盾。
- **修正建议**：§7.5/§8.2 改为"actual 22 中 documented duplicate 需在 manifest 登记并注明理由，其余必须互不同"，
  与 §7.3 一致。

### 3.3 次要（P2：事实/口径小错，随修复一并改）

| # | 位置 | 问题 | 修正 |
|---|---|---|---|
| F9 | 附录 C-6（726 行） | "`render_frame` 的 idx 只存 `ntr`" **不准确**：idx 实际写 `{palette_src,attrs,tiles}`（`render_frame.rs:1002-1008`），tiles/attrs 取 `nt[ntr]`；**未记录 ntr 字段**（ntr 仅内存默认 0） | 改为"idx 只持久化 RENDER_NT 选中的单个 nametable 的 attrs/tiles（默认 ntr=0，未记录 ntr）；nt0 归属依赖默认值，manifest 必须记 env" |
| F10 | §0:25、§2.4:200、附录 C-3（723 行） | probe 上限口径不一致：`capture.lua` 是 `probeN > 3000000` ⇒ 最多 **3,000,001** 事件 + 头行；§0/C 说"硬上限 3,000,000"，§2.4 又说允许 ≤3,000,001 | 统一写"实现上限=3,000,001 事件（`probeN>3000000` 后静默丢）；监控行数 <3,000,000 安全裕度" |
| F11 | §2.6 产物清单（224 行） | 列了 `title-menu.probe.jsonl`，但 I1 明确 title 无 probe_mem、`probe sig=0` ⇒ 该文件不会产出 | 清单按场景分列（title 无 probe），或注明"无 probe 场景不产" |
| F12 | §2.3（171 行） | run2 复制命令字面为 `cp analysis/traces/... analysis/repro/run2/ ; cp analysis/phase/...`（省略号占位），不可直接执行 | 给出与 run1 对应的完整 run2 cp 命令（或脚本） |
| F13 | §8.1（574-582 行） | 产物替换/归档只有 `mkdir`+注释+`git ls-files`，**没有实际复制/替换/归档 old expected/actual 的命令** | 补 `cp -a`/`git rm`/`.archive` 的具体命令与顺序（含旧 expected 归档） |
| F14 | §4.1（365-367 行） | 回归判据"改前/改后两个二进制同输入同 env 输出 png sha 相同"未给具体做法（如何拿旧二进制） | 给 `git worktree add`/`git stash` + 两次 build 的命令模板（或写明由执行卡自定，但给出可复现步骤） |
| F15 | §7.2（520 行） | "四张 expected 同 blob（**shop 屏**，6,074B）"——"shop 屏"语义无指针；A 级只支撑"同 blob"，OBS-1（`docs/bug-ledger.md:27`）是"末关商店出店转场截断"，可作 B 级解释 | 补指针：同 blob=sha 事实（A）；"shop 屏"引 OBS-1（B），不要写成 A |
| F16 | §3.3（317 行）/§3.5（346 行） | ①"±0 头行扣除"表述含混（各场景头行数不同）；组 A 的 capture 标签（`stage-start`/`mid-combat`/`end`，来自 `gameplay-stage1.txt` 的 capture 行）未写入 mapping.tsv 列说明 | 明确"总行数 − 该场景头行 = 事件数"；mapping.tsv 增加 capture_tag 并给组 A 三个标签 |
| F17 | §7.5（560 行） | "所有 DUO 必须写入 manifest" 拼写错（应为 DUP） | 改 DUP |

### 3.4 建议（P3：提升可执行性/可验收性）

- **S1**：完成 F1/F3 后把 §5.3 wrapper 升级为真正可跑的 P0-0 工具：读 `mapping.tsv`→填 22 pair、canonical env、
  逐对 expected/actual/input sha、dedupe、`--strict` 记录、失败 pair `blocked` 落盘；加 `if not PAIRS: raise`。
- **S2**：§7.5 去重脚本改为以 manifest pair 名为输入（而不是 glob 整目录），并把 expected/actual 目标 unique 写成
  "P0-0 后公式"；输出同时落 `analysis/repro/dedupe.tsv`。
- **S3**：§1.5/§11 增加 Linux 系统依赖清单（Xvfb/X11 开发库等，winit/softbuffer 构建可能需要），避免 E5 `cargo build`
  在干净环境失败；环境接收时一并核对。
- **S4**：附录 D.1 D8 状态"overlay/wrapper 已写就绪"应随 F1/F3 修正更新（当前 wrapper 为骨架）。
- **S5**：把组 D 的"同余"与"来源"分开写更醒目（D2 已做，正文 §3.4 可加交叉引用），避免读者把
  `capture_every 1500` 同余误当来源证据。
- **S6**：手册引用的时点 HEAD `167d8a9` 与当前仓库 HEAD 不同；建议在 §11 注明"分析时点"，并在执行卡里重新核对
  `--out-dir` 是否已实现（当前 HEAD 仍未实现，手册事实目前成立）。

## 4. P0-0 执行前的修正顺序建议

1. **F1**（overlay probe_mem 去重）+ **F4**（头行数判据）一起改，避免新 probe 数据一生成就是错的；
2. **F3**（wrapper 完整实现/降级自述）+ **F2/F8**（去重/unique 口径）一起改，使 P0-0 交付物可验收；
3. **F5**（两遍 capture）+ **F6**（错误指针）+ **F7**（Python 版本）在 §2/§3/§7 中同步；
4. F9-F17 随文一并修；S1-S6 可并入执行卡。

## 5. 复核边界与声明

- 本复核**未**在 Linux/Mesen2 实跑采集/渲染（环境未就绪，属 §9.2 边界）；F1 的后果是按源码/工具行为
  静态+脚本模拟推得（`capture.lua` 无 dedupe、overlay 16 行实测），**不是**低估：一旦按手册执行即会触发。
- 本复核未修改被审手册、未改实现/工具/资产；仅新增本文件。工作区其他改动（并发任务）与本复核无关。
- 本复核的"通过/驳回"只针对**手册作为 P0-0 执行依据**，不构成对 L4 现状或方案本身的判定。
- 复核命令摘要（只读）：
  ```text
  # 附录 A：解析方案 §2.6 与附录 A 各 22 行 -> 逐项比对 0 mismatch
  # 附录 B：grep render_frame.rs env::var -> 40（39 诊断 + REPO_ROOT），附录 B 39 项全等
  # 组 D：v5 frames=300000 / capture_every=1500 / bot from 1260 / 3 poke；
  #       28500..36000 = 19..24 × 1500，全部 ≤36500；v4/v3/v2 frames=30000
  # overlay 模拟：stg03 源 probe_mem 8 -> 手册 make = 16（unique 8）；capture.lua 无 dedupe
  # probe：gameplay 15 条、stg03/stg08 8 条、v5 0 条；capture.lua probeN>3000000
  # 去重现状：expected unique 17/22；actual unique 127/257（含大量实验图）
  # 工具：Mesen2 2.1.1 文档依据；bins render-frame/phase-batch；run.json 字段齐全；
  #       shots/<name>_f%05d_<tag>.png；gates/cli.py rom=PASS；diff_visual 两模式
  # 文档：本手册相对链接 0 断链；被审文件 783 行/sha 9f548f2a 稳定
  ```

## 6. 结论

**needs_revision**。手册的**数据面/事实核验（附录 A/B、组 D 同余、D3/D5/D6、工具路径/bin/输出、纪律边界、
P0-0 vs P0-4 分工、门禁两步走）可信且复核通过**；但 **F1 overlay probe 重复、F2 去重验收口径错、
F3 wrapper 空跑/不完整**会直接导致 P0-0 产出错误或假绿，必须修正；F4-F8 为重要项，也应同批修。
修正后请提交复审（重点复核 F1 的 overlay 输出头行/事件数、F2 的去重脚本对 22 对过滤、F3 的 wrapper 端到端
（含空 PAIRS 断言）、F5 两遍 capture、F6 指针、F7 Python 版本）。

---

## 复审（2026-10-18，task-33）

> 复审对象：[l4-baseline-freeze-plan.md](l4-baseline-freeze-plan.md) **1163 行，sha256 `4df844f65d21fe05…`**
> （mtime 13:53:37；l4-analyst 已 inactive）。相对前版 783 行 +380 行（含 §13 修正映射、§0.5 执行顺序、
> §3.0 capture 源现况、§7a/§7b/包装器/去重脚本/命令补齐）。本复核只核 **F1-F8** + 抽查 **§13/S1-S6** + **§0.5**，
> 不重审已确认事实面；仍用独立重算/重跑只读命令，不采信自述。

### 复审 VERDICT：**needs_revision**

本轮修正量很大：**F1（overlay 去重）、F4（逐场景头行表）、F5（两遍机制）、F6（指针）、F7（Python≥3.10）**
已实质闭环；**F2/F3/F8 的文本口径也改对了**（expected∈{21,22}、actual=22−documented_dups、按 manifest 22 名过滤）。
但仍有 **3 个会破坏 P0-0 验收/产出的阻塞项 + 2 个重要项**（都是小范围文档/脚本修，但必须修后再执行）。

### 1. F1（overlay probe 去重）——✅ 已闭环

- §3.2（376-446 行）已改为"读源已注册区间集合，只追加缺失"：`have.add(...)` + `missing` + 生成后
  `grep -c '^probe_mem '` / 重复区间断言。我抽其代码独立模拟（临时 dst，不动仓库）：
  - `stg03-timeline`（源 8）→ `make` 输出 **8（source=8 + missing=0）**，captures=1（5400）；
  - `tas-clear-full-v5`（源 0）→ 输出 **8（source=0 + missing=8）**，captures=0（由源 `capture_every 1500` 提供）。
  - 源文件 `grep -c '^probe_mem '`：gameplay **15**、stg03 **8**、stg08 **8**、title **0**、v5 **0** ——与 §2.1/§2.4 表一致。
- **关键简化核对（逐帧源 capture）**：
  - gameplay：`capture 1260 stage-start` / `1500 mid-combat` / `2349 end` ✓；
  - stg03：`3404 stg02-start` / `4600 stg02-clear-inject` / `5700 stg03-start-approx` / `6500 stg03-combat` /
    `12000 stg03-late` ✓；**5400 无显式 capture 行**（§3.0 表正确），但源有 `capture_every 300 probe`（5400=18×300，
    运行会产出 `stg03-timeline_f05400_probe.png`，见 §6 的 M2）；
  - stg08：`14600/16000/16600/18000/22000/28000` 全部有显式 capture ✓；
  - title：`capture 180 boot-black` ✓；组 D v5：`capture_every 1500` 覆盖 28500/30000…/36000 ✓。
  结论：§3.0"组 A/B/C/E 目标帧 capture 已在源场景"**基本成立**；I3b/I5 两个 overlay 的定位基本正确（M2 见后）。

### 2. F4（逐场景头行表）——✅ 已闭环

§2.4（233-272 行）表 + §2.2（178-180）+ §3.3（447-459）已统一为：
title **0/无 `.probe.jsonl`**；gameplay **15**；stg03 **8**；`l4-baseline-stg03`（overlay）**8**；
stg08 **8**；`l4-baseline-v5`（overlay）**8**。我按源场景 `grep -c '^probe_mem '` 与模拟 overlay 输出核对
（15/8/8/0 + overlay 8/8），与表一致；`capture.lua` 确为每个注册写 1 条独立头行（不是"首行 8 条"），
脚本用 `startswith('{"probe"')` 计头行、`len−hdr=事件数` 写法正确。**F4 通过**（仅 I5b 未入表，见 M4）。

### 3. F5 / F6——F5 机制 ✅；F6 指针 ✅；但 §7.1 动作仍有一处错误（见 B5）

- **F5**：§2.1（I5b 行 126）、§2.2（167 行 phase-batch pass2）、§7.2（797-818 行）已给出两遍机制：
  第一遍 I5 出 `l4-baseline-v5.frames.jsonl` → `pick` 选帧写 `l4-selected-frames.json` →
  `make(..., captures=[...])` 生成 `l4-baseline-v5-pass2` → 重跑取 `shots/l4-baseline-v5-pass2_f<f>_l4f<f>.png`。
  我核对 `capture.lua` 的 trace 格式确为 `{"f":N,"w":{label:val}}`、v5 有 `watch 0180 stage_no` /
  `watch 0188 intro_ctr` / `watch 0189 w0189`，示例脚本键名可用；`capture` 只能对预置帧出图的两遍原理正确。✅
- **F6**：§7.1（789-790 行）已改指 **`l4-visual-plan.md:349-350@1be66e5`**（并显式注明非
  `l4-signing-criteria` §4.5）；expected_source 改为 `stg03-timeline/3404/stg02-start`。指针本身 ✅。
- **但 §7.1 动作行（784 行）仍写"从组 B overlay（`l4-baseline-stg03`）在候选帧 3404 capture"**：
  I3b 的 captures 只有 `[5400]`（§3.2:422），overlay **不会**产出 3404 的 `l4f3404` 图；而源
  `stg03-timeline` 已有 `capture 3404 stg02-start`（shots `stg03-timeline_f03404_stg02-start.png`）。
  ⇒ 按文字执行会找不存在的 `l4-baseline-stg03_f03404_l4f3404.png`。应改为源场景 capture（或把 3404 加进
  I3b captures）。这是 B5。

### 4. F7（Python）——✅ 已闭环

E4（60 行）已改 **python3 ≥3.10**，带 `assert sys.version_info >= (3,10)` 校验；§3.2 生成器（388/415 行）、
§5.3 wrapper（739 行）均保持 `Path.write_text(..., newline="\n")`（3.10+），手册自洽。
（仓库 `tools/scenario_run.py:570`、`tools/diff_visual.py:242` 也确为 3.10+ 写法——原 F7 依据成立。）

### 5. F2 / F3 / F8——文本口径 ✅，脚本/字段仍有落地缺口（B2/M1）

- **F2 文本**：§5.1（589-596）、§7.5（839-871）、§8.2（925-935）已统一为
  `expected_unique ∈ {21,22}`、`actual_unique = 22 − documented_duplicates 数`，并明确 17/127 只是
  **P0-0 前背景**；§7.5 脚本改用 `manifest["pairs"]` 的 22 个 name + `actual_canonical_run1`，不再 glob 整个
  `actual/`（脚本语法我编译通过；当前实际目录 257/127 的背景数字也确实如此）。**公式/过滤面 ✅。**
- **F3 文本**：§5.3（621-753）已加 `len(rows)!=22 ⇒ BLOCKED`、`if not rows ⇒ BLOCKED`、
  `render` 缺输出即 exit、`det && dedupe_ok` 才返回 0、失败 exit1——**空跑假绿已消除** ✅；
  manifest 已含 expected/input_sha/render_env/argv/status/reuse/capture_tag/trigger/deterministic 字段名 ✅。
- **F8 文本**：§7.3（830-832）、§7.5（869-871）、§8.2（930-931）已统一 actual documented duplicate 口径。✅
- **但仍存在实现/字段语义缺口（B2/M1/M3）**：
  - §5.3 `dedupe()`（677-688 行）**只从 `act`（actual）构造 `documented_duplicates`**，未收集 expected 侧重复
    （ending-4/5 若 expected 仍同文件、actual 不同 → expected unique=21 被接受，但 manifest 未登记该 expected
    重复，违反 §7.5/§8.2"所有重复必须登记"）。
  - §5.3 `dedupe_ok`（722-724 行）用 `documented_act == len(allowed_doc)`：若 mapping 只给重复对中一帧标
    `duplicate_of`（常见约定），`allowed_doc=1` 而 `documented_act=2` ⇒ **合法 documented duplicate 被判 FAIL**；
    反之若两个无关行都标了 `duplicate_of`，可能放行未登记组。没有按"重复组 ↔ mapping 组"逐组核对，也未直接断言
    `actual_unique == 22 − Σ(group_size−1)`。
  - §7.5 脚本（859-866）只验 `len(A)==22-doc`，**没有断言 expected 重复 ⊆ manifest.documented_duplicates**；
    其 `doc` 还假设每个 documented 组只含 2 帧（若 3 帧组，`22-doc` 会高估 unique）。
  - **F3 manifest 字段值**：`reuse` 取 `p.get("reuse") or "independent"`（713-716 行），但 §3.5 mapping.tsv
    schema（508-511 行）**没有 `reuse` 列** ⇒ 所有行（含 ending/四联非独立）都默认 `independent`；
    `status` 也总是 `"ok"`/`blocked:nondeterministic`，未从 mapping 的 status 列透传
    `replaced-invalid-asset`/`documented-duplicate`。字段存在但语义不对，F3 要求未真正满足。
  - **F13/F3 的 strict 回填**：§8.1 步骤 5（918-919 行）只有注释、没有命令；`--strict` 记录要回填
    manifest.strict，但 §5.3 wrapper 只在带 `--strict` 参数时写 `strict`，而 §0.5 中 wrapper 跑在 §8 资产替换
    之前；没有"资产换新后重跑 wrapper --strict 或脚本回填"的可执行命令。§8.2（932 行）却把该记录列为验收项。
  - **F13 的 expected 替换命令**：§8.1 步骤 3（912-914 行）只给 stg02 一个示例，且是从
    `analysis/expected-new/...` 复制，但**`analysis/expected-new` 在全文再没有定义/生成命令**（仅 914 行出现），
    §7.2 只说"取 shots/... 作为新 expected 源图"，没有落到 expected-new/或 `oracle/screens/expected/` 的命令。

### 6. §13 修正映射抽查（F9-F17 / S1-S6）

- **已落实**：F9（附录 C-6 改为"只持久化 RENDER_NT 选中的单个 nametable，未记 ntr"）、F10（上限统一
  3,000,001 事件）、F11（§2.5/§2.6 title 无 probe 不产出）、F12（§2.3 run2 完整命令，无省略号）、
  F15（同 blob=A、商店冻结屏=B 引 OBS-1）、F16（§3.0/§3.1/§3.3/§3.5 增 capture_tag/duplicate_of）、
  F17（DUO→DUP）、S3（E11 系统依赖）、S4（附录 D D8 状态更新）、S5（同余≠来源交叉引用）、
  S6（头部标"分析时点 HEAD 167d8a9，执行前重核 --out-dir"）——抽查通过。
- **部分落实**：F13（§8.1 有归档/安装命令，但 strict 回填与 expected 替换仍不完整，见 B3/M6）、
  F14（§4.1 给 worktree/AB 步骤，但旧二进制仍带 `--out-dir` 调用，见 B4）、F2/F3/F8（文本✅，实现见 B2/M1）、
  S1/S2（wrapper/dedupe 已建但存在 B2/M1 缺口）。

### 7. §0.5 执行顺序（B1：expected 安装点与 manifest 冲突）

§0.5（38-47 行）写成：
`§1 → §2 → §3 → §4 → §7a（资产重采/裁定，含两遍 capture）→ §5 wrapper+manifest → §7b ending actual 重渲/dup →
§6 两跑 → §8 冻结`。
**问题**：§5.3 wrapper 的 `expected_sha256` 读的是 `oracle/screens/expected/{name}.png`；若 §7a 只是
"收集/裁定"新 expected（§7.2 的 shots、§7.1 的源 capture），而**安装到 `oracle/screens/expected/` 的动作在
§8.1 步骤 3（912-914 行，位于 §6 之后）**——则 §5 manifest 记录的是**旧无效 expected**（stg02=boot-black、
四联=同 blob），P0-0 产物自相矛盾。反之若 §7a 已经"取代旧文件"（§7.1:787），§8.1 步骤 3 又会把**新文件当旧件
归档**再复制一遍，且 `analysis/expected-new` 未定义。§7b（§7.3:826-829）若在 §5 之后决定替换 expected-4/5，
同样会让 manifest 的 expected_sha 过期。
**修正**：在 §0.5/§7/§8.1 明确唯一安装点（建议 §7a 把新 expected 落到 `oracle/screens/expected/` 并归档旧件，
§8.1 只做"验证/记录 + diff 重生成"，不再复制），或 wrapper 改读 staging 目录并最后统一安装；所有 expected
替换必须发生在 §5 wrapper 之前；§8.1 删除重复安装或改为核对。

### 8. 重要/剩余问题（修正建议）

| # | 级 | 位置 | 问题 | 建议 |
|---|---|---|---|---|
| **B1** | **阻塞** | §0.5:38-47、§7.1:787、§7.2:805-806、§8.1:912-914 | expected 安装点与 §5 manifest expected_sha 冲突（见 §7）；`analysis/expected-new` 未定义 | 明确唯一安装点（§7a 安装/§8.1 只核验），补 expected-new→expected 命令；所有 expected 替换先于 §5 |
| **B2** | **阻塞** | §5.3:677-688、722-724；§7.5:859-866；§8.2:930-931 | expected 重复未登记；`dedupe_ok` 的 `documented_act==len(allowed_doc)` 在"只标一帧"时会误 FAIL、在"两无关行"时可能误 PASS；`22-doc` 未按组大小算；expected 重复未纳入 manifest | `dedupe()` 同时收 expected+actual 重复组；按 mapping.duplicate_of 逐组核对；断言 `actual_unique==22−Σ(len(group)−1)`；expected 重复也写 documented_duplicates；§7.5 加"所有重复 ⊆ manifest"断言 |
| **B3** | **阻塞** | §8.1:912-919、§8.2:932 | strict 记录回填无命令；expected 替换仅 stg02 示例且源目录未定义 | 补"资产换新后重跑 wrapper --strict 或读 visual-strict.log 回填 manifest.strict"的命令；expected 替换给出四个资产集合（stg02/四联/ending-4/5/fade）的归档+落新命令与 staging 目录 |
| **B4** | 重要 | §4.1:531-544 | 旧二进制（`<change-commit>~1`）**尚无 `--out-dir`**，第 3 步却给旧二进制传 `--out-dir` 并 `|| true`；随后 sha 比较缺文件；fallback 仅一句文字、无命令 | 旧二进制改用 `REPO_ROOT=/tmp/reg-old-sandbox` 沙盒（§4.2）并给完整命令；新二进制用 `--out-dir`；两者各出 png 再 sha 对比 |
| **B5** | 重要 | §7.1:784 | 写"从 `l4-baseline-stg03` overlay 在 3404 capture"，但 I3b 只加 5400 capture（§3.2:422），不存在 3404 overlay 图 | 改为源 `stg03-timeline` 的 `capture 3404 stg02-start`（shots `stg03-timeline_f03404_stg02-start.png`），或把 3404 加进 I3b captures |
| **M1** | 重要 | §3.5:508-511；§5.3:713-716 | mapping.tsv 无 `reuse` 列 ⇒ wrapper 全部默认 `independent`；`status` 未从 mapping 透传 | 增 `reuse` 列（按 §2.6/附录 A 的独立/复用分组）并在 wrapper 读 `p["status"]` |
| **M2** | 中低 | §3.0:330；§2.1:123 | #13 的 5400 实际已被源 `capture_every 300` 覆盖（`stg03-timeline_f05400_probe.png`）；I3b"仅补 capture 5400"并非唯一路径、多跑 10-20min | 说明为何仍需 I3b（如要稳定 tag l4f5400），或直接用源 capture_every 图、删 I3b |
| **M3** | 中低 | §7.2:802-815 | 文字要求 combat"中位帧"、late"接近 clear/boss 的帧"，示例脚本却是 `next()` 首匹配；`w0189>0` 只是启发式；未写 `analysis/expected-new`/pass2 产物如何成为 expected | 统一选帧判据（脚本与文字一致）；给 pass2 shots → expected 安装命令与路径；pick 加 StopIteration 处理 |
| **M4** | 低 | §2.2:167；§2.4:250-251 | phase-batch `l4-baseline-v5-pass2` 写在命令块，但 pass2 场景/输入到 §7.2 才生成；§2.4 EXP 表未含 pass2 | 把 pass2 phase-batch 移到 §7.2，或在命令处显式注"§7.2 后再跑"；EXP 增 I5b 或注明不检查 |
| **M5** | 低 | §5.1:583 vs §7.1:788 | expected_source 在 schema 是字符串（`"stg03-timeline:4600:..."`），§7.1 写成对象 `{scenario,frame,...}` | 统一为字符串（或两者都对象）；mapping.tsv expected_source 列格式写死 |
| **M6** | 低 | §8.1:912-914 | expected 替换只示例 stg02；四联/ending-4/5/fade 的归档+落新命令缺失（与 B3 相关） | 按 §7.1-7.4 四类逐条给命令或写成一个脚本 |
| **M7** | 低 | §2.4:250；§5.3:674 | title 无 probe → `input_sha.probe=null`；`env.os` 仅 `os.uname().machine`（不是 "Linux x86_64"） | 注明 null 合法/或省略该 key；env.os 拼 uname.sysname+machine |
| **M8** | 低 | §2.3:228 | "frames/probe/text 三类 × 6 场景集合"实际 probe/text 只有 5 个场景（title 仅 frames） | 改为"6 个 frames 集合 + 5 个 probe/text 集合" |

### 9. 独立复核通过的正向项（防止误伤）

- **F1** overlay 去重逻辑正确（模拟：stg03 8、v5 8；断言脚本可跑）；源场景 capture 事实逐帧核对见 §1。
- **F4** 头行表与源一致（title 0/无文件、gameplay 15、stg03 8、stg03-overlay 8、stg08 8、v5-overlay 8）。
- **F5** 两遍机制/`frames.jsonl` 格式/`watch` 键核对成立；**F6** 指针正确；**F7** ≥3.10 与代码一致。
- **F2/F8 文本**公式统一；**F3** 空 PAIRS/22 行断言已加；§5.3 wrapper 与 §2.4/§7.5/§8.1 中的 Python
  代码块均通过 `compile()` 语法检查；§13 映射表 **23 行（F1-F17 + S1-S6）齐全**；§0.5 结构清晰（除 B1）。
- **文档**：相对链接 **0 断链**；被审文件复核期稳定（**1163 行/sha `4df844f6`**）。
- 命令输出摘要（只读）：
  ```text
  grep -c '^probe_mem '：gameplay 15 / stg03 8 / stg08 8 / title 0 / v5 0
  capture 事实：gameplay 1260/1500/2349；stg03 3404/4600/5700/6500/12000（无5400）；
               stg08 14600/16000/16600/18000/22000/28000；title 180；v5 capture_every 1500
  overlay 模拟（手册 §3.2 代码，临时 dst）：stg03 probe_mem=8（源8+缺0）/ captures=1；
                                        v5 probe_mem=8（源0+缺8）/ captures=0
  Python 代码块：§3.2/§5.3/§2.4/§2.3/§3.3/§7.2/§7.5/§8.1 共 8 段 + 3 段围栏块 compile 全 OK
  去重现状：expected 17/22；actual 目录 257/127（§7.5 已改用 manifest 22 名，方向正确）
  ```

### 10. 复审结论

**needs_revision**：F1/F4/F5/F6/F7 已闭环；F2/F3/F8 的文本口径已改对，但 **B1（expected 安装点/manifest 顺序）、
B2（去重登记与 dedupe_ok）、B3（strict 回填/expected 替换命令）** 会直接影响 P0-0 验收真实性，必须修；
**B4（旧二进制 AB 回归）、B5（3404 overlay 路径）** 为执行会踩坑的重要项；M1（reuse/status）影响 manifest
provenance。修正后（建议一段 doc 改动即可）再复审本轮 B1/B2/B3 + B4/B5 即可放行。

---

## 复审（2026-10-18，task-35）

> 复审对象：[l4-baseline-freeze-plan.md](l4-baseline-freeze-plan.md) **1406 行，sha256 `7b58beafaf424a69…`**
> （mtime 14:05:09；l4-analyst 已 inactive）。相对 1163 行 +243（§7.7 expected 唯一安装、B1-B5/M1 修正、
> §14 映射）；本复核只核 **B1-B5/M1** + 抽查 **M2-M8 / §14 / §0.5**，F1/F4/F5/F6/F7 已在上轮确认闭合。

### 复审 VERDICT：**pass**（附 5 条非阻塞残留；R1/R2 建议提交前顺手修）

B1-B5/M1 六项**均实质闭环**：唯一安装点/时序、expected+actual 双侧去重与 registry、strict 回填命令、
旧二进制沙盒 A/B、3404 源 capture、mapping 必需列与透传都已落地。剩余为**映射状态写回步骤**（R1）与
**wrapper 一条 side 的 sha 取错**（R2）等非阻塞项；不阻塞 P0-0 启动，建议提交前改。

### 1. B1（expected 唯一安装点/时序）——✅ 已闭环

- §0.5（38-54 行）已明确：§7a 完成全部 expected 安装（stg02/四联/ending-4/5/fade，`analysis/expected-new/` →
  `oracle/screens/expected/` + 旧件归档），**先于 §5**；§7b 只做 actual 侧核对；§8 只核验不改 expected。
- §7.7（988-1068 行）给出 A) 四类 staging（含 ending-1..5）→ B) 归档旧件+落新件 → C) 安装后自检；
  `analysis/expected-new` 已定义为 staging 目录；§7.1/§7.2/§7.3/§7.4 均指向它。
- §8.1 步骤 3（1103-1115 行）已改为**只核验** manifest `expected_sha256` == 实际文件 sha，不再复制/归档；
  ending-4/5 裁定已在 §7.3 §7a（919-930 行）完成，§7b 仅 actual 侧。
- 我核对 `§7.7 A` 的源图路径：stg02=`shots/stg03-timeline_f03404_stg02-start.png`；四联=pass2
  `_f<frame>_l4f<frame>.png`；ending/fade=`l4-baseline-v5_f<frame>_probe.png`——与 capture.lua 命名规则一致。
- **残留 R1**：§7.7 的 A/B 与 §5.3 wrapper 都**读 mapping.tsv 的 status/reuse/duplicate_of**，但 §7.1/7.3/7.4
  的"status=..."只写了裁定值，**没有给出写回/更新 mapping.tsv 的命令**（§7 引言 843 行只说"随安装定稿"）。
  若 operator 不在 §7a 显式更新 mapping：conditional 替换（ending-1..3/fade 的 `replaced-invalid-asset`）
  可能不 staging；wrapper 的 status/reuse/duplicate_of 可能仍是 §3.5 初值，B2 registry_ok 可能失败或
  manifest provenance 失真。**建议 §7.7 加 step 0：按 §7.1-§7.4 裁定写回 mapping.tsv（status/duplicate_of），
  或给一条 python 更新命令**。属中优先、不阻塞正常路径（stg02/四联无条件 staging）。

### 2. B2（dedupe 双侧/公式/registry）——✅ 已闭环（一条 sha 细节 R2）

- §5.3 `dedupe()`（718-750 行）已收 **expected+actual** 双侧重复，`side` 标 `expected+actual/expected/actual`；
  `documented_duplicates` 同时登记 expected 侧（不再只 actual）。
- `dedupe_ok`（746-747 行）已改为 **按组公式** `len(exp)==22−Σ(len−1)`、`len(act)==22−Σ(len−1)` + 范围
  `21/22` + `registry_ok`（组内名字 ⊆ mapping `duplicate_of` 组）；§7.5 脚本（944-974 行）用
  manifest 22 名、`actual_canonical_run1`，并断言 `covered(Edup)`/`covered(Adup)`（重复组 ⊆
  `documented_duplicates`），输出 `dedupe.tsv`；§8.2 验收口径同步（1146-1148 行）。
- **残留 R2（中低）**：wrapper `doc` 的 `sha256` 恒取
  `oracle/screens/expected/<v[0]>.png`（732-735 行）；对 **side=`actual`** 的重复组应取 canonical actual 的
  sha（`OUT1/<v[0]>.png`），否则 `documented_duplicates[].sha256` 写的是 expected sha；建议按 side 取对应
  sha，或同时记 `expected_sha256`/`actual_sha256`。
- **残留 R3（低）**：§7.5 独立脚本的 `ok` 只按"unique==22−Σ(len−1)"（组数定义下恒真）+ covered，
  没有显式 `expected_unique in {21,22}`/`actual_unique in {21,22}`（wrapper 有）；建议补一行范围断言，
  避免独立跑 §7.5 时放过 20/23 等异常组。

### 3. B3（strict 回填/四类资产命令）——✅ 已闭环

- §8.1 步骤 4（1117-1119 行）：actual 换新后先跑默认 diff，再跑 `--strict`，日志落
  `analysis/repro/visual-strict.log`；步骤 5（1121-1134 行）给出完整 python：从日志取 `STRICT:` 行 →
  写 `manifest.strict = {cmd, exit, line, ran_after_asset_swap:true}`；时序（actual 换新之后）正确。
- 四类资产 staging/归档/落新收敛到 §7.7（998-1053 行），不再只有 stg02 示例；`analysis/expected-new`
  定义/生成/安装命令齐全。验收 §8.2（1149-1150 行）引用 strict 回填。
- 我核对 `diff_visual.py --strict` 输出确为 `STRICT: FAIL/PASS diff=<n>`（tool:274-277），步骤 5 的
  `startswith("STRICT: FAIL")` 判定成立。**B3 通过。**

### 4. B4（旧二进制沙盒/新 --out-dir 的 A/B）——✅ 已闭环

- §4.1（531-573 行）已把旧二进制走 **REPO_ROOT 沙盒**（mkdir/symlink/运行→`$OLDSB/oracle/screens/actual`），
  新二进制走 `--out-dir`（`$NEWOUT`）；两路 png 都产出后 `sha256sum` 对比；**已删除 `|| true`**，
  缺文件即命令失败/FAIL；fallback 也给了文字说明。
- 我核对 `REPO_ROOT` 在改前版本已存在（git `-S REPO_ROOT` 见 `77c70b8`，早于本轮 change-commit），
  旧二进制走沙盒可行。**B4 通过。**

### 5. B5（3404 源 capture）——✅ 已闭环

- §7.1（850-852 行）已改指源 `stg03-timeline` 的 `capture 3404 stg02-start`
  （`shots/stg03-timeline_f03404_stg02-start.png`），并明确"不是 I3b overlay——I3b 已按 M2 取消"。
- 全文 I3b 引用仅剩"已取消/备选 fallback"语境（§7.1 fallback、§13/§14 映射说明），**无实际 I3b 命令/悬空
  文件路径**；§3.0/§3.1/§3.3 的 #14 均已用源 capture。**B5 通过**（R5 见后：fallback 文案建议改词）。

### 6. M1（mapping reuse/status/透传/缺列 BLOCKED）——✅ 已闭环

- §3.5（514-524 行）mapping.tsv schema 已含 `status`/`reuse`/`duplicate_of`，并定义值域；
- §5.3 `load_pairs()`（674-689 行）`REQUIRED` 含 status/reuse，缺列 ⇒ `SystemExit BLOCKED`；
  manifest（769-781 行）`"status": "blocked:nondeterministic" if not ok else p.get("status")`、
  `"reuse": p["reuse"]`，**已透传**；§5.1 schema 同步（601-634 行）。
- **R1（同前）**：mapping 的 `status/duplicate_of` 值在 §7a 裁定后如何写回未给命令；`registry_ok`/manifest
  依赖它。建议 §7.7 显式补写回步骤。

### 7. M2-M8 / §14 抽查

| # | 落点 | 复核 |
|---|---|---|
| M2 | §2.1:134-138/192-193、§3.0:336、§3.1/§3.3 #13、§3.4 | ✅ I3b 取消；#13 用 `stg03-timeline_f05400_probe.png`（5400=18×300），§3.3 ③ 路径已改 |
| M3 | §7.2:865-911 | ✅ 文字=脚本（start 首个 / combat 中位 / late,stg06 last），`first/median/last` + StopIteration；pass2 生成/重跑/phase-batch 命令完整；staging 路径明确（R4 仅代码块缩进） |
| M4 | §2.1/§2.2:164-173、§2.4:250 | ✅ pass2 phase-batch 已移 §7.2；§2.4 增 pass2 行并注明事件数不作校验 |
| M5 | §3.5:518-519、§5.1:611、§7.1:855 | ✅ expected_source 统一字符串 `scene:frame:capture_tag`（§7.1 stg02 示例一致） |
| M6 | §7.7:988-1053 | ✅ 四类资产 staging+归档+落新脚本；`analysis/expected-new` 定义 |
| M7 | §5.3:706-710/784-785 | ✅ title probe null 注明合法；`env.os=sysname+machine` |
| M8 | §2.3:234 | ✅ "6 frames + 5 probe/text"（title 仅 frames） |
| §14 | 1386-1406 | ✅ 13 行（B1-B5/M1 + M2-M8）与正文一致；§13 仍保留首轮映射 |

### 8. 独立检查/命令输出摘要

```text
docs-links（我实跑 `python gates/cli.py check`）：PASS 325 md / 372 本地链接全部有效（自称属实）
gates check：PASS=11 FAIL=0 SKIP=1 TOLERATED=0（rust-tests 371 vs 基线 366 为 WARN PENDING，
             与 P0-0 手册无关，属并发任务基线待上调）
plan 稳定：1406 行 / sha 7b58beafaf424a69（复核期间不变）；相对链接 0 断链
代码块：3 个 ```python 块 + 14 个 bash heredoc 块 compile 通过（R4 的 pass2 块因嵌列表缩进提取报错，
        属复制/缩进问题，非语法内容错误）
源事实复核（沿用上轮 + 本轮改点）：gameplay 15 probe/capture 1260/1500/2349；stg03 8 probe、
 capture 3404/4600/5700/6500/12000、capture_every 300（5400）；stg08 8 probe、14600/16000/16600/
 18000/22000/28000；title 0 probe、capture 180；v5 0 probe、capture_every 1500；I3b 无命令
strict 输出格式：`STRICT: FAIL/PASS diff=<n>`（tool:274-277）与 §8.1 步骤 5 判定一致
```

### 9. 剩余问题（非阻塞）

- **R1（中，建议提交前改）**：§7.7/§5.3 读 mapping status/reuse/duplicate_of，但 §7a 裁定后的**写回
  mapping.tsv 步骤/命令缺失**（§7 引言 843 行仅口头"随安装定稿"）；建议 §7.7 A 前加 step 0 写回
  status/duplicate_of，或给 python 更新命令；否则 conditional ending/fade 替换可能未 staging、
  manifest status/reuse/dup 可能仍是 §3.5 初值。
- **R2（中低）**：wrapper §5.3:732-735，`documented_duplicates[].sha256` 对 side=`actual` 仍取 expected sha；
  应按 side 取 `OUT1`/expected 的 sha（或同时记两 sha）。
- **R3（低）**：§7.5 独立脚本未显式断言 expected ∈{21,22}（wrapper 有），建议补范围断言。
- **R4（低）**：§7.2 pass2 bash 代码块嵌在列表内，原始 Markdown 带 4 空格缩进；按原样复制会使
  `PY` 终止符/Python 代码带缩进（heredoc/IndentationError）；建议移到无缩进代码块或注明 dedent。
- **R5（低）**：§7.1:852 fallback 仍写"再把 3404 加入 I3b 备选方案"（I3b 已取消）；建议改为
  "临时重建 stg03 overlay/加 capture 3404" 避免误导。

### 10. 复审结论

**pass**。B1-B5/M1 六项全部实质闭环，M2-M8/§14 抽查通过，docs-links 325/372 实跑 PASS；
R1（mapping 写回命令）/R2（side sha）建议在提交手册前顺手修，R3-R5 随文/执行卡修即可。
P0-0 可按 §0.5 启动（等用户 Linux+Mesen2 环境）；R1 不阻塞启动，但建议先补以避免 §7a 条件替换/
manifest status 在实跑时歧义。

---

## 点验（2026-10-18，task-49）

> 对象：[l4-baseline-freeze-plan.md](l4-baseline-freeze-plan.md) 当前工作区版本（**1622 行 /
> sha256 `16b76accc327…` / mtime 14:31，l4-analyst inactive 稳定**）。说明：Lead 转述 task-48 版本为
> 1555 行/sha `0878badd…`，但复核时终稿已继续并入 §3.7/§15.1（task-50 的四联来源/帧预算更正）；
> 本点验按**当前最终版**执行，聚焦 §3.6 及影响面，§3.7/§15.1 的同类问题一并记录。
> 只读复核 + 本文件写入；未 git add/commit。

### 点验 VERDICT：**needs_revision**

**证据本体成立**：`tas-clear-full.l5.jsonl` 的 f28364/f29078/f29478/f51332 与"v5 28500-36000=关 2
战斗段"结论**可复算**；C1 `tas-stg08b2-direct` 的 10 probe/capture_every 1500/frames 40000/
poke 1265 0180 08+assist 三 poke **逐项属实**；原假设保留、§15 历史链与多数影响面已指向 §3.6。
但仍有 **3 个阻塞/重要问题**：§3.2/§3.3 仍留旧 v5 语义且无更正指针；`stg06-timeline` 的 probe_mem
数量写错（源 0 写成 8）；§3.6 判据②"frames.jsonl 中 mode=Ending"在当前 `frames.jsonl` 格式下不可执行。

### 1. 证据核验——✅

`oracle/traces/tas-clear-full.l5.jsonl` 实读（10 行）：
```
28364 StageClear stg01-clear（tas-clear-full-v5 assist=invincible）
29078 Shop stg02-shop
29478 Combat stg02-combat
51332 StageClear stg02-clear
```
⇒ §3.6 的 A 级反证表**逐字段一致**；v5 在 28500-36000 不可能是 ending/fade（stg02-clear 才 f51332），
"组 D 来源≠v5"结论成立 ✅。C1 的场景文件实测：`frames 40000`、`probe_mem` **10**、
`capture_every 1500 probe`、`poke 1265 0180 08` + assist 三 poke——与 §3.6 C1 理由**全同**；
且全场景扫描中 `tas-stg08b2-direct` 是唯一 probe_mem≥10 的候选（C1 优先级理由成立）✅。
C2/C3/C4 场景均存在，帧数/capture_every/poke 与表一致 ✅。

### 2. 更正合规——原假设保留 ✅；候选/判据/无来源流程——部分 ✅

- §3.6 明确"原假设（保留留档，不删）"并给出 A 级反证；§15/§15.1 明确 §13/§14 为历史留档且不再用
  v5 作组 D 来源 ✅。
- 候选 C1-C5 与负候选均有；C1/C2/C3/C4 证据核过；无来源时 `blocked:no-source` + `s5-full-inject`
  新采集方向有规划 ✅。
- **但三判据的可执行性不齐**：
  - ① blob：候选 capture_every 截图 sha==旧 expected——可执行 ✅；
  - ② mode：写"该帧在候选 `frames.jsonl` 中 mode=Ending"——**当前 `scenario_capture.lua` 写出的
    `frames.jsonl` 只有 `{"f","w":{...}}`，没有 mode/checkpoint 字段**；`tas-clear-full.l5.jsonl` 才是
    L5 golden（含 mode/checkpoint），但 C1/C2/C3/C4 没有对应 `.l5.jsonl`。C1 虽有 `watch 0026 flags26`，
    可从 `$26 bit7`/`$95F9` 推导 Ending，但 §3.6 没有给出映射/命令；因此判据②按文字**不可执行**。
  - ③ trigger：C1 选关 poke ⇒ injected、assist ⇒ assisted——可执行 ✅。
- **C5 `s5-full-inject` 规划**只有"每关 $26:=$10 + $0400:=$01 + 转场 +34f + 帧预算 ~40k + capture/probe"
  的概念级描述；**没有给出场景 DSL 文件、各关注入帧表/帧数、命令行**，若 C1-C4 全失败则无法直接执行；
  建议补一个 scenario 模板或明确"新卡再出"（task 卡要求"规划"，可接受但应标不可直接跑）。

### 3. 影响面检查——❌ 仍有旧 v5 引用/事实错误

- ✅ 已更新：§0.5（新增 §3.6 子步骤）、§0（L26-28）、§2.1（I5-old 划线/更正）、§3.0 表（#6-#11
  来源待重裁）、§3.1 注（#6-#11 作废 + §3.6 指针）、§3.4 D 行、§7.3/§7.4（§3.6 选中来源）、§7.7
  （conditional staging/`l4-groupD-source.json`）、§8.2（§3.6 重裁完成项）、§10（C1-C5 裁定）、
  §15/§15.1（历史勘误）。
- ❌ **§3.2（第 414/421 行）仍写"本手册只需 stg03/v5 两个"、"capture_every 提供 v5 的 ending/fade
  帧 capture"**——与 §3.6/§3.7 矛盾；§3.2 主程序仍保留 v5 make（虽有注释说作废），且没有"组 D=v5
  已作废→见 §3.6"指针。
- ❌ **§3.3 的 ① HDR/OLD（490-492 行）仍只有 gameplay/stg03/stg08/l4-baseline-v5**，缺
  `tas-stg08b2-direct`（10 头行）/`stg06-timeline`；③ 的 checks（513-514 行）仍用
  `l4-baseline-v5_f30000_probe.png`/`f28500_probe.png` 作为 `l4_ending-1`/`l4_fade-clear` 的源图，
  **没有 §3.6 更正指针**（只在段末 527-528 行补了泛注）。这两处会让执行者继续按 v5 取图/取头行。
- ❌ **`stg06-timeline` 的 probe_mem 事实错误**：实读场景文件 **`probe_mem` 行数=0**
  （仅 `frames 30000`、`capture_every 300 probe` 与 poke/watch）；§2.1 I6/§2.4 表/I6b 说明/§3.7/
  §15.1 均写"源 8 probe"——实际源 **0 probe**，I6 源运行不会产出 `.probe.jsonl`；正确写法应是
  "I6 源 0 probe；I6b pass2 overlay 补 8（生成后头行 8）"。§2.4 EXP 脚本也把 `stg06-timeline` 期望
  设为 8，会直接误报；`phase-batch stg06-timeline` 也会因缺 probe 失败（需用 I6b）。
- **§15.1/task-50 同错**：`stg06-late ← stg06-timeline（源 8 probe）`；应改为源 0 + pass2 补 8。
- **§3.6 判据②的缺 mode 字段**未在影响面中修正（见上）。

### 4. findings（四分类）

#### 阻塞（P0）
- **F49-1（阻塞）**：§3.2（414/421 行）与 §3.3（490-492、513-514 行）**仍按旧 v5 取图/取头行**，
  且缺 §3.6 更正指针——执行者会继续用 v5 做组 D 的 blob 校验，直接违背 task-48 的目的。要求：
  删除/改写 §3.2 的 v5 生成与"v5 ending/fade"句；§3.3 HDR/OLD 改为 I5/I6 源（`tas-stg08b2-direct` 10、
  `stg06-timeline` 源 0/I6b pass2 8）；③ checks 中 ending/fade 改读 §3.6 选中来源的 shots；所有保留的
  v5 对照加"见 §3.6，v5 仅负候选"指针。
- **F49-2（阻塞）**：`stg06-timeline` **源 probe_mem=0**，手册多处写 8（§2.1 I6、§2.4 表/EXP、§3.7、
  §15.1）——事实错误且会导致 I6 源运行/phase-batch/probe 头行误判。改为"I6 源 0 probe（无 .probe.jsonl）；
  I6b pass2 overlay 补 8（生成后 8 头行）"，并同步所有表/脚本/§15.1。
- **F49-3（阻塞/重要）**：§3.6 判据②"候选 `frames.jsonl` 中 mode=Ending"**不可执行**：实际 frames.jsonl
  只有 `{"f","w"}`，无 mode/checkpoint；请改成可执行口径，例如 C1 的 `watch 0026 flags26` 按 `$26 bit7`
  判 Ending/fade（给出映射/命令），或生成对应 `.l5.jsonl`/checkpoint trace 后再判；否则三判据不完整。

#### 重要（P1）
- **F49-4**：C5 `s5-full-inject` 只有概念规划（无 DSL/帧表/命令），C1-C4 全失败时不可直接执行；
  建议给 scenario 骨架/各关注入帧表或明确"新卡再出"。
- **F49-5**：§3.6 判据①说"`capture_every` 截图"——C2/C3/C4 若需 overlay 加 probe，capture_every 仍在
  源中可覆盖目标帧；但 C5 新采集要显式 capture；建议在判据里注明"源 capture_every 或 overlay 显式 capture，
  标签需登记"。

#### 次要（P2）
- **F49-6**：当前终稿 1622 行/sha `16b76acc` 与 Lead 转述的 1555/`0878badd` 不同（多出 §3.7/§15.1
  task-50）；提交前请确认以哪一版为准，避免复核对象漂移。
- **F49-7**：§3.3 的 ③ 校验列表仍只有 13 项（缺 #6-10 中 ending-2..5、#14 等），建议按 §3.6 重裁后来源
  补齐或明确"由 §7.3/§7.4/§3.6 补"。

#### 建议（P3）
- **F49-8**：§3.6 候选表 C1 理由写"唯一"前建议附全场景 probe_mem 扫描命令（我已用场景文件扫到唯一 10，
  但手册未给命令）；C2 的 `stg07-clear=f28683` 来自 l5 的第 10 行（source_run=tas-stg07-relay），建议注明。
- **F49-9**：§15.1 属 task-50 新增，建议在 §3.7/§15.1 与 §3.6 之间加互链，避免读者把两组更正混为一条。

### 5. 独立命令摘要

```text
oracle/traces/tas-clear-full.l5.jsonl：28364 StageClear/stg01-clear；29078 Shop/stg02-shop；
  29478 Combat/stg02-combat；51332 StageClear/stg02-clear；...28683 StageClear/stg07-clear
  （source_run=tas-stg07-relay）——与 §3.6/§3.7 一致
tas-stg08b2-direct.txt：frames 40000 / probe_mem 10 / capture_every 1500 probe /
  poke 1265 0180 08 + poke 1255 01be/01b6/01b7 01；全场景唯一 probe_mem=10
tas-stg07-relay / tas-stg08-direct / tas-stg08b-direct / tas-stg05-relay：frames 120000、
  probe_mem 0、capture_every 1500、0180 poke 各对应 stage
stg06-timeline.txt：frames 30000 / probe_mem **0** / capture_every 300 probe / poke 2400...
frames.jsonl 实际格式（scenario_capture.lua）：{"f":N,"w":{label:value,...}}（无 mode/checkpoint）
docs-links：PASS 334 md/420 links；全量 check：PASS=11 FAIL=0 SKIP=1 TOLERATED=0
被审手册：1622 行 / sha16b76acc / mtime 14:31（l4-analyst inactive）
```

### 6. 结论

**needs_revision**。§3.6 的 A 级证据、C1-C5 候选事实、原假设留档、§15 历史勘误与多数影响面**成立**；
但 **F49-1（§3.2/§3.3 旧 v5 取图/头行未改 + 无更正指针）**、**F49-2（stg06-timeline probe_mem 源 0 写成 8）**、
**F49-3（mode 判据不可执行）** 是实跑会踩的硬问题，必须先修；F49-4/F49-5 建议同批补。
修后只需点验 F49-1/F49-2/F49-3 + §3.7 stg06 修正即可放行。

---

## 点验（2026-10-18，task-55）

> 对象：[l4-baseline-freeze-plan.md](l4-baseline-freeze-plan.md) 当前终稿 **1731 行 / sha256
> `817408df34bd…` / mtime 14:37，l4-analyst inactive 稳定**（与 Lead 转述一致）。本点验只核
> **F49-1..F49-6**，不重审已通过证据面；只读 + 本文件写入，未 git add/commit。

### 点验 VERDICT：**pass**（2 条非阻塞残留；F55-1 建议顺手补）

F49-1..F49-6 全部实质落地：§3.2 overlay 名单改为 pass2 双轨；§3.3 HDR/OLD/③ 改新轨；
stg06 源 probe=0 已全链修正；§3.6 判据②改为可执行的 `flags26 bit7`+fade 亮度；C5 骨架/明确"不作当前回退"；
capture 标签与版本口径已补；docs-links PASS。残留仅为 §3.3 HDR 中 pass2 文件时序与 §3.2 自检时序的
**执行顺序提示**（不影响修正确认，但建议提交前加一句）。

### 1. F49-1 ——✅ 已闭环

- §3.2（420-423 行）：overlay 名单已改 `l4-stg08b2-pass2`（源 `tas-stg08b2-direct` 10 probe→只加 capture）
  与 `l4-stg06-pass2`（源 `stg06-timeline` **0 probe→补 8**+加 capture）；旧 v5 明示"作废降级负候选"。
- §3.2（428-432 行）补全规则：源 0 补 8、源 10 补 0；已删除"capture_every 提供 v5 ending/fade"句并划线留档；
  主程序旧 v5 make 已注释，pass2 调用留 §3.7。
- §3.3 ①（501-504 行）HDR/OLD **已加** `tas-stg08b2-direct`:10、`l4-stg06-pass2`:8，OLD 新轨均 `None`；
  ③（525-527 行）已删除 `l4-baseline-v5_f30000_probe.png`/`f28500_probe.png` 两行，注释明确改 §3.6 命中来源；
- §3.1 #6-#11 行（370/392 等）已划线"来源待重裁（§3.6）"；全文旧 v5 取图路径已不再作为有效 checks（仅提及
  "作废/负候选"）。
- 自检脚本（479-487 行）对两个 pass2 期望 10/8 + 无重复，逻辑正确。

### 2. F49-2 ——✅ 已闭环

- 实测 `oracle/scenarios/stg06-timeline.txt`：`frames 30000`、`probe_mem` **0 行**、`capture_every 300 probe`、
  仅 poke/watch——确认源 **0 probe**。
- 全链已改：§2.1 I6 **0**/I6b **8**；§2.2 **phase-batch 删 stg06 行**（191 行注"源 0 不产 probe，只作
  expected capture 源"）；§2.3 run1/run2 的 probe/text 集合已去掉 stg06；§2.4 表 `stg06-timeline` **0**/
  `l4-stg06-pass2` **8**，EXP `stg06-timeline:0`；§2.6 probe/text 产物排除 stg06；§3.2（422/429）源 0 补 8；
  §3.7（1131 行）"源 0→I6b pass2 补 8"；§7.2（1144 行）同；§15.1（1728 行）同；D.3（1649 行）新轨计数
  `stg06-timeline 0（pass2 补 8）`。**无残留"源 8"错值**（grep 全部命中均为 0/补 8 语义）。

### 3. F49-3 ——✅ 已闭环

- §3.6 判据②（666-700 行）已改为可执行：明确 `frames.jsonl` 只有 `{"f","w"}`、无 mode 字段；
  改为读候选场景 `watch 0026 flags26`，`$26 bit7=0x80 ⇒ Ending`（附 python 命令，`<winner>` 占位）；
  fade 用 `tools/diff_visual.decode_png` 平均亮度（<~20/255，B 级辅助）；`mode_evidence` 字段已加入
  `l4-groupD-source.json`（706-707 行）。
- C1/C2/C3 场景均有 `watch 0026 flags26`（我 grep 确认 `tas-stg08b2-direct`/`tas-stg07-relay`/
  `tas-stg08-direct`/`tas-stg08b-direct` 均有；C4 `tas-stg05-relay` 也有该 watch）；C5 骨架含该 watch。
  criteria 现在**有可执行的命令路径** ✅。

### 4. F49-4 ——✅ 已闭环

- §3.6 C5（606-664 行）：给出 `s5-full-inject` 骨架（frames 40000、前 6 关注入帧、8 probe、capture_every/
  watch），并明确 **stage-8 注入/ending 观测窗 TODO（B4 卡）**、**"当前仅半可执行、不作 P0-0 当前回退"**；
  未发明 stage8 帧号。符合"规划"要求且不冒充可执行。

### 5. F49-5/F49-6 ——✅ 已闭环

- **F49-5**：§3.3（537-539 行）+ §3.6（668-670 行）要求 capture 出处标签（源 `capture_every N tag` 或 overlay
  `capture <f> l4f<f>`），manifest `capture_tag` 与文件名 tag 一致；C5 显式 capture 已标注。
- **F49-6**：头部（10-14 行）改为"连续点改 task-30/32/34/46/48/50/54；行数/sha 以提交时实体为准，不硬编码
  中间版本"；不再自指旧行数/sha。终稿 1731/sha `817408df` 与实际一致。
- **docs-links PASS 334 md/422 links**（实跑 `gates/cli.py check`；SUMMARY PASS=11 FAIL=0 SKIP=1）。

### 6. 非阻塞残留（建议提交前顺手补）

- **F55-1（中低）**：§3.3 ① HDR/OLD（501-504 行）包含 `l4-stg06-pass2`，但该 pass2 的 `.probe.jsonl`
  要到 **§7.2/§3.7 之后**才生成；而 §0.5 顺序是 §3 先于 §7a/§7.2。若按顺序执行 §3.3 ①，会
  `FileNotFoundError`。建议：该行加"（§7.2 后补跑）"或脚本对 pass2 缺失 `continue`/显式 skip；
  `tas-stg08b2-direct` 源文件在 §2 已存在，无此问题。
- **F55-2（低）**：§3.2 自检（477-487 行）对两个 pass2 overlay 做 grep，但 pass2 生成在 §3.7；建议注明
  "本自检在 §3.7 make 之后执行"。两者不影响 F49 修正确认。
- **F55-3（低）**：§3.6 判据②的 python 片段用 `<winner>` 占位，建议在 §3.7 选中 run 后把实际文件名填入
  示例（当前可执行但需人工替换）。

### 7. 命令输出摘要

```text
stg06-timeline.txt：probe_mem 0 / frames 30000 / capture_every 300 probe（F49-2 事实成立）
flags26 watch：tas-stg08b2-direct / tas-stg07-relay / tas-stg08-direct / tas-stg08b-direct / tas-stg05-relay
 均含 watch 0026 flags26；C5 骨架含之
manual：1731 行 / sha817408df34bd / mtime 14:37（稳定）
docs-links：PASS 334 md/422 links；全量 check PASS=11 FAIL=0 SKIP=1 TOLERATED=0
§3.2/§3.3 旧 v5 取图行：仅剩"作废/负候选/§3.6 指针"语义（有效 checks 已删）
```

### 8. 结论

**pass**：F49-1..F49-6 全部实质闭环，证据/候选/判据/影响面与 docs-links 均核实通过；
F55-1（§3.3 pass2 时序）/F55-2/F55-3 为非阻塞执行提示，建议提交前顺手补，不阻塞 P0-0 启动。

---

# 复审（2026-10-18，task-85）—— P0-0 手册环境章节（Windows headless 主路径）

> 复核方：`shell-verifier`（task-85，verifier）；被审：task-81 / `l4-analyst`
> 被审版：`docs/findings/l4-baseline-freeze-plan.md` **1855 行**，SHA256 前 16=`d166b40dcb44307e`（与 task-85 通告一致；未提交，`git diff` = +185/−83）
> 范围：§1 E1-E11 + §1.2 冒烟/两跑确定性 + §1.3 版本记录 + §1.4/Linux-WSL 备选 + §16 更正节（只核环境章节与实测一致性；其余章节内容不在本卡范围）。
> 运行环境：Windows 10.0.26100（Win11 x64）、Python 3.12.1、cargo 1.96.0-nightly；串行锁下与 task-83/84 错开跑（复核期间确有并发任务运行同一工具，见 F-6）。
> **VERDICT：needs_revision** —— E1-E10/实测数字/版本记录/Linux-WSL 备选表述**逐项核实通过**；但 **E11 有一处事实错误**（"P0-0 只构建 game-desktop/game-oracle（不构建 winit/softbuffer）"不成立：`game-desktop` 直接依赖 `winit`/`softbuffer`，构建该包必然编译它们）。另 2 条 Low/Info 建议。E11 改一处措辞即可转 **pass**。

## 复审 §A：E1-E11 逐项核

| # | 手册断言 | 我的独立核 | 结果 |
|---|---|---|---|
| E1 | Windows 10/11 x64 | `[Environment]::OSVersion.VersionString` = `Microsoft Windows NT 10.0.26100.0`；`Is64BitOperatingSystem=True` | ✅ |
| E2 | `tools/mesen2/Mesen.exe`（Win 2.1.1 x64，gitignored 可再生） | 文件 70,470,144 B；SHA256=`d2eb03c2590c648bf329f127ebcfefd70130e7690a9e2ccdba8616faea1fe96b`（与 §1.3/§16.1 一致）；FileVersion **2.1.1**、ProductVersion `2.1.1+137ae7ce…`；`git check-ignore` 命中 `.gitignore:19 tools/mesen2/`、`:20 tools/mesen2.zip` | ✅ |
| E3 | `%USERPROFILE%\Documents\Mesen2\settings.json` → `Debug.ScriptWindow.AllowIoOsAccess=true`；顶层 `ScriptWindowConfig.AllowIoOsAccess` 实测不生效；改后重启 | 路径存在（166,391 B，mtime 2026-10-18 16:18:18）；JSON 可解析；**当前** `Debug.ScriptWindow.AllowIoOsAccess=True`、顶层 `ScriptWindowConfig.AllowIoOsAccess` 也存在（True）。**旁证 E3 有效性**：复核期间首个快照读到 `Debug=False / top=True`（并发任务 A/B 负例中），对应窗口的 `title-menu` 运行产物（`analysis/traces/.archive/title-menu-20261018-162952/title-menu.run.json`）恰为 **frames=1080 / trace_lines=0 / done=null / shots=0 / wall=120.8s**（典型"io 被拒、无采集件"症状）；随后恢复 Debug=True 后我两跑即正常 ⇒ 与手册表述一致。**注意：该 A/B 由并发任务执行，我未翻改用户级 settings（避免与 task-83 互相干扰）；手册"顶层不生效"缺证据指针（见 F-2）** | ✅（结构/值/症状一致；证据指针待补） |
| E3b | 无需 Xvfb/显示器（headless `--testRunner`；`scenario_run.py` 在 Windows 自动选 .exe） | `python tools/scenario_run.py list` exit 0（打印场景清单、无窗口）；我两跑 headless 均无 Xvfb 进程、无 GUI；进程命令行实测 `Mesen.exe --testRunner analysis\_gen\title-menu.lua rom\gunnac_cn.nes` | ✅ |
| E4 | `python` ≥3.10（本机 3.12.1；`python3` Store 别名不可用） | `python -V`=3.12.1 ✓；`python3 --version` → "Python was not found; run without arguments to install from the Microsoft Store"（确认别名不可用） | ✅ |
| E5 | 能构建 workspace（render-frame/phase-batch） | `cargo build -q -p game-desktop -p game-oracle` → **exit 0**（本机已有缓存；无缺失 DLL 报错） | ✅ |
| E6 | HEAD ≥`0cf42f0`；工作树干净 | HEAD=`51da8e5`；`git merge-base --is-ancestor 0cf42f0 HEAD` exit 0 ✓。工作树当前有 task-81 等 WIP 改动（正常，提交后应干净）——**时点说明**，非缺陷 | ✅ |
| E7 | `rom/gunnac_cn.nes` sha 与基线一致 | `python gates/cli.py rom` → `[PASS] rom-integrity` 4 锚点 sha 一致，rom=`4d76901c1b2455f2…` | ✅ |
| E8 | 磁盘 ≥20GB 空闲 | D: Free = **53,994,188,800 B ≈ 50.3 GiB** ≥20 ✓ | ✅ |
| E9 | 内存 ≥16GB | `Get-CimInstance Win32_ComputerSystem`.TotalPhysicalMemory/1GB = **15.9 GiB**（=17.1 GB 十进制；Windows 标称 16GB）。按二进制 GiB 比较与"≥16GB"差 0.1 GiB/按十进制达标——**单位口径建议写明**（F-4，非阻塞） | ⚠️ 口径 |
| E10 | 同一时刻只跑一个 Mesen（`analysis/.scenario.lock`；Windows=msvcrt 1-byte / POSIX=flock） | 代码核对：Windows 分支 `import msvcrt; seek(0); locking(LK_NBLCK,1)`，POSIX 分支 fcntl.flock；实际行为：我第二次 `run` 在并发任务持锁时被拒，输出 `另一场景运行中（串行纪律）：analysis/.scenario.lock 被占用`（exit 2）⇒ 锁真实生效 | ✅ |
| E11 | Windows 主路径无 X11/Xvfb 依赖；**P0-0 只构建 game-desktop/game-oracle（不构建 winit/softbuffer）**；`cargo build --workspace` 非必需 | **前半句成立**（Windows 无 Xvfb/X11；实测无 Xvfb 进程、headless 跑通），**但"不构建 winit/softbuffer"不成立**：`crates/game-desktop/Cargo.toml` 直接依赖 `winit="0.30"`、`softbuffer="0.4"`；`cargo tree -p game-desktop -i winit` / `-i softbuffer` 均显示 `game-desktop v0.1.0` 直接依赖 ⇒ `cargo build -p game-desktop` **必然编译 winit/softbuffer**（无论哪个 bin；依赖是包级）。Windows 后端不需要 X11 库，但 POSIX 备选构建 game-desktop 时才需 X11（手册 §1.4/附录 S3 方向正确，但 E11 行与之自相矛盾） | ❌ **事实错误（必改）** |

## 复审 §B：冒烟与两跑确定性（自跑）

**我自己的两跑**（串行锁下；`python tools/scenario_run.py run title-menu --timeout 600`）：
- runA：`frames: 1080 trace_lines: 1080 captures: 37 shots: 37 wall: 5.5s` + `DONE: {"frames":1080,"trace_lines":1080,"captures":37,"shots":37,"bot_stop":false}`；run.json `timed_out=false`；exit 0。
- runB：同为 **1080/1080/37/37**（wall 4.0s），trace sha 与 runA **完全相同**。
- **trace sha256 = `62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331`**（589,653 B）= 手册 §1.2/§16.2 记录值 ✓。
- **shots**：runA 37 张；runB 37 张 + 1 张**外部注入**的 `title-menu_f99999_fake.png`（4 B，16:30:44 由并发任务写入，见 F-6）；排除该注入后 **37/37 逐字节相同**；runA 37 张 vs 归档 `title-menu-20261018-161941` 的 37 张 **37/37 逐字节相同**。
- 归档旁证：`analysis/traces/.archive/` 中 **12 个** `title-menu.frames.jsonl` 全部 sha=`62c9ea09…`（跨多次运行天然确定性）。
- 手册"wall 3.5s"：我实测 4.0–5.5s（含启动/加载波动）；`run.json.wall_s` 本就随机器状态变化，文档已给判据不看 wall，属正常波动（Info）。
- **门禁**：`python gates/cli.py status` → **PASS=11 / FAIL=0 / SKIP=1（addr-bridge）/ TOLERATED=0、STATUS: OK**；`docs-links` PASS（347 md/438 links）、`knowledge-format` PASS（24 条）；`python gates/cli.py rom` → `[PASS] rom-integrity` ✓ —— 与 §1.2/§16.2 数字一致。

## 复审 §C：版本记录 / Linux-WSL 备选

- **版本记录**（§1.3/§16.1）：Mesen.exe sha `d2eb03c2…` ✅、70,470,144 B ✅、FileVersion 2.1.1 ✅；Python 3.12.1 ✅；cargo 1.96.0-nightly (cbb9bb8bd 2026-03-13) ✅ —— 四项全部与实测一致。
- **Linux/WSL 表述**：`std::bad_cast` 均限定为"**WSL2 内 Linux 版 Mesen2 2.1.1 实测**（任何 invocation，exit 134）"，并明确"不采用/仅备选/换可用构建才谈 Linux 路径"（§1 注 :80、§1.4 :150、§16.1 :1821）——**未写成通用结论** ✓。
- **POSIX 路径保留**：§1.4 :148-149 保留自管 Xvfb + `DISPLAY` + flock + `pgrep` 孤儿清扫 + `/DoNotSaveSettings`；附录 C :1651 亦标注"`xvfb-run` 仅 POSIX 历史文案"——未删除备选 ✓。
- **§16 落点**：`## 16` 存在（:1808-1855），16.3 影响面（§1/§0.5/§2/§8.2/§9.2/§10.1/§11/附录 S3/附录 C）与 `git diff` 实际改动逐条对得上 ✓（diff 另含全文 `python3`→`python` 命令替换，属同一环境口径，Info）。

## 复审 §D：findings（四分类）

### 实现/工具（B）——无（本卡只审手册；`tools/scenario_run.py` 的 Windows 分支/锁已在代码层核对并实跑）

### 证据/文档错误（E）

- **F-1（必改，Med）E11 事实错误**：见 §A E11。建议改为："Windows 主路径构建 `game-desktop`/`game-oracle` 时会编译 `game-desktop` 的 **winit/softbuffer 依赖**（Windows 后端无 X11/Xvfb 需求；POSIX 备选构建才需 X11 库）；P0-0 不需要构建 `shell-win` 等 workspace 其它包"；若真要"不构建 winit/softbuffer"，需把 `render-frame` 拆独立 crate（后续卡，不在本卡）。
- **F-2（Low）E3 "顶层不生效"缺证据指针**：§1.1 E3 与 §16.1 均标"实测不生效"，但未给 A/B 记录/源码指针。建议交叉引用 `docs/knowledge/gotcha-mesen2-headless-windows-io.md`（task-82，含 Mesen 源码 `LuaApi.cpp:984`/`DebugConfig.cs:104` 与 A/B 判据）或 §16.2 增一行 A/B 记录（Debug=false ⇒ frames 1080/trace 0/done null）。
- **F-3（Low）`analysis/p0-0-env.txt` 未生成**：§1.3 给出生成命令、§8.2 验收清单要求"记录进 `analysis/p0-0-env.txt`"，当前工作区 `Test-Path=False`（环境已宣称 P0-0-0 通过）。建议执行 P0-0-0 时生成该记录（gitignored 工作区件）并在 §8.2 勾选；否则"环境接收已完成"只有文档数字、无本机留档。

### 规格缺口（S）

- **F-4（Info）E9 内存单位口径**：15.9 GiB（=17.1 GB 十进制）vs "≥16GB"；建议写"≥16GB（Windows 标称；GiB 实测约 15.9）"或统一十进制。
- **F-5（Info）E11 与 §1.4/附录 S3 的自洽**：修 F-1 后同步检查 §1.4 末行"POSIX 备选才需 X11 库"与 E11 行不冲突（当前矛盾点在 E11 的前半句）。

### 平台差异（P）

- **F-6（环境/并发，非缺陷）**：复核期间 task-83/84 并发运行同一工具：①我的第二次 `run` 被串行锁拒绝（证明 E10 有效）；②并发任务在 16:30:44 向 `shots/` 注入 `title-menu_f99999_fake.png`，导致我 runB 的 `outputs.shots` 计数=38（DONE/手册判据仍 37）。串行锁只保护"场景驱动"，不阻止外部手工/实验文件写入——验收跑应与其它 agent/GUI 手工实验错开，且 artifacts 以 manifest/DONE 判据而非裸 glob（手册 §7.5 F2 已有同类纪律，可复用）。我的定稿结论以 runA + 归档 37/37 为准。

## 复审 §E：命令与输出摘要（自跑）

| # | 命令 | 摘要 |
|---|---|---|
| 1 | `(Get-FileHash tools/mesen2/Mesen.exe)` / VersionInfo | sha `d2eb03c2…`、70,470,144 B、FileVersion 2.1.1 |
| 2 | `python -V` / `cargo -V` / OS | 3.12.1 / 1.96.0-nightly (cbb9bb8bd 2026-03-13) / Windows NT 10.0.26100.0 x64 |
| 3 | settings.json 解析 | `Debug.ScriptWindow.AllowIoOsAccess=True`；顶层 `ScriptWindowConfig.AllowIoOsAccess` 亦存在（True）；路径/大小/时间已记录 |
| 4 | `python tools/scenario_run.py run title-menu --timeout 600` ×2（串行锁下） | runA 1080/1080/37/37 wall 5.5s、runB 同、trace sha `62c9ea09…` 两跑相同；shots 37/37 逐字节同（排除 F-6 注入件）；`python tools/scenario_run.py list` exit 0 |
| 5 | 归档比对 | `.archive/` 12 条 `title-menu.frames.jsonl` sha 全同 `62c9ea09…`；runA vs `title-menu-20261018-161941` shots **37/37** 同 |
| 6 | `python gates/cli.py status` / `rom` | PASS=11 FAIL=0 SKIP=1 STATUS OK；docs-links PASS（347/438）、knowledge-format PASS（24）；rom-integrity PASS（4 锚点） |
| 7 | `cargo build -q -p game-desktop -p game-oracle` | exit 0；`cargo tree -p game-desktop -i winit/softbuffer` 显示 **game-desktop 直接依赖**（E11 反证） |
| 8 | `git merge-base --is-ancestor 0cf42f0 HEAD` / `git check-ignore` | exit 0；`tools/mesen2/`、`tools/mesen2.zip` 均 ignored |
| 9 | E7 磁盘/内存 | D: free 53.99GB；RAM 15.9 GiB（E9 单位口径见 F-4） |
| 10 | 并发锁实测 | 并发任务运行时我的 run 被 `analysis/.scenario.lock` 拒绝（exit 2，"另一场景运行中"）；记录 F-6 |

## 复审结论

**needs_revision（F-1 必改；F-2/F-3 建议同批；F-4/F-5/F-6 为 Info）**：
- **E1-E10 全部核实通过**：Windows x64/Mesen.exe sha+版本/headless `--testRunner`/E3 IO 症状旁证/E4 python/串行锁/E7 rom/E8 磁盘/门禁数字/两跑 trace sha+37/37 shots 逐字节全对；版本记录四项一致；Linux-WSL `bad_cast` 限定与 POSIX/Xvfb 备选保留均如实。
- **唯一阻塞项=E11 的"不构建 winit/softbuffer"事实错误**（`game-desktop` 直接依赖 `winit 0.30`/`softbuffer 0.4`，构建必编译；Windows 后端无 X11 需求这半句成立）。改为"会编译 game-desktop 的 winit/softbuffer 依赖；Windows 后端无需 X11/Xvfb；POSIX 备选构建才需 X11 库"后即可 **pass**，无需重跑环境（我两跑+归档+门禁日志在 `target/review-l4env/` 可复算）。
- 附加建议：F-2 补 E3 证据指针、F-3 生成 `analysis/p0-0-env.txt`；F-4 统一内存单位；并发验收与 GUI 手工实验错开（F-6）。

> 本复审只追加本节到本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。

---

# 点验（2026-10-18，task-88）—— F-1..F-4 修正点验

> 被审版：`docs/findings/l4-baseline-freeze-plan.md` **1866 行**，SHA256 前 16=`fbf10a1a07bd4981`（task-87 改后；未提交）。
> 范围：只核 task-85 的 **F-1（必核）+ F-2/F-3/F-4 抽查**；其余环境项/两跑/门禁 task-85 已验证，不再重跑。
> **VERDICT：pass** —— F-1 事实更正**正确且四处同步一致**、`cargo tree` 依据可复现；F-2 证据指针可解析；F-3/F-4 落点到位；docs-links PASS。仅 1 条 Info（唯一保留的"不构建 winit/softbuffer"短语为条件句反例，非事实断言）。

## 点验 §A：F-1（E11 依赖事实）——✅

| 检查 | 结果 |
|---|---|
| §1.1 E11 行（L99） | 已改为："**Windows 主路径无 X11/Xvfb 依赖**（Mesen2 x64 自带 runtime）。**但构建 game-desktop/game-oracle 时会编译 game-desktop 的 winit/softbuffer 依赖**（Cargo.toml 直接依赖 `winit="0.30"`/`softbuffer="0.4"`，包级依赖与 bin 无关 ⇒ `cargo build -p game-desktop` 必编译）；Windows 后端无需 X11，**POSIX 备选构建才需 X11 库**；P0-0 不需要构建 shell-win 等其它包；若真要'不构建'需拆 render-frame 独立 crate（后续卡）"——与要求的四要素**逐项命中** |
| 依据（自己跑） | `cargo tree -p game-desktop -i winit` → `winit v0.30.13 └── game-desktop v0.1.0`；`-i softbuffer` → `softbuffer v0.4.8 └── game-desktop v0.1.0`（**直接依赖**，可复现）——E11 的"必编译"结论成立 |
| §1.4（L150/L157） | L150 仍写 headless `--testRunner` 无需显示/Xvfb；L157 "（winit/softbuffer 的 X11 后端；Windows 主路径不需要，见 E11）"——与 E11 一致 |
| 附录 S3（L1760） | 已改："Windows 主路径无 Xvfb/X11；P0-0 构建 game-desktop/game-oracle（**会编译 game-desktop 的 winit/softbuffer 依赖**；task-87 F-1）；POSIX 备选构建才需 X11 库" ✓ |
| §16.3（L1846-1847） | "§1 … E11=无 X11/Xvfb 依赖〔**构建 game-desktop 仍会编译 winit/softbuffer 依赖；POSIX 备选构建才需 X11 库**，见 §1.1 E11〕" ✓ |
| §16 尾补记（L1855-1857） | 已登记 task-87 的 F-1..F-4 修正记录 ✓ |
| 残留检查 | 全文件仅 1 处"不构建 winit/softbuffer"字面（L99 内 "**若要真正'不构建 winit/softbuffer'需把 render-frame 拆独立 crate**"）——是**条件句/反例**，非"P0-0 不构建"的事实断言 ⇒ 可接受（Info-1：若要求 grep=0，可改为"若要避免该依赖…"） |

## 点验 §B：F-2/F-3/F-4 抽查——✅

- **F-2（E3 证据指针）**：§1.1 E3（L90）与 §16.1（L1827）均新增 `[gotcha-mesen2-headless-windows-io](../knowledge/gotcha-mesen2-headless-windows-io.md)` 指针；目标文件存在（`docs/knowledge/gotcha-mesen2-headless-windows-io.md`，74 行）；`docs-links` **PASS（348 md/440 links）** ⇒ 链接可解析 ✓。
- **F-3（env 记录时点）**：§1.3（L142）新增"`analysis/p0-0-env.txt` 说明（task-85 F-3）：该文件是 **P0-0-0 执行时生成**的 gitignored 工作区记录…"；§8.2（L1497）同步"（记录进 `analysis/p0-0-env.txt`——**该文件由 P0-0-0 执行时生成**、gitignored，内容清单见 §1.3）"⇒ 明确"执行前不存在≠环境未接收" ✓。
- **F-4（E9 内存口径）**：§1.1 E9（L97）已改"**≥16GB（Windows 标称；实测 15.9 GiB ≈ 17.1 GB 十进制）**"、"记录 **GiB/GB 双口径**；十进制 ≥16GB 达标" ✓。

## 点验 §C：其它

- **docs-links**：`gates status` → docs-links **PASS**（348 md/440）、knowledge-format PASS（24）、SUMMARY PASS=11/FAIL=0/SKIP=1、STATUS OK ✓。
- **写范围**：task-87 只改 `docs/findings/l4-baseline-freeze-plan.md`（工作树 M）；本点验只追加本文件 ✓。
- **时点**：手册现行 1866 行 / sha16 `fbf10a1a…`；提交以该实体为准。

**剩余问题：无阻塞**（Info-1：L99 条件句短语可改措辞以便 grep=0；不影响语义与提交）。
**点验结论：pass** —— F-1 四处同步修正且依据可复现，F-2/F-3/F-4 落点完整；手册+知识条目+工具复核报告可提交。

> 本点验只追加本节到本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。





