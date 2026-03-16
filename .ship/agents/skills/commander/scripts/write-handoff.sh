#!/usr/bin/env bash
# write-handoff.sh <session-id> [summary]
# Scaffolds a handoff.md for the given session.
# The Commander fills in the sections before ending the session.

set -euo pipefail

SESSION_ID="${1:-}"
SUMMARY="${2:-}"

if [[ -z "$SESSION_ID" ]]; then
  echo "Usage: write-handoff.sh <session-id> [summary]" >&2
  exit 1
fi

SESSIONS_DIR=".ship/sessions"
HANDOFF_DIR="$SESSIONS_DIR/$SESSION_ID"
HANDOFF_FILE="$HANDOFF_DIR/handoff.md"

mkdir -p "$HANDOFF_DIR"

if [[ -f "$HANDOFF_FILE" ]]; then
  echo "Handoff already exists at $HANDOFF_FILE — not overwriting." >&2
  exit 0
fi

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > "$HANDOFF_FILE" <<EOF
# Handoff: Session $SESSION_ID

**Completed:** $TIMESTAMP
**Session:** $SESSION_ID

## Summary

${SUMMARY:-_Fill in what was accomplished this session._}

## Accomplished

-

## In Flight

-

## Blockers

-

## Next Steps

1.

## Context

_Any non-obvious state the next session should know._
EOF

echo "✓ handoff scaffold written to $HANDOFF_FILE"
