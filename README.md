# ~/.claude

Claude Code configuration. Skills, rules, hooks, and workflows
for AI-assisted development with work issue tracking.

## Quick Start

```bash
/explore "add user authentication"   # Research + design
/prepare <issue-id>                  # Create epic + task briefs
/implement <epic-id>                 # Execute (solo or swarm)
/review                              # Adversarial code review
/commit                              # Conventional commit
```

## Pipeline

```
brainstorm → explore → prepare → implement → split-commit → review → commit
```

- **brainstorm**: Collaborative design for greenfield features
- **explore**: Research codebase, produce phased design in issue
- **prepare**: One subagent creates epic + task briefs (no code)
- **implement**: Workers own TDD from briefs (auto solo/swarm)
- **split-commit**: Repackage branch into clean, tested commits
- **review**: Adversarial review with built-in fix loop + polish
- **commit**: Conventional commit

## All Skills

### Core Pipeline

| Command                | What it does                                   |
| ---------------------- | ---------------------------------------------- |
| `/explore <prompt>`    | Deep research, stores design in issue          |
| `/prepare <issue-id>`  | Design → epic + task briefs                    |
| `/implement <epic-id>` | Execute tasks (auto solo/swarm)                |
| `/review [--team]`     | Adversarial review (3-perspective with --team) |
| `/commit`              | Conventional commit                            |

### Investigation & Planning

| Command           | What it does                             |
| ----------------- | ---------------------------------------- |
| `/brainstorm`     | Collaborative design for new ideas       |
| `/debugging`      | Root-cause-first bug diagnosis           |
| `/fix <feedback>` | Convert feedback → one issue with phases |
| `/next`           | Resume branch work or dispatch next item |

### Git & PRs

| Command            | What it does                        |
| ------------------ | ----------------------------------- |
| `/start <desc>`    | Create branch + link issue          |
| `/graphite`        | Graphite stack operations           |
| `/split-commit`    | Repackage branch into clean commits |
| `/git-surgeon`     | Hunk-level staging/unstaging        |
| `/pr-description`  | Update PR title and body            |
| `/pr-fix-comments` | Address PR review feedback          |
| `/pr-fix-gha`      | Fix failing CI/GitHub Actions       |
| `/pr-reviewers`    | Smart reviewer recommendations      |

### Scaffolding & UI

| Command                       | What it does                            |
| ----------------------------- | --------------------------------------- |
| `/bootstrap:web <name>`       | Scaffold a new web project              |
| `/bootstrap:caddy <name>`     | Register project in local dev routing   |
| `/frontend-design`            | Production-grade frontend interfaces    |

### Utilities

| Command           | What it does            |
| ----------------- | ----------------------- |
| `/writing-skills` | Create or verify skills |

## Directory Structure

```
~/.claude/
├── skills/               # Slash command skills (SKILL.md each)
├── rules/                # Global rules (inherited by subagents)
│   ├── test-quality.md   # Test standards + banned patterns
│   ├── subagent-trust.md # Adversarial verification policy
│   ├── rust.md, cargo.md # Language-specific rules
│   └── ...
├── hooks/                # Pre/post tool automation
├── CLAUDE.md             # Core instructions
├── settings.json         # Permissions & environment
└── README.md
```

## Work Issues (Issue Tracking)

All plans, notes, and state live in work issues — no filesystem
docs. Lifecycle: open → active → review → done / cancelled.

```bash
work create "title" --type chore     # Create issue
work list                            # List open issues
work show <id>                       # Show details
work edit <id> --description "..."   # Store design findings
work start <id>                      # Mark active
work review <id>                     # Submit for review
work approve <id>                    # Mark done (after review)
```

## License

Do whatever you want with this.
