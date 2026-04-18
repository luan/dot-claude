No, a rule isn't the right fix here. A few reasons:

1. **It's a typo, not a knowledge gap.** Rules are effective for non-obvious conventions or project-specific decisions. Claude (and any developer) already knows `==` vs `=` — a rule won't prevent a future typo any more than knowing the difference prevented this one.

2. **Linters catch this reliably.** Most languages have lint rules for assignment-in-conditional (`no-cond-assign` in ESLint, `-Wparentheses` in GCC/Clang, etc.). If your project doesn't have one enabled, that's the fix — it catches it at build time, every time, for humans and AI alike.

3. **Rules have a cost.** Every rule added to CLAUDE.md consumes context and attention. Spending that budget on things a linter handles for free dilutes the rules that actually matter.

**What to do instead:** enable the relevant lint rule in your project config. That's a permanent, automated fix at the right layer.
