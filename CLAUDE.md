1. External actions (PR comments, issues, Slack, email, Notion) need explicit request.
2. Questions are reflections — analyze, don't execute.
3. Delete dead code completely. No commented-out code, shims, or "just in case."
4. Comments for WHY / edge cases / surprises only. No docstrings unless convention. Don't comment code you didn't write.
5. Subagent trust is adversarial. Spot-check: 1-2 claims for small tasks, ALL architectural claims for epics. If a subagent confirms everything without tradeoffs, re-verify the likeliest nuanced claim. Build/test-verified results skip checks.
6. Tool precedence: LSP > Grep > Glob > `rg`/`fd` in Bash.
7. Restore files with targeted edits. Confirm before discarding uncommitted work.
8. Unrecognized artifacts are user work — ask before modifying.
9. Skills: brainstorm → spec → develop → review → commit. Shortcut: vibe.
10. Fix exactly what was asked. No scope creep. Fix it or push back with a reason.
11. All tests pass before committing. You own every failure you can see.
12. Skill scripts: use `${CLAUDE_SKILL_DIR}` in SKILL.md.
13. **Edit files with `apply_patch`.** Default for every file change — single-line tweaks, single-file edits, multi-file edits, renames, creates, deletes. Do not reach for Edit or Write because "it's just one line" or "it's just one file"; that is the exact failure mode to avoid. Use Edit/Write only when `apply_patch` genuinely cannot express the change (binary files, or apply_patch unavailable) — and say why in the same turn.
