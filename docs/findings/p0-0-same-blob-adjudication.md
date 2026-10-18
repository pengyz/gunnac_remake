# P0-0-2 actual 6 对同 blob 裁定：真实 OBS-1 冻结屏（非渲染缺陷）（task-126）

> 【2026-10-18 更正·superseded】本文件基于 headless(testRunner) 采集，其 emu.setInput 不生效 ⇒ 输入未注入、语义无效；重采见 task-133（GUI 模式）；原数字保留作历史。

> 日期：2026-10-18｜角色：l4-analyst（取证/记录）｜状态：**结论 = `真实同画面`（OBS-1 冻结屏）；6 对 actual 同 blob 合理、非渲染/输入映射缺陷**
> 任务：task-126（P0-0-2 复核 F125-2 证据并入）；输入 = P0-0-1 trace/probe/text + P0-0-2 重渲产物（`target/p0-0/render/`）。
> 纪律：只读取证；不改代码/expected/threshold；未跑 Mesen（沙盒补渲仅 render-frame）；`analysis/`、`target/`、`shots/` gitignored 不入库。
> 关联：[`p0-0-render-run1.md`](p0-0-render-run1.md)（P0-0-2 manifest/dedupe）、[`p0-0-groupD-source-decision.md`](p0-0-groupD-source-decision.md)、
> [`docs/bug-ledger.md`](../bug-ledger.md) OBS-1（"末关出店转场截断/结构冻结"候选观察区）。

## 0. 结论（先读）

**`真实同画面`**：6 对 actual 同 blob 不是渲染缺陷、不是输入映射缺陷，而是 **oracle 源画面本身在这 6 帧就是同一张冻结屏（OBS-1 现象）**。证据链（A 级为主）：

1. **源侧（A）**：`stg08-timeline` run 的 Mesen 截图（`shots/stg08-timeline_f*`）在**这 6 个帧全部同 sha**
   `2b2d6ab04350b637…`（6240B）——模拟器实际屏幕一致（复核方 F125-2 独立发现，本报告复算确认）；
2. **trace 侧（A）**：6 帧的**屏幕内容键完全相同**（`bg_nmi` / `oam` / `shot_objs`），仅 `iram`/`zp`
   等易变计数不同；log 显示 6 对 `nt_nz=1684`、`scroll=(0,0)`、`probe=240995`、`env={}` 全同；
3. **渲染侧（A）**：render 只对帧号/输入状态响应——沙盒补渲 **f5000 输出不同**（`a1b59af2…`），而
   **f10000/14600/22000/28000/34000 输出全同**（`5f93bdbb…`）⇒ 冻结窗口内同图；无"空写流→固定图"回退
   （`TEXTFALLBACK` 只在文本缺失时启用，本次 text sha 存在；见 §4）；
4. **deterministic**：6 对 ×2 跑 png/idx 全同（P0-0-2 `two_run_agreement=16/16`）。

**处置**：6 对（`l4_stg06-late, l4_stg07-combat, l4_stg07-start, l4_stg08-combat, l4_stg08-late, l4_stg08-start`）
**不按缺陷剔除**；actual 同 blob 应登记为 **documented_duplicates（真实同画面，OBS-1）**。**但与 expected 无关**：
13 个 expected 有文件的 rendered pair 全部 actual≠expected（P0-0-2 的 L4 差异照旧，基线预期 FAIL by design）；
expected 四联仍按 §7.2 重采、组 D 仍 `blocked:no-source`（P0-0 不得宣称全绿）。

## 1. 逐对证据表（6 对；全部 scene=`stg08-timeline`）

| name | frame | 输入 frames sha（前 12） | probe sha | text sha | nt_nz | scroll | actual png sha256（前 16） | idx sha（前 16） | 源 Mesen shot sha（前 16） | expected sha（前 16） |
|---|---:|---|---|---|---|---|---|---|---|---|
| `l4_stg06-late` | 22000 | `5f0fb2289cae` | `66502e3071d0` | `a0b2496fe7d2` | 1684 | (0,0) | `5f93bdbb2c84612b` | `eb5828ec629c8e77` | `2b2d6ab04350b637` | `62a8ef06c4b9bcd1` |
| `l4_stg07-combat` | 16000 | 同上 | 同上 | 同上 | 1684 | (0,0) | `5f93bdbb2c84612b` | `eb5828ec629c8e77` | `2b2d6ab04350b637` | `2824cb134987c7b3` |
| `l4_stg07-start` | 14600 | 同上 | 同上 | 同上 | 1684 | (0,0) | `5f93bdbb2c84612b` | `eb5828ec629c8e77` | `2b2d6ab04350b637` | `ddb322ac57021bc6` |
| `l4_stg08-combat` | 18000 | 同上 | 同上 | 同上 | 1684 | (0,0) | `5f93bdbb2c84612b` | `eb5828ec629c8e77` | `2b2d6ab04350b637` | `62a8ef06c4b9bcd1` |
| `l4_stg08-late` | 28000 | 同上 | 同上 | 同上 | 1684 | (0,0) | `5f93bdbb2c84612b` | `eb5828ec629c8e77` | `2b2d6ab04350b637` | `62a8ef06c4b9bcd1` |
| `l4_stg08-start` | 16600 | 同上 | 同上 | 同上 | 1684 | (0,0) | `5f93bdbb2c84612b` | `eb5828ec629c8e77` | `2b2d6ab04350b637` | `62a8ef06c4b9bcd1` |

> 完整 sha/argv/log 见 `target/p0-0/render/manifest.json` 与 `target/p0-0/render/run1/<name>.log`；
> 源 shot 文件：`shots/stg08-timeline_f{14600,16000,16600,18000,22000,28000}_*.png`（均 6240B，sha 前 16 全同）。

## 2. 输入侧：输入不同吗？

- **文件级**：6 对同一 scene，`frames/probe/text` 三个输入文件**完全相同**（sha 见 §1）；唯一差异是 `frame` 参数
  （14600/16000/16600/18000/22000/28000）。
- **帧级**：6 帧的 trace `w` 中 `bg_nmi`/`oam`/`shot_objs` **逐字段相同**；`iram`/`zp` 不同（计数器/临时值）。
- ⇒ "输入不同而输出相同"**只发生在易变计数上**；决定画面的字段相同 ⇒ 同图合理（非映射缺陷）。
- 渲染 log 六对均为 `probe=240995 nt_nz=1684 scroll=(0,0) env={}`（canonical：39 诊断 env 全 unset）。

## 3. 语义侧：是否处于冻结/过场状态？

- **A 级（oracle 真值）**：该 run 的 Mesen 截图在 6 帧同 sha（§1）；扩展检查显示该窗口存在**两种持续静态屏**
  交替：`2b2d6ab04350…`（51 张，覆盖 9000–33600）与 `d531d33a65d3…`（45 张，覆盖 7500–33900），
  两者都跨数万帧不变化（非正常游戏滚动/战斗画面）——与 **OBS-1「末关出店转场截断/结构冻结」** 一致
  （`docs/bug-ledger.md` 候选观察区：注入链下转场分叉后结构冻结，未立号）。
- 6 对目标帧**全部落在** `2b2d6ab0…` 组 ⇒ 源画面同图。
- **【F128-2】静态组 shots 名单（可复算；glob 命令见下）**：
  - `2b2d6ab04350…` **51 张**，frames = `[9000,9600,10200,10800,11000,11400,12000,12600,13000,13200,13800,14400,14600,15000,15600,16000,16200,16600,16800,17400,18000,18600,19200,19800,20000,20400,21000,21600,22000,22200,22800,23400,24000,24600,25200,25800,26000,26400,27000,27600,28000,28200,28800,29400,30000,30600,31200,31800,32400,33000,33600]`
  - `d531d33a65d3…` **45 张**，frames = `[7500,8100,8700,9300,9900,10500,11100,11700,12300,12900,13500,14100,14700,15300,15900,16500,17100,17700,18300,18900,19500,20100,20700,21300,21900,22500,23100,23700,24300,24900,25500,26100,26700,27300,27900,28500,29100,29700,30300,30900,31500,32100,32700,33300,33900]`
  - 另有小重复组（`988809e0b414af92`×3 = 7200/7800/8400 等，见命令输出）。
  ```bash
  python - <<'PY'
  import hashlib, pathlib, collections
  g=collections.defaultdict(list)
  for p in pathlib.Path('shots').glob('stg08-timeline_f*_*.png'):
      try: f=int(p.stem.split('_f')[1].split('_')[0])
      except Exception: continue
      g[hashlib.sha256(p.read_bytes()).hexdigest()].append(f)
  for h,fs in g.items():
      if len(fs)>1: print(h[:16], len(fs), sorted(fs))
  PY
  ```
- 附：expected 中四联 4 张同 blob `62a8ef06…`（旧 expected 侧同族现象）；#17/#18 的 expected 不同 blob
  （`2824cb13…`/`ddb322ac…`）——**旧 expected 来自另一条历史 run 的帧域**（跨 run 帧号不可硬映射，
  已在 groupD 决策 §3 记录）；本裁定**不判 expected #17/#18 哪侧有效**（那是 §7.1/§7.2 重采范围）。

## 4. 渲染侧：是否有"无内容→回退固定图"路径？

- 代码检查（`crates/game-desktop/src/bin/render_frame.rs`）：
  - `TEXTFALLBACK` 仅在 **text 文件缺失**时启用（`if std::env::var("TEXTFALLBACK").is_ok()`；strict 模式缺文本=空集，
    禁跨场景 fallback）——本 run 6 对 text sha=`a0b2496f…` **存在**，未触发；
  - `pending_row` 是"帧内多行重写只留末行"的行装配逻辑，不产生固定图回退；
  - 未发现 "empty write stream → fixed image" 分支。
- **沙盒补渲实验（A）**：用同一 binary（`target/p0-0/build/debug/render-frame.exe`，canonical env）对
  `stg08-timeline` 补渲 6 帧到 `target/p0-0/adjudicate/`：
  - f5000 → `a1b59af2b4cebe79…`（**不同**）
  - f10000 / f14600 / f22000 / f28000 / f34000 → 全部 `5f93bdbb2c84612b…`（**同**）
  ⇒ render 对 `frame` 有响应；同图只出现在冻结窗口内，**不是渲染忽略帧或固定图回退**。
- 另注：render 输出（256×240，均亮 13.28，5 色）与 Mesen 源屏（均亮 49.94，11 色）仍有 L4 差异 ⇒ 属
  P0-1 渲染差异（基线预期 FAIL by design），与同 blob 无关；且 render 把上述两种交替静态屏都渲成同一图
  （沙盒 10000–34000 同图），属 L4 渲染差异的一部分，登记给 §8/P0-1，不影响本裁定。

## 5. expected 侧名单与 actual 名单关系

| side | 名单 | sha（前 12） | 说明 |
|---|---|---|---|
| expected 同 blob | `l4_stg06-late` + `l4_stg08-{combat,late,start}`（4） | `62a8ef06c4b9` | 旧 expected 四联同图（OBS-1 同族；仍按 §7.2 重采） |
| **actual 同 blob** | 上述 4 + **`l4_stg07-combat`/`l4_stg07-start`**（6） | `5f93bdbb2c84` | 本次重渲同图（真实冻结屏） |
| expected 单独 | `l4_stg07-combat`（`2824cb13…`）、`l4_stg07-start`（`ddb322ac…`） | — | 与 actual 不同属**跨 run 帧域**问题，非本裁定范围 |

- actual 6-group **不是** mapping 四联组（4 名）的子集 ⇒ §7.5 wrapper 的 `registry_ok` 口径需扩展
  （建议：actual-only 组显式登记并加 `adjudicated_by=task-126`），否则 dedupe 校验会把真实同画面判为异常。

## 6. 对 §7.5/§8 的建议

1. **登记（建议采纳）**：在 `mapping.tsv`/manifest `dedupe.documented_duplicates` 增加
   actual-only 组 `["l4_stg06-late","l4_stg07-combat","l4_stg07-start","l4_stg08-combat","l4_stg08-late","l4_stg08-start"]`
   （sha=`5f93bdbb…`，reason=`OBS-1 冻结屏：oracle 源截图同 sha 2b2d6ab0… + trace 内容键同 + 渲染补渲实验`，
   `adjudicated_by=task-126`；不影响 expected 侧 4 组登记）。
2. **可冻结**：6 对 actual 为确定性真实同画面，**不需要因同 blob 剔除/阻断 P0-0**；但 P0-0 仍**不得宣称全绿**
   （expected 四联无效待 §7.2、组 D blocked:no-source、actual≠expected 差异属 P0-1 目标）。
3. **区分两类"同 blob"**：expected 四联同 blob=旧资产无效（§7.2 重采）；actual 六联同 blob=真实冻结屏
   （本裁定，登记白名单）。二者**不可混用**。
4. §8.2 验收：`actual_unique` 按 manifest 22 名过滤 = 11（16 rendered − 5 reduction，含本 actual 组）；
   `expected_unique`=14（expected 三组）；同 blob 组需在 §7.5 与 mapping.duplicate_of 对账后再置
   `registry_ok=true`。

## 7. 复算命令（只读）

```powershell
# ① 六对 actual/expected/源 shot sha
python - <<'PY'
import hashlib, pathlib, json
sha=lambda p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
R=pathlib.Path("target/p0-0/render/run1")
for n,f in {"l4_stg06-late":22000,"l4_stg07-combat":16000,"l4_stg07-start":14600,
            "l4_stg08-combat":18000,"l4_stg08-late":28000,"l4_stg08-start":16600}.items():
    print(n, f, sha(R/f"{n}.png")[:16],
          "shot", sha(f"shots/stg08-timeline_f{f:05d}_" +
                      {"l4_stg06-late":"stg06-late","l4_stg07-combat":"stg07-combat",
                       "l4_stg07-start":"stg07-start-approx","l4_stg08-combat":"stg08-combat",
                       "l4_stg08-late":"stg08-late","l4_stg08-start":"stg08-start-approx"}[n]+".png")[:16])
PY
# ② 沙盒补渲对照（f5000 不同；f10000–34000 同；compare.json 为 {frame:{rc,exists,sha}} 映射，
#    "同图"判据 = sha == d['14600']['sha']；重生成命令见下）
#   python 生成：对 f in [5000,10000,14600,22000,28000,34000] 用 target/p0-0/build/debug/render-frame.exe
#   `--out-dir target/p0-0/adjudicate --log-path .../adj_stg08_f<f>.log`（canonical env、无 Mesen），
#   结果 dict {f:{rc,exists,sha}} 写 target/p0-0/adjudicate/compare.json
python - <<'PY'
import json, pathlib
d=json.loads(pathlib.Path("target/p0-0/adjudicate/compare.json").read_text(encoding="utf-8"))
base=d["14600"]["sha"]
print({f:(v["sha"]==base) for f,v in d.items()})
PY
# ③ trace 内容键（6 帧 bg_nmi/oam/shot_objs 相同）
python - <<'PY'
import json
fs={14600,16000,16600,18000,22000,28000}; rows={}
for l in open("analysis/traces/stg08-timeline.frames.jsonl",encoding="utf-8"):
    r=json.loads(l)
    if r["f"] in fs: rows[r["f"]]=r["w"]
for k in ("bg_nmi","oam","shot_objs","iram","zp"):
    print(k, len({json.dumps(rows[f].get(k),sort_keys=True) for f in fs}))
PY
```

## 8. 验收自查

- [x] 输入侧（文件级/帧级）、语义侧（oracle shots 同 sha + trace 内容键）、渲染侧（fallback 代码 + 沙盒补渲）、
      expected/actual 名单关系逐项给出证据（§1-§5）；
- [x] 结论 `真实同画面`（OBS-1），**非渲染缺陷**；未剔除 6 对、未改 expected/threshold；
- [x] F125-2 证据（源 shots 同 sha `2b2d6ab04350`）已并入 §1/§0；F125-1/5 在
      `p0-0-render-run1.md` 的注记/§5 同步；
- [x] `analysis/`、`target/`、`shots/` 未入库；未跑 Mesen/gates；
- [x] docs-links PASS（374 md/459 链接；platform-verifier task-128 复核 pass）。

## 9. 【2026-10-18 task-143 dated 注】GUI trace 版：actual 6-group 不再成立（旧结论属 headless superseded 语境）

> GUI 重采 trace（输入生效）重渲 18 对后：**actual_unique=14**，actual 同 blob 组只剩 **2 组**：
> ①四联 `l4_stg06-late`/`l4_stg08-combat`/`l4_stg08-late`/`l4_stg08-start` = `406f3962…`（仍为真实同画面：Mesen 源截图
> 同屏 `62a8ef06…` + 新 trace）；②`l4_ending-4`/`l4_ending-5` = `5aa8a4f8…`（assisted 来源）。
> **旧 headless 版的 actual 6-group（含 `l4_stg07-combat`/`l4_stg07-start`）不再成立**——新 trace 下二者已不同图
> （`65de647d…`/`c9070704…`）；旧 6-group 属"headless 输入失效 + 冻结 trace"语境，**作为历史记录保留不删**，
> 但不得再用于 GUI 版 actual 去重/基线口径。四联同 blob 的 OBS-1「真实同画面」结论对 GUI 版仍成立（源截图证据未变）。
> 最新口径与行号见 `docs/findings/p0-0-readjudication-gui.md` §3/§4 与手册 §7.5 task-143 注。
