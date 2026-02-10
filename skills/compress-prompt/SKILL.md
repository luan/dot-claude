---
name: compress-prompt
description: Use when compressing text for AI consumption - skill files, system prompts, agent instructions. Not for human docs.
context: fork
agent: general-purpose
model: sonnet
---

# Compress Prompt

Reduce tokens in AI-targeted text. **Always use subagents** - never compress on main thread.

## Process

1. For each file/text to compress, spawn Task:
   - subagent_type: "general-purpose"
   - model: "haiku"
   - prompt: include COMPRESSION RULES below + the text
2. Subagent writes compressed output directly to file
3. Main thread reports: file path + before/after token counts

**Measure with:** `./scripts/count-token <file>`

## Subagent Prompt Template

```
Compress this text for AI consumption. Apply these rules:

COMPRESSION RULES:
- Drop articles when obvious ("the file" → "file")
- Drop filler phrases ("In order to" → delete, "Make sure to" → delete)
- Imperative voice ("You should run" → "Run")
- Use symbols where clear ("results in" → "→", "and" → "+")
- Condense repetitive lists (keep structure, merge similar items)
- Merge redundant examples (4 similar examples → 1 representative + "etc")
- Keep headers but simplify ("## Step 1: Setup" → "## Setup")

PRESERVE EXACTLY:
- Commands: syntax, flags, args
- Code blocks: unless clearly redundant
- File paths
- Keywords: errors, APIs, technical terms
- Distinct cases with different behavior

TEXT TO COMPRESS:
{text}

Write compressed version to: {output_path}
```

## Parallel Compression

Multiple files → spawn multiple subagents in single message for parallel execution.
