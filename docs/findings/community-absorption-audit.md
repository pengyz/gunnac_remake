# 社区成果系统性吸收审计：zno5/gun-nac-ko 全资产裁定

> 【2026-10-15】重派收口（前次执行体中断零落盘；本文件为完整版，取代同名 41 行粗稿——
> 与前稿差异见 §10.3 更正表）。对象：`analysis/community/gun-nac-ko/` 全部未挖资产
> （tools/ 25 脚本、config/ 4、translation/ 4、docs/ 7 + README/CONTRIBUTING/publish_manifest/CI）。
> 前次仅采 `docs/NES_LOCALIZATION_NOTES.ko.md`（11 项互证 + N-0200..0206，
> 见 [community-research-gun-nac-ko.md](community-research-gun-nac-ko.md)）。
> **纪律**：社区主张=B 级线索；入结论前必须我方 ROM/trace 复证（N-0202 五度修正教训）；
> 未复证入 §10 假设区（C 级）。逐节增量落盘。

## 0. 方法、口径与复现

### 0.1 复证通道（本次新开，全部字节复证的共同基础）

社区 `config/rom.json` 五指纹与我方 `reference/jp/gunnac_jp.nes` **5/5 吻合**
（262,160B / CRC32 `F39B41A4` / MD5 `029daa68…` / SHA1 `48ce296d…` / SHA256 `08ead6a2…111da44d`）
⇒ 社区一切"JP 文件偏移"主张可直接在我方 JP 副本上做 **A 级字节复证**；`rom/gunnac_cn.nes`
（九班）与 `reference/us/gunnac_us.nes` 做跨版对照。**我方 ROM 从未改动**（社区亦只动 build/ 副本）。

### 0.2 复现命令（本文件全部字节数值同源于此脚本，可重跑）

```bash
cd /home/peng/workspace/gunnac_remake
python3 - <<'EOF'
import hashlib, zlib, json
from pathlib import Path
jp = Path('reference/jp/gunnac_jp.nes').read_bytes()
ko = Path('analysis/community/gun-nac-ko')
spec = json.loads((ko/'config/rom.json').read_text())
fp = dict(bytes=len(jp), crc32=f'{zlib.crc32(jp)&0xffffffff:08X}',
          md5=hashlib.md5(jp).hexdigest(), sha1=hashlib.sha1(jp).hexdigest(),
          sha256=hashlib.sha256(jp).hexdigest())
print('fingerprints', {k: fp[k]==spec[k] for k in fp})            # 5/5 True
print('FFF9', hex(jp[16+jp[4]*16384-7]), 'FFF8', hex(jp[16+jp[4]*16384-8]))  # 0x00 / 0x0a
sp = json.loads((ko/'config/story_pointers.json').read_text())
print('story_pointers', sum(int.from_bytes(jp[int(p['pointer_file'],16):][:2],'little')
      == int(p['cpu_target'],16) for p in sp), '/', len(sp))       # 19/19
for i in range(5):                                                 # 场景表 5 槽
    print('slot%d' % i, jp[0x1755+i*8:0x175d+i*8].hex(' '))
print('boot', jp[0xE61:0xE68].hex(' '))                            # a9 01 20 45 ed a9 91
for cpu, off in ((0x96D6,0x16E6),(0x96DE,0x16EE),(0x96E2,0x16F2),
                 (0xA90D,0x291D),(0xA790,0x27A0),(0xA6A2,0x26B2),
                 (0xD772,0x1D782),(0xE256,0x1E266)):
    print(hex(cpu), jp[off:off+3].hex(' '))
print('D7B9 tile-correction', jp[0x1D7C9:0x1D7C9+9].hex(' '))      # 00*7 c1 c1
print('F340', jp[0x1F350:0x1F358].hex(' '), 'F400', jp[0x1F410:0x1F418].hex(' '))  # ff…
EOF
```

实测：指纹 5/5；`$FFF9=0x00`/`$FFF8=0x0a`；story_pointers **19/19**；场景表 5 槽见 §3.1；
8 处钩点 before-bytes **全中**；`$D7B9`= `00 00 00 00 00 00 00 c1 c1`；`$F340/$F400` 全 FF（原版真空区）。

### 0.3 计数勘误

任务卡称 tools/ 15 脚本——实测 **25 个 .py**（下表全裁，验收按实测清单计）。
公开文件共 44（publish_manifest 全列）+ .gitignore/.gitattributes/.githooks/.github。

### 0.4 分级口径

| 级 | 本文用法 |
|---|---|
| A | 我方字节实证（JP/CN/US hexdump、seed 反汇编行、§0.2 可重跑） |
| B | 社区代码/manifest 语义 + 我方证据咬合（钩点/地址族/计数吻合），或我方静态推理 |
| C | 社区独有主张无我方咬合 ⇒ 只入 §10 假设区，禁止据此改台账/入 spec |

---

## 1. 逐资产裁定表 A：tools/ 25 脚本

| # | 资产 | 可吸收内容 | 复证 | 可关缺口 |
|---|---|---|---|---|
| T01 | verify_settings_runtime.py | ① `$FFF9` 0→1 **test-only ROM 配方**（off=`16+prg*16384-7`、断言原值 0、只写副本、`release_rom_sha256`/`executed_rom_sha256` 双哈希留痕）；② 隐藏行=设置屏第 6-8 行；③ **AREA 编辑需 SOUND=05**（`area_locked_without_sound_05`）；④ area 值域 0-8、sound 0x01-0x48、光标 $1AF 游程/上下界；⑤ BDF→2bpp→阈值掩码逐像素断言法 | **A**（$FFF9 偏移/值/公式，§0.2）+B（行语义） | **D2/R-D2-1**；E-R03 |
| T02 | verify_caption_cards.py | 5 场景（idx 0..4）逐卡渲染、图区像素保真断言（idx1-3 图下移 16px：top 16→32）、`LDA #idx/JSR $96D1` 直装载场景 idx | B（与场景表 5 槽吻合） | R-E4 |
| T03 | verify_credits_transition.py | ① `$2F/$30`=字幕流指针（与我方 A3 `$2F:$30:=$9955` 互证）；② 流指针 `$8000..$8003`/`≥$A800` 判据分残余故事文字 vs 字幕；③ 强控结局字体对照 ROM→断言换字体时机 | B | R-E2（机制） |
| T04 | run_mesen.py | **无头 libretro Mesen runner**（ctypes：逐帧/按键集/全 RAM dump/PNG/音频 hash/trace JSON）；`--press frame:btn:dur` 输入脚本 | B（接口自洽） | B2 v4（§6） |
| T05 | prepare_inputs.py | ① 输入五指纹 fail-closed（含 16B 头）；② jp_text.json 本地导出；③ story_pointers 断言=指针 LE16==cpu_target | **A**（19/19 我方 JP 全过） | 对账 §8；A3 |
| T06 | rom_text.py | 双记录族解码明文（0x5E 假名开关/0x20-0x5D ASCII/其余 TILE；FE 行事件/FF 流尾；credits=位置字节族；cpu=off+0x7FF0） | A（前次已互证 t2/t138/t171） | — |
| T07 | verify_story_runtime.py | **19 故事帧**逐帧掩码验证（opening 11+ending 8）；`$12/$13`=故事流指针（完成停 000000 终止符）；逐字像素比对 | A（19=11+8 计数）+B（$12/$13） | A3/R-E3 |
| T08 | verify_credits_runtime.py | 字幕滚动 **8 纵向像素相位**逐相断言（8/8 齐才过）；post-roll START 两次回放含松键等待；end_pointer 判据 | B（方法↔t189 相位系列） | R-E2（方法） |
| T09 | build_stage_patch.py | ① **$F340=关卡名精灵渲染器/$F400=x,y-1,tile 表**（9 槽指针 18B+0 终止三元组；≤8 精灵/行断言）；② `$D772` 钩点 before=`ac 80 01`（`LDY $0180`）；③ `$E256` 钩点 before=`20 18 cb`，插入 `$0188<0x7E` 时清 **$54 bit6（OAM 分配方向）**；④ 复用原表 `$D7B9` 每关 tile 校正（0/$C1）；⑤ 16×16 字=4×8×8 tile 四象限、碎片共享 13 字→24 碎→48 tile；⑥ CHR R4/R5 共享 1KiB bank 注；⑦ IPS 回环自检 | **A**（两钩点+空白区+D7B9）/B（渲染语义） | **§5 布局工程** |
| T10 | build_caption_patch.py | ① **场景记录 8B=[chr_b1,chr_b0,mt,pal,cmd]；命令链=[8B 包…]+`00 00` 尾；包=[fn LE16]+6 args**：$E9E8 图片包 `[h,w,ids_ptr,dst_ppu]`、$E875 原始包 `[0,n,src,dst]`；② **metatile=5B（4 tile+1 属性）**、图片=h×w metatile id；③ dst_ppu=$2046/$2086、+64=下移 16px（图片落点字段实锤）；④ 6 钩点 before-bytes **A 级全中**（$96D6/$96DE/$96E2/$A90D/$A790/$A6A2）；⑤ 原版图片 IRQ 读 $0406/$0407（滚动）+$0408（latch 默认 $78） | **A**（钩点/命令记录） | **R-E4（大部）**；R-E2 |
| T11 | build_story_patch.py | ① ROM 扩张术：PRG/CHR 128K→256K、固定 16KiB 复制到尾槽保向量、旧副本保留；② story_pointers 19 条 repoint；③ `$A97D` 故事渲染器 + **字体 IRQ 仲裁（$30≥$A8=故事字体）**；④ 钩点 `20 d1 e6`/`a9 00 8d` | **A**（钩点） | A3/R-E2/R-E3 |
| T12 | build_credits_patch.py | ① 字幕事件化（FE 空行/双半行打包→事件表）；② **字体切换规则：流指针 $8000..$8003 保持结局字体、首文本事件切字幕字体**；③ 双 32B 半行=单包预算内（t94/t117 互证）；④ $95A6/$95AA/$9642/$A9C4 钩点 + $AE60 切换例程 | B | R-E2（字幕侧） |
| T13 | build_settings_patch.py | ① **8 设置行 RAM 全图**：area=$0180/sound=$01AE/difficulty=$004C/counterattack=$004E/priority=$00F1/weapon=$01B6/bomb=$01B7/death=$01BE；② 隐藏 3 行=weapon/bomb/death（`i==5` 起 `LDA $FFF9/BEQ`）；③ **$FFF9 门在补丁中保留不动**；④ "$F1 场外可 $FF，勿作裸索引"；⑤ 全行宽 ERASE 清残影 | B（$0180/$004C/$004E 已有我方 A 级同址；$01AE/$01AF seed 咬合） | **D2**；E-R03 |
| T14 | build_shop_patch.py | ① 动态值读取器读点表：$018E/$34+1/$33(min7+1)/$3C+1/**$0405(ASCII F/B/T/W)/$0404(计数)/$0403 索引 $0178(>>6 型/&63 量)**；② 原版更新例程指针 **$9FA1(钱)/$A13F(武器)/$A1E7(连射)** + 原版数字输出 **$E959/$E97D**；③ 仅钩商店 10 处 `JSR $E6D1/$E6E3` 调用点，不全局改道；④ 价目行模板 `N개 N엔` | B（地址族） | E-R05；L5 商店购买侧 |
| T15 | verify_shop_runtime.py | ① 6 场景（welcome/purchase/shipping/delivery/blocked/converter）RAM+像素双断言；② **价 10/6/4 动态实证**（2 次武器升级 −20、连射×4 各 −6、发货×N 各 −4、量封顶 63）；③ wing 买断置 `$32 bit7`、converter 置 `$32 bit5`；④ `$0400` 主菜单光标/$0401 横光标/$0403<8 哨兵；⑤ "RAM fixture 只写运行内存、绝不入发行 ROM" + fixture 逐条留痕 | B | E-R05/06/07/08；N-0201 打包 |
| T16 | verify_stage_clear.py | `ram[$0188]=0x7E` fixture 调出原版 AREA/CLEAR 渲染；0/1/7/8 区逐像素回归 | B | §5 |
| T17 | verify_stage_runtime.py | ① 9 区（AREA-0..8）真渲染路径逐关验证；② **tile 校正 `offset=0xC1 if area>=7 else 0`**（↔$D7B9）；③ OAM 断言 `[y-1, tile+off, 0, x]`、8 精灵/行计数；④ `$01A3-$01A8`=MMC3 bank 影子、`$0188` 递减至 0 介绍毕；⑤ 229 连续帧/关像素验证法 | **A**（offset 0/$C1 ↔ $D7B9 字节） | §5；N-0200 |
| T18 | build_font_preview.py | 离线文本渲染管线（BDF→1bpp 掩码→2bpp→CHR/NAM/PNG/HTML 预览）；28 格/行、5 行/页、每页 ≤256 pattern 断言；限制自述诚实 | B | 方法（golden 文本渲染） |
| T19 | font_bitmap.py | BDF 直读光栅 + PNG 读写（无字体数据内嵌） | B | — |
| T20 | font_policy.py | B2 混排策略：**原版 ASCII 字模=file 0x3F010+code*16 直取**（tile 码=ASCII 码再证）、y=5-12 放置、`tile[y]\|tile[y+8]` 2bpp 合成；断言 JP sha | **A**（CHR 偏移公式↔t171） | — |
| T21 | fetch_fonts.py | 固定 commit+sha 的官方 BDF 拉取（8MiB 上限、原子写、失败即停） | B | — |
| T22 | check_sources.py | 无资产源检：语法/键唯一/布局与翻译 whitespace-insensitive 一致性/display_overrides 同步 | B | 方法 |
| T23 | export_source.py | 仅发布清单文本打包（zip 自校验） | B | — |
| T24 | audit_public.py | **fail-closed 发布审计**：BLOCKED 扩展（含 **.wav/.mp3/.nsf** 等）+MAGIC+密钥模式+原文列禁令+`publish_manifest` 白名单闭环 | **A**（无音频=事实面，§8） | 对账 §8 |
| T25 | build.py | 7 步构建链（font_preview→story→settings→credits→shop→caption→stage）逐步 log+断言+参考哈希比对；禁 `python -O` | B | 方法 |

## 2. 逐资产裁定表 B：config/ 4 + translation/ 4 + docs/ 7 + 杂项

| # | 资产 | 可吸收内容 | 复证 | 可关缺口 |
|---|---|---|---|---|
| C1 | config/rom.json | JP 五指纹+PRG/CHR 128K+mapper4 | **A**（与我方 JP 副本 5/5） | §8 |
| C2 | config/reference_build.json | 发行参照：韩化成品 524,304B（=256K+256K+头）sha `2172180d…` + IPS sha + 4 个 translation 输入 sha 锁链 | A（形态可核） | §8 |
| C3 | config/story_pointers.json | **19 条故事页指针族**（pointer_file→LE16 cpu_target；text_file=cpu-0x7FF0；opening 11+ending 8） | **A**（19/19） | A3/R-E3 |
| C4 | config/font_sources.json | 3 BDF 固定 commit+sha+许可证 URL；"不入库、本地下载"政策 | B | — |
| L1 | translation/text.csv | **168 行全文本映射**（settings 23/shop 47/opening 35/ending 20/credits 43=43 记录）；jp_record_offset/jp_text_offset 稳定键+PPU 落点+`{}`模板+review_status 分类（배치용 공백/로고 유지/원문 표기 유지） | A（结构族↔t2/t138）+B（语义） | **E-R03/04/05/06/07/08** |
| L2 | translation/graphics.csv | 31 项图形文本决策（14 实现/17 原样保留）：stage_0..8（**AREA-0=隐藏关**"별의 별"）、5 图片字幕（ending_robot/normal/hard/puttsun+opening_bubble）、fire_* 4 项位置未定背景 tile、money_unit/bank/coin/robot_gold 装饰、title/area/clear/continue/game_over 公共 UI、logos（含"무기 식별"武器图形码） | B | R-E4；E-R05 |
| L3 | translation/layout_overrides.json | 10 条故事记录逐行重排（键=jp_text_offset）——与 text.csv whitespace-insensitive 一致性由 check_sources 门禁 | B | §5 |
| L4 | translation/display_overrides.json | **7 条原文英文串**：`AREA-`/`SOUND TEST-`/`GAME LEVEL-`/`WEAPON-{n}`/`BOMBER-{c}`/`DEATH`/`NON DEATH` | B（与我方 $22C6/$2306 落点咬合） | **E-R03/H-04** |
| D1 | docs/ARCHITECTURE.ko.md | ① **注入钩子全表**（$A97D/$AA00/$AB20/$AB90/$ACD8/$97F1/$F340/$F400/$D772/$E256）；② 扩张内存布局（bank16-19 数据/CHR 80-AB 字模/CHR $1C tile $10-3F=48 关卡名字模）；③ 覆盖层工程原则（钩调用点不换全局、bank 切换前压栈、空白区 assert）；④ 设置 flag 可 $FF；⑤ 字幕 32B 双 tile 包/8 相位；⑥ 三结局卡=staff roll 后 START 现形、**"彼时才把图形/字体制换 IRQ 延迟 16px 并校正底部"**（=补丁行为！见 §3.3）；⑦ 关卡名 2 精灵/字、tile 校正 0/$C1、$0188 1-125 正向分配防闪烁；⑧ **AREA-0 用指针表第 0 项** | A（钩子地址逐条与我方 w-p1/A3/D2 对上） | §5/§7 |
| D2 | docs/TESTING.ko.md | ① 验证范围基线：**opening 11+ending 8 屏**、settings 106 检查点、43 记录×8 相位、9 关×229 帧、AREA/CLEAR 0/1/7/8、5 字幕、shop 126 检查 22 输出；② **未竟自述**：8 关全程/全部隐藏分支/实机/其他模拟器未验、文本时序改变后帧一致性不承诺；③ fixture 与发行分离声明；④ 2026-10-11 两笔修正记录（AREA-0 CHR 引用错误修复；结尾→字幕字体切换时机修复） | B | §3/§7 |
| D3 | docs/FONTS.ko.md | BDF 直读政策、基线 14、影 (+1,+1)/x-1、"字形计数≠tile 计数"、BDF ENCODING 不可信须按实际字符核 | B（↔glyph-shape-truth/cjk-quadrant） | — |
| D4 | docs/BUILD.ko.md | 7 步构建链+每步 manifest sha 链+翻译/布局分离编辑规则+容量断言不许关 | B | 方法 |
| D5 | docs/RIGHTS.ko.md | 法务边界（工具 MIT、翻译/原作衍生/字体排除）——对 RIGHTS 登记类有参考，无 ROM 语义 | — | — |
| D6 | docs/GITHUB.ko.md | 发布流程（IPS ZIP+校验和+prerelease 直至完跑验证） | — | — |
| D7 | docs/NES_LOCALIZATION_NOTES.ko.md | 前次已采（11 项互证+N-0200..0206） | A（已收） | — |
| M1 | README.md | 范围自述（opening/ending/settings/credits/shop/5 字幕/9 关卡名；31 图形决策 14+17；PRG/CHR 256K） | B | §8 |
| M2 | CONTRIBUTING.md | 贡献纪律（问题/行为/检查/已知界限必记；ROM 版本哈希+CPU/file/PPU 地址类型必分；fixture 必须自标）——与我方证据纪律同构 | — | — |
| M3 | publish_manifest.json + .githooks + .github workflow | 发布白名单 44 文件闭环；pre-commit/CI 只跑无资产源检 | A（44 文件实测零音频/零 ROM/零字体） | §8 |
| M4 | （前次粗稿 41 行版） | 其独有主张已逐条核入本文；2 处更正见 §10.3 | — | — |

---

## 3. 缺口裁定 1+2：D2 残项与 A3 残项

### 3.1 A3 场景表 5 槽 —— A 级复证（本次最大单点收获）

我方 JP 副本 file 0x1755（CPU $9745）起 5×8B 场景表实测；US/CN 选择表 `$97D3`（file 0x17E3）=`00 01 02 03` 实测：

| 槽 | CHR (b1/b0) | metatile | palette | cmd | 社区命名（graphics.csv） | 我方 A3 对应 |
|---|---|---|---|---|---|---|
| 0 | `2E/2C` | $BA79 | $1777 | $976D | ending_robot（"이걸로 이겼다고 생각하지 마!"） | scene0（A3 记"卡后收尾屏"） |
| 1 | `5E/5C` | $BB5A | $1777 | $9791 | ending_normal（보통 结局） | scene1 ✓ |
| 2 | `22/20` | $B8A8 | $1777 | $979B | ending_hard（어려움 结局） | scene2 ✓ |
| 3 | `22/20` | $B8A8 | $1777 | $97A5 | ending_puttsun（이판사판 结局） | scene3 ✓ |
| 4 | `2E/2C` | $BA79 | **$9777（bit15）** | $976D（同槽0） | robot 复用槽（调色板标志差） | （A3 未记第 5 槽） |

- **CHR 对逐值与 A3 吻合（A 级）**；JP/US/CN 同内容不同表址（JP 选择表在 file 0x1751；US/CN 在 $97D3）。
- 命令记录实测（槽0 `e8 e9 05 09 4f a8 86 20`）= fn $E9E8 + h5×w9 + ids $A84F + dst $2086；
  槽 0 dst=$2086 vs 槽 1-3 dst=$2046（差 $40=2 tile 行=16px）→ **图片 PPU 落点字段实锤**，
  与社区"일반·하드·이판사판 카片만 y16→32 下移、robot/opening 不动"完全自洽。
- 装载链 A 级再现：seed `bank00:9734 B9 D3 97 LDA $97D3,Y`；`bank00:985C/98FA、bank01:A9F5 JSR $E9DE`
  （A3"scene1 链$9855"紧邻 985C ✓）。

### 3.2 A3 残项逐条裁定

| 残项 | 裁定 | 依据 |
|---|---|---|
| **R-E4**（$E9DE tile 块记录语义） | **大部闭合（B+）**：包格式/字段/metatile 切分/链尾齐；剩 fn 分派细节与属性字节位义，靶点=seed 逐指令展开 $E9DE 一次 | T10 构造者视角 + §3.1 命令记录字节 + seed `JSR $E9DE`×3 |
| **R-E2**（结局卡 IRQ 16px+底部滚动校正） | **前提勘误 + 收窄**：16px 延迟=社区补丁行为（其卡下移 16px 后的分割跟随修正），非原版待证数值；残项收窄为"原版图片 IRQ 分割机制数值"（变量 $0406/$0407/$0408、默认 $78、钩点 $A790=`ad 07 04` 已 A 级） | ARCHITECTURE"그때만 … 16px 늦추고"+build_caption_patch 注释/代码；钩点字节 §0.2 |
| **R-E3**（$A925 页序列记录格式逐字段） | **维持开放**：社区 story 渲染走自有 `<HB` 流格式，未解剖原版页序列记录；19 指针族 A 级旁证到位 | C3/T07 |
| **R-E1** | 前次已收（43 条实数） | — |
| **R-E5** | 前次已收（H7 美版自带） | — |
| **$AFE4 故事页族** | 存在性/指针表形态获 **A 级跨版旁证**（JP 19 条 vs US 5 指针 4 页=翻译重排再量化：JP ending 8 条 3/4/5 行页 vs US 4 页 4 行） | C3 + A3 |

### 3.3 缺口 1（D2/R-D2-1）：隐藏门开启效果 —— **部分可定**

| 开启效果（$FFF9 0→1） | 社区证据 | 我方证据 | 状态 |
|---|---|---|---|
| 设置屏追加 3 行 WEAPON-{1..5}（$01B6）/BOMBER-{F,B,T,W}（$01B7）/DEATH·NON DEATH（$01BE） | T13 specs+T01 `--hidden` 5→8 行断言面 | xref 92DE 追加 $9519 记录落点 $22C6/$2306="WEAPON-0"/"BOMBER-0" | **互证 ✓** |
| 行尾值=运行时改写 | L4 模板+T13 值表（5 值/4 值） | E-R03"尾位数字疑为运行时改写→H-04"；948C 表 $95F5,Y 写 $230D | **H-04 可销**（B 级） |
| AREA 编辑需 SOUND=05（普通版锁定） | T01 断言+注 | xref 9387 旁路条件 flag==0&&$1AF==0&&$1AE≠5 | **互证 ✓** |
| $01AF=光标 / $01AE=SOUND TEST 值 / $0180=AREA(0-8) | T13+T01 | seed bank00:9306/930B STA $1AE/$1AF、932C..937F LDA/LDY $1AF；xref 93EE"$1AF 语义未证"**可销** | **A/B 收口** |
| $004C=GAME LEVEL(4) / $004E=反撃 flag / $00F1=priority flag | T13 | A3 $4C（A 级）；$4E/$F1 新语义 | B |
| $018E:=30（默认 10） | — | 我方 xref 9C7D 三重门 | 已在我方表 |

**不能定（维持开放）**：`sub_95F9` 语义、$F270 按键支路实效、$E08E 放松 $0180 钳制的动态值域。
**闭合路径（吸收社区配方）**：test-only 副本翻 $FFF9 + Mesen 全 RAM watch，一次运行可同验 3 行渲染、
$018E 初值 30、$E08E 钳制差、9387 旁路差；T01 即脚手架，其 `release/executed` 双 sha 留痕做法照抄。
**纪律注**：社区"test-only ROM、发行不带作弊位"与 D3 同构——实验只动临时副本（需用户授权边界确认，见 §9）。

---

## 4. 缺口裁定 3：表现面（text.csv/graphics.csv vs E-R 系列）

| E-R | 我方登记（completeness） | 社区对应 | 裁定 |
|---|---|---|---|
| E-R01 | HI SCORE/SCORE+版权串 | graphics.csv title_start/config/high/score、logos 保留 | 旁证登记（图形 tile 层）；版权串不覆盖 |
| E-R02 | `10000000000` | 无 | 不覆盖（H-03 保持） |
| E-R03 | WEAPON-0/BOMBER-0（尾位疑运行时改写→H-04） | L4 `WEAPON-{n}`/`BOMBER-{c}`+T13 值表+T01 隐藏行 | **B 语义闭合**（设置屏隐藏行标签+行尾运行时写）→H-04 可销，trace 一次入 spec |
| E-R04 | `-------$` 金钱显示行 | text.csv `가진 돈: {소지금}엔`（PPU 0x2203）+fid0 宽 3+graphics money_unit | **B 闭合**（破折号=数字位占位+货币符模板） |
| E-R05 | 价目 4 条（$10/$6/$4，4 号无价串@8993 ff 截断） | text.csv 行 41-44：**1.무기 구입 1개 10엔 / 2.연사력 1회 6엔 / 3.폭탄 보내기 1개 4엔 / 4.나가기（退出，无价）** + T15 价 10/6/4 动态实证 | **B 闭合（4 号=退出项之谜解开）**；图形码商品名=로고/무기 식별"原样保留"⇒ **图形码语义仍开**（可从我方 CHR 直读推进） |
| E-R06 | 平行串 `1 2 3 4 5 ` | text.csv 行 48 `1 2 3 4 5 날개`=武器菜单 5 级+WING 选项（T15：wing 买断置 $32 bit7） | **B 闭合**（行语义=武器升级菜单行） |
| E-R07 | 平行串 `1..8` | 行 59=发货 AREA 选择行 | **B 闭合** |
| E-R08 | 平行串 `F B T W` | 行 63=炸弹型选择行；$0405=ASCII 字母、ship>>6→'FBTW'[i] | **B 闭合** |
| E-R09/E-R10 | opening/ending 文本 | text.csv opening 35/ending 20 行；story 11+8 页（19 指针 A 级） | 计数互证（11+8=19）；文本内容不增益 |
| E-R11 | CONTINUE/GAME OVER | graphics.csv continue/game_over 图形 tile | 旁证登记 |
| E-R12 | `\t@2011 JIUBAN` 汉化标记 | 无（社区 JP 基底） | 不覆盖 |

**总判**：text.csv=可直接吸收的 **JP 文本地址索引+逐行 golden 格式模板**（168 行 5 组，键设计与我方
台账双层同构）；E-R03/04/05/06/07/08 六项语义闭合候选全部 **B 级**——入 spec 前各需一次我方复证
（像素行核 `shots/auto_f01080.png` 已有商店全屏即可半自动；E-R03 行尾写 trace 一次）。
graphics.csv 补齐 E-R05 图形码商品名的"未译保留"事实面（商品名图形码语义维持开放）。

---

## 5. 缺口裁定 4：布局工程（layout_overrides+build_stage_patch vs w-p1-relocation-audit）

| 社区事实（复证状态） | 对我方 w-p1/布局的旁证 |
|---|---|
| `$F340/$F400` 注入于**原版 FF 空白区**（我方 JP 实测 968/1024 FF）（A） | 社区关卡名 renderer 是**新增实现**而非原址改写；我方 w-p1"九班关卡名走另一机制"两端并立：JP 精灵发射块（$D772 51B）+ 社区重实现 renderer vs 九班删除块 |
| `$D772` 钩点 before=`ac 80 01`（`LDY $0180`）（A） | **w-p1 反例 1 语义再证**：该块=关卡介绍期 OAM 发射（读 $0180 关号）；社区选择钩它=社区认可其为关卡名输出路径 |
| `$D7B9` 每关 tile 校正表=`00×7, C1, C1`（A）+ T17 `offset=0xC1 if area>=7` | **新 A 级数据表**：AREA-0..6 校正 0、AREA-7/8 校正 $C1（同 pattern 映不同 PPU 位置）；w-p1"孤儿数据表 $DABC/$DACD"家族新成员，可入 remap-table 登记 |
| `$E256` 钩点 before=`20 18 cb`；$0188∈[1,125] 清 **$54 bit6**=OAM 正向分配（A 钩点+B 语义） | t145 槽位漂移根因链补强（$0188 介绍窗+分配方向 flag）；w-p1 R3（开场可见影响面）可借 T17 OAM dump 法复核 |
| 8 精灵/行上限断言、229 帧/关验证、`[y-1,tile+off,0,x]` OAM 形态（B） | D4 quirk 互证；我方 L4 视觉基线可引其"逐关逐帧相位"采样面 |
| layout_overrides=10 条故事记录行重排（28 格/行、5 行/页、特殊起始行 19/21）（B） | 纯翻译布局工程，无 ROM 语义；"翻译与布局分离+一致性门禁"做法可借鉴到我方文本 golden |
| ROM 扩张 256K+固定 bank 复制保向量（B） | 与九班原地覆盖（2240B/3 bank）**两种路线对照**：社区=扩张+钩调用点，九班=原地替换+搬移。ARCHITECTURE 覆盖层结构表 = D3/九班覆盖层 2240B 的**工程同型旁证**（N1 5B 表 vs metatile 5B=4tile+1 属性的形态类比仍 **C 级**，见 §10） |

---

## 6. 缺口裁定 5：Mesen 自动化（run_mesen+prepare_inputs vs B2 v4 bot）

B2 v4 bot=我方 `tools/tas_bot.py/.lua`（Mesen2 CLI+Lua、record-to-scenario 闭环、poke 禁用）。裁定：

| 社区资产 | 对 B2 v4 的参考价值 | 裁定 |
|---|---|---|
| run_mesen.py Runner（逐帧/按键/RAM/PNG/音频 hash） | 帧推进-断言-采集循环模式同构；但 libretro 接口无 Lua/PPU 状态/实时决策面，**不能替代**我方 Mesen2 主轨 | 工程参考（低-中） |
| verify_* 的**像素掩码断言**（BDF→2bpp→阈值） | bot 通关判据的 UI 面（关卡名/AREA/CLEAR/HUD）可引此法做可见性 golden | **可吸收（中）** |
| **RAM fixture 纪律**（fixture 只写运行内存、逐条留痕、绝不入发行） | 与我方 poke 禁用红线/G-S5-6 injected 锚分离同构，可写进 bot 场景 DSL 文档 | **可吸收（纪律）** |
| release/executed 双 sha 留痕 | R-D2-1 test-only 实验、bot 场景 fixture 的审计字段直接照抄 | **可吸收（纪律）** |
| audio_frames/audio_sha256/无音频帧计数 | **D1 音频 golden 哈希机制**（首个可抄方案） | **可吸收（D1）** |
| prepare_inputs 五指纹 fail-closed + 本地导出物不入库 | 与我方 sha256 门禁同类；"衍生文本产物本地化"值得仿 | 工程参考 |

---

## 7. 缺口裁定 6：docs 余 7 份快扫

- **ARCHITECTURE（重头）**：注入钩子全表与我方 w-p1/A3/D2 地址逐条对上（A 级对账）；内存布局表
  （bank16-19 文本/CHR 分族/CHR $1C 48 关卡字模）；覆盖层工程原则=**D3/九班覆盖层 2240B 的工程同型旁证**
  （同为"翻译支持数据+渲染支持代码、零游戏逻辑改动"的覆盖层形态；N1 5B 表形态类比留 C 级）；
  "三结局卡=staff roll 后 START 现形、彼时才 IRQ 延迟 16px"→§3.3 前提勘误源。
- **TESTING**：验证范围基线（11+8 屏/106 检查点/43×8 相位/9×229 帧/126 检查）+**未竟自述**
  （8 关全程/隐藏分支/实机/其他模拟器未验）——"真机 8 关可通关验证=社区同款未竟事业"口径互证，
  我方 G-S5-6 分层口径获外部印证（fixture 捷径同款：选区/CLEAR/结局直通均 test fixture）。
- **FONTS**：BDF 直读/基线/阴影/四象限计数↔我方 glyph-shape-truth、cjk-quadrant-rule 互证；
  "BDF ENCODING 不可信"gotcha 可入 knowledge。
- **BUILD**：7 步构建链+manifest sha 链+断言不许关——工程参考。
- **RIGHTS/GITHUB/CONTRIBUTING**：法务/发布/贡献纪律；`audit_public` fail-closed 发布审计
  （白名单+MAGIC+密钥扫描+原文列禁令）值得我方 tools 借鉴；无 ROM 语义。
- **README**：范围/计数自述（31 图形决策 14+17；PRG/CHR 256K；"最后的关卡名工作未再扩张"=用 FF 空白区）。

---

## 8. 对账（reference_build / rom.json vs 我方三锚；音频=D1）

| 项 | 社区 | 我方 | 裁定 |
|---|---|---|---|
| 输入 ROM | JP `08ead6a2…`（rom.json 五指纹） | `reference/jp/gunnac_jp.nes` 同 sha | **A 级同源 ✓**（zno5 社区标准=JP；我方基准=九班 CN `4d76901c…`，US 参照亦持有） |
| 参照构建 | 韩化成品 524,304B（256K+256K）sha `2172180d…`+IPS `749f0d0a…` | 我方三锚均为 262,160B 原生 128K+128K | 形态差异=路线差异（扩张 vs 原地）；无可共用字节面，**结构知识全部可共用** |
| 文本区偏移 | JP：settings 0x1471-…/shop 0x1FB1-…/opening 0x298D-…/ending 0x2CEA-…/credits 0x1801-… | 九班不同（翻译替换） | 定位器用途，勿跨版直搬（N-0202 教训） |
| **音频** | **零音频资产**（44 公开文件实测无 wav/mp3/nsf/flac/ogg/vgm/pcm；audit_public 明令封禁；无任何音频语义主张；仅 audio_frames/hash 作活性信号） | D1 音频=新疆土 | **D1 维持新疆土 ✓**；可吸收其 audio hash 机制（§6） |

---

## 9. 收益总结

**可关/收窄缺口 6 项**：
1. **R-E4 大部闭合**（包格式/metatile 5B/dst_ppu 字段/5 槽场景表 A 级）——剩 $E9DE 分派细节一处靶点。
2. **R-E2 前提勘误+收窄**（16px=社区补丁行为，非原版数值；原版 IRQ 变量/钩点已 A 级）。
3. **E-R03/H-04 闭合候选**（WEAPON-/BOMBER- 模板+运行时行尾写）。
4. **E-R05 之谜解开**（4 号=退出项无价）+ E-R04/06/07/08 语义闭合候选（均 B 级待一次复证）。
5. **D2/R-D2-1 部分闭合**（隐藏 3 行身份+RAM+SOUND=05 门+$1AF/$1AE 语义收口）+ test-only 实验配方。
6. **w-p1 语义补强**（$D772=关卡介绍 OAM 发射再证；$D7B9 tile 校正表新 A 级数据；$54 bit6 OAM 方向；AREA-0=隐藏关）。

**新开 A 级事实 8 组**：指纹同源、story_pointers 19/19、场景表 5 槽 CHR 对、8 钩点 before-bytes、
$F340/$F400 空白区、$D7B9 校正表、$FFF9/$FFF8 双标志再证、boot 改道位。
**纪律/工具吸收**：双 sha 留痕、RAM fixture 分离、fail-closed 发布审计、音频 hash、
像素掩码断言、翻译/布局分离+一致性门禁、BDF 直读。
**待授权动作 1 项**：R-D2-1 动态收口实验（test-only 副本翻 $FFF9+Mesen watch）——
涉"ROM 字节改动"边界（即使副本），按 D3 须用户授权后执行。

## 10. 假设区与遗留（C 级，禁止据此改台账/入 spec）

### 10.1 C 级假设

| ID | 假设 | 来源 | 闭合条件 |
|---|---|---|---|
| H-A3-1 | 槽0 robot 图=NOVICE 结局图（graphics.csv"스트레스 해소 엔딩"）与我方 A3"$4C=0 无结局卡/scene0=收尾屏"**语义冲突**——scene0 身份（robot 结局图 vs 收尾屏）待定 | §3.1 | JP↔CN 结局段 Mesen 双跑（一次 START 轨迹） |
| H-A3-2 | $1BE=DEATH 设置（社区）与 A3"$1BE≠0→回标题"在结局流的用法冲突 | T13 vs A3 | 同上双跑 |
| H-D2-1 | 隐藏行 5/6/7 在我方九班版同样存在（JP 与 CN 同门同表） | T13 | R-D2-1 实验顺带验 |
| H-N1-1 | 九班 N1 5B 表（w-p1 H3）与社区 metatile 5B（4tile+1attr）**形态同源类比**（字形组合记录） | §5 | N1 运行时读取追踪（w-p1 H3 闭合条件不变） |
| H-Shop-1 | $0178 打包 `type<<6|qty`（社区）与我方 N-0201 观测值 0F/0F/10/30 分解不符 | T14/T15 vs N-0201 | shop watch 扩 $0100-$01FF（R-D2-2 同批） |

### 10.2 遗留清单

- R-D2-1 剩余面：sub_95F9 / $F270 / $E08E 动态值域（实验一次可清）。
- R-E2 新靶点：我方 seed bank01 IRQ 块（`bank01:AB0F STA $C000` 一带）+ Mesen 滚动 trace。
- R-E3 靶点：$A925/$AC87 页序列逐指令 + $E9DE 同族包消费。
- E-R05 图形码商品名语义（社区"原样保留"未译）→ 我方 CHR 直读路线。

### 10.3 对前稿（41 行粗稿）的更正

| 前稿主张 | 更正 | 依据 |
|---|---|---|
| "A3 R-E2 关账（=16px 延迟来源）" | **不关账**：16px=社区补丁行为，原版数值仍待复核（前提勘误） | ARCHITECTURE §그림 자막·엔딩 + build_caption_patch |
| "三结局卡（일반/하드/이판사판）" | **4 图 5 槽**：robot/normal/hard/puttsun + robot 复用槽（palette bit15）；"三张"仅指 staff roll 后 START 现形的卡 | §3.1 场景表 A 级 |
| "text.csv 169 行" | 168 数据行（169 含表头） | 实测 |
| 其余（钩子全表/AREA-0/8 相位/无音频/D2 实验法） | **保留**，已在本文逐条复证升级 | §0.2/§3-8 |

## 11. 验收自查

| 验收 | 状态 |
|---|---|
| ① 资产全覆盖 | tools **25/25**（任务卡"15"为少计，§0.3 勘误）+ config 4/4 + translation 4/4 + docs 7/7 + 杂项 4/4 |
| ② 缺口 1-6 各有结论 | D2=§3.3 部分闭合+配方；A3=§3.2 逐条（R-E4 大部/R-E2 勘误/R-E3 开）；表现面=§4（6 项 E-R 闭合候选）；布局=§5（A 级补强 w-p1）；Mesen=§6（5 条可吸收）；docs=§7 |
| ③ gates 12/12 | 见文末 commandsRun 实跑输出 |
| ④ 收益总结 | §9（6 缺口收窄+8 组 A 级+7 条纪律工具+1 待授权） |

**未证不发明声明**：本文所有"B/C 级"条目未写入 `data/naming/`、未入 `docs/spec/`；
A 级字节数值均可由 §0.2 脚本独立重导；未对任何社区语义直接采信为结论。
> 【2026-10-15 补注（§12 增补轮）】声明微调：§12 的 **A 级字节事实**按任务口径 dated 注记进
> `docs/spec/completeness.md`（E-R03/E-R05/E-R08 三行）；B/C 级仍不入 spec/台账。

---

## 12.

> 【2026-10-15 更正（复核 #86 F1）】本节 bank00 CPU 地址标签系统性 +0x10（N-0094 换算坑第 5 次复发：file+0x8000 忘 −$10）——已按字节反查逐行重标（$9480→$9470 等，ROM 字节定位命令复验）；字节事实/语义不受影响。 增补（2026-10-15，decomp-analyst 字节深化轮；只读，与 §1-11 并行不冲突）

本轮独立补做**九班侧**（`rom/gunnac_cn.nes`）字节深化：将 §3.3/§4 中三项"B 级可销"升 **A 级定谳**，
并新发现**第二解锁条件 $F2**。复现锚：

```bash
python3 -c "print(open('rom/gunnac_cn.nes','rb').read()[0x1470:0x14D1].hex(' '))"  # $9460-$94C1 支路
python3 -c "print(open('rom/gunnac_cn.nes','rb').read()[0x1529:0x1545]))"          # $9519 记录
python3 -c "print(open('rom/gunnac_cn.nes','rb').read()[0x1605:0x1609]))"          # $95F5 表
python3 -c "print(open('rom/gunnac_cn.nes','rb').read()[0x22D0:0x2330].hex(' '))"  # E-R05 区
```

### 12.1 隐藏行值格/记录字节定谳（A 级；销 H-04）

| bank00 指令 | 字节事实 | 升级 |
|---|---|---|
| `$9470 AD F9 FF / 05 F2 / D0 01 / 60` | 隐藏行文本支路门=**$FFF9≠0 或 $F2≠0** | **新发现：第二解锁条件 $F2** |
| `$948C AD F9 FF / D0 01 / 60` | 值格写支路门=**仅 $FFF9**（与文本门不对称） | §3.3 门语义细化 |
| `$947C AD BE 01 / F0 04` → 记录 `$9597`（$01BE==0）或 `$95AB`（≠0），`JSR $E6D9` 写 PPU $2266/$2286 | **DEATH/NON DEATH 行=$01BE 二选一文本记录**（行落点 row19/20，两行制） | D2 第 3 行身份 A 级 |
| `$9492-$94A0`：`$15/$14:=$22CD`；`LDA $01B6 / CLC / ADC #$31 / JSR $E94F` | **$01B6=WEAPON 号，`'1'+值` 写 $22CD**（=行 22 "WEAPON-0" 尾位格） | **H-04 销案（A 级）** |
| `$94B3-$94C1`：`$15/$14:=$230D`；`LDY $01B7 / LDA $95F5,Y / JMP $E94F`；**$95F5=file 0x1605=`46 42 54 57`="F B T W"** | **$01B7=BOMBER 型索引**，表值 F/B/T/W 写 $230D（行 24 尾位格） | E-R08 语义 A 级（炸弹字母表实锤） |
| `$9460-$947E`：`LDA $F1 / F0 04` → 记录 `$95BF/$95D1` 二选一（`JSR $E6C7`） | $00F1=priority 行两值文本选择器 | §3.3 priority 行 A 级 |
| `$9519` 记录=恰 2 行：`WEAPON-0`@PPU $22C6(row22,col6)、`BOMBER-0`@PPU $2306(row24,col6) | 隐藏行标签落点复核（与 §3.3 咬合） | A 级 |

**D2 综合（升级 §3.3）**：隐藏 3 行 = WEAPON-#（$01B6，'1'+值）/ BOMBER-#（$01B7，$95F5[值]）/
DEATH·NON DEATH（$01BE 二记录）；文本门=$FFF9|$F2、值格门=$FFF9。R-D2-1 动态面（现形/可编辑/光标上限）
仍按 §9 待授权实验；**"隐藏选项是什么"至此 A/B 双源定谳**。

### 12.2 E-R05 区字节结构（A 级形态；证伪"截断"说）

file 0x22D2-0x2330 逐记录（含 4 组编号品名 + 价行）：

```
$2285: "1." + <BD BE BF>     $2294: "1 <D8> $10"    ← 1 号（品名 3 tile）
$22C5: "2." + <93 94 95>     $22D4: "1 <D8> $6"     ← 2 号
$2305: "3." + <F9 FA FB>     $2314: "1 <D8> $4"     ← 3 号
$2345: "4." + <DB DC>        （无价目行）→ FF        ← 4 号（品名 2 tile）
```

- **"4 号价目缺失=数据截断"（completeness E-R05 原注/data-tables C 级）证伪**：4 号记录结构完整
  （编号+名+FF），无价目行=**退出项无价**（JP text.csv 行 41-44 `4. 나가기`=EXIT；B 级语义）。
- 品名 tile ID 固定为考证靶点：`BD BE BF`/`93 94 95`/`F9 FA FB`/`DB DC`（S1 CHR 直读路线）；
  `<D8>`=数量/单位 tile 候补（graphics.csv money_unit 族，C 级）。

### 12.3 H7/JIUBAN 水印逐行归属对照面（补 §4 E-R12"不覆盖"）

text.csv credits 43 行（jp_record_offset+x 位）↔ 我方 CN 流 43 条**逐行对齐**即可归属
"JP 原行/US 改行/九班插行"三类（对齐键=行序+x 位，可半自动）。例证（B 级）：
JP `0x18D8 C++ 없인 못 사는 TAKA` ↔ CN `SIIPU.RA.SU. TAKA`；JP `0x1917 TONKIN HOUSE STAFF`
↔ CN `TOKYO SHOSEKI STAFF`；JP 尾部 logo 行（`로고 유지`）↔ CN `@2011 JIUBAN` 水印插行。
→ H7/E-R12 收窄为"尾部 logo 行替换 + 署名行 US 改写"两类；**水印面复证对照面成立**。

### 12.4 对 §9/§11 的增量与门禁

- §9-3 "E-R03/H-04 闭合候选（B 级）"→ **A 级定谳**（12.1）；E-R08 同步 A 级。
- §9 新增两项：**$F2 第二解锁条件**（新开放项）；**E-R05"截断"说证伪**（12.2）。
- §11② 表现面补 H7 归属对照面（12.3）。
- **门禁（本轮实跑）**：`python3 gates/cli.py status` → **PASS=12 FAIL=0 SKIP=0 TOLERATED=0**，
  基线 7 项（rom_files/seed/ledger/assets/knowledge/rust_tests/completeness_cells）全部 OK 不降。
- A 级字节事实按任务口径 dated 注记进 `docs/spec/completeness.md`（E-R03/E-R05/E-R08 三行）；
  N 系吸收记录见 [community-research-gun-nac-ko.md](community-research-gun-nac-ko.md) §7（N-0207/N-0208）。

### 12.5 假设区增补（并入 §10 纪律）

| ID | 假设 | 闭合条件 |
|---|---|---|
| H-D2-2 | $F2=通关/成就解锁标志（"通关一次→隐藏选项永久解锁"） | $F2 全 ROM 写点扫描 + 通关 trace |
| H-D2-3 | $F1/$F2 为相邻 flag 对（priority=$F1、解锁=$F2） | 同上 |
| H-D2-4 | `<D8>`=单位 tile（"엔/個"） | E-R05 品名 CHR 直读同批 |
