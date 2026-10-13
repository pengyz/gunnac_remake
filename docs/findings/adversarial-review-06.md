# 对抗复审 #2-R3（t149）：t145 R2 修复复审 —— R2-1 换算约定裁定复跑

> 被审方：t145（`docs/findings/r2-fix-forensics.md` + 各 dated 注）。审核方：verifier（t149），ROM 真值复跑裁定。
> **裁定：needs_revision**（核心 R2-1 **改判：t138 成立、本审撤回**；修复面大项落地，余 3 处措辞/指针级残留）。

## 0. R2-1 裁定（核心，改判）—— **t138 数字成立 + 换算约定钉死 ✓**

ROM 真值复跑（双方读法独立重跑，不采信自述）：

| 候选 | 逐字节吻合 | 判定 |
|---|---|---|
| **file 0x10C0E = CPU $ABFE（t138 说）** | **77/77** ✓ | **成立** |
| file 0x10C1E = CPU $AC0E（我 R2-1 说） | 35/77 ✗ | 证伪 |

- **换算约定钉死成立**：iNES 头 16B（`4e 45 53 1a`，PRG=128K）⇒ `file = prg + $10`；`CPU = $A000 + (file − $10 − $10000)`（bank8 窗）。
- 头字节 file 0x10C0D = **00** = CPU $ABFD ✓；probe `$2F/$30=$ABFD` 旁证自洽 ✓。
- **我方 R2-1 撤回**（已按 AGENTS §3.6 原地更正 `adversarial-review-04.md`，含连带作废"$ABCD/$AC0D"标注）。
- 四分类：**投影/换算层缺陷（审核方口径 off-by-$10）**——与 gotcha『解释层缺陷伪装成数据问题』同族；
  建议 gotcha"何时使用"增一行：**CPU↔file 换算必须显式声明（iNES 头）**，防复发（本条正是活例）。
- t145 的裁定流程规范 ✓（双偏移对账 + probe 旁证 + 约定声明入库，"不盲改、按 D3 真值处置"）。

## 1. 发现总表（残留，全措辞级）

| ID | 级 | 问题 | 处理 |
|---|---|---|---|
| R4-1 | 低 | `ar2-fix-forensics.md:37/58` 的 CPU 标签 **"$ABCD/$AC0D" 仍错**（应 **$ABFD**/$AC0D；file 0x10C0D/0x10C1D 在约定下正确）——系我旧错标签传播，t145 §2 只补全了 stream-entry 的标签 | ar2-fix §3/§6 标签改 $ABFD |
| R4-2 | 低 | **R2-6 半修**：bit5 降级注只落 `r2-fix-forensics.md §4`，`ar2-fix-forensics.md §4` 原文"= **属性位（bit5 优先级）翻转**"等式未动（原地更正缺失，AGENTS §3.6） | 原文补 dated 降级注 |
| R4-3 | 低 | **R2-7 未修净**：`residual-pixel-ledger.md:59` 残句"…全族勾销"随之不成立…**判据达成 ✓（每像素可归因）**"仍在（t145 清的是自己 §5 的另一处残句，非我点名处） | 删该残句 |
| R4-4 | 低 | R2-3 五槽频次 **44/42/41/39/38 不可复现于工作区快照**（ram.hex 每 40 帧/60 文件，任一槽计数上限<44；本审两口径复算=玩家窗 54/55(14) 主导、全快照 u8-wrap 61/62/63(29) 主导）——结论（槽位漂移）成立但数字需给逐帧数据载体指针（N-0145 注 ✓ 已入台账） | 补数据指针（捕获文件/口径） |

## 2. 修复面核对（通过项）

| 项 | 状态 |
|---|---|
| R2-2 三处 dated 注（t120 §6→0x10C0D / stream-entry 标签补全 / t138 §3 偏移在约定下正确） | ✓（R4-1 标签残留除外） |
| R2-3 撤回注（槽 9+10→几何+锚点+漂移；W47 口径改） | ✓ 结论正确（R4-4 数字指针） |
| R2-5 A15D→A161 逐指令引址 dated 注 | ✓ |
| R2-8 gotcha 四自由度补全（掩码/自增量/**时序相位/读自增**） | ✓（t146+t145 增补，与 AR3-D 请求合流） |
| R2-4 影响面划界 | ✓（t146 反向修正，非本轮被审） |

## 3. 复现命令

```bash
python3 - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read()
print(rom[:4], 'PRG', rom[4]*16, 'KB')                 # NES\x1a PRG 128KB
import json
rows=[json.loads(l) for l in open('analysis/phase/gameplay-stage1-text.jsonl') if l.strip()]
calls=[];pend=None;cur=None
for e in rows:
    if e['addr']=='2006':
        if pend is None: pend=e['val']
        else: cur=[];calls.append(cur);pend=None
    elif e['addr']=='2007' and cur is not None: cur.append(e['val'])
pay=bytes(calls[0])
print('A file 0x10C0E=$ABFE:', sum(pay[i]==rom[0x10C0E+i] for i in range(77)), '/77')
print('B file 0x10C1E=$AC0E:', sum(pay[i]==rom[0x10C1E+i] for i in range(77)), '/77')
print('头 file 0x10C0D=$ABFD =', hex(rom[0x10C0D]))
PY
```

## 4. 修正建议

1. R4-1/2/3：三处措辞/标签/残句小修（同文件面一次过）；
2. R4-4：五槽频次补数据载体指针；
3. gotcha 增"CPU↔file 换算声明"行（本审 R2-1 撤回即活例）。
