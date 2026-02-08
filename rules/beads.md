# Beads Conventions

## Atomic Claims
`bd update <id> --claim` not `--status in_progress`.
Sets assignee + status atomically, fails if claimed. Race-safe.

## Discovery Linking
```bash
bd create "Found: <description>" --type bug --validate --deps discovered-from:<parent-id>
```

## Multi-Phase Epics
5+ tasks → phases with blocking deps.
`bd ready` auto-scopes to current phase.

## Labels
`bd label add <id> architecture|implementation|testing`
Enables `bd ready --label <label>` filtering.

## Commit Traceability
Append beads ID when task in_progress:
```
fix(auth): handle token expiry (bd-abc123)
```

## Git Hooks
`bd hooks install` per repo. Prevents stale JSONL on push.
Verify: `bd hooks list` (5 hooks: pre-commit, post-merge,
pre-push, post-checkout, prepare-commit-msg).

## Bulk Creation
`bd create --file plan.md`

## Sync
`bd sync` — no flags. Exports to JSONL.
NOT `bd sync --from-main` (flag doesn't exist).

## Swarm Coordination
`bd swarm validate` before team work — parallelism + DAG check.
`bd swarm status` monitors progress.
Workers: `bd ready --parent <epic-id> --unassigned` for scoped discovery.
`bd merge-slot acquire --wait` then `release` serializes git ops.
Slots auto-release after 5min (deadlock prevention).

**Bug (v0.49.4):** `bd swarm create` fails —
"invalid issue type: molecule". Workaround:
```bash
bd create --type molecule --title "swarm: <epic-id>" \
  --parent <epic-id> --mol-type swarm
```
Skip `bd swarm create` until fixed.
