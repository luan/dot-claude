#!/bin/bash
set -euo pipefail

# Only activate if gh stack extension is installed
gh extension list 2>/dev/null | grep -q 'gh-stack' || exit 0

cat <<'EOF'
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "## gh-stack Available\n\nThe gh-stack plugin is loaded. For branches in a `gh stack` stack, use gh-stack skills:\n\n- Push / create-update PRs → `Skill(gh-stack:submit)`\n- Rebase / sync with main → `Skill(gh-stack:restack)`\n- Create branch / navigate / stack ops → `Skill(gh-stack)`\n\nDetect which tool manages the current branch: `gh stack view --json` succeeds → gh-stack; `gt log --stack` succeeds → gt. Use the matching tool's skills. Raw git push, git rebase, and git checkout -b are blocked on gh-stack branches."
  }
}
EOF
