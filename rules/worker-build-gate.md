# Worker Build Gate

When dispatching worker agents, their prompts MUST include a mandatory build verification exit gate. Workers run the project's build/test command and verify compilation passes before reporting completion.

A worker that reports "done" with compilation errors forces the orchestrator to do fixup work — this wastes tokens and breaks the orchestration model. The orchestrator should verify builds after workers complete, but workers should ALSO verify their own builds.

Evidence: multiple workers reported "completed" but left compilation errors. The orchestrator spent ~30-40% of its time fixing worker output. Workers must be self-verifying.
