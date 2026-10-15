#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/diff_l5.py —— L5 流程级对拍骨架（StageProbe 序列 ↔ oracle/traces/*.l5.jsonl golden）。

骨架口径（最小可用面，2026-10-15 第三次开工）：单场景逐条对拍，**f/mode/checkpoint 三字段
named 相等即 OK**；golden 有而 probe 无 = 显式 unknown（golden 只升不降、不静默通过）。

用法：
  python3 tools/diff_l5.py --probe <probe5.jsonl> --golden <*.l5.jsonl> [--scene <name>]
  python3 tools/diff_l5.py --selftest
后续扩面（勿在骨架混入）：八关 StageProbe 全字段、trigger 分级、s5 三场景对照。
"""
import json, sys

FIELDS = ("f", "mode", "checkpoint")


def load(path):
    return [json.loads(l) for l in open(path, encoding="utf-8") if l.strip()]


def diff(probe, golden, scene=None):
    idx = {(r.get("scene"), r["f"]): r for r in probe}
    out = []
    for g in golden:
        if scene and g.get("scene") != scene:
            continue
        p = idx.get((g.get("scene"), g["f"]))
        if p is None:
            out.append((g, "unknown", "golden 无 probe 覆盖"))
        elif all(p.get(k) == g.get(k) for k in FIELDS):
            out.append((g, "ok", ""))
        else:
            bad = [k for k in FIELDS if p.get(k) != g.get(k)]
            out.append((g, "mismatch", f"{bad} probe={p}"))
    return out


def main(argv):
    if "--selftest" in argv:
        g = [{"scene": "s5-title", "f": 180, "mode": "Black", "checkpoint": "boot-black"}]
        p = [{"scene": "s5-title", "f": 180, "mode": "Black", "checkpoint": "boot-black"},
             {"scene": "s5-title", "f": 360, "mode": "Title", "checkpoint": "title-in"}]
        r = diff(p, g)
        assert r[0][1] == "ok", r
        r2 = diff(p[:1], [{"scene": "s5-title", "f": 360, "mode": "Title", "checkpoint": "title-in"}])
        assert r2[0][1] == "unknown", r2
        r3 = diff([{"scene": "s5-title", "f": 180, "mode": "Menu", "checkpoint": "boot-black"}], g)
        assert r3[0][1] == "mismatch", r3
        print("selftest: ok（ok/unknown/mismatch 三态）")
        return 0
    get = lambda k: argv[argv.index(k) + 1] if k in argv else None
    probe, golden, scene = get("--probe"), get("--golden"), get("--scene")
    if not (probe and golden):
        print(__doc__)
        return 2
    res = diff(load(probe), load(golden), scene)
    for g, v, note in res:
        print(f"[{v:8}] f={g['f']} {g.get('mode')} {g.get('checkpoint')} {note}")
    bad = sum(1 for _, v, _ in res if v != "ok")
    print(f"L5-DIFF: {len(res) - bad}/{len(res)} 通过（三字段 named 相等；unknown 不计通过）")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
