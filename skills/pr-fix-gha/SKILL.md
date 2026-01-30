---
name: pr-fix-gha
description: Fetch failed GitHub Actions checks from a PR and fix them
user-invocable: true
allowed-tools:
  - "Bash(gh pr view:*)"
  - "Bash(gh pr checks:*)"
  - "Bash(gh pr list:*)"
  - "Bash(gh run view:*)"
  - "Bash(gh run list:*)"
  - "Bash(gh repo view:*)"
  - "Bash(git branch --show-current)"
  - "Bash(git add:*)"
  - "Bash(git commit:*)"
  - "Bash(gt *)"
  - Read
  - Edit
  - Glob
  - Grep
---

# PR GHA Fixer

Fetch failed GitHub Actions checks from a PR and fix them based on the error logs.

**⚠️ SAFETY: This skill NEVER rebases or performs destructive git actions. Push is optional and always requires explicit user confirmation.**

## Usage

Try to auto-detect the PR from the current branch first:
```bash
gh pr view --json number,headRefName -q '.number'
```

If that fails (no PR for current branch), ask the user which PR they want to fix.

## Step 0: Verify Branch (if PR was specified manually)

If the user explicitly provided a PR number (not auto-detected), verify the current branch matches:
- Get current branch: `git branch --show-current`
- Get PR's branch: `gh pr view <PR_NUMBER> --json headRefName -q .headRefName`

If they don't match, ask the user how to proceed.

## Step 1: Fetch and Display Failed Checks

```bash
gh pr checks <PR_NUMBER> --json name,state,bucket,link
```

Display failed checks as a numbered list:

```
## Failed Checks (N total)

1. `build` - Failed
2. `test-unit` - Failed
3. `lint` - Failed
...
```

If no checks have failed, inform the user and end.

Then use **AskUserQuestion** to ask how to proceed:

```
Question: "Which check(s) would you like me to investigate and fix?"
Header: "Fix Checks"
Options:
  1. "Fix all" - "Investigate and fix all failed checks"
  2. "Other" - "Custom selection or guidance"
```

## Step 2: Fetch Logs for Failed Checks

For each selected failed check:

1. Get the run ID from the check details or list runs:
   ```bash
   gh run list --branch <BRANCH> --json databaseId,name,conclusion --limit 20
   ```

2. View the failed run logs:
   ```bash
   gh run view <RUN_ID> --log-failed
   ```

3. If logs are too long, focus on the error sections (look for "error:", "failed:", "FAILED", etc.)

## Step 3: Analyze and Plan Fixes

For each failed check:
1. Identify the root cause from the logs
2. Find the relevant code that needs fixing
3. Create a concise description of the fix

Create a **concise** fix plan:
- Briefly describe each fix (one line each)
- If something needs clarification, note the question
- Don't over-explain trivial fixes

Example plan output:
```
**Fix plan:**
- `build`: Missing import in src/foo.ts
- `test-unit`: Test assertion outdated after API change
- `lint`: Unused variable in src/bar.ts
```

Then use **AskUserQuestion** to confirm:

```
Question: "Ready to execute these fixes?"
Header: "Fix Plan"
Options:
  1. "Execute" - "Apply all planned fixes"
  2. "Other" - "Custom prompt"
```

## Step 4: Execute Fixes

**Always ask the user to confirm the plan** before making changes.

**If anything is unclear**: Ask for clarification before showing the plan.

After user confirms and fixes are applied, briefly summarize what was done.

## Step 5: Commit

After fixes are applied, show the suggested commit message and use **AskUserQuestion**:

```
Fixes applied.

Suggested commit message: "fix: resolve GHA failures" (or more specific if only one type of failure)
```

```
Question: "Ready to commit these changes?"
Header: "Commit"
Options:
  1. "Commit" - "Commit with suggested message"
  2. "Other" - "Custom prompt"
```

**If user agrees (or provides custom message)**:
1. Stage the changed files with `git add`
2. Commit with the message - do NOT add "Co-Authored-By: Claude" tags

## Step 6: Push to Trigger CI (Optional)

After committing, ask the user if they want to push to trigger new CI runs:

```
Question: "Push changes to trigger new CI runs?"
Header: "Push"
Options:
  1. "Push" - "Push changes to remote"
  2. "Skip" - "I'll push manually later"
  3. "Other" - "Custom prompt"
```

**If user wants to push:**

Use Graphite with `--update-only` (since PRs already exist):
```bash
gt ss --update-only
```

This pushes changes and updates existing PRs without creating new ones.

**⚠️ Always confirm before running.**

## Common GHA Failure Patterns

### Build Failures
- Missing imports/dependencies
- Type errors
- Syntax errors
- Missing files

### Test Failures
- Outdated assertions
- Missing test fixtures
- Flaky tests (timing issues)
- Environment differences

### Lint Failures
- Formatting issues
- Unused imports/variables
- Style violations

### Other
- Missing secrets/env vars (can't fix directly - inform user)
- Rate limits (can't fix directly - inform user)
- Infrastructure issues (can't fix directly - inform user)

## Notes

- This is NOT a fully automated workflow - the user confirms the plan before execution
- **Push is optional** - always ask before pushing, detect stack tools first
- **This skill NEVER rebases or performs destructive git operations**
- Be concise in planning - don't describe trivial changes in detail
- Some failures can't be fixed in code (secrets, infra) - inform the user
- If a failure seems flaky, suggest re-running the workflow instead
- Always use `gt ss` for pushing
