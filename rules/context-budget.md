# Context Budget

Context window is finite. Treat it like memory — never waste it.

## Rules

- Verbose commands: pipe through `| tail -20` or `| head -50`. Never dump full logs.
- Use `--quiet`, `--summary`, `-s` flags when available
- Grep for relevant lines instead of reading full output
- When passing info between agents: pre-compute a summary, don't forward raw output
- Spawning subagents: focused context only. Include what they need, omit everything else.
- If output exceeds ~30 lines, summarize before continuing
