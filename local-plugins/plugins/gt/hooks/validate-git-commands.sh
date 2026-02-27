#!/bin/bash
set -euo pipefail

input=$(cat)
command=$(echo "$input" | jq -r '.tool_input.command // empty')
[[ -n "$command" ]] || exit 0

# Only enforce in Graphite-managed repos
git_dir=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
[[ -f "$git_dir/.graphite_repo_config" ]] || exit 0

deny() {
  echo "{\"hookSpecificOutput\":{\"permissionDecision\":\"deny\"},\"systemMessage\":\"$1\"}" >&2
  exit 2
}

# git push — use /gt:submit
[[ "$command" =~ (^|[;\&\|])\ *git\ +push ]] && \
  deny "BLOCKED: raw 'git push' in Graphite repo. Use /gt:submit instead."

# gh pr create — use /gt:submit
[[ "$command" =~ (^|[;\&\|])\ *gh\ +pr\ +create ]] && \
  deny "BLOCKED: raw 'gh pr create' in Graphite repo. Use /gt:submit instead."

# git rebase — use /gt:restack
[[ "$command" =~ (^|[;\&\|])\ *git\ +rebase ]] && \
  deny "BLOCKED: raw 'git rebase' in Graphite repo. Use /gt:restack instead."

# git checkout -b — use /gt:gt create or /start
[[ "$command" =~ (^|[;\&\|])\ *git\ +checkout\ +-b ]] && \
  deny "BLOCKED: raw 'git checkout -b' in Graphite repo. Use /start or /gt:gt create instead."

# git branch -d/-D — use /gt:gt delete
[[ "$command" =~ (^|[;\&\|])\ *git\ +branch\ +-[dD] ]] && \
  deny "BLOCKED: raw 'git branch -d/-D' in Graphite repo. Use /gt:gt delete instead."

exit 0
