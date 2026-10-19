# 对抗复核·族①首迭代（MODELGEO 门控 + p0-1-family-report）

> 【2026-10-20】verifier=独立对抗复核子 agent（mimo，证伪立场，未参与实施）。
> 范围：render_frame.rs MODELGEO 门控、A/B 数字、L4 回归、报告结论、改动面。
> 方法：全部现成产物复算（`diff_visual.compare` 数 len(samples) + 渲染日志 out=/nt_nz 对账），
> 未跑 cargo、未渲染、未跑 gates。

## VERDICT: **PASS**

四个指定 diff 数字逐一复算与文档完全一致（2352/2138/3742/4054）；MODELGEO 几何与 A 级模型
描述逐项相符且默认=legacy；legacy 路 out= 哈希与签署 actual 逐字节自证；结论"混合负结果/停试错/
回退默认"无夸大。仅有 3 条待补强项（证据卫生类，均不阻断）。

## 1. 四个 diff 数字（独立复算，零渲染）

| # | 对比（expected × render） | 复算值 | 文档值 | 一致 |
|---|---|---:|---:|:--:|
| 1 | `l4_stg02-start.png` × `p01-legacy/stg02-start.png` | **2352** | 2352 | ✓ |
| 2 | `l4_stg02-start.png` × `p01-modelgeo/stg02-start.png` | **2138** | 2138 | ✓ |
| 3 | `gameplay-stage1_f01260_stage-start.png` × `p01-legacy/gameplay-stage1_f01260.png` | **3742** | 3742 | ✓ |
| 4 | `gameplay-stage1_f01260_stage-start.png` × `p01-modelgeo/gameplay-stage1_f01260.png` | **4054** | 4054 | ✓ |

附加（报告第 3 行 clear-inject 对，也逐一复算）：`l4_clear-inject.png` × `g_run1/g_run2`=**58159**（before），
× `p01-run1/l4_clear-inject.png`=**58169**（after）→ Δ=+10 与报告一致。模式归属经日志对账确证：
g_run（基线）clear-inject `nt_nz=955 out=246dd5712608`；p01-run1 s3（MODELGEO）`nt_nz=750 out=56709818b6be`；
legacy 路复跑 `out=27fa37373353`(stg02) / `c39bbeffe098`(gameplay) 与 ledger 已知 actual 全同
⇒ 默认路逐字节回归成立、58169 确为 MODELGEO=1 产物。

## 2. findings 四分类

### ① 已证实（Verified，A 级）
- **MODELGEO 几何与 A 级模型逐项相符**（`git diff` 直读）：base=`((hi<<8)|lo)+0x0F`；4 组×8B、
  组间步进 `$20`、源 `row[grp*8+j]`；attr 行=`(((hi|3)<<8)|c)+$0F` 8B 拷入 attr 表（idx+8≤64 边界守卫）。
  与报告 §1 模型（bank15:$F040-F0DA 镜像）表述一致。
- **默认=legacy**：`MODELGEO` 未设 ⇒ `model_geo=false` 走旧 32 连写路；HUD LUT 路恒 `false`（legacy 形态）。
- **FLAGUPLOAD 路/阈值/expected 未动**：diff 仅触及 pending_row 元组+两个几何分支+赋值点；
  FLAGUPLOAD 互斥门与经验拟合路零改动；`oracle/`、`gates/`、`tools/`、expected 无任何改动（git status 佐证）。
- **数字与结论无夸大**：attack-plan dated 注（2352→2138 / 3742→4054 / 58159→58169）与 family-report §3
  表格均与复算逐字一致；"混合、非单调改进/未胜出/按 §3.5 停试错/回退默认=legacy"为如实表述，
  未把负结果包装为改进。
- **L4 回归全绿**：`python -X utf8 tools/test_l4_render_all.py` → 11/11 PASS（exit 0）。

### ② 误报 / 过度声明（False positive）
- 无。未发现任何被审方结论超出其证据。

### ③ 待补强（Minor，不阻断）
1. **渲染日志不自描述模式**：`env={}` 字段只记录既有 env 清单，不含 MODELGEO，A/B 产物的模式归属
   需靠 out=/nt_nz 排除法推定；且 `p01-run1/` 一个目录混放两种模式（s1/r1/r2=legacy、s2/s3=MODELGEO，
   末次写入决定 png 归属）。建议：env 日志增记 MODELGEO、按模式分目录。
2. **clear-inject "before=58159" 用的是 10-18 g_run 基线产物**，非新 binary 默认路复渲；clear-inject 的
   默认路逐字节一致性目前是"代码路不变 + stg02/gameplay out= 哈希全同"的推定（B 级），非直接实测。
   建议补一次新 binary 默认 clear-inject 复渲闭环（可留待下次渲染窗口）。
3. **测试数漂移**：任务口径 10/10，实际套件 11 项（新增 `test_v2_run1_sha_allowlist_semantics`），
   11/11 全绿——非破坏，仅记录口径差。

### ④ 建议（Suggestion）
- 下一迭代前先落取证卡（t93 四字段映射 (x,x+1,x+16,x+17) / $0160 批次语义 v=1/2/3）再动几何，
  与报告 §5 优先级一致；MODELGEO=1 保留为取证通道的做法合理。
- 报告 §1 "209,086 px（74.63%）"未在本次零渲染快速范围内独立复算，按"沿用已接受基线数字"处理。

## 3. 改动面核对（git status）

- `crates/game-desktop/src/bin/render_frame.rs`（未暂存，+41/−4）
- `docs/findings/l4-zero-tolerance-attack-plan.md`（未暂存，dated 注 +7）
- `docs/findings/p0-1-family-report.md`（未跟踪新文件）
- `docs/findings/team-ops-ledger.md`（**已暂存**，00:59 巡查记录 +10）
- `docs/findings/adversarial-review-v2-allowlist.md`（未跟踪，他批判稿，非本批）
- **红线路径零改动**：`rom/`、`oracle/`、`gates/`、`tools/`、`data/`、expected 均不在改动面 ✓。
  口径提示：任务书"2 findings"若不含 team-ops-ledger 则实为 3 份 docs + 1 份复核稿；无越界文件。
- 本次复核未 git add/commit（唯一写入=本文档）。

## 4. 一屏总结

门控实现=A 级模型逐项镜像且默认 legacy ✓；4 个指定数字 2352/2138/3742/4054 全部复算命中 ✓；
clear-inject 58159→58169 亦复算命中且模式归属经 out=/nt_nz 确证 ✓；legacy 默认路与签署 actual
逐字节自证（27fa3737/c39bbeff）✓；FLAGUPLOAD/阈值/expected 零改动 ✓；L4 回归 11/11 全绿 ✓；
报告与 attack-plan 数字/结论无夸大 ✓。**VERDICT=PASS**，3 条 minor（日志模式自描述、
clear-inject 默认路补渲、测试数口径）留档跟进。


## 【2026-10-20 01:12 dated 更正·归属】
本文档实际为**独立复核子 agent（8b3c4703）**产出（VERDICT=PASS，含 clear-inject 渲染日志
out=/nt_nz 模式归因等独立证据）；commit 3606dfa 说明误写"Lead 独立重导代审"系写入竞态所致
（Lead 代审稿 write 被 fs 守卫拒、未落盘；子 agent 版先行入库）。**归属以本更正为准：
复核方=独立 mimo 子 agent，实施≠复核严格成立。** 复核 minor②（默认路 clear-inject 复渲闭环）
已由 Lead 补渲核销：diff=58159 与基线全同。
