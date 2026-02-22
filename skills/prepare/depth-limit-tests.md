# Manual Test Protocol: Depth Limit Enforcement

The depth limit (`depth ≤ 3`) is enforced by the prepare subagent via prompt instructions in SKILL.md (step 5, decomposition rule). No Rust code enforces it — `phases.rs` only parses plan documents, not task hierarchies.

## Rule Under Test

> Depth limit: never create tasks at depth > 3 (1=phase, 2=sub-task, 3=leaf). If decomposition would require 4+ levels, flatten to depth ≤ 3 instead.

---

## Scenario A: Depth 1 — flat phase (accepted)

**Input plan:**
```
### Phase 1: Add user login
1. Create LoginForm component
2. Add POST /auth/login endpoint
```

**Expected TaskCreate calls:**

| depth | parent_id | subject |
|-------|-----------|---------|
| 1     | <epic>    | Phase 1: Add user login — LoginForm |
| 1     | <epic>    | Phase 1: Add user login — auth endpoint |

OR as a single task if concerns are unified (1-2 concerns → no decomposition):

| depth | parent_id | subject |
|-------|-----------|---------|
| 1     | <epic>    | Phase 1: Add user login |

**Verification:** `ck list` shows tasks with `depth=1`, no tasks with `depth=2+`.

---

## Scenario B: Depth 2 — phase with 3+ concerns (accepted)

**Input plan:**
```
### Phase 1: Authentication
1. LoginForm component (UI)
2. POST /auth/login endpoint (API)
3. JWT token storage (state)
4. Auth middleware (infra)
```

**Expected TaskCreate calls:** (4 concerns → decompose)

| depth | parent_id    | subject |
|-------|--------------|---------|
| 1     | <epic>       | Phase 1: Authentication ← grouping node |
| 2     | <phase-task> | Phase 1: LoginForm component |
| 2     | <phase-task> | Phase 1: auth/login endpoint |
| 2     | <phase-task> | Phase 1: JWT token storage |
| 2     | <phase-task> | Phase 1: Auth middleware |

**Verification:** Grouping task at depth 1; all sub-tasks at depth 2; no depth 3.

---

## Scenario C: Depth 3 — deeply nested concerns (accepted, boundary case)

**Input plan:**
```
### Phase 1: Storage layer
1. Define schema (data model)
2. Write migration (database)
3. Build repository (business logic — reads/writes/queries/aggregates)
```

The repository concern (3) has 4 sub-concerns. Decompose to depth 3:

| depth | parent_id     | subject |
|-------|---------------|---------|
| 1     | <epic>        | Phase 1: Storage layer |
| 2     | <phase-task>  | Phase 1: Define schema |
| 2     | <phase-task>  | Phase 1: Write migration |
| 2     | <phase-task>  | Phase 1: Repository |
| 3     | <repo-task>   | Phase 1: Repository — reads |
| 3     | <repo-task>   | Phase 1: Repository — writes |
| 3     | <repo-task>   | Phase 1: Repository — queries |
| 3     | <repo-task>   | Phase 1: Repository — aggregates |

**Verification:** Leaf tasks at depth 3; `ck list` shows no task with `depth=4`.

---

## Scenario D: Depth 4 would-be — must flatten (rejected/flattened)

**Input plan** that naively requires 4 levels:
```
Phase 1 (depth 1)
  └─ Concern A (depth 2)
       └─ Sub-concern X (depth 3)
            └─ Leaf (would be depth 4) ← FORBIDDEN
```

**Expected behavior:** The subagent must NOT create a depth-4 task. Instead, flatten:
- Promote the depth-4 work into the depth-3 task's description, OR
- Merge depth-3 and depth-4 into a single depth-3 leaf task

**Expected output — flattened:**

| depth | parent_id       | subject |
|-------|-----------------|---------|
| 1     | <epic>          | Phase 1: Top |
| 2     | <phase-task>    | Phase 1: Concern A |
| 3     | <concern-task>  | Phase 1: Sub-concern X (includes leaf work) |

**No task with `depth=4` should appear in `ck list`.**

**Verification steps:**
1. Run `/prepare` on a plan with 4+ natural nesting levels
2. Inspect created tasks: `ck list` filtered by the epic
3. Confirm max depth observed = 3
4. Confirm the depth-4 work is absorbed into the depth-3 task description

---

## Depth Value Verification (all scenarios)

After any `/prepare` run, verify depth assignments with:

```
ck list  # check metadata.depth column
```

| Expected | Condition |
|----------|-----------|
| depth=1  | Direct children of epic; phase grouping nodes |
| depth=2  | Sub-tasks under a phase grouping node |
| depth=3  | Leaves under sub-tasks |
| depth≥4  | MUST NOT EXIST — bug if present |
