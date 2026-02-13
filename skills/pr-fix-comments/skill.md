---
name: pr-fix-comments
description: "Fix unresolved PR review comments. Triggers: 'fix comments', 'fix PR comments', 'address review feedback'."
user-invocable: true
allowed-tools:
  - "Bash(python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py *)"
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
   python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py fetch --pr <PR>
   ```

   Display as numbered list with file:line, author, preview.
   Ask "Which comment(s) to fix?" with options: "Fix all" / "Other"

4. **Plan fixes**: For each comment, read code, create one-line fix description.
   Ask "Ready to execute?"

5. **Execute**: Apply fixes, summarize changes.

6. **Commit**: Use `Skill tool: commit` to generate message and commit.

7. **Push** (optional): Ask first, then `gt ss --update-only`

8. **Resolve threads**: Show planned replies, ask confirmation, then:

   ```bash
   python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py reply --pr <PR> --comment-id <ID> --body "<message>"
   python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py resolve --thread-id <THREAD_ID>
   ```

   Reply format:
   - Exact fix: "Done [pr-fix-comments]"
   - Custom fix: "{brief explanation} [pr-fix-comments]"

## Notes

- User confirms plan before execution
- NEVER post thread replies without showing user first
- Be concise — don't over-explain trivial fixes
