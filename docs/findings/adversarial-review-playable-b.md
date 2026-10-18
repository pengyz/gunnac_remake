# 对抗复核：可玩闭环 B（shell 侧 FrameView 渲染）—— 证伪立场独立复核

> 复核方：`shell-verifier`（task-12，verifier 角色）；被审：task-10 / `shell-porter`
> 交付 commit：`a630666`（方案）、`4a0672b`（实现）、`703130a`（§8 回填）；
> 复核基线：HEAD=`1700934`（复核中由 `703130a` → task-29 `1700934` 推进，`crates/shell-win/**` 自 `4a0672b` 起未变）。
> 纪律：[AGENTS.md](../../AGENTS.md) §5 独立对抗复核（不采信自述、自己重导证据）。
> **verdict：pass**（无实现错误；非阻塞 findings：E1 时点证据（Lead 已裁定）、E2 spec §4 回填未交付（阻塞已解除、需补 docs commit）、E3/E4 呈现低效/摘要哨兵、E5 HUD 可见性未证、E6 复核方法提示）。

## 0. 复核独立性 / 边界

1. 先读实现与 diff → 自跑测试/自检 → 自行 GUI 注入 + 截图像素采样 → 最后才比对 §8 自述。
2. 未改实现/被审文档，未 `git add`/`commit`；唯一写入=本文件；探针/截图在 `target/review-playable-b/`（可再生、不入库）。
3. **未覆盖**：L4 像素一致（本卡明确不声称）；音频/模式时长；`--selftest-frames` 之外的 GUI 多分辨率（黑边/缩放仅单测覆盖，本次 528×519 窗口恰为 2× 无黑边）。
4. 复核期并发：HEAD 由 `703130a` 推进到 `1700934`（task-29 B1 修复 + 基线 366）；工作区另有他卡未提交 docs 改动。

## 1. verdict 与最小必查集

| # | 必查项 | 独立证据 | 结论 |
|---|---|---|---|
| 1 | FrameView 每图层只画已证字段 | 逐层代码核对 + 自跑测试（见 §2 表）；`position=None` 不画、`type_code` 不反推坐标、子弹不画、HUD 占位编码、模式角标占位、无字形/文本 | **确认** |
| 2 | 呈现正确性 | `blit_scaled`/`view_rect` 在 `4a0672b` 中**无改动**（v1 语义保持），v1 黑边/2×/3× 测试全过；覆盖层写 256×240 RGBA 后按原 blit 上屏 | **确认** |
| 3 | view 摘要独立于 `Host::signature()` | `frame_view_digest` 只读 FrameView 字段（无 renderer 尺寸/缩放/present 计数）；`4a0672b` 未改 game-shell，`Host::signature()` 语义不动；selftest 新增独立行 | **确认** |
| 4 | selftest ×2 一致 | 自跑两次 stdout **SHA256 相等**（`F4A50EDE…D447B`）；`view=0xf6603346bf818ffd`、`display=0xe99af5849b632fbd`、`canvas=0x963ac9a776a57625`、`host=0x5d7156266d6ac19d` 与 §8 声明逐项一致（HEAD 1700934 复核） | **确认** |
| 5 | smoke 新增测试真断言 | `4a0672b` smoke +2 条均为像素级断言（模式角标色/敌槽 Some/None 门/HUD 位序/“子弹计数变化不改像素”）+ 1100 帧模式推进集成测试；自跑 21 smoke 全过（非空跑） | **确认** |
| 6 | 未证域 TODO / 不夸大“可玩” | `TODO(shell-v2)` 共 7 处（lib 6/main 1）；plan §0 明写“不声称像素级一致/原版 HUD 一致/可玩性达成”；§5 未证域表；§8.3 逐项落实；未见“可玩/原版像素一致”夸大 | **确认** |
| 7 | 写范围/纪律 | `4a0672b` 仅 `crates/shell-win/{src/lib.rs,src/main.rs,tests/smoke.rs}`；`a630666`/`703130a` 仅 `docs/findings/plan-shellwin-v2.md`；**未触** game-core/game-shell/game-platform；均含 Evidence+Confidence（88/88/92） | **确认** |
| 8 | 测试数/门禁 | `cargo test -p shell-win` lib 12 + smoke 21 = **33 passed/0 failed**；`cargo test --workspace` exit 0（365 passed + 1 既有 ignored = 366）；`gates check` **CHECK: PASS**（PASS=11 FAIL=0 SKIP=1 TOLERATED=0、rust-tests 366/366 OK，Lead `9405a35` 已上调基线） | **确认** |

## 2. 渲染真实性逐层核对（抽查 8 处，文件中行号为 HEAD 1700934）

| # | 图层 | 代码事实 | 判定 |
|---|---|---|---|
| 1 | 模式画面 | `draw_frame_view_overlay` 用 `mode_tag(view.mode)` → `MODE_MARKER_COLORS` 画 8×8 角标（mode_tag 0..=13 ↔ 14 色表逐值核对一致）；无文字/美术/布局 | ✅ 占位可见区分（`lib.rs:587-608/724`） |
| 2 | 玩家 | 仍由 game-shell `compose_frame` 单一绘制源；shell 不重复画 | ✅ 无双绘制源 |
| 3 | 敌槽 | `if let (Some(pos), Some(type_code))` 才 `fill_block(pos)`；`Host::frame_view` 运行态 `position` 恒 `None`（task-11/复核探针已证）⇒ **运行态 20 槽不画**；`type_code` 仅作占位色索引 `&3`，**无任何由 type 推坐标的代码** | ✅ 门控正确（`lib.rs:731-735`） |
| 4 | 玩家子弹 | `live_bullets`/`player_bullets_live` 只进 `frame_view_digest`，overlay 无绘制调用；smoke 断言“仅改计数⇒画布逐字节相同、摘要变化” | ✅ 不画（`lib.rs:736-737`） |
| 5 | HUD 金钱 | `money_digits` 逐桁按 `(digit, show)`：show=false 跳过；digit 4bit→2×2 点阵（bit0 左上/bit1 右上/bit2 左下/bit3 右下），固定占位槽；**非原版字形** | ✅ 占位编码（`lib.rs:640-660`） |
| 6 | HUD 炸弹/关号 | `bomb_ammo` 8bit 位行、`stage &0x0F` 4bit 位行，固定占位槽、占位色；**非原版图标/布局** | ✅ 占位编码（`lib.rs:663-668`） |
| 7 | 合成顺序 | `compose_canvas_with_view` = v1 合成（CHR 背景+非清屏 game-canvas 像素）+ overlay；覆盖层后画（可覆盖 CHR/玩家角部，占位可接受） | ✅ 语义清楚（`lib.rs:741-745`） |
| 8 | view 摘要 | `frame_view_digest` 仅 FNV over FrameView 全字段；不混平台态；`Host::signature` 未动 | ✅ 独立（`lib.rs:687-714`） |

**GUI 独立验证（非自述）**：自写 `PostMessage(WM_KEYDOWN)` 注入真实 winit 窗口（本机 SendKeys/前台授权失败，PostMessage 可用）：
`Boot@1 → Black@183 → Title@363 → Menu@737 → Shop@853 → ShopInfo@964 → Transition@1107 → StageStart@1141`
（Transition→StageStart=34，≈33f 常数；与 task-11 独立复核一致，模式派发在 game-shell、shell 只透传）。
PrintWindow 截图逐像素采样（`target/review-playable-b/gui-v2b-printwindow.png`，528×519，客户区 512×480=2×）：
- 左上 8×8 角标 = **RGB(128,0,0)** = `MODE_MARKER_COLORS[7]`（StageStart）✅；
- HUD 炸弹 `bomb_ammo=4`（$018F 初值）→ bit2 白块出现在 canvas(12,216)（=8+2×2）**RGB(255,255,255)** ✅，bit1 位置为背景/黑（未画）✅；
- HUD 关号 `stage=1` → bit0 `GREY4[2]` 灰块在 canvas(28,216)=**RGB(170,170,170)** ✅，bit1 位为 CHR 背景白（非占位灰）✅；
⇒ 模式角标 + HUD 占位编码在**真实窗口**中按预期渲染（此前 §8.2 称 GUI 键推进只能手动，本复核证明可自动注入验证——见 E6）。

## 3. 被确认清单（含重算数字/签名表）

| 指标 | 自述（703130a §8 / 4a0672b msg） | 我实测（HEAD 1700934） | 一致 |
|---|---|---|---|
| `cargo test -p shell-win` | lib 12 + smoke 21 = 33 | 12 + 21 = 33，0 failed | ✅ |
| `cargo test --workspace` | 全绿 | exit 0；365 passed + 1 ignored = 366 | ✅ |
| gates check | PASS | `CHECK: PASS`，PASS=11/FAIL=0/SKIP=1/TOLERATED=0，ratchet 366/366 OK | ✅ |
| selftest ×2 | stdout 逐行一致 | **SHA256 相等**（F4A50EDE…） | ✅ |
| host_signature | 0x5d7156266d6ac19d | 同（HEAD 1700934，task-29 已入库） | ✅ |
| canvas_signature | 0x963ac9a776a57625（不变） | 同 | ✅ |
| display_signature | 0xe99af5849b632fbd（v2 覆盖层后） | 同 | ✅ |
| view_signature（新增） | 0xf6603346bf818ffd | 同 | ✅ |
| 模式序列（1100 帧集成） | Boot→…→Combat（mode_tag 0..8） | smoke 测试通过；GUI 实测同序列至 StageStart | ✅ |
| GUI 截图 | Title 黄色角标/CHR/HUD 位行 | 自截图：StageStart 暗红角标 + HUD 白/灰块位置颜色逐一匹配 | ✅ |
| 测试数基线 | 366（Lead 处理） | `9405a35` 已上调，状态 OK | ✅ |

## 4. findings 四分类（均非阻塞）

### 4.1 证据不足（E）

- **E1（Lead 已裁定，非阻塞）commit 级证据时点**：`4a0672b` message 的 `host_signature=0x5d7156266d6ac19d`
  来自**工作树未提交的 task-29（B1 修复）后**时点，在 `4a0672b` 树（不含 B1）上应为 task-9 值
  `0xf51d5a517e7458b7` ⇒ 该 commit 证据在 commit tree 不可复现。**补救评估**：`703130a` §8.2 已注明
  “HEAD 4a0672b + 并发 task-29 工作树”并给出值链（task-1 `0xcde9…` → task-9 `0xf51d…` → task-29 `0x5d71…`、非回归），
  补救**充分**（值链+时点透明）；且 task-29 已以 `1700934` 入库，当前 HEAD 实测复现 `0x5d71…` ⇒ **值链已自洽**。
  建议：最终闭环汇报统一“HEAD 实测 + 值链来源”时点口径（Lead 已承诺）。
- **E2（AC 未完全交付，阻塞已解除）spec §4 回填缺失**：task-10 写范围/验收含
  `docs/spec/platform-shell.md` §4 状态回填（plan §6:118），但两 commit 均未包含（`703130a` §8.4 明确 defer 至
  task-29 入库）。当前 task-29 已提交 ⇒ 应**立即补 docs(spec) commit**（§4.1 渲染内容/自检行 + §4.2 新 selftest 值 +
  §8 未证域补“HUD 可见性/占位呈现不声称原版”）。不阻塞代码处置与可玩性验收，但闭环前需收口。
- **E3（呈现低效，Low）双 blit/present**：`Host::tick` 内 `present()` 已 `blit`（此时 view 为上一帧），
  `main.about_to_wait` 随后 `refresh_view` 再 `set_frame_view`+`blit` 一次 ⇒ 每采样 softbuffer present **两次**
  （第一次含上一帧覆盖层、随即被第二次覆盖）。plan §3 已承认“按样本收敛”，无可见错误；建议 `WindowRenderer::present`
  在窗口模式跳过 blit（统一由 `refresh_view` 上屏），或后续 game-shell 允许 tick 前取 view。位置：`main.rs:88-111`、`main.rs:275-280`。
- **E4（摘要哨兵，Low）**：`frame_view_digest` 对 `position=None` 用 `(0xFF,0xFF)` 哨兵，与未来真实坐标
  `Some((255,255))` 存在摘要歧义（v1 None 恒真，无实际影响）；建议加 1B tag 区分 None/Some。位置：`lib.rs:705-710`。
- **E5（未证域登记缺口，Low）HUD 可见性/模式门未登记**：overlay 在 **所有模式**（含 Boot/Title/Menu/Shop）画 HUD 占位；
  原版 HUD 的**显示时机/可见性**未证（plan §5 只登记“字形/位图/布局/坐标未证”）。建议 §5 增一行“HUD 可见性/模式门未证（v2 恒画占位）”，
  或保守地仅在已证 gameplay 段画；同时注意不得据此写渲染结论。位置：`lib.rs:717-720`、plan-shellwin-v2.md §5。
- **E6（复核方法提示，Info）GUI 键推进可自动化**：§8.2 记“SendKeys 未送达 winit ⇒ GUI 键推进手动验收”；
  本复核用 `PostMessage` 注入成功并截图像素验证（见 §2），说明该链**可自动验收**（SendKeys 失败是本会话前台授权限制，非实现问题）。
  建议下轮把 GUI 打点验收改为 PostMessage 脚本 + 像素采样（可入 task 卡验收命令）。
- **E7（提示，Info）模式画面最小化**：模式“画面”实际=8×8 角标（非全屏占位）；文档如实写“角标/占位”，无夸大；
  若 Lead 期望更明显模式区分可另卡扩展（不属于本卡缺陷）。

### 4.2 实现错误（B）

**无。** 渲染门控（position/type 均 Some 才画、None 不画、type 不反推坐标）、子弹不画、HUD 占位编码、
模式占位色、view 摘要独立性、blit/缩放语义均经代码+自跑测试+GUI 像素采样确认；未发现空跑测试、伪造 Some 路径或字形/文本/图样发明。

### 4.3 规格缺口（S）

- **S1**：`docs/spec/platform-shell.md` §4 尚无 v2 渲染状态（同 E2）；§8 未登记“HUD 可见性/模式门”（同 E5）。
- **S2**：plan §3 的“同一采样点两次上屏”属已登记口径，但未量化性能影响（E3）；建议在规格/方案中标注“GUI 每采样 2 次 softbuffer present（临时）”。

### 4.4 平台差异（P）

- **P1**：本机 GUI 自动化受 Windows 会话限制：`SendKeys`/`SetForegroundWindow` 失败，`PostMessage` 可用；GUI 偶发窗口不推进
  （本轮 3 次尝试 1 次全链成功；selftest/replay 稳定）。属验证环境差异，非实现缺陷；建议验收脚本用 PostMessage+超时重试。
- **P2**：本次截图窗口 528×519（客户区 2× 整窗）未覆盖黑边/非整数倍窗口；缩放/黑边由 v1 单测（`view_rect`/`blit_scaled`）锁定，
  `4a0672b` 未改该代码 ⇒ 视为保持，未做 v2 多分辨率 GUI 实测。

## 5. 命令与输出摘要（自跑）

| # | 命令 | 输出摘要 |
|---|---|---|
| 1 | `cargo test -p shell-win` | lib 12 passed；smoke 21 passed；0 failed |
| 2 | `cargo test --workspace` | exit 0；365 passed + 1 ignored（engine_tick 既有）=366 |
| 3 | `cargo run -q -p shell-win -- --selftest-frames 300` ×2 | exit 0；stdout SHA256 相等；`host=0x5d7156266d6ac19d canvas=0x963ac9a776a57625 display=0xe99af5849b632fbd view=0xf6603346bf818ffd` |
| 4 | `python gates/cli.py status` / `check` | `STATUS: OK`；`CHECK: PASS`；rust-tests 366/366 OK；docs-links PASS（324 md/369+ 链接） |
| 5 | `git show --name-only 4a0672b 703130a a630666` | 仅 shell-win 3 文件 / plan-shellwin-v2.md；无 game-core/game-platform/game-shell |
| 6 | `git diff 4a0672b..HEAD -- crates/shell-win` | 空（shell-win 自实现 commit 未变） |
| 7 | 自建 GUI 注入（PostMessage Enter×2/X×2）+ PrintWindow 截图 | 模式序列 Boot→…→StageStart（见 §2）；角标 RGB(128,0,0)、HUD 白/灰块像素采样匹配 |
| 8 | 实施方截图 `target/gui-shellwin-v2-printwindow.png` 复核（Title 态） | 标题 `gunnac-remake (Stage A)`、CHR 背景、左上黄色角标=Title 占位色（`MODE_MARKER_COLORS[2]`）、HUD 占位块可见（裁剪区已核） |
| 9 | `git grep -c 'TODO(shell-v2)'` | lib.rs 6 + main.rs 1 |

> 本复核只写本文件；未改实现/被审文档，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
