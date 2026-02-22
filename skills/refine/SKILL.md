---
name: refine
description: "Use before committing to polish code. Triggers: 'refine', 'clean up', 'simplify'. Removes low-value comments, simplifies complexity, applies language-idiomatic rewrites. Does NOT change behavior."
argument-hint: "[optional: file-pattern]"
user-invocable: true
allowed-tools:
  - "Bash(git diff:*)"
  - "Bash(git status:*)"
  - Read
  - Edit
  - Glob
  - Grep
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

Read all identified files using parallel Read tool calls.

## Step 3: Apply Refinements

### Simplify Complexity
- Flatten nesting (early returns)
- Remove redundant defaults (`.get(key, None)` → `.get(key)`)
- Replace inline lambdas with direct expressions
- Extract magic numbers only if used multiple times
- Three similar lines > premature abstraction

### Language Idioms
Detect language from file extensions before applying simplifications.

- Prefer standard library features over manual implementations (e.g., `Path.exists()` over `os.path.exists()`, array spread over manual concat)
- Apply conventional patterns per language: guard statements (Swift/Go/Kotlin), comprehensions (Python), defer blocks (Go/Swift), optional chaining (Swift/JS/TS), context managers (Python), early returns (all)
- Use idiomatic equivalents: prefer language-native constructs over generic ones

### Balance Against Over-Simplification
- Do NOT reduce clarity or maintainability in the name of brevity
- Avoid overly clever one-liners that obscure intent
- Do not combine too many concerns into a single function or expression
- Do not remove helpful abstractions (named intermediates, well-named helpers)
- Do not make code harder to debug or extend

### Remove Low-Value Comments
Remove: code-restating inline comments (`// Create user object` above `user = new User()`), contextless TODOs, valueless section dividers

Keep:
- WHY explanations, edge case warnings, business logic context, performance implications
- **Doc comments by default** — JSDoc, Python docstrings, Rust `///`, Go doc comments are API docs. Preserve them unless genuinely vacuous (`@param name the name`, `@return the result`). Removing a doc comment is deliberate per-case judgment, not a blanket rule.

### Clean Up
- Remove unused imports from this change
- Fix inconsistent formatting in changed code
- Remove debug artifacts (console.log, print, etc.)

## Step 4: Verify Each Edit

- Run linter/parser if available
- If broken → revert, note issue, continue

## Step 5: Summary

Per-file: simplifications applied, idiom rewrites, comments removed, other cleanups.

## Rules

- **Never change behavior** — structural/cosmetic only
- **Only touch uncommitted changes**
- **Preserve existing patterns**
- **When in doubt, leave it**
