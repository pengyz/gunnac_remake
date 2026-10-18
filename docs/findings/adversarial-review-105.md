# 对抗复核 #105 — shell/win 实装面复核（五件+APU 合成+契约测试 13/13）

> 日期：2026-10-18｜角色：adversarial-reviewer（证伪立场；实施方=port-engineer≠审核方）
> 被审：0eee972①crate 建/1cea1b9②帧驱动+blit/4ad41c4③音频件/9ffc6ad④宿主接线/7dca0a3 trait 锁定
> +a57edd0①契约测试/2426d38②宿主 harness/b821b72③冒烟自测/9266ce0 APU 合成模型（9d661b6 五线）。
> 五证伪靶：①五件 ②trait 实现 vs 契约测试 ③APU 四件 ROM 重导 ④分层合同守恒 ⑤宿主真边。拆段落盘。

## 段 A：①shell/win 五件 + ④分层合同守恒——**五件全落+合同守恒** ✅

### A.1 ① 五件逐件核（shell-win/src/lib.rs 177 行全读）

| 件 | 实装 | 裁定 |
|---|---|---|
| ① crate 基 | shell-win crate 立（deps=game-{core,platform}+winit+softbuffer） | ✓ |
| ② 帧驱动 60Hz | `FrameDriver`（Instant 节流+accum 固定步——`tick()→应推进帧数`） | ✓ 60Hz 固定步语义正确 |
| ③ blit 整数缩放 | `blit_scaled`+`view_rect`（256×240 RGBA→窗口，**保比例黑边**——game-desktop view_rect 同语义注） | ✓ |
| ④ 音频件 | `AudioSink`（ApuWrite 流 `writes:Vec<(u32,u16,u8)>` 同源三元组+**`silent:true` 静音占位显式**）+`impl Audio` | ✓ 合成器未证期显式占位不发明 |
| ⑤ 宿主接线 | `GameHost`/`HostState{Title,Stage(1-8),Ending}`+`press_start` 边（见段B ⑤ 真边核） | ✓ 骨架落位（真边面段B） |
| 契件 | `map_key`（键盘→8 键）+`storage_dir`（**cfg 三路**：windows=%APPDATA%/gunnac/ unix=~/.local/share/gunnac/ 其他=fallback） | ✓ #104 §3 设计逐字兑现 |

### A.2 ④ 分层合同守恒（Cargo 复审）——**守恒** ✅

| crate | deps 实读 | #104 基线 | 裁定 |
|---|---|---|---|
| game-core | 仅 game-platform | 同 | ✓ 守恒 |
| game-platform | **空** | 同 | ✓ 守恒 |
| game-data | 空 | 同 | ✓ |
| game-oracle | 仅 game-core | 同 | ✓ |
| shell-win（新） | game-{core,platform}+winit/softbuffer | — | ✓ 平台库=平台壳 |

核心合同面（core/platform 零外部依赖）**守恒无回退** ✓。
（口径注：平台库现于 shell-win+game-desktop 两处——desktop=先存工具壳，spec §4"仅 shell/win"
按正式壳口径；核心合同不受影响，若要字面收紧宜将 desktop 归类注一词。）

---

## 段 B：②trait 实现 vs 契约测试锁 + ⑤宿主真边——**对表全合+真边非发明** ✅

### B.1 ② 实现面 vs 契约测试锁（mock 对表）——**六 trait 对表全合** ✅

| 面 | 契约锁（mock 断言） | 实现面 | 对表 |
|---|---|---|---|
| Renderer | `canvas_mut().len()==256×240×4`+present 计数 | shell `blit_scaled` 消费 256×240 RGBA（同面） | ✓ |
| Audio | 流序保持（`(1,0x4000,0x30),(1,0x4001,0x08),(2,0x4015,0x0F)`+帧序 [1,2]） | `AudioSink.writes` 同序 Vec<(u32,u16,u8)> | ✓ **流序锁同形** |
| Storage | slot0 往返+slot1 空 None+**越界槽拒** | Storage trait 形（#104 已核最小面） | ✓ 往返/边界双锁 |
| Clock/GameRng | 确定性锁（同输入→同 mock 态/同种子→同序列） | 虚拟时钟/确定性契约 | ✓ |

**契约面四套实跑全绿**：trait_contract **5**/trait_shapes **2**/full_flow **3**/smoke **5**（含
smoke_input_mapping_contract）——"13/13"=被审时点三面合计口径；我方实跑零红 ✓
（计数口径差 13 vs 15=smoke 5 件归属面分法，实质全绿不受影响）。
**域声明补全**：mock 用 reg=0x4000/0x4001/0x4015 **u16 全地址** ⇒ #104 微注②解——
trait u16 域=APU 全寄存器地址（game-core u8 偏移→u16 全址=实现侧换算），域语义已明。

### B.2 ⑤ 宿主接线真边（F1-F13 边集非发明）——**真边锁于 game-core harness，shell 骨架=真子集** ✅
- **真边集本体**：game-core `full_flow.rs` harness（2426d38）逐关断言**段锚四连序**
  （shop/Transition→StageStart→Combat→StageClear）+结局入口=**AllClear（$180==9→$95F9
  语义，l5-all-clear R-l5-4）**——与我 #96 段A ⑤ ROM 面（E02C-E031/$95F9/8E4E）**互证** ✓✓；
- **shell 骨架边**：Title→Stage(1)→…→Stage(8)→Ending→Title（press_start 驱动）——
  四条边**全部为真流宏观边**（title 起跑/清关进关/八关毕入结局/结局归零重启=E02C 复位族），
  **无发明边** ✓（触发简化 press_start=harness 契约面② 声明口径，非边集发明）。

### B.3 中间记数
①五件+④守恒（段A）+②对表+⑤真边（本段）——四靶收口；③APU 四件 ROM 重导=段C。

---

## 段 C：③APU 合成四件 ROM 重导 + Verdict

### C.1 ③ APU 五线规则 ROM 重导（抽 4）——**四件逐指令精确** ✅

| 线 | ROM 直读（bank11 窗） | 声称 | 裁定 |
|---|---|---|---|
| ① 音调合成式 | `83A9` 起 `BD E3 03`(**LDA $03E3,X**)`18 65 0A`(`+transpose`)`29 0F`(**AND #$0F**=16 音阶)`28 10 02 09 80`(PLP/BPL/**ORA $80** 八度旗) | "$03E3+transpose→AND #$0F 16 音阶+$80 八度旗（83A9）" | ✓✓ 逐指令精确 |
| ③ duty 4 形 | `84A3` `B9 D7 8A`(**LDA $8AD7,Y**)`29 60`(**AND #$60** 高2)`2A×4`(ROL 移位) | "$8AD7 高2→$4000 7-6" | ✓✓ 精确 |
| ④ pulse 影子族 | `818C` `BD 09 03`(**LDA $0309,X**)`9D 00 40`(**STA $4000,X**)…`9D 01 40`(**STA $4001,X** 扫频特判) | "$0309+X→$4000+X+扫频特判" | ✓✓ 精确 |
| ②⑤ 写簇端 | **$81E5=STA $400E**（noise）/**$81CD=STA $4008**（triangle） | "写簇 818C-81E5"+noise/triangle 三字段 | ✓✓ **簇界 81E5 精确** |

合成链总图（$8AD5 族→状态槽→变换→影子→$4000+X 直写）四点全落位 ✓；
残项挂（pulse 周期 11 位/包络位形/长度计数表）**显式不发明** ✓。

### C.2 #105 Verdict：**pass** ✅（微注 3 笔随档）

| 靶 | 裁定 |
|---|---|
| ① 五件 | ✅ crate 基/FrameDriver 60Hz/blit 保比例/ApuWrite+静音显式/宿主骨架（段A） |
| ② trait vs 契约测试 | ✅ 六 trait 对表全合+四套实跑全绿（5/2/3/5）；mock u16 全地址域明 |
| ③ APU 四件 ROM 重导 | ✅ 音调式/duty/影子族+扫频/簇端四件逐指令（段C） |
| ④ 分层合同守恒 | ✅ core/platform 零依赖无回退（段A） |
| ⑤ 宿主真边 | ✅ full_flow harness 真边锁（$180==9→$95F9 与 #96 互证）+shell 骨架真子集无发明（段B） |

**五靶全中，零必修**——shell/win 实装面过审。微注 3 笔：①game-desktop 归类口径
（字面"仅 shell/win"宜注工具壳例外）②契约 13/13 vs 实跑 15 计数口径差（实质全绿）
③#104 微注② 已解（mock u16=APU 全寄存器址，建议 trait 文档补一词）。

### C.3 纪律自查
lib.rs 177 行全读+Cargo 逐 crate 复审+契约四套实跑+full_flow harness 断言体+
APU 四件 ROM 逐指令；未改被审产物；与 #96/#104 互证面全数引用（真边/合同）。
