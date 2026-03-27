---
name: start
description: "Create a new branch and optionally link a task. Uses Graphite (gt) if available, falls back to git."
argument-hint: "<branch-name> [task-id] [--auto]"
user-invocable: true
allowed-tools:
  - "Bash(git checkout:*)"
  - "Bash(git branch:*)"
  - "Bash(git rev-parse:*)"
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - Skill
---

# Start

Create branch + optionally link task.

## Steps

1. Parse args: first = branch name, second = optional task ID
2. Normalize: prefix with !`echo "${GIT_USERNAME:-$(whoami)}"/` if not already present
3. Create branch:
   - gt plugin loaded → `Skill(gt:gt, "create <branch-name>")`
   - Otherwise → `git checkout -b <branch-name>`
4. Task linking:
   - Task ID provided → `TaskUpdate(taskId, status: "in_progress", metadata: {branch: "<branch-name>"})`
   - No task ID + `--auto` → create task automatically
   - No task ID, no `--auto` → ask "Create a task for this branch?"
   - Task type inferred from branch name: feat/fix/chore. Priority: P2.
5. Report branch + task. If `--auto` was NOT passed, suggest `/scope` or `/develop`. If `--auto` was passed, output nothing — no report, no suggestions. The caller is an orchestrator that will handle next steps; any output text risks the model ending its turn prematurely.

## Error Handling

- **Branch exists** → check `git branch -a`, suggest alternate name
- **Task not found** → verify with `TaskGet`, report if missing
- **Wrong parent** → warn user, suggest checking out intended parent first
