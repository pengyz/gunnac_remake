# 第四候选判别实验收果 —— 中距单发 ×10：受击保护/命中冷却（arena-midboss §2.3 收口）

> 日期：2026-10-15｜角色：verifier（test-runner，流水线第 3 棒收果）
> 判据来源：[b2v4-bot-closure.md](b2v4-bot-closure.md) §4 预登记判别实验（"中距对准单发 ×10 + 逐帧 watch `$064B/$05DC/$05F7/$04DF/$04C`；
> 同框消失+全字段 0 变化连续出现 ⇒ 坐实；逐发 −2 ⇒ 排除"）+ [adversarial-review-82.md](adversarial-review-82.md) §4/§6-1 判据修订
> （**判据=游戏侧写点非几何框**——模型框≠游戏命中框，f10068 反例）。
> 硬红线：无 poke（纯按键轨，解析器 fail-closed）；不改 ROM/台账/门禁/golden。证据分级依 AGENTS.md §3。

## 0. 定谳：**排除**（受击保护/命中冷却作为 0 差分解释被证伪）——机制本身坐实存在，但吞伤样本 = 0

| 预登记分支 | 触发条件 | 实测 | 触发 |
|---|---|---|---|
| 坐实 | 同框消失 + 全字段 0 变化连续出现（修订后：**游戏侧命中注册** + 0 伤害） | 22 发连续 0 差分 ✓，但**命中注册写点 0/22**（修订判据的"命中"前提不成立） | ❌ |
| 排除 | 逐发 −2（全中） | 0/22（字面条件未出现） | ❌（字面） |
| **机制级判别（本轮新增，修订判据的直接应用）** | 0 差分事件是否落在**受击保护窗**内、且经游戏侧写点确认"注册后被吞伤" | 保护窗实测 **f10068-10078（10 帧）**；35 发 boss 近旁消费中 **34 发在窗外**、1 发窗内但也**未过命中测试**；窗内窗外均无任何"注册+吞伤"签名 | ✅ **排除** |

**一句话**：受击保护（`$6D8,X` 受击倒数，中 boss 命中后置 10、1/帧衰减）是**真实机制**（bank14 静态 + 逐帧动态双证），
但它在本窗**没有吞掉任何一次已注册命中**；22 发中距单发与首击前 19 发的 0 差分全部是**游戏侧根本未注册命中**
（清弹框 ≠ 游戏命中框）——主因 = **命中覆盖率**（与 arena-midboss §2.3 "命中覆盖率"单因收敛、W-tas "命中 1/7 发所需"同向闭环）。

## 1. 实验面（三 run 全部在盘，可复算）

| run | 角色 | 输入 | 产物 |
|---|---|---|---|
| `tas-arena-shots`（**预登记判别轨**） | 中距单发 ×10 | stale 录制轨 f≤10070 原样（含 f10068 −2 正控）+ f>10070 仅改 B = 4f 点按 ×10（10074/82/90/98/1006/114/122/130/138/146） | `analysis/traces/tas-arena-shots.*`（run.json 记 sha）；frames sha256 `4710346ab3478323…` |
| `tas-arena-repro`（复现对照轨） | stale 10800 帧开环复现 | stale inputs 原样（record-to-scenario） | `analysis/traces/tas-arena-repro.*`；frames sha256 `cf778c190de3d154…`（复现轨无 run.json——前实施体中断在清单落盘前，已如实登记） |
| `tas-arena-shots2`（**续观测轨**，判别钥匙） | 同输入轨 + 观察面扩展 | `tas-arena-shots` 输入**逐字节同源**（动词行仅 `name` 不同，自算核验），新增 `watch 06f3`（受击保护倒数）/`watch_range 06e8-0708`（消费标记）/`probe_mem 06d8 06f8`/`probe_mem 0668 0708` | `analysis/traces/tas-arena-shots2.*`（run.json 记 sha）；frames sha256 `c94f1947aff8d938…` |

- **确定性**：shots vs shots2 共享 39 字段 × 10800 帧 **0 mismatch**（扩展观察面不改行为）；shots f≤10070 vs repro 39 字段 **0 mismatch**（正控前全等），首分歧 f10072（`held`，设计内）。
- **帧坐标系**（重要）：probe.jsonl 的 `f` = `st.frameCount`（模拟器绝对帧），watch 面 `f` = harness 计数 ⇒ **恒定偏移 +14**（自算核验：全部 78 个 HP 变化事件的 `$064B` 写点均落 watch f+14，如 watch f10068 ↔ probe f10082）。本文一律用 **watch 帧坐标**（w 表示已折算）。
- **probe 语义**：Mesen2 回调在本 harness 下记录的是**目标地址的访问**（读写皆有，PC=访问指令邻域），因此判别用**签名簇**（如伤害链 `D6F9/D704/D70E/ACBB`）而非单条访问。

## 2. 数据（A 级：全部自算，脚本可复现 §6）

### 2.1 正控（同 run 内自证伤害路径可用）

f≤10070 与 repro/stale 全等 ⇒ f10068 事件完整保留：**HP 14→12**，游戏侧写点链 =
`D6F9@$004C`（y=130=命中弹槽，x=27=boss 槽）→ `D704/D70E@$064B`（14）→ `ACBB@$064B`（12，A=10），
同帧 `D71A@$06F3`（受击保护倒数 := **10**）。弹 = yslot130 tcode0 f10060→f10068，消失点 (101,134) vs boss (102,116)。

### 2.2 中距单发 ×10：22 发逐发（判据 = 游戏侧写点）

B 点按 ×10 产出 **22 发**（每 tap 1 主弹 tcode0 + 0-2 子母弹 tcode1/2）：**15 发在 boss 近旁被消费**
（10 主弹全数，die−boss 11-24px，与正控 18px 同量级；另 5 子母弹同带），7 子母弹出画。逐发 **ΔHP = 0**
（`$064B` 自 w10068 后恒 12 至 run 末 f10800）。游戏侧写点（判据面）：

| 判别面 | 正控 f10068 | tap 22 发（w10075-w10165） |
|---|---|---|
| 伤害链 `D6F9/D704/D70E/ACBB@$064B/$004C` | ✅ 全链触发 | **0 次**（tap 窗 `$064B` 访问 = 空集） |
| `$06F3` 受击保护倒数 | ✅ 0→10（`D71A`） | 窗外恒 0（无 SET/吞伤） |
| take-hit 消费标记 `ORA #$20/$668+Y`（`D756/D75B`） | —（伤害路径自有标记） | **0 次**（全 run 仅 w10202 一次且 Y=0 非弹槽） |
| `$05F7`（`$5DC+27` 命中态） | 无写（恒 4） | 无写（恒 4） |
| `$05DC/$04DF/$04C` | 仅周期性背景写点 | 仅周期性背景写点（`A0FF/A0F2`、`CC54/CC5E`、`AC81`@4f） |

⇒ 22/22 **未过游戏命中测试**（既无伤害链、也无 take-hit 路径标记）——0 差分的原因是**未注册**，不是**注册后被吞**。

### 2.3 受击保护倒数（判别钥匙，shots2 逐帧）

`$06F3`（=`$6D8,X` X=27）：**w10068: 0→10（正控命中置位）→ w10069-10078 每帧 DEC 1 → w10078 归零**。
受保护路径（`D73A/D73D`）访问仅出现在 **w10069-w10077**（倒数存续期），且其中**无一命中测试通过**（无 `D75B` 标记）。
tap 10 发主弹消费帧 = w10082/10089/10097/10105/10112/10120/10127/10134/10142/10149，**全部在保护窗外**；
窗内唯一 boss 近旁消费 = yslot128 f10067→f10075（w10075，倒数=3）——亦未过命中测试，**不构成"保护吞伤"样本**。
⇒ **保护吞伤样本 = 0**。

### 2.4 首击前对照（保护未激活段，同 run 自证）

boss w9916 HP14 入场后至正控前，**19 发主弹**在 boss 近旁被消费（w9945-w10062，die−boss 9-18px，
消失点均在屏内 y>8）且 **ΔHP 全 0、伤害链 0 次、`$06F3`=0**（保护未激活、`$05F7`=4 伤害门开着）。
⇒ 与保护无关的"消费≠命中"在**首击之前**已是常态——"被击后保护"解释不了这段。

### 2.5 同 run 其他敌对照（全局伤害路径无冷却）

同工具态 run 内：type `$50` 敌 f9207/9214/9220/9226 **四连 −2（间隔 6-7f）**、type `$49` f4546/f4557（11f）、
f7792/f7804（12f）、f8807/f8827（20f）——伤害链写点逐次触发、`D71A` 逐次置倒数。⇒ 伤害路径本身可在数帧级
间隔重复生效；中 boss 侧的稀疏是**注册率**问题（与 b2v4 §4 F4-1 对照同向，出处已按复核 #82 更正）。

### 2.6 RNG 假设排除

`rng_2b/2c/2d` 为自由运行计数（+1/帧，全窗无逐发掷点）；消费帧字段态全同（attr=19/hitctr=4/cnt=128/dmg=1/form=51）。
⇒ "命中概率掷点"在观察面上无支撑。

## 3. 机制考证（静态 listing × 动态 probe 交叉，B 级归因）

`reference/seed/gunnac.asm` bank14 `sub_D6CC`（伤害/受击处理，probe 签名簇落点吻合）：

```
D6CC  STY $12              ; 攻方参数
D6CE  LDA $6D8,X           ; ★受击保护倒数（每对象一字节，$6D8+X）
D6D1  BEQ D6D8             ; =0 ⇒ 走命中测试
D6D3  DEC $6D8,X           ; >0 ⇒ 衰减（实测 1/帧）
D6D6  BNE loc_D73A         ;   仍>0 ⇒ 受保护路径（无伤，`D73A`）
D6D8  JSR sub_D78B         ; ★命中测试（游戏命中框）
...
D701  LDA $630,X           ; HP（X=27 ⇒ $64B）
D705  SBC $12              ; 减伤
D70B  STA $630,X           ; 写回（正控 ACBB/D70E 签名）
D715  LDA $13
D717  STA $6D8,X           ; ★命中后置保护倒数 := $13
```

- take-hit 入口多处 `JSR $D6C6`（各对象型码自传保护时长）；中 boss 段见 `LDA #$A`（=10）——与动态实测
  **置 10**、RAM dump zp `$13`=10、`$10`=$FF（`D6C6` 入口签名）三方吻合。
- 受保护路径 `D73A` 仍调 `sub_D78B`，命中才走 `D74C` 消费标记（`ORA #$20/$668,Y`）——**注册+吞伤会留标记**；
  实测 tap 窗 0 标记 ⇒ 连"注册"都未发生。
- **遗留（不发明）**：`sub_D78B` 命中框几何（清弹框 vs 命中框的尺寸/形状/量化步进）未考证——属
  decomp-analyst 取证面（见 §4 路由）。bank 归属（probe PC → listing bank）为 B 级邻域吻合，
  精确 bank 映射留 W 项。

## 4. 路由与遗留

| 去向 | 事项 |
|---|---|
| **arena-midboss §2.3** | 第四候选**正式排除**（作为解释）；"命中覆盖率"单因收敛补 A 级动态证据（消费≠命中：35 消费 1 注册） |
| **decomp-analyst** | `sub_D78B` 游戏命中框考证（清弹框/命中框/量化步进）→ 命中覆盖率几何根因；`$5DC,X`（W18）语义 |
| **spec-writer** | 受击保护机制登记（`$6D8,X`，命中后 := 保护时长（型别参数；中 boss=10f），窗内命中走无伤路径+标记）→ enemy-system/shot-system R 条目 |
| **port-engineer** | game-core 命中判定必须用游戏命中框语义（不得用 BOX_DEFAULT 近似替代）；W15′/Wt-6 接线 |
| **adversarial-reviewer** | 本收果 + F3-1/F3-2/F4-1 修 + attrib 新字段 + 归档机制（task-2） |

**遗留假设清单（C 级，不入结论）**：① 清弹框为独立大框（消费于框缘 11-24px）——未取证；
② 正控何以通过命中测试（量化步进落框 vs 相位弱点）——未取证，恰是命中覆盖率根因；
③ 长间隔 tap（>10f 间距）是否改变注册率——未测（本轮 8f 间距设计）。

## 5. 工具态留档事故（诚实登记，2026-10-15）

本轮 `tas-arena-shots2` 首跑复制场景时**漏改 `name` 行**（仍为 `tas-arena-shots`）⇒ 产物前缀撞车：
原 shots run 产物被 `archive_existing` 整体归档至 `.archive/tas-arena-shots-20261015-153445/`（**frames/run.json 无损**），
但 **原 `tas-arena-shots.probe.jsonl` 与 `analysis/_gen/tas-arena-shots.lua` 被覆盖**（旧归档模式未含这两类）。
处置：① 撞车产物移入 `.archive/tas-arena-shots-misnamed-20261015-154200/`；② 原 shots 产物全量复位（run.json sha 链完好）；
③ 生成 lua 用 `scenario_run.py gen tas-arena-shots` **重生成，sha256 `1ab6f574a7db…` 与 run.json 记录逐字节一致** ⇒ 无"旧态不可复原"；
④ probe 内容由 shots2 超集（同输入确定性）+ repro probe 可复现；⑤ 名称修正后重跑 shots2（净名产物）。
**堵漏**：`scenario_run.py` 已改——生成 lua 每 run 同步落 `analysis/traces/<name>.gen.lua` 持久副本、
随 run 归档（`archive_existing` 迁移 `.gen.lua`/`.probe.jsonl`），manifest 增 `generated_lua_archive` 字段。
教训与复核 #82 建议 4 同源：**工具态留档必须机器化，不能靠自觉**。

## 6. 复现命令（本轮实际执行）

```bash
# 判别三 run（顺序执行——Mesen 串行纪律）
python3 tools/scenario_run.py run tas-arena-shots     # 预登记判别轨（已跑，产物在盘）
python3 tools/scenario_run.py run tas-arena-shots2    # 续观测轨（已跑，净名产物在盘）
python3 tools/arena_discriminant.py analyze tas-arena-shots    # 逐发判别表 → .disc.json
python3 tools/arena_discriminant.py analyze tas-arena-shots2   # 同上（含保护倒数/消费标记面）
# 关键数字（自算脚本口径，本文 §2）
#   保护倒数：读 frames.jsonl 的 boss_protectctr 字段（shots2）——w10068 0→10，w10078 归零
#   命中注册：probe.jsonl 过滤 pc∈{D6F9,D704,D70E,ACBB,AA65,D857,D71A,D756,D75B}
#   帧坐标：probe f − 14 = watch f（锚：HP 写点 D70E/ACBB 全部落 watch HP 变化帧）
# 三件 + 单测（验收⑤）
cargo test --workspace        # 144 passed / 0 failed（基线 144）
python3 tools/diff_subroutines.py   # L2-DIFF 121/121
python3 gates/cli.py check    # SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0；CHECK: PASS
python3 tools/test_tas_bot.py # 9/9
python3 tools/tas_bot.py attrib tas-stg01-bot1   # hp_delta_ge2_unattributed=11 @ [4440,…,8412]
```

## 7. 纪律自查

- 零 ROM/台账/门禁/golden 改动；三场景**纯按键**（无 poke；`bot+poke` 混入 fail-closed 由解析器锁）。
- **不放宽阈值不加白名单**：判别用写点签名簇，无任何"相似度"式判定。
- 归因诚实：0 差分**不粉饰为**"命中被保护"（注册写点 0/22 明示）；正控唯一注册如实单列；首击前 19 发对照不省略；
  `sub_D78B` 几何根因**显式挂账**不发明。
- 分级：§2 全部数字 = A（三 run frames/probe 在盘可重导）；§3 机制 = B（listing×probe 交叉，bank 邻域吻合）；
  §4 遗留 = C（明示）。工具态事故 = A（sha 复验链）。
