# bd CLI Reference

Never guess bd flags. Use only flags listed here. When unsure, run
`bd <command> --help`.

## Common Commands

### bd show <id>
View issue details. Flags: `--short`, `--children`, `--refs`,
`--thread`, `--json`, `--as-of <commit>`, `--local-time`.

To get a single field: `bd show <id> --json | jq '.status'`

### bd list
List issues. Key flags:
- Filter: `--status`, `--type`, `--assignee`, `--parent`, `--label`,
  `--id` (comma-separated), `--ready`, `--all`, `--pinned`,
  `--overdue`, `--no-assignee`
- Search: `--title`, `--desc-contains`, `--notes-contains`
- Display: `--pretty`/`--tree`, `--long`, `--limit`, `--sort`,
  `--reverse`, `--json`, `--no-pager`
- Date: `--created-after`, `--updated-after`, `--closed-after`,
  `--due-before`, `--due-after`

### bd create [title]
Create issue. Key flags: `-t`/`--type`, `-p`/`--priority`,
`-d`/`--description`, `-a`/`--assignee`, `--parent`, `--labels`,
`--notes`, `--design`, `--acceptance`, `--file` (bulk from markdown),
`--silent`, `--body-file`, `--deps`, `--estimate`, `--due`, `--defer`.

### bd update <id>
Update issue. Key flags: `-s`/`--status`, `--title`,
`-d`/`--description`, `-a`/`--assignee`, `-p`/`--priority`,
`--claim` (atomic: sets assignee + in_progress), `--notes`,
`--design`, `--acceptance`, `--parent`, `--add-label`,
`--remove-label`, `--set-labels`, `--due`, `--defer`, `--estimate`,
`--metadata`.

### bd close <id>
Close issue. Flags: `-r`/`--reason`, `--continue` (auto-advance
molecule), `--suggest-next`, `-f`/`--force` (pinned issues).

### bd edit <id>
Open editor for field. Flags: `--description` (default), `--design`,
`--notes`, `--acceptance`, `--title`.

### bd sync
Sync JSONL <-> DB. Flags: `--status` (show state), `--export-only`,
`--import-only`, `--resolve` + `--ours`/`--theirs` (conflicts).

## Does NOT Exist
- `bd claim` — use `bd update <id> --claim`
- `bd show --field` — use `bd show <id> --json | jq`
- `bd list --short` — use `bd list --json` or `bd show <id> --short`

## Global Flags (all commands)
`--json`, `-q`/`--quiet`, `-v`/`--verbose`, `--readonly`,
`--sandbox`, `--no-daemon`.
