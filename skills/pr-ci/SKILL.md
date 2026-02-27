---
name: pr-ci
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
  - "Bash(git push:*)"
  - Skill
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

5. **Plan fixes**: Identify root cause, create concise plan. Ask "Ready to execute?"

6. **Execute**: Apply fixes, summarize changes.

7. **Commit**: Ask first, suggest message like `fix: resolve CI failures`

8. **Push** (optional): Ask first. Use `Skill(gt:submit)` if gt plugin is loaded, otherwise `git push`.

## Common Failures & Remediation

- **Build** — missing imports, type errors, syntax → read error output, fix source directly
- **Test** — outdated assertions, missing fixtures → update expectations or add missing test data
- **Lint** — formatting, unused imports/vars → run the project formatter, remove dead code
- **Infra** — secrets, rate limits, runner issues → can't fix locally; inform user to check repo/org settings
