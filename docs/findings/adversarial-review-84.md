# 对抗复核 #84 — port-engineer 批次 1+2（wave.rs 记录机 + item_effect/member_schedule）

> 日期：2026-10-15｜角色：adversarial-reviewer（独立对抗复核员，证伪立场；实施方=port-engineer≠审核方）
> 被审：commit `2026f67`（crates/game-core/src/wave.rs + tests/wave_sched.rs + 基线 144→148）+
> commit `d7c27ca`（item_effect/member_schedule，批次 2）。
> 方法：不采信被审方自述、不复用其结论——全部关键数字**自写脚本直读 ROM 字节/trace 重导**；
> 被审方自带三个证伪靶（①tick→帧误用 ②$630 首现=数−1 突变绕过 ③未证 ctrl fail-closed）照单实测。
> 证据分级依 AGENTS.md §3。**拆段落盘防中断**（每段一 commit）。

## 段 A：43 记录 golden 三方对账 —— **全等** ✅

### A.0 方法（独立性声明）
自写 Python 解码器（不 import `game_core::wave`、`tools/wave_timeline`），数据面 = `rom/gunnac_cn.nes`
直读字节（iNES 头 0x10 + bank8×0x2000 8KB 窗 = CPU `$A000-$BFFF` 数据窗，偏移=addr−$A000；
该映射由"解出连贯 43 记录流"自证，见 A.3 注）。记录文法 `[delay@0][项…][ctrl]` 取自被审声明，
其 6502 listing 落点核（bank00:86CE handler 逐条）在**段 B**独立完成——两段交叉后文法才升 A 级。

### A.1 43 记录 (addr, 累计 tick) 三方对账

| 对账方 | 结果 |
|---|---|
| 我的 ROM 直读解码 | 43 条，`truncated=false`，自环 `@0xA397`，`loop_tick=1337` |
| `tests/wave_sched.rs` golden（43 行常量） | **43/43 地址+tick 逐条全等**（A26D=1 / A275=33 / A27C=49 / … / A390=1137 / A397=1237） |
| `enemy-timeline.md` §2.2 表 | **全等**：+97t(A299)/+113t(A2A0)/+129t(A2A7)/+137t(A2AC)/+145t(A2B3)/+161t(A2BA)/+177t(A2BF)/…/+1137t(A390)/自环 A397 delay 100t/圈（1237+100=1337 ✓） |

tick 口径自算复现：首记录 `tick=delay+1`（流头 delay=0x00 → tick=1）；其后 1:1 累计（rec1 delay=0x20=32 → 33）。
**与"sub_86A0 `$51:=(@0)+1` quirk + 86BD 1:1 装载"声明一致**（listing 落点核见段 B）。

### A.2 项解码抽查（6 处，逐字节）

| 记录 | 我的解码 | 测试 golden / §2.2 | 裁定 |
|---|---|---|---|
| @A26D 首记录 | `reg(13,1C,02) reg(14,04,67)` | 同 | ✅ |
| @A275 | `spawn ctrl2 T=$2F (2B,06,0A,20)`，delay=0x20 | 同（f1341 组长签名记录） | ✅ |
| @A2A7 | `settype T=$2A p=00` | 同（records[8]） | ✅ |
| @A2E1 | `pair (44,00)`（全流唯一 type≥$10 对） | 同（records[17]） | ✅ |
| @A339 | `spawn ctrl1 T=$2E (4D,04,28,0E)` + `spawn ctrl1 T=$2E (4D,04,28,8E)` 双项 | 同（records[30]） | ✅ |
| @A397 | `jmp ctrl9 target=$A397`（自址=自环） | 同（records[42]） | ✅ |

### A.3 trace 帧锚（部分独立复核）
- **f1341-1391 簇**：从 `tas-stg01-bot1`/repro 轨独立重导（自算脚本读 frames 的 `$064B`=`$630+27`）——
  f1341 置 5 后 **f1351/1361/1371/1381/1391 每 10f 递减至 0**，恰为 rec@$A275（组员数 6、间隔 10f）的
  "剩员计数 6−1=5 首现 + 10f 节律" ✓ 与 §2.2 签名注（`$630=$05(=6−1)`）及 §2.3「f1341-1391（+10f×6）」全等。
- 其余锚（f1469/1597/1725/1789/2109/2365）为槽 25/23 组长（bot 轨仅 watch slot27）——独立复核走
  **段 C**（iram `$0400+n` 0→非0 判定式自算 120 事件面，数据源 `.archive/gameplay-stage1-*/frames.jsonl` 已在盘）。
- 锚间隔自洽：f1469−f1341=f1597−f1469=f1725−f1597=128f=16t×8 ✓；f2109−f1789=320f=40t×8 ✓；
  f2365−f2109=256f=32t×8 ✓（与 §2.2 锚点校验注一致；注意该间隔式与 tick_to_frame 公式同源，**单靠间隔不构成
  映射验证**——绝对帧锚定留段 C）。

### A.4 advisory（不阻断）
1. `tests/wave_sched.rs` `bank8_window()` 注释「bank8 = **$8000** 窗」与实际口径不符：偏移=addr−**$A000**
   （wave.rs `WINDOW_BASE=0xA000`），解码仅在 $A000 基下连贯——注释应改 `$A000` 数据窗（R7 映射）。
2. §2.2 摘要行「[10]$2A ×4、[1]$2E ×10、[2]$2F ×6」为**节选概数**（实际全流 [10]×4、ctrl1 spawn 项 21 个、
   ctrl2 spawn 项 12 个）；该行语义是"摘要排至 +1137t"非精确普查，不构成失实，建议标注"概数"。

### A.5 本段结论
**43 记录 golden 三方全等成立**（含自环与 +1 quirk 累计口径）；记录文法的 listing 落点与其余锚定
分别在段 B/C 收口。段 A 单独不足以判 pass——verdict 于段 C 给出。

---

## 段 B：6502 落点核 + 两时序域红线 + 三证伪靶 mutation 实测

### B.1 记录机语义 listing 落点核（seed 868B-87F1 逐条）——**全部吻合** ✅

| 被审声明 | listing 落点 | 裁定 |
|---|---|---|
| 首记录 +1 quirk（`$51:=(@0)+1`） | `86A4 LDA ($4F),Y / 86A6 CLC / 86A7 ADC #1 / 86A9 STA $51` | ✅ |
| 后续 delay 1:1（无移位无 +1） | `86BD LDA ($14),Y / 86BF STA $51` | ✅ |
| pair/ctrl 分界 = type≥$10 | `86D2 CMP #$10 / 86D4 BCC loc_86FC`（< $10 走 ctrl 跳表） | ✅（哨兵⑤分界=硬件真值） |
| pair 双写 `$400+槽:=type、$470+槽:=param` | `86E6 STA $400,X / 86EB STA $470,X`（86E4 `LDA $13`=type） | ✅ |
| ctrl0=记录分隔 | `86FD ASL A / 86FE BEQ locret_86CD`（0→RTS，下字节=新 delay） | ✅ |
| ctrl3..9 只改流指针 | `8795-87A3`：2B→`$14/$15`，无槽写 | ✅（ItemEffect::NoWrite ✓） |
| ctrl1/2/11/12→`$2E/$2F/$2C/$2D` | `8738/8776/872F/877F LDA #imm`（跳表 870E ctrl1..14 全命中） | ✅ |
| 四参写链 args→`$518/$630/$534/$470` | `874C-8760`：`STA $518,X / $630,X / $534,X / $470,X` 顺序装载 | ✅ |
| step 判据=**type&1**（非 ctrl 判据） | `8763 LDA $13 / 8766 AND #1 / 8768 BEQ`（$13=8743/8786 写入的 leader_type） | ✅（但测试面盲区见 B.4-M4） |
| ctrl2/12 附加码流读 `($3E),0→$48C+槽` | `8783-878E`：`STA $400,Y / STA $13 / LDA ($3E),Y / STA $48C,X`；ctrl1/11 BCC→8743 不走此段 | ✅ |
| ctrl10 双写+**无条件** step | `87A4-87CA`：`$400,X/$470,X/$48C,X` 三写 + `87C4 INC $3E`（无 AND#1 测试） | ✅ |
| ctrl13/14 透传 `$E63D(Y=0,1/4,5)` | `87CD-87E2`（Y=0,1）+ `87EA-87F1`（Y=4 起）JSR $E63D | ✅ |
| 未证 ctrl fail-closed | 跳表仅 14 项（ctrl1..14）；ctrl≥15 越表=硬件未定义 ⇒ 软件 fail-closed 正确 | ✅ |

**段 A 的记录文法由此升 A 级**（字节文法与 handler 语义互证）。

### B.2 两时序域红线实现落点核
- **tick 域**：`SchedRecord.delay` 文档="上一记录触发至本记录触发的 gap tick"；累计=86A0+1 quirk/86BD 1:1（B.1 ✓）；
- **帧域**：`member_schedule`/`GroupRelease.interval_frames` 直接以帧兑现、无 ×8/÷8 换算（实测 M1 突变必红）；
  文档 5 处显式红线措辞（模块 doc「禁止换算常数跨关复用」、`stg01_tick_to_frame`「⚠ 禁止跨关复用」、
  `member_schedule`「勿乘 tick 换算」等）；
- **隔离性**：`stg01_tick_to_frame` 全仓调用点 = 0（grep `--include=*.rs` 除 wave 自身）⇒ 当前树**无误用面**；
  `$534` 不经该函数（fn doc 显式标注）✓；
- Wt-1 换算式（tick≤17→1193；18+k→1221+8k）与 7 锚的内部自洽已核（段 A.A.3）；绝对帧锚定留段 C。

### B.3 三证伪靶 mutation 实测（改错→红→还原，正本 sha 复验）

| # | 靶 | 突变 | 预期 | 实测 |
|---|---|---|---|---|
| M1 | ①tick→帧误用 | `interval_frames` 消费处 ×8（member_schedule 与 GroupRelease 双点） | 节律测红 | ✅ **2 测红**（⑦golden+⑧哨兵①"15 帧整非 120"） |
| M2 | ②$630 首现 | `first_seen = count - 1` → `count` | 首现测红 | ✅ **1 测红**（⑧ `assert_eq!(first,5)`+`assert_ne!(first,6)`） |
| M3 | ③未证 fail-closed | `Unproven` 分支→伪出参 `PairWrite` | 出参测红 | ✅ **1 测红**（⑥ 末断言） |
| M5 | （我加）pair/ctrl 分界 | `v>=0x10` → `v>=0x20` | 哨兵⑤红 | ✅ **1 测红** |
| M4 | （我加）step 判据盲区探测 | `type_&1` → `ctrl==2\|\|ctrl==12` | —— | ⚠️ **9 测全绿**（见 B.4） |

全部突变已 `git checkout` 还原，`cargo test --test wave_sched` **9/9 复绿**、`git status crates/game-core/src/wave.rs` 干净。

### B.4 发现 F-B4（哨兵检测面盲区，必修小项）
**"step 判据=type&1（非 ctrl 判据）"无测试可咬**：解码器保证 type↔ctrl 恒映射（1→$2E/2→$2F/11→$2C/12→$2D），
type&1 与 ctrl∈{2,12} 在全部可达输入上**等价** ⇒ M4 突变全绿。listing 事实（8763 `LDA $13`/`AND #1`）不受影响，
但"负向哨兵锁死口径"的覆盖面声明**高估一格**。
**修正建议**：哨兵④/⑤旁增合成用例 `item_effect(&Spawn{ctrl:1, type_:0x2F, ..})` 断言 `code_stream_step==true`
（type 奇=step，与 ctrl 无关），使该判据可被 mutation 咬合。

### B.5 fail-closed 全路径试调（靶③补充）
`item_effect` 六分支试调全覆盖：PairWrite ✓（⑥）/QuadWrite ctrl1,2,11,12 ✓（⑤）/SetTypeWrite ✓（⑥）/
RegWrite ctrl13 ✓（⑥；ctrl14 同臂未单列——琐碎，建议补一行）/NoWrite(jump) ✓（⑥）/Unproven ✓（⑥+M3 咬合）；
decode 侧：ctrl0 分隔 ✓（哨兵⑤第 2 条）/ctrl10,13,14,pair,jmp 实流 ✓（A2A7/A26D/A2E1/A397 golden）/
未知 ctrl ✓（哨兵④+M3 语义）；`count==0`/`interval==0` → `None` ✓（⑧）。**结论：fail-closed 真闭合**（除 B.4 盲区）。

### B.6 工作区并发登记（诚实）
复核期间 `crates/game-core/src/shot.rs`（+163 行）与 `tests/stage_content.rs`（+111 行）存在**他人未提交改动**
（port-engineer 批次 3 在途）——本段全部测试均在 wave 面（`--test wave_sched`），不受影响；
段 C 的 cargo 全量复跑将注明树态。
