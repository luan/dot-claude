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

Convert user feedback into ONE bead with phased design field — directly consumable by `/prepare`.
Does NOT implement — creates actionable work items for later scheduling.

## Workflow

### 1. Gather Context (Parallel)

Run these in parallel to understand what was recently implemented:
```bash
git diff --name-only HEAD~3..HEAD
git log --oneline -5
git branch --show-current
```

If user references specific files, read those files.

### 2. Analyze Feedback

Break feedback ($ARGUMENTS) into individual findings:
- Classify each: `bug`, `task`, or `feature`
- Set priority (P0-P4):
  - P0: Critical bugs, blocking issues
  - P1: Important bugs, high-priority features
  - P2: Normal priority (default for most feedback)
  - P3: Nice-to-have improvements
  - P4: Low priority, future consideration
- Group findings by type for phase structure

### 3. Create Single Bead with Phased Design

Create ONE task bead containing all findings:

```bash
bd create "Fix: <brief-summary-of-feedback>" --type task --priority 2 \
  --description "$(cat <<'EOF'
## Acceptance Criteria
- All feedback items addressed
- Findings stored in design field as phased structure
- Consumable by /prepare for epic creation
EOF
)"
```

Validate: `bd lint <id>` — fix violations if needed.
Mark in progress: `bd update <id> --status in_progress`

Then structure findings as phases in the design field:

```bash
bd update <id> --design "$(cat <<'EOF'
## Feedback Analysis

**Phase 1: Bug Fixes**
1. Fix X in file.ts:123 — description of bug
2. Fix Y in module.ts:45 — description of bug

**Phase 2: Improvements**
3. Update Z configuration — description of improvement
4. Add W feature — description of feature

Each phase groups findings by type (bugs first, then tasks, then features). Skip empty phases.
EOF
)"
```

**Phase grouping rules:**
- Phase 1: Bugs (highest priority first)
- Phase 2: Tasks / improvements
- Phase 3: Features / new functionality
- Skip phases with no findings
- Each item: actionable title with file:line when available

### 4. Report

Output format:
```
## Fix Issue: #<id>

**Findings**: N items (X bugs, Y tasks, Z features)

**Next**: `bd edit <id> --design` to review findings, `/prepare <id>` to create epic with tasks.
```

## Error Handling
- No beads CLI → tell user to install, stop
- `bd create` fails → show error, retry once, then report
- Ambiguous feedback → AskUserQuestion for clarification before creating issues
