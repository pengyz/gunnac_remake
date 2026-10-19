# P0-1 族①第二迭代·事件序回放（$0160 置位时刻单槽快照）实施报告

> 2026-10-20 03:4x（结果补齐 03:35）| 实施 agent（渲染核心）。范围=**只改
> [crates/game-desktop/src/bin/render_frame.rs](../../crates/game-desktop/src/bin/render_frame.rs)（默认路 + 单测）+ 本文档**；
> 未动 oracle/gates/tools/expected/threshold；**未 git 提交**（Lead 复核后提交）。
> 语义输入=三份定谳：[l5-t93-field-mapping.md](l5-t93-field-mapping.md)（A）、
> [l5-0160-batch-semantics.md](l5-0160-batch-semantics.md)（含 dated 更正）、
> [probe-setter-blindspot-fix.md](probe-setter-blindspot-fix.md)（双流 probe 已含真写流）。
> 证据等级标注：A=硬证据（渲染产物 sha/逐像素复算/测试输出/计数复算），B=静态推理。**区分"已证实/假设"**。

## 0. 判定摘要

1. **事件序回放已落地（A）**：probe 事件按文件序（=时间序）逐条回放；写事件镜像进
   buf/attrs/regs/pal；`$0160` 写事件（`op=w` 且值非 0）⇒ **当刻**取 buf/attrs/regs 快照入
   **单 pending 槽**（多置位=覆盖单槽）；**NMI 消费点=帧边界**（帧首）上传槽内容并清空；
   **无置位的帧不上传**。"帧末单快照"主路语义已按 l5-0160 §5 建议 2 撤回。
2. **op=r 读流隔离（A）**：双流 probe 的读事件（自旋轮询 149K+/场景）不再污染镜像
   （旧实现把读流当写镜像）。缺 op 旧 probe（无 op 字段）**按旧行为**（全事件镜像+帧末单快照）
   并**显式标注**（stderr WARN + 渲染日志 `replay_mode=legacy-opless-probe`）。
3. **上传几何走 geo 档（不变）**：MODELGEO=1 启用模型镜像（geo=1 裸基+$0F / geo=2 LUT 精确行基
   分组不加 $0F）；默认=legacy 32 连写。MODELGEO **新增进 ENV_KNOBS 记录**（模式归属可证，
   补 p01 A/B 只能靠 nt_nz 反推的缺口）。
4. **验证环（A，§3）**：三样本 × {legacy, MODELGEO=1} × 2 跑全部完成；**before/after 同输入可比**。
   ⚠️ **测量混淆项（A，务必按此读表）**：p0-1 历史基线用的是**旧读流 probe**；gameplay/gs5 的
   probe 已被 blindspot-fix 重采为双流（sha 已变）⇒ **改动前 binary 在当前输入上 ≠ 历史基线**
   （gameplay 3,742→3,247、stg02 2,352→2,355）；stg03 probe 未重采 ⇒ **逐字节复现历史**
   （out sha `246dd5712608` 与 p01-legacy s3.log 全同）。表中双列并陈。
   **实测结论文本（A）**：事件序回放**非单调**——gs5/stg02 **显著转优**（legacy 2,355→**1,524**、
   geo 2,440→**1,524**），gameplay **回退**（legacy 3,247→**5,202**、geo 3,508→**5,292**），
   stg03（旧 probe）**逐字节不变**（58,159/58,152）。三样本总和 legacy 63,761→**64,885（+1,124）**、
   geo 64,100→**64,968（+868）** ⇒ **净负**；按纪律**如实在账、不调阈值/不白名单**，默认 legacy 保持。
   回退假设（B，未证）：gameplay 置位仅 v=3（70 次，调色板挂起型）而 gs5 应用窗为 v=3（94 次）——
   v 分型路由未接（§6-2），v=3 触发行上传快照在 HUD/属性行可能错配；待 v 分型（G4/G5）与
   gameplay 专用取证后复测。
5. **测试（A）**：`cargo test -p game-desktop` **61 项全绿**（52 基线 + 9 新增槽语义单测：
   多置位并入单槽 / 无置位不上传 / 快照点=置位时刻 / 读事件不改状态 / 缺 op 帧末回退 /
   无置位帧不补帧末快照 / geo 档选择 / MODELGEO 记录）。
6. **两跑确定性（A，§4）**：同配置 2 次渲染 png sha256 逐对全同。

## 1. 语义映射表（定谳 → 实现）

| # | 定谳结论 | 出处 | 实现落点 |
|---|---|---|---|
| 1 | `$0160`=**单 pending 槽位域 + NMI 单点消费**（置位:消费=70:70:70 严格 1:1，非脉冲队列） | l5-0160 §0-3/§5 判定 4-6；probe-setter §3.2（写侧直证） | `PendingSlot`（单槽）；置位=`op=w 且 v≠0`；消费=帧边界 `consume()` 上传+清空 |
| 2 | **"帧末单快照"已撤回**：帧内可 ~15 次 32B 填充（帧 1205/1206 各 488 条）；应按"填充→置位→下一 NMI 消费"事件序回放 | l5-0160 §5 判定 9/建议 2（dated） | 快照点=**`$0160` 置位时刻**（`EventReplay::apply` 的 `ApplySet` 分支）；帧末不再取样 |
| 3 | 多置位**并入单槽**（同帧多次置位≠多批，NMI ≤1 消费/帧） | l5-0160 §5 判定 6 | `PendingSlot::set` 覆盖单槽（后置位胜）；单测 `slot_multi_set_merges_single_pending` |
| 4 | 无置位=无更新（pending==0） | l5-0160 §5 建议 1（单槽 pending 位域） | 槽空 `consume()` ⇒ None ⇒ 不上传；单测 `slot_no_set_no_upload`/`op_frame_without_set_does_not_upload` |
| 5 | NMI 消费点=帧边界（`F0DC` 轮询 1 次/帧；帧 N 置位→帧 N+1 NMI 见挂起并消费） | l5-0160 §2/§4（帧 181/182 序列） | 帧首 `replay.slot.consume()`（现有边界不变）——帧 N 置位快照在帧 N+1 上传 |
| 6 | probe 事件=写读混合，**op 消歧**（`"op":"w"/"r"`；写事件值=存入值、读事件值=读到值） | probe-setter §2/§5.3 | 解析保留 op；`op=r` 跳过（不改状态）；`op=w` 镜像。单测 `read_events_do_not_mutate_state` |
| 7 | 行缓冲 32B=4 行×8 列 tile 块；上传几何=4 组×8B 步进 $20；attr 行=(($8F\|3):$90)+$0F | l5-t93 §2/§3-1（A 级） | MODELGEO geo 档（既有 p0-1 迭代2 实现不动）：geo=1 裸基+$0F / geo=2 LUT 精确行基 |
| 8 | 记录 4 字段→(x,x+1,x+16,x+17)；buf 直接镜像写序=等价 | l5-t93 §3-2 | 不改（镜像填充照旧） |
| 9 | 兼容旧 probe（无 op 字段）：缺 op 按旧行为（帧末快照）**并显式标注** | 任务边界 | `legacy_frame_wrote`/`EventReplay::end_frame` 帧末回退；stderr WARN + `replay_mode=legacy-opless-probe` + `legacy_snap_frames` 计数；单测 `legacy_opless_frame_end_fallback` |

## 2. 实现要点（render_frame.rs 默认路）

- **`PendingSlot<RowSnap>`**：单 pending 槽（`set`/`consume`/`peek`；`sets`/`consumes` 计数）。
- **`EventReplay`**：回放状态机（buf 0x31B 行缓冲镜像 / attrs $D3-$DA / regs $8E-$90 /
  pal $161-$167 / $8D LUT 标记）+ 帧旗标（`frame_set`/`frame_legacy`）+ 诊断计数。
  - `apply(a,v,op)`：读事件跳过；写/缺 op 事件镜像；`$0160` 置位写 ⇒ **当刻** `snapshot()` 入槽；
  - `end_frame(wrote)`：**仅缺 op 旧 probe**（`legacy_file` 或帧内有缺 op 触发事件）走帧末单快照
    回退（旧行为，显式标注）；op 齐全帧不补帧末快照。
- **`build_row_snap`**：行快照构造（ROWWIN 行窗 / HUD LUT 回退 / geo 档选择）——置位时刻与帧末
  回退共用同一构造，保证两路几何口径一致。
- **上传路（帧首 consume）完全不变**：geo=0 legacy 32 连写 / geo=1 base+=$0F 4×8 分组+$20 /
  geo=2 分组不加 $0F / attr 行 8B。
- **诊断**：DBGUPLOAD 行增 `replay(sets/consumes/legacy_frames/reads_skipped/opless)`；
  渲染日志行尾增 `replay_mode= event-replay|legacy-opless-probe`、`replay_sets=`、
  `legacy_snap_frames=`、`probe_opless=`、`probe_reads_skipped=`（旧字段前缀不变）。
- **`ENV_KNOBS` 增补 `MODELGEO`**（env 摘要按 K 排序不变；单测 `modelgeo_knob_is_recorded`）。

## 3. 验证环（三样本 × {legacy, MODELGEO=1}，`tools/diff_visual.compare` 计 len(samples)）

口径：expected=`oracle/screens/expected/{l4_stg02-start, gameplay-stage1_f01260_stage-start,
l4_clear-inject}.png`；渲染=`render-frame 3450 stg02-start gs5-anchor-render` /
`1260 gameplay-stage1_f01260 gameplay-stage1` / `4600 l4_clear-inject stg03-timeline`，
`REPO_ROOT` 显式；legacy=env {}，geo=`MODELGEO=1`。
**列定义**：历史基线=p0-1 迭代报告数（**旧读流 probe** 输入）；before-current=**改动前 binary ×
当前输入**（同输入可比口径）；after=改动后 binary × 当前输入（事件序回放）。

| 样本 | 配置 | 历史基线(旧 probe) | before-current(同输入) | after（事件序回放） | Δ(after−before-current) |
|---|---|---:|---:|---:|---:|
| gs5-anchor-render f3450（stg02-start） | legacy | 2,352 | 2,355 | **1,524** | **−831** |
| gameplay-stage1 f1260 | legacy | 3,742 | 3,247 | **5,202** | **+1,955（回退）** |
| stg03-timeline f4600（clear-inject） | legacy | 58,159 | 58,159 | **58,159** | **0（逐字节同）** |
| gs5-anchor-render f3450 | MODELGEO=1 | 2,644(变量①)/2,476(①') | 2,440 | **1,524** | **−916** |
| gameplay-stage1 f1260 | MODELGEO=1 | 3,386(变量①)/3,520(①') | 3,508 | **5,292** | **+1,784（回退）** |
| stg03-timeline f4600 | MODELGEO=1 | 58,148(变量①)/58,152(①') | 58,152 | **58,152** | **0（逐字节同）** |

**测量混淆项（A）**：
- stg03（probe=b71ee8fcbb84 未重采，598,571 事件**全缺 op**）：before-current 与历史基线
  **逐字节全同**（legacy out sha `246dd5712608`=p01-legacy s3.log `out=246dd5712608`；
  MODELGEO=1 out `532936218b01`/58,152=历史变量①' 口径）⇒ 该样本上"缺 op 按旧行为"=回归核，
  after 亦应逐字节不变。
- gameplay/gs5（probe 已重采为双流：7fd1643cfaaf / 935678c325c9）：改动前 binary 在新 probe 上
  把读+写双流都镜像 ⇒ 与旧读流输入的历史基线不同（gameplay 3,742→3,247、stg02 2,352→2,355）。
  **after 只与 before-current 同输入可比**；历史列仅供参照，不作 Δ 基准。
- 置位数交叉对账（A，独立 python 复算）：gameplay `replay_sets`=**70**（v=3×70，=probe-setter §3.2 70:70:70）；
  gs5 **应用帧窗内** `replay_sets`=**94**（v=3×94；目标帧 f3450 之前）——全文件 v 分型为 3×94 + 1×197=291，
  v=1 的 197 次落在 f3450 之后未消费（渲染目标帧边界口径，§6-7）；stg03 `replay_mode=legacy-opless-probe`、
  `legacy_snap_frames=1503`、`replay_sets=0`、`probe_opless=379865`（读流旧 probe 回退，逐字节回归同 §3-混淆项）。
- **置位→消费逐拍结构（A，`set_seq.py`）**：gameplay 179 条 `$0160` 写事件序列=严格交替
  「帧 N 置位(v=3) → 帧 N+1 `F14E`=0（NMI 消费清零）」；38 条非 NMI v=0 写（900D/904E/ECD3/827D）
  全部落在**无挂起**帧（idle 清零），本 trace 无一取消待消费槽。
- **观察（A）/解释候选（B）**：gs5 after 的 legacy 与 geo **输出逐字节同**（`245cbedc4398`，
  nt_nz 355/359）——94 次置位快照疑全走 HUD LUT 回退（置位时刻 `$8E/$8F`<`$2000` ⇒ geo 强制
  legacy 形），B 级待逐拍取证；gameplay 两档不同（`51b7a201f021`/`efc5fb2dbc33`）⇒ 存在非回退快照。

## 4. 两跑确定性（同配置跑 2 次，png sha256 对比）

| 样本 | 配置 | run1 = run2 sha256 | 判定 |
|---|---|---|---|
| stg02-start | legacy | `245cbedc4398d24c1582be12394545550a9f4a2766ff318c70d30e58a84a490d` | **同** |
| gameplay-stage1_f01260 | legacy | `51b7a201f021aacaa30658c74b70982f84282a5808dadd0ec3975a0d1d870ff0` | **同** |
| l4_clear-inject | legacy | `246dd5712608160e5c81447ff540b58ba023bb8e8c6ada3ce721907278b83a0b` | **同** |
| stg02-start | MODELGEO=1 | `245cbedc4398d24c1582be12394545550a9f4a2766ff318c70d30e58a84a490d` | **同** |
| gameplay-stage1_f01260 | MODELGEO=1 | `efc5fb2dbc335f621d2d1e0b330559cc3e1849718996643a657b884e630c355e` | **同** |
| l4_clear-inject | MODELGEO=1 | `532936218b01144ebf06500075a6e69113a7317be0b515f9f0f4ced81c94bf2b` | **同** |

（`python target\p0-1\determinism.py` 复算；6/6 对 r1≡r2 全值 sha256 相同 ⇒ **两跑确定性成立**。）

## 5. 测试（cargo test -p game-desktop，61 项全绿）

- 基线 52 项不动全绿（cli_tests 14 + render/render_rules/smoke 33 + lib 5）。
- 新增 9 项（`event_replay_tests`，render_frame.rs 内）：
  `slot_multi_set_merges_single_pending`（多置位并入单槽）、`slot_no_set_no_upload`（无置位不上传）、
  `snapshot_taken_at_set_moment`（快照点=置位时刻）、`classify_set_requires_op_w_and_nonzero`
  （置位判定=op=w 且 v≠0；v=0 清零不置位；缺 op=旧行为）、`read_events_do_not_mutate_state`、
  `legacy_opless_frame_end_fallback`（缺 op 帧末快照回退+触发谓词）、
  `op_frame_without_set_does_not_upload`、`snap_geo_modes_and_hud_fallback`（geo 档/回退）、
  `modelgeo_knob_is_recorded`。

## 6. 未证残留（明示）

1. **G3 填充↔置位↔消费逐拍链仍未闭合（B）**：置位时刻快照是按 l5-0160 §5 建议实现的模型；
   但真实硬件 NMI 上传读 buf 的时点=消费时（`$2007` 上传体逐拍），若"置位后、NMI 前"仍有
   填充写，本模型（置位时快照）与真机（消费时 buf 当时态）会有**帧内尾部填充差**——量化待 G3 闭合。
2. **v 分型路由未接**：v=3（调色板挂起型）/v=1（行完成候选）不分型，置位一律行上传快照
   （G4/G5 挂；$0161 参数组/attr 合成内容=位属性合成嫌疑项 t93 §3-② 未动）。
3. **几何变量=二阶未裁**：MODELGEO=1 沿用迭代2 变量①' 混合档（LUT→geo=2 / 裸基→geo=1）；
   逐对非单调问题仍在，本卡不锁定配置（默认 legacy 保持）。
4. **LOOPABL=1 消融 knob 行为差（登记）**：填充相位关闭时，op 齐全 probe 不再产生帧末空行上传
   （旧行为会上传空 buf）——仅影响消融诊断口径，env={} 主路无影响。
5. **旧 probe 场景仍走帧末快照**：stg03/stg08/tas-* probe 均为旧读流（缺 op），事件序回放
   需重采双流 probe 后启用（gameplay/gs5 已具备）；重采前这些场景=旧行为（本文 §3 回归核）。
6. **gameplay 样本次迭代回退（A，净负主因）**：legacy 3,247→5,202（+1,955）、geo 3,508→5,292（+1,784），
   而 gs5 同步转优（−831/−916）、stg03 逐字节同 ⇒ 事件序回放模型按 l5-0160 落地，但**未过单点验证**；
   回退假设（B，未证）=gameplay 置位全为 v=3（调色板挂起型）×70，v 分型路由未接（§6-2→G4/G5）时
   以"行上传快照"消费调色板挂起型，导致 HUD/属性行错位；需在 v 分型取证后复测，**不得据此回滚或改阈值**；
   默认 legacy 保持、事件序回放为可开关语义（旧 probe 自动回退）。
7. **probe 3M 上限未动**（blindspot-fix §6 登记）；FLAGUPLOAD 经验拟合路未动（E1 约束）。
8. 帧号边界口径不变：目标帧自身置位快照不上传（下一帧 NMI 超出目标）——与旧行为一致。
9. **`$0160` v=0 写=取消挂起？**：真机"单槽挂起位域"下 v=0 写应清 pending；本实现**不回退**
   已入槽快照。本次 trace 无一例取消（§3 逐帧序列复算：idle 清零全在无挂起帧）⇒ 对本次三样本
   无影响（A）；跨场景（bank3 BBBF 族/tas-stg08b2 轨道）未证。
10. **HUD LUT 回退与置位时刻快照的交互（B）**：gs5 两几何档输出逐字节同（疑 94 次快照全走回退，
    §3 观察）；回退路（t109 降序+双写拟合形态）本为帧末快照设计，置位时刻是否仍应触发待逐拍取证。

## 7. 复现命令

```bash
cargo test -p game-desktop                                   # 61 项全绿
cargo build -p game-desktop --bin render-frame
# 渲染（REPO_ROOT 显式；每样本×{legacy,MODELGEO=1}×2 跑）
REPO_ROOT=D:\workspace\gunnac_remake target\debug\render-frame.exe 3450 stg02-start gs5-anchor-render --out-dir target\p0-1\render\after-legacy-r1 --log-path target\p0-1\render\after-legacy-r1\s1.log
# …（见 target/p0-1/run-after.ps1：12 渲染全序列；before 侧 run-before.ps1 + render-frame-before.exe）
# 复算
python target\p0-1\compare.py        # diff_visual.compare × 6 runs × 3 samples（len(samples)）
python target\p0-1\determinism.py    # 两跑 png sha256 全值对（6/6 同）
python target\p0-1\probe_op_stats.py # op 覆盖 + $0160 置位数（对账 probe-setter §3.2）
python target\p0-1\set_seq.py gameplay-stage1 1260   # 置位/清零逐帧序列（1:1 结构）
python target\p0-1\set_v_stats.py    # 目标帧内置位 v 分型构成（70/94 全 v=3）
```

产物目录：`target/p0-1/render/{before-legacy,before-geo,after-{legacy,geo}-r{1,2}}/`；
二进制：`target/p0-1/build/{render-frame-before,render-frame-after}.exe`（同输入可比口径的双 binary）。


## 【2026-10-20 03:50 dated 收尾注】
- F4（复核）：总和数字经复核指摘后核对——现文 §3 行值与总和自洽（legacy 63,761→64,885=+1,124；
  geo 64,100→64,968=+868；Lead grep 全文零命中旧值 776/64,537）⇒ **F4 已在源头修正**；
- F5：diff 行数头更正 +480/−76（复核实测）；
- F1（latent hud_seq 快照计数）/F2（ptr_writes 吞 op=r）/F3（空 probe 口径）→ 后续小卡；
  F6（MODELGEO=1 列复渲）→ v=3 分型路由迭代时一并复测。


## 【2026-10-20 05:00 dated 原地更正（task v=3 复核 F3）】
- §3（L105 附近）「94 快照疑全走 HUD LUT 回退」**误诊更正**：置位时刻 regs 全 ≥$2000
  （set_moment_state.py 独立复跑：gs5 94 次=00:20:lut1×65/00:21×19/c0:22×10；gameplay 70 次=
  00:20:lut1×65/c0:22×5）；legacy≡geo 真因=gs5 draws=0（BG 面不绘，nametable 差不传导 PNG）。
- §6-6/§10（L32/L155 附近）**B 假设「v=3 误刷行=回退主因」已证伪**（v 分型路由实测：剔除后
  gameplay 5202→5292 恰 +90px 且全为 before 偶然修对）——真因悬空（候选=帧末快照撤回后行更新
  缺失面，见 p0-1-vtype-routing.md §7-1）；原文主张保留为历史，以本 dated 注为准。
