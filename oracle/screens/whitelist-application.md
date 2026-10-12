# 视觉白名单申请清单（t96 4；未启用——启用需用户确认 + gates/visual_whitelist.json 登记）

> 申请判据（t33 §2）：仅平台差异（渲染路径不同但语义等价）可申请；游戏行为差异（含 D4 quirks）永不入。

## 申请项
- W-C1 调色板映射（强候选）：NES 调色板↔GREY4 公共域量化差——整帧系统性色偏；证据：diff/ 逐帧差异图 + 灰阶公共口径比对记录；建议先共用映射表再议豁免。
- W-C4 色彩空间/伽马（强候选，与 W-C1 合并处理优先）。
- W-C2/W-C3（弱候选）：字形/文本渲染差——预期移植对齐后消失，暂不申请。

## 归因证据
- 差异图：oracle/screens/diff/*.diff.png
- 分类记录：oracle/screens/visual-report.md
- 时序相位证据：docs/findings/linebuf-timing-wnew.md（t94）+ t96 处置

## 状态
申请中（未启用）——captain 转用户裁定前 gates/visual_whitelist.json 不建立。
