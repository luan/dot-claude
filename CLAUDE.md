1. External actions (PR comments, GitHub issues, Slack, email, Notion) require explicit user request.
2. Questions are reflections to analyze, not disguised commands. Think critically and answer the question.
3. Delete dead code completely — no commented-out code, deprecation shims, versioned names, or "just in case" code.
4. Comments for WHY / edge cases / surprising only. No docstrings unless project convention. No comments on code you didn't write.
5. Subagent trust is adversarial by default. Spot-check claims (1-2 for small tasks; ALL architectural claims for epics). Echo detection: if a subagent confirms every assumption without surfacing tradeoffs, re-verify the claim most likely to have nuance. Build/test-verified results skip spot-checks.
6. **Use the LSP tool first** for go-to-definition, find-references, find-callers, and type info. Fall back to Grep for text-pattern searches. Tool precedence: LSP > Grep > Glob > `rg`/`fd` in Bash.
7. Restore files with targeted edits. Confirm before discarding uncommitted work.
8. Unrecognized artifacts (commits, files, branches, config) are the user's work — stop and ask before modifying.
9. Skills flow: brainstorm → spec → develop → review → commit. Shortcut: vibe (spec→develop→review→commit).
10. Fix exactly what was asked — no scope creep, no deferring. Either fix it or push back with a specific reason.
11. When tests fail, investigate. Verify via `git stash` whether the failure predates your changes.
12. Skill scripts: use `${CLAUDE_SKILL_DIR}` in SKILL.md to reference skill-local files.

@RTK.md
