## Non-Negotiable Rules

1. **Never implement on main thread.** After plan approval → `Skill tool: implement` with epic-id. Never raw `Task` calls.
2. **Never explore on main thread.** Use `Skill tool: explore`. Subagents investigate, main thread orchestrates.
3. **Never use Task tool directly for implementation.** Skills handle dispatch. Task tool is for use INSIDE skills only.

## Plan Mode
- Extremely concise. Sacrifice grammar.
- End with unresolved questions list if any.

## Code Organization
- Comments to explain sections → split into functions
- Group related functionality → clear packages
- Many small files > few large files

## Architecture Principles
- Delete old code completely (no deprecation)
- No semantic prefix/suffix (OptimizedProcessor, FastHandler, ClientImpl)
- No versioned names (processV2, handleNew, ClientOld)
- No migration code unless requested
- No "removed code" comments
- No unnecessary comments. Doc comments only if project style.
- Comments for subtle/surprising only.

## Maximize Efficiency
- **Subagents first:** Main orchestrates, subagents work. No explore/implement on main.
- **Parallel ops:** Multiple searches, reads, greps in single message
- **Batch work:** Group related file edits
- **Context finite:** Truncate verbose output, summarize between agents. See `rules/context-budget.md`.

## Learning & Memory
- **Never use auto-memory** (`projects/*/memory/`). Not version-controlled.
- Universal → `~/.claude/rules/<topic>.md` (dot-claude repo)
- Project-specific → `CLAUDE.md`
- After writing rule, remind user to commit in dot-claude.
- See `rules/memory.md`.

## Problem Solving
- **Stuck:** Stop. Simple solution usually correct.
- **Uncertain:** `AskUserQuestion`
- **Before done:** Run verification. Evidence before assertions.

## Testing Strategy
**TDD default.** No production code without failing test first.

- Write test → fail → minimal code → pass → refactor
- Bug fix? Failing test first.
- No "manually tested"/"too simple to test" excuses.
- Every test answers: "what bug caught?" No answer → delete.
- **Banned:** getter/setter, tautology, mock-returns-mock, happy-path-only
- **Mocks:** external services only. 3+ mocks → simplify design.
- Comments say something code doesn't. No restatement/filler docstrings.

**Exceptions (ask first):** throwaway prototypes, generated code, config-only.

## Workflow Skills
Via `Skill` tool. Not on main thread.

| Invoke | When | Chemistry |
|--------|------|-----------|
| `explore` | Plan feature, investigate, research | WISP |
| `continue-explore` | Refine plan with feedback | Same epic |
| `implement` | Execute plan | MOL |
| `feedback` | Quick fix recent work | None |

**Flow:** explore → [continue-explore]* → plan mode → approval → implement → PR

## Agent Teams
Multiple Claude instances DISCUSS. Higher token cost.

| Invoke | When |
|--------|------|
| `team-explore` | Multi-angle exploration, architecture |
| `team-implement` | Cross-layer/multi-module parallel |
| `team-review` | Adversarial multi-lens review |
| `team-debug` | Competing hypothesis debugging |

**Decision:** Agents benefit from talking? Yes → team. No → subagent.
**Auto-escalation:** Base skill triages + auto-escalates to team variant when warranted.

**After plan approval** ("yes", "go ahead", "proceed"):
- **IMMEDIATELY** invoke `Skill tool: implement` with epic-id and STOP
- No Task tool directly—skill handles all dispatch
- No main thread implementation under any circumstances

## Branch Naming
Prefix `luan/` + short description: `gt create luan/<description>`
Examples: `luan/fix-container-minimize`, `luan/add-theme-constants`

## Beads Commands
```bash
bd ready                    # Next task (no blockers)
bd show <id>               # Read instructions
bd update <id> --status in_progress
bd close <id>              # Complete
bd lint <id>               # Validate (REQUIRED)
bd mol wisp <formula>      # Ephemeral workflow
bd mol pour <formula>      # Persistent workflow
```
**CRITICAL:** `bd lint` NOT optional. Run on all issues before plan complete.
Session state survives compaction via beads notes.
