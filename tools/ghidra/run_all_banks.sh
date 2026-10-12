#!/usr/bin/env bash
# S1: Ghidra analyzeHeadless per-bank 6502 C pseudocode export (projection generator; D3: ROM is truth).
# Usage: tools/ghidra/run_all_banks.sh   (from repo root)
# Output: analysis/ghidra/bankNN.c + project analysis/ghidra/proj (regenerable, not in repo)
set -euo pipefail
GHIDRA="${GHIDRA:-analysis/tools_dl/ghidra}"
HEADLESS="$GHIDRA/support/analyzeHeadless"
OUTDIR="${OUTDIR:-analysis/ghidra}"
PROJ="$PWD/analysis/ghidra/proj"
mkdir -p "$OUTDIR"

base_for() {
  case "$1" in
    14) echo 0xC000 ;;
    15) echo 0xE000 ;;
    1|3|7|8|12) echo 0xA000 ;;
    *) echo 0x8000 ;;
  esac
}

for k in $(seq -w 0 15); do
  kn=$((10#$k))
  base=$(base_for "$kn")
  echo "== ghidra bank$k base=$base =="
  "$HEADLESS" "$PROJ" s1 \
    -import "$PWD/analysis/ida/bank$k.bin" \
    -loader BinaryLoader -processor "6502:LE:16:default" -loader-baseAddr "$base" \
    -scriptPath "$PWD/tools/ghidra" \
    -postScript ExportPseudo.java "$PWD/$OUTDIR/bank$k.c" \
    -analysisTimeoutPerFile 600 >/dev/null 2>&1 || echo "bank$k: headless rc=$?"
  test -s "$OUTDIR/bank$k.c" && echo "bank$k: OK ($(wc -l < "$OUTDIR/bank$k.c") lines)" || echo "bank$k: OUTPUT MISSING"
done
