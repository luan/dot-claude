# Cron Over Sleep

When monitoring, polling, or waiting for external state (CI checks, PR merges, deploy status), use `CronCreate` to set up periodic checks instead of `sleep` loops in Bash. Sleep blocks the session and wastes tokens on idle time; cron jobs run independently and notify when something changes.

Bad: `sleep 300 && gh pr checks ...` in a loop
Good: `CronCreate` with a 5-minute interval that checks status and acts on failures
