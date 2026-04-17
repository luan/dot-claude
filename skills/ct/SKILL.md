---
name: ct
description: "Reference for the ct CLI and the ~/blueprints/ artifact system. Use this skill whenever the user mentions ct, blueprints, vault, artifacts, specs, plans, reviews, reports, or docs in the context of documentation or project knowledge — even if they don't say 'ct' explicitly. Also trigger when the user wants to find, list, search, read, create, archive, or link artifacts, or asks about the blueprints vault, Obsidian integration, or artifact lifecycle. Covers correct ct command patterns, artifact metadata (tags, source links), and the blueprints directory layout."
user-invocable: false
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
---

# ct — Artifact CLI

`ct` manages structured artifacts in `~/blueprints/`, a separate git repo and Obsidian vault. Five artifact types share identical CRUD commands: **spec**, **plan**, **review**, **report**, **doc**.

Workflow skills (`/spec`, `/develop`, `/crit`, `/report`, `/archive`) handle their respective lifecycle phases. This skill is the reference for direct ct operations and correct command patterns.

## Quick Reference

| Operation | Command |
|-----------|---------|
| Create (scaffold) | `ct <type> create --topic "..."` — prints path, author body via Read/Edit |
| Create dive | `ct spec create --dive --source "<hub-stem>" --slug "<hub-slug>-<subtopic>" --topic "..."` |
| Read | `ct <type> read <path-or-slug>` |
| List active | `ct <type> list` |
| List with dives | `ct spec list --include-dives` |
| List all | `ct <type> list --all` |
| Latest | `ct <type> latest` |
| Archive | `ct <type> archive <path>` |
| Archive batch | `ct <type> archive --batch <f1> <f2> ...` |
| Archive preview | `ct <type> archive --dry-run [--batch ...] <path>` |
| Prune old | `ct <type> prune [--days N]` |
| Show by slug | `ct <type> show <slug>` |
| Search vault | `ct vault search "<query>"` |
| Find related | `ct vault related "<topic>"` |
| Check links | `ct vault check` |
| Project name | `ct vault project` |

**Type aliases:** `ct p` = plan, `ct s` = spec, `ct r` = review, `ct rp` = report, `ct d` = doc.

## Create with Full Metadata

`ct create` scaffolds an empty artifact — it writes frontmatter only, prints the absolute path, and commits. **Author the body after creation using Read/Edit on the returned path**, then re-commit. Never pipe body content into `ct create`.

### Pattern A — MCP (preferred)

When `ct` is exposed as an MCP server, prefer the MCP tools over the CLI. The exact tool names depend on how the harness prefixes MCP tools; look for a create tool on the `ct` server (e.g. `artifact_create`). Typical flow:

1. Call the `artifact_create` tool with `{ kind, topic, project?, slug?, source?, tags? }`. Capture `path` from the structured response.
2. Use Read/Edit to author the body at that path.
3. Call the `artifact_commit_edits` tool with `{ path }` to commit and push the edits.

### Pattern B — CLI (fallback)

```bash
ARTIFACT=$(ct <type> create \
  --topic "<human-readable title>" \
  --source "<parent-artifact-stem>" \
  --tags "domain/combat,stage/research")
# $ARTIFACT is the absolute path to the new (frontmatter-only) file.
```

Then Read/Edit the body at `$ARTIFACT`. To commit the edits afterwards, prefer calling `artifact_commit_edits` via MCP. If MCP is unavailable, commit by hand from the vault:

```bash
git -C "$(dirname "$(dirname "$ARTIFACT")")" commit -am "<message>" && \
git -C "$(dirname "$(dirname "$ARTIFACT")")" push
```

### Metadata

**Auto-derived tags** (always added): `type/<kind>`, `project/<name>`.
**User tags** (via `--tags` or the MCP `tags` field): `domain/<area>`, `stage/<phase>`, or freeform.

`--source` (or MCP `source`) adds `source: "[[stem]]"` to frontmatter — use when an artifact derives from another (plan from spec, review against spec). Pass a bare stem; ct wraps it in `[[...]]`.

## Dives: vision-level specs in `dive/`

A dive is a spec at vision/architecture altitude — the broader context or high-level solution design that a hub spec links to. Dives are semantically specs (same `type/spec` tag, same ct machinery) but live in a sibling `dive/` folder so the top-level `spec/` list stays scannable as "major things we're building."

```bash
ct spec create --dive \
  --topic "<dive subtopic>" \
  --slug "<hub-slug>-<dive-subtopic-slug>" \
  --source "<hub-stem>" \
  --tags "domain/<area>,stage/research"
```

Rules for dives:
- `--dive` requires `--source` — a dive without a hub link is rejected.
- `--dive` is only valid for `spec` artifacts; `ct plan create --dive` is rejected.
- Pass `--source` as a bare stem (e.g. `20260411-protocol-hub`); ct wraps it in `[[...]]` automatically when writing the frontmatter. Passing `[[...]]` yourself results in `[[[[...]]]]`.
- Always pass an explicit `--slug` composed as `<hub-slug>-<dive-subtopic-slug>`. The hub prefix prevents collisions across brainstorms and groups dives from the same hub in alphabetical sort.
- `ct spec list` hides dives by default. Use `ct spec list --include-dives` to see them.
- `ct spec read <stem>` and `ct spec show <stem>` find dives by bare stem without a flag.
- Archiving a dive preserves the subfolder: `ct spec archive <path-to-dive>` moves to `archive/<project>/dive/`, not `archive/<project>/spec/`.

## Linking

After creating any artifact, check for related work:

```bash
RELATED=$(ct vault related "<topic>")
# If non-empty, append a ## Related section with [[wiki-links]] to the artifact
```

Link by stem (filename without extension or path) — Obsidian resolves across the vault. Keep linking shallow: don't read related files to summarize, just link by name.

## Common Patterns

**Find what exists for this project:**

```bash
ct spec list && ct plan list && ct review list && ct report list && ct doc list
```

**Resume from latest artifact:**

```bash
ct plan latest
# Falls back to: ct spec latest
```

**Read an artifact's content:**

```bash
ct <type> read <path>           # body only
ct <type> read <path> --json    # frontmatter as JSON
```

**Archive after consumption:**

```bash
ct <type> archive <path>
# Moves to archive/, stores content as git note, commits+pushes

# Batch archive (single commit for all files):
ct <type> archive --batch <file1> <file2> <file3>

# Preview what would be archived:
ct <type> archive --dry-run --batch <file1> <file2>
```

**Check doc staleness:**

```bash
ct tool check-refs <doc-path> --project-root "$PROJECT_ROOT"
# Outputs JSON: {doc, total_refs, valid, missing, staleness (0.0-1.0)}
```

**Module stability analysis:**

```bash
ct tool churn --project-root "$PROJECT_ROOT" --since 2w --min-loc 500
# Outputs JSON array: [{module, loc, commits, last_change}, ...]
```

## Rules

- Always use `ct <type> create` (or the MCP `artifact_create` tool) — never write blueprint files directly.
- Prefer MCP tools when `ct` is registered as an MCP server; fall back to the CLI otherwise.
- `ct create` scaffolds frontmatter only. Author the body via Read/Edit on the returned path, then commit (MCP `artifact_commit_edits`, or `git -C <vault> commit/push`).
- `ct` auto-commits and pushes after `create` and `archive`. If push fails, stop and report.
- Never force-push the blueprints repo.
- No absolute paths in frontmatter — use tags and project names.
- The vault is canonical. Repos may snapshot frozen copies, but edits happen in the vault.

For the full blueprints directory layout, tag system, and Obsidian integration details, read `${CLAUDE_SKILL_DIR}/references/blueprints.md`.
