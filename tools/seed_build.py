#!/usr/bin/env python3
# Assemble the normalized frozen disassembly seed from IDA bank exports (D3: projection, ROM is truth).
#
# Usage: python3 tools/seed_build.py
# Input : analysis/ida/bank{00..15}.json  (produced by tools/ida/run_all_banks.sh)
# Output: reference/seed/gunnac.asm       (deterministic: byte-identical on re-run)
#
# Normalization rules (fixed for reproducibility):
#   - one line per IDA item: "bankNN:XXXX  <hex bytes>  <disasm or .byte ...>"
#   - data items emitted as up to 8 ".byte $XX" per line
#   - no timestamps / absolute paths in output

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
IDA_DIR = ROOT / "analysis" / "ida"
OUT = ROOT / "reference" / "seed" / "gunnac.asm"
ROM = ROOT / "rom" / "gunnac_cn.nes"

# canonical CPU base per bank (window evidence: docs/findings/rom-forensics.md §2)
BASES = {14: 0xC000, 15: 0xE000, 1: 0xA000, 3: 0xA000, 7: 0xA000, 8: 0xA000, 12: 0xA000}
WINDOW_EVIDENCE = {
    0: "R6=$00（$E60C 初始化）",
    1: "R7=$01",
    2: "R6=$02",
    3: "R7=$03",
    7: "R7=$07（bank15:$F25B）",
    8: "R7=$08（bank0 起始）",
    11: "R6=$0B（bank11 $8000 入口 JMP 表）",
    12: "R7=$0C",
    14: "固定第二末 bank（mode 0，$C000）",
    15: "固定末 bank（mode 0，$E000）",
}


def base_of(k: int) -> int:
    return BASES.get(k, 0x8000)


def main() -> None:
    rom_sha = hashlib.sha256(ROM.read_bytes()).hexdigest()
    lines = []
    lines.append("; =====================================================================")
    lines.append("; gunnac_cn.nes — normalized 6502 disassembly seed (FROZEN projection, D3)")
    lines.append("; =====================================================================")
    lines.append(f"; rom sha256: {rom_sha}")
    lines.append("; decoder   : IDA 9.1 processor M6502 (tools/ida/export_bank.py, linear sweep)")
    lines.append("; regen     : tools/ida/run_all_banks.sh && python3 tools/seed_build.py")
    lines.append("; freeze    : reference/seed/ 一经冻结不可变；重生成需 commit 说明（AGENTS.md §2）")
    lines.append("; windows   : R6->$8000-$9FFF  R7->$A000-$BFFF  bank14->$C000  bank15->$E000 (mode 0)")
    lines.append("; vectors   : NMI=$EF85 RESET=$E000 IRQ=$F20B (bank15 fixed; file 0x2000A)")
    lines.append("; NOTE      : bankNN:XXXX = bank 内规范 CPU 地址（窗口证据见各行注释；无证据者约定 $8000）")
    lines.append("")

    totals = {"items": 0, "funcs": 0, "xrefs": 0, "far_calls": 0}
    for k in range(16):
        data = json.loads((IDA_DIR / f"bank{k:02d}.json").read_text(encoding="utf-8"))
        assert data["bank"] == k and data["base"] == base_of(k), f"bank{k:02d} base mismatch"
        prg_off = k * 0x2000
        ev = WINDOW_EVIDENCE.get(k, "窗口无证据（约定 $8000）")
        lines.append(f"; ==== PRG bank {k:02d} | PRG 0x{prg_off:05X} | CPU ${base_of(k):04X}-"
                     f"${base_of(k) + 0x1FFF:04X} | {ev} ====")
        for item in data["items"]:
            ea = item["ea"]
            raw = bytes.fromhex(item["bytes"])
            if item["kind"] == "code":
                hx = " ".join(f"{b:02X}" for b in raw)
                lines.append(f"bank{k:02d}:{ea:04X}  {hx:<12s}  {item['text']}")
            else:
                for off in range(0, len(raw), 8):
                    chunk = raw[off:off + 8]
                    hx = " ".join(f"{b:02X}" for b in chunk)
                    lines.append(f"bank{k:02d}:{ea + off:04X}  {hx:<12s}  "
                                 + ",".join(f"${b:02X}" for b in chunk))
        totals["items"] += len(data["items"])
        totals["funcs"] += len(data["funcs"])
        totals["xrefs"] += len(data["xrefs"])
        totals["far_calls"] += len(data["far_calls"])
        lines.append("")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    n_lines = sum(1 for _ in open(OUT, encoding="utf-8"))
    print(f"wrote {OUT.relative_to(ROOT)} lines={n_lines} "
          f"items={totals['items']} funcs={totals['funcs']} "
          f"xrefs={totals['xrefs']} far_calls={totals['far_calls']}")


if __name__ == "__main__":
    main()
