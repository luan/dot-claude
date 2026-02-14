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

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Severity judgment borderline (medium vs high) → ask user's priority
- Pattern violation unclear (style preference vs correctness issue) → clarify importance

Do NOT ask when the answer is obvious or covered by the task brief.

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

Read references/prompts.md for Solo Mode lens prompt templates.

### File-Split Mode (>15 files)

Split files into groups of ~8. Spawn parallel Task agents, one per group. Each gets full diff for its group. Use same 2-lens prompt combined.

### Perspective Mode (--team)

Spawn EXACTLY 3 Task agents in SINGLE message. Each gets FULL changeset (no splitting).

Read references/prompts.md for Perspective Mode prompt templates.

If `--against`: append "Check plan adherence: implementation match plan? Missing/unplanned features? Deviations justified? Plan: {beads design field}"

## Step 4: Consolidate + Present

1. Deduplicate (same issue from multiple lenses → highest severity)
2. Sort by severity. **NEVER truncate.** Output EVERY finding.
3. --team only: tag [architect]/[code-quality]/[devil], detect consensus (2+ flag same issue), note disagreements

Output: `# Adversarial Review Summary`
- Sections by severity: Critical → High → Medium → Low
- --team adds: Consensus (top), Disagreements (bottom)
- Table: `| Severity | File:Line | Issue | Suggestion |`
- Footer: Verdict (APPROVE/COMMENTS/CHANGES), Blocking count, Review bead-id, "Clean review → /refine then /commit", "New work discovered → /prepare <bead-id>"

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "Use AskUserQuestion: Fix all / Fix critical+high only / Fix critical only / Skip fixes"`

## Step 4b: Store Findings

Store consolidated findings in design field:
`bd edit <review-id> --design "<full-consolidated-output>"`

## Step 5: Dispatch Fixes

Spawn general-purpose agent (model: sonnet). Read references/prompts.md for fix dispatch prompt template.

## Step 6: Re-review

Re-run Step 3 after fixes. Loop until clean or user stops.

## Step 6b: Close Review Bead

After review complete (user approves or skips fixes):
`bd close <review-id>`

## Step 7: Interactive Continuation

Note: Fix selection happens in Step 4 above. This step handles pipeline continuation after review completes.

Context-aware next-step prompt based on review outcome:

**Clean review (no issues found):**

Use AskUserQuestion:
- "Continue to /refine" (Recommended) — description: "Polish code style, imports, comments"
- "Skip to /commit" — description: "Code is ready, go straight to commit"
- "Done for now" — description: "Leave bead in_progress for later /resume-work"

**Issues found and fixed (fix loop completed):**

Use AskUserQuestion:
- "Re-review to verify fixes" (Recommended) — description: "Run review again to confirm fixes are clean (max 2 cycles, then default to /refine)"
- "Continue to /refine" — description: "Fixes look good, move to polish"
- "Done for now" — description: "Leave bead in_progress for later /resume-work"

**Issues found but not all fixed:**

Use AskUserQuestion:
- "Continue fixing" (Recommended) — description: "Address remaining issues"
- "Continue to /refine anyway" — description: "Move on despite remaining issues"
- "Done for now" — description: "Leave bead in_progress for later /resume-work"

Skill invocations based on user selection:
- "Continue to /refine" → `Skill tool: skill="refine"`
- "Skip to /commit" → `Skill tool: skill="commit"`
- "Re-review to verify fixes" → `Skill tool: skill="review"`
- "Continue fixing" → Resume fix loop at Step 5
- "Done for now" → Exit skill

## Receiving Feedback

- **Verify** claims against code before agreeing/disagreeing
- Respond with evidence, not performative agreement
- Push back when: breaks functionality, violates YAGNI, incorrect
- No "done" claims without fresh evidence
