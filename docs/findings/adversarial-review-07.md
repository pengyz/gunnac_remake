# 对抗复审 #2-R4（t151）：t148 R3 修复复审 —— 定界重做 + 注记落档

> 被审方：t148（e6af530）。审核方：verifier（t151），独立重导。
> **裁定：needs_revision**——①a/①b/③/④ 修复成立（计数假象清算 ✓、220/220 复现 ✓、注记落档 ✓、滤表钉死 ✓）；
> 但 **①c 拆分数字两头皆错**（485=345+140 不成立）与 **② 机制引用链断裂**（t150 已证文本族无码表转换）。

## 0. 发现总表

| ID | 级 | 问题 | 处理 |
|---|---|---|---|
| R5-1 | 中 | **② L4 首分歧机制命名被上游推翻**：§6.6"首分歧=渲染层（**码表转换**…t147 散射实证）"——但 `text-code-table.md`（t150，在制待审稿）以 **A 级决定性实验**（118 调用 burst↔ROM 全长对账：102/118 逐字节 ROM 直存、16/118 小值参数族）定谳"**文本 tile 直存，不存在 code→CHR 转换表**"，且 t147『83 唯一值×8 bank 散射』被重定位为**小值参数族（attrs/坐标/参数记录，非字符码表）**——§6.6 的引用链指向与自己相反的结论。"首分歧=**渲染层**"层面成立（内容层 220/220 ✓），但机制部分应改"候选/待 t150 定谳回写" | §6.6 机制句降级为候选并挂 t150 联动注 |
| R5-2 | 中 | **①c "485=345 tile+140 attr"拆分不成立**：本审精确复刻——**485 = t131 旧滤法 vs 全量回放的 tile 面分歧数**（t134 原口径，tile 面专用）；含 attr 面重算应为 **549 = 485 tile + 64 attr**（attr 既非 140 也非 485 的子项）。拆分两头皆误 | 拆分数字按实测更正（485 tile-only / 549 全窗 / attr 64）或撤回 |
| R5-3 | 低 | ⑤ AR3-C 清单重组措辞：§6.4"**掩码域（$2000 bit2 32 模式）**"把 **14 位 PPUADDR 掩码**（hi&$3F）与 **增量位**（$2000 bit2 决定 +1/+32）混为一项；t141 的"回绕"项在重组中丢失 | 两自由度分列；补回绕注 |

## 1. 通过项（独立复现）

| 项 | 声称 | 独立重导 | 判定 |
|---|---|---|---|
| ①a 计数假象清算 | 80/67/13=77/64/10+装订行 | 组构成实测 `80 = $2007×77 + $2006×2 + $2000×1`（三组抽样一致） | ✓ |
| ①b 修后复现 | 220/220 逐址精确 | 组末帧钉死+每组新鲜锁存独立回放=**220** ✓ | ✓ |
| ③ 注记落档 | §6 dated 六项（假阳性/计数假象/83 十进制/AR3-C/AR3-E/F/定界） | 全部落 `l4-visual-plan §6`，dated+原地更正文化 ✓（此前 docs/ 零改动违规已修） | ✓ |
| ④ 滤表钉死 | 组末帧时刻语义（非字典序） | 真覆盖同起点场景实测：**28 个同起点不同内容起点**，组末帧时刻与字典序选择 28/28 一致（inv 流单调 ⇒ 本集两序等价）；钉死语义正确且被真实用例覆盖 | ✓ |

## 2. ② L4 首分歧 diff-case 定界核 —— 层面对 ✓ / 机制 ✗（R5-1）

- **内容/放置层无分歧 ✓**（220/220 独立复现）⇒ 首分歧在渲染链——层面推理成立；
- 但机制名"码表转换"当前证据基础**反向**：t150 决定性实验（102/118 ROM 直存）+ t147 散射重定位（小值参数族）；
- "『glyph 层白行』说维持待定界" ✓ 保持；
- 建议 diff-case 模板补全：首个分歧点应给**帧/像素区域**坐标（目前只有层名+机制候选）。

## 3. 复现命令

```bash
python3 - <<'PY'
# ①a 组构成 / ①b 220 复现 / ①c 485 口径
import json
from collections import defaultdict, Counter
rows=[json.loads(l) for l in open('analysis/phase/gameplay-stage1-text.jsonl') if l.strip()]
g=defaultdict(list)
for e in rows: g[(e['inv'],e.get('seg',-1))].append(e)
def st(ws):
    hi=None
    for e in ws:
        if e['addr']=='2006':
            if hi is None: hi=e['val']
            else: return (hi<<8)|e['val']
T={0x2170,0x21A2,0x2289,0x22B7,0x21B8,0x2205,0x2252}
last={}
for k in g:
    s=st(g[k])
    if s in T:
        t=max(e['f'] for e in g[k])
        if s not in last or (t,k)>=last[s]: last[s]=(t,k)
def rep(keys,lo=0x2000,hi=0x23C0):
    nt={};latch=True;addr=0
    for k in sorted(keys):
        latch=True
        for e in g[k]:
            if e['addr']=='2006':
                addr=((e['val']<<8)|(addr&0xFF)) if latch else ((addr&0xFF00)|e['val']); latch=not latch
            elif e['addr']=='2007':
                if lo<=addr<hi: nt[addr]=e['val']
                addr=(addr+1)&0x3FFF
    return nt
print('①b 220:', sum(1 for v in rep([v[1] for v in last.values()]).values() if v))
old=[k for k in sorted(g) if g[k] and g[k][0]['inv'] in (0,11,14,15,16,17,18)]
a=rep(sorted(g)); o=rep(old)
print('①c 485(tile):', sum(1 for x in set(o)|set(a) if o.get(x)!=a.get(x)))
o2=rep(old,0x2000,0x2400); a2=rep(sorted(g),0x2000,0x2400)
d2=[x for x in set(o2)|set(a2) if o2.get(x)!=a2.get(x)]
print('①c 全窗:', len(d2), 'attr:', sum(1 for x in d2 if x>=0x23C0))
PY
grep -n "码表\|直存\|参数族" docs/findings/text-code-table.md | head -4   # ② 引用链反证
```

## 4. 修正建议

1. R5-1：§6.6 机制句降级（"渲染层·机制候选=码表转换/小值参数族变换，t150 定谳后回写"），diff-case 补帧/像素区域坐标；
2. R5-2：485 拆分按实测更正（485 tile-only / 549 全窗 / attr=64）；
3. R5-3：AR3-C 清单掩码/增量分列 + 回绕注。
