#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/cleanup-worktrees.sh list
  scripts/cleanup-worktrees.sh status [PATH]
  scripts/cleanup-worktrees.sh stale
  scripts/cleanup-worktrees.sh cleanup PATH
  scripts/cleanup-worktrees.sh prune-stale

Commands:
  list       Show all registered worktrees for the current repository.
  status     Show current metadata for all worktrees, or classify one PATH.
  stale      List stale worktree registrations (recorded but missing on disk).
  cleanup    Validate PATH and remove it as a registered worktree only if safe.
  prune-stale
             Prune stale worktree registrations from git metadata.
USAGE
}

repo_root="$(git -C "$PWD" rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$repo_root" ]]; then
  echo "scripts/cleanup-worktrees.sh must be run inside an adze git repository"
  exit 1
fi

resolve_path() {
  local path="$1"
  local candidate
  if [[ "$path" = /* ]]; then
    candidate="$path"
  else
    candidate="$repo_root/$path"
  fi
  if command -v realpath >/dev/null 2>&1; then
    realpath -m "$candidate"
  else
    printf '%s\n' "$candidate"
  fi
}

worktree_records() {
  git -C "$repo_root" worktree list --porcelain
}

is_registered() {
  local target="$1"
  local line
  while IFS= read -r line; do
    if [[ "$line" == worktree* ]]; then
      local worktree_path="${line#worktree }"
      if [[ "$worktree_path" == "$target" ]]; then
        return 0
      fi
    fi
  done < <(worktree_records)
  return 1
}

git_metadata_state() {
  local path="$1"
  if [[ -f "$path/.git" ]]; then
    if grep -q '^gitdir:' "$path/.git"; then
      printf 'linked-worktree\n'
    else
      printf 'git-file-having-nonstandard-format\n'
    fi
    return 0
  fi
  if [[ -d "$path/.git" ]]; then
    printf 'standalone-repo\n'
    return 0
  fi
  printf 'unknown\n'
}

list_worktrees() {
  echo "Registered worktrees:"
  worktree_records
}

status_all() {
  echo "Registered worktrees:"
  worktree_records
  echo
  echo "Stale registrations:"
  stale_worktrees
}

clean_recommendation() {
  local path="$1"
  local state="$2"
  local registered="$3"
  if [[ "$path" == "$repo_root" ]]; then
    echo "do not remove git repository root with this helper"
    return
  fi
  if [[ "$state" == "standalone-repo" ]]; then
    echo "rm -rf \"$path\""
  elif [[ "$state" == "linked-worktree" && "$registered" == "yes" ]]; then
    echo "./scripts/cleanup-worktrees.sh cleanup \"$path\""
  elif [[ "$state" == "linked-worktree" ]]; then
    echo "Inspect path metadata before attempting cleanup"
  else
    echo "inspect $path/.git before cleanup"
  fi
}

status_path() {
  local path="$1"
  local state
  state="$(git_metadata_state "$path")"
  local registered="no"
  if is_registered "$path"; then
    registered="yes"
  fi
  local recommendation
  recommendation="$(clean_recommendation "$path" "$state" "$registered")"
  echo "Path: $path"
  if [[ "$registered" == "yes" ]]; then
    echo "  worktree state: registered in current git metadata"
  else
    echo "  worktree state: not registered in current git metadata"
  fi
  echo "  git metadata: $state"
  echo "  cleanup recommendation: $recommendation"
}

stale_worktrees() {
  local line worktree_path found=0
  while IFS= read -r line; do
    if [[ "$line" == worktree* ]]; then
      worktree_path="${line#worktree }"
      if [[ ! -d "$worktree_path" ]]; then
        echo "  $worktree_path"
        found=1
      fi
    fi
  done < <(worktree_records)

  if [[ "$found" == 0 ]]; then
    echo "  none"
  fi
}

cleanup_path() {
  local path="$1"
  if [[ "$path" == "$repo_root" ]]; then
    echo "Refusing to remove repository root: $path"
    exit 1
  fi

  if ! is_registered "$path"; then
    echo "Path is not a registered worktree: $path"
    echo "Use 'rm -rf \"$path\"' only if this is a standalone temporary checkout."
    exit 1
  fi

  case "$(git_metadata_state "$path")" in
    standalone-repo)
      echo "Refusing safe cleanup because .git is a directory: $path"
      echo "Use 'rm -rf \"$path\"' after ensuring no important files remain."
      exit 1
      ;;
    linked-worktree)
      ;;
    *)
      echo "Unexpected git metadata format at $path/.git"
      exit 1
      ;;
  esac

  echo "Removing registered worktree: $path"
  git -C "$repo_root" worktree remove --force "$path"
  echo "Pruning stale worktree metadata:"
  git -C "$repo_root" worktree prune
}

stale_prune() {
  stale_worktrees
  echo "Running git worktree prune..."
  git -C "$repo_root" worktree prune
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

cmd="$1"
shift

case "$cmd" in
  list)
    list_worktrees
    ;;
  status)
  if [[ $# -eq 0 ]]; then
      status_all
    else
      status_path "$(resolve_path "$1")"
    fi
    ;;
  stale)
    stale_worktrees
    ;;
  cleanup)
    if [[ $# -ne 1 ]]; then
      echo "cleanup requires one PATH argument"
      usage
      exit 1
    fi
    cleanup_path "$(resolve_path "$1")"
    ;;
  prune-stale)
    stale_prune
    ;;
  *)
    usage
    exit 1
    ;;
esac
