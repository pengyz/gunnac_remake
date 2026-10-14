# D3 基底归属修正（W-P4）：九班基底裁定 + L3 菜单时序差归因

> 【2026-10-14】decomp-analyst 只读考证产出。对象：`rom/gunnac_cn.nes`（九班，sha 4d76901c…）
> vs `reference/jp/gunnac_jp.nes`（标准日版，sha 08ead6a2…）。本文件为**裁定书**：
> ① 基底归属（美/日/混合）② L3「mode 110 菜单时序差」归因（区域/翻译/复合）
> ③ 对 jp-cn-parity.md 与 l3-jp-cn-dualrun.md 的 dated 更正清单（**只列不改**，交 captain 复核执行）。
> 证据分级依 AGENTS.md §3；C 级一律入 §6 假设区。外部网络取证本次不可用（web_search API 余额不足），
> 全部证据来自 ROM 字节 / Mesen2 trace·截图 / seed 投影（A/B 级）。

## 0. 结论速览

| # | 裁定项 | 结论 | 等级 |
|---|---|---|---|
| ① | 九班基底归属 | **美版基底**（US 构建 + 九班汉化覆盖层；非日版、非自混合） | A |
| ② | L3 菜单时序差（707f vs 173f） | **(a) 区域/构建差异为主**——美版标题/菜单流程结构（分期揭示 + 输入锁 + 菜单独立成步）；"翻译内容更长"假设**已证伪**；另叠加 L3 固定输入脚本间距的**测量膨胀**（约 +240f） | A（归因）/B（分解） |
| ③ | 更正清单 | 9 项（l3-jp-cn-dualrun.md 5 项 + jp-cn-parity.md 4 项），见 §4 | — |
| ④ | 门禁 | `python3 gates/cli.py check` 全绿（12 PASS / 0 FAIL，见 §8） | — |

---

## 1. US 基底证据复核（①）

### 1.1 completeness §1.1 声称复现（A 级）

completeness.md §1.1 称：`rom:7106–7118` `\t@2011 JIUBAN` + 标题画面含
`LICENSED BY NINTENDO OF AMERICA INC.` ⇒ 美版基底。复现：

```bash
sha256sum rom/gunnac_cn.nes reference/jp/gunnac_jp.nes
grep -abo 'JIUBAN' rom/gunnac_cn.nes            # → 7113
grep -abo 'NEXOFT' rom/gunnac_cn.nes            # → 4349, 7092
grep -abo 'LICENSED' rom/gunnac_cn.nes          # → 4364
grep -abo 'NINTENDO' rom/gunnac_cn.nes          # → 4378
grep -abo 'AMERICA' rom/gunnac_cn.nes           # → 4390
grep -abo 'TONKIN' reference/jp/gunnac_jp.nes   # → 4280, 6424（JP 侧）
grep -abo 'TONKIN' rom/gunnac_cn.nes            # → 无（exit 1）
grep -abo 'NEXOFT' reference/jp/gunnac_jp.nes   # → 无（exit 1）
grep -abo 'AMERICA' reference/jp/gunnac_jp.nes  # → 无（exit 1）
```

标题版权块逐字节复现（两侧同文法：`{PPU地址2B, ASCII/ tile码, FE 分行, FF 终止}` 记录制）：

```bash
python3 -c "d=open('rom/gunnac_cn.nes','rb').read(); print(repr(d[4260:4402]))"
python3 -c "j=open('reference/jp/gunnac_jp.nes','rb').read(); print(repr(j[4240:4312]))"
```

| 版本 | 标题版权块内容（file 偏移） | 市场标记 |
|---|---|---|
| 九班 | `1990 @TOKYO SHOSEKI`(4297) / `PROGRAMED BY COMPILE`(4326) / `PRESENTED BY NEXOFT CORP.`(4349) / `LICENSED BY`(4364) + `NINTENDO OF AMERICA INC.`(4378) | **NEXOFT（美版发行）+ NOA 授权行 = 美版市场内容** |
| JP | `1990 @TONKIN HOUSE`(4280) / `PROGRAMED BY COMPILE`(4308) | TONKIN HOUSE（日版发行），**全 ROM 无 NEXOFT/NINTENDO/AMERICA 串** |

§1.1 声称**复现成立**（E-R01/E-R12 字节与偏移逐项吻合）。

### 1.2 扩展区域签名（本次新增，全 A 级）

| ID | 证据 | 复现命令 | 说明 |
|---|---|---|---|
| E-B01 | 见 §1.1 版权块对照 | 见 §1.1 | US 版权块 vs JP 版权块互斥 |
| **E-B02** | **隐藏构建串**：两版同址 file `130832`（0x1FF10，前 16B `\xa4R:&]\xad\xb4\r\x9f\xb8J\xe3\xa4Ul\xa9` 两侧全同）：九班=`1990-12-17 USA` + `SEE YOU` ASCII 画（224B）；JP=`1990-06-21 MOU  JIKI MASTER-UP, KOREGA FC SAIGO NO SIGOTONI…`（罗马字开发者留言） | `python3 -c "d=open('rom/gunnac_cn.nes','rb').read(); j=open('reference/jp/gunnac_jp.nes','rb').read(); print(repr(d[130832:131056])); print(repr(j[130832:131056]))"` | **同址替换**：九班侧是「1990-12-17 **USA**」构建标记，JP 侧是 1990-06-21 日版构建留言。汉化补丁没有动机把隐藏开发者留言换成 "USA" 构建串——此为基底=美版构建的最强签名 |
| E-B03 | 开场文本：九班 ROM 含英文 `SEVEN PLANETS`(11577/11879)、`HIS MISSION IS TO`(12016)、`A POWERFUL INVADER`(12143/12303…)、结局 `THANKS TO GUN-NAC`(12278)；JP ROM 全部无这些 ASCII（日文假名 tile 存储） | `grep -abo 'SEVEN' rom/gunnac_cn.nes; grep -abo 'PLANETS' rom/gunnac_cn.nes; grep -abo 'HIS MISSION' rom/gunnac_cn.nes; grep -abo 'THANKS' rom/gunnac_cn.nes`（JP 侧同命令均无命中） | **运行时显示已证（像素 A 级）**：`shots/gameplay-stage1_f00240_probe.png`（sha 36dde3a1…）显示 `IN THE VAST UNIVERSE, FAR AWAY, … SEVEN PLANETS ORBITING AROUND AN ARTIFICIAL SUN.`；JP 同帧 `shots_jp/gameplay-stage1_f00240_probe.png`（sha f326c8d8…）为日文 `7つの星と じんこうようようの…`。开场为**美版英文原文且未汉化** |
| E-B04 | JP 独有串在九班全部缺席：`GAME START`(JP 4246)、`CONFIG.SYS`(JP 4260)、`SOUND TEST`(JP 5265)、`GAME LEVEL`(JP 5279)、`AREA-`(JP 5257)、`DEATH/NON DEATH`(JP 5426/5442)、`TONKIN`(4280/6424)、`JIKI/MASTER-UP`(JP 130848/130853) | 对两 ROM 分别 `grep -abo '<串>'` | 九班不存在任何日版专属标记（排除"日版基底+美版串移植"的混合假设） |
| E-B05 | 九班独有 `OPTION DEMO`（12264/12635）框住美版结局文本块（`THANKS TO GUN-NAC…TO PRESERVE PEACE.`，file 0x2FE6–0x3160）；JP 同区段为完全不同的二进制数据 | `grep -abo 'OPTION DEMO' rom/gunnac_cn.nes; grep -abo 'OPTION DEMO' reference/jp/gunnac_jp.nes`（后者无） | 美版结局文本（E-W07：美版改过 intro/ending）物理存在且被标记串框住；语义未证（入 §6） |

**排除"自混合"**：若为日版基底+美版串/文本移植，需要补丁作者 (a) 整体替换开场/结局为英文、
(b) 重写版权块、(c) 把隐藏开发者留言换成 "USA" 构建串、(d) 清除全部日版专属串——动机与工作量均不成立；
且 E-B02 的同址构建标记、E-B03 的英文开场**运行时实际显示**，与"日版行为+美版贴皮"不相容。

### 1.3 裁定 ①：**美版基底**（US 构建 + 九班汉化覆盖层）

- 九班 ROM 的构建身份 = **1990-12-17 USA 构建**（E-B02），文本/版权/开场层 = 美版内容（E-B01/E-B03/E-B05）；
- 汉化覆盖层 = 标题字标（加纳战机）、标题菜单（开始/设置）、商店/价目面板文本、HUD 串、字模 CHR（中文字模）等；
- **未汉化残留**：开场/结局（英文）、标题版权行（英文）、`HI SCORE/SCORE/CONTINUE/GAME OVER`（英文）。
- 对 completeness §1.1 的复核结论：**声称成立**，且证据链可再扩展（E-B02 为最强新增签名）。

---

## 2. 区域特征签名对照（JP vs 九班）

| 维度 | JP | 九班（美版基底） | 级 |
|---|---|---|---|
| 标题版权 | 2 行：`1990 @TONKIN HOUSE / PROGRAMED BY COMPILE` | 5 行：`1990 @TOKYO SHOSEKI / PROGRAMED BY COMPILE / PRESENTED BY NEXOFT CORP. / LICENSED BY / NINTENDO OF AMERICA INC.` | A |
| 标题菜单 | `GAME START / CONFIG.SYS`，**内联在标题文本流**（PPU 0x226B/0x22AB，与版权同流同帧出现） | `开始 / 设置`（CJK 象限字形），**独立脚本流、输入后才写出**（PPU 0x226C–0x22CC 同屏区） | A |
| 开场文本 | 日文假名（`7つの星と じんこうようようの…`） | 英文美版原文（`IN THE VAST UNIVERSE…`），未汉化 | A |
| 隐藏构建串 @130832 | `1990-06-21 MOU JIKI MASTER-UP…` | `1990-12-17 USA` + `SEE YOU` | A |
| 标题脚本记录层 | 2×`$E520` + 5×`$EB28` + 1×`$E6E3`（文本流） | 2×`$E516` + **6**×`$EB1E` + 1×`$E6D9`（文本流）+ **5×`$E8B3`（逐行记录）** + **1×`$E6D9`（第二文本流=菜单）** | A |
| 引擎代码 | 入口 `$E520/$EB28/$E6E3` | 入口 `$E516/$EB1E/$E6D9`——**一律 −10B 平移**（=jp-cn-parity 已记 bank15 +10B 布局漂移的脚本引擎面） | A |
| 字模 CHR | chr00-07/11/12/14/15 = 日版字模/字标 | 同号 CHR = 中文字模/中文宇标（jp-cn-parity §1 已记） | A |

脚本记录层对照复现（记录=引擎调用形 `{cmd16}{arg16}{ptr16}{ppu16}` 8B 族，两侧同构）：

```bash
python3 -c "d=open('rom/gunnac_cn.nes','rb').read(); print(repr(d[0x1064:0x1176]))"   # 九班标题脚本
python3 -c "j=open('reference/jp/gunnac_jp.nes','rb').read(); print(repr(j[0x1042:0x1082]))"  # JP 标题脚本
grep -n 'bank15:E516\|bank15:E8B3\|bank15:EB1E\|bank15:E6D9' reference/seed/gunnac.asm  # 引擎入口实码
```

`$E8B3` 逐行记录（九班独有）的 len 字段 = 版权各行 ASCII 长度、ppu = 各行地址，逐条吻合：

| 记录（file 起点） | len | ppu | 对应行 | 行长 |
|---|---|---|---|---|
| 0x1132 `b3 e8 13 00 00 00 66 22` | 19 | 0x2266 | `1990 @TOKYO SHOSEKI` | 19 ✓ |
| 0x113A `b3 e8 14 00 00 00 a6 22` | 20 | 0x22A6 | `PROGRAMED BY COMPILE` | 20 ✓ |
| 0x1142 `b3 e8 19 00 00 00 e4 22` | 25 | 0x22E4 | `PRESENTED BY NEXOFT CORP.` | 25 ✓ |
| 0x114A `b3 e8 0b 00 00 00 2a 23` | 11 | 0x232A | `LICENSED BY` | 11 ✓ |
| 0x1152 `b3 e8 18 00 00 00 64 23` | 24 | 0x2364 | `NINTENDO OF AMERICA INC.` | 24 ✓ |

→ 逐行机制**围绕美版英文版权行构建**（汉化作者无动机为未改动的英文行写逐行记录）。

---

## 3. L3 菜单时序差归因（②）

### 3.1 窗口复现（A 级）

`docs/findings/l3-jp-cn-dualrun.md` 类①：mode 110（`$0401==110(0x6E)`，语义仍 H-15/C 级不猜）
窗口 九班 f206–913（707f）vs JP f200–373（173f）。用 gameplay-stage1 场景 RAM 探针复现：

```bash
python3 - <<'EOF'
for rom,dirp in [('CN','analysis/traces'),('JP','analysis_jp/traces')]:
    for f in range(240,1261,60):
        import glob
        fn=glob.glob(f'{dirp}/gameplay-stage1_f{f:05d}_*.ram.hex')
        if fn:
            d=bytes.fromhex(open(fn[0]).read().strip())
            print(rom,f,hex(d[0x401]))
EOF
```

- 九班：`$0401=0x6E` 贯穿 f240–900，f925 起 =0 ⇒ 退出点 f913 ✓（707f ✓）
- JP：`$0401=0x6E` f240–360，f420 起 =0 ⇒ 退出点 f373 ✓（173f ✓）

### 3.2 窗口去向分解（A/B 级，游戏段对齐）

| 段 | 九班（gameplay-stage1） | JP（gameplay-stage1） | 差异性质 |
|---|---|---|---|
| 开场文本页 | f206–270（英文页，press@240 一按即过） | f200–270（日文页，同上） | **同构同时长**（不是差异源） |
| 标题 | f300 仅版权（无字标）→ f360 字标出现 → 字标调色动画至 ~f600；**f600 前输入全部无效** | f300 起 字标+菜单+版权 全部到位（同款字标调色动画继续），输入即响应 | **+约 300f 输入锁定期**（差异源 1） |
| 菜单 | f630 起 `开始/设置` 由输入触发后才写出（独立脚本流）；f900 press START 选中 | 菜单内联标题屏，f360 press START 直接选中 | **+1 个导航步**（菜单独立成步，差异源 2） |
| 退出 | f913 → f925 商店 | f373 → f420 商店 | — |

像素证据（repo 内截图可复核）：

- 九班 f300/f360/f420/f480：`shots/gameplay-stage1_f00300…f00480_probe.png`（版权+字标渐次出现）
- 九班 f660–f900 菜单态：`shots/gameplay-stage1_f00660…f00900_probe.png`（`◆开始/设置`）
- JP f300/f360 标题+菜单：`shots_jp/gameplay-stage1_f00300_probe.png`、`f00360_probe.png`
- JP f420 起商店：`shots_jp/gameplay-stage1_f00420_probe.png`

### 3.3 判别实验（A 级；场景 DSL 内联可复现）

三组对照跑（Mesen2，`tools/scenario_run.py run <path>`，产物落 /tmp 临时区，DSL 如下）：

```text
# menudelay：仅 press 240 start，之后零输入（1080 帧）
name menudelay / frames 1080 / press_len 4 / press 240 start / capture_every 30 probe
# menuearly：早期密集 START（240/270/300/330/360）后静默
name menuearly / frames 1080 / press_len 4 / press 240 start / press 270 start / press 300 start / press 330 start / press 360 start / capture_every 30 probe
# jpearly（JP ROM）：press 240 start + press 270 start
name jpearly / frames 600 / press_len 4 / press 240 start / press 270 start / capture_every 30 probe
```

（复现方式：`GUNNAC_TRACES=<目录> GUNNAC_SHOTS=<目录> [GUNNAC_ROM=reference/jp/gunnac_jp.nes] python3 tools/scenario_run.py run <dsl>`。注：输出目录若在仓库外，scenario_run.py 收尾的 run.json 清单步骤会因 `relative_to(ROOT)` 抛错——采集本身完整，属工具小瑕疵，未改工具。）

| 实验 | 结果（截图 sha 逐帧比对） | 结论 |
|---|---|---|
| menudelay（九班，仅 1 按） | f240–600 与 gameplay-stage1 **逐帧全同**；f630–1080 **始终无菜单** | 标题揭示/动画 = **纯定时脚本行为**（与输入无关）；菜单出现**必须有输入** |
| menuearly（九班，5 个早期 START） | **与 menudelay 逐帧全同（f240–1080 全部）**，菜单始终不出现 | f600 前输入（含 5×START）**全部被忽略** ⇒ 存在 ~400f **输入锁定期**；"按次数推进菜单"假设证伪 |
| jpearly（JP，2 个早期按） | `$0401=0x6E` 持续至 f600（mode 110 不退出）；对照 gameplay-stage1 JP press@360 即 f373 退出 | JP 同样"标题就绪前忽略输入"，但锁止于 **~f300（标题出现即就绪）** |
| gameplay-stage1（九班，标准脚本） | press@600(START) → f630 菜单 VRAM 写入队列出现（RAM 探针 $0102–$014F：`03 22 6c c0 c1 c2 / 03 22 8c d0 d1 d2 / 03 22 ac c3 c4 c5 / 03 22 cc d3 d4 d5` = PPU 0x226C 起 12 tile 写入）；press@900(START) → f913 退出 | 菜单=输入触发的独立脚本流（$E6D9 第二文本流），非文本长度驱动 |

### 3.4 证伪排查：翻译内容假设（默认立场，逐条找"翻译造成"的证据）

| 预测（若"翻译造成"成立） | 实测 | 判定 |
|---|---|---|
| 多耗时间花在中文文本页/文本更长 | 多耗时间花在**标题揭示+输入锁+菜单步**；开场文本页两版同构同长 | ✗ |
| 开场/菜单文本应为中文且更长 | 开场=**美版英文原文未汉化**（f240 像素）；菜单 `开始/设置`（2+2 字）比 JP `GAME START/CONFIG.SYS`（10+10 字符）**更短** | ✗ |
| 菜单分离成步是 CJK 渲染通道受限所致 | 中文文本走同一文本记录通道（商店面板记录 `ppu+tile码+FE/FF` 同构，tile 码大量 ≥$80：`c6 c7 c8 / ad ae af / e6 e7 e8…`，file 8880–9010 一例）；菜单流本身也是 `$E6D9` 标准文本流（file 0x1164 起 `6c 22 e0 e1 e2 fe …`）⇒ **无编码约束迫使拆流** | ✗ |
| 时序应随文本量浮动（换文本即变） | 揭示/动画/锁相**四次独立 run 逐帧全同**（exp-shop-b/start/nav + gameplay-stage1 f300–840 sha 全同；menudelay/menuearly 再证）⇒ 硬定时，非内容量 | ✗ |
| 菜单流程计数/页数表值应为汉化内容派生 | 标题脚本结构差（+1 图形记录、+5 逐行记录、+1 文本流）中，逐行记录长度**键合美版英文版权行**（§2 表）；两版记录层同构、引擎入口 −10B 整体平移 ⇒ **构建级结构差** | ✗（指向区域） |

**结论：翻译内容差异（b）无支撑证据，全部预测被证伪。**

### 3.5 裁定 ②：**(a) 区域/构建差异为主**（附测量膨胀说明）

- **差异源 1（主）：美版标题流程结构**——分期揭示（版权→字标→调色动画）+ **约 400f 输入锁定期**
  （f600 前 8 个按键含 5×START 全部无效，A 级判别实验）+ **菜单独立成步**（输入触发第二文本流）。
  JP：标题一次到位、~f300 即就绪、菜单内联、一按选中。这与 E-W07（美版改过 intro/title/ending）及
  E-B02/E-B03（构建=1990-12-17 USA）互相咬合。
- **差异源 2（测量膨胀，方法学）**：L3 用固定输入脚本（press 600/660/900…），九班菜单态在 f630–913
  期间实际是**在等脚本的下一个 press（f900）**——约 +240f 是脚本间距伪影而非游戏内时长。
  若背靠背输入，九班窗口约 f206→610 附近结束（Δ≈240f），707f 会被高估。
  量级估算（B 级）：Δ534f ≈ 输入锁 ~300f + 导航步 +1 ~30–60f + 脚本间距伪影 ~240f。
- **(c) 复合**仅保留 C 级残差（§6-H1）：无美版 ROM 直比前，不能 100% 排除汉化作者对字标/菜单脚本插步
  的贡献；但其动机/必要性均无证据，且主导项（输入锁、菜单成步、逐行记录）均为构建级流程结构。
- 对移植端的含义补充：L3 既定裁决"分歧处以 JP 为准"仍成立，但**性质是区域构建行为选择**（美版流程 vs
  日版流程），不是"修掉汉化内容时序"——backlog 项描述建议随 §4-C1 同步改写。

---

## 4. 归因裁定书：dated 更正清单（③，只列不改，交 captain 复核执行）

> 格式：目标文件 → 位置/原措辞 → 建议更正注原文（`> 【2026-10-14 更正】…` 就地加注，不删历史主张）。

### l3-jp-cn-dualrun.md

| # | 位置 | 原措辞 | 建议 dated 更正 |
|---|---|---|---|
| C1 | §2 类①行（L19） | "**内容驱动时序差**（汉化菜单/文本页流程更长）…落在**内容层**" | 【2026-10-14 更正】归因改判：**区域/构建差异为主**（美版标题流程：分期揭示 + ~400f 输入锁 + 菜单独立成步），"文本页更长"已证伪（开场为美版英文未汉化、菜单文本更短）；707f 另含固定输入脚本间距伪影 ~240f（详见 base-attribution.md §3）。既定"菜单时序按 JP 重基线"不变，但性质=区域构建行为选择 |
| C2 | §2 类②行（L20） | "内容/初值类（**文本地址不同必然**）" | 【2026-10-14 更正】归因过窄：f1 差异含计数/长度类（$11:0e/0a、$20:33/1a、$21:03/02）非文本指针；$04/$05=$9269/$91C4 为 bank0 数据指针对，差来自**构建级布局漂移**（美版构建 + 汉化支持代码，bank00 −0x188 族）而非"翻译文本地址必然"。建议改述为"构建级初值/布局漂移类" |
| C3 | §3（L25） | "**绝对帧对齐失效于内容驱动时序段**" | 【2026-10-14 更正】"内容驱动时序段"→"**构建/区域流程时序段**"（同 C1） |
| C4 | §4（L32） | "改版差异 = 菜单/文本流程时序（类①②）+ 级联漂移（类③），**全部内容层**" | 【2026-10-14 更正】"全部内容层"不成立：类①为美版构建的标题/菜单**流程结构差**（区域差异），类②为构建级初值/布局漂移；仅文本/字模替换部分属内容层 |
| C5 | §2 类③行（L21） | "早期分叉的级联残余（拾取/事件时序）" | 【2026-10-14 更正】补注源头性质：早期分叉 = 类①区域/构建流程时序差（非翻译内容），级联残余定性不变 |

### jp-cn-parity.md

| # | 位置 | 原措辞 | 建议 dated 更正 |
|---|---|---|---|
| C6 | 文首（L3） | "目的：量化'**九班版改过什么**'——评估移植成果的版本普适性" | 【2026-10-14 更正】对照前提修正：JP↔九班差异**不全是"九班改的"**——包含**美日区域构建差**（已证：标题/菜单流程结构、版权块、开场/结局文本、隐藏构建串 1990-12-17 USA）。正确分解=「美日构建差」+「九班汉化层」（见 base-attribution.md §1–2） |
| C7 | §1 表·文本层（L13） | "文本层 \| 全换（**日文→中文**）" | 【2026-10-14 更正】两处错：①基底不是日版——文本层基底=**美版英文**（开场/结局/版权行保留英文原文，f240 像素已证）；②非"全换"——汉化只覆盖标题菜单/商店价目面板/HUD 等，开场/结局/版权/HI SCORE 等未汉化。改述："文本层=美版英文基底 + 部分汉化（菜单/商店/HUD 中文）；开场/结局/版权仍英文" |
| C8 | §1 表·布局层（L15） | "位移原因未明（翻译支持代码注入 or 基版微差）——W 挂" | 【2026-10-14 更正·证据补充】bank15 +10B 实为**脚本引擎入口整体平移**（$E516/$E520、$EB1E/$EB28、$E6D9/$E6E3 三对入口均 −10B，seed 可查）⇒ 构建级代码漂移特征（"基版微差"分支增强）；W-P1 可按"引擎入口平移 + bank00 −0x188 族"收窄审计面 |
| C9 | §2（L21–22） | "「改过的」全部落在**内容/布局层**（文本/字模/指针/银行内移位），逻辑算法与数值未动" | 【2026-10-14 更正】"全部落在内容/布局层"过窄：标题/菜单**脚本记录层与流程**有结构性差异（+1 图形记录、+5 逐行记录、+1 文本流、~400f 输入锁、菜单独立成步）——属**区域构建流程差**。"逻辑算法与数值未动"（钱加法器/VM/加数表等）仍成立 |

### 附带发现（不在两文件内，仅提示 captain，勿混入上表执行）

- `docs/spec/ui-flow.md` R-ui-7 "开机路径未出现开场文本（负证据）"与像素证据冲突：
  title-menu/gameplay-stage1 f240（sha 36dde3a1）即开场文本页（`IN THE VAST UNIVERSE…SEVEN PLANETS…`）。
  建议 spec 侧另行 dated 勘误（或把"开场文本"限定为 E-R09 `HIS MISSION…` 页）。
- `docs/spec/completeness.md` E-R09/E-R10 "是否显示未证"可升级：开场文本**已获像素显示证据**（f240）。

---

## 5. 证伪立场自查留痕

1. **先立"翻译造成"假设**（任务给定默认立场）→ 派生 5 条可测预测（§3.4 表）→ 逐条设计实测 →
   5/5 预测证伪（英文未汉化开场、菜单更短、同通道无编码约束、硬定时、逐行记录键合英文行）。
2. **对"区域归因"也做对称证伪**：找"九班脚本插步"的动机与必要性 → 无证据（CJK 走同一文本通道）；
   但因无美版 ROM 直比，保留 C 级残差 H1（不升结论）。
3. **独立重导**：结论均以可复现命令/逐帧 sha 重导，不引用 l3-jp-cn-dualrun.md 自述；
   其 707f/173f 数字先复现（§3.1）后才用于分解。
4. **测量伪影主动披露**：707f 含输入脚本间距膨胀（~240f），不拿高估值支撑结论（§3.5）。
5. **不猜语义**：$0401（H-15）、`OPTION DEMO`、`$E8B3` 精确语义、输入锁精确边界（f420–600 之间未细分）
   全部留在 §6，未进结论。

## 6. 遗留与假设区（C 级，不得引为结论）

| # | 项 | 级 | 建议 |
|---|---|---|---|
| H1 | 汉化作者是否对标题脚本插步（字标/菜单拆流）有贡献——无美版 ROM 直比不能 100% 排除 | C | 若日后取得美版 ROM：直比 file 0x1064–0x1176 标题脚本区 + 130832 构建串 |
| H2 | `$E8B3`（bank15）逐行记录精确语义（揭示/属性/填充？）——已证 len/ppu 键合版权行，行为未证 | C | Mesen2 插桩该入口写队列时序 |
| H3 | 输入锁精确边界（f420–600 之间）与锁的实现层（代码门 or 脚本阶段） | C | 二分补跑 menudelay×press@450/510/540 |
| H4 | `OPTION DEMO`（12264/12635）语义（美版 attract/结局块标记？） | C | 全流程 S2 场景看是否上屏 |
| H5 | $0401 值域 110/124 语义（H-15 沿用） | C | 不猜，按切换点判定 |
| H6 | web 侧美版 ROM/标题截图外部互证本次未做（web_search API 余额不足） | — | 通道恢复后补证 E-B01/E-B02 的社区互证 |
| T1 | scenario_run.py 在输出目录位于仓库外时 run.json 收尾抛 `relative_to` 异常（采集不受影响） | B | 工具小修（本次不改工具，登记） |

## 7. 复现命令汇总

```bash
# ① 基底证据
sha256sum rom/gunnac_cn.nes reference/jp/gunnac_jp.nes
grep -abo 'NEXOFT' rom/gunnac_cn.nes; grep -abo 'AMERICA' rom/gunnac_cn.nes; grep -abo 'JIUBAN' rom/gunnac_cn.nes
grep -abo 'TONKIN' reference/jp/gunnac_jp.nes; grep -abo 'TONKIN' rom/gunnac_cn.nes   # 后者无命中
python3 -c "d=open('rom/gunnac_cn.nes','rb').read(); j=open('reference/jp/gunnac_jp.nes','rb').read(); print(repr(d[130832:131056])); print(repr(j[130832:131056]))"
python3 -c "d=open('rom/gunnac_cn.nes','rb').read(); print(repr(d[4260:4402]))"

# ② 标题脚本层对照
python3 -c "d=open('rom/gunnac_cn.nes','rb').read(); print(repr(d[0x1064:0x1176]))"
python3 -c "j=open('reference/jp/gunnac_jp.nes','rb').read(); print(repr(j[0x1042:0x1082]))"
grep -n 'bank15:E516\|bank15:E8B3\|bank15:EB1E\|bank15:E6D9' reference/seed/gunnac.asm

# ③ 菜单时序（窗口复现 + 判别实验）
# 窗口：见 §3.1 内联脚本（读 analysis/traces、analysis_jp/traces 的 gameplay-stage1 探针）
# 判别实验：§3.3 三份 DSL → GUNNAC_TRACES=… GUNNAC_SHOTS=… [GUNNAC_ROM=reference/jp/gunnac_jp.nes] \
#           python3 tools/scenario_run.py run <dsl>
# 像素复核：shots/gameplay-stage1_f00240…f00925_*.png、shots_jp/gameplay-stage1_f00240…f00420_*.png
```

## 8. 门禁

`python3 gates/cli.py check` → **CHECK: PASS**（PASS=12 FAIL=0 SKIP=0 TOLERATED=0；
rom/seed/assets/naming/knowledge/docs-links/rust/subroutine/trace/visual/completeness/addr-bridge 全绿；
既有 WARN `ratchet:knowledge_entries 17→18 PENDING` 为本任务前已存在项，与本文件无关）。本任务仅新增本文件。
