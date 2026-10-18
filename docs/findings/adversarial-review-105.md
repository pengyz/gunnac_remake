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
