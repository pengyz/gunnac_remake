# 对抗复核：L5 八关证据缺口盘点（task-39）—— 证伪立场独立复核

> 复核方：`shell-verifier`（task-51，verifier 角色）；被审：task-39 / `l4-analyst`
> 被审文件：`docs/findings/l5-gap-inventory.md`（316 行，**未提交**；快照 HEAD `b92139a`）。
> 复核时工作树另有并发改动（task-50 的 `docs/findings/l4-baseline-freeze-plan.md` 等），已按任务归属隔离。
> 纪律：[AGENTS.md](../../AGENTS.md) §5 独立对抗复核（不采信自述、自己重读 jsonl/源码/文档重算）。
> **verdict：needs_revision**（核心数字与绝大多数结论**独立复算通过**；但两项 Lead 指定的必查点有**事实性错误/时点过期**：
> ①`probe5` 生产面误判（漏报仓库内已存在的 ignored 发射器，导致 B5/GL5-6 环境分档错误）；②§5 跨任务状态已过期
> （task-50 已派且工作树 L4 手册已落地更正，但仍写“尚未写入/请 Lead 决定”。另 5 条 Low 精度项。全部为 docs 内可修，预计 1 次小改即可复审。）

## 0. 被审对象与复核边界

- 文件 316 行，内容=数字盘点 + 逐关已证程度 + A/B/C 分档 + 与 L4 P0-0 共用环境清单 + 跨任务新证 + 假设表。
- 复核未改该文件；只写本复核文件；日志在 `target/review-l5/`（可再生、不入库）。
- **未覆盖**：`analysis/traces/*`、`shots/`、Mesen2 真机——环境不存在（已独立核：`analysis/` 仅 `visual-scout/workset`，
  `analysis/traces`、`shots/`、`tools/mesen2/` 均不存在），故所有“需重跑/真机”结论只核**证据链与算术**，不核运行结果本身。

## 1. 数字复算（Lead 指定必查，全部自读 jsonl 重算）

| 口径 | 自述 | 我实测（`oracle/traces/*`） | 结论 |
|---|---|---|---|
| s5-all-clear 行/真锚/占位 | 44=39+5 | 44 行；`f>0`=**39**；`f==0`=**5** | ✅ |
| stg01-chain | 17=16+1 | 17 行；真 **16** / 占位 **1** | ✅ |
| tas-clear-full | 10=10+0 | 10 行；真 **10** / 占位 **0**（v5 9 + relay 1） | ✅ |
| 锚台账 | 50=44+6 | **50** 条；`golden=true` **44**；`golden=false` **6** | ✅ |
| trigger（50 锚基） | 38 injected / 7 natural / 5 placeholder | **`Counter({'injected':38,'natural':7,'placeholder':5})`** | ✅（任务卡口径） |
| trigger（44 golden 基） | 文中“7 natural + 32 injected + 5 placeholder” | 44 golden 基确为 **7/32/5**（7+32+5=44）；6 条附加锚全 injected | ✅（**但 base 未标注，见 E1**） |
| 真锚 natural 名单 | 7 条标题+STG-01 前缀 | `boot-black/title-in/menu-in/stg01-shop/stg01-exit/stg01-start/stg01-combat` | ✅ |
| 真锚 injected 范围 | stg01-clear..stg08-shop | 首 `stg01-clear@2401` / 尾 `stg08-shop@16221`，共 32 | ✅ |
| 证据指针分类 | 33 traces + 12 shots + 5 none | **Counter({'traces':33,'shots':12,'none':5})**（和=50） | ✅ |
| 5 占位内容 | stg08-exit/start/combat/clear/all-clear-95f9 | golden/anchors：**第 1 行 `checkpoint=None`（mode=Transition）** + `stg08-start/combat/clear/all-clear-95f9`（4 命名）；锚台账同 | ⚠️ 命名不精确（E4） |
| 44 行/3 场景/36 命名检查点 | 36 | 场景 `s5-chain`32/`s5-stg01`9/`s5-title`3=44；**唯一命名 checkpoint=36**；null checkpoint=8（含 7 真锚过渡 + 1 占位） | ✅ |
| `l5_s5.rs:68` 锁 | (39,5) | `crates/game-oracle/tests/l5_s5.rs:68`：`assert_eq!((real, ph), (39, 5), ...)` | ✅（行号精确） |
| `l5.rs` 529 行 / API 行号 | 529；56/111/131/161/307 | Python 行数 **529**；`l5_legality`:56、`edge_proven`:111、`diff_l5`:131、`diff_run_face`:161、`s5_chain_run`:307 **全部命中** | ✅ |
| `s5_chain_run_for(8)` 断言 | 8 clear/8 shop/8 exit/ending | `l5_s5.rs:77-80` 逐条命中（`:73-92` 范围成立） | ✅ |
| diff_l5 自检 | `--selftest` exit 0 | 实跑 **exit 0**（`selftest: ok`） | ✅ |

**算术核**（跨任务新证，§5）：
- `tas-clear-full.l5.jsonl` 实测：stg01-clear **f28364** → stg02-shop 29078 → stg02-combat 29478 → **stg02-clear f51332**
  → stg03-shop 52047 → stg03-combat 52447 → **stg03-clear f72823** → **stg04-shop f73539 / stg04-combat f73939**
  （+ relay stg07-clear f28683）——与自述逐帧一致。
- 36500f 截断：`29478 < 36500 < 51332` ⇒ **在关 2 战斗段** ✓（“按 stage_no==6/8 选帧必然失败”成立）。
- “stage6/8 内容帧 >100k”：已测关时长 stg2≈22968f、stg3≈21491f；stg4 起于 ~73.5k ⇒ stg6 内容帧约 **>110k**、stg8 更高
  ⇒ “>100k”作为**估算**成立（建议标“按已测关时长外推，B 级估算”）。
- `tas-clear-full-v5.txt`：`frames 300000` ✓；`tas-stg08b2-direct.txt`：`frames 40000` + **10 条 probe**（8 标准+048c+4000-4017）✓；
  `tas-stg07-relay`：`frames 120000` ✓。

## 2. 必查项逐条结论

| # | 必查点 | 独立证据 | 结论 |
|---|---|---|---|
| 1 | 39 真锚/5 占位、44/17/10、50=44+6、trigger、l5_s5.rs:68 | 见 §1 表（全部 jsonl 重算 + 源码行验证） | ✅ 数字真实 |
| 2 | 八关冻结点与 trace 一致；boss 击杀/自然关末/流末边界全缺如实；stg08+ending 5 占位（OBS-1） | `f5033`(stg02-timeline:310)/`f6893`(stg03:471,stg04:288)/`f9164`(stg04:488,stg05:262)/`f11093`(stg05:482,stg06:352)/`f13977`(stg06:680,stg07:50)/`f15959`(stg07:53,326;stg08:41) **均在库内 time-line findings 命中**；stg07 观测窗 f14504-15500=finding 原文；stg08 注入 #7 f15500→f15501 `$27:=$C0`→f16221 商店→出店受阻= finding 原文；OBS-1 在 `docs/bug-ledger.md:27` 为**候选不立号**；gs5-6 Run B 负结果（`$CA` 注入不消费）在库 | ✅（负结果表述如实） |
| 3 | A/B/C 三档可执行性 + 环境依赖标注 + 17 行清单与 L4 P0-0 复用/去重 | 三档均有角色/范围/判据（A1-A8/B1-B7/C1-C6）✓；`analysis/traces`/`shots`/`tools/mesen2` 缺失✓；L4 P0-0 I1-I4=title-menu/gameplay-stage1/stg03-timeline/stg08-timeline 与 §4.1 “完全重合”✓（L4 手册对应行已核）；17 行表格数=17✓。**但**：`probe5` 发射器漏报 ⇒ B5/GL5-6 分档错误（E2）；17 行含 1 取消/2 降级，§0.6 仍说“跑 17 个”（E5） | ⚠️ 需修（E2/E5/E6/E7） |
| 4 | 跨任务新证（36500 失效、>100k、转 task-50） | 算术 ✓（§1）；task-50 **存在且 in_progress**（owner=l4-analyst，写入范围=docs/findings/l4-baseline-freeze-plan.md）；**工作树 L4 手册已含该更正**（`l4-baseline-freeze-plan.md` 现文 §2.1 I5=tas-stg08b2-direct 40000f/10 probe、I5-old 36500 降级对照、§3.6/§3.7 dated 更正、§8 不再跑 v5 36500f）——与 §5 建议一致 | ✅ 已转卡且已落地；**但被审文件 §5 状态写“尚未写入/请 Lead 决定”，已过期（E3）** |
| 5 | 未证不发明/引用可核 ≥5 指针；docs-links；只 1 文件 | 指针抽验 12+ 条全命中（见 §3）；`gates status` docs-links **PASS**（332 md/417 links）；新增文件=本盘点稿（其它工作树改动可归因并发 task-50/其它卡，mtime 14:29>本文件 14:27） | ✅（除 E3 时点） |

## 3. 指针抽验（≥5，全部命中）

1. `crates/game-oracle/tests/l5_s5.rs:68`（39/5 锁）✅；`:77-80` 8 clear/8 shop/8 exit/ending ✅。
2. `crates/game-oracle/src/l5.rs`：529 行 + `l5_legality:56`/`edge_proven:111`/`diff_l5:131`/`diff_run_face:161`/`s5_chain_run:307` ✅。
3. `tools/diff_l5.py --selftest` → exit 0 ✅（并核 --probe 消费口径：StageProbe JSONL ↔ golden，`unknown` fail-closed）。
4. `docs/spec/l5-all-clear.md` R-l5-8：确为 2026-10-15 旧口径（7 natural + 5 injected + 32 placeholder=12 真锚/32 占位）✅（doc 的“仍写 12/32”为忠实转述）。
5. `docs/bug-ledger.md:27` OBS-1 候选观察区、不立号 ✅；`docs/findings/stg08-timeline.md` §7.1 四位菜单矩阵五 run A 级 + 确定性截断 ✅。
6. 冻结帧 6 处在 `docs/findings/stg02..stg08-timeline.md` 命中（行号见 §2）✅。
7. `docs/spec/l5-plan.md:109` “未启用前 L5 显式 SKIP（不静默）” ✅；`gates/cli.py`/gates 目录 **grep 无 L5** ⇒ “无 L5 门禁条目”✅（status 12 项亦无）。
8. 环境缺失三项实测：`analysis/traces` 不存在 / `shots/` 不存在 / `tools/mesen2/` 不存在；`analysis/` 仅 `visual-scout`+`workset` ✅。
9. L4 P0-0 手册：I1-I4 场景与 §4.1 重合声明 ✅；I5/I5b/I6b/I5-old 已按 L5 交叉核对改正 ✅。
10. 场景文件：`tas-clear-full-v5.txt`(300000f)/`tas-stg08b2-direct.txt`(40000f,10 probe)/`tas-stg07-relay.txt`(120000f) ✅。
11. bot 负结果指针：`bot-combat-v2-diagnosis.md`（f9045 GAME OVER、避让/簇聚合等五缺陷）✅；`gs5-6-dynamic-replay.md` §2.2 Run B “注入点不消费”✅。
12. `docs/findings/l5-instrumentation-survey.md` §1 `StageProbe`/`stage_probe`（engine.rs）✅。

## 4. findings（四分类；**E2/E3 为需修主项**）

### 4.1 实现错误（B）——无（本卡为文档盘点，无代码改动）

### 4.2 证据不足 / 事实错误（E）

- **E2（中，必查点 3 失败）`probe5` 生产面误判 → B5/G-L5-6 环境分档错误**：
  §1.1 称 run 面产物不存在（✓ 文件确实不存在），但写“**全仓 grep `probe5` 仅 2 处文档引用**”**不实**：
  实仓 grep 命中 **`crates/game-core/tests/engine_tick.rs` 10 处**（`#[ignore] fn emit_probe5()` 发射器，写
  `analysis/traces/stg01..08.probe5.jsonl` + `gameplay-stage1.probe5.jsonl` + `respawn-window.probe5.jsonl`）
  + `docs/findings/l5-instrumentation-survey.md` 1 + `tas-bot-eight-stage-input-plan.md` 2 + `tools/diff_l5.py` 1。
  影响：engine 侧 probe5 **不需要 Mesen2**（仅需 `mkdir analysis/traces` + `cargo test -p game-core --test engine_tick -- --ignored emit_probe5`；
  发射器不建目录，当前目录缺失会 panic）；`diff_l5.py --probe` 可对本机发射的 StageProbe 与库内 golden 做 **7 natural 锚** 的零依赖对拍。
  ⇒ B 档 B5 应拆为：**A 档（本机可做）** `emit_probe5`+`diff_l5 --probe`（natural 面/字段面）；**B 档（需 Mesen2）** 真机 run 面重采。
  建议：§1.1 行改“文件不存在；**引擎侧发射器已落（engine_tick.rs `emit_probe5`，ignored；需先建 analysis/traces）**”；
  G-L5-6 与 §3.2 B5 同步改分档；A7 契约卡可加“跑通 emit_probe5 + diff_l5 --probe 对 natural 锚”验收。
- **E3（中，时点过期）§5 状态与事实不符**：§5 写“该条**尚未写入 L4 手册**（本轮 task-48 范围仅组 D）；请 Lead 决定是否再派一张 L4 点改卡”。
  复核时：**task-50 已存在**（in_progress，写范围=l4-baseline-freeze-plan.md，描述与 §5 建议逐项一致）；且**工作树 L4 手册已落地更正**
  （I5=`tas-stg08b2-direct` 40000f/10 probe、I5-old 36500f 降级对照、§3.6/§3.7 dated 更正注、§8 不再跑 v5 36500f）。
  建议：§5 状态改为“已转 task-50（工作树已改，待提交/复核）；原卡面点改清单=…”，保留历史陈述。
- **E4（Low）占位命名**：golden/anchors 中 5 占位的第 1 行 `checkpoint=None`（mode=Transition，scene=s5-chain），
  无 `stg08-exit` 该命名字段；§0.4/§1.2 写“stg08-exit/start/combat/clear/all-clear-95f9”应注“(未命名 Transition 行，语义=出店段)”。
- **E5（Low）§1.2 附加锚清单不准**：6 条附加锚实测=`stg02-exit-thanks@3380 / gate-26-bit4@2400 / flyout-vector@2794 /
  gate300-open@3066 / inc-180@3096 / stg02-shop-menu@3300`；文中“（$26 bit4 置位/$0300 开/INC $180/**$9BC3 init**/主菜单/出店帧）”
  与实际不符（无 `$9BC3 init`、漏 `flyout-vector`）。建议直接列 6 个 id 或指向 anchors 文件。
- **E6（Low）17 行清单口径**：§4.2 表确 17 行，但 #17 已取消（M2）、#15/#16 按 §5 可能降级/废弃（L4 二次更正后），
  §0.6“一次性跑 **17 个**场景/overlay”应改为“17 行清单（1 行已取消、2 行降级对照 ⇒ 有效 14-16 行，按 §4.3 优先级执行）”。
- **E7（Low）§4.2 内部矛盾**：#10 v5 overlay 预算 80000“覆盖 f73939”✓，但同格写“四联 stage6/8 候选（若跑到）”——
  80000 < 自述“>100k 才到 stage6/8” ⇒ 不可能跑到；建议删“stage6/8 候选”或把预算改 >100k。
- **E8（Low）工时估算偏乐观**：逐行区间取**下限**求和 ≈3.5h（#1 1 + #2 2 + #3 2 + 8×10 + #10 30 + #11 15 + #12 15 + #13 20 + #14 15 + #15 15 + #16 15），
  上限约 8h；§0.6/§4.2“~3-4h”只覆盖低端。建议写“3.5h（下限）–8h（上限），C 级”。
- **E9（Low）§1.4 “需重跑场景=11”与所给命令不符**：命令只统计 anchors 的 `source_run` 去重=**9**；
  11 = anchors 9 + `tas-clear-full.l5.jsonl` 的 2 个 run（tas-clear-full-v5/tas-stg07-relay）。建议命令改为并集或分两行注明。
- **E10（Low）trigger 基数未标注**：§0.2 用 44-golden 基（7/32/5），§0.3 用 50-anchor 基（38 injected），与 §1.2 命令输出（50 基）并列易混；
  建议统一写“44 golden 基=7/32/5；50 锚基=7/38/5（36 号任务卡口径）”。

### 4.3 规格缺口（S）

- **S1**：A3/A6/A7 与 L5 门禁（A2）互有依赖（A3 需要 A6 manifest 口径、A2 需要 l5-plan §5 落地），建议在 A 档行加依赖箭头（不阻塞）。
- **S2**：`diff_l5 --probe` 的 `unknown` 语义（golden 无 probe 覆盖 / probe 缺字段面）建议在 A7 契约卡中举例说明（工具 docstring 已足够，可引用）。

### 4.4 平台差异（P）

- **P1**：`analysis/traces` 为 .gitignore 工作区产物 ⇒ L5 锚/L4 基线不可复现同族问题（被审文件已识别，§1.4/C3 ✓）；建议 C3 裁定与 L4 P0-0 合并（文件已建议 ✓）。
- **P2**：Mesen2/`tools/mesen2` 缺失且本机 Windows——B 档“需 Linux+Mesen2”正确；但 engine 侧 probe5 例外（见 E2）。

## 5. 被确认/可直接作为行动依据的部分

- 数字口径（44/39/5、17/16/1、10/10/0、50=44+6、38/7/5、33/12/5、36 命名检查点、l5_s5.rs:68 锁、l5.rs 529 行与 API 行号）**全部独立复算通过**。
- 八关冻结点 6 帧、stg07 观测窗、stg08 注入 #7/OBS-1（候选不立号）、L4 I1-I4 重合、环境缺失三项、L4 手册已改正——**均与库内文档/代码一致**。
- A/B/C 分档整体可执行（角色/范围/判据齐），除 E2 的 B5 环境分档与 E6/E7/E8 的清单/估算需修。
- 跨任务新证算术正确，且 task-50 已派/工作树已落地（E3 只需改状态）。

## 6. 命令与输出摘要（自跑，只读）

| # | 命令 | 摘要 |
|---|---|---|
| 1 | Python 重算 3 个 `.l5.jsonl` + anchors | `44/39/5`、`17/16/1`、`10/10/0`；anchors `50=44+6`；trigger `38/7/5`（50 基）；golden 基 `7/32/5` |
| 2 | Python 证据分类 | `Counter({'traces':33,'shots':12,'none':5})`；唯一 checkpoint=36；scenes 32/9/3 |
| 3 | Python 打印占位/附加锚 | 5 占位：1×`checkpoint=None` + 4 命名；6 附加锚 id/帧/判定式（§4.2 E5） |
| 4 | `Select-String l5_s5.rs` / `l5.rs` | `:68 assert_eq!((real,ph),(39,5))`；`l5.rs` 529 行 + 5 个 API 行号全命中 |
| 5 | `python tools/diff_l5.py --selftest` | **exit 0**（`selftest: ok`） |
| 6 | 冻结帧 grep `stg0*-timeline.md` | 6 帧全部命中（行号见 §2） |
| 7 | `git grep probe5` | `crates/game-core/tests/engine_tick.rs` **10 处**（`emit_probe5` 发射器）+ 文档/tool 4 处（E2） |
| 8 | `git grep -i l5 -- gates` | 无命中 ⇒ L5 无门禁条目（与自述一致） |
| 9 | `python gates/cli.py status` | `docs-links PASS`（332 md/417 links）；`rust-tests FAIL` 系并发任务在飞（与 L5 无关，自述已声明） |
| 10 | `Test-Path` 环境三项 | `analysis/traces`/`shots`/`tools/mesen2` 均不存在；`analysis/`=visual-scout+workset |
| 11 | 读 `l4-baseline-freeze-plan.md`（task-50 工作树） | I5=tas-stg08b2-direct 40000f/10 probe；I5-old 36500f 降级对照；§3.6/§3.7 dated 更正 ⇒ §5“尚未写入”过期（E3） |
| 12 | `team_task_get task-50` | 存在、in_progress（l4-analyst），描述与盘点 §5 建议逐项一致 |

## 7. 修正建议（按优先级）

1. **E2（必改）**：§1.1 probe5 行补“engine 侧 `emit_probe5`（ignored 测试，需先建 `analysis/traces`）”并改 G-L5-6/§3.2 B5 分档
   （A 档=emit_probe5+diff_l5 --probe natural 面；B 档=真机 run 面），消除“仅 2 处文档引用”事实错误。
2. **E3（必改）**：§5 状态改为“已转 task-50（工作树 L4 手册已改，待提交/复核）”。
3. **E10/E9**：trigger/场景数统一标注基数（44 golden / 50 anchors；11=9+2）；E1 类命令与文字对齐。
4. **E4/E5**：占位首行标“未命名 Transition（出店段）”；6 附加锚改为实际 id 清单。
5. **E6/E7/E8**：§0.6/§4.2 清单口径（17 行→有效 14-16）、删 #10 的 stage6/8 候选或改预算、工时改区间 3.5–8h（C）。
6. 复审判据：上述 2 条必改落地 + 5 条 Low 修正后，本复核可直接 **pass**（其余数字/结论已独立复算通过，无需重跑环境）。

> 本复核只写本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。

---

# 复审（2026-10-18，task-53）—— 定点复审 task-52 修正

> 被复审对象：`docs/findings/l5-gap-inventory.md` 修正版（**360 行**，SHA256 `39AE7A9B8411E896D32ADC6CE9F187096708362993088BE7DDFAC210268A58AE`，
> 新增 §9 修正记录表；我上轮核对的行数/内容 s5 数字/指针**未变**）。复审范围=task-51 的 E2/E3 必改 + E4-E10 Low（其余数字/结论上轮已独立复算，不再重跑）。
> **VERDICT（复审）：pass —— 上轮 needs_revision 解除。** E2/E3 已落实且有**本机实跑证据**；E4-E10 逐条落实；
> 剩余 4 条 Info/Low（A9 退出码语义 / A9 scene-map 具体值 / “10 处”计数基 / “E10/E1” 笔误）不阻塞，已在 §R 列明。

## 复审 §A：E2（必改）——已修，且 A9 命令链**本机实跑通过**

**文档侧（逐行核）**：§1.1 两行重写（`:59` run 面产物=发射器事实/写文件列表/不建目录；`:60` 新增“run 面本机对拍（零 Mesen2）”命令链）；
`G-L5-6:163` 改 **A（本机 emit_probe5+diff_l5 --probe natural 7 锚）/ B（Mesen2 真机 run 面）**；`A7:195` 加验收；
**`A9:197` 新增本机卡（命令/判据/12 文件产物/不复用共享 target）**；`B5:207` 改为“真机 run 面重采（A9 已覆盖 natural 面）”；`B1:203` 改“11 run = anchors 9 + bot 域 2”；§8:342 自验收补 probe5 本机面；§9:352 记录。

**独立核（不是只看文档）**：
1. `git grep -c 'probe5' -- crates/game-core/tests/engine_tick.rs` = **10 行**；`emit_probe5` 符号命中 **2 行**（`:656` 注释 + `:669 fn emit_probe5`）——文档“10 处引用”的基是 **probe5 行数**（见 §R3 建议精确措辞）。
2. 写文件语句 5 处：`:718`（stg01..08 循环）+ `:753` gameplay-stage1 + `:783` respawn-window + `:825` s5-chain + `:840` clear-run.trace；**无任何 `create_dir*`**（文档“不建目录”✓）。
3. **实跑 A9**：`mkdir analysis/traces` → `cargo test -p game-core --test engine_tick -- --ignored emit_probe5` → **1 passed**；产物**恰好 12 文件**：
   `stg01..08.probe5.jsonl`（8）+ `gameplay-stage1.probe5.jsonl`（1.09MB）+ `respawn-window.probe5.jsonl` + `s5-chain.probe5.jsonl` + `clear-run.trace.jsonl` ✓（与 A9 判据一致）。
4. **实跑对拍**：`python tools/diff_l5.py --probe analysis/traces/gameplay-stage1.probe5.jsonl --golden oracle/traces/s5-all-clear.l5.jsonl --anchors oracle/traces/s5-all-clear.anchors.jsonl --scene-map title-menu=s5-title,gameplay-stage1=s5-stg01`
   → **natural 7 锚全 `ok`**：`s5-title f180 boot-black / f360 title-in / f540 menu-in` + `s5-stg01 f1080 stg01-shop / f1200 Transition(None) / f1260 stg01-start / f1500 stg01-combat`；
   另 4 行 `ok-injected`（stg01-clear/stg02-shop/stg02-exit/stg02-start），其余 `unknown`（golden 未被 probe 覆盖=工具语义预期）。
   ⇒ E2 的“natural 7 锚可零依赖对拍”**经验证成立**（A/B 分档正确；Mesen2 确非 natural 面必需）。

## 复审 §B：E3（必改）——已修，状态与事实一致

- §0.7:41-44 已写“该条**已转 task-50**（工作树 L4 手册已改，待提交/复核）”；§5:289-295 把“尚未写入/请 Lead 决定”**划线保留**并加 `【2026-10-18 更正·task-52 E3】` 新状态（历史留档 ✓）；§8:341 同步。
- 独立核：`docs/findings/l4-baseline-freeze-plan.md` 为**未提交工作树文件**（`??`）且已含修正——I5=`tas-stg08b2-direct`(40000f/10 probe, `:136`)、`I5-old(36500f)` 降级对照(`:140`)、§3.7 dated 更正/§8 不再跑 v5 36500f（`:145/117` 等）；`team_task_get task-50` = in_progress（l4-analyst，写范围一致）。⇒ 文档状态=事实 ✓。

## 复审 §C：E4-E10 逐条

| ID | 文档落点 | 复核结果 |
|---|---|---|
| E4 | §0.4:30-34、§1.2:89-91 | 已改“5 占位=1 条 `checkpoint=null` 未命名 Transition（出店段）+ stg08-start/combat/clear/all-clear-95f9；golden/anchors 无 `stg08-exit` 字面” ✓（与 jsonl 一致） |
| E5 | §1.2:92-96 | 已改实际 6 附加锚 id（stg02-exit-thanks@3380/gate-26-bit4@2400/flyout-vector@2794/gate300-open@3066/inc-180@3096/stg02-shop-menu@3300）、删 `$9BC3 init` ✓（与 anchors 文件逐 id 一致） |
| E6 | §0.6:35-39、§4.2 注:261-262 | “17 行→有效 14-16 行（#17 取消 M2 + #15/#16 降级对照）” ✓ |
| E7 | §4.2 #10:250 | 已删“四联 stage6/8 候选”；改“bot 域 9 锚 stg01-04 证据；80000 不可能跑到 stage6/8（>110k）” ✓ 与 §5 自洽 |
| E8 | §4.2 注:259-260 | 改“3.5h（下限）–8h（上限）C 级；原 ~3-4h 只覆盖低端” ✓（我上轮逐行下限和 ≈3.5h 一致） |
| E9 | §1.4:137-143、§4.1:228-230、B1:203 | 已拆“anchors source_run 去重=9”+“并集 11=9+bot 域 2（v5/relay）” ✓（与命令/文件一致） |
| E10 | §0.2:20-23、§1.2:99-100 | 已双基注明：**44 golden 基=7/32/5；50 锚基（任务卡口径）=7/38/5** ✓（与 jsonl 重算一致） |

**§9 修正记录表**（`:345-360`）：E2/E3/E4/E5/E6/E7/E8/E9/E10 **9 行一一对应**、级/落点/内容齐 ✓（无漏项；E1 见 §R4）。

## 复审 §D：命令与输出摘要（自跑，只读 + A9 工作区产物）

| # | 命令 | 输出摘要 |
|---|---|---|
| 1 | `(Get-FileHash l5-gap-inventory.md)` / Python 行数 | SHA `39AE7A9B…58AE`；**360 行**（与 Lead 通告一致） |
| 2 | `git grep -c 'probe5' -- crates/game-core/tests/engine_tick.rs` | **10**（`emit_probe5` 符号 2 行，见 §R3） |
| 3 | `Select-String engine_tick.rs 'analysis/traces|create_dir'` | 5 写语句（stg01..08/gameplay/respawn/s5-chain/clear-run.trace）+ **无 create_dir** |
| 4 | `New-Item analysis/traces`；`cargo test -p game-core --test engine_tick -- --ignored emit_probe5` | **1 passed**；产物 **12 文件**（stg01..08+gameplay+respawn+s5-chain+clear-run.trace，尺寸 2.4KB–1.09MB） |
| 5 | `python tools/diff_l5.py --probe … --golden s5-all-clear.l5.jsonl --anchors … --scene-map title-menu=s5-title,gameplay-stage1=s5-stg01` | **natural 7/7 `ok`**；4×`ok-injected`；余 `unknown`（未覆盖 golden）；**exit=1**（见 §R1） |
| 6 | `python gates/cli.py status` | `docs-links PASS`（334 md/422 links）；`STATUS: OK`（PASS=11 FAIL=0 SKIP=1） |
| 7 | `git status` | 被审文件 `?? docs/findings/l5-gap-inventory.md`（未提交，符合“只写不改仓库”）；`l4-baseline-freeze-plan.md` `??`（task-50 工作树） |
| 8 | `team_task_get task-50` | 存在、in_progress（owner=l4-analyst）⇒ E3 状态属实 |

> 注：A9 实跑在 `analysis/traces/` 生成 12 个 probe5/trace 文件（**gitignored 工作区产物、不入库**；可在 A9 卡复用/重跑）。

## 复审 §R：剩余问题（Info/Low，**不阻塞 pass**，建议并入 A9 卡面或下次 docs touch）

- **R1（Low）A9 判据未写“退出码语义”**：`tools/diff_l5.py` 的 `bad` 集合包含 `unknown`（golden 行未被 probe 覆盖）⇒ 只喂
  gameplay-stage1 单文件时**必然 exit=1**（本次实测如此），而 natural 7/7 已 `ok`。建议 A9 判据补一句：
  “验收以 `L5-DIFF[s5-title]/[s5-stg01] natural 7/7` 行为准；exit=1 属预期（unknown=golden 未覆盖行），
  如需 exit=0 用 `--scene`/补全 8 关 probe 或工具增 `--ignore-unknown`”。否则执行者易误判 FAIL。
- **R2（Low）A9 命令 `[--scene-map ...]` 占位未给具体值**：本次实测可用 `--scene-map title-menu=s5-title,gameplay-stage1=s5-stg01`；建议写入 A9 卡面。
- **R3（Info）“10 处引用”基数**：实为 `engine_tick.rs` 中 **probe5 相关 10 行**；`emit_probe5` 符号仅 **2 行**（注释+fn）。建议 §1.1/§9 措辞改
  “probe5 共 10 行（含发射器 fn 与 5 处写路径）；`fn emit_probe5` 2 处”，避免“10 个 emit_probe5 断言”误读。
- **R4（Info）`E10/E1` 引用笔误**：本复审上轮报告 §1 表“见 E1”系笔误（实际编号 **E10**；task-51 报告正文 findings 从 E2 起、无 E1）。
  更新版 §0.2:20 的“task-52 E10/E1”沿袭了该笔误；§9 表无 E1 行、编号一一对应不受影响。**在此 dated 更正**：上轮“见 E1”→“见 E10”；建议 task-52 文档去掉“/E1”。

## 复审结论

**pass**：E2/E3 两项必改**已闭环并附本机实跑证据**（emit_probe5 12 产物 + diff_l5 natural 7/7）；E4-E10 **逐条落实**；
§9 表与 E 编号一一对应；docs-links PASS；被审文件仍以“未提交盘点稿”形态工作，符合任务纪律。
剩余 R1-R4 为 Info/Low 精度项，**不影响本报告作为 L5 规划依据**（建议 A9 卡创建时一并把 R1/R2 写进判据/命令）。
上轮 verdict（needs_revision）至此解除；本复核对修正版的结论为 **pass**。

> 本复审只改本文件（追加本节）；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
