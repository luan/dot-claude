# Subagent Trust — Adversarial by Default

Parent agents do NOT implicitly trust subagent output. Subagents are
useful but prone to: shallow analysis, premature conclusions, missing
edge cases, and confident-sounding wrong answers.

## Validation Protocol

After receiving subagent results, BEFORE acting on them:

1. **Spot-check claims.** Verify specific assertions (file paths,
   function signatures, behavioral claims) with Read/Grep.
   Scale to scope: 1-2 checks for small tasks, ALL architectural
   claims + 50% of behavioral claims for explore findings driving
   epics. Any failed check → re-examine all conclusions.

2. **Challenge simple answers.** If the conclusion is "just do X" for
   a non-trivial problem, push back. Ask: what alternatives were
   considered? What could go wrong? What was ruled out and why?

3. **Reject unsupported recommendations.** "Best approach is X" without
   evidence of alternatives evaluated = insufficient. Send back with
   explicit request to compare approaches.

4. **Watch for echo.** Subagents tend to confirm the framing they were
   given. If findings suspiciously match initial assumptions, ask one
   targeted follow-up about the most likely point of friction before
   accepting.

## Red Flags (require deeper validation)

- Output is disproportionately short relative to scope
- Fewer files examined than the scope warrants
- No tradeoffs or risks mentioned
- Conclusion matches the most obvious approach with no nuance
- "No issues found" on non-trivial code

## What NOT to Do

- Don't re-do the subagent's entire job (defeats the purpose)
- Don't block on validation for raw tool output (file listings, grep
  matches, build logs) with no interpretive conclusions attached.
  Any synthesis or recommendation requires validation regardless of
  how it's presented.
- Don't add validation ceremony to haiku-tier tasks (commits,
  compression)
- Implementation results verified by build gate — spot-check not
  required for implement completion reports

## Applies To

All skills that dispatch subagents and receive conclusions: explore
findings, review lenses, prepare task structures, fix issue creation,
debugging diagnoses.

Skills should reference this rule at their consolidation/synthesis
step: "Apply subagent trust validation before acting on results."
