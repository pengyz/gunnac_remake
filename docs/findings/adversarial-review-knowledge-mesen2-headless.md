# 对抗复核：知识库 gotcha（Mesen2 headless Windows IO）——adversarial-review-knowledge-mesen2-headless

> 日期：2026-10-18｜角色：decomp-verifier（复核方，证伪立场；**不复用 spec-writer 自述**）
> 被审：**`docs/knowledge/gotcha-mesen2-headless-windows-io.md`**（74 行，sha256 `f44bf0ab…`，`status: confirmed`，未提交；task-82 产出）。
> 复核时 HEAD=`5a675b8`（`feat(tools): scenario_run.py 支持 Mesen2 headless`，复核期间入库；复核开始时该文件为工作区 M 版，实测用其工作树版本，内容与入库 commit 一致）；默认 `analysis/`、`shots/` 已在 `.gitignore`，复核产物未污染仓库。
> 方法：①版本/gitignore 自核 ②**settings A/B 实测**（Debug 段 vs 顶层段；改前备份/改后恢复）③真实跑 `scenario_run.py run title-menu` 多次并逐字节比对 ④knowledge-format/docs-links/gates ⑤文档逐条对证据。本文只写该文件、未 git add/commit。

---

## 0. VERDICT：**pass**（0 阻塞；3 条低风险 finding + 2 info）

- **事实核验全部通过**：
  - `tools/mesen2/Mesen.exe` **FileVersion=2.1.1**、ProductVersion=`2.1.1+137ae7ce…`（与文档一致）；`tools/mesen2/`、`tools/mesen2.zip` 在 `.gitignore`（第 19/20 行）✓。
  - **A/B 实测确认配置位置**（详见 §2）：`Debug.ScriptWindow.AllowIoOsAccess=false` 且顶层=true ⇒ **采集失败**（90.2s 超时、`trace_lines=0`、无 DONE/lua.log）；`Debug=true` 且顶层=false ⇒ **采集成功**（1080 帧/1080 trace/37 shots、3.5s）；改前备份/改后恢复，settings sha256 回到原值 `5FEB2659…`。⇒ **`Debug` 段是必要条件；顶层 `ScriptWindowConfig.AllowIoOsAccess` 既不充分也不必要（对 headless 无效）**——文档核心结论成立。
  - **实测数字/确定性**：`title-menu` 跑 3 次成功（run1/run2 原配置、run4 顶层=false 配置）均为 `frames=1080 trace_lines=1080 captures=37 shots=37 wall=3.5s`，DONE json 一致；三跑 `frames.jsonl` sha256 **全同** `62c9ea09580654ce…`，**37/37 shots 逐字节相同**；`title-menu.lua.log` 含 started/done at frame 1080；`mesen.log` 仅 CPU uninitialized-read 提示（无显示错误）✓。
  - **命令语义**：工具代码构造 `[Mesen.exe, "--testRunner", <lua>, <rom>]`（顺序=先脚本后 ROM）；Windows 分支无 Xvfb/显示；脚本 `emu.stop` 后 testRunner 自行结束（成功 run 3.5s 内落 DONE）；`takeScreenshot` headless 可用（37 png）✓。
  - **格式/门禁**：`knowledge-format` **PASS**（24 条知识条目 frontmatter/type/文件名前缀合规）；`docs-links` **PASS**（347 md/438 links）；`gates check` **PASS=11 FAIL=0 SKIP=1**（rust-tests 382/基线 381；`ratchet:knowledge_entries` WARN=PENDING 23→24，见 info）✓。
- **低风险 finding（提交前建议顺手修，非阻塞）**：
  1. **F1（低，证据指针/措辞）**：`std::bad_cast` 断言（标题 L3 + L12-14/L22/L38）在条目内**无可复现命令/输出指针**（仅指向 task-80/81 与 P0-0 手册 §1.4/§16；我本地复核尝试 WSL2 Linux `Mesen --testRunner` 未取得输出/复现，不能作为反证）；建议补 WSL2 命令 + stderr/`exit 134` 或明确指到手册实测记录，并把标题/L12"任何 invocation 都崩"限定为"**本机 WSL2 Ubuntu 22.04 实测的 GUI/testRunner 调用**"。
  2. **F2（低，时点）**：L49-50 写 headless 支持为"2026-10-18 工作区改动（task-80 独立复核 + 提交后生效）…引用前验证 git log"；复核期间该支持已入库 **`5a675b8`**（commit message 引 task-80+84 复核）⇒ 措辞已过期，建议改为"已入库 `5a675b8`（task-80 复核 pass）"。另 L67"判据：把 Debug 段该键改回 false **应**复现"可改为"**已 A/B 实测**（见本条证据指针/复核报告）"。
  3. **F3（低，证据强度）**：L29"`AutoStartScriptOnLoad` 是 GUI 脚本窗口装载行为，`--testRunner` 路径不依赖它"——本机 settings 该键当前=True，成功跑不能区分"依赖/不依赖"；建议补源码/权威指针或改为"未观察到依赖（未做 A/B）"。
- **info**：①`docs/knowledge/README.md` §4 条目清单未收录本条（另有 3 条既有条目也未收录；gate 不校验清单，建议顺手补行）；②`ratchet:knowledge_entries` 23→24 WARN=PENDING——条目合法新增，需 Lead 用 baseline update 上调并在 commit 说明（`gates check` 仍 PASS）。

---

## 1. 版本 / gitignore（自核）

| 项 | 实测 | 文档 |
|---|---|---|
| `tools/mesen2/Mesen.exe` | FileVersion **2.1.1**；ProductVersion `2.1.1+137ae7ce3bf3f539d007e2c4ef3cb3b6c97672a1`；70470144 B | L63-64 一致 ✓ |
| Linux 版 `tools/mesen2/Mesen` | 存在（83,367,840 B，Mesen 2.1.1 Linux x64） | 文档提及 Linux 版路径 ✓ |
| `.gitignore` | L19 `tools/mesen2/`、L20 `tools/mesen2.zip`；L11 `analysis/`、L14 `shots/` | L64/L55 一致 ✓ |

## 2. settings A/B 实测（核心证伪点）

**改前备份** `%USERPROFILE%\Documents\Mesen2\settings.json` → `%TEMP%\mesen-settings-backup.json`，sha256=`5FEB2659263C963520FAC9AF08390A075BE510200DDB9291480CA3A2BABC4708`（Debug=true、顶层=true 两段并存）。

| 配置 | 命令/产物 | 结果 | 判定 |
|---|---|---|---|
| 原配置：Debug=true, 顶层=true | `scenario_run.py run title-menu`（run1/run2） | **成功**：1080/1080/37/37、DONE、3.5s；trace sha `62c9ea09…`；37/37 shots 相同 | 基线 ✓ |
| **A：Debug=false, 顶层=true** | 同上（run3） | **失败**：`timed_out=true, wall=90.2s, trace_lines=0, shots=0`；**无 DONE、无 lua.log**；mesen.log 仅 CPU 提示 | Debug 键必要；顶层 true **不足以**启用 IO ✓ |
| **B：Debug=true, 顶层=false** | 同上（run4） | **成功**：1080/1080/37/37、DONE、3.5s；trace sha **同** `62c9ea09…`；37/37 shots 相同 | 顶层键**非必要/对 headless 无效** ✓ |
| 恢复原配置 | 复制备份回 settings | sha 回到 `5FEB2659…`，Debug=true/顶层=true | 无残留 ✓ |

> 因果性：A/B 仅改 settings 两个键，其余工具/ROM/场景不变；A 失败、B/基线成功 ⇒ 文档"`Debug.ScriptWindow.AllowIoOsAccess=true` 才生效、顶层键无效"成立。
> 限制：Mesen2 源码不在本仓库，且本机对 `raw.githubusercontent.com`/`github.com` 的抓取被解析限制，**无法独立核对**任务提到的 `LuaApi.cpp:984`/`DebugConfig.cs:104` 源码行；按任务允许的 A/B 路径已独立证实。A 失败表现为脚本 IO 不可用后驱动等 DONE 超时，而非显式 Lua 错误（驱动不打印脚本 stderr）。

## 3. scenario_run 实测（工作树版本；复核期间入库 `5a675b8`）

| run | 配置 | frames | trace_lines | shots | wall | trace sha256 | 结果 |
|---|---|---|---|---|---|---|---|
| run1 | Debug=true/顶层=true | 1080 | 1080 | 37 | 3.5s | `62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331` | DONE ✓ |
| run2 | 同 run1 | 1080 | 1080 | 37 | 3.5s | **同 run1** | DONE ✓ |
| run3 | Debug=false/顶层=true | — | 0 | 0 | 90.2s | 无 trace | timed_out（A） |
| run4 | Debug=true/顶层=false | 1080 | 1080 | 37 | 3.5s | **同 run1** | DONE ✓（B） |

- run1 vs run2：trace sha 相同、37/37 shots 逐字节相同 ✓（文档 L53-54 数字成立；wall "~4s" 对应 3.5s）。
- DONE=`{"frames":1080,"trace_lines":1080,"captures":37,"shots":37,"bot_stop":false}` ✓（L57）；lua.log=`scenario title-menu started…`+`scenario done at frame 1080` ✓（L56）；mesen.log 14 行仅 CPU uninitialized-read（L58"无显示相关错误"成立）。
- 归档旁证：`analysis/traces/.archive/title-menu-*` 共 **23** 个快照目录、其中 **19** 个含 `frames.jsonl`，19/19 sha 均 `62c9ea09…`（≥ 文档"跨 11 跑全同"，支持"天然确定性"；文档声明的"1 个 37-shot 不同的实验跑未定性未采信"为透明保留，我未采信/未复算该异常快照）。
- 产物路径：`analysis/review-mesen2/run{1,2,4}/...`（本次复核用 `GUNNAC_TRACES`/`GUNNAC_SHOTS` 覆盖到仓库 `analysis/` 下、gitignored，不踩默认 `analysis/traces` 的 P0-0 产物）。

## 4. 文档逐条核对（与实测/代码对照）

| 文档行 | 内容 | 判定 |
|---|---|---|
| L3/L12-14/L22 | Linux 版 WSL2 bad_cast/exit 134、不可用路径 | 措辞已限定"本机 WSL2"（F1：缺可复现指针；我未能复现/反证） |
| L17-18/L27-28/L65-67 | 顶层 key 不生效、Debug 段才生效 | **A/B 实测成立**（§2）；L67"应复现"可升级为"已实测" |
| L24-26 | `--testRunner <lua> <rom>` 顺序、无 Xvfb、`emu.stop` 退出 | 代码 L507 + 4 次成功跑证实顺序/无 Xvfb/DONE 退出 ✓ |
| L29 | `AutoStartScriptOnLoad` 不依赖 | F3：当前 settings=True，成功不能区分；需指针/软化 |
| L44-48 | 驱动内容锚（MESEN/HEADLESS/testRunner/GUNNAC_MESEN） | 与 `tools/scenario_run.py` L59-69/L505-509 一致 ✓ |
| L49-50 | headless 支持"工作区改动/提交后生效"，引用前查 git log | 复核期间已入库 `5a675b8`（F2：更新措辞/commit 指针） |
| L51-58 | 实测 1080/1080/37/37、~4s、两跑 sha/37 shots 相同、日志 | 全部复现 ✓ |
| L59-61 | 归档跨 11 跑 sha 全同、异常快照不采信 | 19/19 同 sha（≥11）✓；异常快照保留声明 ✓ |
| L62 | `takeScreenshot` headless 可用 | 37 png 产出 ✓ |
| L63-64 | 版本 2.1.1 / gitignore | 自核一致 ✓ |
| L65-67 | settings 位置/两段并存/判据 | 自核 settings 两段并存、值均 True；A/B 成立 ✓ |
| L68-70 | methodology L94/L96/L165 旧口径旁证 | 三处存在且语义相符（L96 GUI 位置参数 + AllowIoOsAccess；L165 配置放 IO） ✓ |
| L71-74 | task-80/81/82 任务链、权威落档 manual §1 | 存在/一致（manual §1.4 与 §16.2 smoke 记录与实测同） ✓ |

## 5. 格式与门禁

- 文件名 `gotcha-mesen2-headless-windows-io.md`（`gotcha-` 前缀）✓；frontmatter `type: gotcha`/title/date/status ✓；正文四节 `现象`/`为什么`/`何时使用`/`证据指针` 齐全 ✓。
- `knowledge-format` gate **PASS**（24 条合规）；`docs-links` **PASS**（347 md/438 links）；`gates check` **PASS=11 FAIL=0 SKIP=1**。
- `ratchet:knowledge_entries` **WARN=PENDING**（基线 23/现值 24）——合法新增，需 Lead baseline update（commit 说明），不阻塞本复核。

## 6. Findings 四分类

### 6.1 证据不足/措辞（低，提交前建议修，非阻塞）
- **F1**：`std::bad_cast` 无条目内可复现指针（命令行+输出/exit 134），标题/`任何 invocation` 过于绝对；建议补指针并限定"本机 WSL2 Ubuntu 22.04 实测的 GUI/testRunner"。本机 WSL2（Ubuntu-22.04, Stopped/Version 2）存在，但我对 Linux Mesen 的 `--testRunner`/`--version` 探测未取得输出（挂起/包装退出 2），**不能作为反证**；文档所指 bad_cast 需保留原始命令输出才能成为可复现知识。
- **F2**：L49-50 的"工作区改动/提交后生效"在复核期间已变为事实（`5a675b8`）；L67"应复现"可改"已 A/B 实测"。
- **F3**：`AutoStartScriptOnLoad` 不依赖的断言缺源码/A-B 证据（当前 settings=True）。

### 6.2 规格/格式（info）
- **F4**：README §4 条目清单未收录本条（另有 3 条既有条目未收录，非本卡引入）；建议补一行（gate 不校验，属可读性维护）。
- **F5**：knowledge_entries ratchet WARN PENDING 23→24，Lead 处理基线。

### 6.3 实现/行为
- 无。文档只描述环境/配置事实，未涉及代码改动；复核未发现新行为声称。

### 6.4 越权/伪造
- **无**。文档仍为未提交单文件；未改工具/门禁/基线；A/B 仅动用户级 settings 且已恢复原 sha；本次复核产物均在 gitignored `analysis/` 下。

---

## 7. 命令与输出摘要

```text
# 版本/gitignore
(Get-Item tools/mesen2/Mesen.exe).VersionInfo → FileVersion=2.1.1 ProductVersion=2.1.1+137ae7ce3bf3f539…
.gitignore L19 tools/mesen2/ ; L20 tools/mesen2.zip ; L11 analysis/ ; L14 shots/

# settings 备份/恢复
backup sha256 = 5FEB2659263C963520FAC9AF08390A075BE510200DDB9291480CA3A2BABC4708
Debug.ScriptWindow.AllowIoOsAccess=True ; ScriptWindowConfig.AllowIoOsAccess=True

# A/B + 确定性（GUNNAC_TRACES/GUNNAC_SHOTS 覆盖到 analysis/review-mesen2/*，gitignored）
run1/run2 (True/True):  frames=1080 trace_lines=1080 shots=37 wall=3.5s DONE ✓
  trace sha256=62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331（两跑相同；37/37 shots 逐字节相同）
run3 (Debug=False, 顶层=True): timed_out=true wall=90.2s trace_lines=0 shots=0 DONE 无（A：IO 不可用）
run4 (Debug=True, 顶层=False): 1080/1080/37/37 DONE wall=3.5s trace sha 同 run1（B：顶层键无效）
settings 恢复 → sha256=5FEB2659… ; Debug=True/顶层=True

# 门禁
python gates/cli.py check → CHECK: PASS；PASS=11 FAIL=0 SKIP=1
  knowledge-format PASS（24 条）; docs-links PASS（347 md/438 links）
  ratchet:knowledge_entries WARN PENDING（基线 23 / 现值 24）

# 归档旁证
analysis/traces/.archive/title-menu-*: 23 dirs（19 含 frames.jsonl，sha 19/19=62c9ea09…）
```

## 8. 复核边界

1. 被审=knowledge 单文件（74 行，sha256 `f44bf0ab…`）；复核时 HEAD `5a675b8`（scenario_run headless 支持在复核中入库；本次实测使用工作树版本，与入库内容一致）。
2. Mesen2 **源码不在仓库**、在线源码抓取受本机网络解析限制 ⇒ 未独立核 `LuaApi.cpp:984`/`DebugConfig.cs:104`；按任务允许的 A/B 实验独立证实配置结论。
3. WSL2 Linux 版 bad_cast：本机探测未取得输出/未能复现（挂起/包装退出），**不据此否定文档**；建议作者补可复现命令输出（F1）。
4. 未跑 GUI 模式；未对 `emu.stop` 后 testRunner 自行退出的进程退出码做单独测量（成功 run+DONE 与驱动逻辑一致）。
5. A/B 修改的是用户级 `%USERPROFILE%\Documents\Mesen2\settings.json`，已备份并恢复原 sha；复核期间有其他 agent 并发跑 Mesen（串行锁生效，run4 首次冲突已重试），本次成功 run 为独占期间产物。

---

【2026-10-18】decomp-verifier，复核 knowledge 条目（HEAD `5a675b8`）；settings A/B + title-menu ×3 + 双跑确定性。
