# commons

Personal Claude Code plugin — skills, rules, and tools for AI-assisted development.

## Setup

This repo is a Claude Code plugin installed via a local marketplace.
It is **not** a `$HOME/.claude` directory —
personal config (CLAUDE.md, settings.json, hooks) lives in `$HOME/.claude` directly.

```
$HOME/AI/
├── commons/          ← this repo (commons plugin)
├── gt/               ← Graphite CLI plugin (optional)
└── dot-claude/       ← symlink to $HOME/.claude

$HOME/.claude/
├── local-plugins/
│   ├── .claude-plugin/marketplace.json   ← declares all local plugins
│   └── plugins/
│       ├── commons → $HOME/AI/commons
│       └── gt      → $HOME/AI/gt
├── settings.json     ← enabledPlugins: commons@local, gt@local
├── CLAUDE.md         ← personal global instructions
└── rules/            ← personal global rules
```

To install from scratch:

```bash
git clone <repo-url> $HOME/AI/commons
ln -sf $HOME/AI/commons $HOME/.claude/local-plugins/plugins/commons
```

Then in `$HOME/.claude/settings.json`:

```json
{
  "extraKnownMarketplaces": {
    "local": { "source": { "source": "directory", "path": "<home>/.claude/local-plugins" } }
  },
  "enabledPlugins": { "commons@local": true }
}
```

## What's in here

### Skills

```
/vibe           Full autonomous pipeline: scope → develop → commit
/scope          Research codebase, design, create implementation tasks
/develop        Execute epic/tasks (auto solo or swarm)
/brainstorm     Collaborative design for greenfield features
/review         Adversarial code review with fix loop
/commit         Conventional commit
/start          Create branch (gt or git)
/next           Resume branch work or pick next item from board
/split-commit   Repackage branch into clean vertical commits
/debugging      Systematic root cause investigation
/triage         Convert feedback into phased tasks (no implementation)
/test-plan      Manual test plan from current diff
/acceptance     Verify implementation against acceptance criteria
/pr-descr       Update PR title/description from branch context
/pr-comments    Fix unresolved PR review comments
/frontend-design  Production-grade UI design
/git-surgeon    Hunk-level git operations
/promote        Move skill/rule from personal to shared plugin
/sync-plugins   Pull latest updates for all plugins
/writing-skills Create and edit Claude Code skills
```

### Rules

Language-specific conventions (Rust, Python, Swift, Svelte 5, Cargo)
plus skill authoring guides.

### Tools

Rust crates used by skills (in `tools/crates/`).

## Pipeline

```
brainstorm → scope → develop → split-commit → review → commit
```
