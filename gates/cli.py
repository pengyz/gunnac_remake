#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""gates/cli.py —— 门禁工具链单一真相源（AGENTS.md §5 门禁表的命令映射）。

硬性契约（gates/README.md「设计原则」）：
1. SKIP 与 TOLERATED 显式打印，绝不静默通过；
2. 基线 ratchet：data/status/baselines/baseline.json 只升不降，更新必须留说明；
3. 失败先定位层（AGENTS.md §4），不许调阈值/加白名单掩盖；
4. 白名单唯一例外 = gates/visual_whitelist.json（S6 建立，需用户确认）；
5. 无第二套扫描逻辑：git hooks 只是薄封装，一切扫描/裁定逻辑只存在于本文件。

用法：
    python3 gates/cli.py status          # 只读预检（动手前必跑）
    python3 gates/cli.py check          # 总闸（pre-commit 调用）
    python3 gates/cli.py rom|seed|naming|trace|visual   # 单门禁
    python3 gates/cli.py all            # 全量（= 所有门禁 + selftest + ratchet）
    python3 gates/cli.py baseline show|update|tolerate  # 基线管理（留说明机制）
    python3 gates/cli.py commit-msg <msgfile>           # commit-msg 钩子入口
    python3 gates/cli.py selftest       # 门禁工具链自证（ratchet/commit-msg/SKIP 打印）
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE_DIR = ROOT / "data" / "status" / "baselines"
BASELINE_PATH = BASELINE_DIR / "baseline.json"
HISTORY_PATH = BASELINE_DIR / "baseline_history.jsonl"

# ROM sha256 锚点：优先 rom/SHA256SUMS（若日后由资产归档者建立），
# 否则 data/status/baselines/SHA256SUMS（当前唯一锚点，sha256sum -c 兼容格式）。
SHA_ANCHORS = (
    Path("rom/SHA256SUMS"),
    Path("data/status/baselines/SHA256SUMS"),
)

# docs-links / knowledge-format 扫描排除目录（可再生工作区与第三方目录）。
EXCLUDE_DIRS = {
    ".git",
    ".agent-teams",
    "target",
    "node_modules",
    "analysis",
    "assets",
    "mesen2",
}

PASS = "PASS"
FAIL = "FAIL"
SKIP = "SKIP"
TOLERATED = "TOLERATED"

# baseline.json.metrics 的固定键集合（ratchet 计数器，均为"只增"型）。
METRIC_KEYS = (
    "rom_files_verified",
    "seed_projection_files",
    "naming_ledger_entries",
    "assets_manifest_entries",
    "knowledge_entries",
    "rust_test_count",
    "completeness_cells",
)

KNOWLEDGE_TYPES = {"architecture", "gotcha", "pattern", "debug", "decision", "reference"}

# metric > 0 即视为对应门禁的前置对象已建立（established）；此后前置对象消失不得静默回
# SKIP/PASS，必须 FAIL（t10 R4②『前置对象存在后不得静默消失』守卫）。
METRIC_TO_GATE = {
    "rom_files_verified": "rom-integrity",
    "seed_projection_files": "seed-integrity",
    "naming_ledger_entries": "naming-ledger",
    "assets_manifest_entries": "assets-manifest",
    "knowledge_entries": "knowledge-format",
    "rust_test_count": "rust-tests",
    "completeness_cells": "completeness-ratchet",
}


class RatchetError(Exception):
    """基线 ratchet 违规（尝试把基线降下去）。"""


class BaselineFormatError(Exception):
    """baseline.json 结构不合法。"""


# ---------------------------------------------------------------- 基础工具

def rel(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def parse_sha256sums(path: Path) -> dict[str, str]:
    """解析 sha256sum 格式：`<hex>  <path>`（兼容单空格与 `*` 二进制标记）。"""
    entries: dict[str, str] = {}
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        m = re.match(r"^([0-9a-fA-F]{64})\s+\*?(.+)$", line)
        if not m:
            raise ValueError(f"{rel(path)}:{lineno} 不是合法 sha256sum 行: {raw!r}")
        entries[m.group(2).strip()] = m.group(1).lower()
    return entries


def find_sha_anchor() -> Path | None:
    for candidate in SHA_ANCHORS:
        p = ROOT / candidate
        if p.is_file():
            return p
    return None


def find_sha_anchor_label() -> str:
    anchor = find_sha_anchor()
    return rel(anchor) if anchor else "（无锚点：rom/SHA256SUMS 与 data/status/baselines/SHA256SUMS 均缺失）"


def iter_markdown_files() -> list[Path]:
    out = []
    for p in sorted(ROOT.rglob("*.md")):
        if any(part in EXCLUDE_DIRS for part in p.relative_to(ROOT).parts[:-1]):
            continue
        out.append(p)
    return out


def parse_name_status(text: str) -> list[str]:
    """解析 `git diff --cached --name-status` 输出为触及路径集合。

    覆盖 A/M/D/T/C/R 全部状态：删除（D）与改名（R）的新旧两侧路径都算"触及"
    （t10 R4②：--diff-filter 排除 D 会让删敏感路径的提交免证据字段）。
    """
    paths: list[str] = []
    for line in text.splitlines():
        cols = line.rstrip("\n").split("\t")
        if len(cols) < 2:
            continue
        # A/M/D/T => [status, path]；R100/C75 => [status, old, new]
        paths.extend(p for p in cols[1:] if p)
    return paths


def git_staged_files() -> list[str]:
    """当前提交（暂存区）触及的文件。钩子裁定用；git 失败按拒绝处理（fail-closed）。"""
    proc = subprocess.run(
        ["git", "diff", "--cached", "--name-status", "-M"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"git diff --cached 失败（fail-closed）: {proc.stderr.strip()}")
    return parse_name_status(proc.stdout)


# ---------------------------------------------------------------- 基线（ratchet）

def load_baseline(path: Path | None = None) -> dict:
    p = path or BASELINE_PATH
    if not p.is_file():
        raise BaselineFormatError(f"{rel(p)} 缺失：基线不可用")
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise BaselineFormatError(f"{rel(p)} 解析失败（JSON 损坏，显式裁定不裸抛）: {exc}") from exc
    validate_baseline(data)
    return data


def validate_baseline(data: dict) -> None:
    if not isinstance(data, dict):
        raise BaselineFormatError("baseline.json 根必须是对象")
    metrics = data.get("metrics")
    if not isinstance(metrics, dict):
        raise BaselineFormatError("baseline.json 缺 metrics 对象")
    for key, value in metrics.items():
        if key not in METRIC_KEYS:
            raise BaselineFormatError(f"metrics 含未知键 {key!r}（固定键集：{METRIC_KEYS}）")
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            raise BaselineFormatError(f"metrics.{key} 必须是非负整数，实际 {value!r}")
    for key in ("assets", "seed"):
        if not isinstance(data.get(key), dict):
            raise BaselineFormatError(f"baseline.json 缺 {key} 对象")
    if not isinstance(data.get("tolerated"), list):
        raise BaselineFormatError("baseline.json 缺 tolerated 列表")
    for item in data["tolerated"]:
        if not isinstance(item, dict) or not item.get("gate") or not item.get("bug") or not item.get("reason"):
            raise BaselineFormatError(f"tolerated 条目必须含 gate/bug/reason（AGENTS.md §6）: {item!r}")
    established = data.setdefault("established", {})
    if not isinstance(established, dict):
        raise BaselineFormatError("baseline.json 的 established 必须是对象")
    for key, value in established.items():
        if key not in METRIC_TO_GATE.values() or value is not True:
            raise BaselineFormatError(f"established 含非法条目 {key!r}={value!r}")


def ratchet_compare(metric: str, old: int, new: int) -> None:
    """只升不降：new < old 即违规。等值/上调合法。"""
    if new < old:
        raise RatchetError(f"ratchet 违规：{metric} 基线 {old} 不允许降为 {new}（只升不降）")


def baseline_payload(data: dict) -> str:
    return json.dumps(data, ensure_ascii=False, indent=2, sort_keys=False) + "\n"


# 状态锚点文件（rom-integrity 同验；删/改状态文件即 FAIL —— t10 R1 / t14 F1）
STATE_ANCHOR_FILES = (
    "data/status/baselines/baseline.json",
    "data/status/baselines/baseline_history.jsonl",
)


def sync_state_anchors() -> None:
    """把状态文件（baseline.json + baseline_history.jsonl）的 sha256 写入 SHA256SUMS 锚点。

    t10 R1：baseline.json 手工改动可检出；t14 F1：删/改 baseline_history.jsonl 同样可检出。
    """
    anchor = BASELINE_DIR / "SHA256SUMS"
    lines = []
    if anchor.is_file():
        lines = [ln for ln in anchor.read_text(encoding="utf-8").splitlines() if ln.strip()]
    wanted = {}
    for relpath in STATE_ANCHOR_FILES:
        p = ROOT / relpath
        if p.is_file():
            wanted[relpath] = sha256_file(p)
    out: list[str] = []
    seen: set[str] = set()
    for ln in lines:
        m = re.match(r"^([0-9a-fA-F]{64})\s+\*?(.+)$", ln.strip())
        rpath = m.group(2).strip() if m else None
        if rpath in wanted:
            out.append(f"{wanted[rpath]}  {rpath}")
            seen.add(rpath)
        else:
            out.append(ln)
    for relpath in STATE_ANCHOR_FILES:
        if relpath in wanted and relpath not in seen:
            out.append(f"{wanted[relpath]}  {relpath}")
    anchor.write_text("\n".join(out) + "\n", encoding="utf-8", newline="\n")


def save_baseline(data: dict) -> str:
    """写 baseline.json 并同步 sha 锚点；返回 baseline.json 的 sha256（写入历史 sha 链）。"""
    validate_baseline(data)
    BASELINE_DIR.mkdir(parents=True, exist_ok=True)
    payload = baseline_payload(data)
    BASELINE_PATH.write_text(payload, encoding="utf-8", newline="\n")
    digest = hashlib.sha256(payload.encode("utf-8")).hexdigest()
    sync_state_anchors()
    return digest


def append_history(record: dict) -> None:
    BASELINE_DIR.mkdir(parents=True, exist_ok=True)
    with open(HISTORY_PATH, "a", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(record, ensure_ascii=False) + "\n")
    sync_state_anchors()


def history_records(path: Path | None = None) -> list[dict]:
    """读基线更新历史。**存在性前置校验（t14 F1）**：缺失/为空 ⇒ 抛错（下界不可验证，禁 vacuous 通过）。"""
    p = path or HISTORY_PATH
    if not p.is_file():
        raise BaselineFormatError(f"{rel(p)} 缺失：历史缺失，基线单调性下界不可验证")
    text = p.read_text(encoding="utf-8")
    if not [ln for ln in text.splitlines() if ln.strip()]:
        raise BaselineFormatError(f"{rel(p)} 为空：历史缺失，基线单调性下界不可验证")
    out = []
    for lineno, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        try:
            rec = json.loads(line)
        except json.JSONDecodeError as exc:
            raise BaselineFormatError(f"{rel(p)}:{lineno} 解析失败（JSON 损坏，显式裁定不裸抛）: {exc}") from exc
        if not isinstance(rec, dict) or not str(rec.get("reason", "")).strip():
            raise BaselineFormatError(f"{rel(p)}:{lineno} 缺 reason（基线更新必须留说明）")
        out.append(rec)
    return out


def history_lower_bounds(records: list[dict]) -> tuple[dict[str, int], set[str]]:
    """历史下界与覆盖集（t14 F1）。

    下界来源：kind=metric 记录的 new，以及任意记录携带的 metrics 快照（baseline_update/tolerate
    每条记录都带快照）。返回 (每 metric 下界, 有下界可依的 metric 集合)。
    """
    bounds: dict[str, int] = {}
    covered: set[str] = set()
    for rec in records:
        if rec.get("kind", "metric") == "metric" and rec.get("metric") in METRIC_KEYS:
            new = rec.get("new")
            if isinstance(new, int) and not isinstance(new, bool):
                covered.add(rec["metric"])
                bounds[rec["metric"]] = max(bounds.get(rec["metric"], 0), new)
        snap = rec.get("metrics")
        if isinstance(snap, dict):
            for key, val in snap.items():
                if key in METRIC_KEYS and isinstance(val, int) and not isinstance(val, bool):
                    covered.add(key)
                    bounds[key] = max(bounds.get(key, 0), val)
    return bounds, covered


def history_metric_last_new(records: list[dict]) -> dict[str, int]:
    """每 metric 的历史下界（兼容旧名；合法历史单调递增，故用 max 取下界）。"""
    return history_lower_bounds(records)[0]


def verify_baseline_integrity(baseline: dict, records: list[dict] | None = None,
                              baseline_path: Path | None = None) -> list[str]:
    """baseline.json ↔ baseline_history.jsonl 完整性校验（t10 R1 + t14 F1）。返回问题列表（空=完整）。

    检出手工绕过（非自洽/单侧改写机器不可见 = 不允许）：
    0. 历史缺失/为空/损坏 ⇒『历史缺失：基线单调性下界不可验证』（禁止 vacuous 通过）；
    0b. 任一 metric 无历史下界可依 ⇒ 同上（只留 genesis 记录的删改同样被拒）；
    1. 历史记录自身回退（new < old）；
    2. metric 现值低于历史下界 ⇒『基线被手工调低』；
    3. 最新历史记录的 baseline_sha256 与当前 baseline.json 字节 sha 不符 ⇒ sha 链断链。
    """
    problems: list[str] = []
    if records is None:
        try:
            records = history_records()
        except BaselineFormatError as exc:
            return [str(exc)]
    if not records:
        return ["历史缺失：基线单调性下界不可验证（history 为空）"]
    for i, rec in enumerate(records, 1):
        if rec.get("kind", "metric") == "metric" and rec.get("metric") in METRIC_KEYS:
            old, new = rec.get("old"), rec.get("new")
            if (isinstance(old, int) and not isinstance(old, bool)
                    and isinstance(new, int) and not isinstance(new, bool) and new < old):
                problems.append(f"历史第 {i} 条记录自身回退：{rec['metric']} {old}→{new}")
    bounds, covered = history_lower_bounds(records)
    for metric in METRIC_KEYS:
        if metric not in covered:
            problems.append(f"历史缺失：{metric} 基线单调性下界不可验证（历史无该 metric 的 new/快照）")
    for metric, floor in sorted(bounds.items()):
        cur = int(baseline["metrics"].get(metric, 0))
        if cur < floor:
            problems.append(f"『基线被手工调低』：{metric} 现值 {cur} 低于该 metric 历史最后一条记录的 new={floor}")
    chain = [r for r in records if r.get("baseline_sha256")]
    if chain:
        want = chain[-1]["baseline_sha256"]
        got = sha256_file(baseline_path or BASELINE_PATH)
        if want != got:
            problems.append(
                f"baseline.json 被手工改动（与历史 sha 链不符）：历史 {want[:12]}… 实际 {got[:12]}…"
            )
    return problems


def gate_established(baseline: dict, gate_name: str) -> bool:
    """前置对象已建立？建立后对象消失不得静默回 SKIP/PASS（t10 R4② 守卫）。"""
    return bool(baseline.get("established", {}).get(gate_name))


def missing_precondition(baseline: dict, gate_name: str, fallback_verdict: str, fallback_detail: str) -> tuple[str, str]:
    """前置对象缺失时的统一裁定：已建立 ⇒ FAIL『前置对象消失』；未建立 ⇒ 显式 fallback（SKIP/PASS）。"""
    if gate_established(baseline, gate_name):
        return FAIL, f"『前置对象消失』：{gate_name} 前置对象曾建立（基线对应 metric>0）现不存在，不得静默消失"
    return fallback_verdict, fallback_detail


def baseline_update(data: dict, metric: str, value: int, reason: str,
                    hist_floor: dict[str, int] | None = None) -> tuple[dict, dict]:
    """基线唯一写入口：只升不降 + 不低于历史 new + 强制说明 + 历史留痕 + established 登记。"""
    if metric not in METRIC_KEYS:
        raise BaselineFormatError(f"未知指标 {metric!r}（固定键集：{METRIC_KEYS}）")
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise ValueError(f"value 必须是非负整数，实际 {value!r}")
    reason = (reason or "").strip()
    if not reason:
        raise ValueError("基线更新必须 --reason 留说明（AGENTS.md §2：更新基线需写明『确认无回退』的证据）")
    if hist_floor is None:
        hist_floor = history_metric_last_new(history_records())
    old = int(data["metrics"].get(metric, 0))
    floor = int(hist_floor.get(metric, 0))
    ratchet_compare(metric, max(old, floor), value)
    data["metrics"][metric] = value
    if value > 0:
        data.setdefault("established", {})[METRIC_TO_GATE[metric]] = True
    record = {
        "ts": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "kind": "metric",
        "metric": metric,
        "old": old,
        "new": value,
        "reason": reason,
        "metrics": dict(data["metrics"]),  # 全量快照：任一 metric 的历史下界都可验证（t14 F1）
    }
    return data, record


def baseline_tolerate(data: dict, gate: str, item: str, bug: str, reason: str) -> tuple[dict, dict]:
    """登记显式 TOLERATED 已知偏差（AGENTS.md §6：必须注明 BUG 编号）。"""
    reason = (reason or "").strip()
    bug = (bug or "").strip()
    if not reason:
        raise ValueError("TOLERATED 登记必须 --reason 留说明")
    if not bug:
        raise ValueError("TOLERATED 登记必须 --bug 注明 BUG 编号（AGENTS.md §6）")
    entry = {"gate": gate, "item": item, "bug": bug, "reason": reason}
    data["tolerated"].append(entry)
    record = {
        "ts": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "kind": "tolerated",
        "gate": gate,
        "item": item,
        "bug": bug,
        "reason": reason,
        "metrics": dict(data["metrics"]),  # 全量快照（t14 F1）
    }
    return data, record


def current_metrics(baseline: dict) -> dict[str, int]:
    """现值采集（单一实现，status/check/all/自检共用）。来源缺失 = 0，显式打印。"""
    metrics = dict.fromkeys(METRIC_KEYS, 0)

    anchor = find_sha_anchor()
    if anchor:
        try:
            entries = parse_sha256sums(anchor)
        except ValueError:
            entries = {}
        ok = 0
        for rpath, want in entries.items():
            if not rpath.startswith("rom/"):
                continue
            p = ROOT / rpath
            if p.is_file() and sha256_file(p) == want:
                ok += 1
        metrics["rom_files_verified"] = ok

    seed_dir = ROOT / "reference" / "seed"
    if seed_dir.is_dir():
        metrics["seed_projection_files"] = sum(1 for p in seed_dir.rglob("*") if p.is_file())

    ledger = ROOT / "data" / "naming" / "ledger.jsonl"
    if ledger.is_file():
        metrics["naming_ledger_entries"] = sum(
            1 for line in ledger.read_text(encoding="utf-8").splitlines() if line.strip()
        )

    metrics["assets_manifest_entries"] = len(baseline.get("assets", {}))

    knowledge_dir = ROOT / "docs" / "knowledge"
    if knowledge_dir.is_dir():
        metrics["knowledge_entries"] = sum(
            1 for p in knowledge_dir.rglob("*.md") if p.name != "README.md"
        )

    crates = ROOT / "crates"
    if crates.is_dir():
        count = 0
        for p in crates.rglob("*.rs"):
            count += p.read_text(encoding="utf-8", errors="replace").count("#[test]")
        metrics["rust_test_count"] = count

    matrix_path = ROOT / "data" / "status" / "completeness_matrix.json"
    if matrix_path.is_file():
        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
        cells = matrix.get("cells", {})
        metrics["completeness_cells"] = sum(
            1 for v in cells.values() if isinstance(v, dict) and v.get("covered")
        )
    return metrics


def ratchet_rows(baseline: dict) -> list[tuple[str, int, int, str]]:
    """返回 (metric, 基线, 现值, 状态)。现值低于基线 = FAIL；高于基线 = PENDING 显式提示。"""
    measured = current_metrics(baseline)
    rows = []
    for key in METRIC_KEYS:
        old = int(baseline["metrics"].get(key, 0))
        new = int(measured[key])
        if new < old:
            state = "FAIL（现值低于基线，ratchet 违规）"
        elif new > old:
            state = "PENDING（现值高于基线：请 baseline update --reason 上调并在 commit 说明）"
        else:
            state = "OK"
        rows.append((key, old, new, state))
    return rows


# ---------------------------------------------------------------- 门禁实现
# 每个 runner 返回 (verdict, detail)。未达启用前置 ⇒ (SKIP, 显式原因)。

def gate_rom_integrity() -> tuple[str, str]:
    anchor = find_sha_anchor()
    if anchor is None:
        return FAIL, "SHA256SUMS 锚点缺失（rom/SHA256SUMS 或 data/status/baselines/SHA256SUMS）"
    try:
        entries = parse_sha256sums(anchor)
    except ValueError as exc:
        return FAIL, str(exc)
    if not entries:
        return FAIL, f"{rel(anchor)} 无任何登记条目"
    problems = []
    if "rom/gunnac_cn.nes" not in entries:
        problems.append("rom/gunnac_cn.nes 未登记于 SHA256SUMS")
    if "data/status/baselines/baseline.json" not in entries:
        problems.append("data/status/baselines/baseline.json 未纳入 sha 锚点管理（t10 R1）")
    if "data/status/baselines/baseline_history.jsonl" not in entries:
        problems.append("data/status/baselines/baseline_history.jsonl 未纳入 sha 锚点管理（t14 F1）")
    for rpath, want in sorted(entries.items()):
        p = ROOT / rpath
        if not p.is_file():
            problems.append(f"{rpath}: 文件缺失")
            continue
        got = sha256_file(p)
        if got != want:
            extra = "【rom/ 任何字节改动=严重违规】" if rpath.startswith("rom/") else "【基线文件被手工改动】"
            problems.append(f"{rpath}: sha256 不符（锚点 {want[:12]}… 实际 {got[:12]}…）{extra}")
    if problems:
        return FAIL, "; ".join(problems)
    return PASS, (f"{len(entries)} 个锚点文件 sha256 一致（rom/gunnac_cn.nes={entries['rom/gunnac_cn.nes'][:16]}…"
                  f"；baseline.json 纳入 sha 锚点管理）")


def gate_addr_bridge() -> tuple[str, str]:
    """坐标桥一致性 v2（对抗复核 #56 AR56-1 加固）：
    meta sha 双锚定 + 条目数锚定（≡diff runs）+ 字段校验（bytes/delta）+ relocated **全量**逐字节复验。
    v1 抽样 8/54 可被'改未抽样条目/删条目/改 delta'穿透（#56 攻击实测 5 攻 4 过）——v2 逐项封死。"""
    bridge = ROOT / "data" / "reference" / "addr-bridge-jp-cn.json"
    jp_p = ROOT / "reference" / "jp" / "gunnac_jp.nes"
    cn_p = ROOT / "rom" / "gunnac_cn.nes"
    if not bridge.is_file() or not jp_p.is_file() or not cn_p.is_file():
        return missing_precondition(load_baseline(), "addr-bridge", SKIP,
                                    "坐标桥或双 ROM 缺失（条件 C：JP 参照锚到位后启用）")
    try:
        data = json.loads(bridge.read_text())
    except ValueError as exc:
        return FAIL, f"addr-bridge JSON 解析失败：{exc}"
    problems = []
    jp = jp_p.read_bytes()
    cn = cn_p.read_bytes()
    if data.get("meta", {}).get("jp_sha256") != sha256_file(jp_p):
        problems.append("桥 meta jp_sha256 与 reference/jp/gunnac_jp.nes 不符（桥已过期，重生成）")
    if data.get("meta", {}).get("cn_sha256") != sha256_file(cn_p):
        problems.append("桥 meta cn_sha256 与 rom/gunnac_cn.nes 不符【rom/ 任何字节改动=严重违规】")
    n_relocated = 0
    for bi in range(16):
        base = 16 + bi * 0x2000
        # 重算 diff runs（与 gen_addr_bridge 同口径）
        runs, in_run = [], False
        for k in range(0x2000):
            d = jp[base + k] != cn[base + k]
            if d and not in_run:
                st = k
                in_run = True
            elif not d and in_run:
                runs.append((st, k))
                in_run = False
        if in_run:
            runs.append((st, 0x2000))
        entries = data.get("banks", {}).get(f"bank{bi:02d}", [])
        if len(runs) != len(entries):
            problems.append(f"bank{bi:02d}: 条目数 {len(entries)} ≠ diff runs {len(runs)}（条目被删/改）")
            continue
        for (st, e), ent in zip(runs, entries):
            # 字段校验：cn 区间与字节数
            try:
                cst, cen = (int(x, 16) for x in ent["cn"].split("-"))
            except (KeyError, ValueError):
                problems.append(f"bank{bi:02d}:{ent} cn 字段不可解析")
                continue
            if (cst, cen) != (st, e):
                problems.append(f"bank{bi:02d}: cn 区间 {ent['cn']} ≠ 实测 {st:04X}-{e:04X}")
            if ent.get("bytes") != (e - st):
                problems.append(f"bank{bi:02d}:{ent['cn']} bytes 字段 {ent.get('bytes')} ≠ {e - st}")
            kind = ent.get("kind")
            if kind in ("relocated", "relocated-cross-bank"):
                n_relocated += 1
                try:
                    jst = int(ent["jp"].split("-")[0], 16)
                except (KeyError, ValueError):
                    problems.append(f"bank{bi:02d}:{ent['cn']} jp 字段不可解析")
                    continue
                jb = int(ent.get("jp_bank", bi))
                if kind == "relocated" and ent.get("delta") != jst - cst:
                    problems.append(f"bank{bi:02d}:{ent['cn']} delta {ent.get('delta')} ≠ {jst - cst}")
                seg_c = cn[base + cst : base + cen]
                seg_j = jp[16 + jb * 0x2000 + jst : 16 + jb * 0x2000 + jst + (e - st)]
                if seg_c != seg_j:
                    problems.append(f"bank{bi:02d}:{ent['cn']}→{ent.get('jp')} 字节不符（映射失效/被改）")
            elif kind == "same-site":
                if ent.get("delta") != 0:
                    problems.append(f"bank{bi:02d}:{ent['cn']} same-site delta ≠ 0")
            elif kind != "unmapped":
                problems.append(f"bank{bi:02d}:{ent['cn']} kind 异常 {kind}")
    if problems:
        return FAIL, "; ".join(problems[:5])
    return PASS, (f"坐标桥 v2 一致（relocated {n_relocated} 全量逐字节复验；16 bank 条目数锚定；"
                  f"bytes/delta 字段校验；meta sha 双锚定；禁手工换算）")


def gate_seed_integrity() -> tuple[str, str]:
    seed_dir = ROOT / "reference" / "seed"
    if not seed_dir.is_dir():
        return missing_precondition(load_baseline(), "seed-integrity", SKIP,
                                    "reference/seed/ 不存在（S1 交付后启用；D3 冻结基准）")
    baseline = load_baseline()
    frozen = baseline.get("seed", {})
    if not frozen:
        return FAIL, "reference/seed/ 已存在但 baseline.json 的 seed 段为空：投影基准未冻结（D3：先冻结再改动）"
    problems = []
    actual = {p.relative_to(ROOT).as_posix(): p for p in seed_dir.rglob("*") if p.is_file()}
    for rpath, meta in sorted(frozen.items()):
        p = actual.get(rpath)
        if p is None:
            problems.append(f"{rpath}: 基线登记的投影文件缺失")
            continue
        got = sha256_file(p)
        if meta.get("sha256") != got:
            problems.append(f"{rpath}: sha256 与冻结基准不符")
        if meta.get("lines") != sum(1 for _ in open(p, encoding="utf-8", errors="replace")):
            problems.append(f"{rpath}: 行数与冻结基准不符")
    for rpath in sorted(set(actual) - set(frozen)):
        problems.append(f"{rpath}: 未登记于冻结基准（新增投影需 baseline 更新 + commit 说明）")
    if problems:
        return FAIL, "; ".join(problems)
    detail = f"{len(frozen)} 个投影文件 sha+行数 与冻结基准一致"
    if (ROOT / "reference" / "src").is_dir():
        detail += "; reference/src/ 轨道 trace 比对：SKIP（oracle trace 未建立，显式跳过）"
    return PASS, detail


def gate_assets_manifest() -> tuple[str, str]:
    baseline = load_baseline()
    assets = baseline.get("assets", {})
    if not assets:
        return PASS, "0 条目（CHR/PRG 资产清单为空：S1 提取后经 baseline 机制登记；ratchet 骨架已生效）"
    problems = []
    for rpath, meta in sorted(assets.items()):
        p = ROOT / rpath
        if not p.is_file():
            problems.append(f"{rpath}: 资产文件缺失")
            continue
        if meta.get("sha256") != sha256_file(p):
            problems.append(f"{rpath}: sha256 与清单不符")
        if "size" in meta and meta["size"] != p.stat().st_size:
            problems.append(f"{rpath}: size 与清单不符")
    if problems:
        return FAIL, "; ".join(problems)
    return PASS, f"{len(assets)} 条资产 sha256/size 与 baseline.json assets 一致"


def gate_naming_ledger() -> tuple[str, str]:
    ledger = ROOT / "data" / "naming" / "ledger.jsonl"
    if not ledger.is_file():
        return missing_precondition(load_baseline(), "naming-ledger", SKIP,
                                    "data/naming/ledger.jsonl 不存在（台账创建后自动启用）")
    lines = [line for line in ledger.read_text(encoding="utf-8").splitlines() if line.strip()]
    if not lines:
        return SKIP, "台账为空（0 条记录，显式 SKIP；出现 rename 记录后自动启用）"
    problems: list[str] = []
    warnings: list[str] = []
    for i, line in enumerate(lines, 1):
        try:
            rec = json.loads(line)
        except json.JSONDecodeError as exc:
            problems.append(f"第{i}行 JSON 解析失败: {exc}")
            continue
        if not isinstance(rec, dict):
            problems.append(f"第{i}行不是 JSON 对象")
            continue
        evidence = rec.get("evidence", rec.get("证据"))
        if not isinstance(evidence, str) or not evidence.strip():
            problems.append(f"第{i}行缺 evidence（每条 rename 必须有可复现指针，AGENTS.md §2/§3）")
        elif not re.search(r"(0x[0-9A-Fa-f]+|bank\d+|[:：#/]|\.\w)", evidence):
            problems.append(f"第{i}行 evidence 不含可复现指针: {evidence!r}")
        confidence = rec.get("confidence", rec.get("置信"))
        if confidence is None:
            warnings.append(f"第{i}行缺 confidence（建议与 evidence 配套记录 0-100）")
        elif isinstance(confidence, bool) or not isinstance(confidence, (int, float)) or not 0 <= confidence <= 100:
            problems.append(f"第{i}行 confidence 非 0-100 数值: {confidence!r}")
    detail = f"{len(lines)} 条记录逐条校验证据指针"
    if warnings:
        detail += "；WARN（显式）: " + "; ".join(warnings)
    if problems:
        return FAIL, "; ".join(problems)
    return PASS, detail


def gate_knowledge_format() -> tuple[str, str]:
    knowledge_dir = ROOT / "docs" / "knowledge"
    if not knowledge_dir.is_dir():
        return missing_precondition(load_baseline(), "knowledge-format", PASS,
                                    "0 检对象（docs/knowledge/ 尚不存在，建立后自动开始校验；显式打印非静默）")
    entries = [p for p in sorted(knowledge_dir.rglob("*.md")) if p.name != "README.md"]
    if not entries:
        return PASS, "0 检对象（docs/knowledge/ 为空）"
    problems = []
    for p in entries:
        text = p.read_text(encoding="utf-8", errors="replace")
        m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
        if not m:
            problems.append(f"{rel(p)}: 缺 frontmatter")
            continue
        tm = re.search(r"^type:\s*([A-Za-z_-]+)\s*$", m.group(1), re.M)
        if not tm:
            problems.append(f"{rel(p)}: frontmatter 缺 type")
            continue
        ktype = tm.group(1)
        if ktype not in KNOWLEDGE_TYPES:
            problems.append(f"{rel(p)}: type={ktype!r} 不在 {sorted(KNOWLEDGE_TYPES)}")
        elif not p.name.startswith(f"{ktype}-"):
            problems.append(f"{rel(p)}: 文件名须以 '{ktype}-' 前缀（写入规则见 docs/knowledge/README.md）")
    if problems:
        return FAIL, "; ".join(problems)
    return PASS, f"{len(entries)} 条知识条目 frontmatter/type/文件名前缀合规"


MD_LINK_RE = re.compile(r"\[[^\]]*\]\(([^)]+)\)")


def gate_docs_links() -> tuple[str, str]:
    problems = []
    n_files = 0
    n_links = 0
    for md in iter_markdown_files():
        n_files += 1
        text = md.read_text(encoding="utf-8", errors="replace")
        for m in MD_LINK_RE.finditer(text):
            target = m.group(1).strip()
            if target.startswith(("http://", "https://", "#", "mailto:")):
                continue
            target = target.split("#")[0]
            if not target:
                continue
            n_links += 1
            resolved = (md.parent / target).resolve()
            if not resolved.exists():
                problems.append(f"{rel(md)} → {m.group(1)}（目标不存在）")
    if problems:
        shown = "; ".join(problems[:10])
        if len(problems) > 10:
            shown += f"；…共 {len(problems)} 条"
        return FAIL, shown
    return PASS, f"{n_files} 个 md / {n_links} 条本地链接全部有效"


def gate_rust_tests() -> tuple[str, str]:
    if not (ROOT / "crates").is_dir() or not (ROOT / "Cargo.toml").is_file():
        return SKIP, "crates/ 尚未创建（创建后自动启用）"
    proc = subprocess.run(
        ["cargo", "test", "--workspace"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        tail = (proc.stdout + proc.stderr).strip().splitlines()[-5:]
        return FAIL, "cargo test --workspace 失败: " + " | ".join(tail)
    baseline = load_baseline()
    measured = current_metrics(baseline)["rust_test_count"]
    floor = int(baseline["metrics"].get("rust_test_count", 0))
    if measured < floor:
        return FAIL, f"测试数 ratchet 违规：现值 {measured} < 基线 {floor}"
    return PASS, f"cargo test --workspace 通过；测试数 {measured}（基线 {floor}）"


def _gate_runner_dispatch(runner_name: str, prereq: Path, label: str) -> tuple[str, str]:
    """差分三兄弟（L2/L3/L4）的共用骨架：单一 runner（tools/<name>.py），无第二套扫描逻辑。"""
    runner = ROOT / "tools" / runner_name
    if not prereq.exists() or not runner.is_file():
        missing = []
        if not prereq.exists():
            missing.append(f"{rel(prereq)} 未建立")
        if not runner.is_file():
            missing.append(f"tools/{runner_name} 未建立")
        return SKIP, f"{label}：{'；'.join(missing)}（启用前置齐备后自动启用，显式 SKIP）"
    proc = subprocess.run([sys.executable, str(runner)], cwd=ROOT, capture_output=True, text=True)
    if proc.returncode != 0:
        tail = (proc.stdout + proc.stderr).strip().splitlines()[-5:]
        return FAIL, f"{label} 差分不一致: " + " | ".join(tail)
    return PASS, f"{label} 全绿（tools/{runner_name}）"


def gate_subroutine_diff() -> tuple[str, str]:
    return _gate_runner_dispatch("diff_subroutines.py", ROOT / "oracle" / "subroutines", "L2 子程序级")


def gate_trace_diff() -> tuple[str, str]:
    return _gate_runner_dispatch("diff_traces.py", ROOT / "oracle" / "traces", "L3 帧级 trace")


def gate_visual_diff() -> tuple[str, str]:
    verdict, detail = _gate_runner_dispatch("diff_visual.py", ROOT / "oracle" / "screens", "L4 视觉")
    if verdict == SKIP:
        detail += "；visual 白名单 gates/visual_whitelist.json 于 S6 建立（唯一允许的白名单，变更需用户确认）"
    return verdict, detail


def gate_completeness_ratchet() -> tuple[str, str]:
    matrix_path = ROOT / "data" / "status" / "completeness_matrix.json"
    if not matrix_path.is_file():
        return SKIP, "完备性矩阵未建立（data/status/completeness_matrix.json；矩阵建立后自动启用）"
    baseline = load_baseline()
    measured = current_metrics(baseline)["completeness_cells"]
    floor = int(baseline["metrics"].get("completeness_cells", 0))
    if measured < floor:
        return FAIL, f"完备性覆盖 ratchet 违规：已覆盖格 {measured} < 基线 {floor}"
    return PASS, f"完备性已覆盖格 {measured}（基线 {floor}，只增不降）"


# ---------------------------------------------------------------- 门禁注册表
# readme_state 必须与 gates/README.md「门禁清单」表的"状态"列逐字一致（selftest 自动比对）。

GATES: list[tuple[str, str, str, str, object]] = [
    # (name, readme_state, 所属命令, 一句话内容, runner)
    ("rom-integrity", "启用（P0）", "rom", "rom/ sha256 对照 SHA256SUMS", gate_rom_integrity),
    ("seed-integrity", "S1 交付后启用", "seed", "reference/seed/ 反汇编投影 sha+行数对照 baseline", gate_seed_integrity),
    ("assets-manifest", "启用", "check", "CHR/PRG 资产清单 ratchet（对照 baseline.json）", gate_assets_manifest),
    ("naming-ledger", "台账创建后自动启用", "naming", "每条 rename 带 evidence 可复现指针", gate_naming_ledger),
    ("knowledge-format", "启用", "check", "知识库 frontmatter/type/文件名前缀", gate_knowledge_format),
    ("docs-links", "启用", "check", "文档相对链接有效性", gate_docs_links),
    ("rust-tests", "crates/ 创建后自动启用", "check", "cargo test --workspace + 测试数 ratchet", gate_rust_tests),
    ("subroutine-diff", "oracle/subroutines 建立后启用", "all", "L2：ROM 原字节执行 vs Rust 函数出入参比对", gate_subroutine_diff),
    ("trace-diff", "S6 启用", "trace", "L3：帧级状态向量逐 tick 比对（oracle vs Rust）", gate_trace_diff),
    ("visual-diff", "S6 启用", "visual", "L4：截图逐像素（白名单外）", gate_visual_diff),
    ("completeness-ratchet", "矩阵建立后启用", "all", "完备性矩阵覆盖只增不降", gate_completeness_ratchet),
    ("addr-bridge", "启用", "check", "九班↔JP 坐标桥 meta sha + relocated 抽样复验", gate_addr_bridge),
]


def run_gate(entry) -> dict:
    name, readme_state, command, summary, runner = entry
    try:
        verdict, detail = runner()
    except (RatchetError, BaselineFormatError, ValueError) as exc:
        verdict, detail = FAIL, f"{type(exc).__name__}: {exc}"
    return {
        "name": name,
        "readme_state": readme_state,
        "command": command,
        "summary": summary,
        "verdict": verdict,
        "detail": detail,
    }


def run_all_gates() -> list[dict]:
    return [run_gate(entry) for entry in GATES]


def format_gate_lines(results: list[dict]) -> list[str]:
    lines = []
    for r in results:
        state = "SKIP" if r["verdict"] == SKIP else "ENABLED"
        lines.append(f"[{r['verdict']:<9}] {r['name']:<20} ({state}, README: {r['readme_state']}) {r['detail']}")
    return lines


def tolerated_lines(baseline: dict) -> list[str]:
    items = baseline.get("tolerated", [])
    if not items:
        return ["[TOLERATED ] （无）"]
    out = []
    for item in items:
        out.append(
            f"[TOLERATED ] {item['gate']} / {item['item']} —— {item['bug']}: {item['reason']}（显式登记，绝不静默）"
        )
    return out


def summarize(results: list[dict], baseline: dict) -> str:
    counts = {PASS: 0, FAIL: 0, SKIP: 0, TOLERATED: 0}
    for r in results:
        counts[r["verdict"]] = counts.get(r["verdict"], 0) + 1
    counts[TOLERATED] = len(baseline.get("tolerated", []))
    return (
        f"SUMMARY: PASS={counts[PASS]} FAIL={counts[FAIL]} "
        f"SKIP={counts[SKIP]} TOLERATED={counts[TOLERATED]}"
    )


# ---------------------------------------------------------------- commit-msg 证据门禁

# 别名集合与 gates/README.md 承诺严格一致（t10 R4①）：Evidence:/证据: 与 Confidence:/置信度:
# —— 不接受未文档化别名（如『信心:』/『置信:』）。
EVIDENCE_RE = re.compile(r"^(?:Evidence|证据)\s*[:：]", re.I | re.M)
CONFIDENCE_RE = re.compile(r"^(?:Confidence|置信度)\s*[:：]", re.I | re.M)
# 敏感路径（触及即要求 Evidence/Confidence）。data/status/ 纳入 = t14 F1 攻击链"降基线无痕
# 提交"封口：baseline 更新本就有 --reason 证据，提交摩擦可接受（captain 裁定 + t15 复审共同确认）。
SENSITIVE_PREFIXES = ("data/naming/", "data/status/", "data/reference/", "reference/", "rom/")
SENSITIVE_TEXT = "、".join(SENSITIVE_PREFIXES)  # 由集合派生，无第二份文案（t10 R3）


def touched_sensitive(paths: list[str]) -> list[str]:
    hits = []
    for p in paths:
        norm = p.replace("\\", "/").lstrip("./")
        for prefix in SENSITIVE_PREFIXES:
            if norm == prefix.rstrip("/") or norm.startswith(prefix):
                hits.append(norm)
                break
    return hits


def commit_msg_check(message: str, touched: list[str]) -> tuple[bool, str]:
    """触及 data/naming/、reference/、rom/ 的提交必须含 Evidence/Confidence 字段（AGENTS.md §9）。

    删除（D）与改名（R）的路径同样计为"触及"（t10 R4②）。
    """
    hits = touched_sensitive(touched)
    body = "\n".join(ln for ln in message.splitlines() if not ln.lstrip().startswith("#"))
    if not hits:
        return True, f"未触及 {SENSITIVE_TEXT}（触及文件 {len(touched)} 个），无需 Evidence/Confidence 字段"
    missing = []
    if not EVIDENCE_RE.search(body):
        missing.append("Evidence")
    if not CONFIDENCE_RE.search(body):
        missing.append("Confidence")
    if missing:
        return False, (
            f"拒绝 commit：触及 {', '.join(sorted(set(hits)))} 的提交必须含 Evidence/Confidence 字段"
            f"（AGENTS.md §9），缺 {', '.join(missing)}"
        )
    return True, f"证据门禁通过：触及 {', '.join(sorted(set(hits)))}，Evidence/Confidence 字段齐备"


# ---------------------------------------------------------------- 命令实现

def integrity_lines(problems: list[str]) -> list[str]:
    if problems:
        return [f"[FAIL     ] baseline-integrity: {p}" for p in problems]
    return ["[OK       ] baseline-integrity: 历史存在且非空；全部 metric 有历史下界可依；sha 链与状态锚点一致（非自洽/单侧改写可检出）"]


def baseline_section() -> tuple[dict | None, list[str], list[tuple[str, int, int, str]], str | None]:
    """基线段统一装配：解析/校验错误一律转显式问题（t14 F2：禁止裸 traceback）。"""
    try:
        baseline = load_baseline()
        integrity = verify_baseline_integrity(baseline)
        rows = ratchet_rows(baseline)
        return baseline, integrity, rows, None
    except (BaselineFormatError, ValueError) as exc:
        return None, [f"{exc}"], [], None


def cmd_status(_args) -> int:
    results = run_all_gates()
    baseline, integrity, rows, parse_error = baseline_section()
    parse_error = parse_error or (None if baseline else "baseline 解析失败，见上方 baseline-integrity 行")
    print("== gates/cli.py status —— 只读预检（SKIP/TOLERATED 显式打印，绝不静默通过） ==")
    print(f"门禁清单 {len(GATES)} 项（与 gates/README.md 对齐）：")
    for line in format_gate_lines(results):
        print(line)
    print("-- 基线对照（data/status/baselines/baseline.json，只升不降） --")
    print(f"{'metric':<26} {'基线':>6} {'现值':>6}  状态")
    ratchet_ok = True
    for key, old, new, state in rows:
        if state.startswith("FAIL"):
            ratchet_ok = False
        print(f"{key:<26} {old:>6} {new:>6}  {state}")
    for line in integrity_lines(integrity):
        print(line)
    if baseline is not None:
        for line in tolerated_lines(baseline):
            print(line)
    print(summarize(results, baseline or {"tolerated": []}))
    any_fail = (any(r["verdict"] == FAIL for r in results) or not ratchet_ok or bool(integrity)
                or baseline is None)
    print(f"STATUS: {'FAIL（见上方 FAIL 行）' if any_fail else 'OK（可动手；先看清 SKIP 项）'}")
    return 1 if any_fail else 0


def cmd_check(_args) -> int:
    results = run_all_gates()
    baseline, integrity, rows, parse_error = baseline_section()
    parse_error = parse_error or (None if baseline else "baseline 解析失败，见上方 baseline-integrity 行")
    print("== gates/cli.py check —— 总闸（聚合全部门禁；SKIP/TOLERATED 显式打印） ==")
    for line in format_gate_lines(results):
        print(line)
    ratchet_ok = True
    for key, old, new, state in rows:
        verdict = "FAIL" if state.startswith("FAIL") else ("WARN" if state.startswith("PENDING") else "OK")
        if verdict == "FAIL":
            ratchet_ok = False
        print(f"[{verdict:<9}] ratchet:{key} 基线 {old} / 现值 {new} —— {state}")
    for line in integrity_lines(integrity):
        print(line)
    if baseline is not None:
        for line in tolerated_lines(baseline):
            print(line)
    print(summarize(results, baseline or {"tolerated": []}))
    failed = [r["name"] for r in results if r["verdict"] == FAIL]
    if failed or not ratchet_ok or integrity or baseline is None:
        print(f"CHECK: FAIL —— {'; '.join(failed) if failed else ''}"
              f"{'' if ratchet_ok else ' [ratchet]'}"
              f"{'' if not integrity else ' [baseline-integrity: 历史缺失/基线被手工改动/调低，非自洽/单侧改写绕过已被检出]'}。"
              f"按 AGENTS.md §4 定位层后修复根因，禁止调阈值/加白名单掩盖。")
        return 1
    print("CHECK: PASS（含 0 个静默跳过：所有 SKIP 均已显式打印）")
    return 0


def cmd_single(invoked: str, gate_name: str):
    def run(_args) -> int:
        entry = next(e for e in GATES if e[0] == gate_name)
        result = run_gate(entry)
        print(f"== gates/cli.py {invoked} —— 单门禁 {gate_name} ==")
        for line in format_gate_lines([result]):
            print(line)
        return 1 if result["verdict"] == FAIL else 0
    return run


def cmd_all(_args) -> int:
    rc = cmd_check(_args)
    print("-- selftest（门禁工具链自证） --")
    rc_self = cmd_selftest(_args)
    return 1 if (rc or rc_self) else 0


def cmd_baseline(args) -> int:
    baseline = load_baseline()
    if args.baseline_action == "show":
        print(json.dumps(baseline, ensure_ascii=False, indent=2))
        for line in integrity_lines(verify_baseline_integrity(baseline)):
            print(line)
        print(f"-- 历史（{rel(HISTORY_PATH)}，每条更新必须留说明） --")
        for rec in history_records():
            print(json.dumps(rec, ensure_ascii=False))
        return 0
    if args.baseline_action == "update":
        try:
            prior_problems = verify_baseline_integrity(baseline)
            baseline, record = baseline_update(baseline, args.metric, args.value, args.reason)
        except (RatchetError, BaselineFormatError, ValueError) as exc:
            print(f"BASELINE UPDATE 拒绝: {exc}")
            return 1
        record["baseline_sha256"] = save_baseline(baseline)
        if prior_problems:
            record["tamper_recovered"] = prior_problems
        append_history(record)
        print(f"BASELINE UPDATE: {record['metric']} {record['old']} -> {record['new']}")
        if prior_problems:
            print("【留痕】更新前基线处于污染状态（本次更新记为 tamper_recovered，历史永久可见）：")
            for p in prior_problems:
                print(f"  - {p}")
        print(f"说明已留痕 {rel(HISTORY_PATH)}；commit message 必须写明『确认无回退』的证据（AGENTS.md §2）")
        return 0
    if args.baseline_action == "tolerate":
        try:
            baseline, record = baseline_tolerate(baseline, args.gate, args.item, args.bug, args.reason)
        except (RatchetError, BaselineFormatError, ValueError) as exc:
            print(f"BASELINE TOLERATE 拒绝: {exc}")
            return 1
        record["baseline_sha256"] = save_baseline(baseline)
        append_history(record)
        print(f"BASELINE TOLERATE 已登记（显式打印，绝不静默）: {json.dumps(record, ensure_ascii=False)}")
        return 0
    print(f"未知 baseline 子命令: {args.baseline_action}")
    return 2


def cmd_commit_msg(args) -> int:
    msg_path = Path(args.msgfile)
    if not msg_path.is_file():
        print(f"commit-msg: 消息文件不存在: {msg_path}")
        return 1
    message = msg_path.read_text(encoding="utf-8")
    try:
        touched = git_staged_files()
    except RuntimeError as exc:
        print(f"commit-msg 拒绝（fail-closed）: {exc}")
        return 1
    ok, detail = commit_msg_check(message, touched)
    print(f"commit-msg: {detail}")
    return 0 if ok else 1


# ---------------------------------------------------------------- selftest（自证）

def cmd_selftest(_args) -> int:
    cases: list[tuple[str, bool, str]] = []

    def check(name: str, ok: bool, detail: str = "") -> None:
        cases.append((name, ok, detail))

    # 1) ratchet：只升不降
    try:
        ratchet_compare("rust_test_count", 5, 4)
        check("ratchet 拒绝下降", False, "未抛 RatchetError")
    except RatchetError:
        check("ratchet 拒绝下降", True, "下降被 RatchetError 拒绝")
    try:
        ratchet_compare("rust_test_count", 5, 5)
        ratchet_compare("rust_test_count", 5, 6)
        check("ratchet 允许持平/上调", True, "5→5、5→6 合法")
    except RatchetError as exc:
        check("ratchet 允许持平/上调", False, str(exc))

    # 2) 基线更新：强制说明 + 只升不降 + 不低于历史 new + established 自动登记
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        data = {"schema": 1, "metrics": dict.fromkeys(METRIC_KEYS, 0), "assets": {}, "seed": {},
                "tolerated": [], "established": {}}
        try:
            baseline_update(data, "rom_files_verified", 2, "", hist_floor={})
            check("基线更新拒绝空说明", False, "空 reason 被接受")
        except ValueError:
            check("基线更新拒绝空说明", True, "空 reason 被拒绝")
        data["metrics"]["rom_files_verified"] = 5
        try:
            baseline_update(data, "rom_files_verified", 4, "试图下调", hist_floor={})
            check("基线更新拒绝下调", False, "5→4 下调被接受")
        except RatchetError:
            check("基线更新拒绝下调", True, "5→4 下调被 RatchetError 拒绝")
        try:
            baseline_update(data, "rust_test_count", 3, "试图低于历史 new", hist_floor={"rust_test_count": 5})
            check("基线更新拒绝低于历史 new", False, "历史 new=5 时 value=3 被接受")
        except RatchetError:
            check("基线更新拒绝低于历史 new", True, "历史 new=5 时 value=3 被 RatchetError 拒绝")
        data["metrics"]["rom_files_verified"] = 0
        data, record = baseline_update(data, "rom_files_verified", 2, "M0 自证：ROM 校验计数上调", hist_floor={})
        (tmp / "baseline.json").write_text(baseline_payload(data), encoding="utf-8")
        hist = tmp / "history.jsonl"
        with open(hist, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, ensure_ascii=False) + "\n")
        rec = json.loads(hist.read_text(encoding="utf-8").splitlines()[0])
        check(
            "基线更新留说明机制",
            rec.get("reason") == "M0 自证：ROM 校验计数上调" and rec.get("old") == 0 and rec.get("new") == 2,
            f"history 记录: {json.dumps(rec, ensure_ascii=False)}",
        )
        check("metric>0 自动登记 established", data.get("established", {}).get("rom-integrity") is True,
              f"established={data.get('established')}")
        try:
            baseline_tolerate(data, "docs-links", "docs/x.md", "", "无 bug 编号")
            check("TOLERATED 必须带 BUG 编号", False, "空 bug 被接受")
        except ValueError:
            check("TOLERATED 必须带 BUG 编号", True, "空 BUG 编号被拒绝")

    # 3) baseline.json ↔ history 完整性（t10 R1 + t14 F1：手工旁路必须机器可检出）
    with tempfile.TemporaryDirectory() as td:
        bpath = Path(td) / "baseline.json"
        hpath = Path(td) / "baseline_history.jsonl"

        def mk(v: int) -> dict:
            m = dict.fromkeys(METRIC_KEYS, 0)
            m["rust_test_count"] = v
            return {"schema": 1, "metrics": m, "assets": {}, "seed": {}, "tolerated": [], "established": {}}

        base = mk(5)
        bpath.write_text(baseline_payload(base), encoding="utf-8")
        records = [{
            "ts": "t", "kind": "metric", "metric": "rust_test_count", "old": 0, "new": 5,
            "reason": "初始化", "metrics": dict(base["metrics"]),
            "baseline_sha256": sha256_file(bpath),
        }]
        probs = verify_baseline_integrity(base, records, bpath)
        check("完整性校验通过（未污染且快照覆盖全指标）", probs == [], f"problems={probs}")
        tampered = mk(0)                       # 复现 t4 取证：手工把 rust_test_count 5→0
        bpath.write_text(baseline_payload(tampered), encoding="utf-8")
        probs = verify_baseline_integrity(tampered, records, bpath)
        check("手工调低基线被检出（『基线被手工调低』）", any("『基线被手工调低』" in p for p in probs), "; ".join(probs))
        check("手工改动断 sha 链被检出", any("sha 链" in p for p in probs), "; ".join(probs))
        lifted = mk(5)                         # 非下调的非自洽/单侧手改（如塞 tolerated）也必须断链
        lifted["tolerated"] = [{"gate": "x", "item": "y", "bug": "BUG-1", "reason": "r"}]
        bpath.write_text(baseline_payload(lifted), encoding="utf-8")
        probs = verify_baseline_integrity(lifted, records, bpath)
        check("非下调手改同样断链（非自洽/单侧改写不可行）", any("sha 链" in p for p in probs), "; ".join(probs))
        forged = records + [{
            "ts": "t2", "kind": "metric", "metric": "rust_test_count", "old": 5, "new": 1,
            "reason": "伪造", "baseline_sha256": "0" * 64,
        }]
        bpath.write_text(baseline_payload(mk(5)), encoding="utf-8")
        probs = verify_baseline_integrity(mk(5), forged, bpath)
        check("历史记录自身回退被检出", any("回退" in p for p in probs), "; ".join(probs))

        # t14 F1：删历史/空历史 ⇒ FAIL『历史缺失：基线单调性下界不可验证』（禁 vacuous 通过）
        probs = verify_baseline_integrity(mk(5), [], bpath)
        check("删/空历史被检出（禁 vacuous 通过）",
              any("历史缺失：基线单调性下界不可验证" in p for p in probs), "; ".join(probs))
        hpath.write_text("", encoding="utf-8")
        try:
            history_records(hpath)
            check("history_records 空文件前置校验", False, "空历史被接受")
        except BaselineFormatError as exc:
            check("history_records 空文件前置校验", "历史缺失" in str(exc), str(exc))
        hpath.unlink()
        try:
            history_records(hpath)
            check("history_records 缺失前置校验", False, "缺失历史被接受")
        except BaselineFormatError as exc:
            check("history_records 缺失前置校验", "历史缺失" in str(exc), str(exc))
        partial = [{                       # 只留单条记录 ≠ 下界锚：未覆盖 metric 必须报
            "ts": "t", "kind": "metric", "metric": "rust_test_count", "old": 0, "new": 5,
            "reason": "r", "baseline_sha256": sha256_file(bpath),
        }]
        probs = verify_baseline_integrity(base, partial, bpath)
        check("下界覆盖缺口被检出（只留部分记录不算下界锚）",
              any("rom_files_verified 基线单调性下界不可验证" in p for p in probs), "; ".join(probs))
        # t14 F2：损坏 JSON ⇒ BaselineFormatError（显式裁定行，禁裸 traceback）
        hpath.write_text("{broken json\n", encoding="utf-8")
        try:
            history_records(hpath)
            check("history JSON 损坏转 BaselineFormatError", False, "损坏历史被接受")
        except BaselineFormatError as exc:
            check("history JSON 损坏转 BaselineFormatError", "解析失败" in str(exc), str(exc))
        bpath.write_text("{broken", encoding="utf-8")
        try:
            load_baseline(bpath)
            check("baseline JSON 损坏转 BaselineFormatError", False, "损坏基线被接受")
        except BaselineFormatError as exc:
            check("baseline JSON 损坏转 BaselineFormatError", "解析失败" in str(exc), str(exc))

    # 4) commit-msg 证据门禁（t10 R4① 别名收紧 + R4② 删除/改名路径语义）
    ok, _ = commit_msg_check("feat(data): rename", ["data/naming/ledger.jsonl"])
    check("commit-msg 拒绝缺 Evidence 的触及提交", not ok, "触及 data/naming/ 且缺 Evidence ⇒ 拒绝")
    ok, _ = commit_msg_check("feat(data): rename\n\nEvidence: bank3:$9A2C", ["rom/gunnac_cn.nes"])
    check("commit-msg 拒绝缺 Confidence 的触及提交", not ok, "触及 rom/ 且缺 Confidence ⇒ 拒绝")
    ok, _ = commit_msg_check(
        "refactor(ref): 拆分\n\nEvidence: bank3:$9A2C STA $0320\nConfidence: 90",
        ["reference/src/a.s"],
    )
    check("commit-msg 接受带 Evidence/Confidence 的触及提交", ok, "字段齐备 ⇒ 接受")
    ok, _ = commit_msg_check("docs(kb): 知识条目", ["docs/knowledge/x.md"])
    check("commit-msg 非触及提交免证据字段", ok, "未触及 data/naming/、reference/、rom/ ⇒ 免除")
    ok, _ = commit_msg_check(
        "feat(data): rename\n\n证据: bank3:$9A2C\n置信度: 90",
        ["data/naming/ledger.jsonl"],
    )
    check("commit-msg 中文 证据/置信度 别名可识别", ok, "文档化中文别名接受")
    ok, _ = commit_msg_check(
        "feat(data): rename\n\n证据: bank3:$9A2C\n信心: 90",
        ["data/naming/ledger.jsonl"],
    )
    check("commit-msg 拒绝未文档化别名『信心:』（R4①）", not ok, "『信心:』不再通过（README 仅承诺 Evidence/证据 + Confidence/置信度）")
    ok, _ = commit_msg_check(
        "feat(data): rename\n\n证据: bank3:$9A2C\n置信: 90",
        ["data/naming/ledger.jsonl"],
    )
    check("commit-msg 拒绝未文档化别名『置信:』（R4①）", not ok, "裸『置信:』不再通过")
    ok, _ = commit_msg_check("# Evidence: 仅在注释行\nfeat: x", ["rom/gunnac_cn.nes"])
    check("commit-msg 注释行 Evidence 不算数", not ok, "仅注释行 ⇒ 拒绝（t4 回归项）")
    ns_text = "D\trom/gunnac_cn.nes\nR100\tdata/naming/ledger.jsonl\tdrafts/old.jsonl\nA\tcrates/x.rs\n"
    ns_paths = parse_name_status(ns_text)
    ok, _ = commit_msg_check("feat: 删除与改名", ns_paths)
    check("删除/改名两侧路径均计入触及（R4②）", not ok and "rom/gunnac_cn.nes" in ns_paths
          and "drafts/old.jsonl" in ns_paths and "data/naming/ledger.jsonl" in ns_paths,
          f"parse_name_status => {ns_paths}")
    ok, msg = commit_msg_check("feat: 删普通文件", parse_name_status("D\tcrates/x.rs\n"))
    check("文案 join 无双斜杠且与敏感集派生一致（R3）",
          ok and SENSITIVE_TEXT in msg and "//" not in msg and SENSITIVE_TEXT == "、".join(SENSITIVE_PREFIXES), msg)
    ok, _ = commit_msg_check("feat(status): 基线调整", ["data/status/baselines/baseline.json"])
    check("data/status/ 计入敏感路径（t14 F1 攻击链封口）", not ok,
          "触及 data/status/ 的提交缺 Evidence/Confidence ⇒ 拒绝（降基线提交必须带证据）")

    # 5) SKIP/TOLERATED 显式打印（不许静默通过）+ 前置对象消失守卫
    results = run_all_gates()
    text = "\n".join(format_gate_lines(results))
    silent = [r["name"] for r in results if r["verdict"] == SKIP and ("SKIP" not in text or not r["detail"].strip())]
    check("SKIP 显式打印（含原因）", not silent, f"违规项: {silent}" if silent else f"{sum(1 for r in results if r['verdict'] == SKIP)} 个 SKIP 全部带原因打印")
    check(
        "TOLERATED 显式打印",
        all("TOLERATED" in ln for ln in tolerated_lines(load_baseline())),
        "baseline.tolerated 全部显式打印",
    )
    check(
        "每个门禁都有裁定行",
        all(r["verdict"] in (PASS, FAIL, SKIP, TOLERATED) and r["detail"].strip() for r in results),
        f"{len(results)} 个门禁均有 verdict+detail",
    )
    v1, _ = missing_precondition({"established": {}}, "naming-ledger", SKIP, "台账不存在")
    v2, d2 = missing_precondition({"established": {"naming-ledger": True}}, "naming-ledger", SKIP, "台账不存在")
    check("前置对象消失守卫（established ⇒ FAIL『前置对象消失』）", v1 == SKIP and v2 == FAIL,
          f"未建立={v1} 已建立={v2} :: {d2}")

    # 6) 注册表 ↔ gates/README.md 门禁清单逐字对齐 + 命令面双向相等（t10 R2）
    readme = (ROOT / "gates" / "README.md").read_text(encoding="utf-8")
    section = readme.split("## 门禁清单", 1)[1].split("## ", 1)[0]
    rows = re.findall(r"^\|\s*`([a-z0-9-]+)`\s*\|[^|]*\|\s*([^|]+?)\s*\|", section, re.M)
    readme_pairs = [(name, state) for name, state in rows]
    reg_pairs = [(e[0], e[1]) for e in GATES]
    check("注册表与 README 门禁清单一致", readme_pairs == reg_pairs,
          f"README={readme_pairs}" if readme_pairs != reg_pairs else f"{len(reg_pairs)} 项逐字一致")
    cmds = set(re.findall(r"gates/cli\.py ([a-z-]+)", readme))
    actual_cmds = parser_command_names()
    check("CLI 子命令面 ↔ README 命令表双向相等", cmds == actual_cmds,
          f"README={sorted(cmds)} CLI={sorted(actual_cmds)}")

    # 7) rom 锚点校验逻辑（临时文件篡改可检出）
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        f = tmp / "x.bin"
        f.write_bytes(b"gunnac")
        digest = sha256_file(f)
        sums = tmp / "SHA256SUMS"
        sums.write_text(f"{digest}  x.bin\n", encoding="utf-8")
        entries = parse_sha256sums(sums)
        ok = entries.get("x.bin") == sha256_file(f)
        f.write_bytes(b"tampered")
        detected = entries.get("x.bin") != sha256_file(f)
        check("SHA256SUMS 篡改可检出", ok and detected, "一致→通过，改 1 字节→检出不符")

    failed = [c for c in cases if not c[1]]
    for name, ok, detail in cases:
        print(f"  [{'ok' if ok else 'FAIL'}] {name}" + (f" —— {detail}" if detail else ""))
    print(f"SELFTEST: {len(cases) - len(failed)}/{len(cases)} 通过")
    return 1 if failed else 0


# ---------------------------------------------------------------- 入口

def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="gates/cli.py", description="门禁工具链单一真相源（AGENTS.md §5）")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("status", help="只读预检：门禁清单 + 启用/SKIP 状态 + 基线对照").set_defaults(func=cmd_status)
    sub.add_parser("check", help="总闸（pre-commit 调用）").set_defaults(func=cmd_check)
    # 单门禁命令只有 README 命令表所列的 5 个（t10 R2：CLI 子命令面与 README 双向一致，
    # 不再注册门禁名别名）。
    for cmd_name, gate_name in (("rom", "rom-integrity"), ("seed", "seed-integrity"),
                                ("naming", "naming-ledger"), ("trace", "trace-diff"), ("visual", "visual-diff")):
        p = sub.add_parser(cmd_name, help=f"单门禁 {gate_name}")
        p.set_defaults(func=cmd_single(cmd_name, gate_name))
    sub.add_parser("all", help="全量：所有门禁 + selftest").set_defaults(func=cmd_all)
    sub.add_parser("selftest", help="门禁工具链自证（ratchet/commit-msg/SKIP 打印/清单对齐）").set_defaults(func=cmd_selftest)

    p_base = sub.add_parser("baseline", help="基线管理（只升不降 + 强制说明）")
    base_sub = p_base.add_subparsers(dest="baseline_action", required=True)
    base_sub.add_parser("show", help="显示基线与更新历史")
    p_up = base_sub.add_parser("update", help="上调基线指标（拒绝下调/空说明）")
    p_up.add_argument("--metric", required=True, choices=METRIC_KEYS)
    p_up.add_argument("--value", required=True, type=int)
    p_up.add_argument("--reason", required=True, help="必须写明『确认无回退』的证据")
    p_tol = base_sub.add_parser("tolerate", help="登记显式 TOLERATED 已知偏差（需 BUG 编号）")
    p_tol.add_argument("--gate", required=True)
    p_tol.add_argument("--item", required=True)
    p_tol.add_argument("--bug", required=True, help="BUG-xxx（AGENTS.md §6）")
    p_tol.add_argument("--reason", required=True)
    p_base.set_defaults(func=cmd_baseline)

    p_msg = sub.add_parser("commit-msg", help="commit-msg 钩子入口：证据门禁")
    p_msg.add_argument("msgfile", help="commit message 文件路径")
    p_msg.set_defaults(func=cmd_commit_msg)
    return parser


def parser_command_names() -> set[str]:
    """CLI 实际注册的子命令面（从 argparse 解析器提取，非第二份手写清单）。"""
    parser = build_parser()
    names: set[str] = set()
    for action in parser._actions:  # noqa: SLF001 —— 自检需要读取子命令面
        if isinstance(action, argparse._SubParsersAction):  # noqa: SLF001
            names.update(action.choices.keys())
    return names


def main() -> int:
    args = build_parser().parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
