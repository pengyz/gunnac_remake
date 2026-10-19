# 敌槽可见化：已证坐标列渲染 + 命中事件呈现（plan-enemy-render）

> 任务卡：task-45（前置 task-38 敌坐标取证 / task-37 弹丸·命中接线，均复核 pass）+
> **task-164（C3 实现：`$2B` 全接线 + `$4E` 初始位置 + §4.2 正式对拍；前置
> [enemy-coords-c3-audit.md](enemy-coords-c3-audit.md) task-157/161/162/163 pass）**。
> 状态：**实施完成**（task-45 C2 → task-118 F1 收窄登记 → **task-164 C3 扩容**），待复核点验。
> 按卡要求**不 git add/commit**（改动留工作树）。
> 纪律依据：AGENTS.md §3（未证不发明）+
> [enemy-coords-impl-spec.md](enemy-coords-impl-spec.md)（task-71/75/77/79 复核 pass，C1-C5 拆分）。

## 0. 结论速览

| # | 项 | v1 落地 |
|---|---|---|
| 1 | 可渲染已证坐标列 | **V1_TYPES={$2F,$2B,$4E}（task-164/C3）**：`$2F` 恒 `(128,8)`（字面量 A）；`$2B` init（y=8/vy pair/x=param 或 `$18/$E8` 按 `$2D` 奇偶）+ tick（`$5F8` 目标 ±#$20 + 积分）全接线；`$4E` **仅初始位置**（`$C8A7[16]`→脚本 `[x,y]` + dir 翻转） |
| 2 | fail-closed（position=None） | `$4E` **运动**（`$C64D` 脚本命令集/时序未解 C，`TODO(c3-vm)`）；`$12/$1A/$1C/$2A`（**无坐标写者证据**：wave/EN 生成源未解码 W32；oracle trace 实见 154/207/46/34 样本）、低槽 `$05/$07`、ctrl1 家族 |
| 3 | 渲染 | V1 型 `position=Some` ⇒ **4×4 纯色占位块**（尺寸/图样/颜色未证 ⇒ `TODO(shell-v2)`，不发明精灵视觉） |
| 4 | 命中事件呈现 | `hits_this_frame`（task-37 事件面）保持暴露/入摘要；**闪烁/消失视觉未证**（`$D616` 死亡/回收门未证、HP0 不回收敌槽）⇒ **不改敌像素、不发明命中特效** |
| 5 | oracle 对拍（C3 正式） | **N=199（≥20；98+96+5 冻结）**：x ≤1px **199/199=100%**、y 0 容差 **199/199=100%**；`$2B` 配对 12 spawn 段（含 **reuse 段 7-12**）；两次 replay 全等 |
| 6 | selftest | `--selftest-frames 300` ×2 逐行一致；**四签名不变**（selftest 无关卡装载⇒无 spawn，理由见 §5/§9.4） |

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

1. **`$4E` 运动**（`$C64D` 脚本命令集/时序/param 域未全解，C）⇒ 仅初始位置可见、**不积分**
   （`TODO(c3-vm)`；对拍运动列不入分母）。
2. `$12/$1A/$1C/$2A` + 低槽 `$05/$07` 生成源未解码（W32）⇒ 不 spawn/不画。
3. ctrl1 家族（`$2E`+`$40-$50`）trace 零实例 ⇒ 不接。
4. 敌精灵图样/尺寸/调色板/优先级、命中闪烁/消失、受击动画 ⇒ 未证，占位块/不画（`TODO(shell-v2)`）。
5. H-16 精确判定盒残项（W15/W15′/W18 等）不在本卡；命中链沿用 task-37/67/42 口径。
6. 逐帧运动/trace 8f 采样间的插值 ⇒ 不对拍、不声称。
7. **敌人死亡/释放语义未证**（`$D616` 门）⇒ game-shell 网格不释放（约 f1750 饱和）；
   oracle 复用段 13-24 无 Rust 对应段（§9.3 不可对拍清单，fail-closed 边界）。
8. `$2B` param=0 字面量 `$18/$E8` 的 `$2D` 全局奇偶（audit §7-4 B 级模型，首值 $E8）——
   非逐值 ROM 级证据，已在对拍中体现（96/96 通过）。

## 8. 待办 / 需 Lead 裁定

1. 独立对抗复核（Lead 派 shell-verifier）：重点核「只渲染已证列」、fail-closed 型 position None、
   对拍口径**收窄登记**（见 §4）、selftest 不变原因、命中视觉未发明。
2. **C3 已实施（task-164，§9）**：`$2B` 全接线 + `$4E` 初始位置 + §4.2 正式对拍（N=199/199）。
   剩余：`$4E` 运动（脚本 VM 解码卡，`TODO(c3-vm)`）、敌人释放语义（H-16 门，另一域）。
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

### 9. C3 实现与 §4.2 正式对拍（task-164，2026-10-19 回填）

#### 9.1 实现（`crates/game-shell/src/enemy_coords.rs` + `host.rs`）

**`$2B` 全接线（audit §3 A）**

| 面 | 实现 | 证据 |
|---|---|---|
| init x | `param=$0470`≠0 ⇒ x=param（`$C599`）；param=0 ⇒ `$18/$E8` 按 `$2D` 奇偶（`$90EE-$90F7`） | A（ROM 指令）+ B（`$2D` 全局奇偶模型，首值 `$E8` 与 trace 复用段 x=231≈232 相容） |
| init y/vy | y=8（`$90CB`）；`$588=$90/$5A4=0`（`$90D0/$90BA`）⇒ 0.5625 px/f | A |
| 目标 `$5F8` | 偶 x⇒`$80`；奇 x⇒`(x&$FE)`，`<$80`⇒`+$30` 否则 `−$2F`（`$90DA-$90E9`） | A 逐指令 |
| tick | `$CD63` 每帧 vx ±`#$20` 向目标（`$9120`）→ 有符号定点积分（`$CD08` x / `$CCD3` y，高字节二补码符号分支）→ 相位 `$CD9D`（`$5C0+=#$30`，进位 `INC $5DC`，4 回卷） | A |
| **reuse 重置（F1）** | 每次落地（`land_pair` 内部落地 ⇒ 槽型 0→非0 检测；leader 走 `StageEffects` 落地事件）**延迟一帧**（真实 handler init 在落地次帧由引擎 dispatch 先执行）后显式复位：y=8、vy pair、x/y 子像素（`$0470/$048C`）、相位（`$05DC/$05C0`）、目标；随后**立即积分 1 tick**（采样口径"首帧=init 后 1 tick"，y-subpix=0x90） | audit §3.4/§7-1（B：ROM init 未显式清子像素，但 trace 24/24 段以 subpix 0x90 起仿 0 mismatch ⇒ 显式复位是保真做法） |
| leader `$0470` | `$2F` 的 `$0470`=成员 drip param（`drip_step` 消费）⇒ 复位**保留不清零**（单测守卫） | B（否则成员 param 丢失 → 全部 x=24，已由 diag 发现） |

**`$4E` 仅初始位置（audit §4 A/B）**：`param&0x7F` → `$C8A7[16]` 偏移 → `$C8B7` 脚本首 `[x,y]`
（表已从 ROM 静态解出，见 `ENEMY_4E_SPAWN_TABLE`）；dir=`param&0x80` 翻转 x（x=0&dir⇒`$FF`）；
param 越界（idx>15）fail-closed（C 级参数域）。**运动不实现**（`$C64D` 命令集未解 C）→
位置静态可见、`TODO(c3-vm)`。

**渲染面（shell-win）**：V1 型 `position=Some` ⇒ 4×4 纯色占位（沿用 task-45/task-164 同一路径，
`ENEMY_VISIBLE_PLACEHOLDER_RGBA`）；不发明图样/尺寸/颜色。

#### 9.2 正式对拍（`crates/game-shell/tests/enemy_coords_replay.rs`）

spec [enemy-coords-impl-spec.md](enemy-coords-impl-spec.md) §4.2 口径 + **spawn 分段对齐**（显式登记：
"精确 spawn 帧不可比"——本实现无敌人释放⇒网格约 f1750 饱和、spawn 绝对帧与 oracle 不同；
按 audit §3.4 GUI spawn 段表（A 证据，tracked）对齐，段内相对帧比较；含 **reuse 段 7-12**）。

```
cargo test -p game-shell --test enemy_coords_replay -- --nocapture
C3 共同采样点 N=199；x≤1px=199/199 (100.0%)；y 0容差=199/199 (100.0%)；$2B 配对段=12（含 reuse 段 7-12）
test result: ok. 2 passed; 0 failed; 0 ignored
```

| 项 | 值 | 阈值/口径 |
|---|---|---|
| `$2F` 采样帧 | **98**（恒 (128,8) 全等） | 冻结分母（`EXPECTED_2F_SAMPLES`） |
| `$2B` 采样点 | **96**（y 0 容差 **96/96**、x ≤1px **96/96**） | 冻结分母；y 0 容差优先 |
| `$2B` 配对 spawn 段 | **12**（GUI 段 1-12；**段 7-12=槽 8-13 复用段**） | 冻结（`EXPECTED_2B_MATCHED_SEGMENTS`） |
| `$4E` 初始位置 | **5** 段 x 全等（16/240/232…表值；运动列不入分母） | 冻结 |
| 总共同点 | **199**（≥20） | 分母先冻结 + `assert_eq!` 守卫 |
| 不可对拍清单（不判 FAIL） | oracle 额外型 `$12/$1A/$1C/$2A`；GUI `$2B` 段 13-24、`$4E` 槽 #5+（无释放语义） | 显式打印（spec §4.2 步骤 2/6 的"未实现清单"语义） |

**类型集合断言**：Rust 暴露坐标的型集合 == V1_TYPES；oracle 按 V1 过滤后相等（spec 步骤 2 已实现，
task-118 F1 收窄登记由 task-164 正式对拍取代——但仍是"V1 三型可达部分"，非全集合）。

#### 9.3 复用/边界（复核重点）

- **leader 复用**：`$2F` 槽经 `alloc_c2c6` Recycle/Reassign 复用（如 slot27 f1438→2038→2221），
  每次落地事件触发显式复位（position 恒 (128,8) 单测+对拍覆盖）。
- **成员复用不可达**：Rust 无敌人释放（H-16 死亡门未证 fail-closed）⇒ 成员槽不被回收；
  oracle GUI 复用段 13-24 无 Rust 对应段（§9.2 不可对拍清单）。**不发明释放语义**。
- 网格饱和：20 槽约 f1750 满 ⇒ 之后成员 spawn 停止（engine `alloc_c304` 只找空槽）；
  对拍分母只含双方均可见段（先冻结），不以此放宽阈值。

#### 9.4 selftest 与签名（如实记录）

`--selftest-frames 300` ×2：stdout 逐行一致、exit 0；**四签名不变**：
`host=0x68547665bf1ab396` / `canvas=0x963ac9a776a57625` / `display=0xe99af5849b632fbd` /
`view=0x15dbc2c7b0ba0a5d`。

原因：selftest 用 `apply_event` 走已证边集、**不经转场装载**（`stage_loaded=false`）⇒ 无 spawn
⇒ FrameView 敌槽坐标全 None、`Host::signature()` 也未含坐标模型状态（只哈希槽型值）。C3 确定性证据
= replay 两次全等 + 对拍数字；**不声称** selftest 覆盖敌槽（口径同 §5）。

#### 9.5 改动文件（本卡，未 commit）

```
crates/game-shell/src/enemy_coords.rs   （C3：V1 三型模型/$2B 全接线/$4E 初值表/reuse 复位/有符号积分/单测）
crates/game-shell/src/host.rs           （pending/deferred 落地检测、enemy_coord_target、update 重写）
crates/game-shell/src/lib.rs            （导出 C3 常量/函数）
crates/game-shell/src/render.rs         （EnemySlotView.position 文档：V1 三型）
crates/game-shell/tests/enemy_coords_replay.rs（§4.2 正式对拍 + spawn 段对齐 + 冻结分母）
crates/shell-win/tests/smoke.rs         （V1 三型渲染/可见性断言）
docs/findings/plan-enemy-render.md      （本节 + §0/§1/§7/§8）
docs/spec/platform-shell.md             （§3.1/§4.1/§4.2/§4 未证/§8 回填）
```

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
