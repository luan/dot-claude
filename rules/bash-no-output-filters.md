# Bash Tool: No Output Filtering

Never pipe Bash tool output through `tail`, `head`, `grep`/`rg`, or redirect to a file when you need to read the results. The Bash tool captures and returns the complete stdout/stderr — filtering throws away information you may need, forcing redundant re-runs.

- `<test cmd> | tail -40` — WRONG: truncates failures you'll need to debug
- `<test cmd> | rg "FAIL"` — WRONG: discards context around failures
- `<test cmd> > /tmp/output.txt` — WRONG: pointless indirection, tool already has the output
- `<test cmd>` — RIGHT: read the full output from the tool result

If the output is too long, read what the tool returns and extract what you need from it. One unfiltered run beats three filtered re-runs.

Exception: pipelines that *transform* output for a downstream command (not for your own reading) are fine — e.g., `cmd | sort | uniq` when the sorted result is the goal.
