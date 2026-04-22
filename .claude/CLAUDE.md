Personal `~/.claude` — Claude Code configuration, the `ct` Rust CLI, skills, rules, hooks, and a local plugin marketplace. Edits here change how Claude Code behaves in every session.

## Build, test, install

Common tasks live in the `justfile` (run from repo root):

```
just build          # cargo build --release
just check          # cargo fmt --check && cargo clippy -- -W clippy::all && cargo test
just install        # cargo install --path . and re-register ct's MCP servers (blueprint, apply-patch)
just completions    # regenerate fish completions for ct
just setup          # install + completions
```

Run a single test: `cargo test --test apply_patch_contract <name>` or `cargo test --test cli_contract <name>`. The two contract tests under `tools/tests/` are the authoritative integration suite for `ct tool apply-patch` and the CLI surface.

## The `ct` CLI (`tools/`)

`ct` is the central automation binary — one crate, edition 2024, entry `tools/main.rs`. It fans out to subcommand modules:

- `cli/` — clap definitions and dispatch. Top-level commands: `plan|spec|review|report|doc` (blueprint artifact CRUD), `vault` (blueprint-repo management), `read`, `notify`, `sym`, `mcp {blueprint|apply-patch}`, `tool {slug|phases|completion|gitcontext|check-refs|cochanges|churn|apply-patch}`, `apply-patch {stats|prune}`.
- `artifact/` — blueprint artifact CRUD, listing, archiving. All specs/plans/reviews/reports/docs live in the separate blueprints vault repo (`$CT_BLUEPRINTS_DIR`, default `~/blueprints/`), never in this repo.
- `mcp/` — two MCP servers exposed over stdio: `blueprint` (artifact operations for skills) and `apply-patch` (the hardened patch tool). Both are registered via `just install`.
- `apply_patch/` — the patch engine: `parser.rs` (envelope parser), `apply.rs` (the applier with trim/Unicode-normalized fallback matching), `seek_sequence.rs`, plus a `telemetry/` sqlite store (`stats`, `prune`, `enrich`) that records every patch attempt.
- `vault.rs`, `refs.rs`, `gitcontext.rs`, `cochanges.rs`, `churn.rs`, `phases.rs`, `slug.rs`, `notify.rs` — standalone utilities wired into `tool` / `vault` subcommands.
- `tools/crates/sym/` — standalone tree-sitter symbol indexer crate, exposed as `ct sym`. Its CLI (`src/cli.rs`) backs the `sym` skill's `ct sym` invocations.

When wiring a new subcommand: add the variant in `tools/cli/args.rs` (or the relevant submodule), dispatch in `tools/main.rs::main`, implement in the matching module, and cover it in `tools/tests/cli_contract.rs`.

## Skills (`skills/`) and the pipeline

27 user-invocable skills in `skills/<name>/SKILL.md`. The canonical workflow is:

```
brainstorm → spec → develop → split-commit → review (crit|superreview) → commit
```

`vibe` chains spec → develop → review → commit → report. `supervibe` breaks a larger spec into chunks and runs a vibe cycle per chunk. Skills dispatch subagents via the Agent tool; only `develop`-style skills do implementation work directly.

Each `SKILL.md` has YAML frontmatter: `name`, `description` (used for trigger matching), optional `allowed-tools`, `argument-hint`, `user-invocable`, `context`, `agent`, `model`. When editing a SKILL.md, follow `rules/skills-editing.md` — integrate into existing flow, don't append.

Use `${CLAUDE_SKILL_DIR}` in SKILL.md when referencing bundled scripts (it resolves to the skill's own directory).

## Rules (`rules/`)

Shared guidance loaded into sessions as system reminders — `cargo.md`, `rust.md`, `python.md`, `testing.md`, `bash-tools.md`, `blueprints.md`, `skills-editing.md`, `positive-framing.md`, `rtk.md`, `svelte5.md`, `arc-core-workflow.md`. Edit rules here rather than inlining guidance into individual skills.

## Hooks (`hooks/`)

Active hooks (see `settings.json`):

- `SessionStart` → `session_start_apply_patch.sh` — nudges the model to load the deferred `apply_patch` MCP tool on turn 1.
- `PreToolUse:Bash` → `rtk-rewrite.sh` — delegates to `rtk rewrite` to substitute token-saving equivalents for common Bash commands. All rewrite rules live in rtk's Rust registry, not in this shell wrapper.
- `post_tool_use_format.py` and `post_tool_use_generations.py` — auto-format hooks (not currently wired in `settings.json`; enable via hook config if needed).

## Local plugins (`local-plugins/`)

A local plugin marketplace (`local-plugins/.claude-plugin/marketplace.json`) with two plugins:

- `gt` — Graphite CLI wrapper for stacked PRs. Exposes `gt:gt`, `gt:submit`, `gt:restack` skills.
- `gh-stack` — gh-stack wrapper, same role without Graphite.

Only one is enabled at a time via `settings.json::enabledPlugins`.

## Blueprints (specs/plans/reviews/reports/docs)

Never write these as files in this repo. They live in the blueprints vault at `$CT_BLUEPRINTS_DIR` (default `~/blueprints/`), which is a separate git repo. Use `ct <type> create|read|list|latest|archive` or the `blueprint` MCP tools. See `rules/blueprints.md` for the full contract.

## Rust conventions

Edition 2024, latest stable toolchain, zero-warning policy (`cargo clippy -- -W clippy::all`). Flat dependency list in `Cargo.toml`; use the highest unambiguous version (`^3`, not `^3.0`). See `rules/rust.md` and `rules/cargo.md`.

## Patching files

Prefer `mcp__apply-patch__apply_patch` over Edit/Write. The global rule in `~/.claude/CLAUDE.md` spells out when to fall back — don't duplicate that reasoning here.
