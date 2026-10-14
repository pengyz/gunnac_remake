# 对抗复核 #75 — G-S5-6 修复效果终验（第二轮轻量）

> 独立复审（fresh context，证伪立场）。被审对象：修复 commit `0b70a15`
> （「G-S5-6 修复循环——复核 #74 F1/F2 表述 dated 更正 + F3-F6/A2/A3/F4 清」），
> 承接前审 `adversarial-review-74.md`（needs_revision：F1/F2 M 级 + F3-F6 低 + A1-A3 advisory）。
> 方法：不采信修复方自述——**事实面先独立重导**（自写脚本逐帧复算 raw trace，非复用
> `s5_anchor_rederive.py`）→ 措辞面逐 grep（裸露活主张全仓扫描）→ 重导工具独立重跑 +
> 非空转探针 → mutation 电池实测 → 对拍四件独立复跑贴数。
>
> **Verdict: pass**（六项核验全部落地；F1/F2 更正口径与 raw trace 独立复算逐项吻合，
> 裸露假主张归零，重导工具 15/15 且非空转探针咬合，mutation 锁咬合 + 还原净）。
> 剩余 2 条 advisory（M-b/M-c 边界 = 前审 A1 同域残余，不在本轮修复清单，见 §6）。

---

## 0. 对拍独立复跑（贴数，未采信自述）

| 复跑项 | 命令 | 实测输出 | 判定 |
|---|---|---|---|
| workspace 测试 | `cargo test --workspace` | **144 passed / 0 failed**（FAILED 计数 0） | ✅ |
| L2 子程序级 | `python3 tools/diff_subroutines.py` | **L2-DIFF: 121/121 通过（golden==ROM 执行==game-core）** | ✅ |
| 门禁总闸 | `python3 gates/cli.py check` | **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0**（0 个静默跳过；rust-tests 基线 144/现值 144） | ✅ |
| 锚锁测试 | `cargo test -p game-oracle --test l5_s5` | **4 passed / 0 failed** | ✅ |
| 锚重导工具（F3，独立重跑） | `python3 tools/s5_anchor_rederive.py` | **SUMMARY: 15/15 通过（对拍 8 + 更正口径 7）→ ALL MATCH**，exit 0 | ✅ |
| 工作树 | `git status --short` | 净（全部 mutation 已还原） | ✅ |

## 1. 六项核验总表

| # | 核验项 | 要求 | 实测 | 判定 |
|---|---|---|---|---|
| 1 | **F1** dated 更正 + 裸露归零 | findings §2.1/§5 + anchors.jsonl gate-26-bit4 含「关末语义自然触发未观测」+ 三联判据 + bit4 过渡窗 f913-1192 定界；「bit4 全程 0」裸露活主张零 | findings L88（原文删除线）+ L92-99 更正注 + §5 L223 同步 + anchors.jsonl:46 判定式改写（dated【2026-10-15 更正（复核 #74）】）；全仓 grep「bit4 全程 0」= 0 处裸露（仅删除线/引文/更正注内"为假"）；**事实独立复算全吻合**（§2） | ✅ |
| 2 | **F2** 真口径三段 + 裸露归零 | findings §2.3/§2.4 含 c1↔c2 f1..3200 / c2↔anchor 全程 / GP↔full 7 字段×2400f；「全 watch×5000 帧 0 mismatch」裸露零 | §2.3 L118-121 + §2.4 L151-153 三段齐（含 f3201..5000=1800 帧设计性分歧注 + watch 映射附注）；grep「全 watch × 5000」= 0 处裸露（仅删除线）；**事实独立复算全吻合**（§2） | ✅ |
| 3 | **F3** 重导工具存在可跑 | tools/s5_anchor_rederive.py 存在且独立重跑 15/15 | 工具 244 行入库（`0b70a15`）；独立重跑 **15/15 ALL MATCH**（8 对拍锚 + 7 更正口径）；**非空转探针**：trace f2794 vec438 13→14 → 工具 **13/15 MISMATCH exit 1**（flyout-vector + c2↔anchor 双检查咬住）→ 还原后 15/15；findings §6① 复现命令已入库（原「复跑见 git 记录」已替换） | ✅ |
| 4 | **F5/F6** 日期 + 头注 | README 日期 2026-10-15；inject 头注 dated 更正（预期链非事实） | README L40：`~~2026-01-15~~ 2026-10-15` + F5 dated 更正注 ✓；inject.txt L12-23：F6 dated 注（「ROM 自然执行 E2F3…」=**预期链误写成事实**，实测=负结果 run 零锚入库 fail-closed，原文删除线留档）+ F1 同错复读处 dated 注 ✓ | ✅ |
| 5 | **A2/A3/F4** 措辞收紧在文 | findings+spec:146 注；F4 分级单一真相源注 | A2：findings §3 L163-166（44 条 golden 1:1 + 6 附加；source_run=run 名、sha256 在 run.json）+ **spec:146**（`~~50 条；golden 1:1~~【…复核 #74 A2 同族】44 条 golden 1:1 + 6 附加`）✓；A3：标题收窄「…真机动态层**·关 1 段**」+ dated 注（原标题留档）✓；F4：findings §4 L210-215 单一真相源裁定（分级语义单一真相源 = 台账 `trigger` 三分 + `l5_s5` ①锁；golden 单独被消费时**帧锚语义以台账为准**（R-l5-8 口径，spec:146「逐锚判定式/证据指针/触发来源 = anchors.jsonl」承载）） | ✅ |
| 6 | **mutation 复验** | anchors.jsonl 判定式或占位标记篡改 → l5_s5 锁红 → 还原 | **2 咬合**：占位标记篡改（stg03-shop placeholder→natural，f=0 不动）→ 锁红 @l5_s5.rs:53 `[13] f=0 必须标注 placeholder`；golden 真锚判定式清空（stg01-clear）→ 锁红 @l5_s5.rs:60 `[7] 真锚必须带判定式`。**2 边界**（§6）：非 golden 附加锚判定式清空 / 判定式文本内容替换 → 锁绿。全部已 `git checkout` 还原，工作树净 | ✅（咬合面见 §4/§6） |

**六项全落地 ⇒ Verdict: pass。**

## 2. 事实面独立重导（自写脚本逐帧复算 raw trace，不采信重导工具）

### 2.1 F1（Run A = `gs5-stg01-full` 15000f）

| 断言 | 独立复算实测 | 判定 |
|---|---|---|
| `$26 bit4` 置位窗 = **f913..1192（280 帧连续）** | 逐帧扫描 flags26&0x10：窗 = `[(913,1192,280)]` 唯一；f912=0 → f913=16 … f1192=16 → f1193=0 | ✅ 精确 |
| 过渡窗语义 = 开 stage（非关末） | f1193 intro_ctr=64（关始引入）；窗口期 countdown2f=199（histogram：199×5613 帧 / 0×8971 帧，无 50→0 环） | ✅ |
| `$CA` 全程 0（原断言存活部分） | trig_ca 非零帧数 = **0** | ✅ |
| 三联腿② 倒数环（50→0，每 8f DEC） | **无**（全 run 无满足 8f cadence 的 50→0 序列） | ✅ 未出现 |
| 三联腿③ INC $180（stage_no 转移） | **无**（stage_no 全 run 恒 1） | ✅ 未出现 |
| 三联腿① flags27 bit7 入场 | Run A 轻量面未 watch $27 → 经同前缀 anchor run 旁证：f1..2399 共有 14 字段 **0 mismatch**（同源前缀成立），anchor 同窗 f913-1192 **flags27 bit7 全 0**；flags27 bit7 全 anchor run 仅 **f2401..3403**（=poke 触发的关末段） | ✅ 旁证链闭合 |

→ **F1 更正口径（「关末语义自然触发未观测」三联判据 + bit4 过渡窗定界）与 raw trace 逐项吻合**；
findings §2.1/§5 与 anchors.jsonl gate-26-bit4 判定式的更正表述可机器重导。

### 2.2 F2（确定性对账三段）

| 对账 | 独立复算实测 | 判定 |
|---|---|---|
| c1↔c2（18 watch 字段） | **f1..3200 = 0 mismatch**；f3201..5000 = **1800 帧** mismatch（连续 3201..5000，差异字段=flags27/intro_ctr/invuln_6a0/phase2b/player_state/slot1_state/vec41c/vec438——脚本差异仅 f3116 后：c1 press_every B 试探 vs c2 down 导航，**设计性分歧**） | ✅ 精确 |
| c2↔anchor（18 字段 × 5000f） | **mismatch = 0**（全程） | ✅ 精确 |
| gameplay-stage1 ↔ full（7 字段 × 2400f） | **mismatch = 0**（按场景 watch 映射取值：game_action_mode=$0401 / zp=$0000-00FF / iram=$0300-07FF / wram=$6000-7FFF SRAM 区）；**反证**：若 $26 误按 wram $0000 基址解 = 2400 mismatch——附注的映射警告确属必要 | ✅ 精确 |

→ **F2 三段真口径（c1↔c2 f1..3200 / c2↔anchor 全程 / GP↔full 7 字段×2400f）全部独立复算成立**；
「全 watch×5000 帧 0 mismatch」假主张在文档层已删除线归零。

## 3. F3 重导工具独立重跑（贴输出）+ 非空转探针

```
$ python3 tools/s5_anchor_rederive.py
[ok] RAM stg01-clear f2401: flags27=0x80 ∧ countdown2f=50（f2400 flags27=0）
[ok] RAM flyout-vector f2794: countdown2f=0 ∧ vec41c=164 ∧ vec438=13（f2793=1，判定帧唯一）
[ok] RAM gate300-open f3066: gate300=0x10 ∧ flags26=0x00（bit4 清）
[ok] RAM inc-180 f3096: stage_no 1→2（INC $180 写帧）
[ok] RAM stg02-shop f3116: stage_no=2 ∧ player_state=0（$9BC3 init）
[ok] SHOT stg02-exit f3400 / stg02-start f3450 / stg02-combat f3900: capture+截图/RAM dump 实存
TRACE-DIFF: ALL MATCH (8/8)
── 复核 #74 更正口径复算（F1/F2）──
[ok] F1 bit4 过渡窗定界（Run A）: 置位窗 = [(913, 1192)]（280 帧；非「全程 0」）
[ok] F1 过渡窗语义（开 stage）: f1193 bit4 清 + intro_ctr=64
[ok] F1 $CA 全程 0（此条原断言成立）: trig_ca 非零帧数 = 0
[ok] F1 关末语义三联未出现（Run A）: 倒数环=无；INC $180 转移=无
[ok] F2 c1↔c2 共享段（18 字段 × f1..3200）: mismatch=0；f3201..5000=1800 帧设计性分歧
[ok] F2 c2↔anchor 全程（18 字段 × 5000f）: mismatch=0
[ok] F2 gameplay-stage1↔full（7 字段 × 2400f）: mismatch=0
SUMMARY: 15/15 通过（对拍 8 + 更正口径 7）→ ALL MATCH   [exit 0]
```

**非空转探针**（证工具真咬合，非恒真）：篡改 `analysis/traces/gs5-stg01-clear-anchor.frames.jsonl`
f2794 `vec438` 13→14 → 工具 **[FAIL] RAM flyout-vector f2794** + **[FAIL] F2 c2↔anchor mismatch=1**
⇒ **SUMMARY: 13/15 MISMATCH，exit 1**（双检查咬住）；还原后复跑 15/15 ALL MATCH。

## 4. mutation 电池（实测，全部还原）

| # | 篡改 | 预期 | 实测 | 判定 |
|---|---|---|---|---|
| M-a | 占位标记：stg03-shop trigger placeholder→natural（f=0 不动） | 锁红 | `s5_golden_parses_and_legality_gate` **FAILED** @l5_s5.rs:53 `[13] f=0 必须标注 placeholder（fail-closed）` | ✅ 咬合 |
| M-b' | 判定式：stg01-clear（golden:true 真锚）predicate 清空 | 锁红 | **FAILED** @l5_s5.rs:60 `[7] 真锚必须带判定式（可机器重导）` | ✅ 咬合 |
| M-b | 判定式：gate-26-bit4（**golden:false 附加锚**）predicate 清空 | （探针） | **4/4 全绿**——锁断言环只遍历 golden:true 条目（`ganchors = filter(|a| a.golden)`），6 条附加锚的存在性不在锁面 | ⚠️ 边界（§6） |
| M-c | 判定式文本：gate-26-bit4 判定式改回 #74 已证伪假主张（非空） | （探针） | **4/4 全绿**——predicate 为 presence-only 校验，文本语义不入锁 | ⚠️ 边界（§6） |

全部篡改 `git checkout` / 逐字节还原；`git status --short` 净；还原后 l5_s5 4/4 + 重导 15/15 复绿。

## 5. 纪律合规抽查

- **dated 更正纪律（AGENTS.md §3.6）**：8 处更正全部 `【2026-10-15 更正（复核 #74）】` 原地标注，
  历史主张一律删除线留档不删（findings §2.1/§2.3/§2.4/§3/§4/§5、anchors.jsonl、inject.txt、README）✓。
- **commit 规范**：`0b70a15` 含 Evidence + Confidence 字段；修复面 7 文件=文档/台账判定式/工具，
  **gates/ data/status/ 既有场景/阈值/边集零改动**（`git show --stat` 核过）✓。
- **不冒充自然通关**并列声明未回退：injected 锚 trigger 标注与 Run A 负结果并列仍在
  （findings §2.1/§4、spec R-l5-8/G-S5-6′、README、场景头、l5.rs 注）✓。
- 前审 #74 报告已入库归档（`docs/findings/adversarial-review-74.md`）✓。

## 6. 残余（advisory，不阻断）—— 前审 A1 同域，锁覆盖面精确定界

| # | 内容 | 实测 | 建议 |
|---|---|---|---|
| R1 | `l5_s5` 诚实锁的判定式断言（存在性）只覆盖 **golden:true 12 真锚**；6 条附加锚（golden:false，含本轮 F1 修复的 gate-26-bit4）的 predicate/source_run/trigger 存在性**不在锁面**（M-b 不咬） | M-b 绿 | 下轮与 A1 一并加锁：非 golden 锚的存在性约束 + 「source_run 场景含 poke ⇒ trigger=injected」归因断言 |
| R2 | predicate 为 **presence-only**：判定式文本内容（语义正确性）不被锁校验（M-c 不咬）——内容正确性现由 F3 重导工具（trace 事实重导）+ 复核人工层兜底 | M-c 绿 | 维持现状可接受；若要机器闭环，需判定式 DSL 化（成本高，不建议短期做） |

二者均属前审 #74 A1 advisory 范畴（「锁咬不住归因篡改 / 证据指针无机器校验」的精确化），
**不在本轮修复清单**（#74 §8：「A1 建议顺手加锁」；本轮任务书只列 A2/A3/F4）——故不阻断 verdict。

## 7. Verdict

**pass** —— 六项核验全部落地：

1. **F1**：dated 更正三要素（关末语义三联判据 + bit4 过渡窗 f913-1192 定界 + 「自然触发未观测」口径）
   在 findings §2.1/§5 与 anchors.jsonl gate-26-bit4 全部落地，「bit4 全程 0」裸露活主张全仓归零；
   事实面经 raw trace 独立复算逐项吻合。
2. **F2**：真口径三段（c1↔c2 f1..3200 / c2↔anchor 全程 / GP↔full 7 字段×2400f）落文，
   假主张删除线归零；三段数字独立复算精确一致（含 wram 基址反证）。
3. **F3**：重导工具入库可跑，独立重跑 **15/15**，非空转探针咬合（13/15 on tampered trace），
   §6① 复现命令入库。
4. **F5/F6**：README 日期 2026-10-15、inject 头注 dated 更正（预期链≠事实）落地。
5. **A2/A3/F4**：措辞收紧注在文（findings + spec:146）；F4 分级单一真相源裁定落地。
6. **mutation 复验**：占位标记 + golden 真锚判定式两类篡改锁均红（l5_s5.rs:53/:60），还原净。

对拍四件独立复跑全绿（**144/144 + 121/121 + gates 12/12 + l5_s5 4/4**）+ 重导 **15/15**；
门禁/阈值/边集/既有场景零改动；dated 更正纪律与「不冒充自然通关」并列声明无回退。
残余 R1/R2（A1 同域锁覆盖面边界）记 advisory，建议下轮顺手收口。
