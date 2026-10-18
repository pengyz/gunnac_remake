# 改写完成度与像素一致性权威审计（rewrite-completion-audit）

> 日期：2026-10-18｜角色：decomp-auditor（分析/审计方，只读取证，不写实现代码）
> 对象：HEAD `e7b8285`（工作区存在并发任务改动——审计时点 task-5 平台抽象层施工面为 `crates/game-platform/**`，其后续为 `crates/game-shell/` 新建与根 `Cargo.toml/lock`；另有未跟踪 `status.log`。**均不在本审计数据源内**，不影响 workset 输入 `crates/game-core/src/**` 与 docs/status 文件）。
> 纪律：AGENTS.md（证据 A/B/C 分级；已证/假设分离；数字可独立复算；未证不发明；不夸大亦不矮化）。
> 写范围：仅本文件。审计过程中**未修改/未新增工作区任何其它文件**（workset 生成器重导在系统临时目录执行，见 §2.3 与 §10）。
> 本审计**未重跑** cargo 测试/L2/L3/L4 门禁（避免与并发任务争用 `target/` 且 L4 门禁会重写 `oracle/screens/`）；凡引用此类结论均标注"仓库留痕、本审计未重跑"。已重跑且只读的门禁：`rom` / `seed` / `naming`（§8）。
> 复核：`docs/findings/adversarial-review-completion-audit.md`（decomp-verifier，verdict=**needs_revision（低风险）**）；本文件已按 D1/E2/C2·F6 等修订（见 §10 文末修订条），结论主干不变。

---

## 0. 三句话结论

1. **改写未 100% 完成**：以 `data/status/rewrite-workset.json`（提交版镜像）复算，394 段中 `L2-diff`（**workset 档位内**唯一带机器差分证据的档位；另有 L2 121/121、L3 8/8、单测 280/326 等机器证据见 §3.3/§3.5）**15 段（3.81%）/ 6,865B（5.24%）**；`≥core-cited`（含 L2，源码出现 bank:addr 引用）**69 段（17.51%）/ 36,192B（27.61%）**；按 HEAD 只读重导为 **76 段（19.29%）/ 36,796B（28.07%）**。三个"档位"都**不是**全量，且 `spec-cited` 325 段（82.49%）只是"有规格/文档引用"，不等于已改写。**引用覆盖 ≠ 行为完成度**：workset 档位是引用扫描口径（§1.1），行为面读数见 §3.3。
2. **不是"逐像素一致"**：现有实测 22 对截图中**仅 1 对逐像素完全相同**（title-menu boot-black，diff=0，占 4.55%）；全 22 对合计差异 **280,132 / 1,351,680 像素 = 20.72%**（同点 79.28%）。L4 签署文件用的是**判定式 + 差异分类账 + 显式缺口声明**（ending 五值 diff 4552/3682/3347/3981/3981 均低于各自基准 7680/6810/6475/7109/7109，"转正 5/5"），它**不是** `master-plan.md` §6 / `l4-visual-plan.md` §1 的"白名单外像素 100% 一致（0 容差）"判据。**"判定式达标/带声明签署" ≠ "像素级一致"**。且 2026-10-18 用户已裁定（`24093be`，`docs/knowledge/decision-l4-zero-tolerance-ruling.md`）：签署仅为**过程里程碑，不构成 L4 达成**。
3. **最高优先缺口**（对照 Stage A DoD）：①**L5/8 关动态可通关**未达成（`adversarial-review-72.md` §4.3 分层：链架构达成、逐关战斗内容与动态回放未达成，G-S5-6 自然通关为负结果）；②**L4 白名单外像素一致**未达成（用户 2026-10-18 裁定维持 0 容差 `24093be`；白名单 `gates/visual_whitelist.json` 至今未建立）；③内容/行为侧最高优先取证域 = **H-16 命中判定**（`bank14:$D7A3-$D7EC` 判定盒调用点 W15 未定位）+ **Boss 相位/EN-9+**（B-STG-01-BOSS 子格）。

---

## 1. 数据源、口径与镜像时效

### 1.1 workset 是什么 / 不是什么

`data/status/rewrite-workset.json` 由 `tools/workset_inventory.py` 机械生成（394 入口 = 394 段，地址序切片，字节合计 131,072 = 全 PRG，每字节恰属一段）。其 `status` 档位判据是**文件引用扫描**（工具源码 L376-430）：

| 档位 | 机械判据 | 本审计判定 |
|---|---|---|
| `L2-diff` | `oracle/subroutines/l2_cases.jsonl` 的 entry 落在段内 | **唯一"机器差分"信号**（121 例/29 子程序，三方对拍 golden==ROM 执行==game-core） |
| `core-cited` | `crates/game-core/src/*.rs` 文本中出现 `bankNN:AAAA` 且与段相交 | **引用信号**：54/54 段的"首个相交引用行"全部位于注释行（doc comment）；不能等同于"该段已实现" |
| `spec-cited` | `docs/spec/*.md` 出现相交引用 | 引用信号（任何范围引用会同时覆盖多个段） |
| `findings-cited` | `docs/findings/*.md` 出现相交引用 | 引用信号 |
| `unexplored` | 以上皆无 | — |

> **口径红线**：`unexplored=0` 的含义是"**每一段都至少被某份 spec/findings 提到过一次地址**"，**不是**"每段都考证完成"、更不是"每段都已改写"。`git log` 中 `61ed634 "工作全集进度重导（unexplored 247→0，考证覆盖 100%）"` 的"100%"应读作**引用覆盖 100%**。

### 1.2 镜像过期实证：三代数字并存

同一仓库内存在三代 workset 数字，互不一致（**镜像已过期，且旧文书更过期**）：

| 来源 | 时点（证据） | L2 | core | spec | findings | unexplored |
|---|---|---|---|---|---|---|
| `docs/spec/rewrite-workset.md` §4 进度读数 | 自称 "2026-10-15 快照"（`git log -1 -- docs/spec/rewrite-workset.md` = `fe34e43 2026-10-15 17:45`） | 15 | ≥core 49 | ≥spec 79 | ≥findings 147 | **247** |
| `data/status/rewrite-workset.json` 提交镜像 | 最后变更 `61ed634 2026-10-15 21:38` | 15 | 54 | 325 | 0 | 0 |
| HEAD 只读重导（本审计） | 2026-10-18，读 HEAD `e7b8285` 工作树 | 15 | **61** | **318** | 0 | 0 |

**判据**：镜像相对 HEAD **已过期**，差异恰为 **7 段**（`core-cited` 54→61、`spec-cited` 325→318，L2 不变）。根因：生成器把 `crates/game-core/src/*.rs` 里的 bank:addr 注释也计入引用；镜像生成后（2026-10-15 21:38）有提交新增了此类注释，但未重导镜像。

### 1.3 7 段 delta 逐条归因（可复算）

| 段 | 镜像 → HEAD | 命中引用（core 文件:行） | 引入 commit（`git log -S` 实测） |
|---|---|---|---|
| `bank00:8185-81A2` | spec-cited → core-cited | `engine.rs:619 bank00:8185-82FA` | `7eb2123 2026-10-15` 关末门判据 |
| `bank00:82EF-82FC` | spec-cited → core-cited | `engine.rs:619 bank00:8185-82FA` | 同上 |
| `bank00:8DDF-8EC7` | spec-cited → core-cited | `ui.rs:615 bank00:8DEB-8DF3` | `2acb1e1 2026-10-16` 双 A+B 站点分野注 |
| `bank00:929E-9319` | spec-cited → core-cited | `engine.rs:1259 bank00:92AA` | `63ab0d8 2026-10-16` S5-2 ② $18 门控 |
| `bank00:93C1-9409` | spec-cited → core-cited | `engine.rs:1260 bank00:9400` | 同上 |
| `bank14:CCD3-CD07` | spec-cited → core-cited | `engine.rs:967 bank14:$CCD3` | `279911b 2026-10-16` 运动更新真身 |
| `bank14:D679-D6C5` | spec-cited → core-cited | `shot.rs:355 bank14:$D67C-$D6BB` | `5407dd3 2026-10-16` W-DMG3 模型重铸 |

另：镜像 `cross_check.ida_entry_starts_total=294 / ida_export_present=true`，HEAD 重导为 `ida_entry_starts_total=0 / ida_export_present=false` —— 因为 `analysis/ida/` 是可再生工作区且当前未生成（不入库）。**该交叉核对项在干净检出上不可复现**，引用 294 时必须注明依赖 `analysis/ida/`。

### 1.4 其它状态镜像的时效

| 镜像 | 最后变更/生成 | 与 HEAD 的偏差 | 处理建议 |
|---|---|---|---|
| `data/status/coverage_map.json` | `generated: 2026-10-12`（t34） | `l2_cases: 54`，而 `oracle/subroutines/l2_cases.jsonl` 现为 **121**；`rules: 44` | 已过期，引用须带"2026-10-12 快照" |
| `data/status/completeness_matrix.json` | 最后提交 `c25b229 2026-10-13`；`faces.behavior.covered=19/59` | 与 baseline `completeness_cells: 19` 一致，**与 ratchet 同步**；但行为面分母 59 的"覆盖"是证据强度 ≥L1，不是移植完成 | 可用（须注明"证据覆盖"非"改写完成"） |
| `docs/spec/m3-inventory.md` | 标题 "2026-10-12 v1" | **严重过期**：其 §2 称 shots/enemy/stage "❌ 未动"，但 HEAD 已有 `shot.rs(460 行)/enemy.rs(1161 行)/stage.rs(265 行)` 且带测试 | **禁止**用作当前子系统状态源 |
| `oracle/screens/visual-report.md` | `ab6383c 2026-10-18 12:23` | 与 `l4-visual-plan.md`【2026-10-18 终局】五值一致（ending-1=4552 等） | 可用（L4 唯一现行报告） |

---

## 2. 机器复算：各档位段/字节/占比

### 2.1 提交镜像 vs HEAD 只读重导（同分母 394 段 / 131,072B）

**提交镜像** `data/status/rewrite-workset.json`：

| 档位 | 段数 | 段占比 | 字节 | 字节占比 |
|---|---|---|---|---|
| `L2-diff` | 15 | 3.81% | 6,865 | 5.24% |
| `core-cited` | 54 | 13.71% | 29,327 | 22.37% |
| `spec-cited` | 325 | 82.49% | 94,880 | 72.39% |
| `findings-cited` | 0 | 0.00% | 0 | 0.00% |
| `unexplored` | 0 | 0.00% | 0 | 0.00% |
| **合计** | **394** | **100%** | **131,072** | **100%** |

**HEAD 只读重导**（生成器在系统临时目录重跑，未写工作区；§2.4 命令）：

| 档位 | 段数 | 段占比 | 字节 | 字节占比 |
|---|---|---|---|---|
| `L2-diff` | 15 | 3.81% | 6,865 | 5.24% |
| `core-cited` | 61 | 15.48% | 29,931 | 22.84% |
| `spec-cited` | 318 | 80.71% | 94,276 | 71.93% |
| `findings-cited` | 0 | 0.00% | 0 | 0.00% |
| `unexplored` | 0 | 0.00% | 0 | 0.00% |

**累计口径**（"≥ 某档位"，workset 文档 §4 的读法）：

| 累计档位 | 镜像 段/字节 | 镜像 占比 | HEAD 段/字节 | HEAD 占比 |
|---|---|---|---|---|
| `≥ L2-diff` | 15 / 6,865 | 3.81% / 5.24% | 15 / 6,865 | 3.81% / 5.24% |
| `≥ core-cited` | 69 / 36,192 | 17.51% / 27.61% | **76 / 36,796** | **19.29% / 28.07%** |
| `≥ spec-cited` | 394 / 131,072 | 100% / 100% | 394 / 131,072 | 100% / 100% |

> **必须并列的两个数**：19.29%（HEAD 重导）与 17.51%（提交镜像）。若引用"改写完成度 = ≥core-cited 比例"，请同时给出镜像过期说明；本审计推荐以 **HEAD 重导值 19.29% 段 / 28.07% 字节** 为现状、以镜像值为"上一提交态"。

### 2.2 机械画像分类 × 状态（HEAD；镜像分类不变，已与 HEAD 重导逐段 diff 校验 class 差异 = 0）

| 分类（规则） | 段数 | 字节 | L2-diff | core-cited | spec-cited |
|---|---|---|---|---|---|
| 引擎 R-E | 311 | 52,416 | 10 段 / 2,062B | 51 段 / 9,069B | 250 段 / 41,285B |
| 数据 R-D | 2 | 379 | 0 | 0 | 2 段 / 379B |
| 图形 R-G | 17 | 4,778 | 1 段 / 578B | 1 段 / 172B | 15 段 / 4,028B |
| 音频 R-A | 6 | 1,564 | 0 | 1 段 / 115B | 5 段 / 1,449B |
| 待定 R-X | 23 | **70,760** | 3 段 / 4,189B | 4 段 / 20,493B | 16 段 / 46,078B |
| 待定 R-? | 35 | 1,175 | 1 段 / 36B | 4 段 / 82B | 30 段 / 1,057B |

> **矩阵口径注（D1，2026-10-18 复核修订）**：`class`/`class_rule` 逐段差=0 已验证，但 **status×rule 矩阵必须逐段过滤 `class_rule` 后累加**，不能用"待定合计"反推（早期版本即因此把 R-? 误当"全 spec-cited"、把待定合计错拆到 R-X）。行合计：R-X 4,189+20,493+46,078=70,760 ✓；R-? 36+82+1,057=1,175 ✓。`bank15:ED03-ED26`（36B）是 **R-?/L2**；R-? 的 4 个 core 段 = `bank00:86CE-86CF`(2B)、`bank00:9CE9-9CEB`(3B)、`bank03:A2FA-A342`(73B)、`bank11:8012-8015`(4B)，合计 82B。复现命令：§8 **C9**。
>
> 字节口径的**结构性陷阱**：`R-X`（投影噪声/数据项主导，分类不可当语义结论）占 **70,760B = 54.0%**。字节占比被大段数据/噪声区主导，**不能**当行为覆盖率的代理指标。且"段"是**地址序切片**，不是已证函数体（workset 文档 §5 G4）。

### 2.3 复现命令与输出摘要

见 §8 命令 C1/C2/C9。关键 stdout（HEAD 重导）：`entries=394 segments=394 segment_bytes=131072`；`status: {'L2-diff': 15, 'core-cited': 61, 'spec-cited': 318, ...}`；`class` 与镜像一致；`unmapped L2 entries: []`。**C9 复算 §2.2 的 R-X/R-? 两行（D1 目标行，镜像与 HEAD 同值）并给出镜像口径全表；R-E 行的 HEAD 值见 §2.2**（与镜像差 7 段 delta=604B，归因见 §1.3）。

---

## 3. 问题①：改写完成度（精确数字，不用"大约/基本"）

### 3.1 段维度

| 定义 | 数字 | 占比 | 证据等级 |
|---|---|---|---|
| 全量分母（入口全集，地址序切片） | **394 段 / 131,072B** | 100% | A（机械，可重导） |
| 有机器差分（L2 三方对拍落段内） | **15 段** | **3.81%** | A（L2 用例 121/121 留痕） |
| 源码有 bank:addr 引用（含 L2） | **76 段**（镜像 69） | **19.29%**（镜像 17.51%） | B（引用级；54/54 首引用行为注释） |
| 仅文档引用（spec/findings） | 318 段 | 80.71% | B（引用级，可被范围引用放大） |

> **口径限定**：本表两列性质不同——"有机器差分"是行为证据（A 级三方对拍），"源码有 bank:addr 引用"是**引用覆盖**（54/54 core 首引用行位于注释行，§1.1/§6）。**引用覆盖 ≠ 行为完成度**；citation 扫描可被范围引用放大（一条 `bankNN:AAAA-BBBB` 可同时覆盖多段），故 19.29% 只能读作"源码挂了证据指针的段占比"，不能读作"已改写 19.29%"。行为面读数见 §3.3（子系统）与 §3.5。

### 3.2 字节维度

| 定义 | 数字 | 占比 | 说明 |
|---|---|---|---|
| 全量分母 | **131,072B** | 100% | 全 PRG |
| L2 差分覆盖 | **6,865B** | **5.24%** | 与 `docs/spec/rewrite-workset.md` §3.2 "121 例全落 15 段、6,865B（5.2%）"一致（该文档 §4 为 **2026-10-15 快照**、数字已过期——仅此处 §3.2 与现行一致，勿与 §1.2 的三代数字混读） |
| 源码引用覆盖（含 L2） | **36,796B**（镜像 36,192B） | **28.07%**（镜像 27.61%） | 引用级 |
| 其中 `R-X` 噪声/数据大段 | 70,760B | 54.00% | 分类不可当语义结论 |

### 3.3 子系统维度

判据声明（本审计口径，可复算）：
- **有行为代码** = `crates/game-core/src/<模块>.rs` 存在且承载该域行为（行数见命令 C4）；
- **有机器差分** = 该域子程序在 `oracle/subroutines/l2_cases.jsonl` 有 ≥1 用例（29 子程序→6 域映射见命令 C4）；
- **无主缺口** = 对应 spec 的"遗留假设/待证残项"中无该域未闭环项（以各 spec 原文为准）。

| # | 子系统 | game-core 模块（行数） | L2 差分（子程序/用例） | 机器差分证据 | spec 未证主缺口（原文指针） | 判定 |
|---|---|---|---|---|---|---|
| 1 | 自机移动/钳制 | `player.rs`(145) | 5 / 26 | L2 121/121（留痕） | W6 速度表分划（`ram-watch-s3.md` §7）；`velocity_from_input` 对未证组合 fail-closed panic（`plan-platform-abstraction.md` **§3.3 输入策略 / L76**；该文档由 task-5 提交 `0809317`/`6362895` 入库） | 部分 |
| 2 | 武器拾取/等级 | `weapons.rs`(155) | 2 / 16 | L2 121/121 | 射击模式分派仅**查表骨架**（模块头注释）；H-11 残项 | 部分 |
| 3 | 弹丸调度/配置/运动 | `shot.rs`(460) | 6 / 15 | L2 121/121 | `shot-system.md` §5：A2/A3/A4/A5、`$925F` 副作用、type3/4/6 齐射逐型参数 | 部分 |
| 4 | 炸弹投放/消耗 | l2ports.rs+player.rs | 3 / 22 | L2 121/121 | H-11 残项（1=B/2=T 推定）；B-BMB 四格 L0 | 部分 |
| 5 | 敌对象/清除 | `enemy.rs`(1161) | **0** | 仅单元测试（`enemy_behavior.rs` 28 例、`enemy_hit.rs` 4 例） | H-16 命中本体、W10/11/14-18 | 部分（无 L2） |
| 6 | 地形脚本机 | `terrain.rs`(473) | 5 / 11 | L2 121/121 | W21′ 字节级归属（`completeness.md` §7.1 W 表） | 部分 |
| 7 | 敌波调度/记录机 | `wave.rs`(555)+`wave_record.rs`(196) | **0** | 单元测试（`wave_sched.rs` 14 例、`wave_record.rs` 10 例） | W21′/W28/W26/W27（`w28-boss.md`、`enemy-timeline.md`） | 部分（无 L2） |
| 8 | 卷轴/关卡推进 | `stage.rs`(265) | **0** | 单元测试（`stage*.rs` 27 例） | B-STG-01..08 内容格 L0/B、BOSS 子格 | 部分（无 L2） |
| 9 | 流程/死亡/GAME OVER/Continue | `flow.rs`(745) | **0** | 单元测试（`flow.rs` 7 + `full_flow` 3 + `death_chain` 10） | F6-F12 仅静态链（`game-over-continue.md` §4.2）；R-G4/R-G5/R-G6；G-S5-1..7 | 部分（无 L2） |
| 10 | 商店/金钱/UI | `ui.rs`(647) | 8 / 31 | L2 121/121 | H-09 残项（死亡/结算）、W-NEW2 双轨金钱、W-NEW5 槽基差、H-10 | 部分 |
| 11 | 音频（APU） | `lib.rs`(264，APU 面) | **0** | 仅 spec/L2 外部留痕（`audio-golden.md`） | AUD-2/AUD-3 内部体挂账（`lib.rs` 注释） | 部分（无 L2） |
| 12 | 渲染/L4 | `game-desktop::render_rules` | **0** | L4 22 对分类（非逐像素） | W-C1 调色板映射/nametable 持久化/glyph 层（`l4-visual-plan.md`、`visual-report.md`） | 部分（过程里程碑，非像素一致；L4 未达成） |
| 13 | RNG | `lib.rs::LcgRng`（占位） | **0** | 无 | 代码自述"占位…S4 替换为考证后的原版 RNG"（`lib.rs` L60-83）；真身已定位未移植（`b2-tas-input.md` §6） | **未完成（占位）** |

**子系统维度精确读数**：
- **有行为代码/占位**：13/13（其中 RNG 为显式占位）；
- **有 L2 机器差分**：**6/13 = 46.15%**（自机、武器、弹丸、炸弹、地形、商店/金钱/UI），121/121 例；
- **无 L2 机器差分**：7/13（敌对象、敌波、卷轴、流程、音频、渲染、RNG）；
- **"有代码 + 有 L2 + 无主缺口"全闭环**：**0/13**（每个域至少挂 1 条已登记的未证/残项；清单见上表"spec 未证主缺口"列）。

> **口径声明（C3，2026-10-18 复核修订）**：上表的"域↔子程序"映射是**本审计口径（非仓库既有口径）**——尤其 `bcd_sub_ammo`→炸弹、`death_money_halve`→商店/金钱两条，有 spec 支撑（R-fire-4：`$018F` 弹药经 `$ED03` 扣减；死亡结算 −金钱见 `game-over-continue.md` §3.3）但仓库没有独立的分域映射表；若换映射口径，6/13 可能变动。13/13 有代码、全闭环 0/13、段/字节读数不受映射影响。

补充机器事实（可复算，命令 C4）：`game-core` 13 个源文件合计 **6,853 行**；`game-core` 自有测试 **185 例**（`crates/game-core/tests/*.rs`）；**审计时点**全 crate `#[test]` 合计 **280**（= baseline `rust_test_count: 280`；其中 game-core 185 + 其它 crate/内联 95）。复核时点（HEAD `ff5aa7c`）已上调 **326**（+46 为 task-5 新增；基线同步上调），复核方实测 `cargo test --workspace` 326 全过（见 `adversarial-review-completion-audit.md` §3）——**核对应以 `data/status/baselines/baseline.json` 与当时 HEAD 为准**，勿把并发增量当回退/虚增。

### 3.4 与 Stage A DoD 判据逐条对照

`docs/master-plan.md` §6 判据："Stage A：8 关可通关（L5 全绿），L1–L3 全绿，L4 白名单外像素一致"。

| DoD 项 | 现状 | 证据 | 判定 |
|---|---|---|---|
| L1 文本 golden | 有登记（`m3-inventory.md` §3 判据 4 称 L1 ✅） | 过期文书（2026-10-12），未重导 | 未复核 |
| L2 全绿 | **121/121**（多份复核留痕） | `adversarial-review-74/75/77/89/93.md` 等 | 仓库留痕（本审计未重跑） |
| L3 全绿 | **TRACE-DIFF: ALL MATCH (8/8)** | `adversarial-review-75.md` L77 | 仓库留痕（本审计未重跑） |
| L4 白名单外像素一致 | **未达成**：22 对仅 1 对 diff=0；`gates/visual_whitelist.json` 不存在（0 条白名单）。**2026-10-18 用户裁定维持 0 容差（`24093be`，`docs/knowledge/decision-l4-zero-tolerance-ruling.md`）：签署仅过程里程碑、不构成 L4 达成** | `oracle/screens/visual-report.md`；§4；`master-plan.md` §6 裁定注；`l4-signing-criteria.md` 顶部【用户裁定·Lead 记录】 | **✗** |
| L5 全关卡自动通关 | **未达成** | `adversarial-review-72.md` §4.3 分层：链架构/机制字节层达成；逐关战斗内容层（B 级骨架）、动态回放层（G-S5-6 未跑）、帧锚层（golden 37/44 占位）未达成 | **✗** |
| 8 关可通关 | 同上（"能走通这条链 ≠ 能打通这 8 关"——复核 #72 原话） | 同上 | **✗** |
| 门禁 12 门 | **历史留痕（2026-10-15）**：`PASS=12 FAIL=0 SKIP=0 TOLERATED=0`（`adversarial-review-74.md` L23、`l5-all-clear.md` §6）。**现状（复核 checkout HEAD `ff5aa7c`，2026-10-18 实测）**：`PASS=11 FAIL=0 SKIP=1 TOLERATED=0`（12 门中 11 PASS；**`addr-bridge` SKIP**="坐标桥或双 ROM 缺失（条件 C）"——`reference/jp/gunnac_jp.nes` 未入库属资产政策，**clean checkout 预期 SKIP，不是回退**；见 `adversarial-review-completion-audit.md` §3） | 时点差异已标注；本审计未重跑，最终 HEAD（`a11161d`+并发施工态）需最终验收时再跑 `python gates/cli.py check` | 历史留痕 + 现状 11/1/0 |

### 3.5 结论（精确）

- **改写完成度 ≠ 100%**。按三个维度：
  - **段**：机器差分 15/394 = **3.81%**；源码引用覆盖 76/394 = **19.29%**（镜像 69/394 = 17.51%）；
  - **字节**：机器差分 6,865/131,072 = **5.24%**；源码引用覆盖 36,796/131,072 = **28.07%**（镜像 36,192/131,072 = 27.61%）；
  - **子系统**：13 个子系统"有代码/占位"13/13，机器差分覆盖 **6/13 = 46.15%**，**全闭环 0/13**。
- 另有 workset 未计入的机器证据：L2 121/121、L3 8/8、L4 22 对（判定式）——但它们同样**不构成全量**，且 L4 非逐像素（§4）。
- Stage A 合取判据中 **L4 与 L5 两项不满足** ⇒ Stage A **未达成**，不得报"改写 100% 完成"。
- L4 判据已由用户裁定锚定 **0 容差**（`24093be`）：签署口径只是过程里程碑 ⇒ **L4 判定=未达成**，不存在"按签署口径算达成"的读法。

---

## 4. 问题②：像素级一致性

### 4.1 判据原文：仓库里存在**两套**"L4 判据"（2026-10-18 用户裁定：以门禁/规划判据为准，见 §4.3）

| 标准 | 出处（原文） | 含义 |
|---|---|---|
| **门禁/规划判据** | `docs/spec/l4-visual-plan.md` §1："判定式 = **白名单外像素 100% 一致**（0 容差默认）；差异二分类 = 白名单内（平台差异，豁免）/ 白名单外（FAIL）"；`docs/master-plan.md` §6："L4 白名单外像素一致" | **逐像素相等**（唯一例外=白名单，且白名单需用户确认） |
| **签署判据** | `docs/spec/l4-signing-criteria.md` §1/§6："判定式=实测转正（diff **4552<7680**）已达成"；"终攻一轮未破对位缺口 ⇒ **带 §4 全部声明签署**；L4 判定式**按显式缺口封存**" | **阈值判定式 + 差异分类账 + 缺口声明**（非逐像素相等） |

> 【2026-10-18 用户裁定（`24093be`；`docs/knowledge/decision-l4-zero-tolerance-ruling.md`）】Stage A 的 L4 判据 = 上表**门禁/规划判据（0 容差）**；签署判据仅保留为**过程里程碑记录，不构成 L4 达成**（`l4-signing-criteria.md` 顶部已增【用户裁定·Lead 记录】覆盖终签的完成语义；`master-plan.md` §6 同条注）。据此本文 K2 的"待裁定"状态解除，L4 判定 = **未达成**。

`tools/diff_visual.py` 的退出码语义（文件头 L9）："0 = 全部对通过 **或** 全部差异有显式四分类记录（visual-report.md）；1 = 存在未分类差异"——即**门禁可因"全部差异已分类"而 PASS**（visual-report 已 22/22 分类）。这与门禁名"截图逐像素（白名单外）"及规划判据不同，是本审计要求会签裁定的**口径裂缝之一**（该裂缝已由 24093be 裁定"签署≠达成"收口，但 `diff_visual.py` 的 PASS 语义本身仍非"像素一致"）。

### 4.2 现有实测（`oracle/screens/visual-report.md`，2026-10-18 12:23，`ab6383c`）

- 期望图 **22** 张；**逐像素 diff=0 的仅 1 张**（`title-menu_f00180_boot-black.png`）→ **1/22 = 4.55%**。
- 22 对合计：总像素 22×61,440 = **1,351,680**；差异 **280,132**（**20.72%**）；同点 **1,071,548**（**79.28%**）。
- 单帧最大差异：`l4_fade-clear` 59,371（同点 3.37%）、`l4_clear-inject` 58,101（同点 5.44%）、`l4_combat-variety` 27,962。
- 低于 7680 阈值的帧：**12/22 = 54.55%**（其余 10 帧超阈，最大 59,371）。
- 签署帧 `l4_ending-1`：diff 4552 → 同点率 **56,888/61,440 = 92.59%**（与签署文件"92.6%"一致；**这是单帧数**，不是 22 帧平均）。
- 签署文件 "转正 5/5" 对应：ending 五值 **4552 / 3682 / 3347 / 3981 / 3981** 对各自基准 **7680 / 6810 / 6475 / 7109 / 7109**（全部低于基准；`l4-visual-plan.md`【2026-10-18 终局】）。
- 差异构成：`visual-report.md` 四分类显示，最大族为"**投影未覆盖区（条带外 BG 内容——nametable 持久化模型未证）**"，例如 `l4_clear-inject` 54,295/58,101、`l4_fade-clear` 55,310/59,371；另一族为 W-C1 调色板映射候选（条带内，如 `l4_combat-variety` 4,096）。

### 4.3 "判定式达标 + 差异分类账"与"逐像素相等"的差别（精确表述）

1. **覆盖范围不同**：判定式只对 **5 个 ending 帧**定义并"转正 5/5"；其余 17 帧未纳入判定式（其中 10 帧 diff>7680）。逐像素相等要求 **22/22 帧 × 61,440 像素全等**。
2. **容差不同**：判定式允许 diff 至帧像素的 12.5%（7680/61440）且按帧各自基准；逐像素相等是 **0 容差**。
3. **豁免机制不同**：判定式用"§4 声明缺口"封存；逐像素相等的唯一豁免是 `gates/visual_whitelist.json`（**当前不存在**，0 条白名单）。
4. **结论**：`l4-signing-criteria.md` 的"L4 100% 签署"与 `l4-visual-plan.md` 的"L4→100% 战役收官"只是**过程里程碑签署**（2026-10-18 用户裁定 `24093be`：**不构成 L4 达成**；见 `docs/knowledge/decision-l4-zero-tolerance-ruling.md`），**不能**表述为"与原游戏像素级一致"。若对外/对用户汇报"像素级一致"或"L4 已完成"，当前证据不支持（1/22 帧逐像素等）；**L4 判定 = 未达成**（0 容差判据未满足）。
5. **反向也不夸大缺口**：差异不是"渲染算法全错"——22 对全分类、假差异持续 0，且 12/22 帧已在 7680 内；主要缺口已被定位到 **nametable 持久化模型未证 + 调色板映射 W-C1**（`visual-report.md`；`residual-pixel-ledger.md` 10 族中 9 族维持定位、1 族 AR2-A 撤销待 W47）。

### 4.4 证据指针（现存/已失效）

| 证据 | 路径 | 状态 |
|---|---|---|
| 22 对现行差分报告 | `oracle/screens/visual-report.md` | **现行**（数值与签署终局一致） |
| 签署判据与 §4 声明 | `docs/spec/l4-signing-criteria.md` | 现行（**内部数字冲突**见 §4.5） |
| 判定式轨迹/终局 | `docs/spec/l4-visual-plan.md`（L338-382） | 现行 |
| 白名单申请（未启用） | `oracle/screens/whitelist-application.md`；`gates/visual_whitelist.json`（**不存在**） | 未启用 |
| 残余像素定位账（10 族） | `docs/findings/residual-pixel-ledger.md` | 部分失效：AR2-A 撤销 1 族（`adversarial-audit-l4.md` §0），"每像素可归因"判据不成立 |
| ending-1 定位表 | `docs/findings/l4-final-localization-ending1.md` | **时点失效**：标题为 "7753 态残差 7380px/261 格"，与现行 diff **4552** 不同时点；不得当现行残差账引用 |
| L4 深挖链对抗审计 | `docs/findings/adversarial-audit-l4.md` | 现行（AR2-A/B/C 修正仍有效） |
| L4 判据 0 容差裁定 | `docs/knowledge/decision-l4-zero-tolerance-ruling.md`；commit `24093be`；`l4-signing-criteria.md` 顶部【用户裁定·Lead 记录】；`master-plan.md` §6 裁定注 | **现行**（覆盖签署的"完成"语义；L4 判定=未达成） |

### 4.5 口径冲突（K2 已由 `24093be` 裁定收口；K1/K7 仍待会签；逐条带原文）

| # | 冲突 | 原文位置 | 影响 |
|---|---|---|---|
| K1 | **对位率三值并存**：`43.0%` / `55%` / `61%` | `l4-signing-criteria.md` §1 数字纪律注（43.0%）；§1 表（双有 4236 = 55%）；§4 item1（对位率 61%，双有 4675） | 同一文件三处不一致；`4236/7680=55.16%`、`4675/7680=60.87%` 可复算，`43.0%` 无独立复算来源；§4 item1 显系量控终局（4675→4236）后未回填 |
| K2 | **L4 判据切换 —— 已由用户裁定收口**：`l4-visual-plan.md`【2026-10-16 签署终判】**NOT READY**（"0 容差目标…远"）→【2026-10-18 终局】**转正 5/5、"L4→100% 战役收官"** | `l4-visual-plan.md` L352 vs L375-382；裁定 `24093be`（`docs/knowledge/decision-l4-zero-tolerance-ruling.md`） | **已裁定**（2026-10-18，用户）：Stage A 的 L4 判据回到 master-plan §6 / l4-visual-plan §1 的 **0 容差**；`l4-signing-criteria.md` 签署口径仅过程里程碑，**不构成 L4 达成** ⇒ **L4 未达成**（`master-plan.md` §6 与 `l4-signing-criteria.md` 顶部已原地加注）。K2 原"待裁定/若…"措辞作废 |
| K3 | **同点率分母混淆**：92.6% 是 ending-1 单帧；22 帧全量同点 79.28% | `l4-signing-criteria.md` §1；`visual-report.md` | 引用"同点率 92.6%"必须带"ending-1 单帧"限定 |
| K4 | **任务书表述与仓库现状不符**（双向纠偏）：任务书列 "H-01 弹丸行为 / 商店消费侧" 为未证域；仓库现状为 H-01/H-02 已收口（`ram-watch-s3.md` §9.1）、商店消费侧扣减/封顶/不足已 L2 覆盖 15 例（`shop-purchase-flow.md` §5） | — | 未证域清单应按 §7 更新（残项而非整域） |
| K5 | **§4 "五类"与现行条目数**：现行 `l4-signing-criteria.md` §4 有 **7 条编号**，其中 #1-#5 是声明缺口（=任务书"五类"），#6 是 lead 松散数撤销注、#7 是量控终局注 | `l4-signing-criteria.md` §4 | 引用时写成"5 类缺口 + 2 条注" |
| K6 | **workset 三世代表**（§1.2）：文档 §4（unexplored 247）/ 镜像（core 54）/ HEAD 重导（core 61） | `docs/spec/rewrite-workset.md` §4；`data/status/rewrite-workset.json` | 引用完成度数字必须带来源与提交/时点 |
| K7 | **对位率/形合率分母语义未定义**：`l4-visual-plan.md`【2026-10-17】表用 `双有/7680`（7680 恰为判定式阈值，非内容量；同表另有内容量 dg=8046） | `l4-visual-plan.md` L356-366 | 7680 作为"形合率"分母的语义**未证/口径待定** |

---

## 5. L2-diff 15 段清单（含职责与判定）

判据：`oracle/subroutines/l2_cases.jsonl` 的 entry 落在段内（121 例 / 29 子程序；`unmapped=0`）。"core 引用"列为首个相交证据行（`文件:行 引用串`）。**15 段判定统一为"有代码 + L2 差分"**（L2 用例是三方对拍，非注释级引用）。

| # | 段（bank:addr 范围） | 字节 | 类 | 职责证据（L2 子程序 / 用例数） | core 引用（首行） |
|---|---|---|---|---|---|
| 1 | `bank00:83E7-8445` | 95 | 引擎 | `terrain_load_record`(4) | `terrain.rs:37 bank00:$83F6-$8432` |
| 2 | `bank00:844A-8504` | 187 | 引擎 | `terrain_fetch_params`,`terrain_loop_load`,`terrain_loop_step`,`terrain_x_adjust`(7) | `terrain.rs:84 bank00:$8471-$8483` |
| 3 | `bank00:9D71-9E07` | 151 | 引擎 | `digit_conversion`(5) | `l2ports.rs:48 bank00:9DE8-9DF6` |
| 4 | `bank00:9EB3-9EF2` | 64 | 引擎 | `shop_spend_rapid`(3) | `l2ports.rs:147 bank00:9ECD-9ED9` |
| 5 | `bank00:9F86-9FE2` | 93 | 引擎 | `shop_ship_qty_next`,`shop_spend_ship`(7) | `l2ports.rs:158 bank00:9FAB-9FB7` |
| 6 | `bank01:A92E-AB6F` | 578 | 图形 | `ui_scroll_write`(1) | `stage.rs:5 bank1:$AB3B` |
| 7 | `bank02:863B-922A` | 3,056 | 待定(R-X) | `shop_money_earn`,`shot_alloc`,`shot_lut_accumulate`,`shot_script_fetch`(15) | `l2ports.rs:180 bank02:8FA9-8FB5` |
| 8 | `bank02:922B-92C4` | 154 | 待定(R-X) | `pickup_handler`(12) | `ui.rs:253 bank02:$922B` |
| 9 | `bank02:92C5-9697` | 979 | 待定(R-X) | `level_config_index`,`shot_config_loader`(6) | `ui.rs:255 bank02:$92C5` |
| 10 | `bank03:A039-A0FB` | 195 | 引擎 | `movement_tick`+4 clamp(26) | `l2ports.rs:7 bank03:A0D0-A0E8` |
| 11 | `bank03:A0FC-A23C` | 321 | 引擎 | `shot_schedule_step`,`shot_spawn_init`(3) | `player.rs:13 bank03:$A1FD-$A22B` |
| 12 | `bank03:A2B2-A2F9` | 72 | 引擎 | `bomb_drop`,`consumption_lookup`(16) | `l2ports.rs:67 bank03:A2C4-A2CF` |
| 13 | `bank03:A343-A4B5` | 371 | 引擎 | `death_money_halve`(6) | `flow.rs:76 bank03:$A376` |
| 14 | `bank14:DBCE-DDCE` | 513 | 引擎 | `money_add_event`(4) | `ui.rs:18 bank14:$DCCF` |
| 15 | `bank15:ED03-ED26` | 36 | 待定(R-?) | `bcd_sub_ammo`(6) | `l2ports.rs:80 bank15:ED03-ED26` |

> 段级状态仅证明"**该段含对拍面**"；29 名中只有 3 名落在段首（`pickup_handler` bank2:922B、`money_add_event` bank14:DBCE、`bcd_sub_ammo` bank15:ED03），其余 26 名为段内手选片段（`docs/spec/rewrite-workset.md` §3.2 原注）。
> **判定汇总：15/15 段 = "有代码 + L2 差分"**（无"仅规格"段；121 例三方对拍是唯一 A 级机器差分信号）。

---

## 6. core-cited 54 段清单（提交镜像口径；附 7 段 delta）

判据：`crates/game-core/src/*.rs` 出现相交 bank:addr 引用。**判定统一为"有代码引用（注释级）"**：本审计逐段取"首个相交引用行"，**54/54 行均为注释/doc-comment**（见 §8 命令 C3 输出 `首个引用行位于注释行 = 54`）。因此该档位**只能**读作"源码中挂了证据指针/骨架"，不能读作"该段行为已实现"；实现真值只有 L2 差分与 L3/L5 trace 证据。

| # | 段 | 字节 | 类 | 首个 core 引用（文件:行 引用串 + 注释摘要） |
|---|---|---|---|---|
| 1 | `bank00:8035-80D9` | 165 | 引擎 | `engine.rs:372` `bank00:806A-80D9` 关首快进环 |
| 2 | `bank00:81A3-82EE` | 332 | 引擎 | `engine.rs:619` `bank00:8185-82FA` 脚本记录 |
| 3 | `bank00:86AE-86CD` | 32 | 引擎 | `wave.rs:1` `bank00:86AE` 波调度器记录机解码 |
| 4 | `bank00:86CE-86CF` | 2 | 待定 | `wave.rs:3` `bank00:86CE` 记录格式 |
| 5 | `bank00:86D0-86FD` | 46 | 引擎 | `wave.rs:436` `bank00:86F3-86F8` `$4B` 饱和计数 |
| 6 | `bank00:86FE-8806` | 265 | 引擎 | `wave.rs:167` `bank00:8743-8763` ctrl 定型出敌 |
| 7 | `bank00:897F-8D39` | 955 | 引擎 | `enemy.rs:74` `bank00:8B89-8B95` spawn 三写出参 |
| 8 | `bank00:95F9-962B` | 51 | 引擎 | `flow.rs:61` `bank00:$95F9` 结局段 |
| 9 | `bank00:962C-9638` | 13 | 引擎 | `flow.rs:591` `bank00:962C` 滚屏循环 |
| 10 | `bank00:971E-9759` | 60 | 引擎 | `flow.rs:593` `bank00:971E` 结局卡 |
| 11 | `bank00:9BC3-9CE8` | 294 | 引擎 | `flow.rs:332` `bank00:9C5D` 炸弹弹药初始化 |
| 12 | `bank00:9CE9-9CEB` | 3 | 待定 | `ui.rs:316` `bank00:$9CE9` 清行→序列入口 |
| 13 | `bank00:9CEC-9D5D` | 114 | 引擎 | `ui.rs:176` `bank00:$9D02-$9D3E` 手柄位主环 |
| 14 | `bank00:9D5E-9D70` | 19 | 引擎 | `ui.rs:214` `bank00:$9D5E` 消息 ack 返回面板 |
| 15 | `bank00:9E08-9E1D` | 22 | 引擎 | `ui.rs:234` `bank00:$9D5E-$9E1B` 武器子面板 |
| 16 | `bank00:9E40-9E72` | 51 | 引擎 | `ui.rs:176` `bank00:$9E40` 横向移动 |
| 17 | `bank00:9E86-9EB2` | 45 | 引擎 | `ui.rs:200` `bank00:$9E94` 连射满级 |
| 18 | `bank00:9EF3-9F85` | 147 | 引擎 | `ui.rs:206` `bank00:$9F03-$9F16` 发货阶段门 |
| 19 | `bank01:A000-A06B` | 108 | 引擎 | `l2ports.rs:145` `bank01:$A05B` 不足统一出口 |
| 20 | `bank01:A0BC-A92D` | 2,162 | 引擎 | `shot.rs:371` `bank01:A4E1` `$35` 配置字段 |
| 21 | `bank02:8000-82F7` | 760 | 引擎 | `enemy.rs:405` `bank02:827D-82CD` 调用环 |
| 22 | `bank02:82F8-83DD` | 230 | 待定 | `enemy.rs:100` `bank02:82F8` 自向量指针 |
| 23 | `bank03:A23D-A2B1` | 117 | 引擎 | `weapons.rs:44` `bank03:$A271` slot=1..2 路径 |
| 24 | `bank03:A2FA-A342` | 73 | 待定 | `l2ports.rs:99` `bank03:A2B4-A2FA` 投放例程 |
| 25 | `bank03:AB52-AC41` | 240 | 引擎 | `shot.rs:244` `bank3:$ABFB` 敌弹型码 |
| 26 | `bank03:AC42-BFFF` | 5,054 | 待定(R-X) | `shot.rs:261` `bank3:$AC42-$AC63` 敌弹槽分配器 |
| 27 | `bank05:8497-9FFF` | 7,017 | 待定(R-X) | `enemy.rs:267` `bank05:8C8D-8CA9` 摆动动力学 |
| 28 | `bank08:A000-BFFF` | 8,192 | 待定(R-X) | `stage.rs:67` `bank8:$A000` 关卡目录行 |
| 29 | `bank11:8012-8015` | 4 | 待定 | `engine.rs:88` `bank11:$8012` far-call 速度重置流 |
| 30 | `bank11:8298-83CD` | 310 | 引擎 | `flow.rs:610` `bank11:$82C1` `$0300` 关末推进闸 |
| 31 | `bank14:CB4F-CB62` | 20 | 引擎 | `flow.rs:301` `bank14:$CB4F` `$041C` 状态向量高 |
| 32 | `bank14:CFEA-D048` | 95 | 引擎 | `ui.rs:79` `bank14:D009-D046` 金钱显示渲染 |
| 33 | `bank14:D10C-D216` | 267 | 引擎 | `flow.rs:392` `bank14:$D10C-D11A` F3 窗尽释放 |
| 34 | `bank14:D4FE-D514` | 23 | 引擎 | `enemy.rs:568` `bank14:$D511-$D5AE` 敌侧取框 |
| 35 | `bank14:D515-D547` | 51 | 引擎 | 同上（取框族） |
| 36 | `bank14:D548-D5E6` | 159 | 引擎 | 同上（取框族） |
| 37 | `bank14:D60E-D615` | 8 | 引擎 | `flow.rs:421` `bank14:D613-D651` 碰撞段转移应用 |
| 38 | `bank14:D616-D62E` | 25 | 引擎 | `flow.rs:305` `bank14:$D625` `$04A8` 转移旧态备份 |
| 39 | `bank14:D62F-D673` | 69 | 引擎 | `flow.rs:307` `bank14:$D64C-D651` `$0668` 命中标志 |
| 40 | `bank14:D674-D678` | 5 | 引擎 | `shot.rs:353` `bank14:$D674` 伤害公式 |
| 41 | `bank14:D78B-D7A2` | 24 | 引擎 | `shot.rs:288` `bank14:$D78B` 双阅读端 4 边比较 |
| 42 | `bank14:D7A3-D7EC` | 74 | 引擎 | `shot.rs:318` `bank14:$D78B-$D7EC` 重叠判定 |
| 43 | `bank14:D860-D93A` | 219 | 引擎 | `wave_record.rs:74` `bank14:$D86B` 状态转移表 |
| 44 | `bank14:DAE7-DB13` | 45 | 引擎 | `ui.rs:70` `bank14:DAE7` extend 阈值检查 |
| 45 | `bank15:E000-E072` | 115 | 音频 | `flow.rs:613` `bank15:$E04A` 关口号 |
| 46 | `bank15:E0D9-E16A` | 146 | 引擎 | `flow.rs:87` `bank15:$E10A` `sub_ECBF` 双金钱清零 |
| 47 | `bank15:E16B-E21B` | 177 | 引擎 | `flow.rs:340` `bank15:$E19B` F12 复位清零 |
| 48 | `bank15:E239-E2F2` | 186 | 引擎 | `flow.rs:52` `bank15:$E2BF-E2F0` GAME OVER 待决 |
| 49 | `bank15:E2F3-E3AA` | 184 | 引擎 | `engine.rs:58` `bank15:E2F3` `$27` 关末块防重入 |
| 50 | `bank15:E3AB-E424` | 122 | 引擎 | `flow.rs:55` `bank15:$E3AB-$E484` GAME OVER/CONTINUE 菜单 |
| 51 | `bank15:E425-E486` | 98 | 引擎 | 同上（菜单族） |
| 52 | `bank15:EEE5-EEFA` | 22 | 引擎 | `wave.rs:463` `bank15:EEE5-EF1E` 累加器加权进位 |
| 53 | `bank15:EEFB-EF84` | 138 | 引擎 | `shot.rs:397` `bank15:$EF40` `$4D` 等级档公式 |
| 54 | `bank15:F030-F0DB` | 172 | 图形 | `terrain.rs:285` `bank15:$F040-F0DA` nametable 持久化模型 |

**判定汇总：54/54 段 = "有代码引用（注释级）"**；本清单**不含**"仅规格"段——"仅规格"= `spec-cited` 318 段（只被 docs/spec 引用、无 game-core 引用）与 `L2-diff` 15 段（另有机器差分）互斥，故"有代码/仅规格"二分下的精确账为：**有代码引用 76 段（L2 15 + core 61，镜像口径 69 = 15 + 54）/ 仅规格 318 段（镜像 325）**。

**HEAD 重导后新增的 7 段**（镜像 54 → 61，清单与归因见 §1.3）：`bank00:8185-81A2`、`bank00:82EF-82FC`、`bank00:8DDF-8EC7`、`bank00:929E-9319`、`bank00:93C1-9409`、`bank14:CCD3-CD07`、`bank14:D679-D6C5`（判定同为"有代码引用（注释级）"）。

---

## 7. Top 未证域优先级表（现状/证据指针/可玩性阻塞/下一张任务卡）

> 排序依据：先"阻塞 Stage A DoD 的合取项"，再"可玩性"，再"行为保真审计"。每条均按 §1.1 口径区分**已证/未证**，并对任务书中的旧表述做双向纠偏（见 §4.5 K4）。

| 优先级 | 域 | 现状（已证 / 未证，截至 HEAD） | ROM 证据指针（可复现） | 现有 findings/spec | 对可玩性阻塞 | 建议下一张任务卡（角色 + 范围 + 完成判据） |
|---|---|---|---|---|---|---|
| **P0** | **8 关战斗内容层（STG-02..08）**（任务书未列，但为 DoD 直接阻塞） | 已证：8 关目录/段链/关末状态机/通关判定（A 级）；未证：逐关战斗内容可打通性、8 关动态 trace | `bank8:$A000` 目录（8B×8）；`bank8:$ADD0-$ADF7` EN 记录；`bank15:$E392 CMP #9` 通关判定 | `adversarial-review-72.md` §4.3；`l5-all-clear.md` G-S5-1..7；`stage-timeline.md` | **最高**（无逐关内容=不可通关） | **decomp-analyst + porter**：范围 = STG-02..08 各关 EN 记录/时刻层逐关展开 + L5 检查点；完成判据 = 每关 ≥1 组动态 trace 锚（非 f=0 占位）+ B-STG-xx-EN 子格按 `completeness.md` §8「防钝化三约束」准入 + `l5_s5` 全绿 |
| **P0** | **L4 像素一致性（白名单外 0 容差）** | 已证：22 对全分类/五值判定式转正/2 帧族根因定位；未证：逐像素一致（1/22）、调色板映射 W-C1、nametable 持久化模型 | `oracle/screens/visual-report.md`；`l4-visual-plan.md` L352/L375-382；`bank15:$F040-$F0DA`（NM 持久化） | `visual-report.md`、`residual-pixel-ledger.md`、`l4-visual-plan.md` W-C1/W-C2..C5 | **高**（DoD 判据之一） | **decomp-analyst + verifier**：范围 = W-C1 调色板映射 + nametable 持久化模型；完成判据 = 调色板 64 条索引→RGB A 级表 + 22 对 diff 全 0（或白名单经用户确认后逐条登记）+ `gates/visual_whitelist.json` 状态显式 |
| **P1** | **H-16 命中/受击判定（本体）** | 已证：命中清除链 `$05DC` 归零→槽释放（A 级 trace）；玩家受击样本 2 例；未证：判定盒常数、触发例程 W15、`$05DC` 双读 W18 | `bank14:$D7A3-$D7EC`（`sub_D7A3` 4 边比较+掩码）；`bank14:$D613-D673`（转移）；`bank14:$D86B`（状态表）；trace 指针 `analysis/traces/{fire-hold,respawn-window}.frames.jsonl`（**可再生工作区、不入库；当前不可直接复现**——再生成命令 `python tools/scenario_run.py run fire-hold` / `respawn-window`，样本表见 `hit-sampling-h16.md` §3/§4；注意 `scenario_run.py` 依赖 Linux/oracle 宿主（`import fcntl`，POSIX-only），Windows 本机不可执行，需 WSL/Linux；复核可直接用 findings §3 已入库样本表） | `hit-sampling-h16.md` §1-§3；`enemy-final-bridge.md` §2；`game-over-continue.md` §1.1 W15；`enemy-system.md` §4 | **高**（敌我伤害/死亡闭环） | **decomp-analyst**：范围 = W15 调用点静态反查 + `$7A9/$7BE/$77F/$794` 盒表逐字节 + 无敌窗口；完成判据 = ≥10 样本/侧 + 盒常数定谳 + `sub_D7A3` 新增 L2 用例 ≥1 + spec R-enemy-hit 更新 |
| **P1** | **Boss / EN-9+** | 已证：boss 生成链（type $CA/$CB/$CD）与 `$48C` 状态机、节拍驱动相位判据（A 级）。**B 级推断（未证）**：EN-9+ = `$ADF8+` 同格式延续（源 `w28-boss.md` §2.3 **注记**，非定谳）。未证：相位数终值（`$D0<5` 候选）、相位伤害系数、相位行为差异、逐条记录展开 | `bank7:$A6EC`（$CA 生成）、`bank7:$ACE5`（$CB）、`bank7:$AEB0`（$CD）；`bank14:$C5DF`（节拍计数器）；`bank8:$ADF8+` | `boss-chain-survey.md` §1/§4；`w28-boss.md` §2.2/§2.3；`boss-face-design.md`；`arena-midboss.md` | **高**（终关/多相） | **decomp-analyst + porter**：范围 = 相位 trace 插桩（`$48C`/`$D0`/HP 采样）+ EN-9+ 逐条；完成判据 = 相位数定谳 + 每相 HP/节拍/行为参数 A 级表 + `boss_behavior_step` 去 fail-closed + L2 用例 ≥3 |
| **P2** | **弹丸行为残项（H-01/H-02 收口后遗）** | 已证：六型生成循环/击发模式分派/等级配置/运动脚本机/碰撞链坐标（A 级 + L2 15 例）；H-01（炸弹字母/武器槽语义名）与 H-02（升级机制）已收口；未证：`shot-system.md` §5 的 A2/A3/A4/A5、`$925F` 侧副作用、type3/4/6 齐射逐型参数、LUT 精确索引规则 | `bank3:$A1FD-$A22B`（六型循环）；`bank2:$9343-$938D`（配置流）；`bank2:$87B3-$8810`（脚本机）；`bank2:$9698`（拾取/$3B）；`bank3:$A1F5`（模式分派） | `shot-behavior.md` §1-§6；`shot-system.md` §5；`shot-slot-geometry.md`；`upgrade-branch-h02.md` | 中（弹幕/升级行为精度；不阻断"能玩"，阻断行为一致） | **decomp-analyst**：范围 = A4（16 向 LUT 逐项/表界）+ A5（`$3B`→`$33` 触发链）+ 齐射逐型参数（`bank03:$A24C-$A291`）；完成判据 = LUT 16 项逐字节表入 constants + A5 调用链逐指令 A/B 级 + ≥3 条齐射 L2 用例 + `shot-system.md` 残项表回填（不发明未证参数） |
| **P2** | **商店消费侧残项**（整域已基本收口） | 已证：`$018E` 扣减 −4/−6/−10/保护 −10、封顶、不足出口、打包格式（A 级 + L2 15 例）；未证：`$E0-$E6`↔`$018E` 关系（W-NEW2）、死亡惩罚/过关结算（H-09 残）、信息页 A 键（H-10）、槽基差（W-NEW5）、4 号商品价格 | `bank00:$9DE8/$9ECD/$9FAB`；`bank01:$A05B`（不足）/`bank01:$A091`（重绘）；`bank02:$8FBC`（保护 −10） | `shop-purchase-flow.md` §1-§7；`shop-economy.md`；`shop-purchase.md` | 中（数值一致性与 4 号商品） | **spec-writer**：范围 = 双轨金钱关系 + H-09 消费侧；完成判据 = 关系证明或显式负结论（含负证据口径）+ H-09 残项关闭 + spec 同步（不新增 rename） |
| **P2** | **GAME OVER / Continue 动态锚** | 已证：F1-F5 动态+静态双证（A 级）；未证：F6-F12 仅静态链（无动态 trace）、R-G4 continue 场景、死亡-关末并发窗 | `bank15:$E2A2-E2F0`（GAME OVER 装/进）、`bank15:$E3AB-$E484`（菜单）、`sub_E0D9`（续关复位）、`bank14:$D86B` | `game-over-continue.md` §4.1/§4.2/§6.4；`l5-all-clear.md` G-S5-1..7 | 中高（死亡→结束→续关环） | **test-runner/verifier**：范围 = 单变量场景（`$18D:=0` 死亡→Start→菜单）；完成判据 = F6-F12 动态锚全落 + L5 检查点回填 + **必引 `l3-jp-cn-dualrun.md` AR57-3 更正口径**（防继承错误死亡模型，`adversarial-review-61.md` AR61 要求） |
| **P2** | **调色板映射（W-C1）** | 已证：8 色实测表（A 级，`multicolor-palette-truth.md`）；未证：完整 NES 调色板↔Rust RGB 映射、whitelist 决策 | NES PPU palette 域（Mesen2 截图实采）；`l4-visual-plan.md` §2 W-C1/C4 | `multicolor-palette-truth.md`；`l4-visual-plan.md` §2；`visual-report.md` "调色板映射未证" | 中（全局色偏，L4 阻塞） | **decomp-analyst + verifier**：范围 = PPU palette 快照×64 索引 + Rust 映射表；完成判据 = 64 条 A 级 RGB 表 + boot-black/字体帧 0 色差 + 决定"共用映射表 vs 白名单"并留用户确认记录 |
| **P3** | **RNG 真身**（代码占位） | 已证：状态机定位 `zp $2B/$2C/$2D`、`bank15:$EC12` 算术、骰子 helper **`bank14:$D422` / `bank15:$EC1F`**（A 级；`$D422∈$C000-$DFFF`=bank14 窗口，`$EC12/$EC1F∈$E000-$FFFF`=bank15 窗口）；未证：Rust 侧实现（`LcgRng` 显式占位） | `bank15:$EC12`（`$2C := ($2C+$2D+1) EOR $2B`，`$2D++`）；`bank14:$D422`（骰子）；`bank15:$EC1F`（RNG 二形） | `b2-tas-input.md` §6；`behavior-vm-handlers.md` §3.1；`arena-discriminant.md` §2.6 | 低（不影响可玩，影响行为保真/确定性） | **porter**：范围 = `GameRng` 实现替换 LCG（trait 形状不动）；完成判据 = `bank15:$EC12` L2 用例 ≥2（穷举位组合）+ 2 场景 L3 对拍 `rng_2b/2c/2d` 0 mismatch + 占位注删除 |

> 【2026-10-18 后续裁定·引用同步注（W-C1/W-C4 白名单候选撤销）】上表 **P0「L4 像素一致性」行**
> （"范围 = W-C1 调色板映射 + …"）与 **P2「调色板映射（W-C1）」行**（末句"决定'共用映射表 vs 白名单'
> 并留用户确认记录"）已被同日用户裁定更新：**W-C1/W-C4 白名单候选撤销**——11,156 个双非黑异色像素
> 两侧颜色 100% 落在 `crates/game-desktop/src/bin/render_frame.rs:19-28` 的 64 色 P 表内 ⇒ **非平台差异**，
> 残余=调色板索引绑定（ink/attr/$3F00），白名单决策项关闭（不启用、`gates/visual_whitelist.json` 仍无条目）。
> 证据：`docs/findings/l4-zero-tolerance-attack-plan.md` §0/§3.2（C7 命令 §7）、
> `docs/findings/adversarial-review-l4-attack-plan.md` §1.6 独立复现；落地标注见
> `docs/spec/l4-visual-plan.md` §2 撤销注。本注系定稿后的 dated 引用同步，**不删改**上表历史主张。

**旧任务书两处纠偏（证据在本节）**：
- "H-01 弹丸行为"：H-01（炸弹字母/武器槽语义名）已在 t25/t35 收口（`ram-watch-s3.md` §9.1、`upgrade-branch-h02.md`）；弹丸系统已成 spec+L2+Rust，真正残项见上表 **P2 弹丸行为残项** 行（`shot-system.md` §5 A2/A3/A4/A5 + `$925F` 副作用 + type3/4/6 齐射逐型参数）。
- "商店消费侧"：整域**不是未证**——扣减/封顶/不足已 A 级 + L2 15 例；未证的是 4 条残项（P2）。

---

## 8. 复现命令总清单（全部只读；C2 的生成器重导在系统临时目录）

> 环境：本机 `python3` 为损坏别名（exit 9009），**用 `python`**。工作目录 = 仓库根。
> 下列 `python - <<'PY' … PY` 为 bash/heredoc 写法；Windows PowerShell 下把 `<<'PY' … PY` 中间段落另存为 `.py` 再 `python <文件>` 即可（命令内容不变）。

**C1 提交镜像各档位段/字节/占比（§2.1 镜像表）**

```bash
python - <<'PY'
import json
d=json.load(open('data/status/rewrite-workset.json',encoding='utf-8'))
TOT=131072
for st in ['L2-diff','core-cited','spec-cited','findings-cited','unexplored']:
    ss=[s for s in d['segments'] if s['status']==st]
    b=sum(s['seg_end']-s['seg_start'] for s in ss)
    print(f'{st:14s} seg={len(ss):3d} ({len(ss)/394*100:6.2f}%) bytes={b:6d} ({b/TOT*100:6.2f}%)')
PY
# 期望：L2-diff 15 (3.81%) 6865 (5.24%)；core-cited 54 (13.71%) 29327 (22.37%)；
#       spec-cited 325 (82.49%) 94880 (72.39%)；findings/unexplored 0
```

**C2 HEAD 只读重导（不写工作区：生成器 OUT_DIR/MIRROR 重定向到系统临时目录）**

```bash
python - <<'PY'
import sys, json, tempfile, pathlib, io, contextlib
sys.path.insert(0,'tools'); import workset_inventory as wi
tmp=pathlib.Path(tempfile.mkdtemp(prefix='ws_audit_'))
wi.OUT_DIR=tmp/'workset'; wi.MIRROR=tmp/'rewrite-workset.json'
with contextlib.redirect_stdout(io.StringIO()): wi.main()          # 只写临时目录
new=json.load(open(tmp/'rewrite-workset.json',encoding='utf-8'))
old=json.load(open('data/status/rewrite-workset.json',encoding='utf-8'))
print('OLD status',json.dumps(old['counts']['status'],ensure_ascii=False))
print('NEW status',json.dumps(new['counts']['status'],ensure_ascii=False))
om={s['seg']:s for s in old['segments']}
for s in new['segments']:
    if om[s['seg']]['status']!=s['status']:
        print('DELTA',s['seg'],om[s['seg']]['status'],'->',s['status'],s['core_files'])
PY
# 期望：NEW core-cited 61 / spec-cited 318；DELTA 7 段（§1.3）；OLD 为 54/325
```

**C3 引用级核对：core-cited 首个相交引用行是否注释行（§6 结论）**

```bash
python - <<'PY'
import sys, json, pathlib
sys.path.insert(0,'tools'); import workset_inventory as wi
ROOT=pathlib.Path('.'); d=json.load(open('data/status/rewrite-workset.json',encoding='utf-8'))
core=list((ROOT/'crates/game-core/src').glob('*.rs'))
def hits(r):
    out=[]
    for f in core:
        t=f.read_text(encoding='utf-8',errors='replace'); lines=t.splitlines()
        for m in wi.CITE.finditer(t):
            a0=int(m.group(2),16); a1=int(m.group(3),16) if m.group(3) else a0
            if a1<a0: a0,a1=a1,a0
            if int(m.group(1))==r['bank'] and not (a1<r['seg_start'] or a0>=r['seg_end']):
                raw=lines[t[:m.start()].count('\n')].strip()
                out.append((f.name,t[:m.start()].count('\n')+1,m.group(0),raw.startswith('//')))
    return out
cc=[s for s in d['segments'] if s['status']=='core-cited']
print('core-cited',len(cc),'; first-line-is-comment =',sum(hits(s)[0][3] for s in cc))
PY
# 期望：core-cited 54 ; first-line-is-comment = 54
```

**C4 子系统维度：L2 用例→子系统映射 + 模块行数/测试数（§3.3）**

```bash
python - <<'PY'
import json, collections, pathlib, re
rows=[json.loads(l) for l in open('oracle/subroutines/l2_cases.jsonl',encoding='utf-8') if l.strip()]
cnt=collections.Counter(r['subroutine'] for r in rows); print('L2',len(rows),'cases /',len(cnt),'subroutines')
groups={'player':['movement_tick','player_x_clamp_hi','player_x_clamp_lo','player_y_clamp_hi','player_y_clamp_lo'],
 'weapons':['pickup_handler','level_config_index'],
 'shot':['shot_lut_accumulate','shot_alloc','shot_schedule_step','shot_script_fetch','shot_config_loader','shot_spawn_init'],
 'bomb':['bomb_drop','consumption_lookup','bcd_sub_ammo'],
 'shop/money/ui':['shop_spend_rapid','shop_ship_qty_next','shop_money_earn','shop_spend_ship','money_add_event','digit_conversion','death_money_halve','ui_scroll_write'],
 'terrain':['terrain_load_record','terrain_x_adjust','terrain_loop_step','terrain_loop_load','terrain_fetch_params']}
tot=0
for g,subs in groups.items():
    n=sum(cnt[s] for s in subs); tot+=n; print(g,len(subs),n)
print('total',tot)
print('game-core src lines',sum(len(f.read_text(encoding='utf-8').splitlines()) for f in pathlib.Path('crates/game-core/src').glob('*.rs')))
gc=sum(len(re.findall(r'#\[test\]',f.read_text(encoding='utf-8'))) for f in pathlib.Path('crates/game-core/tests').glob('*.rs'))
print('game-core #[test]',gc)
allt=sum(len(re.findall(r'#\[test\]',f.read_text(encoding='utf-8'))) for f in pathlib.Path('crates').glob('*/tests/*.rs'))
allt+=sum(len(re.findall(r'#\[test\]',f.read_text(encoding='utf-8'))) for f in pathlib.Path('crates').glob('*/src/**/*.rs'))
print('#[test] all crates (working-tree snapshot)',allt)
PY
# 期望：121/29；各域 26/16/15/22/31/11 合计 121；src 6853 行；game-core #[test] 185；
# all crates 在审计时点=280（=当时 baseline 280）；复核实测 326（基线已上调 326）——以**当时** baseline.json/HEAD 为准
```

**C5 L4 实测数字（§4.2）**

```bash
python - <<'PY'
import re
rep=open('oracle/screens/visual-report.md',encoding='utf-8').read()
pairs=re.findall(r'- ([\w\-.]+\.png): 差异像素 (\d+)',rep)
diffs=[int(n) for _,n in pairs]; tot=len(pairs)*61440
print('frames',len(pairs),'zero',sum(1 for d in diffs if d==0),'sumdiff',sum(diffs),
      'differ%.2f%%'%(sum(diffs)/tot*100),'same%.2f%%'%((tot-sum(diffs))/tot*100))
print('<7680 frames',sum(1 for d in diffs if d<7680),'/',len(diffs))
print('ending-1 same-point %.4f%%'%((61440-4552)/61440*100))
print('4675/7680=%.2f%% 4236/7680=%.2f%%'%(4675/7680*100,4236/7680*100))
PY
# 期望：frames 22 zero 1 sumdiff 280132 differ 20.72% same 79.28%；<7680 12/22；ending-1 92.5911%；
#       4675/7680=60.87% 4236/7680=55.16%
```

**C6 镜像/文档时效与 delta 归因（§1.2/§1.3）**

```bash
git log -1 --format='%h %ad %s' --date=iso -- data/status/rewrite-workset.json   # 61ed634 2026-10-15 21:38
git log -1 --format='%h %ad' --date=iso -- docs/spec/rewrite-workset.md          # fe34e43 2026-10-15 17:45
git log -1 --format='%h %ad' --date=iso -- data/status/coverage_map.json         # c729ebd 2026-10-12
git log -1 --format='%h %ad' --date=iso -- oracle/screens/visual-report.md       # ab6383c 2026-10-18
git log -S 'bank00:8185-82FA' --format='%h %ad %s' --date=short -- crates/game-core/src   # 7eb2123
git log -S 'bank14:$D67C-$D6BB' --format='%h %ad %s' --date=short -- crates/game-core/src # 5407dd3
```

**C7 本审计已重跑的只读门禁（§3.4 门禁行）**

```bash
python gates/cli.py rom      # [PASS] 4 个锚点 sha256 一致（rom/gunnac_cn.nes=4d76901c1b2455f2…）
python gates/cli.py seed     # [PASS] 1 个投影文件 sha+行数 与冻结基准一致
python gates/cli.py naming   # [PASS] 206 条记录逐条校验证据指针
```

**C8 基线证据（280 测试 = baseline）**

```bash
python - <<'PY'
import json
b=json.load(open('data/status/baselines/baseline.json',encoding='utf-8'))
print(b['metrics'])
PY
# 期望：rust_test_count 280 / completeness_cells 19 / knowledge_entries 22 / naming_ledger_entries 206
# 注：此为**审计时点**快照；复核时点已并发上调（`ff5aa7c` rust_test_count 280→326；`24093be` knowledge_entries 22→23），
#     复核实测 cargo test 326（见 adversarial-review-completion-audit.md §3）
tail -2 data/status/baselines/baseline_history.jsonl   # 最近上调理由含"cargo test --workspace 全过"
```

**C9 §2.2 `class_rule`×`status` 矩阵（镜像口径；R-X/R-? 与 HEAD 同值=D1 目标行；R-E 行 HEAD 见 §2.2——7 段 delta=604B，见 §1.3）**

```bash
python - <<'PY'
import json
d=json.load(open('data/status/rewrite-workset.json',encoding='utf-8'))
for rule in ['R-E','R-D','R-G','R-A','R-X','R-?']:
    ss=[s for s in d['segments'] if s['class_rule']==rule]
    cols=[]
    for st in ['L2-diff','core-cited','spec-cited']:
        x=[s for s in ss if s['status']==st]
        cols.append('%s %d段/%dB'%(st,len(x),sum(s['seg_end']-s['seg_start'] for s in x)))
    print(rule,len(ss),'段 /',sum(s['seg_end']-s['seg_start'] for s in ss),'B |',' | '.join(cols))
PY
# 期望：R-X 23 段 / 70,760B | L2 3/4,189 | core 4/20,493 | spec 16/46,078
#       R-? 35 段 / 1,175B | L2 1/36 | core 4/82 | spec 30/1,057
# 注：本命令读**提交镜像**；镜像与 HEAD 仅 R-E 行不同（core 44 段/8,465B → 51 段/9,069B；spec 257 段/41,889B → 250 段/41,285B；
#     即 7 段 delta=604B 由 spec 迁到 core，见 §1.3）。R-X/R-? 两行镜像与 HEAD 同值（D1 目标行）。
# 注：必须逐段按 class_rule 过滤后累加；不得用"待定合计"反推 status 列（早期版本 D1 错误即此成因）
```

---

## 9. 不确定项 / 口径待定 / 本审计未做

1. **未重跑**：`cargo test --workspace`、`python tools/diff_subroutines.py`（会编 `target/`）、`gates/cli.py trace/visual/check`（trace 会跑构建、visual 会**重写** `oracle/screens/*`）。相关结论引用仓库留痕：L2 121/121（`adversarial-review-74/75/77/89/93`）、L3 8/8（`adversarial-review-75`）、门禁历史留痕 12/12（`adversarial-review-74`、`l5-all-clear.md` §6）、280 测试（`baseline_history.jsonl`）。**复核方已独立补跑**（`adversarial-review-completion-audit.md` §3，HEAD `ff5aa7c`）：`check` **PASS=11 FAIL=0 SKIP=1 TOLERATED=0**（addr-bridge SKIP=JP ROM 未入库）、cargo **326**（基线 326）、subroutine-diff/trace-diff/visual-diff PASS（visual-diff PASS≠像素一致，见 §4.1）。本审计**仍未重跑**；最终验收 HEAD（`a11161d` 之后的干净态）需再跑 `python gates/cli.py check`。
2. **"同点率/对位率"定义不完整（K1/K7，仍为口径待定）**：`同点率=(61440−diff)/61440` 可复算；`对位率/形合率=双有/7680` 的 7680 分母语义文档未定义（7680 同时是判定式阈值；同表内容量 dg=8046），且 `43.0%` 无复算来源 → **口径待定**（K2 已由 `24093be` 裁定解除：L4 判据=0 容差、签署仅过程里程碑）。建议 spec-writer 在 `l4-signing-criteria.md` §1 给"对位率"算式与 7680 分母定义后再会签（K1/K7 仍开）。
3. **`analysis/ida/` 缺失**：镜像 `cross_check.ida_entry_starts_total=294` 在 HEAD 干净检出不可复现（`ida_export_present=false`）。不构成结论冲突，但任何引用 294 的图表须注明依赖可再生工作区。**`analysis/traces/` 同理**（§7 H-16 行已给再生成命令）。
4. **R-X 分类不可当语义结论**（workset 文档 §2.3 自述 + 74,935B 数据项区含真代码）：`bank02:863B-922A` 等段"待定"分类不代表其无行为；L2 差分另证其代码存在。
5. **任务书未列、但影响结论的并发事实**：审计时工作区有 task-5 施工改动（`crates/game-platform/**` → 后续 `crates/game-shell/`、`crates/shell-win/**`、`Cargo.toml/lock`、`docs/spec/platform-shell.md`）与未跟踪 `status.log`；workset 输入仅含 `crates/game-core/src/**` 与 docs，未受污染（复核亦验证 `game-core/src` 自 `e7b8285` 未变）。
6. **`docs/spec/rewrite-workset.md` §4 与镜像不一致**（247 vs 0 unexplored）：文档自标"2026-10-15 快照；重跑生成器即刷新"，但镜像已重导而文档 §4 未回填 → **建议**：Lead 指派（非本审计）重跑 `tools/workset_inventory.py` 并同步文档 §3.3/§4（预期 core-cited 61/spec-cited 318，模块引用计数 flow 20/ui 20/shot 10/engine 9/wave 9/enemy 7/…）。
7. **未证不发明的边界**：本审计对"完成度"的所有数字均给口径与命令；对"像素级一致"的结论只用现行 visual-report 数字（22 对）与判据原文，不做外推。
8. **本审计未验证门禁实现细节**（如 `rust-tests` 在 cp936 下的解码健壮性）：复核方发现并已由 `a11161d` 修复（子进程统一 UTF-8 解码）。该修复属门禁层，不在本审计范围；如需确认门禁全绿，以最终 HEAD 的 `python gates/cli.py check` 输出为准。

---

## 10. 自查与变更记录

- 只写本文件：`docs/findings/rewrite-completion-audit.md`；未 `git add/commit`；未改 `crates/`、`data/`、`gates/`、`rom/`、`reference/`。
- 生成器重导在 `%TEMP%` 执行；审计后 `analysis/workset/*` mtime 仍为 2026-10-18 12:24（未被本审计重写）。
- 证据分级：§1.2/§1.3/§2.1/§2.2/§4.2 为 A 级（字节级可复算）；§3.1-§3.3 的"核心差分"为 A/B 级、引用覆盖为 B 级；§7 中"已证/未证"逐条带 findings 指针，未证项不进入结论。
- 对抗复核提示（初版给出；复核已完成，结论见文末修订条）：(a) C2 重导的 7 段 delta 是否复现；(b) §4.2 的 22 帧合计 280,132 是否复算一致；(c) §3.3 "6/13 子系统有 L2" 的映射是否可接受（口径已声明）；(d) §4.5 K1/K7 的口径冲突是否成立（K2 已由 `24093be` 裁定收口）。
- 无假设区内容混入结论（本审计为元层审计，不新增游戏语义主张）。

【2026-10-18 复核修订】依 `docs/findings/adversarial-review-completion-audit.md`（decomp-verifier，**verdict=needs_revision（低风险）**）与用户裁定 commit `24093be`（`docs/knowledge/decision-l4-zero-tolerance-ruling.md`）修订 10 处，**结论主干不变**（改写未 100% / L4 未达成 / Stage A 未达成）：
- **D1**（§2.2）：R-X / R-? 两行 6 个 status 交叉格不可复算 → 按本审计独立重算值替换（R-X L2 3/4,189B、core 4/20,493B、spec 16/46,078B；R-? L2 1/36B、core 4/82B、spec 30/1,057B），加"必须逐段按 class_rule 过滤、不得用待定合计反推"注与 **C9** 复现命令。
- **D2**（§7 P3）：`bank15:$D422` → **`bank14:$D422`**（`$D422∈$C000-$DFFF`=bank14 窗口；seed = `bank14:D422 29 1F`、workset = `bank14:D422-D4FD` 段），`$EC1F` 明确为 `bank15:$EC1F`。
- **E1**（§3.3 行1）：断链指针补为 `plan-platform-abstraction.md §3.3 输入策略 / L76`，并注明该文档由 task-5 提交 `0809317`/`6362895` 入库。
- **E2**（§3.4 门禁行）：区分**历史留痕**（2026-10-15 PASS=12/SKIP=0）与**复核现状**（HEAD `ff5aa7c` 实测 PASS=11/FAIL=0/**SKIP=1**，addr-bridge SKIP=JP ROM 未入库、政策预期）。
- **E4**（§7 P1）：EN-9+ = `$ADF8+` 同格式延续由"已证"降为 **B 级推断（未证）**，注明源 `w28-boss.md` §2.3 为注记。
- **F6/C2**（§4.3 item4 与 §4.5 K2）："项目治理意义上的签署完成" → **"过程里程碑签署（不构成 L4 达成，2026-10-18 用户裁定 `24093be`）"**；K2 由"若…则 L4 未达成"改为**已裁定**（L4 未达成）。
- **C1**（§0/§3.1）：补"引用覆盖 ≠ 行为完成度；workset 档位内唯一机器差分信号 = L2-diff"限定。
- **C3**（§3.3）：6/13 域↔子程序映射标注为"**本审计口径**（非仓库既有口径）"。
- **C4**（§3.2）：`rewrite-workset.md` §3.2 引用补"该文档 §4 为 2026-10-15 快照、数字已过期"限定。
- **E3**（§7 P1）：`analysis/traces/*` 改标"可再生/不入库、当前不可直接复现"，补再生成命令。
另同步：`24093be` 后 `l4-signing-criteria.md` 顶部新增【用户裁定·Lead 记录】、`master-plan.md` §6 增裁定注（§4.1/§3.4 已引用）；基线并发上调（`ff5aa7c` rust_test_count 280→326、`24093be` knowledge 22→23）在 §3.3/C8 已标注。**复核方仅需复审 D1/E2/C2·F6 三项**。

【2026-10-18 初版】decomp-auditor，HEAD `e7b8285`；【2026-10-18 复核修订】decomp-auditor，复核 HEAD `1be66e5`/裁定 `24093be`。
