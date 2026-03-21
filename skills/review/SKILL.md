---
name: review
description: "Thorough adversarial code review covering correctness, security, architecture, and performance. Triggers: 'review', 'review my changes', 'check this code', 'code review'. Use --team for 3-perspective mode. Do NOT use when: investigating unknown bug — use /debugging."
argument-hint: "[base..head | file-list | PR#] [--against <issue-id>] [--team] [--perfection] [--continue] [--auto]"
user-invocable: true
allowed-tools:
  - Agent
  - Skill
  - Read
  - "Bash(git diff:*)"
  - "Bash(git log:*)"
  - "Bash(git status:*)"
  - "Bash(ct tool:*)"
  - "Bash(gh pr:*)"
  - "Bash(gh api:*)"
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - Write
---

# Adversarial Review

Three modes: solo (default), file-split (auto when ≥15 files), perspective (--team). All consolidate into phase-structured output.

Solo has two sub-modes based on diff size:
- **Solo-combined** (<500 diff lines): single agent with all lenses — avoids duplicate reads and overlapping findings on small diffs.
- **Solo-split** (≥500 diff lines): two agents (Correctness & Security + Architecture & Performance).

**`--perfection` mode:** Zero tolerance. Loop until every finding — including nits — is resolved. No findings survive. Overrides mode selection → always uses Perspective (3 specialists) + Completeness + Codex (if available), regardless of diff size. Reviewers are additionally prompted with a `{perfection_block}` that demands: trace every code path end-to-end, verify the diff actually solves the stated problem, read production code beyond the diff to check for latent issues the change exposes. The fix loop (Step 5) runs with NO iteration cap and treats ALL severities as FIX (nothing is IGNORE). The loop continues until a review pass returns zero findings — not even nits. Use for cherry-picks, high-stakes bugfixes, or when you want the code to be immaculate.

**NEVER review inline.** Always dispatch at least one subagent via the Agent tool. The orchestrator reading the diff and writing a verdict directly is not a review — it's a skim. In a real session, the orchestrator did an inline "review" of a 242-line bugfix, read 80 lines of 1 of 3 files, declared PASS, and the bug was still present. Subagents read all files, cross-reference patterns, and catch what a quick skim misses.

## Interviewing

See rules/skill-interviewing.md.

## Step 1: Scope + Mode

BASE=!`gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`

Parse $ARGUMENTS: `--against <task-id>` (plan adherence), `--team` (perspective mode), `--no-simplify` (skip pre-pass), remaining args override BASE.

| Input        | Diff source                       |
| ------------ | --------------------------------- |
| (none)       | `git diff $BASE...HEAD`           |
| `main..HEAD` | BASE=main                         |
| file list    | `git diff HEAD -- <files>` + read |
| `#123`       | `gh pr diff 123`                  |

Mode: `--team` → Perspective (3 specialists), ≥15 files → File-Split (~8/agent), else → Solo. Solo sub-mode: count diff lines from `git diff --stat`; <500 lines → Solo-combined (1 agent, all lenses), ≥500 lines → Solo-split (2 agents by concern).

## Step 1.5: Simplify Pre-pass (mandatory)

**ALWAYS run.** The only valid skip condition is `--continue` or `#<PR>` input. Do NOT invent skip reasons ("test-only changes", "already clean", "already carefully designed"). These are not valid — the assistant fabricated them in a real session and missed real duplication, naming, and reuse issues that simplify later caught.

`Skill("simplify")`

Cleans up quality and efficiency issues before the adversarial review — reduces noise so reviewers focus on real bugs, not style. Any edits simplify makes become part of the diff reviewers see.

## Step 2: Setup + Context

TaskCreate `metadata: {type: "review", project: REPO_ROOT}`, in_progress. `--continue`: TaskList `metadata.type == "review"` + `in_progress`, first match; not found → stop. Resume: prepend metadata.design.

`ct tool gitcontext --base $BASE --stat --cochanges` → diff-stat, changed-files, log, cochanges (no full diff). `--against`: TaskGet for plan.

## Step 2.5: Bugfix Scope Validation

When reviewing a bugfix (commit message contains "fix", "bugfix", "hotfix", or `--against` references a bug/issue), run a scope check before dispatching reviewers:

1. Classify changed files as production vs test (test files match `*test*`, `*spec*`, `*mock*`, `*fixture*`, or live under `tests/`/`test/`/`__tests__/`).
2. If ALL changed files are test-only (zero production code changes), verdict is **FAIL** with a Critical finding: "Bugfix contains no production code changes — tests alone cannot fix the reported bug."
3. If production files are changed, proceed normally. Reviewers should still verify the production changes actually address the reported bug (not just related code).

This prevents rubber-stamping test-only diffs as valid bugfixes.

## Step 3: Dispatch Reviewers

All agents, spawn in ONE message. Prompts in `${CLAUDE_SKILL_DIR}/references/reviewer-prompts.md`. `--against`: append plan adherence to every prompt.

- **Solo-combined** (<500 diff lines): single agent with all lenses (Correctness, Security, Architecture, Performance)
- **Solo-split** (≥500 diff lines): Correctness & Security + Architecture & Performance
- **File-Split**: Combined lens per ~8-file group
- **Perspective**: Architect + Code Quality + Devil's Advocate
- **Additional**: Completeness (if cochanges non-empty), Codex (if available AND files≥5 or lines≥200)

## Step 4: Consolidate

1. **Validate**: spot-check 1-2 claims per reviewer; ALL codex claims. Codex duplicate → keep reviewer version.
2. **Deduplicate**: same issue → highest severity.
3. **Consensus**: critical from any reviewer survives. Non-critical needs 2+ at same tier. Solo-split: both lenses must agree. Solo-combined: single reviewer, all findings stand (no consensus filter). Single-reviewer in multi-agent mode → IGNORE "1-of-N".
4. Sort by severity (Critical > High > Medium > Low). **Never truncate.** Judge each finding independently — one false claim doesn't taint others.

Output `# Adversarial Review Summary`:

- **FIX table** columns: Severity | File | Finding | Recommendation. Severity ∈ {Critical, High, Medium, Low}.
- **IGNORE** section (collapsed): findings below consensus threshold, labeled "1-of-N".
- **--team disagreements**: when specialists differ on severity, show attribution (e.g., "Architect: High, Code Quality: Medium → resolved: High") before the resolved row.
- **Verdict footer**: PASS (no FIX items), CHANGES_REQUESTED (any FIX items), FAIL (any Critical).

Store via `ct plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "review"` + TaskUpdate metadata.design.

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "AskUserQuestion: Fix all / Fix critical+high / Fix critical only / Skip fixes"`

`--auto` → fix critical+high automatically (skip AskUserQuestion).

## Step 5: Fix + Re-review Loop

Spawn agent with FIX items → fix, verify, self-check (remove debug artifacts, low-value comments, unused imports), report.

Re-run Step 3, max 4 iterations. Track fixed_issues by (file, description) — not line numbers (lines shift after edits). Skip matches when consolidating. Exit: all resolved, user stops, or iteration 4.

**`--perfection` override:** No iteration cap. ALL findings are FIX (nothing is IGNORE — nits, style, naming, everything gets fixed). The loop continues until a review pass returns **zero findings of any severity**. Exit only when reviewers have nothing left to say. This can be expensive but produces immaculate code.

## Step 6: Summary + Next

Output: Fixes Applied, Ignored, Remaining. `--auto` → skip defer selection, complete task, stop. Without `--auto` → Remaining + interactive: multiSelect to defer. Close: TaskUpdate → completed. Next via AskUserQuestion.

**Receiving feedback:** Verify claims by reading the file. Push back with evidence when feedback breaks functionality.
