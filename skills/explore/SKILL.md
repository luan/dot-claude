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
  - TeamCreate
  - TeamDelete
  - SendMessage
---

# Explore

**IMMEDIATELY dispatch to subagent.** Never explore on main thread.

## Interviewing

See rules/skill-interviewing.md.

## New Exploration

1. TaskCreate: subject "Explore: <topic>", metadata `{project: <repo root>, type: "explore", priority: "P2"}`. TaskUpdate(taskId, status: "in_progress", owner: "explore").

2. Dispatch Task (subagent_type="codebase-researcher"):

```
Research <topic>. Return findings as text (do NOT write files or create tasks).

## Output
1. **Current State**: per file — path, what it exports/defines, patterns used (e.g. "src/auth.ts — exports verifyToken middleware, uses JWT RS256")
2. **Recommendation**: chosen approach + rationale
3. **Key Files**: exact paths to modify/create
4. **Risks**: edge cases, failure modes
5. **Next Steps** — per phase:
   **Phase N: <title>**
   Files: exact/path/to/file.ts
   Approach: <what and why>
   1. <step>

Phases must include file paths + approach.

## Escalation
3+ independent subsystems or 3+ viable approaches → "ESCALATE: team — <reason>"
```

   **On "ESCALATE: team":**

   a. `TeamCreate(team_name="explore-<topic-slug>")`
   b. Dispatch 3 agents (all model: opus, mode: "plan"):
      - **Researcher** — breadth-first investigation: map all relevant subsystems, surface patterns, catalog interfaces
      - **Architect** — design & tradeoffs: evaluate approaches, propose architecture, identify constraints
      - **Skeptic** — challenge assumptions: stress-test claims, find counter-evidence, identify risks others miss
   c. **Plan review:** each agent calls ExitPlanMode, sending a plan_approval_request. Review each plan — `SendMessage(type="plan_approval_response", approve: true/false)` with feedback if rejecting. Agents proceed only after approval.
   d. **Collect results:** agents send findings via SendMessage when done.
   e. **Synthesize:** merge all reports, produce unified output using Architect's approach. Add a **Contradictions** section that quotes specific claims from Architect vs Skeptic (e.g., "Architect: X — Skeptic: Y"). If agents fully agree, write "No contradictions found." Fold remaining caveats into Risks.
   f. `TeamDelete` — clean up team after synthesis.

3. **Validate**: spot-check ALL architectural claims + 50% of file/behavioral claims. Failed check → follow-up.

4. **Store findings:**
   1. `PLAN_FILE=$(echo "<findings>" | ct plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "explore" 2>/dev/null)`
   2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"})`

5. Output: `Explore: t<id> — <topic>`, problem, recommendation, phases with key files, `Next: /prepare t<id>`.

6. Stop — user reviews before proceeding.

7. **Design refinement:** If user feedback after step 6 changes the recommendation (new approach, different scope, architectural shift — not acknowledgment), revise on main thread without subagent dispatch:
   1. Incorporate feedback into revised findings (rewrite recommendation, key files, phases)
   2. `TaskUpdate(taskId, metadata: {design: "<revised>", status_detail: "review"})`
   3. If `metadata.plan_file`, overwrite plan file with revised findings
   4. Re-output revised summary. Stop for review.
   5. Repeat on further substantive feedback. `/prepare` reads stored artifacts, not conversation — stale artifacts in a fresh session = wrong plan.

   Use `--continue` instead when the feedback requires new codebase research (not just design-level redirection).

## Continuation (--continue)

1. Resolve task: argument → task ID; bare `--continue` → TaskList filtered by `metadata.type === "explore"` + `status_detail === "review"`, first result
2. TaskGet → extract `metadata.design`. TaskUpdate to in_progress, clear status_detail
3. Dispatch subagent with previous findings as context + new prompt. Instruct: "Compare against prior findings. Flag new files not previously covered. Produce a single unified output merging prior + new — no separated 'Old Findings' / 'New Findings' sections."
4. **Update existing task** — TaskUpdate(taskId, metadata: {design: "<merged findings>", status_detail: "review"}). If `metadata.plan_file`, overwrite plan file with merged findings. Do NOT ct plan create a new plan.
5. Output summary. Stop for user review.

## Key Rules

- Main thread does NOT research the codebase — subagent does. Design revision (step 7) stays on main thread.
- Next Steps must include file paths — prepare depends on them
