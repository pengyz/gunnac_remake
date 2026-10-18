# 敌槽可见化：已证坐标列渲染 + 命中事件呈现（plan-enemy-render）

> 任务卡：task-45（前置 task-38 敌坐标取证 / task-37 弹丸·命中接线，均已复核 pass）。
> 状态：**实施完成**；task-111 复核 needs_revision 的 **F1 已按方案 (b) 显式登记收窄**（task-118：
> 只承诺 `$2F` 坐标列；spec §4.2 步骤 2/6 类型集合相等断言未实现、差异登记），待复核点验。
> 按卡要求**不 git add/commit**（改动留工作树）。
> 纪律依据：AGENTS.md §3（未证不发明）+
> [enemy-coords-impl-spec.md](enemy-coords-impl-spec.md)（task-71/75/77/79 复核 pass，C1-C5 拆分）。

## 0. 结论速览

| # | 项 | v1 落地 |
|---|---|---|
| 1 | 可渲染已证坐标列 | **仅 `$2F` leader 的 x/y 双列**：trace 全采样恒 `(128,8)`（A 观测）+ handler `bank02:$899B` 字面量写值（A：`$89D5 A9 80`→`$89D7 9D E0 04` `STA $04E0,X`；`$89DA A9 08`→`$89DC 9D FC 04` `STA $04FC,X`） |
| 2 | fail-closed（position=None） | `$2B`/`$4E` 成员型（handler init x/y 与速度对写值未逐值审计）、`$12/$1A/$1C/$2A`（**无坐标写者证据**：wave/EN 生成源未解码 W32；oracle trace 实见 154/207/46/34 样本）、低槽 `$05/$07` |
| 3 | 渲染 | 敌槽 `position=Some` ⇒ **4×4 纯色占位块**（尺寸/图样/颜色未证 ⇒ `TODO(shell-v2)`，不发明精灵视觉） |
| 4 | 命中事件呈现 | `hits_this_frame`（task-37 事件面）保持暴露/入摘要；**闪烁/消失视觉未证**（`$D616` 死亡/回收门未证、HP0 不回收敌槽）⇒ **不改敌像素、不发明命中特效** |
| 5 | oracle 对拍 | 共同 (帧,型) 采样点 **N=28（≥20，已钉死 `assert_eq!(total,28)`）**、≤1px 达成率 **28/28=100%（≥90%）**；$2F 已证坐标暴露 318 槽帧；两次 replay 全等 |
| 6 | selftest | `--selftest-frames 300` ×2 逐行一致；**四签名不变**（selftest 无关卡装载⇒无 spawn，理由见 §5） |

## 1. 证据映射（哪些列能画、为什么）

来源：[enemy-coords-impl-spec.md](enemy-coords-impl-spec.md) §1-§5（A/B 级指针原文保留）。

| 型 | 生成源 | 坐标写者证据 | v1 处置 |
|---|---|---|---|
| `$2F` leader | sched ctrl2 Spawn（A；43 记录中 ×16） | trace 5 槽/多帧恒 `(128,8)`（A 观测）**+ handler 字面量写值（A）**：`bank02:$899B` 体内 `$89D5 A9 80`（LDA #$80）→ `$89D7 9D E0 04`（STA $04E0,X）；`$89DA A9 08`（LDA #$08）→ `$89DC 9D FC 04`（STA $04FC,X）；静止 ⇒ 速度四元组=0（trace 无位移，不发明速度值） | **暴露 x/y 双列并渲染** |
| `$2B` 成员 | leader drip（A） | handler `bank02:$90AB` 写速度对/坐标（A 指令 / B 归属）；**init x/y 与逐型速度值/节奏未审计**（spec §3/§5.1-4）；trace 观测横摆/锯齿（非恒定速度） | **fail-closed：position=None，不画** |
| `$4E` 成员 | leader drip（A） | handler `bank04:$AE44` 写速度对/坐标（A 指令 / B 归属）；同上未逐值审计 | **fail-closed：position=None，不画** |
| `$12/$1A/$1C/$2A` | wave 流/EN 内容层（**未解码 W32**） | **无坐标写者证据**（生成源未解码；oracle trace 实见这些型 154/207/46/34 样本 ⇒ 存在但坐标链未证） | 不 spawn、不画 |
| `$05/$07`（低槽 boss 候选） | 同上 + bank 归属未定 | 无 | 不 spawn、不画 |
| `$2E`+`$40-$50`（ctrl1 家族） | sched ctrl1（静态解码 A；trace 零实例） | 语义未证 | 不接 |

**禁止项（代码内已落实）**：不用 `type_code` 反推坐标；不用常量/预测/插值位置冒充已证坐标；
不套用九字段 `$865C` 生成路径（STG-01 普通敌型走该路径未证 W32）；不把 `None` 当 `(0,0)`。

## 2. 实现（文件级）

| 文件 | 改动 |
|---|---|
| `crates/game-shell/src/enemy_coords.rs`（新） | `ENEMY_LEADER_TYPE=$2F`、`ENEMY_LEADER_POSITION=(128,8)`、`ENEMY_FAIL_CLOSED_COORDS=[$2B,$4E]`、`has_proven_coords()`、`init_proven_coords()`、`step_proven_coords()`（`engine::motion_update`，速度=0）；单测（仅 leader 有已证坐标/成员 fail-closed） |
| `crates/game-shell/src/host.rs` | `enemy_coord_ready/onscreen[28]` 状态；`update_enemy_coords()`（新激活 `$2F` 置 (128,8)+速度清零；已初始化跑已证 `motion_update`；出屏停止暴露；槽复用/清 0 重置）；每逻辑帧在 spawn 落地后调用；`frame_view().enemy_slots[].position` 只对已证槽 `Some` |
| `crates/game-shell/src/render.rs` | `EnemySlotView.position` 文档更新（只暴露已证坐标列；其余 None；不得 type_code 反推） |
| `crates/game-shell/src/lib.rs` | 导出 `enemy_coords` 模块与常量/函数 |
| `crates/game-shell/tests/enemy_coords_replay.rs`（新） | oracle 对拍（§4 逻辑）：`ExitTransition`@f1193 装载 ⇒ host 帧号==stage abs 帧号；trace 8f 采样帧取 view；类型集合（rust 暴露坐标型 ⊆ RENDER_TYPES）；共同 (帧,型) 点 N/≤1px 达成率（阈值不放宽）；fail-closed 型 position 必 None；$2F 恒 (128,8)；两次 replay 全等 |
| `crates/shell-win/src/lib.rs` | 敌槽渲染：`position=Some` ⇒ 4×4 纯色占位块（`ENEMY_VISIBLE_PLACEHOLDER_RGBA` 唯一色便于 GUI 抽样）；`position=None` 不画；命中事件不改变敌像素（事件仅入摘要）；单测更新 |
| `crates/shell-win/tests/smoke.rs` | 敌槽渲染契约：4×4/position=None 不画；模式推进集成测试断言 `$2F` 已证坐标出现且其余型 None、画布出现敌占位色像素 |

## 3. 命中事件呈现（严格按已证写集）

- task-37 已接 `hit_resolves`/`clear_hit_entry`（per-rect 消费）+ 弹回收 `$87FD` + 受限额伤害
  （仅 `type==$4A` HP=14 A 级；其余 0 伤 + `fail_closed` 事件）。本案只读消费，不改命中链。
- **已证写集**：命中清框（`$D7DA` 一框一伤，per-rect）+ 弹回收 + `$4A` HP 递减。
  `$2F/$2B/$4E` 均非 `$4A` ⇒ 伤害 0、HP 无写。
- **未证**：敌被击闪烁/消失（`$D616` 死亡/回收门未证；task-37 明示 HP0 **不回收**敌槽）、
  受击动画/音效。⇒ shell **不改敌像素**（无闪烁/消失），`hits_this_frame` 仅作为事件面入
  `frame_view_digest` 与 `Host::enemy_hit_log`（可观测）。**不发明命中特效**（`TODO(shell-v2)`）。

## 4. oracle 对拍（可复现；**收窄口径**已显式登记，阈值不放宽）

```sh
cargo test -p game-shell --test enemy_coords_replay -- --nocapture
# 实测（本机 Windows，2026-10-18）：
# C2 共同 (帧,型) 采样点 N=28 / ≤1px 通过=28
# C2 $2F 已证坐标暴露槽帧数=318
# test result: ok. 1 passed（含两次 replay 确定性全等）
```

**口径（task-118 F1 显式登记收窄；不得表述为"spec §4.2 原样"）**

- **本卡承诺范围=仅 `$2F` 坐标列（x/y 双列）**。已实现/已断言：
  - 共同 (帧,型) 采样点计数定义同 spec §4.2 步骤 3（同帧同型只计 1 点）；分母先冻结，
    **实测 N=28 并钉死 `assert_eq!(total, EXPECTED_COMMON_SAMPLES=28)`**（防覆盖回退）；
  - 每点 |dx|≤1 且 |dy|≤1（字节模 256）；达成率 28/28=100%（阈值 ≥90% **未放宽**）；
  - `$2B/$4E` 及非 `RENDER_TYPES` 型 `position` 必 `None`（fail-closed 断言保留）；
    rust 不得暴露 `RENDER_TYPES` 外型坐标（spec 步骤 6 的**方向性部分**覆盖）。
- **未实现（差异登记，交 Lead 认可）**：spec §4.2 **步骤 2/6 的
  `V1_TYPES={$2F,$2B,$4E}` 过滤后类型集合相等断言未实现**——`$2B/$4E` 坐标写者未逐值审计、
  生成源未接（本卡只做坐标列可见化）；因此"漏 spawn `$2B/$4E`"本测试不会发现，由后续 C3/gen 卡覆盖。
  oracle 额外型（`$12/$1A/$1C/$2A/$05/$07`）入未实现清单、不判 FAIL。
- **不可对拍列（未声明/未断言）**：精确 spawn 帧（8f 采样+1 帧滴灌偏差）、槽号身份（分配器差异）、
  帧内逐帧运动（trace 仅 8f 采样）。
- **实现方式变更（task-118 Info-5）**：spec C2 原设计独立工具
  `tools/enemy_coords_compare.py` + FrameView JSONL 导出**未采用**；改为
  `crates/game-shell/tests/enemy_coords_replay.rs` replay 内联测试（直接读 oracle trace、内存比较）。
  优点=零新增工具/依赖、随 crate 测试跑；缺点=trace 可追溯性弱于"独立工具+导出产物"（无独立
  JSONL 中间物；复核需直接复跑该测试）。如需跨实现复用工具/产物，另卡。

## 5. 自检与签名口径（无 spawn ⇒ 四签名不变，如实登记）

`--selftest-frames 300` ×2：stdout 逐行一致、exit 0；四签名**与本卡前完全相同**：
`host=0x68547665bf1ab396` / `canvas=0x963ac9a776a57625` / `display=0xe99af5849b632fbd` /
`view=0x15dbc2c7b0ba0a5d`。

**原因（如实）**：selftest 用 `apply_event` 走已证边集（task-1 口径），**不经过转场帧装载**
（`stage_loaded=false`）⇒ 无敌 spawn ⇒ FrameView 敌槽 position 全 None、画布不含敌占位块；
因此本卡不改变 selftest 合成/摘要。敌槽可见化的确定性证据由：
① `enemy_coords_replay` 两次 2408 帧 replay 全等（本卡新增测试）；
② 既有 `tests/stg01_replay.rs`（3000 帧两次全等，含 spawn 摘要）；
③ GUI PrintWindow 实拍（§6）三重覆盖。**不声称** selftest 覆盖敌槽（未装载即无 spawn，口径已登记）。

## 6. GUI 实拍（PostMessage + PrintWindow；可复现方法见平台 spec §4.1）

```
序列：PostMessage WM_KEYDOWN/UP → Title→Menu→Shop→ShopInfo→Transition→StageStart→Combat
（stderr 打点：Combat@frame 1062）；等待 $2F spawn（trace abs f1344）后 PrintWindow 抽样。
结果：窗口 528×519（客户区 512×480=2×）；敌占位色 RGB(255,128,0) 像素 **64px**，
      bbox **(264,47)-(271,54)** ⇒ 画布坐标 (128,8) 处 4×4 占位块（×2 缩放）。
截图：target/gui-shellwin-v2-enemies.png（可再生，不入库）。
```

## 7. 未证 / fail-closed 清单（与 spec §5 一致）

1. `$2B/$4E` 成员型坐标（handler init x/y + 速度对写值/节奏未逐值审计）⇒ **不画**（position=None）。
2. `$12/$1A/$1C/$2A` + 低槽 `$05/$07` 生成源未解码（W32）⇒ 不 spawn/不画。
3. ctrl1 家族（`$2E`+`$40-$50`）trace 零实例 ⇒ 不接。
4. 敌精灵图样/尺寸/调色板/优先级、命中闪烁/消失、受击动画 ⇒ 未证，占位块/不画（`TODO(shell-v2)`）。
5. H-16 精确判定盒残项（W15/W15′/W18 等）不在本卡；命中链沿用 task-37/67/42 口径。
6. 逐帧运动/trace 8f 采样间的插值 ⇒ 不对拍、不声称。

## 8. 待办 / 需 Lead 裁定

1. 独立对抗复核（Lead 派 shell-verifier）：重点核「只渲染已证列」、fail-closed 型 position None、
   对拍口径**收窄登记**（见 §4）、selftest 不变原因、命中视觉未发明。
2. C3（`$2B/$4E` 坐标真值）需 raw 区 handler init/速度逐段审计（decomp-analyst 后续卡）；
   本卡把其间坐标 fail-closed，不以占位/预测冒充。
3. 若 Lead 要求 selftest 覆盖敌槽：需另卡把 selftest 切换为"装载+replay"口径（会改变已记录四签名，
   须同步更新 spec §4.2 时点值链）——本卡按"不改既有签名语义"原则未做。
4. **【task-118 F1/Info 收口记录（2026-10-18）】** 按 task-111 复核 needs_revision 的方案 (b)：
   - **F1**：删除测试注释/plan §4 的"spec §4.2 原样/步骤 2 已实现"失实表述；显式登记收窄口径=
     仅 `$2F` 坐标列；spec §4.2 步骤 2/6 的 `V1_TYPES={$2F,$2B,$4E}` 过滤后类型集合相等断言
     **未实现**（`$2B/$4E` 坐标写者未审计、生成源未接；差异登记于此）；
   - **Info-1**：N=28 钉死（`EXPECTED_COMMON_SAMPLES=28` + `assert_eq!(total,28)`）；
   - **Info-2**：新增 `shell-win/tests/smoke.rs::hit_events_do_not_change_enemy_pixels`
     （有/无 hit 事件渲染结果相同；事件仅入 view 摘要）；
   - **Info-3**：补 `$89D5 A9 80`/`$89D7 9D E0 04`、`$89DA A9 08`/`$89DC 9D FC 04` 字面量证据（§1）；
   - **Info-4**：§1 `$12/$1A/$1C/$2A` 行改"无坐标写者证据（wave/EN 生成源未解码 W32；oracle 实见
     154/207/46/34 样本）"；
   - **Info-5**：登记实现方式变更（spec C2 独立工具+JSONL 未采用，改 replay 内联测试；见 §4）。
   平台 `docs/spec/platform-shell.md` 的敌坐标可见化行同步收窄口径说明。

## 附录 A：复现命令与关键输出（交付时点工作树；2026-10-18 本机 Windows）

> 本卡按任务卡要求**未 git add/commit**、未跑 `gates check/status/visual`（采集窗口期）；以下为已实跑命令原文与输出摘要。

```sh
# A1) 目标 crate 全量（game-shell + shell-win，含新增对拍测试）
cargo test -p game-shell -p shell-win
# 关键输出（全绿，0 failed）：
#   game-shell lib: 58 passed
#   game-shell contract: 2 passed
#   game-shell enemy_coords_replay: 1 passed（含两次 2408 帧 replay 全等；约 21s）
#   game-shell mock_platform: 3 passed
#   game-shell stg01_replay: 3 passed（约 26s）
#   shell-win lib: 14 passed
#   shell-win smoke: 22 passed

# A2) oracle 敌坐标对拍（task-118 **收窄口径**=仅 $2F 坐标列；阈值不放宽；--nocapture 看读数）
cargo test -p game-shell --test enemy_coords_replay -- --nocapture
# 输出：
#   C2 共同 (帧,型) 采样点 N=28 / ≤1px 通过=28
#   C2 $2F 已证坐标暴露槽帧数=318
#   test result: ok. 1 passed; 0 failed

# A3) 无窗口确定性自检 ×2（两模式 stdout 必须逐行一致）
cargo run -q -p shell-win -- --selftest-frames 300   # 第一次
cargo run -q -p shell-win -- --selftest-frames 300   # 第二次
# 两次输出完全相同（exit 0）：
#   selftest: frames=300 advanced=300 presents=300
#   selftest: mode=Combat stage=1 frame=300
#   selftest: host_signature=0x68547665bf1ab396
#   selftest: canvas_signature=0x963ac9a776a57625
#   selftest: display_signature=0xe99af5849b632fbd
#   selftest: view_signature=0x15dbc2c7b0ba0a5d
#   selftest: audio_writes=2 audio_frames=300
#   selftest: rom_len=Some(262160)
#   selftest: OK
# （四签名与 task-44 时点完全相同——原因见 §5：selftest 不经转场装载⇒无 spawn）

# A4) GUI 实拍（PostMessage WM_KEYDOWN + PrintWindow；环境=本机 Windows 会话）
# 序列：Start×2（Title→Menu→Shop）→ B×2（Shop→ShopInfo→Exit）→ 等 Transition@33f +
#       StageStart@240f → Combat（stderr `mode=Combat stage=1 frame=1062`）→ PrintWindow
# 抽样敌占位色 RGB(255,128,0)：
#   像素数=64（=4×4 canvas ×2× 缩放 = 8×8 屏像素），bbox=(264,47)-(271,54)
#   ⇒ 画布 (128,8) 处画出 `$2F` 已证坐标占位块（窗口 528×519=客户区 512×480 的 2× 上屏）
# 截图（可再生、不入库）：target/gui-shellwin-v2-enemies.png
```

**工作树文件清单（交付时点；不含并发他人改动）**

```
新增: crates/game-shell/src/enemy_coords.rs
新增: crates/game-shell/tests/enemy_coords_replay.rs
新增: docs/findings/plan-enemy-render.md
修改: crates/game-shell/src/host.rs
修改: crates/game-shell/src/lib.rs
修改: crates/game-shell/src/render.rs
修改: crates/shell-win/src/lib.rs
修改: crates/shell-win/tests/smoke.rs
修改: docs/spec/platform-shell.md
```

**复核方重点复算入口**：`$2F` 已证双列暴露（A2 `leader_seen=318`、恒 (128,8)）；`$2B/$4E` position 必 None
（A2 同测试断言）；四签名不变（A3）；对拍 N=28/28（A2）；GUI bbox（A4，可按同序列重跑）。
