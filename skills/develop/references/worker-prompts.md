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
2. Skill("implement-worker", args="<task-id>")

## Rules
- Never run git commands — orchestrator handles commits
- Never invoke Skill("commit") — orchestrator handles commits
- Fundamental design conflict → stop immediately, report "RESCOPE: <reason>" in output. Do not attempt workarounds.
- Task too large (3+ subsystems, unclear approach) → TaskCreate child tasks under current task, mark current task completed. Scheduler picks up children automatically.

## Exit Gate (mandatory)
Before reporting completion, run the project's build command (e.g., `just build`, `swift build`, `npm run build`, `cargo build` — check Justfile/Makefile/package.json for the right one). If the build fails, fix the errors before completing. A worker that reports "done" with compilation errors forces the orchestrator to do fixup work — this wastes tokens and breaks the orchestration model. Only report completion when the build passes.
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
- File conflict or blocker → SendMessage to team lead, wait
- Never run git commands
- Never invoke Skill("commit") — orchestrator handles commits
- Fundamental design conflict → stop, SendMessage "RESCOPE: <reason>" to team lead. Do not attempt workarounds.
- Task too large (3+ subsystems, unclear approach) → TaskCreate child tasks under current task, SendMessage "DECOMPOSED: <task-id> into N subtasks" to team lead. Mark current task completed.

## Exit Gate (mandatory)
Before reporting completion, run the project's build command (e.g., `just build`, `swift build`, `npm run build`, `cargo build` — check Justfile/Makefile/package.json for the right one). If the build fails, fix the errors before completing. A worker that reports "done" with compilation errors forces the orchestrator to do fixup work — this wastes tokens and breaks the orchestration model. Only report completion when the build passes.
```
