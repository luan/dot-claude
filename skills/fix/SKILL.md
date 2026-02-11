---
name: fix
description: "Convert user feedback into beads issues. Does NOT implement fixes. Triggers: 'fix', 'create issues from feedback', 'file bugs from feedback'."
argument-hint: "<feedback-text>"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
---

# Fix

Feedback → beads converter. Creates classified issues.
Does NOT implement — use `/implement` after.

## Context

Branch: !`git branch --show-current`
Recent changes: !`git diff --name-only HEAD~3..HEAD 2>/dev/null`
Recent commits: !`git log --oneline -5 2>/dev/null`

## Steps

1. **Analyze feedback ($ARGUMENTS):**
   - Break into individual findings
   - Classify type: `bug`, `task`, or `feature`
   - Set priority:
     - P1: Urgent/blocking
     - P2: High
     - P3: Normal (default)
     - P4: Low

3. **Create beads for each finding:**
   - `bd create "<title>" --type <type> --priority <N>`
   - Bug type: include `## Steps to Reproduce` + `## Acceptance Criteria` in description
   - Task/feature: include `## Acceptance Criteria`
   - `bd lint <id>` after each create

4. **Report:**
   ```
   ## Created Issues
   - [bd-xxx] Fix X in file.ts:123 (bug, P2)
   - [bd-yyy] Add Y feature (task, P3)

   Next: `/implement <id>` or `/prepare` to structure
   ```
