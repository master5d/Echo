#!/usr/bin/env bash
# Magenta Jam Lab — Phase 0 feasibility bench (run ON the Mac, Apple Silicon).
# Gate G0: steady-state RTF < 1.0 AND no swap growth during generation.
# Usage:  bash magenta-phase0-bench.sh [model] [duration_s]
#   model: mrt2_small (default) | mrt2_base
#   duration_s: audio seconds to generate for the timed run (default 30)
set -uo pipefail

MODEL="${1:-mrt2_small}"
DUR="${2:-30}"
LOG="$HOME/magenta-phase0-$(date +%Y%m%d-%H%M%S).log"
OUTDIR="$HOME/magenta-phase0-out"
mkdir -p "$OUTDIR"

say() { echo "[$(date +%H:%M:%S)] $*" | tee -a "$LOG"; }

say "=== Magenta Phase 0 bench — model=$MODEL duration=${DUR}s ==="
say "host: $(scutil --get ComputerName 2>/dev/null) | $(uname -m) | macOS $(sw_vers -productVersion)"
say "chip/mem: $(sysctl -n machdep.cpu.brand_string 2>/dev/null) | $(( $(sysctl -n hw.memsize)/1073741824 )) GB"

# --- 1. toolchain ---
if ! command -v uv >/dev/null 2>&1; then
  say "installing uv..."
  curl -LsSf https://astral.sh/uv/install.sh | sh >>"$LOG" 2>&1
  export PATH="$HOME/.local/bin:$PATH"
fi
say "uv: $(uv --version 2>&1)"

cd "$HOME"
[ -d magenta-rt-lab ] || mkdir magenta-rt-lab
cd magenta-rt-lab
[ -d .venv ] || uv venv --python 3.12 >>"$LOG" 2>&1
# shellcheck disable=SC1091
source .venv/bin/activate
say "python: $(python --version 2>&1)"

# --- 2. install + fetch weights (idempotent) ---
say "installing magenta-rt[mlx] (first run downloads a lot)..."
uv pip install "magenta-rt[mlx]" >>"$LOG" 2>&1 || { say "PIP INSTALL FAILED — see $LOG"; exit 1; }
say "mrt models init/download ($MODEL)..."
mrt models init >>"$LOG" 2>&1
mrt models download >>"$LOG" 2>&1 || say "WARN: 'mrt models download' returned nonzero (may download lazily on first generate)"

# --- 3. warm-up run (pays one-time .mlxfn compile / model load) ---
say "warm-up generate (compiles .mlxfn; not timed)..."
mrt mlx generate --prompt "warm up" --duration 4.0 --model="$MODEL" \
  --output "$OUTDIR/warmup.wav" >>"$LOG" 2>&1 || { say "WARMUP GENERATE FAILED — see $LOG"; exit 1; }

# --- 4. swap baseline ---
swap_used() { sysctl -n vm.swapusage | sed -n 's/.*used = \([0-9.]*\)M.*/\1/p'; }
SWAP_BEFORE="$(swap_used)"; SWAP_BEFORE="${SWAP_BEFORE:-0}"
say "swap before timed run: ${SWAP_BEFORE} MB"

# --- 5. timed steady-state run ---
say "TIMED generate: ${DUR}s of audio (prompt='disco funk')..."
START=$(python -c 'import time;print(time.time())')
/usr/bin/time -l mrt mlx generate --prompt "disco funk" --duration "$DUR" --model="$MODEL" \
  --output "$OUTDIR/timed.wav" >>"$LOG" 2>&1
RC=$?
END=$(python -c 'import time;print(time.time())')

if [ $RC -ne 0 ]; then say "TIMED GENERATE FAILED (rc=$RC) — see $LOG"; exit 1; fi

ELAPSED=$(python -c "print(f'{$END-$START:.2f}')")
RTF=$(python -c "print(f'{($END-$START)/$DUR:.3f}')")
MAXRSS_MB=$(grep -i "maximum resident set size" "$LOG" | tail -1 | awk '{print int($1/1048576)}')
SWAP_AFTER="$(swap_used)"; SWAP_AFTER="${SWAP_AFTER:-0}"
SWAP_DELTA=$(python -c "print(f'{$SWAP_AFTER-$SWAP_BEFORE:.1f}')")

say "----------------------------------------------------------------"
say "RESULT: elapsed=${ELAPSED}s for ${DUR}s audio  ->  RTF=${RTF}"
say "peak RSS: ${MAXRSS_MB:-?} MB | swap delta: ${SWAP_DELTA} MB (before ${SWAP_BEFORE} / after ${SWAP_AFTER})"

# --- 6. verdict (Gate G0) ---
PASS_RTF=$(python -c "print(1 if $RTF < 1.0 else 0)")
PASS_SWAP=$(python -c "print(1 if $SWAP_DELTA <= 50 else 0)")  # <=50MB growth tolerated
if [ "$PASS_RTF" = 1 ] && [ "$PASS_SWAP" = 1 ]; then
  say "GATE G0: ✅ PASS  (RTF<1.0 and no meaningful swap growth) — green light for Phase 1"
else
  say "GATE G0: ❌ FAIL  (RTF<1.0=${PASS_RTF}, swap-ok=${PASS_SWAP}) — park or try smaller/again"
fi
say "audio: $OUTDIR/timed.wav | full log: $LOG"
