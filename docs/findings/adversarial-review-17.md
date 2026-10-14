# 对抗复审 #11（t174）：t172 L5 开跑（链 golden + 检查点机制）复审

> 被审方：t172（46d30e8，`l5.rs`/`l5_chain.rs`/`stg01-chain.l5.jsonl`/flow.rs）。审核方：verifier（t174），
> 独立重跑链对拍 + 边集逐边核对 + 源码级桩检查。
> **裁定：needs_revision**——②③④⑤主体成立（golden 逐项一致、9 边全落 t81 已证边集、fail-closed 合规）；
> 但 **R14-1：l5_legality 的"模式序列过 FlowState 已证边集"是未实现的桩**（`let _ = m;` 丢弃模式）——
> "合法性 fail-closed"声称超出实现，非法已知模式序列（如 Black→Combat）可通过。

## 0. 发现总表

| ID | 级 | 问题 | 处理 |
|---|---|---|---|
| R14-1 | **中** | `l5_legality` 边集校验是桩：注释称"模式序列过 FlowState 已证边集"，代码为 `let _ = m; let _ = FlowState::default();`——实际只查帧序单调+未知模式拒绝；**已知模式的非法序列（Black→Combat、Menu→DeathRespawn 等）不被拒**，与"合法性 fail-closed"声称不符 | 补边集校验（复用 `FlowState::advance`）+ 非法序列负向用例 |
| R14-2 | 低 | "4 场景接力 10 检查点"口径不严：golden 实为 **3 个 scene 字段 / 10 事件 / 8 命名检查点 + 2 空**（shop 是模式非场景；10 是事件数） | 口径注 |
| R14-3 | 低 | 负向自证缺 **帧序倒退** 用例（单调性代码在、无负向测）；边违规负向随 R14-1 补 | 补 2 条负向 |
| R14-4 | 低 | L5 产物未显式引用 t81/t84 任务 id（链条本体已验 ✓，指针仅到"FlowState 已证边集"/l5-plan） | 引用注 |

## 1. ①L5 基建复核 —— 复用一致 ✓ / 合法性声称过宽（R14-1）

| 项 | 实现 | 判定 |
|---|---|---|
| 0 容差 diff（帧号/模式/检查点全等，事件数不等即差） | `diff_l5` ✓ | 与 L3 逐 tick 全等口径一致 ✓ |
| A1 流程可达 + A3 事件窗口 | 检查点字段（8 命名）+ 模式推进 ✓ | ✓ |
| 帧序单调 | `l5_legality` ✓（场景内 f 倒退即违规） | ✓ |
| 未知模式 fail-closed | ✓（"未证模式 …fail-closed，不发明"） | ✓ |
| **已证边集校验** | **桩**（R14-1） | ✗ |

## 2. ②链 golden 复核 —— ✓ 逐项一致

- `stg01-chain.l5.jsonl` 10 事件 vs `l5_chain.rs` 内联向量 **逐项一致** ✓（scene/f/mode/checkpoint 全等）；
- `cargo test --workspace` **96/96 通过** ✓（独立复跑，含 l5_chain 4 测试）；
- 4 测试覆盖：合法性/未证 fail-closed 负向/diff 三向负向/解析往返 ✓。

## 3. ③STG-01 全程链引用链核 —— ✓ 9 边全落已证边集

golden 模式序列逐边对 t81（`game-core/flow.rs` 边表）核对：

| 链 | 转移 | t81 边 |
|---|---|---|
| title-menu | Black→Title→Menu | `(Boot,Black)→Black`/`(Black,Title)→Title`/`(Title,Menu)→Menu` ✓ |
| gameplay-stage1 | Shop→Transition→StageStart→Combat | `(Menu,ShopArrival)→Shop`/`(Shop…,ExitTransition)→Transition`/`(Transition,StageStart)`/`(StageStart,MidCombat)→Combat` ✓ |
| respawn-window | StageStart→Combat→DeathRespawn | 上两条 + `(Combat,DeathRespawn)→DeathRespawn` ✓ |

- 锚点帧 180/360/540 = R-ui-7 常量（flow.rs 单测断言同值）✓；f1080/1200/1260 序列 = R-ui-6 ✓；
- t81 侧负向边（`Boot→Combat 必须拒`/`Combat→Shop 必须拒`）已测 ✓——**L5 若接入边集校验可直接复用**；
- t81/t84 任务 id 未显式落档（R14-4，链条本体已验）。

## 4. ④未证段 fail-closed 核 —— ✓ 合规

- golden 内 **无 GameOver/Continue**（grep 0 命中）✓ 未证不发明；
- 2 个 `checkpoint: null` 行 = 时序锚不发明检查点名 ✓；
- 负向自证：`GameOverUnproven` → "fail-closed" 违规 ✓。

## 5. ⑤负向自证核 —— 主负向 ✓ / 缺 2 条（R14-3）

diff 三向（全等/帧差/模式差）✓；未证模式 fail-closed ✓；缺帧序倒退负向、边违规负向（后者依赖 R14-1）。

## 6. 复现命令

```bash
cargo test --workspace 2>&1 | grep -c "test result: ok"   # 33 suite / 96 passed
python3 - <<'PY'                                           # golden vs 内联对账
import json,re
g=[(e['scene'],e['f'],e['mode'],e.get('checkpoint')) for e in map(json.loads, open('oracle/traces/stg01-chain.l5.jsonl'))]
src=open('crates/game-oracle/tests/l5_chain.rs').read()
blk=src.split('fn stg01_chain_parses_and_legality',1)[1].split('let problems',1)[0]
print(len(g), re.findall(r'ev\(', blk).__len__())
PY
grep -n "let _ = m" crates/game-oracle/src/l5.rs   # R14-1 桩实证
sed -n '95,109p' crates/game-core/src/flow.rs      # t81 已证边集
```
