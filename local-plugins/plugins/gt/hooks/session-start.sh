#!/bin/bash
set -euo pipefail

# Check if gt is initialized in this repo (common-dir works in bare worktrees)
git_dir=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
[[ -f "$git_dir/.graphite_repo_config" ]] || exit 0

cat <<'EOF'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "## Graphite Workflow\n\nThis repo uses Graphite for stacked PRs. For any push, PR, rebase, stack, or branch operation, invoke the gt skill as your FIRST action — before running any Bash commands to explore.\n\n- Push / create PRs / update PRs / ship it → `Skill(gt:submit)`\n- Rebase / sync with main / stack out of date → `Skill(gt:restack)`\n- Everything else (check stack, create branch, move changes between branches, split work into separate PRs, navigate stack) → `Skill(gt:gt)`\n\nThe gt skills handle the full workflow including inspecting the current state — you do not need to run `git diff`, `git status`, or `gt log` in Bash first. Invoke the skill and it will handle exploration, stack safety, and PR metadata.\n\nRaw git/gt in Bash is fine only when the user explicitly requests it. Return `app.graphite.com/...` URLs."
  }
}
EOF
