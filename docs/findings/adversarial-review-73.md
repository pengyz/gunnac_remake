# 对抗复核 #73 — DEV-3 修复效果终验（第二轮轻量）

> 独立复审（fresh context，证伪立场）。被审对象：commit `95f28ec`
> （fix(test)：F1 负向断言特异化 + F3 边界注 + 复核 #72 归档）——即 #72 判 needs_revision
> 后的**唯一必修项 F1** 的修复落点。
> 方法：不采信 commit 自述/不复用 #72 结论；四项逐 grep 独立核；**mutation 复验独立改码实测**
> （含 #72 M5 原样重放 + 错误串改名第二变体）；三件对拍独立复跑贴数；每项贴实测输出。
>
> **Verdict: pass**（四项全落地；mutation 复验双向咬合，F1 缺口实证闭合）。

---

## 0. 对拍独立复跑（贴数，未采信自述）

| 复跑项 | 命令 | 实测输出 | 判定 |
|---|---|---|---|
| workspace 测试 | `cargo test --workspace`（逐 result 求和） | **144 passed / 0 failed**（37 个 test target：4+10+4+7+7+17+5+5+12+4+6+6+3+8+5+14+3+12+6+4+2=144） | ✅ 与任务书④一致 |
| L2 子程序级 | `python3 tools/diff_subroutines.py` | **L2-DIFF: 121/121 通过（golden==ROM 执行==game-core 同名函数，三者缺一不可）** | ✅ |
| 门禁总闸 | `python3 gates/cli.py check` | **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0**；`CHECK: PASS（含 0 个静默跳过：所有 SKIP 均已显式打印）` | ✅ |
| S5 判定测试 | `cargo test -p game-oracle --test l5_s5` | **4 passed / 0 failed** | ✅ |
| 预检 status | `python3 gates/cli.py status` | ratchet 7 项全 OK（rust_test_count 144/144）；`STATUS: OK（可动手；先看清 SKIP 项）` | ✅ |

---

## 1. 四项核验表（逐项独立 grep/实测）

| # | 核验项 | 核验物 | 判定 | 证据位 |
|---|---|---|---|---|
| 1 | 负向② 断言特异化（`.contains("未证模式")`，非 "fail-closed"） | `crates/game-oracle/tests/l5_s5.rs` L107 逐行 | **过** | §2 |
| 2 | **mutation 复验（F1 咬合力）**：改掉实现 fail-closed ⇒ 负向② 红 ⇒ 还原 tree 干净 | M5-replay + M-a 双变异独立实测 | **过（双向咬合）** | §3 |
| 3 | `docs/spec/l5-all-clear.md` "S5-ALL-CLEAR 链可跑通"带 claim-boundary 注（状态机层/G-S5-6） | 验收② 行 L187 逐字 | **过** | §4 |
| 4 | 复跑三件不降（144 / 121 / 12） | §0 独立复跑 | **过** | §0 |

---

## 2. 核验 1：负向② 断言特异化 —— 过

`crates/game-oracle/tests/l5_s5.rs` L102-108 逐行（grep 直读）：

```rust
102:    // 负向②：结局段伪造为未证模式（GAME OVER 动态锚仍缺 ⇒ golden 禁入）
103:    let mut bad2 = g.clone();
104:    let last = bad2.len() - 1;
105:    bad2[last].mode = "GameOverMenu".into();
106:    assert!(
107:        l5_legality(&bad2).iter().any(|p| p.contains("未证模式")),
108:        "未证模式必 fail-closed"
```

- **断言串 = `"未证模式"`**（L107），非 `"fail-closed"` ✓ —— 与 #72 F1 修正建议逐字一致。
- **特异性独立核**：`l5_legality` 仅 3 类 problem 串（`grep -n "problems.push" crates/game-oracle/src/l5.rs`）：
  ①`帧序倒退（前 f…）`（L64）、②`未证模式 {other}（fail-closed，不发明）`（L86）、
  ③`非法模式序列 {prev}→{cur}（t81 已证边集外，fail-closed）`（L94-95）。
  **"未证模式" 子串全仓唯一出现于 ②**（`grep -rn "未证模式" --include="*.rs" crates/`：
  实现侧仅 L5.rs:86 一处产串；测试侧仅 L5_s5.rs:107/108 引用）⇒ 断言**不可能**被 ①/③ 误满足。
- **对照旧断言的缺口面**：③ 含 "fail-closed" 子串，旧断言 `.contains("fail-closed")` 会被
  边违规串误满足（#72 M5 根因）；新断言与 ③ 无交集 ⇒ 缺口闭合。

## 3. 核验 2：mutation 复验（F1 咬合力）—— 过（双向咬合，#72 M5 由绿转红）

全程独立改码实测：`cp` 备份 + `md5sum` 锚定（`ea0a4fa4270763b0e3a72e05c08caa74`）→ 改 → 跑 → 看 FAILED →
`git checkout --` 还原 → 校验 md5 回到锚定值 + `git status --short` 空。

### M5-replay（**#72 M5 原样重放**：关闭未证模式 fail-closed，`other => Boot`）

改 `crates/game-oracle/src/l5.rs` L84-88 分支为 `other => { let _ = other; Mode::Boot }`（即
**删掉未证模式拒入**，复刻 #72 的 M5 变异）。实测：

```
running 4 tests
test s5_chain_runs_and_diffs_golden ... ok
test s5_golden_parses_and_legality_gate ... ok
test s5_all_clear_judgement_is_state_derived ... ok
test s5_legality_gate_rejects_invented_joints ... FAILED

thread 's5_legality_gate_rejects_invented_joints' panicked at crates/game-oracle/tests/l5_s5.rs:106:5:
未证模式必 fail-closed

test result: FAILED. 3 passed; 1 failed; 0 filtered out
```

⇒ **同一变异在 #72 是「全 4 passed（不咬合）」，本轮为 `s5_legality_gate_rejects_invented_joints`
FAILED（负向② 咬合）**。F1 缺口（M5 不咬合）**实证闭合**——这是本复核的核心判据。

### M-a（错误串改名：`未证模式` → `unproven-mode`，fail-closed 分支本身保留）

只改 L5.rs:86 报错串前缀，**"（fail-closed，不发明）" 尾串原样保留**。实测：

```
test s5_legality_gate_rejects_invented_joints ... FAILED
panicked at crates/game-oracle/tests/l5_s5.rs:106:5: 未证模式必 fail-closed
test result: FAILED. 3 passed; 1 failed
```

⇒ 断言**咬的是 "未证模式" 这一特异串**，不是 "fail-closed" 泛串（旧断言在此变异下会**仍绿**，
因尾串仍含 "fail-closed"）——特异化不是形式主义，是**真咬合面转移**。

### 还原核验（tree 干净）

```
$ git checkout -- crates/game-oracle/src/l5.rs
$ git status --short                # 空
$ md5sum crates/game-oracle/src/l5.rs
ea0a4fa4270763b0e3a72e05c08caa74  crates/game-oracle/src/l5.rs        # = 备份锚定值 = HEAD
$ cargo test -p game-oracle --test l5_s5
test result: ok. 4 passed; 0 failed
```

三件字节等 HEAD 复核（`git show HEAD:<path> | md5sum` 对拍）：

| 文件 | HEAD md5 | 实测 md5 | 一致 |
|---|---|---|---|
| `crates/game-oracle/tests/l5_s5.rs` | `c2cabe221c6ee90efc86c2ebcff73277` | `c2cabe221c6ee90efc86c2ebcff73277` | ✓（复核全程未触碰） |
| `crates/game-oracle/src/l5.rs` | `ea0a4fa4270763b0e3a72e05c08caa74` | `ea0a4fa4270763b0e3a72e05c08caa74` | ✓（变异已还原） |
| `docs/spec/l5-all-clear.md` | `d8478e621d6d83df6837857b1b3c9d0b` | `d8478e621d6d83df6837857b1b3c9d0b` | ✓（复核全程未触碰） |

## 4. 核验 3：claim-boundary 注 —— 过

`docs/spec/l5-all-clear.md` L187（验收② 行，grep 逐字）：

```
| ② | L5 golden 扩八关 + S5-ALL-CLEAR 链可跑通（**状态机层**——G-S5-6 未跑动态回放，
    8 关真机可通关口径见复核 #72 达成度分层） | `oracle/traces/s5-all-clear.l5.jsonl`
    （八关全程 4 检查点/关）+ `s5_chain_run` 端到端跑通（R-l5-8） | ✅ |
```

- **"状态机层"** ✓ + **"G-S5-6 未跑动态回放"** ✓ 双要素齐（任务书③括注两点均落地），
  并显式区分"链可跑通"与"8 关真机可通关"两个口径 ⇒ #72 F3（advisory）落地，误读面消除。
- **一致性核**：边界注所引 G-S5-6 与 §3 缺口登记 L160 同源（"8 关全程 oracle 动态 trace（帧号真值）
  │ 未跑…golden f=0 占位"）；所引"复核 #72 达成度分层"= `docs/findings/adversarial-review-72.md`
  §4.3（六层裁定表，同 commit 归档入库）⇒ 引用可解析、口径自洽。
- **全文件扫描**：`grep -n "跑通\|端到端\|可通关"` 仅命中 L187 一处（含边界注）⇒ 无第二处
  无边界裸主张；§1 段表/§6 自查未见拔高措辞。

## 5. 非问题 + advisory（均不阻断）

| ID | 级 | 描述 | 裁定 |
|---|---|---|---|
| A1 | advisory | `crates/game-core/tests/stage_content.rs:180` `use game_core::stage::{stage_skeleton, StageSkeleton};` 编译告警 `unused import: StageSkeleton` | **非本次修复引入**（95f28ec 仅触 3 文件，未及此）；非门禁项（gates 12/12 绿），可择机顺手清 |
| A2 | advisory | `l5-all-clear.md` §6 自查第 2 条把"拾取次数/`$018C` 显式占位常量"也挂 G-S5-6，与 §3 表 G-S5-6 主旨（8 关动态 trace/帧号占位）措辞略宽 | **既有行**（#72 未列为问题）；同为"golden 占位"族，语义可容，不构成拔高 |

**非问题（本轮证伪尝试均失败的潜在质疑）**：
- "特异化只是换个词、仍可被误满足"——M-a（尾串保留 `fail-closed`）证明旧断言会绿、新断言红 ⇒ 咬合面真转移；
- "变异后没真还原"——三文件 md5 对拍 HEAD 全等 + `git status --short` 空 + 复跑 4/4 绿；
- "对拍数是抄的"——§0 三件均本轮独立实跑，输出原文粘贴；
- "边界注只是应付措辞、口径仍可误读为可通关"——注内显式"状态机层"+"G-S5-6 未跑动态回放"+
  "8 关真机可通关口径见 #72"三段，两个口径已切开。

## 6. 纪律自查

- **证伪立场**：四项均独立重导（grep 逐行/mutation 独立改码/复跑贴数），未采信 commit 自述；
  未复用 #72 结论（mutation 为本轮自改自测，含 #72 未做的 M-a 变体）。
- **实施≠审核**：本轮全程只读 + 临时变异（已还原，md5 对拍）；未改被审实现/测试/spec 一字。
- **完成 ≠ 声称**：所有"过"均带实测输出原文（§0/§3/§4），无未运行声称。
- **mutation 无残留**：两次变异均 `git checkout --` 还原 + md5/`git status` 双核，tree 干净。

---

## 7. Verdict

**pass**（四项全落地，F1 缺口实证闭合）。

- 核验 1 ✅：负向② 断言 `.contains("未证模式")` 特异化（L107），全仓唯一产串点锁定（L5.rs:86）。
- 核验 2 ✅：**#72 M5 原样重放由绿转红**（`s5_legality_gate_rejects_invented_joints` FAILED@L106）
  + M-a 错误串改名亦红 ⇒ F1 咬合力双向实证；还原后 tree 干净、4/4 复绿、三文件 md5 等 HEAD。
- 核验 3 ✅：验收②"S5-ALL-CLEAR 链可跑通"已带 claim-boundary 注（状态机层 / G-S5-6 未跑动态回放
  / 真机口径见 #72 §4.3），全文件无第二处裸主张。
- 核验 4 ✅：cargo **144/144** + l2-diff **121/121** + gates **12/12**（SKIP=0 TOLERATED=0 显式打印），
  与 95f28ec 自述一致、与基线 ratchet 144 无回退。

**遗留（不阻断，随下次触碰顺带）**：A1 unused import 告警（stage_content.rs:180）；
A2 G-S5-6 引用措辞略宽（既有）。**Stage A 口径维持 #72 §4.3 建议**：S5 记"机制链已落地（A 级字节背书）"，
动态可通关层挂 G-S5-6 + STG-02..08 内容待办，勿以 `s5_chain_run` 跑通充抵真机可通关。
