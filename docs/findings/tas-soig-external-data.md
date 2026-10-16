# 外部 TAS 数据采集（Soig「All levels」46:05.70 —— 网上搜寻令产物）

> 【2026-10-16】captain 直办（web 检索+抓取）。来源：B 站 BV1yDHpeXEQQ 简介（Soig 全流程 TAS，
> nicovideo sm44063557 同源）+ mediafire 统计表。**社区主张=B 级**，入结论前须我方复证。

## 1. 输入件搜寻结论（用户令：找 TAS 录制好的输入）

| 源 | 结果 |
|---|---|
| TASVideos | **无**（4421G：0 Publications/0 UserFiles；2015 WIP 帖弃坑未传件） |
| nicovideo commons sm44063557 | 描述含 fm2 头（ROM MD5/SHA1/FrameCount 166215/FCEUX 2.2.3）——**件体/附件本机网络不可达**（nicovideo 域全折） |
| mediafire | **Gun_Nac_infomations.xlsx 到手**（三表，见 §2） |
| 百度网盘（pwd=pxx5） | 同款统计表（描述自述），未取 |
| B 站分段视频 | Area-0..8 逐关视频+全流程+无 HUD 版（season 41827）——**视频件**非输入件 |

**ROM 兼容性**：Soig 用 `Gun-Nac (J) [!].nes` MD5 `029DAA688D2C51A296EB4E7FED953D0D`
= **我方 reference/jp 逐字吻合**（zno5/Soig 同一 JP 基准）——若输入件到手可直接回放！

## 2. 统计表三表（reference/external/tas-soig/Gun_Nac_infomations.xlsx）

### 2.1 武器统计（验证我方伤害公式金数据）
- **常规武器 1#-5# × 等级 1-9 伤害矩阵**：1#=−2HP 恒定（L9 crash）；2# 穿透 −3HP；
  3#/4#/5# 按等级 −2/−3HP+弹数倍增（double/triple/quatra/penta/hexa）；4# 不穿透
  —— **与我方弹命中公式 `HP−=$D688 SBC #$02`（−2HP）同源** ✓ 待逐格复证
- **特殊武器 F/B/T/W**：ready timer 27；等级时长 3×60..7×60 帧（-2/-3/-5 修正）；伤害 −3HP；L5=crash

### 2.2 boss 统计（战斗面金数据——W-tas-2R 同族）
- middle/end boss × Area 0-6 × 武器 1#-5#/F 的**击杀命中数**（"As usual"=常规公式/"2"=2 发/"-"=不可用）
- 例：Area 3 middle boss=3# 武器 6 发；F 武器 Area 3 middle=20 发

### 2.3 黄色补给统计（item ID 语义）
- ID 1-140+ 循环映射：Money/P/Money/P/…/F|T|FBTW 按模式穿插——补给掉落 ID 表

## 3. 秘籍 9 条（D2/A1 商店语义直答——B 级待复证）

| # | 秘籍 | 我方对照 |
|---|---|---|
| 1 | 设置界面声音测试选 5=选关+**激活 AREA-0** | $01AE SOUND 字段+AREA-0 假设坐实（三方）|
| 2 | **免费购物**：首次进店离店前按住 左+选择 | **A1 商店隐藏交互直答**（H-Shop 同族）|
| 3 | **钱袋→特殊武器转换**：第 8 关前激活；「结束」光标按住 右+A/B/开始 提前激活 | **A1/D2 商店隐藏选项效果直答**！ |
| 4 | 开局第 5 帧按 A+B 直进设置 | 速通技巧 |
| 5 | 即时补给（作者自测未实现） | 存疑 |
| 6 | debug 模式 **PENNOYAA（FFF9=01）**：设置+不死/武器/特殊武器选项 | **D2 $FFF9 门三方坐实**（美/日通用）|
| 7 | logo 界面上+开始=直跳 ending | 速通技巧 |
| 8 | 隐藏 OP：兔子画面长按 A+B→"Death & Non-Death Free Select Mode OK!"+设置+不死模式 | **$01BE DEATH·NON DEATH 行直答**（限美版）|
| 9 | 隐藏 ED：不死模式开启且未用→ending 长按 A+B 解锁日版画面 | A3 结局流旁证 |

## 4. Soig 运行笔记（机制旁证）
- **boss 登场条件=杂兵彻底消失**（与我方关末块槽扫描语义互证）
- 第 8 关利用敌人上限占槽卡掉高血敌→胚胎第一时间登场（B1 槽位语义互证）
- 拖板流程防 lag+boss 用 dps 最高武器

## 5. 待办路由
- 秘籍 2/3 → decomp（A1 商店隐藏交互/钱袋转换语义考证）
- 武器矩阵+boss 击杀数 → test-runner（伤害公式复证 L2 级）+port（等级时长 timer）
- OBS-1 → Area-8 视频可作观察面（出店段可见）
- 输入件：nicovideo commons 附件待网络可达时取（或用户代取）
