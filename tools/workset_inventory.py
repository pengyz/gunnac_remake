#!/usr/bin/env python3
"""tools/workset_inventory.py — T1 工作全集盘点（rewrite-workset）机械导出器。

从冻结投影 + 机械层导出**子程序入口全集**（call-graph/入口枚举）、机械画像分类
（引擎/数据/图形/音频/待定）与移植状态映射（L2 用例 / game-core 引用 / spec / findings）。

输入（库内只读）：
  reference/seed/gunnac.asm          冻结 6502 反汇编投影（D3）
  data/naming/remap-table.json       S1 机械层（向量 / far-call / bank11 表 / f_000..f_275）
  analysis/ida/bankNN.json           S1 IDA 导出（仅作交叉核对；可再生，不入库）
  oracle/subroutines/l2_cases.jsonl  L2 差分用例（121 条 / 29 子程序）
  crates/game-core/src/*.rs          移植面（bank:addr 证据引用）
  docs/spec/*.md  docs/findings/*.md 考证面（bank:addr 证据引用）

入口全集规则（机械，A 级字节指针；bankNN:AAAA 均可回 seed grep 重导）：
  E1 每 bank 段起点（窗口基址）
  E2 JSR/JMP 绝对目标（seed 操作数 sub_XXXX / loc_XXXX / $XXXX）：
     同窗目标归本 bank；$C000-$DFFF 归 bank14；$E000-$FFFF 归 bank15；
     其余跨窗目标计 unresolved（R6/R7 运行期决定，静态不可判）
  E3 中断向量 3 个（bank15 固定窗，file 0x2000A）
  E4 far-call 内联目标（JSR $E649 + (r6,r7,target) 约定，rom-forensics §2）
  E5 bank11 $8000 JMP 表目标 7 个
  E6 JMP 间接（表驱动分派）**不展开表项**——站点单独计数入分母缺口（待证）

切分约定（B 级）：bank 内按入口地址升序切段，段 i = [entry_i, entry_{i+1})，
末段至窗口尾——全 PRG 每字节恰属一段（131,072/131,072）。段是**地址序切片**，
不是已证明的函数体；函数体边界考证（S3）按需后补。

分类规则（B 级机械画像，指纹随表列出供逐行重导；第一条命中生效）：
  R-X 待定   data 项行 > 指令行（线性扫描噪声区：足迹失真，不作语义分类）
  R-A 音频   apu_write>0（写 $4000-$4013/$4015/$4017）
  R-G 图形   ppu>0（$2000-$2007）或 oamdma_write>0（$4014）
  R-D 数据   无 MMIO + rom_idx_reads>=2 + loop + ram_writes>=1 + calls_out==0
             （表行走/记录装载签名）
  R-E 引擎   其余带实质流（calls_out>0 或 ram_writes>0 或 mapper_write>0）
  R-? 待定   其余（过小/纯透传/无签名）

输出：
  analysis/workset/subroutines.json  全量指纹（可再生工作区，不入库）
  analysis/workset/tables.md         表格片段（docs/spec/rewrite-workset.md 嵌入源）
  data/status/rewrite-workset.json   机器可读镜像（分类+状态+计数；进度可算）
  stdout                             摘要 + 交叉核对

复现：python3 tools/workset_inventory.py   # 幂等全量重导（diff 可比）
纪律：只做机械画像（A 级字节足迹 + B 级结构推理），不做语义定谳。
"""
from __future__ import annotations

import collections
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SEED = ROOT / "reference" / "seed" / "gunnac.asm"
REMAP = ROOT / "data" / "naming" / "remap-table.json"
IDA_DIR = ROOT / "analysis" / "ida"
L2CASES = ROOT / "oracle" / "subroutines" / "l2_cases.jsonl"
CORE_SRC = ROOT / "crates" / "game-core" / "src"
SPEC_DIR = ROOT / "docs" / "spec"
FIND_DIR = ROOT / "docs" / "findings"
OUT_DIR = ROOT / "analysis" / "workset"
MIRROR = ROOT / "data" / "status" / "rewrite-workset.json"

SEED_LINE = re.compile(
    r"^bank(\d\d):([0-9A-F]{4})\s+((?:[0-9A-F]{2} )*[0-9A-F]{2})\s+(\S+)\s*(.*)$"
)
# data 项行（seed_build.py 渲染：助记符列 = "$XX,$XX,..." 无操作数）
DATA_LINE = re.compile(r"^\$[0-9A-F]{2}(?:,\$[0-9A-F]{2})*$")
CITE = re.compile(r"bank(\d{1,2}):\$?([0-9A-Fa-f]{4})(?:\s*-\s*\$?([0-9A-Fa-f]{4}))?")

# bank 窗口（t2 布局，rom-forensics §2）：R6->$8000 R7->$A000 bank14->$C000 bank15->$E000
WIN = {0: 0x8000, 1: 0xA000, 2: 0x8000, 3: 0xA000, 4: 0x8000, 5: 0x8000, 6: 0x8000,
       7: 0xA000, 8: 0xA000, 9: 0x8000, 10: 0x8000, 11: 0x8000, 12: 0xA000,
       13: 0x8000, 14: 0xC000, 15: 0xE000}
WEND = {b: WIN[b] + 0x2000 for b in WIN}

PPU = range(0x2000, 0x2008)
APU = range(0x4000, 0x4014)
OAM_DMA = 0x4014
CTRL = (0x4016, 0x4017)
MAPPER_LO, MAPPER_HI = 0x8000, 0xFFFF

WRITE_MNEM = {"STA", "STX", "STY"}
RMW_MNEM = {"ASL", "LSR", "ROL", "ROR", "INC", "DEC"}
READ_MNEM = {"LDA", "LDX", "LDY", "CMP", "CPX", "CPY", "BIT", "ADC", "SBC",
             "AND", "ORA", "EOR", "NOP"}
BRANCH_MNEM = {"BPL", "BMI", "BVC", "BVS", "BCC", "BCS", "BNE", "BEQ"}
CALL_MNEM = {"JSR"}
JUMP_MNEM = {"JMP"}

LABEL_OPERAND = re.compile(r"^(?:[A-Za-z_][A-Za-z0-9_]*_)?([0-9A-F]{4})(?:([+-])([0-9]+))?$")
RAW_OPERAND = re.compile(r"^\$([0-9A-Fa-f]{4})$")
IND_OPERAND = re.compile(r"^\(\$?[0-9A-Fa-f]{2,4}([,XxYy])?\)$")


def parse_seed():
    """bank -> [(addr, nbytes, mnem, operand)] 地址升序。"""
    insns = collections.defaultdict(list)
    for line in SEED.read_text(encoding="utf-8").splitlines():
        m = SEED_LINE.match(line)
        if not m:
            continue
        bank = int(m.group(1))
        addr = int(m.group(2), 16)
        nbytes = len(m.group(3).split())
        insns[bank].append((addr, nbytes, m.group(4), m.group(5).strip()))
    for bank in insns:
        insns[bank].sort()
    return insns


def operand_addr(operand):
    """操作数 → (addr, indexed, indirect)。addr=None 表示非绝对地址操作数。"""
    if not operand:
        return None, False, False
    indexed = operand.upper().endswith(",X") or operand.upper().endswith(",Y")
    indirect = operand.startswith("(")
    if indirect:
        m = re.match(r"^\(\$?([0-9A-Fa-f]{2,4})", operand)
        return (int(m.group(1), 16) if m else None), indexed, True
    m = re.match(r"^\$([0-9A-Fa-f]{2,4})", operand)
    if m:
        return int(m.group(1), 16), indexed, False
    lm = LABEL_OPERAND.match(operand)
    if lm:
        base = int(lm.group(1), 16)
        if lm.group(2):
            off = int(lm.group(3))
            base = base + off if lm.group(2) == "+" else base - off
        return base & 0xFFFF, indexed, False
    return None, indexed, False


def enumerate_entries(insns, remap):
    """入口全集（规则 E1-E5）+ E6 间接分派站点。"""
    entries = collections.defaultdict(set)
    indirect_jmp_sites = []
    unresolved_cross = []          # (from_bank, from_addr, target) 跨窗不可判
    for bank, lst in insns.items():
        entries[bank].add(WIN[bank])                     # E1
        for (addr, nb, mnem, operand) in lst:
            if mnem == "JMP" and operand.startswith("("):
                indirect_jmp_sites.append("bank%02d:%04X" % (bank, addr))   # E6
                continue
            if mnem not in JUMP_MNEM | CALL_MNEM:
                continue
            tgt, indexed, indirect = operand_addr(operand)
            if tgt is None or indirect:
                continue
            if WIN[bank] <= tgt < WEND[bank]:            # E2 同窗
                entries[bank].add(tgt)
            elif 0xC000 <= tgt < 0xE000:
                entries[14].add(tgt)
            elif tgt >= 0xE000:
                entries[15].add(tgt)
            else:
                unresolved_cross.append((bank, addr, tgt))
    for v in remap["vectors"]:                           # E3
        entries[15].add(int(v["cpu"].lstrip("$"), 16))
    for fc in remap["entries"]["far_call_sites"]:        # E4
        t = int(fc["target"].lstrip("$"), 16)
        entries[fc["r6"] if t < 0xA000 else fc["r7"]].add(t)
    for e in remap["entries"]["bank11_jmp_table"]:       # E5
        entries[11].add(int(e["target"].lstrip("$"), 16))
    return entries, indirect_jmp_sites, unresolved_cross


def classify(fp):
    """机械画像分类（B 级）。投影噪声护栏：data 项行 > 指令行 ⇒ 足迹失真，一律 R-X 待定。"""
    if fp["data_lines"] > fp["insn_lines"]:
        return "待定", "R-X"
    if fp["apu_write"] > 0:
        return "音频", "R-A"
    if fp["ppu"] > 0 or fp["oamdma_write"] > 0:
        return "图形", "R-G"
    if (fp["mmio_total"] == 0 and fp["rom_idx_reads"] >= 2 and fp["loop"]
            and fp["ram_writes"] >= 1 and fp["calls_out"] == 0):
        return "数据", "R-D"
    if fp["calls_out"] > 0 or fp["ram_writes"] > 0 or fp["mapper_write"] > 0:
        return "引擎", "R-E"
    return "待定", "R-?"


def main():
    insns = parse_seed()
    remap = json.loads(REMAP.read_text(encoding="utf-8"))
    entries, indirect_jmp_sites, unresolved_cross = enumerate_entries(insns, remap)

    # ---- 段切分（B 级约定：入口地址序切片，全 bank 平铺）----
    segments = []                       # 每段一条指纹记录
    seg_index = {}                      # (bank, addr) -> seg record（段起点）
    entry_to_seg = {}                   # (bank, entry_addr) -> seg record
    for bank in sorted(entries):
        ents = sorted(entries[bank])
        for i, e in enumerate(ents):
            seg_end = ents[i + 1] if i + 1 < len(ents) else WEND[bank]
            r = {
                "bank": bank,
                "seg": "bank%02d:%04X-%04X" % (bank, e, seg_end - 1),
                "seg_start": e, "seg_end": seg_end,
                "entry": e,
            }
            segments.append(r)
            seg_index[(bank, e)] = r
            entry_to_seg[(bank, e)] = r

    def seg_of(bank, addr):
        """bank 窗口地址 → 所属段（二分找 <= addr 的最大段起点）。"""
        ents = sorted(entries[bank])
        lo, hi = 0, len(ents) - 1
        cand = None
        while lo <= hi:
            mid = (lo + hi) // 2
            if ents[mid] <= addr:
                cand = ents[mid]
                lo = mid + 1
            else:
                hi = mid - 1
        return seg_index.get((bank, cand)) if cand is not None else None

    # ---- 逐段指纹 + 出边 ----
    for r in segments:
        bank, s0, s1 = r["bank"], r["seg_start"], r["seg_end"]
        fp = dict(ppu=0, ppu_write=0, apu_write=0, apu_read=0, oamdma_write=0,
                  ctrl_read=0, mapper_write=0, mmio_total=0,
                  rom_reads=0, rom_idx_reads=0, ram_writes=0, ram_rw=0,
                  calls_out=0, jmps_out=0, loop=0, insn_lines=0,
                  data_lines=0, data_bytes=0)
        call_targets, jmp_targets = [], []
        for (addr, nb, mnem, operand) in insns[bank]:
            if addr < s0 or addr >= s1:
                continue
            if DATA_LINE.match(mnem) and not operand:
                fp["data_lines"] += 1
                fp["data_bytes"] += nb
                continue
            fp["insn_lines"] += 1
            tgt, indexed, indirect = operand_addr(operand)
            is_write = mnem in WRITE_MNEM
            is_rmw = mnem in RMW_MNEM
            is_read = mnem in READ_MNEM or is_rmw or mnem in BRANCH_MNEM
            if tgt is not None and not indirect:
                if tgt in PPU:
                    fp["ppu"] += 1
                    if is_write or is_rmw:
                        fp["ppu_write"] += 1
                if tgt in APU or (tgt == 0x4017 and (is_write or is_rmw)):
                    if is_write or is_rmw:
                        fp["apu_write"] += 1
                    else:
                        fp["apu_read"] += 1
                if tgt == OAM_DMA and (is_write or is_rmw):
                    fp["oamdma_write"] += 1
                if tgt in CTRL and is_read and not is_write:
                    fp["ctrl_read"] += 1
                if MAPPER_LO <= tgt <= MAPPER_HI and (is_write or is_rmw):
                    fp["mapper_write"] += 1
                if MAPPER_LO <= tgt <= MAPPER_HI and is_read and not is_write:
                    fp["rom_reads"] += 1
                    if indexed:
                        fp["rom_idx_reads"] += 1
                if tgt < 0x0800:
                    if is_write or is_rmw:
                        fp["ram_writes"] += 1
                    if is_rmw:
                        fp["ram_rw"] += 1
                if mnem in BRANCH_MNEM and tgt <= addr:
                    fp["loop"] += 1
                if mnem in CALL_MNEM:
                    fp["calls_out"] += 1
                    call_targets.append(tgt)
                if mnem in JUMP_MNEM:
                    fp["jmps_out"] += 1
                    jmp_targets.append(tgt)
        fp["mmio_total"] = (fp["ppu"] + fp["apu_write"] + fp["apu_read"]
                            + fp["oamdma_write"] + fp["ctrl_read"])
        # 出边解析（同窗 → 本 bank；固定窗 → bank14/15；其余 unresolved）
        resolved, unresolved = [], []
        for tgt in call_targets + jmp_targets:
            if WIN[bank] <= tgt < WEND[bank]:
                hit = entry_to_seg.get((bank, tgt))
            elif 0xC000 <= tgt < 0xE000:
                hit = entry_to_seg.get((14, tgt)) or seg_of(14, tgt)
            elif tgt >= 0xE000:
                hit = entry_to_seg.get((15, tgt)) or seg_of(15, tgt)
            else:
                hit = None
            if hit is not None:
                resolved.append(hit["seg"])
            else:
                unresolved.append(tgt)
        cls, rule = classify(fp)
        r["fp"] = fp
        r["class"] = cls
        r["class_rule"] = rule
        r["callees"] = sorted(set(resolved))
        r["unresolved_targets"] = sorted(set(unresolved))
        r["far_sites"] = []

    # far-call 边：站点段 → 目标段（E4 约定：target<$A000 ⇒ R6=r6 bank，否则 R7=r7 bank）
    for fc in remap["entries"]["far_call_sites"]:
        m = CITE.match(fc["site"])
        sbank, saddr = int(m.group(1)), int(m.group(2), 16)
        taddr = int(fc["target"].lstrip("$"), 16)
        tbank = fc["r6"] if taddr < 0xA000 else fc["r7"]
        src = seg_of(sbank, saddr)
        dst = entry_to_seg.get((tbank, taddr)) or seg_of(tbank, taddr)
        if src is not None:
            src["far_sites"].append(dict(fc))
            if dst is not None:
                src["callees"] = sorted(set(src["callees"]) | {dst["seg"]})

    # 入边
    callers = collections.defaultdict(set)
    for r in segments:
        for c in r["callees"]:
            if c != r["seg"]:
                callers[c].add(r["seg"])
    for r in segments:
        r["callers"] = sorted(callers[r["seg"]])

    # ---- 根可达（NMI/RESET/IRQ BFS）----
    by_seg = {r["seg"]: r for r in segments}

    def root_seg(bank, addr):
        r = entry_to_seg.get((bank, addr)) or seg_of(bank, addr)
        return r["seg"] if r else None

    roots = {}
    for v in remap["vectors"]:
        roots[v["name"]] = root_seg(15, int(v["cpu"].lstrip("$"), 16))
    far_targets, b11_targets = set(), set()
    for fc in remap["entries"]["far_call_sites"]:
        taddr = int(fc["target"].lstrip("$"), 16)
        sg = root_seg(fc["r6"] if taddr < 0xA000 else fc["r7"], taddr)
        if sg:
            far_targets.add(sg)
    for e in remap["entries"]["bank11_jmp_table"]:
        sg = root_seg(11, int(e["target"].lstrip("$"), 16))
        if sg:
            b11_targets.add(sg)

    reach = {}
    for name, start in roots.items():
        seen = set()
        if start:
            seen.add(start)
            stack = [start]
            while stack:
                cur = stack.pop()
                for nxt in by_seg[cur]["callees"]:
                    if nxt not in seen:
                        seen.add(nxt)
                        stack.append(nxt)
        reach[name] = seen
    for r in segments:
        r["roots"] = sorted(n for n in ("NMI", "RESET", "IRQ") if r["seg"] in reach[n])
        r["far_target"] = r["seg"] in far_targets
        r["bank11_table_target"] = r["seg"] in b11_targets

    # 入口来源可信度（B 级）：调用方全为 R-X 投影噪声段 ⇒ 噪声可疑；
    # 无调用方且非根/far/b11 ⇒ 无来源（可能是未枚举分派表项或死码/噪声）。
    for r in segments:
        cs = [by_seg[c] for c in r["callers"]]
        r["caller_count"] = len(cs)
        if not cs:
            r["entry_trust"] = ("root" if r["roots"] or r["far_target"]
                                or r["bank11_table_target"] else "no-caller")
        elif all(c["class_rule"] == "R-X" for c in cs):
            r["entry_trust"] = "noise-suspect"
        else:
            r["entry_trust"] = "solid"

    # ---- 移植状态映射 ----
    l2_by_seg = collections.defaultdict(lambda: {"subs": set(), "cases": 0})
    l2_unmapped = []
    for line in L2CASES.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        c = json.loads(line)
        m = CITE.search(c["entry"])
        bank = int(m.group(1))
        addr = int(m.group(2), 16)
        r = seg_of(bank, addr)
        if r is None:
            l2_unmapped.append(c["entry"])
            continue
        l2_by_seg[r["seg"]]["subs"].add(c["subroutine"])
        l2_by_seg[r["seg"]]["cases"] += 1

    def citation_index(paths):
        idx = collections.defaultdict(set)
        for p in paths:
            text = p.read_text(encoding="utf-8", errors="replace")
            for m in CITE.finditer(text):
                bank = int(m.group(1))
                a0 = int(m.group(2), 16)
                a1 = int(m.group(3), 16) if m.group(3) else a0
                if a1 < a0:
                    a0, a1 = a1, a0
                # 引用区间与段相交即计（段=地址序切片，B 级归属）
                for e in sorted(entries[bank]):
                    r = seg_index[(bank, e)]
                    if not (a1 < r["seg_start"] or a0 >= r["seg_end"]):
                        idx[r["seg"]].add(p.name)
        return idx

    core_idx = citation_index(sorted(CORE_SRC.glob("*.rs")))
    spec_idx = citation_index(sorted(SPEC_DIR.glob("*.md")))
    find_idx = citation_index(sorted(FIND_DIR.glob("*.md")))

    for r in segments:
        l2 = l2_by_seg.get(r["seg"])
        r["l2"] = ({"subroutines": sorted(l2["subs"]), "cases": l2["cases"]}
                   if l2 else {"subroutines": [], "cases": 0})
        r["core_files"] = sorted(core_idx.get(r["seg"], ()))
        r["spec_files"] = sorted(spec_idx.get(r["seg"], ()))
        r["findings_files"] = sorted(find_idx.get(r["seg"], ()))
        if l2:
            r["status"] = "L2-diff"
        elif r["core_files"]:
            r["status"] = "core-cited"
        elif r["spec_files"]:
            r["status"] = "spec-cited"
        elif r["findings_files"]:
            r["status"] = "findings-cited"
        else:
            r["status"] = "unexplored"

    # ---- 计数 ----
    cls_order = ["引擎", "数据", "图形", "音频", "待定"]
    st_order = ["L2-diff", "core-cited", "spec-cited", "findings-cited", "unexplored"]
    matrix = {c: {s: 0 for s in st_order} for c in cls_order}
    for r in segments:
        matrix[r["class"]][r["status"]] += 1
    cls_count = {c: sum(matrix[c].values()) for c in cls_order}
    st_count = {s: sum(matrix[c][s] for c in cls_order) for s in st_order}
    trust_count = collections.Counter(r["entry_trust"] for r in segments)

    # 与 S1 remap 机械层交叉核对
    xcheck = {
        "ida_entry_starts_total": 0,
        "remap_symbols": len(remap["symbols"]),
        "seed_entries_total": sum(len(v) for v in entries.values()),
        "unresolved_cross_window_calls": len(unresolved_cross),
        "indirect_jmp_sites": len(indirect_jmp_sites),
    }
    ida_present = IDA_DIR.is_dir()
    if ida_present:
        tot = 0
        for b in range(16):
            p = IDA_DIR / ("bank%02d.json" % b)
            if not p.exists():
                ida_present = False
                break
            d = json.loads(p.read_text(encoding="utf-8"))
            tot += len({d["base"]} | {x["to"] for x in d["xrefs"]
                                      if x["kind"] in ("call", "jump")})
        xcheck["ida_entry_starts_total"] = tot if ida_present else None
    xcheck["ida_export_present"] = ida_present

    out = {
        "schema": 1,
        "kind": "rewrite-workset-subroutines",
        "generated_by": "tools/workset_inventory.py",
        "sources": {
            "seed": "reference/seed/gunnac.asm",
            "remap": "data/naming/remap-table.json",
            "l2_cases": "oracle/subroutines/l2_cases.jsonl",
        },
        "entry_rules": ["E1 window-base", "E2 JSR/JMP abs target",
                        "E3 vectors", "E4 far-call target", "E5 bank11 jmp-table",
                        "E6 indirect-jmp sites (not expanded)"],
        "segment_rule": "bank-internal address-order slices [entry_i, entry_{i+1})",
        "class_rules": {"R-X": "data-item lines > insn lines (sweep-noise guard)",
                        "R-A": "apu_write>0", "R-G": "ppu>0 or oamdma_write>0",
                        "R-D": "no-MMIO + rom_idx_reads>=2 + loop + ram_writes>=1 + calls_out==0",
                        "R-E": "calls_out>0 or ram_writes>0 or mapper_write>0",
                        "R-?": "otherwise"},
        "universe": {
            "entries_total": sum(len(v) for v in entries.values()),
            "entries_per_bank": {("bank%02d" % b): len(entries[b])
                                 for b in sorted(entries)},
            "segments": len(segments),
            "segment_bytes": sum(r["seg_end"] - r["seg_start"] for r in segments),
        },
        "cross_check": xcheck,
        "counts": {"class": cls_count, "status": st_count,
                   "class_x_status": matrix,
                   "entry_trust": dict(trust_count)},
        "roots": roots,
        "far_call_targets": sorted(far_targets),
        "bank11_table_targets": sorted(b11_targets),
        "indirect_jmp_sites": sorted(indirect_jmp_sites),
        "unresolved_cross_window_calls": [
            "bank%02d:%04X->%04X" % (b, a, t) for (b, a, t) in unresolved_cross],
        "segments": segments,
        "l2_unmapped_entries": sorted(set(l2_unmapped)),
    }
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "subroutines.json").write_text(
        json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8")
    MIRROR.write_text(json.dumps(out, ensure_ascii=False, indent=1),
                      encoding="utf-8", newline="\n")

    # ---- tables.md ----
    lines = ["<!-- generated by tools/workset_inventory.py — 勿手改，重导覆盖 -->", ""]
    lines.append("### 摘要矩阵（段 × 分类 × 状态）")
    lines.append("")
    lines.append("| 分类 \\ 状态 | " + " | ".join(st_order) + " | 小计 |")
    lines.append("|---" * (len(st_order) + 2) + "|")
    for c in cls_order:
        lines.append("| %s | %s | %d |" % (
            c, " | ".join(str(matrix[c][s]) for s in st_order), cls_count[c]))
    lines.append("| **小计** | " + " | ".join("**%d**" % st_count[s] for s in st_order)
                 + " | **%d** |" % len(segments))
    lines.append("")
    lines.append("### L2 已对拍子程序映射（29 名 / 121 例 → 段）")
    lines.append("")
    lines.append("| 子程序 | entry | 用例数 | entry=段首 | 所在段 | 段分类 | 段状态 |")
    lines.append("|---|---|---|---|---|---|---|")
    l2_named = collections.OrderedDict()
    for line in L2CASES.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        c = json.loads(line)
        e = l2_named.setdefault(c["subroutine"], {"entry": c["entry"], "cases": 0})
        e["cases"] += 1
    for name, info in l2_named.items():
        m = CITE.search(info["entry"])
        bank, addr = int(m.group(1)), int(m.group(2), 16)
        r = seg_of(bank, addr)
        exact = (r is not None and r["seg_start"] == addr)
        lines.append("| %s | %s | %d | %s | %s | %s | %s |" % (
            name, info["entry"], info["cases"],
            "是" if exact else "否（段内片段）",
            r["seg"] if r else "—",
            r["class"] if r else "—", r["status"] if r else "—"))
    lines.append("")
    lines.append("### 逐段清单（机械画像 + 移植状态）")
    lines.append("")
    lines.append("| 段（入口-尾） | 来源 | 分类(规则) | 投影 d/i | ppu/apu/oam/map | rom_idx/ram_w | calls/loop | 根 | 状态 | 依据 |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|")
    for r in segments:
        fp = r["fp"]
        basis = []
        if r["l2"]["subroutines"]:
            basis.append("L2:" + ",".join(r["l2"]["subroutines"]) + "(%d)" % r["l2"]["cases"])
        if r["core_files"]:
            basis.append("core:" + ",".join(f[:-3] for f in r["core_files"]))
        if r["spec_files"]:
            basis.append("spec×%d" % len(r["spec_files"]))
        if r["findings_files"]:
            basis.append("find×%d" % len(r["findings_files"]))
        root = "/".join(r["roots"]) if r["roots"] else (
            "far" if r["far_target"] else ("b11" if r["bank11_table_target"] else "—"))
        lines.append("| %s | %s | %s(%s) | %d/%d | %d/%d/%d/%d | %d/%d | %d/%d | %s | %s | %s |" % (
            r["seg"], r["entry_trust"], r["class"], r["class_rule"],
            fp["data_lines"], fp["insn_lines"],
            fp["ppu"], fp["apu_write"] + fp["apu_read"], fp["oamdma_write"],
            fp["mapper_write"], fp["rom_idx_reads"], fp["ram_writes"],
            fp["calls_out"], fp["loop"], root, r["status"],
            "; ".join(basis) if basis else "—"))
    (OUT_DIR / "tables.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    # ---- stdout ----
    print("== workset inventory ==")
    print("entries=%d segments=%d segment_bytes=%d"
          % (xcheck["seed_entries_total"], len(segments),
             out["universe"]["segment_bytes"]))
    print("roots:", roots)
    print("class:", cls_count)
    print("status:", st_count)
    print("entry_trust:", dict(trust_count))
    print("cross_check:", xcheck)
    print("unmapped L2 entries:", out["l2_unmapped_entries"])
    print("wrote:", OUT_DIR / "subroutines.json")
    print("wrote:", OUT_DIR / "tables.md")
    print("wrote:", MIRROR)
    return 0


if __name__ == "__main__":
    sys.exit(main())
