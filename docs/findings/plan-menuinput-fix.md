# 方案：修 game-shell `MenuInputGate` 跨键吞按（task-29 / task-11 B1）

> 任务卡：task-29（来源 task-11 复核 B1，严重度中；独立小卡，**不混入 task-10**）。
> 状态：**方案（实施前）**；实施后 §6 回填。纪律：[AGENTS.md](../../AGENTS.md) §3/§4——
> 已证机制才落结论；未证域 fail-closed；不改已证边集/帧锚。

## 0. 缺陷现象（复核方探针，可复现）

- **跨键吞按**：`right@590` 或 `A@595` 后 12f 内 `START@600` 被吞（窗口内无法从 Title 进入 Menu）；
- **两连 B 间隔敏感**：Shop 内两连 B 退出时，第二个 B 若距第一个 **2..12f** 被吞，**≥13f** 正常。

## 1. 层定位与动手前三问（AGENTS.md §4）

| 问 | 答 |
|---|---|
| 问题在哪一层？ | **平台无关宿主层（game-shell）输入派发逻辑**：`Host::dispatch_menu` 把 `$18` 锁存位图**整幅**喂给 `engine::MenuInputGate::dispatch`，该门层 1 以「任意位保持」判定按下沿 ⇒ 其它键的 12f 保持吞掉目标键按下沿。**不是** ROM 行为错误、不是 game-core 边界错误（`MenuInputGate` 的"参数化"契约不限定键域，键域选择属调用方=宿主）。 |
| 消费者实际读哪个数据？ | `game-shell::Host` 每帧读 `engine::InputLatch`（`$18` 域）+ `pressed_bitmap`（`$16` 域），按当前 `flow::Mode` 分派到 `Heading`/`Menu`/`Shop`/`ShopInfo`。现状 `dispatch_menu` 在**任何模式**都用整幅 `menu_latch.bitmap_18` 调门。 |
| 证据级别？ | START 早退门=**A 级**（bank00:92AA `LDA $18/AND #$10/BNE $92A7` 按住自旋；bank00:9400 `LDA $18/AND #$10/BNE RTS` 按住早退；`flow-response-latency.md` §1/§3、§8 两层模型）；`$18` 锁存 press@N→N+1f/保持 12f=A 级。Shop B 的保持/边沿语义=**未证**（只有"两连 B 退出"与"B 单击切页"的按下事件证据，无保持门代码证据）。 |

## 2. 已证机制（复述，带指针）

1. `$18` 锁存：press@N → 锁存 **N+1f**、保持 **12f**（`flow-response-latency.md` §1，A；10 press 全一致）。
2. 标题段输入锁：(450,550] 内不派发（`flow-response-latency.md` §3，A 引证；本实现 `phase_unlocked` 在 frame>550 置真）。
3. **层 1 早退门 = START 专用**：bank00:92AA `LDA $18 / AND #$10 / BNE`（按住 START 自旋）；
   bank00:9400 `LDA $18 / AND #$10 / BNE RTS`（按住早退，本次派发作废）——**掩码字面 #$10 只含 START**，
   不含方向/A/B（A 级；`flow-response-latency.md` §8 两层模型）。
4. 两连 B 退出商店：`scenario:gameplay-stage1` press 1100 b / 1160 b → f1200 transition（R-shop-7 A/B；
   两按相隔 60f，未覆盖短间隔语义）。

## 3. 现实现问题（根因）

`Host::dispatch_menu`：

```rust
let Some(bits) = self.menu_gate.dispatch(self.menu_latch.bitmap_18) else { return; };
match mode { Title/Menu if bits & START != 0 => ..., Shop/ShopInfo if bits & B != 0 => ..., _ => {} }
```

- `MenuInputGate` 层 1 的 `held = buttons_18 != 0` **对整幅位图**成立 ⇒ right/A 的 12f 锁存保持期间
  `prev_held=true`，START 按下沿被判定为"保持中"⇒ 吞按；
- Shop 路径同样把整幅位图喂门：前一个 B 的 12f 锁存保持会吞掉 2..12f 内的第二个 B（≥13f 正常=保持已过）；
- 该缺陷**只在输入时序**上体现，不改变已证模式边集本身；但会破坏"按下沿派发"的 A 级语义
  （START 门本应只看 START 位）。

## 4. 修法候选与选择

| 候选 | 描述 | 评价 |
|---|---|---|
| A. 按模式只喂触发键 | Title/Menu 只喂 `menu_latch.bitmap_18 & START`；Shop/ShopInfo 只喂 B 位 | 直接对齐 A 级「START 专用早退门」；对 Shop B 仍受 12f 锁存保持影响（短间隔两连 B 仍吞）⇒ 只修一半 |
| B. 按键级保持门控 | 对每个触发键独立记录 held 沿（不共用整幅位图），Raw `pressed`（`$16`）边沿判定 | 修 round-trip：任意键互不干扰；两连 B 短间隔可识别（真实按下沿）。但 B 无 A 级保持门证据 ⇒ 引入"边沿派发"选择，须显式登记未证域 |
| **选定：A+B 组合（最小且对齐证据）** | Title/Menu：`$18` 锁存 + `MenuInputGate`，但**只喂 START 位**（严格对齐 bank00:92AA/9400 的 `AND #$10`）；Shop/ShopInfo：用 **`$16` 原始按下沿（`pressed & B`）** 键级边沿派发（不共用 `$18` 12f 保持） | START 侧=完全 A 级机制；B 侧=证据只到"每次按下一次动作"（R-shop-6/7），边沿模型是最小 fail-closed（不发明保持重复/连发语义），短间隔两连 B 恢复 |

**选择理由**：不改 `game-core::MenuInputGate`（任务卡写范围外，且其"相位解锁/按下沿"契约仍被复用）；
不改已证边集/帧锚；只改宿主**键域选择**与 Shop 路径的边沿源。B 的保持语义未证 ⇒ 采用"一次按下=一次动作"
的边沿模型（fail-closed：不重复、不连发）。

## 5. 实现与回归测试设计

### 5.1 实现（`crates/game-shell/src/host.rs`，必要时 `input.rs` 注释）

```rust
// tick(): 锁存只喂 START（A 级 AND #$10 键域）
self.menu_latch.press(menu_pressed_bitmap(&self.prev_buttons, &buttons) & MENU_START);

// dispatch_menu(frame, pressed):
let trigger = match self.flow.mode {
    Mode::Title | Mode::Menu => {
        // START：$18 锁存 + 两层门（层1 = START 专用早退，A 级）
        if self.menu_gate.dispatch(self.menu_latch.bitmap_18 & MENU_START).is_some() { MENU_START } else { 0 }
    }
    Mode::Shop | Mode::ShopInfo => pressed & MENU_B, // $16 原始按下沿（键级边沿；B 保持语义未证）
    _ => 0,
};
match (self.flow.mode, trigger) { (Title, START) => Menu, (Menu, START) => ShopArrival,
    (Shop, B) => ShopPage, (ShopInfo, B) => ExitTransition(pending +33f), _ => {} }
```

- `MenuInputGate.phase_unlocked` 仍按 (450,550] 解锁（层 2 不变）；
- `menu_gate` 只在 Title/Menu 被调用 ⇒ 层 1 的 `prev_held` 只反映 START；
- 其它键不再影响派发；已证边集/帧锚不变。

### 5.2 回归测试（2 条；`crates/game-shell/src/host.rs` 测试模块）

1. `menu_start_not_swallowed_by_other_keys`：帧 590 按 right（或 595 按 A）、600 按 START ⇒ 修复后
   Title→Menu 必须发生（旧实现被吞）。断言语义=模式序列含 Menu。
2. `shop_two_b_presses_within_12_frames_both_register`：到 Shop 后 B@800、释放、B@804（间隔 4f<12）
   ⇒ 修复后 Shop→ShopInfo→（+33f）Transition（旧实现第二 B 被吞、停在 ShopInfo）。
   断言 ShopState Main→Info→Exited + 模式序列含 Transition。

### 5.3 确定性/签名影响

- 修复改变 `menu_latch` 内容（仅 START 位）与 `menu_gate.prev_held` 更新时机 ⇒ `Host::signature()`
  **数值会变**（语义扩展/修正，非回退）：需重算并更新记录值（`platform-shell.md` §4.2 selftest /
  §9 mock/replay 记录；shell-win 侧 selftest 新值同步 shell-porter；`plan-playable-loop.md` §10 记录值
  已获 Lead 授权更新，见 §8）。
- **两次运行确定性不变**（要求）；`stg01_replay` 两次全等仍须成立（replay 输入无 START 位时签名预期不变，
  实跑核实）；`Host::frame_view`/FPS/画布签名不受影响。
- **F1（task-31 复核低项）**：Shop/ShopInfo 改用 `$16` **同帧**按下沿后带来可观测 **−1f 响应时延**
  （ShopInfo 进入帧：旧 802 → **新 801**，相对旧 `$18` 锁存 +1f 路径提前 1f）；**出店 +33f 帧锚
  （press@1160→f1193）不受影响**——帧锚是响应常数、与触发键边沿源独立，已证边集/帧锚未动。

## 6. 未证域声明（fail-closed）

| 域 | 状态 | 处理 |
|---|---|---|
| Shop B 的 ROM 保持/边沿门（是否存在类似 START 的早退门） | **未证** | 采用 `$16` 按下沿、一次按下一次动作（最小 fail-closed）；不发明连发/保持重复；后续若取到 B 门证据再改 |
| 两连 B 的**最短合法间隔** | 未证（scenario 只证 60f 间隔） | 不设阈值；物理按下沿即派发，行为由输入决定 |
| 菜单导航（A/Down/Up 选择项）语义 | 未证（H-13） | 不接；仅 START/B 触发键，其余键不参与派发 |
| 标题段层 2 相位计时源 | 未证（S5-2 封存） | 仍用已证窗口 (450,550] 边界，不发明相位表 |
| 按住 START 连续两次（<12f） | ROM 锁存语义下不可区分（A 级机制边界） | 保持现有 `$18` 锁存模型（第二次需释放后 ≥1f 才能形成新锁存），不发明 |

## 7. 验收命令

```sh
cargo test -p game-shell                 # 新增 2 回归测试 + 全部既有
cargo test -p game-shell --test stg01_replay -- --nocapture   # 两次确定性/replay 签名记录
cargo test --workspace
cargo run -p shell-win -- --selftest-frames 300   # 两次逐行一致（新 host_signature 记录）
python gates/cli.py check                # CHECK: PASS
```

## 8. 实施状态（2026-10-18 回填）

**结论**：B1 已修（键盘域分离），跨键吞按与两连 B 短间隔吞按均消除；已证边集/帧锚未变、
game-core/game-platform 未改（`MenuInputGate` 契约原样复用）；改动仅在 game-shell 宿主派发层。

**改动文件（未 git add/commit，按任务卡）**

| 文件 | 改动 |
|---|---|
| `crates/game-shell/src/host.rs` | `tick()`：`menu_latch` 只锁存 `MENU_START`（对齐 A 级 `AND #$10`）；`dispatch_menu(frame, pressed)`：Title/Menu 只喂 START 给 `MenuInputGate`（层 2 相位解锁不变）；Shop/ShopInfo 用 `pressed & MENU_B`（`$16` 原始按下沿、键级）；其它模式不派发；+2 回归测试与脚本驱动 helper |
| `docs/findings/plan-menuinput-fix.md` | 本方案与实施回填 |
| `docs/spec/platform-shell.md` | §3.1 增"触发键分离（task-29 B1 修）"行 + 签名值链；§4.2/§9 更新记录值 |

`input.rs` 无需改（`menu_pressed_bitmap` 保留全键域通用函数，宿主侧按模式取用）。

**回归测试（2 条，修复前必红）**

1. `menu_start_not_swallowed_by_other_keys`：right@590 / A@595 / START@600 ⇒ 模式序列
   `Boot→Black→Title→Menu`（修复前停在 Title）。
2. `shop_two_b_presses_within_12_frames_both_register`：Shop 内 B@800、B@804（间隔 4f<12）
   ⇒ `Shop→ShopInfo→(+33f)Transition→(33f)StageStart`，`ShopState=Exited`（修复前停在 ShopInfo）。

**实跑结果**

- `cargo test -p game-shell --lib`：**43 passed / 0 failed**（原 41 + 新 2；既有菜单/模式/确定性全绿）。
- `python gates/cli.py check`：**CHECK: PASS**（含 `cargo test --workspace` 通过；docs-links 有效；
  rust-tests 只升不降）。
- 确定性（两次运行一致，值变化=语义修正，非回归）：
  - mock-300：`host` `0x63b5a6195287a6d8` → **`0x3085adda845a078d`**（canvas/advanced 不变）；
  - **STG-01 replay-3000：`host=0x521d397a6c609f34 canvas=0xdf75da247b4c3825 spawns=21
    engine_frames=1807` 未变**（replay 输入无 START 菜单键，C1 1207/1207、C6 571/571 同前）；
  - shell-win `--selftest-frames 300` ×2：stdout 逐行一致；`host_signature`
    `0xf51d5a517e7458b7` → **`0x5d7156266d6ac19d`**（canvas/display 不变）。

**记录值同步边界（本卡不 commit；提交待 task-31 复核 pass 后由 Lead 通知、精确 pathspec 执行）**

- `docs/spec/platform-shell.md` 已更新（本卡写范围）；
- `docs/findings/plan-playable-loop.md` §10 的 mock-300 记录值已由 **Lead 授权更新**为
  `0x63b5a6195287a6d8` → `0x3085adda845a078d`（附"B1 语义修正非回归"；该文件与主提交分开/一并
  均由精确 pathspec 提交）；
- shell-win `docs/findings/plan-shellwin-v1.md` §9.6 的时点值（`0xf51d…` → `0x5d7156266d6ac19d`）
  由 shell-porter 在 task-10/其文档更新时同步（本卡不动 shell-win 写范围）；
- 基线：Lead 已上调 `rust_test_count` 355→**357**（`cc4c692`），本卡不再处理 `data/status/`。

**遗留未证域**：见 §6（Shop B 保持门/最短合法间隔/菜单导航/相位计时源/START 连按 <12f）。
