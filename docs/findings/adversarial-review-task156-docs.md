# 对抗复核：冻结前置③文档 dated（task-156）——adversarial-review-task156-docs

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用 spec-writer 自述**）
> 被审：手册 `docs/findings/l4-baseline-freeze-plan.md`（工作树 M，§3.7 dated 更正 + §8.2 状态化）、方案 `docs/findings/l4-zero-tolerance-attack-plan.md`（工作树 M，§2.1/§2.6 数字 dated 更新）。
> 纪律：**未跑 Mesen/gates check/status/visual**；只读 `shots/`、`target/p0-0/render/g-manifest-v2.json`、`p0-0-readjudication-gui.md` 与两份文档；本报告只写该文件、未 git add/commit。

---

## 0. VERDICT：**pass**（三块核心事实/数字全部独立复算一致；仅 1 条低风险格式建议：§2.1/§2.6 旧表未实际加 strikethrough，仅以 dated 块标"旧 headless 历史"）

- **§3.7 四联来源扫描 PASS（A）**：`shots/stg08-timeline_f*.png` **125 张**全量 sha：`62a8ef06c4b9…` **64 帧**（min/max=16500/33900）；`f14600`=`ddb322ac5702…`、`f16000`=`2824cb13…`——与 §3.7 L960-969 逐一一致；expected 侧 `l4_stg06-late/l4_stg08-{combat,late,start}` 均 `62a8ef06…`、#17 `l4_stg07-start`=`ddb322ac…`、#18 `l4_stg07-combat`=`2824cb13…`。`#14 l4_stg02-start` 仍 `788d6984…`（与 title expected 同 boot-black 复制品；f3404/f16200 亦命中该 blob）→ "唯一 remaining expected-invalid"成立。
- **方案数字 PASS（A）**：`g-manifest-v2`（GUI）summary = rendered **18**/blocked **4**/replaced **5**/expected_matches **0**；18 对 diff 独立复算 min **0** / median **9,766** / max **58,159** / sum **242,555**；expected_unique=**13** / actual_unique=**14**（重复组：expected `788d`×2、`62a8ef06`×4、`6ff345dc`×2；actual `406f3962`×4、`5aa8a4f8`×2）——与 readjudication-gui §0/§3 及方案 §2.1/§2.6 dated 块逐项一致。旧数字（280,132/20.7247%/1/22/12/22 等）原文保留为历史。
- **§8.2 状态化 PASS（A）**：`oracle/screens/l4-manifest.json` **不存在**（实测 `Test-Path=False`；只有 gitignored `target/p0-0/render/g-manifest-v2.json`）；状态块 ✅2（环境接收、§3.6 组 D 2 sourced+4 blocked）/❌9（②manifest、③22/22、④expected 安装、⑥四联安装、⑦去重登记、⑧visual-report、⑩路径、⑪冻结 commit、⑫Lead/复核）与实际一致；⑨方案 dated 更新标"本卡完成、提交后可打勾"；"**只有 §8.2 全绿后才做冻结 commit**"明写（L1781）；结论"2/12 ✅ ⇒ 不得冻结"成立。
- **trigger=injected 如实**：`stg08-timeline` 场景含多次 `poke 0026/0400` 关末门注入（`oracle/scenarios/stg08-timeline.txt` L33/53-54/64-65/…/108-109），§3.7 标 injected、不冒充 natural ✓。
- **旧 pass2 留档**：§3.7 旧"I5/I5b pass2 选帧"段落保留并明示作废（L942-944/L952-958）；四联/#17/#18 标 `valid-with-source`、**待 §7.7 安装核验**（L966-968）✓。
- **格式/范围**：`docs-links` **PASS**（392 md/467 links）；未改 expected/threshold/code/gates；工作树仅两份 docs M（+并行 untracked 证据文件）。

---

## 1. §3.7 四联/#17/#18 扫描（A，独立复算）

命令（只读）：

```python
import hashlib, pathlib, collections
g=collections.defaultdict(list)
for p in pathlib.Path('shots').glob('stg08-timeline_f*_*.png'):
    f=int(p.stem.split('_f')[1].split('_')[0])
    g[hashlib.sha256(p.read_bytes()).hexdigest()].append(f)
```

结果：

| blob sha（前缀） | shots 数 | frames min/max | 对应 expected |
|---|---:|---|---|
| `62a8ef06c4b9` | **64** | **16500 / 33900** | `l4_stg06-late`+四联 #16/#19/#20/#21（同 blob） |
| `ddb322ac5702` | 1 | 14600 | #17 `l4_stg07-start` |
| `2824cb13…` | 1 | 16000 | #18 `l4_stg07-combat` |
| `788d6984…` | 2 | 3404 / 16200 | #14 `l4_stg02-start`（boot-black 复制品；仍 invalid） |

- 总 shots **125**，与文档 §3.7 "125 张全量"一致；64 帧 = 16500/16600/16800/…/33900 的 group（含显式 tag 帧），不是简单等差帧号 —— 文档写"f16500–33900 共64帧"按 blob group 核无误。
- `l4_ending-4/5` 不在本扫描范围（C1/C3 `6ff345dc…` 已有另证），与 §3.7 组 D 说明不冲突。
- **处置口径**：`valid-with-source`（待 §7.7 安装）；source=`stg08-timeline:<frame>:<tag>`；trigger=injected；旧 pass2 作废留档——均与文档一致。

## 2. 方案数字（A，与 g-manifest-v2/readjudication-gui 逐项）

`g-manifest-v2.json`：22 pairs、`pairs` 18（4 blocked 不在 pairs）；summary rendered18/blocked4/replaced5/expected_matches0/deterministic18。18 对 `diff_pixels` 排序复算：

- min **0**（title）；median **9,766.0**；max **58,159**；sum **242,555**；
- `expected_matches=0` ⇒ **0/18 actual==expected sha**；
- expected_unique **13** / actual_unique **14**（重复组见 §0）；
- 方案 §2.1 L109-117 / §2.6 L201-209 的 dated 块与上述一致；旧 280,132/20.7247%/1/22/12/22 与旧 17/15 口径均留存历史（**仅建议补 strikethrough，见 F1**）。

## 3. §8.2 状态化（A）

- ✅2：①环境接收（GUI 输入有效性证据）、⑤§3.6 组 D（2 sourced +4 显式 blocked）；❌9：②manifest/l4-manifest.json 不存在、③22/22（现18/18）、④expected §7.7 安装、⑥四联来源落地安装、⑦去重登记进 manifest、⑧visual-report/strict、⑩产物路径、⑪冻结 commit、⑫Lead/独立复核；另⑨本卡 dated 数字完成待提交打勾。
- `oracle/screens/l4-manifest.json` 不存在（硬证据）；`target/p0-0/render/g-manifest-v2.json` gitignored，且缺 trigger/input_sha/render_env/render_argv/dedupe 等字段——②未达成成立。
- 结论"**§8.2 未全绿（2/12 ✅）⇒ 不得冻结、不得生成 baseline freeze commit**"与"**冻结 commit 只有全绿后才做**"表述准确。

## 4. Findings 四分类

### 4.1 证据/数字（无）
- §3.7 64帧/f14600/f16000/#14、方案 18/4/5+diff 统计+0/18+unique13/14、§8.2 硬证据/状态，全部独立复算一致；未发现伪造或超证据。

### 4.2 格式/历史保留（低，建议）
- **F1（低）**：方案 §2.1 L94-107 旧表与 §2.6 L174-199 旧 provenance 表**未加 `~~` 划线**，仅在后文 dated 块声明"旧 headless 历史/作废"；task-156 卡面要求"**旧主张划线保留**"。建议给旧表加 `~~` 或表头显式 `【历史/作废】`（不删数据）；不影响新数字/§8.2 结论。
- 旧 §3.7 pass2 段落已用"作废留档"文字标记（L952-958），符合历史保留要求；如按同一风格统一亦可。

### 4.3 行为/实现（无）
- 不涉及代码；`g-manifest-v2` 是 gitignored 工作产物，文档只引用其 summary/字段；未改 expected/threshold。

### 4.4 越权/伪造（无）
- 只读；未改两份文档/expected/gates；本报告仅新增文件、未 git add/commit；未跑 Mesen/gates check/status/visual。工作树另有的 `p0-0-groupD-s1-source.md`/`tas-stg08-direct-dense.txt`/`tools/l4_render_all.py` 属并行卡，不是本卡改动。

## 5. 命令与输出摘要

```text
# shots scan（125 张）
62a8ef06c4b9… 64 shots frames 16500..33900
ddb322ac5702… 1 shot f14600
2824cb13… 1 shot f16000
788d6984… 2 shots f3404/f16200（#14 invalid boot-black）
expected: l4_stg06-late & four = 62a8ef06…; #17=ddb322ac…; #18=2824cb13…
# manifest g-manifest-v2
pairs=18 rendered=18 blocked=4 replaced=5 expected_matches=0
diff min=0 median=9766.0 max=58159 sum=242555
expected_unique=13 actual_unique=14
# §8.2
l4-manifest.json Test-Path=False
status ✅2 / ❌9（+⑨本卡已完成待提交）；"全绿后才冻结"
docs-links PASS（392 md/467 links）
```

## 6. 复核边界

1. 未跑 Mesen/scenario_run/gates check/status/visual；未重渲任何 actual；所有数字来自既有 gitignored 产物 + 两份文档。
2. 未修改两份被审文档/expected/threshold/code/gates；本报告唯一新增文件。
3. §3.7 的 64 帧为 blob group（含显式 capture tag 帧），不是等差步长；按"f16500–33900 共64帧"口径核无误。
4. F1 仅为历史表 strikethrough 格式建议，不改变 §3.7/方案/§8.2 的事实与结论。

---

【2026-10-18】decomp-verifier，独立复核 task-156（HEAD `61402d8`；手册+方案 docs M）；shots 125/64帧/f14600/f16000/#14、manifest 18/4/5+min0/median9766/max58159/sum242555+0/18+unique13/14、§8.2 ✅2/❌9/l4-manifest 不存在全部复算一致；仅旧表未划线建议。
