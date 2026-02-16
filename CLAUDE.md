## Non-Negotiable

1. **Never implement on main.** `Skill tool: implement` with epic-id. Never raw `Task`.
2. **Never explore on main.** `Skill tool: explore`. Subagents investigate, main orchestrates.
3. **Never Task tool directly.** Skills dispatch. Task tool INSIDE skills only.

## Architecture
- Delete old code completely (no deprecation)
- No semantic prefix/suffix (OptimizedProcessor, FastHandler, ClientImpl)
- No versioned names (processV2, handleNew, ClientOld)
- No migration code unless requested
- No "removed code" comments
- Doc comments only if project style. Comments for subtle/surprising only.

## Communication
- Direct. Skip preamble + summaries unless asked.
- Prefer bullets over prose. Omit filler.
- Wrap prose at 80 chars. Don't wrap code, URLs, headings, tables.

## Efficiency
- **Subagents first:** Main orchestrates, subagents work
- **Parallel ops:** Multiple searches/reads/greps per message
- **Batch:** Group related file edits
- **Context finite:** Pipe `| tail -20`, use `--quiet`, summarize between agents. >30 lines → summarize. Context low → finish current, don't start new.
- **Tickets:** 1-2 on main fine. 3+ → subagent (loop `work create` for bulk).

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
- Banned: tautology, getter/setter, implementation mirroring, happy-path-only, coverage padding
- Mocks: external services only. 3+ → simplify design.
- Exceptions (ask first): prototypes, generated code, config-only.

## Workflow Skills
Via `Skill` tool. Not on main.

| Invoke | When |
|--------|------|
| `brainstorm` | Greenfield design via interactive dialogue |
| `explore` | Research existing systems (findings → issue description) |
| `implement` | Execute plan (auto-detects solo vs swarm) |
| `review` | Adversarial review (--team for 3-perspective) |
| `start` | Create branch + link work issue |
| `resume-work` | Context recovery after break |
| `fix` | Convert feedback → work issues |
| `prepare` | Plan → epic + phased tasks |
| `commit` | Conventional commit |
| `split-commit` | Repackage WIP into vertical commits |
| `refine` | Polish after review passes |
| `debugging` | Systematic bug diagnosis |

**Flow:** brainstorm|explore → prepare → implement → split-commit → review → refine → commit
**Never auto-commit/auto-PR.** User explicitly requests.

**Note:** Review includes internal fix loop until clean.

**After explore:** review findings, then `/prepare <id>`.
**After prepare:** review tasks, then `/implement <epic-id>`.

## Subagent Rules
- **Self-healing:** Iterate until build passes. No partial work.
- **Claim tasks:** `work start <id>` + `work edit <id> --assignee <name>`
- **Git operations:** Workers never run git commands. Orchestrator commits.
- **File ownership:** Never edit files outside task scope. Need change in another worker's file → message owner.
- **Build failures:** Yours → fix. Another worker's → message lead, wait. Pre-existing → report once, continue.
- **Completion:** No `work review` without build + tests passing.
- **Escalation:** 2 failed attempts → message lead with details.

## Branch Naming
`gt create luan/<description>`
Examples: `luan/fix-container-minimize`, `luan/add-theme-constants`

## Work Issues — Single Source of Truth

All plans, notes, state live in work issues. No filesystem documents.

- **Exploration plans**: `work edit <id>` (stored in description)
- **Review findings**: `work edit <id> --description`
- **Session notes**: `work comment <id> "note text"`
- **Task state**: `work status <id> <state>`
- **Lifecycle**: open → active → review → done / cancelled
