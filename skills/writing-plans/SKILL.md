---
name: writing-plans
description: Write implementation plans with bite-sized tasks. Each step = one action (2-5 min). Used by explore skill.
user-invocable: false
---

# Writing Plans

Write comprehensive plans assuming the implementer has zero context.

**Core principle:** Bite-sized tasks. Each step = one action (2-5 min).

## Bite-Sized Task Granularity

**Each step is one action:**
- "Write the failing test" - step
- "Run it to make sure it fails" - step
- "Implement minimal code to pass" - step
- "Run tests, verify passing" - step
- "Commit" - step

**Not:** "Implement the feature with tests and commit"

## Plan Document Structure

```markdown
# [Feature Name] Implementation Plan

**Goal:** [One sentence]

**Architecture:** [2-3 sentences about approach]

---

### Task 1: [Component Name]

**Files:**
- Create: `exact/path/to/file.ts`
- Modify: `exact/path/to/existing.ts`
- Test: `tests/exact/path/to/test.ts`

**Step 1: Write failing test**
```typescript
test('specific behavior', () => {
  // exact code
});
```

**Step 2: Run test, verify fails**
```bash
npm test path/to/test.ts
```
Expected: FAIL with "function not defined"

**Step 3: Write minimal implementation**
```typescript
// exact code
```

**Step 4: Run test, verify passes**
Expected: PASS

**Step 5: Commit**
```bash
git add path/to/files
git commit -m "feat(scope): add specific feature"
```

---

### Task 2: ...
```

## Key Requirements

1. **Exact file paths** - always
2. **Complete code in plan** - not "add validation"
3. **Exact commands with expected output**
4. **TDD for each task** - red-green-refactor
5. **Frequent commits** - after each task

## Save Location

Save plans to beads issue notes field: `bd update <issue-id> --notes "..."`

## Plan Footer

End every plan summary (in plan mode file) with:

```
To continue: use Skill tool to invoke `implement` with arg `<issue-id>`
```

## Integration

- **explore** skill creates plans using this format
- **implement** skill executes plans in this format
- **subagent-driven-development** expects this task structure
