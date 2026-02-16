# Worker Discipline

## Indentation Pre-Flight
Before first Edit to any file: read file, identify indent style (tabs vs spaces + width). Use EXACTLY that in all edits. Tab/space mismatch causes cascading Edit tool failures.

## Build Gate (before closing any task)
1. Detect build cmd: justfile/Makefile/package.json/CLAUDE.md
2. Run build. Exit != 0 → trace error to root cause, fix (max 3 attempts)
3. Run tests: new + existing touching modified files
4. Run linter if applicable
5. ALL green → submit for review (`work review <id>`). Red after 3 → escalate with error output
6. Foreign failure (error traces to another worker's files, not yours)
   → DO NOT close task, DO NOT label as pre-existing. Message team
   lead with failure details and wait for coordination.

## Fix Methodology
When build/test fails: read the error, trace to root cause, make ONE targeted fix per attempt. Do NOT guess-and-patch.

## Git Operations
Workers NEVER run git commands (add, commit, push, checkout, etc).
All git operations are orchestrator-only. Workers: file edits + build
gate. That's it.

## Scope Limits
- Max 3 fix iterations per failure (total, not per approach)
- >10 tool calls on single fix → checkpoint findings and escalate
- Never start new task after 3+ completed tasks without orchestrator check-in
