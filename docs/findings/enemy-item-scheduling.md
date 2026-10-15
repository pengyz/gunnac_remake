# 敌型行为语义族③：item 排程语义（批次 4b）—— 记录内原子批量投放

> 状态：2026-10-15 v1（T2，decomp-analyst）——批次 4b 块③（item 排程：关4 多 item 18 条/
> set10 峰值的投放器逻辑）。前置：[enemy-timeline.md](enemy-timeline.md) §1.3（记录机）、
> [stg04-timeline.md](stg04-timeline.md)（关4 差异面）、[wave-ca-flags.md](wave-ca-flags.md)。
> 本篇 = port-engineer 批次 4 排程面输入。

## 0. 结论速览

| 面 | 结论 | 级 |
|---|---|---|
| 记录语义 | `[delay@0][项…][ctrl]`：**项序列在同一 tick 内原子顺序消费**——一条记录 = 一次批量投放 | A |
| 项语法 | 项 = `(type≥$10, param)` pair 直投 **或** ctrl 项（1/2/11/12 组长流、10 流内定型、13/14 寄存器写）；ctrl0=记录尾、ctrl3..9=流跳转（唯二出口） | A |
| pair 直投 | 槽分配 `$C304` → `$400+槽:=type`、`$470+槽:=param`、`JSR $EEE5(A=8)` 副效果、`$4B` 饱和计数 | A |
| set10 | 槽分配 `$C2C6` → `$400/$470` ← 流 2B + **`$48C:=(\$3E),0` 码流取参 + `INC \$3E/\$3F`**（每次消费 1 码流字节）→ 回项环 | A |
| ctrl1/2/11/12 | 组长投放 + 四参写（→组员流**跨 tick 滴灌**，stg02 §3）→ **回项环**（可与后续项同记录） | A |
| 双分配器 | pair 项=`$C304`、ctrl1/2/10=`$C2C6`（两变体，B 级注） | A |
| 排程风格谱 | **批量压缩**（关4：18 多 item/记录内 pair×3、[1][1][1] 三组长同 tick）↔ **单发长尾**（关6：85 记录/item 恒单）——两极对照的机制根因=记录内项数 | A |

## 1. 项消费环（A 级，bank00:86CE-870D 逐指令）

```
86CE  LDY #1              ; 跳过 delay@0
86D0  LDA ($14),Y          ; 项首字节
86D2  CMP #$10
86D4  BCC 86FC             ; < $10 ⇒ ctrl 分派
      ---- pair 项（type≥$10）----
86D6  STA $13              ; type
86D8  INY / 86D9 STY $12   ; 保存参数位置
86DB  JSR $C304            ; 槽分配器（变体 A）
86E1  BCS 86F9             ; 无空槽 → 跳过本项
86E3  TAX                  ; X=槽
86E6  STA $400,X           ; $400+槽 := type
86E9  LDA ($14),Y
86EB  STA $470,X           ; $470+槽 := param
86EE  LDA #8 / 86F0 JSR $EEE5   ; 副效果（A=8；效果未名 S4b-6）
86F3  INC $4B / BNE / DEC $4B   ; 饱和计数（$FF 不回绕 0）
86F9  INY / 86FA BNE 86D0  ; 下一项（项环）
      ---- ctrl 分派 ----
86FC  INY / 86FD ASL A
86FE  BEQ 86CD             ; ctrl0 ⇒ 记录结束（RTS）
8702  TAY / JMP ($870E,Y)  ; ctrl 跳表（1..14，B1 §1.3 同表）
```

**出口唯二**：ctrl0（记录尾）或 ctrl3..9（流跳转，`$88` 型新基址 + Y=0 + RTS，8795-87A3）；
**其余 handler 全部 `JMP loc_86D0` 回项环**（ctrl1 8770 / set10 87CA）——故记录可**混排**任意项序列。

## 2. set10 流内定型（A 级，bank00:87A4-87CA）

```
87A4  JSR $C2C6            ; 槽分配器（变体 B）87A7 BCS 87C1 失败跳
87AB  LDY $12 / LDA ($14),Y → $400,X    ; type := 流字节
87B2  INY / LDA ($14),Y → $470,X        ; param := 流字节
87BA  LDY #0 / LDA ($3E),Y → $48C,X     ; 附加参数 := 码流当前字节
87C1  LDY $12 / INY
87C4  INC $3E / BNE / INC $3F           ; 码流步进（失败分支亦步进）
87CA  JMP 86D0             ; 回项环
```

- set10 是**项环成员**（非终止单元）：一条记录可含多个 set10 + pair + ctrl 混排。
- **码流耦合**：每 set10 消费 `($3E),0` 一字节——关4 set10×32 ⇒ 码流 32B 消费面
  （码流 = sub_8619 输出执行器族的输入，B1 §1.5；语义未名 S4b-6 同族）。

## 3. 关4 排程实例（A 级，`sched 4` 重导）

| 记录 | +tick | 项序列 | 语义 |
|---|---|---|---|
| @ADB2 | +25t | pair($84,$30) pair($84,$D0) [13]reg | 同 tick 双直投 + 寄存器写 |
| @ADD5 | +65t | [10]T=$2A(p=$00) pair($84,$80) | set10 与 pair 混排 |
| @ADEA | +97t | pair($84,$C0) [1]T=$2E(7E,04,28,00) | 直投 + 组长流（组员后续 40f 滴灌） |
| @AE38 | +237t | pair($84,$30) pair($84,$80) pair($84,$D0) | **三连直投**（同型 $84 三参变体） |
| @AE7F | +357t | [1]T=$2E(7C,04,3C,00) [1]T=$2E(7D,03,50,00) [1]T=$2E(84,02,5A,40) | **三组长同 tick**——三条独立组员流并行挂载（4/3/2 员、60/80/90f 间隔） |

**对照关6**（单发长尾）：85 记录 × 恒单 item（ctrl1×45/ctrl2×30/pair×10）——同总量投放
摊在 1125t；关4 用 60 记录/861t 打包 78 个投放动作（ctrl1/2 35 + set10 32 + pair 11）。**排程风格差异 = 记录内项数分布**，
与 set10 峰值（关4 32 vs 关6 0）共同构成批量/单发两极（S6-2 横切对照的机制解释）。

## 4. 组员滴灌 vs 批量原子（移植关键，A 级）

| 投放通道 | 时序 | 机制 |
|---|---|---|
| pair/set10 直投 | **tick 原子**（记录内全量一次落地） | 项环顺序消费 |
| ctrl1/2/11/12 组员流 | **跨 tick 滴灌**（$534 帧间隔逐员） | 四参挂载后由组员调度器展开（stg02 §3） |
| ctrl13/14 | tick 原子（寄存器写 $E63D） | — |

移植时两条时序域不可混（与 stg02 §6.1 的 tick/帧双域一致）。

## 5. 边界（未证不发明）

| 项 | 状态 |
|---|---|
| `JSR $EEE5(A=8)` 副效果 | 效果未名（S4b-6；疑音画/事件登记） |
| `$4B` 饱和计数器用途 | 未名（S4b-6） |
| `$C304`/`$C2C6` 分配器差异语义 | 变体行为未展开（槽序/保留槽差异待证，S4b-7） |
| set10 码流字节语义（($3E),0 → $48C） | 码流=动作参数族（B1 §1.5）；单字节语义未名 |
| ctrl 跳转（3..9）后的流内容 | 各关未见跳转实触发（记录尾/哨兵已覆盖主流程） |

## 6. 复现命令

```bash
# 项环/set10/ctrl 分派（§1/§2）
grep -n "" reference/seed/gunnac.asm | sed -n '/bank00:86CE/,/bank00:87CD/p'
# 关4 多 item 实例（§3）
python3 -c "
import tools.wave_timeline as w
d=w.load()
for r in w.iter_sched_records(d,4,maxrec=300):
    if r.get('loop'): break
    if len(r['items'])>1: print(f"@{r['addr']:04X} +{r['tick']}t:", r['items'])
"
# 两极对照（§3 末）：关4 vs 关6 记录/item 分布
python3 -c "
import tools.wave_timeline as w, collections
d=w.load()
for n in (4,6):
    acts=recs=0; multi=0
    for r in w.iter_sched_records(d,n,maxrec=300):
        if r.get('loop'): break
        recs+=1; acts+=len(r['items']); multi+=len(r['items'])>1
    print(f'STG-{n}: records={recs} actions={acts} multi_item={multi}')
"
```
