---
name: scope
description: "Research an existing codebase and create phased implementation tasks with design context. Triggers: 'scope', 'research', 'investigate', 'design', 'architect', 'plan a feature', 'how does X work', 'figure out', 'best way to', 'state of the art', 'which lib/tool', 'create tasks from plan'. Also use when an implementation request contains an unresolved technology choice. Do NOT use when: the user wants to brainstorm design options for a greenfield feature — use /brainstorm instead."
argument-hint: "<prompt> [--continue] [--no-develop] [--auto]"
user-invocable: true
allowed-tools:
  - Task
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

Research → spec → approve → plan → approve → develop. **Never research on main thread** (subagents do all codebase exploration so the main thread stays responsive).

Two-phase output: **spec** (what) then **plan** (how). Each gets user approval before proceeding.

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

**Warm-start (rich context in prompt):** When the prompt already contains detailed research context — file locations, architecture patterns, prior findings, phase plans (e.g., from supervibe/superscope) — the research subagent does **targeted validation** instead of broad exploration:
   - Validate that referenced files exist and match described exports/patterns
   - Check for recent changes (git log) that might invalidate the context
   - Fill gaps the context doesn't cover (e.g., missing error paths, untouched subsystems)
   - Skip broad codebase exploration — the context already provides it
   - The subagent prompt should include the provided context and say: "Prior research is provided below. Validate and fill gaps — do NOT re-explore what's already covered."

**On "ESCALATE: team":** TeamCreate, dispatch 3 agents (mode: "plan") — Researcher, Architect, Skeptic. Synthesize: Architect's approach + contradictions vs Skeptic. TeamDelete.

3. **Validate research:** spot-check ALL architectural claims (wrong architecture = wrong plan). File/behavioral claims: check every odd-numbered claim (1st, 3rd, 5th...), minimum 3. Each check: Grep or Read a few lines — do NOT read entire files. Failed check → follow-up subagent.

### Spec Phase (what we're building)

4. **Synthesize spec** from validated research. The spec is a **timeless target-state document** — it describes the system as if already built. After implementation, it should still read as a valid specification (not a dated change request).
   - **Problem**: what's broken or missing (the only section that may describe current state).
   - **Recommendation**: target behavior in present tense, strategy-level. "Webhook delivery uses exponential backoff via BullMQ" — not "Add exponential backoff." No transition verbs (add, replace, migrate, move, change) because those describe actions, not the system.
   - **Architecture Context**: the code landscape post-implementation. Describe by module role and pattern, not file path. Paths may appear parenthetically but the description stands without them. No Create/Modify annotations.
   - **Risks**: edge cases, failure modes, constraints

   The spec excludes implementation details: phases, task breakdowns, files to create/modify, specific code changes. Those belong to the plan.

5. **Codex review (spec):** See [Codex Review](#codex-review). Prompt: gaps, ambiguities, edge cases, feasibility. "This is a WHAT document — do NOT suggest implementation details." High-severity actionable issues → revise. Best-effort — if codex fails, proceed. **Skip if `--auto`** — codex review is best-effort and adds latency; when scope runs as an inner call (vibe/supervibe), speed matters more.

6. **Store spec:**
   - `SPEC_FILE=$(echo "<spec content>" | ct spec create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "scope" 2>/dev/null)`
   - `TaskUpdate(taskId, metadata: {spec: "<spec content>", spec_file: "$SPEC_FILE" (omit if empty), status_detail: "spec_review"})`

7. **Present spec** — `Spec: t<id> — <topic>`, then Problem, Recommendation, Architecture Context, Risks. If `--auto` → skip to step 9. Otherwise → stop for user review.

8. **Spec refinement** — if user gives feedback:
   - **Challenge check:** Does this feedback contradict the rationale in the current spec without citing new evidence? If yes, name the contradiction and ask whether the rationale should be revised. Do not silently absorb it.
   - **Minor (no new research needed):** Revise from stored research + feedback. TaskUpdate revised metadata.spec. If metadata.spec_file → overwrite existing path (do NOT `ct spec create` again — that orphans the reference). status_detail stays `"spec_review"`.
   - **Major (unexplored code or new approach):** Dispatch follow-up subagent with current spec as context. Merge findings. TaskUpdate. Overwrite spec_file if set.
   - Persist metadata.spec before re-presenting (develop reads stored artifacts, not conversation).

9. **Approve spec:** `TaskUpdate(taskId, metadata: {status_detail: "spec_approved"})`.

### Plan Phase (how we're building it)

10. **Generate plan** from approved spec + research findings. The plan is the HOW — phased implementation approach:
    - Per phase: title, files (Read/Modify/Create), approach, steps
    - Dependencies between phases
    - Research Next Steps must include file paths — develop depends on them.

11. **Codex review (plan):** See [Codex Review](#codex-review). Prompt: spec coverage gaps, missing phases, dependency issues. "The approved spec is: \<metadata.spec\>. Does this plan fully implement it?" High-severity → revise. Best-effort. **Skip if `--auto`** — same rationale as step 5.

12. **Store plan:**
    1. If a previous metadata.plan_file exists from a prior scope run for this project, archive it first: `ct plan archive <old_plan_file> 2>/dev/null`
    2. `PLAN_FILE=$(echo "<plan content>" | ct plan create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "scope" 2>/dev/null)`
    3. `TaskUpdate(taskId, metadata: {design: "<plan content>", plan_file: "$PLAN_FILE" (omit if empty), status_detail: "review"})`

    metadata.design must be self-contained — full phased breakdown with file paths, approaches. Develop reads this without conversation context.

13. **Present plan and set approved** — the user may run `/develop` in a fresh session with no conversation context, so `status_detail` must be `"approved"` before presenting. Set it atomically with the presentation:
    - `TaskUpdate(taskId, metadata: {status_detail: "approved"})`
    - Output as conversation text:
      - `Plan: t<id> — <topic>`
      - Phased approach — per phase: title, files (Read/Modify/Create), approach
      - Dependencies
      - `Next: /develop t<id>`

    If `--auto` → skip to step 15.
    Otherwise → stop for user review.

14. **Plan refinement** — if user gives feedback:
    - **Challenge check:** Does this feedback contradict the rationale in the current plan without citing new evidence? If yes, name the contradiction and ask whether the rationale should be revised. Do not silently absorb it.
    - **Minor:** Revise from stored plan + feedback. TaskUpdate metadata.design. Overwrite plan_file if set (do NOT `ct plan create` again — orphans reference).
    - **Major (new codebase data):** Dispatch follow-up subagent with `metadata.design` as prior findings. Merge new + prior. TaskUpdate. Overwrite plan_file. When in doubt, dispatch.
    - **Spec affected?** If feedback changes WHAT (scope, goals, risks) — not just HOW — update metadata.spec too. Approach-only changes leave spec untouched.
    - Persist metadata before re-presenting (develop reads stored artifacts, not conversation).
    - After revision, re-set `TaskUpdate(taskId, metadata: {status_detail: "approved"})` before re-presenting.

15. **Finalize:**
    - **Spec-to-repo option:** `--auto` → skip (don't save to repo). Without `--auto` → AskUserQuestion — "Save spec as a file in the repo?" If yes: write spec content to `docs/specs/<slug>.md` (or project-appropriate path). The spec already exists in `$HOME/.claude/specs/` — this copies it into the project tree so it can be committed alongside implementation files.
    - If `--no-develop` → report scope task ID, stop.
    - Otherwise → `Skill("develop", "t<scopeTaskId>")`.

## Codex Review

Adversarial review using OpenAI Codex before storing each artifact. Codex gets read-only codebase access to validate claims against actual code.

```bash
echo "<content to review>" | codex exec \
  --sandbox read-only --ephemeral \
  -C "$(git rev-parse --show-toplevel)" \
  -o /tmp/codex-review-output.txt \
  "Review prompt here. Content to review follows on stdin." -
```

Flags: `--sandbox read-only`, `--ephemeral`, `-C` (project root), `-o` (output file), `-` (stdin). No `-m` — use user's default model. Timeout: 120s. Failure → log and proceed (best-effort, never blocks).

Read output file. High-severity actionable → revise before storing. Low/med or intentional choices → note, don't block.

## Continuation (--continue)

Resolve task: argument → task ID; bare → TaskList `type === "scope"`, `status_detail` in `["spec_review", "review", "approved"]`, most recent. Extract relevant metadata.

- `status_detail === "approved"` → already approved. `Skill("develop", "t<taskId>")`.
- `status_detail === "spec_review"` → if metadata.spec_file is set, read content via `ct spec read <spec_file>`; otherwise use metadata.spec. Re-present spec. Resume from step 7.
- `status_detail === "review"` → dispatch subagent with `metadata.design` as prior findings verbatim: "Prior findings: \<metadata.design\>. New prompt: \<user prompt\>. Merge both into updated findings." TaskUpdate merged. If metadata.plan_file → overwrite existing path (do NOT run `ct plan create` again). Re-enter from step 12.

## Key Rules

- Main thread does NOT research — subagents do.
- Spec (what) → plan (how). Each has its own approval gate.
- Codex reviews both artifacts before storing — best-effort, never blocks.
- Archival: `ct spec create` / `ct plan create`. Present as conversation text.
- Scope does NOT create epic or tasks — develop handles that.
- metadata.spec = spec. metadata.design = plan. Separate fields.
- Research Suggested Phases must include file paths — plan and develop depend on them.
- Refinement: minor → revise from findings; major → dispatch follow-up subagent.
- `--auto` bypasses both user review gates AND codex reviews (speed over polish for inner calls).
