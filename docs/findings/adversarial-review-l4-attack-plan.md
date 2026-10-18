# 对抗复核：L4 0 容差攻坚方案（task-8）——证伪立场

> 复核方：platform-verifier（verifier 角色；被审方 l4-analyst 与之无共享上下文，独立性成立）
> 日期：2026-10-18
> 被审对象：[l4-zero-tolerance-attack-plan.md](l4-zero-tolerance-attack-plan.md)
> （665 行；sha256 `23a3a978c8564ace…`；mtime 12:57；复核时 l4-analyst 已 inactive=定稿；
> 方案自报分析 HEAD=`1be66e5`）。
> 上游裁定：[decision-l4-zero-tolerance-ruling](../knowledge/decision-l4-zero-tolerance-ruling.md)、
> commit `24093be`（L4=白名单外像素 100% 一致；"带声明签署"不构成达成）。
> 复核形态：**默认证伪**。C1-C10 数字全部由复核方独立重写脚本重算（未直接执行被审方脚本）；
> ROM/seed/git/代码指针逐条重导；不复用被审方结论。
>
> **复核边界**：复核期间 `docs/spec/l4-signing-criteria.md` 与 `docs/spec/l4-visual-plan.md`
> 有 spec-writer（task-13）未提交改动；本报告对方案引用行的核验一律以**方案自报 HEAD `1be66e5`**
> 为准，并显式标注该时点。被审方案文件本体（untracked）在复核期无改动（sha 复核对齐）。

---

## 0. Verdict

**verdict = needs_revision**

- **数据面全部通过**：C1-C10 十组数字/字节/行号**逐项精确复现**；五族像素量、F-0 的
  blob/输入缺失/日志口径等核心事实**全部成立**；代码级指针（render_frame/tile_page/nmi_flush/
  r4b 取错 bank）**逐条属实**；l4-final 旧账/AR2-A 撤销/#103 pass 的时效处理基本正确。
- **不通过的是"判据/账目/边界"**：
  1) **0 容差账目自相矛盾**（§0「17/22 可闭环」vs §4.3「10 对需换资产」vs「G1 11 帧」三套数；
     按方案自己的判据严格计=**12/22 独立对**、**10 对**需重采/重渲）；
  2) **§4.2 分步 total-D 判据算术不可达**（P0-1 的 total≤25,000 在只修族①时下界=71,046 等）；
  3) **L4 判定载体未声明**：P0-1..P0-3 全部改 `game-desktop` 离线对拍器，运行时可玩渲染是
     `game-shell`/`shell-win` 另一套代码、Android 还要再换 → "对拍器 0 容差 ≠ 游戏显示 0 容差"的
     裂缝未声明；
  4) 证据等级超标（C3 的 A 级依赖未记录 env；F-0-D 异构=推断；三篇自标"待审稿"doc 被升为"现行 A 级"）；
     证据引用错位（"902=文本内容本身差"在 l4-visual-plan 而非 l4-signing-criteria §4.5；
     cf6847b 自述与其提交物矛盾）。
- 方案**方向与顺序仍是有价值的**（P0-0 基线冻结先行、P0-1 写流优先有量化依据）；
  修正后可作为实施依据。**修完本节 findings（docs-only 级为主）+ 复审 → 可 pass**。

---

## 1. 我执行的独立重算（数据面：逐项通过）

> 环境：Windows / pwsh / `python` 3.12（仓库约定）。脚本自写（不复用被审方 C1-C10 代码），
> 只 import 既有 `tools/diff_visual.py::decode_png` 做 PNG 解码。

### 1.1 C1 22 对差异（独立双层循环：逐字节 RGB 不等）

```text
gameplay-stage1_f01260_stage-start: 1019    gameplay-stage1_f01500_mid-combat: 1490
gameplay-stage1_f02349_end: 1753            l4_clear-inject: 58101
l4_combat-variety: 27962                    l4_ending-1: 4552 / -2: 3682 / -3: 3347
l4_ending-4: 3981 / -5: 3981                l4_fade-clear: 59371
l4_late-combat: 1579                        l4_shop-window: 17454
l4_stg02-start: 902                         l4_stg03-start: 1205
l4_stg06-late: 17284                        l4_stg07-combat: 9627 / -start: 10990
l4_stg08-combat/late/start: 17284×3         title-menu_f00180_boot-black: 0
TOTAL 280,132 / 1,351,680 = 20.7247%；同点 79.2753%；逐像素相等 1/22；<7680 的帧 12/22
```

与方案 §2.1、`oracle/screens/visual-report.md` **逐项全同**。

### 1.2 C1b/C2：几何分桶与像素性质三分

```text
in_strip(y<16)=13,246 / out_strip(y>=16)=266,886（=visual-report 的 4.73%/95.27% 对照）
N-3way：missing 262,352 / spurious 6,624 / color 11,156；合计 280,132
```

### 1.3 C2b：G1-G5 分组（按方案 §2.3 的帧归属独立求和）

| 组 | D-total | missing | spurious | color |
|---|---|---|---|---|
| G1（12 帧） | 255,425 | 246,860 | 2,952 | 5,613 |
| G2（ending 5 帧） | 19,543 | 14,003 | 2,590 | 2,950 |
| G3（gameplay 3 帧） | 4,262 | 1,489 | 180 | 2,593 |
| G4（stg02） | 902 | 0 | 902 | 0 |
| G5（title） | 0 | 0 | 0 | 0 |

与方案 §2.3 **完全一致**。

### 1.4 C3 F-family 六桶（按"实际 nametable 码"二分）

```text
empty 130,466 / skip 7,907 / presblank 45,359 / nt2 78,620 / spurious 6,624 / color 11,156
empty+skip+presblank+nt2 = 262,352 = missing ✓；六桶合计 280,132 ✓
```

与方案 §2.2/§3.0 期望值**完全一致**（含各帧分布：stg03 color 1,001 / stg02 全 spurious 902 /
fade-clear 全 missing 59,371 / ending empty 7,003+skip 4,705+presblank 2,295 等）。

### 1.5 C4：blob 去重与 git 对象

```text
unique actual（22 对内）：15 个；DUP：l4_ending-1..5 = sha256 65af76c95e1132d0…
                       DUP：l4_stg06-late/l4_stg08-{start,combat,late} = db7262bcdd0c5592…
unique expected：17 个；DUP：stg02=title（788d6984576b…，均 258B）
                       DUP：stg06/stg08 四帧（62a8ef06c4b9…，均 6,074B）
                       DUP：ending-4/5（6ff345dc5c3f…，均 4,154B）
unique (expected,actual) 比较：18 对；去重合计 224,299 /（18×61,440=1,105,920）= 20.2817%
git rev-parse HEAD: actual/ending-1 = ending-5 = 37aeef94…；expected/stg02 = title = d8d2f9ff…；
expected/stg06 = stg08×3 = 065224fc…；expected/ending-4 = ending-5 = e4eb467f…
```

与方案 §2.4 F-0-A/F-0-B、§2.5 **完全一致**（含 PNG 字节 sha256 前缀与 git blob 两个口径）。

### 1.6 C6/C7：异构旁证与调色板域

```text
C6：clear-inject tiles 非零 379 / ending-1 非零 159；同位置"非零且相等"= 0
C7：color 差异像素 11,156；expected 与 actual 颜色 100% 落在 render_frame.rs:19-28 的 64 色 P 表
    （P 表 64 项/54 唯一色）——比方案 C7 的 NES∪GOLD 集合更强的检验亦成立
    Top 对：白→暗青 3,239 / 暗青→白 2,269 / 粉→白 1,680 / 鲑→白 775 / 红→白 600 / 淡紫→白 510 / 浅蓝→白 427
    白↔暗青 5,508=49.4%；彩色→白 3,992=35.8%
```

### 1.7 C8/C9/C10：ROM 与 seed 指针

```text
file[0xBF0B:+32] = 10 80 f0 10 f0 10 80 f0 30 30 30 88 88 e0 e0 e0 22 22 22 41 41 23 23 07 04 04 04 05 05 06 06 07 ✓
bank05:$BF0B（file 0xBF1B）=22 22 22 41 41 23 23 07；$BF13（file 0xBF23）=04 04 04 05 05 06 06 07 ✓
描述符项 0（file 0x17E7）=2e 2c fb be 09 18 ff 97 ✓；项 1-4 窗内字节与方案 C8 期望逐字节一致 ✓
PALETTE_RECORDS[96] == ROM file[0xBF0B:+96]（True）；!= file[0xBEFB:+96]（两者差 = iNES 16B 头，方案 +0x10 换算正确）
C9：bank00:83A4/84E6/85A9/85E8/8FA5/9785/8105 = seed L492/662/765/797/2009/2972/145 ✓
    bank15:F040/F04B/F083/F09A/F105/F14B/E6DF/E7DC/E828 = L89377/89382/89405/89416/89460/89487/88116/88255/88295 ✓
C10：file[0x632F]=00 40 23 81…（非流）/ file[0x232F]=6d 22 0e 0f 20 20 0e 0f fe 8d…（真 R-4b 流）✓
```

### 1.8 族归因抽查（≥5 族；证据等级复核结论）

| 族 | 方案数字/归因 | 我的复算/重导 | 等级判定 |
|---|---|---|---|
| ① nametable 内容空 | 209,086（empty 130,466 + nt2 78,620） | C3 精确复现；`nmi_flush` 模型（terrain.rs:301-339，4 组×8B/+$0F/+$20）与 `pending_row` 单行代码（render_frame.rs:565/:694-715/:784-801）均属实 | 像素量 A；**empty-cell 的 A 级依赖未记录 env（见 E2）→ 实际 B**；nt2 B；"丢行"根因 B（且未绑定签署 env，见 E1） |
| ② 调色板索引绑定 | 11,156；Top 对如上 | 全部颜色在 P 表内；Top 对与方案一致 | 颜色域 A；"索引绑定而非 RGB 映射"B+/A-（boot-black 0 差只证黑场，真正支撑=全色域成员） |
| ③ 写进但渲不出（tile_page） | 53,266=presblank 45,359+skip 7,907 | C3 精确复现；tile_page 缺省 124（:894）、只在文本回放更新（:773）、码 8/≥128 跳过（:851-853）、raw-95 拟合（:856）全部属实 | 像素量 A；根因 B；三样本证据来自 3 篇**自标"待审稿"**doc（见 E5） |
| ④ 3B 表残余 | 0 直接像素（默认路不用） | PALETTE_RECORDS==ROM 96B；rec_pal 只在 RECPAL 分支使用（:921-935）；RECP3/RECPAL/TRIHOP/FUSE 均 env 逃生口（:344/:368/:393/:930/:935） | 结构 A；编码 C——与方案分级一致 |
| ⑤ stg02-start | 902 全 spurious；expected=black 副本 | expected 258B **全黑**（0 非黑像素）；actual 246,083B/非黑 902px；blob 全等 ✓ | A（结论成立） |
| 余项 spurious | 5,722 | C2/C3 复现（G1 2,952+G2 2,590+G3 180） | 计数 A；根因 B |

### 1.9 代码/文件指针核验（方案 §1.3/§3/§5 抽查）

- 任务卡写的 `crates/game-desktop/src/render_frame.rs` **不存在**；实际 `src/bin/render_frame.rs`（1,036 行）✓；
  `render_rules.rs` 124 行 ✓（`family_of`:27-37 / `family_bank`:47-54 / `ink_table`:58-73）。
- `terrain.rs:301-339 Nametable::nmi_flush`（4 组×8B、+$0F、+$20、attr 路）✓；`:342 block_palette` ✓。
- `lib.rs:915-948 r4b_stream_parse`：`base=0x10+2*0x2000+(stream_addr-0x8000)` 公式 ✓；
  R4B 只被 `render_frame.rs:150-176` 诊断路调用 ✓；$A31F→file 0x632F（非流）与真流 file 0x232F ✓。
- `phase_batch.rs` 只回放 `$2F/$30` 指针驱动的文本装配写集（无舞台 BG 写流）✓（与方案 §3.1 "A 级代码事实"一致）。
- `l4-render-support.md` R-1（$DA4C/9B=`00×7,C1,C1`、file 0x1DA5C、seed L86602 ✓）、
  R-2（$97D7 8B/项 + 5 项基址 `$BEFB/$C0D2/$BB4E/$BB4E/$BFFB` + 3B stride/B 级编码/C 级止步 ✓）、
  R-6（上传通道/位域改写器三表/3B 数据源 三件套 ✓）。
- `render_frame.rs` 的 `.render.log` 只写 `name/frame/base/scroll/nt_nz/probe`（:1011），
  `idx.json` 只写 `palette_src/attrs/tiles`（:1003-1008）——方案 F-0-C 成立。
- 场景/素材 pointer：`bug-ledger.md` OBS-1 ✓、`tas-bot-eight-stage-input-plan.md` ✓、
  `oracle/scenarios/README.md` 秘籍表 `poke 0180` 选关 ✓、`gs5-6-dynamic-replay.md` stg02-start 锚
  f3450（injected）✓、`adversarial-review-98.md` 同锚 ✓。

---

## 2. F-0 与旧账时效核验（被确认清单）

| 项 | 方案主张 | 我的核验 | 结论 |
|---|---|---|---|
| F-0-A actual 同 blob | ending-1..5 / stg06-late+stg08×3 同 blob | sha256 + git blob 双口径全部相等；`.render.log` 末条 ending 均 `nt_nz=159` | ✅ 成立 |
| F-0-B expected 错配 | stg02=title 复制品；4 联同文件；ending4/5 同文件 | git blob/sha256/尺寸全对；stg02 expected 解码=**全黑**（0 非黑像素），actual=关名屏 902 非黑 | ✅ 成立（但见 E4 的引用问题） |
| F-0-C 输入不可复现 | analysis/traces|phase 缺、mesen2 缺、scenario_run POSIX、log 无 env/args | 全部属实（`analysis/` 仅 visual-scout/workset；`import fcntl` L34；`.render.log` 6 字段） | ✅ 成立 |
| F-0-D 诊断态混记 | 签署态 env TBLDRV/GOLDPAL/FAMMAP/FLAGUPLOAD/ROWMAP（l4-visual-plan:377@1be66e5）；TBLDRV 读 golden 表 | 代码事实全部属实（:127-149 读 doc、FAM_MAP :40-71、GOLDPAL :940-943） | ✅ 代码事实成立；但"22 对异构"为推断（E3） |
| 旧账 l4-final-localization-ending1 | 7753 态残差 7380px/261 格，时点失效 | 文件标题即该值；现行 ending-1 diff=4552 | ✅ 处理正确 |
| 旧账 residual-pixel-ledger | AR2-A 撤销、AR2-C 更正 | 文件 :57-60 有 AR2-A 撤销注；:72 复现命令已改 0x10C0D（AR2-C 更正） | ✅ 处理正确 |
| 旧账 adversarial-review-103 | pass（F-A103 已修） | #103 C.2 改判 pass；F-A103=96B 窗 32B 数据+64B 码流，真表 $BF0B/$BF13（$4D 索引） | ✅ 正确 |
| 旧账 l4-visual-plan L375-382 | 过程里程碑，不构成 L4 达成 | 1be66e5 版 :375-382 正是"转正 5/5/战役收官"，用户裁定已覆盖其完成语义 | ✅ 正确 |
| 任务卡路径 | `src/render_frame.rs` 不存在，真实在 `src/bin/` | 属实（见 §1.9） | ✅ 正确 |

**旧账时效结论**：方案对"失效旧账不引用/不据以定谳"的纪律执行基本到位（仅 E6 一处引用归错文档）。

---

## 3. Findings 四分类

> 严重度：高=阻塞方案作为实施依据；中=会使实施跑偏/判据失效；中低/低=文档与引用精度。
> 行号=被审方案（665 行版）行号。

### 3.1 证据不足（Evidence insufficiency）

| ID | 级 | 位置 | 问题与证据 | 修正建议 |
|---|---|---|---|---|
| **E1** | 中 | plan:163-164（§3.1 代码偏离）、:295-303（§4.1 顺序）、:350-353（§5.1）；对照 [render_frame.rs:692-694](../../crates/game-desktop/src/bin/render_frame.rs#L692-L694)、[l4-visual-plan.md:377@1be66e5](../../docs/spec/l4-visual-plan.md) | **§3.1 的族①根因未绑定生成 actual 的实际 env**：方案 F-0-D 自己指出签署态 env 含 `FLAGUPLOAD=1`（l4-visual-plan:377@1be66e5），但 `FLAGUPLOAD` 存在时 `pending_row` 上传路被**关闭**（render_frame.rs:694 `if var("FLAGUPLOAD").is_err() { if let Some(pending_row)... }`），改走 flag 事件直写路（:301+）。因此"单 pending_row/每帧一行丢行"可能不是生成签署 actual 的路径 → P0-1 的改动面与根因在当前 env 证据下**未经确认** | P0-1 前必须由 P0-0 产出**逐对 (scene,frame,env) manifest**，逐对确认是否走 FLAGUPLOAD/probe/$2007/文本回放；根因表述改为"默认路（FLAGUPLOAD off）代码偏离"并注明签署态待 P0-0 确认 |
| **E2** | 中低 | plan:165-166（"nt0 empty-cell 130,466 A（码=0）"）、:553-554（C3 注释）；对照 [render_frame.rs:815](../../crates/game-desktop/src/bin/render_frame.rs#L815)、[:1003-1008](../../crates/game-desktop/src/bin/render_frame.rs#L1003-L1008)、actual/`*.idx.json` | `idx.json` **不记录 `ntr`（nametable 号）也不记录 env**：实际 22 个 idx 文件字段只有 `palette_src/attrs/tiles`；tiles 来自 `nt.tiles[ntr]`，`ntr` 由 `RENDER_NT` env 决定（默认 0）。在 F-0-C"命令/env 不可从产物反推"的前提下，"empty-cell=nt0 码 0"的 **A 级标签不成立**（应为 B：默认 ntr=0 假设下成立）；C3 的 nt2 桶同理（B，方案已标 B） | 将 §3.1 的 "empty-cell 130,466 A" 改为 "A（像素量）/B（nt0 归属依赖默认 RENDER_NT=0，P0-0 记录 env 后升 A）"；C3 命令改为先断言 idx/manifest 的 ntr=0 |
| **E3** | 中低 | plan:118-121（F-0-A~D）、:573-580（C6） | F-0-D 用"C6：clear-inject 379 码与 ending 159 码同位置非零交集=0"推"**22 对 actual 不是同一套 env 的产物（异构）**"——不同场景的 nametable 内容天然不同，交集为 0 不能区分"不同 env"与"不同场景"；且 env 本就未被记录（F-0-C 自认）。A 级代码事实（TBLDRV 读 golden 表等）成立，但"异构"是**推断** | "异构"降为 B 并写明推断链；真正的判定依赖 P0-0 的逐对 env manifest（record argv/env/输入 sha/输出 sha） |
| **E4** | 中低 | plan:119（F-0-B 引 109b4d7/cf6847b 自述） | **cf6847b 自述与提交物矛盾**：其 message 称"l4_stg02-start 关名屏 **0 像素差**=内容帧首过"，但同 commit 提交的 `expected/l4_stg02-start.png`=**全黑 258B**、`actual/…`=246,083B/非黑 902px（git cat-file 复核）——该 commit 物下 diff 应为 902 而非 0。用该自述支撑 F-0-B 不成立（F-0-B 结论仍由 blob 全等独立成立） | 删除该自述引用，或改为"cf6847b 自述与提交物矛盾（0 vs 902）→ 反证签署态自述数字不可作证据；F-0-B 结论改由 blob/解码独立支撑" |
| **E5** | 中低 | plan:59（§1.2 "现行 A 级数据面"）、:184-204（§3.2）、:216-222（§3.3）；对照 [strip-tile-family.md:1](strip-tile-family.md)、[strip-family-direct.md:1](strip-family-direct.md)、[chr-org-timing.md:1](chr-org-timing.md)、[multicolor-palette-truth.md:1](multicolor-palette-truth.md) | 被引的 4 篇证据 doc 标题均自标 **"待审稿"**（strip-tile-family/strip-family-direct/chr-org-timing/multicolor-palette-truth），方案 §1.2 却将其列为"现行 A 级数据面"、§3.3 称"A 级三样本"、§3.2 把 multicolor 表列入"已证到哪一步（A 级）" | 将 4 篇的等级改为"B/待复核（doc 自标待审稿）"；A 级只保留可独立复导的字节/实测面（如 R-1/R-2/R-6 的 ROM/seed 字节）；或在引用前补对抗复核 |
| **E6** | 低 | plan:57（§1.2 l4-signing-criteria 行）、:276-277（§3.5 "§4.5 表述不成立"）、:415（§5.4 "同步 §4.5 dated 更正"） | **引错文档**："902=文本内容本身差"实际在 `l4-visual-plan.md:349-350@1be66e5`（2026-10-16 L4 签署终判段），不在 `l4-signing-criteria.md §4.5`（该 §4.5 是"stg02-start 902 回归：fallback 修不适用域…"，无该表述）。另外 "§4 五条缺口=本方案五族的来源"映射不成立：§4 五缺口=对位率/trace-v1v2/tile_page/3B/stg02，而方案族①（nametable）与②（W-C1 调色板）**不在 §4 五项内** | 更正为指向 `l4-visual-plan.md:349-350`；§1.2 改为"§4 五缺口对应族③④⑤+两项度量缺口，族①②来自 §3.5/§2 语义面" |
| **E7** | 低 | plan:120（F-0-C）、:474-477（§6.4） | F-0-C 称 `scenario_run.py` "走 xvfb-run（POSIX-only）"；实际 t24 已改为**自管 Xvfb**（直接 Popen `Xvfb :N`，非 xvfb-run 包装；scenario_run.py:467-475）。"POSIX-only"结论成立，但引用事实不精确 | 改为"POSIX-only（fcntl + 自管 Xvfb/DISPLAY，t24 已弃 xvfb-run 包装）" |
| **E8** | 低 | plan:59（strip docs 的自查命令 `grep -c "N-0188..."` 等） | 方案引用 strip 系列 doc 时未标注其自查命令依赖 `data/naming/ledger.jsonl` 条目；我只读复核未发现该 4 篇 doc 的待审状态被方案提及（见 E5） | 并入 E5 |

### 3.2 实现错误（Implementation errors）

**未发现代码级实现错误**（方案未做实现）。抽查项全部为真：tile_page 缺省/:773 更新/:851-853 跳码/:856 拟合、
FAMROW/FAMMAP/:894、GOLDPAL/:940-943、TBLDRV 读 golden 表/:127-149、单 pending_row/:694-715/:784-801、
`nmi_flush` 模型/terrain.rs:301-339、r4b 取错 bank/lib.rs:915-948、phase_batch 只回放文本写集——
无"文件/函数不存在"或"代码事实说反"的情况。

> 方案内部的**判据错误**归入 §3.3 S1/S2/S4（它们是方案设计缺陷，不是实现代码缺陷）。

### 3.3 规格缺口（Spec gaps）

| ID | 级 | 位置 | 问题 | 修正建议 |
|---|---|---|---|---|
| **S1** | **高** | plan:28-29（§0 结论3）、:318-324（§4.3） | **0 容差账目三套数互相矛盾**：①§0"17/22 对在数据面有闭环（仅 stg02+4 联冻结帧需重采）"；②§4.3"需先换资产才可判：stg02-start、stg06/stg08 四联、**ending 五帧**"（=10 对）；③§4.3"可达 G1 的 **11 帧**+G3 3 帧"——G1 只有 12 帧且**含 4 联冻结帧**，不可能有 11 帧可达；按方案自己的独立性判据严格计：**独立 expected+actual 对=12/22**（G1 非冻结 8+G3 3+title 1），**需重采/重渲=10 对**（stg02 1+四联 4+ending 5）。§0 的 17/22 把 ending 五帧同时算作"可闭环"与"需换资产" | 统一为"12/22 独立对 + 10/22 需重采/重渲（其中 ending 五帧 actual 同 blob 必须逐帧重渲，expected4/5 同文件需裁定真重复或重采）"；§4.3 重算可达数与不可达数（12+10=22 自洽） |
| **S2** | **高** | plan:308-316（§4.2 分步判据） | **分步 total-D 判据按方案自身步骤范围算术不可达**：<br>• P0-1 只治族①，残余 ②+③+⑤+余项=11,156+53,266+902+5,722=**71,046** > 其 `total D≤25,000`（即使族①全消，下界也是 71,046，因 P0-1 不修渲染/调色板）；<br>• P0-2 后（①③已修）下界=②+⑤+余项=11,156+902+5,722=**17,780** > 其 `total D≤15,000`；<br>• P0-3 只治②+余项，而 ⑤ 902 要到 P0-4 才治 ⇒ 下界 **902** > 其 `total D≤500` | 把每步 total-D 判据改为"**族内判据**（如 ①→≤10,000、③→≤5,000、②→0）+ 累计只作趋势观测"，或改成"该步结束时 **total D ≤ 上一步 total − 本步族上界 + 迁移项**"并给出迁移估计；否则判据不可自动验证 |
| **S4** | 中低 | plan:332-345（§5.0 P0-0）、:344（回退） | **P0-0 回退承诺不可执行**：`render-frame` 二进制把输出写死到 `oracle/screens/actual/{name}.png` + `.idx.json` + 追加 `.render.log`（[render_frame.rs:999-1011](../../crates/game-desktop/src/bin/render_frame.rs#L999-L1011)），无输出目录参数；"先输出到 `actual_canonical/` 或临时目录，经 Lead 同意后替换"需要**先给 render_frame 加输出目录/日志路径参数**（方案 §5.0 未列此代码改动），或复制整个工作区跑 | §5.0 增"render_frame 增加 `--out-dir`/`--log-path` 参数（最小改动，同 commit 记录）"或"在仓库副本中跑"；并说明 `.render.log` 追加语义与新 manifest 的关系 |
| **S5** | 低 | plan:203-204（§3.2 ink 表）、:402-403（§5.3） | 对现状描述不准：`render_rules.rs:58-73 ink_table` **已经是 family×index 二维表**，且已含 CJK 槽 1=暗青/3=白、Graphics（logo）槽 1=深蓝/2=浅蓝/3=白（与 multicolor §1 A 级表一致）；P0-3 真正要改的是 **BG 路径的 family 选择**（bank≥124→Cjk 否则 Game，render_frame.rs:945）与 **GOLDPAL env 覆盖**（:940-943），不是"单式→二维" | 改写 §3.2/§5.3：ink_table 已二维；P0-3 改动=①删 GOLDPAL 默认覆盖 ②BG family 选择改由 WindowContext/槽位决定 ③必要时把 Graphics 深蓝值对齐 multicolor 表（当前 DARK_BLUE=(18,44,110) 与 multicolor 表 (21,95,217) 不同，需先裁定） |
| **S6** | 低 | plan:414（§5.4 stg02 重采"帧对齐 f3450"）vs :641（§8 H4"正确帧未证"） | P0-4 把 f3450 当作重采对齐帧，但 H4 明确"正确帧未证（render 用 3404 vs 锚 3450）"——§5.4 应写"候选帧 f3450（H4 C 级），先定 render 用帧并与 expected 同帧" | §5.4 改为"候选 f3450（C，待 H4 定谳）；重采前先定 render 帧并记录 trigger=injected" |
| **S7** | 低 | plan:312（§4.2 P0-2 判据"含 ending skip 4,705 由 CJK 窗基承接"） | 与 F-0-A 冲突：ending 五帧 actual 是同一 blob，P0-2 的窗基修正对五帧产生**同一渲染结果**，而五帧 expected 不同（ending4/5 甚至 expected 同文件）——不可能靠一次渲染修复五帧 skip 4,705。ending 必须走 P0-4 的逐帧重渲 | P0-2 判据删"ending skip 4,705 由 CJK 窗基承接"，改注"ending 5 帧的 skip 需逐帧独立重渲（P0-4），P0-2 只承接其他场景" |
| **S8** | 低 | plan:233（§3.3"上下文键候选（B）"）、:380（§5.2"B 级候选"）vs :639（§8 H2"C"） | 同一"tile_page 上下文键=CHR R0 写流终值"在 §3.3/§5.2 标 B、在 §8 标 C，等级不一致 | 统一为 B/候选（或全 C）；§8 表与正文互相引用 |
| **S9** | 低 | plan:294-303（§4.1 顺序表） | P0-0 行写"除根数 1（F-0）"、像素上界 0——但 P0-0 的**输入恢复**本身依赖用户/环境裁定（Linux+Mesen2 或 fixtures 入库），属"不可由实施方单方完成"，方案已写"风险/回退 BLOCKED"但未在 §4.1 前置列显式标注 | §4.1 前置列 P0-0 加"需用户裁定环境/资产入库政策；未裁定则 P0-1..全部 BLOCKED" |
| **S10** | 低 | plan:485-630（§7 C1-C10） | 部分行号/命令口径不精确：①linebuf 测试引 `linebuf_timing.rs:9-53`，实际首测在 :6（范围内含 :27/:39/:53，可接受但边界漂）；②C5 用 POSIX `test -d`/`sed` 而 §7 标题称"全部只读；本机 python"，Windows 下需 Git Bash/WSL；③`diff_visual.py` 退出码引 L9/L191/L244-247，L191 实为报告模板行（退出逻辑在 :245-247） | 行号改"约 :6-53"；C5 明确"Git Bash/WSL"；C1-C10 全部给出可复制命令与"Windows 可跑/需 POSIX"标注 |

### 3.4 平台差异（Platform differences）

| ID | 级 | 位置 | 问题 | 修正建议 |
|---|---|---|---|---|
| **P1** | **高** | 全案（§5.1-§5.3 全部改 `game-desktop`）；对照 [game-shell/src/render.rs:1-9](../../crates/game-shell/src/render.rs#L1-L9)、[shell-win/src/lib.rs:557-630](../../crates/shell-win/src/lib.rs#L557-L630) | **L4 判定载体与运行时渲染器脱节且未声明**：P0-1..P0-5 的文件级改动全在 **`game-desktop/src/bin/render_frame.rs`（离线 L4 对拍器）** 与 `render_rules.rs`；而可玩 shell 的渲染是 **`game-shell::render`（占位画布）**+**`shell-win::FrameRenderer`（独立复制 GREY4/blit 的第四套渲染路径）**，未来 Android 又用另一套 `Renderer` 实现。⇒ 即便 P0-6 达成"22/22 diff=0"，收敛的是**离线对拍器**；游戏窗口/Android 显示是否一致无判据；且规则若要移植到运行时/Android 就是"换渲染器重做"（任务卡 check3 明列的裂缝）。方案全文 0 处提及 shell/game-shell/Android | 增加"L4 判定载体与运行时渲染器"一节：①明确 L4 判定对象=离线 render-frame（现状 gate 语义）；②声明运行时渲染器另立判据/或"L4 收敛后移植"路线；③把可复用渲染规则下沉到共享模块（如 game-core/render 或独立 crate），让 render_frame 与 game-shell/shell-win/Android 共用，避免二次实现；④Android 任务卡前置引用本决策 |
| **P2** | 低 | plan:565-571（C5）、:474-477（§6.4） | 采集链 POSIX-only（fcntl/自管 Xvfb/Mesen2）；方案正确声明"Windows 无法重放"，但 C5 混用 POSIX 命令与 Windows `cmd /c dir`，且"analysis 输入归档入库"是**政策/体积裁定**（analysis/ 被 gitignore） | 明确"C5 需 Git Bash/WSL"；§6.2 把"analysis fixtures 入库政策（体积上限/最小切片）"列为需用户裁定项（方案已列，但未给体积估算） |
| **P3** | 低 | plan:418-419（P0-4） | 逐关/逐结局帧重采依赖"八关自然链 bot 或 poke 选关（injected）"，自然链 bot 尚在规划（tas-bot-eight-stage-input-plan）；方案未给"bot 未就绪时的最小 injected 采集集"边界 | P0-4 增"natural vs injected 两档采集清单 + injected 必须显式登记（trigger 字段）"（方案已有 trigger 要求，补最小集与依赖顺序） |

---

## 4. 数字重算表（自述 vs 复核方独立复算）

| 指标 | 方案自述 | 我的独立复算 | 判定 |
|---|---|---|---|
| D-total 22 对 | 280,132 / 1,351,680 = 20.7247%；同点 79.2753%；1/22 零；<7680 12/22 | 逐项全同；22/22 值逐一相同 | ✅ |
| N-3way | 262,352 / 6,624 / 11,156 | 全同 | ✅ |
| 几何分桶（对照） | 条带内 13,246 / 条带外 266,886 | 全同 | ✅ |
| G1-G5 | 255,425 / 19,543 / 4,262 / 902 / 0（含三分量） | 全同 | ✅ |
| F-family 六桶 | 130,466 / 7,907 / 45,359 / 78,620 / 6,624 / 11,156 | 全同（且内部和=262,352/280,132 自洽） | ✅ |
| dedupe | 唯一 actual 15 / expected 17 / 比较 18；224,299/20.28% | 全同（20.2817%） | ✅ |
| F-0-A/B blob | ending 同 blob、4 联同 blob、stg02=title、ending4/5 同文件 | git blob + sha256 全同；stg02 expected 全黑 258B | ✅ |
| F-0-C env/输入 | analysis 缺、mesen2 缺、log 无 env、scenario_run POSIX | 全对（E7 仅 xvfb-run 表述不精确） | ✅（细节 E7） |
| C6 | 379/159 同位置非零交集 0 | 全同 | ✅（推断问题见 E3） |
| C7 | 11,156 全在 NES 域；Top 对 3,239/2,269/1,680/775/600/510/427 | 全同（且 11,156 全在 64 色 P 表内） | ✅ |
| C8/C9/C10 | ROM 字节、seed 行号、r4b 取址 | 逐字节/逐行号全同（含 +0x10 iNES 头换算正确） | ✅ |
| 旧账时效 | l4-final 失效、AR2-A 撤销、#103 pass | 全部核实 | ✅ |

---

## 5. 修正建议汇总（按优先级）

**阻塞 pass（必须修 + 复审；多为 docs/判据级，不改实现）**

1. **S1**（0 容差账目）：统一 17/22 → 12/22 独立对 + 10/22 需重采/重渲；重写 §4.3 可达/不可达清单。
2. **S2**（分步 total-D 判据）：改为族内判据/累计趋势；删除算术不可达的 `total D≤25,000/15,000/500`。
3. **P1**（L4 载体 vs 运行时渲染器）：新增声明节 + 共享渲染规则路线（render_frame / game-shell / shell-win / Android）。
4. **E1+E2**（证据等级与根因前提）：族①根因条件化（P0-0 env 冻结后才定）；empty-cell A→B；F-0-D 异构降 B；
   4 篇"待审稿"doc 降 B/待复核（E5）；删除/更名 cf6847b 自述引用（E4）。
5. **E6**（引错文档）："902=文本内容本身差"→l4-visual-plan:349-350；"§4 五缺口=五族来源"改写映射。
6. **S4**（P0-0 回退）：补 render_frame 输出目录/日志路径参数或"仓库副本"路线。

**建议修正（可与实施并行/下一卡）**

7. S5（ink_table 现状描述与 P0-3 改动点更正）、S6（f3450 候选帧）、S7（P0-2 删 ending skip 承接）、
   S8（上下文键等级统一）、S9（P0-0 环境裁定前置）、S10（行号/命令口径）、E7（xvfb 表述）、P2/P3（采集依赖与体积）。
8. 方案应补一张**逐对 provenance 表**（22 行：scene/frame/env/expected_sha/actual_sha/复用与否），
   作为 P0-0 的验收物，也是 S1/E1/E2 的消除手段。

**非本任务引入、顺带观察（不改变 verdict）**

- 方案引用的 3 篇 strip/chr-org 系列与 multicolor 文档自标"待审稿"——不是方案的问题，但方案引用前应先复核（E5）。
- 工作树中 `docs/spec/l4-signing-criteria.md`/`l4-visual-plan.md` 正被 task-13 修改；方案若后续仍按行号引用会漂移，
  建议改为"锚点+commit sha"。

---

## 6. 是否建议按方案顺序实施？先决条件

**顺序判断：原则同意，但不能按现判据直接开工。**

- **P0-0 先行正确**：F-0-A/B/C（actual 同 blob、stg02 expected=全黑复制品、输入/env 不可复现）
  全部经我独立复核成立——在不可复现基线上做逐族消解=无效度量。**P0-0 是唯一合理的第一步**。
- **P0-1 > P0-2 > P0-3 有量化依据**：族① 209,086 是族③ 53,266 的 3.9×、族② 11,156 的 18.7×；
  且 P0-1 会填 nametable 使族③ 规模显露（①→③ 迁移），先做族映射会在空 nametable 上无效——这条推理成立。
- **但顺序执行前必须先修**：S1/S2 判据（否则每步验收"不可达"无法判定完成）；P1（否则 22/22=0 只证明离线对拍器）；
  E1（否则 P0-1 可能改的不是生成 actual 的路径）；S4（否则 P0-0 会覆盖失效基线）——这四项是**开工前置**。

**先决条件（需 Lead/用户裁定）**：
1. **环境/资产**：Linux+Mesen2 采集环境 或 `analysis/` 最小 fixtures 入库政策（体积/脱敏）；P0-4 的
   stg02/4 联/ending 逐帧重采依赖此项——未裁定则 P0-1..P0-3 只能做"代码面预研"、无法闭环验收。
2. **P0-0 canonical render 清单**：逐对 env/args/输入 sha/输出 sha（方案已有设计，需明确为硬验收）。
3. **门禁硬化**（P0-6 的 diff_visual 退出码 0 容差化）：独立 commit + Lead 批准（AGENTS §5），
   且必须先有 canonical 基线，否则门禁转红后无法区分"真回归"与"基线噪声"。
4. **L4 载体声明**：确认 L4 判定对象=离线 render-frame；运行时 shell/Android 的像素判据是否另立
   （本报告 P1 的决策点）。

**0 容差是否真能收敛到 0？**
- **诚实性判断：方案顶部"当前 0 容差不可达、P0-0/P0-4 需裁定"是诚实的**（未把分类账/进程签署当达成；
  §4.3/§6.2/§6.3 均显式要求用户/环境）。这一点**通过**。
- **可达性证据不足**：在其"12/22 独立对"范围内，族①/②/③ 均有 A/B 根因与可复现数据面，**有收敛路径**；
  但 10/22（stg02、四联、ending 五帧）必须重采/重渲，其中 ending 五帧的 actual 同 blob 是**渲染命令/
  诊断态产物**而非游戏内容，重渲是否可达 0 取决于输入恢复与 expected 逐帧有效性——方案尚未证明。
  因此结论应为"**0 容差在 12/22 上有可执行路径；剩余 10/22 未证明，需 P0-0/P0-4 落地后用新基线重判**"，
  而非"17/22 可闭环"。

---

## 7. 复核声明

- 本复核对方案 C1-C10 的**全部数字/字节/行号**做了独立重算（自写脚本，未执行被审方脚本）；
  五族归因逐族核对，另抽查代码事实 10+ 处、旧账 6 项、git blob 6 组。
- 未修改被审方案/任何实现/文档；未 `git add`/`git commit`；仅新增本复核文件。
- 方案文件在复核期稳定（sha256 `23a3a978c8564ace…`，665 行）；其引用的 spec 文档行号按
  `1be66e5` 核验（工作树中 task-13 正在改 spec，故本报告不引用其未提交行）。
- 结论不代表 L4 现状判定（沿用用户裁定：L4=未达成）；本报告只判定"攻坚方案"是否可作实施依据。

---

## 复审（2026-10-18，第二轮修订）

> 复审对象：task-19 修订后的 [l4-zero-tolerance-attack-plan.md](l4-zero-tolerance-attack-plan.md)
> （复核时实际 **911 行**、sha256 `1be086c05cfd4713…`、mtime 13:16:20；Lead 转述 909 行为前一次保存，
> 以 911 行定稿核）；l4-analyst 已 inactive。只核 task-8 findings 的修正项（S1/S2/P1 + E1-E7/S4-S10/P2/P3）
> 与用户三项裁定；数据面按"未变"假定做**独立重算抽样验证**（重跑族总量/逐对 provenance/C11 脚本逐字运行）。

### 复审 VERDICT：**pass**（附非阻塞残留 R1-R9；其中 R1/R2 建议在提交方案前顺手改）

S1/S2/P1 三条高项**已实质闭环**；E1-E7/S4-S10/P2/P3 逐条落点均已核销；用户三项裁定中
环境（§5.0）与 W-C1/C4 撤销（plan + l4-visual-plan §2）已到可执行位置，门禁硬化的"两步走"
在仓库工具侧已落地（`64b3a44`）、但**方案 §5.6 未对齐**（R1，见下）。P0-0 可在用户环境就绪后启动。

### 1. S1 账目（原高）——已闭环

- **唯一口径已统一**：§0:37-40、§2.6:150-183、§4.3/§4.2 统一为 **12/22 独立 expected+actual 对 +
  10/22 需重采/重渲**（stg02 1 + 四联冻结 4 + ending 5）；`12+10=22` 在 §0、§2.6、§4.2、§4.3 四处一致。
- **"17/22 可闭环"/"G1 11 帧"已删**：全文件仅剩两处**否定/勘误语境**（§0:40 "不再使用旧口径"、
  §9.1:879 "删…"），无任何活性主张。
- **§2.6 逐对 provenance 表自洽（22/22）**：我从前文正则解析 22 行，逐行比对
  `.render.log` 末条（render 帧/scroll/probe）与 expected/actual PNG sha256(12)——**0 处不符**；
  复用关系（ending-1..5 同 actual 65af76c9、四联同 actual db7262bc、stg02/title 同 expected 788d6984、
  ending4/5 同 expected 6ff345dc）与 §2.5 dedupe 完全一致。独立对清单 = #1/#2/#3/#4/#5/#11/#12/#13/
  #15/#17/#18/#22（G1 8 + G3 3 + title 1）✓；需重采清单 = #6-#10+#14+#16/#19/#20/#21 ✓（12+10=22）。
- 表格明确 scene/env 两列为 F-0-C 缺口、P0-0 补全后才作最终裁定（§2.6:152-154）——口径诚实。

### 2. S2 分步判据（原高）——已闭环（算术自洽）

- **total-D 阈值已删**：§4.2:376-381 把 `total D≤25,000/15,000/500` 仅作为"为何不用 total-D"的
  勘误引用保留；分步表 §4.2:398-405 全部改为**本步族内 + 独立 12 对**计数，无 total-D PASS 条件。
- **C11 逐字实跑（未改写其脚本）**：从方案 §7-C11 代码块原样抽出运行，exit 0，输出：

```text
independent 12: {'f1': 137263, 'f3': 45514, 'f2': 6814, 'spur': 960} sum 190551
non-independent 10: {'f1': 71823, 'f3': 7752, 'f2': 4342, 'spur': 5664} sum 89581
```

- **非独立拆分与 C3 相容（我独立精算）**：
  `①71,823 = 四联 4×16,205(64,820) + ending 7,003`；
  `③7,752 = 四联 4×188(752) + ending(941 skip+459 presblank=1,400)×5(7,000)`；
  `②4,342 = 四联 4×348(1,392) + ending 590×5(2,950)`；
  `spur 5,664 − ⑤902 = 余项 4,762 = 四联 4×543(2,172) + ending 518×5(2,590)`；
  独立侧：`empty-cell 58,643 = 130,466−71,823`、`nt2 78,620`（全在独立帧）、
  `fade-clear ①=50,071`、`clear-inject ①=38,400`——与 §4.2:389-396/401 的每个数一致；
  `190,551+89,581=280,132` ✓（与 C1 总差一致）。
- **族总量复核（本轮逐像素重算）**：f1=209,086、f3=53,266、f2=11,156、spur=6,624，
  合计 280,132 ✓（C3 六桶：empty 130,466/skip 7,907/presblank 45,359/nt2 78,620/spur 6,624/color 11,156
  与上轮独立复算同值；PNG/idx 未变）。
- **判据可达性判断**：族内+独立 12 对判据（①137,263→≤10,000；③45,514→≤5,000；②6,814→0；
  余项 960→≤200）**算术自洽、无互相矛盾**；具体数值能否工程达成需 P0-0 重采后实测，不属本轮可判。
  但 **P0-1 的子判据只覆盖 nt0 empty 58,643，未覆盖 nt2 78,620**（占独立①57%）——见残留 R2，
  不构成算术矛盾，但建议补子目标以免验收口径不完整。

### 3. P1 载体（原高）——已闭环（事实成立、路线可执行、需前置）

- **§4.4:419-440 已声明**：L4 判定对象=**离线 `render-frame`**；全部像素数字/判据均指此载体、
  不得表述为运行时显示一致；运行时渲染器另立判据（另立任务卡）；共享渲染规则模块路线
  （候选 `game-core/src/rendering_rules.rs` 或新 crate `game-render`，消费者=render_frame/game-shell/
  shell-win/Android）；重构判据=render_frame 改调后 22 对 diff 不变；Android 任务卡前置声明。
- **与现状一致（我核 HEAD）**：`game-shell/src/render.rs` 仍是 v1 占位（只画玩家块，内容锚"渲染合成…v1 显式占位"）；
  `shell-win` 存在独立 `pub struct FrameRenderer`（HEAD `lib.rs:594`，含复制 GREY4/blit——"另一套"事实成立）；
  `game-platform::Renderer` trait 存在；平台壳/Android 未来实现同 trait。方案 §4.4 对"三套现状+未来 Android"
  的描述准确，且已注明 game-shell 正被并发任务修改、以内容锚为准。
- **可执行性**：共享模块放 game-core（无 I/O、确定性）在架构上成立（纯函数即可）；但两点前置——
  ①"22 对 diff 不变"的重构验收依赖 **P0-0 canonical 基线**（否则以不可复现 actual 验收，见 R6）；
  ②结构变更需 Lead/用户批准（§6.2-4 已列）。故 P1 是"可执行、待批、须在 P0-0 后验"的状态，诚实。

### 4. 应核项逐条核销（E1-E7 / S4-S10 / P2/P3）

| finding | 方案落点（核销） | 复审结论 |
|---|---|---|
| E1 | §3.1 前提（族①根因限默认路、绑定 FLAGUPLOAD env）、§5.1:482 前置（P0-0 逐对 env manifest 先确认路径） | ✅ 已按"先证 env 再定根因"处理 |
| E2 | §3.0:197（empty-cell 130,466 A→**B+**，RENDER_NT/ntr 未记录）、§7-C3 补注"先断言 ntr=0" | ✅ 等级降级到位 |
| E3 | §2.4:133（"22 对异构"降 **B**，C6 只否定同 TBLDRV 态，判定依赖 P0-0 manifest） | ✅ |
| E4 | §2.4:135-137（删 cf6847b 自述证据，标注其与提交物矛盾 0 vs 902；F-0-B 由 blob/解码独立支撑） | ✅ |
| E5 | §1.2、§3.0:197-199、§3.2:240、§3.3:271（4 篇自标"待审稿"doc 降 **B/待复核**；A 级只留字节面） | ✅ |
| E6 | §1.2:68（§4 五缺口映射改写）、§3.5:341-344、§5.4:559-560（"902=文本内容本身差"改指 l4-visual-plan:349-350@1be66e5，非 §4.5） | ✅ |
| E7 | §2.4:132、§6.4:638/744（POSIX-only=fcntl+自管 Xvfb/DISPLAY，t24 已弃 xvfb-run 包装；行号 :467-477） | ✅ |
| S4 | §5.0-2:459-462（render_frame 加 `--out-dir`/`--log-path`，默认保持、先写 actual_canonical、不覆盖现行 actual；回退/整仓副本） | ✅ |
| S5 | §3.2 差什么②、§5.3:543-546（ink_table 已是 family×index 二维；真改动=删 GOLDPAL 覆盖/BG family 选择/Graphics DARK_BLUE 裁量） | ✅ |
| S6 | §5.4:557-560（f3450 标 **C 级假设 H4**，先定 render 帧并与 expected 同帧） | ✅ |
| S7 | §4.2:402、§5.2:529（删"ending skip 4,705 由 CJK 窗基承接"；ending 交 P0-4 逐帧重渲） | ✅ |
| S8 | §3.3:280/§5.2:520/§8:848（上下文键统一 **C 级假设 H2**；数据面 B/待复核） | ✅ |
| S9 | §4.1:362、§5.0:477（P0-0 前置；未完成则 P0-1..全部 BLOCKED；环境已裁定提供） | ✅ |
| S10 | §5.6:583（diff_visual 退出逻辑改参考 L245-247、L191=报告模板）、§7 平台注（linebuf :6-53、C5 跨平台 python） | ✅（细节行号残留见 R5） |
| P2 | §6.2-6 fixtures 入库政策列为裁定项（体积估算 C 级待实测）、§6.4、C5 跨平台化 | ✅ |
| P3 | §5.4:562-566 自然/注入两档；注入最小集=10 帧（stg02/四联/ending 各 1）且 trigger=injected 登记 | ✅ |

### 5. 用户三项裁定落地核验

| 裁定 | 落点 | 复审结论 |
|---|---|---|
| ① 提供 Linux+Mesen2 环境 | §5.0:448-478 可执行清单：环境接收（python3/Rust/Mesen2 2.1.1/Xvfb）、`analysis/.scenario.lock` 串行、逐场景 `scenario_run.py run` + `phase-batch`、render_frame `--out-dir/--log-path`（待 P0-0-2 最小改动）、canonical 命令+逐对 env、manifest 字段、两跑 sha 全同、ending4/5 真重复裁定、新基线/回退/BLOCKED | ✅ 可执行；我核对 `analysis/.scenario.lock` 路径、`render-frame`/`phase-batch` bin 名与现有工具一致 |
| ② 撤销 W-C1/W-C4 白名单候选 | plan 头部:19、§3.2:268-269、§5.3:551-552、§6.2:607-610；`l4-visual-plan.md §2` 工作树已完成撤销注（W-C1/W-C4 划线/撤销、W-C2/C3/C5 状态不变、whitelist 保持不存在） | ✅ 已落（spec 改动 staged 待提交，由 spec-writer/Lead 收口） |
| ③ 门禁硬化（两步走） | 仓库工具侧：`tools/diff_visual.py --strict` 已实现（`64b3a44`，默认行为不变、TODO 默认翻转待 P0-0 后——即标准两步走）；**但 plan §5.6 未引用 `--strict`/两步走**，仍写"改为存在差异⇒exit1（默认），前置=P0-0" | ⚠️ 裁定本身已可执行；**方案 §5.6 需对齐**（R1） |

### 6. 我的实跑/复算命令与输出摘要

```text
# 1) C11 逐字运行（从方案 §7 代码块原样抽出，未改写）
python $TEMP/l4_c11_verbatim.py   -> exit 0
  independent 12: {'f1': 137263, 'f3': 45514, 'f2': 6814, 'spur': 960} sum 190551
  non-independent 10: {'f1': 71823, 'f3': 7752, 'f2': 4342, 'spur': 5664} sum 89581

# 2) §2.6 provenance 表逐对复核（自写解析 + 独立计算）
逐对 = .render.log 末条(frame/scroll/probe) + expected/actual sha256(12)
结果：22/22 行全部一致；0 mismatch；复用关系与 §2.5 dedupe 一致

# 3) 族总量/拆分独立重算（逐像素 22 对）
f1 137,263+71,823=209,086；f3 45,514+7,752=53,266；f2 6,814+4,342=11,156；
spur 960+5,664=6,624（含⑤902 → 余项非独立 4,762）；总 280,132
非独立拆分：①=4×16,205+7,003；③=4×188+7,000；②=4×348+2,950；余项=4×543+2,590（+902 单列）✓
独立侧：empty 58,643、nt2 78,620、fade-clear ①50,071、clear-inject ①38,400 ✓

# 4) P1 事实（HEAD）
game-shell/src/render.rs:1-10 = "v1 显式占位"（只画玩家块）✓
shell-win/lib.rs HEAD：pub struct FrameRenderer = :594 ✓（方案分析时点 :557，见 R5）
game-platform::Renderer trait 存在 ✓；platform-shell §4/§4.1 与之一致

# 5) 用户裁定 ③ 的仓库现状
tools/diff_visual.py --strict 已实现（64b3a44 ancestor of HEAD）：默认=分类账口径、strict=0 容差
（:156-159/:196-199）；TODO: 默认翻转待 P0-0（:20）。plan §5.6 未引用（R1）。
```

### 7. 剩余问题（非阻塞；建议收口）

| # | 级 | 位置 | 问题 | 建议 |
|---|---|---|---|---|
| **R1** | **中** | [plan §4.2:406](l4-zero-tolerance-attack-plan.md)（P0-6 行）、[plan §5.6:581-592](l4-zero-tolerance-attack-plan.md)（判据行 :589）；对照 `tools/diff_visual.py:15-20,156-159` | **门禁硬化两步走未对齐**：用户裁定落地形态=先加 `--strict`（默认不变，`64b3a44`），默认翻转留至 P0-0 后（task-21 TODO）；plan 仍写"改为差异⇒exit1（默认）+ 前置 P0-0"。若照字面①P0-6/§5.6 判据 `python tools/diff_visual.py → 22/22 diff=0` 用默认口径会**假绿**（默认"全部有分类即 0"）；②P0-0 后即翻默认会让 pre-commit 在 22 对未清零时恒红、阻断 P0-1.. 提交 | §5.6 改为：L4 0 容差验收判据=`python tools/diff_visual.py --strict` exit 0；默认/pre-commit 翻转留至 P0-6 全绿后（引 task-21 / `64b3a44`）；§4.2:406 与 §5.6:589 判据同步加 `--strict`。**建议提交方案前改（3-5 行）** |
| **R2** | 中低 | [plan §4.2:401](l4-zero-tolerance-attack-plan.md) | P0-1 总①137,263→≤10,000 隐含 **nt2 78,620 同步下降**（占独立①57%），但子判据只列 nt0 empty 58,643→≤2,000；nt2 域当前是 B 级推定（§3.1） | 补 nt2 子目标或注明"由 `$2006/$2007` 默认投影+4-NT 持久化共同承接、P0-0 后按实测重算"；避免 P0-1 验收口径不完整 |
| **R3** | 低 | plan §0:34-36、§4.1:360-374 | 顺序/收益仍用 22 对全量（①200,879、③53,266、②+余16,878），与 §4.2 独立 12 对判据（137,263/45,514/6,814/960）并存；§4.1 有"像素上界=22 对全量，分步判据=独立 12 对"注，但 §0 未交叉引用 | §0 结论3 加一句"上界口径，分步验收见 §4.2" |
| **R4** | 低 | [plan §9.1:881](l4-zero-tolerance-attack-plan.md) | P1 行写"§5.0/§5.3 交叉引用"，实际 P1/载体在 §4.4、§5.6、§6.2-4，§5.0/§5.3 无 P1 引用 | 更正映射落点 |
| **R5** | 低 | plan §5.1:500（`:1006`）、§9:903（`shell-win/lib.rs:557`） | 行号漂移：idx 写入实际 :1003-1008、ntr env :815；shell-win FrameRenderer 在 HEAD 为 :594（task-18 后） | 改内容锚或标注"分析时点 HEAD 1be66e5"并更新当前行 |
| **R6** | 低 | plan §4.4:438 | 共享渲染规则重构判据"22 对 diff 不变"依赖 P0-0 canonical 基线；方案未写重构时序 | §4.4 加"重构须在 P0-0 后，以 manifest/canonical 判据验证" |
| **R7** | 低 | 数据面声明 vs §7 | "C1-C10 一字未动"不严格：C3 有 E2/S10 增注、C5 改为跨平台 python（命令/期望值不变，数值独立复算同上轮） | 记录口径即可（"数值/期望未变；C3/C5 有 S10/P2 要求的命令注记"） |
| **R8** | 低 | `tools/diff_visual.py:213/220`（生成报告模板） | W-C1 撤销未同步到生成物文案：报告仍输出"W-C1 候选未启用/平台差异候选 W-C1"；task-20 已声明生成物改动属 tools、需先报 Lead | 由 task-20/21 收口时同步报告模板（去掉/划线 W-C1 候选表述）；不影响当前 gate |

**复审结论**：S1（账目/§2.6 provenance/12+10）、S2（total-D 删除/族内+独立 12 对/C11 实跑）、
P1（载体声明/运行时另立/共享模块路线）三条高项**已闭环**；E1-E7/S4-S10/P2/P3 全部核销；用户裁定 ①② 已落、
③ 已可执行（工具侧 `--strict`）但 plan §5.6 需对齐（R1）。**建议：pass；R1/R2 在提交方案前顺手改
（doc-only，不影响 P0-0 启动），其余残留随 P0-0/P0-6 收口。**
