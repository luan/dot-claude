# Comment Quality Standards

## The Gate

Every comment must tell reader something they **cannot get from code itself**.

## Banned

- Restating code: `counter += 1  // increment counter`
- Filler docstrings: `@param name the name`, `@return the result`
- Section dividers: `// ---- helpers ----`
- Changelog in comments: `// v2: added validation - Jan 2025`
- TODOs without context: `// TODO: fix this`
- Comments on every line or every function

## Allowed

- **WHY** explanations: why non-obvious approach chosen
- Edge case warnings: gotchas future maintainers need
- Business logic context: domain rules not evident from code
- Non-obvious constraints: performance implications, ordering requirements
- Surprising behavior: anything making reader double-take

## Rules

- Never generate docstrings unless project already uses them as convention
- Never add comments to code you didn't write (unless fixing bug there)
- If you need comment to explain WHAT code does, code needs renaming or splitting — not comment
- Three clear function names > one function with three section comments
