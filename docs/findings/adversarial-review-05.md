# 对抗复审 #2-R2（t144）：t141 AR3 修复复审

> 被审方：t141（7512aec）。审核方：verifier（t144），独立重导（(inv,seg) 分组回放/行构成核算）。
> **裁定：needs_revision**——AR3-A 滤表修正**语义成立且流序回放即达逐址精确 220**（其自报 277/差 57 系测量 artifact）；但 **AR3-B"段写溢出"定界被证伪**（80/67/13=装订行计数假象），且**全部文档注记未落档**（只写 commit message，docs/ 零改动）。

## 0. 发现总表

| ID | 级 | 问题 | 处理 |
|---|---|---|---|
| R3-1 | **高** | **AR3-B"首分歧=放置层段写溢出 +1-3 字节"证伪**：80/67/13 = 组**总行数**（77/64/10 条 $2007 **+ 2×$2006 + 1×$2000 装订行**）——把寄存器装订行当数据写的计数假象；"277 vs 220 差 57"**不可复现**：本审流序回放（7 最后写者段 / 10 过选组）**皆=220 逐址精确**，f211/f212 同起点组内容**逐字节相同**（幂等重载）。溢出说不成立，残余定界维持**待定界** | 回 porter：撤"溢出"定界；277/57 自查回放口径（疑其 while 按帧消费行致锁存相位错位）；L4 残余根因按 diff-case 模板重新给首个分歧点 |
| R3-2 | **高** | **文档注记零落档**（`git show --stat 7512aec` = 仅 crates/+palette-used.json）：t141 声称的"假阳性清算 dated 更正/『段 1 内容精确+其余待对账』/t130 表 83=十进制 0x53 修正/AR3-C 四缺口 latent 注/AR3-E/F 措辞收窄"**全部只在 commit message**；`l4-visual-plan:193` 仍滞留"待 t141 重跑后定去留"未更新。违反 AGENTS §7（产出归宿=仓库）与 §3.6（原地 dated 更正） | 回 porter：五注按 dated 注落对应文档（l4-visual-plan/write-set-attribution/gotcha） |
| R3-3 | 低 | AR3-A 实现**过选 10 组 vs 目标 7 段**（段起点匹配：$21A2/$2289/$22B7 各有 f211(inv2)/f212(inv3) 双组）——本数据集双组内容逐字节相同（幂等）故无害，但归属钉死应取**七段最后写者**（inv=[0,3,3,3,4,4,4]，t141 已识别 ✓ 未用于过滤）；真覆盖族命中同起点时会错 | 滤表加 (inv,seg) 最后写者钉死或注记幂等前提 |
| R3-4 | 低 | AR3-C"四缺口"清单错位：t141 注写 `$2002 复位/$2005/$2000 bit2/回绕`，AR3-C 原案=**14 位掩码/bit2/锁存相位/读自增**——两清单应合并（均为真实自由度，掩码与读自增不可丢） | 合并 8 项或分列 |

## 1. ①AR3-A 滤表修正复核 —— 语义成立 ✓ / 实测全绿（220 逐址）

| 项 | t141 声称 | 独立重导 | 判定 |
|---|---|---|---|
| 滤表口径 | 段起点址匹配 + seg 归属键 | (inv,seg) 分组=9,756 组；7 起点命中 **10 组**（3 起点双组） | ✓（过选见 R3-3） |
| 七段最后写者 inv | [0,3,3,3,4,4,4] | **[0,3,3,3,4,4,4]** ✓ 复现（f209/f212×3/f218×3） | ✓ |
| 重建非零 | **277**（真值 220） | **7 段流序回放=220、10 组流序回放=220（逐址一致=真值）**；(gstart) 排序回放=213（排序 artifact） | ✗ 277 不可复现（R3-1） |
| 差 57 归因 | 段写溢出 +1-3 字节 | 80/67/13=装订行计数假象（组构成 2×$2006+1×$2000+77/64/10 实测）；差集实测≈0 | ✗（R3-1） |

→ **AR3-A 的修复本体（段起点匹配+seg 键）实测可达逐址精确**；t141 自报的残余 57 是其测量口径问题，非数据/放置层缺陷。

## 2. ②假阳性撤回合规 —— 部分

- `l4-visual-plan:193` 的 t142 假阳性更正注合规（划线留档+dated）✓，但尾注仍"待 t141 重跑后定去留"**未收尾**；
- t141 的"撤回→『段 1 内容精确+其余待对账』"新注**未落任何文档**（R3-2）。

## 3. ③AR3-B 定界复核 —— 溢出说证伪（R3-1），维持待定界

- glyph 归因撤回注 ✓（t142 落档）；但新定界（放置层溢出）不成立：见 §1 末段；
- L4 数字链（97.57→94.50→92.74）各击口径不一，**首分歧点仍未具名**——t141 撤"溢出"后应回到 diff-case 模板（帧/像素区域/层）。

## 4. ④AR3-D 真比对断言核 —— 方向 ✓ / 注记未落档

- 段 1 `[74,74,74,0x53]` + 起点 `$2170` 的层 3 断言方向正确（替代层 1 自比对）✓；
- "t130 表 `83`=十进制（=0x53）"修正是对的（t130 §2 表值为十进制），但注记未落 `write-set-attribution.md`（其表仍 `74 74 74 83…` 无标注）——R3-2。

## 5. ⑤AR3-C/E/F 注记核 —— 方向对/全部未落档

- AR3-E/F 收窄措辞 ✓ 方向（gotcha 已有 t142 的"port 消费侧"收窄）；t141 再收窄注未落档；
- AR3-C 清单错位（R3-4）；"latent"定性 ✓。

## 6. 复现命令

```bash
python3 - <<'PY'
import json
from collections import defaultdict
rows=[json.loads(l) for l in open('analysis/phase/gameplay-stage1-text.jsonl') if l.strip()]
g=defaultdict(list)
for e in rows: g[(e['inv'],e.get('seg'))].append(e)
def st(ws):
    hi=None
    for e in ws:
        if e['addr']=='2006':
            if hi is None: hi=e['val']
            else: return (hi<<8)|e['val']
def replay(keys):
    nt={};latch=True;addr=0
    for k in keys:
        for e in g[k]:
            if e['addr']=='2006':
                addr=((e['val']<<8)|(addr&0xFF)) if latch else ((addr&0xFF00)|e['val']); latch=not latch
            elif e['addr']=='2007':
                if 0x2000<=addr<0x23C0: nt[addr]=e['val']
                addr=(addr+1)&0x3FFF
    return nt
T={0x2170,0x21A2,0x2289,0x22B7,0x21B8,0x2205,0x2252}
m=sorted([k for k in g if st(g[k]) in T]); last={}
for k in m: last[st(g[k])]=k
print('最后写者 inv:', [g[k][0]['inv'] for k in sorted(last.values())])   # [0,3,3,3,4,4,4]
print('7 段流序非零:', sum(1 for v in replay(sorted(last.values())).values() if v))  # 220
print('10 组流序非零:', sum(1 for v in replay(m).values() if v))                      # 220
for k in m[:1]+m[-2:-1]:
    from collections import Counter; print(k, dict(Counter(e['addr'] for e in g[k])))  # 80/67/13=装订行
PY
git show --stat 7512aec | tail -5    # docs/ 零改动实证
```

## 7. 修正建议

1. R3-1：撤"段写溢出"定界；自查 277 的回放口径（重点：按帧 while 消费行是否跨组错相）；修复后 L4 残余重新给 diff-case 首分歧点；
2. R3-2：五注按 dated 注落档（l4-visual-plan:193 尾注收尾、write-set-attribution 83 十进制注、gotcha AR3-C/E/F 注、t131 撤回注）；
3. R3-3：滤表钉死最后写者（或注记幂等前提）；
4. R3-4：AR3-C 缺口清单合并（掩码/读自增勿丢）。
