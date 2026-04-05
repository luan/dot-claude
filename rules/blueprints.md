---
paths:
  - "skills/**"
  - "rules/**"
---

# Blueprints

All structured artifacts (specs, plans, reviews, reports) live in `~/blueprints/<project>/` via the `ct` tool. The blueprints repo is a separate git repository — `ct` handles commit+push automatically after every write.

## Layout

```
~/blueprints/<project>/
  spec/       # target-state specifications
  plan/       # implementation plans
  review/     # code review findings
  report/     # post-implementation summaries
  archive/    # consumed artifacts from all types
```

## Commands

| Operation | Command |
|-----------|---------|
| Init repo | `ct blueprint init` |
| Migrate from ~/.claude/ | `ct blueprint migrate` |
| Project name | `ct blueprint project` |
| Create | `ct <type> create --topic "..." --project "$(git rev-parse --show-toplevel)"` |
| Read | `ct <type> read <file>` |
| List | `ct <type> list [--all]` |
| Latest | `ct <type> latest` |
| Archive | `ct <type> archive <file>` |
| Prune | `ct <type> prune [--days N]` |

Where `<type>` is `spec`, `plan`, `review`, or `report`.

## Rules

- Always use `ct <type> create` for artifact writes — never write blueprint files directly.
- If push fails during commit+push, stop and report to user. Never force-push.
- `ct blueprint init` must be run before first use. `ct` errors if `~/blueprints/` is missing.
