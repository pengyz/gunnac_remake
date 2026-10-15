---
type: gotcha
title: CPU↔file 偏移换算必须显式声明——漏 iNES 头 $10 的 off-by-16 陷阱
date: 2026-10-12
status: confirmed
---

# gotcha：CPU↔file 换算声明（iNES 头 `$10` 陷阱）

## 现象

对抗复审 #2-R3（t149）活例：复审方口径 **off-by-$10**（漏 iNES 头 16B）——按错误换算读出的
字节值与被审方对不上，险些误判"数字不成立"。换算钉死后对账：**读法 A（`file=prg+$10`）**
file `0x10C0E` = CPU `$ABFE` 读 77/77 ✓ vs 读法 B（漏头）35/77 ✗；头字节 file `0x10C0D`=`00`
= CPU `$ABFD` ✓；probe `$2F/$30=$ABFD` 旁证自洽——t140 R2-1 整行撤回（错在换算层不在数据）。

## 为什么

- NES ROM 文件以 **16B iNES 头**开头，PRG 数据从 file `+$10` 起；CPU 地址还要过 bank 窗映射。
  **两层换算**（CPU→PRG 窗内偏移、PRG→file +`$10`）任何一层漏掉都会整体错位。
- 换算错的特征是"**所有数据整体偏移一个常量**"——看起来像数据错，实为换算层错
  （解释层缺陷伪装家族新员，见 [gotcha-register-address-is-not-nametable-target.md](gotcha-register-address-is-not-nametable-target.md)）。

## 何时使用

- 任何 CPU↔file 换算处**显式声明公式**（如 `file = prg + $10`、bank 窗换算式），不许口头"你懂的"。
- 对账用**双偏移读法对照 + 头字节校验 + probe 旁证**三件套（t145 裁定流程规范）：
  同一目标字节按两种换算各读一次，+`$10` 侧与 probe/头字节吻合方为真。
- 复核他人数字前先核对其**换算声明**；无声明=先要声明再对数。
- 仓库内 bank2 场景示例：`file = cpu − 0x3FF0`（= cpu − $8000 + bank2×$2000 + $10 的化简）。
- 【2026-10-15 实例补录·第 7 例】enemy-item-scheduling §6.2 `$A325` 表标注「file 0x4325」实为 **file 0x4335**（0x4325=PRG 偏移误记为 file）——port-engineer/test-runner 双偏移实测裁定；本会话累计 7 实例，**PRG 偏移与 file 偏移必须带前缀词区分**（`PRG 0x4325`/`file 0x4335` 并书最稳）。

## 证据指针

- t149 对抗复审 #2-R3（`docs/findings/adversarial-review-04.md` R2-1 撤回注：换算层缺陷四分类）
- 双偏移对账记录（读法 A/B 对照 + `xxd` 头字节 + probe `$2F/$30` 旁证）
- t138 数字成立复跑（77/77 按 `file=prg+$10` 读法）
