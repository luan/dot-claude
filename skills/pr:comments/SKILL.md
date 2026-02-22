---
name: pr:comments
description: "Fix unresolved PR review comments. Triggers: 'fix comments', 'fix PR comments', 'address review feedback'."
user-invocable: true
disable-model-invocation: false
allowed-tools:
  - "Bash(scripts/fetch_threads.py *)"
  - "Bash(gh pr view *)"
  - "Bash(gh pr list *)"
  - "Bash(git branch --show-current)"
  - "Bash(git add *)"
  - "Bash(git push:*)"
  - Skill
  - Read
  - Edit
  - Glob
  - Grep
---

# PR Comments Fixer

Fix unresolved review comments from a PR.

**Safety: NEVER auto-pushes. NEVER replies to or resolves threads — only fetches and fixes locally.**

## Steps

1. **Detect PR**: `gh pr view --json number -q '.number'` or ask user

2. **Verify branch** (if PR specified manually):
   `git branch --show-current` vs `gh pr view <PR> --json headRefName -q .headRefName` — mismatch → ask user

3. **Fetch comments**:

   First verify `scripts/fetch_threads.py` exists (check repo root and `~/.claude/scripts/`). If missing, fall back to `gh api repos/{owner}/{repo}/pulls/{PR}/comments`.

   ```bash
   scripts/fetch_threads.py --pr <PR>
   ```

   Display as numbered list with file:line, author, preview. Ask "Which comment(s) to fix?" — options: "Fix all" / "Other"

4. **Plan fixes**: For each comment, read code, create one-line fix description. Ask "Ready to execute?"

5. **Execute**: Apply fixes, summarize changes.

6. **Run tests**: Detect and run the project test suite (look for `test` script in package.json, pytest, cargo test, etc.). Fix failures caused by your changes. If 3+ failures persist after fixes, report and stop.

7. **Commit**: Use `Skill(commit)` to generate message and commit.

8. **Push** (optional): Ask first. Use `Skill(gt:submit)` if gt plugin is loaded, otherwise `git push`.
