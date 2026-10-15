#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/replay_stg0304_frames.py —— STG-03/04 段实帧回填一键重放（lead 裁定 b 案：事件帧同域）。

前置条件（port 契约共演落地后方可实放）：l5_s5 双契约已同域化
  ①l5_legality 帧序：实帧行适用单调性、f=0 占位行显式豁免类
  ②s5_chain_run 0 容差对拍：事件帧与 golden 实帧同域
（oracle/traces/README.md 2026-10-16 预备注存档为权威方案；本脚本=机器可执行版。）

重放内容（fail-closed：身份断言任一不符即全弃，不部分写入）：
  s5-chain 前 10 行 f=0→实帧：4600/5314/5570/5604/5844/6400/7116/7370/7404/7644
  （5570=port 统一公式尾起点=出店−34；5844/7644=B 级推断注记，实帧异即钉正）
  anchors 同步 f + trigger（stg02-clear/stg03-clear=injected，余 8 行 placeholder 保持分级未决）

用法：
  python3 tools/replay_stg0304_frames.py --dry-run   # 只验证身份断言+打印计划
  python3 tools/replay_stg0304_frames.py             # 实放（前置未满足勿跑：l5_s5 会红）
"""
import json, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
L5 = ROOT / "oracle/traces/s5-all-clear.l5.jsonl"
ANCH = ROOT / "oracle/traces/s5-all-clear.anchors.jsonl"

PLAN = [  # (f, mode, checkpoint, trigger)
    (4600, "StageClear", "stg02-clear", "injected"),
    (5314, "Shop", "stg03-shop", None),
    (5570, "Transition", None, None),
    (5604, "StageStart", "stg03-start", None),
    (5844, "Combat", "stg03-combat", None),
    (6400, "StageClear", "stg03-clear", "injected"),
    (7116, "Shop", "stg04-shop", None),
    (7370, "Transition", None, None),
    (7404, "StageStart", "stg04-start", None),
    (7644, "Combat", "stg04-combat", None),
]


def apply(path, tag):
    lines = [l for l in path.read_text(encoding="utf-8").splitlines() if l.strip()]
    out, n = [], 0
    for l in lines:
        g = json.loads(l)
        if g.get("scene") == "s5-chain" and n < 10:
            f, mode, cp, trig = PLAN[n]
            assert g.get("f") == 0, (tag, n, g)
            assert g.get("mode") == mode and g.get("checkpoint") == cp, (tag, n, g)
            if tag == "anchors":
                assert g.get("trigger") == "placeholder", (tag, n, g)
            g["f"] = f
            if tag == "anchors" and trig:
                g["trigger"] = trig
            l = json.dumps(g, ensure_ascii=False)
            n += 1
        out.append(l)
    assert n == 10, (tag, n)
    return out


def main(argv):
    for p in (L5, ANCH):
        if not p.exists():
            print(f"缺文件 {p}")
            return 2
    l5 = apply(L5, "l5")
    an = apply(ANCH, "anchors")
    print(f"身份断言全过：l5 10 行 + anchors 10 行（2 injected/8 placeholder）")
    if "--dry-run" in argv:
        print("dry-run：未写入。前置核对：cargo test -p game-oracle --test l5_s5 须已同域转绿后实放。")
        return 0
    L5.write_text("\n".join(l5) + "\n", encoding="utf-8")
    ANCH.write_text("\n".join(an) + "\n", encoding="utf-8")
    print(f"已写入 {L5} / {ANCH}；随即跑 cargo test -p game-oracle --test l5_s5 与 diff_l5 对照贴账。")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
