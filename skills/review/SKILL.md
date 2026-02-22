---
name: review
description: "Adversarial code review dispatching parallel reviewer agents across correctness, security, architecture, and performance lenses. Triggers: 'review', 'review my changes', 'check this code', 'code review'. Use --team for 3-perspective mode. Do NOT use when: cosmetic polish only — use /refine. Do NOT use when: investigating unknown bug — use /debugging."
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

Three modes: solo (default), file-split (auto ≥15 files), perspective (--team). All consolidate into phase-structured output.

## Interviewing

See rules/skill-interviewing.md. Ask on: borderline severity judgment, unclear pattern violation (style vs correctness).

## Constants

- BASE=!`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`
- REPO_ROOT=!`git rev-parse --show-toplevel 2>/dev/null`
- CODEX_AVAILABLE=!`which codex >/dev/null 2>&1 && echo "true" || echo "false"`
- NON_INTERACTIVE=!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "true" || echo "false"`

## Step 1: Scope + Mode

Parse $ARGUMENTS: `--against <task-id>` (plan adherence), `--team` (perspective mode), remaining args override BASE.

| Input | Diff source |
|---|---|
| (none) | `git diff $BASE...HEAD` |
| `main..HEAD` | BASE=main |
| file list | `git diff HEAD -- <files>` + read |
| `#123` | `gh pr diff 123` |

Mode: `--team` → Perspective (3 specialists), ≥15 files → File-Split (~8/agent), else → Solo (2 lenses).
CODEX_TRIGGERED = (files≥5 OR lines≥200) AND CODEX_AVAILABLE.

## Step 2: Setup + Context

TaskCreate with `metadata: {type: "review", project: REPO_ROOT}`, set in_progress. `--continue`: resume existing — extract metadata.design, prepend to prompts.

Parallel: `git diff --stat`, `--name-only`, `git log --oneline`, `ck tool cochanges --base $BASE` (empty → skip completeness). `--against`: TaskGet for plan.

## Step 3: Dispatch Reviewers

All persistent-reviewer Task agents, spawn in ONE message. Full prompt templates in `references/reviewer-prompts.md`. If `--against`: append plan adherence check to every prompt.

- **Solo**: Correctness & Security + Architecture & Performance
- **File-Split**: Combined lens per ~8-file group
- **Perspective**: Architect + Code Quality + Devil's Advocate
- **Additional** (all modes): Completeness (if cochanges non-empty), Codex (if CODEX_TRIGGERED)

## Step 4: Consolidate

1. **Validate** (subagent-trust.md): spot-check 1-2 claims per reviewer; ALL codex claims. Codex duplicate → keep reviewer.
2. **Deduplicate**: same issue → highest severity/tier.
3. **Consensus**: critical from any reviewer survives. Non-critical needs 2+ at same tier. Solo: 2-of-2. Single-reviewer → IGNORE "1-of-N".
4. Sort by severity. **Never truncate.**

Output `# Adversarial Review Summary`: FIX table, collapsed IGNORE, --team tags/disagreements, verdict footer. Store via `ck plan create` + TaskUpdate metadata.design.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "AskUserQuestion: Fix all / Fix critical+high / Fix critical only / Skip fixes"`

## Step 5: Fix + Re-review Loop

Spawn general-purpose agent with FIX items → fix, verify, report. Then `Skill("refine")`.

Re-run Step 3, max 4 iterations. Track fixed_issues by (file, description) — not line numbers (they shift). Skip matches when consolidating. Exit: all resolved, user stops, or iteration 4.

## Step 6: Summary + Next

Output: Fixes Applied, Ignored, Remaining counts. If remaining + interactive: multiSelect to track as deferred-review tasks.

Close: TaskUpdate → completed. Next via AskUserQuestion based on outcome (clean/fixed/partial) → Skill dispatch (commit, review, refine, test-plan).

## Receiving Feedback

Verify claims against code. Push back when it breaks functionality or violates YAGNI. No "done" claims without fresh evidence.
