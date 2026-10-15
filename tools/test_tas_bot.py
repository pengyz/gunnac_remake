#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/test_tas_bot.py —— tas_bot.py 闭环驱动回归测试（TDD；对拍/归因/锚帧提取纯函数面）。

运行：python3 tools/test_tas_bot.py（自跑）或 pytest tools/test_tas_bot.py。
断言口径：对拍=逐帧全字段全等才 pass；归因=第四候选数据面（boss 框相交×HP 差分）可重导；
锚帧=判定式可机器重导（自然 trigger）。
"""
from __future__ import annotations

import json
import shutil
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
from tools import tas_bot  # noqa: E402

WATCH_KEYS = ["flags26", "flags27", "countdown2f", "gate300", "stage_no", "pstate",
              "boss_hp", "boss_x", "boss_y", "boss_form", "boss_type"]


def _trace(tmp: Path, name: str, rows: list[tuple[int, dict]]) -> None:
    with open(tmp / f"{name}.frames.jsonl", "w", encoding="utf-8") as f:
        for fr, w in rows:
            f.write(json.dumps({"f": fr, "w": w}) + "\n")


def _w(**kw) -> dict:
    base = {k: 0 for k in WATCH_KEYS}
    base.update(kw)
    return base


def _swap_traces(tmp: Path):
    old = tas_bot.TRACES
    tas_bot.TRACES = tmp
    return old


# ---------------------------------------------------------------- 对拍

def test_compare_identical_pass():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        rows = [(f, _w(stage_no=1, boss_hp=14)) for f in range(1, 21)]
        _trace(tmp, "ca", rows)
        _trace(tmp, "cb", list(rows))
        old = _swap_traces(tmp)
        try:
            rep = tas_bot.compare_traces("ca", "cb")
        finally:
            tas_bot.TRACES = old
        assert rep["verdict"] == "pass", rep
        assert rep["mismatch_total"] == 0


def test_compare_detects_field_mismatch():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        rows = [(f, _w(stage_no=1, boss_hp=14)) for f in range(1, 21)]
        bad = [dict(w) for _, w in rows]
        bad[9]["boss_hp"] = 12   # f10 处 HP 差 1
        _trace(tmp, "ca", rows)
        _trace(tmp, "cb", list(zip([f for f, _ in rows], bad)))
        old = _swap_traces(tmp)
        try:
            rep = tas_bot.compare_traces("ca", "cb")
        finally:
            tas_bot.TRACES = old
        assert rep["verdict"] == "fail", rep
        assert rep["mismatch_total"] == 1
        assert rep["mismatch_by_field"].get("boss_hp") == 1
        assert rep["mismatch_sample"][0]["f"] == 10


def test_compare_detects_length_mismatch():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        rows = [(f, _w()) for f in range(1, 21)]
        _trace(tmp, "ca", rows)
        _trace(tmp, "cb", rows[:15])
        old = _swap_traces(tmp)
        try:
            rep = tas_bot.compare_traces("ca", "cb")
        finally:
            tas_bot.TRACES = old
        assert rep["verdict"] == "fail"
        assert not rep["frame_count_equal"]


# ---------------------------------------------------------------- 锚帧提取

def _clear_chain_rows() -> list[tuple[int, dict]]:
    """合成关末链：f2400 门位置位 → f2401 入场 → 倒数 → f2794 尽 → f3066 gate300 →
    f3096 INC $180 → f3116 商店 init。"""
    rows = []
    for f in range(1, 4001):
        w = _w(stage_no=1, pstate=1)
        if f >= 2401:
            w["flags26"] = 0x10
        if f == 2401:
            w["flags27"] = 0x80
        if 2401 <= f < 2794:
            w["countdown2f"] = max(0, 50 - (f - 2401) // 8)
        if f >= 3066:
            w["gate300"] = 0x10
        if f >= 3096:
            w["stage_no"] = 2
        if f >= 3116:
            w["pstate"] = 0
        rows.append((f, w))
    return rows


def test_detect_anchors_clear_chain():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        _trace(tmp, "chain", _clear_chain_rows())
        old = _swap_traces(tmp)
        try:
            rep = tas_bot.detect_anchors("chain")
        finally:
            tas_bot.TRACES = old
        a = rep["anchors"]
        assert a["clear_gate"]["f"] == 2401, a["clear_gate"]
        assert a["stg01-clear"]["f"] == 2401
        assert a["flyout"]["f"] == 2794
        assert a["gate300-open"]["f"] == 3066
        assert a["inc-180"]["f"] == 3096
        assert a["stg02-shop"]["f"] == 3116
        assert a["stg02-exit"]["f"] == 3116 + 284
        assert a["stg02-start"]["f"] == 3116 + 334
        assert a["stg02-combat"]["f"] == 3116 + 784
        for k, v in a.items():
            assert v is not None, k
            assert v["predicate"], k


def test_detect_anchors_menu_window_not_clear():
    """f913-1192 的 bit4 过渡窗不得判为关末门（f>2000 约束）。"""
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        rows = []
        for f in range(1, 1301):
            w = _w()
            if 913 <= f <= 1192:
                w["flags26"] = 0x10
            rows.append((f, w))
        _trace(tmp, "menuwin", rows)
        old = _swap_traces(tmp)
        try:
            rep = tas_bot.detect_anchors("menuwin")
        finally:
            tas_bot.TRACES = old
        assert rep["anchors"]["clear_gate"] is None


# ---------------------------------------------------------------- 逐发归因

def test_attribute_shots_fourth_candidate():
    """自机弹 die 于 boss 框内：HP−2 ⇒ boss_confirmed；0 差分 ⇒ boss_no_damage（第四候选数据）。"""
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        rows = []
        hp = 14
        for f in range(1, 101):
            if f == 29:
                hp = 12          # f28→f29 一发命中（die 帧同帧 HP−2）
            rows.append((f, _w(boss_hp=hp, boss_x=128, boss_y=160, boss_form=0x33, boss_type=0x4A)))
        _trace(tmp, "attr", rows)
        events = [
            {"f": 20, "kind": "shot_spawn", "yslot": 128, "x": 128, "y": 200, "tcode": 0, "owner": 0},
            {"f": 29, "kind": "shot_die", "yslot": 128, "x": 128, "y": 160, "tcode": 0, "owner": 0,
             "boss_x": 128, "boss_y": 160, "boss_hp": 12, "boss_type": 0x4A, "hit_slot": 27, "hit_type": 0x4A},
            {"f": 40, "kind": "shot_spawn", "yslot": 128, "x": 128, "y": 200, "tcode": 0, "owner": 0},
            {"f": 49, "kind": "shot_die", "yslot": 128, "x": 128, "y": 160, "tcode": 0, "owner": 0,
             "boss_x": 128, "boss_y": 160, "boss_hp": 12, "boss_type": 0x4A, "hit_slot": 27, "hit_type": 0x4A},
        ]
        (tmp / "attr.events.jsonl").write_text(
            "\n".join(json.dumps(e) for e in events) + "\n", encoding="utf-8")
        old = _swap_traces(tmp)
        try:
            rep = tas_bot.attribute_shots("attr")
        finally:
            tas_bot.TRACES = old
        assert rep["shots_self_total"] == 2
        assert rep["class_counts"].get("boss_confirmed") == 1, rep["class_counts"]
        assert rep["boss_no_damage"] == 1, rep["class_counts"]


def test_attribute_shots_offscreen_and_object():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        rows = [(f, _w(boss_hp=14, boss_x=128, boss_y=160, boss_form=0x33, boss_type=0x4A))
                for f in range(1, 61)]
        _trace(tmp, "attr2", rows)
        events = [
            {"f": 5, "kind": "shot_spawn", "yslot": 128, "x": 128, "y": 200, "tcode": 0, "owner": 0},
            {"f": 15, "kind": "shot_die", "yslot": 128, "x": 128, "y": 1, "tcode": 0, "owner": 0,
             "boss_x": 128, "boss_y": 160, "boss_hp": 14, "boss_type": 0x4A, "hit_slot": -1, "hit_type": 0},
            {"f": 20, "kind": "shot_spawn", "yslot": 129, "x": 200, "y": 200, "tcode": 1, "owner": 0},
            {"f": 30, "kind": "shot_die", "yslot": 129, "x": 200, "y": 180, "tcode": 1, "owner": 0,
             "boss_x": 128, "boss_y": 160, "boss_hp": 14, "boss_type": 0x4A, "hit_slot": 24, "hit_type": 0x22},
        ]
        (tmp / "attr2.events.jsonl").write_text(
            "\n".join(json.dumps(e) for e in events) + "\n", encoding="utf-8")
        old = _swap_traces(tmp)
        try:
            rep = tas_bot.attribute_shots("attr2")
        finally:
            tas_bot.TRACES = old
        assert rep["class_counts"].get("offscreen") == 1
        assert rep["class_counts"].get("object_slot24") == 1
        assert rep["boss_no_damage"] == 0


# ---------------------------------------------------------------- 回放场景生成

def test_build_replay_writes_hold_only_track():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        scen = tmp / "bot.txt"
        scen.write_text(
            "name botx\nframes 10\nbot stg01-clear from 2\n"
            "watch 0400 pstate\nprobe_mem 05dc 05dc\n", encoding="utf-8")
        (tmp / "botx.inputs.jsonl").write_text(
            '{"f":1,"a":false,"b":false,"start":true,"select":false,"up":false,'
            '"down":false,"left":false,"right":false}\n'
            '{"f":2,"a":false,"b":true,"start":false,"select":false,"up":false,'
            '"down":false,"left":true,"right":false}\n'
            '{"f":3,"a":false,"b":true,"start":false,"select":false,"up":false,'
            '"down":false,"left":true,"right":false}\n', encoding="utf-8")
        old_traces, old_scen = tas_bot.TRACES, tas_bot.SCEN
        tas_bot.TRACES, tas_bot.SCEN = tmp, tmp
        try:
            out = tas_bot.build_replay(scen, "botx", "botx-replay",
                                       {"stg01-clear": {"f": 8, "tag": "clear-entry", "capture": True}})
        finally:
            tas_bot.TRACES, tas_bot.SCEN = old_traces, old_scen
        text = out.read_text(encoding="utf-8")
        assert "hold 1-1 start" in text
        assert "hold 2-3 b" in text and "hold 2-3 left" in text
        assert "watch 0400 pstate" in text
        assert "probe_mem 05dc 05dc" in text
        assert "capture 8 clear-entry" in text
        for bad in ("poke", "press ", "when "):
            assert bad not in text, f"回放轨不得含 {bad!r}"


def test_closure_rejects_non_bot_scenario():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        scen = tmp / "plain.txt"
        scen.write_text("name plain\nframes 5\npress 1 a\n", encoding="utf-8")
        old_scen = tas_bot.SCEN
        tas_bot.SCEN = tmp
        try:
            plan = tas_bot.sr.parse_scenario(scen)
        finally:
            tas_bot.SCEN = old_scen
        assert not plan.get("bot")


ALL_TESTS = [v for k, v in sorted(globals().items()) if k.startswith("test_") and callable(v)]


def main() -> int:
    failed = 0
    for t in ALL_TESTS:
        try:
            t()
            print(f"  PASS  {t.__name__}")
        except AssertionError as e:
            failed += 1
            print(f"  FAIL  {t.__name__}: {e}")
        except Exception as e:  # noqa: BLE001
            failed += 1
            print(f"  ERROR {t.__name__}: {type(e).__name__}: {e}")
    total = len(ALL_TESTS)
    print(f"tas_bot 闭环驱动回归: {total - failed}/{total} 通过")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
