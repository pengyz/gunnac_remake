# P0-0 组 D 来源重裁：C3 `tas-stg08-direct` / C2 `tas-stg07-relay` 裁定（task-106）

> 日期：2026-10-18｜角色：l4-analyst（取证/记录）｜状态：**组 D 结论 = `blocked:no-source`（C1 BLOCKED；C2/C3 采集合法但 blob 0/6、无 Ending 语义）**
> 任务：task-106（P0-0 手册 §3.6 组 D 重裁）；环境=task-81/85 的 Windows Mesen2 2.1.1 headless（`tools/mesen2/Mesen.exe`
> sha256 `d2eb03c2…fe96b`）；采集工具 = `tools/scenario_run.py` + capture.lua（task-105 修复后：`frame>=frames` +
> overshoot/finalize_by + `frame_errors`；C3 用首波修复，C2 用含 pcall 韧性的最终版）。
> 纪律：串行锁、未改代码/gates/基线；`analysis/`、`shots/` gitignored 不入库；**未命中如实写、不硬凑**。

## 0. 结论（先读）

| 候选 | 采集状态 | blob（旧 expected） | mode/Ending 语义 | trigger | 组 D 可用性 |
|---|---|---|---|---|---|
| **C1** `tas-stg08b2-direct`（I5） | ✅ **修复后重跑合法**（40000/40000，`frame_errors=0`，见 §2.4）；旧 lua 时 BLOCKED: capture-termination | **0/6 命中**（全 26 张 shots 也无命中） | ❌ 同 C2/C3：目标帧 stage1/2、`flags26`=0x00/0x80 | injected/assisted（`0180`+assist） | **不可用（blob）** |
| **C3** `tas-stg08-direct`（120000f） | ✅ DONE 合法（旧 lua：`frames_actual=120001/overshoot=true/finalize_by=inputPolled-overshoot`；**修复后重跑** 120000/120000、`frame_errors=0`，§2.4） | **0/6 命中**（目标帧全 miss；全 80 张 shots 也无命中） | ❌ 目标帧 `stage_no`=1/2、`flags26`=0x00/0x80（bit7 在 stage2 战斗段出现，非 Ending） | injected/assisted（`0180=08`+3 assist poke+bot） | **不可用** |
| **C2** `tas-stg07-relay`（120000f） | ✅ **修复后重跑合法**（120000/120000、`frame_errors=0`，§2.4；旧跑为 DEGRADED `frame_errors=142149`） | **0/6 命中**（且目标帧 shots 与 C3 **逐字节相同**） | ❌ 同 C3（`flags26`=0x00/0x80、`stage_no`=1/2） | injected/assisted（`0180=07`+3 assist poke+bot） | **不可用（blob）** |
| C4 `tas-stg05-relay` / C5 `s5-full-inject` | 未采（C5 仅骨架，无 DSL 落地） | — | — | — | 未评估 |

**裁定：组 D（fade/ending 6 对 #6-#11）在 C1/C2/C3（含修复后重跑，§2.4）均无来源 ⇒ `blocked:no-source`。**
- 6 对 expected **不得 staging/替换**；P0-0 **不得宣称全绿**（manifest 对应 6 行 `blocked:no-source`，§8.2 对 6 行不通过）。
- **不硬凑**：C3/C2 的 shots 与旧 expected 完全不同 blob（连"同屏不同帧"都不是——见 §3 语义分析）。
- 建议后续在 P0-0 手册 §3.6/§3.7/§7.3/§7.4 做 dated 更正：明确 `$26 bit7` **不是** Ending 判据、目标帧映射假设不成立、
  组 D 来源需另立取证卡（§5）。

## 1. 采集记录（合法性与 DONE 字段）

| 项 | C3 `tas-stg08-direct` | C2 `tas-stg07-relay` |
|---|---|---|
| 命令 | `python tools/scenario_run.py run tas-stg08-direct --timeout 5400` | 同（`tas-stg07-relay`） |
| 启动/结束 | 18:03:19 → 18:09:54（库内 wall 393.7s） | 18:10:29 → 18:19:03（库内 wall 513.8s） |
| run.json | frames=120000 trace_lines=120000 timed_out=false | frames=120000 trace_lines=120000 timed_out=false |
| DONE | `{"frames":120001,"frames_plan":120000,"frames_actual":120001,"overshoot":true,"trace_lines":120000,"captures":80,"shots":80,"bot_stop":false,"finalize_by":"inputPolled-overshoot"}` | `{"frames":120000,"frames_plan":120000,"frames_actual":120000,"overshoot":false,"trace_lines":120000,"captures":80,"shots":80,"bot_stop":false,"finalize_by":"endFrame","frame_errors":142149}` |
| trace sha256 | `d6944fe9a4de8b712eaeadb8f710624ba8bd983610791a9aafb62866a107a547` | `e12e6ac4d51afaaba8912b38ca77b98bf9f5ffe4f2bc040afe616fda61a71c24` |
| 判据（Lead F1） | `done!=null ∧ trace_lines==frames ∧ ¬timed_out ∧ frame_errors==0`（"无字段按 0"**仅首轮约定**；frame_errors 证据以 §2.4 final rerun=0 为准）⇒ 首轮 **PASS** | 同判据 ⇒ 首轮 **DEGRADED**（`frame_errors=142149`，不能算 PASS） |

> **【F3】首轮 vs final 区分**：本表为**首轮（旧 lua）**记录；**final rerun**（最终 lua，§2.4）DONE 为
> C1 40000/0、C2 120000/`frame_errors=0`、C3 120000/`frame_errors=0`。磁盘上 `analysis/traces/tas-stg0{7,8}-*.done`
> 与 `.run.json` **已被 final rerun 覆盖**；首轮快照只保留在 `analysis/repro/p0-0-groupD/capture-tas-stg0{7,8}-*.json`（gitignored）。
>
> C3 的 `overshoot=true/frames_actual=120001/trace_lines=120000/finalize_by=inputPolled-overshoot` 同时是
> **capture.lua task-105 修复生效的实车证据**（旧版：119994/120000 无 DONE；新版正常收尾）。
> C2 的 `frame_errors=142149` 属 decomp-verifier F1 类（bot 逐帧 nil hp 等错误仍完成），本报告按 DEGRADED 记录；
> 根因（platform-porter task-121 定位并修复）：`tas-stg07-relay.lua.log` 首条错误 =
> `inputPolled error at frame 1260: attempt to perform arithmetic on a nil value (local 'hp')` ——
> `tools/tas_bot.lua` 的 `stg01-clear` 读 hp 为 nil；修复后 bot-short `frame_errors=0`（非 bot title sha 不变）。
> **C2/C3/I5 将在 bot 修复（task-121）+ capture.lua 修复后重跑**；若重跑仍 blob 0/6，则维持本裁定并附
> "修复后仍无来源"负证据（重跑数据另记，不替换本报告已记录的首轮证据）。

## 2. §3.6 三判据逐对结果

### 2.1 判据① capture blob == 旧 expected（逐对）

旧 expected sha256（`oracle/screens/expected/`）：

| expected | bytes | sha256 |
|---|---:|---|
| `l4_fade-clear.png` | 7486 | `14802bcf460a4cb1c6e76b1d6f8c2101c67dc3b98b3e41d13f2762d335953f65` |
| `l4_ending-1.png` | 4348 | `19b88b1ffce9a2b7463c689455979c662ec2b65dc271cfc46c283d1b6fea8d79` |
| `l4_ending-2.png` | 4014 | `0f8e11694d53cde85c740a7600855093e4b3a4de34cb10633879630d86b7066b` |
| `l4_ending-3.png` | 3838 | `f319f1fe5256e5ad4f535f6b75e722b0ad58747c697c09996fbd7fad10cf6957` |
| `l4_ending-4.png` | 4154 | `6ff345dc5c3f454ee5545fd343387a906aa0373832997531d7dacd1ba8997091` |
| `l4_ending-5.png` | 4154 | `6ff345dc5c3f454ee5545fd343387a906aa0373832997531d7dacd1ba8997091`（与 ending-4 同 blob） |

逐帧对照（候选 `capture_every 1500 probe` → `shots/<name>_f%05d_probe.png`）：

| 目标帧 | expected | C3 shot sha256 | C3 blob_eq | C2 shot sha256 | C2 blob_eq |
|---:|---|---|---|---|---|
| 28500 | `l4_fade-clear` | `99465a06aad57dde…` | ❌ | `99465a06aad57dde…` | ❌ |
| 30000 | `l4_ending-1` | `9eac3cb799a6b2b0…` | ❌ | `9eac3cb799a6b2b0…` | ❌ |
| 31500 | `l4_ending-2` | `e8d387c60c94de64…` | ❌ | `e8d387c60c94de64…` | ❌ |
| 33000 | `l4_ending-3` | `0cde881ea75736a3…` | ❌ | `0cde881ea75736a3…` | ❌ |
| 34500 | `l4_ending-4` | `8a27717f2d93b060…` | ❌ | `8a27717f2d93b060…` | ❌ |
| 36000 | `l4_ending-5` | `3359c59f3fb881de…` | ❌ | `3359c59f3fb881de…` | ❌ |

**全量扫描**：C3 80 张 shots、C2 80 张 shots **逐张 sha 与 6 张旧 expected 全不匹配（BLOB_HITS: NONE）**——
不是"目标帧选错"级别问题，而是候选 run 里**根本没有出现过**这 6 张画面（至少不在 capture_every 相位上）。
> 另注：C3 与 C2 的目标帧 shots **逐字节相同** ⇒ 两个场景在这些帧等价（输入前缀相同、`poke 0180` 的
> stage-select 未持久生效，见 §3），组 D 没有从 C2 获得新信息。

### 2.2 判据② `$26 bit7` → mode=Ending/fade 推导

从 `frames.jsonl` 的 `watch 0026 flags26` 读取（候选帧）：

| 帧 | C3 flags26 | C3 bit7 | C3 stage_no | C2 flags26 | C2 bit7 | C2 stage_no |
|---:|---|---:|---:|---|---:|---:|
| 28500 | 0x00 | False | 1 | 0x00 | False | 1 |
| 30000 | 0x00 | False | 1 | 0x00 | False | 1 |
| 31500 | 0x00 | False | 1 | 0x00 | False | 1 |
| 33000 | 0x80 | True | 2 | 0x80 | True | 2 |
| 34500 | 0x80 | True | 2 | 0x80 | True | 2 |
| 36000 | 0x80 | True | 2 | 0x80 | True | 2 |

**结论：判据②不成立，且暴露 §3.6 口径问题**：
- 目标帧 `stage_no`=**1/2**（不是 8），即这些帧仍是**前两关游戏段**，不可能是 ending/fade；
- `flags26 bit7=0x80` 在 f≈5217 起大段出现（stage2 战斗段），**并非 Ending 专属** ⇒ **`$26 bit7` 不能作为
  Ending 判据**（原 §3.6 判据②需 dated 更正；正确判据应来自 `$95F9` 入口后的 mode/checkpoint 或专用 watch）。
- C3 全帧 `stage_no` 计数 **{1:26901, 2:65993, 3:23144, 8:3962}（合计 120,000）**：`stage_no==8` 只在 **f1255–5216**
  （poke 0180=08 后被引擎覆盖；f1–f7 为预热帧，旧稿 26894 少计 7 帧，已按独立复算更正）⇒ "stage8 直入"
  不成立、120000f 内未跑到 stage8 关末/ending。

### 2.3 判据③ trigger 合法性

- C3：`poke 1255 0180 08`（stage-select 尝试）+ `poke 01BE/01B6/01B7=01`（assist 三秘技）+ `bot stg01-clear from 1260`
  ⇒ **`trigger=injected/assisted`**（README 纪律：不得冒充 natural）；
- C2：同上但 `0180=07`；**旧跑 DEGRADED（frame_errors=142149）**；**重跑合法（frame_errors=0）但 blob 0/6、
  目标帧 stage1/2**（§2.4）⇒ 即便修好 bot 也不构成组 D 来源。
- 两场景 trigger **可如实登记为 injected/assisted**（合法性 OK），但 trigger 合法**不能替代** ① blob / ② 语义。

### 2.4 修复后重跑（2026-10-18 晚，dated；capture.lua + tas_bot 修复后）——仍 0/6

> 背景：旧跑受两类工具缺陷影响（capture.lua 终止条件 → C1/C3 无 DONE；`tas_bot.lua` 3 处 nil 变量 →
> C2 DEGRADED `frame_errors=142149`）。修复后（Lead 提交 capture.lua 199b542；tas_bot task-121）用**最终 lua**
> 重跑 C1/C2/C3，判据 `done!=null ∧ ¬timed_out ∧ trace_lines==done.trace_lines ∧ frame_errors==0`。

| 候选 | DONE / 判据 | trace sha256 | blob（目标帧） | 全 shots 扫描 |
|---|---|---|---|---|
| C1 `tas-stg08b2-direct` | `{"frames":40000,"frames_actual":40000,"trace_lines":40000,"captures":26,"shots":26,"overshoot":false,"finalize_by":"endFrame","frame_errors":0}` ⇒ **PASS** | `486a157e968e1665c77049af63840b054e3a1d581dc912ff3401d008aa16c198` | **0/6** | **NONE**（26 张） |
| C2 `tas-stg07-relay` | `{"frames":120000,"frames_actual":120000,"trace_lines":120000,"captures":80,"shots":80,"overshoot":false,"finalize_by":"endFrame","frame_errors":0}` ⇒ **PASS** | `e12e6ac4d51afaaba8912b38ca77b98bf9f5ffe4f2bc040afe616fda61a71c24` | **0/6** | **NONE**（80 张） |
| C3 `tas-stg08-direct` | `{"frames":120000,"frames_actual":120000,"trace_lines":120000,"captures":80,"shots":80,"overshoot":false,"finalize_by":"endFrame","frame_errors":0}` ⇒ **PASS** | `d6944fe9a4de8b712eaeadb8f710624ba8bd983610791a9aafb62866a107a547` | **0/6** | **NONE**（80 张） |

- 三候选重跑后**全部合法**（`frame_errors=0`），但 **blob 仍 0/6、全 shots NONE**；目标帧语义仍为
  `stage_no`=1/2、`flags26`=0x00/0x80（与 §2.2 一致）⇒ **"修复后仍无来源"（负证据）**。
- C1 的合法化只解除"采集/工具"层面的 BLOCKED，**不改变组 D 来源结论**；I5 采集层状态更新见
  `docs/findings/p0-0-capture-run1.md` 的 dated 追加。
- 机器可读：`analysis/repro/p0-0-groupD/capture-{tas-stg08b2-direct,tas-stg07-relay,tas-stg08-direct}-final.json`
  + `ledger-rerun.jsonl`（gitignored）。

## 3. 为什么未命中（证据化分析）

1. **"stage8 直入"假设不成立**：`poke 0180=08`（C3）⇒ `stage_no=8` 仅 **f1255–5216（3962 帧）**；
   `poke 0180=07`（C2）⇒ `stage_no=7` 仅 **f1255–5216（3962 帧）**、**stage8=0**；随后均被引擎流程覆盖为
   1/2/3。两个"direct/relay"场景实际从 stg01 起 bot 自推，120000f 内**未到达 stage8 关末/ending**
   （C2 连 stage8 都未进入 ⇒ 结论更强）。
2. **目标帧映射假设不成立**：`28500/30000/31500` 在候选 run 里是 stage1/2 战斗段；旧 expected 的帧号来自
   **另一条历史 run 的帧域**，跨 run 的绝对帧号不可直接比较（这正是 §3.6 "帧同余≠来源"的更强反例）。
3. **画面级不可能**：全 80 张 shots 与 6 张 expected 全不匹配 ⇒ 不是相位/±1 帧问题。
4. **C2 旧跑 DEGRADED；重跑合法但画面仍不匹配**：旧跑 `frame_errors=142149`（nil hp 等 bot 缺陷，task-121 已修）；
   重跑 `frame_errors=0` 且 DONE 合法，但 **blob 仍 0/6、目标帧 stage1/2**（§2.4）⇒ 可用性问题是**画面/语义**，
   不是 bot 错误计数。

## 4. 处置与后续建议

### 4.1 立即处置（P0-0 冻结前）

- 组 D 6 对：#6-#11 → **`status=blocked:no-source`**；**不 staging、不替换 expected**；manifest 6 行 blocked，
  §8.2 验收对 6 行不通过（P0-0 不得宣称全绿）。
- §7.7 staging A 的 ending-1..5/fade 项**跳过**（保留旧 expected + `blocked:no-source`）。
- 已采 C3/C2 作为**负证据**存档（`analysis/repro/p0-0-groupD/`，gitignored）：`capture-tas-stg08-direct.json`、
  `capture-tas-stg07-relay.json`、`blob-compare-C3.json`、`ledger.jsonl`。

### 4.2 手册 dated 更正建议（需 Lead 派卡，本报告不直接改手册）

1. **§3.6 判据②错误**：`$26 bit7` 不是 Ending 判据（f5217 起 stage2 战斗段即置位）⇒ 改为 `$95F9` 入口后的
   mode/checkpoint（或 `watch` 增加 mode/`$180`/`$95F9` 面），并在 C1/C2/C3 候选上重验。
2. **目标帧假设**：停止用"旧 run 帧号 28500-36000"作跨 run 硬映射；改为 **any-frame blob 搜索**（dense capture，
   如 `capture_every 300`）在"确实到达 ending 的 run"里找匹配。
3. **候选顺序**：C1（I5）在 capture.lua 修复后可选重试（bot+probe 仍重，建议 probe 稀疏 overlay，预算 60-90min）；
   C4/C5 未评估；**若要组 D 定谳，需先有一条"到达 ending"的合法 run**（v5 300000f 或注入全链 `s5-full-inject`）。
4. **C2 若复用**：bot `frame_errors` 已由 task-121 修复（重跑 frame_errors=0）；但**画面/语义仍不匹配**，
   暂不复用于组 D 真值；若未来复用需重新过三判据。

## 5. 证据指针（可复算）

| 内容 | 指针 |
|---|---|
| C3 **首轮** DONE（overshoot=120001/finalize_by=inputPolled-overshoot） | `analysis/repro/p0-0-groupD/capture-tas-stg08-direct.json`（首轮快照，gitignored） |
| C3 **final rerun** DONE（120000/`frame_errors=0`/endFrame） | `analysis/traces/tas-stg08-direct.done` / `.run.json`（当前磁盘=final；含 `frame_errors`） |
| C2 **首轮** DONE（`frame_errors=142149`，DEGRADED） | `analysis/repro/p0-0-groupD/capture-tas-stg07-relay.json`（首轮快照，gitignored） |
| C2 **final rerun** DONE（120000/`frame_errors=0`） | `analysis/traces/tas-stg07-relay.done` / `.run.json`（当前磁盘=final） |
| 逐对 blob+flags 数据 | `analysis/repro/p0-0-groupD/capture-tas-stg08-direct.json`、`capture-tas-stg07-relay.json`、`blob-compare-C3.json` |
| 采集台账 | `analysis/repro/p0-0-groupD/ledger.jsonl` |
| 旧 expected sha | `oracle/screens/expected/l4_{fade-clear,ending-1..5}.png`（本报告 §2.1 列全 sha） |
| I5 失败证据（C1） | `analysis/repro/p0-0-capture/incident-tas-stg08b2-stall.json` + `docs/findings/p0-0-capture-run1.md` §4 |
| 采集方法/环境 | `docs/findings/p0-0-capture-run1.md` §1；`analysis/p0-0-env.txt` |

复算命令（只读）：

```powershell
# blob 逐张扫描（C3/C2 shots vs 6 张旧 expected）
python - <<'PY'
import hashlib, pathlib
SH=pathlib.Path("shots"); EXP=pathlib.Path("oracle/screens/expected")
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
exps={n:sha(EXP/f"{n}.png") for n in ["l4_fade-clear","l4_ending-1","l4_ending-2","l4_ending-3","l4_ending-4","l4_ending-5"]}
for name in ["tas-stg08-direct","tas-stg07-relay"]:
    hits=[(s.name,n) for s in sorted(SH.glob(f"{name}_f*.png")) for n,h in exps.items() if sha(s)==h]
    print(name, "shots", len(list(SH.glob(f"{name}_f*.png"))), "hits", hits or "NONE")
PY
# flags26/stage_no @目标帧（任一候选）
python -c "import json;[print(r['f'],r['w'].get('flags26'),r['w'].get('stage_no')) for r in map(json.loads,open('analysis/traces/tas-stg08-direct.frames.jsonl',encoding='utf-8')) if r['f'] in (28500,30000,31500,33000,34500,36000)]"
```

## 6. 验收自查

- [x] C3/C2 采集合法（run.json + DONE 字段记录；C2 按 `frame_errors` 标 DEGRADED）；
- [x] §3.6 三判据逐对结果（①0/6 全 miss + 全量 80 张 NONE；② `stage_no`=1/2、`$26 bit7` 非 Ending 证据；
  ③ trigger=injected/assisted 如实标注）；
- [x] 组 D 结论 `blocked:no-source`（未硬凑；保留旧 expected、不 staging）；
- [x] 证据指针/复算命令齐全；`analysis/`、`shots/` 未入库；未改代码/gates/基线；
- [ ] docs-links PASS（提交前复核）。
