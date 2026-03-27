---
name: babysit
description: "Watch a Graphite PR stack until all PRs merge, automatically fixing CI failures and review comments using configurable skills on a cron schedule. Triggers: 'babysit', 'babysit my PRs', 'babysit stack', 'watch my stack', 'keep PRs green', 'monitor until merged', 'shepherd PRs', 'keep fixing until merged', 'babysit-prs'. Use whenever the user wants ongoing, unattended PR stack maintenance — not for one-time fixes (use /pr-ci or /pr-comments directly)."
argument-hint: "[interval] [skills...]"
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
  - CronCreate
  - CronDelete
  - CronList
---

# Babysit

Autonomous PR stack shepherd. Sets up a recurring cron that monitors every PR in a Graphite stack, running specified skills to fix CI failures and review comments until all PRs merge.

Relentless by design — the cron never gives up. If a fix doesn't stick, it tries again next interval. If CI fails the same way twice, it tries a different approach. Comments are checked on every open PR regardless of approval status — addressing review feedback is how PRs get approved. CI fixing is skipped for unapproved PRs (code may change after review). Babysit stops only when all PRs merge, the 7-day cron auto-expires, or the user cancels.

## Arguments

`[interval] [skills...]`

| Arg | Pattern | Default | Example |
|-----|---------|---------|---------|
| interval | `\d+[smhd]` (first token only) | `10m` | `5m`, `1h` |
| skills | remaining tokens | `pr-ci pr-comments` | `pr-ci pr-comments review` |

Internal: `--check <task-id>` — cron loop mode, not user-facing.

**Examples:**
- `/babysit` — 10m, pr-ci + pr-comments
- `/babysit 5m` — 5m, pr-ci + pr-comments
- `/babysit pr-ci` — 10m, CI only
- `/babysit 5m pr-ci pr-comments review` — 5m, three skills

---

## Setup mode (default)

### [1] Parse arguments

Split by whitespace. If the first token matches `^\d+[smhd]$`, extract as interval; remaining tokens are skill names. Otherwise all tokens are skills, interval defaults to `10m`.

Interval to cron expression:
- `Nm` (N <= 59) → `*/N * * * *`
- `Nm` (N >= 60) → `0 */H * * *` (H = ceil(N/60))
- `Nh` → `0 */N * * *`
- `Nd` → `0 0 */N * *`
- `Ns` → ceil(N/60) minutes, minimum `*/1 * * * *`

Avoid minute 0 and 30 for fixed-time crons. For `*/N` intervals, the built-in jitter handles load spreading — don't nudge the interval itself.

### [2] Discover the stack

```bash
gt log --stack 2>&1
```

Parse branch names in bottom-to-top order. For each branch:

```bash
gh pr list --head <branch> --json number,state --jq '.[0]'
```

Collect `{num, branch}`. Warn the user if any branch has no PR ("branch X has no PR — submit first?").

### [3] Create tracking task

```
TaskCreate(
  subject: "Babysit: #X, #Y, #Z",
  activeForm: "Babysitting",
  metadata: {
    type: "babysit",
    prs: [{num: N, branch: "name", merged: false}, ...],
    skills: ["pr-ci", "pr-comments"],
    interval: "10m",
    cron_id: null,
    work_dir: "<cwd>",
    idle_checks: 0
  }
)
TaskUpdate(<id>, status: "in_progress")
```

### [4] Create cron

```
CronCreate(
  cron: "<expression>",
  prompt: "/babysit --check <task-id>"
)
```

Store the returned ID:

```
TaskUpdate(<task-id>, metadata: {cron_id: "<returned-id>"})
```

### [5] Report to user

Tell them:
- PR count, numbers, and branches
- Skills that will run (all invoked with `--auto`)
- Check interval and cron job ID
- How to cancel: `CronDelete(<id>)`
- 7-day auto-expiry reminder

---

## Check mode (--check <task-id>)

Invoked by cron, not the user. Each invocation is self-contained: read state, act on current reality, update state.

### [1] Load state

```
TaskGet(<task-id>)
```

Extract `prs`, `skills`, `cron_id`, `work_dir`, `last_check`, `idle_checks`. If task is not `in_progress` → exit silently.

**Cron fork dedup:** If `last_check` exists and `now - last_check < interval`, another session already handled this check. Exit silently — don't duplicate work. This prevents parallel cron fork sessions from racing on the same task.

```bash
cd <work_dir>
```

### [2] Safety: uncommitted changes

```bash
git status --porcelain
```

If non-empty → "Babysit check skipped: uncommitted changes in worktree. Will retry next interval." → exit. This prevents the cron from interfering with the user's in-progress work.

Save the current branch for restoration later:
```bash
git branch --show-current
```

### [3] Detect newly merged parent PRs

When a parent PR in the stack merges, child branches go stale — their CI tests against old parent code, producing phantom failures. Detect this before doing any per-PR work.

Do NOT use `gt log --stack`'s `(needs restack)` flag for this — it fires on any trunk divergence (other teams merging to main), which would cause restacking on every interval in an active repo. Instead, check if any PR that was previously unmerged is now merged:

For each PR in `metadata.prs` where `merged` is false:

```bash
gh pr view <num> --json state --jq '.state'
```

If any returns `MERGED` — a parent just merged and child branches are stale:

1. Mark the newly merged PRs as `merged: true` in metadata.

2. **Sync with remote** — this cleans up merged branches and restacks:
   ```bash
   gt sync 2>&1
   ```
   `gt sync` fetches remote, removes merged branches, and restacks the stack onto trunk. This is the ONLY correct way to handle merged parents. **NEVER** use `gt track`, `gt delete`, or manual reparenting — Graphite handles this automatically.

3. If `gt sync` reports conflicts, resolve them:
   ```
   Skill("gt:restack")
   ```

4. Push the restacked branches:
   ```
   Skill("gt:submit")
   ```

5. Update task metadata with the merged flags.

6. **Exit early** — don't fix anything this interval. The restack changed code on child branches, so CI needs to re-run. Checking or fixing PRs now would be working against stale data. Report: "Parent PR(s) merged — restacked and pushed. Waiting for fresh CI."

If no PRs are newly merged → proceed to step [4]. Trunk having new commits is normal and doesn't invalidate CI.

### [4] Process each PR (bottom to top)

Process PRs from the bottom of the stack upward. Fixes on lower branches may resolve issues on upper branches after restacking — don't worry about cascade effects during a single pass. If a lower fix resolves an upper CI failure, the next interval will confirm it.

For each PR where `merged` is false:

```bash
gh pr view <num> --json state,reviewDecision --jq '{state,reviewDecision}'
```

**Routing — state only (not approval):**

| state | Action |
|-------|--------|
| MERGED | Mark `merged: true`, skip |
| CLOSED | Report "closed", skip |
| OPEN | Process (below) |

For each open PR, run comments and CI as **independent** concerns. Approval status gates CI fixing but NOT comment checking — review comments arrive before approval and must be addressed to unblock it.

**GitHub `reviewDecision` quirk:** An empty string `""` does NOT mean "not approved." GitHub resets `reviewDecision` to `""` when a bot reviewer (like `bcny-ai-agent`, `cursor[bot]`) posts a review after a human approved. Treat `""` the same as `"APPROVED"` — only `"REVIEW_REQUIRED"` or `"CHANGES_REQUESTED"` mean explicitly unapproved.

```bash
gt checkout <branch>
```

**a) Check comments** (if `pr-comments` or `pr-fix-comments` in skills)

Always run comment checking on every open PR, regardless of approval or CI status. Comments and CI are independent workstreams — unresolved comments block approval, and addressing them while CI runs is productive use of time.

```
Skill("pr-comments", "--auto")
```

The skill exits quickly if there are no unresolved comments — safe to invoke unconditionally.

**b) Check CI** (if `pr-ci` in skills)

```bash
gh pr checks <num> --json name,state,bucket --jq '[.[] | {name,state,bucket}]'
```

Skip CI fixing if:
- Any check is `IN_PROGRESS` or `QUEUED` — don't fix stale failures while a new run is pending
- `reviewDecision` is `"REVIEW_REQUIRED"` or `"CHANGES_REQUESTED"` — reviewer may request changes that invalidate current code, fixing CI before that is wasted effort

If checks have `bucket: "fail"` and CI fixing is not skipped:
```
Skill("pr-ci", "--auto")
```

Never classify CI failures yourself (e.g., "flaky", "intermittent"). Always delegate to `pr-ci` — it reads the actual logs and determines the cause. Your job is to detect fail/pass status, not diagnose failures.

**c) Other skills**

For each remaining skill in the list that isn't `pr-ci` or `pr-comments`:

```
Skill("<name>", "--auto")
```

All skills are invoked with `--auto` because cron runs are non-interactive. If a skill doesn't recognize `--auto`, it proceeds with its default behavior.

**Error handling:** If `gt checkout <branch>` fails (branch deleted, force-pushed, conflicts), skip this PR and report the error. Don't abort the entire check — continue to the next PR.

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

Before reporting, build a status object for each PR tracking exactly what happened. This prevents fabricating information that wasn't checked.

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

Only populate fields for dimensions that were actually checked. If `comments_checked` is false, `comments_result` MUST be null — never infer or guess.

### [8] Update state

Determine whether this check was idle (no skills invoked, no merges detected, no actions taken on any PR):

```
TaskUpdate(<task-id>, metadata: {
  prs: <updated array with merged flags>,
  last_check: "<ISO 8601>",
  idle_checks: <previous idle_checks + 1 if idle, else 0>
})
```

### [9] Completion check and reporting

If **all** PRs have `merged: true`:

```
CronDelete(<cron_id>)
TaskUpdate(<task-id>, status: "completed")
```

Report: "All PRs merged. Babysit complete."

**Blocked-on-humans escalation:** If `idle_checks >= 3` and the only blocker across all PRs is reviewer approval (CI is green or in-progress, no comments to fix), report this once:

```
Babysit: stack blocked on human review.
  All CI green. No unresolved comments. Awaiting reviewer approval on N PRs.
  This is the 3rd consecutive idle check — nothing actionable for babysit.
```

This surfaces the bottleneck to the user so they know babysit isn't broken — it's just waiting on humans.

Otherwise, format the status report from the structured status objects. Only include dimensions that were checked:

```
Babysit: 1/3 merged.
  #123 (branch-a): merged
  #124 (branch-b): CI fixed + pushed, comments addressed (2 resolved)
  #125 (branch-c): CI in progress (skipped), comments checked (0 unresolved)
Next check in 10m.
```

---

## Stopping babysit

| Trigger | How |
|---------|-----|
| All merged | Automatic — cron self-deletes |
| User cancels | `CronDelete(<id>)` |
| Session ends | Cron dies with session |
| 7-day expiry | Cron auto-expires |

To see active babysit tasks: `TaskList` and look for `type: "babysit"`.
