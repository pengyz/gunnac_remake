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

## t45 发现（弹轨迹 L3 暂缓）：弹对象槽位几何未证【t50 更新：几何已由 t48 定谳，但归因受阻于生成器零活动——见下节】

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

## t50 发现（shots golden 转正受阻）：六型生成器在 gameplay-stage1 全程零活动

槽位几何/分配器已定谳（t48，20 槽 $80-$84/$9C-$A0/$B8-$BC/$D4-$D8）且 L2 四用例三方锚定，
但 trace 侧**真弹对象归因不成立**（t50 实测，2400 帧逐帧）：
- `R-shot-gen-3` 五字段同帧写入链（寿命=1/坐标=玩家/弹型 0..5）**0 次命中**；
- 六型计数 `$072E,X` **全程恒 0**（periods=[5,0,…]=级 0 配置 ✓ 但计数从未 INC）⇒ 生成器循环体未执行；
- 20 槽 state 值为**静态非寿命数据**（20 帧不递减，如 115/133 恒值）——t48 §3"弹数 churn 并发最大 9"
  解读**被反证**（同址值非弹对象寿命）；槽地址与 $1C 网格他系统交错（B5 假设域）。

**路由 decomp-analyst**：六型生成器的**调用方门控**（`$0401` mode/场景状态？）与计数器复位例程取证；
在生成器活动可观测的场景/窗口重采后，`--shots-extract` 实验入口（20 槽+n=4 采样+出生帧全向量，
口径已声明）一键转正。机器侧（tick 重放/首次适应断言）已就绪并有单测（28/28）。

## t54 F3 结案：场景重跑时长异常 = 孤儿 Mesen 穿透串行锁

**根因（实测复现）**：串行锁（`analysis/.scenario.lock`）只罩**驱动进程**——驱动崩溃/被强杀后
Mesen/Xvfb 成孤儿继续跑，与后续运行**争显示/CPU 并写同一批产物路径**。复现链：驱动崩溃
（计时代码 TypeError 于 Mesen 启动后）→ 孤儿 Mesen 幸存 → 下一 run trace 交错（1214/2400 行、
无 DONE、lua.log 双 started 行）；t50 的 >25min 爬行同族（同期多实例/orphan 干扰）。

**修复（t54）**：①驱动崩溃安全——观测/基线设置全部移到 Mesen 启动前；②启动清场——持锁后
pgrep 清孤儿 Mesen（此时存活者必为孤儿，不影响他人持锁运行）；③时长基线机制——run.json 记
`first_sample_s`（启动→首采样帧），超历史中位 3 倍显式告警 + 等待循环每 60s 无进展即打印
`[时长告警]`（防静默挂起）。修复后干净重跑 **51.2s/2400 帧 DONE**（历史 51.7s 同量级）；
`first_sample_s=2.1s`（基线 ~1.3-2.1s，3 倍阈值+15s 地板）。
