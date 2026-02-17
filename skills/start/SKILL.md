---
name: start
description: "Create a new Graphite branch and optionally link to a task. Triggers: 'start', 'start new branch', 'begin work on'."
argument-hint: "<branch-name> [task-id]"
user-invocable: true
allowed-tools:
  - Bash
  - AskUserQuestion
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
---

# Start

Create branch + optionally link task.

## Steps

1. Parse args: branch name + optional task ID
2. Normalize: prefix `luan/` if needed
3. `gt create <branch-name>`
4. If task ID:
   - `TaskUpdate(taskId, status: "in_progress")`
   - `TaskUpdate(taskId, metadata: {branch: "<branch-name>"})`
5. If no task ID:
   - AskUserQuestion: "Create task?"
   - If yes:
     ```
     TaskCreate:
       subject: "<branch-name>"
       activeForm: "Creating task"
       metadata:
         project: <repo root>
         type: "chore"
         priority: 2
     ```
   - `TaskUpdate(taskId, status: "in_progress", metadata: {branch: "<branch-name>"})`
6. Report branch + issue, suggest `/explore` or `/implement`

## Error Handling
- `gt create` fails → check if branch exists (`git branch -a | grep <name>`), suggest alternate name
- `TaskUpdate` fails → verify task ID exists with `TaskGet`, report if missing
- Not on expected parent branch → warn user, suggest `gt checkout` first
