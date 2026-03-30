---
name: develop
description: "Execute implementation from a spec file. Reads the spec, decomposes into tasks, dispatches workers with full spec context, verifies each against the spec. Triggers: 'develop', 'execute the plan', 'build this', 'implement this spec'."
argument-hint: "<spec-file-path> [--solo] [--auto]"
user-invocable: true
allowed-tools:
  - Agent
  - Bash
  - Read
  - Glob
  - Grep
  - Write
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - SendMessage
  - TeamCreate
  - TeamDelete
---

# Develop

Read a spec, decompose execution, dispatch workers with full context, verify each worker's output against the spec.

## Arguments

- `<spec-file-path>` — path to spec file (from /spec or ct spec latest)
- `--solo` — force single-worker mode
- `--auto` — skip user confirmations (for vibe/supervibe calls)
- No argument → find most recent spec: `ct spec latest --project "$(git rev-parse --show-toplevel)"`

## Step 1: Read the Spec

Read the spec file directly:
```bash
SPEC_CONTENT=$(ct spec read <spec-file-path>)
```

If the file doesn't exist or is empty, report and stop.

Parse sections: Problem, Recommendation, Architecture Context, Risks. The Architecture Context contains the key files and module roles that inform task decomposition.

## Step 2: Decompose

Break the spec into implementation tasks. Each task is a coherent chunk of work that one worker can complete. Use the Architecture Context to determine natural boundaries (files, modules, layers).

For each task, create a TaskCreate with:
- Subject describing what to build
- Description with relevant spec excerpts, file paths, and approach
- The full Recommendation and Architecture Context from the spec

Set up blockedBy relationships where tasks have dependencies.

Task system is for progress tracking — the real context comes from the spec text in the worker prompt.

## Step 3: Dispatch Workers

For each ready task, spawn an Agent with the worker prompt below. Cap: 4 concurrent workers.

**Solo mode** (`--solo` or single task): one worker at a time.
**Team mode** (2+ tasks): TeamCreate, dispatch unblocked tasks in parallel (up to 4), re-dispatch as tasks complete and unblock others.

### Worker Prompt

```
Implement this task.

## Task
<task description>

## Full Spec
<entire spec content — Problem, Recommendation, Architecture Context, Risks>

## Your Workflow
1. Read the relevant files listed in Architecture Context
2. Write a failing test that describes the target behavior
3. Run the test — confirm it fails for the right reason (missing method, wrong behavior — not random error)
4. Implement the minimum code to make the test pass
5. Run all tests — confirm green
6. Run the project build command — confirm it compiles
7. Report completion with a summary of what you implemented

## If blocked
- Design conflict with the spec → report "RESCOPE: <reason>" and stop
- Task too large → break it down and report the subtasks
```

Workers get the **full spec text** — not compressed metadata summaries. This is the key design choice: the spec is small enough to fit in a worker's context, and having the full "why" prevents workers from building the wrong thing.

## Step 4: Spec Compliance Review

After each worker completes, spawn a reviewer Agent:

```
Review this implementation against the spec.

## Spec
<full spec content>

## Changes
<git diff for this worker's changes>

## Check
1. Does the implementation match the Recommendation section?
2. Does it fit the Architecture Context (right files, right patterns)?
3. Are the Risks addressed or acknowledged?

Output: PASS or FAIL with specific citations.
If FAIL: list exactly what doesn't match and what should change.
```

**PASS** → mark task complete, dispatch next worker.
**FAIL** → send feedback to the worker, worker fixes, re-review. Max 2 fix cycles per worker.

## Step 5: Completion

After all workers complete and pass spec compliance:

1. Run full test suite. Red → spawn fix agent (max 2 cycles). Still red → report to user.
2. Stage changes: `git add -u`, show `git diff --cached --stat`.
3. Stop for user verification.

Output:
```
Develop: <spec topic>
Workers: N/N completed, all passed spec compliance
Files changed: <list>
Next: verify the implementation, then /review and /commit
```
