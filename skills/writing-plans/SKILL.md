---
name: writing-plans
description: Write implementation plans with bite-sized tasks. Each step = one action (2-5 min). Used by explore skill.
user-invocable: false
---

# Writing Plans

Write comprehensive plans as epic + child tasks. Implementer is a fresh agent with NO conversation history.

**Core principle:** Each task issue contains EVERYTHING needed to implement it. No assumptions.

## Description vs Notes

| Field | Purpose | When Written | Example |
|-------|---------|--------------|---------|
| **description** | Static implementation plan | At creation (explore) | Complete code, exact commands |
| **notes** | Dynamic session state | During work (implement) | COMPLETED/IN PROGRESS/NEXT |

Description = WHAT to do (doesn't change)
Notes = WHERE we are (updated each session)

## Resumability Format (for complex technical work)

For work that spans sessions, include in task description:

```markdown
## Implementation

### Working Code (tested)
```python
# Actual code that works - not pseudocode
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
# What the output should look like
Not just "return data" but actual structure
```

### Research Context
- Why this approach?
- What alternatives considered?
- Key discoveries that informed design
```

**The test:** Would a fresh agent struggle to resume from description alone? If yes, add these sections.

## Structure: Epic + Child Tasks

Instead of one giant notes field, create:
1. **Epic** - high-level context (problem, solution, key files)
2. **Child tasks** - one per implementation unit, with complete code

## Step 1: Create Epic

```bash
bd create "<feature-name>" --type epic --validate --description "$(cat <<'EOF'
## Problem
[Why this change is needed - specific user/developer pain]

## Solution
[Approach chosen and why - reference codebase patterns found]

## Key Files
- `src/existing/pattern.ts` - follow this pattern for X
- `tests/example.test.ts` - test style to match

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
EOF
)"
```

Save the epic ID (e.g., `bd-abc123`).

## Step 2: Create Child Tasks

For EACH implementation unit, create a child task:

```bash
bd create "<task-title>" --type task --parent <epic-id> --validate --description "$(cat <<'EOF'
## Context
[1-2 sentences linking to epic goal]

## Files
- Create: `exact/path/to/file.ts`
- Modify: `exact/path/to/existing.ts` (line ~45, after FooClass)
- Test: `tests/exact/path/to/test.ts`

## Acceptance Criteria
- [ ] Test exists and fails without implementation
- [ ] Implementation passes test
- [ ] No regressions

## Implementation

### Step 1: Write failing test
```typescript
// COMPLETE test code - not pseudocode
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
Expected: FAIL with "validateEmail is not defined"

### Step 3: Write minimal implementation
```typescript
// COMPLETE implementation - not "add validation logic"
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

## Step 3: Add Dependencies (if sequential)

```bash
bd dep add <task-2-id> <task-1-id>  # task-2 blocked by task-1
```

## Step 4: Validate All Issues

```bash
bd lint <epic-id>
bd children <epic-id> | xargs bd lint
```

Fix any issues flagged by lint before proceeding.

## Key Requirements

1. **Complete code in each task** - implementer copies and pastes
2. **Exact file paths** - no ambiguity
3. **Exact commands with expected output**
4. **TDD steps** - red-green-refactor baked in
5. **--validate flag** - enforces required sections

## Task Granularity

Each task = one logical unit of work:
- One new function/class with its test
- One refactoring with verification
- One integration point

**Size target:** 30-80 lines per task (test + impl combined)
- Allows 3-4 tasks per PR (~250 line limit)
- Small enough to complete atomically
- Large enough to be meaningful

**Not:** "Implement the whole feature" (too big, can't finish in one PR)
**Not:** "Write line 45" (too small, overhead not worth it)

**CRITICAL:** Tasks must be completable atomically. Once started, a task MUST be finished before any PR operations. Size tasks so they can always be completed.

## Plan Footer

End every plan summary (in plan mode file) with:

```
Epic: <epic-id>
Tasks: <task-1-id>, <task-2-id>, ...

To continue: use Skill tool to invoke `implement` with arg `<epic-id>`
```
