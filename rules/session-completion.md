# Session Completion ("Landing the Plane")

When ending work session, ALL steps mandatory:

1. File remaining work as beads issues
2. Run quality gates (tests, linters, build) if code changed
3. Update beads status — close finished, update in_progress
4. Sync beads: `bd sync`
5. Commit code changes if uncommitted (orchestrator only — workers never commit)
6. Note stale branches for cleanup (optional) — list local luan/* branches without matching in_progress beads as cleanup suggestion

## Hard Rules

- NEVER say "ready to commit when you are" — YOU commit
- Push only when user explicitly requests
