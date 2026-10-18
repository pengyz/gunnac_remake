# 对抗复核：平台抽象层 v1（task-5）——证伪立场

> 复核方：platform-verifier（verifier 角色，非实施方）
> 日期：2026-10-18
> 被审对象：task-5 交付 commits `0809317`（方案）、`c43f08d`（trait 缝补全）、
> `98ad36c`（game-shell crate + 契约测试）、`6362895`（platform-shell v2 + 方案回填）；
> 配套基线 commit `ff5aa7c`（`rust_test_count` 280→326，Lead 处置，非 task-5 写范围）。
> 复核形态：**默认证伪**；不采信实施方自述、不复用其结论，全部数字/断言由复核方独立重跑或重导。
>
> **复核边界（并发施工声明）**：复核期间工作树被 task-1（shell-porter）并发修改
> （`crates/shell-win/**`、`docs/spec/platform-shell.md`、`Cargo.lock`，mtime 12:50–12:53）；
> **复核取证完成后**，task-1 已把这些改动提交为 `1be66e5`/`a11161d`/`7b5e4da`/`a93704b`/`16fb5b2`
> （PC shell v1 + 门禁解码修复 + 基线 326→338）。**task-1 的 commit 不在本报告评审面**
> （由 task-3 复核），本报告对 task-5 的判定一律以 task-5 的 commits 状态为准；
> `crates/game-platform/**`、`crates/game-shell/**`、`docs/findings/plan-platform-abstraction.md`
> 在整个复核期无改动（`git status` 复核确认），故针对它们的实跑证据有效；
> workspace/gates 级实跑采集于 task-1 落盘前（=task-5 tip `ff5aa7c`），提交态测试属性数另以
> `git grep -F -c '#[test]' ff5aa7c -- crates` 独立重算（见 §3）。
> 本报告行号均指 commit 状态；`docs/spec/platform-shell.md` 行号按 `6362895` 版本
> （task-1 的 `a93704b` 只改了 §4/§4.1，§3/§8/§9 行号不变；`plan-platform-abstraction.md`
> 自 `6362895` 后未再改动）。

---

## 0. Verdict

**verdict = needs_revision**

- **实现层通过**：架构合规、trait 契约真实性、mock 300 帧确定性、宿主行为、Android 方案
  可执行性、纪律与数字，六类必查项均**实测通过**；未发现功能性实现错误（见 §1/§2）。
- **不通过的是「证据/声明面」**：存在 1 条**把未证行为写成"已证"**的过强表述（E1）、
  1 组**已证时序未接线且未登记缺口**（S8/E4）、2 条**平台接入契约未定义**（S1/S3，
  直接影响 task-1/Android），以及若干文档/引用不一致。均可由**一个小型 docs-only 修正 commit**
  收敛，不需要推翻实现。
- 因此按 AGENTS.md「findings → 四分类路由修复 → 复审；未过复核只算待审稿」执行：
  **task-5 在修正 commit + 复审前不得记为完成**；实现本身不建议返工。

---

## 1. 必查项逐项结论（被确认 / 被证伪）

| # | 必查项 | 我的独立取证 | 结论 |
|---|---|---|---|
| 1 | game-platform 零外部依赖 | `cargo metadata --no-deps` 解析：`game-platform: deps=[] devdeps=[]`；`crates/game-platform/Cargo.toml` `[dependencies]` 为空；`cargo test -p game-platform` 的 `dependency_contract` 2 例实跑通过（且它确实读 `src/` 文件做扫描，非只看 manifest） | ✅ 确认 |
| 1 | game-shell 依赖仅 game-core+game-platform | metadata：`game-shell: deps=[game-core,game-platform] devdeps=[]`；无其它 workspace/外部依赖 | ✅ 确认 |
| 1 | game-shell 源码无 winit/softbuffer/cpal/std::fs/time/process/net | 独立全量 grep（不只看门禁）：**代码行 0 命中**；`android` 仅出现在 3 处 `//!` 文档注释（`host.rs:4`/`lib.rs:4`/`render.rs:10`，契约测试去注释后不计）；`src` 中唯一 `std::` 是 `host.rs:633` 测试模块的 `std::collections::VecDeque` | ✅ 确认 |
| 1 | 资产读取只走 AssetSource | game-shell 无 `std::fs`；`Host::load_asset(&mut dyn AssetSource,...)`（`host.rs:262-272`）是唯一取字节入口；mock/单测用内存/伪 NES 头，无真实文件系统 | ✅ 确认 |
| 2 | `cargo test -p game-platform` | 14 passed / 0 failed（dependency_contract 2 + trait_contract 10 + trait_shapes 2） | ✅ 确认 |
| 2 | `cargo test -p game-shell` | 39 passed / 0 failed（src 单元 34 + contract 2 + mock_platform 3） | ✅ 确认 |
| 2 | `cargo test --workspace` | task-5 tip `ff5aa7c` 干净树实跑：exit 0；54 个 test binary 全 ok；**passed 325 / ignored 1 / failed 0**。ignored=game-core `engine_tick` 既有用例（`#[test]` 38 属性=37 passed+1 ignored），非本任务跳过（task-1 落地后基线另升 338，不属 task-5 判定面） | ✅ 通过（口径注记见 E4） |
| 2 | mock 300 帧两次签名一致（自己跑，不读报告） | 两次**独立进程**运行 `--nocapture`：`host=0xefe39a9ba5e7c5b0 canvas=0x963ac9a776a57625 advanced_frames=290 storage_len=24`，两次逐字符一致；与 docs/plan 记录值一致 | ✅ 确认 |
| 2 | 每个新 trait 被 mock 真实实现并断言 | `trait_contract.rs` 10 例：AssetSource present/缺失 fail-closed、Lifecycle 事件序+取尽 None、Renderer size 默认回退+平台覆盖、canvas 常量锁、既有形状锁；`mock_platform.rs` 3 例：Input 脚本真实消费、Renderer present/size/resize、Audio 流序+end_frame、Storage 往返/损坏 fail-closed、AssetSource 装载/缺失、Lifecycle suspend/resume/quit 门控 | ✅ 确认（非空实现） |
| 2 | 「零平台依赖」测试扫描真实源码 | `game-shell/tests/contract.rs:86-101` 读 `src/**/*.rs` 去注释扫描；`game-platform/tests/dependency_contract.rs:96-110` 同。我用独立 grep 复算一致 | ✅ 确认 |
| 3 | 状态机只走 flow 已证边集 | `game-core/src/flow.rs:148-186` 为显式 edge 表，未命中即 `Err`；host `apply_event`（`host.rs:320-343`）委托 core + 关号守卫；host 单测遍历 Boot→8 关→Ending，mock 44 事件全部 `expect` 通过；未证边单测断言 `Err` | ✅ 确认 |
| 3 | L+R/U+D 消解在平台侧、不传 core | `game-core/src/player.rs:49-63` 对 L+R/U+D 同持 **panic**（fail-closed）；host 在 `player::tick` 之前 `resolve_directions`（`host.rs:372-374,392,401`）；host 单测断言对向同持 → 按钮全 0、无位移、不 panic；与 `game-desktop/src/lib.rs:137-149 held_from_keys` 同语义（已核对源码） | ✅ 确认 |
| 3 | `stage_tick` 调用条件 / 重生窗冻结 / `$26 bit4` 关末门 | 代码：`engine_tick_active=Combat|DeathRespawn|StageClear`（`host.rs:521-523`）；`player_tick_active=Combat`（`host.rs:529-531`），重生窗冻结有单测断言；`$26 bit4` 三条件门在 `game-core/src/engine.rs:303-316`（armed+slot0_live+!already_ran），host 仅在 `Combat` 自动投递 `StageClear`（`host.rs:407-412`）。**但 DeathRespawn/StageClear 的"已证 gameplay 段"表述过强，见 E1** | ⚠️ 行为确认，证据等级需修正 |
| 3 | suspend 窗不推进是否真断言 | host 单测断言 frame 不变、presents 不增；mock 300 帧断言 `advanced_frames=290=300-(210-200)`、`present_count==advanced_frames`、`audio_frames.len()==advanced_frames`、`final_frame==advanced_frames` | ✅ 确认 |
| 4 | 未证不发明 / 新增项标注已证·占位 | game-platform 新项逐条标注「占位契约/占位最小枚举/占位默认」，`CANVAS_*` 标已锁定常量；game-shell 模块（render/audio/snapshot）全部显式占位；§8 未证域逐项**未实现**（见 §2 明细）。**唯一例外**：DeathRespawn/StageClear 跑 `engine::stage_tick` 被写成"已证 gameplay 段"（E1） | ⚠️ 除 E1 外合规 |
| 5 | Android 方案可执行性 / 无"推倒重来"裂缝 | game-shell 全 crate 无平台词面/无桌面假设（仅 std::collections）；资产/存储/音频/呈现/生命周期全部经 trait；platform-shell §5/§6 的 winit-Android/EGL/AAudio-oboe/APK assets/内部目录/生命周期映射在方案级自洽。**挂接细节缺口**：winit Android surface 与原生 EGL/ANativeWindow 的获取/恢复路径未定义（P1）；PC 侧 AssetSource 路径基准未定义（S3） | ✅ 方案级可执行；细节待补 |
| 6 | 4 commit 小步 / 写范围 / Evidence-Confidence / 未动禁区 | 逐 commit `git show --name-status`：`0809317` 仅 plan 文档；`c43f08d` 仅 game-platform（lib+tests，**纯加法**：既有 6 trait 方法签名零改动，删除行仅 import 一行）；`98ad36c` 仅 game-shell 新 crate + 根 Cargo.toml（仅 members +1）+ Cargo.lock（生成物）；`6362895` 仅 plan 回填 + platform-shell。**未触及** game-core/game-desktop/shell-win/rom/reference/data；`ff5aa7c` 仅 data/status 基线（Lead 处置，plan §7.2 已申报）。4 commit 均含 `Evidence:` + `Confidence:` | ✅ 确认（Cargo.lock 属写范围外生成物，见 §2 观察） |
| 6 | docs 回填与实现相符（不声称未实现能力） | platform-shell §7 状态行明确 shell/win=骨架、Android=仅方案；§8 未证域逐项与代码核对一致（桌面端/音频生产侧/射击投放/菜单派发/关卡装载/存档语义/H-01/H-16/Boss 均未实现）；未发现「未实现却声称已实现」 | ✅ 确认 |
| 7 | 数字核对（326 / docs-links） | 提交态独立重算 `#[test]`：**326**（game-core 185、game-data 11、game-desktop 38、game-oracle 31、game-platform 14、game-shell 39、shell-win 8）；增量 326-280=46 = game-platform +7（dependency_contract 2 + trait_contract 5）+ game-shell 39。实跑 325 passed + 1 ignored（既有）；gates `rust-tests` 口径=属性数 326/326 PASS。`docs-links`：306 md / 206 本地链接全有效（**task-5 tip 口径**；task-1 追加文档后链接数会再变，不属 task-5 判定面） | ✅ 数字一致；"326 全绿"措辞需精确化（E4） |

---

## 2. 未证不发明专项核对（§8 逐域 vs 实现）

| §8 未证域 | 实现事实（复核依据） | 一致？ |
|---|---|---|
| H-01 弹丸/武器槽语义 | render 不画；host 暴露 `live_bullets` 仅计数供签名（`render.rs:36-38` 注释明示不画） | ✅ |
| H-16 命中判定 | host 不新增碰撞代码；仅调用 core 既有 `stage_tick`（其 `combat_frame` 为核内既有移植面，空 `StageState` 下零对象=零命中）。§6 措辞「不引入碰撞行为」与"调用含碰撞的 core 编排器"存在字面张力，见 S10 | ⚠️ 措辞 |
| Boss / EN-9+ | 无装载、无 spawn | ✅ |
| 商店消费侧 | 只有 `Mode::Shop` 模式边，无买卖 | ✅ |
| 音效/音乐触发与 PCM 合成（AUD-2/3） | `AudioPipe` 消费契约已接但**生产者空**（`audio.rs:6-9`）；平台 PCM 未实现 | ✅ |
| 调色板/BG/文本/结局卡渲染 | render 只清屏+玩家占位块；`TODO(platform-v1)` 列全部不画 | ✅ |
| 玩家主武器射击/A 边沿投放 | `pressed_bitmap` 只产出、不消费；`player::fire` 从未调用 | ✅ |
| 菜单输入门控派发 | `ModeEvent` 仅显式投递，无按键自动转移 | ✅ |
| 关卡数据装载 | v1 `StageState::default()` 空态；`stage_state_mut()` 仅测试/后续装载口 | ✅（见 S2） |
| 触摸虚拟手柄/Android 后端细节 | 仅文档清单，无代码 | ✅ |
| 原版存档语义 | `GNSH` 内部快照，不自动保存（`snapshot.rs:2-13`） | ✅ |
| StageStart 可动性 / 死亡窗滚动 | `player::tick` 仅 Combat；非 Combat 段 scroll_delta=0；但 stage_tick 仍运行（E1） | ⚠️ 见 E1 |

---

## 3. Findings 四分类

> 严重度：**高**=阻塞交付的正确性；**中**=影响下一步（task-1/Android）或违反证据纪律；
> **低**=文档/措辞/加固。本报告无「高」级发现。

### 3.1 证据不足（Evidence insufficiency）

| ID | 严重度 | 位置 | 问题与我的复核证据 | 修正建议 |
|---|---|---|---|---|
| **E1** | **中** | [host.rs:520](../../crates/game-shell/src/host.rs#L520)；[plan-platform-abstraction.md:110](../../docs/findings/plan-platform-abstraction.md#L110)、[:113-116](../../docs/findings/plan-platform-abstraction.md#L113-L116)；[platform-shell.md:77-79](../../docs/spec/platform-shell.md#L77-L79) | 「引擎 tick 活跃模式（**已证 gameplay 段**：R-ui-6/7 stage-start/combat、A2 respawn、A3 stage-clear）」三重问题：①`R-ui-6/7` 在 [ui-flow.md:66,71](../../docs/spec/ui-flow.md#L66) 中定义为 **UI 流程序列 capture 锚**（f1080/1200/1260/1500），不是"engine tick 活跃段"的证据；②注释列了 stage-start，但代码 `engine_tick_active` **不含 `Mode::StageStart`**（`host.rs:521-523`），注释与实现自相矛盾；③DeathRespawn 跑 `engine::stage_tick`（含 behavior/collision/main_loop）的行为依据不足：`respawn-window.md §3` 的 R-respawn-2 只证**玩家 x/y/亚像素冻结**，A2 证的是细粒度 `death_chain_step`，而 [l5-all-clear.md:171](../../docs/spec/l5-all-clear.md#L171) G-S5-5 明确"死亡窗与关末门并发时序**未取证**"。当前空 `StageState` 下不可观测，但"已证"是过强表述 | 把措辞降级为：「`Combat`=已证 gameplay 段；`DeathRespawn`/`StageClear`=**接线约定**，细粒度行为（死亡窗滚动/主环 poll/关末门并发 G-S5-5）未隔离」；注释删掉 stage-start 或改指 `R-stage-tick`/A2/A3；§8 增登记该项 |
| **E2** | 低 | [plan-platform-abstraction.md:121](../../docs/findings/plan-platform-abstraction.md#L121) | 帧内次序证据指针写「A2 帧序锚「行为→碰撞→主环」（[game-over-continue.md](../../docs/spec/game-over-continue.md)；`engine::FRAME_ORDER`）」；我对全库 grep「行为→碰撞」「帧序」**仅命中该 plan 自身**，被引 spec 无此锚。可复现锚只有 [engine.rs:3-4,23-24](../../crates/game-core/src/engine.rs#L3-L4)（`FRAME_ORDER` 常量与注释） | 指针改为 `crates/game-core/src/engine.rs:3-4/23-24`，或注明"帧序=A2 findings 归纳，spec 未单列" |
| **E3** | 低 | [plan-platform-abstraction.md:113-114](../../docs/findings/plan-platform-abstraction.md#L113-L114) | 把「重生窗 x/y/亚像素冻结=A 级」指向 `respawn-window.md §R-respawn-3`；实际 **R-respawn-2** 才是冻结断言（[respawn-window.md:43](../../docs/findings/respawn-window.md#L43)），R-respawn-3 是坐标复位锚（同文件 :44） | 引用改为 §3 R-respawn-2（R-respawn-3 仅用于 (128,160) 复位锚） |
| **E4** | 低 | [plan-platform-abstraction.md:259-266](../../docs/findings/plan-platform-abstraction.md#L259-L266)；commit `6362895`/`ff5aa7c` message | 两处数字/状态口径：①「rust-tests 326 测试全过」——326 是 gates `current_metrics` 统计的 **`#[test]` 属性数**（[gates/cli.py:478-483](../../gates/cli.py#L478-L483)），实际执行为 **325 passed + 1 ignored**（既有 game-core `engine_tick`）；②plan §9 仍写「rust_test_count 因新增测试高于基线…已报 Lead 处置」，但 `ff5aa7c` 已把基线 280→326，gates 现为 `326/326 OK`，回填未反映最终状态 | 文档改为「326 个 `#[test]`（325 通过 + 1 既有 ignored）」；plan §9 补一行「`ff5aa7c` 已上调基线 280→326，`gates rust-tests` OK」 |

### 3.2 实现错误（Implementation errors）

**未发现**。复核方独立验证了：trait 形状纯加法（`git show c43f08d` 删除行仅 import）、
mock 300 帧两次跨进程签名一致、对向同持不 panic 且不位移、suspend 真停逻辑帧与呈现、
关号守卫正反例（`StageAdvance` 仅 <8 / `AllClear` 仅 ==8、失败不改状态）、快照损坏 fail-closed、
音频写序保真、AssetSource 缺失 fail-closed、固定步时钟回拨不推进。
**无功能性错误需要返工**。

> 潜在风险（不作为当前缺陷，归入 S2）：`Host` 存在**两份状态源**——`stage_index` vs
> `StageState.stage_counter`（$180）、`player.bomb_ammo` vs `FlowState.data.bomb_ammo`；
> 当前空 `StageState`/未接死亡链故不可观测，接 game-data 前必须定单一真相源契约。

### 3.3 规格缺口（Spec gaps）

| ID | 严重度 | 位置 | 问题 | 修正建议 |
|---|---|---|---|---|
| **S1** | **中** | [host.rs:146](../../crates/game-shell/src/host.rs#L146)、[:347-361](../../crates/game-shell/src/host.rs#L347-L361)；[fixed_step.rs:22-31](../../crates/game-shell/src/fixed_step.rs#L22-L31)；[platform-shell.md:96](../../docs/spec/platform-shell.md#L96) | **平台接入时间基准未定义**：`Host::new` 用 `FixedStep::new(0)`，而 `Host::advance(now_ns)` 无 `start_ns` 参数、无重置口；规格只写"平台单调时钟→`Host::advance(now_ns)`"。若平台传 `Instant::now()` 的**绝对 uptime ns**（Android/PC 常见写法），首帧会追 4 帧（MAX_CATCHUP）造成启动偏移；若传相对 ns 才正确。另外 `advance` 注释"返回执行的逻辑帧数"在 suspend/quit 时返回 pacer 帧数但逻辑帧未推进（tick 空转），返回值语义不精确 | 在 platform-shell §4/§5 与 `Host::advance` doc 明确「`now_ns` 必须相对 `Host` 创建时刻（首帧=0）」；或提供 `Host::new_with_start_ns(start_ns)`/`Host::reset_pacer(now_ns)`；`advance` 返回改为"折算帧数（suspend/quit 帧不推进逻辑）" |
| **S2** | **中** | [host.rs:108-123](../../crates/game-shell/src/host.rs#L108-L123)、[:173](../../crates/game-shell/src/host.rs#L173)；[engine.rs:61](../../crates/game-core/src/engine.rs#L61)；[player.rs:43](../../crates/game-core/src/player.rs#L43)；[flow.rs:72](../../crates/game-core/src/flow.rs#L72)、[:181](../../crates/game-core/src/flow.rs#L181) | **双状态源无同步契约**：①关号：`Host.stage_index` 与 `StageState.stage_counter`（$180，关末 `INC $180`）各自维护；②炸弹弹药：`PlayerState.bomb_ammo` 与 `FlowState.data.bomb_ammo`（`PersistData::on_respawn` 在**进入** DeathRespawn 时回满 4，而 R-respawn-4 证据是**窗末**回满）。v1 不可观测（空 StageState/未接死亡链），但接 `game-data`/死亡链后可能分叉，且会让后续实现者自行发明同步规则 | 在 plan §6/§9 与 platform-shell §8 登记「单一真相源/同步契约待定」；在接关卡装载/死亡链的任务卡里明确由哪一侧写、何时写（$180 与 player 字段） |
| **S8** | 中低 | [plan-platform-abstraction.md:100-102](../../docs/findings/plan-platform-abstraction.md#L100-L102)、[:268-270](../../docs/findings/plan-platform-abstraction.md#L268-L270)；[platform-shell.md:85-89](../../docs/spec/platform-shell.md#L85-L89) | **已证时序未接线且未登记**：host 只自动接 `Combat→StageClear` 一条边，其余 ModeEvent 全部由平台/测试显式投递；但**已证时序**未接线也未在遗留缺口列出：①关始 intro 240f（`engine.rs:1372` RUN_STAGE_INTRO_FRAMES「R-ui-6 锚 1260→1500」；[l5-all-clear.md:45](../../docs/spec/l5-all-clear.md#L45)）；②关末细段 `stage_clear_step`（A3 E312-E396：50×8f 倒数+飞离+第 4 关特判；[l5-all-clear.md:94-103](../../docs/spec/l5-all-clear.md#L94-L103)、`ClearRun` 常量 `engine.rs:1475-1483`）；③重生玩家侧复位 R-respawn-3/4（[respawn-window.md:44-45](../../docs/findings/respawn-window.md#L44-L45)）。task-1 若按 §3 边集实现会不知道"何时投递 StageStart→MidCombat / StageClear→StageAdvance|AllClear"，只能在 shell 层重新发明时机；而 §9 遗留缺口只列「死亡窗/GAME OVER 菜单自动接线」 | plan §6/§9 与 platform-shell §8 增一行：「v1 中除 `Combat→StageClear` 外 ModeEvent 由平台/测试显式投递；关始 240f intro、A3 关末细段、A2 死亡链/R-respawn 玩家侧复位**已证但未接线**，属后续任务卡」；§3 状态机段补"驱动方=平台/上层" |
| **S3** | 中低 | [host.rs:511-512](../../crates/game-shell/src/host.rs#L511-L512)（`ROM_ASSET_PATH` 注释「PC=**仓库工作目录**相对路径」）；[platform-shell.md:100](../../docs/spec/platform-shell.md#L100)、[:116](../../docs/spec/platform-shell.md#L116) | **AssetSource 路径基准未定义/不可发布**：`ROM_ASSET_PATH="rom/gunnac_cn.nes"` 为相对路径；PC 从任意 cwd（Explorer/快捷方式）启动会读不到 ROM；Android 则需映射到 APK assets 根。规格只写"`std::fs`（`rom/`）"，未定义基准目录 | §4/§5 明确：PC 基准=可执行文件所在目录（找不到时可回退 exe/../rom 与 CARGO_MANIFEST_DIR），Android=APK `assets/` 根且路径语义与 PC 相同；host 注释去掉"仓库工作目录" |
| **S4** | 低 | [plan-platform-abstraction.md:143](../../docs/findings/plan-platform-abstraction.md#L143)、[:174](../../docs/findings/plan-platform-abstraction.md#L174) | 方案两处写 `Host::load_rom(...)`，实现与文档其余处是 `Host::load_asset`（[host.rs:262](../../crates/game-shell/src/host.rs#L262)）；规格未出现该名，但 plan 是带证据指针的交付文档，API 名不一致会误导 task-1 | plan 改 `load_asset`（或在 §9 注明"实现改名 load_rom→load_asset"） |
| **S5** | 低 | [input.rs:62-86](../../crates/game-shell/src/input.rs#L62-L86)；[platform-shell.md:68](../../docs/spec/platform-shell.md#L68)；对照 [flow.rs:249-254](../../crates/game-core/src/flow.rs#L249-L254) | `pressed_bitmap` 自称 `$16` 边沿位图，但只产方向+A/B，**START/SELECT 不入**（input.rs:64 有注明；规格 §3 未注明"partial"）。后续菜单域（flow.rs `MENU_BTN_START=0x10`/`MENU_BTN_SELECT=0x20`）需要 START/SELECT 边沿，接线时容易误用该函数 | 规格 §3 标注"v1 `$16` 位图为方向+A/B 子集（不含 START/SELECT）"；或直接补 START/SELECT 位（`Buttons.start/select` → bit4/bit5）并保持 v1 不消费 |
| **S6** | 低 | [dependency_contract.rs:71-94](../../crates/game-platform/tests/dependency_contract.rs#L71-L94)、[contract.rs(game-shell):103-132](../../crates/game-shell/tests/contract.rs#L103-L132) | 依赖门禁只解析 `[dependencies]` 段字符串：`[dev-dependencies]`、`[build-dependencies]`、`[target.'cfg(...)'.dependencies]` 均可绕过"零外部/纯逻辑依赖"判定（当前 manifest 干净，属加固项） | 用 `cargo metadata` 解析全部依赖（kind/target）或在扫描器中覆盖所有 `[...dependencies]`/`[...dependencies.*]` 段 |
| **S7** | 低 | [snapshot.rs:110-143](../../crates/game-shell/src/snapshot.rs#L110-L143)；[host.rs:286-291](../../crates/game-shell/src/host.rs#L286-L291) | 快照只做格式 fail-closed（magic/version/len/flags/mode tag），**不做语义校验**：`stage` 接受 0..=255、mode 与 stage 可组合出不可能态；`restore` 直接置 `flow/stage/all_clear/player`，且不恢复 `prev_buttons/FrameClock/pacer`（clock/StageState 不在快照内已有说明，但 prev_buttons 未说明）。损坏存档（合法格式但任意字节）可造成宿主不可能状态 | decode 增 `1<=stage<=8`（或注明由上层校验）；`restore` doc 注明 prev_buttons/pacer 不恢复（或一并重置） |
| **S9** | 低 | [plan-platform-abstraction.md:201-217](../../docs/findings/plan-platform-abstraction.md#L201-L217)、[:257-266](../../docs/findings/plan-platform-abstraction.md#L257-L266) | plan §9「门禁输出摘要」只给文字摘要，未贴 `gates check` 实跑输出/`cargo test --workspace` 的真实计数；§8 提交计划 6 步 vs 实际 4 commit 虽有偏差说明，但 §9 未记录 `ff5aa7c` 基线处置结果（见 E4） | §9 补 `PASS=11 FAIL=0 SKIP=1`、`cargo test --workspace` 计数、基线 326/326 OK 三行 |
| **S10** | 低 | [plan-platform-abstraction.md:206](../../docs/findings/plan-platform-abstraction.md#L206) | §6 写 H-16「不接；**不引入碰撞行为**」，但 host 每帧调用 `engine::stage_tick`，其 `combat_frame`（`engine.rs:209+`）就是碰撞/命中管线；空 `StageState` 下零对象所以不可观测，但"不引入碰撞行为"字面不准确 | 措辞改为「不新增碰撞/命中逻辑；沿用 core 既有 `stage_tick` 管线，空 StageState 下 inert」 |

### 3.4 平台差异（Platform differences）

| ID | 严重度 | 位置 | 问题 | 修正建议 |
|---|---|---|---|---|
| **P1** | 中低 | [platform-shell.md:112-113](../../docs/spec/platform-shell.md#L112-L113)、[:123-126](../../docs/spec/platform-shell.md#L123-L126) | **winit-Android 与原生 EGL/ANativeWindow 的挂接面未定义**：§5/§6 说 winit 0.30 Android + EGL/GLES2 纹理 + `ANativeWindow`，但 winit Android 后端自管 surface/生命周期；平台侧 `Renderer` 如何取得 native window、Resume 时如何重建 EGLSurface/context、与 winit 事件循环的线程/时机约束均未写。这是 Android 任务卡的第一块硬骨头（不是"推倒重来"裂缝——game-shell 层无桌面假设已复核） | §6.2 增一行「surface 获取/Resume 重建路径待定（候选：winit android 扩展取 window + EGLSurface 重建；或 ndk glsurface）」，Android 任务卡先做 spike |
| **P2** | 低 | [platform-shell.md:46](../../docs/spec/platform-shell.md#L46)、[:99](../../docs/spec/platform-shell.md#L99)、[:115](../../docs/spec/platform-shell.md#L115) | `Audio` trait 只收 `ApuWrite` 流，无 PCM 输出面；规格写 PC「cpal 族」、Android「AAudio/oboe」但未写明"合成器/PCM 属平台侧、trait 不承载"，三平台有各自发明合成语义的风险 | §2 `Audio` 行补「PCM 合成与设备输出=平台实现侧；trait 仅承载写流消费（合成器未证期静音占位显式）」 |
| **P3** | 低 | [platform-shell.md:96](../../docs/spec/platform-shell.md#L96)；对照旧 `shell-win` `FRAME_MS=16`（当前并发工作树已删，commit 状态下 `crates/shell-win/src/lib.rs`） | 迁移口径未写：旧 `FrameDriver` 16ms 节流与新的 ns 固定步是两套时钟；task-5 文档只说"与 shell-win 现行 FrameDriver 同值"（指追帧上限 4），未写"16ms 常量必须删除/由 ns 固定步取代" | §4 注明「旧 `FRAME_MS=16`/`FrameDriver` 由 `game_shell::FixedStep` 取代，不得同时保留两套节流」 |

---

## 4. 我执行的命令与输出摘要

> 环境：Windows / pwsh；`python`（`python3` 损坏，按仓库约定用 `python`）。
> 所有 `cargo`/`gates` 运行均在 task-5 交付 commit 状态下、无 task-1 并发改动时采集
> （见报告头「复核边界」）。

```text
# 0) 依赖面
cargo metadata --no-deps --format-version 1  | (解析 packages.dependencies)
  game-platform: deps=[] devdeps=[]
  game-shell:     deps=[game-core,game-platform] devdeps=[]
  （对照：game-core 仅 game-platform；game-desktop/shell-win 才有 winit/softbuffer）

# 1) 契约真实性（自己跑）
cargo test -p game-platform
  test result: ok. 2 passed   (dependency_contract)
  test result: ok. 10 passed  (trait_contract)
  test result: ok. 2 passed   (trait_shapes)          => 14 passed / 0 failed
cargo test -p game-shell
  test result: ok. 34 passed  (lib unit)
  test result: ok. 2 passed   (tests/contract.rs)
  test result: ok. 3 passed   (tests/mock_platform.rs) => 39 passed / 0 failed
cargo test --workspace
  CARGO_EXIT=0；54 个 test result 行；passed 325 / ignored 1 / failed 0

# 2) mock 300 帧：两次独立进程 + 比对（不读报告）
cargo test -p game-shell --test mock_platform mock_platform_drives_300_frames_deterministically -- --nocapture   （第 1 次）
cargo test -p game-shell --test mock_platform mock_platform_drives_300_frames_deterministically -- --nocapture   （第 2 次）
  run1: mock-300 signature: host=0xefe39a9ba5e7c5b0 canvas=0x963ac9a776a57625 advanced_frames=290 storage_len=24
  run2: mock-300 signature: host=0xefe39a9ba5e7c5b0 canvas=0x963ac9a776a57625 advanced_frames=290 storage_len=24
  两次逐字符一致；RUN1_EXIT=0 RUN2_EXIT=0

# 3) 总闸（自己跑）
python gates/cli.py check
  [PASS] rom-integrity / seed-integrity / assets-manifest / naming-ledger / knowledge-format
         / docs-links(306 md / 206 链接全有效) / rust-tests(326，基线 326)
         / subroutine-diff / trace-diff / visual-diff / completeness-ratchet
  [SKIP] addr-bridge（条件 C：JP 参照锚到位后启用；显式打印）
  ratchet: 7 项 OK（rom/seed/naming/assets/knowledge/rust_test_count 326-326/completeness 19-19）
  SUMMARY: PASS=11 FAIL=0 SKIP=1 TOLERATED=0
  CHECK: PASS（含 0 个静默跳过）

# 4) 独立重算测试属性数（提交态，避免工作树并发污染）
git grep -F -c '#[test]' ff5aa7c -- crates   （逐文件求和）
  TOTAL 326
  game-core 185 | game-data 11 | game-desktop 38 | game-oracle 31 | game-platform 14
  game-shell 39 | shell-win 8
  增量 326-280 = 46 = 7（game-platform 新增）+ 39（game-shell 全部）

# 5) 独立扫描（不只看门禁）
grep game-shell/src 全部禁词（winit/softbuffer/cpal/sdl/oboe/aaudio/jni/android/
  std::fs/net/process/env/time/Instant::now/SystemTime/thread_rng/rand::/getrandom/tokio）
  → 代码行 0 命中；仅 3 处 //! 文档注释含 "android"（契约测试去注释，合理）
  → src 内唯一 std:: = host.rs:633（#[cfg(test)] 的 std::collections::VecDeque）
game-core/src 同类扫描：0 命中（game-core 未被本任务修改）

# 6) commit 合规
git show --name-status/{--stat} 0809317 c43f08d 98ad36c 6362895 ff5aa7c
  → task-5 4 commit 只触及：docs/findings/plan-platform-abstraction.md、
    crates/game-platform/**、crates/game-shell/**、Cargo.toml(members+1)、Cargo.lock、
    docs/spec/platform-shell.md；均含 Evidence/Confidence；
    ff5aa7c 仅 data/status/baselines/**（基线 280→326，Lead 处置）
git show c43f08d -- crates/game-platform/src/lib.rs
  → 纯加法；既有 Clock/GameRng/Input/Audio/Storage 方法签名零改动；
    trait_contract 既有 5 例保留（+5 新增），删除行仅 use 行

# 7) 证据指针抽查（重导，不复用被审方结论）
docs/spec/player-movement.md §R-move-1  → $18 bit0/1/2/3/6/7 A 级（host 位图一致）
crates/game-core/src/player.rs:49-63    → L+R/U+D 同持 panic（宿主消解前提成立）
docs/findings/respawn-window.md §3      → R-respawn-2 冻结 / R-respawn-3 (128,160) / R-respawn-4 $018F:=4
docs/spec/l5-all-clear.md:16,94-103,171 → E2F3 $26 bit4 门、E37C INC $180、E392 $180==9、G-S5-5 未证
crates/game-core/src/engine.rs:97-108,288-317 → stage_tick 帧序、$26 bit4 三条件门
docs/spec/ui-flow.md:66,71              → R-ui-6/7 为 UI capture 锚（非 engine tick 证据 → E1）
docs/findings/ram-watch-s3.md:55        → bank00:9C5D $018F:=4（INITIAL_BOMB_AMMO 依据成立）
crates/game-desktop/src/lib.rs:137-149  → held_from_keys 对向互斥（与 resolve_directions 同语义）
docs/findings/external-data-verification.md:296 → boot $180:=1 / $4C:=1（关号初值 1 有 A 级依据）
```

> 环境噪声（非 task-5 归因，且已在复核期外修复）：我这次 `gates/cli.py check` 的 stderr 出现
> `UnicodeDecodeError: 'gbk' codec ...`（`subprocess._readerthread`，Windows cp936 本地编码问题；
> 本机 check 仍以 PASS 结束，基于 returncode 的判定不受影响，但失败路径的输出摘要可能丢失）。
> 复核取证后并发施工方已提交 `a11161d fix(gates)`，给 `gates/cli.py` 的 3 处
> `subprocess.run(capture_output=True, text=True)` 加 `encoding="utf-8", errors="replace"`。
> 该 commit 属 task-1/Lead 范围、不改变本报告的 gates PASS 结论；此处保留观察记录仅作复现链说明。

---

## 5. 数字重算表（自述 vs 我的独立复算）

| 指标 | 实施方/文档自述 | 我的独立复算 | 判定 |
|---|---|---|---|
| game-platform 测试 | 10+2+2 全绿 | 14 passed / 0 failed | ✅ 一致 |
| game-shell 测试 | 34+2+3 全绿 | 39 passed / 0 failed | ✅ 一致 |
| workspace 测试 | 「326 全绿」 | task-5 tip `ff5aa7c` 干净树实跑：exit 0；**325 passed + 1 ignored + 0 failed**；326 为 `#[test]` 属性口径（task-1 落地后基线已另升 338，不属 task-5 判定） | ⚠️ 数字口径需精确化（E4） |
| `rust_test_count` 基线 | 280→326（ff5aa7c） | 提交态独立重算 326；增量 46=7+39；gates 326/326 OK | ✅ 一致 |
| mock 300 帧签名 | host=0xefe39a9ba5e7c5b0 canvas=0x963ac9a776a57625 advanced=290 storage=24（两次一致） | 两次独立进程逐字符一致、值全同 | ✅ 一致 |
| docs-links | 306 md / 206 链接全有效 | gates 输出一致 | ✅ 一致 |
| gates check | CHECK: PASS（PASS=11 FAIL=0 SKIP=1） | 实跑一致（TOLERATED=0；addr-bridge 显式 SKIP） | ✅ 一致 |
| 平台依赖面 | game-platform 零外部、game-shell 仅 game-core+game-platform | metadata + 独立 grep 一致 | ✅ 一致 |
| 宿主行为 | suspend 窗 10 帧不推进（290=300-10）、present/audio 计数=推进帧 | 代码+断言+实跑一致 | ✅ 一致 |

---

## 6. 修正建议汇总（按优先级；目标=一个 docs-only commit）

**阻塞 pass（必须修正后复审）**

1. **E1**：`host.rs:520` 注释、plan §3.5（:110）、platform-shell §3（:77-79）把
   DeathRespawn/StageClear 的 `stage_tick` 从"已证 gameplay 段"降级为"接线约定，
   细粒度行为未隔离（G-S5-5）"；注释删除 stage-start，改指 R-stage-tick/A2/A3；
   platform-shell §8 增登记该项。
2. **S8 + E4（缺口部分）**：plan §6/§9、platform-shell §8 增加
   「除 `Combat→StageClear` 外 ModeEvent 由平台显式投递；关始 240f intro（R-ui-6）、
   A3 关末细段 `stage_clear_step`、A2 死亡链与 R-respawn 玩家侧复位=已证未接线」
   并把 §3 状态机段补"驱动方=平台/上层"。
3. **S1**：定义 `Host::advance(now_ns)` 的时间基准（相对 Host 创建=0）或新增
   `Host::new_with_start_ns`；明确 `advance` 返回值语义（suspend/quit 不推进逻辑）。
   ——最省事：platform-shell §4/§5 + host.rs:347-355 doc 各一段，不改 API。
4. **S3**：明确定义 PC `AssetSource` 基准目录（exe 目录）与 Android assets 根的路径映射；
   `host.rs:511` 注释去掉"仓库工作目录"。

**建议修正（不阻塞，但应在本任务/下任务前落）**

5. E2（帧序指针）、E3（重生冻结引用）、S4（load_rom→load_asset）、S5（$16 partial）、
   S7（快照语义校验/说明）、S9（plan §9 门禁实录+基线处置回填）、S10（H-16 措辞）。
6. S2（双状态源单一真相契约）——建议作为**接 game-data/死亡链任务卡的前置条款**写进 plan §6。
7. S6（依赖门禁扫描面加固）、P1（Android surface/EGL 挂接）、P2（Audio PCM 归属）、
   P3（旧 FRAME_MS 迁移口径）。
8. E4（数字措辞）：文档统一为「326 个 `#[test]`：325 passed + 1 既有 ignored」；
   `ff5aa7c` 基线处置结果回填 plan §9。

**非本任务引入、顺带观察（不计入本复核 findings）**

- `crates/game-platform/tests/trait_shapes.rs:3` 注释写「196608 字节」，实际常量/断言为
  245760（256×240×4）；该文件自 `7dca0a3` 起未改，属既有文档陈旧。
- `crates/game-platform/tests/trait_contract.rs` 的 `MockClock`/`MockInput` 从未被构造
  （编译警告 dead_code）；即 Clock/Input 的"mock 断言"实际不存在（既有代码，非 task-5 新增）。
  若要支撑"每个 trait 都被 mock 断言"，建议后续补 `now_ms` 与 `buttons()` 的真断言。
- `Cargo.lock` 属 task-5 写范围外生成物（98ad36c 携带），plan §7.3 已申报，属可接受偏差。

---

## 7. 对 task-1（PC shell）的解锁建议

**建议：task-1 已不需要"等待解锁"——它在复核期间已并行落地并提交**（`7b5e4da` feat shell-win v1 +
`a93704b` 规格回填 + `16fb5b2` 基线 326→338 等，均不在本报告评审面，由 task-3 复核）。
本报告结论对 task-1 的实际含义是：**task-5 的宿主/缝底座已被独立验证通过，task-1 建立在其上不踩空**；
但 task-1 必须吸收本报告 4 条「task-1 相关」注意事项（转交 task-3/shell-porter）：
  1. **S1**：`Host::advance(now_ns)` 传**相对**时间（首帧 0），不要传 `Instant::now()` 绝对 uptime；
  2. **S3**：`AssetSource` PC 实现请用 **exe 目录**为基准解析 `rom/gunnac_cn.nes`，不要依赖进程 cwd；
  3. **S8**：除 `Combat→StageClear` 外 ModeEvent 需 shell 显式投递；关始 intro/关末细段/死亡链
     的已证时序**尚未接线**——shell v1 若只做可跑演示，请按"已证锚点+显式脚本"实现并在文档声明，
     不要静默硬编码 240f/400f/265f 当作已收敛行为；
  4. **E1**：DeathRespawn/StageClear 内的 stage_tick 行为等级是"约定"而非已证；做差分对拍时
     死亡窗/G-S5-5 并发窗仍应视为未证域。
- task-5 侧建议由 platform-porter 基于**当前 HEAD** 提交一个 **docs-only 修正 commit**
  （§6 第 1-4 条 + 便宜项；`plan §6/§9` 与 `platform-shell §8` 未被 task-1 改动，仍是原状，可直接就地修订），
  复核方复审后转 pass、把 task-5 标完成。

---

## 8. 复核声明

- 本复核的每条结论均来自复核方自己执行的命令/源码阅读/文档 grep，**未复用实施方结论**，
  未把 `plan-platform-abstraction.md` §9 或 commit message 的自述当作证据。
- 未修改任何实现代码或被审文档；未 `git add`/`git commit`；仅新增本文件。
- 复核期间 task-1 在 `crates/shell-win/**`、`docs/spec/platform-shell.md`、`Cargo.lock`
  有并发未提交改动（不属本报告评审面）；因此对 `docs/spec/platform-shell.md` 的引用一律
  按 commit `6362895` 版本与行号。
- 复现：命令见 §4；若在并发工作树上重跑 workspace 测试，需先确认 task-1 改动不影响
  `crates/game-platform/**`、`crates/game-shell/**`（当前 `git status` 显示这两个目录干净）。
