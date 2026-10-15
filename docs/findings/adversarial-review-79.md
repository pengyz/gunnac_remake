# 对抗复核 #79 — B2 修复效果终验（第二轮轻量：F1-F5 + 两顺手结论路由）

> 独立复审（fresh context，证伪立场）。被审对象：commit **fe883f1**（fix(tools)：tas_gen when
> 死语法 fail-open 封堵）+ **7c7d798**（fix(data)：B2 数字口径 dated 更正 + Wt-2 关闭 + G-S5-6 补注），
> 即复核 #78 判 needs_revision（F1-F5 + 两顺手结论路由）的修复落地。
> 方法：**不采信被审自述**——自建试调矩阵（5 形态 when 攻击 + 对照组 + 3 形态路径穿越）、
> sha 三重锁独立复算（重生成==入库==run.json）、mutation 1（VEL 篡改→还原全链）、
> dated 更正数字全数从 trace/快照独立复推（pstate 直方图/死窗签名/快照 0x4A 差分/tick 段统计）、
> 全库 grep 裸露扫描、四件测试电池复跑。

**Verdict: pass** —— 六项核验全落地：F1 fail-closed 真拒（试调 5+3 形态全 exit=1 零落盘，
test_tas_gen 14/14）；F2-F5 dated 四点更正齐且数字全数独立复算逐值吻合；"5.1×"全库无裸露；
Wt-2 dated 裁定关闭（四证链 + W-tas-3 显式保留 + 全库 8 处全标裁定）；G-S5-6 补注在文；
sha 三重锁零变动；mutation 真咬合且还原无痕。四件电池 144/144 + 121/121 + 12/12 + 14/14 全绿。
残余 2 条 advisory（L，不阻断）见 §8。

---

## 0. 六项核验表（验收判据①）

| # | 核验项 | 方法（本审独立） | 结果 |
|---|---|---|---|
| 1 | F1：when 显式拒绝 + test 14/14 + --name 拒穿越 | 自建 5 形态 when 试调 + 对照组 + 3 形态 --name 试调；`tools/test_tas_gen.py` 复跑 | ✅ §1 |
| 2 | F2-F5：dated 四点更正 + 5.1× 无裸露 | 逐点读文 + 全库 grep + trace/快照独立复算数字 | ✅ §2 |
| 3 | Wt-2 关闭：dated 裁定（四证链）+ W-tas-3 保留 + 标已裁定 | 逐行读文 + 全库 grep "Wt-2" + trace 独立复推证链①④ | ✅ §3 |
| 4 | G-S5-6 补注（未连发主武器口径） | 逐行读文 | ✅ §4 |
| 5 | sha 三重锁：重生成三场景 = 0882cb26/ea4ded70/06eaa85d | 三场景重生成（scratch 出盘）sha256 + cmp 逐字节 + run.json 对账 | ✅ §5 |
| 6 | mutation 1：改 VEL→sha 应变→还原 | 篡改 2.8125→2.0 重生成→git checkout 还原→复验 | ✅ §6 |

## 0.1 复跑电池（验收判据②）

| 电池 | 命令 | 实际输出 |
|---|---|---|
| cargo | `cargo test --workspace` | **总通过 144**（37 条 result 行求和，`grep failed` 无非 0 项）——基线 144 不降 |
| l2-diff | `python3 tools/diff_subroutines.py` | `L2-DIFF: 121/121 通过（golden==ROM 执行==game-core 同名函数，三者缺一不可）` |
| gates | `python3 gates/cli.py check` | `SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0`；`CHECK: PASS（含 0 个静默跳过：所有 SKIP 均已显式打印）` |
| tas_gen | `python3 tools/test_tas_gen.py` | `tas_gen fail-closed 回归: 14/14 通过` |

---

## 1. F1：when 死语法 fail-open 封堵 —— **PASS**（试调 5+3 形态 + 对照 + 14/14）

### 1.1 自建试调矩阵（不复用被审方测试；基计划=stg01-diag.tas 去注释体 + 攻击行，`-o` 指向 scratch）

| # | 输入 | 实际输出 | exit | 落盘 |
|---|---|---|---|---|
| T1 | `when frame in [3,4] then hold 5-10 b`（白名单动词） | `PlanError: …when 生成期展开未实现（档位=显式拒绝…）：hold 动作不展开；条件相位切换请用多段 band <起帧>` | 1 | 无 |
| T2 | `when frame in [3,4] then poke 0026 10`（**子句内嵌写注入**） | `PlanError: …生成器拒绝 when 子句内 poke：TAS 输入轨只允许按键（README 写注入纪律）——when 内嵌写注入不得静默吞` | 1 | 无 |
| T3 | `when frame in [3,4] then restore 50` | 同上（restore 语义） | 1 | 无 |
| T4 | `when frame in [3,4] then selfdestruct`（白名单外） | `PlanError: …动作动词 'selfdestruct' 不在白名单 ('hold','press','avoid')` | 1 | 无 |
| T5 | `when frame in [3,4] hold 5-10 b`（缺 then） | `PlanError: …when 子句缺 then <action>` | 1 | 无 |
| 对照 | 同基计划无 when | `生成 …/trial-valid.txt（8156 行）` | 0 | 正常出盘（证明拒绝非一刀切） |
| T6 | `--name "../evil"` | `PlanError: --name 含路径分隔符/非法字符（只允许单段名）` | 1 | 无（`/tmp/adv79-f1/evil*` 不存在） |
| T7 | `--name "a/b"` | 同上 | 1 | 无 |
| T8 | `--name ".."` | `PlanError: --name 非法（空名/点段）` | 1 | 无 |

- 落盘核验：`ls /tmp/adv79-f1/out/` 仅对照组 2 文件（trial-valid.txt/.report.json），攻击面零写入。
- 注：`/tmp/evil.report.json`（mtime 08:49:18）经查为**修复前漏洞复现遗留物**（早于两 commit 09:11 与本试调 09:13），
  非现时泄漏——修复后路径穿越在解析期即拒。
- `python3 tools/test_tas_gen.py` → **14/14 通过**（14 测试名逐条见输出：when 内嵌 poke/restore/snapshot、
  白名单外动词、畸形 when、CLI exit=1 不落盘、路径穿越、顶层 poke/未知指令不回退、合法计划仍可解析）。
- 附落地面（F1 路由同步项）：`docs/spec/tas-input-dsl.md` §1.2 已挂【2026-10-15 更正】——
  when 降级"未实施（显式拒绝）"、谓词/动作/展开三处原文全打删除线并注档位理由（未证不发明：
  谓词求值语义未证 → 不展开，走已验证的多段 band 路径）；生成器 report 新增 `unimplemented_dsl`
  显式声明 ✅。A1 顺手项（`--name`/`prefix` 消毒 + `BOMB_DEPLOY_FRAMES`→`BOMB_DEPLOY_DEDUP_HINT`
  改名防"禁止硬编码 209"误读）一并落地 ✅。

## 2. F2-F5：数字口径 dated 更正 —— **PASS**（四点齐 + 数字全数独立复算 + 5.1× 无裸露）

### 2.1 四点逐项（docs/findings/b2-tas-input.md）

| 项 | 落地文（dated） | 本审独立复算 | 判 |
|---|---|---|---|
| **F2** f9689 语义/f9930 首现 | L62：~~f9689 出敌~~ → **f9689=波次记录 1 装载帧（tick 816，投影 5561+(816−558)×16=9689 与实测全等）**，非出敌；slot27 type $4A **首现 f9930 自顶部入画**（f9900 全场无 type 74；出敌点≈f9910-9930；下行轨迹 f9930(153,15)→f9960(148,35)→f9990(155,54)→f10020(153,75)，快照 30f 粒度） | v2-diag 快照直读：f9900→f9930 差分 290 字节，**0x4A 于 f9930 在 0x150-0x152/0x41b 新现**（f9870/9900 无）并持续 f9960-10020 ✅ | ✅ |
| **F3** f9204 分类/倍率 | L58：~~f9204 首死~~ → **1 帧 pstate15 瞬态**（f9205 续移 (116,158)、残机不变、金钱不变——非真死）；真死对照=f10140；**同类对比 5.68×**（f10140 vs 盲开环 f1787）/3.44×（vs 探针 f2945），原 5.1×/3.1× 标"非同类对比"双口径并列 | trace 直读：pstate15 窗 = [9204-9204]**窗长 1** + 10140/10448/10736/11014/11298 各 128f；签名 f9203(1,120,158,3,3)→9204(15,118,156,3,3)→9205(1,116,158,3,3)（**续移+残机 3+金钱 3 不变**）；f10140(15,68,189,4,3)→10141(15,68,189,**3,1**)（**残机−1+金钱跌**）✅ | ✅ |
| **F4** 9592f 分解 | L68-80：dated 全表（pstate 直方图 `{1:9592, 0:1777, 108:987, 15:641, 124:3}`）：战斗活跃 **9592f** / 死窗 641f / 前奏 990f / boot+GO 后 1777f；旧"复活 1777f 非活跃，存活 10582f"作废并注因 | trace 逐帧重算直方图 = `{0:1777, 124:3, 108:987, 1:9592, 15:641}`，合计 13000，**逐值全等** ✅ | ✅ |
| **F5** 433 点+三附项 | L109-110：~~383 快照~~ → **433 点**（13000/30，run.json captures=433；"383"=v2-diag 点数复制笔误）；附项① $0630 相位限定（f10020=$0E、f10260 起=$0C）✅；附项② f10736 粒度注（f10710 最近 79.5px/f10740 距 14.9px，"不宣称接触帧"）✅；附项③ 弹网格活动槽改"首组 Y=$80-$84 为主，含组 2 槽 Y=$9C/$9D 偶发活跃"✅ | run.json 直读：v3 `done.captures=433`、v2-diag `done.captures=383`——与更正文逐值吻合 ✅ | ✅ |

### 2.2 裸露扫描（验收判据①第 2 项后半）

- `grep -rn "5\.1×\|5\.1x\|3\.1×"` 全库（md/py/txt/json/tas）命中 4 处：b2 L13/L58 均在
  **删除线 +【2026-10-15 更正】语境内**（历史主张按证伪更正文化保留）；adversarial-review-78.md
  2 处为**复核 #78 归档自身**（F3 发现陈述与修正建议，历史语境）⇒ **无裸露主张** ✅。
- 连带旧口径扫描：`f9689 出敌` 仅存删除线内；"复活 1777f 非活跃/存活 10582f" 仅存删除线与
  "旧分解…作废"语境；"存活 9204f" 仅存删除线与"=首个 pstate15 事件帧（1f 瞬态）"口径注 ✅。

## 3. Wt-2 关闭 —— **PASS**（dated 裁定 + 四证链 + W-tas-3 保留 + 全库标裁定）

- `docs/findings/enemy-timeline.md` §8 Wt-2 行（L418）**原地 dated 裁定关闭**（删除线留原文）：
  **tick = 16px 滚动进度（A 级）**，四证链齐——①三段全等（tick17-192 恒 8f n=175 × 2.0px/f；
  tick192-288 恒 2f n=96 × 8.0px/f；tick450-815 恒 16f n=365 × 1.0px/f；中间 ramp 与 set-speed 吻合）
  ②滚动位移直证（`$8B` 逐帧 −2 + `$8C:$8B` 16 位计数递减，1px/f 段 90f 恒 −90）③`bank00:8D3A`
  减速表逐指令（`$8D84,Y` 表 `40 60 80 A0 C0 D0 E0 F0`，`$194/$195:=tick+8`，816+8=**824** 冻结 ✓）
  ④减速段 tick 间隔 16/17/18/19/21/24/29/40/59 ≈ 16px/速度逐点，速度归 0→tick 停 ✅。
- **W-tas-3 显式保留**："不因本裁定关闭"（tick 816→1232 尾段外推未实测——arena 停滚期 tick 不推进、
  rec12 set-speed 后区间未测），外推仍标假设 ✅。状态列已改"**C→已裁定**｜已解（复核 #78 §4），不再作假设"。
- 全库 `grep "Wt-2"` 命中 8 处（enemy-timeline L85-86/418、tas-input-dsl L119-121/156、
  b2-tas-input L198、tas_gen.py L58/69/404）**全部带裁定/更正标注**，无裸露"假设 Wt-2"残留 ✅
  （L85 旧句已打删除线）。
- **本审 trace 独立复推（证链①④，v2 trace tick_hi:tick_lo 逐帧自写脚本）**：tick17-192 间隔集合={8}
  n=175 ✅；tick192-288={2} n=96 ✅；tick450-815={16} n=365 ✅；tick815-824 间隔序列=
  **[16,17,18,19,21,24,29,40,59]** 逐值全等 ✅；tick 最大值 824、首现 f9916 ✅（"f9916 起停"吻合）。
- 同步面：`docs/spec/tas-input-dsl.md` §7 Wt-2 行 + §4 更正注、`tools/tas_gen.py`
  `TICK_EXTRAP_F_PER_TICK` 注释（补"tick558-815 实测 257 tick 全 16f"+外推仅覆盖 tick 816→1232 未测尾段）
  三处同步落地 ✅。

## 4. G-S5-6 补注 —— **PASS**

- `docs/findings/gs5-6-dynamic-replay.md` L104-116【2026-10-15 补注（复核 #78 §5）】在文：
  **盲开环战斗段未连发主武器（≤1 单击）**——按键语义重导（A 级：B 按住=主武器经 `bank3:$A1A9`
  门控连发；A=投放/炸弹 `$16` bit7 边沿 + `$0401=7` 在途）；Run A 输入=`press_every … a` 全为
  投放 + 仅 `press 1500 b` 单击（press_len 4f<8f 重装窗，至多 0-1 发）；场景注释"自动射击"系误读
  （**场景文件不改**——改注释会破 run.json scenario_sha256 对账，口径以补注更正——处置正确 ✅）；
  "存活时长/死因按未连发主武器口径重读"、"不构成弹幕不可避证据"、"不代表连发主武器口径下不可存活
  （该口径未测）"保守三注齐；主干"无法自然通关（盲开环）"**不推翻** ✅。
- 结论行同步：L101 "~~自动射击~~【2026-10-15 更正】未连发主武器（≤1 单击）+摆动" ✅。

## 5. sha 三重锁 —— **PASS**（重生成==入库==run.json，逐字节零变动）

| 场景 | 重生成 sha256 | 入库 sha256 | run.json `scenario_sha256` | cmp |
|---|---|---|---|---|
| tas-stg01-v2 | `0882cb26503a68e0…c2e0` | 同 | 同（`analysis/traces/tas-stg01-v2.run.json`） | 逐字节相同 |
| tas-stg01-v2-diag | `ea4ded70d848f8d8…2dc2` | 同 | 同 | 逐字节相同 |
| tas-stg01-v3 | `06eaa85d03f1c9cb…1725` | 同 | 同 | 逐字节相同 |

- 重生成命令（scratch 出盘，未触入库）：`python3 tools/tas_gen.py gen data/tas/stg01{,-diag,-v3}.tas
  --name tas-stg01-v2{,-diag,}/v3 -o /tmp/adv79-sha`。
- 三份 `.report.json` 与重生成**逐字节一致**（入库 report 已随修复同步；不在 sha 锁面）✅。
- run.json 历史记录**零改写**（两 commit 未触 `analysis/traces/*.run.json`，不 falsify 过往 run）✅。
- 结论：F1/A1 修复对生成物**字节零变动**，锁面未漂移。

## 6. mutation 1（锁真咬合）—— **PASS**

| 步 | 操作 | 实测 |
|---|---|---|
| 篡改 | `VEL_PX_PER_FRAME 2.8125→2.0`（tools/tas_gen.py:44） | 工作树生效 |
| 重生成 | gen stg01.tas → scratch | sha=`a76d1d7eeba3098e…` **≠ 锁 0882cb26**；diff **12801 行** ✅ 咬合 |
| 交叉吻合 | — | 与复核 #78 M1 同篡改得同 sha `a76d1d7e…`——生成器确定性独立交叉佐证 |
| 还原 | `git checkout -- tools/tas_gen.py` | 文件 sha=`fc6be7f7…` == `HEAD:tools/tas_gen.py` 逐字节同 ✅ |
| 复验 | 还原后重生成 | sha 回 **0882cb26…** == 锁 ✅ |

——锁面（入库 sha / run.json 记录 / 重生成复现）三向对篡改敏感、对还原自洽。

## 7. 纪律面（附查）

- 两 commit diff 范围 = `tools/` + `docs/` + `data/tas/` + `oracle/scenarios/*.report.json`；
  **rom/、reference/、data/naming/、data/status/、gates/ 零触碰** ✅。
- 证伪更正文化执行到位：全部旧主张以删除线保留 + 日期更正注，未删历史 ✅。
- commit message 两份均含 Evidence/Confidence 字段 ✅。

## 8. 残余 advisory（L，不阻断 verdict）

| # | 内容 | 建议 |
|---|---|---|
| A1（L） | b2-tas-input.md L66（"首死同 f9204（前缀同源）"）/L108（"首死同帧 f9204"）两处沿用"首死"措辞未挂"1f 瞬态"限定——与 L58/74/82 的 dated 口径并存。同文档口径已立（非裸露主张），但措辞残留可致快读误取 | 下轮顺手补挂限定（如"首个 pstate15 事件同帧 f9204（1f 瞬态）"） |
| A2（L） | `docs/findings/adversarial-review-78.md` 至今 **git untracked**（#75/#76/#77 均已归档入库）——前审判定书缺 commit，#79 同样面临归档问题 | 归档 #78+#79 入库（docs commit），与既往归档惯例一致 |

## 9. 结论（验收判据③）

**Verdict: pass** —— 六项核验逐项落地（§0 表：5+3 试调形态、dated 四点、Wt-2 四证链裁定、
G-S5-6 补注、sha 三重锁零变动、mutation 咬合+还原无痕），复跑四件全绿（144/144 + 121/121 +
12/12 + 14/14，实测输出见 §0.1），红线面零触碰，证伪更正文化执行完整。复核 #78 的
needs_revision 项 **全部收口**；B2 战役负结果主干（中 boss type $4A 未击杀→时间线冻结→GAME OVER）
与"不宣称不可避"诚实姿态维持不变。残余 2 条 advisory（§8）不阻断，建议下轮顺手清。
