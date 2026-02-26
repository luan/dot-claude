# ~/.claude

Claude Code configuration. Skills, rules, and workflows
for AI-assisted development.

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
