#!/usr/bin/env bash
# Install script for adze git hooks
# Sets up symlinks from .git/hooks to .githooks for version-controlled hooks
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Set locale to avoid warnings
export LC_ALL=C.UTF-8
export LANG=C.UTF-8

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GIT_HOOKS_DIR="$REPO_ROOT/.git/hooks"
GITHOOKS_DIR="$REPO_ROOT/.githooks"

echo -e "${BLUE}adze Git Hooks Installer${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Check if we're in a git repository
if [ ! -d "$REPO_ROOT/.git" ]; then
    echo -e "${RED}✖ Error: Not in a git repository${NC}"
    echo "This script must be run from within the adze repository"
    exit 1
fi

echo -e "${YELLOW}→${NC} Repository root: $REPO_ROOT"
echo -e "${YELLOW}→${NC} Git hooks directory: $GIT_HOOKS_DIR" 
echo -e "${YELLOW}→${NC} Versioned hooks directory: $GITHOOKS_DIR"
echo ""

# Ensure the .githooks directory exists
if [ ! -d "$GITHOOKS_DIR" ]; then
    echo -e "${RED}✖ Error: .githooks directory not found${NC}"
    echo "Expected to find hooks at: $GITHOOKS_DIR"
    exit 1
fi

# Function to install a single hook
install_hook() {
    local hook_name="$1"
    local source_hook="$GITHOOKS_DIR/$hook_name"
    local target_hook="$GIT_HOOKS_DIR/$hook_name"
    
    if [ ! -f "$source_hook" ]; then
        echo -e "${YELLOW}⚠${NC} No $hook_name hook found in .githooks/, skipping"
        return
    fi
    
    # Check if hook is executable
    if [ ! -x "$source_hook" ]; then
        echo -e "${YELLOW}→${NC} Making $hook_name executable"
        chmod +x "$source_hook"
    fi
    
    # Handle existing hooks
    if [ -f "$target_hook" ] || [ -L "$target_hook" ]; then
        if [ -L "$target_hook" ]; then
            local existing_target
            existing_target=$(readlink "$target_hook")
            if [ "$existing_target" = "$source_hook" ]; then
                echo -e "${GREEN}✓${NC} $hook_name already correctly linked"
                return
            else
                echo -e "${YELLOW}→${NC} Removing old symlink for $hook_name (pointed to $existing_target)"
                rm "$target_hook"
            fi
        else
            echo -e "${YELLOW}→${NC} Backing up existing $hook_name hook"
            mv "$target_hook" "$target_hook.backup.$(date +%Y%m%d_%H%M%S)"
        fi
    fi
    
    # Create the symlink
    echo -e "${YELLOW}→${NC} Installing $hook_name hook"
    ln -s "$source_hook" "$target_hook"
    echo -e "${GREEN}✓${NC} $hook_name hook installed"
}

# Install available hooks
echo -e "${BLUE}Installing available hooks:${NC}"
echo ""

# List of hooks to check for and install
HOOK_TYPES=(
    "pre-commit"
    "pre-push"
    "commit-msg"
    "pre-rebase"
    "post-merge"
)

INSTALLED_COUNT=0
for hook in "${HOOK_TYPES[@]}"; do
    if install_hook "$hook"; then
        INSTALLED_COUNT=$((INSTALLED_COUNT + 1))
    fi
done

echo ""

# Verify installation
echo -e "${BLUE}Verifying installation:${NC}"
echo ""

for hook in "${HOOK_TYPES[@]}"; do
    source_hook="$GITHOOKS_DIR/$hook"
    target_hook="$GIT_HOOKS_DIR/$hook"
    
    if [ -f "$source_hook" ]; then
        if [ -L "$target_hook" ]; then
            link_target=$(readlink "$target_hook")
            if [ "$link_target" = "$source_hook" ]; then
                echo -e "${GREEN}✓${NC} $hook: correctly linked"
            else
                echo -e "${RED}✖${NC} $hook: linked to wrong target ($link_target)"
            fi
        else
            echo -e "${RED}✖${NC} $hook: not linked (file exists but is not a symlink)"
        fi
    fi
done

echo ""
echo -e "${GREEN}✅ Git hooks installation complete${NC}"
echo ""

# Show usage information
echo -e "${BLUE}Usage Information:${NC}"
echo ""
echo "The following hooks are now active:"
echo ""

if [ -f "$GITHOOKS_DIR/pre-commit" ]; then
    echo -e "${YELLOW}pre-commit:${NC}"
    echo "  • Detects partially staged files (fails if found)"
    echo "  • Formats only staged Rust files (not all files)"
    echo "  • Runs clippy with full diagnostics output"
    echo "  • Validates GOTO indexing patterns"
    echo "  • Checks for disabled test files (.rs.disabled)"
    echo "  • Verifies test connectivity"
    echo ""
fi

echo -e "${BLUE}Environment Variables:${NC}"
echo ""
echo "  RUN_QUICK_TESTS=1    Enable quick invariant tests in pre-commit"
echo ""

echo -e "${BLUE}Key Improvements:${NC}"
echo ""
echo "  ✓ Partial staging detection prevents format conflicts"
echo "  ✓ Targeted formatting (staged files only)"
echo "  ✓ Full diagnostic output from all checks"
echo "  ✓ Version-controlled hooks in .githooks/"
echo "  ✓ Preserved locale fixes and colored output"
echo ""

echo -e "${YELLOW}To disable a hook temporarily:${NC}"
echo "  chmod -x .git/hooks/hook-name"
echo ""
echo -e "${YELLOW}To uninstall all hooks:${NC}"
echo "  rm .git/hooks/pre-commit .git/hooks/pre-push  # etc."
echo ""
echo "Happy coding! 🦀"