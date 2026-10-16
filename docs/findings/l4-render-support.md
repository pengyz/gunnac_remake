# L4 渲染数据支援面（L4→100% 战役令·随需供）

> 机制：test-runner/port **点名即挖**；每项落此（A/B 分级，版本差异显式）。
> 菜单：$D7B9 校正值语义 / scene CHR 对完整表 / fade 调色板表 / 隐藏 OP-ED 画面 CHR / graphics.csv tile 语义。

## R-1 · $D7B9 校正值语义（2026-10-16 点名首项，收）

**语义（A 级）**：**每关 tile 校正值表，9 字节/关序 1-9**——`00 00 00 00 00 00 00 C1 C1`：
关 1-7 校正 0（无校正），关 8-9 校正 **$C1**（tile 索引基值校正——关名/特殊图形 tile 映射偏置）。

**版本迁移注（重要）**：
- **JP ROM**：表在 file 0x1D7C9（=JP $D7B9）——社区 build_stage_patch.py 复用点
  （[community-absorption-audit.md](community-absorption-audit.md) T09）。
- **CN ROM（本 ROM）**：同 file 偏移=**代码流**（`B9 94 07 C5 70…`）——**汉化版重排，表迁移至
  file 0x1DA5C = bank14:$DA4C**（偏移 +0x293）；9 字节同值（00×7,$C1,$C1）✓ 原字节直读。
- **警示**：JP 表址引用照抄会踩版本差异（本项第 1 例）——L4 支援面所有 JP 派生地址须 CN 复核。

**读点**：CN 侧 `LDA $DA4C,Y` 直搜零命中（疑索引基址形/数据标记区扫读盲区）——挂 R-1b
（读点 xref 可选）；数据面（语义+值+地址）已齐，L4 用面可用。

## R-2 · scene CHR 对完整表（待点名）

## R-3 · fade 调色板表（待点名——S5-2 §7 fade_level 77→…→0 数值源疑同源面！）

## R-4 · 隐藏 OP-ED 画面 CHR（待点名——cheat #8/#9 兔子/日版画面域）

## R-5 · graphics.csv tile 语义（待点名）
