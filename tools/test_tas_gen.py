#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/test_tas_gen.py —— tas_gen.py 的 fail-closed 回归测试（TDD，复核 #78 F1/A1）。

背景（docs/findings/adversarial-review-78.md §1.4）：
- F1：`when` 条件子句是死语法（零消费者）且 fail-open 静默吞（when 内嵌 poke/snapshot → exit=0 出轨）；
  修复档位 = **显式拒绝未实现指令**（when 子句动作动词白名单外 → PlanError；when 本身 → "未实现"显式拒绝）。
- A1：`--name`/`prefix` 路径消毒（防 `../` 越出 SCENARIO_DIR 读/写）。

运行：python3 tools/test_tas_gen.py（自跑）或 pytest tools/test_tas_gen.py（如环境有 pytest）。
断言口径与复核 #78 §1.2 fail-closed 试调表一致：**exit=1 + 不落盘**。
"""
from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
from tools.tas_gen import PlanError, parse_plan  # noqa: E402

try:  # A1 消毒函数（TDD RED 期尚未实现 → 相关用例 ERROR = 红）
    from tools.tas_gen import safe_name_component  # noqa: E402
except ImportError:  # pragma: no cover
    safe_name_component = None

TAS_GEN = ROOT / "tools" / "tas_gen.py"

BASE = """stage 1
frames 13000
fire 1250-12999
band x 40-200 y 150-215
"""


def _plan(tmp: Path, extra: str, name: str = "p.tas") -> Path:
    p = tmp / name
    p.write_text(BASE + extra + "\n", encoding="utf-8")
    return p


def _cli_gen(plan: Path, out_dir: Path, name: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(TAS_GEN), "gen", str(plan), "-o", str(out_dir), "--name", name],
        capture_output=True, text=True)


# ---------------------------------------------------------------- F1：when fail-closed

def test_when_then_poke_rejected():
    """when 子句内嵌 poke：必须 fail-closed（拒绝），不得静默吞（复核 #78 F1 试调）。"""
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "when frame in [3,4] then poke 0026 10")
        try:
            parse_plan(plan)
        except PlanError as e:
            assert "poke" in str(e), f"错误信息须点名 poke 拒绝，实得: {e}"
            return
        raise AssertionError("when … then poke 未被拒绝（fail-open）")


def test_when_then_snapshot_rejected():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "when frame in [3,4] then snapshot 50")
        try:
            parse_plan(plan)
        except PlanError as e:
            assert "snapshot" in str(e), f"错误信息须点名 snapshot 拒绝，实得: {e}"
            return
        raise AssertionError("when … then snapshot 未被拒绝（fail-open）")


def test_when_then_restore_rejected():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "when frame in [3,4] then restore 50")
        try:
            parse_plan(plan)
        except PlanError as e:
            assert "restore" in str(e), f"错误信息须点名 restore 拒绝，实得: {e}"
            return
        raise AssertionError("when … then restore 未被拒绝（fail-open）")


def test_when_non_whitelist_verb_rejected():
    """when 子句内白名单外动词（未知指令）→ PlanError 显式拒绝。"""
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "when frame in [3,4] then frobnicate x")
        try:
            parse_plan(plan)
        except PlanError as e:
            assert "frobnicate" in str(e), f"错误信息须点名未知动词，实得: {e}"
            return
        raise AssertionError("when 子句未知动词未被拒绝（fail-open）")


def test_when_whitelist_verb_still_rejected_unimplemented():
    """when 本身 = 未实现档位：白名单动作动词（hold/press/avoid）也显式拒绝，不展开。"""
    for clause in ("when frame in [3,4] then hold left 30",
                   "when frame in [3,4] then press a",
                   "when frame in [3,4] then avoid x 40-200 for 30 from 100"):
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            plan = _plan(tmp, clause)
            try:
                parse_plan(plan)
            except PlanError as e:
                assert "when" in str(e).lower() or "未实现" in str(e), \
                    f"错误信息须声明 when 未实现，实得: {e}"
                continue
            raise AssertionError(f"{clause!r} 未被拒绝（when 死语法 fail-open）")


def test_when_malformed_rejected():
    """when 缺 then 子句 = 畸形 → PlanError。"""
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "when frame in [3,4] hold left 30")
        try:
            parse_plan(plan)
        except PlanError:
            return
        raise AssertionError("畸形 when 未被拒绝")


# ---------------------------------------------------------------- 回归：既有 fail-closed 面不回退

def test_toplevel_poke_still_rejected():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "poke 50 0026 10")
        try:
            parse_plan(plan)
        except PlanError as e:
            assert "poke" in str(e)
            return
        raise AssertionError("顶层 poke 未被拒绝")


def test_toplevel_unknown_op_still_rejected():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "POKE 50 0026 10")
        try:
            parse_plan(plan)
        except PlanError as e:
            assert "POKE" in str(e) or "未知" in str(e)
            return
        raise AssertionError("顶层未知指令未被拒绝")


def test_valid_plan_still_parses():
    """既有三计划（无 when）不受影响：回归。"""
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "bomb budget 20\nsweep period 114")
        p = parse_plan(plan)
        assert p["stage"] == 1 and p["frames"] == 13000 and p["fire"] == (1250, 12999)


# ---------------------------------------------------------------- A1：路径消毒

def test_name_path_traversal_rejected():
    if safe_name_component is None:
        raise AssertionError("safe_name_component 未实现（A1 路径消毒缺失）")
    for bad in ("../evil", "a/b", "..\\evil", "", "..", "/etc/passwd"):
        try:
            safe_name_component(bad, "--name")
        except PlanError:
            continue
        raise AssertionError(f"--name {bad!r} 未被消毒拒绝")


def test_prefix_path_traversal_rejected():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "", name="pf.tas")
        plan.write_text(BASE + "prefix ../../etc/passwd\n", encoding="utf-8")
        try:
            parse_plan(plan)
        except PlanError as e:
            assert "prefix" in str(e)
            return
        raise AssertionError("prefix 路径穿越未被拒绝")


def test_name_cli_no_write_on_bad_name():
    """CLI 层：--name 穿越 → exit≠0 且不落盘（断言面全部落在 fresh 沙箱内，防跨 run 污染）。"""
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)          # 全新沙箱根
        out = tmp / "out"      # 输出目录；../evil 会解析到 tmp/evil.txt（沙箱内可断言）
        out.mkdir()
        plan = _plan(tmp, "")
        r = _cli_gen(plan, out, "../evil")
        assert r.returncode != 0, f"穿越 --name 应 exit≠0，实得 {r.returncode}"
        assert not (tmp / "evil.txt").exists(), "穿越 --name 不得写出沙箱根"
        assert not (out / "evil.txt").exists(), "穿越 --name 不得落盘"
        assert not (out / "p.txt").exists(), "拒绝后不得落盘"
        assert list(out.iterdir()) == [], "拒绝后输出目录必须为空"


# ---------------------------------------------------------------- CLI 层：fail-closed 行为（exit=1 + 不落盘）

def test_cli_when_poke_exit1_no_output():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "when frame in [3,4] then poke 0026 10")
        r = _cli_gen(plan, tmp, "out1")
        assert r.returncode == 1, f"when 内嵌 poke 应 exit=1，实得 {r.returncode}: {r.stderr[-300:]}"
        assert not (tmp / "out1.txt").exists(), "拒绝后不得落盘"
        assert not (tmp / "out1.report.json").exists(), "拒绝后不得落报告"


def test_cli_when_hold_exit1_no_output():
    with tempfile.TemporaryDirectory() as d:
        tmp = Path(d)
        plan = _plan(tmp, "when frame in [3,4] then hold left 30")
        r = _cli_gen(plan, tmp, "out2")
        assert r.returncode == 1, f"when（未实现）应 exit=1，实得 {r.returncode}: {r.stderr[-300:]}"
        assert not (tmp / "out2.txt").exists(), "拒绝后不得落盘"


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
    print(f"tas_gen fail-closed 回归: {total - failed}/{total} 通过")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
