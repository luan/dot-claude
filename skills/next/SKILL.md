---
name: next
description: "Context-aware dispatch: resumes branch work if on a feature branch, otherwise picks the next work item from the board. Triggers: 'next', 'what next', 'pick up next task', 'resume', 'where was I', 'continue working'."
argument-hint: "[branch-name|PR#]"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - AskUserQuestion
  - TaskList
  - TaskGet
---

# Next

Figure out where you are, then either resume in-flight work or pick
something new.

## Context

Branch: !`git branch --show-current`
Status: !`git status -sb 2>/dev/null`
Recent commits: !`git log --oneline -5 2>/dev/null`

## Step 0: Detect trunk

```bash
TRUNK=$(gt parent 2>/dev/null || gt trunk)
CURRENT=$(git branch --show-current)
```

If `$CURRENT` equals `$TRUNK`, or the user passed an explicit
branch/PR# argument, go to the appropriate path:

- **On trunk, no argument** → Jump to [Trunk Path](#trunk-path)
- **On feature branch** → Jump to [Feature Branch Path](#feature-branch-path)
- **Explicit argument (branch name or PR#)** → Resolve the branch
  first (PR# → `gh pr view <num> --json headRefName`), then follow
  Feature Branch Path for that branch

---

## Feature Branch Path

Resume in-flight work. Gather state, suggest next action.

### 1. Gather branch state

Already have branch name, status, and recent commits from Context.

### 2. Gather PR context (if PR exists)

- `gh pr view --json title,state,isDraft,reviewDecision,statusCheckRollup`
- `gh pr checks`
- PR review comments: `gh api repos/{owner}/{repo}/pulls/{num}/comments`

### 3. Check for stale branches

- List all local `luan/*` branches: `git branch --list 'luan/*' --format='%(refname:short)'`
- Get active tasks: `TaskList()` filtered by status === "in_progress" and metadata.project === repoRoot
- Check `metadata.branch` field on each task
- Cross-reference: branches not referenced in any in_progress task's `metadata.branch` are potentially stale
- Never auto-delete — only surface for user awareness

### 4. Summarize

```
Branch: <name>
Commits: last 3 messages
PR: #N (draft/ready) — title
Review: Approved | Changes requested | Pending
CI: Passing | Failing (list failures)
Comments: N unresolved (summarize)
Tasks: N active, M pending
Stale branches: <list> (no matching active issues)
```

Only show "Stale branches" line if stale branches exist.

### 5. Suggest next action (priority order)

1. CI failing → "Fix checks"
2. Changes requested → "Address N review comments"
3. Unresolved comments → "Respond to feedback"
4. Tasks active → "Continue: ..."
5. Draft PR, all passing → "Mark ready"
6. Ready PR, approved → "Merge"
7. No PR → "Create with /commit then /graphite"
8. All clear → "Wait for review"

---

## Trunk Path

Read the board, find what's ready, dispatch the right skill.

### 1. Read the board

```
TaskList() filtered by metadata.project === repoRoot
Split into: pending (open) and in_progress without status_detail (active)
```

Sort by metadata.priority (lowest number first), then creation order.

### 2. Pick the top candidate

Select the highest-priority item that is NOT blocked. Skip:
- Items with status `completed` or `deleted`
- Items with status_detail === 'review' (waiting on human)
- Items whose description says "blocked by" an open issue

If an item has status `in_progress`, prefer it over `pending` items at the
same priority (someone already started it — resume it).

### 3. Read the candidate

```
TaskGet(taskId)
```

### 4. Classify the action

Read the issue description and classify:

| Signal | Action |
|--------|--------|
| Type is `bug` | `/debugging` |
| Title starts with "Brainstorm:" or description says "Needs brainstorm" | `/brainstorm` |
| Title starts with "Explore:" or description says "Needs explore" or "Explore first" | `/explore` |
| Has design/plan but no children yet (TaskList shows no tasks with metadata.parent_id === taskId) | `/prepare` |
| Has children (TaskList shows tasks with metadata.parent_id === taskId), or is a leaf task ready to build | `/implement` |

**Default:** If no signal matches → `/explore` (cheapest to course-correct)

**Tie-breaking:**
- Feature with no "## Approach" or "## Design" section → `/explore`
- Feature with approach but no concrete phases → `/brainstorm`

### 5. Present to user

Use AskUserQuestion with:
- Recommended action as first option (with "(Recommended)")
- 1-2 alternatives that could also apply
- Brief explanation of why this issue and this action

Format the question header as the issue ID.

Example:
```
Question: "Next up: <title> (P<n>). What should we do?"
Options:
  - "/explore (Recommended)" — "Investigate before designing"
  - "/brainstorm" — "Jump to design dialogue"
  - "Skip — show me the next one" — "Pick a different issue"
```

### 6. Dispatch

Based on user's choice, invoke the selected skill:

```
Skill tool: skill="<chosen-skill>", args="<issue-id>"
```

If user chose "Skip", go back to step 2 with the next candidate.

## Key Rules

- Never dispatch without showing the user what you picked and why
- Never skip reading the issue — the description drives classification
- If TaskList returns no matching tasks actionable, say so plainly
- Don't create issues — this skill discovers, it doesn't plan
