# 波次记录 $CA 标志位义横切定谳（S3-3）—— 八关 17 记录全谱

> 状态：2026-10-15 v1（T2，decomp-analyst）——S3-3 收口：CA 八位全解析（bit0-2 已知位复核 +
> **bit4-7 语义定谳**）。读者/写者全集：bank00:8987 单写点 + 9 读点（$CA=波次记录机私有标志，
> 无其他 RAM 写主——全 ROM `$CA` 操作数扫描 10 点全列）。
> 依据：[stg02-timeline.md](stg02-timeline.md) §5（波次记录字段）、[enemy-timeline.md](enemy-timeline.md)
> §1.4（记录机）、八关差异面 stg03..stg08-timeline。**本篇 = port-engineer 批次 4 CA 面输入**。

## 0. 结论速览（A 级：掩码逐指令 + 17/17 全谱相关性）

| 位 | 语义 | 探测点 | 全谱验证 |
|---|---|---|---|
| bit0 | **臂关末门**：`$26|=\$10` + `$D1:=0`（B1 §5 链）；副支见 §2 | bank00:8CFC-8D0A | 17/17：bit0=1 ⟺ 该关**末流**（$05/$31/$01/$B1/$81） |
| bit1 | **速度恢复增量**：`$82/$83 ← 快照 $192/$191` | bank00:8CDD-8CEB | 数据未现（§4） |
| bit2 | **清单 type 匹配重指派**（跳空槽分配，按已有 type 槽改指） | bank00:8AB1-8ADE | 仅关1 $05（B1 已录） |
| bit3 | **状态机强制** `$C8:=6`（已 6 则 RTS） | bank00:8B4A-8B58 | 数据未现（§4） |
| **bit4&5** | **写表1 执行**（组合判 `AND #$30==#$30` → 分派 `$CE/$CF:=$8A90`，`sub_8D8D Y=4` 读字段 0x04/0x05） | bank00:8A80-8A8E + 8A97-8A99 | **17/17 全等**（§3） |
| **bit6** | **写表2 执行**（`sub_8D8D Y=$0C` 读字段 0x0C/0x0D） | bank00:8C10-8C18 | **17/17 全等**（§3） |
| **bit7** | **速度恢复段选择**（与 **C9 bit1 异或**：`CA b7 ≠ C9 b1` → 走 8CDD 恢复段；相等 → 跳过） | bank00:8CC9-8CDB | 2/2 bit7 记录一致（§2） |

**CA 的角色定位（B 级归纳）**：波次记录的**收尾动作集**——臂门/速度恢复/清单策略/状态强制/
写表执行/路径选择六类 epilogue 动作各占一位（bit4&5 合判一位组）；与 C9（前置动作：速度快照/
far-call/跳转）配对构成记录机的完整行为面。

## 1. 读者/写者全集（A 级，全 ROM `$CA` 操作数扫描）

```
bank00:8987  85 CA   STA $CA        ← 唯一写点（记录装载：LDA ($CC),Y Y=1 → $CA）
bank00:8A80  A5 CA   LDA $CA        bit4&5 组合判（写表1 分派）
bank00:8AB1  A5 CA   LDA $CA        bit2（清单重指派）
bank00:8B4A  A5 CA   LDA $CA        bit3（$C8:=6）
bank00:8C10  A5 CA   LDA $CA        bit6（写表2）
bank00:8CA1  A5 CA   LDA $CA        bit0 副支（§2）
bank00:8CCF  A5 CA   LDA $CA        bit7（路径 A：C9 bit1=0 时）
bank00:8CD7  A5 CA   LDA $CA        bit7（路径 B：C9 bit1=1 时）
bank00:8CDD  A5 CA   LDA $CA        bit1（$82/$83 恢复）
bank00:8CFC  A5 CA   LDA $CA        bit0 主支（臂关末门）
```

## 2. 逐位语义（代码行级，bank00 固定窗=引擎全局）

### 2.1 bit0 主支（臂关末门，A 级）

```
8CFC  A5 CA   LDA $CA / 8CFE 29 01 AND #$1 / 8D00 F0 0A BEQ 8D0C
8D02  A5 26   LDA $26 / 8D04 09 10 ORA #$10 / 8D06 85 26 STA $26   ; $26|=关末门
8D08  A9 00   LDA #$00 / 8D0A 85 D1 STA $D1                        ; $D1:=0（B1 未录的第二效果）
```

### 2.2 bit0 副支（A 级字节 / 效果未名挂 S3-3b）

```
8CA1  A5 CA   LDA $CA / 8CA3 29 01 AND #$1 / 8CA5 D0 17 BNE 8CBE   ; bit0=1 → 跳过
8CA7  20 BD 8D JSR sub_8DBD / 8CAA 90 12 BCC 8CBE                  ; 需 SEC
8CAC  A5 26   LDA $26 / 8CAE 29 88 AND #$88 / 8CB0 D0 0C BNE 8CBE  ; 需 $26&$88==0
8CB2  20 56 ED JSR $ED56 / 8CB5 AC 80 01 LDY $180 …                 ; 效果未名（含关号参）
```

`sub_8DBD`（8DBD-8DCB）= `SEC ⟺ C9 bit2=0 ∧ C9(bit0|bit3)≠0`。副支全条件：
**CA bit0=0 ∧ C9 见上 ∧ $26&$88==0 → 调 $ED56**（音画/事件面，效果未证 S3-3b）。

### 2.3 bit7 异或路径选择（A 级双支推导）

```
8CC9  A5 C9   LDA $C9 / 8CCB 29 02 AND #$2 / 8CCD D0 08 BNE 8CD7   ; 入口路径 = C9 bit1
8CCF  A5 CA   LDA $CA / 8CD1 29 80 AND #$80 / 8CD3 F0 27 BEQ 8CFC  ; 路径A：b7=0→8CFC
8CD5  D0 06   BNE 8CDD                                              ; 路径A：b7=1→8CDD
8CD7  A5 CA   LDA $CA / 8CD9 29 80 AND #$80 / 8CDB D0 1F BNE 8CFC  ; 路径B：b7=1→8CFC
                                                                     ; 路径B：b7=0→8CDD
```

**合成规则**：`8CDD`（速度恢复段：$80/$81/$7F←快照流 + $82/$83 视 bit1）执行 ⟺
**CA bit7 ≠ C9 bit1**；相等则直跳 8CFC（保留当前速度）。两段最终都汇入 8CFC 臂门检查。

### 2.4 bit1/bit3/bit4&5/bit6（A 级）

- bit1（8CDD-8CEB）：`$82:=$191 / $83:=$192`（快照=$C9 bit1 时装的 $191/$192）——恢复滚动速度对。
- bit3（8B4A-8B58）：`$C8:=6`（波次机状态强制；`$C8==6` 则直接 RTS）。
- bit4&5（8A80-8A8E）：`AND #$30 / CMP #$30 / BNE 8A9C` → `$CE/$CF:=$8A90`（下轮分派）并落入
  `$8A90` 段（`LDY #4 / JSR sub_8D8D` = **写表1（字段 0x04/0x05）执行**，另受 `$CB bit1` 门）。
- bit6（8C10-8C18）：`LDY #$0C / JSR sub_8D8D` = **写表2（字段 0x0C/0x0D）执行**。

## 3. 全谱相关性验证（A 级 17/17；`wave 1..8` 重导）

| 关 | 流 | CA | 位集 | 写表1 | 写表2 | b45↔w1 | b6↔w2 | bit0=末流 |
|---|---|---|---|---|---|---|---|---|
| 1 | $A3CD | $00 | {} | $0000 | $0000 | ✓ | ✓ | 否 |
| 1 | $A3E4 | $05 | {0,2} | $0000 | $0000 | ✓ | ✓ | 是 |
| 2 | $B1CE | $00 | {} | $0000 | $0000 | ✓ | ✓ | 否 |
| 2 | $B1E5 | $31 | {0,4,5} | $B1FC | $0000 | ✓ | ✓ | 是 |
| 3 | $A5BF | $70 | {4,5,6} | $A5D6 | $A5D9 | ✓ | ✓ | 否 |
| 3 | $A5DC | $31 | {0,4,5} | $A5D6 | $0000 | ✓ | ✓ | 是 |
| 4 | $AF51 | $00 | {} | $0000 | $0000 | ✓ | ✓ | 否 |
| 4 | $AF68 | $01 | {0} | $0000 | $0000 | ✓ | ✓ | 是 |
| 5 | $B7D7 | $70 | {4,5,6} | $B7F9 | $B7FC | ✓ | ✓ | 否 |
| 5 | $B801 | $31 | {0,4,5} | $B816 | $0000 | ✓ | ✓ | 是 |
| 6 | $A5D9 | $00 | {} | $0000 | $0000 | ✓ | ✓ | 否 |
| 6 | $A5F0 | $01 | {0} | $0000 | $0000 | ✓ | ✓ | 是 |
| 7 | $AF7F | $B1 | {0,4,5,7} | $AF96 | $0000 | ✓ | ✓ | 是 |
| 8 | $B830 | $70 | {4,5,6} | $B847 | $B84C | ✓ | ✓ | 否 |
| 8 | $B851 | $70 | {4,5,6} | $B868 | $B84C | ✓ | ✓ | 否 |
| 8 | $B86D | $70 | {4,5,6} | $B884 | $B84C | ✓ | ✓ | 否 |
| 8 | $B889 | $81 | {0,7} | $0000 | $0000 | ✓ | ✓ | 是 |

**计数：bit4&5↔写表1 非零 17/17；bit6↔写表2 非零 17/17；bit0 ⟺ 该关末流 17/17**。

## 4. 未现位与残余（诚实边界）

| 项 | 状态 |
|---|---|
| bit1（$82/$83 恢复）| 代码已证（A），八关数据**未置位**——移植按代码面实现，行为断言待动态场景 |
| bit3（$C8:=6）| 同上，未置位 |
| bit7 观测面 | 仅 2 条（关7 $B1/关8 $81），与 C9 bit1=1 组合=跳过恢复段——与异或规则一致 ✓ |
| C9 bit1=0 路径（8CCF 入口）| 八关波次流未现（C9 全为 $F2/$E2/$FB 均 bit1=1）——代码面已录 |
| $ED56 副支效果 | 未名（S3-3b）；`sub_8D8D` 写表流语义=S2-5 同族（写表内容解读） |

## 5. 复现命令

```bash
# 全谱相关性表（§3）+ CA 值全景
python3 - <<'PY'
import importlib.util
spec=importlib.util.spec_from_file_location('wsd','tools/wave_stream_decode.py')
wsd=importlib.util.module_from_spec(spec); spec.loader.exec_module(wsd)
d=wsd.load()
for n in range(1,9):
    st=wsd.script_streams(d,n)
    for ptr,thr in st['waves']:
        r=wsd.decode_wave_record(d,st['bank'],ptr)
        ca=r['ca']
        print(f"{n} ${ptr:04X} CA=${ca:02X} bits={{{','.join(str(i) for i in range(8) if ca&(1<<i))}}} "
              f"w1=${r['wtab1']:04X} w2=${r['wtab2']:04X}")
PY
# 读点/掩码（§1/§2）
grep -n "LDA \$CA\|STA \$CA" reference/seed/gunnac.asm
```
