---
name: pr-fix-comments
description: "Fix unresolved PR review comments. Triggers: 'fix comments', 'address review', 'resolve threads', 'PR feedback'."
user-invocable: true
disable-model-invocation: true
allowed-tools:
  - "Bash(python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py:*)"
  - "Bash(gh pr view:*)"
  - "Bash(gh pr list:*)"
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

# PR Comments Fixer

Fix unresolved PR review comments.

**Safety: NEVER rebases. Push optional + requires confirmation.**

## Steps

1. **Detect PR**: `gh pr view --json number -q '.number'` or ask user
2. **Verify branch** (if PR specified manually): confirm current branch matches PR's branch
3. **Fetch comments**:

   ```bash
   python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py fetch --pr <PR>
   ```

   Display as numbered list, ask "Which to fix?" (Fix all / Other)

4. **Plan fixes**: Read code, understand request, create concise plan (one line each)
   Ask "Ready to execute?"

5. **Execute**: Apply fixes, summarize

6. **Commit**:
   - Message: `pr-fix-comments` or `pr-fix-comments: <brief summary>`
   - Ask "Ready to commit?"
   - Stage + commit (no Co-Authored-By tags)

7. **Thread replies** (show first, ask confirmation):
   - Simple fix: "Done [pr-fix-comments]"
   - With explanation: "{how it was fixed} [pr-fix-comments]"

   ```bash
   python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py reply --pr <PR> --comment-id <ID> --body "<msg>"
   python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py resolve --thread-id <ID>
   ```

8. **Push** (optional): Ask first, then `gt ss --update-only`
