# 对抗复核：geo2x16（MODELGEO 2×16 翻案）| 2026-10-20 | 审核 agent（非实施方）

**VERDICT: pass**（独立复渲逐字节复现；2 条备注：复现命令二进制路径勘误 + 1 项边界残留补录，均不阻断）

1. **映射核对（A，diff 实读）**：`modelgeo_addr_off(i)=(i/16)*$20+(i%16)` ⇒ buf[0..16]→base+0..15、buf[16..32]→base+$20..+$2F ✓；`modelgeo_attr_addr(hi,c)=((hi|3)<<8)|c` **无 +$0F**（main 中 attr_raw 已去 `wrapping_add(add_off)`）✓；与 [p0-1-geo2x16.md](p0-1-geo2x16.md) §1 表逐格一致；F09A 行体流/preserve 分治段**零改动**（diff hunk 仅 516/647/1708/2697，且 `git diff --stat` 只 1 文件，未触阈值/gates/expected）✓。
2. **独立复渲（A）**：MODELGEO=1 stg03 f4600 复渲 **diff=58,128、png sha256=`a9ca5a7f6c2e…`、nt_nz=993、row_uploads=1502**，与 findings §2/§3 逐字节一致 ✓。**勘误**：任务卡命令的 `target\p0-0\build\debug\render-frame.exe`（07:19:34，修复前旧二进制）得 diff=**58,152**、sha `532936218b01…`（恰=作废的 4×8 vtype 期数 58,152）；正确复现二进制=`target\debug\render-frame.exe`（08:00:54，随源 08:00:47 同批构建，即 run-geo2x16.ps1 所用）——**58,128/a9ca5a7f 结论成立，仅复现路径需换二进制**。
3. **findings 一致性（A）**：§2 数字表（legacy 1,524/1,265/58,159 + MODELGEO 1,524/1,265/58,128 + sha）与本次复渲一致；旧 MODELGEO 六数（2,138/4,054/58,169/1,524v/5,292/58,152）全部划线作废 ✓；§6 未证残留 4 项明示 ✓。未复核项（如实）：75 测试/gates 本卡不跑 cargo 未验；gs5/gameplay 两样本未复渲。**补录残留（B）**：落位用 `base.wrapping_add(off)` 为 16 位进位，ROM 语义 lo 回绕 **hi 不进位**——仅 lo≥$D1 的行基才分歧，行基 $40 对齐下永不触发，样本零效应，建议并入 §6 登记。
