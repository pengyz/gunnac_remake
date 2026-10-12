#!/usr/bin/env bash
# S1: build per-bank IDA databases and export mechanical JSON (projection generator; D3: ROM is truth).
# Usage: tools/ida/run_all_banks.sh   (from repo root; idat must exist at $IDAT)
# Output: analysis/ida/bankNN.bin / bankNN.json / bankNN.log  (regenerable, not in repo)
set -euo pipefail
IDAT="${IDAT:-/home/peng/ida-pro-9.1/idat}"
ROM="${ROM:-rom/gunnac_cn.nes}"
OUTDIR="${OUTDIR:-analysis/ida}"
mkdir -p "$OUTDIR"

# canonical CPU base per bank (evidence in docs/findings/rom-forensics.md §2; unobserved banks use $8000 convention)
base_for() {
  case "$1" in
    14) echo 0xC000 ;;
    15) echo 0xE000 ;;
    1|3|7|8|12) echo 0xA000 ;;
    *) echo 0x8000 ;;
  esac
}

python3 - "$ROM" "$OUTDIR" <<'EOF'
import sys
rom, outdir = sys.argv[1], sys.argv[2]
d = open(rom, 'rb').read()
prg = d[0x10:0x20010]
for k in range(16):
    open(f"{outdir}/bank{k:02d}.bin", "wb").write(prg[k*0x2000:(k+1)*0x2000])
print("extracted 16 bank bins")
EOF

for k in $(seq -w 0 15); do
  kn=$((10#$k))
  base=$(base_for "$kn")
  rm -f "$OUTDIR/bank$k.bin.i64" "$OUTDIR/bank$k.bin.id0" "$OUTDIR/bank$k.bin.id1" \
        "$OUTDIR/bank$k.bin.nam" "$OUTDIR/bank$k.bin.til"
  S1_BANK="$kn" S1_BASE="$base" S1_OUT="$PWD/$OUTDIR/bank$k.json" \
    "$IDAT" -A -pm6502 -S"$PWD/tools/ida/export_bank.py" -L"$PWD/$OUTDIR/bank$k.log" \
    "$PWD/$OUTDIR/bank$k.bin" >/dev/null 2>&1 || true
  grep -h "S1 export" "$OUTDIR/bank$k.log" || echo "bank$k: EXPORT MISSING (see $OUTDIR/bank$k.log)"
done
