#!/usr/bin/env bash
# Workaround for just(1) runtime-dir permission error:
#   error: I/O error in runtime dir `/run/user/1000/just`: Permission denied (os error 13)
#
# Occurs when XDG_RUNTIME_DIR points to a non-existent or unwritable directory
# (common in containers, CI, and shared dev environments).
#
# Usage:
#   source scripts/just-ensure-tmpdir.sh   # then run `just` normally
#   # or:
#   eval "$(scripts/just-ensure-tmpdir.sh)" && just ci-supported

if [ -n "${XDG_RUNTIME_DIR:-}" ] && [ ! -d "$XDG_RUNTIME_DIR" ]; then
  fallback="/tmp/just-${USER:-$(id -u)}"
  mkdir -p "$fallback"
  export JUST_TEMPDIR="$fallback"
fi

# If JUST_TEMPDIR is already set but dir doesn't exist, create it
if [ -n "${JUST_TEMPDIR:-}" ] && [ ! -d "$JUST_TEMPDIR" ]; then
  mkdir -p "$JUST_TEMPDIR"
fi
