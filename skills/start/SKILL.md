---
name: start
description: "Create a new branch and optionally link to a task. Uses Graphite (gt) if available, falls back to git. Triggers: 'start', 'start new branch', 'begin work on'."
argument-hint: "<branch-name> [task-id]"
user-invocable: true
allowed-tools:
  - "Bash(git checkout:*)"
  - "Bash(git branch:*)"
  - "Bash(git rev-parse:*)"
  - AskUserQuestion
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - Skill
---

# Start

Create branch + optionally link task.

## Steps

1. Parse args: branch name + optional task ID
2. Normalize: prefix `luan/` if needed
3. Create branch:
   - If gt plugin is loaded: `Skill(gt:gt, "create <branch-name>")`
   - Otherwise: `git checkout -b <branch-name>`
4. If task ID:
   - `TaskUpdate(taskId, status: "in_progress", owner: "start")`
   - `TaskUpdate(taskId, metadata: {branch: "<branch-name>"})`
5. If no task ID:
   - AskUserQuestion: "Create task?"
   - If yes:
     ```
     TaskCreate:
       subject: "<branch-name>"
       activeForm: "Creating task"
       metadata:
         project: <repo root from git rev-parse --show-toplevel>
         type: "chore"
         priority: "P2"
     ```
   - `TaskUpdate(taskId, status: "in_progress", owner: "start", metadata: {branch: "<branch-name>"})`
6. Report branch + issue, suggest `/explore` or `/implement`

## Error Handling
- Branch creation fails → check if branch exists (`git branch -a | grep <name>`), suggest alternate name
- `TaskUpdate` fails → verify task ID exists with `TaskGet`, report if missing
- Not on expected parent branch → warn user, suggest checkout first
