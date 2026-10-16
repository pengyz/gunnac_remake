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
