# 对抗复核：PC shell findings 收口（task-18）+ diff_visual --strict（task-21）——adversarial-review-shell-fixes

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不得复用被审方结论**）
> 被审对象：`ddb1558`（task-18：shell-win findings 收口，6 files +302/−71）、`64b3a44`（task-21：`tools/diff_visual.py --strict`，2 files +133/−6）。
> 复核时 HEAD `0cf42f0`；两 commit 均已入库；工作区无未提交代码改动（仅并发 docs）。纪律：AGENTS.md §3/§5；本文只写本文件、未 git add/commit。
> 复核方法：先自己跑命令/构造探针，再与 commit 自述/plan 比对；未复用被审方输出。

---

## 0. VERDICT：**pass（4 条非阻塞 findings/info；0 阻塞）**

- **B1（drive-relative 逃逸）成立且已修**：独立探针（自建临时 crate 调 `FsAssets::read_all`）11 条路径全部符合 fail-closed：`C:foo`/`D:foo\x`/`C:`/`C:/x`/`C:\Windows\x`/`\\?\C:\x`/`\\server\share\x`/`..\AGENTS.md`/`rom/../../AGENTS.md`/`rom\..\..\AGENTS.md` ⇒ **None**；`rom/gunnac_cn.nes` ⇒ **Some(262160)**。smoke/unit 断言真实执行（见 §1）。
- **S1（资产根 exe 优先）成立**：`asset_root_from_exe` 三布局单测通过；异 cwd 子进程（自跑）exit 0、`rom_len=Some(262160)`、stderr `shell-win: 资产根=D:\workspace\gunnac_remake`；selftest 300 帧 ×2 stdout 逐行一致；启动两模式都打印资产根（代码+实跑）。
- **task-21 成立且默认行为不变**：默认 `exit 0`（`1 通过/22 比对；分类记录 21；未分类 0`，与改前一致、`git status oracle/screens` 空）；`--strict` 现基线 **exit 1 + `STRICT: FAIL diff=280132`**；全对相同临时样本 `--strict` **exit 0 + PASS diff=0**；未知参数 exit 2；目录缺失 exit 2；`gates/cli.py` 未改（最后触碰 `a11161d`）；visual 门禁仍 PASS（走默认模式）。
- **测试/门禁**：`cargo test -p shell-win` = lib **5** + smoke **19** = **24 passed / 0 failed**（与 `ddb1558` message、plan §9.6 一致）；`cargo test --workspace` 全绿；`gates/cli.py check` = **CHECK: PASS（PASS=11 FAIL=0 SKIP=1）**。
- **写范围/纪律**：两 commit 均未动 game-core/game-platform/game-shell/rom/reference/data；`gates/cli.py`/`oracle/` 未改；ddb1558 的并发误卷仅 **platform-shell.md 一处**，且三方留痕（knowledge/plan-playable-loop/b94ede8 message，见 §5）。
- **非阻塞 findings**：F1 plan 计数时点未标（§9.2/§9.3 仍 20，实测 24；§9.6 已写 24）；F2 误卷留痕不在 ddb1558 自身 message；F3 E2 授权链为任务卡指令级证据（可核性）；F4 `tools/diff_visual.py` docstring 仍含"候选"旧文案 + strict 的"外差异"与 diff 计数口径提示（均为 info/文档层）。

---

## 1. B1 复核（Windows drive-relative 前缀）

**代码**（`crates/shell-win/src/lib.rs`）：`safe_relative` L268-278 新增 `Component::Prefix(_)` 拒绝；`read_all` L283-289 先取 `root?` 再 `safe_relative` 判定；`FsAssets::new` 显式根/`detect` 自动根均走同一判定。

**独立探针**（自建 `%TEMP%/shellprobe` crate，`shell-win` path 依赖 + `game_platform::AssetSource`，`FsAssets::new(仓库根)`）：

| 路径 | `read_all` 结果 | 期望 |
|---|---|---|
| `C:foo` | **None** | None（B1 核心） |
| `D:foo\x` | **None** | None |
| `C:` / `C:/x` / `C:\Windows\x` | **None** | None |
| `\\?\C:\x`（verbatim）/ `\\server\share\x`（UNC） | **None** | None |
| `..\AGENTS.md` / `rom/../../AGENTS.md` / `rom\..\..\AGENTS.md` | **None** | None |
| `rom/gunnac_cn.nes` | **Some(262160)** | 可读（正例不受影响） |

**断言真实执行**：`cargo test -p shell-win` 输出 24 项（名字含 `tests::safe_relative_rejects_escape_paths`、`fs_assets_reads_relative_files_and_fails_closed`、`fs_assets_detect_resolves_dev_asset_root`、`selftest_locates_assets_from_foreign_cwd`）；定向复跑 `--lib safe_relative`=1 passed、`--test smoke fs_assets`=2 passed。smoke 断言与 commit message 一致（真实 `read_all`，非 mock）。
> 边界：NTFS 符号链接/junction 指向根外不在 B1 声明范围（本卡只声明 drive-relative/前缀/`..`/绝对路径）；如需防 symlink 逃逸应另立 finding（信息）。

## 2. S1 复核（资产根 exe 目录优先、不依赖 cwd）

- **解析顺序**（`lib.rs` L204-249）：`asset_root()` = `current_exe()` → `asset_root_from_exe`（`exe_dir/rom` → `exe_dir/../rom`，均要求 `ROM_ASSET_PATH` 为文件）→ **仅 `#[cfg(debug_assertions)]`** 回退 `CARGO_MANIFEST_DIR` 上两级 → 否则 `None`（fail-closed）。`FsAssets::detect()`（L257）未解析⇒`root=None`⇒`read_all` 全 `None`（L283）。
- **纯函数单测**（`lib.rs` L~800-840）：布局 A（ROM 与 exe 同目录）⇒ exe 目录；布局 B（ROM 在 exe 上一级）⇒ exe/..；两候选均无 ⇒ None；实跑 passed ✓。
- **异 cwd 子进程**（实跑）：`target/debug/shell-win.exe --selftest-frames 5`，`cwd=%TEMP%/<随机>` → exit 0、stdout `rom_len=Some(262160)`、stderr `shell-win: 资产根=D:\workspace\gunnac_remake`（debug 回退命中；**不依赖 cwd** 成立）。smoke 同名测试（`CARGO_BIN_EXE_shell-win` + 临时 cwd）实跑 passed ✓。
- **启动始终打印**：窗口模式 `main.rs` L153-156（load_assets 成功/失败两分支）；selftest 模式 L344；实跑 stderr 见上 ✓。
- **确定性**：`--selftest-frames 300` 连跑两次 stdout 逐行全等（`host_signature=0xf51d5a517e7458b7`、`canvas_signature=0x963ac9a776a57625`、`display_signature=0xf68f9ffc0607cbad`、`audio_writes=2`、`rom_len=Some(262160)`、`selftest: OK`），exit 0 ✓。
> 边界（信息）：release 构建"无 CARGO_MANIFEST_DIR 回退 ⇒ None"为代码阅读结论，未实跑 release exe（本机仅 debug 构建跑测试；发布态行为建议最终打包验证）。

## 3. task-21 复核（`diff_visual --strict`）

**实现**（`tools/diff_visual.py` L156-281）：`main(argv)` 解析 `--strict`；未知参数 exit 2；strict 下任一分对差异/缺失 actual/尺寸或解码错误 ⇒ `strict_fail`（白名单存在时整对豁免，沿用既有"白名单=容差具名化"语义），末尾固定行 `STRICT: FAIL/PASS diff=<总差异像素数>`；默认路径（分类账）只加注释、判定逻辑未改。

| 验收项 | 我的实跑 | 结果 |
|---|---|---|
| 默认（回归） | `python tools/diff_visual.py` | **exit 0**；`L4 视觉：1 通过 / 22 比对；分类记录 21；未分类 0`（与改前一致）；`git status oracle/screens` 空（report/diff 重生成字节一致） |
| 现基线 strict | `python tools/diff_visual.py --strict` | **exit 1**；`STRICT: FAIL diff=280132`（=22 对差异总像素，与 visual-report/我此前独立复算一致） |
| 全对相同临时样本 | 临时 screens（expected=actual 同字节 257 对）`main(['--strict'])`（module paths 重定向） | **exit 0**；`L4 视觉：257 通过 / 257 比对`；`STRICT: PASS diff=0` |
| 缺失 actual | 临时样本 expected 有图/actual 空 + strict | **exit 1**；`STRICT: FAIL diff=0`（外差异不计入 diff 像素——见 F4 提示） |
| 目录缺失 | EXPECTED 不存在 + strict | **exit 2** |
| 未知参数 | `python tools/diff_visual.py --bogus` | **exit 2**（stderr 明示仅支持 `--strict`） |
| 门禁默认路径 | `python gates/cli.py check` | visual-diff **PASS**（走默认模式）；`gates/cli.py` 最后改动为 `a11161d`（早于 64b3a44），**未改** ✓ |
| 判据落位 | `git show 53aceb8`（后续 docs 提交） | `l4-visual-plan.md` §0 新增"L4 0 容差验收判据 = `--strict` exit 0"✓（plan-diff-visual-strict 声明的待补一行已落） |

## 4. 测试与门禁（实跑）

| 项 | 命令 | 我的实测 |
|---|---|---|
| 单 crate | `cargo test -p shell-win` | lib **5** + smoke **19** = **24 passed / 0 failed**（`--list` 亦 24；与 §9.6 一致） |
| 全量 | `cargo test --workspace` | 全绿（exit 0；多个 test target 0 failed） |
| 总闸 | `python gates/cli.py check` | **CHECK: PASS；SUMMARY: PASS=11 FAIL=0 SKIP=1 TOLERATED=0**；docs-links PASS |
| 自检 | `--selftest-frames 300` ×2 | stdout 逐行一致、exit 0 |

## 5. 文档/纪律复核

**E1-E5/S2 逐条**（对照 shell-verifier 报告 §4/§5 与 plan §9.6）：
- E1 resize 口径：plan L52 已改"仅记录尺寸元组；softbuffer `Surface::resize` 在 `blit()`"；代码核对 `FrameRenderer::resize` 仅 `self.size=(w,h)`（lib.rs L661-663），`main.rs::blit` L65-73 调 `surface.resize` ✓。
- E2 授权留痕：plan §9.1 L190 已补 `16fb5b2` 并注明 Lead 授权；证据等级见 F3。
- E3 平台依赖范围：`crates/shell-win/Cargo.toml` 注释已改；ddb1558 的 Cargo.toml diff **仅注释**（无新增依赖）；`git grep` 平台依赖仍只在 shell-win/game-desktop（pre-existing），game-core/game-platform/game-shell/game-data 未被新增平台依赖 ✓。
- E4 Cargo.lock：plan §7 L149/§9.3 L223 已列根 `Cargo.lock` ✓。
- E5 rust-tests 口径：plan §9.2 L199 已注"338=`#[test]` 静态计数；运行态 337 passed+1 ignored" ✓。
- S2 音频措辞：platform-shell L127"v1=ApuWrite 写流消费+PCM 静音占位（`PcAudio`，无 cpal 设备流）" ✓（与 lib 注释/实现一致）。
- S1 资产行：platform-shell L148 注明旧"相对 cwd"已由 ddb1558 修正；§4.1 资产行含 exe 优先 + 前缀拒绝 ✓。

**并发误卷（task 要求核对"仅此一处且有留痕"）**：
- ddb1558 六个文件里，**仅 `docs/spec/platform-shell.md`** 含他人（task-9/platform-porter）内容：§3 模块表 input/host/render、tick 序列、新增 §3.1 可玩闭环 A、§8 未证域更新（**diff 实证**）；其余 5 文件（shell-win ×4 + plan-shellwin）均为 task-18 内容。**范围=一处文件** ✓。
- 留痕**三处可核**：①`docs/knowledge/gotcha-git-add-commit-atomicity.md` L47-55（Lead 记录：ddb1558 误含 task-9 platform-shell §3 模块表/tick）；②`docs/findings/plan-playable-loop.md` §10 L214-226（含 §3.1/§8 更全描述 + 纪律留痕）；③`b94ede8` commit message（"platform-shell §3 主体随 ddb1558 入库；本 commit 不含 ddb1558 内容"）。内容与 task-9 最终稿一致（b94ede8 仅"核入剩余补差"§4.2/§8/§9，未重提 §3 主体）✓。
- F2（info）：ddb1558 自身 commit message 只写"期间 HEAD 含并发 task-9 未提交改动；本 commit 只含 task-18 写范围内 6 个文件"，未点名误卷内容；归属靠后续留痕补齐。建议后续提交在 message 直接写"捕获了 X 的 Y 内容"（历史不 rewrite，符合 knowledge 纪律）。

**写范围（两 commit 文件清单）**：ddb1558 = shell-win Cargo/lib/main/smoke + plan-shellwin + platform-shell（均 task-18 卡写范围）；64b3a44 = tools/diff_visual.py + plan-diff-visual-strict.md；**两 commit 均未触碰** game-core/game-platform/game-shell/rom/reference/data/gates/cli.py/oracle ✓。

## 6. Findings 四分类 + 修正建议

### 6.1 数据/口径（低，非阻塞）
- **F1** `docs/findings/plan-shellwin-v1.md` **L198（§9.2）与 L229（§9.3）**仍写"lib 3 + smoke 17 = 20 passed / smoke.rs 17 项"——为 task-1 时点值；task-18 同时新增 2 lib+2 smoke ⇒ 现测 **24 = lib 5 + smoke 19**（§9.6 L280 已正确记录 24，L285 也注 §9.2 为 task-1 时点，但 L198/L229 无 inline 时点标）。建议改"**task-1 时点** lib 3+smoke 17=20；task-18 后 24=lib 5+smoke 19（§9.6）"，或在两行尾注"（历史时点）"。
  > 这与 shell-verifier E1 的关系：E1 要求的"smoke.rs 17 项 + crate 级 20=lib3+smoke17"**确已修**（原 plan:224 的"20 项契约测试"已拆清）；F1 是同 commit 新增测试后产生的**新时点差**，非 E1 未修。

### 6.2 证据不足（info）
- **F3** E2 授权链：plan L190 写"由 Lead 依 master-plan §5 授权执行（非实施方自行提交）"；`16fb5b2` 作者=同一 git 身份、commit body 无授权字段，仓库内可核证据仅"task-18 任务卡第 4 条 Lead 指令 + plan 自述"。按 B 级可接受（任务卡为授权记录），无需改；若要求 A 级可在交付汇报/轮次日志补一句授权记录。

### 6.3 表述/遗留（info，非阻塞）
- **F2** 见 §5（误卷留痕位置；历史不 rewrite，仅建议未来 commit message 点明捕获内容）。
- **F4** `tools/diff_visual.py` L7 docstring 仍写"本卡仅列候选不启用"（W-C1/W-C4 已于 2026-10-18 撤销）；生成器文案同步属另卡（task-20 约定，未偷改 ✓）。另 strict 固定行 `diff=<总差异像素数>` 不包含"缺失 actual/尺寸错"这类**非像素外差异**（实测缺 actual 时输出 `STRICT: FAIL diff=0`）——行为已在 plan §1 声明"外差异不只是像素差"，建议在 docstring/使用说明加半句"diff 仅像素差计数，外差异看 FAIL 本身"，避免误读。
- **F5（info）** plan §9.6 L282-283"全量/总闸：见交付汇报（并发 task-9 工作树口径）"——非自足证据指针；我已在当前 HEAD 复跑补齐（全量 exit 0、check PASS），建议后续把实测行直接写回 §9.6。

### 6.4 越权/伪造
- **无**。未发现伪造证据、超范围改动或"声称未实现能力"；B1/S1/task-21 均为真实修复，失败面（缺资产/无 cwd/差异存在）均 fail-closed/FAIL 而非放宽。

## 7. 命令与输出摘要

```text
# 1) B1 独立探针（自建 %TEMP% crate，shell-win path 依赖）
"C:foo" None / "D:foo\\x" None / "C:" None / "C:/x" None / "C:\\Windows\\x" None
"\\\\?\\C:\\x" None / "\\\\server\\share\\x" None / "..\\AGENTS.md" None
"rom/../../AGENTS.md" None / "rom\\..\\..\\AGENTS.md" None / "rom/gunnac_cn.nes" Some(262160)

# 2) S1
cargo test -p shell-win → lib 5 / smoke 19 = 24 passed
异 cwd 实跑（python subprocess, cwd=%TEMP%）→ exit 0; rom_len=Some(262160)
  stderr: shell-win: 资产根=D:\workspace\gunnac_remake
--selftest-frames 300 ×2 → stdout 逐行一致; host=0xf51d5a517e7458b7 canvas/display 不变; exit 0

# 3) task-21
python tools/diff_visual.py            → exit 0 ; 1 通过 / 22 比对 ; 分类 21 ; 未分类 0
python tools/diff_visual.py --strict   → exit 1 ; STRICT: FAIL diff=280132
临时全同样本 strict                    → exit 0 ; STRICT: PASS diff=0（257 对）
缺 actual strict / 默认                → exit 1（FAIL diff=0）/ exit 0
目录缺失 / 未知参数                     → exit 2 / exit 2
git status --porcelain oracle/screens  → 空
git log -1 -- gates/cli.py             → a11161d（早于 64b3a44，未改）

# 4) 测试/门禁
cargo test --workspace                 → exit 0
python gates/cli.py check              → CHECK: PASS ; SUMMARY PASS=11 FAIL=0 SKIP=1 TOLERATED=0
docs-links                             → PASS

# 5) 范围/留痕
git show --name-only ddb1558           → 6 files（shell-win×4 + plan-shellwin + platform-shell）
git show --name-only 64b3a44           → tools/diff_visual.py + docs/findings/plan-diff-visual-strict.md
knowledge gotcha L47-55 / plan-playable-loop L214-226 / b94ede8 message → ddb1558 误卷留痕三处
```

---

## 8. 复核边界

1. 复核对象=已入库 commit `ddb1558`/`64b3a44`（及判据落位 `53aceb8`）；HEAD `0cf42f0`。此后新改动不在本次范围。
2. 未实跑 release 构建的资产根行为（见 §2 边界）；未实跑 GUI 窗口视觉（无头环境；plan §9.4-9 已自述手动项）。
3. `--strict` 的"全对相同临时样本"用 module 路径重定向模拟（不写仓库、建 257 对同字节 PNG）；与 plan 的"临时 root 复制工具"等价。
4. 未审计 ddb1558 误卷的 task-9 content 本身质量（属 task-9/task-8 复核范围）；仅核"归属/留痕/与当前 spec 一致性"。

---

【2026-10-18】decomp-verifier，复核 HEAD `0cf42f0`；被审 `ddb1558` + `64b3a44`。
