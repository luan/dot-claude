## Non-Negotiable

1. Never complete a review-status task without explicit user consent.
2. Never external actions without explicit request (PR comments, GitHub issues, Slack, email, Notion).

## Code Style

- Three similar lines > premature abstraction.
- Clarity over brevity. No clever one-liners that obscure intent, no over-combining concerns.
- No dead code, commented-out code, "just in case" code. Delete old code completely — no deprecation, versioned names, migration code.
- Comments for WHY / edge cases / surprising only. No docstrings unless project convention. No comments on code you didn't write.

## Efficiency

- Run parallel operations in single messages when possible
- Delegate work to subagents; main thread orchestrates
- Pre-compute summaries for subagent context rather than passing raw content

## Context Budget

- Monitor context usage carefully throughout sessions
- Pipe long command output through `tail`/`head` to limit volume
- Summarize large file contents rather than reading in full when a summary suffices
- When context is running low, prefer finishing current work over starting new tasks

## Safety

- Never `git checkout` to "restore" — make targeted edits. Ask before discarding uncommitted work.
- Never drop, revert, or modify things you don't recognize (commits, files, branches, config). If something unexpected appears, **stop and ask** — it's the user's work.
- `replace_all: true` only for simple renames. Never for config surgery.
- Don't close/delete PRs, issues, comments — update in place.
- Shared/visible systems: additive fixes > destructive.

## Memory

- Never auto-memory (`projects/*/memory/`). Not version-controlled.
- Universal → `~/.claude/rules/<topic>.md`; Project → `CLAUDE.md`
- After writing rule, remind user to commit dot-claude.

## Debugging

- Root cause first. Explain cause, get approval before fix.
- One bug at a time. Fix, verify, next. Never batch speculative fixes.
- Fix failed? Re-read runtime flow from interaction to break. Don't guess from static code.

## Testing

TDD default. Standards in `rules/test-quality.md`.

## Graphite & PR Workflow

- All branch operations go through `/gt`. Never raw `git rebase`, `git push`, `git branch -d`, `git checkout -b`.
- Push → `/gt:submit`. Restack → `/gt:restack`. Commit → `/commit`.
- Return `app.graphite.com/...` URLs, not GitHub.
- Review scope: diff vs stack parent (`gt log`), not trunk.

## Skill Flow

explore → prepare → implement → test-plan (manual QA) → commit
Optional: brainstorm (before explore), split-commit (repackage messy WIP), review (adversarial), refine (polish code).
After explore: `/prepare <id>`. After prepare: `/implement <epic-id>`.

## Branch Naming

`gt create luan/<description>` (e.g. `luan/fix-container-minimize`)

## Session End

- File remaining work as tasks. Run quality gates if code changed.
- Commit. Push only when user explicitly requests.

## Tasks

All plans, notes, state live in native Tasks. No filesystem documents. Lifecycle: pending → in_progress → review (metadata) → completed
