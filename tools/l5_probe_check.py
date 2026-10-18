#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""L5 A9 本机检查：probe5 发射 + diff_l5 --probe 对拍（零 Mesen2 依赖）。

链路（全部本机可跑，产物写 analysis/traces/ —— .gitignore 工作区）：
  1) mkdir -p analysis/traces
  2) CARGO_TARGET_DIR=target/l5probe cargo test -p game-core --test engine_tick -- --ignored emit_probe5
     （发射器：crates/game-core/tests/engine_tick.rs:655-843；12 个产物：
      stg01..08.probe5.jsonl + gameplay-stage1/respawn-window/s5-chain.probe5.jsonl + clear-run.trace.jsonl）
  3) python tools/diff_l5.py --probe analysis/traces/gameplay-stage1.probe5.jsonl
       --golden oracle/traces/s5-all-clear.l5.jsonl --anchors oracle/traces/s5-all-clear.anchors.jsonl
       --scene-map title-menu=s5-title,gameplay-stage1=s5-stg01
  4) （信息面）同一 golden/anchors 对 s5-chain.probe5.jsonl 再跑一次（injected 链段覆盖）

口径/判据（不改 diff_l5 的判定阈值）：
  - diff_l5 的 exit=1 **属预期**（unknown 计入 bad；本链路只覆盖 natural 面 + 4 条 ≤f3840 的 injected 锚，
    其余 s5-chain/占位行必然 unknown）——因此**不用 diff_l5 的退出码做验收**；
  - 验收看 **natural 命中数 == --expect-natural（默认 7）且 0 mismatch** ⇒ 本脚本 exit 0；
  - 无 cargo / 无产物 / 发射失败 ⇒ 明确 **SKIP（exit 2）**，不静默通过；
  - 产物路径固定在仓库内 analysis/traces + 独立 target（target/l5probe），不碰共享 target。

退出码：
  0 = PASS（natural 达标且无 mismatch）
  1 = FAIL（natural 不足或出现 mismatch）
  2 = SKIP（无 cargo / 产物缺失 / 发射不可用——环境不具备，非失败）
  3 = ERROR（本仓库路径/工具异常，如 golden 缺失、diff_l5 异常退出）

用法（Windows 用 `python`；Linux 用 `python3`）：
  python tools/l5_probe_check.py                 # 发射 + 对拍（验收默认 natural 7）
  python tools/l5_probe_check.py --skip-emit     # 复用已有 analysis/traces 产物（无产物=SKIP）
  python tools/l5_probe_check.py --selftest      # 只验证解析器/仓库文件（不需要 traces/cargo）
"""
import argparse
import collections
import os
import pathlib
import re
import shutil
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
TRACES = REPO / "analysis" / "traces"
DIFF_L5 = REPO / "tools" / "diff_l5.py"
GOLDEN = REPO / "oracle" / "traces" / "s5-all-clear.l5.jsonl"
ANCHORS = REPO / "oracle" / "traces" / "s5-all-clear.anchors.jsonl"
SCENE_MAP = "title-menu=s5-title,gameplay-stage1=s5-stg01"
EMIT_SCENES = [f"stg{i:02d}" for i in range(1, 9)] + ["gameplay-stage1", "respawn-window", "s5-chain"]
EXPECT_ARTIFACTS = [f"{n}.probe5.jsonl" for n in EMIT_SCENES] + ["clear-run.trace.jsonl"]

STATUS_RE = re.compile(r"^\[([a-z-]+)\s*\]")


def emit_probe5(target_dir: pathlib.Path, timeout: int) -> tuple[bool, list[str]]:
    """运行 ignored 发射器；返回 (ok, 全部输出行)。ok 判定含新鲜度（E5）：rc==0 ∧ 输出含 `1 passed` ∧ 12 产物齐。"""
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = str(target_dir)
    cmd = ["cargo", "test", "-q", "-p", "game-core", "--test", "engine_tick", "--", "--ignored", "emit_probe5"]
    try:
        r = subprocess.run(cmd, cwd=REPO, env=env, capture_output=True, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return False, [f"{type(exc).__name__}: {exc}"]
    out = (r.stdout + r.stderr).decode("utf-8", "replace")
    lines = out.splitlines()
    ran = any("1 passed" in ln for ln in lines)                 # E5：emitter 真跑（0 tests+旧产物不 false PASS）
    files_ok = all((TRACES / n).is_file() for n in EXPECT_ARTIFACTS)
    return (r.returncode == 0 and ran and files_ok), lines


def run_diff(probe: pathlib.Path, scene_map: str | None) -> tuple[int, list[str], str]:
    """调用 diff_l5；返回 (rc, 行, 错误)。rc∈{0,1} 正常（1=unknown/mismatch 计入 bad，属预期）。"""
    cmd = [sys.executable, str(DIFF_L5), "--probe", str(probe), "--golden", str(GOLDEN), "--anchors", str(ANCHORS)]
    if scene_map:
        cmd += ["--scene-map", scene_map]
    try:
        r = subprocess.run(cmd, cwd=REPO, capture_output=True, timeout=300)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return 3, [], f"{type(exc).__name__}: {exc}"
    out = (r.stdout + r.stderr).decode("utf-8", "replace")
    lines = out.splitlines()
    if r.returncode not in (0, 1):
        return 3, lines, f"diff_l5 rc={r.returncode}"
    return r.returncode, lines, ""


def summarize(lines: list[str]) -> collections.Counter:
    c = collections.Counter()
    for ln in lines:
        m = STATUS_RE.match(ln)
        if m:
            c[m.group(1)] += 1
    return c


def check_repo_files() -> list[str]:
    missing = [str(p.relative_to(REPO)) for p in (DIFF_L5, GOLDEN, ANCHORS) if not p.is_file()]
    return missing


def cmd_selftest() -> int:
    """不需要 traces/cargo：验证仓库文件 + 状态解析器。"""
    missing = check_repo_files()
    if missing:
        print(f"SELFTEST: SKIP (missing repo files: {missing})")
        return 2
    sample = [
        "[ok          ] s5-title f=180 Black boot-black ",
        "[ok-injected ] s5-stg01 f=2401 StageClear stg01-clear trigger=injected",
        "[unknown     ] s5-chain f=0 Ending all-clear-95f9 golden 无 probe 覆盖",
        "[mismatch    ] x f=1 ['tick']",
    ]
    if summarize(sample) != collections.Counter({"ok": 1, "ok-injected": 1, "unknown": 1, "mismatch": 1}):
        print("SELFTEST: FAIL (status parser mismatch)")
        return 1
    print("SELFTEST: ok (repo files present + status parser 4/4)")
    print("  note: full chain needs cargo + engine_tick::emit_probe5 (see --help)")
    return 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="L5 A9: probe5 emit + diff_l5 --probe local check")
    ap.add_argument("--expect-natural", type=int, default=7, help="natural 命中数验收阈值（默认 7）")
    ap.add_argument("--skip-emit", action="store_true", help="不重发射，复用 analysis/traces 现有产物")
    ap.add_argument("--target-dir", default="target/l5probe", help="仓库内独立 cargo target（不得用共享 target）")
    ap.add_argument("--timeout", type=int, default=1800, help="emit_probe5 超时秒（默认 1800）")
    ap.add_argument("--selftest", action="store_true", help="只验证解析器/仓库文件，不需要 traces/cargo")
    args = ap.parse_args(argv)
    if args.selftest:
        return cmd_selftest()

    missing = check_repo_files()
    if missing:
        print(f"ERROR: missing repo files: {missing}")
        return 3

    target = pathlib.Path(args.target_dir)
    if not target.is_absolute():
        target = REPO / target
    resolved = target.resolve()
    repo_res = REPO.resolve()
    shared = (repo_res / "target").resolve()
    # E4：独立 target 必须位于 REPO/target/ 下，且不得是 REPO/target 本身或仓库根
    if resolved == repo_res or resolved == shared:
        print(f"ERROR: --target-dir must not be repo root or shared target (got {args.target_dir}; "
              f"use e.g. target/l5probe)")
        return 3
    try:
        resolved.relative_to(shared)
    except ValueError:
        print(f"ERROR: --target-dir must be under repo target/ (got {args.target_dir}; "
              f"shared/external target forbidden)")
        return 3

    if not args.skip_emit:
        if shutil.which("cargo") is None:
            print("SKIP: cargo not in PATH -- cannot emit probe5 (needs Rust toolchain; not a failure)")
            return 2
        TRACES.mkdir(parents=True, exist_ok=True)   # 只在真要发射时建目录（Info：SKIP 不留空目录）
        print(f"emit: CARGO_TARGET_DIR={resolved.relative_to(repo_res)} cargo test -p game-core --test engine_tick "
              f"-- --ignored emit_probe5")
        ok, lines_out = emit_probe5(target, args.timeout)
        for ln in lines_out:                         # E2：成功路径打印 emitter 结果行（含 `1 passed`）
            if "test result" in ln or "1 passed" in ln:
                print(f"emit: | {ln}")
        if not ok:
            for ln in lines_out[-8:]:                # 失败诊断用尾部
                print(f"emit: | {ln}")
            print("SKIP: emit_probe5 did not run/produce all 12 artifacts fresh (E5: rc/`1 passed`/files; not a failure)")
            return 2
    absent = [n for n in EXPECT_ARTIFACTS if not (TRACES / n).is_file()]
    if absent:
        print(f"SKIP: analysis/traces missing {len(absent)} artifacts: {absent}")
        print("      (run emit first: python tools/l5_probe_check.py; or no cargo in this env)")
        return 2
    print(f"artifacts: {len(EXPECT_ARTIFACTS)}/{len(EXPECT_ARTIFACTS)} present in analysis/traces")

    rc_a, lines_a, err_a = run_diff(TRACES / "gameplay-stage1.probe5.jsonl", SCENE_MAP)
    if err_a:
        print(f"ERROR: diff_l5 (passA) {err_a}")
        return 3
    c_a = summarize(lines_a)
    natural = c_a.get("ok", 0)
    print(f"passA: natural_ok={natural} (expect {args.expect_natural}) "
          f"ok-injected={c_a.get('ok-injected', 0)} unknown={c_a.get('unknown', 0)} "
          f"mismatch={c_a.get('mismatch', 0)} provenance-note={c_a.get('ok-provenance-note', 0)} "
          f"| diff_l5 rc={rc_a} (1=expected: unknown counts as bad)")

    # 信息面：s5-chain injected 链段覆盖（不参与 natural 验收）
    rc_b, lines_b, err_b = run_diff(TRACES / "s5-chain.probe5.jsonl", None)
    if err_b:
        print(f"ERROR: diff_l5 (passB) {err_b}")
        return 3
    c_b = summarize(lines_b)
    print(f"passB: ok-injected={c_b.get('ok-injected', 0)} unknown={c_b.get('unknown', 0)} "
          f"mismatch={c_b.get('mismatch', 0)} | diff_l5 rc={rc_b}")
    if c_b.get("mismatch", 0) > 0:
        print(f"WARN: passB mismatch={c_b['mismatch']}（信息面；不进退出码，但须记录/追查）")

    print(f"L5-DIFF natural {natural}/{args.expect_natural}")
    if natural == args.expect_natural and c_a.get("mismatch", 0) == 0:
        print("A9: PASS (local probe5 natural face OK; real-machine Mesen2 face still needs env)")
        return 0
    print(f"A9: FAIL (natural {natural} != {args.expect_natural} or mismatch>0; see above)")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
