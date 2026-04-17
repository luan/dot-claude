---
name: archive
description: "Archive consumed blueprint artifacts. Moves artifacts to archive/ and stores content in git notes. Triggers: 'archive', 'archive blueprint', 'clean up blueprints', 'done with this spec/plan/review'."
argument-hint: "[<slug-or-path>] [--type <spec|plan|review|report>]"
allowed-tools:
  - Bash
  - Glob
user-invocable: true
---

# Archive

Move consumed blueprint artifacts to `archive/` and store their content as git notes for long-term retrieval.

## Arguments

- `<slug-or-path>` — file path or slug substring to match
- `--type <spec|plan|review|report>` — restrict search to one artifact type (optional)
- No arguments → list active artifacts across all types, let user pick

## Workflow

### Without arguments

List active artifacts:

```bash
ct spec list --all
ct plan list --all
ct review list --all
ct report list --all
```

Present the combined list and ask which to archive via AskUserQuestion.

### With slug argument

Search for matching artifact across types (or filtered by `--type`):

```bash
# Try each type, find file whose name contains the slug
fd "<slug>" ~/blueprints/ --type f --extension md --exclude archive
```

If exactly one match → archive it. Multiple matches → present choices. No match → report and stop.

### Archive

Prefer the MCP `blueprint_archive` tool (`{ stem, kind? }`). It moves the file to `~/blueprints/<project>/archive/`, stores content as a git note, and commits+pushes.

CLI equivalent, useful for batch archival or previewing:

```bash
ct <type> archive <file-path>                     # single
ct <type> archive --batch <f1> <f2> <f3>          # one commit for all
ct <type> archive --dry-run --batch <f1> <f2>     # preview, no writes
```

Report what was archived.
