# Worker Prompts

## Standalone Variant

Used by Solo mode and as TeamCreate fallback. No team messaging — worker completes and returns.

```
Implement task <task-id>.

## Task
<task description from TaskGet>

## Epic Context (if applicable)
<from task metadata.breadcrumb + metadata.epic_design>

## Protocol
1. TaskUpdate(taskId, status: "in_progress", owner: "solo")
2. Read every file in scope. Read 2-3 existing test files in the same module/directory to learn conventions (imports, framework, base classes, assertion patterns, naming, fixtures). Match their style. No nearby tests → use rules/test-quality.md defaults.
   Follow TDD: write failing tests, confirm red, implement until green. No test infra → note in report, implement directly.
3. Build + test. All green → continue.
   On failure: deduplicate errors (strip paths/line numbers). Same root error 2x → stop, report with context. 3 distinct errors → report all, stop.
4. Self-check: re-read changed files. Remove debug artifacts (console.log, print, debugger), low-value comments (code-restating, contextless TODOs), unused imports. Flatten nesting via early returns. Apply language-idiomatic patterns.
5. TaskUpdate(taskId, status: "completed", metadata: {completedAt: "<current ISO 8601 timestamp>"})

## Rules
- TDD: test first. Standards: rules/test-quality.md
- Never run git commands — orchestrator handles commits
- Never invoke Skill("commit") — orchestrator handles commits
- Only modify files in your task scope
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
- Task too large (3+ subsystems, unclear approach) → TaskCreate child tasks under current task, mark current task completed. Scheduler picks up children automatically.
- Fundamental design conflict (wrong approach, missing prerequisite, contradictory requirements) → stop immediately, report "RESCOPE: <reason>" in output. Do not attempt workarounds.
```

## Codex Conventions Component

Injected into every Codex dispatch prompt — not used by Claude workers.

```
{codex_conventions}

## Project Conventions (injected from $HOME/.claude)

### Code Style
- Clarity over brevity. No clever one-liners that obscure intent.
- No dead code, commented-out code, "just in case" code.
- Comments for WHY / edge cases / surprising behavior only.
- Three similar lines before abstracting.

### Testing (TDD required)
- Write failing test first, confirm red, then implement.
- Every test must answer: "What bug would this catch?"
- Banned: tautology mocks, getter/setter tests, implementation mirroring, coverage padding.
- Mock only: external services, network, filesystem, clock. Never mock what you own.

### File Structure
- Exact file paths from task description — do not create new files outside scope.
- One logical change per file modification.

### Naming
- Match surrounding code conventions (check 2-3 nearby files first).
- No versioned names (FooV2), no migration wrappers.
```

## Team-based Variant

Used by Team mode when TeamCreate succeeded. Adds team lead messaging and shutdown handshake.

```
Implement task <task-id>.

## Task
<task description from TaskGet>

## Epic Context
<from task metadata.breadcrumb + metadata.epic_design>

## Protocol
1. TaskUpdate(taskId, status: "in_progress", owner: "worker-<taskId>")
   If fails → someone else claimed it. Report and stop.
2. Skill("implement-worker", args="<task-id>")
3. SendMessage(type="message", recipient="<team-lead-name>",
     content="Completed <task-id>: <summary>",
     summary="Completed <task-id>")
4. Wait for shutdown request. Approve it.

## Rules
- TDD: test first. Standards: rules/test-quality.md
- Only modify files in your task scope
- File conflict or blocker → SendMessage to team lead, wait
- Never run git commands
- Never invoke Skill("commit") — orchestrator handles commits
- Bug found elsewhere → TaskCreate(subject: "Found: ...", metadata: {type: "bug", priority: "P2", project: "<repo root>"})
- Task too large (3+ subsystems, unclear approach) → TaskCreate child tasks under current task, SendMessage "DECOMPOSED: <task-id> into N subtasks" to team lead. Mark current task completed.
- Fundamental design conflict → stop, SendMessage "RESCOPE: <reason>" to team lead. Do not attempt workarounds.
```
