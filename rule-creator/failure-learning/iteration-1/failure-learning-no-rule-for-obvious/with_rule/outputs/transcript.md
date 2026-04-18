## Prompt
I found a bug — someone used = instead of == in a conditional. It was a typo. Should we create a rule about this?

## Response
No, this does not qualify for a project rule.

A typo using `=` instead of `==` is a one-off bug with an obvious fix. It falls under "What does NOT qualify": one-off bugs with obvious fixes. Nothing codebase-specific or non-obvious.

Most linters already catch it (ESLint's `no-cond-assign`, Clippy warnings, compiler warnings). A rule would add noise without preventing a meaningful class of bugs.

If this keeps happening repeatedly, the better fix is to enable or configure a linter rule, not write a `.claude/rules` file.
