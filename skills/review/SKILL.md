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

TaskCreate with `metadata: {type: "review", project: REPO_ROOT}`, set in_progress. `--continue`: TaskList filtered by `metadata.type == "review"` + `status == "in_progress"`, first match; not found → "No review to continue", stop. Resume: extract metadata.design, prepend.

Parallel: `git diff --stat`, `--name-only`, `git log --oneline`, `ck tool cochanges --base $BASE` (empty/unavailable → skip completeness). `--against`: TaskGet for plan.

## Step 3: Dispatch Reviewers

All persistent-reviewer Task agents, spawn in ONE message. Full prompt templates in `references/reviewer-prompts.md`. If `--against`: append plan adherence check to every prompt.

- **Solo**: Correctness & Security + Architecture & Performance
- **File-Split**: Combined lens per ~8-file group
- **Perspective**: Architect + Code Quality + Devil's Advocate
- **Additional** (all modes): Completeness (if cochanges non-empty), Codex (if `which codex` succeeds AND files≥5 or lines≥200)

## Step 4: Consolidate

1. **Validate** (subagent-trust.md): spot-check 1-2 claims per reviewer; ALL codex claims (codex has no codebase context, so hallucination rate is higher). Codex duplicate → keep reviewer version.
2. **Deduplicate**: same issue → highest severity/tier.
3. **Consensus**: critical from any reviewer survives (false negative on security/data-loss is far costlier than a false positive). Non-critical needs 2+ at same tier (single-reviewer non-critical findings are often style opinions; consensus filters noise). Solo: both lenses must agree. Single-reviewer → IGNORE "1-of-N".
4. Sort by severity. **Never truncate.**

Output `# Adversarial Review Summary`: FIX table, collapsed IGNORE, --team tags/disagreements, verdict footer. Store via `ck plan create` + TaskUpdate metadata.design.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "AskUserQuestion: Fix all / Fix critical+high / Fix critical only / Skip fixes"`

## Step 5: Fix + Re-review Loop

Spawn general-purpose agent with FIX items → fix, verify, report. Then `Skill("refine")`.

Re-run Step 3, max 4 iterations (beyond 4, fix quality degrades as context fills with prior findings). Track fixed_issues by (file, description) — not line numbers (they shift). Skip matches when consolidating. Exit: all resolved, user stops, or iteration 4.

## Step 6: Summary + Next

Output: Fixes Applied, Ignored, Remaining. If remaining + interactive: multiSelect to track as deferred-review tasks. Close: TaskUpdate → completed. Next via AskUserQuestion (commit/review/refine/test-plan).

**Receiving feedback:** Verify claims by reading the file. Push back with evidence when feedback breaks functionality or violates YAGNI.
