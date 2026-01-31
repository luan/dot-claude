# ~/.claude

My Claude Code configuration. Skills, rules, and workflows for AI-assisted development.

## Quick Start

```bash
/explore "add user authentication"   # Research + propose approaches
/implement                           # Execute the plan
/commit                              # Conventional commit
```

## Directory Structure

```
~/.claude/
├── .agents/              # Workflow state tracking
│   ├── sessions/         # Cross-project session context
│   └── archive/          # Historical sessions
├── skills/               # Custom slash commands
├── rules/                # Language-specific guidelines
├── hooks/                # Automation scripts
├── plugins/              # Plugin configs
├── CLAUDE.md             # Core development guidelines
└── settings.json         # Permissions & environment
```

## Skills

### Workflow

| Command | What it does |
|---------|--------------|
| `/explore <prompt>` | Deep codebase exploration, writes plan with 2-3 approaches |
| `/implement [plan]` | Execute plan, track progress in `.agents/active-{branch}.md` |
| `/next-phase` | Continue multi-phase implementations |
| `/save-state` | Save session context for later |
| `/resume-state` | Load session context |

### Git & PRs

| Command | What it does |
|---------|--------------|
| `/commit` | Conventional commits that explain WHY |
| `/pr-fix-comments` | Fetch + fix unresolved PR comments |
| `/pr-fix-gha` | Fix failed GitHub Actions |
| `/pr-reviewers` | Smart reviewer recommendations |
| `/pr-superfresh` | Full PR refresh: sync → fix → submit |

### Graphite Stacks

| Command | What it does |
|---------|--------------|
| `/stack-create` | New branch in stack |
| `/stack-nav` | Navigate up/down/top/bottom |
| `/stack-sync` | Sync with trunk |
| `/stack-submit` | Push + create/update PRs |
| `/stack-modify` | Amend, squash, absorb |
| `/stack-ops` | Fold, move, split, reorder |

## Workflow Philosophy

```
/explore → understand before building
/implement → execute with progress tracking
/commit → explain why, not what
```

State persists across sessions. Pick up where you left off.

## Per-Project Setup

Projects can have their own `.agents/` folder:

```
your-project/
└── .agents/
    ├── plans/           # Exploration outputs
    ├── active-*.md      # Implementation progress
    └── archive/         # Completed work
```

## Rules

Language-specific guidelines in `rules/`:

- `rust.md` - Zero warnings, clippy, latest edition
- `cargo.md` - Dependency preferences
- `style.md` - Code style preferences

## License

Do whatever you want with this.
