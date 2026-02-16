# Session Completion ("Landing the Plane")

When ending work session, ALL steps mandatory:

1. File remaining work as work issues
2. Run quality gates (tests, build) if code changed
3. Update work status — review/approve finished, update active
4. Commit code changes if uncommitted (orchestrator only — workers never commit)

## Optional Housekeeping

- Note stale branches for cleanup — list local luan/* branches without matching active work issues

## Hard Rules

- NEVER say "ready to commit when you are" — YOU commit
- Push only when user explicitly requests
