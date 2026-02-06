---
name: refine
description: "Pre-commit code polish. Simplifies complexity, removes low-value comments. Does NOT change behavior."
argument-hint: "[optional: file-pattern]"
user-invocable: true
allowed-tools:
  - "Bash(git diff:*)"
  - "Bash(git status:*)"
  - Read
  - Edit
  - Glob
  - Grep
---

# Refine

Simplify code and remove comment bloat in uncommitted changes before committing.

## Step 1: Identify Files

- If $ARGUMENTS: use as file pattern (e.g. `*.py`, `src/**/*.ts`)
- Otherwise: `git diff --name-only HEAD`
- Filter out: lock files, generated files, binaries, config
- No files → exit: "No uncommitted changes to refine."

## Step 2: Read All Files in Parallel

Read all identified files simultaneously for full context.

## Step 3: Apply Refinements

For each file, look for and fix:

### Simplify Complexity
- Flatten unnecessary nesting (early returns)
- Remove redundant defaults (`.get(key, None)` → `.get(key)`)
- Replace inline lambdas with direct expressions
- Extract magic numbers only if used multiple times
- Three similar lines > premature abstraction

### Remove Low-Value Comments
Remove:
- `// Create user object` above `user = new User()`
- `// Loop through items` above `for item in items`
- `// Return result` above `return result`
- `// TODO: fix this` without context
- Section dividers with no value (`// ---- helpers ----`)

Keep:
- Why a non-obvious approach was chosen
- Edge cases or gotchas
- Business logic / domain-specific behavior
- Performance implications or limitations

### Clean Up
- Remove unused imports added in this change
- Fix inconsistent formatting within changed code
- Remove debugging artifacts (console.log, print, etc.)

## Step 4: Verify After Each Edit

- Check syntax (run linter/parser if available)
- If broken: revert that edit, note the issue, continue

## Step 5: Summary

Show per-file:
- Simplifications applied
- Comments removed/improved
- Other cleanups

## Rules

- **Never change behavior** - only structural/cosmetic changes
- **Only touch uncommitted changes** - don't refine unrelated code
- **Preserve existing patterns** - match the codebase style
- **When in doubt, leave it** - conservative > aggressive

## Skill Composition

| When | Invoke |
|------|--------|
| After refine | `use Skill tool to invoke commit` |
| Found real issues | `use Skill tool to invoke review-and-fix` |
