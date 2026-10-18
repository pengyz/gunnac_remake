# 对抗复核：P0-0 手册组 D dated 更正（task-113）

> 日期：2026-10-18｜复核方：decomp-verifier（verifier 角色，证伪立场）｜被审：spec-writer 的 task-113
> （`docs/findings/l4-baseline-freeze-plan.md` dated 更正，工作区未提交，2007 行）
> 复核方式：**只读** + 单门 docs-links（未跑 Mesen/gates check/status/visual，未动串行锁）；独立重导证据，不采信被审自述。
> 数据源：`docs/findings/p0-0-groupD-source-decision.md`（untracked）、`analysis/repro/p0-0-groupD/*.json`、`analysis/traces/{tas-stg08-direct,tas-stg07-relay,tas-stg08b2-direct}.frames.jsonl`、`oracle/screens/expected/*.png`、`shots/*.png`、`oracle/scenarios/*.txt`。
> 本文件为唯一写入，未 git add/commit；未改被审文档。

---

## 0. VERDICT

**needs_revision（Low-only：1 必改 + 4 info）**——task-113 声明的 **5 条定谳内容、dated 更正纪律、影响面 4 处同步、证据指针语义、docs-links 全部通过**；
唯一必改 = **AR113-1**（§3.7 新加更正注把 C1 的 `stage_no==8` 起点写成 f1255，实测 C1=**f1265**；10 帧数字失实，不影响"硬帧号映射不可用"结论）。
其余 4 条为 info（§2.1 残留"直入"表述未加行内注 / 证据文档 stage 计数差 7 / decision 文档需随手册入库 / §3.3 无定谳小注）。

| ID | 分类 | 级 | 问题 | 处理 |
|---|---|---|---|---|
| **AR113-1** | **规格缺口（数字失实）** | **低** | §3.7 L933-935 新注：`tas-stg08b2-direct` 的 `stage_no==8` 只在 **f1255–5216**。实测 **C1 首帧=1265**（`oracle/scenarios/tas-stg08b2-direct.txt:28` = `poke 1265 0180 08`；`analysis/traces/tas-stg08b2-direct.frames.jsonl` stage_no==8 min=1265、n=3952）。f1255 是 **C3/C2** 的 poke（各自 scenario L28）。 | 改 "f**1265**–5216（C1；C2/C3 为 f1255–5216）"，或统一写 "≈f1255/1265–5216"；不影响结论 |
| AR113-2 | 规格缺口（残留旧主张未标） | 低/Info | §2.1 L191 I5 行 "**stage8 直入轨**"、L200（四联真内容帧来源）、§3.7 标题 L868/L1316/L1949 仍用"直入"表述且无行内 dated 标记；§3.6/§3.7 顶部更正已覆盖，但 §2.1 读者可能仍按"直入"理解 I5 用途 | 建议 §2.1 表头或 I5 行加 1 行：**【2026-10-18 定谳：直入前提不成立，见 §3.6/§3.7 顶部】** |
| AR113-3 | 报告/证据（信息） | Info | 证据文档 `p0-0-groupD-source-decision.md` §2.2 "C3 全帧 stage_no 计数 {1:26894, 2:65993, 3:23144, 8:3962}" 与实测 **1:26901** 差 7（=f1–7 帧；doc 未注明排除预热帧，四项合计 119,993≠120,000）。手册**未引用该计数**，不影响手册结论 | 证据文档补注"排除 f1–7"或改 26901（l4-analyst 侧） |
| AR113-4 | 纪律/指针（信息） | Info | 证据指针语义正确且 analysis 侧已注"gitignored 可再生"（L713-715；实测 `.gitignore:11 analysis/` 命中）；但 `docs/findings/p0-0-groupD-source-decision.md` 当前 **`??` untracked** ⇒ 若不同 commit 入库，L712 指针在 HEAD 不可解析 | 提交时把 decision 文档 **与手册同 commit** `git add`；可选把 L712 内联 code 改 markdown 链接（docs-links 可校验防断链） |
| AR113-5 | 规格缺口（一致性小注） | Info | §3.3 L659-661 只有 task-48 的"改用 §3.6 重裁选中来源"，**未标 2026-10-18 定谳 `blocked:no-source`**；与任务卡"§0.5/§3.1/§3.3 一致性小注"期望不完全对齐（§0.5 L64/L3.1 L517 已有，§3.3 无） | L661 后加半句（同 L517 风格）：**（2026-10-18 定谳：组 D=`blocked:no-source`，见 §3.6 顶部）** |

---

## 1. 独立复算（不采信被审数字）

### 1.1 五条定谳：decision 文档 vs 手册 vs 实测

| # | 手册 §3.6 顶部更正（L695-715） | decision 文档 | 我的独立复算 | 判定 |
|---|---|---|---|---|
| ① | `$26 bit7` **不是** Ending 判据；C3/C2 目标帧 `flags26=0x00/0x80`、`stage_no=1/2` | §0 表、§2.2 | 目标帧实测（C3/C2 各 6 帧）：`(28500/30000/31500)=0x00/stage1`、`(33000/34500/36000)=0x80/stage2`；C3 `bit7` 首帧 **5217**、共 93,915 帧 | **✓ 一致（A）** |
| ② | "stage8 直入"不成立：`stage_no==8` 只在 f1255–5216 | §2.2/§3.1 | **C3** `tas-stg08-direct`：f1255–5216（n=3962）；C2 `tas-stg07-relay`：stage7 f1255–5216（无 stage8）；**C1** `tas-stg08b2-direct`：**f1265**–5216（n=3952，见 AR113-1） | **✓（C1 起点差 10 帧 → AR113-1）** |
| ③ | blob **0/6**；C3/C2 各 **80 shots 全 NONE** | §0/§2.1 | 6 张 expected sha 重算 6/6 与文档 §2.1 相同；`shots/` 下 C3 80 张 + C2 80 张（unique blob 各 74）逐张 sha vs 6 expected ⇒ **hits=NONE** | **✓ 一致（A）** |
| ④ | C2 **DEGRADED `frame_errors=142149`** | §0/§1 | `capture-tas-stg07-relay.json` done.frame_errors=**142149**；ledger 同；DONE frames_actual=120000/overshoot=false/finalize_by=endFrame | **✓ 一致（A）** |
| ⑤ | C1 **BLOCKED: capture-termination** | §0/§1 | `analysis/repro/p0-0-capture/incident-tas-stg08b2-stall.json`：status=`blocked:capture-termination`、done_exists=false、frames_lines_written=39998/40000、shots=26 | **✓ 一致（A）** |

补充复核（§3.6 块内其它数字）：C3 DONE `frames_actual=120001/overshoot=true/finalize_by=inputPolled-overshoot` ✓（capture JSON）；trace sha `d6944fe9…`/`e12e6ac4…` ✓；C3 全帧 stage_no 分布 `{1:26901, 2:65993, 3:23144, 8:3962}`（sum=120000）✓（AR113-3 仅指 decision 文档少 7）。

### 1.2 dated 更正纪律（原文保留/不删历史）

| 检查 | 结果 |
|---|---|
| §3.6 旧主张保留 | L717-719 明写"**原假设（保留留档，不删）**"；L673-677 以 `~~…~~`+dated 注保留"v5 同余"旧论；L735-736 候选清单保留旧 C1-C5 表并加"用前必读定谳更正" |
| 表格旧值作废方式 | L670/L472-473/L494/L514-517 均 `~~旧值~~` + **已定谳 blocked:no-source（2026-10-18）**；L670 同时保留旧文字（strike） |
| 判据②作废注 | L812 整条判据以 `~~…~~`+【2026-10-18 定谳·判据②作废…】开头，L817 "~~Ending 判定~~【…作废…历史留档，不得再用】"，L843 句尾 `〔2026-10-18 定谳：本句判据作废…〕` —— **清晰不误导** ✓ |
| 历史删除检查 | `git diff` 删除行均为**被重写行**（旧文以 strike 形式回到新行，如 L670/L472-473/L494），未发现原文结论被静默删除；git HEAD 仍可回溯 ✓ |
| 未改 §2.2（任务卡声明） | 工作区 §2.2 区（L277-310）有并行任务改动（文本含 task-105/task-110 判据），**无 task-113 组 D 相关内容**；本复核按任务卡声明将其视为并行任务产物，不计入本卡落点（仅作范围观察，不作作者归属断言） |

### 1.3 影响面同步（4 处 + 2 小注）

| 落点 | 期望 | 实测 | 判定 |
|---|---|---|---|
| §3.7 L933-938（四联降级） | 四联来源降"待重评"；重评前不 staging | L933-938 新块：`stage_no==8` 只在 f1255–5216（**C1 起点应为 1265，AR113-1**）；旧选帧脚本"降级为待重评"，"重评完成前四联 expected 不得按旧脚本 staging" ✓ | **✓（1 数字 → AR113-1）** |
| §7.3 L1348-1353（ending 5 对） | 5 对不 staging/整体不执行/恢复条件 | `blocked:no-source`；"5 对保持旧 expected、不 staging、§7a 替换分支不触发" ✓；恢复条件=§3.6 第 4 条 ✓ | ✓ |
| §7.4 L1373-1379（fade #11） | 同组 D blocked/不 staging | `blocked:no-source`、旧 expected 保留、不 staging、s5-full-inject 尚未采 ✓ | ✓ |
| §8.2 L1635-1641（验收项） | 组 D 现状 blocked:no-source、取得合法来源前维持不通过、P0-0 不得宣称全绿 | 勾选条件（重裁完成=winner 或 6 对 blocked）与 dated 注（**本清单在取得来源前维持不通过**）并存无矛盾（前者=步骤完成，后者=整体验收不通过）✓ | ✓ |
| §0.5 L62-65 | 组 D 定谳分支 | L64-65 "现状即此分支 blocked:no-source"+指针 ✓ | ✓ |
| §3.1 L514-517 | 组 D 表 groupD 定谳 | 已补 blocked+指针 ✓ | ✓ |
| §3.3 一致性小注 | 任务卡期望 | 无新增定谳注（AR113-5） | **Info** |

### 1.4 证据指针可解析 / 门禁

| 项 | 结果 |
|---|---|
| `docs/findings/p0-0-groupD-source-decision.md` | 存在（12,464B，162 行）；**`git status` = `??` untracked** ⇒ 需随手册同 commit 入库（AR113-4） |
| `analysis/repro/p0-0-groupD/{decision,capture-*,blob-compare-C3}.json` | 存在；`git check-ignore` 命中 `.gitignore:11 analysis/` ⇒ **gitignored**，手册 L715 已注"gitignored 可再生" ✓ |
| 手册内联引用 vs markdown 链接 | L712/1353 以内联 code 引用 decision 文档（非链接）⇒ 当前 docs-links 不会因该文件 untracked 而 FAIL；但可选改链接以便断链检查（AR113-4） |
| docs-links | **PASS**（362 md / 454 本地链接全部有效，本卡单门实跑） |

---

## 2. findings 详情（证据指针）

**AR113-1（必改，低）**：证据链——
- `oracle/scenarios/tas-stg08b2-direct.txt:28` = `poke 1265 0180 08`（同文件 assist poke 在 1255）；
- `analysis/traces/tas-stg08b2-direct.frames.jsonl`（C1，40000f，**文件尾部 1 行截断**——即 C1 被 kill 的已知产物）解析：stage_no==8 **min=1265**, max=5216, n=3952；f1255-1264 均 stage_no=1；
- 对比：`tas-stg08-direct.txt:28`/`tas-stg07-relay.txt:28` 为 `poke 1255 0180 08/07`，其 stage 8/7 起点=1255。
- 结论：手册 L934 的 f1255 是从 C3 数值顺手复制到 C1 名称；改为 f1265（或分列）即可。**不改变"跨 run 硬帧号不可用"的判定**。

**AR113-2（Info）**：`直入` 残留出现点（grep）：L191（I5 表）、L200、L230-231（命令注释）、L403/L650（中性描述，不需改）、L596/L609、L868（§3.7 标题）、L1316、L1949；其中 §3.7 L933 顶部注已说明"直入轨前提同样不成立"，但 §2.1 表头无指引。建议只在 §2.1 I5 行加注即可，不必逐处 strike（避免大范围 diff）。

**AR113-3（Info）**：decision 文档 §2.2 计数 vs 实测：文档 sum 119,993；实测 C3 `{1:26901, 2:65993, 3:23144, 8:3962}=120,000`；差值 7 恰为 f1–7（预热帧）。若按"全帧"口径应为 26901，或补"排除 f1–7"。手册未引用该计数，故仅 info。

**AR113-4（Info）**：提交纪律——decision 文档 untracked；若不同 commit 入库，L712 指针在 HEAD 断链（当前工作区可解析）。建议同 commit + 可选链接化。

**AR113-5（Info）**：§3.3 L659-661 有 task-48 更正但无定谳注；与 §0.5/§3.1 已同步风格不一致（极小）。

---

## 3. 维持成立（本审复现，勿动）

- §3.6 L695-715 五条定谳内容与 decision 文档 §0/§2/§3/§5 **逐条一致**（除 AR113-1 的 10 帧起点）；
- §3.6 顶部 dated 块、"原假设保留留档"、判据②作废注（L812/817/843）**清晰不误导**；
- §7.3/§7.4/§8.2/§0.5/§3.1 影响面同步 ✓；§8.2 "步骤完成≠整体通过"的语义无矛盾；
- docs-links PASS；证据指针（除未入库项）可解析；analysis 可再生已注明；
- 未发现被审文档改动 `crates/`/`gates/`/`rom/`/`reference/` 或削弱门禁；仅手册单文件（+证据文档待入库）。

---

## 4. 复现命令（只读；PowerShell 可执行）

```powershell
# 1) expected sha + shots blob 全扫（期望 6/6 与 NONE）
@'
import hashlib, pathlib
EXP=pathlib.Path("oracle/screens/expected"); SH=pathlib.Path("shots")
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
names=["l4_fade-clear","l4_ending-1","l4_ending-2","l4_ending-3","l4_ending-4","l4_ending-5"]
exps={n:sha(EXP/f"{n}.png") for n in names}
print("expected", {n:exps[n][:12] for n in names})
for sc in ["tas-stg08-direct","tas-stg07-relay"]:
    shots=sorted(SH.glob(sc+"_f*.png"))
    hits=[s.name for s in shots if sha(s) in exps.values()]
    print(sc,"shots",len(shots),"hits",hits or "NONE")
'@ | python -
# 2) 目标帧 flags26/stage_no + stage8 范围（C3/C2/C1）
@'
import json, collections, pathlib
for sc in ["tas-stg08-direct","tas-stg07-relay","tas-stg08b2-direct"]:
    tgt={28500,30000,31500,33000,34500,36000,14600,16000,16600,18000,22000,28000}
    rows=[]; s8=[]; b7=[]
    for l in open(f"analysis/traces/{sc}.frames.jsonl",encoding="utf-8",errors="replace"):
        try: r=json.loads(l)
        except Exception: continue
        w=r.get("w",{}); f=r.get("f")
        if f in tgt: rows.append((f,w.get("flags26"),w.get("stage_no")))
        if w.get("stage_no")==8: s8.append(f)
        if isinstance(w.get("flags26"),int) and w["flags26"]&0x80: b7.append(f)
    print(sc,"targets",rows)
    if s8: print("  stage8",min(s8),max(s8),len(s8))
    if b7: print("  bit7 first",min(b7),len(b7))
'@ | python -
# 3) C2 frame_errors / C1 incident
python -c "import json;d=json.load(open('analysis/repro/p0-0-groupD/capture-tas-stg07-relay.json',encoding='utf-8'));print(d['done'])"
python -c "import json;d=json.load(open('analysis/repro/p0-0-capture/incident-tas-stg08b2-stall.json',encoding='utf-8'));print(d['status'],d['done_exists'],d['frames_lines_written'],d['frames_expected'])"
# 4) poke 帧（C1=1265 vs C3/C2=1255）
Select-String -Path oracle/scenarios/tas-stg08b2-direct.txt,oracle/scenarios/tas-stg08-direct.txt -Pattern 'poke 12\d+ 0180'
# 5) 单门 docs-links
python -c "import sys;sys.path.insert(0,'gates');import cli;print(cli.gate_docs_links())"
```

---

## 5. 自查

- 只写本文件；未 git add/commit；未改被审文档/代码/门禁；未跑 Mesen/gates check/status/visual（仅 docs-links 单门）。
- 证据独立重导：expected sha/161 shots/3 条 frames.jsonl/repro JSON/incident JSON/poke 行/decision 文档逐项实读；未采信被审自述。
- 未证不发明：worktree 并行改动（§2.2 等）只作范围观察，不归属作者；decision 文档 untracked 属"待入库"事实，不作越权修改。
- 建议修复顺序：AR113-1（1 行）→ AR113-4（提交时同 `git add` decision 文档）→ 其余 info 可顺手。

【2026-10-18】decomp-verifier，task-117。

---

## 点验（2026-10-18，task-122）

**VERDICT：pass**（AR113-1 修正成立；AR113-2/AR113-5 抽查通过；docs-links PASS；可随手册 + decision 文档同批入库，AR113-4 提醒保留）。

### 1. AR113-1（必核，task-117 唯一必改）—— ✓ 已修

- **手册 §3.7 L937-945 定谳注实测**：C1 `tas-stg08b2-direct` 的 `stage_no==8` 已改为 **f1265–5216**（含证据 `oracle/scenarios/tas-stg08b2-direct.txt:28` 的 `poke 1265 0180 08`、frames 实测 min=1265/n=3952）；**C2/C3 为 f1255–5216**（C2 stage7 min=1255/n=3962、C3 stage8 min=1255/n=3962）✓
- **独立重算**（逐行容错解析；C1 文件尾部 1 行截断为已知 kill 产物）：
  - `tas-stg08b2-direct` stage8：**n=3952 / min=1265 / max=5216**
  - `tas-stg08-direct` stage8：**n=3962 / min=1255 / max=5216**
  - `tas-stg07-relay` stage7：**n=3962 / min=1255 / max=5216**
  - 四数（C1 n/min/max + C3/C2 n/min）与手册新注全匹配 ✓
- **poke 证据**：C1 scenario L28=`poke 1265 0180 08`；C3 L28=`poke 1255 0180 08`；C2 L28=`poke 1255 0180 07` ✓
- **结论**：AR113-1 关闭；task-117 的 f1255→f1265 更正准确，不改变"跨 run 硬帧号选帧不可用"判定。
- 全文残留检查：`f1255–5216` 仅存于 §3.6 的 **C3** 语境（L701/L705，均正确）与本次新注 C2/C3 分列（L940）；**无 C1=f1255 残留** ✓

### 2. AR113-2（抽查 "直入" 括注）—— ✓ 全部到位

- §2.1 **L191** I5 行括注：`〔2026-10-18 定谳："直入"前提不成立，见 §3.6/§3.7 顶部 dated 更正；四联来源待重评〕` ✓
- §2.1 **L203-204**：新增定谳注（C1 `stage_no==8` 仅 f1265–5216、目标帧实为 stage1/2）✓
- §3.7 **标题 L872**：`（〔2026-10-18 定谳："直入"前提不成立，见本节末 dated 更正〕）` ✓
- §7.2 **L1323-1325**：`〔2026-10-18 定谳："直入"前提不成立（C1 stage_no==8 仅 f1265–5216…）⇒ 本节选帧来源待重评，见 §3.7 末 dated 更正；重评前不得按旧硬帧号 staging〕` ✓
- §15.1（"§3.7 影响"段）**L1958-1960**：`〔2026-10-18 定谳："直入"前提不成立——C1 stage_no==8 仅 f1265–5216…〕` ✓
- 括注均指向 §3.6/§3.7 的 dated 更正、不误导；未发现括注内新错误（f1265/n=3952 与实测一致）。

### 3. AR113-5（抽查 §3.3 定谳半句）—— ✓

- §3.3 **L661-665** 已补：`【2026-10-18 定谳】组 D 现状=blocked:no-source（C3/C2 blob 0/6、C2 DEGRADED、C1 BLOCKED；见 §3.6 顶部更正块/§8.2）——当前无适用组 D 来源，③ 校验无对象` ✓（与 §0.5 L64/§3.1 L517 风格一致）。

### 4. 门禁 / 范围 / 纪律

- **docs-links PASS**（365 md / 454 本地链接全部有效，单门实跑）。
- 本卡 diff 落点 = AR113-1/2/5 对应处（+191、+203、+664、+872、+937..+945、+1323..+1325、+1958..+1960）与 task-113/并行 §2.2 既有改动；**未发现 task-119 越界改 §2.2 或其它无关节**（§2.2 仍为 task-110 判据文本）。
- **只读 + 单门**；未跑 Mesen/gates check/status/visual；未改被审文档；本文件为唯一写入、未 git add/commit。

### 5. 剩余（info，不阻塞本卡 pass）

- **AR113-3**（open，证据文档侧）：decision 文档 §2.2 C3 计数 26894 vs 实测 26901（差=f1–7 预热帧）；手册未引用，建议 l4-analyst 顺手改/注。
- **AR113-4**（提交时执行）：`docs/findings/p0-0-groupD-source-decision.md` 当前 `??` untracked，须与手册**同 commit** 入库（可选把 §3.6 L712 内联 code 改 markdown 链接）。
- AR113-2 的"直入"历史/中性用法（§2.1 L200、§3.7 正文、§15.1 标题等）已被括注 + dated 注覆盖；按"原假设留档不删"纪律**不再改动**。

**点验 VERDICT：pass** —— 任务卡要求的三项落点全部可核；可进入入库（随手册 + decision 文档同批）。
