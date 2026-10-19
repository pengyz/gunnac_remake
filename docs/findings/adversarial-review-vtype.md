# 对抗复核：$0160 v 分型路由（adversarial-review-vtype）
> 2026-10-20 04:5x | verifier（证伪立场，实施方≠复核方）×被审：工作树 `crates/game-desktop/src/bin/render_frame.rs` + `docs/findings/p0-1-vtype-routing.md`。独立重导：diff 全读、cargo test、自建 binary 独立复渲（`target/vtype-review/render/`）、逐像素复算、probe 逐拍复扫。未跑 `gates/cli.py check/status/visual`，无 git 操作。

## 0. VERDICT：**pass**（声称逐条对上；3 条非阻塞 findings，F3 建议随下一小 commit 清）

实现面逐条证伪未破（A）：`SetSnap{Palette{v,pal:$161-167},Row{v,snap}}` + `is_pal_type(v)=(v==3)` 落地；NMI 消费 v=3⇒只落 `pal_upload_state`（**不动色选**：grep 全文该变量仅 4 处=声明/赋值/sidecar/日志，零渲染消费）+ 行快照不应用（gameplay/gs5 `row_uploads=0` 实测）；v≠3 与缺 op 回退⇒行上传，几何块与 HEAD **去空白逐字符全同**（geo!=0/legacy 32 连写/attr 三块均 True）；stg03 缺 op 回退=固定行路（`replay_row_sets=1503/row_uploads=1502/pal_uploads=0`）；日志/sidecar 新字段全在。**测量全中**：gameplay f1260=**5,292**（`efc5fb2dbc33`）两跑同 sha；gs5 f3450=**1,524**（sha=**`245cbedc4398…`** 逐字吻合）两跑同 sha；stg03=**58,159**（`246dd5712608`）与改前**逐字节同**（before/vtype/历史三方比对 True）。**90px 复算**：before（我方用 `p0-1/build/render-frame-after.exe` 重渲=5,202/`51b7a201f021`）vs vtype 差**恰 90px**，方向 **90/90 全为"before 误刷行偶然修对"**（抽 5 点：(1,64)等 exp=before=黑、vtype=白），反向 0、其他 0 ⇒ B 假设证伪表述如实。测试 **64/64**（5+26+14+14+5；3 新单测真测路由/边界：v=3 不建行快照+pal 快照+计数、v=1/2 行路、混型覆盖以最后 v 定型）。总和 64,885→64,975（+90）如实。

## 1. findings 四分类
### ① 规格/语义精度（实现层，均非阻塞）
- **F1（info）**：任务语「v=0 清槽」≠实现字面——实现=v=0 写**不入槽**+`consume()` 清空，**不回退已挂起槽**；doc §7-4 已如实披露。独立复扫 probe：gameplay/gs5 `frames_with_set_then_cancel=0`（v 直方图 gameplay {0:109,3:70}、gs5 {0:134,3:94}）⇒ 无现行影响，无取消例属实。
- **F2（info）**：混型覆盖时 `row_sets/pal_sets` 计入被覆盖的中间置位（doc §6 已注）；`hud_seq` 随 v=3 不建行快照不推进（上轮 F1 latent 同域），无专项单测。
### ② 数字/证据精度（报告层）
- **F3（P2，建议修）**：更正只登记于 p0-1-vtype §4/§7-7，**p0-1-event-replay.md 原文未原地加 dated 注**——§3 L105/§10 L155（HUD LUT 误诊）与 B 假设（L32/§6-6）仍以旧文持论；AGENTS.md §3-6「在原文**原地**标注日期更正」与实施方援引的 G7 同式（登记向不改他人文档）冲突，按 AGENTS.md §1 应以前者为准 ⇒ 建议补一条原地 dated 注（小 commit）。更正内容本身经复核为真（`set_moment_state.py` 复跑：gs5 94 次 regs=`00:20:lut1×65/00:21×19/c0:22×10`、gameplay 70 次=`00:20:lut1×65/c0:22×5`，**全 ≥$2000** ⇒ HUD LUT 回退条件不成立；行非零 gs5 69/94、gameplay 57/70 全对）。
- **F4（info）**：§4「gs5 真因=draws=0」的 draws 计数本轮未复算；但可观测事实我方直接证实（before/vtype 的 gs5 PNG 逐字节同、而 nt_nz 355→373 差 18 格 ⇒ NT 差异确不可见）。
### ③ 证据不足/边界（复核边界披露）
- MODELGEO=1 列 3 样本与 6/6 全矩阵未全复跑（按任务范围只复渲 2 样本 ×2 跑 + stg03 单跑）；`pal_3f00/3f10` 组内槽位映射与 `$3F00`→像素索引绑定沿用「未证不发明」——doc §7-2 显式未证 ✓、§7-1 真因悬空显式 ✓、§7-6 draws=0 警示显式 ✓，无「仍成立」残留（grep 全仓核过）。
### ④ 越权/伪证
- **无**。声称各数（5,292/1,524/`245cbedc4398`/58,159/90px/64 测试/pal_sets=70·94/row_uploads=0/总和 +90）独立复算全部相符；无阈值/白名单/expected 改动（git status 仅 render_frame.rs+新 findings）。

## 2. 命令输出摘要
```text
cargo test -p game-desktop ⇒ 64 passed/0 failed（lib5+bin26[含 v3_set_routes_palette_only/v1_set_routes_row/
  mixed_vtype_frames_route_separately]+render14+render_rules14+smoke5）；LASTEXITCODE=0（首跑 exit1=PowerShell 2>&1 重定向伪信号）
复渲（REPO_ROOT 显式，legacy）run1/run2：gameplay 5292/5292 sha efc5fb2dbc33… 同；gs5 1524/1524 sha 245cbedc4398d24c1582be12394545550a9f4a2766ff318c70d30e58a84a490d 同
  日志：gameplay replay_sets=70 replay_pal_sets=70 row_uploads=0 pal_uploads=70；gs5 =94/94/0/94；sidecar {palette_src,pal_3f00,pal_3f10,attrs,tiles} 齐
stg03 4600：diff=58159 out=246dd5712608… 与 before/历史逐字节同；legacy_snap_frames=1503 replay_row_sets=1503 row_uploads=1502 pal_uploads=0
逐像素（decode_png 全 256×240）：before_vs_exp=5202、vtype_vs_exp=5292、before_vs_vtype=90px、方向 90 修对/0 弄坏/0 其他
python probe 复扫：gameplay $0160 w=179（v=0×109,v=3×70）置位后取消例=0；gs5 w=228（0×134,3×94）取消例=0
几何块去空白比对（HEAD vs 工作树）：geo!=0/legacy32/attr 三块 normalized-identical=True
```
