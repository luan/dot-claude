---
name: token-stats
description: "Show token usage, cost breakdown, cache efficiency, and context fill. Triggers: '/token-stats', 'how much am I spending', 'cost report', 'burn rate', 'token usage', 'cache stats'."
argument-hint: "[--block] [--daily]"
user-invocable: true
allowed-tools:
  - "Bash(ccusage blocks:*)"
  - "Bash(ccusage daily:*)"
  - "Bash(ccusage session:*)"
  - "Bash(cat /tmp/claude-context-pct-*)"
---

# Token Stats

Surface session cost data, cache efficiency, and context fill on demand.
Read-only — no file writes, no git operations.

## Modes

Check the invocation argument:

- No args → **Default** (compact summary)
- `--block` → **Block detail** view
- `--daily` → **Daily** view (last 7 days)

---

## Default mode (no args)

Run:

```bash
ccusage blocks --active --json
```

Parse `blocks[0]` from the JSON output.

**If the array is empty or the command returns no active block**, output:

```
No active billing block.
```

and stop.

**Otherwise**, extract these fields from `blocks[0]`:

| Field | Path in JSON |
|---|---|
| Block cost | `costUSD` |
| Burn rate | `burnRate.costPerHour` |
| Projected total | `projection.totalCost` |
| Time remaining | `projection.remainingMinutes` |
| Cache reads | `tokenCounts.cacheReadInputTokens` |
| Cache writes | `tokenCounts.cacheCreationInputTokens` |

Compute cache efficiency:

```
efficiency = tokenCounts.cacheReadInputTokens / (tokenCounts.cacheReadInputTokens + tokenCounts.cacheCreationInputTokens) * 100
```

If the denominator is zero, show `—` instead of a percentage.

Also run:

```bash
cat /tmp/claude-context-pct-${CLAUDE_SESSION_ID}
```

That file contains a single integer (0–100) representing context fill percentage.
If the file is missing or the command errors, show `—` for context fill.

Format output as:

```
Token Stats
───────────────────────────────
Block cost:       $X.XXXX
Burn rate:        $X.XX/hr
Projected total:  $X.XXXX
Time remaining:   Xm

Cache efficiency: XX%  (reads / (reads + writes))
Context fill:     XX%
```

**Alert logic** (append after the table, only if triggered):

- If context fill > 80%:
  `⚠  Context above 80% — consider running /handoff before compaction.`
- If burn rate > $50/hr:
  `⚠  Burn rate above $50/hr — unusually high spend.`

---

## --block mode

Run:

```bash
ccusage blocks --active --json
```

Parse `blocks[0]`.
If no active block, output `No active billing block.` and stop.

Show a detailed view including:

- All fields from default mode
- Models used: list `blocks[0].models` (array of model name strings)
- Total input tokens (`tokenCounts.inputTokens`)
- Total output tokens (`tokenCounts.outputTokens`)
- Cache read tokens (`tokenCounts.cacheReadInputTokens`)
- Cache write tokens (`tokenCounts.cacheCreationInputTokens`)
- Entry count (`entries` — integer, display directly)

Format as a structured report with labelled sections:
**Summary**, **Tokens**, **Cache**, **Models**.

---

## --daily mode

Run:

```bash
ccusage daily --json
```

Take the last 7 entries from the returned array (most recent first, or sort by date descending).

For each day, compute cache hit rate:

```
hit_rate = cacheReadTokens / (cacheReadTokens + cacheCreationTokens) * 100
```

Display as a table:

```
Date         Cost      Cache Hit
──────────────────────────────────
2026-04-01   $X.XXXX   XX%
2026-03-31   $X.XXXX   XX%
...
```

If fewer than 7 days of data exist, show what is available.
If no data is returned, output `No daily data available.`

---

## Output rules

- Plain text only, no markdown rendering in output.
- Compact tables using spaces and `─` separators.
- Dollar amounts: 4 decimal places for costs under $1, 2 decimal places for $1 and above.
- Percentages: round to nearest integer.
- Minutes remaining: round to nearest integer, display as `Xm`.
- Never write files. Never run git commands.
