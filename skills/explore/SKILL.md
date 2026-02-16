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

Research, investigate, design. Findings stored in issue description.
Auto-escalates to team for complex multi-system work.

**IMMEDIATELY dispatch to subagent.** Never explore on main thread.

## Context

Active parent issues: !`work list --status active --roots --format short 2>/dev/null`

## Mid-Skill Interviewing

Use AskUserQuestion when facing genuine ambiguity during execution:
- Multiple viable approaches with unclear tradeoffs → ask which direction to pursue
- Domain ambiguity (business logic interpretation) → clarify with user before deep-diving

Do NOT ask when the answer is obvious or covered by the task brief.

## Instructions

### New Exploration

1. Create work issue:
   ```bash
   work create "Explore: <topic>" --type chore --priority 2 \
     --labels explore \
     --description "$(cat <<'EOF'
   ## Acceptance Criteria
   - Findings stored in issue description
   - Structured as Current State, Recommendation, and phased Next Steps
   - Each phase includes file paths and is independently actionable
   EOF
   )"
   ```
2. `work start <id>`

3. Dispatch via Task (subagent_type="codebase-researcher"):

```
Research <topic> thoroughly. Return COMPLETE findings as text
(do NOT write files, do NOT create work issues).

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

4. **Validate findings** (subagent-trust.md): spot-check ALL
   architectural claims + 50% of file/behavioral claims before storing.
   If echo suspected or key claims fail → send targeted follow-up.

5. Store findings: `work edit <id> --description "<full-findings>"`
6. Submit for review: `work review <id>`

7. Output summary:
```
Explore: <issue-id> — <topic>
Problem: <1 sentence>
Recommendation: <1 sentence>

Phases:
1. <title> — <key files>
2. <title> — <key files>

Key decisions:
- <why this approach>

Next: /prepare <issue-id>
```

8. → See Continuation Prompt below.

### Continuation (--continue flag)

1. Resolve issue ID:
   - If $ARGUMENTS matches a work ID → use it
   - If --continue → `work list --status review --label explore`
     or `work list --status active --label explore`, use first result
2. Load existing: `work show <id> --format=json` → extract description
3. Move back to active: `work start <id>`
4. Dispatch subagent with: "Previous findings:\n<description>\n\n
   Continue exploring: <new prompt>"
5. Update: `work edit <id> --description "<combined>"`
6. Submit for review: `work review <id>`
7. Output updated summary

8. → See Continuation Prompt below.

### Continuation Prompt

Use AskUserQuestion:
- "Continue to /prepare <issue-id>" (Recommended) — description: "Create epic + implementation tasks from findings"
- "Re-explore with different focus" — description: "Investigate a different angle on the same topic"
- "Done for now" — description: "Leave issue active for later /resume-work"

If user selects "Continue to /prepare":
→ Invoke Skill tool: skill="prepare", args="<issue-id>"

If user selects "Re-explore":
→ Ask what to focus on, then re-run from step 3 with updated prompt

### Escalation

Subagent returns "ESCALATE: team — [reason]":

Spawn 2-3 Task agents in parallel:
- **Researcher** (model: opus): breadth-first investigation
- **Architect** (model: opus): design analysis, tradeoffs
- **Devil's Advocate** (model: opus): challenges assumptions

Each investigates independently, returns findings text.
Synthesize into unified findings, store in description.

Escalation triggers:
- 3+ viable approaches, unclear tradeoffs
- Spans 3+ independent subsystems
- Cross-cutting concerns needing adversarial analysis

## Key Rules

- Main thread does NOT explore — subagent does
- Findings stored in issue description
- Next Steps must include file paths (prepare depends on it)
