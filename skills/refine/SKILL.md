---
name: refine
description: "Use before committing to polish code. Triggers: 'refine', 'clean up', 'simplify'. Removes low-value comments, simplifies complexity, cleans imports. Does NOT change behavior."
argument-hint: "[optional: file-pattern]"
user-invocable: true
allowed-tools:
  - "Bash(git diff:*)"
  - "Bash(git status:*)"
  - Read
  - Edit
  - Glob
  - Grep
  - Skill
  - AskUserQuestion
---

# Refine

Simplify code + remove comment bloat in uncommitted changes.

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Simplification might change semantics → confirm intent before applying
- Uncertain if removing code/comment changes behavior → ask

Do NOT ask when the answer is obvious or covered by the task brief.

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
Remove: code-restating inline comments (`// Create user object` above `user = new User()`), contextless TODOs, valueless section dividers

Keep:
- WHY explanations, edge case warnings, business logic context, performance implications
- **Doc comments by default** — JSDoc, Python docstrings, Rust `///`, Go doc comments are API docs. Preserve them unless genuinely vacuous (`@param name the name`, `@return the result`). Removing doc comment is deliberate per-case judgment, not blanket rule.

### Clean Up
- Remove unused imports from this change
- Fix inconsistent formatting in changed code
- Remove debug artifacts (console.log, print, etc.)

## Step 4: Verify Each Edit

- Run linter/parser if available
- If broken → revert, note issue, continue

## Step 5: Summary

Per-file: simplifications applied, comments removed, other cleanups.

## Step 6: Continuation Prompt

Use AskUserQuestion:
- "Continue to /commit" (Recommended) — description: "Create conventional commit from changes"
- "Review changes first" — description: "Inspect the polished diff before committing"
- "Done for now" — description: "Leave bead in_progress for later /resume-work"

If user selects "Continue to /commit":
→ Invoke Skill tool: skill="commit", args=""

## Rules

- **Never change behavior** — structural/cosmetic only
- **Only touch uncommitted changes**
- **Preserve existing patterns**
- **When in doubt, leave it**

## Skill Composition

| When | Invoke |
|------|--------|
| Before refine | Run review until clean |
| After refine | `use Skill tool to invoke commit` |
