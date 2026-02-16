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
explore → prepare → implement → review → commit
```

- **explore**: Research codebase, produce phased design in issue
- **prepare**: One subagent creates epic + task briefs (no code)
- **implement**: Workers own TDD from briefs (auto solo/swarm)
- **review**: Adversarial review with built-in fix loop + polish
- **commit**: Conventional commit

## All Skills

### Core Pipeline

| Command | What it does |
|---------|--------------|
| `/explore <prompt>` | Deep research, stores design in issue |
| `/prepare <issue-id>` | Design → epic + task briefs |
| `/implement <epic-id>` | Execute tasks (auto solo/swarm) |
| `/review [--team]` | Adversarial review (3-perspective with --team) |
| `/commit` | Conventional commit |

### Investigation & Planning

| Command | What it does |
|---------|--------------|
| `/debugging` | Root-cause-first bug diagnosis |
| `/fix <feedback>` | Convert feedback → one issue with phases |
| `/resume-work` | Context recovery after a break |
| `/reference-sync` | Check upstream for new patterns |

### Git & PRs

| Command | What it does |
|---------|--------------|
| `/start <desc>` | Create branch + link issue |
| `/graphite` | Graphite stack operations |
| `/split-commit` | Repackage branch into clean commits |
| `/git-surgeon` | Hunk-level staging/unstaging |
| `/pr-description` | Update PR title and body |
| `/pr-fix-comments` | Address PR review feedback |
| `/pr-fix-gha` | Fix failing CI/GitHub Actions |
| `/pr-reviewers` | Smart reviewer recommendations |

### Utilities

| Command | What it does |
|---------|--------------|
| `/compress-prompt` | Compress text for AI consumption |
| `/writing-skills` | Create or verify skills |

## Directory Structure

```
~/.claude/
├── skills/               # Slash command skills (SKILL.md each)
├── rules/                # Global rules (inherited by subagents)
├── hooks/                # Pre/post tool automation
├── references/           # Upstream patterns (git submodule)
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

## Model Tiering

| Tier | Model | When |
|------|-------|------|
| Deep | opus | Default — anything with ambiguity |
| Standard | sonnet | Purely mechanical, zero ambiguity |
| Fast | haiku | Commit messages, compression |

## License

Do whatever you want with this.
