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
  - **medium**: behavioral analysis, phases follow natural code boundaries
  - **high**: component contracts, integration points, gates between phases
  - **max**: exhaustive analysis, full execution model with agent allocation and dependency graph

## Phase 1: Spec (the "what")

### 1. Research

Dispatch Agent (subagent_type="Explore"):

```
Research <topic>. Return findings as text.

## Output
1. **Current Behavior**: what the system does today, where it falls short, and why
2. **Component Boundaries**: modules/layers involved, their contracts, how they interact
3. **Constraints**: what's load-bearing, what must not break, external interfaces
4. **Design Space**: viable approaches with tradeoffs — name at least two, recommend one with rationale
```

**Warm-start:** When the prompt contains prior research (from brainstorm or previous spec), include it and say: "Prior research provided below. Validate and fill gaps — focus on what's new."

**Complex domains (3+ subsystems or 3+ viable approaches):** Dispatch 3 parallel Explore agents — Researcher (breadth), Architect (approach), Skeptic (risks). Synthesize the architect's approach with the skeptic's contradictions.

### 2. Validate research

Spot-check architectural claims — wrong understanding invalidates the spec. Check every odd-numbered claim (1st, 3rd, 5th...), minimum 3. Each check: Grep or Read a few lines. Failed check → follow-up subagent to correct.

**Production data correlation** (when upstream context includes logs, error traces, database state): list each concrete observation, state which hypothesis explains it, flag observations that multiple hypotheses explain equally. An unvalidated hypothesis produces a wrong spec.

### 3. Synthesize spec

Build the spec from validated research. The spec defines the **target state** — the system as if already built, present tense throughout. A developer should understand the target system from the spec alone — without needing to read source code to resolve ambiguities about intent.

**Sections:**

- **Problem**: What's broken or missing. The only section describing current state.

- **Recommendation**: The system as-if-built — behavior, contracts, data flow. This is the heart of the spec and must be self-sufficient. Include:
  - **Concrete behavior** in present tense. For UI: ASCII mockups showing exactly what appears on screen. For APIs: request/response shapes. For data models: type definitions or schemas. For protocols: message sequences.
  - **Data flow**: numbered steps showing how data moves end-to-end. "1. User clicks Save → 2. App serializes to KDL → 3. MapStore writes sector files."

  For refactors, add a **removal table** mapping every current element to its disposition — deleted, renamed, absorbed into X. Every element accounted for.
  - **Key decisions**: choices baked into the design and the reasoning. Alternatives considered and why rejected.

  File paths do not belong in the Recommendation. The spec defines *what the system is*, not *where code lives*. Implementation geography belongs in the plan.

- **Architecture Context**: How components relate post-implementation. Use Mermaid diagrams to show relationships visually. Include a brief new/modified/deleted file list as a reference — the diagram carries the meaning, but the file list gives developers a geographic anchor. Keep prose minimal; the file list is cheap.

- **Risks**: Each risk names a concrete failure scenario and a mitigation. Not vague categories — specific things that can go wrong and what prevents them.

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

From the approved spec and retained research, produce an execution plan. The plan is the contract `/develop` consumes — precise enough that an agent can execute each step without judgment calls about intent.

**Plan structure:**

Start with an **execution model** — a Mermaid graph showing phases and their dependencies. The shape of this graph should reflect the work's actual dependency structure, not a template. Most plans are sequential; parallelism is justified only when phases touch genuinely independent code with no shared state. Show agent allocation on each node when multiple tiers are used.

Think critically about agent allocation. Default to the strongest available agent — only delegate to faster/cheaper agents when the work is unambiguously mechanical (additive type definitions, dead code deletion, imports). If in doubt, use the stronger agent.

Add **gates** (build/test/review checkpoints) between phases when the next phase depends on correctness of the previous one. Gates are where an agent reviews the diff against the spec and flags deviations. Not every phase boundary needs a gate — use them at natural convergence points.

**Per-phase details:**

```markdown
## Phase N: <title>

- **Agent**: <tier>, <execution mode>
- **Files**: Modify: <paths> | Create: <paths>
- **Approach**: what this phase accomplishes and why it's sequenced here
- **Steps**:
  1. `<file path>` — <exact change: "add X with Y", "change A from B to C", "remove D">
  2. ...
- **Dependencies**: Phase M (if any)
- **Verification**: <build/test> + <spec compliance checks for this phase>
```

**Rules:**
- Every step specifies an exact behavioral change with a file path — "add enum variant X with derives Y", "change method Z to accept A instead of B", "remove struct W." Never "refactor" or "update as needed."
- Phase boundaries follow natural code boundaries. The number of phases emerges from the dependency structure — don't pad or compress to hit a count.
- Earlier phases unblock later ones — order by dependency, not importance.
- Verification for phases at `--depth high|max` includes a spec compliance checklist — concrete requirements from the spec that this phase should satisfy.

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
