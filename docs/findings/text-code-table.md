# R1 修净 + 文本引擎码表取证（t150）— 待审稿

> 步骤 0 = R1-1..6 修净（已入 write-set-attribution.md dated 注）；主攻 = 码表取证
> （t147 F 级发现复核）。核心结论：**文本 tile 直存，无需码→CHR 转换表**——
> 『转换表』问题重定位到小值参数族（§2）。

## 1. 码→CHR 转换表取证（①②）—— **文本直存定谳（A 级）**

**决定性实验**（burst 写集 vs ROM 源逐字节对账，118 调用全测）：

| 分类 | 计数 | 证据 |
|---|---|---|
| **ROM 逐字节直存**（tile-stored） | **102/118（86%）**【R6-4 分列：79% 全长命中 + 8% 仅前 16B 命中（≥16B 窗口判据）】 | 全长/16B 序列 ROM 全等（burst0 = 77/77 典例） |
| 无全等源（需转换族） | 16/118（14%） | 写值 = **0-16 小值簇**（85 唯一值 ≈ t147『83』） |

| 裁定 | 级 |
|---|---|
| **文本 tile = ROM 直存**（写值 = 流字节 1:1）——**不存在 code→CHR 转换表**（文本族） | A |
| t147『83 唯一值×8 bank 散射』= **小值参数族**（attrs/坐标/参数记录）——转换表问题**重定位**：参数编码变换（bit5 翻转 AR2-D 族），非字符码表 | A |
| t2 charmap（PRG 0x01190/0x01830 = CPU $9190/$9830）**不在写链参与**（t120 已注）——其服务对象 = 更上游输入/菜单字库（W62） | B |

- **重试 #15 结论**：码表驱动重建**不成立**（文本族直存）；重建参数 = **ROM 直拷 + 小值族
  变换表**（§3）。

## 2. 小值参数族（转换真身候选）

16 调用写值：`0,1,2,…,16,22,24,25…`（85 唯一值）——与 AR2-D『bit5 翻转』（ac→8c/88→a8）
同族特征（值域低段 + 位变换）→ **参数记录族**（属性/坐标/模式），变换点 W60 维持。

## 3. 形状匹配假阳性注（③，t147 F2）

> 简单形状值 **1-15** 的形状匹配假阳性率高（笔画少、碰撞多）——匹配判据须排除该值域
> 或加长匹配窗口；本任务的『无全等源』判定已用 ≥16B 窗口规避。

## 4. R1-1..6 修净记录（步骤 0）

均以 dated 注入 write-set-attribution.md：R1-1（46=地址步距误读，矛盾修）/ R1-2（口径
M vs t131 实现口径 O 不一致直书）/ R1-3（『等差族』撤，Δ=8,12,16,16）/ R1-4（337 来源
=t132 AR1-4）/ R1-5（3 组→2 簇 $2170-$27FD/$9261-$9742）/ R1-6（CHR 通道降级）。

## 5. 重试 #15 参数

1. **文本 tile ROM 直拷**（无码表——102/118 直存实证）；
2. **小值参数族变换表**（bit5 翻转等位变换，W60 点定位后补）；
3. 形状匹配排除 1-15 简单形状值域（FP 防护）；
4. 既有合流（真制 1 字节头+直写 t138 / 归属轴 t130 / 口径 M t136）。

## 6. 复现命令

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
def find(s):
    for i in range(0x10,0x20010-len(s)):
        if d[i:i+len(s)]==s: return i
v=sum(1 for c in calls[:120] if len(c)>=16 and (find(bytes(c)) is not None or find(bytes(c[:16])) is not None))
print('ROM 直存:',v,'/118')
PY
```

## 7. 遗留

| # | 项 | 级 |
|---|---|---|
| W60 | 小值族 bit5 变换组装点 | C |
| W62 | charmap $9190/$9830 服务对象 | B |
