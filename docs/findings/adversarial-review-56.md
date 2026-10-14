# 对抗复核 #56：欠账批批量复核（L2/L3 双跑+坐标桥 · addr-bridge 门禁 · 路径归一化 · 历史清账 · KB 17→18 执策）——fresh context·证伪立场

> 被审：①L2/L3 双跑与坐标桥（670a741/69d90e4/0d113d9）②addr-bridge 门禁（bbb5831）③路径归一化（022285c）
> ④历史产出清账（fc83cd3，39 报告+4 轮次日志）⑤knowledge 条目 `gotcha-workspace-drift-images-freeze-before-diff.md`
> （t202 产物，17→18 基线执策前置）。
> 审核方：独立 verifier #56（fresh context subagent，实施≠审核，只读独立重导，不采信被审任何自述数字）。
> 纪律：AGENTS.md §5 对抗复核（2026-10-12）、§3 证据分级、verifier 角色契约（禁改阈值/白名单/用例）。
> 数据源（只读）：`rom/gunnac_cn.nes`、`reference/jp/gunnac_jp.nes`、`oracle/subroutines/l2_cases{,_jp}.jsonl`、
> `analysis{,_jp}/traces/*.frames.jsonl`、`data/reference/addr-bridge-jp-cn.json`、`gates/cli.py`、`docs/`。
> 过程注：复核期间 captain 并行提交（85c2ca6/426a1a8）；本审全部数字在收尾 HEAD 重导，本审未改动任何被审文件
> （门禁攻击测试对桥文件做临时篡改后逐字节还原，sha 前后一致，见 §2.3）。

**verdict：needs_revision** —— 被审核心技术数字**全部独立复现成立**（L2 94/94、L3 六切换点全同、桥 2592 条
全量自洽、verify 20/20、109/109）；但 **addr-bridge 门禁攻击面失守**（手工改桥 4/5 场景无痕通过，
"meta sha 锚定"只锚双 ROM 不锚桥自身）、**`grep -r` 路径判据不成立**（.archive 历史快照残留 1607 行
绝对路径，022285c commit 声称"全部 jsonl/json 零残留"在递归口径下过强）、**round-024 轮次统计
与库内报告对不上**（10/9 vs 实测 16/4）、**坐标桥覆盖不到 10 条已知搬家例程**（表述与实际不符）。
另有 jp 用例集滞后、L3 报告定性待修（D3 C1-C5 未执行，按预期如实记）、报告编号错位等 7 项。

## 0. 验收判据对照（本审自身）

| 判据 | 结果 | 证据 |
|---|---|---|
| ① 5 项各独立重导/裁定 | ✓ 5/5 | §1–§5（每项含独立重导命令与实测数字） |
| ② knowledge 执策裁定明确 | ✓ **可执基线（17→18）** | §5（P3 引用指针小修不阻塞） |
| ③ verdict 明确 | ✓ **needs_revision** | §7 |
| ④ gates 全绿 | ✓ 12/12 PASS + selftest 37/37 | §6（收尾复跑贴出） |

## 0.5 发现总表（四分类，高危优先）

| ID | 分类 | 级 | 问题 | 处理 |
|---|---|---|---|---|
| AR56-1 | 规格缺口（门禁强度） | **高** | addr-bridge 门禁**拦不住手工改桥**：实测 5 攻击 4 通过——改被抽样条目 `delta`、改未抽样条目 `jp`、删 unmapped 条目、改 same-site 区间均 PASS，仅"改被抽样条目 jp"被拦。meta sha 只锚双 ROM 不锚桥文件；抽样固定 step=6 仅 8/54；`delta`/`bytes`/条目数/结构全无校验；`data/reference/` 不在 commit-msg 敏感前缀（data/naming/、data/status/、reference/、rom/）——"禁手工换算"实际不可检 | 回 gates（captain 单独 commit）：①桥 sha 入 SHA256SUMS 锚点或 meta 自哈希 ②relocated 54 条全量复验（成本可忽略）③校验 delta=jp−cn、bytes=len ④敏感前缀增补 data/reference/ |
| AR56-2 | 规格缺口（产物清账） | 中 | 任务判据 `grep -r "/home/peng" analysis/traces analysis_jp/traces` 零命中**不成立**：实测 **1607 行 / 31 文件**残留（analysis/traces/.archive 30 文件 + analysis_jp/traces/.archive 1 文件，全为 ROOT 前缀）；live 产物 0 命中 ✓（修复本身有效）。022285c commit 声称"analysis+analysis_jp **全部** jsonl/json 零残留"过强——其 Evidence grep 用非递归 glob `analysis*/traces/*.json*` 规避了 .archive | 二选一：清洗 .archive 历史快照后重贴递归 grep；或改述口径"live 产物零残留（.archive 修复前快照除外）"并注明快照生成时间 |
| AR56-3 | 规格缺口（统计失实） | 中 | round-024 §②"复审 #23–#41 共 19 报告：**10 pass / 9 needs_revision**"与库内报告不符：逐份实测（adversarial-review-29..48 = 20 份 = 19 内部号 + #37b）= **16 pass / 4 needs_revision**（needs_revision = file 32/37/43/47 = 内部 #26/#31/#37/#40）；限定 t212–t256 窗口（file 32..48，17 份）= 13 pass / 4 needs_revision。两口径都对不上 10/9 | 回 captain 裁定：或更正 round-024 统计口径，或补交"9 份 needs_revision"中未入库的报告（若真存在=错漏） |
| AR56-4 | 规格缺口（覆盖与表述） | 中 | 坐标桥对 **10 条 L2 已证重定位例程全部无映射**：digit×5（bank00:1D45-1E01）、sc-1（bank01:0AC3-0D11）、mn×4（bank14:1BA7-1C27）所在 diff 块全被判 `unmapped/content-not-found`（块含操作数重写→整块唯一命中失败），桥内 54 条 relocated 无一覆盖它们。jp-cn-parity §5"10 条重定位（上表=**坐标桥种子**）"、§6"跨坐标换算**一律走此表**"表述与实际不符（表答不出这 10 条，L2 case 文件才是其唯一真值） | §5/§6 补注"10 条搬家点未入桥（块级匹配失败），换算须查 l2_cases_jp.jsonl"；或 gen 增加 sub-block 匹配 |
| AR56-5 | 规格缺口（用例同步） | 中 | jp 用例集滞后 A1：`l2_cases.jsonl` **109** 条（CN，109/109 复跑通过）vs `l2_cases_jp.jsonl` **94** 条——缺 15 条 = sp-*/sq-*/se-*（商店扣费/数量/收入族）。JP 侧 A1 商店经济行为未获字节级双跑覆盖；各报告"L2 94/94"口径需注明 jp 集滞后（数字本身如实） | 补 15 条 jp 用例（入口地址经桥/L2 定位）后重跑；在 jp-cn-parity §5 注明"jp 集=94（A1 15 条未同步）" |
| AR56-6 | 规格缺口（定性待修） | 中 | L3 报告现存表述与 D3 归因翻案不一致：`l3-jp-cn-dualrun.md` §2 类①"**内容驱动时序差**（汉化菜单/文本页流程更长）"、§3"内容驱动时序段"、§4"全部内容层"仍在原文，**无 dated 更正注**；D3（base-attribution.md §0/§4）已判"(a) 区域/构建差异为主，'翻译内容更长'**已证伪**"并列 C1–C5 更正清单（设计=只列不改交 captain 执行）。**本项属预期待修状态，如实记录** | 执行 C1–C5 dated 更正（就地加注不删历史）后可关闭；本审不代执行 |
| AR56-7 | 规格缺口（可追溯性） | 中 | 抽 3 份验"内容与文件名匹配"：**内容齐备 ✓ 但编号错位**——file N ↔ 内部"复审 #N-6"（#10→#4、#30→#24、#48→#41），且 file 43/44 同挂内部 #37/#37b；跨文档引用一律用内部号（"复审 #2-R3""复审 #23–#41"），按文件名检索必错位。10..48 连续 39 份无缺号 ✓，4 份轮次日志（round-021..024）在库 ✓，M 类 dated 更正格式合规（R18/R20/R9 系，删除线+日期锚）✓ | 各报告文首补一行"（文件序号 N = 复审轮次 #M）"映射注，或建索引表；后续报告统一文件名=内部号 |
| AR56-8 | 规格缺口（表述过强） | 低 | jp-cn-parity §5"84/94 **同址同码**同行为"中"同码"过强：84 条同址用例中 **45 条（全在 bank03）**入口 64B 内存在 2B 操作数重写（成对偏移 23/24、46/47…= 指针层重定向指纹）；仅 56 条 entry16 字节全同。行为等价由执行级 94/94 支撑（不受影响），但"同码"应改"同址同构（操作数重写）" | 就地改述或补"（码=指令流同构，操作数按指针层重写）"括注 |
| AR56-9 | 规格缺口（假设标注） | 低 | ①桥 same-site **2377/2592（91.7%）= MIN_BLOCK<24 启发式**（delta 恒 0），是假设非逐条证据——短段若属搬家块内部会错标恒等；②jp-cn-parity §6 族枚举只列 −0x8C/−0x354/−0x2C6/−0x293（覆盖 12/54），漏 **bank01 −0x2A6 族 ×21（最大族）**、bank00 −0x16F ×1、bank15 **+10 ×15** | §6 补全 7 族枚举；桥 meta/文档标注 same-site=启发式假设 |
| AR56-10 | 规格缺口（不可复现子项） | 低 | L3 报告"shop **4 键**中 1 键全段同"未具名 4 键；当前 shop 场景 watch 集=0320+hex 段（zp/iram/wram），scalar 键仅 0320（155/1450 帧差）——该子项对现存 trace **不可复现**（同报告其余子项全复现，见 §1.2） | 具名 4 键（地址+口径）或删除该子项 |
| AR56-11 | 规格缺口（工具口径） | 低 | `gen_addr_bridge.py verify`（抽 20，step=2）与 gate_addr_bridge（抽 8，step=6）抽样集合不同、均不校验 `delta`/`bytes` 字段——"verify 20/20"不构成 gate 覆盖证据；verify 断言双 ROM sha ✓（好）但无桥内容锚 | 与 AR56-1 合并修：全量复验 + 字段自洽校验后抽样可废 |
| AR56-12 | 规格缺口（覆盖面） | 低 | `normalize_paths_in_outputs` 仅覆盖 `{name}.captures.jsonl`/`{name}.run.json` 两文件、仅替换 `ROOT+"/"` 前缀（ROOT 之外的 `/home/...` 不会清）；当前全 1607 行残留均 ROOT 前缀（无越界路径），风险未兑现 | 随 AR56-2 口径一并注记即可 |
| AR56-13 | 规格缺口（引用纪律） | 低 | KB 条目证据指针写"t189 O1"（任务+观察号），未按 README §2.4 的可复现物形式给**文件指针**；实际证据在 `docs/findings/adversarial-review-24.md` §0 O1（95.32%→98.07% 逐字在案，本审已核）。另"历次 L4 数字微差以此为**统一解释候选**"含候选性表述（条目已自标"候选"，可接受） | 补文件指针"（实录：adversarial-review-24.md §0 O1）"；不阻塞 17→18 基线 |

被审**维持成立**部分（本审独立复现，勿动）：L2 94/94 与 109/109 执行等价；L3 六切换点/209f 周期/7f 窗/707f↔173f/+24f 漂移；
桥 2592 条全量自洽（entries≡diff runs 16/16 bank、bytes/delta 字段 0 错、relocated 字节全等 54/54）；unmapped "不猜"判定；
meta sha 双 ROM 锚定（换 ROM 可检出）；.archive 归档隔离机制本身；KB 条目结构/家族互链/README 对齐。

---

## 1. L2/L3 双跑（670a741/69d90e4）——核心数字全复现 ✓

### 1.1 L2 重跑（不采信 94/94 自述）

```bash
cargo run --release -p game-oracle --bin l2-diff -- --rom reference/jp/gunnac_jp.nes \
    --cases oracle/subroutines/l2_cases_jp.jsonl     # → L2-DIFF: 94/94 通过（三者缺一不可）
cargo run --release -p game-oracle --bin l2-diff -- --rom rom/gunnac_cn.nes \
    --cases oracle/subroutines/l2_cases.jsonl        # → L2-DIFF: 109/109 通过
```

| 项 | 声称 | 独立重导 | 判 |
|---|---|---|---|
| JP 集通过率 | 94/94 | **94/94**（JP ROM 字节执行三方对拍） | ✓ |
| 用例集同步 | （未提） | CN **109** vs JP **94**，缺 15 = sp-r1-r3/sp-s1-s2/sq-1..5/se-1..5（A1 商店经济族） | **✗ AR56-5** |
| 10 条重定位适配 | 搬址但行为全同 | 10 条 entry/stops 与 CN 集逐字段 diff（其余 84 条逐字节相同）：digit×5（9DE8→9C60）、sc-1（AB3B→A7CF）、mn×4（DBCE→D93B） | ✓ |
| Δ 值 | −0x188/−0x36C/−0x293 | 独立算 **−392/−876/−659** = 同值 | ✓ |
| "84/94 同址同码" | 同码 | entry16 字节全同仅 **56/84**；45 条 64B 窗内有 2B 操作数重写（全 bank03） | ✗ AR56-8 |

**抽 2 条重定位用例入口字节（ROM 直读，A 级）**：

| 用例 | CN 入口 16B | JP 入口 16B | 差 |
|---|---|---|---|
| digit-1 | `ad 8e 01 c9 0a b0 03 4c 5b a0 e9 0a 8d 8e 01 20`（bank00:9DE8） | `ad 8e 01 c9 0a b0 03 4c d3 9e e9 0a 8d 8e 01 20`（bank00:9C60） | 仅 `JMP` 目标操作数（$A05B→$9ED3，跳转目标自身亦搬家） |
| mn-1 | `85 04 0a 65 04 0a f0 29 a8 86 77 a2 05 18 b9 cf dc`（bank14:DBCE） | `85 04 0a 65 04 0a f0 29 a8 86 77 a2 05 18 b9 3c da`（bank14:D93B） | 仅 2B 表址操作数（`B9 CF DC`→`B9 3C DA`）——与 jp-cn-parity §1"仅差 2B=表地址操作数"**逐字吻合** |

（另核 sc-1 ui_scroll_write：bank1:AB3B↔bank1:A7CF **entry16 字节全同**=纯搬家，与"16B 精确全同"吻合。）

### 1.2 L3 抽 1 场景重导（gameplay-stage1，逐帧 watch 向量）

独立扫描 `game_action_mode` 双侧全部转移帧（不看报告）：

| 转移 | CN | JP | 判 |
|---|---|---|---|
| 入 124 / 124→110 | f203 / f206 | f198 / f200 | 类①早期分叉 |
| **mode 110 窗** | **f206–913 = 707f** | **f200–373 = 173f** | ✓ 与报告/D3 两数逐位吻合 |
| **六切换点（死亡/重生节拍器）** | **1251→1460→1467→1676→1683→1892** | **完全相同 6 帧** | ✓ **"六切换点帧级全同"成立** |
| 周期/窗 | 1460−1251=1676−1467=1892−1683=**209f ×3**；1467−1460=1683−1676=**7f ×2** | 同 | ✓ |
| f1892 后下一转移 | f1923 | f1899 | ✓ **+24f** 与报告类③"整体 +24f 偏移"吻合 |
| hit-test `0320` | 差异 **76/2350** 帧 | — | ✓ 与"76 帧差/2350"吻合 |

→ 报告 §1"六切换点全同 + 209f×3 + 7f×2"、§2 类① 707f/173f、类③ +24f、hit-test 76/2350 **全部复现**。
未复现子项仅"shop 4 键中 1 键全段同"（AR56-10，未具名键）。

**与 D3 更正清单一致性（AR56-6，预期待修）**：l3-jp-cn-dualrun.md 现存"内容驱动时序差/内容层"三处
（§2 类①、§3、§4）与 base-attribution.md §4 的 C1–C5 相抵，全文**无 dated 更正注**——D3 设计即"只列不改
交 captain 执行"，故现状=待修，如实记录，不判被审失实；但**在其执行前，"内容驱动"定性不得再被引用**
（D3 已 A 级证伪）。

## 2. 坐标桥（0d113d9）与 addr-bridge 门禁（bbb5831）

### 2.1 桥全量审计（比 verify 抽样更强的独立核）✓

| 项 | 声称 | 独立重导 | 判 |
|---|---|---|---|
| 条目总数 | 2592 | **2592**（same-site 2377 + relocated 54 + unmapped 161） | ✓ |
| 覆盖完备性 | —（生成物） | **entries ≡ diff runs 逐段相等（16/16 bank，2592 段）**——无漏段/无多段 | ✓ |
| 字段自洽 | — | `bytes` 字段 0 错、`delta`=jp−cn 54/54 自洽、区间合法 0 错 | ✓ |
| relocated 字节 | verify 抽 20/20 | **全量 54/54 CN 段≡JP 段逐字节**（非抽样） | ✓ |
| unmapped 判定 | 不猜 | 抽 2 条（bank00:0EDC-0EF5 25B、bank15:124E-12C2 116B）复判：全 blob 及两半块在 JP **0 命中** ⇒ content-not-found 属实 | ✓ |
| verify 命令 | 20/20 | `python3 tools/gen_addr_bridge.py verify` → **verify: 20/20 通过** | ✓ |
| 族枚举 | 4 族 | 实测 7 族：bank01 **−678×21**/−852×6、bank14 −710×7/−659×1、bank00 −367×1/−140×3、bank15 **+10×15** | ✗ AR56-9 |
| 对 10 条搬家例程的映射 | "坐标桥种子" | **0/10 有 relocated 映射**（全落 unmapped 块） | ✗ AR56-4 |

### 2.2 门禁常规面 ✓

`python3 gates/cli.py check` → **12 PASS / 0 FAIL / 0 SKIP / 0 TOLERATED**（addr-bridge 项 PASS）；
`python3 gates/cli.py selftest` → **37/37 通过**（含 SHA256SUMS 篡改可检出、SKIP 显式打印、established 后消失守卫）。

### 2.3 攻击面（任务点名项）——**失守，5 攻击 4 通过** ✗ AR56-1

对 `data/reference/addr-bridge-jp-cn.json` 逐项篡改→跑 `gate_addr_bridge()`→还原（还原后 sha
`e0217ee4db89…` 与前一致，gate 复绿）：

| # | 篡改 | 期望 | 实测 | 判 |
|---|---|---|---|---|
| A1 | **被抽样条目** `delta` −140→−999 | 拦 | **PASS** | ✗ delta 字段零校验 |
| A2 | **未抽样条目** jp 起点 +1 | 拦 | **PASS** | ✗ 抽样 8/54（step=6 固定）外可无痕改 |
| A3 | 被抽样条目 jp 起点 +1（正对照） | 拦 | **FAIL**（"字节不符（映射失效）"） | ✓ 抽样在其覆盖处有效 |
| A4 | 删 1 条 unmapped 审计记录 | 拦 | **PASS** | ✗ unmapped/same-site 无守护 |
| A5 | 改 same-site 条目 cn 区间 | 拦 | **PASS** | ✗ 2377 条恒等映射可无痕错标 |

**锚定强度定性**："meta sha 双锚定"锚的是**双 ROM**（换 ROM/桥过期可检出 ✓），**桥文件自身无任何内容锚**
（不在 SHA256SUMS、不在 commit-msg 敏感前缀 `data/naming/、data/status/、reference/、rom/`——
`data/reference/` 不匹配任何前缀）⇒ 手工改桥=机器近无痕，与"生成物勿手改；禁手工换算"的设计目标相悖。
修复建议见总表 AR56-1（全量 54 条复验的开销可忽略，抽样本无必要）。

## 3. 路径归一化（022285c）——live 干净 ✓，递归判据不成立 ✗ AR56-2

```bash
grep -rn "/home/peng" analysis/traces analysis_jp/traces          # → 1607 行 / 31 文件
grep -rn "/home/peng" analysis/traces analysis_jp/traces | grep -v "/.archive/"   # → 0 行
```

| 项 | 实测 | 判 |
|---|---|---|
| live 产物（captures.jsonl/run.json 等） | **0 命中** | ✓ 修复有效 |
| `.archive/` 历史快照 | **1607 行 / 31 文件**（analysis 30 + analysis_jp 1），全为 `/home/peng/workspace/gunnac_remake/{analysis,shots}…` ROOT 前缀 | ✗ 任务判据"grep -r 零命中"不成立 |
| commit 表述 | "存量 19 文件已清洗（analysis+analysis_jp **全部** jsonl/json 零残留）"；Evidence 用非递归 glob `analysis*/traces/*.json*` | ✗ 表述过强（递归口径下不实） |
| 实现覆盖面 | 仅 `{name}.captures.jsonl`/`{name}.run.json` 两文件、仅剥 `ROOT+"/"` | △ AR56-12（当前无兑现风险） |

归档隔离机制本身（`.archive` 时间戳目录）是正确的 run 隔离设计；残留属修复口径遗漏而非机制缺陷。

## 4. 历史产出清账（fc83cd3）——入库无缺漏 ✓，编号与统计两处失实 ✗

| 项 | 声称 | 独立重导 | 判 |
|---|---|---|---|
| 39 份复核报告 | #10–#48 | **39 份连续无缺号**（01–49 齐、另 52/53 在库） | ✓ |
| 4 份轮次日志 | round-021..024 | 全在库 | ✓ |
| 抽 3 份内容匹配 | 各文件自带 verdict/findings | #10/#30/#48 均含裁定行+分节证据+复现命令（结构合规）；但**编号错位** file N=内部 #N-6，#43/#44 同挂 #37/#37b | ✗ AR56-7 |
| M 类改动 | 合法 dated 更正 | cjk-quadrant-rule（R18）、sprite-shape-rowshift（R20）、font-display-bridge（R9）均删除线+`【2026-10-13 更正】`锚，不删历史——§3.6 合规 ✓ | ✓ |
| 轮次统计 | round-024："#23–#41 共 19 报告：10 pass / 9 needs_revision" | 逐份实测 **16 pass / 4 needs_revision**（19 内部号口径）/ 13 pass / 4（t212–256 窗口口径） | ✗ AR56-3 |

## 5. knowledge 17→18 执策验收（专项）——**可执基线** ✓

对象：`docs/knowledge/gotcha-workspace-drift-images-freeze-before-diff.md`（t202 产物）。

| 维度 | 核查 | 判 |
|---|---|---|
| frontmatter | `type: gotcha`/title/date/status: confirmed，文件名前缀=type | ✓ |
| 正文结构 | 现象/为什么/何时使用/证据指针 四节齐（README §2.3） | ✓ |
| 内容正确性 | 核心教训（L4 比对禁读工作区漂移图→冻结 A/B 或新鲜即测）成立；"95.32%→98.07%"活例在 `adversarial-review-24.md` §0 O1 逐字在案（"首轮曾测到旧图 95.32%，冻结副本 A/B 后方为准 98.07%"） | ✓ |
| 引用有效性 | 三家族互链文件均存在（docs-links 门禁 102 链接全绿交叉印证）；`oracle/screens/actual/` 仍在 | ✓ |
| 家族编号一致性 | "第 4 员"与 gotcha-concurrent-mesen-output-corruption 的【t202 家族扩员】注、git-add-commit"第三员"自洽 | ✓ |
| README 对齐 | §4 表 **18 行**，新行 type/摘要与文件一致；knowledge-format 门禁 18 条 PASS、ratchet 18/18 | ✓ |
| 证据指针形式 | "t189 O1"非文件指针（README §2.4 列举轮次日志§/commit/命令输出/门禁输出） | △ AR56-13（P3） |

**裁定：knowledge_entries 17→18 可执基线** —— 条目为已确认教训、证据真实可溯、格式/清单/家族互链全合规；
AR56-13（补文件指针）为 P3 文档小修，不阻塞基线执行（依 decision-baseline-raise-policy：进度类上调，
captain 验收后代执行；下调/放宽仍须用户确认）。

## 6. 门禁收尾（贴实际输出）

```
python3 gates/cli.py check
  [PASS] ×12（rom-integrity / seed-integrity / assets-manifest / naming-ledger(206 条) /
          knowledge-format(18 条) / docs-links(196 md / 102 链) / rust-tests(108) /
          subroutine-diff / trace-diff / visual-diff / completeness-ratchet / addr-bridge）
  ratchet: 7/7 OK；baseline-integrity OK；TOLERATED（无）
  SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0
  CHECK: PASS（含 0 个静默跳过：所有 SKIP 均已显式打印）
python3 gates/cli.py selftest   → SELFTEST: 37/37 通过
```

## 7. verdict 与路由

**verdict：needs_revision**（欠账批核心交付真实、数字全复现，但 4 处需修后方可称"步骤完成"）：

1. **AR56-1（高）门禁强度** → gates 修（captain 单独 commit，禁与功能混提）：桥内容锚 + relocated 全量复验 +
   delta/bytes 校验 + `data/reference/` 入敏感前缀；修后须重跑本表 A1–A5 攻击复验（期望 5/5 拦截）。
2. **AR56-2/AR56-3（中）** → 产物清账口径与 round-024 统计更正（dated 注，不删历史主张）。
3. **AR56-4/5/6/7（中）** → 桥覆盖表述、jp 用例集 15 条同步、D3 C1–C5 执行、报告编号映射注。
4. P3 批（AR56-8..13）→ 与上述同批顺手清。

knowledge 基线（17→18）**不挂起**：可执。L2/L3 双跑与坐标桥的**行为等价结论**（用户核心恐惧
"汉化改版是否改了游戏逻辑"=否）经本审独立重导继续成立；挂起项为口径/工具/门禁层，不动摇该结论。

**四分类路由**：全部 13 findings 落 **规格缺口**（工具/门禁/文档层，回 captain·gates·spec-writer）；
无移植错、无投影错误、无平台差异项。
