# 对抗复审 #3-R（t155）：t152 ROM 直拷击复审（含 R5 并入修复核）

> 被审方：t152（970bf0a 渲染侧 + 4424288 R5 回写）。审核方：verifier（t155），独立复算。
> **裁定：needs_revision**——①③⑤主体精确落地（R5-2 数字与本审 t151 实测逐位一致）；余 1 中 3 低（回绕目标事实错误 + 三处措辞/指针级）。

## 0. 发现总表

| ID | 级 | 问题 | 处理 |
|---|---|---|---|
| R7-1 | 中 | **R5-3 回绕目标事实错误**：§6.4(c) 写"地址回绕（**$3FFF→$3000**）"——NES PPU 14 位地址增量回绕为 **$3FFF→$0000**（$3000 系镜像区无此回绕语义） | 改 $3FFF→$0000 |
| R7-2 | 低 | ④ W45 范围澄清注缺：§6.6 已注"tile 族分离后 bank 57:22/54:4/124:4 仍散射——CHR 组织未证层"，但 W45 仍标"**关闭**"（t125）——两者并存需一行范围澄清（W45 收口=字模 bank 映射 124/126；tile→图形组织/散射=Wg 未证层） | 补 W45 范围注 |
| R7-3 | 低 | ② +2.8%（92.20→95.04）为管线台账计数（palette-used.json:231），无独立可复现比对命令指针——结构归因 ✓ 但数值未独立复测 | 挂 L4 比对复现命令 |
| R7-4 | 低 | §6.4(a)"$2000 bit0-1/2 掩码域"与 (b)"增量步长（1/32 由 bit2）"bit2 表述重叠 | (a) 限 bit0-1/NT 选择，bit2 归 (b) |

## 1. ①ROM 直拷口径复核 —— ✓

- 渲染 `tile = raw_tile + base*256` = **直接 CHR 索引** ✓ 与 t150 A 级"文本 tile=ROM 直存、无码表转换"一致；
- t147 码表假说处置合规：§6.6"t147『码表转换』散射说**降级候选**（挂 t150 联动注）"+ text-code-table"不存在 code→CHR 转换表（文本族）| A"——降级+联动+原地 dated ✓。

## 2. ②参数族分离复核 —— 结构 ✓ / 数值挂命令（R7-3）

| 项 | 声称 | 独立重算 | 判定 |
|---|---|---|---|
| ≤16 簇 | 参数字节非 tile | 可见 220 非零中 **69 个 ≤16，值=1..16 连续簇**（参数样结构 ✓） | ✓ |
| ≥128 高值族 | 非 glyph 跳过 | **32 个**（128,140,141,142,145,166-168,184,194,197,240） | ✓ |
| 中段渲染 | 17-127 | **119 个（54.1%）**，最小 22 | ✓ |
| +2.8% 归因 | 92.20→95.04 | 结构自洽（跳过 46% 非 glyph 值）；数值=台账计数未独立复测 | 部分（R7-3） |

## 3. ③内容层 220/220 保持 —— ✓

跳过仅渲染层（`continue` 在取 tile 后），nametable 值保留——独立回放**220/220** ✓。

## 4. ④bank 散射=CHR 未证层定界 —— 注 ✓ / W45 范围缺（R7-2）

§6.6"tile 族分离后 bank 57:22/54:4/124:4 仍散射——CHR 组织未证层注" ✓（分离后散射仍在=不是参数噪声、是 CHR 组织层未证）；但 W45 仍"关闭"需范围澄清。

## 5. ⑤R5 并入修复核 —— 3.5/4 落地

| 项 | 状态 |
|---|---|
| R5-1 机制命名回写 | ✓『ROM 直拷缺失 + 小值参数族变换』+ t147 降级候选 + t150 联动（与本审 t151 要求逐字对应） |
| R5-2 数字修正 | ✓ **485=旧滤法 tile 面数 / 549=485+64**（与本审 t151 实测逐位一致） |
| R5-3 AR3-C 三分离 | 三分离 ✓ 但回绕目标错（R7-1）+ bit2 重叠（R7-4） |
| R5-4 diff-case 坐标字段 | ✓（§6.7 含示例 `{layer, frame, region, cause}`） |

## 6. 复现命令

```bash
python3 - <<'PY'
# ②③：可见 220 值域分布 + 内容保持（同 t151 回放）
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
nt={};latch=True;addr=0
for k in sorted(v[1] for v in last.values()):
    latch=True
    for e in g[k]:
        if e['addr']=='2006':
            addr=((e['val']<<8)|(addr&0xFF)) if latch else ((addr&0xFF00)|e['val']); latch=not latch
        elif e['addr']=='2007':
            if 0x2000<=addr<0x23C0: nt[addr]=e['val']
            addr=(addr+1)&0x3FFF
vis={a:v for a,v in nt.items() if v}
c=Counter(vis.values())
print(len(vis), sum(n for v,n in c.items() if v<=16), sum(n for v,n in c.items() if v>=128),
      sum(n for v,n in c.items() if 16<v<128))   # 220 69 32 119
PY
grep -n "3FFF" docs/spec/l4-visual-plan.md   # R7-1 回绕目标实证
```

## 7. 修正建议

1. R7-1：$3FFF→$3000 改 $3FFF→$0000；
2. R7-2：W45 范围澄清注（字模 bank 映射收口 vs CHR 组织未证层）；
3. R7-3：+2.8% 挂 L4 比对复现命令；
4. R7-4：bit2 归 (b)。
