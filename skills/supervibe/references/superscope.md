# Superscope

A dedicated multi-phase research and planning session for supervibe. Unlike regular `/scope` which produces a single spec + plan, superscope produces a phase-decomposed plan where each phase has enough detail to drive an independent `/vibe` run.

The key difference: regular scope optimizes for a single implementation pass. Superscope optimizes for N sequential implementation passes where each must be self-contained and later passes need to account for what earlier passes actually built.

## Flow

### 1. Research

Call scope with a multi-phase-aware prompt:

```
Skill("scope", args="<original prompt>.

This is a supervibe run. The plan must decompose into 2-5 phases, each producing one independent commit. Design phases as vertical slices — each delivers working functionality, not a single architectural layer.

For the plan, provide per-phase detail at this level:
- Title and goal (one sentence: what this phase delivers)
- Files to READ (existing files needed for context)
- Files to MODIFY (existing files that change)
- Files to CREATE (new files)
- Dependencies on prior phases (explicit: 'needs phase 1's DB schema')
- Verification (how to confirm this phase works — a test, a build, a curl command)

The research findings should capture: file locations, existing patterns, architecture context, key types/interfaces. These findings will be passed to per-phase scope calls so they don't re-research from scratch.

--auto")
```

### 2. Extract and store

After scope completes (verify: `status_detail === "approved"`, `metadata.spec` and `metadata.design` populated):

**End-state** (`metadata.end_state`): Extract the Recommendation section from `metadata.spec`. This is the present-tense target state that serves as north star for every phase.

**Phases** (`metadata.phases`): Parse `metadata.design` into a structured array. Each entry:
```json
{
  "title": "Webhook delivery with retry",
  "goal": "Webhook events are delivered via BullMQ with exponential backoff",
  "files": {
    "read": ["src/lib/queue.ts", "src/webhooks/types.ts"],
    "modify": ["src/webhooks/sender.ts"],
    "create": ["src/webhooks/delivery.ts", "src/webhooks/retry.ts"]
  },
  "dependencies": ["Phase 1: webhook model and DB schema must exist"],
  "verification": "npm test -- --grep webhook-delivery"
}
```

**Research findings** (`metadata.superscope_findings`): Distill scope's research into reusable context — the parts that per-phase scope calls need to skip broad exploration:
- Key file locations and their roles
- Existing patterns (e.g., "uses BullMQ for async jobs, see src/lib/queue.ts")
- Architecture decisions (e.g., "Prisma ORM, migrations in prisma/migrations/")
- Types and interfaces that phases will consume or extend

Keep findings concise — this gets passed in every phase prompt. ~500 words max. Focus on what a developer starting phase N needs to know about the codebase, not exhaustive file listings.

### 3. Validate phase plan

Check each property and re-invoke scope with specific feedback if any fails:

1. **Coverage**: Every capability in the end-state maps to at least one phase. Walk the end-state sentence by sentence and verify.

2. **Vertical slices**: Each phase delivers user-visible or testable functionality. Red flags: "Phase 1: DB schema and models" (horizontal layer), "Phase 1: all backend, Phase 2: all frontend" (horizontal split). Each phase should touch multiple layers to deliver one complete capability.

3. **Count**: ≤5 phases. If more, consolidate related phases. Supervibe's reliability degrades with phase count — fewer, meatier phases are better than many thin ones.

4. **Balance**: No phase >3× the file count of another. Rebalance if lopsided. A phase with 2 files next to one with 15 files means the decomposition is off.

5. **Dependencies are explicit**: Each phase's `dependencies` field names the specific prior phase outputs it needs. Implicit dependencies (phase 3 assumes phase 2 created a file but doesn't say so) cause failures when phase 2 deviates from plan.

### 4. Store on tracker

All of the above stored via TaskUpdate on the supervibe tracker task:

```
TaskUpdate(trackerId, metadata: {
  end_state: "<from spec Recommendation>",
  phases: [<structured phase array>],
  superscope_findings: "<distilled research context>",
  scope_task_id: <scope task ID>
})
```

Mark the scope task `status: "completed"`.

## What makes good superscope output

The test for superscope quality: could a developer who has never seen the codebase read `metadata.phases[2]` + `metadata.superscope_findings` and know exactly what to build, which files to touch, and what prior phases already handled?

If the answer is "they'd need to re-read the codebase to figure out where things are" — the research findings are too thin. If the answer is "they'd need to guess what phase 1 did" — the dependencies are too vague.
