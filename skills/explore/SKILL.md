---
name: explore
description: "Research, investigate, and design via subagent dispatch with auto-escalation for complex work. Triggers: 'explore', 'how does X work', 'understand', 'research', 'plan a feature', 'figure out', 'investigate', 'design', 'architect', 'best way to', 'state of the art', 'which lib/tool'. Also use when an implementation request contains an unresolved technology choice. Do NOT use when: the user wants to brainstorm design options for a greenfield feature — use /brainstorm instead."
argument-hint: "<prompt> [--continue]"
user-invocable: true
allowed-tools:
  - Task
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - Skill
  - AskUserQuestion
  - Bash
  - Write
---

# Explore

Research, investigate, design. Findings stored via plan-storage. Auto-escalates to team for complex multi-system work.

**IMMEDIATELY dispatch to subagent.** Never explore on main thread.

## Interviewing

See rules/skill-interviewing.md. Ask when: multiple viable approaches with unclear tradeoffs, or domain ambiguity needing clarification before deep-diving.

## New Exploration

1. TaskCreate: subject "Explore: <topic>", acceptance criteria (findings as Current State/Recommendation/phased Next Steps with file paths), metadata `{project: <repo root>, type: "explore", priority: "P2"}`. Then `TaskUpdate(taskId, status: "in_progress", owner: "explore")`.

2. Dispatch Task (subagent_type="codebase-researcher"):

```
Research <topic>. Return COMPLETE findings as text (do NOT write files or create tasks).

1. Explore codebase  2. Design approach — 2-3 options, choose best

## Output
1. **Current State**: files, patterns, architecture
2. **Recommendation**: chosen approach + rationale
3. **Key Files**: exact paths to modify/create
4. **Risks**: edge cases, failure modes
5. **Next Steps** — per phase:
   **Phase N: <title>**
   Files: exact/path/to/file.ts
   Approach: <what and why>
   1. <step>

Phases must include file paths + approach (downstream depends on it).

## Escalation
3+ independent subsystems, 3+ viable approaches with unclear tradeoffs,
or cross-cutting adversarial concerns → "ESCALATE: team — <reason>"
```

   **On "ESCALATE: team":** spawn 2-3 parallel Task agents (Researcher: breadth-first, Architect: design/tradeoffs, Devil's Advocate: challenges assumptions; all model: opus). Synthesize into unified findings.

3. **Validate** (subagent-trust.md): spot-check ALL architectural claims (system structure, component boundaries, data flow, integration points) + 50% of file/behavioral claims. Failed check or echo → targeted follow-up.

4. **Store findings:**
   1. `PLAN_FILE=$(echo "<findings>" | ck plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "explore" 2>/dev/null)` — warn if fails/empty.
   2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"}, description: "Explore: <topic> — findings in plan file and metadata.design")`

5. Output summary:
```
Explore: t<id> — <topic>
Problem: <1 sentence>
Recommendation: <1 sentence>
Phases:
1. <title> — <key files>
Key decisions: <why this approach>
Next: /prepare t<id>
```

6. Stop after summary — user reviews before proceeding, do not auto-invoke prepare.

## Continuation (--continue)

1. Resolve task: argument → task ID; bare `--continue` → TaskList filtered by `metadata.type === "explore"` + (`status_detail === "review"` or `status === "in_progress"`), first result
2. TaskGet → extract `metadata.design`. TaskUpdate to in_progress, clear status_detail
3. Dispatch subagent with previous findings as context + new prompt
4. Store via same plan-storage pattern (step 4 above). Always set `status_detail: "review"` on completion — continuations must be consistent with new explorations.
5. Output summary. Stop for user review.

## Key Rules

- Main thread does NOT explore — subagent does
- Findings via plan-storage (plan file + task metadata)
- Next Steps must include file paths — prepare depends on them
