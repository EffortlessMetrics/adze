#!/usr/bin/env bash
set -euo pipefail

# Keep cargo output small inside Claude
export CARGO_TERM_COLOR=never
export CARGO_TERM_PROGRESS=never
: "${RUSTFLAGS:=-Awarnings}"
export RUSTFLAGS

# Capture output to file, show small window
log="${TMPDIR:-/tmp}/cargo.$$.log"
status=0
cargo "$@" -q >"$log" 2>&1 || status=$?

total=$(wc -l <"$log" | tr -d ' ')
head_n=160
tail_n=160
echo "── cargo output (head) ─────────────────────────────────────────"
sed -n "1,${head_n}p" "$log" | sed 's/^/│ /'
if [ "$total" -gt $((head_n + tail_n)) ]; then
  echo "…"
fi
echo "── cargo output (tail) ─────────────────────────────────────────"
sed -n "$(( total - tail_n + 1 )),\$p" "$log" | sed 's/^/│ /' || true

exit "$status"