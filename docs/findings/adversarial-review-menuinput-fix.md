# 对抗复核：B1 菜单跨键吞按修复（task-29）——adversarial-review-menuinput-fix

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不得复用实施方自述**）
> 被审（工作树未提交）：`crates/game-shell/src/host.rs`（patch 备份 `target/t29-fix.patch`）、`docs/findings/plan-menuinput-fix.md`（新）、`docs/spec/platform-shell.md`（§3.1/§4.2/§9 记录值）、`docs/findings/plan-playable-loop.md`（§10 mock-300 记录值，Lead 授权）。
> 复核时 HEAD `703130a`（复核期间并发推进：`4a0672b`=shell-win v2 实现、`703130a`=其 §8 回填、`cc4c692`=基线 357）。本文只写本文件、未改实现/被审文档、未 git add/commit。
> 复核方法：**双版本探针**——当前（修复后）game-shell 与 `git show HEAD:crates/game-shell/src/host.rs`（修复前）分别编译成临时探针（`%TEMP%`，不入库），同一脚本对跑；配合自跑测试/签名命令。

---

## 0. VERDICT：**pass（B1 修复成立；2 条非阻塞 findings/info；0 阻塞）**

- **修复前必红 / 修复后绿（独立双版本探针，非复用实施测试）**：
  - `right@590 + A@595 + START@600`：**修复前停在 Title（START 被吞）**；修复后 `Boot→Black@181→Title@361→**Menu@602**` ✓；
  - `A@595 + START@600`：修复前 Title；修复后 Menu ✓；
  - `B@800 + B@804`（Shop 内）：修复前停在 **ShopInfo（第二 B 被吞，ShopState=Info）**；修复后 `Shop→ShopInfo@801→Transition@837→StageStart@870→Combat@1110`，ShopState=**Exited** ✓；
  - `B@800 + B@812`（12f 间隔）：修复前 stuck ShopInfo；修复后 Transition@845→StageStart@878 ✓。
- **只改触发键域、已证机制未动（探针核）**：层 2 标题锁窗不变（START@500 被忽略、START@560 成功进入 Menu）；保持 START 600..620 只派发一次；保持 B 800..820 只触发一次 Shop→ShopInfo（**无边沿重复/连发**）；帧锚实测 `Transition→StageStart=33f`、`StageStart→Combat=240f`、`出店 B→Transition=+33f` 与声明一致。
- **Shop B 边沿模型**：`plan-menuinput-fix.md` §6 已显式登记 5 条未证域（B 保持/边沿门、两连最短间隔、菜单导航、相位计时源、START 连按<12f），选择"一次按下一次动作"fail-closed；无连发/保持重复 ✓。
- **实跑/签名**（与自述逐项核对）：`cargo test -p game-shell --lib` = **43 passed**（旧 41+2）；mock-300 `host=0x3085adda845a078d canvas=0x963ac9a776a57625 advanced_frames=290` ✓；STG-01 replay-3000 `host=0x521d397a6c609f34 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807` **未变** ✓；shell selftest `host=0x5d7156266d6ac19d`、两次 stdout 逐行一致 ✓。
- **写范围/架构**：patch 只含 `host.rs` + `platform-shell.md`（`input.rs`/`game-core::MenuInputGate`/game-core/game-platform 未改）；`game-shell` 仍零平台依赖；基线 357（`cc4c692`）与 +2 测试一致。
- **非阻塞 findings**：F1（低）Shop B 由 `$18` 锁存改 `$16` 边沿带来**可观测 -1f 响应时延**（同一脚本 ShopInfo 模式帧 802→801），建议在 plan §5.3/§6 显式登记；F2（低/并发）`platform-shell.md` §4.2 selftest 记录块（本卡更新）在**当前 HEAD（已含 task-10 shell-win v2 `4a0672b`）** 下与实跑不符（见 §4），需 task-10 spec 回填或注时点；见 §5。

---

## 1. 行为探针（独立双版本，修复前 vs 修复后）

探针方法：`%TEMP%` 临时 crate 依赖 `game-shell`（修复后=工作区路径；修复前=复制 crate 后把 `host.rs` 换为 `git show HEAD:crates/game-shell/src/host.rs`，Cargo 依赖改绝对路径）；实现 `Input/Renderer/Audio` mock，按帧脚本驱动 `Host::tick`，记录 `(host.frame(), mode)` 序列与 `shop_state`。

| 脚本 | 修复前（HEAD host.rs） | 修复后（工作区） | 判定 |
|---|---|---|---|
| right@590, A@595, START@600（700f） | Boot→Black@181→Title@361（**停在 Title**） | Boot→Black@181→Title@361→**Menu@602** | **B1 修复成立** |
| A@595, START@600（700f） | 停在 Title | Menu@602 | 跨键吞按消除 |
| START@600 only（对照） | Menu@602 | Menu@602 | 控制组一致 |
| START@500（标题锁窗内） | Title（忽略） | Title（忽略） | 层 2 (450,550] 未变 |
| START@560（窗外） | — | Menu@562 | 层 2 未变 |
| START 600..620 按住 | — | 仅 Menu 一次 | 无重复派发 |
| B@800, B@804（Shop 内，1000/1300f） | Shop→ShopInfo（**第二 B 被吞**，ShopState=Info） | Shop@702→ShopInfo@801→**Transition@837**→**StageStart@870**→**Combat@1110**（1300f 运行），Exited | **B1 修复成立** |
| B@800, B@812（12f 间隔） | 停在 ShopInfo | ShopInfo@801→Transition@845→StageStart@878，Exited | 两连 B 短间隔可识别 |
| B 800..820 按住 | — | 仅 ShopInfo 一次（无 Transition） | 未引入保持重复/连发 |

**帧锚测量（修复后，同一探针）**：Transition@837→StageStart@870 = **33f**；StageStart@870→Combat@1110 = **240f**；出店第二 B（脚本@804）→Transition@837 = **+33f**——与 task-11 C6/既有常数声明一致；probe 中 Boot/Black/Title 帧锚（0/181/361）未变。
> 说明：修复前两连 B 路径无法到达 Transition，故帧锚对照以修复后探针 + 常量/代码未动（diff 仅 dispatch/latch/tests 三处）双重佐证。

## 2. 代码层复核（只改触发键域？）

`git diff crates/game-shell/src/host.rs` 仅 3 个 hunk（与 `target/t29-fix.patch` 一致）：
1. `tick()` L766-767：`menu_latch.press(menu_pressed_bitmap(...) & MENU_START)`——`$18` 锁存只保留 START 位（A 级 `bank00:92AA/9400 LDA $18/AND #$10`）；
2. `dispatch_menu(frame, pressed)` L566-598：`Title|Menu` 只把 `bitmap_18 & MENU_START` 喂 `MenuInputGate`；`Shop|ShopInfo` 用 `pressed & MENU_B`（`$16` 原始按下沿）；`_ => 0`（其它模式不派发，gate 只在 Title/Menu 被调用 ⇒ 层 1 `prev_held` 只反映 START）；
3. `mod tests` 新增 `run_with_button_script` + 2 回归测试（old host.rs `#[test]`=21 → new=23，lib 总 43=41+2 ✓）。
- **未动**：`auto_advance`/`due_events`/`SHOP_EXIT_RESPONSE_FRAMES`/`MENU_INPUT_UNLOCK_FRAME`/`33f/240f` 常数/已证边集；`input.rs` 与 `game-core::MenuInputGate` 未改（patch file list 只有 host.rs+platform-shell）✓。

## 3. 未证域与未引入行为

- `plan-menuinput-fix.md` §6 表显式登记：Shop B 保持/边沿门（未证→边沿 fail-closed）、两连最短合法间隔（未证→不设阈值）、菜单导航 A/Down/Up（未证→不接）、相位计时源（未证→沿用 450/550 窗口）、START 连按<12f（A 级机制边界→保持现有锁存）✓。
- 探针：B 按住只触发一次、两次按下两次触发（间隔 4f/12f 均可），**无连发/保持重复** ✓；START 保持只进一次 Menu ✓。
- **F1（低，非阻塞）**：Shop/ShopInfo 的触发源由 `$18` 锁存（press@N→N+1f）改为 `$16` 同帧边沿，带来**可观测 -1f 响应时延**——同一脚本 `B@800`：修复前 ShopInfo 模式帧 802、修复后 801（`host.frame()` 记录口径下）。该时延是所选边沿模型的固有副作用（B 门未证，fail-closed 合理），但 plan §5.3 只登记了"签名变化"，未登记该**可观测时延变化**；建议在 §5.3/§6 补一句（并说明出店 `+33f` 帧锚以派发帧起算、不受影响）。不影响 B1 目标与已证帧锚。

## 4. 测试/签名重算表（自跑，对照实施自述）

| 项 | 实施自述 | 我的实测 | 结论 |
|---|---|---|---|
| `cargo test -p game-shell --lib` | 43 passed | running 43 → **43 passed / 0 failed** | ✓ |
| 新增回归测试 | 2 条（修复前必红） | `menu_start_not_swallowed_by_other_keys`、`shop_two_b_presses_within_12_frames_both_register` 均 passed；且双版本探针独立证明修复前红 | ✓ |
| mock-300 | host 0x63b5a6…→**0x3085adda845a078d** | `host=0x3085adda845a078d canvas=0x963ac9a776a57625 advanced_frames=290 storage_len=24` | ✓ |
| STG-01 replay-3000 | host 0x521d… canvas 0xdf75… spawns=21 engine_frames=1807（未变） | 逐值一致：`host=0x521d397a6c609f34 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807`；C1 1207/1207、C6 571/571、C4 共同槽 16/16 | ✓ |
| shell selftest（300f ×2） | host 0xf51d…→**0x5d7156266d6ac19d**，两次一致 | `host_signature=0x5d7156266d6ac19d`、两次 stdout 逐行一致；`canvas_signature=0x963ac9a776a57625` 同 | ✓（display 见 F2） |
| `cargo test --workspace` | 全绿 | exit 0（56 个 test target 全 ok） | ✓ |
| `python gates/cli.py check` | PASS | **CHECK: PASS；PASS=11 FAIL=0 SKIP=1 TOLERATED=0**；rust-tests 366（基线 **357**；+9 为并发 task-10 shell-win v2 已入库测试，非本卡） | ✓（附并发说明） |
| 测试数基线 | 355→**357**（+2） | `cc4c692` 基线 357；本卡 +2（host.rs 21→23、lib 41→43）自洽 | ✓ |

**值链自洽性**：`platform-shell.md` L118-119 记录 host 值链 `task-1 0xcde9…→task-9 0xf51d…→task-29 0x5d7156266d6ac19d`；`plan-playable-loop.md` §10 记录 mock 链 `0xefe39a…→0x63b5a6…→0x3085adda…`（Lead 授权更新留痕）——与实测逐值一致；变化原因=签名覆盖 `menu_latch/menu_gate` 状态（修复改变锁存键域/层 1 更新时机），mock-300 脚本每 64 帧含 START（`mock_platform.rs` L149-169）故签名必变；replay 输入无 START 菜单键 ⇒ 签名预期不变（实测确认）✓。

## 5. 并发/文档状态（F2，非阻塞）

- 复核期间并发推进：`4a0672b`（shell-win v2 FrameView 渲染实现）+ `703130a`（其 §8 回填）已入库。**该 shell-win v2 变更不属于 task-29 patch**（patch 仅 host.rs+platform-shell）。
- **F2（低/流程）**：`platform-shell.md` §4.2 selftest 记录块（本卡更新）仍记 `display_signature=0xf68f9ffc0607cbad`、无 `view_signature` 行；但当前 HEAD（含 shell-win v2）实跑为 **`display_signature=0xe99af5849b632fbd` + 新增 `view_signature=0xf6603346bf818ffd`**。该差异**归因 task-10 shell-win v2 渲染/自检输出变更**（`4a0672b` main 新增 view_signature 行），非 task-29；task-29 声称"canvas/display 不变"在其时点树（shell-win 未改）成立。建议：task-10 的 spec 回填一并更新 §4.2 记录块（display/view 新值 + view 行 + 时点），否则本卡 platform-shell 提交后该记录块立即与实现不一致（不影响 B1 行为验收）。
- **F3（info）**：`plan-shellwin-v1.md` §9.6 的 host 值仍为 `0xf51d…`（待 shell-porter/task-10 同步；`platform-shell.md` §3.1/§9 已注明"该文件由 shell-porter 同步（不在本卡写范围）"）✓ 边界已声明。
- **写范围核对**：task-29 未提交改动 = `host.rs` + `platform-shell.md` + 新 `plan-menuinput-fix.md` + `plan-playable-loop.md`（§10 mock 记录值，Lead 授权、与主提交分开 pathspec）；`git status` 中 shell-win 改动已由 task-10 提交、`game-core/game-platform` 无改动、`input.rs` 未改 ✓。

## 6. 命令与输出摘要

```text
# 行为探针（临时 crate，修复后 / 修复前=HEAD host.rs 副本）
A_right590_A595_Start600    FIXED: Boot,Black@181,Title@361,Menu@602   | OLD: Boot,Black,Title (stuck)
A2_A595_Start600            FIXED: Menu@602                           | OLD: Title (stuck)
A3_Start600_only            FIXED: Menu@602                           | OLD: Menu@602 (control)
LOCK_Start500_window        FIXED: Title (ignored)                    | （层2未变）
LOCK_Start560_after         FIXED: Menu@562
HOLD_START_600to620         FIXED: Menu once
B_two_B_4f                  FIXED: Shop@702,ShopInfo@801,Transition@837,StageStart@870,Combat@1110,Exited
                            OLD:   Shop@702,ShopInfo@802 (stuck, Info)
C_hold_B_800to820           FIXED: ShopInfo only (no exit)  
D_two_B_12f                 FIXED: ShopInfo@801,Transition@845,StageStart@878,Exited | OLD: stuck ShopInfo

# 自跑测试/签名
cargo test -p game-shell --lib → running 43 tests; 43 passed; 0 failed
cargo test -p game-shell --test mock_platform ... → host=0x3085adda845a078d canvas=0x963ac9a776a57625 advanced_frames=290
cargo test -p game-shell --test stg01_replay → host=0x521d397a6c609f34 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807
target/debug/shell-win.exe --selftest-frames 300 ×2 → host_signature=0x5d7156266d6ac19d; stdout 逐行一致
  canvas_signature=0x963ac9a776a57625; display_signature=0xe99af5849b632fbd; view_signature=0xf6603346bf818ffd
cargo test --workspace → exit 0（56 test target 全 ok）
python gates/cli.py check → CHECK: PASS; SUMMARY PASS=11 FAIL=0 SKIP=1 TOLERATED=0; rust-tests 366(基线 357)
git show HEAD:crates/game-shell/src/host.rs | grep -c '#\[test\]' = 21 ; 工作区 = 23（lib 41→43）
git diff --stat → host.rs 111 行、platform-shell 23 行、plan-playable-loop 7 行；patch=target/t29-fix.patch（host.rs+platform-shell）
```

---

## 7. 复核边界

1. 只复核 task-29 未提交改动（HEAD `703130a` 时点）；task-10（shell-win v2）已并发入库，其渲染/自检输出差异按 §5 归因、不在本卡判据。
2. 行为探针用 mock Input/Renderer/Audio 驱动 `Host::tick`（未跑 GUI/winit）；GUI 键盘映射与窗口路径任务-11 已覆盖。
3. 未对 Shop B 的真实 ROM 保持/边沿语义取证（本就未证并已登记）；探针只验证"一次按下一次动作/不重复"。
4. 签名值按当前工作树（含 task-10 已提交代码）实测；task-10 后续 spec 回填可能再次更新 §4.2 记录块（F2）。

---

【2026-10-18】decomp-verifier，复核 HEAD `703130a`；被审=task-29 未提交 B1 修复 patch。
