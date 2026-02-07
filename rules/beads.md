# Beads Conventions

## Atomic Claims
Use `bd update <id> --claim` instead of `--status in_progress`. Atomic: sets assignee + status in one operation, fails if already claimed. Race-safe for team workflows.

## Discovery Linking
Single command for side quests found during implementation:
```bash
bd create "Found: <description>" --type bug --validate --deps discovered-from:<parent-id>
```

## Multi-Phase Epics
For 5+ tasks, group into phases with blocking deps. `bd ready` auto-scopes to current phase — blocked tasks stay hidden until earlier phases complete.

## Labels for Persona Filtering
Tag tasks during exploration: `bd label add <id> architecture|implementation|testing`. Enables `bd ready --label <label>` filtering in team workflows.

## Commit Traceability
Append beads issue ID to commit messages when a task is in_progress:
```
fix(auth): handle token expiry (bd-abc123)
```

## Git Hooks
Run `bd hooks install` in each project repo. Installs git hooks that prevent stale JSONL on push. Verify with `bd hooks list` (5 hooks: pre-commit, post-merge, pre-push, post-checkout, prepare-commit-msg).

## Bulk Task Creation
For plans with many tasks: `bd create --file plan.md`

## Swarm Coordination
`bd swarm validate` before team work — computes parallelism, detects DAG issues.
`bd swarm create` registers swarm. `bd swarm status` for monitoring.
Workers: `bd ready --parent <epic-id> --unassigned` for scoped task discovery.
`bd merge-slot acquire/release` serializes git ops across agents.
Slots auto-release after 5min inactivity to prevent deadlocks from crashed agents.
