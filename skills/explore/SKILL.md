---
name: explore
description: "Deep exploration - gathers context, proposes approaches, writes plan to .agents/plans/"
argument-hint: "<prompt>"
---

# Explore

Subagent explores codebase → proposes approaches → writes plan.

## Steps

1. `git branch --show-current` → sanitize (`/` → `-`)
2. Generate `YYYYMMDD-HHMMSS` + slug
3. Spawn Task: subagent_type="Explore", prompt below
4. Display plan path + summary + recommendation
5. Ask "Ready to implement?"

## Agent Prompt

```
Explore: {prompt}
Project: {pwd} | Branch: {branch}

Tasks:
1. Search - patterns, related code, constraints
2. 2-3 approaches: pros/cons/complexity/files
3. Recommend one
4. Task checklist

Output: .agents/plans/{timestamp}-{slug}.md

**Use compress-prompt techniques** - AI consumption.

Format:
# Exploration: {topic}
Created: {ISO} | Branch: {branch}

## Request
{verbatim}

## Context
{files:lines + relevance}

## Approaches

### Option 1: {name} (Recommended)
{desc}
Pros: / Cons: / Complexity: simple|moderate|complex / Files:

### Option 2-3: ...

## Recommendation
{which + why}

## Next Steps
- [ ] {task}

## Open Questions
{or "None"}
```

Return: file path + 2-3 sentence summary.
