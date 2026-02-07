# Beads Conventions

## Atomic Claims
`bd update <id> --claim` instead of `--status in_progress`.
Atomic: sets assignee + status in one op, fails if already claimed.
Race-safe for team workflows.

## Discovery Linking
```bash
bd create "Found: <description>" --type bug --validate --deps discovered-from:<parent-id>
```

## Multi-Phase Epics
5+ tasks → group into phases with blocking deps.
`bd ready` auto-scopes to current phase — blocked tasks hidden.

## Labels
Tag tasks: `bd label add <id> architecture|implementation|testing`.
Enables `bd ready --label <label>` filtering in team workflows.

## Commit Traceability
Append beads ID to commits when task in_progress:
```
fix(auth): handle token expiry (bd-abc123)
```

## Git Hooks
`bd hooks install` per repo. Prevents stale JSONL on push.
Verify: `bd hooks list` (5 hooks: pre-commit, post-merge,
pre-push, post-checkout, prepare-commit-msg).

## Bulk Creation
`bd create --file plan.md`

## Swarm Coordination
`bd swarm validate` before team work — parallelism + DAG check.
`bd swarm create` registers swarm. `bd swarm status` monitors.
Workers: `bd ready --parent <epic-id> --unassigned` for scoped discovery.
`bd merge-slot acquire/release` serializes git ops across agents.
Slots auto-release after 5min inactivity (deadlock prevention).
