---
name: prepare
description: "Convert exploration or review findings into beads epic + phased child tasks + swarm validation. Triggers: 'prepare', 'prepare work', 'create tasks from plan'."
argument-hint: "[beads-issue-id]"
user-invocable: true
allowed-tools:
  - Task
  - Bash
  - Read
  - Glob
  - Grep
  - AskUserQuestion
---

# Prepare

Findings → epic + phased tasks with full implementation code.
Reads beads design field, creates implementable task hierarchy.

**IMMEDIATELY dispatch to subagent.** Never prepare on main thread.

## Steps

1. **Find plan source:**
   - If arg is beads ID → `bd show <id> --json`, extract design field
   - Otherwise → `bd list --status in_progress` and find first task with title "Explore:" or "Review:"
   - No plan found → suggest `/explore` or `/review`

2. **Pre-check design quality:**
   - Design field must have "Phase" sections with file paths
   - Missing file paths or vague descriptions → suggest re-running `/explore` with more detail, STOP

3. **Parse plan from design field:**
   - Extract title from first heading or recommendation
   - Find "Next Steps" or "Phase" sections
   - Parse phases: `**Phase N: Description**`
   - Extract files + approach per phase

4. **Create epic:**
   ```bash
   bd create "<title>" --type epic --priority 1 --validate \
     --description "## Problem\n<from design>\n\n## Solution\n<from design>\n\n## Acceptance Criteria\n- [ ] All phases complete"
   bd lint <epic-id>
   ```

5. **Create tasks per phase** — dispatch ONE subagent per phase via Task (subagent_type="general-purpose", model=sonnet):

   ```
   Create implementation task for Phase N: <title>

   ## Phase Context (from design field)
   <phase description, files, approach>

   ## Epic
   <epic-id>

   ## Job
   1. Read referenced files to understand current code
   2. Design exact changes needed
   3. Create beads task with FULL implementation code:

   bd create "<task-title>" --type task \
     --parent <epic-id> --validate --description "$(cat <<'EOF'
   ## Context
   [link to epic]

   ## Files
   - Create/Modify: exact/path/to/file
   - Test: exact/path/to/test

   ## Acceptance Criteria
   - [ ] Test exists + fails without impl
   - [ ] Implementation passes test
   - [ ] No regressions

   ## Implementation

   ### Step 1: Write failing test
   [complete test code]

   ### Step 2: Run test, verify fails
   [exact command + expected output]

   ### Step 3: Minimal implementation
   [complete implementation code]

   ### Step 4: Run test, verify passes
   [exact command + expected output]
   EOF
   )"

   4. bd lint <task-id>
   5. Return task ID

   ## Quality Requirements
   - Complete copy-pasteable test + implementation code
   - Exact file paths, no ambiguity
   - Exact commands with expected output
   - TDD: red → green → refactor baked in
   - Each task = one logical unit (30-80 lines test + impl)
   ```

   Spawn phases sequentially (each needs epic-id).

6. **Detect dependencies:**
   - Default: sequential (each phase blocks next)
   - Override if phase text says "parallel with Phase N" or "independent of"
   - `bd dep add <phase-N> <phase-N-1>` for sequential

7. **Validate swarm:**
   `bd swarm validate <epic-id> --verbose`

8. **Report:**
   ```
   Epic: <epic-id> — <title>
   Phases: N (M parallel, K sequential)
   Ready front: <first ready tasks>

   Next: /implement <epic-id>
   ```
