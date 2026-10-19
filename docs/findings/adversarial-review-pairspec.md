# 对抗复核 — tools/l4_render_all.py PAIR_SPEC 9 行同步（task-166）

> 独立复审（verifier 角色，证伪立场；不采信实施方 platform-porter 自述）。
> 被审对象：工作树**未提交**改动 `tools/l4_render_all.py`（+89）/ `tools/test_l4_render_all.py`（+76）。
> 方法：git diff 逐行核对 9 行口径 → 自跑单测/`--validate` → 独立复算 expected blob sha 与 dedupe 根因
> → spec 自检 mutation 探针（非空转）→ 写入面 grep。
> 复核口径来源：Lead 本次裁定 5 条（见 §1 逐条）。
> 【2026-10-19 独立交叉确认】本文件由并发双复核之一先落盘；第二复核 agent 以独立路径重导全部关键事实
> （git diff 逐行、expected sha256 自算、`--validate` 实测、9/9 单测、mutation 三探针原样复跑吻合、
> 写入面 grep）——**结论一致，无一处修正**；补充 §2.5（trigger/tag 的 A 级 provenance）与 PF-6（提交卫生）。

## VERDICT: **needs_revision**

- ✅ 9 行中 **8 行（#16/#17/#18/#19/#20/#21 + ending-4/5）与口径完全一致**（frame/status/trigger/
  duplicate_of/expected_source 逐项吻合；四联 expected 同 blob `62a8ef06…` 独立复算成立）；
- ❌ **F1（P1）**：#14 `l4_stg02-start` scene/expected_source 与本次口径不符
  （实现=`gs5-stg01-clear-anchor`，口径=`gs5-anchor-render`）；且仓库内 Lead 裁定记录自相矛盾，需收口；
- ❌ **F2（P1）**：`--validate target/p0-0/render/g-manifest-full.json` 实测 **MANIFEST-CHECK: FAIL**
  （口径期望 PASS）——根因=task-165 expected 安装未落地的过渡态，非 9 行本身写错，但验收判据未达成；
- ⚠️ F3（P2）trigger 枚举单测覆盖不足；F4（P3）`--validate` 咬合面不含 PAIR_SPEC 对账。
- 单测 9/9 PASS、mapping.tsv 同步、无 expected/gates/threshold 写入路径 —— 均独立复核通过。

---

## 1. 九行口径逐条核验总表

| # | 口径（Lead 裁定） | 实测（git diff + 生成物独立核） | 判定 |
|---|---|---|---|
| 1 | #14 `l4_stg02-start`：scene=**`gs5-anchor-render`**、frame=3450、status=ok、trigger=injected、expected_source=`gs5-anchor-render:3450:stg02-start`（Option B 重指向 render-ready overlay） | frame=3450 ✓、status=ok ✓、trigger=injected ✓、reuse=independent ✓、duplicate_of 清空 ✓；**scene=`gs5-stg01-clear-anchor`** ✗、**expected_source=`gs5-stg01-clear-anchor:3450:stg02-start`** ✗（PAIR_SPEC L115、spec_issues L205、manifest、mapping.tsv、单测 L73/L223-224 五处一致地取后者） | ❌ **F1** |
| 2a | 四联主帧 `l4_stg06-late` status=ok | status=ok、reuse=independent、duplicate_of=""、frame=22000、expected_source=`stg08-timeline:22000:stg06-late` | ✅ |
| 2b | 其余 3 帧 documented-duplicate + duplicate_of=`l4_stg06-late` | `l4_stg08-start`(16600)/`l4_stg08-combat`(18000)/`l4_stg08-late`(28000) 均 `documented-duplicate` + `duplicate_of=l4_stg06-late` ✓；expected_source=`stg08-timeline:<frame>:<tag>` ✓（tag=stg08-start-approx/stg08-combat/stg08-late） | ✅ |
| 2c | trigger natural→injected 更正 | 四联 4 行 trigger 全 `injected`（diff L127/131/139/143/147 均由 natural 改） | ✅ |
| 2d | expected 同 blob `62a8ef06…` | 独立 sha256 实测：4 个 expected PNG 全 `62a8ef06c4b9…`（6074B）——与裁定同 blob 吻合 | ✅ |
| 3 | #17 `l4_stg07-start`/#18 `l4_stg07-combat`：status ok、trigger injected、expected_source=`stg08-timeline:14600/16000:<tag>` | 两行 status=ok ✓、trigger=injected ✓、frame 14600/16000 ✓、expected_source=`stg08-timeline:14600:stg07-start-approx`/`stg08-timeline:16000:stg07-combat` ✓ | ✅ |
| 4 | ending-4/5：expected_source tag=`tas-stg08b2-direct:34500/36000:probe`、trigger=assisted、documented-duplicate、duplicate_of ending-4→ending-5 | capture_tag=probe ✓（旧 l4f34500/l4f36000 已改）、expected_source=`tas-stg08b2-direct:34500:probe`/`…:36000:probe` ✓、trigger=assisted ✓、两行 documented-duplicate ✓、ending-4.duplicate_of=`l4_ending-5` ✓（ending-5.duplicate_of=""） | ✅ |
| 5a | mapping.tsv 生成同步 | `write_mapping(PAIR_SPEC, …)` 在 main 生成路径调用；expected_source 公式 `scene:frame:capture_tag` 与 build_manifest 同式；`analysis/repro/mapping.tsv` 实测已含全部 9 行新值 | ✅ |
| 5b | 单测覆盖 PAIR_SPEC 校验 / trigger 枚举（含 injected/assisted/unknown）/ #14 新定位 | 自跑 **9/9 PASS**；PAIR_SPEC 校验 ✓、#14 新定位 ✓（但锁的 scene 值随 F1 同错）；**trigger 枚举仅部分覆盖**（无显式枚举断言/无非法 trigger 负测试；assisted 仅集成分支断言、unknown 仅隐式） | ⚠️ **F3** |
| 5c | `--validate` 对 g-manifest-full.json 仍 PASS | **MANIFEST-CHECK: FAIL**（exit 1）——`dedupe 校验失败`，registry_ok=False | ❌ **F2** |
| 5d | 不动 expected 文件本体/gates/threshold | 两文件写入面仅 2 处：`out.write_text`（`--out`，默认 `target/...`）与 `write_mapping` 内 `path.write_text`（`--mapping-out`，默认 `analysis/repro/mapping.tsv`）；单测写入仅 tempfile；expected 仅 `sha256_file` 只读；grep `threshold/gates` 命中全为注释/参数名。git diff 未触 expected/gates/阈值文件 | ✅ |

## 2. 事实面独立重导（不采信自述）

### 2.1 #14 scene 名冲突的三面证据（F1）

| 面 | 实测 | 说明 |
|---|---|---|
| 本次复核口径 | `gs5-anchor-render:3450:stg02-start`（Option B：pair 重指向 **render-ready overlay**） | Lead 给本复核的裁定文本 |
| 实现面 | PAIR_SPEC/spec_issues/单测/manifest/mapping.tsv 全部 `gs5-stg01-clear-anchor:3450:stg02-start` | 五处一致，非笔误 |
| 仓库内 Lead 裁定记录 | `p0-0-expected-install-plan.md`：§7.7/表 D 写 "pair 重指向 `gs5-stg01-clear-anchor:3450`"，但 §2.1/表 G 的**重渲命令**又写 `render-frame 3450 l4_stg02-start gs5-anchor-render`（overlay 才有 probe/RAM blob 可渲） | **同一文档内两口径并存** |
| 场景资产 | `oracle/scenarios/gs5-anchor-render.txt` **不存在**（Test-Path=False）；`gs5-stg01-clear-anchor` 是 trace-only（无 probe.jsonl/RAM blob，见 install-plan §2.1 前置 blocker） | overlay 是待建 C4 场景 |

→ 不论最终采哪个名，**现状是两份 Lead 口径互相矛盾**（本次裁定=overlay 名；仓库 install-plan §7.7=trace 名）；
若以本次裁定为准，实现需连改 5 处并重生成 mapping.tsv/manifest；且 `gs5-anchor-render` 场景未建前
该 expected_source 指向不存在的场景。**请 Lead 收口单一真相源**后由 porter 一次性落地。

### 2.2 `--validate` FAIL 根因独立复算（F2）

```
$ python -X utf8 tools/l4_render_all.py --validate target/p0-0/render/g-manifest-full.json
MANIFEST-CHECK: FAIL
  - dedupe 校验失败 {'expected': '13 == 18 - 5', 'actual': '14 == 18 - 4', ...}
[exit code: 1]
```

独立重算 `build_dedupe(manifest.pairs)`：`equations_ok=True`、`documented_ok=True`、**`registry_ok=False`**。
根因链（自证，非引用实施方结论）：

1. expected 实测 `oracle/screens/expected/l4_stg02-start.png` = **`788d6984576b…`（258B）**，与
   `title-menu_f00180_boot-black.png` 同 blob ⇒ expected 重复组 `{title, l4_stg02-start}` 存在；
2. 新 PAIR_SPEC 已清空 #14 的 `duplicate_of` ⇒ mapping 组不再覆盖该重复组 ⇒ registry_ok=False；
3. task-165 的安装（`44e7800d…` 1411B）**未落地**（expected 文件本体未动——符合本卡"不动 expected"约束，
   但同时使 registry 无法成立）。

补充事实：`target/p0-0/render/g-manifest-full.json` 是**用新 PAIR_SPEC 重新生成过的**（其 pairs 行已含
scene=gs5-stg01-clear-anchor/tag=probe 等新值），且生成时 `validation.ok=false` 已写盘——即产物是在
**明知 FAIL 的过渡态**下落盘的。实现方单测对此有自适应分支（`test_build_manifest_integration` 的 else
断言 `validation.ok is False`），测试全绿，但**验收口径"--validate 仍 PASS"不成立**。
定性：**任务排序/验收判据问题**（task-166 同步先于 task-165 安装），不是 9 行字段本身写错。

### 2.3 spec 自检非空转探针（mutation，内存内改表，未写文件）

| 探针 | 预期 | 实测 |
|---|---|---|
| `l4_stg06-late.trigger='natural'` | 自检咬住 | ✅ `l4_stg06-late.trigger 应='injected'（task-166），实际 'natural'` |
| `l4_ending-4.trigger='manual'`（枚举外） | 自检咬住 | ✅ `trigger 非法 'manual'`（ALLOWED_TRIGGER={natural,injected,assisted,unknown}） |
| `l4_stg02-start.scene='gs5-anchor-render'` | — | ⚠️ 自检**反向锁死** `gs5-stg01-clear-anchor`（`应='gs5-stg01-clear-anchor'`）——若 Lead 收口采 overlay 名，此检查会**阻断正确修复**，须同步改 |

### 2.4 生成物/资产独立核

- expected 独立 sha256：四联 4 件全 `62a8ef06c4b9…` ✓；ending-4/5 同 `6ff345dc5c3f…` ✓（documented-duplicate 理由成立）；
- `analysis/repro/mapping.tsv` 实测 9 行新值全部在位（含 `tas-stg08b2-direct:34500:probe`、
  `stg08-timeline:22000:stg06-late` 等）；duplicate_of 非空行=4（四联 3 孩子+ending-4）✓；
- manifest summary 实测：`rendered=18 / blocked=4 / replaced=0 / documented_duplicate=5 /
  two_run 18/18`，与新计数口径（13 ok+5 dup）一致 ✓。

### 2.5 trigger/expected_source 口径的 A 级 provenance（第二复核补充）

- **trigger natural→injected 更正依据**成立（A 级）：`oracle/scenarios/stg08-timeline.txt` 双 poke 实存
  ——L33 `poke 2400 0026 10`、L53-54 `poke 4600 0026 10`+`poke 4600 0400 01`、L64-65/L75-76/L86-87/
  L97-98/L108-109 同型；头注 L7"注入=双 poke（$26 bit4 + $0400）" ⇒ 该场景产出帧 trigger 必须 injected。
- **新 capture_tag 与真实 shot tag 逐字对齐**（A 级）：同文件 L139-145 `capture 22000 stg06-late` /
  `14600 stg07-start-approx` / `16000 stg07-combat` / `16600 stg08-start-approx` / `18000 stg08-combat` /
  `28000 stg08-late` 全部命中；旧 `l4f22000` 等 tag 场景中不存在（install-plan L53 同述）。
- **#14 来源字符串可复现**（A 级）：`oracle/scenarios/gs5-stg01-clear-anchor.txt:64` `capture 3450
  stg02-start`；`oracle/traces/s5-all-clear.anchors.jsonl:12` 记 f3450 锚，evidence 指
  `shots/gs5-stg01-clear-anchor_f03450_stg02-start.png`（44e7800d/1411B）——但该场景名与本次口径的
  overlay 名冲突（F1/PF-1）。
- **ending tag=probe 依据**（A 级）：`oracle/scenarios/tas-stg08b2-direct.txt:37` `capture_every 1500
  probe`（34500/36000 均为 1500 倍数 ⇒ shot tag=probe，旧 `l4f34500` 不存在）；L27 `assist-mode
  stage-select-fallback` + L28-31 不死三址 poke ⇒ trigger=assisted 有据。

## 3. findings（四分类）

### 阻塞（P0）
- 无（未发现伪造证据、expected/gates/threshold 写入、阈值松动、ROM/基准触碰）。

### 重要（P1）
- **PF-1 #14 scene/expected_source 与本次口径不符**：实现取 `gs5-stg01-clear-anchor:3450:stg02-start`，
  本次裁定取 `gs5-anchor-render:3450:stg02-start`（render-ready overlay）；且 `p0-0-expected-install-plan.md`
  内部两口径并存（§7.7=trace 名 vs §2.1 重渲命令=overlay 名），`gs5-anchor-render` 场景文件尚未创建。
  **路由：Lead 收口单一真相源** →（若采 overlay 名）porter 连改 5 处（PAIR_SPEC.scene、spec_issues L205
  抽检、单测 L73/L223-224、重新生成 mapping.tsv + manifest），并把 install-plan §7.7 与本次裁定对齐。
- **PF-2 `--validate` 口径判据未达成**：`MANIFEST-CHECK: FAIL`（registry_ok=False，过渡态根因见 §2.2）；
  `g-manifest-full.json` 在 validation.ok=false 下落盘。**路由：Lead 定排序**（建议二选一：
  ① task-165 安装 `44e7800d…` 后重新生成 manifest + `--validate` 复跑转 PASS 再收卡；
  ② 本卡撤销对 `g-manifest-full.json`/mapping.tsv 的重写、待安装后一并生成），porter 执行。

### 次要（P2）
- **PF-3 trigger 枚举单测覆盖不足**：口径要求"单测覆盖 trigger 枚举（含 injected/assisted/unknown）"，
  实测仅 injected 有直接断言；assisted 只在**依赖 target 输入在场**的集成测试里断言（输入缺失即 SKIP）；
  unknown 仅随 blocked 行隐式过 `spec_issues`；无 `ALLOWED_TRIGGER` 枚举断言、无非法 trigger 负测试
  （mutation 探针证明校验逻辑本身有效，但回归网未锁）。**路由：porter 补 2 个测试断言**。

### 建议（P3）
- **PF-4 `--validate` 咬合面缺口**：`validate_manifest` 不核 pairs↔PAIR_SPEC 一致性
  （scene/frame/capture_tag/trigger/expected_source/status）也不核新 counts 口径（replaced=0/dup=5/ok=13）
  ⇒ PASS/FAIL 均不能证明 9 行同步（本次 FAIL 只在 dedupe 层咬合）。建议 `--validate` 增加
  "manifest 行 vs PAIR_SPEC 对账"与 counts 口径检查。
- **PF-5 措辞小疵**：`spec_issues` 注释"5 documented-duplicate（ending2+四联3）"中"ending2"应为
  "ending-4/5 两行"，防误读。
- **PF-6 提交卫生（第二复核补充）**：工作区并存大量非本卡未提交改动（`crates/game-shell/*` 约 +950 行、
  `docs/spec/platform-shell.md`、`docs/findings/plan-enemy-render.md` 等，见 `git diff --name-only` 9 文件）
  ——本审只覆盖 tools 两文件；按"小步提交：一个 commit 一个逻辑变更"纪律，task-166 两文件须单独 commit
  （commit message 含 Evidence/Confidence），勿与 game-shell 改动混提。

## 4. 执行命令输出摘要

```text
$ git diff --stat -- tools/
  tools/l4_render_all.py       | 89 +++--
  tools/test_l4_render_all.py  | 76 +++--

$ python -X utf8 tools/test_l4_render_all.py
  PASS test_build_manifest_integration_if_inputs_present   （过渡态分支：validation.ok=False 自适应断言通过）
  PASS test_dedupe_equations_and_documented_groups
  PASS test_parse_render_log_and_strict_line
  PASS test_registry_ok_rejects_unmapped_duplicates
  PASS test_spec_22_rows_schema_and_counts
  PASS test_strict_from_args_records_only
  PASS test_validate_cli_roundtrip
  PASS test_validate_manifest_accepts_good_and_rejects_schema_breaks
  PASS test_write_mapping_22_rows_with_expected_source
  l4_render_all 回归: 9/9 通过

$ python -X utf8 tools/l4_render_all.py --validate target/p0-0/render/g-manifest-full.json
  MANIFEST-CHECK: FAIL
    - dedupe 校验失败 {'expected': '13 == 18 - 5', 'actual': '14 == 18 - 4', ...}
  [exit code: 1]        ← 口径期望 PASS；根因 registry_ok=False（expected 788d6984 未安装，见 §2.2）

$ 独立 sha256（expected 侧）
  l4_stg02-start.png 788d6984576b 258B   ← 44e7800d 安装未落地（task-165 待办）
  l4_stg06-late/start/combat/late.png 62a8ef06c4b9 6074B ×4   ← 与口径"同 blob 62a8ef06"吻合
  l4_ending-4/5.png 6ff345dc5c3f 4154B ×2
  oracle/scenarios/gs5-anchor-render.txt → 不存在

$ spec_issues mutation 探针（内存改表）
  trigger='natural' → 咬住 ✓ | trigger='manual' → 咬住 ✓ | scene='gs5-anchor-render' → 反向锁死 ✗（PF-1）

$ grep 写入面
  l4_render_all.py 仅 2 处 write_text（--out 默认 target/、--mapping-out 默认 analysis/repro/）
  单测写入仅 tempfile；expected 仅只读 sha256_file；无 gates/threshold 写入路径 ✓
```

## 5. 修正建议（按路由）

1. **Lead**：收口 #14 scene 单一真相源（`gs5-anchor-render` overlay vs `gs5-stg01-clear-anchor` trace），
   并同步更正 `p0-0-expected-install-plan.md` §7.7/表 D 与本口径的矛盾处（dated 注）。
2. **Lead/porter**：定 task-165↔166 顺序——expected 安装（`44e7800d…`）落地后重新生成
   `g-manifest-full.json` + `mapping.tsv` 并复跑 `--validate` 期望 PASS；在此之前不应把
   validation.ok=false 的 manifest 当交付物。
3. **porter**：若采 overlay 名，连改 5 处 + 解除 spec_issues 对旧名的锁；补 trigger 枚举显式断言
   （ALLOWED_TRIGGER 四值 + 非法值负测试 + assisted/unknown 直接断言，勿依赖集成 SKIP 分支）。
4. **porter（建议）**：`validate_manifest` 增加 pairs↔PAIR_SPEC 对账与 counts 口径检查（PF-4）。
5. **提交时（PF-6）**：task-166 两文件单独 commit（Evidence: 本复核 §2.5 A 级重导；Confidence 建议 85），
   与工作区中 game-shell/文档改动分卡提交。


## 点验追加（2026-10-19，Lead 收口 + F1/F3 修复）

- **F1（P1）已收口**：Lead 单点裁定 **#14 pair = \gs5-anchor-render:3450\**（render-ready overlay）；
  已连改 5 处（l4_render_all.py PAIR_SPEC L115/注释 L112/L119、spec_issues L205、
  test_l4_render_all.py L73/L223/L224 的 scene/expected_source）；install-plan 15 处 + s1-source 2 处
  命名统一；**历史 shot 磁盘实名保留**（shots/gs5-stg01-clear-anchor_f03450_stg02-start.png，复现命令按实名）；
  install-plan 头部加 dated 收口注。
- **F3（P2）已修复**：新增 test_trigger_enum_and_negative（枚举显式断言 + injected/assisted/unknown 覆盖
  + 非法 trigger 负测试）；负测试暴露 validate_manifest 原**不拒非法 trigger**（真实缺口）⇒ 已补校验
  （只紧不松）；回归 **10/10 通过**。
- **F2（P1）= 排序依赖**：--validate 转绿依赖 task-165 expected 安装（44e7800d）落地，已写入安装流程
  step 0（安装→重生成→复跑转绿）；在此之前该 manifest 为待装态（target/ gitignored）。
- PF-4/5（P3）记入后续卡。
