# 对抗复核：改写完成度与像素一致性审计（adversarial-review-completion-audit）

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不得复用被审方结论**）
> 被审对象：`docs/findings/rewrite-completion-audit.md`（539 行，未跟踪；复核时 sha256 `D94A5C1FDC44…`）
> 被审自称基准：HEAD `e7b8285`。复核时点 HEAD：`1be66e5`（复核期间并发推进 7 个提交；`git diff e7b8285..1be66e5 -- crates/game-core/src` 为空 ⇒ workset 输入未变，重导值可比）。
> 纪律：AGENTS.md §3/§5（A/B/C 分级、数字可独立复算、未证不发明、对抗复核）；本文只写本文件，未改被审文档/实现/`rom/`/`reference/`/`data/`/`gates/`，未 git add/commit。
> 环境：Windows + `python`（`python3` 为损坏别名）；生成器重导在 `%TEMP%` 执行（复核后 `analysis/workset/*` 与 `data/status/rewrite-workset.json` mtime 仍为 12:24:30/12:24:49，未被本复核重写）。

---

## 0. VERDICT

**needs_revision（低风险；结论主干全部成立，6 处须修，均不改变任何结论性裁定）**

- **被确认（独立重导一致）**：改写未 100%（L2 15/394=3.81%；≥core 镜像 69/394=17.51% / HEAD 重导 76/394=19.29%）；L4 非逐像素（1/22 帧 diff=0，合计 280,132/1,351,680=20.72%，同点 79.28%）；"镜像过期 7 段"成立且逐条归因可复现；K1/K3/K5/K6/K7 口径冲突属实；"判定式达标+分类账 ≠ 像素级一致"的核心定性正确；Stage A 未达成（L4/L5 两项不满足）成立。
- **被证伪（须修正）**：
  1. **§2.2 待定 R-X / R-? 两行的 6 个 status 交叉格不可复算**（L115-116）——行合计对，列内数据错（把"待定合计"按"R-? 全为 spec-cited"的假设硬拆）；
  2. **§7 P3 RNG 指针 bank 标错**（L365）：`bank15:$D422` 应为 `bank14:$D422`（`$D422∈$C000-$DFFF` 窗口）；
  3. §3.3 L155 断链指针（`plan-platform-abstraction.md` § 缺节号）；
  4. §3.4 L189 门禁行把 2026-10-15 历史留痕当现状（复核 checkout 实测 **PASS=11 SKIP=1**，非 PASS=12 SKIP=0）；
  5. §7 P1 L360 把 B 级"EN-9+ 同格式延续"写进"已证"句；
  6. §4.3 L228"项目治理意义上的签署完成"措辞偏宽（2026-10-18 用户裁定 commit `24093be` 已明确"带声明签署不构成 L4 达成"，应同步收紧）。
- **无结论级错误**：上述修正不改变"改写未 100% / L4 未达成 / Stage A 未达成"三项裁定；修正后可 pass。

---

## 1. 独立重算数字表（我得到的数）

### 1.1 档位段/字节/占比（同分母 394 段 / 131,072B）

命令：读镜像 JSON 逐段累加 + `tools/workset_inventory.py` 重导到 `%TEMP%`（`wi.OUT_DIR`/`wi.MIRROR` 重定向，见 §5 C2'）。

| 档位 | 镜像（提交版） | HEAD 重导（`1be66e5`，workset 输入=e7b8285 同） |
|---|---|---|
| `L2-diff` | 15 段 3.81% / 6,865B 5.24% | 15 / 3.81% / 6,865B / 5.24% |
| `core-cited` | 54 段 13.71% / 29,327B 22.37% | **61 段 15.48% / 29,931B 22.84%** |
| `spec-cited` | 325 段 82.49% / 94,880B 72.39% | 318 段 80.71% / 94,276B 71.93% |
| `findings-cited` | 0 | 0 |
| `unexplored` | 0 | 0 |
| **≥ core-cited（累计）** | **69 段 17.51% / 36,192B 27.61%** | **76 段 19.29% / 36,796B 28.07%** |
| 合计 | 394 / 131,072B | 394 / 131,072B |

→ 与审计 §2.1 三张表**逐格一致**。镜像字节合计 29,327+94,880+6,865=131,072 ✓。

### 1.2 镜像过期 7 段：独立复现（§10 高危点 a）

7 段 delta 与审计 §1.3 清单**逐段一致**，同步节给 `core_files`：`bank00:8185-81A2`(engine.rs)、`bank00:82EF-82FC`(engine.rs)、`bank00:8DDF-8EC7`(ui.rs)、`bank00:929E-9319`(engine.rs)、`bank00:93C1-9409`(engine.rs)、`bank14:CCD3-CD07`(engine.rs)、`bank14:D679-D6C5`(shot.rs)；**delta 字节 = 604B**（29,931−29,327）。

归因复核（`git log -S '<引用串>' -- crates/game-core/src`）：
`bank00:8185-82FA`→`7eb2123`(2026-10-15 21:47)；`bank00:8DEB-8DF3`→`2acb1e1`(10-16)；`bank00:92AA`/`bank00:9400`→`63ab0d8`(10-16)；`bank14:CCD3`→`279911b`(10-16)；`bank14:D67C-D6BB`→`5407dd3`(10-16)。全部为镜像提交 `61ed634`(10-15 21:38) 的**后代**（`git merge-base --is-ancestor` exit=0 ×5）⇒ 审计"镜像生成后有新增引用但未重导"的根因成立。
另：镜像 `cross_check.ida_entry_starts_total=294/ida_export_present=true`；HEAD 重导 `0/false`（`analysis/ida/` 可再生且未生成）——审计 §1.3 该条成立。

### 1.3 class_rule × status 矩阵（发现 §2.2 数据错误的位置）

镜像与 HEAD 的 `class`/`class_rule` 逐段差均=0（审计该前提成立）。实际矩阵（HEAD；镜像除 R-E 的 status 因 7 段 delta 外相同）：

| 规则 | 段/字节 | L2-diff | core-cited | spec-cited |
|---|---|---|---|---|
| R-E 引擎 | 311 / 52,416 | 10 段/2,062B | **51/9,069** | **250/41,285** |
| R-D 数据 | 2 / 379 | 0 | 0 | 2/379 |
| R-G 图形 | 17 / 4,778 | 1/578 | 1/172 | 15/4,028 |
| R-A 音频 | 6 / 1,564 | 0 | 1/115 | 5/1,449 |
| **R-X 待定** | 23 / 70,760 | **3/4,189** | **4/20,493** | **16/46,078** |
| **R-? 待定** | 35 / 1,175 | **1/36** | **4/82** | **30/1,057** |

- 审计 §2.2 的 R-E/R-D/R-G/R-A 四行与上表**一致**；**R-X/R-? 两行 6 个 status 格全部不符**（审计：R-X 4/4,225、8/20,575、11/45,960；R-? 0、0、35/1,175）。
- 成因可定位：审计似把"待定合计"（L2 4/4,225、core 8/20,575、spec 46/47,135）按"R-? 整行=35 段全为 spec-cited（1,175B）"拆开 ⇒ R-X 拿全部 L2/core + 剩余 spec。实际 R-? 含 L2 1 段（`bank15:ED03-ED26` 36B，bcd_sub_ammo）与 core 4 段（`bank00:86CE-86CF` 2B、`bank00:9CE9-9CEB` 3B、`bank03:A2FA-A342` 73B、`bank11:8012-8015` 4B，合计 82B）。
- 影响范围：仅 §2.2 描述表；§2.2 的"R-X 70,760B=54.0% 结构性陷阱"段落与全部结论不受影响（行合计正确）。

### 1.4 L2-diff 15 段（§5 清单）

- `l2_cases.jsonl`：**121 例 / 29 子程序**；镜像 `unmapped L2 entries=[]` ✓。
- 15 段集合与镜像 `status=L2-diff` 集合**完全相等**；段字节合计 **6,865B** ✓；逐段 `l2.subroutines/cases` 与审计 §5 表逐格一致（26/16/15/22/31/11 → 121）。
- "29 名中仅 3 名落在段首"成立：`pickup_handler bank02:922B`、`money_add_event bank14:DBCE`、`bcd_sub_ammo bank15:ED03`。
- §5 表 15 行"core 引用（文件:行 + 引用串）"：15/15 在 `crates/game-core/src/<file>` 对应行号命中引用串。

### 1.5 core-cited 54 段（§6 清单）

- 54 段集合与镜像 `status=core-cited` **完全相等**；字节合计 **29,327B** ✓；class 列逐行与 JSON 一致。
- 51 行显式 `file:line` 指针 51/51 命中；3 行"同上（取框族/菜单族）"继承前一条已验指针（`enemy.rs:568`、`flow.rs:55`）。
- "首个相交引用行位于注释行 = 54/54"独立复现：**54/54** ✓；全量 177 条相交引用行均为注释（含 1 条行尾注释 `flow.rs:743`，故"注释级"结论成立）。

### 1.6 L4 实测（§10 高危点 b）

独立解析 `oracle/screens/visual-report.md`（22 对）：

| 量 | 值 | 审计值 |
|---|---|---|
| 帧数 / 总像素 | 22 / 1,351,680 | 一致 |
| diff=0 帧 | **1**（title-menu_f00180_boot-black）| 一致 |
| 差异合计 / 占比 | **280,132 / 20.7247%** | 280,132 / 20.72% ✓ |
| 同点 / 占比 | **1,071,548 / 79.2753%** | 79.28% ✓ |
| <7680 帧 | **12/22 = 54.55%** | 一致 |
| ending-1 单帧同点 | **56,888/61,440 = 92.5911%** | 92.59% ✓ |
| ending 五值 | 4552/3682/3347/3981/3981 | 一致 |
| 最大单帧 | fade-clear 59,371（同点 3.37%）、clear-inject 58,101（5.44%） | 一致 |
| 四分类构成 | 条带外 BG 266,886 + 条带内 W-C1 13,246 = 280,132 | 审计分族举例一致 |

→ 与审计 §4.2 逐项一致（§10 高危点 b 未被证伪）。

### 1.7 子系统/测试/模块（§3.3）

- 13 个 `game-core/src/*.rs` 合计 **6,853 行** ✓（player 145/weapons 155/shot 460/enemy 1161/terrain 473/wave 555+wave_record 196/stage 265/flow 745/ui 647/lib 264 全部命中；engine 1588、l2ports 199 未列入子表但计入 13）。
- `game-core/tests/*.rs` `#[test]` 合计 **185** ✓；审计时点全 crate 280 = 当时 `baseline.json` ✓（复核时因 task-5 已上调至 326，审计 §3.3 已预先声明该并发口径）。
- 子系统 L2 映射 26/16/15/22/31/11 可复算 ✓；6/13=46.15% 算术正确。映射是审计自声明口径（尤其 `bcd_sub_ammo`→炸弹、`death_money_halve`→商店/金钱），有 R-fire-4（`$018F` 弹药经 `$ED03` 扣减）等 spec 支撑，**可接受但应标注"本审计口径"而非仓库口径**。

---

## 2. 证据指针抽查（实际 20+ 条；四高危点见 §1.2/§1.6/§1.7/§3）

| # | 审计指针 | 审计主张 | 我的复核 | 结论 |
|---|---|---|---|---|
| 1 | §1.2 L41-43 三代 workset | 文档快照 247 / 镜像 core 54 / HEAD core 61 | `docs/spec/rewrite-workset.md` L154-162（≥core 49、spec 79、findings 147、unexplored 247）✓；`git log` 时点 fe34e43/61ed634 ✓ | 成立 |
| 2 | §1.3 L49-57 7 段归因 | 逐段 commit 归因 | `git log -S` 5 commit 全命中且均为镜像 commit 后代（§1.2）| 成立 |
| 3 | §1.3 L59 ida 294 | 干净检出不可复现 | 镜像 294/true、重导 0/false ✓ | 成立 |
| 4 | §3.2 L142 | 121 例全落 15 段 6,865B(5.2%) | `rewrite-workset.md` L135 原文一致 ✓ | 成立 |
| 5 | §3.4 L184 | L2 121/121 留痕于 review-74/75/77/89/93 | 逐文件命中：74:L22、75:L21、77:L24、89:L80、93:L119 ✓ | 成立 |
| 6 | §3.4 L185 | L3 `TRACE-DIFF: ALL MATCH (8/8)` | review-75 L77 原文 ✓ | 成立 |
| 7 | §3.4 L187 | review-72 §4.3 分层 + "能走通这条链 ≠ 能打通这 8 关" | review-72 L208-228 原文 ✓（含 6 层达成度表）| 成立 |
| 8 | §3.4 L189 | 门禁 12 门 PASS=12 SKIP=0 | review-74 L23 / l5-all-clear L210 是**2026-10-15 历史**；复核 checkout 实测 **PASS=11 SKIP=1**（addr-bridge SKIP，JP ROM 未入库）| **时效缺口**（E2）|
| 9 | §4.1 L208 | l4-visual-plan §1："白名单外像素 100% 一致（0 容差默认）…白名单外 FAIL" | 原文 L14 **逐字一致** ✓ | 成立 |
| 10 | §4.1 L209/211 | 签署判据=判定式+声明封存；diff_visual L9 退出码语义 | l4-signing-criteria L5-8/§6；diff_visual L9 原文 ✓ | 成立 |
| 11 | §4.2 L215-221 | 22 对/L4 数字 | §1.6 逐项复算一致 ✓ | 成立 |
| 12 | §4.4 L240 | l4-final-localization-ending1 标题 7753 态/7380px/261 格 | 文件 L1 原文 "7753态残差7380px/261格" ✓ | 成立 |
| 13 | §4.4 L239 | residual ledger 10 族 9 维持 + AR2-A 撤销 | §4 表 10 行 + L57-59 更正注 ✓ | 成立 |
| 14 | §4.5 K1 | 43.0/55/61 三值并存 | l4-signing-criteria L26(4236=55%)/L37(43.0%)/L63(61%,4675)；全仓无 43.0% 推导来源 ✓（K1 成立）| 成立 |
| 15 | §4.5 K2 | NOT READY→转正 5/5 判据切换无裁定 | l4-visual-plan L352 vs L375-382 原文 ✓；**复核时点已有用户裁定 commit 24093be**（见 §3 裁定）| 成立（现已裁定）|
| 16 | §4.5 K7 | 双有/7680 分母未定义 | l4-visual-plan L356-366："864/7680=11% 形合率"、7680=ending-1 基准 ✓ | 成立 |
| 17 | §5 L263-277 | 15 段 L2 清单 | 段集合/字节/子程序/用例/15 条 core 指针全验 ✓ | 成立 |
| 18 | §6 L290-343 | 54 段 core 清单（注释级） | 54 段集合相等、51 显式指针命中、3 同上、54/54 首引用为注释 ✓ | 成立 |
| 19 | §7 L357 P0 | `bank8:$A000` 目录 8B×8、`bank8:$ADD0-$ADF7`、`bank15:$E392 CMP #9` 通关判定 | seed 命中 A000/ADD0/ADF7；E392 字节 `C9 09 = CMP #9` ✓ | 成立 |
| 20 | §7 L359 P1 | H-16 `bank14:$D7A3-$D7EC` 判定盒、`$D613-D673`、`$D86B` 状态表 | seed 四处命中 ✓ | 成立 |
| 21 | §7 L365 P3 | RNG `bank15:$EC12` 算术；**`bank15:$D422/$EC1F`** | EC12 从原始字节解码 = `LDA $2C/ADC $2D/ADC #1/EOR $2B/STA $2C/INC $2D/RTS` ✓；EC1F ✓；**$D422 在 bank14 窗口**（seed `bank14:D422`；workset 表 `bank14:D422-D4FD`）| **指针错**（D2）|
| 22 | §7 L368-369 K4 | H-01/H-02 已收口；商店消费侧 L2 15 例 | ram-watch-s3 §9.1 L201-219 ✓；shop-purchase-flow §5 表 3+2+5+5=15 ✓ | 成立 |
| 23 | §1.4 L65-68 | coverage_map 54/44、completeness 19/59、m3-inventory 严重过期、visual-report 现行 | 逐项命中（coverage `generated=2026-10-12, l2_cases=54, rules=44`；completeness `19/59`；m3 §2 shots/enemy/stage "❌ 未动"）| 成立 |
| 24 | §4.4 L238 | `gates/visual_whitelist.json` 不存在 | `Test-Path` = false ✓ | 成立 |
| 25 | §10 L534 | 重导不写工作区（analysis/workset mtime 12:24）| 我的 temp 重导前后镜像/analysis 产物 mtime 不变（12:24:49/12:24:30）✓ | 成立 |

---

## 3. 门禁与测试独立重跑（审计完整性缺口核对）

审计只跑 `rom`/`seed`/`naming` 三门并声明未重跑 cargo/L2/L3/L4/check（§9.1 诚实披露）。我的独立重跑（**HEAD ff5aa7c**；此后 HEAD 仅余并发文档/task-1 施工改动）：

| 项 | 我的实测 | 说明 |
|---|---|---|
| `python gates/cli.py check` | **CHECK: PASS；SUMMARY: PASS=11 FAIL=0 SKIP=1 TOLERATED=0**（exit 0）| 12 门中 11 PASS；**addr-bridge SKIP**："坐标桥或双 ROM 缺失（条件 C）"——`reference/jp/gunnac_jp.nes` 未入库（`reference/jp/` 仅 MANIFEST.md，资产政策如此）；**clean checkout 预期 SKIP，不是回退** |
| rust-tests | PASS；**`cargo test --workspace` 326 例（基线 326）** | 审计时点 280=当时基线；+46 为 task-5 新增；基线已由 commit `ff5aa7c` 上调 280→326 |
| subroutine-diff (L2) | PASS | 121/121 留痕可复现（未重跑逐例输出，但门禁 exit 0）|
| trace-diff (L3) | PASS | 同上 |
| visual-diff (L4) | PASS（exit 0） | **关键口径**：PASS=退出码 0（"全部对通过 **或** 全部差异有显式四分类记录"），**不等于像素一致**——本机 20.72% 差异下仍 PASS，正是审计 §4.1 指出的口径裂缝，复核确认 |
| `python gates/cli.py visual`（单独） | PASS，exit 0 | 重写 `oracle/screens/visual-report.md`+`diff/*.png`+`whitelist-application.md`：复核前后 **558 个文件 sha256 全同**（S1=S2=S3 快照），`git status oracle/screens` 为空 ⇒ 重写确定性、无内容变化，可安全复跑 |
| 并发基线变动 | `knowledge_entries 22→23`、`rust_test_count 280→326` 复核期间被并发上调 | 我的 check 运行跨越该上调，最终态 ratchet 全 OK；复核报告以最终态为准 |

**完整性缺口结论**：审计"只跑三门"是**自认且不构成虚假**；但 §3.4 门禁行引用的 "12 门 PASS=12 SKIP=0" 是历史留痕、**不是当前 checkout 可复现状态**（当前 11 PASS + 1 显式 SKIP）。审计应在该行补当前态与 SKIP 原因（修正建议 F4）。

> **附带发现（门禁健壮性，供 Lead 转 gates 负责方，非本审计错误）**：复核末尾并发施工 `crates/shell-win`（未跟踪 `main.rs` 半成品）时，`python gates/cli.py check` 在 `gate_rust_tests`（`gates/cli.py` L773-781）崩溃：`subprocess.run(..., text=True)` 用 locale(cp936) 解码 cargo 的非 GBK 输出失败 ⇒ reader 线程异常 ⇒ `proc.stdout=None` ⇒ L780 `proc.stdout + proc.stderr` TypeError，check 以 traceback+exit 1 结束（不产生 SUMMARY）。**不构成假绿**（仍非 0 退出），但"门禁全绿"只在干净/可解码工作区可复现；建议 gate 子进程统一 `encoding='utf-8', errors='replace'`。本复核 §3 的 PASS 结论绑定干净态 HEAD `ff5aa7c`，不受此影响。

---

## 4. K1 / K2 / K7 口径裁定建议（供 Lead/用户会签）

> 原则引用：`l4-signing-criteria.md` 自己的"**未对账数字不入签署面**"（L37 数字纪律注）+ AGENTS.md §3"数字可独立复算"。

- **K1（对位率 43.0% / 55% / 61%）— 建议裁定：以可复算值为准，43.0% 撤销或补推导。**
  - 可复算仅两个：量控后 `4236/7680=55.16%`（L26）、量控前 `4675/7680=60.87%`（L63）；`43.0%`（L37）全仓无算式/无来源（我 grep `43\.0|3302` 仅本文自引）。
  - 且 7680 是**判定式阈值**（ending-1 基准），不是"内容量"（同文件 8046 态才是内容量，见 l4-visual-plan L356-366）——把 `双有/7680` 叫"对位率/形合率"本身语义未定义（即 K7）。
  - 建议：spec-writer 在 l4-signing-criteria §1 给算式与定义；对位率统一为 `双有/7680` 并**注明量控阶段**；若要表达"形合率"改用内容量分母；43.0% 删除或给独立复现命令。**一次文件内不得三值并存。**

- **K2（判据切换：0 容差 vs 判定式阈值）— 建议裁定：stage A DoD 唯一判据 = master-plan §6 / l4-visual-plan §1 的"白名单外像素 100% 一致（0 容差，唯一例外=白名单且需用户确认）"；"判定式转正 5/5 + 带声明签署"只是过程里程碑，不构成 L4 达成。**
  - 复核确认：l4-visual-plan L352 明确 "签署判定：NOT READY（0 容差目标；白名单外像素 0 目标远）"，而 L375-382 的"L4→100% 战役收官"改用了阈值判据且无用户裁定记录（审计时点）——**审计 K2 定性正确**。
  - 复核时点已出现落地裁定：commit `24093be`（`docs/knowledge/decision-l4-zero-tolerance-ruling.md`）记录 2026-10-18 用户裁定，与上述裁定一致 ⇒ 建议：审计 K2 由"待裁定"更新为"已裁定（24093be），结论=L4 未达成"；l4-signing-criteria 终签段/l4-visual-plan L375-382 加"过程里程碑，不构成达成"互链注。
  - 顺带：审计 K2 原文"**若** Stage A DoD 仍按 master-plan §6 的 0 容差判据"应去条件化——master-plan §6 就是 DoD，没有第二套；已由 24093be 明示。

- **K7（7680 作形合率分母）— 建议裁定：7680 只能作"阈值"，不可再当"形合率/对位率"分母；形合率分母应为内容量（dg，如 8046 态），并注明时点。**
  - 原文证据 l4-visual-plan L360-366：diff=14344（基准 7680）、内容量 dg=8046、双有=864、"864/7680=11% 形合率"——同一表引入两个语义不同的量并混用为分母。
  - 建议：`双有/内容量`（如 864/8046）或明确改名"占阈值比"；引用 11%/55%/61% 时逐条带"分母=7680（阈值）"限定。

---

## 5. 证伪分析：越权 / 反向夸大 / 口径混用

1. **"判定式达标+分类账 = L4 已完成"是否越权？——越权，但审计没有犯此错，反而揭发它。**
   - 该越权表述来自被引原文（l4-signing-criteria L7-8/L90-92 "L4 100% 签署…L4 齐=全判据齐 ⇒ DoD 终签呈报出"；l4-visual-plan L382 "L4→100% 战役收官"）。
   - 审计 §0.2/§3.4/§4.3 **明确拒绝**"像素级一致"等价，并把它列为口径裂缝 K2 ⇒ 此点**确认审计正确**。唯一偏宽处：§4.3 L228 说这是"项目治理意义上的签署完成"——在 24093be 裁定后应改述为"过程里程碑签署"（修正 F6）。

2. **"改写未 100%"是否反向夸大（低估 L2/L3）？——结论方向正确，无实质夸大，但有被误读风险。**
   - 3.81%/19.29%/5.24%/28.07% 是**引用覆盖**口径（workset 段/字节），不是行为完成度；审计已在 §2.2 末尾与 §3.3 用"13/13 有代码、6/13 有 L2、0/13 全闭环"三读数对冲，并在 §3.5 补 L2 121/121、L3 8/8、L4 22 对为"workset 未计入的机器证据"。
   - 建议（非错误）：§0.1 "L2-diff（唯一带机器差分证据的档位）"加"workset 档位内"限定，并加一句"引用覆盖≠行为完成度，行为面见 §3.3"，避免"仅 19% 已改写"的误读。

3. **数字口径混用检查**：
   - 段/字节/单帧/全量：审计分列正确；**92.6% 单帧 vs 79.28% 全量**已由 K3 显式限定 ✓；`4236/7680`、`4675/7680`、`4552/61440` 均可复算 ✓。
   - **但 §2.2 待定两行把"行合计"当"状态列"**（D1），属"口径混用型数据错误"；`43.0%` 与其他 L4 历史量（83.9%、31.0%）不得与新实测混列（K1）。

4. **未证不发明抽查**：
   - §7 各行的"已证/未证"分列总体严格；`bank15:$EC12` 属合法 mid-line 入口（线性扫描不出、原始字节可解），审计 A 级定位成立。
   - 唯一越级：§7 P1 L360 把 `EN-9+ = $ADF8+ 同格式延续` 放进"已证"句；源头 `w28-boss.md` §2.3 是"子表族注记"（B 级格式延续推断）⇒ 建议移入未证或标 B（E4）。
   - §7 建议任务卡的完成判据总体可执行（L2/trace/门禁可复现）；P0 的"B-STG-xx-EN 子格按约束①准入"中"约束①"未在本审计内定义，建议改为直接引用 `docs/spec/l5-all-clear.md` 对应条目（低）。

---

## 6. Findings 四分类 + 逐条修正建议（精确到行）

### 6.1 数据错误（必须改）

- **D1（中）** 审计 L109-116 §2.2 待定两行 6 个格错。改为实测矩阵（HEAD=镜像，见 §1.3）：
  `R-X 23 段/70,760B | L2 3 段/4,189B | core 4/20,493B | spec 16/46,078B`；
  `R-? 35 段/1,175B | L2 1/36B | core 4/82B | spec 30/1,057B`。
  并注明"class 差异=0 已验证，但 status×rule 矩阵须逐段过滤，不能用待定合计反推"。
- **D2（低）** 审计 L365 §7 P3：`bank15:$D422/$EC1F` → **`bank14:$D422`** + **`bank15:$EC1F`**（`$D422∈$C000-$DFFF`=bank14 窗口；seed `bank14:D422 29 1F`；`rewrite-workset.md` L532 亦标 `bank14:D422-D4FD`）。

### 6.2 证据不足（补指针）

- **E1（低）** 审计 L155 §3.3 行 1：`（plan-platform-abstraction.md §）` → 补 `§3.3 输入消解（平台侧统一消解）`/`L76`；并注明该文件在审计时点为 task-5 未提交文档，须带版本（commit `6362895`/`0809317` 之后才入库）。
- **E2（中）** 审计 L189 §3.4 门禁行：在"最近有记录 PASS=12 FAIL=0 SKIP=0（2026-10-15）"后补：**"复核 checkout（HEAD ff5aa7c）实测 PASS=11 SKIP=1（addr-bridge SKIP：`reference/jp/gunnac_jp.nes` 未入库，资产政策预期）"**；并说明 audit 未重跑的完整性缺口已由复核补跑（§3）。
- **E3（低）** 审计 L359 §7 P1：`analysis/traces/{fire-hold,respawn-window}.frames.jsonl（可再生）`——复核时 `analysis/traces/` 为空（可再生工作区不入库），当前不可直接复现；建议补再生成命令或改指针为 findings 内已入库记录。
- **E4（低）** 审计 L360 §7 P1："已证：…；EN-9+ = `$ADF8+` 同格式延续；未证：…"——`$ADF8+` 延续在 `w28-boss.md` §2.3 为 B 级注记，建议从"已证"移入"未证/待证"或标 B。

### 6.3 口径问题（补限定/会签）

- **C1（低）** 审计 L365 head（§0.1 L13）：`L2-diff（唯一带机器差分证据的档位）` → `（workset 档位内唯一…；另有 L2 121/121、L3 8/8、单测 280/326 见 §3.3/§3.5）`；并加"引用覆盖≠行为完成度"一句。
- **C2（低）** 审计 L228 §4.3 item4："项目治理意义上的签署完成" → "**过程里程碑签署（2026-10-18 用户裁定 24093be：不构成 L4 达成）**"（与 K2 裁定同步）。
- **C3（低）** 审计 L169-173 §3.3 子系统读数：把 6/13 的域映射标注为"本审计口径（非仓库既有口径）"，尤其 `bcd_sub_ammo`→炸弹、`death_money_halve`→商店/金钱 两条。
- **C4（低）** 审计 L142 §3.2 引用 `rewrite-workset.md §3.2` 同时应注明该文档为 2026-10-15 快照（其 §4 的 ≥core 49 与镜像 54/HEAD 61 不同代，见 §1.2）；避免读者把"6,865B"与旧文档其他数字混读。

### 6.4 表述越权

- **O1（低）** 见 C2（唯一近似越权表述，且方向是被审计文件原文，审计已揭发）。
- **O2（低）** 见 E4。
- **O3（低）** 审计 §0 与 §3.4 的"12 门"表述易被读成"当前 12/12 全绿"；以 E2 修正（12 门=门数，当前 11 PASS+1 显式 SKIP）。
- 反向夸大缺口：**未发现实质问题**（§4.3 item5 已主动声明"差异不是渲染算法全错"；22 对全分类、12/22 在阈值内、根因定位均照录）。

---

## 7. 我的复现命令与输出摘要

> 全部在仓库根执行；`python`（非 python3）；生成器重导重定向 `%TEMP%`。

**C1' 镜像各档位（§1.1 左列）**
```powershell
@'
import json, collections
d=json.load(open('data/status/rewrite-workset.json',encoding='utf-8')); TOT=131072
c=collections.Counter(); b=collections.Counter()
for s in d['segments']: c[s['status']]+=1; b[s['status']]+=s['seg_end']-s['seg_start']
for st in ['L2-diff','core-cited','spec-cited']:
    print(st,c[st],round(c[st]/394*100,2),b[st],round(b[st]/TOT*100,2))
print('>=core',c['L2-diff']+c['core-cited'],b['L2-diff']+b['core-cited'])
'@ | python -
# 输出：L2-diff 15 3.81 6865 5.24 / core-cited 54 13.71 29327 22.37 / spec-cited 325 82.49 94880 72.39 / >=core 69 36192
```

**C2' HEAD 只读重导（§1.1 右列、§1.2）**
```powershell
@'
import sys, json, tempfile, pathlib, io, contextlib, collections
sys.path.insert(0,'tools'); import workset_inventory as wi
tmp=pathlib.Path(tempfile.mkdtemp(prefix='ws_verify_'))
wi.OUT_DIR=tmp/'workset'; wi.MIRROR=tmp/'rewrite-workset.json'
with contextlib.redirect_stdout(io.StringIO()): wi.main()
new=json.load(open(tmp/'rewrite-workset.json',encoding='utf-8'))
old=json.load(open('data/status/rewrite-workset.json',encoding='utf-8'))
print(new['counts']['status'], new['counts']['class_x_status'] if False else '')
print('unmapped',new['l2_unmapped_entries'],'xcheck',new['cross_check'])
om={s['seg']:s for s in old['segments']}
for s in new['segments']:
    if om[s['seg']]['status']!=s['status']: print('DELTA',s['seg'],om[s['seg']]['status'],'->',s['status'],s['core_files'])
'@ | python -
# 输出：{'L2-diff':15,'core-cited':61,'spec-cited':318,...}；unmapped []；xcheck ida 0/false；
#       DELTA 7 行与审计 §1.3 逐段一致；重导后 analysis/workset/* 与镜像 mtime 不变。
```

**C3' §2.2 矩阵复算（D1）**：按 `s['class_rule']` 分组 × `s['status']` 累加（镜像与 HEAD 各跑一次，两版 R-X/R-? 完全一致）；输出见 §1.3。

**C4' L2/§5**：`len([...l2_cases.jsonl])`→121、`len(set(subroutine))`→29、`unmapped []`；15 段集合=`status==L2-diff`；逐段 `l2.subroutines/cases` 与 §5 对照；15 条 core 指针 `file:line` 含引用串。

**C5' §6/54 段**：解析审计表 54 行 → 集合=镜像 core-cited 54；字节和 29,327；51 显式指针 `pathlib + lines[n-1]` 命中；3 行"同上"继承；另用 `wi.CITE` 独立扫 177 条相交引用，首引用注释 54/54。

**C6' L4 数字（§1.6）**
```powershell
@'
import re
rep=open('oracle/screens/visual-report.md',encoding='utf-8').read()
pairs=re.findall(r'- ([\w\-.]+\.png): [^\d]*(\d+)',rep); diffs=[int(n) for _,n in pairs]
tot=len(diffs)*61440
print(len(diffs),sum(1 for d in diffs if d==0),sum(diffs),'%.4f'%(sum(diffs)/tot*100),'%.4f'%((tot-sum(diffs))/tot*100))
print(sum(1 for d in diffs if d<7680),'%.4f'%((61440-4552)/61440*100))
'@ | python -
# 输出：22 1 280132 20.7247 79.2753 / 12 92.5911
```

**C7' 门禁（§3）**
```powershell
python gates/cli.py check     # CHECK: PASS; SUMMARY: PASS=11 FAIL=0 SKIP=1 TOLERATED=0; rust-tests 326(基线 326)
python gates/cli.py visual    # [PASS] visual-diff; exit 0
cargo test --workspace        # 由 rust-tests 门禁实跑：326 passed（审计时点 280）
# oracle/screens 重写前后 558 文件 sha256 全同；git status oracle/screens 空。
```

**C8' ROM/seed 指针（§2 表 #19-21）**：`reference/seed/gunnac.asm` grep `bank15:E392`（`C9 09 CMP #9`）、`bank14:D7A3/$D7EC/$D613/$D673/$D86B`、`bank08:A000/ADD0/ADF7`、`bank14:D422 29 1F`（=bank14，非 bank15）；`bank15:EC12` 由原始字节从 EC12 起解码（线性扫描落 EC11+1）。

**C9' K1/K2/K7 原文**：`l4-signing-criteria.md` L26/L37/L63、§4 L63-74；`l4-visual-plan.md` L14/L352/L356-366/L375-382；`git log -1 -- docs/knowledge/decision-l4-zero-tolerance-ruling.md`→`24093be`。

---

## 8. 复核边界与不确定项

1. 复核期间仓库并发推进（audit HEAD `e7b8285` → 复核时 `1be66e5`，7 commits；另有 task-1/5/6/7 并行施工与 `.tmp_verify_*` 未跟踪文件）。所有数字均绑定到具体 HEAD：workset 重导绑定"`game-core/src` 自 e7b8285 未变 + docs/spec 变更后仍 318"；门禁实测绑定 `ff5aa7c`。若最终验收 HEAD 再变，建议重跑 C2'/C7'。
2. 我未重跑 `tools/diff_subroutines.py` 的逐例 stdout，仅以门禁 exit 0 佐证（审计 121/121 另有 5 份复核留痕，抽查指针有效）；`tools/diff_traces.py` 同。
3. 未验证 `analysis/` 可再生工作区产物（traces/ida），仅按其"可再生/不入库"属性处理（E3）。
4. 本复核不裁决游戏语义；§7 建议任务卡的合理性仅到"指针存在/判据可执行"层。
5. 被审文档为未跟踪文件（sha256 `D94A5C1FDC44…`，539 行）；若审计方按本文修正后内容变化，需对修订版复审（重点：D1/E2/F6 三项）。

---

【2026-10-18】decomp-verifier，复核 HEAD `1be66e5`；被审 HEAD `e7b8285`。

---

## 9. 复审（2026-10-18）——定点复审

> 复审对象：`docs/findings/rewrite-completion-audit.md` 修订版（**585 行**；sha256 `BA01C1FC1A06A6EF1F40EE3E8FE388DD1D3254E4F79D385E6C83795882F8E954`；未提交）。
> 复审范围：**只审 D1 / E2 / C2·F6 三项必改 + 应改 5 项落实 + 未引入新错误**（不做全量重审；§1-§8 初版结论继续有效）。
> 复审时点：HEAD `16fb5b2`（gates 实测同 HEAD；门禁 UTF-8 解码修复 `a11161d` 已入库）。

### 9.1 VERDICT：**needs_revision（仅 1 项微修 R1；其余 8 项全部确认）**（**9.5 已闭环：最终 verdict = pass**）

- 三项必改（D1/E2/C2·F6）**全部落实且我实跑复现**；应改 5 项全部落实；未发现结论级新错误。
- 唯一阻塞为新引入的**口径过度声明**：`§8 C9 标题 L530`"镜像与 HEAD 重导同值"与 `§2.3 L125`"C9 给出 §2.2 矩阵…逐格复算"**不成立**（详见 9.2 R1）。修 1 句/补 1 行期望值即可 pass，不涉及任何数字结论。

### 9.2 逐项确认/驳回

**1) D1 §2.2 矩阵（L116-119 + 矩阵口径注 L119）——确认修复**
- 按 §8 C9 命令**逐字抽出执行**，输出：
  `R-X 23 段 / 70,760B | L2 3段/4,189B | core 4段/20,493B | spec 16段/46,078B`
  `R-? 35 段 / 1,175B | L2 1段/36B | core 4段/82B | spec 30段/1,057B`
  → 与修订表**逐格一致**（初版错值 4/4,225、8/20,575、11/45,960、0、0、35/1,175 已全部消除）。
- 行合计：4,189+20,493+46,078=70,760 ✓；36+82+1,057=1,175 ✓。
- R-? 四个 core 段逐条复核：`bank00:86CE-86CF`(2B)、`bank00:9CE9-9CEB`(3B)、`bank03:A2FA-A342`(73B)、`bank11:8012-8015`(4B)，合计 82B ✓；`bank15:ED03-ED26` 为 R-?/L2 36B ✓。
- **R1（唯一阻塞，微修）= C9 的口径句**：C9 读**镜像**，而 §2.2 表是 **HEAD**；7 段 delta 全部落在 R-E ⇒ **R-E 行 C9 与 §2.2 不符**：C9/镜像 `core 44段/8,465B、spec 257段/41,889B` vs §2.2/HEAD `core 51段/9,069B、spec 250段/41,285B`（2 格）。D1 目标行（R-X/R-?）确为镜像=HEAD 同值，故 **D1 修复本身正确**；错在标题"镜像与 HEAD 重导同值"与 §2.3"逐格复算"的外延。
  修法（二选一）：(a) C9 标题改"镜像口径；R-X/R-? 与 HEAD 同值（D1 目标）；R-E 行 HEAD 为 51/9,069+250/41,285（差 7 段 delta=604B，§1.3）"，§2.3 改"C9 复算 §2.2 的 R-X/R-? 两行 + 镜像全表"；或 (b) C9 改为 HEAD 重导（复用 C2 的 `wi.OUT_DIR/wi.MIRROR` 重定向）后再打印矩阵，并补 R-E 期望值=51/9,069+250/41,285。修后即 pass。

**2) E2 §7 P3 RNG 指针（L376）——确认修复**
- 现文：`bank14:$D422` / `bank15:$EC1F`，并注窗口（`$D422∈$C000-$DFFF`=bank14；`$EC12/$EC1F∈$E000-$FFFF`=bank15）。
- 独立佐证：seed `bank14:D422 29 1F`；workset 段 `bank14:D422-D4FD`；`b2-tas-input.md` §6 的 `bank15:$EC12/$EC1F` 保持 bank15。
- 全 0 处 `bank15:$D422` 实质引用（唯 L574 §10 修订条以"旧值→新值"引号形式留痕，属修订记录、非错误）。
- 全文 `bankNN:AAAA` 引用扫描：**205 处 / 172 unique**；与 bank 窗口（bank14→$C000-$DFFF、bank15→$E000-$FFFF、$A000 窗=1/3/7/8/12、$8000 窗=0/2/4/5/6/9/10/11/13）比对，唯一越界即 L574 的历史旧值引用；其余 **204/204 自洽**。

**3) C2/F6 K2 去条件化（§0 L15、§4.1 L219、§4.3 L238、§4.5 L259、§3.4 L193、§3.5 L206）——确认修复**
- L238 已改为"只是**过程里程碑签署**（2026-10-18 用户裁定 `24093be`：**不构成 L4 达成**）…L4 判定=未达成"；L259 K2 已改为"**已裁定**（0 容差；签署仅过程里程碑）…K2 原'待裁定/若…'措辞作废"；L219/L193/L206/§0 L15 同步引用 `24093be` 与 decision 文件。初版"项目治理意义上的签署完成"正文已不存在（仅 §10 修订记录引用）。
- 复核时点旁证：`24093be` 已入库（含 `docs/knowledge/decision-l4-zero-tolerance-ruling.md`、`l4-signing-criteria.md` 顶部裁定注、`master-plan.md` §6 注）；`l4-visual-plan.md` L352 NOT READY / L375-382 收官原文未变（其"收官"仍属被裁定为过程里程碑的原文，审计引用口径正确）。

**4) 应改 5 项落实——全部确认**
- E1 → L160 已补 `plan-platform-abstraction.md` **§3.3 输入策略 / L76**（实测该文档 §3.3 标题正是"输入策略（平台侧统一消解）"，L76 为 `velocity_from_input` panic 句；版本注 `0809317`/`6362895` 正确）。
- 门禁历史 vs 现状 → L196 同时给历史 `PASS=12/SKIP=0`（2026-10-15 留痕）与现状 `PASS=11/FAIL=0/SKIP=1`（HEAD `ff5aa7c` 复核实测；addr-bridge SKIP=JP ROM 未入库政策预期）✓；我方 2026-10-18 于 HEAD `16fb5b2` 复跑同样得 `PASS=11/SKIP=1`（见 9.4）。
- EN-9+ 降级 → L371 已改"**B 级推断（未证）**：EN-9+ = `$ADF8+` 同格式延续（源 `w28-boss.md` §2.3 **注记**，非定谳）"✓。
- §0/§3.1 引用覆盖限定 → L14"**workset 档位内**唯一带机器差分证据…**引用覆盖 ≠ 行为完成度**"、L140 独立口径限定段 ✓。
- §10 复核修订条 → L572-583 逐条列 D1/D2/E1/E2/E4/F6·C2/C1/C3/C4/E3 共 10 项、声明结论主干不变、末行双时点 ✓。
- 附带确认：C3 域映射口径声明 L180 ✓；C4 rewrite-workset 快照限定 L147 ✓；E3 traces "可再生/不入库、当前不可直接复现"+再生成命令 L370 ✓（`oracle/scenarios/fire-hold.txt`、`respawn-window.txt` 存在，`scenario_run.py run <name>` 语法有效）。

**5) 未引入新错误（定点抽查）**
- §5（15 行）/§6（54 行）表格独立重解析：段集合相等、字节和 6,865 / 29,327、51 条显式 `file:line` 指针全命中、3 条"同上"继承、0 失配（与初版结论一致）。
- 全文旧措辞残留 grep：`若 Stage A DoD`/正文`项目治理意义`/旧错值 `4,225`、`20,575`、`45,960` 均无残留（仅 §10 修订记录引用）。
- 新引用有效性：`completeness.md` §8「防钝化三约束」存在（L663）；`hit-sampling-h16.md` §3/§4 存在（L48/L68）；`plan-platform-abstraction.md` §3.3/L76 存在。
- 门禁/链接：docs-links PASS（310 md / 262 links，含本复核报告与修订版审计）✓。

### 9.3 剩余问题清单

| ID | 级别 | 位置 | 问题 | 修法 |
|---|---|---|---|---|
| **R1** | **阻塞（微修）** | §8 C9 标题 L530 + §2.3 L125 | "镜像与 HEAD 重导同值 / C9 逐格复算 §2.2"不成立：C9 读镜像，§2.2 是 HEAD，R-E 行 2 格不符（镜像 core 44/8,465+spec 257/41,889 vs HEAD 51/9,069+250/41,285；7 段 delta 全在 R-E） | 见 9.2-1 (a) 或 (b)；一行修正即可 |
| R2 | 非阻塞建议 | §7 L370 | 再生成命令 `python tools/scenario_run.py run …` 依赖 Linux/oracle 宿主（`scenario_run.py` `import fcntl`，Windows 本机不可执行） | 加"需 oracle 宿主（Linux）"前置说明 |
| R3 | 非阻塞提示 | §8 C9 期望注释 | 仅列 R-X/R-? 期望值；若采 (b) 方案需补 R-E 期望值 51/9,069+250/41,285 | 随 R1 一并补 |

### 9.4 复审命令与输出摘要

```text
# ① 逐字抽出 §8 C9 执行：
R-E 311 段 / 52416 B | L2-diff 10段/2062B | core-cited 44段/8465B | spec-cited 257段/41889B
R-D 2 段 / 379 B | 0 | 0 | 2段/379B
R-G 17 段 / 4778 B | 1段/578B | 1段/172B | 15段/4028B
R-A 6 段 / 1564 B | 0 | 1段/115B | 5段/1449B
R-X 23 段 / 70760 B | L2 3段/4189B | core 4段/20493B | spec 16段/46078B
R-? 35 段 / 1175 B | L2 1段/36B | core 4段/82B | spec 30段/1057B   ← D1 行与 §2.2 一致
# ② bank 引用窗口扫描（正则 bank\d{1,2}:\$?[0-9A-F]{4}(-…)?）：
total 205 / unique 172 / 窗口越界 1（唯一 = §10 L574 "bank15:$D422 → bank14:$D422" 的旧值留痕，非实质引用）
# ③ §5/§6 表格复算：rows 15/54；set equal True；refs ok 51 fails 0；bytes 6,865 / 29,327
# ④ 当前 HEAD 16fb5b2 门禁实跑：
SUMMARY: PASS=11 FAIL=0 SKIP=1 TOLERATED=0 ; CHECK: PASS ; cargo 338（基线 338）；EXIT=0
[SKIP] addr-bridge（JP ROM 未入库，政策预期）；无 traceback（a11161d UTF-8 解码修复生效）
docs-links: PASS 310 md / 262 links
```

> 结论：除 R1 一处口径句/期望值外，修订版已满足上次 needs_revision 的全部修正要求且未引入结论级新错误；**R1 修复后本轮 verdict 转为 pass**（R2/R3 可随同或留待后续）。

### 9.5 R1 二次复审（2026-10-18）——VERDICT：**pass（R1 已闭环）**

> 复审对象：`docs/findings/rewrite-completion-audit.md` 二次修订版（**587 行**，585+2；sha256 `E746C7B97D58B8D3FE13953FD162C868143E720AA7699FCE23E64248EFA115C0`；未提交）。按 Lead 要求**只核 R1 三处 + E3 附带追加说明**，不做全量复审（§1-§9.4 结论继续有效）。

**逐项核（全部确认，审计方自报与实文一致）**

1. **§2.3 L125（修改点 1）**：现文"**C9 复算 §2.2 的 R-X/R-? 两行（D1 目标行，镜像与 HEAD 同值）并给出镜像口径全表；R-E 行的 HEAD 值见 §2.2（与镜像差 7 段 delta=604B，归因见 §1.3）**"——与事实一致：C9 实跑确给镜像全表；R-X/R-? 镜像=HEAD ✓；R-E 行差 7 段/604B ✓（数字见下）。
2. **§8 C9 标题 L530（修改点 2）**：现文"**镜像口径；R-X/R-? 与 HEAD 同值=D1 目标行；R-E 行 HEAD 见 §2.2——7 段 delta=604B，见 §1.3**"——与事实一致；旧过度声明"镜像与 HEAD 重导同值"/"逐格复算"全文 grep 0 残留。
3. **§8 C9 注释 L546-547（修改点 3）**：数字逐格核对——镜像 R-E `core 44 段/8,465B → HEAD 51 段/9,069B`、`spec 257 段/41,889B → 250 段/41,285B`；差值 **core +7 段/+604B、spec −7 段/−604B**；"由 spec 迁到 core"与 §1.3 七段均为 spec→core 的归因方向一致 ✓；L548"不得用待定合计反推"保留 ✓。
4. **E3 §7 L370 附带追加**：已加"`scenario_run.py` 依赖 Linux/oracle 宿主（`import fcntl`，POSIX-only），Windows 本机不可执行，需 WSL/Linux；复核可直接用 findings §3 已入库样本表" ⇒ 上次 R2 非阻塞建议亦闭环。

**独立复算证据**

```text
# C9 逐字执行（587 行版）：
R-E 311 段 / 52416 B | L2-diff 10段/2062B | core-cited 44段/8465B | spec-cited 257段/41889B
R-X 23 段 / 70760 B | L2-diff 3段/4189B | core-cited 4段/20493B | spec-cited 16段/46078B
R-? 35 段 / 1175 B | L2-diff 1段/36B | core-cited 4段/82B | spec-cited 30段/1057B
# HEAD 重导（%TEMP% 重定向）：R-E core 51段/9,069B ; spec 250段/41,285B
# 差值：core +7 段 / +604B ; spec −7 段 / −604B   （与 L546-547 一致）
# bank 引用扫描 205 处：唯一"窗口越界"仍是 §10 L574 旧值→新值留痕（非实质引用）；204/204 自洽
# §5/§6 表格复算：refs ok 51、fails 0、set equal True（0 失配）
# 行数 587 = 585+2，与自报"新增两行注释"一致（无未声明增行）
# docs-links: PASS（310 个 md / 278 条本地链接全部有效）
```

**剩余问题**：**无阻塞项**。R1 闭环；R2 已随同闭环；R3（C9 期望补 R-E 值）因采 (a) 案、期望注释只针对 D1 两行，已由标题+注释覆盖，不适用。

**结论：VERDICT = pass。** 被审修订版（587 行，sha256 `E746C7B9…`）可提交；本复核报告可同提交。建议：两文档互链（审计 L8 已引用本复核；本 §9.5 引用审计 sha256），并保留"初版/复核修订/二次复审"三段时点。本复核只追加本子节，未改被审文档、未 git add/commit。
