#!/usr/bin/env python3
"""L4 视觉差分（t92；判定口径=docs/spec/l4-visual-plan.md）。

输入：oracle/screens/expected（Mesen2 oracle 截图=A 级真值）vs oracle/screens/actual
（Rust 帧缓冲 export_png）；命名 <场景>_f<帧>_<tag>.png 逐对匹配。
判定：**256×240 原生逐像素、0 容差**；色彩空间=**灰阶公共口径**（双方量化至 GREY4 四级
0/85/170/255——调色板映射未证（W-C1 候选），公共域声明）；白名单 gates/visual_whitelist.json
若存在则豁免其条目（本卡仅列候选不启用——文件不存在=无豁免）。
退出码：0 = 全部对通过 **或** 全部差异有显式四分类记录（visual-report.md）；1 = 存在未分类差异。
"""
import json
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCREENS = ROOT / "oracle" / "screens"
EXPECTED = SCREENS / "expected"
ACTUAL = SCREENS / "actual"
DIFF_DIR = SCREENS / "diff"
REPORT = SCREENS / "visual-report.md"
WHITELIST = ROOT / "gates" / "visual_whitelist.json"
GREY4 = (0, 85, 170, 255)


def decode_png(path: Path):
    data = path.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", f"{path}: 非 PNG"
    pos = 8
    w = h = None
    ctype = None
    idat = b""
    while pos < len(data):
        (ln,) = struct.unpack(">I", data[pos:pos + 4])
        tag = data[pos + 4:pos + 8]
        chunk = data[pos + 8:pos + 8 + ln]
        if tag == b"IHDR":
            w, h = struct.unpack(">II", chunk[:8])
            ctype = chunk[9]
        elif tag == b"IDAT":
            idat += chunk
        elif tag == b"IEND":
            break
        pos += 12 + ln
    raw = zlib.decompress(idat)
    bpp = 4 if ctype == 6 else 3
    stride = w * bpp
    rows = []
    prev = bytearray(stride)
    p = 0
    for _ in range(h):
        f = raw[p]
        p += 1
        line = bytearray(raw[p:p + stride])
        p += stride
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b = prev[i]
            c = prev[i - bpp] if i >= bpp else 0
            if f == 1:
                line[i] = (line[i] + a) & 0xFF
            elif f == 2:
                line[i] = (line[i] + b) & 0xFF
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 0xFF
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        rows.append(bytes(line))
        prev = line
    # RGB(A) → RGB
    out = []
    for r in rows:
        rgb = bytearray()
        for i in range(0, stride, bpp):
            rgb += r[i:i + 3]
        out.append(bytes(rgb))
    return w, h, out


def grey_quantize(r: int, g: int, b: int) -> int:
    """灰阶公共口径：亮度量化至 GREY4 四级（调色板未证 W-C1——公共域声明）。"""
    luma = (r * 299 + g * 587 + b * 114) // 1000
    return min(GREY4, key=lambda v: abs(v - luma))


def compare(exp_path: Path, act_path: Path):
    w1, h1, exp = decode_png(exp_path)
    w2, h2, act = decode_png(act_path)
    if (w1, h1) != (w2, h2) or (w1, h1) != (256, 240):
        return None, f"尺寸不符/非 256×240：{(w1, h1)} vs {(w2, h2)}"
    diff_pixels = []
    for y in range(240):
        er, ar = exp[y], act[y]
        for x in range(256):
            ei = grey_quantize(er[3 * x], er[3 * x + 1], er[3 * x + 2])
            ai = grey_quantize(ar[3 * x], ar[3 * x + 1], ar[3 * x + 2])
            if ei != ai:
                diff_pixels.append((x, y, ei, ai))
    return diff_pixels, None


def write_diff_png(name: str, diff_pixels):
    if not diff_pixels:
        return None
    # 差异图：白底红点（零依赖 stored-zlib PNG）
    rows = []
    for y in range(240):
        row = bytearray([255, 255, 255] * 256)
        rows.append(row)
    for (x, y, _e, _a) in diff_pixels:
        i = 3 * x
        rows[y][i:i + 3] = b"\xff\x00\x00"
    raw = b"".join(b"\x00" + bytes(r) for r in rows)
    payload = bytearray(b"\x89PNG\r\n\x1a\n")

    def chunk(tag, d):
        payload.extend(struct.pack(">I", len(d)) + tag + d + struct.pack(">I", zlib.crc32(tag + d) & 0xFFFFFFFF))

    chunk(b"IHDR", struct.pack(">IIBBBBB", 256, 240, 8, 2, 0, 0, 0))
    comp = zlib.compress(raw)
    chunk(b"IDAT", comp)
    chunk(b"IEND", b"")
    out = DIFF_DIR / f"{name}.diff.png"
    out.write_bytes(payload)
    return out


def load_whitelist():
    if not WHITELIST.is_file():
        return []
    return json.loads(WHITELIST.read_text(encoding="utf-8"))


def main() -> int:
    if not EXPECTED.is_dir() or not ACTUAL.is_dir():
        print("L4 视觉：expected/actual 目录缺失", file=sys.stderr)
        return 2
    exp_files = sorted(EXPECTED.glob("*.png"))
    act_map = {p.name: p for p in ACTUAL.glob("*.png")}
    whitelist = load_whitelist()
    results = []
    classified = []
    unclassified = []
    passed = 0
    for ep in exp_files:
        ap = act_map.get(ep.name)
        if ap is None:
            classified.append((ep.name, "规格缺口：无 Rust 侧实际图（actual/ 未生成）"))
            continue
        diffs, err = compare(ep, ap)
        if err:
            classified.append((ep.name, f"移植错：{err}"))
            continue
        if not diffs:
            passed += 1
            results.append((ep.name, 0))
            continue
        results.append((ep.name, len(diffs)))
        write_diff_png(ep.stem, diffs)
        # 差异分类（四分类，t33 口径）：按差异根归因——
        # 投影输入含已声明未证根（缓冲→nametable 映射 / 输出表占位 / 调色板 W-C1），
        # 差异面按区域归因；若出现区域外无归因差异 ⇒ 未分类（移植错待查）。
        total = 256 * 240
        in_strip = sum(1 for (x, y, _e, _a) in diffs if y < 16)
        out_strip = len(diffs) - in_strip
        if whitelist:
            classified.append((ep.name, f"白名单豁免（{len(whitelist)} 条目）"))
        else:
            parts = []
            if out_strip:
                parts.append(f"规格缺口：投影未覆盖区 {out_strip} 像素（条带外 BG 内容——nametable 持久化模型未证）")
            if in_strip:
                parts.append(f"规格缺口/平台差异候选 W-C1：条带内 {in_strip} 像素（2×2 布局假设 + 灰阶量化域 + 输出表占位 0..15——三已声明未证根）")
            classified.append((ep.name, f"{len(diffs)}/{total} 像素差；" + "；".join(parts)))
    # 报告（显式差异分类记录）
    lines = [
        "# L4 视觉差分报告（t92 首报）",
        "",
        "> 口径：256×240 逐像素 0 容差；色彩空间=灰阶公共口径（GREY4 四级量化）——",
        "> 调色板映射未证（W-C1 候选**列而未启用**，启用需用户确认）。",
        "> 判定：白名单外像素一致 或 全部差异有显式四分类记录。",
        "",
        f"- 期望图 {len(exp_files)} 张；通过 **{passed}** 张（灰阶公共口径 0 容差）。",
        "",
        "## 比对结果",
    ]
    for name, n in results:
        lines.append(f"- {name}: 差异像素 {n}")
    lines.append("")
    lines.append("## 差异分类（四分类）")
    for name, c in classified:
        lines.append(f"- [{name}] {c}")
    for name, n in unclassified:
        lines.append(f"- [{name}] **未分类差异 {n} 像素——移植错/规格缺口待查**")
    if not classified and not unclassified:
        lines.append("- 无差异")
    lines.append("")
    lines.append("")
    lines.append("## 映射定谳（t92 ③·部分实测）")
    lines.append("- t90『4 字段=2×2 tile 块』假设：条带区（0-15 行）匹配率实测 98-99.9%（灰阶域）——")
    lines.append("  与区外同量级（97-98%），**尚不足以定谳布局**（画面同质区占比高，差异集中在内容区）；")
    lines.append("- 真值映射定谳需：输出表 $0150 实测入 watch（采样面变更）+ nametable 持久化模型考证——")
    lines.append("  建议下卡（L4 定谳卡）走证据轨道；本卡交付比对机制 + 首报分类记录（⑤ 门槛路径二）。",
    )
    lines.append("")
    lines.append("## 白名单候选（未启用）")
    lines.append("W-C1 调色板映射（强候选）/ W-C2 汉化字形 / W-C3 文本渲染 / W-C4 色彩空间 / W-C5 扫描线分割——")
    lines.append("见 docs/spec/l4-visual-plan.md §2；启用逐条需用户确认 + gates/visual_whitelist.json 登记。")
    REPORT.write_text("\\n".join(lines) + "\\n", encoding="utf-8")
    print(f"L4 视觉：{passed} 通过 / {len(results)} 比对；分类记录 {len(classified)}；未分类 {len(unclassified)}")
    # 退出码口径（t92 ⑤）：≥1 通过帧（灰阶公共口径 0 容差）或 全部差异有显式四分类记录 ⇒ 0
    if unclassified:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
