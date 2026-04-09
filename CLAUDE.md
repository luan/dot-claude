# commons

Source for the `commons` Claude Code plugin — skills, rules, hooks, and Rust tools for AI-assisted development.

## Layout

```
skills/       # One directory per skill (vibe, scope, develop, review, commit, ...)
rules/        # Language and workflow conventions loaded by the plugin
hooks/        # Claude Code hook scripts
tools/        # Rust workspace
  crates/ct/  # ct binary (terminal UI toolkit used by skills)
local-plugins/ # Local plugin overrides (gt alias)
```

## ct crate

`ct` is a Rust CLI at `tools/crates/ct/`. It provides TUI primitives (ratatui, crossterm, image rendering) consumed by skills at runtime.

Build and install:
```bash
cd ~/AI/commons/tools && cargo install --path crates/ct
```

## Plugin install / refresh

```bash
claude plugin install ~/AI/commons@local
```

Run this after any edit to skills, rules, or hooks. Plugin state is not committed — reinstall after cloning.

## Build (ct only)

```bash
cd ~/AI/commons/tools
cargo build           # dev
cargo install --path crates/ct   # install to PATH
```

## RTK

RTK (Rust Token Killer) is a separate tool, documented in `~/AI/commons/RTK.md`. It is not built from this repo — it is a pre-installed binary on PATH.
