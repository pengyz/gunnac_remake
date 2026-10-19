# 对抗复核：probe 上限 + dated 更正 + BBBD 口径（2026-10-20）

**VERDICT: pass**（1 项 nit，不阻断）

1. `git diff tools/scenario_capture.lua` ✓ 仅 1 处：`probeN > 3000000 → 10000000`（1+/1-，无其它改动）。
2. numstat ✓ 4 个文档全 deletions=0 纯追加（adversarial-review-101 8、probe-setter-blindspot-fix 5、text-line-flag-survey 4、l4-signing-criteria 3）；diff 新增 `【2026-10-20 更正】` = **5 条**（≥5 ✓）。nit：numstat 共 5 行但第 5 行是 `tools/scenario_capture.lua`(1/1)，**文档实为 4 份非 5 份**；若预期第 5 份文档在本 diff 中则缺失（l5-0160-batch-semantics.md 的 dated 注已在此前 commit 中，未混入本次改动）。
3. BBBD/BBBF 口径一致 ✓（抽查 l4-signing-criteria.md:113-114、adversarial-review-101.md:24-26 / :82-83、text-line-flag-survey.md:50-51、l5-0160-batch-semantics.md:113-114）：BBBD=`bank3:BBBA AD 60 01 LDA $160` 读的后取指 PC（读点）、写点=`BBBF 8D 60 01 STA $160`；seed gunnac.asm:23309-23317 字节流 `AD 60 01 09 01 8D 60 01 60` 独立复核吻合（A 级）。
