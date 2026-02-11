---
name: review
description: "Adversarial code review with parallel reviewers. Triggers: 'review', 'review my changes', 'check this code', 'code review'. Use --team for 3-perspective mode."
argument-hint: "[base..head | file-list | PR#] [--against <issue-id>] [--team] [--continue]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - AskUserQuestion
  - Read
  - Bash
---

# Adversarial Review

Three modes: solo (default), file-split (auto for large diffs), perspective (--team). All consolidate findings into phase-structured output.

## Step 1: Scope + Mode

Parse $ARGUMENTS:
- `--against <issue-id>`: beads issue for plan adherence
- `--team`: force 3-perspective mode
- Remaining args:

| Input | Diff source |
|-------|-------------|
| (none) | `git diff HEAD` |
| `main..HEAD` | `git diff main..HEAD` |
| file list | `git diff HEAD -- <files>` + read files |
| `#123` | `gh pr diff 123` |

Count files: `git diff --stat`

Choose mode:
- `--team` → **Perspective Mode** (3 specialists)
- 15+ files, no `--team` → **File-Split Mode**
- Otherwise → **Solo Mode** (2 lenses)

## Step 1b: Create Review Bead

```bash
bd create "Review: <scope-summary>" --type task --priority 2 --validate
bd lint <id>
bd update <id> --status in_progress
```

If `--continue`: skip creation, find existing:
- $ARGUMENTS matches beads ID → use it
- Else: `bd list --status in_progress --type task`, find first with "Review:"
- `bd show <id> --json` → read design field
- Prepend to reviewers: "Previous findings:\n<design>\n\nContinue reviewing..."

## Step 2: Gather Context

1. Get diff
2. If `--against`: `bd show <issue-id>` for plan
3. List changed files
4. Read all changed files in parallel

## Step 3: Dispatch Reviewers

### Solo Mode (2 lenses)

Spawn 2 Task agents (persistent-reviewer) in SINGLE message. Each gets full diff + changed file contents.

**Lens 1: Correctness & Security**
```
You are an adversarial correctness and security reviewer.

{diff + full file contents}

Focus:
- Edge cases (empty, null, overflow, concurrent access)
- Invalid states, race conditions
- Resource leaks (unclosed handles, missing cleanup)
- Silent failures, swallowed errors
- Off-by-one, logic inversions
- Injection (SQL, command, XSS, template)
- Auth/authz gaps, data exposure, cryptographic misuse

Output: table with Severity | File:Line | Issue | Suggestion
Then brief summary.
```

**Lens 2: Architecture & Performance**
```
You are an adversarial architecture and performance reviewer.

{diff + full file contents}

Focus:
- Incomplete refactors, dead code, unused params
- Unnecessary abstractions, coupling
- Could this be simpler?
- O(n^2) in loops, unnecessary allocations
- Memory (retained refs, unbounded growth)
- I/O (blocking calls, N+1 queries)
- Concurrency (thread safety, deadlock, contention)

Output: table with Severity | File:Line | Issue | Suggestion
Then brief summary.
```

### File-Split Mode (>15 files)

Split files into groups of ~8. Spawn parallel Task agents, one per group. Each gets full diff for its group. Use same 2-lens prompt combined.

### Perspective Mode (--team)

Spawn EXACTLY 3 Task agents in SINGLE message. Each gets FULL changeset (no splitting).

**Perspective 1: Architect** (model: opus)
```
Architecture reviewer. Focus:
- System boundaries, coupling, scalability
- Design flaws, incomplete abstractions
- Dependency direction, module cohesion
- Could this be simpler or more maintainable?

Tag: [architect]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
```

**Perspective 2: Code Quality** (model: sonnet)
```
Code quality reviewer. Focus:
- Readability, naming, error handling
- Edge cases, off-by-one, null safety
- Consistency with surrounding code
- Resource leaks, missing cleanup

Tag: [code-quality]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
```

**Perspective 3: Devil's Advocate** (model: opus)
```
Devil's advocate reviewer. Focus:
- Failure modes others miss
- Security: injection, auth gaps, data exposure
- Bad assumptions, race conditions
- What breaks under load, bad input, or partial failure?

Tag: [devil]
Output: Phase 1 (Critical) → Phase 2 (Design) → Phase 3 (Testing Gaps)
```

If `--against`: append "Check plan adherence: implementation match plan? Missing/unplanned features? Deviations justified? Plan: {beads design field}"

## Step 4: Consolidate + Present

1. Deduplicate (same issue from multiple lenses → highest severity)
2. Sort by severity. **NEVER truncate.** Output EVERY finding.
3. --team only: tag [architect]/[code-quality]/[devil], detect consensus (2+ flag same issue), note disagreements

Output: `# Adversarial Review Summary`
- Sections by severity: Critical → High → Medium → Low
- --team adds: Consensus (top), Disagreements (bottom)
- Table: `| Severity | File:Line | Issue | Suggestion |`
- Footer: Verdict (APPROVE/COMMENTS/CHANGES), Blocking count, Review bead-id, "Next: `/prepare <bead-id>`"

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "Use AskUserQuestion: Fix all / Fix critical+high only / Fix critical only / Skip fixes"`

## Step 4b: Store Findings

Store consolidated findings in design field:
`bd edit <review-id> --design "<full-consolidated-output>"`

## Step 5: Dispatch Fixes

Spawn general-purpose agent (model: sonnet):

```
Fix these review issues in code.

## Issues to Fix
{issues with file:line refs}

## Your Job
1. Fix each listed issue
2. Verify fixes (syntax check, tests if quick)
3. Report what you fixed

Do NOT: fix unlisted things, refactor beyond needed, add features
```

## Step 6: Re-review

Re-run Step 3 after fixes. Loop until clean or user stops.

## Step 6b: Close Review Bead

After review complete (user approves or skips fixes):
`bd close <review-id>`

## Receiving Feedback

- **Verify** claims against code before agreeing/disagreeing
- Respond with evidence, not performative agreement
- Push back when: breaks functionality, violates YAGNI, incorrect
- No "done" claims without fresh evidence
