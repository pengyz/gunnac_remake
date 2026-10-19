# P0-1 族①第三迭代·$0160 v 分型路由（v=3 调色板挂起型 / v≠3 行上传）实施报告

> 2026-10-20 04:3x | 实施 agent（渲染核心）。范围=**只改
> [crates/game-desktop/src/bin/render_frame.rs](../../crates/game-desktop/src/bin/render_frame.rs)（默认路 + 单测）+ 本文档**；
> 未动 oracle/gates/tools/expected/threshold；**未 git 提交**（Lead 复核后提交）。
> 语义输入=[l5-0160-batch-semantics.md](l5-0160-batch-semantics.md)（含 dated 更正）、
> [p0-1-event-replay.md](p0-1-event-replay.md) §3/§6-2/§6-6（B 假设=本卡验证对象）、
> [linebuf-timing-wnew.md](linebuf-timing-wnew.md) §2.1/§3、`docs/spec/l4-visual-plan.md` R-l4-linebuf-3/4/8。
> 证据等级：A=硬证据（渲染产物 sha/逐像素复算/测试输出/计数复算/DBGUPLOAD 实测），B=静态推理。**区分"已证实/假设"**。

## 0. 判定摘要

1. **v 分型路由已落地（A）**：置位快照**记 v 值**（`SetSnap`）；NMI 消费按 v 分流——
   **v=3 ⇒ 调色板上传路**（pal 源 `$161-$167` → 渲染端调色板状态，**行快照不应用**）、
   **v≠3（含 1/其它）⇒ 行上传**（现行为）；v=0 写=清槽语义不入槽（现行为）；缺 op 旧 probe
   不分型 ⇒ **固定行路**（旧行为回归核）。路由在置位时刻定型（变体=路由真源）。
   实测对账（A，渲染日志）：gameplay `pal_sets=70/row_uploads=0`、gs5 `pal_sets=94/row_uploads=0`、
   stg03 `legacy_row_sets=1503/row_uploads=1502/pal_uploads=0`——分流 100% 按 v 定型。
2. **B 假设被证伪（A，本卡主结论）**：p0-1 §6-6 假设「gameplay 置位全为 v=3 被误当行上传
   ⇒ f1260 回退 3,247→5,202」**不成立**——剔除全部 v=3 行应用后 gameplay **5,202→5,292（+90）**，
   不降反微升；before/vtype 两输出逐像素差恰 **90 px**（`diff_visual.compare` 实测），即 v=3 误刷行的
   **净像素效应仅 90px 且方向为"偶然修对"**。3,247→5,202 的 +1,955 回退主因另有其因（§7-1 候选）。
3. **期望达成情况（A，如实）**：gameplay「5202→显著下降」**未达成**（+90 微幅回退）；
   gs5「1524 不回退」**达成**（1,524，PNG 逐字节不变——但该样本 BG 面 draws=0，验证力有限，§7-6）；
   stg03「58159 逐字节不变」**达成**（sha `246dd5712608` = 历史值）；两跑确定性 **6/6**。
   按纪律**如实在账、不调阈值/不白名单/不回滚**。
4. **测试（A）**：`cargo test -p game-desktop` **64 项全绿**（52 基线 + 9 p0-1 槽语义 + 3 新增
   v 分型路由单测：v=3 只走调色板 / v=1 走行 / 混型帧正确分流）。
5. **两跑确定性（A）**：3 样本 × 2 配置 × 2 跑 png sha256 逐对全同（6/6）。

## 1. 语义映射表（定谳 → 实现）

| # | 定谳结论 | 出处 | 实现落点 |
|---|---|---|---|
| 1 | **v=3=调色板挂起型**：NMI `CMP #3 → bank15:$F105` 路=调色板上传 `$3F00/$3F10` **2 组×7B，源=`$161-$167`** | l5-0160 背景条；R-l4-linebuf-4（A）；linebuf-timing-wnew §2.1/§3（A） | `SetSnap::Palette{v, pal}`（置位时刻 `$161-$167` 快照）；NMI 消费 ⇒ `pal_upload_state=[[u8;7];2]`（双组同源）；**行快照不应用** |
| 2 | **v≠3（含 1/其它）=行上传**（1=行完成候选，B+；其它未证） | l5-0160 §5 建议 4（v 分型路由保留为后续步）；p0-1 §6-2 | `SetSnap::Row{v, snap}`（行快照落 nametable，现行为几何路全不动） |
| 3 | **置位快照记 v 值**；NMI 消费按 v 分流 | 任务卡；l5-0160 §0-3（单槽挂起位域=类型位域） | `SetSnap::v()`；路由在置位时刻定型（变体=路由真源）——混型覆盖以最后 v 定型 |
| 4 | v=0 写=清槽语义（不入槽）；NMI 消费点=清空 | l5-0160 §3（NMI 消费后 `STX $160` 清零闭环）；p0-1 §6-9 | `classify_event` 现行为不动（v=0 ⇒ Apply 镜像不置位）；`consume()` 取走即清空 |
| 5 | 缺 op 旧 probe（读写混合）**无法按 v 分型** ⇒ 按旧行为固定行路 | p0-1 §1-9（旧 probe 回归核） | `end_frame` 回退 ⇒ `SetSnap::Row`（v 记 `$0160` 镜像值仅供诊断）；stg03 逐字节不变（A） |
| 6 | 多置位并入单槽（判定 6 不变） | l5-0160 §5 判定 6 | `PendingSlot<SetSnap>` 覆盖语义不变；单测 `mixed_vtype_frames_route_separately` |
| 7 | 调色板面接入=**既有 `palette_src`/`$0161` 族**（W-C1 影子上传族注释线索） | render_frame 既有调色板段；probe 无 `$3Fxx` 写事件（A，三场景 0 条）⇒ 无"$3F00/$3F10 写事件协同"输入 | sidecar：`palette_src`（源镜像，既有语义不变）+ 新增 `pal_3f00`/`pal_3f10`（v=3 消费落下状态）；日志 `pal_state=`/`pal_uploads=` |

## 2. 实现要点（render_frame.rs 默认路）

- **`SetSnap` 枚举**（新增）：`Palette{v, pal:[u8;7]}` / `Row{v, snap:RowSnap}`；纯函数
  `SetSnap::is_pal_type(v)=(v==3)`、`SetSnap::v()`。
- **`EventReplay`**：`slot: PendingSlot<SetSnap>`；`apply` 的 `ApplySet` 分支按 v 分型快照——
  **v=3 不建行快照**（`snapshot()` 不调用，连带 `hud_seq` 不推进）；诊断计数增
  `row_sets`/`pal_sets`。
- **NMI 消费（帧首 `consume`）**：`match SetSnap`——`Palette` ⇒ `pal_upload_state=[pal,pal]`
  + `pal_uploads+=1`（**不动 nametable**）；`Row` ⇒ 既有上传路逐字不动（geo=0/1/2 + attr 行）。
- **可观测**（旧字段前缀不变，新字段追加行尾/新 sidecar 键）：日志 `replay_row_sets=
  replay_pal_sets= row_uploads= pal_uploads= pal_state=`；DBGUPLOAD 增 `row_sets/pal_sets/
  row_uploads/pal_uploads`；sidecar `pal_3f00`/`pal_3f10`。
- **未证不发明**：`$3F00`→像素索引绑定未证（attack-plan §3.2 残差）⇒ 调色板路**只落上传状态**
  （渲染端调色板状态=被 v=3 消费应用的上传态），**不动像素色选**；缺口显式登记 §7-2。

## 3. 验证环（3 样本 × {legacy, MODELGEO=1} × 2 跑；`tools/diff_visual.compare` len(samples)）

口径同 p0-1 §3：expected=`oracle/screens/expected/{l4_stg02-start, gameplay-stage1_f01260_stage-start,
l4_clear-inject}.png`；渲染=`render-frame 3450 stg02-start gs5-anchor-render` /
`1260 gameplay-stage1_f01260 gameplay-stage1` / `4600 l4_clear-inject stg03-timeline`。
**before=上轮 after**（p0-1 事件序回放 binary `render-frame-after.exe`）；after=本卡 v 分型路由
binary（`target/debug/render-frame.exe`）。输入 sha 与上轮全同（probe `935678c325c9`/`7fd1643cfaaf`/
`b71ee8fcbb84`；frames/text 同）⇒ 同输入可比（A）。

| 样本 | 配置 | before=上轮 after | after=v 分型路由 | Δ(after−before) | 判定 |
|---|---|---:|---:|---:|---|
| gs5-anchor-render f3450（stg02-start） | legacy | 1,524（`245cbedc4398`） | **1,524**（`245cbedc4398`） | **0（PNG 逐字节同）** | 不回退 ✓ |
| gameplay-stage1 f1260 | legacy | 5,202（`51b7a201f021`） | **5,292**（`efc5fb2dbc33`） | **+90（微幅回退）** | 「显著下降」未达成 ✗ |
| stg03-timeline f4600（clear-inject） | legacy | 58,159（`246dd5712608`） | **58,159**（`246dd5712608`） | **0（逐字节同）** | 旧 probe 回归核 ✓ |
| gs5-anchor-render f3450 | MODELGEO=1 | 1,524（`245cbedc4398`） | **1,524**（`245cbedc4398`） | 0 | ✓ |
| gameplay-stage1 f1260 | MODELGEO=1 | 5,292（`efc5fb2dbc33`） | **5,292**（`efc5fb2dbc33`） | 0 | ✓ |
| stg03-timeline f4600 | MODELGEO=1 | 58,152（`532936218b01`） | **58,152**（`532936218b01`） | 0（逐字节同） | ✓ |

- **before 侧复核（A）**：本轮用 `render-frame-after.exe` 重渲 legacy 三样本，diff/sha 与 p0-1 §3
  持久产物（`target/p0-1/render/after-legacy-r1`）逐项全同（5,202/1,524/58,159）⇒ before 口径可信。
- **after 侧口径**：gameplay after（5,292）与 before-geo（5,292）sha 相同（`efc5fb2dbc33`）——
  本卡后 gameplay 无行上传 ⇒ MODELGEO 不再产生差异（自洽检验 ✓）。

## 4. 机理证据（DBGUPLOAD 实测 + 逐像素复算，A 级）

| 量（gameplay f1260） | before（上轮 after） | after（v 分型） | 差 |
|---|---:|---:|---|
| `upload_write`（行上传落格数） | **2,240**（=70×32，全 legacy 32 连写形） | **0** | v=3 行应用全数消失 ✓ |
| `nt_nz` | 319 | 326 | before 的 0 码擦写抹掉 7 格 |
| `draws`/`canvas_nz` | 186 / 4,259 | 189 / 4,351 | BG 可视（该样本 draws>0） |
| 两输出逐像素差 | — | — | **90 px**（=5,292−5,202，全部为"before 偶然修对"） |

| 量（gs5 f3450） | before | after | 差 |
|---|---:|---:|---|
| `upload_write` | **3,008**（=94×32） | **0** | ✓ |
| `nt_nz` | 355 | 373 | before 擦写抹 18 格 |
| `draws`/`canvas_nz` | **0 / 0** | **0 / 0** | **BG 面不绘** ⇒ nametable 差异不可见，PNG 逐字节同 |

- **置位时刻状态（`set_moment_state.py`，A）**：gameplay/gs5 置位时刻 regs 多为
  `$2000(lut)/$2100/$22C0`（**≥$2000 有效基**）且行内容非零（gs5 69/94、gameplay 57/70）⇒ 写**落位**
  （与 `upload_write`=32×置位数精确吻合）；**p0-1 §3「94 次快照疑全走 HUD LUT 回退」被证伪**
  （HUD LUT 回退条件=`!addr_ok`，实测 addr_ok=true）——legacy≡geo 逐字节同的真因是 **draws=0**。
  > 【2026-10-20 更正（登记向，不改他人文档——G7 同式）】p0-1-event-replay.md §3「94 次置位快照疑全走
  > HUD LUT 回退（置位时刻 `$8E/$8F`<$2000 ⇒ geo 强制 legacy 形）」**误诊**：set_moment_state.py 实测
  > 94 次置位时刻 regs=`00:20(lut1)×65 / 00:21×19 / c0:22×10`（全 ≥$2000，HUD LUT 回退条件不成立）；
  > 两几何档输出逐字节同（`245cbedc4398`）真因=**BG 面 draws=0**（f3450 视口下 BG 不绘，
  > nametable 差异对 PNG 不可见）。
- **v=3 误刷行确有其事但非回退主因（A）**：before 的 70 次 v=3 行应用（2,240 格擦写/写入）在
  逐像素上仅 90px 净效应且**误打误撞修正**（5,292→5,202）；剔除后 5,292 ⇒ **B 假设证伪**。

## 5. 两跑确定性（同配置 2 次渲染 png sha256 全值对，A）

| 样本 | 配置 | run1 = run2 sha256 | 判定 |
|---|---|---|---|
| stg02-start | legacy | `245cbedc4398d24c1582be12394545550a9f4a2766ff318c70d30e58a84a490d` | **同** |
| gameplay-stage1_f01260 | legacy | `efc5fb2dbc335f621d2d1e0b330559cc3e1849718996643a657b884e630c355e` | **同** |
| l4_clear-inject | legacy | `246dd5712608160e5c81447ff540b58ba023bb8e8c6ada3ce721907278b83a0b` | **同** |
| stg02-start | MODELGEO=1 | 同 legacy 行 | **同** |
| gameplay-stage1_f01260 | MODELGEO=1 | 同 legacy 行 | **同** |
| l4_clear-inject | MODELGEO=1 | `532936218b01144ebf06500075a6e69113a7317be0b515f9f0f4ced81c94bf2b` | **同** |

（`python target\p0-1-vtype\determinism.py`：**6/6 SAME**。）

## 6. 测试（cargo test -p game-desktop，64 项全绿）

- 基线 52 项 + p0-1 槽语义 9 项不动全绿。
- 新增 3 项（`event_replay_tests`）：`v3_set_routes_palette_only`（v=3 只走调色板：pal=$161-$167
  置位时刻快照入槽/不建行快照/消费=Palette）、`v1_set_routes_row`（v=1 走行）、
  `mixed_vtype_frames_route_separately`（混型帧正确分流：v=3/v=1 跨帧各自走路 + 同帧混型
  后置位覆盖以最后 v 定型 + 计数对账）。
- 既有 2 项适配新 API（语义不变）：`snapshot_taken_at_set_moment`（改用 v=1 行路验证置位时刻快照）、
  `legacy_opless_frame_end_fallback`（增断言：缺 op 回退**固定行路**）。

## 7. 未证残留与证伪记录（明示）

1. **B 假设证伪 ⇒ gameplay 3,247→5,202 回退主因悬空（B 候选，未证）**：候选=①p0-1 撤回"帧末单快照"
   后**行更新缺失**（旧模型每活跃帧一行 vs 现模型仅置位快照——gameplay 行内容更新可能不全经
   `$0160` 置位路）；②读流隔离（op=r 不镜像）改变了 buf 镜像相位。需逐拍取证（与 G3 合并）；
   **不得据此回滚或改阈值**。
2. **调色板内容语义缺口（未证不发明）**：`$3F00/$3F10` 组内槽位映射（7B 落 `+0..6` 还是 `+1..7`）与
   `$3F00`→像素索引绑定未证（attack-plan §3.2 残差=索引绑定）；本卡调色板路只落**上传状态**
   （`pal_upload_state`/sidecar `pal_3f00`/`pal_3f10`/日志 `pal_state`），不动像素色选。
   probe 三场景 `$3Fxx` 写事件 **0 条**（A）⇒ 任务卡"$3F00/$3F10 写事件协同"无输入，待 PPU
   调色板 watch 采样（attack-plan §5.4 调色板 watch 项）后闭合。
3. **v=2（完成型）**：任何 trace 零观测（G5 继承，A）；实现按 v≠3 ⇒ 行路（现行为），语义未证。
4. **v=0 写取消已挂起槽（p0-1 §6-9 继承）**：本实现不回退已入槽快照；gameplay 逐拍序列无取消例
   （A，p0-1 §3），跨场景（bank3 BBBF 族/tas-stg08b2）未证。
5. **G3 填充↔置位↔消费逐拍链未闭合**（继承）：pal 快照同理——本模型取**置位时刻** `$161-$167`，
   真机 NMI 读源在**消费时刻**；若置位后仍有 `$161-$167` 填充写，会有帧内尾部差（量化待 G3）。
6. **gs5 样本验证力警示（A）**：f3450 视口下 BG 面 `draws=0`（BG 不绘）⇒ nametable 差异对 PNG
   不可见；「gs5 1524 不回退」成立但**主要只证 sprite/可视面不回归**。建议后续卡增补 BG 可视
   帧样本（或 gs5 另取帧）做行上传敏感验证。
7. p0-1 §3 HUD LUT 回退误诊更正见 §4 登记注（不改他人文档）；latent F1（hud_seq 快照计数）/
   F2（ptr_writes 吞 op=r）/F3（空 probe 口径）未动（p0-1 收尾注登记）。
8. 三样本总和（legacy）before 64,885 → after **64,975（+90，净微负）**：如实在账；默认 legacy
   配置保持，v 分型路由为语义正确性落地（B 假设证伪后**不得**以 diff 反推回滚语义）。

## 8. 复现命令

```bash
cargo test -p game-desktop                                   # 64 项全绿
cargo build -p game-desktop --bin render-frame
powershell -ExecutionPolicy Bypass -File target\p0-1-vtype\run-vtype.ps1    # 3 样本×{legacy,geo}×2 跑
powershell -ExecutionPolicy Bypass -File target\p0-1-vtype\run-before.ps1  # before 重渲（上轮 binary）
python target\p0-1-vtype\compare.py        # diff_visual.compare（含 p0-1 持久产物对照）
python target\p0-1-vtype\determinism.py    # 两跑 png sha256（6/6）
python target\p0-1-vtype\set_moment_state.py  # 置位时刻行缓冲/regs 状态（HUD LUT 误诊更正证据）
# 机理（DBGUPLOAD）：DBGPROBE=1 分别跑 before/vtype binary 对比 upload_write/draws
```

产物目录：`target/p0-1-vtype/render/{before-legacy-r1, vtype-{legacy,geo}-r{1,2}}/`；
对照用上轮产物：`target/p0-1/render/after-{legacy,geo}-r1/`。
