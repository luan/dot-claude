#!/usr/bin/env bash
set -euo pipefail

# Run a single RED eval in an isolated git repo
# Usage: run-red.sh <eval-id> <setup-script> <prompt> <check-script> <output-dir>

EVAL_ID="$1"
SETUP_SCRIPT="$2"
PROMPT="$3"
CHECK_SCRIPT="$4"
OUTPUT_DIR="$5"

WORKDIR=$(mktemp -d)
mkdir -p "$OUTPUT_DIR"

# Setup: create repo with files
cd "$WORKDIR"
git init -q
git config user.email "test@test.com"
git config user.name "Test"
eval "$SETUP_SCRIPT"

# Run claude -p in the repo
CLAUDE_CODE_DISABLE_AUTO_MEMORY=1 \
  claude -p \
  --setting-sources "" \
  --permission-mode bypassPermissions \
  --output-format json \
  --max-budget-usd 1.00 \
  "$PROMPT" \
  2>&1 > "$OUTPUT_DIR/response.json" || true

# Capture post-state
git log --oneline 2>/dev/null > "$OUTPUT_DIR/git-log.txt" || true
git branch 2>/dev/null > "$OUTPUT_DIR/git-branches.txt" || true
git status --short 2>/dev/null > "$OUTPUT_DIR/git-status.txt" || true
git diff 2>/dev/null > "$OUTPUT_DIR/git-diff.txt" || true

# Extract response text
python3 -c "import json,sys; d=json.load(open('$OUTPUT_DIR/response.json')); print(d.get('result',''))" > "$OUTPUT_DIR/response.txt" 2>/dev/null || true

# Run check script — outputs PASS or FAIL with reason
cd "$WORKDIR"
RESULT=$(eval "$CHECK_SCRIPT" 2>&1) || true
echo "$RESULT" > "$OUTPUT_DIR/verdict.txt"

# Cleanup
rm -rf "$WORKDIR"

echo "$EVAL_ID: $RESULT"
