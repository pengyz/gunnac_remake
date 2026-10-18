# 对抗复核：shell 侧玩家子弹渲染（task-44）—— 证伪立场独立复核

> 复核方：`shell-verifier`（task-65，verifier 角色）；被审：task-44 / `shell-porter`
> 被审实现：commit `3dc4333`（`feat(platform): shell-win 渲染玩家子弹`；在 `5a60ff8` 树内）
> 被审文档：`docs/findings/plan-shellwin-v2.md` + `docs/spec/platform-shell.md` 回填（复核时已由 `fb08c39` 提交；
> 与 Lead 最初告知的 staged 版本内容相同）
> 独立验证树：**干净 HEAD worktree** `5a60ff8`（`%TEMP%\gunnac_wt_65`）+ 独立 target `target/verify-65`；
> 工作树中 `crates/game-shell/src/{fire,render}.rs`、`docs/findings/plan-bullets-hit.md` 改动属其它卡（task-58/63 隔离，未混入）。
> 纪律：[AGENTS.md](../../AGENTS.md) §5（证伪立场；自跑门禁/自抓 GUI）
> **verdict：pass**（①-⑤ 全部独立证实；GUI 由我自跑 PostMessage+PrintWindow 复现，非采信实施方截图。
> 另附 2 Low + 4 Info 改进项，均不阻塞交付：集成测试对照污染、plan §8.2/§8.3 陈旧行、commit 文案计数/typo、
> 基线 ratchet PENDING、smoke 4×4 只查对角、实施方截图时点在 commit 前。）

## 0. 复核方法

- **实现真实性**：读 commit diff（`lib.rs` 108 行 / `main.rs` 8 行 / `smoke.rs` 142 行）+ 独立重跑测试。
- **干净树**：`git worktree add --detach %TEMP%\gunnac_wt_65 5a60ff8`，`CARGO_TARGET_DIR=target/verify-65`，
  排除主工作树并发改动（task-58 等）。
- **GUI 自跑**：干净树二进制 + `PostMessage(WM_KEYDOWN)` 注入真实 winit 窗口 + `PrintWindow` 截图 + 逐像素统计
  （本会话 `SendKeys`/前台授权仍失败，PostMessage 可用——与既往 task-12 复核一致）。
- 只写本文件；未改被审文件/资产，未 `git add`/`commit`；GUI 截图/日志在 `target/review-bullet/`（可再生不入库）。

## 1. 必审点逐条（①-⑤）

| # | 要求 | 独立验证 | 结论 |
|---|---|---|---|
| ① | 只画**已证坐标面**；图样/颜色未证⇒占位块+TODO，不发明弹型视觉；敌/HUD 不回归 | `lib.rs:786-795` 绘制循环只消费 `view.player_bullets`，坐标=`BulletView.x/y`（game-shell `fire.rs:67-77` 注释：`x=$04EE+Y`、`y=$055E+Y`、`type_code=$063E+Y` A 地址面；`player_bullet_views` 存活=弹网格空槽过滤）；**shell 不读 ROM/RAM、不按 type_code 选图**（grep `0x04EE/0x055E/prg_bank/rom[` 均无绘制侧引用）；`PLAYER_BULLET_PLACEHOLDER_SIZE=4`/`RGBA=[0x00,0xE0,0x40]` 均注"非原版弹型/尺寸/调色板`TODO(shell-v2)`"；`main.rs` 仅注释改（无代码）；敌块/HUD 绘制顺序与旧行为同（bullet 循环画在所有既有块之后，运行态敌槽 position 恒 None） | ✅ |
| ② | smoke 有"子弹数量/坐标变化→画布像素"真断言（构造对照） | `tests/smoke.rs:413-477` `player_bullets_draw_at_proven_coordinates_and_count_change_changes_pixels`：0 弹=背景色（CLEAR）、1 弹坐标处=占位色、4×4 验、多弹各自坐标、无弹处=背景、边缘 (255,239) 裁剪不 panic；`assert_ne!(r_empty.composed_canvas(), r_one...)`（**两个 renderer 同默认背景**，对照有效）；摘要敏感性另测。集成测试 `menu_dispatch...:495-588` 真实 Host 到 Combat 后按 B：`view.player_bullets` 非空、弹坐标处=占位色、去弹后该像素≠占位色、画布变化（**该对照有污染，见 E1**） | ✅（E1 Low） |
| ③ | view 摘要纳入 player_bullets（独立于 Host::signature）；`view_signature=0x15dbc2c7b0ba0a5d` 两次一致；host/canvas/display 不变 | 干净树 `cargo run -p shell-win -- --selftest-frames 300` ×2：stdout **逐字节一致**（SHA256 `FA5E87A3…`）；四值=`host=0x68547665bf1ab396`（task-37 语义）/ `canvas=0x963ac9a776a57625` / `display=0xe99af5849b632fbd` / **`view=0x15dbc2c7b0ba0a5d`**；与 spec §4.2 task-37 历史行（前 3 值）+ task-44 更新行（view）**逐值对上**。digest 代码 `lib.rs:725-746` 只对 FrameView 字段做 FNV（20 槽 None/Some tag+slot/x/y/type_code；4 条命中事件 tag+enemy_slot/bullet_slot/damage/fail_closed/frame），无平台态、未改 `Host::signature()`（本 commit 未触 game-shell） | ✅ |
| ④ | GUI PrintWindow 抽样真出现子弹像素（自跑；角标/HUD 不变） | **自跑成功**（干净树二进制，postMessage 自适应键序）：stderr 实测模式链 `Boot→Black→Title→Menu→Shop→ShopInfo→Transition→StageStart→Combat`（Combat 达于 frame 915）；Combat 基线截图=**0** 个子弹占位色像素；按住 B（X 键）2.4s 后截图=**128px** 占位色（=2 弹×8×8@2×），bbox `(264,153)-(271,286)`，列 264-271 各 16 行；**基线↔子弹帧逐像素 diff=128，全部为"新增绿色"、非预期变化 0**⇒模式角标(16×16 白=Combat)/HUD/CHR 背景/game 画布全部不变。截图 `target/review-bullet/gui3-{baseline,bullets}.png` | ✅ |
| ⑤ | 写范围 & 文件清单 & 文档归属 | `3dc4333` 文件仅 `crates/shell-win/{src/lib.rs,src/main.rs,tests/smoke.rs}`（+237/−21），与 message "仅改 crates/shell-win/**" 一致；`2bcb92a..HEAD` 对 `game-core/game-platform/game-shell/rom/reference/data` **零 diff**；文档回填=`fb08c39` 仅 plan-shellwin-v2/platform-shell 两文件，属 task-44（先 staged、复核期间已提交），内容与实现一致（§2/§3/§4/§5/§8.6 + spec §4.1/§4.2/§4/§9） | ✅（E2 陈旧行 Low） |

## 2. 被确认清单（可复算）

| # | 结论 | 证据 |
|---|---|---|
| C1 | 干净树 `cargo test -p shell-win` = lib **14** + smoke **22** = **36 passed / 0 failed** | `target/review-bullet/clean-test-shellwin.log` |
| C2 | `cargo test --workspace` exit 0 | `clean-test-workspace.log` |
| C3 | selftest 300f ×2 stdout 逐字节一致；四签名见 ③ | SHA256 `FA5E87A3792FD410…`；输出原文已存 `clean-selftest-1/2.out` |
| C4 | 干净树 `gates status`：PASS=11 FAIL=0 SKIP=1(TOLERATED=0)，`STATUS: OK`；docs-links PASS（336 md/430 links）；rom/seed/naming/knowledge/rust-tests/subroutine/trace/visual/completeness 全 PASS | `gates status` 全表；仅 ratchet `rust_test_count 377→380 PENDING`（见 E4） |
| C5 | 子弹坐标=已知 A 地址面且只经 FrameView 传递，shell 不反推 | `game-shell/fire.rs:67-77/277-285`；`lib.rs:786-795` |
| C6 | 占位尺寸/颜色显式标注非原版 + `TODO(shell-v2)`；不按 type_code 视觉区分 | `lib.rs:640-644/762-764/786-794`；docs plan §8.6/spec §4.1 同步 |
| C7 | 敌/HUD 无回归：既有 `frame_view_overlay_renders_mode_hud_enemy_placeholders_only` 等 21 条旧 smoke 仍全过；我 GUI 基线/子弹帧非绿色像素差=0 | 测试日志 + §1④ |
| C8 | 实施方 GUI 截图（`target/gui-shellwin-v2-bullets.png`）独立复算：128 px、cols 264-271、bbox `(264,99)-(271,232)`、client origin (8,31)、左上 16×16=白（Combat 角标）——与 message 声明一致（截图时点早于 commit，见 E6） | 逐像素统计 |

## 3. findings 四分类

### 3.1 实现错误（B）——无

### 3.2 证据不足 / 测试/文档缺陷（E）

- **E1（Low）集成测试的"去弹对照"被背景污染**：`smoke.rs:508` 对主 `renderer` 调了 `set_chr_rom(rom)`，
  而 `:576` 的 `renderer_no_bullets = FrameRenderer::new()` 未设 CHR ⇒ 两者合成背景不同（CHR 灰阶 vs 黑+清屏色），
  使 `"去弹后不再为占位色"`（:578-582，背景为黑，恒≠绿）与 `"画布变化"`（:583-587，背景差异恒真）**不能独立证明去弹生效**。
  建议：`renderer_no_bullets.set_chr_rom(rom)` 后重合成，或直接复用同一 renderer 依次 `set_frame_view(view)`/`set_frame_view(view_no_bullets)` 比较。
  （主断言"弹坐标处=占位色"仍是真证据；单元测试对照有效，故仅测试强度问题，不影响实现正确性。）
- **E2（Low）文档陈旧行未随 task-44 收口**（`fb08c39` 只追加 §8.6/§2/§3/§4/§5，未标注 §8.2/§8.3）：
  plan §8.2:175 仍写"**1100 帧**按键透传"（实际 smoke 已扩展 **1400** 帧）；§8.2:176/177 的
  `view_signature=0xf660…`/`host=0x5d71…` 为 task-10/29 时点值；§8.3:183-185 仍写"**玩家子弹不画**
  （仅 `view_digest` 登记计数，集成测试断言'计数变化不改像素'）"——与现行实现/测试**相反**。
  plan 头注允许 §0 保留历史主张，但 §8 是状态回填节，建议加一行 dated 注（"task-37/task-44 后以 §8.6 为准"）或更新 §8.2/§8.3 行。
  （spec/platform-shell 用 dated note 链已覆盖；plan §8.6 本身正确。）

### 3.3 规格缺口（S）/ Info

- **E3（Info）commit 文案与实测计数不符**：`3dc4333` message 写"lib **+3**（子弹坐标/裁剪/摘要敏感命中）；smoke +1"，
  实测新增 lib **+2**（12→14：`player_bullet_overlay_draws_placeholder…`、`view_digest_covers_bullets_and_hits_fields`）
  + smoke **+1**（21→22）；总数 +3 与 message 的"14+22=36"及 plan §8.6"lib 2+smoke 1"一致 ⇒ 仅 message 的"lib +3"笔误。
  另有 message 内 `ype_code`（应为 `type_code`）typo（Lead 已裁不改历史，记 Info）。
- **E4（Info）ratchet bookkeeping**：干净树 `gates status` 显示 `rust_test_count` 377→**380 PENDING**（+3 测试未更新基线）；
  `gates check` 仍 PASS（PENDING 非 FAIL），与 message "gates check PASS" 不矛盾；基线更新属 Lead/captain 的 `data/status` 范围（task-44 写范围禁止），建议 milestone 时 `baseline update --reason` 收口。
- **E5（Info）smoke 的 4×4 断言只查对角 4 点**（`:431-437` `for d` 只用 `(40+d,50+d)`）；lib 单测已用 dy/dx 双重循环验全 16 px（`:1309-1317`），故无覆盖漏洞，仅 smoke 断言可顺手加强。
- **E6（Info）实施方 GUI 截图时点**：`target/gui-shellwin-v2-bullets.png` mtime **14:58:09** 早于 commit **15:04:55**（开发树二进制所摄）。
  我方已在干净 HEAD 树独立复现同类证据（128 px/列 264-271/仅绿色新增），故不影响结论；建议未来证据截图注明"对应 commit/binary SHA"。
- **E7（Info）plan §0 历史目标行**（"玩家子弹不画（FrameView 无坐标面…）"）未改写——符合该文档"不改写历史主张"头注，且 §2 已加新行；不判缺陷，仅提示读者以 §2/§8.6 为准。

### 3.4 平台差异（P）

- **P1（环境，非缺陷）** GUI 自动化：本会话 `SendKeys`/前台授权失败，`PostMessage(WM_KEYDOWN)` 可用（与既往一致）；
  且**按键注入需按 host 帧时序自适应**（墙钟固定延时会在每秒误差下错过菜单 latch——我第一版脚本即卡在 Menu，改为"轮询 stderr 模式链→按模式注入"后成功到 Combat frame 915）。
  建议把该自适应脚本作为 GUI 验收模板（可入 task 卡验收命令）。
- **P2（环境，非缺陷）** 干净 worktree 构建的 exe 资产根=`%TEMP%\gunnac_wt_65`（dev 构建按 manifest 目录回退），ROM 存在于 worktree 内；
  不影响像素验证（资产/ROM 与主树同内容、rom-integrity PASS）。

## 4. 命令与输出摘要（自跑）

| # | 命令/操作 | 输出摘要 |
|---|---|---|
| 1 | `git worktree add --detach %TEMP%\gunnac_wt_65 5a60ff8`；`CARGO_TARGET_DIR=target/verify-65 cargo test -p shell-win` | lib 14 + smoke 22 = **36 passed**，TEST_EXIT=0 |
| 2 | `cargo test --workspace`（同上干净树） | **WORKSPACE_EXIT=0** |
| 3 | `cargo run -q -p shell-win -- --selftest-frames 300` ×2 | 两次 stdout SHA256 相同 `FA5E87A3…`；host=0x68547665bf1ab396 / canvas=0x963ac9a776a57625 / display=0xe99af5849b632fbd / **view=0x15dbc2c7b0ba0a5d** |
| 4 | `python gates/cli.py status`（干净树） | PASS=11 FAIL=0 SKIP=1；docs-links PASS（336 md/430 links）；**rust_test_count 377→380 PENDING**；STATUS: OK |
| 5 | GUI：干净树 exe + `PostMessage` 自适应键序 + `PrintWindow` ×2 | 模式链至 **Combat（frame 915）**；基线 0 弹像素；按住 B 后 **128 弹像素**（bbox (264,153)-(271,286)，列 264-271）；基线↔子弹 diff=128 且**全部绿色新增**、非预期 0 |
| 6 | 实施方截图独立复算 `target/gui-shellwin-v2-bullets.png` | 128 px、cols 264-271、bbox (264,99)-(271,232)、client (8,31) 白=Combat 角标——与实施方声明一致 |
| 7 | `git show --stat 3dc4333` / `git diff --stat 2bcb92a..HEAD -- <红线路径>` | 仅 shell-win 3 文件（+237/−21）；红线路径 **零 diff**；docs 回填 fb08c39 仅 2 docs |
| 8 | 读 `lib.rs:725-746/786-795`、`smoke.rs:413-588`、`game-shell/fire.rs:67-77` | digest 字段全含；绘制只读 FrameView；坐标出处 A 地址面；hits 不画布 |

## 5. 修正建议（优先级）

1. **E1（Low，建议本卡后补）**：集成测试去弹对照改用同背景 renderer（`set_chr_rom` 或复用同一 renderer），使 "去弹⇒像素/画布变化" 成为独立断言。
2. **E2（Low）**：plan §8.2（1100→1400、旧签名值）/§8.3（"玩家子弹不画"）补 dated 注或更新，避免读者把旧状态当现状。
3. **E3/E4（Info）**：后续 milestone 时更新 `rust_test_count` 基线（377→380）并在 commit 文案区分 lib/smoke 计数；`ype_code` typo 留档不改历史。
4. **E5/E6（Info）**：smoke 4×4 改双重循环；GUI 证据截图注明 commit/binary SHA。
5. **P1**：把"轮询 stderr 模式链 + 按模式注入"的自适应 GUI 脚本纳入验收模板（本次已实证可用）。

## 6. 复核结论

**pass**：①-⑤ 全部独立证实——渲染只消费 FrameView 已证坐标面（4×4 纯色占位块+TODO，不发明弹型视觉）；smoke/lib 有真实像素断言；
view 摘要完整纳入 `player_bullets`+`hits_this_frame`、`view_signature=0x15dbc2c7b0ba0a5d` 两次逐字节一致、host/canvas/display 与 task-37 基线逐值不变；
**我自跑 GUI（非采信截图）在干净 HEAD 树复现子弹 128px 上屏且基线↔子弹帧非绿色像素差=0**（角标/HUD/背景不回归）；
写范围仅 shell-win + docs 回填，红线零 diff。E1/E2 两条 Low（测试对照/文档陈旧行）建议后续顺手修，不影响本卡交付验收。

> 本复核只写本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
