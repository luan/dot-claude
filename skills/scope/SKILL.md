---
name: scope
description: "Research an existing codebase and create a phased implementation plan. Lighter than /spec — skips the formal target-state document and goes straight to 'what files to change and in what order.' Triggers: 'scope', 'research', 'investigate', 'plan a feature', 'how does X work', 'figure out', 'best way to', 'which lib/tool', 'create tasks from plan'. Also use when: an implementation request contains an unresolved technology choice, or a spec task needs a plan. Do NOT use when: the user wants to define WHAT to build (use /spec), wants to brainstorm design OPTIONS (use /brainstorm), or wants to just execute (use /vibe or /develop)."
argument-hint: "<prompt | t<spec-task-id>> [--continue] [--no-develop] [--auto]"
user-invocable: true
allowed-tools:
  - Agent
  - TaskCreate
  - TaskUpdate
  - TaskList
  - TaskGet
  - Skill
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

Research → plan → approve → develop. **Never research on main thread** — subagents do all codebase exploration.

Scope is the "how" — it produces a phased implementation plan with file paths and approaches. If a spec exists (from /spec or a parent task), scope consumes it as the goal. If not, scope does its own lightweight research.

## Interviewing

See rules/skill-interviewing.md.

## Input Resolution

Scope accepts three kinds of input:

- **Spec task** (`t<id>` where metadata.type === "spec"): Read metadata.spec as the goal. Research is warm-started — validate and fill gaps only.
- **Scope task** (`t<id>` where metadata.type === "scope"): Resume via `--continue` logic.
- **Raw prompt**: Research from scratch.

## Workflow

!`test -n "$CLAUDE_CODE_TASK_LIST_ID" && echo "" || echo "⚠ CLAUDE_CODE_TASK_LIST_ID is not set — TaskCreate/TaskUpdate/TaskList/TaskGet will not work. Tell the user to set it in .claude/settings.json under env, then retry. Stop here."`

### 1. Create tracking task

```
TaskCreate(
  subject: "Scope: <topic>",
  metadata: { project: <repo root>, type: "scope", priority: "P2" }
)
TaskUpdate(taskId, status: "in_progress", owner: "scope")
```

If consuming a spec task, also store: `metadata: { spec_task_id: "<spec task id>", spec: "<spec content from source task>" }`

### 2. Research

Dispatch subagent (subagent_type="Explore"):

```
Research <topic>. Return findings as text (do NOT write files or create tasks).

## Output
1. **Current State**: per file — path, exports/defines, patterns
2. **Key Files**: exact paths to modify/create
3. **Suggested Phases** — per phase: title, file paths, approach, steps

## Escalation
3+ independent subsystems or 3+ viable approaches → "ESCALATE: team — <reason>"
```

**Warm-start (spec or rich context in prompt):** When a spec or prior research exists, the subagent does targeted validation:
   - Validate referenced files exist and match described patterns
   - Check for recent changes (git log) that might invalidate context
   - Fill gaps (missing error paths, untouched subsystems)
   - Include the spec/context and say: "Prior research provided. Validate and fill gaps — do NOT re-explore what's already covered."

**On "ESCALATE: team":** TeamCreate, dispatch 3 agents (mode: "plan") — Researcher, Architect, Skeptic. Synthesize: Architect's approach + contradictions from Skeptic. TeamDelete.

### 3. Validate research

Spot-check ALL architectural claims. File/behavioral claims: check every odd-numbered claim (1st, 3rd, 5th...), minimum 3. Each check: Grep or Read a few lines. Failed check → follow-up subagent.

### 4. Generate plan

The plan is the HOW — phased implementation approach:
- Per phase: title, files (Read/Modify/Create), approach, steps
- Dependencies between phases
- File paths are required — develop depends on them

If a spec exists (metadata.spec), verify the plan covers every capability in the spec.

### 5. Codex review

**Skip if `--auto`.**

See [Codex Review](#codex-review). Prompt: coverage gaps, missing phases, dependency issues. If spec exists: "The spec is: \<metadata.spec\>. Does this plan fully implement it?"

High-severity → revise. Best-effort — if codex fails, proceed.

### 6. Store plan

1. If a previous metadata.plan_file exists, archive: `ct plan archive <old_plan_file> 2>/dev/null`
2. `PLAN_FILE=$(echo "<plan>" | ct plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "scope" 2>/dev/null)`
3. `TaskUpdate(taskId, metadata: { design: "<plan>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review" })`

metadata.design must be self-contained — full phased breakdown with file paths. Develop reads this without conversation context.

### 7. Present and approve

```
TaskUpdate(taskId, metadata: { status_detail: "approved" })
```

If `--auto` → skip to step 9. No plan output — caller reads `metadata.design`.

Output (interactive only):
```
Plan: t<id> — <topic>
<phased approach — per phase: title, files, approach>
<dependencies>
Next: /develop t<id>
```

Stop for user review.

### 8. Refinement

If user gives feedback:
- **Minor:** Revise from stored plan + feedback. TaskUpdate metadata.design. Overwrite plan_file if set.
- **Major (new codebase data):** Dispatch follow-up subagent with metadata.design as context. Merge new + prior. TaskUpdate. Overwrite plan_file.
- Persist metadata before re-presenting.
- Re-set `status_detail: "approved"` before re-presenting.

### 9. Finalize

- If `--no-develop` → return. Caller reads `metadata.design`.
- Otherwise → `Skill("develop", "t<scopeTaskId>")`.

## Codex Review

Adversarial review using OpenAI Codex. Read-only codebase access.

```bash
echo "<content>" | codex exec \
  --sandbox read-only --ephemeral \
  -C "$(git rev-parse --show-toplevel)" \
  -o /tmp/codex-review-output.txt \
  "Review prompt here. Content follows on stdin." -
```

Timeout: 120s. Failure → log and proceed (best-effort, never blocks).

## Resume (`--continue`)

Resolve task: argument → task ID; bare → TaskList `type === "scope"`, `status_detail` in `["review", "approved"]`, most recent.

- `status_detail === "approved"` → already approved. `Skill("develop", "t<taskId>")`.
- `status_detail === "review"` → dispatch subagent with metadata.design as prior findings. Merge. Re-enter from step 6.

## Key Rules

- Main thread does NOT research — subagents do.
- Scope is the "how" — phased plan with file paths. No spec production (that's /spec).
- If consuming a spec, verify plan covers every spec capability.
- Codex review is best-effort, never blocks.
- Scope does NOT create epic or tasks — develop handles that.
- metadata.design = plan. metadata.spec = inherited from spec task (if any).
- File paths in every phase — develop depends on them.
- `--auto` bypasses approval gate, codex review, AND all text output. Caller reads task metadata.
