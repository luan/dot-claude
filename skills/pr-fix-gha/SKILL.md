---
name: pr-fix-gha
description: "Fix failing CI/GitHub Actions checks. Triggers: 'fix CI', 'fix GHA', 'build failing', 'tests failing in CI', 'checks red'."
user-invocable: true
disable-model-invocation: true
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

Fix failed GitHub Actions checks.

**Safety: NEVER rebases. Push optional + requires confirmation.**

## Steps

1. **Detect PR**: `gh pr view --json number -q '.number'` or ask user
2. **Verify branch** (if PR specified manually)

3. **Fetch failed checks**:

   ```bash
   gh pr checks <PR> --json name,state,bucket,link
   ```

   Display as numbered list, ask "Which to fix?"

4. **Fetch logs**:

   ```bash
   gh run list --branch <BRANCH> --json databaseId,name,conclusion --limit 20
   gh run view <RUN_ID> --log-failed
   ```

5. **Plan fixes**: Identify root cause, create concise plan
   Ask "Ready to execute?"

6. **Execute**: Apply fixes, summarize

7. **Commit**: Ask first, suggest message like `fix: resolve CI failures`

8. **Push** (optional): Ask first, then `gt ss --update-only`

## Common failures

- **Build**: missing imports, type errors, syntax
- **Test**: outdated assertions, missing fixtures, flaky tests
- **Lint**: formatting, unused imports/vars
- **Infra**: secrets, rate limits (can't fix - inform user)
