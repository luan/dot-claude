---
name: implement
description: "Execute a work epic or task — auto-detects solo vs swarm mode, dispatches subagents to implement. Triggers: \"implement\", \"execute the plan\", \"build this\", \"code this plan\", \"start implementing\", \"ready to implement\", work issue/epic ID."
argument-hint: "[epic-id|task-id] [--solo]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - SendMessage
  - TeamCreate
  - TeamDelete
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - AskUserQuestion
  - Read
  - Glob
  - Grep
  - Bash
---

# Implement

Detects solo vs swarm automatically. Handles both single-agent + multi-agent parallel execution.

**IMMEDIATELY dispatch.** Never implement on main thread.

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Design decisions not covered in brief → surface to user before implementing
- Test strategy unclear (integration vs unit, what to mock) → ask approach

Do NOT ask when the answer is obvious or covered by the task brief.

## Step 1: Classify Work

1. `work show <id> --format=json` (from $ARGUMENTS)
2. Check for children: does `work show <id>` list children?
3. Choose mode:
   - `--solo` flag → **Solo Mode**
   - Single task (no children) → **Solo Mode**
   - Has children, fewer than 3 open → **Solo Mode**
   - Has children, 3+ open → **Swarm Mode**

## Solo Mode

Dispatch via Task (subagent_type="general-purpose"):

```
Implement: $ARGUMENTS

## Job
1. **Pre-flight:** `work show <epic-id>` — no children or tasks lack acceptance criteria or file paths → STOP, return "prepare phase incomplete — no implementable tasks". Do NOT create tasks.
2. `work list --status open --parent <epic-id>` or `work show <id>`
3. Per task:
   - `work show <task-id>` + `work start <task-id>` + `work edit <task-id> --assignee solo`
   - **Step 0 — Understand:** Read EVERY file listed in task. Note indent style (tabs vs spaces + width). Verify assumptions from task description. Investigate current structure.
   - **Indentation pre-flight:** Before first Edit to any file: read file, identify indent char + width. Use EXACTLY that in all edits to that file.
   - Implement using TDD from brief: failing test first → minimal implementation → verify passes
   - **Completion gate (before work review):**
     1. Detect build cmd: justfile/Makefile/package.json/CLAUDE.md
     2. Run build. Exit != 0 → trace error to root cause, fix (max 3 attempts)
     3. Run tests: new + existing touching modified files
     4. **Polish pass:** flatten unnecessary nesting (early returns), remove code-restating comments and contextless TODOs, remove unused imports and debug artifacts (console.log, print). Never change behavior.
     5. ALL green → `work review <task-id>`. ANY red after 3 attempts → report error output, do NOT review
   - **Fix methodology:** Read error → trace to root cause → ONE targeted fix. No guess-and-patch. >10 tool calls on single fix → checkpoint findings + escalate to caller.
   - **Never run git commands.** Orchestrator handles commits. You: edits + build gate only.
   - `work review <task-id>`
4. Done → report completion to caller

## Task Atomicity
NEVER stop mid-task. Finish before any PR ops.

## Side Quests
Bug found? `work create "Found: ..." --type bug`
```

5. **Orchestrator** creates single WIP commit: `git add . && git diff --staged --quiet || git commit -m 'wip: implement <epic-title> (<id>)'`
6. → See Continuation Prompt below.

## Swarm Mode

Orchestrate parallel workers grouped by phase number.

### Setup

1. `work show <epic-id>` — get title + description
2. `work list --status open --parent <epic-id> --format=json` — get children
3. **Group by phase number:** Parse "Phase N:" from each child's
   title. Group into phases. Tasks without phase numbers → last phase.
   Sort phases numerically.
4. Create team:
   ```
   TeamCreate:
     team_name: "impl-<short-desc>"
     description: "Implementing <epic summary>"
   ```

### Phase Loop

```
for each phase_group (ordered by phase number):
  Spawn ALL tasks in this phase in SINGLE message (parallel).
  Workers = min(task_count, 4).

  Per-task model selection:
    For each task, `work show <id>` and evaluate against
    mechanical criteria in rules/model-tiering.md.
    Insufficient brief to evaluate → opus.
    opus is the DEFAULT. sonnet is rare.

  Each worker (Task, subagent_type="general-purpose", mode="plan",
  team_name="<team>", name="worker-<n>"):

  """
  Worker-<n> implementing <task-id>.

  ## Your Task
  <issue description from work show>

  ## Protocol
  1. `work start <task-id>` + `work edit <task-id> --assignee worker-<n>`
  2. **Understand first:** Read every file in task. Note indent
     (char + width). Verify assumptions from brief.
  3. Failing test FIRST → minimal implementation
  4. **Build gate (max 3 attempts):**
     a. Build cmd from justfile/Makefile/package.json/CLAUDE.md
     b. Build + tests. All green → continue. Red → root-cause, ONE fix.
     c. 3 fails → report error, do NOT submit for review.
     d. >10 tool calls on one fix → checkpoint + escalate.
     e. Failure traces to another worker's file → message team
        lead, wait.
  5. **Polish pass:** flatten unnecessary nesting (early returns), remove code-restating comments and contextless TODOs, remove unused imports and debug artifacts. Never change behavior.
  6. `work review <task-id>`
  6. Send completion message to team lead via SendMessage
  7. Wait for shutdown or next assignment.

  ## File Boundaries (HARD RULE)
  NEVER edit files outside your task scope.
  Need change in another worker's file → MESSAGE team lead.

  ## Git Operations
  **Never run git commands.** Orchestrator handles commits.
  """

  Wait for all workers in this phase to complete.
  If any worker reported escalation or >2 tasks failed build gate
  → PAUSE. Report status to user before continuing next phase.

  Shut down phase workers (SendMessage shutdown_request).
  Orchestrator commits phase: `git add . && git diff --staged --quiet || git commit -m 'wip: implement phase <N> (<brief-summary>)'`
```

### Verify (after final phase)

1. Full test suite
2. **Green** → continue
3. **Red** → spawn fix agent targeting failures (max 2 cycles)
4. 2 failed → escalate to user

### Teardown

1. `work review <epic-id>` — marks implementation complete, NOT approved
2. TeamDelete
3. Report completion to caller
4. → See Continuation Prompt below.

## Continuation Prompt

Use AskUserQuestion:
- "Continue to /split-commit" (Recommended) — description: "Split WIP commit into clean, tested vertical commits, then /review"
- "Continue to /review" — description: "Skip split-commit and review WIP commit directly"
- "Review changes manually first" — description: "Inspect the diff before automated review"
- "Done for now" — description: "Leave epic + tasks in review for later /resume-work"

If user selects "Continue to /split-commit":
→ Determine base branch: `git log --oneline --first-parent | head -20` to find the merge base, or use `main` as default
→ Invoke Skill tool: skill="split-commit", args="<base-branch>"
If user selects "Continue to /review":
→ Invoke Skill tool: skill="review", args=""

## Key Rules

- Main thread does NOT implement — subagent/team does
- Workers own implementation — briefs give direction, not code
- Task atomicity — never stop mid-task
- Pre-flight required — bail if no implementable tasks
- No branch creation — implement assumes branch already exists (use /start)
- Single WIP commit after all work — orchestrator only, workers never git. Use /split-commit to repackage before review.
- Swarm: spawn ALL wave workers in single message
- Fix cycles capped at 2 → escalate to user
- Workers submit `work review` — NO `work approve` anywhere in implement. Approval only after user-driven /review or explicit request.
