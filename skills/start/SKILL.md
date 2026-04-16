---
name: start
description: "Create a new branch. Uses gh-stack or Graphite (gt) if available, falls back to git."
argument-hint: "<branch-name> [--auto]"
user-invocable: true
allowed-tools:
  - "Bash(git checkout:*)"
  - "Bash(git branch:*)"
  - "Bash(git rev-parse:*)"
  - TaskUpdate
  - TaskGet
  - Skill
---

# Start

Create branch.

## Steps

1. Parse args: first = branch name
2. Normalize: prefix with !`echo "${GIT_USERNAME:-$(whoami)}"/` if not already present
3. Create branch (detect stack tool for current branch):
   - `gh stack view --json 2>/dev/null` succeeds → `Skill(gh-stack, "add <branch-name>")`
   - gt plugin loaded → `Skill(gt:gt, "create <branch-name>")`
   - Otherwise → `git checkout -b <branch-name>`
4. Report branch. If `--auto` was NOT passed, suggest `/spec` or `/develop`. If `--auto` was passed, output nothing — no report, no suggestions. The caller is an orchestrator that will handle next steps; any output text risks the model ending its turn prematurely.

## Error Handling

- **Branch exists** → check `git branch -a`, suggest alternate name
- **Wrong parent** → warn user, suggest checking out intended parent first
