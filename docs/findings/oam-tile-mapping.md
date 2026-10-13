# OAM tile→bitmap 映射取证（t209）— W73' 唯一单点收口（待审稿）

> ①标签族映射规则 = **连续对（T&$FE, +1）@ 标签 bank 族（96/98）**，『田』128/128 逐像素；
> ②128+ 族反例注（t208 留档）；③→ t210 单点修复输入。

## 1. 标签族 sprite tile→CHR bitmap 映射规则（①）—— **连续对 @ bank96/98（A 级）**

**『田』glyph 逐像素基准**（f1260 x70-77×y17-32，OAM Y=16→屏 17（Y+1 惯例 N-0168）、
槽 63：tile=1、attr=00、X=70）：

| 映射候选 | 匹配 |
|---|---|
| **连续对（bank98, tiles 0/1）** | **128/128 = 100%** ✓ |
| **连续对（bank96, tiles 0/1）** | **128/128 = 100%** ✓（tile 0/1 与 bank98 **完全重复**，全 bank 685/1024 部分同） |
| 106/107 双 bank 对（t167 机制） | 无命中（<120） |

**映射规则定谳**：

```
OAM tile T（8×16）→ 对基 = T & $FE
顶 = CHR[标签 bank 族][对基]、底 = CHR[标签 bank 族][对基+1]   （同 bank 连续对）
标签 bank 族 = bank96/98（2KB 对配置；tile 0/1 双重复，取哪侧等价）
```

> 【2026-10-13 修订（t214 裁定，N-0172 修订）】**族选择器真身 = T bit0 分窗**：
> **odd tile → bank96/97 窗、even tile → bank28/29 窗** + 对基 T&$FE 连续对——替换
> 『四族分派』表述。**8/8 精确 128/128 证据链**：田(1)/HUD(5,23)/自机(65)/杂项(79) = **odd 族**
> → 96/97；CJK(16,18,20,22,12) = **even 族** → 28/29；**106/107 与 bank30 零命中**。
> 【2026-10-13 降级注】『游戏 106/107』= **数据模型事实**（R0=$6A BG/图形窗，t154 probe）
> 非渲染输出源——**渲染输出 vs 数据模型混层教训**（W 项族注）。

- 与 t188 CJK 族（bank28 连续对）**同机制、异 bank 族**——族 = 上下文配置决定（t154/t160 族映射续）；
- 索引图（0/1/3 分布）× palette（CJK 槽 1=暗青/3=白）100% 对应 ✓（t199/t204 机制一致）。

## 2. 128+ 族 bank28 反例核对（②）—— 注（t208 留档）

128+ 族（tile ≥128）**非 bank28 根**（t208 已留档）——本单点 = **标签族（低 tile 号）**，
与 128+ 族分属不同族；映射规则按族分派（§1 公式 + 族选择器）。

## 3. t210 单点修复输入（③）

1. 标签族 sprite：`对基 = T&$FE`，顶/底 = 连续 tile @ bank96/98（**勿用 106/107 对**）；
2. ~~族选择器：上下文配置 → {标签 96/98、CJK 28、图块 20-23、游戏 106/107}~~【2026-10-13 修订】**T bit0 分窗**：odd→96/97、even→28/29 + 对基 T&$FE；待证 W：图块 20-23 族、弹幕 251/253（attr $52 vflip）；
3. 索引×palette（t204 二维表）+ Y+1 惯例（N-0168）。
→ 预期残余 ≈1.85% 单点清零 → L4 彻底收官。

## 4. 自查纪律记录

```bash
grep -c "N-0171\|N-0172" data/naming/ledger.jsonl   # → 1/1
grep -c "128/128\|bank96" docs/findings/oam-tile-mapping.md   # → ≥1
grep -n "bank28\|106/107" docs/findings/oam-tile-mapping.md   # 同文矛盾扫（应全为对比/反例语境）
```

## 5. 复现命令

```bash
python3 - <<'PY'
from PIL import Image
d=open('rom/gunnac_cn.nes','rb').read(); chr0=0x20010
im=Image.open('shots/gameplay-stage1_f01260_stage-start.png').convert('RGB'); px=im.load()
def cls(p): return 0 if p==(0,0,0) else 3 if p==(255,254,255) else 1 if p==(0,64,77) else 2
sp=[[cls(px[70+c,17+r]) for c in range(8)] for r in range(16)]
def idx_of(bank,t):
    g=d[chr0+bank*0x400+t*16:chr0+bank*0x400+t*16+16]
    return [[((g[r]>>(7-c))&1)|(((g[8+r]>>(7-c))&1)<<1) for c in range(8)] for r in range(8)]
pair=idx_of(98,0)+idx_of(98,1)
print(sum(1 for r in range(16) for c in range(8) if pair[r][c]==sp[r][c]),'/128')
PY
```
