1. Never external actions without explicit request (PR comments, GitHub issues, Slack, email, Notion).
2. Questions are reflections to analyze, not disguised commands. Think critically and answer the question. Don't treat "do you think X needs Y?" as "do Y."
3. No dead code, commented-out code, "just in case" code. Delete old code completely — no deprecation, versioned names, migration code.
4. Comments for WHY / edge cases / surprising only. No docstrings unless project convention. No comments on code you didn't write.
5. Always delegate work to subagents or teams.
6. Subagent trust is adversarial by default. Spot-check claims (1-2 for small tasks; ALL architectural claims for epics). Echo detection: if a subagent confirms every assumption without surfacing tradeoffs or caveats, re-verify the claim most likely to have nuance. Build gate exemption: build/test-verified results skip spot-checks.
7. **Use the LSP tool first** for: go-to-definition, find-references, find-callers (incomingCalls), and type info. LSP resolves through re-exports, trait impls, and abstractions that Grep cannot follow. Only fall back to Grep when LSP is unavailable or for text-pattern searches (TODO, string literals). Tool precedence: LSP > Grep > Glob > `rg`/`fd` in Bash > `ck`. Never raw `grep`/`find` in Bash (hook-enforced).
8. Never `git checkout` to "restore" — make targeted edits. Ask before discarding uncommitted work.
9. Never drop, revert, or modify things you don't recognize (commits, files, branches, config). If something unexpected appears, **stop and ask** — it's the user's work.
10. When saving memories, consider if a universal rule would be more useful → `~/.claude/rules/<topic>.md`
11. Skills flow: brainstorm → spec → scope → develop [acceptance] → review → commit. Shortcuts: vibe (spec→scope→develop→review→commit), supervibe (spec→loop(vibe)→PRs)
12. On resume after compaction: if tasks exist with `metadata.impl_team` set and status `in_progress`, re-invoke `/develop` to trigger recovery.
13. Skill scripts: use `${CLAUDE_SKILL_DIR}` in SKILL.md to reference skill-local files (scripts, references, agents). Expands to the skill's absolute directory at load time.

@RTK.md
