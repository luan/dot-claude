---
name: spec
description: "Research a codebase and produce a target-state spec — what to build, with enough architecture context to execute. Triggers: 'spec', 'specify', 'define the target', 'what are we building', 'what should we build', 'write a spec'. Use for both greenfield features and bug investigations."
argument-hint: "<topic> [--auto] [--continue]"
allowed-tools:
  - Agent
  - Bash
  - Read
  - Glob
  - Grep
user-invocable: true
---

# Spec

Research a codebase and produce a **target-state spec** — a document describing the system as if already built, with enough architecture context for develop to execute without losing intent.

Subagents do all codebase exploration. The main thread synthesizes, validates, and presents.

## Arguments

- `<topic>` — what to spec (required unless `--continue`)
- `--auto` — skip approval gate and codex review. Return the spec file path silently.
- `--continue` — resume from existing spec file

## Workflow

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

- **Architecture Context**: The code landscape post-implementation. Module roles, patterns, key file paths, and how components interact. Include enough detail that a developer reading only this section understands where to work and why things are structured this way.

- **Risks**: Edge cases, failure modes, constraints.

**Confidence gate** (for bug investigations):
- **Root-cause confidence**: HIGH / MEDIUM / LOW
- **Supporting evidence**: what confirms the hypothesis
- **Not yet ruled out**: alternatives that remain plausible
- If LOW, flag explicitly — the user must know they're approving under uncertainty.

### 4. Store as file

```bash
SPEC_FILE=$(echo "<spec content>" | ct spec create --topic "<topic>" --project "$(git rev-parse --show-toplevel)" --prefix "spec" 2>/dev/null)
```

The spec file is the durable artifact. Downstream skills (develop, vibe) read it via `ct spec read`.

### 5. Present

If `--auto` → return silently. Caller reads the spec file path.

Otherwise → present the spec: Problem, Recommendation, Architecture Context, Risks (+ confidence gate if bug investigation). Stop for user review.

### 6. Refinement

If user gives feedback:
- **Minor (no new research):** Revise from stored research + feedback. Overwrite the spec file.
- **Major (unexplored code or new approach):** Dispatch follow-up subagent with current spec as context. Merge findings. Overwrite spec file.

### 7. Approve

Output:
```
Spec: <topic>
<one-line recommendation>
Spec file: <path>
Next: /develop <path>, /vibe, or /plan
```

## Resume (`--continue`)

Find the most recent spec file: `ct spec latest --project "$(git rev-parse --show-toplevel)"`. Read via `ct spec read <path>`. Re-present and resume from step 5.
