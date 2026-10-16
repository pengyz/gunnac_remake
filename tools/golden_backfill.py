#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/golden_backfill.py —— golden 回填接口（v5 收账预备面·秒收工具）。

口径（lead ①预备面令）：锚台账直出同步路径工具化——台账（anchors.jsonl）golden
条目 f 值 → 同步 golden（*.l5.jsonl）占位行 f=0 → 三面帧值全等自检（台账/golden）。
fail-closed：identity 断言（scene/mode/checkpoint 全等才允许值同步）；差异行报出
不写；golden 只升不降（占位→实帧单向）。

用法：
    python3 tools/golden_backfill.py --dry-run   # 只报差异不写
    python3 tools/golden_backfill.py             # 同步+自检
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GOLDENS = {
    "s5-all-clear": ROOT / "oracle/traces/s5-all-clear.l5.jsonl",
    "stg01-chain": ROOT / "oracle/traces/stg01-chain.l5.jsonl",
}
ANCHORS = ROOT / "oracle/traces/s5-all-clear.anchors.jsonl"


def load(p):
    return [json.loads(l) for l in open(p, encoding="utf-8") if l.strip()]


def selftest():
    """秒收链逻辑自检（合成数据——不触真台账/golden）。"""
    anch = [
        {"golden": True, "scene": "s5-chain", "checkpoint": "stg08-start", "mode": "StageStart", "f": 21000},
        {"golden": True, "scene": "s5-chain", "checkpoint": "stg08-clear", "mode": "StageClear", "f": 25000},
    ]
    gold = [
        {"scene": "s5-chain", "mode": "StageStart", "checkpoint": "stg08-start", "f": 0},
        {"scene": "s5-chain", "mode": "StageClear", "checkpoint": "stg08-clear", "f": 0},
        {"scene": "s5-chain", "mode": "Combat", "checkpoint": "stg08-combat", "f": 0},  # 无锚
    ]
    ai = {(a["scene"], a.get("checkpoint"), a["mode"]): a["f"] for a in anch}
    filled = 0
    for g in gold:
        if g["f"] != 0:
            continue
        key = (g["scene"], g.get("checkpoint"), g["mode"])
        f = ai.get(key)
        if f is None or f == 0:
            continue
        match = [a for a in anch if (a["scene"], a.get("checkpoint"), a["mode"]) == key and a["f"] == f]
        if len(match) != 1:
            continue
        g["f"] = f
        filled += 1
    assert filled == 2, f"回填 2 行（有锚）: {filled}"
    assert gold[0]["f"] == 21000 and gold[1]["f"] == 25000
    assert gold[2]["f"] == 0, "无锚行保持占位（fail-closed）"
    print("selftest ok: 2 回填 + 1 无锚保持占位")
    return 0


def backfill_cross_scene(gold_path, anch_rows, dry):
    """跨场景同域回填（lead 域裁定：7 占位=注入域——stgXX-timeline §7 帧值共享链
    前缀帧域；identity 断言=checkpoint+mode 精确（scene 不同源但同域共享帧值））。"""
    gold = load(gold_path)
    # 锚索引：(checkpoint, mode) → f（取同域实锚 f>0）
    ai = {}
    for a in anch_rows:
        if a.get("golden") and a.get("f", 0) > 0:
            ai[(a.get("checkpoint"), a["mode"])] = a["f"]
    changed = 0
    for g in gold:
        if g["f"] != 0:
            continue
        key = (g.get("checkpoint"), g["mode"])
        f = ai.get(key)
        if f is None:
            print(f"[cross] 占位无同域锚: {key}")
            continue
        print(f"[cross] 回填: {key} f=0 -> {f}（注入域共享帧值）")
        g["f"] = f
        changed += 1
    if not dry and changed:
        with open(gold_path, "w", encoding="utf-8") as fh:
            fh.write("\n".join(json.dumps(r, ensure_ascii=False) for r in gold) + "\n")
    return changed


def main():
    if "--selftest" in sys.argv:
        return selftest()
    if "--cross-scene" in sys.argv:
        dry = "--dry-run" in sys.argv
        anch = load(ANCHORS)
        n = backfill_cross_scene(GOLDENS["stg01-chain"], anch, dry)
        print(f"跨场景回填 {n} 行{'（dry-run 未写）' if dry else ''}")
        return 0
    dry = "--dry-run" in sys.argv
    anch = [a for a in load(ANCHORS) if a.get("golden")]
    ai = {(a["scene"], a.get("checkpoint"), a["mode"]): a["f"] for a in anch}
    total = 0
    for name, gp in GOLDENS.items():
        gold = load(gp)
        ga = [a for a in anch if any(g["scene"] == a["scene"] for g in gold)]
        changed = 0
        for g in gold:
            if g["f"] != 0:
                continue
            key = (g["scene"], g.get("checkpoint"), g["mode"])
            f = ai.get(key)
            if f is None or f == 0:
                print(f"[{name}] 占位无台账锚: {key}")
                continue
            # identity 断言（fail-closed）
            match = [a for a in anch if (a["scene"], a.get("checkpoint"), a["mode"]) == key and a["f"] == f]
            if len(match) != 1:
                print(f"[{name}] identity 断言失败（{len(match)} 匹配）: {key} —— 不写")
                continue
            print(f"[{name}] 回填: {key} f=0 -> {f}")
            g["f"] = f
            changed += 1
        total += changed
        if not dry and changed:
            with open(gp, "w", encoding="utf-8") as fh:
                fh.write("\n".join(json.dumps(r, ensure_ascii=False) for r in gold) + "\n")
        # 三面自检
        g_real = sum(1 for g in gold if g["f"] > 0)
        print(f"[{name}] 真锚 {g_real}/{len(gold)}（本次回填 {changed}）")
    print(f"总计回填 {total} 行{'（dry-run 未写）' if dry else ''}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
