#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/audio_golden.py —— L-Audio 写流 hash golden 生成/对拍（audio-golden.md §3）。

口径：APU 写流 (frame,reg,value) 三元组 → 60f 帧窗分段 → seg_hash=SHA-256(
canonical_encode(segment))；canonical=frame_delta(varint LEB128)|reg|value 逐事件。
空段=hash(空串) 显式记录。容忍 0（保真）；差异段报首差异三元组。

用法：
    python3 tools/audio_golden.py gen <events.jsonl> --scene <id> --out <golden.jsonl>
    python3 tools/audio_golden.py diff <a.jsonl> <b.jsonl>
    python3 tools/audio_golden.py --selftest
"""
import hashlib
import json
import sys


def encode_varint(v, out):
    while True:
        b = v & 0x7F
        v >>= 7
        if v == 0:
            out.append(b)
            break
        out.append(b | 0x80)


def canonical_encode(seg):
    out = bytearray()
    base = seg[0]["frame"] if seg else 0
    for e in seg:
        encode_varint(max(0, e["frame"] - base), out)
        out.append(e["reg"] & 0x1F)
        out.append(e["value"] & 0xFF)
    return bytes(out)


def segments(events):
    if not events:
        return []
    n = max(e["frame"] for e in events) // 60 + 1
    segs = [[] for _ in range(n)]
    for e in events:
        segs[e["frame"] // 60].append(e)
    return segs


def gen(events_path, scene, out_path):
    events = [json.loads(l) for l in open(events_path, encoding="utf-8") if l.strip()]
    events.sort(key=lambda e: (e["frame"],))  # 写序保真=文件序；帧内稳定排序注
    segs = segments(events)
    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        for k, seg in enumerate(segs):
            h = hashlib.sha256(canonical_encode(seg)).hexdigest()
            fh.write(json.dumps({"seg": k, "scene": scene, "hash": h, "events": len(seg)}) + "\n")
    print(f"gen: {len(segs)} 段 → {out_path}")


def diff(a_path, b_path):
    a = {(r["scene"], r["seg"]): r for r in (json.loads(l) for l in open(a_path, encoding="utf-8") if l.strip())}
    b = {(r["scene"], r["seg"]): r for r in (json.loads(l) for l in open(b_path, encoding="utf-8") if l.strip())}
    bad = []
    for k in sorted(set(a) | set(b)):
        ra, rb = a.get(k), b.get(k)
        if ra is None or rb is None:
            bad.append((k, "missing", ra, rb))
        elif ra["hash"] != rb["hash"]:
            bad.append((k, "hash", ra, rb))
    print(f"diff: {len(bad)} 差异段 / {len(set(a) | set(b))} 段")
    for k, why, ra, rb in bad[:10]:
        print(f"  [{k}] {why} events {ra and ra['events']}/{rb and rb['events']}")
    return 1 if bad else 0


def selftest():
    evs = [{"frame": 0, "reg": 0, "value": 0x9F}, {"frame": 0, "reg": 0x15, "value": 0x0F},
           {"frame": 59, "reg": 0x10, "value": 0x80}, {"frame": 60, "reg": 0x11, "value": 0x30}]
    segs = segments(evs)
    assert len(segs) == 2 and len(segs[0]) == 3 and len(segs[1]) == 1
    enc = canonical_encode(segs[0])
    assert enc == bytes([0x00, 0x00, 0x9F, 0x00, 0x15, 0x0F, 0x3B, 0x10, 0x80]), enc.hex()
    h1 = hashlib.sha256(enc).hexdigest()
    h2 = hashlib.sha256(canonical_encode(segs[0])).hexdigest()
    assert h1 == h2, "确定性"
    assert canonical_encode([]) == b"", "空段=空串"
    print("selftest ok")
    return 0


def main():
    if "--selftest" in sys.argv:
        return selftest()
    if sys.argv[1] == "gen":
        ev, sc, out = sys.argv[2], sys.argv[sys.argv.index("--scene") + 1], sys.argv[sys.argv.index("--out") + 1]
        gen(ev, sc, out)
        return 0
    if sys.argv[1] == "diff":
        return diff(sys.argv[2], sys.argv[3])
    print(__doc__)
    return 1


if __name__ == "__main__":
    sys.exit(main())
