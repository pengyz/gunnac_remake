# 工作全集盘点（rewrite-workset）—— 分母 + 分类 + 移植状态

> 状态：2026-10-15 v1（T1，decomp-analyst）。
> 用途：给改写流水线（porter）一个**进度可算**的分母——子程序入口全集（call-graph/入口枚举）
> × 机械画像分类（引擎/数据/图形/音频）× 移植状态（L2/core/spec/findings/未动）。
> 机器可读镜像：`data/status/rewrite-workset.json`（schema 1）；生成器：`tools/workset_inventory.py`
> （幂等重导，逐段指纹可独立复核）。完备性矩阵符号面（[completeness.md](completeness.md) §3 占位）
> 由本文件落地为**全集清单**。
>
> **口径铁律**：本文的“分类”是**机械画像**（A 级字节足迹 + B 级结构推理，规则随表列出），
> **不是语义定谳**；语义命名仍走 `data/naming/ledger.jsonl` 取证轨道。任何与 ROM 行为冲突处以 ROM 为准（D3）。

## 0. 进度怎么算

- **段（segment）**= 本盘点的基本工作单元 = 入口地址序切片（§1.1 切分约定）。
  每段带：分类（class）、移植状态（status）、指纹（footprint）、入口来源可信度（entry_trust）。
- **进度双口径**（§4 给当前读数）：
  - 段数口径：`status ≥ core-cited 的段数 / 394`；
  - 字节口径：`status ≥ core-cited 的段字节 / 131,072`。
- 状态梯度单调向好：`unexplored → findings-cited → spec-cited → core-cited → L2-diff`
  （后者包含前者的可迁移证据强度；**L2-diff = 有 ROM↔game-core 差分对拍**）。

---

## 1. 分母：子程序入口全集（call-graph / 入口枚举）

### 1.1 枚举规则与切分约定

**入口枚举（A 级，逐条可回 `reference/seed/gunnac.asm` grep 重导）**：

| 规则 | 内容 | 计数 |
|---|---|---|
| E1 | 每 bank 段起点（窗口基址） | 16 |
| E2 | JSR/JMP 绝对目标（seed 操作数 `sub_XXXX` / `loc_XXXX[±n]` / `$XXXX`）：同窗归本 bank；$C000-$DFFF 归 bank14；$E000-$FFFF 归 bank15；其余跨窗**不可判**（R6/R7 运行期决定） | 同窗 279 + 跨 bank 固定窗 135（去重后并入 14/15） |
| E3 | 中断向量（bank15 固定窗，file 0x2000A）：NMI=$EF85、RESET=$E000、IRQ=$F20B | 3 |
| E4 | far-call 内联目标（`JSR $E649`+`(r6,r7,target)`，[rom-forensics](../findings/rom-forensics.md) §2） | 12 |
| E5 | bank11:$8000 JMP 表目标 | 7 |
| E6 | JMP 间接（表驱动分派）**不展开表项**——27 个站点计入分母缺口（§5） | 27 站点 |

**切分约定（B 级）**：bank 内按入口地址升序切段，段 i = `[entry_i, entry_{i+1})`，末段至窗口尾。
全 PRG 每字节恰属一段（**131,072/131,072**，无重叠无空洞）。段是**地址序切片**，不是已证明的函数体；
函数体边界考证（S3）按需后补。L2 用例中的手选片段入口（29 名里 26 名不是 JSR 目标）按所在段归属（§3.2）。

### 1.2 计数与三方交叉核对

| 口径 | 计数 | 规则 |
|---|---|---|
| **seed 入口全集（本盘点分母）** | **394 入口 = 394 段** | E1-E5 并集（全局合并跨 bank 边） |
| IDA 入口起点（S1 交叉核对） | 294 | 每 bank 独立扫描：本 bank 流内 call/jump 目标 + 段基址（`analysis/ida/bankNN.json` xrefs） |
| remap-table `symbols`（S1 机械层） | 276 条记录 | `tools/remap/run.py` 导出的 IDA funcs 记录（f_000..f_275） |

**对账（A 级，差集逐条可重导）**：seed 394 ⊇ IDA 294（`only_ida = 0`）；增补 100 =
far-call 目标 10（bank00 +4 / bank02 +3 / bank11 +3）+ 跨 bank 固定窗调用目标 90（bank14 +62 / bank15 +28，
含被流内未引用的向量入口）。remap 276 **不等于**入口数：`symbols[].cpu` 记录的是**所属 IDA 函数体起点**
而非调用目标地址（别名塌缩：f_005..f_018 十四条同为 `sub_8185`）——276 条记录仅对应 **192 个唯一体**。
【口径警示】用 remap-table 做入口身份对齐会把多个入口并成一条；入口身份一律以本盘点（seed 重导）为准。

**逐 bank 入口分布**（bank14/15 偏高：固定窗被全 ROM 跨 bank 调用 + E2 噪声嫌疑见 §5）：

| bank | 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 10 | 11 | 12 | 13 | 14 | 15 | 合计 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 入口 | 65 | 9 | 7 | 12 | 1 | 3 | 3 | 16 | 1 | 1 | 7 | 41 | 1 | 3 | 121 | 103 | **394** |

**调用图根（A 级）**：NMI → `bank15:EF85-F02F`；RESET → `bank15:E000-E072`；IRQ → `bank15:F20B-F250`。
far-call 目标段 12 个、bank11 表目标段 7 个（清单见 JSON `far_call_targets` / `bank11_table_targets`）。
段级 callers/callees 边（含 far-call 边）逐段存于镜像 JSON；静态可达闭包是**欠近似**（314 个跨窗调用不可判 + 27 个间接分派未展开）。

### 1.3 非代码分母（沿用 S1 机械层，见 [completeness.md](completeness.md) §4）

数据表 5（t_000/001/005/006/007）+ 数据锚点候选 21 + CHR 128 bank + RAM 560 + MMIO 414；
文本记录区/charmap 候选见 [data-tables](../findings/data-tables.md) 与 remap-table `known_data_candidates`。
数据面的**移植状态**不在本表（段表只收代码入口）；数据资产移植进度随 `game-data` 侧另行计量。

---

## 2. 分类（机械画像）

### 2.1 规则（B 级；第一条命中生效；逐段指纹随表供重导）

| 规则 | 判据 | 含义 |
|---|---|---|
| R-X | data 项行 > 指令行 | 投影噪声区（S1 线性扫描 code/data 失真）：足迹失真，不作语义分类 |
| R-A | `apu_write>0`（写 $4000-$4013/$4015/$4017） | 音频 |
| R-G | `ppu>0`（$2000-$2007）或 `oamdma_write>0`（$4014） | 图形 |
| R-D | 无 MMIO + `rom_idx_reads≥2` + 回边 + `ram_writes≥1` + `calls_out=0` | 数据（表行走/记录装载签名） |
| R-E | `calls_out>0` 或 `ram_writes>0` 或 `mapper_write>0` | 引擎（含玩法逻辑/输入/分 bank/流程） |
| R-? | 其余 | 待定（过小/纯透传/无签名） |

### 2.2 分类计数

| 分类 | 段数 | 字节 | 备注 |
|---|---|---|---|
| 引擎 | 311 | 52,416 | 玩法/流程/输入/分 bank 主体 |
| 图形 | 17 | 4,778 | PPU 寄存器/OAM DMA 足迹 |
| 音频 | 6 | 1,564 | APU 写足迹（含 bank11 音频引擎簇 8166/81B4/85E9/88A7） |
| 数据 | 2 | 379 | R-D 签名极严；表处理代码多带外呼或落 R-X，真数据资产在 §1.3 数据面计 |
| 待定 | 58 | 71,935 | 其中 R-X 投影噪声 23 段 70,760B + R-? 35 段 1,175B |
| **合计** | **394** | **131,072** | |

**校准抽查（A 级锚点）**：`ui_scroll_write`（bank1:AB3B，L2 已对拍）→ 图形 ✓；
`bank15:EF85` NMI 段 → 图形（PPU/OAM DMA 足迹）✓；bank11 音频簇 → 音频 ✓；
`pickup_handler`（bank2:922B，L2 12 例）→ 待定(R-X)——**不是语义否定，是投影失真**（§2.3）。

### 2.3 投影噪声登记（S1 缺陷挂账，decomp-analyst 层）

seed 里 **56,636B 被渲染为 data 项**（与 completeness.md “data 56,636B” 同源），其中混有**真实代码**：

| 现象 | 证据 | 影响 |
|---|---|---|
| bank02:$863B-$9697 连续 byte-data 项（2,995+979+154 行） | `grep -n '^bank02:87BD' reference/seed/gunnac.asm`（无反汇编行）；对照 L2 ss-4..5 **执行** bank2:87BD 为代码 | 该域足迹/分类失真（R-X）；`shot_script_fetch`/`shot_lut_accumulate`/`shot_alloc`/`pickup_handler`/`shot_config_loader`/`level_config_index` 全落 R-X |
| bank03:$AC42-$BFFF、bank01:$B29B-$BFFF 等 | data 项行主导（4,950/46、3,420/4） | 真数据表区（t_000/t_001），R-X 判定正确 |
| bank03:A2FA-A342 一带 39 指令行零足迹 | 段表逐行 | 待 S3 边界考证（介于 bomb_drop 与 death_money_halve 之间） |

**根因定位（§4 三问）**：投影层（S1 IDA 线性扫描在失配后对整段 create_insn 失败逐字节落 data），
非 ROM/移植层问题。**修复挂账 W-WORKSET-1**：以 Ghidra 投影（`analysis/ghidra/bank*.c`）交叉重扫
或 IDA 逐入口建函数重导，恢复该域反汇编行后重跑本生成器（分类 R-X 应大幅收敛）。
**在此之前：R-X 段的分类列一律不可当语义结论用**；移植状态列（L2/core/spec）另有真值来源。

---

## 3. 移植状态映射

### 3.1 状态梯度（每段取最高档）

| 状态 | 判据（机械可查） | 含义 |
|---|---|---|
| `L2-diff` | `oracle/subroutines/l2_cases.jsonl` 有 entry 落在段内 | ROM 原字节 ↔ game-core 同名函数**差分对拍**（121 例） |
| `core-cited` | `crates/game-core/src/*.rs` 引用 bank:addr 与段相交 | 已移植/骨架已挂（模块名见逐段表“依据”列） |
| `spec-cited` | `docs/spec/*.md` 引用相交 | 有规格规则锚定 |
| `findings-cited` | `docs/findings/*.md` 引用相交 | 有考证记录 |
| `unexplored` | 以上皆无 | 未动（分母的多数） |

### 3.2 L2 已对拍面（29 子程序 / 121 例，全表见 §7）

- 121 例全落 **15 个段**、6,865B（5.2%）；`unmapped L2 entries = 0`（29 名全部能归段）。
- **3/29 的 entry 是段首**（`pickup_handler` bank2:922B、`money_add_event` bank14:DBCE、
  `bcd_sub_ammo` bank15:ED03）；其余 26 名是**段内手选片段**（如 `player_x_clamp_hi` bank03:A0D0
  落在段 bank03:A039-A0FB 内）——段级状态只证明“该段有对拍面”，片段语义见
  [oracle/subroutines/README.md](../../oracle/subroutines/README.md) 与 constants v2/v4。

### 3.3 game-core 模块 ↔ 段引用计数（core-cited 34 段）

| 模块 | 引用段数 | 模块 | 引用段数 |
|---|---|---|---|
| flow.rs | 20 | player.rs | 5 |
| ui.rs | 18 | weapons.rs | 6 |
| l2ports.rs | 10 | stage.rs | 2 |
| terrain.rs | 3 | shot.rs | 2 |

（一模块可引多段、一段可被多模块引用；引用=源码注释里的 bank:addr 证据指针相交。）

---

## 4. 进度读数（2026-10-15 快照；重跑生成器即刷新）

| 档位 | 段数 | 段占比 | 字节 | 字节占比 |
|---|---|---|---|---|
| L2-diff | 15 | 3.8% | 6,865 | 5.2% |
| ≥ core-cited（含 L2） | 49 | 12.4% | 21,560 | 16.5% |
| ≥ spec-cited（含以上） | 79 | 20.1% | 31,790 | 24.3% |
| ≥ findings-cited（含以上） | 147 | 37.3% | 74,950 | 57.2% |
| unexplored | 247 | 62.7% | 56,122 | 42.8% |

**入口来源可信度（B 级，噪声会计）**：solid 330 / no-caller 40 / noise-suspect 21 / root 3。
noise-suspect=调用方全为 R-X 投影噪声段（假 JSR 字节序列嫌疑，集中在 bank14/15 增补入口）；
no-caller=无静态调用方（间接分派表目标嫌疑或死码）。**收敛动作**：W-WORKSET-1 修复投影后重算，
预计 394 分母中噪声项下降；分母修订走 dated 更正，不静默改数。

---

## 5. 分母缺口与挂账（明确不完整处）

| # | 缺口 | 计数 | 归宿 |
|---|---|---|---|
| G1 | JMP 间接分派表项未展开（E6） | 27 站点 | S3 表考证（bank11 JMP 表已展开 7 项是其中一例的静态形态） |
| G2 | 跨窗调用不可判（R6/R7 运行期 bank） | 314 处 | 动态 trace 补边（Mesen2 bank 切换观察） |
| G3 | 投影噪声段（R-X）分类失真 | 23 段 / 70,760B | W-WORKSET-1（§2.3） |
| G4 | 函数体边界未证（段=地址序切片） | 全部 | S3 按需（当前不影响进度分母的可算性） |
| G5 | bank03:A2FA-A342 等零足迹段 | 36 段 R-? | 随 W-WORKSET-1 / S3 |

**挂账语义（机制残项不在本表管辖）**：F13 免死支/卖出/隐藏门/$05DC（W18）等机制考证挂账见
[mechanism-closeout](../findings/mechanism-closeout.md) / [game-over-continue](game-over-continue.md)，
其工作量落在对应段（如 flow.rs 引用段）的状态升级里，不另立分母。

---

## 6. 复现与自检

```bash
# 全量重导（幂等）：分类/状态/计数/表格全部由下列命令再生
python3 tools/workset_inventory.py

# 自检点（脚本 stdout 同步打印）
#   entries=394 segments=394 segment_bytes=131072
#   roots: NMI=bank15:EF85-F02F RESET=bank15:E000-E072 IRQ=bank15:F20B-F250
#   class: 引擎311 数据2 图形17 音频6 待定58
#   status: L2-diff15 core-cited34 spec-cited30 findings-cited68 unexplored247
#   entry_trust: solid330 no-caller40 noise-suspect21 root3
#   cross_check: ida=294 remap=276 seed=394  cross_window_unresolved=314  indirect_jmp=27

# 三方对账差集（only_ida 应为 0；only_seed=100 的构成见 §1.2）
python3 - <<'EOF'
import json, collections
d = json.load(open('data/status/rewrite-workset.json'))
mine = collections.defaultdict(set)
for r in d["segments"]:
    mine[r["bank"]].add(r["seg_start"])
for b in range(16):
    ida = json.load(open("analysis/ida/bank%02d.json" % b))
    ts = {ida["base"]} | {x["to"] for x in ida["xrefs"] if x["kind"] in ("call", "jump")}
    print(f"bank{b:02d} only_ida={sorted(ts-mine[b])[:4]} only_seed={len(mine[b]-ts)}")
EOF

# L2 归段校验（29 名全部可归段，l2_unmapped_entries 为空）
python3 -c "import json;d=json.load(open('data/status/rewrite-workset.json'));print(d['l2_unmapped_entries'])"
```

生成物归宿：`analysis/workset/{subroutines.json,tables.md}`（可再生，不入库）；
`data/status/rewrite-workset.json`（入库镜像，进度机器读）；本文 §7 表格 = `analysis/workset/tables.md` 原文嵌入。

---

## 7. 逐段清单 + L2 映射表（机器生成，勿手改）

<!-- generated by tools/workset_inventory.py — 勿手改，重导覆盖 -->

### 摘要矩阵（段 × 分类 × 状态）

| 分类 \ 状态 | L2-diff | core-cited | spec-cited | findings-cited | unexplored | 小计 |
|---|---|---|---|---|---|---|
| 引擎 | 10 | 28 | 23 | 56 | 194 | 311 |
| 数据 | 0 | 0 | 0 | 0 | 2 | 2 |
| 图形 | 1 | 1 | 2 | 2 | 11 | 17 |
| 音频 | 0 | 1 | 0 | 1 | 4 | 6 |
| 待定 | 4 | 4 | 5 | 9 | 36 | 58 |
| **小计** | **15** | **34** | **30** | **68** | **247** | **394** |

### L2 已对拍子程序映射（29 名 / 121 例 → 段）

| 子程序 | entry | 用例数 | entry=段首 | 所在段 | 段分类 | 段状态 |
|---|---|---|---|---|---|---|
| player_x_clamp_hi | bank03:A0D0 | 4 | 否（段内片段） | bank03:A039-A0FB | 引擎 | L2-diff |
| player_x_clamp_lo | bank03:A0DD | 3 | 否（段内片段） | bank03:A039-A0FB | 引擎 | L2-diff |
| player_y_clamp_hi | bank03:A09D | 3 | 否（段内片段） | bank03:A039-A0FB | 引擎 | L2-diff |
| player_y_clamp_lo | bank03:A0AA | 2 | 否（段内片段） | bank03:A039-A0FB | 引擎 | L2-diff |
| digit_conversion | bank00:9DE8 | 5 | 否（段内片段） | bank00:9D71-9E07 | 引擎 | L2-diff |
| consumption_lookup | bank03:A2C4 | 7 | 否（段内片段） | bank03:A2B2-A2F9 | 引擎 | L2-diff |
| bomb_drop | bank03:A2B4 | 9 | 否（段内片段） | bank03:A2B2-A2F9 | 引擎 | L2-diff |
| movement_tick | bank03:A08D | 14 | 否（段内片段） | bank03:A039-A0FB | 引擎 | L2-diff |
| pickup_handler | bank2:922B | 12 | 是 | bank02:922B-92C4 | 待定 | L2-diff |
| level_config_index | bank2:9368 | 4 | 否（段内片段） | bank02:92C5-9697 | 待定 | L2-diff |
| shot_schedule_step | bank3:A1FD | 2 | 否（段内片段） | bank03:A0FC-A23C | 引擎 | L2-diff |
| shot_spawn_init | bank3:A20F | 1 | 否（段内片段） | bank03:A0FC-A23C | 引擎 | L2-diff |
| shot_script_fetch | bank2:87BD | 2 | 否（段内片段） | bank02:863B-922A | 待定 | L2-diff |
| shot_lut_accumulate | bank2:87DA | 4 | 否（段内片段） | bank02:863B-922A | 待定 | L2-diff |
| shot_config_loader | bank2:937C | 2 | 否（段内片段） | bank02:92C5-9697 | 待定 | L2-diff |
| shot_alloc | bank2:888D | 4 | 否（段内片段） | bank02:863B-922A | 待定 | L2-diff |
| terrain_load_record | bank00:83F6 | 4 | 否（段内片段） | bank00:83E7-8445 | 引擎 | L2-diff |
| terrain_x_adjust | bank00:8471 | 3 | 否（段内片段） | bank00:844A-8504 | 引擎 | L2-diff |
| terrain_loop_load | bank00:848B | 1 | 否（段内片段） | bank00:844A-8504 | 引擎 | L2-diff |
| terrain_loop_step | bank00:8492 | 2 | 否（段内片段） | bank00:844A-8504 | 引擎 | L2-diff |
| terrain_fetch_params | bank00:84A9 | 1 | 否（段内片段） | bank00:844A-8504 | 引擎 | L2-diff |
| ui_scroll_write | bank1:AB3B | 1 | 否（段内片段） | bank01:A92E-AB6F | 图形 | L2-diff |
| money_add_event | bank14:DBCE | 4 | 是 | bank14:DBCE-DDCE | 引擎 | L2-diff |
| shop_spend_rapid | bank00:9ECD | 3 | 否（段内片段） | bank00:9EB3-9EF2 | 引擎 | L2-diff |
| shop_spend_ship | bank00:9FAB | 2 | 否（段内片段） | bank00:9F86-9FE2 | 引擎 | L2-diff |
| shop_ship_qty_next | bank00:9FC0 | 5 | 否（段内片段） | bank00:9F86-9FE2 | 引擎 | L2-diff |
| shop_money_earn | bank02:8FA9 | 5 | 否（段内片段） | bank02:863B-922A | 待定 | L2-diff |
| death_money_halve | bank03:A39F | 6 | 否（段内片段） | bank03:A343-A4B5 | 引擎 | L2-diff |
| bcd_sub_ammo | bank15:ED03 | 6 | 是 | bank15:ED03-ED26 | 待定 | L2-diff |

### 逐段清单（机械画像 + 移植状态）

| 段（入口-尾） | 来源 | 分类(规则) | 投影 d/i | ppu/apu/oam/map | rom_idx/ram_w | calls/loop | 根 | 状态 | 依据 |
|---|---|---|---|---|---|---|---|---|---|
| bank00:8000-8034 | no-caller | 引擎(R-E) | 0/24 | 0/0/0/0 | 8/8 | 1/0 | — | spec-cited | spec×3; find×7 |
| bank00:8035-80D9 | solid | 引擎(R-E) | 0/86 | 0/0/0/0 | 0/26 | 7/2 | — | spec-cited | spec×1; find×4 |
| bank00:80DA-80F5 | solid | 引擎(R-E) | 0/13 | 0/0/0/0 | 0/1 | 2/0 | — | unexplored | — |
| bank00:80F6-80F8 | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank00:80F9-8184 | solid | 引擎(R-E) | 0/72 | 0/0/0/0 | 2/12 | 3/0 | — | spec-cited | spec×1; find×1 |
| bank00:8185-81A2 | solid | 引擎(R-E) | 0/16 | 0/0/0/0 | 0/3 | 2/0 | — | findings-cited | find×1 |
| bank00:81A3-82EE | solid | 引擎(R-E) | 0/172 | 0/0/0/0 | 4/48 | 2/0 | — | spec-cited | spec×1; find×3 |
| bank00:82EF-82FC | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/2 | 0/0 | — | findings-cited | find×1 |
| bank00:82FD-8307 | solid | 引擎(R-E) | 0/5 | 0/0/0/0 | 0/2 | 0/0 | — | findings-cited | find×1 |
| bank00:8308-8312 | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/2 | 0/0 | — | spec-cited | spec×1; find×1 |
| bank00:8313-8376 | no-caller | 引擎(R-E) | 0/52 | 0/0/0/0 | 0/15 | 0/3 | — | spec-cited | spec×1; find×1 |
| bank00:8377-839F | solid | 引擎(R-E) | 0/22 | 0/0/0/0 | 0/8 | 0/0 | — | spec-cited | spec×1; find×1 |
| bank00:83A0-83E6 | solid | 引擎(R-E) | 0/37 | 0/0/0/0 | 0/9 | 1/2 | — | unexplored | — |
| bank00:83E7-8445 | solid | 引擎(R-E) | 0/51 | 0/0/0/0 | 0/12 | 0/2 | — | L2-diff | L2:terrain_load_record(4); core:terrain; spec×2; find×2 |
| bank00:8446-8449 | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank00:844A-8504 | solid | 引擎(R-E) | 0/99 | 0/0/0/0 | 0/21 | 0/7 | — | L2-diff | L2:terrain_fetch_params,terrain_loop_load,terrain_loop_step,terrain_x_adjust(7); core:terrain; spec×3; find×4 |
| bank00:8505-8583 | no-caller | 引擎(R-E) | 0/70 | 0/0/0/0 | 0/12 | 1/6 | — | spec-cited | spec×2; find×2 |
| bank00:8584-859F | solid | 引擎(R-E) | 0/12 | 0/0/0/0 | 0/2 | 2/0 | — | spec-cited | spec×2; find×2 |
| bank00:85A0-85A8 | solid | 引擎(R-E) | 0/4 | 0/0/0/0 | 0/1 | 1/0 | — | spec-cited | spec×1; find×1 |
| bank00:85A9-8618 | no-caller | 引擎(R-E) | 0/58 | 0/0/0/0 | 3/8 | 0/0 | — | spec-cited | spec×1; find×2 |
| bank00:8619-865B | solid | 引擎(R-E) | 0/32 | 0/0/0/0 | 0/7 | 6/0 | — | findings-cited | find×1 |
| bank00:865C-869F | solid | 引擎(R-E) | 0/36 | 0/0/0/0 | 0/9 | 0/0 | — | findings-cited | find×3 |
| bank00:86A0-86AD | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/1 | 0/0 | — | findings-cited | find×2 |
| bank00:86AE-86CD | solid | 引擎(R-E) | 0/17 | 0/0/0/0 | 0/6 | 1/1 | — | findings-cited | find×3 |
| bank00:86CE-86CF | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | — | findings-cited | find×2 |
| bank00:86D0-86FD | solid | 引擎(R-E) | 0/24 | 0/0/0/0 | 0/6 | 2/1 | — | findings-cited | find×1 |
| bank00:86FE-8806 | solid | 引擎(R-E) | 0/131 | 0/0/0/0 | 2/34 | 8/4 | — | findings-cited | find×1 |
| bank00:8807-8819 | solid | 引擎(R-E) | 0/9 | 0/0/0/0 | 2/2 | 0/0 | far | unexplored | — |
| bank00:881A-8894 | solid | 引擎(R-E) | 0/56 | 0/0/0/0 | 0/16 | 0/0 | far | findings-cited | find×2 |
| bank00:8895-88BD | solid | 引擎(R-E) | 0/22 | 0/0/0/0 | 0/3 | 1/1 | — | findings-cited | find×1 |
| bank00:88BE-897E | solid | 引擎(R-E) | 0/100 | 0/0/0/0 | 0/25 | 2/0 | — | findings-cited | find×1 |
| bank00:897F-8D39 | no-caller | 引擎(R-E) | 3/460 | 0/0/0/0 | 1/104 | 27/10 | — | core-cited | core:flow; spec×3; find×14 |
| bank00:8D3A-8D8C | solid | 引擎(R-E) | 0/43 | 0/0/0/0 | 2/7 | 0/0 | — | spec-cited | spec×1; find×3 |
| bank00:8D8D-8DAF | solid | 引擎(R-E) | 0/18 | 0/0/0/0 | 0/6 | 1/2 | — | unexplored | — |
| bank00:8DB0-8DBC | solid | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank00:8DBD-8DCB | solid | 待定(R-?) | 0/9 | 0/0/0/0 | 0/0 | 0/2 | — | unexplored | — |
| bank00:8DCC-8DDE | solid | 引擎(R-E) | 0/9 | 0/0/0/0 | 0/2 | 1/1 | — | unexplored | — |
| bank00:8DDF-8EC7 | solid | 引擎(R-E) | 0/99 | 0/0/0/0 | 2/15 | 27/1 | far | findings-cited | find×6 |
| bank00:8EC8-8FED | no-caller | 引擎(R-E) | 2/130 | 0/0/0/13 | 0/14 | 9/0 | — | spec-cited | spec×2; find×10 |
| bank00:8FEE-929D | solid | 引擎(R-E) | 18/334 | 0/0/0/2 | 0/23 | 10/2 | — | findings-cited | find×1 |
| bank00:929E-9319 | solid | 引擎(R-E) | 0/52 | 0/0/0/0 | 0/12 | 15/1 | — | findings-cited | find×5 |
| bank00:931A-936B | solid | 引擎(R-E) | 0/36 | 0/0/0/0 | 0/5 | 6/0 | — | findings-cited | find×2 |
| bank00:936C-93C0 | solid | 待定(R-?) | 0/40 | 0/0/0/0 | 6/0 | 0/0 | — | findings-cited | find×4 |
| bank00:93C1-9409 | solid | 引擎(R-E) | 0/33 | 0/0/0/0 | 0/4 | 1/0 | — | findings-cited | find×5 |
| bank00:940A-95F8 | solid | 引擎(R-E) | 11/220 | 0/0/0/2 | 8/29 | 19/3 | — | findings-cited | find×5 |
| bank00:95F9-962B | solid | 引擎(R-E) | 0/20 | 0/0/0/0 | 0/1 | 9/0 | far | core-cited | core:flow; spec×2; find×4 |
| bank00:962C-9638 | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/3 | 0/0 | — | core-cited | core:flow; find×1 |
| bank00:9639-971D | no-caller | 引擎(R-E) | 0/115 | 0/0/0/0 | 0/24 | 5/1 | — | findings-cited | find×6 |
| bank00:971E-9759 | solid | 引擎(R-E) | 0/26 | 0/0/0/0 | 1/0 | 7/4 | — | core-cited | core:flow; find×1 |
| bank00:975A-9762 | no-caller | 引擎(R-E) | 0/3 | 0/0/0/0 | 0/0 | 1/0 | — | findings-cited | find×1 |
| bank00:9763-9BC2 | solid | 音频(R-A) | 28/495 | 0/1/0/9 | 13/33 | 22/0 | — | findings-cited | find×4 |
| bank00:9BC3-9CE8 | solid | 引擎(R-E) | 1/134 | 0/0/0/1 | 2/30 | 18/1 | far | core-cited | core:flow; spec×4; find×15 |
| bank00:9CE9-9CEB | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | — | core-cited | core:ui |
| bank00:9CEC-9D5D | solid | 引擎(R-E) | 2/52 | 0/0/0/0 | 3/5 | 7/1 | — | core-cited | core:ui; spec×2; find×6 |
| bank00:9D5E-9D70 | solid | 引擎(R-E) | 1/7 | 0/0/0/0 | 0/3 | 2/0 | — | core-cited | core:ui; spec×1; find×5 |
| bank00:9D71-9E07 | solid | 引擎(R-E) | 2/64 | 0/0/0/0 | 0/9 | 12/1 | — | L2-diff | L2:digit_conversion(5); core:l2ports,player,ui; spec×2; find×15 |
| bank00:9E08-9E1D | no-caller | 引擎(R-E) | 2/8 | 0/0/0/0 | 0/1 | 2/0 | — | core-cited | core:ui,weapons; spec×1; find×4 |
| bank00:9E1E-9E3F | solid | 引擎(R-E) | 0/17 | 0/0/0/0 | 0/3 | 1/0 | — | findings-cited | find×2 |
| bank00:9E40-9E72 | solid | 引擎(R-E) | 0/21 | 0/0/0/0 | 0/4 | 0/0 | — | core-cited | core:ui; spec×1; find×5 |
| bank00:9E73-9E85 | solid | 引擎(R-E) | 0/10 | 0/0/0/0 | 0/0 | 1/0 | — | findings-cited | find×3 |
| bank00:9E86-9EB2 | solid | 引擎(R-E) | 0/20 | 0/0/0/0 | 0/6 | 4/0 | — | core-cited | core:ui; spec×1; find×4 |
| bank00:9EB3-9EF2 | no-caller | 引擎(R-E) | 0/26 | 0/0/0/0 | 0/2 | 6/1 | — | L2-diff | L2:shop_spend_rapid(3); core:l2ports,ui; spec×1; find×6 |
| bank00:9EF3-9F85 | solid | 引擎(R-E) | 0/63 | 0/0/0/0 | 0/15 | 11/1 | — | core-cited | core:ui; spec×1; find×7 |
| bank00:9F86-9FE2 | no-caller | 引擎(R-E) | 0/41 | 0/0/0/0 | 0/4 | 6/1 | — | L2-diff | L2:shop_ship_qty_next,shop_spend_ship(7); core:l2ports,ui; spec×1; find×10 |
| bank00:9FE3-9FFF | solid | 引擎(R-E) | 1/14 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank01:A000-A06B | no-caller | 引擎(R-E) | 0/51 | 0/0/0/0 | 1/10 | 7/0 | — | core-cited | core:l2ports,ui; spec×3; find×3 |
| bank01:A06C-A09E | solid | 引擎(R-E) | 0/23 | 0/0/0/0 | 0/9 | 1/0 | — | findings-cited | find×7 |
| bank01:A09F-A0BB | solid | 引擎(R-E) | 0/17 | 0/0/0/0 | 0/2 | 2/1 | — | findings-cited | find×1 |
| bank01:A0BC-A92D | solid | 引擎(R-E) | 60/1022 | 0/0/0/26 | 22/55 | 34/9 | — | core-cited | core:ui; spec×1; find×8 |
| bank01:A92E-AB6F | no-caller | 图形(R-G) | 0/271 | 7/0/0/13 | 3/53 | 14/8 | — | L2-diff | L2:ui_scroll_write(1); core:stage; spec×1; find×11 |
| bank01:AB70-B18A | solid | 待定(R-X) | 1128/200 | 0/0/0/3 | 0/12 | 9/2 | — | findings-cited | find×1 |
| bank01:B18B-B29A | solid | 待定(R-X) | 199/36 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank01:B29B-BEBC | solid | 待定(R-X) | 3097/4 | 0/0/0/0 | 0/0 | 0/0 | — | spec-cited | spec×1; find×2 |
| bank01:BEBD-BFFF | solid | 待定(R-X) | 323/0 | 0/0/0/0 | 0/0 | 0/0 | — | spec-cited | spec×1 |
| bank02:8000-82F7 | no-caller | 引擎(R-E) | 9/355 | 0/0/0/9 | 14/18 | 6/4 | — | findings-cited | find×4 |
| bank02:82F8-83DD | solid | 待定(R-X) | 83/68 | 0/0/0/0 | 1/14 | 3/0 | — | core-cited | core:flow; find×4 |
| bank02:83DE-863A | noise-suspect | 待定(R-X) | 565/16 | 0/0/0/0 | 2/3 | 1/0 | — | unexplored | — |
| bank02:863B-922A | noise-suspect | 待定(R-X) | 2995/27 | 0/0/0/0 | 3/5 | 2/0 | — | L2-diff | L2:shop_money_earn,shot_alloc,shot_lut_accumulate,shot_script_fetch(15); core:l2ports,shot,ui; spec×6; find×19 |
| bank02:922B-92C4 | solid | 待定(R-X) | 154/0 | 0/0/0/0 | 0/0 | 0/0 | far | L2-diff | L2:pickup_handler(12); core:ui,weapons; spec×3; find×6 |
| bank02:92C5-9697 | solid | 待定(R-X) | 979/0 | 0/0/0/0 | 0/0 | 0/0 | far | L2-diff | L2:level_config_index,shot_config_loader(6); core:ui,weapons; spec×4; find×4 |
| bank02:9698-9FFF | solid | 待定(R-X) | 2408/0 | 0/0/0/0 | 0/0 | 0/0 | far | findings-cited | find×2 |
| bank03:A000-A038 | no-caller | 引擎(R-E) | 0/22 | 0/0/0/0 | 1/9 | 1/0 | — | unexplored | — |
| bank03:A039-A0FB | solid | 引擎(R-E) | 0/86 | 0/0/0/0 | 3/10 | 3/0 | — | L2-diff | L2:movement_tick,player_x_clamp_hi,player_x_clamp_lo,player_y_clamp_hi,player_y_clamp_lo(26); core:l2ports,player; spec×4; find×2 |
| bank03:A0FC-A23C | solid | 引擎(R-E) | 0/145 | 0/0/0/0 | 2/29 | 4/3 | — | L2-diff | L2:shot_schedule_step,shot_spawn_init(3); core:player,shot,weapons; spec×4; find×11 |
| bank03:A23D-A2B1 | solid | 引擎(R-E) | 0/52 | 0/0/0/0 | 0/10 | 0/5 | — | core-cited | core:weapons; spec×2; find×2 |
| bank03:A2B2-A2F9 | solid | 引擎(R-E) | 0/29 | 0/0/0/0 | 3/5 | 1/0 | — | L2-diff | L2:bomb_drop,consumption_lookup(16); core:l2ports,player; spec×4; find×5 |
| bank03:A2FA-A342 | solid | 待定(R-?) | 9/39 | 0/0/0/0 | 0/0 | 0/2 | — | core-cited | core:l2ports,player; spec×5; find×4 |
| bank03:A343-A4B5 | no-caller | 引擎(R-E) | 0/155 | 0/0/0/0 | 4/35 | 23/1 | — | L2-diff | L2:death_money_halve(6); core:flow,l2ports,weapons; spec×2; find×14 |
| bank03:A4B6-A9AD | no-caller | 引擎(R-E) | 1/526 | 0/0/0/1 | 13/113 | 51/5 | — | findings-cited | find×6 |
| bank03:A9AE-AB3C | solid | 引擎(R-E) | 2/170 | 0/0/0/0 | 4/41 | 17/1 | — | unexplored | — |
| bank03:AB3D-AB51 | solid | 引擎(R-E) | 0/9 | 0/0/0/0 | 0/4 | 0/0 | — | unexplored | — |
| bank03:AB52-AC41 | solid | 引擎(R-E) | 0/103 | 0/0/0/0 | 6/29 | 9/1 | — | findings-cited | find×2 |
| bank03:AC42-BFFF | solid | 待定(R-X) | 4950/46 | 0/0/0/0 | 7/12 | 0/1 | — | findings-cited | find×3 |
| bank04:8000-9FFF | no-caller | 待定(R-X) | 7257/387 | 0/0/0/0 | 18/92 | 51/0 | — | unexplored | — |
| bank05:8000-8459 | no-caller | 引擎(R-E) | 2/464 | 0/0/0/4 | 18/104 | 42/2 | — | unexplored | — |
| bank05:845A-8496 | solid | 引擎(R-E) | 0/25 | 0/0/0/0 | 2/7 | 1/0 | — | unexplored | — |
| bank05:8497-9FFF | no-caller | 待定(R-X) | 4234/1189 | 0/0/0/7 | 35/259 | 136/9 | — | findings-cited | find×2 |
| bank06:8000-8459 | no-caller | 引擎(R-E) | 4/468 | 0/0/0/1 | 10/106 | 43/2 | — | unexplored | — |
| bank06:845A-8496 | solid | 引擎(R-E) | 0/27 | 0/0/0/0 | 2/7 | 1/0 | — | unexplored | — |
| bank06:8497-9FFF | no-caller | 待定(R-X) | 3192/1685 | 0/0/0/12 | 78/316 | 173/19 | — | unexplored | — |
| bank07:A000-A0A5 | no-caller | 引擎(R-E) | 0/65 | 0/0/0/0 | 3/19 | 9/0 | — | unexplored | — |
| bank07:A0A6-A175 | solid | 引擎(R-E) | 0/86 | 0/0/0/0 | 4/24 | 9/1 | — | unexplored | — |
| bank07:A176-A217 | no-caller | 引擎(R-E) | 0/65 | 0/0/0/0 | 5/8 | 13/0 | — | unexplored | — |
| bank07:A218-A4A8 | solid | 引擎(R-E) | 5/276 | 0/0/0/0 | 8/56 | 29/1 | — | unexplored | — |
| bank07:A4A9-A4D5 | solid | 引擎(R-E) | 0/19 | 0/0/0/0 | 0/5 | 2/0 | — | unexplored | — |
| bank07:A4D6-A772 | solid | 引擎(R-E) | 6/275 | 0/0/0/1 | 25/70 | 22/2 | — | unexplored | — |
| bank07:A773-A7F4 | solid | 引擎(R-E) | 0/56 | 0/0/0/0 | 3/14 | 6/0 | — | unexplored | — |
| bank07:A7F5-A810 | solid | 引擎(R-E) | 0/12 | 0/0/0/0 | 0/3 | 1/0 | — | unexplored | — |
| bank07:A811-A8E0 | solid | 引擎(R-E) | 0/85 | 0/0/0/0 | 2/16 | 15/0 | — | unexplored | — |
| bank07:A8E1-A92F | solid | 引擎(R-E) | 1/35 | 0/0/0/0 | 0/6 | 6/0 | — | unexplored | — |
| bank07:A930-AB74 | solid | 引擎(R-E) | 0/250 | 0/0/0/0 | 7/54 | 31/3 | — | unexplored | — |
| bank07:AB75-ABE2 | solid | 引擎(R-E) | 0/59 | 0/0/0/0 | 0/12 | 2/2 | — | unexplored | — |
| bank07:ABE3-AD8A | solid | 引擎(R-E) | 15/189 | 0/0/0/0 | 4/25 | 10/3 | — | unexplored | — |
| bank07:AD8B-ADC8 | solid | 引擎(R-E) | 0/26 | 0/0/0/0 | 2/8 | 5/0 | — | unexplored | — |
| bank07:ADC9-ADF3 | solid | 引擎(R-E) | 1/20 | 0/0/0/0 | 0/2 | 1/2 | — | unexplored | — |
| bank07:ADF4-BFFF | solid | 引擎(R-E) | 6/1642 | 0/0/0/1 | 17/72 | 27/5 | — | unexplored | — |
| bank08:A000-BFFF | no-caller | 待定(R-X) | 4717/1981 | 0/0/0/33 | 32/62 | 17/10 | — | core-cited | core:stage; spec×4; find×8 |
| bank09:8000-9FFF | no-caller | 待定(R-X) | 7251/506 | 0/0/0/7 | 11/18 | 1/9 | — | findings-cited | find×2 |
| bank10:8000-81BA | solid | 引擎(R-E) | 10/225 | 0/0/0/5 | 8/5 | 1/10 | — | unexplored | — |
| bank10:81BB-8500 | solid | 引擎(R-E) | 28/434 | 0/1/0/1 | 7/15 | 2/4 | — | unexplored | — |
| bank10:8501-85BA | solid | 引擎(R-E) | 14/112 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank10:85BB-87BA | solid | 图形(R-G) | 5/236 | 1/0/0/1 | 1/10 | 4/0 | — | unexplored | — |
| bank10:87BB-8FBA | solid | 图形(R-G) | 60/1099 | 1/0/0/14 | 26/22 | 4/11 | — | unexplored | — |
| bank10:8FBB-97BA | no-caller | 引擎(R-E) | 57/998 | 0/0/0/27 | 22/29 | 15/8 | — | findings-cited | find×4 |
| bank10:97BB-9FFF | solid | 待定(R-X) | 2110/3 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank11:8000-800B | solid | 待定(R-?) | 0/4 | 0/0/0/0 | 0/0 | 0/0 | far | spec-cited | spec×1 |
| bank11:800C-800E | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | far | unexplored | — |
| bank11:800F-8011 | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | far | unexplored | — |
| bank11:8012-8015 | solid | 待定(R-?) | 0/2 | 0/0/0/0 | 0/0 | 0/0 | far | unexplored | — |
| bank11:8016-8064 | solid | 引擎(R-E) | 0/35 | 0/0/0/0 | 0/3 | 3/2 | b11 | unexplored | — |
| bank11:8065-8113 | solid | 引擎(R-E) | 0/71 | 0/0/0/0 | 0/7 | 7/0 | — | unexplored | — |
| bank11:8114-8165 | solid | 引擎(R-E) | 0/34 | 0/0/0/0 | 0/5 | 2/0 | — | unexplored | — |
| bank11:8166-81B3 | solid | 音频(R-A) | 0/33 | 0/2/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank11:81B4-81EE | solid | 音频(R-A) | 0/24 | 0/7/0/0 | 0/1 | 0/1 | — | unexplored | — |
| bank11:81EF-8208 | solid | 引擎(R-E) | 0/14 | 0/0/0/0 | 0/1 | 1/1 | — | unexplored | — |
| bank11:8209-8235 | solid | 引擎(R-E) | 0/21 | 0/0/0/0 | 0/1 | 0/1 | — | unexplored | — |
| bank11:8236-8297 | solid | 引擎(R-E) | 0/46 | 0/0/0/0 | 0/10 | 0/0 | — | unexplored | — |
| bank11:8298-83CD | no-caller | 引擎(R-E) | 0/156 | 0/0/0/0 | 0/25 | 0/3 | — | core-cited | core:flow; spec×1; find×2 |
| bank11:83CE-8410 | solid | 引擎(R-E) | 0/31 | 0/0/0/0 | 0/7 | 0/0 | — | unexplored | — |
| bank11:8411-8437 | solid | 引擎(R-E) | 0/18 | 0/0/0/0 | 0/4 | 0/1 | — | unexplored | — |
| bank11:8438-8448 | solid | 待定(R-?) | 0/10 | 0/0/0/0 | 2/0 | 0/0 | — | unexplored | — |
| bank11:8449-84D2 | solid | 引擎(R-E) | 0/64 | 0/0/0/2 | 5/6 | 2/1 | — | unexplored | — |
| bank11:84D3-84F0 | solid | 引擎(R-E) | 0/13 | 0/0/0/0 | 1/1 | 0/0 | — | unexplored | — |
| bank11:84F1-84FC | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank11:84FD-8535 | solid | 引擎(R-E) | 0/25 | 0/0/0/0 | 0/4 | 2/0 | — | unexplored | — |
| bank11:8536-85AD | solid | 引擎(R-E) | 0/57 | 0/0/0/0 | 2/11 | 0/0 | — | unexplored | — |
| bank11:85AE-85E8 | solid | 引擎(R-E) | 0/25 | 0/0/0/0 | 0/6 | 0/0 | — | unexplored | — |
| bank11:85E9-8644 | solid | 音频(R-A) | 0/41 | 0/4/0/0 | 6/4 | 0/0 | — | unexplored | — |
| bank11:8645-8680 | solid | 引擎(R-E) | 0/29 | 0/0/0/0 | 2/1 | 1/0 | — | unexplored | — |
| bank11:8681-86C2 | solid | 待定(R-?) | 0/39 | 0/0/0/0 | 0/0 | 0/3 | — | unexplored | — |
| bank11:86C3-8726 | solid | 引擎(R-E) | 0/48 | 0/0/0/0 | 2/4 | 1/0 | — | unexplored | — |
| bank11:8727-8792 | solid | 引擎(R-E) | 0/43 | 0/0/0/0 | 0/6 | 0/8 | — | unexplored | — |
| bank11:8793-87AD | solid | 引擎(R-E) | 0/14 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank11:87AE-87BC | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/4 | 0/0 | — | unexplored | — |
| bank11:87BD-8819 | solid | 引擎(R-E) | 0/44 | 0/0/0/0 | 0/1 | 0/3 | — | unexplored | — |
| bank11:881A-8893 | solid | 引擎(R-E) | 0/51 | 0/0/0/0 | 0/8 | 0/2 | — | unexplored | — |
| bank11:8894-889A | solid | 待定(R-?) | 0/4 | 0/0/0/0 | 0/0 | 0/0 | b11 | unexplored | — |
| bank11:889B-88A2 | solid | 待定(R-?) | 0/4 | 0/0/0/0 | 0/0 | 0/1 | b11 | unexplored | — |
| bank11:88A3-88A6 | solid | 待定(R-?) | 0/2 | 0/0/0/0 | 0/0 | 0/0 | b11 | unexplored | — |
| bank11:88A7-890A | solid | 音频(R-A) | 0/43 | 0/12/0/0 | 0/8 | 0/2 | b11 | unexplored | — |
| bank11:890B-890B | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | b11 | unexplored | — |
| bank11:890C-892D | no-caller | 引擎(R-E) | 0/20 | 0/0/0/0 | 0/1 | 1/1 | — | unexplored | — |
| bank11:892E-8981 | solid | 引擎(R-E) | 0/42 | 0/0/0/2 | 3/2 | 0/0 | — | unexplored | — |
| bank11:8982-89E4 | noise-suspect | 引擎(R-E) | 0/46 | 0/0/0/0 | 0/13 | 0/3 | — | unexplored | — |
| bank11:89E5-8E2B | solid | 待定(R-X) | 813/147 | 0/0/0/1 | 1/3 | 2/1 | — | unexplored | — |
| bank11:8E2C-9FFF | solid | 待定(R-X) | 4524/18 | 0/0/0/0 | 0/5 | 0/1 | b11 | spec-cited | spec×2; find×1 |
| bank12:A000-BFFF | no-caller | 引擎(R-E) | 134/3831 | 0/1/0/18 | 24/168 | 46/19 | — | findings-cited | find×1 |
| bank13:8000-82E4 | no-caller | 引擎(R-E) | 22/364 | 0/0/0/9 | 5/35 | 5/2 | — | unexplored | — |
| bank13:82E5-87FF | noise-suspect | 引擎(R-E) | 19/625 | 0/0/0/7 | 5/65 | 6/0 | — | unexplored | — |
| bank13:8800-9FFF | no-caller | 待定(R-X) | 4238/921 | 0/0/0/6 | 11/21 | 13/2 | — | unexplored | — |
| bank14:C000-C2C5 | no-caller | 图形(R-G) | 11/389 | 2/0/0/2 | 2/15 | 2/1 | — | unexplored | — |
| bank14:C2C6-C303 | solid | 引擎(R-E) | 0/35 | 0/0/0/0 | 0/0 | 2/3 | — | unexplored | — |
| bank14:C304-C326 | solid | 引擎(R-E) | 0/16 | 0/0/0/0 | 0/1 | 3/2 | — | unexplored | — |
| bank14:C327-C365 | noise-suspect | 引擎(R-E) | 1/35 | 0/0/0/0 | 2/3 | 0/0 | — | unexplored | — |
| bank14:C366-C37E | solid | 引擎(R-E) | 0/11 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:C37F-C397 | solid | 引擎(R-E) | 0/11 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank14:C398-C3DE | solid | 引擎(R-E) | 0/40 | 0/0/0/0 | 6/6 | 0/0 | — | unexplored | — |
| bank14:C3DF-C40A | solid | 引擎(R-E) | 0/24 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:C40B-C42F | solid | 引擎(R-E) | 0/18 | 0/0/0/0 | 0/6 | 0/0 | — | unexplored | — |
| bank14:C430-C43F | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank14:C440-C473 | solid | 引擎(R-E) | 0/25 | 0/0/0/0 | 0/4 | 0/0 | — | unexplored | — |
| bank14:C474-C47A | solid | 引擎(R-E) | 0/3 | 0/0/0/0 | 0/2 | 1/0 | — | unexplored | — |
| bank14:C47B-C4F6 | solid | 引擎(R-E) | 4/63 | 0/0/0/3 | 3/7 | 2/1 | — | unexplored | — |
| bank14:C4F7-C505 | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/1 | 1/0 | — | unexplored | — |
| bank14:C506-C51F | solid | 引擎(R-E) | 0/12 | 0/0/0/0 | 0/3 | 1/1 | — | unexplored | — |
| bank14:C520-C53A | solid | 待定(R-?) | 0/12 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank14:C53B-C582 | solid | 引擎(R-E) | 0/33 | 0/0/0/0 | 0/8 | 3/2 | — | unexplored | — |
| bank14:C583-C5CE | solid | 引擎(R-E) | 0/33 | 0/0/0/0 | 0/10 | 0/1 | — | unexplored | — |
| bank14:C5CF-C5DE | solid | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:C5DF-C5EB | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/2 | 0/1 | — | unexplored | — |
| bank14:C5EC-C605 | noise-suspect | 引擎(R-E) | 0/10 | 0/0/0/0 | 0/3 | 0/1 | — | unexplored | — |
| bank14:C606-C607 | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank14:C608-C63D | solid | 引擎(R-E) | 0/25 | 0/0/0/0 | 0/6 | 0/0 | — | unexplored | — |
| bank14:C63E-C64C | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:C64D-C670 | solid | 引擎(R-E) | 0/15 | 0/0/0/0 | 0/3 | 0/1 | — | unexplored | — |
| bank14:C671-C691 | solid | 引擎(R-E) | 0/13 | 0/0/0/0 | 0/6 | 1/0 | — | unexplored | — |
| bank14:C692-C6AD | solid | 引擎(R-E) | 0/14 | 0/0/0/0 | 0/2 | 1/0 | — | unexplored | — |
| bank14:C6AE-C783 | solid | 引擎(R-E) | 0/104 | 0/0/0/0 | 0/14 | 0/0 | — | unexplored | — |
| bank14:C784-C797 | solid | 引擎(R-E) | 0/10 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:C798-C7BC | solid | 引擎(R-E) | 0/15 | 0/0/0/0 | 0/7 | 0/0 | — | unexplored | — |
| bank14:C7BD-C836 | no-caller | 引擎(R-E) | 0/61 | 0/0/0/0 | 0/8 | 2/0 | — | unexplored | — |
| bank14:C837-C871 | solid | 引擎(R-E) | 0/25 | 0/0/0/0 | 0/4 | 2/0 | — | unexplored | — |
| bank14:C872-C893 | solid | 引擎(R-E) | 0/16 | 0/0/0/0 | 1/4 | 1/0 | — | unexplored | — |
| bank14:C894-C89E | solid | 引擎(R-E) | 0/5 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:C89F-C91F | solid | 图形(R-G) | 2/65 | 1/0/0/0 | 0/1 | 1/1 | — | unexplored | — |
| bank14:C920-C985 | solid | 图形(R-G) | 3/56 | 2/1/0/0 | 0/0 | 2/0 | — | unexplored | — |
| bank14:C986-C993 | noise-suspect | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank14:C994-CA31 | noise-suspect | 引擎(R-E) | 0/65 | 0/0/0/0 | 0/17 | 0/0 | — | unexplored | — |
| bank14:CA32-CAC8 | solid | 引擎(R-E) | 0/78 | 0/0/0/0 | 0/16 | 2/2 | — | unexplored | — |
| bank14:CAC9-CB30 | solid | 引擎(R-E) | 0/52 | 0/0/0/0 | 0/13 | 2/4 | — | unexplored | — |
| bank14:CB31-CB3D | noise-suspect | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/1 | 0/1 | — | unexplored | — |
| bank14:CB3E-CB4B | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:CB4C-CB4E | solid | 引擎(R-E) | 0/1 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank14:CB4F-CB62 | solid | 引擎(R-E) | 0/10 | 0/0/0/0 | 0/4 | 0/0 | — | core-cited | core:flow; spec×1; find×4 |
| bank14:CB63-CB67 | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/0 | 1/1 | — | unexplored | — |
| bank14:CB68-CBAE | solid | 引擎(R-E) | 0/34 | 0/0/0/0 | 0/8 | 0/1 | — | findings-cited | find×3 |
| bank14:CBAF-CBEF | solid | 引擎(R-E) | 0/29 | 0/0/0/0 | 0/4 | 0/0 | — | unexplored | — |
| bank14:CBF0-CBFD | solid | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank14:CBFE-CC50 | noise-suspect | 引擎(R-E) | 0/41 | 0/0/0/0 | 0/7 | 3/0 | — | unexplored | — |
| bank14:CC51-CC64 | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/0 | 2/0 | — | unexplored | — |
| bank14:CC65-CC69 | solid | 待定(R-?) | 0/2 | 0/0/0/0 | 0/0 | 0/1 | — | unexplored | — |
| bank14:CC6A-CC96 | solid | 引擎(R-E) | 0/20 | 0/0/0/0 | 2/6 | 1/2 | — | unexplored | — |
| bank14:CC97-CCB3 | solid | 引擎(R-E) | 0/16 | 0/0/0/0 | 0/4 | 1/0 | — | unexplored | — |
| bank14:CCB4-CCD2 | solid | 引擎(R-E) | 0/18 | 0/0/0/0 | 0/6 | 1/1 | — | unexplored | — |
| bank14:CCD3-CD07 | solid | 引擎(R-E) | 0/26 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank14:CD08-CD28 | solid | 引擎(R-E) | 0/14 | 0/0/0/0 | 0/3 | 0/2 | — | unexplored | — |
| bank14:CD29-CD2C | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank14:CD2D-CD62 | solid | 引擎(R-E) | 0/24 | 0/0/0/0 | 0/4 | 0/0 | — | unexplored | — |
| bank14:CD63-CD9C | solid | 引擎(R-E) | 0/26 | 0/0/0/0 | 0/5 | 0/1 | — | unexplored | — |
| bank14:CD9D-CDBE | solid | 引擎(R-E) | 0/17 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank14:CDBF-CDC7 | solid | 引擎(R-E) | 0/4 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:CDC8-CDDD | solid | 引擎(R-E) | 0/11 | 0/0/0/0 | 0/3 | 1/1 | — | unexplored | — |
| bank14:CDDE-CDF1 | solid | 引擎(R-E) | 0/11 | 0/0/0/0 | 0/3 | 0/0 | — | spec-cited | spec×1; find×1 |
| bank14:CDF2-CE72 | solid | 引擎(R-E) | 0/65 | 0/0/0/0 | 0/16 | 0/3 | — | unexplored | — |
| bank14:CE73-CEC7 | solid | 引擎(R-E) | 0/46 | 0/0/0/0 | 0/10 | 0/1 | — | unexplored | — |
| bank14:CEC8-CEF0 | solid | 引擎(R-E) | 0/18 | 0/0/0/0 | 3/5 | 0/0 | — | unexplored | — |
| bank14:CEF1-CF2B | solid | 引擎(R-E) | 0/26 | 0/0/0/0 | 4/3 | 0/0 | — | unexplored | — |
| bank14:CF2C-CF69 | solid | 引擎(R-E) | 0/35 | 0/0/0/0 | 0/3 | 2/3 | — | unexplored | — |
| bank14:CF6A-CFBD | solid | 引擎(R-E) | 0/45 | 0/0/0/0 | 0/5 | 0/3 | — | findings-cited | find×2 |
| bank14:CFBE-CFC4 | solid | 引擎(R-E) | 0/4 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank14:CFC5-CFE9 | solid | 引擎(R-E) | 0/16 | 0/0/0/0 | 0/6 | 0/1 | — | spec-cited | spec×1; find×1 |
| bank14:CFEA-D048 | solid | 引擎(R-E) | 0/49 | 0/0/0/0 | 0/5 | 0/2 | — | core-cited | core:ui; spec×4; find×3 |
| bank14:D049-D07C | solid | 引擎(R-E) | 0/26 | 0/0/0/0 | 0/9 | 1/1 | — | unexplored | — |
| bank14:D07D-D086 | solid | 待定(R-?) | 0/5 | 0/0/0/0 | 0/0 | 0/1 | — | unexplored | — |
| bank14:D087-D10B | no-caller | 引擎(R-E) | 0/63 | 0/0/0/0 | 0/10 | 0/2 | — | unexplored | — |
| bank14:D10C-D216 | solid | 引擎(R-E) | 10/134 | 0/0/0/1 | 2/14 | 1/2 | — | core-cited | core:flow; spec×2; find×3 |
| bank14:D217-D238 | solid | 引擎(R-E) | 0/18 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank14:D239-D259 | solid | 引擎(R-E) | 0/16 | 0/0/0/0 | 0/3 | 1/0 | — | unexplored | — |
| bank14:D25A-D26D | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/4 | 0/0 | — | unexplored | — |
| bank14:D26E-D275 | noise-suspect | 引擎(R-E) | 0/4 | 0/0/0/0 | 0/2 | 0/1 | — | unexplored | — |
| bank14:D276-D2DA | noise-suspect | 引擎(R-E) | 0/46 | 0/0/0/0 | 0/11 | 2/0 | — | unexplored | — |
| bank14:D2DB-D322 | noise-suspect | 引擎(R-E) | 0/34 | 0/0/0/0 | 0/8 | 2/2 | — | unexplored | — |
| bank14:D323-D325 | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank14:D326-D32B | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/0 | 1/0 | — | unexplored | — |
| bank14:D32C-D3D3 | solid | 引擎(R-E) | 1/90 | 0/0/0/0 | 2/2 | 0/0 | — | unexplored | — |
| bank14:D3D4-D421 | solid | 待定(R-?) | 0/38 | 0/0/0/0 | 1/0 | 0/0 | — | unexplored | — |
| bank14:D422-D4FD | solid | 数据(R-D) | 4/107 | 0/0/0/2 | 3/9 | 0/1 | — | unexplored | — |
| bank14:D4FE-D514 | solid | 引擎(R-E) | 1/12 | 0/0/0/0 | 0/2 | 1/0 | — | findings-cited | find×1 |
| bank14:D515-D547 | solid | 引擎(R-E) | 0/26 | 0/0/0/0 | 0/8 | 0/1 | — | spec-cited | spec×1; find×2 |
| bank14:D548-D5E6 | solid | 引擎(R-E) | 0/82 | 0/0/0/0 | 0/13 | 0/0 | — | unexplored | — |
| bank14:D5E7-D5F2 | solid | 引擎(R-E) | 0/5 | 0/0/0/0 | 0/1 | 1/0 | — | unexplored | — |
| bank14:D5F3-D5FC | noise-suspect | 引擎(R-E) | 0/5 | 0/0/0/0 | 0/1 | 1/0 | — | unexplored | — |
| bank14:D5FD-D60D | noise-suspect | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/1 | 2/1 | — | unexplored | — |
| bank14:D60E-D615 | solid | 引擎(R-E) | 0/3 | 0/0/0/0 | 0/0 | 2/1 | — | core-cited | core:flow; spec×1; find×2 |
| bank14:D616-D62E | solid | 引擎(R-E) | 0/11 | 0/0/0/0 | 1/3 | 0/0 | — | core-cited | core:flow; spec×1; find×1 |
| bank14:D62F-D673 | solid | 引擎(R-E) | 0/33 | 0/0/0/0 | 1/6 | 1/0 | — | core-cited | core:flow; spec×1; find×3 |
| bank14:D674-D678 | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/0 | 1/1 | — | findings-cited | find×2 |
| bank14:D679-D6C5 | solid | 引擎(R-E) | 0/34 | 0/0/0/0 | 0/5 | 2/0 | — | unexplored | — |
| bank14:D6C6-D6CB | solid | 引擎(R-E) | 0/3 | 0/0/0/0 | 0/2 | 0/0 | — | unexplored | — |
| bank14:D6CC-D734 | solid | 引擎(R-E) | 0/47 | 0/0/0/0 | 0/8 | 4/0 | — | unexplored | — |
| bank14:D735-D746 | solid | 引擎(R-E) | 0/9 | 0/0/0/0 | 0/0 | 1/0 | — | unexplored | — |
| bank14:D747-D74B | noise-suspect | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/0 | 1/0 | — | unexplored | — |
| bank14:D74C-D75C | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/1 | 1/0 | — | unexplored | — |
| bank14:D75D-D769 | solid | 引擎(R-E) | 0/5 | 0/0/0/0 | 0/0 | 2/1 | — | unexplored | — |
| bank14:D76A-D778 | solid | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/1 | 0/0 | — | findings-cited | find×1 |
| bank14:D779-D77A | solid | 引擎(R-E) | 0/1 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank14:D77B-D78A | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/1 | 1/2 | — | unexplored | — |
| bank14:D78B-D7A2 | solid | 引擎(R-E) | 0/11 | 0/0/0/0 | 2/3 | 1/0 | — | findings-cited | find×2 |
| bank14:D7A3-D7EC | solid | 引擎(R-E) | 0/39 | 0/0/0/0 | 0/5 | 0/2 | — | findings-cited | find×2 |
| bank14:D7ED-D84E | noise-suspect | 引擎(R-E) | 0/52 | 0/0/0/0 | 2/5 | 2/2 | — | findings-cited | find×1 |
| bank14:D84F-D85F | noise-suspect | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/0 | 2/3 | — | unexplored | — |
| bank14:D860-D93A | noise-suspect | 引擎(R-E) | 41/86 | 0/0/0/0 | 0/15 | 1/2 | — | findings-cited | find×3 |
| bank14:D93B-DA03 | solid | 引擎(R-E) | 0/97 | 0/0/0/0 | 0/21 | 9/0 | — | findings-cited | find×3 |
| bank14:DA04-DAE6 | no-caller | 引擎(R-E) | 1/121 | 0/0/0/1 | 5/8 | 1/1 | — | unexplored | — |
| bank14:DAE7-DB13 | solid | 引擎(R-E) | 0/22 | 0/0/0/0 | 0/1 | 0/1 | — | core-cited | core:ui; spec×3; find×1 |
| bank14:DB14-DB4B | solid | 引擎(R-E) | 0/30 | 0/0/0/0 | 0/4 | 1/3 | — | unexplored | — |
| bank14:DB4C-DB4F | solid | 待定(R-?) | 0/2 | 0/0/0/0 | 0/0 | 0/1 | — | spec-cited | spec×1; find×1 |
| bank14:DB50-DB5D | noise-suspect | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/0 | 2/1 | — | unexplored | — |
| bank14:DB5E-DBCD | no-caller | 引擎(R-E) | 0/54 | 0/0/0/0 | 0/7 | 2/4 | — | unexplored | — |
| bank14:DBCE-DDCE | solid | 引擎(R-E) | 7/305 | 0/0/0/0 | 1/8 | 0/1 | — | L2-diff | L2:money_add_event(4); core:ui; spec×3; find×5 |
| bank14:DDCF-DFFF | solid | 待定(R-?) | 0/187 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:E000-E072 | root | 音频(R-A) | 0/51 | 3/1/0/2 | 0/7 | 5/3 | RESET | core-cited | core:flow; find×3 |
| bank15:E073-E0D8 | solid | 引擎(R-E) | 2/46 | 0/0/0/2 | 1/9 | 6/1 | — | spec-cited | spec×1; find×11 |
| bank15:E0D9-E16A | solid | 引擎(R-E) | 0/69 | 0/0/0/0 | 0/22 | 11/0 | — | core-cited | core:flow; spec×5; find×11 |
| bank15:E16B-E21B | solid | 引擎(R-E) | 1/72 | 0/0/0/1 | 1/20 | 18/1 | — | core-cited | core:flow; spec×2; find×11 |
| bank15:E21C-E238 | solid | 引擎(R-E) | 0/13 | 0/0/0/0 | 0/0 | 1/0 | — | unexplored | — |
| bank15:E239-E2F2 | solid | 引擎(R-E) | 0/74 | 0/0/0/0 | 0/14 | 12/1 | — | core-cited | core:flow; spec×1; find×6 |
| bank15:E2F3-E3AA | solid | 引擎(R-E) | 0/82 | 0/0/0/0 | 0/12 | 9/1 | — | core-cited | core:flow; spec×2; find×6 |
| bank15:E3AB-E424 | solid | 引擎(R-E) | 0/53 | 0/0/0/0 | 0/6 | 18/1 | — | core-cited | core:flow; spec×2; find×3 |
| bank15:E425-E486 | no-caller | 引擎(R-E) | 0/43 | 0/0/0/0 | 0/3 | 9/0 | — | core-cited | core:flow; spec×1; find×4 |
| bank15:E487-E4E2 | solid | 引擎(R-E) | 1/40 | 0/0/0/0 | 0/5 | 0/0 | — | findings-cited | find×1 |
| bank15:E4E3-E4FF | solid | 待定(R-?) | 0/13 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:E500-E55C | solid | 引擎(R-E) | 0/49 | 0/0/0/1 | 0/7 | 1/1 | — | findings-cited | find×1 |
| bank15:E55D-E560 | solid | 待定(R-?) | 0/2 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:E561-E566 | solid | 图形(R-G) | 0/3 | 1/0/0/0 | 0/1 | 0/0 | RESET | unexplored | — |
| bank15:E567-E57F | solid | 图形(R-G) | 0/11 | 3/0/0/0 | 0/1 | 1/3 | — | unexplored | — |
| bank15:E580-E589 | solid | 图形(R-G) | 0/5 | 1/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank15:E58A-E58F | solid | 待定(R-?) | 0/3 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:E590-E596 | solid | 引擎(R-E) | 0/3 | 0/0/0/0 | 0/0 | 1/0 | — | unexplored | — |
| bank15:E597-E59C | solid | 图形(R-G) | 0/3 | 1/0/0/0 | 0/1 | 0/0 | RESET | unexplored | — |
| bank15:E59D-E5BF | solid | 引擎(R-E) | 0/20 | 0/0/0/0 | 0/4 | 1/0 | — | unexplored | — |
| bank15:E5C0-E5D8 | solid | 引擎(R-E) | 0/13 | 0/0/0/0 | 0/1 | 0/1 | — | unexplored | — |
| bank15:E5D9-E600 | solid | 引擎(R-E) | 0/21 | 0/0/0/0 | 0/2 | 2/2 | — | unexplored | — |
| bank15:E601-E60B | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/1 | 0/1 | — | unexplored | — |
| bank15:E60C-E61E | solid | 引擎(R-E) | 0/9 | 0/0/0/2 | 0/1 | 0/0 | RESET | unexplored | — |
| bank15:E61F-E62D | solid | 引擎(R-E) | 0/7 | 0/0/0/2 | 0/1 | 0/0 | — | unexplored | — |
| bank15:E62E-E63C | solid | 引擎(R-E) | 0/7 | 0/0/0/2 | 0/1 | 0/0 | — | findings-cited | find×1 |
| bank15:E63D-E648 | solid | 引擎(R-E) | 0/5 | 0/0/0/2 | 0/2 | 0/0 | — | findings-cited | find×1 |
| bank15:E649-E671 | solid | 引擎(R-E) | 0/25 | 0/0/0/0 | 0/0 | 3/0 | — | spec-cited | spec×1; find×5 |
| bank15:E672-E691 | solid | 引擎(R-E) | 0/17 | 0/0/0/0 | 0/2 | 2/0 | — | spec-cited | spec×1; find×2 |
| bank15:E692-E698 | solid | 引擎(R-E) | 0/4 | 0/0/0/0 | 0/1 | 0/0 | RESET | unexplored | — |
| bank15:E699-E6C6 | solid | 引擎(R-E) | 0/26 | 0/0/0/0 | 0/5 | 0/0 | — | unexplored | — |
| bank15:E6C7-E6D8 | solid | 引擎(R-E) | 0/10 | 0/0/0/0 | 0/2 | 1/1 | — | findings-cited | find×2 |
| bank15:E6D9-E6DE | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/0 | 1/0 | — | findings-cited | find×2 |
| bank15:E6DF-E6E2 | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/1 | 0/0 | — | spec-cited | spec×1; find×2 |
| bank15:E6E3-E769 | solid | 引擎(R-E) | 0/69 | 0/0/0/0 | 0/13 | 2/3 | — | findings-cited | find×2 |
| bank15:E76A-E7B2 | no-caller | 引擎(R-E) | 0/41 | 0/0/0/0 | 0/8 | 3/0 | — | findings-cited | find×1 |
| bank15:E7B3-E7BD | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/1 | 1/0 | — | unexplored | — |
| bank15:E7BE-E7CB | solid | 待定(R-?) | 0/7 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:E7CC-E7CF | solid | 待定(R-?) | 0/2 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:E7D0-E7DB | solid | 引擎(R-E) | 0/10 | 0/0/0/0 | 0/0 | 1/0 | — | findings-cited | find×1 |
| bank15:E7DC-E7EC | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/0 | 1/0 | — | spec-cited | spec×1; find×1 |
| bank15:E7ED-E7FA | solid | 图形(R-G) | 0/7 | 1/0/0/0 | 0/0 | 0/2 | — | findings-cited | find×1 |
| bank15:E7FB-E804 | solid | 引擎(R-E) | 0/5 | 0/0/0/0 | 0/4 | 0/0 | — | unexplored | — |
| bank15:E805-E812 | solid | 引擎(R-E) | 0/8 | 0/0/0/0 | 0/2 | 1/0 | — | unexplored | — |
| bank15:E813-E827 | solid | 引擎(R-E) | 0/12 | 0/0/0/0 | 0/3 | 0/0 | NMI | findings-cited | find×2 |
| bank15:E828-E851 | solid | 引擎(R-E) | 0/21 | 0/0/0/0 | 0/1 | 1/0 | — | spec-cited | spec×1; find×1 |
| bank15:E852-E8B0 | solid | 引擎(R-E) | 0/50 | 0/0/0/0 | 0/8 | 3/3 | — | unexplored | — |
| bank15:E8B1-E8C5 | solid | 引擎(R-E) | 0/11 | 0/0/0/0 | 0/4 | 0/0 | — | findings-cited | find×1 |
| bank15:E8C6-E91A | no-caller | 引擎(R-E) | 0/43 | 0/0/0/0 | 0/13 | 2/2 | — | unexplored | — |
| bank15:E91B-E920 | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/0 | 1/0 | — | unexplored | — |
| bank15:E921-E94E | solid | 引擎(R-E) | 0/27 | 0/0/0/0 | 0/0 | 2/1 | — | unexplored | — |
| bank15:E94F-E95D | solid | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/2 | 1/0 | — | unexplored | — |
| bank15:E95E-E972 | solid | 引擎(R-E) | 0/11 | 0/0/0/0 | 0/4 | 0/0 | — | unexplored | — |
| bank15:E973-E97F | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/1 | 1/0 | — | unexplored | — |
| bank15:E980-E9DD | noise-suspect | 引擎(R-E) | 0/49 | 0/0/0/0 | 0/7 | 0/2 | — | spec-cited | spec×1 |
| bank15:E9DE-E9F9 | solid | 待定(R-?) | 0/17 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:E9FA-EA01 | solid | 待定(R-?) | 0/6 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:EA02-EAAE | solid | 引擎(R-E) | 0/87 | 0/0/0/3 | 3/13 | 1/0 | — | unexplored | — |
| bank15:EAAF-EAE8 | solid | 引擎(R-E) | 0/33 | 0/0/0/0 | 0/8 | 0/0 | — | unexplored | — |
| bank15:EAE9-EB1D | solid | 引擎(R-E) | 0/30 | 0/0/0/0 | 0/3 | 0/0 | — | unexplored | — |
| bank15:EB1E-EB89 | solid | 引擎(R-E) | 0/57 | 0/0/0/0 | 0/12 | 1/2 | — | findings-cited | find×1 |
| bank15:EB8A-EBC1 | solid | 引擎(R-E) | 0/33 | 0/0/0/0 | 1/8 | 1/0 | — | unexplored | — |
| bank15:EBC2-EBF0 | solid | 引擎(R-E) | 0/28 | 0/0/0/0 | 0/2 | 1/4 | — | unexplored | — |
| bank15:EBF1-EC11 | solid | 引擎(R-E) | 2/17 | 0/0/0/0 | 0/0 | 1/0 | — | unexplored | — |
| bank15:EC12-EC1E | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/2 | 0/0 | — | findings-cited | find×2 |
| bank15:EC1F-EC1F | solid | 引擎(R-E) | 0/1 | 0/0/0/0 | 0/1 | 0/0 | — | findings-cited | find×2 |
| bank15:EC20-ECBE | solid | 数据(R-D) | 0/67 | 0/0/0/0 | 2/20 | 0/1 | — | unexplored | — |
| bank15:ECBF-ECC9 | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/1 | 0/1 | RESET | findings-cited | find×3 |
| bank15:ECCA-ECD8 | solid | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/5 | 0/0 | — | unexplored | — |
| bank15:ECD9-ED02 | solid | 待定(R-?) | 0/24 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:ED03-ED26 | solid | 待定(R-?) | 0/21 | 0/0/0/0 | 0/0 | 0/1 | — | L2-diff | L2:bcd_sub_ammo(6); core:l2ports; spec×1; find×1 |
| bank15:ED27-ED2F | solid | 引擎(R-E) | 0/5 | 0/0/0/0 | 0/1 | 0/1 | — | unexplored | — |
| bank15:ED30-ED3A | solid | 引擎(R-E) | 0/6 | 0/0/0/0 | 0/1 | 1/1 | — | unexplored | — |
| bank15:ED3B-ED55 | solid | 引擎(R-E) | 0/14 | 0/0/0/0 | 0/1 | 0/2 | — | unexplored | — |
| bank15:ED56-ED64 | solid | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/0 | 2/0 | — | unexplored | — |
| bank15:ED65-ED6B | solid | 引擎(R-E) | 0/3 | 0/0/0/0 | 0/1 | 1/0 | — | unexplored | — |
| bank15:ED6C-EDAF | solid | 引擎(R-E) | 0/40 | 0/0/0/0 | 0/3 | 0/2 | — | unexplored | — |
| bank15:EDB0-EDEB | solid | 引擎(R-E) | 0/30 | 0/0/0/0 | 0/5 | 6/1 | — | unexplored | — |
| bank15:EDEC-EDF0 | solid | 待定(R-?) | 0/3 | 0/0/0/0 | 0/0 | 0/1 | — | unexplored | — |
| bank15:EDF1-EE02 | solid | 引擎(R-E) | 0/10 | 0/0/0/0 | 0/2 | 0/1 | — | unexplored | — |
| bank15:EE03-EE1C | solid | 引擎(R-E) | 0/14 | 0/0/0/0 | 0/1 | 4/1 | — | unexplored | — |
| bank15:EE1D-EE1F | solid | 引擎(R-E) | 0/2 | 0/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank15:EE20-EE58 | solid | 引擎(R-E) | 0/27 | 0/0/0/0 | 1/2 | 0/1 | — | unexplored | — |
| bank15:EE59-EECB | solid | 引擎(R-E) | 0/57 | 0/0/0/0 | 0/4 | 0/1 | — | unexplored | — |
| bank15:EECC-EED6 | solid | 引擎(R-E) | 0/5 | 0/0/0/0 | 0/1 | 0/1 | — | unexplored | — |
| bank15:EED7-EEE4 | solid | 引擎(R-E) | 0/7 | 0/0/0/0 | 0/2 | 0/1 | — | unexplored | — |
| bank15:EEE5-EEFA | solid | 引擎(R-E) | 0/12 | 0/0/0/0 | 0/3 | 0/2 | — | unexplored | — |
| bank15:EEFB-EF84 | solid | 引擎(R-E) | 8/77 | 0/0/0/0 | 5/1 | 0/0 | — | unexplored | — |
| bank15:EF85-F02F | root | 图形(R-G) | 0/78 | 9/0/1/16 | 0/4 | 4/0 | NMI | spec-cited | spec×3; find×4 |
| bank15:F030-F0DB | solid | 图形(R-G) | 0/73 | 20/0/0/0 | 0/3 | 1/3 | NMI | core-cited | core:terrain; spec×1; find×3 |
| bank15:F0DC-F14E | solid | 图形(R-G) | 0/47 | 16/0/0/0 | 0/1 | 0/1 | NMI | spec-cited | spec×1; find×1 |
| bank15:F14F-F156 | solid | 引擎(R-E) | 0/4 | 0/0/0/0 | 0/1 | 0/1 | NMI | unexplored | — |
| bank15:F157-F1DB | solid | 图形(R-G) | 0/58 | 19/0/0/0 | 0/0 | 0/1 | — | findings-cited | find×1 |
| bank15:F1DC-F1EA | solid | 图形(R-G) | 0/7 | 1/0/0/0 | 0/1 | 0/0 | — | unexplored | — |
| bank15:F1EB-F20A | solid | 图形(R-G) | 0/14 | 4/0/0/0 | 0/0 | 0/1 | — | unexplored | — |
| bank15:F20B-F250 | root | 引擎(R-E) | 0/37 | 0/0/0/10 | 0/0 | 1/0 | IRQ | findings-cited | find×1 |
| bank15:F251-F253 | solid | 待定(R-?) | 0/1 | 0/0/0/0 | 0/0 | 0/0 | IRQ/NMI | findings-cited | find×1 |
| bank15:F254-F2D5 | solid | 引擎(R-E) | 0/62 | 0/0/0/0 | 0/7 | 8/2 | — | findings-cited | find×3 |
| bank15:F2D6-F9D7 | solid | 引擎(R-E) | 24/724 | 0/0/0/9 | 13/22 | 4/4 | — | unexplored | — |
| bank15:F9D8-FE1F | solid | 引擎(R-E) | 274/427 | 0/0/0/4 | 5/21 | 6/0 | — | unexplored | — |
| bank15:FE20-FE44 | solid | 待定(R-X) | 37/0 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
| bank15:FE45-FFFE | solid | 待定(R-X) | 442/0 | 0/0/0/0 | 0/0 | 0/0 | — | findings-cited | find×1 |
| bank15:FFFF-FFFF | solid | 待定(R-X) | 1/0 | 0/0/0/0 | 0/0 | 0/0 | — | unexplored | — |
