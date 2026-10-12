#!/usr/bin/env python3
"""matrix_sync.py —— 完备性矩阵「人读版 ↔ 机器可读版」机械一致性自检/导出。

人读版（单一真相源）: docs/spec/completeness.md
机器可读版（门禁输入）: data/status/completeness_matrix.json

用法:
    python3 tools/matrix_sync.py --check      # 一致性自检（默认）；不一致 exit 1
    python3 tools/matrix_sync.py --export     # 从人读版机械导出 JSON（覆盖写）
    python3 tools/matrix_sync.py --check --json <path>

校验项（同格两处不得矛盾）:
    1. 行为面格 ID 集合相等（md §2 表格 44 格 ↔ json cells）
    2. 强度（L0-L4）逐格相等
    3. md 证据列的 E-xx 引用 ⊆ json evidence 列表
    4. content 逐字相等（md 表格内容列 ↔ json content）
    5. json 结构契约: 每格含 id/face/content/evidence/strength/bindings/covered；
       covered ⇔ strength ≥ L1（覆盖语义见 completeness.md §6.1/§8）

注意: md 表格证据列的「同上」按上一行解析（含「同上 + …」追加形态）。
路径契约: 可复用工具入 tools/（AGENTS.md §2；gotcha-reusable-tools-belong-in-tools）。
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MD = ROOT / "docs" / "spec" / "completeness.md"
JSON_DEFAULT = ROOT / "data" / "status" / "completeness_matrix.json"

CELL_ROW_RE = re.compile(r"^\|\s*(B-[A-Za-z0-9-]+)\s*\|")
EREF_RE = re.compile(r"E-[RSW][0-9]+[a-z]?")
STRENGTH_RE = re.compile(r"L([0-4])")
SCEN_RE = re.compile(r"(scenario|trace):([^\s（）、；]+)")
RULE_RE = re.compile(r"(R-(?:[a-z]+-){1,2}[0-9]+(?:\.\.[0-9]+)?)")
REQUIRED_KEYS = ("id", "face", "content", "evidence", "strength", "bindings", "covered")


def parse_md() -> tuple[dict[str, dict], dict[str, dict]]:
    """解析人读版 → (cells, records)。cells 以格 ID 为键；records 为 §2.4.x 登记记录（后者胜）。"""
    text = MD.read_text(encoding="utf-8")
    cells: dict[str, dict] = {}
    last_refs: list[str] = []
    for line in text.splitlines():
        m = CELL_ROW_RE.match(line)
        if not m:
            continue
        cols = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cols) < 5:
            continue
        cid, content, ev_col, _grade, mol_col = cols[:5]
        refs = EREF_RE.findall(ev_col)
        if ev_col.startswith("同上"):
            refs = sorted(set(last_refs + refs))
        else:
            refs = sorted(set(refs))
        last_refs = refs
        sm = STRENGTH_RE.search(mol_col)
        if not sm:
            raise SystemExit(f"md 行缺强度标记: {line}")
        cells[cid] = {
            "id": cid,
            "face": "behavior",
            "content": content,
            "evidence": refs,
            "strength": f"L{sm.group(1)}",
            "bindings": {},
            "covered": f"L{sm.group(1)}" != "L0",
        }

    # §2.4.x 分子登记记录（``` 块内 格:/证据:/工件:）——同格多条记录全部保留（L1→L2 增补证据，绑定取并集）
    records: dict[str, list[dict]] = {}
    cur: dict | None = None
    for line in text.splitlines():
        if line.startswith("格: "):
            cur = {"id": line.split(":", 1)[1].strip()}
        elif cur is not None and line.startswith("证据:"):
            cur["evidence_line"] = line[3:]
        elif cur is not None and line.startswith("工件:"):
            cur["artifacts"] = line[3:]
        elif cur is not None and line.strip() == "" and "artifacts" in cur:
            records.setdefault(cur["id"], []).append(cur)
            cur = None
    if cur is not None and "artifacts" in cur:
        records.setdefault(cur["id"], []).append(cur)
    return cells, records


def bindings_from_records(records: dict[str, list[dict]]) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for cid, recs in records.items():
        for rec in recs:
            ev = rec.get("evidence_line", "")
            arts = rec.get("artifacts", "")
            scen = [f"{k}:{v}" for k, v in SCEN_RE.findall(ev)]
            rules = set(RULE_RE.findall(arts))
            slot = out.setdefault(cid, {"scenario": [], "trace": [], "rule": []})
            for s in scen:  # 多记录取并集（覆盖只增：增补证据不丢历史绑定）
                key = "trace" if s.startswith("trace:") else "scenario"
                if s not in slot[key]:
                    slot[key].append(s)
            slot["rule"] = sorted(set(slot["rule"]) | rules)
    for cid in out:
        out[cid]["scenario"] = sorted(out[cid]["scenario"])
        out[cid]["trace"] = sorted(out[cid]["trace"])
    return out


def build_json() -> dict:
    cells, records = parse_md()
    binds = bindings_from_records(records)
    for cid, cell in cells.items():
        if cid in binds:
            cell["bindings"] = binds[cid]
    covered = sum(1 for c in cells.values() if c["covered"])
    return {
        "schema_version": 1,
        "source_of_truth": "docs/spec/completeness.md（本文件由 tools/matrix_sync.py --export 生成后人工核）",
        "id_rule": "B=行为面 / S=符号面 / D=数据面 / R=规则面；格 ID 一经入分母永不复用（completeness.md §0）",
        "strength_scale": "L0-L4 矩阵轴（completeness.md §6.1）；与差分金字塔 L1-L5 是两条轴，勿混淆",
        "covered_rule": "covered ⇔ strength >= L1（带证据指针的分子登记；L0=入分母未验证）",
        "cell_schema": ["id", "face", "content", "evidence", "strength", "bindings{scenario,trace,rule}", "covered"],
        "faces": {
            "behavior": {"denominator": len(cells), "covered": covered},
            "rule": {
                "denominator": len(cells),
                "covered_cells": covered,
                "rule_entries": 21,
                "note": "规则面分母=行为格数×1 下限；条目数为 shop-economy 13 + ui-flow 8（completeness.md §5）",
            },
            "symbol": {
                "status": "S1 机械导出已建立（2026-10-12 t7，completeness.md §3）",
                "counts": {
                    "subroutines": 276,
                    "irq_vectors": 3,
                    "farcall_trampoline_sites": 18,
                    "bank11_entry_table": 7,
                    "ram_dp_symbols": 560,
                    "mmio_symbols": 414,
                    "byte_coverage": "131072/131072",
                },
            },
            "data": {
                "status": "S1 机械导出已建立（2026-10-12 t7，completeness.md §4）",
                "counts": {"data_tables": 8, "known_anchors_not_denominator": 7, "chr_banks_1k": 128},
            },
        },
        "cells": cells,
    }


def check(json_path: Path) -> int:
    md_cells, records = parse_md()
    binds = bindings_from_records(records)
    doc = json.loads(json_path.read_text(encoding="utf-8"))
    cells = doc.get("cells", {})
    problems: list[str] = []

    if set(cells) != set(md_cells):
        only_md = sorted(set(md_cells) - set(cells))
        only_json = sorted(set(cells) - set(md_cells))
        if only_md:
            problems.append(f"仅在 md: {only_md}")
        if only_json:
            problems.append(f"仅在 json: {only_json}")

    for cid in sorted(set(cells) & set(md_cells)):
        j, m = cells[cid], md_cells[cid]
        missing = [k for k in REQUIRED_KEYS if k not in j]
        if missing:
            problems.append(f"{cid}: json 缺字段 {missing}")
            continue
        if j["id"] != cid:
            problems.append(f"{cid}: json id 字段={j['id']!r} 不匹配键名")
        if j["strength"] != m["strength"]:
            problems.append(f"{cid}: 强度矛盾 json={j['strength']} md={m['strength']}")
        if j["content"] != m["content"]:
            problems.append(f"{cid}: content 不一致（json 与 md 内容列不逐字相等）")
        missing_refs = [r for r in m["evidence"] if r not in j["evidence"]]
        if missing_refs:
            problems.append(f"{cid}: md 证据引用 {missing_refs} 不在 json evidence")
        want_covered = j["strength"] != "L0"
        if bool(j["covered"]) != want_covered:
            problems.append(f"{cid}: covered={j['covered']} 与强度 {j['strength']} 矛盾（covered ⇔ ≥L1）")
        if want_covered and cid in binds and j.get("bindings") != binds[cid]:
            problems.append(f"{cid}: bindings 与 md §2.4.x 登记记录不一致（json={j.get('bindings')} md={binds[cid]}）")

    if problems:
        print(f"FAIL: {len(problems)} 处不一致（{json_path.name} ↔ {MD.name}）")
        for p in problems:
            print(" -", p)
        return 1
    covered = sum(1 for c in cells.values() if c["covered"])
    print(f"OK: {len(cells)} 行为面格一致（强度/证据引用/content/结构契约），covered={covered}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="一致性自检（默认动作）")
    ap.add_argument("--export", action="store_true", help="从人读版导出 JSON（覆盖写）")
    ap.add_argument("--json", type=Path, default=JSON_DEFAULT, help="机器可读矩阵路径")
    args = ap.parse_args()

    if args.export:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        doc = build_json()
        args.json.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"EXPORT: {len(doc['cells'])} 格 → {args.json}")
    return check(args.json)


if __name__ == "__main__":
    sys.exit(main())
