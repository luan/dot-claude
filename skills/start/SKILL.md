---
name: start
description: "Create a new Graphite branch and optionally link to a beads issue. Triggers: 'start', 'start new branch', 'begin work on'."
argument-hint: "<branch-name> [beads-issue-id]"
user-invocable: true
allowed-tools:
  - Bash
  - AskUserQuestion
---

# Start

Create branch + optionally link beads issue.

## Steps

1. Parse args: branch name + optional beads ID
2. Normalize: prefix with `luan/` if not already
3. `gt create <branch-name>`
4. If beads ID provided:
   - `bd update <id> --status in_progress`
   - `bd update <id> --notes "Branch: <branch-name>"`
5. If no beads ID:
   - AskUserQuestion: "Create beads issue for this branch?"
   - If yes: `bd create "<branch-name>" --type task --priority 2`
   - `bd update <id> --status in_progress`
6. Report branch + issue, suggest `/explore` or `/implement`
