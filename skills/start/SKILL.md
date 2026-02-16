---
name: start
description: "Create a new Graphite branch and optionally link to a work issue. Triggers: 'start', 'start new branch', 'begin work on'."
argument-hint: "<branch-name> [work-issue-id]"
user-invocable: true
allowed-tools:
  - Bash
  - AskUserQuestion
---

# Start

Create branch + optionally link work issue.

## Steps

1. Parse args: branch name + optional work ID
2. Normalize: prefix `luan/` if needed
3. `gt create <branch-name>`
4. If work ID:
   - `work start <id>`
   - `work comment <id> "Branch: <branch-name>"`
5. If no work ID:
   - AskUserQuestion: "Create work issue?"
   - If yes: `work create "<branch-name>" --type chore --priority 2`
   - `work start <id>`
6. Report branch + issue, suggest `/explore` or `/implement`

## Error Handling
- `gt create` fails → check if branch exists (`git branch -a | grep <name>`), suggest alternate name
- `work` command fails → verify issue ID exists with `work show <id>`, report if missing
- Not on expected parent branch → warn user, suggest `gt checkout` first
