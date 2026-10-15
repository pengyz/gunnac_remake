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
