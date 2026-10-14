# 对抗复审 #29（t221）：t218 待证族（attr 语义/图块候选）复审

> 被审方：t218（98cd85b，`pending-families.md`）。审核方：verifier（t221），OAM 精确口径重导。
> **裁定：pass（待证族取证收口）**——①②③④全过，attr 分布与 bit7 全 0 **逐位复现**。

## 1. ①弹幕 attr 语义复核 —— ✓✓ 逐位

**tile 251/253 精确口径重导**（n=2668）：

| attr | 声称 | 实测 | 判 |
|---|---|---|---|
| $12 | 952 | **952** | ✓ |
| $52 | 876 | **876** | ✓ |
| $02 / $01 / $42 | 280/280/280 | **280/280/280** | ✓ |
| bit7（V 翻转位） | 全程 0 | **0/2668** | ✓ |

- **位语义核**：NESdev OAM attr=vhp000pp——**bit7=V 翻转、bit6=H 翻转**；
  $52=bit6(H)+bit4+pal2、$42=bit6(H)+pal2 ✓——t218"$52=H 翻转非 V 翻转"**正确**
  （本审初读时的疑虑系自查位序误记，以 NESdev vhp 序为准——已自纠）；
- "V 翻转变体在渲染变换层不存在于该族" ✓ 与 bit7 全 0 一致。

## 2. ②图块 20-23 候选 B 级处置核 —— ✓ 合规

- bank22:31 = 58/64（90.6%）候选关联 + **简单 streak 形 FP 警示（t147 F2 族）** ✓；
- **B 级不入结论、作关联注** ✓——未证不发明合规；
- "bit0 分窗预测（251 odd→96/97）与 bank22 候选不符"张力注 ✓ 如实（渲染层归属待定/or FP——与 t214 W 项一致）。

## 3. ③形 2 零命中核 —— ✓

tile 253 形 2 全翻转穷举（vflip×hflip×128 bank×64 tile，≥56 门）**零命中** ✓——
"tile 源未穷尽（W76）"如实开放 ✓ 不发明。

## 4. ④自查留痕核 —— ✓

N-0174/N-0175 落账（2 命中）+ §4 grep 在 ✓。

## 5. 复现命令

```bash
python3 - <<'PY'   # ① tile 251/253 attr 精确口径
import json
from collections import Counter
oam={}
for l in open('analysis/traces/gameplay-stage1.probe.jsonl'):
    e=json.loads(l); a=e.get('a','').lower()
    if a.startswith('02'): oam.setdefault(e['f'],{})[int(a,16)]=e['v']
c=Counter(); b7=0
for pg in oam.values():
    for s in range(64):
        t,at=pg.get(0x201+s*4),pg.get(0x202+s*4)
        if t in (251,253) and at is not None: c[at]+=1; b7+=at&0x80!=0
print(dict(sorted(c.items())), b7)
PY
grep -c "N-0174\|N-0175" data/naming/ledger.jsonl   # ④
```
