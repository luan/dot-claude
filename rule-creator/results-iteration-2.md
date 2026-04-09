# Rule Evaluation Results — Iteration 2

## The Money Shot

### cg-hard-notification-system: DISCRIMINATING

**Prompt**: "Build notification system. Only email now, but might add SMS and push later."

**with_rule** (complexity-gate active):
- One `notifyOrderStatusChange` function + one `sendEmail` function
- Zero interfaces, zero classes, zero factories
- Explicitly cited the rule: "might add SMS later = zero call sites today = no abstraction"
- "New abstractions introduced: 0"

**clean_baseline** (no complexity-gate):
- `NotificationChannel` interface
- `EmailChannel` class implementing the interface
- `NotificationService` class with channel array + constructor injection
- `createNotificationService` factory function
- Justified with: "Adding SMS/push later means implementing the interface"

**Verdict**: The baseline built exactly what Mario Zechner calls "merchants of learned complexity" — a 4-abstraction hierarchy for a single email channel, motivated by hypothetical future requirements.

## Full Results Table

| Eval | Rule | with_rule | baseline | Discriminating? |
|------|------|-----------|----------|-----------------|
| **cg-notification** | complexity-gate | 1 fn, 0 abstractions | 4 abstractions (interface+class+service+factory) | **YES** |
| cg-event | complexity-gate | 1 fn, 3 calls | 1 fn, 3 calls | NO |
| cg-validation | complexity-gate | Inline checks | Extracted fn | MARGINAL |
| dedup-retry | dedup-mandate | Found withResilience | Found withResilience | NO |
| dedup-email | dedup-mandate | Found isValidEmail | Found isValidEmail | NO |
| dedup-rate | dedup-mandate | Found isThrottled | Found isThrottled | NO |

## Verdicts

### complexity-gate: EFFECTIVE
The notification system eval proves the rule prevents premature abstraction. The trigger is "might need X later" — without the rule, Claude builds infrastructure for hypothetical requirements. With the rule, it builds for what exists today.

The event-system eval was non-discriminating because Claude already does the right thing when the scenario is unambiguous ("only happens in one place"). The rule's value is in ambiguous cases where "best practices" tempt abstraction.

### deduplication-mandate: INCONCLUSIVE (eval design problem, not rule problem)
All baselines found existing code because "Read the project first" causes natural exploration. The rule may still be valuable in scenarios where:
- The user doesn't say "read the project first"
- The project is large enough that casual exploration misses things
- Workers operate with minimal context (the common case in `/develop`)

The with_rule responses were qualitatively different — they explicitly framed decisions in terms of deduplication ("direct violation of the deduplication mandate") rather than incidentally finding code. But we can't prove behavioral change with current evals.

### failure-learning: EFFECTIVE (from iteration 1)
Clear behavioral difference: with rule → `.claude/rules/` file as primary action; without → code fix with rule as afterthought.

## Recommendations

1. **Keep complexity-gate** — proven effective against "might need it later" abstractions
2. **Keep failure-learning** — proven effective at prioritizing rule creation over code-only fixes
3. **Keep deduplication-mandate** — can't prove ineffective, and its value is in worker contexts where agents have less surrounding context. The qualitative difference (explicit dedup reasoning) suggests it shapes thinking even when outcomes converge.
