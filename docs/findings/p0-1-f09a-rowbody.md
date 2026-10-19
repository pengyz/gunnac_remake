# P0-1 族①收官：F09A 行体流接入 + 0 码 preserve 分治

> 【2026-10-20 07:25】实施=双 mimo 会话接力（首发 38min 崩、续作崩于评估步，代码由两会话
> 共同完成 +337/−7）；**验证环=Lead 自跑**（结果优先）；独立复核随后。

## 1. 实现（按 G3 定谳结构，render_frame.rs +337/−7）

- **F09A 行体流落位**（`is_rowbody_pc`/`rowbody_write_cell`）：`$2007` 写点 pc∈$F0A0-$F0CA
  （rowbody 族，seed bank15:$F08B-$F0DA）直落 nametable，**0=擦写必须生效**（行体流=真机
  `$2007` 上传流、0=真机擦除语义，不走 0 码 preserve）；
- **$2005/$2006/$2007 流接入**（PPU 锁存/自增语义；探头重叠注册按家族去重——G3 §2-1 坑注）；
- **0 码 preserve 分治**（G3 §6-2 口径：来源/时序分治；44 格实证面、73 格无本地作用口径注）；
- 测试 **73 全绿**（65+8 新：rowbody 落位/擦写/去重/分治边界）。

## 2. 验证环（Lead 自跑；before=上轮 v 分型态同输入；两跑 sha 全同）

| 样本 | before | after | Δ |
|---|---:|---:|---:|
| gameplay f1260 | 5,292 | **1,265** | **−4,027** |
| gs5 f3450 | 1,524 | **1,524** | 0（PNG 逐字节同）|
| stg03 f4600 | 58,159 | **58,159** | 0（逐字节同）|

**累计链**（gameplay）：legacy 基线 3,742（旧读流）→ 3,247（当前输入）→ 5,202（事件序回放）→
5,292（v 分型）→ **1,265（F09A 收官）** ⇒ vs 当前输入基线 **−1,982**、vs 旧读流基线 **−2,477**；
gs5：2,355→1,524（事件序）→1,524（收官保持）⇒ **−831**；stg03 全程逐字节不变（回归锚）。

## 3. 与 G3 定谳对账

- 143 格（F09A 行体流缺失）+ 117 格（preserve 分类口径）预期全修复——gameplay 实际超预期
  （−4,027 > 1,955+90）：行体流接入同时修掉了 G3 未单独归类的陈旧码面（text 留 f209-218
  陈旧码一并被行体擦写覆盖）；逐格归因细化留后续；
- GPFMODE 取证开关保留（缺省=新行为）。

## 4. 未证残留

- attr 面/截图时点疑点（G3 §7-3/§7-4）联动未闭；F1EB 命令流未接入渲染（当前行体流=probe
  $2007 流直读，F1EB 为消费语义旁证）；MODELGEO 几何（2 行×16 列翻案后）需按新几何重测。

## 5. 复现

```
cargo test -p game-desktop   # 73 全绿
target/p0-0/build/debug/render-frame.exe 1260 gameplay-stage1_f01260 gameplay-stage1 --out-dir <dir> --log-path <log>
# diff_visual.compare(expected, actual) → len(samples)
```


## 【2026-10-20 07:45 dated 注（复核 32bda583 跟进）】
- §5 复现命令补 env：`REPO_ROOT=D:\workspace\gunnac_remake`（render-frame 无该 env 时按 exe
  上溯 3 级定仓根=target/p0-0，`--out-dir` 越界 fail-closed；另预存 quirk：越界退出码实测 1
  vs 注释 2，非本 diff 引入，登记待修）。
- 复核未覆盖面（Lead 自跑口径）：cargo 73 全绿、gs5 1524（PNG 逐字节同 245cbedc）、stg03 58159
  （246dd571）——均为"不变"断言（回归锚），两跑 6/6 同 sha 已自跑；闭合复渲随下轮补。
