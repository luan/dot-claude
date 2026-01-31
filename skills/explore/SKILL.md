---
name: explore
description: Deep exploration - gathers context, proposes approaches, writes plan to .agents/plans/
argument-hint: <prompt>
---

# Explore

Spawns subagent to explore codebase and propose implementation approaches.

## Steps

1. `git branch --show-current` → sanitize (`/` → `-`)
2. Generate timestamp `YYYYMMDD-HHMMSS` + slug from prompt
3. Spawn Task tool:
   - subagent_type: "Explore"
   - prompt: agent prompt below
4. Display: plan path + summary + recommendation
5. Ask: "Ready to implement?"

## Agent Prompt

```
Explore codebase for: {prompt}
Project: {pwd}
Branch: {branch}

Tasks:
1. Search codebase - patterns, related code, constraints
2. Identify 2-3 approaches with pros/cons/complexity/key files
3. Recommend one
4. Write concrete task checklist

Output to: .agents/plans/{timestamp}-{slug}.md

**Use compress-prompt techniques** - this file is for AI consumption.

Format:
# Exploration: {topic}
Created: {ISO}
Branch: {branch}

## Request
{verbatim prompt}

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
...

## Open Questions
{or "None"}
```

Return file path + 2-3 sentence summary.
