# 对抗复核：P0-0-1 采集汇总 / I5 BLOCKED 证据（task-86）

> 日期：2026-10-18｜复核方：platform-verifier（verifier 角色，证伪立场）｜被审：l4-analyst/test-runner（task-86）
> 被审：`docs/findings/p0-0-capture-run1.md`（152 行，untracked）+ `analysis/repro/p0-0-capture/`
> （ledger/summary/incident，gitignored）+ `analysis/p0-0-env.txt`。
> 复核方式：**直接从 gitignored 产物 re-hash / 逐行数 / 读 incident JSON/env**；未跑 gates check/status/visual，
> 未抢 Mesen 串行锁重跑；只读 docs-links 单门。本文件唯一写入，未 git add/commit。

## 0. VERDICT

**pass**（3 条低优先级数字/文字修正；证据面无编造）

**核心数字全部独立复算成立**：
- I4 `stg08-timeline`：34000/34000、125/125、wall 213.8s、trace sha `5f0fb228…f34fd` ✅
- I3 `stg03-timeline`：24000/24000、83/83、194.4s、`e9086b8e…94505` ✅
- I6 `stg06-timeline`：30000/30000、108/108、126.3s、`e7798edb…4f473` ✅
- I1 title-menu：1080/1080、37/37、两跑 trace `62c9ea09…a331`、shots 37/37 逐字节相同 ✅
- I2 gameplay-stage1：2400/2400、44/44、两跑 trace `94ec3a07…d9c8b`、shots 44/44 逐字节相同 ✅
- 判据 `done!=null ∧ trace_lines==frames ∧ ¬timed_out` 逐场景成立；ledger 18 行（3 个显式失败/阻塞状态）；
  env 73 行（含 OS/CPU/RAM/disk/toolchain/Mesen sha/settings IO/gates rom/status 摘要）完整。
- I5 `BLOCKED: capture-termination` 证据**存在且如实**：R2/R3 归档 `frames.jsonl` 均 **39998/40000 行**
  （40,014,329B）、`.done` 不存在；incident JSON 记录 probe 188.7MB、events/inputs 持续增长、Mesen CPU 等，
  报告未把 I5 写成 PASS，也未掩盖失败。
- docs-links 只读门 **PASS 356 md/447 links**；`git status -- oracle/screens` **0 条**。
- 报告把 I5 fallback 明确为**候选 C2/C3**（未声称已证），未发现超出证据的结论。

## 1. 数字复算（独立 hash/行数）

| 场景 | run.json 判据 | trace sha256 实测 | 与报告 |
|---|---|---|---|
| I4 stg08-timeline | frames=34000 trace_lines=34000 done!=null timed_out=false wall=213.8 | `5f0fb2289cae0d8d7ba8b485db648ba3118868a56f27474f0fbcf023491f34fd` | ✅ |
| I3 stg03-timeline | 24000/24000, done, false, 194.4 | `e9086b8ef2ad945820a2bf3ec821d4009b4675ed9f17a1170f10b67640294505` | ✅ |
| I6 stg06-timeline | 30000/30000, done, false, 126.3 | `e7798edbc12afa1f497a221f5818be173de799790f6af00c9aba3913e764f473` | ✅ |
| I1 title-menu | 1080/1080, done, false, 3.5 | `62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331` | ✅ |
| I2 gameplay-stage1 | 2400/2400, done, false, 98.6 | `94ec3a07a722228e16e5c9470a2c3f83ceb7a3b65ac5e7e664ebcf9c584d9c8b` | ✅ |

**两跑确定性**：
- I1 归档配对 `title-menu-20261018-173652` / `...-173656`：各 37 个 shot，sha 集合相同（37/37）；
  多组历史归档同为 37 个相同 sha（含旧 fake 污染 38 张的归档已隔离）。
- I2 `gameplay-stage1-20261018-173053` / `...-173307`：各 44 shot，sha 集合相同（44/44）；
  两跑 trace 均 `94ec3a07…`。⇒ 报告 §3 PASS 成立 ✅。
- 报告中的 I1/I2 wall（3.5/3.5、133.5/98.6）与 ledger run.json 一致；I2 wall 差异由负载解释，合理。

## 2. ledger / env

- `ledger.jsonl` **18 行**（name/attempts/rc/driver_wall_s/trace_sha256/done/run_json/output_tail 等）；
  其中显式 `failed:stalled`、`failed:over-budget`、`blocked:capture-termination` 3 条；早期 R1 截断若干条
  `done=None` 但 `status=None`（靠 run.json/output_tail 证明未完成）——未伪造成功。
- `analysis/p0-0-env.txt`：**73 行（66 非空）**，含 OS/CPU/RAM/disk、Python/cargo/rustc、Mesen size/sha/version、
  settings `Debug.ScriptWindow.AllowIoOsAccess=True`/legacy 值、gates rom/status 摘要、串行锁、I5 结果；
  报告 §1 的 OS/CPU/Mesen sha/version/settings 与文件一致 ✅。
- **F107-1（低）**：报告 §1 写 env "63 行"，实测 **73 行（66 非空）**；内容完整但行数口径过期，建议改
  "73 行（66 非空）"或去掉行数。

## 3. I5 BLOCKED 证据

- **R2/R3 归档独立复核**：`tas-stg08b2-direct-20261018-170201` 与 `...-173713` 的
  `frames.jsonl` 均 **39998 行 / 40,014,329B**，`.done` **不存在**；incident JSON 记录
  `frames_lines_written:39998`, `done_exists:false`, `probe_jsonl_bytes:188,704,783`,
  `events_jsonl_bytes:9,968,167`, `inputs_jsonl_bytes:16,921,241`, `mesen_cpu_seconds_at_kill:1239.4`。
- `i5-retry.json`：`status=blocked:capture-termination`, `frames_lines=39998`, `done=false`,
  `reproduced_in=["R2","R3"]`——与报告 §4.3 一致 ✅。
- **F107-2（低）**：报告 §4.1 写 R1 "tas trace=**10984**/40000"；ledger 实际有**两次** R1 截断尝试：
  16:38 首跑 **11069**/40000（row0），16:42 次跑 **10984**/40000（row2）。两数都真，但"首轮=10984"
  与首跑 11069 不一致（task 卡亦引用 11069）；建议写成"R1 两次截断：11069、10984"。
- **F107-3（低）**：报告 §4.2 写 "17:02:01 启动，17:26 kill"，incident JSON 的
  `killed_at=17:23:46`；kill 时间口径不一致（约 2-3min）。建议以 incident 的 17:23:46 为准或注明
  "记录 17:26，incident 17:23:46"。
- 报告把 I5 明确标 `BLOCKED: capture-termination`、未声称 PASS、给出 R2/R3 定点/根因假设/修复建议，
  **不掩盖** ✅。

## 4. 取舍/未证边界

- fallback 写 **候选 C2 `tas-stg07-relay` / C3 `tas-stg08-direct`**（Lead 确认方向），并说明
  `stg07-clear f28683` 等前提；未把候选写成已选/已证，组 D/四联最终命中仍待 C2/C3 采集复核 ✅。
- I5 根因仅是**假设**（`capture.lua` `frame==frames` 未命中），建议 `frame>=frames`/独立计数；
  未写成已定谳，符合 B/C 级边界 ✅。
- 报告未声称 L4 基线完成/P0-0 全绿；I5 阻塞独立列出 ✅。

## 5. 门禁/纪律

- **未跑** `gates check/status/visual`（采集窗口纪律）；仅只读 `gate_docs_links()`：
  **PASS 356 md/447 links**。
- `git status -- oracle/screens` **0 条**；未改 tracked oracle/screens；未抢 Mesen 锁、未重跑采集。
- 报告只描述 `analysis/`（gitignored）与 docs；ledger/summary/incident 均 gitignored；本复核只写本文件。
- 写范围：本文件新增，未改报告/ledger/env/代码。

## 6. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- **F107-1（低）**：env 行数 63→实际 73（66 非空）。
- **F107-2（低）**：R1 两次截断 11069/10984，报告只写 10984 且称"首轮"；建议两数并列。
- **F107-3（低）**：R2 kill 时间 17:26 vs incident 17:23:46，统一口径。
- **F107-4（低/建议）**：ledger 早期 R1 截断行 `status=None`（靠 done=None/output_tail 识别）；
  建议给这类行显式 `status=failed:truncated-100s`，机器台账更可审计（不改结论）。

### 建议（P3）
- **F107-5**：I1/I2 两跑比较在报告中只写"PASS/全等"，建议附 run1/run2 归档目录名与每张 shot 的
  sha 清单文件（gitignored 也可生成），便于后续 rehash。
- **F107-6**：C2/C3 fallback 在 mapping 中写清"候选、未证"，与 §3.6/§3.7 的 conditional 语义一致。

## 7. 命令/证据摘要

```text
trace sha 实测（现 analysis/traces）:
  stg08-timeline 5f0fb228…f34fd；stg03-timeline e9086b8e…94505；stg06-timeline e7798edb…4f473
  title-menu 62c9ea09…a331；gameplay-stage1 94ec3a07…d9c8b
run.json: 5 场景 done!=null ∧ trace_lines==frames ∧ ¬timed_out（见 §1）
I2/I1 归档 shots：44/44、37/37 sha 集合相同（173053/173307；173652/173656）
I5 R2/R3: frames.jsonl 39998 行/40014329B、done 不存在；i5-retry status=blocked:capture-termination
ledger: 18 行；failed:stalled/over-budget/blocked:capture-termination 各 1
env: 73 行（66 非空）
docs-links: PASS 356 md/447 links（只读单门；未跑 gates check/status/visual）
oracle: git status -- oracle/screens = 0
```

## 8. 结论

**pass**。I4/I3/I2/I6/I1 数字/判据/两跑确定性和 I5 BLOCKED 证据均独立复算成立；ledger/env 完整、
未发现夸大或掩盖。F107-1..4 为低优先级数字/台账文字修正，建议提交前顺手改（不阻塞）。
