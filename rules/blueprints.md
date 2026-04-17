---
---

# Blueprints

All structured artifacts (specs, plans, reviews, reports, docs) live in the blueprints vault (`$CT_BLUEPRINTS_DIR`, default `~/blueprints/`) via the `ct` tool. The blueprints repo is a separate git repository — `ct` handles commit+push automatically after every write.

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
| Init repo | `ct vault init` |
| Migrate from ~/.claude/ | `ct vault migrate` |
| Project name | `ct vault project` |
| Create | `ct <type> create --topic "..."` |
| Read | `ct <type> read <file>` |
| List | `ct <type> list [--all]` |
| Latest | `ct <type> latest` |
| Archive | `ct <type> archive <file>` |
| Prune | `ct <type> prune [--days N]` |

Where `<type>` is `spec`, `plan`, `review`, `report`, or `doc`.

## Linking (Obsidian)

The blueprints repo is an Obsidian vault. Use `[[wiki-links]]` to connect related artifacts.

- **`--source`**: When creating an artifact derived from another (plan from spec, review against spec), pass `--source <stem>` to `ct create`. This adds `source: "[[stem]]"` to frontmatter.
- **Related artifacts**: After creating an artifact, run `ct vault related "<topic>"`. If matches found, append a `## Related` section with the wiki-links.
- **Inline links**: When referencing another artifact in body text, use `[[stem]]` (filename without extension or path — Obsidian resolves across the vault).
- Keep linking shallow — don't read related files to summarize them, just link by name.

| Operation | Command |
|-----------|---------|
| Find related | `ct vault related "<topic>"` |
| Link source | `ct <type> create --source "<stem>" ...` |
| Check links | `ct vault check` |
| Search | `ct vault search "<query>"` |

## Tags

All artifacts have `tags:` in frontmatter. `ct` auto-derives `type/` and `project/` tags; add domain/stage tags via `--tags`.

**Auto-derived** (always added by `ct create`):
- `type/spec`, `type/plan`, `type/review`, `type/report`, `type/doc`
- `project/<name>` (from project path)

**User-supplied** (via `--tags "domain/combat,stage/research"`):
- `domain/<area>` — combat, lua, ui, network, etc.
- `stage/<phase>` — research, implementing, shipped, stale
- Any freeform tag

**Permanent docs** in `docs/` use `type/doc` tag. These are reference documents (architecture, API guides) — not workflow artifacts.

## Dives

A dive is a vision-level spec linked to a hub spec. It lives in a sibling `dive/` folder so the top-level `spec/` list stays scannable as "major things we're building." Dives share the `type/spec` tag.

- Create via `dive: true` (MCP `blueprint_create`) or `--dive` (CLI). Both require a `source` — a dive without a hub link is rejected.
- Dive-only for specs; rejected for other artifact kinds.
- Slug convention: `<hub-slug>-<subtopic>` so dives from the same hub sort together.
- `ct spec list` hides dives by default; `--include-dives` to see them. `blueprint_read` / `ct spec read <stem>` finds dives by bare stem.
- Archive preserves the subfolder: dives archive to `archive/<project>/dive/`.

## Rules

- Always use `ct <type> create` for artifact writes — never write blueprint files directly.
- `--project` auto-detects from cwd (git toplevel, falls back to cwd). Pass it only to target a different project.
- If push fails during commit+push, stop and report to user. Never force-push.
- `ct vault init` must be run before first use. `ct` errors if the vault directory is missing.
- Set `CT_BLUEPRINTS_DIR` to override the default `~/blueprints/` location.
