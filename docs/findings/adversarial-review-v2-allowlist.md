# 对抗复核报告：f0e9be1（b100c31 `re_render` 豁免收尾 → 显式历史 sha allowlist）

> 复核对象：commit `f0e9be1262f9bee148e90e58a8a46ad2a759292a`
> （`fix(tools): task-171 移除 re_render 布尔豁免，改 #14 历史 sha 显式 allowlist + 反例测试`）
> 复核方：独立对抗复核 agent（verifier，证伪立场）；与实施方非同一 agent。
> 姿态：不采信被审方自述，全部结论由本 agent 独立重导（重跑命令 / 独立 sha 复算 / mutation 探针）。
> 复核时间：2026-01-19（本地时间 2026-01-20 00:26 起）。
> 证据分级标注：【A】硬证据（字节/哈希/命令输出）；【B】代码语境推理；【C】猜测（本报告不落 C 级结论）。

---

## VERDICT: **pass**

六项必查全部独立复现通过；claims ①–⑤ 与实测一致；claims ⑥（写入面）确认只触及
`tools/` 两个文件。mutation 探针（定点 5 例 + 穷举 18 对 × 2 变异 = 36 例）证实：
allowlist **只**放行 `l4_stg02-start` 的历史值 `ebc6372f…`，任意其他 sha（含该历史值
出现在其他 pair）一律 FAIL，`re_render` 无功能残留。4 条 Minor/Info 级观察项均不阻断
（见 findings），其中 2 条为改动前既有行为、1 条为文案硬编码、1 条为"旧 2076"历史值
无法从现存备份复核（不影响被审写入面的正确性）。

---

## 1. 复核范围与方法（独立重导路径）

| # | 必查项 | 复核方法（本 agent 自跑） | 结果 |
|---|---|---|---|
| 1 | allowlist 语义：只放行旧 ebc、其他 sha FAIL、其他 21 对严格、无 re_render 残留 | `git show f0e9be1` 逐行读 diff；`git grep re_render HEAD`；定点 5 例 + 穷举 36 例 mutation 探针 | ✅（详见 §2 F1、§4） |
| 2 | 单测 11/11 | `python -X utf8 tools/test_l4_render_all.py`；另用 `python -m pytest tools/test_l4_render_all.py -q` 交叉 | ✅ 11/11；pytest `11 passed in 0.22s` |
| 3 | `--validate g-manifest-full.json` PASS | `python -X utf8 tools/l4_render_all.py --validate target/p0-0/render/g-manifest-full.json` | ✅ `MANIFEST-CHECK: PASS` |
| 4 | mutation 探针 | monkeypatch `_load_pairs_v2`（内存态变异，脚本落 `target/_ar171_probe/`，未动 tracked） | ✅ 全部按预期 FAIL/放行 |
| 5 | g-manifest-v2.json #14 字段刷新 | 直接读 JSON + 独立 sha256 复算磁盘文件 + `diff_visual.compare` 独立复算 diff | ✅（详见 §3） |
| 6 | 无 expected/gates/threshold 写入面 | `git show --stat f0e9be1` + `git status --porcelain`（干净） | ✅ 仅 2 个 tools/ 文件 |

---

## 2. Findings（四分类）

### 分类一：阻断（Blocker）— **0 条**

无。

### 分类二：严重（Major）— **0 条**

无。

### 分类三：次要（Minor）— **1 条**

| ID | 项 | 结论 | 证据 |
|---|---|---|---|
| F-M1 | `v2_run1_sha_ok()` 在 **v2 sha 或磁盘 sha 缺失**时返回 `(True, None)`（不否决）。commit 正文措辞"其余任何漂移/其他 21 对一律 FAIL"严格说只覆盖"两侧都在且不等"的分支 | 【A】代码 `tools/l4_render_all.py:174`（`if not v2_sha or not disk_sha or v2_sha == disk_sha: return True, None`）；单测 `tools/test_l4_render_all.py:97-98` 显式断言该行为 | **非新增弱化**：与改动前 `rec.get(...) and a1 and != a1` 的否决条件等价；且缺图/缺记录另有独立否决（`actual run1/run2 缺图`、`g-manifest-v2 缺该 pair 记录`，`tools/l4_render_all.py:357,379`）。残余缝隙仅剩"v2 记录存在但 `actual_run1_sha256` 字段为空/缺失 ⇒ 交叉核对静默跳过"。建议（不阻断）：v2 记录存在但该字段为空时降级为 issue 或显式注明"跳过" |

### 分类四：信息（Info）— **3 条**

| ID | 项 | 结论 | 证据 |
|---|---|---|---|
| F-I1 | 交叉核对只覆盖 `actual_run1_sha256`；v2 的 `actual_run2_sha256`/`idx_sha`/`input_sha` 不参与否决 | 【A】`git grep actual_run2_sha256 tools/l4_render_all.py` 仅出现于字段写入（L335），无核对分支 | 改动前既有口径，非本次回退。两跑一致性另有 `deterministic`/`两跑 png/idx 不一致` 对**磁盘文件**否决（L316,360）；本 agent 独立复算 run1/run2 png 同为 `27fa3737…`、idx 同为 `436120b6…` |
| F-I2 | 运行时 notes 标注文案硬编码 `ebc6372f…`/`27fa3737…` 两个字面值（`tools/l4_render_all.py:374`） | 【A】代码直读 | 纯文案；若未来重渲致磁盘 sha 变化而 v2 仍是历史值，标注措辞会过期，但放行判定仍严格（allowlist 集合只含 ebc 一个值，穷举探针已证）。可改为 f-string 动态取值 |
| F-I3 | commit 正文 "diff_pixels=2352/diff_ratio=0.038281（旧 2076/0.033789）" 的"旧 2076"无法从现存备份复核：`g-manifest-v2.json.pre-task165.bak` 旧 entry 的 `diff_pixels=2495`（旧 expected `788d6984…` 对旧 actual `ebc6372f…`，与 `docs/findings/p0-0-render-gui-run.md:49` 记录一致） | 【A】bak JSON 直读 | "2076"应属 AR171-1 更正中"旧 expected 历史值区分"的中间态（见 `45093dd` 提交说明），gitignored 工作区历史态不可再证。**当前值 2352/0.038281 已被本 agent 独立复算确认（§3），被审写入面正确性不受影响**；仅记录口径差异 |

### 已证实确认项（Claims 逐条核销）

| Claim | 核销结果 | 证据 |
|---|---|---|
| ① 删除 `re_render=True` 布尔豁免 | **已证实**【A】 | diff 中 `PAIR_SPEC` 的 `"re_render": True` 行与 `build_manifest` 的 `spec.get("re_render")` 跳过分支均被删除；`git grep re_render HEAD` 仅余：注释 1 处（`tools/l4_render_all.py:164`）、负向断言 1 处（`tools/test_l4_render_all.py:78`）、历史复核文档 4 处（`docs/findings/adversarial-review-install-workspace.md`）；探针打印 `re_render_keys_in_spec: []`。**无功能残留** |
| ② 显式 allowlist + `v2_run1_sha_ok()` 语义（相等⇒OK；仅历史值⇒OK+notes；其余/其他 21 对⇒FAIL） | **已证实**【A】 | `HISTORICAL_V2_RUN1_SHA = {"l4_stg02-start": frozenset({"ebc6372f…cb94"})}`（`tools/l4_render_all.py:165-169`）；`v2_run1_sha_ok`（L172-178）；定点 5 例 + 穷举 36 例探针全部按预期（§4）；notes 标注分支（L371-374）实测命中。allowlist 值的真实性：`ebc6372f…` 与 bak 旧 entry 的 `actual_run1_sha256`/`actual_run2_sha256`/`run1.png_sha` **三处完全一致**，确系真实历史值而非杜撰 |
| ③ g-manifest-v2.json #14 已刷新 | **已证实**【A】 | 直读 target JSON：`diff_pixels=2352`、`diff_ratio=0.038281`、`actual_run1/2_sha256=27fa3737…6281`、`run1/2.idx_sha=436120b6…cab2`、`input_sha frames=b628ad3e…73bd / probe=7db8a474…2995 / text=fbe7c209…2e62`、`expected_sha256=44e7800d…68f3`；备份 `g-manifest-v2.json.pre-task165.bak` 存在。独立复算：磁盘 `g_run1/g_run2` 的 png/idx sha、`oracle/screens/expected/l4_stg02-start.png` sha 与上述逐一吻合；`diff_visual.compare` 独立复算 diff=**2352**/61440=**0.038281** ✓ |
| ④ 单测 11/11（新增 allowlist 正/反例、断言无 re_render） | **已证实**【A】 | `python -X utf8 tools/test_l4_render_all.py` → `l4_render_all 回归: 11/11 通过`（11 个 `def test_`）；`python -m pytest tools/test_l4_render_all.py -q` → `11 passed in 0.22s`；`test_v2_run1_sha_allowlist_semantics` 覆盖相等/历史值/任意漂移/他 pair 旧值/缺失两侧 6 断言；`assert not any(p.get("re_render") for p in L.PAIR_SPEC)` 在位 |
| ⑤ `--validate` PASS + DEDUPE 14=18-4 双侧 registry_ok=True | **已证实**【A】 | `--validate target/p0-0/render/g-manifest-full.json` → `MANIFEST-CHECK: PASS / pairs=22 rendered=18 blocked=4 replaced=0 / DEDUPE: expected 14=18-4 actual 14=18-4 registry_ok=True`；另独立跑 build（写入 target/_ar171_probe/）复现 `DEDUPE: expected 14=18-4 actual 14=18-4 registry_ok=True documented_ok=True -> PASS / TWO-RUN: PASS (18/18) / VALIDATION: PASS`。且被验 full manifest 的 `inputs.g_manifest_v2_sha256=c4b279e2…016b` = 当前 v2 文件实测 sha ⇒ 校验确系针对**刷新后**的 v2 |
| ⑥ 无 expected/gates/threshold 写入面 | **已证实**【A】 | `git show --stat f0e9be1` → 仅 `tools/l4_render_all.py`（+39/−7）、`tools/test_l4_render_all.py`（+24/−2）；`git status --porcelain` 空（tracked 零改动）。commit 含 `Evidence:`/`Confidence: 97` 字段（touch tools/ 未触发证据门禁敏感清单，字段齐备亦合规） |

---

## 3. #14 字段核对明细（claim ③，全部独立复算）

| 字段 | v2 #14 记录值 | 本 agent 独立复算 | 一致 |
|---|---|---|---|
| run1/run2 png_sha | `27fa3737335346322615d8cdc7ba21ff50d764b1a05f7928a09a51baf40d6281` | sha256(g_run1/g_run2 png)=同值（246083 B，两跑字节相同） | ✅ |
| run1/run2 idx_sha | `436120b680f7c3c23b6f3bf60ada9b67ceab23a914b95a54fc41e55b778ccab2` | sha256(g_run1/g_run2 idx.json)=同值（2102 B，两跑字节相同） | ✅ |
| diff_pixels / diff_ratio | 2352 / 0.038281 | `tools/diff_visual.py` `compare()` 独立调用 = 2352 / 0.038281（256×240=61440） | ✅ |
| input_sha frames/probe/text | `b628ad3e…73bd` / `7db8a474…2995` / `fbe7c209…2e62` | 字段在位（12 位前缀与 commit 声明 b628ad3e/7db8a474/fbe7c209 吻合） | ✅ |
| expected_sha256 | `44e7800dac894205b248ee5e0e3ebe436ec604c48235f203e2cdea1d8c8368f3` | sha256(oracle/screens/expected/l4_stg02-start.png)=同值（1411 B） | ✅ |
| 旧 entry 备份 | `g-manifest-v2.json.pre-task165.bak` | 存在；旧 run1/run2/png_sha 三处=`ebc6372f…cb94`（=allowlist 唯一值） | ✅ |

---

## 4. mutation 探针输出摘要（脚本：`target/_ar171_probe/probe_mut.py`、`probe_sweep.py`；monkeypatch 内存态 v2，tracked 零改动）

定点 5 例（`probe_mut.py`）：

```
M0 baseline(真实 v2)            validation.ok=True   issues=[]                      => PROBE-OK
M1 #14 run1 sha -> deadbeef*8   validation.ok=False  "…deadbeefdead… 不符（不在历史 allowlist）" => PROBE-OK
M2 #14 run1 sha -> ebc6372f     validation.ok=True   notes 含 "历史 ebc6372f" 标注     => PROBE-OK
M3 gameplay-stage1_f01260 -> ebc6372f  validation.ok=False  "…ebc6372f79d9… 不符（不在历史 allowlist）" => PROBE-OK
M4 gameplay-stage1_f01260 -> deadbeef*8 validation.ok=False  "…deadbeefdead… 不符（不在历史 allowlist）" => PROBE-OK
SUMMARY: ALL-PROBES-PASS (5/5)
```

穷举扫描（`probe_sweep.py`）：v2 全部 **18** 个 pair × {历史 ebc, 任意漂移（cafebabe*8）}：

```
18/18 行符合预期：仅 l4_stg02-start + ebc 一格放行（ok=True 且带标注），
其余 17 对的 ebc 旧值、全部 18 对的任意漂移 ⇒ ok=False，issue 均为"不在历史 allowlist"。
SWEEP: PASS
```

即"只放行旧 ebc 值、任意其他 sha FAIL、其他 pair 严格"三条语义均机器证伪不成立（探针无法推翻）。

---

## 5. 命令输出摘要（本 agent 自跑，Windows，`python -X utf8`）

```
$ python -X utf8 tools/test_l4_render_all.py
l4_render_all 回归: 11/11 通过          （11 个 PASS 明细含 test_v2_run1_sha_allowlist_semantics）

$ python -m pytest tools/test_l4_render_all.py -q
11 passed in 0.22s

$ python -X utf8 tools/l4_render_all.py --validate target/p0-0/render/g-manifest-full.json
MANIFEST-CHECK: PASS
  pairs=22 rendered=18 blocked=4 replaced=0
  DEDUPE: expected 14=18-4 actual 14=18-4 registry_ok=True

$ python -X utf8 tools/l4_render_all.py --out target/_ar171_probe/g-manifest-full.probe.json \
      --mapping-out target/_ar171_probe/mapping.probe.tsv
MANIFEST: … pairs=22 rendered=18 blocked=4 replaced=0 documented_duplicate=5
TWO-RUN: PASS (18/18 png+idx)
DEDUPE: expected 14=18-4 actual 14=18-4 registry_ok=True documented_ok=True -> PASS
VALIDATION: PASS

$ git show --stat f0e9be1
 tools/l4_render_all.py      | 39 ++++++++++++++++++++++++++++++---------
 tools/test_l4_render_all.py | 24 ++++++++++++++++++++++--
 2 files changed, 52 insertions(+), 11 deletions(-)

$ git status --porcelain
（空）
$ git grep re_render HEAD
（仅注释 1 + 负向断言 1 + 历史复核文档 4；无功能残留）
```

---

## 总结

`f0e9be1` 的五项声称经独立重导全部成立：布尔豁免 `re_render` 已彻底移除（无功能残留），
收窄后的 `HISTORICAL_V2_RUN1_SHA` 确实只含 `l4_stg02-start` 的一个真实历史值
`ebc6372f…cb94`（与备份文件旧 entry 三处哈希互证，非杜撰白名单）；定点 + 穷举共 41 例
mutation 探针证明该 allowlist 除"该 pair 该历史值"外对一切漂移一律 FAIL（含历史值外溢到
其他 pair），且历史值放行时在 pair notes 显式标注；g-manifest-v2 #14 的 png/idx/input/expected
/diff 六组字段经独立 sha 复算与 `diff_visual.compare` 复算全部吻合（diff=2352/0.038281）；
单测 11/11（自跑 + pytest 双证）、`--validate` PASS、DEDUPE 14=18-4 双侧 registry_ok=True
独立复现；`git show --stat`/`git status` 证明写入面仅 `tools/` 两文件，未触碰
expected/gates/threshold/基线。4 条 Minor/Info 观察（缺失侧不否决的既有缝隙、run2/input 不入
交叉核对、标注文案硬编码、"旧 2076"历史值不可复核）均不构成回退或纪律红线，其中前两条为
改动前既有口径。**结论：pass**——被审变更可从"待审稿"转为"步骤完成"；建议实施方在后续
迭代顺手处理 F-M1（v2 记录存在但 run1 sha 字段为空时显式告警）与 F-I2（标注文案动态化），
不阻断本轮验收。
