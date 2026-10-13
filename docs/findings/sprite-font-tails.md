# 双尾取证：sprite base 107 + 字模 tile→图形（t167）— 待审稿

> ①107 奇数基 = 8×16 对的第二半 1KB（非第 5 配置）；②字模 tile→glyph = CHR 直读位图
> （JJJS 字形实证）。retry #19 双根齐。自查纪律 §5。

## 1. sprite base 107 奇数基（①）—— **8×16 对第二半 1KB 定谳（A 级）**

**CHR 直读实证**（CHR 128K，1KB bank 单位；R0=106 = 2KB 窗 = **bank106+107**）：

| tile | bank106 侧（前 8B） | bank107 侧（前 8B） |
|---|---|---|
| 0 | `ff ff ff ff ff ff ff ff` | `47 81 34 74 e8 d3 a7 4f` |
| 1 | `ff ff ff ff ff ff fc fb` | `b8 1c 08 0f 86 43 a1 d8` |
| 5 | `9f 0f 67 e6 e6 22 23 12` | `01 01 01 11 1b 1f bf f9` |

| 裁定 | 级 |
|---|---|
| **107 = R0(106) 2KB 窗的第二 1KB**（8×16 tile 对的下半/top-bottom 对）——**非第 5 配置点** | A |
| 8×16 对解析：tile bit0 → 106 侧 / 107 侧（W42 尾收口） | A |
| MMC3 宽扫（STA $8000 前 LDA #imm 回扫）：寄存器集 = **{0,1,6,7}**（R2-R5 未现 = 共享存储形式，R9-1 注族） | A |

## 2. 字模 tile→glyph 缺口（②）—— **CHR 直读位图定谳（A 级）**

**字模页（R0=124 = bank124+125）逐 tile 直读**（tile×16 偏移，16B 位图）：

| tile | 前 8B 位图 | 字形 |
|---|---|---|
| 74 ($4A) | `06 06 06 06 06 c6 7c 00` | **J**（竖笔+钩）✓ |
| 83 ($53) | `7e c0 c0 7c 06 06 fc 00` | **S** ✓ |
| 72 ($48) | `c6 c6 c6 fe c6 c6 c6 00` | **H** ✓ |

- **菜单文本『JJJS』（74 74 74 83）= 字面 J-J-J-S** ✓✓ 字形直读成功；
- 每 tile 16B = **两半重复 8B**（8×16 对全高字母）✓ 与 W42 8×16 一致；
- **tile→glyph 映射 = `CHR[bank124 + tile]`（1KB 单位 + tile×16）**——无重映射表
  （与 N-0151 一致）；≥16B 窗口判据满足（16B 全位图）。

## 3. retry #19 双根收敛（③）

| 根 | 公式 | 几何（待测） |
|---|---|---|
| sprite 8×16 对 | tile bit0 → bank106/107 半 | ≈一半残余 |
| 字模 glyph | CHR[124+tile] 直读 | ≈另一半 |

**retry #19 参数**：①8×16 对解析（bit0 分半，R0 2KB 窗内）；②字模 glyph = CHR 直读
（bank124/125，tile×16，8×16 双半）；③既有合流（t164/t160/t154 参数）。

## 4. 定位账更新

| 族 | 根因 | 状态 |
|---|---|---|
| sprite base 107 | 8×16 对第二半（本任务） | ✅ 勾销 |
| 字模 tile→glyph | CHR 直读位图（本任务） | ✅ 勾销 |
| glyph ≈750px 总残余 | 双根几何分摊待测（retry #19） | ✅ **定界完成** |

## 5. 自查纪律记录

```bash
grep -c "N-0154\|N-0155" data/naming/ledger.jsonl   # → 1/1
grep -c "8×16 对第二" docs/findings/sprite-font-tails.md   # → ≥1
```

## 6. 复现命令

```bash
python3 - <<'PY'
d=open('rom/gunnac_cn.nes','rb').read()
chr0=0x20010
print('字模 tile74:', d[chr0+124*0x400+74*16:chr0+124*0x400+74*16+8].hex(' '))
print('游戏对 106/107:', d[chr0+106*0x400:chr0+106*0x400+8].hex(' '), d[chr0+107*0x400:chr0+107*0x400+8].hex(' '))
PY
```
