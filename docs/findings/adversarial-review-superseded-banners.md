# 对抗复核：headless 输入失效 gotcha + superseded 横幅 + 手册指针（task-135）

> 日期：2026-10-18｜复核方：decomp-verifier（verifier 角色，证伪立场）｜被审：spec-writer 的 task-135
> （新知识条目 + 6 处 superseded 横幅 + 手册局部指针；docs-only，工作区未提交）
> 复核方式：**只读 + 单门 knowledge-format/docs-links**（未跑 Mesen/gates check/status/visual，未争串行锁；l4-analyst 的 task-133 GUI 重采进行中）。
> 证据源：`docs/knowledge/*`、6 份 findings、`docs/findings/l4-baseline-freeze-plan.md`、`docs/findings/l5-b1-anchor-rederive-run1.md` §0/§2、`tools/scenario_run.py`、`analysis/traces/*.frames.jsonl`、`shots/*`、`%USERPROFILE%\Documents\Mesen2\settings.json`。
> 本文件为唯一写入，未 git add/commit；未改被审文档/代码/配置。

---

## 0. VERDICT

**pass（2 Low 建议 + 2 Info；全部核心断言独立复现）**——task-135 的新 gotcha、6 处横幅、手册 banner+5 指针、交叉引用、knowledge-format（26）/docs-links（376/465）均成立。
两条 Low 均**不阻塞**：①截图 sha 指针加"历史"限定（文件已被 task-133 GUI 重采覆盖）；②**提交顺序**需保证 task-133 的 `tools/scenario_run.py` GUI 默认与 gotcha 同批/先入库（否则知识条目"当前处置"在 HEAD 暂不可复现——gotcha 已含自查句，需执行）。

| ID | 分类 | 级 | 问题 | 处理 |
|---|---|---|---|---|
| AR136-1 | 规格缺口（证据指针时效） | 低 | gotcha L17-19 以**现在时**引用 `shots/gameplay-stage1_f01260_stage-start.png`=开场故事（sha `8f5c2ffa…`）；该文件已被 task-133 GUI 重采**覆盖**（当前实际 sha=`38365db2cb4ff5f6…`）。历史值有据：`l5-b1-anchor-rederive-run1.md` L19/L94 记录 sha12=`8f5c2ffa6910` ✓；`gs5…_f02400_gate-inject.png` 仍=`4f3d7af258a8` ✓（未被覆盖） | 建议括注"（headless 期历史截图；该文件已被 task-133 GUI 重采覆盖，sha 见 l5-b1 §0）" |
| AR136-2 | 规格缺口（断言适用范围） | 低/Info | gotcha 处置②"输入有效性断言 4 条（**缺一不可**）"中第④条"c1/c2 必须分叉"**只适用于设计上有双输入序列的场景**；对单序列/单输入场景"缺一不可"不成立，未来照抄会误报 | 建议第④条加"（仅对有 c1/c2 双序列设计的场景）" |
| AR136-3 | 报告/证据（信息） | Info | `docs/knowledge/README.md` §4 索引只列 **23/26**（3 条既有条目未索引：`gotcha-mesen2-ppu2006-callback-quirk.md`、`gotcha-shared-tree-mutation-restore.md`、`gotcha-shared-workspace-git-staging-and-tdd-red.md`）；**新条目已正确索引**（L71） | 另卡补索引（pre-existing，不在本卡范围） |
| AR136-4 | 纪律（提交顺序） | Info | `tools/scenario_run.py` 的 GUI 默认改动是 **task-133 未提交工作区改动**（`HEAD:tools/scenario_run.py` 的 Windows 默认仍是 headless）；gotcha 处置①断言"Windows 默认 GUI 模式"并已写"引用前验证 `git log -1 -- tools/scenario_run.py`"自查句 | 提交时保证 **task-133 工具改动与 gotcha 同批/先入库**（或 gotcha 改为"（task-133 提交后生效）"） |
| AR136-5 | 运行提醒（信息） | Info | `settings.json` 当前 `"EmulationSpeed": 1000`（GUI 批次进行中，符合 gotcha 描述） | task-133 结束**恢复 100**（gotcha 已写；执行项） |

---

## 1. 新知识条目逐项核验

文件：`docs/knowledge/gotcha-mesen2-testrunner-setinput-noop.md`（**73 行**，untracked）。格式：frontmatter `type: gotcha/title/date/status: confirmed` ✓、文件名前缀 `gotcha-` ✓、正文 `现象/为什么/何时使用/处置/证据指针` ✓（README §2-3 要求三节+指针；多"处置"节不违规）。

### 1.1 现象（逐条证据）

| gotcha 断言 | 独立核验 | 判定 |
|---|---|---|
| headless 采集 DONE/trace 满行/`frame_errors=0` 但游戏没动 | `l5-b1` §0 L12-13：3 场景 DONE 但输入未生效；`gs5` trace `run.json` done 有效（5000/5000、wall 14.5s）且 f2400 `controller_held=0`、`stage_no=1`、`flags27=0` | ✓ A |
| `$18`=0 / `getInput` 空 / `flags27`=0 / `stage_no`=1 / `countdown2f` 无 50 环 / `flags26 bit4` 窗空 | `l5-b1` L21（flags27/stage_no/countdown2f）、L22（flags26 bit4 窗=[]，预期 f913..1192）、L62/65（F1 FAIL）、L104（`pcall(emu.setInput,btn,0)` 未记录读回）；`$18`=0/getInput 空：Lead 实测（gotcha L59-61 已注明来源）+ `tools/scenario_run.py:67` 代码注释同述 | ✓ A（`$18/getInput` 为 Lead 实测；其余 tracked 文档） |
| 截图 `gameplay-stage1_f01260`=开场故事 sha `8f5c2ffa…`；`gs5…f2400` sha `4f3d7af2…` | `l5-b1` L19/L94 = `8f5c2ffa6910`；L20/L96 = `4f3d7af258a8`；当前 `gs5…_f02400_gate-inject.png` 实测同 sha ✓；`gameplay-stage1_f01260` 当前已被 GUI 重采覆盖（见 AR136-1） | ✓（历史值）/AR136-1 |
| `c1↔c2 mismatch=0`（f1..5000 全同） | `l5-b1` L23、L66（F2 文本同） | ✓ A |
| L5 B1 `TRACE-DIFF: MISMATCH (3/8)`、SUMMARY `6/15` | `l5-b1` L13、L59、L65-69 | ✓ A |

### 1.2 为什么 / 处置

| gotcha 断言 | 独立核验 | 判定 |
|---|---|---|
| testRunner 下 `emu.setInput` no-op ⇒ 输入未注入；DONE 只证明收尾不证明语义 | 与 `l5-b1` §0 结论（L12-17/106）一致；`tools/scenario_run.py:67` 注释同述 | ✓ A/B |
| GUI 生效对比：`getInput={start=true}`、`game_action_mode=7`、玩家 x 128→243、bomb 3→4 | 当前 **GUI 重采中的** `analysis/traces/gameplay-stage1.frames.jsonl`（复核时 n=2278 且仍在增长）实测：`controller_held` 非零帧 **1022**、`game_action_mode=7` 出现、`player_x` 范围 **(0,243)** 且 f1260=128、`bomb_ammo` 值集含 **3 与 4** ⇒ 数字可独立复现；`getInput={start=true}` 仅 Lead 实测（只读不可复现，gotcha 已注明） | ✓ A（除 getInput 为 Lead 源） |
| 处置① Windows 默认 GUI：`HEADLESS=GUNNAC_HEADLESS=="1"`、`GUI_WINDOWS=(not HEADLESS) and os.name=="nt"`、命令 `[MESEN, ROM, lua, "/DoNotSaveSettings"]` | `tools/scenario_run.py` 实读：L69 `HEADLESS = os.environ.get("GUNNAC_HEADLESS", "0") == "1"`、L71 `GUI_WINDOWS = (not HEADLESS) and os.name == "nt"`、L604-608 GUI 分支命令含 `/DoNotSaveSettings` ✓ | ✓ A（**工作区未提交** → AR136-4） |
| 处置② 输入有效性断言 4 条 | 正文 L51-53 四条齐全；仅第④条范围问题 → AR136-2 | ✓/AR136-2 |
| 处置③ `EmulationSpeed=1000` 提速、批次后恢复 100 | `%USERPROFILE%\Documents\Mesen2\settings.json` 实测 `"EmulationSpeed": 1000` ✓（gotcha 记"本机实测当前=1000"准确；恢复 100 为执行项） | ✓ A（AR136-5） |

### 1.3 索引 / 门禁

- `docs/knowledge/README.md` L71 已加新行（type=gotcha + 一句话）✓；
- **knowledge-format 单门实跑 PASS：`26 条知识条目 frontmatter/type/文件名前缀合规`** ✓（与任务卡"knowledge-format PASS 26"一致）；
- README 索引覆盖 23/26（pre-existing gap → AR136-3；新条目已索引，非本卡遗漏）。

---

## 2. 6 处 superseded 横幅核验

6 文件 L3 横幅**逐字一致**（集合去重=1）：

> 【2026-10-18 更正·superseded】本文件基于 headless(testRunner) 采集，其 emu.setInput 不生效 ⇒ 输入未注入、语义无效；重采见 task-133（GUI 模式）；原数字保留作历史。

| 文件 | 行数(cur/HEAD) | diff numstat | 原文保留抽查（关键结论仍在） |
|---|---|---|---|
| `p0-0-capture-run1.md` | 172/170 | +2/-0 | I5 `BLOCKED: capture-termination`、`frames_lines=39998/40000`、判据/回退建议仍在（L15/L105/L126-128）✓ |
| `p0-0-render-run1.md` | 183/181 | +2/-0 | 两跑 png/idx sha 16/16、actual 侧 6 对同 blob、ROM sha 等仍在（L17-18/L29-38）✓ |
| `p0-0-groupD-source-decision.md` | 201/199 | +2/-0 | 组 D=blocked:no-source、blob 0/6、frame_errors=142149、C1 BLOCKED 仍在（L18/L65 等）✓ |
| `p0-0-same-blob-adjudication.md` | 166/164 | +2/-0 | "真实同画面/OBS-1"结论 + 证据链仍在（L5/L13-25/L31 表）✓ |
| `p0-0-readjudication.md` | 131/129 | +2/-0 | "16 可渲/6 blocked、P0-0 不得全绿"仍在（L5/L16/L20/L45）✓ |
| `l5-b1-anchor-rederive-run1.md` | 139/137 | +2/-0 | MISMATCH 3/8、SUMMARY 6/15、flags27/stage_no、截图 sha 仍在（L13/L19-24/L59-69/L94-104）✓ |

- **原文未删**：每文件 `+2/-0`（横幅行+空行），`git diff` 无删除行 ✓；行数 +2 与 HEAD 对应 ✓。
- **语义正确**：headless 输入未注入 ⇒ 采集语义无效 ⇒ task-133 GUI 重采；原数字保留作历史 ✓；`same-blob` 的"同画面"结论也被正确标注 superseded（其证据来自输入未注入的 headless 源）✓。
- 覆盖面：`docs` 全库 grep `superseded` 的 findings 横幅恰为上述 6 份（+手册/知识库引用），无扩散/漏标 ✓。

---

## 3. 手册横幅 + 5 处局部指针

| 落点 | 实测 | 判定 |
|---|---|---|
| 顶部 **L3-7** | `【2026-10-18 更正·superseded（采集环境/组 D 定谳部分）】`：headless setInput 不生效 ⇒ 输入未注入、语义无效；相关定谳（含组 D `blocked:no-source`、四联"直入"判定）**待 task-133 GUI 重采后复核**；原数字/结论保留；链接知识库 gotcha ✓ | ✓ |
| §3.6 **L726-728**（第 6 条） | `【2026-10-18 环境层 superseded】` C3/C2 采集=headless；`blocked:no-source` 待 task-133 复核、原结论保留 + gotcha 链接 ✓ | ✓ |
| §3.7 **L956-958** | 四联来源候选采集=I5 headless；"直入不成立/待重评"复核须在 task-133 后；原注保留 + gotcha 链接 ✓ | ✓ |
| §7.3 **L1375-1376** | 组 D 定谳依据 C1/C2/C3=headless；待 task-133 重采后复核；原结论保留 ✓ | ✓ |
| §7.4 **L1403-1404** | fade 定谳依据采集同组 D=headless；待 task-133 重采后复核、原结论保留 ✓ | ✓ |
| §8.2 **L1687-1688** | 组 D 采集=headless testRunner、输入未注入语义无效；待 task-133 GUI 重采复核后重判、原结论保留 ✓ | ✓ |

- **living doc 局部指针策略合理**：仅头部总横幅 + 5 处受影响段落局部注，不重写全文/不删历史；指针语义一致（环境层 superseded、原结论保留、task-133 复核）+ 知识库链接 ✓。
- 手册本卡 diff = **+19/-0**（横幅+5 指针），无删除行 ✓。

---

## 4. 交叉引用（headless-io gotcha L10/L12-14）

- L10：`§ 多段式坑（… + 输入层）… 语义无效` 已加"输入层" ✓；L12-15 新增块：`testRunner 下 emu.setInput 不生效 ⇒ 输入未注入、语义无效、含输入场景必须 GUI`，链接 `[gotcha-mesen2-testrunner-setinput-noop.md](../knowledge/gotcha-mesen2-testrunner-setinput-noop.md)` ✓；并声明"与 IO 开关、100s 超时截断彼此独立" ✓。
- 链接可解析：docs-links 单门 PASS 已覆盖 ✓。

---

## 5. 门禁 / 范围 / 纪律

- **knowledge-format PASS**：`26 条知识条目 frontmatter/type/文件名前缀合规`（单门实跑）。
- **docs-links PASS**：`376 个 md / 465 条本地链接全部有效`（单门实跑，与任务卡一致）。
- **未动代码/expected/门禁**：`git status --porcelain oracle gates data` **空** ✓；本卡 docs 面：6 findings（+2 行/份）、手册（+19）、knowledge README（+1 行）、headless-io gotcha（+6/-1）、新 gotcha（新增）；`crates/`、`tools/scenario_run.py` 的 M 属并行任务（task-133 等），范围归因仅按文件类别记录，不作作者断言。
- **未跑**：Mesen/gates check/status/visual（遵守串行锁与任务卡纪律）；只读+两个单门。
- 工作区并行状态：task-133 GUI 重采进行中（`gameplay-stage1.frames.jsonl` 复核时 2278 帧且增长、shots 被覆盖）——本复核已据此区分"历史 sha/当前文件"（AR136-1）。

---

## 6. 复现命令（只读；PowerShell 可执行）

```powershell
# 1) 横幅逐字一致 + 原文未删（+2/-0）
@'
import pathlib
files=['p0-0-capture-run1.md','p0-0-render-run1.md','p0-0-groupD-source-decision.md',
       'p0-0-same-blob-adjudication.md','p0-0-readjudication.md','l5-b1-anchor-rederive-run1.md']
vals=[pathlib.Path('docs/findings/'+f).read_text(encoding='utf-8').splitlines()[2] for f in files]
print('identical:',len(set(vals))==1); print(vals[0])
'@ | python -
git diff --numstat docs/findings/p0-0-capture-run1.md docs/findings/p0-0-render-run1.md docs/findings/p0-0-groupD-source-decision.md docs/findings/p0-0-same-blob-adjudication.md docs/findings/p0-0-readjudication.md docs/findings/l5-b1-anchor-rederive-run1.md
# 2) 证据：l5-b1 sha/flags/断言；GUI trace 数字
Select-String -Path docs/findings/l5-b1-anchor-rederive-run1.md -Pattern '8f5c2ffa|4f3d7af2|MISMATCH|6/15|flags26 bit4|mismatch=0'
@'
import json
xs=[]; bombs=set(); modes=set(); held=0
for l in open('analysis/traces/gameplay-stage1.frames.jsonl',encoding='utf-8',errors='replace'):
    r=json.loads(l); w=r.get('w',{})
    if 'player_x' in w: xs.append(w['player_x'])
    if 'bomb_ammo' in w: bombs.add(w['bomb_ammo'])
    if 'game_action_mode' in w: modes.add(w['game_action_mode'])
    if w.get('controller_held'): held+=1
print('x range',(min(xs),max(xs)),'bombs',sorted(bombs),'modes',sorted(modes),'held',held)
'@ | python -
# 3) 工具代码事实（GUI 默认）
Select-String -Path tools/scenario_run.py -Pattern 'HEADLESS =|GUI_WINDOWS|DoNotSaveSettings|setInput'
# 4) EmulationSpeed
python -c "import pathlib; [print(l.strip()) for l in (pathlib.Path.home()/'Documents/Mesen2/settings.json').read_text(encoding='utf-8',errors='replace').splitlines() if 'EmulationSpeed' in l]"
# 5) 单门
python -c "import sys;sys.path.insert(0,'gates');import cli;print(cli.gate_knowledge_format());print(cli.gate_docs_links())"
```

---

## 7. 自查

- 只写本文件；未 git add/commit；未改被审文档/代码/配置/门禁；未跑 Mesen/gates check/status/visual。
- 证据独立重导：6 横幅逐字+numstat、l5-b1 §0/§2 证据行、6 文件关键结论抽查、手册 6 落点、交叉引用、工具代码、settings、GUI/headless trace、两单门输出；未采信被审自述。
- 未证不发明：`getInput={start=true}` 仅 Lead 实测源（已注明）；task-133 进行中导致 shots/traces 变化已按"历史/当前"区分（AR136-1）；worktree 并行改动不作作者归属。
- 建议：AR136-1/AR136-2 两处括注（各 1 行）；AR136-4 提交顺序（工具+gotcha 同批/先入库）；AR136-3/AR136-5 另行/执行项。

【2026-10-18】decomp-verifier，task-136。
