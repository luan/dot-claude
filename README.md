# ~/.claude

Claude Code configuration. Skills, rules, hooks, and workflows
for AI-assisted development with beads issue tracking.

## Quick Start

```bash
/explore "add user authentication"   # Research + design
/prepare <bead-id>                   # Create epic + task briefs
/implement <epic-id>                 # Execute (solo or swarm)
/review                              # Adversarial code review
/refine                              # Polish (after clean review)
/commit                              # Conventional commit
```

## Pipeline

```
explore → prepare → implement → review → refine → commit
```

- **explore**: Research codebase, produce phased design in bead
- **prepare**: One subagent creates epic + task briefs (no code)
- **implement**: Workers own TDD from briefs (auto solo/swarm)
- **review**: Adversarial review with built-in fix loop
- **refine**: Cosmetic polish on reviewed code
- **commit**: Conventional commit + beads sync

## All Skills

### Core Pipeline

| Command | What it does |
|---------|--------------|
| `/explore <prompt>` | Deep research, stores design in bead |
| `/prepare <bead-id>` | Design → epic + task briefs |
| `/implement <epic-id>` | Execute tasks (auto solo/swarm) |
| `/review [--team]` | Adversarial review (3-perspective with --team) |
| `/refine` | Cosmetic polish after review |
| `/commit` | Conventional commit + beads sync |

### Investigation & Planning

| Command | What it does |
|---------|--------------|
| `/debugging` | Root-cause-first bug diagnosis |
| `/fix <feedback>` | Convert feedback → one bead with phases |
| `/resume-work` | Context recovery after a break |
| `/reference-sync` | Check upstream for new patterns |

### Git & PRs

| Command | What it does |
|---------|--------------|
| `/start <desc>` | Create branch + link bead |
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
├── .beads/               # Issue tracking database
├── CLAUDE.md             # Core instructions
├── settings.json         # Permissions & environment
└── README.md
```

## Beads (Issue Tracking)

All plans, notes, and state live in beads — no filesystem docs.

```bash
bd create "title" --type task    # Create issue
bd list                          # List open issues
bd show <id>                     # Show details
bd update <id> --design "..."    # Store design findings
bd swarm validate <epic-id>      # Validate parallel execution
bd sync                          # Sync after commits
```

## Model Tiering

| Tier | Model | When |
|------|-------|------|
| Deep | opus | Default — anything with ambiguity |
| Standard | sonnet | Purely mechanical, zero ambiguity |
| Fast | haiku | Commit messages, compression |

## License

Do whatever you want with this.
