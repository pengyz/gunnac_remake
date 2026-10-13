# 精灵装配/优先级语义取证（t103）— OAM 链 + 翻转/优先级 + 槽分区

> 方法：装配例程反查（bank14 发射器）+ attr LUT 字节实证 + NMI DMA 链 + 截图常识交叉。
> 精灵层规则素材 = §4（登记归 spec-writer，移植归 porter）。

## 1. OAM 装配链（①）—— **发射器 + NMI DMA 定谳（A 级）**

**发射器 `bank14:$CFC5-$CFE7`**（调用方：t72 弹绘制 $A17C/A9C5/AEFC/AFE6 等 10 处）：

```
$CFC5: LDA $54 / LSR / BCS 退出    ; $54 bit0 = 发射许可门
$CFCC: LDX $52                     ; ★$52 = OAM 槽索引
$CFCE: $0200,X := $F8（Y 预置隐藏）→ $58(tile) → $57(attr) → $56(X) → $55(Y 覆盖)
$CFE7: JMP $CE73（槽推进）
```

**精灵条目 = NES 标准四元组**（Y/tile/attr/X → `$0200+4n`）；发射参数：`$55`=Y、`$56`=X、
`$57`=attr、`$58`=tile（t72 弹绘制同源 ✓）。

**NMI DMA 链（bank15:$EF9B-$EFA2）**：`$2003:=0`（OAMADDR 清零）→ **`LDA #2 / STA $4014`**
（$0200 页整页 DMA）→ 渲染恢复（`$1B` PPUMASK 影子）——每帧 vblank 一次全量上传 ✓。

## 2. 优先级/翻转语义（②）—— **NES 标准位 + LUT 实证（A 级）**

**attr LUT `$88F7`（20 项）实录**：`41 C1 C1 C1 81 81 81 81 01 01 01 01 01 41 41 41 00 80 00 80`

| 值 | 位解 | 语义 |
|---|---|---|
| $41 | 0100 0001 | **H 翻转** + 调色板 1 |
| $C1 | 1100 0001 | **H+V 翻转** + 调色板 1 |
| $81 | 1000 0001 | **V 翻转** + 调色板 1 |
| $01/$00 | — | 调色板 1/0 |
| $80 | 1000 0000 | V 翻转 + 调色板 0 |

→ **位语义 = NES 标准**（bit7=V 翻转、bit6=H 翻转、bit5=优先级（0=BG 前）、bits1-0=调色板）✓；
**$88E3 tile LUT**（`59 5B 5B 5D 5D 5D…`）+ attr 四向翻转组合 = **弹旋转动画实证**（t72 的
运动码→(tile,attr) 对）。

## 3. 8×8 vs 8×16（③）—— **8×8（B 级）+ 残余**

- tile LUT 为**单 tile/方向**（$59/$5B/$5D 族）+ 翻转组合动画 = **8×8 模式典型用法**。
- PPUCTRL 影子 = **`$1A`**（bank15:$EFC1 `LDA $1A / STA $2000`）；bit5 直读值**未定**（写者为
  计算值非立即数）→ W42（不影响 8×8 判断的置信度但留口）。

## 4. 槽归属与限制（④⑤）

| 项 | 证据 | 级 |
|---|---|---|
| **OAM 槽分区**：`$52` 初值 **$10（游戏对象）/ $F0（bit6 分流）**（bank14:$CDE8/$CDED） | A |
| `$00-$0C` = **HUD 预留区候选**（发射器槽从 $10 起） | B |
| 发射许可门：`$54` bit0（清零=允许）、bit6 = 分流标志 | A |
| 槽推进：`$CE73` 族（+4 步进） | A |
| **8/行溢出/闪烁 = 硬件 sprite evaluation 限制**（D4 保真对象——原版 quirk 必保留，BUG 登记通道） | A（硬件事实） |

## 5. 精灵层规则素材（报 captain，登记归 spec-writer，移植归 porter）

1. **R-sprite-entry**：四元组格式（Y/tile/attr/X → $0200+4n，Y=$F8 预置隐藏协议）；
2. **R-sprite-attr**：attr 位语义（V/H 翻转、优先级、调色板 2 位）+ $88E3/$88F7 运动码→(tile,attr) 表；
3. **R-sprite-dma**：NMI $2003:=0 + $4014 DMA $0200 页（vblank 全量）；
4. **R-sprite-slot**：槽分区 $10/$F0（bit6 分流）+ 发射门 $54 bit0；
5. **R-sprite-quirk**：8/行溢出闪烁（D4 保真——Stage A 不修，BUG 登记）。

## 6. 复现命令

```bash
sed -n '/bank14:CFC5/,/bank14:CFE8/p' reference/seed/gunnac.asm
sed -n '/bank15:EF9B/,/bank15:EFA3/p' reference/seed/gunnac.asm
xxd -s 0x48F3 -l 40 rom/gunnac_cn.nes   # tile/attr LUT
sed -n '/bank14:CDE0/,/bank14:CDF0/p' reference/seed/gunnac.asm
```

## 7. 遗留

| # | 项 | 级 |
|---|---|---|
| W42 | $1A bit5（8×16 模式位）直读值 | C |
| W43 | HUD 槽分配例程（$00-$0C 写者） | B |
| W44 | 8/行溢出的运行时观测（需密集场景 trace） | C |
