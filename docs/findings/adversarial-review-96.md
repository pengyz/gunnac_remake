# 对抗复核 #96 — 通关跑三判据（"Rust 版 8 关可通关"终验收）

> 日期：2026-10-16｜角色：adversarial-reviewer（证伪立场；实施方=port-engineer≠审核方）
> 被审：clear-run.trace.jsonl 41 事件（e600195 全链驱动/feef4f3+6a2d5d7 bot 策略/
> 8c78e19 run 面对拍/a6a06cd trace 落盘）+ 三结局 $4C 分支 + diff_run_face 41/41 + 确定性 replay。
> 六证伪靶：①41 事件逐关段锚 ②三结局 listing 对账 ③diff_run_face 复跑 ④确定性 replay ⑤通关支链
> ⑥EndingFx 五步/staff roll 43。拆段落盘。

## 段 A：①41 事件逐关段锚独立重导 + ⑤通关支 $180==9→$95F9 链

### A.1 ① 41 事件结构+锚——**8/8 关段锚=gate tick 精确命中** ✅
trace 结构自算：`Shop/Transition/StageStart/Combat/StageClear ×8 + Ending` = **41 事件** ✓；
流形常量**全关恒定**：shop 段 168f / trans 33f / start→combat 240f / 关间隙 665f（8 段全同）。

**combat→clear 帧距 vs 各关 gate tick**（#93/#94 已独立重导的锚）：

| 关 | 帧距 | gate tick | 裁定 |
|---|---|---|---|
| STG-01 | **1232** | 0x4D0=1232（#93 ⑬ 锚） | ✅ |
| STG-02 | 848 | 0x350 | ✅ |
| STG-03 | 1088 | 0x440 | ✅ |
| STG-04 | 792 | 0x318 | ✅ |
| STG-05 | 649 | 0x289 | ✅ |
| STG-06 | 1042 | 0x412 | ✅ |
| STG-07 | 652 | 0x28C | ✅ |
| STG-08 | 772 | 0x304 | ✅ |

**8/8 逐点精确** ✓✓——41 事件的关段锚全部由既证 gate tick（尾 thr 链自算）推出，
"stg01 1232 tick 自然跑面"=0x4D0 ✓；末事件 Ending f15923=stg08-clear+665f（=关间隙常量同形）
checkpoint `all-clear-95f9` 与 ⑤ 的 $95F9 命名一致 ✓。

### A.2 ⑤ 通关支链核（listing）——**三件齐** ✅
- **$180==9 判点**：`bank15:E02C LDA $180 / E02F CMP #9 / E031 BCC` ✓（<$9 续关流；≥9=通关态）；
- **$95F9=结局初始化体**：`95F9 LDA #$80/STA $26`（flags 位7 置）→ JSR $E590/$E7FB/$EED7/$E567 →
  `LDA #$13/JSR $ED3B`（PPU 令）→ **`LDA $1BE`**（DEATH 选卡——#86 D2 面互证）✓；
- **直跳边**：`bank00:8E4E JMP sub_95F9` ✓（该边由 $FFF9/$18==$18 门引——隐藏支同入口，
  通关支经 $180 态汇入同结局体）。trace `all-clear-95f9` 锚名与 $95F9 精确对应 ✓。

---

## 段 B：②三结局分支 listing 对账 + ⑥EndingFx 五步/staff roll 43

### B.1 ② 三结局分支——**表/分支/兜底三面精确** ✅

| 件 | listing | 裁定 |
|---|---|---|
| $97D3[$4C] 恒等表 | bank0:$97D3 = `00 01 02 03 …` ROM 直读 ✓ | ✅ 索引 0-3 恒等 |
| 表读点 | `9734 B9 D3 97 LDA $97D3,Y` | ✅ |
| 分支条件 | `960E LDA $1BE / 9611 BNE` + `9613 LDA $4C / 9615 BNE loc_9625` | ✅ **`$1BE==0 && $4C!=0`⇒故事页（9625: $AC87 指针）/否则 scene4（9617: JSR $AB70…LDA #4/JSR sub_9763）** |
| 实现同构 | EndingStep::PreScene 注"前置场景（$1BE==0 && $4C!=0 故事页；否则 scene4）" | ✅✓ **与 listing 逐条件 1:1 吻合** |

scene1/2/3=恒等表索引 1/2/3（结局卡面）；**NOVICE 无卡**=scene0 收尾屏（#86 H-A3-1 已裁定
"功能名=内容名同物"）✓；**EXPERT 常驻**=结局卡常驻面（9431/9439 卡服务读者）；
$95F9 体 `LDA $1BE` 选卡与 #86 D2 面（DEATH/NON DEATH 二记录）互证 ✓。

### B.2 ⑥ EndingFx 五步 + staff roll 43 记录——**五步锚全对 + 43 字节级精确** ✅
五步序（engine.rs EndingStep）逐锚：
1. **EntryScene**（$95F9 入口场景服务）← 段A.2 结局体 ✓
2. **StaffRollBgm** ✓
3. **PreScene**（$1BE==0&&$4C!=0 故事页/否则 scene4）← B.1 分支 ✓✓
4. **StaffRollScroll**（流 $9955 **43 记录**+**$FF 尾** ⇒ $180:=1）
5. **EndingCard**（$97D3[$4C]）← B.1 表 ✓

**staff roll 流 ROM 直解**（bank0:$9955，`[attr][chars]FE` 记录族）：
总 FE 记录 66（空行 23）+**非空 43** ✓✓ 精确命中；**$FF 尾 @offset 621** ✓；
内容实读="COMPILE STAFF"/"GAME DESIGN & PROGRAM"/…/"PRESENTED BY NEXOFT CORP."/
"@2011 JIUBAN"（真 credits 流）✓。边界诚实注（"内部时长挂起不发明=run 步进率非时序主张"）在案 ✓。
（自查记：初读误用 bank2 窗——$95xx 族实为 bank0，已改；口径修正入自查。）
