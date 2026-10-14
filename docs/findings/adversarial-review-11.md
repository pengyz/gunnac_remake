# 对抗复审 #5（t161）：t159 知识库批量（gotcha 转正 + 两新条）复审

> 被审方：t159（79e6cbf）。审核方：verifier（t161），公式验算/命令复跑/引用核对。
> **裁定：pass（批量收口）**——①升级三前置齐备合规；②换算条公式与三件套准确；③自查 pattern 方法与案例引用准确；④解释层家族四员封口边界准确；⑤R8-1..3 逐字修净+knowledge-format 17 条合规。余 2 条琐碎观察（不入 findings）。

## 0. 观察（琐碎，不阻塞）

- O1：t159 本批未留正式 grep 自查记录段（t154 §5 那种表）——刚立的 pattern 要求"grep 目标行"，本批为新条目新增（非修复类）豁免合理，但下批修复类应 dogfood；
- O2：cpu-file-offset 条"险些误判"叙述压缩（实际是 t140 已发 R2-1 误判 → t149 撤回）——同条目下句已准确写"整行撤回"，可不改。

## 1. ①gotcha-register-address 升级前置核 —— ✓ 合规

| 前置 | 状态 | 核验 |
|---|---|---|
| 四自由度注（掩码/自增量/时序相位/读自增） | ✓ | t146/t148 落地，正文四项齐全（AR3-D 请求逐字对应） |
| 影响面收窄（port 消费侧） | ✓ | AR3-E 收窄注在（t142） |
| 链 pass | ✓ | **t158 pass（adversarial-review-10）** 由本审前轮给出 |
| 升级注形式 | ✓ | 【t159 升级转正】~~候选注~~ 正式条目 + status: confirmed + 历史注留档不删（§3.6 合规） |

## 2. ②gotcha-cpu-file-offset-conversion 新条核 —— ✓ 准确

| 项 | 核验 | 判定 |
|---|---|---|
| 公式 `file = prg + $10` | ✓（iNES 头 16B，多轮实测） | ✓ |
| bank2 化简 `file = cpu − 0x3FF0` | 验算：cpu−$8000+2×$2000+$10 = cpu−$3FF0 ✓ | ✓ |
| 双偏移对账三件套（读法 A/B+头字节+probe） | 与 t145 裁定流程一致；复跑读法 A 77/77、B 35/77、头 00 ✓（多轮已验） | ✓ |
| t149 活例表述 | "off-by-$10…t140 R2-1 整行撤回（错在换算层不在数据）" ✓ 无夸大 | ✓ |
| 家族互链 | ✓ 指向 register-address 条 | ✓ |

## 3. ③pattern-repair-selfcheck-grep 新条核 —— ✓ 准确

- 方法两步（grep 目标行 + 新引入事实复核）✓ 纪律可执行、判据清晰；
- 案例引用逐条核对：**三轮"声称已修原地未动"** ✓（t144 R3-2 docs 零改动 → t149 R4-2/R4-3 半修 → t153 R6-1 未落地）；**R7-1"修复引入新错"** ✓（t155 回绕目标 $3FFF→$3000）；**t154 §5 正面样板 + t158 复审验证** ✓——引用无一失实；
- 与 false-green-idle-assertions 的边界（测试层 vs 文档/编辑层）✓ 划分清晰。

## 4. ④解释层缺陷家族四员封口核 —— ✓ 边界准确

| 员 | 载体 | 伪装对象 | 边界 |
|---|---|---|---|
| 时间轴网眼 | gotcha-observation-granularity | 观测粒度造不变量 | ✓ |
| 地址轴盲区 | gotcha-stack-page-game-var-watch | watch 覆盖缺口 | ✓ |
| 协议未解释 | gotcha-register-address（本条） | 协议时序流≠落点 | ✓ |
| 换算层错 | gotcha-cpu-file-offset（t159 新员） | CPU↔file 整体偏移 | ✓ |

四员边界互斥且穷尽当前已知解释层缺陷面；家族句"观测/解释层的缺陷伪装成数据问题"统领 ✓。

## 5. ⑤R8-1..3 修净 + 合规核 —— ✓ 全落地

| 项 | 落点（chr-org-timing.md dated 注） | 与本审 t158 原文对照 |
|---|---|---|
| R8-1 | "12×5=60 ≠ 43；5× 仅 7 簇成立；14=12 逻辑+2 溢出合并口径" | ✓ 逐字对应 |
| R8-2 | "$8FA8 实为 `INC $1A9` 完整指令起点（入口方向不变）" | ✓ 逐字对应 |
| R8-3 | "本 run 仅 R0 被写（$8000×3 全 0 + $8001×40 全 $6A）" | ✓ 与我实测逐位一致 |

- frontmatter/knowledge-format：**17 条全合规**（gates check [PASS] knowledge-format）✓；
- grep 自查：本批新条目类豁免记录（O1 观察）；R8 修净带新事实复核（43 事件复现/指令起点）✓。

## 6. 复现命令

```bash
python3 gates/cli.py check 2>&1 | grep knowledge-format    # 17 条合规
grep -n "R8-1\|R8-2\|R8-3" docs/findings/chr-org-timing.md # 修净三注
python3 - <<'PY'                                            # ② 化简式验算
print(hex(0x922B - 0x3FF0))                                  # bank2 场景 file 偏移
PY
grep -n "四员\|家族\|同族" docs/knowledge/gotcha-register-address-is-not-nametable-target.md
```
