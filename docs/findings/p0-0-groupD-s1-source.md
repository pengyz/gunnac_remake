# P0-0 冻结前置①：S1 dense 采集设计 + 四联/#17/#18 expected 来源裁定 + #14 定位（task-154）

> 日期：2026-10-18｜执行：l4-analyst｜任务：task-154（Lead 裁定"冻结硬门槛=§8.2 全绿"后的前置①）
> 输入：GUI 重采 trace/shots（task-133，input-effective；`analysis/traces/`、`shots/`）+ 旧 expected（`oracle/screens/expected/`，只读）
> 边界：新 oracle 场景=**先出 diff 批准后才 commit/跑**（本报告 §1 diff 待批）；不改代码/gates/threshold；不跑 gates check/status/visual；Mesen 锁归本卡。
> 并行：platform-porter（manifest 面）/spec-writer（文档面）互不冲突；本卡只写 `docs/findings/p0-0-groupD-s1-source.md` + 新增 untracked 场景草稿。

## 0. 结论摘要

| # | 事项 | 状态 |
|---|---|---|
| 1 | **S1 dense 双网格采集**（复用 C3 输入逐字节不变；300f 两跑 + 100f 单遍） | **300f：两跑 trace `2c7962ec…`=C3 原 run、shots 406/406 同 ⇒ 确定性 PASS；100f：1204 张单遍（trace 同）**；4 blob **双网格均零命中** ⇒ 4 对 `blocked` **终态**（§1.1/§1.2）；两个新场景文件均待 Lead commit |
| 2 | **四联 #16/#19/#20/#21 expected 来源**（`62a8ef06`） | **已找到（A 级）**：stg08-timeline GUI f16500–33900 共 **64 帧**逐字节同；逐对 canonical 帧 f16600/f18000/f22000/f28000 全 MATCH ⇒ `replaced-invalid-asset → valid-with-source`（待 §7.7 安装核验） |
| 3 | **#17 `l4_stg07-start`（`ddb322ac`）/ #18 `l4_stg07-combat`（`2824cb13`）expected 来源** | **已找到（A 级）**：stg08-timeline GUI **f14600 / f16000** 逐字节同 ⇒ 同上 valid-with-source（待 §7.7） |
| 4 | **#14 `l4_stg02-start`** | 旧 expected `788d6984`（258B）= **frame-mismatch（错帧覆盖；与 title boot-black 逐字节同，f3404 在原作/重采都是过场黑屏，非伪造）**；真实 STG-02 开始屏已定位：**gs5-stg01-clear-anchor f3450 `stg02-start`**（`44e7800d`，1411B，「地区-2」开始屏，见 §3）⇒ **Lead 裁定 Option B：pair 重指向 gs5-anchor-render:3450 + actual 重渲**（见 `p0-0-expected-install-plan.md` §2/§2.1） |
| 5 | 组 D 4 blob（fade-clear/ending-1/2/3） | 现有 **907 张** GUI shots 全扫**零命中**；S1 dense 300f 网格（120000f→400 张+7 显式）待批准后跑 |

> **冻结判定**：§8.2 仍**不满足**（expected 安装未执行、4 blob 无源待 S1、strict 未跑等）；本卡只恢复 expected **可复现性证据**，不改 expected/门禁。

## 1. S1 dense 场景设计（Lead 已批准 300f 版；文件：`oracle/scenarios/tas-stg08-direct-dense.txt`，untracked 待 commit）

**设计**：以 C3 `tas-stg08-direct`（120000f）为底，**输入前缀/assist poke/bot/watch/probe_mem 逐字节不变**；仅改 `name` + `capture_every 1500→300` + 7 张显式 capture。diff（`git diff --no-index`）：

```diff
-name tas-stg08-direct
+name tas-stg08-direct-dense
-capture_every 1500 probe
+capture_every 300 probe
+capture 27036 stg08-clear
+capture 27701 gate300-open
+capture 27731 all-clear-180
+capture 27734 all-clear-95f9
+capture 28000 ending-early
+capture 29000 ending-mid
+capture 30000 ending-late
```
（另仅首行注释改为 C4 dense 用途说明；其余 60 行不动。）

- **目的双用**：①组 D 4 blob（`14802bcf` fade-clear / `19b88b1f` ending-1 / `0f8e1169` ending-2 / `f319f1fe` ending-3）any-frame 搜索（含 ending 前 fade 窗）；②L5 G-L5-4 ending 锚（f27036/f27701/f27731/f27734 为已证链帧，见 `l5-gui-trace-feasibility.md` §2）。
- **采样预算**：120000f / 300 = **400 张**（+7 显式、与 capture_every 帧可能重合自动去重）；相比现有 1500f 网格（80 张）加密 **5×**；若仍零命中，可再出 100f 变体（1200 张）作二轮。
- **判据**：`rc=0 ∧ done≠null ∧ ¬timed_out ∧ trace_lines==done.trace_lines ∧ frame_errors==0` + 输入有效性（`controller_held` 非零、`stage_no` 到 9）；**跑两遍核确定性**（同脚本两跑 trace sha 相同；shots sha 相同）。
- **any-frame 判定**：133→**407 张**全量 sha 对 4 blob 比对；命中 ⇒ 记帧号+shot 名+trigger=injected（源场景含 assist poke，如实）；零命中 ⇒ 4 对维持 blocked（负证据升级为 **300f 网格**）。
- **成本**：GUI 120000f @1000% ≈ 6min/跑 ×2 = ~12min（+截图开销）。**批准后**：commit（`feat(oracle): C4 dense overlay tas-stg08-direct-dense（组 D 4 blob any-frame；inputs 不变）` + Evidence）→ 跑两遍 → 回报。

### 1.1 S1 采集执行与结果（2026-10-19 00:33 定稿；dated）

**有效 run（两遍，确定性 PASS）**：

| run | 时间 | 帧/trace | done | trace sha256（前 16） | shots | 输入断言 | wall |
|---|---|---|---|---|---|---:|---|
| A | 23:54:52–00:08:25 | 120000/120000 | `frames_actual=120000, overshoot=false, finalize_by=endFrame, frame_errors=0` | `2c7962ec97c66107` | 406 | held 22512 帧、stage [1,8,9] | 350.9s |
| B | 00:26–00:33 | 120000/120000 | 同上 | `2c7962ec97c66107` | 406 | held 22512、stage [1,8,9] | 354.8s |

- **两跑确定性**：trace sha **相同**（`2c7962ec97c661079dc4f0aa88bbaa36a86c99bc7a3fee4b12c291d08e0a8480`）、**406/406 shots sha 集合同** ⇒ PASS。
- **输入不变印证**：dense trace sha 与原始 C3 `tas-stg08-direct` GUI run **逐字节相同**（`2c7962ec…`，41,196,062B）⇒ 仅改 capture 面、输入/行为未变（B 级）。
- 产物：run A 归档 `analysis/traces/.archive/tas-stg08-direct-dense-20261019-001941/`（trace/done/shots 406）；run B 当前 `analysis/traces/tas-stg08-direct-dense.*` + `shots/tas-stg08-direct-dense_*.png`（406）；汇总 `analysis/repro/recapture-gui/s1-dense-run5.json`（gitignored）。

**any-frame 4 blob 搜索（dense 300f）**：406 张全扫 → **`l4_fade-clear`/`l4_ending-1`/`l4_ending-2`/`l4_ending-3` 全部 0 命中**（连此前 907 张共 1313 张 GUI shots 均零命中）。⇒ **组 D 4 对仍 `blocked:no-source`**，负证据升级为 **300f 网格**；按 Lead 指示，下一步可二轮 capture_every=100（1200 张）或判本链无此 4 屏。

**7 显式锚点（帧/capture tag/shot sha/watch 语义值；均存在）**：

| 帧 | tag | shot sha（前 12） | stage_no | flags26 | flags27 | gate300 | trig_ca | countdown2f | pstate |
|---:|---|---|---:|---:|---:|---:|---:|---:|---:|
| 27036 | stg08-clear | `6fcebbc761ff` | 8 | 16 | 128 | 0 | 5 | 50 | 1 |
| 27701 | gate300-open | `d3113aab429b` | 8 | **0（bit4 清）** | 128 | **16** | 5 | 0 | 1 |
| 27731 | all-clear-180 | `788d6984576b`（黑屏） | **9** | 0 | 128 | 16 | 5 | 0 | 1 |
| 27734 | all-clear-95f9 | `788d6984576b`（黑屏） | 9 | **128** | 128 | 16 | 5 | 0 | 1 |
| 28000 | ending-early | `d1bea1323e82` | 9 | 128 | 128 | 16 | 5 | 103 | 44 |
| 29000 | ending-mid | `a6ae0d537fc0` | 9 | 128 | 128 | 16 | 5 | 9 | 44 |
| 30000 | ending-late | `11b703ce61a5` | 9 | 128 | 128 | 16 | 5 | 164 | 44 |

> 语义核：f27036=`$27 bit7`+`$26 bit4` 关末入口；f27701=`$0300=0x10` 且 `$26 bit4` 清；f27731=`$180==9`；f27734=`$26:=0x80`（`$95F9` 入口，归属 B 级）；f28000+ = ending（`pstate=44`）。
> 注：f27731/f27734 截图是黑屏（过渡帧），L5 锚应以 **watch 值**为准（或视觉锚取 f28000+）；f36000 不在 A/B 窗口（ending 段 f27734–32034）。

**环境干扰与失败尝试（可复算性风险，如实记录）**：

| 时间 | 尝试 | 结果 | 原因/证据 |
|---|---|---|---|
| 23:54:52 | run1 | 失败（wall 460s，trace 303 行，done=None） | Mesen **重复装载脚本**（lua.log 两次 `script load`：game_frame=13 后 reload game_frame=1/`align_gate=0`）；归档 `…-000233` |
| 23:54:52 | run2 | **成功**（trace `2c7962ec`、406 shots） | 归档 `…-001941`（后作为 run A 参照） |
| 00:09:06/00:09:27 | 2 次 | 失败（~21s，无 lua.log） | **外部项目 `D:\workspace\GuardicGaidenRemake` 的 Mesen 占 `Documents/Mesen2`+`Preferences.SingleInstance=True`**（其进程 00:08:57 起、boot-irq-settle；00:10:32 释放） |
| 00:10:36/00:10:57 | 2 次 | 失败（21s / 129s trace=34337） | 同上干扰窗口 + **`ScriptTimeout=1s` 随机杀脚本**（129s 版 120 shots 后中断） |
| 00:19:41–00:23:58 | attempt2 | 失败（trace 27MB 部分，无 run.json done） | 运行中**进程被外部终止**（00:23:58；无 Mesen/驱动残留，原因未定） |
| 00:26:03/24/45 | attempt1-3 | 失败（~21s，无 lua.log） | **启动即退**（无外部 Mesen 残留；疑似单实例/设置文件重写竞态，原因未定） |
| 00:26–00:33 | attempt4 | **成功**（run B） | `ScriptTimeout=60` + speed=1000 下稳定 |

> 缓解：S1 采集期间临时 `EmulationSpeed=1000`、**`ScriptTimeout=1→60`**（避免截图回调 >1s 被 Mesen 杀脚本）；结束后恢复 100/1。
> settings sha 链（内容核对均为 speed/allowio/autosave 正确值）：任务前 `f9c3ffc5…` → 中间 `5886325646b98dcf…`/`9bf908354d0d7172…` → **最终恢复 `e46d001c9564eb0d…`（EmulationSpeed=100、ScriptTimeout=1、AllowIoOsAccess=true、AutoSave/Rewind=false）**；字节 sha 变化含 Mesen 自身重写 settings 的格式差异。

### 1.2 dense100 二轮搜索（2026-10-19 00:42；4 blob 终局负证据）

- 场景 `oracle/scenarios/tas-stg08-direct-dense100.txt`（untracked；相对 `…-dense` **仅 name + `capture_every 300→100`**，输入/assist/watch/probe_mem 逐字节不变）；
- 1 遍有效 run：`frames=120000/trace_lines=120000/done.frame_errors=0`、trace sha `2c7962ec…`（与 C3/300f 同）、**shots 1204 张**、held 22512、stage [1,8,9]、wall 339.1s（attempt1 出现一次 20.8s 启动失败后重试成功——同 §1.1 的外部干扰/启动竞态，非场景问题）；
- **any-frame 4 blob：零命中**（1204 张；`14802bcf`/`19b88b1f`/`0f8e1169`/`f319f1fe` 全无）⇒ 与 300f 406 张合计 **1610 张 dense shots 双网格零命中**；
- 7 锚点仍全部存在（sha 与 300f 版逐字节同：f27036 `6fcebbc761ff`、f27701 `d3113aab429b`、f27731/27734 `788d6984576b`、f28000 `d1bea1323e82`、f29000 `a6ae0d537fc0`、f30000 `11b703ce61a5`）；
- settings：采集期临时 speed=1000/ScriptTimeout=60 → 恢复 **speed=100/ScriptTimeout=1**（sha `e46d001c9564eb0d…`）。
- **结论（Lead 口径）**：4 blob 在本链 **300f+100f 双网格均无来源** ⇒ **`l4_fade-clear`/`l4_ending-1`/`l4_ending-2`/`l4_ending-3` 4 对 `blocked:no-source` 终态**（负证据=双网格 1610 张）；如需再战只能换链路/换采集源（非本卡）。

## 2. expected 来源裁定（四联 + #17/#18；A 级逐字节复现）

方法：GUI 版 `stg08-timeline` run（input-effective；trigger=**injected**，场景含 `poke 2400 $26:=$10` + `poke $0400:=01`）的全部 shots 与旧 expected 逐字节比对；用 `tools/diff_visual.compare`/sha256 复核。

| expected（旧件） | sha256（前 16） | 字节 | 来源场景/帧 | 来源 shot | 复现结果 |
|---|---|---|---|---|---|
| `l4_stg06-late` | `62a8ef06c4b9bcd1` | 6074 | stg08-timeline **f22000**（64 帧窗 f16500–33900 任取） | `stg08-timeline_f22000_stg06-late.png` | **MATCH**（逐字节） |
| `l4_stg08-start` | `62a8ef06c4b9bcd1` | 6074 | stg08-timeline **f16600** | `stg08-timeline_f16600_stg08-start-approx.png` | **MATCH** |
| `l4_stg08-combat` | `62a8ef06c4b9bcd1` | 6074 | stg08-timeline **f18000** | `stg08-timeline_f18000_stg08-combat.png` | **MATCH** |
| `l4_stg08-late` | `62a8ef06c4b9bcd1` | 6074 | stg08-timeline **f28000** | `stg08-timeline_f28000_stg08-late.png` | **MATCH** |
| `l4_stg07-start`（#17） | `ddb322ac57021bc6` | 3930 | stg08-timeline **f14600** | `stg08-timeline_f14600_stg07-start-approx.png` | **MATCH** |
| `l4_stg07-combat`（#18） | `2824cb134987c7b3` | 3755 | stg08-timeline **f16000** | `stg08-timeline_f16000_stg07-combat.png` | **MATCH** |

- **四联同 blob 事实**：f16500–33900 共 **64 帧**全部 = `62a8ef06…`（OBS-1 冻结屏；见 `l5-gui-trace-feasibility.md` §2/`p0-0-same-blob-adjudication.md`）；四联 expected 同 blob 属真实同画面，逐帧可复现。
- **裁定**：上述 6 对 expected **可复现性恢复**；`status` 应从 `replaced-invalid-asset` → **`valid-with-source`（待 §7.7 安装核验）**；`expected_source` 建议：
  `l4_stg06-late=stg08-timeline:22000:stg06-late`、`l4_stg08-start=…:16600:stg08-start-approx`、`l4_stg08-combat=…:18000:stg08-combat`、`l4_stg08-late=…:28000:stg08-late`、`l4_stg07-start=…:14600:stg07-start-approx`、`l4_stg07-combat=…:16000:stg07-combat`；**trigger=injected**（如实，不冒充 natural）。
- **额外发现（bonus，不阻塞）**：`l4_stg03-start` expected（`24a4cb45`，1197B）可在 **stg03-timeline f5700**（`stg03-timeline_f05700_stg03-start-approx.png`）与 **stg08-timeline f5700** 逐字节复现 ⇒ #15 也具备 same-source 证据（本卡不展开，供 spec-writer 记账）。
- **安装纪律**：本裁定只给**来源证据**；按 §7.7 唯一安装点执行（旧件归档 + 落新 + manifest `expected_sha` 回填 + mapping `expected_source/status/trigger`）。**未安装前 §8.2 不得勾选 expected 项**。

## 3. #14 `l4_stg02-start` 定位（旧 expected=frame-mismatch 错帧覆盖，非伪造）

> **【dated 更正 2026-10-19】** 旧 expected 定性由「boot-black 复制品/invalid」更正为 **frame-mismatch（错帧覆盖）**：f3404 在原作与 GUI 重采中均为过场黑屏（`788d6984…`），旧 expected 是**覆盖错帧**而非伪造；真开始屏在 f3450。Lead 已裁定 #14 pair 采用 **Option B**（重指向 gs5-anchor-render:3450 + actual 重渲），执行前置见 `p0-0-expected-install-plan.md` §2.1（gs5-anchor trace 缺 probe/RAM blob，需 render-ready overlay 采集）。

**旧 expected 无效（A 级事实）**：
- `oracle/screens/expected/l4_stg02-start.png` = **258B**，sha `788d6984576bb134…`，与 `title-menu_f00180_boot-black.png` **逐字节相同**（同 blob）；
- GUI 重采后同样 258B 黑屏出现在 `stg08-timeline_f03404_stg02-start.png`、`stg08-timeline_f16200_probe.png`、`gs5-stg01-clear-anchor_f03404_stg02-start-intro.png`（均是黑屏/near-black），**不是关名屏**。

**真实 STG-02 开始屏（已定位，A 级图像+来源）**：
- **`shots/gs5-stg01-clear-anchor_f03450_stg02-start.png`**：sha `44e7800dac894205…`（**1411B**），图像内容 = **"地区-2" 开始屏**（HUD+关名，人工目视复核）；
- 链路：gs5-anchor GUI run（trigger=injected：`poke 2400 $26:=$10`）f2400 关末门 → 商店 → 出店（f3400 exit-fade4）→ **f3450 stg02-start**（capture tag 内置）；同 run f3600 `stg02-start-late`=`586db858…`（1546B），与 stg03-timeline f3600 同 sha（跨场景一致）。
- **建议 §7.7 安装源**：`gs5-stg01-clear-anchor:3450:stg02-start`（`trigger=injected`；`expected_source` 记该字符串；capture_tag=`stg02-start`）。
- **备选（链更原生）**：`stg02-timeline` 场景已存在（frames=19000，capture tags：3404 `stg02-start-intro`/3450 `stg02-start`/3900 `stg02-combat`），但**尚未 GUI 重采**（现存 shots 为 headless 版）——若要求"关 2 战斗段自然执行"的证据源，可加一次 GUI 采集（~2min）后取 f3450；当前先用 gs5-anchor f3450（同源前缀+同 capture tag，图像已验证）。

## 4. 组 D 4 blob 的 any-frame 搜索现状（S1 前后）

- 现有 GUI 版 **907 张** shots（14 场景：含 stg08-timeline 125、C1 26、C2 80、C3 80、tas-stg08b2-dense 133 等）全量 sha 扫 4 blob：**零命中**（`14802bcf`/`19b88b1f`/`0f8e1169`/`f319f1fe`）。
- 已有采集网格：C1/C2/C3 = capture_every 1500（80/80/26 张）、tas-stg08b2-dense = 300（133 张，但只覆盖 40000f 的 C1 帧域且已零命中 fade/ending-1/2/3）。
- **S1 结果（dense 300f 406 张 + dense100 100f 1204 张，输入逐字节同、trace 均 `2c7962ec…`）**：4 blob **双网格均零命中**（dense 合计 1610 张；连此前 907 张共 2517 张 GUI shots 零命中）
  ⇒ **4 blob 在本链无来源（终局负证据）**；4 对 `blocked:no-source` 终态。

## 5. 对 §8.2 冻结硬门槛的影响（"全绿"仍不满足）

| §8.2 项 | 现状 | 本卡贡献 |
|---|---|---|
| expected 安装（§7.7 在 §5 前） | 未执行 | 四联/#17/#18 来源已恢复、#14 源已定位（待安装核验） |
| 组 D 4 对来源 | blocked（4/6 终态） | dense 300f（两跑确定性）+dense100 100f **双网格零命中** ⇒ 4 blob 本链无来源，负证据终态；§8.2 该项**无法转绿**（需换链路/用户裁定） |
| two-run 确定性 | 18 对已定（P0-0-2） | 不变 |
| 去重/registry | expected_unique=13/actual_unique=14（task-143/144 复核过） | 四联/#17/#18 从 invalid 转 valid 后 expected 侧去重口径需同步（spec-writer 记账） |
| strict 视觉/门禁 | 未跑（口径：P0-0 不翻默认） | 不变 |

> 结论：**本卡不触发冻结**；§8.2 仍需 S1 结果 + §7.7 安装 + strict 记录后才可能全绿。

## 6. 证据指针与复算命令

- 场景草稿：`oracle/scenarios/tas-stg08-direct-dense.txt`（untracked；diff 见 §1；**未 commit/未跑**）。
- 复算 expected 来源（只读）：
  ```powershell
  python - <<'PY'
  import hashlib, pathlib
  sha=lambda p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
  pairs=[("l4_stg06-late","stg08-timeline_f22000_stg06-late.png"),
         ("l4_stg08-start","stg08-timeline_f16600_stg08-start-approx.png"),
         ("l4_stg08-combat","stg08-timeline_f18000_stg08-combat.png"),
         ("l4_stg08-late","stg08-timeline_f28000_stg08-late.png"),
         ("l4_stg07-start","stg08-timeline_f14600_stg07-start-approx.png"),
         ("l4_stg07-combat","stg08-timeline_f16000_stg07-combat.png")]
  for e,s in pairs:
      print(e, sha(f"oracle/screens/expected/{e}.png")[:16], sha(f"shots/{s}")[:16],
            sha(f"oracle/screens/expected/{e}.png")==sha(f"shots/{s}"))
  PY
  ```
- #14 对照：`sha256sum` 比较 `expected/l4_stg02-start.png`、`expected/title-menu_f00180_boot-black.png`、`shots/gs5-stg01-clear-anchor_f03450_stg02-start.png`、`shots/gs5-stg01-clear-anchor_f03404_stg02-start-intro.png`（§3 已列 sha/字节）。
- 4 blob 全扫：`python` 遍历 `shots/*.png` 对 4 expected sha（907 张零命中，§4）。

## 7. 验收自查

- [x] S1 场景 diff 已出（§1），输入逐字节不变；**未 commit/未跑**（待 Lead 批准）；
- [x] 四联/#17/#18 expected 来源 A 级逐字节复现（§2），status 处置建议明确；
- [x] #14 旧 expected 无效证明 + 真实 stg02-start 帧定位（§3）；
- [x] 4 blob any-frame 现状（907 张零命中）与 S1 采样预算（§4）；
- [x] 对 §8.2 冻结门槛影响如实（§5：不触发冻结）；
- [x] 300f 两跑确定性 PASS + 100f 单遍 1204 张；4 blob **双网格零命中**；7 锚点双版本同 sha 已核（§1.1/§1.2）；`dense100` 场景待 Lead commit；
- [ ] 场景 commit（C4，Lead）+ 独立复核 pass；docs-links 提交前复核。

## 8. 安装执行记录（2026-10-20 · task-165 Option B；dated）

- 来源裁定已落地：四联/#17/#18 bytes 不变、登记 `expected_source=stg08-timeline:<f>:<tag>`、trigger=**injected**（更正）、status ok/documented-duplicate；
- #14：旧 expected `788d6984…`(258B, frame-mismatch 黑屏) 归档 `oracle/screens/expected/.archive/20261020-000358/`；新 expected **`44e7800d…`(1411B 真「地区-2」屏)**；pair 重指向 `gs5-anchor-render:3450`（overlay commit `86e18d4`）；
  actual=overlay trace f3450 重渲两跑（`27fa3737…`/`436120b6…`，确定性 PASS），**diff=2352 px（ratio 0.038281）**；
- 安装后：expected full unique **18**（原 17）/rendered expected_unique **14**（原 13）/actual_unique 14；replaced-invalid-asset **5→0**；blocked **4 不变**（本报告 §1.1/§1.2 双网格负证据终态）；
- 工具验证：`MANIFEST pairs=22 rendered=18 blocked=4 replaced=0 documented_duplicate=5; TWO-RUN PASS 18/18; DEDUPE expected 14=18-4 actual 14=18-4 registry_ok=True PASS; VALIDATION PASS`；
- 执行细节/证据表见 `p0-0-expected-install-plan.md` §8；4 blocked 不安装；strict/visual 由 §8.1 冻结卡统一。
