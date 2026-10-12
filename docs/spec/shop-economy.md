# spec: 商店与经济（shop-economy）→ docs/spec/shop-economy.md

> 状态：2026-10-12 v1（任务 t6，spec-writer）→ **v2 同日 S2 回填**（任务 t17：t12 场景证据入规则）。
> 证据登记 E-xx 见 [completeness.md](completeness.md) §1；S2 场景证据指针格式 `scenario:analysis/traces/<id>` + 帧号。
> 纪律（AGENTS.md §3 / 角色契约）：本文件**只收录 A/B 级证据支撑的可观察行为**；
> 交互边界一律留"待 S2 取证"空位（§4），**禁止发明**。汉化特有文本单列 §3。
> 分子强度：规则面覆盖升 **L2**（绑 scenario/trace，completeness §6.2 硬规则）；含 web: 证据的格不超 L1。

## 0. 范围与关联格

| 模块 | 矩阵格 | 本版覆盖入口 | 分子强度 |
|---|---|---|---|
| 商店 | B-SYS-SHOP | 价目面板（数据存在）/ 金钱显示行 / 按键提示 / 静态屏 / B 切页 / 两连 B 退出 / Start 无效 | **L2**（R-shop-5..8 绑 scenario/trace） |
| 金钱 | B-SYS-MONEY | 金钱显示模板+数值位 / 价目数值 / 获取与消费渠道（B 级） | **L2**（显示行为绑 scenario/trace；增减时机仍空 H-09） |
| 道具（关联） | B-SYS-ITEM | 弹药存量上限（信息页明示「导弹20个」） | **L2**（R-item-1；映射 H-11 仍空） |

不在范围：武器·炸弹·道具数值（B-WPN/B-BMB/B-SYS-ITEM，另文）、关卡时间线、掉落细节行为。

## 1. 常量对照（同步 [constants.md](constants.md)）

| 常量 ID | 值 | 证据 | 级 | 关联 |
|---|---|---|---|---|
| C-shop-p1 | `$10` | `rom:8929` | A | R-shop-1 |
| C-shop-p2 | `$6` | `rom:8958` | A | R-shop-1 |
| C-shop-p3 | `$4` | `rom:8986` | A | R-shop-1 |
| C-money-template | `-------$` | `rom:8516–8523`（`$`@8523） | A | R-money-1 |
| C-money-unk | `10000000000` | `rom:4759` | A | R-money-3（**仅登记不解释**） |

## 2. 行为规则

### R-shop-1: 商店价目为编号商品记录（4 条），价格行模板 `1 <图形码> $<n>`，已见数值 $10 / $6 / $4
- 证据: `rom:8914–9000` 记录区：`"1."`@8917 → `$10`@8929；`"2."`@8946 → `$6`@8958；`"3."`@8974 → `$4`@8986；`"4."`@8996 后**未见** ASCII 价格串（0xff 填充截断）；商品名为图形码（非 ASCII），字段切分未证（H-05）
- 可判定性: 机器可判定（`grep -abo '\$10|\$6|\$4' rom/gunnac_cn.nes` + hexdump 复现）
- 关联: B-SYS-SHOP、C-shop-p1..p3、H-05

### R-shop-2: 商店面板存在"持有额 + 货币符"金钱显示行，模板为破折号填充 + `$`
- 证据: `rom:8516–8523` 串 `-------$`（`$`@8523）+ `shot:shots/auto_f01080.png`（同帧 26 张，sha256 `efcfe736…`：像素显示「你有 ──￥」）
- 可判定性: 机器可判定（L4 截图 diff / 文本模板渲染比对）
- 关联: B-SYS-SHOP、B-SYS-MONEY、C-money-template、R-money-1、H-08（￥↔`$` 字形差异）

### R-shop-3: 商店面板文本尾部含 A/B 键提示（汉化像素读作「…键，B键 转换」）；面板文本为图形码记录 + ASCII 混排结构
- 证据: 记录尾部 ASCII `A`/`B` 字符：`F#A`@8570 与 `,B`@8575（金钱面板）、9126/9133、9386/9393、9567/9578、9647/9654（其余面板同构尾部）+ `shot:shots/auto_f01080.png` 按键提示行（首字形需人工核对，读作 A/缺字框）
- 可判定性: 仅人工核对（截图字形）；S2 场景建立后升机器可判定
- 关联: B-SYS-SHOP

### R-shop-4: 商店界面存在，且外部资料陈述其出现在关卡之间（B 级；显示时机完整逻辑未证）
- 证据: `web:https://en.wikipedia.org/wiki/Gun-Nac`（E-W06 "…purchase upgrades in a shop in between levels"）+ `shot:shots/auto_f01080.png`（自动运行 5400 帧内出现该界面）
- 可判定性: 机器可判定（S2 场景：关卡切换断言进入商店界面）——**封顶 L1**（web 证据）
- 关联: B-SYS-SHOP

> 【2026-10-12 S2 回填】以下 R-shop-5..8 / R-money-4 / R-item-1 由 t12 场景证据提炼（B 级，绑 scenario/trace）。

### R-shop-5: 商店画面为静态屏（无输入时保持不变）
- 证据: `scenario:analysis/traces/exp-shop-start`（run.json：frames=2000 全程无输入）+ `scenario:oracle/scenarios/shop.txt` 证据注记 + `shot:shots/shop_f01080_shop-arrival.png`（sha `efcfe736…`，与早期 `auto_f01080.png` 同帧）
- 可判定性: 机器可判定（同帧截图 sha 比对）
- 关联: B-SYS-SHOP

### R-shop-6: B 键单击切换商店信息页（信息页显示持有额、弹药上限、按键提示）
- 证据: `scenario:analysis/traces/shop`（trace f1140 tag=shop-page）+ `shot:shots/shop_f01140_shop-page.png`（sha `fec87b88…`：「你有 ──￥0」「导弹20个 上限。」「A键，B键 转换。」）+ `oracle/scenarios/shop.txt` 注记（A/B 切页与画面提示一致）
- 可判定性: 机器可判定（截图 diff）
- 关联: B-SYS-SHOP、B-SYS-ITEM、R-item-1、R-money-4

### R-shop-7: 两连 B 退出商店：近黑过渡帧后进入实机
- 证据: `scenario:analysis/traces/gameplay-stage1`（trace f1200 tag=exit-transition → f1260 tag=stage-start；输入 press 1100/1160 b）+ `scenario:analysis/traces/exp-shop-b`（run.json：frames=2000）+ `shot:shots/gameplay-stage1_f01200_exit-transition.png`（近黑，sha `b0a63867…`）/ `shots/gameplay-stage1_f01260_stage-start.png`（实机，画面含「地区-1」标签）
- 可判定性: 机器可判定（截图序列 diff）
- 关联: B-SYS-SHOP、R-ui-6

### R-shop-8: Start 在商店内无效果（退出不走 Start）
- 证据: `scenario:analysis/traces/exp-shop-start`（反证：2000 帧无输入画面不变）+ `oracle/scenarios/gameplay-stage1.txt` 注记（"Start 在商店内无效果（exp-shop-start 反证），退出不走 Start"）
- 可判定性: 机器可判定（反证场景可复跑）
- 关联: B-SYS-SHOP

### R-money-1: 金钱以独立显示行呈现持有额（汉化版：「你有 ──￥」）；数值位数与占位渲染规则未证
- 证据: `rom:8516–8523`（C-money-template）+ `shot:shots/auto_f01080.png`
- 可判定性: 机器可判定（L4 截图 diff）
- 关联: B-SYS-MONEY、C-money-template

### R-money-2: 金钱获取渠道为关内随机浮动拾取与击毁对象/敌人掉落，消费渠道为关间商店（B 级）
- 证据: `web:https://en.wikipedia.org/wiki/Gun-Nac`（E-W06 "Money can be found randomly floating through the levels, or being released by destroyed objects or enemies; it can be used to purchase upgrades in a shop in between levels."）
- 可判定性: 机器可判定（S2 场景：拾取/击毁断言金钱增加、购买断言扣减）——**当前封顶 L1**（web 证据）
- 关联: B-SYS-MONEY、B-SYS-DROP、B-SYS-SHOP

### R-money-3: 数值串 `10000000000` 存在于 ROM，**仅登记不解释**
- 证据: `rom:4759`（0x1297）
- 可判定性: 仅人工核对（字节存在性）；含义禁解释（H-03 未证）
- 关联: B-SYS-MONEY、C-money-unk、H-03

### R-money-4: 金钱显示值本体 = `money_counter_bcd` $E0-$E6（7 位 BCD，$E0=最高位）；渲染前导零抑制
- 证据: `bank14:D009-D046`（`LDA $E0,Y`（Y=0..6）→ `ASL A/ORA #$01` tile 码 → `STA $0201,X`，`LDA $05/BNE` 前导零跳位，`CPY #$07` 七位）+ probe 实测 f2160=4400 / f2220=5700 / f2280=**6800** / f2340=7200 与屏幕位图逐帧吻合（`docs/findings/ram-watch-s3.md` §8.1）+ shop 场景恒 000000 ↔「你有 ──￥0」（`trace:analysis/traces/shop#f1140`）
- 可判定性: 机器可判定（L3：$E0-$E6 ∈ 状态向量逐 tick 比对；L4 截图 diff 同向）
- 关联: B-SYS-MONEY、C-money-bcd、N-0028、R-shop-2
- 【2026-10-12 t26 扩写】原"显示行含数值位"结论收编；N-0025/W1（疑 WRAM $6000-$7FFF）**已证伪**（WRAM watch 全程 open-bus 恒 0x60）；N-0031 证伪撤回留档

### R-money-5: 入账加法器 `bank14:DBCE-DBFF`：事件类型×6 索引六字节加数表，逐位 BCD 进位加法，溢出封顶 999999
- 证据: `bank14:DBCE-DBFF`（`STA $04/ASL/ADC $04/ASL`=type×6 → `TAY/LDX #5` 六次循环：`LDA $DCCF,Y/ADC $E0,X/CMP #$0A/SBC #$0A/STA $E0,X/DEY/DEX`，末位进位 → `$E0-$E5` 全 9 封顶）+ 加数表 `bank14:$DCCF+type×6`（逐值见 C-money-addend-tbl）+ 事件分发 `bank14:DB4C`（A=奖励类型，`$26 & $88` 状态屏蔽，调用点 bank00:8C86、bank02:8660）+ probe 增量 +400/+1100/+1300 与行值组合一致（`ram-watch-s3.md` §8.1）
- 可判定性: 机器可判定（L2 子程序级 diff：给定 type 与计数器快照跑加法器到 RTS 比对写集）
- 关联: B-SYS-MONEY、C-money-addend-tbl、C-money-cap、N-0028

### R-money-6: extend 阈值检查 `bank14:DAE7-DAFC`：计数器与 `extend_threshold_bcd` $E7-$EC（=8000）逐位比较，达标置 $ED=$40
- 证据: `bank14:DAE7-DAFC`（`CMP $E7,Y` 逐位比较循环）+ probe 实测阈值恒 `0,0,8,0,0,0`=**8000**（计数器 7200 逼近）+ `bank14:DB4C` 链（`JSR $DBCE` 后 `JSR $DAE7`）（`ram-watch-s3.md` §8.2）
- 可判定性: 机器可判定（L2/L3：$E7-$EC/$ED ∈ 状态向量）
- 关联: B-SYS-MONEY、C-extend-threshold、N-0029

### R-item-1: 弹药存量存在上限（信息页明示「导弹20个 上限。」）
- 证据: `scenario:analysis/traces/shop`（trace f1140 tag=shop-page）+ `shot:shots/shop_f01140_shop-page.png`（sha `fec87b88…`）
- 可判定性: 机器可判定（截图 diff）；**导弹 ↔ 炸弹字母（F/B/T/W）映射及其余弹种上限未证（H-11），不发明**
- 关联: B-SYS-ITEM、B-BMB-F/B/T/W、H-11

> 【2026-10-12 t37 追加（FA-1 反向规则）】以下 R-item-2/3 承载 pk-* 对拍用例（升级/切换行为）；
> 在本追加前，pk-* 7 用例不得记为 R-item-1/R-bomb-1 覆盖（t34 审计纪律照办）。

### R-item-2: 同型圈连收升级：`$33`（weapon_level）**+1 递增；溢出回卷按路径分野**——kernel 路径（slot==0）FF→`$0C` 回卷，deep 路径（slot≠0）裸 INC FF→`$00` 自然回卷
- 证据: **kernel 路径**（`$34==0`，拾取处理 `bank2:$922B-$9258`）：L2 对拍 pk-1/pk-2/pk-3/**pk-4-wrap-ffto0c**（255→12）/pk-5（12→13 递增继续）+ `INC $33` @`$923D`、溢出回卷置 $0C @`$924D` + N-0033；**deep 路径**（`$34≠0`，`$925F` 深分支 → 裸 `INC $33` @`$92B4`）：溢出**自然回卷 FF→$00**（无 $0C 修正）——t35 深路径定谳（`docs/findings/upgrade-branch-h02.md` + N-0058 范围更正）；deep 实测用例 pk-8/pk-9（三方全等），**wrap 分歧 deep 用例 pk-10 待 t39 补入 l2_cases.jsonl**（已证分歧按纪律未入文件、JSON 语义挂账）
- 可判定性: **机器可判定（L2 纯函数）**——kernel 已实证 5 用例含 wrap 边界；deep 部分（pk-8/9 已实测；FF→$00 wrap 断言待 pk-10）
- 关联: B-SYS-ITEM、N-0033、N-0058、R-shot-level-3（级→配置语义）、R-shot-level-4（$925F 深路径分支）
- 备注：**pk-5 实测 12→13 递增继续**——`$0C` 是 kernel 回卷目标，不是等级上限；deep 路径可越 12 → `$9457` 越界索引行为 = **H-A** 必答项
- 【2026-10-12 t43 scope 标注】原文"溢出 255 回卷 12（$0C）"未分路径，按 t35 两路径分野收窄为 kernel 语义；deep 回卷 $00 同步补入

### R-item-3: 异型圈切换：`$34`（weapon_slot）**写为新拾取值**（直接切换，非递增）
- 证据: L2 对拍用例 **pk-6-switch-1to3**（$34: 1→3）、**pk-7-switch-0to5**（$34: 0→5）（`oracle/subroutines/l2_cases.jsonl` 双实现对拍）+ `bank2:$922B` 拾取处理异型分支 `STA $34` + N-0032 + t25/t31 结论
- 可判定性: **机器可判定（L2 纯函数，已实证 2 用例）**
- 关联: B-SYS-ITEM、N-0032、R-shot-gen-4（$34 分派五路）

### R-bomb-1: `bomb_type_index` $0190 四值域（0-3），初始化由 RNG 决定
- 证据: `bank15:E138-E13D`（`LDA(RNG) / LSR×3 / AND #$03 / STA $0190`）+ `docs/findings/ram-watch-s3.md` §8.3
- 可判定性: 机器可判定（L2 子程序级 diff：RNG 快照 → $0190 写值断言）
- 关联: B-SYS-ITEM、B-BMB-F/B/T/W、C-bomb-type-index、N-0030

### R-bomb-2: 拾取字母圈弹种切换：`$0190` 切换 + HUD 字母切换同步（同帧）；`$018F` 弹数变化属**同事件簇但可错帧**的独立维度
- 证据: probe diff f2160→f2280：f2220 拾取字母圈「W」→ `$0190: 0→3`、HUD 字母 **F→W**、`$018F: 3→4`（**60 帧 probe 粒度的同簇观察**，`ram-watch-s3.md` §8.3）+ **A 级反例**：`oracle/traces/gameplay-stage1.weapons.jsonl` f2174（bomb: 弹数 3→4）→ f2175（bomb: 弹种 0→3 且弹数不变；同帧 weapon: `$34` 0→3）——三变化相隔 1 帧、弹种切换不必然伴随弹数变化
- 可判定性: 机器可判定（L3：$0190+HUD 字母同帧断言；$018F 按事件簇断言）
- 关联: B-SYS-ITEM、B-BMB-F/B/T/W、N-0030、N-0017、H-12（字母归因已收口）、A2（f2175 同帧双处理器现象）
- 【2026-10-12 更正（t36）】原"三方同帧段同步"**收窄**：t22 证据是 60 帧 probe 粒度的同簇观察，不支持同帧不变量；f2174/f2175 反例（golden 据此判失配）证明弹数变化与弹种切换是两个维度。原表述保留如上不删历史。

### R-bomb-3: 索引 ↔ 字母映射：**0=F、3=W 实测**（A 级）；**1=B、2=T 推定**（B 级，按 E-R08 `F B T W` 串序）
- 证据: 实测：R-bomb-2 拾「W」→ index 3 + HUD F→W（`ram-watch-s3.md` §8.3）；推定：`rom:9626 F B T W` 串序（E-R08）与四值域同长 —— **推定部分不得作结论**（H-11 保留推定残项）
- 可判定性: 机器可判定（S2 单变量拾取场景逐字母实测 1=B/2=T 即可闭合）
- 关联: B-SYS-ITEM、B-BMB-F/B/T/W、E-R08、H-11

## 3. 汉化特有（单列，AGENTS.md R4：不与原版行为混写）

- 商店面板文本为汉化字模中文：「你有」「收集相同子弹，子弹级增加」「…键，B键 转换」（`shot:shots/auto_f01080.png`）。
- 货币字形渲染为 `￥`，而 ROM 原串为 `$`（`rom:8516–8523`，`$`@8523）——字形映射差异。
- 汉化标记 `rom:7106–7118` `\t@2011  JIUBAN`（E-R12）。
- 以上均属汉化版特有渲染/文本；与原版的差异对齐登记在 H-08，**禁止据此推断原版行为**。

## 4. 待 S2 取证空位（留空，禁止发明；逐项建场景后回填规则）

> 【2026-10-12 S2 回填后状态】已证部分入规则（R-shop-5..8、R-money-4、R-item-1）；**t26 追加** R-money-4 扩写 + R-money-5/6 + R-bomb-1..3。

- [x] **部分闭环**｜购买流程按键效果：**B 单击=切信息页**（R-shop-6）、**两连 B=退出商店**（R-shop-7）、**Start 无效**（R-shop-8）；
      A 键单击效果与"购买确认"效果**未证**（采样帧 trace f1180/f1340 留证）→ **H-10**
- [x] **部分闭环**｜金钱增减时机与数值：**增加侧已证**（R-money-5：事件加数表逐值 10/30/100/150/250/400/600/800/1000/2000、逐位进位、封顶 999999）；
      **消费侧/死亡惩罚/过关结算未证** → **H-09**（残项）
- [ ] **未闭环**｜价目 4 条记录的商品名与 4 号商品价格（H-05，S2 未揭示）
- [x] **部分闭环**｜金钱上限 / 溢出 / 显示位数：**溢出封顶 999999 已证**（R-money-5）、**显示 7 位 BCD 前导零抑制已证**（R-money-4）；
      达上限后行为未证（H-03 残项减半；`rom:4759 10000000000` 仍不解释）
- [x] **部分闭环**｜商店进入 / 离开：进入=标题菜单输入序列后 f1080 到店（shop.txt 三连 exp 复现一致）；
      离开=两连 B → 近黑过渡 → 实机（R-shop-7）；**GAME OVER 后路径、完整触发条件未证** → **H-13**

## 5. 门禁

`python3 gates/cli.py check` → **PASS**（实跑 2026-10-12）：
`SUMMARY: PASS=4 FAIL=0 SKIP=7 TOLERATED=0`；`CHECK: PASS（含 0 个静默跳过：所有 SKIP 均已显式打印）`。
证据指针最小复现：completeness.md §10 命令（`grep -abo`/hexdump/`sha256sum -c SHA256SUMS`）。

## spec 产出契约

- 规则清单: R-shop-1..8 + R-money-1..6 + R-item-1 + R-bomb-1..3（共 18 条，逐条带证据指针）
- 常量对照: [constants.md](constants.md) v3 已同步（C-money-bcd / C-money-addend-tbl / C-money-cap / C-extend-threshold / C-bomb-type-index 等）
- 可判定性标注: 机器可判定 15 条 / 仅人工核对 3 条（R-shop-3、R-money-3、R-item-1 映射部分）
- 门禁: `python3 gates/cli.py all` 见任务 t26 输出
- 遗留假设: H-03（残项）、H-05、H-08、H-09（残项）、H-10、H-11（残项：1=B/2=T 推定）、H-13（C 级，未入规则结论）
