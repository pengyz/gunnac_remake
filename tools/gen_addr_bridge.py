#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/gen_addr_bridge.py —— 九班↔JP 坐标桥生成与校验（对抗复核 #49 条件 C）。

背景：九班汉化版是**重布局型补丁**（代码 bank 内搬家 + 指针操作数重写），
社区/JP 坐标不可直套九班坐标（docs/findings/jp-cn-parity.md）。本工具产出
**机器映射表**供交叉引用换算，禁止手工换算（N-0094 同款错误已两次）。

用法：
    python3 tools/gen_addr_bridge.py gen       # 生成 data/reference/addr-bridge-jp-cn.json
    python3 tools/gen_addr_bridge.py verify    # 抽样复验映射（对照双 ROM 字节）

映射分三类：
  relocated —— CN 侧 diff 块内容在 JP 同 bank 唯一命中（代码/数据搬家，记 Δ）
  same-site —— 同址差异（指针/常量操作数重写，地址不变，桥=恒等）
  unmapped  —— 未定（内容级搜索未命中/多命中）——**不猜，留待审计**

证据纪律：每条映射带命中偏移与命中计数；verify 模式独立重导抽样条目。
"""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JP = ROOT / "reference" / "jp" / "gunnac_jp.nes"
CN = ROOT / "rom" / "gunnac_cn.nes"
OUT = ROOT / "data" / "reference" / "addr-bridge-jp-cn.json"

BANKS, BANK_SZ = 16, 0x2000
HEADER = 16
MIN_BLOCK = 24  # 小于此长度的 diff 段按 same-site（操作数）处理


def load() -> tuple[bytes, bytes]:
    return JP.read_bytes(), CN.read_bytes()


def bank_runs(jp: bytes, cn: bytes, bi: int) -> list[tuple[int, int]]:
    s = HEADER + bi * BANK_SZ
    runs, in_run = [], False
    for k in range(BANK_SZ):
        d = jp[s + k] != cn[s + k]
        if d and not in_run:
            st, in_run = k, True
        elif not d and in_run:
            runs.append((st, k))
            in_run = False
    if in_run:
        runs.append((st, BANK_SZ))
    return runs


def window_of(addr: int) -> int:
    """CPU 窗口段（$8000/$A000/$C000/$E000）——bank:addr 标注用。"""
    return addr & 0xE000


def gen() -> int:
    jp, cn = load()
    assert len(jp) == len(cn) == HEADER + 2 * BANKS * BANK_SZ, "ROM 尺寸不符（fail-closed，PRG+CHR）"
    result = {
        "meta": {
            "jp_sha256": hashlib.sha256(jp).hexdigest(),
            "cn_sha256": hashlib.sha256(cn).hexdigest(),
            "min_block": MIN_BLOCK,
            "note": "九班↔JP bank 内块级坐标桥（生成物，勿手改；verify 抽样复验）",
        },
        "banks": {},
    }
    for bi in range(BANKS):
        runs = bank_runs(jp, cn, bi)
        base = HEADER + bi * BANK_SZ
        entries = []
        for st, e in runs:
            length = e - st
            if length < MIN_BLOCK:
                entries.append({
                    "kind": "same-site",
                    "cn": f"{st:04X}-{e:04X}",
                    "delta": 0,
                    "bytes": length,
                })
                continue
            blob = cn[base + st : base + e]
            hits = []
            pos = jp.find(blob)
            while pos != -1:
                hits.append(pos)
                pos = jp.find(blob, pos + 1)
            same_bank_hits = [h for h in hits if HEADER + bi * BANK_SZ <= h < HEADER + (bi + 1) * BANK_SZ]
            if len(same_bank_hits) == 1:
                h = same_bank_hits[0]
                jst = h - base
                entries.append({
                    "kind": "relocated",
                    "cn": f"{st:04X}-{e:04X}",
                    "jp": f"{jst:04X}-{jst + length:04X}",
                    "delta": jst - st,
                    "bytes": length,
                    "hit_count_same_bank": 1,
                })
            elif len(same_bank_hits) > 1:
                entries.append({
                    "kind": "unmapped",
                    "reason": "multi-hit",
                    "cn": f"{st:04X}-{e:04X}",
                    "bytes": length,
                    "hit_count_same_bank": len(same_bank_hits),
                })
            else:
                # 全 PRG 搜索（跨 bank 搬家？）
                if len(hits) == 1:
                    prg = hits[0] - HEADER
                    bb, bo = divmod(prg, BANK_SZ)
                    entries.append({
                        "kind": "relocated-cross-bank",
                        "cn": f"{st:04X}-{e:04X}",
                        "jp_bank": bb,
                        "jp": f"{bo:04X}-{bo + length:04X}",
                        "bytes": length,
                    })
                else:
                    entries.append({
                        "kind": "unmapped",
                        "reason": "content-not-found" if not hits else "multi-hit-cross",
                        "cn": f"{st:04X}-{e:04X}",
                        "bytes": length,
                        "hit_count_total": len(hits),
                    })
        result["banks"][f"bank{bi:02d}"] = entries
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(result, ensure_ascii=False, indent=1),
                   encoding="utf-8", newline="\n")
    # 汇总
    from collections import Counter

    c = Counter(e["kind"] for v in result["banks"].values() for e in v)
    total = sum(c.values())
    print(f"生成 {OUT.relative_to(ROOT)}：{total} 条映射（{dict(c)}）")
    return 0


def verify(sample: int = 20) -> int:
    jp, cn = load()
    data = json.loads(OUT.read_text())
    assert data["meta"]["jp_sha256"] == hashlib.sha256(jp).hexdigest(), "JP sha 不符（fail-closed）"
    assert data["meta"]["cn_sha256"] == hashlib.sha256(cn).hexdigest(), "CN sha 不符（fail-closed）"
    # 抽样确定性（种子固定）：按序取 relocated 类均匀抽样
    items = [
        (bi, e)
        for bi, v in data["banks"].items()
        for e in v
        if e["kind"] in ("relocated", "relocated-cross-bank")
    ]
    if not items:
        print("无 relocated 条目可验")
        return 1
    step = max(1, len(items) // sample)
    picked = items[::step][:sample]
    bad = 0
    for bank, e in picked:
        bi = int(bank[4:])
        st, en = (int(x, 16) for x in e["cn"].split("-"))
        blob = cn[HEADER + bi * BANK_SZ + st : HEADER + bi * BANK_SZ + en]
        if e["kind"] == "relocated":
            jst = int(e["jp"].split("-")[0], 16)
            jblob = jp[HEADER + bi * BANK_SZ + jst : HEADER + bi * BANK_SZ + jst + len(blob)]
        else:
            jst = int(e["jp"].split("-")[0], 16)
            jblob = jp[HEADER + e["jp_bank"] * BANK_SZ + jst : HEADER + e["jp_bank"] * BANK_SZ + jst + len(blob)]
        ok = blob == jblob
        if not ok:
            bad += 1
        print(f"  {bank}:{e['cn']} → {e.get('jp')}  {'✓' if ok else '✗ 不符'}")
    print(f"verify: {len(picked) - bad}/{len(picked)} 通过")
    return 0 if bad == 0 else 1


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "gen"
    if cmd == "gen":
        sys.exit(gen())
    if cmd == "verify":
        sys.exit(verify())
    print("用法: gen_addr_bridge.py [gen|verify]")
    sys.exit(2)
