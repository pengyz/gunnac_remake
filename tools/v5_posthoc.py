#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/v5_posthoc.py —— v5 全程跑后置窗分析（收账预备件，2026-10-16 激活令①）。

四观测面一体（v5 trace 到手即跑：python3 tools/v5_posthoc.py [run_name]）：
  ① 关号段锚链提取（phase/shop_init/stage_advance/clear_gate/ending 事件链+逐关段锚四连）
  ② OBS-1 出店段检测（stg08 shop→Transition→stg08-start 通/截断/分叉三分支判定）
  ③ 隐藏三行观测（$01B6 WEAPON/$01B7 BOMBER/$01BE 不死——watch 面 death/weapon/bomb_flag 轨迹）
  ④ S-CH-4（$32 旗）顺带观测（watch 缺 $32 ⇒ probe RAM dump 稀疏采样，不足显式注）

判据（lead 令）：八关段锚四连推进+出店段 OBS-1+隐藏三行效果观测；输出=贴账数字面。
"""
import json, sys, glob
from pathlib import Path

TRACES = Path(__file__).resolve().parent.parent / "analysis/traces"


def load(name, suffix):
    p = TRACES / f"{name}.{suffix}"
    if not p.exists():
        return []
    return [json.loads(l) for l in p.read_text().splitlines() if l.strip()]


def anchor_chain(evs):
    kinds = ("stage_clear_gate", "shop_init", "stage_advance", "ending_window", "phase")
    return [(e["f"], e["kind"], {k: v for k, v in e.items() if k not in ("kind", "f")})
            for e in evs if e.get("kind") in kinds]


def obs1_check(evs, frames):
    """OBS-1 三分支：stg08 出店段（shop_init(stage=9 前最后一次)→Transition→start 推进）。"""
    shops = [e for e in evs if e.get("kind") == "shop_init"]
    adv = [e for e in evs if e.get("kind") == "stage_advance"]
    clear8 = [e for e in evs if e.get("kind") == "stage_clear_gate"]
    stages = [r for r in frames if r.get("w", {}).get("stage_no") == 9]
    if any(e.get("stages_cleared") == 8 for e in shops):
        return {"verdict": "pass（stg08 出店达 stg09/结局域）", "shops": len(shops), "advances": len(adv)}
    if len(clear8) >= 8 and not stages:
        return {"verdict": "截断（8 清关门后未及出店段）", "clear_gates": len(clear8)}
    return {"verdict": f"未及（清关门 {len(clear8)}/8——stg08 出店段未达）",
            "clear_gates": len(clear8), "shops": len(shops)}


def hidden_rows(frames):
    """隐藏三行轨迹（首值/末值/变迁数）。"""
    out = {}
    for key, name in (("death_flag", "$01BE 不死"), ("weapon_flag", "$01B6 WEAPON"), ("bomb_flag", "$01B7 BOMBER")):
        vals = [(r["f"], r["w"].get(key)) for r in frames if key in r.get("w", {})]
        if vals:
            changes = sum(1 for a, b in zip(vals, vals[1:]) if a[1] != b[1])
            out[name] = {"first": vals[0], "last": vals[-1], "changes": changes, "samples": len(vals)}
    return out


def sch4_probe(name):
    """S-CH-4（$32 旗）：watch 缺 ⇒ probe RAM dump 稀疏采样（$32 在全 RAM dump 内）。"""
    hexes = sorted(glob.glob(str(TRACES / f"{name}*_probe.ram.hex")))
    out = []
    for h in hexes[:12]:
        try:
            data = bytes.fromhex(Path(h).read_text().split())
            if len(data) > 0x32:
                out.append((Path(h).name, data[0x32]))
        except Exception:
            pass
    return {"samples": out, "note": "watch 无 $32——probe 稀疏采样；充足观测需下批 watch 行加 0032"}


def main(argv):
    name = argv[1] if len(argv) > 1 else "tas-clear-full-v5"
    evs, frames = load(name, "events.jsonl"), load(name, "frames.jsonl")
    if not evs or not frames:
        print(f"缺产物 {name}.events/frames.jsonl")
        return 2
    print(f"== {name} 后置窗分析（events={len(evs)} frames={len(frames)}）==")
    print("\n① 关号段锚链:")
    for f, k, extra in anchor_chain(evs):
        print(f"  f{f:>7} {k} {extra}")
    print("\n② OBS-1 出店段:", json.dumps(obs1_check(evs, frames), ensure_ascii=False))
    print("\n③ 隐藏三行:", json.dumps(hidden_rows(frames), ensure_ascii=False, indent=1))
    print("\n④ S-CH-4 $32:", json.dumps(sch4_probe(name), ensure_ascii=False))
    last = frames[-1]
    print(f"\n末帧 f{last['f']} stage_no={last.get('w', {}).get('stage_no')}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
