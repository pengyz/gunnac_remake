# 对抗复审 #15（t183）：t180 L5 合法性实装（桩清算）复审

> 被审方：t180（92317eb）。审核方：verifier（t183），边集完备性核 + 测试独立复跑 + 语义回归核。
> **裁定：pass（R14 全项清算）**——①反演实现完备（9/9 事件逐一对应，t81 由构造不漂移）；
> ②四负向全落+独立复跑绿；③"无语义新错"声明验证成立（98/98）；④低危三项全落。观察 2 条（琐碎）。

## 0. 观察（琐碎，不阻塞）

- O1：自配对跳过（`prev != m` 才查边）= 持续态语义（非转移）——合理；若严格 fail-closed 可加"持续须在自环集
  {Menu, Combat, DeathRespawn}"注（t81 明列三自环），现语义可接受；
- O2：`l5_scope_note` 测试用相对路径 `../../oracle/traces/…`（crate 根起算）——现可用，脆但不误报。

## 1. ①桩→实装复核 —— ✓ 反演完备、声称与实现对齐

| 项 | 实现 | 判定 |
|---|---|---|
| 边集来源 | `edge_proven(prev,cur)` = 逐一尝试 **9 个 ModeEvent** 经 `FlowState::advance` 反演 | **由构造忠实 t81**（直接调用状态机，边表不漂移）✓ |
| 事件表完备性 | `ModeEvent` 枚举 = 9 变体（Black/Title/Menu/ShopArrival/ShopPage/ExitTransition/StageStart/MidCombat/DeathRespawn） | 与反演数组**逐一对应，无遗漏** ✓ |
| 检查点 | 场景内相邻模式对（prev≠cur）过边集，违规="非法模式序列…fail-closed" | 与"合法性 fail-closed"声称**对齐** ✓（R14-1 桩清算） |
| 未知模式 | 维持 fail-closed | ✓ |

## 2. ②非法序列负向复核 —— ✓ 四例全落（独立复跑）

| 负向例 | 断言 | 测试 |
|---|---|---|
| Black→Combat | "非法模式序列"必拒 | `l5_illegal_sequence_fails_closed` ✓ |
| Boot→Combat | 同上（t81 现成负向边复用） | ✓ |
| Combat→Shop | 同上（t81 现成负向边复用） | ✓ |
| 帧序倒退（f100→f50） | "帧序倒退"必检出 | ✓ |

**独立复跑**：`cargo test --workspace` = **98/98 全绿**（含 2 新测试，基线 96→98 一致）。

## 3. ③导入误清处置核 —— ✓ "无语义新错"声明验证成立

- 两次 `use` 误删（regex 误清）→ 编译错 → 即时回修——最终态 `use game_core::flow::Mode`（l5_legality）+
  `use game_core::flow::{FlowState, ModeEvent}`（edge_proven）在位 ✓；
- 语义回归核：diff 面仅边集实装 + use 行调整；**全套 98 测试绿** = 无语义新错 ✓（声明成立）；
- 自查纪律（grep 三查捕获误清）= t159 立的 pattern 的实战成功案例 ✓。

## 4. ④低危三项核 —— ✓ 全落

| 项 | 落点 | 判 |
|---|---|---|
| R14-2 口径注 | **测试化**：`l5_scope_note_3scenes_10events`（3 场景/10 事件/8 命名，golden 对账断言） | ✓（口径以测试钉死，防漂移） |
| R14-3 帧序倒退负向 | `bad4` 断言 | ✓ |
| R14-4 id 落档 | t81 多处引用（l5.rs:86/91/101、l5_chain.rs:61/68）+ t84 断言消息 | ✓ |

## 5. 复现命令

```bash
cargo test --workspace 2>&1 | grep -c "test result: ok"   # 33 suite / 98 passed
grep -n "edge_proven\|非法模式序列" crates/game-oracle/src/l5.rs
sed -n '73,83p' crates/game-core/src/flow.rs               # ModeEvent 9 变体对账
cargo test -p game-oracle --test l5_chain                  # 四负向 + 口径测试
```
