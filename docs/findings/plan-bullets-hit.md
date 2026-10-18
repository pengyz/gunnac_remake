# 方案：STG-01 战斗增强——玩家弹丸生成/运动/命中接线（task-37）

> 任务卡：task-37（game-shell 集成 + FrameView 暴露；渲染由后续 shell 卡接）。
> 状态：**已实施，待独立复核**（§8 回填 2026-10-18）。纪律：[AGENTS.md](../../AGENTS.md) §3/§4——
> 已证才落结论、未证 fail-closed；**不改 game-core/game-platform**（shell-win 仅按 Lead 授权做
> FrameView 字面量适配，见 §8.1）。

## 0. 目标 / 非目标

**目标**：STG-01 战斗段（Combat）玩家主武器可发弹（B 长按门控 + 计数驱动生成）、弹按已证运动机
推进/释放，命中走已证 `shot_hitbox_register` + 敌框形 + `hit_resolves` + `clear_hit_entry` 管线并
回收；伤害按 A 级白名单受限；FrameView 暴露子弹坐标/活弹数与命中事件。
**非目标**：不改 game-core/engine 语义；不做渲染；玩家被击/敌弹友伤/敌死亡回收/普通小怪 HP 不入；
不声称 H-16 收口（精确盒/受击链未证，见 §6）。

## 1. 层定位与动手前三问（AGENTS.md §4）

| 问 | 答 |
|---|---|
| 问题在哪一层？ | **平台无关宿主层集成缺口**：`game-core::shot` 生成/运动 API 与 `enemy::fetch_shape_boxes`/`shot::hit_resolves` 已证（L2/样本），但 game-shell 从不生成玩家弹、`StageState.shape_data` 生产侧从未喂 ⇒ 命中在 host 内不可达（task-38 §1.2 C1/C2）。 |
| 消费者实际读哪个数据？ | 生成/运动/命中全部在 **game-shell 自有弹网格**（`BulletGrid`）上执行；`StageState.bullets` 保持空 ⇒ engine `combat_frame` 的弹/命中管线惰性（不双步进/不重复判定）。敌框形数据由装载侧从 ROM bank2（`$8000` 窗）喂 `StageState.shape_data`。 |
| 证据级别？ | 生成/门控/槽/运动/框形/4 边属性=A（R-shot-gen/motion/alloc、arena-midboss §2、task-38 §2.1/§2.4）；伤害=**仅 `type $4A` 白名单** A（HP=14 快照），其余 fail-closed；敌死亡回收/普通敌 HP/玩家被击未证。 |

## 2. 已证面清点（逐条带指针）

| 面 | 已证内容 | 指针 |
|---|---|---|
| 主武器门控 | **B 键按住连发**（`$18` bit6）→ `$39/$3A` 重复计时器 → `$32` bit2 六型生成器门；`$34` weapon_slot 分派（0=六型） | R-shot-gen-2/4（t56/t29 定谳）+ `shot-gating-f1.md` §1（A 反汇编链） |
| 计时器 reload | **默认 STG-01=`$3C` 连射级 0 ⇒ reload=7**：`bank02:$9391 LDA $94DF,Y / STA $39`，`$94DF`=`07 05 04 03 02`；`$9200/$92C9` 的 `#8` 仅是初始化、随后被配置装载覆盖。动态对拍 `oracle/traces/fire-hold.shots.jsonl` 出生帧 `1301,1308,1315,1322,…` **Δ=7** | A（ROM 直读 + trace；task-42 预研复核） |
| 六型计数调度 | `$072E,X`（在途弹计数，派生）< `$074A,Y`（在途上限）⇒ 生成并 INC 回写 | R-shot-gen-1（t75 补正，A） |
| 生成初值 | 寿命=1、脚本指针=配置参、坐标=玩家、弹型码=X | R-shot-gen-3（A）+ `fire-hold.shots.jsonl`（出生帧前两帧字段：128,160→128,151/ptr=13/motion=12/lifetime=255 与本实现一致） |
| 等级配置 | `$33` weapon_level → `config_index_for_level` → `$9463` 段 (上限,脚本参) 对；级 0=1 对 (05,0C) | R-shot-level-1/2（A） |
| 弹槽分配 | 20 槽（4 组×5，$80-$84/$9C-$A0/$B8-$BC/$D4-$D8），首次适应、空判 state==0 | R-shot-alloc-1/2/3（A） |
| 运动机 | 寿命 DEC；归零取指（时长非 0=设置寿命+运动码+指针++；0=跳转）；LUT 双轴带符号累加；溢出/借位=死亡释放 | R-shot-motion-1/2（A）→ 直接调用 `shot::tick` |
| 敌框形取框 | `bank2:$97BA/$9812` 88 形 → 流 `[count][4B 符号偏移]`；翻转 `$0534` bit6/7 | arena-midboss §2.1（A）；生产侧 `shape_data` 必须装载侧喂（task-38 C1） |
| 命中判定 | 弹注册 7×10+attr `$30`；4 边包含 + 属性相交；清框 | `shot::hit_resolves`（A，t66 样本 10+2） |
| 伤害（受限） | 仅 `type==$4A`（中 boss HP=14 A 级快照）可 `damage_apply(Const2)`；其余 0 伤 + 登记 | task-38 §2.4（A/未证边界）+ `engine_tick.rs` ⑫ |
| 敌弹防护 | `type_code & 0x80`（ENEMY_SHOT_TYPE=$80）/`PLAYER_ONLY∪SHARED` 槽过滤 | task-38 §1.2 C4 |

## 3. 集成点设计（全部现有公开 API，不改 game-core）

### 3.1 框形数据装载（game-shell 新模块 `shape.rs`）

- ROM 布局：框形表/流在 **bank2（CPU `$8000` 窗）**；`shape_stream_addr(shape)`（A 表）给 CPU 地址；
  file offset=`0x10 + 2*0x2000 + (addr-$8000)`（N-0094 双窗注：窗基按引用地址判）。
- `game_data::rom::prg_bank(rom, 2)` → 逐形解析 `[count][count*4B]`（越窗/截断⇒跳过该形，
  fail-closed 不补零；重复地址只保留首个）→ 填 `StageState.shape_data`；`Host::load_stg01` 成功后
  调用 `Host::load_shape_data()`（实测装载 59 个唯一形地址）。

### 3.2 玩家弹生成（`fire.rs`）

- Combat 帧、`B held`（`$18` bit6=`Buttons.b`）、`weapon_slot==0`（R-shot-gen-4 已接分派）时：
  `timer`: 未按 B ⇒ 1（下按帧即发）；按住 ⇒ DEC，归零帧重装 `$39`（默认 7）并跑生成器。
- 生成器：`pairs = config_pairs_for_level(weapon_level)`；X=0..pairs.len().min(6)：在途 < 上限 ⇒
  `shot_slot_alloc` 首适应 + `spawn_init` 写六字段；满载/达上限 ⇒ 该型不生成（A：SEC 路径）。
- 在途计数由弹网格派生（`$072E,X` 语义；无双源漂移；§6 登记等价假设）。

### 3.3 运动 / 寿命

- 直接调用已证 `shot::tick`（R-shot-motion-1/2 整体）进行玩家弹逐帧推进；死亡（寿命 0/溢出/借位）
  ⇒ 回收槽（`$87FD` 死亡处理器语义）。
- **玩家弹在 game-shell 自有 `BulletGrid`**，不写 `StageState.bullets` ⇒ engine `combat_frame`
  看到空网格（无双步进/无双份命中）；engine 的弹/命中管线缺陷（未接 `shot::tick`、未过滤敌弹、
  用 `f630` 当敌 HP）因此不在本路径上生效（§6 登记）。

### 3.4 命中 + 受限额伤害（`fire::resolve_player_hits`）

1. 每颗玩家弹（`type_code & 0x80 == 0`）→ `shot_hitbox_register`（7×10+attr `$30`）；
2. 敌槽 8..=27（`type_!=0` 且 `type_ & 0x80 == 0`）：`shape_stream_addr(slot.f518)` → `StageState.shape_data`
   查流（缺流⇒跳过，fail-closed 不造框）→ `fetch_shape_boxes`（x/y/`$0534` 翻转）；
3. `hit_resolves`（4 边 + `f4c4 & 0xF0` 属性相交）⇒ 命中：`clear_hit_entry`（清框）+ 弹回收；
4. 伤害：**仅 `type==$4A`** → `damage_apply(Const2)` 写 `f630`（HP14 A 级域）；其余 ⇒ **0 伤 +
   `fail_closed=true` 事件**，**不写 `f630`**（B4/W18 未定谳，禁止当通用敌 HP）；
5. `$4A` HP 归 0 也**不回收敌槽**（`$D616` 死亡门语义未证 ⇒ fail-closed）。

### 3.5 事件（FrameView / Host）

- `EnemyHitEvent{frame, enemy_slot, bullet_slot, damage, fail_closed}`；`Host::enemy_hit_log()` 全量，
  `FrameView.hits_this_frame` 本帧 ≤4 条（超出计 `hit_overflow`）。
- 事件=命中管线直接结果（非"HP 差观测"），fail-closed 也显式登记（不静默零伤）。

## 4. FrameView 扩展（只读；未证 None/TODO）

| 字段 | 类型 | 来源/口径 |
|---|---|---|
| `player_bullets` | `[Option<BulletView>; 20]` | 玩家槽序；`BulletView{slot,x,y,type_code}`=`$04EE/$055E/$063E`（弹网格 A 地址面；型值不命名） |
| `player_bullets_live` | `u16`（既有） | 玩家域活槽数（A 级槽分类） |
| `hits_this_frame` | `[Option<EnemyHitEvent>; 4]` | 本帧命中/受限额伤害事件；≤4 条 |

- 敌槽 `position` 仍为 **None**（task-9/38 口径：敌坐标写者链未收口，不因本卡改变）；玩家弹坐标是
  弹网格 A 地址面（与敌槽不同源）。
- `FrameView::default()`（task-37 新增）：供平台侧构造/测试用，不改变字段语义。

## 5. 确定性 & 测试

- 单测（`fire.rs`）：门控/7f 周期 + `$94DF` 表；生成上限/槽序/出生字段；`shot::tick` 运动+死亡回收；
  命中正例（几何+属性）、负例 ≥3（远框/属性不交/敌弹）；`$4A` 7 发 14→0 且不回收敌槽；视图过滤。
- 单测（`host.rs`）：B 长按生成（7f 周期、engine 弹网格保持空）；host 级命中正例（非白名单
  0 伤 fail-closed、弹回收）+ 负例 2（远框/属性不交）。
- 集成（`tests/stg01_combat.rs`）：真实 ROM 装载（shapes=59）→ capture replay 到 Combat（2500 帧）；
  两次独立 run 全等（签名/弹/命中/终态）；弹生成/FrameView 暴露；命中数只打印不设阈值
  （弹/敌对齐受 H-01/spawn 生成层未收口影响，非 H-16 收口声明）。
- `frame_view` 既有单测更新：新字段默认值 + 只读幂等。

## 6. 未证 / fail-closed 声明

| 域 | 状态 | 处理 |
|---|---|---|
| **H-16 精确判定盒**（玩家×敌 W15/W15′、`$05DC` 双读 W18） | 未证 | 只做"玩家弹×敌"已证框形管线；玩家被击/物撞**不接** |
| **普通敌 HP 字段（B4 四候选）** | 未证 | 一律 0 伤 + fail-closed 事件；**不用 `f630` 当通用敌 HP** |
| **`$4A` HP/死亡链** | HP=14 A 级；死亡门 `$D616` 未证 | 白名单 Const2 施伤，HP 0 不回收敌槽 |
| 伤害路径选择（`auto_hit_path`/`$6E` 标志 B 级） | 未证 | 仅 `Const2`（1# 弹路 A）；Field0630/Zp12 不接 |
| `$34` weapon_slot 非 0 分派（type3/4/6 齐射、=3 退出） | 未证/未接 | fail-closed 不生成（`weapon_slot!=0` 时静默停火，登记于此） |
| `$3C` 连射级（商店 `INC $3C`） | 未建模（PersistData 无此字段） | 固定 `$3C=0` ⇒ reload=7；表 `$94DF` 已录待接 |
| 拾取/升级流（weapon_level 变化） | 未接 | weapon_level 取 `PersistData.weapon_level`（默认 0） |
| 敌弹生成/敌弹打玩家/玩家被击 | 未证 | 不生成/不接；弹/命中路径过滤 `type_code & 0x80` |
| engine `combat_frame` 已知缺陷（未接 `shot::tick`、未过滤敌弹、`f630` 当敌 HP） | 不在本路径 | 玩家弹在 shell 自有网格，engine 弹网格空 ⇒ 不触发；不改 game-core |
| 弹离屏回收边界（死亡帧不入 L3 比对面） | 未证 | 仅寿命/溢出/借位判死（A） |
| oracle shots golden 全列对拍 | 未接（replay 时序与 fire-hold 场景不同；fire-hold 出生帧 Δ=7 已用于 reload 取证） | 不做全列对拍声明；只对 reload/出生字段做证据引用 |
| `inflight` 派生 vs ROM 显式 `$072E` | 等价假设（B） | 每帧由网格派生；登记 |

## 7. 验收命令

```sh
cargo test -p game-shell                      # lib 53 + contract/mock/replay/combat 集成
cargo test -p game-shell --test stg01_combat -- --nocapture
cargo test -p game-shell --test mock_platform -- --nocapture
cargo test -p game-shell --test stg01_replay -- --nocapture
cargo run -p shell-win -- --selftest-frames 300            # ×2 逐行一致
CARGO_TARGET_DIR=target/verify37 cargo test --workspace    # 独立 target（排除并发假红）
python gates/cli.py check                     # CHECK: PASS
```

## 8. 实施状态（2026-10-18 回填；待复核）

### 8.1 改动清单

| 文件 | 改动 |
|---|---|
| `crates/game-shell/src/fire.rs`（新） | 玩家弹：B 门控+`$39/$3A`（reload=7/`$94DF` 表）、六型生成器、`shot::tick` 运动/死亡、shell 命中管线（注册/敌框/4边属性/清框/回收）、`$4A` 伤害白名单、事件类型/视图、8 组单测 |
| `crates/game-shell/src/shape.rs`（新） | ROM bank2 框形流装载（`$97BA/$9812` 表 → `shape_data`；fail-closed 截断/重复）+ 2 单测 |
| `crates/game-shell/src/host.rs` | 自有弹网格字段/装载重置；Combat 帧生成→tick→stage_tick→命中解析；`weapon_slot==0` 门；FrameView/签名/读数 getter；host 级正/负例测试 |
| `crates/game-shell/src/render.rs` | FrameView 新增 `player_bullets`/`hits_this_frame` + `Default` |
| `crates/game-shell/src/lib.rs` | 模块/导出（fire/shape/BulletView/EnemyHitEvent…） |
| `crates/game-shell/tests/stg01_combat.rs`（新） | 真实 ROM 集成自检（两次 run 全等；弹生成；shapes=59） |
| `crates/shell-win/src/lib.rs`、`tests/smoke.rs` | **仅 FrameView 字面量适配**（改用 `..FrameView::default()`；Lead 授权范围，不改渲染行为/签名语义） |

### 8.2 实跑结果（本地 Windows；独立 target 复验）

- `cargo test -p game-shell --lib`：**53 passed / 0 failed**。
- `CARGO_TARGET_DIR=target/verify37 cargo test --workspace`：**exit 0，全 workspace 绿**
  （game-shell lib 53 + contract 2 + mock 3 + combat 1 + replay 3；shell-win lib 12 + smoke 21；
  此前 shell-win smoke E0063 已由 §8.1 FrameView `Default`/字面量适配修复）。
  复现时请用独立 target（仓库内相对路径），勿指向共享 `target/`（构建设施纪律 2026-10-18）。
- `cargo test -p game-shell --test stg01_combat -- --nocapture`：
  `frames=2500 shapes=59 bullets_seen=17 hits=0 killed=0 stage_clear=false mode=Combat`
  两次 run 全等；`signature=0x7c8bb02930e895dc`（hits=0 为弹/敌对齐未收口，非 H-16 收口声明）。
- mock-300：`host=0x8170870e7c5e4ac6 canvas=0x963ac9a776a57625 advanced_frames=290`
  （旧值 `0x3085adda845a078d`；原因=签名新增弹网格/计时器/shape/命中字段，语义扩展非回归）。
- STG-01 replay-3000：`host=0xdc3496b92f49ef45 canvas=0xdf75da247b4c3825 spawns=21 engine_frames=1807`
  （旧值 `0x521d397a6c609f34`；原因同上；C1 1207/1207、C6 571/571、C4 共同槽 16/16 均未变）。
- shell-win `--selftest-frames 300` ×2：stdout 逐行一致；
  `host_signature=0x68547665bf1ab396`（旧 `0x5d7156266d6ac19d`）、`canvas=0x963ac9a776a57625` 不变、
  `display=0xe99af5849b632fbd`/`view_signature=0xf6603346bf818ffd` 不变（未改渲染/视图字段语义）。
- `python gates/cli.py check`：见交付回报（CHECK: PASS；测试数较基线上升由 Lead 统一处理）。

### 8.3 表述边界

- "能生成/能命中"= 上述已证管线在 STG-01 战斗段可用；**不是 H-16 收口**：精确受击盒/受击链
  （W15/W15′/W18）、普通敌 HP（B4）、敌死亡回收、伤害路径选择仍未证，均 fail-closed 登记（§6）。
- 玩家被击、敌弹友伤、`$34` 非 0 分派、`$3C` 连射升级均未接（§6）。
