#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/test_l4_render_all.py —— task-155 manifest/去重 registry 工具回归测试。

运行：python tools/test_l4_render_all.py（自跑）或 pytest tools/test_l4_render_all.py。
覆盖：22 行 schema/枚举/blocked 空语义、mapping duplicate_of 组、B2 去重等式与
documented_duplicates、registry_ok 异常、strict 行解析、render 日志解析、--validate 往返。
集成（存在 target/p0-0/render 时）：对现有 g_run1/g_run2 只读生成并断言 summary（不写文件）。
"""
from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
from tools import l4_render_all as L  # noqa: E402


def _fake_manifest() -> dict:
    """按 PAIR_SPEC 构造一份合法 22 行 manifest（sha 组与 task-166 安装后裁定表一致）。"""
    exp_dup = {
        "l4_stg06-late": "E_FOUR", "l4_stg08-combat": "E_FOUR",
        "l4_stg08-late": "E_FOUR", "l4_stg08-start": "E_FOUR",
        "l4_ending-4": "E_ENDING", "l4_ending-5": "E_ENDING",
    }
    act_dup = {
        "l4_stg06-late": "A_FOUR", "l4_stg08-combat": "A_FOUR",
        "l4_stg08-late": "A_FOUR", "l4_stg08-start": "A_FOUR",
        "l4_ending-4": "A_ENDING", "l4_ending-5": "A_ENDING",
    }
    pairs = []
    for spec in L.PAIR_SPEC:
        name = spec["name"]
        blocked = spec["status"] == L.STATUS_BLOCKED
        pairs.append({
            "name": name, "scene": spec["scene"], "frame": spec["frame"],
            "capture_tag": spec["capture_tag"], "frame_offset": 0,
            "trigger": spec["trigger"],
            "expected_source": (f"{spec['scene']}:{spec['frame']}:{spec['capture_tag']}"
                                if spec["scene"] else ""),
            "expected_sha256": "" if blocked else exp_dup.get(name, f"E_{name}"),
            "actual_sha256": "" if blocked else act_dup.get(name, f"A_{name}"),
            "actual_run1_sha256": "" if blocked else act_dup.get(name, f"A_{name}"),
            "actual_run2_sha256": "" if blocked else act_dup.get(name, f"A_{name}"),
            "input_sha256": {"frames": f"F_{name}", "probe": "", "text": ""},
            "render_env": {}, "render_argv": ["x"] if not blocked else [],
            "status": spec["status"], "reuse": spec["reuse"],
            "duplicate_of": spec["duplicate_of"],
            "deterministic": not blocked,
        })
    dedupe, _ = L.build_dedupe(pairs)
    return {"summary": {"pairs_total": 22}, "pairs": pairs, "dedupe": dedupe}


def test_spec_22_rows_schema_and_counts():
    assert L.spec_issues() == [], f"内嵌裁定表自检失败: {L.spec_issues()}"
    assert len(L.PAIR_SPEC) == 22
    names = [p["name"] for p in L.PAIR_SPEC]
    assert len(set(names)) == 22
    counts = {s: sum(1 for p in L.PAIR_SPEC if p["status"] == s) for s in
              (L.STATUS_OK, L.STATUS_BLOCKED, L.STATUS_REPLACED, L.STATUS_DUPLICATE)}
    assert counts[L.STATUS_OK] == 13, counts
    assert counts[L.STATUS_BLOCKED] == 4, counts
    assert counts[L.STATUS_REPLACED] == 0, counts           # task-166：5 replaced 全部登记完成
    assert counts[L.STATUS_DUPLICATE] == 5, counts          # ending-4/5 + 四联非主帧 3
    # task-166 #14 Option B：pair 重指向 gs5-anchor-render:3450
    p14 = next(p for p in L.PAIR_SPEC if p["name"] == "l4_stg02-start")
    assert p14["scene"] == "gs5-anchor-render" and p14["frame"] == 3450
    assert p14["capture_tag"] == "stg02-start" and p14["trigger"] == "injected"
    assert p14["status"] == L.STATUS_OK and p14["reuse"] == "independent"
    assert p14["duplicate_of"] == ""
    # task-165 step0 已重渲 g_run1/g_run2 的 #14 actual ⇒ v2 旧 run1 sha 仅历史对照（不作否决）
    assert p14.get("re_render") is True
    # task-166 四联/#17/#18：trigger natural→injected 更正
    for n in ("l4_stg06-late", "l4_stg08-combat", "l4_stg08-late", "l4_stg08-start",
              "l4_stg07-combat", "l4_stg07-start"):
        assert next(p for p in L.PAIR_SPEC if p["name"] == n)["trigger"] == "injected"
    # 4 blocked：expected 无源 ⇒ manifest expected_sha256 必须空（build_manifest 里实现，这里锁裁定表）
    assert all(p["scene"] == "" for p in L.PAIR_SPEC if p["status"] == L.STATUS_BLOCKED)
    # ending-4/5 tag 更正为 probe（C1 实际 shot tag）
    assert next(p for p in L.PAIR_SPEC if p["name"] == "l4_ending-4")["capture_tag"] == "probe"
    assert next(p for p in L.PAIR_SPEC if p["name"] == "l4_ending-5")["capture_tag"] == "probe"


def test_dedupe_equations_and_documented_groups():
    m = _fake_manifest()
    d, ok = L.build_dedupe(m["pairs"])
    assert ok, d
    assert d["expected_present"] == 18 and d["actual_present"] == 18
    assert d["expected_unique"] == 14, d["equations"]     # 18 − 4（四联3+ending1；#14 已拆开）
    assert d["actual_unique"] == 14, d["equations"]
    assert d["expected_reduction"] == 4 and d["actual_reduction"] == 4
    assert d["registry_ok"] is True
    groups = [frozenset(x["names"]) for x in d["documented_duplicates"]]
    assert len(groups) == 2, groups
    assert frozenset({"l4_stg06-late", "l4_stg08-combat",
                      "l4_stg08-late", "l4_stg08-start"}) in groups
    assert frozenset({"l4_ending-4", "l4_ending-5"}) in groups
    # #14 expected 不再与 title 同 blob ⇒ title↔stg02 组消失
    assert not any("l4_stg02-start" in g for g in groups)
    sides = {frozenset(x["names"]): x["side"] for x in d["documented_duplicates"]}
    assert sides[frozenset({"l4_ending-4", "l4_ending-5"})] == "expected+actual"


def test_registry_ok_rejects_unmapped_duplicates():
    m = _fake_manifest()
    for p in m["pairs"]:
        if p["name"] in ("l4_stg08-combat", "l4_stg08-late", "l4_stg08-start"):
            p["duplicate_of"] = ""  # 四联孩子失去 mapping 组 ⇒ registry_ok 必须 False
    d, ok = L.build_dedupe(m["pairs"])
    assert d["registry_ok"] is False and ok is False
    issues = L.validate_manifest(m)
    assert any("dedupe 校验失败" in x for x in issues), issues


def test_validate_manifest_accepts_good_and_rejects_schema_breaks():
    m = _fake_manifest()
    assert L.validate_manifest(m) == []
    # blocked 带 expected sha ⇒ 拒绝
    bad = _fake_manifest()
    p = next(x for x in bad["pairs"] if x["status"] == L.STATUS_BLOCKED)
    p["expected_sha256"] = "DEADBEEF"
    assert any("blocked" in x for x in L.validate_manifest(bad))
    # 非 blocked 空 expected ⇒ 拒绝
    bad = _fake_manifest()
    p = next(x for x in bad["pairs"] if x["status"] == L.STATUS_OK)
    p["expected_sha256"] = ""
    assert any("expected_sha256 空" in x for x in L.validate_manifest(bad))
    # render_env 污染 ⇒ 拒绝
    bad = _fake_manifest()
    p = next(x for x in bad["pairs"] if x["status"] == L.STATUS_OK)
    p["render_env"] = {"TBLDRV": "1"}
    assert any("canonical" in x for x in L.validate_manifest(bad))
    # 行数 ≠22 ⇒ 拒绝
    bad = _fake_manifest()
    bad["pairs"] = bad["pairs"][:21]
    assert any("22 行" in x for x in L.validate_manifest(bad))


def test_validate_cli_roundtrip(tmp_root: Path | None = None):
    m = _fake_manifest()
    with tempfile.TemporaryDirectory() as td:
        p = Path(td) / "m.json"
        p.write_text(json.dumps(m, ensure_ascii=False), encoding="utf-8")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = L.main(["--validate", str(p)])
        assert rc == 0, (rc, buf.getvalue())
        out = buf.getvalue()
        assert "MANIFEST-CHECK: PASS" in out and "registry_ok=True" in out
        # 坏 manifest ⇒ exit 1
        m["pairs"] = m["pairs"][:21]
        p.write_text(json.dumps(m), encoding="utf-8")
        with contextlib.redirect_stdout(io.StringIO()):
            assert L.main(["--validate", str(p)]) == 1


def test_parse_render_log_and_strict_line():
    log = ('x frame=1 base=0 scroll=(0,0) nt_nz=0 probe=1 scene=s env={} '
           'argv=["1", "n", "s", "--out-dir", "d"] in=frames:aa,probe:-,text:bb out=cc\n')
    parsed = L.parse_render_log(log)
    assert parsed["env"] == {}
    assert parsed["argv"][:3] == ["1", "n", "s"]
    assert L.parse_strict_log("foo\nSTRICT: FAIL diff=1\nbar\nSTRICT: FAIL diff=2\n") == \
        "STRICT: FAIL diff=2"
    assert L.parse_strict_log("no strict line") is None


def test_write_mapping_22_rows_with_expected_source():
    with tempfile.TemporaryDirectory() as td:
        p = Path(td) / "mapping.tsv"
        L.write_mapping(L.PAIR_SPEC, p)
        lines = p.read_text(encoding="utf-8").splitlines()
        assert len(lines) == 23, f"header+22 行，实际 {len(lines)}"
        header = lines[0].split("\t")
        assert header == L.MAPPING_COLUMNS
        assert len(header) == len(set(header)) == 9
        rows = [dict(zip(header, ln.split("\t"))) for ln in lines[1:]]
        assert len(rows) == 22
        for r in rows:
            assert r["name"]
            if r["scene"]:
                assert r["expected_source"] == f'{r["scene"]}:{r["frame"]}:{r["capture_tag"]}'
        assert sum(1 for r in rows if r["duplicate_of"]) == 4  # 3 个四联孩子 + ending-4（#14 已清空）


def test_build_manifest_integration_if_inputs_present():
    """只读集成：现有 g_run1/g_run2 实跑一次（不写文件），锁 summary/dedupe 数值。"""
    render_dir = ROOT / "target/p0-0/render"
    if not (render_dir / "g_run1").is_dir() or not (render_dir / "g-manifest-v2.json").is_file():
        print("  SKIP  build_manifest 集成（target/p0-0/render 输入不在）")
        return
    m = L.build_manifest(render_dir, ROOT / "oracle/screens/expected")
    s = m["summary"]
    assert s["pairs_total"] == 22
    assert s["rendered"] == 18 and s["blocked_no_source"] == 4
    assert s["replaced_invalid_asset"] == 0 and s["documented_duplicate"] == 5
    assert s["two_run_deterministic"] == 18
    # #14 expected 安装（task-165）前：旧件仍与 title 同 blob ⇒ expected_unique 13 / reduction 5；
    # 安装后（44e7800d…）⇒ 14 / 4。两种状态都应 --validate PASS；测试按当前文件状态自适应。
    p14 = ROOT / "oracle/screens/expected/l4_stg02-start.png"
    installed = p14.is_file() and L.sha256_file(p14).startswith("44e7800d")
    if installed:
        assert m["dedupe"]["expected_unique"] == 14 and m["dedupe"]["expected_reduction"] == 4
        assert len(m["dedupe"]["documented_duplicates"]) == 2
        assert m["dedupe"]["registry_ok"] is True
        assert m["validation"]["ok"] is True, m["validation"]
    else:
        # task-166 同步完成、task-165 安装前的过渡态：旧 expected 仍与 title 同 blob，
        # 但 mapping 已按新裁定清空 duplicate_of ⇒ registry 暂不成立（这是**预期阻塞**，
        # 安装 44e7800d 后自动 PASS）。见 test_postinstall_simulation_if_stage_available。
        assert m["dedupe"]["expected_unique"] == 13 and m["dedupe"]["expected_reduction"] == 5
        assert len(m["dedupe"]["documented_duplicates"]) == 3
        assert m["dedupe"]["registry_ok"] is False
        assert m["validation"]["ok"] is False
        assert not m["validation"]["pair_issues"], m["validation"]["pair_issues"]
    assert m["dedupe"]["actual_unique"] == 14 and m["dedupe"]["actual_reduction"] == 4
    # #14 pair 重指向（task-166 Option B）与 44e7800d 预期安装目标
    p14row = next(p for p in m["pairs"] if p["name"] == "l4_stg02-start")
    assert p14row["scene"] == "gs5-anchor-render" and p14row["frame"] == 3450
    assert p14row["expected_source"] == "gs5-anchor-render:3450:stg02-start"
    assert p14row["trigger"] == "injected" and p14row["status"] == L.STATUS_OK
    assert p14row["reuse"] == "independent" and p14row["duplicate_of"] == ""
    # 4 blocked 行：expected/actual 空 + status
    for p in m["pairs"]:
        if p["status"] == L.STATUS_BLOCKED:
            assert p["expected_sha256"] == "" and p["actual_sha256"] == ""
            assert p["deterministic"] is False
    # 四联/#17/#18 trigger 更正 + expected_source tag 更正
    for p in m["pairs"]:
        if p["name"].startswith(("l4_stg06-late", "l4_stg08-", "l4_stg07-")):
            assert p["trigger"] == "injected", p["name"]
        if p["name"] in ("l4_ending-4", "l4_ending-5"):
            assert p["expected_source"].endswith(":probe")
            assert p["trigger"] == "assisted"


def test_strict_from_args_records_only():
    """--strict 接口只回填 STRICT 行，不执行 diff_visual（本测试不产生 subprocess）。"""
    import argparse
    ns = argparse.Namespace(strict_line="STRICT: FAIL diff=12", strict_log=None,
                            strict_exit=1, strict_after_asset_swap=True)
    rec = L.strict_from_args(ns)
    assert rec["line"] == "STRICT: FAIL diff=12"
    assert rec["exit"] == 1 and rec["cmd"].endswith("diff_visual.py --strict")
    assert rec["ran_after_asset_swap"] is True
    ns = argparse.Namespace(strict_line=None, strict_log=None, strict_exit=1,
                            strict_after_asset_swap=True)
    assert L.strict_from_args(ns) is None


def test_trigger_enum_and_negative():
    # task-168 F3：trigger 枚举显式断言 + 负向测试
    assert L.ALLOWED_TRIGGER == {"natural", "injected", "assisted", "unknown"}
    ts = {p["trigger"] for p in L.PAIR_SPEC}
    assert ts <= L.ALLOWED_TRIGGER
    assert "injected" in ts and "assisted" in ts  # 口径覆盖
    assert "unknown" in ts  # blocked 4 对
    bad = _fake_manifest()
    q = next(x for x in bad["pairs"] if x["status"] == L.STATUS_OK)
    q["trigger"] = "cheat"
    assert any("trigger" in x for x in L.validate_manifest(bad))


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
    print(f"l4_render_all 回归: {len(ALL_TESTS) - failed}/{len(ALL_TESTS)} 通过")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
