---
name: spec
description: "Research a codebase and produce a target-state spec and implementation plan — what to build and how. Two approval gates: spec (what) then plan (how). Triggers: 'spec', 'specify', 'define the target', 'what are we building', 'write a spec', 'plan'. Use for both greenfield features and bug investigations."
argument-hint: "<topic> [--auto] [--continue] [--spec-only] [--depth medium|high|max]"
allowed-tools:
  - Agent
  - Bash
  - Read
  - Glob
  - Grep
user-invocable: true
---

# Spec

Research a codebase and produce a **target-state spec** (what to build) and an **implementation plan** (how to build it). Two approval gates — the user approves the spec before seeing the plan. The plan becomes the contract `/develop` consumes.

Subagents do all codebase exploration. The main thread synthesizes, validates, and presents.

## Arguments

- `<topic>` — what to spec (required unless `--continue`)
- `--auto` — skip both approval gates. Return the plan file path silently.
- `--continue` — resume from existing spec or plan file
- `--spec-only` — stop after spec approval, don't generate a plan
- `--depth medium|high|max` — controls research and plan granularity (default: medium)
  - **medium**: key files, 3-5 phases
  - **high**: call chains, integration points, 5-7 phases with verification steps
  - **max**: exhaustive analysis, cross-reference matrix, 7+ phases with sub-steps

## Phase 1: Spec (the "what")

### 1. Research

Dispatch Agent (subagent_type="Explore"):

```
Research <topic>. Return findings as text.

## Output
1. **Current State**: per relevant file — path, purpose, patterns
2. **Recommendation**: chosen approach + rationale
3. **Key Files**: exact paths relevant to the change, with role descriptions
4. **Risks**: edge cases, failure modes, constraints
```

**Warm-start:** When the prompt contains prior research (from brainstorm or previous spec), include it and say: "Prior research provided below. Validate and fill gaps — focus on what's new."

**Complex domains (3+ subsystems or 3+ viable approaches):** Dispatch 3 parallel Explore agents — Researcher (breadth), Architect (approach), Skeptic (risks). Synthesize the architect's approach with the skeptic's contradictions.

### 2. Validate research

Spot-check architectural claims — wrong understanding invalidates the spec. Check every odd-numbered claim (1st, 3rd, 5th...), minimum 3. Each check: Grep or Read a few lines. Failed check → follow-up subagent to correct.

**Production data correlation** (when upstream context includes logs, error traces, database state): list each concrete observation, state which hypothesis explains it, flag observations that multiple hypotheses explain equally. An unvalidated hypothesis produces a wrong spec.

### 3. Synthesize spec

Build the spec from validated research. The spec is **timeless** — it describes the system as if already built.

**Sections:**

- **Problem**: What's broken or missing. The only section describing current state.

- **Recommendation**: Target behavior in present tense, strategy-level. "Webhook delivery uses exponential backoff via BullMQ" — describes the system, not the change.

- **Architecture Context**: The code landscape post-implementation. Module roles, patterns, key file paths, and how components interact. Include enough detail that a developer reading only this section understands where to work and why things are structured this way. Use Mermaid diagrams (flowchart, sequence, class) when relationships between components would be clearer visually than in prose.

- **Risks**: Edge cases, failure modes, constraints.

**Confidence gate** (for bug investigations):
- **Root-cause confidence**: HIGH / MEDIUM / LOW
- **Supporting evidence**: what confirms the hypothesis
- **Not yet ruled out**: alternatives that remain plausible
- If LOW, flag explicitly — the user must know they're approving under uncertainty.

**Quality gates** (run in parallel before presenting):
- **Simplifier** (conditional — fires when Recommendation has >5 bullets or Architecture Context has >3 subsections): Spawn a subagent to flag over-specification and suggest cuts.
- **Devil's advocate** (always): Spawn a subagent to challenge — is the problem real? Is the scope right? What's the simplest version that works? Carry challenges forward for the user to see.

### 4. Store spec

```bash
SPEC_FILE=$(echo "<spec content>" | ct spec create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" 2>/dev/null)
```

The spec file is the durable artifact. After writing, check for related artifacts and append wiki-links if found:

```bash
RELATED=$(ct vault related --project "$(git rev-parse --show-toplevel)" "<topic>")
# If non-empty, append a ## Related section to the spec file with the links
```

### 5. Present spec

If `--auto` → skip to Phase 2 silently.

Otherwise → present the spec: Problem, Recommendation, Architecture Context, Risks (+ confidence gate if bug investigation). Include devil's advocate challenges. **Stop for user review.**

### 6. Spec refinement

If user gives feedback:
- **Minor (no new research):** Revise from stored research + feedback. Overwrite the spec file.
- **Major (unexplored code or new approach):** Dispatch follow-up subagent with current spec as context. Merge findings. Overwrite spec file.

If `--spec-only` → after approval, output spec path and stop.

## Phase 2: Plan (the "how")

Generated from the approved spec + research findings. The plan is tactical and consumable — `/develop` auto-parses it into tasks.

### 7. Generate plan

From the approved spec and retained research, produce a phased implementation plan. Each phase:

```markdown
**Phase N: <title>**
- **Files**: Read: <paths> | Modify: <paths> | Create: <paths>
- **Approach**: what this phase accomplishes and why
- **Steps**:
  1. <concrete step with file path>
  2. ...
- **Dependencies**: Phase M (if any)
- **Verification**: how to confirm this phase works (test command, expected output)
```

**Rules:**
- Every step must include a file path — `/develop` depends on them.
- Phase boundaries follow natural code boundaries (module, layer, feature slice).
- Earlier phases should unblock later ones — order by dependency, not importance.
- Scale phase count to `--depth`: medium → 3-5, high → 5-7, max → 7+.

### 8. Store plan

```bash
SPEC_STEM=$(basename "$SPEC_FILE" .md)
PLAN_FILE=$(echo "<plan content>" | ct plan create --topic "<topic>" --source "$SPEC_STEM" --project "$(git rev-parse --show-toplevel)" 2>/dev/null)
```

The plan links back to its source spec via `--source`.

### 9. Present plan

If `--auto` → return the plan file path silently.

Otherwise → present the plan phases. **Stop for user review.**

### 10. Plan refinement

If user gives feedback:
- **Reorder/resize phases:** Revise plan from existing research.
- **New approach or missed files:** Dispatch follow-up subagent, update plan.
- Overwrite the plan file after each revision.

### 11. Approve

Output:
```
Spec: <topic>
<one-line recommendation>
Spec file: <spec-path>
Plan file: <plan-path>
Phases: N
Next: /develop <plan-path> or /vibe
```

## Resume (`--continue`)

Check for existing plan first: `ct plan latest --project "$(git rev-parse --show-toplevel)"`. If found, read via `ct plan read <path>`, re-present, resume from step 9.

Otherwise check for spec: `ct spec latest --project "$(git rev-parse --show-toplevel)"`. If found, read via `ct spec read <path>`, then run `ct spec comments <path>`. If comments are non-empty, append them as `## Inline Comments` to the re-presented spec — these are user review feedback that should be addressed during refinement. Resume from step 5 (spec review → plan generation).
