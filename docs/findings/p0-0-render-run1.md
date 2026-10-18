# P0-0-2：22 对 canonical 重渲两跑 + manifest/去重（task-114）

> 【2026-10-18 更正·superseded】本文件基于 headless(testRunner) 采集，其 emu.setInput 不生效 ⇒ 输入未注入、语义无效；重采见 task-133（GUI 模式）；原数字保留作历史。

> 日期：2026-10-18｜执行：l4-analyst（test-runner）｜状态：**16/16 可渲对两跑 png+idx 全部一致；6 对组 D blocked:no-source；0 失败**
> 任务：task-114（P0-0 手册 §4.1/§5.1/§5.3）；**不需要 Mesen**（render-frame 纯离线，输入=P0-0-1 采集 trace/probe + phase-batch 文本）。
> 边界：未改代码/gates/基线；`target/`、`analysis/`、`shots/` gitignored 不入库；**未跑 gates check/status/visual**。
> 产物：`target/p0-0/render/{run1,run2}/`、`target/p0-0/render/manifest.json`、`target/p0-0/render/summary.json`（均 gitignored）。

## 0. 结论摘要

| 指标 | 值 |
|---|---|
| 22 对总数 | 22 |
| 可渲（×2 跑） | **16**（组 A 3 + B 6 + C 6 + E 1） |
| blocked:no-source（不渲染） | **6**（组 D #6-#11 ending-1..5/fade-clear） |
| 两跑 png sha 一致 | **16/16** |
| 两跑 idx sha 一致 | **16/16** |
| 渲染失败/缺失 | **0** |
| replaced-invalid-asset（expected 待 §7 重采；actual 已重渲） | 5（#14 stg02-start + 四联 #16/#19/#20/#21） |
| 总耗时 | ~66min（3976.6s；phase-batch 3 场景 + 32 次 render） |
| manifest | `target/p0-0/render/manifest.json`（22 行 + env + summary + dedupe） |

> **【F125-1】`status` 语义（provisional，勿误用）**：本 manifest 的 `status` 是 **`render_status`（provisional）**，
> 只表示 **render 成功 + 两跑 deterministic**，**不代表 actual 与 expected blob 已确认相等**（P0-0-2 expected 尚未定稿）。
> 实测：16 个 rendered pair **全部 actual≠expected**（其中 13 个 expected 文件存在、3 个 group A 的 expected 文件名带 tag 后缀）；
> `status=ok` 的 11 个里 8 个属这 13 个「已知不等」集合。后续 wrapper/mapping 请改用 `render_status` 字段。

> **实际重渲发现（需 §7.5/§8 裁定，不得静默）**：actual 侧 6 对重渲成**同一 blob**
> `5f93bdbb2c84…`（`l4_stg06-late, l4_stg07-combat, l4_stg07-start, l4_stg08-combat, l4_stg08-late,
> l4_stg08-start`，全部来自 `stg08-timeline`）——与旧 expected「同 blob 复制」现象一致/同族（OBS-1 商店冻结屏，
> B 级解释），但**actual 侧也同图**必须记入 dedupe/documented_duplicates 并在 §7.5 明确裁定（不视为 0 容差通过，
> 也不据此改 expected）。

## 1. 方法与命令（可复现）

环境（写入 manifest `env`）：Windows x64；Python 3.12.1；repo HEAD `dabd6afd…`；
ROM sha256 `4d76901c…c77ae`；render 二进制 `target/p0-0/build/debug/render-frame.exe`
sha256 `ab115570b8ebaf8c…520f4`（独立 target `target/p0-0/build` 构建；**39 个诊断 env 全部 unset**，
仅保留 `REPO_ROOT/PATH` 等基础 env —— canonical 口径 §5.2）。

```powershell
# 1) 构建（独立 target，不污染共享 target）
$env:CARGO_TARGET_DIR = "target/p0-0/build"
cargo build -q -p game-desktop --bin render-frame

# 2) phase-batch：生成 render_frame 需要的文本写集（3 个 render 场景；标题无文本）
cargo run -q -p game-oracle --bin phase-batch -- gameplay-stage1 gameplay-stage1-text
cargo run -q -p game-oracle --bin phase-batch -- stg03-timeline stg03-timeline-text
cargo run -q -p game-oracle --bin phase-batch -- stg08-timeline stg08-timeline-text
# → analysis/phase/{gameplay-stage1,stg03-timeline,stg08-timeline}-text.jsonl
#   写点：7040 / 13,987,517 / 3,499,346

# 3) 每对两跑（隔离目录；真实路径均在 repo 内，--out-dir/--log-path fail-closed）
$env:REPO_ROOT = (Get-Location).Path
target/p0-0/build/debug/render-frame.exe <frame> <name> <scene> `
  --out-dir target/p0-0/render/runN --log-path target/p0-0/render/runN/<name>.log
# 4) 两跑比较：png sha256 + idx sha256（逐对）
```

**输入 sha**：每对记录 `analysis/traces/<scene>.frames.jsonl`、`<scene>.probe.jsonl`、
`analysis/phase/<scene>-text.jsonl` 的 sha256（title-menu 的 probe/text = null，合法 M7）。

> **【F125-3/4 口径注】**：①`phase-batch` 只打印「N 调用回放/M 写点」，**无 run.json/wall 字段**，故本报告
> 总耗时 3976.6s 是 driver wall（含 phase-batch + 32 次 render），`§2` 的 wall 仅为 render-frame 单跑耗时；
> ②manifest `env` 只记 **9 类基础键摘要 + REPO_ROOT/二进制 sha**（39 个诊断 env 全 unset），非完整 env dump
> （完整 env 卫生见 `run1/<name>.log` 的 `env={}` 字段）。

## 2. 逐对结果（22 行；`png sha12` = run1 实际输出，两跑全等）

| # | name | frame | scene | status | png sha12 | 两跑 png/idx | 单跑 wall（run1/run2） |
|---:|---|---:|---|---|---|---|---|
| 1 | gameplay-stage1_f01260 | 1260 | gameplay-stage1 | ok | `62417db716bc` | ✅/✅ | 18.5 / 18.4s |
| 2 | gameplay-stage1_f01500 | 1500 | gameplay-stage1 | ok | `05de7bb46cdf` | ✅/✅ | 21.9 / 25.6s |
| 3 | gameplay-stage1_f02349 | 2349 | gameplay-stage1 | ok | `28f094fd515d` | ✅/✅ | 34.7 / 34.1s |
| 4 | l4_clear-inject | 4600 | stg03-timeline | ok | `a666b16136f5` | ✅/✅ | 129.2 / 130.8s |
| 5 | l4_combat-variety | 6500 | stg03-timeline | ok | `50b9d9ae983b` | ✅/✅ | 144.7 / 150.5s |
| 6 | l4_ending-1 | 30000 | — | **blocked:no-source** | — | —（不渲染） | — |
| 7 | l4_ending-2 | 31500 | — | **blocked:no-source** | — | —（不渲染） | — |
| 8 | l4_ending-3 | 33000 | — | **blocked:no-source** | — | —（不渲染） | — |
| 9 | l4_ending-4 | 34500 | — | **blocked:no-source** | — | —（不渲染） | — |
| 10 | l4_ending-5 | 36000 | — | **blocked:no-source** | — | —（不渲染） | — |
| 11 | l4_fade-clear | 28500 | — | **blocked:no-source** | — | —（不渲染） | — |
| 12 | l4_late-combat | 12000 | stg03-timeline | ok | `801e99d5143b` | ✅/✅ | 192.6 / 184.2s |
| 13 | l4_shop-window | 5400 | stg03-timeline | ok | `b560e275d5e6` | ✅/✅ | 144.8 / 146.7s |
| 14 | l4_stg02-start | 3404 | stg03-timeline | replaced-invalid-asset | `af6cedfbef08` | ✅/✅ | 135.2 / 116.7s |
| 15 | l4_stg03-start | 5700 | stg03-timeline | ok | `d808b2349802` | ✅/✅ | 161.8 / 160.7s |
| 16 | l4_stg06-late | 22000 | stg08-timeline | replaced-invalid-asset | `5f93bdbb2c84` | ✅/✅ | 119.3 / 122.3s |
| 17 | l4_stg07-combat | 16000 | stg08-timeline | ok | `5f93bdbb2c84` | ✅/✅ | 97.3 / 93.4s |
| 18 | l4_stg07-start | 14600 | stg08-timeline | ok | `5f93bdbb2c84` | ✅/✅ | 86.3 / 89.3s |
| 19 | l4_stg08-combat | 18000 | stg08-timeline | replaced-invalid-asset | `5f93bdbb2c84` | ✅/✅ | 103.2 / 102.0s |
| 20 | l4_stg08-late | 28000 | stg08-timeline | replaced-invalid-asset | `5f93bdbb2c84` | ✅/✅ | 120.0 / 111.1s |
| 21 | l4_stg08-start | 16600 | stg08-timeline | replaced-invalid-asset | `5f93bdbb2c84` | ✅/✅ | 79.1 / 79.0s |
| 22 | title-menu_f00180_boot-black | 180 | title-menu | ok | `1b1940f30bf8` | ✅/✅ | 0.08 / 0.08s |

- `status` 规则：组 D → `blocked:no-source`（task-106 定谳；不 staging/不硬凑）；四联+stg02-start →
  `replaced-invalid-asset`（expected 待 §7 重采，actual 按当前 trace 已重渲）；其余 `ok`。
- 完整 sha256（png/idx/输入三件/argv/log 路径）见 `target/p0-0/render/manifest.json` 的 `pairs[]`。
- `l4_ending-1..5` / `l4_fade-clear` 六对**没有渲染**：来源 blocked:no-source（见
  `docs/findings/p0-0-groupD-source-decision.md`；C1/C2/C3 修复后重跑仍 blob 0/6）。

### 2.1 actual 六对同 blob 的逐对 expected sha（F125-5；与 `p0-0-same-blob-adjudication.md` 一致）

| name | frame | actual run1 sha（前 16） | expected sha（前 16） | expected 是否同 blob |
|---|---:|---|---|---|
| `l4_stg06-late` | 22000 | `5f93bdbb2c84612b` | `62a8ef06c4b9bcd1` | 是（四联同 blob） |
| `l4_stg07-combat` | 16000 | `5f93bdbb2c84612b` | `2824cb134987c7b3` | 否（旧 expected 单独图） |
| `l4_stg07-start` | 14600 | `5f93bdbb2c84612b` | `ddb322ac57021bc6` | 否（旧 expected 单独图） |
| `l4_stg08-combat` | 18000 | `5f93bdbb2c84612b` | `62a8ef06c4b9bcd1` | 是（四联同 blob） |
| `l4_stg08-late` | 28000 | `5f93bdbb2c84612b` | `62a8ef06c4b9bcd1` | 是（四联同 blob） |
| `l4_stg08-start` | 16600 | `5f93bdbb2c84612b` | `62a8ef06c4b9bcd1` | 是（四联同 blob） |

> 裁定（task-126）：actual 6 对同 blob = **真实 OBS-1 冻结屏**（源 Mesen 截图同 sha `2b2d6ab0…`、trace 内容键同、
> 沙盒补渲 f5000 不同 vs f10000–34000 同），**非渲染缺陷**；expected 四联同 blob 仍按 §7.2 重采。详见
> [`p0-0-same-blob-adjudication.md`](p0-0-same-blob-adjudication.md)。

## 3. 两跑确定性（16/16 PASS）

- 逐对比较：`run1/<name>.png` vs `run2/<name>.png` sha256、`run1/<name>.idx.json` vs `run2/<name>.idx.json` sha256；
- **16/16 对 png 相同、16/16 对 idx 相同**（manifest `summary.deterministic_ok=16, deterministic_fail=0`）；
- 失败即 FAIL 的判据未触发；无 `blocked:nondeterministic`；
- wall 差异（如 #5 144.7 vs 150.5s）来自机器负载，不影响输出 sha。

## 4. 去重（dedupe；按 manifest 22 名过滤，不 glob actual 目录）

manifest `dedupe`：**expected_unique=14（reduction 5）、actual_unique=11（reduction 5）**，documented groups：

| side | names | sha256（前 12） | 说明 |
|---|---|---|---|
| expected | `l4_ending-4`, `l4_ending-5` | `6ff345dc5c3f` | 旧 expected 已知同 blob（ending 静态页；§7.4 裁定） |
| expected | `l4_stg02-start`, `title-menu_f00180_boot-black` | `788d6984576b` | #14 expected 无效（=boot-black 复制品；§7.1 重采） |
| expected | `l4_stg06-late`, `l4_stg08-combat`, `l4_stg08-late`, `l4_stg08-start` | `62a8ef06c4b9` | 四联旧 expected 同 blob（OBS-1 解释 B；§7.2 重采） |
| **actual** | `l4_stg06-late`, `l4_stg07-combat`, `l4_stg07-start`, `l4_stg08-combat`, `l4_stg08-late`, `l4_stg08-start` | `5f93bdbb2c84` | **本次实际渲染也同图（stg08-timeline 6 个帧域）** —— 需 §7.5/§8 明确裁定（见 §5） |

> **口径**：本 manifest 是 P0-0-2 产物（expected 重采/裁定尚未定稿）；`duplicate_of`/`reuse` 需 mapping.tsv
> （P0-0-1b）与 §7 裁定后回填，本报告不臆断「独立/复用」。
> **actual 6-group 裁定（task-126）**：真实同画面（OBS-1 冻结屏），非缺陷；建议登记 actual-only
> documented_duplicate 并扩展 §7.5 `registry_ok`（actual 组非 mapping 四联组子集），详见
> [`p0-0-same-blob-adjudication.md`](p0-0-same-blob-adjudication.md)。

## 5. 失败/跳过/待裁定清单

| 类型 | 对 | 处置 |
|---|---|---|
| `blocked:no-source`（组 D） | #6-#11（ending-1..5/fade-clear） | **不渲染、不 staging**；expected 保留旧件；manifest 6 行 blocked（P0-0 不得宣称全绿） |
| `replaced-invalid-asset` | #14 stg02-start、#16/#19/#20/#21 四联 | actual 已重渲；expected 待 §7.1/§7.2 重采后置 `expected_sha256`+status 回填 |
| **actual 同 blob（新发现）** | #16-#21 六对（`stg08-timeline`） | **待 §7.5/§8 裁定**：若为 OBS-1 冻结屏真实渲染 → 记 documented_duplicates 并在 §8.2 验收说明；若为渲染缺陷 → 立差异族修复卡。**不得据此改 expected/threshold**。 |
| `skip:stale-input` | **无** | 所有 render 输入 trace 均来自 P0-0-1 有效采集（I1/I2/I3/I4/I6；I5 重跑有效但未被 22 对使用）；无截断历史 trace 入渲染。 |
| 渲染失败 | **无** | 0 失败；所有 log 在 `target/p0-0/render/runN/<name>.log` |

> 与 task-110/121 的旁证：修复后 C1/C2/C3 重跑均 `frame_errors=0` 且 DONE 合法（见
> `p0-0-groupD-source-decision.md` §2.4）；组 D 结论仍 `blocked:no-source`。I5 采集层 BLOCKED 已解除
> （`p0-0-capture-run1.md` §4.4）。

## 6. 复算/抽查命令（复核方用）

```powershell
# 两跑一致性（逐对）
python - <<'PY'
import hashlib, json, pathlib
R=pathlib.Path("target/p0-0/render"); m=json.loads((R/"manifest.json").read_text(encoding="utf-8"))
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
bad=[]
for p in m["pairs"]:
    if not p["render"]: continue
    n=p["name"]
    a=sha(R/"run1"/f"{n}.png"); b=sha(R/"run2"/f"{n}.png")
    ai=sha(R/"run1"/f"{n}.idx.json"); bi=sha(R/"run2"/f"{n}.idx.json")
    ok=(a==b and ai==bi and a==p["actual_run1_sha256"])
    print("OK " if ok else "DIFF", n, a[:12]); bad += [] if ok else [n]
print("AGREEMENT:", f"{16-len(bad)}/16", "bad=", bad)
PY
```

## 7. 验收自查

- [x] 22 对清单齐（16 渲 + 6 组 D blocked）；两跑 png/idx 一致 **16/16**；失败 0（§2/§3）；
- [x] manifest 齐全（22 行 + env + summary + dedupe + blocked 列表）→ `target/p0-0/render/manifest.json`；
- [x] 组 D 按 `blocked:no-source` 标注、未渲染/未 staging（§5）；无 stale-input 使用；
- [x] 未跑 gates check/status/visual；未改代码/gates/基线；产物均在 target/、analysis/（gitignored）；
- [x] actual 6 对同 blob 已裁定为真实 OBS-1 冻结屏（task-126，`p0-0-same-blob-adjudication.md`），非渲染缺陷；
      F125-1（render_status/provisional，§0）、F125-5（§2.1 六对 expected sha）、F125-3/4（§1 注）已补；
- [x] docs-links PASS（374 md/459 链接）；独立复核 pass（task-125）。
