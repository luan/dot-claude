# commons

Personal Claude Code plugin — skills, rules, and tools for AI-assisted development.

## New User Setup

1. **Set your username**
   Add `"GIT_USERNAME": "<your-handle>"` to the `env` block in `settings.json` or `settings.local.json`. This controls branch prefixes and other user-specific behaviour.

2. **Install the `ct` CLI**
   ```bash
   cd ~/.claude/tools && cargo install --path crates/ct
   ```
   Requires Rust (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).

3. **Reinstall plugins**
   Plugin state is not committed. Open Claude Code and reinstall plugins via the plugin manager.

## Quick Start

```bash
/vibe "add user authentication"      # Full pipeline, hands-off
/scope "add user authentication"     # Research, design, create tasks
/develop <epic-id>                   # Execute (solo or swarm)
/review                              # Adversarial code review
/commit                              # Conventional commit
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

- **brainstorm**: Collaborative design for greenfield features
- **scope**: Research codebase, design, and create implementation tasks
- **develop**: Workers own TDD from briefs (auto solo/swarm)
- **split-commit**: Repackage branch into clean, tested commits
- **review**: Adversarial review with built-in fix loop + polish
- **commit**: Conventional commit

Other skills: `/test-plan`, `/triage`, `/debugging`, `/next`, `/start`,
`/split-commit`, `/gt`, `/acceptance`, `/pr:descr`,
`/pr:comments`, `/frontend-design`

## License

Do whatever you want with this.
