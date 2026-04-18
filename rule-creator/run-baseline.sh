#!/usr/bin/env bash
set -euo pipefail

RULE_PATH="$1"
PROMPT="$2"
MODEL="$3"
SETUP="${4:-}"
TEARDOWN="${5:-}"

BACKUP="${RULE_PATH}.evaloff"

cleanup() {
  [ -f "$BACKUP" ] && mv "$BACKUP" "$RULE_PATH"
  [ -n "$TEARDOWN" ] && eval "$TEARDOWN" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

# Run setup if provided
[ -n "$SETUP" ] && eval "$SETUP"

mv "$RULE_PATH" "$BACKUP"

CMD=(claude -p "$PROMPT" --dangerously-skip-permissions --model "$MODEL")

mkdir -p /tmp/rule-creator-eval
TRANSCRIPT=$(cd /tmp/rule-creator-eval && "${CMD[@]}" 2>&1)

echo "$TRANSCRIPT"
