---
name: scope
description: "Research an existing codebase and create phased implementation tasks with design context. Triggers: 'scope', 'research', 'investigate', 'design', 'architect', 'plan a feature', 'how does X work', 'figure out', 'best way to', 'state of the art', 'which lib/tool', 'create tasks from plan'. Also use when an implementation request contains an unresolved technology choice. Do NOT use when: the user wants to brainstorm design options for a greenfield feature — use /brainstorm instead."
argument-hint: "<prompt> [--continue] [--no-develop] [--auto-approve]"
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
  - Read
  - Glob
  - Grep
  - Write
  - TeamCreate
  - TeamDelete
  - SendMessage
---

# Scope

Research → spec → approve spec → plan → approve plan → develop. **Never research on main thread.**

Two-phase output: **spec** (what we're building) then **plan** (how we're building it). User approves each before proceeding.

## Interviewing

See rules/skill-interviewing.md.

## New Scope

!`test -n "$CLAUDE_CODE_TASK_LIST_ID" && echo "" || echo "⚠ CLAUDE_CODE_TASK_LIST_ID is not set — TaskCreate/TaskUpdate/TaskList/TaskGet will not work. Tell the user to set it in .claude/settings.json under env, then retry. Stop here."`

1. **Create tracking task:** TaskCreate: subject "Scope: \<topic\>", metadata `{project: <repo root>, type: "scope", priority: "P2"}`. TaskUpdate(taskId, status: "in_progress", owner: "scope").

2. **Research:** Dispatch Task (subagent_type="Explore"):

```
Research <topic>. Return findings as text (do NOT write files or create tasks).

## Output
1. **Current State**: per file — path, exports/defines, patterns
2. **Recommendation**: chosen approach + rationale
3. **Key Files**: exact paths to modify/create
4. **Risks**: edge cases, failure modes
5. **Suggested Phases** — per phase: title, file paths, approach, steps

## Escalation
3+ independent subsystems or 3+ viable approaches → "ESCALATE: team — <reason>"
```

   **On "ESCALATE: team":** TeamCreate, dispatch 3 agents (mode: "plan") — Researcher, Architect, Skeptic. Synthesize: Architect's approach + contradictions vs Skeptic. TeamDelete.

3. **Validate research:** spot-check ALL architectural claims. File/behavioral claims: check every odd-numbered claim (1st, 3rd, 5th...), minimum 3. Each check: Grep or Read a few lines to confirm existence — do NOT read entire files. Failed check → follow-up subagent.

### Spec Phase (what we're building)

4. **Synthesize spec** from validated research. The spec is a **timeless target-state document** — after implementation it should read as a valid specification of the system, not as a dated change request. Write it as if the system already works this way.

   - **Problem**: what's broken or missing — the only section that may describe current broken state. This is motivational context explaining WHY the spec exists.
   - **Recommendation**: the target behavior in present tense. Strategy-level — WHY this approach, not WHAT code to change. "Webhook delivery uses exponential backoff via BullMQ" not "Add exponential backoff to webhook delivery." Avoid transition verbs (add, replace, migrate, move, change X to Y) — those describe what to DO, not what the system IS.
   - **Architecture Context**: describe the relevant code landscape in present tense, as it will look post-implementation. Module roles, patterns, architectural layers — not hardcoded file paths. Describe components by what they do ("the webhook delivery module", "the job queue infrastructure"), not by path. If paths help orient the reader, include them parenthetically, but the description must stand without them. No Modify/Create annotations, no files-to-create, no change descriptions.
   - **Risks**: edge cases, failure modes, constraints

   The spec does NOT include: implementation phases, step-by-step approaches, task breakdowns, files to create/modify, or specific code changes (e.g., "add Codable conformance to X", "create Y.swift", "SyncChangeQueue.swift — add persistence backing"). Those belong to the plan.

5. **Store spec:**
   - `SPEC_FILE=$(echo "<spec content>" | ct spec create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "scope" 2>/dev/null)`
   - `TaskUpdate(taskId, metadata: {spec: "<spec content>", spec_file: "$SPEC_FILE" (omit if empty), status_detail: "spec_review"})`

6. **Present spec** — output as conversation text:
   - `Spec: t<id> — <topic>`
   - Problem statement
   - Recommendation + rationale
   - Architecture context (code landscape)
   - Risks and constraints

   If `--auto-approve` → skip to step 8.
   Otherwise → stop for user review.

7. **Spec refinement** — if user gives feedback:
   - **Minor (no new research needed):** Revise spec from stored research + feedback. TaskUpdate revised metadata.spec. If metadata.spec_file → overwrite it by writing to the existing path. Do NOT run `ct spec create` again — that generates a new file and orphans the reference in metadata.spec_file. status_detail stays `"spec_review"`.
   - **Major (user references unexplored code or new approach):** Dispatch follow-up research subagent with current spec as context. Merge findings. TaskUpdate merged spec. Overwrite spec_file if set.
   - Re-present. Repeat until user approves.
   - Always persist changes to metadata.spec before re-presenting.

8. **Approve spec:** `TaskUpdate(taskId, metadata: {status_detail: "spec_approved"})`.

### Plan Phase (how we're building it)

9. **Generate plan** from approved spec + research findings. The plan is the HOW — phased implementation approach:
   - Per phase: title, files (Read/Modify/Create), approach, steps
   - Dependencies between phases
   - Research Next Steps must include file paths — develop depends on them.

10. **Store plan:**
    1. If a previous metadata.plan_file exists from a prior scope run for this project, archive it first: `ct plan archive <old_plan_file> 2>/dev/null`
    2. `PLAN_FILE=$(echo "<plan content>" | ct plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "scope" 2>/dev/null)`
    3. `TaskUpdate(taskId, metadata: {design: "<plan content>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"})`

    The design field must be substantive — full phased breakdown with file paths, approaches. If a reader can't understand the plan from metadata.design alone, it's too sparse.

11. **Present plan** — output as conversation text:
    - `Plan: t<id> — <topic>`
    - Phased approach — per phase: title, files (Read/Modify/Create), approach
    - Dependencies
    - `Next: /develop t<id>`

    If `--auto-approve` → skip to step 13.
    Otherwise → stop for user review.

12. **Plan refinement** — if user gives feedback:
    - **Minor (no new files needed):** Revise from stored plan + feedback. TaskUpdate revised metadata.design.
      If metadata.plan_file → overwrite it by writing to the existing path. Do NOT run `ct plan create` again — that generates a new file and orphans the reference in metadata.plan_file.
    - **Major (new codebase data required):** If the user references unexplored code, asks to research something, or introduces a new architectural approach — dispatch a follow-up subagent with `metadata.design` as prior findings verbatim in the prompt. Merge new + prior. TaskUpdate merged design. Overwrite plan_file if set. When in doubt, dispatch.
    - **Spec affected?** If feedback changes WHAT we're building (scope, goals, key files, risks) — not just HOW — update metadata.spec too and overwrite spec_file if set. Approach-only changes leave the spec untouched.
    - Re-present. Repeat until user approves.
    - Always persist changes to metadata before re-presenting — develop reads stored artifacts, not conversation context. Stale artifacts = wrong plan.

13. **Approve plan and finalize:**
    - `TaskUpdate(taskId, metadata: {status_detail: "approved"})`.
    - **Spec-to-repo option:** AskUserQuestion — "Save spec as a file in the repo?" If yes: write spec content to `docs/specs/<slug>.md` (or project-appropriate path). The spec already exists in `$HOME/.claude/specs/` — this copies it into the project tree so it can be committed alongside implementation files.
    - If `--no-develop` → report scope task ID, stop.
    - Otherwise → `Skill("develop", "t<scopeTaskId>")`.

## Continuation (--continue)

Resolve task: argument → task ID; bare → TaskList `type === "scope"`, `status_detail` in `["spec_review", "review", "approved"]`, most recent. Extract relevant metadata.

- `status_detail === "approved"` → already approved. `Skill("develop", "t<taskId>")`.
- `status_detail === "spec_review"` → if metadata.spec_file is set, read content via `ct spec read <spec_file>`; otherwise use metadata.spec. Re-present spec. Resume from step 6.
- `status_detail === "review"` → dispatch subagent with `metadata.design` as prior findings verbatim: "Prior findings: \<metadata.design\>. New prompt: \<user prompt\>. Merge both into updated findings." TaskUpdate merged. If metadata.plan_file → overwrite existing path (do NOT run `ct plan create` again). Re-enter from step 10.

## Key Rules

- Main thread does NOT research — subagent does.
- Two-phase output: spec (what) THEN plan (how). Each has its own approval gate.
- Spec archival: `ct spec create`. Plan archival: `ct plan create`.
- Present findings as conversation text, not plan mode. Stop for user review at each gate.
- Scope does NOT create epic or tasks — develop handles that.
- metadata.spec = the spec (what). metadata.design = the plan (how). Separate fields.
- Research Suggested Phases must include file paths — plan depends on them, develop depends on plan.
- Refinement: minor → revise from findings; major → dispatch follow-up subagent.
- `--auto-approve` bypasses BOTH spec and plan review gates.
