# 秘籍/隐藏选项证据表（Soig 外部情报复证，P2 spec）

> 状态：2026-10-16 v1（T2，decomp-analyst）——[tas-soig-external-data.md](../findings/tas-soig-external-data.md)
> §3 秘籍 9 条按证据纪律复证（B 级→A 级逐条）；每条带证据指针（汇编行/文档）。
> 复证方法：ROM 组合键代码路径定位（掩码/CMP 立即数全扫 bank00/03）+ 既有三方坐实引证。
> **未定位代码路径者维持 B 级**（负结果显式记录，不发明语义）。

## 复证总表

| # | 秘籍 | 复证级 | 证据指针 | 判定 |
|---|---|---|---|---|
| 1 | 设置声音测试选 5=选关+激活 AREA-0 | **A**（三方坐实） | bank00:935A-9369（$01AE SOUND 字段）/938F `CMP #5`/9387 旁路条件（flag==0∧$1AF==0∧$1AE≠5）；[hidden-option-gate-fff9.md](../findings/hidden-option-gate-fff9.md) §3.3；复核 #86 行 6/7 | 复证维持 |
| 2 | 免费购物：首次进店离店前按住 左+选择 | **B**（路径未定位） | 组合键掩码全扫 bank00/03 **负结果**（无 `AND #$21/#$20/#$01` 立即数形态）；推定经 $16/EED7 输入处理间接判（未展开） | 待复证（S-CH-x） |
| 3 | 钱袋→特殊武器：「结束」光标按住 右+A/B/开始 提前激活 | **B**（路径未定位） | 同上负结果（无 `AND #$82/#$42/#$12` 形态）；效果语义（D2/A1 隐藏选项）与 [$01B6/$01B7](../findings/community-absorption-audit.md) WEAPON/BOMBER 字段族相容 | 待复证（S-CH-x） |
| 4 | 开局第 5 帧按 A+B 直进设置 | **A**（全收，含归属）【2026-10-16 S-CH-1 收 A】 | **bank00:8DEB-8DF3**：`LDA $18/AND #$C0/CMP #$C0/BNE`→**`JMP $929E`（设置屏入口**，93xx 菜单域）——A+B 同按直入设置 ✓ 行为吻合；场景 init 归属（第 5 帧=启动 init 单次取样窗）B 级 | 复证升 A（归属注） |
| 5 | 即时补给 | **存疑** | 源文档自注「作者自测未实现」；无代码线索 | 存疑不发明 |
| 6 | debug 模式 PENNOYAA（FFF9=01）：设置+不死/武器/特殊武器 | **A**（三方坐实） | [hidden-option-gate-fff9.md](../findings/hidden-option-gate-fff9.md)（$FFF9 门 9 处 CN/8 处 JP 全扫）；N-0094 换算注；复核 #86 行 5 | 复证维持 |
| 7 | logo 界面 上+开始=直跳 ending | **B**（路径未定位） | 掩码全扫负结果（无 `AND #$18` 形态于 UI 域）；速通技巧类 | 待复证（S-CH-x） |
| 8 | 隐藏 OP：兔子画面长按 A+B→"Death & Non-Death Free Select Mode OK!"+设置+不死 | **A**（$01BE 语义）+**A 族**（触发面同 #4 门，2026-10-16 收；兔子 OP 场景过 init 的 pass-through 注=B 小注） | $01BE 语义 **A**：bank00:$947C `LDA $1BE`（复核 #86 行 8）+ `$9597/$95AB` 记录解码（DEATH/NON DEATH 二选一）；A+B 触发=bank00:8DEB 门同族候选（见 #4 注） | 半 A 半 B |
| 9 | 隐藏 ED：不死模式开启且未用→ending 长按 A+B 解锁日版画面 | **B**（旁证） | ending 流=[ending-sequence.md](../findings/ending-sequence.md)（A3 结局流域）；A+B 门同族候选（bank01:A0CD `LDA $16/AND #$C0` 在局内域未定归属） | 待复证（S-CH-x） |

## A+B 门代码路径（#4/#8 共用族定谳【2026-10-16 S-CH-1 收】，A 级站点）

> 【2026-10-16 S-CH-1 归属定谳】**8DEB 门 = 通用场景 init 单次取样**：8DE1-8DE3 清 $26/$27 ↔
> trace **f1193 关 init 清旗**（$26/$27: $10/$00→$00/$00，$0300:=$01 同帧）直接匹配——
> 该 init 流含**启动 init（「第 5 帧」窗，boot 零→零不可见）与关 init（f1193 实证）**两调用位；
> A+B 于取样帧→`JMP $929E` 设置屏（93xx 域含 SOUND TEST $01AE + Death&Non-Death $01BE 配置行——
> #8「设置+不死模式」效果吻合 ✓）。**#4/#8 触发面同门收 A**；#9（ending A+B）维持独立待证（S-CH-3）。

```
bank00:8DDF  $26:=$00 / $27:=$00          ; 场景 init 清流程旗
bank00:8DE8  JSR $E59D                    ; 帧同步
bank00:8DEB  LDA $18 / AND #$C0 / CMP #$C0 / BNE 8DF6
bank00:8DF3  JMP $929E                    ; **A+B 同按 → 设置屏入口**（93xx 菜单域）
bank01:A0CD  LDA $16 / AND #$C0 / BEQ / JMP $A06C   ; 局内 A+B 保持判（归属未定，#9 候选）
```

- **#4 与 #8 的 A+B 触发疑为同一门**（8DEB 单次取样=「第 5 帧」窗口 / 兔子画面 init）——
  场景归属一处钉死即两行同时收 A（S-CH-x 联动项）。
- #2/#3/#7 掩码形态负结果本身为证据（直接立即数判不存在）——推定经输入处理 helper
  （$16/EED7）间接判，属深层取证（S-CH-x）。

## 待复证挂账（S-CH-x）

| 项 | 内容 |
|---|---|
| S-CH-1 | ~~8DEB 场景归属钉死~~ **已收**（2026-10-16：通用场景 init 定谳，f1193 实证+boot 窗注）；余小注=兔子 OP pass-through |
| S-CH-2 | #2 左+选择 / #3 右+A/B/开始 / #7 上+开始 的输入判代码路径（$16/EED7 间接判展开） |
| S-CH-3 | #9 ending A+B 解锁路径（bank01:A0CD 归属 + ending 流对照） |
| S-CH-4 | #3 效果实测（钱袋→特殊武器转换的 $01B6/$01B7 字段变化 trace） |

## 复现命令

```bash
# 组合键掩码全扫（#2/#3/#4/#7 复证负结果/正结果）
python3 - <<'PY'
import re
pat=re.compile(r'^bank(\d\d):([0-9A-F]{4})\s+((?:[0-9A-F]{2} )*[0-9A-F]{2})\s+(\S+)\s*(.*)$')
for line in open('reference/seed/gunnac.asm'):
    m=pat.match(line.rstrip())
    if m and m.group(4) in ('AND','CMP'):
        op=m.group(5).strip()
        if re.match(r'^#\$?(C0|21|20|82|42|12|18)$', op):
            print(line.rstrip())
PY
```

## 【2026-10-16 R-D2-1 收口】FFF9=1 设置屏新增行 RAM 写点逐行定位（bot poke 面）

**行指针表（A 级，bank00:$94D0 起倒序对）**：`AE 01 | 4C 00 | 4E 00 | F1 00 | BE 01 | B6 01 | B7 01`
= **七行**→$01AE(SOUND)/$004C(武器索引)/$004E/?/$00F1/?/$01BE/$01B6/$01B7。

**写机制（A 级，93C1-93D3）**：行光标 $01AF → 指针表项 → **`STA ($04),Y` 间接写**（93D3）
+ 值环绕 $95ED 表（93C1 `CMP $95ED,Y` 循环）+ $16&$2C 输入判（上下移行/左右调值）。
——**间接写=直接搜 STA 零命中的谜底**（W-DMG3-BIND 同源现象互证）。

**三行效果语义（秘籍 #6 直答）**：

| 行 | RAM（bot poke 地址） | 值域/语义 |
|---|---|---|
| **不死模式** | **$01BE** | DEATH·NON DEATH 二选一（$9597/$95AB 记录对=`66 22 63 64 65…/86 22 73 74 75…` 两态文本）——**bot 不死=poke $01BE**（NON DEATH 态值） |
| 武器选择 | $01B6 | 武器号（显示尾字符=值+$31，复核 #86 H-04） |
| 特殊武器选择 | $01B7 | 炸弹型索引（$95F5 表=`FBTW` F/B/T.W） |

- 附：$004C 行=W-DMG3 武器索引字段**同表在列**——武器选择行可改 $4C（BIND 结论适用域已 scoped，
  见 external-data-verification.md dated 补注）。
- R-D2-1 动态面收口 ✓（bot poke 地址+三行语义+写机制全链）。
