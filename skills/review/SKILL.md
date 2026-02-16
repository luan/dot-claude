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

- `--against <issue-id>`: work issue for plan adherence
- `--team`: force 3-perspective mode
- Remaining args:

| Input        | Diff source                             |
| ------------ | --------------------------------------- |
| (none)       | `git diff HEAD`                         |
| `main..HEAD` | `git diff main..HEAD`                   |
| file list    | `git diff HEAD -- <files>` + read files |
| `#123`       | `gh pr diff 123`                        |

Count files: `git diff --stat`

Choose mode:

- `--team` → **Perspective Mode** (3 specialists)
- 15+ files, no `--team` → **File-Split Mode**
- Otherwise → **Solo Mode** (2 lenses)

## Step 1b: Create Review Issue

```bash
work create "Review: <scope-summary>" --type chore --priority 2 \
  --labels review
work start <id>
```

If `--continue`: skip creation, find existing:

- $ARGUMENTS matches work ID → use it
- Else: `work list --status active --label review`, use first result
- `work show <id> --format=json` → read description
- Prepend to reviewers: "Previous findings:\n<description>\n\nContinue reviewing..."

## Step 2: Gather Context

1. Get diff
2. If `--against`: `work show <issue-id>` for plan
3. List changed files
4. Read all changed files in parallel

### Large Diff Handling

If total diff exceeds 3000 lines: for each file with >200 lines
of diff, truncate to first 50 + last 50 lines in the prompt
passed to subagents. Note truncations so reviewers know to
`Read` full files when needed.

## Step 3: Dispatch Reviewers

### Solo Mode (2 lenses)

Spawn 2 Task agents (persistent-reviewer) in SINGLE message. Each gets full diff + changed file contents.

Read references/prompts.md for Solo Mode lens prompt templates.

### File-Split Mode (>15 files)

Split files into groups of ~8. Spawn parallel Task agents, one per group. Each gets full diff for its group. Use same 2-lens prompt combined.

### Perspective Mode (--team)

Spawn EXACTLY 3 Task agents in SINGLE message. Each gets FULL changeset (no splitting).

Read references/prompts.md for Perspective Mode prompt templates.

If `--against`: append "Check plan adherence: implementation match plan? Missing/unplanned features? Deviations justified? Plan: {issue description}"

## Step 4: Consolidate + Present

0. **Validate reviewer output** (subagent-trust.md): spot-check 1-2
   specific file:line claims from each reviewer before consolidating.
   If a claimed issue doesn't exist at that location → discard it.
1. Deduplicate (same issue from multiple lenses → highest severity)
2. Sort by severity. **NEVER truncate.** Output EVERY finding.
3. --team only: tag [architect]/[code-quality]/[devil], detect consensus (2+ flag same issue), note disagreements

Output: `# Adversarial Review Summary`

- Sections by severity: Critical → High → Medium → Low
- --team adds: Consensus (top), Disagreements (bottom)
- Table: `| Severity | File:Line | Issue | Suggestion |`
- Footer: Verdict (APPROVE/COMMENTS/CHANGES), Blocking count, Review issue-id, "Clean review → /commit", "New work discovered → /prepare <issue-id>"

!`[ "$CLAUDE_NON_INTERACTIVE" = "1" ] && echo "Return findings to caller. Don't fix." || echo "Use AskUserQuestion: Fix all / Fix critical+high only / Fix critical only / Skip fixes"`

## Step 4b: Store Findings

Store consolidated findings in description:
`work edit <review-id> --description "<full-consolidated-output>"`

## Step 5: Dispatch Fixes

Spawn general-purpose agent. Read references/prompts.md for fix dispatch prompt template.

Fix agent also applies polish: flatten unnecessary nesting (early returns), remove code-restating comments and contextless TODOs, remove unused imports and debug artifacts. Never change behavior.

## Step 6: Re-review

Re-run Step 3 after fixes. Loop until clean or user stops.

## Step 6b: Close Review Issue + Approve Implementation

After review complete (user approves or skips fixes):
`work review <review-id>`
`work approve <review-id>`

Do NOT auto-approve implementation work. User must explicitly request approval.

## Step 7: Interactive Continuation

Note: Fix selection happens in Step 4 above. This step handles pipeline continuation after review completes.

Check for implementation issues in review: `work list --status review`
If any exist, note them in the prompt so the user knows approval is pending.

Context-aware next-step prompt based on review outcome:

**Clean review (no issues found):**

Use AskUserQuestion:

- "Approve + commit" (Recommended) — description: "Approve implementation work and create conventional commit"
- "Done for now" — description: "Leave issues in review for later"

**Issues found and fixed (fix loop completed):**

Use AskUserQuestion:

- "Re-review to verify fixes" (Recommended) — description: "Run review again to confirm fixes are clean (max 2 cycles)"
- "Approve + commit" — description: "Fixes look good, approve and commit"
- "Done for now" — description: "Leave issues in review for later"

**Issues found but not all fixed:**

Use AskUserQuestion:

- "Continue fixing" (Recommended) — description: "Address remaining issues"
- "Done for now" — description: "Leave issues in review for later"

Skill invocations based on user selection:

- "Approve + commit" → `work list --status review` → `work approve <id>` for each, then `Skill tool: skill="commit"`
- "Re-review to verify fixes" → `Skill tool: skill="review"`
- "Continue fixing" → Resume fix loop at Step 5
- "Done for now" → Exit skill (issues stay in review)

## Receiving Feedback

- **Verify** claims against code before agreeing/disagreeing
- Respond with evidence, not performative agreement
- Push back when: breaks functionality, violates YAGNI, incorrect
- No "done" claims without fresh evidence
