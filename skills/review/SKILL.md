---
name: review
description: "Adversarial code review: parallel reviewers find issues, fix with subagent pattern."
argument-hint: "[base..head | file-list | PR#] [--against <issue-id>]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - AskUserQuestion
  - Read
  - Bash
---

# Adversarial Review

Parallel adversarial reviewers → consolidate → fix via subagent.

## Step 0: Triage — Auto-Escalate

- `git diff --stat $ARGUMENTS` (or `gh pr diff $ARGUMENTS --stat`)
- Count files changed + unique top-level dirs
- **5+ files across 3+ dirs** OR auth/security/crypto touched → `Skill tool: team-review` with same args, STOP
- Otherwise continue

## Step 1: Determine Scope

Parse $ARGUMENTS:
- `--against <issue-id>`: beads issue for plan adherence
- Remaining args:

| Input | Diff source |
|-------|-------------|
| (none) | `git diff HEAD` |
| `main..HEAD` | `git diff main..HEAD` |
| file list | `git diff HEAD -- <files>` + read files |
| `#123` | `gh pr diff 123` |

## Step 2: Gather Context

1. Get diff
2. If `--against`: `bd show <issue-id>` for plan
3. List changed files
4. Read all changed files in parallel (full content)

## Step 3: Dispatch 4 Adversarial Reviewers

Spawn 4 Task agents (general-purpose) **ALL in a single message**. Each gets full diff + changed file contents + one adversarial lens.

If `--against` provided, append to each prompt: "Also check plan adherence: does implementation match plan? Missing/unplanned features? Deviations justified? Plan: {beads issue notes}"

### Lens 1: Architecture

```
You are an adversarial architecture reviewer. Find structural problems others miss.

{diff + full file contents}

Focus:
- Incomplete refactors (changed in some places, not others)
- Unnecessary abstractions, wrappers, indirection
- Dead code, stale flags, unused params
- Design pattern violations or inconsistencies
- Coupling that makes future changes harder
- Could this be simpler?

Output: table with Severity (Critical/High/Medium/Low) | File:Line | Issue | Suggestion
Then brief summary.
```

### Lens 2: Correctness & Data Integrity

```
You are an adversarial correctness reviewer. Find bugs and data integrity issues others miss.

{diff + full file contents}

Focus:
- Edge cases (empty, null, overflow, concurrent access)
- Invalid states possible? Race conditions?
- Resource leaks (unclosed handles, uncancelled tasks, missing cleanup)
- Silent failures, swallowed errors, missing fallbacks
- Off-by-one, wrong operator, logic inversions
- Multiple sources of truth, stale caches

Output: table with Severity (Critical/High/Medium/Low) | File:Line | Issue | Suggestion
Then brief summary.
```

### Lens 3: Security

```
You are an adversarial security reviewer. Find vulnerabilities others miss.

{diff + full file contents}

Focus:
- Injection (SQL, command, XSS, template)
- Auth/authz gaps (missing checks, privilege escalation)
- Data exposure (secrets in logs, oversharing in APIs, PII leaks)
- Input validation at trust boundaries
- Cryptographic misuse
- Dependency risks

Output: table with Severity (Critical/High/Medium/Low) | File:Line | Issue | Suggestion
Then brief summary.
```

### Lens 4: Performance

```
You are an adversarial performance reviewer. Find efficiency problems others miss.

{diff + full file contents}

Focus:
- Algorithmic complexity (O(n²) hidden in loops, unnecessary iterations)
- Memory (large allocations, retained references, unbounded growth)
- I/O (blocking calls, missing batching, N+1 queries)
- Concurrency (thread safety, deadlock potential, contention)
- Resource cleanup and lifecycle management

Output: table with Severity (Critical/High/Medium/Low) | File:Line | Issue | Suggestion
Then brief summary.
```

## Step 4: Consolidate & Present

Merge findings from all 4 reviewers:

1. Deduplicate (same issue from multiple lenses → keep highest severity)
2. Sort by severity

```markdown
# Adversarial Review Summary

| Severity | File:Line | Issue | Lens |
|----------|-----------|-------|------|
| Critical | ... | ... | ... |
| High | ... | ... | ... |

**Per-lens notes**: [one line each — what's good, what's concerning]

**Verdict**: APPROVE / APPROVE WITH COMMENTS / REQUEST CHANGES
**Blocking Issues**: N
```

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "Use AskUserQuestion: Fix all / Fix critical+high only / Fix critical only / Skip fixes"`

## Step 5: Dispatch Fixes

Spawn general-purpose agent with:

```
Fix these review issues in the code.

## Issues to Fix
{issues with file:line refs from review}

## Context
{what code is supposed to do}

## Your Job
1. Fix each listed issue
2. Verify fixes (syntax check, tests if quick)
3. Report what you fixed

Do NOT: fix unlisted things, refactor beyond needed, add features
```

## Step 6: Re-review

Re-run Step 3 after fixes. Loop until clean or user stops.

## Receiving Feedback

When user provides feedback on findings:

- **Verify** claims against code before agreeing/disagreeing
- Respond with evidence: "Verified: [evidence]. Implementing." or "Checked [X]. Disagree because [reason]."
- Never: "You're absolutely right!" / "Great catch!" / performative agreement without verification
- Push back when: breaks existing functionality, reviewer lacks context, violates YAGNI, technically incorrect
- No "done" claims without fresh evidence (test output, specific results)

## Skill Composition

| When | Invoke |
|------|--------|
| Complex bug found | `use Skill tool to invoke debugging` |
| Before claiming done | Run verification — evidence before assertions |
| User has feedback | `use Skill tool to invoke feedback` |
