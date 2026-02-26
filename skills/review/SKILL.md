---
name: review
description: "Adversarial code review dispatching parallel reviewer agents across correctness, security, architecture, and performance lenses. Triggers: 'review', 'review my changes', 'check this code', 'code review'. Use --team for 3-perspective mode. Do NOT use when: investigating unknown bug — use /debugging."
argument-hint: "[base..head | file-list | PR#] [--against <issue-id>] [--team] [--continue]"
user-invocable: true
allowed-tools:
  - Task
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

Three modes: solo (default), file-split (auto ≥15 files), perspective (--team). All consolidate into phase-structured output.

## Interviewing

See rules/skill-interviewing.md. Ask on: borderline severity, unclear pattern violation.

## Step 1: Scope + Mode

BASE=!`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`

Parse $ARGUMENTS: `--against <task-id>` (plan adherence), `--team` (perspective mode), remaining args override BASE.

| Input | Diff source |
|---|---|
| (none) | `git diff $BASE...HEAD` |
| `main..HEAD` | BASE=main |
| file list | `git diff HEAD -- <files>` + read |
| `#123` | `gh pr diff 123` |

Mode: `--team` → Perspective (3 specialists), ≥15 files → File-Split (~8/agent), else → Solo (2 lenses).

## Step 2: Setup + Context

TaskCreate `metadata: {type: "review", project: REPO_ROOT}`, in_progress. `--continue`: TaskList `metadata.type == "review"` + `in_progress`, first match; not found → stop. Resume: prepend metadata.design.

Parallel: `git diff --stat`, `--name-only`, `git log --oneline`, `ct tool cochanges --base $BASE` (unavailable → skip). `--against`: TaskGet for plan.

## Step 3: Dispatch Reviewers

All Task agents, spawn in ONE message. Prompts in `references/reviewer-prompts.md`. `--against`: append plan adherence to every prompt.

- **Solo**: Correctness & Security + Architecture & Performance
- **File-Split**: Combined lens per ~8-file group
- **Perspective**: Architect + Code Quality + Devil's Advocate
- **Additional**: Completeness (if cochanges non-empty), Codex (if available AND files≥5 or lines≥200)

## Step 4: Consolidate

1. **Validate**: spot-check 1-2 claims per reviewer; ALL codex claims. Codex duplicate → keep reviewer version.
2. **Deduplicate**: same issue → highest severity.
3. **Consensus**: critical from any reviewer survives. Non-critical needs 2+ at same tier. Solo: both lenses must agree. Single-reviewer → IGNORE "1-of-N".
4. Sort by severity (Critical > High > Medium > Low). **Never truncate.** Judge each finding independently — one false claim doesn't taint others.

Output `# Adversarial Review Summary`:
- **FIX table** columns: Severity | File | Finding | Recommendation. Severity ∈ {Critical, High, Medium, Low}.
- **IGNORE** section (collapsed): findings below consensus threshold, labeled "1-of-N".
- **--team disagreements**: when specialists differ on severity, show attribution (e.g., "Architect: High, Code Quality: Medium → resolved: High") before the resolved row.
- **Verdict footer**: PASS (no FIX items), CHANGES_REQUESTED (any FIX items), FAIL (any Critical).

Store via TaskUpdate metadata.design.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "AskUserQuestion: Fix all / Fix critical+high / Fix critical only / Skip fixes"`

## Step 5: Fix + Re-review Loop

Spawn agent with FIX items → fix, verify, self-check (remove debug artifacts, low-value comments, unused imports), report.

Re-run Step 3, max 4 iterations. Track fixed_issues by (file, description) — not line numbers. Skip matches when consolidating. Exit: all resolved, user stops, or iteration 4.

## Step 6: Summary + Next

Output: Fixes Applied, Ignored, Remaining. Remaining + interactive: multiSelect to defer. Close: TaskUpdate → completed. Next via AskUserQuestion.

**Receiving feedback:** Verify claims by reading the file. Push back with evidence when feedback breaks functionality.
