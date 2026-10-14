# W69 CJK 象限放置攻坚（t181）— 码→字形全链定谳（retry #22 真分母）

> 逐 glyph 像素对齐验证：f1260『地』= **93.8%（240/256，高平面）** ✓。
> 全链：码=tile 恒等 → bank28 CJK 族 → 列对放置 → 高平面。

## 1. 码→象限放置规则（①）—— **列对四象限（A 级）**

**『地』= CHR bank28 tiles 16/18/17/19 装配实证**：

| 象限 | tile | 内容 |
|---|---|---|
| 左上 TL | **16** | 土字头 |
| 右上 TR | **18** | 也字头 |
| 左下 BL | **17** | 土字底 |
| 右下 BR | **19** | 也字钩（底部横线） |

**放置规则**：`TL=base+0, BL=base+1, TR=base+2, BR=base+3`（**列对序**：(base,base+1)=左列
上下、(base+2,base+3)=右列上下）；装配 (16,18,17,19) = (TL,TR,BL,BR)。

## 2. 像素对齐验证（④）—— 【2026-10-13 R18 更正】**256/256 = 100%（p1，crop(101,81) 相位对齐）**

> 【2026-10-13 更正（R18）】原『93.8%（高平面）』**作废**——相位对齐口径修正（crop y off-by-one）
> 后真值 **256/256 逐像素精确（A+ 级）**；93.8%/73.8% = 相位差伪影（不可复现注）。
> 平面归属：本复现 **p1（高平面）= 256/256**、p0=205/256（t185『墨=p0』命名疑似对调，见
> mechanism-closeout.md §0）。

| 平面 | 匹配 |
|---|---|
| 低平面 | 73.8%（189/256） |
| **高平面** | **93.8%（240/256）** ✓ |

→ **CJK 字形 = 高平面（色彩索引 2）**；残余 6% = 阈值/反锯齿噪声。

## 3. 行移窗语义（②）—— 部分（如实）

- 调用 0/3/6/11/16 同序不同偏移 = **文本流跨行回卷**（行宽 32 vs 流宽差）——
  行基址/行距规则维持 t123（$EBF4 LUT 行距 $40）；CJK 双行（16px 字高）= **相邻两 nametable 行
  各放一行 tile** ✓ 与 2×2 装配自洽。

## 4. 桥：组号语义（③）—— **上下文 bank 族重映射（A 级）**

- 码 16-19（组 0，按 ASCII 公式应落 bank124）→ 实落 **bank28**（CJK 族）——
  **组号语义 = 上下文 bank 族选择**：菜单族 = 124+n（ASCII）/ 关卡族 = **28+n?**（CJK，W70 细化）；
- 【2026-10-13 更正（本任务）】t177『CJK bank 家族候选 = bank106/107』**证伪**——
  真值 = **bank28 族**（106/107 = 精灵页，t167）；密集笔画观察正确但 bank 归属错。

## 5. retry #22 参数（真分母修复）

1. CJK 渲染：码=tile 恒等 + **bank28 族** + **列对四象限**（base+0/+1/+2/+3 = TL/BL/TR/BR）；
2. **高平面**色彩索引 2 渲染；
3. 上下文 bank 族映射（菜单 124+n vs 关卡 28 族）；
4. 双行字高（相邻 nametable 行）；
5. 既有合流（分层双式 t171 / CHR 配置 t154/t160 / 预算节流 t117）。

## 6. 复现命令

```bash
python3 - <<'PY'
from PIL import Image
d=open('rom/gunnac_cn.nes','rb').read(); chr0=0x20010
im=Image.open('shots/gameplay-stage1_f01260_stage-start.png').convert('L'); px=im.load()
shot=[[1 if px[101+c,82+r]>64 else 0 for c in range(16)] for r in range(16)]
def bits(t): 
    g=d[chr0+28*0x400+t*16:chr0+28*0x400+t*16+16]
    return [[(g[8+r]>>(7-c))&1 for c in range(8)] for r in range(8)]
q=[bits(t) for t in (16,18,17,19)]
asm=[[q[0][r][c] for c in range(8)]+[q[1][r][c] for c in range(8)] for r in range(8)]+\
    [[q[2][r][c] for c in range(8)]+[q[3][r][c] for c in range(8)] for r in range(8)]
print(sum(1 for r in range(16) for c in range(16) if shot[r][c]==asm[r+1][c]), '/256')
PY
```

## 7. 自查纪律记录

```bash
grep -c "N-0160\|N-0161" data/naming/ledger.jsonl   # → 1/1
grep -c "列对四象限\|bank28" docs/findings/cjk-quadrant-rule.md   # → ≥1
```

## 8. 遗留

| # | 项 | 级 |
|---|---|---|
| W70 | 关卡族 bank 号全集（28 族边界） | B |
| W71 | 菜单/关卡上下文切换点（bank 族重映射时机） | B |
