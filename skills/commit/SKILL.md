---
name: commit
description: Create conventional commit messages by analyzing changes. Use for regular commits, amending, rebasing, squashing, or ANY commit operation. Explains WHY not just WHAT.
user-invocable: true
allowed-tools:
  - "Bash(git status)"
  - "Bash(git diff:*)"
  - "Bash(git log:*)"
  - "Bash(git add:*)"
  - "Bash(git commit:*)"
  - "Bash(git branch:*)"
  - Read
  - Glob
  - Grep
---

# Committer

Create concise, meaningful conventional commit messages that explain WHY changes were made.

## Step 1: Analyze Changes (Run in Parallel)

```bash
git status
git diff --cached  # Or git diff if nothing staged
git log --oneline -5  # For context
```

## Step 2: Create Message

### Type Selection

| Type         | Use When                | Example                                        |
| ------------ | ----------------------- | ---------------------------------------------- |
| **feat**     | New user-facing feature | `feat(auth): add OAuth2 login`                 |
| **fix**      | Bug fix                 | `fix(payment): validate expiry before charge`  |
| **refactor** | Code restructuring      | `refactor(api): extract validation middleware` |
| **perf**     | Performance improvement | `perf(search): add query caching`              |
| **docs**     | Documentation only      | `docs(readme): add setup instructions`         |
| **test**     | Test changes            | `test(auth): add OAuth integration tests`      |
| **style**    | Formatting only         | `style: apply prettier formatting`             |
| **build**    | Build/dependencies      | `build(deps): upgrade to React 18`             |
| **ci**       | CI/CD changes           | `ci: add staging deployment workflow`          |
| **chore**    | Maintenance             | `chore: update gitignore`                      |
| **revert**   | Reverting commits       | `revert: "feat(auth): add OAuth2 login"`       |

### Format Rules

```
type(scope): description
```

- **Max 72 characters**
- **Lowercase**
- **No period at end**
- **Imperative mood** ("add" not "added")

### Scope Guidelines

- Use the primary area affected: `auth`, `api`, `ui`, `db`, etc.
- Omit scope if change is truly global
- For multiple areas, use most significant or omit

## Step 3: Confirm with User

Show the proposed commit message and use **AskUserQuestion**:

```
Proposed commit:
  feat(auth): add session timeout handling

Changes:
  - src/auth/session.ts (new)
  - src/auth/middleware.ts (modified)
```

```
Question: "Commit with this message?"
Header: "Commit"
Options:
  1. "Commit" - "Use this message"
  2. "Edit" - "Modify the message"
  3. "Other" - "Custom input"
```

## Step 4: Execute Commit

```bash
git commit -m "$(cat <<'EOF'
type(scope): description
EOF
)"
```

### Handle Pre-commit Hooks

If hooks modify files (formatting, linting):

```bash
git add -u && git commit --amend --no-edit
```

If hooks fail:

1. Show the error clearly
2. Suggest fix if obvious
3. Let user decide how to proceed

## Special Operations

### Amending (`--amend`)

1. Analyze both previous commit AND new changes
2. Decide if message needs updating
3. Use `--amend` with new message or `--amend --no-edit`

```bash
git commit --amend -m "$(cat <<'EOF'
type(scope): updated description
EOF
)"
```

### Squashing / Rebasing

When consolidating multiple commits:

1. List the commits being combined
2. Identify the primary purpose
3. Create unified message preserving the most important "why"
4. Combine scopes if multiple areas affected

### Fixup Commits

For commits that fix a previous commit:

```bash
git commit --fixup=<SHA>
```

## Quality Checklist

**✅ DO:**

- Explain the business/technical WHY
- Keep commits atomic and focused
- Use imperative mood
- Group related changes

**❌ DON'T:**

- Describe WHAT (code shows that)
- Include debug statements
- Mix unrelated changes
- Write long descriptions in subject line

## Edge Cases

### Nothing to Commit

```
No changes to commit. Working tree is clean.
```

### Multiple Unrelated Changes

```
These changes appear to cover multiple concerns:
- Auth changes in src/auth/
- UI changes in src/components/

Recommend splitting into separate commits. Would you like to:
1. Commit auth changes first
2. Commit all together anyway
```

### Unstaged Changes

If nothing is staged but there are changes:

```
Question: "Stage all changes before committing?"
Header: "Stage"
Options:
  1. "Stage all" - "git add -A"
  2. "Stage tracked" - "git add -u"
  3. "Other" - "Select specific files"
```

## Examples

**Good commits:**

```
feat(search): add fuzzy matching for better UX
fix(cart): prevent duplicate items on rapid clicks
refactor(api): consolidate error handling middleware
perf(images): lazy load below-fold images
docs(api): document rate limiting behavior
```

**Bad commits:**

```
updated files              # Too vague
fix bug                    # What bug?
WIP                        # Not descriptive
changes to auth module     # Describes what, not why
Fixed the thing.           # Period, past tense, vague
```

## Notes

- The code diff shows WHAT changed; your message explains WHY
- Future developers need context, not a changelog
- When in doubt, be more specific about the reason for the change
- Small commits with clear messages > large commits with vague messages
