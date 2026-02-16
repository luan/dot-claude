---
name: next
description: "Discover and dispatch the next highest-priority unblocked work item. Triggers: 'next', 'what next', 'pick up next task', 'continue working'."
---

# Next

Read the work board, find what's ready, dispatch the right skill.

## Instructions

### 1. Read the board

```bash
work list --format json --status open
work list --format json --status active
```

Merge results. Sort by priority (1 highest), then by created date
(oldest first).

### 2. Pick the top candidate

Select the highest-priority item that is NOT blocked. Skip:
- Items with status `done` or `cancelled`
- Items with status `review` (waiting on human)
- Items whose description says "blocked by" an open issue

If an item has status `active`, prefer it over `open` items at the
same priority (someone already started it — resume it).

### 3. Read the candidate

```bash
work show <id>
```

### 4. Determine the action

Read the issue description and classify:

| Signal | Action |
|--------|--------|
| Type is `bug` | `/debugging` |
| Title starts with "Brainstorm:" or description says "Needs brainstorm" | `/brainstorm` |
| Title starts with "Explore:" or description says "Needs explore" or "Explore first" | `/explore` |
| Has design/plan but no children and isn't a leaf task | `/prepare` |
| Has children or is a leaf task ready to build | `/implement` |
| Status is `active` and has prior work context | `/resume-work` |

**Tie-breaking:**
- Feature with no "## Approach" or "## Design" section → `/explore`
- Feature with approach but no concrete phases → `/brainstorm`
- When genuinely ambiguous → `/explore` (cheaper to course-correct)

### 5. Present to user

Use AskUserQuestion with:
- Recommended action as first option (with "(Recommended)")
- 1-2 alternatives that could also apply
- Brief explanation of why this issue and this action

Format the question header as the issue ID.

Example:
```
Question: "Next up: <title> (P<n>). What should we do?"
Options:
  - "/explore (Recommended)" — "Investigate before designing"
  - "/brainstorm" — "Jump to design dialogue"
  - "Skip — show me the next one" — "Pick a different issue"
```

### 6. Dispatch

Based on user's choice, invoke the selected skill:

```
Skill tool: skill="<chosen-skill>", args="<issue-id>"
```

If user chose "Skip", go back to step 2 with the next candidate.

## Key Rules

- Never dispatch without showing the user what you picked and why
- Never skip reading the issue — the description drives classification
- If `work list` returns nothing actionable, say so plainly
- Don't create issues — this skill discovers, it doesn't plan
