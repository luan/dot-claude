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
  - Skill
---

# Prepare

Findings → epic + phased tasks with design briefs.
Reads beads design field, creates implementable task hierarchy.

**IMMEDIATELY dispatch to subagent.** Never prepare on main thread.

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Unclear task boundaries (should X and Y be one task or two?) → ask splitting preference
- Dependency ambiguity (can tasks run parallel or must they serialize?) → confirm with user

Do NOT ask when the answer is obvious or covered by the task brief.

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
     --description "## Problem\n<from design>\n\n## Solution\n<from design>" \
     --acceptance "## Success Criteria\n- [ ] All phases complete\n- [ ] <criteria from design>"
   bd lint <epic-id>
   ```

5. **Create all tasks** — dispatch ONE sonnet subagent (subagent_type="general-purpose", model=sonnet) to create ALL tasks:

   ```
   Create implementation tasks for all phases

   ## All Phases (from design field)
   <all phase descriptions, files, approaches>

   ## Epic
   <epic-id>

   ## Job
   For each phase:
   1. Read referenced files to understand current structure
   2. Design exact changes needed
   3. Create beads task with design brief:

   bd create "<task-title>" --type task \
     --parent <epic-id> --validate --description "$(cat <<'EOF'
   ## Context
   Epic: <epic-id>

   ## Goal
   [what needs to be implemented + why]

   ## Files
   - Read: exact/path/to/file (why: understand current X)
   - Modify: exact/path/to/file (why: add Y)
   - Create: exact/path/to/test (why: verify Z)

   ## Approach
   [how to implement: patterns to use, key decisions, implementation strategy]

   ## Acceptance Criteria
   - [ ] [testable criterion 1]
   - [ ] [testable criterion 2]
   - [ ] No regressions

   ## Assumptions
   - [what must be true about file structure]
   - [what must be true about imports/exports]
   - [what must be true about dependencies]
   EOF
   )"

   4. bd lint <task-id>
   5. Return task ID

   ## Quality Requirements
   - Exact file paths with Read/Modify/Create labels
   - Clear implementation approach and key decisions
   - Testable acceptance criteria
   - Explicit assumptions about file structure
   - Each task = one logical unit (one feature/fix/change)
   ```

   Process all phases in one dispatch (subagent has epic-id for all).

6. **Validate task quality** (subagent-trust.md): spot-check that
   created tasks have real file paths (Read 1-2 referenced files),
   acceptance criteria are testable, and approach is specific enough
   for a worker to implement without guessing. Vague tasks → send
   back to subagent with specific feedback.

7. **Detect dependencies:**
   - Default: sequential (each phase blocks next)
   - Override if phase text says "parallel with Phase N" or "independent of"
   - `bd dep add <phase-N> <phase-N-1>` for sequential

8. **Validate swarm:**
   `bd swarm validate <epic-id> --verbose`

9. **Report:**
   ```
   Epic: <epic-id> — <title>
   Phases: N (M parallel, K sequential)
   Ready front: <first ready tasks>

   Next: /implement <epic-id>
   ```

10. **Continuation prompt:**
   Use AskUserQuestion:
   - "Continue to /implement <epic-id>" (Recommended) — description: "Execute implementation tasks"
   - "Review tasks first" — description: "Inspect the created tasks before implementing"
   - "Done for now" — description: "Leave bead in_progress for later /resume-work"

   If user selects "Continue to /implement":
   → Invoke Skill tool: skill="implement", args="<epic-id>"

## Error Handling
- No design field content → "Run `/explore` first to generate a design", stop
- `bd create` fails for epic → check beads CLI available, report error
- Subagent fails on phase → report which phase, continue others, note gap in report
- `bd swarm validate` fails → show validation errors, ask user how to proceed
