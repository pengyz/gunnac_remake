# spec: 死亡 / GAME OVER / Continue 链（game-over-continue）→ docs/spec/game-over-continue.md

> 状态：2026-10-14 v1（任务 DEV-2，porter）。证据基准 = [../findings/game-over-continue.md](../findings/game-over-continue.md)
> （A2 定谳 + AR63/AD66 系列更正后终态，§1 死亡语义链 / §2 GAME OVER 触发链 / §3 Continue 机制 /
> §4 边集建议表 F1-F13）。纪律（AGENTS.md §3）：**只收录 A/B 级证据支撑的可观察行为**；
> 未证不发明（F13 免死支、`$26` bit7 直回标题支=C 级 §3.4 ⇒ 暂挂，不入行为断言）；
> 游戏环显示/音效 ⇒ 挂起枚举（`DeathFx`），不发明写集/渲染。
> 实现锚：`crates/game-core/src/flow.rs::death_chain_step`（`DeathChainState`，纯逻辑、确定性、无 I/O；
> 粗粒度投影 `FlowState::on_game_over`/`on_continue` + `Mode::GameOverPending`/`GameOverMenu`）；
> 结算算术锚：`crates/game-core/src/l2ports.rs::death_money_halve`（新增，bank03:A39F）+
> `bcd_sub_ammo`（既有，bank15:ED03 同例程）。
> 测试锚：`crates/game-core/tests/death_chain.rs`（10 用例，每用例注记期望 RAM 终态）。
> 与 [ui-flow.md](ui-flow.md) 的场景级流程序列（R-ui-6/7）**分层**：粗粒度 `Mode` 边=本文件 F 边的
> 模式投影，两层口径以细粒度 `death_chain_step` 为准（ui.rs ShopState/ShopPurchaseState 同型先例）。

## 0. 范围与 RAM 契约

| RAM | 台账名 | 语义（已证） | 证据 |
|---|---|---|---|
| `$0400` | 对象网格 slot0 | 玩家态：1=ALIVE / 0x0F=DEAD / 0=释放 | findings §0-§1；`$D86B[1]=0x0F` |
| `$041C`/`$0438` | 状态向量 | 自向量 `$A3:$A5`（trampoline `bank14:$CB4F` 装入） | findings §1.3 |
| `$04A8` | 旧态备份 | 转移写点 `bank14:$D625` | findings §1.1 |
| `$0668` | 命中标志 | bit4=`$10`（`bank14:$D64C-D651 ORA #$10`）；释放清零 `$D11A` | findings §1.1/§1.4 |
| `$06FE-$07FD` | 对象网格尾段 | 续关清零域（`$7FE-$7FF` **不清**） | findings §3.2（AR63-5/AD66-1） |
| `$0710` | （语义开放 R-G3） | 复活写 1（`bank15:$E2B9`） | findings §1.4 |
| `$018D` | 命数 | BCD，stage 开局 :=2；每次死亡 −1（终死跳过） | findings §2.1 |
| `$018E` | 商店金钱 | 0..255 二进制字节；死亡结算 ÷2 | findings §1.3（A2 归因终局） |
| `$E0-$E6` | money_counter_bcd（N-0028） | 7 位 BCD 计数器；续关清零（sub_ECBF） | findings §3.2（AR63-1 更正） |
| `$26` | （标志字节） | bit3=不可复活位；bit4=关末门（ending-sequence）；bit7=模式位（C 级） | findings §2.1/§3.4 |
| `$2F` | 菜单光标 | 0=CONTINUE / 1=GAME OVER | findings §2.4 |
| `$018F` | bomb_ammo（N-0017） | 复活 `max(4,·)`；不在 F12 复位集 | findings §1.4 |
| `$017A-$017F` | （语义开放 R-G1） | 菜单入口逐字节减半；续关带入、标题路径清零 | findings §2.5/§3.2 |
| `$180` | 关口号 | 续关保留（仅此项跨续关保留） | findings §3.1/§3.2 |
| `$188` | stage_intro_counter（N-0200） | GAME OVER 入场 :=$FF；续关 :=$40 | findings §2.2/§3.2 |
| `$0189`/`$0190` | （$0190=bomb_type_index N-0030） | 续关清零 | findings §3.2 |
| `$27` | 一次性入场标志 | bit6=GAME OVER 入场已过（`$E2C1 ORA #$40`） | findings §2.2 |
| `$0300` | 关末推进闸（ending-sequence §6） | 非零=动画完成事件；F8 次触发源 | findings §2.2 |
| `$C8` | （语义未定谳） | 续关清零（登记不解释） | findings §3.2 |

**按键位（菜单域，A 级 `bank15:$E43A-$E456`）**：bit4=Start（确认）、bit5=Select（翻转）、
bit3=Up（仅 `$2F=1` 翻转）、bit2=Down（仅 `$2F=0` 翻转）。**与 ui.rs 商店域
`BTN_DOWN=0x24`/`BTN_SELECT=0xD0` 分域，禁止混用。**

**字节级复现（A 级，可复跑）**：

```bash
python3 - <<'PY'
rom=open('rom/gunnac_cn.nes','rb').read(); prg=rom[0x10:0x10+0x20000]
def show(bank,cpu,n):
    o=bank*0x2000+(cpu&0x1FFF); print(f"bank{bank:02d}:{cpu:04X}", prg[o:o+n].hex(' '))
for args in [(3,0xA382,0x24),(3,0xA39F,4),(14,0xD613,0x1C),(14,0xD10C,0x12),
             (15,0xE29A,0x59),(15,0xE3B9,0x15),(15,0xE425,0x60),(15,0xE0EF,0x1A),
             (15,0xE160,0x9),(15,0xE18C,0x10),(15,0xE1B6,0x12),(15,0xED03,0x24),(15,0xECBF,0xB)]:
    show(*args)
PY
```

## 1. 行为规则

### R-death-trigger: 碰撞命中（转移表 `$D86B[1]=0x0F`）⇒ 四写签名：`$0400:=0x0F`、`$041C:=0`、`$04A8:=旧态`、`$0668|=$10`
- 证据: findings §1.1（转移应用点 `bank14:$D613-D62E`/`$D654-D673` 逐指令 + 四写签名 trace 表：
  f1787/f2046 双事件 `$0400/$041C/$04A8/$0668` 逐一吻合）；§1.2 `$D86B` 全表（CN==JP 全等）
- 可判定性: 机器可判定（`tests/death_chain.rs::f1_death_transition_four_write_signature`；hex 复现第 3 行）
- 关联: F1、N/A（对象网格 slot0）、R-death-immune、R-death-cost

### R-death-immune: 死亡窗内复撞无效——`$D86B[0x0F]=0x0F` 恒等 ⇒ 转移段零写入（防重入）
- 证据: findings §1.2（表值恒等 + `D619-D61F` `CMP/BEQ` 短路）；§1.3 窗内 128f `$0400` 恒 0x0F
- 可判定性: 机器可判定（`f2_window_freeze_immune_and_frame_count`：逐帧复撞断言 `$04A8/$0668` 不重写）
- 关联: F2、R-death-trigger

### R-death-cost: dead 态**首帧**一次性结算（非复活边）——终死（`$018D==0`）⇒ `$26|=8` 且跳过减一；否则 `$018D` BCD −1；`$018E` ÷2 两路同做
- 证据: findings §1.3（`bank03:$A382-A39F` 链：`LDA $18D/BNE`→`$26|=8`/`A390 BNE` 跳过减一/
  `LDY #1/JSR sub_ED03/STA $18D`/`LSR $18E`）+ F5 行（AR63-6 更正：首帧口径，~~复活边~~）；
  动态：ram.hex f1740=2→f1800=1→f2100=0；结算链尾 trampoline f1788 起 `$A3:$A5` 常驻（f1787 仅四写）
- 可判定性: 机器可判定（`f5_death_cost_settles_once_at_dead_first_frame` / `f6_terminal_death_arms_without_decrement`；
  结算算术三方对拍 l2-diff `dmh-*`/`ed3-*`，121/121）
- 关联: F5、F6、N-0017（sub_ED03）、R-gameover-arm
- 注: 任务测试清单"窗末结算"表述以 F5 更正（首帧）为准；测试以"首帧结算+窗末/复活边不再扣"双向断言锁死

### R-death-release: 窗尽释放（128f 后下一帧）——`$0400:=0`、`$041C:=0`、`$0668:=0`（同帧紧接 F4/F7 分叉）
- 证据: findings §1.3（窗长 gameplay-stage1 双事件 128f：f1787-1914/f2046-2173）+ §1.4
  （`bank14:$D10C-D11A` 释放写集）+ trace：f1915/f2174 **同帧** 0F→0→1（watch 只见 0F→1）
- 可判定性: 机器可判定（`f4_respawn_fork_tops_up_bomb_ammo` 断言释放写；`f2_*` 断言 128f 计数）
- 关联: F3、R-respawn、R-gameover-enter、R-G5（计时源开放 ⇒ `window_len` 参数化）
- 注: 同块 `$0470/$048C/$04C4`（亚像素/命中盒）清零属移动域，本链不镜像

### R-respawn: 复活分叉（`$26&8==0`）——`$018F := max(4,$018F)`、`$0400:=1`、`$0710:=1`
- 证据: findings §1.4（`bank15:$E2A2-E2BC`：`LDA $18F/CMP #4/BCS/ LDA #4`、`STA $400`、`STA $710`）
  + trace：f1915 ammo 1→4、f2174 3→4（max 两样本）、`$0710` 0→1
- 可判定性: 机器可判定（`f4_respawn_fork_tops_up_bomb_ammo`：ammo<4 回满 + ammo≥4 保留双分支）
- 关联: F4、R-death-release、R-G3（`$0710` 语义开放）
- 注: 粗粒度 `PersistData::on_respawn`（R-respawn-4）为场景段级近似（:=4），细粒度以本条 `max` 为准

### R-gameover-arm: 终死 arm——`$018D==0` 时死亡 ⇒ `$26|=$08`（不可复活位），死亡窗照常走完
- 证据: findings §2.1（`bank03:$A387-A38E`：`JSR sub_ED56/ LDA $26/ ORA #8/ STA $26`）+
  §2.6 负结果（两次死亡后 `$18D=0` 仍复活；第三次死才进 GAME OVER）
- 可判定性: 机器可判定（`f6_terminal_death_arms_without_decrement`）
- 关联: F6、R-death-cost、R-gameover-enter

### R-gameover-enter: GAME OVER 入场（`$26&8≠0` 分叉）——一次性 `$27|=$40`、`$188:=$FF`、`$0300:=0`、音效 `$11`；此后等待 Start/`$0300≠0`
- 证据: findings §2.2（`bank15:$E2BF-E2F0` 逐指令：`$27` bit6 一次性短路（E2C3 `CMP $27`）、
  `$188:=$FF`（E2D2）、`$0300:=0`（E2D7）、`LDA #$11/JSR $ED3B`（E2DD）、`$16&$10`/`$0300≠0`（E2E2-E2EB）、
  否则停留（E2ED））
- 可判定性: 机器可判定（`f7_gameover_fork_menu_entry`：入场写集 + 无 Start 不转场）
- 关联: F7、R-gameover-arm、R-gameover-menu
- 注: E2C9 `$26` bit7 → `loc_E073` 直回标题支 = **C 级语义（§3.4），未接线**（登记不解释）

### R-gameover-menu: 菜单装载（F8）——`$017A-$017F` 逐字节减半 + `$2F:=0` + 文本流 `$E4A0`（CONTINUE/GAME OVER）
- 证据: findings §2.4（`bank15:$E3AB-E422` 场景装载）+ §2.3（文本记录 `[ppu_lo ppu_hi] ASCII 0xFE…0xFF`，
  流基址 `$E4A0`，CONTINUE=0x1E4de / GAME OVER=0x1e4e9；CN==US 全等）+ §2.5（减半变换 `E3B9-E3CD`）
- 可判定性: 机器可判定（`f7_gameover_fork_menu_entry`：`$2F:=0` + 减半值断言）；文本内容=字节级可查（§2.3）
- 关联: F8、R-gameover-entry-transform、R-menu-toggle

### R-gameover-entry-transform: `$017A-$017F` 入口变换——逐字节 `结果 = (b & $C0) | ((b >> 1) & $1F)`（保 bit7-6；6 位量÷2 存 5 位：丢 bit0、bit5→bit4）
- 证据: findings §2.5（`bank15:$E3B9-E3CD` 逐指令 + AR63-7 位级更正）；全 ROM 该字段仅 3 触点（E0D3 清/E3BB 读/E3C9 写）
- 可判定性: 机器可判定（`f7_gameover_fork_menu_entry` 断言 6 字节减半值；`f12_continue_reset_set` 断言带入）
- 关联: R-gameover-menu、R-continue、R-gameover-finish、R-G1（语义开放：疑发货槽 2-7 数量减半）

### R-menu-toggle: 菜单翻转（F11）——Select 无条件 / Up 仅 `$2F=1` / Down 仅 `$2F=0` ⇒ `$2F^=1` + 音效 `$45`
- 证据: findings §2.4（`bank15:$E43E-E460` 逐指令：`AND #$20/#$08/#$04` + `$2F` 守卫分支 + `EOR #1` + 音效）
- 可判定性: 机器可判定（`f11_menu_toggle_flips_cursor`：四方向×光标双态全组合）
- 关联: F11、R-gameover-menu

### R-continue: 续关出口（F9）——`$2F==0`+Start ⇒ `bank15:$E481 JMP sub_E0D9`（复位集 R-continue-reset）→ ALIVE 关首
- 证据: findings §3.1（双出口表）+ §2.4（`E47D LDA $2F/BNE E484`、`E481 JMP sub_E0D9`）
- 可判定性: 机器可判定（`f9_f10_menu_dual_exit` (a)）
- 关联: F9、R-continue-reset

### R-gameover-finish: 结束出口（F10）——`$2F==1`+Start ⇒ `bank15:$E484 JMP loc_E073`：`$180:=1`、`$017A-$017F` 清零
- 证据: findings §3.1/§3.3（`bank15:$E073-E0D7`：`$180:=1`（E09A-E09C）、`$17A-$17F` 清零（E0D1-E0D7））
- 可判定性: 机器可判定（`f9_f10_menu_dual_exit` (b)）
- 关联: F10、R-gameover-entry-transform
- 注: 同块 `$32:=$40`/`$3C:=0`/`$F1/$4E` 复位属 shop/未证域字段（不在本链镜像）——登记不解释

### R-continue-reset: 续关复位集（F12）——`$18D:=2`、`$189/$190/$18E:=0`、`$E0-$E6:=0`、网格清 `$0400-$06FF`+`$6FE-$7FD`、`$0400:=1`、`$27/$C8:=0`、`$188:=$40`、`$26 两写点序（E0F4 瞬态 `($26&$F7)|$10` → E1BB 终态 `$26&=$80`）**净效应 $26&=$80**`；**仅 `$180` 保留**（`$017A-$017F` 减半值带入）
- 证据: findings §3.2 复位范围表（E0EF/E160-E168/E18C/E19B/E1B6/E1C1-E1C7/E1DD）+
  §2.1（`$26` 复活权重置 E0F4-E0F8）+ AR63-1 更正（`$E0-$E6` **不保留**：`E10A JSR sub_ECBF` 双金钱清零）+
  AR63-5/AD66-1（清零粒度=四页 X=0..255，`$6FE-$7FD`；**`$7FE-$7FF` 不清**）
- 可判定性: 机器可判定（`f12_continue_reset_set`：双金钱清零 + 网格边界（$7FE/$7FF 保留哨兵）+ 仅 `$180` 保留）
- 关联: F12、R-continue、R-death-cost
- 注 1: 同段 `bank15:$E1BB-E1BF` `$26:=$26&$80` 为 A2 §3.2 未列写点——**规格缺口**（已上报 spec-writer），本实现按 A2 复位集闭合，未接 E1BB
- 注 2: `$16/$18` 清（E1C5-E1C7）= 输入锁存写；模型按步传参无锁存 ⇒ 不镜像

### R-death-bypass（暂挂，F13）: 免死支——`$06A0≠0`（stage-clear 保护，`bank03:$A33E`）或 `$32` bit7（速爆+立即复活，`A34B-A36B`）⇒ 跳过结算立即复活
- 证据: findings §1.3（`bank03:$A33E-A36B`）+ §3.4（模式位语义 C 级，未定谳）
- 状态: **暂挂 fail-closed**（findings §4.2 建议"先以 fail-closed 标注待解、不入行为断言"）——未接线
- 关联: F13、R-G2（模式位残项归 game-action-mode 域）

## 2. 显式负结果与暂挂项

- **GAME OVER 屏动态未观测**（findings §2.6）：现有双版 trace 场景段未达 GAME OVER 屏——菜单/续关为
  A 级**静态链**定谳，动态证据止于 `$26|=8` 前一步；`event_from_tag` 不映射 GAME OVER 链标签（防发明动态锚）。
- **R-G1**：`$017A-$017F` 语义（疑发货槽 2-7 数量减半）= C 级开放。
- **R-G2**：`$26` bit7 / `$32` bit7 / `$06A0` 模式位语义 = C 级开放（F13/R-death-bypass 暂挂）。
- **R-G3**：`$0710` 字段语义开放（写点已证）。
- **R-G5**：死亡窗长计时源（`$CD9D` 相位 vs `$05DC` 索引饱和）= B 级开放；窗长随事件源变化
  （85f/129f）⇒ `window_len` 参数化，128=gameplay-stage1 实测口径。
- **E1BB 缺口**：`bank15:$E1BB-E1BF` `$26:=$26&$80`（A2 §3.2 未列，见 R-continue-reset 注 1）。
- 游戏环显示/音效：`DeathFx` 挂起枚举（死亡 $21 / GAME OVER $11 / 菜单 $45 / 文本流 $E4A0），不发明渲染。

## 3. 门禁

- `python3 gates/cli.py check` → 见任务验收输出（12/12）。
- L2 三方对拍：`python3 tools/diff_subroutines.py` → **121/121**（109+12：`dmh-*`×6、`ed3-*`×6）；
  JP 集同步 121/121（`--rom reference/jp/gunnac_jp.nes`；bank03:A39F 双版同址 Δ0，sub_ED03 JP=$ED0D +0x0A）。

## spec 产出契约

- 规则清单: R-death-trigger / R-death-immune / R-death-cost / R-death-release / R-respawn /
  R-gameover-arm / R-gameover-enter / R-gameover-menu / R-gameover-entry-transform / R-menu-toggle /
  R-continue / R-gameover-finish / R-continue-reset（13 条，逐条带证据指针）+ R-death-bypass（暂挂）
- 常量对照: 本模块无新常量（`DEATH_WINDOW_FRAMES=128`=实测口径非 ROM 常量；结算算术见
  [constants.md](constants.md) 既有条目）
- 可判定性标注: 13 条机器可判定；R-death-bypass 暂挂（不入行为断言）
- 门禁: `cargo test --workspace` 135 绿 / l2-diff 121/121（CN+JP）/ gates 12/12
- 遗留假设: R-G1、R-G2、R-G3、R-G5（C/B 级）；E1BB 规格缺口（上报 spec-writer）；§2.6 动态负结果
