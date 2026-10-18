# L4 0 容差攻坚方案（task-7）——22 对差异逐族归因 + 实现方案

> 日期：2026-10-18｜角色：decomp-analyst（取证/方案，**未做实现**）｜分析时点 HEAD：`1be66e5`
> 上游裁定：用户 2026-10-18 裁定 L4 判据回到 **0 容差（白名单外像素 100% 一致）**，
> 签署口径不构成达成（[decision-l4-zero-tolerance-ruling.md](../knowledge/decision-l4-zero-tolerance-ruling.md)，commit `24093be`）。
> 纪律：AGENTS.md（A/B/C 证据分级；未证不发明；数字可独立复算；结论带证据指针；推翻处 dated 更正）。
> 数据源（只读）：`oracle/screens/{expected,actual,diff}`、`oracle/screens/visual-report.md`、
> `crates/game-desktop/src/bin/render_frame.rs`、`crates/game-desktop/src/{lib,render_rules}.rs`、
> `crates/game-core/src/terrain.rs`、`crates/game-oracle/src/bin/phase_batch.rs`、
> `tools/diff_visual.py`、`reference/seed/gunnac.asm`、`rom/gunnac_cn.nes`、`docs/findings/*`、
> `git` 对象库。**本文件是唯一写入产物**；未改 `crates/`、`data/`、`gates/`、`rom/`、`reference/`，未 git add/commit。
> 本方案**不主张 L4 达成**：0 容差当前未达成，且（见 §2）现行 22 对基线本身尚不可复现。
>
> **修订记录（2026-10-18 第二轮，按 task-8 对抗复核修正）**：复核报告
> [adversarial-review-l4-attack-plan.md](adversarial-review-l4-attack-plan.md) verdict=needs_revision——
> 数据面 C1-C10 全部独立精确复现（本轮新增 C11 独立/非独立拆分）；不通过项=判据/账目/载体边界。本修订处理
> **S1/S2/P1/E1-E7/S4-S10/P2/P3**（映射见 §9.1），并纳入用户 **2026-10-18 本轮裁定**：
> ① **提供 Linux+Mesen2 采集环境**（P0-0/P0-4 可执行）；
> ② **撤销 W-C1/W-C4 白名单候选**（0 容差判据下不需要白名单；`gates/visual_whitelist.json` 保持不存在）；
> ③ **批准门禁硬化**（`diff_visual.py` 0 容差语义；**两步走**：`--strict` 已实现、默认翻转待 P0-6 后，见 §5.6）。
> 另：P0-0 前本方案所有"diff 下降"数字仅为**离线对拍器**度量，**不代表游戏运行时显示一致**（P1，见 §4.4）。
>
> **修订记录（2026-10-18 第三轮，task-24 复审 pass 后折入 R1/R2 + 顺手 R3-R7）**：验收判据统一为
> `python tools/diff_visual.py --strict`（L4 0 容差；默认/pre-commit 翻转后置 P0-6 全绿后，引
> task-21/`64b3a44`）；P0-1 补 nt2 子目标；R3-R7 折入映射见 §9.2（R8 归 task-20/21）。本案仅 docs-only 改动。

## 0. 三句话结论

1. **现行 22 对差异 280,132/1,351,680 = 20.7247% 可独立复算一致**（逐帧与 `visual-report.md` 全同）；
   但**这套 22 对基线不可复现（A）且可能非同态（B）**：5 张 ending 实际图是同一 blob、4 张 stg06/stg08
   实际图是同一 blob、stg02-start 的期望图是 boot-black 复制品、渲染输入工作区（`analysis/`）不在库、
   Mesen2 缺失（新证 F-0）——**0 容差第一步不是改渲染，而是冻结可复现基线（P0-0）**。
2. 差异按**像素性质**可作**互斥**分解（A 级）：缺内容 262,352（93.66%）+ 多画内容 6,624（2.36%）+
   双非黑异色 11,156（3.98%）。再按"实际 nametable 该格是否有码"二分为：
   **① 投影/nametable 内容空 209,086（74.63%）**、**③ 写进但渲不出（跳码/空字形）53,266（19.01%）**、
   **② 调色板索引绑定 11,156（3.98%）**、**⑤ stg02 资产错配 902（0.32%）**、交叠余项 5,722（2.04%）。
   仓库 `visual-report.md` 的"投影未覆盖区 266,886"是**几何启发式**（`y>=16` 的行），不是根因账。
3. **顺序与收益**：P0-1 写流投影（可动 ① 的 200,879 舞台/商店像素，含 fade-clear 59,371 全量）>
   P0-2 族映射窗基矩阵（③ 53,266）> P0-3 调色板索引绑定（② 11,156 + 余项 5,722）>
   P0-4 资产/场景重采（⑤ 902 + 四联重复 69,136 的假重复）> P0-5 3B 表（0 直接像素，是 ② 文本场景的依赖）。
   **0 容差可达性（S1 修正，唯一口径）**：22 对中 **12/22 为"独立 expected+actual 对"**
   （G1 非冻结 8 + G3 3 + title 1），在数据面有**可执行收敛路径**；**10/22 需重采/重渲后用新基线重判**
   （stg02 1 + 四联冻结帧 4 + ending 5——ending 五帧 actual 同 blob 必须逐帧重渲，ending-4/5
   expected 同文件需裁定"真重复"或重采）。12+10=22，**不再使用"17/22 可闭环"旧口径**。
   （口径交叉引用：本节与 §4.1 的族像素为**22 对全量上界**；**分步验收判据用 §4.2 的"独立 12 对"口径**
   ——①137,263/③45,514/②6,814/余项 960，见 §4.2；非独立 10 对交 P0-4 新基线重判。R3）

---

## 1. 口径钉死（先读，避免与仓库旧账混用）

### 1.1 本方案的四个度量口径

| 口径 | 定义 | 复现 |
|---|---|---|
| **D-total** | 逐像素 RGB 不等计数（真彩域，alpha 不参与——与 `tools/diff_visual.py:83,101-107` 一致）；22 对合计/22×61,440 | 命令 C1 |
| **N-3way（像素性质）** | 每差异像素按 `(expected, actual)` 二分：`missing`=actual 黑&expected 非黑；`spurious`=expected 黑&actual 非黑；`color`=双非黑异色 | 命令 C2 |
| **F-family（互斥族）** | missing 再按"实际 nametable 该格码值"二分：`empty-cell`（码=0）→ 族①；`skip-code`（码=8 或 ≥128，渲染器主动跳过）/`present-blank`（码≠0 但渲黑）→ 族③；color→族②；stg02 的 spurious→族⑤；其余 spurious→交叠余项 | 命令 C3 |
| **dedupe** | 以 PNG 字节 sha256 识别完全相同的 expected/actual 对，剔除重复比较后另算 | 命令 C4 |

> **为什么不用 `visual-report.md` 的"条带内/条带外"作根因**：`tools/diff_visual.py:173-175` 的实现是
> `in_strip = y<16`、`out_strip = y>=16`——这是**行坐标几何分桶**，与"投影未覆盖/调色板映射"没有因果绑定。
> 本方案保留其数字（4.73%/95.27%）作对照，但**归因一律以 N-3way/F-family 为准**。

### 1.2 旧账核验（引用前已逐条验证；失效项不得当现行）

| 证据 | 现状（2026-10-18 核验） | 本方案处置 |
|---|---|---|
| `oracle/screens/visual-report.md` | **现行**：22 对/1 zero/280,132 差；数值经 C1 复算全同 | 作基线数字源 |
| `docs/findings/l4-final-localization-ending1.md` | **时点失效**：标题为 "7753 态残差 7380px/261 格"，现行 ending-1 diff=4552（TBLDRV 态）——不同时点/不同实现态 | **不引用为现行残差账** |
| `docs/findings/residual-pixel-ledger.md` | **部分失效**：AR2-A（自机 −$30 锚点）已撤销；"每像素可归因/全族勾销"判据不成立；AR2-C 复现命令错址已更正（$ABCD/$AC0D、file 0x10BCD/0x10C0D） | 只取其仍有效的机制条目 |
| `docs/findings/adversarial-audit-l4.md` | **现行**（AR2-A/B/C/D/E/F/G 仍有效） | 引用 |
| `docs/findings/adversarial-review-103.md` | **pass**（F-A103 已修）：$BEFB 96B 窗=32B 数据+64B 码流混入；真消费形=$BF0B/$BF13 双并列字节表（$4D 索引） | 引用；§3.4 补一处分级校正 |
| `docs/spec/l4-signing-criteria.md` | 顶部用户裁定覆盖"终签"的完成语义；**§4 第 3/4/5 项对应本方案族③/④/⑤；第 1/2 项（对位率、trace v=1/v=2）是度量/对账缺口、不是像素族；族① 另有来源（nametable 持久化模型缺口：`linebuf-timing-wnew.md` + R-l4-linebuf 面 + `l4-visual-plan.md`【2026-10-16】主缺口判定），族② 另有来源（§2 R-6/W-C1 语义面）**（E6 修正：原"§4 五缺口=五族来源"映射不成立）；"902=文本内容本身差"**不在本文件 §4.5**（见 §3.5 E6 修正） | 引用；§3.5/§5.4 误引已更正 |
| `docs/spec/l4-visual-plan.md` L375-382@1be66e5 "L4→100% 战役收官" | **过程里程碑**，非 0 容差达成（用户裁定） | 仅作历史轨迹；行号引用带 `@1be66e5`（该文件正被 task-13 修改，防漂移） |
| `docs/findings/strip-tile-family.md`(t238) / `strip-family-direct.md`(t242) / `chr-org-timing.md`(t154) / `multicolor-palette-truth.md`(t204) | **B/待复核**（E5 修正）：四篇 doc 标题均**自标"待审稿"**，方案原列"现行 A 级数据面"属**证据等级超标**；本方案只作**候选证据**，不得据此定谳 | 引用降级；其数据面待对抗复核后方可升 A |
| `docs/findings/l4-render-support.md` R-1/R-2/R-3/R-6 | **现行 A 级**（ROM/seed 字节可独立复导：调色板三件套/流全谱/描述符） | 引用（仅取可独立复算的字节/指令面） |
| `docs/findings/adversarial-review-l4-attack-plan.md` | task-8 复核报告（verdict=needs_revision；**数据面 C1-C10 全对**） | 本修订的来源；§9.1 给逐条映射 |

### 1.3 任务卡路径更正（A 级）

任务卡写的 `crates/game-desktop/src/render_frame.rs` **不存在**；实际渲染器在
**`crates/game-desktop/src/bin/render_frame.rs`**（1036 行，`render-frame` bin）。
`crates/game-desktop/src/render_rules.rs` 存在（124 行）。本方案文件指针一律用实际路径。

---

## 2. 基线复算（A 级）

### 2.1 22 对逐帧差异复算（与 `visual-report.md` 逐项一致）

| # | 帧 | diff | # | 帧 | diff |
|---|---|---|---|---|---|
| 1 | gameplay-stage1_f01260_stage-start | 1,019 | 12 | l4_late-combat | 1,579 |
| 2 | gameplay-stage1_f01500_mid-combat | 1,490 | 13 | l4_shop-window | 17,454 |
| 3 | gameplay-stage1_f02349_end | 1,753 | 14 | l4_stg02-start | 902 |
| 4 | l4_clear-inject | 58,101 | 15 | l4_stg03-start | 1,205 |
| 5 | l4_combat-variety | 27,962 | 16 | l4_stg06-late | 17,284 |
| 6 | l4_ending-1 | 4,552 | 17 | l4_stg07-combat | 9,627 |
| 7 | l4_ending-2 | 3,682 | 18 | l4_stg07-start | 10,990 |
| 8 | l4_ending-3 | 3,347 | 19 | l4_stg08-combat | 17,284 |
| 9 | l4_ending-4 | 3,981 | 20 | l4_stg08-late | 17,284 |
| 10 | l4_ending-5 | 3,981 | 21 | l4_stg08-start | 17,284 |
| 11 | l4_fade-clear | 59,371 | 22 | title-menu_f00180_boot-black | **0** |

合计 **280,132 / 1,351,680 = 20.7247%**；同点 1,071,548 = **79.2753%**；逐像素相等 **1/22**；
`<7680` 的帧 **12/22**。全部可由命令 C1 复算。

> **【2026-10-18 更正（task-156；GUI 重采 + 新二进制基线；旧数字留档作历史）】** 上表与合计基于**旧 headless
> actual**（`emu.setInput` 不生效、输入未注入，语义无效）。**现行基线**（GUI 重采 trace + render-frame
> `00c29d6a31c0…`）：
> - **22 对 = 18 可渲 + 4 `blocked:no-source`**（fade-clear / ending-1/2/3）+ **5 `replaced-invalid-asset`**
>   （#14 stg02-start + 四联 #16/#19/#20/#21；其中四联/#17/#18 expected 来源已恢复，待 §7.7 安装）；
> - 18 对 diff：**min 0**（title，像素级 0）/ **median 9,766** / **max 58,159**（clear-inject）/ **sum 242,555 px**；
> - **actual==expected sha = 0/18**（L4 0 容差**未达成**，属 P0-1 修复目标）；去重 **expected_unique=13 / actual_unique=14**。
> 证据：`p0-0-readjudication-gui.md` §0/§3、`target/p0-0/render/g-manifest-v2.json`（rendered 18 / blocked 4 /
> replaced 5 / expected_matches 0）。旧 `280,132 / 20.7247% / 1/22 / 12/22` 数字**保留作历史**（旧基线作废）。

### 2.2 N-3way 像素性质三分（A 级）

| 性质 | 像素 | 占比 | 判定 |
|---|---|---|---|
| **missing**（actual 黑 / expected 有内容） | **262,352** | 93.6635% | A：像素级机器复算 |
| **spurious**（expected 黑 / actual 有内容） | **6,624** | 2.3646% | A |
| **color**（双非黑异色） | **11,156** | 3.9824% | A |
| 合计 | 280,132 | 100% | — |

### 2.3 分组（G1-G5）——后续族表的行基准

| 组 | 帧 | D-total | missing | spurious | color |
|---|---|---|---|---|---|
| **G1** 舞台/商店/冻结屏（12 帧） | clear/fade/combat/stg06/stg08×3/shop/stg07×2/late/stg03 | **255,425** | 246,860 | 2,952 | 5,613 |
| **G2** ending（5 帧） | ending-1..5 | **19,543** | 14,003 | 2,590 | 2,950 |
| **G3** gameplay-stage1（3 帧） | f1260/f1500/f2349 | **4,262** | 1,489 | 180 | 2,593 |
| **G4** stg02-start（1 帧） | stg02-start | **902** | 0 | 902 | 0 |
| **G5** title（1 帧） | boot-black | **0** | 0 | 0 | 0 |

> 单帧极值：`fade-clear` 59,371 **全部**是 missing（0 color/0 spurious）——最干净的"投影缺失"样本；
> `stg03-start` 1,205 中 **1,001 是 color**（内容位置对、颜色错）——最干净的"调色板"样本；
> `stg02-start` 902 **全部 spurious**——最干净的"实际多画/期望无效"样本。

### 2.4 新证 F-0：现行 22 对基线不可复现（A），且可能非同态（B，待 P0-0 定谳）

| ID | 事实 | 证据（可复现） | 影响 |
|---|---|---|---|
| **F-0-A** | **actual 端同 blob 重复**：`l4_ending-1..5` 的五张实际图**逐字节相同**（sha256 前缀 `65af76c95e11`；`git rev-parse HEAD:oracle/screens/actual/l4_ending-1.png` = `37aeef94…` 与 ending-5 相同）；`l4_stg06-late`/`stg08-{start,combat,late}` 四张实际图相同（sha256 `db7262bcdd0c`） | 命令 C4/C5；`.render.log` 末尾五条 ending 均 `nt_nz=159`（同状态） | 签署"5/5 转正"是**同一张 actual 对 5 张不同 expected** 的 5 次比较；四联帧是**同一比较×4**（4×17,284=69,136=总差 24.68%） |
| **F-0-B** | **expected 端资产错配/非独立**：`l4_stg02-start` 期望图与 `title-menu_f00180_boot-black` **同一文件**（git blob `d8d2f9ff…`；sha256 `788d6984576b…`，258B；解码=**全黑**）；`stg06-late`/`stg08×3` 四张期望图**同一文件**（git blob `065224fc…`；sha256 `62a8ef06c4b9…`，6,074B）；`ending-4/5` 期望图同一文件（sha256 `6ff345dc5c3f…`） | 命令 C4/C5；`git rev-parse` 两两相等 + expected 解码全黑/actual 非黑 902px 独立支撑；`109b4d7` commit message 自述"stg06-late/stg08 四帧同值 16752=同一冻结画面（出店受阻结构冻结像素级实证——OBS-1 截断态可视）"（**与提交物一致**） | stg02-start 不是渲染回归（见 §3.5）；四联帧不是逐关内容帧（见 §5.4） |
| **F-0-C** | **输入不可复现**：`render_frame` 依赖 `analysis/traces/{scene}.frames.jsonl`（:92）、`{scene}.probe.jsonl`（:574-575）、`analysis/phase/{scene}-text.jsonl`（:610-618）；`analysis/` 被 `.gitignore` 忽略且**当前工作区不存在**（仅 visual-scout/workset）；`tools/mesen2/` 缺失（忽略项）；`tools/scenario_run.py` 顶部 `import fcntl`（L34）且 **POSIX-only**（自管 Xvfb/DISPLAY，t24 已弃 `xvfb-run` 包装；`:467-477`），**本 Windows 工作区无法重放采集**；`.render.log` 只记 `name/frame/base/scroll/nt_nz/probe`（`render_frame.rs:1011`），**不记 env/args** | 命令 C5；`cmd /c dir /b analysis` 仅两目录；`.gitignore` `analysis/`；`scenario_run.py` L34 `import fcntl` + :467-477 | 22 对 actual 的生成命令不可从产物反推；"diff 下降"可能只是 render 命令/env 变化而非实现进步 → 必须先冻结基线 |
| **F-0-D** | **诊断表混记（代码事实 A）**：签署态 `TBLDRV=1 GOLDPAL=1 FAMMAP=1 FLAGUPLOAD=1 ROWMAP=1`（`l4-visual-plan.md:377@1be66e5`）中，`TBLDRV=1` 直接读 **`docs/findings/l4-golden-lookup-ending1.md`（golden 反查 160/960）** 落位（`render_frame.rs:127-149`），`FAMMAP=1` 用静态 `FAM_MAP` golden 族表（:40-71）、`GOLDPAL=1` 用 golden 三色域（:940-943）；代码自注"**诊断面非正实现**"（:129） | 代码逐行；commit `65f9b29`/`33348f2`/`22d896c` 自述"表驱动诊断/纯化/量控" | ending 签署值来自诊断态；`clear-inject` 的实际图**不含** ending golden 表（其上 379 个 nt0 码与 ending 的 159 个同位置非零交集=0，命令 C6）⇒ **推断（B）：22 对 actual 可能非同套 env 产物**——C6 只能否定"clear-inject 与 ending 同为 TBLDRV 态"，不能单独证明"不同 env"；判定依赖 **P0-0 逐对 env manifest**（E3 修正） |

> **（E4 修正）** 原稿引 `cf6847b` 自述"l4_stg02-start 关名屏 **0 像素差**"支撑 F-0-B——该自述**与其提交物矛盾**：
> 该 commit 提交的 expected=全黑 258B、actual 非黑 902px，物下 diff 应为 902 而非 0。**自述数字不得作证据**；
> F-0-B 结论仅由 `git blob` 全等 + expected 解码全黑（命令 C4/C5）独立支撑。

**F-0 结论（A/B 分级）**：F-0-A/B/C 为 A 级（blob/解码/文件系统事实）；F-0-D 的"代码事实"为 A，
其"22 对异构"推论为 B。在 F-0 闭合前，任何"逐族消解"的实施都会在不可复现的基线上度量。
**P0-0 基线冻结必须先于 P0-1..5**（见 §5.0）。

### 2.5 dedupe 辅助口径（派生，A 级算术）

- 唯一 actual blob **15** 个（22 张中 ending-1..5 合并、stg06/stg08 四张合并）；
- 唯一 expected blob **17** 个（stg02/title 合并、stg06/stg08 四张合并、ending-4/5 合并）；
- 唯一 `(expected, actual)` 比较 **18** 对；**去重后合计差 224,299 /（18×61,440=1,105,920）= 20.28%**。
- 该口径只用于**排优先级**（识别重复样本），**不替代** 22 对判定（判定仍逐对 0 容差）。

### 2.6 逐对 provenance 表（P0-0 验收物模板 / S1+E1+E2 的消除手段）

> 下表列**全部可从现有产物读出**（A 级：`.render.log` 末条 + PNG sha256）。**scene 与 env 两列缺失=正是 F-0-C**；
> P0-0 必须在 Linux+Mesen2 环境补齐 `scene/env/argv/输入 sha` 后才能作为"独立对/需重采"的最终裁定依据。
> 复用列=dedupe 关系（同 expected 或同 actual）。

| # | 帧 | render 帧 | scroll | probe 数 | expected sha256(12) | actual sha256(12) | 复用关系 |
|---|---|---|---|---|---|---|---|
| 1 | gameplay-stage1_f01260_stage-start | 1260 | (0,114) | 1,307,322 | 38365db2cb4f | 80047b496b89 | 独立 |
| 2 | gameplay-stage1_f01500_mid-combat | 1500 | (0,114) | 1,307,322 | 7f4ac8a23511 | fba4a5778b7c | 独立 |
| 3 | gameplay-stage1_f02349_end | 2349 | (0,96) | 1,307,322 | 4274a33f45f5 | 5e67e8aa53b5 | 独立 |
| 4 | l4_clear-inject | 4600 | (0,120) | 598,571 | 5cacbe294d53 | 519a6476ec8e | 独立 |
| 5 | l4_combat-variety | 6500 | (0,52) | 598,571 | 6f7fe4992e59 | 75261b68a6e5 | 独立 |
| 6 | l4_ending-1 | 30000 | (0,0) | 990,904 | 19b88b1ffce9 | 65af76c95e11 | actual 复用（5 帧共用） |
| 7 | l4_ending-2 | 31500 | (0,0) | 990,904 | 0f8e11694d53 | 65af76c95e11 | actual 复用 |
| 8 | l4_ending-3 | 33000 | (0,0) | 990,904 | f319f1fe5256 | 65af76c95e11 | actual 复用 |
| 9 | l4_ending-4 | 34500 | (0,0) | 990,904 | 6ff345dc5c3f | 65af76c95e11 | actual 复用；expected 与 #10 同文件 |
| 10 | l4_ending-5 | 36000 | (0,0) | 990,904 | 6ff345dc5c3f | 65af76c95e11 | actual 复用；expected 复用 |
| 11 | l4_fade-clear | 28500 | (0,170) | 990,904 | 14802bcf460a | 2e57455e5db6 | 独立 |
| 12 | l4_late-combat | 12000 | (0,0) | 598,571 | c0baa0f8d21b | d7f37bc898bc | 独立 |
| 13 | l4_shop-window | 5400 | (0,0) | 598,571 | 988809e0b414 | 7f5de7294764 | 独立 |
| 14 | l4_stg02-start | 3404 | (0,208) | 598,571 | 788d6984576b | 164b6e48c6ca | **expected 与 #22 同文件（boot-black）** |
| 15 | l4_stg03-start | 5700 | (0,132) | 598,571 | 24a4cb45c0d8 | 569b4600aef1 | 独立 |
| 16 | l4_stg06-late | 22000 | (0,0) | 1,167,314 | 62a8ef06c4b9 | db7262bcdd0c | 四联复用（#16-19） |
| 17 | l4_stg07-combat | 16000 | (0,74) | 1,167,314 | 2824cb134987 | 72c47da73a5a | 独立 |
| 18 | l4_stg07-start | 14600 | (0,144) | 1,167,314 | ddb322ac5702 | d08c894ebb8b | 独立 |
| 19 | l4_stg08-combat | 18000 | (0,0) | 1,167,314 | 62a8ef06c4b9 | db7262bcdd0c | 四联复用 |
| 20 | l4_stg08-late | 28000 | (0,0) | 1,167,314 | 62a8ef06c4b9 | db7262bcdd0c | 四联复用 |
| 21 | l4_stg08-start | 16600 | (0,0) | 1,167,314 | 62a8ef06c4b9 | db7262bcdd0c | 四联复用 |
| 22 | title-menu_f00180_boot-black | 180 | (0,0) | 0 | 788d6984576b | c7081ba52b85 | expected 与 #14 同文件（diff=0 已过） |

**S1 独立对核算**：12/22 独立对 = #1/#2/#3/#4/#5/#11/#12/#13/#15/#17/#18/#22
（G1 非冻结 8：#4/#5/#11/#12/#13/#15/#17/#18；G3 3：#1/#2/#3；title 1：#22）；
**10/22 需重采/重渲** = #6-#10（ending 5）+ #14（stg02）+ #16/#19/#20/#21（四联 4）。

> **【2026-10-18 更正（task-156；旧 provenance 作废留档）】** 上表 expected/actual sha 与"复用关系"来自**旧
> headless 基线**（输入未生效）。现行 GUI 基线口径：
> - 22 对 = **18 可渲 / 4 blocked / 5 replaced**；逐对 expected/actual sha、diff、bbox、分类以
>   `target/p0-0/render/g-manifest-v2.json` + `p0-0-readjudication-gui.md` §2 为准；
> - 去重：**expected_unique=13 / actual_unique=14**（旧 17/15 与"唯一比较 18 对"口径不再适用）；
> - **四联/#17/#18 expected 来源已恢复**（`stg08-timeline` GUI：四联=f16500–33900 共 64 帧同 `62a8ef06`、
>   #17=f14600 `ddb322ac`、#18=f16000 `2824cb13`；见 §3.7 末 task-156 块），**待 §7.7 安装核验**；
>   **#14 `l4_stg02-start` expected 仍 invalid**（`788d6984`=boot-black 复制品）。
> 旧"S1 12/22 独立对、10/22 需重采"数字保留作历史。


---

## 3. 差异族五分表（核心交付物）

### 3.0 总表

> 互斥归因；像素量=22 对合计。证据等级：A=字节/机器复算/trace；B=可复现静态推理；C=猜测。
> `visual-report` 对照：几何"条带外 266,886 / 条带内 13,246"仅作对照列。

| 族 | 差异形态 | 像素量 | 占 280,132 | 归因（主根） | 证据等级 | `visual-report` 几何对照 |
|---|---|---|---|---|---|---|
| **① 投影/nametable 内容空** | actual 黑、expected 有内容；且实际 nt 该格码=0 | **209,086** | **74.63%** | nametable 写流未投影/丢行（BG 内容持久化写侧）；ending 文本行为 0 码族 | 像素量 A；`empty-cell 130,466` **B+**（E2：nt0 归属依赖默认 `RENDER_NT=0`，idx 未记 ntr/env；P0-0 记录 env 后升 A）；`nt2 域 78,620` B（idx 仅存 nt0） | 落在"条带外"多数 |
| **② 调色板索引绑定（W-C1）** | 双非黑异色，且两侧颜色 100% 在 NES 调色板域 | **11,156** | **3.98%** | **调色板索引选择/绑定**（attr 组、ink 表、$3F00 编程）；**RGB 映射与色彩空间已排除** | 颜色域 A（C7 可独立复导）；索引绑定根因 B；ink/机制引用 B/待复核（multicolor doc 自标待审稿，E5） | 分布在"条带内"多数 |
| **③ 写进但渲不出** | actual 黑、expected 有内容；且实际 nt 该格码≠0（跳码或空字形） | **53,266** | **19.01%** | 族映射 `tile_page` 一般规则缺（window base/CHR bank）＋码≥128/08 的 fail-closed 跳码语义 | 像素量 A（码≠0 且渲黑）；归因 B（三样本来自自标待审稿 doc→**B/待复核**；上下文键=C 级假设 §8-H2） | 落在"条带外"多数 |
| **④ 3B 表残余** | 当前**无直接像素**（默认路不启用 RECP3/RECPAL/TRIHOP/FUSE） | **0** | 0% | 该表是 ②/①b 文本场景的**上游依赖**；96B 窗超读已裁，真结构=双字节表但语义未证 | 结构 A（96B 窗字节=32B 数据+64B 码流；$BF0B/$BF13 双表，adversarial-review-103 pass）；"非调色板记录表"语义 B/待复核；编码 C | 不计入（依赖项） |
| **⑤ stg02-start 资产错配** | actual 有关名屏内容、expected 全黑 | **902** | **0.32%** | **expected 资产错配**（=boot-black 复制品；非渲染 bug） | A（blob 全等 + 902 全 spurious） | 全在"条带外" |
| **余项（交叠）** | spurious（除 stg02 外 5,722） | **5,722** | **2.04%** | ②/③ 的"索引 0 vs 非 0"交互项：actual 多画了 expected 黑处内容；随 ②③ 修复收敛 | A（像素量）；根因 B | 落在"条带外"多数 |
| **合计** | — | **280,132** | 100% | — | — | 266,886+13,246 |

### 3.1 族① 投影/nametable 内容空（209,086；74.63%）

**已证到哪一步**
- **前提（E1 修正，开工前置）**：本节代码事实对应**默认路（`FLAGUPLOAD` off）**；签署态 env 含
  `FLAGUPLOAD=1`（`l4-visual-plan.md:377@1be66e5`），而该 env 会**关闭** `pending_row` 上传路
  （`render_frame.rs:694` `if var("FLAGUPLOAD").is_err()`），改走 per-flag 事件直写路（:301+）。
  ⇒ **生成签署 actual 的实际路径待 P0-0 逐对 env manifest 确认**；族①根因在 P0-0 前只作
  "默认路代码偏离"表述，P0-1 改动面不得以签署态路径为依据（复审 requirement）。
- 模型层（A）：NMI 上传=行缓冲 `$130-$14F` 32B → `$2007`，**4 组×8B，组间地址步进 +$20**，
  基址=`$8F:($8E+$0F)`，`$160` 挂起类型（0/3）上传后清零；调色板源 `$161-$167` 双组×7B。
  证据：`bank15:F09A`（`BD 30 01 LDA $130,X`，seed L89416）、`bank15:F14B`（`8E 60 01 STX $160`，L89487）、
  `bank15:F105`（L89460）、`bank15:F040`（NMI 入口，L89377）、`bank00:85D0/85DC`（bridge 写）；
  `docs/findings/linebuf-timing-wnew.md` §2/§3（NMI 级插桩 196,738 写事件）；
  `docs/spec/l4-visual-plan.md` §3.5 R-l4-linebuf-1/2/3/4/6/7/10；
  模型实现已在 `crates/game-core/src/terrain.rs:301-339 Nametable::nmi_flush`（4 组×8B/+$0F/+$20/attr 路），
  有测试 `crates/game-core/tests/linebuf_timing.rs:6-53`、`stage_content.rs:129-150`。
- 代码偏离（A，限默认路）：`render_frame.rs:565` 只有**单个 `pending_row`**；:694-715 每帧只上传**一行**；
  :784-801 每帧只入队一行——**与 A 级模型"4 组/帧"不一致**（丢行）。`FLAGUPLOAD=1` 路的行为另测（P0-0 后）。
- 现状量（A/B）：nt0 可见域**码=0 的缺失像素 130,466**（**B+**：像素量 A，nt0 归属依赖默认 `RENDER_NT=0`，
  见 E2；P0-0 记录 env 后升 A）；
  非 nt0 域（idx.json 只持久化 `ntr=0`，见 `render_frame.rs:1003-1008`（idx 写入）与 `:815`（`RENDER_NT` env））**78,620**（B，同机制推定）。
- 场景分布：G1 200,879（96.07%）、G2 ending 7,003、G3 1,204。
  G1 典型：`fade-clear` 59,371（**全部** missing）、`clear-inject` 57,340、`combat-variety` 27,283、
  `stg08` 四联 16,393×4、`shop-window` 16,553、`stg07-start` 10,186、`stg07-combat` 8,911。

**差什么（取证/输入）**
1. **写流输入**：`--probe-file`（`render_frame.rs:229-518`）是唯一能吃到 `$0130-$016F`/`$2006/$2007` 写事件的通道，
   但 `analysis/traces/*.probe.jsonl` 当前不存在（F-0-C）→ **需重采**（场景 DSL 已有 `probe_mem` 指令）；
2. **逐场景 text 写集**：`crates/game-oracle/src/bin/phase_batch.rs` 只回放**文本装配子程序**（`$2F/$30` 指针）
   → 舞台 BG 写流不在其中（A 级代码事实）；需要把**全写流**（行缓冲+NMI+直写）纳入投影输入；
3. **帧内相位**：bridge 整块扫写/读后即废/NMI 相位（R-l4-linebuf-5，W-NEW）；`nmi_flush` 模型已 A 级，可直接接线；
4. **ending 文本子族**：`l4-golden-lookup-ending1.md` 的 y17-20/23-28 行为 `????`（CJK 未匹配）→
   ending 5 帧 missing 14,003 中 **empty 7,003 / skip 4,705 / present-blank 2,295**（A 级分类），
   即"文本码未成格（7,003）+ 码≥128 被跳（4,705）+ 有码但空字形（2,295）"——不是单一 BG 写流问题。

### 3.2 族② 调色板索引绑定（W-C1）（11,156；3.98%）

**已证到哪一步（A 级）**
- **色彩空间/RGB 映射已排除**（本方案新证，A）：11,156 个 color 差异像素中，
  **expected 与 actual 颜色 100% 落在 `render_frame.rs:19-28` 的 64 色 NES 调色板表内**
  （命令 C7）；`title-menu boot-black` 帧 0 容差也证明两侧同一调色板/同一色彩空间。
  ⇒ **W-C1 的残余根因不是"NES→RGB 映射表"，而是"逐像素选哪个索引"（palette slot / ink / attr）**。
- 颜色对 Top（占 color 族比例）：`白→暗青` 3,239（29.0%）+`暗青→白` 2,269（20.3%）= **5,508（49.4%，
  即 CJK ink 表 1/3 绑定反/错）**；`粉(254,204,197)→白` 1,680、`鲑(254,129,112)→白` 775、
  `红(181,49,32)→白` 600、`淡紫(254,196,234)→白` 510、`浅蓝(100,176,255)→白` 427
  = **3,992（35.8%，GOLDPAL 三色域把彩色画成白）**（命令 C7）。
- 机制文档：`docs/findings/l4-render-support.md` R-6/R-2b/R-1/R-2/R-3（**A 级**：ROM/seed 字节可独立复导；
  上传通道 `bank15:$F04B-$F072`，seed L89382；位域改写器 `bank00:85E8-8618`，seed L797；三表
  `$E9D2/$E9D6/$E9DA`，seed L88516；数据源 `$0150+X×3+($7A:$7B)`，`bank00:85A9` seed L765；
  `$0150` 三写点 `bank00:83A4/84E6/8515`，seed L492/L662）；`multicolor-palette-truth.md` §1
  （8 色实测 + 槽×索引表）= **B/待复核**（doc 自标待审稿，E5）。
- 场景分布：G1 5,613（50.3%，`stg03-start` 1,001 / `stg07-start` 774 / `clear-inject` 754 /
  `stg07-combat` 665 等）、G2 ending 2,950（590×5）、G3 2,593。
  **`stg03-start` 最干净**：内容位置全对，1,001/1,205 是颜色错。

**差什么**
1. **调色板状态输入**：当前 trace 的 watch 面**不含 PPU 调色板**（`$3F00-$3F1F` 经 `$2006/$2007` 写），
   需 probe `2006 2007` + 场景采样（`analysis/traces/*.frames.jsonl` 的 watch 字段无此项）→ **新场景/trace**；
2. **P0-3 真实改动点（S5 修正）**：`render_rules.rs:58-73 ink_table` **已是 family×index 二维表**
   （已含 CJK 槽 1=暗青/3=白、Graphics(logo) 槽 1=深蓝/2=浅蓝/3=白、Game 槽 2=白）——**不是"单式→二维"**。
   P0-3 真正要改的是：①删 `GOLDPAL` 默认覆盖（`render_frame.rs:940-943`）；②**BG family 选择**
   （`:945` 现 `bank>=124→Cjk else Game`）改由 WindowContext/槽位决定；③Graphics `DARK_BLUE=(18,44,110)`
   与 multicolor 表 `(21,95,217)` 不一致——需先裁定（该表 B/待复核）；
3. **白名单判定（已裁定）**：用户 2026-10-18 **撤销 W-C1/W-C4 白名单候选**——证据指向实现索引错、
   非平台差异；0 容差判据下不设白名单条目（见 §6.2）。

### 3.3 族③ 写进但渲不出（tile_page/族映射）（53,266；19.01%）

**已证到哪一步**
- 像素量（A）：实际 nt 该格码≠0 但渲黑的 **present-blank 45,359** + 渲染器主动跳过的
  **skip-code 7,907**（码=8 或 ≥128）→ 53,266。G1 45,981（86.3%）、G2 7,000、G3 285。
- 代码根因（A）：`render_frame.rs:894` 缺省 `tile_page[ntr][ty]==2 ? 44 : 124`——**`tile_page` 只在文本回放
  路径 :773 更新**（`if f<1207 {1} else {2}`）；未经文本回放的 BG 行 `tile_page=0` ⇒ **一律按 124（字模族）渲染**。
  `FAMROW/FAMMAP`（:873-894）是结局卡专用逃生口；`render_rules.rs:47-54 family_bank` 只有 4 个固定族 bank。
  `:851-853` 把码 8/≥128 一律当"参数/填充"跳过（对 CJK 文本行是**误杀**）。
- 真规则候选（**B/待复核**：三样本来自自标"待审稿"doc，E5；上下文键=C 级假设，见 §8-H2）：
  `strip-tile-family.md` §1：**`bank = 窗基 + T//64`、`tile = T%64`**，窗基 菜单=124 / CJK=28 / 图形=20
  （样本：'J'74→bank125:10 非零 19；CJK 16→bank28:16 非零 32；图形 31→bank22:31 非零 38）。
  `strip-family-direct.md` §1：条带文本=**bank124/125（75+2 直接 CHR 命中）**，窗基 20/28/22/30/96/106 全 0。
  `chr-org-timing.md` §1/§2（B/待复核）：R0 三配置=游戏 `$6A` / 字模 `$7C,$7E` / 图形 `$36,$38`；
  **12 个切换点**（f1207/1292/1474/1508/1690/1724/1906/1929/1964/2146/2188/2216），渲染期 R0=$6A 无帧内切换。
  `t104-two-family.md` §1：nametable 基址上下文 `$2000` 系（HUD/菜单）vs `$2Bxx` 系（关卡内容）。
- 代码级旁证（A）：`render_frame.rs:856` 的 `mapped = raw_tile-95 if >=96` 是**拟合映射**（非一般规则）；
  `render_rules.rs:27-37 family_of` 的 slot≥60/tile≤3 判据只对单个实证案例成立（t210），不能当一般律。
- **旁发现（A，非默认路）**：`crates/game-desktop/src/lib.rs:915-948 r4b_stream_parse` 的 ROM 取址
  `0x10 + 2*0x2000 + (stream_addr-0x8000)` 对 `$A000` 窗的 R-4b 屏流**取错 bank**：$A31F →
  file 0x632F（读到的不是流），而文档载明的流字节在 file 0x232F（bank1 窗 $A000）。命令 C10；
  该函数只被 `R4B` env 诊断路（`render_frame.rs:150-176`）调用，不在签署/默认路，但若 P0-1/P0-4
  要接 R-4b 屏流（fade/OP/结局画面），**必须先修**。

**差什么**
1. **窗口上下文的判定键**（**C 级假设**，§8-H2；R0 三配置数据面 B/待复核）：需要"当前渲染上下文→窗基
   （124/28/20/106）"的可判定键——候选：CHR R0 状态（`$8000/$8001` 写流，t154 方法）、场景描述符/关卡号、
   `$1A` bit3（精灵侧）；**需新 trace**：`probe_mem 8000 8001` 按 L4 各场景重采（历史仅 `gameplay-stage1` 有）；
2. **码段边界**：≥128 段（参数/标记/CJK）与 08 填充的语义边界要定谳后才能"该画/不画"（现 fail-closed 跳码）；
3. **`$D7B9` tile 校正**（`l4-render-support.md` R-1：CN ROM 迁至 `bank14:$DA4C`，9B=`00×7,$C1,$C1`，
   seed L86602 / file 0x1DA5C 已验证）——关 8/9 的 tile 索引校正，需接 `draw_chr` 索引修正；
4. 本族与族①**共用同一批像素**（present-blank 是"码在但画错"），修复 P0-1 后这些像素会从 ① 迁移到 ③，
   **判据必须同时看 total diff + 分类矩阵**（不能只看单一类别下降）。

### 3.4 族④ 3B 表残余（$BEFB 窗/动态基 $7A:$7B）（0 直接像素；依赖项）

**已证到哪一步**
- **场景描述符**（A）：`bank00:$97D7`，8B/项（`bank00:9785 LDA $97D7,Y`，seed L2972）；
  项 0 字节实测=`2e 2c fb be 09 18 ff 97`（file 0x17E7）→ 流指针 `$2E/$2C`、**调色板记录基址
  `$7A:$7B=$BEFB`**、表 2=`$1809`（`l4-render-support.md` R-2 表；本方案 hexdump 复核 ✓）。
- **消费算式**（A）：`$0150+X×3 + ($7A:$7B)`（`bank00:85A9-85C6`；`bank00:85A9 STX $10`，seed L765）。
- **96B 窗真相**（A）：`render_frame.rs:1020-1029 PALETTE_RECORDS[96]` ≡ ROM **`bank05:$BEFB`**（file 0xBF0B）
  起 96B：**前 32B=数据（16B+8B+8B）+ 后 64B=6502 码流**（本方案 hexdump 复核；
  `adversarial-review-103.md` §A.2 的 F-A103）。
- **双字节表**（A 字节面）：`bank05:$BF0B`=`22 22 22 41 41 23 23 07`、`bank05:$BF13`=`04 04 04 05 05 06 06 07`
  （file 0xBF1B/0xBF23 复核 ✓），被同 bank 码流 `LDA $BF0B,Y / STA $064C` / `LDA $BF13,Y / STA $0630` 消费
  （W-DMG3 互证）——**注意：这两张是"游戏字段表"，不是调色板记录表**（**B/待复核**：本方案读法，
  仅"非 3B 记录表"的证伪意义成立；"真表结构=这两张"的反推未经复核方裁定）。
- **当前直接像素=0**：默认渲染路不用 `palette_record_rgb`（`render_frame.rs:1033-1036`）；RECP3/RECPAL/
  TRIHOP/FUSE 均为 env 逃生口，签署 actual 未启用（`adversarial-review-103.md` §B.2 三态对照：
  RECPAL 18,390 劣化未入默认路）。

**差什么**
1. **每场景 bank 映射（r7）**：`$7A:$7B`=5 项基址（`$BEFB/$C0D2/$BB4E/$BB4E/$BFFB`）；
   在 bank5 映射下只有项 0（$BEFB）是数据表，项 1-4 落 code 字节（命令 C8）⇒ **基址需按场景 r7 换 bank**；
   每场景的 r7 表/映射未证（B）；
2. **记录编码**：16B 块的非 3B 整除（16 不能被 3 除尽）⇒ "3B/项"模型与 16B 数据区结构矛盾；
   3B=RGB 三元组是 **C 级**（值超 6bit 色域），**禁止据此实现**；
3. 需 **$0150/$7A/$7B/$8D-$90 动态 trace**（ending/shop 帧）才能定位真消费形态；
4. 由于是 ②/①b 的上游依赖，本族**不产生独立收益**，但 ② 的文本场景闭环需要它（或显式裁定弃用）。

### 3.5 族⑤ stg02-start 902 回归（902；0.32%）

**已证到哪一步（A 级）**
- 期望图 `oracle/screens/expected/l4_stg02-start.png` 与 `title-menu_f00180_boot-black.png` **同一 blob**
  （`git rev-parse HEAD:...` 均 `d8d2f9ff384160118ca9a0fb40ba6f300599ee99`，258B）——**boot-black 复制品**；
- 实际图 `l4_stg02-start.png`（`164b6e48…`）是**关名屏**（nt0 562 码，file log `frame=3404 scroll=(0,208)`），
  902 差异**全部是 spurious**（expected 黑/actual 有关名屏内容）；
- 时序不一致：L5 锚 `stg02-start f3450`（`gs5-6-dynamic-replay.md` §/`adversarial-review-98.md`），
  而本次渲染用 `frame=3404`（`.render.log`）——渲染帧与锚不一致。
⇒ **这是 oracle 期望资产错配（错帧/错文件复制），不是渲染回归**。`l4-visual-plan.md:349-350@1be66e5`
（2026-10-16 L4 签署终判段）"902=文本内容本身差"的表述**不成立**（expected 全黑时不存在可比的文本内容），
提案 spec-writer dated 更正。**（E6 修正：该表述不在 `l4-signing-criteria.md` §4.5——§4.5 是"fallback 修不适用域"，
无"文本内容本身差"字样。）**
**差什么**：重采正确 expected（候选帧 f3450 = C 级假设 H4，见 §5.4；先定 render 帧并与 expected 同帧），
并重配对；属于**需新 oracle 场景/资产**。

### 3.6 交叠余项 spurious 5,722（2.04%）

- 构成：G2 ending 2,590（518×5，字形/色块画到 expected 黑处）+ G1 2,952 + G3 180。
- 归因（B）：② 的"索引 0 vs 非 0"与 ③ 的"跳码/空字形"在 expected 黑处的镜像表现；随 P0-2/P0-3 收敛。
- 处置：**不单列收益**，但计入各步验收判据（每帧 spurious 必须单调下降至 0）。

---

## 4. 可消解顺序与收益量化

### 4.1 顺序依据（三因子：像素上界 / 除根数 / 前置依赖）

| 序 | 动作 | 直接改善族 | 像素上界 | 除根数 | 前置 |
|---|---|---|---|---|---|
| P0-0 | 基线冻结（可复现 render 清单 + 输入恢复） | —（使度量有效） | 0 | 1（F-0） | **环境已由用户裁定提供（Linux+Mesen2，2026-10-18）**；fixtures 入库政策未定（备选）；未完成 P0-0 则 P0-1..全部 BLOCKED（S9） |
| P0-1 | 写流投影（Nametable::nmi_flush 接线） | ① | **209,086**（其中 G1 200,879） | 1（写流回放） | P0-0 |
| P0-2 | 族映射窗基矩阵 | ③ | **53,266** | 1（上下文窗口键）+1（跳码语义） | P0-0；建议在 P0-1 后（像素迁移） |
| P0-3 | 调色板索引绑定 | ②+余项 | **16,878**（11,156+5,722） | 1（$3F00/ink/attr） | P0-0；在 P0-2 后 |
| P0-4 | 资产/场景重采 | ⑤ + 去重面 | 902 +（重复 69,136→17,284） | 1（oracle 资产） | 需 Mesen2 |
| P0-5 | 3B 表定谳/弃用 | ④（依赖） | 0 | 1（表语义） | 新 trace |
| P0-6 | 门禁硬化 + 终验 | — | 剩余全部 | 1（判据） | 全部 |

**为什么先写流后族映射**：①（209,086）是 ③（53,266）的 3.9 倍；且 ③ 的 present-blank 判定
依赖"码已在 nt 里"，P0-1 会把大量空单元填上码，使 ③ 的真实规模**显露**（当前 ③ 可能是低估）。
反之先做族映射会在空 nametable 上无效。
**口径注**：本表"像素上界"=22 对族全量（§3.0）；**分步验收判据用 §4.2 的"独立 12 对"口径**——
非独立 10 对（四联/ending/stg02）在 P0-4 重采重渲后按新基线重判，不并入分步承诺（S1+S2）。

### 4.2 分步收益（S2 修正：**分步判据只在本步族内 + 独立 12 对口径计数**；total-D 不作 PASS 条件）

> **为何不用 total-D**：按本方案自身步骤范围，P0-1 只治族①，其后残余 ②+③+⑤+余项 =
> 11,156+53,266+902+5,722 = **71,046**；P0-2 后残余 ②+⑤+余项 = **17,780**；P0-3 时 stg02 902
> 尚未到 P0-4——原稿 `total D≤25,000/15,000/500` 在算术上不可达（task-8 S2）。故各步 PASS 只看
> **本步族内像素**；total-D 仅记录趋势（允许因"①→③ 迁移"暂时上升）。
>
**独立 12 对 / 需重采重渲 10 对口径（S1+S2 合）**：分步判据只对**独立 12 对**（§2.6）承诺数值；
10 对（ending 5 + stg02 + 四联 4）在 P0-4 重采/重渲后**按新基线重判**，不并入本表判据。
按此口径的族内像素（本方案对 C3 六桶按独立/非独立逐对重算，命令 C11）：

| 族 | 22 对合计（§3.0） | 独立 12 对 | 非独立 10 对（交 P0-4） |
|---|---|---|---|
| ① | 209,086 | **137,263** | 71,823（四联 4×16,205 + ending 7,003） |
| ③ | 53,266 | **45,514** | 7,752（四联 4×188 + ending 7,000） |
| ② | 11,156 | **6,814** | 4,342（四联 4×348 + ending 2,950） |
| 余项 spurious | 5,722 | **960** | 4,762（四联 4×543 + ending 2,590；**另 ⑤ stg02 902 单列**） |
| ⑤ | 902 | 0 | 902 |

校验：137,263+71,823=209,086；45,514+7,752=53,266；6,814+4,342=11,156；960+4,762=5,722 ✓；
独立 12 对合计=190,551、非独立 10 对合计=89,581、两者 190,551+89,581=280,132 ✓（命令 C11）。

| 步 | 验收判据（机器复算，命令见 §7；**族内 + 独立 12 对**） | 族上界（22 对） | 备注 |
|---|---|---|---|
| P0-0 | 22 对 canonical render **两次 sha 全同**；manifest 补齐 `scene/env/argv/输入 sha/输出 sha`；22 对 provenance 表（§2.6 模板）填满 | — | 不改渲染语义；同时冻结 E1 逐对 env |
| P0-1 | 族①（独立 12 对）：**137,263→≤10,000**；其中 `fade-clear` ① 50,071→≤2,000、`clear-inject` ① 38,400→≤3,000；独立对 nt0 empty-cell 58,643→≤2,000；**独立对 nt2 域 78,620→≤8,000（R2；承接=`$2006/$2007` 默认投影 + 4-NT 持久化；nt2 现为 B 级推定 §3.1/§8-H6，P0-0 后按实测重算）**（前提=P0-0 确认 env/输入可复现；否则本步不判） | 209,086 | 非独立 10 对的 ①（71,823）交 P0-4；**不设 total-D 阈**；nt2 子目标与 nt0 子目标之和=10,000 与总支一致 |
| P0-2 | 族③（独立 12 对）：**45,514→≤5,000**；非独立 10 对的 ③（7,752，含 ending skip 4,705）交 P0-4 逐帧重渲（S7） | 53,266 | 先修"码≥128 误跳"再修窗基；**不设 total-D 阈** |
| P0-3 | 族②（独立 12 对）：**6,814→0**；余项 spurious 960→≤200；非独立 10 对的 ②（4,342）交 P0-4 | 16,878 | 以 `stg03-start`（1,001 color）为单帧验收哨兵；**不设 total-D 阈** |
| P0-4 | stg02 重采后**按新基线重判**（旧 902 仅为"expected 无效"证据，不作对错基准）；四联帧从"同图×4"变 4 个独立比较；ending 五帧 actual 不再同 blob；10 对全部重渲后用**新基线重判** | 902+重复面 | 需 Linux+Mesen2（已裁定提供） |
| P0-5 | `$7A/$7B` 动态 trace 落档；表语义 A/B 定谳或显式弃用（C 级不进默认路） | 0 | 依赖项 |
| P0-6 | **22/22 diff=0**；**`python tools/diff_visual.py --strict` exit 0**（L4 0 容差验收判据；输出 `STRICT: PASS diff=0`，task-21/`64b3a44` 已实现；**默认模式"全部有分类即 PASS"会假绿，不得用于验收**）；`gates/cli.py check` PASS；两次 render sha 一致 | 全部 | 0 容差才算达成；**默认/pre-commit visual 门禁翻转（切 `--strict`）留到 P0-6 全绿后另卡**（避免未清零恒红阻断 P0-1..）；§4.4 载体声明同时生效 |

### 4.3 0 容差可达性（S1 修正：唯一口径=12/22 独立对 + 10/22 需重采/重渲）

- **12/22 独立对**（G1 非冻结 8 + G3 3 + title 1，逐对见 §2.6）：①/③/② 有 A/B 级根因与可复现数据面，
  **有可执行收敛路径**（title 已 0；其余按 P0-1→P0-2→P0-3 判据）；
- **10/22 需重采/重渲后用新基线重判**：stg02（期望=boot-black，须重采）、stg06/stg08 四联
  （同一冻结画面 OBS-1 截断，须逐关重采）、ending 五帧（actual 同 blob，须逐帧独立重渲；
  ending-4/5 expected 同文件须裁定"真重复"或重采）；
- **结论**：**L4 0 容差当前不可达**；不是"算法全错"而是"基线/资产/输入三缺 + 五个已定位族的实现缺口"。
  在 P0-0 与 P0-4 完成前，任何"趋近 0"的数字都不能作为 L4 达成证据。
  **12 有路径 / 10 未证明 / 22 合计**（12+10=22，不再使用"17/22 可闭环"）。

### 4.4 L4 判定载体与运行时渲染器（P1 新增，任务卡 check3 裂缝的处置）

> **问题**：P0-1..P0-3 的文件级改动全部落在 **`game-desktop/src/bin/render_frame.rs`（离线 L4 对拍器）**；
> 而可玩 shell 的渲染是另外两套：`game-shell::render`（v1 显式占位：只画已证坐标面玩家块，
> pattern/palette=占位，内容锚"渲染合成（调用 Renderer 256×240 RGBA 画布；v1 显式占位）"）+
> `shell-win::FrameRenderer`（独立复制 GREY4/blit 的第四套路径，`lib.rs` 内 `pub struct FrameRenderer`）；
> 未来 Android 还要再实现一套 `game-platform::Renderer`。
> （行号按复核/分析时点；`game-shell/src/render.rs` 正被并发任务修改——**以内容锚为准**，见 §9.1-P1。）
> ⇒ 即使 P0-6 达成"22/22 diff=0"，收敛的是**离线对拍器**；"对拍器 0 容差 ≠ 游戏显示 0 容差"。

**本方案声明**：
1. **L4 判定对象 = 离线 `render-frame`**（与现行 `gates/cli.py visual` 语义一致）；本方案所有像素数字/判据
   均指此载体，不得表述为"游戏运行时像素一致"。
2. **运行时渲染器另立判据**（另立任务卡）：运行时截图 vs 同一 expected 集，判据与白名单政策沿用本方案
   （用户已裁定 0 容差/无白名单）；在运行时判据建立前，**运行时不宣称 L4**。
3. **共享渲染规则模块路线（建议，需 Lead 批准结构变更）**：把已证/待证渲染规则从
   `game-desktop` 下沉为平台无关纯函数模块——候选 `crates/game-core/src/rendering_rules.rs`
   （game-core 无 I/O、确定性，符合纪律）或独立 crate `game-render`；消费者= `render_frame`（离线对拍）
   + `game-shell`（平台无关合成）+ `shell-win`/`Android`（`game-platform::Renderer` 实现）；
   重构判据：**render_frame 改调共享模块后 22 对 diff 数字不变**才算行为保持。
   **时序（R6）**：重构须在 **P0-0 之后**、以 canonical 基线/manifest 判据验证（否则以不可复现 actual
   验收无效）；结构变更本身需 Lead/用户批准（§6.2-4）。
4. **Android 任务卡前置**：新平台任务必须引用本决策（判定载体=离线对拍器；运行时判据另立；
   规则只从共享模块来，不得再复制第四套）。

---

## 5. 实现方案（文件级改动点 + 完成判据 + 风险回退）

> 实施归 porter；以下为可执行的最小改动面（不含"发明未证行为"）。

### 5.0 P0-0 基线冻结（可执行清单；环境已由用户裁定提供）

> **前置裁定（2026-10-18 用户）**：提供 **Linux + Mesen2 2.1.1 采集环境** → P0-0/P0-4 可执行。
> 下列为**可执行清单**（P0-0 是 P0-1.. 的硬前置，S9）：
>
> **0. 环境接收**（Linux）：`python3`（非损坏别名）+ Rust toolchain + `tools/mesen2/Mesen`（2.1.1）
> + Xvfb；采集**串行**（`analysis/.scenario.lock`，`tools/scenario_run.py` 自管 Xvfb/DISPLAY）。
> **1. 恢复输入**：逐对所属场景执行 `python3 tools/scenario_run.py run <scene>`（产出
> `analysis/traces/<scene>.{frames,probe}.jsonl` + `run.json` sha）→
> `cargo run -p game-oracle --bin phase-batch -- <scene>`（产出 `analysis/phase/<scene>-text.jsonl`）；
> 场景↔22 对的映射必须**新采后确认**（旧 `.render.log` 未记 scene/env，F-0-C）。
> **2. render_frame 输出参数化（S4）**：现输出写死 `oracle/screens/actual/{name}.png/.idx.json` +
> 追加 `.render.log`（`:999-1011`）——**最小改动**：新增 `--out-dir <dir>`、`--log-path <file>`；
> 默认值保持现状（兼容）；本步先输出到 `oracle/screens/actual_canonical/` 与独立日志，
> **不覆盖** 现行 actual（回退可执行）。
> **3. 日志/manifest + canonical 命令**：`.render.log` 每条记录 `name/frame/scene/env(argv)/输入文件 sha256/输出 sha256`
> （`render_frame.rs:1010-1011` 一行扩展）；canonical 命令（每对一条，env 显式）：
> `cargo run -q -p game-desktop --bin render-frame -- <frame> <name> <scene> --out-dir oracle/screens/actual_canonical --log-path oracle/screens/actual_canonical/.render.log`
> （env 按 manifest 逐对显式设置，**禁止继承 shell 环境**——env 是 F-0-D 的根因面）；
> 产出 **`oracle/screens/l4-manifest.json`**：
> 22 行 = §2.6 模板 + 补齐 `scene/env/argv/输入 sha/expected_sha/actual_sha/independent|reuse`。
> **4. 两跑确定性**：同一 manifest 连跑两次，22 对输出 sha 全同。
> **5. 资产裁定**：ending-4/5 expected 同文件→裁定"真重复"（登记）或重采；四联 expected 重采归 P0-4。
> **6. 新基线**：用 canonical 命令重建 `visual-report.md`；若数值与 §2.1 不同，**以新值为基线**并
> dated 更正本方案 §2.1/§2.6。
>
> **判据**：manifest 完整（22 行含 scene/env/输入/输出 sha）；两跑 sha 全同；新报告落盘；
> 逐对 env 结论表落盘（E1 的族①路径确认依据）。
> **回退**：`--out-dir` 指向 `actual_canonical/`（不覆盖现行 actual）；或整仓库副本执行；
> 输入恢复失败 ⇒ 本步显式 **BLOCKED**，不进入 P0-1（S9）。
> （备选政策：L4 最小 fixtures 入库——`analysis/` 现为 gitignore；体积待实测，见 §6.2-4。）

### 5.1 P0-1 写流投影（族①，209,086 上界）

> **前置（E1）**：先由 P0-0 逐对 env manifest 确认**每对实际走哪条路**（`FLAGUPLOAD` on/off、probe 投影、
> `$2007` 直写、文本回放）；本步根因/改动面只对 **`FLAGUPLOAD` off 默认路**成立；签署态
> `FLAGUPLOAD=1` 会关闭 `pending_row` 路（`render_frame.rs:694`），其行为另测。**未确认前不得宣称
> "丢行=签署 actual 根因"**。

**改动文件/行**
1. `crates/game-desktop/src/bin/render_frame.rs`
   - `:561-565 / :686-814`（默认路）：删除单 `pending_row` 模型；按帧维护**行队列**（本帧所有 `wrote` 行），
     帧首 NMI 语义对每行调用 `game_core::terrain::Nametable::nmi_flush(&row32, &attrs8, addr_lo, addr_hi, attr_lo, pending)`
     （`terrain.rs:307`）；地址取**当帧** `$8E/$8F`（`apply_frame_writes` :657-684 已采），
     属性取 `$D3-$DA`，`pending` 取 `$160`。
   - `FLAGUPLOAD=1` 路（`:301+` per-flag 事件直写）：P0-0 确认后若为签署态路径，按 A 级 `nmi_flush`
     语义**重核其落位**（本方案不预置结论）。
   - `:483-503`：把 `$2006/$2007` 投影路（现 `PROJ2007` 逃生）改为**默认**，落实 R-l4-linebuf-15
     协议：`$2006` 高/低锁存、14 位掩码、`$2007` 自增（步长按 `$2000` bit2 1/32）、`$3FFF→$0000` 回绕
     （`l4-visual-plan.md` §6 更正后的三分离）。
   - `:610-635`：text 写集**保持**全写流时序回放（t228 持久化模型），但严格模式不再吞掉跨场景；
     新增 `--manifest` 或 env 固化输入路径（pairs→scene 映射）。
   - `:1003-1008`（idx 写入；`ntr` 取 `:815`）：`idx.json` 持久化**全部 4 个 nametable**（现在只存 `ntr`）——供复核脚本分类（A 级可验证性）。
2. `crates/game-desktop/src/lib.rs`：`render_bg_projection` 与 `Nametable` 的接口对齐（可选，若无调用方则不动）。
3. `crates/game-oracle/src/bin/phase_batch.rs`：可选扩展为"BG 写流批次"（或保持文本回放，BG 走 probe 路）。

**判据**：§4.2 P0-1 行（族①内计数）；另 `cargo test -p game-core -p game-desktop` 全绿；A 级模型测试不得放宽。
**风险**：输入缺失 → BLOCKED；`$2007` 路可能引入噪声（历史"PROJ2007=污染源"消融定谳，
`render_frame.rs:488-490`）→ 需 A/B 两跑对照，**不许为降 diff 而选择性开/关**。
**回退**：单 commit；保留旧 actual 归档；`--out-dir` 双跑验证（旧 actual 不覆盖）。

### 5.2 P0-2 族映射窗基矩阵（族③，53,266 上界）

**改动文件/行**
1. `crates/game-desktop/src/render_rules.rs`：新增纯函数（TDD，先断言后实现）
   - `pub fn bg_family_bank(code: u8, window_base: usize) -> usize { window_base + (code as usize)/64 }`
   - `pub fn bg_tile(code: u8) -> usize { (code as usize) % 64 }`
   - `pub enum WindowContext { Font /*124*/, Cjk /*28*/, Graphics /*20*/, Game /*106*/ }` + `window_base()`；
   - 单测固定 t238/t242 三样本（menu 'J' 74→bank125:10；CJK 16→bank28:16；图形 31→bank22:31 非零）；
     保留现有 `pair_indices/compose_index/ink_table` 的测试不降。
2. `crates/game-desktop/src/bin/render_frame.rs`
   - `:641/:773/:873-894`：用 `WindowContext` 上下文选择替代 `tile_page` 两态 + `FAMROW/FAMMAP`；
     上下文键 = 场景/关卡装载状态（**C 级假设，§8-H2**：CHR R0 写流终值；未证时 fail-closed 到 Font=124，
     **不用 expected 图选窗基**，不发明）；
   - `:851-853`：码 8（fill）/≥128（参数/CJK 标记）的跳码规则按 `strip-family-direct.md` §2 码段语义收窄
     （该 doc **B/待复核**）：≥128 段中 CJK 文本族应走窗基字形，不得一刀切跳过（先用其 75+2 直接命中样本做哨兵）；
   - `:856`：`raw_tile-95` 拟合映射替换为 `bg_tile()` 一般式（保留 A/B 对照开关）。
3. `docs/spec/l4-visual-plan.md` §3.6 相关行（spec-writer 更新，非本方案写范围）：
   `R-l4-linebuf-13` 真制、`FAMROW` 逃生口标注为"已被一般规则取代"。

**判据**：§4.2 P0-2 行（族③内计数）；`cargo test -p game-desktop`；三样本 Hamming=0 回归；
TBLDRV/FAMMAP/GOLDPAL 退出生产默认路（仅诊断）。**ending 五帧的 skip 4,705 不在本步**（P0-4 逐帧重渲承接，S7）。
**风险**：上下文键未证 → 可能"换个场景又错"；**不许**用 expected 图选窗基（拟合即违规）。
**回退**：env 双路（默认旧、新路待判据过线再切）；单 commit。

### 5.3 P0-3 调色板索引绑定（族②+余项，16,878 上界）

**改动文件/行**
1. `crates/game-desktop/src/bin/render_frame.rs`
   - 新增 **PPU 调色板状态**：32B `$3F00-$3F1F`（含 `$3F10/$3F14/$3F18/$3F1C` 镜像规则），
     由 `$2006/$2007` 写流维护（复用 P0-1 的协议实现）；渲染时 `color = palette[attr_group*4 + pixel]`
     （attr_group 由 `Nametable::block_palette`（`terrain.rs:342`）或 `$D3-$DA` 属性行给出）；
   - `:940-943`：**删除 `GOLDPAL` 三色域默认覆盖**（保留诊断 env）——这是本族当前最大色偏源；
   - `:945`：**BG family 选择**（现 `bank>=124→Cjk else Game`）改由 `WindowContext`/槽位决定（与 P0-2 共用）；
   - `:259-263`：`EC46` 色码表（11B）接线为色码→NES 索引（`l4-render-support R-6` 的流字节语法）。
2. `crates/game-desktop/src/render_rules.rs`：**`ink_table` 已是 family×index 二维表（:58-73），无需重做**；
   仅需：①`Graphics` 分支 `DARK_BLUE=(18,44,110)` 与 `multicolor-palette-truth.md`（**B/待复核**）的
   `(21,95,217)` 不一致 → 先裁定量值再改；②保留 `INK_NONE` fail-closed 与现有测试。
3. `:1015-1036`（族④）：`palette_record_rgb` 标注"超读注/非默认路"，不接生产。

**判据**：§4.2 P0-3 行（族②内计数）；`stg03-start` diff→≤50（单帧哨兵）；每帧颜色直方图 = expected；
所有出现颜色仍 ⊆ `render_frame.rs` P 表（回归：不引入新色）；`cargo test`。
**风险**：调色板状态输入缺失（无 `$3F00` watch）→ BLOCKED 或先静态推导场景调色板（`bank00:$9809`/
`bank01:$A7C2` 流全谱 A 级，可先置初值再逐帧更新）；**用户已裁定 W-C1/W-C4 白名单候选撤销**，
不得以白名单掩盖索引绑定错误。
**回退**：env `INKLEGACY=1` 回旧表；单 commit。

### 5.4 P0-4 资产/场景修复（族⑤ + 去重面；需 Mesen2，环境已裁定提供）

- **stg02-start**：重采 expected——**候选帧 f3450 仅为 C 级假设（§8-H4）**；重采前先定 render 用帧
  （现 actual 用 `frame=3404`）、使 expected 与 render **同帧**，并记录 `trigger=injected`（若注入触发）；
  更新 `oracle/screens/expected/l4_stg02-start.png` + diff；同步 `l4-visual-plan.md:349-350@1be66e5`
  的"902=文本内容本身差"dated 更正（E6；非 l4-signing-criteria §4.5）。
- **四联冻结帧**（stg06-late/stg08-start/combat/late）：OBS-1 出店截断未解（`bug-ledger.md` OBS-1），
  需 per-stage 可达 trace。**采集两档（P3）**：
  - 自然档（优先）：八关 bot（`tas-bot-eight-stage-input-plan.md`，未就绪）→ trigger=natural；
  - 注入档（最小集）：`poke 0180 <area>` 选关（`oracle/scenarios/README.md` §秘籍表），最小集=
    stg02-start、stg06-late、stg08-{start,combat,late}、ending-1..5 各 1 帧（共 **10 帧**），全部
    `trigger=injected` 显式登记；自然链到位后再替换。依赖顺序：注入档可先解锁重渲，自然档后补。
- **ending 五帧**：逐帧独立渲染（禁止 TBLDRV 共用 ending-1 表）；expected 逐帧独立采集。
- **provenance**：`oracle/screens/l4-manifest.json`（与 P0-0 合并）记录 scenario/frame/env/trigger/sha。
- **判据**：22 expected 两两不同 blob（真重复帧除外，且需显式登记）；stg02 expected ≠ boot-black；
  四联帧变 4 个独立内容比较；ending actual 五帧互不同 blob；**重渲后用新基线重判**（不以旧 22 对数字为准）。
- **风险**：OBS-1 未解则只能注入面；**回退**：旧资产归档 `.archive/`，保留历史。

### 5.5 P0-5 3B 表定谳/弃用（族④）

- `render_frame.rs:1015-1036`：保留 32B 数据区字节事实（A），删除"3B/项表"语义表述；
- 新 trace：`$0150/$7A/$7B/$8D-$90/$2006/$2007` 在 ending/shop 帧的动态值；按 r7 映射找每场景表基；
- 判据：表结构 A/B 定谳 **或** 显式裁定"3B 记录不进默认路（C 级封存）"；`data/naming/` 如需新增条目带证据。
- 风险：无 trace 时不可定谳 → 保持显式弃用（0 直接像素，不影响 0 容差判定，只影响文本场景色彩精度）。
- 回退：不动默认路。

### 5.6 P0-6 门禁硬化 + 终验（**用户已裁定批准；两步走**）

- **`tools/diff_visual.py` 两种退出模式（task-21/`64b3a44` 已实现；默认行为不变）**：
  - **默认模式**（分类账口径，pre-commit/既有流水线沿用）：exit 0 = 全部对比对通过**或**全部差异有显式
    四分类记录（文档串 `L9-14` 原文）——⚠ **默认 PASS ≠ 像素一致（22 对差异均已分类 ⇒ "假绿"），
    不得作 L4 验收判据**；
  - **`--strict` 模式 = L4 0 容差验收判据**：exit 0 = **任意外差异为零**（含"已分类差异"/缺失实际图/
    尺寸不符；白名单外像素差全部计入），输出固定行 `STRICT: FAIL/PASS diff=<总差异像素数>`
    （实现 `:156-162/:276-277`；文档串 `:15-18`）。
- **验收一律用 `--strict`**：`python tools/diff_visual.py --strict` exit 0 ⇔ 0 容差达成（22/22、
  `STRICT: PASS diff=0`）。
- **默认翻转（pre-commit 的 visual 门禁切到 `--strict`）留到 P0-6 全部 22 对清零后**（工具
  `TODO(task-21)` `:20-21`）；若现在翻默认：22 对未清零时 pre-commit 会恒红，**阻断 P0-1.. 提交**。
  翻转=门禁加强（非放宽），独立 commit + Lead 批准 + Evidence（AGENTS §5）；翻转前置=P0-0 canonical
  基线冻结 + P0-6 全绿。
- **`gates/cli.py`**（:815-818 `gate_visual_diff`）：目前仍派发**默认模式**（未带 `--strict`）——
  **翻转前 `gates/cli.py visual` 的 PASS 不表示 0 容差**；翻转卡里同步语义（输出"0 容差"字样）。
- **`gates/visual_whitelist.json`**：保持不存在；**用户已裁定撤销 W-C1/W-C4 候选**（0 容差下无白名单条目）。
- **判据**：`python tools/diff_visual.py --strict` → exit 0 / `STRICT: PASS diff=0`（22/22）；
  `python gates/cli.py check` PASS（其中 visual 门仍默认口径，待翻转）；`cargo test --workspace`；
  两次 render sha 一致；`visual-report` 全 0；**§4.4 载体声明**（离线对拍器）与运行时判据的分离记录。
- **风险**：strict 转红（这正是目的）；默认翻转过早会阻断流水线（故两步走）；基线更新需 Lead 批准；
  **回退**：门禁改动独立 commit 可 revert。

---

## 6. 边界：本轮可做 / 需用户确认（白名单）/ 需新 oracle 场景

### 6.1 本轮可做（分析产物，无需实现）
- 本方案（§1-§5）；C1-C11 复现命令；族归因与判据设计；
- 建议的 `data/naming/` 新增条目（**未写入**，待 Lead 派发）：`palette_record_base_scene0`（$7A:$7B=$BEFB，
  A 级描述符+字节）、`strip_window_base_font/cjk/graphics`（124/28/20，**B/待复核**：三样本来自自标
  待审稿 doc，E5）——按台账证据纪律逐条。

### 6.2 需 Lead/用户裁定事项（含**本轮已裁定**）

> **用户 2026-10-18 本轮裁定**：① **提供 Linux+Mesen2 采集环境**（P0-0/P0-4 可执行，见 §5.0 清单）；
> ② **撤销 W-C1/W-C4 白名单候选**；③ **批准门禁硬化**（`diff_visual.py` 0 容差语义，见 §5.6）。

**已裁定（无需再问，落地要求）**：
1. **W-C1/W-C4 撤销**：色彩空间/RGB 映射已一致（§3.2），残余=索引绑定 bug；`gates/visual_whitelist.json`
   保持不存在；l4-visual-plan §2 候选表更新由 spec-writer 执行并引用本裁定。
2. **门禁硬化（两步走）**：**已实现** `tools/diff_visual.py --strict`（任意外差异 ⇒ exit 1；task-21/`64b3a44`，
   默认行为不变）；**默认翻转**（pre-commit 的 visual 门禁切 `--strict`）留到 P0-6 全部 22 对清零后另卡
   （独立 commit + Lead 批准 + Evidence）；翻转前置=P0-0 基线冻结 + P0-6 全绿（§5.6）。
3. **环境**：Linux+Mesen2 已可执行 P0-0/P0-4；Windows 端只跑只读复算（C1-C11，需 Python）。

**仍需 Lead/用户裁定**：
4. **L4 判定载体（P1）**：确认"L4 判定对象=离线 `render-frame`，运行时渲染器另立像素判据"
   （§4.4）；共享渲染规则模块（`game-core` 纯函数或独立 crate）是否批准为结构变更；
5. **22 对样本集重构**：stg02 重采、四联帧拆分、ending 逐帧——改变 L4 样本面（P0-4 产出后需确认替换）；
6. **analysis 输入归档入库**（恢复 F-0-C 的备选）：`analysis/` 现为 gitignore 工作区；若要入库需定
   体积上限/最小切片政策（体积估算：单对投影写流切片估计 10^5 事件级×~40B ≈ 数 MB，22 对量级
   ~几十 MB，**待 P0-0 实测后定**；C 级估算）。若选 Linux 执行机路线则无需入库；
7. **D4 quirks**（8 sprites/scanline 等）是保真对象，**永不入白名单**（既有共识，终验保持）。

### 6.3 需新 oracle 场景/资产（Mesen2，环境已裁定提供）
| # | 场景/资产 | 目的 | 备注 |
|---|---|---|---|
| ① | stg02-start 正确 expected（候选 f3450=C 级假设 H4） | 替换 boot-black 复制品 | 先定 render 帧并与 expected 同帧；trigger=injected 时登记 |
| ② | stg06/stg08 逐关独立帧 | 拆四联冻结帧 | OBS-1 未解→自然档（bot）或注入档（选关 poke，injected 标注） |
| ③ | ending-1..5 独立帧 | 结束"同一 actual 比较 5 次" | 各帧独立 phase/写集 |
| ④ | 调色板 watch：`probe_mem 2006 2007` + watch `$D3-$DA/$161-$169/$0150-$015F` | 族②输入（$3F00 状态） | 现有 watch 面无 PPU 调色板 |
| ⑤ | NMI 上传 probe：`probe_mem 0130 016F` + `00d3 00da` | 族①写流 | 历史 fire-hold/gameplay 有，需 L4 各场景重采 |
| ⑥ | CHR bank probe：`probe_mem 8000 8001` | 族③窗基上下文键（C 级假设 H2 的证伪轨） | t154 方法（该 doc B/待复核） |
| ⑦ | `$7A/$7B/$8D-$90` 动态 trace | 族④真表基址/记录编码 | ending/shop 帧 |

### 6.4 需环境（**已裁定提供 Linux+Mesen2**）
- 用户已裁定提供 **Linux + Mesen2 2.1.1** 采集环境 → P0-0/P0-4 可执行（清单见 §5.0）。
- 本 Windows 工作区仍**不能**跑采集：`tools/scenario_run.py` **POSIX-only**（`fcntl` L34 +
  自管 Xvfb/DISPLAY，:467-477，t24 已弃 `xvfb-run` 包装；E7 修正）、`tools/mesen2/` 缺失（gitignore）。
  Windows 端职责=只读复算（C1-C11）+ `cargo build/test` + 文档。
- 备选（若 Linux 机不可长期用）：L4 最小 fixtures 入库（政策/体积裁定，§6.2-6）。

---

## 7. 复现命令清单（全部只读；python=本机 `python` 3.12）

> 脚本写 `%TEMP%`，不写仓库。`ROOT` 固定为仓库根。
> **平台标注（S10）**：C1/C2/C3/C5/C7/C8/C10/C11 = 纯 Python，**Windows 本机可跑**；C4 `git rev-parse` 跨平台；
> **C9 用 `grep` → 需 Git Bash/WSL**（PowerShell 等价：`Select-String -Path reference/seed/gunnac.asm -Pattern 'bank00:83A4\|…'`）。

**C1 D-total 22 对复算**
```python
# l4_c1.py：逐对 expected vs actual RGB 不等计数（与 tools/diff_visual.py 同口径）
import sys; sys.path.insert(0, "tools")
from diff_visual import decode_png
from pathlib import Path
R = Path("."); E = R/"oracle/screens/expected"; A = R/"oracle/screens/actual"
tot = 0
for ep in sorted(E.glob("*.png")):
    _,_,e = decode_png(ep); _,_,a = decode_png(A/ep.name)
    d = sum(1 for y in range(240) for x in range(256)
            if e[y][3*x:3*x+3] != a[y][3*x:3*x+3])
    tot += d; print(f"{ep.stem}: {d}")
print("TOTAL", tot, f"{tot/(22*61440):.4%}")
```
期望：逐项与 §2.1 表相同；TOTAL 280132 20.7247%。

**C2 N-3way（missing/spurious/color）**
```python
# l4_c2.py
import sys; sys.path.insert(0, "tools")
from diff_visual import decode_png
from pathlib import Path
E = Path("oracle/screens/expected"); A = Path("oracle/screens/actual")
c = {"missing":0,"spurious":0,"color":0}
for ep in sorted(E.glob("*.png")):
    _,_,e = decode_png(ep); _,_,a = decode_png(A/ep.name)
    for y in range(240):
        for x in range(256):
            ei = tuple(e[y][3*x:3*x+3]); ai = tuple(a[y][3*x:3*x+3])
            if ei == ai: continue
            if ai == (0,0,0): c["missing"] += 1
            elif ei == (0,0,0): c["spurious"] += 1
            else: c["color"] += 1
print(c, sum(c.values()))
```
期望：`{'missing': 262352, 'spurious': 6624, 'color': 11156}`，合计 280132。

**C3 F-family（empty/skip/present-blank/nt2）**
```python
# l4_c3.py：需 oracle/screens/actual/<name>.idx.json + .render.log 末条 scroll=(0,sy)
import sys, json; sys.path.insert(0, "tools")
from diff_visual import decode_png
from pathlib import Path
E = Path("oracle/screens/expected"); A = Path("oracle/screens/actual")
def sy_of(name):  # 从 .render.log 取该 name 最后一条的 scroll 高字节
    last = [l for l in (A/".render.log").read_text().splitlines() if l.startswith(name+" ")]
    return int(last[-1].split("scroll=(0,")[1].split(")")[0])
c = {"empty":0,"skip":0,"presblank":0,"nt2":0,"spurious":0,"color":0}
for ep in sorted(E.glob("*.png")):
    n = ep.stem; sy = sy_of(n)
    _,_,e = decode_png(ep); _,_,a = decode_png(A/ep.name)
    tiles = bytes.fromhex(json.loads((A/(n+".idx.json")).read_text())["tiles"])
    for py in range(240):
        for px in range(256):
            ei = tuple(e[py][3*px:3*px+3]); ai = tuple(a[py][3*px:3*px+3])
            if ei == ai: continue
            vy = py + sy
            if ai == (0,0,0):
                if sy != 0 and vy >= 240: c["nt2"] += 1
                else:
                    t = tiles[(vy//8)*32 + px//8]
                    c["empty" if t == 0 else ("skip" if (t == 8 or t >= 128) else "presblank")] += 1
            elif ei == (0,0,0): c["spurious"] += 1
            else: c["color"] += 1
print(c)
```
期望：`empty 130466 / skip 7907 / presblank 45359 / nt2 78620 / spurious 6624 / color 11156`
（empty+skip+presblank+nt2 = 262352 = missing）。注意：①视口映射只用 log 的 scroll，未与 trace zp 逐帧对齐（§8-H6）；
②**empty 桶的 A 级依赖 `RENDER_NT=0`/idx 未记 ntr/env**（E2）——P0-0 manifest 记录 env 后，命令须先断言
`ntr=0`（idx 仅存 `nt.tiles[ntr]`，`render_frame.rs:815/:1003-1008`）再升 A；否则本条为 B+。
③本节命令为 **Python 跨平台可跑**（Windows 本机可复算）。

**C4 dedupe（blob 同异）**
```bash
python -c "import hashlib,glob; [print(hashlib.sha256(open(p,'rb').read()).hexdigest()[:12],p) for p in sorted(glob.glob('oracle/screens/actual/l4_*.png'))]"
git rev-parse HEAD:oracle/screens/actual/l4_ending-1.png HEAD:oracle/screens/actual/l4_ending-5.png
git rev-parse HEAD:oracle/screens/expected/l4_stg02-start.png HEAD:oracle/screens/expected/title-menu_f00180_boot-black.png
git rev-parse HEAD:oracle/screens/expected/l4_stg06-late.png HEAD:oracle/screens/expected/l4_stg08-start.png
```
期望：ending-1/5 actual 同 blob（37aeef94…）；stg02/title expected 同 blob（d8d2f9ff…）；
stg06/stg08 expected 同 blob（065224fc…）；对应 sha256 前缀 65af76c9… / 788d6984… / 62a8ef06…。

**C5 基线可复现性证据（跨平台 python 版；旧 POSIX 写法需 Git Bash/WSL）**
```python
# l4_c5.py（Windows/Linux 均可）
import pathlib, subprocess
p = pathlib.Path('analysis'); print('analysis dirs:', [x.name for x in p.iterdir()] if p.is_dir() else 'MISSING')
print('MESEN', 'OK' if pathlib.Path('tools/mesen2').exists() else 'MISSING')
print('scenario_run L34:', open('tools/scenario_run.py', encoding='utf-8', errors='replace').read().splitlines()[33].strip())
print('Xvfb-managed:', 'Xvfb' in open('tools/scenario_run.py', encoding='utf-8', errors='replace').read().splitlines()[466])
print(subprocess.run(['git','log','-1','--format=%h %s','--','oracle/screens/actual/l4_ending-1.png'], capture_output=True, text=True, encoding='utf-8', errors='replace').stdout.strip())
```
期望：analysis 仅 `visual-scout/workset`（traces/phase 不在盘）；`MESEN MISSING`；
L34=`import fcntl`；Xvfb 行含 `"Xvfb"`（t24 自管，非 xvfb-run 包装）；
git 末条=`f3d5c43 chore(oracle): L4 战役渲染产物归档（签署态+实验面全存档）`。

**C6 异构证据（TBLDRV 非同态）**
```python
import json
A="oracle/screens/actual/"
t=lambda n: bytes.fromhex(json.load(open(A+n+".idx.json"))["tiles"])
c,e=t("l4_clear-inject"),t("l4_ending-1")
print("shared nonzero same-pos:", sum(1 for x,y in zip(c,e) if x==y and x!=0))  # 0
```

**C7 调色板域成员（族②）**
```python
# l4_c7.py
import sys; sys.path.insert(0, "tools")
from diff_visual import decode_png
from pathlib import Path
from collections import Counter
E = Path("oracle/screens/expected"); A = Path("oracle/screens/actual")
NES = {(0x66,0x66,0x66),(0x00,0x2A,0x88),(0x14,0x12,0xA7),(0x3B,0x00,0xA4),(0x5C,0x00,0x7E),(0x6E,0x00,0x40),(0x6C,0x06,0x00),(0x56,0x1D,0x00),(0x33,0x35,0x00),(0x0B,0x48,0x00),(0x00,0x52,0x00),(0x00,0x4F,0x08),(0x00,0x40,0x4D),(0x00,0x00,0x00),(0xAD,0xAD,0xAD),(0x15,0x5F,0xD9),(0x42,0x40,0xFF),(0x75,0x27,0xFE),(0xA0,0x1A,0xCC),(0xB7,0x1E,0x7B),(0xB5,0x31,0x20),(0x99,0x4E,0x00),(0x6B,0x6D,0x00),(0x38,0x87,0x00),(0x0C,0x93,0x00),(0x00,0x8F,0x32),(0x00,0x7C,0x8D),(0xFF,0xFE,0xFF),(0x64,0xB0,0xFF),(0x92,0x90,0xFF),(0xC6,0x76,0xFF),(0xF3,0x6A,0xFF),(0xFE,0x6E,0xCC),(0xFE,0x81,0x70),(0xEA,0x9E,0x22),(0xBC,0xBE,0x00),(0x88,0xD8,0x00),(0x5C,0xE4,0x30),(0x45,0xE0,0x82),(0x48,0xCD,0xDE),(0x4F,0x4F,0x4F),(0xC0,0xDF,0xFF),(0xD3,0xD2,0xFF),(0xE8,0xC8,0xFF),(0xFB,0xC2,0xFF),(0xFE,0xC4,0xEA),(0xFE,0xCC,0xC5),(0xF7,0xD8,0xA5),(0xE4,0xE5,0x94),(0xCF,0xEF,0x96),(0xBD,0xF4,0xAB),(0xB3,0xF3,0xCC),(0xB5,0xEB,0xF2),(0xB8,0xB8,0xB8)}
GOLD = {(0xFF,0xFE,0xFF),(0xFE,0x81,0x70),(0xFE,0xCC,0xC5),(0x00,0x40,0x4D),(0x12,0x2C,0x6E),(0x64,0xB0,0xFF)}
pairs = Counter(); n = 0; ein = 0; ain = 0
for ep in sorted(E.glob("*.png")):
    _,_,e = decode_png(ep); _,_,a = decode_png(A/ep.name)
    for y in range(240):
        for x in range(256):
            ei = tuple(e[y][3*x:3*x+3]); ai = tuple(a[y][3*x:3*x+3])
            if ei == ai or ei == (0,0,0) or ai == (0,0,0): continue
            n += 1; ein += ei in (NES|GOLD); ain += ai in (NES|GOLD); pairs[(ei,ai)] += 1
print(n, "in-table:", ein, ain)
for (e_,a_),k in pairs.most_common(12): print(e_, "->", a_, k)
```
期望：`11156 in-table: 11156 11156`；Top=`(255,254,255)->(0,64,77) 3239`、`(0,64,77)->(255,254,255) 2269`、
`(254,204,197)->(255,254,255) 1680`、`(254,129,112)->(255,254,255) 775`、`(181,49,32)->(255,254,255) 600`…

**C8 3B 窗/双字节表 ROM 复核**
```bash
python -c "r=open('rom/gunnac_cn.nes','rb').read(); print(r[0xBF0B:0xBF0B+32].hex(' '))"  # bank5:$BEFB 32B 数据
python -c "r=open('rom/gunnac_cn.nes','rb').read(); print(r[0xBF1B:0xBF1B+16].hex(' '))"  # bank5:$BF0B/$BF13 双表
python -c "r=open('rom/gunnac_cn.nes','rb').read(); print(r[0x17E7:0x17E7+8].hex(' '))"   # bank00:$97D7 描述符项0
# 项 1-4 基址在对应窗口下的字节（非数据表 ⇒ 需按场景 r7 换 bank；$BB4E/$BFFB 落 $A000 窗、$C0D2 落 $C000 窗）：
python -c "r=open('rom/gunnac_cn.nes','rb').read(); print('BB4E:',r[0xBB5E:0xBB5E+16].hex(' ')); print('BFFB:',r[0xC00B:0xC00B+16].hex(' ')); print('C0D2:',r[0xC0E2:0xC0E2+16].hex(' '))"
```
期望：`10 80 f0 10 f0 10 80 f0 30 30 30 88 88 e0 e0 e0 22 22 22 41 41 23 23 07 04 04 04 05 05 06 06 07`；
`22 22 22 41 41 23 23 07 04 04 04 05 05 06 06 07`；`2e 2c fb be 09 18 ff 97`；
项 1-4=`05 a9 03 9d 34 05…`（$BB4E）/`ff ff ff ff ff a9 93 9d c4 04…`（$BFFB）/`20 39 d2 bd 34 05…`（$C0D2，code 字节，非表）。

**C9 ROM 指令指针核验（seed）**
```bash
grep -n 'bank00:83A4\|bank00:84E6\|bank00:85A9\|bank00:85E8\|bank00:8FA5\|bank00:9785\|bank00:8105' reference/seed/gunnac.asm
grep -n 'bank15:F040\|bank15:F04B\|bank15:F083\|bank15:F09A\|bank15:F105\|bank15:F14B\|bank15:E6DF\|bank15:E7DC\|bank15:E828' reference/seed/gunnac.asm
```
期望：seed L492/L662/L765/L797/L2009/L2972/L145；L89377/L89382/L89405/L89416/L89460/L89487/L88116/L88255/L88295。

**C10 r4b_stream_parse 取 bank 旁发现（A）**
```bash
python -c "r=open('rom/gunnac_cn.nes','rb').read(); print('fn:', r[0x632F:0x632F+16].hex(' ')); print('true:', r[0x232F:0x232F+16].hex(' '))"
```
期望：`fn: 00 40 23 81 42 26 0a 09…`（非 R-4b 流）vs `true: 6d 22 0e 0f 20 20 0e 0f fe 8d 22…`
（R-4b-4 规则 1：段首 lo,hi=`6d 22`→PPU `$226D`，与 `l4-render-support.md` R-4b "$A31F `6D 22`=$226D" 一致）。

**C11 独立 12 对 / 非独立 10 对族内拆分（S1+S2 口径，A 级）**
```python
# l4_c11.py（跨平台）
import sys, json; sys.path.insert(0, "tools")
from diff_visual import decode_png
from pathlib import Path
E = Path("oracle/screens/expected"); A = Path("oracle/screens/actual")
NON = {"l4_ending-1","l4_ending-2","l4_ending-3","l4_ending-4","l4_ending-5",
       "l4_stg02-start","l4_stg06-late","l4_stg08-combat","l4_stg08-late","l4_stg08-start"}
def sy_of(name):
    last = [l for l in (A/".render.log").read_text().splitlines() if l.startswith(name+" ")]
    return int(last[-1].split("scroll=(0,")[1].split(")")[0])
ind={"f1":0,"f3":0,"f2":0,"spur":0}; non={"f1":0,"f3":0,"f2":0,"spur":0}
for ep in sorted(E.glob("*.png")):
    n = ep.stem; sy = sy_of(n)
    _,_,e = decode_png(ep); _,_,a = decode_png(A/ep.name)
    tiles = bytes.fromhex(json.loads((A/(n+".idx.json")).read_text())["tiles"])
    tgt = non if n in NON else ind
    for py in range(240):
        for px in range(256):
            ei = tuple(e[py][3*px:3*px+3]); ai = tuple(a[py][3*px:3*px+3])
            if ei == ai: continue
            if ai == (0,0,0) and ei != (0,0,0):
                vy = py + sy
                if sy != 0 and vy >= 240:
                    tgt["f1"] += 1                      # nt2 域（同①推定）
                else:
                    t = tiles[(vy//8)*32 + px//8]
                    tgt["f1" if t == 0 else "f3"] += 1
            elif ei == (0,0,0): tgt["spur"] += 1
            else: tgt["f2"] += 1
print("independent 12:", ind, "sum", sum(ind.values()))
print("non-independent 10:", non, "sum", sum(non.values()))
```
期望：independent `f1 137263 / f3 45514 / f2 6814 / spur 960`（合计 190,551）；
non-independent `f1 71823 / f3 7752 / f2 4342 / spur 5664`（合计 89,581；spur 含 ⑤ 902，故余项非独立=5,664−902=4,762）；
两组合计 280,132。

---

## 8. 假设区（C 级——禁止入实现/禁止写入结论/台账）

> **等级一致性（S8 修正）**：H2（tile_page 上下文键）在本方案正文统一为 **C 级假设**
> （§3.3 差什么①、§5.2 第 1 条均引用 §8-H2）；其支撑数据面（R0 三配置）来自自标待审稿 doc
> （`chr-org-timing.md`）=**B/待复核**，不得因数据面为 B 而把假设升 B。

| # | 假设 | 为何是 C | 证伪路径 |
|---|---|---|---|
| H1 | 3B 记录=RGB 三元组（值超 6bit 色域） | 16B 数据区不被 3 整除；无动态消费样本；96B 窗已证超读 | `$0150/$7A/$7B` trace + 渲染对照 |
| H2 | tile_page 上下文键 = CHR R0 写流终值 | R0 三配置数据面 B/待复核，且"当前上下文=哪个 R0"无直接证据 | 按场景 `probe_mem 8000 8001` + 逐帧对照 |
| H3 | 码≥128 段 = CJK/参数混合族（`strip-family-direct` §2 候选） | 码段语义仅分布观测，未逐族定谳；该 doc B/待复核 | 800 级码值逐族 CHR 反查（t147 方法） |
| H4 | stg02-start 应为 f3450 关名屏 | expected=boot-black（A），但"正确帧"未证（渲染帧 3404 vs 锚 3450 不一致） | 重采 f3450 + 像素对照 |
| H5 | 四联帧的"正确"逐关内容帧可由选关 poke 复现 | OBS-1 未裁；poke 场景=injected，不能冒充自然可达 | 八关自然链 bot 或显式 injected 登记 |
| H6 | 剩余 present-blank 中有部分属 scroll/视口映射误差而非 CHR 窗基 | 本方案视口映射用 log 的 scroll，未与 trace zp 逐帧对齐 | P0-1 后重算 idx-vs-像素对照 |

---

## 9. 自查、修订映射与给复核方的证伪清单

**自查（本修订稿）**
- 只写本文件；未改 `crates/`、`data/`、`gates/`、`rom/`、`reference/`；未 git add/commit；未跑 visual 门禁
  （不写 `oracle/screens`）；未改任何实现/规格文档。
- 数字均可由 §7 C1-C11 独立复算（本轮已复跑 C1/C2/C3/C5/C7/C11）；ROM 字节指针已用 `reference/seed/gunnac.asm`
  行号/hexdump 逐条复核；旧账失效项已 dated 标注（§1.2），未删除历史主张。
- **【R7 口径】"数据面未动"指数值/期望值未变**（非逐字符）：与 task-8 版相比，C3 增 E2/S10 要求的
  注记（"ntr=0 断言/nt2 桶 B 级"）、C5 改为跨平台 python——命令与期望值不变，数值已独立复跑同值
  （C1/C2/C3/C5/C7/C11）；C11 为新增（数值由 C3 桶聚合，非新测量）。
- A/B/C 分级（本轮按复核 E2/E3/E5 收紧）：像素量/颜色域/ROM 字节/代码行=A；空单元归属（依赖 RENDER_NT）、
  根因映射、待审稿 doc 数据面=B/待复核；3B 编码/上下文键/正确帧=C（§8）。
- 本方案**未**把"差异已分类/数字下降"写成"0 容差达成"；**未**把离线对拍器结果写成运行时显示一致（§4.4）。
- 用户本轮裁定已纳入：环境提供（§5.0/§6.4）、W-C1/C4 撤销（§3.2/§6.2）、门禁硬化（§5.6）。

### 9.1 复核 findings 修正映射（task-8 → 本修订稿）

| finding | 级 | 修正落点（本文件） | 修正内容 |
|---|---|---|---|
| **S1** | 高 | §0 结论3、§2.5/§2.6、§4.3 | 统一为 **12/22 独立对 + 10/22 需重采/重渲**；删"17/22"与"G1 11 帧"；§2.6 逐对 provenance 表 |
| **S2** | 高 | §4.2 | 分步判据改**族内 + 独立 12 对**计数（①137,263/③45,514/②6,814/余项960）；删 total-D≤25,000/15,000/500；给非独立 10 对拆分并交 P0-4 |
| **P1** | 高 | **§4.4 新增**；§5.6（判据）、§6.2-4（裁定/前置项）；§9-P1 证伪点 | L4 判定载体=离线 render-frame 声明 + 运行时另立判据 + 共享渲染规则模块路线 + Android 前置（R4：原写"§5.0/§5.3 交叉引用"有误，§5.0/§5.3 无 P1 引用） |
| **E1** | 中 | §3.1 前提、§5.1 前置 | 族①根因绑定 `FLAGUPLOAD` env（:694 关闭 pending_row 路）；P0-0 manifest 先行 |
| **E2** | 中低 | §3.0/§3.1、§7-C3 | empty-cell 130,466 A→**B+**（RENDER_NT/ntr 未记录）；C3 先断言 ntr=0 |
| **E3** | 中低 | §2.4 F-0-D | "22 对异构"降 **B**（C6=0 只否定同 TBLDRV 态，不证明异 env） |
| **E4** | 中低 | §2.4 F-0-B 后注 | 删除 `cf6847b` 自述证据，标注其与提交物矛盾（0 vs 902）；F-0-B 由 blob/解码独立支撑 |
| **E5** | 中低 | §1.2、§3.0、§3.2、§3.3 | 4 篇自标"待审稿"doc 降 **B/待复核**；A 级只留可独立复导字节面 |
| **E6** | 低 | §1.2、§3.5、§5.4 | "902=文本内容本身差"改指 `l4-visual-plan.md:349-350@1be66e5`；"§4 五缺口=五族"映射改写 |
| **E7** | 低 | §2.4 F-0-C、§6.4 | 改"POSIX-only（fcntl + 自管 Xvfb/DISPLAY，t24 已弃 xvfb-run 包装；:467-477）" |
| **S4** | 中低 | §5.0-2 | render_frame 加 `--out-dir/--log-path`（现写死 :999-1011）；`actual_canonical/` 回退 |
| **S5** | 低 | §3.2 差什么②、§5.3 | ink_table 已二维；P0-3 真改动=删 GOLDPAL/BG family 选择/Graphics DARK_BLUE 裁量 |
| **S6** | 低 | §5.4 | f3450 标 **C 级假设 H4**；先定 render 帧并与 expected 同帧 |
| **S7** | 低 | §4.2 P0-2、§5.2 判据 | 删"ending skip 4,705 由 CJK 窗基承接"；ending 交 P0-4 逐帧重渲 |
| **S8** | 低 | §3.3/§5.2/§8 | 上下文键统一 **C 级假设（§8-H2）**；数据面 B/待复核 |
| **S9** | 低 | §4.1 P0-0 行、§5.0 | P0-0 前置列写"环境已裁定提供；未完成则 P0-1..全部 BLOCKED" |
| **S10** | 低 | §5.6、§7-C3/C5、§3.1 | diff_visual 口径改引两种模式（`--strict` 验收；默认=分类账）；linebuf 测试改 :6-53；C5 改跨平台 python；行号漂移见 §9.2-R5 |
| **P2** | 低 | §6.2-6、§6.4 | fixtures 入库政策列裁定项（体积估算 C 级待实测）；C5 跨平台化 |
| **P3** | 低 | §5.4 | 自然/注入两档采集清单（注入最小集 10 帧，trigger=injected） |

### 9.2 task-24 复审残留（R1-R8）折入映射

> 来源：`adversarial-review-l4-attack-plan.md` 复审 §7（2026-10-18，verdict=pass；R1/R2 建议提交前折入）。
> 本稿已折入 **R1/R2（必改）+ R3/R4/R5/R6/R7**；**R8 不在本卡**（tools 报告模板 W-C1 文案属 task-20/21）。
> 落点以**节锚**为准（行号随后续编辑漂移，不逐字保证）。

| # | 级 | 落点（本文件·节锚） | 折入内容 |
|---|---|---|---|
| **R1** | 中 | §4.2 P0-6 行；§5.6（两种模式/两步走） | 验收判据改 **`python tools/diff_visual.py --strict` exit 0**（L4 0 容差判据，task-21/`64b3a44` 已实现；默认模式=分类账口径会假绿不得验收）；**默认/pre-commit 翻转留到 P0-6 全绿后另卡**（避免未清零恒红阻断 P0-1..）；`gates/cli.py visual` 翻转前≠0 容差 |
| **R2** | 中低 | §4.2 P0-1 行 | 补独立对 **nt2 域 78,620→≤8,000** 子目标（承接=`$2006/$2007` 默认投影+4-NT 持久化；nt2 现 B 级推定，P0-0 后按实测重算）；nt0 58,643→≤2,000 与 nt2 子目标之和=10,000 与总支一致 |
| **R3** | 低 | §0 结论3 | 加"§0/§4.1=22 对全量上界；分步验收用 §4.2 独立 12 对口径"交叉引用 |
| **R4** | 低 | §9.1 P1 行 | P1 落点更正为 **§4.4 + §5.6 + §6.2-4**（删"§5.0/§5.3 交叉引用"——该两节无 P1 引用） |
| **R5** | 低 | §3.1（nt2 量）、§5.1（idx 持久化）、§9-P1 证伪点 | 行号漂移：idx 写入 `:1003-1008` + `RENDER_NT` `:815`；`shell-win/lib.rs FrameRenderer` = HEAD :594（分析时点 :557）；`game-shell/render.rs` 用内容锚 |
| **R6** | 低 | §4.4 声明 3 | 共享渲染规则重构**须在 P0-0 后**、以 canonical/manifest 判据验证；结构变更需 Lead/用户批准 |
| **R7** | 低 | §9 自查 | "数据面未动"精确化为**数值/期望值未变**（C3 增注、C5 跨平台化；C11 新增为桶聚合非新测量） |
| **R8** | 低 | 不在本卡 | `tools/diff_visual.py` 生成报告模板的 W-C1 文案 → 由 **task-20/21** 收口（生成物属 tools，需先报 Lead）；不影响当前 gate |

**给复核方（task-8/task-24 复审）优先证伪点**
1. S1 的 **12+10=22** 分类是否与 §2.6 provenance 表自洽（独立对 12 个、复用/需重采 10 个）；
2. S2 的族内/独立 12 对判据是否算术可达（①137,263→≤10,000，含 nt2 子目标 78,620→≤8,000；
   ③45,514→≤5,000；②6,814→0；余项 960→≤200；且各步只在本步族内判、无 total-D 阈）；
3. P1 的运行时渲染器事实（`game-shell/src/render.rs`（v1 占位，内容锚；行号按分析时点 :1-22）、
   `shell-win/src/lib.rs` 的 `pub struct FrameRenderer`（HEAD :594；分析时点 :557）、
   `game-platform::Renderer`）是否成立；§4.4 声明是否足以消除"对拍器≠运行时"裂缝；
4. E1/E2/E3/E5/E6 的降级/改指是否到位（无残留 A 级超标/错引）；
5. 用户三项裁定（环境/W-C1-C4 撤销/门禁硬化）是否都落到可执行位置（§5.0/§6.2/§5.6）；
6. 旧账引用是否仍有失效未标（`l4-final-localization-ending1`、`residual-pixel-ledger` AR2-A）；
7. **R1/R2 折入核**：§4.2 P0-6/§5.6 是否统一用 `--strict`（默认翻转后置 P0-6 后）、P0-1 是否含
   nt2 子目标且与总支 10,000 一致；R3-R7 落点见 §9.2；R8 归 task-20/21（不在本案）。

【2026-10-18 初版】decomp-analyst，HEAD `1be66e5`，未完成实现。
【2026-10-18 修订稿（第二轮）】decomp-analyst，按 task-8 `adversarial-review-l4-attack-plan.md` findings
（S1/S2/P1/E1-E7/S4-S10/P2/P3）修正 + 纳入用户裁定（环境/W-C1-C4 撤销/门禁硬化）。**未提交**。
【2026-10-18 修订稿（第三轮，task-26）】decomp-analyst，按 task-24 复审 pass 的残留折入 **R1/R2（必改）
+ R3/R4/R5/R6/R7（顺手）**；映射见 §9.2；R8 归 task-20/21。**docs-only；未提交**。
