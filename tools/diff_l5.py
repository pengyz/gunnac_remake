#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/diff_l5.py —— L5 流程级对拍（StageProbe 序列 ↔ oracle/traces/*.l5.jsonl golden）。

骨架（af16930）：单场景 f/mode/checkpoint 三字段 named 相等；golden 无 probe 覆盖=显式 unknown。
扩面（本版）：
  ① 全字段对拍 = **golden 驱动投影**——golden 行携带哪些 named 字段就比哪些
    （tick/flags_26/waves/script_recs/sched_recs/slot_types/live_shots 全支持，八关 probe 面）；
  ② trigger 分级（anchors.jsonl 口径）：injected 锚单列 ok-injected，**不计 natural 通过**；
    trigger 值 ∉ {natural, injected} ⇒ unknown（fail-closed 不发明）；
  ③ s5 三场景对照：--scene 省略 ⇒ 逐场景汇总（golden 全场景覆盖）。
红线：golden 只升不降；unknown/injected 不进通过数；三字段骨架语义不变。

用法：
  python3 tools/diff_l5.py --probe <probe5.jsonl> --golden <*.l5.jsonl> [--scene <name>]
        [--anchors <s5-all-clear.anchors.jsonl>]
  python3 tools/diff_l5.py --selftest
"""
import json, sys

BASE_FIELDS = ("f", "mode", "checkpoint")
PROBE_FIELDS = BASE_FIELDS + ("tick", "flags_26", "waves", "script_recs",
                              "sched_recs", "slot_types", "live_shots")


def load(path):
    return [json.loads(l) for l in open(path, encoding="utf-8") if l.strip()]


def trigger_of(anchors, g):
    for a in anchors or []:
        if a.get("checkpoint") == g.get("checkpoint") and a.get("f") == g.get("f"):
            t = a.get("trigger")
            return t if t in ("natural", "injected") else "unknown"
    return None


def diff(probe, golden, scene=None, anchors=None, scene_map=None):
    smap = scene_map or {}
    idx = {(smap.get(r.get("scene"), r.get("scene")), r["f"]): r for r in probe}
    out = []
    for g in golden:
        if scene and g.get("scene") != scene:
            continue
        p = idx.get((g.get("scene"), g["f"]))
        keys = [k for k in PROBE_FIELDS if k in g]  # golden 驱动投影（scene=谱系不入判据）
        if p is None:
            # 谱系容错查（裁决②）：同 f 内容全等但 scene 归属差 ⇒ ok-provenance-note
            alt = [r for (sc, ff), r in idx.items() if ff == g["f"] and sc != g.get("scene")]
            hit = next((r for r in alt if all(k in r and r.get(k) == g.get(k)
                                              for k in PROBE_FIELDS if k in g)), None)
            if hit is not None:
                out.append((g, "ok-provenance-note",
                            f"内容全等 scene 归属差（probe={hit.get('scene')} golden={g.get('scene')}）"))
                continue
            out.append((g, "unknown", "golden 无 probe 覆盖"))
            continue
        absent = [k for k in keys if k not in p]  # probe 无字段面 ⇒ 不可裁（fail-closed）
        if absent:
            out.append((g, "unknown", f"probe 无字段面: {absent}"))
            continue
        bad = [k for k in keys if p.get(k) != g.get(k)]
        if bad:
            out.append((g, "mismatch", f"{bad}"))
            continue
        t = trigger_of(anchors, g)
        if t == "injected":
            out.append((g, "ok-injected", "trigger=injected 不计 natural 通过"))
        elif t == "unknown":
            out.append((g, "unknown", "trigger 值非法（fail-closed）"))
        else:
            out.append((g, "ok", ""))
    return out


def main(argv):
    if "--selftest" in argv:
        g = [{"scene": "s", "f": 1, "mode": "Black", "checkpoint": "boot-black"},
             {"scene": "s", "f": 2, "mode": "Combat", "checkpoint": "stg01-combat",
              "tick": 33, "slot_types": [1] * 28}]
        p = [{"scene": "s", "f": 1, "mode": "Black", "checkpoint": "boot-black", "tick": 9},
             {"scene": "s", "f": 2, "mode": "Combat", "checkpoint": "stg01-combat",
              "tick": 33, "slot_types": [1] * 28}]
        assert diff(p, g)[0][1] == "ok", "①投影：golden 未携带 tick 不比 tick"
        assert diff(p, [dict(g[1], tick=34)])[0][1] == "mismatch", "①全字段：tick 必比"
        assert diff([{"scene": "s", "f": 9}], [{"scene": "s", "f": 9, "mode": "Black", "checkpoint": "c"}])[0][1] == "unknown", "⑤probe 缺字段面=unknown 非 mismatch"
        inj = [{"checkpoint": "stg01-combat", "f": 2, "trigger": "injected"}]
        assert diff(p, [g[1]], anchors=inj)[0][1] == "ok-injected", "②injected 单列"
        bad_t = [{"checkpoint": "boot-black", "f": 1, "trigger": "???"}]
        assert diff(p, [g[0]], anchors=bad_t)[0][1] == "unknown", "②非法 trigger fail-closed"
        r = diff(p, g)
        assert [x[1] for x in r] == ["ok", "ok"], r
        assert diff([{"scene": "stg02", "f": 1, "mode": "StageStart", "checkpoint": "stg02-start"}],
                    [{"scene": "stg02-08-chain", "f": 1, "mode": "StageStart", "checkpoint": "stg02-start"}],
                    scene_map={"stg02": "stg02-08-chain"})[0][1] == "ok", "④场景映射"
        rpn = diff([{"scene": "gameplay-stage1", "f": 180, "mode": "Black", "checkpoint": "boot-black"}],
                   [{"scene": "title-menu", "f": 180, "mode": "Black", "checkpoint": "boot-black"}])
        assert rpn[0][1] == "ok-provenance-note", rpn
        print("selftest: ok（投影/全字段/trigger 分级/三场景汇总/场景映射/谱系注记）")
        return 0
    get = lambda k: argv[argv.index(k) + 1] if k in argv else None
    probe, golden = get("--probe"), get("--golden")
    if not (probe and golden):
        print(__doc__)
        return 2
    smap = {}
    if get("--scene-map"):
        for pair in get("--scene-map").split(","):
            a, _, b = pair.partition("=")
            smap[a] = b
    res = diff(load(probe), load(golden), get("--scene"),
               load(get("--anchors")) if get("--anchors") else None, smap)
    by_scene = {}
    for g, v, note in res:
        print(f"[{v:12}] {g.get('scene')} f={g['f']} {g.get('mode')} {g.get('checkpoint')} {note}")
        by_scene.setdefault(g.get("scene"), []).append(v)
    for sc, vs in sorted(by_scene.items()):
        ok = sum(1 for v in vs if v == "ok")
        print(f"L5-DIFF[{sc}]: {ok}/{len(vs)} natural 通过（unknown/ok-injected 不计）")
    notes = sum(1 for _, v, _ in res if v == "ok-provenance-note")
    if notes:
        print(f"L5-DIFF 谱系注记: {notes} 行（内容 match+scene 归属差，公开不静默；不计 natural）")
    bad = sum(1 for _, v, _ in res if v not in ("ok", "ok-provenance-note"))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
