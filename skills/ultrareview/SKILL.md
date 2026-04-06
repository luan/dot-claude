---
name: ultrareview
description: "Deep adversarial code review with 5-7 parallel perspective specialists. Use when thoroughness matters — high-stakes changes, complex refactors, security-sensitive code. Triggers: 'ultrareview', 'deep review', 'thorough review', 'full review'. For quick reviews use /review instead."
argument-hint: "[base..head | file-list | PR#] [--auto critical|high|medium|all] [--loop]"
user-invocable: true
allowed-tools:
  - Agent
  - Skill
  - Read
  - Glob
  - Grep
  - "Bash(git diff:*)"
  - "Bash(git log:*)"
  - "Bash(git status:*)"
  - "Bash(ct tool:*)"
  - "Bash(ct review:*)"
  - "Bash(ct spec:*)"
  - "Bash(ct plan:*)"
  - "Bash(ct blueprint:*)"
  - "Bash(gh pr:*)"
  - "Bash(gh api:*)"
---

# Ultrareview

5-7 parallel perspective specialists with lane boundaries, shared concern tags, and a rigorous aggregation pipeline. Always runs simplify pre-pass. Auto-detects spec/plan for coherence review.

**NEVER review inline.** Always dispatch subagents via the Agent tool.

## Arguments

- `[base..head | file-list | PR#]` — diff source (default: branch diff vs parent)
- `--auto critical|high|medium|all` — auto-fix findings at or above the given severity. `all` = zero tolerance (every finding including nits gets fixed).
- `--loop` — enable fix + re-review loop. Without this, fixes are applied once with no re-review.

## Step 1: Scope

Resolve BASE: `gt parent 2>/dev/null || gt trunk 2>/dev/null || git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's|refs/remotes/||'`. Args override.

| Input        | Diff source                       |
| ------------ | --------------------------------- |
| (none)       | `git diff $BASE...HEAD`           |
| `main..HEAD` | BASE=main                         |
| file list    | `git diff HEAD -- <files>` + read |
| `#123`       | `gh pr diff 123`                  |

## Step 1.5: Simplify Pre-pass

**ALWAYS run** unless `#<PR>` input.

`Skill("simplify")`

## Step 2: Setup + Context

`ct tool gitcontext --base $BASE --stat --cochanges` → diff-stat, changed-files, log, cochanges.

**Auto-detect spec/plan** for coherence review:
```bash
BRANCH_SLUG=$(git branch --show-current | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g; s/-*$//')
PROJECT=$(git rev-parse --show-toplevel)
SPEC_FILE=$(ct blueprint search "$BRANCH_SLUG" --project "$PROJECT" --type spec 2>/dev/null | head -1)
PLAN_FILE=$(ct blueprint search "$BRANCH_SLUG" --project "$PROJECT" --type plan 2>/dev/null | head -1)
```
Set `$HAS_SPEC` = true if found. Read spec content for coherence reviewer.

**Detect primary language** from changed file extensions. Set `$LANG` if one dominates.

**Fetch PR context:**
```bash
PR_CONTEXT=$(gh pr view --json title,body,labels -q '{title,body,labels}' 2>/dev/null || echo "")
```

**Bugfix detection:** If commit messages contain "fix"/"bugfix"/"hotfix", classify changed files as production vs test. ALL test-only → verdict **FAIL** with Critical: "Bugfix contains no production code changes."

## Step 3: Dispatch Reviewers

**All agents spawned in ONE message.** Pass raw diffs — never summaries.

**Large diffs (>3000 lines):** Truncate files with >200 lines of diff to first 50 + last 50 lines. Note truncations.

Load perspective prompts from `${CLAUDE_SKILL_DIR}/perspectives/`. Each file has a `## Prompt` section.

**Core (always):** architect, code-quality, devils-advocate, operations, test-quality

**Conditional (same message):**
- `perspectives/coherence.md` — when `$HAS_SPEC` is true
- Language reviewer — when `$LANG` is set (template in `${CLAUDE_SKILL_DIR}/references/reviewer-prompts.md`)

**Additional (same message):**
- Completeness — if cochanges non-empty
- Codex — if available AND (files≥5 or lines≥200), or if `--auto all`

Append protocol to every prompt:
```
## Protocol
Return COMPLETE findings as text. Do NOT write files.
Structure findings as phases:

**Phase 1: Critical Issues** — numbered list
**Phase 2: Design Improvements** — numbered list
**Phase 3: Testing Gaps** — numbered list

Only include phases that have findings. Skip empty phases.
For each finding: file, line(s), what's wrong, suggested fix.
Stay in your lane — only flag issues in your domain.
Exception: cross-cutting shared concerns tagged [shared:<category>].
```

## Step 4: Aggregate

### 4a. Concatenate with source headers

### 4b. Group shared concerns
Collect `[shared:<category>]`-tagged findings. Group by category + file. Synthesize into multi-angle findings. Remove individuals from per-perspective sections.

### 4c. Consensus detection
Same file + same issue by 2+ perspectives = consensus. Critical from any reviewer survives. Non-critical needs 2+.

### 4d. Approach evaluation
1. **Goal alignment**: Does the diff achieve what the PR describes?
2. **Premise check**: Does the fix actually fix the stated problem?
3. **Approach fitness**: Right approach? Simpler alternatives?
4. **Scope assessment**: Appropriately scoped?

Rate: **Sound** | **Minor Concerns** | **Significant Concerns** | **Alternative Recommended**

### 4e. Correctness verification (MANDATORY)
Every finding verified against source. Read code at file:line ± 20 lines. Classify: Confirmed / False positive (REMOVE) / Pre-existing (downgrade) / Uncertain ([needs-review]).

Be aggressive about pruning. Log verification summary.

### 4f. Group by root cause
Multiple findings about same issue → ONE finding with multiple facets.

### 4g. Build unified output

Reviewer Summaries → Approach Assessment → Verification stats → Consensus findings → Perspective Disagreements → Design Coherence (if applicable) → Phase 1/2/3 non-consensus → Verdict (PASS / CHANGES_REQUESTED / FAIL)

## Step 5: Store

```bash
ct review create --topic "Review: $(git branch --show-current)" --project "$(git rev-parse --show-toplevel)"
```

Auto-link to spec if `$HAS_SPEC`: add `--source "<spec-stem>"`.

## Step 6: Fix

Determine fix scope from `--auto`:
- `--auto critical` → fix Critical only
- `--auto high` → fix Critical + High
- `--auto medium` → fix Critical + High + Medium
- `--auto all` → fix everything including nits
- No `--auto` → ask user what to fix

Spawn fix agent with FIX items → fix, verify, self-check, report.

**`--loop`:** Re-run Step 3 after fixes. Track fixed issues by (file, description). Max 4 iterations — unless `--auto all` which has no cap. Exit when zero findings remain, user stops, or cap hit.

Without `--loop`: single fix pass, no re-review.

## Step 6b: Failure Learning

After fixes, check if Critical/High findings revealed a codebase-specific antipattern. If yes: create project rule in `<project>/.claude/rules/<topic>.md`.

## Step 7: Summary + Next

Output: Fixes Applied, Ignored, Remaining. Suggest next steps.
