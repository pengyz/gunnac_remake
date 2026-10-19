# P0-0 冻结前置④：expected §7.7 安装方案（四联/#17/#18 来源登记 + #14 替换）（task-165·方案稿）

> 【2026-10-19 命名收口】pair/expected_source 语义名统一为 **gs5-anchor-render:3450:stg02-start**（Lead 单点裁定，render-ready overlay；task-168 P1-F1）。**注意**：历史采集 shot 的磁盘实名仍是 shots/gs5-stg01-clear-anchor_f03450_stg02-start.png（复现命令按实名执行，见 §6）；overlay 场景 oracle/scenarios/gs5-anchor-render.txt 建成并采集后，以 overlay 同帧 shot 为正式源。

> 日期：2026-10-19｜执行：l4-analyst｜任务：task-165（冻结硬前置④；**先方案，Lead 批准后才执行**）
> 依据：task-154 来源裁定（`p0-0-groupD-s1-source.md` §2/§3）、`l4-baseline-freeze-plan.md` §7.7（唯一安装点，L1543+）、§8.2（B1 核验）；
> 工具：`tools/l4_render_all.py`（PAIR_SPEC/mapping/manifest/registry，L58+）、`tools/diff_visual.py`（0 容差 diff，只读调用）。
> 边界：本卡只写本方案；**执行阶段**才改 `oracle/screens/expected/`（tracked，单独 commit+Evidence+独立复核）、`analysis/repro/mapping*.{tsv,json}`、manifest；不改 threshold/gates、不跑 gates check/status/visual。
> 当前状态：expected 22 文件/17 unique；mapping.tsv 已由 l4_render_all 生成（22 行）；4 blocked 终态（task-154 dense 双网格负证据）。
> **【2026-10-19 Lead 裁定 · dated】** #14 pair 采用 **Option B（重指向 `gs5-anchor-render:3450`，actual 用 gs5-anchor trace f3450 重渲）**；旧 expected 定性改为 **frame-mismatch（错帧覆盖，非伪造）**；
> **前置 blocker（本方案新增）**：现有 `gs5-anchor-render` trace **无 `probe.jsonl`/无 zp·iram·oam 等 RAM blob**（scenario 只有 watch、无 probe_mem/watch_range）⇒ `render-frame` **无法从该 trace 渲出 actual**；需先加 **render-ready overlay（补 probe_mem + 全 RAM watch_range）+ GUI 重采**（C4 新场景，~1–2min/5000f）。见 §2.1。

## 0. 方案摘要（待批准项）

| 项 | 方案 | 批准点 |
|---|---|---|
| A. 四联 #16/#19/#20/#21 + #17/#18 | **bytes 不重写**；登记 `expected_source=stg08-timeline:<帧>:<tag>`、sha 逐字节核验；status：主帧 `ok`、四联非主帧 `documented-duplicate`（duplicate_of=l4_stg06-late）；**trigger 由 natural 更正为 injected**（stg08-timeline 含 `poke $26/$0400`） | ✅ 需批 |
| B. ending-4/5 | bytes 保留；`expected_source` 更新为 C1 实际 shot tag（`tas-stg08b2-direct:34500:probe` / `:36000:probe`，原 `l4f34500` 标签不存在）；status 保持 `documented-duplicate`、trigger=assisted | ✅ 需批 |
| C. #14 `l4_stg02-start` | **替换 expected bytes**：`788d6984…`（258B boot-black 复制品）→ `44e7800d…`（1411B 真实"地区-2"开始屏，源 `gs5-anchor-render:3450:stg02-start`）；旧件归档；status→`ok`、duplicate_of 清空、reuse→independent、trigger=injected | ✅ 需批 |
| D. #14 pair 定义 | **Lead 已裁定 Option B**：pair 重指向 **`gs5-anchor-render:3450`**（expected=`44e7800d…` 真「地区-2」屏；actual=gs5-anchor trace f3450 **重渲**）；**前置 blocker**：需 render-ready overlay+GUI 重采（现有 gs5-anchor trace 无 probe/RAM blob）→ §2.1；A（错帧比较）已否决，C 后置 | ✅ 已裁定；blocker 待解 |
| E. 工具依赖 | `tools/l4_render_all.py` 的 **PAIR_SPEC 需同步 9 行**（status/capture_tag/trigger/expected_source）；由 platform-porter 出小卡或随本安装卡执行（见 §5） | ✅ 需批 |
| F. 4 blocked | 不动（`blocked:no-source` 终态；expected_sha256 空） | — |
| G. 前置采集 | **`gs5-anchor-render`（C4 overlay：输入不变+补 probe_mem/全 RAM watch_range/capture tag）+ GUI 重采 5000f → phase-batch → render f3450 ×2**；需 Lead 批准设计+owner（我或 platform-porter） | ✅ 需批 |

## 1. 安装清单表（22 行；登记=bytes 不写，替换=写文件）

| # | expected 文件 | 旧 sha（12） | 动作 | 新 sha（12） | 来源（scene:frame:tag） | trigger | status（安装后） | reuse | duplicate_of |
|---:|---|---|---|---|---|---|---|---|---|
| 1 | gameplay-stage1_f01260_stage-start.png | `38365db2cb4f` | 不动 | 同 | gameplay-stage1:1260:stage-start | natural | ok | independent | — |
| 2 | gameplay-stage1_f01500_mid-combat.png | `7f4ac8a23511` | 不动 | 同 | gameplay-stage1:1500:mid-combat | natural | ok | independent | — |
| 3 | gameplay-stage1_f02349_end.png | `4274a33f45f5` | 不动 | 同 | gameplay-stage1:2349:end | natural | ok | independent | — |
| 4 | l4_clear-inject.png | `5cacbe294d53` | 不动 | 同 | stg03-timeline:4600:stg02-clear-inject | injected | ok | independent | — |
| 5 | l4_combat-variety.png | `6f7fe4992e59` | 不动 | 同 | stg03-timeline:6500:stg03-combat | injected | ok | independent | — |
| 6 | l4_late-combat.png | `c0baa0f8d21b` | 不动 | 同 | stg03-timeline:12000:stg03-late | injected | ok | independent | — |
| 7 | l4_shop-window.png | `988809e0b414` | 不动 | 同 | stg03-timeline:5400:probe | injected | ok | independent | — |
| 8 | **l4_stg02-start.png** | `788d6984576b`（258B，frame-mismatch 黑屏） | **替换+pair 重指向** | **`44e7800dac89`（1411B）** | **gs5-anchor-render:3450:stg02-start** | injected | **ok** | **independent** | **（清空）** |
| 9 | l4_stg03-start.png | `24a4cb45c0d8` | 不动 | 同 | stg03-timeline:5700:stg03-start-approx | injected | ok | independent | — |
| 10 | **l4_stg06-late.png** | `62a8ef06c4b9` | **登记** | 同 | **stg08-timeline:22000:stg06-late** | **injected** | **ok** | independent | — |
| 11 | l4_stg07-combat.png | `2824cb134987` | 登记 | 同 | stg08-timeline:16000:stg07-combat | **injected** | ok | independent | — |
| 12 | l4_stg07-start.png | `ddb322ac5702` | 登记 | 同 | stg08-timeline:14600:stg07-start-approx | **injected** | ok | independent | — |
| 13 | **l4_stg08-combat.png** | `62a8ef06c4b9` | 登记 | 同 | **stg08-timeline:18000:stg08-combat** | **injected** | **documented-duplicate** | reuse | **l4_stg06-late** |
| 14 | **l4_stg08-late.png** | `62a8ef06c4b9` | 登记 | 同 | **stg08-timeline:28000:stg08-late** | **injected** | **documented-duplicate** | reuse | **l4_stg06-late** |
| 15 | **l4_stg08-start.png** | `62a8ef06c4b9` | 登记 | 同 | **stg08-timeline:16600:stg08-start-approx** | **injected** | **documented-duplicate** | reuse | **l4_stg06-late** |
| 16 | l4_ending-4.png | `6ff345dc5c3f` | 登记 | 同 | tas-stg08b2-direct:34500:probe | assisted | documented-duplicate | reuse | l4_ending-5 |
| 17 | l4_ending-5.png | `6ff345dc5c3f` | 登记 | 同 | tas-stg08b2-direct:36000:probe | assisted | documented-duplicate | reuse | — |
| 18 | l4_fade-clear.png | `14802bcf460a` | **不安装** | — | —（4 blob 双网格零命中） | — | blocked:no-source | — | — |
| 19 | l4_ending-1.png | `19b88b1ffce9` | **不安装** | — | — | — | blocked:no-source | — | — |
| 20 | l4_ending-2.png | `0f8e11694d53` | **不安装** | — | — | — | blocked:no-source | — | — |
| 21 | l4_ending-3.png | `f319f1fe5256` | **不安装** | — | — | — | blocked:no-source | — | — |
| 22 | title-menu_f00180_boot-black.png | `788d6984576b` | 不动 | 同 | title-menu:180:boot-black | natural | ok | independent | — |

> 注：表中"#"为手册 canonical pair#（非 earlier readjudication 行序）；#16 四联主帧；#17/#18 对应 l4_stg07-*。
> 登记行为**零文件写入**（sha 不变）；替换仅 #14 一个文件（258B→1411B）。
> **#14 pair 定位变更**：`scene:frame` 由 `stg03-timeline:3404` → **`gs5-anchor-render:3450`**（Option B；actual 需按 §2.1 重渲后更新 g_run1/g_run2 的 `l4_stg02-start.png`）。
> `expected_source` 行必须与 GUI 实际 shot tag 对齐（四联旧 mapping 的 `l4f22000` 等 tag 不存在于 GUI shots；ending-4/5 旧 `l4f34500` 同理）。

## 2. #14 pair 定义：Lead 裁定 Option B（同链同源）+ 前置 render-ready 采集

> **【dated 更正】** 旧 expected `788d6984…`(258B) 的定性由「boot-black 复制品/invalid」更正为 **frame-mismatch（错帧覆盖，非伪造）**：
> `stg03-timeline:3404` 在原作与 GUI 重采中**都是过场黑屏**（`788d6984…`，与 title boot-black 同 blob）——旧 expected 并非伪造，而是**覆盖了错误帧**（真 STG-02 开始屏在 f3450，见 `p0-0-groupD-s1-source.md` §3）；历史结论保留。

**Lead 裁定（2026-10-19）**：#14 pair 采用 **Option B**——**重指向 `gs5-anchor-render:3450`**：
- expected = `44e7800d…`（1411B，真「地区-2」屏，来自 `shots/gs5-stg01-clear-anchor_f03450_stg02-start.png`）；
- actual = 用 **gs5-anchor GUI trace f3450 重渲**（同链同源，L4 对拍必须同逻辑屏；Option A 的 stg03 f3404 vs 原作 f3450 属错帧比较，已否决）。

### 2.1 前置 blocker 与解法（render-ready 采集）
- **现状**：`analysis/traces/gs5-anchor-render.frames.jsonl` 只有 watch 标量（`stage_no/flags26/...`），**无 zp/iram/oam/shot_objs 等 RAM blob**；`analysis/traces/` 下**无该场景 `.probe.jsonl`**（scenario 无 `probe_mem`）⇒ `render-frame` 的帧状态/PPU 写流输入缺失，**无法渲出可与 expected 对拍的 actual**。
- **解法（C4，先设计批准）**：新建 `oracle/scenarios/gs5-anchor-render.txt` = `gs5-anchor-render` 的 **render-ready overlay**：
  - 输入/assist poke/watch **逐字节不变**；
  - 增加 **RAM blob watch_range**（对齐渲染场景：`watch_range 0000-00ff zp` / `0400-06ff shot_objs` / `0200-02ff oam` / `0130-016f bg_nmi`）；
  - 增加 **probe_mem**（对齐 gameplay/stg03 已用的 $2F/$30/$8E/$8F/$8D/$130-160/$D3-DA/$161-167 等，至少覆盖渲染器需要的写流）；
  - 增加 `capture 3450 stg02-start`（保留验收 shot）；
  - GUI 重采 5000f（speed=1000，约 **1–2min**）→ phase-batch 生成 text → **render-frame 3450 ×2**（隔离 out-dir，核两跑 png/idx 确定性）→ 新 actual 与 expected `44e7800d` 做 0 容差 diff（数字执行时如实记录）。
- **owner**：由 Lead 指定（我或 platform-porter；platform-porter 已负责 task-166 PAIR_SPEC 同步）。**未被指定/未批准前不采集、不改 tracked expected。**

### 2.2 被否决/后置的选项（留档）
- ~~选项 A（pair 定位不变、expected 跨源）~~：**否决**——stg03 f3404 是错帧，跨源 expected 与错帧 actual 对拍无 L4 语义（3943 数字无效）。
- **选项 C**（stg03 自身真帧）：需 Mesen 新 overlay 在 stg03 找真实开始帧+改 pair frame/规则；成本与 B 相当且无「同链同源」优势；冻结后另卡。

**结论**：执行 **Option B**（Lead 裁定）；前置 §2.1 render-ready 采集为**开工条件**。

## 3. §7.7 安装后核验流程（执行阶段命令；不跑 gates visual）

```powershell
# 0) 前置（render-ready 采集成功后）：重渲 actual 到 g_run1/g_run2 并核两跑
#    （gs5-anchor-render GUI 采集 → phase-batch → render-frame 3450 ×2；见 §2.1）
#    render-frame 3450 l4_stg02-start gs5-anchor-render --out-dir target/p0-0/render/g_run1 ...（run2 同）
#    run1/run2 png+idx sha 必须相同（否则 BLOCKED）

# 0b) staging（gitignored）：把真实开始屏字节复制到 expected-new 暂存区
mkdir analysis/expected-new -Force
Copy-Item shots/gs5-stg01-clear-anchor_f03450_stg02-start.png analysis/expected-new/l4_stg02-start.png
#   校验 staging sha = 44e7800dac894205…（1411B）

# 1) 归档旧件（tracked 目录内 .archive/<STAMP>/）
$stamp = Get-Date -Format yyyyMMdd-HHmmss
mkdir oracle/screens/expected/.archive/$stamp -Force
Copy-Item oracle/screens/expected/l4_stg02-start.png oracle/screens/expected/.archive/$stamp/l4_stg02-start.png
#   记旧 sha 788d6984576bb134…（258B）

# 2) 安装新件（字节级复制，保持 44e7800d…）
Copy-Item analysis/expected-new/l4_stg02-start.png oracle/screens/expected/l4_stg02-start.png

# 3) mapping-decisions（只写变化行；**禁改 name/scene/frame**；status 用 ALLOWED 集）
#    Option B 的 #14 scene/frame 变更必须由 **task-166 的 PAIR_SPEC**（工具层）改，不能在 mapping-decisions 里改；
#    工具重生成 mapping/manifest 后，本步骤只做 status/trigger/source 等行级裁定（或全部由 PAIR_SPEC 一次生成）。
#    → analysis/repro/mapping-decisions.json（内容见 §4.2）
python tools/l4_render_all.py ...（待 PAIR_SPEC 同步后生成 mapping/manifest；或按 §7.7 脚本写回 mapping.tsv）

# 4) expected_sha256 全等核验（B1；只读脚本）
python -X utf8 - <<'PY'
import hashlib, json, pathlib
m=json.loads(pathlib.Path("target/p0-0/render/g-manifest-full.json").read_text(encoding="utf-8"))
E=pathlib.Path("oracle/screens/expected")
sha=lambda p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
bad=[]
for p in m["pairs"]:
    if not p.get("expected_sha256"): continue   # 4 blocked 无资产
    if sha(E/p["expected_file"])!=p["expected_sha256"]: bad.append(p["name"])
print("EXPECTED-SHA-MISMATCH:", bad or "NONE")
PY

# 5) diff 重算（#14 用新 expected；0 容差）
python -X utf8 - <<'PY'
import sys, pathlib, json; sys.path.insert(0,"tools"); from diff_visual import compare
dp,err=compare("oracle/screens/expected/l4_stg02-start.png",
               "target/p0-0/render/g_run1/l4_stg02-start.png")
print("#14 diff:", len(dp) if dp is not None else err)
PY
#   期望：Option B 新 actual vs 新 expected 的 diff（执行时如实记录；旧 A 口径 3943 仅历史，不再作为验收数字）

# 6) expected_unique / registry 重算（l4_render_all --validate）
python tools/l4_render_all.py --validate target/p0-0/render/g-manifest-full.json
#   期望：expected_unique=18（22−4；ending4/5+四联两重复组）、actual_unique=14（rendered18−4）、registry_ok=true
```

**验证判据**：①全部 present expected sha 与 manifest 相等；②#14 sha=44e7800d、字节 1411；③old 件已归档且 sha 可查；④`--validate` PASS（schema/registry/两跑）；⑤manifest 22 行状态与 §1 表一致；⑥docs-dated 更新完成（readjudication/手册 §7.2/§7.3）。**strict/visual 不在本卡执行**（§8.1 步骤 5 由冻结卡按 §B3 统一采集记录）。

## 4. 影响面预演（执行前已算）

### 4.1 数字变化
| 项 | 安装前 | 安装后 | 说明 |
|---|---|---|---|
| #14 expected sha/字节 | `788d6984…`/258B | **`44e7800d…`/1411B** | 唯一文件写入 |
| #14 diff（同链 actual vs expected） | 旧 2495（错帧/旧 actual）；A 口径 3943（已否决） | **执行时实测（gs5-anchor f3450 新 actual vs `44e7800d`）** | Option B 同链同源；若 actual 仍白底条纹= P0-1 渲染缺陷 |
| expected_unique（全 22） | **17** | **18** | stg02↔title 复制组拆开（−1 重复组） |
| expected_unique（rendered 18） | **13** | **14** | 同上 |
| actual_unique（rendered 18） | 14 | 14 | 两跑/去重不变 |
| registry 公式 | expected 13=18−5?（旧） | **expected 14=18−4；actual 14=18−4** | 新 documented_duplicates：ending4/5（2）、四联（4） |
| 22 对总数 | 22 | 22 | Option B 下 pair#14 定位改 gs5-anchor:3450（scene/frame 变） |
| pair#14 scene/frame | stg03-timeline:3404 | **gs5-anchor-render:3450** | 需 §7.7 规则更新（Lead 已裁定）、PAIR_SPEC 同步（task-166） |
| g_run1/g_run2 `l4_stg02-start.png` | 旧 actual `ebc6372f…`（stg03 f3404） | **新 actual（gs5-anchor f3450 重渲，两跑待测）** | 基线 actual 变更需 manifest/readjudication 重算 |

### 4.2 mapping-decisions.json（执行阶段写入；9 行变化，禁改定位列）

> **Option B 定位变更例外**：`l4_stg02-start` 的 `scene/frame` 改 `gs5-anchor-render`/`3450` **不在 mapping-decisions**（禁改定位列），而由 **task-166 PAIR_SPEC** 改；mapping-decisions/工具输出里该行 scene/frame 以 PAIR_SPEC 为准。
```json
{
  "l4_stg02-start": {"status":"ok","reuse":"independent","duplicate_of":"","expected_source":"gs5-anchor-render:3450:stg02-start","capture_tag":"stg02-start","trigger":"injected"},
  "l4_stg06-late":  {"status":"ok","duplicate_of":"","expected_source":"stg08-timeline:22000:stg06-late","capture_tag":"stg06-late","trigger":"injected"},
  "l4_stg08-start": {"status":"documented-duplicate","duplicate_of":"l4_stg06-late","expected_source":"stg08-timeline:16600:stg08-start-approx","capture_tag":"stg08-start-approx","trigger":"injected"},
  "l4_stg08-combat":{"status":"documented-duplicate","duplicate_of":"l4_stg06-late","expected_source":"stg08-timeline:18000:stg08-combat","capture_tag":"stg08-combat","trigger":"injected"},
  "l4_stg08-late":  {"status":"documented-duplicate","duplicate_of":"l4_stg06-late","expected_source":"stg08-timeline:28000:stg08-late","capture_tag":"stg08-late","trigger":"injected"},
  "l4_stg07-start": {"status":"ok","expected_source":"stg08-timeline:14600:stg07-start-approx","capture_tag":"stg07-start-approx","trigger":"injected"},
  "l4_stg07-combat":{"status":"ok","expected_source":"stg08-timeline:16000:stg07-combat","capture_tag":"stg07-combat","trigger":"injected"},
  "l4_ending-4":    {"status":"documented-duplicate","duplicate_of":"l4_ending-5","expected_source":"tas-stg08b2-direct:34500:probe","capture_tag":"probe","trigger":"assisted"},
  "l4_ending-5":    {"status":"documented-duplicate","expected_source":"tas-stg08b2-direct:36000:probe","capture_tag":"probe","trigger":"assisted"}
}
```
> 四联/ending 的 `expected_sha256` 不变；#14 新 sha 由 manifest 生成时重算。

### 4.3 工具/文档同步（依赖）
1. **`tools/l4_render_all.py` PAIR_SPEC 9 行**（#14 + 四联 4 + #17/#18 + ending-4/5）需按 §4.2 更新（否则工具重生成会覆盖 mapping 回旧值）——由 platform-porter 小卡（owner 已知）或随本安装卡执行；
2. `docs/findings/p0-0-readjudication-gui.md` dated 追加：#14 diff 2495→3943、状态四联/#17/#18 转 ok/source、expected_unique 13→14；
3. 手册 §7.2/§7.3 dated 注：四联 expected 来源恢复（bytes 不变、登记来源）；#14 替换完成；4 blocked 终态（dense 负证据）；
4. `p0-0-groupD-s1-source.md` §2/§3 已记来源；安装后指向本方案执行记录。

## 5. 风险与回退
- **前置风险**：Option B 依赖 §2.1 render-ready 采集（`gs5-anchor-render`）——未采集/未批准则 actual 无法重渲，安装不能开工；
- **tracked 文件变更** = `oracle/screens/expected/l4_stg02-start.png`（+旧件归档）以及 g_run1/g_run2 actual（gitignored）重渲；任何异常可 `Copy-Item .archive/<STAMP>/l4_stg02-start.png expected/l4_stg02-start.png` 或 `git checkout -- oracle/screens/expected/` 回退；
- mapping/manifest 均 gitignored 或可再生（`l4_render_all.py` 重生成）；不触 threshold/gates；
- 若复核认为选项 A 的跨源 `expected_source` 不可接受 ⇒ 回退安装、转选项 B/C 另卡（不影响其它 expected）。

## 6. 复算/核验证据指针
- 来源证据：`p0-0-groupD-s1-source.md` §2（四联/#17/#18 逐字节表）、§3（#14 定位与图像）、`shots/gs5-stg01-clear-anchor_f03450_stg02-start.png`（44e7800d/1411B）；
- current actual：`target/p0-0/render/g_run1/l4_stg02-start.png`（ebc6372f，diff 2495/3943）；
- 工具：`tools/l4_render_all.py`（PAIR_SPEC L58+、validate/registry）、`tools/diff_visual.compare`（只读）；
- 期望 unique 复算命令见 §3 步骤 6。

## 7. 验收自查（方案稿）
- [x] 安装清单 22 行（动作/旧新 sha/来源/trigger/status/reuse/duplicate_of）与 §7.7 规则自洽（duplicate_of 仅 documented-duplicate；status 在 ALLOWED 集）；
- [x] #14 替换影响预演（sha/字节/diff 2495→3943/expected_unique 17→18、13→14）；
- [x] #14 pair 定义两选项（A 推荐/B 后置）+ C 备选，说明 §7.7 兼容性；
- [x] 核验流程（sha 全等/manifest/diff/registry）与回退方案；
- [x] 工具 PAIR_SPEC 同步依赖已列出（platform-porter）；
- [ ] **Lead 批准 §2.1 render-ready overlay 设计/owner** → 采集+重渲 → 执行阶段：改 expected（单独 commit+Evidence）+ mapping/manifest + dated 更新 + 独立复核；
- [ ] docs-links PASS（提交前复核）。

## 8. 执行记录（2026-10-19/20 完成；dated）

> 本 section 为**已执行事实**，覆盖上文 §2/§3/§4 中"待执行/TBD"表述（历史方案文本保留）。

**前置 overlay（C4，commit `86e18d4`）**：`oracle/scenarios/gs5-anchor-render.txt`（Lead 批准/commit；输入/poke/原 watch 43 行逐字节同）。
- GUI 采集：`gs5-anchor-render` 5000f @speed=1000 → **valid**（`done.frames_actual=5000`、`frame_errors=0`、held=1094、trace_lines=5000）；
  trace sha `b628ad3ef6be16813c630184c561dfbf485e66d5d2d4ffe8f60c96debffc73bd`（**与旧 gs5-anchor trace `afd2fc1f…` 不同=观测面新增的预期差异**，非行为差异：输入行/assist 不变）；
  probe `7db8a474628f…`（23,606,517B）、frames `b628ad3ef6be…`（15,000,838B）。
- phase-batch：`gs5-anchor-render-text.jsonl` 81,322,458B sha `fbe7c209279d97f9…`。
- render f3450 ×2（g_run1/g_run2）：png **`27fa373733534632…`**、idx **`436120b680f7c3c2…`**，**两跑确定性 PASS**（wall 27.0/27.2s；`early_exit=false`、in_frames_read=3080、text_records_read=1588037）。
- **新 actual vs expected `44e7800d`：diff=2352 px，ratio=0.038281，bbox=(4,9,249,239)**（0 容差口径；actual 仍白底条纹/文字可见，属 P0-1 渲染缺陷族，非 expected 问题）。
  历史：旧 expected `788d` vs 旧 actual=2495；Option A 的 3943 已否决、不作验收。

**安装（§7.7 执行）**：
- old expected 归档：`oracle/screens/expected/.archive/20261020-000358/l4_stg02-start.png`（258B `788d6984…`，frame-mismatch 原件保留）；
- 新 expected 安装：`oracle/screens/expected/l4_stg02-start.png` = **1411B `44e7800dac894205…`**（字节级复制自 `analysis/expected-new/l4_stg02-start.png`，staging 校验同 sha）；
- pair 重指向：`scene=gs5-anchor-render, frame=3450`；manifest/mapping 由 `tools/l4_render_all.py` 重生成；
- **g-manifest-v2.json #14 entry 已更新**（pair 重指向后旧 run1 sha 交叉核对会误报；备份 `g-manifest-v2.json.pre-task165.bak`；target/ gitignored），其余 21 entry 未动。

**核验结果（工具真实输出）**：
```
MANIFEST pairs=22 rendered=18 blocked=4 replaced=0 documented_duplicate=5
TWO-RUN: PASS (18/18 png+idx)
DEDUPE: expected 14=18-4 actual 14=18-4 registry_ok=True documented_ok=True -> PASS
MAPPING: analysis/repro/mapping.tsv
VALIDATION: PASS
```
- `expected_sha256` 全等：present 18 / mismatch **NONE**（expected_file 经 v2 映射）；expected full 22 files → **unique 18**；rendered expected_unique **14**、actual_unique **14**。
- mapping 22 行：`ok=13`、`documented-duplicate=5`、`blocked:no-source=4`；#14 row=`scene=gs5-anchor-render, frame=3450, trigger=injected, expected_source=gs5-anchor-render:3450:stg02-start, status=ok, reuse=independent, duplicate_of=""`。
- 四联/#17/#18：bytes 不变；`trigger natural→injected`；四联主帧 `l4_stg06-late` status=ok、其余 3=documented-duplicate；#17/#18 status=ok；ending-4/5 tag=`probe`、assisted、documented-duplicate。
- 4 blocked 不安装（expected_sha256 空）；`expected_matches` 仍 0/18（L4 0 容差未达成，本卡只解决 expected 来源/可复现性）。

**遗留**：独立对抗复核 pass → tracked expected 变更 commit（含 `.archive/` 旧件）+ push；4 blocked 终态；strict/visual 由 §8.1 冻结卡统一采集记录。
