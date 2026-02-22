---
name: review
description: "Adversarial code review with parallel reviewers. Triggers: 'review', 'review my changes', 'check this code', 'code review'. Use --team for 3-perspective mode. Do NOT use when: only cosmetic polish is needed — use /refine instead. Do NOT use when: investigating an unknown bug — use /debugging instead."
argument-hint: "[base..head | file-list | PR#] [--against <issue-id>] [--team] [--continue]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - AskUserQuestion
  - Read
  - Bash
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - Write
---

# Adversarial Review

Three modes: solo (default), file-split (auto for large diffs), perspective (--team). All consolidate findings into phase-structured output.

## Interviewing

See rules/skill-interviewing.md. Skill-specific triggers:

- Severity judgment borderline (medium vs high) → ask user's priority
- Pattern violation unclear (style preference vs correctness issue) → clarify importance

## Constants

Expanded at load:

- BASE=!`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`
- REPO_ROOT=!`git rev-parse --show-toplevel 2>/dev/null`
- CODEX_AVAILABLE=!`which codex >/dev/null 2>&1 && echo "true" || echo "false"`
- NON_INTERACTIVE=!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "true" || echo "false"`

## Step 1: Scope + Mode

Parse $ARGUMENTS:

- `--against <task-id>`: task for plan adherence
- `--team`: force 3-perspective mode
- Remaining args override BASE:

| Input        | Diff source                             |
| ------------ | --------------------------------------- |
| (none)       | `git diff $BASE...HEAD`                 |
| `main..HEAD` | BASE=main                               |
| file list    | `git diff HEAD -- <files>` + read files |
| `#123`       | `gh pr diff 123`                        |

Count files: `git diff --stat`

Choose mode:

- `--team` → **Perspective Mode** (3 specialists)
- 15+ files, no `--team` → **File-Split Mode**
- Otherwise → **Solo Mode** (2 lenses)

Size check: from `git diff --stat` output, count changed files and total lines (+/-).
Set CODEX_TRIGGERED=true if: (file_count >= 5 OR total_lines >= 200) AND CODEX_AVAILABLE.

## Step 1b: Create Review Issue

```
TaskCreate:
  subject: "Review: <scope-summary>"
  description: "Adversarial review of <scope-details>"
  activeForm: "Creating review task"
  metadata:
    project: REPO_ROOT
    type: "review"
    priority: "P2"

TaskUpdate(taskId, status: "in_progress", owner: "review")
```

If `--continue`: skip creation, find existing:

- $ARGUMENTS matches task ID → use it
- Else: `TaskList()` filtered by `metadata.type === "review"` and `status === "in_progress"`, use first result
- `TaskGet(taskId)` → extract `metadata.design`
- Prepend to reviewers: "Previous findings:\n<metadata.design>\n\nContinue reviewing..."

## Step 2: Gather Context

<!-- Lightweight commands — reviewers fetch their own full diff via gitcontext -->
Run in parallel:
- `git diff --stat $BASE...HEAD` → file list with change summary
- `git diff --name-only $BASE...HEAD` → CHANGED_FILES
- `git log --oneline $BASE..HEAD` → commit summary
- `ck tool cochanges --base $BASE` → COCHANGES (empty output or command failure → skip completeness lens silently)

If `--against`: `TaskGet(issueId)` for plan.

## Prompt Components

Substituted into Step 3 prompts via `{name}` markers.

**{context_preamble}:**
```
## Gather Context
1. Run: `ck tool gitcontext --base {base_ref} --format json`
2. Read all changed files from the output
3. If `truncated_files` is non-empty, `Read` those files in full
```

**{disposition_block}:**
```
Classify each finding:
- FIX: correctness bugs, security issues, test gaps — will be auto-fixed
- IGNORE: style preferences, subjective, low-signal, out-of-scope tech debt — skip

Assign a tier to each finding:
- critical: correctness bugs, security vulnerabilities, data loss risks
- notable: design issues, performance problems, missing tests
- nitpick: style, naming, minor improvements
```

## Step 3: Dispatch Reviewers

Substitute `{base_ref}` → BASE, `{files}` → file list, `{changed_files}` → CHANGED_FILES, `{cochange_candidates}` → COCHANGES, and prompt components.

All reviewers are persistent-reviewer Task agents. Spawn all agents for the chosen mode in a SINGLE message.

If `--against`: append to every reviewer prompt: "Also check plan adherence: does implementation match plan? Missing/unplanned features? Deviations justified? Plan: {issue description}"

### Solo Mode

**Agent 1 — Correctness & Security:**
```
You are an adversarial correctness and security reviewer.

{context_preamble}

Focus:
- Edge cases (empty, null, overflow, concurrent access)
- Invalid states, race conditions
- Resource leaks (unclosed handles, missing cleanup)
- Silent failures, swallowed errors
- Off-by-one, logic inversions
- Injection (SQL, command, XSS, template)
- Auth/authz gaps, data exposure, cryptographic misuse
- Missing tests for new or changed behavior, untested edge cases

{disposition_block}

Output: table with Tier | Severity | Disposition | File:Line | Issue | Suggestion
Then brief summary.
```

**Agent 2 — Architecture & Performance:**
```
You are an adversarial architecture and performance reviewer.

{context_preamble}

Focus:
- Incomplete refactors, dead code, unused params
- Unnecessary abstractions, coupling
- Over-engineering: near-identical blocks that should stay flat, abstractions/layers with no callsite outside this diff, "just in case" scaffolding or versioned names (FooV2), unused functions/params, wrapper types or indirection adding no invariant
- O(n^2) in loops, unnecessary allocations
- Memory (retained refs, unbounded growth)
- I/O (blocking calls, N+1 queries)
- Concurrency (thread safety, deadlock, contention)

{disposition_block}

Output: table with Tier | Severity | Disposition | File:Line | Issue | Suggestion
Then Simplicity table (same columns, severity capped at medium) for over-engineering findings.
Then brief summary.
```

### File-Split Mode

Split CHANGED_FILES into groups of ~8. One agent per group:

```
You are an adversarial reviewer covering correctness/security and architecture/performance.

## Gather Context
Files in scope: {files}

1. Run: `ck tool gitcontext --base {base_ref} --format json`
2. Read these files in full: {files}
3. If `truncated_files` is non-empty for any scoped file, `Read` those files in full

Focus (Correctness & Security):
- Edge cases (empty, null, overflow, concurrent access)
- Invalid states, race conditions
- Resource leaks (unclosed handles, missing cleanup)
- Silent failures, swallowed errors
- Off-by-one, logic inversions
- Injection (SQL, command, XSS, template)
- Auth/authz gaps, data exposure, cryptographic misuse
- Missing tests for new or changed behavior, untested edge cases

Focus (Architecture & Performance):
- Incomplete refactors, dead code, unused params
- Unnecessary abstractions, coupling
- Over-engineering: near-identical blocks that should stay flat, abstractions/layers with no callsite outside this diff, "just in case" scaffolding or versioned names (FooV2), unused functions/params, wrapper types or indirection adding no invariant
- O(n^2) in loops, unnecessary allocations
- Memory (retained refs, unbounded growth)
- I/O (blocking calls, N+1 queries)
- Concurrency (thread safety, deadlock, contention)

{disposition_block}

Output: table with Tier | Severity | Disposition | File:Line | Issue | Suggestion
Then Simplicity table (same columns, severity capped at medium) for over-engineering findings.
Then brief summary.
```

### Perspective Mode (--team)

Spawn EXACTLY 3 agents (+ extras if applicable):

**Agent 1 — Architect:**
```
Architecture reviewer.

{context_preamble}

Focus:
- System boundaries, coupling, scalability
- Design flaws, incomplete abstractions
- Dependency direction, module cohesion
- Over-engineering: near-identical blocks that should stay flat, abstractions/layers with no callsite outside this diff, "just in case" scaffolding or versioned names (FooV2), unused functions/params, wrapper types or indirection adding no invariant
- Testing gaps: new/changed logic with no coverage, boundary conditions not exercised, untested error paths

{disposition_block}

Tag: [architect]
Output: Phase 1 (Critical) → Phase 2 (Design & Simplicity, cap simplicity severity at medium) → Phase 3 (Testing Gaps)
Each finding: table with Tier | Severity | Disposition | File:Line | Issue | Suggestion
```

**Agent 2 — Code Quality:**
```
Code quality reviewer.

{context_preamble}

Focus:
- Readability, naming, error handling
- Edge cases, off-by-one, null safety
- Consistency with surrounding code
- Resource leaks, missing cleanup
- Testing gaps: new/changed logic with no coverage, boundary conditions not exercised, untested error paths

{disposition_block}

Tag: [code-quality]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
Each finding: table with Tier | Severity | Disposition | File:Line | Issue | Suggestion
```

**Agent 3 — Devil's Advocate:**
```
Devil's advocate reviewer.

{context_preamble}

Focus:
- Failure modes others miss
- Security: injection, auth gaps, data exposure
- Bad assumptions, race conditions
- What breaks under load, bad input, or partial failure?
- Testing gaps: new/changed logic with no coverage, boundary conditions not exercised, untested error paths

{disposition_block}

Tag: [devil]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
Each finding: table with Tier | Severity | Disposition | File:Line | Issue | Suggestion
```

### Additional Agents (all modes)

Spawned in the same message as the mode's primary agents.

**Completeness (only if COCHANGES non-empty):**
```
You are a completeness reviewer. Find files NOT updated that likely should have been.

## Changed Files
{changed_files}

## Co-change Candidates
These files historically change alongside the above but were NOT in this diff:
{cochange_candidates}

## Your Job
1. Read each co-change candidate file
2. Read the changed files to understand what changed
3. For each candidate: determine if the change warrants an update (pattern consistency, missing counterpart, stale references)
4. Only flag files with a specific, concrete reason — not just statistical co-change

{disposition_block}

Severity: medium if pattern is clearly broken (counterpart not updated); low if speculative.

Output: table with Tier | Severity | Disposition | File | Issue | Suggestion
Then brief summary.
```

**Codex (only if CODEX_TRIGGERED):**
```
Run `codex review --base {base_ref}` via Bash. Capture the full output.
If the command fails or is not found, return empty findings with a warning note.

Parse the output into individual findings. For each finding, extract file:line, issue description, and severity estimate.

Tag all findings with [external].

{disposition_block}

Output: table with [external] | Tier | Severity | Disposition | File:Line | Issue | Suggestion
```

## Step 4: Consolidate + Present

0. **Validate reviewer output** (subagent-trust.md): spot-check 1-2 specific file:line claims from each reviewer. Claimed issue doesn't exist at that location → discard.
   For [external] codex findings: spot-check ALL file:line claims. Codex duplicate of reviewer finding → keep reviewer version. [external] tag persists.
1. Deduplicate (same issue from multiple lenses → highest severity, highest tier)
2. Consensus filter: a finding survives if (a) any reviewer tagged it critical, OR (b) 2+ reviewers flagged it at the same tier. Solo (2 reviewers): 2-of-2 for notable/nitpick. File-Split/Perspective (3+ reviewers): 2-of-N. Single-reviewer notables and nitpicks → demote to IGNORE with "1-of-N reviewers" label.
3. Sort by severity. **NEVER truncate validated findings.** Output EVERY finding that survived consensus + validation.
4. --team only: tag [architect]/[code-quality]/[devil], note disagreements

Output: `# Adversarial Review Summary`

- --team: **Consensus** (top, issues flagged by 2+ reviewers)
- **FIX items** (sorted by severity): table with Tier | Severity | File:Line | Issue | Suggestion
- **IGNORE items** (grouped by category, one line each): collapsed summary — includes consensus-demoted findings labeled "1-of-N reviewers"
- --team: **Disagreements** (bottom)
- Footer: Verdict (APPROVE/COMMENTS/CHANGES), Blocking count, Review task-id

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "Use AskUserQuestion: Fix all FIX items / Fix critical+high only / Fix critical only / Skip fixes"`

## Step 4b: Store Findings

1. `PLAN_FILE=$(echo "<findings>" | ck plan create --topic "<topic>" --project "REPO_ROOT" --prefix "review" 2>/dev/null)` — if command fails or empty, warn: "Plan file creation failed — findings in task metadata only."
2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"}, description: "Review: <topic> — findings in plan file and metadata.design")`

## Step 5: Dispatch Fixes

Spawn general-purpose agent:

```
Fix these review issues in code.

## Issues to Fix
{FIX-classified issues with file:line refs}

## Your Job
1. Fix each listed issue
2. Verify fixes (syntax check, run tests to confirm no regressions)
3. Report what you fixed with file:line for each fix

Do NOT: fix unlisted things, refactor beyond needed, add features
```

After fix agent returns, invoke `Skill("refine")` on changed files.

## Step 6: Re-review

Re-run Step 3 after fixes. Track iteration count starting at 1 (max 4).

Before re-running:

- Maintain `fixed_issues` set: `(file, issue-description)` pairs (not file:line — lines shift)
- When consolidating: skip findings matching `fixed_issues`

On each iteration: announce "Re-review iteration N/4"

Loop exits when:

- All FIX items resolved
- OR user selects "Stop fixing"
- OR iteration count reaches 4 (report remaining as unresolved)

## Step 6a: Review Summary

### Fixes Applied (N)

- [file:line] Description of fix

### Ignored Issues (N)

- [Severity] Description (grouped by type)

### Remaining Issues (N)

- [Severity] [file:line] Description

If remaining issues exist and NON_INTERACTIVE is false, ask which to track as tasks:

```
AskUserQuestion: "Which remaining issues should be tracked as tasks?"
  multiSelect: true
  options: [one per remaining issue] + "None"
```

For each selected item:

```
TaskCreate:
  subject: "<one-line issue description>"
  description: "From review <reviewId>.\n\nFile: <file:line>\nSeverity: <severity>\n\n<full issue description + suggestion>"
  activeForm: "Creating deferred issue task"
  metadata:
    project: REPO_ROOT
    type: "deferred-review"
    source_review: "<reviewId>"
    priority: "<P2 for high/critical, P3 for medium/low>"
```

Store summary in `metadata.design` via TaskUpdate.

## Step 6b: Close Review Issue

```
TaskUpdate(reviewId, status: "completed")
```

## Step 7: Interactive Continuation

Check for pending review tasks: `TaskList()` filtered by `metadata.status_detail === "review"`. If any exist, note them.

Present next step based on outcome — use AskUserQuestion only when there's a genuine choice:

- **Clean review** → "Approve + commit" or "Refine before commit" or "Test plan"
- **Issues found and fixed** → "Re-review to verify?" or "Approve + commit" or "Refine before commit"
- **Issues found but not all fixed** → "Continue fixing?" or "Approve as-is" or "Refine before commit"

Skill dispatch:

- Approve + commit → `TaskList()` filtered by `metadata.project === REPO_ROOT` and `status_detail === "review"` → `TaskUpdate(id, status: "completed", metadata: {status_detail: null})` for each, then `Skill("commit")`
- Re-review → `Skill("review")`
- Continue fixing → Resume fix loop at Step 5
- Refine → `Skill("refine")`
- Test plan → `Skill("test-plan")`

## Receiving Feedback

- **Verify** claims against code before agreeing/disagreeing
- Respond with evidence, not performative agreement
- Push back when: breaks functionality, violates YAGNI, incorrect
- No "done" claims without fresh evidence
