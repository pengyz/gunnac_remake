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

## t45 发现（弹轨迹 L3 暂缓）：弹对象槽位几何未证

> 【2026-10-12 更正（t51）】本节"槽位几何未证"**已由 t48 定谳撤销**（`docs/findings/shot-slot-geometry.md`）：
> 几何 = 4 组×5 槽（$80-$84/$9C-$A0/$B8-$BC/$D4-$D8），组内 stride=1、组距 $1C；18 例矛盾值 = stride-1
> 跨 $18 组间空隙**伪读**；分配器 `bank2:$888D` 首次适应/空槽=state==0/回收=寿命归零/满载 SEC。
> L2 shot-alloc-1..4 三方全等；`--shots-extract` 转正前提=t50 golden（R-shot-alloc-* 见 shot-system.md §4.5）。
> 下文 t45 原文保留（证伪更正文化）。

`gameplay-stage1` 已采 `shot_objs` watch（$0400-$06FF，captain 批准），但**槽位索引几何未证**：
t32/t29 的"$70 间距"只锚定六个字段基址（$040E/$047E/$04EE/$055E/$05CE/$063E），槽位步长/
容量/分配器（`JSR $888D`）语义属 B5/A5 未证域——按 stride-1 解析 trace 会跨系统串扰（$1C 网格
敌/关卡对象区同址），五字段同帧写入签名仅 18 例且含矛盾值（槽位越出字段容量）。**未证不发明**：
`tools/diff_traces.py --shots-extract` 保留实验入口，默认抽取链不产出 shots golden；
L3 弹轨迹对拍的机器侧（tick 重放 = 脚本机+LUT，L2 74/73 已证片段组合）已就绪并有自证单测。
**路由**：decomp-analyst 取证槽位几何/分配器（$888D）后即可启用抽取入 golden。
