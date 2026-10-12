# H-02 深路径考证（t35）— slot≠0 同型升级分支定谳

> 目标：闭合 t31 L2 线索（拾取例程 `bank2:$922F TAY/BNE $925F` 的深路径），
> 复核 t29「12 级配置/回卷 $0C」结论的分支完整性。方法：seed 反汇编 + Ghidra 伪代码交叉 +
> weapon-pickup 合成入参 L2 实测（只加用例，未动实现）。

## 1. 分支全景（已证实 A/B 级；IDA seed + Ghidra FUN_922b 逐分支一致）

**拾取例程 dispatch = 2×2（同型/异型 × $018C==$FF 与否）+ $32 门控**：

```
bank2:$922B  CMP $34            ; A=拾取圈值
$922D  BNE $9248                ; 异型 → STA $34 切换（t25 N-0032 维持）
$922F  TAY / BNE $925F          ; 同型：圈值≠0 → ★深路径 $925F
;— slot==0 同型（圈值=0）直落：
$9232  LDA $018C / CMP #$FF / BNE $9271   ; $018C≠$FF → 也进深路径汇合点
$9239  LDA $32 / BPL $927A                ; $32≥0 → 连收计数路径
$923D  INC $33 / BNE / LDA #$0C/STA $33   ; ★内联升级 kernel（**溢出 255→0 回卷 $0C**）
```

**深路径 `bank2:$925F-$92C5`**：

```
$925F  LDA $32 / AND #$04 / BNE $9271     ; $32 bit2 守卫（置位则跳过 $9177/$91B3 副作用）
$9265  LDY #$02 / JSR $9177               ;（$32 bit2 清时的附带调用）
$926A  LDA $34 / BNE $9271
$926E  JSR $91B3
$9271  LDA $018C / CMP #$FF
$9276  BEQ $927A                          ; $018C==$FF → 连收计数路径
$9278  BNE $92B4                          ; 否则 → ★裸 INC $33（**无 $0C 回卷**）
$927A  INC $018A                          ; 同型连收计数 +1（N-0048）
$927D  CMP #$1E / BCC                     ; 满 30 → 归 $018A-=$1E + JSR $DBA9（★金钱/奖励事件，bank14 计数器族）
$928C  LDA $33 / CMP #$08 / BCC $9299     ; $33<8 → 跳
$9292  LDA $34 / BNE $9299                ; $34≠0（深路径固有）→ 跳过 kernel
$9296  JMP $923D                          ; （仅 slot==0 且 $33≥8 时回 kernel）
$9299  LDA $018C / CMP #$FF / BEQ $92C5   ; $018C==$FF → 收尾（★等级不加！）
$92A3  LDA $018A / SEC / SBC $018C / BCC $92B6
$92AC  STA $018A / LDA #$35 / JSR $ED3B   ; 扣减门槛 + 事件调用
$92B4  INC $33                            ; ★深路径升级点（裸 INC）
$92B6  LDA $32 / AND #$04 / BNE $92C5     ; 收尾
```

### 语义定谳（对照 t29）

| 路径 | 触发条件 | 等级变化 | 溢出行为 | 判定 |
|---|---|---|---|---|
| 内联 kernel | 同型 且 圈值=0 且 $018C=$FF 且 $32<0 | INC $33 | **255→0 回卷置 $0C** | t29/t25 结论维持（pk-2..5 锁定） |
| 深路径·裸升 | 同型 且（圈值≠0 或 $018C≠$FF） | $92B4 INC $33 | **自然回卷 255→0（无 $0C）** | ★t29 撤回范围 |
| 深路径·连收 | 同型 且 $018C=$FF（$018A 计数分支） | **不加等级**（仅 $018A+1） | — | ★新事实 |
| 异型 | 不同圈值 | 切换 $34，等级不动 | — | t25 N-0032 维持 |

**对 t29 结论的更正**：「$0C=12 级上限/回卷」**仅为 kernel 路径语义**；`$9457` 12 项配置偏移表
的索引输入 $33 在深路径可越过 12（裸 INC 至 255）→ t29 A1 假设（索引变换越界行为）成为**必答项**
（`bank2:$9368` 变换 `3+((级-4)&7)` 对 >11 输入的取模行为 → 假设 H-A）。

**新事实**：`$018A` = 同型连收计数（N-0048 语义补强）——满 **30** 触发 `JSR $DBA9`
（bank14 计数/奖励例程族，与 t22 `money_counter_bcd` 加法器同源）；`$018C` = 拾取门槛/成本值
（$FF=免检特判），`SBC $018C` 扣减路径附带 `JSR $ED3B($35)` 事件。

## 2. L2 实测（只加用例，未动实现）

新增 `oracle/subroutines/l2_cases.jsonl`（三方全等 ✓ 通过）：

| case id | 输入 | golden（=ROM 实测） | 三方结果 |
|---|---|---|---|
| pk-8-deep-same-3inc | $33=5,$34=3,a=3,$018C=55,$32=04 | A=4,X=7,Y=3,$33←6 | ✅ ROM==golden==Rust |
| pk-9-deep-same-feto ff | $33=FE,$34=5,a=5,$018C=55,$32=04 | A=4,X=7,Y=5,$33←FF | ✅ 同上 |

- 有效性自证：篡改 pk-8 golden→测试 FAIL 并打印 `rom=Out{a:4,x:7,y:3,writes:{51:6}}`
  （ROM 实测值=手算值，已复原）。
- **待补用例（需先改实现，本轮按『不动实现』不入文件）**：
  - `pk-10-deep-overflow-feto00`：深路径 $33=FF→**00**（对照 pk-4 kernel 的 FF→0C）；
  - `pk-11-deep-noinc-ff`：$018C=$FF 深路径 → **$33 不变**、$018A+1。
  两条均为当前 game-core `pickup_handler`（kernel 语义）与 ROM 的**已证分歧**，
  由 porter 按本 findings 实现深路径分支后补入（TDD 用例 JSON 见 git 历史/本文件 §1 语义表可推导）。

## 3. 复现命令

```bash
python3 tools/da6502.py rom/gunnac_cn.nes 0523F 922F 40     # dispatch 点
python3 tools/da6502.py rom/gunnac_cn.nes 0526F 925F 68     # 深路径体
sed -n '/FUN_922b @ 922b/,+45p' analysis/ghidra/bank02.c     # Ghidra 交叉
cargo test -p game-oracle --test l2_cases                    # 三方对拍
```
