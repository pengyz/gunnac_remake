# 对抗复核 #62：D3 修复效果终验（第三轮轻量·fresh context·证伪立场）

> 被审：commit `2059401`（fix(data): D3 复审 #61 小修批——AR61-1 必修 4 文档 + AR61-2/3/4）。
> 审核方：独立 verifier #62（fresh context subagent，实施≠审核；不采信被审自述、不复用 #61 结论）。
> 纪律：AGENTS.md §5 对抗复核（2026-10-12）——证伪立场独立重导；未贴出的运行结果不存在。
> 范围（本轮为轻量终验）：① 五项 grep 逐项核验 ② `gates/cli.py check` 复跑 ③ 核心字节事实独立直读抽验。
> 数据源（只读）：`rom/gunnac_cn.nes`、`reference/jp/gunnac_jp.nes`、`docs/findings/*.md`、`gates/`（未改动）。

**裁定：pass**——五项核验全部落地（第 5 项经"dated 语境"通道成立，见 §1-⑤ 分项判定），
门禁 **12/12 全绿**（PASS=12 FAIL=0 SKIP=0 TOLERATED=0），
AR61-1/2 的字节级事实由本审独立直读复现（首差 $E054 / 10 处操作数改写 / 块体 $E093–$E09C / JMP 对 $E7CC↔$E7D6）。
附 2 条**非阻断**形态备注（§3，供 captain 裁量，不改变 verdict）。

## 1. 五项核验表（逐 grep）

| # | 判据 | 核验方法（grep/直读） | 实测 | 判定 |
|---|---|---|---|---|
| ①a | "首差=$E092" 全仓无裸露活主张（仅存 ~~删除线~~+AR61-1 dated 更正语境） | 全仓 grep `首差`／`首差=\$E092`／`首差字节`，逐条判定删除线与更正语境 | 4 文档 5 处命中分两类：(a) `~~首差字节 $E092=BNE 操作数~~`（base-attribution L76/L262、w-p1 L233、jp-cn-parity L18、review-52 L58）**全部删除线**且紧随 `【2026-10-14 更正（复核 #61 AR61-1）】`；(b) 更正文自身 `"首差=$E092"驳回：…`（引述被驳主张的更正语境）。review-61 中 5 处系该审 AR61-1 发现记录（"被本审字节直读推翻/不成立/补 dated 更正"），为驳回语境非活主张。**裸露活主张 = 0** | ✅ |
| ①b | "=$E054" 效应段下界应在 4 文档（base-attribution/jp-cn-parity/w-p1/review-52） | grep `效应段下界应为` | 4/4 命中：base-attribution L76/L262、jp-cn-parity L18、w-p1 L233、review-52 L58 均含"效应段下界应为 **$E054** `d0 0c`→`d0 16`" | ✅ |
| ② | AR61-2：base-attribution 含 "JMP $E7CC↔$E7D6"（非 E7CE） | grep `E7CC\|E7CE\|E7D6` + ROM 直读 | base-attribution L77 现为 `` `JMP $E7CC`↔`$E7D6`（Δ=+0x0A 自洽——【2026-10-14 修（#61 AR61-2）】CN 操作数 2B 笔误更正） ``；base-attribution 内 `E7CE` 0 命中。**独立直读**：CN $E6DC/$E91E 两处 `JMP $E7CC`、JP $E6E6/$E928 两处 `JMP $E7D6`，Δ=+0x0A ✓（复核 #61 裁定复现） | ✅ |
| ③ | AR61-3：hidden-option-gate-fff9 "尾区局部" 已 dated 更正（引 AR61-1 口径） | 读 L10 | L10 现为 "（−$0A 位移=JP 独有 10B 块效应——【2026-10-14 更正（复核 #61 AR61-3）】"尾区局部"系 #52 已作废补注（2 采样点过度概括），效应段 **$E054–$F6C0（AR61-1 口径）**；RESET=$E000 同）"——dated 更正 ✓ 显式引 AR61-1 口径 ✓ 且该处效应段下界已直接改 $E054 ✓ | ✅ |
| ④ | AR61-4：game-action-mode-domain "内容驱动时序差" 带改判注 | 读 §5 表（L283） | 现为 "~~内容驱动时序差（L3 ①）~~【2026-10-14 修（复核 #61 AR61-4）】已改判区域/构建差异（base-attribution 裁定② + L3 C1 更正）；语义同为 scene CHR 影子"——旧标签删除线 + 改判注 ✓ | ✅ |
| ⑤ | "效应段 $E092" 字样无裸露（改 $E054 或 dated 语境） | 全仓 grep `效应段`，逐站点判定 | 字样存 5 站点（4 文档）+ 1 处已改 $E054：见下表分项判定。**无一处裸露于无标注正文**（对照 AR61-3/4 型裸文）；4 站点同句/同段带 AR61-1 dated 更正（"效应段下界应为 $E054"）即更正语境，1 站点（base-attribution L280）处 dated 登记条目内（历史登记语境） | ✅（dated 语境通道；形态备注 AD62-1 非阻断） |

### ⑤ 分项站点判定（"效应段 …$E092" 字样逐处）

| 站点 | 字样形态 | dated 语境 | 随文 $E054 更正 | 判 |
|---|---|---|---|---|
| base-attribution L262（C8 单元格） | 未删线"（效应段 $E092–$F6C0，64B 窗…）" | 同格 `【2026-10-14 更正（复核 #61 AR61-1）】` | 有（同句"效应段下界应为 **$E054**"） | dated 语境 ✓ |
| w-p1 L232 | 未删线粗体"实测效应段 **$E092–$F6C0**" | 块引 `【2026-10-14 更正（AR54/复核 #54）】` + L233 AR61-1 更正 | 有（紧随同括号） | dated 语境 ✓ |
| jp-cn-parity L18（布局层格） | 未删线"（效应段 $E092–$F6C0；…）" | 同格 AR61-1 dated 更正 | 有 | dated 语境 ✓ |
| review-52 L58 | 未删线粗体"实测效应段 **$E092–$F6C0**（…" | 同行 AR61-1 dated 更正 | 有（同括号） | dated 语境 ✓ |
| base-attribution L280（跨文档勘误登记条） | 未删线"（根因模型 = JP 独有 10B 块 + 效应段 $E092–$F6C0；历史主张不删）" | 条目冠 `【2026-10-14 增补（AR54-7 跨文档勘误登记）】`（历史登记） | **无**（该条自身未引 AR61-1） | dated 登记语境 ✓（形态最弱，记 AD62-1） |
| hidden-option-gate-fff9 L10 | **已改**"效应段 $E054–$F6C0（AR61-1 口径）" | AR61-3 dated 更正 | 直接改 $E054 | ✓ |

> 判定口径说明：⑤ 的"裸露"对齐复核 #61 对 AR61-3/AR61-4 的定义——**无 dated 标注的正文活文本**。
> 上表 5 站点均处于 `【2026-10-14 …】` dated 更正/登记语境内，其中 4 站点的 dated 更正**直接改写该主张的下界**
> （"效应段下界应为 $E054"），符合 AGENTS.md §3.6"原地 dated 更正、不删除历史主张"的纪律形态；
> 故按判据给定的两条通道（"改 $E054 或 dated 语境"）判为落地。

## 2. 独立重导（证伪抽验，非复用 #61 结论）

1. **bank15 全域首差直读**（`rom/gunnac_cn.nes` vs `reference/jp/gunnac_jp.nes`，bank15=16+15×8192）：
   首差字节 = **$E054**（CN `0c` / JP `16`，`JSR $E60C`→`JSR $E616` 操作数低字节），**同文前缀 84B（$E000–$E053）** ✓
   —— AR61-1 的更正内容成立。
2. **$E000–$E09F 差异位点全列**：`$E054/$E05E/$E063/$E066/$E071/$E074/$E077/$E082/$E085/$E088`
   （10 处前向引用操作数改写，先于 $E092）+ `$E092`（CN `d0 0c`→JP `d0 16`，BNE）+ 块区 ✓ 与更正文逐项吻合。
3. **块体直读**：JP `$E093–$E09C` = `ad be 01 f0 02 a9 00 8d be 01`（`LDA $01BE/BEQ/LDA #$00/STA $01BE`，10B）
   两侧同址后续 `ad 80 01…` 对齐，CN/美版基底缺失 ⇒ **块体精界 $E093–$E09C 成立**（"字节直读校正 +1B"注正当）。
4. **JMP 对直读**：CN `$E6DC/$E91E` = `JMP $E7CC`、JP `$E6E6/$E928` = `JMP $E7D6`，Δ=+0x0A 自洽
   ⇒ AR61-2 把 `$E7CE` 改 `$E7CC` 正确，base-attribution 现文本与 ROM 一致。
5. **门禁复跑**：`python3 gates/cli.py check` → **CHECK: PASS（PASS=12 FAIL=0 SKIP=0 TOLERATED=0）**，
   全部 ratchet OK 无降（rom/seed/assets/naming 206/knowledge 18/docs-links 205 md·102 链接/rust-tests 108/
   subroutine/trace/visual/completeness 19/addr-bridge v2）。

## 3. 非阻断备注（不入阻断项，供裁量）

| ID | 分类 | 级 | 内容 | 建议 |
|---|---|---|---|---|
| AD62-1 | 纪律（形态一致性） | 低·可选 | "效应段 $E092–$F6C0" 字样 5 站点以"历史文本 + 随文 AR61-1 dated 更正（下界应为 $E054）"形态留存（符合 §3.6 不删历史主张），但与 hidden-option-gate-fff9 L10 已直接改写为 "$E054–$F6C0（AR61-1 口径）" 的形态不一致；其中 base-attribution L280 登记摘要句是唯一无随文 $E054 更正的站点，w-p1 L232 / review-52 L58 的"**实测**效应段 $E092–$F6C0"粗体字样保留"实测"口气与更正并置，同句并置可读性欠佳 | 后续 housekeeping 批统一形态：5 处字样改 "$E054–$F6C0（AR61-1 口径）" 或整句加删除线并入更正注；至少给 base-attribution L280 补一句随文 AR61-1 注 |
| AD62-2 | 规格缺口（引证数字残留） | 低·可选 | `adversarial-review-54.md` L102 例句仍为 `JMP $E7CE`↔`$E7D6`（该文为 #54 冻结评审记录；本轮判据②范围=base-attribution，已达标）。若复核 #61 §8-2"同 4 处例句随文更正"意在覆盖此点，则该点未落 | 若清账，给该行补 dated 注（AR61-2 口径）；不阻断 |

## 4. 留痕（复现命令）

```bash
# 五项 grep
grep -rn '首差' --include='*.md' .            # ①a：删除线/更正语境逐条判定
grep -rn '效应段下界应为' --include='*.md' .   # ①b：4 文档命中
grep -rn 'E7CC\|E7CE\|E7D6' --include='*.md' . # ②
grep -rn '效应段' --include='*.md' .           # ⑤：5 站点 + 1 处已改
# 字节直读（①②③④ 的事实底座）
python3 -c "d=open('rom/gunnac_cn.nes','rb').read(); j=open('reference/jp/gunnac_jp.nes','rb').read(); b=16+15*8192; k=0
while d[b+k]==j[b+k]: k+=1
print(hex(0xE000+k), k, d[b+k], j[b+k])"
# 门禁
python3 gates/cli.py check                    # CHECK: PASS（12/12）
```

**verdict = pass**（五项全落地 + 门禁 12/12 + 核心字节事实独立复现；AD62-1/AD62-2 非阻断）。
复核产物仅新增本文件（未改动任何被审文件、门禁、ROM/投影）。
