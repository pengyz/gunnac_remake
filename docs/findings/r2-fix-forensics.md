# R2 修复·取证：地址勘误裁定 + AR2-C 修净 + W47 撤回（t145）— 待审稿

> R2-1 为**裁定项**（ROM 重导双偏移对账，见 §1——不盲改，按 D3 真值处置）；
> R2-2/3/5~8 为**照改项**（dated 注）。修复≠重写。

## 1. R2-1 地址勘误 —— **ROM 重导裁定：t138 数字成立（附换算约定声明）**

**双偏移对账（burst0 = 77 写值，机器真值 D3）**：

| 候选 | 逐字节吻合 | 前 8 字节 |
|---|---|---|
| **file 0x10C0E（CPU $ABFE，t138 说）** | **77/77 ✓** | `4a 4a 4a 53 00 00 48 00` = burst0 全等 |
| file 0x10C1E（CPU $AC0E，R2 说） | 35/77 ✗ | `00 00 00 54 4a 4a 48 00` |

**独立旁证（probe 机器真值）**：t120 `$2F/$30` 写值 = **$ABFD**（=头址）✓ 与 77/77 结论自洽。

**裁定**：**头 = CPU $ABFD（file 0x10C0D）/载荷 = CPU $ABFE（file 0x10C0E）** 维持（t138 §2 无误）；
R2-1 的『真值 $AC0D/$AC0E』**系 iNES 头换算约定差（off-by-$10）**——
**换算约定声明（钉死）**：`file = prg + 0x10`（16 字节 iNES 头）；`CPU = 0xA000 + (file − 0x10 − 0x10000)`（bank8 窗）。
> 【2026-10-13 裁定注】R2-1 复现对账命令见 §5（双方数字各自复跑即辨）；约定声明入库防复发。

## 2. R2-2 AR2-C 修净（照改）

> 【2026-10-13 更正（R2-2）】t120 §6 `xxd -s 0x12BFD` **原样未动**——更正为
> **`xxd -s 0x10C0D`**（bank8 头址 $ABFD，按 §1 换算约定）；t138 §3 的『0x10C0D/0x10C1D』
> 在本约定下正确（复核方标注的『应 0x10BCD/0x10C0D』为其无头换算读法——两读法对照表入 §1 声明）；
> stream-entry-r7.md 标签半改处补全（`$ABFD` 统一 CPU 标注）。

## 3. R2-3 W47 撤回（照改）—— **槽位漂移实证**

**独立复核（五槽频次）**：`entity_y−8` 跟踪槽频次 = **9(44)/10(42)/55(41)/15(39)/54(38)**（【2026-10-13 补（R4-4）】可复现指针：数据=analysis/traces/gameplay-stage1.probe.jsonl（probe 0200-02ff）+ gameplay-stage1.frames.jsonl（player_y）；复现命令=按槽扫描 `pg[o]==py[f]-8` 计频，见 §5 命令族）
（+16(20)/49(19)…）——五槽近均匀 = **同一逻辑精灵逐帧在槽间漂移** ✓ R2-3 成立。

> 【2026-10-13 更正（R2-3）】t138『玩家本体 = 槽 9+10 双件』**撤回**（收口过强）——
> 修订为：**锚点 −8 + 双件几何（X −5/+3），槽位随帧漂移（OAM 分配轮转）**；
> W47 收口口径改为『几何 + 锚点』（非固定槽号）。

## 4. R2-5~8 低危四项（照改）

> 【2026-10-13 更正（R2-5）】t104/§1 与 ar2-fix §1 的 `SBC #$30` 引址 **A15D→A161**
> （`LDA $4FC`=A15D、`SEC`=A160、`SBC #$30`=**A161**——逐指令引址修正）。
> 【2026-10-13 更正（R2-6）】AR2-D『bit5 翻转=attr 位组装』**超证据降级**：观测=值等式
> （ac→8c/88→a8），位语义归 W60 假设（不再引 t103 结论背书）。
> 【2026-10-13 更正（R2-7）】ar2-fix §5 判据注尾残句清理（『重试 #13 参数』表补全移交行）。
> 【2026-10-13 增补（R2-8）】gotcha 素材：时序相位（NMI/f−1/预算门）+ **读自增**
> （$2007 读/写自增行为）两自由度——记录供 porter 复刻清单。

## 5. 复现命令（R2-1 双方对账 + R4-4 槽频次）

```bash
python3 - <<'PY'
import json
d=open('rom/gunnac_cn.nes','rb').read()
rows=[json.loads(l) for l in open('analysis/phase/gameplay-stage1-text.jsonl')]
calls=[];cur=None;pend=None
for e in rows:
    if e['addr']=='2006':
        if pend is None: pend=e['val']
        else: cur=[];calls.append(cur);pend=None
    elif e['addr']=='2007' and cur is not None: cur.append(e['val'])
b0=calls[0]
for name,off in (('0x10C0E/$ABFE',0x10C0E),('0x10C1E/$AC0E',0x10C1E)):
    seg=d[off:off+77]
    print(name, sum(1 for i,v in enumerate(b0) if v==seg[i]), '/77')
print('换算: file=prg+0x10; CPU=0xA000+(file-0x10-0x10000)（bank8 窗）')
PY
```

## 6. 遗留

| # | 项 | 级 |
|---|---|---|
| W61 | 换算约定统一入 gotcha（防 off-by-$10 复发） | B |
| W60 | bit5 组装点（维持假设） | C |

### R4-4 槽频次复现

```bash
python3 - <<'PY'
import json
from collections import defaultdict, Counter
evs=[json.loads(l) for l in open('analysis/traces/gameplay-stage1.probe.jsonl') if '"a"' in l]
oam=defaultdict(dict)
for e in evs:
    a=int(e['a'],16)
    if 0x200<=a<=0x2FF: oam[e['f']][a-0x200]=e['v']
py={}
for line in open('analysis/traces/gameplay-stage1.frames.jsonl'):
    r=json.loads(line); d=r['w']
    if 'player_y' in d: py[r['f']]=int(d['player_y'])
hit=Counter()
for f,pg in oam.items():
    if f in py:
        for s in range(64):
            o=s*4
            if o in pg and pg[o]!=248 and pg[o]==py[f]-8: hit[s]+=1
print(dict(sorted(hit.items(), key=lambda kv:-kv[1])))
PY
```
> 【2026-10-13 口径修正（R6-3）】上述命令为**纯 Y 匹配口径**（全槽扫描，含 HUD 固定 Y 帧），
> 与 t138 实测数（44/42/41/39/38）不同源——t138 数为**纯 Y 匹配 + t109 时点 probe 快照**
> （probe.jsonl 每次 run 覆盖重生成，不同时点数据不可比）。精确口径：纯 Y 匹配、全槽、
> 以 t109 时点 probe.jsonl 为准；如复跑请用**同版本 probe 数据**或改报纯 Y 计数（数量级 ~10³）。
> 本条数字挂『t138 实测（t109 时点数据）』口径。

## 8. gotcha 素材（R4-5，下批入库）

1. **CPU↔file 换算声明**：`file = prg + $10`（iNES 头 16B）；`CPU = 窗基 + (file − $10 − bank_prg_base)`——
   off-by-$10 家族（t149 复审方踩坑 = 最佳活例；R2-1/R4-1 同族）。
2. **『解释层缺陷伪装成数据问题』新员**：投影/换算层缺陷（反汇编投影标签错、file↔CPU 换算错）
   会被误判为 ROM 数据问题——先验换算层再疑数据（D3 但需正确的字节定位）。
