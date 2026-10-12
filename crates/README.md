# crates/ —— Rust 重制工作区（AGENTS.md §2 红线为架构合同）

| crate | 职责 | 红线 |
|---|---|---|
| `game-data` | 资产/ROM 纯解析（无渲染）：文本记录、CHR、数据表 → 已证结构 | 只允许已证实的结构规则（证据指针进模块文档）；禁猜测性格式结论 |
| `game-core` | 纯逻辑（tick/状态机/武器/敌人/…） | **无 I/O、确定性**：禁真实时钟/线程 RNG/平台依赖；虚拟时钟 + GameRng |
| `game-platform` | trait 缝：Clock/GameRng/Renderer/Audio/Storage/Input | 零平台代码、零外部依赖 |
| `game-oracle` | 差分宿主（L1–L3/L5，S6 落地） | 比对逻辑不得为"通过测试"放宽阈值 |

依赖政策：`game-core` 只允许依赖 `game-platform`（纯 trait 缝）；
平台 crate（winit/sdl/tokio/rand/…）不得进入 `game-core`/`game-data`/`game-platform`。
机器把关：`crates/game-core/tests/contract.rs` 扫描源码禁词 + Cargo.toml 禁依赖，
`cargo test --workspace`（= rust-tests 门禁）不过即不许提交。

L1 golden 纪律（`game-data/tests/text_records.rs` 是范例）：
golden 输入 = **ROM 原始字节切片**（file 偏移进测试注释），期望 = **已证结构**
（`docs/findings/` 的 A 级证据逐条转写，测试注释带证据指针）；先写测试再实现。
ROM 缺失 ⇒ 测试 FAIL（fail-closed，禁静默 skip）。

时间/随机契约：`Clock::now_ms` 由 `FrameClock::tick()`（60Hz 帧）换算；
`GameRng` 序列由种子完全决定。算法语义属游戏行为，待 S4 spec 考证后锁定
（当前 `LcgRng` 是占位骨架，禁止据其写游戏数值）。
