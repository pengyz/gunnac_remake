# 对抗复核：manifest 22 行全字段 + registry_ok wrapper（task-155/159）

> 日期：2026-10-18｜复核方：platform-verifier（verifier，证伪立场）｜被审：platform-porter task-155
> 被审：`tools/l4_render_all.py`（32,305B，untracked）+ `tools/test_l4_render_all.py`（9 单测）+
> 手册 §5.1/§7.5 注；产物 `target/p0-0/render/g-manifest-full.json`、`analysis/repro/mapping.tsv`。
> 复核方式：自跑 9 单测 + `--validate` + 独立读 manifest/mapping/字段语义；**未跑 Mesen/gates**；
> 未改源码/expected/threshold。本文件唯一写入。

## 0. VERDICT

**pass**（1 条 P3 旧手册片段、1 条 P3 字段口径；工具/测试/数据全过）

- 自跑 `python -X utf8 tools/test_l4_render_all.py` → **9/9 通过**；
  `python -X utf8 tools/l4_render_all.py --validate target/p0-0/render/g-manifest-full.json` →
  **MANIFEST-CHECK: PASS**（pairs22/rendered18/blocked4/replaced5；
  `DEDUPE: expected 13=18-5 actual 14=18-4 registry_ok=True`）。
- manifest 22 行全字段齐、blocked 4 空语义/trigger unknown、replaced 5 expected 旧 sha+notes、
  ending-4/5 documented-duplicate、expected_matches=0/18：独立读 JSON 逐项成立。
- `mapping.tsv` = **23 行（header+22）× 9 列**；duplicate_of 链
  stg02→title、四联孩子→stg06-late、ending-4→ending-5；registry_ok 反例测试有效。
- `--strict-line/--strict-log` 只采集 `STRICT:` 行，不执行 diff_visual/不写 oracle；单测覆盖。
- 未发现工具写 `oracle/`/expected/threshold 的路径；只有 target manifest + analysis/repro mapping。

## 1. 自跑

### 1.1 单测（9/9）
```
PASS test_build_manifest_integration_if_inputs_present
PASS test_dedupe_equations_and_documented_groups
PASS test_parse_render_log_and_strict_line
PASS test_registry_ok_rejects_unmapped_duplicates
PASS test_spec_22_rows_schema_and_counts
PASS test_strict_from_args_records_only
PASS test_validate_cli_roundtrip
PASS test_validate_manifest_accepts_good_and_rejects_schema_breaks
PASS test_write_mapping_22_rows_with_expected_source
l4_render_all 回归: 9/9 通过
```

### 1.2 `--validate g-manifest-full.json`
```
MANIFEST-CHECK: PASS
  pairs=22 rendered=18 blocked=4 replaced=5
  DEDUPE: expected 13=18-5 actual 14=18-4 registry_ok=True
```
退出码 0 ✅。

## 2. 22 行全字段/manifest 语义（独立读）

- `pairs=22`；summary：
  `rendered=18, blocked_no_source=4, replaced_invalid_asset=5, documented_duplicate=2,
   two_run_deterministic=18, expected_matches=0,
   trigger_breakdown={natural:10, injected:6, assisted:2}`。
- 必填字段逐行检查 **无缺失**：
  `name/scene/frame/capture_tag/frame_offset/trigger/expected_source/input_sha256/render_env/
   render_argv/expected_sha256/actual_sha256/actual_run1_sha256/actual_run2_sha256/status/reuse/
   duplicate_of/deterministic/notes` + 顶层 `env/summary/dedupe/strict/validation/inputs/canonical`。
- **blocked 4 对**：`l4_ending-1/2/3/fade-clear`：
  `expected_sha256=""`、`actual_sha256=""`、run1/run2 空、`trigger=unknown`、`deterministic=false`；
  未发明 expected/trigger ✅。
- **replaced 5 对**：`l4_stg02-start`（expected `788d6984…`）+ 四联
  `l4_stg06-late/l4_stg08-combat/l4_stg08-late/l4_stg08-start`（expected `62a8ef06…`）；
  每行 `notes=expected-invalid 待 §7.2 重采（expected_sha256=旧件 sha，A 级证据）` ✅。
- **ending-4/5**：`status=documented-duplicate`，`trigger=assisted`，expected `6ff345dc…`/actual
  `5aa8a4f8…`，`duplicate_of` ending-4→ending-5；summary `documented_duplicate=2` ✅。
- **expected_matches=0**：独立计算 18 rendered 中 expected_sha256==actual_sha256 的行数 = **0/18**；
  报告/manifest 未宣称 L4 通过 ✅。
- **inputs/render_env/render_argv**：inputs 含 render_dir/g_manifest_v2_sha256/run dirs/expected dir/
  render binary sha；render_env 为空对象（canonical 39 unset），argv 每对保留；无伪造 ✅。

## 3. 去重 registry

`dedupe` 实读：
```
expected_present=18 expected_unique=13 expected_reduction=5
actual_present=18  actual_unique=14  actual_reduction=4
equations: expected 13=18-5 / actual 14=18-4 /
           expected_22 13=22-4-5 / actual_22 14=22-4-4 / blocked_not_counted=4
documented_duplicates 3 组:
  expected+actual: l4_ending-5,l4_ending-4（6ff345/5aa8a4）
  expected: title-menu_f00180_boot-black,l4_stg02-start（788d6984）
  expected+actual: l4_stg06-late,l4_stg08-combat,l4_stg08-late,l4_stg08-start（62a8ef/406f39）
registry_ok=True documented_ok=True equations_ok=True
```
- `mapping.tsv` 23 行（header+22）9 列；duplicate_of 链独立 parse：
  - `l4_stg02-start → title-menu_f00180_boot-black`；
  - 四联 3 个孩子 → `l4_stg06-late`，master 自身空；
  - `l4_ending-4 → l4_ending-5`，ending-5 空。
- **registry_ok 反例有效**：单测
  `test_registry_ok_rejects_unmapped_duplicates` 把四联孩子 duplicate_of 清空 ⇒
  `registry_ok=False` 且 `validate_manifest` 报 dedupe 失败；9/9 已通过 ✅。

## 4. strict/只读/手动注

- 代码/测试：`--strict-line`/`--strict-log` 只解析 `STRICT:` 行并回填 manifest；不调用
  `diff_visual.py`、不写 `oracle/screens/`。单测 `test_strict_from_args_records_only`、
  `test_parse_render_log_and_strict_line` 通过 ✅。
- 工具写路径只有 `target/p0-0/render/g-manifest-full.json`、`analysis/repro/mapping.tsv`（任务允许的
  报告产物）；未见写 `oracle/`、expected、threshold。
- 手册 §5.1/§7.5 注：task-155 段（L1119/L1523-1534）与实现/9 单测/去重公式 13/14 一致；
  docs-links 只读门 PASS ✅。

## 5. findings（四分类）

### 阻塞（P0）
- 无。

### 重要（P1）
- 无。

### 次要（P2）
- 无（工具/数据/registry/测试均通过）。

### 建议（P3）
- **F159-1**：手册 §5.3 的**旧参考代码片段**仍保留 `len(exp) in (21,22)`/旧 dedupe 逻辑（L1249-1251），
  与 task-155 B2 后实际 13/14 公式并存；虽然后文 L1523-1534 已注更新，建议顺手在旧片段加
  “已由 task-155 取代”或删除，避免读者照抄。
- **F159-2**：`summary.documented_duplicate=2` 是 **status 行数**（ending-4/5），而
  `dedupe.documented_duplicates=3 组`；两者口径不同，报告/字段最好显式注明“2 行 / 3 组”。
- **F159-3**：`mapping.tsv` 23 行=header+22 行；“23 行”与“22 对”需在文档区分，否则易误读为 23 对。

## 6. 命令/证据摘要

```text
python -X utf8 tools/test_l4_render_all.py → 9/9 通过（含 registry 反例/strict/validate/mapping）
python -X utf8 tools/l4_render_all.py --validate target/p0-0/render/g-manifest-full.json
  → MANIFEST-CHECK: PASS; pairs22 rendered18 blocked4 replaced5;
    DEDUPE expected13=18-5 actual14=18-4 registry_ok=True
manifest: 22 rows, expected_matches=0/18, blocked4 empty/unknown, replaced5 old sha+notes,
          ending4/5 documented-duplicate, 3 dedupe groups
mapping.tsv: 23 lines (header+22) × 9 cols；duplicate_of 链齐全
strict: STRICT 行只采集，不写 oracle/expected
docs-links PASS；oracle/screens 0；未跑 Mesen/gates；未改源码/expected/threshold
```

## 7. 结论

**pass**：22 行全字段/blocked/replaced/documented-duplicate/expected_matches 语义正确；去重
13/14、3 组、registry_ok 反例、9/9 单测、--validate PASS、mapping 23×9 均独立通过；strict 只读采集。
F159-1..3 为 P3 文档/字段口径建议，不阻塞入库工具+测试+手册注。
