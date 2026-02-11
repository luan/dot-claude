# Retry Limits

After 2 failed attempts at same approach (same command, same strategy), STOP and either:
- Try fundamentally different approach, or
- AskUserQuestion / message team lead

Never loop on infrastructure failures (network, build infra down, service unavailable). Report blocker + wait.
