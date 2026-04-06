---
---

# Blueprints

All structured artifacts (specs, plans, reviews, reports) live in `~/blueprints/<project>/` via the `ct` tool. The blueprints repo is a separate git repository — `ct` handles commit+push automatically after every write.

**Principle:** The vault is canonical. Repos may snapshot frozen copies for contributor access, but edits happen in the vault.

## Layout

```
~/blueprints/<project>/
  spec/       # target-state specifications
  plan/       # implementation plans
  review/     # code review findings
  report/     # post-implementation summaries
  docs/       # permanent reference docs (architecture, guides)
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

## Linking (Obsidian)

The blueprints repo is an Obsidian vault. Use `[[wiki-links]]` to connect related artifacts.

- **`--source`**: When creating an artifact derived from another (plan from spec, review against spec), pass `--source <stem>` to `ct create`. This adds `source: "[[stem]]"` to frontmatter.
- **Related artifacts**: After creating an artifact, run `ct blueprint related --project "$(git rev-parse --show-toplevel)" "<topic>"`. If matches found, append a `## Related` section with the wiki-links.
- **Inline links**: When referencing another artifact in body text, use `[[stem]]` (filename without extension or path — Obsidian resolves across the vault).
- Keep linking shallow — don't read related files to summarize them, just link by name.

| Operation | Command |
|-----------|---------|
| Find related | `ct blueprint related --project "..." "<topic>"` |
| Link source | `ct <type> create --source "<stem>" ...` |
| Check links | `ct blueprint check` |
| Search | `ct blueprint search "<query>"` |

## Tags

All artifacts have `tags:` in frontmatter. `ct` auto-derives `type/` and `project/` tags; add domain/stage tags via `--tags`.

**Auto-derived** (always added by `ct create`):
- `type/spec`, `type/plan`, `type/review`, `type/report`
- `project/<name>` (from project path)

**User-supplied** (via `--tags "domain/combat,stage/research"`):
- `domain/<area>` — combat, lua, ui, network, etc.
- `stage/<phase>` — research, implementing, shipped, stale
- Any freeform tag

**Permanent docs** in `docs/` use `type/doc` tag. These are reference documents (architecture, API guides) — not workflow artifacts.

## Rules

- Always use `ct <type> create` for artifact writes — never write blueprint files directly.
- If push fails during commit+push, stop and report to user. Never force-push.
- `ct blueprint init` must be run before first use. `ct` errors if `~/blueprints/` is missing.
