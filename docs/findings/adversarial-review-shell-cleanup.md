# 对抗复核：shell-win v2 findings 收口（task-36）—— 证伪立场独立复核

> 复核方：`shell-verifier`（task-41，verifier 角色）；被审：task-36 / `shell-porter`（**未提交 WIP**）
> 被审范围：工作树 4 文件 +58/−6——`crates/shell-win/src/lib.rs`（+3 纯注释）、
> `crates/shell-win/src/main.rs`（+12/−3，E3）、`docs/findings/plan-shellwin-v2.md`（+41）、
> `docs/spec/platform-shell.md`（+8）。基线 HEAD=`b92139a`（复核时 HEAD 未推进）。
> 并发隔离：工作树另有 **task-37 WIP**（game-shell host.rs/render.rs + 新 fire.rs/shape.rs）——按文件归属隔离，
> **不计入本卡**；本报告测试时点与计数已显式标注。
> 纪律：[AGENTS.md](../../AGENTS.md) §5 独立对抗复核。
> **verdict：pass**（E3 行为变化与"四签名不变"均有独立证据；E4/E5/S1/S2/E6 文档落实；无实现错误。
> 非阻塞：E1 隔离时点 workspace/gates 因并发 WIP 不可复现（建议提交后由 Lead 顺手跑一次记录时点）、
> E2 clean-copy 重跑被 target 锁污染（改用隔离时点证据）、S1 每采样两次 recompose 的 CPU 小项登记。）

## 0. 独立性 / 边界

1. 先读 4 文件 diff 与代码路径 → 自跑测试/自检 → 自行 GUI 截图 SHA 对照 → 才比对 §8.5 自述。
2. 未改实现/被审文档，未 `git add`/`commit`；唯一写入=本文件；日志/截图在 `target/review-cleanup/`（可再生、不入库）。
3. 未覆盖：L4 像素一致（本卡不声称）；GUI 多分辨率；E3 的 softbuffer present **调用次数**无直接插桩计数（以代码路径+视觉不变+计数语义推断）。
4. 测试时点声明：**隔离时点 T0**=`task-36 4 文件为工作树唯一改动、game-shell 为 HEAD 干净态`（本卡自跑 `cargo test -p shell-win` 33 passed、selftest ×2）；
   **当前时点 T1**=task-37 WIP 已进入工作树（game-shell 编译失败、测试计数 373）——T1 的 workspace/gates 结果**不能归因本卡**。

## 1. 必查项结论

| # | 必查项 | 独立证据 | 结论 |
|---|---|---|---|
| 1 | **E3**：窗口模式 present 跳过 blit、统一 refresh 上屏 | `main.rs` `WindowRenderer::present` 现只剩 `self.core.present()`（原 `self.blit()` 已删）；全文件 `blit()` 调用点仅 `refresh_view()`（L73-75）+ 两处调用 `refresh_view`（L218 resumed 初始、L285 `frames>0` 采样）⇒ **每采样 1 次 softbuffer present**（旧=present 内 1 次 + refresh 1 次） | ✅ 确认 |
| 2 | selftest ×2 一致 + **四签名不变** | T0 自跑 ×2：stdout **SHA256 相等**（`F4A50EDE…D447B`）；`host=0x5d7156266d6ac19d` / `canvas=0x963ac9a776a57625` / `display=0xe99af5849b632fbd` / `view=0xf6603346bf818ffd`——与收口前完全一致 | ✅ 确认 |
| 3 | presents 计数语义变化是否说明 | selftest `presents=300` 不变（`FrameRenderer` headless 路径不受 `WindowRenderer` 改影响）；plan §3 + §8.5、spec §4.1/§8 均明写“计数语义不变（=逻辑帧数），变化的是 present/blit cadence（2→1）” | ✅ 说明充分 |
| 4 | frames==0 行为与旧一致 | 旧代码只在 `Host::tick`（frames>0 才会调用）内 present→blit；frames==0 时无 tick⇒无 present⇒无 blit。新代码 frames==0 同样无 refresh（L283 `if frames > 0`）⇒ **一致**；`resumed` 初始 `refresh_view` 为 v2 既有行为、非本卡改动 | ✅ 确认 |
| 5 | **E5**：HUD 可见性未证登记 + 绘制行为未变 | plan §5:112 新增“HUD 可见性/显示时机（模式门）未证，v2 所有模式恒画占位，不声称原版一致，不改绘制行为”；spec §8:276 新增同条；**绘制代码未变**（`lib.rs` 本卡 diff 仅 `//` 注释 3 行；overlay/HUD 绘制函数无改动）⇒ 像素不变（另见 #8） | ✅ 确认 |
| 6 | **E4**：哨兵歧义仅登记、不改签名 | `lib.rs:712-718` 只新增 3 行注释（`frame_view_digest` 逻辑未动）；plan §5:118 注记“不修/未来接线需加 tag 并更新 view_signature”；T0 实测 `view_signature` 不变 | ✅ 确认 |
| 7 | **S1/S2**：2 次 present 采样临时口径 | plan §3:90-96 新增口径段（收口前 2 次/收口后 1 次、presents 语义不变）；spec §8:279 同步 | ✅ 确认 |
| 8 | **E6**：GUI 自动化方法记录准确 | plan §8.2:172 改为“复核方用 `PostMessage(WM_KEYDOWN)` 注入成功 + PrintWindow 像素采样 ⋯ 该链可自动化（E6）”；§8.5:201 同表；方法描述与 task-12 实测一致（SendKeys/前台失败=本会话限制，PostMessage 可用） | ✅ 确认 |
| 9 | 写范围/纪律 | `git diff --stat` 仅 4 文件（shell-win 2 + 2 docs）且无测试文件改动（本卡对测试计数贡献=0）；game-shell 并发改动属 task-37（文件不重叠）；未提交（按卡指示由 Lead 精确 pathspec 提交） | ✅ 确认 |
| 10 | 回归/门禁 | T0：`cargo test -p shell-win` lib 12 + smoke 21 = 33 passed/0 failed；T1：`gates check` **FAIL 仅 rust-tests**（task-37 WIP game-shell `E0063/E0432/E0433` 编译失败；docs-links PASS 326 md/373 links；ratchet 366/373 WARN 亦为 task-37 新增测试）——**均非本卡引入** | ⚠️ 见 E1 |

## 2. E3 的独立视觉等价证据（关键）

- **代码路径**：`present()` 现在只 `core.present()`（重合成上一帧 view）；唯一上屏在 `refresh_view()`（`set_frame_view(最新 view)`→`recompose`→`blit` 1 次）。因此**最终上屏内容**=最新 view 合成结果，与旧口径第二次 blit 相同；旧口径的第一次 blit（上一帧 view）被移除。
- **自跑 GUI（真实 winit 窗口）**：用 `PrintWindow` 抓取 Title 态，SHA256=`8AFF2339D3CDC8FB2F5BEAF4C003578E93EB381F82D583F44F90080187C12643`，
  与实施方 `target/gui-shellwin-v2-e3.png`（收口后）及 `target/gui-shellwin-v2-printwindow.png`（收口前）**SHA 完全相同**。
  ⇒ E3 前后**逐字节相同**的窗口画面（Title 态确定性画面），独立佐证“E3 不改显示结果”。
- selftest `display_signature` 不变（`0xe99af5849b632fbd`）⇒ headless 合成结果不变（与 GUI 截图互证）。

## 3. 重算数字/签名表（时点声明）

| 指标 | 收口前（task-12 复核） | 本卡复核 T0（隔离时点） | 一致 |
|---|---|---|---|
| `cargo test -p shell-win` | 12+21=33 | 12+21=**33 passed / 0 failed** | ✅ |
| selftest stdout ×2 | 一致 | **SHA256 相等**（F4A50EDE…） | ✅ |
| host_signature | 0x5d7156266d6ac19d | 同 | ✅ |
| canvas_signature | 0x963ac9a776a57625 | 同 | ✅ |
| display_signature | 0xe99af5849b632fbd | 同（E3 不改合成） | ✅ |
| view_signature | 0xf6603346bf818ffd | 同（E4 仅注释） | ✅ |
| GUI Title 截图 SHA | 8AFF2339…（收口前） | **8AFF2339…**（本卡独立抓取，T0 二进制） | ✅ |
| presents 计数 | 300（=逻辑帧） | 300 | ✅ |
| workspace 测试计数 | 366（T0 前） | T1=373（task-37 WIP 新增 7）——**非本卡** | ⚠️ 时点 |

## 4. findings 四分类（均非阻塞）

### 4.1 证据不足（E）

- **E1（时点/并发，Low）**：task-36 §8.5 自报“`cargo test --workspace` exit 0 全绿、`gates check` CHECK: PASS”，
  在**当前工作树（T1，含 task-37 WIP）不可复现**：game-shell 编译失败（`E0063/E0432/E0433`，新 `fire.rs/shape.rs` 接口未完成）
  ⇒ rust-tests FAIL、ratchet 373>366 WARN。属并发他卡，**不归因本卡**；T0 隔离时点本卡单 crate 测试+selftest 已复现。
  建议：Lead 以精确 pathspec 提交这 4 文件后（或 task-37 收口后）跑一次 gates 记录“clean HEAD”时点结果；
  本卡未新增测试 ⇒ **不需要**为 task-36 上调基线（373 的基线属 task-37）。
- **E2（环境/方法，Info）**：为取得“干净树 + 仅 task-36 改动”的 workspace/gates 证据，我尝试 `git worktree` 复制
  HEAD 并覆盖 4 文件（共享 `target/`），被并发 cargo 锁/文件占用污染（`failed to remove shell-win.exe: os error 5`），
  该尝试**失败即弃**，未产出 clean-tree 数字；已改用 T0 隔离时点 + 代码/diff/截图三方证据。建议后续复核用独立
  `CARGO_TARGET_DIR` 避免共享 target 污染。

### 4.2 实现错误（B）

**无。** E3 仅移动平台上屏调用点（同一采样最终内容不变，GUI 截图逐字节相同、四签名不变）；E4 仅注释；E5 仅文档；
未改游戏行为/未证域/签名语义。

### 4.3 规格缺口（S）

- **S1（Info，性能）**：`present()` 仍会 `core.present()`→`recompose`（上一帧 view），随后 `refresh_view` 再 `recompose`
  一次 ⇒ **每采样 2 次 recompose**（softbuffer present 已降为 1 次）。属小 CPU 开销、无功能影响；建议后续若需优化，
  在保证 `FrameRenderer::present` 语义（headless 测试/计数）前提下让窗口模式跳过第一次 recompose。**本卡不必改**。
- **S2（Info，口径）**：“每采样 1 次 softbuffer present”精确指 `frames>0` 的采样；`resumed` 首帧另有 1 次初始
  `refresh_view`（启动即显示 Boot 角标/HUD，v2 既有行为）。文档未提首帧特例，不影响验收；建议如追求严格可加半句。

### 4.4 平台差异（P）

- **P1**：GUI 复验依赖桌面会话；本机 `SendKeys`/前台授权失败、`PostMessage` 可用（E6 已如实记录）。
- **P2**：并发 task-37 WIP 使 workspace/gates 时点漂移（E1/E2）；建议 Lead 提交顺序：先 task-36（4 文件，precise pathspec），
  再 task-37；两条线各自跑 gates 记录时点。

## 5. 命令与输出摘要

| # | 命令（时点） | 输出摘要 |
|---|---|---|
| 1 | `git diff --stat`（T0） | 仅 4 文件：lib.rs +3（纯注释）/ main.rs +12−3 / plan +41 / spec +8 = +58/−6 |
| 2 | `cargo test -p shell-win`（T0） | lib 12 passed + smoke 21 passed，0 failed |
| 3 | `cargo run -q -p shell-win -- --selftest-frames 300` ×2（T0） | exit 0；stdout SHA256 相等；四签名见表 §3（不变） |
| 4 | 代码路径核对（HEAD b92139a + 4 文件） | `present()` 无 blit；`blit()` 唯一调用点=refresh_view；refresh_view 调用点=resumed 初始 + `frames>0`；frames==0 路径与旧一致 |
| 5 | `git diff crates/shell-win/src/lib.rs` | 新增行全部为 `//` 注释（E4 登记）⇒ 摘要/绘制逻辑零改动 |
| 6 | 独立 GUI 截图（T0 二进制 + PrintWindow） | Title 态 SHA256 `8AFF2339…`，= 实施方 e3/收口前截图 SHA ⇒ 画面逐字节相同 |
| 7 | `python gates/cli.py check`（T1，含 task-37 WIP） | `CHECK: FAIL` 仅 rust-tests（game-shell `E0063/E0432/E0433` 编译失败，task-37 WIP）；docs-links PASS；ratchet 366/373 WARN（task-37）——**不归因本卡** |
| 8 | `git diff --stat crates/shell-win` | 仅 lib.rs/main.rs 2 文件（无测试文件改动）⇒ 本卡测试计数贡献 0 |

## 6. 放行判断

- E3/E4/E5/S1-S2/E6 收口**全部落实且与实现一致**；无实现错误、无签名变化、无像素变化（GUI 截图 SHA 逐字节相同 + 四签名不变）。
- **建议 Lead 以精确 pathspec 提交 4 文件**（不要把 task-37 的 game-shell WIP 卷入；当前混合树因 task-37 无法通过
  workspace 编译），提交后跑一次 gates 记录 clean-HEAD 时点；task-36 不新增测试 ⇒ 无需为其上调基线（373 属 task-37）。
- verdict=pass：本卡可转正；P1/E1/E2 为流程/环境项，不阻塞。

> 本复核只写本文件；未改实现/被审文档，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
