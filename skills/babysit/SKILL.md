---
name: babysit
description: "Watch a Graphite PR stack until all PRs merge, automatically fixing CI failures and review comments. Single-pass: checks every PR once, fixes what it can, reports status. Use with /loop for recurring checks (e.g. /loop 10m /babysit). Triggers: 'babysit', 'babysit my PRs', 'babysit stack', 'watch my stack', 'keep PRs green', 'monitor until merged', 'shepherd PRs', 'keep fixing until merged'. Use whenever the user wants ongoing or one-shot PR stack maintenance."
argument-hint: "[skills...]"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - Skill
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TodoWrite
---

# Babysit

Single-pass PR stack shepherd. Checks every PR in a Graphite stack, runs specified skills to fix CI failures and review comments, reports status. Designed to be invoked repeatedly via `/loop` (e.g. `/loop 10m /babysit`).

Each invocation is stateless: discover the stack, check each PR, act, report, exit.

## Arguments

`[skills...]`

Remaining tokens are skill names. Defaults to `pr-ci pr-comments`.

**Examples:**
- `/babysit` — pr-ci + pr-comments
- `/babysit pr-ci` — CI only
- `/babysit pr-ci pr-comments review` — three skills
- `/loop 10m /babysit` — recurring babysit every 10m
- `/loop 5m /babysit pr-ci` — recurring, CI only, every 5m

---

## Invocation flow

### [1] Parse arguments

Split by whitespace. All tokens are skill names. If empty, default to `["pr-ci", "pr-comments"]`.

### [2] Discover the stack

```bash
gt log --stack 2>&1
```

Parse branch names in bottom-to-top order. For each branch:

```bash
gh pr list --head <branch> --json number,state --jq '.[0]'
```

Collect `{num, branch}`. Warn the user if any branch has no PR ("branch X has no PR — submit first?").

### [3] Safety: uncommitted changes

```bash
git status --porcelain
```

If non-empty → "Babysit check skipped: uncommitted changes in worktree. Will retry next invocation." → exit.

Save the current branch for restoration later:
```bash
git branch --show-current
```

### [4] Detect newly merged parent PRs

When a parent PR merges, child branches go stale — their CI tests against old parent code. Detect this before doing per-PR work.

Do NOT use `gt log --stack`'s `(needs restack)` flag — it fires on any trunk divergence, causing unnecessary restacks in active repos. Instead, check each PR's state:

For each PR:

```bash
gh pr view <num> --json state --jq '.state'
```

If any returns `MERGED`:

1. Sync with remote:
   ```bash
   gt sync 2>&1
   ```
   `gt sync` fetches remote, removes merged branches, and restacks onto trunk. **Never** use `gt track`, `gt delete`, or manual reparenting.
2. If `gt sync` reports conflicts:
   ```
   Skill("gt:restack")
   ```
3. Push the restacked branches:
   ```
   Skill("gt:submit")
   ```
4. **Exit early** — don't fix anything this pass. CI needs to re-run against restacked code. Report: "Parent PR(s) merged — restacked and pushed. Waiting for fresh CI."

If no PRs are newly merged → proceed to step 5.

### [5] Process each PR (bottom to top)

Process from the bottom of the stack upward. Fixes on lower branches may resolve issues on upper branches after restacking.

For each open PR:

```bash
gh pr view <num> --json state,reviewDecision --jq '{state,reviewDecision}'
```

**Routing by state:**

| state | Action |
|-------|--------|
| MERGED | Skip |
| CLOSED | Report "closed", skip |
| OPEN | Process (below) |

For each open PR, run comments and CI as **independent** concerns. Approval status gates CI fixing but NOT comment checking.

**GitHub `reviewDecision` quirk:** An empty string `""` does NOT mean "not approved." GitHub resets `reviewDecision` to `""` when a bot reviewer posts a review after a human approved. Treat `""` the same as `"APPROVED"` — only `"REVIEW_REQUIRED"` or `"CHANGES_REQUESTED"` mean explicitly unapproved.

```bash
gt checkout <branch>
```

**a) Check comments** (if `pr-comments` or `pr-fix-comments` in skills)

Always run on every open PR regardless of approval or CI status. Comments and CI are independent — unresolved comments block approval.

```
Skill("pr-comments", "--auto")
```

The skill exits quickly if no unresolved comments — safe to invoke unconditionally.

**b) Check CI** (if `pr-ci` in skills)

```bash
gh pr checks <num> --json name,state,bucket --jq '[.[] | {name,state,bucket}]'
```

Skip CI fixing if:
- Any check is `IN_PROGRESS` or `QUEUED` — don't fix stale failures while a new run is pending
- `reviewDecision` is `"REVIEW_REQUIRED"` or `"CHANGES_REQUESTED"` — reviewer may request changes that invalidate current code

If checks have `bucket: "fail"` and CI fixing is not skipped:
```
Skill("pr-ci", "--auto")
```

Never classify CI failures yourself. Always delegate to `pr-ci`.

**c) Other skills**

For each remaining skill not `pr-ci` or `pr-comments`:
```
Skill("<name>", "--auto")
```

All skills invoked with `--auto` because this may run unattended.

**Error handling:** If `gt checkout <branch>` fails, skip this PR and report the error. Don't abort — continue to the next PR.

### [6] Restack and push

After processing all PRs, if any skill made changes (new commits):

```
Skill("gt:restack")
Skill("gt:submit")
```

### [7] Restore branch

```bash
gt checkout <saved-branch>
```

### [8] Completion check and reporting

If **all** PRs are merged or closed:

Report: "All PRs merged. Babysit complete."

Otherwise, format the status report:

```
Babysit: 1/3 merged.
  #123 (branch-a): merged
  #124 (branch-b): CI fixed + pushed, comments addressed (2 resolved)
  #125 (branch-c): CI in progress (skipped), comments checked (0 unresolved)
```

---

## Task management

Use TaskCreate/TaskUpdate/TaskList or TodoWrite to track progress when useful — e.g. tracking which PRs have been processed, what actions were taken.

## Stopping babysit

| Trigger | How |
|---------|-----|
| All merged | Reports "complete" — stop the `/loop` manually |
| User cancels | Stop the `/loop` |
| Session ends | Loop dies with session |
| 7-day expiry | Loop auto-expires |
