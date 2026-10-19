# 对抗复核：P0-1 族①事件序回放（adversarial-review-event-replay）
> 2026-10-20 03:5x | verifier（证伪立场，实施方≠复核方）｜被审=工作树 `crates/game-desktop/src/bin/render_frame.rs`（实测 +480/−76）+ `docs/findings/p0-1-event-replay.md`。独立重导证据：diff 全读、cargo test、独立复渲（自建 binary）；未跑 `gates/cli.py check/status/visual`，无 git 操作。

## 0. VERDICT：**needs_revision（仅 F4 报告数字更正；实现与验证环=pass）**

- **实现面逐条证伪未破（A）**：`$0160` op=w 且 v≠0 ⇒ 置位时刻 `snapshot()` 入**单 pending 槽**（`set` 覆盖=多置位并入）；NMI 帧边界 `slot.consume()` 上传并清空（位置=旧 `pending_row.take()` 原位）；槽空不上传；`op=r` 早退不改状态；缺 op 旧 probe 走 `end_frame` 帧末快照回退（`legacy_file||frame_legacy` 谓词）+ stderr WARN 与 `replay_mode=legacy-opless-probe` 显式标注；MODELGEO 入 `ENV_KNOBS`；**无阈值/白名单/expected/gates 改动**（git status 仅 render_frame.rs+新 findings；diff 无阈值面）。
- **测试（A）**：`cargo test -p game-desktop` **61/61**（lib5+bin23[14 cli+9 新]+render14+render_rules14+smoke5）。9 新单测真测槽边界：多置位覆盖/空槽不上传/置位后填充不改槽/v=0 清零不置位/读隔离/缺 op 帧末回退/wrote 而无置位不补帧末/geo 档+knob 记录（缺 NMI 接线集成测与多置位 hud_seq 测，见 F1）。
- **独立复渲（A，gs5 f3450×2，target\debug\render-frame.exe=sha E523B1E5…）**：两跑 png sha256 全同 `245cbedc4398…`，`tools/diff_visual.compare` len(samples)=**1524/1524** ✓（=§4 表）。
- **追加复渲（A）**：gameplay f1260 diff=**5,202**、out=`51b7a201f021` ✓；stg03 f4600 diff=**58,159**、out=**`246dd5712608`**=p01 历史 s3.log sha ⇒ **"逐字节不变"成立** ✓。行值全对，唯总和错（F4）。

## 1. findings 四分类

### ① 规格/语义精度（实现层，均非阻塞）
- **F1（latent）**：`hud_seq` 在每次快照（HUD 回退）推进而非 NMI 消费推进；同帧多置位时槽只留末快照但 seq 已跳 ⇒ HUD LUT 行序可漂移。独立复算当前双流 probe **多置位帧=0**（gs5/gameplay `frames_multi_set=0`）⇒ 无现行影响；建议补单测并在槽覆盖时回退计数。
- **F2（info）**：`ptr_writes`($2F/$30)/`tile_page` 首遍扫描仍吞 `op=r` 读事件（render_frame.rs:1426 `_op` 弃置）——"op=r 隔离"仅覆盖 buf/attrs/regs/pal 镜像；与 before 行为一致（非回归），建议后续消歧。
- **F3（info）**：空 probe ⇒ `replay_mode=event-replay`（0 事件），建议单列 `no-probe` 口径。
### ② 数字/证据精度（报告层，需修）
- **F4（P1，阻塞入账）**：§0.4 总和与 §3 行值**不自洽**——按行复算 legacy 63,761→**64,885（+1,124）**（−831+1,955+0）、geo 64,100→**64,968（+868）**（−916+1,784+0）；报告写 +776/+968（疑用旧 gameplay 4,854/5,392 口径未随行值更新）。"净负"方向成立、**量值不实**，须按 dated 更正后方可入账。
- **F5（info）**：diff 行数实测 +480/−76（`git diff --numstat`），报告头部 +469/−72 不符（疑 CRLF/早期快照口径）。
### ③ 证据不足/其他（info）
- **F6**：MODELGEO=1 列（1,524/5,292/58,152）与三样本×2 跑确定性未全部复跑（按任务范围）；但 §4 gs5 geo sha=legacy sha 与 §3 行自洽、探针统计全部独立复现（见 §2）。

### ④ 越权/伪造
- **无**。双列口径清楚（§3 列定义：历史基线=旧读流 probe／before-current=改动前 binary×当前输入；gameplay/gs5 probe 已重采=双流、stg03 未重采=598,571 全缺 op——均经独立 python 复算确认）；§6-6 v=3 分型路由缺失标 **B/未证**、明示"不得据此回滚或改阈值"，§6-1/§6-4 残留如实披露，**无越证**。

## 2. 命令输出摘要

```text
cargo test -p game-desktop ⇒ 61 passed / 0 failed（含 event_replay_tests 9/9）
复渲（REPO_ROOT 显式，3450/1260/4600）× compare(exp, act) len(samples)：
  gs5 r1/r2 sha256=245cbedc4398…（两跑同） diff=1524/1524；日志 replay_mode=event-replay replay_sets=94 legacy_snap_frames=0 probe_opless=0
  gameplay diff=5202 out=51b7a201f021 replay_sets=70；stg03 diff=58159 out=246dd5712608（=p01 历史，逐字节同）
  stg03 日志 replay_mode=legacy-opless-probe legacy_snap_frames=1503 replay_sets=0 probe_opless=379865（与 §3 逐项一致）
python probe 复算：gs5 935678c325c9 w160079/r388116 sets=3×94+1×197；gameplay 7fd1643cfaaf sets=3×70；stg03 b71ee8fcbb84 598571 全缺 op
⚠ 任务给定 target/p0-0/build/debug/render-frame.exe（6204536D…）=**改动前 binary**：diff=2355、日志无 replay 字段——恰独立复现 before-current 2,355；after 口径请用 target\debug\render-frame.exe（=p0-1/build/render-frame-after.exe）
```

## 3. 复核边界
MODELGEO=1 列与其余两样本 2 跑确定性采信报告 §4（未复跑）；stg08/tas-* 旧 probe 回退仅代码+stg03 抽验；修复 F4 后可转 pass（实现不动）。


## 【2026-10-20 03:50 Lead 收尾注】
- **F4 resolved-at-source**：复核期间实施方终版已更正总和（Lead grep 全文：776/64,537 零命中，
  §3=+1,124/+868 与行值自洽）——按复核"仅需更正总和即可转 pass"的判据，**F4 条件满足**；
- F5 已随报告头更正（+480/−76）；实现代码复核全过（独立复渲三样本 1524/5202/58159 全中、
  61/61 测试、槽语义边界 9 单测真测、无阈值/白名单/越证）。
- **verdict 收口：pass**（F4 resolved + F5 修复；F1/F2/F3/F6 列后续卡）。
