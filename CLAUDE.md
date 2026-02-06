## Plan Mode

- Make the plan extremely concise. Sacrifice grammar for the sake of concision.
- At the end of each plan, give me a list of unresolved questions to answer, if any.

## Code Organization

**Keep functions small and focused:**

- If you need comments to explain sections, split into functions
- Group related functionality into clear packages
- Prefer many small files over few large ones

## Architecture Principles

- Delete old code completely - no deprecation needed
- No semantic prefix or suffix (OptimizedProcessor, FastHandler, ClientImpl)
- No versioned names (processV2, handleNew, ClientOld)
- No migration code unless explicitly requested
- No "removed code" comments - just delete it
- Don't add unnecessary comments. Only add doc comments if it's the style of the existing project.
- Comments that explain something subtle or surprising are OK.

## Maximize Efficiency

- **Subagents first:** Main thread orchestrates, subagents do heavy lifting. Don't explore/implement on main thread.
- **Parallel operations:** Run multiple searches, reads, and greps in single messages
- **Batch similar work:** Group related file edits together
- **Context is finite:** Truncate verbose output, summarize between agents. See `rules/context-budget.md`.

## Learning & Memory

- **Never use auto-memory** (`projects/*/memory/`). Not version-controlled, not reviewable.
- Universal learnings → `~/.claude/rules/<topic>.md` (in dot-claude repo)
- Project-specific learnings → that project's `CLAUDE.md`
- After writing a rule, remind user to commit in dot-claude.
- See `rules/memory.md` for full guidelines.

## Problem Solving

- **When stuck:** Stop. The simple solution is usually correct.
- **When uncertain:** Ask clarification via `AskUserQuestion`.
- **Before claiming done:** Run verification. Evidence before assertions — no "should work now."

## Testing Strategy

**TDD is the default.** No production code without a failing test first.

- Write test → watch it fail → minimal code to pass → refactor
- Bug fix? Write failing test that reproduces bug first
- "Already manually tested" or "too simple to test" are not valid reasons to skip TDD
- Every test must answer: "what bug would this catch?" No answer = delete it
- Banned: getter/setter tests, tautology tests, mock-returns-mock tests, happy-path-only
- Mocks: external services only. 3+ mocks = simplify the design
- Every comment must say something the code doesn't. No restating, no filler docstrings

**Exceptions (ask first):** throwaway prototypes, generated code, config-only changes.

## Workflow Skills

You have these skills via `Skill` tool. Use them—don't do this work on main thread.

| Invoke | When | Chemistry |
|--------|------|-----------|
| `Skill tool: explore` | Plan feature, investigate, research | WISP |
| `Skill tool: continue-explore` | Refine existing plan with feedback | Same epic |
| `Skill tool: implement` | Execute plan from explore | MOL |
| `Skill tool: feedback` | Quick fix to recent work | None |

**Flow:** explore → [continue-explore]* → plan mode → approval → implement → PR

## Agent Teams

Agent teams = multiple Claude instances that DISCUSS. Higher token cost.

| Invoke | When |
|--------|------|
| `Skill tool: team-explore` | Multi-angle exploration, architecture decisions |
| `Skill tool: team-implement` | Cross-layer or multi-module parallel implementation |
| `Skill tool: team-review` | Adversarial multi-lens review |
| `Skill tool: team-debug` | Competing hypothesis debugging |

**Decision:** Do agents benefit from talking to each other? Yes → team. No → subagent.
**Auto-escalation:** Each base skill triages and auto-escalates to its team variant when warranted.

**After plan approval** (user says "yes", "go ahead", "proceed"):
- **IMMEDIATELY** invoke implement skill with the epic-id
- Do NOT manually implement—skill handles subagent dispatch

## Branch Naming

Always use prefix `luan/` with short description: `gt create luan/<description>`

Examples:
- `luan/fix-container-minimize`
- `luan/add-theme-constants`
- `luan/refactor-drag-drop`

## Beads Commands

```bash
bd ready                    # Find next task (no blockers)
bd show <id>               # Read task instructions
bd update <id> --status in_progress
bd close <id>              # Complete task
bd lint <id>               # Validate issue quality (REQUIRED)
bd mol wisp <formula>      # Ephemeral workflow
bd mol pour <formula>      # Persistent workflow
```

**CRITICAL:** `bd lint` is NOT optional. Run on all issues before claiming plan complete.

Session state survives compaction via beads notes field.
