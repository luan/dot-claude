#!/bin/bash
set -euo pipefail

# Check if gt is initialized in this repo (common-dir works in bare worktrees)
git_dir=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
[[ -f "$git_dir/.graphite_repo_config" ]] || exit 0

cat <<'EOF'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "## Graphite Workflow\n\nThis repo uses Graphite for stacked PRs. Decision rule: if on a gt-managed branch, use gt commands exclusively (never raw git rebase, git push, or git checkout -b). If not on a gt-managed branch, use git normally. Never mix.\n\n- Push / create-update PRs → `Skill(gt:submit)`\n- Rebase / sync with main → `Skill(gt:restack)`\n- Create branch / navigate / stack ops → `Skill(gt:gt)`\n\nRaw git/gt in Bash is fine only when the user explicitly requests it. Return `app.graphite.com/...` URLs."
  }
}
EOF
