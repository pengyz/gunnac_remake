# 对抗复核 #104 — 平台抽象层架构复核（spec v1+trait 锁定 WIP·架构面先行）

> 日期：2026-10-18｜角色：adversarial-reviewer（证伪立场；实施方=port-engineer≠审核方）
> 被审：572475a platform-shell.md（架构规格 v1）+ crates/game-platform（trait 缝 WIP 75 行）。
> 三证伪靶：①分层合同（平台依赖仅 shell/win/零依赖照守）②trait 锁定形状=已证语义最小面
> （Canvas/ApuWrite 流/Buttons/字节块——不发明审查）③OS 差异 cfg 面设计（差异实现非分叉）。
> 拆段落盘。

## 段 A：①分层合同 + ②trait 锁定形状——**合同全净 + 最小面七 trait 不发明** ✅

### A.1 ① 分层合同（Cargo 依赖实审计）——**零依赖照守** ✅

| crate | [dependencies] 实读 | 裁定 |
|---|---|---|
| game-core | 仅 `game-platform`（workspace） | ✓ **零外部依赖**（无 I/O/平台 crate） |
| game-platform | **空**（+`#![forbid(unsafe_code)]`） | ✓✓ 零平台代码零外部依赖 |
| game-data | 空 | ✓ |
| game-oracle | 仅 game-core | ✓ |
| game-desktop（现平台侧具） | game-{core,data,platform}+**winit/softbuffer** | ✓ 平台库仅平台侧 crate |

分层合同三行图（core→trait 缝←平台实现）与依赖实况**逐行吻合** ✓；
"平台依赖（winit/softbuffer/音频 crate）仅 shell/win（平台侧）"+**跨平台库+cfg 差异**
（用户裁定：Linux/Windows 同源开发验证，无须交叉工具）照守 ✓。
（注：现树平台侧 crate=game-desktop（工具壳）；shell/win 正式壳落地后同规则迁移——合同面不冲突。）

### A.2 ② trait 锁定形状（七 trait 逐面不发明审查）——**最小面成立** ✅

| trait | 形状 | 已证语义锚 | 不发明核 |
|---|---|---|---|
| `Clock` | `now_ms()→u64` | 虚拟时钟（game-core 确定性根基——#85-#103 差分全链所依） | ✓ 仅时间读口 |
| `GameRng` | `next_u32`+`next_below`（默认） | 确定性 LCG 契约（算法待 S4 锁=显式注） | ✓ 仅序列契约（见微注） |
| `Buttons`/`Input` | 8 键 bool 结构+`buttons()` | 秘籍掩码位图 A 级（bit0=LEFT 族——#97 段C 核）+方向注随档 | ✓ "无组合语义；连发/宏=平台侧策略"显式 |
| `Renderer` | `canvas_mut()→&mut [u8]`+`present()` | 256×240 RGBA Canvas（game-desktop 同语义——#99-103 全链核） | ✓ 像素级最小面；缩放黑边=实现侧显式注 |
| `Audio` | `write_apu(frame,reg,value)`+`end_frame` | ApuWrite 流三元组（L-Audio hash 面同源——#100 同式核） | ✓ 流消费最小面；"无合成器期=透传/静音占位显式" |
| `Storage` | `load(slot)→Option<Vec<u8>>`+`save(slot,&[u8])→bool` | 存档面预留（语义未证） | ✓ **纯字节存取不发明**（格式/语义零承载） |

**锁定原则核**："trait 形状=已证语义最小面（Canvas 像素/ApuWrite 流/Buttons/字节块）；
绘制细节/混音算法=实现侧语义，trait 不承载未证行为"——七 trait 逐面**无越级** ✓✓。
Stage A 约束注（"接口形状是缝的占位…接口不得提前发明行为（AGENTS.md §3）"）在 crate 顶注 ✓。
**微注×2（不阻断）**：①`next_below` 默认实现=% bound（模偏置语义是轻量行为承诺——
算法待 S4 锁注已在，impl 可覆写，建议默认体加"偏置可覆写"注）②`write_apu` reg:u16 vs
game-core ApuWrite.reg:u8（$00-$17）——宽化对称宜注一词（寄存器域声明）。

---

## 段 B：③OS 差异 cfg 面设计 + Verdict

### B.1 ③ cfg 面设计（差异实现非分叉）——**设计面成立** ✅

| 设计件 | spec 面 | 裁定 |
|---|---|---|
| cfg 点极小化 | **跨平台库优先**（winit 窗口/softbuffer 帧缓冲/cpal 音频族）——真 OS 特有点才 cfg | ✓ 设计原则正确（差异面最小化） |
| cfg 差异点声明 | 存储路径（Windows=%APPDATA%/gunnac/ vs Linux=~/​.local/share/gunnac/）+音频后端（若有） | ✓ 具体点显式列名 |
| **非分叉原则** | "cfg(windows)/cfg(unix) 差异实现点=…**差异实现非分叉**" | ✓✓ 同 crate 内 cfg 分支≠fork 代码库——设计语义正确 |
| 同源开发 | 用户裁定："无须交叉工具，Linux/Windows 同源开发验证" | ✓ 与 §4 工程纪律一致 |
| 帧驱动/呈现 | 60Hz Instant 节流→stage_tick 族→合成→present；blit 整数倍缩放（game-desktop blit_scaled 同语义） | ✓ 与已证面同构 |
| 游戏宿主 | title→设置→八关→结局（game-core flow/stage 已证 F1-F13 边集接线） | ✓ 状态机=已证面接线不发明 |

**设计裁定**：cfg 面设计=**差异实现（同 crate 内 cfg 分支+跨平台库）非分叉** ✓——
与分层合同（段A）自洽闭合；trait 缝（段A ②）+cfg 面（本段）+零依赖实审计（段A ①）三面互锁。

### B.2 #104 Verdict：**pass** ✅（微注×2 随档）

| 靶 | 裁定 |
|---|---|
| ① 分层合同 | ✅ Cargo 实审计全净（core/platform/data/oracle 零外部依赖+forbid unsafe） |
| ② trait 锁定形状 | ✅ 七 trait 最小面逐面不发明（组合/缩放/混音/格式全显式留实现侧） |
| ③ OS 差异 cfg 面 | ✅ 差异实现非分叉+cfg 点极小化+同源开发纪律自洽 |

**三靶全中，零必修**——架构面先行件过审；微注×2（next_below 模偏置可覆写注/
write_apu reg:u16→u8 域声明）随档不阻断。**架构合同三面互锁**（零依赖×最小 trait×非分叉）
=平台抽象层立项的成立性核毕。

### B.3 纪律自查
Cargo.toml 逐 crate 实读+trait 75 行全读+spec 全文核；未改被审产物；
**provenance 注**：段A 内容被并发提交 9d661b6 反向卷入（暂存污染第三例——内容完好，
本注为 provenance 权威）；守窗轮询×2+锁瞬态处置（核活进程→确认消→重提）。
