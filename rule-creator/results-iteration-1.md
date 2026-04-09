# Rule Evaluation Results — Iteration 1

Rules under test:
1. `complexity-gate.md` — Prevent unnecessary abstractions
2. `deduplication-mandate.md` — Search before creating
3. `failure-learning.md` — Codify codebase-specific antipatterns as rules

## Summary Table

| Eval | Rule | with_rule | clean_baseline | Discriminating? |
|------|------|-----------|----------------|-----------------|
| cg-unnecessary-wrapper | complexity-gate | PASS (inline) | PASS (inline) | NO |
| cg-justified-abstraction | complexity-gate | PASS (1 function) | PASS (1 function) | NO |
| cg-manager-class | complexity-gate | PASS (struct+fn) | PASS (struct+fn) | NO |
| dedup-search | deduplication-mandate | PASS (found existing) | PASS (found existing) | NO |
| dedup-keyword | deduplication-mandate | PASS (found existing) | PASS (found existing) | NO |
| dedup-acceptable | deduplication-mandate | PASS (separate helper) | PASS (separate helper) | NO |
| **fl-create-rule** | **failure-learning** | **PASS (rule file)** | **PARTIAL (hook first)** | **YES** |
| fl-no-rule | failure-learning | PASS (decline) | PASS (decline) | NO (expected) |

## Verdicts

### complexity-gate: INCONCLUSIVE
Claude's default behavior already avoids over-abstraction for these straightforward scenarios. The evals test cases where inlining or simple solutions are clearly correct — Claude chooses them naturally.

**Why keep it anyway:** The rule's value is as a guardrail against drift. Workers with less context, different model versions, or edge cases that tempt abstraction may benefit. The "report abstraction count" worker instruction has no baseline equivalent. Can't be proven effective with current evals.

**Next iteration needed:** Evals that genuinely tempt over-engineering — "create a notification system", "build a plugin architecture", "design a caching layer" — where the abstraction-heavy approach is a reasonable default.

### deduplication-mandate: INCONCLUSIVE
Both variants found existing code, but the prompts hint at existing code ("There's an existing src/utils/ directory with helper functions"). This makes the search behavior natural regardless of the rule.

**Why keep it anyway:** Real-world prompts rarely mention existing code. The rule's value is in "add feature X to module Y" where the agent might not think to search for X elsewhere. The with_rule responses were notably more explicit about *why* they refused to create duplicates (citing "deduplication mandate" by name).

**Next iteration needed:** Evals without hints — just "add a slugify function to this project" with no mention of existing utils. Larger mock codebases where the existing implementation isn't obvious.

### failure-learning: EFFECTIVE
Clear behavioral difference on the key eval:
- **with_rule**: Leads with creating `.claude/rules/event-bus-closures.md` with full paths frontmatter, structured content (what went wrong, why non-obvious, what to do instead). Proposes hook as secondary.
- **baseline**: Leads with code fix (useEventBusHandler hook). Mentions "CLAUDE.md rule" as afterthought.

The negative case (typo bug) correctly declines in both variants — no false positives.

## Recommendations

1. **Keep all three rules.** The failure-learning rule is proven effective. The other two serve as guardrails even if current Claude already aligns — they protect against model drift, worker behavior, and edge cases.

2. **Iterate evals for complexity-gate and deduplication-mandate.** Current evals are non-discriminating due to:
   - Complexity-gate: scenarios too straightforward (Claude already chooses simple)
   - Deduplication: prompts too leading (hint at existing code)

3. **Consider the worker angle.** These rules primarily affect subagent/worker behavior, which is harder to eval with `claude -p`. The "report abstraction count" instruction and "search before creating" mandate have their biggest impact in delegated work contexts.
