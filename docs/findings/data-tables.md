# 数据面考证第一批（t13）— 8 机械数据表 + 7 已知候选区字段切分

> 纪律（AGENTS.md §3）：每条结论分 **已证实 / 假设**；已证实项带字节偏移指针与可复现命令。
> 投影来源：`reference/seed/gunnac.asm`（IDA M6502 冻结 listing，引用格式 `bankNN:CPU`）、
> `analysis/ghidra/bank00-15.c`（Ghidra 6502 伪代码，交叉验证用）；ROM 字节是最终真相（D3）。
> 语义命名登记：`data/naming/ledger.jsonl`（A/B 级 → `applied`，C 级 → `hypothesis` + `_todo_` 前缀）。

**偏移换算**：`file` = ROM 文件偏移；`PRG` = file − 0x10；bank1 CPU = PRG − 0x2000 + 0xA000；
bank0 CPU = PRG + 0x8000；bank15 CPU = PRG − 0x10000。

## 1. H-05 商店价目记录区（rom:8914–9000 = file 0x22D2–0x2328）字段切分

### 已证实（A 级）

**结构假设裁定：`{前缀,编号,名称,价格}` 单记录布局 —— 证伪。** 实际为**按槽位排布的记录流**
（记录格式见 §2）：4 个商品槽位，PPU 行 stride = **$40**，每槽 2 行；字段切分如下：

| 槽位 | 名称记录（`{PPU, "N." + 名称 tile 码, FE}`） | 价格记录（`{PPU, "1 " + 图形码 $D8 + " $" + 数字, FE}`） |
|---|---|---|
| 1 | `$2285: 31 2E BD BE BF`（"1."+3 glyph） | `$2294: 31 20 D8 20 24 31 30` = `1 <D8> $10` |
| 2 | `$22C5: 32 2E 93 94 95` | `$22D4: 31 20 D8 20 24 36` = `1 <D8> $6` |
| 3 | `$2305: 33 2E F9 FA FB` | `$2314: 31 20 D8 20 24 34` = `1 <D8> $4` |
| 4 | `$2345: 34 2E DB DC`（2 glyph） | **缺失**：`45 23 34 2E DB DC FF` 后直接 FF 填充（E-R05 现象在记录级复现） |

- 复现：`xxd -s 0x22B0 -l 176 rom/gunnac_cn.nes`（记录流自 file 0x22B9 起：
  `23 22 c6 c7 c8 fe | 43 22 d6 d7 d8 3f fe | 67 22 ad ae af fe | 75 22 9f c8 fe | …`）。
- "前缀" 2 字节（`85 22` 等）= **记录内 PPU nametable 地址字段**（LE），不是独立前缀结构。
- "编号"（`1.`-`4.`）与名称 = **同一记录 tile 流的前 2 码**（ASCII 0x31-0x34 0x2E）；
  "价格"是**独立记录**（同槽下一行内右列 $2294 = $2285+$0F）。
- 每槽尾随两条辅助记录（如槽 1：`A7 22 A3 A4 A5 FE`、`B5 22 9F C8 FE`）；槽 4 无尾随记录（FF 截断）。
- **消费代码**（A 级，listing 可复现）：`bank00:9CC9` `LDA #$58/STA $12`、`LDA #$A2/STA $13`、`JSR $A09F`
  —— 记录流指针 **$A258**（本面板块）；`bank00:9CF0` 同页第二流 `JSR $E6D9`。
  记录渲染引擎 = bank15 固定 bank `sub_E6DF`（§2）。
- **RAM 变量**（A 级）：`RAM $0400` = 商店槽位光标：`bank00:9D08` `DEC $0400`、`bank00:9D17` `INC $0400`、
  `bank00:9D1D` `CMP #$04` 回绕 0-3 —— 与 4 槽位一致；`bank00:9CAD` `STA $0177,Y` 每槽状态清零。

### 假设（B/C 级）

- 价格记录首码 "1" 三个槽位恒同 → 非槽位号；疑为数量/单件前缀（"1×"）或固定标签。**C 级**。
- 图形码 $D8 为商品类别 icon tile；名称 glyph（$BDBEBF/$939495/$F9FAFB/$DBDC）为汉字字模 tile 码
  （对照 S0 §5 字模区 bank124-127）。**B 级**。
- 4 号槽价缺失 = 汉化/原版数据就地截断（未证实是 BUG 还是刻意；关联 completeness E-R05）。**C 级**。

## 2. 文本记录格式：记录边界与终止规则

### 已证实（A 级，双投影一致）

**记录 = `{ppu_lo, ppu_hi, tile 码流…, 终止字节}`**，终止字节 **≥ $FE**：

- 引擎 bank15 `sub_E6DF`（`bank15:E6DF`；Ghidra `FUN_e6df`，analysis/ghidra/bank15.c:987 起，与 IDA 反汇编逐分支一致）：
  `bank15:E6EF` `LDA ($12),Y / CMP #$FE / BCC …` → **字节 < $FE = 记录内 tile 码**；≥ $FE → 返回（`bank15:E6F5`）。
- **tile 流控制字节**（`bank15:E706-E73A`）：
  - `$5E`：属性翻转开关（`EOR #$20 → $77`）；
  - `$86-$9F`、`$A6-$DD`：`EOR $77` 属性变换后写入；
  - `$A0-$A5`、`$B0`：直接 tile；
  - `$DE-$DF`：特殊处理分支（`bank15:$E78D`，调色板/属性，语义待 S3）。
- **流级终止规则**（`bank00:$96D8-$971B`，同族消费器）：`FE` = 本记录结束、继续读下一条记录
  （`bank00:96DD` `CMP #$FE/BEQ` 回主环）；`FF` = **整块结束**（`bank00:96E1` `LDA #$1/STA $0180`，退出）。
- 记录边界 = 上一记录终止字节后 1 字节即下一记录 `ppu_lo`；五区（标题 file 0x10AE、剧情 0x2CDC、
  结局 0x2FFE、STAFF 0x1969、商店面板 0x22B9）全部满足（hexdump 可逐一复核）。
- 块结束后观察到填充字节（00 或 FF）；个别块 FF 后随 2 字节 **$FFFF / $0000**（如 file 0x2328、0x2165）
  —— 为后续块指针或填充，两者未分辨（假设区）。
- 关联证据：S0 rom-forensics §4 记录实录；E-R04 金钱串 `-------$` = 记录 `{23 22, 2D×7, 24}`（file 0x2144）。

### 假设（B/C 级）

- FF 后 2 字节 = 块链 continuation 指针（$0000/$FFFF = 终止）—— 与对象脚本流约定
  （`bank00:8485` 族：`$FE`=计数循环、`$FF`=2 字节跳转指针）同构，但文本块消费器未见解引用。**C 级**。
- `$DE-$DF` 分支为调色板列选择。**C 级**。

## 3. charmap 重排表（PRG 0x01190 / 0x01830，关联 H-08）

### 已证实（A 级，结构事实）

- **表 A**（PRG 0x01190–0x01220 = file 0x11A0–0x1230；复现 `xxd -s 0x11A0 -l 144 rom/gunnac_cn.nes`）：
  单调递增 tile 索引流（0x0A…0x84），**不规则 0x00 空洞**（如值 0x5B、0x5E、0x6A、0x75、0x78、0x84 处缺失，
  空洞后数值继续跳增）。
- **表 B**（PRG 0x01830–0x018F8 = file 0x1840–0x1908）：同类结构但空洞模式不同，且与 8 字节记录
  `DE E9 .. .. .. .. 46 20`（S0 假设 H-2）交错。
- **无绝对地址 xref**：全 16 bank 的 IDA xref 集内无指向 $9190-$9220 / $9830-$9900 的代码操作数
  （仅表内自引用噪声）→ 两表经**间接指针（立即数装载 $14/$15 或 ($xx),Y）**访问（机械扫描结论）。

### 假设（B/C 级）

- 表 = **字符码 → 字形 tile 索引映射**（0x00 = 未映射字符）；空洞为汉化替换字形后腾挪 tile 号所致
  （即 H-08 汉化字模映射线索）。**C 级**（未定位读取代码，语义未闭环）。
- 表 A/表 B 分属两套文本系统（剧情/面板）。**C 级**。

## 4. 武器/炸弹/关卡平行串（rom:9095 / 9534 / 9626）与 RAM 引用关系

### 已证实（A 级）

三串均为**记录流内单条记录**，PPU 地址**同为 $22E9**（三个平行面板共用同一 VRAM 行）：

| 证据 | 记录（file 复现偏移） | 槽位数 | 面板装载（listing） | 光标/槽位 RAM |
|---|---|---|---|---|
| E-R06 | `{E9 22, "1 2 3 4 5 " + CD CE}`（`xxd -s 0x2386`） | **5** | `bank00:9D62` 流 $A336 族 | `$0401`：`bank00:9D91 CMP #$05` |
| E-R07 | `{E9 22, "1 2 3 4 5 6 7 8"}`（`xxd -s 0x253E`） | **8** | `bank00:9F25` 流 **$A50D** | `$0401/$0402/$0403`：`bank00:9F1C-9F50` |
| E-R08 | `{E9 22, "F B T W"}`（`xxd -s 0x259A`） | **4** | `bank00:9F78` 流 **$A56F** | `$0178,Y` 2 位字段 → `$0401`（`bank00:9F6A-9F72`） |

- 槽数与外部资料（E-W01/E-W02/E-W04）**三方一致**：5 主武器、8 关、4 炸弹（B 级语义绑定）。
- 三面板块模板平行（块首记录同构 `23 22 6E 6F 80 C6 C7 C8 …`，file 0x251C/0x2580 vs 0x2344）。
- `bank00:9F67` `LDY $0403 / LDA $0178,Y / ROL×3 / AND #$03` = 每槽炸弹类型 2 位字段（0-3）。

### 假设（B/C 级）

- E-R06 串 = 武器选择面板槽位条、E-R07 = 关卡选择、E-R08 = 炸弹选择（字母 F/B/T/W 语义未考）。**B 级**。
- E-R08 尾码 `CD CE`（E-R06 串尾）为方向/确认箭头 tile。**C 级**。

## 5. 8 个机械数据表（remap-table.json t_000–t_007）切分状态

| id | 位置 | 大小 | 切分结论 | 级 |
|---|---|---|---|---|
| t_000 | bank1 $AD64-$B18A | 1063B | **开场/结局文本记录流**（记录格式 §2；起点落在 "PEACE,HOWEVER,IS ABOUT" 记录内，含结局段 $AFE4+） | A |
| t_001 | bank1 $B2A4-$BFFF | 3420B | 4 字节组结构（如 `31 9D 00`、`31 9F 00`、`33 A1 00` 第 3/4 字节递增）→ **OAM/精灵模板表候选**；字段切分未定 | C |
| t_002 | bank2 $8678-$9FFF | 6536B | **代码/数据混排区**（头 32B 即 `JSR/JMP/RTS` 码流）：线性扫描误归数据；需 S3 重分类后切分 | B |
| t_003 | bank5 $8F8D-$9FFF | 4211B | 头部 `34 30 2C 28 24 20 …` 递减曲线字节 → **数值表候选**（速度/轨迹/音量包络） | C |
| t_004 | bank6 $93A8-$9FFF | 3160B | 头部呈 ~13 字节周期（`…40 00 00 10 00 00 03 80 00…` 小整数重复）→ **配置/生成记录表候选** | C |
| t_005 | bank8 $ADD0-$BFFF | 4656B | 5 字节组含重复 LE 指针字（$B2B4/$B3E6/$B235，指向 bank8 本窗）→ **指针型记录表候选**（脚本/关卡段） | C |
| t_006 | bank10 $97B0-$97BA | 11B | 7 字节记录形（`10 01 C1 03 3C 00 00`）→ **配置记录候选**（仅 1.5 条） | C |
| t_007 | bank15 $FD2A-$FFFF | 726B | **大字模/日期文本块**（含 file 0x1FF10 "1990-12-17 USA" 与 "YOU…AGAIN!" 大字流）+ **向量表 $FFFA-$FFFF 被并入**（xref 计数含向量）→ 边界需 S3 修正 | A(内容)/B(边界) |

- 复现：`python3 tools/remap/run.py`（重建 t_000–t_007 边界与 xref 计数）+ 上表对应 `xxd -s <file> -l 32 rom/gunnac_cn.nes`。

## 6. 7 个已知候选区（known_data_candidates）状态

| 候选 | 切分结论 | 级 |
|---|---|---|
| charmap A（PRG 0x01190） | §3 表 A | A(结构)/C(语义) |
| charmap B（PRG 0x01830） | §3 表 B | A(结构)/C(语义) |
| 文本记录区-标题（0x0109E-0x01123） | 记录格式 §2 已证；边界=首记录 ppu_lo 至块 FF（file 0x10AE-0x1133） | A |
| 文本记录区-剧情（0x02CDC-0x02F8B） | 同上；行记录 PPU 复用 $2223/$2262…（滚动文本） | A |
| 文本记录区-结局（0x02FE4-0x03155） | 同上；`OPTION DEMO` 尾随 | A |
| STAFF 字幕区（0x01959-0x01BC0） | **独立记录变体**：`{1 字节前缀, 文本, FE}`（如 `FE 09 "AND" FE`），非 {PPU,tiles} 双字头；含 `@2011 JIUBAN`（0x01BB2） | A |
| 大字模文本块（0x1FF00-0x1FFEF） | 单块大字 tile 流（"1990-12-17 USA" + 3×3 tile 宽字模 "YOU…AGAIN!"） | A |

## 7. 遗留假设清单（不得作结论/命名依据）

| # | 假设 | 级 | 去向 |
|---|---|---|---|
| D1 | 价格记录首码 "1" 的语义（数量/固定标签） | C | S2 trace 观察写入序列 |
| D2 | FF 后 2 字节 = 块链指针 | C | S1 消费器定点反汇编 |
| D3 | charmap A/B = 字符→字形映射（H-08） | C | 定位间接读取代码 |
| D4 | `$DE-$DF` 控制字节语义 | C | S3 |
| D5 | t_001 精灵模板、t_003 数值曲线、t_004 配置、t_005 指针记录、t_006 配置的字段语义 | C | S3 逐表 |
| D6 | E-R08 字母 F/B/T/W 对应炸弹名 | C | 与 docs/spec 对齐 |
| D7 | 4 号商品价缺失是截断 BUG 还是刻意（→bug-ledger?） | C | S2 复现商店画面 |

## 8. 复现命令附录

```bash
# H-05 商店价目记录区（含 4 槽位 name/price 记录与 FF 截断）
xxd -s 0x22B0 -l 176 rom/gunnac_cn.nes
# 文本记录 FE/FF 终止样例（标题块尾 FF、金钱串 FF）
xxd -s 0x10AE -l 144 rom/gunnac_cn.nes; xxd -s 0x2140 -l 32 rom/gunnac_cn.nes
# charmap 表 A / B
xxd -s 0x11A0 -l 144 rom/gunnac_cn.nes; xxd -s 0x1840 -l 200 rom/gunnac_cn.nes
# 平行串（5/8/4 槽位条，均 PPU $22E9）
xxd -s 0x2386 -l 32 rom/gunnac_cn.nes; xxd -s 0x253E -l 32 rom/gunnac_cn.nes; xxd -s 0x259A -l 32 rom/gunnac_cn.nes
# 消费引擎与面板装载（冻结投影）
grep -n "bank15:E6DF\|bank15:E6EF\|bank00:96DD\|bank00:9CC9\|bank00:9F67" reference/seed/gunnac.asm
sed -n '987,1050p' analysis/ghidra/bank15.c        # Ghidra FUN_e6df 交叉验证
# 机械表边界与 xref 计数重建
python3 tools/remap/run.py
```

## 9. 台账登记摘要

`data/naming/ledger.jsonl` 本次登记 10 条（N-0001…N-0010）：8 条 A/B 级 `applied`
（shop_price_panel_records、shop_slot_cursor、ui_select_cursor、weapon_slot_strip、
stage_slot_strip、bomb_slot_strip、bomb_type_per_slot、text_record_terminator），
2 条 C 级 `hypothesis`（`_todo_charmap_glyph_map_a/b`）。未做任何 reference/ 轨道改名
（seed 冻结不可变，D3）；rename 落地留待命名任务按台账执行。
