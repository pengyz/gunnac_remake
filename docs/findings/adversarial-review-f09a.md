# 对抗复核 · p0-1-f09a（F09A 行体流接入 + 0 码 preserve 分治）

**VERDICT: pass**（3/3 复算一致；2 项备注不构成回退）

1. diff（`git diff --stat` = +337/−7，与自述一致）：`rowbody_write_cell` 0 写直落 `nt.tiles` ⇒ **0=擦写生效**（不经 preserve）；
   `text_zero_erase_ok` mode0/1/2/3 **分治语义在**（缺省 1=真值确认放行、无真值保持 preserve fail-closed）；**无阈值/像素白名单改动**
   （全仓改动=render_frame.rs + ledger + 本 findings；ENV_KNOBS 仅登记新旋钮 F09AROW/TEXZERO，属记录用 env 清单，非门禁阈值/视觉白名单）。
2. 独立复渲 1 样本 1 跑 f1260：`diff_visual.compare` = **1265** = 预期 ✅。备注：父命令缺 `REPO_ROOT`，二进制按 exe 上溯 3 级误定仓根=target/p0-0
   ⇒ out-dir 越界 fail-closed（实测退出码 1，与代码注释"退出码 2"不符——预存非本 diff）；补 `REPO_ROOT=D:\workspace\gunnac_remake` 后复现成功，复现命令建议补此 env。
3. findings 数字表一致：f1260 after=**1,265** ✅、+337/−7 ✅；未复核面=cargo test 73 全绿（本次禁跑 cargo）与 gs5/stg03 两样本（任务限定 1 样本 1 跑），仍属 Lead 自跑自述。

复核人：adversarial-review 子 agent（独立于实施方）｜2026-10-20
