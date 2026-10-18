# 对抗复核：可玩闭环 A（平台无关侧）—— 证伪立场独立复核

> 复核方：`shell-verifier`（task-11，verifier 角色）；被审：task-9 / `platform-porter`
> 复核基线：task-9 交付 tip=`b94ede8`（代码提交 `96415c1`、`05e3135`；方案 `cce3bb1`）；
> 复核时 HEAD=`167d8a9`（并发任务推进），`git diff b94ede8..HEAD -- crates/game-shell crates/game-data` 为空 ⇒ 结论适用于交付态。
> 纪律：[AGENTS.md](../../AGENTS.md) §5 独立对抗复核（不采信自述、不采复核方复用、自己重导证据）。
> **verdict：pass**（最小必查集全部确认；核心数字独立复现。非阻塞 findings 见 §5，其中 B1 建议下一小卡修复。）

## 0. 复核独立性 / 边界

1. 先读实现 → 自建探针 + 自跑命令复算 → 最后才与 `plan-playable-loop.md` §10 自述比对；自述签名/数字不作输入。
2. 未改实现/被审文档，未 `git add`/`commit`；唯一写入=本文件；探针/日志在 `target/review-playable-a/`（可再生、不入库）。
3. 未覆盖项（明示）：C4 的"槽序列"未做独立重演（只做了槽集合/型集与帧差统计）；**菜单→商店中间动画时长**（167f 量级）无 oracle 证据可对；L4 像素/渲染不在本卡；Linux/Android 未测。
4. 复核期并发：HEAD 由 `b94ede8` → `53aceb8` → `167d8a9`；工作区另有他卡未提交 docs 改动（不影响 game-shell/game-data 代码）。

## 1. 结论与最小必查集

| # | 必查项 | 我的独立证据 | 结论 |
|---|---|---|---|
| 1 | C1 滚动 1207/1207 失配口径真实 | 自跑 replay 测试（3 passed）；另用 Python 从 `game-core::stage::scroll_delta` 公式独立重算 `.scroll.jsonl`：rows=1208（f1193..2400 逐帧连续）、compared=1207、**mismatches=0**。口径=C1 行=公式 vs oracle 逐帧差；未剔窗口（全帧比较） | **真实** |
| 2 | C6 玩家 571/571 一致、剔除声明 | 自建 Rust 探针（不引用测试代码）复跑 3000 帧 replay：compared=571、mismatches=0；trace 分布 955 行 f1260..2400，3 个死亡/重生缺口（README 声明 f1831-1916/f2047-2101/f2131-2175），对比段 f1260..=1830 连续 571 行；段内有 206 个不同玩家状态、499/571 帧 held≠0 ⇒ 非平凡对拍；约定 `state(N)=tick(state(N-1),held(N))` 与 trace README 一致 | **真实**（剔除=只取死亡窗前连续段，已声明） |
| 3 | game-shell 零平台依赖；未改 game-core/game-platform/shell-win | `cargo test -p game-shell`：41 单元 + contract 2 + mock_platform 3 + stg01_replay 3 全绿（contract 依赖白名单=game-core/game-data/game-platform，平台词禁令未放宽）；`git show --name-only 05e3135 96415c1 b94ede8` 无 game-core/game-platform/shell-win 文件 | **确认** |
| 4 | FrameView 未证域诚实、无夸大“可玩” | FrameView 未证坐标 `position=None`（`render.rs` 注释 + `host.rs:822-849`）；compose_frame 仍只画玩家 8×8（敌/弹/HUD/不画）；plan §8/§10 显式列“spawn 帧级对齐/初生记录 loader/敌坐标写者链未证”；platform-shell §4.2 明写“shell-win 渲染仍只画 CHR 背景+玩家块（FrameView 画面=task-10）” ⇒ **未见“可玩画面”夸大** | **确认** |
| 5 | gates 检查 | `python gates/cli.py status` → `STATUS: OK`；`python gates/cli.py check` → exit 0、`CHECK: PASS`、`PASS=11 FAIL=0 SKIP=1 TOLERATED=0`；ratchet 355/355 OK（Lead `c83a19a` 已上调基线） | **确认** |
| 2b | selftest 两次一致 | `cargo run -q -p shell-win -- --selftest-frames 300` ×2：exit 0；stdout **SHA256 相等**（`42D93F9B…ACE24`）；`host_signature=0xf51d5a517e7458b7`（与 task-9 声明一致），canvas/display 不变（`0x963ac9…a57625`/`0xf68f9ffc…07cbad`） | **确认** |

**verdict=pass 依据**：核心数字（C1/C6/replay 确定性签名）全部**独立复现**；GUI 端到端跑通（§3）；gates/tests 真绿；架构与未证边界诚实。非阻塞 findings 集中在“证据表述精度/一个菜单门控边角 bug/CD4 断言未机器化”，不影响 task-10 依赖面（FrameView 字段来源），故判 pass 并列出收口建议。

## 2. 被确认清单（自己重导，关键项）

| # | 声明 | 我的证据 | 结论 |
|---|---|---|---|
| C1 | replay-3000 确定性签名 `host=0x521d397a6c609f34 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807` | 自建探针（独立 `Host` 复跑）逐项相等；测试内两次 run 全等 | 确认 |
| C2 | 调度 golden 43 记录 / tick 锚 [1,33,49,65,81] | replay 测试断言 43；**预置独立来源**：`crates/game-core/tests/engine_tick.rs:82-108` golden 断言 43 记录 + 1/33/49/65/81；`tools/wave_timeline.py sched 1` 直接解 ROM bank8:$A26D 显示 @A26D +1t / @A275 +33t / @A27C +49t / @A283 +65t / @A28A +81t | 确认 |
| C3 | STG-01 装载用已证数据 | `ScriptCursor::stg01`=`from_records(STG01_SCRIPT[14])`（engine.rs:753）；`decode_sched($A26D)`；槽 0 初态 `type=1,x=128,y=160` 与 oracle `enemy.jsonl` f1264 槽0 字节一致（state[0]=01, x[0]=0x80, y[0]=0xA0）；装载帧=1193（trace 起点） | 确认 |
| C4 | 菜单/模式派发只走已证边集 | 所有迁移经 `flow::FlowState::advance`（非法边 fail-closed，单测 `unproven_flow_edge_is_rejected`）；Boot@180/Black@360 帧锚 A；输入边沿锁存 +1f（探针：Start@600→Menu@602）；按住 START 只派发一次（探针 G） | 确认（键序映射为 B 级推断，见 E1） |
| C5 | 无未证发明 | 弹药生成/命中/Boss/敌坐标写者链/初生 loader 均未实现（H-01/H-16/Boss 不 spawn；spawn 只消费 `land_pair`/`land_leader`，其余 SchedItem fail-closed）；渲染仍只玩家块 | 确认 |
| C6 | GUI 可从 Boot 推进到战斗 | PostMessage 注入真实 winit 键盘消息实测：`Boot@1→Black@181→Title@361→Menu@586→Shop@695→ShopInfo@803→Transition@944→StageStart@977→Combat@1217`；Transition→StageStart=33f、StageStart→Combat=240f、出店 B→Transition=+32f（与声明常数一致）；标题锁窗内首按 Enter 被忽略（A 级 (450,550] 行为在 GUI 可观测） | 确认 |
| C7 | 规模/门禁 | `cargo test --workspace` exit 0：354 passed + 1 ignored（engine_tick 既有 ignored）=355 `#[test]`；game-shell 49、game-data rom 2、shell-win 24（5 lib+19 smoke）；mock-300 新签名 `host=0x63b5a6195287a6d8 canvas=0x963ac9a776a57625` 与 spec §9 一致 | 确认 |
| C8 | 纪律 | 4 个 commit 均含 Evidence/Confidence（88/96/97/90）；两处**并发归属偏差**（task-9 game-shell 代码随 `96415c1` 入库；spec §3 主体随 `ddb1558` 入库）已在 plan §10 与 KB `gotcha-git-add-commit-atomicity` 留痕，未 rewrite 历史；写范围外仅 `Cargo.lock`/Cargo 依赖同步（必要） | 确认（附 E7） |

## 3. 命令与输出摘要（自跑）

| # | 命令 | 输出摘要 |
|---|---|---|
| 1 | `cargo test -p game-shell --test stg01_replay -- --nocapture` | exit 0；3 passed；`C6 玩家列 571/571 帧一致（f1260..=f1830，死亡窗剔除）`；`C4…engine=20 事件/20 槽/3 型；trace=16 槽/7 型；共同槽=16；engine 型不在 trace 型集：[]`；`replay-3000 signature: host=0x521d397a6c609f34 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807` |
| 2 | Python 独立重算 C1 | `rows=1208 compared=1207 mismatches=0` |
| 3 | Rust 独立探针（target 内，链接 game_shell） | `host_sig=0x521d397a6c609f34 canvas_sig=0xdf75da247b4c3825 engine_frames=1807 spawn_log_len=21 C6_compared=571 C6_mismatches=0 stage_loaded=true sched=43 load_frame=1193` |
| 4 | `cargo run -q -p shell-win -- --selftest-frames 300` ×2 | exit 0；stdout SHA256 相同；`host_signature=0xf51d5a517e7458b7 canvas=0x963ac9a776a57625 display=0xf68f9ffc0607cbad audio=2/300 rom=262160` |
| 5 | `cargo test --workspace` | exit 0；354 passed + 1 ignored（=355 `#[test]`） |
| 6 | `cargo test -p game-shell` / `-p shell-win` | 41+2+3+3 / 5+19 全绿 |
| 7 | `python gates/cli.py status`、`python gates/cli.py check` | `STATUS: OK`；`CHECK: PASS`、PASS=11 FAIL=0 SKIP=1 TOLERATED=0；rust-tests 355/355；docs-links PASS |
| 8 | GUI 全链路（PostMessage 注入 En/Enter/Enter/X/X） | 模式序列见 C6；退出后无 CRH |
| 9 | 菜单门控探针（自建） | Start@600 单独→Menu@602；right@590+Start@600→仍 Title；A@595+Start@600→仍 Title；Start@450 忽略；Start@550→Menu@553；按住 Start 600..620→仅 1 次 Menu |
| 10 | 两连 B 退出商店探针 | 第二 B 间隔 2..12f → 未进 Transition（被吞）；间隔 ≥13f → Transition 正常 |

## 4. 数字/签名重算表

| 指标 | 自述 | 我实测 | 一致 |
|---|---|---|---|
| C1 滚动帧对拍 | 1207/1207，0 失配 | 1207/1207，0 失配（独立 Python） | ✅ |
| C6 玩家列 | 571/571 | 571/571（独立 Rust 探针，0 mismatch） | ✅ |
| C3 调度记录/tick 锚 | 43 / [1,33,49,65,81] | 43 / [1,33,49,65,81]（测试+预置 golden+工具三方） | ✅ |
| replay-3000 host | `0x521d397a6c609f34` | 同 | ✅ |
| replay-3000 canvas | `0xdf75da247b4c3825` | 同 | ✅ |
| spawns / engine_frames | 21 / 1807 | 21 / 1807 | ✅ |
| C4 共同槽 | 16/16 | 16/16（trace 16 槽全被 engine 20 槽覆盖、engine-only {19,20,21,22}） | ✅（断言弱，见 E3） |
| shell selftest host_sig | `0xf51d5a517e7458b7` | 同（两次） | ✅ |
| canvas/display sig | 不变 | 不变 | ✅ |
| mock-300 host_sig | `0x63b5a6195287a6d8` | 同 | ✅ |
| workspace `#[test]` / gates | 355 / PASS | 355 / CHECK: PASS | ✅ |
| C4 帧差（自算，未报告项） | 帧列“不可对拍” | engine-oracle 首现帧差：槽8-13 **+98f**、槽14 **−181f**、槽15 **−186f**、槽16/17 **−314f**、槽18 **−392f**、槽23 **−65f**、槽24 **+14f**、槽25 **+81f**、槽26 **+89f**、槽27 **+97f**；oracle 槽1-7 有 7 个初生对象，engine 全 0 | 声明“不可对拍”成立，但幅度未报告（E3） |

## 5. findings 四分类（非阻塞；B1 建议下一小卡修）

### 5.1 证据不足（E）

- **E1（表述级，需收口）StageStart 可动的证据级别**：plan §5 更正注/§10 偏差1 称“关始段玩家可动=**A 级 trace 事实**”，据 `.player.jsonl` f1301 首移。复核：f1301 前无**方向**输入样本（1260-1300 仅 A 键 `$80` 脉冲，A 不影响移动）⇒ trace 只证明“f1301 起可动”，无法区分“f1260 起可动”与“冻结至 ~1301”；C6（571/571）因该窗无方向输入，**不构成早段可动性验证**。实现从 f1260 即 `player_tick_active`，属外推。
  建议：host.rs:636-646 注释、plan:124-129/190/255 降级为“f1301 起可动=A；intro 早段可动性未隔离（B/接线约定）”；若要 A 级，需补 oracle 场景（f1261 起方向输入）重采。
- **E2（文档-实现不符，低）**：plan §4.3/§7 称 `decode_sched(...,max=64)` 有“**truncated=false 断言**”，实际 `load_stg01_from_bank8` 只校验 `records.len()==43 && unknown_ctrl.is_none()`（host.rs:453-457），无 truncated 断言。
  建议：加 `|| stream.truncated` fail-closed（推荐，成本一行）+ 测试；或删掉该声称。
- **E3（对拍口径，低-中）**：C4 的“共同槽 16/16”只 `println` 不 assert（stg01_replay.rs:336-354 仅断言槽域/型非0/`unknown_types.is_empty()`）；且未披露 engine-only 槽 {19,20,21,22}、oracle 槽 1-7 未建模（f1264 oracle 槽1-7 型值 7,2,2,2,2,2,2，engine 全 0；属 §8“初生记录 loader 未证”）、首现帧差跨度 +98f～−392f。当前表述易被读成“16/16 槽对拍一致”。
  建议：断言 `trace_slots ⊆ engine_slots`；输出 engine-only/帧差范围；plan §10 C4 行注明“仅槽集合覆盖，不含槽 1-7、不含帧”。
- **E4（低）**：“replay 两次独立运行”实为同进程内两个 `Host` 实例（`run_replay()` ×2）；`held` 在 f1260 前默认 0（player trace 起于 1260）——即 replay 在 Transition 窗（1193-1260）输入=0 的假设未声明。建议在 plan §7 标注口径（或补双进程复跑）。
- **E5（低）**：C2 “序列与 `.flow.jsonl` 一致”是**构造性一致**（replay 事件表直接取自 flow trace 的 4 行），非独立对拍；且 Transition 用 1193 而 flow 行是 1200（capture 采样帧，测试注释有解释，plan §10/§3.1 未明写）。建议 plan §10 加一行口径说明。
- **E6（低）**：HUD 双状态源：FrameView `hud.money_digits` 取 `flow.data.money`、`hud.bomb_ammo` 取 `player.bomb_ammo`；spec §8 只登记 bomb 双源（$018F vs flow.data），money 的 `StageState.money_counter` 未登记。建议 §8 补一行（不影响暴露面）。
- **E7（低，纪律）**：两处归属偏差已留痕（好），但 `96415c1` message 未含代码内容，建议 plan §10 已做（已做，仅提示 Lead 知悉）。

### 5.2 实现错误（B）

- **B1（中；非阻塞但建议尽快修）菜单门控层 1 按“任意 `$18` 位保持”判定，导致跨键吞按**：
  - 证据（自建探针）：`Start@600` 单独 → Menu@602；`right@590 + Start@600` → **停留 Title（Start 被吞）**；`A@595 + Start@600` → 同样被吞。两连 B 退出商店：第二 B 间隔 **2..12f 被吞**（不进 Transition），间隔 ≥13f 正常。
  - 根因：`host.rs:745` 把 `menu_pressed_bitmap`（A/B/START/SELECT/方向全谱）喂 `InputLatch`；`host.rs:559` 把整张位图喂 `MenuInputGate::dispatch`；而 gate 层 1 判据=“任意位保持”（engine.rs:1278-1286），A 级机制是**按住 START 自旋/早退**（bank00:92AA/9400，[flow-response-latency.md](../../docs/findings/flow-response-latency.md) §8）。
  - 影响：标题/商店内快速连按（含文档 R-shop-7“两连 B”）可被吞；主链路（>13f 间隔）正常。
  - 建议：按模式只喂触发键（Title/Menu 传 `bitmap & MENU_START`；Shop/ShopInfo 传 `bitmap & MENU_B`），或把层 1 改为按键级保持；补“方向/A 后 12f 内按 START 仍派发”“两连 B 间隔 10f 退出”测试（host.rs:559/745、tests/smoke 或 host 单测）。

### 5.3 规格缺口（S）

- **S1（低）Transition 时长常量的语义**：`auto_advance` 用 `TRANSITION_TAIL_FRAMES=33`（A 级证据=`press→gameplay 场景`响应时延，flow-response §2）作为 Transition→StageStart 时长；capture 锚 Transition f1200→StageStart f1260（采样帧）。实现行为=33f（GUI 实测一致），但“33f 是否= Transition 全段时长”未证。建议 plan §4.2 该行标注“常量取自响应时延；Transition 全段时长=接线约定（采样锚 1200/1260 不足以定边界）”。
- **S2（低）**“可玩闭环”命名 vs shell 渲染仍占位：文档已显式声明（platform-shell §4.2 + plan §10 遗留=task-10），未见夸大；建议在 plan 标题/§0 加一句“本卡=宿主层可跑通，不是可玩画面”。
- **S3（已登记，免修）**：DeathRespawn/StageClear 的 engine tick 细粒度未隔离、A2/A3 未接线（§8/G-S5-5）——实现未掩盖，属既有规格缺口。

### 5.4 平台差异（P）

- **P1**：本机 GUI 键盘注入：`SendKeys`/`SetForegroundWindow` 在复核会话不可用（前台授权失败），改用 `PostMessage(WM_KEYDOWN)` 成功；结论只证明“winit 键盘事件→派发链”在 Windows 实窗成立，不覆盖输入法/焦点/非激活窗口行为。
- **P2**：GUI 实跑在本环境偶发“窗口未推进”（6 次注入尝试中 2 次仅 `Boot frame=1`，疑似桌面会话/窗口可见性节流），非 game-shell 逻辑缺陷（selftest/replay 均确定性通过）；建议 task-10 验收 GUI 时以“模式打点+超时重试”为验收脚本口径。
- **P3**：Linux/Android 未测（不在本卡范围）。

## 6. 对“可玩闭环 B（task-10）”的放行判断

- **放行**：FrameView 提供的 task-10 依赖字段（mode/stage/frame/player/live_bullets/player_bullets_live/enemy_slots.type_code/HUD 数值域）来源均可查；`position=None` 已明确“不得当 (0,0)”，未证域（敌坐标写者链/初生 loader/图样/文本）显式声明，task-10 不会因接口歧义发明行为。
- task-10 需注意：不要消费 `position` 当坐标；敌槽只画“已激活”占位；HUD 只画数值域不画布局/文本（spec §8 未证）；建议把 B1 修复与 task-10 分开提交（B1 属本卡）。

## 7. 未覆盖 / 未验证声明

- C4 槽**序列**（顺序）未独立重演，只核集合/型集/帧差；C2 序列为构造性一致（E5）。
- 菜单→商店中间动画（~167f）无 oracle 证据，未验证（文档已声明未证）。
- 未对 shell-win 渲染做像素/视觉验收（task-10 范围）。
- GUI 仅一次全链路成功（3 次尝试中 1 次完整、2 次环境卡 Boot）；结论以“成功一次 + 单测/自检双侧”为准，P2 已记。

> 本复核只写本文件；未改实现/被审文档，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
