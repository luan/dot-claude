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
  - "Bash(gt *)"
  - Read
  - Edit
  - Glob
  - Grep
---

# PR Comments Fixer

Fix unresolved review comments from a PR.

**Safety: NEVER auto-pushes. Push optional + requires confirmation.**

## Steps

1. **Detect PR**: `gh pr view --json number -q '.number'` or ask user

2. **Verify branch** (if PR specified manually):
   - `git branch --show-current` vs `gh pr view <PR> --json headRefName -q .headRefName`
   - Mismatch → ask user

3. **Fetch and display comments**:

   ```bash
   scripts/fetch_threads.py --pr <PR>
   ```

   Display as numbered list with file:line, author, preview. Ask "Which comment(s) to fix?" with options: "Fix all" / "Other"

4. **Plan fixes**: For each comment, read code, create one-line fix description. Ask "Ready to execute?"

5. **Execute**: Apply fixes, summarize changes.

6. **Run tests**: Run the project test suite (or relevant subset if large). Fix any failures caused by your changes. If 3+ test failures persist after fixes, report and stop.

7. **Commit**: Use `Skill tool: commit` to generate message and commit.

8. **Push** (optional): Ask first, then use `/gt:submit`

## Notes

- User confirms plan before execution
- NEVER reply to or resolve threads — only fetch and fix locally
- Be concise — don't over-explain trivial fixes
