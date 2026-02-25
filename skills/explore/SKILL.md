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

2. Dispatch Task (subagent_type="Explore"):

```
Research <topic>. Return findings as text (do NOT write files or create tasks).

## Output
1. **Current State**: per file — path, exports/defines, patterns
2. **Recommendation**: chosen approach + rationale
3. **Key Files**: exact paths to modify/create
4. **Risks**: edge cases, failure modes
5. **Next Steps** — per phase: title, file paths, approach, steps

## Escalation
3+ independent subsystems or 3+ viable approaches → "ESCALATE: team — <reason>"
```

   **On "ESCALATE: team":**

   a. `TeamCreate(team_name="explore-<topic-slug>")`
   b. Dispatch 3 opus agents (mode: "plan"): **Researcher** (map subsystems, catalog interfaces), **Architect** (evaluate approaches, propose design), **Skeptic** (stress-test claims, find counter-evidence).
   c. Review each plan via ExitPlanMode → plan_approval_request. Approve or reject with feedback.
   d. Collect via SendMessage. Synthesize: unified output from Architect's approach + **Contradictions** quoting Architect vs Skeptic. Full agreement → "No contradictions found."
   e. `TeamDelete`.

3. **Validate**: spot-check ALL architectural claims + 50% of file/behavioral claims. Failed check → follow-up.

4. **Store findings:**
   1. `PLAN_FILE=$(echo "<findings>" | ct plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "explore" 2>/dev/null)`
   2. `TaskUpdate(taskId, metadata: {design: "<findings>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"})`

5. Output: `Explore: t<id> — <topic>`, problem, recommendation, phases with key files, `Next: /prepare t<id>`.

6. Stop — user reviews before proceeding.

7. **Design refinement:** If feedback changes the recommendation (not acknowledgment), revise on main thread:
   1. Rewrite recommendation, key files, phases with feedback incorporated.
   2. `TaskUpdate(taskId, metadata: {design: "<revised>", status_detail: "review"})`. If `metadata.plan_file`, overwrite it.
   3. Re-output summary. Stop for review. Repeat on further substantive feedback.
   4. `/prepare` reads stored artifacts, not conversation — stale artifacts = wrong plan.

   Feedback needs new codebase research → `--continue` instead.

## Continuation (--continue)

1. Resolve task: argument → task ID; bare `--continue` → TaskList `metadata.type === "explore"` + `status_detail === "review"`, first match.
2. TaskGet → extract `metadata.design`. TaskUpdate to in_progress, clear status_detail.
3. Dispatch subagent with prior findings + new prompt: "Merge prior + new into unified output. Flag newly covered files."
4. TaskUpdate with merged findings. If `metadata.plan_file`, overwrite. Do NOT create a new plan.
5. Output summary. Stop for review.

## Key Rules

- Main thread does NOT research the codebase — subagent does. Design revision (step 7) stays on main thread.
- Next Steps must include file paths — prepare depends on them
