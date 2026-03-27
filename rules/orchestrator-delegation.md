# Orchestrator Delegation

When the develop skill or any orchestrator is running, the main thread must stay responsive for orchestration. Implementation work (editing files, fixing compilation errors, applying mechanical renames, resolving merge conflicts) should be delegated to worker agents.

The orchestrator's job is: dispatch workers, monitor progress, handle dependencies, integrate results, communicate with the user. If the orchestrator is spending multiple sequential tool calls editing files, it has been pulled into worker-level work.

Exceptions:
- Single-line fixes that take 1-2 tool calls (not worth the subagent overhead)
- Emergency fixes after a crash when no workers are available (pragmatic fallback)

Evidence: In a real session, the orchestrator made 68 direct Edit calls, spent ~15 minutes on a mechanical rename across 9 files, manually resolved merge conflicts, and added diagnostic logging. All of this should have been delegated. The orchestrator was unavailable for user communication during these periods.
