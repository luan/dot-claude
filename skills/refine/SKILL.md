---
name: refine
description: "Use before committing to polish code. Triggers: 'refine', 'clean up', 'simplify'. Removes low-value comments, simplifies complexity, applies language-idiomatic rewrites. Does NOT change behavior. Do NOT use when: a full adversarial review with bug-finding is needed — use /review instead."
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

Use AskUserQuestion when facing genuine ambiguity:
- Simplification might change semantics → confirm before applying
- Uncertain if removing code/comment changes behavior → ask

## Step 1: Identify Files

- $ARGUMENTS → use as file pattern (`*.py`, `src/**/*.ts`)
- No args → `git diff --name-only HEAD`
- Skip: lock files, generated, binaries, config
- No files → exit: "No uncommitted changes to refine."

## Step 2: Read All Files in Parallel

Read all identified files using parallel Read calls. Detect language from extension — apply only idioms appropriate to that language.

## Step 3: Apply Refinements

**Behavior boundary:** "Never change behavior" means same inputs → same outputs, same side effects, same error paths. Structural rewrites (early returns, constant extraction, idiom substitution) are in-scope when they preserve this contract.

### Simplify Complexity
- Flatten nesting via early returns
- Remove redundant defaults (`.get(key, None)` → `.get(key)`)
- Replace inline lambdas with direct expressions
- Extract magic numbers only when used multiple times — single-use literals are clearer inline
- Three similar lines > premature abstraction

### Language Idioms
Apply conventional patterns per language: guard clauses, comprehensions, defer blocks, optional chaining, context managers — whatever is idiomatic. Prefer standard library over manual implementations.

### Remove Low-Value Comments
Remove: code-restating inline comments, contextless TODOs, valueless section dividers

Keep: WHY explanations, edge case warnings, business logic context, performance implications

**Doc comments** (JSDoc, docstrings, Rust `///`, Go doc): preserve by default — they're API surface. Remove only when vacuous: the doc adds zero beyond what function name + types convey (e.g., `@param name the name`).

### Clean Up
- Remove unused imports from this change
- Fix inconsistent formatting in changed code
- Remove debug artifacts (console.log, print, etc.)

### Do NOT Over-Simplify
- Avoid clever one-liners that obscure intent
- Don't combine too many concerns into a single expression
- Don't remove helpful abstractions (named intermediates, well-named helpers)

## Step 4: Verify Each Edit

Run linter/parser if available. Broken → revert, note issue, continue.

## Step 5: Summary

Per-file: simplifications applied, idiom rewrites, comments removed, other cleanups.

## Rules

- **Never change behavior** — structural/cosmetic only
- **Only touch uncommitted changes**
- **Preserve existing patterns**
- **When in doubt, leave it**
