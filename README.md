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

3. **Set up the blueprints vault**
   ```bash
   ct blueprint init
   ```
   Creates `~/blueprints/` as a git repo for specs, plans, reviews, and reports. Override the location with `CT_BLUEPRINTS_DIR`. The vault is an [Obsidian](https://obsidian.md) vault — open it in Obsidian for graph navigation, search, and wiki-link resolution. The [Note Annotations](obsidian://show-plugin?id=note-annotations) plugin is worth grabbing for inline highlights on artifacts.

4. **Reinstall plugins**
   Plugin state is not committed. Open Claude Code and reinstall plugins via the plugin manager.

## Quick Start

```bash
/vibe "add user authentication"      # Full pipeline, hands-off
/spec "add user authentication"      # Research codebase, produce spec
/develop                             # Execute implementation from spec
/review                              # Adversarial code review
/commit                              # Conventional commit
```

## Pipeline

```
brainstorm → spec → develop → split-commit → review → commit
```

- **brainstorm**: Collaborative design for greenfield features
- **spec**: Research codebase and produce a target-state spec
- **develop**: Execute implementation from a spec file
- **split-commit**: Repackage branch into clean, tested commits
- **review**: Adversarial review with built-in fix loop + polish
- **commit**: Conventional commit

## Other Skills

| Category | Skills |
|----------|--------|
| **Workflow** | `/supervibe`, `/simplify`, `/loop`, `/schedule`, `/challenge` |
| **Git & PRs** | `/start`, `/gt`, `/git-surgeon`, `/pr-descr`, `/pr-comments`, `/pr-ci`, `/babysit` |
| **Testing & Debug** | `/test-plan`, `/debugging`, `/agent-tdd` |
| **Creation** | `/rule-creator`, `/skill-creator`, `/frontend-design`, `/claude-api` |
| **Utilities** | `/show-image`, `/update-config`, `/keybindings-help` |

## License

Do whatever you want with this.
