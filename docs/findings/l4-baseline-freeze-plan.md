# L4 P0-0 基线冻结执行手册（task-16）——可复现 22 对基线的采集/渲染/资产裁定/冻结

> 日期：2026-10-18｜角色：decomp-analyst（取证/方案）｜状态：**方案就绪；环境已就绪（2026-10-18 Windows headless 已打通，见 §1/§16）⇒ 可启动 P0-0-0→P0-0-1**
> 上游方案：[l4-zero-tolerance-attack-plan.md](l4-zero-tolerance-attack-plan.md)（commit `0cf42f0`，§5.0 六步清单）
> 复审报告：[adversarial-review-l4-attack-plan.md](adversarial-review-l4-attack-plan.md)（task-24 复审 pass；§7 R1-R8）
> 环境事实核验时点：HEAD `167d8a9`（**分析时点**；执行前重核，尤其 `--out-dir` 是否已实现，S6）。
> **修订记录（2026-10-18 第二轮，task-30 复核 needs_revision → task-32 修）**：按
> [adversarial-review-l4-baseline-freeze-plan.md](adversarial-review-l4-baseline-freeze-plan.md) 修
> **F1-F8（阻塞/重要）+ F9-F17/S1-S6（随文）**；逐条落点见文末 **§13 修正映射**。
> **后续点改（同日）**：task-46 R1-R5（§7.7 Step0/mapping 回写等）→ task-48 组 D 来源更正（§3.6）→
> task-50 四联帧预算更正（§3.7）→ task-54 F49-1..6（overlay 名单/stg06 probe=0/组 D 判据可执行化/C5 骨架）。
> **环境更正（2026-10-18，task-81）**：主路径改为 **Windows 宿主 + Mesen2 2.1.1 Win x64 headless**
> （`Mesen.exe --testRunner`；Lua IO 需 `Debug.ScriptWindow.AllowIoOsAccess=true`）；WSL2/Linux 版 Mesen2
> `std::bad_cast` 不可用仅备选。§1/§2 命令/§8.2 验收同步；实测记录见 §1.2/§16。
> **【F49-6 版本口径】** 本稿经 `task-30/32/34/46/48/50/54` 连续点改；**行数/sha 以提交时文件实体为准**
> （不硬编码中间版本号，避免自指不一致；核对命令：`wc -l docs/findings/l4-baseline-freeze-plan.md` +
> `git hash-object docs/findings/l4-baseline-freeze-plan.md`）。
> 纪律：AGENTS.md（A/B/C 分级；未证不发明；数字可复算；结论带证据指针）。**本文件只做 P0-0 方案手册**
> （只写本文件；不改 `crates/`、`oracle/`、`tools/`、`rom/`、`reference/`、`data/`；不 git add/commit）。
> **本手册不主张任何 L4 达成**：P0-0 的产物是"可复现基线 + 逐对 provenance + 资产裁定记录"；
> 冻结后的 `--strict` 基线**预期仍是 FAIL（diff>0）**——这是后续 P0-1.. 的起点，不是失败。

## 0. 结论与执行口径

1. **P0-0 的唯一目标**：把现行 22 对 L4 基线从"不可复现的战役产物"变成**可复现、逐对可溯源、资产有效**的
   基线——产物 = `oracle/screens/l4-manifest.json`（22 行）+ 两跑 sha 全同 + 新 `actual/` + 资产裁定记录
   + 新 `visual-report`（含 `--strict` 记录）。
2. **执行环境 = Linux + Mesen2 2.1.1**（用户已裁定提供）；**当前 Windows 工作区（无 Mesen2/`analysis/`）
   只能做只读预研**（§9）：manifest 模板、映射推断、overlay DSL 草案、`--out-dir` 代码 diff 规格、wrapper
   源码——**零验收数字**。
3. **三条关键风险（先读）**：
   - **R-1 场景来源**：组 A/B/C 的**期望图来源 capture 已在库内场景 DSL 中**（§3.0：gameplay-stage1/
     stg03-timeline/stg08-timeline/title-menu 的 `capture <帧> <tag>` 与 22 对帧号精确对应）——这是本轮新核
     出的强简化；组 D（fade+ending 6 对）~~候选=`tas-clear-full-v5` overlay 重放~~【**2026-10-18 更正·
     L5 交叉核对**：`tas-clear-full.l5.jsonl` 证明 v5 的 f28364=stg01-clear、f29478=stg02-combat、
     f51332=stg02-clear ⇒ 28500-36000 落在**关 2 战斗段**，组 D 来源≠v5；**重裁流程/候选清单见 §3.6**，
     未重裁前沿用旧假设会取错帧】。
   - **R-2 probe 事件上限**：`tools/scenario_capture.lua` 实现上限 **3,000,001 事件**（`probeN>3,000,000`
     后静默丢弃；**不是"3,000,000 含头行"**，F10）；监控 `wc -l` 留安全裕度（§2.4）。
   - **R-3 规模化资源**：`phase-batch` 产物历史曾达 **1.27GB**（stg03-text）；`render_frame` 会把 text 写集
     全量读入内存 → 建议 **磁盘 ≥20GB 空闲、RAM ≥16GB**（C 级估算，实测后回填，见 §1.5）。
4. **一切映射先用三重校验**：①新 probe 文件**事件数 = wc-l − 该场景头行数** == 旧 `.render.log` 的 `probe=` 值
   （头行数逐场景不同，见 §2.4 表）；②同场景两跑 trace/probe/text sha256 全同；③**capture PNG sha256 == 旧
   expected blob（主判据）**。①不过 = 输入/注册可疑；②不过 = 输入不可复现；③不过 = 映射不成立。未过校验的
   映射一律标 `blocked`，不得当"已复现"。

### 0.5 执行顺序（步骤编号不变，顺序按依赖；B1 唯一安装点=§7a）

```
§1 环境 → §2 采集（I1-I6；含 capture 源图产出） → §3 映射三重校验（含 §3.0 capture 现况/§3.7 来源更正）
→ §4 输出参数化（或 REPO_ROOT 沙盒先行）
→ §7a 资产重采/裁定 + **expected 唯一安装**（stg02/四联/ending-4/5/fade 的新图落到
      `oracle/screens/expected/` 并归档旧件；不晚于此全部完成——§7.7 安装流程）
→ §5 wrapper 首次 canonical 渲染 + manifest（expected 必须已定稿安装）
→ §7b ending actual 重渲校验/dup **登记核对**（不再改动 expected；若需改动须回到 §5 之前）
→ §6 两跑确定性 → §8 核验+冻结（**只核验安装结果/diff 记录/方案数字 dated 更正，不再复制 expected**）
```
> **B1 唯一安装点**：expected 新件**只在 §7a（§7.7）安装**；§8.1 步骤 3 只做"核验 manifest
> `expected_sha256` == 实际文件 sha"，不得再复制/归档（否则会把新件当旧件归档或让 manifest 过期）。
> ending-4/5 的 expected 替换也必须在 §5 之前完成（其裁定证据来自 §2/I5 的 capture blob 比较，不依赖
> actual 渲染）；§7b 只做 actual 侧重渲与 dup 登记核对。
> **【2026-10-18 更正·组 D】** §7a 内新增子步骤 **§3.6 组 D 重裁**（候选来源/三判据），仍必须在 §5 前完成；
> 重裁未决/无来源时 ending+fade 6 对**不 staging、manifest 标 blocked:no-source**（P0-0 不得宣称全绿）。
> **【2026-10-18 更正·四联来源】** §7a 内四联选帧改为 **§3.7**（stg08-* ← `tas-stg08b2-direct`；
> stg06-late ← `stg06-timeline`），原 v5 36500f 方案作废；环境清单同步删 v5 36500f、加 I5/I6（§2.1）。
> **【F55-1 执行顺序】** §3.3 ① 的 **POST 段（`l4-stg06-pass2`）依赖 §7a（§7.2/§3.7）生成的 pass2
> probe 文件**——§3 阶段先跑 ① 的 PRE 段，POST 段到 §7a 后补跑（脚本对缺失打印 `SKIP` 不崩）；或把整段 ①
> 移到 §7a 后统一跑一次（两种等价，取其一）。
> **【2026-10-18 环境就绪】** Windows 宿主 + Mesen2 2.1.1 Win x64 headless 已打通（§1；冒烟 title-menu
> 1080/1080/37 + 两跑 trace sha 同 + 37/37 shots 逐字节同）⇒ **P0-0-0 通过，可启动 P0-0-1（§2 采集）**；
> WSL2/Linux 版 Mesen2 `std::bad_cast` 仅备选。
> §5 的 manifest 需要 §7a 的最终 expected sha，故实际执行顺序 §7a 先于 §5（步骤编号保留原方案 §5.0 的
> 0-6 映射，不重排）。

---

## 1. 步骤 0：环境接收（P0-0-0）

> **【2026-10-18 环境更正·Windows headless 已打通】** 主路径 = **Windows 宿主 + Mesen2 2.1.1 Win x64
> headless**（`tools/mesen2/Mesen.exe --testRunner`）；WSL2 内 Linux 版 Mesen2 实测 `std::bad_cast`
> （exit 134）**不可用**，仅保留为备选（§1.4 注）。**环境已就绪，可启动 P0-0-0 → P0-0-1（§2 采集）**。
> 本节 E1-E11 为 Windows 主路径判据；原 Linux/Xvfb 判据留档于 §1.4/§16。

### 1.1 环境要件与校验（逐项打勾）

| # | 要件 | 要求（Windows 主路径） | 校验命令（PowerShell/`python`） | 期望 |
|---|---|---|---|---|
| E1 | OS | **Windows 10/11 x64**（本机实测 Windows；WSL2 仅作备选外壳，Mesen2 用 Windows 版） | `[Environment]::OSVersion.VersionString` | 记录 OS Build |
| E2 | Mesen2 | **2.1.1 Win x64**：`tools/mesen2/Mesen.exe`（gitignored 可再生件；sha 记录见 §1.3） | `Test-Path tools/mesen2/Mesen.exe; (Get-FileHash tools/mesen2/Mesen.exe -Algorithm SHA256).Hash` | `True` + sha 记录 |
| E3 | **Lua IO 配置（前置，必做）** | `%USERPROFILE%\Documents\Mesen2\settings.json` 中 **`Debug.ScriptWindow.AllowIoOsAccess = true`**（有效键；同级 `ScriptWindowConfig.AllowIoOsAccess` 实测不生效——机理/判据见 [gotcha-mesen2-headless-windows-io](../knowledge/gotcha-mesen2-headless-windows-io.md)）；改后**重启 Mesen**；否则脚本 `io.open/os` 被拒 → 无 trace/shots | `python -c "import json,os,pathlib;d=json.loads((pathlib.Path(os.path.expanduser('~'))/'Documents'/'Mesen2'/'settings.json').read_text(encoding='utf-8-sig'));print(d['Debug']['ScriptWindow']['AllowIoOsAccess'])"` | `True` |
| E3b | 显示 | **无需** Xvfb/显示器（headless `--testRunner`；`scenario_run.py` 在 Windows 自动选 `.exe` 且 headless） | `python tools/scenario_run.py list` | 打印场景清单（无 GUI 弹窗） |
| E4 | Python | **`python` ≥3.10（本机 3.12.1）**（`Path.write_text(newline=)` 需 ≥3.10；F7；本机 `python3` 为 Store 别名不可用） | `python -V` | ≥3.10（记录版本） |
| E5 | Rust | 能构建 workspace（render-frame / phase-batch） | `cargo build -q -p game-desktop -p game-oracle` | exit 0（首次编译数分钟） |
| E6 | 仓库 | HEAD ≥`0cf42f0`；工作树干净 | `git log -1 --format=%h; git status --short` | 记录 HEAD；无与本任务冲突改动 |
| E7 | ROM | `rom/gunnac_cn.nes` 与基线 sha 一致 | `python gates/cli.py rom` | `[PASS]`（4 锚点 sha） |
| E8 | 磁盘 | ≥20GB 空闲（probe/trace/phase + 两跑副本；C 级估算） | `Get-PSDrive D`（POSIX：`df -h .`） | 可用 ≥20G |
| E9 | 内存 | **≥16GB（Windows 标称；实测 15.9 GiB ≈ 17.1 GB 十进制）**（render_frame 读大 text 写集；C 级估算） | `Get-CimInstance Win32_ComputerSystem`（`TotalPhysicalMemory/1GB`） | 记录 **GiB/GB 双口径**；十进制 ≥16GB 达标 |
| E10 | 串行纪律 | 同一时刻只跑一个 Mesen（`analysis/.scenario.lock` 串行锁；Windows=msvcrt 1-byte / POSIX=flock） | — | 不并行、不 `run-all`（见 §2.2 注） |
| E11 | 系统依赖（S3 改写；2026-10-18 task-87 F-1 更正） | **Windows 主路径无 X11/Xvfb 依赖**（Mesen2 Win x64 自带 runtime）。**但构建 `game-desktop`/`game-oracle` 时会编译 `game-desktop` 的 winit/softbuffer 依赖**（`crates/game-desktop/Cargo.toml` 直接依赖 `winit="0.30"`/`softbuffer="0.4"`，**包级依赖与 bin 选择无关** ⇒ `cargo build -p game-desktop` 必编译）；Windows 后端无需 X11，**POSIX 备选构建才需 X11 库**；P0-0 不需要构建 `shell-win` 等 workspace 其它包；若要真正"不构建 winit/softbuffer"需把 `render-frame` 拆独立 crate（后续卡，不在 P0-0） | `cargo build -p game-desktop -p game-oracle`；`cargo tree -p game-desktop -i winit`（应显示 `winit → game-desktop` 直接依赖） | exit 0；winit/softbuffer 已编译、无缺失 DLL |

### 1.2 冒烟测试（环境"可执行"判据；2026-10-18 实测通过）

```powershell
cd <repo-root>
python gates/cli.py status         # 只读预检：SKIP/TOLERATED 显式打印（AGENTS §5）
python gates/cli.py rom            # [PASS] 4 锚点 sha
python tools/scenario_run.py list  # 场景可解析（无 GUI 弹窗）
python tools/scenario_run.py run title-menu --timeout 600
```

**本次实测输出**（2026-10-18 Windows；`analysis/traces/title-menu.run.json`）：
```
== scenario run: title-menu ==
  DSL: oracle/scenarios/title-menu.txt sha256=258067a319351ef9…
  frames: 1080 trace_lines: 1080 captures: 37 shots: 37 wall: 3.5s
  DONE: {"frames": 1080, "trace_lines": 1080, "captures": 37, "shots": 37, "bot_stop": false}
  trace: analysis\traces\title-menu.frames.jsonl
  run清单: analysis/traces/title-menu.run.json
```
**两跑确定性（实跑）**：trace `sha256=62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331` 两跑相同；
`shots/title-menu_f*.png` **37/37 逐字节相同**；`gates status` = **PASS=11 FAIL=0 SKIP=1（addr-bridge）/TOLERATED=0，
STATUS: OK**；`gates rom` = `[PASS] rom-integrity`（4 锚点 sha，rom=`4d76901c1b2455f2…`）。
**判据**：`run.json` 中 `timed_out=false`、`done!=null`、`frames==trace_lines==1080`、`done.shots==37`；
两跑 trace sha 相同 + shots 逐字节相同。
**失败回退**：查 `analysis/traces/title-menu.mesen.log`/`title-menu.lua.log`；确认 **E3** 配置生效并重启
Mesen 后重试 1 次；仍失败 ⇒ BLOCKED 并上报 run.json/日志。E3 未配置的典型症状：Lua `io.open` 被拒 /
无 trace 产物（headless 仍能启动，但写不出采集件）。

### 1.3 版本/来源记录（写进 manifest 头）

```powershell
"OS: $([Environment]::OSVersion.VersionString)"            >  analysis/p0-0-env.txt
"python: $(python -V 2>&1)"                               >> analysis/p0-0-env.txt
"cargo: $(cargo -V)"                                      >> analysis/p0-0-env.txt
"Mesen.exe sha256: $((Get-FileHash tools/mesen2/Mesen.exe -Algorithm SHA256).Hash)" >> analysis/p0-0-env.txt
git log -1 --format='%H %ad %s' --date=iso                >> analysis/p0-0-env.txt
```
**本次实测**：Python **3.12.1**；cargo **1.96.0-nightly (cbb9bb8bd 2026-03-13)**；
`tools/mesen2/Mesen.exe` sha256=`d2eb03c2590c648bf329f127ebcfefd70130e7690a9e2ccdba8616faea1fe96b`
（70,470,144 B）。Mesen2 Win x64 来源/获取方式由用户环境提供者登记（`tools/mesen2/` 为 gitignore 可再生件）。

> **`analysis/p0-0-env.txt` 说明（task-85 F-3）**：该文件是 **P0-0-0 执行时生成**的 gitignored 工作区记录，
> **内容清单** = OS / CPU / RAM / 磁盘 / `Mesen.exe` sha256 / Python / cargo / `gates rom` 输出 / 当前 HEAD；
> **当前尚未生成属执行时产物**（执行前不存在 ≠ 环境未接收）。上列"本次实测"值来自同机同命令，
> 只作手册依据、不替代执行留档。

### 1.4 驱动/显示方案（Windows headless 主路径；POSIX 备选）

- **Windows（主路径）**：`scenario_run.py` 检测到 `tools/mesen2/Mesen.exe` ⇒ `HEADLESS=1`，以
  `Mesen.exe --testRunner <analysis/_gen/<name>.lua> rom/gunnac_cn.nes` 运行——**不需要显示/Xvfb**（脚本
  `emu.stop` 后 testRunner 自行退出；无 GUI 窗口）；串行锁用 `msvcrt` 1-byte，无 Xvfb/孤儿 `pgrep` 分支。
- 环境覆盖：`GUNNAC_MESEN=<path>` 指定可执行；`GUNNAC_HEADLESS=0/1` 强制 GUI/headless。
- **POSIX（备选，非主路径）**：原自管 Xvfb 路径保留（`Xvfb :N -screen 0 1280x1024x24 -nolisten tcp` +
  `DISPLAY` 注入 + flock + `pgrep -f tools/mesen2/Mesen` 孤儿清扫；GUI 模式传 `rom <lua> /DoNotSaveSettings`）。
  **WSL2 内 Linux 版 Mesen2 2.1.1 实测不可用**（任何 invocation `std::bad_cast`，exit 134）⇒ 不采用；
  若要 Linux 路径需另换可用 Mesen2 构建（记录来源/版本）；**POSIX 构建 `game-desktop` 另需系统 X11 开发库**
  （winit/softbuffer 的 X11 后端；Windows 主路径不需要，见 E11）。
- 操作者**无需**手工起显示/驱动：只用 `python tools/scenario_run.py run <name>`（Windows Python 3.12）。
- **命令解释器**：本文命令统一写 `python`（Windows 主路径）；若走 POSIX 备选，把 `python` 换成 `python3`。

### 1.5 资源预算（C 级估算，执行后以 `run.json.wall_s` 回填）

| 项 | 估算 | 依据 |
|---|---|---|
| 单场景耗时 | 带 probe/capture 时约 **30-60 fps** → `帧数/40` 秒量级；如 stg03-timeline(24000f) ≈ 10-20min、stg08-timeline(34000f) ≈ 15-30min、`tas-stg08b2-direct`(40000f) ≈ 15min、`stg06-timeline`(30000f) ≈ 10-20min（2026-10-18 更正：原 v5-overlay 36500f 计划作废，见 §3.7） | 历史 `gs5-stg01-full` 15000f/260.9s≈57.5fps（轻量面）；带回调更慢（B/C） |
| 磁盘 | probe 3M 事件上限 × ~80B ≈ 240MB/场景；frames.jsonl ~0.6KB/帧（zp 512 hex）；phase text 可达 GB 级 | 历史 stg03-text 1.27GB（方案 §3.1 引用）；**执行时用 `du -sh analysis` 实测回填** |
| 内存 | render-frame 全量读 text 写集（Vec）→ 大场景 GB 级 | `render_frame.rs` 读入+排序实现（A 级代码事实） |
| **实测（title-menu，Windows headless）** | **1080 帧 / 1080 trace / 37 shots / wall 3.5s**（轻量无 probe 场景） | 2026-10-18 `analysis/traces/title-menu.run.json` |

---

## 2. 步骤 1：输入恢复（P0-0-1）

### 2.1 采集清单（只跑需要的场景；**不要 `run-all`**）

> 依据：22 对旧 `.render.log` 的 probe 签名分组（§3.1）+ 现行场景帧数/probe 配置（§11 工具事实）。
> "重启后旧产物自动归档"：`scenario_run.py` 会把同名旧产物移到 `analysis/traces/.archive/<name>-<stamp>/`（run 隔离）。
> **长场景超时（2026-10-18 task-91 / P0-0 采集新根因）**：I3/I4/I5/I5b/I6/I6b 预计 >100s——Mesen testRunner
> 自带 `TestRunnerTimeout` 默认 **100s**；`scenario_run.py` headless 会**自动**给 Mesen 传 `--timeout`
> （见 §2.2 超时注）。**手工**调用 `Mesen.exe --testRunner` 时必须自带 `--timeout=<≥场景预算秒>`，
> 且采集后必须核 `done!=null && trace_lines==frames`（防 ~100s 截断产物混入基线）。

| 序 | 场景/overlay | 为什么需要 | 帧数 | probe_mem（注册数=头行数） | 预计耗时 | 产物（analysis/traces/ 下） |
|---|---|---|---|---|---|---|
| I1 | `title-menu` | #22 title f180 冒烟/映射（源已有 `capture 180 boot-black`） | 1080 | **0**（无 `.probe.jsonl` 产出） | ~1min | `title-menu.{frames,captures,run}.jsonl/json` + `shots/` |
| I2 | `gameplay-stage1` | #1-3（probe sig 1,307,322；源已有 capture 1260/1500/2349） | 2400 | **15**（8 标准 + 7 额外：`0200 02ff`,`0014 0015`,`0020 0021`,`8000 8001`,`001a 001a` + `002f 0030`/`2006 2007` 各重复一次）——**源原样跑，不 overlay**（F1/F4） | ~2-4min | 同 I1 |
| I3 | `stg03-timeline` | #4/#5/#12/#13/#14/#15（sig 598,571；源已有 capture 3404/4600/5700/6500/12000 + `capture_every 300`） | 24000 | **8**（源原样） | ~10-20min | 同 I1 |
| I4 | `stg08-timeline` | #16-#21（sig 1,167,314；源已有 capture 14600/16000/16600/18000/22000/28000） | 34000 | **8**（源原样） | ~15-30min | 同 I1 |
| I5 | `tas-stg08b2-direct`（**stage8 直入轨**） | **四联 stg08-start/combat/late 真值候选 + §3.6 组 D 重裁候选 C1**：`poke 1265 0180 08` 选关 stage8、frames 40000、源已含 **10** 条 probe（8 标准 + `048c` + `4000-4017`）、capture_every 1500、assist 三 poke | 40000 | **10**（源原样） | ~15min | `tas-stg08b2-direct.*` |
| I5b | `l4-stg08b2-pass2`（§7.2 两遍 capture；**仅加 capture**） | 按 §7.2 选帧（stage_no==8）加 `capture <f> l4f<f>` → 新 expected 源图 | 40000 | **10**（源原样 + 选定帧 capture） | ~15min | `l4-stg08b2-pass2.*` |
| I6 | `stg06-timeline`（**STG-06 自关窗**） | **四联 stg06-late 真值候选**：源已有 STG-06 关内数据（f13000 stg06-combat、`capture_every 300`、冻结前窗 f13000-13977）；**【F49-2 更正】源 `probe_mem` 实测 = 0** | 30000 | **0**（不产 `.probe.jsonl`；pass2 补） | ~10-20min | `stg06-timeline.*` |
| I6b | `l4-stg06-pass2`（§7.2 两遍 capture；**补 8 probe + 加 capture**） | 按 §7.2 选帧（stage_no==6）加 `capture <f> l4f<f>`；§3.2 make 对源 0 probe **补 8 标准区间** | 30000 | **8**（源 0 + 补 8 + 选定帧 capture） | ~10-20min | `l4-stg06-pass2.*` |
| ~~I5-old~~ | ~~`l4-baseline-v5`/`l4-baseline-v5-pass2`（36500f 截断）~~ **降级为负候选对照** | ~~四联/组 D 真内容帧~~【**2026-10-18 更正·L5 交叉核对**：v5 到 f73939 才 stage4-combat ⇒ 36500f 仍在关 2，stage6/8 选帧必然失败；v5 仅保留作"blob 必然不等"对照，不进入四联/组 D staging（见 §3.6/§3.7）】 | ~~36500~~ | ~~8~~ | — | 对照用（可跳过） |

> **capture 现况（本轮新核，§3.0）**：组 A/B/C/E 的目标帧 capture **全部已在源场景 DSL 内**——
> 其中 **#13 shop-window(5400) 由 stg03 源 `capture_every 300 probe` 覆盖**（5400=18×300，产物
> `shots/stg03-timeline_f05400_probe.png`；M2：不再为它造 I3b overlay，省一次 24000 帧重跑）。
> **四联真内容帧来源**：stg08-* → I5/I5b（`tas-stg08b2-direct` 直入轨）；stg06-late → I6/I6b
> （`stg06-timeline` 自关窗）——【**2026-10-18 更正·L5 交叉核对**】原"由 v5 `capture_every 1500` 覆盖"
> 不成立（v5 帧预算见 §3.7 反证）。overlay 生成器对 `probe_mem` **按区间去重**（F1）：已存在区间不重复追加。
> **【2026-10-18 更正·组 D】** 组 D（fade/ending 6 对）**不再假定 v5 为来源**（语义反证见 §3.6）；
> 组 D 的 expected 重采改为 §3.6 重裁选中来源（候选含 `tas-stg08b2-direct` 等）后再 staging。

**标准 8 区间**（与 `gameplay-stage1`/`stg03-timeline`/`stg08-timeline` 现有配置一致；`render_frame` 消费面）：
```
probe_mem 0130 0160        # 行缓冲 $130-$160（含 flag $160）
probe_mem 00d3 00da        # 属性行 $D3-$DA
probe_mem 2005 2006        # PPU 地址/寄存器写（$2005/$2006）
probe_mem 008e 008f        # 行基址 zp $8E/$8F
probe_mem 0161 016f        # 调色板源 $161-$16F
probe_mem 008d 008d        # 行索引 $8D（行 LUT 触发）
probe_mem 002f 0030        # 文本流指针 $2F/$30（phase-batch 必需）
probe_mem 2006 2007        # PPU 地址/数据写（$2006/$2007 协议）
```

### 2.2 精确命令（串行执行；逐步核对）

```bash
cd <repo-root>
mkdir -p analysis/repro

# I1-I4：按序逐条执行（每条等它打印 DONE 再跑下一条；默认 timeout=1800s，长场景显式加大）
python tools/scenario_run.py run title-menu        --timeout 1200
python tools/scenario_run.py run gameplay-stage1   --timeout 1800
python tools/scenario_run.py run stg03-timeline    --timeout 5400
python tools/scenario_run.py run stg08-timeline    --timeout 7200

# I5/I6（2026-10-18 更正：四联真值来源改直入/自关窗轨；v5 36500f 截断已废弃）
python tools/scenario_run.py run tas-stg08b2-direct --timeout 5400   # stage8 直入；源含 10 probe
python tools/scenario_run.py run stg06-timeline     --timeout 5400   # STG-06 自关窗；源 0 probe（F49-2）
# I5b/I6b 为 §7.2 两遍 capture 的第二遍，在选帧后按 §3.2 make 生成 overlay 再跑（§7.2 内给命令）；
#   I6b = l4-stg06-pass2：源 0 probe → make 补 8 probe + 选定帧 capture（F49-2）

# 文本写集（phase-batch；cwd 必须是仓库根；缺省 out=analysis/phase/<scene>-text.jsonl）
cargo run -q -p game-oracle --bin phase-batch -- gameplay-stage1
cargo run -q -p game-oracle --bin phase-batch -- stg03-timeline
cargo run -q -p game-oracle --bin phase-batch -- stg08-timeline
cargo run -q -p game-oracle --bin phase-batch -- tas-stg08b2-direct
# stg06-timeline：源 0 probe ⇒ **不跑 phase-batch**（F49-2；它只作 expected capture 源，非 render 场景）
# l4-stg08b2-pass2 / l4-stg06-pass2 的 phase-batch 在 §7.2 选帧/重跑之后执行（M4：不在此处提前跑）
```
> **Windows 宿主执行提示（2026-10-18 环境更正）**：以上 `bash` 块在 **Git Bash**（或 POSIX 备选）可直接跑；
> Windows PowerShell 下：`mkdir -p analysis/repro` 换成 `New-Item -ItemType Directory -Force analysis/repro`，
> 其余 `python ...` / `cargo ...` 命令原样可用；多行 heredoc 脚本块（§3.3 等）建议在 Git Bash 执行或
> 存为 `.py` 后 `python <file>`（不要直接在 PowerShell 里贴 heredoc）。

> **headless 超时注（2026-10-18 task-91）**：上面命令里的 `--timeout <秒>` 是**驱动级**超时
> （`scenario_run.run_scenario(timeout_s=…)`）；`scenario_run.py` 的 headless 分支会**自动**再给
> `Mesen.exe` 传 **Mesen 级** `--timeout=max(120, timeout_s+120)`（`--testRunner` 参数区内），
> **无需手填 Mesen 级**。但若绕过驱动**手工** `Mesen.exe --testRunner <lua> <rom>`：**必须自带
> `--timeout=<≥场景预算秒>`**——Mesen testRunner 自带 `TestRunnerTimeout` 默认 **100s**（上游源码
> `CommandLineHelper.cs:31`），长场景（stg08 34000f / tas-stg08b2 40000f）会在 ~100s 被 Mesen 自身强停
> （trace 截断、`done` 缺失、`wall≈100-120s`）。**该失败不一定表现为 `timed_out=true`**（Mesen 自停、
> 驱动在进程退出后 20s 宽限即收工）⇒ **采集后必须校验 `done!=null && trace_lines==frames`**（见下判据）。

**每条采集的期望输出**（`scenario_run.py` 打印 + 文件判据）：
```
== scenario run: <name> ==
  DSL: oracle/scenarios/<name>.txt sha256=<16hex>…
  frames: <N> trace_lines: <N> captures: <n> shots: <n> wall: <s>s
  DONE: {...}
  trace: analysis/traces/<name>.frames.jsonl
  run清单: analysis/traces/<name>.run.json
```
**判据（逐场景；防截断，2026-10-18 task-91 强化）**：`run.json` 中 `timed_out=false`、**`done!=null`**、
**`trace_lines==frames`**（**三项缺一不可**：testRunner 内置 100s 超时会造成 `timed_out=false` 但 `done`
缺失/`trace_lines<frames` ⇒ 截断产物不得进入基线；见上方超时注）；
`.probe.jsonl` **头行数按 §2.4 表断言**（title 无文件；gameplay 15；stg03/stg08 **8**；`tas-stg08b2-direct` **10**；
**`stg06-timeline` 0 → 不产 `.probe.jsonl`**；`l4-stg06-pass2` 8），并记录到 `analysis/repro/probe_counts.tsv`
（用于 §3.3 校验）。【F49-2 更正】**
**`phase-batch` 期望**：打印 `phase-batch: <n> 调用回放，<m> 写点 → analysis/phase/<scene>-text.jsonl`；
`run.json` 无（该工具无清单）→ 记录文件大小+sha 到 `analysis/repro/phase_sha.tsv`。

> **不要用 `run-all`**：全量 40 场景会跑数小时且引入无关产物/踩踏风险；只跑 I1-I6。
> `run-all` 仅在全量重建时使用，且必须确认 `analysis/.scenario.lock` 独占。
> **M2**：#13 shop-window(5400) 直接用 `stg03-timeline` 源的 `capture_every 300 probe` 图
> （`shots/stg03-timeline_f05400_probe.png`），**不需要** I3b overlay。
> **2026-10-18 更正**：不再跑 `l4-baseline-v5`(36500f) 作为四联来源（v5 帧预算反证见 §3.7）；
> 它若保留仅作负候选对照（可跳过，环境清单见 L5 报告 §4.2）。

### 2.3 两跑确定性（输入层判据，P0-0-4 前置）

```bash
# run1 存档（在 I1-I6 全部完成后；title 无 probe 文件，不复制）
mkdir -p analysis/repro/run1
cp analysis/traces/{title-menu,gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct,stg06-timeline}.frames.jsonl analysis/repro/run1/
cp analysis/traces/{gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct}.probe.jsonl analysis/repro/run1/
cp analysis/phase/{gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct}-text.jsonl analysis/repro/run1/

# run2：同命令逐条重跑（旧分析产物会被自动归档；phase-batch 重生成）
python tools/scenario_run.py run title-menu        --timeout 1200
python tools/scenario_run.py run gameplay-stage1   --timeout 1800
python tools/scenario_run.py run stg03-timeline    --timeout 5400
python tools/scenario_run.py run stg08-timeline    --timeout 7200
python tools/scenario_run.py run tas-stg08b2-direct --timeout 5400
python tools/scenario_run.py run stg06-timeline     --timeout 5400
cargo run -q -p game-oracle --bin phase-batch -- gameplay-stage1
cargo run -q -p game-oracle --bin phase-batch -- stg03-timeline
cargo run -q -p game-oracle --bin phase-batch -- stg08-timeline
cargo run -q -p game-oracle --bin phase-batch -- tas-stg08b2-direct
mkdir -p analysis/repro/run2
cp analysis/traces/{title-menu,gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct,stg06-timeline}.frames.jsonl analysis/repro/run2/
cp analysis/traces/{gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct}.probe.jsonl analysis/repro/run2/
cp analysis/phase/{gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct}-text.jsonl analysis/repro/run2/

# 比对
python - <<'PY'
import hashlib, pathlib
r1=pathlib.Path("analysis/repro/run1"); r2=pathlib.Path("analysis/repro/run2")
bad=[]
for p in sorted(r1.rglob("*")):
    if not p.is_file(): continue
    q=r2/p.name
    h1=hashlib.sha256(p.read_bytes()).hexdigest()
    h2=hashlib.sha256(q.read_bytes()).hexdigest() if q.exists() else "MISSING"
    print(("OK " if h1==h2 else "DIFF "), p.name, h1[:16], h2[:16])
    if h1!=h2: bad.append(p.name)
print("DETERMINISM:", "PASS" if not bad else f"FAIL {bad}")
PY
```
**判据**：全部 `OK`（**6 个 frames 集合 + 5 个 probe/text 集合**；title 仅 frames；M8）；`DETERMINISM: PASS`。
**失败回退**：出现 DIFF ⇒ ①查是否并行/孤儿 Mesen（`pgrep -f tools/mesen2/Mesen`）；②查 Lua/DSL sha 是否变化
（`run.json.generated_lua_sha256`、`scenario_sha256` 应与 run1 相同）；③仍 DIFF ⇒ 该场景输入**不可复现**，
P0-0 **BLOCKED**（不得用"取某一次"绕过；上报差异文件与三方 sha）。

### 2.4 probe 上限与长跑监控（R-2；F4/F10）

**逐场景 probe 头行数（= `probe_mem` 注册数；`capture.lua` 每个区间写 1 条独立头行，不是"首行含 8 条"）**：

| 场景/overlay | probe_mem 注册数（头行） | `.probe.jsonl` | 旧 `.render.log` probe 签名 | 说明 |
|---|---|---|---|---|
| `title-menu` | 0 | **不产出** | 0 | 无 probe_mem |
| `gameplay-stage1` | **15** | 产出 | 1,307,322 | 源含 8+7（含 `002f 0030`/`2006 2007` 各重复一次，事件会双计——**源原样跑，勿去重**，签名以此为准） |
| `stg03-timeline` | **8** | 产出 | 598,571 | 标准 8 |
| `stg08-timeline` | **8** | 产出 | 1,167,314 | 标准 8 |
| `tas-stg08b2-direct` | **10** | 产出 | 不校验（L5 交叉核对） | 源含 8 标准 + `048c 048c` + `4000 4017`；四联 stg08-* 真值来源（§3.7） |
| `l4-stg08b2-pass2` | **10** | 产出 | 不作校验（M4） | 同 I5 + 选帧 capture；§7.2 之后才存在 |
| `stg06-timeline` | **0** | **不产出** | 不校验（L5 交叉核对；F49-2） | **源 0 条**（仅 frames/capture_every/poke/watch）；expected 采集源，非 render 场景 |
| `l4-stg06-pass2` | **8** | 产出 | 不作校验（M4） | I6 + 补 8 probe（源 0）+ 选帧 capture；§7.2 之后才存在 |
| ~~`l4-baseline-v5`~~ | ~~8~~ | — | ~~990,904（候选）~~ | **2026-10-18 更正**：v5 36500f 覆盖不到 stage6/8（§3.7），降级为负候选对照 |

```bash
# 每文件：总行数 − 该场景头行数 = probe 事件数；头行内容可 grep 断言
python - <<'PY'
import pathlib
EXP={"title-menu":0,"gameplay-stage1":15,"stg03-timeline":8,"stg08-timeline":8,
     "tas-stg08b2-direct":10,"stg06-timeline":0}   # F49-2：stg06 源 0 probe（不产 .probe.jsonl）
rows=[]
for name,n in EXP.items():
    p=pathlib.Path(f"analysis/traces/{name}.probe.jsonl")
    if n==0:
        print(("OK " if not p.exists() else "UNEXPECTED "), name, "no-probe"); continue
    lines=p.read_text(encoding="utf-8").splitlines()
    hdr=sum(1 for l in lines if l.startswith('{"probe"'))
    events=len(lines)-hdr
    print(("OK " if hdr==n else f"HEAD-MISMATCH({hdr}!={n}) "), name, f"events={events}")
    rows.append((name,hdr,events))
pathlib.Path("analysis/repro/probe_counts.tsv").write_text(
    "scene\theaders\tevents\n"+"\n".join(f"{a}\t{b}\t{c}" for a,b,c in rows)+"\n",
    encoding="utf-8", newline="\n")
PY
```
**判据**：①每场景 `headers == 上表`；②事件数留**安全裕度 < 3,000,000**（`capture.lua` 实现上限
= **3,000,001 事件**，`probeN>3,000,000` 后静默丢弃；超限的 `wc -l` 只减头行不算"完整"）；③对应场景事件数与
旧 `.render.log` 的 `probe=` 值相等（B+/A− 映射信号；组 D/直入轨因源配置不同，**事件数不作来源判据**，以
capture blob + mode 语义为准）。
**触顶处置**：该 pair 标 `blocked:probe-cap`；缓解 = 精简非必需 probe 区间（保留 8 标准区间）或由执行卡做
capture.lua 上限提升（tools 改动，独立 commit+复核），不得静默截断后当完整输入。

### 2.5 备选：fixtures 入库（政策待裁定）

若不希望每次重采（或 Linux 机不长期可用），可把 **render_frame 最小输入集**入库：

| 场景 | 需入库文件 | 用途 |
|---|---|---|
| 每场景 | `analysis/traces/<scene>.frames.jsonl` | render_frame 逐帧 zp/oam |
| 有 probe 的场景（title 除外） | `analysis/traces/<scene>.probe.jsonl` | 写流回放（注册数见 §2.4） |
| 每场景 | `analysis/phase/<scene>-text.jsonl` | 文本放置写集 |

- **候选目的地**（待 Lead/用户裁定）：`oracle/l4-fixtures/<scene>/{frames,probe,text}.jsonl` + `MANIFEST.json`
  （sha256/生成命令/commit/run.json sha）。
- **体积**：执行时 `du -h analysis/traces/*.jsonl analysis/phase/*.jsonl` 实测回填；若 >仓库可接受阈值
  （建议 ≤200MB 入库，超出走外部存储/不入库），则保留 Linux 执行机路线。
- **纪律**：入库件属于"可再生工作区产物"的例外，需 Lead 批准 + commit 说明（AGENTS §2/§9）；不得改
  `.gitignore` 绕过（需用户裁定政策，见 §10）。

### 2.6 步骤 1 产物汇总（本步交付）

```
# frames（全部场景集合；title 无 probe）
analysis/traces/{title-menu,gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct,stg06-timeline}.frames.jsonl
analysis/traces/{title-menu,gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct,stg06-timeline}.captures.jsonl
analysis/traces/{title-menu,gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct,stg06-timeline}.run.json
# probe（无 probe 场景不产出：title-menu）
analysis/traces/{gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct}.probe.jsonl
# text（stg06-timeline 源 0 probe ⇒ 不产 text，F49-2）
analysis/phase/{gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct}-text.jsonl
# 截图（capture 源图；§3.0 blob 校验）
shots/{gameplay-stage1,stg03-timeline,stg08-timeline,tas-stg08b2-direct,stg06-timeline,title-menu}_f*.png
# 复现清单
analysis/repro/{run1/,run2/,probe_counts.tsv,phase_sha.tsv,DETERMINISM.txt}
analysis/p0-0-env.txt
```
**判据**：§2.2 逐场景 DONE + §2.3 `DETERMINISM: PASS` + §2.4 头行/事件数断言通过；否则 BLOCKED。

---

## 3. 步骤 1b：映射恢复（22 对 ↔ 场景/帧）——P0-0 的核心取证步

> 旧 `.render.log` 只记 `name/frame/base/scroll/nt_nz/probe`（**不记 scene/env**，方案 F-0-C）。
> 因此映射必须用"**源 capture 现况 + probe 签名 + capture blob**"重导；**任何映射在 §3.3 三重校验前都是 B 级假设**。

### 3.0 capture 源现况（本轮新核，**关键简化**：绝大多数目标帧 capture 已在库内场景 DSL）

> 依据：`grep '^capture ' oracle/scenarios/*.txt`（A 级文件事实）。**这是 B 级"来源候选"**——最终仍以
> §3.3 ③ "重采 capture PNG sha256 == 旧 expected blob" 定谳（A）。`shots/` 命名 `shots/<scene>_f%05d_<tag>.png`。

| # | 帧 | 目标 capture 帧 | 源场景 | 源 capture 行（tag） | 是否需 overlay |
|---|---|---|---|---|---|
| 1-3 | gameplay-stage1_f01260/1500/2349 | 1260/1500/2349 | `gameplay-stage1` | `capture 1260 stage-start` / `1500 mid-combat` / `2349 end` | **否** |
| 4 | l4_clear-inject | 4600 | `stg03-timeline` | `capture 4600 stg02-clear-inject` | **否** |
| 5 | l4_combat-variety | 6500 | `stg03-timeline` | `capture 6500 stg03-combat` | **否** |
| 12 | l4_late-combat | 12000 | `stg03-timeline` | `capture 12000 stg03-late` | **否** |
| 14 | l4_stg02-start | 3404 | `stg03-timeline` | `capture 3404 stg02-start` | **否**（§7.1 重采真值） |
| 15 | l4_stg03-start | 5700 | `stg03-timeline` | `capture 5700 stg03-start-approx` | **否** |
| 13 | l4_shop-window | 5400 | `stg03-timeline` | `capture_every 300 probe`（5400=18×300，tag=`probe`） | **否**（M2：直接用源 capture_every 图） |
| 16 | l4_stg06-late | 22000 | `stg08-timeline` | `capture 22000 stg06-late` | **否**（§7.2 重采真值） |
| 17 | l4_stg07-combat | 16000 | `stg08-timeline` | `capture 16000 stg07-combat` | **否** |
| 18 | l4_stg07-start | 14600 | `stg08-timeline` | `capture 14600 stg07-start-approx` | **否** |
| 19 | l4_stg08-combat | 18000 | `stg08-timeline` | `capture 18000 stg08-combat` | **否**（§7.2 重采真值） |
| 20 | l4_stg08-late | 28000 | `stg08-timeline` | `capture 28000 stg08-late` | **否**（§7.2 重采真值） |
| 21 | l4_stg08-start | 16600 | `stg08-timeline` | `capture 16600 stg08-start-approx` | **否**（§7.2 重采真值） |
| 22 | title-menu_f00180_boot-black | 180 | `title-menu` | `capture 180 boot-black` | **否** |
| 6-10 | l4_ending-1..5 | 30000/31500/33000/34500/36000 | ~~`tas-clear-full-v5`~~ **来源待重裁（§3.6）** | `capture_every 1500 probe` → tag=`probe`（**仅当重裁命中该来源**） | **否**（capture 已覆盖；I5 只为补 probe） |
| 11 | l4_fade-clear | 28500 | ~~`tas-clear-full-v5`~~ **来源待重裁（§3.6）** | 同上 | **否**（同上） |

> 结论：**overlay 只用于 pass2 选帧 capture**——四联：`l4-stg08b2-pass2`（源 10 probe）+ `l4-stg06-pass2`
> （源 **0 probe → make 补 8**）；#13(5400) 用源 `capture_every 300` 图（M2），**不再造 I3b**（省一次 24000 帧重跑）。
> 组 A/B/C/E 直接跑源场景即可产 capture；**不需要为它们生成 overlay**（也就不会踩 F1 的重复 probe 注册）。
> **【2026-10-18 更正】** 上表 #6-#11 的"源场景=v5"**已被证伪**（`tas-clear-full.l5.jsonl`：
> stg01-clear f28364 / stg02-combat f29478 / stg02-clear f51332 ⇒ 28500-36000 = 关 2 战斗段）；
> 重裁候选与判据见 **§3.6**，未命中前这 6 对 `blocked:no-source`，不得按 v5 帧 staging。
> **【2026-10-18 更正·四联】** 上表未列出的四联 #16/#19/#20/#21 的 **expected 重采源**见 **§3.7**
> （stg08-* ← `tas-stg08b2-direct`；stg06-late ← `stg06-timeline`）；其 actual 渲染场景仍按 §3.1 的
> `stg08-timeline`（两者不同源，勿混）。

### 3.1 旧基线签名表（22 行；数据来自方案 §2.6 / `.render.log` 末条 / PNG sha256）

| # | 帧 | render 帧 | scroll | 旧 probe 签名 | 候选组 | scene 候选（B） | 源 capture（帧/tag，§3.0） | expected 现状 | P0-0 动作 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | gameplay-stage1_f01260 | 1260 | (0,114) | 1,307,322 | A | `gameplay-stage1` | 1260/stage-start | 有效 | 恢复源 capture→blob 校验→canonical 渲染 |
| 2 | gameplay-stage1_f01500 | 1500 | (0,114) | 1,307,322 | A | `gameplay-stage1` | 1500/mid-combat | 有效 | 同上 |
| 3 | gameplay-stage1_f02349 | 2349 | (0,96) | 1,307,322 | A | `gameplay-stage1` | 2349/end | 有效 | 同上 |
| 4 | l4_clear-inject | 4600 | (0,120) | 598,571 | B | `stg03-timeline` | 4600/stg02-clear-inject | 有效 | 同上 |
| 5 | l4_combat-variety | 6500 | (0,52) | 598,571 | B | `stg03-timeline` | 6500/stg03-combat | 有效 | 同上 |
| 6 | l4_ending-1 | 30000 | (0,0) | 990,904 | D | ~~`l4-baseline-v5`（v5 overlay）~~ **来源待重裁（§3.6）** | 30000/probe（capture_every） | 有效 | 恢复+重渲（破同 blob） |
| 7 | l4_ending-2 | 31500 | (0,0) | 990,904 | D | 同上 | 31500/probe | 有效 | 同上 |
| 8 | l4_ending-3 | 33000 | (0,0) | 990,904 | D | 同上 | 33000/probe | 有效 | 同上 |
| 9 | l4_ending-4 | 34500 | (0,0) | 990,904 | D | 同上 | 34500/probe | 与 #10 同文件（待裁） | 同上+裁定 |
| 10 | l4_ending-5 | 36000 | (0,0) | 990,904 | D | 同上 | 36000/probe | 与 #9 同文件（待裁） | 同上+裁定 |
| 11 | l4_fade-clear | 28500 | (0,170) | 990,904 | D | 同上 | 28500/probe | 有效 | 恢复+渲染 |
| 12 | l4_late-combat | 12000 | (0,0) | 598,571 | B | `stg03-timeline` | 12000/stg03-late | 有效 | 恢复+渲染 |
| 13 | l4_shop-window | 5400 | (0,0) | 598,571 | B | `stg03-timeline` | 5400/probe（源 `capture_every 300`，5400=18×300） | 有效 | 恢复+渲染（M2） |
| 14 | l4_stg02-start | 3404 | (0,208) | 598,571 | B | `stg03-timeline` | 3404/stg02-start | **无效（=boot-black 复制品）** | **重采真值**（§7.1） |
| 15 | l4_stg03-start | 5700 | (0,132) | 598,571 | B | `stg03-timeline` | 5700/stg03-start-approx | 有效 | 恢复+渲染 |
| 16 | l4_stg06-late | 22000 | (0,0) | 1,167,314 | C | `stg08-timeline` | 22000/stg06-late | **无效（=同 blob 复制；OBS-1 解释为 B）** | **重采真值**（§7.2） |
| 17 | l4_stg07-combat | 16000 | (0,74) | 1,167,314 | C | `stg08-timeline` | 16000/stg07-combat | 有效 | 恢复+渲染 |
| 18 | l4_stg07-start | 14600 | (0,144) | 1,167,314 | C | `stg08-timeline` | 14600/stg07-start-approx | 有效 | 恢复+渲染 |
| 19 | l4_stg08-combat | 18000 | (0,0) | 1,167,314 | C | `stg08-timeline` | 18000/stg08-combat | **无效（=同 blob 复制）** | **重采真值**（§7.2） |
| 20 | l4_stg08-late | 28000 | (0,0) | 1,167,314 | C | `stg08-timeline` | 28000/stg08-late | **无效（=同 blob 复制）** | **重采真值**（§7.2） |
| 21 | l4_stg08-start | 16600 | (0,0) | 1,167,314 | C | `stg08-timeline` | 16600/stg08-start-approx | **无效（=同 blob 复制）** | **重采真值**（§7.2） |
| 22 | title-menu_f00180_boot-black | 180 | (0,0) | 0 | E | `title-menu` | 180/boot-black | 有效 | 恢复+渲染（冒烟） |

> 组 A/B/C/D/E 的依据：旧 probe 签名唯一分组 + 场景帧数覆盖 + 现有 probe 配置 + §3.0 源 capture；**均为 B 级候选**，
> 必须经 §3.3 校验升 A。若所有候选都不过校验 → 该对 `blocked:no-source`，交 §7 重采裁定。
> **【2026-10-18 更正·组 D】** 上表 #6-#11 的 "scene 候选=`l4-baseline-v5`（v5 overlay）/源 capture=30000/probe"
> **作废**：`tas-clear-full.l5.jsonl` 证明 v5 的 f28500-36000 是关 2 战斗段（stg01-clear f28364、
> stg02-combat f29478、stg02-clear f51332），组 D 来源待重裁——**候选/判据/影响见 §3.6**。
> 标"同 blob 复制"的 4 张 expected：**同 blob=sha 事实（A）**；"为 OBS-1 商店冻结屏"是 B 级解释（`docs/bug-ledger.md`
> OBS-1 末关出店截断），不作为 A 级断言（F15）。

### 3.2 overlay 场景生成规则（**不改原场景**；**probe_mem 按区间去重**，F1）

对需要的组生成 `oracle/scenarios/l4-baseline-<src>.txt`（**【2026-10-18 更正·F49-1】overlay 名单**：
`l4-stg08b2-pass2`（源 `tas-stg08b2-direct`，源已 10 probe → 只加 capture）与
`l4-stg06-pass2`（源 `stg06-timeline`，源 **0 probe** → **补 8 probe** + 加 capture）；
旧 `l4-baseline-v5`/`l4-baseline-v5-pass2` **作废降级为负候选**，见 §3.7）：
1. 取源场景全部指令（`press/hold/poke/press_every/poke_every/bot/press_len/watch/capture_every` 原样保留；
   **输入语义不变**）；
2. `name l4-baseline-<src>`（产物独立命名，不覆盖源场景归档）；
3. `frames <覆盖窗口>`（只截断到需要帧，不改输入前缀）；
4. **probe_mem 增量补全**：读源已注册区间集合，**只追加缺失的标准区间**（已存在的**不重复追加**；
   `stg06-timeline` 源 0 条 ⇒ 补 8；`tas-stg08b2-direct` 源 10 条 ⇒ 补 0）；
5. 增补 `capture <frame> l4f<frame>`（仅缺 capture 的目标帧；tag 稳定，供 blob 比对）；
6. 保留源 `capture_every`/`watch`（`capture_every` 提供**各源自身**的帧 capture；watch 供 §7.2 选帧）。
   **【F49-1 更正】** ~~capture_every 提供 v5 的 ending/fade 帧 capture~~ 已作废（组 D 来源≠v5，见 §3.6）。

```python
# tools/l4_make_overlay.py（执行卡实现；本手册给规格实现；Python>=3.10）
import pathlib, sys
STD_PROBES = [("0130","0160"),("00d3","00da"),("2005","2006"),("008e","008f"),
              ("0161","016f"),("008d","008d"),("002f","0030"),("2006","2007")]
KEEP_OPS = {"press","hold","press_every","poke","poke_every","press_len","bot","watch","watch_range","ramframe","capture_every"}

def make(src: str, dst: str, frames: int, captures=()) -> None:
    src_lines = pathlib.Path(src).read_text(encoding="utf-8").splitlines()
    have, body = set(), []
    for raw in src_lines:
        s = raw.split("#", 1)[0].strip()
        if not s:
            continue
        toks = s.split()
        op = toks[0]
        if op in ("name", "frames", "capture"):
            continue
        if op == "probe_mem":
            have.add((toks[1].lower(), toks[2].lower()))
            body.append(raw)                      # 源区间原样保留（含源内重复登记，不改既有签名）
            continue
        if op in KEEP_OPS:
            body.append(raw)
    missing = [(lo, hi) for lo, hi in STD_PROBES if (lo, hi) not in have]
    out = [f"name {pathlib.Path(dst).stem}", f"frames {frames}"] + body
    out += [f"probe_mem {lo} {hi}" for lo, hi in missing]     # 只补缺失（F1）
    out += [f"capture {f} l4f{f}" for f in captures]
    pathlib.Path(dst).write_text("\n".join(out) + "\n", encoding="utf-8", newline="\n")
    # 生成后自检：注册数 = 源注册数 ∪ 缺失数；且同区间不重复（源内自带的重复登记原样保留、不新增）
    total = sum(1 for l in out if l.startswith("probe_mem "))
    print(f"{dst}: probe_mem={total} (source={len(have)} + missing={len(missing)})")

if __name__ == "__main__":
    # ── 2026-10-18 更正（§3.7 / F49-1）：旧 v5-overlay 作废，四联来源改直入/自关窗轨 ──
    # 旧（作废）：make("oracle/scenarios/tas-clear-full-v5.txt","oracle/scenarios/l4-baseline-v5.txt",36500)
    # 新：stg08b2 源已含 10 probe → pass2 只加 capture；stg06 源 0 probe → pass2 补 8 probe + 加 capture
    # make("oracle/scenarios/tas-stg08b2-direct.txt","oracle/scenarios/l4-stg08b2-pass2.txt",40000,[...])
    # make("oracle/scenarios/stg06-timeline.txt","oracle/scenarios/l4-stg06-pass2.txt",30000,[...])
    pass   # 2026-10-18 更正：旧 v5 调用行已注释留档；实际调用见 §3.7 pass2 命令块
```
> **M2**：不再为 #13(5400) 生成 I3b overlay——源 `stg03-timeline` 的 `capture_every 300 probe` 已产出
> `shots/stg03-timeline_f05400_probe.png`（5400=18×300），直接用于 §3.3 ③ blob 校验。
**自检判据**（生成后立刻跑；两个 pass2 overlay）：
> **执行时点（F55-2）**：本自检必须在 **§3.7 的 `make(...)` 生成两个 pass2 overlay 之后**执行
> （§3.2 本节只给生成规则；`l4-stg08b2-pass2.txt`/`l4-stg06-pass2.txt` 到 §3.7 才落盘）。
> 若在 §3.2 立即跑会 `FileNotFoundError`——属预期（不是生成失败），按 §3.7 后补跑。
```bash
python - <<'PY'
import collections, pathlib
for p,exp in [("oracle/scenarios/l4-stg08b2-pass2.txt",10),("oracle/scenarios/l4-stg06-pass2.txt",8)]:
    regs=[l.split()[1:3] for l in pathlib.Path(p).read_text(encoding="utf-8").splitlines()
          if l.startswith("probe_mem ")]
    dup=[k for k,v in collections.Counter(map(tuple,regs)).items() if v>1]
    print(p, "regs=",len(regs), "dup=",dup)
    assert len(regs)==exp and not dup, f"{p}: probe 注册数/去重异常"
PY
```
**注意（2026-10-18 更正）**：
- `tas-stg08b2-direct`（stage8 直入轨）：含 `poke 1265 0180 08`（选关）+ assist 三 poke + 源 10 条 probe
  ⇒ 产出锚/expected 来源**必须标 `trigger=injected`**（选关 poke；README 纪律：poke run 不得冒充 natural）；
- `stg06-timeline`：含前关注入链（poke）⇒ 同标 `trigger=injected`；
- **不再使用** v5 36500f 方案（v5 到 f73939 才 stage4-combat，选帧必失败——§3.7 A 级反证）。

### 3.3 三重校验（映射成立判据；全部通过才升 A；F4/F16）

```bash
# ① probe 事件数（映射信号）：wc-l − 该场景头行数（§2.4 表）== 旧 .render.log 的 probe= 值
python - <<'PY'
import pathlib
# 【F55-1 更正】分两段：PRE 在 §3 即可跑；POST（pass2）在 §7.2/§3.7 生成后补跑——缺失显式 SKIP 而非崩。
PRE={"gameplay-stage1":15,"stg03-timeline":8,"stg08-timeline":8,
     "tas-stg08b2-direct":10}   # F49-1：直入轨 10 条
POST={"l4-stg06-pass2":8}       # 到 §3.7 才生成（F55-1）
OLD={"gameplay-stage1":1307322,"stg03-timeline":598571,"stg08-timeline":1167314,
     "tas-stg08b2-direct":None,"l4-stg06-pass2":None}   # 新轨无对应旧签名 → 不作来源判据
for n,h in PRE.items():
    lines=pathlib.Path(f"analysis/traces/{n}.probe.jsonl").read_text(encoding="utf-8").splitlines()
    ev=len(lines)-h; print(n,"events=",ev,"old=",OLD[n],"match=", OLD[n]==ev if OLD[n] else "n/a")
for n,h in POST.items():
    p=pathlib.Path(f"analysis/traces/{n}.probe.jsonl")
    if not p.exists():
        print(f"SKIP {n}: 文件未生成（§7.2/§3.7 后补跑本步 POST 段；F55-1）"); continue
    lines=p.read_text(encoding="utf-8").splitlines()
    ev=len(lines)-h; print(n,"events=",ev,"old=",OLD[n],"match=", OLD[n]==ev if OLD[n] else "n/a")
PY
# ② 输入确定性：§2.3 DETERMINISM: PASS
# ③ capture blob == 旧 expected（A 级主判据；路径=§3.0 源 capture / pass2 overlay capture）
python - <<'PY'
import hashlib, pathlib
E=pathlib.Path("oracle/screens/expected"); S=pathlib.Path("shots")
checks=[("gameplay-stage1_f01260_stage-start", "gameplay-stage1_f01260_stage-start.png"),
        ("gameplay-stage1_f01500_mid-combat", "gameplay-stage1_f01500_mid-combat.png"),
        ("gameplay-stage1_f02349_end",        "gameplay-stage1_f02349_end.png"),
        ("l4_clear-inject", "stg03-timeline_f04600_stg02-clear-inject.png"),
        ("l4_combat-variety","stg03-timeline_f06500_stg03-combat.png"),
        ("l4_late-combat",   "stg03-timeline_f12000_stg03-late.png"),
        ("l4_stg03-start",   "stg03-timeline_f05700_stg03-start-approx.png"),
        ("l4_shop-window",   "stg03-timeline_f05400_probe.png"),   # 源 capture_every 300（M2）
        ("l4_stg07-combat",  "stg08-timeline_f16000_stg07-combat.png"),
        ("l4_stg07-start",   "stg08-timeline_f14600_stg07-start-approx.png"),
        ("title-menu_f00180_boot-black","title-menu_f00180_boot-black.png"),
        # 【F49-1 更正】组 D（ending/fade）：旧行 `l4-baseline-v5_f30000_probe.png`/`f28500_probe.png` 作废；
        # 来源待 §3.6 重裁命中后填 `shots/<winner_scene>_f<帧>_<tag>.png`（禁止再用 v5 图）。
        # 四联（#16/#19/#20/#21）：expected 将被替换，③ 不比旧图——改比 §3.7 pass2 选帧 capture（§7.2/§7.7）。
        ]
for exp, shot in checks:
    a=(E/f"{exp}.png").read_bytes(); b=(S/shot).read_bytes()
    print(("BLOB-EQ " if a==b else "BLOB-DIFF "), exp, "<-", shot,
          "sha256exp=",hashlib.sha256(a).hexdigest()[:12], "shot=",hashlib.sha256(b).hexdigest()[:12])
PY
```
**判据**：③ **capture blob 相等 = 主判据（A）**；① 事件数相等 = 强信号（B+），不等**不单独排除**（旧战役
probe 区间集可能不同；组 D/直入轨无旧签名、不作来源判据）；② 确定性为前置门。全部通过 ⇒ `mapping=confirmed`
（scene/frame/capture_tag/`frame_offset` 写入 manifest）。**capture 来源标签（F49-5）**：③ 每行必须注明
capture 出处——源 `capture_every N tag`（如 `stg03-timeline_f05400_probe.png`）或 overlay 显式
`capture <f> l4f<f>`（如 pass2）；manifest `capture_tag` 字段与文件名 tag 一致。
**执行顺序（F55-1）**：① 的 **PRE 段**（gameplay/stg03/stg08/tas-stg08b2）在 §3 跑；**POST 段**
（`l4-stg06-pass2`）在其文件生成后（§7.2/§3.7）补跑本脚本 POST 段——缺失时打印 `SKIP ...` 并跳过、不
`FileNotFoundError`；或把整段 ① 移到 §7a 后统一跑一次（两种执行等价，取其一）。
**不成立时**：尝试同组下一候选场景/相邻帧（Q4 +1 帧延迟约定：`capture N` 与 render 帧 N 存 ±1 帧可能——
若 N±1 命中同样记 A，但 manifest 记 `frame_offset`）；全候选失败 ⇒ `blocked:no-source`。
> 组 D（v5）的"同余≠来源"：`capture_every 1500` 只保证帧可被覆盖；来源判定**只靠 ③ blob**（附录 D2/S5）。
> **【2026-10-18 更正·L5 交叉核对】** "组 D=v5"假设已被语义反证（v5 f28500-36000 = 关 2 战斗段，
> 见 §3.6 的 A 级证据表）；③ 校验对组 D 需改用 §3.6 重裁选中的来源，v5 只作负候选对照。

### 3.4 候选回退顺序（逐组）

| 组 | 第一候选（源 capture 已覆盖，§3.0） | 第二候选 | 失败处置 |
|---|---|---|---|
| A | `gameplay-stage1`（capture 1260/1500/2349 已在源） | — | `blocked:no-source` |
| B | `stg03-timeline`（capture 3404/4600/5700/6500/12000 已在源 + `capture_every 300` 覆盖 5400） | `stg02-timeline`(19000f)/`stg04-timeline`(26000f) | `blocked:no-source` |
| C | `stg08-timeline`（capture 14600/16000/16600/18000/22000/28000 已在源） | `stg07-timeline`(32000f) | `blocked:no-source` |
| D | ~~`l4-baseline-v5`（源 `tas-clear-full-v5`；capture_every 覆盖 ending/fade）~~ **来源待重裁（§3.6）** | ~~`tas-clear-full-v4/v3/v2`~~ **§3.6 候选 C1-C5（`tas-stg08b2-direct` 优先）** | **重裁命中→回填；无来源→`blocked:no-source` + `s5-full-inject` 新采集**（§3.6） |
| E | `title-menu`（capture 180 boot-black 已在源） | — | `blocked` |

> 组 D 的 probe 签名 990,904 与现行场景不直接对应，是**最不确定**组；~~`tas-clear-full-v5` 的
> `capture_every 1500 probe` 与 ending 帧（30000/31500/33000/34500/36000）**精确同余**（1500 的倍数），
> 且 fade 28500=19×1500 ⇒ 强候选（B+）~~【**2026-10-18 更正**：同余算术成立但来源=v5 语义不成立（v5
> f28500-36000=关 2 战斗段）— 候选改为 §3.6 C1-C5，v5 仅作负候选】。**同余只证帧可被覆盖、不证来源**
> （附录 D2/S5）；只有候选 capture 与旧 expected blob 相等且 mode 语义匹配才升 A（§3.6 判据）。

### 3.5 步骤 1b 产物

`analysis/repro/mapping.tsv`（22 行，**schema 如下**）+ 通过校验的 overlay 场景文件（待执行卡 commit）。
列：`name \t scene \t frame \t capture_tag \t frame_offset \t trigger \t expected_source \t
probe_match \t det \t blob_eq \t status \t duplicate_of \t reuse`
（组 A 的 capture_tag = `stage-start`/`mid-combat`/`end`，见 §3.0；`duplicate_of` 供 §7.5/F8 登记）。
- **`expected_source` 格式写死为字符串 `scene:frame:capture_tag`**（如 `stg03-timeline:4600:stg02-clear-inject`；
  与 §5.1 schema/M5 一致，不用对象）；
- **`reuse`**：`independent`（12 对）或 `reuse`（10 对；按方案 §2.6/附录 A 的"独立/复用"列逐个填，M1）——
  wrapper 直接读该列，**不再一律默认 independent**；
- **`status` 透传（M1）**：`ok` / `replaced-invalid-asset`（#14、#16/#19/#20/#21 及 §7.4 重采）/ 
  `documented-duplicate`（ending-4/5 裁定）；wrapper 读该列写入 manifest，**不得丢失**（仅渲染失败时覆盖为
  `blocked:nondeterministic`）。

### 3.6 【2026-10-18 更正·L5 交叉核对】组 D（fade/ending 6 对）来源≠v5 → 重裁流程

> **原假设（保留留档，不删）**：§0 R-1/§2.1 I5/§3.0-§3.4 曾以 `tas-clear-full-v5` 为组 D
> （#6-#11：ending-1..5 + fade-clear）来源候选，依据是其 `capture_every 1500 probe` 与
> 28500/30000/31500/33000/34500/36000 精确同余（附录 D2）。**该假设"帧可覆盖"的算术成立，但"来源=v5"不成立。**

**A 级反证（可复算，`oracle/traces/tas-clear-full.l5.jsonl`）**：

| 行 | scene | f | mode | checkpoint |
|---|---|---|---|---|
| 1 | tas-clear-full | **28364** | StageClear | stg01-clear |
| 2 | 同 | 29078 | Shop | stg02-shop |
| 3 | 同 | 29478 | Combat | stg02-combat |
| 4 | 同 | **51332** | StageClear | stg02-clear |

（来源 `tas-clear-full-v5(assist=invincible)`；复现：`python -c "import json;[print(json.loads(l)) for l in open('oracle/traces/tas-clear-full.l5.jsonl')]"`）
⇒ 在 v5 帧域里 **f28500 仍在关 1 关末段、f30000-36000 = 关 2 战斗段**（stg02-clear 要到 f51332），不可能是
ending/fade 画面。**组 D 来源待重裁**；P0-0 若按原假设执行会取错帧/错判来源。

**重裁候选清单（环境内按序验；优先级=证据强度）**：

| 序 | 候选来源 | 理由/证据 | 需 overlay |
|---|---|---|---|
| C1 | **`tas-stg08b2-direct`** | 仓库内**唯一**同时具备 10 条 probe_mem（8 标准 + `048c` + `4000-4017`）、`capture_every 1500`、`frames 40000`（覆盖 28500-36000）、`poke 1265 0180 08` 选关 stage8 + assist 三 poke 的 run ⇒ 与旧 `.render.log`（帧号+probe 存在）最契合 | 否（源已含 probe） |
| C2 | `tas-stg07-relay` | `poke 1255 0180 07` 选关 stage7；`stg07-clear=f28683` 已在 `tas-clear-full.l5.jsonl` ⇒ 其后 stage8/ending 可能落在 30000-36000；`capture_every 1500` 覆盖目标帧 | **是**（源无 probe_mem，需加 8 区间才能产 probe 文件） |
| C3 | `tas-stg08-direct` / `tas-stg08b-direct` | 选关 stage8（`poke 0180 08`），frames 120000，`capture_every 1500` | 是（无 probe_mem） |
| C4 | `tas-stg05-relay` | 选关 stage5；ending 可能更晚（帧域不同） | 是（无 probe_mem） |
| C5 | **新采集 `s5-full-inject`**（无现成来源时） | 注入全链：每关 `$26:=$10 + $0400:=$01`（前 6 关已证）+ 转场 +34f 常数，第 8 关关末门后自然 `INC $180→9` 走 E392 通关支；帧预算 ~40k；`capture_every 300/1500` + `probe_mem` 8 区间 + 显式 `capture` 抓 Ending/fade | —（新建） |
| 负候选 | `tas-clear-full-v5` / v4/v3/v2 | **语义已证伪**（见上表）：f28500-36000 不是 ending；仅用于"blob 必然不等"的对照 | — |

**C5 `s5-full-inject` 骨架（F49-4；当前仅"半可执行"——不作 P0-0 当前回退）**：
> 下方为**骨架**：前 6 关注入帧来自现有 timeline 链实测锚（§2.2/§3.7 同源），**stage-8 注入 #8 帧与 ending
> 观测窗未定**（需先有 stg08 关内 trace）⇒ 标 `TODO(新卡 B4)`，**不得发明帧号**；故 C5 须由 B4 卡在环境内
> 定稿后才可当回退（C1-C4 均未命中时）。
```text
name s5-full-inject
frames 40000
press_len 4
press 240 start
press 300 a
press 360 start
press 420 a
press 600 start
press 660 a
press 900 start
press 1020 a
press 1100 b
press 1160 b
# 关末注入（前 6 关已证；帧号=现有 timeline 链实测锚，双写 $26 bit4 + slot0 非零门）
poke 2400 0026 10
poke 2400 0400 01
poke 4600 0026 10
poke 4600 0400 01
poke 6400 0026 10
poke 6400 0400 01
poke 8500 0026 10
poke 8500 0400 01
poke 10600 0026 10
poke 10600 0400 01
poke 13500 0026 10
poke 13500 0400 01
poke 15500 0026 10
poke 15500 0400 01
# TODO(stage8)：注入 #8 帧 + ending 观测窗待 stg08 关内 trace 定（B4 卡；本骨架不得自行发明）
probe_mem 0130 0160
probe_mem 00d3 00da
probe_mem 2005 2006
probe_mem 008e 008f
probe_mem 0161 016f
probe_mem 008d 008d
probe_mem 002f 0030
probe_mem 2006 2007
capture_every 300 probe
watch 0180 stage_no
watch 0026 flags26
watch 0027 flags27
watch 002b phase2b
watch 002f countdown2f
watch 0300 gate300
watch 0400 pstate
watch 018d lives
watch 018e money_disp_copy
watch 0188 intro_ctr
watch 041c vec41c
watch 0438 vec438
watch 0018 controller_held
```
**判据（C5 定稿后同 §3.6 三判据）**：blob == 旧 expected；`flags26 bit7`=Ending / fade 平均亮度；
trigger=injected（每关都 poke）。**当前状态**：骨架已给，stage-8/ending 帧待 B4 卡，**不得作为当前可执行回退**。

**判据（三条同时满足才升 A）**：
1. **blob**：候选 run 在目标帧（28500/30000/31500/33000/34500/36000）的 capture 截图 sha256 == 旧 expected
   （fade/ending-1..3 逐对；ending-4/5 同 blob 一并核）。**capture 来源标签（F49-5）**：必须注明是源
   `capture_every <N> <tag>`（如 `_probe.png`）还是 overlay 显式 `capture <f> l4f<f>`；manifest `capture_tag`
   与文件名 tag 一致；
2. **语义/mode 匹配（F49-3 更正：可执行化）**：`frames.jsonl` 只有 `{"f","w"}`，**没有 mode 字段** ⇒
   改为从候选场景的 `watch 0026 flags26` 推导（C1/C2/C3 均含该 watch）。**F55-3 更正：`<winner>` 给实际
   文件示例**（按命中候选替换文件名即可）：
   - **C1（首选）** → `analysis/traces/tas-stg08b2-direct.frames.jsonl`；**C2** → `analysis/traces/tas-stg07-relay.frames.jsonl`；
     **C3** → `analysis/traces/tas-stg08-direct.frames.jsonl`（若该 run 未跑则先按 §2.2 采集）。
   - **Ending 判定**：`$26 bit7=0x80`（`$95F9` 入口 `LDA #$80/STA $26` 后置位；见 l5-all-clear R-l5-5）：
     ```bash
     python - <<'PY'
     import json
     F=30000                                                     # 目标帧（28500/30000/...）
     PATH="analysis/traces/tas-stg08b2-direct.frames.jsonl"      # C1 示例（C2: tas-stg07-relay）
     for l in open(PATH, encoding="utf-8"):
         r=json.loads(l)
         if r["f"]==F:
             f26=int(r["w"]["flags26"])
             print(f"f={F} flags26={f26:#04x} bit7={bool(f26&0x80)} => {'Ending' if f26&0x80 else '非 Ending'}")
     PY
     ```
   - **fade（近黑过渡）判定**：`flags26 bit7=0` 时不能靠 mode 字段 ⇒ 用 **capture 截图平均亮度**（`tools/diff_visual.decode_png` 可复用）：
     ```bash
     python - <<'PY'
     import sys; sys.path.insert(0,"tools")
     from diff_visual import decode_png
     _,_,px = decode_png("shots/tas-stg08b2-direct_f28500_probe.png")   # C1 示例（C2 用 tas-stg07-relay_…）
     w,h=256,240
     mean=sum(px[y*3*w+x*3]+px[y*3*w+x*3+1]+px[y*3*w+x*3+2] for y in range(h) for x in range(w))/(3*w*h)
     print("mean_luminance=",round(mean,2),"（fade 经验阈值 < ~20/255 记近黑，B 级辅助）")
     PY
     ```
   **必须排除 Combat/Shop**：`flags26` 无 bit7 且画面平均亮度不低 ⇒ 不是 ending/fade（A 级 trace 字段+截图，
   不看文件名）。若需**精确 mode 字段**：须另产 `.l5.jsonl`（当前无独立 bin；由 port/engine 侧另出，不属 P0-0
   路径）——本步骤以 flags26+亮度为可执行判据；
3. **trigger 合法**：stage-select poke（`0180`）⇒ `trigger=injected`；assist（`01be/01b6/01b7`）⇒ 标
   `trigger=assisted`（README 纪律：poke run 不得冒充 natural）；natural 仅当无 poke 且自然到达。

**处置**：
- 命中 ⇒ 更新 §3.1/§3.3 的 mapping 行（scene/frame/capture_tag/frame_offset/trigger）+ §7.3/§7.4 的
  `expected_source` + §7.7 staging 源；并落 **`analysis/repro/l4-groupD-source.json`**（§7.7 A 读取的机器台账）：
  `{"winner":"tas-stg08b2-direct","targets":[28500,30000,...],"captures":{"l4_fade-clear":"shots/tas-stg08b2-direct_f28500_probe.png",...},
  "trigger":{"l4_ending-1":"injected",...},"mode_evidence":{"l4_ending-1":"frames#f30000 flags26=0x80 (bit7)",
  "l4_fade-clear":"capture 平均亮度=<v>（近黑）"},...}`（**F55-3**：`winner`/文件名给 C1 实际示例；
  命中 C2 时把 `tas-stg08b2-direct` 整体换成 `tas-stg07-relay` 即可）；
  原"组 D=v5"假设在本节划线留档。
- 无来源 ⇒ 组 D 6 对 `status=blocked:no-source`；**新采集 `s5-full-inject`**（§C5；场景名/帧锚/assist 规则按
  C5 表；产物 `analysis/traces/s5-full-inject.*` 与 capture 图），按上述三判据复验后回填（同样落 `l4-groupD-source.json`）。

**对 P0-0 的影响（必读）**：
- **§0.5**：§7a 内新增子步骤"§3.6 组 D 重裁"，仍必须在 §5 wrapper 之前完成（expected 定稿依赖它）；
  重裁未决时**不得**按旧假设 staging 组 D 资产。
- **§7.7**：staging A 的 ending-1..5/fade 项 conditional 于 §3.6 命中结果——未命中/未决则**跳过 staging**
  （保留旧 expected 并标 `blocked:no-source`），只安装已重裁的 stg02/四联；manifest 对应 6 行 status=blocked，
  §8.2 验收对这 6 行不通过（P0-0 不得宣称全绿，直到重裁完成）。
- **§7.3/§7.4**：动作语义从"用 v5 capture 比对"改为"用 §3.6 重裁选中来源的 capture 比对"；旧动作留档。

### 3.7 【2026-10-18 更正·L5 交叉核对】I5/§7.2 的 36500f 截断失效——四联真值来源改"直入/自关窗轨"

> **原假设（留档不删）**：§2.1 I5/I5b 用 `l4-baseline-v5`（**截断 v5 到 36500f**）+ pass2，按
> `stage_no==6/8` 选帧 → 替换四联 #16/#19/#20/#21 expected。
> **A 级反证**（`oracle/traces/tas-clear-full.l5.jsonl`，bot 域）：
> `stg01-clear f28364` → `stg02-shop f29078` → `stg02-combat f29478` → `stg02-clear f51332` →
> `stg03-clear f72823` → `stg04-shop f73539` → `stg04-combat f73939`
> ⇒ **v5 到 f73939 才到 stage4**；36500f 时仍在关 2 ⇒ `stage_no==6/8` 选帧**必然失败**（原 I5/I5b 作废）。
> 复现：`python -c "import json;[print(json.loads(l)['f'],json.loads(l)['checkpoint']) for l in open('oracle/traces/tas-clear-full.l5.jsonl')]"`

**更正后来源表（四联真值）**：

| pair | 原（作废） | 新来源场景 | 选帧判据（trace `frames.jsonl`） | pass2 overlay | 产物 |
|---|---|---|---|---|---|
| #21 `l4_stg08-start` | ~~v5 36500f stage_no==8~~ | **`tas-stg08b2-direct`**（stage8 直入、40000f、源含 10 probe） | `stage_no==8 ∧ intro_ctr>0` 的**首帧** | `l4-stg08b2-pass2` | `shots/l4-stg08b2-pass2_f<f>_l4f<f>.png` |
| #19 `l4_stg08-combat` | 同上 | 同上 | `stage_no==8 ∧ intro_ctr==0` 的**中位帧** | 同上 | 同上 |
| #20 `l4_stg08-late` | 同上 | 同上 | `stage_no==8 ∧ w0189>0` 的**最后帧** | 同上 | 同上 |
| #16 `l4_stg06-late` | 同上 | **`stg06-timeline`**（自关窗 f13000-13977；源 **0 probe**（I6b 补 8）+ capture_every 300） | 窗口 `13000≤f≤13977` 内 `stage_no==6 ∧ w0189>0` 的**最后帧** | `l4-stg06-pass2` | `shots/l4-stg06-pass2_f<f>_l4f<f>.png` |

**选帧脚本（替换 §7.2 旧 v5 版；写含 scene 字段的选择文件）**：
```python
import json, pathlib
def rows(p): return [json.loads(l) for l in open(p, encoding="utf-8")]
a = rows("analysis/traces/tas-stg08b2-direct.frames.jsonl")
b = rows("analysis/traces/stg06-timeline.frames.jsonl")
def first(rs, pred):
    c=[r["f"] for r in rs if pred(r["w"])]; assert c, "选帧失败（first）"; return c[0]
def last(rs, pred):
    c=[r["f"] for r in rs if pred(r["w"])]; assert c, "选帧失败（last）"; return c[-1]
def median(rs, pred):
    c=[r["f"] for r in rs if pred(r["w"])]; assert c, "选帧失败（median）"; return c[len(c)//2]
sel={
 "l4_stg08-start":  {"scene":"l4-stg08b2-pass2","frame":first(a,  lambda w: w["stage_no"]==8 and w["intro_ctr"]>0)},
 "l4_stg08-combat": {"scene":"l4-stg08b2-pass2","frame":median(a, lambda w: w["stage_no"]==8 and w["intro_ctr"]==0)},
 "l4_stg08-late":   {"scene":"l4-stg08b2-pass2","frame":last(a,   lambda w: w["stage_no"]==8 and w["w0189"]>0)},
 "l4_stg06-late":   {"scene":"l4-stg06-pass2",  "frame":last([r for r in b if 13000<=r["f"]<=13977],
                                                             lambda w: w["stage_no"]==6 and w["w0189"]>0)},
}
pathlib.Path("analysis/repro/l4-selected-frames.json").write_text(
    json.dumps(sel, ensure_ascii=False, indent=2)+"\n", encoding="utf-8", newline="\n")
print(sel)
```
**pass2 生成/重跑（两跑，§7.2 第二遍）**：
```bash
python - <<'PY'
import json, pathlib, sys
sel=json.loads(pathlib.Path("analysis/repro/l4-selected-frames.json").read_text(encoding="utf-8"))
sys.path.insert(0,"tools"); from l4_make_overlay import make
make("oracle/scenarios/tas-stg08b2-direct.txt","oracle/scenarios/l4-stg08b2-pass2.txt", 40000,
     [sel[n]["frame"] for n in sel if sel[n]["scene"]=="l4-stg08b2-pass2"])
make("oracle/scenarios/stg06-timeline.txt","oracle/scenarios/l4-stg06-pass2.txt", 30000,
     [sel[n]["frame"] for n in sel if sel[n]["scene"]=="l4-stg06-pass2"])
PY
python tools/scenario_run.py run oracle/scenarios/l4-stg08b2-pass2.txt --timeout 5400
python tools/scenario_run.py run oracle/scenarios/l4-stg06-pass2.txt     --timeout 5400
cargo run -q -p game-oracle --bin phase-batch -- l4-stg08b2-pass2
cargo run -q -p game-oracle --bin phase-batch -- l4-stg06-pass2
```
**trigger 登记**：两源均含 poke（`tas-stg08b2-direct`：`0180` 选关 + assist 三 poke；`stg06-timeline`：前关注入链）
⇒ 四联新 expected 的 `expected_source` 可写，但 manifest `trigger` 如实记 **`injected`**（选关 poke）——
不得冒充 natural（README 写注入纪律）；这与"expected 图来自 redirect/自关窗轨"的来源说明并列。
**影响面（本更正同时改）**：§0.5（执行顺序/环境清单）、§2.1（I5/I5b→I5/I5b + I6/I6b；旧 v5 降负候选）、
§2.2（命令/phase-batch）、§2.3（两跑集合）、§2.4（头行表）、§2.6（产物清单）、§7.2（选帧来源）、
§7.7（staging 源）、§8.2（验收项）、附录 D（新增 D9 帧预算复算）。**旧 I5/I5b 不再用于四联/组 D staging。**

---

## 4. 步骤 2：输出参数化（P0-0-2）——**实现动作，属执行卡（crates 写权限），本手册只给规格**

### 4.1 最小代码改动规格（`crates/game-desktop/src/bin/render_frame.rs`）

现状（A 级代码事实）：输出写死 `oracle/screens/actual/{name}.png` + `.idx.json` + **追加**
`oracle/screens/actual/.render.log`（导出段）。改动（保持默认行为不变）：

1. 参数解析（与现有 `--base/--probe-file/--stream-file/--text-file` 同风格）：
   - `--out-dir <dir>`：输出目录（png+idx 写此处）；缺省 = `<repo_root>/oracle/screens/actual`；
   - `--log-path <file>`：日志文件路径；缺省 = `<out_dir>/.render.log`。
2. 日志行扩展（兼容旧字段，新增在行尾）：
   `"{name} frame={frame} base={base} scene={scene} scroll=(sx,sy) nt_nz=… probe=… env={K=V,…} in=frames:<sha12>,probe:<sha12>,text:<sha12> out=<png_sha12>"`；
   `env` 只记录**显式设置**的键（空=生产默认），`in` 为三个输入文件 sha256 前 12 hex，`out` 为输出 png sha256。
3. 缺失输入时**显式报错退出码非 0**（现在 `expect("trace")` 会 panic；改为 `eprintln!` + `exit(2)` 便于脚本判定）。
**验收判据（执行卡内）**：`cargo build -p game-desktop` 通过；**不带新参数时输出路径/行为与改前一致**——
回归方式（**B4 完整步骤**，A/B 编译对照；旧二进制**尚无 `--out-dir`** → 走 `REPO_ROOT` 沙盒，新二进制走
`--out-dir`；两路都产出 png 后再 sha 对比）：
```bash
# 0) 变量
REPO=<repo-root>; OLD=/tmp/l4-old; OLDSB=/tmp/reg-old-sandbox; NEWOUT=/tmp/reg-new

# 1) 取改前提交的独立 worktree（不污染工作树）并构建旧二进制
git worktree add "$OLD" <change-commit>~1
( cd "$OLD" && cargo build -q -p game-desktop --bin render-frame )
cp "$OLD/target/debug/render-frame" /tmp/render-frame-old

# 2) 当前树（改后）构建
( cd "$REPO" && cargo build -q -p game-desktop --bin render-frame )

# 3) 旧二进制：REPO_ROOT 沙盒（旧版无 --out-dir，输出落沙盒 oracle/screens/actual）
rm -rf "$OLDSB"; mkdir -p "$OLDSB/oracle/screens/actual"
ln -sfn "$REPO/rom"      "$OLDSB/rom"
ln -sfn "$REPO/analysis" "$OLDSB/analysis"
REPO_ROOT="$OLDSB" /tmp/render-frame-old 4600 l4_clear-inject stg03-timeline

# 4) 新二进制：--out-dir（默认行为不变，但显式隔离输出）
rm -rf "$NEWOUT"; mkdir -p "$NEWOUT"
( cd "$REPO" && REPO_ROOT="$REPO" target/debug/render-frame \
    4600 l4_clear-inject stg03-timeline --out-dir "$NEWOUT" )

# 5) 两路 png sha 对比（A/B 判据；两文件都必须存在，缺一即 FAIL）
sha256sum "$OLDSB/oracle/screens/actual/l4_clear-inject.png" \
          "$NEWOUT/l4_clear-inject.png"
# 期望：两行 sha256 完全相同；不同 ⇒ P0-0-2 改坏了默认路径行为（BLOCKED，不得继续 §5）
```
**不得用现行 `actual/` 的 sha 作回归基准**（那是旧诊断态产物，见方案 §2.4 F-0-D）；带 `--out-dir` 时文件落在指定目录。

### 4.2 备选（无代码改动的先行路径）：`REPO_ROOT` 沙盒

`render_frame` 用 `REPO_ROOT` 定位 `rom/`、`analysis/`、`oracle/screens/actual`（A 级代码事实：`repo_root`
参与全部读写路径）。因此可建**只读输入 + 独立输出**的沙盒，从而在不改代码时做两跑隔离：

```bash
SB=/tmp/l4run1
mkdir -p "$SB/oracle/screens/actual"
ln -sfn "$PWD/rom"        "$SB/rom"
ln -sfn "$PWD/analysis"   "$SB/analysis"
cd <repo-root>
REPO_ROOT="$SB" cargo run -q -p game-desktop --bin render-frame -- 4600 l4_clear-inject stg03-timeline
ls "$SB/oracle/screens/actual/"     # → l4_clear-inject.png/.idx.json/.render.log
```
**注意**：①`create_dir_all(repo_root/oracle/screens/actual)` 会写沙盒（OK）；②不要 symlink 整个 `oracle/`
（会写回仓库）；③最终 canonical 产物仍需 `--out-dir`（4.1）或把沙盒 run1 产物复制回仓库（经 Lead 同意）。
**用途**：环境就绪但 4.1 未实现时的**先行**两跑验证/映射渲染；不替代 4.1 的长期方案。

### 4.3 BLOCKED

`--out-dir` 未实现且操作者无法用沙盒（权限/路径限制）⇒ P0-0 的 canonical 渲染步 **BLOCKED**（采集步可先行）。

---

## 5. 步骤 3：日志 + manifest（P0-0-3）

### 5.1 `oracle/screens/l4-manifest.json` schema（22 行 `pairs` + 头）

```json
{
  "generated_at": "2026-10-18T00:00:00Z",
  "env": {"os":"Linux x86_64","python":"3.x","mesen_sha256":"…","repo_head":"…","rom_sha256":"…"},
  "canonical": {"env": {}, "argv_template": ["cargo","run","-q","-p","game-desktop","--bin","render-frame","--"]},
  "pairs": [{
    "name": "l4_clear-inject", "scene": "stg03-timeline", "frame": 4600, "capture_tag": "stg02-clear-inject",
    "frame_offset": 0, "trigger": "natural",
    "expected_sha256": "…", "expected_source": "stg03-timeline:4600:stg02-clear-inject",
    "actual_sha256": "…", "actual_run1_sha256": "…", "actual_run2_sha256": "…",
    "input_sha256": {"frames":"…","probe":"…","text":"…"},
    "render_env": {}, "render_argv": ["4600","l4_clear-inject","stg03-timeline","--out-dir","…"],
    "status": "ok", "reuse": "independent", "duplicate_of": "", "deterministic": true, "notes": ""
  }],
  "dedupe": {
    "expected_unique": "22 − Σ(expected 重复组 len-1)",
    "actual_unique": "22 − Σ(actual 重复组 len-1)",
    "documented_duplicates": [
      {"side":"expected+actual","names":["l4_ending-4","l4_ending-5"],
       "sha256":{"expected":"…","actual":"…"},"frames":[34500,36000],"reason":"ending 静态页同图（重采后仍相同）","adjudicated_by":"…"}
    ]
  },
  "strict": {"cmd":"python tools/diff_visual.py --strict","exit":1,"line":"STRICT: FAIL diff=<n>","ran_after_asset_swap": true}
}
```
`status` 枚举：`ok` / `blocked:no-source` / `blocked:probe-cap` / `blocked:nondeterministic` /
`replaced-invalid-asset`（§7）/ `documented-duplicate`（§7.5）；status **从 mapping.tsv 透传**（M1）。
`reuse`：`independent|reuse`（按附录 A 逐对填，M1）。
> **F2/F8 口径**：`expected_unique`/`actual_unique` 是 **P0-0 完成后**的验收值，**不是** P0-0 前的
> 17/127（旧现状：expected 17/22、actual 目录 257 张/127 unique——actual 判据**必须按 manifest 22 名过滤**，
> 不得 glob 整目录）。**重复组按组核对**（B2）：`unique = 22 − Σ(len(group)−1)`；expected 与 actual 两侧的
> 重复都写进 `documented_duplicates`，且**必须与 mapping.duplicate_of 的组一致**；expected 重复也登记。

### 5.2 canonical 命令 + env 卫生（A 级关键）

**canonical = 生产默认路**：禁止任何诊断 env（方案 F-0-D：签署态曾用 TBLDRV/GOLDPAL/FAMMAP/FLAGUPLOAD/
ROWMAP 等诊断面）。`render_frame` 共 **39 个诊断 env + REPO_ROOT**（清单见附录 B）——canonical 必须全部
**unset**，只保留 `REPO_ROOT`（需要时）+ 显式 argv。命令模板：
```bash
env -u TBLDRV -u GOLDPAL -u FAMMAP -u FLAGUPLOAD -u ROWMAP -u FAMROW -u R4B -u COMP -u QUAD -u ... \
  REPO_ROOT="$PWD" cargo run -q -p game-desktop --bin render-frame -- \
  <frame> <name> <scene> --out-dir oracle/screens/actual_canonical --log-path oracle/screens/actual_canonical/.render.log
```
> 推荐用**干净 env**跑：`env -i PATH="$PATH" HOME="$HOME" REPO_ROOT="$PWD" cargo run …`（cargo 需 CARGO_HOME/RUSTUP_HOME
> 时一并保留）；wrapper 脚本内 `os.environ` 白名单实现（§5.3）。
> 生产默认值提醒：`--base` 缺省 0；`--probe-file/--stream-file/--text-file` 一律**不传**（render_frame 缺省
> 自动读 `analysis/traces/{scene}.probe.jsonl` + `analysis/phase/{scene}-text.jsonl`）；`--probe-file` 的
> 旧投影路是实验面，不属于 canonical。

### 5.3 wrapper 脚本规格 `tools/l4_render_all.py`（执行卡实现；**完整参考实现，非空跑骨架**；F3/S1）

> 前置：§7a 的 expected 已定稿（blob 校验 + 重采/裁定完成）。`mapping.tsv` = §3.5 schema（22 行）。
> 本脚本：读 mapping → 两跑渲染 → 填 §5.1 全字段 manifest → 去重（按 manifest 22 名过滤）→ 可选 `--strict` 记录。

```python
#!/usr/bin/env python3
"""L4 P0-0 canonical 渲染 + 两跑确定性 + 去重 + manifest（执行卡实现；写 tools/ 需 Lead 批准）。"""
import hashlib, json, os, pathlib, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
STATUS_OK = {"ok", "replaced-invalid-asset", "documented-duplicate"}
ALLOWED_ENV = {"PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME", "TMPDIR"}   # REPO_ROOT 显式注入
MAP = ROOT / "analysis/repro/mapping.tsv"
OUT1 = ROOT / "oracle/screens/actual_canonical_run1"
OUT2 = ROOT / "oracle/screens/actual_canonical_run2"
MANIFEST = ROOT / "oracle/screens/l4-manifest.json"

def sha(p):
    p = pathlib.Path(p)
    return hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None

def load_pairs():
    if not MAP.is_file():
        raise SystemExit(f"BLOCKED: {MAP} 不存在（先按 §3.5 生成）")
    lines = MAP.read_text(encoding="utf-8").splitlines()
    hdr = lines[0].split("\t"); rows = []
    REQUIRED = {"name","scene","frame","capture_tag","trigger","expected_source","status","reuse"}
    if not REQUIRED.issubset(set(hdr)):
        raise SystemExit(f"BLOCKED: mapping.tsv 缺列 {sorted(REQUIRED - set(hdr))}（M1）")
    for l in lines[1:]:
        if not l.strip(): continue
        rows.append(dict(zip(hdr, l.split("\t"))))
    if len(rows) != 22:
        raise SystemExit(f"BLOCKED: mapping.tsv 需 22 行，实际 {len(rows)}")
    if not rows:
        raise SystemExit("BLOCKED: PAIRS 为空")                      # 防空跑假绿
    return rows

def render(p, out_dir):
    name, scene, frame = p["name"], p["scene"], int(p["frame"])
    out_dir.mkdir(parents=True, exist_ok=True)
    env = {k: os.environ[k] for k in ALLOWED_ENV if k in os.environ}
    env["REPO_ROOT"] = str(ROOT)
    argv = ["cargo", "run", "-q", "-p", "game-desktop", "--bin", "render-frame", "--",
            str(frame), name, scene, "--out-dir", str(out_dir), "--log-path", str(out_dir/".render.log")]
    r = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"render FAIL {name} rc={r.returncode}\n{r.stderr[-2000:]}")
    png = out_dir/f"{name}.png"; idx = out_dir/f"{name}.idx.json"
    if not png.is_file() or not idx.is_file():
        raise SystemExit(f"render MISSING output {name}")
    return {"png": sha(png), "idx": sha(idx), "argv": argv}

def input_sha(scene):
    # title-menu 无 probe → "probe": null（合法；M7）
    return {"frames": sha(ROOT/f"analysis/traces/{scene}.frames.jsonl"),
            "probe":  sha(ROOT/f"analysis/traces/{scene}.probe.jsonl"),
            "text":   sha(ROOT/f"analysis/phase/{scene}-text.jsonl")}

def png_groups(base, names):
    g = {}
    for n in names:
        g.setdefault(sha(pathlib.Path(base)/f"{n}.png"), []).append(n)
    return {k: v for k, v in g.items() if k}      # 忽略缺失（缺失由 render/expected 检查兜底）

def dedupe(pairs, out1):
    names = [r["name"] for r in pairs]
    byname = {r["name"]: r for r in pairs}
    exp = png_groups(ROOT/"oracle/screens/expected", names)
    act = png_groups(out1, names)
    exp_dup = [v for v in exp.values() if len(v) > 1]
    act_dup = [v for v in act.values() if len(v) > 1]
    # 合并两侧同组成员：expected+actual / expected / actual（B2：expected 重复也登记）
    merged = []
    for v in exp_dup:
        merged.append(("expected+actual" if v in act_dup else "expected", v))
    for v in act_dup:
        if v not in exp_dup:
            merged.append(("actual", v))
    def dup_sha(side, v):
        e = sha(ROOT/"oracle/screens/expected"/f"{v[0]}.png")
        a = sha(out1/f"{v[0]}.png")
        if side == "expected": return e
        if side == "actual":   return a
        return {"expected": e, "actual": a}          # expected+actual（R2：按 side 取正确 sha）
    doc = [{"side": side, "names": v, "sha256": dup_sha(side, v),
            "frames": [int(byname[n]["frame"]) for n in v],
            "reason": "重采后同图（mapping.duplicate_of 组）", "adjudicated_by": "P0-0 §7"}
           for side, v in merged]
    exp_reduction = sum(len(g)-1 for g in exp_dup)
    act_reduction = sum(len(g)-1 for g in act_dup)
    # B2 判据：重复组必须与 mapping.duplicate_of 组逐组对应（组内名字应是某 mapping 组的子集）
    mgroups = {}
    for r in pairs:
        d = r.get("duplicate_of")
        if d: mgroups.setdefault(d, {d}).add(r["name"])
    mapping_sets = [frozenset(v) for v in mgroups.values()]
    registry_ok = all(any(frozenset(v) <= ms for ms in mapping_sets) for v in exp_dup + act_dup)
    dedupe_ok = (len(exp) == 22 - exp_reduction and len(act) == 22 - act_reduction
                 and len(exp) in (21, 22) and len(act) in (21, 22) and registry_ok)
    return {"expected_unique": len(exp), "expected_reduction": exp_reduction,
            "actual_unique": len(act), "actual_reduction": act_reduction,
            "documented_duplicates": doc}, dedupe_ok, registry_ok

def run_strict():
    # 只在 §8.1 步骤 2（actual 换新）之后运行；运行会重写 oracle/screens/report（§8.3/R8）
    r = subprocess.run([sys.executable, "tools/diff_visual.py", "--strict"],
                       cwd=ROOT, capture_output=True, text=True)
    line = [l for l in (r.stdout + r.stderr).splitlines() if l.startswith("STRICT:")]
    if not line: raise SystemExit("strict: 未捕获 STRICT 行")
    return {"cmd": "python tools/diff_visual.py --strict", "exit": r.returncode,
            "line": line[-1], "ran_after_asset_swap": True}

def main():
    rows = load_pairs()
    OUT1.mkdir(parents=True, exist_ok=True); OUT2.mkdir(parents=True, exist_ok=True)
    pairs = []
    for p in rows:
        a1 = render(p, OUT1); a2 = render(p, OUT2)
        ok = a1["png"] == a2["png"] and a1["idx"] == a2["idx"]
        pairs.append({
            "name": p["name"], "scene": p["scene"], "frame": int(p["frame"]),
            "capture_tag": p["capture_tag"], "frame_offset": int(p.get("frame_offset") or 0),
            "trigger": p["trigger"],
            "expected_sha256": sha(ROOT/f"oracle/screens/expected/{p['name']}.png"),
            "expected_source": p["expected_source"],
            "actual_sha256": a1["png"], "actual_run1_sha256": a1["png"], "actual_run2_sha256": a2["png"],
            "input_sha256": input_sha(p["scene"]),
            "render_env": {}, "render_argv": a1["argv"],
            # M1：status 从 mapping 透传；仅渲染不确定时降级
            "status": "blocked:nondeterministic" if not ok else (p.get("status") or "ok"),
            "reuse": p["reuse"], "duplicate_of": p.get("duplicate_of") or "",
            "deterministic": ok,
        })
    det = all(r["deterministic"] for r in pairs)
    dsum, dedupe_ok, registry_ok = dedupe(pairs, OUT1)
    env = {"os": f"{os.uname().sysname} {os.uname().machine}" if hasattr(os, "uname") else "unknown",
           "python": sys.version.split()[0],
           "mesen_sha256": sha(ROOT/"tools/mesen2/Mesen"),
           "rom_sha256": sha(ROOT/"rom/gunnac_cn.nes")}
    try:
        env["repo_head"] = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT,
                                          capture_output=True, text=True, check=True).stdout.strip()
    except Exception:
        env["repo_head"] = "unknown"
    manifest = {"generated_at": subprocess.run(["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], cwd=ROOT,
                                               capture_output=True, text=True).stdout.strip(),
                "env": env, "canonical": {"env": {}, "argv_template": ["cargo","run","-q","-p",
                "game-desktop","--bin","render-frame","--"]},
                "pairs": pairs, "dedupe": dsum, "strict": run_strict() if "--strict" in sys.argv else None,
                "two_run_deterministic": det}
    MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
                        encoding="utf-8", newline="\n")
    print(f"TWO-RUN: {'PASS' if det else 'FAIL'} ({len(pairs)} 对)")
    print(f"DEDUPE: expected {dsum['expected_unique']}=22-{dsum['expected_reduction']} "
          f"actual {dsum['actual_unique']}=22-{dsum['actual_reduction']} registry_ok={registry_ok} "
          f"-> {'PASS' if dedupe_ok else 'FAIL'}")
    print(f"MANIFEST: {MANIFEST}")
    return 0 if (det and dedupe_ok) else 1

if __name__ == "__main__":
    sys.exit(main())
```
**验收**：①`len(pairs)==22` 且 `TWO-RUN: PASS`（22 对 png+idx sha 逐对相等）；②`DEDUPE: ... PASS`
（按组公式：expected/actual 各自 `unique == 22 − Σ(len−1)`；expected 与 actual 的重复都登记在
`documented_duplicates`；`registry_ok=true` 即每个重复组非主帧都在 mapping 标了 `duplicate_of`）；
③manifest 逐字段符合 §5.1（`status`/`reuse` 来自 mapping 透传，M1）；④`--strict` 仅在 §8.1 步骤 2
（actual 换新）之后运行（否则 strict 比的是旧 actual，记录无意义）；其输出由 §8.1 步骤 5 回填。
**失败回退**：任一非 ok → 打印该 pair 并 exit 1（**绝不空跑 PASS**；`PAIRS` 空/≠22/缺列直接 BLOCKED）。

---

## 6. 步骤 4：两跑确定性（P0-0-4）

| 判据 | 命令 | 期望 |
|---|---|---|
| 输入层 | §2.3 脚本 | `DETERMINISM: PASS` |
| 渲染层 | §5.3 wrapper | `TWO-RUN: PASS`（22 对 png+idx sha 逐对相等；**PAIRS/mapping ≠22 或空 ⇒ 直接 BLOCKED，不得空跑 PASS**） |
| 去重层 | §5.3 wrapper / §7.5 脚本 | `DEDUPE: ... PASS`（两侧各自 `unique == 22 − Σ(len−1)`；expected+actual 重复均登记；`registry_ok=true`） |
| 交叉 | `cmp` run1/run2 的 `.render.log` 语义字段（去掉时间） | env/argv/输入 sha 行一致 |

**失败处置**：单对 DIFF ⇒ ①查 env 泄漏（wrapper 打印实际 env）；②查输入 sha 是否变（应同）；③查是否并行；
仍 DIFF ⇒ 该对 `blocked:nondeterministic`，P0-0 **不得宣告冻结**（冻结基线必须全部确定性）。
**回退**：保留 run1/run2 目录、manifest 与日志，上报 Lead；不覆盖现行 `actual/`。

---

## 7. 步骤 5：资产裁定（P0-0-5 / P0-0b）

> **【2026-10-18 Lead 裁定·分工】** P0-0 **含资产裁定与真值重采/去重**（本节全部）；**P0-4 保留**
> "重渲后按新基线重判 / 自然链更优真值替换"——本手册按任务卡优先，不拆分（对应 Lead 口径确认 1）。
>
> 来自方案 §2.4/§4.3 的"10/22 需重采/重渲"：stg02 1 + 四联冻结 4 + ending 5。本步只做**资产真值**裁定；
> 真正的 0 容差修复属于 P0-1..P0-4。
> **B1 唯一安装点**：§7.1-§7.4 产出新图后，**统一由 §7.7 安装到 `oracle/screens/expected/` 并归档旧件**；
> **所有 expected 替换必须在 §5 wrapper 之前完成**；mapping.tsv 的 `status/expected_source/capture_tag/
> duplicate_of` 由 **§7.7 Step 0** 按 §7a 裁定写回定稿（R1；不再只是口头"随安装定稿"）。
> §7b（§7.3 的 actual 重渲部分、§7.5 去重核对）在 §5 之后，只核对**不再改 expected**。

### 7.1 stg02-start（#14）：expected 无效 → 重采真值

- **现状（A）**：expected == `title-menu_f00180_boot-black` 同 blob（全黑 258B 复制品）；actual 为关名屏
  （`.render.log` frame=3404、scroll=(0,208)）。
- **动作（B5 修正）**：用源 `stg03-timeline` **已存在的** `capture 3404 stg02-start` 产物
  `shots/stg03-timeline_f03404_stg02-start.png`（**不是** I3b overlay——I3b 已按 M2 取消）；
  若该 shot 不存在/异常，**临时重建 stg03 overlay**（§3.2 `make(..., captures=[3404])`，仅加 capture 3404）作为
  fallback（不默认执行）；±1 帧按 Q4 备查（R5：已去掉"I3b"措辞）。
  语义校验 = 画面含关名屏内容（非全黑；文字/关名区域非零；与 actual 的关名屏结构同域，**不与 actual 直接相等**——
  actual 是移植渲染，expected 是 oracle 真值）。
- **裁定**：staging 到 `analysis/expected-new/l4_stg02-start.png`（§7.7 安装），`expected_source` 记字符串
  **`stg03-timeline:3404:stg02-start`**（M5）；`status=replaced-invalid-asset`；**`l4-visual-plan.md:349-350@1be66e5`
  的"902=文本内容本身差"由 spec-writer dated 更正（F6：该表述不在 `l4-signing-criteria` §4.5——§4.5 是
  fallback 不适用域）**。

### 7.2 四联冻结帧（#16/#19/#20/#21）：expected 为同一 blob 复制 → 按标签重采真值

> **【2026-10-18 更正·L5 交叉核对】选帧来源已更正（§3.7）**：原"v5 36500f 截断 + stage_no==6/8 选帧"
> 前提失效（v5 到 f73939 才 stage4-combat，36500f 仍在关 2）。**新来源**：
> `l4_stg08-start/combat/late` ← `tas-stg08b2-direct`（stage8 直入、40000f、源含 10 probe）；
> `l4_stg06-late` ← `stg06-timeline`（自关窗 f13000-13977、源 **0 probe** → I6b pass2 补 8）。
> 选帧脚本/pass2 生成/重跑命令统一见 **§3.7**（本节的旧 v5 脚本/命令作废，保留留档见 §3.7 原假设）。

- **现状**：四张 expected **同 blob（sha 事实 A）**，6,074B；"该屏=商店冻结屏"是 **B 级解释**（OBS-1 末关
  出店截断，`docs/bug-ledger.md` OBS-1），不作 A 级断言（F15）。`stg08-timeline` 在商店处冻结、后续
  capture 均为同屏。
- **动作（F5：两遍机制——capture 只能对预置帧出图，不能事后补；来源=§3.7）**：
  - **第一遍（选帧）**：I5 `tas-stg08b2-direct` + I6 `stg06-timeline` 已有 `frames.jsonl` → 按 §3.7 选帧脚本
    写 `analysis/repro/l4-selected-frames.json`（含 `scene`/`frame` 字段）；
  - **第二遍（capture）**：按 §3.7 生成 `l4-stg08b2-pass2` 与 `l4-stg06-pass2`（§3.2 `make(..., captures=[...])`）
    → 重跑 → 取 `shots/<pass2-scene>_f<frame>_l4f<frame>.png` 作为新 expected 源图；随后 phase-batch
    两个 pass2 场景（M4）。
- **裁定**：staging 到 `analysis/expected-new/<name>.png`（§7.7 安装）；`status=replaced-invalid-asset`；
  `trigger=injected`（两源均含 poke——`tas-stg08b2-direct` 的 `0180` 选关/assist、`stg06-timeline` 的前关注入链；
  README 纪律：poke run 不得冒充 natural）；`expected_source=<pass2-scene>:<frame>:l4f<frame>`；
  若自然链（八关 bot 自然通关，未就绪）后补产出更优真值，另卡替换。

> **旧动作留档（2026-10-18 更正前，作废）**：~~用 `l4-baseline-v5`(36500f) + pass2 按 stage_no==6/8 选帧~~；
> 作废原因与反证见 §3.7。~~`trigger=assisted`~~ 同上，改按实际来源登记。

### 7.3 ending 五帧（#6-#10）：actual 同 blob（TBLDRV 诊断态遗留）→ 逐帧独立重渲

> **【2026-10-18 更正·组 D】** 原动作"用 `l4-baseline-v5_f*_probe.png` 比对"的前提（v5 覆盖
> f28500-36000=ending）**不成立**（v5 该段=关 2 战斗，§3.6 A 级证据）⇒ 本节 §7a 改为：
> **先用 §3.6 重裁选中来源**（候选 C1 `tas-stg08b2-direct` 优先），再用其目标帧 capture 与旧 expected 比对。
> 重裁未命中 ⇒ 这 5 对 `blocked:no-source`（不 staging）；原 v5 动作仅作负候选对照留档（预期 blob 不等）。
> 旧动作留档：~~用 I5 的 capture_every 产物比对~~。

**§7a（§5 之前，expected 侧）**：
- **现状（A）**：五张 actual 同 blob（TBLDRV=1 共用 ending-1 golden 表）；expected-4/5 同文件。
- **动作（重裁后）**：用 **§3.6 命中来源**在 `{30000,31500,33000,34500,36000}` 的 capture 与旧 expected-1..5
  逐一做 sha256 比对（同时核该帧 mode=Ending/fade，§3.6 判据②）：
  - 全部相等 ⇒ 映射确认（A），**expected 不动**（`status=ok`）；
  - ending-4/5 的重采仍相同 ⇒ **裁定 documented duplicate**：staging 保留原两份（或合并一份+manifest 注记，
    由 Lead 裁），`status=documented-duplicate`、mapping `duplicate_of` 标注从属名；
  - 若不等 ⇒ 用新 capture 替换对应帧并记 dated，staging 到 `analysis/expected-new/`（§7.7 安装）。
  **该步必须在 §5 之前完成**（B1：manifest 的 expected_sha 才不会是旧值）。
**§7b（§5 之后，actual 侧）**：canonical 渲染后核对五帧 actual 两两不同 blob；若存在
documented duplicate（静态同图），必须已在 manifest `dedupe.documented_duplicates` 登记（sha/帧/理由）——
与 §7.5/§8.2 的"22−Σ(len−1)"口径一致（F8/B2）；manifest 记 `expected_source` 帧号。

### 7.4 fade-clear（#11）：来源场景待证

> **【2026-10-18 更正·组 D】** 原"用 `l4-baseline-v5_f28500_probe.png` 比对"前提同 §7.3 作废；
> **先用 §3.6 重裁选中来源**的 28500 capture 比对（候选 C1/C2 优先；v5 作负候选，预期不等）。

**§7a**：用 **§3.6 命中来源**的 f28500 capture 与旧 expected sha 比对 → 相等即映射确认（expected 不动）；
不等/无来源 ⇒ 按 §3.6 走 `blocked:no-source` + 新采集 `s5-full-inject`（命中后再 staging 到
`analysis/expected-new/`，`status=replaced-invalid-asset`，记录 run/帧/trigger）。**须在 §5 前完成安装**（B1）。

### 7.5 去重判据与登记（**F2/F8/B2：按 manifest 22 名过滤 + 按组核对 + 重复 ⊆ manifest**）

> 现状背景（P0-0 前）：expected unique **17/22**；`oracle/screens/actual/` 有 **257 张实验 PNG/127 unique**
> ——所以任何"glob actual 目录"的脚本都会误判（F2）。判据一律**只用 manifest 的 22 个 name**。

```bash
python - <<'PY'
import hashlib, json, pathlib, collections
M=json.loads(pathlib.Path("oracle/screens/l4-manifest.json").read_text(encoding="utf-8"))
names=[p["name"] for p in M["pairs"]]
def group(base):
    g=collections.defaultdict(list)
    for n in names:
        p=pathlib.Path(base)/f"{n}.png"
        assert p.is_file(), f"missing {p}"
        g[hashlib.sha256(p.read_bytes()).hexdigest()].append(n)
    return g
E=group("oracle/screens/expected"); A=group("oracle/screens/actual_canonical_run1")
Edup=[v for v in E.values() if len(v)>1]; Adup=[v for v in A.values() if len(v)>1]
print("expected unique", len(E), "| actual unique", len(A))
rows=[]
def report(side, dup, base):
    for v in dup:
        h=hashlib.sha256((pathlib.Path(base)/f"{v[0]}.png").read_bytes()).hexdigest()
        print("  DUP",side,h[:12],v); rows.append(f"{side}\t{h[:12]}\t{','.join(v)}")
report("expected",Edup,"oracle/screens/expected")
report("actual",Adup,"oracle/screens/actual_canonical_run1")
pathlib.Path("analysis/repro/dedupe.tsv").write_text("side\tsha12\tnames\n"+"\n".join(rows)+"\n",
                                                    encoding="utf-8", newline="\n")
# B2 判据：unique == 22 − Σ(len(group)−1)；且所有重复 ⊆ manifest.documented_duplicates
doc=M["dedupe"]["documented_duplicates"]
doc_sets=[set(d["names"]) for d in doc]
def covered(groups): return all(any(set(v)==s for s in doc_sets) for v in groups)
ok = (len(E)==22-sum(len(v)-1 for v in Edup) and len(A)==22-sum(len(v)-1 for v in Adup)
      and len(E) in (21,22) and len(A) in (21,22)          # R3：expected/actual 范围断言
      and covered(Edup) and covered(Adup))
print("DEDUPE:", "PASS" if ok else "FAIL", "| doc_groups=",len(doc_sets))
PY
```
**判据（P0-0 后口径，B2/F8）**：①expected `unique == 22 − Σ(expected 重复组 len−1)`；②actual
`unique == 22 − Σ(actual 重复组 len−1)`；③**所有重复组（expected 与 actual 两侧）都必须能在
manifest `dedupe.documented_duplicates` 找到同 name 集**（expected 重复也登记，不能只在 actual 侧登记）；
④每组"非主帧"成员在 mapping.tsv 标 `duplicate_of`（wrapper 的 `registry_ok` 会再核一次）。
未登记/公式不符 ⇒ `blocked` 排查。所有 **DUP**（非 DUO）写入该表与 manifest（F17）。

### 7.6 trigger 标注纪律

- 自然场景（title-menu/gameplay-stage1/stg03-timeline/stg08-timeline 无 poke）→ `trigger=natural`；
- v5 overlay（含 `poke 1255 01be/01b6/01b7 01` = assist）→ `trigger=assisted`（等价 injected 族，不得冒充自然）；
- 选关/注入帧（若用 `poke 0180`）→ `trigger=injected`。

### 7.7 expected 唯一安装流程（B1/B3/M6：**只在 §7a 执行一次**，必须先于 §5）

> 输入 = §7.1-§7.4 产出的 staging 图（`analysis/expected-new/<name>.png`）；输出 = 
> `oracle/screens/expected/<name>.png`（旧件归档到 `.archive/<STAMP>/`）。**§8.1 步骤 3 只核验，不再执行本流程。**

**Step 0 — 按 §7.1-§7.4 裁定写回 `analysis/repro/mapping.tsv`（R1；必做，先于下面 A）**：
§3.5 生成 mapping.tsv 时写的是**初值**；§7a 的裁定（是否替换/是否 documented duplicate/来源帧）必须先写回，
否则下面 A 的 conditional staging 与 §5 wrapper 读到的 `status/expected_source/duplicate_of` 仍是旧值。
规则：`analysis/repro/mapping-decisions.json`（**裁定台账**，由操作者按 §7.1-§7.4 结果填写，只写发生变化的行；
键=`mapping.tsv` 列名；值一律字符串；**禁止改 `name/scene/frame`**——定位列以 §3.3 校验为准）：
```json
{
  "l4_stg02-start": {"status":"replaced-invalid-asset","expected_source":"stg03-timeline:3404:stg02-start","capture_tag":"stg02-start"},
  "l4_stg06-late":  {"status":"replaced-invalid-asset","expected_source":"l4-stg06-pass2:<f>:l4f<f>","capture_tag":"l4f<f>","trigger":"injected"},
  "l4_ending-4":    {"status":"documented-duplicate","duplicate_of":"l4_ending-5"},
  "l4_fade-clear":  {"status":"replaced-invalid-asset","expected_source":"<winner>:28500:<tag>","capture_tag":"<tag>","trigger":"injected/assisted（按 §3.6 重裁来源如实登记）"}
}
```
> **F49-1/F49-2 更正**：`l4_stg06-late` 源已改 `l4-stg06-pass2`（stg06 源 0 probe → pass2 补 8）；
> `l4_fade-clear` 源待 §3.6 重裁（`<winner>` 为 C1-C5 命中 run），**不得再填 `l4-baseline-v5*`**。
```bash
python - <<'PY'
import json, pathlib
mp=pathlib.Path("analysis/repro/mapping.tsv")
dec=json.loads(pathlib.Path("analysis/repro/mapping-decisions.json").read_text(encoding="utf-8"))
lines=mp.read_text(encoding="utf-8").splitlines(); hdr=lines[0].split("\t")
assert {"status","duplicate_of","expected_source","capture_tag"} <= set(hdr), "mapping.tsv 缺 R1 列（§3.5）"
rows=[dict(zip(hdr,l.split("\t"))) for l in lines[1:] if l.strip()]
assert len(rows)==22, f"mapping.tsv 需 22 行，实际 {len(rows)}"
ALLOWED={"ok","replaced-invalid-asset","documented-duplicate","blocked:no-source","blocked:probe-cap","blocked:nondeterministic"}
for name,d in dec.items():
    r=next((x for x in rows if x["name"]==name), None)
    assert r is not None, f"mapping 无此 name: {name}"
    for k,v in d.items():
        assert k in hdr, f"{name}: 未知列 {k}"
        assert k not in ("name","scene","frame"), f"{name}: 禁改定位列 {k}"
        r[k]=str(v)
    assert r["status"] in ALLOWED, f"{name}: status={r['status']} 非法"
    if r.get("duplicate_of"): assert r["status"]=="documented-duplicate", f"{name}: duplicate_of 但 status≠documented-duplicate"
out=[lines[0]]+["\t".join(r[k] for k in hdr) for r in rows]
mp.write_text("\n".join(out)+"\n", encoding="utf-8", newline="\n")
print("mapping.tsv 写回 OK：", {n:dec[n].get("status") for n in dec})
PY
```
**判据**：写回后 `grep -c 'replaced-invalid-asset' mapping.tsv` = §7.1-§7.4 实际替换数（≥1）；
若 §7.3 裁 documented duplicate 则 `grep -c 'documented-duplicate'` ≥1 且两帧名字互指；
下面 A 的 conditional staging 命中项与 §5 wrapper manifest 的 `status/duplicate_of` **必须非 §3.5 初值**。

```bash
STAMP=$(date +%Y%m%d)
mkdir -p analysis/expected-new oracle/screens/expected/.archive/$STAMP

# ── A) 四类资产 staging（源图来自 §7.1-§7.4；缺失按 mapping status 跳过）──────────
python - <<'PY'
import json, pathlib, shutil
NEW=pathlib.Path("analysis/expected-new"); NEW.mkdir(parents=True, exist_ok=True)
mp=pathlib.Path("analysis/repro/mapping.tsv")
rows=[dict(zip(mp.read_text(encoding="utf-8").splitlines()[0].split("\t"), l.split("\t")))
      for l in mp.read_text(encoding="utf-8").splitlines()[1:] if l.strip()]
MP={r["name"]:r for r in rows}
def stage(name, src):
    s=pathlib.Path(src)
    assert s.is_file(), f"缺源图 {src}（先按 §7.1-§7.4 采集）"
    shutil.copy2(s, NEW/f"{name}.png")
# 1) stg02（#14，B5：源 stg03 显式 capture）
stage("l4_stg02-start", "shots/stg03-timeline_f03404_stg02-start.png")
# 2) 四联（#16/#19/#20/#21，§3.7/§7.2 第二遍 pass2 选帧；sel 含 scene 字段）
sel=json.loads(pathlib.Path("analysis/repro/l4-selected-frames.json").read_text(encoding="utf-8"))
for name, d in sel.items():
    f=int(d["frame"]); scene=d["scene"]
    stage(name, f"shots/{scene}_f{f:05d}_l4f{f}.png")
# 3-5) 组 D（fade/ending）：源图必须来自 §3.6 重裁命中来源（analysis/repro/l4-groupD-source.json）；
#      未命中/未决 ⇒ 不 staging（保留旧 expected，mapping 保持 blocked:no-source，manifest 对应行 blocked）
gd=json.loads(pathlib.Path("analysis/repro/l4-groupD-source.json").read_text(encoding="utf-8"))
SRC=gd["captures"]          # {"l4_ending-1": "shots/<winner_scene>_f<帧>_<tag>.png", ...}
for name in ("l4_ending-1","l4_ending-2","l4_ending-3","l4_ending-4","l4_ending-5","l4_fade-clear"):
    if MP[name]["status"]=="replaced-invalid-asset":
        assert name in SRC, f"{name}: 需替换但 §3.6 未给出源图（先完成组 D 重裁）"
        stage(name, SRC[name])
# documented-duplicate（ending-4/5 同图裁定）→ 不 staging，保留旧件 + mapping duplicate_of
print("staged:", sorted(p.name for p in NEW.glob("*.png")))
PY

# ── B) 校验 + 安装（归档旧件 → 落新件；staging 缺项按 status/duplicate_of 跳过）────
python - <<'PY'
import datetime, hashlib, pathlib, shutil
NEW=pathlib.Path("analysis/expected-new"); OLD=pathlib.Path("oracle/screens/expected")
STAMP=datetime.date.today().strftime("%Y%m%d")
ARCH=OLD/".archive"/STAMP; ARCH.mkdir(parents=True, exist_ok=True)
mp=pathlib.Path("analysis/repro/mapping.tsv")
rows=[dict(zip(mp.read_text(encoding="utf-8").splitlines()[0].split("\t"), l.split("\t")))
      for l in mp.read_text(encoding="utf-8").splitlines()[1:] if l.strip()]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
changed=[]
for r in rows:
    n=r["name"]; src=NEW/f"{n}.png"
    if not src.is_file():
        assert r["status"]=="documented-duplicate" or r["duplicate_of"] or r["status"]=="ok", \
            f"{n}: 需 staging 但 analysis/expected-new 缺图（status={r['status']}）"
        continue                                    # 保留旧件（映射确认不动 / documented duplicate）
    old=OLD/f"{n}.png"
    if old.is_file(): shutil.copy2(old, ARCH/f"{n}.png")       # 归档旧件
    shutil.copy2(src, old)                                     # 落新件（唯一安装点）
    changed.append((n, sha(src)[:12]))
print("installed:", changed)
print("skipped(映射确认不动/dup):", [r["name"] for r in rows if not (NEW/f"{r['name']}.png").is_file()])
PY
# ── C) 安装后自检（只读；必须通过，才允许进入 §5）──────────────────────────
python - <<'PY'
import hashlib, pathlib
OLD=pathlib.Path("oracle/screens/expected")
miss=[n for n in ["l4_stg02-start","l4_stg06-late","l4_stg08-start","l4_stg08-combat","l4_stg08-late"]
      if not (OLD/f"{n}.png").is_file()]
assert not miss, f"expected 缺文件: {miss}"
print("expected 安装自检: OK；样本 sha12:",
      {n: hashlib.sha256((OLD/f"{n}.png").read_bytes()).hexdigest()[:12]
       for n in ["l4_stg02-start","l4_stg08-start"]})
PY
```
**判据**：①所有 `status in {replaced-invalid-asset, documented-duplicate}` 的图在
`analysis/expected-new/` 有对应 staging（或 documented duplicate 显式跳过）；②安装后 5 个重采样本存在；
③安装完成前**不得**进入 §5；④§8.1 只核验不复制（B1）。

---

## 8. 步骤 6：新基线冻结（P0-0-6）

### 8.1 产物替换/归档（F13：具体命令与顺序）

```bash
STAMP=$(date +%Y%m%d)
mkdir -p oracle/screens/actual/.archive/$STAMP oracle/screens/expected/.archive/$STAMP
# 1) 旧 actual 归档：只归档 22 对（按 manifest 名单），实验图不动
python - <<PY
import json, pathlib, shutil
M=json.loads(pathlib.Path("oracle/screens/l4-manifest.json").read_text(encoding="utf-8"))
dst=pathlib.Path("oracle/screens/actual/.archive/$STAMP")
for p in M["pairs"]:
    for ext in (".png",".idx.json"):
        src=pathlib.Path("oracle/screens/actual")/(p["name"]+ext)
        if src.is_file(): shutil.copy2(src, dst/src.name)
print("archived", len(list(dst.iterdir())))
PY
git ls-files oracle/screens/actual/*.png | head -3     # 确认旧文件在 git 中（历史可回溯）

# 2) canonical run1 → actual/（仅 22 对；实验图保持原样）
python - <<PY
import json, pathlib, shutil
M=json.loads(pathlib.Path("oracle/screens/l4-manifest.json").read_text(encoding="utf-8"))
src=pathlib.Path("oracle/screens/actual_canonical_run1"); dst=pathlib.Path("oracle/screens/actual")
for p in M["pairs"]:
    for ext in (".png",".idx.json"):
        shutil.copy2(src/(p["name"]+ext), dst/(p["name"]+ext))
print("installed", len(M["pairs"]))
PY

# 3) expected 安装：**B1 —— §7a 已在 §5 之前完成安装（§7.7）**；此处只核验，不再复制/归档！
python - <<'PY'
import json, hashlib, pathlib
M=json.loads(pathlib.Path("oracle/screens/l4-manifest.json").read_text(encoding="utf-8"))
E=pathlib.Path("oracle/screens/expected"); bad=[]
for p in M["pairs"]:
    f=E/f"{p['name']}.png"
    if not f.is_file(): bad.append((p["name"],"missing")); continue
    h=hashlib.sha256(f.read_bytes()).hexdigest()
    if h!=p["expected_sha256"]: bad.append((p["name"],"sha!=manifest"))
assert not bad, f"expected 安装/ manifest 不一致（B1）: {bad}"
print("expected 核验 OK:", len(M["pairs"]), "对与 manifest expected_sha256 一致")
PY

# 4) 重新生成 diff/report（注意 §8.3 R8 交互）；--strict 在 actual 已换新后跑（B3）
python tools/diff_visual.py          > analysis/repro/visual-default.log 2>&1; echo "default exit=$?"
python tools/diff_visual.py --strict > analysis/repro/visual-strict.log 2>&1; echo "strict exit=$?"

# 5) strict 记录回填 manifest.strict（B3：资产替换后执行，命令完整）
python - <<'PY'
import json, pathlib
mp=pathlib.Path("oracle/screens/l4-manifest.json")
M=json.loads(mp.read_text(encoding="utf-8"))
log=pathlib.Path("analysis/repro/visual-strict.log").read_text(encoding="utf-8", errors="replace").splitlines()
line=[l for l in log if l.startswith("STRICT:")]
assert line, "strict 日志无 STRICT 行（检查 diff_visual.py --strict 是否运行）"
exit_code=1 if line[-1].startswith("STRICT: FAIL") else 0
M["strict"]={"cmd":"python tools/diff_visual.py --strict","exit":exit_code,
             "line":line[-1],"ran_after_asset_swap":True}
mp.write_text(json.dumps(M, ensure_ascii=False, indent=2)+"\n", encoding="utf-8", newline="\n")
print("strict 回填:", M["strict"])
PY
```
**注意**：`diff_visual.py` 会重写 `oracle/screens/visual-report.md`、`diff/*.png`、`whitelist-application.md`
（A 级代码事实）——属**有意产物更新**，与 R8（报告模板残留 W-C1 文案，归 task-20/21）协调后再提交；
`--strict` 预期 **exit 1 / `STRICT: FAIL diff=<n>`**（P0-0 不要求清零）。

### 8.2 基线验收清单（全过才算 P0-0 完成）

- [ ] **环境接收（§1）已完成并记录**：Windows headless 冒烟 title-menu = 1080 frames/1080 trace/37 shots、
      两跑 trace sha 相同、37/37 shots 逐字节相同；`gates status` PASS=11/FAIL=0/SKIP=1 且 `gates rom` PASS
      （记录进 `analysis/p0-0-env.txt`——**该文件由 P0-0-0 执行时生成**、gitignored，内容清单见 §1.3；
      **执行前不存在 ≠ 环境未接收**；E3 Lua IO 配置已生效）；
- [ ] manifest 22 行齐全：`scene/frame/capture_tag/trigger/input_sha256/render_env/render_argv/expected_sha256/actual_sha256/status/reuse/duplicate_of`；
- [ ] `two_run_deterministic=true`（22/22）；输入层 `DETERMINISM: PASS`；
- [ ] §7 资产裁定全部落盘（5 类：stg02 1、四联 4、ending 5、fade 1、documented duplicate 判据）；
      **expected 安装由 §7.7 在 §5 前完成**，§8.1 步骤 3 核验 `expected_sha256` 全等（B1）；
- [ ] **§3.6 组 D 重裁完成**（`analysis/repro/l4-groupD-source.json` 存在且 winner 非空/或 6 对显式
      `blocked:no-source`）；ending-4/5/fade 的 expected 必须来自重裁命中来源，**不是 v5**；
      未完成 ⇒ 本清单不通过（P0-0 不得宣称全绿）；
- [ ] **§3.7 四联来源更正落地**：四联 expected 来自 `l4-stg08b2-pass2`/`l4-stg06-pass2` 的选帧 capture
      （`l4-selected-frames.json` 含 `scene`/`frame`；`expected_source` 指向 pass2 场景），**不是**
      `l4-baseline-v5-pass2`(36500f)；I1-I6 采集与两跑集合已按 §2.3 更新；
- [ ] expected/actual 去重符合 §7.5 **B2 后口径**：两侧各自 `unique == 22 − Σ(len(group)−1)`；
      **expected 与 actual 的重复都**登记在 manifest `dedupe.documented_duplicates`，且每组非主帧在 mapping
      标 `duplicate_of`（wrapper `registry_ok=true`；**不是**旧账面 17/127）；
- [ ] `visual-report.md` 重生成；`--strict` 输出行由 §8.1 步骤 5 回填 `manifest.strict`
      （FAIL 可接受，`ran_after_asset_swap=true`；B3）；
- [ ] 方案 §2.1/§2.6 的旧数字**dated 更正**（新基线数值替换；旧主张划线保留，AGENTS §3.6）；
- [ ] 产物路径齐：`oracle/screens/{expected,actual,diff}`、`oracle/screens/l4-manifest.json`、`analysis/repro/*`；
- [ ] Lead 复核 + 独立对抗复核 pass。

### 8.3 与 R8 / 门禁的交互（引用复核 §7）

- **R8**（报告模板 W-C1 残留）未修前，重新生成 `visual-report.md` 会复述旧 W-C1 候选文案——建议：
  ①先由 task-20/21 同步报告模板（tools 生成物，需 Lead 批准），再跑 baseline 报告；或
  ②P0-0 记录中只引用 `--strict` 输出行，不把旧模板报告当结论（并在 commit 说明该残留）。
- **门禁**：P0-0 完成后 `gates/cli.py visual` 仍走**默认模式**（分类账口径，方案 §5.6 R1 两步走）；
  **默认翻转留到 P0-6 全绿后**——P0-0 不得翻默认（否则 pre-commit 恒红阻断 P0-1..）。
- `gates/visual_whitelist.json` 保持不存在（用户已裁定撤销 W-C1/W-C4）。

### 8.4 提交与回退

- 提交拆分（Lead/执行方）：①overlay 场景 DSL（`oracle/scenarios/l4-stg08b2-pass2.txt`/`l4-stg06-pass2.txt`，
  旧 `l4-baseline-v5*.txt` 不提交/仅负候选）独立 commit；
  ②`--out-dir/--log-path` 实现独立 commit（crates）；③`tools/l4_render_all.py`（tools）独立 commit；
  ④新基线资产+manifest+report（oracle/screens）+ 方案数字 dated 更正（docs）单独 commit。
- **回退**：旧 actual/expected 均在 git 历史中（必要时 `.archive/` 副本）；基线 commit 可 revert；
  回退后 manifest 标 `superseded`。
- **冻结标记**：commit message 含 `P0-0 baseline freeze` + Evidence（manifest sha + 两跑 sha）+ Confidence；
  是否打 git tag 由 Lead 裁定（建议 `l4-baseline-<date>`）。

---

## 9. Windows 本机（无 Mesen2）可做/不可做边界 + 降级路径

### 9.1 可做（只读/文档，不产生验收数字）

| 项 | 产物 |
|---|---|
| 22 对签名/映射推断（§3.1 表） | 本文档 §3.1；`analysis/repro/mapping.tsv` 模板 |
| manifest schema + 模板（22 行空壳，旧 sha 预填） | §5.1 + 本文件附录 A 数据 |
| overlay 场景 DSL 生成规则/脚本（§3.2） | 本文档代码块（执行卡落盘） |
| `--out-dir/--log-path` 代码 diff 规格（§4.1） | 本文档 + 交 porter 执行卡 |
| wrapper 源码（§5.3） | 本文档代码块 |
| fixtures 入库清单/体积估算（§2.5） | 本文档 |
| 只读复算 C1-C11（现有基线现状） | 方案 §7；不改基线 |

### 9.2 不可做（需 Mesen2 实机采集；Windows headless 主路径）

> 【2026-10-18 环境更正】以下不再「不可做」——Windows headless 环境已就绪（§1）：`scenario_run.py` 采集、
> `phase-batch`、`render-frame`、oracle 截图/RAM/probe、两跑确定性、`diff_visual` 重生成、资产重采均可执行；
> 仍受 E10 串行纪律约束（同一时刻一个 Mesen）。

`scenario_run.py` 采集、`phase-batch`、`render-frame`（无 `analysis/` 输入）、oracle 截图/RAM/probe、
两跑确定性、`diff_visual` 重生成（会写 oracle/screens）、资产重采。

### 9.3 降级路径（环境延迟时）

- **降级 A（推荐）**：只做 **12/22 独立对的数据面预研**——P0-1 写流投影的**代码面**设计/静态反查/纯函数单测
  （`render_rules` 等可脱离输入测试的部分），**不验收、不代入 diff 数字**；判据等 P0-0 后重跑。
  （注：任务卡旧文"17/22"已被 S1 修正为 **12/22 独立对**，以方案 §0/§4.3 为准。）
- **降级 B**：把 §3.2 overlay 场景与 §5.3 wrapper **写好并提交评审**，环境就绪后可一键执行。
- **禁止**：把"代码预研完成/静态推理合理"当 P0-0 或 P0-1 达成；在 P0-0 未完成时以旧 22 对数字作任何验收。

---

## 10. 需用户/Lead 裁定事项

1. ~~**Linux+Mesen2 入口**：提供方式（机器/容器/镜像 URL）、Mesen2 2.1.1 来源、可写入工作目录、最长保留时间；~~
   **【2026-10-18 已解决】** 环境改走 **Windows 宿主 + Mesen2 2.1.1 Win x64 headless**（§1；用户已提供
   `tools/mesen2/Mesen.exe` + settings 配置）；WSL2 Linux 版 Mesen2 `std::bad_cast` 不可用，仅备选。
2. **fixtures 入库政策**（§2.5）：是否允许 `oracle/l4-fixtures/**` 入库、体积上限（建议 ≤200MB）、脱敏/来源说明；
3. **样本集重构**：stg02/四联/ending 的 expected 替换与标签重定义（会改变 L4 样本面，方案 §6.2-5）；
4. **assist/injected 采集允许性**：**组 D 重裁候选来源**（§3.6 C1-C5：`tas-stg08b2-direct` 等选关/assist run）
   与选关注入是否可作 P0-0 真值（需登记 `trigger=injected/assisted`，不得冒充 natural）；若均不可接受，
   批准新采集 `s5-full-inject`（§3.6 C5）；
5. **基线提交/tag**：`l4-manifest.json` + 新 expected/actual 是否入库；是否打 `l4-baseline-<date>` tag；
6. **R8 协调**：`visual-report.md` 模板 W-C1 文案由 task-20/21 先修还是 P0-0 接受残留；
7. **资源**：磁盘/RAM 不足时的裁剪方案（是否允许缩短长跑/只做独立对）。

## 11. 工具/路径事实核验（本手册引用的 A/B 级依据）

| 事实 | 来源（只读核验） |
|---|---|
| **Mesen2 2.1.1 Win x64** `tools/mesen2/Mesen.exe`（gitignore 可再生；**Windows headless 主路径**）；Linux 版 `Mesen` 仅备选（bad_cast 不可用） | `tools/scenario_run.py`（MESEN/HEADLESS 判定）/`docs/methodology.md`/`docs/ai-decompile-pipeline.md` |
| 串行锁/清场：Windows=msvcrt 1-byte 字节锁（无孤儿分支）；POSIX 备选=flock + `pgrep` 孤儿清扫 + 自管 Xvfb（t24 弃 xvfb-run 包装层） | `tools/scenario_run.py`（lock、`pgrep`、`archive_existing`、`Xvfb`+`DISPLAY`；docstring 的 xvfb-run=历史文案） |
| 输出：`analysis/traces/<name>.{frames,captures,run}.jsonl/json`、`shots/<name>_f%05d_<tag>.png`、`.probe.jsonl`、`.done` | `tools/scenario_run.py`；`tools/scenario_capture.lua` |
| probe 事件上限 = **3,000,001 事件**（`probeN>3,000,000` 后静默丢弃；F10）；probe_mem 为观察性回调（不改行为） | `tools/scenario_capture.lua`（probeN 上限；addMemoryCallback） |
| Q4 输入注入 +1 帧延迟约定（frame 对齐口径） | `oracle/scenarios/README.md` §帧计数 |
| `phase-batch`：读 `analysis/traces/{scene}.{frames,probe}.jsonl` → 写 `analysis/phase/<scene>-text.jsonl`（cwd=仓库根；需 probe 含 `$2F/$30`） | `crates/game-oracle/src/bin/phase_batch.rs` |
| `render_frame`：读 `analysis/traces/{scene}.frames.jsonl`/`.probe.jsonl`/`analysis/phase/{scene}-text.jsonl`；输出写死 actual+append `.render.log`；39 诊断 env+REPO_ROOT；`--out-dir/--log-path` 尚不存在 | `crates/game-desktop/src/bin/render_frame.rs`（参数/输入/导出段） |
| `diff_visual.py` 两种模式：默认=分类账口径、`--strict`=0 容差；会重写 report/diff/whitelist-application；模板残留 W-C1（R8） | `tools/diff_visual.py`；方案 §5.6/R1 |
| 旧 22 对 probe 签名/帧/scroll/expected/actual sha | 方案 §2.6；`.render.log` 末条；git blob |
| 22 对资产问题（F-0：同 blob/无效 expected/输入缺失） | 方案 §2.4；task-24 复审 §2 |

## 12. 本方案自验收（可独立执行判据）

- **可执行性**：他人按 §0.5 执行顺序（§1→§2→§3→§4→§7a **+ §3.6 组 D 重裁**→§5→§7b→§6→§8）执行，每步有
  命令/期望输出/失败回退/BLOCKED 判据；全部路径与产物显式；
- **口径一致**：映射/判据与方案 §2.6/§4.2/§4.4 对齐（12/22 独立对、10/22 重采重渲、`--strict` 两步走）；
  去重为 **P0-0 后口径**（expected 21/22、actual 22−dups，§7.5/§8.2）；
- **组 D 来源**：**不沿用 v5 假设**（§3.6 已给 A 级反证 + 重裁流程）；组 D 未重裁前 P0-0 不宣称全绿；
- **四联来源**：**不沿用 v5 36500f 假设**（§3.7 已给 v5 帧预算 A 级反证 + 新来源/选帧/pass2 命令）；
  四联 expected 必须来自 `tas-stg08b2-direct`/`stg06-timeline` 的 pass2 选帧 capture；
- **纪律**：未把 B 级映射/assist 采集写成 A（§3.3 三重校验）；未把 P0-0 写成 0 容差达成；未改受保护路径；
- **task-30/33 findings 修正确认**：F1-F8（首轮阻塞/重要）、F9-F17/S1-S6（随文）、**B1-B5/M1（二轮）**与
  M2-M8 的逐条落点分别见 **§13 / §14**；无空跑假绿（wrapper 22 行断言）、无重复 probe 注册（§3.2 自检）、
  无过期 unique 口径（按组公式）、**expected 唯一安装点=§7a/§7.7（先于 §5）**、strict 回填有完整命令；
- **docs-links PASS**（本文件相对链接均指向存在文件/目录）；独立对抗复核 pass。

---

## 附录 A：22 对旧基线完整签名（供 manifest 预填；来自方案 §2.6，A 级）

| # | name | frame | scroll | probe | expected sha256(12) | actual sha256(12) | reuse |
|---|---|---|---|---|---|---|---|
| 1 | gameplay-stage1_f01260_stage-start | 1260 | (0,114) | 1,307,322 | 38365db2cb4f | 80047b496b89 | 独立 |
| 2 | gameplay-stage1_f01500_mid-combat | 1500 | (0,114) | 1,307,322 | 7f4ac8a23511 | fba4a5778b7c | 独立 |
| 3 | gameplay-stage1_f02349_end | 2349 | (0,96) | 1,307,322 | 4274a33f45f5 | 5e67e8aa53b5 | 独立 |
| 4 | l4_clear-inject | 4600 | (0,120) | 598,571 | 5cacbe294d53 | 519a6476ec8e | 独立 |
| 5 | l4_combat-variety | 6500 | (0,52) | 598,571 | 6f7fe4992e59 | 75261b68a6e5 | 独立 |
| 6 | l4_ending-1 | 30000 | (0,0) | 990,904 | 19b88b1ffce9 | 65af76c95e11 | actual 复用×5 |
| 7 | l4_ending-2 | 31500 | (0,0) | 990,904 | 0f8e11694d53 | 65af76c95e11 | actual 复用 |
| 8 | l4_ending-3 | 33000 | (0,0) | 990,904 | f319f1fe5256 | 65af76c95e11 | actual 复用 |
| 9 | l4_ending-4 | 34500 | (0,0) | 990,904 | 6ff345dc5c3f | 65af76c95e11 | actual 复用；expected 同 #10 |
| 10 | l4_ending-5 | 36000 | (0,0) | 990,904 | 6ff345dc5c3f | 65af76c95e11 | 上述复用 |
| 11 | l4_fade-clear | 28500 | (0,170) | 990,904 | 14802bcf460a | 2e57455e5db6 | 独立 |
| 12 | l4_late-combat | 12000 | (0,0) | 598,571 | c0baa0f8d21b | d7f37bc898bc | 独立 |
| 13 | l4_shop-window | 5400 | (0,0) | 598,571 | 988809e0b414 | 7f5de7294764 | 独立 |
| 14 | l4_stg02-start | 3404 | (0,208) | 598,571 | 788d6984576b | 164b6e48c6ca | **expected 无效（=#22）** |
| 15 | l4_stg03-start | 5700 | (0,132) | 598,571 | 24a4cb45c0d8 | 569b4600aef1 | 独立 |
| 16 | l4_stg06-late | 22000 | (0,0) | 1,167,314 | 62a8ef06c4b9 | db7262bcdd0c | 四联复用×4 |
| 17 | l4_stg07-combat | 16000 | (0,74) | 1,167,314 | 2824cb134987 | 72c47da73a5a | 独立 |
| 18 | l4_stg07-start | 14600 | (0,144) | 1,167,314 | ddb322ac5702 | d08c894ebb8b | 独立 |
| 19 | l4_stg08-combat | 18000 | (0,0) | 1,167,314 | 62a8ef06c4b9 | db7262bcdd0c | 四联复用 |
| 20 | l4_stg08-late | 28000 | (0,0) | 1,167,314 | 62a8ef06c4b9 | db7262bcdd0c | 四联复用 |
| 21 | l4_stg08-start | 16600 | (0,0) | 1,167,314 | 62a8ef06c4b9 | db7262bcdd0c | 四联复用 |
| 22 | title-menu_f00180_boot-black | 180 | (0,0) | 0 | 788d6984576b | c7081ba52b85 | expected 与 #14 同文件 |

## 附录 B：render_frame 的诊断 env 清单（canonical 必须全部 unset；A 级代码事实）

`BOTH, COLOFF, COMP, DBGPROBE, DBGSTREAM, FAMMAP, FAMROW, FLAGUPLOAD, FLIPOFF, FUSE, FUSE2, GOLDPAL,
GRPSEL, LOOPABL, NOHUDLUT, NOOWF, PALROW, PROJ2007, QBASE, QUAD, R4B, R4BOFF, R4BREV, RECP3, RECPAL,
RENDER_NT, REVORD, ROWDBL, ROWGOLD, ROWMAP, ROWOFF, ROWWIN, SLOTEND, SPRABL, TBLDRV, TEXTABL,
TEXTFALLBACK, TRIHOP, WHITE13`（39 个）+ `REPO_ROOT`（唯一允许显式设置的路径 env）。
> 其中 `FLAGUPLOAD` 会关闭 `pending_row` 路（方案 §3.1/E1）；`TBLDRV` 读 golden 表（方案 §2.4 F-0-D）——
> canonical 若带这些 = 诊断态，不得作基线。

## 附录 C：已知坑清单（操作者必读）

1. **不要 `run-all`**（40 场景数小时）；只跑 §2.1 的 I1-I5。
2. 【2026-10-18 环境更正】Windows headless 主路径**无 Xvfb**（`Mesen.exe --testRunner`）；`scenario_run.py`
   docstring 的 `xvfb-run` 仅属 POSIX 历史文案——**不要手工套 Xvfb/包装层**（Linux 备选时才涉及）。
3. probe 事件上限 = **3,000,001 事件**（`probeN>3,000,000` 后静默丢弃；不是"3,000,000 含头行"）→ 必须
   `wc -l − 该场景头行数` 核对（§2.4）。
4. `phase-batch` 与 `render-frame` 的 cwd 必须是仓库根（相对路径输入）；`render-frame` 另受 `REPO_ROOT` 影响。
5. `.render.log` 是**追加**写；两跑对照需用 `--log-path` 或分别删/隔离日志（§4.1/§4.2）。
6. `render_frame` 的 idx 只持久化**单个 nametable**（`RENDER_NT` 选中的 `nt[ntr]`，默认 ntr=0；idx 内**未记录
   ntr 字段**，只有 `palette_src/attrs/tiles`）——nt0 归属依赖默认值，manifest 必须记 env（F9）。
7. Mesen2 孤儿进程会让下一场景 trace 交错/变慢；`scenario_run.py` 已内置清场，手工跑 Mesen 后务必
   `pgrep -f tools/mesen2/Mesen` 检查。
8. `diff_visual.py` 会重写 `oracle/screens/` 下报告/差异图/申请单——P0-0 执行时属有意更新，提交前与 R8 协调。
9. 旧 expected 的"真值"地位不因本次重采失效：重采须 blob 校验或显式 dated 替换（§7）。
10. 【2026-10-18 环境更正】Windows 主路径已打通（§1）：`analysis/` 路径/`render-frame` 均按 Windows 宿主验证；
    不再要求「最终在 Linux 执行机完成」。若仍遇 `expect("trace")` panic ⇒ 先查 `analysis/traces/*.frames.jsonl`
    是否由本机 scenario_run 生成（§2.2），属输入缺失而非平台问题。

## 附录 D：降级 A 预研可先行项（2026-10-18 Lead 口径确认 2；只读，**零验收数字**）

> 环境入口未到期间的只读预研。**以下均为数据面/算术事实（A 级可复算），不是 P0-0 或 P0-1 的验收结论**；
> 任何"diff 改善/收敛"数字必须在 Linux P0-0 完成后重算。

### D.1 已完成的只读预研

| # | 项 | 结果（只读） | 复现 |
|---|---|---|---|
| D1 | 22 对旧签名预填表 | 与方案 §2.6 逐行比对 **0 mismatch**（附录 A） | 见 §附录 A |
| D2 | **组 D 帧同余算术**（复核方点名项）：`tas-clear-full-v5` 的 `capture_every 1500 probe` 与 ending/fade 帧 | 28500/30000/31500/33000/34500/36000 = 1500×**19/20/21/22/23/24**，全部整除且 ≤36500 ⇒ **同余成立（A）**；但"v5 即来源"此前是 B——**【2026-10-18 更正】** 已由 `tas-clear-full.l5.jsonl` **证伪**（v5 f28500-36000=关 2 战斗段），本行只保留"同余算术"，来源改见 §3.6（C1 `tas-stg08b2-direct` 等） | `python -c "[print(f, f%1500==0, f//1500) for f in (28500,30000,31500,33000,34500,36000)]"` |
| D3 | 候选场景覆盖算术 | 全部 True：gameplay-stage1 2400≥2349；stg03-timeline 24000≥12000；stg02-timeline 19000≥12000；stg04-timeline 26000≥12000；stg08-timeline 34000≥28000；stg07-timeline 32000≥28000；tas-clear-full-v5 300000≥36000；v4/v3/v2 30000≥30000；title-menu 1080≥180 | 读各 `oracle/scenarios/<name>.txt` 的 `frames` 行 |
| D4 | probe 配置对照 | `gameplay-stage1` 15 条 probe_mem（含 `002f 0030`/`2006 2007` **重复登记**）；`stg03-timeline`/`stg08-timeline` = 标准 8 区间；`tas-clear-full-v5` **无 probe_mem**（→ overlay 必须新增，§3.2） | `grep -c '^probe_mem ' oracle/scenarios/*.txt` |
| D5 | 独立 12 对 expected/actual **内容画像**（非验收；供 P0-1 后族判据参考） | expected 非黑像素：clear-inject 58,209 / fade-clear 59,371 / combat-variety 27,974 / shop-window 16,880 / stg07-start 11,017 / stg07-combat 9,649 / gameplay 1,074/1,571/1,751 / stg03-start 1,222 / late-combat 1,552 / title 0；actual 非黑像素同序仅 876/0/756/902/861/789/951/1,027/1,109/1,066/116/0 ⇒ 数据面印证"多数内容帧 actual 近黑"（B，待 P0-0 后重算） | 见下方脚本 |
| D6 | 附录 B 39 env 计数（复核方点名项） | `render_frame.rs` 中 `var("…")` 去重 = **39 个诊断 env + REPO_ROOT**（附录 B 与代码逐项一致） | 见下方脚本 |
| D7 | `--out-dir/--log-path` 代码 diff 规格 | 已可交 porter 预实现（§4.1）；**验收仍等 P0-0**（此改动本身不产生基线数字） | — |
| D8 | overlay 生成脚本 / wrapper 完整实现 | **已按 task-30 F1/F3 修正**：overlay 生成器按区间去重（§3.2）、wrapper 为完整实现（读 mapping.tsv→22 行断言→全字段 manifest→dedupe→可选 `--strict`，§5.3）；可先提交评审、环境就绪直接跑 | §3.2/§5.3 |
| D9 | **v5 帧预算复算（2026-10-18 更正·L5 交叉核对）** | `tas-clear-full.l5.jsonl` 检查点：stg01-clear **f28364** / stg02-clear f51332 / stg03-clear f72823 / stg04-combat **f73939** ⇒ **36500f 截断时 v5 仍在关 2** ⇒ 原 I5/§7.2"按 stage_no==6/8 选帧"**必然失败**；四联来源更正为 §3.7（`tas-stg08b2-direct` + `stg06-timeline`），v5 降为负候选 | `python -c "import json;[print(json.loads(l)['f'],json.loads(l)['checkpoint']) for l in open('oracle/traces/tas-clear-full.l5.jsonl')]"` |

**D5/D6 复现脚本**（只读；本机可跑）：
```python
import re, sys, pathlib
sys.path.insert(0, "tools")
from diff_visual import decode_png
E=pathlib.Path("oracle/screens/expected"); A=pathlib.Path("oracle/screens/actual")
IND=["gameplay-stage1_f01260_stage-start","gameplay-stage1_f01500_mid-combat","gameplay-stage1_f02349_end",
     "l4_clear-inject","l4_combat-variety","l4_fade-clear","l4_late-combat","l4_shop-window",
     "l4_stg03-start","l4_stg07-combat","l4_stg07-start","title-menu_f00180_boot-black"]
for n in IND:
    _,_,e=decode_png(E/(n+".png")); _,_,a=decode_png(A/(n+".png"))
    en=sum(1 for y in range(240) for x in range(256) if tuple(e[y][3*x:3*x+3])!=(0,0,0))
    an=sum(1 for y in range(240) for x in range(256) if tuple(a[y][3*x:3*x+3])!=(0,0,0))
    print(f"{n:42s} exp_nonblack={en:6d} act_nonblack={an:5d}")
t=open("crates/game-desktop/src/bin/render_frame.rs",encoding="utf-8",errors="replace").read()
env=sorted(set(re.findall(r'var\("([A-Z0-9_]+)"\)',t)))
print("env_count:", len(env), "| diag(不含 REPO_ROOT):", len([x for x in env if x!="REPO_ROOT"]))
print(", ".join(x for x in env if x!="REPO_ROOT"))
```

### D.2 可先行 / 不可先行（边界）

| 类 | 项 | 状态 |
|---|---|---|
| ✅ 可先行（只读） | 附录 A/D1-D6 数据面算术与画像；§3.1 映射候选表；overlay/wrapper/代码 diff 规格（D7/D8）；只读复算 C1-C11 | 本附录已做/可重复 |
| ⏸ 待环境 | §1 冒烟、§2 采集/phase-batch、§3.3 blob 校验、§4 canonical 渲染、§5 manifest 数值、§6 两跑、§7 资产重采/裁定、§8 `--strict` 记录 | 阻塞（环境入口未到） |
| 🚫 禁止 | 用预研画像/静态推理冒充 P0-0/P0-1 验收；在 P0-0 完成前用旧 22 对 diff 数字作验收依据 | — |

### D.3 给复核方（task-30/32 复审）的点名自检

- **§3 组 D 帧同余**：D2 已给算术（19..24 × 1500 = 28500..36000，≤36500）；**"同余"只证帧可由 v5 的
  `capture_every 1500` 覆盖，不证来源**——来源判定仍是 §3.3 的 capture blob 相等（本手册已明确标注 B→A 的升级条件）。
  **【2026-10-18 更正】** 来源=v5 已被 §3.6 证伪；且 **36500f 截断连 stage6/8 都到不了**（§3.7/D9）——
  四联改直入/自关窗轨，v5 仅负候选。
- **附录 B 39 env**：D6 给可复算计数（39 诊断 + REPO_ROOT；脚本见 D.1），与附录 B 清单逐项对应。
- **本轮 F1/F4 修正后的复核点**：overlay 生成器现在**只补缺失区间**（§3.2 自检：stg03 源 8→输出 8、
  v5 源 0→输出 8；断言无新增重复）；probe 头行判据按 §2.4 逐场景表（title 0/无文件、gameplay 15、stg03 8、
  stg08 8、overlay 8；**2026-10-18 更正**：新增 `tas-stg08b2-direct` **10**、`stg06-timeline` **0**（pass2 补 8））。

---

## 13. task-30 复核 findings（F1-F17/S1-S6）修正映射（**二轮 B1-B5/M1 见 §14**）

> 来源：[adversarial-review-l4-baseline-freeze-plan.md](adversarial-review-l4-baseline-freeze-plan.md)
> （verdict=needs_revision，被审 = 783 行/sha `9f548f2a…`）。本稿逐条修正如下（行号以本修订稿为准，节锚为主）。

| # | 级 | 修正落点（节） | 修正内容摘要 |
|---|---|---|---|
| **F1** | 阻塞 | §2.1（I5 行）、§3.2、§3.4 | overlay 生成器**按区间去重**（读源 probe_mem 集合，只补缺失）；生成后断言 `grep -c '^probe_mem '`=8 且无新增重复。**（二轮 M2 后 overlay 只剩 v5 一个：stg03 的 5400 改用源 capture_every——见 §14/M2）** |
| **F2** | 阻塞 | §5.1 dedupe、§7.5、§8.2 | 去重口径改 **P0-0 后公式**：expected unique ∈{21,22}、actual unique = 22 − documented_duplicates；脚本**按 manifest 22 名过滤**（expected/actual_canonical_run1），不 glob 整目录（旧 17/127 只作背景）；输出 `analysis/repro/dedupe.tsv` |
| **F3** | 阻塞 | §5.3 | wrapper 升级为**完整实现**：读 `mapping.tsv`（22 行断言、非空防假绿）→ 两跑渲染 → §5.1 全字段 manifest（expected/expected_source/input/render_env/argv/status/reuse/capture_tag/trigger/deterministic）→ dedupe → 可选 `--strict` 记录；失败 exit1，绝不空跑 PASS |
| **F4** | 重要 | §2.2 判据、§2.4 表、§3.3 | 头行判据改**逐场景表**（title 0/无文件；gameplay 15；stg03 8；stg03-overlay 8；stg08 8；v5-overlay 8）；事件数=`wc-l − 该场景头行`；C3 校验脚本带 HDR/OLD 表 |
| **F5** | 重要 | §2.1（I5b）、§7.2 | 两遍机制：第一遍 v5 只出 `frames.jsonl` → watch 选帧（脚本落 `l4-selected-frames.json`）→ 第二遍 `l4-baseline-v5-pass2` 带 `capture <f> l4f<f>` 重跑取 shots |
| **F6** | 重要 | §7.1 | 改指 **`l4-visual-plan.md:349-350@1be66e5`**（非 l4-signing-criteria §4.5）；expected_source 改 `stg03-timeline/3404/stg02-start` |
| **F7** | 重要 | E4、§3.2/§5.3 | Python 要求改 **≥3.10**（`Path.write_text(newline=)` 需 3.10）+ 版本断言命令；代码片段保持 3.10+ |
| **F8** | 重要 | §7.3、§7.5、§8.2 | actual 口径统一：五帧 actual 两两不同；**若裁定 documented duplicate 必须在 manifest 登记**，与"actual 22−dups unique"一致 |
| **F9** | 次要 | 附录 C-6 | idx 表述改为"只持久化 RENDER_NT 选中的单个 nametable（默认 ntr=0，idx 内未记录 ntr）" |
| **F10** | 次要 | §0 R-2、§2.4、附录 C-3、§11 | 上限统一为 **3,000,001 事件**（`probeN>3,000,000` 后丢）；监控留 <3,000,000 裕度 |
| **F11** | 次要 | §2.5、§2.6 | title 无 probe：fixtures/产物清单注明"无 probe 场景不产 `.probe.jsonl`" |
| **F12** | 次要 | §2.3 | run2 复制命令改为**完整可执行**（列出全部场景集合，含 overlay；无占位省略号） |
| **F13** | 次要 | §8.1 | 补实际归档/替换命令：旧 actual 22 对归档、canonical run1 安装、expected 归档+落新、diff/strict 生成、strict 回填 manifest |
| **F14** | 次要 | §4.1 | 回归 A/B 给可复现步骤：`git worktree add` 取改前提交→分别 build→同输入渲染→sha 对比（旧版无 --out-dir 时走 REPO_ROOT 沙盒） |
| **F15** | 次要 | §3.1 注、§7.2 | "同 blob"=A；"商店冻结屏"=B（引 `docs/bug-ledger.md` OBS-1），不写成 A |
| **F16** | 次要 | §3.1（新增源 capture 列）、§3.3、§3.5 | mapping.tsv 增 `capture_tag`（组 A=`stage-start`/`mid-combat`/`end` 等）与 `duplicate_of` 列；③校验按 §3.0 源 capture 路径/tag |
| **F17** | 次要 | §7.5 | "DUO"→**DUP** |
| **S1** | 建议 | §5.3 | wrapper 完整工具化（同 F3） |
| **S2** | 建议 | §7.5 | 去重脚本按 manifest 名过滤 + 输出 `dedupe.tsv`（同 F2） |
| **S3** | 环境（2026-10-18 更正） | E11 | **Windows headless 主路径无 Xvfb/X11 依赖**（Mesen.exe 自带 runtime）；P0-0 构建 game-desktop/game-oracle（**会编译 game-desktop 的 winit/softbuffer 依赖**；task-87 F-1）；POSIX 备选构建才需 X11 库 |
| **S4** | 建议 | 附录 D D8 | 状态更新：overlay/wrapper 已按 F1/F3 修正 |
| **S5** | 建议 | §3.3 末注、附录 D2/D3 | "同余≠来源"醒目交叉引用 |
| **S6** | 建议 | 头部、§11 | 标注"环境事实核验时点 HEAD 167d8a9（分析时点）"，执行前重核 `--out-dir` 是否已实现 |

---

## 14. task-33 复审（B1-B5/M1 及 M2-M8）修正映射

> 来源：[adversarial-review-l4-baseline-freeze-plan.md](adversarial-review-l4-baseline-freeze-plan.md) 复审段
> （2026-10-18 task-33，被审 = 1163 行/sha `4df844f6…`；verdict=needs_revision，F1/F4/F5/F6/F7 已闭环）。
> 本稿在 §13 基础上二次修正（行号以本修订稿为准，节锚为主）。

| # | 级 | 修正落点（节） | 修正内容摘要 |
|---|---|---|---|
| **B1** | 阻塞 | §0.5、§7 引言、§7.3、§7.7、§8.1-步骤3 | **expected 唯一安装点 = §7a/§7.7**：新图先进 `analysis/expected-new/`，由 §7.7 归档旧件+落新件；**所有 expected 替换先于 §5**；ending-4/5 替换移入 §7a；§8.1 步骤 3 改为**只核验** manifest `expected_sha256` 全等（不再复制/归档，消除"新件被当旧件归档"与 manifest 过期） |
| **B2** | 阻塞 | §5.1 dedupe、§5.3 `dedupe()`、§7.5、§8.2 | `dedupe()` 同时收 **expected+actual** 重复组、合并为 side=`expected+actual/expected/actual` 登记；`dedupe_ok` 改**按组公式** `unique == 22 − Σ(len−1)`（不再用 documented_act 计数误判）+ `registry_ok`（组内非主帧必须在 mapping 标 `duplicate_of`）；§7.5 脚本断言**所有重复 ⊆ manifest.documented_duplicates**（含 expected 侧、支持 >2 帧组） |
| **B3** | 阻塞 | §8.1 步骤 4/5 | strict 回填给**完整可执行命令**（读 `visual-strict.log` 的 STRICT 行→`manifest.strict`，`ran_after_asset_swap=true`）；时序明确=**actual 换新之后**；expected 替换命令收敛到 §7.7（stg02/四联/ending-1..5/fade 全四类，`analysis/expected-new` 为 staging，定义/生成/安装命令齐全） |
| **B4** | 重要 | §4.1 | A/B 回归步骤重写：改前二进制**无 `--out-dir`** → 旧版走 `REPO_ROOT` 沙盒（完整 mkdir/symlink/运行命令），新版走 `--out-dir`；两路 png 齐全后 `sha256sum` 对比；**删除短路后缀（缺文件即 FAIL）** |
| **B5** | 重要 | §7.1、§3.0/§3.1 | #14 改用源 `stg03-timeline` 的 `capture 3404 stg02-start`（shots `stg03-timeline_f03404_stg02-start.png`）；I3b 已按 M2 取消（不再引用不存在的 `l4-baseline-stg03_f03404_l4f3404.png`），备选用"把 3404 加进 overlay"仅作 fallback 说明 |
| **M1** | 重要 | §3.5 schema、§5.3、§5.1 | mapping.tsv 增 `reuse` 列（按附录 A 独立/复用逐对填）、`status` 列**透传**（`replaced-invalid-asset`/`documented-duplicate` 不丢）；wrapper 读 `p["reuse"]`/`p["status"]`（仅确定性失败降级 blocked:nondeterministic），`load_pairs` 校验必需列 |
| **M2** | 中低 | §2.1/§2.2/§3.0/§3.1/§3.4 | #13(5400) 直接用源 `stg03-timeline` 的 `capture_every 300 probe` 图（`shots/stg03-timeline_f05400_probe.png`）；**取消 I3b overlay**（省一次 24000 帧重跑），overlay 只剩 v5 |
| **M3** | 中低 | §7.2 | 选帧文字判据与脚本一致（start=首个、combat=中位、late/last、stg06-last）；`first/median/last` 带 StopIteration 处理；pass2 生成/重跑/phase-batch 命令完整；staging→§7.7 安装路径明确 |
| **M4** | 低 | §2.1/§2.2/§2.4 | pass2 的 phase-batch 移到 §7.2 之后（不再提前出现在 §2.2）；§2.4 表增 `l4-baseline-v5-pass2` 行并注明"事件数不作校验/来源判据" |
| **M5** | 低 | §5.1/§7.1/§3.5 | `expected_source` 统一为**字符串** `scene:frame:capture_tag`（schema 与 §7.1 一致；mapping.tsv 列格式写死） |
| **M6** | 低 | §7.7 | 四类资产（stg02/四联/ending-1..5/fade）的 staging+归档+落新命令补全为可执行脚本（不再只有 stg02 示例） |
| **M7** | 低 | §5.3 `input_sha()`/`env` | title 无 probe ⇒ `input_sha.probe=null`（注明合法）；`env.os` 用 `os.uname().sysname + machine`（"Linux x86_64" 而非仅 machine） |
| **M8** | 低 | §2.3 判据 | 改为"**6 个 frames 集合 + 5 个 probe/text 集合**（title 仅 frames）" |

---

## 15. 【2026-10-18 更正·L5 交叉核对】组 D 来源勘误（覆盖 §13/§14 历史口径）

- §13/§14 的修正映射为**历史留档**；其中所有"组 D=v5 / I5 覆盖 ending/fade"口径已被 **§3.6** 覆盖：
  `oracle/traces/tas-clear-full.l5.jsonl` 证明 v5 的 f28364=stg01-clear、f29478=stg02-combat、
  f51332=stg02-clear ⇒ 28500-36000 落在**关 2 战斗段**，不可能是 ending/fade。
- 组 D 6 对（#6-#11）来源=**待重裁**（§3.6 候选 C1-C5；判据=blob+mode 语义+trigger 合法）；
  重裁完成前：不 staging、manifest 6 行 `blocked:no-source`、P0-0 不宣称全绿。
- 该更正**不改变**：组 A/B/C 的 capture 源（§3.0）、F1/F4/F5/F6/F7/B1-B5/M1 的历史修法、去重/两跑/wrapper 判据。

### 15.1 【2026-10-18 更正·四联帧预算】I5/§7.2 的 36500f 截断失效（task-50）

- **追加更正**：除组 D 外，**四联 #16/#19/#20/#21 的 v5 选帧方案同样失效**——`tas-clear-full.l5.jsonl`
  显示 v5 到 **f73939 才 stage4-combat**（stg01-clear f28364/stg02-clear f51332/stg03-clear f72823）⇒
  I5 截断 **36500f 时仍在关 2**，`stage_no==6/8` 选帧必然失败。
- **更正落点**：§2.1（I5/I5b→I5/I5b+I6/I6b；旧 v5 降负候选）、§2.2/§2.3/§2.4/§2.6（命令/两跑/头行/产物）、
  **新增 §3.7**（来源表+选帧脚本+pass2 命令+trigger 登记）、§7.2（新来源+旧动作留档）、§7.7 staging 源、
  §0.5（执行顺序/环境清单）、§8.2（验收项）、附录 D 新增 **D9**。
- **新来源**：stg08-start/combat/late ← `tas-stg08b2-direct`（stage8 直入、40000f、源 10 probe）；
  stg06-late ← `stg06-timeline`（自关窗 f13000-13977、源 **0 probe → I6b pass2 补 8**，F49-2 更正）。
  `trigger=injected` 如实登记。
- **不改变**：expected 唯一安装点（§7.7）、四联"重采真值"裁定本身、组 D 重裁（§3.6）与其余判据。

---

## 16. 【2026-10-18 环境更正】Windows Mesen2 2.1.1 headless 打通（task-81）

> 本节=环境主路径的 dated 更正与实测证据。原「Linux+Mesen2 入口/自管 Xvfb/xvfb-run/`/DoNotSaveSettings`」
> 表述仅作 **POSIX 备选**留档（§1.4）；主路径以本节 + §1 为准。

### 16.1 环境事实（本机实测）

| 事实 | 值/证据 |
|---|---|
| 宿主/工具链 | Windows x64；Python **3.12.1**；cargo **1.96.0-nightly (cbb9bb8bd 2026-03-13)** |
| Mesen2 | **Windows 版 2.1.1 x64**：`tools/mesen2/Mesen.exe`，sha256=`d2eb03c2590c648bf329f127ebcfefd70130e7690a9e2ccdba8616faea1fe96b`（70,470,144B；gitignored 可再生件） |
| headless 驱动 | `Mesen.exe --testRunner <lua> <rom>`（**无显示/Xvfb**）；`scenario_run.py` 在 Windows 自动 `HEADLESS=1`（`GUNNAC_MESEN`/`GUNNAC_HEADLESS` 可覆盖） |
| Lua IO 配置前置 | `%USERPROFILE%\Documents\Mesen2\settings.json` → **`Debug.ScriptWindow.AllowIoOsAccess=true`**（有效键；`ScriptWindowConfig.AllowIoOsAccess` 实测不生效；机理/判据见 [gotcha-mesen2-headless-windows-io](../knowledge/gotcha-mesen2-headless-windows-io.md)）；改后重启 Mesen；校验命令见 §1.1 E3 |
| WSL2/Linux 版 Mesen2 | 2.1.1 Linux 任何 invocation `std::bad_cast`（exit 134）⇒ **不可用**，仅备选（换可用构建才谈 Linux 路径） |

### 16.2 冒烟实测（2026-10-18，本机）

```
== scenario run: title-menu ==
  DSL: oracle/scenarios/title-menu.txt sha256=258067a319351ef9…
  frames: 1080 trace_lines: 1080 captures: 37 shots: 37 wall: 3.5s
  DONE: {"frames": 1080, "trace_lines": 1080, "captures": 37, "shots": 37, "bot_stop": false}
```
- **两跑确定性**：trace sha256=`62c9ea09580654ce691662c4d35ef98d0e722bdeaa88ccd893172a5d9389a331` 两跑相同；
  `shots/title-menu_f*.png` **37/37 逐字节相同**；
- **门禁**：`gates/cli.py status` = PASS=11 / FAIL=0 / SKIP=1（addr-bridge）/ TOLERATED=0，**STATUS: OK**；
  `gates/cli.py rom` = `[PASS] rom-integrity`（4 锚点 sha）。
- **结论：P0-0-0（环境接收）通过，可启动 P0-0-1（§2 输入采集）。**

### 16.3 影响面（已同步修改）

- **§1**：E1-E11 改 Windows 主路径（E3=Lua IO 配置、E3b=无显示、E11=无 X11/Xvfb 依赖〔**构建 game-desktop
  仍会编译 winit/softbuffer 依赖；POSIX 备选构建才需 X11 库**，见 §1.1 E11〕）；§1.2 冒烟判据+
  两跑确定性；§1.3 版本记录（Mesen.exe sha）；§1.4 headless 驱动/POSIX 备选。
- **§0.5**：新增「环境就绪 ⇒ P0-0-0 通过，可启动 P0-0-1」。
- **§2**：命令解释器统一 `python`（Windows；POSIX 备选换 `python3`）。
- **§8.2**：验收清单新增环境接收项（冒烟/两跑/gates 证据）。
- **§9.2/§10.1**：「必须 Linux+Mesen2」降级为「需 Mesen2 实机采集（Windows headless 主路径）」；
  Linux 入口事项标记**已解决**（历史留档）。
- **§11/附录 S3/附录 C**：Xvfb/`/DoNotSaveSettings`/Windows 路径风险降级为 POSIX 备选表述。
- **【2026-10-18 task-87（task-85 复审 F-1..F-4）】** E11 依赖事实更正（winit/softbuffer 必编译、
  POSIX 才需 X11；§1.4/附录 S3 同步）；E3/§16.1 补证据指针（知识库 gotcha）；
  §1.3/§8.2 的 `analysis/p0-0-env.txt` 注明为 P0-0-0 执行时产物；E9 内存 GiB/GB 双口径统一。

### 16.4 复现

```powershell
python tools/scenario_run.py run title-menu --timeout 600
python gates/cli.py status
python gates/cli.py rom
```
判据见 §1.2；若 E3 未配置，典型症状=无 trace/shots（Lua `io.open` 被拒）。
