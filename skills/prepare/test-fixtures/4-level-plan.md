---
topic: "4-Level Decomposition Integration Test"
---

# 4-Level Decomposition: End-to-End Scenario

Demonstrates the full flow: plan file → `ct tool phases` → `prepare` (task hierarchy) → `implement` scheduling → `acceptance` criteria gathering.

## Plan Input

A plan with two phases, each containing numbered tasks with indented sub-items (3-level markdown → 4-level task system: epic → phase → sub-task → leaf).

```
### Phase 1: Data Layer
1. Schema design
  - define entity relationships
  - write migration files
1. Repository layer
  - create base repository
  - implement query methods
  - add transaction support

### Phase 2: API Layer
1. Route handlers
  - authentication middleware
  - request validation
  - response serialization
1. Error handling
```

## Observable Checkpoints

### Checkpoint 1: ct tool phases JSON

`ct tool phases <plan-file>` emits a JSON array. Each task object carries a `sub_tasks` array:

```json
[
  {
    "phase": 1,
    "title": "Data Layer",
    "tasks": [
      {"text": "Schema design", "sub_tasks": ["define entity relationships", "write migration files"]},
      {"text": "Repository layer", "sub_tasks": ["create base repository", "implement query methods", "add transaction support"]}
    ],
    "deps": []
  },
  {
    "phase": 2,
    "title": "API Layer",
    "tasks": [
      {"text": "Route handlers", "sub_tasks": ["authentication middleware", "request validation", "response serialization"]},
      {"text": "Error handling", "sub_tasks": []}
    ],
    "deps": [1]
  }
]
```

### Checkpoint 2: prepare → task hierarchy

`prepare` reads the JSON, detects phases with 3+ distinct concerns, and decomposes them:

- **Epic** (no depth metadata): "4-Level Decomposition Integration Test"
- **Phase 1 task** (`depth: 1`, grouping node — 3 concerns triggers decomposition):
  - Sub-task (`depth: 2`): "Schema design with migrations"
  - Sub-task (`depth: 2`): "Repository layer implementation"
- **Phase 2 task** (`depth: 1`, grouping node — 3 concerns in Route handlers):
  - Sub-task (`depth: 2`): "Route handlers"
  - Sub-task (`depth: 2`): "Error handling" (flat, no children)

Depth never exceeds 3. If decomposition would require depth 4+, `prepare` flattens to depth ≤ 3.

### Checkpoint 3: implement scheduling

`implement` on the epic detects **Swarm mode** (tasks have `blockedBy` relationships — Phase 2 blocked by Phase 1):

- Phase 1 sub-tasks (depth-2 leaves) dispatched first — up to 4 concurrent workers
- Phase 2 sub-tasks dispatched after all Phase 1 leaves complete
- Rolling scheduler re-scans descendants after each completion to pick up newly unblocked work
- Progress line emitted after each worker finishes: `N completed, M active, K pending`

### Checkpoint 4: acceptance criteria gathering

`acceptance` recursively collects all descendants and groups by subtree:

```
Phase 1: Data Layer
  Task <id>: Schema design with migrations
  - [ ] Migration files exist and run without error
  - [ ] Entity relationships reflected in schema
  Task <id>: Repository layer implementation
  - [ ] All CRUD operations implemented
  - [ ] Transaction support verified via rollback test

Phase 2: API Layer
  Task <id>: Route handlers
  - [ ] Authentication middleware rejects unauthenticated requests
  - [ ] Request validation returns 400 on bad input
  - [ ] Response serialization produces valid JSON
  Task <id>: Error handling
  - [ ] All error paths return structured error responses
  - [ ] No 500s leak internal details
```

### Checkpoint 5: acceptance verdict

- **Verifier** confirms each criterion against the diff, outputs PASS/FAIL/PARTIAL per row
- **Breaker** hunts implied requirements (e.g., does auth middleware apply to all routes?) and integration gaps (e.g., does error handler wrap repository exceptions?)
- Reconciliation: verifier PASS + no HIGH breaker findings → overall **PASS**
- Findings stored in `metadata.acceptance_result` on the epic task
