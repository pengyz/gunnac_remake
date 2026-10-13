# 对抗复审 #3（t153）：t150 码表终谳 + R1/R4 修净复审

> 被审方：t150（`text-code-table.md` + write-set-attribution.md dated 注）。审核方：verifier（t153），独立复算。
> **裁定：needs_revision**——①码表终谳核心成立（102/118 逐位复现）；④gotcha 素材准确；但 **R1-6"已修"未落地** + R4-2/R1-3 半修 + R4-4 数字仍不可复现（"声称 vs 现实"类 3 处）。

## 0. 发现总表

| ID | 级 | 问题 | 处理 |
|---|---|---|---|
| R6-1 | 中 | **R1-6 声称"CHR 通道降级"未落地**：t150 §4 记"R1-6（CHR 通道降级）"，但 `write-set-attribution.md:77` 仍以事实陈述"＞$3F 高字节段 = **CHR 图样写入通道**（非 nametable）"且**无降级注**（W60/W62 均不覆盖此句） | 该句补 dated 降级（通道语义未证：CHR-ROM 写 no-op，落区事实保留） |
| R6-2 | 低 | **R4-2/R1-3 半修**：①`ar2-fix-forensics.md:42` bit5 等式"= **属性位（bit5 优先级）翻转**"原地未降级（text-code-table 换新解释"参数编码变换"仍 A 级陈述——解释换了、强度没降）；②"等差族"（`write-set-attribution.md:38`）字面未撤（W58 降级注只点名"属性行/数字段"归属标签，未点名 Δ=8,12,16,16 非等差） | 各补一句点名 dated 注 |
| R6-3 | 低 | **R4-4 数字仍不可复现**：指针已给（probe 0200-02ff + frames player_y ✓），但按其命令 `pg[o]==py[f]-8` 复跑 = **{61/62/63:1182, 55:918, 1/2/3:915, 54:915, …}**（全 Y 匹配），与 44/42/41/39/38 差两个量级——命令缺 X 过滤/帧窗条件 | 命令补全过滤条件或数字改挂"见附命令精确口径" |
| R6-4 | 低 | ①"86% ROM 直存"含 **9 例仅前 16B 命中**（严口径全长=93/118=79%）——表头已披露"全长/16B"两级判据（诚实 ✓），建议数字分列 79%+8% | 表头分列（可选） |

## 1. ①码表终谳复核 —— 核心 ✓

| 项 | t150 声称 | 独立重算 | 判定 |
|---|---|---|---|
| 直存率 | 102/118（86%） | **102/118**（t150 口径原样复跑逐位一致）；严口径拆分：**全长 93 + 仅前 16B 9 + 全无 16** | ✓（R6-4 分列建议） |
| 全无族 | 16 调用/85 唯一值/0-16 簇 | **16 / 85** ✓；值域 0-16 簇+22,24,25,29,…（其 §2 已列扩展值）；长度 77×14+64×2 | ✓ |
| 小值参数族定性 | attrs/坐标/参数记录（候选） | 定性合理且**标"候选"、W60 开放** ✓（未证不发明合规） | ✓ |
| charmap 不在写链 | W62（B 级，服务对象待证） | 与直存结论自洽（文本族 ROM 直拷 ⇒ 写链无转换表）；W62 开放诚实 | ✓ |
| 形状匹配 FP 注 | 1-15 简单形状值域排除 | 方法论注合理（≥16B 窗口规避） | ✓ |

## 2. ②R1-1..6 修净核

| 项 | 状态 |
|---|---|
| R1-1（46=地址步距+矛盾修） | ✓ dated 注（line 11） |
| R1-2（口径 M vs O 不一致直书） | ✓ dated 注（line 62） |
| R1-3（"等差族"撤） | **半**（W58 降级注在，"等差族"字面未点名——R6-2） |
| R1-4（337 来源=t132 AR1-4） | ✓ dated 注（line 56） |
| R1-5（3 组→2 簇） | ✓ dated 注（line 47，实测值 $2170-$27FD/$9261-$9742 与本审一致） |
| R1-6（CHR 通道降级） | **✗ 未落地**（R6-1） |

## 3. ③R4 追加核

| 项 | 状态 |
|---|---|
| R4-1（$ABCD→$ABFD 两处 + 笔误注） | ✓（ar2-fix:37/58 已改并注"原标 $ABCD 系 t140 旧错传播笔误"） |
| R4-2（bit5 降级） | **半**（R6-2：解释替换但未降级） |
| R4-3（残句） | ✓（residual-pixel-ledger:60 dated 删注） |
| R4-4（五槽频次指针） | **半**（指针 ✓ / 命令复跑不达数——R6-3） |

## 4. ④gotcha 素材两条核 —— ✓ 准确

- **CPU↔file 换算声明**（r2-fix-forensics §6.1）：`file = prg + $10`、`CPU = 窗基 + (file − $10 − bank_prg_base)` ✓ 公式正确；"t149 复审方踩坑=最佳活例；R2-1/R4-1 同族" ✓ 表述准确（我方撤回即活例，无夸大）；
- **解释层伪装家族**：已入 gotcha（"观测/解释层的缺陷伪装成数据问题"，与时间轴网眼/地址轴盲区同族注）✓ 准确；建议合并入库时保留"off-by-$10 家族"独立一行（防复发价值最高）。

## 5. 复现命令

```bash
python3 - <<'PY'
import json
from collections import Counter
d=open('rom/gunnac_cn.nes','rb').read()
rows=[json.loads(l) for l in open('analysis/phase/gameplay-stage1-text.jsonl')]
calls=[];cur=None;pend=None
for e in rows:
    if e['addr']=='2006':
        if pend is None: pend=e['val']
        else: cur=[];calls.append(cur);pend=None
    elif e['addr']=='2007' and cur is not None: cur.append(e['val'])
def find(s):
    for i in range(0x10,0x20010-len(s)):
        if d[i:i+len(s)]==s: return i
sel=[c for c in calls[:120] if len(c)>=16]
print('t150 口径:', sum(1 for c in sel if find(bytes(c)) or find(bytes(c[:16]))), '/118')  # 102
print('严口径:', sum(1 for c in sel if find(bytes(c)) is not None), '全长 /',
      sum(1 for c in sel if find(bytes(c)) is None and find(bytes(c[:16]))), '仅16B')     # 93 / 9
PY
# R4-4 复跑（缺 X 过滤=量级差 25×，R6-3）
python3 - <<'PY'
import json
from collections import Counter
oam={};py={}
for l in open('analysis/traces/gameplay-stage1.probe.jsonl'):
    e=json.loads(l);a=e.get('a','')
    if a and a.lower().startswith('02'): oam.setdefault(e['f'],{})[int(a,16)]=e['v']
for l in open('analysis/traces/gameplay-stage1.frames.jsonl'):
    e=json.loads(l);w=e.get('w',{})
    if 'player_y' in w: py[e['f']]=w['player_y']
s=Counter()
for f,pg in oam.items():
    if f in py:
        t=(py[f]-8)&0xFF
        for o in range(0,256,4):
            if pg.get(0x200+o)==t: s[o//4]+=1
print(dict(s.most_common(6)))
PY
```

## 6. 修正建议

1. R6-1：write-set-attribution.md:77 补 CHR 通道降级注；
2. R6-2：ar2-fix:42 bit5 降级 + write-set-attribution:38"等差族"点名撤；
3. R6-3：五槽频次命令补 X 过滤/帧窗（或数字改口径注）；
4. R6-4：直存率分列 79% 全长 / 8% 仅前 16B（可选）。
