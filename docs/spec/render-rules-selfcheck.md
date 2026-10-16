# render_rules 渲染面自查（L4→100% 战役·步骤③第一件）

> 日期：2026-10-16｜角色：porter（覆盖面 vs 数据面缺口盘点——先查后修）

## 1. 现覆盖面（game-desktop::lib.rs 原语层）

| 件 | 状态 | 注 |
|---|---|---|
| Canvas 256×240 + draw_tile8（8×8 调色板绘制） | ✅ | 像素坐标/越界裁剪 |
| draw_chr（bank/tile 索引直绘） | ✅ | 解码=game-data::chr 同源 |
| Renderer trait 缝（占位语义） | ⚠ 占位 | draw_tile=纯色块占位（行为待实装） |
| view_rect/blit_scaled（缩放） | ✅ | |
| export_png（L4 对拍输出） | ✅ | |
| **fade 7 档** | ❌ | FADE_LEVELS 在 game-core（㉖）**未接渲染面** |
| $D7B9 tile 校正 | ❌ | 数据面缺口① |
| F340-F400 关名渲染 | ❌ | 数据面缺口② |
| scene 5 槽 CHR | ❌ | 数据面缺口③ |
| type 切 bank 敌型 CHR | ❌ | 数据面缺口⑤（type≥$30 切 bank 面已有——CHR 侧未接） |
| text.csv PPU 坐标 | ❌ | 数据面缺口⑥ |

## 2. 六数据面缺口对表（lead 弹药清单）

| # | 弹药 | 我方缺口 | 接线面 |
|---|---|---|---|
| ① | $D7B9 tile 校正 | tile 索引校正语义未接 | draw_chr 索引修正参数 |
| ② | F340-F400 关名渲染 | 关名 tile 流渲染未接 | 新 draw_stage_name（F340 流解析） |
| ③ | scene 5 槽 CHR | 场景 5 槽位 CHR 映射未接 | draw_chr bank 参数表 |
| ④ | fade 7 档 | **FADE_LEVELS 未接渲染面**（游戏侧已证㉖） | Canvas.apply_fade(level)——最小接线 |
| ⑤ | type 切 bank 敌型 CHR | TYPE_DISPATCH bank 面✓/CHR 侧未接 | draw_enemy_chr(type) 组合 |
| ⑥ | text.csv PPU 坐标 | 文本 PPU 坐标渲染未接 | text.csv 解析+draw_text 基础 |

## 3. 修复序建议（随 test-runner 差异清单）
1. **④fade 7 档**（我方已有证数据——最小接线即通）
2. ①$D7B9 tile 校正+⑤type bank CHR（tile/CHR 索引族——同面）
3. ②关名渲染+⑥text.csv（文本族）
4. ③scene 5 槽 CHR（场景特化）

## 4. fail-closed 边界
- 六件均按数据面字面接（弹药=decomp/test-runner 侧产物）；语义未证部分参数化不发明；
- Renderer 占位语义不提前实装（AGENTS.md Stage A 约束）。
