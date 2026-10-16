# tas_bot 八关输入面规划（通关令·二线：真机 bot 通关 + OBS-1 裁定）

> 日期：2026-10-16 08:30｜角色：test-runner（tools/oracle 面）｜状态：**规划稿**（通关令指示先行备规划，执行待 lead 放行）
> 上游：通关令·验收仪器面——一线=port Rust 全通跑（trace→diff_l5 扩 run 面+通关判据）；二线=本规划（真机 bot 八关真实输入全程，**同时裁 OBS-1**）。

## 1. 目标与裁决面

1. **真机 bot 八关全程通关**（纯输入驱动：bot 只读 RAM→按键，录制→开环轨回放可复现）。
2. **OBS-1 裁定**（5 行注因区核心疑点）：末关出店截断（Transition 半途）= **注入链伪影**（injected 注入点截断）or **真身**（原版行为）——bot 自然通关轨出店实测即裁。
3. 通关判据面喂料：结局到达（$95F9 入口/$26:=$80）/ 三结局分支观测 / 逐关段锚（shop→start→combat→clear 8×4）供 diff_l5 扩 run 面全程对拍。

## 2. 输入面架构（三级，照 tas_bot 既有链）

```
① bot 决策（tools/tas_bot.lua，只读 RAM→按键，无 poke 无条件指令）
   → ② 逐帧录制 analysis/traces/<run>.inputs.jsonl
   → ③ tas_gen record-to-scenario 转写**纯 press/hold 开环轨**（oracle/scenarios/<run>.txt）
```
- 复用面：`tools/tas_bot.py`（load_trace/compare_traces/build_replay）+ `scenario_run.py`（gen lua 归档）+ `tas_gen` 转写。
- 每关一个 run 命名：`tas-clear-stg01` … `tas-clear-stg08`（跨关拼接=通关全程）。

## 3. 逐关段推进轨（四段 × 8 关，段锚驱动）

| 段 | 推进触发（输入面） | 段锚（验证点=golden 段锚同名） |
|---|---|---|
| 商店段 | 菜单导航（方向+A 选购/双 B 出店——R-shop-7 双 B→近黑过渡） | `stgN-shop` |
| 出店过渡 | 自动（近黑 f≈出店−34 两段同式） | Transition（无名） |
| 关始段 | 自动（「地区-N」+目录行索引 $180） | `stgN-start` |
| 战斗段 | **战斗输入轨**（§4）+关末 BOSS 击破 | `stgN-combat` → `stgN-clear` |

- 段锚帧=probe5 已收数段（STG-02..07 25/25 全等）直接作**中程校验点**；stg08 段=OBS-1 采样区（§5）。
- Wt-1 红线照旧：跨关换算常数禁止——每关独立关数参数（stage_no→$180 目录行）。

## 4. 战斗输入轨（每关战斗段）

- **复用优先**：W-tas 既有轨（tas-alldemo/alldemo2/W-tas20）战斗段按键流可整段摘取为该关保底轨；tas-arena 系（b2/复现轨）火力模式复用。
  > 【2026-10-16 dated 更正（port 实测）】三轨全仓/analysis/tools/docs **零命中=轨不存在**（历史 W-tas 期产物未入库）——
  > 本条"整段摘取"前提失效。既有实轨素材=tas-stg01-v2/v3（→tick 824 中 boss 竞技场）+tas-arena-repro 系（arena 段）
  > +tas-stg01-bot1-replay；**stg01 中后段（tick 824→关门）无轨**。轨源受阻待裁（外部取轨 or 方向③ bot 战斗面扩）。
- bot 决策面（tas_bot.lua 既有：游走/火力/保命）+ **boss 段扩展**：BOSS 血量 RAM 面→输出循环（未取证 RAM 挂账——按 §3 规则现用 dodge+hold 火力通解，取到 boss 血址再精化）。
- 战斗输入轨验收：段内存活（player_state≠死亡态）+ 段末达成 `stgN-clear` 锚。

## 5. OBS-1 采样设计（末关出店截断裁定）

- **采样点**：stg08 出店双 B 输入帧起 → 近黑过渡全程 → `stg08-shop`→Transition 边界后至 `stg08-start` 出现（或截断复现）。
- **判定式**（写点/事件非几何）：
  - `Transition` 帧推进至 `stg08-start` 锚 ⇒ **真身**（注入链伪影否定，OBS-1 5 行按自然帧回填）；
  - 出店后停滞/半途截断**自然复现** ⇒ **真身截断**（原版行为，5 行按真身锚回填+B 级标注）；
  - 自然轨正常过、仅注入 run 截断 ⇒ **注入链伪影**（5 行注因保留、改 OBS 标注不入行为断言）。
- 采样产物：probe5 续行（21→26 行）+ 出店段 capture 帧组（shots/）。

## 6. 与 diff_l5 扩 run 面对接

- 通关 trace（port Rust 全通跑 + 本 bot 通关轨）→ `oracle/traces/*.l5.jsonl` 全程对拍（golden 驱动投影现成）。
- 新增消费：三结局分支行（golden 待 port/spec 面补行后即对拍）+ 逐关段锚 32 行（现 25 已收数，stg08 段+OBS-1 5 行待本跑补）。
- 预期账面推进：13 unknown → 关始段占位 7 行另议（引擎相对帧域，本跑不裁）；OBS-1 5 行按 §5 三分支裁定入账；3900 行待 gameplay probe 域扩展。

## 7. 执行顺序（待 lead 放行）

1. tas_bot.lua boss 段扩展 + 八关参数化（stage_no 推进表）——**工具面改动我做**；
2. 逐关试跑（stg01→stg08，每关通过段锚四连再进下一关）；
3. OBS-1 采样（stg08 出店段专项）；
4. 全程拼接轨 + 开环回放复现验证（inputs.jsonl→scenario→re-run 段锚全等）；
5. diff_l5 扩 run 面消费对拍 → 贴账（含 OBS-1 裁定结论）。

## 8. 风险与回退

- 真机非确定性：录制轨开环回放可复现性先验（build_replay 双跑段锚全等）；漂移⇒bot 重录。
- 计时敏感段（出店双 B 窗）：W-tas 逐帧按键精度面已证（7285f/12852f 轨）。
- 中途失败：每关独立 run 颗粒度，失败关重录不重跑全程。
- OBS-1 三分支任一结论均显式入账（不预设真身/伪影）。
