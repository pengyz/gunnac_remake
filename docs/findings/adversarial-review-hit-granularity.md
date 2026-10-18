# 对抗复核：per-rect 命中消费（task-67）——adversarial-review-hit-granularity

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用实施方自述**）
> 被审：**task-67 未提交工作树**（`target/t67-fix.patch` 备份，2 files：`crates/game-shell/src/fire.rs`、`docs/findings/plan-bullets-hit.md`）。
> 复核时 HEAD=`3d8b557`；上一版（槽级 `slot_hit`，task-58）已入库（`113b2d8`）⇒ 旧版可直接 `git show HEAD:crates/game-shell/src/fire.rs` 取得。全部构建用**独立 target `target/verify67`**；双版本探针在 `%TEMP%` 临时 crate（真实 ROM/真实 Host），不入库。本文只写该文件、未 git add/commit。

---

## 0. VERDICT：**pass（0 阻塞；2 条 info）**

- **F1-new 收口成立（多矩形同帧 per-rect 对齐 engine）**：真实 Host/直接 API 双版本探针（当前 vs 旧槽级版）——
  - **新版**：同敌双矩形、2 弹同帧各命中一框 ⇒ `shell events=2, $4A f630=14→10, 两弹回收`；
  - **engine 对照**（`engine::combat_frame`，同一 shape/敌/弹/物理帧位）⇒ `f630=10, 两弹回收` ⇒ **完全一致** ✓；
  - **旧版（task-58 槽级）**：`events=1, f630=12, 第二弹保留` ⇒ **旧版红** ✓（与 task-63 F1-new 证据一致）。
- **同矩形同帧仍 1 伤**：新版 `events=1, 第二弹保留`（既有 `two_bullets_same_frame_same_enemy_box_one_damage` 保持绿；与旧槽级版该形态相同）✓。
- **不回归**：`cargo test -p game-shell --lib`=**55 passed**（+1 per-rect 回归）；全 package（lib55+contract2+mock3+combat1+replay3）全绿；`cargo test --workspace` exit 0（381 passed/1 ignored，静态 **382**）；`gates check` **PASS=11 FAIL=0 SKIP=1**（rust-tests 382/基线 381；docs-links 342 md/436 links）✓。
- **签名全未变**：mock-300 `0x8170870e7c5e4ac6`、replay-3000 `0xdc3496b92f49ef45`、combat `0x7c8bb02930e895dc`、selftest `host=0x68547665bf1ab396`、canvas `0x963a…`、display `0xe99a…`、view `0x15dbc2c7b0ba0a5d`（task-44 时点）全部与 task-44/58 时点值一致；selftest ×2 逐行一致 ✓。
- **方案/文档**：plan §3.4 已改为 **per-rect consumed set**（每矩形 1 伤、同矩形同帧不回打；删除槽级 `slot_hit`），判据=engine `combat_frame` L236-285 + task-38 §2.4 L104；`fire.rs` L212-217 措辞**不再称"同构"**，明确写"per-rect 与 engine 判据一致"；task-58 的 F1-new 标记已收口 ✓。
- **写范围**：patch 仅上述 2 文件；未碰 shell-win/game-core/game-platform/rom/reference/data ✓。
- **info**：①新增回归只覆盖 `$4A` 白名单多矩形（非白名单多矩形会产 2 条 fail_closed 事件但无测试）；②`clear_hit_entry(&enemy_box)` 返回值仍未使用（engine 同为局部副本，无可观测影响；task-63 已记 info，未变化）。

---

## 1. 双版本独立探针（真实 ROM + 真实 Host；旧版=task-58 槽级版=HEAD fire.rs）

探针：`%TEMP%/probe67*` 临时 crate（`game-shell` path 依赖；旧版=复制当前 crate 后仅替换 `fire.rs` 为 `git show HEAD:crates/game-shell/src/fire.rs`）；场景=真实 ROM 装载→事件到 Combat→`Host` 驱动；多矩形对比=直接 `fire::resolve_player_hits` vs `engine::combat_frame`（同一 shape/敌/弹/物理帧位；engine 先 LUT 步进 dx=+9,dy=0，shell 用步进后等效位置）。

| 场景 | 旧版（task-58 槽级） | 新版（task-67 per-rect） | engine 对照 | 判定 |
|---|---|---|---|---|
| 2 弹同帧命中**同一矩形**（单框，Host） | events=1, 第二弹保留 | **events=1, 第二弹保留** | 每矩形 inner break ⇒ 1 伤 | 不回归 ✓ |
| 2 弹同帧分别命中**同一敌两个矩形** | **events=1, f630=12, bullets=[0,255]** | **events=2, f630=10, bullets=[0,0]** | **f630=10, bullets=[0,0]** | 新版=engine ✓；旧版红 ✓ |
| `$4A` 同帧同框（Host） | events=1, f630=12 | events=1, f630=12 | — | 保持 ✓ |
| 非白名单同帧同框（Host） | events=1, f630=4 不变 | events=1, f630=4 不变 | — | 保持 ✓ |

> 多矩形 shape 流=`[2, 0xD8,0xE2,0xF8,8, 30,40,0xF8,8]`（左框 x∈[−40,−30]、右框 x∈[+30,+40]，y∈[−8,8]）；与新增单测 `stage_with_two_rects` 同构，但探针独立构造/驱动，未引用实施测试。

## 2. 测试/签名重算（独立 target `target/verify67`）

| 项 | 结果 | 说明 |
|---|---|---|
| `cargo test -p game-shell --lib` | **55 passed / 0 failed** | task-37 53 + task-58 1 + task-67 1 ✓ |
| `cargo test -p game-shell` | lib55+contract2+mock3+combat1+replay3 全绿 | ✓ |
| `cargo test --workspace` | exit 0；381 passed / 1 ignored（静态 382） | ✓ |
| combat `--nocapture` | `shapes=59 bullets_seen=17 hits=0 killed=0 signature=0x7c8bb02930e895dc` | 未变 ✓ |
| mock-300 | `host=0x8170870e7c5e4ac6 canvas=0x963ac9a776a57625 advanced=290` | 未变 ✓ |
| replay-3000 | `host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807`；C1/C6/C4 通过 | 未变 ✓ |
| shell-win selftest ×2 | host `0x68547665bf1ab396`、canvas `0x963a…`、display `0xe99a…`、view `0x15dbc2c7b0ba0a5d`；两次逐行一致 | 全同 task-44/58 时点 ✓ |
| `gates/cli.py check` | **CHECK: PASS；PASS=11 FAIL=0 SKIP=1**；rust-tests 382（基线 381）；docs-links 342/436 | ✓ |
| 旧版红证据 | 双版本探针（§1）旧版 1 伤/f630=12/第二弹保留 | ✓ 独立于实施测试 |

**回归测试内容核对**（fire.rs）：
- `two_bullets_same_frame_different_rects_of_same_enemy_two_damage`（L471）：双矩形、两弹 x=93/163（敌 x=128；左框 88..98、右框 158..168），断言 **2 事件/enemy_slot=8/damage=2/`f630=14−4=10`/两弹 state=0**；注释标"修复前槽级=1 伤且第二弹保留（红）"——与我的双版本探针一致 ✓。
- `two_bullets_same_frame_same_enemy_box_one_damage`（L441）保持绿 ✓；单弹/多弹/`$4A` 7 发 14→0/负例（远框/属性不交/敌弹）全 lib 测试通过 ✓。

## 3. 方案/文档核对

- plan §3.4（L69-86）：明确三阶段沿革（task-42 F1 去重 → task-58 槽级 → task-67 收口 per-rect），判据=engine `combat_frame` L236-285 + task-38 §2.4 L104；回归设计 1（不同矩形 2 伤，旧红）/2（同矩形 1 伤保持绿）/3（单弹/多弹/7 发/负例）；默认装载不可达多矩形同帧形态的说明保留 ✓。
- `fire.rs` L212-217：改为"命中粒度=逐矩形消费……每矩形每帧最多 1 伤、同矩形同帧不回打；**不做**槽级跳过……= engine `combat_frame`（L236-285，每矩形 break）与 task-38 §2.4 L104 判据的 per-rect 语义（task-63 复核多矩形探针已对拍：shell=2 伤 / engine=2 伤）"——**无"同构"失实表述** ✓。
- plan §8.2（L171-198）：lib 55、workspace 绿、签名未变、rust-tests 382（基线 381）与实跑一致；F1-new 标记为"已由 task-67 收口" ✓。
- 写范围：`target/t67-fix.patch`（UTF-16 解码）仅 2 文件（fire.rs + plan-bullets-hit.md）；未改 shell-win/task-59 的 platform-shell 行（F3 剩余项归属 task-59，未越权）✓。

## 4. Findings 四分类

### 4.1 实现/文档（无阻塞）
- **F1-new（task-63）已收口**：per-rect 消费与 engine/evidence 对齐；旧槽级版由双版本探针复现红。**无剩余实现 finding**。
- **F2（info，测试对称性）**：新增回归只覆盖 `$4A` 白名单的"不同矩形 2 伤"；**非白名单**多矩形场景（应产 2 条 `fail_closed` 事件、0 伤）无专门断言——逻辑同一路径，建议顺手补一条对称用例（不阻塞）。
- **F3（info）**：`clear_hit_entry(&enemy_box)` 返回值 `_cleared` 仍未使用（task-63 已记；engine 同为局部副本，无可观测影响）；如要清理可在后续 docs/refactor 卡顺手改为注释说明或去掉未用绑定。

### 4.2 证据不足
- 无。方向为"实现向已证 engine/判据对齐"，未新增能力声称；多矩形同帧仍属未达默认装载形态（Boss/多弹型接线前必须保持此对齐）。

### 4.3 越权/伪造
- **无**。patch 2 文件、未碰 shell-win/game-core/game-platform/rom/reference/data；文档与实现一致；双版本旧版红可复现。

---

## 5. 命令与输出摘要

```text
HEAD=3d8b557（被审=task-67 工作树 2 files；target/t67-fix.patch）
CARGO_TARGET_DIR=target/verify67
cargo test -p game-shell --lib              → 55 passed
cargo test -p game-shell                    → lib55+contract2+mock3+combat1+replay3 全绿
cargo test --workspace                      → exit 0；381 passed / 1 ignored（静态 382）
cargo test --test stg01_combat -- --nocapture
  → shapes=59 bullets_seen=17 hits=0 killed=0 ... signature=0x7c8bb02930e895dc
cargo test --test mock_platform ...         → host=0x8170870e7c5e4ac6 canvas=0x963ac9a776a57625
cargo test --test stg01_replay              → host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807
target/verify67/debug/shell-win.exe --selftest-frames 300 ×2
  → host=0x68547665bf1ab396 canvas=0x963ac9a776a57625 display=0xe99af5849b632fbd view=0x15dbc2c7b0ba0a5d；两次逐行一致
python gates/cli.py check                   → CHECK: PASS；PASS=11 FAIL=0 SKIP=1；rust-tests 382（基线 381）
cargo tree -p game-shell                    → 仅 game-core/game-data/game-platform

# 双版本探针（%TEMP%/probe67*，真实 ROM/Host；旧版=git show HEAD:...fire.rs=task-58 槽级）
NEW:  same_rect events=1 (second bullet alive) ; MULTIRECT shell(events=2,f630=10,bullets=[0,0]) == engine(f630=10,bullets=[0,0])
OLD:  same_rect events=1 ; MULTIRECT shell(events=1,f630=12,bullets=[0,255])  ← 红（槽级）
```

## 6. 复核边界

1. 被审=task-67 工作树 2 文件；HEAD `3d8b557`（task-58 槽级版已入库 `113b2d8`，旧版对比基准明确）；其余并行 docs/crate 改动不计入本卡。
2. 双版本探针用 mock `Input/Renderer/Audio` 驱动真实 `Host`；多矩形直接 API 对比用合成 shape 流（几何受控），未跑 GUI。
3. 默认装载（level0/单弹型/Δ7f/弹间 63px）不可达多矩形同帧形态；Boss/多弹型接线前本对齐必须保持（回归已锁）。
4. 未对拍 oracle shots golden（t50 未转正，沿用既有口径）。

---

【2026-10-18】decomp-verifier，复核 task-67 工作树（HEAD `3d8b557`）；独立 target `target/verify67`。
