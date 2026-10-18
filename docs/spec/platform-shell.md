# 平台抽象层 + Shell 架构规格（2026-10-18 v1）

> 用户裁定：多平台支持（Windows/安卓/其他）——game-platform trait 缝=唯一对接面；
> 第一个实现=shell/win（正式把游戏跑起来）。本规格=**接口锁定依据**，接已证语义不发明。

## 1. 分层合同（AGENTS.md §2 延伸）

```
game-core（纯逻辑，零 I/O 零平台依赖）
  ↓ 依赖 trait
game-platform（trait 缝：零平台代码零外部依赖——已立 70 行，本阶段锁定占位项）
  ↑ 实现 trait
shell/win（平台实现 1：Win32——平台依赖（windows-rs）仅此 crate）
  （未来：shell/android、shell/wasm… 同缝实现）
```

## 2. trait 锁定（L4/L5 落地后语义锁定——未证项显式占位不发明）

| trait | 语义（已证面） | 状态 |
|---|---|---|
| `Clock` | 虚拟时钟 now_ms（确定性 60Hz） | ✅ 已立 |
| `GameRng` | 确定性 LCG（LcgRng 已实现） | ✅ 已立 |
| `Input`/`Buttons` | A/B/START/SELECT/↑↓←→（秘籍掩码位图 A 级：bit0=LEFT/bit1=RIGHT 族注） | ✅ 已立 |
| **`Renderer`** | 256×240 逻辑帧呈现（RGBA Canvas——game-desktop Canvas 同语义）+整数倍缩放保比例（黑边）+present | 🔒 本阶段锁定 |
| **`Audio`** | ApuWrite 流消费（(frame,reg,value) 三元组——L-Audio 写流 hash 面同源）→ 混音输出；无合成器期=流透传/静音占位显式 | 🔒 本阶段锁定 |
| **`Storage`** | 字节块存取（save/load——存档面预留，语义未证=纯字节存取不发明） | 🔒 本阶段锁定 |

**锁定原则**：trait 形状=已证语义最小面（Canvas 像素/ApuWrite 流/Buttons/字节块）；
绘制细节/混音算法=实现侧语义（shell 自有），trait 不承载未证行为。

## 3. shell/win 架构

| 件 | 设计 |
|---|---|
| 窗口 | Win32 CreateWindowExW + 消息泵（windows-rs——依赖限定本 crate）|
| 帧驱动 | 60Hz 固定步（QPC 计时+Sleep 节流）→ game-core 状态推进（stage_tick 族）→ 渲染合成 → present |
| 帧呈现 | 256×240 RGBA → StretchDIBits 整数倍缩放（GDI DIB——保比例黑边） |
| 输入 | WM_KEYDOWN/UP → Buttons 映射（Z/X/Enter/RShift/方向键 预定） |
| 音频 | waveOut（Win32 基础面）消费 ApuWrite 流→PCM（合成器未证期=静音占位显式） |
| 存储 | %APPDATA%/gunnac/ 存档文件（Storage trait） |
| 游戏宿主 | title→设置→八关→结局 状态机（game-core flow/stage 状态接线——已证 F1-F13 边集） |

## 4. 工程纪律
- 平台依赖（windows-rs）**仅 shell/win**；game-core/game-platform 零依赖照守
- TDD：trait 契约测试（game-platform）+ shell 冒烟自测（窗口创建/帧循环/输入映射单元级）
- 双关照旧：实现→对拍（cargo test 全集+gates 12/12）→独立对抗复核
- Windows 目标编译验证：x86_64-pc-windows-gnu 交叉 check（Linux 宿主 CI 面）；真机运行验证=用户侧
