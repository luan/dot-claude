# Context Budget

Context window is finite. Treat like memory — never waste.

## Rules

- Verbose commands: pipe `| tail -20` or `| head -50`. Never dump full logs.
- Use `--quiet`, `--summary`, `-s` flags when available
- Grep relevant lines instead of reading full output
- Passing info between agents: pre-compute summary, don't forward raw output
- Spawning subagents: focused context only. Include what needed, omit rest.
- Output exceeds ~30 lines → summarize before continuing
