---
name: test-plan
description: "Analyze current diff, classify changes by risk, and produce structured manual test plan. Triggers: 'test plan', 'what should I test', 'manual testing', 'verification steps', 'QA checklist'. Exits early for trivial changes."
argument-hint: "[base..head | file-list | PR#]"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - AskUserQuestion
---

# Test Plan

Analyze changes, classify by risk, produce structured manual test plan. Auto-exits for trivial changes (docs-only, config formatting, comment edits).

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:

- Risk classification unclear (e.g., config change that might affect runtime) → ask user's assessment
- Testing environment ambiguous (e.g., requires specific device/OS) → clarify setup

Do NOT ask when the answer is obvious or covered by the diff context.

## Step 1: Scope

Parse $ARGUMENTS:

| Input        | Diff source                           |
| ------------ | ------------------------------------- |
| (none)       | `git diff HEAD`                       |
| `main..HEAD` | `git diff main..HEAD`                 |
| file list    | `git diff HEAD -- <files>`            |
| `#123`       | `gh pr diff 123`                      |

Collect the diff and `git diff --stat` for file summary.

**Early exit.** If ALL changed files match trivial patterns, output:

```
## Test Plan: No Manual Testing Required

Changes are trivial (docs/comments/formatting only). No functional
impact — skip manual verification.

Changed files:
<file list from --stat>
```

Trivial patterns:
- Docs-only (*.md, *.txt, LICENSE, CHANGELOG)
- Comment-only changes (no code lines changed)
- Whitespace/formatting-only changes
- CI config that doesn't affect build output (.github/workflows metadata like labels, assignees)

Exceptions (never trivial): SKILL.md, CLAUDE.md, *.mdx — these contain executable specifications.

If not trivial, proceed to Step 2.

## Step 2: Analyze

Read the full diff. For each changed file, classify the change:

### Risk Levels

**Critical** — changes that can break core functionality or lose data:
- Authentication/authorization logic
- Data persistence (migrations, schema changes, write paths)
- Payment/billing/financial calculations
- Security-sensitive code (crypto, input validation, CORS, CSP)
- Infrastructure (deployment configs, env vars, networking)

**High** — changes affecting user-visible behavior:
- UI components users interact with directly
- API endpoint behavior (request/response contracts)
- Business logic (state machines, validation rules, feature flags)
- Error handling paths that surface to users
- Performance-critical paths (queries, loops, caching)

**Medium** — changes with indirect user impact:
- Internal refactors that change control flow
- Dependency updates (library versions)
- Logging/monitoring/observability changes
- Test infrastructure changes
- Build configuration changes

**Low** — changes unlikely to cause user-facing issues:
- Code style/linting fixes within logic files
- Adding tests (not changing test infra)
- Internal documentation (code comments)
- Dev tooling (scripts, local config)

### Change Types

Tag each file with its change type to guide verification approach:
- **new-feature**: net-new functionality
- **behavior-change**: existing functionality modified
- **refactor**: same behavior, different structure
- **bugfix**: corrects incorrect behavior
- **config**: configuration/infrastructure
- **dependency**: library/package updates

## Step 3: Generate

Build the test plan from analyzed changes. Group verification steps by risk level, highest first. Each step must be concrete and actionable.

### Verification Step Format

Each step includes:
1. **What to test** — specific user action or system behavior
2. **How to test** — exact steps (click X, run Y, send request Z)
3. **Expected result** — observable outcome that confirms correctness
4. **Regression check** — related functionality that should still work

### Plan Structure

```markdown
## Test Plan: <one-line scope summary>

Risk profile: <N critical, N high, N medium, N low>
Estimated effort: <quick (5min) | moderate (15min) | thorough (30min+)>

### Critical Risk
<verification steps for critical-risk changes>

### High Risk
<verification steps for high-risk changes>

### Low Risk (spot-check)
<brief list — these don't need full verification steps>

### Regression Checklist
- [ ] <area adjacent to changes that should still work>
- [ ] <another adjacent area>
```

Omit empty risk sections entirely. Only include sections that have actual verification steps.

For **refactor** changes: focus verification on behavior preservation — same inputs produce same outputs, no regressions in adjacent features.

For **bugfix** changes: include the original bug reproduction steps as a verification step, plus edge cases around the fix boundary.

## Step 4: Output

Present the test plan.

If `$CLAUDE_NON_INTERACTIVE` is set, output the plan and stop. Do not present AskUserQuestion.

Offer next steps via AskUserQuestion:

- "Looks good, done" (Recommended) — description: "Accept test plan as-is"
- "Expand a section" — description: "Add more detail to a specific risk area"
- "Regenerate with different focus" — description: "Re-analyze with adjusted risk assessment"

If user selects "Expand a section": ask which section, then regenerate that section with more granular steps.

If user selects "Regenerate": ask what to adjust, then re-run from Step 2 with updated classification.
