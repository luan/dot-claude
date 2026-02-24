# Search Tools

## Tool Hierarchy

Prefer higher in this list:

1. **Grep tool** — text search (ripgrep-backed, correct permissions)
2. **Glob tool** — file pattern matching
3. `rg` / `fd` in Bash — when you need shell-level search (piping, scripting)
4. `ck` — semantic code search (BeaconBay/ck, grep-compatible + AI embeddings)

## Banned in Bash

- `grep` → use Grep tool or `rg`
- `find` → use Glob tool or `fd`

A PreToolUse hook enforces this globally.
