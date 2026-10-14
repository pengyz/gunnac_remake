# 对抗复审 #12（t176）：t171 分层双式定谳（ASCII 恒等）复审

> 被审方：t171（7ee8f55，`tile-index-transform.md`）。审核方：verifier（t176），CHR 逐码独立解码 + oracle 截图字形三方闭环。
> **裁定：pass（链收口）**——①分层双式成立（文本层恒等+//64 **决定性验证**：ASCII 全表逐码字形成形）；
> ②25/25 全中 ✓ 且扩展全中；三方闭环坐实（f600 credits 实屏字形 ↔ CHR 位图 ↔ nametable 码）；③R12-2 同源解决
> 大体成立（与 t173 R13 裁定一致）；④自查留痕 ✓。余 2 低危观察。

## 0. 观察（不阻塞）

| ID | 级 | 问题 | 处理 |
|---|---|---|---|
| R15-1 | 低 | ③分层归因未覆盖 **t169 font 列同层对照**（pair vs linear 均为 BG 路径）的 +0.26%——t173 逐格证据（171px 稀疏/判别格 38:37 近平）表明该差为**噪声级巧合**，归因"pair 实测（精灵）"不精确 | ~~注记一句~~ **【2026-10-12 t202 已清】噪声注已挂**：font 列同层 +0.26%=**噪声级巧合**（t173：171px 稀疏/38:37 近平），归因表述以分层双式为准 |
| R15-2 | 低 | ④自查 assert 口径（声称"→ 1/1"，实测 grep -c 合计 4 行——ledger 含引用行；『分层双式』5 处 ≥1 ✓） | ~~口径微调~~ **【2026-10-12 t202 已清】口径注**：自查 assert 口径=**目标行命中 ≥1**（多行含引用行属正常，不以合计行数报 1/1）——grep 自查纪律（pattern-repair-selfcheck-grep）口径补 |

## 1. ①分层双式复核 —— ✓ 决定性

**文本层（tile 码=ASCII + bank=124+code//64）**：本审 CHR 逐码独立解码 **A-Z/0-9/标点 33-47 全部字形成形**：

- A B C D E F G H I J K L M N O P Q R S T U V W X Y **25/25 全中** ✓（Z 亦中=26）；
- 0-9 数字 **10/10 全中** ✓；标点 `! " # $ % & ' ( ) * + , - . /` **15/15 全中** ✓；
- 范围外 58-62（`:` `;` `<` `=` `>`）位图非对应 ASCII 形（font 该段另有内容）——超出声称范围。
  > 【2026-10-12 t202 W 注（t176 范围外注）】码 **58-62 超范围留 W**：font 该段另有内容（非对应
  > ASCII 形），不在分层双式声称域内——不修不猜，随 font 结构考证批次再议。

**精灵层（8×16 对，bank106/107 bit0 分半）**：t167 双侧字节表 ✓（t170 已逐字节复现）。
**三模型各测一层的解释**：A（跨半）/B（邻接）= 精灵层对照 ✓、精确式=文本层 ✓——A/B↔公式的张力由分层归因解决 ✓
（font 列噪声残余见 R15-1）。

## 2. ②ASCII 25/25 + 三方闭环 —— ✓ 坐实

三方对照（oracle 截图字形 × nametable 码 × CHR 位图）：

| 方 | 本审实证 |
|---|---|
| oracle 截图字形 | `title-menu_f00600` 裁图：**"1990 ©TOKYO SHOSEKI / PROGRAMED BY COMPILE / PRESENTED BY NEXOFT CORP."** 可读 ASCII ✓ |
| nametable 码 | 写流码=ASCII 码（74=J/83=S/72=H，t170 JJJS ✓） |
| CHR 位图 | bank124+code//64 逐码字形（本审 25/25+扩展全表） |

**闭环成立**：码=ASCII → 位图=字母形 → 截图=可读文本 ✓✓（字形风格与位图逐字对应）。
§4 诚实注（JJJS 行与 f240 未直接对上=他屏内容候选）✓ 纪律良好。

## 3. ③R12-2 张力同源解决核 —— ✓ 大体成立（R15-1 精度注）

t171"分层错位=根因"与本审 t173 R13 裁定（口径混用）**同向**：公式=文本层（现被 ASCII 全表证实）、
pair=精灵层；t179 已采纳 R13 入档 ✓。残余精度问题见 R15-1（font 列同层差的噪声归因）。

## 4. ④自查 grep 留痕核 —— ✓

N-0156/N-0157 落账 ✓（ledger 命中 4 行含引用）、『分层双式』5 处 ✓；assert 口径微调（R15-2）。

## 5. 复现命令

```bash
python3 - <<'PY'   # ②ASCII 全表逐码解码
rom=open('rom/gunnac_cn.nes','rb').read(); C=0x20010
for code in range(33,91):
    bank,off=124+code//64, code%64
    f=rom[C+bank*0x400+off*16:C+bank*0x400+off*16+16]
    art=[''.join('#' if (f[r]>>(7-c))&1 or (f[r+8]>>(7-c))&1 else '.' for c in range(8)) for r in range(8)]
    print(chr(code), *art, sep='\n')
PY
# ②三方：title-menu_f00600_probe.png 裁 y120-200 看 credits 行
# ④ grep -c "分层双式" docs/findings/tile-index-transform.md
```
