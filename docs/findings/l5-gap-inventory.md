# L5 八关可通关：证据缺口盘点 + 环境依赖分档（task-39）

> 日期：2026-10-18｜角色：decomp-analyst（取证/方案）｜状态：**只读盘点稿（未改代码/资产；不 git add/commit）**
> 快照：HEAD `b92139a`（2026-10-18 14:14）；`analysis/` 只有无关目录（`visual-scout/`、`workset/`，**无 traces**）、
> `shots/` 不存在、`tools/mesen2/` 不存在 ⇒ 本报告所有"需重跑"结论都在此前提下成立。
> 纪律：AGENTS.md（A/B/C 分级；未证不发明；数字可复算；结论带证据指针）。判定口径 = `docs/spec/l5-plan.md`
> §2 + `docs/spec/l5-all-clear.md` R-l5-1..8（golden/真闸/诚实锁）。
> 本报告**不主张 L5 达成**：只回答"现在到哪、还缺什么、哪些无需环境、环境窗口内要跑什么"。

## 0. 结论摘要（先读）

1. **L5 判定物现状 = 状态机层达成、真机动态层未达成**：`s5_chain_run_for(8)` 能走 8 关段级链
   （8 stage clear / 8 shop / ending_reached，`crates/game-oracle/tests/l5_s5.rs` 断言），`diff_l5`/`l5_legality`/
   `s5_anchor_rederive.py` 工具链在库；但逐关"战斗内容驱动的自然通关"未达成（G-S5-6′：关 2..8 动态 trace 未跑，
   自然可通关性=显式负结果）。
2. **39 真锚 / 5 占位口径复算成立（A）**：`oracle/traces/s5-all-clear.l5.jsonl` 44 行 = **39 行 f>0 + 5 行 f=0**；
   `crates/game-oracle/tests/l5_s5.rs:68` 锁定 `(real, ph)==(39,5)`；另有 `stg01-chain.l5.jsonl` 17 行（16/1）、
   `tas-clear-full.l5.jsonl` 10 行（10/0）。**但"39 真锚"≠"39 自然锚"**：trigger 分布 = **7 natural +
   32 injected + 5 placeholder**（真锚中只有 7/39 是 natural，均来自标题/STG-01 前缀）。
   > **【2026-10-18 更正·task-52 E10/E1】基数标注**：上面的 7/32/5 是 **44 golden 行基**；**50 锚基**
   > （anchors 台账含 6 条附加注入锚）= **7 natural / 38 injected / 5 placeholder**——任务卡口径=**50 锚基**
   > （`Counter({'injected':38,'natural':7,'placeholder':5})`）。两套口径都保留，引用时须注明基数。
3. **每组动态 trace 的覆盖 = 关始 + 早段战斗到"冻结点"**：stg02 f5033 / stg03 f6893 / stg04 f9164 /
   stg05 f11093 / stg06 f13977 / stg07 f15959 处停滚/冻结（无有效战斗输入 → 中 boss 不死 → tick 冻结）；
   关末/商店/下一关全部由 **poke 注入**推进（38 injected 锚的来源）；**每关 Boss 击杀/自然关末/流末记录边界均未观测**。
4. **stg08 + ending 仍为 5 占位**：注入链（`stg08-timeline` run #1-#5）在末关出店处**结构冻结/截断**（OBS-1，
   bug-ledger 候选不立号）；5 占位 = **1 条未命名 Transition 行（`checkpoint=null`，语义=出店段）** +
   `stg08-start/stg08-combat/stg08-clear/all-clear-95f9` —— 保持 f=0 fail-closed。
   【2026-10-18 更正·task-52 E4】原写"stg08-exit"为不精确命名（golden/anchors 中无该字面 checkpoint）。
5. **缺口的根 = 三条**：①**能打到底的输入轨**（自然 bot 战斗面未达标；或外部 TAS 输入件）；
   ②**每关 boss 击杀/流末边界 trace**（自然通关判据与 golden 需要）；③**stg08/ending 合法来源**（自然链、
   relay 链或注入全链，均需先裁 OBS-1/来源）。其余为规格/门禁/可复现性 A 档工作。
6. **同一环境窗口建议**：一次性跑 **17 行清单（有效 14-16 行：1 行已取消 + 2 行降级为负候选）**
   （§4.2 表；L4 P0-0 + L5 锚重建 + 组 D 重裁候选 + 新 `s5-full-inject`），按 §4.3 优先级执行，
   避免二次开环境；预算 **3.5h（下限）– 8h（上限），C 级估算**。
   【2026-10-18 更正·task-52 E6/E8】原写"17 个场景/overlay + ~3-4h"未扣已取消/降级行、只取下限。
7. **跨任务新证（需 L4 手册二次更正，见 §5）**：L4 P0-0 组 D"来源=v5"已被 `tas-clear-full.l5.jsonl` 证伪
   （f28500-36000=关 2 战斗）；**并且** v5 达到 stage4-combat 已在 f73939 ⇒ L4 现手册 I5 截断 36500 帧既不能
   覆盖 ending（组 D）**也不能覆盖四联所需的 stage6/8 内容帧**（§7.2 选帧前提同时失效；stage6/8 帧位按已测
   关时长外推为 **>110k**，B 级估算）。建议改用 `tas-stg08b2-direct`（stage8 直入、40000f、已含 10 条 probe）
   作 stg08 内容/ending 主来源。该条**已转 task-50**（工作树 L4 手册已改，待提交/复核）。

---

## 1. 现状盘点（可复算口径）

### 1.1 判定物与工具链

| 面 | 内容 | 状态 | 证据 |
|---|---|---|---|
| golden | `oracle/traces/s5-all-clear.l5.jsonl`（44 行/3 场景/36 命名检查点） | ✅ 在库 | 见 §1.2 |
| golden | `oracle/traces/stg01-chain.l5.jsonl`（17 行；16 真锚/1 占位） | ✅ | 文件 + 复审 #98 段 A（16/17 工具直跑自证） |
| golden | `oracle/traces/tas-clear-full.l5.jsonl`（10 行：v5 9 + v7 relay 1） | ✅ | 文件 |
| 锚台账 | `oracle/traces/s5-all-clear.anchors.jsonl`（50 条 = 44 golden 1:1 + 6 附加） | ✅ | 见 §1.2 |
| 规则/真闸 | `l5_legality`（场景内帧序单调 + 已证边集）、`edge_proven`、`diff_l5`（golden 驱动投影 0 容差） | ✅ 在库 | `crates/game-oracle/src/l5.rs`（529 行；`l5_legality`:56、`edge_proven`:111、`diff_l5`:131、`diff_run_face`:161、`s5_chain_run`:307） |
| 状态机层驱动 | `s5_chain_run_for(8)`：8 stage clear/8 shop visit/8 shop exit/ending_reached | ✅ 测试断言 | `crates/game-oracle/tests/l5_s5.rs:73-92` |
| 诚实锁 | 台账↔golden 1:1、`f=0 ⇔ placeholder`、真锚必带 source_run+判定式、**39/5 显式计数** | ✅ | `tests/l5_s5.rs:34-68` |
| run 面对拍工具 | `tools/diff_l5.py`（probe5 ↔ golden；injected 单列不计 natural；非法 trigger fail-closed） | ✅ 工具在库，`--selftest` 实测 **exit 0** | `python3 tools/diff_l5.py --selftest` |
| run 面产物 | `analysis/traces/<scene>.probe5.jsonl`（StageProbe 逐帧） | ❌ **文件当前不存在**，但**引擎侧发射器已落**：`crates/game-core/tests/engine_tick.rs` 的 `#[ignore] fn emit_probe5()`（**10 处**引用）写 `analysis/traces/stg01..08.probe5.jsonl` + `gameplay-stage1.probe5.jsonl` + `respawn-window.probe5.jsonl` + `s5-chain.probe5.jsonl` + `clear-run.trace.jsonl`（**不建目录**，需先 `mkdir -p analysis/traces`） | 【2026-10-18 更正·task-52 E2】原写"全仓 grep probe5 仅 2 处文档引用"**不实**；实仓：engine_tick.rs 10 处 + 文档/tool 4 处。`StageProbe`/`stage_probe` 另见 `l5-instrumentation-survey.md` §1 |
| run 面本机对拍（零 Mesen2） | **A 档可跑**：`mkdir -p analysis/traces && CARGO_TARGET_DIR=target/verify-l5probe cargo test -p game-core --test engine_tick -- --ignored emit_probe5` → `tools/diff_l5.py --probe ... --golden oracle/traces/s5-all-clear.l5.jsonl --anchors oracle/traces/s5-all-clear.anchors.jsonl` | ✅ 命令链可执行（本机；用独立 target 目录避免共享 target 污染） | 【2026-10-18 更正·task-52 E2】natural 面 7 锚可零依赖对拍（injected/placeholder 按工具语义单列/不计 natural；场景映射用 `--scene-map`） |
| 门禁 | L5 **无 gates 条目**（`gates/cli.py` 12 项：rom/seed/assets/naming/knowledge/docs-links/rust-tests/subroutine-diff/trace-diff/visual-diff/completeness-ratchet/addr-bridge；无 L5） | ❌ 未落地（l5-plan §5 承诺"未启用前 L5 显式 SKIP（不静默）"未实现） | `python3 gates/cli.py status` 实跑（快照时 rust-tests FAIL 系并发任务在飞，与 L5 无关） |

### 1.2 39 真锚 / 5 占位 — 复算（A 级）

```bash
python3 - <<'PY'
import json, collections, pathlib
for n in ["s5-all-clear","stg01-chain","tas-clear-full"]:
    rows=[json.loads(l) for l in pathlib.Path(f"oracle/traces/{n}.l5.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
    z=sum(1 for r in rows if r.get("f")==0)
    print(n, "rows", len(rows), "real", len(rows)-z, "placeholder", z)
a=[json.loads(l) for l in pathlib.Path("oracle/traces/s5-all-clear.anchors.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
print("anchors", len(a), "golden", sum(1 for r in a if r.get("golden") is True), "extra", sum(1 for r in a if r.get("golden") is False))
print("trigger", collections.Counter(r.get("trigger") for r in a))
PY
```
实测输出（本机只读）：
```
s5-all-clear rows 44 real 39 placeholder 5
stg01-chain rows 17 real 16 placeholder 1
tas-clear-full rows 10 real 10 placeholder 0
anchors 50 golden 44 extra 6
trigger Counter({'injected': 38, 'natural': 7, 'placeholder': 5})
```
口径要点：
- `s5-all-clear` 真锚 39 = **7 natural**（boot-black/title-in/menu-in/stg01-shop/stg01-exit/stg01-start/stg01-combat）
  + **32 injected**（stg01-clear..stg08-shop，逐段由 poke 注入推进；证据都在 `analysis/traces/<run>.frames.jsonl`，
  **当前 checkout 不存在**，见 §1.5）；
- 5 占位 = **1 条未命名 Transition 行（`checkpoint=null`，mode=Transition，scene=s5-chain，语义=出店段）**
  + `stg08-start / stg08-combat / stg08-clear / all-clear-95f9`（全在 stg08 出店之后；golden/anchors 中**无
  `stg08-exit` 字面 checkpoint**）【2026-10-18 更正·task-52 E4】；
- 6 附加锚（`golden:false`）= 任务书②指定动态捕获项，实际 id 为：
  **`stg02-exit-thanks`@3380 / `gate-26-bit4`@2400 / `flyout-vector`@2794 / `gate300-open`@3066 /
  `inc-180`@3096 / `stg02-shop-menu`@3300**（共 6 条，全 injected；以 `s5-all-clear.anchors.jsonl` 为准）
  【2026-10-18 更正·task-52 E5：原写"$26 bit4 置位/$0300 开/INC $180/$9BC3 init/主菜单/出店帧"不准——
  无 `$9BC3 init`、漏 `flyout-vector`】；
- bot 域 `tas-clear-full.l5.jsonl` 10 行（v5 9 + v7 relay 1）的 `trigger=natural` 与 README §写注入纪律
  "poke 过的 run 产出锚=trigger=injected" **存在口径冲突**（v5/v7 均含 assist 三 poke）——见 C1。
- **trigger 基数（E10）**：44 golden 行基 = **7 natural / 32 injected / 5 placeholder**；
  50 锚基（任务卡口径）= **7 natural / 38 injected / 5 placeholder**（6 条附加锚全 injected）。

### 1.3 每关已证程度表（时间线/敌波/关末/Boss/动态通关）

> 依据：`docs/findings/stage-timeline.md`（STG-01 完整+其余定位）、`docs/findings/stg02..stg08-timeline.md`
> （逐关波次流/记录处理帧表/冻结点）、`docs/findings/stg02-08-en.md`（三级骨架）、`docs/findings/b-stg-01-en.md`
> （STG-01 EN 8 记录 A 级）、`docs/findings/stage-wave-crosscut.md`（八关横切）、`docs/findings/boss-*.md`。
> 级别：A=字节/trace 直证；B=可复现静态推理；C=假设/未证。

| 关 | 时间线/敌波 | trace 窗口（真机） | 关末/商店 | Boss | 动态通关 |
|---|---|---|---|---|---|
| STG-01 | **最全**：EN 8 条生成记录 A（X 布局/4 行为脚本）；L3 player 2400f；`gs5-stg01-full` 15000f 自然回放 | f1080-2400（shop/start/combat）；full 15000f（4 死→GAME OVER） | 注入链 A 级：f2401 关末段→f2794 飞离向量→f3066 gate→f3096 INC $180→f3116 商店→f3450 关 2 开始（全 injected） | 中 boss arena 机制 A（`arena-midboss.md`：HP `$064B`/相位/伤害公式）；**自然击杀未观测** | ❌ 自然关末未达（Run A 15000f 负结果；`$CA` 流末记录全程 0 ⇒ 关长>4000 战斗帧未闭环） |
| STG-02 | EN 242/14 记录、脚本集 {A39B}；波次流/生成器表（`stg02-timeline` §1-§3） | `stg02-timeline` 19000f：12 记录处理帧全验（`$84=+Nt−1` 12/12）；**停滚 f5033**（arena 中 boss 不死→tick 冻结 `$006C=108`）；op5 波次装载视界内**零装载**（负结果） | 注入（下一 run 于滚动活段双 poke f4600） | 中 boss 停滚机制 A 观察；击杀未达 | ❌ 关始+早段战斗；关末/商店/boss 全注入 |
| STG-03 | EN 29/13 + **CNT tick 表 7 段 [16,16,8,16,8,4,16]**；链段族 8 脚本；3 op5 流表 | `stg03-timeline` 24000f：t0=5604、**39 triggers** 记录处理帧表；**冻结 f6893** | 注入 #2（f4600）→ 转场链五段实测（+34f 出店常数） | 中 boss/关末未击杀；行为族 ADxx 脚本已知 | ❌ |
| STG-04 | EN 214/38 + **BOSS/后半族 16 脚本（$A4xx）**；3 op5 流 | `stg04-timeline` 26000f：t0=7404、**52 triggers**；**冻结 f9164** | 注入 #3（f6400） | BOSS 族脚本池已定位（`stg02-08-en`）；**触发/HP 未接** | ❌ |
| STG-05 | EN 219/41 + **EN 族 13 脚本（最全）** | `stg05-timeline` 28000f：t0=9504、**62 triggers**；**冻结 f11093** | 注入 #4（f8500） | 同上（未达） | ❌ |
| STG-06 | EN 250/10（脚本 {B4E6}） | `stg06-timeline` 30000f：t0=11604、**85 triggers**；**冻结 f13977** | 注入 #5（f10600） | 同上 | ❌ |
| STG-07 | 1 条 op5 流（$AF7F，CA=$B1 新形态）；生成器流 $ACDB 46 行（八关最小）；pair 项 17（峰值） | `stg07-timeline` 32000f：t0=14504、**103 triggers**；**早冻结 f15959**（观测窗实际 f14504-15500） | 注入 #6（f13500） | 同上 | ❌ |
| STG-08 | **4 条 op5 流**：`$4A`=中 boss、`$88`/`$62`、`$CA`=终段（`thr=$FFFF` 哨兵语义 S8-2 待证）；CNT 表同 STG-03；写表2 三流共享（S8-3） | `stg08-timeline` 34000f：注入 #7（f15500）→ f15501 $27:=$C0 倒数 → f16221 商店 → **出店截断**（4 位菜单矩阵五 run 实测：唯 pos3 出店→结构冻结/确定性截断）；**无 stg08 关内 trace** | fail-closed 5 占位 | 终 boss 多相机制**机制 A**（`boss-chain-survey.md`：$48C 状态机/64f 节拍/HP 表重装）；**Area-8 四 boss 型值/HP 待 v8 实测**（`bossunit-adjudication.md` W-BOSSID 半收） | ❌ OBS-1 待裁 |

**小结**：8 关的**数据面**（关卡目录/EN 记录/波次流/行为脚本指针/骨架）已到 A/B 级；**动态面**只到
"关始 + 早段战斗 + 冻结点"，**每关的 boss 击杀、关末自然三联（`$27 bit7`+`$2F` 倒数+`INC $180`）、流末记录
（`$CA`）边界均未观测**；stg08 关内与 ending 完全无 trace。

### 1.4 L5 证据可复现性缺口（与 L4 F-0 同族）

锚台账 50 条中：**33 条 evidence 指向 `analysis/traces/<run>.frames.jsonl#fNNN`、12 条指向
`shots/<run>_fNNNNN_<tag>.png`、5 条无 evidence（placeholder）**。这些路径均为 **.gitignore 工作区产物**，
当前 checkout **不存在**（`analysis/` 只有 visual-scout/workset；`shots/` 无）⇒ **39 真锚的判定式当前不可复现**：
- 复算命令（本机可跑，得 33/12/5 计数）：
```bash
python3 - <<'PY'
import json, collections, pathlib
a=[json.loads(l) for l in pathlib.Path("oracle/traces/s5-all-clear.anchors.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
c=collections.Counter("traces" if "analysis/traces" in (r.get("evidence") or "") else "shots" if (r.get("evidence") or "").startswith("shots/") else "none" for r in a)
print(c)   # Counter({'traces': 33, 'shots': 12, 'none': 5})
print(sorted({r.get("source_run") for r in a if r.get("source_run")}))
PY
```
- 需重跑的场景（**E9 更正**）：**anchors 的 `source_run` 去重 = 9**
  （`title-menu`、`gameplay-stage1`、`gs5-stg01-clear-anchor`、`stg03-timeline`、`stg04-timeline`、
  `stg05-timeline`、`stg06-timeline`、`stg07-timeline`、`stg08-timeline`）；
  **并集 11 = 9 + bot 域 2**（`tas-clear-full.l5.jsonl` 的 `tas-clear-full-v5`、`tas-stg07-relay`）——
  原写"11 场景（source_run 去重）"与命令不符，故拆两行（§4 已并入环境清单）。
- **这是 L5 的 P0-0 等价问题**：在环境窗口内应同时产出 **L5 输入冻结清单**（同 L4 P0-0 思路：
  `analysis/traces/{frames,probe}` + `shots` + anchors sha 台账），否则 L5 golden 只是"不可复现的结论快照"。

---

## 2. 逐关"动态通关还缺什么"（新采集 vs 可静态/已有 trace 推出）

> 通用缺口编号 G-L5-x；每关具体缺口在表中引用。判据 = L5 判定物：mode 序列过已证边集 + 4 检查点/关
> + ending 入口；自然通关另需 `$CA` 流末记录→`$26 bit4`→`$27 bit7`→倒数→`INC $180` 三联**自然观测**。

### 2.1 通用缺口

| # | 缺口 | 类型 | 说明/判据 | 现状指针 |
|---|---|---|---|---|
| **G-L5-1** | **能打到底的输入轨**（自然/assisted bot 或外部 TAS） | **B 需采集** | bot 战斗面缺陷已知（避让致命窗冻结/簇聚合/瞄准对齐压制/炸弹闲置；`bot-combat-v2-diagnosis.md`）；v2 于 f9045 GAME OVER；需 bot v3（避让优先/簇聚合/按速前瞻/炸弹决策）或外部 TAS 输入件回放 | `bot-combat-v2-diagnosis.md`；`b2-tas-input.md` §4/§8 |
| **G-L5-2** | 每关 **boss 击杀/HP/相位** trace | **B 需采集** | watch `$0630`(HP)/`$064B`(boss_hp)/`$048C`(相位态)/`hp_delta` 事件；判据=type→HP 绑定 + 击杀帧 + 相位序列；填 W-BOSSID（Area-8 四 boss） | `bossunit-adjudication.md`（5 型 HP A 级；四 boss 待 v8）；`boss-chain-survey.md`（多相机理 A） |
| **G-L5-3** | 每关**流末记录/自然关末**边界 | **B 需采集** | `$CA`(流末)/`$CC/$C8`(记录游标) watch；判据=`$CA bit0`→`$26 bit4`→`$27 bit7`→`$2F` 50→0→`$0300≠0`→`INC $180` 全链自然观测（G-S5-2′ 记录边界帧定位） | `gs5-6-dynamic-replay.md` §2.2（Run B 负结果：单点注入不消费） |
| **G-L5-4** | **stg08 关内 + ending** | **B 需采集/裁定** | 候选：①自然链（bot v8/relay）；②注入全链 `s5-full-inject`；③直接选关（`tas-stg08-direct/b/b2` 在库）。判据=stg08-start/combat/clear 真帧 + `$180==9`→`$95F9` 入口 + 5 占位回填；**OBS-1 同批裁定**（自然链复现截断⇒真身；否则注入伪影） | `stg08-timeline.md` §7；`docs/bug-ledger.md` OBS-1（候选区不立号） |
| **G-L5-5** | 每关敌波**内容语义深度** | **A 可静态推进** | `stage-timeline` S1（段参数=时刻/序号/掩码）未定谳；W28′（CNT→间隔换算，STG-3/8 有序列）、W26（rec6 参数）、W27′（STG-1/7 共享流差异）、W29（子流 tick）——可用 ROM+`wave_timeline/wave_stream_decode` 静态解，无需 Mesen2 | `stage-timeline.md` §7；`stg02-08-en.md` §5 |
| **G-L5-6** | run 面（StageProbe→probe5）对拍 | **A（本机 emit_probe5）+ B（Mesen2 真机 run 面）** | **A**：引擎侧 `engine_tick.rs emit_probe5`（ignored 测试，需先建 `analysis/traces`）+ `diff_l5 --probe` 对 natural 7 锚零依赖对拍；**B**：真机 run 面重采（Mesen2 场景 + StageProbe）用于全字段/injected 面 | `engine_tick.rs:655-843`（发射器）；`l5-instrumentation-survey.md` §1/§5；`l5.rs diff_run_face` |
| **G-L5-7** | L5 门禁/显式 SKIP + 台账可复现 | **A 可推进** | gates 无 L5 条目（l5-plan §5 承诺显式 SKIP 未落）；L5 anchors 的 evidence 全在工作区（§1.4）→ 需 manifest/复现流程（可先文档化） | §1.1/§1.4 |
| **G-L5-8** | spec 口径过期 | **A 可推进** | `l5-all-clear.md` R-l5-8 仍写"12 真锚/32 占位"（2026-10-15 时点）；现行 golden 已 **39/5**（32 injected 回填后）——须 dated 更正（spec-writer） | `l5-all-clear.md` §R-l5-8 注；`gs5-6-dynamic-replay.md` §4 |

### 2.2 逐关缺口与"需新采集 vs 可推出"

| 关 | 还缺（动态） | 需新采集（B） | 可静态/已有 trace 推出（A） |
|---|---|---|---|
| STG-01 | 自然关末（`$CA` 流末→关末门）；自然 boss 击杀 | 有效战斗输入轨（G-L5-1）；boss hp/phase trace（G-L5-2）；流末记录窗（G-L5-3） | 关末/商店/关 2 链的**机制**已 A（`gs5-6` §2.3 注入链动态复现）；EN 8 记录 A；arena 中 boss 机制 A |
| STG-02 | 停滚后恢复（中 boss 击杀）；op5 波次装载；关末 | G-L5-1/2/3；停滚恢复实验（击杀中 boss） | 12 记录处理帧全验（`$84=+Nt−1` 12/12）；停滚机制 A（f5033）；波次流表 A/B |
| STG-03 | 中 boss→关末；CNT 7 段→帧对齐 | G-L5-1/2/3 | 39 triggers 帧表（t0=5604）；CNT 表（[16,16,8,16,8,4,16]）；链段族 8 脚本 |
| STG-04 | $A4xx BOSS 族触发/HP/击杀 | G-L5-1/2/3（BOSS 段专项） | 52 triggers；BOSS 族 16 脚本定位；第 4 关特判 `$82/$83→$64` 字节忠实（G-S5-3 语义待证） |
| STG-05 | EN 族 13 脚本→实际波次；关末 | G-L5-1/2/3 | 62 triggers；EN 族 13 脚本（最全） |
| STG-06 | 最长关（250/10 记录）；关末 | G-L5-1/2/3 | 85 triggers；记录 250/10 |
| STG-07 | 早冻结 f15959 前战斗量；关末 | G-L5-1/2/3（+冻结前战斗） | 103 triggers；1 条 op5 流全解；pair 项 17 |
| STG-08 | **关内 trace 全缺**（start/combat/clear） | G-L5-1/2/3/4（自然或注入全链）；OBS-1 裁定 | 4 条 op5 流表（含 $4A/$CA）；终 boss 多相机制 A；CNT 表；写表共享 S8-3 待证 |
| Ending | `$95F9` 后的 staff roll/结局卡动态 | G-L5-4（到 ending 的 run 的 `$26 bit7`/mode=Ending 帧） | 入口/分派 A（`ending-sequence.md`）；内部 `EndingFx` 挂起（G-S5-4，L5 判定只需 mode=Ending 入口） |

---

## 3. 可执行性分档 + 建议任务卡

### 3.1 A 档 —— 无需环境（静态/已有 trace/文档；本机可推进）

| # | 任务（建议卡） | 角色 | 范围 | 判据 |
|---|---|---|---|---|
| A1 | **L5 golden/台账口径更正**：`l5-all-clear.md` R-l5-8 的 12/32 → **39/5**（32 injected 回填）+ anchors 50(44+6)+trigger 分布 + bot 域 10 行；原地 dated 更正 | spec-writer | `docs/spec/l5-all-clear.md`（+ `l5-plan.md` §5 交叉注） | docs-links PASS；数字可复算（§1.2 命令）；保留历史主张 |
| A2 | **L5 门禁显式条目**：`gates/cli.py` 增加 L5 只读状态/检查（未启用显式 SKIP+理由），不得静默缺失 | gate/tools 负责人 | `gates/cli.py`（+`gates/README.md`） | `gates status` 出现 L5 行（SKIP 或 PASS）；不改其他门禁 |
| A3 | **spec 缺口登记**：把 §2 的 G-L5-1..8 + 每关动态缺口写入 B-STG-xx/EN/TL 子格与 `completeness.md` 状态（L0→L1 等），未证不升级 | spec-writer | `docs/spec/completeness.md`、`rewrite-workset` 相关行（若需 data 改动另卡） | 每格有证据指针；不把"注入链"写成 natural |
| A4 | **敌波语义静态收官**：W28′/W26/W27′/W29 + S1 段参数（用 STG-3/8 的 CNT 序列做对照） | decomp-analyst | findings（stg02-08/stage-wave-crosscut 增量） | 每条结论 A/B 级+复现命令；未证留 C |
| A5 | **boss 型值/HP/相位交叉表**：type→HP（已有 5 型 A）+ 各关 op5 流型值 + Soig 表（B）对账，输出每关 boss 候选与待验判据 | decomp-analyst | findings（boss-unit/chain 增量） | 型值/HP 绑定可复算；Area-8 四 boss 保持 B 候选 |
| A6 | **L5 台账可复现设计**：仿 L4 P0-0 写 `l5-manifest.json` schema + anchors evidence sha 清单 + fixtures 入库提案（与 L4 共用政策） | l4/l5 analyst | findings（L5 部分）或并入 L4 P0-0 手册附录 | schema 可机器校验；fixtures 体积/政策列明 |
| A7 | **probe5 契约文档**：StageProbe JSONL 字段/场景映射/与 `diff_l5 --probe` 消费口径（含 `unknown` 语义示例） | port + spec-writer | `docs/spec/l5-plan.md` 或 `l5-instrumentation` 增量 | 契约与 `diff_l5.py`/`l5.rs` 一致；示例可 parse；**【2026-10-18 更正·task-52 E2】加验收：本机跑通 `emit_probe5` + `diff_l5 --probe`（natural 7 锚全 `ok`，injected/placeholder 按语义单列）** |
| A8 | **OBS-1 台账整理**：断言/证据/三种裁定后果（真身/伪影/未定）预写，待环境裁定即回填 | verifier/test-runner | `docs/bug-ledger.md` OBS-1 条目 + findings | 不提前立号/不预设结论 |
| A9 | **本机 probe5 数据面发射+对拍（E2 新增，零 Mesen2）**：建目录→跑 `emit_probe5`→`diff_l5 --probe` 对 golden/anchors | test-runner/port | `mkdir -p analysis/traces`；`CARGO_TARGET_DIR=target/verify-l5probe cargo test -p game-core --test engine_tick -- --ignored emit_probe5`；`python3 tools/diff_l5.py --probe analysis/traces/gameplay-stage1.probe5.jsonl --golden oracle/traces/s5-all-clear.l5.jsonl --anchors oracle/traces/s5-all-clear.anchors.jsonl [--scene-map ...]` | natural 7 锚 `ok`；injected 单列/placeholder pending 不计 natural；产物 12 文件生成（stg01..08/gameplay/respawn/s5-chain/clear-run）；**不复用共享 target**（lead 设施纪律） |

### 3.2 B 档 —— 需 Linux+Mesen2（采集；独立卡）

| # | 任务（建议卡） | 角色 | 范围/命令 | 判据 |
|---|---|---|---|---|
| B1 | **L5 锚证据重建跑**（11 run = anchors 9 + bot 域 2；§4） | test-runner | `scenario_run.py run <11 场景>`；产出 `analysis/traces` + `shots` | `tools/s5_anchor_rederive.py` → TRACE-DIFF ALL MATCH；33+12 evidence 指针可解析 |
| B2 | **bot v3/v8 通关战役**（自然/assist invincible） | test-runner | `tas_bot.lua` combat v3（G-L5-1 五条升级）+ 八关参数化 + v8 关 8/ending | 段锚四连逐关通过；stg08/ending 帧锚；OBS-1 三分支裁定；trigger 如实（assisted） |
| B3 | **boss 击杀 trace 专项**（每关中 boss+关末 boss） | test-runner+decomp | watch `$0630/$064B/$048C`+hp_delta；每关独立短场景（B1 输入轨复用） | type→HP 绑定≥8 关；击杀帧/相位序列入账；W-BOSSID 收口 |
| B4 | **注入全链通关 run `s5-full-inject`**（无自然链时的合法兜底） | test-runner | 新场景：每关 `$26:=$10+$0400:=$01`（前 6 关已证）+转场+34f；帧预算 ~40k；probe 8 区间+capture_every 300/1500 | **5 占位回填** = 1 条未命名 Transition（出店段，`checkpoint=null`）+ `stg08-start/combat/clear/all-clear-95f9`（trigger=injected 登记）；ending mode 帧存在 |
| B5 | **probe5 真机 run 面重采**（Mesen2；本机 A9 已覆盖 natural 面） | test-runner+port | 8 关真机场景跑 StageProbe→`analysis/traces/<stage>.probe5.jsonl`；`diff_l5.py --probe` 全字段/injected 面 | 全字段 0 mismatch（natural 面）；injected 单列不计 natural；**A9（本机 emit_probe5）已先行交付 natural 7 锚** |
| B6 | **G-S5-3 第 4 关特判验证** | decomp | 第 4 关 `$82/$83→$64` 写入前后 trace 对照 | 字节忠实复现+语义判据（或保持 todo） |
| B7 | **L4 组 D 重裁**（与 L4 P0-0 共用窗口；L4 §3.6 C1-C5） | l4-analyst/test-runner | 候选 C1 `tas-stg08b2-direct` 优先；blob+mode+trigger 三判据 | 命中⇒回填 L4 mapping/expected_source；无⇒`s5-full-inject`/新采集；**同时服务 L5 G-L5-4** |

### 3.3 C 档 —— 需用户/Lead 裁定

| # | 事项 | 选项/影响 |
|---|---|---|
| C1 | **"8 关可通关"验收口径**：natural（无 assist/poke）vs assisted（不死）vs injected（注入链） | 决定 B2 的投入与 L5 golden natural 子集判据；现状 39 真锚仅 7 natural；bot 域 10 行 natural 与 README poke 纪律冲突须一并裁 |
| C2 | **外部 TAS 输入件**（Soig fm2；nicovideo/mediafire） | 若用户代取/网络可达 ⇒ 可能有全流程自然输入轨（JP ROM 与 reference/jp 同 MD5；CN 版需文本差异评估）；否则 bot 自研 |
| C3 | **analysis/fixtures 入库政策**（与 L4 P0-0 §10-2 共用） | 决定 L5 锚/L4 基线能否在干净 checkout 复现（当前 evidence 全在工作区） |
| C4 | **新场景/样本集重构 commit**（`s5-full-inject`、`l5-probe5-*`、组 D 重裁候选 overlay） | 改输入=改证据轨道，需单独 commit+证据说明（AGENTS §9） |
| C5 | **OBS-1 定性后的处置**（真身⇒bug-ledger 立号+Stage A 保真；伪影⇒注入链注记，不改行为断言） | 影响 stg08/ending 5 行与 L4 组 D 重裁结论 |
| C6 | **L5 门禁启用时点**（A2 后何时把 L5 纳入 `gates check`/CI） | 未全绿前置为 SKIP；启用需 Lead/用户批准（同 L4 --strict 两步走） |

---

## 4. 与 L4 P0-0 的相互依赖 + 同一环境窗口采集清单（避免二次开环境）

### 4.1 依赖关系

- **共用输入**：L5 锚重建所需 **11 个场景/run = anchors 9 + bot 域 2**（E9 更正）中，**title-menu /
  gameplay-stage1 / stg03-timeline / stg08-timeline 与 L4 P0-0 I1-I4 完全重合**（L4 手册 §2.1）；
  L5 另需 `gs5-stg01-clear-anchor`、`stg04..stg07-timeline`、`tas-clear-full-v5`、`tas-stg07-relay`。
- **组 D 重裁（L4 §3.6）与 L5 G-L5-4 同源**：候选 `tas-stg08b2-direct`（L4 C1）若命中，既解决 L4 fade/ending
  6 对来源，又给出 **stg08 关内容 + ending 帧锚**（L5 5 占位回填）——一次采集双收益。
- **可复现性同族**：L4 F-0（analysis 不入库/不可复现）与 L5 §1.4（33+12 证据指针悬空）是同一问题 ⇒
  环境窗口内应同时产出 L4 `l4-manifest.json` 与 L5 `l5-manifest.json`（或合并一份 manifest）。
- **C3 政策共用**：analysis/fixtures 入库政策一次裁定，服务两个面。

### 4.2 一次性采集清单（建议顺序；串行跑；C 级耗时估算）

| # | 场景/overlay | 服务 | frames | probe 需求 | 预计 | 关键产物/判据 |
|---|---|---|---|---|---|---|
| 1 | `title-menu` | L4 I1 + L5 锚（3） | 1080 | 无 | ~1min | shots boot/title/menu + anchors 3 条 |
| 2 | `gameplay-stage1` | L4 I2 + L5 锚（4） | 2400 | 源 15 条 | ~2-4min | 22 对 #1-3 + L5 stg01 四锚 |
| 3 | `gs5-stg01-clear-anchor` | L5 锚（11） | 5000 | 无需（RAM 判定式） | ~2min | 关 1 末→商店→关 2 链 11 锚 |
| 4 | `stg03-timeline` | L4 I3 + L5 锚（5） | 24000 | 源 8 | ~10-20min | #4/#5/#12/#13/#14/#15 capture + L5 5 锚 |
| 5 | `stg04-timeline` | L5 锚（5） | 26000 | 源 8 | ~10-20min | L5 5 锚 |
| 6 | `stg05-timeline` | L5 锚（5） | 28000 | 源 8 | ~10-20min | L5 5 锚 |
| 7 | `stg06-timeline` | L5 锚（5）+ L4 四联 stg06 内容帧候选 | 30000 | 源 8 | ~10-20min | L5 5 锚；stg06-late 真值候选（f13000-13977 窗） |
| 8 | `stg07-timeline` | L5 锚（5） | 32000 | 源 8 | ~10-20min | L5 5 锚 |
| 9 | `stg08-timeline` | L4 I4 + L5 锚（2） | 34000 | 源 8 | ~15-30min | #16-#21 capture（四联真值候选）；L5 2 锚；OBS-1 复核 |
| 10 | `tas-clear-full-v5`（或 L5 overlay 截 80000） | L5 bot 域 9 锚证据（stg01-04 面） | 300000（建议 overlay 80000：覆盖 f73939 的 stage4-combat） | **需 overlay 加 8 区间** | ~30-90min（80000f） | bot 域 9 锚 evidence（**不含 stage6/8：按已测关时长外推 >110k，80000 不可能跑到——E7 更正**） |
| 11 | `tas-stg07-relay` | L5 锚（1：stg07-clear f28683）+ L4 组 D 候选 C2 | 120000（可截 ~40000） | **overlay 加 8 区间** | ~15-40min | stg07-clear 锚；ending 候选帧（若达） |
| 12 | `tas-stg08b2-direct` | **L4 组 D C1 + L4 四联 stg08 候选 + L5 stg08/ending** | 40000 | 源已含 10 条 | ~15min | blob==旧 expected 判据；stg08-start/combat/late 真帧；ending 帧锚 |
| 13 | `tas-stg08-direct` / `tas-stg08b-direct` | L4 C3 备选 + OBS-1 | 120000（可截） | 需 overlay | ~20-40min | 选关 stage8 轨对照 |
| 14 | `s5-full-inject`（**新建**，§3.2 B4） | L5 5 占位 + L4 组 D 兜底 | ~40000 | 8 区间+capture_every | ~15-20min | stg08+ending 5 行回填（injected 登记） |
| 15 | `l4-baseline-v5`（overlay） | L4 四联旧路线（**若 L4 手册二次更正后可能废弃/改用途**） | 36500 | 8 | ~15-40min | 仅负候选/对照 |
| 16 | `l4-baseline-v5-pass2`（overlay） | L4 §7.2 四联 pass2（**同 #15，待更正**） | 36500 | 8+选帧 capture | ~15-40min | 同上 |
| 17 | `l4-baseline-stg03` | **已取消**（M2；#13 用源 capture_every） | — | — | — | 不跑 |

> 总计（估算，**E8 更正**）：逐行区间取下限求和 ≈ **3.5h**，上限约 **8h** ⇒ **墙钟 3.5h（下限）– 8h（上限），
> C 级估算**（原写"~3-4h"只覆盖低端）。
> **清单口径（E6）**：表 17 行；**#17 已取消（M2）、#15/#16 降级为负候选对照** ⇒ **有效采集 14-16 行**
> （按 L4 二次更正后 #15/#16 可跳过则 14 行，保留对照则 16 行）。
> 建议顺序：1→2→3→4..9→12→11→10→14→13→15/16（先出 L4 组 D/四联与 L5 stg08 的判据，再跑长跑）。
> **不要 run-all**（40 场景数小时）。

### 4.3 环境内第一优先实验（P0 级）

1. **组 D 重裁 C1**：`tas-stg08b2-direct` 跑完 → 取 28500/30000/31500/33000/34500/36000 的 capture blob 与旧
   expected 比 + 该帧 mode 语义（Ending/fade）→ 命中即回填 L4 §3.6 + L5 stg08/ending。
2. **四联 stg08 真值候选**：同 run 的 stg08-start/combat/late 帧（trace `stage_no==8`）→ 替换 L4 四联中
   stg08 三张（§7.2）；`stg06-late` 用 `stg06-timeline` 自关窗（f13000-13977）。
3. **B1 锚重建**：先跑 #1-#9（9 场景）→ `s5_anchor_rederive.py` 全绿，恢复 L5 可复现性。

---

## 5. 跨任务新证：L4 P0-0 手册需二次更正（另一处 v5 假设失效）

> 本报告 §0-7 已报告并进入 L4 手册 §3.6（组 D 来源≠v5）。**本轮追加一条同类新证（建议立 L4 小卡）：**

- **L4 手册 I5 截断 36500 帧不足以覆盖 v5 的 stage6/8 内容**：`tas-clear-full.l5.jsonl`（bot 域）显示 v5
  进度 = stg01-clear **f28364** → stg02-clear **f51332** → stg03-clear **f72823** → stg04-shop/combat **f73539/f73939**
  ⇒ **stage6/8 内容帧要到 >100k 帧**（v5 总预算 300000f）；而 L4 手册 §2.1 I5/§7.2 用 **36500 帧截断**的
  `l4-baseline-v5-pass2` 按 `stage_no==6/8` 选帧 ⇒ **选帧必然失败**（该 run 在 36500 帧时仍在关 2）。
- **影响**：L4 §7.2 四联（#16/#19/#20/#21）的"v5 pass2 选帧"方案与组 D 同源失效；建议改用：
  - **stg08-start/combat/late** → `tas-stg08b2-direct`（stage8 直入、40000f、源含 10 条 probe；
    trace `stage_no==8` 选帧 + pass2 overlay 加 capture）；
  - **stg06-late** → `stg06-timeline` 自关窗（f13000-13977，源已有 `capture_every 300` + 8 probe），
    或 `tas-stg07-relay` 的前置段（其 stage7 起点前的 stage6 段）。
- ~~**状态**：该条**尚未写入 L4 手册**（本轮 task-48 范围仅组 D）；请 Lead 决定是否再派一张 L4 点改卡
  （预计 docs-only 小卡：§2.1 I5/I5b 行 + §7.2 选帧来源 + §7.7 staging 源 + §8.2 验收项）。~~
  **【2026-10-18 更正·task-52 E3】状态=已转 task-50（工作树 L4 手册已改，待提交/复核）**：
  task-50（owner=l4-analyst，写范围 `l4-baseline-freeze-plan.md`）已派出并已落地工作树改动——
  L4 手册 §2.1 I5=`tas-stg08b2-direct`(40000f/10 probe)、I5-old(36500f) 降级对照、新增 §3.7 dated 更正、
  §8 不再跑 v5 36500f；原卡面点改清单 = §2.1 I5/I5b 行 + §7.2 选帧来源 + §7.7 staging 源 + §8.2 验收项。
  历史陈述（"尚未写入/请 Lead 决定"）已过期，保留留档。

---

## 6. 未证/假设清单（本报告内）

| # | 假设 | 级 | 去向 |
|---|---|---|---|
| H1 | 每关冻结点之后"停滚/冻结"是**中 boss 未击杀**导致（非原版卡死） | B | B2/B3 环境内验证（击杀中 boss 后 tick 恢复） |
| H2 | `tas-stg08b2-direct` 的 28500-36000 帧是 ending/fade（而非 stage8 战斗） | B（候选优先级据 probe 配置/帧数/选关语义，未跑） | §4.3 C1 实验（blob+mode 三判据） |
| H3 | L5 判定可只到 `$95F9` 入口（ending 内部渲染归 L4/后续） | B | C1 口径裁定 + l5-all-clear R-l5-5 |
| H4 | bot v3 升级五条可把自然通关打到关 2+（当前负结果只覆盖 v1/v2 盲开环） | C | B2 战役 |
| H5 | `stg06-timeline` f13000-13977 窗可作 `stg06-late` 真值（"late"=关末前战斗段） | B | §4.3 实验 + spec 定义确认 |
| H6 | 39/5 计数在台账/golden/测试三处一致且为现行口径 | A（已复算） | `l5_s5.rs:68` 锁 |

## 7. 复现命令汇总（只读）

```bash
# 39/5 + 锚分布（§1.2）
python3 - <<'PY'
import json, collections, pathlib
print([ (n, len([1 for l in pathlib.Path(f"oracle/traces/{n}.l5.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]),
         sum(1 for l in pathlib.Path(f"oracle/traces/{n}.l5.jsonl").read_text(encoding="utf-8").splitlines()
             if l.strip() and json.loads(l).get("f")==0)) for n in ["s5-all-clear","stg01-chain","tas-clear-full"]])
a=[json.loads(l) for l in pathlib.Path("oracle/traces/s5-all-clear.anchors.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
print(collections.Counter(r.get("trigger") for r in a))
PY
# 诚实锁/39-5 测试（源码级；环境内跑 cargo test -p game-oracle --test l5_s5）
grep -n "39, 5" crates/game-oracle/tests/l5_s5.rs
# diff_l5 工具自检
python3 tools/diff_l5.py --selftest
# 锚 evidence 悬空计数（§1.4）
python3 - <<'PY'
import json, collections, pathlib
a=[json.loads(l) for l in pathlib.Path("oracle/traces/s5-all-clear.anchors.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
print(collections.Counter("traces" if "analysis/traces" in (r.get("evidence") or "") else "shots" if (r.get("evidence") or "").startswith("shots/") else "none" for r in a))
PY
# 门禁（只读；快照时 rust-tests FAIL 系并发任务，L5 无条目）
python3 gates/cli.py status
```

## 8. 自验收

- **数字可复算**：§1.2/§1.4/§7 全部给命令；39/5、50 锚、33/12/5 证据分类均可本机只读复现。
- **未证不发明**：每关动态状态按 trace/ROM 事实分级；H1-H5 显式假设；不把注入链写成自然通关。
- **可执行**：A/B/C 三档每项有角色/范围/判据；§4.2 采集清单可排期（有效 14-16 行、3.5-8h）。
- **跨任务**：组 D 冲突写入 §5（L4 §3.6 已更正）；I5/§7.2 失效新证**已转 task-50**（工作树已改）。
- **probe5 本机面（E2）**：引擎侧 `emit_probe5` 发射器在库 ⇒ natural 7 锚对拍归 A 档（A9），Mesen2 仅真机 run 面。
- **docs-links PASS**（本文件相对链接均指向存在文件/目录）。

## 9. task-51 复核修正记录（E2/E3 + Low，2026-10-18 task-52）

> 来源：[adversarial-review-l5-gap-inventory.md](adversarial-review-l5-gap-inventory.md)（verdict=needs_revision；
> 核心数字/结论已独立复算通过，仅 2 必改 + 5 Low）。本表为定点修正落点（原地 dated 更正，不 rewrite 历史）。

| # | 级 | 修正落点 | 内容 |
|---|---|---|---|
| **E2** | 中（必改） | §1.1（`run 面产物`/新增`run 面本机对拍`行）、G-L5-6、§3.1 A7+A9、§3.2 B5、§8 | 更正"仅 2 处文档引用"错误：`engine_tick.rs` 10 处 `#[ignore] fn emit_probe5()`（写 stg01..08/gameplay-stage1/respawn-window/s5-chain probe5 + clear-run trace；**不建目录**）；G-L5-6 拆 A（本机 `mkdir -p analysis/traces` + `CARGO_TARGET_DIR=target/verify-l5probe cargo test -p game-core --test engine_tick -- --ignored emit_probe5` + `diff_l5 --probe` natural 7 锚）/ B（Mesen2 真机 run 面）；新增 A9 卡；A7 加验收；B5 改真机面 |
| **E3** | 中（必改） | §0.7、§5、§8 | 状态由"尚未写入/请 Lead 决定"改"**已转 task-50（工作树 L4 手册已改，待提交/复核）**"，原句划线留档 |
| E4 | Low | §0.4、§1.2 | 5 占位第 1 行 = `checkpoint=None` 未命名 Transition（出店段），非字面 `stg08-exit` |
| E5 | Low | §1.2 | 6 附加锚改实际 id：`stg02-exit-thanks`@3380 / `gate-26-bit4`@2400 / `flyout-vector`@2794 / `gate300-open`@3066 / `inc-180`@3096 / `stg02-shop-menu`@3300；删 `$9BC3 init` |
| E6 | Low | §0.6、§4.2 总计注 | 17 行清单 → **有效 14-16 行**（1 取消 + 2 降级） |
| E7 | Low | §4.2 #10 | 删"四联 stage6/8 候选（若跑到）"（80000f < >110k 矛盾），只作 bot 9 锚 stg01-04 证据 |
| E8 | Low | §4.2 总计注 | 工时改 **3.5h（下限）– 8h（上限），C 级** |
| E9 | Low | §0.7、§1.4、§4.1 | "11 场景" = anchors source_run 去重 **9** + bot 域 **2**（v5/relay） |
| E10 | Low | §0.2、§1.2 | trigger 基数注明：**44 golden 基 7/32/5；50 锚基（任务卡口径）7/38/5** |
