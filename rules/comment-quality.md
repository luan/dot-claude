# Comment Quality Standards

## The Gate

Every comment must tell the reader something they **cannot get from the code itself**.

## Banned

- Restating code: `counter += 1  // increment counter`
- Filler docstrings: `@param name the name`, `@return the result`
- Section dividers: `// ---- helpers ----`
- Changelog in comments: `// v2: added validation - Jan 2025`
- TODOs without context: `// TODO: fix this`
- Comments on every line or every function

## Allowed

- **WHY** explanations: why a non-obvious approach was chosen
- Edge case warnings: gotchas future maintainers need to know
- Business logic context: domain rules not evident from code
- Non-obvious constraints: performance implications, ordering requirements
- Surprising behavior: anything that would make a reader do a double-take

## Rules

- Never generate docstrings unless the project already uses them as convention
- Never add comments to code you didn't write (unless fixing a bug there)
- If you need a comment to explain WHAT code does, the code needs renaming or splitting — not a comment
- Three clear function names > one function with three section comments
