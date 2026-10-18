# 对抗复核：同帧多弹重复伤害修复（task-58 F1）——adversarial-review-hit-dedup

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用实施方自述**）
> 被审：**task-58 未提交工作树**（`target/t58-fix.patch` 备份，3 files：`crates/game-shell/src/fire.rs`、`crates/game-shell/src/render.rs`（注释）、`docs/findings/plan-bullets-hit.md`）。
> 复核时 HEAD=`2bcb92a`（task-42 复核报告入库）；工作树同时含并行 task-44（shell-win WIP）/task-60 等改动——本报告只把上述 3 文件计入 task-58，其余按 §5 归因。
> 全部构建用**独立 target `target/verify58`**（仓库内相对）；双版本独立探针（当前 vs `git show HEAD:crates/game-shell/src/fire.rs` 旧版）在 `%TEMP%` 临时 crate 运行，不入库。本文只写该文件、未 git add/commit。

---

## 0. VERDICT：**pass**（F1 修复目标成立；**1 条中等级非阻塞新发现 F1-new**：`slot_hit` 粒度比 engine 更严、文档"同构"表述不准确；+2 info）

- **修复目标（同帧多弹同框只 1 伤）成立（双版本探针）**：
  - 真实 Host 场景（2 弹同帧落入同一大框）：
    - **修复后**：非白名单 `events=1, alive_after=1, f630=4`；`$4A` `events=1, alive_after=1, f630=12`（14→12）✓
    - **修复前（HEAD 旧 fire.rs）**：非白名单 `events=2, alive_after=0`；`$4A` `events=2, alive_after=0, f630=10`（14→10）⇒ **修复前必红** ✓
- **不回归**：`cargo test -p game-shell --lib`=**54 passed**（+1 回归）；全 package 全绿；`cargo test --workspace` exit 0（380 passed/1 ignored，静态 381）；`gates check` **PASS=11 FAIL=0 SKIP=1**（rust-tests 381/基线 377；docs-links 338 md/432 links）✓
- **确定性签名未变**（对照 task-37/42 时点值）：mock-300 `0x8170870e7c5e4ac6`、replay-3000 `0xdc3496b92f49ef45`、combat `0x7c8bb02930e895dc`、selftest `host=0x68547665bf1ab396` 全部逐位一致；canvas/display 不变；`view_signature` 由 task-44 WIP 变为 `0x15dbc2c7b0ba0a5d`（**非本卡**，与 Lead 说明一致）✓
- **F2/F3/F5 落实且可核**：`fire.rs` L6 reload=7（`$94DF` 表；t13=8 为初始化值已在注释）；`render.rs` L74 `live_bullets` 改为"game-shell 自有弹网格"；plan §3.1/§3.4/§6/§8.2 增 `$4A` 端到端不可达（43 记录主调度流无 0x4A、2500f `first_4a=None`）、HP=14 TAS f9930、shape 88→60 唯一→59 装载（`$9E48` 首字节 `0xFF` 越窗）——与我的 ROM/trace 独立复算一致 ✓
- **写范围**：patch 仅上述 3 文件；**未碰 shell-win**（工作树 shell-win 改动=task-44 WIP）；未改 game-core/game-platform/rom/reference/data ✓
- **新发现 F1-new（中，非阻塞 latent）**：新实现把"同帧不回打"做成**每敌槽**门（`slot_hit`，fire.rs L246），而 `engine::combat_frame`（L236-285）是**每矩形**一伤（多矩形敌同帧可被不同弹各打中一框）；我的多矩形探针：**新 shell = 1 伤（$4A 14→12）vs engine = 2 伤（14→10，与旧 shell 一致）**。默认装载（level0/单弹型/弹间 63px）不可达、无 over-damage，故**不阻塞 task-58 目标**；但 `fire.rs` L212 / plan L71 声称"与 engine 同构"**不准确**，建议提交前改文档措辞，或下一卡改回**逐矩形消费集合**并补多矩形回归（Boss/multi-rect 敌接线前必须解决）。

---

## 1. 双版本独立探针（真实 ROM + 真实 Host；旧版=`git show HEAD:...fire.rs` 复制 crate）

探针：`%TEMP%/probe58*` 临时 crate（`game-shell` path 依赖；旧版=复制当前 crate 后仅替换 `fire.rs`）；场景=真实 ROM 装载→事件序列到 Combat→B 长按生成弹→控制几何后 tick。

| 探针 | 旧版（HEAD fire.rs） | 新版（task-58 工作树） | 期望 |
|---|---|---|---|
| 2 弹同帧同框（非白名单 `0x2B`，f630=4） | `events=2, alive_after=0, f630=4` | **`events=1, alive_after=1, f630=4`** | 新版=1 伤/第二弹保留 ✓（旧版红） |
| 2 弹同帧同框（`$4A` HP=14） | `events=2, alive_after=0, f630=10` | **`events=1, alive_after=1, f630=12`** | 14→12、第二弹保留 ✓（旧版 14→10 红） |
| 多矩形敌、2 弹分别命中两个矩形（shell 直接调用 `fire::resolve_player_hits`） | shell `events=2, f630=10`（与 engine 同） | **shell `events=1, f630=12`** | **F1-new**：engine=2 伤/`f630=10`，新版过严 |
| 同场景 engine 对照（`engine::combat_frame`，同一 shape/敌/弹） | `f630=10, bullets=[0,0]` | `f630=10, bullets=[0,0]` | engine 逐矩形：2 伤 |
| 单弹/负例（远框/属性不交/敌槽 bit7） | — | 与既有断言一致（lib 54 全绿） | ✓ |

> 探针细节：多矩形 shape 流=`[2, 0xF0,0xF8,0xF0,0x10, 0x08,0x10,0xF0,0x10]`（敌 x=100；矩形 x=84..92 与 108..116，y=84..116）；shell 弹放在 engine 步进后的等效位置（x=88/112），engine 初值 x=79/103（motion0: LUT `dx=+9,dy=0`）——两侧比较的是同一物理帧位。2 弹同框 Host 探针用单大框 `[1,0xC0,64,0xC0,64]` 覆盖两弹（真实 Host 生成、非注入弹）。

## 2. 测试/签名重算（独立 target `target/verify58`）

| 项 | 结果 | 说明 |
|---|---|---|
| `cargo test -p game-shell --lib` | **54 passed / 0 failed** | task-37 的 53 + F1 回归 1 ✓ |
| `cargo test -p game-shell`（全 package） | lib54 + contract2 + mock3 + combat1 + replay3 全绿 | ✓ |
| `cargo test --workspace` | exit 0；380 passed / 1 ignored（静态 381） | 含并行 task-44 工作树 +3；task-58 +1 |
| combat `--nocapture` | `shapes=59 bullets_seen=17 hits=0 killed=0 signature=0x7c8bb02930e895dc` | 未变 ✓ |
| mock-300 | `host=0x8170870e7c5e4ac6 canvas=0x963ac9a776a57625 advanced=290` | 未变 ✓ |
| replay-3000 | `host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807`；C1/C6/C4 通过 | 未变 ✓ |
| shell-win selftest ×2 | `host=0x68547665bf1ab396`、`canvas=0x963a…`、`display=0xe99a…` 不变；`view=0x15dbc2c7b0ba0a5d`；两次逐行一致 | view 变=task-44 WIP（非本卡）✓ |
| `gates/cli.py check` | **CHECK: PASS；PASS=11 FAIL=0 SKIP=1**；rust-tests 381（基线 377）；docs-links 338/432 | ✓ |
| 旧版红证据 | 双版本探针（§1）旧版 2 事件/14→10 | ✓ 独立于实施测试 |

**签名值链**：task-37 `0x68547665bf1ab396` 等四签名在 task-58 前后**逐位相同**（本卡只改"同帧多弹"这一不可达形态，默认单弹路径事件序不变）——与实施自述一致 ✓。

## 3. F2/F3/F5 与写范围核对

- **F2**：`fire.rs` L6 已改"默认 reload=7；`$94DF` 表"；同文件 L27-32 保留 t13=8=初始化说明（ROM `$9200/$92C9` 的 `#8` 后由配置装载覆盖）✓。
- **F3**：`render.rs` L74 `live_bullets` 注释改为"game-shell 自有弹网格全槽；A 级槽域"✓；`platform-shell.md` L113 的 FrameView 旧行未在本卡改（任务卡说明归 task-59 spec-writer）——**未越权** ✓。
- **F5**：plan L48（88 表项→60 唯一→59 装载，`$9E48` `0xFF` 越窗）、L84/L124（`$4A` 端到端不可达：43 记录主调度流无 0x4A、2500f `first_4a=None`；HP=14=TAS f9930 快照）、L171-184（签名未变/381 计数）——与我的独立复算一致 ✓。
- **写范围**：`target/t58-fix.patch` 解码（UTF-16）仅含 3 文件（fire.rs/render.rs/plan）；工作树 shell-win/其他 docs 改动分别在 task-44/task-60 范围，未计入本卡 ✓。

## 4. Findings 四分类

### 4.1 实现/文档精度（中，非阻塞 latent）
- **F1-new：`slot_hit` 粒度=敌槽，比 engine 的"每矩形一伤"更严；"同构"表述不准确**。
  - 位置：`fire.rs` L212 注释"与 engine::combat_frame L236-285 **同构**"；L246 `slot_hit **同一敌槽**本帧已命中 ⇒ 跳过其余矩形`；plan L71"已改为**同构**顺序"、L73"同一敌槽…跳过其余矩形 ⇒ 同帧多弹对同一框最多 1 次伤害"。
  - 事实：engine（L236-285）是**每矩形**内层弹循环 break，**不同矩形可同帧各被一颗弹命中**；task-38 §2.4 L104 的 A 级判据是"一框一伤、同帧不回打"（框=hitbox 条目）。我的多矩形探针：新版 shell 1 伤（14→12），engine 2 伤（14→10），旧 shell 与 engine 同（2 伤）⇒ 新实现把"同帧不回打"扩大到敌槽级。
  - 影响：默认装载不可达（level0/单弹型/Δ7f/弹间 63px；多矩形敌多为 Boss，当前未生成）；但 **Boss/多矩形敌接线前**会少算伤害（每帧每敌至多 1 伤），且文档"同构"为失实陈述。
  - 建议（二选一，提交前至少做 (b)）：(a) 改回**逐矩形消费**（per-rect 已消费集合/或 engine 式"每矩形取首颗弹"），补"同帧 2 弹命中同一敌的**不同矩形** ⇒ 2 伤（与 engine 对拍）"回归；(b) 若保留"每槽 1 伤"更严模型，则删/改 `fire.rs` L212 与 plan L71 的"同构"措辞，显式登记"同帧每敌槽≤1 伤（比 engine 更严，待 ROM 多框同帧证据）"，并把它列为未证排序。
- **F2（info）**：`clear_hit_entry(&enemy_box)` 由旧 `&shot_box` 改为 `&enemy_box`，但返回值 `_cleared` 未使用、engine 同处也是局部副本——**无可观测影响**，仅注释"清框"与实际借用对象可再对齐。
- **F3（info）**：新增回归仅覆盖**单矩形**同框（`two_bullets_same_frame_same_enemy_box_one_damage`）；多矩形场景无测试（F1-new 的缺口）。

### 4.2 证据不足/其他
- 无新的未证声明；F1-new 为"实现比证据更严"的方向，不是发明能力/夸大；"能命中≠H-16 收口"表述未被本卡改变（plan §6/§8.3 保持）。

### 4.3 越权/伪造
- **无**。patch 3 文件、未碰 shell-win/game-core/game-platform/rom/reference/data；docs 只改本卡范围；双版本旧版红可复现。

## 5. 命令与输出摘要

```text
HEAD=2bcb92a（被审=工作树 task-58 patch；target/t58-fix.patch 3 files）
CARGO_TARGET_DIR=target/verify58
cargo test -p game-shell --lib              → 54 passed
cargo test -p game-shell                    → lib54+contract2+mock3+combat1+replay3 全绿
cargo test --workspace                      → exit 0；380 passed / 1 ignored（静态 381）
cargo test --test stg01_combat -- --nocapture
  → shapes=59 bullets_seen=17 hits=0 ... signature=0x7c8bb02930e895dc
cargo test --test mock_platform ...         → host=0x8170870e7c5e4ac6 canvas=0x963ac9a776a57625
cargo test --test stg01_replay              → host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807
target/verify58/debug/shell-win.exe --selftest-frames 300 ×2
  → host=0x68547665bf1ab396 canvas=0x963ac9a776a57625 display=0xe99af5849b632fbd view=0x15dbc2c7b0ba0a5d；两次逐行一致
python gates/cli.py check                   → CHECK: PASS；PASS=11 FAIL=0 SKIP=1；rust-tests 381（基线 377）；docs-links 338/432

# 双版本探针（%TEMP%/probe58*，真实 ROM/Host；旧版=git show HEAD:...fire.rs）
NEW: HOST_NONWHITE events=1 alive_after=1 f630=4 ; HOST_4A events=1 alive_after=1 f630=12
OLD: HOST_NONWHITE events=2 alive_after=0 f630=4 ; HOST_4A events=2 alive_after=0 f630=10
NEW: MULTIRECT shell(events=1,f630=12,bullets=[0,255])  engine(f630=10,bullets=[0,0])
OLD: MULTIRECT shell(events=2,f630=10,bullets=[0,0])    engine(f630=10,bullets=[0,0])
```

## 6. 复核边界

1. 被审=task-58 工作树 3 文件；HEAD `2bcb92a`；工作树并行 task-44（shell-win/plan-shellwin-v2）+task-60（l5 spec）等改动不计入本卡（仅作签名/计数归因）。
2. 双版本探针用 mock `Input/Renderer/Audio` 驱动真实 `Host`；未跑 GUI/window。
3. 多矩形探针用合成 shape 流（几何受控）；单框探针用真实 Host 生成弹。多矩形敌在真实 replay 中当前不可达（Boss 后段未接线），故 F1-new 为 latent。
4. 未对拍 oracle shots golden（t50 未转正，沿用 task-37/42 口径）。

---

【2026-10-18】decomp-verifier，复核 task-58 工作树（HEAD `2bcb92a`）；独立 target `target/verify58`。
