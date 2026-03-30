---
name: loop
description: "Run a prompt or slash command on a recurring interval (e.g. /loop 5m /foo, defaults to 10m). Use whenever the user wants to set up a recurring task, poll for status, or run something repeatedly on an interval (e.g. 'check the deploy every 5 minutes', 'keep running /babysit', 'poll CI every 10m', 'run this on a schedule')."
argument-hint: "[interval] <prompt or /skill>"
user-invocable: true
allowed-tools:
  - Bash
  - Read
  - CronCreate
  - CronDelete
  - CronList
---

# Loop

Generic cron scheduler. Parses an interval and a command, creates a recurring cron job, and reports the cron ID so the user can cancel it.

Loop owns scheduling only — it knows nothing about the command it runs. The invoked skill owns its own state and lifecycle. If the skill needs to stop the loop, it finds and deletes the cron via `CronList` / `CronDelete`.

## Arguments

`[interval] <prompt or /skill [args...]>`

| Arg | Pattern | Default | Example |
|-----|---------|---------|---------|
| interval | `\d+[smhd]` (first token only) | `10m` | `5m`, `1h`, `30s` |
| command | everything after interval | *(required)* | `/babysit`, `/pr-ci --auto` |

If the first token doesn't match the interval pattern, treat the entire input as the command and use `10m`.

**Examples:**
- `/loop 5m /babysit` — run /babysit every 5 minutes
- `/loop /babysit` — run /babysit every 10 minutes (default)
- `/loop 1h /pr-ci --auto` — run /pr-ci --auto every hour
- `/loop 5m check if the deploy finished` — run a freeform prompt every 5m

## Workflow

### [1] Parse arguments

Split by whitespace. If the first token matches `^\d+[smhd]$`, extract as interval; the rest is the command. Otherwise the entire input is the command, interval defaults to `10m`.

### [2] Convert interval to cron expression

- `Ns` (seconds) → round up to minutes: `ceil(N/60)`, minimum `*/1 * * * *`
- `Nm` where N <= 59 → `*/N * * * *`
- `Nm` where N >= 60 → `0 */H * * *` where H = ceil(N/60)
- `Nh` → `0 */N * * *`
- `Nd` → `0 0 */N * *`

Avoid minute 0 and 30 for fixed-time crons. For `*/N` patterns, the built-in jitter handles load spreading.

### [3] Create the cron

```
CronCreate(
  cron: "<expression>",
  prompt: "<command>"
)
```

### [4] Report to user

Tell them:
- What command will run and at what interval
- The cron job ID
- How to cancel: "say `CronDelete(<id>)` or just ask me to stop it"
- 7-day auto-expiry reminder
