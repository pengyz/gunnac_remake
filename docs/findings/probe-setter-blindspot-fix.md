# probe 写点盲区修复 + 单场景重采验证（G1 补齐，族①事件序回放第一块砖）

> 2026-10-20 | 实施 agent（tools 取证面）。背景=[l5-0160-batch-semantics](l5-0160-batch-semantics.md)
> §1/§6 G1：`$0160` 的 ROM 写点在 probe 日志中**零事件**（非变址存储全盲）⇒ setter/消费链无法重建。
> 本文=诊断（为何盲）/ 修复 diff / 重采统计 / 回归 sha / 缺口清单。
> 证据等级标注：A=硬证据（Mesen2 实测探针产物 / trace 逐计数），B=静态推理。**区分"已证实/假设"**。
> 只改 `tools/scenario_capture.lua`（捕获面）；未动 DSL/oracle 场景/crates/gates/expected；未 git 提交。

## 0. 判定摘要

1. **根因（A）**：`tools/scenario_capture.lua` 的 `emu.addMemoryCallback(fn, "write", lo, hi)` 传的是
   **字符串** `"write"`，Mesen2 2.1.1 把它**静默转成整数 0 = `emu.callbackType.read`** ⇒ 注册的从来是
   **读回调**；兜底分支查的 `emu.memCallbackType` 在 2.1.1 **不存在**（实测 nil），恒落回同一个字符串。
   旧 probe.jsonl 是**纯读流**（含存储指令的假读），不是写流。
2. **"变址写可见"是假象（A）**：`STA abs,X/abs,Y/zp,X` 有**假读（dummy read）周期**，读回调抓到的是假读；
   `STA/STX abs、zp` 无读周期 ⇒ 全盲。RMW"只出旧值"同理=读相位。6502 周期表逐条对上（§1.3）。
3. **修复（A，验证通过）**：按实测枚举 `emu.callbackType={read=0,write=1,exec=2}` **双流注册**（写流新增、
   读流保留），事件行加 `"op":"w"/"r"`。`$0160` 写事件 0 → gameplay-stage1 **179 条/168 帧**、
   gs5-anchor-render **622 条/608 帧**；seed 15 个 `$0160` 写点中 **9 族直写可见**（EECE/EEE1/88A3/
   900A/904B/ECD0/ED68/827A/F14B），未触发 6 族=场景覆盖（§6）。
4. **置位:消费 1:1 成立（A）**：gameplay-stage1 **置位写 v=3 = 70 = NMI 见挂起 70 = F14B 清零 70**；
   gs5 双类型 **v=3 置位 94 + v=1 置位 197 = NMI 见 94/197 = 清零 291**，严格 1:1（§5.2）。
5. **附带闭合 l5 G2（A）**：会话外 30 次 type-3 置位的 setter 路**直接定位** = `904B×23 + 900A×4 +
   EEE1×2 + ED68×1`（旧 B 级候选 EEE1 只占 2/30）。
6. **附带证伪两条 gotcha + l5「RMW」标签（A）**：`$2005/$2006/$2007` **写事件现在可见**
   （4,898/25,686/47,474 条）⇒「addMemoryCallback 不覆盖 I/O 寄存器区」「$2006 锁存不触发」=同一根因
   误诊；`88A0 0D 60 01` 是 ORA **纯读**非 RMW（§5.3）。待更正清单见 §6。
7. **回归核通过（A）**：重采 `gameplay-stage1.frames.jsonl` sha256 与现存同场景 **逐字节全同**（§4）——
   捕获面改动不改行为。
8. Mesen2 2.1.1 Lua **无 addBreakPoint API**（`emu.addBreakPoint=nil`，A）⇒ l5 §6 G1 建议的
   "addBreakPoint 写断点"绕行法**不可用**；callbackType 双流注册是唯一可用补法。

## 1. 诊断：为什么非变址 STA 全盲

### 1.1 实测探针（A 级，可复现）

`analysis/tmp/probe_cb_diag.lua`（scratch，只读观察）在 Mesen2 2.1.1 headless 下对 `$0000-$07FF`
同域注册 8 个带标签回调（字符串 "write"/"read"、枚举 `emu.callbackType.write/.read`、数值 0/1/2/3），
跑 30 帧，产物 `analysis/tmp/probe_cb_diag.jsonl`。API 面实测：

| 探查项 | 实测结果 |
|---|---|
| `emu.callbackType` | **`{read=0, write=1, exec=2}`** |
| `emu.memCallbackType` | **nil（不存在）** —— scenario_capture 旧兜底是死代码 |
| `emu.addBreakPoint` / `emu.addBreakpoint` | **nil（不存在）** |
| 注册 `"write"`（字符串） | ok=true，但**事件流 ≡ read 流**（见下） |
| 注册 `"read"`（字符串） | ok=true，读流 |
| 注册 `emu.callbackType.write`（=1） | ok=true，**真写流** |
| 注册数值 3 | error `invalid callback type` |

boot 清零块（seed L87329-87338）判别锚逐条对账（帧 5，256 次循环）：

| 后取指 pc | 指令（bank15） | 读流（"write"/"read"/0/枚举read） | 写流（枚举write/1） |
|---|---|---|---|
| `E025` | `E022 CMP $7FE`（读） | 1 | 0 |
| `E036` | `E033 STA $7FE`（abs 直写） | **0** | 1 |
| `E039` | `E036 STY $7FF`（abs 直写） | **0** | 1 |
| `E03E` | `E03C STA 0,X`（zp,X） | 256，**恒 a=$0000**（基址假读） | 256，a=$00+X（真写） |
| `E041` | `E03E STA $100,X` | 256（假读，无跨页=同址） | 256（写） |
| `E048` | `E046 STA $E9`（零页直写） | **0** | 1，v=8（=前置 `LDA #8`） |
| `E04D` | `E04A STA $180`（abs 直写） | **0** | 1，v=1（=前置 `LDA #1`） |

四个标签流的事件计数逐值相同（41,083 = 41,083 = 41,083 = 41,083；写流 6,025）⇒
**字符串 "write" 与 read/0/枚举read 是同一条流**，实锤。

### 1.2 旧日志"可见/不可见"模式的完整解释（A/B）

| 旧日志现象 | 真实机理 | 级 |
|---|---|---|
| `STA/STX abs、zp`（EECE/EEE1/EC42/88A3/F14B/F155/STA $2007…）零事件 | abs/zp 存储**无读周期**，读回调永不触发 | A（实测） |
| 变址存储"可见"（E8E3 `STA $103,X`、E041、EED9） | `STA abs,X/abs,Y/zp,X` 有**假读周期**（跨页时假读还落在**未进位**地址：`STA $161,Y` Y=$FF 假读恰落 $0160 —— l5 §1"回绕命中"一案） | A（实测）/B（6502 周期表） |
| RMW"只出 1 条、值=修改前旧值"（88A0 `ORA $160`） | RMW **读相位**；写相位（新值）在写回调才可见 | A（实测）/B |
| 读事件大量存在（EED4 自旋 149,309、F0DF 2,400） | 本来就是读回调，正常 | A |
| `E041 v=0 ×1` 被 l5 §2 记为"唯一写事件" | 实为 `STA $100,X` X=$60 时的**假读**（本次重采读/写流各 1 条同址成对，op 消歧） | A |

### 1.3 pc 语义不变

pc 仍是**访存指令的后取指 PC**（本次判别锚 E048=E046+2、E04D=E04A+3 复核成立）。写点按
"指令地址 + 指令长度"映射到 probe pc（如 `EECE STA $160` → `EED1`，与自旋读 `EED1 LDA $160` → `EED4`
不冲突；`88A0 ORA $160` RMW 读/写两相位同出 pc `88A3`；`BBBF STA $160` → `BBC2`）。

## 2. 修复（tools/scenario_capture.lua，仅捕获面）

`PLAN.probe_mem` 注册段（:145 起）单点改动，**不改输入/行为/DSL**：

```diff
-    local ok = pcall(emu.addMemoryCallback, function(address, value)
-      probeLog(address, value)
-    end, "write", lo, hi)
-    if not ok then
-      ok = pcall(emu.addMemoryCallback, function(address, value)
-        probeLog(address, value)
-      end, emu.memCallbackType and emu.memCallbackType.write or "write", lo, hi)
-    end
-    probeF:write(string.format('{"probe":"%04X-%04X","cb":%s}\n', lo, hi, tostring(ok)))
+  local CB = emu.callbackType or {}
+  local WTYPE = CB.write or 1   -- Mesen2 2.1.1 实测 {read=0,write=1,exec=2}
+  local RTYPE = CB.read or 0
+  for _, r in ipairs(PLAN.probe_mem) do
+    local lo, hi = tonumber(r[1], 16), tonumber(r[2], 16)
+    for _, s in ipairs({{"w", WTYPE}, {"r", RTYPE}}) do
+      local ok = pcall(emu.addMemoryCallback, function(address, value)
+        probeLog(address, value, op)          -- probeLog 增加 op 参数
+      end, ctype, lo, hi)                     -- 数值/枚举类型，**禁字符串**（"write"=read 的坑）
+      probeF:write(string.format('{"probe":"%04X-%04X","op":"%s","ctype":%d,"cb":%s}\n', ...))
+    end
+  end
```

**probe.jsonl 格式变更**（analysis/ 可再生工作区，无下游 golden 依赖）：
- 声明头：`{"probe":"lo-hi","op":"w"|"r","ctype":0|1,"cb":true|false}`（每区间 2 条，30 条/本场景）
- 事件行：原字段 `f/pc/a/v/x/y/ar` 不变，**新增 `"op":"w"|"r"`**（RMW/变址同址双事件靠 op 消歧）。
- 既有按 key 解析的 scratch 脚本（analysis/tmp/probe_*.py）不受影响；旧日志=读流这一事实见 §1.2。

## 3. 重采统计（gameplay-stage1，2400f，Windows GUI 模式，2026-10-20 02:27）

命令：`python tools/scenario_run.py run oracle/scenarios/gameplay-stage1.txt --timeout 1200`
（任务书 `--scene` 写法与现行 CLI 不符，实际 CLI=`run <场景>`；未改 scenario_run.py）。
DONE 干净：`frames=2400 trace_lines=2400 captures=44 frame_errors=0`，wall 217.3s；
旧产物已由驱动归档至 `analysis/traces/.archive/gameplay-stage1-20261020-022703/`。
事件总量：**op=r 1,309,013 + op=w 638,882**（读流 1,309,013 与旧日志事件行数**完全相同** ⇒
旧=读流实锤 + 双流注册未扰动读侧）。

### 3.1 `$0160` 事件分布（179 写事件 / 168 帧，旧=0 写事件）

| op | pc | 归属指令 | v | 条数 | 旧日志 |
|---|---|---|---|---|---|
| r | `EED4` | `EED1 LDA $160`（EECC 自旋轮询） | 3 | 149,309 | 有（同计数） |
| r | `F0DF` | `F0DC LDA $160`（NMI 轮询） | 0 | 2,330 | 有（同计数） |
| r | `F0DF` | 同上，见挂起 | 3 | 70 | 有（同计数） |
| **w** | `F14E` | `F14B STX $160`（NMI 消费清零） | 0 | **70** | **0（盲）** |
| **w** | `EED1` | `EECE STA $160`（自旋置位） | 3 | **40** | **0（盲）** |
| **w** | `900D` | `900A STA $160` | 0/3 | 24/4 | **0（盲）** |
| **w** | `904E` | `904B STA $160` | 3/0 | 23/11 | **0（盲）** |
| **w** | `EEE4` | `EEE1 STA $160`（脉冲置位候选） | 3 | **2** | **0（盲）** |
| **w** | `ECD3` | `ECD0 STA $160` | 0 | 2 | **0（盲）** |
| **w** | `ED6B` | `ED68 STA $160` | 3 | 1 | **0（盲）** |
| **w** | `827D` | `827A STA $160` | 0 | 1 | **0（盲）** |
| r/w | `E041` | `E03E STA $100,X`（boot 清零，X=$60 命中 $0160） | 0 | 1/1 | 1（假读，曾误记为写） |

### 3.2 置位 : 消费 1:1 对账（A）

- 置位写 v=3：**70**（EED1 40 + 904E 23 + 900D 4 + EEE4 2 + ED6B 1）；
- NMI 轮询见挂起（`F0DF` 读 v=3）：**70**；NMI 消费清零写（`F14E`）：**70**
  ⇒ **70 : 70 : 70 严格 1:1**（一次置位 = 一次 NMI 消费 = 一次清零），l5 §0-3「读侧 1:1」现补上**写侧直证**。
- 自旋路：`EED1` 置位 40 = 自旋会话退出（`EED4` v=0）40 = l5 §2 的 64 会话启发式中 40 次退出观测，自洽。
- **G2 闭合（A）**：会话外 30 次 type-3 置位的 setter 路 = `904B×23 + 900A×4 + EEE1×2 + ED68×1`
  （30 = l5 §2 会话外消费数，逐条对上）；旧 B 级候选 EEE1 仅 2/30，**主路是 904B/900A**。
- v=1（BBBF 族）本场景 0 条 —— 与 l5 §3「gameplay-stage1 全程 v=1=0」一致（场景不覆盖，见 §5）。

### 3.3 附带：I/O 寄存器写点解盲（A，冲击 addr-phase-w33 / ppu2006 gotcha）

| 地址 | op=w（新） | pc 样例 | op=r |
|---|---|---|---|
| `$2005` | **4,898** | `EFCB/EFD0`（scroll 写）、`AB4C/AB4F` | 0 |
| `$2006` | **25,686** | `EFBE/EFC1`、`F097/F09A`、`F1F1/F1F8` | 0 |
| `$2007` | **47,474** | `F1FF`（17,924）、`F0A0-F0B8`（NMI 上传写体） | 1,048（F17E/F181/F18C/F18F 读，同旧计数） |

⇒ `docs/findings/addr-phase-w33.md`「addMemoryCallback 不覆盖 I/O 寄存器区（$2000-$401F）」与
`docs/knowledge/gotcha-mesen2-ppu2006-callback-quirk.md`「$2006 锁存寄存器不触发」**同为本 bug 误诊**：
写回调注册成读回调后，任何**无读周期**的写（含 `STA $2007`）都不可见。**本次只登记，不改他人文档**（G7）。
这直接冲击 `docs/findings/l4-zero-tolerance-attack-plan.md` §3.1「差什么①」的上传写流取证前提：
`$2006/$2007` 写流（PPU 地址锁存 + VRAM 上传字节）现可逐拍重建。

## 4. 回归核（捕获面改动不改行为，A）

| 产物 | 重采前 sha256 | 重采后 sha256 | 判定 |
|---|---|---|---|
| `analysis/traces/gameplay-stage1.frames.jsonl` | `0fdd22821701b84a4778b3130b4d35e8f48a5336b6b268522a307a46866afded` | `0fdd2282…66afded`（53,026,252 B） | **全同** |
| `analysis/traces/gameplay-stage1.captures.jsonl` | `da0d3d1f8ace6fb96bef91ddcd0f326df3c0529ef87c0c5e10f62dda7dcc4040` | `da0d3d1f…7dcc4040`（8,107 B） | **全同** |
| `analysis/traces/gameplay-stage1.probe.jsonl` | `a840f5703cfd5695a3e314fe5b223b9d751f23e262e30cd9e61966ba16b105b0`（82,524,670 B，读流） | `7fd1643cfaaf4ce6f14a343655f822b6d8a788ac11e92a313faae55ed3e31851`（140,952,864 B，双流） | 预期变更（格式） |

前采样归档核对：旧 frames.jsonl 已由 `archive_existing` 整体移入
`analysis/traces/.archive/gameplay-stage1-20261020-022703/`（未丢弃，可再核）。
读流与旧日志逐计数一致（§3）+ trace/captures sha 全同 ⇒ **无行为污染**。

## 5. 补充重采（gs5-anchor-render，5000f）：88A3 写点族 + v=1 族（已闭合）

> gameplay-stage1 不触发 `88A3`/`BBBF` 路——**场景覆盖**而非捕获缺口（l5 §3：两族在
> stg03/stg08/gs5/tas-stg08b2 轨道出现）。补采 `gs5-anchor-render`（stage-1 同型 5,000f，
> probe 区间含 `0130-0160`/`2005-2007`）。命令同 §7；DONE 干净（frames=5000 trace_lines=5000
> captures=28 frame_errors=0，wall 113.7s）；旧产物归档
> `analysis/traces/.archive/gs5-anchor-render-20261020-023303/`。事件量 op=r 388,116 + op=w 160,079。

### 5.1 `$0160` 事件分布（622 写事件 / 608 帧）

| op | pc | 归属指令 | v | 条数 | 备注 |
|---|---|---|---|---|---|
| r | `EED4` | `EED1 LDA $160` 自旋 | 3/0 | 234,182 / 63 | 旧=同计数 |
| r | `F0DF` | `F0DC LDA $160` NMI 轮询 | 0/1/3 | 4,709 / **197** / 94 | 旧=同计数 |
| **w** | `F14E` | `F14B STX $160` 消费清零 | 0 | **291** | =94+197 |
| **w** | `88A6` | **`88A3 STA $160`（v=1 置位）** | 1 | **197** | **88A3 写点族 ✓** |
| r | `88A3` | `88A0 ORA $160` | 0 | 197 | 旧日志"RMW 旧值"案，实为纯读 |
| **w** | `EED1` | `EECE STA $160` | 3 | 63 | 自旋置位=63 会话退出 |
| **w** | `904E`/`900D`/`EEE4`/`ECD3`/`ED6B`/`827D` | 904B/900A/EEE1/ECD0/ED68/827A | 3/0 | 34/28/3/2/1/2 | 同 gameplay-stage1 族 |
| r/w | `E041` | `E03E STA $100,X` boot | 0 | 1/1 | 假读+写成对 |

### 5.2 双类型 1:1 对账（A）

- **type-3（v=3）置位 94** = NMI 见 v=3 94（EED1 63 + 904E 23 + 900D 4 + EEE4 3 + ED6B 1）；
- **type-1（v=1）置位 197** = NMI 见 v=1 197（`88A6` 写 197）；
- **NMI 消费清零 `F14E` 291 = 94 + 197**，NMI 轮询总数 5,000（1/帧）⇒
  **两种置位各自与消费严格 1:1，清零数=两类之和**，l5 §0-3 的单槽挂起位域 + NMI 单点消费模型获写侧直证。

### 5.3 【2026-10-20 更正】l5「RMW」标签不成立

l5 §1/§6 G6 把 `bank00:88A0 0D 60 01 ORA $160` 记作 RMW（"读相位旧值"）——**误标**：
6502 `0D`=ORA absolute 是**纯读指令**（读组 xD，非改写组 xE/x6/CE/EE），无写回周期。旧日志
"RMW 只出 1 条、值=修改前旧值"就是一条普通读事件；真正置位是紧随的 `88A3 STA $160`（写 v=A|旧值，
本次 `op=w pc=88A6 v=1 ×197` 直证）。真 RMW（INC/DEC/ASL/LSR abs 等）的读/写相位事件值语义
仍未取证——**G6 更名保留**（见 §6）。另：读流事件值恒=读到的内存值、写流事件值恒=存入值
（E048 v=8/E04D v=1 判别锚 + 88A6 v=1 复核，A）。

### 5.4 回归核（第二场景亦全同，A）

| 产物 | 重采前 sha256 | 重采后 | 判定 |
|---|---|---|---|
| `gs5-anchor-render.frames.jsonl` | `b628ad3ef6be16813c630184c561dfbf485e66d5d2d4ffe8f60c96debffc73bd` | 同（15,000,838 B） | **全同** |
| `gs5-anchor-render.captures.jsonl` | `5b9704ab30ef1285b3814db153fa0bc930d5d6927fef498443781addd0b03814` | 同（5,562 B） | **全同** |
| `gs5-anchor-render.probe.jsonl` | `7db8a474628fd221ca62a225ebf36f6bb931055d23d5bcd042a712cb5e3f2995`（23,606,517 B 读流） | `935678c325c9247e66f6f7c5230ba6508fdb97ecec0c04b44aa82095005dbc92`（38,450,665 B 双流） | 预期变更 |

**读流与旧日志逐计数全同**（$0160 全部 pc/v 桶：88A3 197/E041 1/EED4 234,182+63/F0DF 4,709+197+94，
零差异）⇒ 旧日志=读流的判定在第二场景复核成立，且双流注册对读侧零扰动。

## 6. 缺口清单

- **G1 关闭（RAM + I/O）**：非变址存储写点盲区已修（§3.1/§3.3）；seed 15 个 `$0160` 写点中
  **9 族已在两场景直写可见**（827D/88A6/900D/904E/ECD3/ED6B/EED1/EEE4/F14E）。
- **G1'（残留，场景覆盖）**：`82BC`（82B9 STX）、`8542`/`85C9`/`87EE`（bank5）、`EC45`（bank15）、
  **`BBC2`（bank3:BBBF STA，v=1 族）** 两场景零触发。机制已证（同 opcode 8D/8E 族多点可见）；
  `BBBF` 需 tas-stg08b2 轨道级重采（40,000f，probe 双流后约 2.5M 事件，逼近上限）才能直证——
  建议排期为独立卡，不在本卡短场景口径内。
- **G6 更名保留**：旧"RMW 事件值语义"基于 `88A0 ORA $160` 单例——该例是纯读（§5.3 误标更正）；
  **真 RMW（INC/DEC/ASL/LSR abs）读/写相位事件值语义仍无第二例取证**（本次机制上写相位必现
  `op=w`、读相位 `op=r`，但缺实测样例）。
- **probe 上限未动**：`probeN > 3,000,000` 静默丢弃仍在（双流后更易触顶：gameplay-stage1 1.95M、
  5,000f 级场景可能截断）——属捕获面既有约束，本次最小改动未扩；长场景重采前需评估。
- **重复注册未动**：PLAN.probe_mem 重叠/重复区间（如 `002F-0030`×2、`2006-2007`×2）会产生重复事件
  （既有行为，`adversarial-review-l4-baseline-freeze-plan.md` 已登记），本修复不放大语义只放大计数。
- **G7 文档更正待办**（本文只登记，改他人文档须单独走）：`gotcha-mesen2-ppu2006-callback-quirk.md`、
  `addr-phase-w33.md`（§3.3 证伪）；`l5-0160-batch-semantics.md` §1/§6 G6 的「RMW」标签（§5.3 更正）；
  `adversarial-review-101.md`/`text-line-flag-survey.md:45`/`l4-signing-criteria.md:111`（l5 §1 连带）；
  `docs/methodology.md:95`、`docs/ai-decompile-pipeline.md:88` 的 `addMemoryCallback` 条目建议补
  "callbackType 数值枚举（read=0/write=1/exec=2），禁字符串参数"。
- **假设区（B/C，不入结论）**：`904B/900A/ED68/827A/ECD0` 各自的语义分型（何种子系统置位）
  未考证——属族①事件序回放后续语义化工作，本卡只补可见性。

## 7. 复现命令

```bash
# 诊断（可重跑，~1s 模拟 + 30 帧）
tools/mesen2/Mesen.exe --testRunner --timeout=60 analysis/tmp/probe_cb_diag.lua rom/gunnac_cn.nes /DoNotSaveSettings
python -c "import json;[print(l.strip()) for l in open('analysis/tmp/probe_cb_diag.jsonl') if '\"api\"' in l or '\"reg\"' in l or '\"done\"' in l]"

# 重采（Windows GUI 模式，串行；Mesen 单实例纪律）
python tools/scenario_run.py run oracle/scenarios/gameplay-stage1.txt --timeout 1200
python tools/scenario_run.py run oracle/scenarios/gs5-anchor-render.txt --timeout 1800

# 统计
python analysis/tmp/probe_fix_stats.py analysis/traces/gameplay-stage1.probe.jsonl

# 回归核（对 .archive 内旧产物再算 sha 复核）
python -c "import hashlib;print(hashlib.sha256(open('analysis/traces/gameplay-stage1.frames.jsonl','rb').read()).hexdigest())"
```

settings 纪律：全程未改 `%USERPROFILE%\Documents\Mesen2\settings.json`（起止 sha 均
`e46d001c9564eb0d2e50ec60a584a159884767f46b960627f289cf2881dec602`，EmulationSpeed=100、
ScriptTimeout=1、AllowIoOsAccess=true），符合"用完恢复 speed=100/timeout=1"。


## 【2026-10-20 02:45 归属与处置注】
- 实施=d82b1496（诊断+补丁+重采，收尾 hygiene 步崩）；独立复核=c4087e1a（根因 A 级定位：
  addMemoryCallback 字符串 "write" 被 Mesen2 2.1.1 静默取整为 0=read 回调 ⇒ 全程只抓读；
  变址存储可见=假读 dummy read；数值枚举双流=唯一补法——**Mesen2 2.1.1 无 addBreakPoint API**，
  原计划补法不可行已更正）；计数复算 70:70:70 严格 1:1、事件量 w=638,882/r=1,309,013 交叉吻合。
- 待办 nit（后续小卡）：§1.1 窗口计数 41,083/6,025 与 done 行 140,918/7,879 口径对齐；§3.1 补
  EED4 r v=0×40 行；§5 gs5-anchor-render 占位回填。
- 风险登记：probeN 上限 3M，双流后 gameplay 已 1.95M/3M——后续采集需提上限或分窗。
- 文档 219 行不裁剪（含重采统计=证据）。

> 【2026-10-20 更正】`tools/scenario_capture.lua` probeN 上限已由 3,000,000 提为 **10,000,000**
> （probeLog 首行 `if probeN > 10000000 then return end`）：双流（w+r）注册后事件量翻倍，
> gameplay-stage1 已 1.95M/3M 逼近静默截断点，长场景（如 tas-stg08b2 40,000f 重采）必触顶——
> 取"改上限常量"这一简者而非分段落盘；上文"probe 上限未动/风险登记 3M"两条为历史口径，不再适用。
