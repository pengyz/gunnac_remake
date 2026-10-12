#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/diff_traces.py —— L3 帧级 trace 差分（trace-diff 门禁入口，S2→S5 试点）。

裁定面：game-core::player 帧 tick 序列的逐帧状态向量 == oracle probe trace（Mesen2 执行）。
对齐约定（940 帧 oracle 实证 0 失配）：`state(N) = tick(state(N-1), held(N))`——
输入当帧生效；帧计数 = 场景 DSL 的 inputPolled 序（tools/scenario_capture.lua）。

职责分工（无第二套扫描逻辑）：
- 本脚本：oracle 全量 trace → L3 golden 抽取（--extract）+ 编排；
- game-oracle 的 `l3-diff` bin：模拟执行 + 比对（模拟语义唯一在 game-core::player）。

用法：
    python3 tools/diff_traces.py --extract   # 从 analysis/traces/ 全量 trace 抽 golden（重跑场景用 scenario_run.py）
    python3 tools/diff_traces.py             # 跑 L3 首用例（默认 golden: oracle/traces/gameplay-stage1.player.jsonl）
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ORACLE_TRACE = ROOT / "analysis" / "traces" / "gameplay-stage1.frames.jsonl"
GOLDEN = ROOT / "oracle" / "traces" / "gameplay-stage1.player.jsonl"
WINDOW_START = 1260  # f1260+：实机段（gameplay-stage1 场景约定，见 oracle/scenarios/README.md）
# 死亡/重生窗口（t24 取证，确定性复现 sha256=3eb1273b635758a7）：事件特征=移动冻结+弹药回满+
# $04FC 重置 160 —— 受击判定/重生属 spec §4 未证域（判定点/受击盒未定位），禁止发明 ⇒ 该窗口
# 帧不入 L3 比对面；窗口之外逐帧全比（缺口即分段，l3-diff 重播种）。帧域：f1831-1916、
# f2047-2101、f2131-2175（对应 spec 遗留 W5『up 段后 Y 回中』观测区，现已定性为死亡/重生）。
SKIP_RANGES = [(1831, 1916), (2047, 2101), (2131, 2175)]

FIELDS = {
    "player_x": "x",
    "player_y": "y",
    "player_x_subpix": "sx",
    "player_y_subpix": "sy",
    "controller_held": "held",
}


def extract() -> int:
    if not ORACLE_TRACE.is_file():
        print(f"oracle 全量 trace 缺失：{ORACLE_TRACE}（先跑 tools/scenario_run.py run gameplay-stage1）", file=sys.stderr)
        return 2
    rows = []
    for line in ORACLE_TRACE.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        rec = json.loads(line)
        if rec["f"] < WINDOW_START:
            continue
        if any(lo <= rec["f"] <= hi for lo, hi in SKIP_RANGES):
            continue
        w = rec["w"]
        missing = [k for k in FIELDS if k not in w]
        if missing:
            print(f"f{rec['f']} 缺 watch 字段 {missing}（场景 watch 集已变更？）", file=sys.stderr)
            return 2
        rows.append({"f": rec["f"], **{out: w[src] for src, out in FIELDS.items()}})
    if len(rows) < 2:
        print(f"窗口 f{WINDOW_START}+ 样本不足（{len(rows)}）", file=sys.stderr)
        return 2
    GOLDEN.parent.mkdir(parents=True, exist_ok=True)
    with open(GOLDEN, "w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    print(f"golden 抽取：{len(rows)} 帧（f{rows[0]['f']}..f{rows[-1]['f']}）→ {GOLDEN.relative_to(ROOT)}")
    return 0


def run_diff() -> int:
    if not GOLDEN.is_file():
        print(f"L3 golden 缺失：{GOLDEN}（先 --extract）", file=sys.stderr)
        return 2
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "game-oracle", "--bin", "l3-diff", "--", str(GOLDEN)],
        cwd=ROOT,
    )
    return proc.returncode


def main() -> int:
    if "--extract" in sys.argv:
        rc = extract()
        return rc if rc else run_diff()
    return run_diff()


if __name__ == "__main__":
    sys.exit(main())
