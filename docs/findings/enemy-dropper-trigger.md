# 敌型行为语义族②：终段投放器触发（批次 4b）—— $FFFF 哨兵 + op5/op7 触发链

> 状态：2026-10-15 v1（T2，decomp-analyst）——批次 4b 块②（终段投放器触发：$5F/$CA 型 +
> thr=$FFFF 哨兵语义）。前置：[enemy-timeline.md](enemy-timeline.md) §1.2/§1.4（脚本解释器/
> 波次记录机）、[stg02-timeline.md](stg02-timeline.md) §5（波次流模板）、
> [wave-ca-flags.md](wave-ca-flags.md)（CA bit0 臂门）。全部 A 级代码行可回 seed 重导。

## 0. 结论速览

| 面 | 结论 | 级 |
|---|---|---|
| 触发链 | 脚本 thr tick → **op5 装载**（`$CC/$CD` + `$C8:=1` + 分派 `$897F`）→ 23B 记录处理 → 清单单体投放 + CA bit0 臂门 + C9 bit0 far-call | A |
| thr 机制 | `$86/$87` 与 tick `$85:$84` **16 位精确相等比较**（bank00:81A3-81AD）；thr 字段 = **下一记录**触发 tick | A |
| **$FFFF 哨兵** | 精确比较 + tick 单调（滚动尽头冻结，Wt-2）⇒ tick 永不等于 $FFFF ⇒ **后续记录永不触发 = 流尾停泊**（防解释器越流尾的设计占位） | A（代码）+B（意图归纳） |
| op7 | jump-if-wave-busy：`$C8≠0` → 跳转体（共享 op3）：`$88/$89:=目标` + **tick 重基**（新流头 2B → `$84/$86/$85/$87`）；`$C8=0` → 跳过 2B 直行。**正常流程不可达**（前驱 thr=$FFFF） | A |
| 终段投放型 | 关1 流2 清单 `[01][$5F]`、关8 流4 清单 `[01][$CA]`——**count=1 单体**（17 条 op5 流唯一例外=关4 流2 的 2×$87）；配 CA bit0 臂门 + C9 bit0 far-call bank11:$8012 | A |
| 命名警示 | 型值 **$CA** 与 **CA 标志字节**同名异物（前者=对象类型表值不命名；后者=记录标志，[wave-ca-flags.md](wave-ca-flags.md)） | — |

## 1. 触发链总图（A 级逐指令）

```
脚本解释器（bank00:8185-82FA，每 tick）：
  8193 INC $84（tick++）→ 81A3 比较 $85:$84 == $87:$86（精确相等）
  → 命中：81BE 取 op → 81D0 分派表 → handler 消费 args（Y 步进）
  → 82EF 取下一记录 thr → $86/$87 → 回 81A3
op5 handler（bank00:82BF-82D4，set-wavestream）：
  82BF $C8:=1（波次机激活） 82C3 $CE/$CF:=$897F（波次机入口）
  82CB $CC/$CD:=流指针 2B
波次记录机（bank00:8972-8D39，JMP ($CE) 入口 897B）：
  处理 23B 记录 → 清单 [count][type…] → $400+槽（8A9C-8B06）
  C9 bit0 ∧ $26 bit3=0 → far-call bank11:$8012（89C9-89DB；89CF 第三门）
  CA bit0 → $26|=$10 + $D1:=0（8CFC-8D0A）→（B1 §5 链）关末块
```

## 2. thr 机制与 $FFFF 哨兵（A 级代码 + B 级意图）

```
81A3  A5 84   LDA $84 / 81A5 C5 86 CMP $86 / 81A7 D0 06 BNE 81AF   ; 低字节不等→不触发
81A9  A5 85   LDA $85 / 81AB C5 87 CMP $87 / 81AD F0 0F BEQ 81BE   ; 高字节相等→触发
```

- **精确相等**（非 ≥）：tick 逐拍 +1 时每个 thr 值都会被命中——**除 $FFFF**：
  tick 计数 = 16px 滚动进度（Wt-2 裁定），滚动尽头 tick 冻结（关 1 实测冻结 824），
  永不达 $FFFF。⇒ **thr=$FFFF 的记录永不触发；其后内容 = 流尾停泊区**。
- 语义分工：记录尾 thr 字段 = **下一记录**触发 tick（B1 §4.1 注）；**op5 波次装载时刻 =
  op5 记录自身的触发 tick = 前一记录的 thr 字段**（本文口径修正细化）。关8 实测序列：
  rec4 thr=$00F0 → rec5 装 $B830；rec5 thr=$01F0 → rec6 装 $B851；rec6 thr=$0270 →
  rec7 装 $B86D；rec7 thr=$0304 → rec8 装 $B889；**rec8 thr=$FFFF → rec9(op7) 永不触发**。

## 3. op7 jump-if-wave-busy（A 级；防御性尾段）

```
8252  A5 C8   LDA $C8 / 8254 D0 04 BNE 825A   ; 波次机忙（$C8≠0）→ 跳转
8256  C8      INY / 8257 4C EF 82 JMP 82EF    ; 空闲→跳过 2B 直行
825A  B1 88   LDA ($88),Y / PHA / INY / LDA ($88),Y → $89 / PLA → $88
8265  A0 00   LDY #0
8267  B1 88   LDA ($88),Y → $84/$86           ; tick 重基：新流头 2B 同写 tick 与阈值
826D  C8      INY / 826E B1 88 … → $85/$87
8274  C8      INY / 8275 4C A3 81 JMP 81A3    ; 重入比较（换流后继续）
```

- 跳转体与 **op3 jump（$825A）完全共享**；差异只在入口判据（op7 多测 `$C8`）。
- **tick 重基**语义：目标流以自带头 2B 重置 `$84/$85`（tick）与 `$86/$87`（首阈值）——
  与关首快进环（B1 §8.1）同型的「换流重基」机制。
- 各关 op7 尾段实测（thr 前驱均为 $FFFF 哨兵 ⇒ 正常不可达）：
  关1 rec14 target=$080F、关2 rec11 target=$1B0F、关8 rec9 target=$090F（目标地址形态
  跨窗/异常——因不可达无运行语义，仅登记字节，S4b-5）。

## 4. 终段投放型 $5F/$CA（A 级字节；型值不命名）

| 关 | 流 | C9 | CA | 清单 | 组合效果 |
|---|---|---|---|---|---|
| 1 | $A3E4（rec11 @ $04D0 装载） | $FB（bit0+bit1） | $05（bit0+bit2） | **[01][$5F]** | 单体 $5F + far-call bank11:$8012 + 臂关末门 + 清单重指派 |
| 8 | $B889（rec8 @ $0304 装载） | $FB | $81（bit0+bit7） | **[01][$CA]** | 单体 $CA + far-call + 臂门 + 跳过速度恢复（CA b7=C9 b1） |

- **终段投放模板**（A 级，17 流全谱）：末流 = `count=1 单体 + C9 bit0 far-call + CA bit0 臂门`；
  唯一多体例外 = 关4 流2 `[02][$87,$87]`（CA=$01 臂门最小集）。【2026-10-15 更正（port-engineer 矛盾裁定）】~~CA=$01，无 far-call~~ **作废**——far-call 门在 **C9**
  （`C9 bit0 ∧ $26 bit3=0`，bank00:89C9-89D3），与 CA 位无关：关4 流2 `C9=$FB bit0=1` ⇒ **far-call 照常
  条件触发**（stg04-timeline §2 解码正确；原文把 CA 位误当 far-call 门）。第三条件 `$26 bit3` = 运行期抑制门（语义未名，S4b-8；与 8A68 `AND #$88`、8CAE `$26&$88` 同族观测点）。
- $5F/$CA = 对象类型表值，**不命名**（禁发明）；与「中 boss $4A」（B1/W-tas 已语义化）同族
  属终段域投放器（B1 §1.4：波次记录机=中 boss/boss 段编队投放器）。

## 5. 边界（未证不发明）

| 项 | 状态 |
|---|---|
| op7 跳转目标字节形态（$080F/$1B0F/$090F） | 不可达段字节登记（S4b-5）；不反推语义 |
| $5F/$CA 投放后的对象行为 | VM 脚本体（t59 族）边界；行为面见块① §5 |
| 波次记录处理时延（装载→投放的 tick 相位） | 无中 boss 段 trace（S3-2 同族）；stg02 段内调度器处理同帧可见为旁证 |
| 「哨兵=防越流尾」意图 | B 级归纳（代码行为 A 级：永不触发） |

## 6. 复现命令

```bash
# thr/op5/op7 逐指令（§1-§3）
python3 - <<'PY'
import re
pat=re.compile(r'^bank(\d\d):([0-9A-F]{4})\s+((?:[0-9A-F]{2} )*[0-9A-F]{2})\s+(\S+)\s*(.*)$')
for line in open('reference/seed/gunnac.asm'):
    m=pat.match(line.rstrip())
    if m and m.group(1)=='00':
        a=int(m.group(2),16)
        if 0x81A3<=a<0x8280 or 0x82BF<=a<0x82D5: print(line.rstrip())
PY
# 各关脚本尾（thr 序列 + op7 目标，§2/§3）
python3 tools/wave_timeline.py script 1   # 尾部 rec14 op7
python3 tools/wave_timeline.py script 8   # 尾部 rec8 thr=\$FFFF + rec9 op7
```
