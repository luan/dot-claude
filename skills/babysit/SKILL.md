---
name: babysit
description: "Watch a Graphite PR stack until all PRs merge, automatically fixing CI failures and review comments. Single-pass: checks every PR once, fixes what it can, reports status. Use with /loop for recurring checks (e.g. /loop 10m /babysit). Triggers: 'babysit', 'babysit my PRs', 'babysit stack', 'watch my stack', 'keep PRs green', 'monitor until merged', 'shepherd PRs', 'keep fixing until merged'. Use whenever the user wants ongoing or one-shot PR stack maintenance."
argument-hint: "[skills...]"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Edit
  - Glob
  - Grep
  - Skill
  - Agent
  - TaskCreate
  - TaskUpdate
  - TaskGet
  - TaskList
  - CronList
  - CronDelete
---

# Babysit

Single-pass PR stack shepherd. Checks every PR in a Graphite stack, runs specified skills to fix CI failures and review comments, reports status. Designed to be invoked repeatedly via `/loop` (e.g. `/loop 10m /babysit`).

Each invocation is self-contained: discover or load state, act on current reality, update state, exit.

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

On every invocation, determine mode by checking for an existing tracking task.

### [1] Find or create tracking task

```
TaskList → look for a task with metadata.type == "babysit" and status == "in_progress"
```

**Existing task found** → load state, go to **Check pass** (step 3).

**No existing task** → **Setup** (step 2).

### [2] Setup (first invocation only)

#### Parse arguments

Split by whitespace. All tokens are skill names. If empty, default to `["pr-ci", "pr-comments"]`.

#### Discover the stack

```bash
gt log --stack 2>&1
```

Parse branch names in bottom-to-top order. For each branch:

```bash
gh pr list --head <branch> --json number,state --jq '.[0]'
```

Collect `{num, branch}`. Warn the user if any branch has no PR ("branch X has no PR — submit first?").

#### Create tracking task

```
TaskCreate(
  subject: "Babysit: #X, #Y, #Z",
  activeForm: "Babysitting",
  metadata: {
    type: "babysit",
    prs: [{num: N, branch: "name", merged: false}, ...],
    skills: ["pr-ci", "pr-comments"],
    work_dir: "<cwd>",
    idle_checks: 0,
    iteration_count: 0,
    started_at: "<ISO 8601>"
  }
)
TaskUpdate(<id>, status: "in_progress")
```

#### Report to user

Tell them:
- PR count, numbers, and branches
- Skills that will run (all invoked with `--auto`)
- How to run recurring: `/loop 10m /babysit`
- How to see status: look for the babysit task in `TaskList`

Then proceed directly to **Check pass** (step 3).

### [3] Check pass

Load state from the tracking task: `prs`, `skills`, `work_dir`, `idle_checks`, `iteration_count`, `started_at`.

Increment `iteration_count` in metadata.

#### Safety: uncommitted changes

```bash
git status --porcelain
```

If non-empty → "Babysit check skipped: uncommitted changes in worktree. Will retry next invocation." → exit.

Save the current branch for restoration later:
```bash
git branch --show-current
```

#### Detect newly merged parent PRs

When a parent PR merges, child branches go stale — their CI tests against old parent code. Detect this before doing per-PR work.

Do NOT use `gt log --stack`'s `(needs restack)` flag — it fires on any trunk divergence, causing unnecessary restacks in active repos. Instead, check if any previously-unmerged PR is now merged:

For each PR in `metadata.prs` where `merged` is false:

```bash
gh pr view <num> --json state --jq '.state'
```

If any returns `MERGED`:

1. Mark the newly merged PRs as `merged: true` in metadata.
2. Sync with remote:
   ```bash
   gt sync 2>&1
   ```
   `gt sync` fetches remote, removes merged branches, and restacks onto trunk. **Never** use `gt track`, `gt delete`, or manual reparenting.
3. If `gt sync` reports conflicts:
   ```
   Skill("gt:restack")
   ```
4. Push the restacked branches:
   ```
   Skill("gt:submit")
   ```
5. Update task metadata with merged flags.
6. **Exit early** — don't fix anything this pass. CI needs to re-run against restacked code. Report: "Parent PR(s) merged — restacked and pushed. Waiting for fresh CI."

If no PRs are newly merged → proceed to step 4.

### [4] Process each PR (bottom to top)

Process from the bottom of the stack upward. Fixes on lower branches may resolve issues on upper branches after restacking.

For each PR where `merged` is false:

```bash
gh pr view <num> --json state,reviewDecision --jq '{state,reviewDecision}'
```

**Routing by state:**

| state | Action |
|-------|--------|
| MERGED | Mark `merged: true`, skip |
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

### [5] Restack and push

After processing all PRs, if any skill made changes (new commits):

```
Skill("gt:restack")
Skill("gt:submit")
```

### [6] Restore branch

```bash
gt checkout <saved-branch>
```

### [7] Build structured status

Before reporting, build a status object per PR tracking exactly what happened. This prevents fabricating information.

```
status = {
  pr_num: N,
  branch: "name",
  state: "OPEN" | "MERGED" | "CLOSED",
  comments_checked: true/false,
  comments_result: "N resolved" | "0 unresolved" | null,
  ci_checked: true/false,
  ci_result: "all passing" | "N failing — delegated to pr-ci" | "in progress" | null,
  ci_skipped_reason: "in progress" | "review required" | null,
  actions_taken: ["pr-comments", "pr-ci", ...]
}
```

Only populate fields for dimensions that were checked. If `comments_checked` is false, `comments_result` MUST be null.

### [8] Update state

Determine whether this check was idle (no skills invoked, no merges detected, no actions taken):

```
TaskUpdate(<task-id>, metadata: {
  prs: <updated array with merged flags>,
  last_check: "<ISO 8601>",
  iteration_count: <incremented>,
  idle_checks: <previous + 1 if idle, else 0>
})
```

### [9] Completion check and reporting

If **all** PRs have `merged: true`:

1. Find and delete any cron that invokes babysit:
   ```
   CronList → find cron where prompt contains "/babysit" → CronDelete(<id>)
   ```
2. Mark task completed:
   ```
   TaskUpdate(<task-id>, status: "completed")
   ```
3. Report: "All PRs merged. Babysit complete."

**Blocked-on-humans escalation:** If `idle_checks >= 3` and the only blocker is reviewer approval (CI green or in-progress, no comments to fix):

```
Babysit: stack blocked on human review.
  All CI green. No unresolved comments. Awaiting reviewer approval on N PRs.
  This is the 3rd consecutive idle check — nothing actionable for babysit.
```

Otherwise, format the status report:

```
Babysit: 1/3 merged.
  #123 (branch-a): merged
  #124 (branch-b): CI fixed + pushed, comments addressed (2 resolved)
  #125 (branch-c): CI in progress (skipped), comments checked (0 unresolved)
```

---

## Stopping babysit

| Trigger | How |
|---------|-----|
| All merged | Automatic — finds and deletes cron, completes task |
| User cancels | Delete the cron (via `/loop` ID) or mark task completed |
| Session ends | Cron dies with session |
| 7-day expiry | Cron auto-expires |

To see active babysit tasks: `TaskList` and look for `type: "babysit"`.
