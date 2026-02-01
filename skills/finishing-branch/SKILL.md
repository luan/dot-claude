---
name: finishing-branch
description: Use when implementation complete and ready to integrate. Presents 4 structured options - merge, PR, keep, discard.
user-invocable: true
---

# Finishing a Development Branch

Verify tests → Present options → Execute choice → Clean up.

## Process

### Step 1: Verify Tests

**Before presenting options:**

```bash
# Run project's test suite
npm test / cargo test / pytest / go test ./...
```

**If tests fail:** Report failures, stop. Cannot proceed until tests pass.

**If tests pass:** Continue to Step 2.

### Step 2: Present Options

Present exactly these 4 options via AskUserQuestion:

```
Implementation complete. What would you like to do?

1. Merge locally - merge to {base-branch}, delete feature branch
2. Create PR - push and open pull request
3. Keep as-is - I'll handle it later
4. Discard - delete this work (requires confirmation)
```

### Step 3: Execute Choice

#### Option 1: Merge Locally

```bash
git checkout {base-branch}
git pull
git merge {feature-branch}
# Verify tests on merged result
git branch -d {feature-branch}
```

#### Option 2: Create PR

```bash
git push -u origin {feature-branch}
gh pr create --title "{title}" --body "$(cat <<'EOF'
## Summary
{2-3 bullets}

## Test Plan
- [ ] {verification steps}
EOF
)"
```

Report PR URL.

#### Option 3: Keep As-Is

Report: "Keeping branch {name}. Ready when you are."

No cleanup.

#### Option 4: Discard

**Confirm first:** "This will permanently delete branch {name} and all commits. Type 'discard' to confirm."

Wait for exact confirmation.

```bash
git checkout {base-branch}
git branch -D {feature-branch}
```

## Quick Reference

| Option | Merge | Push | Keep Branch |
|--------|-------|------|-------------|
| 1. Merge locally | ✓ | - | Delete |
| 2. Create PR | - | ✓ | Keep |
| 3. Keep as-is | - | - | Keep |
| 4. Discard | - | - | Force delete |

## Key Behaviors

- Verify tests BEFORE offering options
- Present exactly 4 options (no variations)
- Option 4 requires typed confirmation
- Use AskUserQuestion for option selection

## Integration

Called by:
- **implement** - after all tasks complete
- **subagent-driven-development** - after final review
- **review-and-fix** - after all fixes applied

Uses:
- **git-surgeon** - for selective staging if needed before merge
- **commit** - for final commit messages
