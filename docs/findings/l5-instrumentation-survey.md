# L5 真机验收仪器 · 勘察记录（四点现状）——接线前只读盘点

> 日期：2026-10-15 22:12｜角色：test-runner（tools/oracle 面）｜回合因基建抖动缩小：本轮**仅勘察落盘**，接线（diff_l5 扩展/真机跑通/s5 对照）下回合做。
> 背景：引擎侧帧探针桥已落（StageProbe/stage_probe）；本仪器 = Stage A DoD 最后一跃的验收面。

## 1. StageProbe 输出面（已落，engine.rs:849-895）

`StageProbe` 逐帧状态向量（named 模式对齐 oracle golden JSONL 口径）：
`f: u32`｜`tick: u16`（$84 滚动域）｜`scroll: u16`（px）｜`flags_26: u8`（bit4=关末门）｜`waves: u8`（已装波次流数）｜`script_recs: u8`（已发脚本记录）｜`sched_recs: u8`（已发调度记录）｜`slot_types: [u8; 28]`（$0400 槽型码逐槽向量）｜`live_shots: u16`（$040E state≠0 计数）。
抽取 = 纯函数 `stage_probe(&StageState) -> StageProbe`（零副作用，可逐帧泵出）。

## 2. diff_l5 现状（**未建**——扩展目标）

tools/ 现有 diff 家族：`diff_subroutines.py`（L2）/`diff_traces.py`（L3）/`diff_visual.py`（L4）/`s5_anchor_rederive.py`——**无 diff_l5**。
接线目标（下回合）：`tools/diff_l5.py` = StageProbe 序列 vs `oracle/traces/*.l5.jsonl` golden 对拍面
（场景/帧号/命名字段逐条；命名映射：StageProbe.f↔golden.f、flags_26 bit4↔checkpoint 关末门、slot_types↔face 型码向量）。

## 3. golden JSONL 口径（oracle/traces/，两族）

- **L5 流程级 golden**（`*.l5.jsonl`）：行 = `{"scene", "f", "mode", "checkpoint"}`——
  现有 `stg01-chain.l5.jsonl`（title-menu 场景 boot-black f180 / title-in f360…）与 `s5-all-clear.l5.jsonl`。
- **S5 锚台账**（`s5-all-clear.anchors.jsonl`）：行 = `{"id", "golden", "checkpoint", "mode", "scene", "f",
  "trigger", "source_run", "predicate", "evidence"}`——trigger 分级（natural/injected）+ 证据指针
  （evidence=analysis/traces/<run>.frames.jsonl#fNNN，指针纪律照旧）。
- 逐 run face 轨（同目录 `*.enemy.jsonl/*.shots.jsonl/*.flow.jsonl/*.player.jsonl/*.scroll.jsonl` 等）=
  named 逐帧 JSONL，StageProbe 的 slot_types/live_shots/tick 等字段与之对齐。

## 4. s5 三场景对照面（现状）

L5 测试族（cargo 内）：`l5_chain.rs`（stg01-chain golden）+ `l5_s5.rs`（s5-all-clear golden + anchors ledger +
三场景口径 `l5_scope_note_3scenes_10events`——3 场景 10 事件范围注）。工具面 `s5_anchor_rederive.py`（锚复推）。
对照面缺口：三场景（title-menu 等）**真机 trace → golden JSONL 的机器对拍**尚无独立驱动（现由测试内嵌断言承担）——
即 diff_l5 要补的第二面。

## 5. 下回合接线顺序（防丢）

1. `tools/diff_l5.py`：StageProbe JSONL（真机/引擎双源）↔ `*.l5.jsonl` golden 逐条对拍 + anchors ledger trigger 校验；
2. tas_bot/Mesen2 真机跑通：八关逐关自然跑场景（oracle/scenarios/ 新 DSL 面，纯按键无 poke）→ StageProbe 输出落
   `analysis/traces/<stage>.probe5.jsonl`；
3. s5 golden 三场景对照：真机三场景 run ↔ s5-all-clear.l5.jsonl + anchors.jsonl（natural trigger 判定走写点非几何）。
   红线照旧：golden 只升不降、不为绿放宽比对、trigger 分级诚实（injected 不冒充 natural）。
