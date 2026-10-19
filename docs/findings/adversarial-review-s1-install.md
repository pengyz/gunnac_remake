# 对抗复核：S1 采集裁定 + §7.7 安装方案（task-154/165）— task-169

> 【2026-10-19】复核执行说明：原指派的独立复核 turn 因基建故障崩溃（2 turn 无收尾），由 **Lead 亲自
> 独立重导全部数字**（实施方=l4-analyst ≠ 复核方=Lead）。以下命令与输出为自跑复现。

## VERDICT: **pass**（附 F2 排序注记；命名收口已另行处理）

## 独立复算（Lead 自跑）

| 声称 | 复算命令 | 结果 |
|---|---|---|
| 4 blob 零命中（`14802bcf/19b88b1f/0f8e1169/f319f1fe`） | 扫 shots/ 全量 png sha256 前 8 位比对 | dense300 **406** + dense100 **1204** + C1 26 + C1-dense 133 = **1769 张全零命中**（报告口径 2517 含其它 GUI 场景同判零）✓ ⇒ **4 对 blocked:no-source 终态成立** |
| 两跑确定性 | sha256(`analysis/traces/tas-stg08-direct-dense.frames.jsonl`) | `2c7962ec97c66107`（=C3/300f 版）✓ |
| 7 锚 sha | 逐锚扫 dense/dense100 png | f27036 `6fcebbc761ff` / f27701 `d3113aab429b` / f27731+f27734 `788d6984576b` / f28000 `d1bea1323e82` / f29000 `a6ae0d537fc0` / f30000 `11b703ce61a5` **7/7 PASS** ✓ |
| 四联/#17/#18 来源 | 扫 stg08-timeline shots | `62a8ef06c4b9` **64 帧**（f16500–33900）✓；f14600=`ddb322ac5702`(#17) ✓、f16000=`2824cb134987`(#18) ✓ |
| #14 frame-mismatch 定性 | expected 与 shot sha | `oracle/screens/expected/l4_stg02-start.png`=**788d6984576b**（旧黑屏未动，替换未执行 ✓）；`shots/gs5-stg01-clear-anchor_f03450_stg02-start.png`=**44e7800dac89**（1411B 真实屏）✓ |

## findings

- **证据不足（无）**：全部声称数字可复算。
- **实现错误（P1-F2 排序注记）**：安装方案的 `--validate` 转绿依赖 task-165 执行（expected `44e7800d` 安装），
  当前 `--validate` FAIL（registry_ok=false，{title,stg02} 重复组无 mapping 覆盖）属 **task-165↔166 排序**问题
  ——安装执行后须重生成 manifest 并复跑 `--validate` 转 PASS 才算收卡（已写入安装流程 step 0）。
- **规格缺口（P2，已随收口修）**：三文档三种命名（gs5-stg01-clear-anchor/gs5-anchor-render/gs5-anchor）
  ⇒ 2026-10-19 Lead 单点收口为 **`gs5-anchor-render:3450:stg02-start`**（render-ready overlay），
  历史 shot 磁盘实名 `gs5-stg01-clear-anchor_f03450_stg02-start.png` 保留供复现（install-plan 收口注）。
- **平台差异（无）**。

## 结论

S1 双网格负证据、确定性、锚点、来源帧、#14 定性全部独立复算一致；安装方案与 Lead 裁定（Option B）一致、
未执行 tracked 变更。**verdict = pass**（F2 为排序依赖，已在安装流程收口）。
