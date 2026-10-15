# 社区成果系统性吸收审计（gun-nac-ko 全资产过筛）

> 【2026-10-16】captain 亲办（专项审计两派两断后收口）。纪律：社区主张=B 级线索，
> 入结论须我方 ROM 复证（N-0202 五度修正教训）；本审计逐资产裁定。

## 1. 逐资产裁定表

| 资产 | 内容 | 裁定 | 复证状态 |
|---|---|---|---|
| config/rom.json | **JP sha256=08ead6a2…111da44d** | 我方 reference/jp **逐字节同源**（zno5 标准）| A 级（我方 MANIFEST 已录）|
| tools/verify_settings_runtime.py | **隐藏门实验法**：test-only ROM 翻 file=16+PRG*16K-7（=CPU $FFF9）0→1 → 设置屏隐藏字段现形；字段表（manifest settings[name,ram]）+光标 $01AF | **可关 D2 R-D2-1 方案**（实验法+字段面）| B 级→待我方复现（实验方案 §3）|
| translation/text.csv（169 行） | 全文本地图：group/jp_record_offset/jp_text_offset/ppu_address/x/y——**5 组**：credits 43/ending 20/opening 35/settings 23/shop 47 | 表现面补全（E-R 系列 PPU 定位）| B 级（record 格式与我方互证体系兼容）|
| tools/verify_caption_cards.py | **5 卡枚举**（idx 0..4）+强制 `$4C:=1/JMP $9572`（0xe61 patch）+卡装载 `JSR $96D1`（0x1597 patch）+**16px shift 实证**（idx1/2/3 y16→32） | **A3 R-E2/R-E4 残项可关**（机制解释齐）| B 级（钩子与我方 A3 $97D3[$4C] 链一致）|
| docs/ARCHITECTURE.ko.md | **注入钩子全表**：$A97D 故事渲染/$AA00 设置行/$AB20 字幕行事件/$AB90 商店分支/$ACD8 字幕+**结局卡 IRQ 边界校正（=16px 延迟来源）**/$97F1 回收 JP 字幕区/$F340/$F400 关卡名/$D772 名字输出改道/$E256 OAM 方向 | 钩子全与我方 w-p1/A3/D2 对上 | A 级对账（地址逐条吻合）|
| ARCHITECTURE §字幕/结局 | 43 记录+**8 相位滚动像素级检查**（32B 双 tile 包/PPU 队列一次传输上限）；三结局卡（일반/하드/이판사판）=staff roll 后按 START 现形，**彼时才延迟 IRQ 16px** | A3 R-E2 关账依据 | B 级（机制链与我方 A3 §2 一致）|
| ARCHITECTURE §关卡名 | 8×16 精灵×2/字；AREA/CLEAR 公用 tile $00-$0F 保留；1-6 区 vs 7-8 区 tile 校正 0 vs $C1；**$0188 1-125 活跃开场窗=OAM 正向分配**（防闪烁）| N-0200 互证 ✓ | A 级（与我方 $0188 语义同）|
| ARCHITECTURE §AREA-0 | **"AREA-0 用指针表第 0 项画'星之星'"** | A3"AREA-0+八关"假设**社区坐实** | B 级（我方 A3 已有加载器行索引 0 空位旁证）|
| ARCHITECTURE §遗留 | **社区自己也没跑通 8 区全程验证**（"실기/其他模拟器仍待"）；其验证对选区/CLEAR/结局直通用 test fixture（=同我方 G-S5-6 注入锚捷径）| 真机 8 关可通关=社区同款未竟事业（口径互证）| — |
| config/story_pointers.json | 指针表（pointer_file→cpu_target $A97D 族→text_file）| 故事页指针族（A3 $A925 链上游）| B 级 |
| translation/layout_overrides.json+build_stage_patch.py | 行布局/关卡名补丁工程 | 表现面参考 | 未复证 |
| tools/run_mesen.py+prepare_inputs.py | Mesen Python harness（core_path/Runner/像素断言）| **B2 v4 bot 参考**（像素断言模式）| 工具参考 |
| docs/FONTS/TESTING/BUILD 等 | 字体 8×16 双 tile/验证环境/构建 | 快扫无新机制主张 | — |
| 音频 | **社区零音频资产**（全仓无 audio/music/sound）| **D1 维持新疆土**（社区不覆盖）| 事实 |

## 2. 顶层收益（可关缺口/加速）

1. **D2 R-D2-1（隐藏选项效果）**：实验法+字段面齐——隐藏字段=设置屏扩展项（소리 듣기=SOUND TEST/적의 반격=ENEMY RETURN FIRE 等），$FFF9 门开现形。**待我方复现实验收口**（§3 方案）
2. **A3 R-E2（IRQ 16px）关账**：=结局卡出现时的图形/字体制换 IRQ 延迟+底部位置校正（$ACD8 注入）——机制解释齐
3. **A3 R-E4（卡渲染）收窄**：5 卡像素级语义+16px y-shift（idx1/2/3）+卡装载钩子 $96D1
4. **AREA-0 假设坐实**（社区明文）
5. 表现面：169 行文本地图（5 组）+钩子全表=UI/文本语义考证加速
6. **口径互证**：真机 8 关通关验证=社区也未竟（其 fixture 捷径与我方注入锚同款）——我方 G-S5-6 分层口径获外部印证
7. **B2 v4 参考**：run_mesen 像素断言/输入脚本模式

## 3. 待授权实验方案（D2 R-D2-1 收口）

复制 ROM 至 analysis/ 作 **TEST-ONLY 标注副本**（锚文件永不动），翻 $FFF9 0→1 → Mesen 跑设置屏 → 隐藏字段行为取证（SOUND TEST/ENEMY RETURN FIRE 效果）。与社区 verify_settings_runtime 同法。**需用户授权**（D3"ROM 字节永不动"约束的 test-only 副本边界确认）。

## 4. 纪律自查

社区主张全部降 B 级入档（除 rom.json hash=A 级同源）；复证状态逐条标注；未复证入假设区。
