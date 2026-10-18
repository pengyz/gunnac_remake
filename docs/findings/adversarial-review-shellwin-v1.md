# 对抗复核：shell/win（PC shell）v1 —— 证伪立场独立复核

> 复核方：`shell-verifier`（角色=verifier，任务卡 task-3）
> 被审对象：task-1 / `shell-porter`（自述「已 complete」）
> 复核基线：task-1 交付 tip=`16fb5b2`（`1be66e5` 方案 → `7b5e4da` 实现 → `a93704b` 回填 → `16fb5b2` 基线）；
> 复核过程中仓库被**并发成员**推进：`16fb5b2` → `b5e68b6`（task-6 复核修正）→ … → 复核完成时 `9bb0fd9`；
> `crates/shell-win/**` 自 `7b5e4da` 起未再变化（`git diff 7b5e4da..HEAD -- crates/shell-win` 为空），
> 故对 shell-win 的结论适用于交付态。
> 纪律依据：[AGENTS.md](../../AGENTS.md) §5「独立对抗复核」——不采信被审方自述、不复用其结论、独立重导证据。
> **verdict：pass（无阻塞项；7 条非阻塞 findings，其中 B1/S1 需后续任务卡修复，E2 需 Lead 说明）**

## 0. 复核独立性声明

1. 先读实现代码 → 自己跑命令/自己重算数字 → 最后才与方案 §9 自述逐条比对；自述中的任何签名/计数都不作为输入。
2. 未修改实现代码、未修改被审文档、未 `git add`/`commit`；唯一写入=本文件。
   临时日志/探针写在 `target/review-shellwin/`（可再生、不入库）。
3. 复核期间并发方改动已显式隔离：HEAD 由 `16fb5b2` → `b5e68b6`；另有 `docs/spec/l4-*.md` 未提交改动
   （不属本次对象）。因此 **workspace 级**数字为「复核时刻工作树」口径；**shell-win 单 crate** 与
   selftest 结果已在最终 HEAD 上重跑，与交付树一致。

## 1. 复核对象与写范围

| commit | 主题 | 触及文件（`git show --name-only`） |
|---|---|---|
| `1be66e5` | docs(platform)：shell-win v1 方案 + 未证域声明 | `docs/findings/plan-shellwin-v1.md` |
| `7b5e4da` | feat(platform)：shell-win v1 实现 | `Cargo.lock`、`crates/shell-win/{Cargo.toml,src/lib.rs,src/main.rs,tests/smoke.rs}` |
| `a93704b` | docs(spec)：platform-shell §4 回填 + 方案 §9 | `docs/spec/platform-shell.md`、`docs/findings/plan-shellwin-v1.md` |
| `16fb5b2`（交付 tip，不在自述 §9.1 列表） | chore(status)：`rust_test_count` 326→338 | `data/status/baselines/*` |

被审自述核心声明：六 trait 实现、winit 0.30 + softbuffer 主循环、删除旧 FrameDriver/GameHost、
CHR 背景 + 玩家 8×8、`--selftest-frames 300` 两次一致（host=`0xcde9dc99514b5f4b`、
canvas=`0x963ac9a776a57625`、display=`0xf68f9ffc0607cbad`、audio_writes=2、rom_len=262160）、GUI 标题正确、
未完成项（菜单派发/敌弹 HUD/音频 PCM/关卡装载）显式 TODO。

## 2. verdict 与总评

**pass。** 理由（全部为自导证据，见 §3/§6）：

- 架构合规：shell-win **无**任何 game-core 逻辑调用（4 处 `game_core::` 均为类型引用）；主循环唯一驱动入口
  是 `game_shell::Host::advance`；平台依赖未进入 game-core/game-platform/game-shell/game-data。
- 六 trait 均为真实实现（非空实现）：Renderer/Audio/Storage/AssetSource/Input/Lifecycle 逐项有可执行断言。
- 确定性：`--selftest-frames 300` 两次 stdout **字节级相等**（SHA256 相同），且与自述数字逐项一致；
  HEAD 移动后第三次重跑仍一致。
- 构建/测试/门禁：`cargo build -p shell-win`、`cargo test -p shell-win`（20/20）、`cargo test --workspace`、
  `python gates/cli.py check` 均自跑通过（CHECK: PASS，exit 0）。
- GUI：实测 `MainWindowTitle="gunnac-remake (Stage A)"`、stderr `mode=Boot stage=1 frame=1`。
- 未证不发明：敌/弹/HUD/文本不画；音频 PCM 静音占位显式（`silent()=true` + 全 0 样本）；主循环不投递
  `ModeEvent`；未证输入组合在进入 game-core 前被 `resolve_directions` 消解。
- 规格回填与实现相符（§4.1 逐件可对码对实测）；Evidence/Confidence 齐（85/88/90/97）。

非阻塞 findings 集中在：一条 Windows 专属 fail-closed 边角（B1）、资产根规格缺口（S1）、
三条文档计数/口径不精确（E1/E3/E4）、一条基线 commit 授权链不明（E2）、平台未测面声明（P1-P3）。
均不构成对「已实现能力」的夸大，故不改变 verdict；但 B1/S1 必须在下一张 shell 卡内闭合。

## 3. 被确认清单（自己重导的证据）

| # | 声明 | 我的证据（命令/文件） | 结果 |
|---|---|---|---|
| C1 | 只经 game-shell 宿主驱动 game-core，无直连逻辑 | `git grep -n "game_core::" -- crates/shell-win` → 仅 `lib.rs:29`（`ModeEvent` 类型）、`lib.rs:682`（`Mode` 类型）、`main.rs:125`（`Mode` 类型）；无 `player::tick`/`engine::stage_tick` 调用；驱动唯一入口 `main.rs:255-257` `host.advance(...)` | 确认 |
| C2 | 平台依赖只在平台壳；game-* 零平台依赖未被破坏 | `git grep winit/softbuffer` 仅 `crates/shell-win/Cargo.toml` + **既有** `crates/game-desktop/Cargo.toml`（243d0a6 起，pre-existing）；game-core/game-platform/game-shell/game-data 的 src 中命中仅为注释与契约测试禁词表 | 确认（措辞需更正，见 E3） |
| C3 | 六 trait 真实实现 | `lib.rs:167`（Input）、`246`（AssetSource）、`286`（Storage）、`369`（Audio）、`417`（Lifecycle）、`610`（Renderer）；`main.rs:88`（窗口 Renderer）；tests 逐 trait 断言（`tests/smoke.rs:44-252`） | 确认 |
| C4 | Audio 真实消费 ApuWrite 流；PCM=静音占位显式 | `game-shell/src/host.rs:416` 每逻辑帧 `audio.flush`；`audio.rs:61-70` 逐事件 `write_apu`+`end_frame`；实测 `audio_writes=2 audio_frames=300`（仅来自自检注入的 2 条，生产者为空，方案 §6 已声明）；`PcAudio::silent()=true`、`synth_pcm` 全 0（`lib.rs:350-360`） | 确认（非「假装有声」） |
| C5 | Storage 路径 cfg 差异 + 字节块语义 | `lib.rs:176-193`：Windows `%APPDATA%/gunnac`、unix `~/.local/share/gunnac`；`slot_path`=`slot{n}.sav`；单测 temp 目录往返/空槽 fail-closed；**不自动保存**（GUI/selftest 均未调用） | 确认（unix 分支未在本机运行，见 P2） |
| C6 | AssetSource fail-closed | `lib.rs:237-253` 拒绝绝对路径/`..`；缺失→`None`；单测 `tests/smoke.rs:221-235`；实测 `rom_len=Some(262160)`（=CHR 域最小值 0x40010，恰好满足 fail-closed 长度门） | 确认（Windows 边角见 B1） |
| C7 | 输入 L+R/U+D 在进入 core 前消解为 0 | `game-shell/src/host.rs:372-374` 先 `resolve_directions` 再 tick；`game-core/src/player.rs:50-65` 对 L+R/U+D 组合 **panic**；selftest 第 4/5 帧脚本注入 L+R、U+D（`lib.rs:651-670`）→ 300 帧完成无 panic，签名可复现 | 确认 |
| C8 | 60Hz 固定步 | `FRAME_NS=16_666_666`；`Host::advance` 每采样折算 0..=4 帧；selftest 每采样 `(f+1)*FRAME_NS` ⇒ 实测 `advanced=300 / presents=300 / frame=300` | 确认 |
| C9 | selftest 两次 stdout 逐行一致且数值=自述 | `cargo run -q -p shell-win -- --selftest-frames 300` ×2：stdout SHA256 均 `8334F04692C1856F2EB8F527E831862B173909075694BC22D8509AAD9A235AB0`，8 行逐字相同；HEAD 移动到 `b5e68b6` 后第三次仍同 hash；退出码均 0 | 确认 |
| C10 | 构建/测试/门禁真绿 | `cargo build -p shell-win`→exit 0；`cargo test -p shell-win`→ lib 3 + smoke 17 = **20 passed / 0 failed**；`cargo test --workspace`→exit 0（55 个 test 块汇总 337 passed + 1 ignored，0 failed）；`python gates/cli.py check`→exit 0、`CHECK: PASS`、`PASS=11 FAIL=0 SKIP=1 TOLERATED=0` | 确认（门禁 WARN 归因见 §6 注） |
| C11 | GUI 实跑标题正确 | 启动 `target/debug/shell-win.exe`：4s 后进程存活，`MainWindowTitle="gunnac-remake (Stage A)"`，stderr `shell-win: mode=Boot stage=1 frame=1`；换 `crates/shell-win` 为 cwd 再跑结果相同（资产回退生效），且未创建 `%APPDATA%/gunnac` | 确认 |
| C12 | 旧 FrameDriver/GameHost 已删；未证域不画 | `git grep FrameDriver/GameHost/AudioSink -- crates` 仅剩文档/注释引用，无代码；`git show 7b5e4da^:crates/shell-win/src/lib.rs` 旧骨架确含三类型；`game-shell/src/render.rs:83-87` 只清屏+玩家 8×8；shell 侧 `compose_canvas` 只做 CHR 背景+非清屏像素覆盖；全仓无敌人/子弹/HUD 绘制路径 | 确认 |
| C13 | 纪律：写范围/未动红线/Evidence | 三 commit 文件清单见 §1，未触 rom/reference/data/game-core/game-platform/game-shell/game-desktop；`git diff 7b5e4da..HEAD -- crates/shell-win` 为空；三 commit 均含 Evidence+Confidence；spec §4.1/§4.2 与实现、实测逐条相符；`rom-integrity`/`seed-integrity` 门禁 PASS | 确认（E2/E4 例外） |

## 4. 被证伪 / 需更正清单

| # | 原表述 | 复核结论 | 分类 |
|---|---|---|---|
| X1 | `crates/shell-win/Cargo.toml:6`、`plan-shellwin-v1.md:218`「平台依赖（winit/softbuffer）只允许出现在这里/仍只在本 crate」 | **不严格成立**：既有 `game-desktop` 也依赖 winit+softbuffer（spec §4 自认「game-desktop 同族」）。应表述为「未新增到 game-core/game-platform/game-shell/game-data」。非本次回归（pre-existing） | 证据不足（口径） |
| X2 | `plan-shellwin-v1.md:224`「`tests/smoke.rs`：20 项契约测试」 | **不实**：`smoke.rs` 实际 17 项 `#[test]`，lib 内 3 项，crate 级共 20；同文 `:195`「lib 3 + smoke 17」才是正确口径（自相矛盾） | 证据不足（计数） |
| X3 | `plan-shellwin-v1.md:52`「`resize` 更新缓冲」 | **不精确**：`main.rs:108-110`→`lib.rs:624-626` 只记录 `size` 元组；softbuffer surface 是在 `blit()`（`main.rs:73`）按窗口尺寸 resize | 证据不足（口径） |
| X4 | `plan-shellwin-v1.md:188`「commit 列表（本任务写范围）」仅 3 个；`:241-242`「`data/status/baselines/` 属写范围外 ⇒ 报 Lead 处置」 | 交付 tip 实际含 **第 4 个 commit `16fb5b2`** 修改 `data/status/baselines/*`（敏感路径、任务卡写范围外）；且基线已被上调（与「报 Lead 处置」措辞时序矛盾）。**内容经复核正确**（见 §7 注），但执行方/授权链无法从仓库判定（全员共用 git 身份） | 纪律 / 证据不足 |
| X5 | 文件清单 `plan-shellwin-v1.md:216-225` | 未列 `Cargo.lock`（`7b5e4da` 实改，依赖同步所必需、无害）；写范围也未列根 `Cargo.lock` | 证据不足（清单） |

## 5. findings（四分类）

### 5.1 证据不足（E）

- **E1（=X2，Low）**：测试计数归属错误。
  - 位置：[plan-shellwin-v1.md:224](../../docs/findings/plan-shellwin-v1.md)。
  - 修正：改为「`cargo test -p shell-win` 共 20 项 = lib 3 + `tests/smoke.rs` 17」；commit `7b5e4da` message 的
    「tests：20 项契约测试」同口径（crate 级，可保留但建议注明拆分）。
- **E2（=X4，Low，需 Lead 说明）**：`16fb5b2` 基线 commit 在任务卡写范围外，且未进入方案 §9.1 commit 列表/§9.3 文件清单；
  方案 §9.4-8 明言「报 Lead 处置」。请 Lead 确认该 commit 由谁/何种授权执行；若为实施方自行提交，应记为流程偏差并补记入
  方案 §9.1。**内容正确性已确认**：`baseline.json` 实算 sha256=`85ba0de8…` 与 `SHA256SUMS` 锚一致；
  `data/status/baselines/baseline_history.jsonl` 末条 `new=338`；`git grep -c '#\[test\]' a93704b -- crates` 合计 **338** = 新基线，
  无下调、无删除。
- **E3（=X1/X3，Info）**：两处口径不精确（平台依赖范围、`resize` 语义）。修正见 §4 表；`Cargo.toml:6` 注释与
  `plan-shellwin-v1.md:218/52` 同步改。
- **E4（=X5，Info）**：`Cargo.lock` 未在文件清单/写范围声明。修正：在方案 §7 表补一行「根 `Cargo.lock`：依赖同步（仅 +game-shell/+game-data）」。
- **E5（Info）**：`plan-shellwin-v1.md:195`「rust-tests 门禁实测 338」未写口径（门禁= `#[test]` 静态计数；运行时为
  337 passed + 1 ignored）。建议补一句口径，避免后续把两种数字当矛盾。

### 5.2 实现错误（B）

- **B1（Low，Windows 专属 fail-closed 缺口）**：`FsAssets` 的路径校验未拒绝 Windows **drive-relative 前缀**。
  - 位置：[lib.rs:237-244](../../crates/shell-win/src/lib.rs)（`safe_relative`）、`lib.rs:246-253`（`read_all`）。
  - 证据（自建探针，复刻同一判定逻辑）：`"C:foo"`→`is_absolute()=false`、无 `ParentDir/RootDir` ⇒ `safe_relative=true`；
    `PathBuf::from("D:\workspace\gunnac_remake").join("C:foo") == "C:foo"`（**逃逸资产根**，按 C: 盘当前目录解析）。
    `"C:../../AGENTS.md"` 因含 `ParentDir` 被拒，但 `"C:Windows/x"` 一类不含 `..` 的前缀路径可通过。
  - 影响评估：当前唯一调用点传编译期常量 `"rom/gunnac_cn.nes"`，**不可由外部输入触发** ⇒ 不构成 v1 阻塞；
    但与「fail-closed（绝对/`..`/缺失）」的声明意图不完全一致。
  - 修正建议：`safe_relative` 增加 `Component::Prefix(_) => false`（Windows 语义；unix 不产生 Prefix）；
    `tests/smoke.rs:221-235` 增 `assets.read_all("C:foo")==None`、`"D:foo\\x"==None` 用例；
    `docs/spec/platform-shell.md` §4.1 资产行注明「拒绝绝对/前缀/`..`/缺失」。

### 5.3 规格缺口（S）

- **S1（Medium，后续卡必做）资产根契约与实现不一致**：`b5e68b6`（task-6 复核修正）把 spec §4
  [platform-shell.md:109](../../docs/spec/platform-shell.md) 定为「PC 资产根=**可执行文件所在目录**（可回退 `exe/../rom/` 与
  `CARGO_MANIFEST_DIR`），**不得依赖进程 cwd**」，并在 §4.1 登记 v1 为「相对 cwd 定位、后续加固」。
  实测实现 [lib.rs:199-208](../../crates/shell-win/src/lib.rs) 是 **cwd 优先**、回退 `CARGO_MANIFEST_DIR`，**无 exe 目录逻辑**。
  - 现状：开发态 `cargo run`/双击 `target/debug/shell-win.exe` 均可用（已实测两种 cwd 都装载成功）；
    但发布态（脱离仓库树）会退化为黑背景 fail-closed。
  - 修正建议（下一张 shell 卡）：`repo_root()` 改为 exe 目录优先（`std::env::current_exe()` → `exe/rom/…`、`exe/../rom/…`），
    再回退 `CARGO_MANIFEST_DIR`（仅 `cfg(debug_assertions)`）；启动时**始终**打印实际资产根（当前仅失败时打印，
    `main.rs:146-165`），便于验收；补集成测试（异 cwd + 期望资产根）。在 S1 闭合前，spec 不应把「exe 目录」写成已实现能力
    （当前 §4.1:126 已正确标注 v1=相对 cwd，无夸大）。
- **S2（Low，口径）Audio 设备输出**：spec §4 [platform-shell.md:108](../../docs/spec/platform-shell.md) 仍写「平台 `Audio` 实现（cpal 族）」，
  而 v1 无 cpal、无设备流（任务卡明确允许：PCM=静音占位显式）。§4.1:125 已如实写 `PcAudio` 静音占位，建议 §4 行
  改为「v1=写流消费+静音占位；cpal/合成器=AUD-2/3 后续」以消除「已接声卡」的误读。
- **S3（Low）**「Win/Linux 同源」未在 Linux 侧验证：`cfg(unix)` 存储分支、unix 路径与 winit/softbuffer Linux 后端均只经代码审读。
  建议后续在 Linux CI 跑 `cargo test -p shell-win`（或在可用环境 `cargo check --target`），否则「同源」仅为设计声明。
- **S4（Info）** GUI 视觉（整数缩放/黑边/CHR 占位像素）无自动证据，属手工验收项（方案 §9.4-9 已如实声明；
  L4 门禁不覆盖 shell 呈现）。建议下一张卡补「GUI 截图 + 与 game-desktop 骨架同口径比对」。
- **S5（Info）双状态源/时序未接线**（属 game-shell/spec 层，非 shell-win）：`b5e68b6` 已把「除 Combat→StageClear 外 ModeEvent
  由平台/上层显式投递」「关始 intro 240f/A2/A3 细段未接线」登记为规格缺口；shell-win v1 主循环确实**不**自动派发，
  与此一致，但意味着**窗口当前不可玩**（见 §8）。

### 5.4 平台差异（P）

- **P1（=B1）**：Windows drive-relative 前缀行为（unix 下 `C:foo` 只是普通相对文件名，无逃逸）——平台特有。
- **P2**：`storage_dir` 的 `cfg(unix)` 分支在本机（Windows）不参与编译，未运行验证；路径语义（`~/.local/share/gunnac`）仅代码审读。
- **P3**：GUI 行为依赖桌面会话（本机实跑已证；无头环境不可测）；焦点→`Suspend/Resume` 是桌面最小策略，
  与 Android `onPause/onResume` 生命周期语义不同（spec §5 已分别登记，未混用）。
- **P4**：`repo_root()` 的 cwd 优先策略使「从资源管理器双击」与「从仓库根 `cargo run`」行为不同（开发态靠
  `CARGO_MANIFEST_DIR` 回退掩盖）——归入 S1 修复。

## 6. 自己执行的命令与输出摘要

工作目录：`D:\workspace\gunnac_remake`（Windows）；日志：`target/review-shellwin/*.log|out|err`。

| # | 命令 | 输出摘要 |
|---|---|---|
| 1 | `git log/show --stat` 三 commit | 文件清单与 §1 一致；仅 `1be66e5/7b5e4da/a93704b` 属 task-1；`16fb5b2` 额外 |
| 2 | `cargo build -p shell-win` | exit 0；仅既有 `game-core scroll_frames dead_code` 警告 |
| 3 | `cargo test -p shell-win` | lib `3 passed`、smoke `17 passed`、doc-tests 0；**0 failed**，exit 0（20/20） |
| 4 | `cargo test --workspace` | exit 0；55 个 test 块；汇总 `337 passed + 1 ignored / 0 failed`（该树含并发方未提交测试断言） |
| 5 | `cargo run -q -p shell-win -- --selftest-frames 300`（×2，仓库根） | exit 0；stdout 8 行；两次 SHA256 均 `8334F046…AB0`（字节相等）；数字见 §7 |
| 6 | 同上（cwd=`crates/shell-win`） | exit 0；输出与 #5 完全相同（资产根回退生效） |
| 7 | HEAD 移动到 `b5e68b6` 后重跑 #5 | exit 0；输出 hash 仍 `8334F046…AB0`（交付签名对当前 HEAD 仍成立） |
| 8 | `target/debug/shell-win.exe`（GUI，仓库根 cwd；4s 后查进程） | 存活；`MainWindowTitle="gunnac-remake (Stage A)"`；stderr=`shell-win: mode=Boot stage=1 frame=1`；随后 kill |
| 9 | 同上，cwd=`crates/shell-win` | 无「资产缺失/CHR 不可用」stderr；标题同；未创建 `%APPDATA%/gunnac` |
| 10 | `python gates/cli.py status` | `STATUS: OK`；PASS=11、SKIP=1（addr-bridge，显式打印）；`rust-tests PASS`「339（基线 338）」PENDING |
| 11 | `python gates/cli.py check` | exit 0；`CHECK: PASS`；`PASS=11 FAIL=0 SKIP=1 TOLERATED=0`；唯一 WARN=`ratchet:rust_test_count 338/339`（后由并发方 `fe8ce68` 消除） |
| 11b | 本复核文件写入后复跑 `check` | `FAIL=1 docs-links`，明细**仅**并发方未跟踪 `.tmp_signing_1be66e5.md`/`.tmp_visual_1be66e5.md`；本文件链接无报错（归因见 §6；未删除他人文件） |
| 12 | `git grep -n "game_core::" -- crates/shell-win` | 4 处，全为类型引用（`ModeEvent`/`Mode`），无 tick 逻辑调用 |
| 13 | `git grep -n "winit\|softbuffer" -- crates/game-core crates/game-platform crates/game-shell crates/game-data` | src 命中仅注释；其余为契约测试禁词表；依赖仅 shell-win + 既有 game-desktop |
| 14 | `git grep -n 'FrameDriver\|GameHost\|AudioSink' -- crates` | 无代码残留（仅文档/注释历史引用） |
| 15 | `git grep -c '#[test]' <rev> -- crates` 汇总 | `ff5aa7c`=326、`a93704b`=338、`16fb5b2`=338、当前 `b5e68b6`=**339** |
| 16 | 自建 rustc 探针（复刻 `safe_relative`；文件在 `target/review-shellwin/`） | `"C:foo" safe=true join="C:foo"`（逃逸）；绝对/`..`/UNC 均 `safe=false` |
| 17 | `git grep -n velocity_from_input` + 读 `player.rs:50-65` | core 对 L+R/U+D 组合 `panic!`（fail-closed 域）；game-shell 已先消解 |

**门禁 WARN 归因与当前脏树说明（重要）**：
1) 复核首轮 `python gates/cli.py check` = `CHECK: PASS`、exit 0，唯一 WARN=`ratchet:rust_test_count 338/339`。
   该 WARN **不是 task-1 缺陷**——交付 tip `a93704b/16fb5b2` 树内 `#[test]` 计数=338=基线；+1 来自并发
   task-6 修正 commit `b5e68b6`（`trait_contract.rs` 新增 1 测）。复核期间该 WARN 已被并发方
   `fe8ce68`（`rust_test_count 338→339`，含 Evidence/Confidence、未下调）消除，故最终基线口径为 339/339。
2) 在我把本复核文件写入工作区后复跑 `check` 得到 **docs-links FAIL**，原因是**另一并发成员**在仓库根留下了
   两个未跟踪临时文件 `.tmp_signing_1be66e5.md`、`.tmp_visual_1be66e5.md`（其内含指向不存在目标的链接）；
   FAIL 明细**只列这两个文件**，不涉及 task-1 交付文件与本复核文件（本文件链接经该门禁扫描无报错）。
   我未删除他人文件（跨写范围/他人工作区纪律）——该 FAIL 归因于并发方临时产物，与本复核对象无关；
   待并发方清理后 `check` 即可恢复 PASS（首轮已证）。
3) workspace 级测试为「复核时刻树」口径（并发方改动后并入 `b5e68b6`）；shell-win 单 crate 与 selftest
   已在最终 HEAD（`9bb0fd9` 之前一次为 `b5e68b6`）复跑，结论不依赖脏树。

## 7. 重算数字表

| 指标 | 自述值 | 我实测值 | 结论 |
|---|---|---|---|
| `--selftest-frames 300` 退出码 | 0 | 0（×2，另最终 HEAD 再×1） | ✅ |
| stdout 两次一致 | 逐行一致 | **SHA256 相等**（字节级） | ✅ |
| frames / advanced / presents | 300/300/300 | 300/300/300 | ✅ |
| mode / stage / frame | Combat/1/300 | Combat/1/300 | ✅ |
| host_signature | `0xcde9dc99514b5f4b` | `0xcde9dc99514b5f4b` | ✅ |
| canvas_signature | `0x963ac9a776a57625` | `0x963ac9a776a57625` | ✅ |
| display_signature | `0xf68f9ffc0607cbad` | `0xf68f9ffc0607cbad` | ✅ |
| audio_writes / audio_frames | 2 / 300 | 2 / 300 | ✅ |
| rom_len | `Some(262160)` | `Some(262160)`（=0x40010，CHR fail-closed 门下限） | ✅ |
| `cargo test -p shell-win` | lib3+smoke17=20 | 3+17=20，0 failed | ✅ |
| workspace `#[test]` 计数（门禁口径） | 338（基线 326） | `a93704b` 树=338；基线已上调 338 | ✅（`16fb5b2` 正确） |
| workspace 运行态（交付树口径） | 「全绿」 | 337 passed + 1 ignored（合计 338），0 failed | ✅ |
| gates check | PASS=11 FAIL=0 SKIP=1 TOLERATED=0 | 首轮实测同（exit 0、`CHECK: PASS`）；唯一 WARN 后由 `fe8ce68` 消除；次轮 FAIL 归因并发方临时文件（§6 注） | ✅（交付态） |
| GUI 标题 | `gunnac-remake (Stage A)` | 实测同；stderr `mode=Boot stage=1 frame=1` | ✅ |
| 三 commit Evidence/Confidence | 有 | `1be66e5`=85、`7b5e4da`=88、`a93704b`=90 均含 | ✅ |
| 写范围合规 | 仅任务卡范围 | 3 commit 未触 rom/reference/data/game-*/game-desktop；例外=`Cargo.lock`+第 4 个基线 commit（E2/E4） | ⚠️（非阻塞） |

## 8. 是否解锁「可玩闭环」下一波

**结论：解锁的是「可运行宿主容器 + 六 trait 注入面 + 确定性回归锚 + 自检/门禁闭环」，不解锁「可玩性」。**

- 已可复用的下一波基座：窗口/呈现/缩放黑边、60Hz 固定步、`Host` 状态机与已证边集、输入消解、资产/存储/生命周期端口、
  确定性签名（回归锚）、`--selftest-frames` 自检协议。
- 当前窗口为什么不可玩（均已如实登记，非隐藏缺口）：主循环不投递 `ModeEvent`（停 `Boot`）；关卡数据未装载
  （`StageState` 空）；敌/弹/HUD 不画；音频静音占位；存储不自动保存。
- 建议下一波任务卡（依赖顺序，均需新卡+独立复核）：
  1. **菜单/输入→ModeEvent 派发**（game-core flow 已证边；需 `$16` bit4/bit5 菜单位图 + 关始 intro/A2/A3 时序定谳；
     不得静默硬编码 240f/400f/265f）；
  2. **game-data 关卡装载 → `Host::stage_state_mut` 接线**（先锁 `StageState` 装载语义/形状与双状态源同步契约）；
  3. **资产根加固（S1/B1）**——小卡，可与上面并行；
  4. **敌弹渲染 / 音频合成器（AUD-2/3 + cpal）**——须排在 H-01/H-16/Boss 等未证战役取证之后，禁止先画后证。

## 9. 复核边界（未覆盖项，不声称已验）

1. 未做真机/真窗口**像素截图对拍**（缩放/黑边/CHR 背景像素为代码+单测证据；GUI 仅验证标题/启动/状态打点）。
2. 未在 Linux 上编译/运行（`cfg(unix)` 分支、Linux winit/softbuffer 后端未实测）。
3. 未验证 Android（本任务明确非目标）。
4. 未对 `game-desktop` 旧骨架做横向像素一致性比对（属 L4 战役，不属 shell v1 判据）。
5. workspace 级测试/门禁为「复核时刻工作树」口径（并发改动已注明）；`crates/shell-win` 单独结论不依赖该口径。

> 本复核只写本文件；未改动任何实现/被审文档，未执行 `git add`/`git commit`。
> 复核方与被审方非同一 agent（AGENTS.md §5）；如对 findings 有异议，按「原地标注日期更正」文化在原文回应后复审。
