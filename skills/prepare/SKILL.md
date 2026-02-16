---
name: prepare
description: "Convert exploration or review findings into epic + phased child tasks. Triggers: 'prepare', 'prepare work', 'create tasks from plan'."
argument-hint: "[work-issue-id]"
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
Reads issue description, creates implementable task hierarchy.

**IMMEDIATELY dispatch to subagent.** Never prepare on main thread.

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Unclear task boundaries (should X and Y be one task or two?) → ask splitting preference
- Dependency ambiguity (can tasks run parallel or must they serialize?) → confirm with user

Do NOT ask when the answer is obvious or covered by the task brief.

## Steps

1. **Find plan source:**
   - If arg is work ID → `work show <id> --format=json`, extract description
   - Otherwise → try labels in order (review status first, then active):
     `work list --status review --label explore`, then
     `work list --status review --label review`, then
     `work list --status review --label fix`, then
     `work list --status active --label explore` — use first result
   - No plan found → suggest `/explore` or `/review`

2. **Pre-check design quality:**
   - Description must have "Phase" sections with file paths
   - Missing file paths or vague descriptions → suggest re-running `/explore` with more detail, STOP

3. **Parse plan from description:**
   - Extract title from first heading or recommendation
   - Find "Next Steps" or "Phase" sections
   - Parse phases: `**Phase N: Description**`
   - Extract files + approach per phase

4. **Generate group label:**
   Derive kebab-case label from plan title (e.g., "Add user auth" → `add-user-auth`).
   This label connects parent + children for easy querying.

5. **Create epic (parent issue):**
   ```bash
   work create "<title>" --type feature --priority 1 \
     --labels <group-label> \
     --description "## Problem\n<from design>\n\n## Solution\n<from design>\n\n## Acceptance Criteria\n- [ ] All phases complete\n- [ ] <criteria from design>"
   ```

6. **Create all tasks** — dispatch ONE sonnet subagent (subagent_type="general-purpose", model=sonnet) to create ALL tasks:

   ```
   Create implementation tasks for all phases

   ## All Phases (from description)
   <all phase descriptions, files, approaches>

   ## Epic
   <epic-id>

   ## Group Label
   <group-label>

   ## Job
   For each phase:
   1. Read referenced files to understand current structure
   2. Design exact changes needed
   3. Create work issue with design brief:

   work create "Phase N: <task-title>" --type chore \
     --parent <epic-id> --labels <group-label> --description "$(cat <<'EOF'
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

   4. Return task ID

   ## Quality Requirements
   - Exact file paths with Read/Modify/Create labels
   - Clear implementation approach and key decisions
   - Testable acceptance criteria
   - Explicit assumptions about file structure
   - Each task = one logical unit (one feature/fix/change)
   ```

   Task titles MUST start with "Phase N:" — implement uses this
   for sequencing. Number matches the phase from the design.

   Process all phases in one dispatch (subagent has epic-id for all).

7. **Validate task quality** (subagent-trust.md): spot-check that
   created tasks have real file paths (Read 1-2 referenced files),
   acceptance criteria are testable, and approach is specific enough
   for a worker to implement without guessing. Vague tasks → send
   back to subagent with specific feedback.

8. **Finalize:**
   - `work start <epic-id>`
   - Approve source: `work comment <source-id> "Converted to epic <epic-id>"`
   - `work approve <source-id>`

9. **Report:**
   ```
   Epic: <epic-id> — <title>
   Phases: N tasks created

   Next: /implement <epic-id>
   ```

10. **Continuation prompt:**
   Use AskUserQuestion:
   - "Continue to /implement <epic-id>" (Recommended) — description: "Execute implementation tasks"
   - "Review tasks first" — description: "Inspect the created tasks before implementing"
   - "Done for now" — description: "Leave issue active for later /next"

   If user selects "Continue to /implement":
   → Invoke Skill tool: skill="implement", args="<epic-id>"

## Error Handling
- No description content → "Run `/explore` first to generate a design", stop
- `work create` fails for epic → check work CLI available, report error
- Subagent fails on phase → report which phase, continue others, note gap in report
