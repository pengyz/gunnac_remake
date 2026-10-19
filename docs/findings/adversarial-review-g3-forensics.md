# 对抗复核：族① gameplay 回退悬案终审（G3 取证）——adversarial-review-g3-forensics

> 2026-01-20 06:2x | verifier（证伪立场：不采信被审方自述，独立重跑/自写脚本 `target/p0-1-verify/`）。
> 被审：[p0-1-gameplay-regression-forensics.md](p0-1-gameplay-regression-forensics.md) + `render_frame.rs`（GPFMODE 门控，工作树未提交）。未 git 操作；未跑 gates check/status/visual。

## 0. VERDICT：**pass**（核心数字与翻案证据链全部独立复现；3 条低/info findings 不阻断，建议 dated 更正）

- **三形态数字（claim 1）✓**：自跑复渲——①5,292(`efc5fb2dbc33`)/②3,337(`054d7b9be21a`)/③3,247(`0f06d4c4cd12`)，与声称逐字同；③≡③' 旧 binary `render-frame-before.exe` **逐字节同**；两两像素差 1,955/2,045/90 ✓。**集合级闭合**：E(①^exp)=A(③^exp)∪C(①^②)∪D(②^③) 且两两不交 ⇒ `5,292=3,247+1,955+90` 逐像素集合成立（非仅算术恒等）。
- **门控与缺省零变化（claim 5）✓**：源码缺省=1（`gpf_mode:1` + env `unwrap_or(1)`；mode2/3 仅改 `end_frame` patch/置位快照分支）；缺省 env 跑（v0）与 `GPFMODE=1` 逐字节同；diff 仅 GPFMODE 面（ENV_KNOBS+1、单测+1）；两跑确定性 ✓（v1==v1b）；`cargo test -p game-desktop` **65/65 绿**（含 `gpfmode_snapshot_discipline_forms`）。
- **真因 A（claim 2）✓**：260 错格分类 **143/117/0** 独立复现；4 例格 (0,448)/(0,465)/(0,576)/(0,704) 帧/格/值逐项 MATCH（如 (0,448)=truth(0,rowbody,1260)/text(74,f218)）；零写 **356,379**/非零 **948** 在 NT 瓦面（rel%$400<$3C0）口径精确复现。
- **G3 闭合（claim 3）✓**：去重双法互证（k 抽 1 ≡ 全键塌缩，21,247 事件逐条相等）；rowbody **82 run×16B=1,312 全零**、**41/41 burst=(16B,16B,+$20)**、帧=[1204-1207,1236/1237,1244/1245,1252/1253,1260] 清屏扫掠；palbody **70 run×32B@$3F00** ↔ `$0160` 置位 **70:70**（全 v=3）；填充串 **68/101** 全命中 F1EB 族（f181→f182@`$2000`、f182→f183@`$204D`、f183→f184@`$209A`，逐帧 +$4D）；F1EB 命令结构 seed 核={cnt,addr_hi,addr_lo,data×cnt}（F157 取 cnt→F1EB→F1DC `DEC $21`）N+1 NMI ✓。
- **t93 翻案（claim 4）✓**：seed F08B-F0DA 步进=X+=8 双连发+`AND #$F` 循环+lo+=$20+`CPX #$20` 收 ⇒ **2 行×16 列**成立、"4 行×8 列"不成立；attr=`($8F|3):$90` **无 +$0F**（F04B-F057 直读）+ attrbody 20/20 run hi 低 2 位=3 + $2007 流 41/41 +$20 双证。
- **裁决表述如实（claim 5）✓**：GPFMODE=2 不设默认（缺省=1 实证；§6-1"双向碰运气/D4 保真反例"表述与证据一致）；gs5 1,524→**2,355**（−831）/stg03 三模式逐字节不变(`246dd5712608`)/三样本总和 64,975/63,851/63,761 全部复现。
- **tracked 洁净（claim 6）✓**：`oracle/screens/diff/` **无** p0-1-gpf-*.png——3 件仅在 `target/p0-1-gpf/img/`（target=再生区不入库）；`git status` 仅 `render_frame.rs` M + finding 文档 ?? ⇒ **无需清理项**。

## 1. Findings 四分类
### 1.1 规格精度：无
### 1.2 数字/示例精度（低；建议原文 dated 更正）
- **F1**：「69/70 置位帧零填充（仅 f5 混合）」错——实测 **67/70**，混合帧=**3**：f5(48 前+224 后)/f924(30 前)/f1207(112 前)；文档自身汇总 101−98=3 与"仅 f5"自相矛盾。「置位后仍有填充 仅 f5」属实；两链独立结论不受影响（f924/f1207 填充均在置位前、内容走 F1EB）。
- **F2**：「写点 pc=EED1/EEE4」不全：70 置位实测 5 个 pc（EED1×40、904E×23、900D×4、EEE4×2、ED6B×1）。
### 1.3 证据不足/其他（info）
- **F3**：117「preserve 拦截格」中 **73 格**在同一模型内 preserve 无本地作用（text_p==text_n==truth；分类序 b==t 先判所致）；像素探针 (0,737)/(0,819) 显示 form1==form3≠exp ⇒ 该类格差异非 preserve 所致（疑 attr 面/截图时点，与 §7-3/§7-4 未证残留同源）。117 宜读作「去 preserve 即=真值」分类口径（其中 44 格有拦截实证）；143 格归属扎实。
### 1.4 越权/伪造：无（未动 oracle/gates/expected/阈值；GPFMODE 缺省零变化经 v0 实跑证实）

## 2. 命令输出摘要
```
cargo test -p game-desktop            → 5+0+27+14+14+5+0=65 全绿（gpfmode 单测 ok）
python target/p0-1-verify/compare.py  → 5292/3337/3247/3247；两两 1955/2045/90/0；v1==v1b；v3==v3p
python target/p0-1-verify/setclosure.py → E==A|C|D 且两两不交=True
python target/p0-1-verify/probe_verify.py → 法A==法B(21,247 事件)；rowbody 82×16B 全零 41/41 +$20；palbody 70×32B@$3F00；cmdstream 194 run/8,962 写(93×77B)；填充 68/101 全命中 F1EB
python target/p0-1-verify/text_verify.py → 260 格=143/117/0；4 例格 MATCH；零写 356,379/948（NT 瓦面）
python target/p0-1-verify/compare_matrix_v.py → gs5 1524/2355/2355；stg03 58159×3 同 sha 246dd5712608
git ls-files/status → oracle/screens/diff/ 无 p0-1-gpf；工作树仅 render_frame.rs M + finding ??
```
