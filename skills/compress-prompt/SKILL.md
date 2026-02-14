---
name: compress-prompt
description: Compress text for AI consumption — skill files, system prompts, agent instructions. Triggers: "compress", "reduce tokens", "shrink prompt", "make it shorter for AI". Not for human docs.
context: fork
agent: general-purpose
model: sonnet
---

# Compress Prompt

Reduce tokens in AI-targeted text. **Always use subagents** - never compress on main.

## Process

1. Per file/text, spawn Task:
   - subagent_type: "general-purpose"
   - model: "haiku"
   - prompt: include COMPRESSION RULES below + text
2. Subagent writes compressed output directly to file
3. Main reports: file path + before/after token counts

**Measure with:** `./scripts/count-token <file>`

## Subagent Prompt Template

```
Compress this text for AI consumption. Apply rules:

COMPRESSION RULES:
- Drop articles when obvious ("the file" → "file")
- Drop filler ("In order to" → delete, "Make sure to" → delete)
- Imperative voice ("You should run" → "Run")
- Use symbols where clear ("results in" → "→", "and" → "+")
- Condense repetitive lists (keep structure, merge similar)
- Merge redundant examples (4 similar → 1 representative + "etc")
- Keep headers but simplify ("## Step 1: Setup" → "## Setup")

PRESERVE EXACTLY (byte-for-byte):
- YAML frontmatter: entire `---` block at top, including delimiters
- Commands: syntax, flags, args
- Code blocks: unless clearly redundant
- File paths
- Keywords: errors, APIs, technical terms
- Distinct cases with different behavior

OUTPUT RULES:
- Output ONLY compressed text. No preamble, no commentary, no wrapper.
- If file starts with `---`, output must start with `---`.

TEXT TO COMPRESS:
{text}

Write compressed version to: {output_path}
```

## Parallel Compression

Multiple files → spawn multiple subagents in single message for parallel execution.
