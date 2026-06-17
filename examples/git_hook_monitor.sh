#!/bin/bash
# Git post-commit hook for quality monitoring.
#
# Installation:
#   cp examples/git_hook_monitor.sh .git/hooks/post-commit
#   chmod +x .git/hooks/post-commit
#
# This hook runs after every commit and exits with:
# - 0 (success) if no CRITICAL quality violations
# - 1 (failure) if CRITICAL violations detected
#
# The output is JSON for integration with CI/CD systems.

set -euo pipefail

# Configuration
METRICS="stubs,types,churn,comments,complexity,clippy"
RULES="all"
BASELINE_COMMITS="20"
INTERVAL="5"

# Colors for terminal output (only if stderr is a TTY)
if [ -t 2 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
fi

# Run the monitor
MONITOR_OUTPUT=$(
    affi receipt monitor \
        --watch . \
        --metrics "$METRICS" \
        --rules "$RULES" \
        --baseline-commits "$BASELINE_COMMITS" \
        --interval "$INTERVAL" \
        --output stderr,json \
        --format json 2>&1 || true
)

# Parse violations from JSON output
if echo "$MONITOR_OUTPUT" | grep -q '"violations"'; then
    # Extract violations array
    VIOLATIONS=$(echo "$MONITOR_OUTPUT" | jq '.violations // []' 2>/dev/null || echo '[]')
    VIOLATION_COUNT=$(echo "$VIOLATIONS" | jq 'length' 2>/dev/null || echo 0)

    if [ "$VIOLATION_COUNT" -gt 0 ]; then
        # Check for CRITICAL violations
        CRITICAL_COUNT=$(echo "$VIOLATIONS" | jq 'map(select(.severity == "CRITICAL")) | length' 2>/dev/null || echo 0)

        if [ "$CRITICAL_COUNT" -gt 0 ]; then
            echo -e "${RED}[HOOK FAILURE] Quality check failed: $CRITICAL_COUNT CRITICAL violations${NC}" >&2
            echo "$MONITOR_OUTPUT" | jq . >&2
            exit 1
        else
            # Warnings only (HIGH, MEDIUM, INFO)
            echo -e "${YELLOW}[HOOK WARNING] Quality check passed but found $VIOLATION_COUNT non-critical violations${NC}" >&2
            echo "$MONITOR_OUTPUT" | jq . >&2
            exit 0
        fi
    else
        echo -e "${GREEN}[HOOK SUCCESS] Quality check passed (no violations)${NC}" >&2
        exit 0
    fi
else
    # Fallback: monitor doesn't exist or failed to produce JSON
    echo -e "${YELLOW}[HOOK WARNING] Monitor not available, skipping quality check${NC}" >&2
    echo "$MONITOR_OUTPUT" >&2
    exit 0
fi
