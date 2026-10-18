#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tools/l4_render_all.py —— L4 P0-0 manifest 生成 + 去重 registry 校验 + strict 行接口（task-155）。

职责（**不代跑 visual-report**；不改 expected/threshold/判定）：
1. 以现有 P0-0-2 GUI 两跑产物为输入（默认 `target/p0-0/render/{g_run1,g_run2}` +
   `g-manifest-v2.json`），生成 **22 行全字段 manifest**（§5.1 schema：scene/frame/capture_tag/
   trigger/input_sha256/render_env/render_argv/expected_sha256/actual_sha256/status/reuse/duplicate_of）。
2. 生成/写回 `analysis/repro/mapping.tsv`（§3.5 schema 22 行；status/reuse/duplicate_of 来自
   本工具内嵌 P0-0 裁定表）。
3. **§7.5 B2 后口径去重登记**：expected/actual 两侧按 manifest 名过滤分组，`unique == present −
   Σ(len(group)−1)`（present=有该侧资产的 manifest 行数；4 blocked 两侧均无资产 ⇒ present=18），
   `documented_duplicates` 登记 title↔stg02 / 四联×4 / ending-4/5；每组非主帧在 mapping 标
   `duplicate_of`；输出 `registry_ok=true`。
4. `--strict`：只**采集/回填** `STRICT:` 输出行（`--strict-line` 直给或 `--strict-log` 读文件），
   不执行 `diff_visual.py`、不写 `oracle/screens/`。

用法：
    python tools/l4_render_all.py                       # 生成 + 校验（默认输出 g-manifest-full.json）
    python tools/l4_render_all.py --validate PATH       # 只校验既有 manifest（schema/registry/两跑）
    python tools/l4_render_all.py --strict-log FILE --strict-exit 1
    python tools/l4_render_all.py --render-dir target/p0-0/render --out target/p0-0/render/g-manifest-full.json

退出码：0=全部校验 PASS；1=校验 FAIL；2=输入缺失/结构错误（BLOCKED）。
"""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import platform
import sys
from datetime import datetime, timezone

ROOT = pathlib.Path(__file__).resolve().parent.parent

STATUS_OK = "ok"
STATUS_BLOCKED = "blocked:no-source"
STATUS_REPLACED = "replaced-invalid-asset"
STATUS_DUPLICATE = "documented-duplicate"
ALLOWED_STATUS = {STATUS_OK, STATUS_BLOCKED, STATUS_REPLACED, STATUS_DUPLICATE,
                  "blocked:probe-cap", "blocked:nondeterministic"}
ALLOWED_REUSE = {"independent", "reuse"}
ALLOWED_TRIGGER = {"natural", "injected", "assisted", "unknown"}
REQUIRED_PAIR_FIELDS = (
    "name", "scene", "frame", "capture_tag", "frame_offset", "trigger", "expected_source",
    "expected_sha256", "actual_sha256", "actual_run1_sha256", "actual_run2_sha256",
    "input_sha256", "render_env", "render_argv", "status", "reuse", "duplicate_of",
    "deterministic",
)

# ---------------------------------------------------------------------------
# 22 对 P0-0 裁定表（A 级来源：手册 §3.1/附录 A + task-143/146 GUI 重裁 + §7.5 GUI 去重实况）。
# status/reuse/duplicate_of/trigger/capture_tag 为 mapping.tsv 透传面（M1）；
# 4 blocked 的 expected/actual 资产为空（expected_sha256=""）。
# ---------------------------------------------------------------------------
PAIR_SPEC: list[dict] = [
    # name, scene, frame, capture_tag, trigger, expected_file, status, reuse, duplicate_of
    {"name": "gameplay-stage1_f01260", "scene": "gameplay-stage1", "frame": 1260,
     "capture_tag": "stage-start", "trigger": "natural",
     "expected_file": "gameplay-stage1_f01260_stage-start.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "gameplay-stage1_f01500", "scene": "gameplay-stage1", "frame": 1500,
     "capture_tag": "mid-combat", "trigger": "natural",
     "expected_file": "gameplay-stage1_f01500_mid-combat.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "gameplay-stage1_f02349", "scene": "gameplay-stage1", "frame": 2349,
     "capture_tag": "end", "trigger": "natural",
     "expected_file": "gameplay-stage1_f02349_end.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_clear-inject", "scene": "stg03-timeline", "frame": 4600,
     "capture_tag": "stg02-clear-inject", "trigger": "injected",
     "expected_file": "l4_clear-inject.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_combat-variety", "scene": "stg03-timeline", "frame": 6500,
     "capture_tag": "stg03-combat", "trigger": "injected",
     "expected_file": "l4_combat-variety.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_ending-1", "scene": "", "frame": 30000,
     "capture_tag": "probe", "trigger": "unknown",
     "expected_file": "l4_ending-1.png",
     "status": STATUS_BLOCKED, "reuse": "reuse", "duplicate_of": ""},
    {"name": "l4_ending-2", "scene": "", "frame": 31500,
     "capture_tag": "probe", "trigger": "unknown",
     "expected_file": "l4_ending-2.png",
     "status": STATUS_BLOCKED, "reuse": "reuse", "duplicate_of": ""},
    {"name": "l4_ending-3", "scene": "", "frame": 33000,
     "capture_tag": "probe", "trigger": "unknown",
     "expected_file": "l4_ending-3.png",
     "status": STATUS_BLOCKED, "reuse": "reuse", "duplicate_of": ""},
    {"name": "l4_ending-4", "scene": "tas-stg08b2-direct", "frame": 34500,
     "capture_tag": "l4f34500", "trigger": "assisted",
     "expected_file": "l4_ending-4.png",
     "status": STATUS_DUPLICATE, "reuse": "reuse", "duplicate_of": "l4_ending-5"},
    {"name": "l4_ending-5", "scene": "tas-stg08b2-direct", "frame": 36000,
     "capture_tag": "l4f36000", "trigger": "assisted",
     "expected_file": "l4_ending-5.png",
     "status": STATUS_DUPLICATE, "reuse": "reuse", "duplicate_of": ""},
    {"name": "l4_fade-clear", "scene": "", "frame": 28500,
     "capture_tag": "probe", "trigger": "unknown",
     "expected_file": "l4_fade-clear.png",
     "status": STATUS_BLOCKED, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_late-combat", "scene": "stg03-timeline", "frame": 12000,
     "capture_tag": "stg03-late", "trigger": "injected",
     "expected_file": "l4_late-combat.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_shop-window", "scene": "stg03-timeline", "frame": 5400,
     "capture_tag": "probe", "trigger": "injected",
     "expected_file": "l4_shop-window.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_stg02-start", "scene": "stg03-timeline", "frame": 3404,
     "capture_tag": "stg02-start", "trigger": "injected",
     "expected_file": "l4_stg02-start.png",
     "status": STATUS_REPLACED, "reuse": "reuse",
     "duplicate_of": "title-menu_f00180_boot-black"},
    {"name": "l4_stg03-start", "scene": "stg03-timeline", "frame": 5700,
     "capture_tag": "stg03-start-approx", "trigger": "injected",
     "expected_file": "l4_stg03-start.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_stg06-late", "scene": "stg08-timeline", "frame": 22000,
     "capture_tag": "l4f22000", "trigger": "natural",
     "expected_file": "l4_stg06-late.png",
     "status": STATUS_REPLACED, "reuse": "reuse", "duplicate_of": ""},
    {"name": "l4_stg07-combat", "scene": "stg08-timeline", "frame": 16000,
     "capture_tag": "stg07-combat", "trigger": "natural",
     "expected_file": "l4_stg07-combat.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_stg07-start", "scene": "stg08-timeline", "frame": 14600,
     "capture_tag": "stg07-start-approx", "trigger": "natural",
     "expected_file": "l4_stg07-start.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
    {"name": "l4_stg08-combat", "scene": "stg08-timeline", "frame": 18000,
     "capture_tag": "l4f18000", "trigger": "natural",
     "expected_file": "l4_stg08-combat.png",
     "status": STATUS_REPLACED, "reuse": "reuse", "duplicate_of": "l4_stg06-late"},
    {"name": "l4_stg08-late", "scene": "stg08-timeline", "frame": 28000,
     "capture_tag": "l4f28000", "trigger": "natural",
     "expected_file": "l4_stg08-late.png",
     "status": STATUS_REPLACED, "reuse": "reuse", "duplicate_of": "l4_stg06-late"},
    {"name": "l4_stg08-start", "scene": "stg08-timeline", "frame": 16600,
     "capture_tag": "l4f16600", "trigger": "natural",
     "expected_file": "l4_stg08-start.png",
     "status": STATUS_REPLACED, "reuse": "reuse", "duplicate_of": "l4_stg06-late"},
    {"name": "title-menu_f00180_boot-black", "scene": "title-menu", "frame": 180,
     "capture_tag": "boot-black", "trigger": "natural",
     "expected_file": "title-menu_f00180_boot-black.png",
     "status": STATUS_OK, "reuse": "independent", "duplicate_of": ""},
]

SPEC_BY_NAME = {p["name"]: p for p in PAIR_SPEC}
MAPPING_COLUMNS = ["name", "scene", "frame", "capture_tag", "trigger", "expected_source",
                   "status", "reuse", "duplicate_of"]


def sha256_file(path: pathlib.Path) -> str | None:
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


def spec_issues(spec: list[dict] | None = None) -> list[str]:
    """内嵌裁定表结构自检（22 行/字段/枚举/blocked 资产空语义/duplicate_of 合法）。"""
    rows = PAIR_SPEC if spec is None else spec
    issues: list[str] = []
    if len(rows) != 22:
        issues.append(f"pair_spec 行数={len(rows)}，需 22")
    names = [r.get("name") for r in rows]
    if len(set(names)) != len(names):
        issues.append("pair_spec name 重复")
    name_set = set(names)
    for r in rows:
        name = r.get("name", "?")
        for k in ("scene", "frame", "capture_tag", "trigger", "expected_file",
                  "status", "reuse", "duplicate_of"):
            if k not in r:
                issues.append(f"{name}: 缺字段 {k}")
        if r.get("status") not in ALLOWED_STATUS:
            issues.append(f"{name}: status 非法 {r.get('status')!r}")
        if r.get("reuse") not in ALLOWED_REUSE:
            issues.append(f"{name}: reuse 非法 {r.get('reuse')!r}")
        if r.get("trigger") not in ALLOWED_TRIGGER:
            issues.append(f"{name}: trigger 非法 {r.get('trigger')!r}")
        if r.get("duplicate_of") and r["duplicate_of"] not in name_set:
            issues.append(f"{name}: duplicate_of 指向不存在的 {r['duplicate_of']}")
        if r.get("duplicate_of") == name:
            issues.append(f"{name}: duplicate_of 不能指向自身")
    # status 计数口径（task-155：4 blocked / 5 replaced / 2 documented duplicate）
    counts = {s: sum(1 for r in rows if r.get("status") == s) for s in
              (STATUS_OK, STATUS_BLOCKED, STATUS_REPLACED, STATUS_DUPLICATE)}
    if counts[STATUS_BLOCKED] != 4:
        issues.append(f"blocked:no-source 应=4，实际 {counts[STATUS_BLOCKED]}")
    if counts[STATUS_REPLACED] != 5:
        issues.append(f"replaced-invalid-asset 应=5，实际 {counts[STATUS_REPLACED]}")
    if counts[STATUS_DUPLICATE] != 2:
        issues.append(f"documented-duplicate 应=2（ending-4/5），实际 {counts[STATUS_DUPLICATE]}")
    return issues


def parse_render_log(text: str) -> dict:
    """解析 render-frame 日志末行：env/argv/in。返回 {} 表示无日志行。"""
    for line in reversed(text.splitlines()):
        if " env=" not in line or " argv=" not in line:
            continue
        try:
            env_part = line.split(" env=", 1)[1].split(" argv=", 1)[0]
            argv_part = line.split(" argv=", 1)[1].split(" in=frames:", 1)[0]
            env = json.loads(env_part)
            argv = json.loads(argv_part)
            return {"env": env, "argv": argv}
        except (ValueError, json.JSONDecodeError):
            continue
    return {}


def parse_strict_log(text: str) -> str | None:
    """取最后一条 `STRICT:` 行（task-155：只采集，不执行 diff_visual）。"""
    lines = [ln.strip() for ln in text.splitlines() if ln.strip().startswith("STRICT:")]
    return lines[-1] if lines else None


def _load_pairs_v2(manifest_v2: pathlib.Path) -> dict[str, dict]:
    data = json.loads(manifest_v2.read_text(encoding="utf-8"))
    return {p["name"]: p for p in data.get("pairs", [])}


def _mapping_group_sets(rows: list[dict]) -> list[set[str]]:
    """mapping.duplicate_of 组：master + 所有孩子（孩子=duplicate_of==master）。"""
    groups: list[set[str]] = []
    for r in rows:
        if not r.get("duplicate_of"):
            children = {x["name"] for x in rows if x.get("duplicate_of") == r["name"]}
            if children:
                groups.append({r["name"]} | children)
    return groups


def build_manifest(render_dir: pathlib.Path, expected_dir: pathlib.Path,
                   strict: dict | None = None) -> dict:
    """从 g_run1/g_run2 + g-manifest-v2 生成 22 行全字段 manifest（不渲染、不写文件）。"""
    manifest_v2 = render_dir / "g-manifest-v2.json"
    run1 = render_dir / "g_run1"
    run2 = render_dir / "g_run2"
    for need in (manifest_v2, run1, run2, expected_dir):
        if not need.exists():
            raise SystemExit(f"BLOCKED: 缺输入 {need}")
    v2 = _load_pairs_v2(manifest_v2)
    pairs: list[dict] = []
    issues: list[str] = list(spec_issues())
    for spec in PAIR_SPEC:
        name = spec["name"]
        rec = v2.get(name)
        blocked = spec["status"] == STATUS_BLOCKED
        exp_path = expected_dir / spec["expected_file"]
        exp_sha = "" if blocked else (sha256_file(exp_path) or "")
        png1 = run1 / f"{name}.png"
        idx1 = run1 / f"{name}.idx.json"
        png2 = run2 / f"{name}.png"
        idx2 = run2 / f"{name}.idx.json"
        a1 = sha256_file(png1)
        a2 = sha256_file(png2)
        i1 = sha256_file(idx1)
        i2 = sha256_file(idx2)
        deterministic = bool(a1 and a2 and i1 and i2 and a1 == a2 and i1 == i2)
        log_text = ""
        for logp in (run1 / f"{name}.log",):
            if logp.is_file():
                log_text = logp.read_text(encoding="utf-8", errors="replace")
        parsed = parse_render_log(log_text)
        input_sha = dict((rec or {}).get("input_sha") or {})
        pairs.append({
            "name": name,
            "scene": spec["scene"],
            "frame": int(spec["frame"]),
            "capture_tag": spec["capture_tag"],
            "frame_offset": 0,
            "trigger": spec["trigger"],
            "expected_source": (f"{spec['scene']}:{spec['frame']}:{spec['capture_tag']}"
                                if spec["scene"] else ""),
            "expected_sha256": exp_sha,
            "actual_sha256": a1 or "",
            "actual_run1_sha256": a1 or "",
            "actual_run2_sha256": a2 or "",
            "input_sha256": {"frames": input_sha.get("frames", ""),
                             "probe": input_sha.get("probe", ""),
                             "text": input_sha.get("text", "")},
            "render_env": parsed.get("env", {}),
            "render_argv": parsed.get("argv", []),
            "status": spec["status"],
            "reuse": spec["reuse"],
            "duplicate_of": spec["duplicate_of"],
            "deterministic": deterministic,
            "notes": ("no-source（dense 全扫零命中，见 p0-0-recapture-gui-run §5.2）"
                      if spec["status"] == STATUS_BLOCKED else
                      ("expected-invalid 待 §7.2 重采（expected_sha256=旧件 sha，A 级证据）"
                       if spec["status"] == STATUS_REPLACED else
                       ("expected/actual 两侧同图，已登记 documented_duplicates"
                        if spec["status"] == STATUS_DUPLICATE else ""))),
        })
        if blocked:
            if exp_sha or a1 or deterministic:
                issues.append(f"{name}: blocked 行应 expected/actual 空且 deterministic=false")
        else:
            if not exp_sha:
                issues.append(f"{name}: expected 文件缺 {exp_path.name}")
            if not a1 or not a2:
                issues.append(f"{name}: actual run1/run2 缺图")
            if not deterministic:
                issues.append(f"{name}: 两跑 png/idx 不一致")
            if parsed.get("env"):
                issues.append(f"{name}: render_env 非空（canonical 污染）{parsed['env']}")
            if not parsed.get("argv"):
                issues.append(f"{name}: render_argv 缺失（日志不可解析）")
        if rec is not None:
            # g-manifest-v2 交叉核对（run shas/expected_file；只报警不覆盖）
            if rec.get("actual_run1_sha256") and a1 and rec["actual_run1_sha256"] != a1:
                issues.append(f"{name}: g-manifest-v2 run1 sha 与磁盘不符")
            if rec.get("expected_file") and rec["expected_file"] != spec["expected_file"]:
                issues.append(f"{name}: expected_file 与 g-manifest-v2 不符")
        else:
            if not blocked:
                issues.append(f"{name}: g-manifest-v2 缺该 pair 记录")
    rendered = [p for p in pairs if p["status"] != STATUS_BLOCKED]
    summary = {
        "pairs_total": len(pairs),
        "rendered": len(rendered),
        "blocked_no_source": sum(1 for p in pairs if p["status"] == STATUS_BLOCKED),
        "replaced_invalid_asset": sum(1 for p in pairs if p["status"] == STATUS_REPLACED),
        "documented_duplicate": sum(1 for p in pairs if p["status"] == STATUS_DUPLICATE),
        "two_run_deterministic": sum(1 for p in rendered if p["deterministic"]),
        "expected_matches": sum(1 for p in rendered
                                if p["expected_sha256"] and p["expected_sha256"] == p["actual_sha256"]),
        "trigger_breakdown": {
            t: sum(1 for p in rendered if p["trigger"] == t)
            for t in ("natural", "injected", "assisted")},
    }
    dedupe, dedupe_ok = build_dedupe(pairs)
    manifest = {
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "task": "task-155（冻结前置②：manifest 22 行全字段 + B2 去重登记）",
        "env": {
            "os": f"{platform.system()} {platform.machine()}",
            "python": platform.python_version(),
            "rom_sha256": sha256_file(ROOT / "rom/gunnac_cn.nes") or "",
            "mesen_sha256": sha256_file(ROOT / "tools/mesen2/Mesen.exe") or "",
        },
        "inputs": {
            "render_dir": render_dir.relative_to(ROOT).as_posix()
            if render_dir.is_relative_to(ROOT) else str(render_dir),
            "g_manifest_v2_sha256": sha256_file(manifest_v2),
            "run1_dir": "g_run1",
            "run2_dir": "g_run2",
            "expected_dir": expected_dir.relative_to(ROOT).as_posix()
            if expected_dir.is_relative_to(ROOT) else str(expected_dir),
            "render_binary_sha256": (json.loads(manifest_v2.read_text(encoding="utf-8"))
                                     .get("summary", {}).get("render_binary_sha256", "")),
        },
        "summary": summary,
        "canonical": {"env": {}, "argv_template": ["cargo", "run", "-q", "-p", "game-desktop",
                                                   "--bin", "render-frame", "--"]},
        "pairs": pairs,
        "dedupe": dedupe,
        "strict": strict,
    }
    manifest["validation"] = {
        "spec_issues": spec_issues(),
        "pair_issues": issues,
        "dedupe_ok": dedupe_ok,
        "registry_ok": dedupe.get("registry_ok", False),
        "two_run_ok": summary["two_run_deterministic"] == summary["rendered"],
        "counts_ok": (summary["rendered"] == 18 and summary["blocked_no_source"] == 4
                      and summary["replaced_invalid_asset"] == 5
                      and summary["documented_duplicate"] == 2),
    }
    manifest["validation"]["ok"] = (not issues and dedupe_ok and manifest["validation"]["registry_ok"]
                                    and manifest["validation"]["two_run_ok"]
                                    and manifest["validation"]["counts_ok"])
    return manifest


def build_dedupe(pairs: list[dict]) -> tuple[dict, bool]:
    """§7.5 B2 后口径：两侧分组 + documented_duplicates + registry 等式。"""
    rows = list(pairs)

    def side_groups(side: str) -> dict[frozenset[str], dict]:
        """按 name 集分组的重复组（len>1）：{names_key: {sha, names}}。"""
        by_sha: dict[str, list[str]] = {}
        for p in rows:
            key = p.get(f"{side}_sha256") or ""
            if not key:  # blocked 行（该侧无资产）不入组
                continue
            by_sha.setdefault(key, []).append(p["name"])
        out: dict[frozenset[str], dict] = {}
        for sha, names in by_sha.items():
            if len(names) > 1:
                out[frozenset(names)] = {"sha": sha, "names": names}
        return out

    def side_unique(side: str) -> int:
        shas = {p.get(f"{side}_sha256") for p in rows if p.get(f"{side}_sha256")}
        return len(shas)

    exp_groups = side_groups("expected")
    act_groups = side_groups("actual")
    present_exp = sum(1 for p in rows if p.get("expected_sha256"))
    present_act = sum(1 for p in rows if p.get("actual_sha256"))
    exp_unique = side_unique("expected")
    act_unique = side_unique("actual")

    byname = {p["name"]: p for p in rows}
    order = {p["name"]: i for i, p in enumerate(rows)}

    def members(key: frozenset[str]) -> list[str]:
        """组内排序：主帧（无 duplicate_of 或指向组外）在前，其余按 manifest 22 名序。"""
        names = list(key)
        masters = [n for n in names
                   if not byname[n].get("duplicate_of")
                   or byname[n]["duplicate_of"] not in key]
        rest = [n for n in names if n not in masters]
        masters.sort(key=lambda n: order[n])
        rest.sort(key=lambda n: order[n])
        return masters + rest

    documented: list[dict] = []
    all_keys = list(dict.fromkeys(list(exp_groups) + list(act_groups)))
    for key in all_keys:
        names = members(key)
        in_exp, in_act = key in exp_groups, key in act_groups
        side = "expected+actual" if (in_exp and in_act) else ("expected" if in_exp else "actual")
        e = byname[names[0]].get("expected_sha256") or None
        a = byname[names[0]].get("actual_sha256") or None
        documented.append({
            "side": side,
            "names": names,
            "sha256": ({"expected": e, "actual": a} if side == "expected+actual"
                       else (e if side == "expected" else a)),
            "frames": [int(byname[n]["frame"]) for n in names],
            "reason": {
                "expected": "expected 同 blob（expected-invalid：复制品/四联旧件；待 §7.2 重采）",
                "actual": "actual 重采后同图（真实同画面）",
                "expected+actual": "expected 与 actual 两侧均同图（真实同画面/静态页）",
            }[side],
            "adjudicated_by": "P0-0 §7.5（task-155 B2 后口径）",
        })

    exp_reduction = sum(len(v["names"]) - 1 for v in exp_groups.values())
    act_reduction = sum(len(v["names"]) - 1 for v in act_groups.values())
    mapping_groups = _mapping_group_sets(rows)
    exp_dup_sets = [set(v["names"]) for v in exp_groups.values()]
    act_dup_sets = [set(v["names"]) for v in act_groups.values()]
    registry_ok = all(any(g <= mg for mg in mapping_groups) for g in exp_dup_sets + act_dup_sets)
    doc_sets = [set(d["names"]) for d in documented]
    documented_ok = all(any(g == s for s in doc_sets) for g in exp_dup_sets + act_dup_sets)
    equations_ok = (exp_unique == present_exp - exp_reduction
                    and act_unique == present_act - act_reduction)
    dedupe = {
        "expected_present": present_exp,
        "expected_unique": exp_unique,
        "expected_reduction": exp_reduction,
        "actual_present": present_act,
        "actual_unique": act_unique,
        "actual_reduction": act_reduction,
        "documented_duplicates": documented,
        "registry_ok": bool(registry_ok),
        "documented_ok": bool(documented_ok),
        "equations_ok": bool(equations_ok),
        "equations": {
            "expected": f"{exp_unique} == {present_exp} - {exp_reduction}",
            "actual": f"{act_unique} == {present_act} - {act_reduction}",
            "expected_22": f"{exp_unique} == 22 - {22 - present_exp} - {exp_reduction}",
            "actual_22": f"{act_unique} == 22 - {22 - present_act} - {act_reduction}",
            "blocked_not_counted": 22 - present_exp,
        },
    }
    dedupe_ok = bool(registry_ok and documented_ok and equations_ok)
    return dedupe, dedupe_ok


def validate_manifest(manifest: dict) -> list[str]:
    """对既有 manifest 做 schema/registry/两跑/计数校验（--validate 用）。"""
    issues: list[str] = []
    pairs = manifest.get("pairs")
    if not isinstance(pairs, list) or len(pairs) != 22:
        return [f"pairs 需 22 行，实际 {len(pairs) if isinstance(pairs, list) else 'N/A'}"]
    for p in pairs:
        name = p.get("name", "?")
        missing = [k for k in REQUIRED_PAIR_FIELDS if k not in p]
        if missing:
            issues.append(f"{name}: 缺字段 {missing}")
            continue
        if p["status"] not in ALLOWED_STATUS:
            issues.append(f"{name}: status 非法 {p['status']!r}")
        if p["reuse"] not in ALLOWED_REUSE:
            issues.append(f"{name}: reuse 非法 {p['reuse']!r}")
        if p["status"] == STATUS_BLOCKED:
            if p["expected_sha256"] or p["actual_sha256"] or p["deterministic"]:
                issues.append(f"{name}: blocked 行应 expected/actual 空、deterministic=false")
        else:
            if not p["expected_sha256"]:
                issues.append(f"{name}: expected_sha256 空（非 blocked）")
            if not p["actual_sha256"]:
                issues.append(f"{name}: actual_sha256 空（非 blocked）")
            if not p["deterministic"]:
                issues.append(f"{name}: deterministic=false")
            if p.get("render_env"):
                issues.append(f"{name}: render_env 非空（canonical 污染）")
            if not p.get("render_argv"):
                issues.append(f"{name}: render_argv 空")
    dedupe, dedupe_ok = build_dedupe(pairs)
    if not dedupe_ok:
        issues.append(f"dedupe 校验失败 {dedupe.get('equations')}")
    if manifest.get("dedupe") != dedupe:
        # 既有 manifest 的 dedupe 需与重算一致（允许 generated_at 等元数据不同）
        old = manifest.get("dedupe") or {}
        for k in ("expected_unique", "actual_unique", "registry_ok",
                  "documented_duplicates"):
            if old.get(k) != dedupe.get(k):
                issues.append(f"dedupe.{k} 与重算不符：{old.get(k)!r} != {dedupe.get(k)!r}")
    return issues


def write_mapping(rows: list[dict], path: pathlib.Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["\t".join(MAPPING_COLUMNS)]
    for r in rows:
        rec = dict(r)
        rec["expected_source"] = (f"{r['scene']}:{r['frame']}:{r['capture_tag']}"
                                  if r.get("scene") else "")
        lines.append("\t".join(str(rec.get(c, "")) for c in MAPPING_COLUMNS))
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def strict_from_args(args) -> dict | None:
    line = args.strict_line
    if line is None and args.strict_log:
        logp = pathlib.Path(args.strict_log)
        if not logp.is_file():
            raise SystemExit(f"BLOCKED: strict 日志缺 {logp}")
        line = parse_strict_log(logp.read_text(encoding="utf-8", errors="replace"))
    if line is None:
        return None
    return {"cmd": "python tools/diff_visual.py --strict", "exit": int(args.strict_exit),
            "line": line, "ran_after_asset_swap": bool(args.strict_after_asset_swap)}


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="L4 P0-0 manifest 生成 + 去重 registry 校验 + strict 行接口")
    ap.add_argument("--render-dir", default=str(ROOT / "target/p0-0/render"))
    ap.add_argument("--expected-dir", default=str(ROOT / "oracle/screens/expected"))
    ap.add_argument("--out", default=str(ROOT / "target/p0-0/render/g-manifest-full.json"))
    ap.add_argument("--mapping-out", default=str(ROOT / "analysis/repro/mapping.tsv"))
    ap.add_argument("--validate", metavar="MANIFEST", help="只校验既有 manifest JSON")
    ap.add_argument("--strict-line", default=None, help="STRICT 输出行（不执行 diff_visual）")
    ap.add_argument("--strict-log", default=None, help="含 STRICT 行的日志文件")
    ap.add_argument("--strict-exit", type=int, default=1)
    ap.add_argument("--strict-after-asset-swap", action="store_true", default=True)
    ap.add_argument("--json", action="store_true", help="stdout 输出 manifest JSON")
    args = ap.parse_args(argv)

    if args.validate:
        p = pathlib.Path(args.validate)
        if not p.is_file():
            print(f"BLOCKED: 缺 {p}", file=sys.stderr)
            return 2
        manifest = json.loads(p.read_text(encoding="utf-8"))
        issues = validate_manifest(manifest)
        if issues:
            print("MANIFEST-CHECK: FAIL")
            for i in issues:
                print(f"  - {i}")
            return 1
        print("MANIFEST-CHECK: PASS")
        summary = manifest.get("summary") or {}
        print(f"  pairs={len(manifest['pairs'])} "
              f"rendered={summary.get('rendered', '?')} "
              f"blocked={summary.get('blocked_no_source', '?')} "
              f"replaced={summary.get('replaced_invalid_asset', '?')}")
        print(f"  DEDUPE: expected {manifest['dedupe']['expected_unique']}="
              f"{manifest['dedupe']['expected_present']}-{manifest['dedupe']['expected_reduction']} "
              f"actual {manifest['dedupe']['actual_unique']}="
              f"{manifest['dedupe']['actual_present']}-{manifest['dedupe']['actual_reduction']} "
              f"registry_ok={manifest['dedupe']['registry_ok']}")
        return 0

    try:
        manifest = build_manifest(pathlib.Path(args.render_dir),
                                  pathlib.Path(args.expected_dir),
                                  strict=strict_from_args(args))
    except SystemExit as exc:
        print(str(exc), file=sys.stderr)
        return 2
    out = pathlib.Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
                   encoding="utf-8", newline="\n")
    write_mapping(PAIR_SPEC, pathlib.Path(args.mapping_out))
    if args.json:
        print(json.dumps(manifest, ensure_ascii=False, indent=2))
    s = manifest["summary"]
    d = manifest["dedupe"]
    print(f"MANIFEST: {out} pairs={s['pairs_total']} rendered={s['rendered']} "
          f"blocked={s['blocked_no_source']} replaced={s['replaced_invalid_asset']} "
          f"documented_duplicate={s['documented_duplicate']}")
    print(f"TWO-RUN: {'PASS' if manifest['validation']['two_run_ok'] else 'FAIL'} "
          f"({s['two_run_deterministic']}/{s['rendered']} png+idx)")
    print(f"DEDUPE: expected {d['expected_unique']}={d['expected_present']}-{d['expected_reduction']} "
          f"actual {d['actual_unique']}={d['actual_present']}-{d['actual_reduction']} "
          f"registry_ok={d['registry_ok']} documented_ok={d['documented_ok']} "
          f"-> {'PASS' if manifest['validation']['dedupe_ok'] else 'FAIL'}")
    if manifest["strict"]:
        print(f"STRICT: recorded exit={manifest['strict']['exit']} "
              f"ran_after_asset_swap={manifest['strict']['ran_after_asset_swap']} "
              f"line={manifest['strict']['line']!r}")
    else:
        print("STRICT: not recorded（--strict-line/--strict-log 未提供；本工具不执行 diff_visual）")
    print(f"MAPPING: {args.mapping_out}")
    if manifest["validation"]["pair_issues"]:
        print("PAIR-ISSUES:")
        for i in manifest["validation"]["pair_issues"]:
            print(f"  - {i}")
    print(f"VALIDATION: {'PASS' if manifest['validation']['ok'] else 'FAIL'}")
    return 0 if manifest["validation"]["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
