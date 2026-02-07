---
name: writing-plans
description: Write implementation plans with bite-sized tasks. Each step = one action (2-5 min). Used by explore skill.
user-invocable: false
---

# Writing Plans

Plans = epic + child tasks. Implementer is fresh agent with NO history.

**Core principle:** Each task contains EVERYTHING needed. No assumptions.

## Description vs Notes

| Field | Purpose | When Written | Example |
|-------|---------|--------------|---------|
| **description** | Static plan | At creation (explore) | Complete code, exact commands |
| **notes** | Session state | During work (implement) | COMPLETED/IN PROGRESS/NEXT |

Description = WHAT (immutable). Notes = WHERE (updated per session).

## Resumability Format

For multi-session work, include in description:

```markdown
## Implementation

### Working Code (tested)
```python
service = build('api', 'v1', credentials=creds)
result = service.method().execute()
# Returns: {'key': 'actual structure'}
```

### API Response Sample
```json
{"actualField": "actualValue", "structure": "as returned"}
```

### Desired Output Format
```markdown
# Actual expected structure, not just "return data"
```

### Research Context
- Why this approach? Alternatives considered?
- Key discoveries informing design
```

**Test:** Would fresh agent struggle to resume from description alone? If yes → add these sections.

## Structure: Epic + Child Tasks

1. **Epic** — problem, solution, key files
2. **Child tasks** — one per implementation unit, complete code

### Multi-Phase Epics (5+ tasks)

For larger epics, group tasks into phases with blocking dependencies:

```bash
# Phase 1 tasks have no cross-phase deps
bd create "Phase 1: <name>" --type task --parent <epic-id> --validate
# Phase 2 tasks blocked by phase 1 completion
bd create "Phase 2: <name>" --type task --parent <epic-id> --validate --deps <phase-1-task-id>
```

- `bd ready` auto-scopes to current phase (blocked tasks hidden)
- Implementers naturally work phase-by-phase without manual coordination

### Bulk Task Creation

For plans with many tasks, write a plan file and create in bulk:

```bash
bd create --file plan.md
```

## Step 1: Create Epic

```bash
bd create "<feature-name>" --type epic --validate --description "$(cat <<'EOF'
## Problem
[Specific user/developer pain]

## Solution
[Approach + why — reference codebase patterns]

## Key Files
- `src/existing/pattern.ts` - follow this pattern
- `tests/example.test.ts` - test style to match

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
EOF
)"
```

Save epic ID (e.g., `bd-abc123`).

## Step 2: Create Child Tasks

Per implementation unit:

```bash
bd create "<task-title>" --type task --parent <epic-id> --validate --description "$(cat <<'EOF'
## Context
[1-2 sentences linking to epic]

## Files
- Create: `exact/path/to/file.ts`
- Modify: `exact/path/to/existing.ts` (line ~45, after FooClass)
- Test: `tests/exact/path/to/test.ts`

## Acceptance Criteria
- [ ] Test exists + fails without impl
- [ ] Implementation passes test
- [ ] No regressions

## Implementation

### Step 1: Write failing test
```typescript
test('validates email format', () => {
  const result = validateEmail('invalid');
  expect(result.valid).toBe(false);
  expect(result.error).toBe('Invalid email format');
});
```

### Step 2: Run test, verify fails
```bash
npm test -- path/to/test.ts
```
Expected: FAIL — "validateEmail is not defined"

### Step 3: Write minimal implementation
```typescript
export function validateEmail(email: string): ValidationResult {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(email)) {
    return { valid: false, error: 'Invalid email format' };
  }
  return { valid: true };
}
```

### Step 4: Run test, verify passes
Expected: PASS
EOF
)"
```

## Step 3: Add Dependencies

```bash
bd dep add <task-2-id> <task-1-id>  # task-2 blocked by task-1
```

## Step 4: Validate All Issues

```bash
bd lint <epic-id>
bd children <epic-id> | xargs bd lint
```

Fix lint issues before proceeding.

## Key Requirements

1. Complete code per task — implementer copy-pastes
2. Exact file paths, no ambiguity
3. Exact commands + expected output
4. TDD steps baked in (red → green → refactor)
5. `--validate` flag on all creates

## Task Granularity

Each task = one logical unit: function/class + test, one refactor, one integration point.

**Size:** 30-80 lines (test + impl). Allows 3-4 tasks per PR (~250 lines).

**Not:** "Implement whole feature" (too big) or "Write line 45" (too small).

**CRITICAL:** Tasks must be atomic. Once started → must finish before PR ops.

## Plan Footer

End every plan summary with:

```
Epic: <epic-id>
Tasks: <task-1-id>, <task-2-id>, ...

To continue: use Skill tool to invoke `implement` with arg `<epic-id>`
```
