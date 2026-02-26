---
name: start
description: "Create a new branch and optionally link to a task. Uses Graphite (gt) if available, falls back to git. Triggers: 'start', 'start new branch', 'begin work on'. User-invoked only — branches are not created autonomously."
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

Create branch + optionally link task. User-invoked only — this skill never creates branches autonomously, respecting the "no branches unless explicitly requested" convention.

## Steps

1. Parse args: first = branch name, second = optional task ID
2. Normalize: prefix with !`echo "${GIT_USERNAME:-$(whoami)}"/` — if not already present
3. Create branch:
   - gt plugin loaded → `Skill(gt:gt, "create <branch-name>")`
   - Otherwise → `git checkout -b <branch-name>`
4. If task ID provided:
   - `TaskUpdate(taskId, status: "in_progress", metadata: {branch: "<branch-name>"})`
5. If no task ID:
   - AskUserQuestion: "Create a task for this branch?"
   - If yes → `TaskCreate` with subject from branch name. Type inferred from branch name: `"feat"` for feature work, `"fix"` for bugfixes, `"chore"` otherwise. Priority defaults to P2.
   - Link: `TaskUpdate(taskId, status: "in_progress", metadata: {branch: "<branch-name>"})`
6. Report branch + task, suggest `/scope` or `/develop`

## Error Handling

- **Branch exists** → `git branch -a | grep <name>`, suggest alternate name
- **Task ID not found** → verify with `TaskGet`, report if missing
- **Wrong parent branch** → warn user, suggest checking out intended parent first
