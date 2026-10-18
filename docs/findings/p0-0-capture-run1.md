# P0-0-1 输入恢复：P0-0 场景采集 I1-I6（run 1）＋两跑确定性（task-86）

> 日期：2026-10-18｜角色：test-runner（执行）/ l4-analyst（记录）｜状态：**I4/I3/I2/I6/I1 已采集并通过判据；I5 两轮定点 stall ⇒ `BLOCKED: capture-termination`（见 §4）**
> 任务：task-86（P0-0 手册 §2 采集；环境=task-81/85 已复核的 Windows Mesen2 2.1.1 headless）。
> 纪律：严格串行（`analysis/.scenario.lock`）；只读采集，不改代码/gates/基线；`analysis/`、`shots/` 为
> gitignored 工作产物，**不入库**；失败/超时如实记录，不掩盖。
> 环境记录：[`analysis/p0-0-env.txt`](../../analysis/p0-0-env.txt)（gitignored；§1 摘要）。

## 0. 结论摘要（先读）

| 场景 | 状态 | frames/trace_lines | captures/shots | wall | trace sha256（前 16） |
|---|---|---|---|---|---|
| **I5** `tas-stg08b2-direct`（40000f, bot） | ❌ **BLOCKED: capture-termination**（R1 100s 截断；R2/R3 均定点停在 `frames_lines=39998/40000`、bot 侧仍写、无 DONE，可复现；§4） | —（R2/R3: 39998/40000） | 26 / 26 | R2 ~21min、R3 ~13min | — |
| **I4** `stg08-timeline`（34000f） | ✅ PASS | 34000 / 34000 | 125 / 125 | 213.8s | `5f0fb2289cae0d8d` |
| **I3** `stg03-timeline`（24000f） | ✅ PASS | 24000 / 24000 | 83 / 83 | 194.4s | `e9086b8ef2ad9458` |
| **I2** `gameplay-stage1`（2400f, ×2） | ✅ PASS ×2（两跑同 sha） | 2400 / 2400 | 44 / 44 | 133.5s / 98.6s | `94ec3a07a722228e` |
| **I6** `stg06-timeline`（30000f） | ✅ PASS | 30000 / 30000 | 108 / 108 | 126.3s | `e7798edbc12afa1f` |
| **I1** `title-menu`（1080f, ×2） | ✅ PASS ×2（两跑同 sha） | 1080 / 1080 | 37 / 37 | 3.5s / 3.5s | `62c9ea09580654ce` |

- **判据（逐跑）**：`rc=0 ∧ run.json.done!=null ∧ trace_lines==frames ∧ timed_out!=true`（I2/I1 另加两跑
  trace sha 相同 + shots 逐字节相同）。
- **两跑确定性**：I1、I2 **PASS**（trace sha 相同、shots 全量逐字节相同）；**I5 未取得有效首跑 ⇒ 两跑
  确定性未达成（BLOCKED，§4）**。
- **单场景耗时实测**：stg08-timeline 213.8s、stg03-timeline 194.4s、gameplay-stage1 98.6–133.5s、
  stg06-timeline 126.3s、title-menu 3.5s（Windows headless；无 bot 场景均可接受）。

## 1. 环境与执行方式（P0-0-0 记录）

> 完整记录见 `analysis/p0-0-env.txt`（**73 行 / 66 行非空**，gitignored；含 §7 采集完成记录）。关键项：

| 项 | 值 |
|---|---|
| OS | Windows 11 10.0.26100.9457（x64） |
| CPU / RAM / 磁盘 | AMD Ryzen 5 5600X（6C/12T）；15.9 GiB（17.1 GB 十进制）；D: free 50.3 GiB（采集前） |
| Python / cargo / rustc | 3.12.1 / 1.96.0-nightly (cbb9bb8bd 2026-03-13) / 1.96.0-nightly (1d8897a4e 2026-03-13) |
| Mesen2 | `tools/mesen2/Mesen.exe` 70,470,144B；sha256 `d2eb03c2590c648bf329f127ebcfefd70130e7690a9e2ccdba8616faea1fe96b`；FileVersion **2.1.1** |
| 驱动方式 | Windows headless：`Mesen.exe --testRunner --timeout=<max(120, timeout_s+120)> <lua> <rom>`（无需 Xvfb） |
| Lua IO 前置 | `%USERPROFILE%\Documents\Mesen2\settings.json` → **`Debug.ScriptWindow.AllowIoOsAccess=true`**（已核 True） |
| 门禁 | `gates rom` PASS；`gates status` PASS=11/FAIL=0/SKIP=1（addr-bridge）/STATUS OK |
| 采集时间窗 | 2026-10-18 16:38 起（P0-0-0 记录）；本文完成时约 17:40+ |

**命令**（每条独立串行；本卡统一用 `python`，Windows）：

```powershell
python tools/scenario_run.py run <name> --timeout 3600
```

**串行与并发纪律**：
- `scenario_run.py` 自动串行锁（`analysis/.scenario.lock`，Windows=msvcrt 1-byte）；驱动遇
  "另一个场景在运行" 等 30s 重试，**绝不并行**；
- 串行锁**不防外部手工写文件**：曾有并发注入 `shots/title-menu_f99999_fake.png` 致 artifacts 计数 38 的先例
  —— 本轮验收一律以 `run.json` 的 `DONE` 判据 + ledger 记录为准；发现 shots 数不符记 `artifacts_warning`，
  不静默删除（本轮未出现）。

## 2. 逐场景采集记录（run.json 关键字段 + trace sha）

| 场景 | frames | trace_lines | captures | shots | wall_s | first_sample_s | done | trace sha256（完整） |
|---|---:|---:|---:|---:|---:|---:|---|---|
| I4 `stg08-timeline` | 34000 | 34000 | 125 | 125 | 213.8 | 7.5 | `{"frames":34000,...,"shots":125,"bot_stop":false}` | `5f0fb2289cae0d8d7ba8b485db648ba3118868a56f27474f0fbcf023491f34fd` |
| I3 `stg03-timeline` | 24000 | 24000 | 83 | 83 | 194.4 | 7.0 | `{"frames":24000,...,"shots":83,...}` | `e9086b8ef2ad945820a2bf3ec821d4009b4675ed9f17a1170f10b67640294505` |
| I2 `gameplay-stage1`（run1） | 2400 | 2400 | 44 | 44 | 133.5 | 6.0 | `{"frames":2400,...,"shots":44,...}` | `94ec3a07a722228e16e5c9470a2c3f83ceb7a3b65ac5e7e664ebcf9c584d9c8b` |
| I2 `gameplay-stage1`（run2） | 2400 | 2400 | 44 | 44 | 98.6 | 5.0 | 同上 | 同上（**相同**） |
| I6 `stg06-timeline` | 30000 | 30000 | 108 | 108 | 126.3 | 1.5 | `{"frames":30000,...,"shots":108,...}` | `e7798edbc12afa1f497a221f5818be173de799790f6af00c9aba3913e764f473` |
| I1 `title-menu`（run1） | 1080 | 1080 | 37 | 37 | 3.5 | 0.5 | `{"frames":1080,...,"shots":37,...}` | `62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331` |
| I1 `title-menu`（run2） | 1080 | 1080 | 37 | 37 | 3.5 | 0.5 | 同上 | 同上（**相同**） |

> 产物路径：`analysis/traces/<name>.{frames.jsonl,captures.jsonl,run.json,mesen.log,lua.log}` +
> `shots/<name>_f%05d_<tag>.png` + `analysis/traces/<name>_f%05d_<tag>.ram.hex`（均 gitignored）。

## 3. 两跑确定性（I1 / I2；I5 待重试）

| 场景 | trace sha256 相同 | shots 逐字节相同 | shots 数 | 判定 |
|---|---|---|---|---|
| I1 `title-menu` | ✅ `62c9ea09…89a331` | ✅ 37/37 全等 | 37 | **PASS** |
| I2 `gameplay-stage1` | ✅ `94ec3a07…4d9c8b` | ✅ 44/44 全等 | 44 | **PASS** |
| I5 `tas-stg08b2-direct` | —（首跑未成功，见 §4） | — | — | **PENDING**（长预算重试后补） |

> 说明：I2 两跑 wall 差异（133.5s vs 98.6s）来自机器负载（采集期间有其它 agent 的 cargo/gate 任务），
> 但 trace 与全部 shots 逐字节相同 ⇒ 输入确定性成立（wall 不影响判据）。

## 4. I5 `tas-stg08b2-direct` 失败记录（failed:over-budget，如实保留）

### 4.1 R1：Mesen2 testRunner 默认 100s 超时（工具缺陷，已修复）

- 首轮（16:38 启动）长场景全部在 wall≈120.6s 被截断：`stg08-timeline trace=18171/34000`、
  `tas-stg08b2-direct trace=11069/40000`（**16:38 首跑**）、`stg03-timeline trace=13681/24000`，`done=None`；
- **16:42 次跑** `tas-stg08b2-direct` 同样截断：`trace=10984/40000`（`done=None`）——**两次 100s 截断并列**
  （首跑 11069、次跑 10984；数值不同但都在第 100s 处被 testRunner 强停）。
- 根因（Lead 定位）：Mesen2 testRunner `TestRunnerTimeout` 默认 **100s**（`CommandLineHelper.cs:31`）；
  `scenario_run.py` headless 未传 `--timeout`。修复：自动传 `--timeout=max(120, timeout_s+120)`（工作区已改，
  task-86 重跑生效；本轮 I4/I3/I2/I6/I1 全部正常）。
- **ledger 审计补记（F107-4）**：v1 早期截断行原 `status=None`，已在 ledger **追加**
  `phase=v1-annotation` 补记（`status=failed:truncated-100s`，注明“审计友好补记，不改历史事实”）；
  涉及 `tas-stg08b2-direct`（11069/40000、10984/40000）、`stg08-timeline`（18171/34000）、
  `stg03-timeline`（13681/24000）。

### 4.2 R2：bot 场景 + 10 条 probe 的 IO 过重（over-budget）

- `tas-stg08b2-direct` 是 **bot 场景**（`bot stg01-clear from 1265`，per-frame 决策）+ **10 条 probe_mem** +
  `capture_every 1500` + assist 三 poke；产物极大。
- 实测（17:02:01 启动，**kill 于 `17:23:46`**——以 `incident-tas-stg08b2-stall.json` 的 `killed_at` 为准，
  报告旧文曾写 17:26，此处统一）：
  - `frames.jsonl` **39998/40000 行**、40,014,329B，最后写入 **17:07:56** 后 15+ 分钟未再增长；
  - `probe.jsonl` **188,704,783B**（≈ 3,000,001 事件上限，17:12 封盘）；
  - `inputs.jsonl` 16,921,241B、`events.jsonl` 9,968,167B——**bot 侧仍持续增长**；
  - `shots/` 26 张（max 11,069B，正常）；D 盘 free 48 GiB；`done` 文件**不存在**；
  - `mesen.log` 尾部仅 `[CPU] Uninitialized memory read` 行（无异常栈）；`lua.log` 无 error；
  - Mesen 进程 CPU≈1239s（~1 核持续占用）。
- **判定**：不是死锁，是 **bot per-frame 决策 + probe 写 + 大帧集 IO 过重**（Lead 裁定
  `failed:over-budget`）；R1 100s 截断 run 线性增长无 stall ⇒ 排除"固定帧号必然卡死"。
- 机器可读记录：`analysis/repro/p0-0-capture/incident-tas-stg08b2-stall.json` + ledger
  （`status=failed:over-budget`，含两轮证据/日志尾部）。

### 4.3 R3：长预算重试复现同一定点 stall ⇒ BLOCKED（capture-termination）

- **R3 实跑**：`python tools/scenario_run.py run tas-stg08b2-direct --timeout 5400`（Mesen 侧
  `--timeout=5520`），17:37:13 启动；前 5min 正常（frames 40MB 写到 **17:42:30**、probe 188.7MB 到
  **17:46:28**），随后**再次停在 `frames_lines=39998/40000`**，bot 侧 `events`/`inputs` 继续增长
  （9.68MB@17:49:27、10.15MB@17:49:30），`done` 不存在、`run.json` 未生成；Mesen CPU=725s。17:50 按证据 kill。
- **可复现性**：R2 与 R3 是**两次独立运行**，都在 `frames_lines=39998` 处停 trace/DONE，而 bot 回调仍在写
  ⇒ **定点可复现，排除"预算不足/IO 慢"**（R1 100s 截断 run 是线性增长，无此现象）。
- **根因假设（待工具卡验证）**：`capture.lua` 终止条件 `if frame == frames or bot_stop` 未命中——bot 场景
  （per-frame 回调 + 10 probe + capture）下 Emu 帧计数/脚本 `frame` 可能跳帧/分叉，**恰好跳过 40000** ⇒
  永不写 DONE，Mesen 一直跑到 testRunner 超时（R2/R3 均如此）。
- **建议修复（tools 卡，独立 commit + 复核）**：`tools/scenario_capture.lua` 终止条件改
  `frame >= frames`（或用独立计数 `n = n + 1; if n >= frames`），DONE 记录实际 frame；可选 bot 场景用
  probe 稀疏 overlay（保留 8 标准区间）降 IO。修复后重试 I5。
- **当前取舍**：I5 **BLOCKED（不阻塞 P0-0 其它步骤）**；组 D/四联 expected 来源按 P0-0 手册
  §3.6/§3.7 改用候选 **C2 `tas-stg07-relay`**（`stg07-clear f28683` 已在 `tas-clear-full.l5.jsonl`）或
  **C3 `tas-stg08-direct`**，并在 mapping 中写明取舍（Lead 已确认此 fallback 方向）。

## 5. 台账与产物清单

| 内容 | 路径（均 gitignored） |
|---|---|
| 环境记录 | `analysis/p0-0-env.txt` |
| 采集台账（逐跑 JSONL，含无效/失败尝试） | `analysis/repro/p0-0-capture/ledger.jsonl` |
| 汇总 JSON | `analysis/repro/p0-0-capture/summary-v4.json`、`summary-v3.json`（历史） |
| I5 事故记录 | `analysis/repro/p0-0-capture/incident-tas-stg08b2-stall.json`、`i5-retry.json` |
| 场景产物 | `analysis/traces/<name>.*`、`shots/<name>_f*.png` |

## 6. 执行历史（诚实记录 driver 迭代）

1. **v1**（16:38）：未感知 testRunner 100s 超时 → 长场景截断（I5/I4/I3），短场景 OK（I2/I1）；
2. **v2**（16:49）：修复后首跑立即 rc=2（前一轮 kill 遗留串行锁）→ 失败记录，未污染产物；
3. **v3**（17:01）：I5 长跑 bot+probe IO 过重，17:02:01 启动到 17:23:46 kill（~21.7min）仍未落 DONE → 按
   Lead 指示 kill，标 over-budget；
4. **v4**（17:24）：跳过 I5，完成 I4/I3/I2×2/I6/I1×2 全部 PASS；
5. **v5**（17:37–17:50）：I5 长预算（5400s）单独重试 → **仍在 f=39998 定点 stall**（与 R2 一致）⇒ 标
   `BLOCKED: capture-termination`（§4.3），不阻塞其它场景。

## 7. 验收自查

- [x] I4/I3/I2/I6/I1 均满足 `done!=null ∧ trace_lines==frames ∧ ¬timed_out`（run.json 实值见 §2）；
- [x] I1/I2 两跑 trace sha 相同 + shots 逐字节相同（§3）；
- [x] I5 两轮失败如实记录（`failed:over-budget` + 证据 + 日志尾部，§4）；
- [x] 串行纪律/并发纪律记录（§1）；`analysis/`、`shots/` 未入库；未改代码/gates/基线；
- [x] I5 已按证据判定 **BLOCKED: capture-termination**（R2/R3 定点复现 + 根因假设 + 修复建议，§4.3）；
      组 D/四联 fallback=C2/C3（Lead 已确认方向）；修复后重试由后续 tools 卡承担；
- [ ] docs-links PASS（本文件提交前复核）。
