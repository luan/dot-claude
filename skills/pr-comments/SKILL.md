---
name: pr-comments
description: "Fix unresolved PR review comments. Triggers: 'fix comments', 'fix PR comments', 'address review feedback'."
user-invocable: true
disable-model-invocation: false
allowed-tools:
  - "Bash(scripts/fetch_threads.py *)"
  - "Bash(gh pr view *)"
  - "Bash(gh pr list *)"
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

## Context

PR: !`gh pr view --json number,headRefName -q '{number,headRefName}' 2>/dev/null`
Repo: !`gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null`
Branch: !`git branch --show-current 2>/dev/null`

## Steps

1. **Detect PR**: Use injected context above. If empty, ask user.

2. **Verify branch**: Compare injected Branch vs PR headRefName — mismatch → ask user and **stop**. Do not proceed to Step 3 until the user confirms or switches branches.

3. **Fetch comments** (execute directly — never prefix with `python3`/`uv run`):

   ```bash
   scripts/fetch_threads.py --pr <PR> --repo <Repo>
   ```

   Display as numbered list with file:line, author, preview. Ask "Which comment(s) to fix?" — options: "Fix all" / "Other"

4. **Plan fixes**: For each comment, read code, create one-line fix description. Ask "Ready to execute?"

5. **Execute**: Apply fixes, summarize changes.

6. **Run tests**: Detect and run the project test suite (look for `test` script in package.json, pytest, cargo test, etc.). Fix failures caused by your changes. If 3+ failures persist after fixes, report and stop.

7. **Commit**: Use `Skill(commit)` to generate message and commit.

8. **Push** (optional): Ask first. Use `Skill(gt:submit)` if gt plugin is loaded, otherwise `git push`.
