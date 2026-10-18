# 对抗复核：玩家弹丸/命中接线（task-37）——adversarial-review-bullets-hit

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用实施方自述**）
> 被审对象：**commit `bda9423`**（11 files，+1280/−44：`crates/game-shell/{fire,shape,host,render,lib}.rs` + `tests/stg01_combat.rs`、`crates/shell-win/{lib,main,smoke}`（task-36/37 适配随行）、`docs/findings/plan-bullets-hit.md`、`docs/spec/platform-shell.md`）；基线同步 commit `2f8a90e`（rust_test_count 366→377）。
> 复核时 HEAD=`2f8a90e`；共享 target 已清理，全部构建用**独立 target `target/verify42`**（仓库内相对路径）。本文只写该文件、未 git add/commit；工作区其余 M/?? 为并行 docs（非本卡）。
> 方法：①自建独立探针（`%TEMP%/probe42*` 临时 crate，真实 ROM + 真实 `Host` 驱动，不入库）②独立复跑全部测试/签名 ③ROM 直读 + oracle trace 独立重算证据链（`$94DF`/fire-hold/形状表）。

---

## 0. VERDICT：**pass（2 条非阻塞 findings 需后续修 + 4 条 info；0 阻塞）**

- **生成/运动/释放真接线（非测试桩）**：自建探针在真实 `Host` 的 STG-01 Combat 段按住 B → 出生帧 `1501,1508,1515,1522,…` **Δ=7 恒定**；`shot_timer` 循环 `7,6,5,4,3,2,1,7,…`；首弹 y 逐帧 `151→142→…→16`（**−9/frame**，`shot::tick` LUT 真执行）；`StageState.bullets` 全程非零计数 **max=0**（shell 自有网格，无双步进）✓。
- **形状数据来自 ROM bank2（非造数据）**：装载 59 形；探针打印首 5 形 `(addr,len,bytes)` 与我用 Python 独立按 ROM 直读+`SHAPE_PTR` 表重算的**逐字节全等**（`986a 5 0101ff01ff` / `986f 5 01fd03fc04` / `987a 5 01fe02fc04` / `9885 5 01fa06fb05` / `989d 5 01fc04fb05`）✓。
- **命中用真实 ROM 形表（非注入）**：用真实 shape idx=1（未注入合成流）命中 → `EnemyHitEvent{enemy_slot:8, damage:0, fail_closed:true}`、弹回收、`f630` 不写 ✓；负例（框远离/属性不交/敌槽 type bit7）均无事件、弹不回收 ✓。
- **`$4A` 白名单**：`type==0x4A` 命中 → `damage=2, fail_closed=false, f630 14→12`；非 `$4A` → 0 伤 + `fail_closed=true`、`f630` 不变 ✓。**但**当前 43 记录 STG-01 主调度流不含 `$4A`（见 F5：端到端不可达，仅单测构造覆盖；文档应写明）。
- **reload=7 更正成立（A 级三重核对）**：ROM `bank02:$94DF`=`07 05 04 03 02`（我直读）、`bank02:$9391 LDA $94DF,Y / $9394 STA $39`（Y=`$3C` 连射级）、`$9200/$92C9` 的 `#8` 只是初始化；`oracle/traces/fire-hold.shots.jsonl` 我独立按槽 lifetime 0→255 检测 **92 次出生，Δ 恒=7**。t13 旧值 8 已按此更正 ✓。
- **确定性读数组全部独立复现**（见 §2）：lib **53**、combat `shapes=59 bullets_seen=17 hits=0 killed=0 signature=0x7c8bb02930e895dc`、mock-300 `0x8170870e7c5e4ac6`、replay-3000 `0xdc3496b92f49ef45`（C1 1207/1207、C6 571/571、C4 16/16 未变）、selftest `0x68547665bf1ab396` ×2 一致；`cargo test --workspace` 376 passed / 0 failed（静态计数 377）；`gates check` **PASS=11 FAIL=0 SKIP=1**（rust-tests 377/377）。
- **架构/写范围**：`cargo tree -p game-shell` 仅 game-core/game-data/game-platform（零平台依赖）；`bda9423` **未改** game-core/game-platform/rom/reference/data（基线 `2f8a90e` 为独立 status commit）；shell-win 三文件为 **task-37 字面量构造适配 + task-36 E3/E4 随行**（commit message 明示 + Lead 授权）；`plan-bullets-hit` §6/§8.3 明确"能命中≠H-16 收口"，未夸大 ✓。

---

## 1. 独立探针（真实 ROM + Host；非引用实施测试）

探针 = `%TEMP%` 临时 crate（`game-shell`/`game-core`/`game-platform` path 依赖），实现 `AssetSource/FixedInput/NullRenderer/Audio` mock，复用 integration 同事件序列（f180 Black / f360 Title / f900 Menu / f1080 ShopArrival / f1193 ExitTransition / f1260 StageStart / f1500 MidCombat）到 Combat；逐帧读取 `Host::player_bullets()/enemy_hit_log()/stage_state()/shot_timer()`。

| 探针 | 我的实测输出（节选） | 判定 |
|---|---|---|
| P0 装载 | `mode=Combat shapes=59 stage_loaded=true sched=43 spawns=8 engine_bullets_nonzero=0` | ✓ |
| PA 生成节奏 | `births=[(1501,128,128,151),(1508,129,128,151),(1515,130,128,151),…]`；`deltas=[7,7,7,7,7,7,7,7,7,7]`；`timers=[7,6,5,4,3,2,1,7,…]` | ✓ 真生成 |
| PA 运动/死亡 | 首槽 `y=151,142,133,…,16`（−9/frame，LUT-0x0C）；之后槽消失（借位死亡回收） | ✓ `shot::tick` 真调用 |
| PB 形状 | `shape_count=59`；首 5 形与 Python ROM 重算逐字节全等（见 §0） | ✓ ROM 来源 |
| PC1 命中正例 | 真实 shape idx=1、`f518=1`，事件 `{frame:1641,slot:8,damage:0,fail_closed:true}`、弹回收、`f630=4` 不变 | ✓ |
| PC2 负例（远框） | `events=[]`，弹仍活（pos 128,88） | ✓ |
| PC3 `$4A` | 事件 `{damage:2,fail_closed:false,f630 14→12}` | ✓ |
| PC4 多弹同帧 | 2 颗弹同帧同槽/同框 → `same_frame_events=2`（frame 1652，bullets 128/129） | **F1 证据** |
| PC5 敌槽 bit7 | 无事件、弹不回收 | ✓ 过滤 |
| PC6 玩家被击 | 敌框盖玩家 `(128,160)`，玩家状态不变、无玩家伤害事件 | ✓ 未接（W15）|
| PR 真实 replay | 2500 帧真实 player trace：`first_4a=None hits=0 killed=0 shapes=59 sig=0x7c8bb02930e895dc`（与集成测试逐位同签名） | ✓ 确定性；$4A 未出现 |

> 说明：PC1 使用**真实装载的 ROM shape 流**（非注入合成数据），仅 PC4（多弹同帧探针）为控制几何而注入单大框；所有注入仅发生在探针进程内，未改仓库。

## 2. 测试/签名重算表（独立 target `target/verify42`）

| 项 | 交付自述 | 我的实测 | 结论 |
|---|---|---|---|
| `cargo test -p game-shell --lib` | 53 passed | `running 53 → 53 passed / 0 failed` | ✓ |
| 全 package | lib 53 + contract 2 + mock 3 + combat 1 + replay 3 | 全部 ok（62 项） | ✓ |
| `stg01_combat --nocapture` | `shapes=59 signature=0x7c8bb02930e895dc`，两次 run 全等 | `frames=2500 shapes=59 bullets_seen=17 hits=0 killed=0 stage_clear=false mode=Combat`；`signature=0x7c8bb02930e895dc` | ✓ |
| mock-300 | `host=0x8170870e7c5e4ac6 canvas=0x963ac9a776a57625 advanced=290` | 逐值一致 | ✓ |
| replay-3000 | `host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807` | 逐值一致；C1/C6/C4 断言通过 | ✓ |
| shell-win selftest ×2 | host `0x68547665bf1ab396`、canvas/display/view 不变 | `host_signature=0x68547665bf1ab396`、`canvas=0x963ac9a776a57625`、`display=0xe99af5849b632fbd`、`view=0xf6603346bf818ffd`；两次 stdout 逐行一致 | ✓ |
| shell-win 测试 | — | lib 12 + smoke 21 全 passed | ✓ |
| `cargo test --workspace` | exit 0 | exit 0；376 passed / 0 failed；静态 `#[test]`=377 | ✓ |
| `gates/cli.py check` | CHECK PASS | **PASS=11 FAIL=0 SKIP=1 TOLERATED=0**；docs-links 334 md/423 links；rust-tests 377（基线 **377**，`2f8a90e`） | ✓ |

**签名值链**（platform-shell L118-119/L202-204）：task-1 `0xcde9…`→task-9 `0xf51d…`→task-29 `0x5d7156266d6ac19d`→task-37 `0x68547665bf1ab396`；mock 链 `0xefe39a…`→`0x63b5a6…`→`0x8170870e7c5e4ac6`；replay 由 `0x521d…`→`0xdc3496b92f49ef45`（原因=签名新增弹网格/计时器/shape/命中事件字段，语义扩展非回归）。canvas/display/view 不变与实测一致 ✓。

## 3. 重点核对（逐条）

1. **已证 API 真调用**：`fire::step_player_bullets` 直接 `shot::tick`（fire.rs L161-180）；`generate_shots` 用 `shot_slot_alloc`/`spawn_init`（L111-132）；命中用 `shot_hitbox_register`/`fetch_shape_boxes`/`hit_resolves`/`clear_hit_entry`（L214-264）；`BulletGrid` 为 game-shell 自有、`StageState.bullets` 保持空（探针 max=0）；无 game-core 改动 ✓。
2. **未证不发明 / 表述边界**：plan §6（L107-120）与 §8.3（L167-171）逐项登记：H-16（W15/W15′/W18）不接、普通敌 HP（B4）0 伤 fail-closed、`$4A` HP0 不回收、伤害路径仅 Const2、`$34` 非 0 分派未接、`$3C` 未建模、敌弹/玩家被击未接；**未把"能命中"夸成 H-16 收口** ✓。
3. **reload=7**：见 §0 三重证据；`SHOT_TIMER_RELOAD=7`（L32）、`RAPID_RELOAD_TABLE=[7,5,4,3,2]`（L34）、默认 `$3C=0` 路径正确；t13=8 已注明为初始化值 ✓。
4. **跨 crate 适配只影响构造**：`bda9423` 的 shell-win diff = `FrameView { ..Default::default() }` 字面量（lib view_tests / smoke 测试）+ task-36 E3/E4（main present 去 blit、E4 注释）；task-37 未改渲染行为；selftest 四签名（canvas/display/view）不变 ✓。
5. **写范围/并发**：commit 11 files；未动 game-core/game-platform/data/status/rom/reference；shell-win 三文件为授权例外（commit message 明示，含 task-36 已复核 E3/E4 随行）；`docs/spec/platform-shell.md` 同时含 task-10 F2/task-36/task-37 回填（commit message 明示）；`plan-shellwin-v2.md` 不在本 commit ✓。
6. **shape 装载**：88 表项→**60 个唯一地址→59 装载**（22 个表内重复地址折叠；唯一地址 `$9E48` 的首字节 `0xFF`=无效/越窗视界被跳过）；loader 逐形越窗/截断 fail-closed、重复保留首个（shape.rs L23-47）✓；建议文档把"59 唯一形"精确为"59/60 唯一（1 个越窗跳过；88 表项含 22 重复）"（F5）。

## 4. Findings 四分类（含精确修正建议）

### 4.1 实现/保真（中，非阻塞，**建议排下一小卡**）
- **F1：shell 命中管线未保真 ROM/engine 的"一框一伤、同帧不回打"语义**。
  - 证据：`fire.rs::resolve_player_hits` **弹外层**（L220）→敌槽内层（L227）→命中即 `release`（L256）；同一帧 N 颗弹都可各命中同一敌槽/同一矩形并各记一次伤害。而 `engine::combat_frame`（engine.rs L236-285）是**敌槽/矩形外层→弹内层**，每矩形命中后 `break` ⇒ 每矩每帧最多 1 伤；task-38 §2.4 明确 `clear_hit_entry`+`release` 的判据是"**一框一伤、同帧不回打**"（[playability-evidence-h16-coords.md](playability-evidence-h16-coords.md) L104）。
  - 我的探针 PC4：单大框 + 2 颗同帧弹 → **2 条事件（frame 1652, slot 8, bullets 128/129）**；若按 engine/ROM 语义应为 1 条。
  - 可达性：当前默认装载（weapon_level=0、单弹型、Δ7f、弹间 63px）下**不可达**（无 2 弹同帧同框）；但一旦接多弹型/weapon_level/`$3C` 连射升级，即成真实伤害放大。**建议**：`resolve_player_hits` 改敌槽/矩形外层、弹内层（或每帧记录"已消费矩形"集合），加一条"2 弹同帧同框 ⇒ 仅 1 伤"回归；**在 H-16/多弹型接线前完成**。非阻塞本卡"能生成/能命中"验收（默认单弹路径正确）。

### 4.2 规格/文档（低）
- **F2（F2a）** `fire.rs` L6 模块文档仍写"**reload=8**"，与同文件 L27-32 更正（7）/plan/实测矛盾——改 7（保留 t13=8=初始化说明）。
- **F3** `render.rs` L74 `live_bullets` 注释"engine 已暴露面"已过期（现为 shell 自有弹网格计数）；`platform-shell.md` L113 FrameView 旧行未列 `player_bullets`/`hits_this_frame`（L117 已新增一行，L113 应同步删/改）。
- **F4** `shell-win/src/lib.rs` L735 overlay 注释"玩家子弹：不画（FrameView 无坐标面）"已过期（task-37 已暴露 `player_bullets` 坐标面；overlay 仍有意不画，属 shell 渲染卡）——建议 task-44 顺手改措辞。
- **F5** plan §8.2 L155-157 把 `hits=0` 归因"弹/敌对齐未收口"；我的补充证据：**当前 43 记录 STG-01 主调度流解码无 `$4A`**（连 `type`/args 均无 0x4A；Boss/EN-9+ 后段流未解码），整合 replay 2500f `first_4a=None` ⇒ `$4A` 白名单伤害路径**端到端不可达、仅单测构造覆盖**（HP=14 证据=TAS f9930 快照 A 级，属未来 Boss 卡）。建议 §6/§8 加一句"主调度流无 $4A ⇒ 白名单仅构造级验证"；另把 shape "59 唯一"精确为 59/60（1 个 `$9E48` 越窗跳过，22 表项重复）。

### 4.3 证据不足/信息（info，不阻塞）
- **F6** 玩家被击 fail-closed 仅有文档登记（plan §6 L116），无运行时事件/字段；若后续要求"可观测 fail-closed"，建议加 `PlayerHitBlocked` TODO/事件（本卡不接正确）。
- **F7** snapshot/restore 不含新 host 状态（player_bullets/shot_timer/enemy_hit_log/shape 计数）；现口径"引擎内部态不入快照"可解释，但建议 snapshot.rs 注一句本卡新态同样不在快照（或后续卡扩展）。
- **F8** `hits_this_frame`/`hit_overflow` 仅在 `stage_loaded && engine_tick_active` 分支内清零；离开 gameplay 模式（如后续 Title/Shop 复位路径）可能残留上一帧显示——建议 tick 顶部清零或 else 清零（低）。
- **F9（流程/已披露）** commit 混入 task-36 E3/E4（shell-win）与 task-10 F2（platform-shell）文档：commit message 明示且 task-36 已 completed；建议 Lead 确认 task-36 复核记录与本提交绑定一致（无功能风险）。

### 4.4 越权/伪造
- **无**。未发现伪造证据/夸大能力；未改 game-core/game-platform/rom/reference/data；未证域均已登记（F5 为文档补充建议，非隐藏）。

## 5. 命令与输出摘要

```text
# 独立 target（仓库内相对）
CARGO_TARGET_DIR=target/verify42
cargo test -p game-shell --lib                     → 53 passed
cargo test -p game-shell                           → lib53+contract2+mock3+combat1+replay3 全绿
cargo test -p game-shell --test stg01_combat -- --nocapture
  → frames=2500 shapes=59 bullets_seen=17 hits=0 killed=0 ... signature=0x7c8bb02930e895dc
cargo test -p game-shell --test mock_platform ...   → host=0x8170870e7c5e4ac6 canvas=0x963ac9a776a57625
cargo test -p game-shell --test stg01_replay        → host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807
cargo test -p shell-win                             → lib12 + smoke21 全绿
cargo test --workspace                              → exit 0；376 passed / 0 failed
target/verify42/debug/shell-win.exe --selftest-frames 300 ×2
  → host=0x68547665bf1ab396 canvas=0x963ac9a776a57625 display=0xe99af5849b632fbd view=0xf6603346bf818ffd；两次逐行一致
python gates/cli.py check                           → CHECK: PASS；PASS=11 FAIL=0 SKIP=1；rust-tests 377（基线 377）
cargo tree -p game-shell                            → 仅 game-core/game-data/game-platform

# 独立探针（%TEMP%/probe42*，真实 ROM/Host；非引用实施测试）
PA births Δ=7；timers 7,6,5,4,3,2,1 循环；y −9/frame；StageState.bullets max=0
PB shapes=59；首5形与 Python ROM 重算逐字节全等
PC1 真实 shape idx=1 命中 → damage0/fail_closed true/弹回收/f630 不变
PC3 $4A → damage2/fail_closed false/f630 14→12
PC4 2 弹同帧同框 → 2 events（F1）
PC5 enemy slot type bit7 → 无事件、弹不回收
PC6 敌框盖玩家 → 玩家 (128,160) 不变、无玩家伤害事件
PR replay 2500f → first_4a=None hits=0 sig=0x7c8bb02930e895dc（与集成逐位相同）
```

## 6. 复核边界

1. 被审 = commit `bda9423`；基线 `2f8a90e` 为复核时 HEAD；其后并行 docs 改动不在范围；本报告写于 HEAD `2f8a90e`。
2. 探针用 mock `Input/Renderer/Audio` 驱动真实 `Host`，未跑 GUI/winit 窗口（task-44 渲染接线后的窗口验证另测）。
3. 无 oracle shots golden（t50 未转正，文档已声明不对拍弹列）；reload/形状/命中证据改用 ROM 直读 + fire-hold trace + 独立形状重算。
4. `$4A` 端到端不可达（主调度流无 0x4A），白名单仅单测构造级验证——已在 F5 要求文档写明；不属于本卡"H-16 收口"。
5. F1 为**潜在**保真偏差（默认装载不可达），故不阻塞本卡转正，但必须在多弹型/weapon_level/连射升级接线前修复。

---

【2026-10-18】decomp-verifier，复核 commit `bda9423`（HEAD `2f8a90e`，基线 377）；独立 target `target/verify42`。
