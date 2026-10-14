# 对抗复核 #65：DEV-1 商店购买链接线（开发步双关第一关·fresh context·证伪立场）

> 被审：commit `96450a0`（`feat(core): DEV-1 商店购买链接线——A1 购买链规格落 game-core（shop_purchase_step）`，
> +894 行：`ui.rs` 追加 403 / `tests/shop_purchase.rs` 新增 377 / `docs/spec/shop-purchase.md` 新增 110 /
> `shop-economy.md` +4）+ `2ee1060`（`chore(data): ratchet rust_test_count 108→122`）。
> 审核方：独立 verifier #65（fresh context subagent，实施≠审核；不采信被审自述、不复用其结论、独立重导）。
> 三焦点：①只接已证规格未发明 ②测试真验行为非自证 ③对拍独立复跑。
> 数据源（只读）：`rom/gunnac_cn.nes`（未改动）、`reference/seed/gunnac.asm`（对照，不作真相）、
> `docs/findings/shop-purchase-flow.md`、`docs/spec/shop-purchase.md`、`crates/game-core/src/{ui,l2ports,weapons}.rs`、
> `crates/game-core/tests/shop_purchase.rs`、`crates/game-oracle/src/l2.rs`。
> 过程注：审中 HEAD 由 `2ee1060` 前移至 `8bee3c6`（后随 `ca4aed7`/`d7ee9ae`/`8bee3c6` 三个 commit
> **均为 docs/ 增改**，`git diff --stat 2ee1060..HEAD` 不含 `crates/`）——被审代码即本次复跑所测代码。
> 复跑时 HEAD=8bee3c6，122/122 全绿可复现。

**裁定：needs_revision** —— 焦点②③全过（4 项真 mutation 全红、三件对拍独立复跑全绿贴数），
未发明面总体干净（6 类消息指针、打包位运算、扣减/封顶调用链逐字节重导全对、无旁路重写、无 I/O）；
但**焦点①有一处已证行为漏接 + 配套 spec 记述不全**（AR65-1，必修·中）：退出隐藏门
`bank01:$A023-$A03B` 在 ROM 里是**两触发源**——`($18&$33)==$11` **或** `$0180==8`——两者都走
`$32|=bit5` + 文本 `$A65C`；实现与 spec R-7 只接了前者，`exit_entry_and_hidden_gate` 测试固定
`stage=0` 盖不住此分支。伴生 AR65-2（必修·低）：该门触发路径的消息 ack 等待（`$A09F/$A0CF`）
未建模。另 3 条非阻断备注（AD65-1/2/3，§6）。**保真红线方向正确但覆盖不全——"原版有的行为必须保留"
在这一条上未闭合**，故不判 pass。

## 0. 三焦点核验表

| 焦点 | 核验项 | 判定 | 本审独立证据 |
|---|---|---|---|
| ①未发明 | spec R-1..R-10 逐条证据指针支持度 | **9 通过 / 1 不合格**（R-7 记述不全且漏接） | §1 逐条表；ROM 直读 10 组片段（§1.1） |
| ①未发明 | 自采证据重导（隐藏门/子光标边界/阶段门/far-call 保活 A） | **3/4 成立，隐藏门漏一支** | §1.1：$A023-$A03B 两触发源直读、$9D77/$9F39/$9F89 边界对寄存器、$9F03-$9F16 双 CMP、$E649/$E68F |
| ①未发明 | 消息文案/音效/渲染未发明；$92C5 未发明写集；W-NEW5 fail-closed | **成立** ✅ | `ShopPrompt` 仅枚举无字符串；`ShopFx::WingBoughtFarCallPending` 显式挂起；`ship_index>=8` 双守卫（ui.rs L490/L521） |
| ②测试 | 抽 5+ 条逐行读：断言 RAM 终态非自证 | **成立** ✅ | §2.1 七条逐行（money/level/slot/delivery/cursor/prompt/phase 终态断言） |
| ②测试 | 负向哨兵真会红（mutation 实测，超出"想定"） | **成立** ✅ | §2.2 四项真 mutation：改扣减值→3 红、打包错位→2 红、门常数错→1 红、封顶丢→1 红（含哨兵测试），改后均 `git checkout` 还原 |
| ②测试 | TDD 叙述（RED=169 E0425）可信度 | **不可复现·可信但缺工件** | §2.3：测试与实现同 commit 落地，RED 期无痕；E0425 对未实现 API 合理；记 AD65-2 |
| ③对拍 | cargo test --workspace = 122 | **122/122** ✅ 独立复跑 | §3 贴数 |
| ③对拍 | l2-diff = 109/109 | **109/109** ✅ 独立复跑 | §3 贴数 |
| ③对拍 | gates check 12/12 | **PASS=12 FAIL=0 SKIP=0 TOLERATED=0** ✅ 独立复跑 | §3 贴数 |
| ③对拍 | 抽 1 条 l2ports 调用链（签名一致、无旁路重写） | **成立** ✅ | §3.1 `shop_spend_ship` 全链 + grep 零内联算术 |
| 架构 | game-core 纯逻辑无 I/O/时钟/平台 | **成立** ✅ | §4 grep：src 零命中（tests 内 std::fs 属测试宿主，contract.rs 为门禁自身） |
| 架构 | 与既有 ShopState（R-shop-5..8）无混接 | **成立** ✅ | §4：`ShopState`/`ShopPurchaseState` 零互引；96450a0 对 ui.rs 为纯追加 403/0 |
| 基线 | 2ee1060 留痕含"确认无回退"证据 | **成立** ✅ | §4：baseline_history reason 含 122/122+109/109+12/12 实测句，108→122 只升 |

## 1. 焦点①未发明核查（spec 逐条 + 自采证据重导）

### 1.1 独立 ROM 直读（不采信被审复现脚本输出，本审自跑 hex 直读 + 逐指令解码）

复现命令（与 spec 同法，本审独立执行；`prg = rom[0x10:0x20000]`，`off = bank*0x2000+(cpu&0x1FFF)`；
bank15 固定窗另计）：

```
python3 - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read(); prg=rom[0x10:0x10+0x20000]
def show(bank,cpu,n):
    o=bank*0x2000+(cpu&0x1FFF); print(f"bank{bank:02d}:{cpu:04X}", prg[o:o+n].hex(' '))
for args in [(0,0x9D02,0x24),(0,0x9D26,0x30),(0,0x9D56,8),(0,0x9D6C,4),(0,0x9D91,0x50),(0,0x9DD8,0x12),
             (0,0x9DE8,0xD),(0,0x9E40,0x38),(0,0x9E86,0x60),(0,0x9EDF,3),(0,0x9F03,0x14),(0,0x9F16,0x50),
             (0,0x9F62,0x11),(0,0x9F75,0x45),(0,0x9FAB,0x30),(0,0x9FDA,0x0A),(0,0x9CE0,0x14),(0,0x9CF4,0x0E),
             (1,0xA023,0x40),(1,0xA05B,0x80),(1,0xA0CF,5),(2,0x8FA9,0x0E),(2,0x92C5,0x20),(15,0xE649,0x48)]:
    show(*args)
PY
```

逐段解码结论（本审手解，与 spec 主张逐条对照）：

| 片段 | 直读字节（首/关键） | 解码 | 支持 |
|---|---|---|---|
| bank00:$9D02-$9D25 | `a5 16 29 08 … ce 00 04 10 19 a9 03 … 29 24 … ee 00 04 … c9 04 d0 05 a9 00 8d 00 04` | Up(AND#08)→`DEC $400` BPL 否则 LDA#3；Down(AND#$24)→`INC $400` CMP#4→0 | R-1 ✅ |
| bank00:$9D26-$9D55 | `a9 43 20 3b ed … a5 16 29 d0 f0 bc a9 44 20 3b ed 20 6c a0 ad 00 04 0a a8 b9 57 9d 48 b9 56 9d 48 60` | 音效$43/$44（未落✓）；`$16&$D0` BEQ 回环；`LDA $9D57,Y/PHA/LDA $9D56,Y/PHA/RTS` 字=入口−1 | R-1/2 ✅ |
| bank00:$9D56 表 | `5d 9d 85 9e 02 9f 22 a0` | $9D5E/$9E86/$9F03/$A023 四入口 | R-2 ✅ |
| bank00:$9D6C | `a5 34 8d 01` | `$0401 := $34` | R-2 ✅ |
| bank00:$9D77-$9D7D | `a9 00 85 11 a9 05 85 10` | **$11=0（下界）/$10=5（上界）**——sub_9E40 左移 `CMP $11` 命中换 `$10`、右移 `CMP $10` 命中换 `$11` | R-2 ✅（"子光标边界 $10/$11"自采成立） |
| bank00:$9D91-$9DE2 | `ad 01 04 c9 05 … a5 32 10 47 … a9 36 a9 a4 … 4c 5e 9d c5 34 d0 32 a5 32 30 17 a5 33 c9 02 … a9 b0 a9 a3 … 4c 5e 9d a5 33 c9 07 … a9 08 a9 a4 … 4c 5e 9d` | 校验链精确分支图：光标=5→bit7 已购→`$A436`；同型无翼 `$33`≥2→`$A3B0`；同型有翼 `$33`≥7→`$A408`；其余（异型/未满）全汇 `$9DE3` 购买 | R-4/8 ✅ |
| bank00:$9DAA/$9DC9/$9DE0 | 三处 `4c 5e 9d` | **消息返回 = JMP $9D5E**（重入武器面板，$9D6C 重置 `$0401:=$34`） | R-8 ✅（"消息返回目标 $9DAA"自采成立） |
| bank00:$9DE3-$9DF5 | `a9 44 20 3b ed ad 8e 01 c9 0a b0 03 4c 5b a0 e9 0a 8d 8e 01` | 音效后扣 10：`CMP #$0A/BCS` 否则 `JMP $A05B` 零写；**扣款在全部校验后** | R-3/4 ✅ |
| bank00:$9E40-$9E77 | `… c5 11 … a5 10 … ce 01 04 … c5 10 … a5 11 … ee 01 04 a9 43 20 3b ed` | 子光标环绕 $11→$10 / $10→$11 | R-2/6 ✅ |
| bank00:$9E86-$9EA5 | `… a5 3c c9 04 90 0e a9 d4 a9 a4 20 9f a0 4c ec 9c` | 连射入口 `$3C`≥4→消息 `$A4D4`→**JMP $9CEC（$9EA5）** | R-8 ✅ |
| bank00:$9ECD/$9EDF | `ad 8e 01 c9 06 b0 03 4c 5b a0 e9 06 …` / `e6 3c` | 扣 6 同构；`INC $3C` | R-3/5 ✅ |
| bank00:$9F03-$9F19 | `ad 80 01 c9 08 b0 04 c9 02 b0 0e a9 2a a9 a6 20 … 4c ec 9c` | **阶段门两 CMP**：≥8→消息 `$A62A`；<2→同消息；[2,7]→$9F1C | R-4 ✅（"$0180∈[2,7]"自采成立） |
| bank00:$9F1C-$9F22 | `8d 01 04 8d 03 04 8d 02 04` | A=`$0180` 三者同写 `$0401/$0403/$0402` | R-6 ✅ |
| bank00:$9F36-$9F3D | `ad 02 04 85 11 a9 07 85 10` | 槽环边界 $11=`$0402`/$10=7 | R-6 ✅ |
| bank00:$9F62-$9F72 | `ac 03 04 b9 78 01 2a 2a 2a 29 03` | 取旧弹种 `ROL×3/AND#03` = `(slot>>6)&3` | R-6 ✅ |
| bank00:$9F89-$9F8C | `a9 00 85 11 a9 03 85 10` | 弹种环边界 $11=0/$10=3 | R-6 ✅ |
| bank00:$9FAB-$9FDA | `ad 8e 01 c9 04 b0 03 4c 5b a0 e9 04 … ac 03 04 b9 78 01 29 3f 18 69 01 c9 40 90 02 a9 3f 85 04 ad 01 04 6a 6a 6a 29 c0 05 04 99` | 扣 4；qty=`min((old&$3F)+1,$3F)`；**`ROR×3/AND#$C0` 逐位= `($0401&3)<<6`**（ROR×3 后 bit7-6=原 bit1-0，进位落 bit5 被 AND 掉——本审逐位推演确认掩码语义精确） | R-3/5 ✅ |
| bank00:$9FDA-$9FE2 | `99 78 01 20 e3 9f 4c 86 9f` | `STA $0178,Y` 后 **JMP $9F86 购后停留弹种环** | R-6 ✅ |
| bank00:$9CF4-$9D01 | `20 d9 e6 a9 00 8d 00 04 …` | **`LDA #0/STA $400` 在 $9CF7/$9CF9**（$9CEC 为回主环序列入口：文本 $A2AB→渲染→复位） | R-3 ⚠️ 行为对、指针偏（AD65-1） |
| bank01:$A05B-$A06B | `20 6c a0 a9 3d a9 a1 20 9f a0 4c ec 9c` | 不足共享出口：文本 **`$A13D`** → ack 等待 → **JMP $9CEC（$A069）** | R-3/8 ✅ |
| bank01:$A091-$A09E | `a9 10 85 14 a9 22 85 15 ad 8e 01 4c 73 e9` | 金钱显示单锚（`$018E`→PPU $2210） | findings §2 ✅ |
| bank01:$A09F/$A0BC-$A0D3 | `20 c7 e6 … 20 bc a0 …` / `… a5 16 29 c0 f0 f4 4c 6c a0` | 消息多行渲染 + **ack 门 `$A0CF: AND #$C0`**（A/B 任一） | R-8 ✅ |
| bank01:$A023-$A03B | `a5 18 29 33 c9 11 f0 07 ad 80 01 c9 08 d0 16 a9 33 20 3b ed a5 32 09 20 85 32 a9 5c …` | **见 §1.2——两触发源** | R-7 ❌（AR65-1） |
| bank02:$8FA9 | `ad 8e 01 18 69 03 90 02 a9 ff 8d 8e 01` | +3 进位即 `$FF` | R-10 ✅ |
| bank02:$92C5 | `a9 01 85 3a a9 08 85 39 a5 32 29 f7 09 01 85 32 a4 34 b9 8f 96 8d a5 01 …` | 翼收尾写集（$39/$3A/$32 位变换/$01A5）——**实现未发明、显式挂起** | R-9 ✅ |
| bank15:$E649-$E68F | `85 04 … a5 04 60 … a4 05 a5 04 6c 24` | **入口 `STA $04` 保活 A、尾 `LDA $04/JMP ($24)`** ⇒ 目标收 A=`$0401` | R-5 ✅（"far-call 跳板保活 A"自采成立） |

**6 类消息指针全部直读坐实**：`$A13D`（不足）/`$A408`（武器满级，`a9 08 a9 a4`）/`$A4D4`（连射满级）/
`$A436`（翼已购）/`$A3B0`（翼前置）/`$A62A`（阶段门）——`ShopPrompt` 6 变体与之**一一对应、零发明字符串** ✅。
（注：任务下发摘要写 `$A048`，系 `$A408` 笔误；spec 与代码注释为 `$A408`，与 ROM 一致。）

### 1.2 AR65-1 现场：退出隐藏门是两触发源（本审证伪点）

`bank01:$A023` 起逐指令解码（直读字节见 §1.1 表末行）：

```
A023: LDA $18 / AND #$33 / CMP #$11 / BEQ A032     ; 触发源(a)：($18&$33)==$11
A02B: LDA $0180 / CMP #$08 / BNE A048              ; 触发源(b)：stage==8 亦落入 A032
A032: LDA #$33 / JSR $ED3B                          ; 音效 $33
A037: LDA $32 / ORA #$20 / STA $32                  ; $32 bit5 置位（登记不解释）
A03D: text $A65C / JSR $A09F                        ; 消息 + ack 等待（$A0CF）
A048: text $A31F / JSR $E6D9 / LDA #$03 / JSR $EE03 / JMP $EEFB   ; 两路在此合流转场
```

`f0 07` 目标 = A032、`d0 16` 目标 = A048，编码无歧义 ⇒ **`(pad2&0x33)==0x11 || stage==8` 的 OR 语义**。
被审方三件套一致地只接了 (a)：

- `docs/spec/shop-purchase.md` R-7：「隐藏选项门 `($18&$33)==$11` ⇒ `$32` bit5 置位」——
  **声称"本轮 seed 复核 `bank01:$A023-$A058`"，该范围完整覆盖 A02B-A031 七字节**，
  但规则文本与证据描述均漏 (b)。证据指针"支持"该条的**方向**（(a) 确实触发），但**不支持其完整性**——
  R-7 作为"已证行为"记述不合格（这就是焦点①问题"每条的证据指针真的支持该条吗"的反例）。
- `ui.rs` L375：`if st.pad2 & 0x33 == 0x11 { st.pickup.flags |= 0x20; }` ——缺 `|| stage==8`。
- `tests/shop_purchase.rs::exit_entry_and_hidden_gate`：三个子场景 `stage` 均默认 0，
  ROM 在 stage=0 时 (b) 也不触发 ⇒ 测试全绿但**对 (b) 分支零覆盖**，属"测试真验行为"面的盲区
  （不是自证问题，是覆盖缺口）。

**分歧态**：`stage=8, pad2=0x00` ⇒ ROM 置 `$32` bit5，实现不置。可复现、稳定、非平台差异。
分类：**移植错（已证行为漏接）+ 规格缺口（R-7 记述不全）**。修法见 §7。

### 1.3 AR65-2 现场：隐藏门触发路径的 ack 等待未建模

ROM 中 (a)/(b) 任一触发后都走 `JSR $A09F`（内含 `$A0CF` ack 轮询，A/B 才放行）才合流转场；
ui.rs 的 exit 分支置位后**直接 `phase=Exited`**，无消息相位。既然 `$32` bit5 写已被实现+测试主张为
已接行为（commit message 亦称"退出+隐藏门"），同一处理链上的输入阻塞（ack 等待）被静默略过
属行为时序分歧。文案 `$A65C` 内容仍未证——按本线既定纪律可落**第 7 个 prompt 变体**（状态枚举），
不发明字符串。分类：移植错（低）。与 AR65-1 同批修。

### 1.4 其余"未发明"面核验（全过）

| 检查 | 结果 |
|---|---|
| 消息文案 | `ShopPrompt` 纯枚举 ✅；测试只断言枚举值 ✅ |
| 音效 `$43/$44/$33`、文本渲染 `$A06C/$A091/$A09F/E6D9` | 未落 ✅（代码注释显式声明"渲染线不在此落"） |
| `$92C5` 翼收尾写集 | 未发明 ✅：`ShopFx::WingBoughtFarCallPending` 显式挂起 + 测试断言挂起标记 |
| W-NEW5（`$0177+Y` vs `$0178+Y` 基差） | fail-closed ✅：`ship_index >= delivery.len()` 双守卫（选槽/购买两处），test 无越界主张 |
| W-NEW1（ShopState 门控未证） | 零混接 ✅（§4） |
| 武器生效 = `weapons::pickup_handler`（far-call `$922B` 同例程） | ✅ 复用既有 t39 定谳模块（pk-* L2 面已裁），非重写；测试显式设 `pickup.threshold=0x00` 并注"shop 期值未证——不发明默认"——**文档化未知、不藏默认**，纪律良好 |
| 转场 `$EE03/$EEFB` | 显式交 flow ✅（本线不发明） |

## 2. 焦点②测试真验行为（抽 7 条逐行 + 4 项真 mutation 实测）

### 2.1 抽样逐行（全部断言 RAM 终态/外部可观察状态，无自证式断言）

| 测试 | 断言对象（逐行核对） | 判定 |
|---|---|---|
| `main_cursor_wraps_0_to_3` | `$0400` 序列 0→3→2→3→0 + 零输入 `assert_eq!(s, before)` **全结构不变**（零写入） | 终态 ✅ |
| `insufficient_funds_never_writes` | 三价 9/5/3 各断言 `prompt==InsufficientFunds`、`money` 不变、`$33/$34/$3C` 不动、`delivery==[0;8]` 零写 | 终态+负向 ✅ |
| `weapon_level_cap_branch` | `money=50` 不扣、`$33=7` 不动、ack 后 `phase=WeaponMenu` 且 `item_cursor:=$34`（对 ROM `JMP $9D5E` 重入语义） | 终态 ✅ |
| `ship_buy_packs_delivery_slot` | `$0401/$0402/$0403` 三者同写、`$0403` 跟踪、旧弹种 `(0x45>>6)&3=1`、终态 `delivery[4]==0x86`、`money=16`、购后停留弹种环 | 终态 ✅（期望值与 ROM 位运算逐位复核一致） |
| `exit_entry_and_hidden_gate` | `phase=Exited`、`money` 不变、门满足 `$32|=0x20`、门不满足不动 | 终态 ✅ 但 **stage 维度零覆盖**（AR65-1 盲区） |
| `rapid_buy_and_level_cap` | `$3C` 3→4、money−6、==4 触 `RapidMax`、ack 回主环 cursor=0、再入 `≥4` 直接消息不扣钱 | 终态 ✅ |
| `shop_purchase_negative_selfproof` | 三条 `assert_ne!` 哨兵（不足误扣/满级先扣/打包错位）+ `qty` 回绕哨兵 | 活哨兵 ✅（与正向断言冗余，无害） |

### 2.2 Mutation 实测（超出"想象"，本审真改真跑真还原）

改实现→只跑 `cargo test -p game-core --test shop_purchase`→`git checkout` 还原（完毕核对 `git status` 干净）：

| # | 变异 | 结果 | 命中的断言 |
|---|---|---|---|
| M1 | `l2ports::digit_conversion` 扣减值 10→**9** | **3 FAILED**（exit=101） | `weapon_buy_same_type_levels_up`（`期望 $018E=20−10`）、`weapon_buy_switches_type`、`wing_buy_sets_bit7…`（`期望 $018E=30−10`） |
| M2 | 打包 `((item&3)<<6)\|qty` → `(item&3)\|qty`（字段错位） | **2 FAILED** | `ship_buy_packs_delivery_slot`（`期望 (2<<6)\|6=0x86`）、`ship_qty_caps_at_63`（`期望 0xFF`） |
| M3 | 隐藏门常数 `$11`→`$21` | **1 FAILED** | `exit_entry_and_hidden_gate`（`期望 $32 bit5`） |
| M4 | `shop_ship_qty_next` 封顶 63→64 | **1 FAILED** | `ship_qty_caps_at_63`（含负向哨兵 `qty 不得回绕 64→0`） |

结论：被审主张的"负向哨兵必红"**实测成立**，且正向终态断言本身已足够拦截扣减值/打包/门常数/封顶四类变异。

### 2.3 TDD 叙述（RED=169 E0425）

测试与实现同 commit 落地，RED 期无独立痕迹（无日志工件/无分步 commit）⇒ **不可独立复现**；
但 (a) E0425（`cannot find function`）对"API 未实现先写测试"的编译面是合理错误族，
(b) 14 条测试引用的全部符号在 96450a0 之前不存在（`git show 96450a0^:crates/game-core/src/ui.rs` 无 `shop_purchase_step`），
(c) 122 基线计数与 14 条新增严丝合缝。**叙述可信、证据不全**，记 AD65-2（建议：TDD 主张附 RED 输出留档）。

## 3. 焦点③对拍独立复跑（三件 + 调用链抽验，全部本审实跑贴数）

| 项 | 命令 | 本审实际输出 |
|---|---|---|
| 全量测试 | `cargo test --workspace` | **TOTAL PASSED: 122**（14 个套件逐个 `test result: ok.`，shop_purchase 套件 `running 14 tests … 14 passed`；0 failed） |
| L2 三向差分 | `cargo run --release -p game-oracle --bin l2-diff` | **L2-DIFF: 109/109 通过（golden==ROM 执行==game-core 同名函数，三者缺一不可）**（含 sp-r*/sp-s*/sq-*/se-*/digit-* 商店族逐条 [ok]） |
| 总闸 | `python3 gates/cli.py check` | **SUMMARY: PASS=12 FAIL=0 SKIP=0 TOLERATED=0**（`ratchet:rust_test_count 基线 122 / 现值 122 —— OK`；SKIP/TOLERATED 显式打印为空） |

### 3.1 l2ports 调用链抽验（`shop_spend_ship`，发货扣款价 4）

```
ui.rs:525   let (money, ok) = crate::l2ports::shop_spend_ship(st.money);        ← 唯一扣款实现点
l2ports.rs:159  pub fn shop_spend_ship(money: u8) -> (u8, bool)                ← 签名一致（u8→(u8,bool)）
l2.rs:708   "shop_spend_ship" => { … l2ports::shop_spend_ship(ram(0x018E)) … }  ← 差分臂在位
findings §5 用例 sp-s1-deduct13 / sp-s2-poor3                                   ← 三向对拍覆盖该片段
ROM bank00:$9FAB `ad 8e 01 c9 04 b0 03 4c 5b a0 e9 04 8d 8e 01`                 ← 本审直读复核语义
```

旁路检查：`grep -nE "money *[-+]=|& *0x3F *\+ *1|overflowing_add\(3\)|- *10|- *6\b|- *4\b" ui.rs` →
**零内联扣减/封顶算术**（命中均为注释）；ui.rs 对四函数
（`digit_conversion`/`shop_spend_rapid`/`shop_spend_ship`/`shop_ship_qty_next`/`shop_money_earn`）
+ `weapons::pickup_handler` 全部**委托调用**，无任何自写副本 ✅。

## 4. 架构合规 + 基线留痕

| 项 | 本审证据 | 判定 |
|---|---|---|
| game-core 纯逻辑无 I/O | `grep -rnE "std::fs\|std::net\|std::time\|SystemTime\|Instant\|rand::\|env::\|std::process" crates/game-core/src/` → **src 零命中**（tests/ 下 `std::fs::read` 属测试宿主读 ROM fixture；`contract.rs` 即此红线的门禁实现） | ✅ |
| ShopState 未触碰/零混接 | `96450a0 --numstat`：ui.rs **403/0 纯追加**；`ShopState`（L94-127）与 `ShopPurchaseState` 零互引；`shop_purchase_step` 全仓唯一调用面=本线测试 | ✅ |
| 确定性 | 状态机纯函数、无时钟/随机；`ShopPurchaseState::default()` 可复现 | ✅ |
| 基线留痕 | `baseline_history.jsonl`（2ee1060 条目）reason 含 **"确认无回退：cargo test --workspace 122/122 + l2-diff 109/109 + gates 12/12；基线上调锁定新增测试面（只升不降）"**；`baseline.json` 108→122 单调上调；SHA256SUMS 链同步 | ✅ |

## 5. findings 四分类路由

| 编号 | 分类 | 级 | 位置 | 内容 |
|---|---|---|---|---|
| **AR65-1** | 移植错（已证行为漏接）+ 规格缺口 | **中·必修** | `ui.rs` L375 / `spec` R-7 / `tests` exit 用例 | 隐藏门两触发源只接 (a)：ROM `($18&$33)==$11 \|\| $0180==8`（bank01:$A023-$A03B，`f0 07`/`d0 16` 双分支直读），`stage==8` 分支缺实现、缺 spec、缺测试；spec 声称的 `$A023-$A058` 逐字节复核范围覆盖该七字节却漏记——证据指针不能支持 R-7 的完整性 |
| **AR65-2** | 移植错（行为时序） | 低·必修 | 同上 | 隐藏门触发后 ROM 有 `$A65C` 消息 + `$A0CF` ack 等待才转场，ui.rs 直接 Exited；文案未证可落 prompt 第 7 变体（枚举不发明） |
| AD65-1 | 文档（证据指针精度） | 低·可选 | `spec` R-3 | "回主环复位 `bank00:$9CEC` `LDA #0/STA $400`"——实为 `$9CF7/$9CF9`（$9CEC 是序列入口：文本 $A2AB→渲染→复位）；行为主张正确、指针偏 ~0x0B；测试注释反而写对（`bank00:$9CF7 STA $400`） |
| AD65-2 | 纪律（TDD 留痕） | 低·可选 | commit 96450a0 | "RED=169 E0425" 自述无工件可复现（同 commit 落地）；建议 RED 输出留档 `analysis/` 或 commit 附件 |
| AD65-3 | 文档（跨模块语义注） | 低·可选 | `weapons.rs` PickupContext / ui.rs | `$32` bit7 双注释并存（"kernel 门" vs "翼已购"）——两者都是 ROM 真读者（$922B dispatch 与 shop 置位），建议互注引用防止语义漂移；bit5 "登记不解释" 处理正确 |

## 6. 修正建议（收敛判据）

1. **AR65-1**：`ui.rs` exit 分支改为 `if st.pad2 & 0x33 == 0x11 || st.stage == 8 { st.pickup.flags |= 0x20; … }`
   （两触发源 OR，以 ROM 为准）；`spec/shop-purchase.md` R-7 规则与证据句按 §1.2 控制流改写（注明 dated 更正）；
   `exit_entry_and_hidden_gate` 补 `stage=8, pad2=0x00` 子场景（期望 `$32|=0x20`）+ `stage=8, pad2=0x11` 不双写。
   验收：新增子场景 RED→GREEN，`cargo test --workspace` ≥123，l2-diff 109/109 不动。
2. **AR65-2**：隐藏门触发路径落 `ShopPrompt` 第 7 变体（如 `ExitHidden`，指针 `$A65C` 登记不解释）+ Message 相位
   ack 后再 Exited；测试补 ack 前不转场/ack 后转场两断言。
3. AD65-1/3 随修：R-3 指针改 `$9CF7/$9CF9`（dated）；`$32` bit 注释互注。
4. AD65-2 流程性：后续 TDD 任务留 RED 日志。

## 7. 验收对照（任务判据）

| 判据 | 结果 |
|---|---|
| ① 未发明核查（spec 逐条 + 自采证据重导） | **逐条**：R-1/2/3/4/5/6/8/9/10 ✅（10 组 ROM 直读重导，6 消息指针、位运算逐位、far-call 保活 A 全坐实）；**R-7 ❌**（AR65-1 两触发源漏一、AR65-2 ack 等待漏）；自采证据重导 3/4 成立（隐藏门发现漏支） |
| ② 测试抽验（≥5 条 + mutation） | ✅ **7 条逐行**（终态断言、无自证）+ **4 项真 mutation 实测全红**（改后还原、tree 干净）+ TDD 叙述可信缺工件（AD65-2） |
| ③ 对拍三件独立复跑贴数 | ✅ **122/122**、**109/109**、**gates 12 PASS/0 FAIL/0 SKIP/0 TOLERATED**（§3 均为本审实跑输出）+ l2ports 调用链抽验无旁路 |
| ④ verdict 明确 | **needs_revision**（AR65-1 中危必修 + AR65-2 低危必修；焦点②③全过、未发明面除 R-7 外干净） |

> 复现：§1.1 hex 脚本（bank 映射 `bank*0x2000+(cpu&0x1FFF)`，bank15 窗另行；直读输出与
> `reference/seed/gunnac.asm` 对照一致）、§2.2 mutation 四连（`git checkout --` 还原）、§3 三条命令。
> ROM 全程只读未改（`rom-integrity` 门禁 PASS 佐证）。
