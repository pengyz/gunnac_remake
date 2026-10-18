#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/test_scenario_run.py —— scenario_run 完成判据回归测试（task-110）。

运行：python tools/test_scenario_run.py（自跑）或 pytest tools/test_scenario_run.py。
覆盖四态：bot 早停（done.trace_lines<plan）、overshoot（trace_lines 与 plan 无关）、
legacy done（无 trace_lines 字段 ⇒ 回退 plan 判据）、失败（done=None / timed_out / trace_lines=0）。
"""
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
from tools import scenario_run  # noqa: E402


def _manifest(**kw) -> dict:
    base = {
        "scenario": "oracle/scenarios/x.txt",
        "frames": 40000,
        "trace_lines": 0,
        "done": None,
        "timed_out": False,
    }
    base.update(kw)
    return base


def test_bot_early_stop_done_ok():
    # bot 早停（GAME OVER/clear 后 wants_stop）：trace_lines < plan，DONE.trace_lines 与磁盘一致
    m = _manifest(
        trace_lines=2998,
        done={"frames": 2998, "frames_plan": 40000, "frames_actual": 2998,
              "overshoot": False, "trace_lines": 2998, "captures": 0, "shots": 0,
              "bot_stop": True, "finalize_by": "endFrame", "frame_errors": 0},
    )
    assert scenario_run.done_ok(m) is True
    # 旧判据（trace_lines==plan）会误判；这里显式锁住"bot 早停合法"
    assert m["trace_lines"] != m["frames"]


def test_bot_overshoot_done_ok():
    # 跳帧/分叉：frame 越过 plan ⇒ trace_lines 可 < plan（或 > plan），DONE 记实际帧仍合法
    m = _manifest(
        trace_lines=119994,
        done={"frames": 120000, "frames_plan": 120000, "frames_actual": 120000,
              "overshoot": True, "trace_lines": 119994, "captures": 80, "shots": 80,
              "bot_stop": False, "finalize_by": "endFrame", "frame_errors": 0},
    )
    assert scenario_run.done_ok(m) is True
    # overshoot 且 trace_lines>plan（极端跳帧）同样以 DONE 为准（无帧级异常 ⇒ 合法）
    m2 = _manifest(
        trace_lines=120050,
        done={"frames": 120050, "frames_plan": 120000, "frames_actual": 120050,
              "overshoot": True, "trace_lines": 120050, "captures": 0, "shots": 0,
              "bot_stop": False, "finalize_by": "inputPolled-overshoot", "frame_errors": 0},
    )
    assert scenario_run.done_ok(m2) is True


def test_legacy_done_without_trace_lines_falls_back_to_plan():
    # legacy DONE（capture.lua task-105 之前）无 trace_lines 字段 ⇒ 回退 trace_lines==plan
    legacy = {"frames": 1080, "captures": 37, "shots": 37, "bot_stop": False}
    assert scenario_run.done_ok(_manifest(frames=1080, trace_lines=1080, done=legacy)) is True
    assert scenario_run.done_ok(_manifest(frames=1080, trace_lines=1079, done=legacy)) is False


def test_failure_states_reject():
    # 无 DONE / timed_out / trace_lines=0 / DONE.trace_lines 与磁盘不一致 ⇒ 均 FAIL
    assert scenario_run.done_ok(_manifest(trace_lines=40000, done=None)) is False
    assert scenario_run.done_ok(
        _manifest(trace_lines=40000, timed_out=True, done={"frames": 40000})
    ) is False
    assert scenario_run.done_ok(
        _manifest(trace_lines=0, done={"frames": 0, "trace_lines": 0})
    ) is False
    assert scenario_run.done_ok(
        _manifest(trace_lines=39998, done={"frames": 40000, "trace_lines": 39998})
    ) is True  # 与磁盘一致 ⇒ 完成（即使 < plan）
    assert scenario_run.done_ok(
        _manifest(trace_lines=39998, done={"frames": 40000, "trace_lines": 40000})
    ) is False  # 磁盘与 DONE 不一致 ⇒ 截断/写盘异常


def test_non_bot_plan_equality_still_ok():
    # 非 bot（title-menu 形态）：DONE.trace_lines == plan == 磁盘
    done = {"frames": 1080, "frames_plan": 1080, "frames_actual": 1080,
            "overshoot": False, "trace_lines": 1080, "captures": 37, "shots": 37,
            "bot_stop": False, "finalize_by": "endFrame", "frame_errors": 0}
    assert scenario_run.done_ok(_manifest(frames=1080, trace_lines=1080, done=done)) is True


def test_cmd_run_exit_codes():
    """cmd `run` 退出码（task-110）：合法 DONE（含 bot 早停）⇒ 0；timeout/无 DONE ⇒ 1。"""
    bot_early = _manifest(
        trace_lines=2998,
        done={"frames": 2998, "frames_plan": 40000, "frames_actual": 2998,
              "overshoot": False, "trace_lines": 2998, "captures": 0, "shots": 0,
              "bot_stop": True, "finalize_by": "endFrame", "frame_errors": 0},
    )
    failed = _manifest(trace_lines=100, timed_out=True, done=None)
    orig_run = scenario_run.run_scenario
    orig_argv = sys.argv
    try:
        scenario_run.run_scenario = lambda *a, **k: bot_early  # type: ignore[assignment]
        sys.argv = ["scenario_run.py", "run", "x", "--timeout", "1"]
        assert scenario_run.main() == 0, "合法 DONE（trace_lines<plan）应 exit 0"
        scenario_run.run_scenario = lambda *a, **k: failed  # type: ignore[assignment]
        assert scenario_run.main() == 1, "timed_out/无 DONE 应 exit 1"
    finally:
        scenario_run.run_scenario = orig_run  # type: ignore[assignment]
        sys.argv = orig_argv


def test_cmd_run_all_exit_codes(tmp_root=None):
    """cmd `run-all` 退出码（task-110 同步）：所有场景合法 DONE ⇒ 0；任一失败 ⇒ 1。"""
    import tempfile

    ok_manifest = _manifest(
        trace_lines=1080,
        done={"frames": 1080, "frames_plan": 1080, "frames_actual": 1080,
              "overshoot": False, "trace_lines": 1080, "captures": 37, "shots": 37,
              "bot_stop": False, "finalize_by": "endFrame", "frame_errors": 0},
    )
    bad_manifest = _manifest(trace_lines=0, done=None)
    orig_dir, orig_run, orig_argv = scenario_run.SCENARIO_DIR, scenario_run.run_scenario, sys.argv
    with tempfile.TemporaryDirectory() as td:
        d = Path(td)
        (d / "a.txt").write_text("# dummy\n", encoding="utf-8")
        (d / "b.txt").write_text("# dummy\n", encoding="utf-8")
        try:
            scenario_run.SCENARIO_DIR = d
            scenario_run.run_scenario = lambda *a, **k: ok_manifest  # type: ignore[assignment]
            sys.argv = ["scenario_run.py", "run-all", "--timeout", "1"]
            assert scenario_run.main() == 0, "全部合法 DONE 应 exit 0"
            scenario_run.run_scenario = lambda *a, **k: bad_manifest  # type: ignore[assignment]
            assert scenario_run.main() == 1, "任一失败应 exit 1"
        finally:
            scenario_run.SCENARIO_DIR = orig_dir  # type: ignore[assignment]
            scenario_run.run_scenario = orig_run  # type: ignore[assignment]
            sys.argv = orig_argv


def test_frame_errors_reject_degraded():
    """task-110 F1：done.frame_errors>0（逐帧 pcall 计数异常）⇒ 降级完成，不得 PASS。"""
    base_done = {"frames": 3000, "frames_plan": 3000, "frames_actual": 3000,
                 "overshoot": False, "trace_lines": 3000, "captures": 0, "shots": 0,
                 "bot_stop": False, "finalize_by": "endFrame"}
    clean = dict(base_done, frame_errors=0)
    assert scenario_run.done_ok(_manifest(frames=3000, trace_lines=3000, done=clean)) is True
    dirty = dict(base_done, frame_errors=2)
    assert scenario_run.done_ok(_manifest(frames=3000, trace_lines=3000, done=dirty)) is False
    legacy = dict(base_done)  # 无 frame_errors 字段（task-105 之前）⇒ 视为 0
    assert scenario_run.done_ok(_manifest(frames=3000, trace_lines=3000, done=legacy)) is True


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
    print(f"scenario_run 完成判据回归: {len(ALL_TESTS) - failed}/{len(ALL_TESTS)} 通过")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
