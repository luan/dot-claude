#!/bin/bash
set -euo pipefail

input=$(cat)
command=$(echo "$input" | jq -r '.tool_input.command // empty')
[[ -n "$command" ]] || exit 0

# Only enforce if gh stack is installed
command -v gh >/dev/null 2>&1 || exit 0
gh extension list 2>/dev/null | grep -q 'gh-stack' || exit 0

# Only enforce if current branch is in a gh-stack stack
stack_json=$(gh stack view --json 2>/dev/null) || exit 0
echo "$stack_json" | jq -e '.branches | length > 0' >/dev/null 2>&1 || exit 0

deny() {
  echo "{\"hookSpecificOutput\":{\"permissionDecision\":\"deny\"},\"systemMessage\":\"$1\"}" >&2
  exit 2
}

current_branch=$(git symbolic-ref --short HEAD 2>/dev/null || true)
trunk=$(echo "$stack_json" | jq -r '.trunk // "main"')

# git push — allow on trunk, block on stacked branches
[[ "$command" =~ (^|[;\&\|])\ *git\ +push ]] && [[ "$current_branch" != "$trunk" ]] && \
  deny "BLOCKED: raw 'git push' on gh-stack branch. Use /gh-stack:submit instead."

# git pull — allow on trunk, block on stacked branches
[[ "$command" =~ (^|[;\&\|])\ *git\ +pull ]] && [[ "$current_branch" != "$trunk" ]] && \
  deny "BLOCKED: raw 'git pull' on gh-stack branch. Use /gh-stack:restack instead."

# gh pr create — use submit skill
[[ "$command" =~ (^|[;\&\|])\ *gh\ +pr\ +create ]] && \
  deny "BLOCKED: raw 'gh pr create' on gh-stack branch. Use /gh-stack:submit instead."

# git rebase — allow --abort/--continue/--skip (recovery), block new rebases
[[ "$command" =~ (^|[;\&\|])\ *git\ +rebase ]] && \
  ! [[ "$command" =~ git\ +rebase\ +--(abort|continue|skip) ]] && \
  deny "BLOCKED: raw 'git rebase' on gh-stack branch. Use /gh-stack:restack instead."

# git checkout -b — use gh stack add or /start
[[ "$command" =~ (^|[;\&\|])\ *git\ +checkout\ +-b ]] && \
  deny "BLOCKED: raw 'git checkout -b' on gh-stack branch. Use /start or /gh-stack add instead."

exit 0
