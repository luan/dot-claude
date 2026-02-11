## Non-Negotiable

1. **Never implement on main thread.** `Skill tool: implement` with epic-id. Never raw `Task` calls.
2. **Never explore on main thread.** `Skill tool: explore`. Subagents investigate, main orchestrates.
3. **Never Task tool directly.** Skills dispatch. Task tool INSIDE skills only.

## Architecture
- Delete old code completely (no deprecation)
- No semantic prefix/suffix (OptimizedProcessor, FastHandler, ClientImpl)
- No versioned names (processV2, handleNew, ClientOld)
- No migration code unless requested
- No "removed code" comments
- Doc comments only if project style. Comments for subtle/surprising only.

## Communication
- Be direct. Skip preamble and summaries unless asked.
- Prefer bullet points over prose. Omit filler words.
- Wrap prose at 80 chars. Don't wrap code, URLs, headings, tables.

## Efficiency
- **Subagents first:** Main orchestrates, subagents work
- **Parallel ops:** Multiple searches/reads/greps per message
- **Batch:** Group related file edits
- **Context finite:** Pipe `| tail -20`, use `--quiet` flags,
  summarize between agents. >30 lines → summarize.
  Context running low → finish current work, don't start new.
- **Tickets:** 1-2 on main fine. 3+ → subagent (`bd create --file` for bulk).

## Memory
- **Never auto-memory** (`projects/*/memory/`). Not version-controlled.
- Universal → `~/.claude/rules/<topic>.md` (dot-claude repo)
- Project → `CLAUDE.md`
- After writing rule, remind user to commit dot-claude.

## Debugging
- **Root cause first.** Explain cause, get approval before fix.
- **One bug at a time.** Fix, verify, next. Never batch speculative fixes.
- **Fix failed?** Re-read runtime flow from interaction to break. Don't guess from static code.
- **Indentation:** Match file style. Hooks auto-format; without hooks, read first.

## Testing
**TDD default.** test → fail → minimal code → pass → refactor.
- Every test answers: "what bug caught?" No answer → delete.
- Banned: tautology, getter/setter, implementation mirroring,
  happy-path-only, coverage padding
- Mocks: external services only. 3+ → simplify design.
- Exceptions (ask first): prototypes, generated code, config-only.

## Workflow Skills
Via `Skill` tool. Not on main thread.

| Invoke | When |
|--------|------|
| `explore` | Research, investigate (findings → beads design field) |
| `implement` | Execute plan (auto-detects solo vs swarm) |
| `review` | Adversarial review (--team for 3-perspective) |
| `start` | Create branch + link beads issue |
| `resume-work` | Context recovery after break |
| `fix` | Convert feedback → beads issues |
| `prepare` | Plan → epic + phased tasks |
| `commit` | Conventional commit + bd sync |
| `refine` | Polish post-implementation |
| `debugging` | Systematic bug diagnosis |

**Flow:** explore (research) → prepare (tasks) → implement → review → commit
**Never auto-commit/auto-PR.** User explicitly requests.

**After explore:** review findings, then `/prepare <id>`.
**After prepare:** review tasks, then `/implement <epic-id>`.

## Subagent Rules
- **Self-healing:** Iterate until build passes. No partial work.
- **Claim atomically:** `bd update <id> --claim` (not status + assignee)
- **File ownership:** Never edit files outside your task scope.
  Need change in another worker's file → message the owner.
- **Build failures:** Yours → fix. Another worker's → report, continue.
  Pre-existing → report once, continue.
- **Completion:** No `bd close` without build + tests + linter passing.
- **Escalation:** 2 failed attempts → message lead with details.

## Branch Naming
`gt create luan/<description>`
Examples: `luan/fix-container-minimize`, `luan/add-theme-constants`

## Beads — Single Source of Truth

All plans, notes, and state live in beads. No filesystem documents.

- **Exploration plans**: `bd edit <id> --design`
- **Review findings**: `bd edit <id> --design`
- **Session notes**: `bd edit <id> --notes`
- **Task state**: `bd update <id> --status`
- **Sync**: `bd sync` after commit, submit, gt ops
