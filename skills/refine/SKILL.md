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

Simplify code + remove comment bloat in uncommitted changes.

## Step 1: Identify Files

- $ARGUMENTS → use as file pattern (`*.py`, `src/**/*.ts`)
- No args → `git diff --name-only HEAD`
- Skip: lock files, generated, binaries, config
- No files → exit: "No uncommitted changes to refine."

## Step 2: Read All Files in Parallel

## Step 3: Apply Refinements

### Simplify Complexity
- Flatten nesting (early returns)
- Remove redundant defaults (`.get(key, None)` → `.get(key)`)
- Replace inline lambdas with direct expressions
- Extract magic numbers only if used multiple times
- Three similar lines > premature abstraction

### Remove Low-Value Comments
Remove: code-restating (`// Create user object` above `user = new User()`), contextless TODOs, valueless section dividers

Keep: WHY explanations, edge case warnings, business logic context, performance implications

### Clean Up
- Remove unused imports from this change
- Fix inconsistent formatting in changed code
- Remove debug artifacts (console.log, print, etc.)

## Step 4: Verify Each Edit

- Run linter/parser if available
- If broken → revert, note issue, continue

## Step 5: Summary

Per-file: simplifications applied, comments removed, other cleanups.

## Rules

- **Never change behavior** — structural/cosmetic only
- **Only touch uncommitted changes**
- **Preserve existing patterns**
- **When in doubt, leave it**

## Skill Composition

| When | Invoke |
|------|--------|
| After refine | `use Skill tool to invoke commit` |
| Found real issues | `use Skill tool to invoke review` |
