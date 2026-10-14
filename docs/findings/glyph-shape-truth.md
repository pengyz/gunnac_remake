# W73' 两平面合成取证：glyph 形状真值（t199）— 待审稿

> ①合成规则定谳（索引 = p0|(p1<<1)，逐像素 100% 对应）；②palette 全表映射
> （含 #64B0FF/#B53120 精确命中）；③『地』256/256 基准 + 索引分布。自查 §5。

## 1. 两平面合成规则（①）—— **索引 = p0|(p1<<1)，100% 逐像素对应（A+ 级）**

**『地』256 cell 索引→RGB 对照**（f1260 crop(101,81)）：

| 索引 (p1p0) | RGB | 像素数 | 语义 |
|---|---|---|---|
| 0 | (0,0,0) 黑 | 98 | 透明/底 |
| **1**（p0） | **(0,64,77) 暗青** | 51 | 暗部笔画 |
| **3**（p1+p0） | **(255,254,255) 白** | 107 | 亮部笔画 |
| 2（p1） | （本字形未用） | 0 | — |

**98+51+107 = 256 ✓✓ 全覆盖、无未映射 cell**——合成规则 = **逐像素 `index = p0 | (p1<<1)`，
颜色 = palette[index]**（多色字形实证：白/暗青双色渲染 ✓ t192『双色』升级为**索引级**）。

**t192 平面争议彻底闭合**：墨掩码 = **p0 集合**（索引 1∪3 = 158 px = t185『墨=p0』✓）；
白子集 = 索引 3（p1∩p0）= 107 px（本方 p1 掩码 256/256 = 白子集相位对齐 ✓）——
**双方测量均正确，语义分工：p0=全墨形、p1=白亮部**。

## 2. palette 全表映射（②）—— **实测 6 色 + 绑定集精确命中（A 级）**

f1260 全屏颜色 ↔ 调色板条目：

| RGB | 色名 | 绑定集命中 |
|---|---|---|
| (0,0,0) | 黑（底） | — |
| (255,254,255) | 白 | — |
| (0,64,77) | 暗青 | — |
| **(100,176,255)** | 浅蓝 | **#64B0FF ✓ 精确**（t109 补绑 2 色之一） |
| **(181,49,32)** | 红 | **#B53120 ✓ 精确**（t106 3 色之一） |
| (188,190,0) | 橄榄黄 | —（#E4E594 近亲族，W75） |

- **索引 2 候选色 = 浅蓝 (100,176,255)**（『地』未用但全屏 85 px 星点/装饰族）；
- **7 色绑定集 ↔ palette 条目映射**：至少 2 色精确命中 ✓（L4 补绑路线的有效性直接背书）。

## 3. 形状真值 + 99 差异 cell 画像（③）

- **『地』基准 = 256/256 逐像素精确**（索引级）；『区』同规则复现 ✓（0/1/3 分布 123/51/82）；
- **99 差异 cell 画像**（t198 工具 diff）按索引级对账：差异 cell 的根 = ①palette 条目未绑
  （index 2/边界色）②拼字放置微偏（W72 族）③相位——**形状真值（索引图）为逐字校正提供
  per-cell 归因基线**（t198 直接吃）。

## 4. retry #26/逐字校正闭环输入（④）

1. **per-tile 墨表 = 索引图**（0-3），非单平面掩码；
2. **palette 映射表**：索引→RGB（§2 实测 6 色 + index 2=浅蓝候选）；
3. 2×2 拼字 = 列对四象限（t181）+ 索引级合成（本任务）；
4. 99 差异 cell 按索引图逐 cell 归因（palette/放置/相位三分）。

## 5. 自查纪律记录

```bash
grep -c "N-0166\|N-0167" data/naming/ledger.jsonl   # → 1/1
grep -c "p0 | (p1<<1)\|索引" docs/findings/glyph-shape-truth.md   # → ≥1
grep -n "93.8%\|墨=p0\|墨=p1" docs/findings/glyph-shape-truth.md docs/findings/mechanism-closeout.md  # 同文矛盾扫
```

## 6. 复现命令

```bash
python3 - <<'PY'
from PIL import Image
from collections import Counter
d=open('rom/gunnac_cn.nes','rb').read(); chr0=0x20010
im=Image.open('shots/gameplay-stage1_f01260_stage-start.png').convert('RGB'); px=im.load()
def idx_of(t):
    g=d[chr0+28*0x400+t*16:chr0+28*0x400+t*16+16]
    return [[((g[r]>>(7-c))&1)|(((g[8+r]>>(7-c))&1)<<1) for c in range(8)] for r in range(8)]
q=[idx_of(t) for t in (16,18,17,19)]
asm=[[q[0][r][c] for c in range(8)]+[q[1][r][c] for c in range(8)] for r in range(8)]+\
    [[q[2][r][c] for c in range(8)]+[q[3][r][c] for c in range(8)] for r in range(8)]
pair=Counter((asm[r][c],px[101+c,81+r]) for r in range(16) for c in range(16))
print(sorted(pair.items()))
PY
```
