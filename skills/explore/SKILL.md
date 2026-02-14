---
name: explore
description: "Triggers: 'explore', 'how does X work', 'understand', 'research', 'plan a feature', 'figure out', 'investigate', 'design', 'architect'"
argument-hint: "<prompt> [--continue]"
user-invocable: true
allowed-tools:
  - Task
  - Skill
  - AskUserQuestion
  - Bash
---

# Explore

Research, investigate, design. Findings stored in beads design field.
Auto-escalates to team for complex multi-system work.

**IMMEDIATELY dispatch to subagent.** Never explore on main thread.

## Context

Active epics: !`bd list --status in_progress --type epic -q 2>/dev/null`

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Multiple viable approaches with unclear tradeoffs → ask which direction to pursue
- Domain ambiguity (business logic interpretation) → clarify with user before deep-diving

Do NOT ask when the answer is obvious or covered by the task brief.

## Instructions

### New Exploration

1. Create bead:
   ```bash
   bd create "Explore: <topic>" --type task --priority 2 \
     --description "$(cat <<'EOF'
   ## Acceptance Criteria
   - Findings stored in bead design field
   - Structured as Current State, Recommendation, and phased Next Steps
   - Each phase includes file paths and is independently actionable
   EOF
   )"
   ```
2. Validate: `bd lint <id>` — if it fails, `bd edit <id> --description` to fix
3. `bd update <id> --status in_progress`

4. Dispatch via Task (subagent_type="codebase-researcher"):

```
Research <topic> thoroughly. Return COMPLETE findings as text
(do NOT write files, do NOT create beads).

## Job
1. Explore codebase
4. Design approach — 2-3 options, choose best

## Output Structure

1. **Current State**: What exists now (files, patterns, architecture)
2. **Recommendation**: Chosen approach with rationale
3. **Key Files**: Exact paths of files to modify/create
4. **Risks**: What could go wrong, edge cases
5. **Next Steps**: Phased implementation plan using format:

**Phase N: <title>**
Files: exact/path/to/file.ts, exact/path/to/other.ts
Approach: <what to change and why>
1. <specific step>
2. <specific step>

Each phase must include file paths and approach hints —
downstream task creation depends on this detail.
```

5. Store findings: `bd update <id> --design "<full-findings>"`

6. Output summary:
```
Explore: <bead-id> — <topic>
Problem: <1 sentence>
Recommendation: <1 sentence>

Phases:
1. <title> — <key files>
2. <title> — <key files>

Key decisions:
- <why this approach>

Next: /prepare <bead-id>
```

7. → See Continuation Prompt below.

### Continuation (--continue flag)

1. Resolve issue ID:
   - If $ARGUMENTS matches a beads ID → use it
   - If --continue → `bd list --status in_progress --type task`,
     find first with title starting "Explore:"
2. Load existing: `bd show <id> --json` → extract design field
3. Dispatch subagent with: "Previous findings:\n<design>\n\n
   Continue exploring: <new prompt>"
4. Update: `bd update <id> --design "<combined>"`
5. Output updated summary

6. → See Continuation Prompt below.

### Continuation Prompt

Use AskUserQuestion:
- "Continue to /prepare <bead-id>" (Recommended) — description: "Create epic + implementation tasks from findings"
- "Re-explore with different focus" — description: "Investigate a different angle on the same topic"
- "Done for now" — description: "Leave bead in_progress for later /resume-work"

If user selects "Continue to /prepare":
→ Invoke Skill tool: skill="prepare", args="<bead-id>"

If user selects "Re-explore":
→ Ask what to focus on, then re-run from step 4 with updated prompt

### Escalation

Subagent returns "ESCALATE: team — [reason]":

Spawn 2-3 Task agents in parallel:
- **Researcher** (model: sonnet): breadth-first investigation
- **Architect** (model: opus): design analysis, tradeoffs
- **Devil's Advocate** (model: opus): challenges assumptions

Each investigates independently, returns findings text.
Synthesize into unified findings, store in design field.

Escalation triggers:
- 3+ viable approaches, unclear tradeoffs
- Spans 3+ independent subsystems
- Cross-cutting concerns needing adversarial analysis

## Key Rules

- Main thread does NOT explore — subagent does
- Findings stored in beads design field
- `bd lint` after bead creation — fix if it fails
- Next Steps must include file paths (prepare depends on it)
