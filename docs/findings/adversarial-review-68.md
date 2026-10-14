# 对抗复核 #68：DEV-1 修复效果复审（第二轮·fresh context·证伪立场）

> 被审：commit `e5066e0`（`fix(core): DEV-1 修复循环——AR65-1 隐藏门两触发源 + AR65-2 消息 ack 相位（review #65)`，
> `ui.rs` +49/−~12、`weapons.rs` +4、`tests/shop_purchase.rs` +76/−~10、`docs/spec/shop-purchase.md` ±20）
> + `d8ba360`（`chore(data): ratchet rust_test_count 122→125`，baseline.json/SHA256SUMS/baseline_history.jsonl）。
> 复审对象 = review #65 判 needs_revision 后的修复批（AR65-1 中·必修 + AR65-2 低·必修 + AD65-1/2/3）。
> 审核方：独立 verifier #68（fresh context subagent，实施≠审核；不采信被审自述、不复用其结论、独立重导）。
> 核验清单（任务下发）：①AR65-1 双触发门+分歧态闭合+spec R-7 dated 更正 ②AR65-2 ExitHidden 第 7 变体+
> ack 相位档位注+触发⇒消息 ack 前不转场 ③新测试 3 条真验 RAM 终态 ④mutation 独立重放（≥2 真测）
> ⑤对拍三件独立复跑贴数 ⑥AD65-1/2/3 清净核。
> 数据源（只读）：`rom/gunnac_cn.nes`（sha256 门禁锚定，全程未改）、`crates/game-core/src/{ui,weapons}.rs`、
> `crates/game-core/tests/shop_purchase.rs`、`docs/spec/shop-purchase.md`、`analysis/tdd-red-dev1-ar65-20261014.log`、
> `data/status/baselines/*`。复现命令（hex 直读 `off=bank*0x2000+(cpu&0x1FFF)`、mutation 改→跑→还原、三条对拍）见 §7。
> 过程注：审中 HEAD 由 `7fb6f2f` 前移至 `fd0c1f0`（并行 agent 的 `docs/findings/`+`ending-sequence.md` 改动，
> `git diff --stat 7fb6f2f..fd0c1f0 -- crates/` = 0 行）——被审代码即本次复跑所测代码，测试面结论不受影响。

**裁定：pass**（附 1 条非阻断必补注 AR68-1 + 1 条低·可选 AR68-2）——六项核验逐项独立重导全部落地：
AR65-1 两触发源 OR 实现+分歧态闭合+spec R-7 dated 控制流改写三处一致（ROM 直读 `f0 07`/`d0 16` 双分支
本审手解坐实）；AR65-2 第 7 变体 `ExitHidden` 指针/ack 门登记+完整 ack 相位（非挂起枚举）+ack 前不转场
（含 trigger-(a)-alone 路径探针直验行为正确）；新测试 3 条断言 RAM 终态非自证；**mutation 本审独立实测 4 项：
3 红 1 存活**——`==8→==7` 复现审方 2 FAILED（逐断言同）、ack 相位跳过 2 红、双触发双写 1 红（0x40 哨兵真咬），
存活项为 trigger-(a)-alone 相位断言缺口（测试锁缺口，非行为分歧，记 AR68-1 非阻断必补注）；
对拍三件独立复跑贴数全绿（125/125、109/109、gates 12/12 + ratchet 125/125）；AD65-1/2/3 全清净
（指针更正字节级复核 `$9CF7`/`$9CF9` 精确、RED 工件在档、bit7 互注两侧齐全）。行为面无分歧。

## 0. 六项核验表

| # | 核验项 | 判定 | 本审独立证据 |
|---|---|---|---|
| ①AR65-1 | ui.rs 退出隐藏门双触发 `($18&$33)==$11 \|\| stage==8` 形态 | **成立** ✅ | ui.rs:399 `if st.pad2 & 0x33 == 0x11 \|\| st.stage == 8`；ROM 直读 bank01:$A023 `a5 18 29 33 c9 11 f0 07`（BEQ→$A032）+ `ad 80 01 c9 08 d0 16`（BNE→$A048，即 ==8 落 $A032）——两分支目标地址本审按位复算：$A02B+7=$A032、$A032+0x16=$A048，OR 语义无歧义 |
| ①AR65-1 | 分歧态闭合（stage=8,pad2=0 ⇒ $32 bit5 置位） | **成立** ✅ | 代码同一 OR 分支 `st.pickup.flags \|= 0x20`；测试 `exit_hidden_gate_stage8_triggers_with_message_ack` 断言 `flags==0x20`（RED-2 留痕 `0x00 vs 0x20` 即此分歧态）；mutation M1 实测该断言真红 |
| ①AR65-1 | spec R-7 dated 更正（两触发源 OR + `$A65C` + `$A0CF` 控制流）在文 | **成立** ✅ | spec:79-81：R-7 正文「**两触发源 OR**——`($18&$33)==$11` **或** `$0180==8`」+ 证据句含完整控制流（A032 音效→A037 `$32` bit5→A03D `text $A65C/JSR $A09F`→A048 合流）+ 【2026-10-14 更正】注明原证据范围「完整覆盖 A02B-A031 七字节却漏记 (b)」的完整性问题与改前分歧态 |
| ②AR65-2 | ShopPrompt 第 7 变体 ExitHidden（`$A65C` 指针登记 + ack 门 `$A0CF` 注） | **成立** ✅ | ui.rs:208-211 枚举注：`text $A65C`+`JSR $A09F`、ack 门 `bank01:$A0CF AND #$C0`、「同 R-shop-purchase-8 六类消息同一路等待语义」、「文本 `$A65C` 内容未证 ⇒ 登记不解释」；ROM 直读 $A03D `a9 5c 85 12 a9 a6 85 13`（指针 $A65C）、$A0CF `29 c0 f0 f4` 坐实 |
| ②AR65-2 | ack 相位实现档位注释 | **成立** ✅ | 实现档位=完整 ack 相位（非挂起枚举）：ui.rs:401「消息 ack 等待相位（第 7 变体 ExitHidden，文案未证只落枚举）」+ 枚举注的等待语义 A 级依据（同 R-8 六类同路）+ ShopMsgReturn::Exit 交 flow 注；commit body 有显式档位句（"完整 ack 相位（非挂起枚举）…仅文案未证故枚举不发明"）；与 `WingBoughtFarCallPending`（挂起枚举）对照可辨两档 |
| ②AR65-2 | 触发⇒消息相位 ack 前不转场 | **成立** ✅ | 测试：stage=8 路径断言 ack 前 `phase==Message`+非 ack 输入不放行+ack 后 `Exited`；本审临时探针（验后删）直验 trigger-(a)-alone 同款三断言全过 ⇒ 行为正确；唯测试锁在 (a)-alone 路径缺相位断言（mutation M4 存活，记 AR68-1） |
| ③新测试 | 3 条齐（stage=8 双触发/双触发幂等 0x20 非 0x40/边界哨兵 ==8 精确） | **成立** ✅ | `exit_hidden_gate_{stage8_triggers_with_message_ack, dual_trigger_sets_bit5_once, stage_boundary_is_exact_8}` 三条在 tests/shop_purchase.rs:314-371；dual 断言 `flags==0x20` 精确值（M3 双写 0x40 被咬）；边界锁 stage=7/9 不触发（M1 `==7` 被咬） |
| ③新测试 | 断言 RAM 终态非自证 | **成立** ✅ | 逐行：`flags==0x20/0x00`（$32）、`money==7` 不变（$018E）、`phase`/`prompt`（外部可观察状态机终态）、`fx==ShopFx::None`；零断言内部字段（`msg_back` 等路由枚举未入断言）；期望值全部来自 ROM 语义常量 |
| ④mutation | `st.stage == 8`→`== 7` 独立重放应红 | **成立** ✅（复现 2 FAILED） | M1：改→跑→**2 FAILED**（`stage8_triggers` `0 vs 32` + `boundary` stage=7 `32 vs 0`，exit=101）→ `git checkout` 还原→`git status` 干净；与审方上轮 2 FAILED 数量与命中断言一致 |
| ④mutation | 再抽 1 处（ack 相位跳过）验覆盖 | **成立** ✅ + 额外 2 项 | M2 ack 相位跳过→**2 FAILED**（两条 ack 前相位断言 `Exited vs Message`）；M3 双触发双写 `+=0x20`→**1 FAILED**（`64 vs 32` 幂等哨兵）；M4 非对称分支→**存活 0 FAILED**（见 AR68-1）。均改→跑→还原→tree 干净 |
| ⑤对拍 | cargo test --workspace = 125 | **125/125** ✅ 独立复跑（34 套件、0 failed） | §4 贴数 |
| ⑤对拍 | l2-diff = 109/109 | **109/109** ✅ 独立复跑 | §4 贴数 |
| ⑤对拍 | gates 12/12 + ratchet 125/125 | **PASS=12 FAIL=0 SKIP=0 TOLERATED=0**；`ratchet:rust_test_count 基线 125 / 现值 125 —— OK` ✅ | §4 贴数 |
| ⑥AD65-1 | R-3 指针偏注更正（`$9CEC`→`$9CF7`/`$9CF9`） | **成立** ✅ | spec:59-60 dated 更正（行为主张不变）+ ROM 直读 bank00:9CEC `a9 ab 85 12 a9 a2 85 13 20 d9 e6 a9 00 8d 00 04`：`LDA #0` 在 **$9CF7**、`STA $0400` 在 **$9CF9**（$9CEC 为文本 $A2AB→渲染→复位序列入口）——指针注与字节逐位相符；ui.rs/测试注同步 |
| ⑥AD65-2 | RED 工件留痕 | **成立** ✅ | `analysis/tdd-red-dev1-ar65-20261014.log` 在档：RED-1 E0599 编译面（ExitHidden 未定义 ×2）+ RED-2 行为面 `0x00 vs 0x20`/`Exited vs Message`（与本审重放 M2/M4 断言点同源）；测试头自述注；（注：`analysis/` 按 AGENTS §2 属可再工作区不入库——与建议选项一相符，关键输出行已转录 commit body 供复核） |
| ⑥AD65-3 | bit7 互注（R-3 指针偏注/bit7 双注互引） | **成立** ✅ | weapons.rs:76-78 ↔ ui.rs:291-296 双侧互注（"同一 RAM 位两处真读者，改语义须双向对账"）；ROM 直读 bank00:$9E0E `a5 32 09 80 85 32`（LDA $32/ORA #$80/STA $32）+ bank02:$922B dispatch 读者注；bit5「登记不解释」处理保持正确 |

## 1. AR65-1 现场重导（两触发源 + 分歧态 + spec）

### 1.1 ROM 直读（本审自跑，不采信被审复现脚本输出）

```
python3 - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read(); prg=rom[0x10:0x10+0x20000]
def show(bank,cpu,n):
    o=bank*0x2000+(cpu&0x1FFF); print(f"bank{bank:02d}:{cpu:04X}", prg[o:o+n].hex(' '))
show(1,0xA023,0x26); show(1,0xA0CF,5); show(0,0x9CEC,0x12); show(0,0x9E0C,10)
PY
```

| 片段 | 直读字节 | 本审手解 | 支持 |
|---|---|---|---|
| bank01:$A023-$A048 | `a5 18 29 33 c9 11 f0 07 ad 80 01 c9 08 d0 16 a9 33 20 3b ed a5 32 09 20 85 32 a9 5c 85 12 a9 a6 85 13 20 9f a0 a9 …` | `LDA $18/AND #$33/CMP #$11/BEQ $A032`（f0 07：$A02B+7=$A032）；`LDA $0180/CMP #$08/BNE $A048`（d0 16：$A032+0x16=$A048）⇒ **==8 落 $A032 置位块，不等于 8 才跳合流**；$A032 音效 → $A037 `ORA #$20/STA $32` → $A03D `LDA #$5C/STA $12/LDA #$A6/STA $13`（**text $A65C**）`/JSR $A09F` | 双触发 OR ✅ + 分歧态（stage=8,pad2=0 应置 bit5）✅ + $A65C ✅ |
| bank01:$A0CF | `29 c0 f0 f4 4c` | `AND #$C0`（A/B 任一）+ 回环 BEQ | ack 门 ✅ |
| bank00:$9CEC-$9CFB | `a9 ab 85 12 a9 a2 85 13 20 d9 e6 a9 00 8d 00 04` | 文本 $A2AB→JSR $E6D9 渲染→**$9CF7 `LDA #0`/ $9CF9 `STA $0400`** | AD65-1 更正精确 ✅ |
| bank00:$9E0E-$9E13 | `a5 32 09 80 85 32` | `LDA $32/ORA #$80/STA $32`（ORA 本体在 $9E10） | AD65-3 互注 ✅（指针粒度注见 AR68-2） |

### 1.2 三件套一致性对照

| 位置 | 主张 | 与 ROM |
|---|---|---|
| ui.rs:391-404 | 两触发源 OR + bit5 置位 + ExitHidden 消息 + ack 后合流 / 不触发直接合流 | ✅ 一致（else 分支 `phase=Exited` 对应 $A048 合流） |
| spec R-7（:79-81） | 同上 + dated 更正注（原漏 (b) 完整性问题 + ack 相位漏建模 + 改前分歧态） | ✅ 一致，更正注符合 §3.6 原地 dated 规范 |
| tests 3 条 | 触发态/幂等/边界 | ✅ 覆盖 OR 语义三象限（(b)-only、(a)+(b)、neither×2 边界），(a)-only 见 AR68-1 |

## 2. AR65-2 现场（ExitHidden + ack 相位档位）

- **第 7 变体**：`ShopPrompt::ExitHidden`（ui.rs:208-211）——`$A65C` 指针登记不解释、ack 门 `$A0CF` 注、
  等待语义 A 级依据（同 R-8 六类消息同路 `JSR $A09F`/`$A0CF AND #$C0`）、`ShopMsgReturn::Exit`
  （ack 后合流 bank01:$A048 转场交 flow）。**零发明文案** ✅。
- **实现档位**：完整 ack 相位（`ShopPurchasePhase::Message` 真相位，非 `WingBoughtFarCallPending` 式挂起枚举）；
  档位依据（等待语义已证、仅文案未证）在枚举注+call-site 注+commit body 三处可溯。
- **触发⇒消息 ack 前不转场**：实现层 `prompt_to_exit` 置 `phase=Message`，`step_message` 非 ack
  直接 `return`（ui.rs:566-569）——本审行为探针（trigger-(a)-alone：停 Message→非 ack 不放行→ack 转 Exited）
  三点全过，验后已删、tree 干净。**行为正确**；测试锁缺口记 AR68-1。

## 3. 新测试逐条 + mutation 实测（本审真改真跑真还原）

### 3.1 三条新测试（17 测试套件 = 14 既有 + 3 新）

| 测试 | 断言对象（逐行核对） | 判定 |
|---|---|---|
| `exit_hidden_gate_stage8_triggers_with_message_ack` | `flags==0x20`（$32 bit5 分歧态闭合）、`phase==Message`（ack 前不转场）、`prompt==ExitHidden`、`fx==None`、`money==7` 不变（$018E 零写）、BTN_UP 非 ack 不放行、BTN_B ack 后 `Exited`+`prompt==None` | 终态 ✅ 非自证 ✅ |
| `exit_hidden_gate_dual_trigger_sets_bit5_once` | `flags==0x20` **精确值**（0x20 非 0x40 幂等哨兵）、`phase==Message`、`prompt==ExitHidden`、BTN_A ack 后 `Exited` | 终态 ✅（M3 实测真咬） |
| `exit_hidden_gate_stage_boundary_is_exact_8` | stage∈{7,9} 循环：`flags==0x00`、`prompt==None`、`phase==Exited`（==8 精确非 ≥8） | 终态 ✅（M1 实测真咬） |

### 3.2 Mutation 实测表（4 项，全部改→跑→`git checkout` 还原→`git status` 干净）

| # | 变异 | 结果 | 命中的断言 | 备注 |
|---|---|---|---|---|
| M1 | `st.stage == 8` → `== 7` | **2 FAILED**（exit=101） | `stage8_triggers`（`0 vs 32`）、`boundary`（stage=7 `32 vs 0`） | **独立重放复现审方上轮 2 FAILED**，数量与断言点一致 |
| M2 | ack 相位跳过（触发后直转 `Exited`，即 AR65-2 修复前行为） | **2 FAILED** | `stage8_triggers`（`Exited vs Message`"ack 前不转场"）、`dual_trigger`（同） | 任务指定的第二处 mutation |
| M3 | 双触发双写：`flags \|= 0x20` → 每触发 `flags += 0x20` | **1 FAILED** | `dual_trigger`（`64 vs 32`，0x40 回绕哨兵） | 幂等断言真咬 |
| M4 | 非对称分支：(a) 直转 `Exited`、(b) 走消息相位 | **存活 0 FAILED**（17 全绿） | 无——(a)-alone 路径无相位断言 | **AR68-1**；行为探针证实现本身正确 |

## 4. 对拍独立复跑（三件 + ratchet，全部本审实跑贴数）

| 项 | 命令 | 本审实际输出 |
|---|---|---|
| 全量测试 | `cargo test --workspace` | **TOTAL PASSED: 125**（34 套件逐 `test result: ok.`，0 failed；shop_purchase 套件 `17 passed` = 14 既有 + 3 新） |
| L2 三向差分 | `cargo run --release -p game-oracle --bin l2-diff` | **L2-DIFF: 109/109 通过（golden==ROM 执行==game-core 同名函数，三者缺一不可）** |
| 总闸 | `python3 gates/cli.py check` | **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0**（SKIP/TOLERATED 显式打印为空） |
| ratchet | 同上 | **`ratchet:rust_test_count 基线 125 / 现值 125 —— OK`**（只升不降；d8ba360 122→125 留痕 reason 含 125/125+109/109+12/12 实测句，SHA256SUMS 链同步，baseline_history 单调） |

## 5. AD65-1/2/3 清净核

| 项 | 核查 | 判定 |
|---|---|---|
| AD65-1（R-3 指针偏注） | spec:59-60 dated 更正 `~~$9CEC~~`→`$9CF7`/`$9CF9`（行为主张不变）；ROM 直读 `9CF7: a9 00`/`9CF9: 8d 00 04` 精确；ui.rs ShopMsgReturn::MainRing 注+to_main_ring 注+测试断言消息三处同步 | ✅ 清净 |
| AD65-2（RED 工件留痕） | `analysis/tdd-red-dev1-ar65-20261014.log` 在档：RED-1 E0599（编译面 ×2）+ RED-2 行为面（`0x00 vs 0x20`/`Exited vs Message`，与 M2/M4 断言点同源）；tests 头自述注引用之 | ✅ 清净（`analysis/` 不入库属 AGENTS §2 设计，关键行已转录 commit body） |
| AD65-3（bit7 互注） | weapons.rs:76-78 ↔ ui.rs:291-296 双侧互注（kernel 门读者 bank02:$922B dispatch ↔ 翼已购置位 bank00:$9E0E），含"改语义须双向对账"警示；bit5 登记不解释保持 | ✅ 清净（指针粒度小注见 AR68-2） |

## 6. findings 路由（四分类）

| 编号 | 分类 | 级 | 位置 | 内容 |
|---|---|---|---|---|
| **AR68-1** | 测试覆盖（锁缺口） | **低·非阻断必补注**（行为无分歧） | `tests/shop_purchase.rs` `exit_entry_and_hidden_gate` 触发(a)-满足子场景（:293-300） | 该子场景只断言 `flags==0x20` 未断言相位 ⇒ trigger-(a)-alone 的「ack 前不转场」无测试锁：mutation M4（(a) 直转 Exited、(b) 走消息）**存活 17 全绿**。行为本身正确（本审探针直验 Message→非 ack 不放行→ack 转场三点全过，验后删）。修法一行：该子场景补 `assert_eq!(s.phase, ShopPurchasePhase::Message, …)` + `assert_eq!(s.prompt, Some(ShopPrompt::ExitHidden))`（或 ack 后 `Exited`），即可咬死 M4；与 review-65 §6-2「ack 前/后两断言」精神对齐（当时断言落在 stage=8 路径）。非阻断依据：六项判据全落地、行为 ROM 正确、review-67 AR67-1 同型先例（结论仍真⇒非阻断必补注） |
| AR68-2 | 文档（证据指针精度） | 低·可选 | weapons.rs:78 / ui.rs:293 新增互注 | 「bank00:$9E0E `ORA #$80`」——`ORA #$80`（`09 80`）本体在 **$9E10**，$9E0E 为 `LDA $32`（RMW 块 $9E0E-$9E13 首）。与 AD65-1 同类但仅差 1 条指令（review-65 表格自身用 `$9E0E-$9E1B` 区间形态故主张方向无误）。建议下轮触碰时改「bank00:$9E10 `ORA #$80`」或区间形态，不单独起修复轮 |

## 7. 复现命令

```
# ROM 直读（§1.1 脚本）
# mutation（§3.2，每项独立）：
sed -i 's/st.stage == 8/st.stage == 7/' crates/game-core/src/ui.rs   # M1
cargo test -p game-core --test shop_purchase                        # 看 FAILED 计数
git checkout -- crates/game-core/src/ui.rs && git status --short    # 还原+核干净
# 对拍三件（§4）：
cargo test --workspace
cargo run --release -p game-oracle --bin l2-diff
python3 gates/cli.py check
```

> 交叉核对：M1 红断言与 RED-2 留痕断言点同源（`shop_purchase.rs:316/321` 系列）；
> mutation 还原后复跑 shop_purchase 套件 17/17 绿、workspace 125/125 绿（本审两次复测）。
> ROM 全程只读未改（rom-integrity 门禁 PASS 佐证）。
