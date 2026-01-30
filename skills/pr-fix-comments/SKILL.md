---
name: pr-fix-comments
description: Fetch unresolved PR comments and suggest code fixes for each one
user-invocable: true
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

Fetch unresolved review comments from a PR and fix them based on user direction.

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

If they don't match, ask the user how to proceed - they may have accidentally specified the wrong PR or be on the wrong branch.

## Step 1: Fetch and Display Comments

```bash
python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py fetch --pr <PR_NUMBER>
```

Display all unresolved comments as a numbered list:

```
## Unresolved Comments (N total)

1. `path/to/file.swift:42` - @author: "Brief preview of comment..."
2. `path/to/other.swift:17` - @author: "Brief preview of comment..."
...
```

Then use **AskUserQuestion** to ask how to proceed:

```
Question: "Which comment(s) would you like me to fix?"
Header: "Fix Comments"
Options:
  1. "Fix all" - "Fix all unresolved comments"
  2. "Other" - "Custom selection or guidance"
```

The "Other" option allows the user to:
- Select specific comments: "1 and 3"
- Provide guidance: "1, but add a code example"
- Give custom instructions: "2, and for that one I want ..."

## Step 2: Plan the Fixes

For each comment to fix:
1. Read the relevant code
2. Understand what the reviewer is asking for
3. Create a concise description of what you'll change

Create a **concise** fix plan:
- Briefly describe each fix (one line each)
- If something needs clarification, note the question
- Don't over-explain trivial fixes

Example plan output:
```
**Fix plan:**
- #1: Add clarifying comment
- #3: Rename variable for clarity
```

Then use **AskUserQuestion** to confirm:

```
Question: "Ready to execute these fixes?"
Header: "Fix Plan"
Options:
  1. "Execute" - "Apply all planned fixes"
  2. "Other" - "Custom prompt"
```

## Step 3: Confirm and Execute

**Always ask the user to confirm the plan** before making changes using the AskUserQuestion format shown above.

**If anything is unclear to you**: Ask for clarification before showing the plan.

After user confirms and fixes are applied, briefly summarize what was done.

## Step 4: Commit (No Push)

**⚠️ IMPORTANT: NEVER post thread replies without showing them to the user first and getting explicit confirmation.**

After fixes are applied, generate a commit message following this format:

### Commit Message Format

**Default (no special guidance):**
```
pr-fix-comments
```

**With user guidance or notable changes:**
```
pr-fix-comments: <brief summary of what was done>
```

**Examples:**
- `pr-fix-comments` - simple fixes, exactly what reviewer asked
- `pr-fix-comments: add error handling per review` - user gave specific direction
- `pr-fix-comments: rename vars for clarity` - describes the type of fix
- `pr-fix-comments: add docstrings to public API` - describes scope

**Rules:**
- Always use `pr-fix-comments` prefix
- Keep summary brief (under 50 chars total)
- Summarize user guidance, don't quote verbatim
- Do NOT use conventional commit prefixes (no `fix:`, `chore:`, etc.)

Then show the suggested message and ask:

```
Suggested commit: pr-fix-comments: <your summary here>
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
2. Commit with the message (default or custom) - do NOT add "Co-Authored-By: Claude" tags
3. Show the planned thread replies to the user:
   ```
   Thread replies:
   - #1: "Done [pr-fix-comments]"
   - #3: "Added example for edge case [pr-fix-comments]"
   ```

   Then use **AskUserQuestion**:
   ```
   Question: "Post these replies and resolve the threads? (You'll need to push manually first)"
   Header: "Thread Replies"
   Options:
     1. "Post & resolve" - "Post all replies and mark threads as resolved"
     2. "Skip" - "Don't post replies (do it manually)"
     3. "Other" - "Custom prompt"
   ```

   Reply format:
   - If the fix is exactly what the reviewer asked for: "Done [pr-fix-comments]"
   - If the fix was done in a particular way (especially with user guidance): "{brief explanation of how it was fixed} [pr-fix-comments]"

4. If user agrees, post replies and resolve:
   ```bash
   python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py reply --pr <PR> --comment-id <ID> --body "<message>"
   python3 ~/.claude/skills/pr-fix-comments/scripts/pr_threads.py resolve --thread-id <THREAD_ID>
   ```

## Step 5: Push/Sync (Optional)

After committing, ask the user if they want to push:

```
Question: "Push changes to update the PR?"
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

## Notes

- This is NOT a fully automated workflow - the user confirms the plan before execution
- **Push is optional** - always ask before pushing, detect stack tools first
- **This skill NEVER rebases or performs destructive git operations**
- Be concise in planning - don't describe trivial changes in detail
- If user provided guidance for a comment, use it
- Follow the user's existing workflow when making fixes
- Always use `gt ss` for pushing
