# oracle/traces/ —— L3 帧级 golden 状态向量

`<场景>.player.jsonl`：逐帧 `{"f","held","x","y","sx","sy}`（player_x/y + 亚像素，标签=命名台账 A 级名）。
期望 = Mesen2 oracle 执行；实际 = game-core::player tick（`tools/diff_traces.py` → `l3-diff`）。

- 来源：`tools/scenario_run.py run gameplay-stage1`（确定性：两次重跑 sha256=3eb1273b635758a7 一致）
  → `tools/diff_traces.py --extract` 抽取 f1260+；
- 对齐约定：`state(N) = tick(state(N-1), held(N))`（940 帧实证 0 失配）；
- **剔除窗口**（死亡/重生事件=spec §4 未证域『判定点/受击盒』，禁止发明）：f1831-1916、
  f2047-2101、f2131-2175（特征=移动冻结+弹药回满+坐标重置 128/160）；帧号缺口=分段边界，
  差分端按段重播种。相关遗留：W5『up 段后 Y 回中』观测即落此窗口，已定性为死亡/重生。

改 golden = 改证据轨道：单独 commit + Evidence/Confidence（AGENTS.md §9）。
