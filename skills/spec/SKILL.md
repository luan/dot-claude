---
name: spec
description: "Define what to build — produces a target-state spec through codebase research and synthesis. The spec describes the system as if already built, excluding implementation details. Use this skill when the user wants to define requirements, specify a target state, answer 'what should we build', write a spec, or needs a clear definition of done before executing. Also triggers on: 'spec', 'specify', 'define the target', 'target state', 'what are we building'. Do NOT use when: the user wants to plan HOW to build (use /scope), wants to brainstorm design OPTIONS (use /brainstorm), or wants to just execute (use /vibe or /develop)."
argument-hint: "<topic> [--auto] [--continue]"
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
user-invocable: true
---

# Spec

Research a codebase and produce a **target-state spec** — a timeless document describing the system as if already built. The spec answers "what are we building?" without prescribing how.

**Never research on main thread** — subagents do all codebase exploration.

## Arguments

- `<topic>` — what to spec (required unless `--continue`)
- `--auto` — skip approval gate and codex review (for inner calls from vibe/supervibe)
- `--continue` — resume from existing spec task

## Interviewing

See rules/skill-interviewing.md.

## Workflow

### 1. Create tracking task

```
TaskCreate(
  subject: "Spec: <topic>",
  metadata: { project: <repo root>, type: "spec", priority: "P2" }
)
TaskUpdate(taskId, status: "in_progress", owner: "spec")
```

### 2. Research

Dispatch subagent (subagent_type="Explore"):

```
Research <topic>. Return findings as text (do NOT write files or create tasks).

## Output
1. **Current State**: per file — path, exports/defines, patterns
2. **Recommendation**: chosen approach + rationale
3. **Key Files**: exact paths relevant to the change
4. **Risks**: edge cases, failure modes
```

**Warm-start:** When the prompt contains prior research (from brainstorm, previous spec, or supervibe context), the subagent validates and fills gaps instead of broad exploration. Include prior context and say: "Prior research provided below. Validate and fill gaps — do NOT re-explore what's already covered."

**On complex domains (3+ subsystems or 3+ viable approaches):** TeamCreate, dispatch 3 agents (mode: "plan") — Researcher, Architect, Skeptic. Synthesize: Architect's approach + contradictions from Skeptic. TeamDelete.

### 3. Validate research

Spot-check ALL architectural claims — wrong understanding of the codebase invalidates the spec. File/behavioral claims: check every odd-numbered claim (1st, 3rd, 5th...), minimum 3. Each check: Grep or Read a few lines. Failed check → follow-up subagent.

### 3b. Production data correlation

When upstream context includes production data (triage logs, sync debug dumps, /dia-inspect-data output, error traces, database state), the spec MUST explain the specific observations in that data before locking in a root cause.

- List each concrete observation from the production data (e.g., "zero update events in sync log", "stale timestamp in database row", "404 in error trace").
- For each observation, state which hypothesis explains it and whether alternative hypotheses also explain it.
- If an observation is equally explained by multiple hypotheses, the root cause is NOT confirmed — flag it.

**Subagent prompt addition** (when production data exists): append to research prompt:

```
## Production Data
<paste observations>

For each observation above: state which part of the codebase produces this data point, trace the code path that leads to it, and determine whether your hypothesis is the ONLY explanation or whether other code paths could produce the same observation.
```

### 3c. Root-cause validation checklist

Before synthesizing, verify the hypothesis survives these checks:

1. **Exclusivity**: Does the hypothesis explain ALL production observations, not just some?
2. **Alternatives ruled out**: For each production observation, are there other code paths that could produce it? If yes, have those been investigated?
3. **Absence of evidence vs. evidence of absence**: If the key evidence is "X didn't happen," confirm whether the instrumentation/logging would have captured X if it did happen.

If any check fails → dispatch follow-up subagent targeting the gap. Do NOT synthesize a spec on an unvalidated hypothesis.

### 4. Synthesize spec

Build the spec from validated research. A spec is a **timeless target-state document** — it describes the system as if already built. After implementation, the spec should still read as a valid description of the system (not a dated change request).

**Sections:**

- **Problem**: What's broken or missing. The only section that describes current state.

- **Recommendation**: Target behavior in present tense, strategy-level. No transition verbs — "Webhook delivery uses exponential backoff via BullMQ" not "Add exponential backoff." Describes the system, not the change.

- **Architecture Context**: The code landscape post-implementation. Describe by module role and pattern, not file path. Paths may appear parenthetically but the description stands without them. No Create/Modify annotations.

- **Risks**: Edge cases, failure modes, constraints.

The spec **excludes** implementation details: phases, task breakdowns, files to create/modify, specific code changes. Those belong to /scope.

### 5. Codex review

**Skip if `--auto`.**

See [Codex Review](#codex-review). Prompt: "Review this spec for gaps, ambiguities, edge cases, and feasibility. This is a WHAT document — do NOT suggest implementation details."

High-severity actionable issues → revise. Best-effort — if codex fails, proceed.

### 6. Store

```bash
SPEC_FILE=$(echo "<spec content>" | ct spec create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "spec" 2>/dev/null)
```

```
TaskUpdate(taskId, metadata: {
  spec: "<spec content>",
  spec_file: "$SPEC_FILE" (omit if empty),
  status_detail: "spec_review"
})
```

### 7. Present

Output: `Spec: t<id> — <topic>`, then Problem, Recommendation, Architecture Context, Risks.

**Confidence gate** (append after Risks, before stopping for review):

- **Root-cause confidence**: HIGH / MEDIUM / LOW
- **Supporting evidence**: what production data or code analysis confirms the hypothesis
- **Not yet ruled out**: alternative explanations that remain plausible and what evidence would confirm or eliminate them
- **Would increase confidence**: specific investigation or data that would move from MEDIUM→HIGH

If confidence is LOW, state this explicitly and recommend further investigation before approving. Do NOT present a LOW-confidence spec without flagging it — the user must know they are approving under uncertainty.

If `--auto` → skip to step 9.
Otherwise → stop for user review.

### 8. Refinement

If user gives feedback:

- **Minor (no new research):** Revise from stored research + feedback. TaskUpdate metadata.spec. If metadata.spec_file → overwrite existing path (do NOT `ct spec create` again). status_detail stays `"spec_review"`.
- **Major (unexplored code or new approach):** Dispatch follow-up subagent with current spec as context. Merge findings. TaskUpdate. Overwrite spec_file if set.
- Persist metadata.spec before re-presenting (downstream skills read stored artifacts, not conversation).

### 9. Approve

```
TaskUpdate(taskId, metadata: { status_detail: "approved" })
TaskUpdate(taskId, status: "completed")
```

Mark the spec task `completed` at approval time — not just `status_detail: approved`. Downstream skills (vibe, scope, develop) consume the spec via metadata, but the task itself is done. Leaving it `in_progress` creates orphaned tasks that persist across sessions.

If `--auto` → return silently. Caller reads `metadata.spec`.

**Output summary (interactive only):**
```
Spec: t<id> — <topic>
<one-line recommendation>
Next: /scope t<id>, /vibe t<id>, or /supervibe t<id>
```

## Codex Review

Adversarial review using OpenAI Codex before storing. Codex gets read-only codebase access to validate claims.

```bash
echo "<content>" | codex exec \
  --sandbox read-only --ephemeral \
  -C "$(git rev-parse --show-toplevel)" \
  -o /tmp/codex-review-output.txt \
  "Review prompt here. Content follows on stdin." -
```

Timeout: 120s. Failure → log and proceed (best-effort, never blocks).

Read output file. High-severity actionable → revise before storing. Low/med → note, don't block.

## Resume (`--continue`)

Resolve task: argument → task ID; bare → TaskList `type === "spec"`, `status_detail` in `["spec_review", "approved"]`, most recent.

- `status_detail === "approved"` → already done. Report and suggest next steps.
- `status_detail === "spec_review"` → if metadata.spec_file is set, read via `ct spec read <spec_file>`; otherwise use metadata.spec. Re-present spec. Resume from step 7.

## Key Rules

- Main thread does NOT research — subagents do.
- Spec is the "what" — no implementation details, no phases, no file-level plans.
- metadata.spec = spec content. metadata.spec_file = ct archive path.
- Codex review is best-effort, never blocks.
- `--auto` bypasses approval gate, codex review, AND all text output. Caller reads task metadata.
- Refinement: minor → revise from findings; major → dispatch follow-up subagent.
- Present `Next:` options after approval — user chooses the executor.
