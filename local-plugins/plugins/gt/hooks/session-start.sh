#!/bin/bash
set -euo pipefail

# Check if gt is initialized in this repo (common-dir works in bare worktrees)
git_dir=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
[[ -f "$git_dir/.graphite_repo_config" ]] || exit 0

cat <<'EOF'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "## Graphite Workflow Rules\n\n- All branch operations go through `/gt:gt`. Never raw `git rebase`, `git push`, `git branch -d`, `git checkout -b`.\n- Push → `/gt:submit`. Restack → `/gt:restack`. Commit → `/commit`.\n- Return `app.graphite.com/...` URLs, not GitHub.\n- Review scope: diff vs stack parent (`gt log`), not trunk."
  }
}
EOF
