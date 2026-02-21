#!/bin/bash
# Safe command runner for adze CI/automation
# Eliminates EAGAIN issues and provides process group management

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCK_DIR="${SCRIPT_DIR}/../.locks"

# Create lock directory if it doesn't exist
mkdir -p "${LOCK_DIR}"

# Cleanup function
cleanup() {
    local exit_code=$?
    if [[ -n "${CHILD_PGID:-}" ]]; then
        echo "Cleaning up process group: ${CHILD_PGID}"
        kill -TERM "${CHILD_PGID}" 2>/dev/null || true
        sleep 1
        kill -KILL "${CHILD_PGID}" 2>/dev/null || true
    fi
    exit $exit_code
}

trap cleanup EXIT INT TERM

# Run command with process group management
run_with_pgid() {
    local cmd="$1"
    shift
    local args=("$@")
    local timeout_sec="${TIMEOUT_SEC:-1800}"  # 30 minutes default
    
    echo "Running: $cmd ${args[*]}"
    echo "Timeout: ${timeout_sec}s"
    
    # Start command in new process group
    setsid "$cmd" "${args[@]}" &
    local child_pid=$!
    CHILD_PGID=-$child_pid  # Process group ID (negative)
    
    echo "Started process group: ${CHILD_PGID} (PID: ${child_pid})"
    
    # Wait with timeout
    local count=0
    while kill -0 "$child_pid" 2>/dev/null; do
        if (( count >= timeout_sec )); then
            echo "Timeout reached (${timeout_sec}s), killing process group"
            kill -TERM "${CHILD_PGID}" 2>/dev/null || true
            sleep 2
            kill -KILL "${CHILD_PGID}" 2>/dev/null || true
            exit 124  # timeout exit code
        fi
        sleep 1
        ((count++))
        
        # Progress indicator every 60 seconds
        if (( count % 60 == 0 )); then
            echo "Still running... (${count}/${timeout_sec}s)"
        fi
    done
    
    # Get exit code
    wait "$child_pid"
    local exit_code=$?
    CHILD_PGID=""  # Clear to prevent cleanup
    
    echo "Process completed with exit code: $exit_code"
    return $exit_code
}

# EAGAIN retry wrapper
run_with_retry() {
    local max_retries=3
    local retry_delay=2
    
    for attempt in $(seq 1 $max_retries); do
        echo "Attempt $attempt/$max_retries"
        
        if run_with_pgid "$@"; then
            return 0
        fi
        
        local exit_code=$?
        
        # Check if this was an EAGAIN-related failure
        # (exit code 1 from fork failures, resource exhaustion, etc.)
        if [[ $exit_code -eq 1 && $attempt -lt $max_retries ]]; then
            echo "Command failed (possibly EAGAIN), retrying in ${retry_delay}s..."
            sleep $retry_delay
            retry_delay=$((retry_delay * 2))  # exponential backoff
            continue
        fi
        
        return $exit_code
    done
}

# Global locking for agent debouncing
with_lock() {
    local lock_name="$1"
    shift
    local lock_file="${LOCK_DIR}/${lock_name}.lock"
    local max_wait=300  # 5 minutes
    local wait_time=0
    local check_interval=1
    
    echo "Acquiring lock: $lock_name"
    
    while [[ $wait_time -lt $max_wait ]]; do
        if (set -C; echo "$$:$(date):$*" > "$lock_file") 2>/dev/null; then
            echo "Lock acquired: $lock_name"
            
            # Ensure lock is cleaned up
            trap "rm -f '$lock_file'; cleanup" EXIT INT TERM
            
            # Run the command
            run_with_retry "$@"
            local exit_code=$?
            
            # Release lock
            rm -f "$lock_file"
            echo "Lock released: $lock_name"
            
            return $exit_code
        fi
        
        # Check if lock is stale (older than 5 minutes)
        if [[ -f "$lock_file" ]]; then
            local lock_age
            lock_age=$(( $(date +%s) - $(stat -c %Y "$lock_file" 2>/dev/null || echo 0) ))
            if [[ $lock_age -gt 300 ]]; then
                echo "Removing stale lock: $lock_file (age: ${lock_age}s)"
                rm -f "$lock_file"
                continue
            fi
            
            echo "Lock held by: $(cat "$lock_file" 2>/dev/null || echo "unknown")"
        fi
        
        echo "Waiting for lock: $lock_name (${wait_time}/${max_wait}s)"
        sleep $check_interval
        wait_time=$((wait_time + check_interval))
    done
    
    echo "Failed to acquire lock $lock_name after ${max_wait}s"
    return 1
}

# Agent runner with debouncing
run_agent() {
    local agent_name="$1"
    local lock_name="agent-${agent_name}"
    
    echo "Running Claude agent: $agent_name"
    
    # This would be replaced with actual Claude agent invocation
    # For now, just simulate the agent workflow
    with_lock "$lock_name" echo "Agent $agent_name would run here"
    
    echo "Agent $agent_name completed"
}

# Cleanup stale locks
cleanup_locks() {
    echo "Cleaning up stale locks..."
    local cleaned=0
    
    if [[ -d "$LOCK_DIR" ]]; then
        find "$LOCK_DIR" -name "*.lock" -type f -mmin +5 | while IFS= read -r lock_file; do
            local age_min
            age_min=$(( ($(date +%s) - $(stat -c %Y "$lock_file")) / 60 ))
            echo "Removing stale lock: $(basename "$lock_file") (age: ${age_min}min)"
            rm -f "$lock_file"
            ((cleaned++)) || true
        done
    fi
    
    echo "Cleaned $cleaned stale locks"
}

# Main CLI interface
main() {
    case "${1:-}" in
        "run")
            shift
            run_with_retry "$@"
            ;;
        "run-with-lock")
            local lock_name="$2"
            shift 2
            with_lock "$lock_name" "$@"
            ;;
        "agent")
            local agent_name="${2:-}"
            if [[ -z "$agent_name" ]]; then
                echo "Usage: $0 agent <agent-name>"
                exit 1
            fi
            run_agent "$agent_name"
            ;;
        "cleanup-locks")
            cleanup_locks
            ;;
        "help"|"--help"|"-h")
            cat << EOF
Usage: $0 <command> [args...]

Safe command runner for adze CI/automation.
Provides process group management, EAGAIN handling, and agent debouncing.

Commands:
  run <cmd> [args...]              - Run command with robust process management
  run-with-lock <name> <cmd> [args] - Run command with global lock
  agent <agent-name>               - Run Claude agent with debouncing
  cleanup-locks                    - Clean up stale locks

Environment Variables:
  TIMEOUT_SEC                      - Command timeout in seconds (default: 1800)

Examples:
  $0 run cargo test -p adze-python
  $0 run-with-lock rust-build cargo build --workspace
  $0 agent pr-cleanup-reviewer
  $0 cleanup-locks
EOF
            ;;
        *)
            echo "Unknown command: ${1:-}"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

main "$@"