#!/usr/bin/env python3
# 机械层 remap-table 生成/校验（pipeline §7）：确定性重映射，不做语义判断。
#
# Usage: python3 tools/remap/run.py [--check]
# Input : analysis/ida/bank{00..15}.json（tools/ida/run_all_banks.sh 产出）
# Output: data/naming/remap-table.json（--check 只校验可再生一致，不写文件）
#
# 计数规则（与 docs/spec/completeness.md §3/§4 一致，全部机械）：
#   符号面 = 子程序（IDA 入口，JSR/JMP 目标 + 段起点）∪ 中断向量 ∪ far-call 目标 ∪ bank11 入口表
#   数据面 = 每个被 ≥2 个不同代码 xref 索引的连续数据区段计一表（intra-bank xref；跨 bank 引用走
#            far-call/指针，机械层不计，已注明）+ 已知数据锚点候选单列（不入分母）

import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
IDA_DIR = ROOT / "analysis" / "ida"
OUT = ROOT / "data" / "naming" / "remap-table.json"
ROM = ROOT / "rom" / "gunnac_cn.nes"

BASES = {14: 0xC000, 15: 0xE000, 1: 0xA000, 3: 0xA000, 7: 0xA000, 8: 0xA000, 12: 0xA000}

OPERAND_RE = re.compile(r"\$([0-9A-Fa-f]{1,4})\b")


def base_of(k: int) -> int:
    return BASES.get(k, 0x8000)


def load_banks():
    banks = []
    for k in range(16):
        banks.append(json.loads((IDA_DIR / f"bank{k:02d}.json").read_text(encoding="utf-8")))
    return banks


def build() -> dict:
    banks = load_banks()
    rom_sha = hashlib.sha256(ROM.read_bytes()).hexdigest()

    # ---- vectors (file 0x2000A) ----
    rom = ROM.read_bytes()
    vbytes = rom[0x2000A:0x20010]
    vectors = []
    for name, off in (("NMI", 0), ("RESET", 2), ("IRQ", 4)):
        addr = vbytes[off] | (vbytes[off + 1] << 8)
        vectors.append({
            "name": name, "cpu": f"${addr:04X}", "bank": 15,
            "prg": f"0x{addr - 0xE000 + 0x1E000:05X}",
        })

    # ---- entries: far-call targets + bank11 entry table + vectors ----
    far_calls = []
    for b in banks:
        for fc in b["far_calls"]:
            far_calls.append({
                "bank": b["bank"], "site": f"bank{b['bank']:02d}:{fc['site']:04X}",
                "r6": fc["r6"], "r7": fc["r7"], "target": f"${fc['target']:04X}",
            })
    b11 = banks[11]
    jmp_entries = []
    for item in b11["items"]:
        ea = item["ea"]
        if item["kind"] == "code" and ea < b11["base"] + 0x20:
            m = re.match(r"JMP\s+(?:[a-z_]+)?\$?([0-9A-Fa-f]{4})", item["text"].strip())
            if m:
                jmp_entries.append({"at": f"bank11:{ea:04X}", "target": f"${int(m.group(1), 16):04X}"})

    # ---- symbols: subroutines (mechanical f_NNN) ----
    subs = []
    for b in banks:
        for f in b["funcs"]:
            subs.append({"bank": b["bank"], "start": f["start"], "end": f["end"]})
    subs.sort(key=lambda s: (s["bank"], s["start"]))
    symbols = []
    for i, s in enumerate(subs):
        symbols.append({
            "id": f"f_{i:03d}", "kind": "subroutine", "mech": f"sub_{s['start']:04X}",
            "bank": s["bank"], "cpu": f"${s['start']:04X}",
            "prg": f"0x{s['bank'] * 0x2000 + (s['start'] - base_of(s['bank'])):05X}",
            "size": s["end"] - s["start"],
        })

    # ---- RAM / mapped-register usage (mechanical operand scan) ----
    ram_use: dict[int, int] = {}
    io_use: dict[int, int] = {}
    for b in banks:
        for item in b["items"]:
            if item["kind"] != "code":
                continue
            text = item["text"]
            for m in OPERAND_RE.finditer(text):
                val = int(m.group(1), 16)
                if val < 0x800:
                    ram_use[val] = ram_use.get(val, 0) + 1
                elif 0x2000 <= val <= 0x401F:
                    io_use[val] = io_use.get(val, 0) + 1
    ram_symbols = [{"id": f"v_{a:03X}", "addr": f"${a:04X}", "uses": n}
                   for a, n in sorted(ram_use.items())]
    io_symbols = [{"id": f"r_{a:04X}", "addr": f"${a:04X}", "uses": n}
                  for a, n in sorted(io_use.items())]

    # ---- data runs: contiguous data segments with >=2 distinct intra-bank code xrefs ----
    # t18 重分类规则（t13 遗留修正）：对每条候选 run 做『xref 目标播种线性反汇编』，
    # 统计条件分支目标落在指令边界上的比例（branch-align%）。6502 几乎任意字节都可解码，
    # 纯数据区的分支目标命中率≈43-77%（t_000/t_001/t_006/t_007 实测），真代码区≈84-97%
    # （t_002/t_003/t_004 实测，抽样含 STA $04FC 玩家 Y 初始化等真指令流）。
    # 规则：branch-align% >= 80 ⇒ 判为 mixed_code（混排代码区），移出数据面分母；
    # 同时 bank15 运行段在 $FFFA 截断（向量表 $FFFA-$FFFF 是独立实体，t13 t_007 边界修正）。
    import sys as _sys
    _sys.path.insert(0, str(ROOT / "tools"))
    import da6502  # noqa: E402  表驱动长度解码器（投影工具）
    BRANCH = {"BPL", "BMI", "BVC", "BVS", "BCC", "BCS", "BNE", "BEQ"}

    def branch_align(b, lo, hi, xref_to):
        buf = bytearray(hi - lo)
        for item in b["items"]:
            if item["kind"] == "data":
                raw = bytes.fromhex(item["bytes"])
                ea = item["ea"]
                if lo <= ea < hi:
                    buf[ea - lo:ea - lo + len(raw)] = raw
        seeds = sorted({x for x in xref_to if lo <= x < hi} | {lo})
        starts, branches = set(), []
        for s in seeds:
            i = s - lo
            while i < hi - lo:
                op = buf[i]
                if op in da6502.T:
                    mn, mode = da6502.T[op]
                    n = da6502.L[mode]
                    if i + n > hi - lo:
                        break
                    starts.add(i)
                    if mn in BRANCH and n == 2:
                        branches.append(i + 2 + ((buf[i + 1] ^ 0x80) - 0x80))
                    i += n
                else:
                    starts.add(i)
                    i += 1
        if not branches:
            return 0, 0, 0.0
        hit = sum(1 for t in branches if t in starts)
        return len(branches), hit, 100.0 * hit / len(branches)

    tables, mixed = [], []
    for b in banks:
        base = b["base"]
        xref_to: dict[int, set] = {}
        for x in b["xrefs"]:
            xref_to.setdefault(x["to"], set()).add(x["from"])
        runs = []
        cur = None
        for item in b["items"]:
            if item["kind"] != "data":
                continue
            ea = item["ea"]
            if b["bank"] == 15 and ea >= 0xFFFA:
                continue  # t_007 修正：向量表 $FFFA-$FFFF 不并入数据 run
            size = min(item["size"], 0xFFFA - ea) if b["bank"] == 15 else item["size"]
            if size <= 0:
                continue
            if cur and ea == cur["end"]:
                cur["end"] = ea + size
            else:
                if cur:
                    runs.append(cur)
                cur = {"start": ea, "end": ea + size}
        if cur:
            runs.append(cur)
        for run in runs:
            refs = set()
            for ea in range(run["start"], run["end"]):
                refs |= xref_to.get(ea, set())
            if len(refs) < 2:
                continue
            nb, hit, pct = branch_align(b, run["start"], run["end"], xref_to)
            entry = {
                "id": f"t_{len(tables) + len(mixed):03d}", "bank": b["bank"],
                "cpu": f"${run['start']:04X}-${run['end'] - 1:04X}",
                "prg": f"0x{b['bank'] * 0x2000 + (run['start'] - base):05X}-"
                       f"0x{b['bank'] * 0x2000 + (run['end'] - 1 - base):05X}",
                "size": run["end"] - run["start"], "xref_count": len(refs),
                "branch_align_pct": round(pct, 1),
            }
            if pct >= 80.0:
                entry["classification"] = "mixed_code"
                entry["evidence"] = f"种子反汇编 branch-align {hit}/{nb}={pct:.0f}%（≥80% ⇒ 混排代码区，移出数据面分母）"
                mixed.append(entry)
            else:
                entry["classification"] = "data"
                tables.append(entry)

    # ---- coverage ----
    code_bytes = sum(len(bytes.fromhex(item["bytes"]))
                     for b in banks for item in b["items"] if item["kind"] == "code")
    data_bytes = sum(len(bytes.fromhex(item["bytes"]))
                     for b in banks for item in b["items"] if item["kind"] == "data")

    doc = {
        "schema": 1,
        "layer": "mechanical",
        "generated_by": "tools/remap/run.py",
        "rom_sha256": rom_sha,
        "projection": {
            "decoder": "IDA 9.1 processor M6502 (tools/ida/export_bank.py)",
            "seed": "reference/seed/gunnac.asm",
            "xref_scope": "intra-bank code xrefs（跨 bank 引用经 far-call/指针，机械层不计）",
        },
        "counts": {
            "subroutines": len(symbols),
            "vectors": len(vectors),
            "far_call_sites": len(far_calls),
            "bank11_entry_table": len(jmp_entries),
            "ram_symbols": len(ram_symbols),
            "io_register_symbols": len(io_symbols),
            "data_tables": len(tables),
            "mixed_regions": len(mixed),
            "code_bytes": code_bytes,
            "data_bytes": data_bytes,
        },
        "vectors": vectors,
        "entries": {
            "far_call_sites": far_calls,
            "bank11_jmp_table": jmp_entries,
        },
        "symbols": symbols,
        "ram_symbols": ram_symbols,
        "io_register_symbols": io_symbols,
        "data_tables": tables,
        "mixed_regions": mixed,
        "known_data_candidates": [
            {"name": "stage_page_table（t42 候选）", "prg": "0x04100-0x04220",
             "evidence": "docs/findings/stage-timeline.md §3（$A0-$BF 页值数据表，读者未定位）"},
            {"name": "wave_event_records（t42）", "prg": "0x10D10-0x10DFF",
             "evidence": "docs/findings/stage-timeline.md §2.1（00 FE 分段事件记录流）"},
            {"name": "enemy_spawn_records=t_005（t32）", "prg": "0x10DD0-0x12DFF",
             "evidence": "docs/findings/enemy-dataface.md §1.1（5 字节 {参数×3, 行为脚本指针}）"},
            {"name": "obj_ram_bind_stream=t_001（t32）", "prg": "0x03294-0x03E00",
             "evidence": "docs/findings/enemy-dataface.md §1.2（变长 {参数, RAM 槽指针}）"},
            {"name": "enemy_script_vm_opcodes（t59）", "prg": "0x0044A-0x00584",
             "evidence": "docs/findings/enemy-script-vm.md §1（bank00 字节码机指令集）"},
            {"name": "script_record_loader_4b（t59）", "prg": "0x003F6-0x00432",
             "evidence": "docs/findings/enemy-script-vm.md §2（4 字节记录格式）"},
            {"name": "enemy_script_stream（t32）", "prg": "0x112B4-0x12DFF",
             "evidence": "docs/findings/enemy-dataface.md §3（bank8 行为脚本流，嵌套指针）"},
            {"name": "wave_script_chain（t32 候选）", "prg": "0x10DE0-0x10EBF",
             "evidence": "docs/findings/enemy-dataface.md §3（指针对记录，语义假设 B6）"},
            {"name": "level_config_offsets（t29）", "prg": "0x05457-0x05462",
             "evidence": "docs/findings/shot-behavior.md §1.2（12 级→配置流偏移）"},
            {"name": "shot_param_stream（t29）", "prg": "0x05463-0x054EE",
             "evidence": "docs/findings/shot-behavior.md §1.2（(周期,脚本参) 对流，$FF 分段）"},
            {"name": "obj_motion_speed_x（t29）", "prg": "0x048BB-0x048CE",
             "evidence": "docs/findings/shot-behavior.md §1.3（X 速度 LUT）"},
            {"name": "obj_motion_speed_y（t29）", "prg": "0x048CF-0x048EA",
             "evidence": "docs/findings/shot-behavior.md §1.3（Y 速度 LUT）"},
            {"name": "obj_action_stream（t29）", "prg": "0x0490B-0x0492E",
             "evidence": "docs/findings/shot-behavior.md §1.3（动作流）"},
            {"name": "obj_action_durations（t29）", "prg": "0x0492F-0x04956",
             "evidence": "docs/findings/shot-behavior.md §1.3（时长表，0=跳转）"},
            {"name": "charmap/字形索引表 A", "prg": "0x01190-0x01220",
             "evidence": "docs/findings/rom-forensics.md §4（A 级字节）"},
            {"name": "charmap/字形索引表 B", "prg": "0x01830-0x018F8",
             "evidence": "docs/findings/rom-forensics.md §4"},
            {"name": "文本记录区（标题/版权）", "prg": "0x0109E-0x01123",
             "evidence": "docs/findings/rom-forensics.md §4；记录格式 {PPU地址,tile码,FE/FF}"},
            {"name": "文本记录区（剧情）", "prg": "0x02CDC-0x02F8B",
             "evidence": "docs/findings/rom-forensics.md §4"},
            {"name": "文本记录区（结局）", "prg": "0x02FE4-0x03155",
             "evidence": "docs/findings/rom-forensics.md §4"},
            {"name": "STAFF 字幕区", "prg": "0x01959-0x01BC0",
             "evidence": "docs/findings/rom-forensics.md §4（含 @2011 JIUBAN 签名 0x01BB2）"},
            {"name": "大字模文本块", "prg": "0x1FF00-0x1FFEF",
             "evidence": "docs/findings/rom-forensics.md §4"},
        ],
        "cross_checks_s0": {
            "far_call_sites_18": {"expect": 18, "actual": len(far_calls), "pass": len(far_calls) == 18},
            "bank11_entry_table": {"expect": "bank11:$8000 起 7 条 JMP", "actual": len(jmp_entries),
                                   "pass": len(jmp_entries) == 7},
            "fixed_windows": {"expect": "bank14=$C000 bank15=$E000",
                              "actual": f"bank14 base=0x{base_of(14):X} bank15 base=0x{base_of(15):X}",
                              "pass": base_of(14) == 0xC000 and base_of(15) == 0xE000},
            "vectors": {"expect": "NMI=$EF85 RESET=$E000 IRQ=$F20B",
                        "actual": ", ".join(f"{v['name']}={v['cpu']}" for v in vectors),
                        "pass": [v["cpu"] for v in vectors] == ["$EF85", "$E000", "$F20B"]},
        },
    }
    return doc


def main() -> None:
    doc = build()
    text = json.dumps(doc, ensure_ascii=False, indent=2) + "\n"
    if "--check" in sys.argv:
        old = OUT.read_text(encoding="utf-8") if OUT.is_file() else ""
        print("CHECK:", "一致" if old == text else "不一致（需重新生成）")
        return
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    c = doc["counts"]
    print(f"wrote {OUT.relative_to(ROOT)}: subroutines={c['subroutines']} vectors={c['vectors']} "
          f"far_calls={c['far_call_sites']} tables={c['data_tables']} ram={c['ram_symbols']} "
          f"io={c['io_register_symbols']} code={c['code_bytes']}B data={c['data_bytes']}B")
    for name, chk in doc["cross_checks_s0"].items():
        print(f"  cross-check {name}: {'PASS' if chk['pass'] else 'FAIL'} ({chk['actual']})")


if __name__ == "__main__":
    main()
