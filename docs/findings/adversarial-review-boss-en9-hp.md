# 对抗复核：Boss/EN-9+ 定界 + 敌 HP 字段定谳（task-61）—— 证伪立场独立复核

> 复核方：`shell-verifier`（task-64，verifier 角色）；被审：task-61 / `decomp-analyst`
> 被审文件：[boss-en9-hp-evidence.md](boss-en9-hp-evidence.md)（271 行，SHA256 前 16=`8475d54ef91a40c6`，**未提交**；对象 HEAD `2bcb92a`）
> 纪律：[AGENTS.md](../../AGENTS.md) §5（证伪立场；不采信自述；自己读 ROM/seed/代码重导证据）
> **verdict：needs_revision** —— 多数结论（表字节/消费指令/伤害链/`$0400` 写点全集/L2 规格）**独立复算通过**；
> 但**自列必查点 (d) 的核心"唯一性/全集"结论被证伪**：seed 有 61.8% 单字节 raw 行（未解码），xref grep 不完备；
> ROM 字节扫描发现 `$D7ED` 共 **8** 个调用点（非 1）、`$D4FE` 共 **9** 处（非 6）、`STA $7D4,Y` 写点 **2** 处（非 1）、
> `$D674` 计数为 seed 解码下界（raw 区另有 ≥4 处真实调用）。须改口径/结论后再 pass。

## 0. 复核方法 / 边界

- 独立证据源：①`rom/gunnac_cn.nes` 字节（`file = 0x10 + bank*0x2000 + (cpu & 0x1FFF)`）②`reference/seed/gunnac.asm`（**含解码行+单字节 raw dump 两种行**）③`crates/game-core/src/{enemy,stage,wave_record,shot}.rs` 与 `docs/findings/*`。
- **关键方法论发现**：seed 共 91,601 行，其中**单字节 raw 行 56,636（61.8%）**、解码指令行 34,922；分 bank：bank02 raw **7,193** vs 解码 466、bank03 raw 4,962 vs 1,382、bank04 raw 7,257 vs 387…… ⇒
  被审文档 §3.1/§3.2 的"全 seed xref"实际只覆盖**解码行**，raw 区内的 `JSR/JMP` 不可被 grep 命中（raw 行逐字节显示，如 `bank02:8CEC ED $ED` / `8CED D7 $D7`）。
- 本机只读；未改被审文件/资产，未 `git add`/`commit`；日志/探针在 `target/review-boss/`（可再生、不入库）。

## 1. 自列必查点逐条结论（a/b/c/d）

| 点 | 自述关键断言 | 我的独立复核 | 结论 |
|---|---|---|---|
| **(a)** `$A16F`=32B 类型表 vs 5B 记录族 | dir 尾=`$A16F`；`$865C LDA $10/AND #$1F/TAY/LDA ($9F),Y/STA $400,X`；32B 字节；5B 贪心扫描段清单；第 4 条 ptr=`$81A4` 失配；原 w28"5B 记录族"不可实装；边界 W32 未证 | ①dir 8B 行实读与表一致（STG-1 `00 A0 FA BC 09 09 6F A1`，STG-8 `6A AC 49 B1 0A 0A 7D AF`）；`$A16F` 32B **逐字节相等**（`CF 1B 10 1F A4 CF 14 10 CA A6 8F 1D 11 86 A4 12 60 05 A4 81 1D 21 35 A4 9D 1A 22 29 A4 82 14 13`，idx8=`$CA`）；`$802A-$8032`/`$865C` 指令逐条命中；②独立 5B 扫描（ptr∈`$A400-$B000`）段起点/条数=**$A16F(3)/$A182(4)/$A19A(3)/$A1B7(5)/$A1D4(5)/$A1F0(5)/$A228(7)/$A24F(4)** 与自述一致；前 3 条 ptr=`$A41F/$A6CA/$A486`✓、**第 4 条 ptr=`$81A4` 失配**✓；`completeness.md` B-STG-01-BOSS ⚠ 项确含"W32 实体识别"✓；`w28-boss.md:33/52` 确为原"5B 生成记录族/~30 指针"主张✓ | **成立**（仅段间"空隙"数值有误，见 E2） |
| **(b)** HP/beat 表字节↔消费指令 | `$4A`=`[0C,0E,18,28]`；`$5F` `$BBF2`/`$BBF6`；`$61` `$A1F9`/`$A201`；`$CB` 组 `$ADE5/$ADE9/$ADED` | 全部表字节**逐字节相等**：`ABCB=0C 0E 18 28`、`BBF2=3C 50 64 78`、`BBF6=B4 A0 8C 78 64 50 3C 32`、`A1F9=04 06 08 0A 0C 0F 14 19 1E`、`A201=1E 1B 19 17 15 13 11 0F`、`ADE5=0A 1E 32 50`、`ADE9=B4 78 5A 50`、`ADED=0A 14 19 23`、`ADE1=8C 5A 3C 1E`；消费指令逐条验证：`bank04:813A LDA $A1F9,Y→$630,X`、`:8140 LDA $A201,Y→$6BC,X`（前置 `LDY $4D`）；`bank07:AD31/AD46/AD4D/ADA6/ADAC`（`$ADE1→$6BC`、`$ADE5/$ADED→$630`、`$ADE9→$6BC`）；`$4A` handler 字节流（`bank04:$8BDB` 起：`LDA #$80/STA $668；JSR $C40B；LDA #$13/STA $4C4；…；LDY $4C；LDA $ABCB,Y；STA $630,X`）与 `$5F` handler（`bank03:$B9DB` 起：…`LDY $4C；LDA $BBF2,Y；STA $630,X；LDY $4D；LDA $BBF6,Y；STA $6A0/$6BC`）逐字节命中 | **成立**（语义标签 B 已如实标注） |
| **(c)** `$D674` 链 HP=`$0630,X`；`$05DC` 非 HP；`$4C=0` 一击死 | 指令序列/27+6 入口/`$D616`/`$D86B[256]`；`$05DC` 三证；`$6F4` 到期倒数 | `$D674` 起 0x50 字节逐字节解码与自述**完全一致**（`20 8B D7 / 90 D2 / 20 2F D6 / A5 4C / F0 3B / A4 6E / 10 0C / BD 30 06 / 38 / E9 02 / F0 2F / B0 15 / 90 29 / … / A9 00 9D 30 06 4C 16 D6`）⇒ HP 字段=`$630,X`、`$4C==0 ⇒ HP:=0→$D616`✓；`$D616` `LDA $D86B,Y`✓；`$05DC`：`bank14:$CD9D-$CDC7`（`ADC $5C0,X/STA；carry⇒INC $5DC,X；CMP Y；≥Y⇒$5DC:=0`）与 `bank05:$8C8D`（`INC $5DC,X; AND #7; TAY; LDA $AC5F,Y; STA $684,X`）**字节级一致**✓；`$05F7`（`$05DC+27`）"恒 0/非 HP"见 `arena-midboss.md:215`（§3.1 字段表）✓；`$6F4 DEC` 写点 47 处✓；L2-HP-1..4 与指令语义一致（HP−2/HP−HP_Y/一击死/进位阈值归零）✓ | **成立**（W15 的"唯一性"另见 (d)） |
| **(d)** W15/W15′ 静态收窄 | `$D7ED` **全 seed 唯一**调用点 `bank04:$81E1`；`$D4FE` 调用点**全集 6 处**（全 bank03）；`$7D4` **唯一**写点 `bank14:$D53A`；"全 seed xref 完备" | **证伪**：①ROM 字节扫描 `20 ED D7/4C ED D7` = **8 处**：`bank04:$81E1` + **`bank02:$8CEB/$8D35/$8E13/$8E75/$8F07/$8F3B/$8F97`（7 处，均在 seed raw 行区；上下文均为合法代码：`JSR $CC51/JSR $CC6A→JSR $D7ED→BCC…`）**；②`20 FE D4/4C FE D4` = **9 处**：自述 6 处 + raw 区 `bank03:$ADDA(JMP)/$B05A(JSR)/$B0FE(JMP)`；③`99 D4 07`（`STA $7D4,Y`）= **2 处**：`bank14:$D53A` + **`bank02:$8882`**（`TXA; STA $7D4,Y; LDA #$30; STA $7E9,Y; INC $74; RTS` = 第二个框表注册写者）；④`20/4C 74 D6` ROM 扫描 64 候选，其中 **≥4 处 raw 区真实调用**（`bank02:$ACA0 JMP`、`bank03:$B3F0 JMP`、`$B6EA JSR`、`$B95F JMP`，上下文均 `JSR $CC51/$CC6A` 后接）⇒ "27+6=33 入口"只是 seed 解码面下界 | **证伪（核心错误）** |

## 2. 被确认清单（可独立复算，作为后续实装依据）

| # | 结论 | 证据 |
|---|---|---|
| C1 | `$A16F`=32B 类型表（非 5B 记录族起点）；生成例程 `$865C` 用 `($9F)[$10&$1F]` | dir/表 32B 逐字节；seed `bank00:802A/865C/8619` 指令逐条 |
| C2 | HP/beat 表 9 组字节 | 见 §1(b)；D2 命令输出逐字节 |
| C3 | HP 字段=`$0630,X`；`$4C=0` 一击死；物撞 `SBC $630,Y`；弹路 `SBC #2` | `$D674` 0x50 字节逐字节；`$D616`/`$D86B` |
| C4 | `$05DC`=相位/动画计数（非 HP）；`$05F7` 弱反证；`$6F4` 到期倒数 | `$CD9D-$CDC7`/`$8C8D` 字节；arena-midboss:215；47 处 `$6F4` 写点 |
| C5 | 敌接触杀玩家 handler = `bank04:$8129`（dispatch `$61→$A129`），`$D7ED` 命中后 `$0400:=#0F`、`$41C:=0` | `enemy.rs:759 TypeDispatch{0xA129, bank4}`；`bank04:$8129` 起 `LDA #$93/STA $4C4; LDA #$37/STA $518; LDA #$CB/STA $684; LDY $4D; LDA $A1F9,Y; STA $630,X; LDA $A201,Y; STA $6BC,X…`；`$81E1 JSR $D7ED; BCC; LDA #$F; STA $400; LDA #0; STA $41C` |
| C6 | `$0400` 绝对写点全集=**9 处**，地址与自述逐项一致 | ROM `8D 00 04` 扫描 9 命中：bank00:$978E/$9CF9/$9D23、bank01:$A98B/$AB88、bank03:$A345、bank04:$81E8、bank15:$E1B8/$E2B6 ✓ |
| C7 | dispatch 表映射 `$4A→$ABDB(bank4)`、`$5F→$B9DB(bank3)`、`$61→$A129(bank4)`；`BANK7_HANDLERS` 12 体（$62/$64/$65/$A0/$A1/$A2/$CA..$CF） | `enemy.rs:736/757/759` + `BANK7_HANDLERS:1010-1022` 逐项 |
| C8 | EN 8×5B 表 ptr=`$B3E6/$B2B4/$B2B4/$B3E6/$B235/$B235/$B21D/$B2B4` 与 `stage.rs` 一致 | ROM `$ADD0-$ADF7`；`stage.rs:106` 注释同四型 |
| C9 | `TERMINAL_DROPPERS`/`$B889` 流、`$CA` 型 boss 存在 | `wave_record.rs:185`；`engine.rs:791`；`enemy.rs` `$CA→$A6EC` |
| C10 | L2-HP-1..4 规格与 A 级指令语义一致（不是实测） | `$D674`/`$CD9D` 字节 + 自述已声明"无 Mesen2，不产出实测"✓ |
| C11 | 环境/指针：`arena-midboss.md` `$05F7` 表、`hit_samples.jsonl` release_golden `hit_cnt:0`、`scenario_run.py` 含 `fcntl`、`tools/da6502.py` 仅反汇编、`hit-test`/`respawn-window` 场景存在 | 逐文件 grep / 读 |

## 3. findings 四分类

### 3.1 实现错误（B）——无（本卡为只读证据报告，无代码改动）

### 3.2 证据不足 / 结论错误（E）

- **E1（高，核心必改）xref "唯一/全集"口径不完备 ⇒ W15/W15′/B4 计数被证伪**：
  seed 61.8% 行为**未解码单字节 raw dump**（bank02 raw 7,193 行 vs 解码 466 行；bank03 raw 4,962 vs 1,382；bank04 raw 7,257 vs 387），
  文档 §5 D4/D5 用 `grep seed` 统计 xref，只能覆盖解码行；raw 区中的真实 `JSR/JMP` 不可命中。ROM 字节扫描独立复算：
  - `$D7ED`：**8 处**（bank04:$81E1 + bank02:$8CEB/$8D35/$8E13/$8E75/$8F07/$8F3B/$8F97；后者上下文均为代码：`… JSR $CC51; JSR $D7ED; BCC …`）⇒
    §0.6/§3.1 "全 seed 唯一调用点 / 无第二调用点"作为 **ROM 级结论不成立**（作为"seed 解码行内唯一"才成立）；W15 已定位静态入口=**8**，不是 1。
  - `$D4FE`：**9 处**（自述 6 + `bank03:$ADDA/$B05A/$B0FE`）⇒ §3.2 "调用点全集 6 处"不成立。
  - `STA $7D4,Y`：**2 处**（bank14:$D53A + `bank02:$8882`：`TXA; STA $7D4,Y; LDA #$30; STA $7E9,Y; INC $74; RTS`）⇒ W15′"归属写点唯一"不成立。
  - `$D674`：seed 解码面 27+6=33 ✓（逐条核对无误），但 ROM 字节扫描 64 个 `20/4C 74 D6` 候选，除数据误报外**≥4 处 raw 区真实调用**
    （bank02:$ACA0 JMP、bank03:$B3F0 JMP/$B6EA JSR/$B95F JMP）⇒ "33 条受击处理入口"应标"seed 解码面下界"，非 ROM 全集。
  **修正建议**（文件+节）：§0.6 第 6 行、§3.1 首条与末段、§3.2 首两条、§2.1 "结论 B4"行、§4.6 改为
  "seed 解码面唯一/6 处；ROM raw 区另有 7/3/1/≥4 处候选（清单见复核命令 D6）"；等级 A→**B（待 raw 区审计/重解码）**；
  §5 增 D6 命令（ROM 字节 `20/4C target` 扫描 + 上下文人工判读），并在报告头注"seed 仅部分解码"事实。
  这是**去夸大/去唯一性断言**的修正，不改变已确认的表/链结论。
- **E2（Low-Med）段间空隙数值错误**：§1.2 写"段间 4–8B 空隙"，独立扫描实际空隙=**4/4/14/4/3/31/4**；
  §1.5 写"段间空隙 1–3B"，实际列出的段间=**3–4B**（首段 `$ADF8` 末→`$AE01`=4B），其后还有 17/15B 等更大空隙。
  另 §1.5 "指针均落 `$B0xx-$B4xx`"对后续（省略号之外）段不成立：`$AED2` 段含 ptr `$BE41`；`$B888+` 段含 `$BAAB/$BCAD/$BEAF` 等。
  建议：改数值/限定"仅所列前 7 段"；§0.4/§1.5 统一口径。
- **E3（Low）§0.4 与 §1.5 的 EN-9+ 段清单互相矛盾**：§0.4 写 `$AE30-$AE53`(7)、`$AE7D-$AE91`(4)、"$ADF8(1)"、
  空隙 4/17/4/18B；§1.5（=独立扫描）为 `$AE22-$AE36`(4)+`$AE3A-$AE53`(5)、`$AE6F-$AE7E`(3)+`$AE82-$AE91`(3)、
  第 9 条自 `$ADF8` 起（在 `$ADD0-$ADFD`(9) 段内）、空隙 3/4B。建议 §0.4 直接引用 §1.5 输出。
- **E4（Low）§1.2 "其后至 `$B885` 无 ≥3 连串"与 §5 D3 命令不一致**：D3 扫描到 `$BFFB`，独立跑会额外输出
  `$B886(3)/$BB1B(4)/$BE6F(3)/$BE88(3)/$BECD(4)` 等段（都在 `$B885` 之后）。建议 D3 加扫描上界 `<=0xB885`（或正文列明"仅数据区段"）。
- **E5（Low）§5 复现命令在本机声明环境不可直接执行**：文首称本机 Windows+`python`（`python3` 别名损坏），但 D1-D5 均为 bash heredoc
  （`python - <<'PY' …`），PowerShell 下报 `Missing file specification after redirection`；我需改写为脚本文件才能跑。
  建议改为 `python target/xxx.py` 或给 POSIX/Windows 双版本。

### 3.3 规格缺口（S）

- **S1（Info）**：L2 判定入参含 `$6E/$4C/$4D/$630,X` 等 RAM 面，但本卡未给"L2 case 文件格式/落点"（已声明交 porter/verifier 卡）；
  建议在 §2.3 末尾补一行"建议写入 `oracle/subroutines/l2_cases*.jsonl` 的 schema 指针"（不阻塞）。
- **S2（Info）**：`$4C` 在 `$D674`（一击死旗）与 boss 表（难度/档索引）同址多义——文档已登记风险✓，建议在 L2-HP 规格中显式拆分两个语义域。

### 3.4 平台差异（P）

- **P1**：seed（IDA 线性扫描）解码不全是仓库级已知现象；本复核量化：raw 行 61.8%（bank02 尤其严重）。建议将"xref 统计必须加 raw 扫描"写入
  `reference/seed` README/知识库（跨任务适用），否则后续 xref 结论会系统性漏计。
- **P2**：本机无 Mesen2/`analysis/traces` 不入库 ⇒ 动态样本不可得（自述如实✓）；L2 实测需 Linux oracle 宿主。

## 4. 命令与输出摘要（自跑，只读）

| # | 命令 | 输出摘要 |
|---|---|---|
| 1 | `python target/review-boss/verify_rom.py` | dir/`$A16F` 32B/HP 表全字节、EN 8×5B ptr、5B 段扫描：与自述 §1.2/§1.4/§1.5 逐项一致（段起点/条数/第 4 条 `$81A4`） |
| 2 | `python target/review-boss/verify_asm2.py` | `$D674` 调用 seed 解码面=27（bank04×1/bank05×11/bank06×11/bank07×4）+ 包装 6（`$D84F×5/$D860×1`）；`$D7ED` seed 解码面=1（bank04:$81E1）；`$D4FE` seed=6；`STA $7D4,Y` seed=1——**与自述一致**（但见 raw） |
| 3 | 自写 raw 扫描（ROM `20/4C target`）：`target/review-boss/verify_rom2.py` | `$D7ED` **8**（bank02×7+bank04×1）；`$D4FE` **9**（bank03，含 `$ADDA/$B05A/$B0FE`）；`STA $7D4,Y` **2**（bank14:$D53A+bank02:$8882）；`8D 00 04` **9**（与自述 9 地址全等） |
| 4 | `python target/review-boss/verify_coverage.py` | seed raw 行 56,636 / 解码 34,922（bank02 7,193/466、bank03 4,962/1,382、bank04 7,257/387）；raw 调用点上下文均为合法代码（附字节） |
| 5 | `git grep -n 'BANK7_HANDLERS'` + `enemy.rs:736/757/759/1010-1022` | dispatch `$4A→$ABDB(bank4)/$5F→$B9DB(bank3)/$61→$A129(bank4)`、bank7 12 体表逐项✓ |
| 6 | `git grep 'TERMINAL_DROPPERS' / 'parse_spawn_records'` + `stage.rs:106` | `$B889` 流、`TERMINAL_DROPPERS`（types `$5F/$CA`）、spawn ptr 四型✓ |
| 7 | `arena-midboss.md:215`、`hit_samples.jsonl:13`、`gunnac.asm` D86B 标签、`scenario_run.py:34/418` fcntl、`da6502.py` 仅 `d/fmt/main` | `$05F7` 恒 0/非 HP、release_golden `hit_cnt:0`、`$D86B[256]`、POSIX 依赖、无 6502 执行器——均与自述相符 |
| 8 | `docs-links`：本复核文件 0 外链；被审文件无 inline markdown 链接 | 0 断链（当前仓库 docs-links FAIL 系并发文件，与本文件无关） |

## 5. 修正建议（按优先级，全部为 docs 内改动）

1. **P0（E1）**：把所有"唯一/全集/33 入口"改为**双口径**：`seed 解码面`（自述现有数字）+ **ROM raw 扫描候选**
   （`$D7ED` 8 / `$D4FE` 9 / `$7D4` 写点 2 / `$D674` raw 区 ≥4）；§3.1 "无第二调用点"、§3.2 "唯一写点/6 处全集" 必改；
   W15/W15′/B4 相关结论等级 A→B，状态改"已知静态点已扩面，仍待 raw 区完整审计/重解码"；§5 增 D6 raw 扫描命令。
2. **P1（E2/E3/E4）**：修正段间空隙数值（4–8B→4/14/31B 实值；1–3B→3–4B 且注明大空隙）；§0.4 与 §1.5 统一（引用同一扫描输出）；
   "指针均 $B0-$B4"限定到所列段；D3 加 `$B885` 上界或列明额外段。
3. **P2（E5）**：§5 复现命令改跨平台（Windows 可直接跑的 `python script.py` 形式）。
4. **P3（P1/S）**：把"seed 部分解码（raw 61.8%）⇒ xref 必须 raw 扫描"写入 seed README/知识库（跨任务防再犯）。
5. 复审条件：P0 改正 + P1/P2 修正后，本复核可 **pass**（其余表/链/写点/L2 结论已独立复算通过，无需重跑动态）。

## 6. 复核结论

**needs_revision**：文档主体（Boss/EN-9+ 定界、HP/beat 表、`$D674` 伤害链、`$0400` 写点全集、L2 规格、dispatch/bank7 表）**独立复算通过**，
但**自列必查点 (d) 的"唯一调用点/调用点全集/唯一写点"与 B4 的"33 入口"结论因 seed 解码不完备而不成立**（ROM 级证伪：8/9/2/≥4 处）；
另有 3 条 Low 数值/口径/命令问题。修正为 docs-only（附 raw 扫描命令与双口径）后可复审 pass。

> 本复核只写本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。

---

# 复审（2026-10-18，task-68）—— 定点复审 task-66 双口径修正

> 被复审对象：`docs/findings/boss-en9-hp-evidence.md` 修正版（**354 行**，SHA256 前 16=`779ff78691aa03af`，未提交，与 Lead 通告 `779ff786…` 一致）。
> 范围=task-68 指定 3 项（双口径/降级、D6-D7 可跑、E2-E5 精度）；其余表/链/字节结论 task-64 已独立复算通过，本轮不重跑。
> **VERDICT（复审）：needs_revision** —— 双口径/降级与 D6/D7 **全部落实且我逐条实跑复核通过**；E2-E5 仅 **§0.1 残留 1 处旧 gap 数值**（一行）＋ 2 条 Info（38.2% 四舍五入、`pwsh` 7 措辞）。
> 属"一行收口"，改后可直接 pass（无需再全量复审）。

## 复审 §A：双口径与降级（项 1）——通过

| 检查 | 独立核 | 结果 |
|---|---|---|
| 双口径是否一律"seed 解码面 vs ROM raw 扫描候选" | 头注 L8-12（更正块：56,636 raw/61.8%、8/9/2/64、A→B）；§0 表 6 行（L20-25）；§2.1 L115-116（B4 双口径+候选→真调用规则）；§3.1 L154-159、§3.2 L163-166（"原文唯一调用点/唯一写点**作废**"+双口径）；§4 L186-187（B，待 raw 审计）；§6 L342（双口径纪律/禁止单称唯一全集） | ✅ 一律双栏，无单口径残留结论 |
| raw 新增调用点是否列全 | D6 实跑（见 §B）：`$D7ED` 8（bank04:$81E1 + bank02 七点 $8CEB/$8D35/$8E13/$8E75/$8F07/$8F3B/$8F97）；`$D4FE` 9（seed 6 + bank03:$ADDA/$B05A/$B0FE）；`STA $7D4,Y` 2（bank14:$D53A + bank02:$8882）；`$D674` 64（9+55，≥4 raw 真调用 bank02:$ACA0 / bank03:$B3F0/$B6EA/$B95F——D6 输出均在列，bank02:$8CA0=窗口别名已在 L312 注明） | ✅ 与自述逐项一致 |
| W15/W15′/B4 是否 A→B | L12 等级调整；L24-25（§0 行标 B）；L115（B4=B）；L159/166（W15/W15′=B）；L343（§6 自查） | ✅ 降级落实 |
| 历史"唯一/全集"是否仅在更正/作废语境 | grep 全部命中：L10（总更正声明）、L115（"原文 33 入口仅 seed 面"）、L154/163（"作废"）、L187（"seed 解码面唯一…raw 另有 7 处"=双栏限定语）、L342/350/351（更正/纪律） | ✅ 无单口径断言存活 |

## 复审 §B：D6/D7 可跑（项 2）——通过（我逐字实跑）

**D6**（按文档 L274-313 原样 `@'…'@ | python -`，在 Windows PowerShell 执行）实测输出：
- `JSR $D7ED candidates=8`（7×`seed_decoded=False` bank02 + bank04:$81E1=True）；`JMP $D7ED candidates=0` ⇒ 计 8 ✓
- `JSR $D4FE=3`（A14F/A700=True，B05A=False）+ `JMP $D4FE=6`（AADC/AB3A/AC39/AC3F=True，ADDA/B0FE=False）= **9** ✓
- `STA $7D4,Y=2`（bank02:$8882=False，bank14:$D53A=True）✓
- `JSR $D674=9` + `JMP $D674=55` = **64** ✓（含 raw 4 真调用：$8CA0(=$ACA0 别名)/$B3F0/$B6EA/$B95F）
- 输出含 `seed_decoded=True/False` 标记；"候选→真调用判定规则"（seed 解码=真；raw=候选需控制流/自向量吻合，或前序 `JSR $C304` 类上下文）在 L116/L310-312 写明 ✓

**D7** 原样实跑：`lines=91601 decoded=34922 raw=56636 raw%=61.8%`；分 bank 与文档 L9/L334 期望**逐值一致**（bank02 466/7193、bank03 1382/4962、bank04 387/7257；完整 16 bank 我另存 `target/review-boss/d7-run.log`）✓

**D3** 原样实跑（供项 3 对照）：EN 段 `$ADD0-$ADFD`(9)+`$AE01`(6)/`$AE22`(4)/`$AE3A`(5)/`$AE57`(4)/`$AE6F`(3)/`$AE82`(3)/`$AE94`(3)/`$AEB4`(3)/`$AED2`(3)，gap=**4/3/4/4/4/4/3/17/15**，`$AED2` 首 ptr=**$BE41**；`$B886+` 另有 5 段（3–5 条，ptr=$A5xx-$AFxx）；boss 段 gap=**4/4/14/4/3/31/4** ✓。D2 spot check 字节全等 ✓。

## 复审 §C：E2-E5 精度（项 3）——**1 处残留**

| 检查 | 结果 |
|---|---|
| boss gap `4/4/14/4/3/31/4` | §1.2:L49 **已改** ✓；**§0.1:L20 仍写"段间 4–8B 空隙"** ❌（与 L49/L353 矛盾；"先读"摘要表残留旧数值） |
| EN gap `4/3/4/4/4/4/3/17/15` | §0.4:L23 ✓、§1.5:L87 ✓（与我 D3 输出逐值一致） |
| `$AED2` 串 ptr=`$BE41`；"均落 $B0-$B4"已删/更正 | §0.4:L23 ✓、§1.5:L88 ✓（D3 同值） |
| §0.4↔§1.5 清单统一 | 逐段对比**完全一致**（含 $AEB4/$AED2 与 gap 串） ✓ |
| §1.2 与 D3 范围一致（$A000-$BFFB） | §1.2:L49 已写"全范围扫描 … `$B886` 起他区/未归属"；D3 命令确为 `a<=0xBFFB` ✓ |
| 其它残留 | ①**L342/L350 "seed 仅 38.2% 解码"**：D7 实测 decoded=34,922/91,601=**38.12%**（raw 61.88%）⇒ 应为 **38.1%**（Info，四舍五入不一致）；②**L195 "`pwsh`（PowerShell 7）实测可跑"**：本机 `pwsh` 命令不存在（仅 Windows PowerShell；here-string→python 在 5.1 实测可跑）⇒ 建议改"Windows PowerShell/PowerShell 7 均可"（Info，非可执行性问题——我已用 5.1 跑通 D2/D3/D6/D7） |

## 复审 §D：剩余问题（全部 docs-only，一行级）

- **R1（必改，Low）**：§0.1:L20 "段间 4–8B 空隙" → 改为 **"段间 4/4/14/4/3/31/4B"**（与 §1.2:L49、§6:L353 一致；或直接写"见 §1.2"避免二次漂移）。
- **R2（Info）**：L342/L350 "38.2% 解码" → **38.1%**（D7: 34,922/91,601；raw 61.8%）。
- **R3（Info）**：L195 "在 `pwsh`（PowerShell 7）实测可跑" → "在 Windows PowerShell（5.1+）/PowerShell 7 均可（`@'…'@ | python -`）"。
- 其余 task-64 findings（E1 双口径/降级、D6/D7、E3/E4/E5）**均已闭环**；R1 改后本复审可直接 **pass**。

## 复审 §E：命令与输出摘要（自跑）

| # | 命令 | 摘要 |
|---|---|---|
| 1 | D6（文档原样，PowerShell here-string→python） | 8 / 0 / 3+6=9 / 2 / 9+55=64，含 raw/decoded 标记；`target/review-boss/d6-run.log` |
| 2 | D7（文档原样） | `lines=91601 decoded=34922 raw=56636 raw%=61.8%`；分 bank 16 行与文档期望一致；`d7-run.log` |
| 3 | D3（文档原样） | EN gap 4/3/4/4/4/4/3/17/15、$AED2 ptr=$BE41、$B886+ 他区 5 段；boss gap 4/4/14/4/3/31/4；`d3-run.log` |
| 4 | D2 spot（here-string→python） | ABCB/BBF2/ADE5 字节全等（D1/D2/D4/D5 同 PowerShell 形态；D5 由 Select-String 形式，命令自明） |
| 5 | grep 残留 | `4–8B` 仅 §0.1 命中；"38.2%" ×2；"pwsh" ×1 ⇒ 即 R1-R3 |
| 6 | 被审文件行数/SHA | 354 行 / `779ff78691aa03af` ✓ 与通告一致 |

## 复审结论

**needs_revision（仅 R1 一行必改；R2/R3 Info 建议同批）**：task-64 的证伪结论（seed 部分解码⇒xref 不完备；W15/W15′/B4 唯一/全集不成立）已被**完整**落实为双口径并降 B，
raw 调用点清单（8/9/2/64 与 ≥4 raw 真调用）与 D6/D7 命令**逐字可跑、输出全对**，EN 段清单/gap/ptr 与 D3 一致；
唯独 §0.1"先读"摘要表仍残留旧"4–8B"（与 §1.2/§6 矛盾）。R1 改掉（可选带 R2/R3）后即可转 **pass**，无需再跑动态/全量。

> 本复审只追加本节到本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。

---

# 点验（2026-10-18，task-70）—— R1/R2/R3 三行点验

> 对象：`docs/findings/boss-en9-hp-evidence.md`（**354 行**，SHA256 前 16=`ad0d23e490324995`，未提交）。
> 范围：仅核 task-68 提的 R1/R2/R3 三处（其余 task-64/68 已通过，本轮不重跑）。

| # | 要求 | 实测 | 结果 |
|---|---|---|---|
| R1 | §0.1:L20 → `段间 4/4/14/4/3/31/4B 空隙（见 §1.2）`，与 §1.2:L49/§6:L353 一致；`4–8B` 0 残留 | L20 精确含 `段间 **4/4/14/4/3/31/4B** 空隙（见 §1.2）`；L49 含 `段间空隙 4/4/14/4/3/31/4B`、L353 含 `boss 段间空隙 4/4/14/4/3/31/4B`（三处同值）；全文件 grep `4–8B` **count=0** | ✅ |
| R2 | L342/L350 → `38.1%`（D7 实测 34,922/91,601=38.124%）；`38.2%` 0 残留 | L342/L350 均含 `38.1%`；grep `38.2%` **count=0**；复算 34,922/91,601=**38.124%** ⇒ 38.1% 正确（与 raw 61.8% 互补口径一致） | ✅ |
| R3 | L195 → `Windows PowerShell（5.1+）/PowerShell 7 均可执行`；`pwsh` 0 残留 | L195 精确含该措辞（并补"here-string 形态经本机 Windows PowerShell 5.1 实测可跑"）；grep `pwsh` **count=0** | ✅ |

**旁证**：`python gates/cli.py status` 当前树 `docs-links PASS（340 md/433 links）`、`STATUS: OK`；被审文件自身 0 inline markdown 链接（无新增断链风险）。文件行数 354 与 task-69 通告一致；三处数值亦与 task-68 我实跑的 D3/D7 输出逐值一致（gap 4/4/14/4/3/31/4；decoded 34,922/91,601）。

**剩余问题：无**（3 处全部按 task-68 修正到位；`4–8B`/`38.2%`/`pwsh` 均 0 残留）。
**点验结论：pass** —— 可提交 `boss-en9-hp-evidence.md` + 三份复核报告（task-64/68/70）；建议提交 message 注明复核链与本文件 SHA `ad0d23e4…`、以及"双口径（seed 解码面 vs ROM raw 扫描）"的最终口径。

> 本点验只追加本节到本文件；未改被审文件/资产，未 `git add`/`commit`。复核方≠实施方（AGENTS.md §5）。
