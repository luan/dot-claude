---
name: develop
description: "Execute implementation from a plan or spec file. Reads the plan phases, dispatches workers with full spec context, verifies each against the spec. Triggers: 'develop', 'execute the plan', 'build this', 'implement this spec'."
argument-hint: "<plan-or-spec-path> [--auto]"
user-invocable: true
allowed-tools:
  - Agent
  - Bash
  - Read
  - Glob
  - Grep
---

# Develop

Read a plan (preferred) or spec, dispatch workers per phase, verify each worker's output against the spec.

## Arguments

- `<path>` — path to plan file (preferred) or spec file
- `--auto` — skip user confirmations (for vibe/supervibe calls)
- No argument → find most recent plan: `ct plan latest --project "$(git rev-parse --show-toplevel)"`. If no plan, fall back to `ct spec latest`.

## Step 1: Read Plan and Spec

If content is already in your conversation (from a preceding /spec call), use it directly.

**Plan file provided (preferred path):**
```bash
PLAN_CONTENT=$(ct plan read <path>)
```
The plan's frontmatter contains `source:` linking to its spec. Read the spec too — workers need both:
```bash
SPEC_CONTENT=$(ct spec read <spec-stem>)
```

**Spec file provided (no plan):**
```bash
SPEC_CONTENT=$(ct spec read <path>)
```
Decompose into tasks yourself (Step 2 fallback).

**Extract reviewer annotations:** Run `ct spec comments <spec-file>` and (if plan exists) `ct plan comments <plan-file>`. If either returns comments, store them — they'll be appended to worker prompts in Step 3.

If the file doesn't exist or is empty, report and stop.

## Step 2: Decompose

**From plan file:** Parse `**Phase N:**` markers directly into tasks. Each phase becomes one task. The plan already specifies files, approach, steps, and dependencies — use them as-is.

**From spec file (fallback):** Break the spec into implementation tasks. Use the Architecture Context to determine natural boundaries (files, modules, layers). For each task, note:
- What to build
- Relevant spec excerpts, file paths, and approach
- Dependencies on other tasks (which must complete first)

## Step 3: Dispatch Workers

For each ready task, spawn an Agent with the worker prompt below. Cap: 4 concurrent workers. Dispatch unblocked tasks in parallel, re-dispatch as tasks complete and unblock others. Single task → one worker.

### Worker Prompt

```
Implement this phase.

## Phase
<phase title, files, approach, steps from plan>

## Full Spec
<entire spec content — Problem, Recommendation, Architecture Context, Risks>

## Your Workflow
1. Read the files listed in the phase (Read + Modify paths)
2. Write a failing test that describes the target behavior
3. Run the test — confirm it fails for the right reason (missing method, wrong behavior — not random error)
4. Implement the minimum code to make the test pass
5. Run all tests — confirm green
6. Run the project build command — confirm it compiles
7. Run the phase's verification step if specified
8. Report completion with a summary of what you implemented

## Reviewer Annotations
<if spec or plan had inline comments, include them here>
Address these in your implementation.

## If blocked
- Design conflict with the spec → report "RESCOPE: <reason>" and stop
- Task too large → break it down and report the subtasks
```

Workers get the **full spec text** plus their **phase details** from the plan. The spec provides the "why", the plan provides the "how". Both fit in a worker's context.

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
