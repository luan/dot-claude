---
name: compress-prompt
description: Use when compressing text for AI consumption - skill files, system prompts, agent instructions. Not for human docs.
---

# Compress Prompt

Reduce tokens in AI-targeted text. **Always use subagents** - never compress on main thread.

## Process

1. For each file/text to compress, spawn Task:
   - subagent_type: "general-purpose"
   - prompt: include COMPRESSION RULES below + the text
2. Subagent writes compressed output directly to file
3. Main thread reports: file path + before/after token counts

**Measure with:** `./scripts/count-token <file>`

## Subagent Prompt Template

```
Compress this text for AI consumption. Apply these rules:

COMPRESSION RULES:
- Strip markdown formatting (bold, headers, rules)
- Drop articles ("the file" → "file")
- Drop filler ("In order to", "Make sure to" → delete)
- Imperative voice ("You should run" → "Run")
- Use symbols ("results in" → "→")
- Condense lists ("- a\n- b" → "a, b" or "a/b")
- Merge similar examples (4 examples → 1 + "etc")
- Tables → inline ("key:val, key:val")
- Headers → context ("## Step 1: Foo" → "1. Foo:")

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
