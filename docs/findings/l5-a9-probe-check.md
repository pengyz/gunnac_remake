# L5 A9 落地记录：probe5 发射 + diff_l5 --probe 本机对拍（task-72）

> 日期：2026-10-18｜角色：decomp-analyst（L5 A 档）｜状态：**已落地并实跑通过（本机，零 Mesen2）**
> 任务：task-72（源自 task-39 §3.1 A9 / task-51 复核确认）——把"引擎侧 `emit_probe5` 发射 + `diff_l5 --probe`
> 对拍"固化为仓库内可复现工具与记录。
> **点改（task-76，task-74 复核 E1-E5）**：文档实跑记录按**真实输出**逐字修正（`None`/injected 行尾后缀/
> emit 结果行）、命令 `python3`→`python`（本机 Store 别名不可用，E3）；工具增 target 防护（拒 `REPO`/
> `REPO/target`，E4）与 emit 新鲜度断言（`1 passed`，E5）；逐条落点见 §8。
> 写范围：`tools/l5_probe_check.py`（新增）、`docs/findings/l5-a9-probe-check.md`（本文件）；**不改** `diff_l5`
> 判定阈值/门禁/基线；不 git add/commit。产物写 `analysis/traces/`（.gitignore 工作区，不入库）。
> 证据分级依 [AGENTS.md](../../AGENTS.md) §3；口径 = [l5-plan.md](../spec/l5-plan.md) §2 +
> [l5-all-clear.md](../spec/l5-all-clear.md) R-l5-1..8（golden/真闸/诚实锁）。

## 0. 结论（先读）

1. **A9 链路已固化并本机实跑通过**：`python tools/l5_probe_check.py` → **exit 0**，输出
   **`L5-DIFF natural 7/7`**；发射 **12 件** probe5 产物；`diff_l5` 返回 **rc=1（预期）**。
2. **对拍计数（本次实跑）**：
   - passA（自然面，gameplay-stage1.probe5 + scene-map）：`natural_ok=7` + `ok-injected=4` +
     `unknown=33` + `mismatch=0`；
   - passB（injected 链段信息面，s5-chain.probe5）：`ok-injected=27` + `unknown=17` + `mismatch=0`。
3. **`diff_l5` 的 exit=1 不作为验收**：其 bad 定义含 `unknown`（本链路只覆盖 natural 面 + 4 条 ≤f3840 的
   injected 锚；其余 s5-chain/占位行必然 unknown）⇒ 验收改为 **natural 命中数 == `--expect-natural`（默认 7）
   且 mismatch==0**（脚本据此给 0/1 退出码）。
4. **零 Mesen2**：probe5 由 `crates/game-core/tests/engine_tick.rs` 的 `#[ignore] fn emit_probe5()` 生成
   （引擎侧 StageProbe；不读 ROM/不入库产物）；**真机 run 面**（B5）仍需 Linux+Mesen2（见 §4 局限）。
5. **不改判定/门禁/基线**：仅新增一个 wrapper 脚本 + 本记录；`diff_l5.py`/`gates/`/`data/status/` 零改动。

## 1. 工具：`tools/l5_probe_check.py`

```bash
python tools/l5_probe_check.py                 # 发射 + 对拍（验收 natural 7）
python tools/l5_probe_check.py --skip-emit     # 复用 analysis/traces 已有产物（缺=SKIP）
python tools/l5_probe_check.py --selftest      # 只验证解析器/仓库文件（不需要 traces/cargo）
python tools/l5_probe_check.py --expect-natural 7 --target-dir target/l5probe --timeout 1800
```

> **命令解释器（E3）**：本仓库 Windows 执行机**无可用 `python3`**（Store 别名不可用）——本文命令一律用
> `python`；Linux/Mesen2 环境用 `python3`（同一脚本，无平台分支）。

**封装链路**（与 task-39 §3.1 A9 一致）：

```bash
mkdir -p analysis/traces
CARGO_TARGET_DIR=target/l5probe cargo test -p game-core --test engine_tick -- --ignored emit_probe5
python tools/diff_l5.py --probe analysis/traces/gameplay-stage1.probe5.jsonl \
  --golden oracle/traces/s5-all-clear.l5.jsonl --anchors oracle/traces/s5-all-clear.anchors.jsonl \
  --scene-map title-menu=s5-title,gameplay-stage1=s5-stg01
```

**设施纪律（E4 加固）**：`CARGO_TARGET_DIR` 固定在 **`REPO/target/` 之下且 ≠ `REPO/target`**
（默认 `target/l5probe`）；`--target-dir` 若等于仓库根（`.`）或共享 target（`target`）→ **ERROR exit 3**
（防污染共享 target/仓库根）；落在 `REPO/target/` 之外同样 ERROR——**禁止共享 target / 外部 worktree target**。

**新鲜度（E5）**：emit 成功判定 = `cargo` rc==0 **∧ 输出含 `1 passed` ∧ 12 件产物齐**（0 tests rc=0 + 旧产物
不构成 false PASS）；成功时工具打印该结果行（见 §2.1）。

**退出码语义**：

| exit | 含义 | 触发 |
|---|---|---|
| 0 | PASS | `natural == --expect-natural` 且 `mismatch==0` |
| 1 | FAIL | natural 不足或出现 mismatch（判定阈值不放宽） |
| 2 | SKIP（非失败） | 无 cargo / `emit_probe5` 未产出 12 件 / `--skip-emit` 时产物缺失——**显式打印 SKIP 原因，不静默通过** |
| 3 | ERROR | 仓库文件缺失 / `--target-dir` 越界 / `diff_l5` 异常退出 |

## 2. 实跑记录（2026-10-18，HEAD `3d8b557`，Windows + Python 3.12.1 + cargo）

### 2.1 发射（12 件产物）

工具真实 stdout（**E2**：成功路径也打印 emitter 结果行 `test result: ok. 1 passed ...`；末行 `artifacts`）：

```
emit: CARGO_TARGET_DIR=target\l5probe cargo test -p game-core --test engine_tick -- --ignored emit_probe5
emit: | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.37s
artifacts: 12/12 present in analysis/traces
```

> `finished in` 耗时随构建缓存/机器状态变化（0.2-0.4s 量级），不影响判定；`1 passed` 是 E5 新鲜度断言的锚点。

| 产物 | 字节 |
|---|---:|
| `stg01.probe5.jsonl` … `stg08.probe5.jsonl`（8 件，每件 ~588-590KB） | 4,714,197（合计） |
| `gameplay-stage1.probe5.jsonl` | 1,090,744 |
| `respawn-window.probe5.jsonl` | 501,208 |
| `s5-chain.probe5.jsonl` | 6,931 |
| `clear-run.trace.jsonl` | 2,440 |

> 发射器（`engine_tick.rs:655-843`）**不建目录**：须先 `mkdir -p analysis/traces`（脚本已封装）；
> 产物写 `../../analysis/traces/*`（cargo 集成测试 cwd=package root ⇒ 仓库 `analysis/traces/`）。

### 2.2 passA（自然面，验收口径）

```
passA: natural_ok=7 (expect 7) ok-injected=4 unknown=33 mismatch=0 provenance-note=0 | diff_l5 rc=1 (1=expected)
L5-DIFF natural 7/7
```

`diff_l5` 原始逐行（**真实输出，UTF-8 规范文本；Windows GBK 控制台直显中文可能乱码，不影响判定**）：

```
[ok          ] s5-title f=180 Black boot-black 
[ok          ] s5-title f=360 Title title-in 
[ok          ] s5-title f=540 Menu menu-in 
[ok          ] s5-stg01 f=1080 Shop stg01-shop 
[ok          ] s5-stg01 f=1200 Transition None 
[ok          ] s5-stg01 f=1260 StageStart stg01-start 
[ok          ] s5-stg01 f=1500 Combat stg01-combat 
[ok-injected ] s5-stg01 f=2401 StageClear stg01-clear trigger=injected 不计 natural 通过
[ok-injected ] s5-stg01 f=3116 Shop stg02-shop trigger=injected 不计 natural 通过
[ok-injected ] s5-stg01 f=3400 Transition None trigger=injected 不计 natural 通过
[ok-injected ] s5-stg01 f=3450 StageStart stg02-start trigger=injected 不计 natural 通过
```

> 其后 33 行 `[unknown     ] ... golden 无 probe 覆盖`（1 条 f3900 + 32 条 s5-chain，见 §3）此处省略；
> `Transition` 行 checkpoint **实际是 `None`**（`diff_l5.py:120` 直接打印 `g.get('checkpoint')`，非 `(null)`）；
> `ok-injected` 行尾带 `trigger=injected 不计 natural 通过` 后缀（`diff_l5.py:73`）。

原始汇总行（`diff_l5.py:124` 打印；真实输出）：`L5-DIFF[s5-title]: 3/3 natural 通过（unknown/ok-injected 不计）`、
`L5-DIFF[s5-stg01]: 4/9 natural 通过（unknown/ok-injected 不计）`、
`L5-DIFF[s5-chain]: 0/32 natural 通过（unknown/ok-injected 不计）`。

### 2.3 passB（injected 链段，信息面，不参与验收）

```
passB: ok-injected=27 unknown=17 mismatch=0 | diff_l5 rc=1
```

### 2.4 自检/边界路径（实跑）

| 场景 | 命令 | 实际输出（真实） | exit |
|---|---|---|---|
| 解析器自检（不需要 traces/cargo） | `--selftest` | `SELFTEST: ok (repo files present + status parser 4/4)` | 0 |
| 无产物且不发射 | 临时移走 `analysis/traces` 后 `--skip-emit` | `SKIP: analysis/traces missing 12 artifacts: [...]` + `(run emit first: python tools/l5_probe_check.py; ...)`（**不建空目录**，Info） | **2** |
| 阈值不满足 | `--expect-natural 8` | `L5-DIFF natural 7/8` + `A9: FAIL (natural 7 != 8 or mismatch>0; see above)` | **1** |
| target=共享 target（E4） | `--target-dir target` | `ERROR: --target-dir must not be repo root or shared target (got target; use e.g. target/l5probe)` | **3** |
| target=仓库根（E4） | `--target-dir .` | 同上（`got .`） | **3** |
| target=仓库外 | `--target-dir C:/Temp/l5probe_bad` | `ERROR: --target-dir must be under repo target/ (got C:/Temp/l5probe_bad; shared/external target forbidden)` | **3** |

> passB 若出现 `mismatch>0`，工具打印 `WARN: passB mismatch=N（信息面；不进退出码，但须记录/追查）`（Info）。

## 3. 计数口径（为什么是 7/7、4、33/17）

- **natural 7** = 锚台账 `trigger=natural` 的 7 条（`boot-black/title-in/menu-in` 3 + `stg01-shop/stg01-exit/
  stg01-start/stg01-combat` 4），全在 `title-menu` + `gameplay-stage1` 前缀域 ⇒ passA 全 `ok`；
- **ok-injected 4** = `s5-stg01` 中 f≤3840 的 4 条注入锚（stg01-clear f2401 / stg02-shop f3116 /
  Transition f3400 / stg02-start f3450）——匹配但**不计 natural**（trigger=injected）；
- **passA unknown 33** = f3900 `stg02-combat`（超 `gameplay-stage1` 发射窗 0x0F00=3840，见
  `engine_tick.rs:730`）+ 32 条 `s5-chain` 行（passA 未喂 s5-chain probe）；属预期 unknown；
- **passB 27 ok-injected** = `s5-chain` 27 条真锚（injected 链）；**未知 17** = 5 条 f=0 占位（placeholder，
  OBS-1 族，本链路不发明帧号）+ 12 条非 `s5-chain` 行（`s5-title` 3 + `s5-stg01` 9，passB 未喂对应 probe）；
- 两 pass 均 `mismatch=0`：**已覆盖字段逐字段相等**（不得用"看起来对"替代；判定仍由 `diff_l5` 负责）。

## 4. 局限与边界（如实）

1. **probe5 仅 natural/engine 面**：`emit_probe5` 由引擎侧 StageProbe 从 stage scripts / flow timeline /
   锚台账生成（合成数据），**不是实机 trace**；真机 run 面（Mesen2 场景跑 StageProbe）仍属 **B5**，需
   Linux+Mesen2 环境（task-39 §3.2 B5）。
2. **5 占位/OBS-1 不覆盖**：s5-chain f=0 五行保持 unknown（fail-closed）；不得在 A9 内回填（§3.6 组 D 重裁/
   §3.7 四联来源在 L4 P0-0 环境窗口处理）。
3. **不改判定**：本工具只封装与解析，`diff_l5.py` 的阈值/`unknown` 计入 bad 语义、`gates/`、`data/status/`
   基线均未改；不新增门禁项（L5 门禁显式 SKIP 仍属 task-39 A2 建议卡）。
4. **产物不入库**：`analysis/traces/*` 与 `target/l5probe/` 均 .gitignore 工作区；本记录只固化命令与判据。

## 5. 复现

```bash
cd <repo-root>
python tools/l5_probe_check.py --selftest      # 解析器自检（无依赖）
python tools/l5_probe_check.py                 # 发射+对拍：期望 exit 0 且 L5-DIFF natural 7/7
python tools/l5_probe_check.py --skip-emit     # 复用已有产物（缺则 SKIP exit 2）
# 单独看 diff 原始输出（exit=1 预期）：
python tools/diff_l5.py --probe analysis/traces/gameplay-stage1.probe5.jsonl \
  --golden oracle/traces/s5-all-clear.l5.jsonl --anchors oracle/traces/s5-all-clear.anchors.jsonl \
  --scene-map title-menu=s5-title,gameplay-stage1=s5-stg01
```

## 6. 证据指针

| 事实 | 指针 |
|---|---|
| probe5 发射器（12 产物、不建目录） | `crates/game-core/tests/engine_tick.rs:655-843`（`#[ignore] fn emit_probe5`：655-669；stg01-08:685-720；gameplay-stage1:721-753；respawn-window:754-783；s5-chain:787-826；clear-run:827-842） |
| 对拍工具语义（golden 驱动投影/unknown/injected 单列/rc=1 含 unknown） | `tools/diff_l5.py:40-78`（`diff()`）、`:81-129`（CLI/结果输出/`return 1 if bad`） |
| 39/5 与 honest lock（golden 44 行=39 真锚+5 占位） | `crates/game-oracle/tests/l5_s5.rs:68` |
| 7 natural / 4 injected 锚来源 | `oracle/traces/s5-all-clear.anchors.jsonl`（trigger 字段） |
| A9 卡来源 | `docs/findings/l5-gap-inventory.md` §3.1 A9（本仓库已入库） |

## 7. 验收自查

- [x] 本机实跑 `python tools/l5_probe_check.py` **exit 0**，输出 `L5-DIFF natural 7/7`（§2.2 真实输出）；
- [x] 12 件产物齐、`--selftest` ok；SKIP/FAIL/ERROR 四类退出码路径实跑可核（§2.4，含 E4 的 `target`/`.` 拒绝）；
- [x] `diff_l5`/`gates/`/基线零改动；产物 gitignored；未 git add/commit（工作树新增 1 工具 + 本记录）；
- [ ] docs-links PASS（本次点改后重跑）；
- [ ] 独立对抗复核 pass（task-76 复审卡；此前 task-74 为 needs_revision，已按 E1-E5 收口）。

## 8. task-74 复核修正记录（E1-E5，task-76）

| # | 级 | 落点 | 修正 |
|---|---|---|---|
| **E1** | 必改 | §2.2 | 样本改**真实输出**：`Transition None`（非 `(null)`）；`ok-injected` 行尾补 `trigger=injected 不计 natural 通过`；汇总行补 `（unknown/ok-injected 不计）` |
| **E2** | 必改 | §2.1 + 工具 | 工具成功路径打印 emitter 结果行（`emit: | test result: ok. 1 passed; ...`）；§2.1 用该真实输出（不再手写 "running 1 test..."） |
| **E3** | 必改 | §0/§1/§5/§7 | 命令 `python3`→`python`，并注明本机 Store 别名限制（Linux 用 `python3`） |
| **E4** | Low（已改码） | §1 + 工具 | `--target-dir` 必须位于 `REPO/target/` 下且 ≠ `REPO/target`；`target`/`.`/仓库外 → ERROR exit 3（实跑见 §2.4） |
| **E5** | Low（已改码） | §1 + 工具 | emit 成功判定加新鲜度：rc==0 ∧ 输出含 `1 passed` ∧ 12 产物齐（0 tests+旧产物不 false PASS） |
| Info | — | 工具/文档 | passB mismatch → `WARN`（不进退出码）；SKIP 不再先建空目录；§7 复选框措辞明确 |
