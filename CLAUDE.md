## Non-Negotiable

1. **Never implement on main thread.** Plan approval → `Skill tool: implement` with epic-id. Never raw `Task` calls.
2. **Never explore on main thread.** `Skill tool: explore`. Subagents investigate, main orchestrates.
3. **Never Task tool directly.** Skills dispatch. Task tool INSIDE skills only.

## Plan Mode
- Extremely concise. Sacrifice grammar.
- End with unresolved questions if any.

## Code Organization
- Sections needing comments → split into functions
- Related functionality → clear packages
- Many small files > few large

## Architecture
- Delete old code completely (no deprecation)
- No semantic prefix/suffix (OptimizedProcessor, FastHandler, ClientImpl)
- No versioned names (processV2, handleNew, ClientOld)
- No migration code unless requested
- No "removed code" comments
- Doc comments only if project style. Comments for subtle/surprising only.

## Efficiency
- **Subagents first:** Main orchestrates, subagents work
- **Parallel ops:** Multiple searches/reads/greps per message
- **Batch:** Group related file edits
- **Context finite:** Truncate verbose output, summarize between agents. See `rules/context-budget.md`.
- **Tickets:** 1-2 on main fine. 3+ → subagent (`bd create --file` for bulk).

## Memory
- **Never auto-memory** (`projects/*/memory/`). Not version-controlled.
- Universal → `~/.claude/rules/<topic>.md` (dot-claude repo)
- Project → `CLAUDE.md`
- After writing rule, remind user to commit dot-claude.
- See `rules/memory.md`.

## Problem Solving
- **Stuck:** Stop. Simple solution usually correct.
- **Uncertain:** `AskUserQuestion`
- **Before done:** Verify. Evidence before assertions.

## Debugging
- **Root cause first.** Explain cause, get approval before fix.
- **One bug at a time.** Fix, verify, next. Never batch speculative fixes.
- **Fix failed?** Re-read runtime flow from interaction to break. Don't guess from static code.
- **Indentation:** Match file style. Hooks auto-format; without hooks, read first.

## Testing
**TDD default.** No production code without failing test.

- test → fail → minimal code → pass → refactor
- Bug fix? Failing test first.
- Every test answers: "what bug caught?" No answer → delete.
- **Banned:** getter/setter, tautology, mock-returns-mock, happy-path-only
- **Mocks:** external services only. 3+ → simplify design.
- No restatement/filler docstrings.

**Exceptions (ask first):** prototypes, generated code, config-only.

## Workflow Skills
Via `Skill` tool. Not on main thread.

| Invoke | When | Chemistry |
|--------|------|-----------|
| `explore` | Plan, investigate, research | WISP |
| `continue-explore` | Refine plan w/ feedback | Same epic |
| `implement` | Execute plan | MOL |
| `refine` | Polish post-implementation | None |
| `review` | Adversarial review | None |
| `feedback` | Quick fix recent work | None |

**Flow:** explore → [continue-explore]* → approval → implement → refine → review
**Never auto-commit/auto-PR.** User explicitly requests.

## Agent Teams
Multiple Claude instances DISCUSS. Higher cost.

| Invoke | When |
|--------|------|
| `team-explore` | Multi-angle exploration, architecture |
| `team-implement` | Cross-layer/multi-module parallel |
| `team-review` | Adversarial multi-lens review |
| `team-debug` | Competing hypothesis debugging |

**Decision:** Agents benefit from talking? Yes → team. No → subagent.
**Auto-escalation:** Base skill auto-escalates to team when warranted.

**After plan approval** ("yes", "go ahead", "proceed"):
- **IMMEDIATELY** `Skill tool: implement` with epic-id and STOP
- No Task tool directly
- No main thread implementation

## Subagent Rules
- **Self-healing:** Iterate until build passes before done. No partial work.
- **Check git log** before starting — avoid duplicate work.
- **Main verifies** full build after all workers complete.
- **Respect deps:** Never start blocked tasks. `bd ready` for unblocked.
- **TDD first:** Failing test before implementation, even in teams.
- **No file collisions:** Coordinate ownership. Two workers never edit same file.
- **Pre-existing failures:** Not caused by your changes → report once, continue.
- See `rules/worker-protocol.md` for full coordination protocol.

## Branch Naming
`gt create luan/<description>`
Examples: `luan/fix-container-minimize`, `luan/add-theme-constants`

## Beads Commands
```bash
bd ready                    # Next task (no blockers)
bd show <id>               # Read instructions
bd update <id> --claim     # Atomic: assignee + in_progress (race-safe)
bd close <id>              # Complete
bd create "Found: ..." --type bug --validate --deps discovered-from:<parent-id>
bd mol wisp <formula>      # Ephemeral workflow
bd mol pour <formula>      # Persistent workflow
bd swarm validate <epic-id>    # Pre-flight: parallelism, cycles, ready fronts
bd swarm status <epic-id>      # Progress: Completed/Active/Ready/Blocked
bd merge-slot acquire --wait  # Block until slot free
bd merge-slot release         # Release after git ops
```
**Lint:** Hook auto-lints after `bd create`. Only run `bd lint` manually as final plan validation (catches issues changed via `bd update`).
Session state survives compaction via beads notes.
