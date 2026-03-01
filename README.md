# dot-claude

Personal Claude Code plugin — skills, rules, and tools for AI-assisted development.

## Why a plugin?

This repo used to be a drop-in `$HOME/.claude` directory. That bundled personal config (settings, hooks, keybindings) with shareable skills and rules into one repo, creating several problems:

- **Not shareable.** Cloning someone else's `$HOME/.claude` overwrites your own config.
- **Not composable.** You can't use skills from two repos — only one `$HOME/.claude` exists.
- **Secrets in scope.** `settings.json`, API tokens, and local state lived alongside skills that should be public.

Converting to a Claude Code plugin solves all three. Skills, rules, and agents ship as a plugin that anyone can install alongside their own config. Personal settings stay in `$HOME/.claude` where they belong.

## Setup

```
$HOME/.claude/
├── local-plugins/
│   ├── .claude-plugin/marketplace.json   ← declares local plugins
│   └── plugins/
│       └── dot-claude → $HOME/AI/dot-claude
├── settings.json     ← enabledPlugins: dot-claude@local
├── CLAUDE.md         ← personal global instructions
└── rules/            ← personal global rules
```

Install:

```bash
git clone <repo-url> $HOME/AI/dot-claude
mkdir -p $HOME/.claude/local-plugins/plugins
ln -sf $HOME/AI/dot-claude $HOME/.claude/local-plugins/plugins/dot-claude
```

Then in `$HOME/.claude/settings.json`:

```json
{
  "extraKnownMarketplaces": {
    "local": { "source": { "source": "directory", "path": "<home>/.claude/local-plugins" } }
  },
  "enabledPlugins": { "dot-claude@local": true }
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

Language-specific conventions (Rust, Python, Swift, Svelte 5, Cargo) plus skill authoring guides.

## Pipeline

```
brainstorm → scope → develop → split-commit → review → commit
```

## License

Do whatever you want with this.
