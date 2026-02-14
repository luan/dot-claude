# Retry Limits

After 2 failed attempts at same approach (same command, same strategy), STOP and either:
- Try fundamentally different approach, or
- AskUserQuestion / message team lead

Never loop on infrastructure failures (network, build infra down, service unavailable). Report blocker + wait.

## Worker Fix Loops
- Max 3 fix iterations per failure (total, not per approach)
- >10 tool calls on single fix → checkpoint findings + escalate to caller
- After 3+ completed tasks in one session → report status before starting next
